// Thread-Local Memory Tracking Strategy
// Optimized implementation for multi-threaded applications
// Uses thread-local storage to avoid lock contention

use crate::unified::tracking_dispatcher::{
    MemoryTracker, TrackerConfig, TrackerError, TrackerStatistics, TrackerType,
};
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Thread-local memory tracking strategy
/// Optimized for multi-threaded applications with minimal cross-thread coordination
/// Each thread maintains its own allocation records to avoid lock contention
pub struct ThreadLocalStrategy {
    /// Global configuration shared across threads
    config: Option<TrackerConfig>,
    /// Global tracking state
    global_state: Arc<GlobalTrackingState>,
    /// Thread registry for managing worker threads
    thread_registry: Arc<ThreadRegistry>,
    /// Aggregated performance metrics
    global_metrics: Arc<GlobalMetrics>,
}

/// Global state shared across all tracking threads
/// Maintains coordination data without locks using atomics
#[derive(Debug)]
struct GlobalTrackingState {
    /// Whether tracking is currently active
    is_active: AtomicU64, // 0 = inactive, 1 = active
    /// Total number of active tracking threads
    active_threads: AtomicUsize,
    /// Global session start timestamp
    session_start_ns: AtomicU64,
    /// Next allocation ID (globally unique)
    next_allocation_id: AtomicU64,
}

/// Registry of all threads participating in tracking
/// Manages thread lifecycle and data collection coordination
#[derive(Debug)]
struct ThreadRegistry {
    /// Total threads that have registered
    total_registered_threads: AtomicUsize,
    /// Threads currently active in tracking
    active_tracking_threads: AtomicUsize,
}

/// Global metrics aggregated from all threads
/// Provides system-wide view of tracking performance
#[derive(Debug)]
struct GlobalMetrics {
    /// Total allocations across all threads
    total_allocations: AtomicU64,
    /// Total bytes allocated across all threads
    total_bytes_allocated: AtomicU64,
    /// Peak concurrent allocations
    peak_concurrent_allocations: AtomicU64,
    /// Total tracking overhead in bytes
    total_overhead_bytes: AtomicUsize,
}

/// Thread-local allocation record
/// Stored in thread-local storage for maximum performance
#[derive(Debug, Clone)]
struct ThreadLocalRecord {
    /// Globally unique allocation ID
    global_id: u64,
    /// Thread-local allocation sequence number
    _local_sequence: u64,
    /// Memory pointer address
    ptr: usize,
    /// Allocated size in bytes
    size: usize,
    /// Variable name (if available)
    var_name: Option<String>,
    /// Type information
    type_name: String,
    /// Allocation timestamp (nanoseconds)
    timestamp_alloc: u64,
    /// Deallocation timestamp (if deallocated)
    timestamp_dealloc: Option<u64>,
    /// Thread ID where allocation occurred
    thread_id: u64,
    /// Thread name (if available)
    thread_name: Option<String>,
}

/// Thread-local tracking data
/// Stored in thread_local! storage for each participating thread
#[derive(Debug)]
struct ThreadLocalData {
    /// Thread-specific allocation records
    allocations: Vec<ThreadLocalRecord>,
    /// Thread-local sequence counter
    local_sequence: u64,
    /// Thread-specific performance metrics
    thread_metrics: ThreadMetrics,
    /// Thread registration state
    is_registered: bool,
}

/// Performance metrics for individual thread
#[derive(Debug, Clone)]
struct ThreadMetrics {
    /// Allocations tracked by this thread
    allocations_count: u64,
    /// Bytes allocated by this thread
    bytes_allocated: u64,
    /// Thread tracking start time
    _start_time_ns: u64,
    /// Average allocation tracking time (nanoseconds)
    avg_allocation_time_ns: f64,
    /// Thread overhead in bytes
    thread_overhead_bytes: usize,
}

// Thread-local storage for tracking data
thread_local! {
    /// Thread-local tracking data storage
    /// Each thread maintains its own independent allocation records
    static THREAD_LOCAL_DATA: RefCell<ThreadLocalData> = const{ RefCell::new(ThreadLocalData {
        allocations: Vec::new(),
        local_sequence: 0,
        thread_metrics: ThreadMetrics {
            allocations_count: 0,
            bytes_allocated: 0,
            _start_time_ns: 0,
            avg_allocation_time_ns: 0.0,
            thread_overhead_bytes: 0,
        },
        is_registered: false,
    })};
}

