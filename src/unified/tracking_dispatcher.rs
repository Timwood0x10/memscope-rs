// Tracking Strategy Dispatcher
// Intelligently routes memory tracking requests to optimal tracking implementations
// Manages lifecycle of different tracking strategies and data aggregation

use crate::lockfree::aggregator::LockfreeAggregator;
use crate::unified::backend::{BackendError, RuntimeEnvironment, TrackingStrategy};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info};

/// Central dispatcher that routes tracking operations to appropriate implementations
/// Maintains active tracking strategies and coordinates data collection
pub struct TrackingDispatcher {
    /// Currently active tracking strategy
    active_strategy: Option<TrackingStrategy>,
    /// Strategy-specific tracker implementations
    tracker_registry: TrackerRegistry,
    /// Shared data aggregator for all tracking sources
    #[allow(dead_code)]
    aggregator: Arc<LockfreeAggregator>,
    /// Dispatcher configuration
    #[allow(dead_code)]
    config: DispatcherConfig,
    /// Performance metrics
    metrics: DispatcherMetrics,
}

/// Registry of available tracker implementations
/// Maintains strategy-specific trackers and their lifecycle
struct TrackerRegistry {
    /// Single-threaded tracker instance
    single_thread_tracker: Option<Box<dyn MemoryTracker>>,
    /// Multi-threaded tracker instance
    multi_thread_tracker: Option<Box<dyn MemoryTracker>>,
    /// Async tracker instance
    async_tracker: Option<Box<dyn MemoryTracker>>,
    /// Hybrid tracker instance
    hybrid_tracker: Option<Box<dyn MemoryTracker>>,
}

/// Configuration for dispatcher behavior
#[derive(Debug, Clone)]
pub struct DispatcherConfig {
    /// Enable automatic strategy switching
    pub auto_switch_strategies: bool,
    /// Maximum number of concurrent trackers
    pub max_concurrent_trackers: usize,
    /// Performance monitoring interval
    pub metrics_interval_ms: u64,
    /// Memory usage threshold for strategy switching
    pub memory_threshold_mb: usize,
}

/// Performance metrics for dispatcher operations
#[derive(Debug, Clone)]
pub struct DispatcherMetrics {
    /// Total tracking operations dispatched
    pub total_dispatches: u64,
    /// Strategy switch count
    pub strategy_switches: u64,
    /// Average dispatch latency in microseconds
    pub avg_dispatch_latency_us: f64,
    /// Current memory overhead percentage
    pub memory_overhead_percent: f64,
    /// Active tracker count
    pub active_trackers: usize,
}

/// Unified memory tracker interface
/// All tracking implementations must implement this trait for dispatcher compatibility
pub trait MemoryTracker: Send + Sync {
    /// Initialize tracker with given configuration
    fn initialize(&mut self, config: TrackerConfig) -> Result<(), TrackerError>;

    /// Start tracking memory operations
    fn start_tracking(&mut self) -> Result<(), TrackerError>;

    /// Stop tracking and collect data
    fn stop_tracking(&mut self) -> Result<Vec<u8>, TrackerError>;

    /// Get current tracking statistics
    fn get_statistics(&self) -> TrackerStatistics;

    /// Check if tracker is currently active
    fn is_active(&self) -> bool;

    /// Get tracker type identifier
    fn tracker_type(&self) -> TrackerType;
}

/// Configuration for individual tracker instances
#[derive(Debug, Clone)]
pub struct TrackerConfig {
    /// Sampling rate (0.0 to 1.0)
    pub sample_rate: f64,
    /// Maximum memory overhead allowed
    pub max_overhead_mb: usize,
    /// Thread affinity settings
    pub thread_affinity: Option<Vec<usize>>,
    /// Custom tracking parameters
    pub custom_params: HashMap<String, String>,
}

/// Statistics from individual tracker
#[derive(Debug, Clone)]
pub struct TrackerStatistics {
    /// Allocations tracked by this tracker
    pub allocations_tracked: u64,
    /// Memory currently tracked (bytes)
    pub memory_tracked_bytes: u64,
    /// Tracker overhead (bytes)
    pub overhead_bytes: u64,
    /// Tracking duration (milliseconds)
    pub tracking_duration_ms: u64,
}

/// Tracker type identifiers
#[derive(Debug, Clone, PartialEq)]
pub enum TrackerType {
    /// Single-threaded global tracker
    SingleThread,
    /// Multi-threaded with thread-local storage
    MultiThread,
    /// Async-aware task-local tracker
    AsyncTracker,
    /// Hybrid multi-thread + async tracker
    HybridTracker,
}

