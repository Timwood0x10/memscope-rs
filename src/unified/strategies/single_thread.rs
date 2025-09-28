// Single-Thread Memory Tracking Strategy
// Optimized implementation for single-threaded applications
// Provides zero-overhead tracking with direct global storage

use crate::core::tracker::{get_tracker, MemoryTracker as CoreMemoryTracker};
use crate::lockfree::aggregator::LockfreeAggregator;
use crate::unified::tracking_dispatcher::{
    MemoryTracker, TrackerConfig, TrackerError, TrackerStatistics, TrackerType,
};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Single-threaded memory tracking strategy
/// Optimized for applications with single execution thread
/// Uses direct global storage for maximum performance
pub struct SingleThreadStrategy {
    /// Configuration for this strategy instance
    config: Option<TrackerConfig>,
    /// Current tracking state
    tracking_state: TrackingState,
    /// Memory allocation records
    allocation_records: Mutex<Vec<AllocationRecord>>,
    /// Performance metrics
    metrics: PerformanceMetrics,
    /// Integration with existing aggregator
    #[allow(dead_code)]
    aggregator: Option<LockfreeAggregator>,
    /// Integration with core global tracker
    core_tracker: Option<std::sync::Arc<CoreMemoryTracker>>,
}

/// Current state of tracking session
#[derive(Debug, Clone, PartialEq)]
enum TrackingState {
    /// Strategy not yet initialized
    Uninitialized,
    /// Strategy initialized but not tracking
    Initialized,
    /// Currently tracking allocations
    Active,
    /// Tracking stopped, data available for collection
    Stopped,
}

/// Individual memory allocation record
/// Stores comprehensive tracking information for single allocation
#[derive(Debug, Clone)]
struct AllocationRecord {
    /// Unique allocation identifier
    id: u64,
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
    /// Stack trace at allocation
    _stack_trace: Vec<String>,
    /// Additional metadata
    _metadata: HashMap<String, String>,
}

/// Performance metrics for single-thread strategy
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    /// Total allocations tracked
    total_allocations: u64,
    /// Total bytes allocated
    total_bytes_allocated: u64,
    /// Peak concurrent allocations
    peak_allocations: u64,
    /// Strategy initialization time (microseconds)
    init_time_us: u64,
    /// Total tracking duration (microseconds)
    tracking_duration_us: u64,
    /// Memory overhead of tracking (bytes)
    overhead_bytes: usize,
    /// Average allocation tracking time (nanoseconds)
    avg_allocation_time_ns: f64,
}

impl Default for PerformanceMetrics {
    /// Initialize performance metrics with zero values
    fn default() -> Self {
        Self {
            total_allocations: 0,
            total_bytes_allocated: 0,
            peak_allocations: 0,
            init_time_us: 0,
            tracking_duration_us: 0,
            overhead_bytes: 0,
            avg_allocation_time_ns: 0.0,
        }
    }
}

impl SingleThreadStrategy {
    /// Create new single-thread strategy instance
    /// Returns uninitialized strategy ready for configuration
    pub fn new() -> Self {
        debug!("Creating new single-thread strategy");

        Self {
            config: None,
            tracking_state: TrackingState::Uninitialized,
            allocation_records: Mutex::new(Vec::new()),
            metrics: PerformanceMetrics::default(),
            aggregator: None,
            core_tracker: None,
        }
    }

    /// Create strategy with integration to existing aggregator
    /// Enables compatibility with existing lockfree infrastructure
    pub fn with_aggregator(output_dir: std::path::PathBuf) -> Self {
        debug!("Creating single-thread strategy with aggregator integration");

        let aggregator = LockfreeAggregator::new(output_dir);

        Self {
            config: None,
            tracking_state: TrackingState::Uninitialized,
            allocation_records: Mutex::new(Vec::new()),
            metrics: PerformanceMetrics::default(),
            aggregator: Some(aggregator),
            core_tracker: None,
        }
    }

    /// Create strategy with full core tracker integration
    /// Enables seamless integration with existing global tracking system
    pub fn with_core_integration() -> Self {
        debug!("Creating single-thread strategy with core tracker integration");

        let core_tracker = get_tracker();

        Self {
            config: None,
            tracking_state: TrackingState::Uninitialized,
            allocation_records: Mutex::new(Vec::new()),
            metrics: PerformanceMetrics::default(),
            aggregator: None,
            core_tracker: Some(core_tracker),
        }
    }

