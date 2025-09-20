//! Task memory profiling and performance metrics
//!
//! Provides data structures for tracking memory usage patterns
//! and performance characteristics of individual async tasks.

use std::time::{Duration, Instant};
// HashMap will be used for aggregated statistics in future versions

use crate::async_memory::TaskId;

/// Memory usage profile for a single async task
#[derive(Debug, Clone)]
pub struct TaskMemoryProfile {
    /// Unique task identifier
    pub task_id: TaskId,
    /// Task creation timestamp
    pub created_at: Instant,
    /// Task completion timestamp (if completed)
    pub completed_at: Option<Instant>,
    /// Total bytes allocated by this task
    pub total_allocated: u64,
    /// Current memory usage (allocated - deallocated)
    pub current_usage: u64,
    /// Peak memory usage observed
    pub peak_usage: u64,
    /// Number of allocation operations
    pub allocation_count: u64,
    /// Number of deallocation operations
    pub deallocation_count: u64,
    /// Average allocation size
    pub average_allocation_size: f64,
}

impl TaskMemoryProfile {
    /// Create new task profile
    pub fn new(task_id: TaskId) -> Self {
        Self {
            task_id,
            created_at: Instant::now(),
            completed_at: None,
            total_allocated: 0,
            current_usage: 0,
            peak_usage: 0,
            allocation_count: 0,
            deallocation_count: 0,
            average_allocation_size: 0.0,
        }
    }

    /// Record allocation event
    pub fn record_allocation(&mut self, size: u64) {
        self.total_allocated += size;
        self.current_usage += size;
        self.allocation_count += 1;

        // Update peak usage
        if self.current_usage > self.peak_usage {
            self.peak_usage = self.current_usage;
        }

        // Update average allocation size
        self.average_allocation_size = self.total_allocated as f64 / self.allocation_count as f64;
    }

    /// Record deallocation event
    pub fn record_deallocation(&mut self, size: u64) {
        self.current_usage = self.current_usage.saturating_sub(size);
        self.deallocation_count += 1;
    }

    /// Mark task as completed
    pub fn mark_completed(&mut self) {
        self.completed_at = Some(Instant::now());
    }

    /// Check if task is completed
    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }

    /// Get task lifetime duration
    pub fn lifetime(&self) -> Duration {
        let end = self.completed_at.unwrap_or_else(Instant::now);
        end.duration_since(self.created_at)
    }

    /// Calculate memory efficiency (deallocated / allocated)
    pub fn memory_efficiency(&self) -> f64 {
        if self.total_allocated == 0 {
            1.0
        } else {
            let deallocated = self.total_allocated - self.current_usage;
            deallocated as f64 / self.total_allocated as f64
        }
    }

    /// Check if task has potential memory leak
    pub fn has_potential_leak(&self) -> bool {
        self.is_completed() && self.current_usage > 0
    }
}

/// Performance metrics for async task execution
#[derive(Debug, Clone)]
pub struct TaskPerformanceMetrics {
    /// Task execution time
    pub execution_time: Duration,
    /// Memory allocation rate (bytes/second)
    pub allocation_rate: f64,
    /// Memory efficiency ratio
    pub efficiency_ratio: f64,
    /// Peak memory usage
    pub peak_memory_mb: f64,
    /// Average allocation size
    pub avg_allocation_size: f64,
}

impl TaskPerformanceMetrics {
    /// Create metrics from task profile
    pub fn from_profile(profile: &TaskMemoryProfile) -> Self {
        let lifetime = profile.lifetime();
        let lifetime_secs = lifetime.as_secs_f64();
        
        let allocation_rate = if lifetime_secs > 0.0 {
            profile.total_allocated as f64 / lifetime_secs
        } else {
            0.0
        };

        Self {
            execution_time: lifetime,
            allocation_rate,
            efficiency_ratio: profile.memory_efficiency(),
            peak_memory_mb: profile.peak_usage as f64 / 1_048_576.0, // Convert to MB
            avg_allocation_size: profile.average_allocation_size,
        }
    }

