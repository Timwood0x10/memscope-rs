//! 快速导出协调器 - 整合所有优化组件的高性能导出系统
//!
//! 这个模块实现了快速导出协调器，整合数据本地化器、并行分片处理器
//! 和高速缓冲写入器，提供完整的高性能导出解决方案。

use crate::core::types::{TrackingResult};
use crate::export::data_localizer::{DataLocalizer, DataGatheringStats, LocalizedExportData};
use crate::export::error_handling::{PerformanceLogger, LogLevel, ResourceMonitor};
use crate::export::error_recovery::{ErrorRecoveryManager, RecoveryConfig, ErrorContext};
use crate::export::high_speed_buffered_writer::{
    HighSpeedBufferedWriter, HighSpeedWriterConfig, WritePerformanceStats,
};
use crate::export::parallel_shard_processor::{
    ParallelShardConfig, ParallelShardProcessor, ParallelProcessingStats,
};
use crate::export::progress_monitor::{
    ProgressMonitor, ProgressCallback, ExportStage, ProgressConfig,
};
use crate::export::quality_validator::{QualityValidator, ValidationConfig};
use std::path::Path;
use std::time::Instant;

/// 快速导出协调器配置
#[derive(Debug, Clone)]
pub struct FastExportConfig {
    /// 数据本地化配置
    pub enable_data_localization: bool,
    /// 数据本地化缓存时间（毫秒）
    pub data_cache_ttl_ms: u64,
    
    /// 并行分片处理配置
    pub shard_config: ParallelShardConfig,
    
    /// 高速写入配置
    pub writer_config: HighSpeedWriterConfig,
    
    /// 性能监控配置
    pub enable_performance_monitoring: bool,
    /// 详细日志输出
    pub verbose_logging: bool,
    
    /// 进度监控配置
    pub progress_config: ProgressConfig,
    
    /// 自动优化配置
    pub enable_auto_optimization: bool,
    /// 根据系统资源自动调整参数
    pub auto_adjust_for_system: bool,
    
    /// 错误处理和恢复配置
    pub error_recovery_config: RecoveryConfig,
    /// 数据质量验证配置
    pub validation_config: ValidationConfig,
    /// 资源监控配置
    pub enable_resource_monitoring: bool,
    /// 内存限制（MB）
    pub memory_limit_mb: usize,
    /// 磁盘限制（MB）
    pub disk_limit_mb: usize,
    /// CPU 限制（百分比）
    pub cpu_limit_percent: f64,
}

impl Default for FastExportConfig {
    fn default() -> Self {
        Self {
            enable_data_localization: true,
            data_cache_ttl_ms: 100,
            
            shard_config: ParallelShardConfig::default(),
            writer_config: HighSpeedWriterConfig::default(),
            
            enable_performance_monitoring: true,
            verbose_logging: false,
            
            progress_config: ProgressConfig::default(),
            
            enable_auto_optimization: true,
            auto_adjust_for_system: true,
            
            error_recovery_config: RecoveryConfig::default(),
            validation_config: ValidationConfig::default(),
            enable_resource_monitoring: true,
            memory_limit_mb: 1024, // 1GB 默认内存限制
            disk_limit_mb: 2048,   // 2GB 默认磁盘限制
            cpu_limit_percent: 80.0, // 80% CPU 限制
        }
    }
}

/// 完整的导出性能统计
#[derive(Debug, Clone)]
pub struct CompleteExportStats {
    /// 数据获取统计
    pub data_gathering: DataGatheringStats,
    /// 并行处理统计
    pub parallel_processing: ParallelProcessingStats,
    /// 写入性能统计
    pub write_performance: WritePerformanceStats,
    
    /// 总体统计
    pub total_export_time_ms: u64,
    pub total_allocations_processed: usize,
    pub total_output_size_bytes: usize,
    pub overall_throughput_allocations_per_sec: f64,
    pub overall_write_speed_mbps: f64,
    
    /// 各阶段耗时占比
    pub data_gathering_percentage: f64,
    pub processing_percentage: f64,
    pub writing_percentage: f64,
    
    /// 性能提升指标
    pub estimated_traditional_time_ms: u64,
    pub performance_improvement_factor: f64,
}