    /// Create strategy with both aggregator and core integration
    /// Provides maximum compatibility with existing infrastructure
    pub fn with_full_integration(output_dir: std::path::PathBuf) -> Self {
        debug!("Creating single-thread strategy with full integration");

        let aggregator = LockfreeAggregator::new(output_dir);
        let core_tracker = get_tracker();

        Self {
            config: None,
            tracking_state: TrackingState::Uninitialized,
            allocation_records: Mutex::new(Vec::new()),
            metrics: PerformanceMetrics::default(),
            aggregator: Some(aggregator),
            core_tracker: Some(core_tracker),
        }
    }

    /// Track new memory allocation
    /// Records allocation details with minimal overhead
    pub fn track_allocation(
        &mut self,
        ptr: usize,
        size: usize,
        var_name: Option<String>,
        type_name: String,
    ) -> Result<(), TrackerError> {
        if self.tracking_state != TrackingState::Active {
            return Err(TrackerError::StartFailed {
                reason: "Strategy not in active tracking state".to_string(),
            });
        }

        let start_time = Instant::now();

        // Generate unique allocation ID
        let id = self.metrics.total_allocations + 1;

        // Capture stack trace (simplified for performance)
        let stack_trace = self.capture_stack_trace();

        // Create allocation record
        let record = AllocationRecord {
            id,
            ptr,
            size,
            var_name,
            type_name,
            timestamp_alloc: self.get_timestamp_ns(),
            timestamp_dealloc: None,
            _stack_trace: stack_trace,
            _metadata: HashMap::new(),
        };

        // Store record
        self.allocation_records.lock().unwrap().push(record);

        // Integrate with core tracker if available
        if let Some(core_tracker) = &self.core_tracker {
            // Track in core system using basic allocation tracking
            if let Err(e) = core_tracker.track_allocation(ptr, size) {
                warn!("Core tracker integration failed: {:?}", e);
                // Continue with unified tracking even if core integration fails
            }
        }

        // Update metrics
        self.metrics.total_allocations += 1;
        self.metrics.total_bytes_allocated += size as u64;

        // Update peak allocations
        let current_allocations = self.allocation_records.lock().unwrap().len() as u64;
        if current_allocations > self.metrics.peak_allocations {
            self.metrics.peak_allocations = current_allocations;
        }

        // Update average allocation time
        let allocation_time_ns = start_time.elapsed().as_nanos() as f64;
        let weight = 0.1; // Exponential moving average
        self.metrics.avg_allocation_time_ns =
            (1.0 - weight) * self.metrics.avg_allocation_time_ns + weight * allocation_time_ns;

        debug!(
            "Tracked allocation: ptr={:x}, size={}, id={}",
            ptr, size, id
        );
        Ok(())
    }

    /// Track memory deallocation
    /// Updates existing allocation record with deallocation timestamp
    pub fn track_deallocation(&mut self, ptr: usize) -> Result<(), TrackerError> {
        if self.tracking_state != TrackingState::Active {
            return Err(TrackerError::StartFailed {
                reason: "Strategy not in active tracking state".to_string(),
            });
        }

        let timestamp = self.get_timestamp_ns();

        // Find and update allocation record
        let mut records = self.allocation_records.lock().unwrap();
        if let Some(record) = records
            .iter_mut()
            .find(|r| r.ptr == ptr && r.timestamp_dealloc.is_none())
        {
            record.timestamp_dealloc = Some(timestamp);
            debug!("Tracked deallocation: ptr={ptr:x}, id={}", record.id);
            Ok(())
        } else {
            warn!("Deallocation tracked for unknown pointer: {ptr:x}",);
            Err(TrackerError::DataCollectionFailed {
                reason: format!("Unknown pointer for deallocation: {ptr:x}"),
            })
        }
    }

    /// Capture simplified stack trace for allocation context
    /// Optimized for single-threaded performance
    fn capture_stack_trace(&self) -> Vec<String> {
        // Simplified stack trace for performance
        // In production, this would integrate with backtrace crate
        vec![
            "single_thread_strategy::track_allocation".to_string(),
            "application_code::allocate_memory".to_string(),
        ]
    }

    /// Get high-precision timestamp in nanoseconds
    fn get_timestamp_ns(&self) -> u64 {
        // Use system time for compatibility with existing codebase
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    }

