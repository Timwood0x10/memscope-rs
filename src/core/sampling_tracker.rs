//! High-performance sampling-based memory tracker using binary serialization.
//!
//! This module implements the core multi-thread tracking system using:
//! - bincode binary serialization for zero-overhead data storage
//! - Intelligent sampling (frequency + size dimensions)
//! - Thread-local storage with file-based communication
//! - Batch writing for optimal performance

use crate::core::types::TrackingResult;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// Core event data structure optimized for binary serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Timestamp when the event occurred (nanoseconds since epoch)
    pub timestamp: u64,
    /// Memory pointer address
    pub ptr: usize,
    /// Size of memory allocation/operation
    pub size: usize,
    /// Hash of the call stack for frequency analysis
    pub call_stack_hash: u64,
    /// Type of memory operation
    pub event_type: EventType,
    /// Variable name (if available)
    pub var_name: Option<String>,
    /// Type name (if available)
    pub type_name: Option<String>,
}

/// Types of memory operations we track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// Memory allocation
    Allocate,
    /// Memory access/read
    Access,
    /// Memory modification/write
    Modify,
    /// Memory deallocation
    Drop,
    /// Variable clone operation
    Clone { target_ptr: usize },
}

/// Frequency data for call stack analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyData {
    /// Hash of the call stack
    pub call_stack_hash: u64,
    /// How many times this call stack occurred
    pub frequency: u64,
    /// Total memory allocated by this call stack
    pub total_size: usize,
    /// Representative variable name from this call stack
    pub sample_var_name: String,
    /// Representative type name from this call stack
    pub sample_type_name: String,
}

/// Thread-local data structure for high-performance tracking
#[derive(Debug)]
struct ThreadLocalData {
    /// Buffer for events before batch writing
    event_buffer: Vec<Event>,
    /// Call stack frequency tracking
    call_stack_frequencies: HashMap<u64, (u64, usize, String, String)>, // (count, total_size, sample_var, sample_type)
    /// File handle for this thread's data
    file_handle: Option<std::fs::File>,
    /// Thread ID for identification
    thread_id: String,
    /// Sample counter for frequency-based sampling
    sample_counter: u64,
    /// Total operations performed by this thread
    total_operations: u64,
}

impl ThreadLocalData {
    fn new() -> Self {
        Self {
            event_buffer: Vec::with_capacity(1000), // Pre-allocate buffer
            call_stack_frequencies: HashMap::new(),
            file_handle: None,
            thread_id: format!("{:?}", thread::current().id()),
            sample_counter: 0,
            total_operations: 0,
        }
    }

    /// Initialize the file handle for this thread
    fn ensure_file_handle(&mut self) -> std::io::Result<()> {
        if self.file_handle.is_none() {
            let filename = format!("memscope_thread_{}.bin", self.thread_id);
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(filename)?;
            self.file_handle = Some(file);
        }
        Ok(())
    }

    /// Flush the event buffer to disk using binary serialization
    fn flush_events(&mut self) -> std::io::Result<()> {
        if self.event_buffer.is_empty() {
            return Ok(());
        }

        self.ensure_file_handle()?;

        if let Some(ref mut file) = self.file_handle {
            // Serialize events to binary format using bincode
            let serialized =
                serde_json::to_vec(&self.event_buffer)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            // Write length prefix for easier parsing
            let len = serialized.len() as u32;
            file.write_all(&len.to_le_bytes())?;
            file.write_all(&serialized)?;
            file.flush()?;
        }

        self.event_buffer.clear();
        Ok(())
    }

    /// Flush frequency data to disk
    fn flush_frequencies(&mut self) -> std::io::Result<()> {
        if self.call_stack_frequencies.is_empty() {
            return Ok(());
        }

        self.ensure_file_handle()?;

        let freq_data: Vec<FrequencyData> = self
            .call_stack_frequencies
            .iter()
            .map(
                |(&hash, &(freq, total_size, ref var_name, ref type_name))| FrequencyData {
                    call_stack_hash: hash,
                    frequency: freq,
                    total_size,
                    sample_var_name: var_name.clone(),
                    sample_type_name: type_name.clone(),
                },
            )
            .collect();

        if let Some(ref mut file) = self.file_handle {
            let serialized = serde_json::to_vec(&freq_data)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            // Write marker for frequency data
            let marker = 0xFEEDFACEu32;
            file.write_all(&marker.to_le_bytes())?;
            file.write_all(&(serialized.len() as u32).to_le_bytes())?;
            file.write_all(&serialized)?;
            file.flush()?;
        }

        self.call_stack_frequencies.clear();
        Ok(())
    }
}

