//! 错误恢复机制
//!
//! 这个模块提供了全面的错误恢复策略，包括自动重试、优雅降级、
//! 部分结果保存和错误状态恢复，确保导出系统在各种异常情况下
//! 都能提供最佳的用户体验。

use crate::core::types::{TrackingError, TrackingResult};
use crate::export::error_handling::{ExportError, ExportStage, ResourceType, ConflictType};
use crate::export::fast_export_coordinator::{FastExportConfig, CompleteExportStats};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
// Removed unused atomic imports

/// 错误恢复管理器
#[derive(Debug)]
pub struct ErrorRecoveryManager {
    /// 恢复配置
    config: RecoveryConfig,
    /// 恢复统计
    stats: RecoveryStats,
    /// 重试历史
    retry_history: HashMap<String, RetryHistory>,
    /// 降级状态
    degradation_state: DegradationState,
}

/// 恢复配置
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// 是否启用自动重试
    pub enable_auto_retry: bool,
    /// 最大重试次数
    pub max_retry_attempts: usize,
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
    /// 重试间隔递增因子
    pub retry_backoff_factor: f64,
    /// 最大重试间隔（毫秒）
    pub max_retry_interval_ms: u64,
    
    /// 是否启用优雅降级
    pub enable_graceful_degradation: bool,
    /// 降级阈值（错误率百分比）
    pub degradation_threshold: f64,
    /// 降级恢复阈值（错误率百分比）
    pub recovery_threshold: f64,
    
    /// 是否启用部分结果保存
    pub enable_partial_save: bool,
    /// 部分结果保存目录
    pub partial_save_directory: PathBuf,
    /// 部分结果保存间隔（操作数）
    pub partial_save_interval: usize,
    
    /// 是否启用详细日志
    pub verbose_logging: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            enable_auto_retry: true,
            max_retry_attempts: 3,
            retry_interval_ms: 1000,
            retry_backoff_factor: 2.0,
            max_retry_interval_ms: 10000,
            
            enable_graceful_degradation: true,
            degradation_threshold: 10.0, // 10% 错误率触发降级
            recovery_threshold: 2.0,     // 2% 错误率恢复正常
            
            enable_partial_save: true,
            partial_save_directory: PathBuf::from("./partial_exports"),
            partial_save_interval: 1000,
            
            verbose_logging: false,
        }
    }
}

/// 恢复统计
#[derive(Debug, Clone, Default)]
pub struct RecoveryStats {
    /// 总错误数
    pub total_errors: usize,
    /// 成功恢复数
    pub successful_recoveries: usize,
    /// 失败恢复数
    pub failed_recoveries: usize,
    /// 总重试次数
    pub total_retries: usize,
    /// 降级次数
    pub degradation_count: usize,
    /// 部分保存次数
    pub partial_saves: usize,
    /// 恢复时间统计（毫秒）
    pub total_recovery_time_ms: u64,
}

/// 重试历史
#[derive(Debug, Clone)]
pub struct RetryHistory {
    /// 操作名称
    pub operation: String,
    /// 重试次数
    pub attempt_count: usize,
    /// 最后重试时间
    pub last_attempt: Instant,
    /// 下次重试间隔
    pub next_interval_ms: u64,
    /// 错误历史
    pub error_history: Vec<String>,
}

/// 降级状态
#[derive(Debug, Clone)]
pub struct DegradationState {
    /// 是否处于降级状态
    pub is_degraded: bool,
    /// 降级开始时间
    pub degradation_start: Option<Instant>,
    /// 当前错误率
    pub current_error_rate: f64,
    /// 降级级别
    pub degradation_level: DegradationLevel,
    /// 降级原因
    pub degradation_reason: Option<String>,
}

/// 降级级别
#[derive(Debug, Clone, PartialEq)]
pub enum DegradationLevel {
    /// 正常运行
    Normal,
    /// 轻微降级（减少并行度）
    Light,
    /// 中等降级（禁用复杂功能）
    Moderate,
    /// 严重降级（仅基本功能）
    Severe,
    /// 紧急模式（最小功能）
    Emergency,
}

