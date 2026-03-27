//! Async adapter for bridging old async memory system to new AsyncTracker
//!
//! This adapter maintains backward compatibility with the old async tracking API
//! while internally using the new unified tracking system.

use crate::manager::TrackingManager;
use crate::data::TaskRecord;
use crate::tracker::base::TrackBase;
use crate::tracker::strategies::AsyncTracker;
use std::sync::Arc;

/// Async adapter for task-level memory tracking
///
/// This adapter bridges the old async tracking API to the new unified tracking system,
/// providing compatibility for legacy async tracking features.
pub struct AsyncAdapter {
    /// New unified tracking manager
    manager: Arc<TrackingManager>,
    
    /// Direct access to AsyncTracker for task-specific operations
    async_tracker: Arc<AsyncTracker>,
}

impl AsyncAdapter {
    /// Create a new async adapter
    pub fn new() -> Self {
        let manager = Arc::new(TrackingManager::new_async());
        
        // Get direct access to AsyncTracker for task operations
        // Note: This requires accessing the internal implementation
        // For now, we'll create a separate AsyncTracker instance
        let async_tracker = Arc::new(AsyncTracker::new());

        Self {
            manager,
            async_tracker,
        }
    }

    /// Get the underlying tracking manager
    pub fn manager(&self) -> &Arc<TrackingManager> {
        &self.manager
    }

    /// Register a new task
    pub fn register_task(&self, task_name: Option<String>) -> u64 {
        self.async_tracker.register_task(task_name)
    }

    /// Complete a task
    pub fn complete_task(&self, task_id: u64) {
        self.async_tracker.complete_task(task_id);
    }

    /// Track allocation associated with a task
    pub fn track_task_alloc(&self, ptr: usize, size: usize, task_id: u64) {
        self.async_tracker.track_task_alloc(ptr, size, task_id);
    }

    /// Track a deallocation
    pub fn track_dealloc(&self, ptr: usize) {
        self.manager.track_dealloc(ptr);
    }

    /// Get current snapshot
    pub fn snapshot(&self) -> crate::data::TrackingSnapshot {
        let mut snapshot = self.manager.snapshot();
        
        // Get task data from the internal async tracker
        let async_snapshot = self.async_tracker.snapshot();
        
        // Merge task data into the snapshot
        snapshot.tasks = async_snapshot.tasks;
        
        snapshot
    }
}

impl Default for AsyncAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_adapter_creation() {
        let adapter = AsyncAdapter::new();
        assert!(adapter.manager().is_enabled());
    }

    #[test]
    fn test_async_task_tracking() {
        let adapter = AsyncAdapter::new();
        let task_id = adapter.register_task(Some("test_task".to_string()));
        
        adapter.track_task_alloc(0x1000, 1024, task_id);
        adapter.complete_task(task_id);

        let snapshot = adapter.snapshot();
        assert_eq!(snapshot.tasks.len(), 1);
    }
}