//! Async-aware memory tracking for futures and tasks
//!
//! This module provides specialized tracking for async contexts:
//! - Task-level memory allocation tracking
//! - Future state transitions with memory impact
//! - Await point performance analysis
//! - Cross-task memory sharing detection

use crate::lockfree::{AllocationEvent, DeallocationEvent};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Global async memory tracker instance
static GLOBAL_ASYNC_TRACKER: once_cell::sync::Lazy<Arc<AsyncMemoryTracker>> =
    once_cell::sync::Lazy::new(|| Arc::new(AsyncMemoryTracker::new()));

/// Get the global async tracker
pub fn get_async_tracker() -> Arc<AsyncMemoryTracker> {
    GLOBAL_ASYNC_TRACKER.clone()
}

/// Async-aware memory tracker
pub struct AsyncMemoryTracker {
    /// Task ID counter
    next_task_id: AtomicU64,
    /// Active tasks and their allocations
    task_allocations: DashMap<u64, TaskMemoryState>,
    /// Future state tracking
    future_states: DashMap<usize, AsyncFutureInfo>,
    /// Await point performance metrics
    await_metrics: DashMap<String, AwaitPointMetrics>,
    /// Task hierarchy (parent -> children)
    task_hierarchy: DashMap<u64, Vec<u64>>,
    /// Cross-task shared memory
    shared_memory: DashMap<usize, SharedMemoryInfo>,
}

impl AsyncMemoryTracker {
    /// Create a new async memory tracker
    pub fn new() -> Self {
        Self {
            next_task_id: AtomicU64::new(1),
            task_allocations: DashMap::new(),
            future_states: DashMap::new(),
            await_metrics: DashMap::new(),
            task_hierarchy: DashMap::new(),
            shared_memory: DashMap::new(),
        }
    }

    /// Start tracking a new async task
    pub fn start_task(&self, task_name: String, parent_task: Option<u64>) -> u64 {
        let task_id = self.next_task_id.fetch_add(1, Ordering::Relaxed);
        
        let task_state = TaskMemoryState {
            task_id,
            task_name: task_name.clone(),
            parent_task,
            start_time: Instant::now(),
            allocations: Vec::new(),
            deallocations: Vec::new(),
            peak_memory: 0,
            current_memory: 0,
            await_count: 0,
            total_await_time: Duration::ZERO,
        };

        self.task_allocations.insert(task_id, task_state);

        // Update task hierarchy
        if let Some(parent_id) = parent_task {
            self.task_hierarchy.entry(parent_id)
                .or_insert_with(Vec::new)
                .push(task_id);
        }

        tracing::debug!("Started async task tracking: {} (ID: {})", task_name, task_id);
        task_id
    }

    /// Track memory allocation in async context
    pub fn track_async_allocation(&self, task_id: u64, allocation: AllocationEvent) {
        if let Some(mut task_state) = self.task_allocations.get_mut(&task_id) {
            task_state.current_memory += allocation.size;
            task_state.peak_memory = task_state.peak_memory.max(task_state.current_memory);
            task_state.allocations.push(allocation.clone());

            // Check for cross-task sharing potential
            self.analyze_sharing_potential(&allocation);
        }
    }

    /// Track memory deallocation in async context
    pub fn track_async_deallocation(&self, task_id: u64, deallocation: DeallocationEvent) {
        if let Some(mut task_state) = self.task_allocations.get_mut(&task_id) {
            // Find matching allocation to calculate size
            if let Some(alloc) = task_state.allocations.iter()
                .find(|a| a.ptr == deallocation.ptr) {
                task_state.current_memory = task_state.current_memory.saturating_sub(alloc.size);
            }
            task_state.deallocations.push(deallocation);
        }
    }

    /// Record await point start
    pub fn start_await(&self, task_id: u64, await_location: String) -> AwaitHandle {
        if let Some(mut task_state) = self.task_allocations.get_mut(&task_id) {
            task_state.await_count += 1;
        }

        AwaitHandle {
            task_id,
            await_location,
            start_time: Instant::now(),
            tracker: self,
        }
    }

    /// Complete await point (called by AwaitHandle::drop)
    fn complete_await(&self, task_id: u64, await_location: String, duration: Duration) {
        // Update task await time
        if let Some(mut task_state) = self.task_allocations.get_mut(&task_id) {
            task_state.total_await_time += duration;
        }

        // Update await metrics
        self.await_metrics.entry(await_location.clone())
            .and_modify(|metrics| {
                metrics.total_calls += 1;
                metrics.total_duration += duration;
                metrics.min_duration = metrics.min_duration.min(duration);
                metrics.max_duration = metrics.max_duration.max(duration);
            })
            .or_insert_with(|| AwaitPointMetrics {
                location: await_location,
                total_calls: 1,
                total_duration: duration,
                min_duration: duration,
                max_duration: duration,
            });
    }

