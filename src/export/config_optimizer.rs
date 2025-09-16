//! configuration optimization tool module
//!
//! This module provides automated configuration optimization and validation tools.

use crate::core::types::TrackingResult;
use crate::export::fast_export_coordinator::FastExportConfigBuilder;
// use crate::export::performance_testing::OptimizationTarget; // Removed - using local definition

/// Optimization target for configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OptimizationTarget {
    Speed,
    Memory,
    Balanced,
}
use crate::export::system_optimizer::SystemOptimizer;
use serde::{Deserialize, Serialize};

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
    pub fn auto_optimize(
        &mut self,
        target: OptimizationTarget,
        dataset_size: Option<usize>,
    ) -> TrackingResult<FastExportConfigBuilder> {
        // generate configuration recommendation
        let recommendation = self
            .system_optimizer
            .generate_configuration_recommendation(target, dataset_size);

        // create configuration
        let config_builder = FastExportConfigBuilder::new()
            .shard_size(recommendation.recommended_shard_size)
            .max_threads(Some(recommendation.recommended_thread_count))
            .buffer_size(recommendation.recommended_buffer_size)
            .performance_monitoring(true);

        // validate configuration
        let validation_result = self
            .system_optimizer
            .validate_configuration(&config_builder);

        if !validation_result.is_valid {
            tracing::warn!("⚠️ configuration validation failed, using default configuration");
            for error in &validation_result.errors {
                tracing::warn!(" Error : {}", error);
            }
            return Ok(FastExportConfigBuilder::new());
        }

        // record optimization history
        let record = OptimizationRecord {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
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
                config_hash: format!(
                    "{:x}",
                    recommendation.recommended_shard_size
                        ^ recommendation.recommended_thread_count
                        ^ recommendation.recommended_buffer_size
                ),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_optimizer_new() {
        // Test creation of new ConfigOptimizer
        let optimizer = ConfigOptimizer::new();
        assert!(optimizer.is_ok());

        let optimizer = optimizer.unwrap();
        assert!(optimizer.optimization_history.is_empty());
    }

    #[test]
    fn test_config_optimizer_default() {
        // Test default implementation
        let optimizer = ConfigOptimizer::default();
        assert!(optimizer.optimization_history.is_empty());
    }

    #[test]
    fn test_optimization_target_equality() {
        // Test OptimizationTarget enum equality
        assert_eq!(OptimizationTarget::Speed, OptimizationTarget::Speed);
        assert_ne!(OptimizationTarget::Speed, OptimizationTarget::Memory);
        assert_ne!(OptimizationTarget::Memory, OptimizationTarget::Balanced);
    }

    #[test]
    fn test_auto_optimize_speed() {
        // Test auto optimize with Speed target
        let mut optimizer = ConfigOptimizer::new().unwrap();
        let result = optimizer.auto_optimize(OptimizationTarget::Speed, Some(10000));

        assert!(result.is_ok());
        let config_builder = result.unwrap();
        // Verify config builder was created (it should have some default values)
        // We can't test exact values as they depend on SystemOptimizer implementation
        let config = config_builder.build();
        // Just verify that the config was built successfully by checking a field
        assert!(config.shard_config.shard_size > 0);
    }

    #[test]
    fn test_auto_optimize_memory() {
        // Test auto optimize with Memory target
        let mut optimizer = ConfigOptimizer::new().unwrap();
        let result = optimizer.auto_optimize(OptimizationTarget::Memory, Some(5000));

        assert!(result.is_ok());
        assert!(!optimizer.optimization_history.is_empty());
    }

    #[test]
    fn test_auto_optimize_balanced() {
        // Test auto optimize with Balanced target
        let mut optimizer = ConfigOptimizer::new().unwrap();
        let result = optimizer.auto_optimize(OptimizationTarget::Balanced, None);

        assert!(result.is_ok());
        // Check that optimization history was recorded
        assert_eq!(optimizer.optimization_history.len(), 1);

        let record = &optimizer.optimization_history[0];
        assert_eq!(record.target, OptimizationTarget::Balanced);
        assert!(record.timestamp > 0);
    }

    #[test]
    fn test_optimization_history() {
        // Test optimization history tracking
        let mut optimizer = ConfigOptimizer::new().unwrap();

        // Perform multiple optimizations
        let _ = optimizer.auto_optimize(OptimizationTarget::Speed, Some(1000));
        let _ = optimizer.auto_optimize(OptimizationTarget::Memory, Some(2000));
        let _ = optimizer.auto_optimize(OptimizationTarget::Balanced, Some(3000));

        // Check history
        let history = optimizer.get_optimization_history();
        assert_eq!(history.len(), 3);

        // Verify each record has correct target
        assert_eq!(history[0].target, OptimizationTarget::Speed);
        assert_eq!(history[1].target, OptimizationTarget::Memory);
        assert_eq!(history[2].target, OptimizationTarget::Balanced);

        // All records should have valid timestamps
        for record in history {
            assert!(record.timestamp > 0);
            assert!(!record.optimized_config.config_hash.is_empty());
        }
    }

    #[test]
    fn test_clear_history() {
        // Test clearing optimization history
        let mut optimizer = ConfigOptimizer::new().unwrap();

        // Add some history
        let _ = optimizer.auto_optimize(OptimizationTarget::Speed, Some(1000));
        let _ = optimizer.auto_optimize(OptimizationTarget::Memory, Some(2000));

        assert!(!optimizer.optimization_history.is_empty());

        // Clear history
        optimizer.clear_history();
        assert!(optimizer.optimization_history.is_empty());
        assert_eq!(optimizer.get_optimization_history().len(), 0);
    }

    #[test]
    fn test_config_snapshot_fields() {
        // Test ConfigSnapshot structure
        let snapshot = ConfigSnapshot {
            shard_size: 1000,
            thread_count: 8,
            buffer_size: 512 * 1024,
            config_hash: "test_hash".to_string(),
        };

        assert_eq!(snapshot.shard_size, 1000);
        assert_eq!(snapshot.thread_count, 8);
        assert_eq!(snapshot.buffer_size, 512 * 1024);
        assert_eq!(snapshot.config_hash, "test_hash");
    }

    #[test]
    fn test_optimization_record_fields() {
        // Test OptimizationRecord structure
        let original = ConfigSnapshot {
            shard_size: 500,
            thread_count: 4,
            buffer_size: 256 * 1024,
            config_hash: "original".to_string(),
        };

        let optimized = ConfigSnapshot {
            shard_size: 1000,
            thread_count: 8,
            buffer_size: 512 * 1024,
            config_hash: "optimized".to_string(),
        };

        let record = OptimizationRecord {
            timestamp: 1234567890,
            target: OptimizationTarget::Speed,
            original_config: original.clone(),
            optimized_config: optimized.clone(),
            performance_improvement: 25.5,
            success_rate: 0.95,
        };

        assert_eq!(record.timestamp, 1234567890);
        assert_eq!(record.target, OptimizationTarget::Speed);
        assert_eq!(record.original_config.shard_size, 500);
        assert_eq!(record.optimized_config.shard_size, 1000);
        assert_eq!(record.performance_improvement, 25.5);
        assert_eq!(record.success_rate, 0.95);
    }

    #[test]
    fn test_multiple_optimizations_different_sizes() {
        // Test multiple optimizations with different dataset sizes
        let mut optimizer = ConfigOptimizer::new().unwrap();

        let sizes = vec![Some(100), Some(1000), Some(10000), Some(100000), None];
        for size in sizes {
            let result = optimizer.auto_optimize(OptimizationTarget::Balanced, size);
            assert!(result.is_ok());
        }

        // Should have 5 records in history
        assert_eq!(optimizer.get_optimization_history().len(), 5);
    }

    #[test]
    fn test_optimization_with_validation_scenario() {
        // Test optimization that might trigger validation warnings
        // This tests the validation path in auto_optimize
        let mut optimizer = ConfigOptimizer::new().unwrap();

        // Use very large dataset size that might trigger different optimization path
        let result = optimizer.auto_optimize(OptimizationTarget::Speed, Some(usize::MAX / 2));
        assert!(result.is_ok());

        // Even with extreme values, we should get a valid config
        let config_builder = result.unwrap();
        let config = config_builder.build();
        assert!(config.shard_config.shard_size > 0);
    }

    #[test]
    fn test_serialization_of_types() {
        // Test that our types can be serialized/deserialized
        let target = OptimizationTarget::Memory;
        let serialized = serde_json::to_string(&target).unwrap();
        let deserialized: OptimizationTarget = serde_json::from_str(&serialized).unwrap();
        assert_eq!(target, deserialized);

        let snapshot = ConfigSnapshot {
            shard_size: 2000,
            thread_count: 16,
            buffer_size: 1024 * 1024,
            config_hash: "hash123".to_string(),
        };
        let serialized = serde_json::to_string(&snapshot).unwrap();
        let deserialized: ConfigSnapshot = serde_json::from_str(&serialized).unwrap();
        assert_eq!(snapshot.shard_size, deserialized.shard_size);
        assert_eq!(snapshot.config_hash, deserialized.config_hash);
    }
}
