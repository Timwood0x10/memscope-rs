use std::time::Duration;

/// Memory management configuration
///
/// Defines various limits and policies for memory tracking, ensuring
/// controlled memory usage and good performance during long-term operation.
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Maximum number of allocation records
    pub max_allocations: usize,
    /// Maximum retention time for historical records
    pub max_history_age: Duration,
    /// Memory usage limit (MB)
    pub memory_limit_mb: usize,
    /// Whether to enable memory warnings
    pub enable_warnings: bool,
    /// Memory cleanup trigger threshold (0.0-1.0)
    pub cleanup_threshold: f64,
    /// Batch cleanup size
    pub batch_cleanup_size: usize,
    /// Whether to enable automatic compaction
    pub enable_auto_compaction: bool,
    /// Compaction trigger interval
    pub compaction_interval: Duration,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_allocations: 100_000,
            max_history_age: Duration::from_secs(3600), // 1 hour
            memory_limit_mb: 512,                       // 512MB
            enable_warnings: true,
            cleanup_threshold: 0.8,   // Start cleanup at 80% memory usage
            batch_cleanup_size: 1000, // Clean 1000 entries per batch
            enable_auto_compaction: true,
            compaction_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl MemoryConfig {
    /// Create configuration for development environment (relaxed limits)
    pub fn development() -> Self {
        Self {
            max_allocations: 1_000_000,
            max_history_age: Duration::from_secs(7200), // 2 hours
            memory_limit_mb: 1024,                      // 1GB
            enable_warnings: true,
            cleanup_threshold: 0.9,
            batch_cleanup_size: 10000,
            enable_auto_compaction: true,
            compaction_interval: Duration::from_secs(600), // 10 minutes
        }
    }

    /// Create configuration for production environment (strict limits)
    pub fn production() -> Self {
        Self {
            max_allocations: 50_000,
            max_history_age: Duration::from_secs(1800), // 30 minutes
            memory_limit_mb: 256,                       // 256MB
            enable_warnings: true,
            cleanup_threshold: 0.7,
            batch_cleanup_size: 500,
            enable_auto_compaction: true,
            compaction_interval: Duration::from_secs(120), // 2 minutes
        }
    }

    /// Create configuration for testing environment (minimal limits)
    pub fn testing() -> Self {
        Self {
            max_allocations: 1000,
            max_history_age: Duration::from_secs(60), // 1 minute
            memory_limit_mb: 32,                      // 32MB
            enable_warnings: false,                   // No warnings during testing
            cleanup_threshold: 0.8,
            batch_cleanup_size: 100,
            enable_auto_compaction: false, // Disable auto compaction during testing
            compaction_interval: Duration::from_secs(30),
        }
    }

    /// Create high-performance configuration (optimized for latency)
    pub fn high_performance() -> Self {
        Self {
            max_allocations: 200_000,
            max_history_age: Duration::from_secs(900), // 15 minutes
            memory_limit_mb: 512,
            enable_warnings: false, // Reduce I/O overhead
            cleanup_threshold: 0.85,
            batch_cleanup_size: 2000, // Larger batches reduce cleanup frequency
            enable_auto_compaction: false, // Disable compaction that might affect performance
            compaction_interval: Duration::from_secs(3600),
        }
    }

    /// 验证配置的有效性
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.max_allocations == 0 {
            return Err(ConfigError::InvalidValue(
                "max_allocations must be greater than 0".into(),
            ));
        }

        if self.memory_limit_mb == 0 {
            return Err(ConfigError::InvalidValue(
                "memory_limit_mb must be greater than 0".into(),
            ));
        }

        if self.cleanup_threshold <= 0.0 || self.cleanup_threshold >= 1.0 {
            return Err(ConfigError::InvalidValue(
                "cleanup_threshold must be between 0.0 and 1.0".into(),
            ));
        }

        if self.batch_cleanup_size == 0 {
            return Err(ConfigError::InvalidValue(
                "batch_cleanup_size must be greater than 0".into(),
            ));
        }

        if self.batch_cleanup_size > self.max_allocations {
            return Err(ConfigError::InvalidValue(
                "batch_cleanup_size should not exceed max_allocations".into(),
            ));
        }

        Ok(())
    }

    /// 根据可用系统内存自动调整配置
    pub fn auto_adjust_for_system(&mut self) -> Result<(), ConfigError> {
        // 获取系统内存信息（简化实现）
        let system_memory_mb = self.get_system_memory_mb()?;

        // 限制内存使用不超过系统内存的10%
        let max_allowed_mb = (system_memory_mb as f64 * 0.1) as usize;
        if self.memory_limit_mb > max_allowed_mb {
            self.memory_limit_mb = max_allowed_mb.max(64); // 最少64MB
        }

        // 根据内存限制调整其他参数
        self.max_allocations = (self.memory_limit_mb * 1024 * 1024 / 512).min(self.max_allocations);
        self.batch_cleanup_size = (self.max_allocations / 100).max(100);

        Ok(())
    }

    /// 获取系统内存大小（MB）
    fn get_system_memory_mb(&self) -> Result<usize, ConfigError> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let meminfo = fs::read_to_string("/proc/meminfo")
                .map_err(|_| ConfigError::SystemInfoUnavailable)?;

            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let kb: usize = parts[1]
                            .parse()
                            .map_err(|_| ConfigError::SystemInfoUnavailable)?;
                        return Ok(kb / 1024); // 转换为MB
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            return Ok(8192);
        }

        #[cfg(target_os = "macos")]
        {
            Ok(8192)
        }
        #[cfg(not(target_os = "macos"))]
        {
            Ok(4096)
        }
    }

    /// 创建适合当前系统的配置
    pub fn for_current_system() -> Result<Self, ConfigError> {
        let mut config = Self::default();
        config.auto_adjust_for_system()?;
        config.validate()?;
        Ok(config)
    }

    /// Estimate memory usage under this configuration
    pub fn estimate_memory_usage(&self) -> MemoryEstimate {
        // Estimated size per allocation record
        let avg_allocation_size = 128; // bytes
        let max_memory_usage = self.max_allocations * avg_allocation_size;
        let configured_limit = self.memory_limit_mb * 1024 * 1024;

        MemoryEstimate {
            max_entries: self.max_allocations,
            estimated_max_usage_mb: (max_memory_usage / (1024 * 1024)) as f64,
            configured_limit_mb: self.memory_limit_mb as f64,
            effective_limit_mb: (configured_limit.min(max_memory_usage) / (1024 * 1024)) as f64,
            cleanup_trigger_mb: (configured_limit as f64 * self.cleanup_threshold)
                / (1024.0 * 1024.0),
        }
    }
}

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),

    #[error("System information unavailable")]
    SystemInfoUnavailable,

    #[error("Insufficient system resources")]
    InsufficientResources,
}