    /// Calculate current memory overhead of tracking system
    fn calculate_overhead(&self) -> usize {
        let records = self.allocation_records.lock().unwrap();
        let record_overhead = records.len() * std::mem::size_of::<AllocationRecord>();
        let base_overhead = std::mem::size_of::<Self>();

        record_overhead + base_overhead
    }

    /// Convert allocation records to JSON format compatible with existing exports
    fn export_as_json(&self) -> Result<String, TrackerError> {
        let records = self.allocation_records.lock().unwrap();

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

            // Add tracking strategy metadata
            allocation.insert(
                "tracking_strategy".to_string(),
                serde_json::Value::String("single_thread".to_string()),
            );

            allocations.push(serde_json::Value::Object(allocation));
        }

        let mut output = serde_json::Map::new();
        output.insert(
            "allocations".to_string(),
            serde_json::Value::Array(allocations),
        );
        output.insert(
            "strategy_metadata".to_string(),
            serde_json::json!({
                "strategy_type": "single_thread",
                "total_allocations": self.metrics.total_allocations,
                "total_bytes": self.metrics.total_bytes_allocated,
                "tracking_duration_us": self.metrics.tracking_duration_us,
                "overhead_bytes": self.calculate_overhead()
            }),
        );

        serde_json::to_string_pretty(&output).map_err(|e| TrackerError::DataCollectionFailed {
            reason: format!("JSON serialization failed: {e}"),
        })
    }
}

impl MemoryTracker for SingleThreadStrategy {
    /// Initialize strategy with provided configuration
    /// Sets up tracking infrastructure and validates configuration
    fn initialize(&mut self, config: TrackerConfig) -> Result<(), TrackerError> {
        let start_time = Instant::now();

        debug!(
            "Initializing single-thread strategy with config: {:?}",
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

        // Initialize allocation storage with reasonable capacity
        let initial_capacity = 1000; // Expect ~1000 allocations initially
        self.allocation_records = Mutex::new(Vec::with_capacity(initial_capacity));

        // Update state and metrics
        self.tracking_state = TrackingState::Initialized;
        self.metrics.init_time_us = start_time.elapsed().as_micros() as u64;

        info!(
            "Single-thread strategy initialized in {}μs",
            self.metrics.init_time_us
        );
        Ok(())
    }

    /// Start active memory tracking
    /// Transitions strategy to active state and begins collecting data
    fn start_tracking(&mut self) -> Result<(), TrackerError> {
        match &self.tracking_state {
            TrackingState::Initialized => {
                debug!("Starting single-thread tracking");

                self.tracking_state = TrackingState::Active;

                // Reset metrics for new session
                self.metrics.total_allocations = 0;
                self.metrics.total_bytes_allocated = 0;
                self.metrics.peak_allocations = 0;

                // Clear previous allocation records
                self.allocation_records.lock().unwrap().clear();

                info!("Single-thread tracking started successfully");
                Ok(())
            }
            TrackingState::Active => {
                warn!("Tracking already active");
                Ok(()) // Already active, not an error
            }
            other_state => Err(TrackerError::StartFailed {
                reason: format!("Cannot start tracking from state: {other_state:?}"),
            }),
        }
    }

    /// Stop tracking and collect all data
    /// Returns serialized tracking data in compatible format
    fn stop_tracking(&mut self) -> Result<Vec<u8>, TrackerError> {
        match &self.tracking_state {
            TrackingState::Active => {
                debug!("Stopping single-thread tracking");

                self.tracking_state = TrackingState::Stopped;

                // Update final metrics
                self.metrics.overhead_bytes = self.calculate_overhead();

                // Export data in JSON format
                let json_data = self.export_as_json()?;

                info!(
                    "Single-thread tracking stopped, collected {} allocations",
                    self.metrics.total_allocations
                );

                Ok(json_data.into_bytes())
            }
            TrackingState::Stopped => {
                // Already stopped, return cached data
                let json_data = self.export_as_json()?;
                Ok(json_data.into_bytes())
            }
            other_state => Err(TrackerError::DataCollectionFailed {
                reason: format!("Cannot stop tracking from state: {other_state:?}"),
            }),
        }
    }

    /// Get current tracking statistics
    /// Provides real-time performance and usage metrics
    fn get_statistics(&self) -> TrackerStatistics {
        TrackerStatistics {
            allocations_tracked: self.metrics.total_allocations,
            memory_tracked_bytes: self.metrics.total_bytes_allocated,
            overhead_bytes: self.calculate_overhead() as u64,
            tracking_duration_ms: self.metrics.tracking_duration_us / 1000,
        }
    }

    /// Check if strategy is currently active
    fn is_active(&self) -> bool {
        self.tracking_state == TrackingState::Active
    }

    /// Get strategy type identifier
    fn tracker_type(&self) -> TrackerType {
        TrackerType::SingleThread
    }
}

impl Default for SingleThreadStrategy {
    /// Create strategy with default configuration
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_strategy_creation() {
        let strategy = SingleThreadStrategy::new();
        assert_eq!(strategy.tracking_state, TrackingState::Uninitialized);
        assert!(!strategy.is_active());
        assert_eq!(strategy.tracker_type(), TrackerType::SingleThread);
    }

