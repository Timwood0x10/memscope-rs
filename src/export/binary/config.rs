//! Binary export configuration for advanced memory analysis metrics

use serde::{Deserialize, Serialize};

/// Advanced metrics level configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdvancedMetricsLevel {
    /// Only basic data (existing behavior)
    None,
    /// Core advanced metrics with minimal performance impact
    Essential,
    /// All advanced metrics, may impact performance
    Comprehensive,
}

impl Default for AdvancedMetricsLevel {
    fn default() -> Self {
        AdvancedMetricsLevel::Essential
    }
}

/// Binary export configuration with advanced metrics support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryExportConfig {
    // Existing fields (maintain compatibility)
    /// Buffer size for I/O operations
    pub buffer_size: usize,
    /// Include detailed information
    pub include_details: bool,
    /// Compression level (0-9, 0 = no compression)
    pub compression_level: u8,

    // New advanced metrics fields
    /// Advanced metrics collection level
    pub advanced_metrics_level: AdvancedMetricsLevel,
    /// Enable source code analysis (stack traces with file/line info)
    pub source_analysis: bool,
    /// Enable lifecycle timeline analysis
    pub lifecycle_timeline: bool,
    /// Enable container structure analysis (Vec, HashMap, etc.)
    pub container_analysis: bool,
    /// Enable memory fragmentation analysis
    pub fragmentation_analysis: bool,
    /// Enable thread context tracking
    pub thread_context_tracking: bool,
    /// Enable Drop chain analysis
    pub drop_chain_analysis: bool,
    /// Enable ZST (Zero-Sized Type) analysis
    pub zst_analysis: bool,
    /// Enable memory health scoring
    pub health_scoring: bool,
    /// Enable performance benchmarking
    pub performance_benchmarking: bool,
    /// Enable string table optimization for repeated strings
    pub string_table_optimization: bool,
}

impl Default for BinaryExportConfig {
    fn default() -> Self {
        Self::performance_first()
    }
}

impl BinaryExportConfig {
    /// Create new configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Performance-first configuration (minimal overhead)
    pub fn performance_first() -> Self {
        Self {
            // Basic settings
            buffer_size: 64 * 1024, // 64KB
            include_details: true,
            compression_level: 0, // No compression for speed

            // Advanced metrics - conservative settings
            advanced_metrics_level: AdvancedMetricsLevel::Essential,
            source_analysis: false,          // Stack tracing has overhead
            lifecycle_timeline: true,        // Based on existing data, low overhead
            container_analysis: true,        // High value container analysis
            fragmentation_analysis: false,   // Computation overhead
            thread_context_tracking: true,   // Low overhead, high value
            drop_chain_analysis: false,      // Can be expensive
            zst_analysis: false,             // Specialized use case
            health_scoring: false,           // Additional computation
            performance_benchmarking: false, // Only for debugging
            string_table_optimization: true, // Good compression with minimal overhead
        }
    }

    /// Debug/development configuration (comprehensive analysis)
    pub fn debug_comprehensive() -> Self {
        Self {
            // Basic settings
            buffer_size: 128 * 1024, // 128KB for better I/O
            include_details: true,
            compression_level: 1, // Light compression

            // Advanced metrics - all enabled
            advanced_metrics_level: AdvancedMetricsLevel::Comprehensive,
            source_analysis: true,
            lifecycle_timeline: true,
            container_analysis: true,
            fragmentation_analysis: true,
            thread_context_tracking: true,
            drop_chain_analysis: true,
            zst_analysis: true,
            health_scoring: true,
            performance_benchmarking: true,
            string_table_optimization: true,
        }
    }

    /// Minimal configuration (fastest export, basic data only)
    pub fn minimal() -> Self {
        Self {
            // Basic settings
            buffer_size: 32 * 1024, // 32KB
            include_details: false,
            compression_level: 0,

            // Advanced metrics - all disabled
            advanced_metrics_level: AdvancedMetricsLevel::None,
            source_analysis: false,
            lifecycle_timeline: false,
            container_analysis: false,
            fragmentation_analysis: false,
            thread_context_tracking: false,
            drop_chain_analysis: false,
            zst_analysis: false,
            health_scoring: false,
            performance_benchmarking: false,
            string_table_optimization: false, // Minimal config disables optimizations
        }
    }