/// 恢复策略
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// 自动重试
    AutoRetry {
        max_attempts: usize,
        interval_ms: u64,
        backoff_factor: f64,
    },
    /// 优雅降级
    GracefulDegradation {
        target_level: DegradationLevel,
        reason: String,
    },
    /// 部分结果保存
    PartialSave {
        save_path: PathBuf,
        progress_percentage: f64,
    },
    /// 配置调整
    ConfigAdjustment {
        new_config: FastExportConfig,
        reason: String,
    },
    /// 资源释放
    ResourceRelease {
        resource_type: ResourceType,
        amount: usize,
    },
    /// 操作跳过
    SkipOperation {
        operation: String,
        reason: String,
    },
}

/// 恢复结果
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    /// 恢复是否成功
    pub success: bool,
    /// 使用的恢复策略
    pub strategy: RecoveryStrategy,
    /// 恢复消息
    pub message: String,
    /// 恢复耗时
    pub recovery_time_ms: u64,
    /// 部分结果路径（如果有）
    pub partial_result_path: Option<PathBuf>,
    /// 建议的后续操作
    pub suggested_actions: Vec<String>,
}

impl ErrorRecoveryManager {
    /// 创建新的错误恢复管理器
    pub fn new(config: RecoveryConfig) -> Self {
        // 确保部分保存目录存在
        if config.enable_partial_save {
            if let Err(e) = std::fs::create_dir_all(&config.partial_save_directory) {
                eprintln!("⚠️ 无法创建部分保存目录: {e}");
            }
        }

        Self {
            config,
            stats: RecoveryStats::default(),
            retry_history: HashMap::new(),
            degradation_state: DegradationState {
                is_degraded: false,
                degradation_start: None,
                current_error_rate: 0.0,
                degradation_level: DegradationLevel::Normal,
                degradation_reason: None,
            },
        }
    }

    /// 处理导出错误并尝试恢复
    pub fn handle_export_error(
        &mut self,
        error: &ExportError,
        operation: &str,
        context: &ErrorContext,
    ) -> TrackingResult<RecoveryResult> {
        let recovery_start = Instant::now();
        self.stats.total_errors += 1;

        if self.config.verbose_logging {
            println!("🔧 开始错误恢复: {} - {}", operation, error);
        }

        // 选择恢复策略
        let strategy = self.select_recovery_strategy(error, operation, context)?;

        // 执行恢复策略
        let result = self.execute_recovery_strategy(strategy, error, operation, context)?;

        // 更新统计信息
        let recovery_time = recovery_start.elapsed().as_millis() as u64;
        self.stats.total_recovery_time_ms += recovery_time;

        if result.success {
            self.stats.successful_recoveries += 1;
        } else {
            self.stats.failed_recoveries += 1;
        }

        // 更新降级状态
        self.update_degradation_state(error, &result);

        if self.config.verbose_logging {
            println!("🔧 恢复完成: {} ({}ms)", result.message, recovery_time);
        }

        Ok(result)
    }

