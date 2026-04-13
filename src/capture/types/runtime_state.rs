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

    #[test]
    fn test_cpu_usage_info_creation() {
        let cpu = CpuUsageInfo {
            current_usage_percent: 75.5,
            average_usage_percent: 50.0,
            peak_usage_percent: 95.0,
            intensive_operations_count: 100,
        };

        assert!((cpu.current_usage_percent - 75.5).abs() < f64::EPSILON);
        assert_eq!(cpu.intensive_operations_count, 100);
    }

    #[test]
    fn test_memory_pressure_info_creation() {
        let pressure = MemoryPressureInfo {
            pressure_level: MemoryPressureLevel::High,
            available_memory_percent: 25.0,
            allocation_failures: 5,
            fragmentation_level: 0.6,
        };

        assert!(matches!(pressure.pressure_level, MemoryPressureLevel::High));
        assert_eq!(pressure.allocation_failures, 5);
    }

    #[test]
    fn test_memory_pressure_level_variants() {
        let levels = [
            MemoryPressureLevel::Low,
            MemoryPressureLevel::Moderate,
            MemoryPressureLevel::High,
            MemoryPressureLevel::Critical,
        ];

        for level in levels {
            let info = MemoryPressureInfo {
                pressure_level: level.clone(),
                available_memory_percent: 50.0,
                allocation_failures: 0,
                fragmentation_level: 0.0,
            };
            assert_eq!(info.pressure_level, level);
        }
    }

    #[test]
    fn test_cache_performance_info_creation() {
        let cache = CachePerformanceInfo {
            l1_hit_rate: 0.98,
            l2_hit_rate: 0.92,
            l3_hit_rate: 0.85,
            cache_miss_penalty_ns: 150.0,
            access_pattern: MemoryAccessPattern::Random,
        };

        assert!((cache.l1_hit_rate - 0.98).abs() < f64::EPSILON);
        assert!((cache.cache_miss_penalty_ns - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_memory_access_pattern_variants() {
        let patterns = [
            MemoryAccessPattern::Sequential,
            MemoryAccessPattern::Random,
            MemoryAccessPattern::Strided { stride: 64 },
            MemoryAccessPattern::Clustered,
            MemoryAccessPattern::Mixed,
        ];

        for pattern in patterns {
            let cache = CachePerformanceInfo {
                l1_hit_rate: 0.0,
                l2_hit_rate: 0.0,
                l3_hit_rate: 0.0,
                cache_miss_penalty_ns: 0.0,
                access_pattern: pattern.clone(),
            };
            assert_eq!(cache.access_pattern, pattern);
        }
    }

    #[test]
    fn test_allocator_state_info_creation() {
        let allocator = AllocatorStateInfo {
            allocator_type: "jemalloc".to_string(),
            heap_size: 1024 * 1024 * 1024,
            heap_used: 512 * 1024 * 1024,
            free_blocks_count: 1000,
            largest_free_block: 64 * 1024 * 1024,
            efficiency_score: 0.95,
        };

        assert_eq!(allocator.allocator_type, "jemalloc");
        assert_eq!(allocator.free_blocks_count, 1000);
    }

    #[test]
    fn test_gc_info_creation() {
        let gc = GcInfo {
            gc_type: "boehm".to_string(),
            gc_runs: 50,
            total_gc_time_ms: 5000,
            average_pause_time_ms: 2.5,
            memory_reclaimed: 1024 * 1024 * 10,
        };

        assert_eq!(gc.gc_type, "boehm");
        assert_eq!(gc.gc_runs, 50);
    }

    #[test]
    fn test_runtime_state_info_with_gc() {
        let state = RuntimeStateInfo {
            cpu_usage: CpuUsageInfo {
                current_usage_percent: 30.0,
                average_usage_percent: 25.0,
                peak_usage_percent: 60.0,
                intensive_operations_count: 5,
            },
            memory_pressure: MemoryPressureInfo {
                pressure_level: MemoryPressureLevel::Moderate,
                available_memory_percent: 50.0,
                allocation_failures: 1,
                fragmentation_level: 0.3,
            },
            cache_performance: CachePerformanceInfo {
                l1_hit_rate: 0.9,
                l2_hit_rate: 0.8,
                l3_hit_rate: 0.7,
                cache_miss_penalty_ns: 80.0,
                access_pattern: MemoryAccessPattern::Mixed,
            },
            allocator_state: AllocatorStateInfo {
                allocator_type: "system".to_string(),
                heap_size: 1024 * 1024,
                heap_used: 768 * 1024,
                free_blocks_count: 5,
                largest_free_block: 128 * 1024,
                efficiency_score: 0.75,
            },
            gc_info: Some(GcInfo {
                gc_type: "simple".to_string(),
                gc_runs: 10,
                total_gc_time_ms: 100,
                average_pause_time_ms: 1.0,
                memory_reclaimed: 1024 * 100,
            }),
        };

        assert!(state.gc_info.is_some());
        assert_eq!(state.gc_info.as_ref().unwrap().gc_runs, 10);
    }

    #[test]
    fn test_runtime_state_info_serialization() {
        let state = RuntimeStateInfo {
            cpu_usage: CpuUsageInfo {
                current_usage_percent: 50.0,
                average_usage_percent: 40.0,
                peak_usage_percent: 80.0,
                intensive_operations_count: 20,
            },
            memory_pressure: MemoryPressureInfo {
                pressure_level: MemoryPressureLevel::Low,
                available_memory_percent: 70.0,
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
                heap_size: 1024,
                heap_used: 512,
                free_blocks_count: 2,
                largest_free_block: 256,
                efficiency_score: 0.9,
            },
            gc_info: None,
        };

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: RuntimeStateInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.cpu_usage.current_usage_percent,
            state.cpu_usage.current_usage_percent
        );
    }

    #[test]
    fn test_memory_pressure_level_serialization() {
        let levels = vec![
            MemoryPressureLevel::Low,
            MemoryPressureLevel::Moderate,
            MemoryPressureLevel::High,
            MemoryPressureLevel::Critical,
        ];

        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let deserialized: MemoryPressureLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, level);
        }
    }

    #[test]
    fn test_memory_access_pattern_serialization() {
        let patterns = vec![
            MemoryAccessPattern::Sequential,
            MemoryAccessPattern::Random,
            MemoryAccessPattern::Strided { stride: 128 },
            MemoryAccessPattern::Clustered,
            MemoryAccessPattern::Mixed,
        ];

        for pattern in patterns {
            let json = serde_json::to_string(&pattern).unwrap();
            let deserialized: MemoryAccessPattern = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, pattern);
        }
    }

    #[test]
    fn test_cpu_usage_info_serialization() {
        let cpu = CpuUsageInfo {
            current_usage_percent: 42.5,
            average_usage_percent: 35.0,
            peak_usage_percent: 75.0,
            intensive_operations_count: 50,
        };

        let json = serde_json::to_string(&cpu).unwrap();
        let deserialized: CpuUsageInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.current_usage_percent,
            cpu.current_usage_percent
        );
    }

    #[test]
    fn test_allocator_state_info_serialization() {
        let allocator = AllocatorStateInfo {
            allocator_type: "mimalloc".to_string(),
            heap_size: 2048,
            heap_used: 1024,
            free_blocks_count: 10,
            largest_free_block: 512,
            efficiency_score: 0.85,
        };

        let json = serde_json::to_string(&allocator).unwrap();
        let deserialized: AllocatorStateInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.allocator_type, allocator.allocator_type);
    }

    #[test]
    fn test_gc_info_serialization() {
        let gc = GcInfo {
            gc_type: "generational".to_string(),
            gc_runs: 100,
            total_gc_time_ms: 1000,
            average_pause_time_ms: 5.0,
            memory_reclaimed: 1024 * 1024,
        };

        let json = serde_json::to_string(&gc).unwrap();
        let deserialized: GcInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.gc_runs, gc.gc_runs);
    }

    #[test]
    fn test_runtime_state_info_clone() {
        let state = RuntimeStateInfo {
            cpu_usage: CpuUsageInfo {
                current_usage_percent: 0.0,
                average_usage_percent: 0.0,
                peak_usage_percent: 0.0,
                intensive_operations_count: 0,
            },
            memory_pressure: MemoryPressureInfo {
                pressure_level: MemoryPressureLevel::Low,
                available_memory_percent: 100.0,
                allocation_failures: 0,
                fragmentation_level: 0.0,
            },
            cache_performance: CachePerformanceInfo {
                l1_hit_rate: 0.0,
                l2_hit_rate: 0.0,
                l3_hit_rate: 0.0,
                cache_miss_penalty_ns: 0.0,
                access_pattern: MemoryAccessPattern::Mixed,
            },
            allocator_state: AllocatorStateInfo {
                allocator_type: String::new(),
                heap_size: 0,
                heap_used: 0,
                free_blocks_count: 0,
                largest_free_block: 0,
                efficiency_score: 0.0,
            },
            gc_info: None,
        };

        let cloned = state.clone();
        assert_eq!(
            cloned.cpu_usage.current_usage_percent,
            state.cpu_usage.current_usage_percent
        );
    }

    #[test]
    fn test_runtime_state_info_debug() {
        let state = RuntimeStateInfo {
            cpu_usage: CpuUsageInfo {
                current_usage_percent: 0.0,
                average_usage_percent: 0.0,
                peak_usage_percent: 0.0,
                intensive_operations_count: 0,
            },
            memory_pressure: MemoryPressureInfo {
                pressure_level: MemoryPressureLevel::Low,
                available_memory_percent: 0.0,
                allocation_failures: 0,
                fragmentation_level: 0.0,
            },
            cache_performance: CachePerformanceInfo {
                l1_hit_rate: 0.0,
                l2_hit_rate: 0.0,
                l3_hit_rate: 0.0,
                cache_miss_penalty_ns: 0.0,
                access_pattern: MemoryAccessPattern::Mixed,
            },
            allocator_state: AllocatorStateInfo {
                allocator_type: String::new(),
                heap_size: 0,
                heap_used: 0,
                free_blocks_count: 0,
                largest_free_block: 0,
                efficiency_score: 0.0,
            },
            gc_info: None,
        };

        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("RuntimeStateInfo"));
        assert!(debug_str.contains("cpu_usage"));
    }

    #[test]
    fn test_boundary_values_cpu_usage() {
        let cpu = CpuUsageInfo {
            current_usage_percent: 100.0,
            average_usage_percent: 100.0,
            peak_usage_percent: 100.0,
            intensive_operations_count: usize::MAX,
        };

        assert!((cpu.current_usage_percent - 100.0).abs() < f64::EPSILON);
        assert_eq!(cpu.intensive_operations_count, usize::MAX);
    }

    #[test]
    fn test_boundary_values_allocator() {
        let allocator = AllocatorStateInfo {
            allocator_type: String::new(),
            heap_size: usize::MAX,
            heap_used: usize::MAX,
            free_blocks_count: usize::MAX,
            largest_free_block: usize::MAX,
            efficiency_score: 1.0,
        };

        assert_eq!(allocator.heap_size, usize::MAX);
        assert!((allocator.efficiency_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_boundary_values_gc_info() {
        let gc = GcInfo {
            gc_type: String::new(),
            gc_runs: usize::MAX,
            total_gc_time_ms: u64::MAX,
            average_pause_time_ms: f64::MAX,
            memory_reclaimed: usize::MAX,
        };

        assert_eq!(gc.gc_runs, usize::MAX);
        assert_eq!(gc.total_gc_time_ms, u64::MAX);
    }

    #[test]
    fn test_cache_performance_boundary() {
        let cache = CachePerformanceInfo {
            l1_hit_rate: 1.0,
            l2_hit_rate: 1.0,
            l3_hit_rate: 1.0,
            cache_miss_penalty_ns: f64::MAX,
            access_pattern: MemoryAccessPattern::Sequential,
        };

        assert!((cache.l1_hit_rate - 1.0).abs() < f64::EPSILON);
        assert!(cache.cache_miss_penalty_ns.is_finite());
    }
}
