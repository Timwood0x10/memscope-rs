//! 配置优化工具模块
//!
//! 这个模块提供自动化的配置优化和验证工具。

use crate::core::types::TrackingResult;
use crate::export::fast_export_coordinator::FastExportConfigBuilder;
use crate::export::system_optimizer::SystemOptimizer;
use crate::export::performance_testing::OptimizationTarget;
use serde::{Serialize, Deserialize};

/// 配置优化器
pub struct ConfigOptimizer {
    system_optimizer: SystemOptimizer,
    optimization_history: Vec<OptimizationRecord>,
}

/// 优化记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecord {
    /// 优化时间戳
    pub timestamp: u64,
    /// 优化目标
    pub target: OptimizationTarget,
    /// 原始配置
    pub original_config: ConfigSnapshot,
    /// 优化后配置
    pub optimized_config: ConfigSnapshot,
    /// 性能改善
    pub performance_improvement: f64,
    /// 优化成功率
    pub success_rate: f64,
}

/// 配置快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    /// 分片大小
    pub shard_size: usize,
    /// 线程数
    pub thread_count: usize,
    /// 缓冲区大小
    pub buffer_size: usize,
    /// 配置哈希
    pub config_hash: String,
}

impl ConfigOptimizer {
    /// 创建新的配置优化器
    pub fn new() -> TrackingResult<Self> {
        Ok(Self {
            system_optimizer: SystemOptimizer::new()?,
            optimization_history: Vec::new(),
        })
    }

    /// 自动优化配置
    pub fn auto_optimize(&mut self, target: OptimizationTarget, dataset_size: Option<usize>) -> TrackingResult<FastExportConfigBuilder> {
        // 生成配置建议
        let recommendation = self.system_optimizer.generate_configuration_recommendation(target, dataset_size);
        
        // 创建配置
        let config_builder = FastExportConfigBuilder::new()
            .shard_size(recommendation.recommended_shard_size)
            .max_threads(Some(recommendation.recommended_thread_count))
            .buffer_size(recommendation.recommended_buffer_size)
            .performance_monitoring(true);

        // 验证配置
        let validation_result = self.system_optimizer.validate_configuration(&config_builder);
        
        if !validation_result.is_valid {
            println!("⚠️ 配置验证失败，使用默认配置");
            for error in &validation_result.errors {
                println!("  错误: {}", error);
            }
            return Ok(FastExportConfigBuilder::new());
        }

        // 记录优化历史
        let record = OptimizationRecord {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            target,
            original_config: ConfigSnapshot {
                shard_size: 1000,
                thread_count: 4,
                buffer_size: 256 * 1024,
                config_hash: "default".to_string(),
            },
            optimized_config: ConfigSnapshot {
                shard_size: recommendation.recommended_shard_size,
                thread_count: recommendation.recommended_thread_count,
                buffer_size: recommendation.recommended_buffer_size,
                config_hash: format!("{:x}", 
                    recommendation.recommended_shard_size ^ 
                    recommendation.recommended_thread_count ^ 
                    recommendation.recommended_buffer_size),
            },
            performance_improvement: recommendation.expected_performance_gain,
            success_rate: recommendation.confidence,
        };

        self.optimization_history.push(record);

        Ok(config_builder)
    }

    /// 获取优化历史
    pub fn get_optimization_history(&self) -> &[OptimizationRecord] {
        &self.optimization_history
    }

    /// 清除优化历史
    pub fn clear_history(&mut self) {
        self.optimization_history.clear();
    }
}

impl Default for ConfigOptimizer {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            system_optimizer: SystemOptimizer::default(),
            optimization_history: Vec::new(),
        })
    }
}