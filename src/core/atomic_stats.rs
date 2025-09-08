//! Atomic statistics system for lock-free performance monitoring
//!
//! This module provides atomic-based statistics to replace mutex-protected
//! counters, reducing lock contention and improving performance.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

/// Cold path statistics stored in mutex-protected structure
#[derive(Debug, Default)]
struct ColdStats {
    active_allocations: u64,
    active_memory: u64,
    peak_allocations: u64,
    peak_memory: u64,
    total_deallocations: u64,
    total_deallocated: u64,
    leaked_allocations: u64,
    leaked_memory: u64,
}

/// Simple memory statistics optimized for cache line efficiency
/// Hot path operations use only the first cache line (16 bytes)
#[repr(align(64))]
#[derive(Debug)]
pub struct SimpleMemoryStats {
    /// Hot path: allocation count (8 bytes)
    pub allocation_count: AtomicU64,
    /// Hot path: total allocated bytes (8 bytes)
    pub total_allocated: AtomicU64,
    /// Cold path: detailed statistics
    detailed: Mutex<ColdStats>,
}

impl SimpleMemoryStats {
    /// Create new simple memory statistics
    pub fn new() -> Self {
        Self {
            allocation_count: AtomicU64::new(0),
            total_allocated: AtomicU64::new(0),
            detailed: Mutex::new(ColdStats::default()),
        }
    }

    /// Record allocation - fast path for hot operations
    pub fn record_allocation_fast(&self, size: u64) {
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        self.total_allocated.fetch_add(size, Ordering::Relaxed);
    }

    /// Record allocation with detailed tracking - slower path
    pub fn record_allocation_detailed(&self, size: u64) {
        // Fast path first
        self.record_allocation_fast(size);

        // Detailed tracking
        if let Ok(mut stats) = self.detailed.try_lock() {
            stats.active_allocations += 1;
            stats.active_memory += size;

            // Update peaks
            if stats.active_allocations > stats.peak_allocations {
                stats.peak_allocations = stats.active_allocations;
            }
            if stats.active_memory > stats.peak_memory {
                stats.peak_memory = stats.active_memory;
            }
        }
    }

    /// Record deallocation
    pub fn record_deallocation(&self, size: u64) {
        if let Ok(mut stats) = self.detailed.try_lock() {
            stats.total_deallocations += 1;
            stats.total_deallocated += size;
            stats.active_allocations = stats.active_allocations.saturating_sub(1);
            stats.active_memory = stats.active_memory.saturating_sub(size);
        }
    }

    /// Record leaked allocation
    pub fn record_leak(&self, size: u64) {
        if let Ok(mut stats) = self.detailed.try_lock() {
            stats.leaked_allocations += 1;
            stats.leaked_memory += size;
        }
    }

    /// Get snapshot of current statistics
    pub fn snapshot(&self) -> MemoryStatsSnapshot {
        let allocation_count = self.allocation_count.load(Ordering::Relaxed);
        let total_allocated = self.total_allocated.load(Ordering::Relaxed);

        // Try to get detailed stats, use defaults if locked
        let detailed = self
            .detailed
            .try_lock()
            .map(|stats| ColdStats {
                active_allocations: stats.active_allocations,
                active_memory: stats.active_memory,
                peak_allocations: stats.peak_allocations,
                peak_memory: stats.peak_memory,
                total_deallocations: stats.total_deallocations,
                total_deallocated: stats.total_deallocated,
                leaked_allocations: stats.leaked_allocations,
                leaked_memory: stats.leaked_memory,
            })
            .unwrap_or_default();

        MemoryStatsSnapshot {
            total_allocations: allocation_count,
            total_allocated,
            active_allocations: detailed.active_allocations,
            active_memory: detailed.active_memory,
            peak_allocations: detailed.peak_allocations,
            peak_memory: detailed.peak_memory,
            total_deallocations: detailed.total_deallocations,
            total_deallocated: detailed.total_deallocated,
            leaked_allocations: detailed.leaked_allocations,
            leaked_memory: detailed.leaked_memory,
        }
    }

    /// Reset all statistics
    pub fn reset(&self) {
        self.allocation_count.store(0, Ordering::Relaxed);
        self.total_allocated.store(0, Ordering::Relaxed);

        if let Ok(mut stats) = self.detailed.try_lock() {
            *stats = ColdStats::default();
        }
    }
}

