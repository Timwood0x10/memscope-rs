// Unified Backend System for Memory Tracking
// Provides intelligent routing between single-thread, multi-thread, and async tracking strategies
// Maintains zero-lock architecture and preserves existing JSON export compatibility

use crate::lockfree::aggregator::LockfreeAggregator;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Main unified backend that orchestrates all memory tracking strategies
/// Acts as the central hub for routing tracking requests to appropriate handlers
pub struct UnifiedBackend {
    /// Current runtime environment detection result
    environment: RuntimeEnvironment,
    /// Active tracking strategy selected based on environment
    active_strategy: TrackingStrategy,
    /// Configuration for backend behavior
    config: BackendConfig,
    /// Aggregator for collecting data from all tracking sources
    aggregator: Arc<LockfreeAggregator>,
}

/// Detected runtime environment characteristics
/// Used to determine optimal tracking strategy
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeEnvironment {
    /// Single-threaded execution detected
    SingleThreaded,
    /// Multi-threaded execution with thread count
    MultiThreaded { thread_count: usize },
    /// Async runtime detected with runtime type
    AsyncRuntime { runtime_type: AsyncRuntimeType },
    /// Hybrid mode with both threads and async tasks
    Hybrid {
        thread_count: usize,
        async_task_count: usize,
    },
}

/// Supported async runtime types for specialized tracking
#[derive(Debug, Clone, PartialEq)]
pub enum AsyncRuntimeType {
    /// Tokio runtime
    Tokio,
    /// async-std runtime  
    AsyncStd,
    /// Custom or unknown runtime
    Custom,
}

/// Selected tracking strategy based on environment analysis
#[derive(Debug, Clone, PartialEq)]
pub enum TrackingStrategy {
    /// Direct global tracking for single-threaded apps
    GlobalDirect,
    /// Thread-local storage for multi-threaded apps
    ThreadLocal,
    /// Task-local storage for async applications
    TaskLocal,
    /// Combined strategy for hybrid applications
    HybridTracking,
}

/// Configuration options for backend behavior
#[derive(Debug, Clone)]
pub struct BackendConfig {
    /// Enable automatic environment detection
    pub auto_detect: bool,
    /// Forced strategy override (bypasses auto-detection)
    pub force_strategy: Option<TrackingStrategy>,
    /// Performance sampling rate (0.0 to 1.0)
    pub sample_rate: f64,
    /// Maximum memory overhead percentage
    pub max_overhead_percent: f64,
}

/// Tracking session handle for controlling active tracking
pub struct TrackingSession {
    /// Session identifier for debugging
    session_id: String,
    /// Backend reference for data collection
    backend: Arc<UnifiedBackend>,
    /// Session start timestamp
    start_time: std::time::Instant,
}

/// Comprehensive memory analysis data from tracking session
#[derive(Debug)]
pub struct MemoryAnalysisData {
    /// Raw tracking data from all sources
    pub raw_data: Vec<u8>,
    /// Aggregated statistics
    pub statistics: MemoryStatistics,
    /// Environment context for analysis
    pub environment: RuntimeEnvironment,
    /// Session metadata
    pub session_metadata: SessionMetadata,
}

/// Statistical summary of memory tracking session
#[derive(Debug)]
pub struct MemoryStatistics {
    /// Total allocations tracked
    pub total_allocations: usize,
    /// Peak memory usage
    pub peak_memory_bytes: usize,
    /// Average allocation size
    pub avg_allocation_size: f64,
    /// Tracking session duration
    pub session_duration_ms: u64,
}

/// Session metadata for analysis context
#[derive(Debug)]
pub struct SessionMetadata {
    /// Session unique identifier
    pub session_id: String,
    /// Environment detection results
    pub detected_environment: RuntimeEnvironment,
    /// Strategy used for tracking
    pub strategy_used: TrackingStrategy,
    /// Performance overhead measured
    pub overhead_percent: f64,
}

/// Backend operation errors
#[derive(Error, Debug)]
pub enum BackendError {
    /// Environment detection failed
    #[error("Failed to detect runtime environment: {reason}")]
    EnvironmentDetectionFailed { reason: String },

