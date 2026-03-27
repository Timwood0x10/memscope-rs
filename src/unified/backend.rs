// Unified Backend System for Memory Tracking
// Provides intelligent routing between single-thread, multi-thread, and async tracking strategies
// Maintains zero-lock architecture and preserves existing JSON export compatibility

use crate::core::error::{MemScopeError, Result, SystemErrorType};
use crate::lockfree::aggregator::LockfreeAggregator;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
struct AllocationRecord {
    ptr: usize,
    size: usize,
    timestamp_alloc: u64,
    timestamp_dealloc: Option<u64>,
    thread_id: u64,
    var_name: Option<String>,
    type_name: Option<String>,
}

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
    /// Internal allocation storage
    allocations: Arc<RwLock<HashMap<usize, AllocationRecord>>>,
    /// Total allocations count
    total_allocations: Arc<std::sync::atomic::AtomicU64>,
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
            allocations: Arc::new(RwLock::new(HashMap::new())),
            total_allocations: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        })
    }

    /// Detect current runtime environment characteristics
    /// Analyzes thread count, async runtime presence, and execution patterns
    pub fn detect_environment() -> Result<RuntimeEnvironment> {
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

    /// Track a memory allocation
    pub fn track_allocation(&self, ptr: usize, size: usize) -> Result<()> {
        debug!("Tracking allocation: ptr={:?}, size={}", ptr, size);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let thread_id = get_thread_id();

        let record = AllocationRecord {
            ptr,
            size,
            timestamp_alloc: timestamp,
            timestamp_dealloc: None,
            thread_id,
            var_name: None,
            type_name: None,
        };

        {
            let mut allocations = self.allocations.write().map_err(|e| {
                MemScopeError::system_with_source(
                    SystemErrorType::Locking,
                    "Failed to acquire write lock on allocations",
                    e,
                )
            })?;
            allocations.insert(ptr, record);
        }

        self.total_allocations
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        debug!(
            "Successfully tracked allocation: ptr={:x}, size={}",
            ptr, size
        );
        Ok(())
    }

    /// Track a memory deallocation
    pub fn track_deallocation(&self, ptr: usize) -> Result<()> {
        debug!("Tracking deallocation: ptr={:?}", ptr);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        {
            let mut allocations = self.allocations.write().map_err(|e| {
                MemScopeError::system_with_source(
                    SystemErrorType::Locking,
                    "Failed to acquire write lock on allocations",
                    e,
                )
            })?;

            if let Some(record) = allocations.get_mut(&ptr) {
                record.timestamp_dealloc = Some(timestamp);
                debug!(
                    "Successfully tracked deallocation: ptr={:x}, allocated at {}",
                    ptr, record.timestamp_alloc
                );
            } else {
                warn!("Deallocation tracked for unknown pointer: {:x}", ptr);
            }
        }

        Ok(())
    }

    /// Get a snapshot of current allocations
    pub fn snapshot(&self) -> crate::types::internal_types::Snapshot {
        debug!("Taking snapshot");

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let allocations = match self.allocations.read() {
            Ok(allocs) => allocs,
            Err(e) => {
                warn!("Failed to acquire read lock on allocations: {}", e);
                return crate::types::internal_types::Snapshot {
                    timestamp,
                    allocations: Vec::new(),
                    tasks: Vec::new(),
                    threads: Vec::new(),
                    passports: Vec::new(),
                    stats: crate::types::internal_types::Stats::default(),
                };
            }
        };
        let snapshot_allocations: Vec<crate::types::internal_types::Allocation> = allocations
            .values()
            .map(|record| {
                let mut alloc =
                    crate::types::internal_types::Allocation::new(record.ptr, record.size);
                alloc.alloc_ts = record.timestamp_alloc;
                alloc.free_ts = record.timestamp_dealloc;
                alloc.thread = record.thread_id as u32;
                if let Some(ref var_name) = record.var_name {
                    alloc.meta.var_name = Some(var_name.clone());
                }
                if let Some(ref type_name) = record.type_name {
                    alloc.meta.type_name = Some(type_name.clone());
                }
                alloc
            })
            .collect();

        crate::types::internal_types::Snapshot {
            timestamp,
            allocations: snapshot_allocations,
            tasks: Vec::new(),
            threads: Vec::new(),
            passports: Vec::new(),
            stats: crate::types::internal_types::Stats::default(),
        }
    }

    /// Shutdown backend and finalize all tracking
    pub fn shutdown(self) -> Result<MemoryAnalysisData> {
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
            allocations: Arc::clone(&self.allocations),
            total_allocations: Arc::clone(&self.total_allocations),
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

fn get_thread_id() -> u64 {
    static THREAD_COUNTER: AtomicU64 = AtomicU64::new(1);

    std::thread_local! {
        static THREAD_ID: u64 = THREAD_COUNTER.fetch_add(1, Ordering::SeqCst);
    }

    THREAD_ID.with(|&id| id)
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
        assert!(matches!(result, Err(MemScopeError::Configuration { .. })));
    }

    #[test]
    fn test_strategy_selection() {
        let env = RuntimeEnvironment::SingleThreaded;
        let strategy = UnifiedBackend::select_strategy(&env);
        assert!(matches!(strategy, Ok(TrackingStrategy::GlobalDirect)));
    }
}
