//! Optimized binary-to-JSON converter with intelligent strategy selection
//!
//! This module provides the main entry point for optimized binary-to-JSON conversion,
//! integrating all optimization components and providing a unified API that maintains
//! compatibility with existing code while delivering significant performance improvements.

use crate::export::binary::{
    AdaptiveMultiJsonExporter, AdaptiveExportConfig, BinaryExportError, SelectiveJsonExporter,
    SelectiveJsonExportConfig, JsonType, MultiExportStats, AllocationField,
};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::{info, warn, error};

/// Main optimized binary-to-JSON converter providing unified API
pub struct OptimizedBinaryToJsonConverter {
    /// Configuration for the converter
    config: SelectiveConversionConfig,
    
    /// Adaptive multi-JSON exporter for intelligent strategy selection
    adaptive_exporter: AdaptiveMultiJsonExporter,
    
    /// Selective JSON exporter for fine-grained control
    selective_exporter: SelectiveJsonExporter,
    
    /// Performance statistics
    stats: ConversionStats,
}

/// Configuration for selective binary-to-JSON conversion
#[derive(Debug, Clone)]
pub struct SelectiveConversionConfig {
    /// Optimization level to use
    pub optimization_level: OptimizationLevel,
    
    /// Whether to enable parallel processing
    pub enable_parallel_processing: bool,
    
    /// Maximum number of concurrent exports
    pub max_concurrent_exports: usize,
    
    /// Whether to enable error recovery mechanisms
    pub enable_error_recovery: bool,
    
    /// Whether to enable performance monitoring
    pub enable_performance_monitoring: bool,
    
    /// Whether to enable automatic strategy selection
    pub enable_auto_strategy_selection: bool,
    
    /// Custom file size thresholds for strategy selection
    pub custom_thresholds: Option<StrategyThresholds>,
    
    /// Whether to enable detailed logging
    pub enable_detailed_logging: bool,
    
    /// Maximum memory usage before triggering optimizations (in bytes)
    pub max_memory_usage: usize,
    
    /// Timeout for conversion operations
    pub conversion_timeout: Option<Duration>,
}

/// Optimization levels for conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    /// Minimal optimization, fastest startup
    Minimal,
    /// Balanced optimization and performance
    Balanced,
    /// Maximum optimization, best performance for large files
    Maximum,
    /// Custom optimization with user-defined parameters
    Custom,
}

/// Custom strategy selection thresholds
#[derive(Debug, Clone, PartialEq)]
pub struct StrategyThresholds {
    /// Threshold for switching from simple to indexed strategy (bytes)
    pub simple_to_indexed: u64,
    /// Threshold for switching from indexed to streaming strategy (bytes)
    pub indexed_to_streaming: u64,
}

/// Runtime configuration update structure
#[derive(Debug, Clone, Default)]
pub struct RuntimeConfigUpdate {
    /// New optimization level
    pub optimization_level: Option<OptimizationLevel>,
    /// Enable/disable parallel processing
    pub enable_parallel_processing: Option<bool>,
    /// New maximum concurrent exports
    pub max_concurrent_exports: Option<usize>,
    /// Enable/disable error recovery
    pub enable_error_recovery: Option<bool>,
    /// Enable/disable performance monitoring
    pub enable_performance_monitoring: Option<bool>,
    /// Enable/disable detailed logging
    pub enable_detailed_logging: Option<bool>,
    /// New maximum memory usage
    pub max_memory_usage: Option<usize>,
    /// New conversion timeout
    pub conversion_timeout: Option<Option<Duration>>,
}

/// Configuration summary for display and monitoring
#[derive(Debug, Clone)]
pub struct ConfigurationSummary {
    /// Current optimization level
    pub optimization_level: OptimizationLevel,
    /// Whether parallel processing is enabled
    pub parallel_processing: bool,
    /// Maximum concurrent exports
    pub max_concurrent_exports: usize,
    /// Whether error recovery is enabled
    pub error_recovery: bool,
    /// Whether performance monitoring is enabled
    pub performance_monitoring: bool,
    /// Whether auto strategy selection is enabled
    pub auto_strategy_selection: bool,
    /// Whether custom thresholds are configured
    pub has_custom_thresholds: bool,
    /// Whether detailed logging is enabled
    pub detailed_logging: bool,
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
    /// Timeout in seconds (None if no timeout)
    pub timeout_seconds: Option<u64>,
}

