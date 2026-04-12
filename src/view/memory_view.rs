//! MemoryView - Unified read-only access to memory data.

use crate::event_store::MemoryEvent;
use crate::snapshot::{build_snapshot_from_events, ActiveAllocation, MemorySnapshot};
use std::sync::Arc;

use super::{FilterBuilder, ViewStats};

/// Memory view - unified read-only access to memory data.
///
/// This is the single source of truth for all analysis modules.
/// Reuses MemorySnapshot to avoid duplicate allocation rebuilding.
#[derive(Clone)]
pub struct MemoryView {
    snapshot: MemorySnapshot,
    events: Arc<[MemoryEvent]>,
}

impl MemoryView {
    /// Create view from snapshot and events.
    pub fn new(snapshot: MemorySnapshot, events: Vec<MemoryEvent>) -> Self {
        Self {
            snapshot,
            events: events.into(),
        }
    }

    /// Create view from GlobalTracker.
    pub fn from_tracker(
        tracker: &crate::capture::backends::global_tracking::GlobalTracker,
    ) -> Self {
        let events = tracker.tracker().events();
        let snapshot = build_snapshot_from_events(&events);
        Self::new(snapshot, events)
    }

    /// Create view from events directly.
    pub fn from_events(events: Vec<MemoryEvent>) -> Self {
        let snapshot = build_snapshot_from_events(&events);
        Self::new(snapshot, events)
    }

    /// Get all active allocations (from snapshot).
    pub fn allocations(&self) -> Vec<&ActiveAllocation> {
        self.snapshot.active_allocations.values().collect()
    }

    /// Get allocation by pointer.
    pub fn get_allocation(&self, ptr: usize) -> Option<&ActiveAllocation> {
        self.snapshot.active_allocations.get(&ptr)
    }

    /// Get all events.
    pub fn events(&self) -> &[MemoryEvent] {
        &self.events
    }

    /// Get underlying snapshot.
    pub fn snapshot(&self) -> &MemorySnapshot {
        &self.snapshot
    }

    /// Get memory stats.
    pub fn stats(&self) -> ViewStats {
        ViewStats::from_snapshot(&self.snapshot)
    }

    /// Create filter builder.
    pub fn filter(&self) -> FilterBuilder<'_> {
        FilterBuilder::new(self)
    }

    /// Get allocation count.
    pub fn len(&self) -> usize {
        self.snapshot.active_allocations.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.snapshot.active_allocations.is_empty()
    }

    /// Get total memory usage.
    pub fn total_memory(&self) -> usize {
        self.snapshot.stats.current_memory
    }

    /// Get thread IDs.
    pub fn thread_ids(&self) -> Vec<u64> {
        self.snapshot.thread_stats.keys().copied().collect()
    }
}

impl Default for MemoryView {
    fn default() -> Self {
        Self::from_events(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_view() {
        let view = MemoryView::from_events(vec![]);
        assert!(view.is_empty());
        assert_eq!(view.len(), 0);
    }

    #[test]
    fn test_single_allocation() {
        let events = vec![MemoryEvent::allocate(0x1000, 64, 1)];
        let view = MemoryView::from_events(events);
        assert_eq!(view.len(), 1);
        assert_eq!(view.total_memory(), 64);
    }

    #[test]
    fn test_allocation_and_deallocation() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::deallocate(0x1000, 64, 1),
        ];
        let view = MemoryView::from_events(events);
        assert!(view.is_empty());
        assert_eq!(view.total_memory(), 0);
    }

    #[test]
    fn test_default_view() {
        let view = MemoryView::default();
        assert!(view.is_empty());
        assert_eq!(view.len(), 0);
        assert_eq!(view.total_memory(), 0);
    }

    #[test]
    fn test_get_allocation_existing() {
        let events = vec![MemoryEvent::allocate(0x1000, 128, 1)];
        let view = MemoryView::from_events(events);

        let alloc = view.get_allocation(0x1000);
        assert!(alloc.is_some());
        assert_eq!(alloc.unwrap().size, 128);
    }

    #[test]
    fn test_get_allocation_not_found() {
        let events = vec![MemoryEvent::allocate(0x1000, 64, 1)];
        let view = MemoryView::from_events(events);

        let alloc = view.get_allocation(0x2000);
        assert!(alloc.is_none());
    }

    #[test]
    fn test_allocations_multiple() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 1),
            MemoryEvent::allocate(0x3000, 256, 1),
        ];
        let view = MemoryView::from_events(events);

        let allocs = view.allocations();
        assert_eq!(allocs.len(), 3);
    }

    #[test]
    fn test_events_access() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::deallocate(0x1000, 64, 1),
        ];
        let view = MemoryView::from_events(events);

        let view_events = view.events();
        assert_eq!(view_events.len(), 2);
    }

    #[test]
    fn test_snapshot_access() {
        let events = vec![MemoryEvent::allocate(0x1000, 64, 1)];
        let view = MemoryView::from_events(events);

        let snapshot = view.snapshot();
        assert_eq!(snapshot.active_allocations.len(), 1);
    }

    #[test]
    fn test_stats() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 1),
        ];
        let view = MemoryView::from_events(events);

        let stats = view.stats();
        assert_eq!(stats.allocation_count, 2);
        assert_eq!(stats.total_bytes, 192);
    }

    #[test]
    fn test_thread_ids() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 2),
            MemoryEvent::allocate(0x3000, 256, 1),
        ];
        let view = MemoryView::from_events(events);

        let threads = view.thread_ids();
        assert_eq!(threads.len(), 2);
        assert!(threads.contains(&1));
        assert!(threads.contains(&2));
    }

    #[test]
    fn test_filter_builder() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 2),
        ];
        let view = MemoryView::from_events(events);

        let _filter = view.filter();
    }

    #[test]
    fn test_clone() {
        let events = vec![MemoryEvent::allocate(0x1000, 64, 1)];
        let view = MemoryView::from_events(events);

        let cloned = view.clone();
        assert_eq!(cloned.len(), 1);
        assert_eq!(cloned.total_memory(), 64);
    }

    #[test]
    fn test_new_constructor() {
        let events = vec![MemoryEvent::allocate(0x1000, 64, 1)];
        let snapshot = build_snapshot_from_events(&events);
        let view = MemoryView::new(snapshot, events);

        assert_eq!(view.len(), 1);
    }

    #[test]
    fn test_multiple_allocations_same_thread() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 1),
            MemoryEvent::allocate(0x3000, 256, 1),
        ];
        let view = MemoryView::from_events(events);

        assert_eq!(view.len(), 3);
        assert_eq!(view.total_memory(), 448);

        let threads = view.thread_ids();
        assert_eq!(threads.len(), 1);
    }

    #[test]
    fn test_partial_deallocation() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 1),
            MemoryEvent::deallocate(0x1000, 64, 1),
        ];
        let view = MemoryView::from_events(events);

        assert_eq!(view.len(), 1);
        assert_eq!(view.total_memory(), 128);
    }
}
