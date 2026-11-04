//! Event-backed queries. Allocator callbacks never acquire AsyncTracker locks.

use super::{get_global_tracker, AsyncTracker};
use crate::capture::backends::async_types::{AsyncSnapshot, AsyncStats, TaskInfo};
use crate::capture::backends::profile_replay::{self, ProfileMetadata, RebuiltProfiles};
use crate::event_store::SharedEventStore;

/// Get current memory usage from the same projection as profiles and stats.
pub fn get_memory_snapshot() -> crate::capture::backends::async_types::AsyncMemorySnapshot {
    let stats = get_global_tracker()
        .map(|tracker| tracker.get_stats())
        .unwrap_or_default();
    crate::capture::backends::async_types::AsyncMemorySnapshot {
        active_task_count: stats.active_tasks,
        total_allocated_bytes: stats.total_memory as u64,
        allocation_events: stats.total_allocations as u64,
        events_dropped: 0,
        buffer_utilization: 0.0,
    }
}

impl AsyncTracker {
    pub(crate) fn with_event_store(event_store: SharedEventStore) -> Self {
        let mut tracker = Self::new();
        tracker.event_store = Some(event_store);
        tracker
    }

    pub(super) fn rebuilt(&self) -> Option<(u64, RebuiltProfiles)> {
        let _guard = crate::core::allocator::TrackingGuard::acquire();
        let events = self.event_store.as_ref()?.snapshot();
        let graph = crate::task_registry::global_registry().export_graph();
        let metadata = ProfileMetadata::capture(&graph, Some(self));
        let captured_at = profile_replay::capture_cut_ns();
        Some((
            captured_at,
            profile_replay::rebuild_at(&events, &graph, &metadata, captured_at),
        ))
    }
}

pub(super) fn stats(rebuilt: &RebuiltProfiles) -> AsyncStats {
    AsyncStats {
        total_tasks: rebuilt.profiles.len(),
        active_tasks: rebuilt
            .profiles
            .iter()
            .filter(|p| !p.is_completed())
            .count(),
        total_allocations: rebuilt.total_allocations as usize,
        total_memory: rebuilt.total_memory as usize,
        total_deallocations: rebuilt
            .profiles
            .iter()
            .map(|p| p.total_deallocations as usize)
            .sum(),
        total_deallocated: rebuilt.total_memory.saturating_sub(rebuilt.current_memory),
        active_memory: rebuilt.current_memory as usize,
        peak_memory: rebuilt.global_peak_memory as usize,
    }
}

pub(super) fn snapshot(timestamp: u64, rebuilt: RebuiltProfiles) -> AsyncSnapshot {
    let stats = stats(&rebuilt);
    let tasks = rebuilt
        .profiles
        .iter()
        .filter(|p| !p.is_completed())
        .map(|p| TaskInfo {
            task_id: p.task_id,
            name: p.task_name.clone(),
            thread_id: p.thread_id_u64,
            created_at: p.created_at_ms.saturating_mul(1_000_000),
            active_allocations: rebuilt
                .live_allocations
                .iter()
                .filter(|a| a.task_id == p.task_id)
                .count(),
            total_memory: p.current_memory as usize,
        })
        .collect();
    AsyncSnapshot {
        timestamp,
        tasks,
        allocations: rebuilt.live_allocations,
        stats,
    }
}

