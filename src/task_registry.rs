//! Task Registry for unified task tracking
//!
//! This module provides a centralized registry for task metadata,
//! enabling task relationship tracking and memory attribution.

use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::RwLock;

// Thread-local storage for current task ID
thread_local! {
    static CURRENT_TASK_ID: Cell<Option<u64>> = const { Cell::new(None) };
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is currently running
    Running,
    /// Task has completed
    Completed,
}

/// Task graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    /// Task ID
    pub id: u64,
    /// Task name
    pub name: String,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Number of allocations
    pub allocation_count: usize,
    /// Task status
    pub status: TaskStatus,
}

/// Task graph edge (parent-child relationship)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEdge {
    /// Parent task ID
    pub from: u64,
    /// Child task ID
    pub to: u64,
}

/// Task graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGraph {
    /// Graph nodes (tasks)
    pub nodes: Vec<TaskNode>,
    /// Graph edges (parent-child relationships)
    pub edges: Vec<TaskEdge>,
}

/// Task metadata
#[derive(Debug, Clone)]
pub struct TaskMeta {
    /// Unique task ID (primary key)
    pub id: u64,
    /// Parent task ID (for hierarchy)
    pub parent: Option<u64>,
    /// Tokio task ID (optional, for async integration)
    pub tokio_id: Option<u64>,
    /// Task name
    pub name: String,
    /// Creation timestamp (nanoseconds)
    pub created_at: u64,
    /// Task status
    pub status: TaskStatus,
    /// Total memory usage in bytes
    pub memory_usage: u64,
    /// Number of allocations
    pub allocation_count: usize,
}

impl TaskMeta {
    /// Create new task metadata
    pub fn new(id: u64, parent: Option<u64>, name: String) -> Self {
        Self {
            id,
            parent,
            tokio_id: None,
            name,
            created_at: Self::now(),
            status: TaskStatus::Running,
            memory_usage: 0,
            allocation_count: 0,
        }
    }

    /// Get current time in nanoseconds
    fn now() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    /// Mark task as completed
    pub fn mark_completed(&mut self) {
        self.status = TaskStatus::Completed;
    }

    /// Record a memory allocation for this task
    pub fn record_allocation(&mut self, size: usize) {
        self.memory_usage += size as u64;
        self.allocation_count += 1;
    }
}

/// Global task ID counter
static TASK_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Global task registry singleton
static GLOBAL_REGISTRY: std::sync::OnceLock<TaskIdRegistry> = std::sync::OnceLock::new();

/// Get the global task registry instance
pub fn global_registry() -> &'static TaskIdRegistry {
    GLOBAL_REGISTRY.get_or_init(TaskIdRegistry::new)
}

/// Generate a new unique task ID with collision detection
///
/// If the generated ID already exists (extremely rare with atomic counter),
/// adds a suffix to make it unique.
pub fn generate_task_id() -> u64 {
    let id = TASK_COUNTER.fetch_add(1, Ordering::Relaxed);

    // In case of collision (extremely rare), add suffix
    // This is a safety measure, not expected to trigger in normal operation
    if id == 0 || id > u64::MAX / 10 {
        // Avoid 0 and reserve high values for suffixed IDs
        TASK_COUNTER.fetch_add(1, Ordering::Relaxed)
    } else {
        id
    }
}

/// Task guard for RAII-style task lifecycle management
///
/// When dropped, automatically completes the task.
pub struct TaskGuard {
    task_id: u64,
}

impl TaskGuard {
    /// Create a new task guard (internal use)
    fn new(task_id: u64) -> Self {
        Self { task_id }
    }
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        global_registry().complete_task(self.task_id);
    }
}

/// Task registry for managing task metadata
pub struct TaskIdRegistry {
    /// Task metadata storage
    tasks: Arc<RwLock<HashMap<u64, TaskMeta>>>,
    /// Set of used task IDs for uniqueness detection
    used_ids: Arc<RwLock<HashSet<u64>>>,
}

impl TaskIdRegistry {
    /// Create new task registry
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            used_ids: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Create a task scope with automatic lifecycle management
    ///
    /// This is the simplified API - just call this and the task is automatically
    /// completed when the guard is dropped.
    ///
    /// # Arguments
    ///
    /// * `name` - Task name
    ///
    /// # Returns
    ///
    /// A TaskGuard that automatically completes the task when dropped
    ///
    /// # Example
    ///
    /// ```rust
    /// # use memscope_rs::task_registry::global_registry;
    /// let registry = global_registry();
    ///
    /// {
    ///     let _main = registry.task_scope("main_process");
    ///     let data = vec![1, 2, 3]; // Automatically attributed to main_process
    ///
    ///     {
    ///         let _worker = registry.task_scope("worker"); // Parent is automatically main_process
    ///         let more_data = vec![4, 5, 6]; // Automatically attributed to worker
    ///     } // worker automatically completed
    /// } // main automatically completed
    ///
    /// let graph = registry.export_graph();
    /// ```
    pub fn task_scope(&self, name: &str) -> TaskGuard {
        let parent = Self::current_task_id();
        let task_id = self.spawn_task(parent, name.to_string());
        TaskGuard::new(task_id)
    }

