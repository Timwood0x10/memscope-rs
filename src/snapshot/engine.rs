//! Snapshot Engine - Snapshot construction and aggregation
//!
//! This module provides the SnapshotEngine which is responsible for
//! building memory snapshots from event data.

use crate::event_store::{MemoryEvent, SharedEventStore};
use crate::snapshot::build_snapshot_from_events;
use crate::snapshot::types::MemorySnapshot;

/// Snapshot Engine - Builds memory snapshots from event data
///
/// The SnapshotEngine is responsible for constructing point-in-time
/// views of memory usage from the events stored in the EventStore.
///
/// Key properties:
/// - Read-only: Does not consume events from EventStore
/// - Efficient: Optimized for fast snapshot construction
/// - Comprehensive: Captures all relevant memory state
pub struct SnapshotEngine {
    /// Reference to the event store
    event_store: SharedEventStore,
}

impl SnapshotEngine {
    /// Create a new SnapshotEngine
    pub fn new(event_store: SharedEventStore) -> Self {
        Self { event_store }
    }

    /// Build a snapshot from the current event store state
    ///
    /// This method reads all events from the event store and
    /// constructs a snapshot representing the current memory state.
    pub fn build_snapshot(&self) -> MemorySnapshot {
        let events = self.event_store.snapshot();
        build_snapshot_from_events(&events)
    }

    /// Build a snapshot from a specific set of events
    ///
    /// This method accepts a slice reference to avoid unnecessary cloning.
    /// The caller retains ownership of the events vector.
    ///
    /// # Arguments
    /// * `events` - A slice of events to build the snapshot from
    ///
    /// # Example
    /// ```ignore
    /// let events = vec![MemoryEvent::allocate(0x1000, 64, 1)];
    /// let snapshot = engine.build_snapshot_from_events(&events);
    /// // events is still available here
    /// ```
    pub fn build_snapshot_from_events(&self, events: &[MemoryEvent]) -> MemorySnapshot {
        build_snapshot_from_events(events)
    }

    /// Get the event store reference
    pub fn event_store(&self) -> &SharedEventStore {
        &self.event_store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::EventStore;
    use std::sync::Arc;

    #[test]
    fn test_snapshot_engine_creation() {
        let event_store = Arc::new(EventStore::new());
        let engine = SnapshotEngine::new(event_store);
        let snapshot = engine.build_snapshot();
        assert_eq!(snapshot.active_count(), 0);
    }

    #[test]
    fn test_snapshot_with_allocations() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(MemoryEvent::allocate(0x2000, 2048, 1));

        let engine = SnapshotEngine::new(event_store);
        let snapshot = engine.build_snapshot();

        assert_eq!(snapshot.active_count(), 2);
        assert_eq!(snapshot.current_memory(), 3072);
    }

    #[test]
    fn test_snapshot_with_deallocations() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(MemoryEvent::deallocate(0x1000, 1024, 1));

        let engine = SnapshotEngine::new(event_store);
        let snapshot = engine.build_snapshot();

        assert_eq!(snapshot.active_count(), 0);
        assert_eq!(snapshot.current_memory(), 0);
        assert_eq!(snapshot.stats.total_allocations, 1);
        assert_eq!(snapshot.stats.total_deallocations, 1);
    }

    #[test]
    fn test_snapshot_peak_memory() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(MemoryEvent::allocate(0x2000, 2048, 1));
        event_store.record(MemoryEvent::deallocate(0x2000, 2048, 1));

        let engine = SnapshotEngine::new(event_store);
        let snapshot = engine.build_snapshot();

        assert_eq!(snapshot.peak_memory(), 3072);
        assert_eq!(snapshot.current_memory(), 1024);
    }

    #[test]
    fn test_snapshot_thread_stats() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(MemoryEvent::allocate(0x2000, 2048, 2));

        let engine = SnapshotEngine::new(event_store);
        let snapshot = engine.build_snapshot();

        assert_eq!(snapshot.thread_stats.len(), 2);

        let thread1 = snapshot
            .thread_stats
            .get(&1)
            .expect("Thread 1 stats should exist");
        assert_eq!(thread1.allocation_count, 1);
        assert_eq!(thread1.total_allocated, 1024);

        let thread2 = snapshot
            .thread_stats
            .get(&2)
            .expect("Thread 2 stats should exist");
        assert_eq!(thread2.allocation_count, 1);
        assert_eq!(thread2.total_allocated, 2048);
    }

    #[test]
    fn test_deallocation_underflow_protection() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(MemoryEvent::deallocate(0x1000, 2048, 1));

        let engine = SnapshotEngine::new(event_store);
        let snapshot = engine.build_snapshot();

        assert_eq!(snapshot.current_memory(), 0);
        let thread1 = snapshot
            .thread_stats
            .get(&1)
            .expect("Thread 1 stats should exist");
        assert_eq!(thread1.current_memory, 0);
    }

    #[test]
    fn test_build_snapshot_from_events_slice() {
        let event_store = Arc::new(EventStore::new());
        let engine = SnapshotEngine::new(event_store);

        let events = vec![
            MemoryEvent::allocate(0x1000, 1024, 1),
            MemoryEvent::allocate(0x2000, 2048, 1),
        ];

        let snapshot = engine.build_snapshot_from_events(&events);

        // events is still available after the call
        assert_eq!(events.len(), 2);
        assert_eq!(snapshot.active_count(), 2);
    }
}
