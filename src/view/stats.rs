//! View statistics.

use crate::snapshot::MemorySnapshot;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// View statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViewStats {
    /// Total number of allocations
    pub allocation_count: usize,
    /// Total number of events
    pub event_count: usize,
    /// Total bytes currently allocated
    pub total_bytes: usize,
    /// Peak memory usage
    pub peak_bytes: usize,
    /// Number of threads
    pub thread_count: usize,
    /// Number of unique types
    pub type_count: usize,
    /// Number of deallocations
    pub deallocation_count: usize,
    /// Number of reallocations
    pub reallocation_count: usize,
}

impl ViewStats {
    /// Create stats from snapshot.
    pub fn from_snapshot(snapshot: &MemorySnapshot) -> Self {
        let types: HashSet<&str> = snapshot
            .active_allocations
            .values()
            .filter_map(|a| a.type_name.as_deref())
            .collect();

        Self {
            allocation_count: snapshot.stats.active_allocations,
            event_count: 0, // Will be set separately if needed
            total_bytes: snapshot.stats.current_memory,
            peak_bytes: snapshot.stats.peak_memory,
            thread_count: snapshot.thread_stats.len(),
            type_count: types.len(),
            deallocation_count: snapshot.stats.total_deallocations,
            reallocation_count: snapshot.stats.total_reallocations,
        }
    }

    /// Create stats from view.
    pub fn from_view(view: &super::MemoryView) -> Self {
        let mut stats = Self::from_snapshot(view.snapshot());
        stats.event_count = view.events().len();
        stats
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.allocation_count == 0
    }

    /// Get average allocation size.
    pub fn avg_allocation_size(&self) -> usize {
        if self.allocation_count == 0 {
            0
        } else {
            self.total_bytes / self.allocation_count
        }
    }

    /// Get memory efficiency (peak vs current).
    pub fn memory_efficiency(&self) -> f64 {
        if self.peak_bytes == 0 {
            1.0
        } else {
            self.total_bytes as f64 / self.peak_bytes as f64
        }
    }
}

impl std::fmt::Display for ViewStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ViewStats {{ allocations: {}, events: {}, bytes: {}, peak: {}, threads: {}, types: {} }}",
            self.allocation_count,
            self.event_count,
            self.total_bytes,
            self.peak_bytes,
            self.thread_count,
            self.type_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::MemoryEvent;

    #[test]
    fn test_empty_stats() {
        let view = super::super::MemoryView::from_events(vec![]);
        let stats = ViewStats::from_view(&view);
        assert!(stats.is_empty());
        assert_eq!(stats.allocation_count, 0);
    }

    #[test]
    fn test_stats_with_allocations() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 2),
        ];
        let view = super::super::MemoryView::from_events(events);
        let stats = ViewStats::from_view(&view);
        assert_eq!(stats.allocation_count, 2);
        assert_eq!(stats.total_bytes, 192);
        assert_eq!(stats.thread_count, 2);
    }

    #[test]
    fn test_view_stats_default() {
        let stats = ViewStats::default();
        assert!(stats.is_empty());
        assert_eq!(stats.allocation_count, 0);
        assert_eq!(stats.event_count, 0);
        assert_eq!(stats.total_bytes, 0);
        assert_eq!(stats.peak_bytes, 0);
        assert_eq!(stats.thread_count, 0);
        assert_eq!(stats.type_count, 0);
    }

    #[test]
    fn test_view_stats_clone() {
        let stats = ViewStats {
            allocation_count: 10,
            event_count: 20,
            total_bytes: 1024,
            peak_bytes: 2048,
            thread_count: 4,
            type_count: 5,
            deallocation_count: 2,
            reallocation_count: 1,
        };

        let cloned = stats.clone();
        assert_eq!(cloned.allocation_count, 10);
        assert_eq!(cloned.event_count, 20);
        assert_eq!(cloned.total_bytes, 1024);
    }

    #[test]
    fn test_view_stats_debug() {
        let stats = ViewStats {
            allocation_count: 5,
            event_count: 10,
            total_bytes: 512,
            peak_bytes: 1024,
            thread_count: 2,
            type_count: 3,
            deallocation_count: 1,
            reallocation_count: 0,
        };

        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("ViewStats"));
        assert!(debug_str.contains("allocation_count"));
    }

    #[test]
    fn test_view_stats_display() {
        let stats = ViewStats {
            allocation_count: 5,
            event_count: 10,
            total_bytes: 512,
            peak_bytes: 1024,
            thread_count: 2,
            type_count: 3,
            deallocation_count: 1,
            reallocation_count: 0,
        };

        let display_str = format!("{}", stats);
        assert!(display_str.contains("allocations: 5"));
        assert!(display_str.contains("events: 10"));
        assert!(display_str.contains("bytes: 512"));
    }

    #[test]
    fn test_avg_allocation_size_zero() {
        let stats = ViewStats::default();
        assert_eq!(stats.avg_allocation_size(), 0);
    }

    #[test]
    fn test_avg_allocation_size_nonzero() {
        let stats = ViewStats {
            allocation_count: 4,
            total_bytes: 100,
            ..Default::default()
        };
        assert_eq!(stats.avg_allocation_size(), 25);
    }

    #[test]
    fn test_memory_efficiency_zero_peak() {
        let stats = ViewStats {
            peak_bytes: 0,
            total_bytes: 100,
            ..Default::default()
        };
        assert_eq!(stats.memory_efficiency(), 1.0);
    }

    #[test]
    fn test_memory_efficiency_half() {
        let stats = ViewStats {
            peak_bytes: 100,
            total_bytes: 50,
            ..Default::default()
        };
        assert!((stats.memory_efficiency() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_memory_efficiency_full() {
        let stats = ViewStats {
            peak_bytes: 100,
            total_bytes: 100,
            ..Default::default()
        };
        assert!((stats.memory_efficiency() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_view_stats_serialization() {
        let stats = ViewStats {
            allocation_count: 10,
            event_count: 20,
            total_bytes: 1024,
            peak_bytes: 2048,
            thread_count: 4,
            type_count: 5,
            deallocation_count: 2,
            reallocation_count: 1,
        };

        let json = serde_json::to_string(&stats);
        assert!(json.is_ok());

        let deserialized: Result<ViewStats, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_from_snapshot() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 2),
        ];
        let view = super::super::MemoryView::from_events(events);
        let stats = ViewStats::from_snapshot(view.snapshot());

        assert_eq!(stats.allocation_count, 2);
        assert_eq!(stats.total_bytes, 192);
        assert_eq!(stats.event_count, 0); // from_snapshot doesn't set event_count
    }

    #[test]
    fn test_from_view_event_count() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 2),
            MemoryEvent::deallocate(0x1000, 64, 1),
        ];
        let view = super::super::MemoryView::from_events(events);
        let stats = ViewStats::from_view(&view);

        assert_eq!(stats.event_count, 3);
    }
}
