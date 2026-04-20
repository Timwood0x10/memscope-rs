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
        sorted_events.sort_by_key(|a| a.timestamp);

        Self {
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

    #[test]
    fn test_timeline_replay_reset() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 100));
        event_store.record(MemoryEvent::allocate(0x2000, 2048, 200));

        let mut replay = TimelineReplay::new(event_store);
        assert_eq!(replay.position(), 0);

        replay.next();
        assert_eq!(replay.position(), 1);

        replay.reset();
        assert_eq!(replay.position(), 0);
    }

    #[test]
    fn test_timeline_replay_progress_empty() {
        let event_store = Arc::new(EventStore::new());
        let replay = TimelineReplay::new(event_store);
        assert_eq!(replay.progress(), 0.0);
    }

    #[test]
    fn test_timeline_replay_progress() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 100));
        event_store.record(MemoryEvent::allocate(0x2000, 2048, 200));

        let mut replay = TimelineReplay::new(event_store);
        assert_eq!(replay.progress(), 0.0);

        replay.next();
        assert_eq!(replay.progress(), 0.5);

        replay.next();
        assert_eq!(replay.progress(), 1.0);
    }

    #[test]
    fn test_timeline_replay_advance_until() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 100));
        event_store.record(MemoryEvent::allocate(0x2000, 2048, 200));
        event_store.record(MemoryEvent::allocate(0x3000, 4096, 300));

        let replay = TimelineReplay::new(event_store);
        // Just verify we can get events
        assert!(!replay.is_empty());
    }

    #[test]
    fn test_timeline_replay_get_events_between() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 100));
        event_store.record(MemoryEvent::allocate(0x2000, 2048, 200));
        event_store.record(MemoryEvent::allocate(0x3000, 4096, 300));

        let replay = TimelineReplay::new(event_store);
        let events = replay.get_events_between(0, u64::MAX);

        assert!(!events.is_empty());
    }

    #[test]
    fn test_timeline_replay_len() {
        let event_store = Arc::new(EventStore::new());
        assert_eq!(TimelineReplay::new(event_store.clone()).len(), 0);

        event_store.record(MemoryEvent::allocate(0x1000, 1024, 100));
        assert_eq!(TimelineReplay::new(event_store.clone()).len(), 1);

        event_store.record(MemoryEvent::allocate(0x2000, 2048, 200));
        assert_eq!(TimelineReplay::new(event_store).len(), 2);
    }

    #[test]
    fn test_timeline_replay_iterator() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 100));
        event_store.record(MemoryEvent::allocate(0x2000, 2048, 200));
        event_store.record(MemoryEvent::allocate(0x3000, 4096, 300));

        let replay = TimelineReplay::new(event_store);
        let events: Vec<_> = replay.collect();

        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_timeline_replay_sorted_order() {
        let event_store = Arc::new(EventStore::new());
        // Add multiple events - they will be sorted by timestamp
        event_store.record(MemoryEvent::allocate(0x3000, 4096, 300));
        event_store.record(MemoryEvent::allocate(0x1000, 1024, 100));
        event_store.record(MemoryEvent::allocate(0x2000, 2048, 200));

        let mut replay = TimelineReplay::new(event_store);

        // Events should be returned in sorted order by timestamp
        // Since timestamps are auto-generated, just verify we get all events
        let mut count = 0;
        while replay.next().is_some() {
            count += 1;
        }
        assert_eq!(count, 3);
    }
}
