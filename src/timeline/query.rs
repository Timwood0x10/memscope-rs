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

    /// Get memory usage over time with cumulative tracking.
    ///
    /// # Arguments
    /// * `start` - Start timestamp (inclusive)
    /// * `end` - End timestamp (exclusive)
    /// * `interval_ms` - Interval between snapshots in milliseconds
    ///
    /// # Returns
    /// Vector of (timestamp, cumulative_memory_bytes) tuples showing
    /// the actual memory usage at each point in time.
    pub fn get_memory_usage_over_time(
        &self,
        start: u64,
        end: u64,
        interval_ms: u64,
    ) -> Vec<(u64, usize)> {
        let mut result = Vec::new();
        let interval_ns = interval_ms * 1_000_000;

        // Get all events in the time range, sorted by timestamp
        let mut all_events: Vec<(u64, usize, bool)> = Vec::new(); // (timestamp, size, is_alloc)

        for event in self.get_events_in_range(start, end) {
            let is_alloc = matches!(event.event_type, MemoryEventType::Allocate);
            all_events.push((event.timestamp, event.size, is_alloc));
        }

        // Sort by timestamp
        all_events.sort_by_key(|(ts, _, _)| *ts);

        // Sample at each interval
        let mut current = start;
        let mut event_idx = 0;
        let mut running_memory: usize = 0;

        while current < end {
            // Process all events up to current timestamp
            while event_idx < all_events.len() {
                let (ts, size, is_alloc) = all_events[event_idx];
                if ts > current {
                    break;
                }
                if is_alloc {
                    running_memory += size;
                } else {
                    running_memory = running_memory.saturating_sub(size);
                }
                event_idx += 1;
            }

            result.push((current, running_memory));
            current += interval_ns;
        }

        result
    }

    /// Get peak memory usage in a time range
    ///
    /// # Arguments
    /// * `start` - Start timestamp (inclusive)
    /// * `end` - End timestamp (exclusive)
    ///
    /// # Note
    /// This function processes all events in chronological order to find
    /// the true peak memory usage, not just the maximum interval delta.
    pub fn get_peak_memory_in_range(&self, start: u64, end: u64) -> usize {
        // Get all events sorted by timestamp
        let mut all_events: Vec<(u64, usize, bool)> = Vec::new();

        for event in self.get_events_in_range(start, end) {
            let is_alloc = matches!(event.event_type, MemoryEventType::Allocate);
            all_events.push((event.timestamp, event.size, is_alloc));
        }

        all_events.sort_by_key(|(ts, _, _)| *ts);

        // Track running memory and find peak
        let mut running_memory: usize = 0;
        let mut peak_memory: usize = 0;

        for (_, size, is_alloc) in all_events {
            if is_alloc {
                running_memory += size;
            } else {
                running_memory = running_memory.saturating_sub(size);
            }
            peak_memory = peak_memory.max(running_memory);
        }

        peak_memory
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
        let usage = query.get_memory_usage_over_time(0, 1000, 100);
        assert!(!usage.is_empty());
    }
}