    /// Validate configuration and apply safe defaults
    pub fn validate_and_fix(&mut self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Validate buffer size
        if self.buffer_size < 1024 {
            warnings.push("Buffer size too small, setting to 1KB minimum".to_string());
            self.buffer_size = 1024;
        } else if self.buffer_size > 1024 * 1024 {
            warnings.push("Buffer size too large, setting to 1MB maximum".to_string());
            self.buffer_size = 1024 * 1024;
        }

        // Validate compression level
        if self.compression_level > 9 {
            warnings.push("Compression level too high, setting to maximum 9".to_string());
            self.compression_level = 9;
        }

        // Check for conflicting settings
        match self.advanced_metrics_level {
            AdvancedMetricsLevel::None => {
                // If metrics level is None, disable all advanced features
                if self.source_analysis
                    || self.lifecycle_timeline
                    || self.container_analysis
                    || self.fragmentation_analysis
                    || self.thread_context_tracking
                    || self.drop_chain_analysis
                    || self.zst_analysis
                    || self.health_scoring
                {
                    warnings.push("Advanced metrics level is None but some advanced features are enabled. Disabling advanced features.".to_string());
                    self.disable_all_advanced_features();
                }
            }
            AdvancedMetricsLevel::Essential => {
                // For Essential level, disable expensive features
                if self.source_analysis {
                    warnings.push("Source analysis is expensive for Essential level. Consider using Comprehensive level or disabling source analysis.".to_string());
                }
                if self.fragmentation_analysis {
                    warnings.push("Fragmentation analysis is expensive for Essential level. Consider using Comprehensive level or disabling fragmentation analysis.".to_string());
                }
            }
            AdvancedMetricsLevel::Comprehensive => {
                // All features are allowed at Comprehensive level
            }
        }

        // Performance vs features conflict detection
        if self.compression_level > 0
            && self.advanced_metrics_level == AdvancedMetricsLevel::Comprehensive
        {
            warnings.push(
                "High compression with comprehensive metrics may significantly impact performance"
                    .to_string(),
            );
        }

        warnings
    }

    /// Disable all advanced features (used internally)
    fn disable_all_advanced_features(&mut self) {
        self.source_analysis = false;
        self.lifecycle_timeline = false;
        self.container_analysis = false;
        self.fragmentation_analysis = false;
        self.thread_context_tracking = false;
        self.drop_chain_analysis = false;
        self.zst_analysis = false;
        self.health_scoring = false;
        self.performance_benchmarking = false;
        self.string_table_optimization = false;
    }

    /// Check if any advanced metrics are enabled
    pub fn has_advanced_metrics(&self) -> bool {
        self.advanced_metrics_level != AdvancedMetricsLevel::None
            || self.source_analysis
            || self.lifecycle_timeline
            || self.container_analysis
            || self.fragmentation_analysis
            || self.thread_context_tracking
            || self.drop_chain_analysis
            || self.zst_analysis
            || self.health_scoring
            || self.performance_benchmarking
            || self.string_table_optimization
    }

    /// Get estimated performance impact (0.0 = no impact, 1.0 = significant impact)
    pub fn estimated_performance_impact(&self) -> f64 {
        let mut impact = 0.0;

        // Base impact from metrics level
        impact += match self.advanced_metrics_level {
            AdvancedMetricsLevel::None => 0.0,
            AdvancedMetricsLevel::Essential => 0.1,
            AdvancedMetricsLevel::Comprehensive => 0.3,
        };

        // Individual feature impacts
        if self.source_analysis {
            impact += 0.2;
        }
        if self.fragmentation_analysis {
            impact += 0.3;
        }
        if self.drop_chain_analysis {
            impact += 0.15;
        }
        if self.zst_analysis {
            impact += 0.1;
        }
        if self.health_scoring {
            impact += 0.1;
        }
        if self.performance_benchmarking {
            impact += 0.05;
        }

        // Compression impact
        impact += self.compression_level as f64 * 0.02;

        impact.min(1.0) // Cap at 1.0
    }
}

/// Builder pattern for BinaryExportConfig
pub struct BinaryExportConfigBuilder {
    config: BinaryExportConfig,
}

impl BinaryExportConfigBuilder {
    /// Create new config builder with performance-first defaults
    pub fn new() -> Self {
        Self {
            config: BinaryExportConfig::performance_first(),
        }
    }

    /// Create builder from existing config
    pub fn from_config(config: BinaryExportConfig) -> Self {
        Self { config }
    }

