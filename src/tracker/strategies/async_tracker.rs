//! Async tracking strategy implementation
//!
//! AsyncTracker provides task-level async memory tracking that tracks
//! memory usage across async tasks.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::data::{
    AllocationRecord, TaskRecord, TaskStatus, TrackingSnapshot, TrackingStats, TrackingStrategy,
};
use crate::tracker::base::TrackBase;

/// Async tracking state
#[derive(Debug)]
struct AsyncState {
    enabled: bool,
    tasks: HashMap<u64, TaskRecord>,
    allocations: HashMap<usize, AllocationRecord>,
    total_allocated: usize,
    total_deallocated: usize,
    allocation_count: u64,
    deallocation_count: u64,
    peak_memory: usize,
    task_counter: u64,
}

impl AsyncState {
    fn new() -> Self {
        AsyncState {
            enabled: true,
            tasks: HashMap::new(),
            allocations: HashMap::new(),
            total_allocated: 0,
            total_deallocated: 0,
            allocation_count: 0,
            deallocation_count: 0,
            peak_memory: 0,
            task_counter: 0,
        }
    }

    fn update_peak_memory(&mut self) {
        let current_memory = self.total_allocated - self.total_deallocated;
        if current_memory > self.peak_memory {
            self.peak_memory = current_memory;
        }
    }
}

/// Async tracking strategy
///
/// Provides task-level async memory tracking that tracks memory usage
/// across async tasks. Associates allocations with specific tasks.
pub struct AsyncTracker {
    state: Arc<RwLock<AsyncState>>,
}

impl AsyncTracker {
    /// Create a new AsyncTracker
    pub fn new() -> Self {
        AsyncTracker {
            state: Arc::new(RwLock::new(AsyncState::new())),
        }
    }

    /// Get current timestamp in microseconds
    fn timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }

    /// Get current thread ID
    fn thread_id() -> u32 {
        use std::hash::{Hash, Hasher};
        let thread_id = std::thread::current().id();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        thread_id.hash(&mut hasher);
        (hasher.finish() & 0xFFFFFFFF) as u32
    }

    /// Register a new task
    ///
    /// # Arguments
    /// * `task_name` - Optional name for the task
    ///
    /// # Returns
    /// Task ID for the new task, or 0 if tracking is disabled
    pub fn register_task(&self, task_name: Option<String>) -> u64 {
        let mut state = self.state.write().unwrap();
        if !state.enabled {
            return 0;
        }

        let task_id = state.task_counter;
        state.task_counter += 1;

        let task = TaskRecord {
            task_id,
            task_name: task_name.unwrap_or(format!("Task-{}", task_id)),
            status: TaskStatus::Running,
            created_at: Self::timestamp(),
            completed_at: None,
            memory_usage: 0,
            allocation_count: 0,
            deallocation_count: 0,
            peak_memory: 0,
        };

        state.tasks.insert(task_id, task);
        task_id
    }

    /// Complete a task
    ///
    /// # Arguments
    /// * `task_id` - ID of the task to complete
    pub fn complete_task(&self, task_id: u64) {
        let mut state = self.state.write().unwrap();
        if !state.enabled {
            return;
        }

        if let Some(task) = state.tasks.get_mut(&task_id) {
            task.status = TaskStatus::Completed;
            task.completed_at = Some(Self::timestamp());
        }
    }

    /// Associate allocation with task
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the allocated memory
    /// * `size` - Size of the allocation
    /// * `task_id` - Task ID to associate with
    pub fn track_task_alloc(&self, ptr: usize, size: usize, task_id: u64) {
        let mut state = self.state.write().unwrap();
        if !state.enabled {
            return;
        }

        let record = AllocationRecord {
            ptr,
            size,
            timestamp: Self::timestamp(),
            thread_id: Self::thread_id(),
            stack_id: None,
            var_name: None,
            type_name: None,
            is_active: true,
            dealloc_timestamp: None,
            is_leaked: false,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
        };

        state.total_allocated += size;
        state.allocation_count += 1;
        state.update_peak_memory();
        state.allocations.insert(ptr, record);

        // Update task statistics
        if let Some(task) = state.tasks.get_mut(&task_id) {
            task.memory_usage += size;
            task.allocation_count += 1;
        }
    }
}

impl Default for AsyncTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackBase for AsyncTracker {
    fn strategy(&self) -> TrackingStrategy {
        TrackingStrategy::Async
    }

    fn track_alloc(&self, ptr: usize, size: usize) {
        // For async tracking, we recommend using track_task_alloc
        // to associate allocations with specific tasks
        let mut state = self.state.write().unwrap();
        if !state.enabled {
            return;
        }

        let record = AllocationRecord {
            ptr,
            size,
            timestamp: Self::timestamp(),
            thread_id: Self::thread_id(),
            stack_id: None,
            var_name: None,
            type_name: None,
            is_active: true,
            dealloc_timestamp: None,
            is_leaked: false,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
        };

        state.total_allocated += size;
        state.allocation_count += 1;
        state.update_peak_memory();
        state.allocations.insert(ptr, record);
    }