/// Comprehensive conversion statistics
#[derive(Debug, Clone, Default)]
pub struct ConversionStats {
    /// Total number of conversions performed
    pub total_conversions: u64,
    
    /// Total processing time across all conversions
    pub total_processing_time: Duration,
    
    /// Total number of allocations processed
    pub total_allocations_processed: u64,
    
    /// Total bytes written across all conversions
    pub total_bytes_written: u64,
    
    /// Number of successful conversions
    pub successful_conversions: u64,
    
    /// Number of failed conversions
    pub failed_conversions: u64,
    
    /// Strategy usage statistics
    pub strategy_usage: HashMap<String, u64>,
    
    /// Average processing speed (allocations per second)
    pub average_processing_speed: f64,
    
    /// Peak memory usage during conversions
    pub peak_memory_usage: usize,
    
    /// Error recovery statistics
    pub error_recoveries: u64,
}

impl SelectiveConversionConfig {
    /// Create a configuration optimized for performance
    pub fn performance_first() -> Self {
        Self {
            optimization_level: OptimizationLevel::Maximum,
            enable_parallel_processing: true,
            max_concurrent_exports: num_cpus::get().max(2),
            enable_error_recovery: false, // Skip recovery for max performance
            enable_performance_monitoring: false, // Skip monitoring overhead
            enable_auto_strategy_selection: true,
            custom_thresholds: Some(StrategyThresholds {
                simple_to_indexed: 64 * 1024,    // 64KB - more aggressive
                indexed_to_streaming: 512 * 1024, // 512KB - more aggressive
            }),
            enable_detailed_logging: false,
            max_memory_usage: 1024 * 1024 * 1024, // 1GB
            conversion_timeout: Some(Duration::from_secs(600)), // 10 minutes
        }
    }

    /// Create a configuration optimized for reliability
    pub fn reliability_first() -> Self {
        Self {
            optimization_level: OptimizationLevel::Balanced,
            enable_parallel_processing: false, // Single-threaded for reliability
            max_concurrent_exports: 1,
            enable_error_recovery: true,
            enable_performance_monitoring: true,
            enable_auto_strategy_selection: true,
            custom_thresholds: None, // Use conservative defaults
            enable_detailed_logging: true,
            max_memory_usage: 256 * 1024 * 1024, // 256MB - conservative
            conversion_timeout: Some(Duration::from_secs(1800)), // 30 minutes
        }
    }

    /// Create a configuration optimized for memory usage
    pub fn memory_efficient() -> Self {
        Self {
            optimization_level: OptimizationLevel::Minimal,
            enable_parallel_processing: false,
            max_concurrent_exports: 1,
            enable_error_recovery: true,
            enable_performance_monitoring: false,
            enable_auto_strategy_selection: true,
            custom_thresholds: Some(StrategyThresholds {
                simple_to_indexed: 32 * 1024,    // 32KB - very conservative
                indexed_to_streaming: 128 * 1024, // 128KB - very conservative
            }),
            enable_detailed_logging: false,
            max_memory_usage: 64 * 1024 * 1024, // 64MB - very conservative
            conversion_timeout: Some(Duration::from_secs(900)), // 15 minutes
        }
    }

