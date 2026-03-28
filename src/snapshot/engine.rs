//! Snapshot Engine - Snapshot construction and aggregation
//!
//! This module provides the SnapshotEngine which is responsible for
//! building memory snapshots from event data.

use crate::event_store::{MemoryEvent, MemoryEventType, SharedEventStore};
use crate::snapshot::types::{ActiveAllocation, MemorySnapshot, MemoryStats, ThreadMemoryStats};
use std::collections::HashMap;
use std::sync::Arc;

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
        self.build_snapshot_from_events(events)
    }

    /// Build a snapshot from a specific set of events
    ///
    /// # Arguments
    /// * `events` - The events to build the snapshot from
    pub fn build_snapshot_from_events(&self, events: Vec<MemoryEvent>) -> MemorySnapshot {
        let mut snapshot = MemorySnapshot::new();
        let mut ptr_to_allocation: HashMap<usize, ActiveAllocation> = HashMap::new();
        let mut thread_stats: HashMap<u64, ThreadMemoryStats> = HashMap::new();
        let mut peak_memory: usize = 0;
        let mut current_memory: usize = 0;

        for event in events {
            match event.event_type {
                MemoryEventType::Allocate | MemoryEventType::Reallocate => {
                    // Record allocation
                    let allocation = ActiveAllocation {
                        ptr: event.ptr,
                        size: event.size,
                        allocated_at: event.timestamp,
                        var_name: event.var_name,
                        type_name: event.type_name,
                        thread_id: event.thread_id,
                    };

                    ptr_to_allocation.insert(event.ptr, allocation);

                    // Update stats
                    snapshot.stats.total_allocations += 1;
                    snapshot.stats.total_allocated += event.size;
                    current_memory += event.size;

                    // Update thread stats
                    let thread_stat = thread_stats.entry(event.thread_id).or_insert_with(|| {
                        ThreadMemoryStats {
                            thread_id: event.thread_id,
                            allocation_count: 0,
                            total_allocated: 0,
                            current_memory: 0,
                            peak_memory: 0,
                        }
                    });
                    thread_stat.allocation_count += 1;
                    thread_stat.total_allocated += event.size;
                    thread_stat.current_memory += event.size;
                    if thread_stat.current_memory > thread_stat.peak_memory {
                        thread_stat.peak_memory = thread_stat.current_memory;
                    }
                }
                MemoryEventType::Deallocate => {
                    // Remove allocation
                    if let Some(allocation) = ptr_to_allocation.remove(&event.ptr) {
                        // Update stats
                        snapshot.stats.total_deallocations += 1;
                        snapshot.stats.total_deallocated += allocation.size;
                        current_memory -= allocation.size;

                        // Update thread stats
                        if let Some(thread_stat) = thread_stats.get_mut(&event.thread_id) {
                            thread_stat.current_memory -= allocation.size;
                        }
                    }
                }
                MemoryEventType::Move | MemoryEventType::Borrow | MemoryEventType::Return => {
                    // These don't affect the current memory state
                    // but we may want to track them for analysis
                }
            }

            // Update peak memory
            if current_memory > peak_memory {
                peak_memory = current_memory;
            }
        }

        // Build final snapshot
        snapshot.active_allocations = ptr_to_allocation;
        snapshot.thread_stats = thread_stats;
        snapshot.stats.active_allocations = snapshot.active_allocations.len();
        snapshot.stats.current_memory = current_memory;
        snapshot.stats.peak_memory = peak_memory;

        snapshot
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

        let thread1 = snapshot.thread_stats.get(&1).unwrap();
        assert_eq!(thread1.allocation_count, 1);
        assert_eq!(thread1.total_allocated, 1024);

        let thread2 = snapshot.thread_stats.get(&2).unwrap();
        assert_eq!(thread2.allocation_count, 1);
        assert_eq!(thread2.total_allocated, 2048);
    }
}