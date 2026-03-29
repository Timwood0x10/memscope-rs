//! Timeline Query - Time-based event queries
//!
//! This module provides functionality for querying events based on
//! time ranges and other temporal criteria.

use crate::event_store::event::{MemoryEvent, MemoryEventType};
use crate::event_store::EventStore;
use std::sync::Arc;

/// Timeline query engine
///
/// Provides functionality to query events based on time ranges
/// and other temporal criteria.
pub struct TimelineQuery {
    /// Reference to the event store
    event_store: Arc<EventStore>,
}

impl TimelineQuery {
    /// Create a new timeline query
    pub fn new(event_store: Arc<EventStore>) -> Self {
        Self { event_store }
    }

    /// Get events in a time range
    ///
    /// # Arguments
    /// * `start` - Start timestamp (inclusive)
    /// * `end` - End timestamp (exclusive)
    pub fn get_events_in_range(&self, start: u64, end: u64) -> Vec<MemoryEvent> {
        self.event_store
            .snapshot()
            .into_iter()
            .filter(|e| e.timestamp >= start && e.timestamp < end)
            .collect()
    }

    /// Get allocation events in a time range
    ///
    /// # Arguments
    /// * `start` - Start timestamp (inclusive)
    /// * `end` - End timestamp (exclusive)
    pub fn get_allocations_in_range(&self, start: u64, end: u64) -> Vec<MemoryEvent> {
        self.get_events_in_range(start, end)
            .into_iter()
            .filter(|e| e.event_type == MemoryEventType::Allocate)
            .collect()
    }

    /// Get deallocation events in a time range
    ///
    /// # Arguments
    /// * `start` - Start timestamp (inclusive)
    /// * `end` - End timestamp (exclusive)
    pub fn get_deallocations_in_range(&self, start: u64, end: u64) -> Vec<MemoryEvent> {
        self.get_events_in_range(start, end)
            .into_iter()
            .filter(|e| e.event_type == MemoryEventType::Deallocate)
            .collect()
    }

    /// Get events for a specific thread in a time range
    ///
    /// # Arguments
    /// * `thread_id` - The thread ID to filter by
    /// * `start` - Start timestamp (inclusive)
    /// * `end` - End timestamp (exclusive)
    pub fn get_thread_events_in_range(
        &self,
        thread_id: u64,
        start: u64,
        end: u64,
    ) -> Vec<MemoryEvent> {
        self.get_events_in_range(start, end)
            .into_iter()
            .filter(|e| e.thread_id == thread_id)
            .collect()
    }

    /// Get memory usage over time
    ///
    /// Returns a series of memory usage snapshots at regular intervals.
    ///
    /// # Arguments
    /// * `start` - Start timestamp (inclusive)
    /// * `end` - End timestamp (exclusive)
    /// * `interval_ms` - Interval between snapshots in milliseconds
    pub fn get_memory_usage_over_time(
        &self,
        start: u64,
        end: u64,
        interval_ms: u64,
    ) -> Vec<(u64, usize)> {
        let mut result = Vec::new();
        let mut current = start;
        let interval_ns = interval_ms * 1_000_000;

        while current < end {
            let interval_end = (current + interval_ns).min(end);
            let allocations: usize = self
                .get_allocations_in_range(current, interval_end)
                .iter()
                .map(|e| e.size)
                .sum();
            let deallocations: usize = self
                .get_deallocations_in_range(current, interval_end)
                .iter()
                .map(|e| e.size)
                .sum();

            result.push((current, allocations.saturating_sub(deallocations)));
            current = interval_end;
        }

        result
    }

    /// Get peak memory usage in a time range
    ///
    /// # Arguments
    /// * `start` - Start timestamp (inclusive)
    /// * `end` - End timestamp (exclusive)
    pub fn get_peak_memory_in_range(&self, start: u64, end: u64) -> usize {
        let memory_usage = self.get_memory_usage_over_time(start, end, 100);
        memory_usage
            .into_iter()
            .map(|(_, usage)| usage)
            .max()
            .unwrap_or(0)
    }

    /// Get event rate (events per second) in a time range
    ///
    /// # Arguments
    /// * `start` - Start timestamp (inclusive)
    /// * `end` - End timestamp (exclusive)
    pub fn get_event_rate(&self, start: u64, end: u64) -> f64 {
        let events = self.get_events_in_range(start, end);
        let duration_ns = end.saturating_sub(start) as f64;
        if duration_ns > 0.0 {
            (events.len() as f64) / (duration_ns / 1_000_000_000.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_query_creation() {
        let event_store = Arc::new(EventStore::new());
        let query = TimelineQuery::new(event_store);
        let events = query.get_events_in_range(0, 1000);
        assert!(events.is_empty());
    }

    #[test]
    fn test_get_events_in_range() {
        let event_store = Arc::new(EventStore::new());
        let event1 = MemoryEvent::allocate(0x1000, 1024, 123);
        event_store.record(event1);
        let event2 = MemoryEvent::deallocate(0x1000, 1024, 456);
        event_store.record(event2);

        let query = TimelineQuery::new(event_store);
        // Use a large time range to capture all events
        let events = query.get_events_in_range(0, u64::MAX);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_memory_usage_over_time() {
        let event_store = Arc::new(EventStore::new());
        let event = MemoryEvent::allocate(0x1000, 1024, 123);
        event_store.record(event);

        let query = TimelineQuery::new(event_store);
        let usage = query.get_memory_usage_over_time(0, u64::MAX, 100);
        assert!(!usage.is_empty());
    }
}
