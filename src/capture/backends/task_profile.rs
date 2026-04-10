//! Task-level memory profiling for the unified tracker
//!
//! This module provides task-aware memory tracking capabilities,
//! allowing users to track memory usage patterns at the granularity
//! of individual tasks or workloads.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Task type classification for categorizing different workloads
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TaskType {
    /// CPU-intensive tasks (e.g., matrix multiplication, data processing)
    CpuIntensive,
    /// IO-intensive tasks (e.g., file operations, database queries)
    IoIntensive,
    /// Network-intensive tasks (e.g., HTTP requests, RPC calls)
    NetworkIntensive,
    /// Memory-intensive tasks (e.g., large data structures, caching)
    MemoryIntensive,
    /// GPU compute tasks (e.g., CUDA, OpenCL operations)
    GpuCompute,
    /// Mixed workload with balanced resource usage
    #[default]
    Mixed,
    /// Streaming data processing tasks
    Streaming,
    /// Background maintenance tasks
    Background,
}

impl TaskType {
    /// Get human-readable description of task type
    pub fn description(&self) -> &'static str {
        match self {
            Self::CpuIntensive => "CPU-intensive workload",
            Self::IoIntensive => "IO-intensive workload",
            Self::NetworkIntensive => "Network-intensive workload",
            Self::MemoryIntensive => "Memory-intensive workload",
            Self::GpuCompute => "GPU compute workload",
            Self::Mixed => "Mixed workload",
            Self::Streaming => "Streaming workload",
            Self::Background => "Background task",
        }
    }

    /// Get resource priority for this task type
    pub fn resource_priority(&self) -> (f64, f64, f64, f64) {
        match self {
            Self::CpuIntensive => (1.0, 0.3, 0.2, 0.1),
            Self::IoIntensive => (0.3, 1.0, 0.2, 0.1),
            Self::NetworkIntensive => (0.3, 0.2, 1.0, 0.1),
            Self::MemoryIntensive => (0.2, 0.3, 0.2, 1.0),
            Self::GpuCompute => (0.5, 0.2, 0.1, 0.8),
            Self::Mixed => (0.5, 0.5, 0.5, 0.5),
            Self::Streaming => (0.4, 0.4, 0.6, 0.4),
            Self::Background => (0.2, 0.2, 0.2, 0.2),
        }
    }
}

/// Memory usage profile for a single task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMemoryProfile {
    /// Unique task identifier (auto-generated, never reused)
    pub task_id: u64,
    /// Tokio task ID (if running in tokio context)
    pub tokio_task_id: Option<u64>,
    /// Task name for identification
    pub task_name: String,
    /// Task type classification
    pub task_type: TaskType,
    /// Task creation timestamp (milliseconds since Unix epoch)
    pub created_at_ms: u64,
    /// Task completion timestamp (milliseconds since Unix epoch, if completed)
    pub completed_at_ms: Option<u64>,
    /// Total bytes allocated by this task
    pub total_bytes: u64,
    /// Current memory usage (allocated - deallocated)
    pub current_memory: u64,
    /// Peak memory usage observed
    pub peak_memory: u64,
    /// Number of allocation operations
    pub total_allocations: u64,
    /// Number of deallocation operations
    pub total_deallocations: u64,
    /// Task duration in nanoseconds
    pub duration_ns: u64,
    /// Memory allocation rate (bytes/second)
    pub allocation_rate: f64,
    /// Memory efficiency score (0.0 to 1.0)
    pub efficiency_score: f64,
    /// Average allocation size
    pub average_allocation_size: f64,
}

impl TaskMemoryProfile {
    /// Create new task profile
    pub fn new(task_id: u64, task_name: String, task_type: TaskType) -> Self {
        Self {
            task_id,
            tokio_task_id: None,
            task_name,
            task_type,
            created_at_ms: Self::now_ms(),
            completed_at_ms: None,
            total_bytes: 0,
            current_memory: 0,
            peak_memory: 0,
            total_allocations: 0,
            total_deallocations: 0,
            duration_ns: 0,
            allocation_rate: 0.0,
            efficiency_score: 1.0,
            average_allocation_size: 0.0,
        }
    }

    /// Create new task profile with tokio task ID
    pub fn with_tokio_id(
        task_id: u64,
        tokio_task_id: u64,
        task_name: String,
        task_type: TaskType,
    ) -> Self {
        let mut profile = Self::new(task_id, task_name, task_type);
        profile.tokio_task_id = Some(tokio_task_id);
        profile
    }

