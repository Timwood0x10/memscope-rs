//! 增强的错误处理和日志记录系统
//!
//! 这个模块提供了专门针对导出系统的错误处理、日志记录和恢复机制，
//! 确保在各种异常情况下都能提供详细的错误信息和适当的恢复策略。

use crate::core::types::{TrackingError, TrackingResult};
use std::fmt;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// 导出系统专用错误类型
#[derive(Debug, Clone)]
pub enum ExportError {
    /// 并行处理错误
    ParallelProcessingError {
        shard_index: usize,
        thread_id: String,
        error_message: String,
        partial_results: Option<Vec<u8>>,
    },
    /// 资源限制超出错误
    ResourceLimitExceeded {
        resource_type: ResourceType,
        limit: u64,
        actual: u64,
        suggested_action: String,
    },
    /// 数据质量验证错误
    DataQualityError {
        validation_type: ValidationType,
        expected: String,
        actual: String,
        affected_records: usize,
    },
    /// 性能阈值超出错误
    PerformanceThresholdExceeded {
        metric: PerformanceMetric,
        threshold: f64,
        actual: f64,
        stage: ExportStage,
    },
    /// 并发访问冲突错误
    ConcurrencyConflict {
        operation: String,
        conflict_type: ConflictType,
        retry_count: usize,
    },
    /// 数据损坏错误
    DataCorruption {
        corruption_type: CorruptionType,
        affected_data: String,
        recovery_possible: bool,
    },
    /// 系统资源不足错误
    InsufficientResources {
        required_memory: usize,
        available_memory: usize,
        required_disk: usize,
        available_disk: usize,
    },
    /// 导出中断错误
    ExportInterrupted {
        stage: ExportStage,
        progress_percentage: f64,
        partial_output_path: Option<String>,
    },
}

/// 资源类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceType {
    Memory,
    Disk,
    CPU,
    FileHandles,
    ThreadPool,
}

/// 验证类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValidationType {
    JsonStructure,
    DataIntegrity,
    AllocationCount,
    FileSize,
    Encoding,
}

/// 性能指标枚举
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceMetric {
    ExportTime,
    MemoryUsage,
    ThroughputRate,
    ErrorRate,
    ResponseTime,
}

/// 导出阶段枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ExportStage {
    Initialization,
    DataLocalization,
    ParallelProcessing,
    Writing,
    Validation,
    Finalization,
}

/// 并发冲突类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    LockContention,
    DataRace,
    ResourceContention,
    ThreadPoolExhaustion,
}

/// 数据损坏类型
#[derive(Debug, Clone, PartialEq)]
pub enum CorruptionType {
    IncompleteData,
    InvalidFormat,
    ChecksumMismatch,
    StructuralDamage,
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportError::ParallelProcessingError { shard_index, thread_id, error_message, .. } => {
                write!(f, "并行处理错误 - 分片 {shard_index} (线程 {thread_id}): {error_message}")
            }
            ExportError::ResourceLimitExceeded { resource_type, limit, actual, suggested_action } => {
                write!(f, "资源限制超出 - {resource_type:?}: 限制 {limit}, 实际 {actual}. 建议: {suggested_action}")
            }
            ExportError::DataQualityError { validation_type, expected, actual, affected_records } => {
                write!(f, "数据质量错误 - {validation_type:?}: 期望 {expected}, 实际 {actual}, 影响记录 {affected_records}")
            }
            ExportError::PerformanceThresholdExceeded { metric, threshold, actual, stage } => {
                write!(f, "性能阈值超出 - {metric:?} 在 {stage:?}: 阈值 {threshold}, 实际 {actual}")
            }
            ExportError::ConcurrencyConflict { operation, conflict_type, retry_count } => {
                write!(f, "并发冲突 - 操作 {operation}, 类型 {conflict_type:?}, 重试次数 {retry_count}")
            }
            ExportError::DataCorruption { corruption_type, affected_data, recovery_possible } => {
                write!(f, "数据损坏 - 类型 {corruption_type:?}, 受影响数据 {affected_data}, 可恢复: {recovery_possible}")
            }
            ExportError::InsufficientResources { required_memory, available_memory, required_disk, available_disk } => {
                write!(f, "资源不足 - 需要内存 {required_memory}MB, 可用 {available_memory}MB, 需要磁盘 {required_disk}MB, 可用 {available_disk}MB")
            }
            ExportError::ExportInterrupted { stage, progress_percentage, partial_output_path } => {
                write!(f, "导出中断 - 阶段 {stage:?}, 进度 {progress_percentage:.1}%, 部分输出: {partial_output_path:?}")
            }
        }
    }
}

