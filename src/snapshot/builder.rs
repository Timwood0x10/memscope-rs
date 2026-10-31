//! Snapshot builder - Reusable snapshot construction logic.
//!
//! This module provides the standalone entry point that turns the raw event
//! stream into a [`MemorySnapshot`]. All binding matching, generation numbering,
//! reallocation handling and byte accounting live in
//! [`crate::snapshot::bindings::BindingLedger`], which is also what the dashboard
//! reconstruction uses. Keeping one implementation is what makes the snapshot,
//! the dashboard and the JSON exports agree on the same event stream.

use crate::core::types::TrackKind;
use crate::event_store::MemoryEvent;
use crate::snapshot::bindings::BindingLedger;
use crate::snapshot::types::{ActiveAllocation, MemorySnapshot, SnapshotBinding};

/// Build a memory snapshot from a slice of events.
///
/// # Accuracy
///
/// * `total_allocated` / `total_deallocated` / `peak_memory` come from the replayed
///   transitions, so they stay correct even after releases have been applied.
/// * A release closes exactly one binding (matched by address and recorded size),
///   so a late release of an older generation cannot delete a newer live binding.
/// * `active_bindings` retains all identities, including overlapping generations.
///   The compatibility address lookup returns the newest binding only.
pub fn build_snapshot_from_events(events: &[MemoryEvent]) -> MemorySnapshot {
    let mut ledger = BindingLedger::new();
    for event in events {
        ledger.apply(event);
    }

    project_snapshot(&ledger, MemorySnapshot::new().timestamp)
}