    /// Validate the configuration and return a validated copy
    pub fn validate(self) -> Result<Self, BinaryExportError> {
        // Validate concurrent exports
        if self.max_concurrent_exports == 0 {
            return Err(BinaryExportError::CorruptedData(
                "max_concurrent_exports must be at least 1".to_string()
            ));
        }

        if self.max_concurrent_exports > 32 {
            return Err(BinaryExportError::CorruptedData(
                "max_concurrent_exports should not exceed 32 for optimal performance".to_string()
            ));
        }

        // Validate memory usage
        if self.max_memory_usage < 16 * 1024 * 1024 { // 16MB minimum
            return Err(BinaryExportError::CorruptedData(
                "max_memory_usage must be at least 16MB".to_string()
            ));
        }

        if self.max_memory_usage > 8 * 1024 * 1024 * 1024 { // 8GB maximum
            return Err(BinaryExportError::CorruptedData(
                "max_memory_usage should not exceed 8GB".to_string()
            ));
        }

        // Validate timeout
        if let Some(timeout) = self.conversion_timeout {
            if timeout < Duration::from_secs(10) {
                return Err(BinaryExportError::CorruptedData(
                    "conversion_timeout must be at least 10 seconds".to_string()
                ));
            }

            if timeout > Duration::from_secs(7200) { // 2 hours
                return Err(BinaryExportError::CorruptedData(
                    "conversion_timeout should not exceed 2 hours".to_string()
                ));
            }
        }

        // Validate custom thresholds
        if let Some(ref thresholds) = self.custom_thresholds {
            if thresholds.simple_to_indexed >= thresholds.indexed_to_streaming {
                return Err(BinaryExportError::CorruptedData(
                    "simple_to_indexed threshold must be less than indexed_to_streaming threshold".to_string()
                ));
            }

            if thresholds.simple_to_indexed < 1024 { // 1KB minimum
                return Err(BinaryExportError::CorruptedData(
                    "simple_to_indexed threshold must be at least 1KB".to_string()
                ));
            }

            if thresholds.indexed_to_streaming > 100 * 1024 * 1024 { // 100MB maximum
                return Err(BinaryExportError::CorruptedData(
                    "indexed_to_streaming threshold should not exceed 100MB".to_string()
                ));
            }
        }

        // Validate parallel processing settings
        if self.enable_parallel_processing && self.max_concurrent_exports == 1 {
            info!("Parallel processing enabled but max_concurrent_exports is 1 - consider increasing for better performance");
        }

        Ok(self)
    }

    /// Update configuration at runtime with validation
    pub fn update_runtime_settings(&mut self, updates: RuntimeConfigUpdate) -> Result<(), BinaryExportError> {
        let mut updated_config = self.clone();

        if let Some(level) = updates.optimization_level {
            updated_config.optimization_level = level;
        }

        if let Some(parallel) = updates.enable_parallel_processing {
            updated_config.enable_parallel_processing = parallel;
        }

        if let Some(max_exports) = updates.max_concurrent_exports {
            updated_config.max_concurrent_exports = max_exports;
        }

        if let Some(error_recovery) = updates.enable_error_recovery {
            updated_config.enable_error_recovery = error_recovery;
        }

        if let Some(monitoring) = updates.enable_performance_monitoring {
            updated_config.enable_performance_monitoring = monitoring;
        }

        if let Some(logging) = updates.enable_detailed_logging {
            updated_config.enable_detailed_logging = logging;
        }

        if let Some(memory) = updates.max_memory_usage {
            updated_config.max_memory_usage = memory;
        }

        if let Some(timeout) = updates.conversion_timeout {
            updated_config.conversion_timeout = timeout;
        }

        // Validate the updated configuration
        let validated_config = updated_config.validate()?;
        
        // Apply the validated changes
        *self = validated_config;
        
        info!("Runtime configuration updated successfully");
        Ok(())
    }

    /// Get a summary of the current configuration
    pub fn summary(&self) -> ConfigurationSummary {
        ConfigurationSummary {
            optimization_level: self.optimization_level,
            parallel_processing: self.enable_parallel_processing,
            max_concurrent_exports: self.max_concurrent_exports,
            error_recovery: self.enable_error_recovery,
            performance_monitoring: self.enable_performance_monitoring,
            auto_strategy_selection: self.enable_auto_strategy_selection,
            has_custom_thresholds: self.custom_thresholds.is_some(),
            detailed_logging: self.enable_detailed_logging,
            max_memory_mb: self.max_memory_usage / (1024 * 1024),
            timeout_seconds: self.conversion_timeout.map(|t| t.as_secs()),
        }
    }
}

impl Default for SelectiveConversionConfig {
    fn default() -> Self {
        Self {
            optimization_level: OptimizationLevel::Balanced,
            enable_parallel_processing: true,
            max_concurrent_exports: 4,
            enable_error_recovery: true,
            enable_performance_monitoring: true,
            enable_auto_strategy_selection: true,
            custom_thresholds: None,
            enable_detailed_logging: false,
            max_memory_usage: 512 * 1024 * 1024, // 512MB
            conversion_timeout: Some(Duration::from_secs(300)), // 5 minutes
        }
    }
}

