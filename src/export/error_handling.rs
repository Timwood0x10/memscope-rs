//! enhanced error handling and logging system
//!
//! this module provides specialized error handling, logging, and recovery mechanisms for export systems,
//! ensuring detailed error information and appropriate recovery strategies in various。

use crate::core::types::{TrackingError, TrackingResult};
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// export system error type
#[derive(Debug, Clone)]
pub enum ExportError {
    /// parallel processing error
    ParallelProcessingError {
        /// shard index
        shard_index: usize,
        /// thread id
        thread_id: String,
        /// error message
        error_message: String,
        /// partial results
        partial_results: Option<Vec<u8>>,
    },
    /// resource limit exceeded error
    ResourceLimitExceeded {
        /// resource type
        resource_type: ResourceType,
        /// limit
        limit: u64,
        /// actual
        actual: u64,
        /// suggested action
        suggested_action: String,
    },
    /// data quality error
    DataQualityError {
        /// validation type
        validation_type: ValidationType,
        /// expected value
        expected: String,
        /// actual value
        actual: String,
        /// affected records
        affected_records: usize,
    },
    /// performance threshold exceeded error
    PerformanceThresholdExceeded {
        /// metric
        metric: PerformanceMetric,
        /// threshold
        threshold: f64,
        /// actual
        actual: f64,
        /// stage
        stage: ExportStage,
    },
    /// concurrency conflict error
    ConcurrencyConflict {
        /// operation
        operation: String,
        /// conflict type
        conflict_type: ConflictType,
        /// retry count
        retry_count: usize,
    },
    /// data corruption error
    DataCorruption {
        /// corruption type
        corruption_type: CorruptionType,
        /// affected data
        affected_data: String,
        /// recovery possible
        recovery_possible: bool,
    },
    /// insufficient resources error
    InsufficientResources {
        /// required memory
        required_memory: usize,
        /// available memory
        available_memory: usize,
        /// required disk
        required_disk: usize,
        /// available disk
        available_disk: usize,
    },
    /// export interrupted error
    ExportInterrupted {
        /// export stage
        stage: ExportStage,
        /// progress percentage
        progress_percentage: f64,
        /// partial output path
        partial_output_path: Option<String>,
    },
}

/// resource type enum
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceType {
    /// memory
    Memory,
    /// disk
    Disk,
    /// cpu
    CPU,
    /// file handles
    FileHandles,
    /// thread pool
    ThreadPool,
}

/// validation type enum
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValidationType {
    /// json structure
    JsonStructure,
    /// data integrity
    DataIntegrity,
    /// allocation count
    AllocationCount,
    /// file size
    FileSize,
    /// encoding
    Encoding,
}

/// performance metric enum
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceMetric {
    /// export time
    ExportTime,
    /// memory usage
    MemoryUsage,
    /// throughput rate
    ThroughputRate,
    /// error rate
    ErrorRate,
    /// response time
    ResponseTime,
}

/// export stage enum
#[derive(Debug, Clone, PartialEq)]
pub enum ExportStage {
    /// initialization
    Initialization,
    /// data localization
    DataLocalization,
    /// parallel processing
    ParallelProcessing,
    /// writing
    Writing,
    /// validation
    Validation,
    /// finalization
    Finalization,
}

/// concurrency conflict type
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    /// lock contention
    LockContention,
    /// data race
    DataRace,
    /// resource contention
    ResourceContention,
    /// thread pool exhaustion
    ThreadPoolExhaustion,
}

/// corruption type enum
#[derive(Debug, Clone, PartialEq)]
pub enum CorruptionType {
    /// incomplete data
    IncompleteData,
    /// invalid format
    InvalidFormat,
    /// checksum mismatch
    ChecksumMismatch,
    /// structural damage
    StructuralDamage,
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportError::ParallelProcessingError {
                shard_index,
                thread_id,
                error_message,
                ..
            } => {
                write!(f, "parallel processing error - shard {shard_index} (thread {thread_id}): {error_message}")
            }
            ExportError::ResourceLimitExceeded {
                resource_type,
                limit,
                actual,
                suggested_action,
            } => {
                write!(f, "resource limit exceeded - {resource_type:?}: limit {limit}, actual {actual}. suggested action: {suggested_action}")
            }
            ExportError::DataQualityError {
                validation_type,
                expected,
                actual,
                affected_records,
            } => {
                write!(f, "data quality error - {validation_type:?}: expected {expected}, actual {actual}, affected records {affected_records}")
            }
            ExportError::PerformanceThresholdExceeded {
                metric,
                threshold,
                actual,
                stage,
            } => {
                write!(f, "performance threshold exceeded - {metric:?} in {stage:?}: threshold {threshold}, actual {actual}")
            }
            ExportError::ConcurrencyConflict {
                operation,
                conflict_type,
                retry_count,
            } => {
                write!(f, "concurrency conflict - operation {operation}, type {conflict_type:?}, retry count {retry_count}")
            }
            ExportError::DataCorruption {
                corruption_type,
                affected_data,
                recovery_possible,
            } => {
                write!(f, "data corruption - type {corruption_type:?}, affected data {affected_data}, recovery possible: {recovery_possible}")
            }
            ExportError::InsufficientResources {
                required_memory,
                available_memory,
                required_disk,
                available_disk,
            } => {
                write!(f, "insufficient resources - required memory {required_memory}MB, available {available_memory}MB, required disk {required_disk}MB, available {available_disk}MB")
            }
            ExportError::ExportInterrupted {
                stage,
                progress_percentage,
                partial_output_path,
            } => {
                write!(f, "export interrupted - stage {stage:?}, progress {progress_percentage:.1}%, partial output: {partial_output_path:?}")
            }
        }
    }
}

impl std::error::Error for ExportError {}

