//! Lockfree tracker types and data structures.
//!
//! This module contains type definitions for the lockfree memory tracking system.

use std::thread::ThreadId;

/// Memory event for lockfree tracking
#[derive(Debug, Clone)]
pub struct Event {
    /// Event timestamp
    pub timestamp: u64,
    /// Event type (allocation/deallocation)
    pub event_type: EventType,
    /// Memory pointer
    pub ptr: usize,
    /// Memory size
    pub size: usize,
    /// Call stack hash
    pub call_stack_hash: u64,
    /// Thread ID
    pub thread_id: ThreadId,
    /// Optional metadata
    pub metadata: Option<EventMetadata>,
}

impl Event {
    /// Create a new allocation event
    pub fn allocation(ptr: usize, size: usize, call_stack_hash: u64, thread_id: ThreadId) -> Self {
        Self {
            timestamp: Self::now(),
            event_type: EventType::Allocation,
            ptr,
            size,
            call_stack_hash,
            thread_id,
            metadata: None,
        }
    }

    /// Create a new deallocation event
    pub fn deallocation(
        ptr: usize,
        size: usize,
        call_stack_hash: u64,
        thread_id: ThreadId,
    ) -> Self {
        Self {
            timestamp: Self::now(),
            event_type: EventType::Deallocation,
            ptr,
            size,
            call_stack_hash,
            thread_id,
            metadata: None,
        }
    }

    /// Get current timestamp
    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}

/// Event type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// Memory allocation event
    Allocation,
    /// Memory deallocation event
    Deallocation,
}

/// Event metadata
#[derive(Debug, Clone)]
pub struct EventMetadata {
    /// Optional variable name
    pub var_name: Option<String>,
    /// Optional type name
    pub type_name: Option<String>,
}

/// Frequency data for allocation patterns
#[derive(Debug, Clone)]
pub struct FrequencyData {
    /// Call stack hash
    pub call_stack_hash: u64,
    /// Number of allocations
    pub count: usize,
    /// Total size allocated
    pub total_size: usize,
    /// First allocation timestamp
    pub first_timestamp: u64,
    /// Last allocation timestamp
    pub last_timestamp: u64,
}

/// Allocation category classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationCategory {
    /// Small allocations (< 1KB)
    Small,
    /// Medium allocations (1KB - 1MB)
    Medium,
    /// Large allocations (> 1MB)
    Large,
}

/// Memory statistics for lockfree tracking
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Total number of allocations
    pub total_allocations: usize,
    /// Total allocated memory
    pub total_allocated: usize,
    /// Total number of deallocations
    pub total_deallocations: usize,
    /// Total deallocated memory
    pub total_deallocated: usize,
    /// Peak memory usage
    pub peak_memory: usize,
    /// Current active memory
    pub active_memory: usize,
}

/// Real call stack information
#[derive(Debug, Clone)]
pub struct RealCallStack {
    /// Stack frames
    pub frames: Vec<StackFrame>,
    /// Stack hash
    pub hash: u64,
}

/// Stack frame information
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Instruction pointer
    pub ip: usize,
    /// Function name
    pub function_name: Option<String>,
    /// File name
    pub file_name: Option<String>,
    /// Line number
    pub line_number: Option<u32>,
}

/// System metrics collection
#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage percentage
    pub memory_usage: f64,
    /// Number of active threads
    pub active_threads: usize,
}

/// Analysis data for lockfree tracking
#[derive(Debug, Clone, Default)]
pub struct AnalysisData {
    /// Memory statistics
    pub stats: MemoryStats,
    /// System metrics
    pub system_metrics: SystemMetrics,
    /// Frequency data
    pub frequency_data: Vec<FrequencyData>,
    /// Timestamp of analysis
    pub timestamp: u64,
}

/// Frequency pattern analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrequencyPattern {
    /// No clear pattern
    None,
    /// Frequent small allocations
    FrequentSmall,
    /// Infrequent large allocations
    InfrequentLarge,
    /// Mixed pattern
    Mixed,
}

/// Access pattern analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPattern {
    /// Sequential access pattern
    Sequential,
    /// Random access pattern
    Random,
    /// Unknown pattern
    Unknown,
}

