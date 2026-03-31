//! Function call tracking types.
//!
//! This module contains types for tracking function calls,
//! call stacks, and performance characteristics.

use serde::{Deserialize, Serialize};

use super::allocation::{BottleneckType, ImpactLevel};
use super::generic::MemoryAccessPattern;
use super::generic::SourceLocation;

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
    pub bottlenecks: Vec<super::allocation::PerformanceBottleneck>,
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
}
