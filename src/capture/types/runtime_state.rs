//! Runtime state tracking types.
//!
//! This module contains types for tracking runtime state information,
//! including CPU usage, memory pressure, cache performance, and allocator state.

use serde::{Deserialize, Serialize};

/// Runtime state information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeStateInfo {
    /// CPU usage information.
    pub cpu_usage: CpuUsageInfo,
    /// Memory pressure.
    pub memory_pressure: MemoryPressureInfo,
    /// Cache performance.
    pub cache_performance: CachePerformanceInfo,
    /// Allocator state.
    pub allocator_state: AllocatorStateInfo,
    /// Garbage collection information (if applicable).
    pub gc_info: Option<GcInfo>,
}

/// CPU usage information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CpuUsageInfo {
    /// Current CPU usage percentage.
    pub current_usage_percent: f64,
    /// Average CPU usage percentage.
    pub average_usage_percent: f64,
    /// Peak CPU usage percentage.
    pub peak_usage_percent: f64,
    /// CPU intensive operations count.
    pub intensive_operations_count: usize,
}

/// Memory pressure information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryPressureInfo {
    /// Current memory pressure level.
    pub pressure_level: MemoryPressureLevel,
    /// Available memory percentage.
    pub available_memory_percent: f64,
    /// Memory allocation failures count.
    pub allocation_failures: usize,
    /// Memory fragmentation level.
    pub fragmentation_level: f64,
}

/// Memory pressure level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryPressureLevel {
    /// Low memory pressure.
    Low,
    /// Moderate memory pressure.
    Moderate,
    /// High memory pressure.
    High,
    /// Critical memory pressure.
    Critical,
}

/// Cache performance information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachePerformanceInfo {
    /// L1 cache hit rate.
    pub l1_hit_rate: f64,
    /// L2 cache hit rate.
    pub l2_hit_rate: f64,
    /// L3 cache hit rate.
    pub l3_hit_rate: f64,
    /// Cache miss penalty in nanoseconds.
    pub cache_miss_penalty_ns: f64,
    /// Memory access pattern analysis.
    pub access_pattern: MemoryAccessPattern,
}

/// Memory access pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryAccessPattern {
    /// Sequential access pattern.
    Sequential,
    /// Random access pattern.
    Random,
    /// Strided access pattern.
    Strided {
        /// Stride size.
        stride: usize,
    },
    /// Clustered access pattern.
    Clustered,
    /// Mixed access pattern.
    Mixed,
}

/// Allocator state information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllocatorStateInfo {
    /// Allocator type.
    pub allocator_type: String,
    /// Heap size.
    pub heap_size: usize,
    /// Used heap space.
    pub heap_used: usize,
    /// Free blocks count.
    pub free_blocks_count: usize,
    /// Largest free block size.
    pub largest_free_block: usize,
    /// Allocator efficiency score.
    pub efficiency_score: f64,
}

/// Garbage collection information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GcInfo {
    /// GC type.
    pub gc_type: String,
    /// GC runs count.
    pub gc_runs: usize,
    /// Total GC time in milliseconds.
    pub total_gc_time_ms: u64,
    /// Average GC pause time.
    pub average_pause_time_ms: f64,
    /// Memory reclaimed.
    pub memory_reclaimed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_state_info() {
        let state = RuntimeStateInfo {
            cpu_usage: CpuUsageInfo {
                current_usage_percent: 25.0,
                average_usage_percent: 20.0,
                peak_usage_percent: 50.0,
                intensive_operations_count: 10,
            },
            memory_pressure: MemoryPressureInfo {
                pressure_level: MemoryPressureLevel::Low,
                available_memory_percent: 80.0,
                allocation_failures: 0,
                fragmentation_level: 0.1,
            },
            cache_performance: CachePerformanceInfo {
                l1_hit_rate: 0.95,
                l2_hit_rate: 0.85,
                l3_hit_rate: 0.75,
                cache_miss_penalty_ns: 100.0,
                access_pattern: MemoryAccessPattern::Sequential,
            },
            allocator_state: AllocatorStateInfo {
                allocator_type: "system".to_string(),
                heap_size: 1024 * 1024,
                heap_used: 512 * 1024,
                free_blocks_count: 10,
                largest_free_block: 256 * 1024,
                efficiency_score: 0.8,
            },
            gc_info: None,
        };

        assert_eq!(state.cpu_usage.current_usage_percent, 25.0);
        assert!(matches!(
            state.memory_pressure.pressure_level,
            MemoryPressureLevel::Low
        ));
    }
}

// Implement From trait for converting from core::types to capture::types
impl From<crate::core::types::RuntimeStateInfo> for RuntimeStateInfo {
    fn from(old: crate::core::types::RuntimeStateInfo) -> Self {
        Self {
            cpu_usage: CpuUsageInfo {
                current_usage_percent: old.cpu_usage.current_usage_percent,
                average_usage_percent: old.cpu_usage.average_usage_percent,
                peak_usage_percent: old.cpu_usage.peak_usage_percent,
                intensive_operations_count: old.cpu_usage.intensive_operations_count,
            },
            memory_pressure: MemoryPressureInfo {
                pressure_level: match old.memory_pressure.pressure_level {
                    crate::core::types::MemoryPressureLevel::Low => MemoryPressureLevel::Low,
                    crate::core::types::MemoryPressureLevel::Moderate => {
                        MemoryPressureLevel::Moderate
                    }
                    crate::core::types::MemoryPressureLevel::High => MemoryPressureLevel::High,
                    crate::core::types::MemoryPressureLevel::Critical => {
                        MemoryPressureLevel::Critical
                    }
                },
                available_memory_percent: old.memory_pressure.available_memory_percent,
                allocation_failures: old.memory_pressure.allocation_failures,
                fragmentation_level: old.memory_pressure.fragmentation_level,
            },
            cache_performance: CachePerformanceInfo {
                l1_hit_rate: old.cache_performance.l1_hit_rate,
                l2_hit_rate: old.cache_performance.l2_hit_rate,
                l3_hit_rate: old.cache_performance.l3_hit_rate,
                cache_miss_penalty_ns: old.cache_performance.cache_miss_penalty_ns,
                access_pattern: match old.cache_performance.access_pattern {
                    crate::core::types::MemoryAccessPattern::Sequential => {
                        MemoryAccessPattern::Sequential
                    }
                    crate::core::types::MemoryAccessPattern::Random => MemoryAccessPattern::Random,
                    crate::core::types::MemoryAccessPattern::Strided { stride } => {
                        MemoryAccessPattern::Strided { stride }
                    }
                    crate::core::types::MemoryAccessPattern::Clustered => {
                        MemoryAccessPattern::Clustered
                    }
                    crate::core::types::MemoryAccessPattern::Mixed => MemoryAccessPattern::Mixed,
                },
            },
            allocator_state: AllocatorStateInfo {
                allocator_type: old.allocator_state.allocator_type,
                heap_size: old.allocator_state.heap_size,
                heap_used: old.allocator_state.heap_used,
                free_blocks_count: old.allocator_state.free_blocks_count,
                largest_free_block: old.allocator_state.largest_free_block,
                efficiency_score: old.allocator_state.efficiency_score,
            },
            gc_info: old.gc_info.map(|gc| GcInfo {
                gc_type: gc.gc_type,
                gc_runs: gc.gc_runs,
                total_gc_time_ms: gc.total_gc_time_ms,
                average_pause_time_ms: gc.average_pause_time_ms,
                memory_reclaimed: gc.memory_reclaimed,
            }),
        }
    }
}