    fn track_dealloc(&self, ptr: usize) {
        let mut state = self.state.write().unwrap();
        if !state.enabled {
            return;
        }

        if let Some(record) = state.allocations.remove(&ptr) {
            state.total_deallocated += record.size;
            state.deallocation_count += 1;

            // Update task statistics
            // Note: In a real implementation, we'd track which task owns each allocation
        }
    }

    fn snapshot(&self) -> TrackingSnapshot {
        let state = self.state.read().unwrap();
        let allocations: Vec<AllocationRecord> = state.allocations.values().cloned().collect();
        let tasks: Vec<TaskRecord> = state.tasks.values().cloned().collect();

        let current_memory = state.total_allocated - state.total_deallocated;
        let fragmentation = if state.total_allocated > 0 {
            ((state.total_allocated - current_memory) as f64 / state.total_allocated as f64) * 100.0
        } else {
            0.0
        };

        let stats = TrackingStats {
            total_allocations: state.allocation_count,
            total_deallocations: state.deallocation_count,
            total_allocated: state.total_allocated as u64,
            total_deallocated: state.total_deallocated as u64,
            peak_memory: state.peak_memory as u64,
            active_allocations: state.allocations.len() as u64,
            active_memory: current_memory as u64,
            leaked_allocations: state.allocations.iter().filter(|a| a.1.is_leaked).count() as u64,
            leaked_memory: state
                .allocations
                .iter()
                .filter(|a| a.1.is_leaked)
                .map(|a| a.1.size as u64)
                .sum(),
            fragmentation_ratio: fragmentation,
            allocation_count: state.allocation_count,
            deallocation_count: state.deallocation_count,
            average_allocation_size: if state.allocation_count > 0 {
                state.total_allocated / state.allocation_count as usize
            } else {
                0
            },
            current_allocated: current_memory,
            fragmentation,
        };

        TrackingSnapshot {
            strategy: TrackingStrategy::Async,
            allocations,
            events: vec![], // Async doesn't track events
            tasks,
            stats,
            timestamp: Self::timestamp(),
        }
    }

    fn clear(&self) {
        let mut state = self.state.write().unwrap();
        state.allocations.clear();
        state.tasks.clear();
        state.total_allocated = 0;
        state.total_deallocated = 0;
        state.allocation_count = 0;
        state.deallocation_count = 0;
        state.peak_memory = 0;
        state.task_counter = 0;
    }

    fn set_enabled(&self, enabled: bool) {
        let mut state = self.state.write().unwrap();
        state.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        let state = self.state.read().unwrap();
        state.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_tracker_creation() {
        let tracker = AsyncTracker::new();
        assert_eq!(tracker.strategy(), TrackingStrategy::Async);
        assert!(tracker.is_enabled());
    }

    #[test]
    fn test_async_tracker_task_tracking() {
        let tracker = AsyncTracker::new();
        let task_id = tracker.register_task(Some("TestTask".to_string()));

        tracker.track_task_alloc(0x1000, 1024, task_id);
        tracker.complete_task(task_id);

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.tasks.len(), 1);
        assert_eq!(snapshot.tasks[0].task_name, "TestTask");
        assert_eq!(snapshot.tasks[0].allocation_count, 1);
        assert_eq!(snapshot.tasks[0].memory_usage, 1024);
    }

    #[test]
    fn test_async_tracker_alloc_dealloc() {
        let tracker = AsyncTracker::new();
        tracker.track_alloc(0x1000, 1024);
        tracker.track_alloc(0x2000, 2048);
        tracker.track_dealloc(0x1000);

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.allocations.len(), 1);
        assert_eq!(snapshot.stats.allocation_count, 2);
        assert_eq!(snapshot.stats.deallocation_count, 1);
    }

    #[test]
    fn test_async_tracker_clear() {
        let tracker = AsyncTracker::new();
        tracker.register_task(Some("TestTask".to_string()));
        tracker.track_alloc(0x1000, 1024);
        tracker.clear();

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.allocations.len(), 0);
        assert_eq!(snapshot.tasks.len(), 0);
        assert_eq!(snapshot.stats.allocation_count, 0);
    }

    #[test]
    fn test_async_tracker_enable_disable() {
        let tracker = AsyncTracker::new();
        tracker.set_enabled(false);
        tracker.register_task(Some("TestTask".to_string()));
        tracker.track_alloc(0x1000, 1024);

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.allocations.len(), 0);
        assert_eq!(snapshot.tasks.len(), 0);

        tracker.set_enabled(true);
        let _task_id = tracker.register_task(Some("TestTask2".to_string()));
        tracker.track_alloc(0x2000, 2048);

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.allocations.len(), 1);
        assert_eq!(snapshot.tasks.len(), 1);
    }
}