    /// 选择恢复策略
    fn select_recovery_strategy(
        &self,
        error: &ExportError,
        operation: &str,
        context: &ErrorContext,
    ) -> TrackingResult<RecoveryStrategy> {
        match error {
            ExportError::ParallelProcessingError { shard_index, .. } => {
                // 并行处理错误：尝试重试或降级并行度
                if self.should_retry(operation) {
                    Ok(RecoveryStrategy::AutoRetry {
                        max_attempts: self.config.max_retry_attempts,
                        interval_ms: self.config.retry_interval_ms,
                        backoff_factor: self.config.retry_backoff_factor,
                    })
                } else {
                    Ok(RecoveryStrategy::GracefulDegradation {
                        target_level: DegradationLevel::Light,
                        reason: format!("分片 {shard_index} 处理失败，降低并行度"),
                    })
                }
            }

            ExportError::ResourceLimitExceeded { resource_type, suggested_action, .. } => {
                // 资源限制错误：释放资源或调整配置
                match resource_type {
                    ResourceType::Memory => {
                        Ok(RecoveryStrategy::ConfigAdjustment {
                            new_config: self.create_memory_optimized_config(context),
                            reason: "内存不足，调整为内存优化配置".to_string(),
                        })
                    }
                    ResourceType::CPU => {
                        Ok(RecoveryStrategy::GracefulDegradation {
                            target_level: DegradationLevel::Moderate,
                            reason: "CPU 使用率过高，降低处理强度".to_string(),
                        })
                    }
                    _ => {
                        Ok(RecoveryStrategy::ConfigAdjustment {
                            new_config: context.current_config.clone(),
                            reason: suggested_action.clone(),
                        })
                    }
                }
            }

            ExportError::DataQualityError { affected_records, .. } => {
                // 数据质量错误：保存部分结果
                if self.config.enable_partial_save && context.progress_percentage > 10.0 {
                    Ok(RecoveryStrategy::PartialSave {
                        save_path: self.generate_partial_save_path(operation),
                        progress_percentage: context.progress_percentage,
                    })
                } else {
                    Ok(RecoveryStrategy::SkipOperation {
                        operation: operation.to_string(),
                        reason: format!("数据质量问题影响 {affected_records} 条记录，跳过处理"),
                    })
                }
            }

            ExportError::PerformanceThresholdExceeded { stage, .. } => {
                // 性能阈值错误：调整配置或降级
                match stage {
                    ExportStage::ParallelProcessing => {
                        Ok(RecoveryStrategy::ConfigAdjustment {
                            new_config: self.create_performance_optimized_config(context),
                            reason: "并行处理性能不佳，调整配置".to_string(),
                        })
                    }
                    _ => {
                        Ok(RecoveryStrategy::GracefulDegradation {
                            target_level: DegradationLevel::Light,
                            reason: format!("阶段 {stage:?} 性能不佳，轻微降级"),
                        })
                    }
                }
            }

            ExportError::ConcurrencyConflict { conflict_type, retry_count, .. } => {
                // 并发冲突错误：重试或调整并发策略
                if *retry_count < self.config.max_retry_attempts {
                    let interval = match conflict_type {
                        ConflictType::LockContention => self.config.retry_interval_ms * 2,
                        ConflictType::ThreadPoolExhaustion => self.config.retry_interval_ms * 3,
                        _ => self.config.retry_interval_ms,
                    };

                    Ok(RecoveryStrategy::AutoRetry {
                        max_attempts: self.config.max_retry_attempts - retry_count,
                        interval_ms: interval,
                        backoff_factor: self.config.retry_backoff_factor,
                    })
                } else {
                    Ok(RecoveryStrategy::GracefulDegradation {
                        target_level: DegradationLevel::Moderate,
                        reason: "并发冲突频繁，降低并发度".to_string(),
                    })
                }
            }

            ExportError::InsufficientResources { .. } => {
                // 资源不足错误：紧急降级
                Ok(RecoveryStrategy::GracefulDegradation {
                    target_level: DegradationLevel::Emergency,
                    reason: "系统资源严重不足，启用紧急模式".to_string(),
                })
            }

            ExportError::ExportInterrupted { progress_percentage, .. } => {
                // 导出中断错误：保存部分结果
                Ok(RecoveryStrategy::PartialSave {
                    save_path: self.generate_partial_save_path(operation),
                    progress_percentage: *progress_percentage,
                })
            }

            ExportError::DataCorruption { recovery_possible, .. } => {
                // 数据损坏错误：根据是否可恢复决定策略
                if *recovery_possible {
                    Ok(RecoveryStrategy::AutoRetry {
                        max_attempts: 1, // 只重试一次
                        interval_ms: self.config.retry_interval_ms,
                        backoff_factor: 1.0,
                    })
                } else {
                    Ok(RecoveryStrategy::SkipOperation {
                        operation: operation.to_string(),
                        reason: "数据损坏且无法恢复，跳过操作".to_string(),
                    })
                }
            }
        }
    }

    /// 执行恢复策略
    fn execute_recovery_strategy(
        &mut self,
        strategy: RecoveryStrategy,
        _error: &ExportError,
        operation: &str,
        context: &ErrorContext,
    ) -> TrackingResult<RecoveryResult> {
        let _execution_start = Instant::now();

        match strategy {
            RecoveryStrategy::AutoRetry { max_attempts, interval_ms, backoff_factor } => {
                self.execute_auto_retry(operation, max_attempts, interval_ms, backoff_factor)
            }

            RecoveryStrategy::GracefulDegradation { target_level, reason } => {
                self.execute_graceful_degradation(target_level, reason)
            }

            RecoveryStrategy::PartialSave { save_path, progress_percentage } => {
                self.execute_partial_save(save_path, progress_percentage, context)
            }

            RecoveryStrategy::ConfigAdjustment { new_config, reason } => {
                self.execute_config_adjustment(new_config, reason)
            }

            RecoveryStrategy::ResourceRelease { resource_type, amount } => {
                self.execute_resource_release(resource_type, amount)
            }

            RecoveryStrategy::SkipOperation { operation, reason } => {
                self.execute_skip_operation(operation, reason)
            }
        }
    }

