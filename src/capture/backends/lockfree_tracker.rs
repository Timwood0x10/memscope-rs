//! Lockfree memory tracker implementation.
//!
//! This module contains the ThreadLocalTracker for thread-local memory tracking.

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

use super::lockfree_types::{Event, EventType, MemoryStats};

/// Thread-local memory tracker for lock-free operation.
///
/// This tracker stores memory events in a thread-local buffer to avoid
/// synchronization overhead during memory allocation tracking.
pub struct ThreadLocalTracker {
    /// Thread ID for this tracker
    thread_id: ThreadId,
    /// Memory events buffer
    events: Arc<Mutex<Vec<Event>>>,
    /// Active allocations tracking
    active_allocations: Arc<Mutex<HashMap<usize, usize>>>,
    /// Memory statistics
    stats: Arc<Mutex<MemoryStats>>,
    /// Output file path for this thread
    output_file: PathBuf,
    /// Sampling rate (0.0 to 1.0)
    sample_rate: f64,
    /// Total allocations seen
    total_seen: Arc<Mutex<usize>>,
    /// Number of allocations actually tracked
    total_tracked: Arc<Mutex<usize>>,
}

impl ThreadLocalTracker {
    /// Create a new thread-local tracker.
    ///
    /// # Arguments
    /// * `thread_id` - The thread ID for this tracker
    /// * `output_file` - Path to the output file for this thread
    /// * `sample_rate` - Sampling rate (0.0 to 1.0)
    pub fn new(thread_id: ThreadId, output_file: PathBuf, sample_rate: f64) -> Self {
        Self {
            thread_id,
            events: Arc::new(Mutex::new(Vec::new())),
            active_allocations: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(MemoryStats::default())),
            output_file,
            sample_rate: sample_rate.clamp(0.0, 1.0),
            total_seen: Arc::new(Mutex::new(0)),
            total_tracked: Arc::new(Mutex::new(0)),
        }
    }

    /// Track a memory allocation.
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer
    /// * `size` - Memory size in bytes
    /// * `call_stack_hash` - Hash of the call stack
    pub fn track_allocation(&self, ptr: usize, size: usize, call_stack_hash: u64) {
        // Check sampling rate
        if self.sample_rate < 1.0 {
            let sample_decision = rand::random::<f64>();
            if sample_decision >= self.sample_rate {
                return; // Skip this allocation
            }
        }

        // Update counters
        if let Ok(mut total_seen) = self.total_seen.try_lock() {
            *total_seen += 1;
        }

        if let Ok(mut total_tracked) = self.total_tracked.try_lock() {
            *total_tracked += 1;
        }

        // Create allocation event
        let event = Event::allocation(ptr, size, call_stack_hash, self.thread_id);

        // Record event
        if let Ok(mut events) = self.events.try_lock() {
            events.push(event);
        }

        // Track active allocation
        if let Ok(mut active) = self.active_allocations.try_lock() {
            active.insert(ptr, size);
        }

        // Update statistics
        if let Ok(mut stats) = self.stats.try_lock() {
            stats.total_allocations += 1;
            stats.total_allocated += size;
            stats.active_memory += size;
            stats.peak_memory = stats.peak_memory.max(stats.active_memory);
        }
    }

    /// Track a memory deallocation.
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer
    /// * `call_stack_hash` - Hash of the call stack
    pub fn track_deallocation(&self, ptr: usize, call_stack_hash: u64) {
        // Get allocation size if available
        let size = if let Ok(mut active) = self.active_allocations.try_lock() {
            active.remove(&ptr)
        } else {
            None
        };

        // Create deallocation event
        let event_size = size.unwrap_or(0);
        let event = Event::deallocation(ptr, event_size, call_stack_hash, self.thread_id);

        // Record event
        if let Ok(mut events) = self.events.try_lock() {
            events.push(event);
        }

        // Update statistics
        if let Ok(mut stats) = self.stats.try_lock() {
            stats.total_deallocations += 1;
            stats.total_deallocated += event_size;
            if let Some(alloc_size) = size {
                stats.active_memory = stats.active_memory.saturating_sub(alloc_size);
            }
        }
    }

    /// Get memory statistics.
    pub fn get_stats(&self) -> MemoryStats {
        if let Ok(stats) = self.stats.try_lock() {
            stats.clone()
        } else {
            MemoryStats::default()
        }
    }

    /// Get sampling statistics.
    pub fn get_sampling_stats(&self) -> (usize, usize) {
        let seen = if let Ok(total_seen) = self.total_seen.try_lock() {
            *total_seen
        } else {
            0
        };

        let tracked = if let Ok(total_tracked) = self.total_tracked.try_lock() {
            *total_tracked
        } else {
            0
        };

        (seen, tracked)
    }

    /// Finalize the tracker and write events to file.
    pub fn finalize(&self) -> std::io::Result<()> {
        // Collect all events
        let events = if let Ok(mut events) = self.events.try_lock() {
            std::mem::take(&mut *events)
        } else {
            return Ok(()); // Skip if we can't get the lock
        };

        // Write events to file
        if events.is_empty() {
            return Ok(());
        }

        // Create parent directory if needed
        if let Some(parent) = self.output_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write events in binary format
        let mut file = File::create(&self.output_file)?;

        // Write header
        let header = "MEMSCOPE_LOCKFREE".to_string();
        file.write_all(header.as_bytes())?;

        // Write events
        for event in events {
            self.write_event(&mut file, &event)?;
        }

        file.flush()?;
        Ok(())
    }

    /// Write a single event to file.
    fn write_event(&self, file: &mut File, event: &Event) -> std::io::Result<()> {
        // Write event type
        let event_type_byte = match event.event_type {
            EventType::Allocation => 1u8,
            EventType::Deallocation => 2u8,
        };
        file.write_all(&event_type_byte.to_le_bytes())?;

        // Write timestamp
        file.write_all(&event.timestamp.to_le_bytes())?;

        // Write pointer
        file.write_all(&event.ptr.to_le_bytes())?;

        // Write size
        file.write_all(&event.size.to_le_bytes())?;

        // Write call stack hash
        file.write_all(&event.call_stack_hash.to_le_bytes())?;

        Ok(())
    }

    /// Get the thread ID.
    pub fn thread_id(&self) -> ThreadId {
        self.thread_id
    }

    /// Get the output file path.
    pub fn output_file(&self) -> &PathBuf {
        &self.output_file
    }

    /// Get the number of buffered events.
    pub fn event_count(&self) -> usize {
        if let Ok(events) = self.events.try_lock() {
            events.len()
        } else {
            0
        }
    }

    /// Clear all buffered events.
    pub fn clear_events(&self) {
        if let Ok(mut events) = self.events.try_lock() {
            events.clear();
        }
    }
}

