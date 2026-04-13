//! Timeline Index - Index for efficient timeline queries
//!
//! This module provides indexing structures for efficient timeline queries.

use crate::event_store::MemoryEvent;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// Timeline index for efficient event lookup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineIndex {
    /// Index by pointer address
    pub by_ptr: HashMap<usize, Vec<usize>>,
    /// Index by thread ID
    pub by_thread: HashMap<u64, Vec<usize>>,
    /// Index by scope name
    pub by_scope: HashMap<String, Vec<usize>>,
    /// Index by task ID
    pub by_task: HashMap<u64, Vec<usize>>,
    /// Index by timestamp (ordered)
    pub by_time: BTreeMap<u64, Vec<usize>>,
}

impl TimelineIndex {
    /// Create a new timeline index
    pub fn new() -> Self {
        Self {
            by_ptr: HashMap::new(),
            by_thread: HashMap::new(),
            by_scope: HashMap::new(),
            by_task: HashMap::new(),
            by_time: BTreeMap::new(),
        }
    }

    /// Index an event
    ///
    /// # Arguments
    /// * `event_index` - The index of the event in the event list
    /// * `event` - The event to index
    pub fn index_event(&mut self, event_index: usize, event: &MemoryEvent) {
        // Index by pointer
        self.by_ptr.entry(event.ptr).or_default().push(event_index);

        // Index by thread
        self.by_thread
            .entry(event.thread_id)
            .or_default()
            .push(event_index);

        // Index by time
        self.by_time
            .entry(event.timestamp)
            .or_default()
            .push(event_index);

        // Index by variable name if available
        if let Some(ref var_name) = event.var_name {
            self.by_scope
                .entry(var_name.clone())
                .or_default()
                .push(event_index);
        }
    }

    /// Index multiple events
    ///
    /// # Arguments
    /// * `events` - The events to index
    pub fn index_events(&mut self, events: &[MemoryEvent]) {
        for (i, event) in events.iter().enumerate() {
            self.index_event(i, event);
        }
    }

    /// Get event indices by pointer
    pub fn get_by_ptr(&self, ptr: usize) -> Option<&Vec<usize>> {
        self.by_ptr.get(&ptr)
    }

    /// Get event indices by thread
    pub fn get_by_thread(&self, thread_id: u64) -> Option<&Vec<usize>> {
        self.by_thread.get(&thread_id)
    }

    /// Get event indices by scope
    pub fn get_by_scope(&self, scope: &str) -> Option<&Vec<usize>> {
        self.by_scope.get(scope)
    }

    /// Get event indices by time range
    pub fn get_by_time_range(&self, start: u64, end: u64) -> Vec<usize> {
        let mut result = Vec::new();
        for (_timestamp, indices) in self.by_time.range(start..=end) {
            result.extend(indices);
        }
        result
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.by_ptr.clear();
        self.by_thread.clear();
        self.by_scope.clear();
        self.by_task.clear();
        self.by_time.clear();
    }
}

