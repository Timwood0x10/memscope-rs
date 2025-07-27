//! error recovery mechanism
//!
//! this module provides comprehensive error recovery strategies,
//! including automatic retries, graceful degradation,
//! partial result saving, and error state recovery,
//! ensuring optimal user experience in various
use crate::core::types::{TrackingError, TrackingResult};
use crate::export::error_handling::{ConflictType, ExportError, ExportStage, ResourceType};
use crate::export::fast_export_coordinator::{CompleteExportStats, FastExportConfig};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
// Removed unused atomic imports

/// error recovery manager
#[derive(Debug)]
pub struct ErrorRecoveryManager {
    /// recovery config
    config: RecoveryConfig,
    /// recovery stats
    stats: RecoveryStats,
    /// retry history
    retry_history: HashMap<String, RetryHistory>,
    /// degradation state
    degradation_state: DegradationState,
}

/// recovery config
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// whether to enable auto retry
    pub enable_auto_retry: bool,
    /// max retry attempts
    pub max_retry_attempts: usize,
    /// retry interval (milliseconds)
    pub retry_interval_ms: u64,
    /// retry interval backoff factor
    pub retry_backoff_factor: f64,
    /// max retry interval (milliseconds)
    pub max_retry_interval_ms: u64,

    /// whether to enable graceful degradation
    pub enable_graceful_degradation: bool,
    /// degradation threshold (error rate percentage)
    pub degradation_threshold: f64,
    /// recovery threshold (error rate percentage)
    pub recovery_threshold: f64,

    /// whether to enable partial result saving
    pub enable_partial_save: bool,
    /// partial result save directory
    pub partial_save_directory: PathBuf,
    /// partial result save interval (operations count)
    pub partial_save_interval: usize,

    /// whether to enable verbose logging
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
            degradation_threshold: 10.0, // 10% error rate triggers degradation
            recovery_threshold: 2.0,     // 2% error rate恢复正常

            enable_partial_save: true,
            partial_save_directory: PathBuf::from("./partial_exports"),
            partial_save_interval: 1000,

            verbose_logging: false,
        }
    }
}

/// recovery stats
#[derive(Debug, Clone, Default)]
pub struct RecoveryStats {
    /// total errors
    pub total_errors: usize,
    /// successful recoveries
    pub successful_recoveries: usize,
    /// failed recoveries
    pub failed_recoveries: usize,
    /// total retries
    pub total_retries: usize,
    /// degradation count
    pub degradation_count: usize,
    /// partial saves count
    pub partial_saves: usize,
    /// total recovery time (milliseconds)
    pub total_recovery_time_ms: u64,
}

/// retry history
#[derive(Debug, Clone)]
pub struct RetryHistory {
    /// operation name
    pub operation: String,
    /// retry count
    pub attempt_count: usize,
    /// last retry time
    pub last_attempt: Instant,
    /// next retry interval (milliseconds)
    pub next_interval_ms: u64,
    /// error history
    pub error_history: Vec<String>,
}

/// degradation state
#[derive(Debug, Clone)]
pub struct DegradationState {
    /// whether in degradation state
    pub is_degraded: bool,
    /// degradation start time
    pub degradation_start: Option<Instant>,
    /// current error rate
    pub current_error_rate: f64,
    /// degradation level
    pub degradation_level: DegradationLevel,
    /// degradation reason
    pub degradation_reason: Option<String>,
}

/// degradation level
#[derive(Debug, Clone, PartialEq)]
pub enum DegradationLevel {
    /// normal operation
    Normal,
    /// light degradation (reduce parallelism)
    Light,
    /// moderate degradation (disable complex features)
    Moderate,
    /// severe degradation (only basic features)
    Severe,
    /// emergency mode (minimum features)
    Emergency,
}

/// recovery strategy
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// auto retry
    AutoRetry {
        max_attempts: usize,
        interval_ms: u64,
        backoff_factor: f64,
    },
    /// graceful degradation
    GracefulDegradation {
        target_level: DegradationLevel,
        reason: String,
    },
    /// partial result saving
    PartialSave {
        save_path: PathBuf,
        progress_percentage: f64,
    },
    /// config adjustment
    ConfigAdjustment {
        new_config: FastExportConfig,
        reason: String,
    },
    /// resource release
    ResourceRelease {
        resource_type: ResourceType,
        amount: usize,
    },
    /// skip operation
    SkipOperation { operation: String, reason: String },
}