    /// Strategy selection failed
    #[error("Cannot select appropriate tracking strategy for environment: {environment:?}")]
    StrategySelectionFailed { environment: RuntimeEnvironment },

    /// Tracking initialization failed
    #[error("Failed to initialize tracking session: {reason}")]
    TrackingInitializationFailed { reason: String },

    /// Data collection error
    #[error("Error collecting tracking data: {reason}")]
    DataCollectionError { reason: String },

    /// Configuration validation error
    #[error("Invalid backend configuration: {reason}")]
    ConfigurationError { reason: String },
}

impl Default for BackendConfig {
    /// Default configuration optimized for most use cases
    fn default() -> Self {
        Self {
            auto_detect: true,
            force_strategy: None,
            sample_rate: 1.0,
            max_overhead_percent: 5.0,
        }
    }
}

impl UnifiedBackend {
    /// Initialize unified backend with configuration
    /// Performs environment detection and strategy selection
    pub fn initialize(config: BackendConfig) -> Result<Self, BackendError> {
        // Validate configuration parameters
        if config.sample_rate < 0.0 || config.sample_rate > 1.0 {
            return Err(BackendError::ConfigurationError {
                reason: "Sample rate must be between 0.0 and 1.0".to_string(),
            });
        }

        if config.max_overhead_percent < 0.0 || config.max_overhead_percent > 100.0 {
            return Err(BackendError::ConfigurationError {
                reason: "Max overhead percent must be between 0.0 and 100.0".to_string(),
            });
        }

        info!("Initializing unified backend with config: {:?}", config);

        // Detect runtime environment
        let environment = if config.auto_detect {
            Self::detect_environment()?
        } else {
            RuntimeEnvironment::SingleThreaded // Default fallback
        };

        debug!("Detected environment: {:?}", environment);

        // Select optimal tracking strategy
        let active_strategy = if let Some(forced) = config.force_strategy.clone() {
            warn!("Using forced strategy: {:?}", forced);
            forced
        } else {
            Self::select_strategy(&environment)?
        };

        info!("Selected tracking strategy: {:?}", active_strategy);

        // Initialize aggregator for data collection
        let output_dir = std::env::temp_dir().join("memscope_unified");
        let aggregator = Arc::new(LockfreeAggregator::new(output_dir));

        Ok(Self {
            environment,
            active_strategy,
            config,
            aggregator,
        })
    }

    /// Detect current runtime environment characteristics
    /// Analyzes thread count, async runtime presence, and execution patterns
    pub fn detect_environment() -> Result<RuntimeEnvironment, BackendError> {
        debug!("Starting environment detection");

        // Check for async runtime presence
        let async_runtime = Self::detect_async_runtime();

        // Count available threads
        let thread_count = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1);

        // Determine environment type based on detection results
        let environment = match (async_runtime, thread_count) {
            (Some(runtime_type), 0) => {
                // Edge case: async runtime detected but no threads available
                RuntimeEnvironment::AsyncRuntime { runtime_type }
            }
            (Some(runtime_type), 1) => RuntimeEnvironment::AsyncRuntime { runtime_type },
            (Some(_runtime_type), threads) => {
                RuntimeEnvironment::Hybrid {
                    thread_count: threads,
                    async_task_count: 0, // Will be detected during runtime
                }
            }
            (None, 1) => RuntimeEnvironment::SingleThreaded,
            (None, threads) => RuntimeEnvironment::MultiThreaded {
                thread_count: threads,
            },
        };

