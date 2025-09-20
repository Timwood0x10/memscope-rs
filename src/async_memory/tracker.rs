//! Task memory tracker for aggregating and analyzing allocation data
//!
//! Provides the core tracking infrastructure that collects allocation events
//! from buffers and maintains task-level memory profiles.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::async_memory::buffer::{collect_all_events, AllocationEvent};
use crate::async_memory::error::{AsyncError, AsyncResult};
use crate::async_memory::profile::{TaskMemoryProfile, AggregatedTaskStats};
use crate::async_memory::TaskId;

/// Re-export TrackedFuture from api module
pub use crate::async_memory::api::TrackedFuture;

/// Central task memory tracker
///
/// Aggregates allocation events from thread-local buffers and maintains
/// per-task memory profiles for analysis and reporting.
pub struct TaskMemoryTracker {
    /// Task profiles indexed by task ID
    profiles: HashMap<TaskId, TaskMemoryProfile>,
    /// Aggregated statistics across all tasks
    aggregated_stats: AggregatedTaskStats,
    /// Whether the tracker is currently active
    is_active: Arc<AtomicBool>,
    /// Background aggregation thread handle
    aggregator_handle: Option<thread::JoinHandle<()>>,
}

impl TaskMemoryTracker {
    /// Create new task memory tracker
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            aggregated_stats: AggregatedTaskStats::new(),
            is_active: Arc::new(AtomicBool::new(false)),
            aggregator_handle: None,
        }
    }

    /// Start background aggregation thread
    pub fn start(&mut self) -> AsyncResult<()> {
        if self.is_active.load(Ordering::Relaxed) {
            return Err(AsyncError::initialization(
                "tracker",
                "Task memory tracker already started",
                false,
            ));
        }

        self.is_active.store(true, Ordering::Relaxed);
        let is_active = Arc::clone(&self.is_active);

        let handle = thread::Builder::new()
            .name("async-memory-aggregator".to_string())
            .spawn(move || {
                Self::aggregator_thread_main(is_active);
            })
            .map_err(|e| {
                AsyncError::system(
                    "thread_spawn",
                    "Failed to start aggregator thread",
                    Some(&e.to_string()),
                )
            })?;

        self.aggregator_handle = Some(handle);
        tracing::info!("Task memory tracker started");
        Ok(())
    }

    /// Stop background aggregation thread
    pub fn stop(&mut self) -> AsyncResult<()> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Ok(()); // Already stopped
        }

        self.is_active.store(false, Ordering::Relaxed);

        if let Some(handle) = self.aggregator_handle.take() {
            handle.join().map_err(|_| {
                AsyncError::system(
                    "thread_join",
                    "Failed to join aggregator thread",
                    None,
                )
            })?;
        }

        tracing::info!("Task memory tracker stopped");
        Ok(())
    }

    /// Process allocation events and update profiles
    pub fn process_events(&mut self, events: Vec<AllocationEvent>) {
        for event in events {
            self.process_single_event(event);
        }
    }

    /// Process a single allocation event
    fn process_single_event(&mut self, event: AllocationEvent) {
        let profile = self.profiles
            .entry(event.task_id)
            .or_insert_with(|| TaskMemoryProfile::new(event.task_id));

        if event.is_allocation() {
            profile.record_allocation(event.size as u64);
        } else if event.is_deallocation() {
            profile.record_deallocation(event.size as u64);
        }

        // Update aggregated stats (simplified)
        self.update_aggregated_stats();
    }

    /// Update aggregated statistics
    fn update_aggregated_stats(&mut self) {
        self.aggregated_stats = AggregatedTaskStats::new();
        for profile in self.profiles.values() {
            self.aggregated_stats.add_task(profile);
        }
    }

    /// Get task profile by ID
    pub fn get_task_profile(&self, task_id: TaskId) -> Option<&TaskMemoryProfile> {
        self.profiles.get(&task_id)
    }

    /// Get all task profiles
    pub fn get_all_profiles(&self) -> &HashMap<TaskId, TaskMemoryProfile> {
        &self.profiles
    }

    /// Get aggregated statistics
    pub fn get_aggregated_stats(&self) -> &AggregatedTaskStats {
        &self.aggregated_stats
    }

    /// Mark task as completed
    pub fn mark_task_completed(&mut self, task_id: TaskId) {
        if let Some(profile) = self.profiles.get_mut(&task_id) {
            profile.mark_completed();
            self.update_aggregated_stats();
        }
    }

    /// Clean up completed tasks (keep only recent ones)
    pub fn cleanup_completed_tasks(&mut self, max_completed: usize) {
        let mut completed_tasks: Vec<_> = self.profiles
            .iter()
            .filter(|(_, profile)| profile.is_completed())
            .map(|(&id, profile)| (id, profile.completed_at.unwrap_or(profile.created_at)))
            .collect();

        if completed_tasks.len() <= max_completed {
            return; // No cleanup needed
        }

        // Sort by completion time (oldest first)
        completed_tasks.sort_by_key(|(_, completion_time)| *completion_time);

        // Remove oldest completed tasks
        let to_remove = completed_tasks.len() - max_completed;
        for i in 0..to_remove {
            let task_id = completed_tasks[i].0;
            self.profiles.remove(&task_id);
        }

        self.update_aggregated_stats();
        tracing::debug!("Cleaned up {} completed tasks", to_remove);
    }

    /// Background aggregator thread main loop
    fn aggregator_thread_main(is_active: Arc<AtomicBool>) {
        tracing::info!("Task memory aggregator thread started");

        while is_active.load(Ordering::Relaxed) {
            // Collect events from all thread buffers
            let events = collect_all_events();

            if !events.is_empty() {
                tracing::debug!("Collected {} allocation events", events.len());
                // TODO: Process events with global tracker instance
                // For now, just log the collection
            }

            // Sleep briefly to avoid busy waiting
            thread::sleep(Duration::from_millis(10));
        }

        tracing::info!("Task memory aggregator thread stopped");
    }
}