    /// Spawn a new task (internal use only)
    ///
    /// # Arguments
    ///
    /// * `parent` - Parent task ID (None for root tasks)
    /// * `name` - Task name
    ///
    /// # Returns
    ///
    /// The new task ID
    fn spawn_task(&self, parent: Option<u64>, name: String) -> u64 {
        let mut task_id = generate_task_id();

        // Check for collision and handle with suffix if needed
        if let Ok(used_ids) = self.used_ids.read() {
            while used_ids.contains(&task_id) {
                // Collision detected (extremely rare), use suffix
                // Format: base_id + suffix * 10^9 to avoid overlap
                let base_id = task_id / 1_000_000_000;
                let suffix = (task_id % 1_000_000_000) + 1;
                task_id = base_id * 1_000_000_000 + suffix;
            }
        }

        let mut meta = TaskMeta::new(task_id, parent, name);

        // Try to get tokio task ID if available
        if let Some(tokio_id) = self.get_tokio_task_id() {
            meta.tokio_id = Some(tokio_id);
        }

        // Store task metadata
        if let Ok(mut tasks) = self.tasks.write() {
            tasks.insert(task_id, meta);
        }

        // Register ID as used
        if let Ok(mut used_ids) = self.used_ids.write() {
            used_ids.insert(task_id);
        }

        // Set as current task in thread-local cache
        CURRENT_TASK_ID.set(Some(task_id));

        task_id
    }

    /// Complete a task (internal use only)
    ///
    /// # Arguments
    ///
    /// * `task_id` - Task ID to complete
    fn complete_task(&self, task_id: u64) {
        if let Ok(mut tasks) = self.tasks.write() {
            if let Some(meta) = tasks.get_mut(&task_id) {
                meta.mark_completed();
            }
        }

        // Clear current task from thread-local cache
        CURRENT_TASK_ID.set(None);
    }

    /// Record a memory allocation for the current task
    ///
    /// # Arguments
    ///
    /// * `size` - Size of the allocation in bytes
    pub fn record_allocation(&self, size: usize) {
        if let Some(task_id) = Self::current_task_id() {
            if let Ok(mut tasks) = self.tasks.write() {
                if let Some(meta) = tasks.get_mut(&task_id) {
                    meta.record_allocation(size);
                }
            }
        }
    }

    /// Get current task ID from thread-local cache
    ///
    /// This is a zero-cost operation (no lock required)
    pub fn current_task_id() -> Option<u64> {
        CURRENT_TASK_ID.get()
    }

    /// Clear all tasks (for testing purposes)
    pub fn clear(&self) {
        if let Ok(mut tasks) = self.tasks.write() {
            tasks.clear();
        }
        if let Ok(mut used_ids) = self.used_ids.write() {
            used_ids.clear();
        }
        CURRENT_TASK_ID.set(None);
    }

    /// Get task metadata by ID
    ///
    /// # Arguments
    ///
    /// * `task_id` - Task ID
    ///
    /// # Returns
    ///
    /// Task metadata if found
    pub fn get_task(&self, task_id: u64) -> Option<TaskMeta> {
        if let Ok(tasks) = self.tasks.read() {
            tasks.get(&task_id).cloned()
        } else {
            None
        }
    }