        debug!("Environment detection completed: {:?}", environment);
        Ok(environment)
    }

    /// Detect presence and type of async runtime
    /// Returns None if no async runtime is detected
    fn detect_async_runtime() -> Option<AsyncRuntimeType> {
        // Check for Tokio runtime
        if Self::is_tokio_present() {
            debug!("Tokio runtime detected");
            return Some(AsyncRuntimeType::Tokio);
        }

        // Check for async-std runtime
        if Self::is_async_std_present() {
            debug!("async-std runtime detected");
            return Some(AsyncRuntimeType::AsyncStd);
        }

        // No known async runtime detected
        debug!("No async runtime detected");
        None
    }

    /// Check if Tokio runtime is currently active
    fn is_tokio_present() -> bool {
        // Use feature detection or runtime introspection
        // This is a simplified implementation - real detection would be more sophisticated
        std::env::var("TOKIO_WORKER_THREADS").is_ok()
        // Note: tokio::runtime::Handle::try_current() requires tokio dependency
    }

    /// Check if async-std runtime is active
    fn is_async_std_present() -> bool {
        // async-std detection logic would go here
        // This is a placeholder for actual implementation
        false
    }

    /// Select optimal tracking strategy based on environment
    fn select_strategy(environment: &RuntimeEnvironment) -> Result<TrackingStrategy, BackendError> {
        let strategy = match environment {
            RuntimeEnvironment::SingleThreaded => TrackingStrategy::GlobalDirect,
            RuntimeEnvironment::MultiThreaded { .. } => TrackingStrategy::ThreadLocal,
            RuntimeEnvironment::AsyncRuntime { .. } => TrackingStrategy::TaskLocal,
            RuntimeEnvironment::Hybrid { .. } => TrackingStrategy::HybridTracking,
        };

        debug!(
            "Selected strategy {:?} for environment {:?}",
            strategy, environment
        );
        Ok(strategy)
    }

    /// Start active memory tracking session
    /// Returns session handle for controlling tracking lifecycle
    pub fn start_tracking(&mut self) -> Result<TrackingSession, BackendError> {
        let session_id = format!(
            "session_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| BackendError::TrackingInitializationFailed {
                    reason: format!("Failed to generate session ID: {}", e),
                })?
                .as_millis()
        );

        info!("Starting tracking session: {}", session_id);

        // Initialize tracking based on selected strategy
        match self.active_strategy {
            TrackingStrategy::GlobalDirect => {
                self.initialize_global_tracking()?;
            }
            TrackingStrategy::ThreadLocal => {
                self.initialize_thread_local_tracking()?;
            }
            TrackingStrategy::TaskLocal => {
                self.initialize_task_local_tracking()?;
            }
            TrackingStrategy::HybridTracking => {
                self.initialize_hybrid_tracking()?;
            }
        }

        let session = TrackingSession {
            session_id: session_id.clone(),
            backend: Arc::new(self.clone()),
            start_time: std::time::Instant::now(),
        };

        debug!("Tracking session {} started successfully", session_id);
        Ok(session)
    }

    /// Initialize global direct tracking for single-threaded applications
    fn initialize_global_tracking(&mut self) -> Result<(), BackendError> {
        debug!("Initializing global direct tracking");
        // Implementation will integrate with existing lockfree aggregator
        Ok(())
    }

    /// Initialize thread-local tracking for multi-threaded applications
    fn initialize_thread_local_tracking(&mut self) -> Result<(), BackendError> {
        debug!("Initializing thread-local tracking");
        // Implementation will use thread_local! storage
        Ok(())
    }

    /// Initialize task-local tracking for async applications
    fn initialize_task_local_tracking(&mut self) -> Result<(), BackendError> {
        debug!("Initializing task-local tracking");
        // Implementation will integrate with AsyncMemoryTracker
        Ok(())
    }

    /// Initialize hybrid tracking for complex applications
    fn initialize_hybrid_tracking(&mut self) -> Result<(), BackendError> {
        debug!("Initializing hybrid tracking");
        // Implementation will combine multiple strategies
        Ok(())
    }

    /// Collect all tracking data from active session
    /// Aggregates data from all tracking sources into unified format
    pub fn collect_data(&self) -> Result<MemoryAnalysisData, BackendError> {
        debug!("Collecting tracking data");

        // Collect raw data from aggregator (placeholder for now)
        // TODO: Implement proper aggregator data collection
        let raw_data = vec![];

        // Calculate statistics
        let statistics = self.calculate_statistics(&raw_data)?;

        // Create session metadata
        let session_metadata = SessionMetadata {
            session_id: "current_session".to_string(), // Will be proper session ID
            detected_environment: self.environment.clone(),
            strategy_used: self.active_strategy.clone(),
            overhead_percent: self.measure_overhead(),
        };

        let analysis_data = MemoryAnalysisData {
            raw_data,
            statistics,
            environment: self.environment.clone(),
            session_metadata,
        };

        info!(
            "Data collection completed, {} allocations tracked",
            analysis_data.statistics.total_allocations
        );

        Ok(analysis_data)
    }

    /// Calculate statistical summary from raw tracking data
    fn calculate_statistics(&self, _raw_data: &[u8]) -> Result<MemoryStatistics, BackendError> {
        // This would parse the raw data and calculate actual statistics
        // For now, return placeholder statistics
        Ok(MemoryStatistics {
            total_allocations: 0,
            peak_memory_bytes: 0,
            avg_allocation_size: 0.0,
            session_duration_ms: 0,
        })
    }

    /// Measure current tracking overhead percentage
    fn measure_overhead(&self) -> f64 {
        // Implementation would measure actual performance impact
        // Return configured max overhead as placeholder
        self.config.max_overhead_percent
    }

    /// Shutdown backend and finalize all tracking
    pub fn shutdown(self) -> Result<MemoryAnalysisData, BackendError> {
        info!("Shutting down unified backend");

        // Collect final data before shutdown
        let final_data = self.collect_data()?;

        debug!("Backend shutdown completed successfully");
        Ok(final_data)
    }
}

