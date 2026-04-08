//! Unified tracker types and environment detection.
//!
//! This module contains type definitions for unified tracking strategy.

use crate::core::{MemScopeError, MemScopeResult};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Runtime environment type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeEnvironment {
    /// Single-threaded environment
    SingleThreaded,
    /// Multi-threaded environment with thread count
    MultiThreaded { thread_count: usize },
    /// Async environment with runtime type
    AsyncRuntime { runtime_type: AsyncRuntimeType },
    /// Hybrid environment
    Hybrid {
        thread_count: usize,
        async_task_count: usize,
    },
}

/// Supported async runtime types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsyncRuntimeType {
    /// Tokio runtime
    Tokio,
    /// async-std runtime
    AsyncStd,
    /// Custom or unknown runtime
    Custom,
}

/// Tracking strategy type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Configuration for backend behavior.
#[derive(Debug, Clone)]
pub struct BackendConfig {
    /// Enable automatic environment detection
    pub auto_detect: bool,
    /// Forced strategy override
    pub force_strategy: Option<TrackingStrategy>,
    /// Performance sampling rate (0.0 to 1.0)
    pub sample_rate: f64,
    /// Maximum memory overhead percentage
    pub max_overhead_percent: f64,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            force_strategy: None,
            sample_rate: 1.0,
            max_overhead_percent: 5.0,
        }
    }
}

/// Environment detection configuration.
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
    /// Confidence level for detection results (0.0 to 1.0)
    pub confidence_level: f64,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            deep_async_detection: true,
            analysis_period_ms: 100,
            multi_thread_threshold: 2,
            max_detection_time_ms: 500,
            confidence_level: 1.0,
        }
    }
}

/// Environment detection result.
#[derive(Debug, Clone)]
pub struct EnvironmentDetection {
    /// Detected environment type
    pub environment: RuntimeEnvironment,
    /// Recommended tracking strategy
    pub recommended_strategy: TrackingStrategy,
    /// Number of detected threads
    pub thread_count: usize,
    /// Memory usage percentage
    pub memory_usage: f64,
    /// Detection confidence score
    pub confidence: f64,
}

/// Tracking session handle.
pub struct TrackingSession {
    /// Session identifier
    session_id: String,
    /// Backend reference
    backend: Arc<UnifiedBackend>,
    /// Session start timestamp
    start_time: std::time::Instant,
}

/// Comprehensive memory analysis data.
#[derive(Debug)]
pub struct MemoryAnalysisData {
    /// Raw tracking data
    pub raw_data: Vec<u8>,
    /// Aggregated statistics
    pub statistics: MemoryStatistics,
    /// Environment context
    pub environment: RuntimeEnvironment,
    /// Session metadata
    pub session_metadata: SessionMetadata,
}

/// Statistical summary of memory tracking session.
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

/// Session metadata for analysis context.
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

/// Unified tracking backend.
#[derive(Debug)]
pub struct UnifiedBackend {
    /// Current environment
    environment: RuntimeEnvironment,
    /// Active tracking strategy
    active_strategy: TrackingStrategy,
    /// Configuration
    config: BackendConfig,
}

impl Clone for UnifiedBackend {
    fn clone(&self) -> Self {
        Self {
            environment: self.environment.clone(),
            active_strategy: self.active_strategy,
            config: self.config.clone(),
        }
    }
}

impl UnifiedBackend {
    /// Initialize unified backend with configuration.
    pub fn initialize(config: BackendConfig) -> MemScopeResult<Self> {
        if config.sample_rate < 0.0 || config.sample_rate > 1.0 {
            return Err(MemScopeError::error(
                "unified_tracker",
                "initialize",
                "Sample rate must be between 0.0 and 1.0",
            ));
        }

        if config.max_overhead_percent < 0.0 || config.max_overhead_percent > 100.0 {
            return Err(MemScopeError::error(
                "unified_tracker",
                "initialize",
                "Max overhead percent must be between 0.0 and 100.0",
            ));
        }

        info!("Initializing unified backend");

        let environment = if config.auto_detect {
            Self::detect_environment()?
        } else {
            RuntimeEnvironment::SingleThreaded
        };

        let active_strategy = if let Some(forced) = config.force_strategy {
            warn!("Using forced strategy: {:?}", forced);
            forced
        } else {
            Self::select_strategy(&environment)?
        };

        info!("Selected tracking strategy: {:?}", active_strategy);

        Ok(Self {
            environment,
            active_strategy,
            config,
        })
    }

