//! Function call tracking types.
//!
//! This module contains types for tracking function calls,
//! call stacks, and performance characteristics.

use serde::{Deserialize, Serialize};

use super::allocation::{BottleneckType, ImpactLevel, PerformanceBottleneck};
use super::generic::MemoryAccessPattern;

/// Function call tracking information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCallTrackingInfo {
    /// Function name.
    pub function_name: String,
    /// Module path.
    pub module_path: String,
    /// Total call count.
    pub total_call_count: u64,
    /// Call frequency per second.
    pub call_frequency_per_sec: f64,
    /// Average execution time per call.
    pub avg_execution_time_ns: f64,
    /// Total execution time.
    pub total_execution_time_ns: u64,
    /// Call stack information.
    pub call_stack_info: CallStackInfo,
    /// Memory allocations per call.
    pub memory_allocations_per_call: f64,
    /// Performance characteristics.
    pub performance_characteristics: FunctionPerformanceCharacteristics,
    /// Call patterns.
    pub call_patterns: Vec<CallPattern>,
}

/// Call stack information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallStackInfo {
    /// Maximum call stack depth.
    pub max_stack_depth: u32,
    /// Average call stack depth.
    pub avg_stack_depth: f64,
    /// Most common call sequences.
    pub common_call_sequences: Vec<CallSequence>,
    /// Recursive call detection.
    pub recursive_calls: Vec<RecursiveCallInfo>,
    /// Stack overflow risk assessment.
    pub stack_overflow_risk: StackOverflowRisk,
}

/// Call sequence information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallSequence {
    /// Sequence of function names.
    pub function_sequence: Vec<String>,
    /// Frequency of this sequence.
    pub frequency: u32,
    /// Average execution time for this sequence.
    pub avg_execution_time_ns: f64,
    /// Memory usage pattern for this sequence.
    pub memory_usage_pattern: MemoryUsagePattern,
}

/// Memory usage pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryUsagePattern {
    /// Peak memory usage in sequence.
    pub peak_memory_usage: usize,
    /// Average memory usage.
    pub avg_memory_usage: usize,
    /// Memory allocation frequency.
    pub allocation_frequency: f64,
    /// Memory deallocation frequency.
    pub deallocation_frequency: f64,
    /// Memory leak potential.
    pub leak_potential: LeakPotential,
}

/// Memory leak potential assessment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LeakPotential {
    /// Low memory leak potential.
    Low,
    /// Medium memory leak potential.
    Medium,
    /// High memory leak potential.
    High,
    /// Critical memory leak potential.
    Critical,
}

/// Recursive call information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecursiveCallInfo {
    /// Function name.
    pub function_name: String,
    /// Maximum recursion depth.
    pub max_recursion_depth: u32,
    /// Average recursion depth.
    pub avg_recursion_depth: f64,
    /// Tail recursion optimization potential.
    pub tail_recursion_potential: bool,
    /// Stack usage per recursion level.
    pub stack_usage_per_level: usize,
    /// Performance impact of recursion.
    pub recursion_performance_impact: RecursionPerformanceImpact,
}

/// Recursion performance impact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecursionPerformanceImpact {
    /// Stack overhead per call.
    pub stack_overhead_per_call: usize,
    /// Function call overhead.
    pub call_overhead_ns: f64,
    /// Cache impact of deep recursion.
    pub cache_impact: f64,
    /// Optimization recommendations.
    pub optimization_recommendations: Vec<String>,
}

/// Stack overflow risk assessment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StackOverflowRisk {
    /// Low stack overflow risk.
    Low,
    /// Medium stack overflow risk.
    Medium,
    /// High stack overflow risk.
    High,
    /// Critical stack overflow risk.
    Critical,
}

/// Function performance characteristics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionPerformanceCharacteristics {
    /// CPU usage percentage.
    pub cpu_usage_percent: f64,
    /// Memory usage characteristics.
    pub memory_characteristics: FunctionMemoryCharacteristics,
    /// I/O characteristics.
    pub io_characteristics: IOCharacteristics,
    /// Concurrency characteristics.
    pub concurrency_characteristics: ConcurrencyCharacteristics,
    /// Performance bottlenecks.
    pub bottlenecks: Vec<PerformanceBottleneck>,
}

/// Function memory characteristics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionMemoryCharacteristics {
    /// Stack memory usage.
    pub stack_memory_usage: usize,
    /// Heap memory allocations.
    pub heap_allocations: u32,
    /// Memory access pattern.
    pub access_pattern: MemoryAccessPattern,
    /// Cache efficiency.
    pub cache_efficiency: f64,
    /// Memory bandwidth utilization.
    pub memory_bandwidth_utilization: f64,
}

