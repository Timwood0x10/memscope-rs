//! Async tracker types and data structures.
//!
//! This module contains type definitions for async memory tracking.

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread::ThreadId;

// Re-export TaskType from async_tracker
pub use super::async_tracker::TaskType;

/// Unique identifier for async tasks
pub type TaskId = u128;

/// Async memory tracking task information.
#[derive(Debug, Clone)]
pub struct TaskInfo {
    /// Task unique identifier
    pub task_id: u64,
    /// Task name
    pub name: String,
    /// Thread ID where the task runs
    pub thread_id: ThreadId,
    /// Task creation timestamp
    pub created_at: u64,
    /// Active allocations count
    pub active_allocations: usize,
    /// Total allocated memory
    pub total_memory: usize,
}

/// Extended task information with waker and span IDs
#[derive(Clone, Copy, Debug, Default)]
pub struct ExtendedTaskInfo {
    /// Primary task ID from TrackedFuture
    pub waker_id: TaskId,
    /// Secondary span ID from tracing ecosystem
    pub span_id: Option<u64>,
    /// Task creation timestamp
    pub created_at: u64,
}

impl ExtendedTaskInfo {
    /// Create new extended task info with current timestamp
    pub fn new(waker_id: TaskId, span_id: Option<u64>) -> Self {
        Self {
            waker_id,
            span_id,
            created_at: current_timestamp(),
        }
    }

    /// Check if any tracking ID is available
    pub fn has_tracking_id(&self) -> bool {
        self.waker_id != 0 || self.span_id.is_some()
    }

    /// Get the primary tracking ID
    pub fn primary_id(&self) -> TaskId {
        if self.waker_id != 0 {
            self.waker_id
        } else {
            self.span_id.map(|id| id as TaskId).unwrap_or(0)
        }
    }
}

/// Memory allocation information for async tracking.
#[derive(Debug, Clone)]
pub struct AsyncAllocation {
    /// Memory pointer
    pub ptr: usize,
    /// Allocation size
    pub size: usize,
    /// Allocation timestamp
    pub timestamp: u64,
    /// Associated task ID
    pub task_id: u64,
    /// Optional variable name
    pub var_name: Option<String>,
    /// Optional type name
    pub type_name: Option<String>,
    /// Optional source location
    pub source_location: Option<SourceLocation>,
}

/// Source location for tracking where allocations occur
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SourceLocation {
    /// File path
    pub file: String,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Function name
    pub function: String,
    /// Module path
    pub module_path: String,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(file: &str, line: u32, column: u32, function: &str, module_path: &str) -> Self {
        Self {
            file: file.to_string(),
            line,
            column,
            function: function.to_string(),
            module_path: module_path.to_string(),
        }
    }

    /// Capture current source location using std::panic::Location
    pub fn capture() -> Self {
        let loc = std::panic::Location::caller();
        Self {
            file: loc.file().to_string(),
            line: loc.line(),
            column: loc.column(),
            function: String::new(),
            module_path: String::new(),
        }
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}:{}",
            self.file, self.line, self.column, self.function
        )
    }
}

/// Async memory tracking statistics.
#[derive(Debug, Clone, Default)]
pub struct AsyncStats {
    /// Total number of tasks tracked
    pub total_tasks: usize,
    /// Total allocations across all tasks
    pub total_allocations: usize,
    /// Total memory allocated
    pub total_memory: usize,
    /// Current active memory (sum of all active allocations)
    pub active_memory: usize,
    /// Peak memory usage (maximum active memory observed)
    pub peak_memory: usize,
    /// Active tasks count
    pub active_tasks: usize,
}

/// Async memory snapshot.
#[derive(Debug, Clone)]
pub struct AsyncSnapshot {
    /// Snapshot timestamp
    pub timestamp: u64,
    /// Task information
    pub tasks: Vec<TaskInfo>,
    /// Active allocations
    pub allocations: Vec<AsyncAllocation>,
    /// Statistics
    pub stats: AsyncStats,
}

