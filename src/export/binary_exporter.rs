//! Binary exporter for high-performance memory tracking data export
//!
//! This module implements the BinaryExporter struct that converts memory tracking data
//! into the optimized binary format. It supports compression, parallel processing,
//! and automatic optimization based on data characteristics.

use crate::analysis::unsafe_ffi_tracker::get_global_unsafe_ffi_tracker;
use crate::core::tracker::MemoryTracker;
use crate::core::types::{AllocationInfo, MemoryStats, TrackingResult, TypeMemoryUsage};
use crate::export::binary_format::{
    BinaryEncoder, BinaryFile, BinaryHeader, CompressionEngine, CompressionType, SectionDirectory,
    SectionEntry, SectionType, StringTable, TypeTable,
};
use crate::export::memory_mapping::{
    MemoryMappedWriter, MemoryMappingConfig, MemoryMappingError, MemoryUsageMonitor,
};
use crate::export::simd_optimizations::{SimdCapability, SimdProcessor};
use crate::export::zero_copy::{
    StringOptimizer, VectorizedProcessor, ZeroCopyBufferPool, ZeroCopyWriter,
};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Binary export options configuration with comprehensive settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BinaryExportOptions {
    /// Compression type to use
    pub compression: CompressionType,
    /// Whether to include allocation history
    pub include_history: bool,
    /// Whether to include FFI analysis data
    pub include_ffi_analysis: bool,
    /// Buffer size for I/O operations (default: 256KB)
    pub buffer_size: usize,
    /// Enable parallel section encoding
    pub enable_parallel_encoding: bool,
    /// Thread count for parallel processing (None = auto-detect)
    pub thread_count: Option<usize>,
    /// Enable progress reporting
    pub enable_progress_reporting: bool,
    /// Target memory usage limit in bytes (0 = no limit)
    pub memory_limit: usize,
    /// Enable automatic compression selection based on data characteristics
    pub auto_compression: bool,
    /// Minimum data size threshold for compression (bytes)
    pub compression_threshold: usize,
    /// Enable security violation analysis
    pub enable_security_analysis: bool,
    /// Section selection configuration
    pub section_selection: SectionSelectionConfig,
    /// Validation settings
    pub validation: ValidationConfig,
    /// Performance tuning settings
    pub performance: PerformanceConfig,
    /// Compatibility settings
    pub compatibility: CompatibilityConfig,
}

/// Section selection configuration for fine-grained control
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SectionSelectionConfig {
    /// Include memory statistics section
    pub include_memory_stats: bool,
    /// Include active allocations section
    pub include_active_allocations: bool,
    /// Include allocation history section
    pub include_allocation_history: bool,
    /// Include type memory usage section
    pub include_type_memory_usage: bool,
    /// Include FFI analysis section
    pub include_ffi_analysis: bool,
    /// Include lifecycle analysis section
    pub include_lifecycle_analysis: bool,
    /// Include performance data section
    pub include_performance_data: bool,
    /// Include variable registry section
    pub include_variable_registry: bool,
    /// Include security violations section
    pub include_security_violations: bool,
    /// Include memory passports section
    pub include_memory_passports: bool,
}

/// Validation configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationConfig {
    /// Enable checksum validation
    pub enable_checksums: bool,
    /// Enable data integrity checks
    pub enable_integrity_checks: bool,
    /// Enable schema validation
    pub enable_schema_validation: bool,
    /// Validation level (strict, normal, lenient)
    pub validation_level: ValidationLevel,
}

/// Performance configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceConfig {
    /// Use memory mapping for large files
    pub use_memory_mapping: bool,
    /// Memory mapping configuration
    pub memory_mapping_config: Option<MemoryMappingConfig>,
    /// Enable zero-copy optimizations
    pub enable_zero_copy: bool,
    /// Enable SIMD optimizations
    pub enable_simd: bool,
    /// Cache size for frequently accessed data
    pub cache_size: usize,
    /// Batch size for processing
    pub batch_size: usize,
}

/// Compatibility configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompatibilityConfig {
    /// Target format version
    pub target_version: (u16, u16), // (major, minor)
    /// Enable backward compatibility mode
    pub backward_compatibility: bool,
    /// Enable forward compatibility features
    pub forward_compatibility: bool,
    /// Compatibility level
    pub compatibility_level: CompatibilityLevel,
}

/// Validation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ValidationLevel {
    /// Strict validation - all checks enabled
    Strict,
    /// Normal validation - standard checks
    Normal,
    /// Lenient validation - minimal checks for performance
    Lenient,
    /// No validation - maximum performance
    None,
}

/// Compatibility levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CompatibilityLevel {
    /// Maximum compatibility - works with all versions
    Maximum,
    /// Standard compatibility - works with recent versions
    Standard,
    /// Minimal compatibility - latest features only
    Minimal,
}

impl Default for SectionSelectionConfig {
    fn default() -> Self {
        Self {
            include_memory_stats: true,
            include_active_allocations: true,
            include_allocation_history: true,
            include_type_memory_usage: true,
            include_ffi_analysis: true,
            include_lifecycle_analysis: false, // Optional by default
            include_performance_data: true,
            include_variable_registry: true,
            include_security_violations: false, // Optional by default
            include_memory_passports: false,    // Optional by default
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            enable_checksums: true,
            enable_integrity_checks: true,
            enable_schema_validation: false, // Disabled for performance
            validation_level: ValidationLevel::Normal,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            use_memory_mapping: false,   // Disabled by default for compatibility
            memory_mapping_config: None, // Use default config when enabled
            enable_zero_copy: true,
            enable_simd: false,    // Disabled by default for compatibility
            cache_size: 64 * 1024, // 64KB cache
            batch_size: 1000,
        }
    }
}

impl Default for CompatibilityConfig {
    fn default() -> Self {
        Self {
            target_version: (1, 0), // Current version
            backward_compatibility: true,
            forward_compatibility: false,
            compatibility_level: CompatibilityLevel::Standard,
        }
    }
}

impl Default for BinaryExportOptions {
    fn default() -> Self {
        Self {
            compression: CompressionType::Lz4, // Balanced speed and size
            include_history: true,
            include_ffi_analysis: true,
            buffer_size: 256 * 1024, // 256KB
            enable_parallel_encoding: true,
            thread_count: None, // Auto-detect
            enable_progress_reporting: false,
            memory_limit: 0, // No limit
            auto_compression: true,
            compression_threshold: 1024, // 1KB
            enable_security_analysis: false,
            section_selection: SectionSelectionConfig::default(),
            validation: ValidationConfig::default(),
            performance: PerformanceConfig::default(),
            compatibility: CompatibilityConfig::default(),
        }
    }
}

impl BinaryExportOptions {
    /// Create new options with specified compression
    pub fn with_compression(compression: CompressionType) -> Self {
        Self {
            compression,
            ..Default::default()
        }
    }

    /// Create fast export options (minimal compression, parallel processing)
    pub fn fast() -> Self {
        let mut section_selection = SectionSelectionConfig::default();
        section_selection.include_allocation_history = false;
        section_selection.include_lifecycle_analysis = false;
        section_selection.include_security_violations = false;
        section_selection.include_memory_passports = false;

        let validation = ValidationConfig {
            enable_checksums: false,
            enable_integrity_checks: false,
            enable_schema_validation: false,
            validation_level: ValidationLevel::None,
        };

        let performance = PerformanceConfig {
            use_memory_mapping: true,
            memory_mapping_config: None,
            enable_zero_copy: true,
            enable_simd: true,
            cache_size: 128 * 1024, // Larger cache for speed
            batch_size: 2000,       // Larger batches
        };

        Self {
            compression: CompressionType::Lz4,
            include_history: false,
            include_ffi_analysis: false,
            enable_parallel_encoding: true,
            auto_compression: false,
            enable_security_analysis: false,
            section_selection,
            validation,
            performance,
            ..Default::default()
        }
    }

    /// Create comprehensive export options (maximum data, best compression)
    pub fn comprehensive() -> Self {
        let section_selection = SectionSelectionConfig {
            include_memory_stats: true,
            include_active_allocations: true,
            include_allocation_history: true,
            include_type_memory_usage: true,
            include_ffi_analysis: true,
            include_lifecycle_analysis: true,
            include_performance_data: true,
            include_variable_registry: true,
            include_security_violations: true,
            include_memory_passports: true,
        };

        let validation = ValidationConfig {
            enable_checksums: true,
            enable_integrity_checks: true,
            enable_schema_validation: true,
            validation_level: ValidationLevel::Strict,
        };

        Self {
            compression: CompressionType::Zstd,
            include_history: true,
            include_ffi_analysis: true,
            enable_parallel_encoding: true,
            auto_compression: true,
            enable_security_analysis: true,
            section_selection,
            validation,
            ..Default::default()
        }
    }

    /// Create options optimized for production use
    pub fn production() -> Self {
        let mut section_selection = SectionSelectionConfig::default();
        section_selection.include_security_violations = true; // Important for production

        let validation = ValidationConfig {
            enable_checksums: true,
            enable_integrity_checks: true,
            enable_schema_validation: false, // Skip for performance
            validation_level: ValidationLevel::Normal,
        };

        let performance = PerformanceConfig {
            use_memory_mapping: true,
            memory_mapping_config: None,
            enable_zero_copy: true,
            enable_simd: false,     // Conservative for compatibility
            cache_size: 256 * 1024, // Larger cache
            batch_size: 1500,
        };

        Self {
            compression: CompressionType::Lz4,
            enable_parallel_encoding: true,
            enable_progress_reporting: true, // Useful in production
            section_selection,
            validation,
            performance,
            ..Default::default()
        }
    }

    // ============================================================================
    // Configuration Builder Methods
    // ============================================================================

    /// Set compression type
    pub fn compression(mut self, compression: CompressionType) -> Self {
        self.compression = compression;
        self
    }

    /// Set whether to include history
    pub fn include_history(mut self, include: bool) -> Self {
        self.include_history = include;
        self.section_selection.include_allocation_history = include;
        self
    }

    /// Set whether to include FFI analysis
    pub fn include_ffi_analysis(mut self, include: bool) -> Self {
        self.include_ffi_analysis = include;
        self.section_selection.include_ffi_analysis = include;
        self
    }

    /// Set buffer size
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Set parallel encoding
    pub fn parallel_encoding(mut self, enabled: bool) -> Self {
        self.enable_parallel_encoding = enabled;
        self
    }

    /// Set thread count
    pub fn thread_count(mut self, count: Option<usize>) -> Self {
        self.thread_count = count;
        self
    }

