//! Lock-free thread-local memory tracking with intelligent sampling and binary file output.
//!
//! This module implements the multi-threaded approach that eliminates lock contention
//! by using completely independent thread-local tracking with binary file intermediates
//! and intelligent sampling strategies.
//!
//! Key features:
//! - Zero-lock design: Each thread operates independently
//! - Intelligent sampling: Frequency + size dual-dimension sampling
//! - Binary format: Uses postcard for zero-overhead serialization
//! - Thread isolation: Complete elimination of shared state
//! - Performance focused: Minimal overhead on target application

use postcard;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// EventType moved to analysis.rs to avoid duplication
pub use crate::lockfree::analysis::EventType;

/// Core tracking event - optimized for postcard serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub timestamp: u64,
    pub ptr: usize,
    pub size: usize,
    pub call_stack_hash: u64,
    pub event_type: EventType,
    pub thread_id: u64,
}

/// Frequency tracking data for intelligent sampling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyData {
    pub call_stack_hash: u64,
    pub frequency: u64,
    pub total_size: usize,
    pub thread_id: u64,
}

// Use sampling config from sampling module
pub use crate::lockfree::sampling::SamplingConfig;

// SamplingConfig is now defined in sampling.rs

// Thread-local tracker that operates completely independently
// Uses RefCell for interior mutability without locks
thread_local! {
    static THREAD_TRACKER: std::cell::RefCell<Option<ThreadLocalTracker>> = 
        std::cell::RefCell::new(None);
}

/// Thread-local memory tracker
#[derive(Debug)]
pub struct ThreadLocalTracker {
    /// Thread ID for identification
    thread_id: u64,
    /// Event buffer for batch writing
    event_buffer: Vec<Event>,
    /// Call stack frequency tracking
    call_stack_frequencies: HashMap<u64, u64>,
    /// Call stack size tracking for intelligent sampling
    call_stack_sizes: HashMap<u64, usize>,
    /// Buffer size before flushing to disk
    buffer_size: usize,
    /// File handle for writing events
    file_path: std::path::PathBuf,
    /// Sampling configuration
    config: SamplingConfig,
    /// Random number generator for sampling decisions
    rng_state: u64,
}

impl ThreadLocalTracker {
    /// Creates a new thread-local tracker with specified configuration
    /// 
    /// # Arguments
    /// * `output_dir` - Directory for storing thread-specific binary files
    /// * `config` - Sampling configuration for intelligent allocation tracking
    /// 
    /// # Returns
    /// Result containing the configured tracker or IO error
    pub fn new(output_dir: &std::path::Path, config: SamplingConfig) -> std::io::Result<Self> {
        let thread_id = get_thread_id();
        let file_path = output_dir.join(format!("memscope_thread_{}.bin", thread_id));
        
        // Ensure output directory exists before creating tracker
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Pre-allocate buffer for optimal performance
        let event_buffer = Vec::with_capacity(1000);
        
        Ok(Self {
            thread_id,
            event_buffer,
            call_stack_frequencies: HashMap::new(),
            call_stack_sizes: HashMap::new(),
            buffer_size: 1000,
            file_path,
            config,
            rng_state: thread_id, // Thread ID as deterministic seed
        })
    }

    /// Tracks allocation with intelligent sampling based on size and frequency
    /// 
    /// # Arguments
    /// * `ptr` - Memory pointer address
    /// * `size` - Allocation size in bytes
    /// * `call_stack_hash` - Hash of the call stack for frequency tracking
    /// 
    /// # Returns
    /// Result indicating success or error during tracking/flushing
    pub fn track_allocation(&mut self, ptr: usize, size: usize, call_stack_hash: u64) -> Result<(), Box<dyn std::error::Error>> {
        // Update frequency tracking for intelligent sampling
        let frequency = self.call_stack_frequencies.entry(call_stack_hash).or_insert(0);
        *frequency += 1;
        let current_frequency = *frequency;
        self.call_stack_sizes.insert(call_stack_hash, size);

        // Apply intelligent sampling decision
        if self.should_sample_allocation(size, current_frequency) {
            let event = Event {
                timestamp: get_timestamp(),
                ptr,
                size,
                call_stack_hash,
                event_type: EventType::Allocation,
                thread_id: self.thread_id,
            };

            self.event_buffer.push(event);

            // Flush buffer when full to prevent memory bloat
            if self.event_buffer.len() >= self.buffer_size {
                self.flush_buffer()?;
            }
        }

        Ok(())
    }

    /// Tracks deallocation events for memory balance analysis
    /// 
    /// # Arguments
    /// * `ptr` - Memory pointer address being deallocated
    /// * `call_stack_hash` - Hash of the call stack for correlation
    /// 
    /// # Returns
    /// Result indicating success or error during tracking/flushing
    pub fn track_deallocation(&mut self, ptr: usize, call_stack_hash: u64) -> Result<(), Box<dyn std::error::Error>> {
        // Always track deallocations for accurate memory balance
        let event = Event {
            timestamp: get_timestamp(),
            ptr,
            size: 0, // Size not required for deallocation events
            call_stack_hash,
            event_type: EventType::Deallocation,
            thread_id: self.thread_id,
        };

        self.event_buffer.push(event);

        // Flush buffer when full to prevent memory bloat
        if self.event_buffer.len() >= self.buffer_size {
            self.flush_buffer()?;
        }

        Ok(())
    }

