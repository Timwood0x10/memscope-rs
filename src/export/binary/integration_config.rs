//! Integration configuration for binary-to-JSON optimization
//!
//! This module provides configuration options for controlling the integration
//! of optimized binary-to-JSON conversion with the existing system.

use std::sync::OnceLock;
use tracing::{info, warn};

/// Global integration configuration
static INTEGRATION_CONFIG: OnceLock<IntegrationConfig> = OnceLock::new();

/// Configuration for binary-to-JSON optimization integration
#[derive(Debug, Clone)]
pub struct IntegrationConfig {
    /// Whether to enable optimized conversion by default
    pub enable_optimization: bool,
    
    /// Whether to enable automatic fallback to legacy method on errors
    pub enable_fallback: bool,
    
    /// Whether to log performance comparisons
    pub log_performance: bool,
    
    /// Minimum file size threshold for enabling optimization (bytes)
    pub optimization_threshold: u64,
    
    /// Whether to enable detailed logging
    pub enable_detailed_logging: bool,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            enable_optimization: true,
            enable_fallback: true,
            log_performance: true,
            optimization_threshold: 10 * 1024, // 10KB
            enable_detailed_logging: false,
        }
    }
}

impl IntegrationConfig {
    /// Get the global integration configuration
    pub fn global() -> &'static IntegrationConfig {
        INTEGRATION_CONFIG.get_or_init(|| {
            let config = Self::from_environment();
            info!("Binary-to-JSON optimization integration initialized: optimization={}, fallback={}", 
                  config.enable_optimization, config.enable_fallback);
            config
        })
    }

    /// Create configuration from environment variables
    pub fn from_environment() -> Self {
        let mut config = Self::default();

        // Check environment variables
        if let Ok(value) = std::env::var("MEMSCOPE_DISABLE_OPTIMIZATION") {
            if value.to_lowercase() == "true" || value == "1" {
                config.enable_optimization = false;
                info!("Binary-to-JSON optimization disabled via MEMSCOPE_DISABLE_OPTIMIZATION");
            }
        }

        if let Ok(value) = std::env::var("MEMSCOPE_DISABLE_FALLBACK") {
            if value.to_lowercase() == "true" || value == "1" {
                config.enable_fallback = false;
                info!("Fallback to legacy method disabled via MEMSCOPE_DISABLE_FALLBACK");
            }
        }

        if let Ok(value) = std::env::var("MEMSCOPE_ENABLE_DETAILED_LOGGING") {
            if value.to_lowercase() == "true" || value == "1" {
                config.enable_detailed_logging = true;
                info!("Detailed logging enabled via MEMSCOPE_ENABLE_DETAILED_LOGGING");
            }
        }

        if let Ok(value) = std::env::var("MEMSCOPE_OPTIMIZATION_THRESHOLD") {
            if let Ok(threshold) = value.parse::<u64>() {
                config.optimization_threshold = threshold;
                info!("Optimization threshold set to {} bytes via MEMSCOPE_OPTIMIZATION_THRESHOLD", threshold);
            } else {
                warn!("Invalid MEMSCOPE_OPTIMIZATION_THRESHOLD value: {}", value);
            }
        }

        if let Ok(value) = std::env::var("MEMSCOPE_DISABLE_PERFORMANCE_LOGGING") {
            if value.to_lowercase() == "true" || value == "1" {
                config.log_performance = false;
                info!("Performance logging disabled via MEMSCOPE_DISABLE_PERFORMANCE_LOGGING");
            }
        }

        config
    }

    /// Update the global configuration (for testing)
    pub fn set_global(config: IntegrationConfig) -> Result<(), IntegrationConfig> {
        INTEGRATION_CONFIG.set(config)
    }

    /// Check if optimization should be enabled for a given file size
    pub fn should_optimize(&self, file_size: u64) -> bool {
        self.enable_optimization && file_size >= self.optimization_threshold
    }

    /// Create a performance-optimized configuration
    pub fn performance_optimized() -> Self {
        Self {
            enable_optimization: true,
            enable_fallback: false, // No fallback for maximum performance
            log_performance: false, // No logging overhead
            optimization_threshold: 0, // Always optimize
            enable_detailed_logging: false,
        }
    }

    /// Create a reliability-focused configuration
    pub fn reliability_focused() -> Self {
        Self {
            enable_optimization: true,
            enable_fallback: true, // Always fallback on errors
            log_performance: true,
            optimization_threshold: 100 * 1024, // 100KB threshold
            enable_detailed_logging: true,
        }
    }

    /// Create a legacy-compatible configuration (no optimization)
    pub fn legacy_compatible() -> Self {
        Self {
            enable_optimization: false,
            enable_fallback: false,
            log_performance: false,
            optimization_threshold: u64::MAX,
            enable_detailed_logging: false,
        }
    }
}

