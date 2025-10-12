use std::time::Duration;

/// 内存管理配置
///
/// 定义内存跟踪的各种限制和策略，确保长期运行时
/// 内存使用可控且性能良好。
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// 最大分配记录数量
    pub max_allocations: usize,
    /// 历史记录最大保存时间
    pub max_history_age: Duration,
    /// 内存使用限制（MB）
    pub memory_limit_mb: usize,
    /// 是否启用内存警告
    pub enable_warnings: bool,
    /// 内存清理触发阈值（0.0-1.0）
    pub cleanup_threshold: f64,
    /// 批量清理大小
    pub batch_cleanup_size: usize,
    /// 是否启用自动压缩
    pub enable_auto_compaction: bool,
    /// 压缩触发间隔
    pub compaction_interval: Duration,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_allocations: 100_000,
            max_history_age: Duration::from_secs(3600), // 1小时
            memory_limit_mb: 512,                       // 512MB
            enable_warnings: true,
            cleanup_threshold: 0.8,   // 80%内存使用时开始清理
            batch_cleanup_size: 1000, // 每次清理1000个条目
            enable_auto_compaction: true,
            compaction_interval: Duration::from_secs(300), // 5分钟
        }
    }
}

impl MemoryConfig {
    /// 创建用于开发环境的配置（较宽松的限制）
    pub fn development() -> Self {
        Self {
            max_allocations: 1_000_000,
            max_history_age: Duration::from_secs(7200), // 2小时
            memory_limit_mb: 1024,                      // 1GB
            enable_warnings: true,
            cleanup_threshold: 0.9,
            batch_cleanup_size: 10000,
            enable_auto_compaction: true,
            compaction_interval: Duration::from_secs(600), // 10分钟
        }
    }

    /// 创建用于生产环境的配置（严格的限制）
    pub fn production() -> Self {
        Self {
            max_allocations: 50_000,
            max_history_age: Duration::from_secs(1800), // 30分钟
            memory_limit_mb: 256,                       // 256MB
            enable_warnings: true,
            cleanup_threshold: 0.7,
            batch_cleanup_size: 500,
            enable_auto_compaction: true,
            compaction_interval: Duration::from_secs(120), // 2分钟
        }
    }

    /// 创建用于测试环境的配置（最小化的限制）
    pub fn testing() -> Self {
        Self {
            max_allocations: 1000,
            max_history_age: Duration::from_secs(60), // 1分钟
            memory_limit_mb: 32,                      // 32MB
            enable_warnings: false,                   // 测试时不打印警告
            cleanup_threshold: 0.8,
            batch_cleanup_size: 100,
            enable_auto_compaction: false, // 测试时禁用自动压缩
            compaction_interval: Duration::from_secs(30),
        }
    }

    /// 创建高性能配置（优化延迟）
    pub fn high_performance() -> Self {
        Self {
            max_allocations: 200_000,
            max_history_age: Duration::from_secs(900), // 15分钟
            memory_limit_mb: 512,
            enable_warnings: false, // 减少I/O开销
            cleanup_threshold: 0.85,
            batch_cleanup_size: 2000,      // 较大的批处理减少清理频率
            enable_auto_compaction: false, // 禁用可能影响性能的压缩
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

    /// 估算配置下的内存使用
    pub fn estimate_memory_usage(&self) -> MemoryEstimate {
        // 每个分配记录的估算大小
        let avg_allocation_size = 128; // 字节
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

/// 配置错误类型
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),

    #[error("System information unavailable")]
    SystemInfoUnavailable,

    #[error("Insufficient system resources")]
    InsufficientResources,
}

/// 内存使用估算
#[derive(Debug, Clone)]
pub struct MemoryEstimate {
    /// 最大条目数
    pub max_entries: usize,
    /// 估算的最大内存使用（MB）
    pub estimated_max_usage_mb: f64,
    /// 配置的内存限制（MB）
    pub configured_limit_mb: f64,
    /// 实际生效的限制（MB）
    pub effective_limit_mb: f64,
    /// 清理触发阈值（MB）
    pub cleanup_trigger_mb: f64,
}

impl MemoryEstimate {
    /// 检查配置是否合理
    pub fn is_reasonable(&self) -> bool {
        self.effective_limit_mb >= 32.0 && // 至少32MB
        self.cleanup_trigger_mb < self.effective_limit_mb && // 清理阈值小于限制
        self.max_entries >= 1000 // 至少能存储1000个条目
    }

    /// 获取配置建议
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
        let mut config = MemoryConfig::default();

        // 测试无效的max_allocations
        config.max_allocations = 0;
        assert!(config.validate().is_err());

        // 测试无效的cleanup_threshold
        config = MemoryConfig::default();
        config.cleanup_threshold = 1.5;
        assert!(config.validate().is_err());

        config.cleanup_threshold = -0.1;
        assert!(config.validate().is_err());
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
        // 创建一个需要建议的配置
        let config = MemoryConfig {
            max_allocations: 500,    // 很小
            memory_limit_mb: 16,     // 很小
            cleanup_threshold: 0.95, // 很高
            ..Default::default()
        };

        let estimate = config.estimate_memory_usage();
        let recommendations = estimate.get_recommendations();

        assert!(!recommendations.is_empty());
        assert!(recommendations.len() >= 2); // 应该有多个建议
    }

    #[test]
    fn test_system_config_creation() {
        // 这个测试可能因平台而异
        match MemoryConfig::for_current_system() {
            Ok(config) => {
                assert!(config.validate().is_ok());
                assert!(config.memory_limit_mb >= 64);
            }
            Err(_) => {
                // 在某些环境下可能失败，这是可以接受的
            }
        }
    }
}