/// I/O characteristics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IOCharacteristics {
    /// File I/O operations.
    pub file_io_operations: u32,
    /// Network I/O operations.
    pub network_io_operations: u32,
    /// Average I/O wait time.
    pub avg_io_wait_time_ns: f64,
    /// I/O throughput.
    pub io_throughput_bytes_per_sec: f64,
    /// I/O efficiency score.
    pub io_efficiency_score: f64,
}

/// Concurrency characteristics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConcurrencyCharacteristics {
    /// Thread safety level.
    pub thread_safety_level: ThreadSafetyLevel,
    /// Lock contention frequency.
    pub lock_contention_frequency: f64,
    /// Parallel execution potential.
    pub parallel_execution_potential: f64,
    /// Synchronization overhead.
    pub synchronization_overhead_ns: f64,
    /// Deadlock risk assessment.
    pub deadlock_risk: DeadlockRisk,
}

/// Thread safety levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThreadSafetyLevel {
    /// Thread safe.
    ThreadSafe,
    /// Conditionally thread safe.
    ConditionallyThreadSafe,
    /// Not thread safe.
    NotThreadSafe,
    /// Unknown thread safety.
    Unknown,
}

/// Deadlock risk assessment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeadlockRisk {
    /// No deadlock risk.
    None,
    /// Low deadlock risk.
    Low,
    /// Medium deadlock risk.
    Medium,
    /// High deadlock risk.
    High,
}

/// Call pattern information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallPattern {
    /// Pattern type.
    pub pattern_type: CallPatternType,
    /// Pattern description.
    pub description: String,
    /// Frequency of this pattern.
    pub frequency: u32,
    /// Performance impact.
    pub performance_impact: f64,
    /// Optimization potential.
    pub optimization_potential: f64,
}

/// Types of call patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CallPatternType {
    /// Sequential call pattern.
    Sequential,
    /// Recursive call pattern.
    Recursive,
    /// Iterative call pattern.
    Iterative,
    /// Conditional call pattern.
    Conditional,
    /// Parallel call pattern.
    Parallel,
    /// Asynchronous call pattern.
    Asynchronous,
    /// Callback call pattern.
    Callback,
    /// Event-driven call pattern.
    EventDriven,
}

