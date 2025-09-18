//! Analysis structures and types for lock-free tracking results
//!
//! This module defines the data structures used to represent analysis
//! results from lock-free multi-threaded tracking data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-thread statistics from lock-free tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub allocation_frequency: HashMap<u64, u64>,
    /// Average allocation size for this thread
    pub avg_allocation_size: f64,
    /// Timeline of allocation events (sampled)
    pub timeline: Vec<AllocationEvent>,
}

/// Allocation event from lock-free tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationEvent {
    /// Timestamp in nanoseconds
    pub timestamp: u64,
    /// Memory pointer address
    pub ptr: usize,
    /// Allocation size in bytes
    pub size: usize,
    /// Hash of the call stack
    pub call_stack_hash: u64,
    /// Event type (allocation or deallocation)
    pub event_type: EventType,
    /// Thread that performed the operation
    pub thread_id: u64,
}

/// Type of memory event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    /// Memory allocation event
    Allocation,
    /// Memory deallocation event
    Deallocation,
}

/// Comprehensive analysis results from multiple threads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfreeAnalysis {
    /// Statistics for each thread
    pub thread_stats: HashMap<u64, ThreadStats>,
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

/// Hot call stack information
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Interaction between threads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInteraction {
    /// First thread in the interaction
    pub thread_a: u64,
    /// Second thread in the interaction
    pub thread_b: u64,
    /// Shared memory regions (simplified as call stack hashes)
    pub shared_patterns: Vec<u64>,
    /// Frequency of interaction
    pub interaction_strength: u64,
    /// Type of interaction detected
    pub interaction_type: InteractionType,
}

/// Type of thread interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    /// Similar allocation patterns
    SimilarPatterns,
    /// Potential memory sharing
    MemorySharing,
    /// Producer-consumer relationship
    ProducerConsumer,
}

/// Memory usage peak
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Performance bottleneck detection
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Type of performance bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckType {
    /// High frequency small allocations
    HighFrequencySmallAllocation,
    /// Large allocation spike
    LargeAllocationSpike,
    /// Potential memory leak
    MemoryLeak,
    /// Thread contention (in case any remains)
    ThreadContention,
    /// Fragmentation risk
    FragmentationRisk,
}

/// Overall analysis summary
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Analysis duration
    pub analysis_duration_ms: u64,
    /// Sampling effectiveness (percentage of allocations sampled)
    pub sampling_effectiveness: f64,
}

impl LockfreeAnalysis {
    /// Creates an empty analysis result
    pub fn new() -> Self {
        Self {
            thread_stats: HashMap::new(),
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

    /// Calculates and updates the summary statistics
    pub fn calculate_summary(&mut self, analysis_start: std::time::Instant) {
        let mut total_allocations = 0;
        let mut total_deallocations = 0;
        let mut peak_memory = 0;
        let mut total_allocated = 0;
        let mut all_call_stacks = std::collections::HashSet::new();

        for stats in self.thread_stats.values() {
            total_allocations += stats.total_allocations;
            total_deallocations += stats.total_deallocations;
            peak_memory = peak_memory.max(stats.peak_memory);
            total_allocated += stats.total_allocated;

            for &call_stack_hash in stats.allocation_frequency.keys() {
                all_call_stacks.insert(call_stack_hash);
            }
        }

        self.summary = AnalysisSummary {
            total_threads: self.thread_stats.len(),
            total_allocations,
            total_deallocations,
            peak_memory_usage: peak_memory,
            total_memory_allocated: total_allocated,
            unique_call_stacks: all_call_stacks.len(),
            analysis_duration_ms: analysis_start.elapsed().as_millis() as u64,
            sampling_effectiveness: if total_allocations > 0 {
                // Estimate based on sampled data vs expected full data
                100.0 // Placeholder - would need more sophisticated calculation
            } else {
                0.0
            },
        };
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

impl Default for LockfreeAnalysis {
    fn default() -> Self {
        Self::new()
    }
}
