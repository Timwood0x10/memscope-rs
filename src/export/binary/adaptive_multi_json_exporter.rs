//! Adaptive multi-file JSON export system
//!
//! This module provides intelligent strategy selection for binary-to-JSON conversion
//! based on file size and complexity. It automatically chooses between simple direct
//! processing, index optimization, and full streaming processing to achieve optimal
//! performance for different file sizes.

use crate::core::types::AllocationInfo;
use crate::export::binary::{
    BinaryExportError, BinaryIndexBuilder, BinaryParser, SelectiveJsonExporter,
    SelectiveJsonExportConfig, SelectiveJsonExportStats, AllocationField,
    AllocationFilter, SelectiveReadOptions,
};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// File size thresholds for strategy selection
const SMALL_FILE_THRESHOLD: u64 = 150 * 1024; // 150KB
const STREAMING_OPTIMIZATION_THRESHOLD: u64 = 1024 * 1024; // 1MB

/// JSON output types supported by the adaptive exporter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonType {
    MemoryAnalysis,
    LifetimeAnalysis,
    PerformanceAnalysis,
    ComplexTypes,
    UnsafeFFI,
}

impl JsonType {
    /// Get the filename suffix for this JSON type
    pub fn filename_suffix(&self) -> &'static str {
        match self {
            JsonType::MemoryAnalysis => "memory_analysis",
            JsonType::LifetimeAnalysis => "lifetime",
            JsonType::PerformanceAnalysis => "performance",
            JsonType::ComplexTypes => "complex_types",
            JsonType::UnsafeFFI => "unsafe_ffi",
        }
    }

    /// Get the required fields for this JSON type
    pub fn required_fields(&self) -> Vec<AllocationField> {
        match self {
            JsonType::MemoryAnalysis => vec![
                AllocationField::Ptr,
                AllocationField::Size,
                AllocationField::VarName,
                AllocationField::TypeName,
                AllocationField::ThreadId,
                AllocationField::TimestampAlloc,
                AllocationField::IsLeaked,
                AllocationField::BorrowCount,
            ],
            JsonType::LifetimeAnalysis => vec![
                AllocationField::Ptr,
                AllocationField::VarName,
                AllocationField::TimestampAlloc,
                AllocationField::TimestampDealloc,
                AllocationField::LifetimeMs,
                AllocationField::ScopeName,
            ],
            JsonType::PerformanceAnalysis => vec![
                AllocationField::Ptr,
                AllocationField::Size,
                AllocationField::TimestampAlloc,
                AllocationField::ThreadId,
                AllocationField::BorrowCount,
            ],
            JsonType::ComplexTypes => vec![
                AllocationField::Ptr,
                AllocationField::Size,
                AllocationField::VarName,
                AllocationField::TypeName,
                AllocationField::SmartPointerInfo,
                AllocationField::MemoryLayout,
                AllocationField::GenericInfo,
            ],
            JsonType::UnsafeFFI => vec![
                AllocationField::Ptr,
                AllocationField::VarName,
                AllocationField::TypeName,
                AllocationField::ThreadId,
                AllocationField::StackTrace,
                AllocationField::RuntimeState,
            ],
        }
    }
}

/// Processing strategy used for export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessingStrategy {
    /// Simple direct processing for small files (<150KB)
    SimpleDirect,
    /// Index optimization for medium files (150KB-1MB)
    IndexOptimized,
    /// Full streaming processing for large files (>1MB)
    FullyStreaming,
}

impl ProcessingStrategy {
    /// Get a human-readable description of the strategy
    pub fn description(&self) -> &'static str {
        match self {
            ProcessingStrategy::SimpleDirect => "Simple direct processing",
            ProcessingStrategy::IndexOptimized => "Index-optimized processing",
            ProcessingStrategy::FullyStreaming => "Full streaming processing",
        }
    }
}

/// Statistics for multi-file export operations
#[derive(Debug, Clone)]
pub struct MultiExportStats {
    /// Total number of records processed
    pub total_records: usize,
    /// Maximum memory used during processing
    pub max_memory_used: usize,
    /// Cache hit rate for field processing
    pub cache_hit_rate: f64,
    /// Statistics for each JSON type
    pub per_json_stats: HashMap<JsonType, SelectiveJsonExportStats>,
    /// Total processing duration
    pub total_duration: Duration,
    /// Strategy used for processing
    pub strategy_used: ProcessingStrategy,
    /// File size that triggered strategy selection
    pub file_size: u64,
}

impl MultiExportStats {
    /// Calculate overall throughput (records per second)
    pub fn overall_throughput(&self) -> f64 {
        if self.total_duration.as_secs_f64() == 0.0 {
            0.0
        } else {
            self.total_records as f64 / self.total_duration.as_secs_f64()
        }
    }

    /// Calculate average processing time per record
    pub fn avg_processing_time_per_record(&self) -> Duration {
        if self.total_records == 0 {
            Duration::ZERO
        } else {
            self.total_duration / self.total_records as u32
        }
    }
}

/// Configuration for adaptive multi-file export
#[derive(Debug, Clone)]
pub struct AdaptiveExportConfig {
    /// Small file threshold (default: 150KB)
    pub small_file_threshold: u64,
    /// Streaming threshold (default: 1MB)
    pub streaming_threshold: u64,
    /// Force a specific strategy (overrides automatic selection)
    pub force_strategy: Option<ProcessingStrategy>,
    /// Enable pretty printing for JSON output
    pub pretty_print: bool,
    /// Buffer size for streaming operations
    pub buffer_size: usize,
    /// Enable parallel processing where applicable
    pub enable_parallel_processing: bool,
}

