//! Event Store Engine - Centralized event storage
//!
//! This module provides the EventStore which is responsible for storing
//! all memory events across all tracking backends. It uses a lock-free
//! SegQueue for high-concurrency recording with parking_lot RwLock for snapshots.

use crate::event_store::event::MemoryEvent;
use crossbeam::queue::SegQueue;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Event Store - Centralized storage for memory events
///
/// The EventStore is the single source of truth for all memory events
/// in the system. It uses a lock-free SegQueue for recording operations
/// and a RwLock-protected Vec for efficient snapshots.
///
/// Key properties:
/// - Lock-free recording: Uses SegQueue for O(1) append without blocking
/// - Thread-safe: All operations are safe for concurrent use
/// - Efficient snapshots: Uses RwLock for fast read access
/// - Clear-safe: Uses atomic flag to prevent event loss during clear operations
#[derive(Debug)]
pub struct EventStore {
    /// Lock-free queue for high-concurrency recording
    queue: SegQueue<MemoryEvent>,
    /// Cached events for fast snapshot access
    cache: RwLock<Vec<MemoryEvent>>,
    /// Approximate count of events (may be slightly stale)
    count: AtomicUsize,
    /// Flag to indicate clear operation is in progress
    clearing: AtomicUsize,
}

impl EventStore {
    /// Create a new EventStore
    pub fn new() -> Self {
        Self {
            queue: SegQueue::new(),
            cache: RwLock::new(Vec::new()),
            count: AtomicUsize::new(0),
            clearing: AtomicUsize::new(0),
        }
    }

    /// Record a memory event
    ///
    /// This method is lock-free and can be called from any thread
    /// without blocking other recording operations.
    ///
    /// # Arguments
    /// * `event` - The memory event to record
    ///
    /// # Note
    /// If a clear operation is in progress, this method will skip recording
    /// the event to prevent data loss. This ensures consistency between
    /// the event queue and the count.
    pub fn record(&self, event: MemoryEvent) {
        // Check if clear operation is in progress
        // If clearing flag is set (non-zero), skip recording to prevent event loss
        if self.clearing.load(Ordering::Acquire) != 0 {
            tracing::trace!("Skipping event recording due to clear operation in progress");
            return;
        }

        self.queue.push(event);
        // Use Release ordering to ensure the push is visible before the count increment
        self.count.fetch_add(1, Ordering::Release);
    }

    /// Flush pending events from queue to cache
    fn flush_to_cache(&self) {
        let mut cache = self.cache.write();
        while let Some(event) = self.queue.pop() {
            cache.push(event);
        }
    }

    /// Get all events as a snapshot
    ///
    /// Returns a snapshot of all events currently in the store.
    /// This method flushes any pending events from the lock-free queue
    /// to the cache before returning.
    ///
    /// # Returns
    /// A vector containing all events in the store
    pub fn snapshot(&self) -> Vec<MemoryEvent> {
        self.flush_to_cache();
        self.cache.read().clone()
    }

    /// Get the number of events in the store
    ///
    /// Note: This returns an approximate count that may be slightly
    /// higher than the actual count due to concurrent operations.
    pub fn len(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all events from the store
    ///
    /// This method removes all events from both the queue and cache.
    /// Uses write lock to ensure atomicity with concurrent record operations.
    /// Sets a clearing flag to prevent record() operations during clear.
    pub fn clear(&self) {
        // Set clearing flag to prevent concurrent record operations
        self.clearing.store(1, Ordering::Release);

        // Acquire write lock first to prevent concurrent modifications
        let mut cache = self.cache.write();

        // Clear the queue
        while self.queue.pop().is_some() {}

        // Clear the cache
        cache.clear();

        // Reset count last, while still holding the write lock
        self.count.store(0, Ordering::Release);

        // Clear the clearing flag to allow record operations again
        self.clearing.store(0, Ordering::Release);
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
        let snapshot = store.snapshot();
        assert_eq!(snapshot.len(), 1000);
    }
}