    /// Set memory limit
    pub fn memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = limit;
        self
    }

    /// Enable auto compression
    pub fn auto_compression(mut self, enabled: bool) -> Self {
        self.auto_compression = enabled;
        self
    }

    /// Set section selection configuration
    pub fn section_selection(mut self, config: SectionSelectionConfig) -> Self {
        self.section_selection = config;
        self
    }

    /// Set validation configuration
    pub fn validation(mut self, config: ValidationConfig) -> Self {
        self.validation = config;
        self
    }

    /// Set performance configuration
    pub fn performance(mut self, config: PerformanceConfig) -> Self {
        self.performance = config;
        self
    }

    /// Set compatibility configuration
    pub fn compatibility(mut self, config: CompatibilityConfig) -> Self {
        self.compatibility = config;
        self
    }

    /// Enable progress reporting
    pub fn progress_reporting(mut self, enabled: bool) -> Self {
        self.enable_progress_reporting = enabled;
        self
    }

    /// Enable security analysis
    pub fn security_analysis(mut self, enabled: bool) -> Self {
        self.enable_security_analysis = enabled;
        self.section_selection.include_security_violations = enabled;
        self
    }

    // ============================================================================
    // Configuration Validation and Conversion
    // ============================================================================

    /// Validate the configuration and return any issues
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate buffer size
        if self.buffer_size < 1024 {
            errors.push("Buffer size must be at least 1KB".to_string());
        }
        if self.buffer_size > 100 * 1024 * 1024 {
            errors.push("Buffer size should not exceed 100MB".to_string());
        }

        // Validate thread count
        if let Some(count) = self.thread_count {
            if count == 0 {
                errors.push("Thread count must be greater than 0".to_string());
            }
            if count > 64 {
                errors.push("Thread count should not exceed 64".to_string());
            }
        }

        // Validate memory limit
        if self.memory_limit > 0 && self.memory_limit < 10 * 1024 * 1024 {
            errors.push("Memory limit should be at least 10MB if specified".to_string());
        }

        // Validate compression threshold
        if self.compression_threshold > 100 * 1024 * 1024 {
            errors.push("Compression threshold should not exceed 100MB".to_string());
        }

        // Validate performance settings
        if self.performance.cache_size < 1024 {
            errors.push("Cache size must be at least 1KB".to_string());
        }
        if self.performance.batch_size == 0 {
            errors.push("Batch size must be greater than 0".to_string());
        }

        // Check for conflicting settings
        if self.compression == CompressionType::None && self.auto_compression {
            errors.push("Auto compression cannot be enabled with no compression".to_string());
        }

        if self.validation.validation_level == ValidationLevel::None
            && (self.validation.enable_checksums || self.validation.enable_integrity_checks)
        {
            errors
                .push("Cannot enable specific validations with validation level None".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Convert from OptimizedExportOptions for backward compatibility
    pub fn from_optimized_export_options(
        options: &crate::export::optimized_json_export::OptimizedExportOptions,
    ) -> Self {
        let mut binary_options = Self::default();

        // Map buffer size
        binary_options.buffer_size = options.buffer_size;

        // Map parallel processing
        binary_options.enable_parallel_encoding = options.parallel_processing;

        // Map thread count if available
        if let Some(thread_count) = options.thread_count {
            binary_options.thread_count = Some(thread_count);
        }

        // Map compression based on compact format setting
        if let Some(use_compact) = options.use_compact_format {
            if use_compact {
                binary_options.compression = CompressionType::Zstd;
                binary_options.auto_compression = true;
            } else {
                binary_options.compression = CompressionType::Lz4;
            }
        }

        // Map validation settings
        binary_options.validation.enable_schema_validation = options.enable_schema_validation;

        // Map security analysis
        binary_options.enable_security_analysis = options.enable_security_analysis;
        binary_options.section_selection.include_security_violations =
            options.enable_security_analysis;

        // Map FFI analysis
        binary_options.include_ffi_analysis = options.enable_enhanced_ffi_analysis;
        binary_options.section_selection.include_ffi_analysis =
            options.enable_enhanced_ffi_analysis;

        // Map memory passport tracking
        binary_options.section_selection.include_memory_passports =
            options.enable_memory_passport_tracking;

        // Map batch size
        binary_options.performance.batch_size = options.batch_size;

        binary_options
    }

    /// Convert to OptimizedExportOptions for integration
    pub fn to_optimized_export_options(
        &self,
    ) -> crate::export::optimized_json_export::OptimizedExportOptions {
        use crate::export::optimized_json_export::{OptimizationLevel, OptimizedExportOptions};

        let mut json_options = OptimizedExportOptions::default();

        // Map buffer size
        json_options.buffer_size = self.buffer_size;

        // Map parallel processing
        json_options.parallel_processing = self.enable_parallel_encoding;

        // Map thread count
        json_options.thread_count = self.thread_count;

        // Map compression to compact format
        json_options.use_compact_format = Some(match self.compression {
            CompressionType::None => false,
            CompressionType::Lz4 => true,
            CompressionType::Zstd => true,
        });

        // Map optimization level based on validation and performance settings
        json_options.optimization_level = match self.validation.validation_level {
            ValidationLevel::None => OptimizationLevel::Maximum,
            ValidationLevel::Lenient => OptimizationLevel::High,
            ValidationLevel::Normal => OptimizationLevel::Medium,
            ValidationLevel::Strict => OptimizationLevel::Low,
        };

        // Map validation settings
        json_options.enable_schema_validation = self.validation.enable_schema_validation;

        // Map security analysis
        json_options.enable_security_analysis = self.enable_security_analysis;

        // Map FFI analysis
        json_options.enable_enhanced_ffi_analysis = self.include_ffi_analysis;

        // Map memory passport tracking
        json_options.enable_memory_passport_tracking =
            self.section_selection.include_memory_passports;

        // Map batch size
        json_options.batch_size = self.performance.batch_size;

        json_options
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to serialize configuration: {}",
                e
            ))
        })?;

        std::fs::write(path, json).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to write configuration file: {}",
                e
            ))
        })?;

        Ok(())
    }

    /// Load configuration from file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> TrackingResult<Self> {
        let json = std::fs::read_to_string(path).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to read configuration file: {}",
                e
            ))
        })?;

        let options: Self = serde_json::from_str(&json).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to parse configuration: {}",
                e
            ))
        })?;

        // Validate the loaded configuration
        if let Err(errors) = options.validate() {
            return Err(crate::core::types::TrackingError::ExportError(format!(
                "Invalid configuration: {}",
                errors.join(", ")
            )));
        }

        Ok(options)
    }

    /// Get a summary of the current configuration
    pub fn summary(&self) -> String {
        format!(
            "BinaryExportOptions Summary:\n\
             - Compression: {:?}\n\
             - Buffer Size: {} KB\n\
             - Parallel Encoding: {}\n\
             - Thread Count: {:?}\n\
             - Memory Limit: {} MB\n\
             - Validation Level: {:?}\n\
             - Sections: {} enabled\n\
             - Performance Optimizations: {}",
            self.compression,
            self.buffer_size / 1024,
            self.enable_parallel_encoding,
            self.thread_count,
            if self.memory_limit > 0 {
                self.memory_limit / 1024 / 1024
            } else {
                0
            },
            self.validation.validation_level,
            self.count_enabled_sections(),
            if self.performance.enable_simd {
                "SIMD enabled"
            } else {
                "Standard"
            }
        )
    }

    /// Count the number of enabled sections
    fn count_enabled_sections(&self) -> usize {
        let mut count = 0;
        if self.section_selection.include_memory_stats {
            count += 1;
        }
        if self.section_selection.include_active_allocations {
            count += 1;
        }
        if self.section_selection.include_allocation_history {
            count += 1;
        }
        if self.section_selection.include_type_memory_usage {
            count += 1;
        }
        if self.section_selection.include_ffi_analysis {
            count += 1;
        }
        if self.section_selection.include_lifecycle_analysis {
            count += 1;
        }
        if self.section_selection.include_performance_data {
            count += 1;
        }
        if self.section_selection.include_variable_registry {
            count += 1;
        }
        if self.section_selection.include_security_violations {
            count += 1;
        }
        if self.section_selection.include_memory_passports {
            count += 1;
        }
        count
    }
}

/// Export progress information
#[derive(Debug, Clone)]
pub struct ExportProgress {
    /// Current section being processed
    pub current_section: SectionType,
    /// Number of sections completed
    pub sections_completed: usize,
    /// Total number of sections
    pub total_sections: usize,
    /// Bytes processed so far
    pub bytes_processed: usize,
    /// Estimated total bytes
    pub estimated_total_bytes: usize,
    /// Export start time
    pub start_time: std::time::Instant,
}

impl ExportProgress {
    /// Calculate completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.total_sections == 0 {
            0.0
        } else {
            (self.sections_completed as f64 / self.total_sections as f64) * 100.0
        }
    }

    /// Calculate elapsed time
    pub fn elapsed_time(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Estimate remaining time
    pub fn estimated_remaining_time(&self) -> Option<std::time::Duration> {
        let completion = self.completion_percentage();
        if completion > 0.0 && completion < 100.0 {
            let elapsed = self.elapsed_time();
            let total_estimated = elapsed.as_secs_f64() / (completion / 100.0);
            let remaining = total_estimated - elapsed.as_secs_f64();
            Some(std::time::Duration::from_secs_f64(remaining.max(0.0)))
        } else {
            None
        }
    }
}

/// Binary export result with statistics
#[derive(Debug, Clone)]
pub struct BinaryExportResult {
    /// Path to the exported file
    pub file_path: String,
    /// Total file size in bytes
    pub file_size: usize,
    /// Original data size before compression
    pub original_size: usize,
    /// Compression ratio (compressed_size / original_size)
    pub compression_ratio: f64,
    /// Export duration
    pub export_duration: std::time::Duration,
    /// Number of sections exported
    pub sections_exported: usize,
    /// Number of allocations exported
    pub allocations_exported: usize,
    /// Memory usage during export
    pub peak_memory_usage: usize,
    /// Whether parallel processing was used
    pub used_parallel_processing: bool,
}

