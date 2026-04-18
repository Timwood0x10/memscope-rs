//! Snapshot types - Data structures for memory snapshots
//!
//! This module defines the core data structures used by the
//! SnapshotEngine for representing memory snapshots.

use crate::core::types::TrackKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Active allocation information in a snapshot
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveAllocation {
    /// Memory pointer address (None for Container/Value types)
    pub ptr: Option<usize>,
    /// Allocation size in bytes
    pub size: usize,
    /// Memory allocation semantic role
    pub kind: TrackKind,
    /// Timestamp when this allocation was made
    pub allocated_at: u64,
    /// Optional variable name
    pub var_name: Option<String>,
    /// Optional type name
    pub type_name: Option<String>,
    /// Thread ID that made this allocation
    pub thread_id: u64,
    /// Optional call stack hash for clone detection
    pub call_stack_hash: Option<u64>,
    /// Module path (from module_path!())
    pub module_path: Option<String>,
}

/// Memory statistics for a snapshot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryStats {
    /// Total number of allocations in the snapshot
    pub total_allocations: usize,
    /// Total number of reallocations
    pub total_reallocations: usize,
    /// Total number of deallocations
    pub total_deallocations: usize,
    /// Number of deallocations without matching allocations
    pub unmatched_deallocations: usize,
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
    /// Total bytes deallocated by this thread
    pub total_deallocated: usize,
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

    /// Build a MemorySnapshot from a list of AllocationInfo (capture module type)
    pub fn from_allocation_infos(allocations: Vec<crate::capture::types::AllocationInfo>) -> Self {
        let mut snapshot = Self::new();
        let mut thread_stats: HashMap<u64, ThreadMemoryStats> = HashMap::new();
        let mut current_memory: usize = 0;

        for alloc in allocations {
            let thread_id = alloc.thread_id_u64;

            let active_alloc = ActiveAllocation {
                ptr: Some(alloc.ptr),
                size: alloc.size,
                kind: TrackKind::HeapOwner {
                    ptr: alloc.ptr,
                    size: alloc.size,
                },
                allocated_at: alloc.timestamp_alloc,
                var_name: alloc.var_name,
                type_name: alloc.type_name,
                thread_id,
                call_stack_hash: None,
                module_path: alloc.module_path,
            };

            current_memory += alloc.size;

            snapshot.stats.total_allocations += 1;
            snapshot.stats.total_allocated += alloc.size;

            let thread_stat = thread_stats
                .entry(thread_id)
                .or_insert_with(|| ThreadMemoryStats {
                    thread_id,
                    allocation_count: 0,
                    total_allocated: 0,
                    total_deallocated: 0,
                    current_memory: 0,
                    peak_memory: 0,
                });

            thread_stat.allocation_count += 1;
            thread_stat.total_allocated += alloc.size;
            thread_stat.current_memory += alloc.size;
            if thread_stat.current_memory > thread_stat.peak_memory {
                thread_stat.peak_memory = thread_stat.current_memory;
            }

            snapshot.active_allocations.insert(alloc.ptr, active_alloc);
        }

        snapshot.stats.current_memory = current_memory;
        snapshot.stats.peak_memory = 0; // Cannot determine peak from current allocations only
        snapshot.stats.active_allocations = snapshot.active_allocations.len();
        snapshot.thread_stats = thread_stats;

        snapshot
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_snapshot_new() {
        let snapshot = MemorySnapshot::new();
        assert!(snapshot.timestamp > 0);
        assert_eq!(snapshot.stats.total_allocations, 0);
        assert!(snapshot.active_allocations.is_empty());
    }

    #[test]
    fn test_memory_snapshot_default() {
        let snapshot = MemorySnapshot::default();
        assert_eq!(snapshot.timestamp, 0);
        assert_eq!(snapshot.stats.total_allocations, 0);
    }

    #[test]
    fn test_active_allocation_creation() {
        let alloc = ActiveAllocation {
            ptr: Some(0x1000),
            size: 1024,
            kind: TrackKind::HeapOwner {
                ptr: 0x1000,
                size: 1024,
            },
            allocated_at: 1000,
            var_name: Some("test".to_string()),
            type_name: Some("Vec<u8>".to_string()),
            thread_id: 1,
            call_stack_hash: None,
            module_path: None,
        };

        assert_eq!(alloc.ptr, Some(0x1000));
        assert_eq!(alloc.size, 1024);
        assert_eq!(alloc.var_name, Some("test".to_string()));
    }

    #[test]
    fn test_active_allocation_clone() {
        let alloc = ActiveAllocation {
            ptr: Some(0x1000),
            size: 1024,
            kind: TrackKind::HeapOwner {
                ptr: 0x1000,
                size: 1024,
            },
            allocated_at: 1000,
            var_name: None,
            type_name: None,
            thread_id: 1,
            call_stack_hash: None,
            module_path: None,
        };

        let cloned = alloc.clone();
        assert_eq!(cloned.size, alloc.size);
    }

    #[test]
    fn test_active_allocation_debug() {
        let alloc = ActiveAllocation {
            ptr: Some(0x1000),
            size: 1024,
            kind: TrackKind::HeapOwner {
                ptr: 0x1000,
                size: 1024,
            },
            allocated_at: 1000,
            var_name: None,
            type_name: None,
            thread_id: 1,
            call_stack_hash: None,
            module_path: None,
        };

        let debug_str = format!("{:?}", alloc);
        assert!(debug_str.contains("ActiveAllocation"));
    }

    #[test]
    fn test_memory_stats_default() {
        let stats = MemoryStats::default();
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.total_reallocations, 0);
        assert_eq!(stats.total_deallocations, 0);
        assert_eq!(stats.active_allocations, 0);
        assert_eq!(stats.current_memory, 0);
        assert_eq!(stats.peak_memory, 0);
    }

    #[test]
    fn test_memory_stats_clone() {
        let stats = MemoryStats {
            total_allocations: 100,
            total_reallocations: 10,
            total_deallocations: 50,
            unmatched_deallocations: 2,
            active_allocations: 50,
            total_allocated: 1024 * 1024,
            total_deallocated: 512 * 1024,
            current_memory: 512 * 1024,
            peak_memory: 1024 * 1024,
        };

        let cloned = stats.clone();
        assert_eq!(cloned.total_allocations, 100);
        assert_eq!(cloned.peak_memory, 1024 * 1024);
    }

    #[test]
    fn test_thread_memory_stats_default() {
        let stats = ThreadMemoryStats::default();
        assert_eq!(stats.thread_id, 0);
        assert_eq!(stats.allocation_count, 0);
        assert_eq!(stats.current_memory, 0);
    }

    #[test]
    fn test_thread_memory_stats_clone() {
        let stats = ThreadMemoryStats {
            thread_id: 1,
            allocation_count: 50,
            total_allocated: 4096,
            total_deallocated: 2048,
            current_memory: 2048,
            peak_memory: 4096,
        };

        let cloned = stats.clone();
        assert_eq!(cloned.thread_id, 1);
        assert_eq!(cloned.allocation_count, 50);
    }

    #[test]
    fn test_memory_snapshot_active_count() {
        let mut snapshot = MemorySnapshot::new();
        assert_eq!(snapshot.active_count(), 0);

        snapshot.active_allocations.insert(
            0x1000,
            ActiveAllocation {
                ptr: Some(0x1000),
                size: 1024,
                kind: TrackKind::HeapOwner {
                    ptr: 0x1000,
                    size: 1024,
                },
                allocated_at: 1000,
                var_name: None,
                type_name: None,
                thread_id: 1,
                call_stack_hash: None,
                module_path: None,
            },
        );

        assert_eq!(snapshot.active_count(), 1);
    }

    #[test]
    fn test_memory_snapshot_current_memory() {
        let mut snapshot = MemorySnapshot::new();
        assert_eq!(snapshot.current_memory(), 0);

        snapshot.stats.current_memory = 4096;
        assert_eq!(snapshot.current_memory(), 4096);
    }

    #[test]
    fn test_memory_snapshot_peak_memory() {
        let mut snapshot = MemorySnapshot::new();
        assert_eq!(snapshot.peak_memory(), 0);

        snapshot.stats.peak_memory = 8192;
        assert_eq!(snapshot.peak_memory(), 8192);
    }

    #[test]
    fn test_memory_snapshot_serialization() {
        let snapshot = MemorySnapshot::new();

        let json = serde_json::to_string(&snapshot);
        assert!(json.is_ok());

        let deserialized: Result<MemorySnapshot, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_active_allocation_serialization() {
        let alloc = ActiveAllocation {
            ptr: Some(0x1000),
            size: 1024,
            kind: TrackKind::HeapOwner {
                ptr: 0x1000,
                size: 1024,
            },
            allocated_at: 1000,
            var_name: Some("test".to_string()),
            type_name: Some("i32".to_string()),
            thread_id: 1,
            call_stack_hash: Some(12345),
            module_path: Some("test_module".to_string()),
        };

        let json = serde_json::to_string(&alloc);
        assert!(json.is_ok());

        let deserialized: Result<ActiveAllocation, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_memory_stats_serialization() {
        let stats = MemoryStats {
            total_allocations: 100,
            total_reallocations: 10,
            total_deallocations: 50,
            unmatched_deallocations: 2,
            active_allocations: 50,
            total_allocated: 1024,
            total_deallocated: 512,
            current_memory: 512,
            peak_memory: 1024,
        };

        let json = serde_json::to_string(&stats);
        assert!(json.is_ok());

        let deserialized: Result<MemoryStats, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }
}