/// Sampling configuration for intelligent allocation tracking
///
/// Uses dual-dimension sampling (size + frequency) to balance performance
/// with data completeness. Large allocations and high-frequency patterns
/// receive priority sampling.
#[derive(Debug, Clone)]
pub struct SamplingConfig {
    /// Sample rate for large allocations - usually 100% to catch memory leaks
    pub large_allocation_rate: f64,
    /// Sample rate for medium allocations - balanced approach
    pub medium_allocation_rate: f64,
    /// Sample rate for small allocations - low to reduce overhead
    pub small_allocation_rate: f64,
    /// Size threshold for large allocations (bytes)
    pub large_threshold: usize,
    /// Size threshold for medium allocations (bytes)
    pub medium_threshold: usize,
    /// Frequency threshold for sampling boost
    pub frequency_threshold: u64,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            large_allocation_rate: 1.0,
            medium_allocation_rate: 0.1,
            small_allocation_rate: 0.01,
            large_threshold: 10 * 1024,
            medium_threshold: 1024,
            frequency_threshold: 10,
        }
    }
}

impl SamplingConfig {
    /// Creates high-precision configuration for debugging scenarios
    pub fn high_precision() -> Self {
        Self {
            large_allocation_rate: 1.0,
            medium_allocation_rate: 0.5,
            small_allocation_rate: 0.1,
            large_threshold: 4 * 1024,
            medium_threshold: 512,
            frequency_threshold: 5,
        }
    }

    /// Creates performance-optimized configuration for production
    pub fn performance_optimized() -> Self {
        Self {
            large_allocation_rate: 1.0,
            medium_allocation_rate: 0.05,
            small_allocation_rate: 0.001,
            large_threshold: 50 * 1024,
            medium_threshold: 5 * 1024,
            frequency_threshold: 50,
        }
    }

    /// Creates configuration for memory leak detection
    pub fn leak_detection() -> Self {
        Self {
            large_allocation_rate: 1.0,
            medium_allocation_rate: 0.8,
            small_allocation_rate: 0.01,
            large_threshold: 1024,
            medium_threshold: 256,
            frequency_threshold: 3,
        }
    }

    /// Creates configuration for demonstrations and testing
    pub fn demo() -> Self {
        Self {
            large_allocation_rate: 1.0,
            medium_allocation_rate: 1.0,
            small_allocation_rate: 1.0,
            large_threshold: 8 * 1024,
            medium_threshold: 256,
            frequency_threshold: 1,
        }
    }

    /// Validates configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.large_allocation_rate) {
            return Err("Large allocation rate must be between 0.0 and 1.0".to_string());
        }
        if !(0.0..=1.0).contains(&self.medium_allocation_rate) {
            return Err("Medium allocation rate must be between 0.0 and 1.0".to_string());
        }
        if !(0.0..=1.0).contains(&self.small_allocation_rate) {
            return Err("Small allocation rate must be between 0.0 and 1.0".to_string());
        }
        if self.large_threshold <= self.medium_threshold {
            return Err("Large threshold must be greater than medium threshold".to_string());
        }
        if self.medium_threshold == 0 {
            return Err("Medium threshold must be greater than 0".to_string());
        }
        Ok(())
    }

    /// Calculates expected sampling rate for given allocation size
    pub fn base_sampling_rate(&self, size: usize) -> f64 {
        if size >= self.large_threshold {
            self.large_allocation_rate
        } else if size >= self.medium_threshold {
            self.medium_allocation_rate
        } else {
            self.small_allocation_rate
        }
    }

    /// Calculates frequency multiplier for sampling boost
    pub fn frequency_multiplier(&self, frequency: u64) -> f64 {
        if frequency > self.frequency_threshold {
            (frequency as f64 / self.frequency_threshold as f64).min(10.0)
        } else {
            1.0
        }
    }
}

/// Memory snapshot for real-time monitoring
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    /// Current memory usage in megabytes
    pub current_mb: f64,
    /// Peak memory usage in megabytes
    pub peak_mb: f64,
    /// Total number of allocations tracked
    pub allocations: u64,
    /// Total number of deallocations tracked
    pub deallocations: u64,
    /// Number of threads currently being tracked
    pub active_threads: usize,
}

/// Thread statistics for lockfree analysis
#[derive(Debug, Clone)]
pub struct ThreadStats {
    /// Thread identifier
    pub thread_id: u64,
    /// Total number of allocations tracked
    pub total_allocations: u64,
    /// Total number of deallocations tracked
    pub total_deallocations: u64,
    /// Peak memory usage observed
    pub peak_memory: usize,
    /// Total bytes allocated (sum of all allocations)
    pub total_allocated: usize,
    /// Frequency of allocations per call stack
    pub allocation_frequency: std::collections::HashMap<u64, u64>,
    /// Average allocation size for this thread
    pub avg_allocation_size: f64,
}