impl Default for TaskMemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TaskMemoryTracker {
    fn drop(&mut self) {
        // Ensure background thread is stopped
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::async_memory::buffer::AllocationEvent;

    #[test]
    fn test_tracker_creation() {
        let tracker = TaskMemoryTracker::new();
        assert_eq!(tracker.profiles.len(), 0);
        assert_eq!(tracker.aggregated_stats.total_tasks, 0);
    }

    #[test]
    fn test_event_processing() {
        let mut tracker = TaskMemoryTracker::new();

        let events = vec![
            AllocationEvent::allocation(1, 0x1000, 1024, 100),
            AllocationEvent::allocation(1, 0x2000, 2048, 200),
            AllocationEvent::deallocation(1, 0x1000, 1024, 300),
            AllocationEvent::allocation(2, 0x3000, 512, 400),
        ];

        tracker.process_events(events);

        // Check task 1 profile
        let profile1 = tracker.get_task_profile(1).expect("Task 1 profile not found");
        assert_eq!(profile1.total_allocated, 3072); // 1024 + 2048
        assert_eq!(profile1.current_usage, 2048);   // 3072 - 1024
        assert_eq!(profile1.allocation_count, 2);
        assert_eq!(profile1.deallocation_count, 1);

        // Check task 2 profile
        let profile2 = tracker.get_task_profile(2).expect("Task 2 profile not found");
        assert_eq!(profile2.total_allocated, 512);
        assert_eq!(profile2.current_usage, 512);
        assert_eq!(profile2.allocation_count, 1);
        assert_eq!(profile2.deallocation_count, 0);

        // Check aggregated stats
        let stats = tracker.get_aggregated_stats();
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.total_memory_allocated, 3584); // 3072 + 512
        assert_eq!(stats.current_memory_usage, 2560);   // 2048 + 512
    }