    /// 执行自动重试
    fn execute_auto_retry(
        &mut self,
        operation: &str,
        max_attempts: usize,
        interval_ms: u64,
        backoff_factor: f64,
    ) -> TrackingResult<RecoveryResult> {
        let history = self.retry_history.entry(operation.to_string())
            .or_insert_with(|| RetryHistory {
                operation: operation.to_string(),
                attempt_count: 0,
                last_attempt: Instant::now(),
                next_interval_ms: interval_ms,
                error_history: Vec::new(),
            });

        if history.attempt_count >= max_attempts {
            return Ok(RecoveryResult {
                success: false,
                strategy: RecoveryStrategy::AutoRetry { max_attempts, interval_ms, backoff_factor },
                message: format!("重试次数已达上限 ({max_attempts})"),
                recovery_time_ms: 0,
                partial_result_path: None,
                suggested_actions: vec!["考虑手动干预或调整配置".to_string()],
            });
        }

        // 等待重试间隔
        if history.attempt_count > 0 {
            std::thread::sleep(Duration::from_millis(history.next_interval_ms));
        }

        history.attempt_count += 1;
        history.last_attempt = Instant::now();
        history.next_interval_ms = (history.next_interval_ms as f64 * backoff_factor) as u64;
        history.next_interval_ms = history.next_interval_ms.min(self.config.max_retry_interval_ms);

        self.stats.total_retries += 1;

        Ok(RecoveryResult {
            success: true,
            strategy: RecoveryStrategy::AutoRetry { max_attempts, interval_ms, backoff_factor },
            message: format!("准备第 {} 次重试 (最多 {} 次)", history.attempt_count, max_attempts),
            recovery_time_ms: history.next_interval_ms,
            partial_result_path: None,
            suggested_actions: vec![
                "监控重试结果".to_string(),
                "如果持续失败，考虑调整策略".to_string(),
            ],
        })
    }

    /// 执行优雅降级
    fn execute_graceful_degradation(
        &mut self,
        target_level: DegradationLevel,
        reason: String,
    ) -> TrackingResult<RecoveryResult> {
        self.degradation_state.is_degraded = true;
        self.degradation_state.degradation_start = Some(Instant::now());
        self.degradation_state.degradation_level = target_level.clone();
        self.degradation_state.degradation_reason = Some(reason.clone());
        self.stats.degradation_count += 1;

        let message = match target_level {
            DegradationLevel::Light => "启用轻微降级模式：减少并行度",
            DegradationLevel::Moderate => "启用中等降级模式：禁用复杂功能",
            DegradationLevel::Severe => "启用严重降级模式：仅保留基本功能",
            DegradationLevel::Emergency => "启用紧急模式：最小功能运行",
            DegradationLevel::Normal => "恢复正常模式",
        };

        Ok(RecoveryResult {
            success: true,
            strategy: RecoveryStrategy::GracefulDegradation { target_level, reason },
            message: message.to_string(),
            recovery_time_ms: 0,
            partial_result_path: None,
            suggested_actions: vec![
                "监控系统状态".to_string(),
                "在条件改善后考虑恢复正常模式".to_string(),
            ],
        })
    }