impl Default for AdaptiveExportConfig {
    fn default() -> Self {
        Self {
            small_file_threshold: SMALL_FILE_THRESHOLD,
            streaming_threshold: STREAMING_OPTIMIZATION_THRESHOLD,
            force_strategy: None,
            pretty_print: false,
            buffer_size: 64 * 1024, // 64KB
            enable_parallel_processing: true,
        }
    }
}

/// Adaptive multi-file JSON exporter with intelligent strategy selection
pub struct AdaptiveMultiJsonExporter {
    config: AdaptiveExportConfig,
}

impl AdaptiveMultiJsonExporter {
    /// Create a new adaptive exporter with default configuration
    pub fn new() -> Self {
        Self {
            config: AdaptiveExportConfig::default(),
        }
    }

    /// Create a new adaptive exporter with custom configuration
    pub fn new_with_config(config: AdaptiveExportConfig) -> Self {
        Self { config }
    }

    /// Export binary file to multiple JSON types with adaptive strategy selection
    pub fn export_adaptive<P: AsRef<Path>>(
        &self,
        binary_path: P,
        base_name: &str,
        json_types: &[JsonType],
    ) -> Result<MultiExportStats, BinaryExportError> {
        let binary_path = binary_path.as_ref();
        let start_time = Instant::now();

        // Determine file size and select strategy
        let file_size = std::fs::metadata(binary_path)
            .map_err(|e| BinaryExportError::IoError(e))?
            .len();

        let strategy = self.select_strategy(file_size);

        info!(
            "Processing file {} ({:.1}KB) using {}",
            binary_path.display(),
            file_size as f64 / 1024.0,
            strategy.description()
        );

        // Execute the selected strategy
        let mut stats = match strategy {
            ProcessingStrategy::SimpleDirect => {
                self.export_simple_direct(binary_path, base_name, json_types)?
            }
            ProcessingStrategy::IndexOptimized => {
                self.export_with_index_optimization(binary_path, base_name, json_types)?
            }
            ProcessingStrategy::FullyStreaming => {
                self.export_streaming_multi_json(binary_path, base_name, json_types)?
            }
        };

        stats.total_duration = start_time.elapsed();
        stats.strategy_used = strategy;
        stats.file_size = file_size;

        info!(
            "Export completed in {:.2}s using {} (throughput: {:.1} records/s)",
            stats.total_duration.as_secs_f64(),
            strategy.description(),
            stats.overall_throughput()
        );

        Ok(stats)
    }

    /// Select the optimal processing strategy based on file size
    fn select_strategy(&self, file_size: u64) -> ProcessingStrategy {
        // Check for forced strategy first
        if let Some(forced) = self.config.force_strategy {
            info!("Using forced strategy: {}", forced.description());
            return forced;
        }

        // Automatic strategy selection based on file size
        if file_size <= self.config.small_file_threshold {
            ProcessingStrategy::SimpleDirect
        } else if file_size <= self.config.streaming_threshold {
            ProcessingStrategy::IndexOptimized
        } else {
            ProcessingStrategy::FullyStreaming
        }
    }
}

impl Default for AdaptiveMultiJsonExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_type_filename_suffix() {
        assert_eq!(JsonType::MemoryAnalysis.filename_suffix(), "memory_analysis");
        assert_eq!(JsonType::LifetimeAnalysis.filename_suffix(), "lifetime");
        assert_eq!(JsonType::PerformanceAnalysis.filename_suffix(), "performance");
        assert_eq!(JsonType::ComplexTypes.filename_suffix(), "complex_types");
        assert_eq!(JsonType::UnsafeFFI.filename_suffix(), "unsafe_ffi");
    }

    #[test]
    fn test_strategy_selection() {
        let exporter = AdaptiveMultiJsonExporter::new();

        // Small file
        assert_eq!(
            exporter.select_strategy(100 * 1024), // 100KB
            ProcessingStrategy::SimpleDirect
        );

        // Medium file
        assert_eq!(
            exporter.select_strategy(500 * 1024), // 500KB
            ProcessingStrategy::IndexOptimized
        );

        // Large file
        assert_eq!(
            exporter.select_strategy(2 * 1024 * 1024), // 2MB
            ProcessingStrategy::FullyStreaming
        );
    }

    #[test]
    fn test_forced_strategy() {
        let config = AdaptiveExportConfig {
            force_strategy: Some(ProcessingStrategy::FullyStreaming),
            ..Default::default()
        };
        let exporter = AdaptiveMultiJsonExporter::new_with_config(config);

        // Should use forced strategy regardless of file size
        assert_eq!(
            exporter.select_strategy(50 * 1024), // 50KB (normally SimpleDirect)
            ProcessingStrategy::FullyStreaming
        );
    }

    #[test]
    fn test_multi_export_stats_calculations() {
        let stats = MultiExportStats {
            total_records: 1000,
            max_memory_used: 1024 * 1024,
            cache_hit_rate: 0.85,
            per_json_stats: HashMap::new(),
            total_duration: Duration::from_secs(2),
            strategy_used: ProcessingStrategy::IndexOptimized,
            file_size: 500 * 1024,
        };

        assert_eq!(stats.overall_throughput(), 500.0); // 1000 records / 2 seconds
        assert_eq!(stats.avg_processing_time_per_record(), Duration::from_millis(2));
    }
}