impl Default for SimpleMemoryStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Atomic memory statistics for lock-free updates
#[derive(Debug)]
pub struct AtomicMemoryStats {
    /// Total number of allocations made
    pub total_allocations: AtomicU64,
    /// Total bytes allocated
    pub total_allocated: AtomicU64,
    /// Current number of active allocations
    pub active_allocations: AtomicU64,
    /// Current active memory in bytes
    pub active_memory: AtomicU64,
    /// Peak number of allocations
    pub peak_allocations: AtomicU64,
    /// Peak memory usage in bytes
    pub peak_memory: AtomicU64,
    /// Number of deallocations
    pub total_deallocations: AtomicU64,
    /// Total bytes deallocated
    pub total_deallocated: AtomicU64,
    /// Number of leaked allocations
    pub leaked_allocations: AtomicU64,
    /// Bytes in leaked allocations
    pub leaked_memory: AtomicU64,
}

impl AtomicMemoryStats {
    /// Create new atomic memory statistics
    pub fn new() -> Self {
        Self {
            total_allocations: AtomicU64::new(0),
            total_allocated: AtomicU64::new(0),
            active_allocations: AtomicU64::new(0),
            active_memory: AtomicU64::new(0),
            peak_allocations: AtomicU64::new(0),
            peak_memory: AtomicU64::new(0),
            total_deallocations: AtomicU64::new(0),
            total_deallocated: AtomicU64::new(0),
            leaked_allocations: AtomicU64::new(0),
            leaked_memory: AtomicU64::new(0),
        }
    }

    /// Record a new allocation
    pub fn record_allocation(&self, size: u64) {
        self.total_allocations.fetch_add(1, Ordering::Relaxed);
        self.total_allocated.fetch_add(size, Ordering::Relaxed);
        self.active_allocations.fetch_add(1, Ordering::Relaxed);
        let new_active_memory = self.active_memory.fetch_add(size, Ordering::Relaxed) + size;

        // Update peaks atomically
        self.update_peak_allocations();
        self.update_peak_memory(new_active_memory);
    }

    /// Record a deallocation
    pub fn record_deallocation(&self, size: u64) {
        self.total_deallocations.fetch_add(1, Ordering::Relaxed);
        self.total_deallocated.fetch_add(size, Ordering::Relaxed);
        self.active_allocations.fetch_sub(1, Ordering::Relaxed);
        self.active_memory.fetch_sub(size, Ordering::Relaxed);
    }

    /// Record a leaked allocation
    pub fn record_leak(&self, size: u64) {
        self.leaked_allocations.fetch_add(1, Ordering::Relaxed);
        self.leaked_memory.fetch_add(size, Ordering::Relaxed);
    }

    /// Update peak allocations atomically
    fn update_peak_allocations(&self) {
        let current_active = self.active_allocations.load(Ordering::Relaxed);
        let mut current_peak = self.peak_allocations.load(Ordering::Relaxed);

        while current_active > current_peak {
            match self.peak_allocations.compare_exchange_weak(
                current_peak,
                current_active,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_peak = actual,
            }
        }
    }

    /// Update peak memory atomically
    fn update_peak_memory(&self, new_memory: u64) {
        let mut current_peak = self.peak_memory.load(Ordering::Relaxed);

        while new_memory > current_peak {
            match self.peak_memory.compare_exchange_weak(
                current_peak,
                new_memory,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_peak = actual,
            }
        }
    }

    /// Get a snapshot of current statistics
    pub fn snapshot(&self) -> MemoryStatsSnapshot {
        MemoryStatsSnapshot {
            total_allocations: self.total_allocations.load(Ordering::Relaxed),
            total_allocated: self.total_allocated.load(Ordering::Relaxed),
            active_allocations: self.active_allocations.load(Ordering::Relaxed),
            active_memory: self.active_memory.load(Ordering::Relaxed),
            peak_allocations: self.peak_allocations.load(Ordering::Relaxed),
            peak_memory: self.peak_memory.load(Ordering::Relaxed),
            total_deallocations: self.total_deallocations.load(Ordering::Relaxed),
            total_deallocated: self.total_deallocated.load(Ordering::Relaxed),
            leaked_allocations: self.leaked_allocations.load(Ordering::Relaxed),
            leaked_memory: self.leaked_memory.load(Ordering::Relaxed),
        }
    }

    /// Reset all statistics
    pub fn reset(&self) {
        self.total_allocations.store(0, Ordering::Relaxed);
        self.total_allocated.store(0, Ordering::Relaxed);
        self.active_allocations.store(0, Ordering::Relaxed);
        self.active_memory.store(0, Ordering::Relaxed);
        self.peak_allocations.store(0, Ordering::Relaxed);
        self.peak_memory.store(0, Ordering::Relaxed);
        self.total_deallocations.store(0, Ordering::Relaxed);
        self.total_deallocated.store(0, Ordering::Relaxed);
        self.leaked_allocations.store(0, Ordering::Relaxed);
        self.leaked_memory.store(0, Ordering::Relaxed);
    }
}

