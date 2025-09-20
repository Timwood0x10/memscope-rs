//! Task identification and propagation for async memory tracking
//!
//! This module provides zero-overhead task identification using Context waker addresses
//! combined with global epoch counters to ensure absolute uniqueness across the
//! application lifetime.

use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::Context;

use crate::async_memory::error::{AsyncError, AsyncResult, TaskOperation};

/// Global monotonic counter to ensure task ID uniqueness
///
/// Combined with waker addresses, this guarantees no ID collisions even if
/// waker memory is reused after task completion.
static TASK_EPOCH: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for async tasks
///
/// Combines a global epoch counter (high 64 bits) with the waker vtable address
/// (low 64 bits) to ensure absolute uniqueness across application lifetime.
pub type TaskId = u128;

/// Extended task information stored in thread-local storage
///
/// Supports dual-track approach: precise TrackedFuture identification
/// plus tracing::Subscriber integration for broader ecosystem coverage.
#[derive(Clone, Copy, Debug, Default)]
pub struct TaskInfo {
    /// Primary task ID from TrackedFuture (high precision)
    pub waker_id: TaskId,
    /// Secondary span ID from tracing ecosystem (broader coverage)
    pub span_id: Option<u64>,
    /// Task creation timestamp for lifecycle analysis
    pub created_at: u64,
}

impl TaskInfo {
    /// Create new task info with current timestamp
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

    /// Get the primary tracking ID (waker_id preferred, fallback to span_id)
    pub fn primary_id(&self) -> TaskId {
        if self.waker_id != 0 {
            self.waker_id
        } else {
            // Convert span_id to TaskId format if waker_id unavailable
            self.span_id.map(|id| id as TaskId).unwrap_or(0)
        }
    }
}

/// Thread-local storage for current task information
///
/// Uses Cell for zero-overhead access from the allocator hook path.
/// More efficient than tokio::task_local for our specific use case.
thread_local! {
    static CURRENT_TASK: Cell<TaskInfo> = const { Cell::new(TaskInfo {
        waker_id: 0,
        span_id: None,
        created_at: 0,
    }) };
}

/// Generate unique task ID from Context waker
///
/// Uses the waker's vtable address combined with a global epoch counter
/// to ensure uniqueness even if waker memory is reused.
#[inline(always)]
pub fn generate_task_id(cx: &Context<'_>) -> AsyncResult<TaskId> {
    // Extract waker vtable address as unique identifier
    // Use waker pointer address as identifier (stable within task lifetime)
    let waker_addr = cx.waker() as *const _ as u64;
    
    // Get monotonic epoch counter
    let epoch = TASK_EPOCH.fetch_add(1, Ordering::Relaxed);
    
    // Combine epoch (high 64 bits) with waker address (low 64 bits)
    let task_id = ((epoch as u128) << 64) | (waker_addr as u128);
    
    // Validate non-zero result
    if task_id == 0 {
        return Err(AsyncError::task_tracking(
            TaskOperation::IdGeneration,
            "Generated zero task ID - invalid waker or epoch overflow",
            None,
        ));
    }
    
    Ok(task_id)
}

/// Set current task information in thread-local storage
///
/// Called by TrackedFuture during poll operations to establish task context
/// for allocation tracking.
#[inline(always)]
pub fn set_current_task(task_info: TaskInfo) {
    CURRENT_TASK.with(|current| current.set(task_info));
}

/// Get current task information from thread-local storage
///
/// Returns the task context for the currently executing async task.
/// Used by the global allocator hook to attribute memory allocations.
#[inline(always)]
pub fn get_current_task() -> TaskInfo {
    CURRENT_TASK.with(|current| current.get())
}

/// Update span ID for tracing integration
///
/// Called by MemScopeSubscriber when entering/exiting tracing spans
/// to provide fallback task identification.
#[inline(always)]
pub fn update_span_id(span_id: Option<u64>) -> AsyncResult<()> {
    CURRENT_TASK.with(|current| {
        let mut info = current.get();
        info.span_id = span_id;
        current.set(info);
    });
    Ok(())
}

/// Clear current task context
///
/// Called when leaving task scope to prevent attribution of allocations
/// to completed tasks.
#[inline(always)]
pub fn clear_current_task() {
    CURRENT_TASK.with(|current| current.set(TaskInfo::default()));
}

/// Get current timestamp using efficient method
///
/// Uses TSC (Time Stamp Counter) on x86_64 for minimal overhead,
/// falls back to system time on other architectures.
#[inline(always)]
fn current_timestamp() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        // Use hardware timestamp counter for minimal overhead
        unsafe { std::arch::x86_64::_rdtsc() }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        // Fallback to system time for other architectures
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    }
}