/// recovery result
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    /// whether recovery is successful
    pub success: bool,
    /// recovery strategy
    pub strategy: RecoveryStrategy,
    /// recovery message
    pub message: String,
    /// recovery time (milliseconds)
    pub recovery_time_ms: u64,
    /// partial result path (if any)
    pub partial_result_path: Option<PathBuf>,
    /// suggested actions
    pub suggested_actions: Vec<String>,
}

impl ErrorRecoveryManager {
    /// create new error recovery manager
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

    /// handle export error and attempt recovery
    pub fn handle_export_error(
        &mut self,
        error: &ExportError,
        operation: &str,
        context: &ErrorContext,
    ) -> TrackingResult<RecoveryResult> {
        let recovery_start = Instant::now();
        self.stats.total_errors += 1;

        if self.config.verbose_logging {
            println!("🔧 Error recovery: {} - {}", operation, error);
        }

        // select recovery strategy
        let strategy = self.select_recovery_strategy(error, operation, context)?;

        // execute recovery strategy
        let result = self.execute_recovery_strategy(strategy, error, operation, context)?;

        // update statistics
        let recovery_time = recovery_start.elapsed().as_millis() as u64;
        self.stats.total_recovery_time_ms += recovery_time;

        if result.success {
            self.stats.successful_recoveries += 1;
        } else {
            self.stats.failed_recoveries += 1;
        }

        // update degradation state
        self.update_degradation_state(error, &result);

        if self.config.verbose_logging {
            println!(
                "🔧 Recovery completed: {} ({}ms)",
                result.message, recovery_time
            );
        }

        Ok(result)
    }