impl Default for GlobalTrackingState {
    /// Initialize global tracking state with inactive values
    fn default() -> Self {
        Self {
            is_active: AtomicU64::new(0),
            active_threads: AtomicUsize::new(0),
            session_start_ns: AtomicU64::new(0),
            next_allocation_id: AtomicU64::new(1),
        }
    }
}

impl Default for ThreadRegistry {
    /// Initialize thread registry with zero counts
    fn default() -> Self {
        Self {
            total_registered_threads: AtomicUsize::new(0),
            active_tracking_threads: AtomicUsize::new(0),
        }
    }
}

impl Default for GlobalMetrics {
    /// Initialize global metrics with zero values
    fn default() -> Self {
        Self {
            total_allocations: AtomicU64::new(0),
            total_bytes_allocated: AtomicU64::new(0),
            peak_concurrent_allocations: AtomicU64::new(0),
            total_overhead_bytes: AtomicUsize::new(0),
        }
    }
}

impl ThreadLocalStrategy {
    /// Create new thread-local strategy instance
    /// Initializes global coordination structures
    pub fn new() -> Self {
        debug!("Creating new thread-local strategy");

        Self {
            config: None,
            global_state: Arc::new(GlobalTrackingState::default()),
            thread_registry: Arc::new(ThreadRegistry::default()),
            global_metrics: Arc::new(GlobalMetrics::default()),
        }
    }

    /// Register current thread for tracking
    /// Must be called from each thread that will participate in tracking
    pub fn register_current_thread(&self) -> Result<(), TrackerError> {
        let thread_id = Self::get_current_thread_id();
        let thread_name = thread::current().name().map(|s| s.to_string());

        debug!(
            "Registering thread for tracking: id={}, name={:?}",
            thread_id, thread_name
        );

        THREAD_LOCAL_DATA.with(|data| {
            let mut thread_data = data.borrow_mut();

            if thread_data.is_registered {
                debug!("Thread already registered: {}", thread_id);
                return Ok(());
            }

            // Initialize thread-local data
            thread_data.is_registered = true;
            thread_data.local_sequence = 0;
            thread_data.allocations.clear();
            thread_data.thread_metrics = ThreadMetrics {
                allocations_count: 0,
                bytes_allocated: 0,
                _start_time_ns: Self::get_timestamp_ns(),
                avg_allocation_time_ns: 0.0,
                thread_overhead_bytes: std::mem::size_of::<ThreadLocalData>(),
            };

            // Update global registry
            self.thread_registry
                .total_registered_threads
                .fetch_add(1, Ordering::Relaxed);

            info!("Thread registered successfully: id={}", thread_id);
            Ok(())
        })
    }

    /// Track memory allocation in current thread
    /// Records allocation with thread-local storage for maximum performance
    pub fn track_allocation(
        &self,
        ptr: usize,
        size: usize,
        var_name: Option<String>,
        type_name: String,
    ) -> Result<(), TrackerError> {
        if self.global_state.is_active.load(Ordering::Relaxed) == 0 {
            return Err(TrackerError::StartFailed {
                reason: "Tracking not active".to_string(),
            });
        }

        let start_time = Instant::now();
        let thread_id = Self::get_current_thread_id();

        THREAD_LOCAL_DATA.with(|data| {
            let mut thread_data = data.borrow_mut();

            if !thread_data.is_registered {
                // Auto-register thread if not already registered
                drop(thread_data);
                self.register_current_thread()?;
                thread_data = data.borrow_mut();
            }

            // Generate globally unique allocation ID
            let global_id = self
                .global_state
                .next_allocation_id
                .fetch_add(1, Ordering::Relaxed);
            thread_data.local_sequence += 1;

            // Create allocation record
            let record = ThreadLocalRecord {
                global_id,
                _local_sequence: thread_data.local_sequence,
                ptr,
                size,
                var_name,
                type_name,
                timestamp_alloc: Self::get_timestamp_ns(),
                timestamp_dealloc: None,
                thread_id,
                thread_name: thread::current().name().map(|s| s.to_string()),
            };

            // Store in thread-local storage
            thread_data.allocations.push(record);

            // Update thread-local metrics
            thread_data.thread_metrics.allocations_count += 1;
            thread_data.thread_metrics.bytes_allocated += size as u64;

            // Update average allocation time
            let allocation_time_ns = start_time.elapsed().as_nanos() as f64;
            let weight = 0.1; // Exponential moving average
            thread_data.thread_metrics.avg_allocation_time_ns = (1.0 - weight)
                * thread_data.thread_metrics.avg_allocation_time_ns
                + weight * allocation_time_ns;

            // Update global metrics atomically
            self.global_metrics
                .total_allocations
                .fetch_add(1, Ordering::Relaxed);
            self.global_metrics
                .total_bytes_allocated
                .fetch_add(size as u64, Ordering::Relaxed);

            debug!(
                "Tracked allocation in thread {}: ptr={:x}, size={}, global_id={}",
                thread_id, ptr, size, global_id
            );

            Ok(())
        })
    }