    /// Get current timestamp in milliseconds
    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Mark task as completed
    pub fn mark_completed(&mut self) {
        self.completed_at_ms = Some(Self::now_ms());
        self.duration_ns = self.lifetime().as_nanos() as u64;
        self.update_metrics();
    }

    /// Record allocation event
    pub fn record_allocation(&mut self, size: u64) {
        self.total_bytes += size;
        self.current_memory += size;
        self.total_allocations += 1;

        if self.current_memory > self.peak_memory {
            self.peak_memory = self.current_memory;
        }

        self.update_metrics();
    }

    /// Record deallocation event
    pub fn record_deallocation(&mut self, size: u64) {
        self.current_memory = self.current_memory.saturating_sub(size);
        self.total_deallocations += 1;
        self.update_metrics();
    }

    /// Check if task is completed
    pub fn is_completed(&self) -> bool {
        self.completed_at_ms.is_some()
    }

    /// Get task lifetime duration
    pub fn lifetime(&self) -> Duration {
        let end_ms = self.completed_at_ms.unwrap_or_else(Self::now_ms);
        let start_ms = self.created_at_ms;
        Duration::from_millis(end_ms.saturating_sub(start_ms))
    }

    /// Calculate memory efficiency (deallocated / allocated)
    pub fn memory_efficiency(&self) -> f64 {
        if self.total_bytes == 0 {
            1.0
        } else {
            let deallocated = self.total_bytes - self.current_memory;
            deallocated as f64 / self.total_bytes as f64
        }
    }

    /// Check if task has potential memory leak
    pub fn has_potential_leak(&self) -> bool {
        self.is_completed() && self.current_memory > 0
    }

    /// Update derived metrics
    fn update_metrics(&mut self) {
        let lifetime_secs = self.lifetime().as_secs_f64();

        self.allocation_rate = if lifetime_secs > 0.0 {
            self.total_bytes as f64 / lifetime_secs
        } else {
            0.0
        };

        self.efficiency_score = self.memory_efficiency();

        self.average_allocation_size = if self.total_allocations > 0 {
            self.total_bytes as f64 / self.total_allocations as f64
        } else {
            0.0
        };
    }

    /// Get memory usage summary
    pub fn summary(&self) -> String {
        format!(
            "Task '{}' (ID: {}, Type: {:?}): {} allocations, {:.2} MB total, {:.2} MB peak, {:.1}% efficiency",
            self.task_name,
            self.task_id,
            self.task_type,
            self.total_allocations,
            self.total_bytes as f64 / 1_048_576.0,
            self.peak_memory as f64 / 1_048_576.0,
            self.efficiency_score * 100.0
        )
    }
}

/// Manager for tracking multiple task profiles
#[derive(Debug, Clone)]
pub struct TaskProfileManager {
    profiles: Arc<Mutex<HashMap<u64, TaskMemoryProfile>>>,
    next_task_id: Arc<AtomicU64>,
}

impl TaskProfileManager {
    /// Create new task profile manager
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(Mutex::new(HashMap::new())),
            next_task_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Create a new task profile
    pub fn create_task(&self, task_name: String, task_type: TaskType) -> u64 {
        let task_id = self.next_task_id.fetch_add(1, Ordering::Relaxed);

        let profile = TaskMemoryProfile::new(task_id, task_name, task_type);

        if let Ok(mut profiles) = self.profiles.lock() {
            profiles.insert(task_id, profile);
        }

        task_id
    }

    /// Record allocation for a task
    pub fn record_allocation(&self, task_id: u64, size: u64) {
        if let Ok(mut profiles) = self.profiles.lock() {
            if let Some(profile) = profiles.get_mut(&task_id) {
                profile.record_allocation(size);
            }
        }
    }

    /// Record deallocation for a task
    pub fn record_deallocation(&self, task_id: u64, size: u64) {
        if let Ok(mut profiles) = self.profiles.lock() {
            if let Some(profile) = profiles.get_mut(&task_id) {
                profile.record_deallocation(size);
            }
        }
    }

    /// Mark task as completed
    pub fn complete_task(&self, task_id: u64) {
        if let Ok(mut profiles) = self.profiles.lock() {
            if let Some(profile) = profiles.get_mut(&task_id) {
                profile.mark_completed();
            }
        }
    }