    /// select recovery strategy
    fn select_recovery_strategy(
        &self,
        error: &ExportError,
        operation: &str,
        context: &ErrorContext,
    ) -> TrackingResult<RecoveryStrategy> {
        match error {
            ExportError::ParallelProcessingError { shard_index, .. } => {
                // parallel processing error: try retry or degrade parallelism
                if self.should_retry(operation) {
                    Ok(RecoveryStrategy::AutoRetry {
                        max_attempts: self.config.max_retry_attempts,
                        interval_ms: self.config.retry_interval_ms,
                        backoff_factor: self.config.retry_backoff_factor,
                    })
                } else {
                    Ok(RecoveryStrategy::GracefulDegradation {
                        target_level: DegradationLevel::Light,
                        reason: format!(
                            "shard {shard_index} processing failed, degrade parallelism"
                        ),
                    })
                }
            }

            ExportError::ResourceLimitExceeded {
                resource_type,
                suggested_action,
                ..
            } => {
                // resource limit exceeded: release resources or adjust configuration
                match resource_type {
                    ResourceType::Memory => Ok(RecoveryStrategy::ConfigAdjustment {
                        new_config: self.create_memory_optimized_config(context),
                        reason: "memory limit exceeded, adjust to memory optimized configuration"
                            .to_string(),
                    }),
                    ResourceType::CPU => Ok(RecoveryStrategy::GracefulDegradation {
                        target_level: DegradationLevel::Moderate,
                        reason: "CPU usage exceeded, degrade processing intensity".to_string(),
                    }),
                    _ => Ok(RecoveryStrategy::ConfigAdjustment {
                        new_config: context.current_config.clone(),
                        reason: suggested_action.clone(),
                    }),
                }
            }

            ExportError::DataQualityError {
                affected_records, ..
            } => {
                // data quality error: save partial results
                if self.config.enable_partial_save && context.progress_percentage > 10.0 {
                    Ok(RecoveryStrategy::PartialSave {
                        save_path: self.generate_partial_save_path(operation),
                        progress_percentage: context.progress_percentage,
                    })
                } else {
                    Ok(RecoveryStrategy::SkipOperation {
                        operation: operation.to_string(),
                        reason: format!("data quality issue affects {affected_records} records, skip processing"),
                    })
                }
            }

            ExportError::PerformanceThresholdExceeded { stage, .. } => {
                // performance threshold exceeded: adjust configuration or degrade
                match stage {
                    ExportStage::ParallelProcessing => Ok(RecoveryStrategy::ConfigAdjustment {
                        new_config: self.create_performance_optimized_config(context),
                        reason: "performance threshold exceeded, adjust configuration".to_string(),
                    }),
                    _ => Ok(RecoveryStrategy::GracefulDegradation {
                        target_level: DegradationLevel::Light,
                        reason: format!(
                            "performance threshold exceeded in stage {stage:?}, degrade processing"
                        ),
                    }),
                }
            }

            ExportError::ConcurrencyConflict {
                conflict_type,
                retry_count,
                ..
            } => {
                // concurrency conflict: retry or adjust concurrency strategy
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
                        reason: "concurrency conflict, degrade processing".to_string(),
                    })
                }
            }

            ExportError::InsufficientResources { .. } => {
                // insufficient resources: emergency degradation
                Ok(RecoveryStrategy::GracefulDegradation {
                    target_level: DegradationLevel::Emergency,
                    reason: "system resources severely insufficient, enable emergency mode"
                        .to_string(),
                })
            }

            ExportError::ExportInterrupted {
                progress_percentage,
                ..
            } => {
                // export interrupted: save partial results
                Ok(RecoveryStrategy::PartialSave {
                    save_path: self.generate_partial_save_path(operation),
                    progress_percentage: *progress_percentage,
                })
            }

            ExportError::DataCorruption {
                recovery_possible, ..
            } => {
                // data corruption: determine strategy based on whether recovery is possible
                if *recovery_possible {
                    Ok(RecoveryStrategy::AutoRetry {
                        max_attempts: 1, // only retry once
                        interval_ms: self.config.retry_interval_ms,
                        backoff_factor: 1.0,
                    })
                } else {
                    Ok(RecoveryStrategy::SkipOperation {
                        operation: operation.to_string(),
                        reason: "data corruption and cannot be recovered, skip operation"
                            .to_string(),
                    })
                }
            }
        }
    }

    /// execute recovery strategy
    fn execute_recovery_strategy(
        &mut self,
        strategy: RecoveryStrategy,
        _error: &ExportError,
        operation: &str,
        context: &ErrorContext,
    ) -> TrackingResult<RecoveryResult> {
        let _execution_start = Instant::now();

        match strategy {
            RecoveryStrategy::AutoRetry {
                max_attempts,
                interval_ms,
                backoff_factor,
            } => self.execute_auto_retry(operation, max_attempts, interval_ms, backoff_factor),

            RecoveryStrategy::GracefulDegradation {
                target_level,
                reason,
            } => self.execute_graceful_degradation(target_level, reason),

            RecoveryStrategy::PartialSave {
                save_path,
                progress_percentage,
            } => self.execute_partial_save(save_path, progress_percentage, context),

            RecoveryStrategy::ConfigAdjustment { new_config, reason } => {
                self.execute_config_adjustment(new_config, reason)
            }

            RecoveryStrategy::ResourceRelease {
                resource_type,
                amount,
            } => self.execute_resource_release(resource_type, amount),

            RecoveryStrategy::SkipOperation { operation, reason } => {
                self.execute_skip_operation(operation, reason)
            }
        }
    }

    /// execute auto retry
    fn execute_auto_retry(
        &mut self,
        operation: &str,
        max_attempts: usize,
        interval_ms: u64,
        backoff_factor: f64,
    ) -> TrackingResult<RecoveryResult> {
        let history = self
            .retry_history
            .entry(operation.to_string())
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
                strategy: RecoveryStrategy::AutoRetry {
                    max_attempts,
                    interval_ms,
                    backoff_factor,
                },
                message: format!("reached maximum retry limit ({max_attempts})"),
                recovery_time_ms: 0,
                partial_result_path: None,
                suggested_actions: vec![
                    "consider manual intervention or configuration adjustment".to_string()
                ],
            });
        }

        // 等待重试间隔
        if history.attempt_count > 0 {
            std::thread::sleep(Duration::from_millis(history.next_interval_ms));
        }

        history.attempt_count += 1;
        history.last_attempt = Instant::now();
        history.next_interval_ms = (history.next_interval_ms as f64 * backoff_factor) as u64;
        history.next_interval_ms = history
            .next_interval_ms
            .min(self.config.max_retry_interval_ms);

        self.stats.total_retries += 1;

        Ok(RecoveryResult {
            success: true,
            strategy: RecoveryStrategy::AutoRetry {
                max_attempts,
                interval_ms,
                backoff_factor,
            },
            message: format!(
                "prepare for retry {} (max {})",
                history.attempt_count, max_attempts
            ),
            recovery_time_ms: history.next_interval_ms,
            partial_result_path: None,
            suggested_actions: vec![
                "monitor retry results".to_string(),
                "if keep failed, consider adjustment strategy".to_string(),
            ],
        })
    }

    /// execute graceful degradation
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
            strategy: RecoveryStrategy::GracefulDegradation {
                target_level,
                reason,
            },
            message: message.to_string(),
            recovery_time_ms: 0,
            partial_result_path: None,
            suggested_actions: vec![
                "monitor system status".to_string(),
                "in conditions improve, consider normal mode".to_string(),
            ],
        })
    }

    /// execute partial save
    fn execute_partial_save(
        &mut self,
        save_path: PathBuf,
        progress_percentage: f64,
        context: &ErrorContext,
    ) -> TrackingResult<RecoveryResult> {
        // create partial save directory
        if let Some(parent) = save_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                TrackingError::IoError(format!("create partial save directory failed: {e}"))
            })?;
        }

        // save partial results (here is a simplified implementation)
        let partial_data = format!(
            "{{\"partial_export\":true,\"progress\":{progress_percentage},\"timestamp\":\"{}\",\"context\":\"unknown\"}}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );

        std::fs::write(&save_path, partial_data)
            .map_err(|e| TrackingError::IoError(format!("save partial results failed: {e}")))?;

        self.stats.partial_saves += 1;

        Ok(RecoveryResult {
            success: true,
            strategy: RecoveryStrategy::PartialSave {
                save_path: save_path.clone(),
                progress_percentage,
            },
            message: format!("partial results saved ({progress_percentage:.1}% completed)"),
            recovery_time_ms: 0,
            partial_result_path: Some(save_path),
            suggested_actions: vec![
                "check partial result file".to_string(),
                "resume from here after fixing the issue".to_string(),
            ],
        })
    }

    /// execute config adjustment
    fn execute_config_adjustment(
        &self,
        new_config: FastExportConfig,
        reason: String,
    ) -> TrackingResult<RecoveryResult> {
        Ok(RecoveryResult {
            success: true,
            strategy: RecoveryStrategy::ConfigAdjustment {
                new_config,
                reason: reason.clone(),
            },
            message: format!("config adjusted: {reason}"),
            recovery_time_ms: 0,
            partial_result_path: None,
            suggested_actions: vec![
                "use new config to retry export".to_string(),
                "monitor new config effects".to_string(),
            ],
        })
    }

    /// execute resource release
    fn execute_resource_release(
        &self,
        resource_type: ResourceType,
        amount: usize,
    ) -> TrackingResult<RecoveryResult> {
        // here is a simplified implementation, should call system API to release resources in actual use
        let message = match resource_type {
            ResourceType::Memory => format!("try to release {amount} bytes of memory"),
            ResourceType::CPU => format!("reduce CPU usage by {amount}%"),
            ResourceType::Disk => format!("clean up {amount} bytes of disk space"),
            ResourceType::FileHandles => format!("close {amount} file handles"),
            ResourceType::ThreadPool => format!("reduce {amount} threads"),
        };

        Ok(RecoveryResult {
            success: true,
            strategy: RecoveryStrategy::ResourceRelease {
                resource_type,
                amount,
            },
            message,
            recovery_time_ms: 0,
            partial_result_path: None,
            suggested_actions: vec![
                "monitor resource usage".to_string(),
                "retry failed operation".to_string(),
            ],
        })
    }

    /// execute skip operation
    fn execute_skip_operation(
        &self,
        operation: String,
        reason: String,
    ) -> TrackingResult<RecoveryResult> {
        Ok(RecoveryResult {
            success: true,
            strategy: RecoveryStrategy::SkipOperation {
                operation: operation.clone(),
                reason: reason.clone(),
            },
            message: format!("skip operation '{operation}': {reason}"),
            recovery_time_ms: 0,
            partial_result_path: None,
            suggested_actions: vec![
                "check the impact of skipping the operation".to_string(),
                "consider manually handling the skipped part".to_string(),
            ],
        })
    }

    /// check if should retry
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

    /// update degradation state
    fn update_degradation_state(&mut self, error: &ExportError, _result: &RecoveryResult) {
        // simplified error rate calculation
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

        // update error rate (simplified calculation)
        self.degradation_state.current_error_rate =
            (self.degradation_state.current_error_rate * 0.9) + (error_weight * 0.1);

        // check if should degrade or recover
        if !self.degradation_state.is_degraded
            && self.degradation_state.current_error_rate > self.config.degradation_threshold
        {
            // trigger degradation
            self.degradation_state.is_degraded = true;
            self.degradation_state.degradation_start = Some(Instant::now());
            self.degradation_state.degradation_level = DegradationLevel::Light;
        } else if self.degradation_state.is_degraded
            && self.degradation_state.current_error_rate < self.config.recovery_threshold
        {
            // trigger recovery
            self.degradation_state.is_degraded = false;
            self.degradation_state.degradation_start = None;
            self.degradation_state.degradation_level = DegradationLevel::Normal;
            self.degradation_state.degradation_reason = None;
        }
    }

    /// generate partial save path
    fn generate_partial_save_path(&self, operation: &str) -> PathBuf {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let filename = format!("partial_export_{operation}_{timestamp}.json");
        self.config.partial_save_directory.join(filename)
    }

    /// create memory optimized config
    fn create_memory_optimized_config(&self, context: &ErrorContext) -> FastExportConfig {
        let mut config = context.current_config.clone();

        // reduce parallelism
        config.shard_config.max_threads = Some(2);
        config.shard_config.shard_size = config.shard_config.shard_size / 2;

        // reduce buffer size
        config.writer_config.buffer_size = config.writer_config.buffer_size / 2;

        // enable streaming
        config.enable_data_localization = false;

        config
    }

    /// create performance optimized config
    fn create_performance_optimized_config(&self, context: &ErrorContext) -> FastExportConfig {
        let mut config = context.current_config.clone();

        // adjust shard size
        config.shard_config.shard_size = 500; // smaller shard size
        config.shard_config.parallel_threshold = 1000; // lower parallel threshold

        // disable verbose logging
        config.verbose_logging = false;
        config.enable_performance_monitoring = false;

        config
    }

    /// get recovery stats
    pub fn get_stats(&self) -> &RecoveryStats {
        &self.stats
    }

    /// get degradation state
    pub fn get_degradation_state(&self) -> &DegradationState {
        &self.degradation_state
    }

    /// generate recovery report
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