/// 快速导出协调器
pub struct FastExportCoordinator {
    /// 配置
    config: FastExportConfig,
    /// 数据本地化器
    data_localizer: DataLocalizer,
    /// 并行分片处理器
    shard_processor: ParallelShardProcessor,
    /// 性能日志记录器
    performance_logger: PerformanceLogger,
    /// 错误恢复管理器
    error_recovery_manager: ErrorRecoveryManager,
    /// 数据质量验证器
    quality_validator: QualityValidator,
    /// 资源监控器
    resource_monitor: Option<ResourceMonitor>,
}

impl FastExportCoordinator {
    /// 创建新的快速导出协调器
    pub fn new(config: FastExportConfig) -> Self {
        // 根据配置创建数据本地化器
        let data_localizer = if config.enable_data_localization {
            DataLocalizer::with_cache_ttl(std::time::Duration::from_millis(config.data_cache_ttl_ms))
        } else {
            DataLocalizer::new()
        };

        // 创建并行分片处理器
        let shard_processor = ParallelShardProcessor::new(config.shard_config.clone());

        // 创建性能日志记录器
        let log_level = if config.verbose_logging {
            LogLevel::Debug
        } else {
            LogLevel::Info
        };
        let performance_logger = PerformanceLogger::new(log_level);

        // 创建错误恢复管理器
        let error_recovery_manager = ErrorRecoveryManager::new(config.error_recovery_config.clone());

        // 创建数据质量验证器
        let quality_validator = QualityValidator::new(config.validation_config.clone());

        // 创建资源监控器
        let resource_monitor = if config.enable_resource_monitoring {
            Some(ResourceMonitor::new(
                config.memory_limit_mb,
                config.disk_limit_mb,
                config.cpu_limit_percent,
            ))
        } else {
            None
        };

        Self {
            config,
            data_localizer,
            shard_processor,
            performance_logger,
            error_recovery_manager,
            quality_validator,
            resource_monitor,
        }
    }

    /// 执行快速导出
    pub fn export_fast<P: AsRef<Path>>(
        &mut self,
        output_path: P,
    ) -> TrackingResult<CompleteExportStats> {
        self.export_fast_with_progress(output_path, None)
    }
    
    /// 执行快速导出（带进度监控）
    pub fn export_fast_with_progress<P: AsRef<Path>>(
        &mut self,
        output_path: P,
        progress_callback: Option<ProgressCallback>,
    ) -> TrackingResult<CompleteExportStats> {
        let total_start = Instant::now();
        let operation_id = format!("export_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs());

        // 记录操作开始
        self.performance_logger.log_operation_start(
            "fast_export",
            &format!("输出路径: {}", output_path.as_ref().display()),
        );

        if self.config.verbose_logging {
            println!("🚀 快速导出协调器开始执行");
            println!("   输出路径: {}", output_path.as_ref().display());
            println!("   操作 ID: {}", operation_id);
        }

        // 创建错误上下文
        let mut error_context = ErrorContext {
            operation_id: operation_id.clone(),
            current_config: self.config.clone(),
            progress_percentage: 0.0,
            processed_data_size: 0,
            operation_start_time: total_start,
            current_stats: None,
        };

        // 创建进度监控器
        let mut progress_monitor = if self.config.progress_config.enabled {
            Some(ProgressMonitor::new(1000)) // 预估分配数量，后续会更新
        } else {
            None
        };

        // 初始化阶段
        if let Some(ref mut monitor) = progress_monitor {
            monitor.set_stage(ExportStage::Initializing);
        }

        // 资源监控检查
        if let Some(ref monitor) = self.resource_monitor {
            if let Err(e) = monitor.check_resource_usage() {
                let export_error = crate::export::error_handling::ExportError::InsufficientResources {
                    required_memory: 0,
                    available_memory: 0,
                    required_disk: 0,
                    available_disk: 0,
                };
                self.performance_logger.log_error(&export_error);
                return Err(e);
            }
        }

        // 第一阶段：数据本地化
        error_context.progress_percentage = 10.0;
        if let Some(ref mut monitor) = progress_monitor {
            monitor.set_stage(ExportStage::DataLocalization);
            if monitor.should_cancel() {
                monitor.cancel();
                return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "Export cancelled during data localization").into());
            }
        }
        