impl Default for TimelineIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::MemoryEventType;

    #[test]
    fn test_timeline_index_creation() {
        let index = TimelineIndex::new();
        assert!(index.by_ptr.is_empty());
        assert!(index.by_thread.is_empty());
    }

    #[test]
    fn test_index_event() {
        let mut index = TimelineIndex::new();
        let event = MemoryEvent {
            timestamp: 1000,
            event_type: MemoryEventType::Allocate,
            ptr: 0x1000,
            size: 1024,
            old_size: None,
            thread_id: 1,
            var_name: Some("test_var".to_string()),
            type_name: Some("i32".to_string()),
            call_stack_hash: None,
            thread_name: None,
            source_file: None,
            source_line: None,
        };

        index.index_event(0, &event);

        assert!(index.get_by_ptr(0x1000).is_some());
        assert!(index.get_by_thread(1).is_some());
        assert!(index.get_by_scope("test_var").is_some());
    }

    #[test]
    fn test_index_multiple_events() {
        let mut index = TimelineIndex::new();
        let events = vec![
            MemoryEvent::allocate(0x1000, 1024, 1),
            MemoryEvent::allocate(0x2000, 2048, 1),
            MemoryEvent::deallocate(0x1000, 1024, 1),
        ];

        index.index_events(&events);

        assert_eq!(index.get_by_ptr(0x1000).unwrap().len(), 2);
        assert_eq!(index.get_by_ptr(0x2000).unwrap().len(), 1);
    }

    #[test]
    fn test_get_by_time_range() {
        let mut index = TimelineIndex::new();
        let events = vec![
            MemoryEvent::allocate(0x1000, 1024, 1),
            MemoryEvent::allocate(0x2000, 2048, 1),
            MemoryEvent::deallocate(0x1000, 1024, 1),
        ];

        index.index_events(&events);

        let range_events = index.get_by_time_range(0, u64::MAX);
        assert_eq!(range_events.len(), 3);
    }

    #[test]
    fn test_timeline_index_default() {
        let index = TimelineIndex::default();
        assert!(index.by_ptr.is_empty());
        assert!(index.by_thread.is_empty());
        assert!(index.by_scope.is_empty());
        assert!(index.by_task.is_empty());
        assert!(index.by_time.is_empty());
    }

    #[test]
    fn test_timeline_index_clear() {
        let mut index = TimelineIndex::new();
        let event = MemoryEvent::allocate(0x1000, 1024, 1);
        index.index_event(0, &event);

        assert!(!index.by_ptr.is_empty());

        index.clear();

        assert!(index.by_ptr.is_empty());
        assert!(index.by_thread.is_empty());
        assert!(index.by_scope.is_empty());
        assert!(index.by_task.is_empty());
        assert!(index.by_time.is_empty());
    }

    #[test]
    fn test_get_by_ptr_not_found() {
        let index = TimelineIndex::new();
        assert!(index.get_by_ptr(0xDEADBEEF).is_none());
    }

    #[test]
    fn test_get_by_thread_not_found() {
        let index = TimelineIndex::new();
        assert!(index.get_by_thread(999).is_none());
    }

    #[test]
    fn test_get_by_scope_not_found() {
        let index = TimelineIndex::new();
        assert!(index.get_by_scope("nonexistent").is_none());
    }

    #[test]
    fn test_index_event_without_var_name() {
        let mut index = TimelineIndex::new();
        let mut event = MemoryEvent::allocate(0x1000, 1024, 1);
        event.var_name = None;

        index.index_event(0, &event);

        assert!(index.get_by_ptr(0x1000).is_some());
        assert!(index.by_scope.is_empty());
    }

    #[test]
    fn test_get_by_time_range_empty() {
        let index = TimelineIndex::new();
        let range_events = index.get_by_time_range(0, 1000);
        assert!(range_events.is_empty());
    }

    #[test]
    fn test_get_by_time_range_partial() {
        let mut index = TimelineIndex::new();
        let mut event1 = MemoryEvent::allocate(0x1000, 1024, 1);
        event1.timestamp = 100;
        let mut event2 = MemoryEvent::allocate(0x2000, 2048, 1);
        event2.timestamp = 200;
        let mut event3 = MemoryEvent::allocate(0x3000, 4096, 1);
        event3.timestamp = 300;

        index.index_event(0, &event1);
        index.index_event(1, &event2);
        index.index_event(2, &event3);

        let range_events = index.get_by_time_range(150, 250);
        assert_eq!(range_events.len(), 1);
    }

    #[test]
    fn test_timeline_index_clone() {
        let mut index = TimelineIndex::new();
        let event = MemoryEvent::allocate(0x1000, 1024, 1);
        index.index_event(0, &event);

        let cloned = index.clone();
        assert!(cloned.get_by_ptr(0x1000).is_some());
    }

    #[test]
    fn test_timeline_index_debug() {
        let index = TimelineIndex::new();
        let debug_str = format!("{:?}", index);
        assert!(debug_str.contains("TimelineIndex"));
    }

    #[test]
    fn test_index_multiple_threads() {
        let mut index = TimelineIndex::new();
        let event1 = MemoryEvent::allocate(0x1000, 1024, 1);
        let event2 = MemoryEvent::allocate(0x2000, 2048, 2);
        let event3 = MemoryEvent::allocate(0x3000, 4096, 1);

        index.index_events(&[event1, event2, event3]);

        assert_eq!(index.get_by_thread(1).unwrap().len(), 2);
        assert_eq!(index.get_by_thread(2).unwrap().len(), 1);
    }

    #[test]
    fn test_index_same_ptr_multiple_times() {
        let mut index = TimelineIndex::new();
        let event1 = MemoryEvent::allocate(0x1000, 1024, 1);
        let event2 = MemoryEvent::deallocate(0x1000, 1024, 1);

        index.index_events(&[event1, event2]);

        let ptr_events = index.get_by_ptr(0x1000).unwrap();
        assert_eq!(ptr_events.len(), 2);
        assert_eq!(ptr_events[0], 0);
        assert_eq!(ptr_events[1], 1);
    }
}