    /// Analyze potential memory sharing between tasks
    fn analyze_sharing_potential(&self, allocation: &AllocationEvent) {
        // Simple heuristic: large allocations (>1MB) might be shared
        if allocation.size > 1_048_576 {
            let shared_info = SharedMemoryInfo {
                ptr: allocation.ptr,
                size: allocation.size,
                allocation_time: allocation.timestamp,
                sharing_tasks: vec![allocation.thread_id],
                sharing_type: SharingType::Potential,
            };
            self.shared_memory.insert(allocation.ptr, shared_info);
        }
    }

    /// Finish tracking an async task
    pub fn finish_task(&self, task_id: u64) -> Option<TaskMemoryReport> {
        if let Some((_, task_state)) = self.task_allocations.remove(&task_id) {
            let duration = task_state.start_time.elapsed();
            
            let report = TaskMemoryReport {
                task_id,
                task_name: task_state.task_name,
                parent_task: task_state.parent_task,
                duration,
                total_allocations: task_state.allocations.len(),
                total_allocated: task_state.allocations.iter().map(|a| a.size).sum(),
                peak_memory: task_state.peak_memory,
                await_count: task_state.await_count,
                total_await_time: task_state.total_await_time,
                memory_efficiency: self.calculate_memory_efficiency(&task_state),
            };

            tracing::debug!("Finished async task tracking: {} (ID: {})", report.task_name, task_id);
            Some(report)
        } else {
            None
        }
    }

    /// Calculate memory efficiency metrics
    fn calculate_memory_efficiency(&self, task_state: &TaskMemoryState) -> MemoryEfficiency {
        let avg_memory = if !task_state.allocations.is_empty() {
            task_state.allocations.iter().map(|a| a.size).sum::<usize>() / task_state.allocations.len()
        } else {
            0
        };

        let fragmentation_ratio = if task_state.peak_memory > 0 {
            task_state.current_memory as f64 / task_state.peak_memory as f64
        } else {
            1.0
        };

        MemoryEfficiency {
            avg_allocation_size: avg_memory,
            peak_to_current_ratio: fragmentation_ratio,
            allocations_per_second: task_state.allocations.len() as f64 / task_state.start_time.elapsed().as_secs_f64(),
            await_overhead_ratio: task_state.total_await_time.as_secs_f64() / task_state.start_time.elapsed().as_secs_f64(),
        }
    }

    /// Get comprehensive async analysis
    pub fn get_async_analysis(&self) -> AsyncMemoryAnalysis {
        let active_tasks: Vec<_> = self.task_allocations.iter()
            .map(|entry| {
                let task_state = entry.value();
                ActiveTaskInfo {
                    task_id: task_state.task_id,
                    task_name: task_state.task_name.clone(),
                    current_memory: task_state.current_memory,
                    await_count: task_state.await_count,
                    duration: task_state.start_time.elapsed(),
                }
            })
            .collect();

        let await_hotspots: Vec<_> = self.await_metrics.iter()
            .filter(|entry| entry.value().total_calls > 10) // Potential hotspots
            .map(|entry| entry.value().clone())
            .collect();

        let shared_memory_regions: Vec<_> = self.shared_memory.iter()
            .map(|entry| entry.value().clone())
            .collect();

        AsyncMemoryAnalysis {
            active_tasks,
            await_hotspots,
            shared_memory_regions,
            total_tasks_created: self.next_task_id.load(Ordering::Relaxed) - 1,
        }
    }
}

/// RAII guard for await point tracking
pub struct AwaitHandle<'a> {
    task_id: u64,
    await_location: String,
    start_time: Instant,
    tracker: &'a AsyncMemoryTracker,
}

impl<'a> Drop for AwaitHandle<'a> {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        self.tracker.complete_await(self.task_id, self.await_location.clone(), duration);
    }
}

/// Memory state for a single async task
#[derive(Debug, Clone)]
pub struct TaskMemoryState {
    pub task_id: u64,
    pub task_name: String,
    pub parent_task: Option<u64>,
    pub start_time: Instant,
    pub allocations: Vec<AllocationEvent>,
    pub deallocations: Vec<DeallocationEvent>,
    pub peak_memory: usize,
    pub current_memory: usize,
    pub await_count: usize,
    pub total_await_time: Duration,
}

/// Performance metrics for await points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwaitPointMetrics {
    pub location: String,
    pub total_calls: usize,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
}

impl AwaitPointMetrics {
    pub fn avg_duration(&self) -> Duration {
        if self.total_calls > 0 {
            self.total_duration / self.total_calls as u32
        } else {
            Duration::ZERO
        }
    }
}

/// Information about shared memory between tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMemoryInfo {
    pub ptr: usize,
    pub size: usize,
    pub allocation_time: u64,
    pub sharing_tasks: Vec<u64>,
    pub sharing_type: SharingType,
}