    #[test]
    fn test_task_completion() {
        let mut tracker = TaskMemoryTracker::new();

        let events = vec![
            AllocationEvent::allocation(1, 0x1000, 1024, 100),
        ];
        tracker.process_events(events);

        // Initially not completed
        let profile = tracker.get_task_profile(1).unwrap();
        assert!(!profile.is_completed());
        assert_eq!(tracker.get_aggregated_stats().completed_tasks, 0);

        // Mark as completed
        tracker.mark_task_completed(1);
        let profile = tracker.get_task_profile(1).unwrap();
        assert!(profile.is_completed());
        assert_eq!(tracker.get_aggregated_stats().completed_tasks, 1);
    }

    #[test]
    fn test_cleanup_completed_tasks() {
        let mut tracker = TaskMemoryTracker::new();

        // Create multiple completed tasks
        for i in 1..=10 {
            let events = vec![AllocationEvent::allocation(i as TaskId, 0x1000 + i as usize, 1024, (100 + i) as u64)];
            tracker.process_events(events);
            tracker.mark_task_completed(i);
        }

        assert_eq!(tracker.profiles.len(), 10);
        assert_eq!(tracker.get_aggregated_stats().completed_tasks, 10);

        // Cleanup keeping only 5 most recent
        tracker.cleanup_completed_tasks(5);
        assert_eq!(tracker.profiles.len(), 5);
        assert_eq!(tracker.get_aggregated_stats().completed_tasks, 5);

        // Verify we kept the most recent ones (6-10)
        for i in 6..=10 {
            assert!(tracker.get_task_profile(i).is_some());
        }
        for i in 1..=5 {
            assert!(tracker.get_task_profile(i).is_none());
        }
    }

    #[test]
    fn test_tracker_lifecycle() {
        let mut tracker = TaskMemoryTracker::new();

        // Start tracker
        tracker.start().expect("Failed to start tracker");
        assert!(tracker.is_active.load(Ordering::Relaxed));
        assert!(tracker.aggregator_handle.is_some());

        // Should not be able to start again
        let result = tracker.start();
        assert!(result.is_err());

        // Stop tracker
        tracker.stop().expect("Failed to stop tracker");
        assert!(!tracker.is_active.load(Ordering::Relaxed));
        assert!(tracker.aggregator_handle.is_none());

        // Should be able to stop again without error
        tracker.stop().expect("Failed to stop tracker second time");
    }

    #[test]
    fn test_multiple_tasks_aggregation() {
        let mut tracker = TaskMemoryTracker::new();

        // Create events for multiple tasks with different patterns
        let mut events = Vec::new();
        
        // Task 1: Large allocations
        events.push(AllocationEvent::allocation(1, 0x1000, 10_000, 100));
        events.push(AllocationEvent::allocation(1, 0x2000, 20_000, 200));
        
        // Task 2: Small frequent allocations
        for i in 0..10 {
            events.push(AllocationEvent::allocation(2, 0x3000 + i, 100, (300 + i) as u64));
        }
        
        // Task 3: Allocation and immediate deallocation
        events.push(AllocationEvent::allocation(3, 0x4000, 5000, 400));
        events.push(AllocationEvent::deallocation(3, 0x4000, 5000, 500));

        tracker.process_events(events);

        // Verify individual profiles
        let profile1 = tracker.get_task_profile(1).unwrap();
        assert_eq!(profile1.total_allocated, 30_000);
        assert_eq!(profile1.allocation_count, 2);

        let profile2 = tracker.get_task_profile(2).unwrap();
        assert_eq!(profile2.total_allocated, 1000); // 10 * 100
        assert_eq!(profile2.allocation_count, 10);

        let profile3 = tracker.get_task_profile(3).unwrap();
        assert_eq!(profile3.total_allocated, 5000);
        assert_eq!(profile3.current_usage, 0); // Fully deallocated
        assert_eq!(profile3.memory_efficiency(), 1.0);

        // Verify aggregated stats
        let stats = tracker.get_aggregated_stats();
        assert_eq!(stats.total_tasks, 3);
        assert_eq!(stats.total_memory_allocated, 36_000); // 30k + 1k + 5k
    }
}