/// Validation-specific error types (separate from export errors)
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// File access error during validation
    FileAccessError {
        /// File path that couldn't be accessed
        file_path: String,
        /// Underlying error message
        error: String,
    },
    /// JSON parsing error during validation
    JsonParsingError {
        /// File path with invalid JSON
        file_path: String,
        /// JSON error details
        error: String,
    },
    /// Validation timeout error
    TimeoutError {
        /// File path that timed out
        file_path: String,
        /// Timeout duration
        timeout_duration: std::time::Duration,
    },
    /// Validation was cancelled
    CancelledError {
        /// File path being validated
        file_path: String,
        /// Cancellation reason
        reason: String,
    },
    /// Configuration error
    ConfigurationError {
        /// Configuration error details
        error: String,
    },
    /// Internal validation error
    InternalError {
        /// Internal error details
        error: String,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::FileAccessError { file_path, error } => {
                write!(f, "file access error for {file_path}: {error}")
            }
            ValidationError::JsonParsingError { file_path, error } => {
                write!(f, "JSON parsing error in {file_path}: {error}")
            }
            ValidationError::TimeoutError {
                file_path,
                timeout_duration,
            } => {
                write!(
                    f,
                    "validation timeout for {file_path} after {timeout_duration:?}",
                )
            }
            ValidationError::CancelledError { file_path, reason } => {
                write!(f, "validation cancelled for {file_path}: {reason}")
            }
            ValidationError::ConfigurationError { error } => {
                write!(f, "validation configuration error: {error}")
            }
            ValidationError::InternalError { error } => {
                write!(f, "internal validation error: {error}")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

impl From<ValidationError> for TrackingError {
    fn from(error: ValidationError) -> Self {
        TrackingError::ExportError(error.to_string())
    }
}

impl From<ExportError> for TrackingError {
    fn from(error: ExportError) -> Self {
        TrackingError::ExportError(error.to_string())
    }
}

/// performance logger
#[derive(Debug)]
pub struct PerformanceLogger {
    /// log level
    log_level: LogLevel,
    /// performance metrics collector
    metrics_collector: Arc<MetricsCollector>,
    /// error statistics
    error_stats: Arc<ErrorStatistics>,
    /// start time
    start_time: Instant,
}

/// log level
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    /// error
    Error,
    /// warn
    Warn,
    /// info
    Info,
    /// debug
    Debug,
    /// trace
    Trace,
}

/// metrics collector
#[derive(Debug)]
pub struct MetricsCollector {
    /// total operations
    total_operations: AtomicUsize,
    /// successful operations
    successful_operations: AtomicUsize,
    /// failed operations
    failed_operations: AtomicUsize,
    /// total processing time (milliseconds)
    total_processing_time_ms: AtomicUsize,
    /// peak memory usage (bytes)
    peak_memory_usage: AtomicUsize,
    /// current memory usage (bytes)
    current_memory_usage: AtomicUsize,
}

/// error statistics
#[derive(Debug)]
pub struct ErrorStatistics {
    /// parallel processing errors
    parallel_processing_errors: AtomicUsize,
    /// resource limit errors
    resource_limit_errors: AtomicUsize,
    /// data quality errors
    data_quality_errors: AtomicUsize,
    /// performance threshold errors
    performance_threshold_errors: AtomicUsize,
    /// concurrency conflict errors
    concurrency_conflict_errors: AtomicUsize,
    /// data corruption errors
    data_corruption_errors: AtomicUsize,
    /// insufficient resources errors
    insufficient_resources_errors: AtomicUsize,
    /// export interrupted errors
    export_interrupted_errors: AtomicUsize,
}

impl PerformanceLogger {
    /// Create new performance logger
    pub fn new(log_level: LogLevel) -> Self {
        Self {
            log_level,
            metrics_collector: Arc::new(MetricsCollector::new()),
            error_stats: Arc::new(ErrorStatistics::new()),
            start_time: Instant::now(),
        }
    }

    /// record operation start
    pub fn log_operation_start(&self, operation: &str, details: &str) {
        if self.should_log(LogLevel::Info) {
            tracing::info!(
                "🚀 [{}] start operation: {} - {}",
                self.format_timestamp(),
                operation,
                details
            );
        }
        self.metrics_collector
            .total_operations
            .fetch_add(1, Ordering::Relaxed);
    }

    /// record operation success
    pub fn log_operation_success(&self, operation: &str, duration: Duration, details: &str) {
        if self.should_log(LogLevel::Info) {
            tracing::info!(
                "✅ [{}] operation success: {} ({:?}) - {}",
                self.format_timestamp(),
                operation,
                duration,
                details
            );
        }
        self.metrics_collector
            .successful_operations
            .fetch_add(1, Ordering::Relaxed);
        self.metrics_collector
            .total_processing_time_ms
            .fetch_add(duration.as_millis() as usize, Ordering::Relaxed);
    }

    /// record operation failure
    pub fn log_operation_failure(&self, operation: &str, error: &ExportError, duration: Duration) {
        if self.should_log(LogLevel::Error) {
            tracing::error!(
                "❌ [{}] operation failure: {} ({:?}) - {}",
                self.format_timestamp(),
                operation,
                duration,
                error
            );
        }
        self.metrics_collector
            .failed_operations
            .fetch_add(1, Ordering::Relaxed);
        self.update_error_statistics(error);
    }

    /// record performance metric
    pub fn log_performance_metric(
        &self,
        metric: PerformanceMetric,
        value: f64,
        threshold: Option<f64>,
    ) {
        if self.should_log(LogLevel::Debug) {
            let threshold_info = if let Some(t) = threshold {
                format!(" (threshold: {t})")
            } else {
                String::new()
            };
            tracing::debug!(
                "📊 [{}] performance metric - {metric:?}: {value}{threshold_info}",
                self.format_timestamp()
            );
        }

        // check if exceeded threshold
        if let Some(threshold) = threshold {
            if value > threshold {
                let error = ExportError::PerformanceThresholdExceeded {
                    metric,
                    threshold,
                    actual: value,
                    stage: ExportStage::ParallelProcessing, // default stage
                };
                self.log_warning(&format!("performance threshold exceeded: {error}"));
                self.update_error_statistics(&error);
            }
        }
    }

    /// record memory usage
    pub fn log_memory_usage(&self, current_usage: usize, peak_usage: usize) {
        if self.should_log(LogLevel::Debug) {
            tracing::debug!(
                "💾 [{}] memory usage - current: {:.2}MB, peak: {:.2}MB",
                self.format_timestamp(),
                current_usage as f64 / 1024.0 / 1024.0,
                peak_usage as f64 / 1024.0 / 1024.0
            );
        }

        self.metrics_collector
            .current_memory_usage
            .store(current_usage, Ordering::Relaxed);

        // update peak memory usage
        let current_peak = self
            .metrics_collector
            .peak_memory_usage
            .load(Ordering::Relaxed);
        if peak_usage > current_peak {
            self.metrics_collector
                .peak_memory_usage
                .store(peak_usage, Ordering::Relaxed);
        }
    }

    /// record warning
    pub fn log_warning(&self, message: &str) {
        if self.should_log(LogLevel::Warn) {
            tracing::warn!("⚠️ [{}] warning: {}", self.format_timestamp(), message);
        }
    }

    /// record debug
    pub fn log_debug(&self, message: &str) {
        if self.should_log(LogLevel::Debug) {
            tracing::debug!("🔍 [{}] debug: {}", self.format_timestamp(), message);
        }
    }

    /// record error
    pub fn log_error(&self, error: &ExportError) {
        if self.should_log(LogLevel::Error) {
            tracing::error!("💥 [{}] error: {}", self.format_timestamp(), error);
        }
        self.update_error_statistics(error);
    }

    /// generate performance report
    pub fn generate_performance_report(&self) -> PerformanceReport {
        let total_time = self.start_time.elapsed();
        let total_ops = self
            .metrics_collector
            .total_operations
            .load(Ordering::Relaxed);
        let successful_ops = self
            .metrics_collector
            .successful_operations
            .load(Ordering::Relaxed);
        let failed_ops = self
            .metrics_collector
            .failed_operations
            .load(Ordering::Relaxed);
        let total_processing_time = self
            .metrics_collector
            .total_processing_time_ms
            .load(Ordering::Relaxed);
        let peak_memory = self
            .metrics_collector
            .peak_memory_usage
            .load(Ordering::Relaxed);
        let current_memory = self
            .metrics_collector
            .current_memory_usage
            .load(Ordering::Relaxed);

        let success_rate = if total_ops > 0 {
            (successful_ops as f64 / total_ops as f64) * 100.0
        } else {
            0.0
        };

        let avg_processing_time = if successful_ops > 0 {
            total_processing_time as f64 / successful_ops as f64
        } else {
            0.0
        };

        PerformanceReport {
            total_runtime: total_time,
            total_operations: total_ops,
            successful_operations: successful_ops,
            failed_operations: failed_ops,
            success_rate,
            average_processing_time_ms: avg_processing_time,
            peak_memory_usage_mb: peak_memory as f64 / 1024.0 / 1024.0,
            current_memory_usage_mb: current_memory as f64 / 1024.0 / 1024.0,
            error_breakdown: self.get_error_breakdown(),
        }
    }

    /// check if should log
    fn should_log(&self, level: LogLevel) -> bool {
        matches!(
            (&self.log_level, &level),
            (LogLevel::Error, LogLevel::Error)
                | (LogLevel::Warn, LogLevel::Error | LogLevel::Warn)
                | (
                    LogLevel::Info,
                    LogLevel::Error | LogLevel::Warn | LogLevel::Info
                )
                | (
                    LogLevel::Debug,
                    LogLevel::Error | LogLevel::Warn | LogLevel::Info | LogLevel::Debug
                )
                | (
                    LogLevel::Trace,
                    LogLevel::Error
                        | LogLevel::Warn
                        | LogLevel::Info
                        | LogLevel::Debug
                        | LogLevel::Trace
                )
        )
    }

    /// format timestamp
    fn format_timestamp(&self) -> String {
        let elapsed = self.start_time.elapsed();
        format!("{:>8.3}s", elapsed.as_secs_f64())
    }

    /// update error statistics
    fn update_error_statistics(&self, error: &ExportError) {
        match error {
            ExportError::ParallelProcessingError { .. } => {
                self.error_stats
                    .parallel_processing_errors
                    .fetch_add(1, Ordering::Relaxed);
            }
            ExportError::ResourceLimitExceeded { .. } => {
                self.error_stats
                    .resource_limit_errors
                    .fetch_add(1, Ordering::Relaxed);
            }
            ExportError::DataQualityError { .. } => {
                self.error_stats
                    .data_quality_errors
                    .fetch_add(1, Ordering::Relaxed);
            }
            ExportError::PerformanceThresholdExceeded { .. } => {
                self.error_stats
                    .performance_threshold_errors
                    .fetch_add(1, Ordering::Relaxed);
            }
            ExportError::ConcurrencyConflict { .. } => {
                self.error_stats
                    .concurrency_conflict_errors
                    .fetch_add(1, Ordering::Relaxed);
            }
            ExportError::DataCorruption { .. } => {
                self.error_stats
                    .data_corruption_errors
                    .fetch_add(1, Ordering::Relaxed);
            }
            ExportError::InsufficientResources { .. } => {
                self.error_stats
                    .insufficient_resources_errors
                    .fetch_add(1, Ordering::Relaxed);
            }
            ExportError::ExportInterrupted { .. } => {
                self.error_stats
                    .export_interrupted_errors
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// get error breakdown
    fn get_error_breakdown(&self) -> ErrorBreakdown {
        ErrorBreakdown {
            parallel_processing_errors: self
                .error_stats
                .parallel_processing_errors
                .load(Ordering::Relaxed),
            resource_limit_errors: self
                .error_stats
                .resource_limit_errors
                .load(Ordering::Relaxed),
            data_quality_errors: self.error_stats.data_quality_errors.load(Ordering::Relaxed),
            performance_threshold_errors: self
                .error_stats
                .performance_threshold_errors
                .load(Ordering::Relaxed),
            concurrency_conflict_errors: self
                .error_stats
                .concurrency_conflict_errors
                .load(Ordering::Relaxed),
            data_corruption_errors: self
                .error_stats
                .data_corruption_errors
                .load(Ordering::Relaxed),
            insufficient_resources_errors: self
                .error_stats
                .insufficient_resources_errors
                .load(Ordering::Relaxed),
            export_interrupted_errors: self
                .error_stats
                .export_interrupted_errors
                .load(Ordering::Relaxed),
        }
    }
}

impl MetricsCollector {
    fn new() -> Self {
        Self {
            total_operations: AtomicUsize::new(0),
            successful_operations: AtomicUsize::new(0),
            failed_operations: AtomicUsize::new(0),
            total_processing_time_ms: AtomicUsize::new(0),
            peak_memory_usage: AtomicUsize::new(0),
            current_memory_usage: AtomicUsize::new(0),
        }
    }
}

impl ErrorStatistics {
    fn new() -> Self {
        Self {
            parallel_processing_errors: AtomicUsize::new(0),
            resource_limit_errors: AtomicUsize::new(0),
            data_quality_errors: AtomicUsize::new(0),
            performance_threshold_errors: AtomicUsize::new(0),
            concurrency_conflict_errors: AtomicUsize::new(0),
            data_corruption_errors: AtomicUsize::new(0),
            insufficient_resources_errors: AtomicUsize::new(0),
            export_interrupted_errors: AtomicUsize::new(0),
        }
    }
}

/// performance report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// total runtime
    pub total_runtime: Duration,
    /// total operations
    pub total_operations: usize,
    /// successful operations
    pub successful_operations: usize,
    /// failed operations
    pub failed_operations: usize,
    /// success rate
    pub success_rate: f64,
    /// average processing time (ms)
    pub average_processing_time_ms: f64,
    /// peak memory usage (MB)
    pub peak_memory_usage_mb: f64,
    /// current memory usage (MB)
    pub current_memory_usage_mb: f64,
    /// error breakdown
    pub error_breakdown: ErrorBreakdown,
}

/// error breakdown
#[derive(Debug, Clone)]
pub struct ErrorBreakdown {
    /// parallel processing errors
    pub parallel_processing_errors: usize,
    /// resource limit errors
    pub resource_limit_errors: usize,
    /// data quality errors
    pub data_quality_errors: usize,
    /// performance threshold errors
    pub performance_threshold_errors: usize,
    /// concurrency conflict errors
    pub concurrency_conflict_errors: usize,
    /// data corruption errors
    pub data_corruption_errors: usize,
    /// insufficient resources errors
    pub insufficient_resources_errors: usize,
    /// export interrupted errors
    pub export_interrupted_errors: usize,
}

impl PerformanceReport {
    /// print detailed report
    pub fn print_detailed_report(&self) {
        tracing::info!("\n📈 detailed performance report");
        tracing::info!("================");

        tracing::info!("⏱️ runtime: {:?}", self.total_runtime);
        tracing::info!("🔢 total operations: {}", self.total_operations);
        tracing::info!(
            "✅ successful operations: {} ({:.1}%)",
            self.successful_operations,
            self.success_rate
        );
        tracing::info!("❌ failed operations: {}", self.failed_operations);
        tracing::info!(
            "⚡ average processing time: {:.2}ms",
            self.average_processing_time_ms
        );
        tracing::info!("💾 peak memory usage: {:.2}MB", self.peak_memory_usage_mb);
        tracing::info!(
            "💾 current memory usage: {:.2}MB",
            self.current_memory_usage_mb
        );

        tracing::info!("\n🚨 error breakdown:");
        tracing::info!(
            "   parallel processing errors: {}",
            self.error_breakdown.parallel_processing_errors
        );
        tracing::info!(
            "   resource limit errors: {}",
            self.error_breakdown.resource_limit_errors
        );
        tracing::info!(
            "   data quality errors: {}",
            self.error_breakdown.data_quality_errors
        );
        tracing::info!(
            "   performance threshold errors: {}",
            self.error_breakdown.performance_threshold_errors
        );
        tracing::info!(
            "   concurrency conflict errors: {}",
            self.error_breakdown.concurrency_conflict_errors
        );
        tracing::info!(
            "   data corruption errors: {}",
            self.error_breakdown.data_corruption_errors
        );
        tracing::info!(
            "   insufficient resources errors: {}",
            self.error_breakdown.insufficient_resources_errors
        );
        tracing::info!(
            "   export interrupted errors: {}",
            self.error_breakdown.export_interrupted_errors
        );
    }
}

/// resource monitor
#[derive(Debug)]
pub struct ResourceMonitor {
    /// memory limit (bytes)
    memory_limit: usize,
    /// disk space limit (bytes)
    disk_limit: usize,
    /// CPU usage limit (percentage)
    cpu_limit: f64,
}

impl ResourceMonitor {
    /// create new resource monitor
    pub fn new(memory_limit_mb: usize, disk_limit_mb: usize, cpu_limit_percent: f64) -> Self {
        Self {
            memory_limit: memory_limit_mb * 1024 * 1024,
            disk_limit: disk_limit_mb * 1024 * 1024,
            cpu_limit: cpu_limit_percent,
        }
    }

    /// check resource usage
    pub fn check_resource_usage(&self) -> TrackingResult<ResourceUsage> {
        let memory_usage = self.get_memory_usage()?;
        let disk_usage = self.get_disk_usage()?;
        let cpu_usage = self.get_cpu_usage()?;

        // check if exceeded limits
        if memory_usage > self.memory_limit {
            return Err(ExportError::ResourceLimitExceeded {
                resource_type: ResourceType::Memory,
                limit: self.memory_limit as u64,
                actual: memory_usage as u64,
                suggested_action: "reduce parallelism or enable streaming processing".to_string(),
            }
            .into());
        }

        if disk_usage > self.disk_limit {
            return Err(ExportError::ResourceLimitExceeded {
                resource_type: ResourceType::Disk,
                limit: self.disk_limit as u64,
                actual: disk_usage as u64,
                suggested_action: "clean up temporary files or select other output location"
                    .to_string(),
            }
            .into());
        }

        if cpu_usage > self.cpu_limit {
            return Err(ExportError::ResourceLimitExceeded {
                resource_type: ResourceType::CPU,
                limit: (self.cpu_limit * 100.0) as u64,
                actual: (cpu_usage * 100.0) as u64,
                suggested_action: "reduce thread count or lower processing priority".to_string(),
            }
            .into());
        }

        Ok(ResourceUsage {
            memory_usage,
            disk_usage,
            cpu_usage,
            memory_limit: self.memory_limit,
            disk_limit: self.disk_limit,
            cpu_limit: self.cpu_limit,
        })
    }

    /// get memory usage (simplified implementation)
    fn get_memory_usage(&self) -> TrackingResult<usize> {
        // in actual implementation, this should call system API to get real memory usage
        // here use simplified implementation
        Ok(0) // placeholder implementation
    }

    /// get disk usage (simplified implementation)
    fn get_disk_usage(&self) -> TrackingResult<usize> {
        // in actual implementation, this should call system API to get real disk usage
        Ok(0) // placeholder implementation
    }

    /// get CPU usage (simplified implementation)
    fn get_cpu_usage(&self) -> TrackingResult<f64> {
        // in actual implementation, this should call system API to get real CPU usage
        Ok(0.0) // placeholder implementation
    }
}

/// resource usage
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// Current memory usage in bytes
    pub memory_usage: usize,
    /// Current disk usage in bytes
    pub disk_usage: usize,
    /// Current CPU usage as percentage (0.0-100.0)
    pub cpu_usage: f64,
    /// Memory limit in bytes
    pub memory_limit: usize,
    /// Disk limit in bytes
    pub disk_limit: usize,
    /// CPU limit as percentage (0.0-100.0)
    pub cpu_limit: f64,
}

impl ResourceUsage {
    /// get memory usage percentage
    pub fn memory_usage_percentage(&self) -> f64 {
        if self.memory_limit > 0 {
            (self.memory_usage as f64 / self.memory_limit as f64) * 100.0
        } else {
            0.0
        }
    }

    /// get disk usage percentage
    pub fn disk_usage_percentage(&self) -> f64 {
        if self.disk_limit > 0 {
            (self.disk_usage as f64 / self.disk_limit as f64) * 100.0
        } else {
            0.0
        }
    }

    /// get CPU usage percentage
    pub fn cpu_usage_percentage(&self) -> f64 {
        self.cpu_usage * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_error_display() {
        let error = ExportError::ParallelProcessingError {
            shard_index: 5,
            thread_id: "thread-1".to_string(),
            error_message: "serialization failed".to_string(),
            partial_results: None,
        };

        let display = format!("{error}");
        assert!(display.contains("parallel processing error"));
        assert!(display.contains("shard 5"));
        assert!(display.contains("thread-1"));
    }

    #[test]
    fn test_performance_logger() {
        let logger = PerformanceLogger::new(LogLevel::Info);

        logger.log_operation_start("test operation", "test details");
        logger.log_operation_success("test operation", Duration::from_millis(100), "success");

        let report = logger.generate_performance_report();
        assert_eq!(report.total_operations, 1);
        assert_eq!(report.successful_operations, 1);
        assert_eq!(report.failed_operations, 0);
        assert_eq!(report.success_rate, 100.0);
    }

    #[test]
    fn test_resource_monitor() {
        let monitor = ResourceMonitor::new(1024, 2048, 80.0);

        // test resource check (using simplified implementation, should always succeed)
        let result = monitor.check_resource_usage();
        assert!(result.is_ok());

        let usage = result.expect("Failed to get memory usage");
        assert_eq!(usage.memory_limit, 1024 * 1024 * 1024);
        assert_eq!(usage.disk_limit, 2048 * 1024 * 1024);
        assert_eq!(usage.cpu_limit, 80.0);
    }

    #[test]
    fn test_resource_usage_percentages() {
        let usage = ResourceUsage {
            memory_usage: 512 * 1024 * 1024,  // 512MB
            disk_usage: 1024 * 1024 * 1024,   // 1GB
            cpu_usage: 0.6,                   // 60%
            memory_limit: 1024 * 1024 * 1024, // 1GB
            disk_limit: 2048 * 1024 * 1024,   // 2GB
            cpu_limit: 0.8,                   // 80%
        };

        assert_eq!(usage.memory_usage_percentage(), 50.0);
        assert_eq!(usage.disk_usage_percentage(), 50.0);
        assert_eq!(usage.cpu_usage_percentage(), 60.0);
    }

    #[test]
    fn test_all_export_error_variants_display() {
        // Test ResourceLimitExceeded
        let resource_error = ExportError::ResourceLimitExceeded {
            resource_type: ResourceType::Memory,
            limit: 1024,
            actual: 2048,
            suggested_action: "reduce memory usage".to_string(),
        };
        let display = format!("{resource_error}");
        assert!(display.contains("resource limit exceeded"));
        assert!(display.contains("Memory"));
        assert!(display.contains("reduce memory usage"));

        // Test DataQualityError
        let quality_error = ExportError::DataQualityError {
            validation_type: ValidationType::JsonStructure,
            expected: "valid JSON".to_string(),
            actual: "malformed JSON".to_string(),
            affected_records: 100,
        };
        let display = format!("{quality_error}");
        assert!(display.contains("data quality error"));
        assert!(display.contains("JsonStructure"));
        assert!(display.contains("affected records 100"));

        // Test PerformanceThresholdExceeded
        let perf_error = ExportError::PerformanceThresholdExceeded {
            metric: PerformanceMetric::ExportTime,
            threshold: 10.0,
            actual: 15.5,
            stage: ExportStage::Writing,
        };
        let display = format!("{perf_error}");
        assert!(display.contains("performance threshold exceeded"));
        assert!(display.contains("ExportTime"));
        assert!(display.contains("Writing"));

        // Test ConcurrencyConflict
        let concurrency_error = ExportError::ConcurrencyConflict {
            operation: "write_data".to_string(),
            conflict_type: ConflictType::LockContention,
            retry_count: 3,
        };
        let display = format!("{concurrency_error}");
        assert!(display.contains("concurrency conflict"));
        assert!(display.contains("write_data"));
        assert!(display.contains("LockContention"));

        // Test DataCorruption
        let corruption_error = ExportError::DataCorruption {
            corruption_type: CorruptionType::ChecksumMismatch,
            affected_data: "allocation_data.json".to_string(),
            recovery_possible: true,
        };
        let display = format!("{corruption_error}");
        assert!(display.contains("data corruption"));
        assert!(display.contains("ChecksumMismatch"));
        assert!(display.contains("recovery possible: true"));

        // Test InsufficientResources
        let insufficient_error = ExportError::InsufficientResources {
            required_memory: 2048,
            available_memory: 1024,
            required_disk: 4096,
            available_disk: 2048,
        };
        let display = format!("{insufficient_error}");
        assert!(display.contains("insufficient resources"));
        assert!(display.contains("required memory 2048MB"));
        assert!(display.contains("available 1024MB"));

        // Test ExportInterrupted
        let interrupted_error = ExportError::ExportInterrupted {
            stage: ExportStage::ParallelProcessing,
            progress_percentage: 75.5,
            partial_output_path: Some("temp_output.json".to_string()),
        };
        let display = format!("{interrupted_error}");
        assert!(display.contains("export interrupted"));
        assert!(display.contains("ParallelProcessing"));
        assert!(display.contains("progress 75.5%"));
    }

    #[test]
    fn test_validation_error_variants_display() {
        // Test FileAccessError
        let file_error = ValidationError::FileAccessError {
            file_path: "/path/to/file.json".to_string(),
            error: "permission denied".to_string(),
        };
        let display = format!("{file_error}");
        assert!(display.contains("file access error"));
        assert!(display.contains("/path/to/file.json"));
        assert!(display.contains("permission denied"));

        // Test JsonParsingError
        let json_error = ValidationError::JsonParsingError {
            file_path: "data.json".to_string(),
            error: "unexpected token".to_string(),
        };
        let display = format!("{json_error}");
        assert!(display.contains("JSON parsing error"));
        assert!(display.contains("data.json"));
        assert!(display.contains("unexpected token"));

        // Test TimeoutError
        let timeout_error = ValidationError::TimeoutError {
            file_path: "large_file.json".to_string(),
            timeout_duration: Duration::from_secs(30),
        };
        let display = format!("{timeout_error}");
        assert!(display.contains("validation timeout"));
        assert!(display.contains("large_file.json"));
        assert!(display.contains("30s"));

        // Test CancelledError
        let cancelled_error = ValidationError::CancelledError {
            file_path: "cancelled_file.json".to_string(),
            reason: "user requested cancellation".to_string(),
        };
        let display = format!("{cancelled_error}");
        assert!(display.contains("validation cancelled"));
        assert!(display.contains("cancelled_file.json"));
        assert!(display.contains("user requested cancellation"));

        // Test ConfigurationError
        let config_error = ValidationError::ConfigurationError {
            error: "invalid timeout value".to_string(),
        };
        let display = format!("{config_error}");
        assert!(display.contains("validation configuration error"));
        assert!(display.contains("invalid timeout value"));

        // Test InternalError
        let internal_error = ValidationError::InternalError {
            error: "unexpected internal state".to_string(),
        };
        let display = format!("{internal_error}");
        assert!(display.contains("internal validation error"));
        assert!(display.contains("unexpected internal state"));
    }

    #[test]
    fn test_error_conversion_to_tracking_error() {
        // Test ValidationError to TrackingError conversion
        let validation_error = ValidationError::FileAccessError {
            file_path: "test.json".to_string(),
            error: "file not found".to_string(),
        };
        let tracking_error: TrackingError = validation_error.into();
        match tracking_error {
            TrackingError::ExportError(msg) => {
                assert!(msg.contains("file access error"));
                assert!(msg.contains("test.json"));
            }
            _ => panic!("Expected ExportError variant"),
        }

        // Test ExportError to TrackingError conversion
        let export_error = ExportError::DataQualityError {
            validation_type: ValidationType::DataIntegrity,
            expected: "valid data".to_string(),
            actual: "corrupted data".to_string(),
            affected_records: 50,
        };
        let tracking_error: TrackingError = export_error.into();
        match tracking_error {
            TrackingError::ExportError(msg) => {
                assert!(msg.contains("data quality error"));
                assert!(msg.contains("DataIntegrity"));
            }
            _ => panic!("Expected ExportError variant"),
        }
    }

    #[test]
    fn test_enum_variants_equality() {
        // Test ResourceType equality
        assert_eq!(ResourceType::Memory, ResourceType::Memory);
        assert_ne!(ResourceType::Memory, ResourceType::Disk);

        // Test ValidationType equality and hash
        use std::collections::HashMap;
        let mut validation_map = HashMap::new();
        validation_map.insert(ValidationType::JsonStructure, 1);
        validation_map.insert(ValidationType::DataIntegrity, 2);
        assert_eq!(validation_map.get(&ValidationType::JsonStructure), Some(&1));
        assert_eq!(validation_map.get(&ValidationType::DataIntegrity), Some(&2));

        // Test PerformanceMetric equality
        assert_eq!(PerformanceMetric::ExportTime, PerformanceMetric::ExportTime);
        assert_ne!(
            PerformanceMetric::ExportTime,
            PerformanceMetric::MemoryUsage
        );

        // Test ExportStage equality
        assert_eq!(ExportStage::Initialization, ExportStage::Initialization);
        assert_ne!(ExportStage::Initialization, ExportStage::Writing);

        // Test ConflictType equality
        assert_eq!(ConflictType::LockContention, ConflictType::LockContention);
        assert_ne!(ConflictType::LockContention, ConflictType::DataRace);

        // Test CorruptionType equality
        assert_eq!(
            CorruptionType::IncompleteData,
            CorruptionType::IncompleteData
        );
        assert_ne!(
            CorruptionType::IncompleteData,
            CorruptionType::InvalidFormat
        );
    }

    #[test]
    fn test_performance_logger_different_log_levels() {
        // Test Error level logging
        let error_logger = PerformanceLogger::new(LogLevel::Error);
        assert!(error_logger.should_log(LogLevel::Error));
        assert!(!error_logger.should_log(LogLevel::Warn));
        assert!(!error_logger.should_log(LogLevel::Info));

        // Test Warn level logging
        let warn_logger = PerformanceLogger::new(LogLevel::Warn);
        assert!(warn_logger.should_log(LogLevel::Error));
        assert!(warn_logger.should_log(LogLevel::Warn));
        assert!(!warn_logger.should_log(LogLevel::Info));

        // Test Info level logging
        let info_logger = PerformanceLogger::new(LogLevel::Info);
        assert!(info_logger.should_log(LogLevel::Error));
        assert!(info_logger.should_log(LogLevel::Warn));
        assert!(info_logger.should_log(LogLevel::Info));
        assert!(!info_logger.should_log(LogLevel::Debug));

        // Test Debug level logging
        let debug_logger = PerformanceLogger::new(LogLevel::Debug);
        assert!(debug_logger.should_log(LogLevel::Error));
        assert!(debug_logger.should_log(LogLevel::Warn));
        assert!(debug_logger.should_log(LogLevel::Info));
        assert!(debug_logger.should_log(LogLevel::Debug));
        assert!(!debug_logger.should_log(LogLevel::Trace));

        // Test Trace level logging
        let trace_logger = PerformanceLogger::new(LogLevel::Trace);
        assert!(trace_logger.should_log(LogLevel::Error));
        assert!(trace_logger.should_log(LogLevel::Warn));
        assert!(trace_logger.should_log(LogLevel::Info));
        assert!(trace_logger.should_log(LogLevel::Debug));
        assert!(trace_logger.should_log(LogLevel::Trace));
    }

    #[test]
    fn test_performance_logger_error_statistics() {
        let logger = PerformanceLogger::new(LogLevel::Debug);

        // Test different error types
        let parallel_error = ExportError::ParallelProcessingError {
            shard_index: 1,
            thread_id: "thread-1".to_string(),
            error_message: "test error".to_string(),
            partial_results: None,
        };
        logger.log_operation_failure("test_op", &parallel_error, Duration::from_millis(50));

        let resource_error = ExportError::ResourceLimitExceeded {
            resource_type: ResourceType::Memory,
            limit: 1000,
            actual: 1500,
            suggested_action: "reduce usage".to_string(),
        };
        logger.log_operation_failure("test_op2", &resource_error, Duration::from_millis(25));

        let report = logger.generate_performance_report();
        assert_eq!(report.failed_operations, 2);
        assert_eq!(report.error_breakdown.parallel_processing_errors, 1);
        assert_eq!(report.error_breakdown.resource_limit_errors, 1);
        assert_eq!(report.error_breakdown.data_quality_errors, 0);
    }

    #[test]
    fn test_performance_logger_memory_tracking() {
        let logger = PerformanceLogger::new(LogLevel::Debug);

        // Test memory usage logging
        logger.log_memory_usage(1024 * 1024, 2048 * 1024); // 1MB current, 2MB peak
        logger.log_memory_usage(1536 * 1024, 2048 * 1024); // 1.5MB current, 2MB peak
        logger.log_memory_usage(512 * 1024, 3072 * 1024); // 0.5MB current, 3MB peak (new peak)

        let report = logger.generate_performance_report();
        assert_eq!(report.current_memory_usage_mb, 0.5);
        assert_eq!(report.peak_memory_usage_mb, 3.0);
    }

    #[test]
    fn test_performance_logger_metric_threshold_checking() {
        let logger = PerformanceLogger::new(LogLevel::Debug);

        // Test metric without threshold
        logger.log_performance_metric(PerformanceMetric::ExportTime, 5.0, None);

        // Test metric within threshold
        logger.log_performance_metric(PerformanceMetric::MemoryUsage, 80.0, Some(100.0));

        // Test metric exceeding threshold
        logger.log_performance_metric(PerformanceMetric::ThroughputRate, 150.0, Some(100.0));

        let report = logger.generate_performance_report();
        // Should have one performance threshold error from the last metric
        assert_eq!(report.error_breakdown.performance_threshold_errors, 1);
    }

    #[test]
    fn test_performance_logger_comprehensive_operations() {
        let logger = PerformanceLogger::new(LogLevel::Info);

        // Test multiple operations
        for i in 0..5 {
            logger.log_operation_start(&format!("operation_{}", i), "test details");
            if i < 3 {
                logger.log_operation_success(
                    &format!("operation_{}", i),
                    Duration::from_millis(100 + i as u64 * 10),
                    "success",
                );
            } else {
                let error = ExportError::DataQualityError {
                    validation_type: ValidationType::AllocationCount,
                    expected: "1000".to_string(),
                    actual: "999".to_string(),
                    affected_records: 1,
                };
                logger.log_operation_failure(
                    &format!("operation_{}", i),
                    &error,
                    Duration::from_millis(50),
                );
            }
        }

        let report = logger.generate_performance_report();
        assert_eq!(report.total_operations, 5);
        assert_eq!(report.successful_operations, 3);
        assert_eq!(report.failed_operations, 2);
        assert_eq!(report.success_rate, 60.0);
        assert!(report.average_processing_time_ms > 0.0);
        assert_eq!(report.error_breakdown.data_quality_errors, 2);
    }

    #[test]
    fn test_performance_logger_timestamp_formatting() {
        let logger = PerformanceLogger::new(LogLevel::Debug);

        // Test that timestamp formatting works
        let timestamp = logger.format_timestamp();
        assert!(timestamp.contains("s"));
        assert!(!timestamp.is_empty());
    }

    #[test]
    fn test_performance_report_detailed_output() {
        let logger = PerformanceLogger::new(LogLevel::Info);

        // Add some test data
        logger.log_operation_start("test", "details");
        logger.log_operation_success("test", Duration::from_millis(100), "success");
        logger.log_memory_usage(1024 * 1024, 2048 * 1024);

        let report = logger.generate_performance_report();

        // Test that the report can be printed without panicking
        report.print_detailed_report();

        // Verify report structure
        assert!(report.total_runtime.as_nanos() > 0);
        assert_eq!(report.total_operations, 1);
        assert_eq!(report.successful_operations, 1);
        assert_eq!(report.success_rate, 100.0);
    }

    #[test]
    fn test_resource_usage_edge_cases() {
        // Test zero limits
        let usage_zero_limits = ResourceUsage {
            memory_usage: 100,
            disk_usage: 200,
            cpu_usage: 0.5,
            memory_limit: 0,
            disk_limit: 0,
            cpu_limit: 0.0,
        };
        assert_eq!(usage_zero_limits.memory_usage_percentage(), 0.0);
        assert_eq!(usage_zero_limits.disk_usage_percentage(), 0.0);
        assert_eq!(usage_zero_limits.cpu_usage_percentage(), 50.0);

        // Test maximum usage
        let usage_max = ResourceUsage {
            memory_usage: 1024,
            disk_usage: 2048,
            cpu_usage: 1.0,
            memory_limit: 1024,
            disk_limit: 2048,
            cpu_limit: 1.0,
        };
        assert_eq!(usage_max.memory_usage_percentage(), 100.0);
        assert_eq!(usage_max.disk_usage_percentage(), 100.0);
        assert_eq!(usage_max.cpu_usage_percentage(), 100.0);

        // Test over-usage
        let usage_over = ResourceUsage {
            memory_usage: 2048,
            disk_usage: 4096,
            cpu_usage: 1.5,
            memory_limit: 1024,
            disk_limit: 2048,
            cpu_limit: 1.0,
        };
        assert_eq!(usage_over.memory_usage_percentage(), 200.0);
        assert_eq!(usage_over.disk_usage_percentage(), 200.0);
        assert_eq!(usage_over.cpu_usage_percentage(), 150.0);
    }

    #[test]
    fn test_resource_monitor_creation() {
        let monitor = ResourceMonitor::new(512, 1024, 75.0);

        // Test that limits are correctly converted to bytes
        assert_eq!(monitor.memory_limit, 512 * 1024 * 1024);
        assert_eq!(monitor.disk_limit, 1024 * 1024 * 1024);
        assert_eq!(monitor.cpu_limit, 75.0);
    }

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();

        // Test initial values
        assert_eq!(collector.total_operations.load(Ordering::Relaxed), 0);
        assert_eq!(collector.successful_operations.load(Ordering::Relaxed), 0);
        assert_eq!(collector.failed_operations.load(Ordering::Relaxed), 0);
        assert_eq!(
            collector.total_processing_time_ms.load(Ordering::Relaxed),
            0
        );
        assert_eq!(collector.peak_memory_usage.load(Ordering::Relaxed), 0);
        assert_eq!(collector.current_memory_usage.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_error_statistics_creation() {
        let stats = ErrorStatistics::new();

        // Test initial values
        assert_eq!(stats.parallel_processing_errors.load(Ordering::Relaxed), 0);
        assert_eq!(stats.resource_limit_errors.load(Ordering::Relaxed), 0);
        assert_eq!(stats.data_quality_errors.load(Ordering::Relaxed), 0);
        assert_eq!(
            stats.performance_threshold_errors.load(Ordering::Relaxed),
            0
        );
        assert_eq!(stats.concurrency_conflict_errors.load(Ordering::Relaxed), 0);
        assert_eq!(stats.data_corruption_errors.load(Ordering::Relaxed), 0);
        assert_eq!(
            stats.insufficient_resources_errors.load(Ordering::Relaxed),
            0
        );
        assert_eq!(stats.export_interrupted_errors.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_all_error_types_statistics_update() {
        let logger = PerformanceLogger::new(LogLevel::Error);

        // Test all error types to ensure statistics are updated correctly
        let errors = vec![
            ExportError::ParallelProcessingError {
                shard_index: 1,
                thread_id: "thread-1".to_string(),
                error_message: "test".to_string(),
                partial_results: None,
            },
            ExportError::ResourceLimitExceeded {
                resource_type: ResourceType::Memory,
                limit: 1000,
                actual: 1500,
                suggested_action: "reduce".to_string(),
            },
            ExportError::DataQualityError {
                validation_type: ValidationType::JsonStructure,
                expected: "valid".to_string(),
                actual: "invalid".to_string(),
                affected_records: 1,
            },
            ExportError::PerformanceThresholdExceeded {
                metric: PerformanceMetric::ExportTime,
                threshold: 10.0,
                actual: 15.0,
                stage: ExportStage::Writing,
            },
            ExportError::ConcurrencyConflict {
                operation: "test".to_string(),
                conflict_type: ConflictType::LockContention,
                retry_count: 1,
            },
            ExportError::DataCorruption {
                corruption_type: CorruptionType::ChecksumMismatch,
                affected_data: "test.json".to_string(),
                recovery_possible: true,
            },
            ExportError::InsufficientResources {
                required_memory: 2048,
                available_memory: 1024,
                required_disk: 4096,
                available_disk: 2048,
            },
            ExportError::ExportInterrupted {
                stage: ExportStage::Finalization,
                progress_percentage: 90.0,
                partial_output_path: None,
            },
        ];

        for (i, error) in errors.iter().enumerate() {
            logger.log_operation_failure(&format!("op_{}", i), error, Duration::from_millis(10));
        }

        let report = logger.generate_performance_report();
        let breakdown = &report.error_breakdown;

        assert_eq!(breakdown.parallel_processing_errors, 1);
        assert_eq!(breakdown.resource_limit_errors, 1);
        assert_eq!(breakdown.data_quality_errors, 1);
        assert_eq!(breakdown.performance_threshold_errors, 1);
        assert_eq!(breakdown.concurrency_conflict_errors, 1);
        assert_eq!(breakdown.data_corruption_errors, 1);
        assert_eq!(breakdown.insufficient_resources_errors, 1);
        assert_eq!(breakdown.export_interrupted_errors, 1);
        assert_eq!(report.failed_operations, 8);
    }

    #[test]
    fn test_log_level_equality() {
        assert_eq!(LogLevel::Error, LogLevel::Error);
        assert_ne!(LogLevel::Error, LogLevel::Warn);
        assert_eq!(LogLevel::Info, LogLevel::Info);
        assert_ne!(LogLevel::Debug, LogLevel::Trace);
    }

    #[test]
    fn test_export_error_with_partial_results() {
        let error = ExportError::ParallelProcessingError {
            shard_index: 2,
            thread_id: "thread-2".to_string(),
            error_message: "partial failure".to_string(),
            partial_results: Some(vec![1, 2, 3, 4, 5]),
        };

        // Test that error can be created with partial results
        match error {
            ExportError::ParallelProcessingError {
                partial_results, ..
            } => {
                assert!(partial_results.is_some());
                assert_eq!(partial_results.unwrap(), vec![1, 2, 3, 4, 5]);
            }
            _ => panic!("Expected ParallelProcessingError"),
        }
    }

    #[test]
    fn test_performance_logger_warning_and_debug_logging() {
        let logger = PerformanceLogger::new(LogLevel::Debug);

        // Test warning logging
        logger.log_warning("This is a test warning");

        // Test debug logging
        logger.log_debug("This is a test debug message");

        // Test error logging
        let error = ExportError::DataCorruption {
            corruption_type: CorruptionType::StructuralDamage,
            affected_data: "critical_data.json".to_string(),
            recovery_possible: false,
        };
        logger.log_error(&error);

        let report = logger.generate_performance_report();
        assert_eq!(report.error_breakdown.data_corruption_errors, 1);
    }
}
