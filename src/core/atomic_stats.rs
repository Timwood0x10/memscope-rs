//! Atomic statistics system for lock-free performance monitoring
//! 
//! This module provides atomic-based statistics to replace mutex-protected
//! counters, reducing lock contention and improving performance.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use serde::{Serialize, Deserialize};

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
        let detailed = self.detailed.try_lock()
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
        self.lock_wait_time_ns.fetch_add(wait_time.as_nanos() as u64, Ordering::Relaxed);
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