impl Default for StrategyThresholds {
    fn default() -> Self {
        Self {
            simple_to_indexed: 150 * 1024,    // 150KB
            indexed_to_streaming: 1024 * 1024, // 1MB
        }
    }
}

impl OptimizedBinaryToJsonConverter {
    /// Create a new optimized converter with default configuration
    pub fn new() -> Result<Self, BinaryExportError> {
        Self::with_config(SelectiveConversionConfig::default())
    }

    /// Create a new optimized converter with custom configuration
    pub fn with_config(config: SelectiveConversionConfig) -> Result<Self, BinaryExportError> {
        info!("Creating OptimizedBinaryToJsonConverter with optimization level: {:?}", 
              config.optimization_level);

        // Create adaptive exporter with custom thresholds if provided
        let adaptive_exporter = if let Some(ref thresholds) = config.custom_thresholds {
            let adaptive_config = AdaptiveExportConfig {
                small_file_threshold: thresholds.simple_to_indexed,
                streaming_threshold: thresholds.indexed_to_streaming,
                ..Default::default()
            };
            AdaptiveMultiJsonExporter::new_with_config(adaptive_config)
        } else {
            AdaptiveMultiJsonExporter::new()
        };

        // Create selective exporter with appropriate configuration
        let selective_config = Self::build_selective_config(&config)?;
        let selective_exporter = SelectiveJsonExporter::with_config(selective_config)?;

        Ok(Self {
            config,
            adaptive_exporter,
            selective_exporter,
            stats: ConversionStats::default(),
        })
    }

    /// Main entry point for binary-to-JSON conversion
    pub fn convert_binary_to_json<P: AsRef<Path>>(
        &mut self,
        binary_path: P,
        output_dir: P,
        json_types: &[JsonType],
    ) -> Result<ConversionResult, BinaryExportError> {
        let start_time = Instant::now();
        let binary_path = binary_path.as_ref();
        let output_dir = output_dir.as_ref();

        info!("Starting optimized binary-to-JSON conversion for: {:?}", binary_path);

        // Validate inputs
        self.validate_conversion_inputs(binary_path, output_dir, json_types)?;

        // Perform conversion with error recovery
        let result = self.perform_conversion_with_recovery(binary_path, output_dir, json_types);

        // Update statistics
        let processing_time = start_time.elapsed();
        self.update_conversion_stats(&result, processing_time);

        // Log results
        if let Ok(ref success_result) = result {
            info!("Conversion completed successfully in {:?} - {} allocations processed", 
                  processing_time, success_result.allocations_processed);
        } else if let Err(ref error) = result {
            error!("Conversion failed after {:?}: {}", processing_time, error);
        }

        result
    }

    /// Convert binary to JSON with selective field inclusion
    pub fn convert_selective<P: AsRef<Path>>(
        &mut self,
        binary_path: P,
        output_path: P,
        json_type: JsonType,
        selected_fields: Option<&[AllocationField]>,
    ) -> Result<ConversionResult, BinaryExportError> {
        let start_time = Instant::now();
        let binary_path = binary_path.as_ref();
        let output_path = output_path.as_ref();

        info!("Starting selective binary-to-JSON conversion for: {:?}", binary_path);

        // Convert selected fields to HashSet
        let fields_set = if let Some(fields) = selected_fields {
            fields.iter().cloned().collect()
        } else {
            json_type.required_fields().into_iter().collect()
        };

        // Use selective exporter for fine-grained control
        let export_stats = self.selective_exporter.export_to_json_selective(
            binary_path,
            output_path,
            &fields_set,
            &[], // No additional filters for now
        )?;

        let processing_time = start_time.elapsed();
        let result = ConversionResult {
            output_files: vec![output_path.to_path_buf()],
            allocations_processed: export_stats.total_allocations_exported,
            bytes_written: export_stats.total_bytes_written,
            processing_time,
            strategy_used: "selective".to_string(),
            optimization_efficiency: export_stats.export_efficiency(),
            memory_peak_usage: export_stats.json_writer_stats.peak_memory_usage,
        };

        // Update statistics
        self.update_conversion_stats(&Ok(result.clone()), processing_time);

        info!("Selective conversion completed in {:?}", processing_time);
        Ok(result)
    }