    /// Get performance rating (0.0 to 1.0, higher is better)
    pub fn performance_rating(&self) -> f64 {
        // Combine multiple factors for overall rating
        let efficiency_score = self.efficiency_ratio;
        let speed_score = if self.allocation_rate < 1_000_000.0 { 1.0 } else { 0.5 }; // Prefer < 1MB/s
        let memory_score = if self.peak_memory_mb < 10.0 { 1.0 } else { 0.7 }; // Prefer < 10MB peak
        
        (efficiency_score + speed_score + memory_score) / 3.0
    }
}

/// Aggregated statistics across multiple tasks
#[derive(Debug, Clone)]
pub struct AggregatedTaskStats {
    /// Total number of tasks tracked
    pub total_tasks: usize,
    /// Number of completed tasks
    pub completed_tasks: usize,
    /// Total memory allocated across all tasks
    pub total_memory_allocated: u64,
    /// Current memory usage across all active tasks
    pub current_memory_usage: u64,
    /// Peak memory usage observed
    pub peak_memory_usage: u64,
    /// Average task lifetime
    pub average_lifetime: Duration,
    /// Memory efficiency across all tasks
    pub overall_efficiency: f64,
    /// Tasks with potential memory leaks
    pub potential_leaks: usize,
}

impl AggregatedTaskStats {
    /// Create empty statistics
    pub fn new() -> Self {
        Self {
            total_tasks: 0,
            completed_tasks: 0,
            total_memory_allocated: 0,
            current_memory_usage: 0,
            peak_memory_usage: 0,
            average_lifetime: Duration::ZERO,
            overall_efficiency: 1.0,
            potential_leaks: 0,
        }
    }

    /// Add task profile to aggregated statistics
    pub fn add_task(&mut self, profile: &TaskMemoryProfile) {
        self.total_tasks += 1;
        
        if profile.is_completed() {
            self.completed_tasks += 1;
        }
        
        self.total_memory_allocated += profile.total_allocated;
        self.current_memory_usage += profile.current_usage;
        
        if profile.peak_usage > self.peak_memory_usage {
            self.peak_memory_usage = profile.peak_usage;
        }
        
        if profile.has_potential_leak() {
            self.potential_leaks += 1;
        }
        
        // Recalculate averages (simplified)
        if self.completed_tasks > 0 {
            self.overall_efficiency = if self.total_memory_allocated > 0 {
                let total_deallocated = self.total_memory_allocated - self.current_memory_usage;
                total_deallocated as f64 / self.total_memory_allocated as f64
            } else {
                1.0
            };
        }
    }

    /// Get memory usage summary
    pub fn memory_summary(&self) -> String {
        format!(
            "Tasks: {} ({}% complete), Memory: {:.1}MB allocated, {:.1}MB current, {:.1}% efficiency",
            self.total_tasks,
            if self.total_tasks > 0 { self.completed_tasks * 100 / self.total_tasks } else { 0 },
            self.total_memory_allocated as f64 / 1_048_576.0,
            self.current_memory_usage as f64 / 1_048_576.0,
            self.overall_efficiency * 100.0
        )
    }
}

impl Default for AggregatedTaskStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_task_memory_profile_basic() {
        let mut profile = TaskMemoryProfile::new(12345);
        
        assert_eq!(profile.task_id, 12345);
        assert!(!profile.is_completed());
        assert_eq!(profile.current_usage, 0);
        assert_eq!(profile.total_allocated, 0);
        
        // Record some allocations
        profile.record_allocation(1024);
        assert_eq!(profile.current_usage, 1024);
        assert_eq!(profile.total_allocated, 1024);
        assert_eq!(profile.peak_usage, 1024);
        assert_eq!(profile.allocation_count, 1);
        
        profile.record_allocation(2048);
        assert_eq!(profile.current_usage, 3072);
        assert_eq!(profile.total_allocated, 3072);
        assert_eq!(profile.peak_usage, 3072);
        assert_eq!(profile.allocation_count, 2);
        