    /// Track memory deallocation in current thread
    /// Updates existing allocation record with deallocation timestamp
    pub fn track_deallocation(&self, ptr: usize) -> Result<(), TrackerError> {
        if self.global_state.is_active.load(Ordering::Relaxed) == 0 {
            return Err(TrackerError::StartFailed {
                reason: "Tracking not active".to_string(),
            });
        }

        let timestamp = Self::get_timestamp_ns();
        let thread_id = Self::get_current_thread_id();

        THREAD_LOCAL_DATA.with(|data| {
            let mut thread_data = data.borrow_mut();

            // Find allocation record in thread-local storage
            if let Some(record) = thread_data
                .allocations
                .iter_mut()
                .find(|r| r.ptr == ptr && r.timestamp_dealloc.is_none())
            {
                record.timestamp_dealloc = Some(timestamp);
                debug!(
                    "Tracked deallocation in thread {}: ptr={:x}, global_id={}",
                    thread_id, ptr, record.global_id
                );
                Ok(())
            } else {
                // Allocation might be in different thread - this is expected in multi-threaded apps
                warn!(
                    "Deallocation tracked for unknown pointer in thread {}: {:x}",
                    thread_id, ptr
                );
                Ok(()) // Not an error in multi-threaded context
            }
        })
    }

    /// Collect data from all registered threads
    /// Aggregates thread-local data into unified format
    fn collect_all_thread_data(&self) -> Result<Vec<ThreadLocalRecord>, TrackerError> {
        debug!("Collecting data from all registered threads");

        let mut all_records = Vec::new();
        let mut total_overhead = 0;

        // Note: In real implementation, we would need a mechanism to collect
        // data from all threads. This simplified version only collects from
        // the current thread. A complete implementation would require:
        // 1. A registry of all active threads
        // 2. A coordination mechanism to collect data from each thread
        // 3. Proper synchronization for data collection

        THREAD_LOCAL_DATA.with(|data| {
            let thread_data = data.borrow();
            all_records.extend(thread_data.allocations.clone());
            total_overhead += thread_data.thread_metrics.thread_overhead_bytes;
        });

        // Update global overhead metrics
        self.global_metrics
            .total_overhead_bytes
            .store(total_overhead, Ordering::Relaxed);

        // Sort records by global ID for consistent ordering
        all_records.sort_by_key(|r| r.global_id);

        info!(
            "Collected {} records from threads, total overhead: {} bytes",
            all_records.len(),
            total_overhead
        );

        Ok(all_records)
    }

