//! Lock-free event buffering for async memory tracking
//!
//! Provides high-performance, lock-free ring buffers for collecting allocation
//! events from async tasks with quality monitoring and overflow handling.

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::async_memory::error::{AsyncError, AsyncResult, BufferType};
use crate::async_memory::task_id::TaskId;
use crate::async_memory::DEFAULT_BUFFER_SIZE;

/// Memory allocation or deallocation event
///
/// Optimized for cache efficiency with 64-byte alignment to avoid false sharing.
/// Uses minimal fields to reduce memory overhead while capturing essential data.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct AllocationEvent {
    /// Unique task identifier
    pub task_id: TaskId,
    /// Memory pointer address
    pub ptr: usize,
    /// Allocation size in bytes
    pub size: usize,
    /// Timestamp (TSC ticks or nanoseconds)
    pub timestamp: u64,
    /// Event type: 0=allocation, 1=deallocation
    pub event_type: u8,
    /// Reserved for future use, ensures 64-byte alignment
    _padding: [u8; 31],
}

impl AllocationEvent {
    /// Create new allocation event
    pub fn allocation(task_id: TaskId, ptr: usize, size: usize, timestamp: u64) -> Self {
        Self {
            task_id,
            ptr,
            size,
            timestamp,
            event_type: 0,
            _padding: [0; 31],
        }
    }

    /// Create new deallocation event
    pub fn deallocation(task_id: TaskId, ptr: usize, size: usize, timestamp: u64) -> Self {
        Self {
            task_id,
            ptr,
            size,
            timestamp,
            event_type: 1,
            _padding: [0; 31],
        }
    }

    /// Check if this is an allocation event
    pub fn is_allocation(&self) -> bool {
        self.event_type == 0
    }

    /// Check if this is a deallocation event
    pub fn is_deallocation(&self) -> bool {
        self.event_type == 1
    }
}

/// Lock-free ring buffer for allocation events
///
/// Uses single-producer, single-consumer (SPSC) design where the producer
/// (allocator hook) runs on the application thread and the consumer
/// (aggregator) runs on a dedicated background thread.
pub struct EventBuffer {
    /// Pre-allocated ring buffer storage using UnsafeCell for interior mutability
    events: UnsafeCell<Box<[AllocationEvent; DEFAULT_BUFFER_SIZE]>>,
    /// Write position (modified only by producer)
    write_pos: AtomicUsize,
    /// Read position (modified only by consumer)
    read_pos: AtomicUsize,
    /// Count of dropped events due to buffer overflow
    dropped_events: AtomicUsize,
    /// Buffer size mask for efficient modulo operations
    mask: usize,
}

impl EventBuffer {
    /// Create new event buffer with default size
    pub fn new() -> Self {
        Self {
            events: UnsafeCell::new(Box::new(
                [AllocationEvent::allocation(0, 0, 0, 0); DEFAULT_BUFFER_SIZE],
            )),
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
            dropped_events: AtomicUsize::new(0),
            mask: DEFAULT_BUFFER_SIZE - 1,
        }
    }

    /// Push event to buffer (producer side)
    ///
    /// Returns Ok(()) if successful, Err with drop count if buffer full.
    /// Never blocks - drops events instead to maintain real-time performance.
    #[inline(always)]
    pub fn push(&self, event: AllocationEvent) -> AsyncResult<()> {
        let write_pos = self.write_pos.load(Ordering::Relaxed);
        let next_write = (write_pos + 1) & self.mask;
        let read_pos = self.read_pos.load(Ordering::Acquire);

        if next_write == read_pos {
            // Buffer full - record drop and return error
            let dropped = self.dropped_events.fetch_add(1, Ordering::Relaxed) + 1;
            return Err(AsyncError::buffer_management(
                BufferType::AllocationEvents,
                "Ring buffer overflow - event dropped",
                Some(dropped),
            ));
        }

        // Safe write - only this thread writes to this position
        // Use UnsafeCell to get mutable access
        unsafe {
            let events_ptr = self.events.get();
            let event_ptr = (*events_ptr).as_mut_ptr().add(write_pos);
            std::ptr::write_volatile(event_ptr, event);
        }

        // Publish write with release ordering
        self.write_pos.store(next_write, Ordering::Release);
        Ok(())
    }

    /// Pop event from buffer (consumer side)
    ///
    /// Returns Some(event) if available, None if buffer empty.
    /// Only called by the aggregator thread.
    pub fn pop(&self) -> Option<AllocationEvent> {
        let read_pos = self.read_pos.load(Ordering::Relaxed);
        let write_pos = self.write_pos.load(Ordering::Acquire);

        if read_pos == write_pos {
            return None; // Buffer empty
        }

        // Safe read - only this thread reads from this position
        let event = unsafe {
            let events_ptr = self.events.get();
            let event_ptr = (*events_ptr).as_ptr().add(read_pos);
            std::ptr::read_volatile(event_ptr)
        };

        // Advance read position
        let next_read = (read_pos + 1) & self.mask;
        self.read_pos.store(next_read, Ordering::Release);

        Some(event)
    }