/// Hot call stack information
#[deprecated(
    since = "0.1.10",
    note = "Use capture::backends::hotspot_analysis::CallStackHotspot instead. This module will be removed in a future version."
)]
#[derive(Debug, Clone)]
pub struct HotCallStack {
    /// Hash of the call stack
    pub call_stack_hash: u64,
    /// Total frequency across all threads
    pub total_frequency: u64,
    /// Total memory allocated by this call stack
    pub total_size: usize,
    /// Impact score (frequency * size)
    pub impact_score: u64,
    /// Threads that use this call stack
    pub threads: Vec<u64>,
}

/// Thread interaction type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionType {
    /// Similar allocation patterns
    SimilarPatterns,
    /// Potential memory sharing
    MemorySharing,
    /// Producer-consumer relationship
    ProducerConsumer,
}

/// Thread interaction information
#[derive(Debug, Clone)]
pub struct ThreadInteraction {
    /// First thread in the interaction
    pub thread_a: u64,
    /// Second thread in the interaction
    pub thread_b: u64,
    /// Shared memory patterns
    pub shared_patterns: Vec<u64>,
    /// Frequency of interaction
    pub interaction_strength: u64,
    /// Type of interaction detected
    pub interaction_type: InteractionType,
}

/// Memory usage peak
#[deprecated(
    since = "0.1.10",
    note = "Use capture::backends::hotspot_analysis::MemoryUsagePeak instead. This module will be removed in a future version."
)]
#[derive(Debug, Clone)]
pub struct MemoryPeak {
    /// Timestamp of the peak
    pub timestamp: u64,
    /// Thread that caused the peak
    pub thread_id: u64,
    /// Memory usage at peak (bytes)
    pub memory_usage: usize,
    /// Number of active allocations
    pub active_allocations: u64,
    /// Call stack that triggered the peak
    pub triggering_call_stack: u64,
}

/// Performance bottleneck type
#[deprecated(
    since = "0.1.10",
    note = "Use capture::backends::bottleneck_analysis::BottleneckKind instead. This module will be removed in a future version."
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottleneckType {
    /// High frequency small allocations
    HighFrequencySmallAllocation,
    /// Large allocation spike
    LargeAllocationSpike,
    /// Potential memory leak
    MemoryLeak,
    /// Thread contention
    ThreadContention,
    /// Fragmentation risk
    FragmentationRisk,
}

/// Performance bottleneck information
#[deprecated(
    since = "0.1.10",
    note = "Use capture::backends::bottleneck_analysis::PerformanceIssue instead. This module will be removed in a future version."
)]
#[derive(Debug, Clone)]
pub struct PerformanceBottleneck {
    /// Type of bottleneck detected
    pub bottleneck_type: BottleneckType,
    /// Thread where bottleneck was detected
    pub thread_id: u64,
    /// Call stack associated with bottleneck
    pub call_stack_hash: u64,
    /// Severity score (0.0 to 1.0)
    pub severity: f64,
    /// Human-readable description
    pub description: String,
    /// Suggested remediation
    pub suggestion: String,
}

/// Analysis summary
#[derive(Debug, Clone)]
pub struct AnalysisSummary {
    /// Total number of threads analyzed
    pub total_threads: usize,
    /// Total allocations across all threads
    pub total_allocations: u64,
    /// Total deallocations across all threads
    pub total_deallocations: u64,
    /// Peak memory usage across all threads
    pub peak_memory_usage: usize,
    /// Total memory allocated across all threads
    pub total_memory_allocated: usize,
    /// Number of unique call stacks detected
    pub unique_call_stacks: usize,
    /// Analysis duration in milliseconds
    pub analysis_duration_ms: u64,
    /// Sampling effectiveness (percentage of allocations sampled)
    pub sampling_effectiveness: f64,
}

/// Comprehensive analysis results from multiple threads
#[derive(Debug, Clone)]
pub struct LockfreeAnalysis {
    /// Statistics for each thread
    pub thread_stats: std::collections::HashMap<u64, ThreadStats>,
    /// Most frequently used call stacks across all threads
    pub hottest_call_stacks: Vec<HotCallStack>,
    /// Detected interactions between threads
    pub thread_interactions: Vec<ThreadInteraction>,
    /// Memory usage peaks across all threads
    pub memory_peaks: Vec<MemoryPeak>,
    /// Detected performance bottlenecks
    pub performance_bottlenecks: Vec<PerformanceBottleneck>,
    /// Overall summary statistics
    pub summary: AnalysisSummary,
}