    /// Create a new unified backend with auto-detection.
    pub fn new() -> Self {
        Self::initialize(BackendConfig::default()).unwrap_or_else(|_| Self {
            environment: RuntimeEnvironment::SingleThreaded,
            active_strategy: TrackingStrategy::GlobalDirect,
            config: BackendConfig::default(),
        })
    }

    /// Create with specified configuration.
    pub fn with_config(config: BackendConfig) -> MemScopeResult<Self> {
        Self::initialize(config)
    }

    /// Get the current tracking strategy.
    pub fn strategy(&self) -> &TrackingStrategy {
        &self.active_strategy
    }

    /// Get the current environment.
    pub fn environment(&self) -> &RuntimeEnvironment {
        &self.environment
    }

    /// Get the configuration.
    pub fn config(&self) -> &BackendConfig {
        &self.config
    }

    /// Detect current runtime environment.
    pub fn detect_environment() -> MemScopeResult<RuntimeEnvironment> {
        debug!("Starting environment detection");

        let async_runtime = Self::detect_async_runtime();
        let thread_count = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1);

        let environment = match (async_runtime, thread_count) {
            (Some(runtime_type), 1) => RuntimeEnvironment::AsyncRuntime { runtime_type },
            (Some(_runtime_type), threads) => RuntimeEnvironment::Hybrid {
                thread_count: threads,
                async_task_count: 0,
            },
            (None, 1) => RuntimeEnvironment::SingleThreaded,
            (None, threads) => RuntimeEnvironment::MultiThreaded {
                thread_count: threads,
            },
        };

        debug!("Environment detection completed: {:?}", environment);
        Ok(environment)
    }

    /// Detect presence and type of async runtime.
    fn detect_async_runtime() -> Option<AsyncRuntimeType> {
        if Self::is_tokio_present() {
            debug!("Tokio runtime detected");
            return Some(AsyncRuntimeType::Tokio);
        }

        if Self::is_async_std_present() {
            debug!("async-std runtime detected");
            return Some(AsyncRuntimeType::AsyncStd);
        }

        None
    }

    /// Check if Tokio runtime is active.
    /// Note: This is a heuristic check using environment variables.
    /// For more reliable detection, consider using `tokio::runtime::Handle::try_current()`
    /// when tokio is available as a dependency.
    fn is_tokio_present() -> bool {
        std::env::var("TOKIO_WORKER_THREADS").is_ok()
    }

    /// Check if async-std runtime is active.
    /// Note: This is a heuristic check using environment variables.
    fn is_async_std_present() -> bool {
        std::env::var("ASYNC_STD_THREAD_COUNT").is_ok()
    }

    /// Select optimal tracking strategy.
    fn select_strategy(environment: &RuntimeEnvironment) -> MemScopeResult<TrackingStrategy> {
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

    /// Start active memory tracking session.
    pub fn start_tracking(&mut self) -> MemScopeResult<TrackingSession> {
        let session_id = format!(
            "session_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| MemScopeError::error(
                    "unified_tracker",
                    "start_tracking",
                    format!("Failed to generate session ID: {}", e)
                ))?
                .as_millis()
        );

        info!("Starting tracking session: {}", session_id);

        let session = TrackingSession {
            session_id: session_id.clone(),
            backend: Arc::new(self.clone()),
            start_time: std::time::Instant::now(),
        };

        debug!("Tracking session {} started", session_id);
        Ok(session)
    }

    /// Collect all tracking data.
    pub fn collect_data(&self) -> MemScopeResult<MemoryAnalysisData> {
        debug!("Collecting tracking data");

        let statistics = MemoryStatistics {
            total_allocations: 0,
            peak_memory_bytes: 0,
            avg_allocation_size: 0.0,
            session_duration_ms: 0,
        };

        let session_metadata = SessionMetadata {
            session_id: "current_session".to_string(),
            detected_environment: self.environment.clone(),
            strategy_used: self.active_strategy,
            overhead_percent: self.config.max_overhead_percent,
        };

        Ok(MemoryAnalysisData {
            raw_data: vec![],
            statistics,
            environment: self.environment.clone(),
            session_metadata,
        })
    }

    /// Shutdown backend.
    pub fn shutdown(self) -> MemScopeResult<MemoryAnalysisData> {
        info!("Shutting down unified backend");
        self.collect_data()
    }
}