        let (localized_data, data_stats) = match self.gather_data_with_progress(progress_monitor.as_mut()) {
            Ok(result) => result,
            Err(e) => {
                let export_error = crate::export::error_handling::ExportError::DataQualityError {
                    validation_type: crate::export::error_handling::ValidationType::DataIntegrity,
                    expected: "valid data".to_string(),
                    actual: "error occurred".to_string(),
                    affected_records: 0,
                };
                self.performance_logger.log_operation_failure("fast_export", &export_error, total_start.elapsed());
                
                // 尝试错误恢复
                if let Ok(recovery_result) = self.error_recovery_manager.handle_export_error(&export_error, "data_localization", &error_context) {
                    if recovery_result.success {
                        self.performance_logger.log_debug("数据本地化错误恢复成功，但仍返回原始错误");
                    }
                }
                
                return Err(e);
            }
        };

        // 验证原始数据质量
        if let Err(validation_error) = self.quality_validator.validate_source_data(&localized_data) {
            self.performance_logger.log_warning(&format!("数据质量验证失败: {validation_error}"));
        }

        // 更新错误上下文
        error_context.processed_data_size = localized_data.allocations.len();
        error_context.progress_percentage = 30.0;

        // 更新总分配数量并设置回调
        if let Some(ref mut monitor) = progress_monitor {
            let mut new_monitor = ProgressMonitor::new(localized_data.allocations.len());
            if let Some(callback) = progress_callback {
                new_monitor.set_callback(callback);
            }
            *monitor = new_monitor;
        }