    /// Convert binary to multiple JSON types with automatic optimization
    pub fn convert_multi_json<P: AsRef<Path>>(
        &mut self,
        binary_path: P,
        output_dir: P,
        json_types: &[JsonType],
    ) -> Result<ConversionResult, BinaryExportError> {
        let start_time = Instant::now();
        let binary_path = binary_path.as_ref();
        let output_dir = output_dir.as_ref();

        info!("Starting multi-JSON conversion for: {:?}", binary_path);

        // Use adaptive exporter for intelligent strategy selection
        let base_name = binary_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        
        let export_stats = self.adaptive_exporter.export_adaptive(
            binary_path,
            base_name,
            json_types,
        )?;

        let processing_time = start_time.elapsed();
        let result = self.build_conversion_result_from_multi_stats(
            &export_stats,
            output_dir,
            json_types,
            processing_time,
        );

        // Update statistics
        self.update_conversion_stats(&Ok(result.clone()), processing_time);

        info!("Multi-JSON conversion completed in {:?}", processing_time);
        Ok(result)
    }

    /// Get current conversion statistics
    pub fn get_stats(&self) -> &ConversionStats {
        &self.stats
    }

    /// Reset conversion statistics
    pub fn reset_stats(&mut self) {
        self.stats = ConversionStats::default();
        info!("Conversion statistics reset");
    }

    /// Update converter configuration at runtime
    pub fn update_config(&mut self, new_config: SelectiveConversionConfig) -> Result<(), BinaryExportError> {
        info!("Updating converter configuration");
        
        // Validate the new configuration
        let validated_config = new_config.validate()?;
        
        // Rebuild components if necessary
        if self.config.optimization_level != validated_config.optimization_level ||
           self.config.custom_thresholds != validated_config.custom_thresholds {
            
            // Recreate adaptive exporter with new thresholds
            self.adaptive_exporter = if let Some(ref thresholds) = validated_config.custom_thresholds {
                let adaptive_config = AdaptiveExportConfig {
                    small_file_threshold: thresholds.simple_to_indexed,
                    streaming_threshold: thresholds.indexed_to_streaming,
                    ..Default::default()
                };
                AdaptiveMultiJsonExporter::new_with_config(adaptive_config)
            } else {
                AdaptiveMultiJsonExporter::new()
            };

            // Recreate selective exporter with new configuration
            let selective_config = Self::build_selective_config(&validated_config)?;
            self.selective_exporter = SelectiveJsonExporter::with_config(selective_config)?;
        }

        self.config = validated_config;
        Ok(())
    }

    /// Update configuration with runtime settings
    pub fn update_runtime_config(&mut self, updates: RuntimeConfigUpdate) -> Result<(), BinaryExportError> {
        info!("Updating runtime configuration settings");
        
        let mut new_config = self.config.clone();
        new_config.update_runtime_settings(updates)?;
        
        self.update_config(new_config)
    }

    /// Get current configuration summary
    pub fn get_config_summary(&self) -> ConfigurationSummary {
        self.config.summary()
    }

    /// Get current configuration
    pub fn get_config(&self) -> &SelectiveConversionConfig {
        &self.config
    }

    // Private helper methods

    fn build_selective_config(config: &SelectiveConversionConfig) -> Result<SelectiveJsonExportConfig, BinaryExportError> {
        let mut selective_config = SelectiveJsonExportConfig::default();
        
        selective_config.enable_parallel_processing = config.enable_parallel_processing;
        selective_config.max_concurrent_exports = config.max_concurrent_exports;
        selective_config.enable_error_recovery = config.enable_error_recovery;

        Ok(selective_config)
    }