/// Project the same reduced capture cut used by another report consumer.
pub(crate) fn project_snapshot(ledger: &BindingLedger, timestamp: u64) -> MemorySnapshot {
    let mut snapshot = MemorySnapshot::new();
    snapshot.timestamp = timestamp;
    let mut bindings = Vec::with_capacity(ledger.live().len());

    for binding in ledger.live() {
        let active = ActiveAllocation {
            ptr: Some(binding.ptr),
            size: binding.size,
            kind: TrackKind::HeapOwner {
                ptr: binding.ptr,
                size: binding.size,
            },
            allocated_at: binding.allocated_at,
            var_name: binding.var_name.clone(),
            type_name: binding.type_name.clone(),
            thread_id: binding.thread_id,
            call_stack_hash: binding.call_stack_hash,
            module_path: binding.module_path.clone(),
            stack_ptr: binding.stack_ptr,
        };
        snapshot
            .active_allocations
            .insert(binding.ptr, active.clone());
        bindings.push(SnapshotBinding {
            generation_id: binding.generation,
            binding_serial: binding.binding_serial,
            allocation: active,
        });
    }

    snapshot.active_bindings = Some(bindings);

    let stats = ledger.stats();
    snapshot.stats.total_allocations =
        usize::try_from(stats.allocation_count).unwrap_or(usize::MAX);
    snapshot.stats.total_deallocations =
        usize::try_from(stats.deallocation_count).unwrap_or(usize::MAX);
    snapshot.stats.total_reallocations =
        usize::try_from(stats.reallocation_count).unwrap_or(usize::MAX);
    snapshot.stats.unmatched_deallocations =
        usize::try_from(stats.unmatched_deallocation_count).unwrap_or(usize::MAX);
    snapshot.stats.total_allocated = usize::try_from(stats.allocated_bytes).unwrap_or(usize::MAX);
    snapshot.stats.total_deallocated =
        usize::try_from(stats.deallocated_bytes).unwrap_or(usize::MAX);
    snapshot.stats.current_memory = usize::try_from(stats.current_bytes).unwrap_or(usize::MAX);
    snapshot.stats.peak_memory = usize::try_from(stats.peak_bytes).unwrap_or(usize::MAX);
    snapshot.stats.active_allocations = snapshot.active_count();
    snapshot.thread_stats = ledger.thread_stats().clone();

    snapshot
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify an empty event stream produces an empty snapshot.
    /// Invariants: no active allocation and zero bytes.
    #[test]
    fn test_build_snapshot_empty() {
        let events: Vec<MemoryEvent> = vec![];
        let snapshot = build_snapshot_from_events(&events);
        assert_eq!(snapshot.active_count(), 0, "no binding may be active");
        assert_eq!(snapshot.current_memory(), 0, "no bytes may be live");
        assert_eq!(snapshot.peak_memory(), 0, "no peak may be recorded");
    }

    /// Objective: Verify allocations are visible with their totals.
    /// Invariants: two bindings are active and current memory is their sum.
    #[test]
    fn test_build_snapshot_with_allocations() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 1024, 1),
            MemoryEvent::allocate(0x2000, 2048, 1),
        ];
        let snapshot = build_snapshot_from_events(&events);
        assert_eq!(snapshot.active_count(), 2, "both bindings must be active");
        assert_eq!(snapshot.current_memory(), 3072, "current bytes must sum up");
        assert_eq!(
            snapshot.stats.total_allocated, 3072,
            "total allocated bytes must be recorded"
        );
        assert_eq!(
            snapshot.stats.peak_memory, 3072,
            "peak must be the high-water mark of the replay"
        );
    }

    /// Objective: Verify releases clear the live set and the ledger.
    /// Invariants: no active allocation remains and current memory returns to zero
    /// while the deallocated total is preserved.
    #[test]
    fn test_build_snapshot_with_deallocations() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 1024, 1),
            MemoryEvent::deallocate(0x1000, 1024, 1),
        ];
        let snapshot = build_snapshot_from_events(&events);
        assert_eq!(snapshot.active_count(), 0, "the binding must be released");
        assert_eq!(snapshot.current_memory(), 0, "current bytes must be zero");
        assert_eq!(
            snapshot.stats.total_deallocated, 1024,
            "released bytes must still be reported"
        );
        assert_eq!(
            snapshot.stats.total_deallocations, 1,
            "exactly one release was matched"
        );
    }

    /// Objective: Verify a late release of the previous address generation keeps
    /// the newer binding alive.
    /// Invariants: the reused address stays active with its own size.
    #[test]
    fn test_build_snapshot_reuse_keeps_newer_binding() {
        let events = vec![
            MemoryEvent::allocate(0x3000, 64, 1),
            MemoryEvent::allocate(0x3000, 256, 1),
            MemoryEvent::deallocate(0x3000, 64, 1),
        ];
        let snapshot = build_snapshot_from_events(&events);
        assert_eq!(
            snapshot.active_count(),
            1,
            "one release must close exactly one binding"
        );
        let active = snapshot
            .active_allocations
            .get(&0x3000)
            .expect("the reused address must stay active");
        assert_eq!(active.size, 256, "the surviving binding keeps its size");
        assert_eq!(
            snapshot.current_memory(),
            256,
            "current bytes must follow the surviving binding"
        );
        assert_eq!(
            snapshot.stats.peak_memory, 320,
            "peak must include the overlapping window"
        );
    }

    /// Objective: Verify an in-place reallocation updates size and totals without
    /// creating a second binding.
    /// Invariants: one active binding with the new size, one reallocation counted.
    #[test]
    fn test_build_snapshot_reallocation_updates_size() {
        let events = vec![
            MemoryEvent::allocate(0x4000, 100, 1),
            MemoryEvent::reallocate(0x4000, 100, 300, 1),
        ];
        let snapshot = build_snapshot_from_events(&events);
        assert_eq!(snapshot.active_count(), 1, "reallocation keeps one binding");
        let active = snapshot
            .active_allocations
            .get(&0x4000)
            .expect("the reallocated binding must stay active");
        assert_eq!(active.size, 300, "the size must be refreshed");
        assert_eq!(
            snapshot.stats.total_reallocations, 1,
            "the reallocation must be counted separately"
        );
        assert_eq!(
            snapshot.stats.current_memory, 300,
            "current bytes must follow the new size"
        );
        assert_eq!(
            snapshot.stats.peak_memory, 300,
            "the peak must not contain an intermediate add-then-subtract value"
        );
    }
}