impl Default for AtomicMemoryStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of memory statistics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatsSnapshot {
    pub total_allocations: u64,
    pub total_allocated: u64,
    pub active_allocations: u64,
    pub active_memory: u64,
    pub peak_allocations: u64,
    pub peak_memory: u64,
    pub total_deallocations: u64,
    pub total_deallocated: u64,
    pub leaked_allocations: u64,
    pub leaked_memory: u64,
}

/// Atomic performance counters for various operations
#[derive(Debug)]
pub struct AtomicPerformanceCounters {
    /// Number of clone operations
    pub clone_count: AtomicU64,
    /// Number of lock acquisitions
    pub lock_acquisitions: AtomicU64,
    /// Number of lock contentions
    pub lock_contentions: AtomicU64,
    /// Total time spent waiting for locks (nanoseconds)
    pub lock_wait_time_ns: AtomicU64,
    /// Number of cache hits
    pub cache_hits: AtomicU64,
    /// Number of cache misses
    pub cache_misses: AtomicU64,
}

impl AtomicPerformanceCounters {
    /// Create new performance counters
    pub fn new() -> Self {
        Self {
            clone_count: AtomicU64::new(0),
            lock_acquisitions: AtomicU64::new(0),
            lock_contentions: AtomicU64::new(0),
            lock_wait_time_ns: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        }
    }

    /// Record a clone operation
    pub fn record_clone(&self) {
        self.clone_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a lock acquisition
    pub fn record_lock_acquisition(&self, wait_time: Duration) {
        self.lock_acquisitions.fetch_add(1, Ordering::Relaxed);
        self.lock_wait_time_ns
            .fetch_add(wait_time.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Record lock contention
    pub fn record_lock_contention(&self) {
        self.lock_contentions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Get performance snapshot
    pub fn snapshot(&self) -> PerformanceSnapshot {
        PerformanceSnapshot {
            clone_count: self.clone_count.load(Ordering::Relaxed),
            lock_acquisitions: self.lock_acquisitions.load(Ordering::Relaxed),
            lock_contentions: self.lock_contentions.load(Ordering::Relaxed),
            lock_wait_time_ns: self.lock_wait_time_ns.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
        }
    }
}

impl Default for AtomicPerformanceCounters {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of performance counters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub clone_count: u64,
    pub lock_acquisitions: u64,
    pub lock_contentions: u64,
    pub lock_wait_time_ns: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl PerformanceSnapshot {
    /// Calculate cache hit ratio
    pub fn cache_hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Calculate average lock wait time
    pub fn avg_lock_wait_time_ns(&self) -> f64 {
        if self.lock_acquisitions > 0 {
            self.lock_wait_time_ns as f64 / self.lock_acquisitions as f64
        } else {
            0.0
        }
    }

    /// Calculate lock contention ratio
    pub fn lock_contention_ratio(&self) -> f64 {
        if self.lock_acquisitions > 0 {
            self.lock_contentions as f64 / self.lock_acquisitions as f64
        } else {
            0.0
        }
    }
}

/// Global atomic memory statistics instance
static GLOBAL_ATOMIC_STATS: std::sync::OnceLock<AtomicMemoryStats> = std::sync::OnceLock::new();

/// Get global atomic memory statistics
pub fn get_global_atomic_stats() -> &'static AtomicMemoryStats {
    GLOBAL_ATOMIC_STATS.get_or_init(AtomicMemoryStats::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_simple_memory_stats_creation() {
        let stats = SimpleMemoryStats::new();
        let snapshot = stats.snapshot();

        assert_eq!(snapshot.total_allocations, 0);
        assert_eq!(snapshot.total_allocated, 0);
        assert_eq!(snapshot.active_allocations, 0);
        assert_eq!(snapshot.active_memory, 0);
    }

    #[test]
    fn test_simple_memory_stats_fast_allocation() {
        let stats = SimpleMemoryStats::new();
        stats.record_allocation_fast(1024);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.total_allocations, 1);
        assert_eq!(snapshot.total_allocated, 1024);
    }

    #[test]
    fn test_simple_memory_stats_detailed_allocation() {
        let stats = SimpleMemoryStats::new();
        stats.record_allocation_detailed(2048);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.total_allocations, 1);
        assert_eq!(snapshot.total_allocated, 2048);
        assert_eq!(snapshot.active_allocations, 1);
        assert_eq!(snapshot.active_memory, 2048);
        assert_eq!(snapshot.peak_allocations, 1);
        assert_eq!(snapshot.peak_memory, 2048);
    }

    #[test]
    fn test_simple_memory_stats_deallocation() {
        let stats = SimpleMemoryStats::new();
        stats.record_allocation_detailed(1024);
        stats.record_deallocation(1024);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.total_deallocations, 1);
        assert_eq!(snapshot.total_deallocated, 1024);
        assert_eq!(snapshot.active_allocations, 0);
        assert_eq!(snapshot.active_memory, 0);
    }

    #[test]
    fn test_simple_memory_stats_leak() {
        let stats = SimpleMemoryStats::new();
        stats.record_leak(512);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.leaked_allocations, 1);
        assert_eq!(snapshot.leaked_memory, 512);
    }

    #[test]
    fn test_simple_memory_stats_reset() {
        let stats = SimpleMemoryStats::new();
        stats.record_allocation_fast(1024);
        stats.record_leak(256);

        stats.reset();
        let snapshot = stats.snapshot();

        assert_eq!(snapshot.total_allocations, 0);
        assert_eq!(snapshot.total_allocated, 0);
    }

    #[test]
    fn test_atomic_memory_stats_creation() {
        let stats = AtomicMemoryStats::new();
        let snapshot = stats.snapshot();

        assert_eq!(snapshot.total_allocations, 0);
        assert_eq!(snapshot.total_allocated, 0);
        assert_eq!(snapshot.active_allocations, 0);
        assert_eq!(snapshot.active_memory, 0);
    }

    #[test]
    fn test_atomic_memory_stats_allocation() {
        let stats = AtomicMemoryStats::new();
        stats.record_allocation(1024);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.total_allocations, 1);
        assert_eq!(snapshot.total_allocated, 1024);
        assert_eq!(snapshot.active_allocations, 1);
        assert_eq!(snapshot.active_memory, 1024);
        assert_eq!(snapshot.peak_allocations, 1);
        assert_eq!(snapshot.peak_memory, 1024);
    }

