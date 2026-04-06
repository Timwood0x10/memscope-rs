//! Timeline Engine - Time-based memory analysis
//!
//! This module provides the TimelineEngine which is responsible for
//! time-based analysis and replay of memory events.

use crate::event_store::{MemoryEvent, SharedEventStore};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

pub struct TimelineEngine {
    event_store: SharedEventStore,
    cached_events: RwLock<Vec<MemoryEvent>>,
    cache_version: AtomicU64,
}

impl TimelineEngine {
    pub fn new(event_store: SharedEventStore) -> Self {
        Self {
            event_store,
            cached_events: RwLock::new(Vec::new()),
            cache_version: AtomicU64::new(0),
        }
    }

    fn ensure_sorted_cache(&self) {
        let current_count = self.event_store.len();
        let cache_count = self.cached_events.read().unwrap().len();

        if cache_count != current_count {
            let mut cache = self.cached_events.write().unwrap();
            *cache = self.event_store.snapshot();
            cache.sort_by_key(|e| e.timestamp);
            self.cache_version.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn get_events_in_range(&self, start: u64, end: u64) -> Vec<MemoryEvent> {
        self.ensure_sorted_cache();

        let cache = self.cached_events.read().unwrap();
        if cache.is_empty() {
            return Vec::new();
        }

        let start_idx = cache.partition_point(|e| e.timestamp < start);
        let end_idx = cache.partition_point(|e| e.timestamp < end);

        if start_idx >= end_idx {
            return Vec::new();
        }

        cache[start_idx..end_idx].to_vec()
    }

    pub fn get_events_for_pointer(&self, ptr: usize) -> Vec<MemoryEvent> {
        self.event_store
            .snapshot()
            .into_iter()
            .filter(|e| e.ptr == ptr)
            .collect()
    }

    pub fn get_events_for_thread(&self, thread_id: u64) -> Vec<MemoryEvent> {
        self.event_store
            .snapshot()
            .into_iter()
            .filter(|e| e.thread_id == thread_id)
            .collect()
    }

    pub fn get_events_for_scope(&self, scope_name: &str) -> Vec<MemoryEvent> {
        self.event_store
            .snapshot()
            .into_iter()
            .filter(|e| {
                e.var_name
                    .as_ref()
                    .map(|name| name == scope_name)
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn replay_up_to(&self, timestamp: u64) -> Vec<MemoryEvent> {
        self.get_events_in_range(0, timestamp)
    }

    pub fn invalidate_cache(&self) {
        self.cache_version.store(0, Ordering::Relaxed);
    }

    pub fn get_event_count(&self) -> usize {
        self.event_store.len()
    }

    pub fn get_time_range(&self) -> Option<(u64, u64)> {
        self.ensure_sorted_cache();

        let cache = self.cached_events.read().unwrap();
        if cache.is_empty() {
            return None;
        }

        Some((
            cache.first().unwrap().timestamp,
            cache.last().unwrap().timestamp,
        ))
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
        let events = engine.get_events_in_range(0, u64::MAX);
        assert!(events.is_empty());
    }

    #[test]
    fn test_get_events_for_pointer() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(MemoryEvent::deallocate(0x1000, 1024, 1));

        let engine = TimelineEngine::new(event_store);
        let events = engine.get_events_for_pointer(0x1000);

        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_get_events_for_thread() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(MemoryEvent::allocate(0x2000, 2048, 2));

        let engine = TimelineEngine::new(event_store);
        let events = engine.get_events_for_thread(1);

        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_replay_up_to() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(MemoryEvent::allocate(0x2000, 2048, 1));

        let engine = TimelineEngine::new(event_store);
        let events = engine.replay_up_to(u64::MAX);

        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_time_range_query_efficiency() {
        let event_store = Arc::new(EventStore::new());

        for i in 0..1000 {
            let mut event = MemoryEvent::allocate(0x1000 + i, 1024, 1);
            event.timestamp = i as u64 * 1000;
            event_store.record(event);
        }

        let engine = TimelineEngine::new(event_store);

        let events = engine.get_events_in_range(100000, 200000);
        assert_eq!(events.len(), 100);

        assert!(events.first().unwrap().timestamp >= 100000);
        assert!(events.last().unwrap().timestamp < 200000);
    }

    #[test]
    fn test_get_time_range() {
        let event_store = Arc::new(EventStore::new());

        let mut e1 = MemoryEvent::allocate(0x1000, 1024, 1);
        e1.timestamp = 100;
        let mut e2 = MemoryEvent::allocate(0x2000, 1024, 1);
        e2.timestamp = 500;

        event_store.record(e1);
        event_store.record(e2);

        let engine = TimelineEngine::new(event_store);
        let range = engine.get_time_range();

        assert_eq!(range, Some((100, 500)));
    }
}
