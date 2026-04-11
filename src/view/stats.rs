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
}