    /// Get current number of events in buffer
    pub fn len(&self) -> usize {
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let read_pos = self.read_pos.load(Ordering::Acquire);
        (write_pos.wrapping_sub(read_pos)) & self.mask
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let read_pos = self.read_pos.load(Ordering::Acquire);
        write_pos == read_pos
    }

    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        DEFAULT_BUFFER_SIZE - 1 // One slot reserved for full/empty distinction
    }

    /// Get number of dropped events
    pub fn dropped_count(&self) -> usize {
        self.dropped_events.load(Ordering::Relaxed)
    }

    /// Reset dropped event counter (for testing)
    #[cfg(test)]
    pub fn reset_dropped_count(&self) {
        self.dropped_events.store(0, Ordering::Relaxed);
    }

    /// Drain all events from buffer
    ///
    /// Returns vector of all current events, clearing the buffer.
    /// Used for bulk processing and testing.
    pub fn drain(&self) -> Vec<AllocationEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.pop() {
            events.push(event);
        }
        events
    }
}

impl Default for EventBuffer {
    fn default() -> Self {
        Self::new()
    }
}

// Safe to share between threads because:
// - Only one thread (producer) writes to write_pos and events at write_pos
// - Only one thread (consumer) reads from read_pos and events at read_pos
// - Atomic operations ensure proper synchronization
unsafe impl Sync for EventBuffer {}
unsafe impl Send for EventBuffer {}

// Thread-local event buffer storage
//
// Each thread gets its own event buffer to avoid contention.
// Uses UnsafeCell for zero-overhead access from allocator hooks.
thread_local! {
    static THREAD_EVENT_BUFFER: UnsafeCell<EventBuffer> = UnsafeCell::new(EventBuffer::new());
}

/// Get reference to current thread's event buffer
///
/// Used by allocator hooks to record allocation events.
/// Safe because each thread has exclusive access to its own buffer.
#[inline(always)]
pub fn with_thread_buffer<F, R>(f: F) -> R
where
    F: FnOnce(&EventBuffer) -> R,
{
    THREAD_EVENT_BUFFER.with(|buffer| {
        // Safe: thread-local access, no concurrent access possible
        let buffer_ref = unsafe { &*buffer.get() };
        f(buffer_ref)
    })
}

/// Record allocation event in current thread's buffer
///
/// High-performance path called from global allocator hook.
/// Optimized for minimal overhead in the allocation fast path.
#[inline(always)]
pub fn record_allocation_event(
    task_id: TaskId,
    ptr: usize,
    size: usize,
    timestamp: u64,
    is_allocation: bool,
) -> AsyncResult<()> {
    let event = if is_allocation {
        AllocationEvent::allocation(task_id, ptr, size, timestamp)
    } else {
        AllocationEvent::deallocation(task_id, ptr, size, timestamp)
    };

    with_thread_buffer(|buffer| buffer.push(event))
}

/// Collect events from all thread buffers
///
/// Called by aggregator to gather events for processing.
/// This is a simplified version - production implementation would
/// maintain a registry of all thread buffers.
pub fn collect_all_events() -> Vec<AllocationEvent> {
    // In real implementation, this would iterate over all thread buffers
    // For now, just return events from current thread
    with_thread_buffer(|buffer| buffer.drain())
}

/// Get buffer statistics for monitoring
#[derive(Debug, Clone)]
pub struct BufferStats {
    /// Current number of events in buffer
    pub current_events: usize,
    /// Buffer capacity
    pub capacity: usize,
    /// Total events dropped due to overflow
    pub events_dropped: usize,
    /// Buffer utilization ratio (0.0 to 1.0)
    pub utilization: f64,
}

impl BufferStats {
    /// Get utilization warning level
    pub fn warning_level(&self) -> Option<&'static str> {
        match self.utilization {
            u if u >= 0.95 => Some("critical"),
            u if u >= 0.85 => Some("high"),
            u if u >= 0.75 => Some("medium"),
            _ => None,
        }
    }
}