    /// 执行部分保存
    fn execute_partial_save(
        &mut self,
        save_path: PathBuf,
        progress_percentage: f64,
        context: &ErrorContext,
    ) -> TrackingResult<RecoveryResult> {
        // 创建部分保存目录
        if let Some(parent) = save_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| TrackingError::IoError(format!("创建部分保存目录失败: {e}")))?;
        }

        // 保存部分结果（这里是简化实现）
        let partial_data = format!(
            "{{\"partial_export\":true,\"progress\":{progress_percentage},\"timestamp\":\"{}\",\"context\":\"{}\"}}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            context.operation_id
        );

        std::fs::write(&save_path, partial_data)
            .map_err(|e| TrackingError::IoError(format!("保存部分结果失败: {e}")))?;

        self.stats.partial_saves += 1;

        Ok(RecoveryResult {
            success: true,
            strategy: RecoveryStrategy::PartialSave { save_path: save_path.clone(), progress_percentage },
            message: format!("部分结果已保存 ({progress_percentage:.1}% 完成)"),
            recovery_time_ms: 0,
            partial_result_path: Some(save_path),
            suggested_actions: vec![
                "检查部分结果文件".to_string(),
                "修复问题后可从此处继续".to_string(),
            ],
        })
    }

    /// 执行配置调整
    fn execute_config_adjustment(
        &self,
        new_config: FastExportConfig,
        reason: String,
    ) -> TrackingResult<RecoveryResult> {
        Ok(RecoveryResult {
            success: true,
            strategy: RecoveryStrategy::ConfigAdjustment { new_config, reason: reason.clone() },
            message: format!("配置已调整: {reason}"),
            recovery_time_ms: 0,
            partial_result_path: None,
            suggested_actions: vec![
                "使用新配置重新尝试导出".to_string(),
                "监控新配置的效果".to_string(),
            ],
        })
    }

    /// 执行资源释放
    fn execute_resource_release(
        &self,
        resource_type: ResourceType,
        amount: usize,
    ) -> TrackingResult<RecoveryResult> {
        // 这里是简化实现，实际应该调用系统 API 释放资源
        let message = match resource_type {
            ResourceType::Memory => format!("尝试释放 {amount} 字节内存"),
            ResourceType::CPU => format!("降低 CPU 使用率 {amount}%"),
            ResourceType::Disk => format!("清理 {amount} 字节磁盘空间"),
            ResourceType::FileHandles => format!("关闭 {amount} 个文件句柄"),
            ResourceType::ThreadPool => format!("减少 {amount} 个线程"),
        };

        Ok(RecoveryResult {
            success: true,
            strategy: RecoveryStrategy::ResourceRelease { resource_type, amount },
            message,
            recovery_time_ms: 0,
            partial_result_path: None,
            suggested_actions: vec![
                "监控资源使用情况".to_string(),
                "重新尝试失败的操作".to_string(),
            ],
        })
    }

    /// 执行跳过操作
    fn execute_skip_operation(
        &self,
        operation: String,
        reason: String,
    ) -> TrackingResult<RecoveryResult> {
        Ok(RecoveryResult {
            success: true,
            strategy: RecoveryStrategy::SkipOperation { operation: operation.clone(), reason: reason.clone() },
            message: format!("跳过操作 '{operation}': {reason}"),
            recovery_time_ms: 0,
            partial_result_path: None,
            suggested_actions: vec![
                "检查跳过操作的影响".to_string(),
                "考虑手动处理跳过的部分".to_string(),
            ],
        })
    }

    /// 检查是否应该重试
    fn should_retry(&self, operation: &str) -> bool {
        if !self.config.enable_auto_retry {
            return false;
        }

        if let Some(history) = self.retry_history.get(operation) {
            history.attempt_count < self.config.max_retry_attempts
        } else {
            true
        }
    }

    /// 更新降级状态
    fn update_degradation_state(&mut self, error: &ExportError, _result: &RecoveryResult) {
        // 简化的错误率计算
        let error_weight = match error {
            ExportError::ParallelProcessingError { .. } => 1.0,
            ExportError::ResourceLimitExceeded { .. } => 2.0,
            ExportError::DataQualityError { .. } => 1.5,
            ExportError::PerformanceThresholdExceeded { .. } => 1.0,
            ExportError::ConcurrencyConflict { .. } => 1.0,
            ExportError::DataCorruption { .. } => 3.0,
            ExportError::InsufficientResources { .. } => 2.5,
            ExportError::ExportInterrupted { .. } => 1.5,
        };

        // 更新错误率（简化计算）
        self.degradation_state.current_error_rate = 
            (self.degradation_state.current_error_rate * 0.9) + (error_weight * 0.1);

        // 检查是否需要降级或恢复
        if !self.degradation_state.is_degraded && 
           self.degradation_state.current_error_rate > self.config.degradation_threshold {
            // 触发降级
            self.degradation_state.is_degraded = true;
            self.degradation_state.degradation_start = Some(Instant::now());
            self.degradation_state.degradation_level = DegradationLevel::Light;
        } else if self.degradation_state.is_degraded && 
                  self.degradation_state.current_error_rate < self.config.recovery_threshold {
            // 恢复正常
            self.degradation_state.is_degraded = false;
            self.degradation_state.degradation_start = None;
            self.degradation_state.degradation_level = DegradationLevel::Normal;
            self.degradation_state.degradation_reason = None;
        }
    }

    /// 生成部分保存路径
    fn generate_partial_save_path(&self, operation: &str) -> PathBuf {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let filename = format!("partial_export_{operation}_{timestamp}.json");
        self.config.partial_save_directory.join(filename)
    }

    /// 创建内存优化配置
    fn create_memory_optimized_config(&self, context: &ErrorContext) -> FastExportConfig {
        let mut config = context.current_config.clone();
        
        // 减少并行度
        config.shard_config.max_threads = Some(2);
        config.shard_config.shard_size = config.shard_config.shard_size / 2;
        
        // 减少缓冲区大小
        config.writer_config.buffer_size = config.writer_config.buffer_size / 2;
        
        // 启用流式处理
        config.enable_data_localization = false;
        
        config
    }

    /// 创建性能优化配置
    fn create_performance_optimized_config(&self, context: &ErrorContext) -> FastExportConfig {
        let mut config = context.current_config.clone();
        
        // 调整分片大小
        config.shard_config.shard_size = 500; // 较小的分片
        config.shard_config.parallel_threshold = 1000; // 较低的并行阈值
        
        // 禁用详细日志
        config.verbose_logging = false;
        config.enable_performance_monitoring = false;
        
        config
    }

    /// 获取恢复统计
    pub fn get_stats(&self) -> &RecoveryStats {
        &self.stats
    }

    /// 获取降级状态
    pub fn get_degradation_state(&self) -> &DegradationState {
        &self.degradation_state
    }

    /// 生成恢复报告
    pub fn generate_recovery_report(&self) -> RecoveryReport {
        let success_rate = if self.stats.total_errors > 0 {
            (self.stats.successful_recoveries as f64 / self.stats.total_errors as f64) * 100.0
        } else {
            0.0
        };

        let avg_recovery_time = if self.stats.successful_recoveries > 0 {
            self.stats.total_recovery_time_ms as f64 / self.stats.successful_recoveries as f64
        } else {
            0.0
        };

        RecoveryReport {
            total_errors: self.stats.total_errors,
            successful_recoveries: self.stats.successful_recoveries,
            failed_recoveries: self.stats.failed_recoveries,
            success_rate,
            total_retries: self.stats.total_retries,
            degradation_count: self.stats.degradation_count,
            partial_saves: self.stats.partial_saves,
            avg_recovery_time_ms: avg_recovery_time,
            current_degradation_level: self.degradation_state.degradation_level.clone(),
            is_currently_degraded: self.degradation_state.is_degraded,
        }
    }
}

