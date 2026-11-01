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

/// Per-task accounting derived from the event stream.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct TaskLedgerStats {
    /// Allocation events attributed to the task.
    pub allocation_count: u64,
    /// Bytes of those allocations.
    pub allocated_bytes: u64,
    /// Release events attributed to the task.
    pub deallocation_count: u64,
    /// Bytes of those releases.
    pub deallocated_bytes: u64,
    /// Live bytes still attributed to the task at the cut.
    pub live_bytes: u64,
}

/// Result of charging an allocation to the current task.
///
/// Three states that used to be one boolean: an allocation can be charged, belong to no
/// task at all, or be deferred because the registry lock was busy. A report must be able
/// to tell them apart (plan A08).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargeOutcome {
    /// The task was charged, `memory_usage` includes this block.
    Charged,
    /// No task context was installed; the block belongs to no task.
    Unattributed,
    /// The registry lock was busy, so the charge was skipped and counted. The affected
    /// task's metrics are a lower bound for this capture window.
    Deferred,
}

/// How complete the task accounting of the current capture window is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct ChargeQuality {
    /// Charges skipped because the registry was busy.
    pub deferred_charges: u64,
    /// Bytes covered by those skipped charges.
    pub deferred_bytes: u64,
    /// Allocations observed without a task context.
    pub unattributed_charges: u64,
    /// Bytes of those allocations.
    pub unattributed_bytes: u64,
}

impl ChargeQuality {
    /// `true` when every observed allocation was charged to a task.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.deferred_charges == 0 && self.unattributed_charges == 0
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
    parent: Option<u64>,
}

impl TaskGuard {
    /// Create a new task guard (internal use)
    fn new(task_id: u64, parent: Option<u64>) -> Self {
        Self { task_id, parent }
    }
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        global_registry().complete_task(self.task_id);

        // Restore the parent instead of blanking the slot: nested scopes used to
        // leave the thread with no current task at all, so allocations after an
        // inner scope ended were silently unattributed (plan §7.2 item 6).
        // The write is skipped when another task has already taken the slot.
        CURRENT_TASK_ID.with(|cell| {
            if cell.get() == Some(self.task_id) {
                cell.set(self.parent);
            }
        });
    }
}

/// Task registry for managing task metadata
pub struct TaskIdRegistry {
    /// Task metadata storage
    tasks: Arc<RwLock<HashMap<u64, TaskMeta>>>,
    /// Set of used task IDs for uniqueness detection
    used_ids: Arc<RwLock<HashSet<u64>>>,
    /// Allocator charges skipped because the registry lock was busy. Reported so a
    /// skipped charge is visible instead of silently lost (plan A08).
    skipped_charges: std::sync::atomic::AtomicU64,
    /// Bytes that were part of the skipped charges, so a report can state the size of the
    /// gap and not only its frequency.
    skipped_bytes: std::sync::atomic::AtomicU64,
    /// Allocations observed while no task context was installed: they belong to no task,
    /// which is different from "the charge was deferred".
    unattributed_charges: std::sync::atomic::AtomicU64,
    /// Bytes of the unattributed allocations.
    unattributed_bytes: std::sync::atomic::AtomicU64,
    /// Deferred charges per task, so a report can mark *that* task's numbers as a lower
    /// bound instead of only flagging the whole window.
    deferred_by_task: dashmap::DashMap<u64, (u64, u64)>,
}

