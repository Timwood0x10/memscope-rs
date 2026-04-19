//! Lockfree tracker types and data structures.
//!
//! This module contains type definitions for the lockfree memory tracking system.

use super::bottleneck_analysis::PerformanceIssue;
use super::hotspot_analysis::{CallStackHotspot, MemoryUsagePeak};
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
    /// Clone event (Rc/Arc clone)
    Clone,
    /// Move event
    Move,
    /// Borrow event
    Borrow,
    /// Mutable borrow event
    MutBorrow,
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
    pub hottest_call_stacks: Vec<CallStackHotspot>,
    /// Detected interactions between threads
    pub thread_interactions: Vec<ThreadInteraction>,
    /// Memory usage peaks across all threads
    pub memory_peaks: Vec<MemoryUsagePeak>,
    /// Detected performance bottlenecks
    pub performance_bottlenecks: Vec<PerformanceIssue>,
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
    pub fn get_critical_bottlenecks(&self, severity_threshold: f64) -> Vec<&PerformanceIssue> {
        self.performance_bottlenecks
            .iter()
            .filter(|b| b.severity >= severity_threshold)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::bottleneck_analysis::{BottleneckKind, BottleneckMetrics};
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

    #[test]
    fn test_event_type_variants() {
        assert_eq!(EventType::Allocation, EventType::Allocation);
        assert_eq!(EventType::Deallocation, EventType::Deallocation);
        assert_eq!(EventType::Clone, EventType::Clone);
        assert_eq!(EventType::Move, EventType::Move);
        assert_eq!(EventType::Borrow, EventType::Borrow);
        assert_eq!(EventType::MutBorrow, EventType::MutBorrow);
        assert_ne!(EventType::Allocation, EventType::Deallocation);
        assert_ne!(EventType::Clone, EventType::Move);
    }

    #[test]
    fn test_event_metadata_creation() {
        let metadata = EventMetadata {
            var_name: Some("test_var".to_string()),
            type_name: Some("Vec<u8>".to_string()),
        };

        assert_eq!(metadata.var_name, Some("test_var".to_string()));
        assert_eq!(metadata.type_name, Some("Vec<u8>".to_string()));
    }

    #[test]
    fn test_event_with_metadata() {
        let thread_id = std::thread::current().id();
        let mut event = Event::allocation(0x2000, 2048, 54321, thread_id);
        event.metadata = Some(EventMetadata {
            var_name: Some("buffer".to_string()),
            type_name: None,
        });

        assert!(event.metadata.is_some());
        assert_eq!(
            event.metadata.as_ref().unwrap().var_name,
            Some("buffer".to_string())
        );
    }

    #[test]
    fn test_frequency_data_creation() {
        let freq = FrequencyData {
            call_stack_hash: 12345,
            count: 100,
            total_size: 10240,
            first_timestamp: 1000,
            last_timestamp: 5000,
        };

        assert_eq!(freq.call_stack_hash, 12345);
        assert_eq!(freq.count, 100);
        assert_eq!(freq.total_size, 10240);
    }

    #[test]
    fn test_memory_stats_with_values() {
        let stats = MemoryStats {
            total_allocations: 1000,
            total_allocated: 1024000,
            total_deallocations: 800,
            total_deallocated: 819200,
            peak_memory: 204800,
            active_memory: 204800,
        };

        assert_eq!(stats.total_allocations, 1000);
        assert_eq!(stats.active_memory, 204800);
    }

    #[test]
    fn test_real_call_stack_creation() {
        let stack = RealCallStack {
            frames: vec![
                StackFrame {
                    ip: 0x1000,
                    function_name: Some("main".to_string()),
                    file_name: Some("main.rs".to_string()),
                    line_number: Some(10),
                },
                StackFrame {
                    ip: 0x2000,
                    function_name: Some("foo".to_string()),
                    file_name: Some("lib.rs".to_string()),
                    line_number: Some(42),
                },
            ],
            hash: 98765,
        };

        assert_eq!(stack.frames.len(), 2);
        assert_eq!(stack.hash, 98765);
    }

    #[test]
    fn test_stack_frame_creation() {
        let frame = StackFrame {
            ip: 0x1234,
            function_name: Some("test_fn".to_string()),
            file_name: Some("test.rs".to_string()),
            line_number: Some(100),
        };

        assert_eq!(frame.ip, 0x1234);
        assert_eq!(frame.function_name, Some("test_fn".to_string()));
    }

    #[test]
    fn test_system_metrics_creation() {
        let metrics = SystemMetrics {
            cpu_usage: 45.5,
            memory_usage: 60.2,
            active_threads: 8,
        };

        assert!((metrics.cpu_usage - 45.5).abs() < f64::EPSILON);
        assert_eq!(metrics.active_threads, 8);
    }

    #[test]
    fn test_system_metrics_default() {
        let metrics = SystemMetrics::default();
        assert_eq!(metrics.cpu_usage, 0.0);
        assert_eq!(metrics.memory_usage, 0.0);
        assert_eq!(metrics.active_threads, 0);
    }

    #[test]
    fn test_analysis_data_default() {
        let data = AnalysisData::default();
        assert_eq!(data.stats.total_allocations, 0);
        assert_eq!(data.system_metrics.cpu_usage, 0.0);
        assert!(data.frequency_data.is_empty());
        assert_eq!(data.timestamp, 0);
    }

    #[test]
    fn test_analysis_data_with_values() {
        let data = AnalysisData {
            stats: MemoryStats {
                total_allocations: 100,
                total_allocated: 1024,
                total_deallocations: 50,
                total_deallocated: 512,
                peak_memory: 512,
                active_memory: 512,
            },
            system_metrics: SystemMetrics {
                cpu_usage: 50.0,
                memory_usage: 75.0,
                active_threads: 4,
            },
            frequency_data: vec![FrequencyData {
                call_stack_hash: 1,
                count: 10,
                total_size: 100,
                first_timestamp: 0,
                last_timestamp: 100,
            }],
            timestamp: 1234567890,
        };

        assert_eq!(data.stats.total_allocations, 100);
        assert_eq!(data.frequency_data.len(), 1);
    }

    #[test]
    fn test_frequency_pattern_variants() {
        let patterns = vec![
            FrequencyPattern::None,
            FrequencyPattern::FrequentSmall,
            FrequencyPattern::InfrequentLarge,
            FrequencyPattern::Mixed,
        ];

        for pattern in patterns {
            assert!(!format!("{pattern:?}").is_empty());
        }
    }

    #[test]
    fn test_access_pattern_variants() {
        let patterns = vec![
            AccessPattern::Sequential,
            AccessPattern::Random,
            AccessPattern::Unknown,
        ];

        for pattern in patterns {
            assert!(!format!("{pattern:?}").is_empty());
        }
    }

    #[test]
    fn test_sampling_config_validate_valid() {
        let config = SamplingConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_sampling_config_validate_invalid_large_rate() {
        let config = SamplingConfig {
            large_allocation_rate: 1.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_sampling_config_validate_invalid_medium_rate() {
        let config = SamplingConfig {
            medium_allocation_rate: -0.1,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_sampling_config_validate_invalid_thresholds() {
        let config = SamplingConfig {
            large_threshold: 100,
            medium_threshold: 200,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_sampling_config_validate_zero_medium_threshold() {
        let config = SamplingConfig {
            medium_threshold: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_sampling_config_frequency_multiplier() {
        let config = SamplingConfig::default();

        let mult1 = config.frequency_multiplier(5);
        assert!((mult1 - 1.0).abs() < f64::EPSILON);

        let mult2 = config.frequency_multiplier(20);
        assert!(mult2 > 1.0);

        let mult3 = config.frequency_multiplier(1000);
        assert!((mult3 - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_thread_stats_creation() {
        let mut freq = std::collections::HashMap::new();
        freq.insert(12345, 100);

        let stats = ThreadStats {
            thread_id: 1,
            total_allocations: 500,
            total_deallocations: 400,
            peak_memory: 10240,
            total_allocated: 51200,
            allocation_frequency: freq,
            avg_allocation_size: 102.4,
        };

        assert_eq!(stats.thread_id, 1);
        assert_eq!(stats.allocation_frequency.len(), 1);
    }

    #[test]
    fn test_interaction_type_variants() {
        let types = vec![
            InteractionType::SimilarPatterns,
            InteractionType::MemorySharing,
            InteractionType::ProducerConsumer,
        ];

        for interaction_type in types {
            assert!(!format!("{interaction_type:?}").is_empty());
        }
    }

    #[test]
    fn test_thread_interaction_creation() {
        let interaction = ThreadInteraction {
            thread_a: 1,
            thread_b: 2,
            shared_patterns: vec![100, 200, 300],
            interaction_strength: 50,
            interaction_type: InteractionType::ProducerConsumer,
        };

        assert_eq!(interaction.thread_a, 1);
        assert_eq!(interaction.thread_b, 2);
        assert_eq!(interaction.shared_patterns.len(), 3);
    }

    #[test]
    fn test_analysis_summary_creation() {
        let summary = AnalysisSummary {
            total_threads: 4,
            total_allocations: 1000,
            total_deallocations: 800,
            peak_memory_usage: 10240,
            total_memory_allocated: 51200,
            unique_call_stacks: 25,
            analysis_duration_ms: 150,
            sampling_effectiveness: 0.85,
        };

        assert_eq!(summary.total_threads, 4);
        assert!((summary.sampling_effectiveness - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lockfree_analysis_new() {
        let analysis = LockfreeAnalysis::new();

        assert!(analysis.thread_stats.is_empty());
        assert!(analysis.hottest_call_stacks.is_empty());
        assert!(analysis.thread_interactions.is_empty());
        assert!(analysis.memory_peaks.is_empty());
        assert!(analysis.performance_bottlenecks.is_empty());
        assert_eq!(analysis.summary.total_threads, 0);
    }

    #[test]
    fn test_lockfree_analysis_default() {
        let analysis = LockfreeAnalysis::default();
        assert!(analysis.thread_stats.is_empty());
    }

    #[test]
    fn test_lockfree_analysis_get_most_active_threads() {
        let mut analysis = LockfreeAnalysis::new();

        analysis.thread_stats.insert(
            1,
            ThreadStats {
                thread_id: 1,
                total_allocations: 100,
                total_deallocations: 80,
                peak_memory: 1024,
                total_allocated: 10240,
                allocation_frequency: std::collections::HashMap::new(),
                avg_allocation_size: 102.4,
            },
        );

        analysis.thread_stats.insert(
            2,
            ThreadStats {
                thread_id: 2,
                total_allocations: 200,
                total_deallocations: 150,
                peak_memory: 2048,
                total_allocated: 20480,
                allocation_frequency: std::collections::HashMap::new(),
                avg_allocation_size: 102.4,
            },
        );

        let active = analysis.get_most_active_threads(1);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].0, 2);
    }

    #[test]
    fn test_lockfree_analysis_get_highest_memory_threads() {
        let mut analysis = LockfreeAnalysis::new();

        analysis.thread_stats.insert(
            1,
            ThreadStats {
                thread_id: 1,
                total_allocations: 100,
                total_deallocations: 80,
                peak_memory: 1024,
                total_allocated: 10240,
                allocation_frequency: std::collections::HashMap::new(),
                avg_allocation_size: 102.4,
            },
        );

        analysis.thread_stats.insert(
            2,
            ThreadStats {
                thread_id: 2,
                total_allocations: 50,
                total_deallocations: 40,
                peak_memory: 4096,
                total_allocated: 40960,
                allocation_frequency: std::collections::HashMap::new(),
                avg_allocation_size: 819.2,
            },
        );

        let highest = analysis.get_highest_memory_threads(1);
        assert_eq!(highest.len(), 1);
        assert_eq!(highest[0].0, 2);
    }

    #[test]
    fn test_lockfree_analysis_get_critical_bottlenecks() {
        let mut analysis = LockfreeAnalysis::new();

        analysis.performance_bottlenecks = vec![
            PerformanceIssue {
                bottleneck_type: BottleneckKind::Cpu,
                task_id: 1,
                task_name: "task1".to_string(),
                severity: 0.9,
                description: "High CPU".to_string(),
                suggestion: "Optimize".to_string(),
                timestamp_ms: 1000,
                metrics: BottleneckMetrics {
                    cpu_usage_percent: 90.0,
                    memory_usage_percent: 0.0,
                    io_bytes_processed: 0,
                    network_bytes_transferred: 0,
                    lock_wait_time_ns: 0,
                    allocation_frequency: 0.0,
                    average_allocation_size: 0.0,
                },
            },
            PerformanceIssue {
                bottleneck_type: BottleneckKind::Cpu,
                task_id: 2,
                task_name: "task2".to_string(),
                severity: 0.3,
                description: "Low CPU".to_string(),
                suggestion: "Ignore".to_string(),
                timestamp_ms: 2000,
                metrics: BottleneckMetrics {
                    cpu_usage_percent: 30.0,
                    memory_usage_percent: 0.0,
                    io_bytes_processed: 0,
                    network_bytes_transferred: 0,
                    lock_wait_time_ns: 0,
                    allocation_frequency: 0.0,
                    average_allocation_size: 0.0,
                },
            },
        ];

        let critical = analysis.get_critical_bottlenecks(0.5);
        assert_eq!(critical.len(), 1);
    }

    #[test]
    fn test_event_clone() {
        let thread_id = std::thread::current().id();
        let event = Event::allocation(0x1000, 1024, 12345, thread_id);
        let cloned = event.clone();

        assert_eq!(cloned.ptr, event.ptr);
        assert_eq!(cloned.size, event.size);
        assert_eq!(cloned.event_type, event.event_type);
    }

    #[test]
    fn test_memory_stats_clone() {
        let stats = MemoryStats {
            total_allocations: 100,
            total_allocated: 1024,
            total_deallocations: 50,
            total_deallocated: 512,
            peak_memory: 512,
            active_memory: 512,
        };

        let cloned = stats.clone();
        assert_eq!(cloned.total_allocations, stats.total_allocations);
    }

    #[test]
    fn test_sampling_config_clone() {
        let config = SamplingConfig::high_precision();
        let cloned = config.clone();

        assert_eq!(cloned.large_allocation_rate, config.large_allocation_rate);
    }

    #[test]
    fn test_memory_snapshot_clone() {
        let snapshot = MemorySnapshot {
            current_mb: 10.0,
            peak_mb: 20.0,
            allocations: 100,
            deallocations: 50,
            active_threads: 2,
        };

        let cloned = snapshot.clone();
        assert_eq!(cloned.current_mb, snapshot.current_mb);
    }

    #[test]
    fn test_event_debug() {
        let thread_id = std::thread::current().id();
        let event = Event::allocation(0x1000, 1024, 12345, thread_id);
        let debug_str = format!("{:?}", event);

        assert!(debug_str.contains("Event"));
        assert!(debug_str.contains("Allocation"));
    }

    #[test]
    fn test_memory_stats_debug() {
        let stats = MemoryStats::default();
        let debug_str = format!("{:?}", stats);

        assert!(debug_str.contains("MemoryStats"));
    }

    #[test]
    fn test_sampling_config_debug() {
        let config = SamplingConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("SamplingConfig"));
    }

    #[test]
    fn test_lockfree_analysis_debug() {
        let analysis = LockfreeAnalysis::new();
        let debug_str = format!("{:?}", analysis);

        assert!(debug_str.contains("LockfreeAnalysis"));
    }
}