impl Drop for ThreadLocalTracker {
    fn drop(&mut self) {
        // Automatically finalize when tracker is dropped
        if let Err(e) = self.finalize() {
            tracing::warn!("Failed to finalize thread-local tracker: {}", e);
        }
    }
}

/// Calculate a hash of the call stack.
///
/// This function hashes the memory addresses in the call stack to create a
/// compact identifier for tracking allocation patterns.
///
/// # Arguments
/// * `call_stack` - Slice of call stack addresses
///
/// # Returns
/// Hash of the call stack
pub fn calculate_call_stack_hash(call_stack: &[usize]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    for addr in call_stack {
        addr.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_local_tracker_creation() {
        let thread_id = std::thread::current().id();
        let output_file = PathBuf::from("/tmp/test_tracker.bin");
        let tracker = ThreadLocalTracker::new(thread_id, output_file, 1.0);

        assert_eq!(tracker.thread_id(), thread_id);
        assert_eq!(
            tracker.output_file(),
            &PathBuf::from("/tmp/test_tracker.bin")
        );
        assert_eq!(tracker.event_count(), 0);
    }

    #[test]
    fn test_allocation_tracking() {
        let thread_id = std::thread::current().id();
        let output_file = PathBuf::from("/tmp/test_tracker2.bin");
        let tracker = ThreadLocalTracker::new(thread_id, output_file, 1.0);

        // Track allocation
        tracker.track_allocation(0x1000, 1024, 12345);

        // Check stats
        let stats = tracker.get_stats();
        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.total_allocated, 1024);
        assert_eq!(stats.active_memory, 1024);

        // Check event count
        assert_eq!(tracker.event_count(), 1);
    }

    #[test]
    fn test_deallocation_tracking() {
        let thread_id = std::thread::current().id();
        let output_file = PathBuf::from("/tmp/test_tracker3.bin");
        let tracker = ThreadLocalTracker::new(thread_id, output_file, 1.0);

        // Track allocation and deallocation
        tracker.track_allocation(0x1000, 1024, 12345);
        tracker.track_deallocation(0x1000, 12345);

        // Check stats
        let stats = tracker.get_stats();
        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.total_deallocations, 1);
        assert_eq!(stats.active_memory, 0);
    }

    #[test]
    fn test_sampling_rate() {
        let thread_id = std::thread::current().id();
        let output_file = PathBuf::from("/tmp/test_tracker4.bin");

        // Low sampling rate
        let tracker = ThreadLocalTracker::new(thread_id, output_file.clone(), 0.1);

        // Track many allocations
        for i in 0..100 {
            tracker.track_allocation(0x1000 + i * 0x100, 1024, 12345);
        }

        let (seen, tracked) = tracker.get_sampling_stats();
        assert_eq!(seen, 100);
        assert!(tracked < seen); // Should be less than 100 due to sampling
    }

    #[test]
    fn test_call_stack_hash() {
        let call_stack = vec![0x1000, 0x2000, 0x3000];
        let hash1 = calculate_call_stack_hash(&call_stack);
        let hash2 = calculate_call_stack_hash(&call_stack);

        assert_eq!(hash1, hash2); // Same stack should produce same hash

        let different_stack = vec![0x1000, 0x2000, 0x4000];
        let hash3 = calculate_call_stack_hash(&different_stack);
        assert_ne!(hash1, hash3); // Different stack should produce different hash
    }
}