pub(super) fn local_snapshot(tracker: &AsyncTracker) -> AsyncSnapshot {
    let allocations: Vec<_> = match tracker.allocations.lock() {
        Ok(allocations) => allocations.values().flatten().cloned().collect(),
        Err(_) => {
            tracing::error!("Failed to acquire allocations lock in snapshot");
            return AsyncSnapshot::default();
        }
    };
    let tasks = tracker
        .lifecycle_profiles()
        .into_iter()
        .filter(|p| !p.is_completed())
        .map(|p| TaskInfo {
            task_id: p.task_id,
            name: p.task_name,
            thread_id: p.thread_id_u64,
            created_at: p.created_at_ms.saturating_mul(1_000_000),
            active_allocations: allocations
                .iter()
                .filter(|a| a.task_id == p.task_id)
                .count(),
            total_memory: p.current_memory as usize,
        })
        .collect();
    AsyncSnapshot {
        timestamp: AsyncTracker::now(),
        tasks,
        allocations,
        stats: tracker.get_stats(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::{EventStore, MemoryEvent};
    use std::sync::Arc;

    /// Objective: Converting a Tokio ID during context inspection is not workload memory.
    /// Invariants: Context retains identity while the allocator event count is unchanged.
    #[tokio::test]
    async fn context_capture_does_not_publish_memory() {
        tokio::spawn(async {
            let store = crate::tracker::allocator_events::global_tracker().event_store();
            let before = store.snapshot().len();
            let context = super::super::TrackerContext::capture();
            let after = store.snapshot().len();
            assert!(
                context.tokio_task_id.is_some(),
                "probe must run in a real Tokio task"
            );
            assert_eq!(
                after, before,
                "context inspection must not publish ID-formatting allocations"
            );
        })
        .await
        .expect("context probe exits normally");
    }

    /// Objective: Local query timing advances without changing accounting or TLS.
    /// Invariants: Completion time is idempotent and freed rows are not counted live.
    #[test]
    fn standalone_queries_preserve_lifecycle() {
        let tracker = AsyncTracker::new();
        tracker
            .track_task_start_with_tokio(401, 5, "local".into(), std::thread::current().id())
            .unwrap();
        tracker.track_allocation(4096, 128, 401);
        tracker.track_allocation(8192, 64, 401);
        tracker.track_deallocation(4096);
        tracker
            .profiles
            .lock()
            .unwrap()
            .get_mut(&401)
            .unwrap()
            .created_at_ms -= 50;
        let profile = tracker.get_task_profile(401).unwrap();
        assert!(
            profile.duration_ns >= 50_000_000,
            "query advances a running task's time"
        );
        assert!(
            profile.allocation_rate > 0.0,
            "derive rate from query cutoff"
        );
        let snapshot = tracker.snapshot();
        assert_eq!(
            snapshot.tasks[0].active_allocations, 1,
            "free removes one live row"
        );
        assert_eq!(
            snapshot.stats.active_memory, 64,
            "free leaves other binding intact"
        );
        tracker.track_task_end(401).unwrap();
        let completed = tracker.get_task_profile(401).unwrap();
        let mut copy = completed.clone();
        copy.completed_at_ms = Some(copy.created_at_ms);
        copy.mark_completed();
        assert_eq!(
            copy.completed_at_ms,
            Some(copy.created_at_ms),
            "same-ms completion is not a sentinel"
        );
        let parent = crate::task_registry::TaskIdRegistry::current_task_id();
        drop(tracker);
        assert_eq!(
            crate::task_registry::TaskIdRegistry::current_task_id(),
            parent,
            "instance destruction cannot clear unrelated context"
        );
    }

    /// Objective: Query all public surfaces after moved realloc and an external free.
    /// Invariants: Independent instances never borrow another source's accounting.
    #[test]
    fn connected_queries_follow_their_source() {
        let source = Arc::new(EventStore::new());
        let tracker = AsyncTracker::with_event_store(source.clone());
        let standalone = AsyncTracker::new();
        let task_id = 9_999_000;
        tracker
            .track_task_start_with_tokio(task_id, 17, "source".into(), std::thread::current().id())
            .unwrap();
        standalone
            .track_task_start_with_tokio(task_id, 18, "local".into(), std::thread::current().id())
            .unwrap();
        standalone.track_allocation(8192, 64, task_id);
        let mut allocation = MemoryEvent::allocate(4096, 128, 1);
        allocation.task_id = Some(task_id);
        allocation.binding_serial = Some(11);
        source.record(allocation);
        let mut resize = MemoryEvent::reallocate_moved(4096, 16384, 128, 4224, 1);
        resize.task_id = None;
        resize.binding_serial = Some(11);
        source.record(resize);
        let snapshot = tracker.snapshot();
        assert_eq!(
            snapshot.stats.active_memory, 4224,
            "snapshot totals must include realloc"
        );
        assert_eq!(
            snapshot.allocations[0].ptr, 16384,
            "snapshot must follow moved binding"
        );
        let mut release = MemoryEvent::deallocate(16384, 4224, 1);
        release.task_id = None;
        release.binding_serial = Some(11);
        source.record(release);
        tracker.track_task_end(task_id).unwrap();
        let profile = tracker.get_task_profile(task_id).unwrap();
        assert_eq!(
            profile.current_memory, 0,
            "external release must update profile"
        );
        assert_eq!(
            profile.total_deallocations, 1,
            "realloc is not an extra final release"
        );
        assert_eq!(profile.peak_memory, 4224, "retain realloc peak after free");
        assert_eq!(
            profile.tokio_task_id,
            Some(17),
            "retain live lifecycle metadata"
        );
        assert!(
            tracker.snapshot().allocations.is_empty(),
            "snapshot must drop released rows"
        );
        assert_eq!(
            tracker.get_stats().active_memory,
            0,
            "stats and profiles agree"
        );
        assert_eq!(
            standalone.get_task_profile(task_id).unwrap().current_memory,
            64,
            "standalone source stays independent"
        );
    }
}
