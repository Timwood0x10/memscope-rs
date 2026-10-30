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
    /// Task scopes active on this thread, outermost first.
    ///
    /// Restoring from a saved `previous` value breaks when guards are dropped out of
    /// order: an inner guard would restore a scope that has already ended. Frames carry
    /// the guard token so a drop removes *its own* frame (not the first frame with the
    /// same task id), and a frame without an owner represents a context installed outside
    /// a guard (for example by `track_task_start`).
    static TASK_STACK: std::cell::RefCell<Vec<TaskFrame>> =
        const { std::cell::RefCell::new(Vec::new()) };
    /// Monotonic token source for guard frames.
    static GUARD_TOKEN: std::cell::Cell<u64> = const { std::cell::Cell::new(1) };
}

tokio::task_local! {
    static TASK_CONTEXT: Option<u64>;
}

/// RAII guard for automatic task cleanup.
///
/// Dropping the guard restores the task context that was active before the guard
/// was created (typically the parent task), instead of blanking the thread-local
/// slot. Blanking lost the parent context for every nested scope, and wiping the
/// slot from a thread that never owned it destroyed another task's context when a
/// guard was moved across threads.
/// One active task scope on the current thread.
#[derive(Debug, Clone, Copy)]
struct TaskFrame {
    task_id: u64,
    /// Guard token that owns the frame; `None` for a context installed outside a guard.
    owner: Option<u64>,
}

pub struct TaskGuard {
    task_id: u64,
    /// Identity of this guard's frame in [`TASK_STACK`].
    token: u64,
    cleaned_up: bool,
}

// SAFETY: TaskGuard can be safely sent between threads because:
// 1. It only contains primitive types (u64, Option<u64>, bool) that are Send + Sync.
// 2. It does NOT hold any references, raw pointers, or non-thread-safe data.
// 3. The restored thread-local (CURRENT_TASK_ID) is per thread.
// 4. `restore_previous` only writes when the slot still holds this guard's task id,
//    so a foreign thread (or an outer guard dropped after an inner one) cannot
//    clobber an unrelated task context.
unsafe impl Send for TaskGuard {}

// SAFETY: TaskGuard can be safely shared between threads because:
// 1. All fields are primitive types that are safe for concurrent access.
// 2. No mutable access to the fields is exposed through `&self`.
// 3. The thread-local access in `restore_previous` operates on the current
//    thread's storage, not a shared resource.
unsafe impl Sync for TaskGuard {}

impl TaskGuard {
    fn new(task_id: u64) -> Self {
        let token = GUARD_TOKEN.with(|cell| {
            let token = cell.get();
            cell.set(token.saturating_add(1));
            token
        });
        Self {
            task_id,
            token,
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
        self.restore_previous();
    }

    /// Remove this guard from the thread's scope stack and restore the new top.
    ///
    /// Works for any drop order: dropping an outer guard first leaves the inner one
    /// current, and the inner one then restores the layer below it (or nothing).
    fn restore_previous(&self) {
        let parent = TASK_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            // Remove exactly this guard's frame: several frames may share a task id when a
            // scope re-enters the same task, and removing the innermost one would leave the
            // wrong task current.
            stack.retain(|frame| frame.owner != Some(self.token));
            stack.last().map(|frame| frame.task_id)
        });

        // Only release the slot while this guard still owns it; a task installed
        // afterwards (manually or by another scope) must not be clobbered.
        CURRENT_TASK_ID.with(|cell| {
            if cell.get() == Some(self.task_id) {
                cell.set(parent);
            }
        });
    }
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        if self.cleaned_up {
            return;
        }
        self.restore_previous();
    }
}

mod dealloc;

/// Numeric id of `thread_id`, when it refers to the calling thread.
///
/// `std::thread::ThreadId` cannot be converted to a number, so an id that belongs to
/// another thread is reported as unknown rather than guessed.
fn numeric_thread_id(thread_id: ThreadId) -> Option<u64> {
    if thread_id == std::thread::current().id() {
        Some(crate::utils::current_thread_id_u64())
    } else {
        None
    }
}

/// Async memory tracker for task-aware memory tracking.
pub struct AsyncTracker {
    /// Live records per address, oldest generation first.
    allocations: Arc<Mutex<HashMap<usize, Vec<AsyncAllocation>>>>,
    /// Highest generation ever used per address, so a reused address gets the next
    /// number instead of restarting at 1 (which would make generations meaningless).
    generations: Arc<Mutex<HashMap<usize, u64>>>,
    stats: Arc<Mutex<AsyncStats>>,
    profiles: Arc<Mutex<HashMap<u64, TaskMemoryProfile>>>,
    initialized: Arc<Mutex<bool>>,
}

