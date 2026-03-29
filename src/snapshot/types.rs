//! Snapshot types - Data structures for memory snapshots
//!
//! This module defines the core data structures used by the
//! SnapshotEngine for representing memory snapshots.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Active allocation information in a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveAllocation {
    /// Memory pointer address
    pub ptr: usize,
    /// Allocation size in bytes
    pub size: usize,
    /// Timestamp when this allocation was made
    pub allocated_at: u64,
    /// Optional variable name
    pub var_name: Option<String>,
    /// Optional type name
    pub type_name: Option<String>,
    /// Thread ID that made this allocation
    pub thread_id: u64,
}

/// Memory statistics for a snapshot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryStats {
    /// Total number of allocations in the snapshot
    pub total_allocations: usize,
    /// Total number of deallocations
    pub total_deallocations: usize,
    /// Number of currently active allocations
    pub active_allocations: usize,
    /// Total bytes allocated
    pub total_allocated: usize,
    /// Total bytes deallocated
    pub total_deallocated: usize,
    /// Currently used memory (sum of active allocations)
    pub current_memory: usize,
    /// Peak memory usage observed
    pub peak_memory: usize,
}

/// Thread-specific memory statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreadMemoryStats {
    /// Thread ID
    pub thread_id: u64,
    /// Number of allocations by this thread
    pub allocation_count: usize,
    /// Total bytes allocated by this thread
    pub total_allocated: usize,
    /// Current memory usage by this thread
    pub current_memory: usize,
    /// Peak memory usage by this thread
    pub peak_memory: usize,
}

/// Memory snapshot - a point-in-time view of memory usage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySnapshot {
    /// Timestamp when this snapshot was taken
    pub timestamp: u64,
    /// Overall memory statistics
    pub stats: MemoryStats,
    /// Active allocations (ptr -> allocation info)
    pub active_allocations: HashMap<usize, ActiveAllocation>,
    /// Per-thread statistics
    pub thread_stats: HashMap<u64, ThreadMemoryStats>,
}

impl MemorySnapshot {
    /// Create a new empty snapshot
    pub fn new() -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            stats: MemoryStats::default(),
            active_allocations: HashMap::new(),
            thread_stats: HashMap::new(),
        }
    }

    /// Get the number of active allocations
    pub fn active_count(&self) -> usize {
        self.active_allocations.len()
    }

    /// Get the current memory usage
    pub fn current_memory(&self) -> usize {
        self.stats.current_memory
    }

    /// Get the peak memory usage
    pub fn peak_memory(&self) -> usize {
        self.stats.peak_memory
    }
}
