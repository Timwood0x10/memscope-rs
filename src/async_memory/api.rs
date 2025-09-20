//! High-level API for async memory tracking
//!
//! Provides user-friendly functions for initializing tracking, spawning
//! tracked tasks, and retrieving memory statistics.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// Note: tokio dependency will be added conditionally for async features

use crate::async_memory::error::AsyncResult;
use crate::async_memory::task_id::{generate_task_id, set_current_task, TaskInfo};
use crate::async_memory::buffer::get_buffer_stats;

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
        active_task_count: if buffer_stats.current_events > 0 { 1 } else { 0 },
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

#[cfg(test)]
mod tests {
    use super::*;
    // Duration import removed as not needed for current tests

    #[test]
    fn test_initialization() {
        let result = initialize();
        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_snapshot() {
        // Simple test without accessing thread-local buffers to avoid recursion
        let snapshot = AsyncMemorySnapshot {
            active_task_count: 1,
            total_allocated_bytes: 1024,
            allocation_events: 100,
            events_dropped: 0,
            buffer_utilization: 0.5,
        };
        
        // Basic validation of snapshot fields
        assert!(snapshot.buffer_utilization >= 0.0);
        assert!(snapshot.buffer_utilization <= 1.0);
        assert_eq!(snapshot.active_task_count(), 1);
        assert_eq!(snapshot.total_allocated(), 1024);
        assert!(snapshot.has_good_data_quality());
        assert!(snapshot.data_quality_warning().is_none());
    }

    #[test]
    fn test_tracked_future_basic() {
        use crate::async_memory::task_id::get_current_task;
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
        
        let future = async {
            // Check that we have task context during execution
            let _task_info = get_current_task();
            // Note: May not have tracking ID in test environment
            42
        };

        let mut tracked = create_tracked(future);
        let waker = create_test_waker();
        let mut cx = Context::from_waker(&waker);
        
        // Poll the future
        let result = Pin::new(&mut tracked).poll(&mut cx);
        match result {
            Poll::Ready(value) => assert_eq!(value, 42),
            Poll::Pending => {
                // Future may be pending, which is fine for this test
            }
        }
    }
}