/// Errors specific to tracker operations
#[derive(Error, Debug)]
pub enum TrackerError {
    /// Tracker initialization failed
    #[error("Tracker initialization failed: {reason}")]
    InitializationFailed { reason: String },

    /// Tracking start failed
    #[error("Failed to start tracking: {reason}")]
    StartFailed { reason: String },

    /// Data collection failed
    #[error("Failed to collect tracking data: {reason}")]
    DataCollectionFailed { reason: String },

    /// Tracker configuration invalid
    #[error("Invalid tracker configuration: {reason}")]
    InvalidConfiguration { reason: String },

    /// JSON serialization/deserialization error
    #[error("JSON processing error: {source}")]
    JsonError {
        #[from]
        source: serde_json::Error,
    },
}

impl Default for DispatcherConfig {
    /// Default dispatcher configuration optimized for most use cases
    fn default() -> Self {
        Self {
            auto_switch_strategies: true,
            max_concurrent_trackers: 4,
            metrics_interval_ms: 1000,
            memory_threshold_mb: 100,
        }
    }
}

impl Default for DispatcherMetrics {
    /// Initialize metrics with zero values
    fn default() -> Self {
        Self {
            total_dispatches: 0,
            strategy_switches: 0,
            avg_dispatch_latency_us: 0.0,
            memory_overhead_percent: 0.0,
            active_trackers: 0,
        }
    }
}

impl Default for TrackerConfig {
    /// Default tracker configuration
    fn default() -> Self {
        Self {
            sample_rate: 1.0,
            max_overhead_mb: 50,
            thread_affinity: None,
            custom_params: HashMap::new(),
        }
    }
}

impl TrackerRegistry {
    /// Create new empty tracker registry
    fn new() -> Self {
        Self {
            single_thread_tracker: None,
            multi_thread_tracker: None,
            async_tracker: None,
            hybrid_tracker: None,
        }
    }

    /// Get tracker for specified type, creating if necessary
    fn get_or_create_tracker(
        &mut self,
        tracker_type: TrackerType,
    ) -> Result<&mut Box<dyn MemoryTracker>, BackendError> {
        let tracker = match tracker_type {
            TrackerType::SingleThread => &mut self.single_thread_tracker,
            TrackerType::MultiThread => &mut self.multi_thread_tracker,
            TrackerType::AsyncTracker => &mut self.async_tracker,
            TrackerType::HybridTracker => &mut self.hybrid_tracker,
        };

        if tracker.is_none() {
            *tracker = Some(Self::create_tracker(&tracker_type)?);
        }

        tracker
            .as_mut()
            .ok_or_else(|| BackendError::TrackingInitializationFailed {
                reason: format!("Failed to create {:?} tracker", tracker_type),
            })
    }

    /// Create new tracker instance of specified type
    fn create_tracker(tracker_type: &TrackerType) -> Result<Box<dyn MemoryTracker>, BackendError> {
        match tracker_type {
            TrackerType::SingleThread => Ok(Box::new(SingleThreadTracker::new())),
            TrackerType::MultiThread => Ok(Box::new(MultiThreadTracker::new())),
            TrackerType::AsyncTracker => Ok(Box::new(AsyncTrackerWrapper::new())),
            TrackerType::HybridTracker => Ok(Box::new(HybridTrackerImpl::new())),
        }
    }

    /// Count currently active trackers
    fn count_active_trackers(&self) -> usize {
        let mut count = 0;
        if let Some(ref tracker) = self.single_thread_tracker {
            if tracker.is_active() {
                count += 1;
            }
        }
        if let Some(ref tracker) = self.multi_thread_tracker {
            if tracker.is_active() {
                count += 1;
            }
        }
        if let Some(ref tracker) = self.async_tracker {
            if tracker.is_active() {
                count += 1;
            }
        }
        if let Some(ref tracker) = self.hybrid_tracker {
            if tracker.is_active() {
                count += 1;
            }
        }
        count
    }
}

impl TrackingDispatcher {
    /// Create new tracking dispatcher with configuration
    pub fn new(config: DispatcherConfig) -> Self {
        info!("Creating tracking dispatcher with config: {:?}", config);

        Self {
            active_strategy: None,
            tracker_registry: TrackerRegistry::new(),
            aggregator: Arc::new(LockfreeAggregator::new(
                std::env::temp_dir().join("memscope_dispatcher"),
            )),
            config,
            metrics: DispatcherMetrics::default(),
        }
    }