impl From<crate::core::types::FunctionCallTrackingInfo> for FunctionCallTrackingInfo {
    fn from(old: crate::core::types::FunctionCallTrackingInfo) -> Self {
        Self {
            function_name: old.function_name,
            module_path: old.module_path,
            total_call_count: old.total_call_count,
            call_frequency_per_sec: old.call_frequency_per_sec,
            avg_execution_time_ns: old.avg_execution_time_ns,
            total_execution_time_ns: old.total_execution_time_ns,
            call_stack_info: CallStackInfo {
                max_stack_depth: old.call_stack_info.max_stack_depth,
                avg_stack_depth: old.call_stack_info.avg_stack_depth,
                common_call_sequences: old
                    .call_stack_info
                    .common_call_sequences
                    .into_iter()
                    .map(|s| CallSequence {
                        function_sequence: s.function_sequence,
                        frequency: s.frequency,
                        avg_execution_time_ns: s.avg_execution_time_ns,
                        memory_usage_pattern: MemoryUsagePattern {
                            peak_memory_usage: s.memory_usage_pattern.peak_memory_usage,
                            avg_memory_usage: s.memory_usage_pattern.avg_memory_usage,
                            allocation_frequency: s.memory_usage_pattern.allocation_frequency,
                            deallocation_frequency: s.memory_usage_pattern.deallocation_frequency,
                            leak_potential: match s.memory_usage_pattern.leak_potential {
                                crate::core::types::LeakPotential::Low => LeakPotential::Low,
                                crate::core::types::LeakPotential::Medium => LeakPotential::Medium,
                                crate::core::types::LeakPotential::High => LeakPotential::High,
                                crate::core::types::LeakPotential::Critical => {
                                    LeakPotential::Critical
                                }
                            },
                        },
                    })
                    .collect(),
                recursive_calls: old
                    .call_stack_info
                    .recursive_calls
                    .into_iter()
                    .map(|r| RecursiveCallInfo {
                        function_name: r.function_name,
                        max_recursion_depth: r.max_recursion_depth,
                        avg_recursion_depth: r.avg_recursion_depth,
                        tail_recursion_potential: r.tail_recursion_potential,
                        stack_usage_per_level: r.stack_usage_per_level,
                        recursion_performance_impact: RecursionPerformanceImpact {
                            stack_overhead_per_call: r
                                .recursion_performance_impact
                                .stack_overhead_per_call,
                            call_overhead_ns: r.recursion_performance_impact.call_overhead_ns,
                            cache_impact: r.recursion_performance_impact.cache_impact,
                            optimization_recommendations: r
                                .recursion_performance_impact
                                .optimization_recommendations,
                        },
                    })
                    .collect(),
                stack_overflow_risk: match old.call_stack_info.stack_overflow_risk {
                    crate::core::types::StackOverflowRisk::Low => StackOverflowRisk::Low,
                    crate::core::types::StackOverflowRisk::Medium => StackOverflowRisk::Medium,
                    crate::core::types::StackOverflowRisk::High => StackOverflowRisk::High,
                    crate::core::types::StackOverflowRisk::Critical => StackOverflowRisk::Critical,
                },
            },
            memory_allocations_per_call: old.memory_allocations_per_call,
            performance_characteristics: FunctionPerformanceCharacteristics {
                cpu_usage_percent: old.performance_characteristics.cpu_usage_percent,
                memory_characteristics: FunctionMemoryCharacteristics {
                    stack_memory_usage: old
                        .performance_characteristics
                        .memory_characteristics
                        .stack_memory_usage,
                    heap_allocations: old
                        .performance_characteristics
                        .memory_characteristics
                        .heap_allocations,
                    access_pattern: match old
                        .performance_characteristics
                        .memory_characteristics
                        .access_pattern
                    {
                        crate::core::types::MemoryAccessPattern::Sequential => {
                            MemoryAccessPattern::Sequential
                        }
                        crate::core::types::MemoryAccessPattern::Random => {
                            MemoryAccessPattern::Random
                        }
                        crate::core::types::MemoryAccessPattern::Strided { stride } => {
                            MemoryAccessPattern::Strided { stride }
                        }
                        crate::core::types::MemoryAccessPattern::Clustered => {
                            MemoryAccessPattern::Clustered
                        }
                        crate::core::types::MemoryAccessPattern::Mixed => {
                            MemoryAccessPattern::Mixed
                        }
                    },
                    cache_efficiency: old
                        .performance_characteristics
                        .memory_characteristics
                        .cache_efficiency,
                    memory_bandwidth_utilization: old
                        .performance_characteristics
                        .memory_characteristics
                        .memory_bandwidth_utilization,
                },
                io_characteristics: IOCharacteristics {
                    file_io_operations: old
                        .performance_characteristics
                        .io_characteristics
                        .file_io_operations,
                    network_io_operations: old
                        .performance_characteristics
                        .io_characteristics
                        .network_io_operations,
                    avg_io_wait_time_ns: old
                        .performance_characteristics
                        .io_characteristics
                        .avg_io_wait_time_ns,
                    io_throughput_bytes_per_sec: old
                        .performance_characteristics
                        .io_characteristics
                        .io_throughput_bytes_per_sec,
                    io_efficiency_score: old
                        .performance_characteristics
                        .io_characteristics
                        .io_efficiency_score,
                },
                concurrency_characteristics: ConcurrencyCharacteristics {
                    thread_safety_level: match old
                        .performance_characteristics
                        .concurrency_characteristics
                        .thread_safety_level
                    {
                        crate::core::types::ThreadSafetyLevel::ThreadSafe => {
                            ThreadSafetyLevel::ThreadSafe
                        }
                        crate::core::types::ThreadSafetyLevel::ConditionallyThreadSafe => {
                            ThreadSafetyLevel::ConditionallyThreadSafe
                        }
                        crate::core::types::ThreadSafetyLevel::NotThreadSafe => {
                            ThreadSafetyLevel::NotThreadSafe
                        }
                        crate::core::types::ThreadSafetyLevel::Unknown => {
                            ThreadSafetyLevel::Unknown
                        }
                    },
                    lock_contention_frequency: old
                        .performance_characteristics
                        .concurrency_characteristics
                        .lock_contention_frequency,
                    parallel_execution_potential: old
                        .performance_characteristics
                        .concurrency_characteristics
                        .parallel_execution_potential,
                    synchronization_overhead_ns: old
                        .performance_characteristics
                        .concurrency_characteristics
                        .synchronization_overhead_ns,
                    deadlock_risk: match old
                        .performance_characteristics
                        .concurrency_characteristics
                        .deadlock_risk
                    {
                        crate::core::types::DeadlockRisk::None => DeadlockRisk::None,
                        crate::core::types::DeadlockRisk::Low => DeadlockRisk::Low,
                        crate::core::types::DeadlockRisk::Medium => DeadlockRisk::Medium,
                        crate::core::types::DeadlockRisk::High => DeadlockRisk::High,
                    },
                },
                bottlenecks: old
                    .performance_characteristics
                    .bottlenecks
                    .into_iter()
                    .map(|b| PerformanceBottleneck {
                        bottleneck_type: match b.bottleneck_type {
                            crate::core::types::BottleneckType::MemoryAllocation => {
                                BottleneckType::MemoryAllocation
                            }
                            crate::core::types::BottleneckType::MemoryDeallocation => {
                                BottleneckType::MemoryDeallocation
                            }
                            crate::core::types::BottleneckType::CacheMiss => {
                                BottleneckType::CacheMiss
                            }
                            crate::core::types::BottleneckType::BranchMisprediction => {
                                BottleneckType::BranchMisprediction
                            }
                            crate::core::types::BottleneckType::FunctionCall => {
                                BottleneckType::FunctionCall
                            }
                            crate::core::types::BottleneckType::DataMovement => {
                                BottleneckType::DataMovement
                            }
                            crate::core::types::BottleneckType::Synchronization => {
                                BottleneckType::Synchronization
                            }
                            crate::core::types::BottleneckType::IO => BottleneckType::IO,
                        },
                        location: b.location,
                        severity: match b.severity {
                            crate::core::types::ImpactLevel::Low => ImpactLevel::Low,
                            crate::core::types::ImpactLevel::Medium => ImpactLevel::Medium,
                            crate::core::types::ImpactLevel::High => ImpactLevel::High,
                            crate::core::types::ImpactLevel::Critical => ImpactLevel::Critical,
                        },
                        description: b.description,
                        optimization_suggestion: b.optimization_suggestion,
                    })
                    .collect(),
            },
            call_patterns: old
                .call_patterns
                .into_iter()
                .map(|p| CallPattern {
                    pattern_type: match p.pattern_type {
                        crate::core::types::CallPatternType::Sequential => {
                            CallPatternType::Sequential
                        }
                        crate::core::types::CallPatternType::Recursive => {
                            CallPatternType::Recursive
                        }
                        crate::core::types::CallPatternType::Iterative => {
                            CallPatternType::Iterative
                        }
                        crate::core::types::CallPatternType::Conditional => {
                            CallPatternType::Conditional
                        }
                        crate::core::types::CallPatternType::Parallel => CallPatternType::Parallel,
                        crate::core::types::CallPatternType::Asynchronous => {
                            CallPatternType::Asynchronous
                        }
                        crate::core::types::CallPatternType::Callback => CallPatternType::Callback,
                        crate::core::types::CallPatternType::EventDriven => {
                            CallPatternType::EventDriven
                        }
                    },
                    description: p.description,
                    frequency: p.frequency,
                    performance_impact: p.performance_impact,
                    optimization_potential: p.optimization_potential,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_call_tracking_info() {
        let info = FunctionCallTrackingInfo {
            function_name: "test_func".to_string(),
            module_path: "test::module".to_string(),
            total_call_count: 100,
            call_frequency_per_sec: 10.0,
            avg_execution_time_ns: 1000.0,
            total_execution_time_ns: 100000,
            call_stack_info: CallStackInfo {
                max_stack_depth: 5,
                avg_stack_depth: 3.0,
                common_call_sequences: vec![],
                recursive_calls: vec![],
                stack_overflow_risk: StackOverflowRisk::Low,
            },
            memory_allocations_per_call: 2.0,
            performance_characteristics: FunctionPerformanceCharacteristics {
                cpu_usage_percent: 5.0,
                memory_characteristics: FunctionMemoryCharacteristics {
                    stack_memory_usage: 1024,
                    heap_allocations: 5,
                    access_pattern: MemoryAccessPattern::Sequential,
                    cache_efficiency: 0.9,
                    memory_bandwidth_utilization: 0.5,
                },
                io_characteristics: IOCharacteristics {
                    file_io_operations: 0,
                    network_io_operations: 0,
                    avg_io_wait_time_ns: 0.0,
                    io_throughput_bytes_per_sec: 0.0,
                    io_efficiency_score: 1.0,
                },
                concurrency_characteristics: ConcurrencyCharacteristics {
                    thread_safety_level: ThreadSafetyLevel::ThreadSafe,
                    lock_contention_frequency: 0.0,
                    parallel_execution_potential: 0.8,
                    synchronization_overhead_ns: 0.0,
                    deadlock_risk: DeadlockRisk::None,
                },
                bottlenecks: vec![],
            },
            call_patterns: vec![],
        };

        assert_eq!(info.function_name, "test_func");
        assert_eq!(info.total_call_count, 100);
    }

    #[test]
    fn test_call_pattern_type() {
        let patterns = vec![
            CallPatternType::Sequential,
            CallPatternType::Recursive,
            CallPatternType::Parallel,
        ];

        for pattern in patterns {
            assert!(!format!("{pattern:?}").is_empty());
        }
    }

    #[test]
    fn test_all_leak_potential_variants() {
        let potentials = [
            LeakPotential::Low,
            LeakPotential::Medium,
            LeakPotential::High,
            LeakPotential::Critical,
        ];

        for potential in potentials {
            let pattern = MemoryUsagePattern {
                peak_memory_usage: 1024,
                avg_memory_usage: 512,
                allocation_frequency: 10.0,
                deallocation_frequency: 8.0,
                leak_potential: potential.clone(),
            };
            assert_eq!(pattern.leak_potential, potential);
        }
    }

    #[test]
    fn test_all_stack_overflow_risk_variants() {
        let risks = [
            StackOverflowRisk::Low,
            StackOverflowRisk::Medium,
            StackOverflowRisk::High,
            StackOverflowRisk::Critical,
        ];

        for risk in risks {
            let info = CallStackInfo {
                max_stack_depth: 10,
                avg_stack_depth: 5.0,
                common_call_sequences: vec![],
                recursive_calls: vec![],
                stack_overflow_risk: risk.clone(),
            };
            assert_eq!(info.stack_overflow_risk, risk);
        }
    }

    #[test]
    fn test_all_thread_safety_level_variants() {
        let levels = [
            ThreadSafetyLevel::ThreadSafe,
            ThreadSafetyLevel::ConditionallyThreadSafe,
            ThreadSafetyLevel::NotThreadSafe,
            ThreadSafetyLevel::Unknown,
        ];

        for level in levels.clone() {
            let chars = ConcurrencyCharacteristics {
                thread_safety_level: level.clone(),
                lock_contention_frequency: 0.5,
                parallel_execution_potential: 0.8,
                synchronization_overhead_ns: 100.0,
                deadlock_risk: DeadlockRisk::None,
            };
            assert_eq!(chars.thread_safety_level, level);
        }
    }

    #[test]
    fn test_all_deadlock_risk_variants() {
        let risks = [
            DeadlockRisk::None,
            DeadlockRisk::Low,
            DeadlockRisk::Medium,
            DeadlockRisk::High,
        ];

        for risk in risks {
            let chars = ConcurrencyCharacteristics {
                thread_safety_level: ThreadSafetyLevel::ThreadSafe,
                lock_contention_frequency: 0.5,
                parallel_execution_potential: 0.8,
                synchronization_overhead_ns: 100.0,
                deadlock_risk: risk.clone(),
            };
            assert_eq!(chars.deadlock_risk, risk);
        }
    }

    #[test]
    fn test_all_call_pattern_type_variants() {
        let patterns = [
            CallPatternType::Sequential,
            CallPatternType::Recursive,
            CallPatternType::Iterative,
            CallPatternType::Conditional,
            CallPatternType::Parallel,
            CallPatternType::Asynchronous,
            CallPatternType::Callback,
            CallPatternType::EventDriven,
        ];

        for pattern in patterns {
            let cp = CallPattern {
                pattern_type: pattern.clone(),
                description: "Test pattern".to_string(),
                frequency: 10,
                performance_impact: 0.5,
                optimization_potential: 0.3,
            };
            assert_eq!(cp.pattern_type, pattern);
        }
    }

    #[test]
    fn test_call_stack_info_creation() {
        let info = CallStackInfo {
            max_stack_depth: 100,
            avg_stack_depth: 45.5,
            common_call_sequences: vec![CallSequence {
                function_sequence: vec!["main".to_string(), "foo".to_string()],
                frequency: 50,
                avg_execution_time_ns: 5000.0,
                memory_usage_pattern: MemoryUsagePattern {
                    peak_memory_usage: 2048,
                    avg_memory_usage: 1024,
                    allocation_frequency: 5.0,
                    deallocation_frequency: 5.0,
                    leak_potential: LeakPotential::Low,
                },
            }],
            recursive_calls: vec![],
            stack_overflow_risk: StackOverflowRisk::Medium,
        };

        assert_eq!(info.max_stack_depth, 100);
        assert_eq!(info.common_call_sequences.len(), 1);
    }

    #[test]
    fn test_recursive_call_info_creation() {
        let info = RecursiveCallInfo {
            function_name: "fibonacci".to_string(),
            max_recursion_depth: 50,
            avg_recursion_depth: 25.5,
            tail_recursion_potential: true,
            stack_usage_per_level: 64,
            recursion_performance_impact: RecursionPerformanceImpact {
                stack_overhead_per_call: 64,
                call_overhead_ns: 10.0,
                cache_impact: 0.2,
                optimization_recommendations: vec!["Convert to iterative".to_string()],
            },
        };

        assert_eq!(info.function_name, "fibonacci");
        assert!(info.tail_recursion_potential);
    }

    #[test]
    fn test_memory_usage_pattern_creation() {
        let pattern = MemoryUsagePattern {
            peak_memory_usage: 10240,
            avg_memory_usage: 5120,
            allocation_frequency: 100.0,
            deallocation_frequency: 95.0,
            leak_potential: LeakPotential::High,
        };

        assert_eq!(pattern.peak_memory_usage, 10240);
        assert!(matches!(pattern.leak_potential, LeakPotential::High));
    }

    #[test]
    fn test_function_memory_characteristics_creation() {
        let chars = FunctionMemoryCharacteristics {
            stack_memory_usage: 2048,
            heap_allocations: 10,
            access_pattern: MemoryAccessPattern::Random,
            cache_efficiency: 0.75,
            memory_bandwidth_utilization: 0.6,
        };

        assert_eq!(chars.stack_memory_usage, 2048);
        assert!(matches!(chars.access_pattern, MemoryAccessPattern::Random));
    }

    #[test]
    fn test_io_characteristics_creation() {
        let chars = IOCharacteristics {
            file_io_operations: 100,
            network_io_operations: 50,
            avg_io_wait_time_ns: 5000.0,
            io_throughput_bytes_per_sec: 1024000.0,
            io_efficiency_score: 0.85,
        };

        assert_eq!(chars.file_io_operations, 100);
        assert_eq!(chars.network_io_operations, 50);
    }

    #[test]
    fn test_call_sequence_creation() {
        let seq = CallSequence {
            function_sequence: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            frequency: 25,
            avg_execution_time_ns: 15000.0,
            memory_usage_pattern: MemoryUsagePattern {
                peak_memory_usage: 4096,
                avg_memory_usage: 2048,
                allocation_frequency: 2.0,
                deallocation_frequency: 2.0,
                leak_potential: LeakPotential::Low,
            },
        };

        assert_eq!(seq.function_sequence.len(), 3);
        assert_eq!(seq.frequency, 25);
    }

    #[test]
    fn test_call_pattern_creation() {
        let pattern = CallPattern {
            pattern_type: CallPatternType::Recursive,
            description: "Recursive tree traversal".to_string(),
            frequency: 100,
            performance_impact: 0.4,
            optimization_potential: 0.7,
        };

        assert!(matches!(pattern.pattern_type, CallPatternType::Recursive));
        assert_eq!(pattern.optimization_potential, 0.7);
    }

    #[test]
    fn test_function_call_tracking_info_serialization() {
        let info = FunctionCallTrackingInfo {
            function_name: "serialize_test".to_string(),
            module_path: "test::serialize".to_string(),
            total_call_count: 50,
            call_frequency_per_sec: 5.0,
            avg_execution_time_ns: 2000.0,
            total_execution_time_ns: 100000,
            call_stack_info: CallStackInfo {
                max_stack_depth: 3,
                avg_stack_depth: 2.0,
                common_call_sequences: vec![],
                recursive_calls: vec![],
                stack_overflow_risk: StackOverflowRisk::Low,
            },
            memory_allocations_per_call: 1.5,
            performance_characteristics: FunctionPerformanceCharacteristics {
                cpu_usage_percent: 10.0,
                memory_characteristics: FunctionMemoryCharacteristics {
                    stack_memory_usage: 512,
                    heap_allocations: 2,
                    access_pattern: MemoryAccessPattern::Sequential,
                    cache_efficiency: 0.9,
                    memory_bandwidth_utilization: 0.4,
                },
                io_characteristics: IOCharacteristics {
                    file_io_operations: 0,
                    network_io_operations: 0,
                    avg_io_wait_time_ns: 0.0,
                    io_throughput_bytes_per_sec: 0.0,
                    io_efficiency_score: 1.0,
                },
                concurrency_characteristics: ConcurrencyCharacteristics {
                    thread_safety_level: ThreadSafetyLevel::ThreadSafe,
                    lock_contention_frequency: 0.0,
                    parallel_execution_potential: 0.9,
                    synchronization_overhead_ns: 0.0,
                    deadlock_risk: DeadlockRisk::None,
                },
                bottlenecks: vec![],
            },
            call_patterns: vec![],
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: FunctionCallTrackingInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.function_name, info.function_name);
        assert_eq!(deserialized.total_call_count, info.total_call_count);
    }

    #[test]
    fn test_leak_potential_serialization() {
        let potentials = vec![
            LeakPotential::Low,
            LeakPotential::Medium,
            LeakPotential::High,
            LeakPotential::Critical,
        ];

        for potential in potentials {
            let json = serde_json::to_string(&potential).unwrap();
            let deserialized: LeakPotential = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, potential);
        }
    }

    #[test]
    fn test_stack_overflow_risk_serialization() {
        let risks = vec![
            StackOverflowRisk::Low,
            StackOverflowRisk::Medium,
            StackOverflowRisk::High,
            StackOverflowRisk::Critical,
        ];

        for risk in risks {
            let json = serde_json::to_string(&risk).unwrap();
            let deserialized: StackOverflowRisk = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, risk);
        }
    }

    #[test]
    fn test_thread_safety_level_serialization() {
        let levels = vec![
            ThreadSafetyLevel::ThreadSafe,
            ThreadSafetyLevel::ConditionallyThreadSafe,
            ThreadSafetyLevel::NotThreadSafe,
            ThreadSafetyLevel::Unknown,
        ];

        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let deserialized: ThreadSafetyLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, level);
        }
    }