impl AsyncTracker {
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(Mutex::new(HashMap::new())),
            generations: Arc::new(Mutex::new(HashMap::new())),
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
        let guard = TaskGuard::new(task_id);
        let previous = Self::get_current_task();

        TASK_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            // A context installed outside a guard (e.g. `track_task_start`) is remembered
            // as the base frame, otherwise dropping the guard would clear it instead of
            // restoring the parent task.
            if let Some(previous) = previous {
                if stack.last().map(|frame| frame.task_id) != Some(previous) {
                    stack.push(TaskFrame {
                        task_id: previous,
                        owner: None,
                    });
                }
            }
            stack.push(TaskFrame {
                task_id,
                owner: Some(guard.token),
            });
        });

        Self::set_current_task(task_id);
        guard
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
        thread_id: ThreadId,
    ) -> Result<(), AsyncError> {
        self.track_task_start_internal(task_id, None, name, thread_id)
    }

    pub fn track_task_start_with_tokio(
        &self,
        task_id: u64,
        tokio_task_id: u64,
        name: String,
        thread_id: ThreadId,
    ) -> Result<(), AsyncError> {
        self.track_task_start_internal(task_id, Some(tokio_task_id), name, thread_id)
    }

    fn track_task_start_internal(
        &self,
        task_id: u64,
        tokio_task_id: Option<u64>,
        name: String,
        thread_id: ThreadId,
    ) -> Result<(), AsyncError> {
        {
            let mut profiles = self
                .profiles
                .lock()
                .map_err(|e| AsyncError::mutex_lock_failed("profiles", &e.to_string()))?;

            if profiles.contains_key(&task_id) {
                return Err(AsyncError::duplicate_task(task_id));
            }

            let mut profile = match tokio_task_id {
                Some(id) => {
                    TaskMemoryProfile::with_tokio_id(task_id, id, name, TaskType::default())
                }
                None => TaskMemoryProfile::new(task_id, name, TaskType::default()),
            };
            // Record the numeric thread id when it is genuinely known: only the
            // thread performing this call can be mapped to a numeric id. The snapshot
            // must never substitute the thread that happens to build it.
            profile.set_thread_id_u64(numeric_thread_id(thread_id));
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

        // Only release the slot when this task still owns it: the task may end on a
        // different thread, or another task may already be running on this one.
        CURRENT_TASK_ID.with(|cell| {
            if cell.get() == Some(task_id) {
                cell.set(None);
            }
        });

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
        } else if let Err(e) = self.track_task_start(unique_task_id, name.clone(), thread_id) {
            tracing::warn!("Failed to track task start: {e}");
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
    ///
    /// Optimized to acquire the lock only once.
    pub fn zombie_task_stats(&self) -> (usize, usize) {
        let profiles = self.profiles.lock().unwrap();
        let zombies = profiles.iter().filter(|(_, p)| !p.is_completed()).count();
        let total = profiles.len();
        (zombies, total)
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
    ///
    /// The same address is reported more than once (hook plus `track!`, repeated
    /// `track!`, reuse). Only the first sighting of an address/size pair opens a
    /// record; a different size starts a new generation, so a release can always name
    /// the binding it refers to and `total_memory - total_deallocated == active_memory`
    /// keeps holding (plan A06 / §7.2).
    pub fn track_allocation_with_location(
        &self,
        ptr: usize,
        size: usize,
        task_id: u64,
        var_name: Option<String>,
        type_name: Option<String>,
        source_location: Option<super::async_types::SourceLocation>,
    ) {
        let mut previous: Option<(u64, u64)> = None;
        let mut generation = 1u64;

        {
            let Ok(mut allocations) = self.allocations.lock() else {
                tracing::error!("Failed to acquire allocations lock during track_allocation");
                return;
            };
            let entries = allocations.entry(ptr).or_default();

            if let Some(existing) = entries.last_mut() {
                if existing.size == size {
                    // Duplicate observation: refresh metadata, keep owner and counters.
                    existing.var_name = var_name;
                    existing.type_name = type_name;
                    if source_location.is_some() {
                        existing.source_location = source_location;
                    }
                    return;
                }
                // A different size cannot be proven to be the same block: an
                // observation never sees the allocator. Close the superseded record and
                // give the new one its own generation, exactly like the core tracker and
                // the binding ledger (plan A06 follow-up).
                previous = Some((existing.task_id, existing.size as u64));
                entries.pop();
            }

            // Every record that is pushed gets the next generation for this address,
            // so a late release can still name the binding it refers to.
            if let Ok(mut generations) = self.generations.lock() {
                let counter = generations.entry(ptr).or_insert(0);
                *counter = counter.saturating_add(1);
                generation = *counter;
            }

            entries.push(AsyncAllocation {
                ptr,
                size,
                timestamp: Self::now(),
                task_id,
                var_name,
                type_name,
                source_location,
                generation,
            });
        }

        let size_u64 = size as u64;

        if let Some((previous_task, previous_size)) = previous {
            // The superseded record leaves its owner's live set (it is a lost block for
            // that task), and the new record is a fresh allocation for the observer.
            // Both sides are booked so `total_memory - total_deallocated ==
            // active_memory` keeps holding (plan A06 follow-up).
            if let Ok(mut profiles) = self.profiles.lock() {
                if let Some(profile) = profiles.get_mut(&previous_task) {
                    profile.record_deallocation(previous_size);
                }
                if let Some(profile) = profiles.get_mut(&task_id) {
                    profile.record_allocation(size_u64);
                    profile.set_thread_id_u64(Some(crate::utils::current_thread_id_u64()));
                }
            } else {
                tracing::error!("Failed to acquire profiles lock during track_allocation");
            }

            if let Ok(mut stats) = self.stats.lock() {
                stats.total_allocations += 1;
                stats.total_memory += size;
                stats.total_deallocated += previous_size;
                stats.active_memory = stats
                    .active_memory
                    .saturating_sub(previous_size as usize)
                    .saturating_add(size);
                if stats.active_memory > stats.peak_memory {
                    stats.peak_memory = stats.active_memory;
                }
            } else {
                tracing::error!("Failed to acquire stats lock during track_allocation");
            }

            return;
        }

        if let Ok(mut profiles) = self.profiles.lock() {
            if let Some(profile) = profiles.get_mut(&task_id) {
                profile.record_allocation(size_u64);
                // The thread that allocates is the best available evidence of where
                // the task actually ran.
                profile.set_thread_id_u64(Some(crate::utils::current_thread_id_u64()));
            }
        } else {
            tracing::error!("Failed to acquire profiles lock during track_allocation");
        }

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
                thread_id: p.thread_id_u64,
                created_at: p.created_at_ms * 1_000_000,
                active_allocations: p.total_allocations as usize,
                total_memory: p.current_memory as usize,
            })
            .collect();
        drop(profiles);

        let allocations = {
            if let Ok(allocs) = self.allocations.lock() {
                allocs
                    .values()
                    .flat_map(|entries| entries.iter().cloned())
                    .collect()
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

/// Register an existing AsyncTracker instance as the global singleton.
/// This allows GlobalTracker to share its async_tracker instance with
/// the async_tracker module, so `spawn_tracked()` can register task lifecycle
/// events on the same instance that the renderer reads from.
pub fn register_global(tracker: Arc<AsyncTracker>) -> AsyncResult<()> {
    let mut global = GLOBAL_TRACKER.lock().map_err(|_| AsyncError::System {
        operation: Arc::from("register_global"),
        message: Arc::from("Failed to acquire global tracker lock"),
    })?;
    *global = Some(tracker);
    Ok(())
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

    tokio::spawn(async move {
        // Register task start with the global async tracker so profiles
        // are populated for the dashboard renderer.
        let tracker = get_global_tracker().ok();
        let task_name = format!("spawned_task_{}", task_id);
        if let Some(ref tracker) = tracker {
            let thread_id = std::thread::current().id();
            let _ = tracker.track_task_start(task_id, task_name.clone(), thread_id);
        }

        // Register with TaskIdRegistry so the Task Relationship Graph
        // in the dashboard shows this task with parent-child hierarchy.
        crate::task_registry::global_registry().register_explicit_task(task_id, &task_name);

        // Set thread-local CURRENT_TASK_ID so that GlobalTracker::track_as()
        // can associate allocations with this task via AsyncTracker::get_current_task().
        AsyncTracker::set_current_task(task_id);

        // Run the user's future inside the tokio task-local context.
        let result = TASK_CONTEXT.scope(Some(task_id), future).await;

        // Cleanup: clear thread-local task ID and signal task completion.
        AsyncTracker::clear_current_task();
        crate::task_registry::global_registry().unregister_task(task_id);
        if let Some(ref tracker) = tracker {
            let _ = tracker.track_task_end(task_id);
        }

        result
    })
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
        // `TaskId` is a u128: truncating it to 32 bits made long-running task ids
        // collide. Report the unsupported width instead of losing the mapping.
        match u64::try_from(task_info.primary_id()) {
            Ok(task_id) => tracker.track_allocation(ptr, size, task_id),
            Err(_) => tracing::warn!(
                target: "memscope::async",
                task_id = %task_info.primary_id(),
                ptr = format_args!("{ptr:#x}"),
                "async task id does not fit into u64; allocation is not attributed"
            ),
        }
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
mod tests;