    /// Select and activate optimal tracking strategy for given environment
    pub fn select_strategy(
        &mut self,
        environment: &RuntimeEnvironment,
    ) -> Result<TrackingStrategy, BackendError> {
        debug!("Selecting strategy for environment: {:?}", environment);

        let strategy = match environment {
            RuntimeEnvironment::SingleThreaded => TrackingStrategy::GlobalDirect,
            RuntimeEnvironment::MultiThreaded { thread_count } => {
                if *thread_count <= 2 {
                    TrackingStrategy::GlobalDirect
                } else {
                    TrackingStrategy::ThreadLocal
                }
            }
            RuntimeEnvironment::AsyncRuntime { .. } => TrackingStrategy::TaskLocal,
            RuntimeEnvironment::Hybrid {
                thread_count,
                async_task_count,
            } => {
                if *thread_count > 1 && *async_task_count > 0 {
                    TrackingStrategy::HybridTracking
                } else if *async_task_count > 0 {
                    TrackingStrategy::TaskLocal
                } else {
                    TrackingStrategy::ThreadLocal
                }
            }
        };

        info!(
            "Selected strategy: {:?} for environment: {:?}",
            strategy, environment
        );
        self.active_strategy = Some(strategy.clone());

        Ok(strategy)
    }

    /// Activate tracking with selected strategy
    pub fn activate_tracking(&mut self, strategy: TrackingStrategy) -> Result<(), BackendError> {
        info!("Activating tracking with strategy: {:?}", strategy);

        let tracker_type = self.strategy_to_tracker_type(&strategy);
        let tracker = self.tracker_registry.get_or_create_tracker(tracker_type)?;

        // Configure tracker
        let tracker_config = TrackerConfig::default();
        tracker.initialize(tracker_config).map_err(|e| {
            BackendError::TrackingInitializationFailed {
                reason: format!("Tracker initialization failed: {}", e),
            }
        })?;

        // Start tracking
        tracker
            .start_tracking()
            .map_err(|e| BackendError::TrackingInitializationFailed {
                reason: format!("Failed to start tracking: {}", e),
            })?;

        self.active_strategy = Some(strategy);
        self.metrics.active_trackers = self.tracker_registry.count_active_trackers();
        self.metrics.strategy_switches += 1;

        info!("Tracking activated successfully");
        Ok(())
    }

    /// Dispatch tracking operation to active tracker
    pub fn dispatch_tracking_operation(
        &mut self,
        operation: TrackingOperation,
    ) -> Result<(), BackendError> {
        let start_time = std::time::Instant::now();

        let strategy = self
            .active_strategy
            .as_ref()
            .ok_or_else(|| BackendError::TrackingInitializationFailed {
                reason: "No active tracking strategy".to_string(),
            })?
            .clone();

        let tracker_type = self.strategy_to_tracker_type(&strategy);

        // Split borrow to avoid conflict
        {
            let tracker = self.tracker_registry.get_or_create_tracker(tracker_type)?;
            // Execute operation
            Self::execute_operation_static(tracker, operation)?;
        }

        // Update metrics
        let dispatch_time = start_time.elapsed().as_micros() as f64;
        self.update_dispatch_metrics(dispatch_time);

        Ok(())
    }

    /// Execute tracking operation on specified tracker
    fn execute_operation_static(
        tracker: &mut Box<dyn MemoryTracker>,
        operation: TrackingOperation,
    ) -> Result<(), BackendError> {
        match operation {
            TrackingOperation::StartTracking => {
                if !tracker.is_active() {
                    tracker.start_tracking().map_err(|e| {
                        BackendError::TrackingInitializationFailed {
                            reason: format!("Failed to start tracking: {}", e),
                        }
                    })?;
                }
            }
            TrackingOperation::StopTracking => {
                if tracker.is_active() {
                    let _data =
                        tracker
                            .stop_tracking()
                            .map_err(|e| BackendError::DataCollectionError {
                                reason: format!("Failed to stop tracking: {}", e),
                            })?;
                    // Data would be processed here
                }
            }
            TrackingOperation::CollectData => {
                let stats = tracker.get_statistics();
                debug!("Collected statistics: {:?}", stats);
            }
        }

        Ok(())
    }

