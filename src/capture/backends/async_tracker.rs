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
    AsyncAllocation, AsyncError, AsyncResult, AsyncSnapshot, AsyncStats, TrackedFuture,
};
use super::task_profile::{TaskMemoryProfile, TaskType};

/// Generate a new unique task ID.
/// This ID is never recycled, ensuring unique identification.
pub fn generate_unique_task_id() -> u64 {
    crate::task_registry::generate_task_id()
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
        let _suppression = crate::core::allocator::TrackingGuard::acquire();
        let task_id_from_context = TASK_CONTEXT.try_with(|ctx| *ctx).ok().flatten();
        let tokio_task_id = tokio::task::try_id().and_then(|id| id.to_string().parse().ok());

        Self {
            thread_id: current_thread_id(),
            task_id: task_id_from_context.or_else(AsyncTracker::get_current_task),
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

mod task_context;
pub use task_context::TaskGuard;

tokio::task_local! {
    static TASK_CONTEXT: Option<u64>;
}

mod dealloc;
mod future;
mod queries;
use future::{ContextualFuture, TaskLifecycleGuard};
pub use queries::get_memory_snapshot;

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
    event_store: Option<crate::event_store::SharedEventStore>,
}

impl AsyncTracker {
    pub(crate) fn lifecycle_profiles(&self) -> Vec<TaskMemoryProfile> {
        let _guard = crate::core::allocator::TrackingGuard::acquire();
        self.profiles
            .lock()
            .map(|p| p.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn new() -> Self {
        Self {
            allocations: Arc::new(Mutex::new(HashMap::new())),
            generations: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(AsyncStats::default())),
            profiles: Arc::new(Mutex::new(HashMap::new())),
            initialized: Arc::new(Mutex::new(false)),
            event_store: None,
        }
    }

    pub fn set_current_task(task_id: u64) {
        task_context::set(task_id);
    }

    pub fn clear_current_task() {
        task_context::clear();
    }

    /// Get the current task ID from the thread-local storage.
    ///
    /// Note: This only returns the manually set task ID.
    /// For automatic tokio task detection, use `track_in_tokio_task()`.
    pub fn get_current_task() -> Option<u64> {
        task_context::current()
    }

    /// Enter a task context with automatic cleanup.
    /// Dropping the returned guard restores the enclosing live scope.
    pub fn enter_task(task_id: u64) -> TaskGuard {
        task_context::enter(task_id)
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
        self.track_task_start_internal(task_id, None, name, thread_id)?;
        // For the public API, set the current task context. Async paths
        // (track_in_tokio_task, spawn_tracked) call track_task_start_internal
        // directly via track_task_start_with_tokio and do NOT want the
        // context set here — ContextualFuture manages it per-poll.
        // However, some callers call track_task_start directly (e.g. tests)
        // and expect the context to be set. To support both, we set the
        // context here only when NOT already inside a poll guard.
        //
        // Since track_in_tokio_task and spawn_tracked both go through
        // track_task_start_with_tokio or track_task_start, we need a way
        // to distinguish. The simplest: async paths call
        // track_task_start_internal directly; external callers use
        // track_task_start. So set context in track_task_start but NOT
        // in track_task_start_with_tokio.
        Self::set_current_task(task_id);
        Ok(())
    }

    pub fn track_task_start_with_tokio(
        &self,
        task_id: u64,
        tokio_task_id: u64,
        name: String,
        thread_id: ThreadId,
    ) -> Result<(), AsyncError> {
        // Do NOT set the current task context here. This method is called
        // from async paths (track_in_tokio_task, spawn_tracked) where the
        // ContextualFuture poll wrapper manages context installation.
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

        // Do NOT call set_current_task here. Context installation is the
        // responsibility of the poll-level wrapper (ContextualFuture), not
        // the metadata registration. Leaving an explicit context here would
        // persist across await points and pollute other tasks' attribution.
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

        // Remove ended base frames even when a nested guard is currently on top.
        task_context::end(task_id);

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
        // Capture parent at call time, consistent with spawn_tracked.
        let parent_id = crate::task_registry::TaskIdRegistry::current_task_id();

        if let Some(tokio_id) = tokio_task_id {
            if let Err(e) =
                self.track_task_start_with_tokio(unique_task_id, tokio_id, name.clone(), thread_id)
            {
                tracing::warn!("Failed to track task start: {e}");
            }
        } else if let Err(e) =
            self.track_task_start_internal(unique_task_id, None, name.clone(), thread_id)
        {
            tracing::warn!("Failed to track task start: {e}");
        }

        crate::task_registry::global_registry().register_explicit_task_with_parent(
            unique_task_id,
            &name,
            parent_id,
        );
        let lifecycle = future::BorrowedLifecycleGuard::new(self, unique_task_id);

        // Use the same poll-level wrapper as spawn_tracked for consistent
        // per-poll context management.
        let contextual = ContextualFuture::new(future, unique_task_id);
        let output = contextual.await;

        drop(lifecycle);

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
        let _suppression = crate::core::allocator::TrackingGuard::acquire();
        self.get_all_profiles()
            .iter()
            .filter(|p| !p.is_completed())
            .map(|p| p.task_id)
            .collect()
    }

    /// Get statistics about zombie tasks.
    ///
    /// Optimized to acquire the lock only once.
    pub fn zombie_task_stats(&self) -> (usize, usize) {
        let profiles = self.get_all_profiles();
        let zombies = profiles.iter().filter(|p| !p.is_completed()).count();
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
        let _guard = crate::core::allocator::TrackingGuard::acquire();
        if let Some((_, rebuilt)) = self.rebuilt() {
            return queries::stats(&rebuilt);
        }
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            tracing::error!("Failed to acquire stats lock in get_stats");
            AsyncStats::default()
        }
    }

    /// Take a snapshot of current state.
    pub fn snapshot(&self) -> AsyncSnapshot {
        // Suppress allocator tracking during the query.
        let _guard = crate::core::allocator::TrackingGuard::acquire();
        if let Some((captured_at, rebuilt)) = self.rebuilt() {
            return queries::snapshot(captured_at, rebuilt);
        }
        queries::local_snapshot(self)
    }

    /// Get task memory profile.
    ///
    /// Connected trackers query their event source; standalone instances use local records.
    pub fn get_task_profile(&self, task_id: u64) -> Option<TaskMemoryProfile> {
        let _guard = crate::core::allocator::TrackingGuard::acquire();

        if let Some((_, rebuilt)) = self.rebuilt() {
            return rebuilt.profiles.into_iter().find(|p| p.task_id == task_id);
        }
        if let Ok(profiles) = self.profiles.lock() {
            profiles.get(&task_id).cloned().map(|mut profile| {
                profile.refresh_at(Self::now());
                profile
            })
        } else {
            tracing::error!("Failed to acquire profiles lock in get_task_profile");
            None
        }
    }

    /// Get all task profiles.
    ///
    /// Uses the same event projection as stats and snapshots when connected.
    pub fn get_all_profiles(&self) -> Vec<TaskMemoryProfile> {
        let _guard = crate::core::allocator::TrackingGuard::acquire();

        if let Some((_, rebuilt)) = self.rebuilt() {
            return rebuilt.profiles;
        }
        if let Ok(profiles) = self.profiles.lock() {
            let captured_at = Self::now();
            profiles
                .values()
                .cloned()
                .map(|mut profile| {
                    profile.refresh_at(captured_at);
                    profile
                })
                .collect()
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
        let _suppression = crate::core::allocator::TrackingGuard::acquire();
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
        let _suppression = crate::core::allocator::TrackingGuard::acquire();
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

/// Get the global async tracker (public for cross-module release notification).
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

/// Notify the global AsyncTracker that a deallocation occurred.
///
/// This function is for explicit API calls (e.g. `track_current_deallocation`).
/// It must NOT be called from the allocator's dealloc callback — doing so
/// re-enters AsyncTracker locks and deadlocks with `shutdown()` and
/// concurrent `track_allocation`. The event-stream replay
/// (`rebuild_with_live_tracker`) is the authoritative source for exports.
pub fn notify_deallocation(ptr: usize) {
    let Ok(tracker) = get_global_tracker() else {
        return;
    };
    tracker.track_deallocation(ptr);
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
/// Wraps a future with an automatically managed task ID.
///
/// Captures the parent at spawn time and restores context after every poll.
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

    // Capture the parent task ID at spawn time from the spawning thread's
    // context. This is critical: if we read it at first poll instead, a
    // different task might have already set its context on this worker
    // thread, producing a wrong parent-child edge.
    let parent_id = crate::task_registry::TaskIdRegistry::current_task_id();

    tokio::spawn(async move {
        // Register task start with the global async tracker so profiles
        // are populated for the dashboard renderer.
        let tracker = get_global_tracker().ok();
        let task_name = format!("spawned_task_{}", task_id);
        if let Some(ref tracker) = tracker {
            let thread_id = std::thread::current().id();
            // Use track_task_start_with_tokio (which does NOT set TLS context)
            // when we have a tokio task ID, or track_task_start_internal
            // directly when we don't. Either way, no TLS context is installed
            // here — the ContextualFuture poll wrapper handles it per-poll.
            let tokio_id = tokio::task::try_id().and_then(|id| id.to_string().parse().ok());
            if let Some(tokio_id) = tokio_id {
                let _ = tracker.track_task_start_with_tokio(
                    task_id,
                    tokio_id,
                    task_name.clone(),
                    thread_id,
                );
            } else {
                let _ =
                    tracker.track_task_start_internal(task_id, None, task_name.clone(), thread_id);
            }
        }

        // Register with TaskIdRegistry using the captured parent_id.
        // We use register_explicit_task_with_parent so the registration
        // does NOT read or write CURRENT_TASK_ID — the poll-level wrapper
        // handles context installation per-poll.
        crate::task_registry::global_registry()
            .register_explicit_task_with_parent(task_id, &task_name, parent_id);

        // The lifecycle guard ensures task-end cleanup runs in all cases:
        // normal completion, cancellation (future dropped), and panic.
        // When the guard drops, it calls unregister_task and track_task_end
        // inside the task's poll context so the cleanup is attributed correctly.
        let mut lifecycle = TaskLifecycleGuard::new(task_id, tracker.clone());

        // Wrap the future with poll-level context management.
        let contextual = ContextualFuture::new(future, task_id);
        let result = contextual.await;

        // The guard's Drop will run the cleanup. Set the output so the
        // guard knows the task completed normally (not cancelled).
        lifecycle.set_completed();
        result
    })
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