impl Default for LockfreeAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

impl LockfreeAnalysis {
    /// Creates an empty analysis result
    pub fn new() -> Self {
        Self {
            thread_stats: std::collections::HashMap::new(),
            hottest_call_stacks: Vec::new(),
            thread_interactions: Vec::new(),
            memory_peaks: Vec::new(),
            performance_bottlenecks: Vec::new(),
            summary: AnalysisSummary {
                total_threads: 0,
                total_allocations: 0,
                total_deallocations: 0,
                peak_memory_usage: 0,
                total_memory_allocated: 0,
                unique_call_stacks: 0,
                analysis_duration_ms: 0,
                sampling_effectiveness: 0.0,
            },
        }
    }

    /// Gets threads with highest allocation activity
    pub fn get_most_active_threads(&self, limit: usize) -> Vec<(u64, u64)> {
        let mut thread_activity: Vec<_> = self
            .thread_stats
            .iter()
            .map(|(&thread_id, stats)| (thread_id, stats.total_allocations))
            .collect();

        thread_activity.sort_by(|a, b| b.1.cmp(&a.1));
        thread_activity.truncate(limit);
        thread_activity
    }

    /// Gets threads with highest memory usage
    pub fn get_highest_memory_threads(&self, limit: usize) -> Vec<(u64, usize)> {
        let mut thread_memory: Vec<_> = self
            .thread_stats
            .iter()
            .map(|(&thread_id, stats)| (thread_id, stats.peak_memory))
            .collect();

        thread_memory.sort_by(|a, b| b.1.cmp(&a.1));
        thread_memory.truncate(limit);
        thread_memory
    }

    /// Gets most severe performance bottlenecks
    pub fn get_critical_bottlenecks(&self, severity_threshold: f64) -> Vec<&PerformanceBottleneck> {
        self.performance_bottlenecks
            .iter()
            .filter(|b| b.severity >= severity_threshold)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let thread_id = std::thread::current().id();
        let alloc_event = Event::allocation(0x1000, 1024, 12345, thread_id);
        assert_eq!(alloc_event.event_type, EventType::Allocation);
        assert_eq!(alloc_event.ptr, 0x1000);
        assert_eq!(alloc_event.size, 1024);

        let dealloc_event = Event::deallocation(0x1000, 1024, 12345, thread_id);
        assert_eq!(dealloc_event.event_type, EventType::Deallocation);
    }

    #[test]
    fn test_memory_stats_default() {
        let stats = MemoryStats::default();
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.total_allocated, 0);
        assert_eq!(stats.peak_memory, 0);
    }

    #[test]
    fn test_allocation_category() {
        let small = AllocationCategory::Small;
        let medium = AllocationCategory::Medium;
        let large = AllocationCategory::Large;

        assert_eq!(small, AllocationCategory::Small);
        assert_eq!(medium, AllocationCategory::Medium);
        assert_eq!(large, AllocationCategory::Large);
        assert_ne!(small, medium);
    }

    #[test]
    fn test_sampling_config_default() {
        let config = SamplingConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.large_allocation_rate, 1.0);
        assert_eq!(config.medium_allocation_rate, 0.1);
        assert_eq!(config.small_allocation_rate, 0.01);
    }

    #[test]
    fn test_sampling_config_presets() {
        assert!(SamplingConfig::high_precision().validate().is_ok());
        assert!(SamplingConfig::performance_optimized().validate().is_ok());
        assert!(SamplingConfig::leak_detection().validate().is_ok());
        assert!(SamplingConfig::demo().validate().is_ok());
    }

    #[test]
    fn test_sampling_config_base_rate() {
        let config = SamplingConfig::default();
        assert_eq!(config.base_sampling_rate(20 * 1024), 1.0);
        assert_eq!(config.base_sampling_rate(5 * 1024), 0.1);
        assert_eq!(config.base_sampling_rate(512), 0.01);
    }

    #[test]
    fn test_memory_snapshot() {
        let snapshot = MemorySnapshot {
            current_mb: 10.0,
            peak_mb: 20.0,
            allocations: 100,
            deallocations: 50,
            active_threads: 2,
        };
        assert_eq!(snapshot.current_mb, 10.0);
        assert_eq!(snapshot.active_threads, 2);
    }
}