    /// Get all tasks
    pub fn get_all_tasks(&self) -> Vec<TaskMeta> {
        if let Ok(tasks) = self.tasks.read() {
            tasks.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get task children
    ///
    /// # Arguments
    ///
    /// * `parent_id` - Parent task ID
    ///
    /// # Returns
    ///
    /// List of child task IDs
    pub fn get_children(&self, parent_id: u64) -> Vec<u64> {
        if let Ok(tasks) = self.tasks.read() {
            tasks
                .values()
                .filter(|meta| meta.parent == Some(parent_id))
                .map(|meta| meta.id)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get task parent
    ///
    /// # Arguments
    ///
    /// * `task_id` - Task ID
    ///
    /// # Returns
    ///
    /// Parent task ID if found
    pub fn get_parent(&self, task_id: u64) -> Option<u64> {
        if let Ok(tasks) = self.tasks.read() {
            tasks.get(&task_id).and_then(|meta| meta.parent)
        } else {
            None
        }
    }

    /// Get Tokio task ID (if available)
    fn get_tokio_task_id(&self) -> Option<u64> {
        // This will be implemented with tokio integration later
        // For now, return None
        None
    }

    /// Export task graph as JSON
    ///
    /// # Returns
    ///
    /// TaskGraph containing all tasks and their relationships
    pub fn export_graph(&self) -> TaskGraph {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        if let Ok(tasks) = self.tasks.read() {
            // Build nodes
            for meta in tasks.values() {
                nodes.push(TaskNode {
                    id: meta.id,
                    name: meta.name.clone(),
                    memory_usage: meta.memory_usage,
                    allocation_count: meta.allocation_count,
                    status: meta.status,
                });
            }

            // Build edges (parent-child relationships)
            for meta in tasks.values() {
                if let Some(parent_id) = meta.parent {
                    edges.push(TaskEdge {
                        from: parent_id,
                        to: meta.id,
                    });
                }
            }
        }

        TaskGraph { nodes, edges }
    }

    /// Get task statistics
    pub fn get_stats(&self) -> TaskRegistryStats {
        if let Ok(tasks) = self.tasks.read() {
            let total = tasks.len();
            let running = tasks
                .values()
                .filter(|m| m.status == TaskStatus::Running)
                .count();
            let completed = tasks
                .values()
                .filter(|m| m.status == TaskStatus::Completed)
                .count();

            TaskRegistryStats {
                total_tasks: total,
                running_tasks: running,
                completed_tasks: completed,
            }
        } else {
            TaskRegistryStats::default()
        }
    }
}

impl Default for TaskIdRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Task registry statistics
#[derive(Debug, Clone, Default)]
pub struct TaskRegistryStats {
    pub total_tasks: usize,
    pub running_tasks: usize,
    pub completed_tasks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id_generation() {
        let id1 = generate_task_id();
        let id2 = generate_task_id();
        assert!(id2 > id1);
    }

    #[test]
    fn test_spawn_task() {
        let registry = global_registry();
        registry.clear();

        let task_id = registry.spawn_task(None, "test_task".to_string());

        let meta = registry.get_task(task_id);
        assert!(meta.is_some());
        assert_eq!(meta.unwrap().name, "test_task");
    }

    #[test]
    fn test_parent_child() {
        let registry = global_registry();
        registry.clear();

        // Using simplified API
        {
            let _parent = registry.task_scope("parent");
            let parent_id = TaskIdRegistry::current_task_id().unwrap();

            {
                let _child = registry.task_scope("child");
                let child_id = TaskIdRegistry::current_task_id().unwrap();

                assert_eq!(registry.get_parent(child_id), Some(parent_id));
                assert_eq!(registry.get_children(parent_id), vec![child_id]);
            }
        }
    }

    #[test]
    fn test_current_task() {
        let registry = global_registry();
        registry.clear();

        assert_eq!(TaskIdRegistry::current_task_id(), None);

        {
            let _task = registry.task_scope("test");
            let task_id = TaskIdRegistry::current_task_id();
            assert!(task_id.is_some());
        }

        assert_eq!(TaskIdRegistry::current_task_id(), None);
    }

    #[test]
    fn test_complete_task() {
        let registry = global_registry();
        registry.clear();

        let task_id;

        {
            let _task = registry.task_scope("test");
            task_id = TaskIdRegistry::current_task_id().unwrap();

            let meta = registry.get_task(task_id).unwrap();
            assert_eq!(meta.status, TaskStatus::Running);
        }

        // Task should be completed after guard is dropped
        let meta = registry.get_task(task_id).unwrap();
        assert_eq!(meta.status, TaskStatus::Completed);
    }

    #[test]
    fn test_stats() {
        let registry = global_registry();
        registry.clear();

        {
            let _t1 = registry.task_scope("task1");
            let _t2 = registry.task_scope("task2");

            let stats = registry.get_stats();
            assert_eq!(stats.total_tasks, 2);
            assert_eq!(stats.running_tasks, 2);
        }

        let stats = registry.get_stats();
        assert_eq!(stats.completed_tasks, 2);
        assert_eq!(stats.running_tasks, 0);
    }

    #[test]
    fn test_export_graph() {
        let registry = global_registry();
        registry.clear();

        {
            let _parent = registry.task_scope("parent");
            {
                let _child = registry.task_scope("child");
            }
        }

        let graph = registry.export_graph();

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }
}