thread_local! {
    static THREAD_TRACKER: std::cell::RefCell<Option<ThreadLocalTracker>> = const { std::cell::RefCell::new(None) };
}

/// Get thread ID as u64
fn get_thread_id() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let thread_id = std::thread::current().id();
    let mut hasher = DefaultHasher::new();
    thread_id.hash(&mut hasher);
    hasher.finish()
}

/// Initialize thread-local tracker for current thread
///
/// # Arguments
/// * `output_dir` - Directory to store tracking data
/// * `sample_rate` - Sampling rate (0.0 to 1.0), defaults to 1.0
pub fn init_thread_tracker(
    output_dir: &std::path::Path,
    sample_rate: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let sample_rate = sample_rate.unwrap_or(1.0);
    let thread_id = std::thread::current().id();
    let output_file = output_dir.join(format!("memscope_thread_{}.bin", get_thread_id()));

    let tracker = ThreadLocalTracker::new(thread_id, output_file, sample_rate);

    THREAD_TRACKER.with(|thread_tracker| {
        *thread_tracker.borrow_mut() = Some(tracker);
    });

    Ok(())
}

/// Track allocation in current thread using lock-free approach
///
/// # Arguments
/// * `ptr` - Memory pointer address
/// * `size` - Allocation size in bytes
/// * `call_stack_hash` - Hash of the call stack for tracking
pub fn track_allocation_lockfree(
    ptr: usize,
    size: usize,
    call_stack_hash: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    THREAD_TRACKER.with(|thread_tracker| {
        if let Some(ref tracker) = *thread_tracker.borrow() {
            tracker.track_allocation(ptr, size, call_stack_hash);
            Ok(())
        } else {
            Err("Thread tracker not initialized. Call init_thread_tracker() first.".into())
        }
    })
}

