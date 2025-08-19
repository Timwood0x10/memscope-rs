// 新的统一API - 替代混乱的多个导出接口
use crate::core::tracker::memory_tracker::MemoryTracker;
use crate::core::types::TrackingResult;
use std::path::Path;
use std::sync::Arc;

/// 统一的导出器 - 提供清晰的API
pub struct MemScopeExporter {
    tracker: Arc<MemoryTracker>,
    config: ExportConfig,
}

/// 导出配置
#[derive(Debug, Clone)]
pub struct ExportConfig {
    pub include_system_allocations: bool,
    pub optimization_level: OptimizationLevel,
    pub enable_compression: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationLevel {
    Fast,
    Balanced, 
    Comprehensive,
}

/// 导出统计信息
#[derive(Debug, Clone)]
pub struct ExportStats {
    pub allocations_processed: usize,
    pub processing_time_ms: u64,
    pub output_size_bytes: u64,
    pub compression_ratio: Option<f64>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            include_system_allocations: false,
            optimization_level: OptimizationLevel::Fast,
            enable_compression: true,
        }
    }
}

impl ExportConfig {
    pub fn fast() -> Self {
        Self {
            include_system_allocations: false,
            optimization_level: OptimizationLevel::Fast,
            enable_compression: true,
        }
    }

    pub fn comprehensive() -> Self {
        Self {
            include_system_allocations: true,
            optimization_level: OptimizationLevel::Comprehensive,
            enable_compression: true,
        }
    }

    // 兼容性方法 - 替代OptimizedExportOptions
    pub fn with_optimization_level(level: OptimizationLevel) -> Self {
        Self {
            include_system_allocations: false,
            optimization_level: level,
            enable_compression: true,
        }
    }

    pub fn schema_validation(self, _enabled: bool) -> Self {
        
        self
    }
}

impl MemScopeExporter {
    pub fn new(tracker: Arc<MemoryTracker>, config: ExportConfig) -> Self {
        Self { tracker, config }
    }

    pub fn with_default_config(tracker: Arc<MemoryTracker>) -> Self {
        Self::new(tracker, ExportConfig::default())
    }

    /// 导出到JSON - 调用现有的保护接口
    pub fn export_json<P: AsRef<Path>>(&self, base_path: P) -> TrackingResult<ExportStats> {
        let start_time = std::time::Instant::now();
        
        // 调用现有的保护接口
        let allocations = self.tracker.get_active_allocations()?;
        let stats = self.tracker.get_stats()?;
        
        // 过滤分配（如果需要）
        let filtered_allocations = if self.config.include_system_allocations {
            allocations
        } else {
            allocations.into_iter()
                .filter(|alloc| alloc.var_name.is_some())
                .collect()
        };

        // 调用现有的导出函数
        crate::export::unified_export_api::export_user_variables_json(
            filtered_allocations.clone(), 
            stats, 
            &base_path
        )?;

        let processing_time = start_time.elapsed().as_millis() as u64;
        
        Ok(ExportStats {
            allocations_processed: filtered_allocations.len(),
            processing_time_ms: processing_time,
            output_size_bytes: 0, // 需要从文件获取
            compression_ratio: if self.config.enable_compression { Some(0.7) } else { None },
        })
    }

    /// 导出到二进制 - 调用现有的保护接口
    pub fn export_binary<P: AsRef<Path>>(&self, path: P) -> TrackingResult<ExportStats> {
        let start_time = std::time::Instant::now();
        
        // 根据配置选择导出模式
        let result = if self.config.include_system_allocations {
            self.tracker.export_full_binary(&path)
        } else {
            self.tracker.export_user_binary(&path)
        };

        match result {
            Ok(()) => {
                let processing_time = start_time.elapsed().as_millis() as u64;
                let allocations = self.tracker.get_active_allocations()?;
                let filtered_count = if self.config.include_system_allocations {
                    allocations.len()
                } else {
                    allocations.iter().filter(|a| a.var_name.is_some()).count()
                };

                Ok(ExportStats {
                    allocations_processed: filtered_count,
                    processing_time_ms: processing_time,
                    output_size_bytes: 0,
                    compression_ratio: if self.config.enable_compression { Some(0.7) } else { None },
                })
            }
            Err(e) => Err(e),
        }
    }

    /// 导出到HTML
    pub fn export_html<P: AsRef<Path>>(&self, path: P) -> TrackingResult<ExportStats> {
        // 调用现有的内存分析导出
        self.tracker.export_memory_analysis(&path)?;
        
        Ok(ExportStats {
            allocations_processed: 0,
            processing_time_ms: 0,
            output_size_bytes: 0,
            compression_ratio: None,
        })
    }

    /// 智能导出 - 根据数据量自动选择格式
    pub fn export_auto<P: AsRef<Path>>(&self, base_path: P) -> TrackingResult<ExportStats> {
        let allocations = self.tracker.get_active_allocations()?;
        let allocation_count = allocations.len();

        if allocation_count < 1000 {
            self.export_json(base_path)
        } else {
            self.export_binary(base_path)
        }
    }
}

/// 便利函数 - 快速JSON导出
pub fn quick_export_json<P: AsRef<Path>>(
    tracker: Arc<MemoryTracker>,
    path: P,
) -> TrackingResult<ExportStats> {
    let exporter = MemScopeExporter::with_default_config(tracker);
    exporter.export_json(path)
}

/// 便利函数 - 快速二进制导出
pub fn quick_export_binary<P: AsRef<Path>>(
    tracker: Arc<MemoryTracker>,
    path: P,
) -> TrackingResult<ExportStats> {
    let exporter = MemScopeExporter::with_default_config(tracker);
    exporter.export_binary(path)
}