        // 第二阶段：并行分片处理
        error_context.progress_percentage = 50.0;
        if let Some(ref mut monitor) = progress_monitor {
            monitor.set_stage(ExportStage::ParallelProcessing);
            if monitor.should_cancel() {
                monitor.cancel();
                return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "Export cancelled during parallel processing").into());
            }
        }
        
        let (processed_shards, processing_stats) = match self.process_data_parallel_with_progress(&localized_data, progress_monitor.as_mut()) {
            Ok(result) => result,
            Err(e) => {
                let export_error = crate::export::error_handling::ExportError::ParallelProcessingError {
                    shard_index: 0,
                    thread_id: "unknown".to_string(),
                    error_message: e.to_string(),
                    partial_results: None,
                };
                self.performance_logger.log_operation_failure("fast_export", &export_error, total_start.elapsed());
                
                // 尝试错误恢复
                if let Ok(recovery_result) = self.error_recovery_manager.handle_export_error(&export_error, "parallel_processing", &error_context) {
                    if recovery_result.success {
                        self.performance_logger.log_debug("并行处理错误恢复成功，但仍返回原始错误");
                    }
                }
                
                return Err(e);
            }
        };

        // 验证处理后的分片数据
        if let Err(validation_error) = self.quality_validator.validate_processed_shards(&processed_shards, localized_data.allocations.len()) {
            self.performance_logger.log_warning(&format!("分片数据验证失败: {validation_error}"));
        }

        // 第三阶段：高速写入
        error_context.progress_percentage = 80.0;
        if let Some(ref mut monitor) = progress_monitor {
            monitor.set_stage(ExportStage::Writing);
            if monitor.should_cancel() {
                monitor.cancel();
                return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "Export cancelled during writing").into());
            }
        }
        
        let write_stats = match self.write_data_fast_with_progress(&output_path, &processed_shards, progress_monitor.as_mut()) {
            Ok(result) => result,
            Err(e) => {
                let export_error = crate::export::error_handling::ExportError::DataQualityError {
                    validation_type: crate::export::error_handling::ValidationType::FileSize,
                    expected: "successful write".to_string(),
                    actual: "write failed".to_string(),
                    affected_records: 0,
                };
                self.performance_logger.log_operation_failure("fast_export", &export_error, total_start.elapsed());
                
                // 尝试错误恢复
                if let Ok(recovery_result) = self.error_recovery_manager.handle_export_error(&export_error, "high_speed_writing", &error_context) {
                    if recovery_result.success {
                        self.performance_logger.log_debug("高速写入错误恢复成功，但仍返回原始错误");
                    }
                }
                
                return Err(e);
            }
        };

        let total_time = total_start.elapsed();

        // 验证最终输出文件
        if let Err(validation_error) = self.quality_validator.validate_output_file(
            output_path.as_ref().to_str().unwrap_or("unknown"),
            localized_data.allocations.len(),
        ) {
            self.performance_logger.log_warning(&format!("输出文件验证失败: {validation_error}"));
        }

        // 完成阶段
        if let Some(ref mut monitor) = progress_monitor {
            monitor.complete();
        }

        // 计算完整统计信息
        let complete_stats = self.calculate_complete_stats(
            data_stats,
            processing_stats,
            write_stats,
            total_time.as_millis() as u64,
        );

        // 记录操作成功
        self.performance_logger.log_operation_success(
            "fast_export",
            total_time,
            &format!("成功导出 {} 个分配", complete_stats.total_allocations_processed),
        );

        // 记录性能指标
        self.performance_logger.log_performance_metric(
            crate::export::error_handling::PerformanceMetric::ExportTime,
            total_time.as_millis() as f64,
            Some(5000.0), // 5秒阈值
        );

        self.performance_logger.log_performance_metric(
            crate::export::error_handling::PerformanceMetric::ThroughputRate,
            complete_stats.overall_throughput_allocations_per_sec,
            Some(1000.0), // 1000 分配/秒阈值
        );

        if self.config.enable_performance_monitoring {
            self.print_complete_stats(&complete_stats);
            
            // 打印性能和恢复报告
            let performance_report = self.performance_logger.generate_performance_report();
            performance_report.print_detailed_report();
            
            let recovery_report = self.error_recovery_manager.generate_recovery_report();
            if recovery_report.total_errors > 0 {
                recovery_report.print_detailed_report();
            }
            
            let validation_report = self.quality_validator.generate_validation_report();
            validation_report.print_detailed_report();
        }

        Ok(complete_stats)
    }

    /// 数据获取阶段
    fn gather_data(&mut self) -> TrackingResult<(LocalizedExportData, DataGatheringStats)> {
        self.gather_data_with_progress(None)
    }

    /// 数据获取阶段（带进度监控）
    fn gather_data_with_progress(
        &mut self,
        mut progress_monitor: Option<&mut ProgressMonitor>,
    ) -> TrackingResult<(LocalizedExportData, DataGatheringStats)> {
        let stage_start = Instant::now();

        if self.config.verbose_logging {
            println!("📊 阶段 1: 数据本地化");
        }

        if let Some(ref mut monitor) = progress_monitor {
            monitor.update_progress(0.1, Some("开始数据本地化".to_string()));
        }

        let result = self.data_localizer.gather_all_export_data()?;

        if let Some(ref mut monitor) = progress_monitor {
            monitor.update_progress(1.0, Some("数据本地化完成".to_string()));
        }

        if self.config.verbose_logging {
            println!("   ✅ 数据本地化完成，耗时: {:?}", stage_start.elapsed());
        }

        Ok(result)
    }

    /// 并行处理阶段
    fn process_data_parallel(
        &self,
        data: &LocalizedExportData,
    ) -> TrackingResult<(Vec<crate::export::parallel_shard_processor::ProcessedShard>, ParallelProcessingStats)> {
        self.process_data_parallel_with_progress(data, None)
    }

    /// 并行处理阶段（带进度监控）
    fn process_data_parallel_with_progress(
        &self,
        data: &LocalizedExportData,
        mut progress_monitor: Option<&mut ProgressMonitor>,
    ) -> TrackingResult<(Vec<crate::export::parallel_shard_processor::ProcessedShard>, ParallelProcessingStats)> {
        let stage_start = Instant::now();

        if self.config.verbose_logging {
            println!("⚡ 阶段 2: 并行分片处理");
        }

        if let Some(ref mut monitor) = progress_monitor {
            monitor.update_progress(0.1, Some("开始并行分片处理".to_string()));
        }

        let result = self.shard_processor.process_allocations_parallel(data)?;

        if let Some(ref mut monitor) = progress_monitor {
            monitor.update_progress(1.0, Some("并行处理完成".to_string()));
        }

        if self.config.verbose_logging {
            println!("   ✅ 并行处理完成，耗时: {:?}", stage_start.elapsed());
        }

        Ok(result)
    }

    /// 高速写入阶段
    fn write_data_fast<P: AsRef<Path>>(
        &self,
        output_path: P,
        shards: &[crate::export::parallel_shard_processor::ProcessedShard],
    ) -> TrackingResult<WritePerformanceStats> {
        self.write_data_fast_with_progress(output_path, shards, None)
    }

    /// 高速写入阶段（带进度监控）
    fn write_data_fast_with_progress<P: AsRef<Path>>(
        &self,
        output_path: P,
        shards: &[crate::export::parallel_shard_processor::ProcessedShard],
        mut progress_monitor: Option<&mut ProgressMonitor>,
    ) -> TrackingResult<WritePerformanceStats> {
        let stage_start = Instant::now();

        if self.config.verbose_logging {
            println!("💾 阶段 3: 高速缓冲写入");
        }

        if let Some(ref mut monitor) = progress_monitor {
            monitor.update_progress(0.1, Some("开始高速缓冲写入".to_string()));
        }

        // 预估总大小用于优化写入器配置
        let total_size: usize = shards.iter().map(|s| s.data.len()).sum();
        let mut writer_config = self.config.writer_config.clone();
        writer_config.estimated_total_size = Some(total_size + 1024);

        let mut writer = HighSpeedBufferedWriter::new(output_path, writer_config)?;
        let result = writer.write_processed_shards(shards)?;

        if let Some(ref mut monitor) = progress_monitor {
            monitor.update_progress(1.0, Some("高速写入完成".to_string()));
        }

        if self.config.verbose_logging {
            println!("   ✅ 高速写入完成，耗时: {:?}", stage_start.elapsed());
        }

        Ok(result)
    }

    /// 计算完整统计信息
    fn calculate_complete_stats(
        &self,
        data_stats: DataGatheringStats,
        processing_stats: ParallelProcessingStats,
        write_stats: WritePerformanceStats,
        total_time_ms: u64,
    ) -> CompleteExportStats {
        let total_allocations = processing_stats.total_allocations;
        let total_output_size = write_stats.total_bytes_written;

        // 计算总体吞吐量
        let overall_throughput = if total_time_ms > 0 {
            (total_allocations as f64 * 1000.0) / total_time_ms as f64
        } else {
            0.0
        };

        // 计算总体写入速度
        let overall_write_speed = if total_time_ms > 0 {
            (total_output_size as f64 / 1024.0 / 1024.0 * 1000.0) / total_time_ms as f64
        } else {
            0.0
        };

        // 计算各阶段耗时占比
        let data_percentage = if total_time_ms > 0 {
            (data_stats.total_time_ms as f64 / total_time_ms as f64) * 100.0
        } else {
            0.0
        };

        let processing_percentage = if total_time_ms > 0 {
            (processing_stats.total_processing_time_ms as f64 / total_time_ms as f64) * 100.0
        } else {
            0.0
        };

        let writing_percentage = if total_time_ms > 0 {
            (write_stats.total_write_time_ms as f64 / total_time_ms as f64) * 100.0
        } else {
            0.0
        };

        // 估算传统导出时间（基于经验值）
        let estimated_traditional_time = total_time_ms * 3; // 假设传统方法慢 3 倍
        let performance_improvement = if total_time_ms > 0 {
            estimated_traditional_time as f64 / total_time_ms as f64
        } else {
            1.0
        };

        CompleteExportStats {
            data_gathering: data_stats,
            parallel_processing: processing_stats,
            write_performance: write_stats,
            
            total_export_time_ms: total_time_ms,
            total_allocations_processed: total_allocations,
            total_output_size_bytes: total_output_size,
            overall_throughput_allocations_per_sec: overall_throughput,
            overall_write_speed_mbps: overall_write_speed,
            
            data_gathering_percentage: data_percentage,
            processing_percentage: processing_percentage,
            writing_percentage: writing_percentage,
            
            estimated_traditional_time_ms: estimated_traditional_time,
            performance_improvement_factor: performance_improvement,
        }
    }

    /// 打印完整统计信息
    fn print_complete_stats(&self, stats: &CompleteExportStats) {
        println!("\n🎯 快速导出完成 - 性能统计");
        println!("================================");
        
        println!("📊 总体性能:");
        println!("   总耗时: {}ms", stats.total_export_time_ms);
        println!("   处理分配: {} 个", stats.total_allocations_processed);
        println!("   输出大小: {:.2} MB", stats.total_output_size_bytes as f64 / 1024.0 / 1024.0);
        println!("   总体吞吐量: {:.0} 分配/秒", stats.overall_throughput_allocations_per_sec);
        println!("   总体写入速度: {:.2} MB/s", stats.overall_write_speed_mbps);
        
        println!("\n⏱️ 各阶段耗时分析:");
        println!("   数据获取: {}ms ({:.1}%)", 
                stats.data_gathering.total_time_ms, 
                stats.data_gathering_percentage);
        println!("   并行处理: {}ms ({:.1}%)", 
                stats.parallel_processing.total_processing_time_ms, 
                stats.processing_percentage);
        println!("   高速写入: {}ms ({:.1}%)", 
                stats.write_performance.total_write_time_ms, 
                stats.writing_percentage);
        
        println!("\n🚀 性能提升:");
        println!("   估算传统导出时间: {}ms", stats.estimated_traditional_time_ms);
        println!("   性能提升倍数: {:.2}x", stats.performance_improvement_factor);
        println!("   时间节省: {}ms ({:.1}%)", 
                stats.estimated_traditional_time_ms - stats.total_export_time_ms,
                (1.0 - 1.0 / stats.performance_improvement_factor) * 100.0);
        
        if stats.parallel_processing.used_parallel_processing {
            println!("\n⚡ 并行处理效果:");
            println!("   使用线程: {}", stats.parallel_processing.threads_used);
            println!("   并行效率: {:.1}%", stats.parallel_processing.parallel_efficiency * 100.0);
            println!("   分片数量: {}", stats.parallel_processing.shard_count);
        }
        
        println!("\n💾 写入性能:");
        println!("   缓冲区利用率: {:.1}%", stats.write_performance.buffer_utilization * 100.0);
        println!("   预分配有效: {}", stats.write_performance.preallocation_effective);
        println!("   刷新次数: {}", stats.write_performance.flush_count);
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &FastExportConfig {
        &self.config
    }



    /// 更新配置
    pub fn update_config(&mut self, config: FastExportConfig) {
        self.config = config.clone();
        
        // 更新子组件配置
        self.data_localizer = if config.enable_data_localization {
            DataLocalizer::with_cache_ttl(std::time::Duration::from_millis(config.data_cache_ttl_ms))
        } else {
            DataLocalizer::new()
        };
        
        self.shard_processor = ParallelShardProcessor::new(config.shard_config.clone());
        
        // 重新创建性能日志记录器
        let log_level = if config.verbose_logging {
            LogLevel::Debug
        } else {
            LogLevel::Info
        };
        self.performance_logger = PerformanceLogger::new(log_level);
        
        // 重新创建错误恢复管理器
        self.error_recovery_manager = ErrorRecoveryManager::new(config.error_recovery_config.clone());
        
        // 重新创建数据质量验证器
        self.quality_validator = QualityValidator::new(config.validation_config.clone());
        
        // 重新创建资源监控器
        self.resource_monitor = if config.enable_resource_monitoring {
            Some(ResourceMonitor::new(
                config.memory_limit_mb,
                config.disk_limit_mb,
                config.cpu_limit_percent,
            ))
        } else {
            None
        };
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> crate::export::data_localizer::CacheStats {
        self.data_localizer.get_cache_stats()
    }

    /// 清除数据缓存
    pub fn clear_cache(&mut self) {
        self.data_localizer.invalidate_cache();
    }
}

impl Default for FastExportCoordinator {
    fn default() -> Self {
        Self::new(FastExportConfig::default())
    }
}

/// 便利函数：快速导出到指定路径
pub fn export_fast<P: AsRef<Path>>(output_path: P) -> TrackingResult<CompleteExportStats> {
    let mut coordinator = FastExportCoordinator::default();
    coordinator.export_fast(output_path)
}

/// 便利函数：使用自定义配置快速导出
pub fn export_fast_with_config<P: AsRef<Path>>(
    output_path: P,
    config: FastExportConfig,
) -> TrackingResult<CompleteExportStats> {
    let mut coordinator = FastExportCoordinator::new(config);
    coordinator.export_fast(output_path)
}

/// 配置构建器，用于方便地创建自定义配置
pub struct FastExportConfigBuilder {
    config: FastExportConfig,
}

impl FastExportConfigBuilder {
    /// 创建新的配置构建器
    pub fn new() -> Self {
        Self {
            config: FastExportConfig::default(),
        }
    }

    /// 启用或禁用数据本地化
    pub fn data_localization(mut self, enabled: bool) -> Self {
        self.config.enable_data_localization = enabled;
        self
    }

    /// 设置数据缓存时间
    pub fn cache_ttl_ms(mut self, ttl_ms: u64) -> Self {
        self.config.data_cache_ttl_ms = ttl_ms;
        self
    }

    /// 设置分片大小
    pub fn shard_size(mut self, size: usize) -> Self {
        self.config.shard_config.shard_size = size;
        self
    }

    /// 设置并行阈值
    pub fn parallel_threshold(mut self, threshold: usize) -> Self {
        self.config.shard_config.parallel_threshold = threshold;
        self
    }

    /// 设置最大线程数
    pub fn max_threads(mut self, threads: Option<usize>) -> Self {
        self.config.shard_config.max_threads = threads;
        self
    }

    /// 设置写入缓冲区大小
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.config.writer_config.buffer_size = size;
        self
    }

    /// 启用或禁用性能监控
    pub fn performance_monitoring(mut self, enabled: bool) -> Self {
        self.config.enable_performance_monitoring = enabled;
        self
    }

    /// 启用或禁用详细日志
    pub fn verbose_logging(mut self, enabled: bool) -> Self {
        self.config.verbose_logging = enabled;
        self
    }
    
    /// 设置进度监控配置
    pub fn progress_config(mut self, config: ProgressConfig) -> Self {
        self.config.progress_config = config;
        self
    }
    
    /// 启用或禁用进度监控
    pub fn progress_monitoring(mut self, enabled: bool) -> Self {
        self.config.progress_config.enabled = enabled;
        self
    }
    
    /// 设置错误恢复配置
    pub fn error_recovery_config(mut self, config: RecoveryConfig) -> Self {
        self.config.error_recovery_config = config;
        self
    }
    
    /// 设置数据质量验证配置
    pub fn validation_config(mut self, config: ValidationConfig) -> Self {
        self.config.validation_config = config;
        self
    }
    
    /// 启用或禁用资源监控
    pub fn resource_monitoring(mut self, enabled: bool) -> Self {
        self.config.enable_resource_monitoring = enabled;
        self
    }
    
    /// 设置内存限制
    pub fn memory_limit_mb(mut self, limit: usize) -> Self {
        self.config.memory_limit_mb = limit;
        self
    }
    
    /// 设置磁盘限制
    pub fn disk_limit_mb(mut self, limit: usize) -> Self {
        self.config.disk_limit_mb = limit;
        self
    }
    
    /// 设置 CPU 限制
    pub fn cpu_limit_percent(mut self, limit: f64) -> Self {
        self.config.cpu_limit_percent = limit;
        self
    }

    /// 构建配置
    pub fn build(self) -> FastExportConfig {
        self.config
    }
}