/// Get current epoch counter value for diagnostics
pub fn current_epoch() -> u64 {
    TASK_EPOCH.load(Ordering::Relaxed)
}

/// Reset epoch counter (for testing only)
#[cfg(test)]
pub fn reset_epoch() {
    TASK_EPOCH.store(1, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::task::{RawWaker, RawWakerVTable, Waker};

    // Helper to create a dummy waker for testing
    fn create_test_waker() -> Waker {
        fn noop(_: *const ()) {}
        fn clone_waker(data: *const ()) -> RawWaker {
            RawWaker::new(data, &VTABLE)
        }
        
        const VTABLE: RawWakerVTable = RawWakerVTable::new(clone_waker, noop, noop, noop);
        
        unsafe {
            Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE))
        }
    }

    #[test]
    fn test_task_id_generation() {
        reset_epoch();
        
        let waker = create_test_waker();
        let cx = Context::from_waker(&waker);
        
        let id1 = generate_task_id(&cx).expect("Failed to generate task ID");
        let id2 = generate_task_id(&cx).expect("Failed to generate task ID");
        
        // IDs should be different due to epoch increment
        assert_ne!(id1, id2);
        
        // IDs should be non-zero
        assert_ne!(id1, 0);
        assert_ne!(id2, 0);
        
        // High 64 bits should contain epoch values
        let epoch1 = (id1 >> 64) as u64;
        let epoch2 = (id2 >> 64) as u64;
        assert_eq!(epoch2, epoch1 + 1);
    }

    #[test]
    fn test_task_info_operations() {
        let info = TaskInfo::new(12345, Some(67890));
        
        assert!(info.has_tracking_id());
        assert_eq!(info.primary_id(), 12345);
        assert_ne!(info.created_at, 0);
        
        // Test fallback to span_id
        let info_no_waker = TaskInfo::new(0, Some(67890));
        assert!(info_no_waker.has_tracking_id());
        assert_eq!(info_no_waker.primary_id(), 67890);
        
        // Test no tracking
        let info_empty = TaskInfo::default();
        assert!(!info_empty.has_tracking_id());
        assert_eq!(info_empty.primary_id(), 0);
    }

    #[test]
    fn test_thread_local_storage() {
        let info = TaskInfo::new(12345, Some(67890));
        
        // Initially empty
        assert!(!get_current_task().has_tracking_id());
        
        // Set and verify
        set_current_task(info);
        let retrieved = get_current_task();
        assert_eq!(retrieved.waker_id, 12345);
        assert_eq!(retrieved.span_id, Some(67890));
        
        // Update span ID
        update_span_id(Some(99999)).expect("Failed to update span ID");
        let updated = get_current_task();
        assert_eq!(updated.waker_id, 12345); // Unchanged
        assert_eq!(updated.span_id, Some(99999)); // Updated
        
        // Clear
        clear_current_task();
        assert!(!get_current_task().has_tracking_id());
    }

    #[test]
    fn test_epoch_progression() {
        reset_epoch();
        let initial_epoch = current_epoch();
        
        let waker = create_test_waker();
        let cx = Context::from_waker(&waker);
        
        // Generate some task IDs
        for i in 0..5 {
            let _id = generate_task_id(&cx).expect("Failed to generate task ID");
            assert_eq!(current_epoch(), initial_epoch + i + 1);
        }
    }

    #[test]
    fn test_timestamp_generation() {
        let ts1 = current_timestamp();
        let ts2 = current_timestamp();
        
        // Timestamps should be non-zero and monotonic (or at least not decreasing)
        assert_ne!(ts1, 0);
        assert_ne!(ts2, 0);
        assert!(ts2 >= ts1);
    }

    #[test]
    fn test_concurrent_task_id_generation() {
        use std::thread;
        use std::sync::{Arc, Mutex};
        
        reset_epoch();
        let ids = Arc::new(Mutex::new(Vec::new()));
        let handles: Vec<_> = (0..10).map(|_| {
            let ids_clone = Arc::clone(&ids);
            thread::spawn(move || {
                let waker = create_test_waker();
                let cx = Context::from_waker(&waker);
                let id = generate_task_id(&cx).expect("Failed to generate task ID");
                ids_clone.lock().expect("Lock poisoned").push(id);
            })
        }).collect();
        
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
        
        let ids = ids.lock().expect("Lock poisoned");
        
        // All IDs should be unique
        let mut sorted_ids = ids.clone();
        sorted_ids.sort();
        sorted_ids.dedup();
        assert_eq!(sorted_ids.len(), ids.len());
        
        // All IDs should be non-zero
        assert!(ids.iter().all(|&id| id != 0));
    }
}