impl std::error::Error for ExportError {}

impl From<ExportError> for TrackingError {
    fn from(error: ExportError) -> Self {
        TrackingError::ExportError(error.to_string())
    }
}

/// 性能日志记录器
#[derive(Debug)]
pub struct PerformanceLogger {
    /// 日志级别
    log_level: LogLevel,
    /// 性能指标收集器
    metrics_collector: Arc<MetricsCollector>,
    /// 错误统计
    error_stats: Arc<ErrorStatistics>,
    /// 开始时间
    start_time: Instant,
}

/// 日志级别
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// 指标收集器
#[derive(Debug)]
pub struct MetricsCollector {
    /// 总操作数
    total_operations: AtomicUsize,
    /// 成功操作数
    successful_operations: AtomicUsize,
    /// 失败操作数
    failed_operations: AtomicUsize,
    /// 总处理时间（毫秒）
    total_processing_time_ms: AtomicUsize,
    /// 峰值内存使用（字节）
    peak_memory_usage: AtomicUsize,
    /// 当前内存使用（字节）
    current_memory_usage: AtomicUsize,
}

/// 错误统计
#[derive(Debug)]
pub struct ErrorStatistics {
    /// 并行处理错误数
    parallel_processing_errors: AtomicUsize,
    /// 资源限制错误数
    resource_limit_errors: AtomicUsize,
    /// 数据质量错误数
    data_quality_errors: AtomicUsize,
    /// 性能阈值错误数
    performance_threshold_errors: AtomicUsize,
    /// 并发冲突错误数
    concurrency_conflict_errors: AtomicUsize,
    /// 数据损坏错误数
    data_corruption_errors: AtomicUsize,
    /// 资源不足错误数
    insufficient_resources_errors: AtomicUsize,
    /// 导出中断错误数
    export_interrupted_errors: AtomicUsize,
}

impl PerformanceLogger {
    /// 创建新的性能日志记录器
    pub fn new(log_level: LogLevel) -> Self {
        Self {
            log_level,
            metrics_collector: Arc::new(MetricsCollector::new()),
            error_stats: Arc::new(ErrorStatistics::new()),
            start_time: Instant::now(),
        }
    }