    fn validate_conversion_inputs<P: AsRef<Path>>(
        &self,
        binary_path: P,
        output_dir: P,
        json_types: &[JsonType],
    ) -> Result<(), BinaryExportError> {
        let binary_path = binary_path.as_ref();
        let output_dir = output_dir.as_ref();

        // Validate binary file exists
        if !binary_path.exists() {
            return Err(BinaryExportError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Binary file not found: {:?}", binary_path)
            )));
        }

        // Validate output directory
        if let Some(parent) = output_dir.parent() {
            if !parent.exists() {
                return Err(BinaryExportError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Output directory not found: {:?}", parent)
                )));
            }
        }

        // Validate JSON types
        if json_types.is_empty() {
            return Err(BinaryExportError::CorruptedData(
                "At least one JSON type must be specified".to_string()
            ));
        }

        Ok(())
    }

    fn perform_conversion_with_recovery<P: AsRef<Path>>(
        &mut self,
        binary_path: P,
        output_dir: P,
        json_types: &[JsonType],
    ) -> Result<ConversionResult, BinaryExportError> {
        let binary_path = binary_path.as_ref();
        let output_dir = output_dir.as_ref();

        // First attempt with adaptive strategy
        match self.convert_multi_json(binary_path, output_dir, json_types) {
            Ok(result) => Ok(result),
            Err(error) if self.config.enable_error_recovery => {
                warn!("Primary conversion failed, attempting recovery: {}", error);
                self.stats.error_recoveries += 1;
                
                // Fallback to selective conversion for each JSON type
                self.fallback_conversion(binary_path, output_dir, json_types)
            }
            Err(error) => Err(error),
        }
    }

    fn fallback_conversion<P: AsRef<Path>>(
        &mut self,
        binary_path: P,
        output_dir: P,
        json_types: &[JsonType],
    ) -> Result<ConversionResult, BinaryExportError> {
        let binary_path = binary_path.as_ref();
        let output_dir = output_dir.as_ref();
        let start_time = Instant::now();

        info!("Performing fallback conversion using selective strategy");

        let mut output_files = Vec::new();
        let mut total_allocations = 0;
        let mut total_bytes = 0;

        for &json_type in json_types {
            let output_path = output_dir.join(format!("{}.json", json_type.filename_suffix()));
            
            match self.convert_selective(binary_path, &output_path, json_type, None) {
                Ok(result) => {
                    output_files.extend(result.output_files);
                    total_allocations += result.allocations_processed;
                    total_bytes += result.bytes_written;
                }
                Err(error) => {
                    warn!("Failed to convert {}: {}", json_type.filename_suffix(), error);
                    // Continue with other types
                }
            }
        }

        if output_files.is_empty() {
            return Err(BinaryExportError::CorruptedData(
                "All fallback conversions failed".to_string()
            ));
        }

        Ok(ConversionResult {
            output_files,
            allocations_processed: total_allocations,
            bytes_written: total_bytes,
            processing_time: start_time.elapsed(),
            strategy_used: "fallback_selective".to_string(),
            optimization_efficiency: 0.0, // Fallback has no optimization
            memory_peak_usage: 0,
        })
    }

    fn build_conversion_result_from_multi_stats(
        &self,
        stats: &MultiExportStats,
        output_dir: &Path,
        json_types: &[JsonType],
        processing_time: Duration,
    ) -> ConversionResult {
        let output_files: Vec<PathBuf> = json_types
            .iter()
            .map(|json_type| output_dir.join(format!("{}.json", json_type.filename_suffix())))
            .collect();

        ConversionResult {
            output_files,
            allocations_processed: stats.total_records as u64,
            bytes_written: stats.per_json_stats.values().map(|s| s.json_writer_stats.bytes_written).sum(),
            processing_time,
            strategy_used: format!("{:?}", stats.strategy_used),
            optimization_efficiency: stats.cache_hit_rate,
            memory_peak_usage: stats.max_memory_used,
        }
    }

    fn update_conversion_stats(&mut self, result: &Result<ConversionResult, BinaryExportError>, processing_time: Duration) {
        self.stats.total_conversions += 1;
        self.stats.total_processing_time += processing_time;

        match result {
            Ok(success_result) => {
                self.stats.successful_conversions += 1;
                self.stats.total_allocations_processed += success_result.allocations_processed;
                self.stats.total_bytes_written += success_result.bytes_written;
                
                // Update strategy usage
                *self.stats.strategy_usage.entry(success_result.strategy_used.clone()).or_insert(0) += 1;
                
                // Update peak memory usage
                if success_result.memory_peak_usage > self.stats.peak_memory_usage {
                    self.stats.peak_memory_usage = success_result.memory_peak_usage;
                }
            }
            Err(_) => {
                self.stats.failed_conversions += 1;
            }
        }

        // Calculate average processing speed
        if self.stats.total_processing_time.as_secs_f64() > 0.0 {
            self.stats.average_processing_speed = 
                self.stats.total_allocations_processed as f64 / self.stats.total_processing_time.as_secs_f64();
        }
    }
}