impl TaskIdRegistry {
    /// Create new task registry
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            used_ids: Arc::new(RwLock::new(HashSet::new())),
            skipped_charges: std::sync::atomic::AtomicU64::new(0),
            skipped_bytes: std::sync::atomic::AtomicU64::new(0),
            unattributed_charges: std::sync::atomic::AtomicU64::new(0),
            unattributed_bytes: std::sync::atomic::AtomicU64::new(0),
            deferred_by_task: dashmap::DashMap::new(),
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
        TaskGuard::new(task_id, parent)
    }

    /// Register a task with an externally-generated task ID.
    ///
    /// Unlike `task_scope()` which generates its own ID, this allows
    /// integration with async trackers that already have an ID assigned.
    /// The parent is inferred from the current thread-local task.
    ///
    /// # Arguments
    ///
    /// * `task_id` - Externally-generated task ID
    /// * `name` - Task name
    pub fn register_explicit_task(&self, task_id: u64, name: &str) {
        let parent = Self::current_task_id();
        let mut meta = TaskMeta::new(task_id, parent, name.to_string());

        if let Some(tokio_id) = self.get_tokio_task_id() {
            meta.tokio_id = Some(tokio_id);
        }

        if let Ok(mut tasks) = self.tasks.write() {
            tasks.insert(task_id, meta);
        }
        if let Ok(mut used_ids) = self.used_ids.write() {
            used_ids.insert(task_id);
        }
        CURRENT_TASK_ID.set(Some(task_id));
    }

    /// Mark a registered task as completed.
    ///
    /// The thread-local slot is only cleared when it still holds this task: another
    /// task (or a parent scope) may be current on this thread.
    pub fn unregister_task(&self, task_id: u64) {
        if let Ok(mut tasks) = self.tasks.write() {
            if let Some(meta) = tasks.get_mut(&task_id) {
                meta.mark_completed();
            }
        }
        CURRENT_TASK_ID.with(|cell| {
            if cell.get() == Some(task_id) {
                cell.set(None);
            }
        });
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

        // The thread-local slot is restored by `TaskGuard::drop`, which knows the
        // parent task. Do not blank it here: completing a nested task would drop
        // the parent context for every following allocation.
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

    /// Charge an allocation without ever blocking on the registry lock.
    ///
    /// The global allocator callback runs while other code may hold the registry read
    /// lock (and allocate inside it). Taking the write lock there self-deadlocks, so this
    /// uses `try_write` and reports [`ChargeOutcome::Deferred`] instead of waiting; the
    /// attempt and its bytes are counted so a report can mark the task metrics as a lower
    /// bound of this capture window.
    pub fn try_record_allocation(&self, size: usize) -> ChargeOutcome {
        let Some(task_id) = Self::current_task_id() else {
            self.unattributed_charges.fetch_add(1, Ordering::Relaxed);
            self.unattributed_bytes
                .fetch_add(size as u64, Ordering::Relaxed);
            return ChargeOutcome::Unattributed;
        };

        match self.tasks.try_write() {
            Ok(mut tasks) => match tasks.get_mut(&task_id) {
                Some(meta) => {
                    meta.record_allocation(size);
                    ChargeOutcome::Charged
                }
                // The task ended while the allocation was in flight: nothing to charge.
                None => ChargeOutcome::Unattributed,
            },
            Err(_) => {
                self.skipped_charges.fetch_add(1, Ordering::Relaxed);
                self.skipped_bytes.fetch_add(size as u64, Ordering::Relaxed);
                // Attribute the gap to the task that missed it. This touches only the
                // DashMap shard, never the registry lock that is currently held, so the
                // allocator callback stays deadlock-free.
                let mut entry = self.deferred_by_task.entry(task_id).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += size as u64;
                ChargeOutcome::Deferred
            }
        }
    }

    /// Per-task deferred charges: `task_id -> (count, bytes)`.
    ///
    /// A task present here has accounting that is a lower bound for this capture window.
    #[must_use]
    pub fn deferred_tasks(&self) -> std::collections::HashMap<u64, (u64, u64)> {
        self.deferred_by_task
            .iter()
            .map(|entry| (*entry.key(), *entry.value()))
            .collect()
    }

    /// Completeness of the task accounting observed so far.
    pub fn charge_quality(&self) -> ChargeQuality {
        ChargeQuality {
            deferred_charges: self.skipped_charges.load(Ordering::Relaxed),
            deferred_bytes: self.skipped_bytes.load(Ordering::Relaxed),
            unattributed_charges: self.unattributed_charges.load(Ordering::Relaxed),
            unattributed_bytes: self.unattributed_bytes.load(Ordering::Relaxed),
        }
    }

    pub fn task_stats_from_events(
        events: &[crate::event_store::MemoryEvent],
    ) -> std::collections::BTreeMap<u64, TaskLedgerStats> {
        use crate::event_store::MemoryEventType;
        use std::collections::{BTreeMap, HashMap};

        /// A binding that is still open at the cut, with the task that allocated it.
        struct OpenBinding {
            ptr: usize,
            serial: Option<u64>,
            size: u64,
            task: u64,
        }

        // Tombstoned slots plus a per-address index: a release touches only the bindings
        // of its own address, so replay stays linear in the number of events instead of
        // scanning every open binding (the same shape the binding ledger uses).
        let mut slots: Vec<Option<OpenBinding>> = Vec::new();
        let mut by_addr: HashMap<usize, Vec<usize>> = HashMap::new();

        fn push(
            slots: &mut Vec<Option<OpenBinding>>,
            by_addr: &mut HashMap<usize, Vec<usize>>,
            binding: OpenBinding,
        ) {
            let ptr = binding.ptr;
            slots.push(Some(binding));
            by_addr.entry(ptr).or_default().push(slots.len() - 1);
        }

        fn take(
            slots: &mut [Option<OpenBinding>],
            by_addr: &mut HashMap<usize, Vec<usize>>,
            ptr: usize,
            serial: Option<u64>,
            size: usize,
        ) -> Option<OpenBinding> {
            let index = {
                let bucket = by_addr.get(&ptr)?;
                let by_serial = serial.and_then(|serial| {
                    bucket.iter().copied().find(|index| {
                        slots[*index]
                            .as_ref()
                            .is_some_and(|binding| binding.serial == Some(serial))
                    })
                });
                let by_size = || {
                    bucket.iter().copied().find(|index| {
                        slots[*index]
                            .as_ref()
                            .is_some_and(|binding| size == 0 || binding.size == size as u64)
                    })
                };
                by_serial
                    .or_else(by_size)
                    .or_else(|| bucket.first().copied())?
            };

            let binding = slots[index].take()?;
            if let Some(bucket) = by_addr.get_mut(&ptr) {
                bucket.retain(|candidate| *candidate != index);
                if bucket.is_empty() {
                    by_addr.remove(&ptr);
                }
            }
            Some(binding)
        }

        let mut stats: BTreeMap<u64, TaskLedgerStats> = BTreeMap::new();

        for event in events {
            match event.event_type {
                MemoryEventType::Allocate => {
                    let Some(task_id) = event.task_id else {
                        continue;
                    };
                    let entry = stats.entry(task_id).or_default();
                    entry.allocation_count += 1;
                    entry.allocated_bytes += event.size as u64;
                    entry.live_bytes = entry.live_bytes.saturating_add(event.size as u64);
                    push(
                        &mut slots,
                        &mut by_addr,
                        OpenBinding {
                            ptr: event.ptr,
                            serial: event.binding_serial,
                            size: event.size as u64,
                            task: task_id,
                        },
                    );
                }
                MemoryEventType::Deallocate => {
                    // Only a release that closes a live binding is a valid lifecycle
                    // transition: a repeated (or unmatched) release must not book bytes
                    // anywhere, or the totals drift away from the ledger.
                    let Some(closed) = take(
                        &mut slots,
                        &mut by_addr,
                        event.ptr,
                        event.binding_serial,
                        event.size,
                    ) else {
                        continue;
                    };
                    if let Some(actor_id) = event.task_id {
                        let actor = stats.entry(actor_id).or_default();
                        actor.deallocation_count += 1;
                        actor.deallocated_bytes += event.size as u64;
                    }
                    // The *live* bytes belong to the task that allocated the binding,
                    // which may have ended or be a different task entirely.
                    if let Some(owner) = stats.get_mut(&closed.task) {
                        owner.live_bytes = owner.live_bytes.saturating_sub(closed.size);
                    }
                }
                MemoryEventType::Reallocate => {
                    let previous = event.old_size.unwrap_or(0) as u64;
                    let new_ptr = event.new_ptr.unwrap_or(event.ptr);
                    let Some(mut binding) = take(
                        &mut slots,
                        &mut by_addr,
                        event.ptr,
                        event.binding_serial,
                        previous as usize,
                    ) else {
                        continue;
                    };
                    // The binding keeps its owner; only its size and address change.
                    if let Some(owner) = stats.get_mut(&binding.task) {
                        owner.live_bytes = owner
                            .live_bytes
                            .saturating_sub(binding.size)
                            .saturating_add(event.size as u64);
                        owner.allocated_bytes += event.size as u64;
                        owner.deallocated_bytes += previous;
                    }
                    binding.size = event.size as u64;
                    binding.ptr = new_ptr;
                    binding.serial = event.binding_serial.or(binding.serial);
                    push(&mut slots, &mut by_addr, binding);
                }
                _ => {}
            }
        }
        stats
    }

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
    // The registry is a process-wide singleton: tests that spawn or clear tasks must
    // not interleave with each other.
    use serial_test::serial;

    #[test]
    fn test_task_id_generation() {
        let id1 = generate_task_id();
        let id2 = generate_task_id();
        assert!(id2 > id1);
    }

    #[test]
    #[serial]
    fn test_spawn_task() {
        let registry = global_registry();
        registry.clear();

        let task_id = registry.spawn_task(None, "test_task".to_string());

        let meta = registry.get_task(task_id);
        assert!(meta.is_some());
        assert_eq!(meta.unwrap().name, "test_task");
    }

    #[test]
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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

    /// Objective: Verify a nested task scope restores the parent context on drop
    /// (plan §7.2 item 6).
    /// Invariants: after the inner scope ends the parent is current again; after the
    /// outer scope ends no task is current.
    #[test]
    #[serial]
    fn test_nested_scope_restores_parent_context() {
        let registry = global_registry();
        // `clear()` also resets the thread-local current task.
        registry.clear();

        {
            let _parent = registry.task_scope("parent");
            let parent_id = TaskIdRegistry::current_task_id()
                .expect("the parent scope must install a current task");

            {
                let _child = registry.task_scope("child");
                assert_ne!(
                    TaskIdRegistry::current_task_id(),
                    Some(parent_id),
                    "the child scope must take over the current context"
                );
            }

            assert_eq!(
                TaskIdRegistry::current_task_id(),
                Some(parent_id),
                "ending the child scope must restore the parent task"
            );
        }

        assert_eq!(
            TaskIdRegistry::current_task_id(),
            None,
            "ending the outermost scope must restore the empty context"
        );
    }
}

#[cfg(test)]
mod ledger_tests;