// Required for Arc usage in TrackingSession
impl Clone for UnifiedBackend {
    fn clone(&self) -> Self {
        Self {
            environment: self.environment.clone(),
            active_strategy: self.active_strategy.clone(),
            config: self.config.clone(),
            aggregator: Arc::clone(&self.aggregator),
        }
    }
}

impl TrackingSession {
    /// Get session identifier
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get elapsed time since session start
    pub fn elapsed_time(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Collect current tracking data without ending session
    pub fn collect_data(&self) -> Result<MemoryAnalysisData, BackendError> {
        self.backend.collect_data()
    }

    /// End tracking session and collect final data
    pub fn end_session(self) -> Result<MemoryAnalysisData, BackendError> {
        info!("Ending tracking session: {}", self.session_id);

        let final_data = self.backend.collect_data()?;

        debug!(
            "Session {} ended after {:?}",
            self.session_id,
            self.start_time.elapsed()
        );

        Ok(final_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_initialization() {
        let config = BackendConfig::default();
        let backend = UnifiedBackend::initialize(config);
        assert!(backend.is_ok());
    }

    #[test]
    fn test_environment_detection() {
        let env = UnifiedBackend::detect_environment();
        assert!(env.is_ok());
    }

    #[test]
    fn test_invalid_config_sample_rate() {
        let config = BackendConfig {
            sample_rate: 1.5, // Invalid: > 1.0
            ..Default::default()
        };
        let result = UnifiedBackend::initialize(config);
        assert!(matches!(
            result,
            Err(BackendError::ConfigurationError { .. })
        ));
    }

    #[test]
    fn test_strategy_selection() {
        let env = RuntimeEnvironment::SingleThreaded;
        let strategy = UnifiedBackend::select_strategy(&env);
        assert!(matches!(strategy, Ok(TrackingStrategy::GlobalDirect)));
    }
}
// Tracking Strategy Dispatcher
// Intelligently routes memory tracking requests to optimal tracking implementations
// Manages lifecycle of different tracking strategies and data aggregation

use crate::lockfree::aggregator::LockfreeAggregator;
use crate::unified::backend::{BackendError, RuntimeEnvironment, TrackingStrategy};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info};

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
// Runtime Environment Detection System
// Intelligently detects execution context to optimize memory tracking strategy
// Supports single-thread, multi-thread, async, and hybrid runtime detection

use crate::unified::backend::{AsyncRuntimeType, BackendError, RuntimeEnvironment};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Advanced environment detector with runtime analysis capabilities
/// Provides deep inspection of execution context for optimal tracking strategy selection
pub struct EnvironmentDetector {
    /// Detection configuration parameters
    config: DetectionConfig,
    /// Runtime statistics collector
    runtime_stats: RuntimeStatistics,
}

/// Configuration for environment detection behavior
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    /// Enable deep async runtime analysis
    pub deep_async_detection: bool,
    /// Sampling period for runtime analysis (milliseconds)
    pub analysis_period_ms: u64,
    /// Threshold for considering environment as multi-threaded
    pub multi_thread_threshold: usize,
    /// Maximum time to spend on detection
    pub max_detection_time_ms: u64,
}

/// Runtime statistics collected during detection
#[derive(Debug, Clone)]
pub struct RuntimeStatistics {
    /// Active thread count observed
    pub active_threads: Arc<AtomicUsize>,
    /// Async task count (if detectable)
    pub async_tasks: Arc<AtomicUsize>,
    /// Peak thread utilization
    pub peak_thread_count: usize,
    /// Detection duration
    pub detection_duration_ms: u64,
}

/// Detailed environment analysis result
#[derive(Debug, Clone)]
pub struct EnvironmentAnalysis {
    /// Primary detected environment
    pub environment: RuntimeEnvironment,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Alternative environments considered
    pub alternatives: Vec<RuntimeEnvironment>,
    /// Detection metadata
    pub detection_metadata: DetectionMetadata,
}

/// Metadata about detection process
#[derive(Debug, Clone)]
pub struct DetectionMetadata {
    /// Time spent on detection
    pub detection_time_ms: u64,
    /// Number of samples taken
    pub sample_count: usize,
    /// Detection method used
    pub method: DetectionMethod,
    /// Warnings or issues during detection
    pub warnings: Vec<String>,
}

/// Method used for environment detection
#[derive(Debug, Clone, PartialEq)]
pub enum DetectionMethod {
    /// Static analysis only
    Static,
    /// Runtime sampling
    Dynamic,
    /// Combined static and dynamic
    Hybrid,
    /// Forced by configuration
    Manual,
}

impl Default for DetectionConfig {
    /// Default detection configuration optimized for accuracy and performance
    fn default() -> Self {
        Self {
            deep_async_detection: true,
            analysis_period_ms: 100,
            multi_thread_threshold: 2,
            max_detection_time_ms: 500,
        }
    }
}

impl Default for RuntimeStatistics {
    /// Initialize runtime statistics with zero values
    fn default() -> Self {
        Self {
            active_threads: Arc::new(AtomicUsize::new(1)),
            async_tasks: Arc::new(AtomicUsize::new(0)),
            peak_thread_count: 1,
            detection_duration_ms: 0,
        }
    }
}

impl EnvironmentDetector {
    /// Create new environment detector with configuration
    pub fn new(config: DetectionConfig) -> Self {
        debug!("Creating environment detector with config: {:?}", config);

        Self {
            config,
            runtime_stats: RuntimeStatistics::default(),
        }
    }

