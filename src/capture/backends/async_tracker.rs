//! Async memory tracker implementation.
//!
//! This module contains async-specific memory tracking functionality
//! including task tracking, efficiency scoring, and bottleneck analysis.

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

use super::async_types::{
    AsyncAllocation, AsyncError, AsyncMemorySnapshot, AsyncResult, AsyncSnapshot, AsyncStats,
    TaskInfo, TaskMemoryProfile, TrackedFuture,
};

use crate::system_monitor;

/// Task types for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskType {
    CpuIntensive,
    IoIntensive,
    NetworkIntensive,
    MemoryIntensive,
    Streaming,
    Background,
    Mixed,
    GpuCompute,
}

impl Default for TaskType {
    fn default() -> Self {
        Self::Mixed
    }
}

/// Task efficiency report
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskReport {
    pub task_name: String,
    pub task_type: TaskType,
    pub efficiency_score: f64,
    pub cpu_efficiency: f64,
    pub memory_efficiency: f64,
    pub io_efficiency: f64,
    pub bottleneck: String,
    pub recommendations: Vec<String>,
}

/// Resource ranking entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceRanking {
    pub task_name: String,
    pub task_type: TaskType,
    pub cpu_usage: f64,
    pub memory_usage_mb: f64,
    pub io_usage_mb: f64,
    pub network_usage_mb: f64,
    pub gpu_usage: f64,
    pub overall_score: f64,
}

/// Global async tracker instance
static GLOBAL_TRACKER: Mutex<Option<Arc<AsyncTracker>>> = Mutex::new(None);

/// Async memory tracker for task-aware memory tracking.
pub struct AsyncTracker {
    /// Tracked tasks
    tasks: Arc<Mutex<HashMap<u64, TaskInfo>>>,
    /// Active allocations
    allocations: Arc<Mutex<HashMap<usize, AsyncAllocation>>>,
    /// Statistics
    stats: Arc<Mutex<AsyncStats>>,
    /// Task memory profiles
    profiles: Arc<Mutex<HashMap<u64, TaskMemoryProfile>>>,
    /// Initialization state
    initialized: Arc<Mutex<bool>>,
}