        // Record deallocation
        profile.record_deallocation(1024);
        assert_eq!(profile.current_usage, 2048);
        assert_eq!(profile.total_allocated, 3072); // Total doesn't decrease
        assert_eq!(profile.peak_usage, 3072); // Peak doesn't decrease
        assert_eq!(profile.deallocation_count, 1);
    }

    #[test]
    fn test_memory_efficiency_calculation() {
        let mut profile = TaskMemoryProfile::new(1);
        
        // Perfect efficiency (all deallocated)
        profile.record_allocation(1000);
        profile.record_deallocation(1000);
        assert_eq!(profile.memory_efficiency(), 1.0);
        
        // Partial efficiency
        profile.record_allocation(1000); // Total now 2000, current 1000
        assert_eq!(profile.memory_efficiency(), 0.5);
        
        // Zero efficiency (nothing deallocated)
        let mut profile2 = TaskMemoryProfile::new(2);
        profile2.record_allocation(1000);
        assert_eq!(profile2.memory_efficiency(), 0.0);
    }

    #[test]
    fn test_memory_leak_detection() {
        let mut profile = TaskMemoryProfile::new(1);
        
        // No leak if not completed
        profile.record_allocation(1000);
        assert!(!profile.has_potential_leak());
        
        // No leak if completed with no current usage
        profile.record_deallocation(1000);
        profile.mark_completed();
        assert!(!profile.has_potential_leak());
        
        // Potential leak if completed with current usage
        let mut profile2 = TaskMemoryProfile::new(2);
        profile2.record_allocation(1000);
        profile2.mark_completed();
        assert!(profile2.has_potential_leak());
    }

    #[test]
    fn test_task_lifetime() {
        let profile = TaskMemoryProfile::new(1);
        thread::sleep(Duration::from_millis(10));
        
        let lifetime = profile.lifetime();
        assert!(lifetime >= Duration::from_millis(10));
        assert!(lifetime < Duration::from_millis(100)); // Should be reasonable
    }

    #[test]
    fn test_performance_metrics() {
        let mut profile = TaskMemoryProfile::new(1);
        
        // Add some allocations
        profile.record_allocation(1_000_000); // 1MB
        profile.record_allocation(500_000);   // 500KB
        profile.record_deallocation(500_000); // Deallocate 500KB
        
        thread::sleep(Duration::from_millis(10));
        profile.mark_completed();
        
        let metrics = TaskPerformanceMetrics::from_profile(&profile);
        
        assert!(metrics.execution_time >= Duration::from_millis(10));
        assert!(metrics.allocation_rate > 0.0);
        assert_eq!(metrics.efficiency_ratio, profile.memory_efficiency());
        assert_eq!(metrics.peak_memory_mb, 1.5); // 1.5MB peak
        
        let rating = metrics.performance_rating();
        assert!(rating >= 0.0 && rating <= 1.0);
    }

    #[test]
    fn test_aggregated_stats() {
        let mut stats = AggregatedTaskStats::new();
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.completed_tasks, 0);
        
        // Add completed task
        let mut profile1 = TaskMemoryProfile::new(1);
        profile1.record_allocation(1000);
        profile1.record_deallocation(500);
        profile1.mark_completed();
        stats.add_task(&profile1);
        
        assert_eq!(stats.total_tasks, 1);
        assert_eq!(stats.completed_tasks, 1);
        assert_eq!(stats.total_memory_allocated, 1000);
        assert_eq!(stats.current_memory_usage, 500);
        assert_eq!(stats.overall_efficiency, 0.5);
        
        // Add active task
        let mut profile2 = TaskMemoryProfile::new(2);
        profile2.record_allocation(2000);
        stats.add_task(&profile2);
        
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.completed_tasks, 1);
        assert_eq!(stats.total_memory_allocated, 3000);
        assert_eq!(stats.current_memory_usage, 2500);
        
        // Test summary string
        let summary = stats.memory_summary();
        assert!(summary.contains("Tasks: 2"));
        assert!(summary.contains("50% complete"));
    }

    #[test]
    fn test_average_allocation_size() {
        let mut profile = TaskMemoryProfile::new(1);
        
        profile.record_allocation(100);
        assert_eq!(profile.average_allocation_size, 100.0);
        
        profile.record_allocation(200);
        assert_eq!(profile.average_allocation_size, 150.0); // (100 + 200) / 2
        
        profile.record_allocation(300);
        assert_eq!(profile.average_allocation_size, 200.0); // (100 + 200 + 300) / 3
    }
}