    /// Determines sampling decision using dual-dimension analysis
    /// 
    /// Combines size-based and frequency-based sampling for intelligent allocation tracking.
    /// Large allocations and high-frequency patterns receive higher sampling rates.
    /// 
    /// # Arguments
    /// * `size` - Allocation size in bytes
    /// * `frequency` - Current frequency count for this call stack
    /// 
    /// # Returns
    /// Boolean indicating whether to sample this allocation
    fn should_sample_allocation(&mut self, size: usize, frequency: u64) -> bool {
        // Determine base sampling rate by allocation size
        let size_based_rate = match size {
            s if s >= self.config.large_threshold => self.config.large_allocation_rate,
            s if s >= self.config.medium_threshold => self.config.medium_allocation_rate,
            _ => self.config.small_allocation_rate,
        };

        // Apply frequency-based boost for performance-critical patterns
        let frequency_multiplier = if frequency > self.config.frequency_threshold {
            // High frequency allocations indicate performance hotspots
            (frequency as f64 / self.config.frequency_threshold as f64).min(10.0)
        } else {
            1.0
        };

        let final_rate = (size_based_rate * frequency_multiplier).min(1.0);

        // Fast linear congruential generator for sampling decision
        self.rng_state = self.rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let random_value = (self.rng_state >> 16) as f64 / 65536.0;

        random_value < final_rate
    }

    /// Flush event buffer to binary file
    fn flush_buffer(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.event_buffer.is_empty() {
            return Ok(());
        }

        // Serialize events using postcard (zero-overhead binary format)
        let serialized = postcard::to_allocvec(&self.event_buffer)?;

        // Append to file (create if doesn't exist)
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;

        // Write length header (4 bytes) followed by data
        let len = serialized.len() as u32;
        file.write_all(&len.to_le_bytes())?;
        file.write_all(&serialized)?;
        file.flush()?;

        // Clear buffer
        self.event_buffer.clear();

        Ok(())
    }

    /// Export frequency data at the end of tracking
    pub fn export_frequency_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        let frequency_data: Vec<FrequencyData> = self.call_stack_frequencies
            .iter()
            .map(|(&call_stack_hash, &frequency)| {
                let total_size = self.call_stack_sizes.get(&call_stack_hash).copied().unwrap_or(0) * frequency as usize;
                FrequencyData {
                    call_stack_hash,
                    frequency,
                    total_size,
                    thread_id: self.thread_id,
                }
            })
            .collect();

        let frequency_file = self.file_path.with_extension("freq");
        let serialized = postcard::to_allocvec(&frequency_data)?;

        std::fs::write(frequency_file, serialized)?;
        Ok(())
    }

    /// Force flush any remaining events
    pub fn finalize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.flush_buffer()?;
        self.export_frequency_data()?;
        Ok(())
    }
}

impl Drop for ThreadLocalTracker {
    fn drop(&mut self) {
        // Ensure all data is flushed on drop
        let _ = self.finalize();
    }
}

/// Get unique thread ID
fn get_thread_id() -> u64 {
    static THREAD_COUNTER: AtomicU64 = AtomicU64::new(1);
    
    thread_local! {
        static THREAD_ID: u64 = THREAD_COUNTER.fetch_add(1, Ordering::Relaxed);
    }
    
    THREAD_ID.with(|&id| id)
}

/// Gets current timestamp in nanoseconds with fallback to zero
/// 
/// Uses system time but handles clock errors gracefully without panicking
fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0)
}

/// Calculate hash for call stack (simplified for now)
pub fn calculate_call_stack_hash(call_stack: &[usize]) -> u64 {
    let mut hasher = DefaultHasher::new();
    call_stack.hash(&mut hasher);
    hasher.finish()
}

/// Global API functions for thread-local tracking

/// Initialize thread-local tracker for current thread
pub fn init_thread_tracker(output_dir: &std::path::Path, config: Option<SamplingConfig>) -> Result<(), Box<dyn std::error::Error>> {
    let config = config.unwrap_or_default();
    
    THREAD_TRACKER.with(|tracker| {
        let mut tracker_ref = tracker.borrow_mut();
        if tracker_ref.is_none() {
            *tracker_ref = Some(ThreadLocalTracker::new(output_dir, config)?);
        }
        Ok(())
    })
}

/// Track allocation in current thread using lock-free approach
pub fn track_allocation_lockfree(ptr: usize, size: usize, call_stack: &[usize]) -> Result<(), Box<dyn std::error::Error>> {
    let call_stack_hash = calculate_call_stack_hash(call_stack);
    
    THREAD_TRACKER.with(|tracker| {
        let mut tracker_ref = tracker.borrow_mut();
        if let Some(ref mut t) = *tracker_ref {
            t.track_allocation(ptr, size, call_stack_hash)
        } else {
            Err("Thread tracker not initialized. Call init_thread_tracker() first.".into())
        }
    })
}