impl BinaryExportResult {
    /// Calculate space savings percentage
    pub fn space_savings_percentage(&self) -> f64 {
        if self.original_size > 0 {
            (1.0 - self.compression_ratio) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate export speed in MB/s
    pub fn export_speed_mbps(&self) -> f64 {
        let seconds = self.export_duration.as_secs_f64();
        if seconds > 0.0 {
            (self.original_size as f64) / (1024.0 * 1024.0) / seconds
        } else {
            0.0
        }
    }
}

/// Variable information for registry
#[derive(Debug, Clone)]
struct VariableInfo {
    name: String,
    type_name: Option<String>,
    total_size: usize,
    allocation_count: u32,
    first_seen: u64,
    last_seen: u64,
}

/// Section creation task types for parallel processing
#[derive(Debug, Clone, Copy)]
enum SectionCreationTask {
    MemoryStats,
    ActiveAllocations,
    TypeMemoryUsage,
    AllocationHistory,
    FfiAnalysis,
    LifecycleAnalysis,
    PerformanceData,
    VariableRegistry,
    SecurityAnalysis,
    MemoryPassports,
}

/// Main binary exporter struct
pub struct BinaryExporter {
    options: BinaryExportOptions,
    string_table: StringTable,
    type_table: TypeTable,
    progress_callback: Option<Box<dyn Fn(&ExportProgress) + Send + Sync>>,
    memory_monitor: Option<MemoryUsageMonitor>,
    buffer_pool: Option<ZeroCopyBufferPool>,
    string_optimizer: Option<StringOptimizer>,
    simd_processor: Option<SimdProcessor>,
}

impl BinaryExporter {
    /// Create a new binary exporter with default options
    pub fn new() -> Self {
        Self {
            options: BinaryExportOptions::default(),
            string_table: StringTable::new(),
            type_table: TypeTable::new(),
            progress_callback: None,
            memory_monitor: None,
            buffer_pool: None,
            string_optimizer: None,
            simd_processor: None,
        }
    }

    /// Create a new binary exporter with custom options
    pub fn with_options(options: BinaryExportOptions) -> Self {
        let memory_monitor = if options.performance.use_memory_mapping {
            let config = options
                .performance
                .memory_mapping_config
                .clone()
                .unwrap_or_default();
            Some(MemoryUsageMonitor::new(config.max_memory_usage))
        } else {
            None
        };

        let buffer_pool = if options.performance.enable_zero_copy {
            Some(ZeroCopyBufferPool::new(
                options.buffer_size,
                16, // Max 16 buffers in pool
            ))
        } else {
            None
        };

        let string_optimizer = if options.performance.enable_zero_copy {
            Some(StringOptimizer::new())
        } else {
            None
        };

        let simd_processor = if options.performance.enable_simd {
            Some(SimdProcessor::new())
        } else {
            None
        };

        Self {
            options,
            string_table: StringTable::new(),
            type_table: TypeTable::new(),
            progress_callback: None,
            memory_monitor,
            buffer_pool,
            string_optimizer,
            simd_processor,
        }
    }

    /// Set progress callback for monitoring export progress
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&ExportProgress) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Export memory tracker data to binary file with memory mapping optimization
    pub fn export_to_binary<P: AsRef<Path>>(
        &mut self,
        tracker: &MemoryTracker,
        path: P,
    ) -> TrackingResult<BinaryExportResult> {
        if self.options.performance.use_memory_mapping {
            self.export_to_binary_with_memory_mapping(tracker, path)
        } else {
            self.export_to_binary_standard(tracker, path)
        }
    }

    /// Export using memory mapping for large files
    fn export_to_binary_with_memory_mapping<P: AsRef<Path>>(
        &mut self,
        tracker: &MemoryTracker,
        path: P,
    ) -> TrackingResult<BinaryExportResult> {
        if self.options.performance.enable_zero_copy {
            self.export_with_zero_copy_and_memory_mapping(tracker, path)
        } else {
            self.export_with_memory_mapping_only(tracker, path)
        }
    }

    /// Export with both zero-copy and memory mapping optimizations
    fn export_with_zero_copy_and_memory_mapping<P: AsRef<Path>>(
        &mut self,
        tracker: &MemoryTracker,
        path: P,
    ) -> TrackingResult<BinaryExportResult> {
        let start_time = std::time::Instant::now();
        let path = path.as_ref();

        // Get configurations
        let mmap_config = self
            .options
            .performance
            .memory_mapping_config
            .clone()
            .unwrap_or_default();

        let buffer_pool = self.buffer_pool.as_ref().ok_or_else(|| {
            crate::core::types::TrackingError::ExportError("Zero-copy not initialized".to_string())
        })?;

        // Create memory monitor if not already created
        let monitor = self
            .memory_monitor
            .get_or_insert_with(|| MemoryUsageMonitor::new(mmap_config.max_memory_usage));

        // Create memory-mapped writer with zero-copy wrapper
        let mmap_writer =
            MemoryMappedWriter::new(path, mmap_config, monitor.clone()).map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Memory mapping failed: {}",
                    e
                ))
            })?;

        let mut zero_copy_writer = ZeroCopyWriter::new(mmap_writer, buffer_pool.clone())
            .with_flush_threshold(self.options.buffer_size);

        // Collect data from tracker
        let allocations = tracker.get_active_allocations()?;
        let stats = tracker.get_stats()?;
        let memory_by_type = tracker.get_memory_by_type()?;

        // Initialize progress tracking
        let total_sections = self.calculate_total_sections();
        let mut progress = ExportProgress {
            current_section: SectionType::MemoryStats,
            sections_completed: 0,
            total_sections,
            bytes_processed: 0,
            estimated_total_bytes: self.estimate_total_bytes(&allocations, &stats, &memory_by_type),
            start_time,
        };

        // Report initial progress
        if let Some(ref callback) = self.progress_callback {
            callback(&progress);
        }

        // Create sections with zero-copy optimization
        let sections = self.create_sections_with_zero_copy(
            &allocations,
            &stats,
            &memory_by_type,
            &mut progress,
        )?;

        // Write binary file using zero-copy writer
        self.write_binary_file_zero_copy(&mut zero_copy_writer, sections, &mut progress)?;

        // Flush and sync data
        zero_copy_writer.flush().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!("Failed to flush writer: {}", e))
        })?;

        let export_duration = start_time.elapsed();
        let pool_stats = zero_copy_writer.pool_stats();

        Ok(BinaryExportResult {
            file_path: {
                let path_ref: &std::path::Path = path.as_ref();
                path_ref.to_string_lossy().into_owned()
            },
            original_size: 0, // Will be calculated properly
            file_size: std::fs::metadata(path)
                .map(|m| m.len() as usize)
                .unwrap_or(0),
            sections_exported: total_sections,
            compression_ratio: 0.0, // Will be calculated properly
            export_duration,
            peak_memory_usage: pool_stats.peak_pool_size,
            allocations_exported: 0, // Will be set properly
            used_parallel_processing: self.options.enable_parallel_encoding,
        })
    }

    /// Export with memory mapping only (fallback)
    fn export_with_memory_mapping_only<P: AsRef<Path>>(
        &mut self,
        tracker: &MemoryTracker,
        path: P,
    ) -> TrackingResult<BinaryExportResult> {
        let start_time = std::time::Instant::now();
        let path = path.as_ref();

        // Get memory mapping configuration
        let mmap_config = self
            .options
            .performance
            .memory_mapping_config
            .clone()
            .unwrap_or_default();

        // Create memory monitor if not already created
        let monitor = self
            .memory_monitor
            .get_or_insert_with(|| MemoryUsageMonitor::new(mmap_config.max_memory_usage));

        // Create memory-mapped writer
        let mut writer =
            MemoryMappedWriter::new(path, mmap_config, monitor.clone()).map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Memory mapping failed: {}",
                    e
                ))
            })?;

        // Collect data from tracker
        let allocations = tracker.get_active_allocations()?;
        let stats = tracker.get_stats()?;
        let memory_by_type = tracker.get_memory_by_type()?;

        // Initialize progress tracking
        let total_sections = self.calculate_total_sections();
        let mut progress = ExportProgress {
            current_section: SectionType::MemoryStats,
            sections_completed: 0,
            total_sections,
            bytes_processed: 0,
            estimated_total_bytes: self.estimate_total_bytes(&allocations, &stats, &memory_by_type),
            start_time,
        };

        // Report initial progress
        if let Some(ref callback) = self.progress_callback {
            callback(&progress);
        }

        // Create sections with memory mapping optimization
        let sections = self.create_sections_with_memory_mapping(
            &allocations,
            &stats,
            &memory_by_type,
            &mut progress,
        )?;

        // Write binary file using memory-mapped writer
        self.write_binary_file_memory_mapped(&mut writer, sections, &mut progress)?;

        // Flush and sync data
        writer.sync_all().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!("Failed to sync file: {}", e))
        })?;

        let export_duration = start_time.elapsed();
        let memory_stats = writer.memory_stats();

        Ok(BinaryExportResult {
            file_path: {
                let path_ref: &std::path::Path = path.as_ref();
                path_ref.to_string_lossy().into_owned()
            },
            original_size: 0, // Will be calculated properly
            file_size: std::fs::metadata(path)
                .map(|m| m.len() as usize)
                .unwrap_or(0),
            sections_exported: total_sections,
            compression_ratio: 0.0, // Will be calculated properly
            export_duration,
            peak_memory_usage: memory_stats.peak_usage,
            allocations_exported: 0, // Will be set properly
            used_parallel_processing: self.options.enable_parallel_encoding,
        })
    }

    /// Standard export without memory mapping
    fn export_to_binary_standard<P: AsRef<Path>>(
        &mut self,
        tracker: &MemoryTracker,
        path: P,
    ) -> TrackingResult<BinaryExportResult> {
        let start_time = std::time::Instant::now();
        let path = path.as_ref();

        // Collect data from tracker
        let allocations = tracker.get_active_allocations()?;
        let stats = tracker.get_stats()?;
        let memory_by_type = tracker.get_memory_by_type()?;

        // Initialize progress tracking
        let total_sections = self.calculate_total_sections();
        let mut progress = ExportProgress {
            current_section: SectionType::MemoryStats,
            sections_completed: 0,
            total_sections,
            bytes_processed: 0,
            estimated_total_bytes: self.estimate_total_bytes(&allocations, &stats, &memory_by_type),
            start_time,
        };

        // Report initial progress
        if let Some(ref callback) = self.progress_callback {
            callback(&progress);
        }

        // Configure thread pool if parallel processing is enabled
        if self.options.enable_parallel_encoding {
            if let Some(thread_count) = self.options.thread_count {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(thread_count)
                    .build_global()
                    .unwrap_or_else(|_| {
                        // Thread pool already initialized, continue with existing one
                    });
            }
        }

        // Create sections in parallel or sequentially
        let sections = if self.options.enable_parallel_encoding && allocations.len() > 1000 {
            self.create_sections_parallel(&allocations, &stats, &memory_by_type, &mut progress)?
        } else {
            self.create_sections_sequential(&allocations, &stats, &memory_by_type, &mut progress)?
        };

        // Create binary file structure
        let mut binary_file = BinaryFile {
            header: BinaryHeader::new(),
            directory: SectionDirectory::new(),
            section_data: HashMap::new(),
        };

        // Add sections to binary file
        let mut current_offset = self.calculate_header_and_directory_size(&sections);
        for (section_type, data) in sections {
            // Apply compression if enabled and data is large enough
            let (compressed_data, compression_used) = self.compress_section_data(&data)?;

            let entry = SectionEntry::new(
                section_type,
                compression_used,
                current_offset,
                compressed_data.len() as u32,
                data.len() as u32,
            );

            binary_file.directory.add_section(entry);
            binary_file
                .section_data
                .insert(section_type, compressed_data);
            current_offset += data.len() as u64;
        }

        // Update header
        binary_file.header.compression_type = self.options.compression;
        binary_file.header.section_count = binary_file.directory.sections.len() as u32;
        binary_file.header.total_size = current_offset;
        binary_file.header.calculate_checksum();

        // Write to file
        let original_size = self.calculate_original_size(&binary_file);
        let file_size = self.write_binary_file(&binary_file, path)?;

        // Create result
        let result = BinaryExportResult {
            file_path: path.to_string_lossy().to_string(),
            file_size,
            original_size,
            compression_ratio: file_size as f64 / original_size as f64,
            export_duration: start_time.elapsed(),
            sections_exported: binary_file.directory.sections.len(),
            allocations_exported: allocations.len(),
            peak_memory_usage: 0, // TODO: Implement memory tracking
            used_parallel_processing: self.options.enable_parallel_encoding
                && allocations.len() > 1000,
        };

        Ok(result)
    }

    /// Calculate total number of sections to be exported
    fn calculate_total_sections(&self) -> usize {
        let mut count = 2; // Always include MemoryStats and ActiveAllocations

        if self.options.include_history {
            count += 1; // AllocationHistory
        }

        count += 1; // TypeMemoryUsage

        if self.options.include_ffi_analysis {
            count += 3; // FfiAnalysis, LifecycleAnalysis, and MemoryPassports
        }

        count += 2; // PerformanceData and VariableRegistry

        if self.options.enable_security_analysis {
            count += 1; // SecurityViolations
        }

        count
    }

    /// Estimate total bytes for progress tracking
    fn estimate_total_bytes(
        &self,
        allocations: &[AllocationInfo],
        _stats: &MemoryStats,
        memory_by_type: &[TypeMemoryUsage],
    ) -> usize {
        // Rough estimation based on data structures
        let allocation_bytes = allocations.len() * 200; // ~200 bytes per allocation
        let type_bytes = memory_by_type.len() * 100; // ~100 bytes per type
        let overhead = 10240; // 10KB overhead for headers, tables, etc.

        allocation_bytes + type_bytes + overhead
    }

    /// Calculate header and directory size
    fn calculate_header_and_directory_size(&self, sections: &HashMap<SectionType, Vec<u8>>) -> u64 {
        let header_size = 64; // BinaryHeader size
        let directory_size = sections.len() * 20; // SectionEntry size
        let string_table_size = self.string_table.to_bytes().len();
        let type_table_size = self.type_table.to_bytes().len();

        (header_size + directory_size + string_table_size + type_table_size) as u64
    }

    /// Calculate original uncompressed size
    fn calculate_original_size(&self, binary_file: &BinaryFile) -> usize {
        let mut size = 64; // Header size
        size += binary_file.directory.to_bytes().len();
        size += self.string_table.to_bytes().len();
        size += self.type_table.to_bytes().len();

        for entry in binary_file.directory.sections.values() {
            size += entry.uncompressed_size as usize;
        }

        size
    }

    /// Compress section data if beneficial
    fn compress_section_data(&self, data: &[u8]) -> TrackingResult<(Vec<u8>, CompressionType)> {
        // Skip compression for small data
        if data.len() < self.options.compression_threshold {
            return Ok((data.to_vec(), CompressionType::None));
        }

        let compression = if self.options.auto_compression {
            // Choose compression based on data characteristics
            let estimated_ratio =
                CompressionEngine::estimate_compression_ratio(data, self.options.compression);
            if estimated_ratio < 0.8 {
                self.options.compression
            } else {
                CompressionType::None // Not worth compressing
            }
        } else {
            self.options.compression
        };

        let compressed = CompressionEngine::compress(data, compression).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!("Compression failed: {}", e))
        })?;

        Ok((compressed, compression))
    }

    /// Write binary file to disk
    fn write_binary_file(&self, binary_file: &BinaryFile, path: &Path) -> TrackingResult<usize> {
        let file = File::create(path).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!("Failed to create file: {}", e))
        })?;

        let mut writer = BufWriter::with_capacity(self.options.buffer_size, file);
        let mut total_written = 0;

        // Write header
        let header_bytes = binary_file.header.to_bytes();
        writer.write_all(&header_bytes).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!("Failed to write header: {}", e))
        })?;
        total_written += header_bytes.len();

        // Write section directory
        let directory_bytes = binary_file.directory.to_bytes();
        writer.write_all(&directory_bytes).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to write directory: {}",
                e
            ))
        })?;
        total_written += directory_bytes.len();

        // Write string table
        let string_table_bytes = self.string_table.to_bytes();
        writer.write_all(&string_table_bytes).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to write string table: {}",
                e
            ))
        })?;
        total_written += string_table_bytes.len();

        // Write type table
        let type_table_bytes = self.type_table.to_bytes();
        writer.write_all(&type_table_bytes).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to write type table: {}",
                e
            ))
        })?;
        total_written += type_table_bytes.len();

        // Write section data in order
        let mut sections: Vec<_> = binary_file.directory.sections.values().collect();
        sections.sort_by_key(|entry| entry.section_type as u8);

        for entry in sections {
            if let Some(data) = binary_file.section_data.get(&entry.section_type) {
                writer.write_all(data).map_err(|e| {
                    crate::core::types::TrackingError::ExportError(format!(
                        "Failed to write section data: {}",
                        e
                    ))
                })?;
                total_written += data.len();
            }
        }

        writer.flush().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!("Failed to flush writer: {}", e))
        })?;

        Ok(total_written)
    }

    /// Create sections sequentially
    fn create_sections_sequential(
        &mut self,
        allocations: &[AllocationInfo],
        stats: &MemoryStats,
        memory_by_type: &[TypeMemoryUsage],
        progress: &mut ExportProgress,
    ) -> TrackingResult<HashMap<SectionType, Vec<u8>>> {
        let mut sections = HashMap::new();

        // Memory Stats section
        progress.current_section = SectionType::MemoryStats;
        if let Some(ref callback) = self.progress_callback {
            callback(progress);
        }
        let stats_data = self.encode_memory_stats(stats)?;
        sections.insert(SectionType::MemoryStats, stats_data);
        progress.sections_completed += 1;

        // Active Allocations section
        progress.current_section = SectionType::ActiveAllocations;
        if let Some(ref callback) = self.progress_callback {
            callback(progress);
        }
        let allocations_data = self.encode_allocations(allocations)?;
        sections.insert(SectionType::ActiveAllocations, allocations_data);
        progress.sections_completed += 1;

        // Type Memory Usage section
        progress.current_section = SectionType::TypeMemoryUsage;
        if let Some(ref callback) = self.progress_callback {
            callback(progress);
        }
        let types_data = self.encode_type_memory_usage(memory_by_type)?;
        sections.insert(SectionType::TypeMemoryUsage, types_data);
        progress.sections_completed += 1;

        // Optional sections
        if self.options.include_history {
            progress.current_section = SectionType::AllocationHistory;
            if let Some(ref callback) = self.progress_callback {
                callback(progress);
            }
            let history_data = self.encode_allocation_history(allocations)?;
            sections.insert(SectionType::AllocationHistory, history_data);
            progress.sections_completed += 1;
        }

        if self.options.include_ffi_analysis {
            progress.current_section = SectionType::FfiAnalysis;
            if let Some(ref callback) = self.progress_callback {
                callback(progress);
            }
            let ffi_data = self.encode_ffi_analysis(allocations)?;
            sections.insert(SectionType::FfiAnalysis, ffi_data);
            progress.sections_completed += 1;

            // Also include lifecycle analysis when FFI analysis is enabled
            progress.current_section = SectionType::LifecycleAnalysis;
            if let Some(ref callback) = self.progress_callback {
                callback(progress);
            }
            let lifecycle_data = self.encode_allocation_history(allocations)?; // Reuse history for now
            sections.insert(SectionType::LifecycleAnalysis, lifecycle_data);
            progress.sections_completed += 1;
        }

        // Performance data section
        progress.current_section = SectionType::PerformanceData;
        if let Some(ref callback) = self.progress_callback {
            callback(progress);
        }
        let performance_data = self.encode_performance_data(allocations, stats)?;
        sections.insert(SectionType::PerformanceData, performance_data);
        progress.sections_completed += 1;

        // Variable registry section
        progress.current_section = SectionType::VariableRegistry;
        if let Some(ref callback) = self.progress_callback {
            callback(progress);
        }
        let registry_data = self.encode_variable_registry(allocations)?;
        sections.insert(SectionType::VariableRegistry, registry_data);
        progress.sections_completed += 1;

        // Security violations section (if enabled)
        if self.options.enable_security_analysis {
            let security_data = self.encode_security_violations(allocations)?;
            sections.insert(SectionType::SecurityViolations, security_data);
            progress.sections_completed += 1;
        }

        // Memory passports section (if FFI analysis is enabled)
        if self.options.include_ffi_analysis {
            let passport_data = self.encode_memory_passports(allocations)?;
            sections.insert(SectionType::MemoryPassports, passport_data);
            progress.sections_completed += 1;
        }

        Ok(sections)
    }

    /// Create sections in parallel using Rayon
    fn create_sections_parallel(
        &mut self,
        allocations: &[AllocationInfo],
        stats: &MemoryStats,
        memory_by_type: &[TypeMemoryUsage],
        progress: &mut ExportProgress,
    ) -> TrackingResult<HashMap<SectionType, Vec<u8>>> {
        // Configure thread pool if specified
        if let Some(thread_count) = self.options.thread_count {
            rayon::ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build_global()
                .unwrap_or_else(|_| {
                    // Thread pool already initialized, continue with existing one
                });
        }

        // Create shared progress tracking
        let progress_mutex = Arc::new(Mutex::new(progress));
        let callback = self
            .progress_callback
            .as_ref()
            .map(|cb| Arc::clone(&Arc::new(cb)));

        // Define section creation tasks
        let mut section_tasks = Vec::new();

        // Always include core sections
        section_tasks.push((SectionType::MemoryStats, SectionCreationTask::MemoryStats));
        section_tasks.push((
            SectionType::ActiveAllocations,
            SectionCreationTask::ActiveAllocations,
        ));
        section_tasks.push((
            SectionType::TypeMemoryUsage,
            SectionCreationTask::TypeMemoryUsage,
        ));
        section_tasks.push((
            SectionType::PerformanceData,
            SectionCreationTask::PerformanceData,
        ));
        section_tasks.push((
            SectionType::VariableRegistry,
            SectionCreationTask::VariableRegistry,
        ));

        // Optional sections
        if self.options.include_history {
            section_tasks.push((
                SectionType::AllocationHistory,
                SectionCreationTask::AllocationHistory,
            ));
        }

        if self.options.include_ffi_analysis {
            section_tasks.push((SectionType::FfiAnalysis, SectionCreationTask::FfiAnalysis));
            section_tasks.push((
                SectionType::LifecycleAnalysis,
                SectionCreationTask::LifecycleAnalysis,
            ));
        }

        // Add security analysis section if enabled
        if self.options.enable_security_analysis {
            section_tasks.push((
                SectionType::SecurityViolations,
                SectionCreationTask::SecurityAnalysis,
            ));
        }

        // Add memory passports section if FFI analysis is enabled
        if self.options.include_ffi_analysis {
            section_tasks.push((
                SectionType::MemoryPassports,
                SectionCreationTask::MemoryPassports,
            ));
        }

        // Create shared string and type tables for parallel access
        let string_table = Arc::new(Mutex::new(StringTable::new()));
        let type_table = Arc::new(Mutex::new(TypeTable::new()));

        // Create sections in parallel
        let results: Result<Vec<_>, _> = section_tasks
            .into_par_iter()
            .map(|(section_type, task)| {
                // Update progress
                if let Some(ref callback) = callback {
                    if let Ok(mut progress) = progress_mutex.lock() {
                        progress.current_section = section_type;
                        callback(&*progress);
                    }
                }

                // Create section data using static methods
                let section_data = match task {
                    SectionCreationTask::MemoryStats => Self::encode_memory_stats_static(stats),
                    SectionCreationTask::ActiveAllocations => {
                        // Use the standard encode_allocations method instead of static version
                        // The static version has encoding format mismatches with the parser
                        println!("DEBUG: Encoding {} allocations for ActiveAllocations section", allocations.len());
                        let mut exporter = BinaryExporter::new();
                        let result = exporter.encode_allocations(allocations);
                        if let Ok(ref data) = result {
                            println!("DEBUG: Encoded data size: {} bytes", data.len());
                            if data.len() >= 8 {
                                println!("DEBUG: First 8 bytes: {:?}", &data[0..8]);
                            }
                        }
                        result
                    },
                    SectionCreationTask::TypeMemoryUsage => Self::encode_type_memory_usage_static(
                        memory_by_type,
                        Arc::clone(&string_table),
                    ),
                    SectionCreationTask::AllocationHistory => {
                        Self::encode_allocation_history_static(allocations, Arc::clone(&type_table))
                    }
                    SectionCreationTask::FfiAnalysis => {
                        Self::encode_ffi_analysis_static(allocations, Arc::clone(&type_table))
                    }
                    SectionCreationTask::LifecycleAnalysis => {
                        Self::encode_allocation_history_static(allocations, Arc::clone(&type_table))
                        // Reuse for now
                    }
                    SectionCreationTask::PerformanceData => {
                        Self::encode_performance_data_static(allocations, stats)
                    }
                    SectionCreationTask::VariableRegistry => Self::encode_variable_registry_static(
                        allocations,
                        Arc::clone(&string_table),
                        Arc::clone(&type_table),
                    ),
                    SectionCreationTask::SecurityAnalysis => {
                        Self::encode_security_violations_static(allocations)
                    }
                    SectionCreationTask::MemoryPassports => {
                        Self::encode_memory_passports_static(allocations)
                    }
                }?;

                // Update progress
                if let Ok(mut progress) = progress_mutex.lock() {
                    progress.sections_completed += 1;
                    if let Some(ref callback) = callback {
                        callback(&*progress);
                    }
                }

                Ok::<(SectionType, Vec<u8>), crate::core::types::TrackingError>((
                    section_type,
                    section_data,
                ))
            })
            .collect();

        // Merge the shared tables back into self
        if let Ok(shared_string_table) = string_table.lock() {
            self.string_table = shared_string_table.clone();
        }
        if let Ok(shared_type_table) = type_table.lock() {
            self.type_table = shared_type_table.clone();
        }

        let sections: HashMap<SectionType, Vec<u8>> = results?.into_iter().collect();
        Ok(sections)
    }

    /// Encode memory stats section
    fn encode_memory_stats(&mut self, stats: &MemoryStats) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        encoder.encode_usize(stats.total_allocated);
        encoder.encode_usize(stats.total_deallocated);
        encoder.encode_usize(stats.active_memory);
        encoder.encode_usize(stats.peak_memory);
        encoder.encode_usize(stats.active_allocations);
        encoder.encode_usize(stats.total_allocations);
        encoder.encode_usize(stats.peak_allocations);
        encoder.encode_usize(stats.leaked_allocations);
        encoder.encode_usize(stats.leaked_memory);

        Ok(encoder.into_bytes())
    }

    /// Encode allocations section with full AllocationInfo support
    fn encode_allocations(&mut self, allocations: &[AllocationInfo]) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        // Write allocation count
        encoder.encode_u32(allocations.len() as u32);

        // Write each allocation with all fields
        for allocation in allocations {
            // Basic allocation info
            encoder.encode_usize(allocation.ptr);
            encoder.encode_usize(allocation.size);
            encoder.encode_optional_string(&allocation.var_name);
            encoder.encode_optional_type_name(&allocation.type_name);
            encoder.encode_optional_string(&allocation.scope_name);
            encoder.encode_u64(allocation.timestamp_alloc);

            // Optional timestamp_dealloc
            match allocation.timestamp_dealloc {
                Some(ts) => {
                    encoder.encode_u8(1);
                    encoder.encode_u64(ts);
                }
                None => {
                    encoder.encode_u8(0);
                }
            }

            // Thread ID (as string)
            encoder.encode_string(&allocation.thread_id);

            // Borrow count
            encoder.encode_usize(allocation.borrow_count);

            // Optional stack trace
            encoder.encode_optional_string_vec(&allocation.stack_trace);

            // Leak status
            encoder.encode_u8(if allocation.is_leaked { 1 } else { 0 });

            // Optional lifetime_ms
            match allocation.lifetime_ms {
                Some(lifetime) => {
                    encoder.encode_u8(1);
                    encoder.encode_u64(lifetime);
                }
                None => {
                    encoder.encode_u8(0);
                }
            }

            // Extended fields (encode as present/absent flags for now)
            // In a full implementation, these would be properly encoded
            encoder.encode_u8(if allocation.smart_pointer_info.is_some() {
                1
            } else {
                0
            });
            encoder.encode_u8(if allocation.memory_layout.is_some() {
                1
            } else {
                0
            });
            encoder.encode_u8(if allocation.generic_info.is_some() {
                1
            } else {
                0
            });
            encoder.encode_u8(if allocation.dynamic_type_info.is_some() {
                1
            } else {
                0
            });
            encoder.encode_u8(if allocation.runtime_state.is_some() {
                1
            } else {
                0
            });
            encoder.encode_u8(if allocation.stack_allocation.is_some() {
                1
            } else {
                0
            });
            encoder.encode_u8(if allocation.temporary_object.is_some() {
                1
            } else {
                0
            });
            encoder.encode_u8(if allocation.fragmentation_analysis.is_some() {
                1
            } else {
                0
            });
            encoder.encode_u8(if allocation.generic_instantiation.is_some() {
                1
            } else {
                0
            });
            encoder.encode_u8(if allocation.type_relationships.is_some() {
                1
            } else {
                0
            });
            encoder.encode_u8(if allocation.type_usage.is_some() {
                1
            } else {
                0
            });
            encoder.encode_u8(if allocation.function_call_tracking.is_some() {
                1
            } else {
                0
            });
            encoder.encode_u8(if allocation.lifecycle_tracking.is_some() {
                1
            } else {
                0
            });
            encoder.encode_u8(if allocation.access_tracking.is_some() {
                1
            } else {
                0
            });
        }

        Ok(encoder.into_bytes())
    }

    /// Encode type memory usage section
    fn encode_type_memory_usage(
        &mut self,
        memory_by_type: &[TypeMemoryUsage],
    ) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        // Write type count
        encoder.encode_u32(memory_by_type.len() as u32);

        // Write each type usage
        for usage in memory_by_type {
            encoder.encode_string(&usage.type_name);
            encoder.encode_usize(usage.total_size);
            encoder.encode_usize(usage.allocation_count);
        }

        Ok(encoder.into_bytes())
    }

    /// Encode allocation history section
    fn encode_allocation_history(
        &mut self,
        allocations: &[AllocationInfo],
    ) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        // Filter allocations that have been deallocated (have history)
        let historical_allocations: Vec<_> = allocations
            .iter()
            .filter(|alloc| alloc.timestamp_dealloc.is_some())
            .collect();

        encoder.encode_u32(historical_allocations.len() as u32);

        for allocation in historical_allocations {
            encoder.encode_usize(allocation.ptr);
            encoder.encode_usize(allocation.size);
            encoder.encode_optional_type_name(&allocation.type_name);
            encoder.encode_u64(allocation.timestamp_alloc);

            if let Some(dealloc_time) = allocation.timestamp_dealloc {
                encoder.encode_u64(dealloc_time);

                // Calculate and encode lifetime
                let lifetime = dealloc_time.saturating_sub(allocation.timestamp_alloc);
                encoder.encode_u64(lifetime);
            }

            // Encode whether it was leaked
            encoder.encode_u8(if allocation.is_leaked { 1 } else { 0 });
        }

        Ok(encoder.into_bytes())
    }

    /// Encode FFI analysis section
    fn encode_ffi_analysis(&mut self, allocations: &[AllocationInfo]) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        // Filter allocations that might be FFI-related
        let ffi_allocations: Vec<_> = allocations
            .iter()
            .filter(|alloc| self.is_ffi_related(alloc))
            .collect();

        encoder.encode_u32(ffi_allocations.len() as u32);

        for allocation in ffi_allocations {
            encoder.encode_usize(allocation.ptr);
            encoder.encode_usize(allocation.size);
            encoder.encode_optional_type_name(&allocation.type_name);
            encoder.encode_optional_string(&allocation.var_name);

            // Encode FFI risk level
            let risk_level = self.assess_ffi_risk(allocation);
            encoder.encode_u8(risk_level);

            // Encode FFI type
            let ffi_type = self.determine_ffi_type(allocation);
            encoder.encode_u8(ffi_type);
        }

        Ok(encoder.into_bytes())
    }

    /// Encode performance data section
    fn encode_performance_data(
        &mut self,
        allocations: &[AllocationInfo],
        stats: &MemoryStats,
    ) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        // Encode basic performance metrics
        encoder.encode_usize(stats.total_allocations);
        encoder.encode_usize(stats.total_deallocations);
        encoder.encode_usize(stats.peak_allocations);
        encoder.encode_usize(stats.peak_memory);

        // Calculate and encode allocation size distribution
        let size_distribution = self.calculate_size_distribution(allocations);
        encoder.encode_u32(size_distribution.len() as u32);

        for (size_range, count) in size_distribution {
            encoder.encode_u32(size_range); // Size range identifier
            encoder.encode_u32(count); // Count of allocations in this range
        }

        // Encode allocation frequency over time (simplified)
        let frequency_data = self.calculate_allocation_frequency(allocations);
        encoder.encode_u32(frequency_data.len() as u32);

        for (time_bucket, count) in frequency_data {
            encoder.encode_u64(time_bucket); // Time bucket (timestamp)
            encoder.encode_u32(count); // Allocations in this bucket
        }

        Ok(encoder.into_bytes())
    }

    /// Encode variable registry section
    fn encode_variable_registry(
        &mut self,
        allocations: &[AllocationInfo],
    ) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        // Create a registry of unique variable names and their usage
        let mut variable_registry: HashMap<String, VariableInfo> = HashMap::new();

        for allocation in allocations {
            if let Some(var_name) = &allocation.var_name {
                let entry = variable_registry
                    .entry(var_name.clone())
                    .or_insert(VariableInfo {
                        name: var_name.clone(),
                        type_name: allocation.type_name.clone(),
                        total_size: 0,
                        allocation_count: 0,
                        first_seen: allocation.timestamp_alloc,
                        last_seen: allocation.timestamp_alloc,
                    });

                entry.total_size += allocation.size;
                entry.allocation_count += 1;
                entry.first_seen = entry.first_seen.min(allocation.timestamp_alloc);
                entry.last_seen = entry.last_seen.max(allocation.timestamp_alloc);
            }
        }

        encoder.encode_u32(variable_registry.len() as u32);

        for variable_info in variable_registry.values() {
            encoder.encode_string(&variable_info.name);
            encoder.encode_optional_type_name(&variable_info.type_name);
            encoder.encode_usize(variable_info.total_size);
            encoder.encode_u32(variable_info.allocation_count);
            encoder.encode_u64(variable_info.first_seen);
            encoder.encode_u64(variable_info.last_seen);
        }

        Ok(encoder.into_bytes())
    }

    /// Check if an allocation is FFI-related
    fn is_ffi_related(&self, allocation: &AllocationInfo) -> bool {
        if let Some(type_name) = &allocation.type_name {
            // Check for common FFI patterns
            type_name.contains("*mut")
                || type_name.contains("*const")
                || type_name.contains("extern")
                || type_name.contains("libc::")
                || type_name.contains("CString")
                || type_name.contains("CStr")
        } else {
            false
        }
    }

    /// Assess FFI risk level (0=low, 1=medium, 2=high)
    fn assess_ffi_risk(&self, allocation: &AllocationInfo) -> u8 {
        if let Some(type_name) = &allocation.type_name {
            if type_name.contains("*mut") {
                2 // High risk - mutable raw pointer
            } else if type_name.contains("*const") {
                1 // Medium risk - const raw pointer
            } else {
                0 // Low risk - other FFI types
            }
        } else {
            0
        }
    }

    /// Determine FFI type (0=raw_pointer, 1=c_string, 2=external_lib, 3=other)
    fn determine_ffi_type(&self, allocation: &AllocationInfo) -> u8 {
        if let Some(type_name) = &allocation.type_name {
            if type_name.contains("*mut") || type_name.contains("*const") {
                0 // Raw pointer
            } else if type_name.contains("CString") || type_name.contains("CStr") {
                1 // C string
            } else if type_name.contains("libc::") {
                2 // External library
            } else {
                3 // Other FFI type
            }
        } else {
            3
        }
    }

    /// Calculate allocation size distribution
    fn calculate_size_distribution(&self, allocations: &[AllocationInfo]) -> Vec<(u32, u32)> {
        let mut distribution = HashMap::new();

        for allocation in allocations {
            let size_range = match allocation.size {
                0..=16 => 0,      // Tiny
                17..=64 => 1,     // Small
                65..=256 => 2,    // Medium
                257..=1024 => 3,  // Large
                1025..=4096 => 4, // Very Large
                _ => 5,           // Huge
            };

            *distribution.entry(size_range).or_insert(0) += 1;
        }

        distribution.into_iter().collect()
    }

    /// Calculate allocation frequency over time
    fn calculate_allocation_frequency(&self, allocations: &[AllocationInfo]) -> Vec<(u64, u32)> {
        let mut frequency = HashMap::new();

        for allocation in allocations {
            // Group by time buckets (e.g., every 1 second)
            let time_bucket = (allocation.timestamp_alloc / 1_000_000_000) * 1_000_000_000;
            *frequency.entry(time_bucket).or_insert(0) += 1;
        }

        let mut result: Vec<_> = frequency.into_iter().collect();
        result.sort_by_key(|(time, _)| *time);
        result
    }

    // Static methods for parallel processing

    /// Static method to encode memory stats (thread-safe)
    fn encode_memory_stats_static(stats: &MemoryStats) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        encoder.encode_usize(stats.total_allocated);
        encoder.encode_usize(stats.total_deallocated);
        encoder.encode_usize(stats.active_memory);
        encoder.encode_usize(stats.peak_memory);
        encoder.encode_usize(stats.active_allocations);
        encoder.encode_usize(stats.total_allocations);
        encoder.encode_usize(stats.peak_allocations);
        encoder.encode_usize(stats.leaked_allocations);
        encoder.encode_usize(stats.leaked_memory);

        Ok(encoder.into_bytes())
    }

    /// Static method to encode allocations (thread-safe)
    fn encode_allocations_static(
        allocations: &[AllocationInfo],
        string_table: Arc<Mutex<StringTable>>,
        type_table: Arc<Mutex<TypeTable>>,
    ) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        // Write allocation count
        encoder.encode_u32(allocations.len() as u32);

        // Process allocations sequentially to avoid data structure corruption
        // Parallel processing was causing the allocation count to be overwritten
        // by chunk data, leading to incorrect binary format
        for allocation in allocations {
            // Basic allocation info
            encoder.encode_usize(allocation.ptr);
            encoder.encode_usize(allocation.size);

            // Use shared string table for variable names
            if let Some(var_name) = &allocation.var_name {
                encoder.encode_u8(1);
                if let Ok(mut table) = string_table.lock() {
                    let id = table.intern(var_name);
                    encoder.encode_u32(id);
                }
            } else {
                encoder.encode_u8(0);
            }

            // Use shared type table for type names
            if let Some(type_name) = &allocation.type_name {
                encoder.encode_u8(1);
                if let Ok(mut table) = type_table.lock() {
                    let id = table.get_or_intern_type_id(type_name);
                    encoder.encode_u16(id);
                }
            } else {
                encoder.encode_u8(0);
            }

            // Scope name
            if let Some(scope_name) = &allocation.scope_name {
                encoder.encode_u8(1);
                if let Ok(mut table) = string_table.lock() {
                    let id = table.intern(scope_name);
                    encoder.encode_u32(id);
                }
            } else {
                encoder.encode_u8(0);
            }

            encoder.encode_u64(allocation.timestamp_alloc);

            // Optional timestamp_dealloc
            match allocation.timestamp_dealloc {
                Some(ts) => {
                    encoder.encode_u8(1);
                    encoder.encode_u64(ts);
                }
                None => {
                    encoder.encode_u8(0);
                }
            }

            // Thread ID
            if let Ok(mut table) = string_table.lock() {
                let id = table.intern(&allocation.thread_id);
                encoder.encode_u32(id);
            }

            encoder.encode_usize(allocation.borrow_count);

            // Stack trace
            if let Some(stack_trace) = &allocation.stack_trace {
                encoder.encode_u8(1);
                encoder.encode_u32(stack_trace.len() as u32);
                for frame in stack_trace {
                    if let Ok(mut table) = string_table.lock() {
                        let id = table.intern(frame);
                        encoder.encode_u32(id);
                    }
                }
            } else {
                encoder.encode_u8(0);
            }

            encoder.encode_u8(if allocation.is_leaked { 1 } else { 0 });

            // Optional lifetime_ms
            match allocation.lifetime_ms {
                Some(lifetime) => {
                    encoder.encode_u8(1);
                    encoder.encode_u64(lifetime);
                }
                None => {
                    encoder.encode_u8(0);
                }
            }

            // Extended fields (simplified flags)
            encoder.encode_u8(if allocation.smart_pointer_info.is_some() { 1 } else { 0 });
            encoder.encode_u8(if allocation.memory_layout.is_some() { 1 } else { 0 });
            encoder.encode_u8(if allocation.generic_info.is_some() { 1 } else { 0 });
            encoder.encode_u8(if allocation.dynamic_type_info.is_some() { 1 } else { 0 });
            encoder.encode_u8(if allocation.runtime_state.is_some() { 1 } else { 0 });
            encoder.encode_u8(if allocation.stack_allocation.is_some() { 1 } else { 0 });
            encoder.encode_u8(if allocation.temporary_object.is_some() { 1 } else { 0 });
            encoder.encode_u8(if allocation.fragmentation_analysis.is_some() { 1 } else { 0 });
            encoder.encode_u8(if allocation.generic_instantiation.is_some() { 1 } else { 0 });
            encoder.encode_u8(if allocation.type_relationships.is_some() { 1 } else { 0 });
            encoder.encode_u8(if allocation.type_usage.is_some() { 1 } else { 0 });
            encoder.encode_u8(if allocation.function_call_tracking.is_some() { 1 } else { 0 });
            encoder.encode_u8(if allocation.lifecycle_tracking.is_some() { 1 } else { 0 });
        }

        Ok(encoder.into_bytes())
    }

    /// Static method to encode type memory usage (thread-safe)
    fn encode_type_memory_usage_static(
        memory_by_type: &[TypeMemoryUsage],
        string_table: Arc<Mutex<StringTable>>,
    ) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        encoder.encode_u32(memory_by_type.len() as u32);

        for usage in memory_by_type {
            if let Ok(mut table) = string_table.lock() {
                let id = table.intern(&usage.type_name);
                encoder.encode_u32(id);
            }
            encoder.encode_usize(usage.total_size);
            encoder.encode_usize(usage.allocation_count);
        }

        Ok(encoder.into_bytes())
    }

    /// Static method to encode allocation history (thread-safe)
    fn encode_allocation_history_static(
        allocations: &[AllocationInfo],
        type_table: Arc<Mutex<TypeTable>>,
    ) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        let historical_allocations: Vec<_> = allocations
            .iter()
            .filter(|alloc| alloc.timestamp_dealloc.is_some())
            .collect();

        encoder.encode_u32(historical_allocations.len() as u32);

        for allocation in historical_allocations {
            encoder.encode_usize(allocation.ptr);
            encoder.encode_usize(allocation.size);

            if let Some(type_name) = &allocation.type_name {
                encoder.encode_u8(1);
                if let Ok(mut table) = type_table.lock() {
                    let id = table.get_or_intern_type_id(type_name);
                    encoder.encode_u16(id);
                }
            } else {
                encoder.encode_u8(0);
            }

            encoder.encode_u64(allocation.timestamp_alloc);

            if let Some(dealloc_time) = allocation.timestamp_dealloc {
                encoder.encode_u64(dealloc_time);
                let lifetime = dealloc_time.saturating_sub(allocation.timestamp_alloc);
                encoder.encode_u64(lifetime);
            }

            encoder.encode_u8(if allocation.is_leaked { 1 } else { 0 });
        }

        Ok(encoder.into_bytes())
    }

    /// Static method to encode FFI analysis (simplified)
    fn encode_ffi_analysis_static(
        allocations: &[AllocationInfo],
        type_table: Arc<Mutex<TypeTable>>,
    ) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        // Basic FFI detection from allocations
        let ffi_allocations: Vec<_> = allocations
            .iter()
            .filter(|alloc| Self::is_ffi_related_static(alloc))
            .collect();

        encoder.encode_u32(ffi_allocations.len() as u32);

        for allocation in ffi_allocations {
            encoder.encode_usize(allocation.ptr);
            encoder.encode_usize(allocation.size);

            if let Some(type_name) = &allocation.type_name {
                encoder.encode_u8(1);
                if let Ok(mut table) = type_table.lock() {
                    let id = table.get_or_intern_type_id(type_name);
                    encoder.encode_u16(id);
                }
            } else {
                encoder.encode_u8(0);
            }

            let risk_level = Self::assess_ffi_risk_static(allocation);
            encoder.encode_u8(risk_level);

            let ffi_type = Self::determine_ffi_type_static(allocation);
            encoder.encode_u8(ffi_type);
        }

        Ok(encoder.into_bytes())
    }

    // Static helper methods

    fn is_ffi_related_static(allocation: &AllocationInfo) -> bool {
        if let Some(type_name) = &allocation.type_name {
            type_name.contains("*mut")
                || type_name.contains("*const")
                || type_name.contains("extern")
                || type_name.contains("libc::")
                || type_name.contains("CString")
                || type_name.contains("CStr")
        } else {
            false
        }
    }

    fn assess_ffi_risk_static(allocation: &AllocationInfo) -> u8 {
        if let Some(type_name) = &allocation.type_name {
            if type_name.contains("*mut") {
                2
            } else if type_name.contains("*const") {
                1
            } else {
                0
            }
        } else {
            0
        }
    }

    fn determine_ffi_type_static(allocation: &AllocationInfo) -> u8 {
        if let Some(type_name) = &allocation.type_name {
            if type_name.contains("*mut") || type_name.contains("*const") {
                0
            } else if type_name.contains("CString") || type_name.contains("CStr") {
                1
            } else if type_name.contains("libc::") {
                2
            } else {
                3
            }
        } else {
            3
        }
    }

    fn calculate_size_distribution_static(allocations: &[AllocationInfo]) -> Vec<(u32, u32)> {
        let mut distribution = HashMap::new();

        for allocation in allocations {
            let size_range = match allocation.size {
                0..=16 => 0,
                17..=64 => 1,
                65..=256 => 2,
                257..=1024 => 3,
                1025..=4096 => 4,
                _ => 5,
            };

            *distribution.entry(size_range).or_insert(0) += 1;
        }

        distribution.into_iter().collect()
    }

    fn calculate_allocation_frequency_static(allocations: &[AllocationInfo]) -> Vec<(u64, u32)> {
        let mut frequency = HashMap::new();

        for allocation in allocations {
            let time_bucket = (allocation.timestamp_alloc / 1_000_000_000) * 1_000_000_000;
            *frequency.entry(time_bucket).or_insert(0) += 1;
        }

        let mut result: Vec<_> = frequency.into_iter().collect();
        result.sort_by_key(|(time, _)| *time);
        result
    }

    /// Static method to encode performance data (thread-safe)
    fn encode_performance_data_static(
        allocations: &[AllocationInfo],
        stats: &MemoryStats,
    ) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        encoder.encode_usize(stats.total_allocations);
        encoder.encode_usize(stats.total_deallocations);
        encoder.encode_usize(stats.peak_allocations);
        encoder.encode_usize(stats.peak_memory);

        let size_distribution = Self::calculate_size_distribution_static(allocations);
        encoder.encode_u32(size_distribution.len() as u32);

        for (size_range, count) in size_distribution {
            encoder.encode_u32(size_range);
            encoder.encode_u32(count);
        }

        let frequency_data = Self::calculate_allocation_frequency_static(allocations);
        encoder.encode_u32(frequency_data.len() as u32);

        for (time_bucket, count) in frequency_data {
            encoder.encode_u64(time_bucket);
            encoder.encode_u32(count);
        }

        Ok(encoder.into_bytes())
    }

    /// Static method to encode variable registry (thread-safe)
    fn encode_variable_registry_static(
        allocations: &[AllocationInfo],
        string_table: Arc<Mutex<StringTable>>,
        type_table: Arc<Mutex<TypeTable>>,
    ) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        let mut variable_registry: HashMap<String, VariableInfo> = HashMap::new();

        for allocation in allocations {
            if let Some(var_name) = &allocation.var_name {
                let entry = variable_registry
                    .entry(var_name.clone())
                    .or_insert(VariableInfo {
                        name: var_name.clone(),
                        type_name: allocation.type_name.clone(),
                        total_size: 0,
                        allocation_count: 0,
                        first_seen: allocation.timestamp_alloc,
                        last_seen: allocation.timestamp_alloc,
                    });

                entry.total_size += allocation.size;
                entry.allocation_count += 1;
                entry.first_seen = entry.first_seen.min(allocation.timestamp_alloc);
                entry.last_seen = entry.last_seen.max(allocation.timestamp_alloc);
            }
        }

        encoder.encode_u32(variable_registry.len() as u32);

        for variable_info in variable_registry.values() {
            if let Ok(mut table) = string_table.lock() {
                let id = table.intern(&variable_info.name);
                encoder.encode_u32(id);
            }

            if let Some(type_name) = &variable_info.type_name {
                encoder.encode_u8(1);
                if let Ok(mut table) = type_table.lock() {
                    let id = table.get_or_intern_type_id(type_name);
                    encoder.encode_u16(id);
                }
            } else {
                encoder.encode_u8(0);
            }

            encoder.encode_usize(variable_info.total_size);
            encoder.encode_u32(variable_info.allocation_count);
            encoder.encode_u64(variable_info.first_seen);
            encoder.encode_u64(variable_info.last_seen);
        }

        Ok(encoder.into_bytes())
    }

    /// Encode security violations section with comprehensive analysis
    fn encode_security_violations(
        &mut self,
        _allocations: &[AllocationInfo],
    ) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        // Get security violations from the global FFI tracker
        let ffi_tracker = get_global_unsafe_ffi_tracker();
        let violations = ffi_tracker.get_safety_violations()?;

        encoder.encode_u32(violations.len() as u32);

        for violation in &violations {
            // Encode violation type
            let violation_type = match violation {
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::DoubleFree { .. } => 1u8,
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::InvalidFree { .. } => 2u8,
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::PotentialLeak { .. } => 3u8,
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::CrossBoundaryRisk {
                    ..
                } => 4u8,
            };
            encoder.encode_u8(violation_type);

            // Encode violation-specific data
            match violation {
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::DoubleFree {
                    first_free_stack,
                    second_free_stack,
                    timestamp,
                } => {
                    encoder.encode_u64(*timestamp as u64);
                    self.encode_stack_trace(&mut encoder, first_free_stack);
                    self.encode_stack_trace(&mut encoder, second_free_stack);
                }
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::InvalidFree {
                    attempted_pointer,
                    stack,
                    timestamp,
                } => {
                    encoder.encode_usize(*attempted_pointer);
                    encoder.encode_u64(*timestamp as u64);
                    self.encode_stack_trace(&mut encoder, stack);
                }
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::PotentialLeak {
                    allocation_stack,
                    allocation_timestamp,
                    leak_detection_timestamp,
                } => {
                    encoder.encode_u64(*allocation_timestamp as u64);
                    encoder.encode_u64(*leak_detection_timestamp as u64);
                    self.encode_stack_trace(&mut encoder, allocation_stack);
                }
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::CrossBoundaryRisk {
                    risk_level,
                    description,
                    stack,
                } => {
                    let risk_value = match risk_level {
                        crate::analysis::unsafe_ffi_tracker::RiskLevel::Low => 1u8,
                        crate::analysis::unsafe_ffi_tracker::RiskLevel::Medium => 2u8,
                        crate::analysis::unsafe_ffi_tracker::RiskLevel::High => 3u8,
                        crate::analysis::unsafe_ffi_tracker::RiskLevel::Critical => 4u8,
                    };
                    encoder.encode_u8(risk_value);
                    encoder.encode_string(description);
                    self.encode_stack_trace(&mut encoder, stack);
                }
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Encode memory passports section for cross-boundary tracking
    fn encode_memory_passports(
        &mut self,
        _allocations: &[AllocationInfo],
    ) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        // Get memory passports from the global FFI tracker
        let ffi_tracker = get_global_unsafe_ffi_tracker();
        let passports = ffi_tracker.get_memory_passports()?;

        encoder.encode_u32(passports.len() as u32);

        for (ptr, passport) in &passports {
            encoder.encode_usize(*ptr);
            encoder.encode_string(&passport.passport_id);

            // Encode origin information
            encoder.encode_string(&passport.origin.context);
            encoder.encode_string(&passport.origin.allocator_function);
            encoder.encode_u64(passport.origin.timestamp as u64);

            // Encode journey stamps
            encoder.encode_u32(passport.journey.len() as u32);
            for stamp in &passport.journey {
                encoder.encode_u64(stamp.timestamp as u64);
                encoder.encode_string(&stamp.location);
                encoder.encode_string(&stamp.operation);
                encoder.encode_string(&stamp.authority);
                encoder.encode_string(&stamp.verification_hash);
            }

            // Encode current ownership
            encoder.encode_string(&passport.current_owner.owner_context);
            encoder.encode_string(&passport.current_owner.owner_function);
            encoder.encode_u64(passport.current_owner.transfer_timestamp as u64);

            // Encode validity status
            let validity_value = match passport.validity_status {
                crate::analysis::unsafe_ffi_tracker::ValidityStatus::Valid => 1u8,
                crate::analysis::unsafe_ffi_tracker::ValidityStatus::Expired => 2u8,
                crate::analysis::unsafe_ffi_tracker::ValidityStatus::Revoked => 3u8,
                crate::analysis::unsafe_ffi_tracker::ValidityStatus::Suspicious => 4u8,
            };
            encoder.encode_u8(validity_value);

            // Encode security clearance
            let clearance_value = match passport.security_clearance {
                crate::analysis::unsafe_ffi_tracker::SecurityClearance::Public => 1u8,
                crate::analysis::unsafe_ffi_tracker::SecurityClearance::Restricted => 2u8,
                crate::analysis::unsafe_ffi_tracker::SecurityClearance::Confidential => 3u8,
                crate::analysis::unsafe_ffi_tracker::SecurityClearance::Secret => 4u8,
            };
            encoder.encode_u8(clearance_value);
        }

        Ok(encoder.into_bytes())
    }

    /// Implement incremental encoding support to reduce duplicate data
    fn encode_with_incremental_support(
        &mut self,
        allocations: &[AllocationInfo],
    ) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        // Create a deduplication map for common data patterns
        let mut type_frequency: HashMap<String, u32> = HashMap::new();
        let mut size_frequency: HashMap<usize, u32> = HashMap::new();
        let mut scope_frequency: HashMap<String, u32> = HashMap::new();

        // Analyze data patterns for optimization
        for allocation in allocations {
            if let Some(type_name) = &allocation.type_name {
                *type_frequency.entry(type_name.clone()).or_insert(0) += 1;
            }
            *size_frequency.entry(allocation.size).or_insert(0) += 1;
            if let Some(scope_name) = &allocation.scope_name {
                *scope_frequency.entry(scope_name.clone()).or_insert(0) += 1;
            }
        }

        // Encode frequency tables for delta compression
        encoder.encode_u32(type_frequency.len() as u32);
        for (type_name, frequency) in &type_frequency {
            encoder.encode_string(type_name);
            encoder.encode_u32(*frequency);
        }

        encoder.encode_u32(size_frequency.len() as u32);
        for (size, frequency) in &size_frequency {
            encoder.encode_usize(*size);
            encoder.encode_u32(*frequency);
        }

        encoder.encode_u32(scope_frequency.len() as u32);
        for (scope_name, frequency) in &scope_frequency {
            encoder.encode_string(scope_name);
            encoder.encode_u32(*frequency);
        }

        // Encode allocations with delta compression
        encoder.encode_u32(allocations.len() as u32);
        let mut last_ptr = 0usize;
        let mut last_size = 0usize;
        let mut last_timestamp = 0u64;

        for allocation in allocations {
            // Delta encode pointer (most allocations are sequential)
            let ptr_delta = allocation.ptr.wrapping_sub(last_ptr);
            encoder.encode_usize(ptr_delta);
            last_ptr = allocation.ptr;

            // Delta encode size (many allocations have similar sizes)
            let size_delta = allocation.size.wrapping_sub(last_size);
            encoder.encode_usize(size_delta);
            last_size = allocation.size;

            // Delta encode timestamp (allocations are usually chronological)
            let timestamp_delta = allocation.timestamp_alloc.wrapping_sub(last_timestamp);
            encoder.encode_u64(timestamp_delta);
            last_timestamp = allocation.timestamp_alloc;

            // Use frequency-based encoding for common fields
            if let Some(type_name) = &allocation.type_name {
                encoder.encode_u8(1);
                if type_frequency[type_name] > 10 {
                    // Use reference to frequency table for common types
                    encoder.encode_u8(1); // Frequency reference marker
                    encoder.encode_string(type_name);
                } else {
                    encoder.encode_u8(0); // Direct encoding marker
                    encoder.encode_string(type_name);
                }
            } else {
                encoder.encode_u8(0);
            }

            // Encode other fields normally
            encoder.encode_optional_string(&allocation.var_name);
            encoder.encode_optional_string(&allocation.scope_name);
            encoder.encode_string(&allocation.thread_id);
            encoder.encode_usize(allocation.borrow_count);
            encoder.encode_u8(if allocation.is_leaked { 1 } else { 0 });
        }

        Ok(encoder.into_bytes())
    }

    /// Static method to encode security violations (thread-safe)
    fn encode_security_violations_static(
        _allocations: &[AllocationInfo],
    ) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        // Get security violations from the global FFI tracker
        let ffi_tracker = get_global_unsafe_ffi_tracker();
        let violations = ffi_tracker.get_safety_violations()?;

        encoder.encode_u32(violations.len() as u32);

        for violation in &violations {
            // Encode violation type
            let violation_type = match violation {
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::DoubleFree { .. } => 1u8,
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::InvalidFree { .. } => 2u8,
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::PotentialLeak { .. } => 3u8,
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::CrossBoundaryRisk {
                    ..
                } => 4u8,
            };
            encoder.encode_u8(violation_type);

            // Encode violation-specific data
            match violation {
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::DoubleFree {
                    first_free_stack,
                    second_free_stack,
                    timestamp,
                } => {
                    encoder.encode_u64(*timestamp as u64);
                    Self::encode_stack_trace_static(&mut encoder, first_free_stack);
                    Self::encode_stack_trace_static(&mut encoder, second_free_stack);
                }
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::InvalidFree {
                    attempted_pointer,
                    stack,
                    timestamp,
                } => {
                    encoder.encode_usize(*attempted_pointer);
                    encoder.encode_u64(*timestamp as u64);
                    Self::encode_stack_trace_static(&mut encoder, stack);
                }
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::PotentialLeak {
                    allocation_stack,
                    allocation_timestamp,
                    leak_detection_timestamp,
                } => {
                    encoder.encode_u64(*allocation_timestamp as u64);
                    encoder.encode_u64(*leak_detection_timestamp as u64);
                    Self::encode_stack_trace_static(&mut encoder, allocation_stack);
                }
                crate::analysis::unsafe_ffi_tracker::SafetyViolation::CrossBoundaryRisk {
                    risk_level,
                    description,
                    stack,
                } => {
                    let risk_value = match risk_level {
                        crate::analysis::unsafe_ffi_tracker::RiskLevel::Low => 1u8,
                        crate::analysis::unsafe_ffi_tracker::RiskLevel::Medium => 2u8,
                        crate::analysis::unsafe_ffi_tracker::RiskLevel::High => 3u8,
                        crate::analysis::unsafe_ffi_tracker::RiskLevel::Critical => 4u8,
                    };
                    encoder.encode_u8(risk_value);
                    encoder.encode_string(description);
                    Self::encode_stack_trace_static(&mut encoder, stack);
                }
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Static method to encode memory passports (thread-safe)
    fn encode_memory_passports_static(_allocations: &[AllocationInfo]) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::new();

        // Get memory passports from the global FFI tracker
        let ffi_tracker = get_global_unsafe_ffi_tracker();
        let passports = ffi_tracker.get_memory_passports()?;

        encoder.encode_u32(passports.len() as u32);

        for (ptr, passport) in &passports {
            encoder.encode_usize(*ptr);
            encoder.encode_string(&passport.passport_id);

            // Encode origin information
            encoder.encode_string(&passport.origin.context);
            encoder.encode_string(&passport.origin.allocator_function);
            encoder.encode_u64(passport.origin.timestamp as u64);

            // Encode journey stamps
            encoder.encode_u32(passport.journey.len() as u32);
            for stamp in &passport.journey {
                encoder.encode_u64(stamp.timestamp as u64);
                encoder.encode_string(&stamp.location);
                encoder.encode_string(&stamp.operation);
                encoder.encode_string(&stamp.authority);
                encoder.encode_string(&stamp.verification_hash);
            }

            // Encode current ownership
            encoder.encode_string(&passport.current_owner.owner_context);
            encoder.encode_string(&passport.current_owner.owner_function);
            encoder.encode_u64(passport.current_owner.transfer_timestamp as u64);

            // Encode validity status
            let validity_value = match passport.validity_status {
                crate::analysis::unsafe_ffi_tracker::ValidityStatus::Valid => 1u8,
                crate::analysis::unsafe_ffi_tracker::ValidityStatus::Expired => 2u8,
                crate::analysis::unsafe_ffi_tracker::ValidityStatus::Revoked => 3u8,
                crate::analysis::unsafe_ffi_tracker::ValidityStatus::Suspicious => 4u8,
            };
            encoder.encode_u8(validity_value);

            // Encode security clearance
            let clearance_value = match passport.security_clearance {
                crate::analysis::unsafe_ffi_tracker::SecurityClearance::Public => 1u8,
                crate::analysis::unsafe_ffi_tracker::SecurityClearance::Restricted => 2u8,
                crate::analysis::unsafe_ffi_tracker::SecurityClearance::Confidential => 3u8,
                crate::analysis::unsafe_ffi_tracker::SecurityClearance::Secret => 4u8,
            };
            encoder.encode_u8(clearance_value);
        }

        Ok(encoder.into_bytes())
    }

    /// Static helper method to encode stack traces
    fn encode_stack_trace_static(
        encoder: &mut BinaryEncoder,
        stack: &[crate::analysis::unsafe_ffi_tracker::StackFrame],
    ) {
        encoder.encode_u32(stack.len() as u32);
        for frame in stack {
            encoder.encode_string(&frame.function_name);
            encoder.encode_optional_string(&frame.file_name);
            if let Some(line_number) = frame.line_number {
                encoder.encode_u8(1);
                encoder.encode_u32(line_number);
            } else {
                encoder.encode_u8(0);
            }
            encoder.encode_u8(if frame.is_unsafe { 1 } else { 0 });
        }
    }

    /// Helper method to encode stack traces
    fn encode_stack_trace(
        &mut self,
        encoder: &mut BinaryEncoder,
        stack: &[crate::analysis::unsafe_ffi_tracker::StackFrame],
    ) {
        Self::encode_stack_trace_static(encoder, stack);
    }

    /// Create sections with memory mapping optimization
    fn create_sections_with_memory_mapping(
        &mut self,
        allocations: &[AllocationInfo],
        stats: &MemoryStats,
        memory_by_type: &[TypeMemoryUsage],
        progress: &mut ExportProgress,
    ) -> TrackingResult<HashMap<SectionType, Vec<u8>>> {
        let mut sections = HashMap::new();

        // Use memory-efficient section creation
        if self.options.section_selection.include_memory_stats {
            let data = self.encode_memory_stats_optimized(stats)?;
            sections.insert(SectionType::MemoryStats, data);
            progress.sections_completed += 1;
            if let Some(ref callback) = self.progress_callback {
                callback(progress);
            }
        }

        if self.options.section_selection.include_active_allocations {
            let data = self.encode_active_allocations_optimized(allocations)?;
            sections.insert(SectionType::ActiveAllocations, data);
            progress.sections_completed += 1;
            if let Some(ref callback) = self.progress_callback {
                callback(progress);
            }
        }

        if self.options.section_selection.include_type_memory_usage {
            let data = self.encode_type_memory_usage_optimized(memory_by_type)?;
            sections.insert(SectionType::TypeMemoryUsage, data);
            progress.sections_completed += 1;
            if let Some(ref callback) = self.progress_callback {
                callback(progress);
            }
        }

        // Add other sections as needed...

        Ok(sections)
    }

    /// Write binary file using memory-mapped writer
    fn write_binary_file_memory_mapped(
        &mut self,
        writer: &mut MemoryMappedWriter,
        sections: HashMap<SectionType, Vec<u8>>,
        progress: &mut ExportProgress,
    ) -> TrackingResult<()> {
        // Create binary file structure
        let mut binary_file = BinaryFile {
            header: BinaryHeader::new(),
            directory: SectionDirectory::new(),
            section_data: HashMap::new(),
        };

        // Calculate offsets and create section directory
        let mut current_offset = self.calculate_header_and_directory_size(&sections);

        for (section_type, data) in &sections {
            // Apply compression if enabled
            let (compressed_data, compression_used) = self.compress_section_data(data)?;

            let entry = SectionEntry::new(
                *section_type,
                compression_used,
                current_offset,
                compressed_data.len() as u32,
                data.len() as u32,
            );

            binary_file.directory.add_section(entry);
            binary_file
                .section_data
                .insert(*section_type, compressed_data.clone());

            current_offset += compressed_data.len() as u64;
        }

        // Update header
        binary_file.header.section_count = sections.len() as u32;
        binary_file.header.total_size = current_offset;
        binary_file.header.calculate_checksum();

        // Write header
        let header_bytes = binary_file.header.to_bytes();
        writer.write(&header_bytes).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!("Failed to write header: {}", e))
        })?;

        // Write section directory
        let directory_bytes = binary_file.directory.to_bytes();
        writer.write(&directory_bytes).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to write directory: {}",
                e
            ))
        })?;

        // Write section data
        for section_type in [
            SectionType::MemoryStats,
            SectionType::ActiveAllocations,
            SectionType::TypeMemoryUsage,
            // Add other sections in order...
        ] {
            if let Some(data) = binary_file.section_data.get(&section_type) {
                writer.write(data).map_err(|e| {
                    crate::core::types::TrackingError::ExportError(format!(
                        "Failed to write section {:?}: {}",
                        section_type, e
                    ))
                })?;

                progress.bytes_processed += data.len();
                if let Some(ref callback) = self.progress_callback {
                    callback(progress);
                }
            }
        }

        Ok(())
    }

    /// Memory-optimized encoding for memory stats
    fn encode_memory_stats_optimized(&mut self, stats: &MemoryStats) -> TrackingResult<Vec<u8>> {
        // Use smaller buffer for memory efficiency
        let mut encoder = BinaryEncoder::with_capacity(256);

        encoder.encode_usize(stats.total_allocations);
        encoder.encode_usize(stats.total_deallocations);
        encoder.encode_usize(stats.peak_allocations);
        encoder.encode_usize(stats.peak_memory);
        encoder.encode_usize(stats.active_allocations);
        encoder.encode_usize(stats.active_memory);

        Ok(encoder.into_bytes())
    }

    /// Memory-optimized encoding for active allocations
    fn encode_active_allocations_optimized(
        &mut self,
        allocations: &[AllocationInfo],
    ) -> TrackingResult<Vec<u8>> {
        // Process allocations in batches to reduce memory usage
        let batch_size = self.options.performance.batch_size;
        let mut encoder = BinaryEncoder::with_capacity(allocations.len() * 64); // Estimate

        encoder.encode_u32(allocations.len() as u32);

        for batch in allocations.chunks(batch_size) {
            for allocation in batch {
                encoder.encode_usize(allocation.ptr);
                encoder.encode_usize(allocation.size);
                encoder.encode_optional_type_name(&allocation.type_name);
                encoder.encode_optional_string(&allocation.var_name);
                encoder.encode_optional_string(&allocation.scope_name);
                encoder.encode_u64(allocation.timestamp_alloc);
                encoder.encode_u64(allocation.timestamp_dealloc.unwrap_or(0));
                encoder.encode_u8(if allocation.is_leaked { 1 } else { 0 });
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Memory-optimized encoding for type memory usage
    fn encode_type_memory_usage_optimized(
        &mut self,
        memory_by_type: &[TypeMemoryUsage],
    ) -> TrackingResult<Vec<u8>> {
        let mut encoder = BinaryEncoder::with_capacity(memory_by_type.len() * 32);

        encoder.encode_u32(memory_by_type.len() as u32);

        for usage in memory_by_type {
            encoder.encode_string(&usage.type_name);
            encoder.encode_usize(usage.total_size);
            encoder.encode_usize(usage.allocation_count);
        }

        Ok(encoder.into_bytes())
    }

    /// Create sections with zero-copy optimization
    fn create_sections_with_zero_copy(
        &mut self,
        allocations: &[AllocationInfo],
        stats: &MemoryStats,
        memory_by_type: &[TypeMemoryUsage],
        progress: &mut ExportProgress,
    ) -> TrackingResult<HashMap<SectionType, bytes::Bytes>> {
        let mut sections = HashMap::new();

        let buffer_pool = self.buffer_pool.as_ref().ok_or_else(|| {
            crate::core::types::TrackingError::ExportError("Zero-copy not initialized".to_string())
        })?;

        let vectorized_processor = VectorizedProcessor::new(buffer_pool.clone());

        // Memory stats section with zero-copy
        if self.options.section_selection.include_memory_stats {
            let buffer_pool_clone = buffer_pool.clone();
            let data = self.encode_memory_stats_zero_copy(stats, &buffer_pool_clone)?;
            sections.insert(SectionType::MemoryStats, data);
            progress.sections_completed += 1;
            if let Some(ref callback) = self.progress_callback {
                callback(progress);
            }
        }

        // Active allocations with vectorized processing
        if self.options.section_selection.include_active_allocations {
            let data = vectorized_processor
                .process_allocations_batch(
                    allocations,
                    self.options.performance.batch_size,
                    |batch, buffer| {
                        for allocation in batch {
                            buffer.write_usize_le(allocation.ptr);
                            buffer.write_usize_le(allocation.size);
                            buffer.write_optional_string(&allocation.type_name);
                            buffer.write_optional_string(&allocation.var_name);
                            buffer.write_optional_string(&allocation.scope_name);
                            buffer.write_u64_le(allocation.timestamp_alloc);
                            buffer.write_u8(if allocation.timestamp_dealloc.is_some() {
                                1
                            } else {
                                0
                            });
                            if let Some(dealloc_time) = allocation.timestamp_dealloc {
                                buffer.write_u64_le(dealloc_time);
                            }
                            buffer.write_u8(if allocation.is_leaked { 1 } else { 0 });
                        }
                        Ok(())
                    },
                )
                .map_err(|e| {
                    crate::core::types::TrackingError::ExportError(format!(
                        "Vectorized processing failed: {}",
                        e
                    ))
                })?;

            sections.insert(SectionType::ActiveAllocations, data);
            progress.sections_completed += 1;
            if let Some(ref callback) = self.progress_callback {
                callback(progress);
            }
        }

        // Type memory usage with zero-copy
        if self.options.section_selection.include_type_memory_usage {
            let data = vectorized_processor
                .process_type_names(memory_by_type)
                .map_err(|e| {
                    crate::core::types::TrackingError::ExportError(format!(
                        "Type processing failed: {}",
                        e
                    ))
                })?;
            sections.insert(SectionType::TypeMemoryUsage, data);
            progress.sections_completed += 1;
            if let Some(ref callback) = self.progress_callback {
                callback(progress);
            }
        }

        Ok(sections)
    }

    /// Write binary file using zero-copy writer
    fn write_binary_file_zero_copy<W: std::io::Write>(
        &mut self,
        writer: &mut ZeroCopyWriter<W>,
        sections: HashMap<SectionType, bytes::Bytes>,
        progress: &mut ExportProgress,
    ) -> TrackingResult<()> {
        // Create binary file structure
        let mut binary_file = BinaryFile {
            header: BinaryHeader::new(),
            directory: SectionDirectory::new(),
            section_data: HashMap::new(),
        };

        // Calculate offsets and create section directory
        let mut current_offset = self.calculate_header_and_directory_size_bytes(&sections);

        for (section_type, data) in &sections {
            // Apply compression if enabled
            let (compressed_data, compression_used) = self.compress_section_data_bytes(data)?;

            let entry = SectionEntry::new(
                *section_type,
                compression_used,
                current_offset,
                compressed_data.len() as u32,
                data.len() as u32,
            );

            binary_file.directory.add_section(entry);
            binary_file
                .section_data
                .insert(*section_type, compressed_data.to_vec());

            current_offset += compressed_data.len() as u64;
        }

        // Update header
        binary_file.header.section_count = sections.len() as u32;
        binary_file.header.total_size = current_offset;
        binary_file.header.calculate_checksum();

        // Write header using zero-copy
        let header_bytes = binary_file.header.to_bytes();
        writer.write_zero_copy(&header_bytes).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!("Failed to write header: {}", e))
        })?;

        // Write section directory using zero-copy
        let directory_bytes = binary_file.directory.to_bytes();
        writer.write_zero_copy(&directory_bytes).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to write directory: {}",
                e
            ))
        })?;

        // Write section data using zero-copy
        for section_type in [
            SectionType::MemoryStats,
            SectionType::ActiveAllocations,
            SectionType::TypeMemoryUsage,
            // Add other sections in order...
        ] {
            if let Some(data) = binary_file.section_data.get(&section_type) {
                writer.write_zero_copy(data).map_err(|e| {
                    crate::core::types::TrackingError::ExportError(format!(
                        "Failed to write section {:?}: {}",
                        section_type, e
                    ))
                })?;

                progress.bytes_processed += data.len();
                if let Some(ref callback) = self.progress_callback {
                    callback(progress);
                }
            }
        }

        Ok(())
    }

    /// Zero-copy encoding for memory stats
    fn encode_memory_stats_zero_copy(
        &mut self,
        stats: &MemoryStats,
        pool: &ZeroCopyBufferPool,
    ) -> TrackingResult<bytes::Bytes> {
        let mut buffer = pool.get_buffer(64); // Small fixed size for stats

        buffer.write_usize_le(stats.total_allocations);
        buffer.write_usize_le(stats.total_deallocations);
        buffer.write_usize_le(stats.peak_allocations);
        buffer.write_usize_le(stats.peak_memory);
        buffer.write_usize_le(stats.active_allocations);
        buffer.write_usize_le(stats.active_memory);

        Ok(buffer.freeze())
    }

    /// Calculate header and directory size for Bytes sections
    fn calculate_header_and_directory_size_bytes(
        &self,
        sections: &HashMap<SectionType, bytes::Bytes>,
    ) -> u64 {
        let header_size = 64u64; // Fixed header size
        let directory_size = sections.len() as u64 * 20; // 20 bytes per section entry
        header_size + directory_size
    }

    /// Compress section data from Bytes
    fn compress_section_data_bytes(
        &self,
        data: &bytes::Bytes,
    ) -> TrackingResult<(bytes::Bytes, CompressionType)> {
        if data.len() < self.options.compression_threshold {
            return Ok((data.clone(), CompressionType::None));
        }

        match self.options.compression {
            CompressionType::None => Ok((data.clone(), CompressionType::None)),
            CompressionType::Lz4 => {
                // For now, return uncompressed - compression can be added later
                Ok((data.clone(), CompressionType::None))
            }
            CompressionType::Zstd => {
                // For now, return uncompressed - compression can be added later
                Ok((data.clone(), CompressionType::None))
            }
        }
    }
}