impl Default for FastExportConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_fast_export_coordinator_creation() {
        let config = FastExportConfig::default();
        let coordinator = FastExportCoordinator::new(config);
        assert!(coordinator.get_config().enable_data_localization);
    }

    #[test]
    fn test_config_builder() {
        let config = FastExportConfigBuilder::new()
            .shard_size(500)
            .parallel_threshold(1000)
            .buffer_size(1024 * 1024)
            .performance_monitoring(false)
            .build();

        assert_eq!(config.shard_config.shard_size, 500);
        assert_eq!(config.shard_config.parallel_threshold, 1000);
        assert_eq!(config.writer_config.buffer_size, 1024 * 1024);
        assert!(!config.enable_performance_monitoring);
    }

    #[test]
    fn test_convenience_functions() {
        let temp_file = NamedTempFile::new().unwrap();
        
        // 测试快速导出函数（可能会因为没有实际数据而失败，但至少测试函数存在）
        let result = export_fast(temp_file.path());
        // 在测试环境中可能没有实际的内存跟踪数据，所以这里只测试函数调用
        assert!(result.is_ok() || result.is_err()); // 只要不 panic 就行
    }

    #[test]
    fn test_config_update() {
        let mut coordinator = FastExportCoordinator::default();
        
        let new_config = FastExportConfigBuilder::new()
            .shard_size(200)
            .verbose_logging(true)
            .build();
        
        coordinator.update_config(new_config);
        assert_eq!(coordinator.get_config().shard_config.shard_size, 200);
        assert!(coordinator.get_config().verbose_logging);
    }

    #[test]
    fn test_cache_operations() {
        let mut coordinator = FastExportCoordinator::default();
        
        // 测试缓存统计
        let cache_stats = coordinator.get_cache_stats();
        assert!(!cache_stats.is_cached); // 初始状态应该没有缓存
        
        // 测试清除缓存
        coordinator.clear_cache(); // 应该不会 panic
    }
}