    /// Export collected data as JSON compatible with existing format
    fn export_as_json(&self) -> Result<String, TrackerError> {
        let records = self.collect_all_thread_data()?;

        // Create JSON structure compatible with existing format
        let mut allocations = Vec::new();

        for record in records.iter() {
            let mut allocation = serde_json::Map::new();

            allocation.insert(
                "ptr".to_string(),
                serde_json::Value::String(format!("{:x}", record.ptr)),
            );
            allocation.insert(
                "size".to_string(),
                serde_json::Value::Number(serde_json::Number::from(record.size)),
            );
            allocation.insert(
                "timestamp_alloc".to_string(),
                serde_json::Value::Number(serde_json::Number::from(record.timestamp_alloc)),
            );

            if let Some(var_name) = &record.var_name {
                allocation.insert(
                    "var_name".to_string(),
                    serde_json::Value::String(var_name.clone()),
                );
            }

            allocation.insert(
                "type_name".to_string(),
                serde_json::Value::String(record.type_name.clone()),
            );

            if let Some(timestamp_dealloc) = record.timestamp_dealloc {
                allocation.insert(
                    "timestamp_dealloc".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(timestamp_dealloc)),
                );
            }

            // Add thread-specific metadata
            allocation.insert(
                "thread_id".to_string(),
                serde_json::Value::Number(serde_json::Number::from(record.thread_id)),
            );

            if let Some(thread_name) = &record.thread_name {
                allocation.insert(
                    "thread_name".to_string(),
                    serde_json::Value::String(thread_name.clone()),
                );
            }

            allocation.insert(
                "tracking_strategy".to_string(),
                serde_json::Value::String("thread_local".to_string()),
            );

            allocations.push(serde_json::Value::Object(allocation));
        }

        let mut output = serde_json::Map::new();
        output.insert(
            "allocations".to_string(),
            serde_json::Value::Array(allocations),
        );
        output.insert("strategy_metadata".to_string(), serde_json::json!({
            "strategy_type": "thread_local",
            "total_allocations": self.global_metrics.total_allocations.load(Ordering::Relaxed),
            "total_bytes": self.global_metrics.total_bytes_allocated.load(Ordering::Relaxed),
            "total_threads": self.thread_registry.total_registered_threads.load(Ordering::Relaxed),
            "active_threads": self.thread_registry.active_tracking_threads.load(Ordering::Relaxed),
            "overhead_bytes": self.global_metrics.total_overhead_bytes.load(Ordering::Relaxed)
        }));

        serde_json::to_string_pretty(&output).map_err(|e| TrackerError::DataCollectionFailed {
            reason: format!("JSON serialization failed: {}", e),
        })
    }

    /// Get current thread ID in a platform-independent way
    fn get_current_thread_id() -> u64 {
        // Simple hash of thread ID for cross-platform compatibility
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        thread::current().id().hash(&mut hasher);
        hasher.finish()
    }

    /// Get high-precision timestamp in nanoseconds
    fn get_timestamp_ns() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    }
}

impl MemoryTracker for ThreadLocalStrategy {
    /// Initialize strategy with provided configuration
    /// Sets up global coordination structures
    fn initialize(&mut self, config: TrackerConfig) -> Result<(), TrackerError> {
        debug!(
            "Initializing thread-local strategy with config: {:?}",
            config
        );

        // Validate configuration
        if config.sample_rate < 0.0 || config.sample_rate > 1.0 {
            return Err(TrackerError::InvalidConfiguration {
                reason: "Sample rate must be between 0.0 and 1.0".to_string(),
            });
        }

        if config.max_overhead_mb == 0 {
            return Err(TrackerError::InvalidConfiguration {
                reason: "Maximum overhead must be greater than 0".to_string(),
            });
        }

        // Store configuration
        self.config = Some(config);

        // Reset global state
        self.global_state.is_active.store(0, Ordering::Relaxed);
        self.global_state.active_threads.store(0, Ordering::Relaxed);
        self.global_state
            .next_allocation_id
            .store(1, Ordering::Relaxed);

        // Reset global metrics
        self.global_metrics
            .total_allocations
            .store(0, Ordering::Relaxed);
        self.global_metrics
            .total_bytes_allocated
            .store(0, Ordering::Relaxed);
        self.global_metrics
            .peak_concurrent_allocations
            .store(0, Ordering::Relaxed);
        self.global_metrics
            .total_overhead_bytes
            .store(0, Ordering::Relaxed);

        // Reset thread registry
        self.thread_registry
            .total_registered_threads
            .store(0, Ordering::Relaxed);
        self.thread_registry
            .active_tracking_threads
            .store(0, Ordering::Relaxed);

        info!("Thread-local strategy initialized successfully");
        Ok(())
    }

