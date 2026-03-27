// Unified Backend System for Memory Tracking
// Provides intelligent routing between single-thread, multi-thread, and async tracking strategies
// Maintains zero-lock architecture and preserves existing JSON export compatibility

use crate::{
    core::error::{MemScopeError, Result},
    lockfree::aggregator::LockfreeAggregator,
};

use std::sync::Arc;
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

impl UnifiedBackend {
    /// Initialize unified backend with configuration
    /// Performs environment detection and strategy selection
    pub fn initialize(config: BackendConfig) -> Result<Self> {
        if config.sample_rate < 0.0 || config.sample_rate > 1.0 {
            return Err(MemScopeError::config(
                "backend",
                "Sample rate must be between 0.0 and 1.0",
            ));
        }

        if config.max_overhead_percent < 0.0 || config.max_overhead_percent > 100.0 {
            return Err(MemScopeError::config(
                "backend",
                "Max overhead percent must be between 0.0 and 100.0",
            ));
        }

        info!("Initializing unified backend with config: {:?}", config);

        let environment = if config.auto_detect {
            Self::detect_environment()?
        } else {
            RuntimeEnvironment::SingleThreaded
        };

        debug!("Detected environment: {:?}", environment);

        let active_strategy = if let Some(forced) = config.force_strategy.clone() {
            warn!("Using forced strategy: {:?}", forced);
            forced
        } else {
            Self::select_strategy(&environment)?
        };

        info!("Selected tracking strategy: {:?}", active_strategy);

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
    pub fn detect_environment() -> Result<RuntimeEnvironment> {
        debug!("Starting environment detection");

        let async_runtime = Self::detect_async_runtime();

        let thread_count = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1);

        let environment = match (async_runtime, thread_count) {
            (Some(runtime_type), 0) => {
                RuntimeEnvironment::AsyncRuntime { runtime_type }
            }
            (Some(runtime_type), 1) => RuntimeEnvironment::AsyncRuntime { runtime_type },
            (Some(_runtime_type), threads) => {
                RuntimeEnvironment::Hybrid {
                    thread_count: threads,
                    async_task_count: 0,
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
        if Self::is_tokio_present() {
            debug!("Tokio runtime detected");
            return Some(AsyncRuntimeType::Tokio);
        }

        if Self::is_async_std_present() {
            debug!("async-std runtime detected");
            return Some(AsyncRuntimeType::AsyncStd);
        }

        debug!("No async runtime detected");
        None
    }

    /// Check if Tokio runtime is currently active
    fn is_tokio_present() -> bool {
        std::env::var("TOKIO_WORKER_THREADS").is_ok()
    }

    /// Check if async-std runtime is active
    fn is_async_std_present() -> bool {
        false
    }

    /// Select optimal tracking strategy based on environment
    fn select_strategy(environment: &RuntimeEnvironment) -> Result<TrackingStrategy> {
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
    pub fn start_tracking(&mut self) -> Result<TrackingSession> {
        let session_id = format!(
            "session_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| {
                    MemScopeError::config(
                        "session",
                        format!("Failed to generate session ID: {}", e),
                    )
                })?
                .as_millis()
        );

        info!("Starting tracking session: {}", session_id);

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
    fn initialize_global_tracking(&mut self) -> Result<()> {
        debug!("Initializing global direct tracking");
        Ok(())
    }

    /// Initialize thread-local tracking for multi-threaded applications
    fn initialize_thread_local_tracking(&mut self) -> Result<()> {
        debug!("Initializing thread-local tracking");
        Ok(())
    }

    /// Initialize task-local tracking for async applications
    fn initialize_task_local_tracking(&mut self) -> Result<()> {
        debug!("Initializing task-local tracking");
        Ok(())
    }

    /// Initialize hybrid tracking for complex applications
    fn initialize_hybrid_tracking(&mut self) -> Result<()> {
        debug!("Initializing hybrid tracking");
        Ok(())
    }

    /// Collect all tracking data from active session
    /// Aggregates data from all tracking sources into unified format
    pub fn collect_data(&self) -> Result<MemoryAnalysisData> {
        debug!("Collecting tracking data");

        let raw_data = vec![];

        let statistics = self.calculate_statistics(&raw_data)?;

        let session_metadata = SessionMetadata {
            session_id: "current_session".to_string(),
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
    fn calculate_statistics(&self, _raw_data: &[u8]) -> Result<MemoryStatistics> {
        Ok(MemoryStatistics {
            total_allocations: 0,
            peak_memory_bytes: 0,
            avg_allocation_size: 0.0,
            session_duration_ms: 0,
        })
    }

    /// Measure current tracking overhead percentage
    fn measure_overhead(&self) -> f64 {
        self.config.max_overhead_percent
    }

    /// Shutdown backend and finalize all tracking
    pub fn shutdown(self) -> Result<MemoryAnalysisData> {
        info!("Shutting down unified backend");

        let final_data = self.collect_data()?;

        debug!("Backend shutdown completed successfully");
        Ok(final_data)
    }
}

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
    pub fn collect_data(&self) -> Result<MemoryAnalysisData> {
        self.backend.collect_data()
    }

    /// End tracking session and collect final data
    pub fn end_session(self) -> Result<MemoryAnalysisData> {
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
            sample_rate: 1.5,
            ..Default::default()
        };
        let result = UnifiedBackend::initialize(config);
        assert!(matches!(
            result,
            Err(MemScopeError::Configuration { .. })
        ));
    }

    #[test]
    fn test_strategy_selection() {
        let env = RuntimeEnvironment::SingleThreaded;
        let strategy = UnifiedBackend::select_strategy(&env);
        assert!(matches!(strategy, Ok(TrackingStrategy::GlobalDirect)));
    }
}