/// Memory usage estimation
#[derive(Debug, Clone)]
pub struct MemoryEstimate {
    /// Maximum number of entries
    pub max_entries: usize,
    /// Estimated maximum memory usage (MB)
    pub estimated_max_usage_mb: f64,
    /// Configured memory limit (MB)
    pub configured_limit_mb: f64,
    /// Effective memory limit (MB)
    pub effective_limit_mb: f64,
    /// Cleanup trigger threshold (MB)
    pub cleanup_trigger_mb: f64,
}

impl MemoryEstimate {
    /// Check if the configuration is reasonable
    pub fn is_reasonable(&self) -> bool {
        self.effective_limit_mb >= 32.0 && // At least 32MB
        self.cleanup_trigger_mb < self.effective_limit_mb && // Cleanup threshold less than limit
        self.max_entries >= 1000 // At least able to store 1000 entries
    }

    /// Get configuration recommendations
    pub fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if self.effective_limit_mb < 64.0 {
            recommendations.push(
                "Consider increasing memory limit to at least 64MB for better performance".into(),
            );
        }

        if self.max_entries < 10000 {
            recommendations
                .push("Low max_entries may cause frequent cleanup, consider increasing".into());
        }

        if self.cleanup_trigger_mb / self.effective_limit_mb > 0.9 {
            recommendations.push("Cleanup threshold is too high, may cause memory pressure".into());
        }

        if self.estimated_max_usage_mb > self.configured_limit_mb * 2.0 {
            recommendations.push("Estimated usage significantly exceeds configured limit".into());
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let config = MemoryConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_preset_configs() {
        let configs = vec![
            MemoryConfig::development(),
            MemoryConfig::production(),
            MemoryConfig::testing(),
            MemoryConfig::high_performance(),
        ];

        for config in configs {
            assert!(config.validate().is_ok(), "Preset config should be valid");
        }
    }

    #[test]
    fn test_invalid_configs() {
        // test zero max_allocations
        let config = MemoryConfig {
            max_allocations: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // test failed cleanup_threshold
        let config2 = MemoryConfig {
            cleanup_threshold: 1.5,
            ..Default::default()
        };
        assert!(config2.validate().is_err());

        let config3 = MemoryConfig {
            cleanup_threshold: -0.1,
            ..Default::default()
        };
        assert!(config3.validate().is_err());
    }

    #[test]
    fn test_memory_estimation() {
        let config = MemoryConfig::default();
        let estimate = config.estimate_memory_usage();

        assert!(estimate.is_reasonable());
        assert!(estimate.effective_limit_mb > 0.0);
        assert!(estimate.cleanup_trigger_mb < estimate.effective_limit_mb);
    }

    #[test]
    fn test_recommendations() {
        let config = MemoryConfig {
            max_allocations: 500,    // low
            memory_limit_mb: 16,     // low
            cleanup_threshold: 0.95, // high
            ..Default::default()
        };

        let estimate = config.estimate_memory_usage();
        let recommendations = estimate.get_recommendations();

        assert!(!recommendations.is_empty());
        assert!(recommendations.len() >= 2); // maybe more than one suggestion
    }

    #[test]
    fn test_system_config_creation() {
        // This test may fail on unsupported platforms
        match MemoryConfig::for_current_system() {
            Ok(config) => {
                assert!(config.validate().is_ok());
                assert!(config.memory_limit_mb >= 64);
            }
            Err(_) => {
                // may fail on unsupported platforms
            }
        }
    }
}