    /// Perform comprehensive environment detection and analysis
    /// Returns detailed analysis with confidence levels and alternatives
    pub fn analyze_environment(&mut self) -> Result<EnvironmentAnalysis, BackendError> {
        let start_time = std::time::Instant::now();
        info!("Starting comprehensive environment analysis");

        let mut warnings = Vec::new();

        // Phase 1: Static analysis
        let static_env = self.perform_static_analysis(&mut warnings)?;
        debug!("Static analysis result: {:?}", static_env);

        // Phase 2: Dynamic runtime analysis (if enabled)
        let dynamic_env = if self.config.deep_async_detection {
            Some(self.perform_dynamic_analysis(&mut warnings)?)
        } else {
            None
        };

        // Phase 3: Combine results and calculate confidence
        let has_dynamic = dynamic_env.is_some();
        let (final_env, confidence, alternatives) =
            self.synthesize_results(static_env, dynamic_env, &warnings)?;

        let detection_time = start_time.elapsed().as_millis() as u64;
        self.runtime_stats.detection_duration_ms = detection_time;

        let analysis = EnvironmentAnalysis {
            environment: final_env,
            confidence,
            alternatives,
            detection_metadata: DetectionMetadata {
                detection_time_ms: detection_time,
                sample_count: self.calculate_sample_count(),
                method: self.determine_detection_method(has_dynamic),
                warnings,
            },
        };

        info!(
            "Environment analysis completed: {:?} (confidence: {:.2})",
            analysis.environment, analysis.confidence
        );

        Ok(analysis)
    }

