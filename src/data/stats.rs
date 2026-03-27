//! Tracking statistics structures
//!
//! Summary statistics for all tracking operations

use serde::{Deserialize, Serialize};

/// Tracking statistics
///
/// Summary statistics for all tracking operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingStats {
    /// Total number of allocations
    pub total_allocations: u64,
    /// Total number of deallocations
    pub total_deallocations: u64,
    /// Peak memory usage in bytes
    pub peak_memory: u64,
    /// Current active allocations
    pub active_allocations: u64,
    /// Current active memory in bytes
    pub active_memory: u64,
    /// Number of leaked allocations
    pub leaked_allocations: u64,
    /// Leaked memory in bytes
    pub leaked_memory: u64,
    /// Total bytes allocated
    pub total_allocated: u64,
    /// Total bytes deallocated
    pub total_deallocated: u64,
    /// Fragmentation ratio (0.0 to 1.0)
    pub fragmentation_ratio: f64,
    /// Average allocation size (computed field)
    #[serde(skip)]
    pub average_allocation_size: usize,
    /// Current allocated bytes (alias for active_memory)
    #[serde(skip)]
    pub current_allocated: usize,
    /// Fragmentation percentage (computed field)
    #[serde(skip)]
    pub fragmentation: f64,
    /// Allocation count (alias for total_allocations, for compatibility)
    #[serde(skip)]
    pub allocation_count: u64,
    /// Deallocation count (alias for total_deallocations, for compatibility)
    #[serde(skip)]
    pub deallocation_count: u64,
}

impl TrackingStats {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self {
            total_allocations: 0,
            total_deallocations: 0,
            peak_memory: 0,
            active_allocations: 0,
            active_memory: 0,
            leaked_allocations: 0,
            leaked_memory: 0,
            total_allocated: 0,
            total_deallocated: 0,
            fragmentation_ratio: 0.0,
            average_allocation_size: 0,
            current_allocated: 0,
            fragmentation: 0.0,
            allocation_count: 0,
            deallocation_count: 0,
        }
    }

    /// Update computed fields
    pub fn update_computed_fields(&mut self) {
        self.current_allocated = self.active_memory as usize;
        self.average_allocation_size = if self.total_allocations > 0 {
            (self.total_allocated / self.total_allocations) as usize
        } else {
            0
        };
        self.fragmentation = self.fragmentation_ratio * 100.0;
        self.allocation_count = self.total_allocations;
        self.deallocation_count = self.total_deallocations;
    }

    /// Get allocation count (alias for total_allocations)
    pub fn allocation_count(&self) -> u64 {
        self.total_allocations
    }

    /// Get deallocation count (alias for total_deallocations)
    pub fn deallocation_count(&self) -> u64 {
        self.total_deallocations
    }

    /// Calculate fragmentation ratio
    pub fn calculate_fragmentation(&mut self) {
        if self.peak_memory > 0 {
            self.fragmentation_ratio = 1.0 - (self.active_memory as f64 / self.peak_memory as f64);
        }
        self.update_computed_fields();
    }

    /// Get memory efficiency (1.0 - fragmentation)
    pub fn memory_efficiency(&self) -> f64 {
        1.0 - self.fragmentation_ratio
    }

    /// Get allocation rate (allocations per second, if time provided)
    pub fn allocation_rate(&self, duration_secs: f64) -> f64 {
        if duration_secs > 0.0 {
            self.total_allocations as f64 / duration_secs
        } else {
            0.0
        }
    }

    /// Get average allocation size
    pub fn avg_allocation_size(&self) -> usize {
        if self.total_allocations > 0 {
            (self.total_allocated / self.total_allocations) as usize
        } else {
            0
        }
    }
}

impl Default for TrackingStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracking_stats_new() {
        let stats = TrackingStats::new();
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.total_deallocations, 0);
        assert_eq!(stats.peak_memory, 0);
        assert_eq!(stats.active_memory, 0);
        assert_eq!(stats.fragmentation_ratio, 0.0);
    }

    #[test]
    fn test_tracking_stats_default() {
        let stats = TrackingStats::default();
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.fragmentation_ratio, 0.0);
    }

    #[test]
    fn test_tracking_stats_calculate_fragmentation() {
        let mut stats = TrackingStats::new();
        stats.peak_memory = 10000;
        stats.active_memory = 8000;

        stats.calculate_fragmentation();
        assert_eq!(stats.fragmentation_ratio, 0.2);
        assert_eq!(stats.memory_efficiency(), 0.8);
    }

    #[test]
    fn test_tracking_stats_zero_peak() {
        let mut stats = TrackingStats::new();
        stats.active_memory = 1000;

        stats.calculate_fragmentation();
        assert_eq!(stats.fragmentation_ratio, 0.0);
    }

    #[test]
    fn test_tracking_stats_avg_allocation_size() {
        let mut stats = TrackingStats::new();
        stats.total_allocations = 10;
        stats.total_allocated = 10240;

        assert_eq!(stats.avg_allocation_size(), 1024);
    }

    #[test]
    fn test_tracking_stats_zero_allocations() {
        let stats = TrackingStats::new();
        assert_eq!(stats.avg_allocation_size(), 0);
    }

    #[test]
    fn test_tracking_stats_allocation_rate() {
        let mut stats = TrackingStats::new();
        stats.total_allocations = 100;

        let rate = stats.allocation_rate(10.0);
        assert_eq!(rate, 10.0);
    }

    #[test]
    fn test_tracking_stats_zero_duration() {
        let mut stats = TrackingStats::new();
        stats.total_allocations = 100;

        let rate = stats.allocation_rate(0.0);
        assert_eq!(rate, 0.0);
    }
}