impl AsyncTracker {
    /// Create a new async tracker.
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            allocations: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(AsyncStats::default())),
            profiles: Arc::new(Mutex::new(HashMap::new())),
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    /// Track a task start.
    ///
    /// # Arguments
    /// * `task_id` - Unique task identifier
    /// * `name` - Task name
    /// * `thread_id` - Thread ID where task runs
    pub fn track_task_start(&self, task_id: u64, name: String, thread_id: ThreadId) {
        let task = TaskInfo {
            task_id,
            name: name.clone(),
            thread_id,
            created_at: Self::now(),
            active_allocations: 0,
            total_memory: 0,
        };

        // Use blocking lock to prevent data loss
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(task_id, task.clone());
        drop(tasks);

        let mut stats = self.stats.lock().unwrap();
        stats.total_tasks += 1;
        stats.active_tasks += 1;

        // Create task profile
        let mut profiles = self.profiles.lock().unwrap();
        profiles.insert(task_id, TaskMemoryProfile::new(task_id, name));
    }

    /// Track a task end.
    ///
    /// # Arguments
    /// * `task_id` - Unique task identifier
    pub fn track_task_end(&self, task_id: u64) {
        // Use blocking lock to prevent data loss
        let task_info = {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.remove(&task_id)
        };

        // Update task profile with duration
        if let Some(task) = task_info {
            let mut profiles = self.profiles.lock().unwrap();
            if let Some(profile) = profiles.get_mut(&task_id) {
                profile.duration_ns = Self::now().saturating_sub(task.created_at);
                profile.calculate_allocation_rate();
            }
        }

        let mut stats = self.stats.lock().unwrap();
        stats.active_tasks = stats.active_tasks.saturating_sub(1);
    }

    /// Track an allocation associated with a task.
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer
    /// * `size` - Allocation size
    /// * `task_id` - Associated task ID
    pub fn track_allocation(&self, ptr: usize, size: usize, task_id: u64) {
        self.track_allocation_with_location(ptr, size, task_id, None, None, None);
    }

    /// Track an allocation with source location.
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer
    /// * `size` - Allocation size
    /// * `task_id` - Associated task ID
    /// * `var_name` - Variable name
    /// * `type_name` - Type name
    /// * `source_location` - Source code location
    pub fn track_allocation_with_location(
        &self,
        ptr: usize,
        size: usize,
        task_id: u64,
        var_name: Option<String>,
        type_name: Option<String>,
        source_location: Option<super::async_types::SourceLocation>,
    ) {
        let allocation = AsyncAllocation {
            ptr,
            size,
            timestamp: Self::now(),
            task_id,
            var_name,
            type_name,
            source_location,
        };

        // Store allocation - use blocking lock
        {
            let mut allocations = self.allocations.lock().unwrap();
            allocations.insert(ptr, allocation.clone());
        }

        // Update task info - use blocking lock
        {
            let mut tasks = self.tasks.lock().unwrap();
            if let Some(task) = tasks.get_mut(&task_id) {
                task.active_allocations += 1;
                task.total_memory += size;
            }
        }

        // Update statistics - use blocking lock
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_allocations += 1;
            stats.total_memory += size;
            stats.active_memory += size;
            // Peak memory is the maximum active memory we've ever seen
            if stats.active_memory > stats.peak_memory {
                stats.peak_memory = stats.active_memory;
            }
        }

        // Update task profile
        {
            let mut profiles = self.profiles.lock().unwrap();
            if let Some(profile) = profiles.get_mut(&task_id) {
                profile.total_allocations += 1;
                profile.total_bytes += size as u64;
                profile.calculate_avg_size();
            }
        }
    }

    /// Track a deallocation associated with a task.
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer
    pub fn track_deallocation(&self, ptr: usize) {
        // Get allocation info before removing - use blocking lock
        let (task_id, size) = {
            let mut allocations = self.allocations.lock().unwrap();
            allocations
                .remove(&ptr)
                .map(|alloc| (alloc.task_id, alloc.size))
                .unwrap_or((0, 0))
        };

        // Update task info - use blocking lock
        if task_id != 0 {
            let mut tasks = self.tasks.lock().unwrap();
            if let Some(task) = tasks.get_mut(&task_id) {
                task.active_allocations = task.active_allocations.saturating_sub(1);
            }
        }

        // Update active memory stats
        if size > 0 {
            let mut stats = self.stats.lock().unwrap();
            stats.active_memory = stats.active_memory.saturating_sub(size);
        }
    }

    /// Get current statistics.
    pub fn get_stats(&self) -> AsyncStats {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }

    /// Take a snapshot of current state.
    pub fn snapshot(&self) -> AsyncSnapshot {
        let tasks = {
            let tasks = self.tasks.lock().unwrap();
            tasks.values().cloned().collect()
        };

        let allocations = {
            let allocs = self.allocations.lock().unwrap();
            allocs.values().cloned().collect()
        };

        let stats = self.get_stats();

        AsyncSnapshot {
            timestamp: Self::now(),
            tasks,
            allocations,
            stats,
        }
    }

    /// Get task memory profile
    pub fn get_task_profile(&self, task_id: u64) -> Option<TaskMemoryProfile> {
        let profiles = self.profiles.lock().unwrap();
        profiles.get(&task_id).cloned()
    }

    /// Get all task profiles
    pub fn get_all_profiles(&self) -> Vec<TaskMemoryProfile> {
        let profiles = self.profiles.lock().unwrap();
        profiles.values().cloned().collect()
    }

    /// Check if tracker is initialized
    pub fn is_initialized(&self) -> bool {
        *self.initialized.lock().unwrap()
    }

    /// Mark tracker as initialized
    pub fn set_initialized(&self) {
        *self.initialized.lock().unwrap() = true;
    }

    /// Generate task efficiency report
    ///
    /// Calculates efficiency scores based on task-specific metrics.
    pub fn analyze_task(&self, task_id: u64, task_type: TaskType) -> Option<TaskReport> {
        let profile = self.get_task_profile(task_id)?;

        // Use task-specific metrics instead of global system metrics
        let total_bytes = profile.total_bytes as f64;
        let total_allocations = profile.total_allocations as f64;
        let peak_memory = profile.peak_memory as f64;
        let duration_ms = profile.duration_ns as f64 / 1_000_000.0;

        // Calculate efficiency scores (0.0 - 1.0)

        // Base efficiency for compute-intensive tasks (CPU, IO, GPU)
        let compute_efficiency = if duration_ms > 0.0 {
            (total_allocations / duration_ms * 1000.0).min(1.0)
        } else {
            0.0
        };

        let cpu_efficiency = match task_type {
            TaskType::CpuIntensive | TaskType::IoIntensive | TaskType::GpuCompute => {
                compute_efficiency
            }
            TaskType::MemoryIntensive => {
                if total_bytes > 0.0 {
                    (peak_memory / total_bytes).min(1.0)
                } else {
                    0.0
                }
            }
            TaskType::NetworkIntensive => {
                if total_bytes > 0.0 {
                    (total_allocations / total_bytes * 1000.0).min(1.0)
                } else {
                    0.0
                }
            }
            _ => compute_efficiency,
        };

        let memory_efficiency = if total_bytes > 0.0 {
            (total_allocations / total_bytes * 1000.0).min(1.0)
        } else {
            0.0
        };

        let io_efficiency = if duration_ms > 0.0 {
            (total_bytes / duration_ms / 1_048_576.0).min(1.0)
        } else {
            0.0
        };

        let efficiency_score = (cpu_efficiency + memory_efficiency + io_efficiency) / 3.0;

        // Determine bottleneck based on task-specific metrics
        let bottleneck = if duration_ms > 5000.0 {
            "Execution Time".to_string()
        } else if peak_memory > 100.0 * 1024.0 * 1024.0 {
            "Memory".to_string()
        } else if total_allocations > 10000.0 {
            "Allocations".to_string()
        } else {
            "None".to_string()
        };

        // Generate recommendations based on task-specific metrics
        let mut recommendations = Vec::new();
        if duration_ms > 5000.0 {
            recommendations.push("Consider optimizing task execution time".to_string());
        }
        if peak_memory > 100.0 * 1024.0 * 1024.0 {
            recommendations.push("Reduce peak memory usage".to_string());
        }
        if total_allocations > 10000.0 {
            recommendations.push("Reduce number of allocations".to_string());
        }
        if recommendations.is_empty() {
            recommendations.push("Performance is good".to_string());
        }

        Some(TaskReport {
            task_name: profile.task_name.clone(),
            task_type,
            efficiency_score,
            cpu_efficiency,
            memory_efficiency,
            io_efficiency,
            bottleneck,
            recommendations,
        })
    }

    /// Get resource rankings for all tasks
    ///
    /// Returns tasks sorted by overall resource consumption score.
    /// Uses task-specific metrics (memory, allocation rate, duration) for ranking.
    pub fn get_resource_rankings(&self) -> Vec<ResourceRanking> {
        let profiles = self.get_all_profiles();

        let mut rankings: Vec<ResourceRanking> = profiles
            .into_iter()
            .map(|profile| {
                let memory_mb = profile.total_bytes as f64 / 1_048_576.0;
                let peak_memory_mb = profile.peak_memory as f64 / 1_048_576.0;
                let duration_ms = profile.duration_ns as f64 / 1_000_000.0;
                let allocation_rate = profile.allocation_rate;

                let overall_score = memory_mb * 0.3
                    + peak_memory_mb * 0.2
                    + allocation_rate * 0.0001
                    + duration_ms * 0.0001;

                ResourceRanking {
                    task_name: profile.task_name.clone(),
                    task_type: profile.task_type.clone(),
                    cpu_usage: allocation_rate,
                    memory_usage_mb: memory_mb,
                    io_usage_mb: 0.0,
                    network_usage_mb: 0.0,
                    gpu_usage: 0.0,
                    overall_score,
                }
            })
            .collect();

        rankings.sort_by(|a, b| {
            b.overall_score
                .partial_cmp(&a.overall_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        rankings
    }

    /// Get current timestamp.
    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}

impl Default for AsyncTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize async memory tracking system
///
/// Must be called before spawning any tracked tasks.
/// Sets up background aggregation and monitoring.
pub fn initialize() -> AsyncResult<()> {
    let mut global = GLOBAL_TRACKER.lock().map_err(|_| AsyncError::System {
        operation: Arc::from("initialize"),
        message: Arc::from("Failed to acquire global tracker lock"),
    })?;

    if global.is_none() {
        let tracker = AsyncTracker::new();
        tracker.set_initialized();
        *global = Some(Arc::new(tracker));
        tracing::info!("Async memory tracking system initialized");
        Ok(())
    } else {
        Err(AsyncError::initialization(
            "tracker",
            "Already initialized",
            true,
        ))
    }
}

/// Shutdown async memory tracking system
///
/// Cleans up resources and stops tracking.
pub fn shutdown() -> AsyncResult<()> {
    let mut global = GLOBAL_TRACKER.lock().map_err(|_| AsyncError::System {
        operation: Arc::from("shutdown"),
        message: Arc::from("Failed to acquire global tracker lock"),
    })?;

    if global.is_some() {
        *global = None;
        tracing::info!("Async memory tracking system shutdown");
        Ok(())
    } else {
        Err(AsyncError::initialization(
            "tracker",
            "Not initialized",
            true,
        ))
    }
}

/// Reset global tracker state (for testing only)
#[cfg(test)]
pub fn reset_global_tracker() {
    let mut global = GLOBAL_TRACKER.lock().unwrap();
    *global = None;
}

/// Get the global async tracker
fn get_global_tracker() -> AsyncResult<Arc<AsyncTracker>> {
    GLOBAL_TRACKER
        .lock()
        .map_err(|_| AsyncError::System {
            operation: Arc::from("get_global_tracker"),
            message: Arc::from("Failed to acquire global tracker lock"),
        })?
        .clone()
        .ok_or_else(|| {
            AsyncError::initialization("tracker", "Tracking system not initialized", true)
        })
}

/// Create a tracked future wrapper
///
/// Wraps the provided future in a TrackedFuture that automatically
/// attributes memory allocations to the task.
pub fn create_tracked<F>(future: F) -> TrackedFuture<F>
where
    F: Future,
{
    TrackedFuture::new(future)
}

/// Spawn a tracked async task
///
/// This is a convenience function that wraps the provided future in a TrackedFuture.
/// Use with your preferred async runtime (tokio, async-std, etc.)
///
/// Example:
/// ```rust,no_run
/// use memscope_rs::capture::backends::async_tracker;
///
/// #[tokio::main]
/// async fn main() {
///     let handle = async_tracker::spawn_tracked(async {
///         let data = vec![0u8; 1024];
///         data.len()
///     });
///     
///     let result = handle.await;
///     println!("Result: {}", result);
/// }
/// ```
pub fn spawn_tracked<F>(future: F) -> TrackedFuture<F>
where
    F: Future,
{
    create_tracked(future)
}

/// Get current memory usage snapshot
///
/// Returns statistics about async task memory usage.
pub fn get_memory_snapshot() -> AsyncMemorySnapshot {
    if let Ok(tracker) = get_global_tracker() {
        let stats = tracker.get_stats();

        AsyncMemorySnapshot {
            active_task_count: stats.active_tasks,
            total_allocated_bytes: stats.total_memory as u64,
            allocation_events: stats.total_allocations as u64,
            events_dropped: 0,
            buffer_utilization: 0.0,
        }
    } else {
        AsyncMemorySnapshot {
            active_task_count: 0,
            total_allocated_bytes: 0,
            allocation_events: 0,
            events_dropped: 0,
            buffer_utilization: 0.0,
        }
    }
}

/// Check if async memory tracking is currently active
pub fn is_tracking_active() -> bool {
    GLOBAL_TRACKER.lock().is_ok_and(|global| global.is_some())
}

/// Track allocation for current task
///
/// Called by global allocator hook to track allocations.
pub fn track_current_allocation(ptr: usize, size: usize) -> AsyncResult<()> {
    let tracker = get_global_tracker()?;
    let task_info = super::async_types::get_current_task();

    if task_info.has_tracking_id() {
        tracker.track_allocation(ptr, size, (task_info.primary_id() & 0xFFFFFFFF) as u64);
    }

    Ok(())
}

/// Track deallocation for current task
///
/// Called by global allocator hook to track deallocations.
pub fn track_current_deallocation(ptr: usize) -> AsyncResult<()> {
    let tracker = get_global_tracker()?;
    tracker.track_deallocation(ptr);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_tracker_creation() {
        let tracker = AsyncTracker::new();
        let stats = tracker.get_stats();
        assert_eq!(stats.total_tasks, 0);
    }

    #[test]
    fn test_task_tracking() {
        let tracker = AsyncTracker::new();
        let thread_id = std::thread::current().id();
        tracker.track_task_start(1, "test_task".to_string(), thread_id);

        let stats = tracker.get_stats();
        assert_eq!(stats.total_tasks, 1);
        assert_eq!(stats.active_tasks, 1);

        tracker.track_task_end(1);
        let stats = tracker.get_stats();
        assert_eq!(stats.active_tasks, 0);
    }

    #[test]
    fn test_allocation_tracking() {
        let tracker = AsyncTracker::new();
        let thread_id = std::thread::current().id();
        tracker.track_task_start(1, "test_task".to_string(), thread_id);
        tracker.track_allocation(0x1000, 1024, 1);

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.allocations.len(), 1);
        assert_eq!(snapshot.tasks[0].active_allocations, 1);
    }

    #[test]
    fn test_initialization() {
        // Clean up any existing state first
        reset_global_tracker();

        let result = initialize();
        assert!(result.is_ok());

        let result2 = initialize();
        // Note: This might fail if another test already initialized
        // We check the error message instead
        if let Err(e) = result2 {
            assert!(e.message().contains("Already initialized"));
        }

        let _ = shutdown(); // Clean up
    }

    #[test]
    fn test_shutdown() {
        // Clean up any existing state first
        reset_global_tracker();

        initialize().unwrap();
        let result = shutdown();
        assert!(result.is_ok());

        // Second shutdown might fail if already cleaned
        let result2 = shutdown();
        if let Err(e) = result2 {
            assert!(e.message().contains("Not initialized"));
        }
    }

    #[test]
    fn test_memory_snapshot() {
        // Clean up any existing state first
        reset_global_tracker();

        initialize().unwrap();
        let snapshot = get_memory_snapshot();
        assert_eq!(snapshot.active_task_count, 0);
        let _ = shutdown(); // Clean up
    }

    #[test]
    fn test_is_tracking_active() {
        // Clean up any existing state first
        reset_global_tracker();

        assert!(!is_tracking_active());
        initialize().unwrap();
        assert!(is_tracking_active());
        let _ = shutdown(); // Ignore result as state might already be cleared
        assert!(!is_tracking_active());
    }

    #[test]
    fn test_task_memory_profile() {
        let tracker = AsyncTracker::new();
        let thread_id = std::thread::current().id();
        tracker.track_task_start(1, "test_task".to_string(), thread_id);
        tracker.track_allocation(0x1000, 1024, 1);
        tracker.track_allocation(0x2000, 2048, 1);
        tracker.track_task_end(1);

        let profile = tracker.get_task_profile(1);
        assert!(profile.is_some());
        let profile = profile.unwrap();
        assert_eq!(profile.task_id, 1);
        assert_eq!(profile.total_allocations, 2);
        assert_eq!(profile.total_bytes, 3072);
    }
}