// Thread-local storage for tracking data
thread_local! {
    static THREAD_DATA: RefCell<ThreadLocalData> = RefCell::new(ThreadLocalData::new());
}

/// High-performance sampling tracker with intelligent sampling strategies
pub struct SamplingTracker {
    /// Configuration for sampling behavior
    config: SamplingConfig,
}

/// Configuration for sampling behavior
#[derive(Debug, Clone)]
pub struct SamplingConfig {
    /// Size threshold for guaranteed sampling (bytes)
    pub large_size_threshold: usize,
    /// Size threshold for medium probability sampling (bytes)
    pub medium_size_threshold: usize,
    /// Sampling rate for medium-sized allocations (0.0-1.0)
    pub medium_sample_rate: f64,
    /// Sampling rate for small allocations (0.0-1.0)
    pub small_sample_rate: f64,
    /// Buffer size before flushing to disk
    pub buffer_size: usize,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            large_size_threshold: 10 * 1024, // 10KB - always sample
            medium_size_threshold: 1024,     // 1KB - 10% sample rate
            medium_sample_rate: 0.1,         // 10%
            small_sample_rate: 0.01,         // 1%
            buffer_size: 1000,               // Flush after 1000 events
        }
    }
}

impl SamplingTracker {
    /// Create a new sampling tracker with default configuration
    pub fn new() -> Self {
        Self {
            config: SamplingConfig::default(),
        }
    }

    /// Create a new sampling tracker with custom configuration
    pub fn with_config(config: SamplingConfig) -> Self {
        Self { config }
    }

    /// Track a variable allocation with intelligent sampling
    pub fn track_variable(
        &self,
        ptr: usize,
        size: usize,
        var_name: String,
        type_name: String,
    ) -> TrackingResult<()> {
        let call_stack_hash = self.calculate_call_stack_hash(&var_name, &type_name);

        THREAD_DATA.with(|data| {
            let mut data = data.borrow_mut();
            data.total_operations += 1;

            // Update frequency tracking (always track frequency, even if we don't sample the event)
            let entry = data
                .call_stack_frequencies
                .entry(call_stack_hash)
                .or_insert((0, 0, var_name.clone(), type_name.clone()));
            entry.0 += 1; // Increment frequency
            entry.1 += size; // Add to total size

            // Intelligent sampling decision
            if self.should_sample(size, &mut data) {
                let event = Event {
                    timestamp: get_timestamp(),
                    ptr,
                    size,
                    call_stack_hash,
                    event_type: EventType::Allocate,
                    var_name: Some(var_name),
                    type_name: Some(type_name),
                };

                data.event_buffer.push(event);

                // Flush if buffer is full
                if data.event_buffer.len() >= self.config.buffer_size {
                    data.flush_events()
                        .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;
                }
            }

            // Periodically flush frequency data
            if data.total_operations % 10000 == 0 {
                data.flush_frequencies()
                    .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;
            }

            Ok(())
        })
    }

    /// Track variable access
    pub fn track_access(&self, ptr: usize) -> TrackingResult<()> {
        self.track_operation(ptr, EventType::Access)
    }

    /// Track variable modification
    pub fn track_modify(&self, ptr: usize) -> TrackingResult<()> {
        self.track_operation(ptr, EventType::Modify)
    }

    /// Track variable drop
    pub fn track_drop(&self, ptr: usize) -> TrackingResult<()> {
        self.track_operation(ptr, EventType::Drop)
    }

    /// Generic operation tracking with lighter sampling
    fn track_operation(&self, ptr: usize, event_type: EventType) -> TrackingResult<()> {
        THREAD_DATA.with(|data| {
            let mut data = data.borrow_mut();
            data.sample_counter += 1;

            // Sample operations less frequently than allocations
            if data.sample_counter % 10 == 0 || matches!(event_type, EventType::Drop) {
                let event = Event {
                    timestamp: get_timestamp(),
                    ptr,
                    size: 0,                     // Operations don't have size
                    call_stack_hash: ptr as u64, // Use ptr as a simple hash for operations
                    event_type,
                    var_name: None,
                    type_name: None,
                };

                data.event_buffer.push(event);

                if data.event_buffer.len() >= self.config.buffer_size {
                    data.flush_events()
                        .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;
                }
            }

            Ok(())
        })
    }