/// Track deallocation in current thread using lock-free approach
///
/// # Arguments
/// * `ptr` - Memory pointer address
/// * `call_stack_hash` - Hash of the call stack for tracking
pub fn track_deallocation_lockfree(
    ptr: usize,
    call_stack_hash: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    THREAD_TRACKER.with(|thread_tracker| {
        if let Some(ref tracker) = *thread_tracker.borrow() {
            tracker.track_deallocation(ptr, call_stack_hash);
            Ok(())
        } else {
            Err("Thread tracker not initialized. Call init_thread_tracker() first.".into())
        }
    })
}

/// Finalize tracking for current thread and write data to file
pub fn finalize_thread_tracker() -> Result<(), Box<dyn std::error::Error>> {
    THREAD_TRACKER.with(|thread_tracker| {
        let mut tracker_ref = thread_tracker.borrow_mut();
        if let Some(ref mut tracker) = *tracker_ref {
            tracker
                .finalize()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        } else {
            Ok(()) // No tracker initialized, nothing to finalize
        }
    })
}

/// Get the current thread's tracker if initialized
pub fn get_current_tracker() -> Option<ThreadLocalTracker> {
    THREAD_TRACKER.with(|thread_tracker| {
        thread_tracker.borrow().as_ref().map(|tracker| {
            // Create a clone of the key components
            ThreadLocalTracker {
                thread_id: tracker.thread_id,
                events: Arc::clone(&tracker.events),
                active_allocations: Arc::clone(&tracker.active_allocations),
                stats: Arc::clone(&tracker.stats),
                output_file: tracker.output_file.clone(),
                sample_rate: tracker.sample_rate,
                total_seen: Arc::clone(&tracker.total_seen),
                total_tracked: Arc::clone(&tracker.total_tracked),
            }
        })
    })
}

#[cfg(test)]
mod global_api_tests {
    use super::*;

    #[test]
    fn test_init_thread_tracker() {
        let temp_dir = std::env::temp_dir().join("memscope_global_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Test initialization
        let result = init_thread_tracker(&temp_dir, Some(1.0));
        assert!(result.is_ok(), "Should successfully initialize tracker");

        // Test duplicate initialization (should not fail)
        let result2 = init_thread_tracker(&temp_dir, Some(0.5));
        assert!(result2.is_ok(), "Should handle duplicate initialization");
    }

    #[test]
    fn test_track_without_init() {
        // Clear any existing tracker
        THREAD_TRACKER.with(|t| {
            *t.borrow_mut() = None;
        });

        // Try to track without initialization
        let result = track_allocation_lockfree(0x1000, 1024, 12345);
        assert!(result.is_err(), "Should fail without initialization");
    }

    #[test]
    fn test_finalize_without_init() {
        // Clear any existing tracker
        THREAD_TRACKER.with(|t| {
            *t.borrow_mut() = None;
        });

        // Try to finalize without initialization (should not fail)
        let result = finalize_thread_tracker();
        assert!(
            result.is_ok(),
            "Should handle finalization without initialization"
        );
    }

    #[test]
    fn test_global_api_workflow() {
        let temp_dir = std::env::temp_dir().join("memscope_workflow_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Initialize
        init_thread_tracker(&temp_dir, Some(1.0)).unwrap();

        // Track allocation and deallocation
        track_allocation_lockfree(0x1000, 1024, 12345).unwrap();
        track_deallocation_lockfree(0x1000, 12345).unwrap();

        // Get current tracker
        let tracker = get_current_tracker();
        assert!(tracker.is_some(), "Should have active tracker");

        if let Some(t) = tracker {
            let stats = t.get_stats();
            assert_eq!(stats.total_allocations, 1);
            assert_eq!(stats.total_deallocations, 1);
        }

        // Finalize
        finalize_thread_tracker().unwrap();
    }
}
