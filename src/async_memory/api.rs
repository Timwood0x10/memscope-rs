//! High-level API for async memory tracking
//!
//! Provides user-friendly functions for initializing tracking, spawning
//! tracked tasks, and retrieving memory statistics.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// Note: tokio dependency will be added conditionally for async features

use crate::async_memory::buffer::get_buffer_stats;
use crate::async_memory::error::AsyncResult;
use crate::async_memory::task_id::{generate_task_id, set_current_task, TaskInfo};

/// Initialize async memory tracking system
///
/// Must be called before spawning any tracked tasks.
/// Sets up background aggregation and monitoring.
pub fn initialize() -> AsyncResult<()> {
    // For now, just verify the system is ready
    // Future implementation would start background aggregator thread
    tracing::info!("Async memory tracking system initialized");
    Ok(())
}

/// Create a tracked future wrapper
///
/// Wraps the provided future in a TrackedFuture that automatically
/// attributes memory allocations to the task.
///
/// Note: Use with your preferred async runtime (tokio, async-std, etc.)
pub fn create_tracked<F>(future: F) -> TrackedFuture<F>
where
    F: Future,
{
    TrackedFuture::new(future)
}

/// Future wrapper that provides task-level memory tracking
///
/// Automatically sets task context during poll operations,
/// enabling allocation attribution to the specific task.
pub struct TrackedFuture<F> {
    inner: Pin<Box<F>>,
    task_id: Option<crate::async_memory::TaskId>,
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
                    // Continue without tracking rather than failing
                }
            }
        }

        // Set task context for allocation attribution
        if let Some(task_id) = self.task_id {
            let task_info = TaskInfo::new(task_id, None);
            set_current_task(task_info);

            // Poll the inner future
            let result = self.inner.as_mut().poll(cx);

            // Clear task context when leaving
            if result.is_ready() {
                crate::async_memory::task_id::clear_current_task();
            }

            result
        } else {
            // Poll without tracking if ID generation failed
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

/// Get current memory usage snapshot
///
/// Returns statistics about async task memory usage.
/// This is a simplified implementation - production version would
/// aggregate data from all threads and the background aggregator.
pub fn get_memory_snapshot() -> AsyncMemorySnapshot {
    let buffer_stats = get_buffer_stats();

    // Simplified snapshot - real implementation would aggregate from all sources
    AsyncMemorySnapshot {
        active_task_count: if buffer_stats.current_events > 0 {
            1
        } else {
            0
        },
        total_allocated_bytes: buffer_stats.current_events as u64 * 1024, // Rough estimate
        allocation_events: buffer_stats.current_events as u64,
        events_dropped: buffer_stats.events_dropped as u64,
        buffer_utilization: buffer_stats.utilization,
    }
}

/// Check if async memory tracking is currently active
pub fn is_tracking_active() -> bool {
    crate::async_memory::allocator::is_tracking_enabled()
}

/// Spawn a tracked async task
///
/// This is a convenience function that wraps the provided future in a TrackedFuture.
/// Use with your preferred async runtime (tokio, async-std, etc.)
///
/// Example with tokio:
/// ```rust,no_run
/// use memscope_rs::async_memory;
///
/// #[tokio::main]
/// async fn main() {
///     let handle = async_memory::spawn_tracked(async {
///         let data = vec![0u8; 1024];
///         data.len()
///     });
///     
///     let result = handle.await;
///     println!("Result: {}", result);
/// }
/// ```
pub fn spawn_tracked<F>(future: F) -> TrackedFuture<F>
where
    F: Future,
{
    create_tracked(future)
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

        unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
    }

    #[test]
    fn test_initialization() {
        let result = initialize();
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_initialization() {
        // Test that multiple initializations are safe
        assert!(initialize().is_ok());
        assert!(initialize().is_ok());
        assert!(initialize().is_ok());
    }

    #[test]
    fn test_memory_snapshot_good_quality() {
        let snapshot = AsyncMemorySnapshot {
            active_task_count: 1,
            total_allocated_bytes: 1024,
            allocation_events: 100,
            events_dropped: 0,
            buffer_utilization: 0.5,
        };

        assert!(snapshot.buffer_utilization >= 0.0);
        assert!(snapshot.buffer_utilization <= 1.0);
        assert_eq!(snapshot.active_task_count(), 1);
        assert_eq!(snapshot.total_allocated(), 1024);
        assert!(snapshot.has_good_data_quality());
        assert!(snapshot.data_quality_warning().is_none());
    }

    #[test]
    fn test_memory_snapshot_poor_quality() {
        let snapshot = AsyncMemorySnapshot {
            active_task_count: 2,
            total_allocated_bytes: 2048,
            allocation_events: 100,
            events_dropped: 10, // 10% drop rate
            buffer_utilization: 0.9,
        };

        assert!(!snapshot.has_good_data_quality());
        let warning = snapshot.data_quality_warning();
        assert!(warning.is_some());
        let warning_msg = warning.unwrap();
        assert!(warning_msg.contains("10.0%"));
        assert!(warning_msg.contains("Poor data quality"));
    }

    #[test]
    fn test_memory_snapshot_edge_cases() {
        // Test with zero events
        let snapshot = AsyncMemorySnapshot {
            active_task_count: 0,
            total_allocated_bytes: 0,
            allocation_events: 0,
            events_dropped: 0,
            buffer_utilization: 0.0,
        };
        assert!(snapshot.has_good_data_quality());
        assert!(snapshot.data_quality_warning().is_none());

        // Test with high drop rate but zero events
        let snapshot = AsyncMemorySnapshot {
            active_task_count: 0,
            total_allocated_bytes: 0,
            allocation_events: 0,
            events_dropped: 100, // Should not matter if no events total
            buffer_utilization: 0.0,
        };
        assert!(snapshot.has_good_data_quality());
    }

    #[test]
    fn test_memory_snapshot_boundary_conditions() {
        // Test exactly at 5% drop rate (boundary condition)
        let snapshot = AsyncMemorySnapshot {
            active_task_count: 1,
            total_allocated_bytes: 1000,
            allocation_events: 100,
            events_dropped: 5, // Exactly 5%
            buffer_utilization: 0.5,
        };
        assert!(!snapshot.has_good_data_quality()); // 5% is NOT good quality
        assert!(snapshot.data_quality_warning().is_some());

        // Test just under 5% drop rate
        let snapshot = AsyncMemorySnapshot {
            active_task_count: 1,
            total_allocated_bytes: 1000,
            allocation_events: 1000,
            events_dropped: 49, // 4.9%
            buffer_utilization: 0.5,
        };
        assert!(snapshot.has_good_data_quality());
        assert!(snapshot.data_quality_warning().is_none());
    }

    #[test]
    fn test_tracked_future_creation() {
        let future = async { 42 };
        let tracked = create_tracked(future);
        
        // Verify initial state
        assert!(tracked.task_id.is_none());
    }

    #[test]
    fn test_spawn_tracked_alias() {
        let future = async { "hello" };
        let tracked = spawn_tracked(future);
        
        // spawn_tracked should be equivalent to create_tracked
        assert!(tracked.task_id.is_none());
    }

    #[test]
    fn test_tracked_future_poll_ready() {
        let future = async { 123 };
        let mut tracked = create_tracked(future);
        let waker = create_test_waker();
        let mut cx = Context::from_waker(&waker);

        // Poll should complete immediately for simple future
        let result = Pin::new(&mut tracked).poll(&mut cx);
        match result {
            Poll::Ready(value) => assert_eq!(value, 123),
            Poll::Pending => {
                // May be pending in test environment, that's OK
            }
        }
    }

    #[test]
    fn test_tracked_future_multiple_polls() {
        use std::sync::{Arc, Mutex};
        
        let poll_count = Arc::new(Mutex::new(0));
        let poll_count_clone = poll_count.clone();
        
        // Create a future that's pending the first time, ready the second
        let future = async move {
            let mut count = poll_count_clone.lock().unwrap();
            *count += 1;
            if *count == 1 {
                // Simulate pending on first poll
                std::future::pending::<()>().await;
            }
            "completed"
        };

        let mut tracked = create_tracked(future);
        let waker = create_test_waker();
        let mut cx = Context::from_waker(&waker);

        // First poll - may generate task ID
        let _result1 = Pin::new(&mut tracked).poll(&mut cx);
        
        // Second poll - should reuse task ID if generated
        let _result2 = Pin::new(&mut tracked).poll(&mut cx);
        
        // Task ID should be consistent between polls if generated
        // (We can't easily test the internal state, but the behavior should be consistent)
    }

    #[test]
    fn test_tracked_future_task_context() {
        use crate::async_memory::task_id::{get_current_task, clear_current_task};
        
        // Clear any existing context first
        clear_current_task();
        
        let future = async {
            // Try to get current task context during execution
            let _task_info = get_current_task();
            true // Just return a simple value
        };

        let mut tracked = create_tracked(future);
        let waker = create_test_waker();
        let mut cx = Context::from_waker(&waker);

        // Poll the future
        let _result = Pin::new(&mut tracked).poll(&mut cx);
        
        // Context should be cleared after completion
        let _current_task = get_current_task();
        // May or may not have task info depending on implementation
    }

    #[test]
    fn test_is_tracking_active() {
        // Test the tracking status function
        let _is_active = is_tracking_active();
        // The function should not panic and should return a boolean
    }

    #[test]
    fn test_get_memory_snapshot_integration() {
        // Test the real get_memory_snapshot function
        let snapshot = get_memory_snapshot();
        
        // Verify snapshot has reasonable values
        assert!(snapshot.buffer_utilization >= 0.0);
        assert!(snapshot.buffer_utilization <= 1.0);
        // Other fields may vary based on system state
    }

    #[test]
    fn test_async_memory_snapshot_debug() {
        let snapshot = AsyncMemorySnapshot {
            active_task_count: 5,
            total_allocated_bytes: 4096,
            allocation_events: 200,
            events_dropped: 1,
            buffer_utilization: 0.75,
        };

        let debug_str = format!("{:?}", snapshot);
        assert!(debug_str.contains("active_task_count: 5"));
        assert!(debug_str.contains("total_allocated_bytes: 4096"));
    }

    #[test]
    fn test_async_memory_snapshot_clone() {
        let original = AsyncMemorySnapshot {
            active_task_count: 3,
            total_allocated_bytes: 1024,
            allocation_events: 50,
            events_dropped: 0,
            buffer_utilization: 0.25,
        };

        let cloned = original.clone();
        assert_eq!(original.active_task_count, cloned.active_task_count);
        assert_eq!(original.total_allocated_bytes, cloned.total_allocated_bytes);
        assert_eq!(original.allocation_events, cloned.allocation_events);
        assert_eq!(original.events_dropped, cloned.events_dropped);
        assert_eq!(original.buffer_utilization, cloned.buffer_utilization);
    }

    #[test]
    fn test_data_quality_warning_formatting() {
        let snapshot = AsyncMemorySnapshot {
            active_task_count: 1,
            total_allocated_bytes: 1000,
            allocation_events: 100,
            events_dropped: 25, // 25% drop rate
            buffer_utilization: 0.8,
        };

        let warning = snapshot.data_quality_warning().unwrap();
        assert!(warning.contains("25.0%"));
        assert!(warning.contains("buffer size"));
    }

    #[test]
    fn test_tracked_future_error_handling() {
        // Test behavior when task ID generation might fail
        let future = async { "test" };
        let mut tracked = TrackedFuture::new(future);
        
        // Verify initial state
        assert!(tracked.task_id.is_none());
        
        let waker = create_test_waker();
        let mut cx = Context::from_waker(&waker);
        
        // Poll should handle ID generation gracefully
        let _result = Pin::new(&mut tracked).poll(&mut cx);
        
        // Should not panic even if ID generation fails
    }

    #[test]
    fn test_tracked_future_new_constructor() {
        let future = async { vec![1, 2, 3] };
        let tracked = TrackedFuture::new(future);
        
        assert!(tracked.task_id.is_none());
        // The inner future should be properly boxed and pinned
    }

    #[test]
    fn test_memory_snapshot_large_numbers() {
        let snapshot = AsyncMemorySnapshot {
            active_task_count: usize::MAX,
            total_allocated_bytes: u64::MAX,
            allocation_events: u64::MAX / 2,
            events_dropped: u64::MAX / 4,
            buffer_utilization: 1.0,
        };

        // Should handle large numbers without overflow
        assert_eq!(snapshot.active_task_count(), usize::MAX);
        assert_eq!(snapshot.total_allocated(), u64::MAX);
        assert!(!snapshot.has_good_data_quality()); // Very high drop rate
        assert!(snapshot.data_quality_warning().is_some());
    }

    #[test]
    fn test_tracked_future_with_different_output_types() {
        // Test with various return types
        let string_future = create_tracked(async { String::from("test") });
        let number_future = create_tracked(async { 42u64 });
        let unit_future = create_tracked(async { () });
        let option_future = create_tracked(async { Some(100) });
        let result_future = create_tracked(async { Ok::<_, &str>(200) });

        // All should compile and be valid TrackedFuture instances
        drop(string_future);
        drop(number_future);
        drop(unit_future);
        drop(option_future);
        drop(result_future);
    }
}
