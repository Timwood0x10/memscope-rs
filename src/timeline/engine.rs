//! Timeline Engine - Time-based memory analysis
//!
//! This module provides the TimelineEngine which is responsible for
//! time-based analysis and replay of memory events.

use crate::event_store::{MemoryEvent, SharedEventStore};
use crate::timeline::index::TimelineIndex;

/// Timeline Engine - Time-based memory analysis
///
/// The TimelineEngine provides time-based analysis capabilities including
/// event replay, time range queries, and historical state reconstruction.
///
/// Key properties:
/// - Time-aware: All operations are time-based
/// - Non-consuming: Does not consume events from EventStore
/// - Efficient: Uses indexing for fast queries
pub struct TimelineEngine {
    /// Reference to the event store
    event_store: SharedEventStore,
    /// Timeline index for efficient queries
    index: TimelineIndex,
}

impl TimelineEngine {
    /// Create a new TimelineEngine
    pub fn new(event_store: SharedEventStore) -> Self {
        Self {
            event_store,
            index: TimelineIndex::new(),
        }
    }

    /// Rebuild the timeline index
    ///
    /// This method rebuilds the index from the current event store state.
    /// Call this after new events are added to ensure the index is up-to-date.
    pub fn rebuild_index(&mut self) {
        let events = self.event_store.snapshot();
        self.index.clear();
        self.index.index_events(&events);
    }

    /// Get events in a time range
    ///
    /// # Arguments
    /// * `start` - Start timestamp (nanoseconds since epoch)
    /// * `end` - End timestamp (nanoseconds since epoch)
    pub fn get_events_in_range(&mut self, start: u64, end: u64) -> Vec<MemoryEvent> {
        self.rebuild_index();
        let events = self.event_store.snapshot();
        let indices = self.index.get_by_time_range(start, end);
        indices
            .into_iter()
            .filter_map(|i| events.get(i).cloned())
            .collect()
    }

    /// Get events for a specific pointer
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer address
    pub fn get_events_for_pointer(&mut self, ptr: usize) -> Vec<MemoryEvent> {
        self.rebuild_index();
        let events = self.event_store.snapshot();
        if let Some(indices) = self.index.get_by_ptr(ptr) {
            indices
                .iter()
                .filter_map(|&i| events.get(i).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get events for a specific thread
    ///
    /// # Arguments
    /// * `thread_id` - Thread ID
    pub fn get_events_for_thread(&mut self, thread_id: u64) -> Vec<MemoryEvent> {
        self.rebuild_index();
        let events = self.event_store.snapshot();
        if let Some(indices) = self.index.get_by_thread(thread_id) {
            indices
                .iter()
                .filter_map(|&i| events.get(i).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get events for a specific scope/variable
    ///
    /// # Arguments
    /// * `scope_name` - Name of the scope or variable
    pub fn get_events_for_scope(&mut self, scope_name: &str) -> Vec<MemoryEvent> {
        self.rebuild_index();
        let events = self.event_store.snapshot();
        if let Some(indices) = self.index.get_by_scope(scope_name) {
            indices
                .iter()
                .filter_map(|&i| events.get(i).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Replay events up to a specific timestamp
    ///
    /// # Arguments
    /// * `timestamp` - Timestamp to replay up to
    pub fn replay_up_to(&mut self, timestamp: u64) -> Vec<MemoryEvent> {
        self.get_events_in_range(0, timestamp)
    }

    /// Get the timeline index
    pub fn index(&self) -> &TimelineIndex {
        &self.index
    }

    /// Get mutable reference to timeline index
    pub fn index_mut(&mut self) -> &mut TimelineIndex {
        &mut self.index
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::EventStore;
    use std::sync::Arc;

    #[test]
    fn test_timeline_engine_creation() {
        let event_store = Arc::new(EventStore::new());
        let engine = TimelineEngine::new(event_store);
        assert!(engine.index().by_ptr.is_empty());
    }

    #[test]
    fn test_rebuild_index() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));

        let mut engine = TimelineEngine::new(event_store);
        engine.rebuild_index();

        assert!(!engine.index().by_ptr.is_empty());
    }

    #[test]
    fn test_get_events_for_pointer() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(MemoryEvent::deallocate(0x1000, 1024, 1));

        let mut engine = TimelineEngine::new(event_store);
        let events = engine.get_events_for_pointer(0x1000);

        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_get_events_for_thread() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(MemoryEvent::allocate(0x2000, 2048, 2));

        let mut engine = TimelineEngine::new(event_store);
        let events = engine.get_events_for_thread(1);

        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_replay_up_to() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(MemoryEvent::allocate(0x2000, 2048, 1));

        let mut engine = TimelineEngine::new(event_store);
        let events = engine.replay_up_to(u64::MAX);

        assert_eq!(events.len(), 2);
    }
}