    #[test]
    fn test_strategy_initialization() {
        let mut strategy = SingleThreadStrategy::new();
        let config = TrackerConfig {
            sample_rate: 1.0,
            max_overhead_mb: 32,
            thread_affinity: None,
            custom_params: HashMap::new(),
        };

        let result = strategy.initialize(config);
        assert!(result.is_ok());
        assert_eq!(strategy.tracking_state, TrackingState::Initialized);
    }

    #[test]
    fn test_invalid_configuration() {
        let mut strategy = SingleThreadStrategy::new();
        let config = TrackerConfig {
            sample_rate: 1.5, // Invalid: > 1.0
            max_overhead_mb: 32,
            thread_affinity: None,
            custom_params: HashMap::new(),
        };

        let result = strategy.initialize(config);
        assert!(result.is_err());
        match result.unwrap_err() {
            TrackerError::InvalidConfiguration { reason } => {
                assert!(reason.contains("Sample rate"));
            }
            _ => panic!("Expected InvalidConfiguration error"),
        }
    }

    #[test]
    fn test_tracking_lifecycle() {
        let mut strategy = SingleThreadStrategy::new();
        let config = TrackerConfig::default();

        // Initialize
        assert!(strategy.initialize(config).is_ok());

        // Start tracking
        assert!(strategy.start_tracking().is_ok());
        assert!(strategy.is_active());

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
        assert!(!strategy.is_active());
    }

    #[test]
    fn test_allocation_tracking() {
        let mut strategy = SingleThreadStrategy::new();
        strategy.initialize(TrackerConfig::default()).unwrap();
        strategy.start_tracking().unwrap();

        // Track multiple allocations
        for i in 0..10 {
            let ptr = 0x1000 + i * 0x100;
            let size = 64 + i * 8;

            let result = strategy.track_allocation(
                ptr,
                size,
                Some(format!("var_{i}")),
                "TestType".to_string(),
            );
            assert!(result.is_ok());
        }

        let stats = strategy.get_statistics();
        assert_eq!(stats.allocations_tracked, 10);
        assert_eq!(
            stats.memory_tracked_bytes,
            64 * 10 + 8 * (0 + 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9)
        );
    }

    #[test]
    fn test_json_export_format() {
        let mut strategy = SingleThreadStrategy::new();
        strategy.initialize(TrackerConfig::default()).unwrap();
        strategy.start_tracking().unwrap();

        // Track one allocation
        strategy
            .track_allocation(
                0x1000,
                256,
                Some("test_variable".to_string()),
                "TestStruct".to_string(),
            )
            .unwrap();

        let data = strategy.stop_tracking().unwrap();
        let json_str = String::from_utf8(data).unwrap();

        // Verify JSON structure
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
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
            "single_thread"
        );
    }

    #[test]
    fn test_performance_metrics() {
        let mut strategy = SingleThreadStrategy::new();
        strategy.initialize(TrackerConfig::default()).unwrap();
        strategy.start_tracking().unwrap();

        // Track allocations to generate metrics
        for i in 0..100 {
            strategy
                .track_allocation(0x1000 + i * 0x10, 64, None, "TestType".to_string())
                .unwrap();
        }

        let stats = strategy.get_statistics();
        assert_eq!(stats.allocations_tracked, 100);
        assert_eq!(stats.memory_tracked_bytes, 6400);
        assert!(stats.overhead_bytes > 0);

        // Check that average allocation time is reasonable
        assert!(strategy.metrics.avg_allocation_time_ns > 0.0);
        assert!(strategy.metrics.avg_allocation_time_ns < 1_000_000.0); // Less than 1ms
    }
}