/// Track deallocation in current thread using lock-free approach
pub fn track_deallocation_lockfree(ptr: usize, call_stack: &[usize]) -> Result<(), Box<dyn std::error::Error>> {
    let call_stack_hash = calculate_call_stack_hash(call_stack);
    
    THREAD_TRACKER.with(|tracker| {
        let mut tracker_ref = tracker.borrow_mut();
        if let Some(ref mut t) = *tracker_ref {
            t.track_deallocation(ptr, call_stack_hash)
        } else {
            Err("Thread tracker not initialized. Call init_thread_tracker() first.".into())
        }
    })
}

/// Finalize tracking for current thread
pub fn finalize_thread_tracker() -> Result<(), Box<dyn std::error::Error>> {
    THREAD_TRACKER.with(|tracker| {
        let mut tracker_ref = tracker.borrow_mut();
        if let Some(ref mut t) = *tracker_ref {
            t.finalize()
        } else {
            Ok(()) // No tracker initialized, nothing to finalize
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_thread_local_tracking_basic() {
        let temp_dir = std::env::temp_dir().join("memscope_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let config = SamplingConfig::default();
        init_thread_tracker(&temp_dir, Some(config)).unwrap();

        let call_stack = vec![0x1000, 0x2000, 0x3000];
        track_allocation_lockfree(0x4000, 1024, &call_stack)
            .expect("Allocation tracking should succeed");
        track_deallocation_lockfree(0x4000, &call_stack)
            .expect("Deallocation tracking should succeed");

        finalize_thread_tracker().unwrap();

        // Check that files were created
        let thread_id = get_thread_id();
        let event_file = temp_dir.join(format!("memscope_thread_{}.bin", thread_id));
        let freq_file = temp_dir.join(format!("memscope_thread_{}.freq", thread_id));

        assert!(event_file.exists());
        assert!(freq_file.exists());

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_multi_thread_independence() {
        let temp_dir = std::env::temp_dir().join("memscope_multithread_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let thread_count = 10;
        let allocations_per_thread = 100;
        let counter = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..thread_count)
            .map(|thread_idx| {
                let temp_dir = temp_dir.clone();
                let counter = Arc::clone(&counter);
                
                thread::spawn(move || {
                    // Initialize tracker for this thread
                    init_thread_tracker(&temp_dir, None).unwrap();

                    for i in 0..allocations_per_thread {
                        let ptr = (thread_idx * 10000 + i * 8) as usize;
                        let size = 64 + (i % 10) * 64; // Varying sizes
                        let call_stack = vec![0x1000 + thread_idx, 0x2000 + i];

                        track_allocation_lockfree(ptr, size, &call_stack)
                            .expect("Allocation tracking should succeed");
                        
                        // Simulate some deallocations
                        if i % 3 == 0 {
                            track_deallocation_lockfree(ptr, &call_stack)
                                .expect("Deallocation tracking should succeed");
                        }

                        counter.fetch_add(1, Ordering::Relaxed);
                    }

                    finalize_thread_tracker().unwrap();
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all operations completed
        assert_eq!(counter.load(Ordering::Relaxed), thread_count * allocations_per_thread);

        // Verify each thread created its own files
        for thread_id in 1..=thread_count {
            let event_file = temp_dir.join(format!("memscope_thread_{}.bin", thread_id));
            let freq_file = temp_dir.join(format!("memscope_thread_{}.freq", thread_id));
            
            assert!(event_file.exists(), "Event file missing for thread {}", thread_id);
            assert!(freq_file.exists(), "Frequency file missing for thread {}", thread_id);
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_intelligent_sampling() {
        let mut config = SamplingConfig::default();
        // Make sampling more predictable for testing
        config.small_allocation_rate = 0.1; // 10% for small
        config.medium_allocation_rate = 0.5; // 50% for medium
        config.large_allocation_rate = 1.0; // 100% for large

        let temp_dir = std::env::temp_dir().join("memscope_sampling_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let mut tracker = ThreadLocalTracker::new(&temp_dir, config).unwrap();

        let mut sampled_small = 0;
        let mut sampled_large = 0;
        let total_small = 100;
        let total_large = 10;

        // Test small allocations (should have lower sample rate)
        for _i in 0..total_small {
            let size = 512; // Small allocation
            let was_sampled = tracker.should_sample_allocation(size, 1);
            if was_sampled {
                sampled_small += 1;
            }
        }

        // Test large allocations (should have higher sample rate)
        for _i in 0..total_large {
            let size = 20 * 1024; // Large allocation
            let was_sampled = tracker.should_sample_allocation(size, 1);
            if was_sampled {
                sampled_large += 1;
            }
        }

        // Large allocations should be sampled more frequently
        assert!(sampled_large as f64 / total_large as f64 > sampled_small as f64 / total_small as f64);

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}