    /// Convert tracking strategy to tracker type
    fn strategy_to_tracker_type(&self, strategy: &TrackingStrategy) -> TrackerType {
        match strategy {
            TrackingStrategy::GlobalDirect => TrackerType::SingleThread,
            TrackingStrategy::ThreadLocal => TrackerType::MultiThread,
            TrackingStrategy::TaskLocal => TrackerType::AsyncTracker,
            TrackingStrategy::HybridTracking => TrackerType::HybridTracker,
        }
    }

    /// Update dispatch performance metrics
    fn update_dispatch_metrics(&mut self, dispatch_time_us: f64) {
        self.metrics.total_dispatches += 1;

        // Update running average of dispatch latency
        let weight = 0.1; // Exponential moving average weight
        self.metrics.avg_dispatch_latency_us =
            (1.0 - weight) * self.metrics.avg_dispatch_latency_us + weight * dispatch_time_us;
    }

    /// Collect data from all active trackers
    pub fn collect_all_data(&mut self) -> Result<Vec<u8>, BackendError> {
        debug!("Collecting data from all active trackers");

        let mut all_data = Vec::new();

        // Collect from each active tracker
        if let Some(ref mut tracker) = self.tracker_registry.single_thread_tracker {
            if tracker.is_active() {
                let data =
                    tracker
                        .stop_tracking()
                        .map_err(|e| BackendError::DataCollectionError {
                            reason: format!("Single thread tracker data collection failed: {}", e),
                        })?;
                all_data.extend(data);
            }
        }

        if let Some(ref mut tracker) = self.tracker_registry.multi_thread_tracker {
            if tracker.is_active() {
                let data =
                    tracker
                        .stop_tracking()
                        .map_err(|e| BackendError::DataCollectionError {
                            reason: format!("Multi thread tracker data collection failed: {}", e),
                        })?;
                all_data.extend(data);
            }
        }

        if let Some(ref mut tracker) = self.tracker_registry.async_tracker {
            if tracker.is_active() {
                let data =
                    tracker
                        .stop_tracking()
                        .map_err(|e| BackendError::DataCollectionError {
                            reason: format!("Async tracker data collection failed: {}", e),
                        })?;
                all_data.extend(data);
            }
        }

        if let Some(ref mut tracker) = self.tracker_registry.hybrid_tracker {
            if tracker.is_active() {
                let data =
                    tracker
                        .stop_tracking()
                        .map_err(|e| BackendError::DataCollectionError {
                            reason: format!("Hybrid tracker data collection failed: {}", e),
                        })?;
                all_data.extend(data);
            }
        }

        info!("Collected {} bytes of tracking data", all_data.len());
        Ok(all_data)
    }

    /// Get current dispatcher metrics
    pub fn get_metrics(&self) -> &DispatcherMetrics {
        &self.metrics
    }

    /// Shutdown dispatcher and cleanup all trackers
    pub fn shutdown(mut self) -> Result<Vec<u8>, BackendError> {
        info!("Shutting down tracking dispatcher");

        let final_data = self.collect_all_data()?;

        debug!("Dispatcher shutdown completed");
        Ok(final_data)
    }
}

/// Tracking operations that can be dispatched
#[derive(Debug, Clone)]
pub enum TrackingOperation {
    /// Start tracking operation
    StartTracking,
    /// Stop tracking operation
    StopTracking,
    /// Collect current data
    CollectData,
}

// Placeholder tracker implementations
// These would be replaced with actual implementations

/// Single-threaded tracker implementation
struct SingleThreadTracker {
    active: bool,
    allocations: u64,
}

impl SingleThreadTracker {
    fn new() -> Self {
        Self {
            active: false,
            allocations: 0,
        }
    }
}

impl MemoryTracker for SingleThreadTracker {
    fn initialize(&mut self, _config: TrackerConfig) -> Result<(), TrackerError> {
        debug!("Initializing single thread tracker");
        Ok(())
    }

    fn start_tracking(&mut self) -> Result<(), TrackerError> {
        debug!("Starting single thread tracking");
        self.active = true;
        Ok(())
    }

    fn stop_tracking(&mut self) -> Result<Vec<u8>, TrackerError> {
        debug!("Stopping single thread tracking");
        self.active = false;
        Ok(vec![]) // Placeholder data
    }