/// Types of memory sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharingType {
    /// Potentially shared (heuristic detection)
    Potential,
    /// Confirmed shared (explicit tracking)
    Confirmed,
    /// Unsafe sharing detected
    Unsafe,
}

/// Future state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncFutureInfo {
    pub ptr: usize,
    pub task_id: u64,
    pub future_type: String,
    pub current_state: FutureState,
    pub memory_allocated: usize,
    pub poll_count: usize,
}

/// Future states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FutureState {
    Created,
    Polling,
    Pending,
    Ready,
    Cancelled,
}

/// Task completion report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMemoryReport {
    pub task_id: u64,
    pub task_name: String,
    pub parent_task: Option<u64>,
    pub duration: Duration,
    pub total_allocations: usize,
    pub total_allocated: usize,
    pub peak_memory: usize,
    pub await_count: usize,
    pub total_await_time: Duration,
    pub memory_efficiency: MemoryEfficiency,
}

/// Memory efficiency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEfficiency {
    pub avg_allocation_size: usize,
    pub peak_to_current_ratio: f64,
    pub allocations_per_second: f64,
    pub await_overhead_ratio: f64,
}

/// Active task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTaskInfo {
    pub task_id: u64,
    pub task_name: String,
    pub current_memory: usize,
    pub await_count: usize,
    pub duration: Duration,
}

/// Comprehensive async memory analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncMemoryAnalysis {
    pub active_tasks: Vec<ActiveTaskInfo>,
    pub await_hotspots: Vec<AwaitPointMetrics>,
    pub shared_memory_regions: Vec<SharedMemoryInfo>,
    pub total_tasks_created: u64,
}

/// RAII guard for async task tracking
pub struct AsyncTrackingGuard {
    task_id: u64,
    tracker: Arc<AsyncMemoryTracker>,
}

impl AsyncTrackingGuard {
    /// Create new async tracking guard
    pub fn new(task_name: String) -> Self {
        let tracker = get_async_tracker();
        let task_id = tracker.start_task(task_name, None);
        
        Self { task_id, tracker }
    }

    /// Create new async tracking guard with parent
    pub fn with_parent(task_name: String, parent_task: u64) -> Self {
        let tracker = get_async_tracker();
        let task_id = tracker.start_task(task_name, Some(parent_task));
        
        Self { task_id, tracker }
    }

    /// Get task ID
    pub fn task_id(&self) -> u64 {
        self.task_id
    }

    /// Track allocation in this task
    pub fn track_allocation(&self, allocation: AllocationEvent) {
        self.tracker.track_async_allocation(self.task_id, allocation);
    }

    /// Track deallocation in this task
    pub fn track_deallocation(&self, deallocation: DeallocationEvent) {
        self.tracker.track_async_deallocation(self.task_id, deallocation);
    }

    /// Start await tracking
    pub fn start_await(&self, location: String) -> AwaitHandle {
        self.tracker.start_await(self.task_id, location)
    }
}

impl Drop for AsyncTrackingGuard {
    fn drop(&mut self) {
        if let Some(report) = self.tracker.finish_task(self.task_id) {
            tracing::info!("Task {} completed: {:?}", report.task_name, report);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_task_tracking() {
        let tracker = AsyncMemoryTracker::new();
        
        // Start a task
        let task_id = tracker.start_task("test_task".to_string(), None);
        assert_eq!(task_id, 1);

        // Track an allocation
        let allocation = AllocationEvent {
            ptr: 0x1000,
            size: 1024,
            timestamp: 12345,
            call_stack: vec![],
            thread_id: 1,
        };
        tracker.track_async_allocation(task_id, allocation);

        // Check task state
        let task_state = tracker.task_allocations.get(&task_id).expect("Task should exist in allocations map");
        assert_eq!(task_state.current_memory, 1024);
        assert_eq!(task_state.peak_memory, 1024);
        assert_eq!(task_state.allocations.len(), 1);

        // Finish task
        let report = tracker.finish_task(task_id).expect("Task should finish successfully");
        assert_eq!(report.task_name, "test_task");
        assert_eq!(report.total_allocations, 1);
        assert_eq!(report.total_allocated, 1024);
    }

    #[test]
    fn test_await_tracking() {
        let tracker = AsyncMemoryTracker::new();
        let task_id = tracker.start_task("await_test".to_string(), None);

        {
            let _await_handle = tracker.start_await(task_id, "test_await".to_string());
            std::thread::sleep(std::time::Duration::from_millis(10));
        } // AwaitHandle drops here

        let metrics = tracker.await_metrics.get("test_await").unwrap();
        assert_eq!(metrics.total_calls, 1);
        assert!(metrics.total_duration.as_millis() >= 10);
    }

    #[tokio::test]
    #[cfg(feature = "async-tokio")]
    async fn test_async_macro_integration() {
        use crate::async_trace;

        let result = async_trace!("./test_analysis", async {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            42
        }).await;

        assert_eq!(result, 42);
    }
}