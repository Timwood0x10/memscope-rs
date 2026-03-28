//! Event Store Engine - Centralized event storage
//!
//! This module provides the EventStore which is responsible for storing
//! all memory events across all tracking backends. It uses a lock-free
//! queue for high-performance concurrent access.

use crate::event_store::event::MemoryEvent;
use crossbeam::queue::SegQueue;
use std::sync::{Arc, Mutex};

/// Event Store - Centralized storage for memory events
///
/// The EventStore is the single source of truth for all memory events
/// in the system. It uses a lock-free queue (SegQueue) to allow
/// concurrent event recording from multiple threads without contention.
///
/// Key properties:
/// - Lock-free: Uses SegQueue for zero-contention concurrent access
/// - Non-blocking: All operations are non-blocking
/// - Thread-safe: Safe to share across threads via Arc
/// - Atomic snapshots: Uses mutex to ensure thread-safe snapshot operations
#[derive(Debug)]
pub struct EventStore {
    /// Lock-free queue of memory events
    events: SegQueue<MemoryEvent>,
    /// Mutex to protect snapshot operations
    snapshot_lock: Mutex<()>,
}

impl EventStore {
    /// Create a new EventStore
    pub fn new() -> Self {
        Self {
            events: SegQueue::new(),
            snapshot_lock: Mutex::new(()),
        }
    }

    /// Record a memory event
    ///
    /// This method is thread-safe and can be called from any thread.
    /// It pushes the event into the lock-free queue without blocking.
    ///
    /// # Arguments
    /// * `event` - The memory event to record
    pub fn record(&self, event: MemoryEvent) {
        self.events.push(event);
    }

    /// Get all events as a snapshot
    ///
    /// Returns a snapshot of all events currently in the store.
    /// This method does not consume or remove events; they remain
    /// available for future queries.
    ///
    /// This method uses a mutex to ensure thread-safe snapshot operations.
    /// Multiple concurrent snapshot calls will be serialized, while event
    /// recording remains lock-free and non-blocking.
    ///
    /// # Returns
    /// A vector containing all events in the store
    pub fn snapshot(&self) -> Vec<MemoryEvent> {
        let _guard = self.snapshot_lock.lock().unwrap();
        let mut events = Vec::new();
        // Temporarily drain all events
        while let Some(event) = self.events.pop() {
            events.push(event);
        }
        // Restore all events to the queue
        for event in &events {
            self.events.push(event.clone());
        }
        events
    }

    /// Get the number of events in the store
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear all events from the store
    ///
    /// This method removes all events from the store. Use with caution
    /// as this will permanently delete all recorded events.
    pub fn clear(&self) {
        while self.events.pop().is_some() {
            // Pop all events
        }
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared reference to EventStore
pub type SharedEventStore = Arc<EventStore>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_store_creation() {
        let store = EventStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_record_event() {
        let store = EventStore::new();
        let event = MemoryEvent::allocate(0x1000, 1024, 1);
        store.record(event);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_snapshot() {
        let store = EventStore::new();
        let event1 = MemoryEvent::allocate(0x1000, 1024, 1);
        let event2 = MemoryEvent::deallocate(0x1000, 1024, 1);
        store.record(event1.clone());
        store.record(event2.clone());

        let snapshot = store.snapshot();
        assert_eq!(snapshot.len(), 2);
        // Verify events are still in store after snapshot
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_clear() {
        let store = EventStore::new();
        let event = MemoryEvent::allocate(0x1000, 1024, 1);
        store.record(event);
        assert_eq!(store.len(), 1);

        store.clear();
        assert!(store.is_empty());
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;
        let store = Arc::new(EventStore::new());
        let mut handles = vec![];

        for i in 0..10 {
            let store_clone = Arc::clone(&store);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let event = MemoryEvent::allocate(i * 1000 + j, 1024, i as u64);
                    store_clone.record(event);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(store.len(), 1000);
    }
}