    fn get_statistics(&self) -> TrackerStatistics {
        TrackerStatistics {
            allocations_tracked: self.allocations,
            memory_tracked_bytes: 0,
            overhead_bytes: 0,
            tracking_duration_ms: 0,
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn tracker_type(&self) -> TrackerType {
        TrackerType::SingleThread
    }
}

/// Multi-threaded tracker implementation
struct MultiThreadTracker {
    active: bool,
}

impl MultiThreadTracker {
    fn new() -> Self {
        Self { active: false }
    }
}

impl MemoryTracker for MultiThreadTracker {
    fn initialize(&mut self, _config: TrackerConfig) -> Result<(), TrackerError> {
        debug!("Initializing multi thread tracker");
        Ok(())
    }

    fn start_tracking(&mut self) -> Result<(), TrackerError> {
        debug!("Starting multi thread tracking");
        self.active = true;
        Ok(())
    }

    fn stop_tracking(&mut self) -> Result<Vec<u8>, TrackerError> {
        debug!("Stopping multi thread tracking");
        self.active = false;
        Ok(vec![])
    }

    fn get_statistics(&self) -> TrackerStatistics {
        TrackerStatistics {
            allocations_tracked: 0,
            memory_tracked_bytes: 0,
            overhead_bytes: 0,
            tracking_duration_ms: 0,
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn tracker_type(&self) -> TrackerType {
        TrackerType::MultiThread
    }
}

/// Async tracker wrapper
struct AsyncTrackerWrapper {
    active: bool,
}

impl AsyncTrackerWrapper {
    fn new() -> Self {
        Self { active: false }
    }
}

impl MemoryTracker for AsyncTrackerWrapper {
    fn initialize(&mut self, _config: TrackerConfig) -> Result<(), TrackerError> {
        debug!("Initializing async tracker wrapper");
        Ok(())
    }

    fn start_tracking(&mut self) -> Result<(), TrackerError> {
        debug!("Starting async tracking");
        self.active = true;
        Ok(())
    }

    fn stop_tracking(&mut self) -> Result<Vec<u8>, TrackerError> {
        debug!("Stopping async tracking");
        self.active = false;
        Ok(vec![])
    }

    fn get_statistics(&self) -> TrackerStatistics {
        TrackerStatistics {
            allocations_tracked: 0,
            memory_tracked_bytes: 0,
            overhead_bytes: 0,
            tracking_duration_ms: 0,
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn tracker_type(&self) -> TrackerType {
        TrackerType::AsyncTracker
    }
}

/// Hybrid tracker implementation
struct HybridTrackerImpl {
    active: bool,
}

impl HybridTrackerImpl {
    fn new() -> Self {
        Self { active: false }
    }
}

impl MemoryTracker for HybridTrackerImpl {
    fn initialize(&mut self, _config: TrackerConfig) -> Result<(), TrackerError> {
        debug!("Initializing hybrid tracker");
        Ok(())
    }

    fn start_tracking(&mut self) -> Result<(), TrackerError> {
        debug!("Starting hybrid tracking");
        self.active = true;
        Ok(())
    }

    fn stop_tracking(&mut self) -> Result<Vec<u8>, TrackerError> {
        debug!("Stopping hybrid tracking");
        self.active = false;
        Ok(vec![])
    }

    fn get_statistics(&self) -> TrackerStatistics {
        TrackerStatistics {
            allocations_tracked: 0,
            memory_tracked_bytes: 0,
            overhead_bytes: 0,
            tracking_duration_ms: 0,
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn tracker_type(&self) -> TrackerType {
        TrackerType::HybridTracker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_creation() {
        let config = DispatcherConfig::default();
        let dispatcher = TrackingDispatcher::new(config);
        assert!(dispatcher.active_strategy.is_none());
    }

    #[test]
    fn test_strategy_selection() {
        let config = DispatcherConfig::default();
        let mut dispatcher = TrackingDispatcher::new(config);

        let env = RuntimeEnvironment::SingleThreaded;
        let strategy = dispatcher.select_strategy(&env);

        assert!(strategy.is_ok());
        assert_eq!(strategy.unwrap(), TrackingStrategy::GlobalDirect);
    }

    #[test]
    fn test_tracker_registry() {
        let mut registry = TrackerRegistry::new();
        let tracker = registry.get_or_create_tracker(TrackerType::SingleThread);
        assert!(tracker.is_ok());
    }

    #[test]
    fn test_single_thread_tracker() {
        let mut tracker = SingleThreadTracker::new();
        assert!(!tracker.is_active());

        let result = tracker.start_tracking();
        assert!(result.is_ok());
        assert!(tracker.is_active());

        let data = tracker.stop_tracking();
        assert!(data.is_ok());
        assert!(!tracker.is_active());
    }
}