    /// Perform static environment analysis based on available system information
    fn perform_static_analysis(
        &self,
        warnings: &mut Vec<String>,
    ) -> Result<RuntimeEnvironment, BackendError> {
        debug!("Performing static environment analysis");

        // Detect available parallelism
        let logical_cores = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or_else(|e| {
                warnings.push(format!("Could not detect CPU cores: {}", e));
                1
            });

        debug!("Detected {} logical CPU cores", logical_cores);

        // Check for async runtime indicators
        let async_runtime = self.detect_async_runtime_static(warnings);

        // Determine base environment from static analysis
        let environment = match (async_runtime, logical_cores) {
            (Some(runtime_type), 0) => {
                warnings.push("Zero cores detected with async runtime".to_string());
                RuntimeEnvironment::AsyncRuntime { runtime_type }
            }
            (Some(runtime_type), 1) => RuntimeEnvironment::AsyncRuntime { runtime_type },
            (Some(_runtime_type), cores) if cores >= self.config.multi_thread_threshold => {
                RuntimeEnvironment::Hybrid {
                    thread_count: cores,
                    async_task_count: 0, // Will be determined dynamically
                }
            }
            (Some(runtime_type), _cores) => {
                // Low core count with async runtime
                RuntimeEnvironment::AsyncRuntime { runtime_type }
            }
            (None, 1) => RuntimeEnvironment::SingleThreaded,
            (None, cores) if cores >= self.config.multi_thread_threshold => {
                RuntimeEnvironment::MultiThreaded {
                    thread_count: cores,
                }
            }
            (None, cores) => {
                warnings.push(format!("Low core count {} but above single-thread", cores));
                RuntimeEnvironment::SingleThreaded
            }
        };

        debug!("Static analysis determined environment: {:?}", environment);
        Ok(environment)
    }

    /// Detect async runtime presence using static indicators
    fn detect_async_runtime_static(&self, warnings: &mut Vec<String>) -> Option<AsyncRuntimeType> {
        debug!("Detecting async runtime using static analysis");

        // Check for Tokio runtime
        if self.is_tokio_runtime_present() {
            debug!("Tokio runtime detected via static analysis");
            return Some(AsyncRuntimeType::Tokio);
        }

        // Check for async-std runtime
        if self.is_async_std_runtime_present() {
            debug!("async-std runtime detected via static analysis");
            return Some(AsyncRuntimeType::AsyncStd);
        }

        // Check environment variables for async indicators
        if let Ok(async_env) = std::env::var("ASYNC_RUNTIME") {
            match async_env.to_lowercase().as_str() {
                "tokio" => {
                    debug!("Tokio runtime detected via environment variable");
                    return Some(AsyncRuntimeType::Tokio);
                }
                "async-std" => {
                    debug!("async-std runtime detected via environment variable");
                    return Some(AsyncRuntimeType::AsyncStd);
                }
                other => {
                    warnings.push(format!("Unknown async runtime specified: {}", other));
                    return Some(AsyncRuntimeType::Custom);
                }
            }
        }

        debug!("No async runtime detected in static analysis");
        None
    }

    /// Check for Tokio runtime presence using available detection methods
    fn is_tokio_runtime_present(&self) -> bool {
        // Method 1: Check for Tokio environment variables
        if std::env::var("TOKIO_WORKER_THREADS").is_ok() {
            return true;
        }

        // Note: tokio::runtime::Handle::try_current() requires tokio dependency
        // Would be enabled when tokio feature is available

        false
    }

