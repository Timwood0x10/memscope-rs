//! Task record structures
//!
//! Used by Async strategy to track task-level memory usage

use super::common::current_timestamp;
use serde::{Deserialize, Serialize};

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is running
    Running,
    /// Task is completed
    Completed,
    /// Task is cancelled
    Cancelled,
}

/// Async task memory record
///
/// Used by Async strategy to track task-level memory usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    /// Task ID
    pub task_id: u64,
    /// Task name
    pub task_name: String,
    /// Task status
    pub status: TaskStatus,
    /// Task creation timestamp
    pub created_at: u64,
    /// Task completion timestamp (if completed)
    pub completed_at: Option<u64>,
    /// Total memory usage
    pub memory_usage: usize,
    /// Allocation count
    pub allocation_count: u64,
    /// Deallocation count
    pub deallocation_count: u64,
    /// peak memory
    pub peak_memory: u64,
}

impl TaskRecord {
    /// Create new task record
    pub fn new(task_id: u64, task_name: String) -> Self {
        Self {
            task_id,
            task_name,
            status: TaskStatus::Running,
            created_at: current_timestamp(),
            completed_at: None,
            memory_usage: 0,
            allocation_count: 0,
            deallocation_count: 0,
            peak_memory: 0,
        }
    }

    /// Record an allocation
    pub fn record_allocation(&mut self, size: usize) {
        self.allocation_count += 1;
        self.memory_usage += size;
        if self.memory_usage > self.peak_memory as usize {
            self.peak_memory = self.memory_usage as u64;
        }
    }

    /// Record a deallocation
    pub fn record_deallocation(&mut self, size: usize) {
        self.deallocation_count += 1;
        self.memory_usage = self.memory_usage.saturating_sub(size);
    }

    /// Mark task as completed
    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(current_timestamp());
    }

    /// Mark task as cancelled
    pub fn cancel(&mut self) {
        self.status = TaskStatus::Cancelled;
        self.completed_at = Some(current_timestamp());
    }

    /// Get task lifetime in milliseconds
    pub fn lifetime_ms(&self) -> Option<u64> {
        self.completed_at.map(|end| (end - self.created_at) / 1_000)
    }

    /// Check if task has potential memory leak
    pub fn has_leak(&self) -> bool {
        self.status == TaskStatus::Completed && self.memory_usage > 0
    }

    /// Get memory efficiency (1.0 - leaked/allocated)
    pub fn memory_efficiency(&self) -> f64 {
        if self.allocation_count == 0 {
            1.0
        } else {
            let total_allocated = self.allocation_count as f64;
            let active_allocations = (self.allocation_count - self.deallocation_count) as f64;
            (total_allocated - active_allocations) / total_allocated
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_record_new() {
        let task = TaskRecord::new(1, "TestTask".to_string());
        assert_eq!(task.task_id, 1);
        assert_eq!(task.task_name, "TestTask");
        assert_eq!(task.status, TaskStatus::Running);
        assert_eq!(task.allocation_count, 0);
        assert!(!task.has_leak());
    }

    #[test]
    fn test_task_record_allocations() {
        let mut task = TaskRecord::new(1, "TestTask".to_string());

        task.record_allocation(1024);
        task.record_allocation(2048);

        assert_eq!(task.allocation_count, 2);
        assert_eq!(task.memory_usage, 3072);
    }

    #[test]
    fn test_task_record_deallocations() {
        let mut task = TaskRecord::new(1, "TestTask".to_string());

        task.record_allocation(1024);
        task.record_allocation(2048);
        task.record_deallocation(1024);

        assert_eq!(task.memory_usage, 2048);
        assert_eq!(task.deallocation_count, 1);
    }

    #[test]
    fn test_task_complete() {
        let mut task = TaskRecord::new(1, "TestTask".to_string());

        task.record_allocation(1024);
        task.record_allocation(2048);

        assert!(!task.has_leak());

        task.complete();
        assert!(task.has_leak());
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[test]
    fn test_task_memory_efficiency() {
        let mut task = TaskRecord::new(1, "TestTask".to_string());

        task.record_allocation(1024);
        task.record_allocation(2048);
        task.record_deallocation(1024);

        let efficiency = task.memory_efficiency();
        assert!(efficiency > 0.0 && efficiency <= 1.0);
    }

    #[test]
    fn test_task_lifetime() {
        let mut task = TaskRecord::new(1, "TestTask".to_string());
        assert!(task.lifetime_ms().is_none());

        std::thread::sleep(std::time::Duration::from_millis(10));
        task.complete();

        let lifetime = task.lifetime_ms();
        assert!(lifetime.is_some());
        assert!(lifetime.unwrap() >= 10);
    }
}