    /// Start active memory tracking across all threads
    fn start_tracking(&mut self) -> Result<(), TrackerError> {
        debug!("Starting thread-local tracking");

        // Set global active state
        let was_active = self.global_state.is_active.swap(1, Ordering::Relaxed);
        if was_active == 1 {
            warn!("Thread-local tracking was already active");
            return Ok(()); // Already active, not an error
        }

        // Record session start time
        self.global_state
            .session_start_ns
            .store(Self::get_timestamp_ns(), Ordering::Relaxed);

        // Register current thread automatically
        self.register_current_thread()?;

        info!("Thread-local tracking started successfully");
        Ok(())
    }

    /// Stop tracking and collect data from all threads
    fn stop_tracking(&mut self) -> Result<Vec<u8>, TrackerError> {
        debug!("Stopping thread-local tracking");

        // Mark as inactive
        let was_active = self.global_state.is_active.swap(0, Ordering::Relaxed);
        if was_active == 0 {
            warn!("Thread-local tracking was not active");
        }

        // Export aggregated data
        let json_data = self.export_as_json()?;

        let total_allocations = self
            .global_metrics
            .total_allocations
            .load(Ordering::Relaxed);
        let total_threads = self
            .thread_registry
            .total_registered_threads
            .load(Ordering::Relaxed);

        info!(
            "Thread-local tracking stopped: {} allocations from {} threads",
            total_allocations, total_threads
        );

        Ok(json_data.into_bytes())
    }

    /// Get current tracking statistics aggregated across all threads
    fn get_statistics(&self) -> TrackerStatistics {
        TrackerStatistics {
            allocations_tracked: self
                .global_metrics
                .total_allocations
                .load(Ordering::Relaxed),
            memory_tracked_bytes: self
                .global_metrics
                .total_bytes_allocated
                .load(Ordering::Relaxed),
            overhead_bytes: self
                .global_metrics
                .total_overhead_bytes
                .load(Ordering::Relaxed) as u64,
            tracking_duration_ms: {
                let start_ns = self.global_state.session_start_ns.load(Ordering::Relaxed);
                if start_ns > 0 {
                    (Self::get_timestamp_ns() - start_ns) / 1_000_000
                } else {
                    0
                }
            },
        }
    }

    /// Check if strategy is currently active
    fn is_active(&self) -> bool {
        self.global_state.is_active.load(Ordering::Relaxed) == 1
    }

    /// Get strategy type identifier
    fn tracker_type(&self) -> TrackerType {
        TrackerType::MultiThread
    }
}

impl Default for ThreadLocalStrategy {
    /// Create strategy with default configuration
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Barrier;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_strategy_creation() {
        let strategy = ThreadLocalStrategy::new();
        assert!(!strategy.is_active());
        assert_eq!(strategy.tracker_type(), TrackerType::MultiThread);
    }

    #[test]
    fn test_strategy_initialization() {
        let mut strategy = ThreadLocalStrategy::new();
        let config = TrackerConfig::default();

        let result = strategy.initialize(config);
        assert!(result.is_ok());
        assert!(!strategy.is_active()); // Not active until start_tracking called
    }

    #[test]
    fn test_thread_registration() {
        let strategy = ThreadLocalStrategy::new();

        let result = strategy.register_current_thread();
        assert!(result.is_ok());

        // Verify thread was registered
        let total_threads = strategy
            .thread_registry
            .total_registered_threads
            .load(Ordering::Relaxed);
        assert_eq!(total_threads, 1);
    }

    #[test]
    fn test_single_thread_tracking() {
        let mut strategy = ThreadLocalStrategy::new();
        strategy
            .initialize(TrackerConfig::default())
            .expect("Strategy initialization should succeed");
        strategy
            .start_tracking()
            .expect("Strategy should start tracking successfully");

        // Track allocation
        let result = strategy.track_allocation(
            0x1000,
            128,
            Some("test_var".to_string()),
            "TestType".to_string(),
        );
        assert!(result.is_ok());

        // Check statistics
        let stats = strategy.get_statistics();
        assert_eq!(stats.allocations_tracked, 1);
        assert_eq!(stats.memory_tracked_bytes, 128);

        // Track deallocation
        assert!(strategy.track_deallocation(0x1000).is_ok());

        // Stop tracking
        let data = strategy.stop_tracking();
        assert!(data.is_ok());
    }

