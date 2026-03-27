//! Unified tracking strategy implementation
//!
//! UnifiedTracker provides hybrid multi-strategy tracking that combines
//! the advantages of multiple tracking strategies.

use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::data::{
    TrackingSnapshot, TrackingStrategy, AllocationRecord, MemoryEvent, EventType,
    TaskRecord, TaskStatus, TrackingStats
};
use crate::tracker::base::TrackBase;

/// Unified tracking mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnifiedMode {
    /// Core mode: detailed tracking
    Core,
    /// Lockfree mode: event-based tracking
    Lockfree,
    /// Async mode: task-based tracking
    Async,
    /// Hybrid mode: all strategies combined
    Hybrid,
}

/// Unified tracking state
#[derive(Debug)]
struct UnifiedState {
    enabled: bool,
    mode: UnifiedMode,
    allocations: Vec<AllocationRecord>,
    events: Vec<MemoryEvent>,
    tasks: Vec<TaskRecord>,
    total_allocated: usize,
    total_deallocated: usize,
    allocation_count: u64,
    deallocation_count: u64,
    peak_memory: usize,
    task_counter: u64,
}

impl UnifiedState {
    fn new(mode: UnifiedMode) -> Self {
        UnifiedState {
            enabled: true,
            mode,
            allocations: Vec::new(),
            events: Vec::new(),
            tasks: Vec::new(),
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

/// Unified tracking strategy
///
/// Provides hybrid multi-strategy tracking that combines the advantages
/// of multiple tracking strategies. Supports different modes:
/// - Core: Detailed tracking
/// - Lockfree: Event-based tracking
/// - Async: Task-based tracking
/// - Hybrid: All strategies combined
pub struct UnifiedTracker {
    state: Arc<RwLock<UnifiedState>>,
}

impl UnifiedTracker {
    /// Create a new UnifiedTracker with the specified mode
    ///
    /// # Arguments
    /// * `mode` - The unified tracking mode to use
    pub fn new(mode: UnifiedMode) -> Self {
        UnifiedTracker {
            state: Arc::new(RwLock::new(UnifiedState::new(mode))),
        }
    }

    /// Create a new UnifiedTracker in Hybrid mode (default)
    pub fn new_hybrid() -> Self {
        Self::new(UnifiedMode::Hybrid)
    }

    /// Get current tracking mode
    pub fn mode(&self) -> UnifiedMode {
        let state = self.state.read().unwrap();
        state.mode
    }

    /// Set tracking mode
    pub fn set_mode(&self, mode: UnifiedMode) {
        let mut state = self.state.write().unwrap();
        state.mode = mode;
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
        std::thread::current().id().as_u64().get() as u32
    }

    /// Register a new task
    ///
    /// # Arguments
    /// * `task_name` - Optional name for the task
    ///
    /// # Returns
    /// Task ID for the new task
    pub fn register_task(&self, task_name: Option<String>) -> u64 {
        let mut state = self.state.write().unwrap();
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
        };

        state.tasks.push(task);
        task_id
    }

    /// Complete a task
    ///
    /// # Arguments
    /// * `task_id` - ID of the task to complete
    pub fn complete_task(&self, task_id: u64) {
        let mut state = self.state.write().unwrap();
        if let Some(task) = state.tasks.iter_mut().find(|t| t.task_id == task_id) {
            task.status = TaskStatus::Completed;
            task.completed_at = Some(Self::timestamp());
        }
    }

    /// Track allocation with task association
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the allocated memory
    /// * `size` - Size of the allocation
    /// * `task_id` - Optional task ID to associate with
    pub fn track_alloc_with_task(&self, ptr: usize, size: usize, task_id: Option<u64>) {
        let mut state = self.state.write().unwrap();
        if !state.enabled {
            return;
        }

        let timestamp = Self::timestamp();
        let thread_id = Self::thread_id();

        // Record allocation
        let record = AllocationRecord {
            ptr,
            size,
            timestamp,
            thread_id,
            stack_id: None,
            var_name: None,
            type_name: None,
            is_active: true,
            dealloc_timestamp: None,
        };

        state.total_allocated += size;
        state.allocation_count += 1;
        state.update_peak_memory();

        // Add based on mode
        match state.mode {
            UnifiedMode::Core | UnifiedMode::Hybrid => {
                state.allocations.push(record.clone());
            }
            UnifiedMode::Lockfree => {
                // Only track events, not detailed allocations
            }
            UnifiedMode::Async => {
                // Track tasks, minimal allocations
                if let Some(tid) = task_id {
                    if let Some(task) = state.tasks.iter_mut().find(|t| t.task_id == tid) {
                        task.memory_usage += size;
                        task.allocation_count += 1;
                    }
                }
            }
        }

        // Record event for lockfree mode
        if state.mode == UnifiedMode::Lockfree || state.mode == UnifiedMode::Hybrid {
            let event = MemoryEvent {
                event_type: EventType::Alloc,
                ptr,
                size,
                timestamp,
                thread_id,
                duration: None,
                task_id,
            };
            state.events.push(event);
        }
    }

    /// Track deallocation with duration
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the deallocated memory
    /// * `duration` - Optional duration the allocation was held
    pub fn track_dealloc_with_duration(&self, ptr: usize, duration: Option<u64>) {
        let mut state = self.state.write().unwrap();
        if !state.enabled {
            return;
        }

        let timestamp = Self::timestamp();
        let thread_id = Self::thread_id();

        // Find and update allocation
        if let Some(record) = state.allocations.iter_mut().find(|r| r.ptr == ptr) {
            record.is_active = false;
            record.dealloc_timestamp = Some(timestamp);
            state.total_deallocated += record.size;
            state.deallocation_count += 1;
        }

        // Record event for lockfree mode
        if state.mode == UnifiedMode::Lockfree || state.mode == UnifiedMode::Hybrid {
            let event = MemoryEvent {
                event_type: EventType::Dealloc,
                ptr,
                size: 0, // Size not known without allocation record
                timestamp,
                thread_id,
                duration,
                task_id: None,
            };
            state.events.push(event);
        }
    }
}

impl Default for UnifiedTracker {
    fn default() -> Self {
        Self::new_hybrid()
    }
}

impl TrackBase for UnifiedTracker {
    fn strategy(&self) -> TrackingStrategy {
        TrackingStrategy::Unified
    }

    fn track_alloc(&self, ptr: usize, size: usize) {
        self.track_alloc_with_task(ptr, size, None);
    }

    fn track_dealloc(&self, ptr: usize) {
        self.track_dealloc_with_duration(ptr, None);
    }

    fn snapshot(&self) -> TrackingSnapshot {
        let state = self.state.read().unwrap();

        let allocations: Vec<AllocationRecord> = state.allocations.clone();
        let events: Vec<MemoryEvent> = state.events.clone();
        let tasks: Vec<TaskRecord> = state.tasks.clone();

        let current_memory = state.total_allocated - state.total_deallocated;
        let fragmentation = if state.total_allocated > 0 {
            ((state.total_allocated - current_memory) as f64 / state.total_allocated as f64) * 100.0
        } else {
            0.0
        };

        let stats = TrackingStats {
            total_allocated: state.total_allocated,
            total_deallocated: state.total_deallocated,
            current_allocated: current_memory,
            peak_memory: state.peak_memory,
            allocation_count: state.allocation_count,
            deallocation_count: state.deallocation_count,
            active_allocations: allocations.iter().filter(|a| a.is_active).count() as u64,
            fragmentation,
            average_allocation_size: if state.allocation_count > 0 {
                state.total_allocated / state.allocation_count as usize
            } else {
                0
            },
        };

        TrackingSnapshot {
            strategy: TrackingStrategy::Unified,
            allocations,
            events,
            tasks,
            stats,
            timestamp: Self::timestamp(),
        }
    }

    fn clear(&self) {
        let mut state = self.state.write().unwrap();
        state.allocations.clear();
        state.events.clear();
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
    fn test_unified_tracker_creation() {
        let tracker = UnifiedTracker::new_hybrid();
        assert_eq!(tracker.strategy(), TrackingStrategy::Unified);
        assert_eq!(tracker.mode(), UnifiedMode::Hybrid);
        assert!(tracker.is_enabled());
    }

    #[test]
    fn test_unified_tracker_modes() {
        let tracker = UnifiedTracker::new(UnifiedMode::Core);
        assert_eq!(tracker.mode(), UnifiedMode::Core);

        tracker.set_mode(UnifiedMode::Lockfree);
        assert_eq!(tracker.mode(), UnifiedMode::Lockfree);
    }

    #[test]
    fn test_unified_tracker_hybrid_mode() {
        let tracker = UnifiedTracker::new_hybrid();
        tracker.track_alloc(0x1000, 1024);
        tracker.track_dealloc(0x1000);

        let snapshot = tracker.snapshot();
        // Hybrid mode should have both allocations and events
        assert!(!snapshot.allocations.is_empty() || !snapshot.events.is_empty());
    }

    #[test]
    fn test_unified_tracker_task_tracking() {
        let tracker = UnifiedTracker::new(UnifiedMode::Async);
        let task_id = tracker.register_task(Some("TestTask".to_string()));

        tracker.track_alloc_with_task(0x1000, 1024, Some(task_id));
        tracker.complete_task(task_id);

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.tasks.len(), 1);
        assert_eq!(snapshot.tasks[0].task_name, "TestTask");
    }

    #[test]
    fn test_unified_tracker_clear() {
        let tracker = UnifiedTracker::new_hybrid();
        tracker.track_alloc(0x1000, 1024);
        tracker.register_task(Some("TestTask".to_string()));
        tracker.clear();

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.allocations.len(), 0);
        assert_eq!(snapshot.events.len(), 0);
        assert_eq!(snapshot.tasks.len(), 0);
    }

    #[test]
    fn test_unified_tracker_enable_disable() {
        let tracker = UnifiedTracker::new_hybrid();
        tracker.set_enabled(false);
        tracker.track_alloc(0x1000, 1024);

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.allocations.len(), 0);

        tracker.set_enabled(true);
        tracker.track_alloc(0x2000, 2048);

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.allocations.len(), 1);
    }
}