//! Async memory tracker implementation.
//!
//! This module contains async-specific memory tracking functionality
//! including task tracking, efficiency scoring, and bottleneck analysis.

use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

use super::async_types::{
    AsyncAllocation, AsyncError, AsyncMemorySnapshot, AsyncResult, AsyncSnapshot, AsyncStats,
    TrackedFuture,
};
use super::task_profile::{TaskMemoryProfile, TaskType};

/// Global task ID counter for unique task identification.
/// Tokio task IDs are recycled after task completion, so we need
/// our own counter to ensure unique identification across all tasks.
static TASK_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a new unique task ID.
/// This ID is never recycled, ensuring unique identification.
pub fn generate_unique_task_id() -> u64 {
    TASK_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Global thread ID counter for unique thread identification.
static THREAD_COUNTER: AtomicU64 = AtomicU64::new(1);

thread_local! {
    static THREAD_ID: u64 = THREAD_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Get the current thread's unique ID.
/// This ID is assigned once per thread and never changes.
pub fn current_thread_id() -> u64 {
    THREAD_ID.with(|id| *id)
}

/// Context for tracking memory allocations.
/// Captures both thread and task information for accurate attribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrackerContext {
    pub thread_id: u64,
    pub task_id: Option<u64>,
    pub tokio_task_id: Option<u64>,
}

impl TrackerContext {
    /// Capture the current tracking context.
    /// Returns thread ID and task ID (if in a task context).
    pub fn capture() -> Self {
        let task_id_from_context = TASK_CONTEXT.try_with(|ctx| *ctx).ok().flatten();
        let tokio_task_id = tokio::task::try_id().and_then(|id| id.to_string().parse().ok());

        Self {
            thread_id: current_thread_id(),
            task_id: task_id_from_context.or(CURRENT_TASK_ID.with(|cell| cell.get())),
            tokio_task_id,
        }
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

thread_local! {
    static CURRENT_TASK_ID: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}

tokio::task_local! {
    static TASK_CONTEXT: Option<u64>;
}

/// RAII guard for automatic task cleanup.
/// When dropped, it automatically clears the current task ID from thread-local storage.
pub struct TaskGuard {
    task_id: u64,
    cleaned_up: bool,
}

// SAFETY: TaskGuard can be safely sent between threads because:
// 1. It only contains primitive types (u64, bool) that are both Send and Sync.
// 2. It does NOT hold any references, raw pointers, or non-thread-safe data.
// 3. The thread-local state it manages (CURRENT_TASK_ID) is accessed via thread_local!(),
//    which provides per-thread isolation - each thread has its own copy of the task ID.
// 4. When TaskGuard is moved to another thread, it clears the task ID in the CURRENT
//    thread's thread-local storage (not the original thread's), which is the expected
//    behavior for task context management in async runtimes.
// 5. The Drop implementation clears the task ID of whichever thread is currently
//    executing, which is correct for RAII cleanup in async contexts where tasks
//    may migrate between threads.
unsafe impl Send for TaskGuard {}

// SAFETY: TaskGuard can be safely shared between threads because:
// 1. All fields (task_id, cleaned_up) are primitive types that are safe for concurrent access.
// 2. TaskGuard does not provide mutable access to its fields through &self.
// 3. The thread-local manipulation via clear_current_task_internal() operates on the
//    CURRENT thread's storage, not a shared resource, so concurrent calls from different
//    threads are inherently isolated.
// 4. No interior mutability or shared mutable state is exposed.
unsafe impl Sync for TaskGuard {}

impl TaskGuard {
    fn new(task_id: u64) -> Self {
        Self {
            task_id,
            cleaned_up: false,
        }
    }

    /// Get the task ID this guard is associated with.
    pub fn task_id(&self) -> u64 {
        self.task_id
    }

    /// Manually release the guard (prevents double cleanup).
    pub fn release(mut self) {
        self.cleaned_up = true;
        TaskGuard::clear_current_task_internal();
    }
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        if !self.cleaned_up {
            TaskGuard::clear_current_task_internal();
        }
    }
}

impl TaskGuard {
    fn clear_current_task_internal() {
        CURRENT_TASK_ID.with(|cell| cell.set(None));
    }
}

/// Async memory tracker for task-aware memory tracking.
pub struct AsyncTracker {
    allocations: Arc<Mutex<HashMap<usize, AsyncAllocation>>>,
    stats: Arc<Mutex<AsyncStats>>,
    profiles: Arc<Mutex<HashMap<u64, TaskMemoryProfile>>>,
    initialized: Arc<Mutex<bool>>,
}

impl AsyncTracker {
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(AsyncStats::default())),
            profiles: Arc::new(Mutex::new(HashMap::new())),
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    pub fn set_current_task(task_id: u64) {
        CURRENT_TASK_ID.with(|cell| cell.set(Some(task_id)));
    }

    pub fn clear_current_task() {
        CURRENT_TASK_ID.with(|cell| cell.set(None));
    }

    /// Get the current task ID from the thread-local storage.
    ///
    /// Note: This only returns the manually set task ID.
    /// For automatic tokio task detection, use `track_in_tokio_task()`.
    pub fn get_current_task() -> Option<u64> {
        CURRENT_TASK_ID.with(|cell| cell.get())
    }

    /// Enter a task context with automatic cleanup.
    /// Returns a TaskGuard that will clear the task ID when dropped.
    pub fn enter_task(task_id: u64) -> TaskGuard {
        Self::set_current_task(task_id);
        TaskGuard::new(task_id)
    }

    /// Execute a closure within a task context with automatic cleanup.
    /// The task ID is automatically cleared after the closure completes,
    /// even if the closure panics.
    pub fn with_task<F, T>(task_id: u64, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let _guard = Self::enter_task(task_id);
        f()
    }

    pub fn track_task_start(
        &self,
        task_id: u64,
        name: String,
        _thread_id: ThreadId,
    ) -> Result<(), AsyncError> {
        self.track_task_start_internal(task_id, None, name)
    }

    pub fn track_task_start_with_tokio(
        &self,
        task_id: u64,
        tokio_task_id: u64,
        name: String,
        _thread_id: ThreadId,
    ) -> Result<(), AsyncError> {
        self.track_task_start_internal(task_id, Some(tokio_task_id), name)
    }

    fn track_task_start_internal(
        &self,
        task_id: u64,
        tokio_task_id: Option<u64>,
        name: String,
    ) -> Result<(), AsyncError> {
        {
            let mut profiles = self
                .profiles
                .lock()
                .map_err(|e| AsyncError::mutex_lock_failed("profiles", &e.to_string()))?;

            if profiles.contains_key(&task_id) {
                return Err(AsyncError::duplicate_task(task_id));
            }

            let profile = match tokio_task_id {
                Some(id) => {
                    TaskMemoryProfile::with_tokio_id(task_id, id, name, TaskType::default())
                }
                None => TaskMemoryProfile::new(task_id, name, TaskType::default()),
            };
            profiles.insert(task_id, profile);
        }

        let mut stats = self
            .stats
            .lock()
            .map_err(|e| AsyncError::mutex_lock_failed("stats", &e.to_string()))?;
        stats.total_tasks += 1;
        stats.active_tasks += 1;

        Self::set_current_task(task_id);

        Ok(())
    }

    /// Track a task end.
    pub fn track_task_end(&self, task_id: u64) -> Result<(), AsyncError> {
        {
            let mut profiles = self
                .profiles
                .lock()
                .map_err(|e| AsyncError::mutex_lock_failed("profiles", &e.to_string()))?;

            let profile = profiles
                .get_mut(&task_id)
                .ok_or_else(|| AsyncError::task_not_found(task_id))?;

            if profile.is_completed() {
                return Ok(());
            }

            profile.mark_completed();
        }

        let mut stats = self
            .stats
            .lock()
            .map_err(|e| AsyncError::mutex_lock_failed("stats", &e.to_string()))?;
        stats.active_tasks = stats.active_tasks.saturating_sub(1);

        Self::clear_current_task();

        Ok(())
    }

    /// Execute an async block within a tokio task context.
    ///
    /// This method automatically detects the tokio task ID and sets up tracking.
    /// When the future completes, the task is automatically marked as ended.
    ///
    /// # Arguments
    ///
    /// * `name` - Task name for identification
    /// * `future` - The async block to execute
    ///
    /// # Returns
    ///
    /// A tuple of (unique_task_id, output).
    /// The unique_task_id is our internal ID (not the tokio task ID).
    pub async fn track_in_tokio_task<F, T>(&self, name: String, future: F) -> (u64, T)
    where
        F: Future<Output = T>,
    {
        let unique_task_id = generate_unique_task_id();
        let tokio_task_id = tokio::task::try_id().and_then(|id| id.to_string().parse().ok());
        let thread_id = std::thread::current().id();

        if let Some(tokio_id) = tokio_task_id {
            if let Err(e) =
                self.track_task_start_with_tokio(unique_task_id, tokio_id, name.clone(), thread_id)
            {
                tracing::warn!("Failed to track task start: {e}");
            }
        } else {
            if let Err(e) = self.track_task_start(unique_task_id, name.clone(), thread_id) {
                tracing::warn!("Failed to track task start: {e}");
            }
        }

        let output = future.await;

        if let Err(e) = self.track_task_end(unique_task_id) {
            tracing::warn!("Failed to track task end: {e}");
        }

        (unique_task_id, output)
    }

    /// Detect zombie tasks.
    ///
    /// A zombie task is a task that was started but never completed.
    /// These tasks may indicate memory leaks or improper task cleanup.
    ///
    /// # Returns
    ///
    /// A vector of task IDs for zombie tasks.
    pub fn detect_zombie_tasks(&self) -> Vec<u64> {
        let profiles = self.profiles.lock().unwrap();
        profiles
            .iter()
            .filter(|(_, p)| !p.is_completed())
            .map(|(&id, _)| id)
            .collect()
    }

    /// Get statistics about zombie tasks.
    pub fn zombie_task_stats(&self) -> (usize, usize) {
        let zombies = self.detect_zombie_tasks();
        let total = self.profiles.lock().unwrap().len();
        (zombies.len(), total)
    }

    pub fn track_allocation_auto(
        &self,
        ptr: usize,
        size: usize,
        var_name: Option<String>,
        type_name: Option<String>,
    ) {
        if let Some(task_id) = Self::get_current_task() {
            self.track_allocation_with_location(ptr, size, task_id, var_name, type_name, None);
        }
    }

    /// Track an allocation associated with a task.
    pub fn track_allocation(&self, ptr: usize, size: usize, task_id: u64) {
        self.track_allocation_with_location(ptr, size, task_id, None, None, None);
    }

    /// Track an allocation with source location.
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

        {
            if let Ok(mut allocations) = self.allocations.lock() {
                allocations.insert(ptr, allocation);
            } else {
                tracing::error!("Failed to acquire allocations lock during track_allocation");
            }
        }

        {
            if let Ok(mut profiles) = self.profiles.lock() {
                if let Some(profile) = profiles.get_mut(&task_id) {
                    profile.record_allocation(size as u64);
                }
            } else {
                tracing::error!("Failed to acquire profiles lock during track_allocation");
            }
        }

        {
            if let Ok(mut stats) = self.stats.lock() {
                stats.total_allocations += 1;
                stats.total_memory += size;
                stats.active_memory += size;
                if stats.active_memory > stats.peak_memory {
                    stats.peak_memory = stats.active_memory;
                }
            } else {
                tracing::error!("Failed to acquire stats lock during track_allocation");
            }
        }
    }

    /// Track a deallocation associated with a task.
    pub fn track_deallocation(&self, ptr: usize) {
        let (task_id, size) = {
            if let Ok(mut allocations) = self.allocations.lock() {
                allocations
                    .remove(&ptr)
                    .map(|alloc| (alloc.task_id, alloc.size))
                    .unwrap_or((0, 0))
            } else {
                tracing::error!("Failed to acquire allocations lock during track_deallocation");
                (0, 0)
            }
        };

        if task_id != 0 {
            if let Ok(mut profiles) = self.profiles.lock() {
                if let Some(profile) = profiles.get_mut(&task_id) {
                    profile.record_deallocation(size as u64);
                }
            } else {
                tracing::error!("Failed to acquire profiles lock during track_deallocation");
            }
        }

        if size > 0 {
            if let Ok(mut stats) = self.stats.lock() {
                stats.active_memory = stats.active_memory.saturating_sub(size);
                stats.total_deallocations += 1;
                stats.total_deallocated += size as u64;
            } else {
                tracing::error!("Failed to acquire stats lock during track_deallocation");
            }
        }
    }

    /// Get current statistics.
    pub fn get_stats(&self) -> AsyncStats {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            tracing::error!("Failed to acquire stats lock in get_stats");
            AsyncStats::default()
        }
    }

    /// Take a snapshot of current state.
    pub fn snapshot(&self) -> AsyncSnapshot {
        let profiles = if let Ok(p) = self.profiles.lock() {
            p
        } else {
            tracing::error!("Failed to acquire profiles lock in snapshot");
            return AsyncSnapshot::default();
        };

        let tasks: Vec<super::async_types::TaskInfo> = profiles
            .values()
            .filter(|p| p.completed_at_ms.is_none())
            .map(|p| super::async_types::TaskInfo {
                task_id: p.task_id,
                name: p.task_name.clone(),
                thread_id: std::thread::current().id(),
                created_at: p.created_at_ms * 1_000_000,
                active_allocations: p.total_allocations as usize,
                total_memory: p.current_memory as usize,
            })
            .collect();
        drop(profiles);

        let allocations = {
            if let Ok(allocs) = self.allocations.lock() {
                allocs.values().cloned().collect()
            } else {
                tracing::error!("Failed to acquire allocations lock in snapshot");
                Vec::new()
            }
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
        if let Ok(profiles) = self.profiles.lock() {
            profiles.get(&task_id).cloned()
        } else {
            tracing::error!("Failed to acquire profiles lock in get_task_profile");
            None
        }
    }

    /// Get all task profiles
    pub fn get_all_profiles(&self) -> Vec<TaskMemoryProfile> {
        if let Ok(profiles) = self.profiles.lock() {
            profiles.values().cloned().collect()
        } else {
            tracing::error!("Failed to acquire profiles lock in get_all_profiles");
            Vec::new()
        }
    }

    /// Check if tracker is initialized
    pub fn is_initialized(&self) -> bool {
        if let Ok(initialized) = self.initialized.lock() {
            *initialized
        } else {
            tracing::error!("Failed to acquire initialized lock in is_initialized");
            false
        }
    }

    /// Mark tracker as initialized
    pub fn set_initialized(&self) {
        if let Ok(mut initialized) = self.initialized.lock() {
            *initialized = true;
        } else {
            tracing::error!("Failed to acquire initialized lock in set_initialized");
        }
    }

    /// Generate task efficiency report
    pub fn analyze_task(&self, task_id: u64, task_type: TaskType) -> Option<TaskReport> {
        let profile = self.get_task_profile(task_id)?;

        let total_bytes = profile.total_bytes as f64;
        let total_allocations = profile.total_allocations as f64;
        let peak_memory = profile.peak_memory as f64;
        let duration_ms = profile.duration_ns as f64 / 1_000_000.0;

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

        let bottleneck = if duration_ms > 5000.0 {
            "Execution Time".to_string()
        } else if peak_memory > 100.0 * 1024.0 * 1024.0 {
            "Memory".to_string()
        } else if total_allocations > 10000.0 {
            "Allocations".to_string()
        } else {
            "None".to_string()
        };

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
                    task_type: profile.task_type,
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

impl Drop for AsyncTracker {
    fn drop(&mut self) {
        Self::clear_current_task();
    }
}

/// Initialize async memory tracking system
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
    if let Ok(mut global) = GLOBAL_TRACKER.lock() {
        *global = None;
    } else {
        tracing::error!("Failed to acquire global tracker lock in reset_global_tracker");
    }
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
pub fn create_tracked<F>(future: F) -> TrackedFuture<F>
where
    F: Future,
{
    TrackedFuture::new(future)
}

/// Spawn a tracked async task with automatic context management.
///
/// This function wraps a future with memscope tracking context.
/// The task ID is automatically generated and managed, and the
/// context is automatically cleaned up when the task completes.
///
/// # Arguments
///
/// * `future` - The async block to execute
///
/// # Returns
///
/// A `tokio::task::JoinHandle` that resolves to the future's output
///
/// # Example
///
/// ```ignore
/// let handle = spawn_tracked(async {
///     let data = vec![1u8; 1024];
///     tracker.track_as(&data, "buffer", file!(), line!());
///     // ... async work
/// });
/// ```
pub fn spawn_tracked<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let task_id = generate_unique_task_id();

    tokio::spawn(async move { TASK_CONTEXT.scope(Some(task_id), future).await })
}

/// Get current memory usage snapshot
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
pub fn track_current_allocation(ptr: usize, size: usize) -> AsyncResult<()> {
    let tracker = get_global_tracker()?;
    let task_info = super::async_types::get_current_task();

    if task_info.has_tracking_id() {
        tracker.track_allocation(ptr, size, (task_info.primary_id() & 0xFFFFFFFF) as u64);
    }

    Ok(())
}

/// Track deallocation for current task
pub fn track_current_deallocation(ptr: usize) -> AsyncResult<()> {
    let tracker = get_global_tracker()?;
    tracker.track_deallocation(ptr);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::backends::async_types::TaskOperation;

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
        tracker
            .track_task_start(1, "test_task".to_string(), thread_id)
            .unwrap();

        let stats = tracker.get_stats();
        assert_eq!(stats.total_tasks, 1);
        assert_eq!(stats.active_tasks, 1);

        tracker.track_task_end(1).unwrap();
        let stats = tracker.get_stats();
        assert_eq!(stats.active_tasks, 0);
    }

    #[test]
    fn test_allocation_tracking() {
        let tracker = AsyncTracker::new();
        let thread_id = std::thread::current().id();
        tracker
            .track_task_start(1, "test_task".to_string(), thread_id)
            .unwrap();
        tracker.track_allocation(0x1000, 1024, 1);

        let profile = tracker.get_task_profile(1);
        assert!(profile.is_some());
        let profile = profile.unwrap();
        assert_eq!(profile.total_allocations, 1);
        assert_eq!(profile.total_bytes, 1024);
    }

    #[test]
    fn test_initialization() {
        reset_global_tracker();

        let result = initialize();
        assert!(result.is_ok());

        let result2 = initialize();
        if let Err(e) = result2 {
            assert!(e.message().contains("Already initialized"));
        }

        let _ = shutdown();
    }

    #[test]
    fn test_shutdown() {
        reset_global_tracker();

        initialize().unwrap();
        let result = shutdown();
        assert!(result.is_ok());

        let result2 = shutdown();
        if let Err(e) = result2 {
            assert!(e.message().contains("Not initialized"));
        }
    }

    #[test]
    fn test_memory_snapshot() {
        reset_global_tracker();

        initialize().unwrap();
        let snapshot = get_memory_snapshot();
        assert_eq!(snapshot.active_task_count, 0);
        let _ = shutdown();
    }

    #[test]
    fn test_is_tracking_active() {
        reset_global_tracker();

        assert!(!is_tracking_active());
        initialize().unwrap();
        assert!(is_tracking_active());
        let _ = shutdown();
        assert!(!is_tracking_active());
    }

    #[test]
    fn test_task_memory_profile() {
        let tracker = AsyncTracker::new();
        let thread_id = std::thread::current().id();
        tracker
            .track_task_start(1, "test_task".to_string(), thread_id)
            .unwrap();
        tracker.track_allocation(0x1000, 1024, 1);
        tracker.track_allocation(0x2000, 2048, 1);
        tracker.track_task_end(1).unwrap();

        let profile = tracker.get_task_profile(1);
        assert!(profile.is_some());
        let profile = profile.unwrap();
        assert_eq!(profile.task_id, 1);
        assert_eq!(profile.total_allocations, 2);
        assert_eq!(profile.total_bytes, 3072);
    }

    #[test]
    fn test_duplicate_task_tracking() {
        let tracker = AsyncTracker::new();
        let thread_id = std::thread::current().id();

        // First registration should succeed
        let result = tracker.track_task_start(1, "test_task".to_string(), thread_id);
        assert!(result.is_ok());

        // Second registration should fail
        let result = tracker.track_task_start(1, "duplicate_task".to_string(), thread_id);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            matches!(error, AsyncError::TaskTracking { operation, .. } if matches!(operation, TaskOperation::Duplicate))
        );
    }

    #[test]
    fn test_task_not_found() {
        let tracker = AsyncTracker::new();

        // Calling track_task_end with non-existent task should fail
        let result = tracker.track_task_end(999);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            matches!(error, AsyncError::TaskTracking { operation, .. } if matches!(operation, TaskOperation::TaskNotFound))
        );
    }

    #[test]
    fn test_task_guard_cleanup() {
        assert!(AsyncTracker::get_current_task().is_none());

        {
            let _guard = AsyncTracker::enter_task(42);
            assert_eq!(AsyncTracker::get_current_task(), Some(42));
        }

        assert!(AsyncTracker::get_current_task().is_none());
    }

    #[test]
    fn test_with_task_closure() {
        assert!(AsyncTracker::get_current_task().is_none());

        let result = AsyncTracker::with_task(123, || {
            assert_eq!(AsyncTracker::get_current_task(), Some(123));
            "test_result"
        });

        assert_eq!(result, "test_result");
        assert!(AsyncTracker::get_current_task().is_none());
    }

    #[test]
    fn test_with_task_panic_cleanup() {
        assert!(AsyncTracker::get_current_task().is_none());

        let result = std::panic::catch_unwind(|| {
            AsyncTracker::with_task(999, || {
                assert_eq!(AsyncTracker::get_current_task(), Some(999));
                panic!("intentional panic");
            });
        });

        assert!(result.is_err());
        assert!(AsyncTracker::get_current_task().is_none());
    }

    #[test]
    fn test_generate_unique_task_id() {
        let id1 = generate_unique_task_id();
        let id2 = generate_unique_task_id();
        let id3 = generate_unique_task_id();

        assert!(id1 > 0);
        assert!(id2 > id1);
        assert!(id3 > id2);
    }

    #[test]
    fn test_track_start_with_tokio() {
        let tracker = AsyncTracker::new();
        let thread_id = std::thread::current().id();

        let result =
            tracker.track_task_start_with_tokio(1, 100, "tokio_task".to_string(), thread_id);
        assert!(result.is_ok());

        let profile = tracker.get_task_profile(1);
        assert!(profile.is_some());
        let profile = profile.unwrap();
        assert_eq!(profile.task_id, 1);
        assert_eq!(profile.tokio_task_id, Some(100));
        assert_eq!(profile.task_name, "tokio_task");
    }

    #[test]
    fn test_track_task_internal_without_tokio() {
        let tracker = AsyncTracker::new();
        let thread_id = std::thread::current().id();

        let result = tracker.track_task_start(2, "normal_task".to_string(), thread_id);
        assert!(result.is_ok());

        let profile = tracker.get_task_profile(2);
        assert!(profile.is_some());
        let profile = profile.unwrap();
        assert_eq!(profile.task_id, 2);
        assert_eq!(profile.tokio_task_id, None);
    }

    #[test]
    fn test_detect_zombie_tasks() {
        let tracker = AsyncTracker::new();
        let thread_id = std::thread::current().id();

        tracker
            .track_task_start(1, "task1".to_string(), thread_id)
            .unwrap();
        tracker
            .track_task_start(2, "task2".to_string(), thread_id)
            .unwrap();
        tracker
            .track_task_start(3, "task3".to_string(), thread_id)
            .unwrap();

        tracker.track_task_end(1).unwrap();

        let zombies = tracker.detect_zombie_tasks();
        assert_eq!(zombies.len(), 2);
        assert!(zombies.contains(&2));
        assert!(zombies.contains(&3));
    }

    #[test]
    fn test_zombie_task_stats() {
        let tracker = AsyncTracker::new();
        let thread_id = std::thread::current().id();

        tracker
            .track_task_start(1, "task1".to_string(), thread_id)
            .unwrap();
        tracker
            .track_task_start(2, "task2".to_string(), thread_id)
            .unwrap();

        tracker.track_task_end(1).unwrap();

        let (zombie_count, total) = tracker.zombie_task_stats();
        assert_eq!(zombie_count, 1);
        assert_eq!(total, 2);
    }

    #[test]
    fn test_no_zombie_tasks_when_all_complete() {
        let tracker = AsyncTracker::new();
        let thread_id = std::thread::current().id();

        tracker
            .track_task_start(1, "task1".to_string(), thread_id)
            .unwrap();
        tracker
            .track_task_start(2, "task2".to_string(), thread_id)
            .unwrap();

        tracker.track_task_end(1).unwrap();
        tracker.track_task_end(2).unwrap();

        let zombies = tracker.detect_zombie_tasks();
        assert!(zombies.is_empty());
    }

    #[test]
    fn test_task_memory_profile_with_tokio_id() {
        let profile = TaskMemoryProfile::with_tokio_id(1, 999, "test".to_string(), TaskType::Mixed);

        assert_eq!(profile.task_id, 1);
        assert_eq!(profile.tokio_task_id, Some(999));
        assert_eq!(profile.task_name, "test");
        assert_eq!(profile.task_type, TaskType::Mixed);
        assert!(!profile.is_completed());
    }

    #[tokio::test]
    async fn test_track_in_tokio_task_basic() {
        let tracker = AsyncTracker::new();

        let (task_id, result) = tracker
            .track_in_tokio_task("async_task".to_string(), async {
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                42
            })
            .await;

        assert!(task_id > 0);
        assert_eq!(result, 42);

        let profile = tracker.get_task_profile(task_id);
        assert!(profile.is_some());
        let profile = profile.unwrap();
        assert_eq!(profile.task_name, "async_task");
        assert!(profile.is_completed());
    }

    #[tokio::test]
    async fn test_track_in_tokio_task_basic_functionality() {
        let tracker = AsyncTracker::new();

        let (task_id, result) = tracker
            .track_in_tokio_task("test_task".to_string(), async {
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                "completed"
            })
            .await;

        assert!(task_id > 0);
        assert_eq!(result, "completed");

        let profile = tracker.get_task_profile(task_id);
        assert!(profile.is_some());
        let profile = profile.unwrap();
        assert_eq!(profile.task_name, "test_task");
        assert!(profile.is_completed());
    }

    #[test]
    fn test_global_tracker_integration() {
        reset_global_tracker();

        let result = initialize();
        assert!(result.is_ok());

        let tracker = get_global_tracker();
        assert!(tracker.is_ok());

        let tracker = tracker.unwrap();
        let stats = tracker.get_stats();
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.active_tasks, 0);

        let _ = shutdown();
    }
}