impl Default for UnifiedBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackingSession {
    /// Get session identifier.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get elapsed time since session start.
    pub fn elapsed_time(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Collect current tracking data.
    pub fn collect_data(&self) -> MemScopeResult<MemoryAnalysisData> {
        self.backend.collect_data()
    }

    /// End tracking session.
    pub fn end_session(self) -> MemScopeResult<MemoryAnalysisData> {
        info!("Ending tracking session: {}", self.session_id);
        self.backend.collect_data()
    }
}

/// Environment detector for runtime analysis.
#[derive(Debug)]
pub struct EnvironmentDetector {
    config: DetectionConfig,
}

impl EnvironmentDetector {
    /// Create new environment detector.
    pub fn new(config: DetectionConfig) -> Self {
        Self { config }
    }

    /// Perform environment detection.
    pub fn detect(&self) -> MemScopeResult<EnvironmentDetection> {
        let environment = UnifiedBackend::detect_environment()?;
        let recommended_strategy = UnifiedBackend::select_strategy(&environment)?;

        let thread_count = match &environment {
            RuntimeEnvironment::SingleThreaded => 1,
            RuntimeEnvironment::MultiThreaded { thread_count } => *thread_count,
            RuntimeEnvironment::AsyncRuntime { .. } => 1,
            RuntimeEnvironment::Hybrid { thread_count, .. } => *thread_count,
        };

        let confidence = self.config.confidence_level;

        Ok(EnvironmentDetection {
            environment,
            recommended_strategy,
            thread_count,
            memory_usage: 0.0,
            confidence,
        })
    }

    /// Get the detection configuration.
    pub fn config(&self) -> &DetectionConfig {
        &self.config
    }
}

impl Default for EnvironmentDetector {
    fn default() -> Self {
        Self::new(DetectionConfig::default())
    }
}

/// Quick initialization function.
pub fn initialize() -> MemScopeResult<UnifiedBackend> {
    UnifiedBackend::initialize(BackendConfig::default())
}

/// Get a backend with default configuration.
pub fn get_backend() -> UnifiedBackend {
    UnifiedBackend::new()
}

/// Detect environment with default configuration.
pub fn detect_environment() -> MemScopeResult<RuntimeEnvironment> {
    UnifiedBackend::detect_environment()
}

/// Configuration for dispatcher behavior.
#[derive(Debug, Clone)]
pub struct DispatcherConfig {
    /// Enable automatic strategy switching
    pub auto_switch_strategies: bool,
    /// Maximum number of concurrent trackers
    pub max_concurrent_trackers: usize,
    /// Performance monitoring interval (milliseconds)
    pub metrics_interval_ms: u64,
    /// Memory usage threshold for strategy switching (MB)
    pub memory_threshold_mb: usize,
}

impl Default for DispatcherConfig {
    fn default() -> Self {
        Self {
            auto_switch_strategies: true,
            max_concurrent_trackers: 4,
            metrics_interval_ms: 1000,
            memory_threshold_mb: 100,
        }
    }
}

/// Performance metrics for dispatcher operations.
#[derive(Debug, Clone, Default)]
pub struct DispatcherMetrics {
    /// Total tracking operations dispatched
    pub total_dispatches: u64,
    /// Strategy switch count
    pub strategy_switches: u64,
    /// Average dispatch latency (microseconds)
    pub avg_dispatch_latency_us: f64,
    /// Current memory overhead percentage
    pub memory_overhead_percent: f64,
    /// Active tracker count
    pub active_trackers: usize,
}

/// Tracking operations that can be dispatched.
#[derive(Debug, Clone)]
pub enum TrackingOperation {
    /// Start tracking operation
    StartTracking,
    /// Stop tracking operation
    StopTracking,
    /// Collect current data
    CollectData,
}

/// Tracker type identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Configuration for individual tracker instances.
#[derive(Debug, Clone)]
pub struct TrackerConfig {
    /// Sampling rate (0.0 to 1.0)
    pub sample_rate: f64,
    /// Maximum memory overhead allowed (MB)
    pub max_overhead_mb: usize,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            sample_rate: 1.0,
            max_overhead_mb: 50,
        }
    }
}

/// Statistics from individual tracker.
#[derive(Debug, Clone, Default)]
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