    #[test]
    fn test_multi_thread_tracking() {
        let mut strategy = ThreadLocalStrategy::new();
        strategy
            .initialize(TrackerConfig::default())
            .expect("Multi-thread strategy initialization should succeed");
        strategy
            .start_tracking()
            .expect("Multi-thread strategy should start tracking successfully");

        let strategy = Arc::new(strategy);
        let barrier = Arc::new(Barrier::new(3)); // Main thread + 2 worker threads
        let mut handles = vec![];

        // Spawn worker threads
        for thread_id in 0..2 {
            let strategy_clone = Arc::clone(&strategy);
            let barrier_clone = Arc::clone(&barrier);

            let handle = thread::spawn(move || {
                // Register thread
                strategy_clone
                    .register_current_thread()
                    .expect("Thread registration should succeed");

                // Wait for all threads to be ready
                barrier_clone.wait();

                // Track allocations in each thread
                for i in 0..10 {
                    let ptr = 0x1000 + (thread_id * 1000) + (i * 0x10);
                    let size = 64 + i * 8;

                    let result = strategy_clone.track_allocation(
                        ptr,
                        size,
                        Some(format!("var_{}_{}", thread_id, i)),
                        "TestType".to_string(),
                    );
                    assert!(result.is_ok());

                    // Small delay to simulate real allocation patterns
                    thread::sleep(Duration::from_micros(100));
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to be ready
        barrier.wait();

        // Wait for all threads to complete
        for handle in handles {
            handle
                .join()
                .expect("Worker thread should complete successfully");
        }

        // Verify statistics
        let stats = strategy.get_statistics();
        assert_eq!(stats.allocations_tracked, 20); // 2 threads * 10 allocations each

        // Verify thread registration
        let total_threads = strategy
            .thread_registry
            .total_registered_threads
            .load(Ordering::Relaxed);
        assert_eq!(total_threads, 3); // Main thread + 2 worker threads
    }

    #[test]
    fn test_json_export_format() {
        let mut strategy = ThreadLocalStrategy::new();
        strategy
            .initialize(TrackerConfig::default())
            .expect("Strategy initialization should succeed for cleanup test");
        strategy
            .start_tracking()
            .expect("Strategy should start tracking successfully for cleanup test");

        // Track one allocation
        strategy
            .track_allocation(
                0x1000,
                256,
                Some("test_variable".to_string()),
                "TestStruct".to_string(),
            )
            .expect("Allocation tracking should succeed");

        let data = strategy
            .stop_tracking()
            .expect("Strategy should stop tracking and return data successfully");
        let json_str = String::from_utf8(data).expect("Strategy data should be valid UTF-8");

        // Verify JSON structure
        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("Generated JSON should be valid");
        assert!(parsed["allocations"].is_array());
        assert!(parsed["strategy_metadata"].is_object());

        let allocations = parsed["allocations"].as_array().unwrap();
        assert_eq!(allocations.len(), 1);

        let first_alloc = &allocations[0];
        assert_eq!(first_alloc["ptr"].as_str().unwrap(), "1000");
        assert_eq!(first_alloc["size"].as_u64().unwrap(), 256);
        assert_eq!(first_alloc["var_name"].as_str().unwrap(), "test_variable");
        assert_eq!(first_alloc["type_name"].as_str().unwrap(), "TestStruct");
        assert_eq!(
            first_alloc["tracking_strategy"].as_str().unwrap(),
            "thread_local"
        );
        assert!(first_alloc["thread_id"].is_number());
    }

    #[test]
    fn test_global_metrics_atomicity() {
        let mut strategy = ThreadLocalStrategy::new();
        strategy.initialize(TrackerConfig::default()).unwrap();
        strategy.start_tracking().unwrap();

        // Track multiple allocations to test atomic updates
        for i in 0..100 {
            strategy
                .track_allocation(0x1000 + i * 0x10, 64, None, "TestType".to_string())
                .unwrap();
        }

        let stats = strategy.get_statistics();
        assert_eq!(stats.allocations_tracked, 100);
        assert_eq!(stats.memory_tracked_bytes, 6400);

        // Verify global metrics consistency
        let global_allocs = strategy
            .global_metrics
            .total_allocations
            .load(Ordering::Relaxed);
        let global_bytes = strategy
            .global_metrics
            .total_bytes_allocated
            .load(Ordering::Relaxed);

        assert_eq!(global_allocs, 100);
        assert_eq!(global_bytes, 6400);
    }
}