/// Error type for async memory tracking operations
#[derive(Debug, Clone)]
pub enum AsyncError {
    /// Initialization or configuration errors
    Initialization {
        component: Arc<str>,
        message: Arc<str>,
        recoverable: bool,
    },
    /// Task tracking errors
    TaskTracking {
        operation: TaskOperation,
        message: Arc<str>,
        task_id: Option<u64>,
    },
    /// Allocation tracking errors
    AllocationTracking {
        event_type: AllocationEventType,
        message: Arc<str>,
        allocation_size: Option<usize>,
    },
    /// System-level errors
    System {
        operation: Arc<str>,
        message: Arc<str>,
    },
}

/// Task tracking operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskOperation {
    IdGeneration,
    Propagation,
    Registration,
    Cleanup,
}

/// Allocation event types for error context
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationEventType {
    Allocation,
    Deallocation,
    BufferWrite,
    Processing,
}

impl AsyncError {
    /// Create an initialization error
    pub fn initialization(component: &str, message: &str, recoverable: bool) -> Self {
        Self::Initialization {
            component: Arc::from(component),
            message: Arc::from(message),
            recoverable,
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Initialization { recoverable, .. } => *recoverable,
            Self::TaskTracking { .. } => true,
            Self::AllocationTracking { .. } => true,
            Self::System { .. } => false,
        }
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        match self {
            Self::Initialization { message, .. }
            | Self::TaskTracking { message, .. }
            | Self::AllocationTracking { message, .. }
            | Self::System { message, .. } => message,
        }
    }
}

impl fmt::Display for AsyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initialization {
                component,
                message,
                recoverable,
            } => {
                write!(
                    f,
                    "Async memory tracking initialization failed in {}: {} ({})",
                    component,
                    message,
                    if *recoverable { "recoverable" } else { "fatal" }
                )
            }
            Self::TaskTracking {
                operation,
                message,
                task_id,
            } => {
                if let Some(id) = task_id {
                    write!(
                        f,
                        "Task tracking error during {:?} for task {}: {}",
                        operation, id, message
                    )
                } else {
                    write!(f, "Task tracking error during {:?}: {}", operation, message)
                }
            }
            Self::AllocationTracking {
                event_type,
                message,
                allocation_size,
            } => {
                if let Some(size) = allocation_size {
                    write!(
                        f,
                        "Allocation tracking error during {:?} ({}B): {}",
                        event_type, size, message
                    )
                } else {
                    write!(
                        f,
                        "Allocation tracking error during {:?}: {}",
                        event_type, message
                    )
                }
            }
            Self::System { operation, message } => {
                write!(f, "System error during {}: {}", operation, message)
            }
        }
    }
}

impl std::error::Error for AsyncError {}

/// Result type for async memory tracking operations
pub type AsyncResult<T> = Result<T, AsyncError>;

/// Task memory profile with performance metrics
#[derive(Debug, Clone)]
pub struct TaskMemoryProfile {
    /// Task ID
    pub task_id: u64,
    /// Task name
    pub task_name: String,
    /// Task type
    pub task_type: TaskType,
    /// Total allocations
    pub total_allocations: u64,
    /// Total bytes allocated
    pub total_bytes: u64,
    /// Peak memory usage
    pub peak_memory: u64,
    /// Allocation count by size
    pub size_distribution: Vec<(usize, u64)>,
    /// Average allocation size
    pub avg_allocation_size: f64,
    /// Task duration in nanoseconds
    pub duration_ns: u64,
    /// Allocation rate (allocations per second)
    pub allocation_rate: f64,
}

impl TaskMemoryProfile {
    /// Create a new task memory profile
    pub fn new(task_id: u64, task_name: String) -> Self {
        Self {
            task_id,
            task_name,
            task_type: TaskType::default(),
            total_allocations: 0,
            total_bytes: 0,
            peak_memory: 0,
            size_distribution: Vec::new(),
            avg_allocation_size: 0.0,
            duration_ns: 0,
            allocation_rate: 0.0,
        }
    }