/// 错误上下文
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// 操作 ID
    pub operation_id: String,
    /// 当前配置
    pub current_config: FastExportConfig,
    /// 进度百分比
    pub progress_percentage: f64,
    /// 已处理的数据量
    pub processed_data_size: usize,
    /// 操作开始时间
    pub operation_start_time: Instant,
    /// 当前导出统计
    pub current_stats: Option<CompleteExportStats>,
}

/// 恢复报告
#[derive(Debug, Clone)]
pub struct RecoveryReport {
    pub total_errors: usize,
    pub successful_recoveries: usize,
    pub failed_recoveries: usize,
    pub success_rate: f64,
    pub total_retries: usize,
    pub degradation_count: usize,
    pub partial_saves: usize,
    pub avg_recovery_time_ms: f64,
    pub current_degradation_level: DegradationLevel,
    pub is_currently_degraded: bool,
}

impl RecoveryReport {
    /// 打印详细的恢复报告
    pub fn print_detailed_report(&self) {
        println!("\n🔧 错误恢复报告");
        println!("================");
        
        println!("📊 总体统计:");
        println!("   总错误数: {}", self.total_errors);
        println!("   成功恢复: {} ({:.1}%)", self.successful_recoveries, self.success_rate);
        println!("   失败恢复: {}", self.failed_recoveries);
        println!("   总重试次数: {}", self.total_retries);
        println!("   降级次数: {}", self.degradation_count);
        println!("   部分保存: {}", self.partial_saves);
        println!("   平均恢复时间: {:.2}ms", self.avg_recovery_time_ms);
        
        println!("\n🎚️ 当前状态:");
        println!("   降级级别: {:?}", self.current_degradation_level);
        println!("   是否降级: {}", if self.is_currently_degraded { "是" } else { "否" });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::fast_export_coordinator::FastExportConfig;

    fn create_test_context() -> ErrorContext {
        ErrorContext {
            operation_id: "test_operation".to_string(),
            current_config: FastExportConfig::default(),
            progress_percentage: 50.0,
            processed_data_size: 1000,
            operation_start_time: Instant::now(),
            current_stats: None,
        }
    }

    #[test]
    fn test_error_recovery_manager_creation() {
        let config = RecoveryConfig::default();
        let manager = ErrorRecoveryManager::new(config);
        assert_eq!(manager.stats.total_errors, 0);
        assert!(!manager.degradation_state.is_degraded);
    }

    #[test]
    fn test_handle_parallel_processing_error() {
        let mut manager = ErrorRecoveryManager::new(RecoveryConfig::default());
        let error = ExportError::ParallelProcessingError {
            shard_index: 5,
            thread_id: "thread-1".to_string(),
            error_message: "测试错误".to_string(),
            partial_results: None,
        };
        let context = create_test_context();

        let result = manager.handle_export_error(&error, "test_operation", &context);
        assert!(result.is_ok());
        
        let recovery_result = result.unwrap();
        assert!(recovery_result.success);
        assert_eq!(manager.stats.total_errors, 1);
    }

    #[test]
    fn test_graceful_degradation() {
        let mut manager = ErrorRecoveryManager::new(RecoveryConfig::default());
        
        let result = manager.execute_graceful_degradation(
            DegradationLevel::Light,
            "测试降级".to_string(),
        );
        
        assert!(result.is_ok());
        let recovery_result = result.unwrap();
        assert!(recovery_result.success);
        assert!(manager.degradation_state.is_degraded);
        assert_eq!(manager.degradation_state.degradation_level, DegradationLevel::Light);
    }

    #[test]
    fn test_partial_save() {
        let config = RecoveryConfig {
            partial_save_directory: std::env::temp_dir().join("test_partial_saves"),
            ..Default::default()
        };
        let mut manager = ErrorRecoveryManager::new(config);
        let context = create_test_context();

        let save_path = manager.generate_partial_save_path("test_op");
        let result = manager.execute_partial_save(save_path.clone(), 75.0, &context);
        
        assert!(result.is_ok());
        let recovery_result = result.unwrap();
        assert!(recovery_result.success);
        assert_eq!(recovery_result.partial_result_path, Some(save_path.clone()));
        
        // 清理测试文件
        let _ = std::fs::remove_file(save_path);
    }

    #[test]
    fn test_recovery_report() {
        let mut manager = ErrorRecoveryManager::new(RecoveryConfig::default());
        
        // 模拟一些错误和恢复
        manager.stats.total_errors = 10;
        manager.stats.successful_recoveries = 8;
        manager.stats.failed_recoveries = 2;
        manager.stats.total_retries = 5;
        manager.stats.degradation_count = 1;
        manager.stats.partial_saves = 2;
        manager.stats.total_recovery_time_ms = 1000;

        let report = manager.generate_recovery_report();
        assert_eq!(report.total_errors, 10);
        assert_eq!(report.successful_recoveries, 8);
        assert_eq!(report.success_rate, 80.0);
        assert_eq!(report.avg_recovery_time_ms, 125.0); // 1000 / 8
    }

    #[test]
    fn test_retry_logic() {
        let mut manager = ErrorRecoveryManager::new(RecoveryConfig::default());
        
        // 第一次重试应该成功
        assert!(manager.should_retry("test_operation"));
        
        // 模拟多次重试
        for i in 0..3 {
            let result = manager.execute_auto_retry("test_operation", 3, 100, 2.0);
            assert!(result.is_ok());
            let recovery_result = result.unwrap();
            assert!(recovery_result.success);
        }
        
        // 超过最大重试次数后应该失败
        let result = manager.execute_auto_retry("test_operation", 3, 100, 2.0);
        assert!(result.is_ok());
        let recovery_result = result.unwrap();
        assert!(!recovery_result.success);
    }
}