/// Result of a binary-to-JSON conversion operation
#[derive(Debug, Clone)]
pub struct ConversionResult {
    /// Paths to the generated JSON files
    pub output_files: Vec<PathBuf>,
    
    /// Number of allocation records processed
    pub allocations_processed: u64,
    
    /// Total bytes written to output files
    pub bytes_written: u64,
    
    /// Time taken for the conversion
    pub processing_time: Duration,
    
    /// Strategy used for the conversion
    pub strategy_used: String,
    
    /// Optimization efficiency (0.0 to 1.0)
    pub optimization_efficiency: f64,
    
    /// Peak memory usage during conversion
    pub memory_peak_usage: usize,
}

impl ConversionResult {
    /// Calculate processing throughput in allocations per second
    pub fn throughput(&self) -> f64 {
        if self.processing_time.as_secs_f64() > 0.0 {
            self.allocations_processed as f64 / self.processing_time.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Calculate write throughput in bytes per second
    pub fn write_throughput(&self) -> f64 {
        if self.processing_time.as_secs_f64() > 0.0 {
            self.bytes_written as f64 / self.processing_time.as_secs_f64()
        } else {
            0.0
        }
    }
}

impl ConversionStats {
    /// Calculate overall success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_conversions > 0 {
            self.successful_conversions as f64 / self.total_conversions as f64
        } else {
            0.0
        }
    }

    /// Calculate average processing time per conversion
    pub fn average_processing_time(&self) -> Duration {
        if self.total_conversions > 0 {
            self.total_processing_time / self.total_conversions as u32
        } else {
            Duration::ZERO
        }
    }

    /// Get the most frequently used strategy
    pub fn most_used_strategy(&self) -> Option<&String> {
        self.strategy_usage
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(strategy, _)| strategy)
    }
}

// Configuration builder for easier setup
pub struct SelectiveConversionConfigBuilder {
    config: SelectiveConversionConfig,
}

impl SelectiveConversionConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: SelectiveConversionConfig::default(),
        }
    }

    pub fn optimization_level(mut self, level: OptimizationLevel) -> Self {
        self.config.optimization_level = level;
        self
    }

    pub fn parallel_processing(mut self, enabled: bool) -> Self {
        self.config.enable_parallel_processing = enabled;
        self
    }

    pub fn max_concurrent_exports(mut self, max: usize) -> Self {
        self.config.max_concurrent_exports = max;
        self
    }

    pub fn error_recovery(mut self, enabled: bool) -> Self {
        self.config.enable_error_recovery = enabled;
        self
    }

    pub fn performance_monitoring(mut self, enabled: bool) -> Self {
        self.config.enable_performance_monitoring = enabled;
        self
    }

    pub fn auto_strategy_selection(mut self, enabled: bool) -> Self {
        self.config.enable_auto_strategy_selection = enabled;
        self
    }

    pub fn custom_thresholds(mut self, thresholds: StrategyThresholds) -> Self {
        self.config.custom_thresholds = Some(thresholds);
        self
    }

    pub fn detailed_logging(mut self, enabled: bool) -> Self {
        self.config.enable_detailed_logging = enabled;
        self
    }

    pub fn max_memory_usage(mut self, bytes: usize) -> Self {
        self.config.max_memory_usage = bytes;
        self
    }

    pub fn conversion_timeout(mut self, timeout: Duration) -> Self {
        self.config.conversion_timeout = Some(timeout);
        self
    }

    /// Build the configuration with validation
    pub fn build(self) -> Result<SelectiveConversionConfig, BinaryExportError> {
        self.config.validate()
    }

    /// Build the configuration without validation (for testing)
    pub fn build_unchecked(self) -> SelectiveConversionConfig {
        self.config
    }
}