    /// Check for async-std runtime presence
    fn is_async_std_runtime_present(&self) -> bool {
        // async-std detection is more challenging as it has fewer runtime introspection APIs
        // Check for async-std specific environment variables or patterns

        // Method 1: Check for async-std environment indicators
        if std::env::var("ASYNC_STD_THREAD_COUNT").is_ok() {
            return true;
        }

        // Method 2: Check if we're running inside async-std executor
        // This would require async-std specific detection logic

        false
    }

    /// Perform dynamic runtime analysis through sampling and observation
    fn perform_dynamic_analysis(
        &mut self,
        warnings: &mut Vec<String>,
    ) -> Result<RuntimeEnvironment, BackendError> {
        debug!("Performing dynamic runtime analysis");

        let analysis_start = std::time::Instant::now();
        let max_duration = std::time::Duration::from_millis(self.config.max_detection_time_ms);

        // Sample runtime characteristics over time
        let mut sample_count = 0;
        let mut thread_samples = Vec::new();
        let mut async_indicators = Vec::new();

        while analysis_start.elapsed() < max_duration {
            // Sample current thread activity
            let current_threads = self.sample_thread_activity();
            thread_samples.push(current_threads);

            // Sample async task activity (if possible)
            let async_activity = self.sample_async_activity();
            async_indicators.push(async_activity);

            sample_count += 1;

            // Sleep for sampling interval
            std::thread::sleep(std::time::Duration::from_millis(
                self.config.analysis_period_ms / 10,
            ));
        }

        // Analyze collected samples
        let avg_threads = if thread_samples.is_empty() {
            1
        } else {
            thread_samples.iter().sum::<usize>() / thread_samples.len()
        };

        let peak_threads = thread_samples.into_iter().max().unwrap_or(1);
        self.runtime_stats.peak_thread_count = peak_threads;

        let has_async_activity = async_indicators.iter().any(|&active| active);

        debug!(
            "Dynamic analysis: avg_threads={}, peak_threads={}, async_activity={}",
            avg_threads, peak_threads, has_async_activity
        );

        // Determine environment from dynamic analysis
        let environment = match (has_async_activity, peak_threads) {
            (true, 0) => {
                warnings.push("Async activity detected with zero threads".to_string());
                let runtime_type = self
                    .detect_async_runtime_static(warnings)
                    .unwrap_or(AsyncRuntimeType::Custom);
                RuntimeEnvironment::AsyncRuntime { runtime_type }
            }
            (true, 1) => {
                // Async activity on single thread suggests async runtime
                let runtime_type = self
                    .detect_async_runtime_static(warnings)
                    .unwrap_or(AsyncRuntimeType::Custom);
                RuntimeEnvironment::AsyncRuntime { runtime_type }
            }
            (true, threads) => {
                // Both async and multi-thread activity
                RuntimeEnvironment::Hybrid {
                    thread_count: threads,
                    async_task_count: sample_count, // Use sample count as proxy
                }
            }
            (false, 1) => RuntimeEnvironment::SingleThreaded,
            (false, threads) => RuntimeEnvironment::MultiThreaded {
                thread_count: threads,
            },
        };

        debug!("Dynamic analysis determined environment: {:?}", environment);
        Ok(environment)
    }

    /// Sample current thread activity level
    fn sample_thread_activity(&self) -> usize {
        // This is a simplified implementation
        // Real implementation would use platform-specific APIs to count active threads

        // For now, use available parallelism as baseline
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1)
    }

    /// Sample async task activity
    fn sample_async_activity(&self) -> bool {
        // This would use runtime-specific APIs to detect async task activity
        // For now, use simple heuristics

        // Check if we can detect any async runtime activity
        self.is_tokio_runtime_present() || self.is_async_std_runtime_present()
    }

