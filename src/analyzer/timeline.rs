//! Timeline analysis module.

use crate::event_store::{MemoryEvent, MemoryEventType};
use crate::view::MemoryView;
use tracing::debug;

/// Timeline analysis module.
///
/// Provides time-based memory analysis.
pub struct TimelineAnalysis {
    view: MemoryView,
}

impl TimelineAnalysis {
    /// Create from view.
    pub fn from_view(view: &MemoryView) -> Self {
        debug!(
            "Creating TimelineAnalysis with {} events",
            view.events().len()
        );
        Self { view: view.clone() }
    }

    /// Query events in time range.
    pub fn query(&self, start: u64, end: u64) -> TimelineResult {
        debug!("Querying events in time range [{}, {}]", start, end);
        let events: Vec<&MemoryEvent> = self
            .view
            .events()
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect();

        let result = TimelineResult {
            event_count: events.len(),
            start,
            end,
        };
        debug!("Query returned {} events", result.event_count);
        result
    }

    /// Get event timeline.
    pub fn timeline(&self) -> Vec<TimelineEvent> {
        let events: Vec<TimelineEvent> = self
            .view
            .events()
            .iter()
            .map(|e| TimelineEvent {
                timestamp: e.timestamp,
                event_type: format!("{:?}", e.event_type),
                ptr: e.ptr,
                size: e.size,
                thread_id: e.thread_id,
            })
            .collect();
        debug!("Timeline has {} events", events.len());
        events
    }

    /// Get allocation rate over time.
    ///
    /// Groups allocations into time buckets and calculates the rate
    /// for each bucket. Returns a vector of allocation rates per bucket.
    pub fn allocation_rate(&self, bucket_ms: u64) -> Vec<AllocationRate> {
        debug!("Computing allocation rate with {}ms buckets", bucket_ms);
        let events = self.view.events();
        if events.is_empty() || bucket_ms == 0 {
            debug!("Allocation rate: no events or invalid bucket size");
            return vec![];
        }

        // Find time range
        let min_time = events.iter().map(|e| e.timestamp).min().unwrap_or(0);
        let max_time = events.iter().map(|e| e.timestamp).max().unwrap_or(0);

        // Calculate number of buckets
        let time_range = max_time.saturating_sub(min_time);
        let bucket_ns = bucket_ms * 1_000_000; // Convert ms to ns
        let num_buckets = ((time_range / bucket_ns) + 1) as usize;

        if num_buckets == 0 || num_buckets > 10000 {
            debug!(
                "Allocation rate: bucket count {} exceeds limit or is zero",
                num_buckets
            );
            return vec![];
        }

        // Initialize buckets
        let mut buckets: Vec<AllocationRate> = (0..num_buckets)
            .map(|i| AllocationRate {
                start: min_time + (i as u64 * bucket_ns),
                end: min_time + ((i + 1) as u64 * bucket_ns),
                count: 0,
                bytes: 0,
            })
            .collect();

        // Fill buckets
        let mut skipped_count = 0usize;
        for event in events {
            if event.event_type == MemoryEventType::Allocate {
                let bucket_idx = ((event.timestamp.saturating_sub(min_time)) / bucket_ns) as usize;
                if bucket_idx < buckets.len() {
                    buckets[bucket_idx].count += 1;
                    buckets[bucket_idx].bytes += event.size;
                } else {
                    skipped_count += 1;
                }
            }
        }

        if skipped_count > 0 {
            debug!(
                "Allocation rate: {} events skipped due to bucket index out of range",
                skipped_count
            );
        }

        // Filter out empty buckets at the end
        while buckets.last().map(|b| b.count == 0).unwrap_or(false) {
            buckets.pop();
        }

        debug!("Allocation rate: {} buckets computed", buckets.len());
        buckets
    }
}

/// Timeline query result.
#[derive(Debug, Clone)]
pub struct TimelineResult {
    /// Number of events in range
    pub event_count: usize,
    /// Start timestamp
    pub start: u64,
    /// End timestamp
    pub end: u64,
}

/// A timeline event.
#[derive(Debug, Clone)]
pub struct TimelineEvent {
    /// Event timestamp
    pub timestamp: u64,
    /// Event type
    pub event_type: String,
    /// Memory pointer
    pub ptr: usize,
    /// Allocation size
    pub size: usize,
    /// Thread ID
    pub thread_id: u64,
}

/// Allocation rate over a time period.
#[derive(Debug, Clone)]
pub struct AllocationRate {
    /// Start of period
    pub start: u64,
    /// End of period
    pub end: u64,
    /// Number of allocations
    pub count: usize,
    /// Total bytes allocated
    pub bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::MemoryEvent;

    #[test]
    fn test_timeline_query() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 2),
        ];
        let view = MemoryView::from_events(events);
        let analysis = TimelineAnalysis::from_view(&view);
        let result = analysis.query(0, u64::MAX);
        assert_eq!(result.event_count, 2);
    }

    #[test]
    fn test_allocation_rate() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 1),
            MemoryEvent::allocate(0x3000, 256, 1),
        ];
        let view = MemoryView::from_events(events);
        let analysis = TimelineAnalysis::from_view(&view);
        let rates = analysis.allocation_rate(1); // 1ms buckets
        assert!(!rates.is_empty());
        let total_count: usize = rates.iter().map(|r| r.count).sum();
        assert_eq!(total_count, 3);
    }

    #[test]
    fn test_allocation_rate_empty() {
        let events: Vec<MemoryEvent> = vec![];
        let view = MemoryView::from_events(events);
        let analysis = TimelineAnalysis::from_view(&view);
        let rates = analysis.allocation_rate(1);
        assert!(rates.is_empty());
    }
}
