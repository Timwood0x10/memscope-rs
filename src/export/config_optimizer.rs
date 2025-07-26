//! configuration optimization tool module
//!
//! This module provides automated configuration optimization and validation tools.

use crate::core::types::TrackingResult;
use crate::export::fast_export_coordinator::FastExportConfigBuilder;
use crate::export::system_optimizer::SystemOptimizer;
use crate::export::performance_testing::OptimizationTarget;
use serde::{Serialize, Deserialize};

/// configuration optimizer
pub struct ConfigOptimizer {
    system_optimizer: SystemOptimizer,
    optimization_history: Vec<OptimizationRecord>,
}

/// optimization record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecord {
    /// optimization timestamp
    pub timestamp: u64,
    /// optimization target
    pub target: OptimizationTarget,
    /// original configuration
    pub original_config: ConfigSnapshot,
    /// optimized configuration
    pub optimized_config: ConfigSnapshot,
    /// performance improvement
    pub performance_improvement: f64,
    /// optimization success rate
    pub success_rate: f64,
}

/// configuration snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    /// shard size
    pub shard_size: usize,
    /// thread count
    pub thread_count: usize,
    /// buffer size
    pub buffer_size: usize,
    /// configuration hash
    pub config_hash: String,
}

impl ConfigOptimizer {
    /// create new configuration optimizer
    pub fn new() -> TrackingResult<Self> {
        Ok(Self {
            system_optimizer: SystemOptimizer::new()?,
            optimization_history: Vec::new(),
        })
    }

    /// auto optimize configuration
    pub fn auto_optimize(&mut self, target: OptimizationTarget, dataset_size: Option<usize>) -> TrackingResult<FastExportConfigBuilder> {
        // generate configuration recommendation
        let recommendation = self.system_optimizer.generate_configuration_recommendation(target, dataset_size);
        
        // create configuration
        let config_builder = FastExportConfigBuilder::new()
            .shard_size(recommendation.recommended_shard_size)
            .max_threads(Some(recommendation.recommended_thread_count))
            .buffer_size(recommendation.recommended_buffer_size)
            .performance_monitoring(true);

        // validate configuration
        let validation_result = self.system_optimizer.validate_configuration(&config_builder);
        
        if !validation_result.is_valid {
            println!("⚠️ configuration validation failed, using default configuration");
            for error in &validation_result.errors {
                println!(" Error : {}", error);
            }
            return Ok(FastExportConfigBuilder::new());
        }

        // record optimization history
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

    /// get optimization history
    pub fn get_optimization_history(&self) -> &[OptimizationRecord] {
        &self.optimization_history
    }

    /// clear optimization history
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