impl Default for BinaryExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AllocationInfo;

    #[test]
    fn test_binary_export_options() {
        let options = BinaryExportOptions::default();
        assert_eq!(options.compression, CompressionType::Lz4);
        assert!(options.include_history);
        assert!(options.include_ffi_analysis);

        let fast_options = BinaryExportOptions::fast();
        assert_eq!(fast_options.compression, CompressionType::Lz4);
        assert!(!fast_options.include_history);
        assert!(!fast_options.include_ffi_analysis);

        let comprehensive_options = BinaryExportOptions::comprehensive();
        assert_eq!(comprehensive_options.compression, CompressionType::Zstd);
        assert!(comprehensive_options.include_history);
        assert!(comprehensive_options.include_ffi_analysis);
    }

    #[test]
    fn test_export_progress() {
        let start_time = std::time::Instant::now();
        let progress = ExportProgress {
            current_section: SectionType::MemoryStats,
            sections_completed: 2,
            total_sections: 5,
            bytes_processed: 1000,
            estimated_total_bytes: 5000,
            start_time,
        };

        assert_eq!(progress.completion_percentage(), 40.0);
        assert!(progress.elapsed_time().as_nanos() > 0);
    }

    #[test]
    fn test_binary_export_result() {
        let result = BinaryExportResult {
            file_path: "test.membin".to_string(),
            file_size: 1000,
            original_size: 2000,
            compression_ratio: 0.5,
            export_duration: std::time::Duration::from_millis(100),
            sections_exported: 5,
            allocations_exported: 100,
            peak_memory_usage: 1024 * 1024,
            used_parallel_processing: true,
        };

        assert_eq!(result.space_savings_percentage(), 50.0);
        assert!(result.export_speed_mbps() > 0.0);
    }

    #[test]
    fn test_binary_exporter_creation() {
        let exporter = BinaryExporter::new();
        assert_eq!(exporter.options.compression, CompressionType::Lz4);

        let custom_options = BinaryExportOptions::with_compression(CompressionType::Zstd);
        let custom_exporter = BinaryExporter::with_options(custom_options);
        assert_eq!(custom_exporter.options.compression, CompressionType::Zstd);
    }

    #[test]
    fn test_section_encoding() {
        let mut exporter = BinaryExporter::new();

        // Test memory stats encoding
        let stats = MemoryStats {
            total_allocated: 1000,
            total_deallocated: 500,
            active_memory: 500,
            peak_memory: 800,
            active_allocations: 10,
            total_allocations: 20,
            peak_allocations: 15,
            leaked_allocations: 2,
            leaked_memory: 100,
            ..Default::default()
        };

        let encoded = exporter.encode_memory_stats(&stats).unwrap();
        assert!(!encoded.is_empty());

        // Test allocations encoding
        let mut allocation = AllocationInfo::new(0x1000, 64);
        allocation.var_name = Some("test_var".to_string());
        allocation.type_name = Some("i32".to_string());

        let allocations = vec![allocation];
        let encoded = exporter.encode_allocations(&allocations).unwrap();
        assert!(!encoded.is_empty());
    }
}