/// Errors specific to tracker operations.
#[derive(Debug)]
pub enum TrackerError {
    /// Tracker initialization failed
    InitializationFailed { reason: String },
    /// Tracking start failed
    StartFailed { reason: String },
    /// Data collection failed
    DataCollectionFailed { reason: String },
    /// Tracker configuration invalid
    InvalidConfiguration { reason: String },
}

impl std::fmt::Display for TrackerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackerError::InitializationFailed { reason } => {
                write!(f, "Tracker initialization failed: {}", reason)
            }
            TrackerError::StartFailed { reason } => {
                write!(f, "Failed to start tracking: {}", reason)
            }
            TrackerError::DataCollectionFailed { reason } => {
                write!(f, "Failed to collect tracking data: {}", reason)
            }
            TrackerError::InvalidConfiguration { reason } => {
                write!(f, "Invalid tracker configuration: {}", reason)
            }
        }
    }
}

impl std::error::Error for TrackerError {}

/// Unified memory tracker interface.
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

/// Central dispatcher for tracking operations.
///
/// **Note**: This struct is currently unused but reserved for future versions
/// where it will handle automatic strategy selection based on runtime environment.
pub struct TrackingDispatcher {
    active_strategy: Option<TrackingStrategy>,
    config: DispatcherConfig,
    metrics: DispatcherMetrics,
}

impl TrackingDispatcher {
    /// Create new tracking dispatcher.
    pub fn new(config: DispatcherConfig) -> Self {
        Self {
            active_strategy: None,
            config,
            metrics: DispatcherMetrics::default(),
        }
    }

    /// Select strategy for environment.
    pub fn select_strategy(&mut self, environment: &RuntimeEnvironment) -> TrackingStrategy {
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

        self.active_strategy = Some(strategy);
        strategy
    }

    /// Get active strategy.
    pub fn active_strategy(&self) -> Option<&TrackingStrategy> {
        self.active_strategy.as_ref()
    }

    /// Get current metrics.
    pub fn metrics(&self) -> &DispatcherMetrics {
        &self.metrics
    }

    /// Get configuration.
    pub fn config(&self) -> &DispatcherConfig {
        &self.config
    }
}

impl Default for TrackingDispatcher {
    fn default() -> Self {
        Self::new(DispatcherConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_backend_creation() {
        let backend = UnifiedBackend::new();
        assert!(matches!(
            backend.environment(),
            RuntimeEnvironment::SingleThreaded | RuntimeEnvironment::MultiThreaded { .. }
        ));
    }

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
            sample_rate: 1.5,
            ..Default::default()
        };
        let result = UnifiedBackend::initialize(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_strategy_selection() {
        let env = RuntimeEnvironment::SingleThreaded;
        let strategy = UnifiedBackend::select_strategy(&env);
        assert!(matches!(strategy, Ok(TrackingStrategy::GlobalDirect)));
    }

    #[test]
    fn test_runtime_environment_variants() {
        let single = RuntimeEnvironment::SingleThreaded;
        let multi = RuntimeEnvironment::MultiThreaded { thread_count: 4 };
        let async_env = RuntimeEnvironment::AsyncRuntime {
            runtime_type: AsyncRuntimeType::Tokio,
        };
        let hybrid = RuntimeEnvironment::Hybrid {
            thread_count: 2,
            async_task_count: 4,
        };

        assert_ne!(single, multi);
        assert_ne!(multi, async_env);
        assert_ne!(async_env, hybrid);
    }

    #[test]
    fn test_tracking_strategy_variants() {
        let global = TrackingStrategy::GlobalDirect;
        let thread_local = TrackingStrategy::ThreadLocal;
        let task_local = TrackingStrategy::TaskLocal;
        let hybrid = TrackingStrategy::HybridTracking;

        assert_ne!(global, thread_local);
        assert_ne!(thread_local, task_local);
        assert_ne!(task_local, hybrid);
    }

    #[test]
    fn test_tracking_session() {
        let mut backend = UnifiedBackend::new();
        let session = backend.start_tracking();
        assert!(session.is_ok());

        let session = session.unwrap();
        assert!(!session.session_id().is_empty());
        let _elapsed = session.elapsed_time();
    }

    #[test]
    fn test_environment_detector() {
        let detector = EnvironmentDetector::default();
        let result = detector.detect();
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_function() {
        let result = initialize();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_backend_function() {
        let backend = get_backend();
        assert!(matches!(
            backend.environment(),
            RuntimeEnvironment::SingleThreaded | RuntimeEnvironment::MultiThreaded { .. }
        ));
    }
}