    /// Synthesize static and dynamic analysis results
    fn synthesize_results(
        &self,
        static_result: RuntimeEnvironment,
        dynamic_result: Option<RuntimeEnvironment>,
        warnings: &[String],
    ) -> Result<(RuntimeEnvironment, f64, Vec<RuntimeEnvironment>), BackendError> {
        debug!("Synthesizing analysis results");

        let mut alternatives = Vec::new();
        let _base_confidence = 0.7; // Base confidence

        let (final_environment, mut confidence) = match dynamic_result {
            Some(dynamic_env) => {
                // Compare static and dynamic results
                if std::mem::discriminant(&static_result) == std::mem::discriminant(&dynamic_env) {
                    // Results agree - high confidence
                    let confidence = 0.95;
                    alternatives.push(static_result);
                    (dynamic_env, confidence)
                } else {
                    // Results disagree - medium confidence, prefer dynamic
                    let confidence = 0.75;
                    alternatives.push(static_result);
                    (dynamic_env, confidence)
                }
            }
            None => {
                // Only static analysis available
                let confidence = 0.80;
                (static_result, confidence)
            }
        };

        // Adjust confidence based on warnings
        if !warnings.is_empty() {
            confidence -= 0.1 * warnings.len() as f64;
            confidence = confidence.max(0.3); // Minimum confidence threshold
        }

        debug!(
            "Final synthesis: {:?} with confidence {:.2}",
            final_environment, confidence
        );
        Ok((final_environment, confidence, alternatives))
    }

    /// Calculate total number of samples taken during analysis
    fn calculate_sample_count(&self) -> usize {
        // This would be tracked during dynamic analysis
        // For now, estimate based on detection duration
        let samples = self.runtime_stats.detection_duration_ms / self.config.analysis_period_ms;
        samples.max(1) as usize
    }

    /// Determine which detection method was primarily used
    fn determine_detection_method(&self, used_dynamic: bool) -> DetectionMethod {
        if used_dynamic {
            DetectionMethod::Hybrid
        } else {
            DetectionMethod::Static
        }
    }

    /// Get current runtime statistics
    pub fn runtime_statistics(&self) -> &RuntimeStatistics {
        &self.runtime_stats
    }
}

/// Convenience function for quick environment detection
/// Uses default configuration for most common use cases
pub fn detect_environment() -> Result<RuntimeEnvironment, BackendError> {
    let mut detector = EnvironmentDetector::new(DetectionConfig::default());
    let analysis = detector.analyze_environment()?;

    if analysis.confidence < 0.5 {
        warn!(
            "Low confidence environment detection: {:.2}",
            analysis.confidence
        );
    }

    Ok(analysis.environment)
}

/// Advanced environment detection with custom configuration
/// Provides detailed analysis results for advanced use cases
pub fn detect_environment_detailed(
    config: DetectionConfig,
) -> Result<EnvironmentAnalysis, BackendError> {
    let mut detector = EnvironmentDetector::new(config);
    detector.analyze_environment()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let config = DetectionConfig::default();
        let detector = EnvironmentDetector::new(config);
        assert_eq!(detector.runtime_stats.peak_thread_count, 1);
    }

    #[test]
    fn test_static_analysis() {
        let config = DetectionConfig::default();
        let detector = EnvironmentDetector::new(config);
        let mut warnings = Vec::new();

        let result = detector.perform_static_analysis(&mut warnings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tokio_detection() {
        let config = DetectionConfig::default();
        let detector = EnvironmentDetector::new(config);

        // This test will pass regardless of Tokio presence
        let _has_tokio = detector.is_tokio_runtime_present();
        // Just ensure the method doesn't panic
    }

    #[test]
    fn test_environment_analysis_confidence() {
        let mut detector = EnvironmentDetector::new(DetectionConfig::default());
        let analysis = detector.analyze_environment();

        assert!(analysis.is_ok());
        let analysis = analysis.unwrap();
        assert!(analysis.confidence >= 0.0 && analysis.confidence <= 1.0);
    }

    #[test]
    fn test_quick_detection() {
        let result = detect_environment();
        assert!(result.is_ok());
    }

    #[test]
    fn test_detailed_detection() {
        let config = DetectionConfig {
            deep_async_detection: false, // Disable for faster test
            max_detection_time_ms: 50,   // Short timeout for test
            ..Default::default()
        };

        let result = detect_environment_detailed(config);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.detection_metadata.detection_time_ms <= 100); // Should be quick
    }
}