/// Performance metrics for integration monitoring
#[derive(Debug, Clone, Default)]
pub struct IntegrationMetrics {
    /// Total conversions performed
    pub total_conversions: u64,
    
    /// Conversions using optimized method
    pub optimized_conversions: u64,
    
    /// Conversions using legacy method
    pub legacy_conversions: u64,
    
    /// Fallbacks from optimized to legacy
    pub fallback_conversions: u64,
    
    /// Total time saved through optimization
    pub total_time_saved: std::time::Duration,
    
    /// Average performance improvement factor
    pub average_improvement_factor: f64,
}

impl IntegrationMetrics {
    /// Calculate optimization usage rate
    pub fn optimization_rate(&self) -> f64 {
        if self.total_conversions > 0 {
            self.optimized_conversions as f64 / self.total_conversions as f64
        } else {
            0.0
        }
    }

    /// Calculate fallback rate
    pub fn fallback_rate(&self) -> f64 {
        if self.optimized_conversions > 0 {
            self.fallback_conversions as f64 / self.optimized_conversions as f64
        } else {
            0.0
        }
    }

    /// Generate integration report
    pub fn generate_report(&self) -> IntegrationReport {
        IntegrationReport {
            total_conversions: self.total_conversions,
            optimization_rate: self.optimization_rate(),
            fallback_rate: self.fallback_rate(),
            average_improvement: self.average_improvement_factor,
            total_time_saved_seconds: self.total_time_saved.as_secs_f64(),
            recommendations: self.generate_recommendations(),
        }
    }

    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if self.fallback_rate() > 0.1 {
            recommendations.push("High fallback rate detected. Consider investigating optimization failures.".to_string());
        }

        if self.optimization_rate() < 0.5 {
            recommendations.push("Low optimization usage. Consider lowering the optimization threshold.".to_string());
        }

        if self.average_improvement_factor > 10.0 {
            recommendations.push("Excellent optimization performance. Consider enabling optimization for smaller files.".to_string());
        }

        recommendations
    }
}

/// Integration performance report
#[derive(Debug, Clone)]
pub struct IntegrationReport {
    pub total_conversions: u64,
    pub optimization_rate: f64,
    pub fallback_rate: f64,
    pub average_improvement: f64,
    pub total_time_saved_seconds: f64,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_config_default() {
        let config = IntegrationConfig::default();
        assert!(config.enable_optimization);
        assert!(config.enable_fallback);
        assert!(config.log_performance);
        assert_eq!(config.optimization_threshold, 10 * 1024);
    }

    #[test]
    fn test_should_optimize() {
        let config = IntegrationConfig::default();
        
        assert!(!config.should_optimize(5 * 1024)); // Below threshold
        assert!(config.should_optimize(20 * 1024)); // Above threshold
        
        let disabled_config = IntegrationConfig {
            enable_optimization: false,
            ..Default::default()
        };
        assert!(!disabled_config.should_optimize(100 * 1024)); // Disabled
    }

    #[test]
    fn test_preset_configurations() {
        let perf_config = IntegrationConfig::performance_optimized();
        assert!(perf_config.enable_optimization);
        assert!(!perf_config.enable_fallback);
        assert_eq!(perf_config.optimization_threshold, 0);

        let reliability_config = IntegrationConfig::reliability_focused();
        assert!(reliability_config.enable_optimization);
        assert!(reliability_config.enable_fallback);
        assert!(reliability_config.enable_detailed_logging);

        let legacy_config = IntegrationConfig::legacy_compatible();
        assert!(!legacy_config.enable_optimization);
        assert!(!legacy_config.enable_fallback);
    }

    #[test]
    fn test_integration_metrics() {
        let mut metrics = IntegrationMetrics::default();
        metrics.total_conversions = 100;
        metrics.optimized_conversions = 80;
        metrics.legacy_conversions = 20;
        metrics.fallback_conversions = 5;
        metrics.average_improvement_factor = 15.0;

        assert_eq!(metrics.optimization_rate(), 0.8);
        assert_eq!(metrics.fallback_rate(), 0.0625);

        let report = metrics.generate_report();
        assert_eq!(report.total_conversions, 100);
        assert_eq!(report.optimization_rate, 0.8);
        assert!(!report.recommendations.is_empty());
    }
}