    /// Get profile for a specific task
    pub fn get_profile(&self, task_id: u64) -> Option<TaskMemoryProfile> {
        if let Ok(profiles) = self.profiles.lock() {
            profiles.get(&task_id).cloned()
        } else {
            None
        }
    }

    /// Get all task profiles
    pub fn get_all_profiles(&self) -> Vec<TaskMemoryProfile> {
        if let Ok(profiles) = self.profiles.lock() {
            profiles.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get profiles by task type
    pub fn get_profiles_by_type(&self, task_type: TaskType) -> Vec<TaskMemoryProfile> {
        if let Ok(profiles) = self.profiles.lock() {
            profiles
                .values()
                .filter(|p| p.task_type == task_type)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get aggregated statistics
    pub fn get_aggregated_stats(&self) -> AggregatedTaskStats {
        let profiles = self.get_all_profiles();

        let total_tasks = profiles.len();
        let completed_tasks = profiles.iter().filter(|p| p.is_completed()).count();

        let total_memory_allocated: u64 = profiles.iter().map(|p| p.total_bytes).sum();
        let current_memory_usage: u64 = profiles.iter().map(|p| p.current_memory).sum();
        let peak_memory_usage: u64 = profiles.iter().map(|p| p.peak_memory).max().unwrap_or(0);

        let total_duration: Duration = profiles.iter().map(|p| p.lifetime()).sum::<Duration>();
        let average_lifetime = if total_tasks > 0 {
            let total_secs = total_duration.as_secs_f64();
            let avg_secs = total_secs / total_tasks as f64;
            Duration::from_secs_f64(avg_secs)
        } else {
            Duration::ZERO
        };

        let overall_efficiency = if total_memory_allocated > 0 {
            let total_deallocated = total_memory_allocated - current_memory_usage;
            total_deallocated as f64 / total_memory_allocated as f64
        } else {
            1.0
        };

        let potential_leaks = profiles.iter().filter(|p| p.has_potential_leak()).count();

        AggregatedTaskStats {
            total_tasks,
            completed_tasks,
            total_memory_allocated,
            current_memory_usage,
            peak_memory_usage,
            average_lifetime,
            overall_efficiency,
            potential_leaks,
        }
    }

    /// Clear all profiles
    pub fn clear(&self) {
        if let Ok(mut profiles) = self.profiles.lock() {
            profiles.clear();
        }
    }

    /// Get number of active tasks
    pub fn active_task_count(&self) -> usize {
        if let Ok(profiles) = self.profiles.lock() {
            profiles.iter().filter(|(_, p)| !p.is_completed()).count()
        } else {
            0
        }
    }
}

impl Default for TaskProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregated statistics across multiple tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Get memory usage summary
    pub fn memory_summary(&self) -> String {
        format!(
            "Tasks: {} ({}% complete), Memory: {:.2}MB allocated, {:.2}MB current, {:.1}% efficiency, {} potential leaks",
            self.total_tasks,
            if self.total_tasks > 0 {
                self.completed_tasks * 100 / self.total_tasks
            } else {
                0
            },
            self.total_memory_allocated as f64 / 1_048_576.0,
            self.current_memory_usage as f64 / 1_048_576.0,
            self.overall_efficiency * 100.0,
            self.potential_leaks
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

    #[test]
    fn test_task_type_description() {
        assert_eq!(
            TaskType::CpuIntensive.description(),
            "CPU-intensive workload"
        );
        assert_eq!(TaskType::IoIntensive.description(), "IO-intensive workload");
        assert_eq!(
            TaskType::NetworkIntensive.description(),
            "Network-intensive workload"
        );
    }

    #[test]
    fn test_task_type_resource_priority() {
        let (cpu, io, net, mem) = TaskType::CpuIntensive.resource_priority();
        assert!(cpu > io && cpu > net && cpu > mem);

        let (cpu, io, net, mem) = TaskType::IoIntensive.resource_priority();
        assert!(io > cpu && io > net && io > mem);
    }

    #[test]
    fn test_task_memory_profile_basic() {
        let profile = TaskMemoryProfile::new(1, "test_task".to_string(), TaskType::CpuIntensive);

        assert_eq!(profile.task_id, 1);
        assert_eq!(profile.task_name, "test_task");
        assert_eq!(profile.task_type, TaskType::CpuIntensive);
        assert!(!profile.is_completed());
        assert_eq!(profile.current_memory, 0);
        assert_eq!(profile.total_bytes, 0);
    }

    #[test]
    fn test_record_allocation() {
        let mut profile = TaskMemoryProfile::new(1, "test".to_string(), TaskType::Mixed);

        profile.record_allocation(1024);
        assert_eq!(profile.current_memory, 1024);
        assert_eq!(profile.total_bytes, 1024);
        assert_eq!(profile.peak_memory, 1024);
        assert_eq!(profile.total_allocations, 1);

        profile.record_allocation(2048);
        assert_eq!(profile.current_memory, 3072);
        assert_eq!(profile.total_bytes, 3072);
        assert_eq!(profile.peak_memory, 3072);
        assert_eq!(profile.total_allocations, 2);
    }

    #[test]
    fn test_record_deallocation() {
        let mut profile = TaskMemoryProfile::new(1, "test".to_string(), TaskType::Mixed);

        profile.record_allocation(3072);
        profile.record_deallocation(1024);
        assert_eq!(profile.current_memory, 2048);
        assert_eq!(profile.total_bytes, 3072);
        assert_eq!(profile.peak_memory, 3072);
        assert_eq!(profile.total_deallocations, 1);
    }

    #[test]
    fn test_memory_efficiency() {
        let mut profile = TaskMemoryProfile::new(1, "test".to_string(), TaskType::Mixed);

        profile.record_allocation(1000);
        profile.record_deallocation(1000);
        assert_eq!(profile.memory_efficiency(), 1.0);

        profile.record_allocation(1000);
        assert_eq!(profile.memory_efficiency(), 0.5);
    }

    #[test]
    fn test_potential_leak_detection() {
        let mut profile = TaskMemoryProfile::new(1, "test".to_string(), TaskType::Mixed);

        profile.record_allocation(1000);
        assert!(!profile.has_potential_leak());

        profile.record_deallocation(1000);
        profile.mark_completed();
        assert!(!profile.has_potential_leak());

        let mut profile2 = TaskMemoryProfile::new(2, "test2".to_string(), TaskType::Mixed);
        profile2.record_allocation(1000);
        profile2.mark_completed();
        assert!(profile2.has_potential_leak());
    }

    #[test]
    fn test_task_profile_manager() {
        let manager = TaskProfileManager::new();

        let task_id = manager.create_task("test_task".to_string(), TaskType::CpuIntensive);
        assert!(task_id > 0);

        manager.record_allocation(task_id, 1024);
        manager.record_allocation(task_id, 2048);

        let profile = manager.get_profile(task_id);
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().total_bytes, 3072);
    }

    #[test]
    fn test_aggregated_stats() {
        let manager = TaskProfileManager::new();

        let task1 = manager.create_task("task1".to_string(), TaskType::Mixed);
        manager.record_allocation(task1, 1000);
        manager.record_deallocation(task1, 500);
        manager.complete_task(task1);

        let task2 = manager.create_task("task2".to_string(), TaskType::Mixed);
        manager.record_allocation(task2, 2000);

        let stats = manager.get_aggregated_stats();
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.completed_tasks, 1);
        assert_eq!(stats.total_memory_allocated, 3000);
        assert_eq!(stats.current_memory_usage, 2500);
    }

    #[test]
    fn test_active_task_count() {
        let manager = TaskProfileManager::new();

        let task1 = manager.create_task("task1".to_string(), TaskType::Mixed);
        let task2 = manager.create_task("task2".to_string(), TaskType::Mixed);

        assert_eq!(manager.active_task_count(), 2);

        manager.complete_task(task1);
        assert_eq!(manager.active_task_count(), 1);

        manager.complete_task(task2);
        assert_eq!(manager.active_task_count(), 0);
    }

    #[test]
    fn test_profiles_by_type() {
        let manager = TaskProfileManager::new();

        let _ = manager.create_task("cpu_task".to_string(), TaskType::CpuIntensive);
        let _ = manager.create_task("io_task".to_string(), TaskType::IoIntensive);
        let _ = manager.create_task("cpu_task2".to_string(), TaskType::CpuIntensive);

        let cpu_profiles = manager.get_profiles_by_type(TaskType::CpuIntensive);
        assert_eq!(cpu_profiles.len(), 2);

        let io_profiles = manager.get_profiles_by_type(TaskType::IoIntensive);
        assert_eq!(io_profiles.len(), 1);
    }
}