    #[test]
    fn test_atomic_memory_stats_multiple_allocations() {
        let stats = AtomicMemoryStats::new();

        stats.record_allocation(512);
        stats.record_allocation(1024);
        stats.record_allocation(256);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.total_allocations, 3);
        assert_eq!(snapshot.total_allocated, 1792);
        assert_eq!(snapshot.active_allocations, 3);
        assert_eq!(snapshot.active_memory, 1792);
        assert_eq!(snapshot.peak_memory, 1792);
    }

    #[test]
    fn test_atomic_memory_stats_deallocation() {
        let stats = AtomicMemoryStats::new();
        stats.record_allocation(1024);
        stats.record_deallocation(1024);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.total_allocations, 1);
        assert_eq!(snapshot.total_deallocations, 1);
        assert_eq!(snapshot.total_deallocated, 1024);
        assert_eq!(snapshot.active_allocations, 0);
        assert_eq!(snapshot.active_memory, 0);
    }

    #[test]
    fn test_atomic_memory_stats_peak_tracking() {
        let stats = AtomicMemoryStats::new();

        // Build up to peak
        stats.record_allocation(1000);
        stats.record_allocation(2000);
        let peak_snapshot = stats.snapshot();

        // Deallocate one
        stats.record_deallocation(1000);
        let current_snapshot = stats.snapshot();

        // Peak should remain at the maximum
        assert_eq!(peak_snapshot.peak_memory, 3000);
        assert_eq!(current_snapshot.peak_memory, 3000);
        assert_eq!(current_snapshot.active_memory, 2000);
    }

    #[test]
    fn test_atomic_memory_stats_leak() {
        let stats = AtomicMemoryStats::new();
        stats.record_leak(512);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.leaked_allocations, 1);
        assert_eq!(snapshot.leaked_memory, 512);
    }

    #[test]
    fn test_atomic_memory_stats_reset() {
        let stats = AtomicMemoryStats::new();
        stats.record_allocation(1024);
        stats.record_leak(256);

        stats.reset();
        let snapshot = stats.snapshot();

        assert_eq!(snapshot.total_allocations, 0);
        assert_eq!(snapshot.total_allocated, 0);
        assert_eq!(snapshot.leaked_allocations, 0);
        assert_eq!(snapshot.leaked_memory, 0);
    }

    #[test]
    fn test_atomic_performance_counters_creation() {
        let counters = AtomicPerformanceCounters::new();
        let snapshot = counters.snapshot();

        assert_eq!(snapshot.clone_count, 0);
        assert_eq!(snapshot.lock_acquisitions, 0);
        assert_eq!(snapshot.lock_contentions, 0);
        assert_eq!(snapshot.cache_hits, 0);
        assert_eq!(snapshot.cache_misses, 0);
    }

    #[test]
    fn test_atomic_performance_counters_clone() {
        let counters = AtomicPerformanceCounters::new();
        counters.record_clone();
        counters.record_clone();

        let snapshot = counters.snapshot();
        assert_eq!(snapshot.clone_count, 2);
    }

    #[test]
    fn test_atomic_performance_counters_lock_acquisition() {
        let counters = AtomicPerformanceCounters::new();
        let wait_time = Duration::from_millis(10);
        counters.record_lock_acquisition(wait_time);

        let snapshot = counters.snapshot();
        assert_eq!(snapshot.lock_acquisitions, 1);
        assert_eq!(snapshot.lock_wait_time_ns, wait_time.as_nanos() as u64);
    }

    #[test]
    fn test_atomic_performance_counters_lock_contention() {
        let counters = AtomicPerformanceCounters::new();
        counters.record_lock_contention();
        counters.record_lock_contention();

        let snapshot = counters.snapshot();
        assert_eq!(snapshot.lock_contentions, 2);
    }

    #[test]
    fn test_atomic_performance_counters_cache() {
        let counters = AtomicPerformanceCounters::new();
        counters.record_cache_hit();
        counters.record_cache_hit();
        counters.record_cache_miss();

        let snapshot = counters.snapshot();
        assert_eq!(snapshot.cache_hits, 2);
        assert_eq!(snapshot.cache_misses, 1);
    }

    #[test]
    fn test_performance_snapshot_calculations() {
        let snapshot = PerformanceSnapshot {
            clone_count: 100,
            lock_acquisitions: 80,
            lock_contentions: 8,
            lock_wait_time_ns: 800_000,
            cache_hits: 90,
            cache_misses: 10,
        };

        // Test cache hit ratio
        let hit_ratio = snapshot.cache_hit_ratio();
        assert!((hit_ratio - 0.9).abs() < f64::EPSILON);

        // Test average lock wait time
        let avg_wait = snapshot.avg_lock_wait_time_ns();
        assert!((avg_wait - 10_000.0).abs() < f64::EPSILON);

        // Test lock contention ratio
        let contention_ratio = snapshot.lock_contention_ratio();
        assert!((contention_ratio - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_performance_snapshot_edge_cases() {
        let empty_snapshot = PerformanceSnapshot {
            clone_count: 0,
            lock_acquisitions: 0,
            lock_contentions: 0,
            lock_wait_time_ns: 0,
            cache_hits: 0,
            cache_misses: 0,
        };

        assert_eq!(empty_snapshot.cache_hit_ratio(), 0.0);
        assert_eq!(empty_snapshot.avg_lock_wait_time_ns(), 0.0);
        assert_eq!(empty_snapshot.lock_contention_ratio(), 0.0);
    }

    #[test]
    fn test_memory_stats_snapshot_creation() {
        let snapshot = MemoryStatsSnapshot {
            total_allocations: 100,
            total_allocated: 10240,
            active_allocations: 50,
            active_memory: 5120,
            peak_allocations: 80,
            peak_memory: 8192,
            total_deallocations: 50,
            total_deallocated: 5120,
            leaked_allocations: 5,
            leaked_memory: 512,
        };

        assert_eq!(snapshot.total_allocations, 100);
        assert_eq!(snapshot.total_allocated, 10240);
        assert_eq!(snapshot.active_allocations, 50);
        assert_eq!(snapshot.peak_memory, 8192);
        assert_eq!(snapshot.leaked_allocations, 5);
    }

    #[test]
    fn test_default_implementations() {
        let simple_stats = SimpleMemoryStats::default();
        let atomic_stats = AtomicMemoryStats::default();
        let counters = AtomicPerformanceCounters::default();

        assert_eq!(simple_stats.snapshot().total_allocations, 0);
        assert_eq!(atomic_stats.snapshot().total_allocations, 0);
        assert_eq!(counters.snapshot().clone_count, 0);
    }

    #[test]
    fn test_global_atomic_stats() {
        let stats = get_global_atomic_stats();

        // Test that we get the same instance
        let stats2 = get_global_atomic_stats();
        assert!(std::ptr::eq(stats, stats2));

        // Test basic functionality
        stats.record_allocation(1024);
        let snapshot = stats.snapshot();
        assert!(snapshot.total_allocations > 0);
    }
}