/// Get current thread buffer statistics
pub fn get_buffer_stats() -> BufferStats {
    with_thread_buffer(|buffer| {
        let current_events = buffer.len();
        let capacity = buffer.capacity();
        let events_dropped = buffer.dropped_count();
        let utilization = current_events as f64 / capacity as f64;

        BufferStats {
            current_events,
            capacity,
            events_dropped,
            utilization,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation_event_creation() {
        let alloc_event = AllocationEvent::allocation(12345, 0x1000, 1024, 567890);
        assert_eq!(alloc_event.task_id, 12345);
        assert_eq!(alloc_event.ptr, 0x1000);
        assert_eq!(alloc_event.size, 1024);
        assert_eq!(alloc_event.timestamp, 567890);
        assert!(alloc_event.is_allocation());
        assert!(!alloc_event.is_deallocation());

        let dealloc_event = AllocationEvent::deallocation(12345, 0x1000, 1024, 567891);
        assert!(dealloc_event.is_deallocation());
        assert!(!dealloc_event.is_allocation());
    }

    #[test]
    fn test_event_buffer_basic_operations() {
        let buffer = EventBuffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.dropped_count(), 0);

        let event = AllocationEvent::allocation(1, 0x1000, 100, 123);
        buffer.push(event).expect("Failed to push event");

        assert!(!buffer.is_empty());
        assert_eq!(buffer.len(), 1);

        let popped = buffer.pop().expect("Failed to pop event");
        assert_eq!(popped.task_id, 1);
        assert_eq!(popped.ptr, 0x1000);
        assert_eq!(popped.size, 100);

        assert!(buffer.is_empty());
        assert!(buffer.pop().is_none());
    }

    #[test]
    fn test_buffer_overflow_handling() {
        let buffer = EventBuffer::new();
        buffer.reset_dropped_count();

        // Fill buffer to capacity
        let capacity = buffer.capacity();
        for i in 0..capacity {
            let event = AllocationEvent::allocation(i as TaskId, i, 100, i as u64);
            buffer.push(event).expect("Failed to push event");
        }

        // Next push should fail with overflow
        let overflow_event = AllocationEvent::allocation(99999, 0x9999, 100, 99999);
        let result = buffer.push(overflow_event);
        assert!(result.is_err());
        assert_eq!(buffer.dropped_count(), 1);

        // Buffer should still be full
        assert_eq!(buffer.len(), capacity);

        // Pop one event and push should succeed again
        buffer.pop().expect("Failed to pop event");
        buffer
            .push(overflow_event)
            .expect("Failed to push after pop");
    }

    #[test]
    fn test_buffer_wraparound() {
        let buffer = EventBuffer::new();
        let capacity = buffer.capacity();

        // Fill and empty buffer multiple times to test wraparound
        for round in 0..3 {
            for i in 0..capacity / 2 {
                let event = AllocationEvent::allocation(
                    (round * 1000 + i) as TaskId,
                    i,
                    100,
                    (round * 1000 + i) as u64,
                );
                buffer.push(event).expect("Failed to push event");
            }

            let events = buffer.drain();
            assert_eq!(events.len(), capacity / 2);

            // Verify correct order
            for (i, event) in events.iter().enumerate() {
                assert_eq!(event.task_id, (round * 1000 + i) as TaskId);
            }
        }
    }

    #[test]
    fn test_thread_local_buffer() {
        use std::thread;

        // Test that each thread gets its own buffer
        let handle1 = thread::spawn(|| {
            record_allocation_event(1, 0x1000, 100, 123, true).expect("Failed to record");
            get_buffer_stats().current_events
        });

        let handle2 = thread::spawn(|| {
            record_allocation_event(2, 0x2000, 200, 456, true).expect("Failed to record");
            get_buffer_stats().current_events
        });

        assert_eq!(handle1.join().expect("Thread 1 panicked"), 1);
        assert_eq!(handle2.join().expect("Thread 2 panicked"), 1);

        // Main thread buffer should be empty
        assert_eq!(get_buffer_stats().current_events, 0);
    }

    #[test]
    fn test_buffer_stats() {
        with_thread_buffer(|buffer| buffer.reset_dropped_count());

        // Empty buffer
        let stats = get_buffer_stats();
        assert_eq!(stats.current_events, 0);
        assert_eq!(stats.utilization, 0.0);
        assert!(stats.warning_level().is_none());

        // Fill buffer to trigger warnings
        let capacity = get_buffer_stats().capacity;
        let high_fill = (capacity as f64 * 0.9) as usize;

        for i in 0..high_fill {
            record_allocation_event(i as TaskId, i, 100, i as u64, true).expect("Failed to record");
        }

        let stats = get_buffer_stats();
        assert!(stats.utilization >= 0.85);
        assert!(stats.warning_level().is_some());
    }

    #[test]
    fn test_concurrent_producer_consumer() {
        use std::sync::Arc;
        use std::thread;
        use std::time::Duration;

        let buffer = Arc::new(EventBuffer::new());
        let producer_buffer = Arc::clone(&buffer);
        let consumer_buffer = Arc::clone(&buffer);

        // Producer thread
        let producer = thread::spawn(move || {
            for i in 0..1000 {
                let event =
                    AllocationEvent::allocation(i as TaskId, (i * 1000) as usize, 100, i as u64);
                // Ignore overflow errors for this test
                let _ = producer_buffer.push(event);
            }
        });

        // Consumer thread
        let consumer = thread::spawn(move || {
            let mut consumed = 0;
            let mut last_task_id = None;

            for _ in 0..100 {
                while let Some(event) = consumer_buffer.pop() {
                    // Verify ordering within what we can consume
                    if let Some(last_id) = last_task_id {
                        assert!(event.task_id >= last_id);
                    }
                    last_task_id = Some(event.task_id);
                    consumed += 1;
                }
                thread::sleep(Duration::from_micros(10));
            }
            consumed
        });

        producer.join().expect("Producer thread panicked");
        let consumed = consumer.join().expect("Consumer thread panicked");

        // Should have consumed some events (exact number depends on timing)
        assert!(consumed > 0);
        assert!(consumed <= 1000);
    }
}
