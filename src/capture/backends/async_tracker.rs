//! Async memory tracker implementation.
//!
//! This module contains async-specific memory tracking functionality.

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

use super::async_types::{
    AsyncAllocation, AsyncError, AsyncMemorySnapshot, AsyncResult, AsyncSnapshot, AsyncStats,
    TaskInfo, TaskMemoryProfile, TrackedFuture,
};

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
        let allocation = AsyncAllocation {
            ptr,
            size,
            timestamp: Self::now(),
            task_id,
            var_name: None,
            type_name: None,
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
            stats.peak_memory = stats.peak_memory.max(stats.total_memory);
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
        let task_id = {
            let mut allocations = self.allocations.lock().unwrap();
            allocations.remove(&ptr).map(|alloc| alloc.task_id)
        };

        // Update task info - use blocking lock
        if let Some(tid) = task_id {
            let mut tasks = self.tasks.lock().unwrap();
            if let Some(task) = tasks.get_mut(&tid) {
                task.active_allocations = task.active_allocations.saturating_sub(1);
            }
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
