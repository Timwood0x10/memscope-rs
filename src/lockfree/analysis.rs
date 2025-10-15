//! Analysis structures and types for lock-free tracking results
//!
//! This module defines the data structures used to represent analysis
//! results from lock-free multi-threaded tracking data.

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-thread statistics from lock-free tracking
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub enum EventType {
    /// Memory allocation event
    Allocation,
    /// Memory deallocation event
    Deallocation,
}

/// Comprehensive analysis results from multiple threads
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum InteractionType {
    /// Similar allocation patterns
    SimilarPatterns,
    /// Potential memory sharing
    MemorySharing,
    /// Producer-consumer relationship
    ProducerConsumer,
}

/// Memory usage peak
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
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
            peak_memory += stats.peak_memory; // Fix: accumulate peak memory from all threads
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn create_test_thread_stats(
        thread_id: u64,
        allocations: u64,
        peak_memory: usize,
    ) -> ThreadStats {
        let mut allocation_frequency = HashMap::new();
        allocation_frequency.insert(12345, allocations / 2);
        allocation_frequency.insert(67890, allocations / 2);

        ThreadStats {
            thread_id,
            total_allocations: allocations,
            total_deallocations: allocations - 1, // Simulate one outstanding allocation
            peak_memory,
            total_allocated: peak_memory,
            allocation_frequency,
            avg_allocation_size: peak_memory as f64 / allocations as f64,
            timeline: vec![
                AllocationEvent {
                    timestamp: 1000,
                    ptr: 0x1000,
                    size: 1024,
                    call_stack_hash: 12345,
                    event_type: EventType::Allocation,
                    thread_id,
                },
                AllocationEvent {
                    timestamp: 2000,
                    ptr: 0x2000,
                    size: 2048,
                    call_stack_hash: 67890,
                    event_type: EventType::Allocation,
                    thread_id,
                },
            ],
        }
    }

    #[test]
    fn test_lockfree_analysis_creation() {
        let analysis = LockfreeAnalysis::new();

        assert!(analysis.thread_stats.is_empty());
        assert!(analysis.hottest_call_stacks.is_empty());
        assert!(analysis.thread_interactions.is_empty());
        assert!(analysis.memory_peaks.is_empty());
        assert!(analysis.performance_bottlenecks.is_empty());
        assert_eq!(analysis.summary.total_threads, 0);
        assert_eq!(analysis.summary.total_allocations, 0);
    }

    #[test]
    fn test_lockfree_analysis_default() {
        let analysis = LockfreeAnalysis::default();
        assert!(analysis.thread_stats.is_empty());
        assert_eq!(analysis.summary.total_threads, 0);
    }

    #[test]
    fn test_calculate_summary_single_thread() {
        let mut analysis = LockfreeAnalysis::new();
        let start_time = Instant::now();

        // Add a single thread
        let thread_stats = create_test_thread_stats(1, 100, 8192);
        analysis.thread_stats.insert(1, thread_stats);

        analysis.calculate_summary(start_time);

        assert_eq!(analysis.summary.total_threads, 1);
        assert_eq!(analysis.summary.total_allocations, 100);
        assert_eq!(analysis.summary.total_deallocations, 99);
        assert_eq!(analysis.summary.peak_memory_usage, 8192); // Single thread
        assert_eq!(analysis.summary.total_memory_allocated, 8192);
        assert_eq!(analysis.summary.unique_call_stacks, 2); // Two call stacks in test data
                                                            // assert!(analysis.summary.analysis_duration_ms >= 0); // Always true for u64
    }

    #[test]
    fn test_calculate_summary_multiple_threads() {
        let mut analysis = LockfreeAnalysis::new();
        let start_time = Instant::now();

        // Add multiple threads
        analysis
            .thread_stats
            .insert(1, create_test_thread_stats(1, 100, 4096));
        analysis
            .thread_stats
            .insert(2, create_test_thread_stats(2, 50, 2048));
        analysis
            .thread_stats
            .insert(3, create_test_thread_stats(3, 200, 8192));

        analysis.calculate_summary(start_time);

        assert_eq!(analysis.summary.total_threads, 3);
        assert_eq!(analysis.summary.total_allocations, 350); // 100 + 50 + 200
        assert_eq!(analysis.summary.total_deallocations, 347); // 99 + 49 + 199
        assert_eq!(analysis.summary.peak_memory_usage, 14336); // Sum of all threads: 4096+2048+8192
        assert_eq!(analysis.summary.total_memory_allocated, 14336); // Sum of all threads
        assert_eq!(analysis.summary.unique_call_stacks, 2); // Same call stacks used by all
    }

    #[test]
    fn test_get_most_active_threads() {
        let mut analysis = LockfreeAnalysis::new();

        analysis
            .thread_stats
            .insert(1, create_test_thread_stats(1, 100, 4096));
        analysis
            .thread_stats
            .insert(2, create_test_thread_stats(2, 300, 2048));
        analysis
            .thread_stats
            .insert(3, create_test_thread_stats(3, 50, 8192));

        let most_active = analysis.get_most_active_threads(2);

        assert_eq!(most_active.len(), 2);
        assert_eq!(most_active[0], (2, 300)); // Most active thread
        assert_eq!(most_active[1], (1, 100)); // Second most active
    }

    #[test]
    fn test_get_most_active_threads_empty() {
        let analysis = LockfreeAnalysis::new();
        let most_active = analysis.get_most_active_threads(5);
        assert!(most_active.is_empty());
    }

    #[test]
    fn test_get_highest_memory_threads() {
        let mut analysis = LockfreeAnalysis::new();

        analysis
            .thread_stats
            .insert(1, create_test_thread_stats(1, 100, 4096));
        analysis
            .thread_stats
            .insert(2, create_test_thread_stats(2, 300, 2048));
        analysis
            .thread_stats
            .insert(3, create_test_thread_stats(3, 50, 8192));

        let highest_memory = analysis.get_highest_memory_threads(2);

        assert_eq!(highest_memory.len(), 2);
        assert_eq!(highest_memory[0], (3, 8192)); // Highest memory usage
        assert_eq!(highest_memory[1], (1, 4096)); // Second highest
    }

    #[test]
    fn test_get_critical_bottlenecks() {
        let mut analysis = LockfreeAnalysis::new();

        analysis
            .performance_bottlenecks
            .push(PerformanceBottleneck {
                bottleneck_type: BottleneckType::HighFrequencySmallAllocation,
                thread_id: 1,
                call_stack_hash: 12345,
                severity: 0.8,
                description: "High frequency allocations detected".to_string(),
                suggestion: "Consider using a memory pool".to_string(),
            });

        analysis
            .performance_bottlenecks
            .push(PerformanceBottleneck {
                bottleneck_type: BottleneckType::MemoryLeak,
                thread_id: 2,
                call_stack_hash: 67890,
                severity: 0.3,
                description: "Potential memory leak".to_string(),
                suggestion: "Check deallocation patterns".to_string(),
            });

        let critical = analysis.get_critical_bottlenecks(0.7);
        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0].severity, 0.8);

        let all_bottlenecks = analysis.get_critical_bottlenecks(0.0);
        assert_eq!(all_bottlenecks.len(), 2);
    }

    #[test]
    fn test_event_type_equality() {
        assert_eq!(EventType::Allocation, EventType::Allocation);
        assert_eq!(EventType::Deallocation, EventType::Deallocation);
        assert_ne!(EventType::Allocation, EventType::Deallocation);
    }

    #[test]
    fn test_allocation_event_creation() {
        let event = AllocationEvent {
            timestamp: 12345,
            ptr: 0x1000,
            size: 1024,
            call_stack_hash: 67890,
            event_type: EventType::Allocation,
            thread_id: 1,
        };

        assert_eq!(event.timestamp, 12345);
        assert_eq!(event.ptr, 0x1000);
        assert_eq!(event.size, 1024);
        assert_eq!(event.call_stack_hash, 67890);
        assert_eq!(event.event_type, EventType::Allocation);
        assert_eq!(event.thread_id, 1);
    }

    #[test]
    fn test_thread_stats_creation() {
        let stats = create_test_thread_stats(42, 1000, 16384);

        assert_eq!(stats.thread_id, 42);
        assert_eq!(stats.total_allocations, 1000);
        assert_eq!(stats.total_deallocations, 999);
        assert_eq!(stats.peak_memory, 16384);
        assert_eq!(stats.total_allocated, 16384);
        assert_eq!(stats.avg_allocation_size, 16.384);
        assert_eq!(stats.allocation_frequency.len(), 2);
        assert_eq!(stats.timeline.len(), 2);
    }

    #[test]
    fn test_hot_call_stack() {
        let hot_stack = HotCallStack {
            call_stack_hash: 12345,
            total_frequency: 100,
            total_size: 8192,
            impact_score: 819200,
            threads: vec![1, 2, 3],
        };

        assert_eq!(hot_stack.call_stack_hash, 12345);
        assert_eq!(hot_stack.total_frequency, 100);
        assert_eq!(hot_stack.total_size, 8192);
        assert_eq!(hot_stack.impact_score, 819200);
        assert_eq!(hot_stack.threads.len(), 3);
    }

    #[test]
    fn test_thread_interaction() {
        let interaction = ThreadInteraction {
            thread_a: 1,
            thread_b: 2,
            shared_patterns: vec![12345, 67890],
            interaction_strength: 50,
            interaction_type: InteractionType::SimilarPatterns,
        };

        assert_eq!(interaction.thread_a, 1);
        assert_eq!(interaction.thread_b, 2);
        assert_eq!(interaction.shared_patterns.len(), 2);
        assert_eq!(interaction.interaction_strength, 50);
    }

    #[test]
    fn test_memory_peak() {
        let peak = MemoryPeak {
            timestamp: 123456789,
            thread_id: 3,
            memory_usage: 1048576,
            active_allocations: 500,
            triggering_call_stack: 12345,
        };

        assert_eq!(peak.timestamp, 123456789);
        assert_eq!(peak.thread_id, 3);
        assert_eq!(peak.memory_usage, 1048576);
        assert_eq!(peak.active_allocations, 500);
        assert_eq!(peak.triggering_call_stack, 12345);
    }

    #[test]
    fn test_performance_bottleneck_types() {
        let bottleneck1 = PerformanceBottleneck {
            bottleneck_type: BottleneckType::HighFrequencySmallAllocation,
            thread_id: 1,
            call_stack_hash: 12345,
            severity: 0.8,
            description: "High frequency".to_string(),
            suggestion: "Use pools".to_string(),
        };

        let bottleneck2 = PerformanceBottleneck {
            bottleneck_type: BottleneckType::LargeAllocationSpike,
            thread_id: 2,
            call_stack_hash: 67890,
            severity: 0.6,
            description: "Large spike".to_string(),
            suggestion: "Optimize allocation".to_string(),
        };

        // Test different bottleneck types can be created
        assert_eq!(bottleneck1.thread_id, 1);
        assert_eq!(bottleneck2.thread_id, 2);
    }

    #[test]
    fn test_interaction_types() {
        let similar = InteractionType::SimilarPatterns;
        let sharing = InteractionType::MemorySharing;
        let producer_consumer = InteractionType::ProducerConsumer;

        // Test that all interaction types can be created
        // (Mainly for coverage, these are simple enums)
        let _interactions = [similar, sharing, producer_consumer];
    }

    #[test]
    fn test_bottleneck_types_complete() {
        let types = [
            BottleneckType::HighFrequencySmallAllocation,
            BottleneckType::LargeAllocationSpike,
            BottleneckType::MemoryLeak,
            BottleneckType::ThreadContention,
            BottleneckType::FragmentationRisk,
        ];

        // Test that all bottleneck types can be created
        assert_eq!(types.len(), 5);
    }

    #[test]
    fn test_analysis_summary_fields() {
        let summary = AnalysisSummary {
            total_threads: 5,
            total_allocations: 1000,
            total_deallocations: 950,
            peak_memory_usage: 16384,
            total_memory_allocated: 32768,
            unique_call_stacks: 25,
            analysis_duration_ms: 500,
            sampling_effectiveness: 85.5,
        };

        assert_eq!(summary.total_threads, 5);
        assert_eq!(summary.total_allocations, 1000);
        assert_eq!(summary.total_deallocations, 950);
        assert_eq!(summary.peak_memory_usage, 16384);
        assert_eq!(summary.total_memory_allocated, 32768);
        assert_eq!(summary.unique_call_stacks, 25);
        assert_eq!(summary.analysis_duration_ms, 500);
        assert_eq!(summary.sampling_effectiveness, 85.5);
    }

    #[test]
    fn test_complex_analysis_workflow() {
        let mut analysis = LockfreeAnalysis::new();
        let start_time = Instant::now();

        // Build a complex analysis scenario
        analysis
            .thread_stats
            .insert(1, create_test_thread_stats(1, 1000, 8192));
        analysis
            .thread_stats
            .insert(2, create_test_thread_stats(2, 500, 4096));
        analysis
            .thread_stats
            .insert(3, create_test_thread_stats(3, 2000, 16384));

        analysis.hottest_call_stacks.push(HotCallStack {
            call_stack_hash: 12345,
            total_frequency: 1000,
            total_size: 16384,
            impact_score: 16384000,
            threads: vec![1, 2, 3],
        });

        analysis.thread_interactions.push(ThreadInteraction {
            thread_a: 1,
            thread_b: 2,
            shared_patterns: vec![12345],
            interaction_strength: 75,
            interaction_type: InteractionType::SimilarPatterns,
        });

        analysis.memory_peaks.push(MemoryPeak {
            timestamp: 1000000,
            thread_id: 3,
            memory_usage: 16384,
            active_allocations: 1000,
            triggering_call_stack: 12345,
        });

        analysis
            .performance_bottlenecks
            .push(PerformanceBottleneck {
                bottleneck_type: BottleneckType::HighFrequencySmallAllocation,
                thread_id: 3,
                call_stack_hash: 12345,
                severity: 0.9,
                description: "Very high allocation frequency".to_string(),
                suggestion: "Consider object pooling".to_string(),
            });

        analysis.calculate_summary(start_time);

        // Verify complex analysis state
        assert_eq!(analysis.summary.total_threads, 3);
        assert_eq!(analysis.summary.total_allocations, 3500);
        assert!(analysis.get_most_active_threads(1)[0].1 == 2000); // Thread 3 most active
        assert!(analysis.get_highest_memory_threads(1)[0].1 == 16384); // Thread 3 highest memory
        assert_eq!(analysis.get_critical_bottlenecks(0.8).len(), 1);
        assert_eq!(analysis.hottest_call_stacks.len(), 1);
        assert_eq!(analysis.thread_interactions.len(), 1);
        assert_eq!(analysis.memory_peaks.len(), 1);
    }

    #[test]
    fn test_edge_cases() {
        let mut analysis = LockfreeAnalysis::new();

        // Test with zero allocations thread
        let empty_stats = ThreadStats {
            thread_id: 99,
            total_allocations: 0,
            total_deallocations: 0,
            peak_memory: 0,
            total_allocated: 0,
            allocation_frequency: HashMap::new(),
            avg_allocation_size: 0.0,
            timeline: Vec::new(),
        };
        analysis.thread_stats.insert(99, empty_stats);

        let start_time = Instant::now();
        analysis.calculate_summary(start_time);

        assert_eq!(analysis.summary.total_threads, 1);
        assert_eq!(analysis.summary.total_allocations, 0);
        assert_eq!(analysis.summary.sampling_effectiveness, 0.0);

        // Test getting active threads with no activity
        let active = analysis.get_most_active_threads(10);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].1, 0); // Zero allocations
    }

    #[test]
    fn test_serialization_deserialization() {
        // Test that structures can be serialized (important for data export)
        let analysis = LockfreeAnalysis::new();

        // This will compile if Serialize/Deserialize are properly implemented
        let _json = serde_json::to_string(&analysis);

        let event = AllocationEvent {
            timestamp: 1000,
            ptr: 0x1000,
            size: 1024,
            call_stack_hash: 12345,
            event_type: EventType::Allocation,
            thread_id: 1,
        };

        let _event_json = serde_json::to_string(&event);
    }
}