    /// Set buffer size
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }

    /// Set compression level
    pub fn compression_level(mut self, level: u8) -> Self {
        self.config.compression_level = level;
        self
    }

    /// Set advanced metrics level
    pub fn advanced_metrics_level(mut self, level: AdvancedMetricsLevel) -> Self {
        self.config.advanced_metrics_level = level;
        self
    }

    /// Enable/disable source analysis
    pub fn source_analysis(mut self, enable: bool) -> Self {
        self.config.source_analysis = enable;
        self
    }

    /// Enable/disable lifecycle timeline
    pub fn lifecycle_timeline(mut self, enable: bool) -> Self {
        self.config.lifecycle_timeline = enable;
        self
    }

    /// Enable/disable container analysis
    pub fn container_analysis(mut self, enable: bool) -> Self {
        self.config.container_analysis = enable;
        self
    }

    /// Enable/disable fragmentation analysis
    pub fn fragmentation_analysis(mut self, enable: bool) -> Self {
        self.config.fragmentation_analysis = enable;
        self
    }

    /// Enable/disable thread context tracking
    pub fn thread_context_tracking(mut self, enable: bool) -> Self {
        self.config.thread_context_tracking = enable;
        self
    }

    /// Enable/disable Drop chain analysis
    pub fn drop_chain_analysis(mut self, enable: bool) -> Self {
        self.config.drop_chain_analysis = enable;
        self
    }

    /// Enable/disable ZST analysis
    pub fn zst_analysis(mut self, enable: bool) -> Self {
        self.config.zst_analysis = enable;
        self
    }

    /// Enable/disable health scoring
    pub fn health_scoring(mut self, enable: bool) -> Self {
        self.config.health_scoring = enable;
        self
    }

    /// Enable/disable performance benchmarking
    pub fn performance_benchmarking(mut self, enable: bool) -> Self {
        self.config.performance_benchmarking = enable;
        self
    }

    /// Enable/disable string table optimization
    pub fn string_table_optimization(mut self, enable: bool) -> Self {
        self.config.string_table_optimization = enable;
        self
    }

    /// Build the configuration
    pub fn build(mut self) -> BinaryExportConfig {
        let warnings = self.config.validate_and_fix();
        if !warnings.is_empty() {
            tracing::warn!("Configuration warnings: {:?}", warnings);
        }
        self.config
    }
}

impl Default for BinaryExportConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BinaryExportConfig::default();
        assert_eq!(
            config.advanced_metrics_level,
            AdvancedMetricsLevel::Essential
        );
        assert!(!config.source_analysis); // Should be false for performance
        assert!(config.lifecycle_timeline); // Should be true for value
        assert!(config.container_analysis); // Should be true for value
    }

    #[test]
    fn test_performance_first_config() {
        let config = BinaryExportConfig::performance_first();
        assert_eq!(config.compression_level, 0);
        assert!(!config.source_analysis);
        assert!(!config.fragmentation_analysis);
        assert!(config.container_analysis); // High value, low cost
    }

    #[test]
    fn test_debug_comprehensive_config() {
        let config = BinaryExportConfig::debug_comprehensive();
        assert_eq!(
            config.advanced_metrics_level,
            AdvancedMetricsLevel::Comprehensive
        );
        assert!(config.source_analysis);
        assert!(config.fragmentation_analysis);
        assert!(config.zst_analysis);
    }

    #[test]
    fn test_minimal_config() {
        let config = BinaryExportConfig::minimal();
        assert_eq!(config.advanced_metrics_level, AdvancedMetricsLevel::None);
        assert!(!config.source_analysis);
        assert!(!config.lifecycle_timeline);
        assert!(!config.container_analysis);
    }

    #[test]
    fn test_config_validation() {
        let mut config = BinaryExportConfig::default();
        config.buffer_size = 100; // Too small
        config.compression_level = 15; // Too high

        let warnings = config.validate_and_fix();
        assert!(!warnings.is_empty());
        assert_eq!(config.buffer_size, 1024);
        assert_eq!(config.compression_level, 9);
    }

    #[test]
    fn test_config_builder() {
        let config = BinaryExportConfigBuilder::new()
            .advanced_metrics_level(AdvancedMetricsLevel::Comprehensive)
            .source_analysis(true)
            .fragmentation_analysis(true)
            .compression_level(3)
            .build();

        assert_eq!(
            config.advanced_metrics_level,
            AdvancedMetricsLevel::Comprehensive
        );
        assert!(config.source_analysis);
        assert!(config.fragmentation_analysis);
        assert_eq!(config.compression_level, 3);
    }

    #[test]
    fn test_performance_impact_estimation() {
        let minimal = BinaryExportConfig::minimal();
        let comprehensive = BinaryExportConfig::debug_comprehensive();

        assert!(minimal.estimated_performance_impact() < 0.1);
        assert!(comprehensive.estimated_performance_impact() > 0.5);
    }

    #[test]
    fn test_has_advanced_metrics() {
        let minimal = BinaryExportConfig::minimal();
        let performance = BinaryExportConfig::performance_first();
        let comprehensive = BinaryExportConfig::debug_comprehensive();

        assert!(!minimal.has_advanced_metrics());
        assert!(performance.has_advanced_metrics());
        assert!(comprehensive.has_advanced_metrics());
    }
}