    #[test]
    fn test_deadlock_risk_serialization() {
        let risks = vec![
            DeadlockRisk::None,
            DeadlockRisk::Low,
            DeadlockRisk::Medium,
            DeadlockRisk::High,
        ];

        for risk in risks {
            let json = serde_json::to_string(&risk).unwrap();
            let deserialized: DeadlockRisk = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, risk);
        }
    }

    #[test]
    fn test_call_pattern_type_serialization() {
        let patterns = vec![
            CallPatternType::Sequential,
            CallPatternType::Recursive,
            CallPatternType::Iterative,
            CallPatternType::Conditional,
            CallPatternType::Parallel,
            CallPatternType::Asynchronous,
            CallPatternType::Callback,
            CallPatternType::EventDriven,
        ];

        for pattern in patterns {
            let json = serde_json::to_string(&pattern).unwrap();
            let deserialized: CallPatternType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, pattern);
        }
    }

    #[test]
    fn test_function_call_tracking_info_clone() {
        let info = FunctionCallTrackingInfo {
            function_name: "clone_test".to_string(),
            module_path: "test::clone".to_string(),
            total_call_count: 10,
            call_frequency_per_sec: 1.0,
            avg_execution_time_ns: 100.0,
            total_execution_time_ns: 1000,
            call_stack_info: CallStackInfo {
                max_stack_depth: 1,
                avg_stack_depth: 1.0,
                common_call_sequences: vec![],
                recursive_calls: vec![],
                stack_overflow_risk: StackOverflowRisk::Low,
            },
            memory_allocations_per_call: 0.0,
            performance_characteristics: FunctionPerformanceCharacteristics {
                cpu_usage_percent: 0.0,
                memory_characteristics: FunctionMemoryCharacteristics {
                    stack_memory_usage: 0,
                    heap_allocations: 0,
                    access_pattern: MemoryAccessPattern::Sequential,
                    cache_efficiency: 0.0,
                    memory_bandwidth_utilization: 0.0,
                },
                io_characteristics: IOCharacteristics {
                    file_io_operations: 0,
                    network_io_operations: 0,
                    avg_io_wait_time_ns: 0.0,
                    io_throughput_bytes_per_sec: 0.0,
                    io_efficiency_score: 0.0,
                },
                concurrency_characteristics: ConcurrencyCharacteristics {
                    thread_safety_level: ThreadSafetyLevel::Unknown,
                    lock_contention_frequency: 0.0,
                    parallel_execution_potential: 0.0,
                    synchronization_overhead_ns: 0.0,
                    deadlock_risk: DeadlockRisk::None,
                },
                bottlenecks: vec![],
            },
            call_patterns: vec![],
        };

        let cloned = info.clone();
        assert_eq!(cloned.function_name, info.function_name);
        assert_eq!(cloned.total_call_count, info.total_call_count);
    }

    #[test]
    fn test_function_call_tracking_info_debug() {
        let info = FunctionCallTrackingInfo {
            function_name: "debug_test".to_string(),
            module_path: "test::debug".to_string(),
            total_call_count: 1,
            call_frequency_per_sec: 1.0,
            avg_execution_time_ns: 1.0,
            total_execution_time_ns: 1,
            call_stack_info: CallStackInfo {
                max_stack_depth: 1,
                avg_stack_depth: 1.0,
                common_call_sequences: vec![],
                recursive_calls: vec![],
                stack_overflow_risk: StackOverflowRisk::Low,
            },
            memory_allocations_per_call: 0.0,
            performance_characteristics: FunctionPerformanceCharacteristics {
                cpu_usage_percent: 0.0,
                memory_characteristics: FunctionMemoryCharacteristics {
                    stack_memory_usage: 0,
                    heap_allocations: 0,
                    access_pattern: MemoryAccessPattern::Sequential,
                    cache_efficiency: 0.0,
                    memory_bandwidth_utilization: 0.0,
                },
                io_characteristics: IOCharacteristics {
                    file_io_operations: 0,
                    network_io_operations: 0,
                    avg_io_wait_time_ns: 0.0,
                    io_throughput_bytes_per_sec: 0.0,
                    io_efficiency_score: 0.0,
                },
                concurrency_characteristics: ConcurrencyCharacteristics {
                    thread_safety_level: ThreadSafetyLevel::Unknown,
                    lock_contention_frequency: 0.0,
                    parallel_execution_potential: 0.0,
                    synchronization_overhead_ns: 0.0,
                    deadlock_risk: DeadlockRisk::None,
                },
                bottlenecks: vec![],
            },
            call_patterns: vec![],
        };

        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("FunctionCallTrackingInfo"));
        assert!(debug_str.contains("function_name"));
    }

    #[test]
    fn test_recursion_performance_impact_creation() {
        let impact = RecursionPerformanceImpact {
            stack_overhead_per_call: 128,
            call_overhead_ns: 15.0,
            cache_impact: 0.35,
            optimization_recommendations: vec![
                "Use tail recursion".to_string(),
                "Convert to iteration".to_string(),
            ],
        };

        assert_eq!(impact.stack_overhead_per_call, 128);
        assert_eq!(impact.optimization_recommendations.len(), 2);
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
            let chars = FunctionMemoryCharacteristics {
                stack_memory_usage: 1024,
                heap_allocations: 5,
                access_pattern: pattern.clone(),
                cache_efficiency: 0.8,
                memory_bandwidth_utilization: 0.5,
            };
            assert_eq!(chars.access_pattern, pattern);
        }
    }

    #[test]
    fn test_performance_bottleneck_in_function() {
        let bottleneck = PerformanceBottleneck {
            bottleneck_type: BottleneckType::MemoryAllocation,
            location: "test_function:42".to_string(),
            severity: ImpactLevel::High,
            description: "Excessive memory allocations".to_string(),
            optimization_suggestion: "Use object pool".to_string(),
        };

        let chars = FunctionPerformanceCharacteristics {
            cpu_usage_percent: 50.0,
            memory_characteristics: FunctionMemoryCharacteristics {
                stack_memory_usage: 2048,
                heap_allocations: 100,
                access_pattern: MemoryAccessPattern::Random,
                cache_efficiency: 0.5,
                memory_bandwidth_utilization: 0.8,
            },
            io_characteristics: IOCharacteristics {
                file_io_operations: 0,
                network_io_operations: 0,
                avg_io_wait_time_ns: 0.0,
                io_throughput_bytes_per_sec: 0.0,
                io_efficiency_score: 1.0,
            },
            concurrency_characteristics: ConcurrencyCharacteristics {
                thread_safety_level: ThreadSafetyLevel::ThreadSafe,
                lock_contention_frequency: 0.0,
                parallel_execution_potential: 0.5,
                synchronization_overhead_ns: 0.0,
                deadlock_risk: DeadlockRisk::None,
            },
            bottlenecks: vec![bottleneck],
        };

        assert_eq!(chars.bottlenecks.len(), 1);
        assert!(matches!(
            chars.bottlenecks[0].bottleneck_type,
            BottleneckType::MemoryAllocation
        ));
    }

    #[test]
    fn test_boundary_values_call_count() {
        let info = FunctionCallTrackingInfo {
            function_name: "boundary_test".to_string(),
            module_path: "test::boundary".to_string(),
            total_call_count: u64::MAX,
            call_frequency_per_sec: f64::MAX,
            avg_execution_time_ns: f64::MIN,
            total_execution_time_ns: u64::MAX,
            call_stack_info: CallStackInfo {
                max_stack_depth: u32::MAX,
                avg_stack_depth: f64::MAX,
                common_call_sequences: vec![],
                recursive_calls: vec![],
                stack_overflow_risk: StackOverflowRisk::Critical,
            },
            memory_allocations_per_call: f64::INFINITY,
            performance_characteristics: FunctionPerformanceCharacteristics {
                cpu_usage_percent: 100.0,
                memory_characteristics: FunctionMemoryCharacteristics {
                    stack_memory_usage: usize::MAX,
                    heap_allocations: u32::MAX,
                    access_pattern: MemoryAccessPattern::Mixed,
                    cache_efficiency: 1.0,
                    memory_bandwidth_utilization: 1.0,
                },
                io_characteristics: IOCharacteristics {
                    file_io_operations: u32::MAX,
                    network_io_operations: u32::MAX,
                    avg_io_wait_time_ns: f64::MAX,
                    io_throughput_bytes_per_sec: f64::MAX,
                    io_efficiency_score: 0.0,
                },
                concurrency_characteristics: ConcurrencyCharacteristics {
                    thread_safety_level: ThreadSafetyLevel::NotThreadSafe,
                    lock_contention_frequency: f64::MAX,
                    parallel_execution_potential: 0.0,
                    synchronization_overhead_ns: f64::MAX,
                    deadlock_risk: DeadlockRisk::High,
                },
                bottlenecks: vec![],
            },
            call_patterns: vec![],
        };

        assert_eq!(info.total_call_count, u64::MAX);
        assert_eq!(info.call_stack_info.max_stack_depth, u32::MAX);
    }
}