/// error context
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// current config
    pub current_config: FastExportConfig,
    /// progress percentage
    pub progress_percentage: f64,
    /// processed data size
    pub processed_data_size: usize,
    /// operation start time
    pub operation_start_time: Instant,
    /// current export stats
    pub current_stats: Option<CompleteExportStats>,
}

/// recovery report
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
    /// print detailed recovery report
    pub fn print_detailed_report(&self) {
        println!("\n🔧 recovery report");
        println!("================");

        println!("📊 total statistics:");
        println!("   total errors: {}", self.total_errors);
        println!(
            "   successful recoveries: {} ({:.1}%)",
            self.successful_recoveries, self.success_rate
        );
        println!("   failed recoveries: {}", self.failed_recoveries);
        println!("   total retries: {}", self.total_retries);
        println!("   degradation count: {}", self.degradation_count);
        println!("   partial saves: {}", self.partial_saves);
        println!(
            "   average recovery time: {:.2}ms",
            self.avg_recovery_time_ms
        );

        println!("\n🎚️ current state:");
        println!("   degradation level: {:?}", self.current_degradation_level);
        println!(
            "   is degraded: {}",
            if self.is_currently_degraded {
                "yes"
            } else {
                "no"
            }
        );
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

        let result =
            manager.execute_graceful_degradation(DegradationLevel::Light, "测试降级".to_string());

        assert!(result.is_ok());
        let recovery_result = result.unwrap();
        assert!(recovery_result.success);
        assert!(manager.degradation_state.is_degraded);
        assert_eq!(
            manager.degradation_state.degradation_level,
            DegradationLevel::Light
        );
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

        // clean up test file
        let _ = std::fs::remove_file(save_path);
    }

    #[test]
    fn test_recovery_report() {
        let mut manager = ErrorRecoveryManager::new(RecoveryConfig::default());

        // simulate some errors and recoveries
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

        // first retry should succeed
        assert!(manager.should_retry("test_operation"));

        // simulate multiple retries
        for i in 0..3 {
            let result = manager.execute_auto_retry("test_operation", 3, 100, 2.0);
            assert!(result.is_ok());
            let recovery_result = result.unwrap();
            assert!(recovery_result.success);
        }

        // should fail after max retries
        let result = manager.execute_auto_retry("test_operation", 3, 100, 2.0);
        assert!(result.is_ok());
        let recovery_result = result.unwrap();
        assert!(!recovery_result.success);
    }
}