    /// Create a new task memory profile with task type
    pub fn with_type(task_id: u64, task_name: String, task_type: TaskType) -> Self {
        Self {
            task_id,
            task_name,
            task_type,
            total_allocations: 0,
            total_bytes: 0,
            peak_memory: 0,
            size_distribution: Vec::new(),
            avg_allocation_size: 0.0,
            duration_ns: 0,
            allocation_rate: 0.0,
        }
    }

    /// Calculate average allocation size
    pub fn calculate_avg_size(&mut self) {
        if self.total_allocations > 0 {
            self.avg_allocation_size = self.total_bytes as f64 / self.total_allocations as f64;
        }
    }

    /// Calculate allocation rate
    pub fn calculate_allocation_rate(&mut self) {
        if self.duration_ns > 0 {
            let seconds = self.duration_ns as f64 / 1_000_000_000.0;
            self.allocation_rate = self.total_allocations as f64 / seconds;
        }
    }
}

/// Tracked future wrapper for task-level memory tracking
pub struct TrackedFuture<F> {
    inner: Pin<Box<F>>,
    task_id: Option<TaskId>,
    #[allow(dead_code)]
    task_name: Option<String>,
}

impl<F> TrackedFuture<F>
where
    F: Future,
{
    /// Create a new tracked future
    pub fn new(future: F) -> Self {
        Self {
            inner: Box::pin(future),
            task_id: None,
            task_name: None,
        }
    }

    /// Create a tracked future with a name
    pub fn with_name(future: F, name: String) -> Self {
        Self {
            inner: Box::pin(future),
            task_id: None,
            task_name: Some(name),
        }
    }
}

impl<F> Future for TrackedFuture<F>
where
    F: Future,
{
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Generate task ID on first poll
        if self.task_id.is_none() {
            match generate_task_id(cx) {
                Ok(id) => self.task_id = Some(id),
                Err(e) => {
                    tracing::warn!("Failed to generate task ID: {}", e);
                }
            }
        }

        // Set task context
        if let Some(task_id) = self.task_id {
            let task_info = ExtendedTaskInfo::new(task_id, None);
            set_current_task(task_info);

            let result = self.inner.as_mut().poll(cx);

            if result.is_ready() {
                clear_current_task();
            }
            result
        } else {
            self.inner.as_mut().poll(cx)
        }
    }
}

/// Memory usage snapshot for async tasks
#[derive(Debug, Clone)]
pub struct AsyncMemorySnapshot {
    /// Number of currently active tracked tasks
    pub active_task_count: usize,
    /// Total memory allocated by tracked tasks
    pub total_allocated_bytes: u64,
    /// Number of allocation events recorded
    pub allocation_events: u64,
    /// Number of events dropped due to buffer overflow
    pub events_dropped: u64,
    /// Buffer utilization ratio (0.0 to 1.0)
    pub buffer_utilization: f64,
}

impl AsyncMemorySnapshot {
    /// Get the number of active tasks
    pub fn active_task_count(&self) -> usize {
        self.active_task_count
    }

    /// Get total allocated memory in bytes
    pub fn total_allocated(&self) -> u64 {
        self.total_allocated_bytes
    }

    /// Check if data quality is good (< 5% events dropped)
    pub fn has_good_data_quality(&self) -> bool {
        if self.allocation_events == 0 {
            return true;
        }
        let drop_rate = self.events_dropped as f64 / self.allocation_events as f64;
        drop_rate < 0.05
    }

    /// Get data quality warning if applicable
    pub fn data_quality_warning(&self) -> Option<String> {
        if !self.has_good_data_quality() && self.allocation_events > 0 {
            let drop_rate = (self.events_dropped as f64 / self.allocation_events as f64) * 100.0;
            Some(format!(
                "Poor data quality: {:.1}% of events dropped. Consider increasing buffer size.",
                drop_rate
            ))
        } else {
            None
        }
    }
}

/// Global monotonic counter for task ID uniqueness
static TASK_EPOCH: AtomicU64 = AtomicU64::new(1);