    /// Intelligent sampling decision based on size and frequency
    fn should_sample(&self, size: usize, data: &mut ThreadLocalData) -> bool {
        // Always sample large allocations
        if size >= self.config.large_size_threshold {
            return true;
        }

        // Medium-sized allocations: probabilistic sampling
        if size >= self.config.medium_size_threshold {
            return rand::random::<f64>() < self.config.medium_sample_rate;
        }

        // Small allocations: very low probability, but frequency-aware
        data.sample_counter += 1;
        if data.sample_counter % 100 == 0 {
            // Every 100th small allocation gets sampled
            return true;
        }

        // Otherwise, use configured small sample rate
        rand::random::<f64>() < self.config.small_sample_rate
    }

    /// Calculate a simple call stack hash for frequency tracking
    fn calculate_call_stack_hash(&self, var_name: &str, type_name: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        var_name.hash(&mut hasher);
        type_name.hash(&mut hasher);
        // In a more sophisticated implementation, we'd include actual call stack
        hasher.finish()
    }

    /// Flush all pending data for the current thread
    pub fn flush_current_thread(&self) -> TrackingResult<()> {
        THREAD_DATA.with(|data| {
            let mut data = data.borrow_mut();
            data.flush_events()
                .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;
            data.flush_frequencies()
                .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;
            Ok(())
        })
    }

    /// Get current thread's basic statistics
    pub fn get_current_thread_stats(&self) -> ThreadStats {
        THREAD_DATA.with(|data| {
            let data = data.borrow();
            ThreadStats {
                thread_id: data.thread_id.clone(),
                total_operations: data.total_operations,
                events_buffered: data.event_buffer.len(),
                unique_call_stacks: data.call_stack_frequencies.len(),
            }
        })
    }
}

impl Default for SamplingTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Basic thread statistics
#[derive(Debug, Clone)]
pub struct ThreadStats {
    pub thread_id: String,
    pub total_operations: u64,
    pub events_buffered: usize,
    pub unique_call_stacks: usize,
}

/// Get current timestamp in nanoseconds
fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Global sampling tracker instance
static GLOBAL_SAMPLING_TRACKER: std::sync::OnceLock<SamplingTracker> = std::sync::OnceLock::new();

/// Get the global sampling tracker
pub fn get_sampling_tracker() -> &'static SamplingTracker {
    GLOBAL_SAMPLING_TRACKER.get_or_init(SamplingTracker::new)
}

/// Initialize the sampling tracker with custom configuration
pub fn init_sampling_tracker(config: SamplingConfig) {
    GLOBAL_SAMPLING_TRACKER
        .set(SamplingTracker::with_config(config))
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_basic_sampling_tracker() {
        let tracker = SamplingTracker::new();

        // Test variable tracking
        tracker
            .track_variable(0x1000, 1024, "test_var".to_string(), "Vec<i32>".to_string())
            .unwrap();

        // Test operations
        tracker.track_access(0x1000).unwrap();
        tracker.track_modify(0x1000).unwrap();
        tracker.track_drop(0x1000).unwrap();

        let stats = tracker.get_current_thread_stats();
        assert!(stats.total_operations > 0);

        // Flush data
        tracker.flush_current_thread().unwrap();
    }

    #[test]
    fn test_intelligent_sampling() {
        let config = SamplingConfig {
            large_size_threshold: 100,
            medium_size_threshold: 50,
            medium_sample_rate: 1.0, // 100% for testing
            small_sample_rate: 0.0,  // 0% for testing
            buffer_size: 10,
        };

        let tracker = SamplingTracker::with_config(config);

        // Large allocation should always be sampled
        tracker
            .track_variable(0x1000, 200, "large_var".to_string(), "Vec<u8>".to_string())
            .unwrap();

        // Medium allocation should be sampled (100% rate)
        tracker
            .track_variable(0x2000, 75, "medium_var".to_string(), "String".to_string())
            .unwrap();

        let stats = tracker.get_current_thread_stats();
        assert_eq!(stats.total_operations, 2);
    }

    #[test]
    fn test_multithread_sampling() {
        let tracker = Arc::new(SamplingTracker::new());
        let mut handles = vec![];

        // Test with multiple threads
        for i in 0..5 {
            let tracker_clone = tracker.clone();
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    let ptr = (i * 1000 + j) as usize;
                    tracker_clone
                        .track_variable(
                            ptr,
                            64,
                            format!("thread_{}_var_{}", i, j),
                            "TestType".to_string(),
                        )
                        .unwrap();
                }

                tracker_clone.flush_current_thread().unwrap();
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify files were created
        let files = std::fs::read_dir(".")
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map(|name| name.starts_with("memscope_thread_"))
                    .unwrap_or(false)
            })
            .count();

        assert!(files >= 5); // At least 5 thread files should be created
    }
}