impl Default for SelectiveConversionConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_optimized_converter_creation() {
        let converter = OptimizedBinaryToJsonConverter::new();
        assert!(converter.is_ok());
    }

    #[test]
    fn test_config_builder() {
        let config = SelectiveConversionConfigBuilder::new()
            .optimization_level(OptimizationLevel::Maximum)
            .parallel_processing(false)
            .max_concurrent_exports(2)
            .error_recovery(false)
            .build()
            .unwrap();

        assert_eq!(config.optimization_level, OptimizationLevel::Maximum);
        assert!(!config.enable_parallel_processing);
        assert_eq!(config.max_concurrent_exports, 2);
        assert!(!config.enable_error_recovery);
    }

    #[test]
    fn test_config_validation() {
        // Test invalid max_concurrent_exports
        let result = SelectiveConversionConfigBuilder::new()
            .max_concurrent_exports(0)
            .build();
        assert!(result.is_err());

        // Test invalid memory usage
        let result = SelectiveConversionConfigBuilder::new()
            .max_memory_usage(1024) // Too small
            .build();
        assert!(result.is_err());

        // Test invalid timeout
        let result = SelectiveConversionConfigBuilder::new()
            .conversion_timeout(Duration::from_secs(5)) // Too short
            .build();
        assert!(result.is_err());

        // Test valid configuration
        let result = SelectiveConversionConfigBuilder::new()
            .max_concurrent_exports(4)
            .max_memory_usage(64 * 1024 * 1024)
            .conversion_timeout(Duration::from_secs(60))
            .build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_preset_configurations() {
        let performance_config = SelectiveConversionConfig::performance_first();
        assert_eq!(performance_config.optimization_level, OptimizationLevel::Maximum);
        assert!(performance_config.enable_parallel_processing);
        assert!(!performance_config.enable_error_recovery);

        let reliability_config = SelectiveConversionConfig::reliability_first();
        assert_eq!(reliability_config.optimization_level, OptimizationLevel::Balanced);
        assert!(!reliability_config.enable_parallel_processing);
        assert!(reliability_config.enable_error_recovery);

        let memory_config = SelectiveConversionConfig::memory_efficient();
        assert_eq!(memory_config.optimization_level, OptimizationLevel::Minimal);
        assert!(!memory_config.enable_parallel_processing);
        assert!(memory_config.max_memory_usage < 128 * 1024 * 1024);
    }

    #[test]
    fn test_runtime_config_update() {
        let mut config = SelectiveConversionConfig::default();
        
        let update = RuntimeConfigUpdate {
            optimization_level: Some(OptimizationLevel::Maximum),
            enable_parallel_processing: Some(false),
            max_concurrent_exports: Some(8),
            ..Default::default()
        };

        let result = config.update_runtime_settings(update);
        assert!(result.is_ok());
        
        assert_eq!(config.optimization_level, OptimizationLevel::Maximum);
        assert!(!config.enable_parallel_processing);
        assert_eq!(config.max_concurrent_exports, 8);
    }

    #[test]
    fn test_config_summary() {
        let config = SelectiveConversionConfig::performance_first();
        let summary = config.summary();
        
        assert_eq!(summary.optimization_level, OptimizationLevel::Maximum);
        assert!(summary.parallel_processing);
        assert!(!summary.error_recovery);
        assert!(summary.has_custom_thresholds);
        assert!(summary.max_memory_mb > 0);
    }

    #[test]
    fn test_conversion_result_calculations() {
        let result = ConversionResult {
            output_files: vec![PathBuf::from("test.json")],
            allocations_processed: 1000,
            bytes_written: 50000,
            processing_time: Duration::from_secs(1),
            strategy_used: "adaptive".to_string(),
            optimization_efficiency: 0.8,
            memory_peak_usage: 1024 * 1024,
        };

        assert_eq!(result.throughput(), 1000.0);
        assert_eq!(result.write_throughput(), 50000.0);
    }

    #[test]
    fn test_conversion_stats_calculations() {
        let mut stats = ConversionStats::default();
        stats.total_conversions = 10;
        stats.successful_conversions = 8;
        stats.total_processing_time = Duration::from_secs(10);

        assert_eq!(stats.success_rate(), 0.8);
        assert_eq!(stats.average_processing_time(), Duration::from_secs(1));
    }

    #[test]
    fn test_strategy_thresholds() {
        let thresholds = StrategyThresholds::default();
        assert_eq!(thresholds.simple_to_indexed, 150 * 1024);
        assert_eq!(thresholds.indexed_to_streaming, 1024 * 1024);
    }

    #[test]
    fn test_optimization_levels() {
        assert_eq!(OptimizationLevel::Minimal, OptimizationLevel::Minimal);
        assert_ne!(OptimizationLevel::Minimal, OptimizationLevel::Maximum);
    }
}