thread_local! {
    static CURRENT_TASK: std::cell::Cell<ExtendedTaskInfo> = const { std::cell::Cell::new(ExtendedTaskInfo {
        waker_id: 0,
        span_id: None,
        created_at: 0,
    }) };
}

/// Generate unique task ID from Context waker
pub fn generate_task_id(cx: &Context<'_>) -> AsyncResult<TaskId> {
    let waker_addr = cx.waker() as *const _ as u64;
    let epoch = TASK_EPOCH.fetch_add(1, Ordering::Relaxed);
    let task_id = ((epoch as u128) << 64) | (waker_addr as u128);

    if task_id == 0 {
        return Err(AsyncError::TaskTracking {
            operation: TaskOperation::IdGeneration,
            message: Arc::from("Generated zero task ID"),
            task_id: None,
        });
    }

    Ok(task_id)
}

/// Set current task information in thread-local storage
pub fn set_current_task(task_info: ExtendedTaskInfo) {
    CURRENT_TASK.with(|current| current.set(task_info));
}

/// Get current task information from thread-local storage
pub fn get_current_task() -> ExtendedTaskInfo {
    CURRENT_TASK.with(|current| current.get())
}

/// Clear current task context
pub fn clear_current_task() {
    CURRENT_TASK.with(|current| current.set(ExtendedTaskInfo::default()));
}

/// Get current timestamp using efficient method
fn current_timestamp() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        unsafe { std::arch::x86_64::_rdtsc() }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_info_creation() {
        let task = TaskInfo {
            task_id: 123,
            name: "test_task".to_string(),
            thread_id: std::thread::current().id(),
            created_at: 789,
            active_allocations: 2,
            total_memory: 2048,
        };

        assert_eq!(task.task_id, 123);
        assert_eq!(task.name, "test_task");
    }

    #[test]
    fn test_async_allocation_creation() {
        let alloc = AsyncAllocation {
            ptr: 0x1000,
            size: 1024,
            timestamp: 12345,
            task_id: 1,
            var_name: Some("test_var".to_string()),
            type_name: Some("Vec<u8>".to_string()),
            source_location: None,
        };

        assert_eq!(alloc.ptr, 0x1000);
        assert_eq!(alloc.size, 1024);
        assert_eq!(alloc.task_id, 1);
    }

    #[test]
    fn test_async_stats_default() {
        let stats = AsyncStats::default();
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.peak_memory, 0);
    }

    #[test]
    fn test_extended_task_info() {
        let info = ExtendedTaskInfo::new(12345, Some(67890));
        assert!(info.has_tracking_id());
        assert_eq!(info.primary_id(), 12345);
        assert_ne!(info.created_at, 0);
    }

    #[test]
    fn test_async_error_creation() {
        let error = AsyncError::initialization("tracker", "Failed to start", true);
        assert!(error.is_recoverable());
        assert_eq!(error.message(), "Failed to start");
    }

    #[test]
    fn test_task_memory_profile() {
        let mut profile = TaskMemoryProfile::new(1, "test".to_string());
        profile.total_allocations = 100;
        profile.total_bytes = 102400;
        profile.calculate_avg_size();
        assert_eq!(profile.avg_allocation_size, 1024.0);
    }

    #[test]
    fn test_async_memory_snapshot() {
        let snapshot = AsyncMemorySnapshot {
            active_task_count: 1,
            total_allocated_bytes: 1024,
            allocation_events: 100,
            events_dropped: 0,
            buffer_utilization: 0.5,
        };
        assert!(snapshot.has_good_data_quality());
        assert_eq!(snapshot.active_task_count(), 1);
    }

    #[test]
    fn test_thread_local_storage() {
        let info = ExtendedTaskInfo::new(12345, Some(67890));
        set_current_task(info);
        let retrieved = get_current_task();
        assert_eq!(retrieved.waker_id, 12345);
        clear_current_task();
        assert!(!get_current_task().has_tracking_id());
    }
}