    /// 记录操作开始
    pub fn log_operation_start(&self, operation: &str, details: &str) {
        if self.should_log(LogLevel::Info) {
            println!("🚀 [{}] 开始操作: {} - {}", 
                    self.format_timestamp(), operation, details);
        }
        self.metrics_collector.total_operations.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录操作成功
    pub fn log_operation_success(&self, operation: &str, duration: Duration, details: &str) {
        if self.should_log(LogLevel::Info) {
            println!("✅ [{}] 操作成功: {} ({:?}) - {}", 
                    self.format_timestamp(), operation, duration, details);
        }
        self.metrics_collector.successful_operations.fetch_add(1, Ordering::Relaxed);
        self.metrics_collector.total_processing_time_ms.fetch_add(
            duration.as_millis() as usize, Ordering::Relaxed);
    }

    /// 记录操作失败
    pub fn log_operation_failure(&self, operation: &str, error: &ExportError, duration: Duration) {
        if self.should_log(LogLevel::Error) {
            println!("❌ [{}] 操作失败: {} ({:?}) - {}", 
                    self.format_timestamp(), operation, duration, error);
        }
        self.metrics_collector.failed_operations.fetch_add(1, Ordering::Relaxed);
        self.update_error_statistics(error);
    }

    /// 记录性能指标
    pub fn log_performance_metric(&self, metric: PerformanceMetric, value: f64, threshold: Option<f64>) {
        if self.should_log(LogLevel::Debug) {
            let threshold_info = if let Some(t) = threshold {
                format!(" (阈值: {t})")
            } else {
                String::new()
            };
            println!("📊 [{}] 性能指标 - {metric:?}: {value}{threshold_info}", 
                    self.format_timestamp());
        }

        // 检查是否超出阈值
        if let Some(threshold) = threshold {
            if value > threshold {
                let error = ExportError::PerformanceThresholdExceeded {
                    metric,
                    threshold,
                    actual: value,
                    stage: ExportStage::ParallelProcessing, // 默认阶段
                };
                self.log_warning(&format!("性能阈值超出: {error}"));
            }
        }
    }

    /// 记录内存使用情况
    pub fn log_memory_usage(&self, current_usage: usize, peak_usage: usize) {
        if self.should_log(LogLevel::Debug) {
            println!("💾 [{}] 内存使用 - 当前: {:.2}MB, 峰值: {:.2}MB", 
                    self.format_timestamp(),
                    current_usage as f64 / 1024.0 / 1024.0,
                    peak_usage as f64 / 1024.0 / 1024.0);
        }
        
        self.metrics_collector.current_memory_usage.store(current_usage, Ordering::Relaxed);
        
        // 更新峰值内存使用
        let current_peak = self.metrics_collector.peak_memory_usage.load(Ordering::Relaxed);
        if peak_usage > current_peak {
            self.metrics_collector.peak_memory_usage.store(peak_usage, Ordering::Relaxed);
        }
    }

    /// 记录警告信息
    pub fn log_warning(&self, message: &str) {
        if self.should_log(LogLevel::Warn) {
            println!("⚠️ [{}] 警告: {}", self.format_timestamp(), message);
        }
    }

    /// 记录调试信息
    pub fn log_debug(&self, message: &str) {
        if self.should_log(LogLevel::Debug) {
            println!("🔍 [{}] 调试: {}", self.format_timestamp(), message);
        }
    }

    /// 记录错误信息
    pub fn log_error(&self, error: &ExportError) {
        if self.should_log(LogLevel::Error) {
            println!("💥 [{}] 错误: {}", self.format_timestamp(), error);
        }
        self.update_error_statistics(error);
    }

    /// 生成性能报告
    pub fn generate_performance_report(&self) -> PerformanceReport {
        let total_time = self.start_time.elapsed();
        let total_ops = self.metrics_collector.total_operations.load(Ordering::Relaxed);
        let successful_ops = self.metrics_collector.successful_operations.load(Ordering::Relaxed);
        let failed_ops = self.metrics_collector.failed_operations.load(Ordering::Relaxed);
        let total_processing_time = self.metrics_collector.total_processing_time_ms.load(Ordering::Relaxed);
        let peak_memory = self.metrics_collector.peak_memory_usage.load(Ordering::Relaxed);
        let current_memory = self.metrics_collector.current_memory_usage.load(Ordering::Relaxed);

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

    /// 检查是否应该记录日志
    fn should_log(&self, level: LogLevel) -> bool {
        match (&self.log_level, &level) {
            (LogLevel::Error, LogLevel::Error) => true,
            (LogLevel::Warn, LogLevel::Error | LogLevel::Warn) => true,
            (LogLevel::Info, LogLevel::Error | LogLevel::Warn | LogLevel::Info) => true,
            (LogLevel::Debug, LogLevel::Error | LogLevel::Warn | LogLevel::Info | LogLevel::Debug) => true,
            (LogLevel::Trace, _) => true,
            _ => false,
        }
    }

    /// 格式化时间戳
    fn format_timestamp(&self) -> String {
        let elapsed = self.start_time.elapsed();
        format!("{:>8.3}s", elapsed.as_secs_f64())
    }

    /// 更新错误统计
    fn update_error_statistics(&self, error: &ExportError) {
        match error {
            ExportError::ParallelProcessingError { .. } => {
                self.error_stats.parallel_processing_errors.fetch_add(1, Ordering::Relaxed);
            }
            ExportError::ResourceLimitExceeded { .. } => {
                self.error_stats.resource_limit_errors.fetch_add(1, Ordering::Relaxed);
            }
            ExportError::DataQualityError { .. } => {
                self.error_stats.data_quality_errors.fetch_add(1, Ordering::Relaxed);
            }
            ExportError::PerformanceThresholdExceeded { .. } => {
                self.error_stats.performance_threshold_errors.fetch_add(1, Ordering::Relaxed);
            }
            ExportError::ConcurrencyConflict { .. } => {
                self.error_stats.concurrency_conflict_errors.fetch_add(1, Ordering::Relaxed);
            }
            ExportError::DataCorruption { .. } => {
                self.error_stats.data_corruption_errors.fetch_add(1, Ordering::Relaxed);
            }
            ExportError::InsufficientResources { .. } => {
                self.error_stats.insufficient_resources_errors.fetch_add(1, Ordering::Relaxed);
            }
            ExportError::ExportInterrupted { .. } => {
                self.error_stats.export_interrupted_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// 获取错误分类统计
    fn get_error_breakdown(&self) -> ErrorBreakdown {
        ErrorBreakdown {
            parallel_processing_errors: self.error_stats.parallel_processing_errors.load(Ordering::Relaxed),
            resource_limit_errors: self.error_stats.resource_limit_errors.load(Ordering::Relaxed),
            data_quality_errors: self.error_stats.data_quality_errors.load(Ordering::Relaxed),
            performance_threshold_errors: self.error_stats.performance_threshold_errors.load(Ordering::Relaxed),
            concurrency_conflict_errors: self.error_stats.concurrency_conflict_errors.load(Ordering::Relaxed),
            data_corruption_errors: self.error_stats.data_corruption_errors.load(Ordering::Relaxed),
            insufficient_resources_errors: self.error_stats.insufficient_resources_errors.load(Ordering::Relaxed),
            export_interrupted_errors: self.error_stats.export_interrupted_errors.load(Ordering::Relaxed),
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

/// 性能报告
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// 总运行时间
    pub total_runtime: Duration,
    /// 总操作数
    pub total_operations: usize,
    /// 成功操作数
    pub successful_operations: usize,
    /// 失败操作数
    pub failed_operations: usize,
    /// 成功率（百分比）
    pub success_rate: f64,
    /// 平均处理时间（毫秒）
    pub average_processing_time_ms: f64,
    /// 峰值内存使用（MB）
    pub peak_memory_usage_mb: f64,
    /// 当前内存使用（MB）
    pub current_memory_usage_mb: f64,
    /// 错误分类统计
    pub error_breakdown: ErrorBreakdown,
}

/// 错误分类统计
#[derive(Debug, Clone)]
pub struct ErrorBreakdown {
    pub parallel_processing_errors: usize,
    pub resource_limit_errors: usize,
    pub data_quality_errors: usize,
    pub performance_threshold_errors: usize,
    pub concurrency_conflict_errors: usize,
    pub data_corruption_errors: usize,
    pub insufficient_resources_errors: usize,
    pub export_interrupted_errors: usize,
}

impl PerformanceReport {
    /// 打印详细的性能报告
    pub fn print_detailed_report(&self) {
        println!("\n📈 详细性能报告");
        println!("================");
        
        println!("⏱️ 运行时间: {:?}", self.total_runtime);
        println!("🔢 总操作数: {}", self.total_operations);
        println!("✅ 成功操作: {} ({:.1}%)", self.successful_operations, self.success_rate);
        println!("❌ 失败操作: {}", self.failed_operations);
        println!("⚡ 平均处理时间: {:.2}ms", self.average_processing_time_ms);
        println!("💾 峰值内存使用: {:.2}MB", self.peak_memory_usage_mb);
        println!("💾 当前内存使用: {:.2}MB", self.current_memory_usage_mb);
        
        println!("\n🚨 错误分类统计:");
        println!("   并行处理错误: {}", self.error_breakdown.parallel_processing_errors);
        println!("   资源限制错误: {}", self.error_breakdown.resource_limit_errors);
        println!("   数据质量错误: {}", self.error_breakdown.data_quality_errors);
        println!("   性能阈值错误: {}", self.error_breakdown.performance_threshold_errors);
        println!("   并发冲突错误: {}", self.error_breakdown.concurrency_conflict_errors);
        println!("   数据损坏错误: {}", self.error_breakdown.data_corruption_errors);
        println!("   资源不足错误: {}", self.error_breakdown.insufficient_resources_errors);
        println!("   导出中断错误: {}", self.error_breakdown.export_interrupted_errors);
    }
}

/// 资源监控器
#[derive(Debug)]
pub struct ResourceMonitor {
    /// 内存限制（字节）
    memory_limit: usize,
    /// 磁盘空间限制（字节）
    disk_limit: usize,
    /// CPU 使用率限制（百分比）
    cpu_limit: f64,
    /// 监控间隔
    monitoring_interval: Duration,
}

impl ResourceMonitor {
    /// 创建新的资源监控器
    pub fn new(memory_limit_mb: usize, disk_limit_mb: usize, cpu_limit_percent: f64) -> Self {
        Self {
            memory_limit: memory_limit_mb * 1024 * 1024,
            disk_limit: disk_limit_mb * 1024 * 1024,
            cpu_limit: cpu_limit_percent,
            monitoring_interval: Duration::from_millis(100),
        }
    }

    /// 检查资源使用情况
    pub fn check_resource_usage(&self) -> TrackingResult<ResourceUsage> {
        let memory_usage = self.get_memory_usage()?;
        let disk_usage = self.get_disk_usage()?;
        let cpu_usage = self.get_cpu_usage()?;

        // 检查是否超出限制
        if memory_usage > self.memory_limit {
            return Err(ExportError::ResourceLimitExceeded {
                resource_type: ResourceType::Memory,
                limit: self.memory_limit as u64,
                actual: memory_usage as u64,
                suggested_action: "减少并行度或启用流式处理".to_string(),
            }.into());
        }

        if disk_usage > self.disk_limit {
            return Err(ExportError::ResourceLimitExceeded {
                resource_type: ResourceType::Disk,
                limit: self.disk_limit as u64,
                actual: disk_usage as u64,
                suggested_action: "清理临时文件或选择其他输出位置".to_string(),
            }.into());
        }

        if cpu_usage > self.cpu_limit {
            return Err(ExportError::ResourceLimitExceeded {
                resource_type: ResourceType::CPU,
                limit: (self.cpu_limit * 100.0) as u64,
                actual: (cpu_usage * 100.0) as u64,
                suggested_action: "减少线程数或降低处理优先级".to_string(),
            }.into());
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

    /// 获取内存使用情况（简化实现）
    fn get_memory_usage(&self) -> TrackingResult<usize> {
        // 在实际实现中，这里应该调用系统 API 获取真实的内存使用情况
        // 这里使用简化的实现
        Ok(0) // 占位符实现
    }

    /// 获取磁盘使用情况（简化实现）
    fn get_disk_usage(&self) -> TrackingResult<usize> {
        // 在实际实现中，这里应该调用系统 API 获取真实的磁盘使用情况
        Ok(0) // 占位符实现
    }

    /// 获取 CPU 使用情况（简化实现）
    fn get_cpu_usage(&self) -> TrackingResult<f64> {
        // 在实际实现中，这里应该调用系统 API 获取真实的 CPU 使用情况
        Ok(0.0) // 占位符实现
    }
}

/// 资源使用情况
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_usage: usize,
    pub disk_usage: usize,
    pub cpu_usage: f64,
    pub memory_limit: usize,
    pub disk_limit: usize,
    pub cpu_limit: f64,
}

impl ResourceUsage {
    /// 获取内存使用率（百分比）
    pub fn memory_usage_percentage(&self) -> f64 {
        if self.memory_limit > 0 {
            (self.memory_usage as f64 / self.memory_limit as f64) * 100.0
        } else {
            0.0
        }
    }

    /// 获取磁盘使用率（百分比）
    pub fn disk_usage_percentage(&self) -> f64 {
        if self.disk_limit > 0 {
            (self.disk_usage as f64 / self.disk_limit as f64) * 100.0
        } else {
            0.0
        }
    }

    /// 获取 CPU 使用率（百分比）
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
            error_message: "序列化失败".to_string(),
            partial_results: None,
        };
        
        let display = format!("{error}");
        assert!(display.contains("并行处理错误"));
        assert!(display.contains("分片 5"));
        assert!(display.contains("thread-1"));
    }

    #[test]
    fn test_performance_logger() {
        let logger = PerformanceLogger::new(LogLevel::Info);
        
        logger.log_operation_start("测试操作", "测试详情");
        logger.log_operation_success("测试操作", Duration::from_millis(100), "成功完成");
        
        let report = logger.generate_performance_report();
        assert_eq!(report.total_operations, 1);
        assert_eq!(report.successful_operations, 1);
        assert_eq!(report.failed_operations, 0);
        assert_eq!(report.success_rate, 100.0);
    }

    #[test]
    fn test_resource_monitor() {
        let monitor = ResourceMonitor::new(1024, 2048, 80.0);
        
        // 测试资源检查（使用简化实现，应该总是成功）
        let result = monitor.check_resource_usage();
        assert!(result.is_ok());
        
        let usage = result.unwrap();
        assert_eq!(usage.memory_limit, 1024 * 1024 * 1024);
        assert_eq!(usage.disk_limit, 2048 * 1024 * 1024);
        assert_eq!(usage.cpu_limit, 80.0);
    }

    #[test]
    fn test_resource_usage_percentages() {
        let usage = ResourceUsage {
            memory_usage: 512 * 1024 * 1024, // 512MB
            disk_usage: 1024 * 1024 * 1024,  // 1GB
            cpu_usage: 0.6,                  // 60%
            memory_limit: 1024 * 1024 * 1024, // 1GB
            disk_limit: 2048 * 1024 * 1024,   // 2GB
            cpu_limit: 0.8,                   // 80%
        };
        
        assert_eq!(usage.memory_usage_percentage(), 50.0);
        assert_eq!(usage.disk_usage_percentage(), 50.0);
        assert_eq!(usage.cpu_usage_percentage(), 60.0);
    }
}