//! Timeline Replay - Time-based memory event replay
//!
//! This module provides functionality for replaying memory events
//! in chronological order, enabling time-based analysis and
//! visualization.

use crate::event_store::event::MemoryEvent;
use crate::event_store::EventStore;
use std::sync::Arc;

/// Timeline replay controller
///
/// Provides functionality to replay memory events in chronological order,
/// allowing for time-based analysis and visualization.
pub struct TimelineReplay {
    /// Reference to the event store
    #[allow(dead_code)]
    event_store: Arc<EventStore>,
    /// Current replay position
    position: usize,
    /// All events sorted by timestamp
    events: Vec<MemoryEvent>,
}

impl TimelineReplay {
    /// Create a new timeline replay
    pub fn new(event_store: Arc<EventStore>) -> Self {
        let events = event_store.snapshot();
        // Sort events by timestamp
        let mut sorted_events = events;
        sorted_events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Self {
            event_store,
            position: 0,
            events: sorted_events,
        }
    }

    /// Reset replay to the beginning
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Replay all events until a specific timestamp, advancing the position.
    ///
    /// **Note**: This method consumes events up to the timestamp. After calling,
    /// the replay position is advanced and those events cannot be replayed again.
    /// Use `get_events_between()` for non-consuming queries.
    ///
    /// # Arguments
    /// * `timestamp` - The timestamp to replay until
    ///
    /// # Returns
    /// Vector of events up to (but not including) the specified timestamp
    pub fn advance_until(&mut self, timestamp: u64) -> Vec<MemoryEvent> {
        let mut result = Vec::new();
        for event in self.by_ref() {
            if event.timestamp > timestamp {
                break;
            }
            result.push(event);
        }
        result
    }

    /// Replay all events until a specific timestamp (deprecated: use advance_until).
    #[deprecated(
        since = "0.1.12",
        note = "Use `advance_until()` instead. This method consumes events and advances the replay position."
    )]
    pub fn replay_until(&mut self, timestamp: u64) -> Vec<MemoryEvent> {
        self.advance_until(timestamp)
    }

    /// Get all events between two timestamps
    ///
    /// # Arguments
    /// * `start` - Start timestamp (inclusive)
    /// * `end` - End timestamp (exclusive)
    pub fn get_events_between(&self, start: u64, end: u64) -> Vec<MemoryEvent> {
        self.events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp < end)
            .cloned()
            .collect()
    }

    /// Get the number of events in the timeline
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the timeline is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get the current replay position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get the progress percentage (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        if self.events.is_empty() {
            0.0
        } else {
            self.position as f64 / self.events.len() as f64
        }
    }
}

impl Iterator for TimelineReplay {
    type Item = MemoryEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.events.len() {
            let event = self.events[self.position].clone();
            self.position += 1;
            Some(event)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_replay_creation() {
        let event_store = Arc::new(EventStore::new());
        let replay = TimelineReplay::new(event_store);
        assert_eq!(replay.position(), 0);
        assert!(replay.is_empty());
    }

    #[test]
    fn test_timeline_replay_next() {
        let event_store = Arc::new(EventStore::new());
        let event = MemoryEvent::allocate(0x1000, 1024, 123);
        event_store.record(event);
        event_store.record(MemoryEvent::deallocate(0x1000, 1024, 456));

        let mut replay = TimelineReplay::new(event_store);
        assert_eq!(replay.len(), 2);

        let first = replay.next();
        assert!(first.is_some());
        assert_eq!(first.unwrap().thread_id, 123);

        let second = replay.next();
        assert!(second.is_some());
        assert_eq!(second.unwrap().thread_id, 456);

        let third = replay.next();
        assert!(third.is_none());
    }
}
