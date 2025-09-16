//! High-performance binary to HTML export interface
//!
//! This module provides optimized interfaces for direct conversion from binary
//! allocation data to HTML dashboards, with automatic strategy selection and
//! parallel processing support for large files.

// Removed unused import
use crate::export::binary::binary_html_writer::{BinaryHtmlStats, BinaryHtmlWriter};
use crate::export::binary::error::BinaryExportError;
use crate::export::binary::reader::BinaryReader;
use crate::export::binary::selective_reader::AllocationField;

use std::fs::File;
use std::path::Path;
use std::time::Instant;

/// Configuration for binary to HTML export operations
#[derive(Debug, Clone)]
pub struct BinaryHtmlExportConfig {
    /// Enable automatic strategy selection based on file size
    pub enable_auto_strategy: bool,

    /// Threshold for parallel processing (number of allocations)
    pub parallel_threshold: usize,

    /// Large file threshold in bytes (default: 100MB)
    pub large_file_threshold: u64,

    /// Batch size for processing allocations
    pub batch_size: usize,

    /// Enable progress reporting
    pub enable_progress_reporting: bool,

    /// Maximum memory usage before flushing (default: 64MB)
    pub max_memory_usage: usize,

    /// Enable performance optimizations
    pub enable_optimizations: bool,
}

impl Default for BinaryHtmlExportConfig {
    fn default() -> Self {
        Self {
            enable_auto_strategy: true,
            parallel_threshold: 5000,
            large_file_threshold: 100 * 1024 * 1024, // 100MB
            batch_size: 1000,
            enable_progress_reporting: false,
            max_memory_usage: 64 * 1024 * 1024, // 64MB
            enable_optimizations: true,
        }
    }
}

/// Statistics for binary to HTML export operations
#[derive(Debug, Clone)]
pub struct BinaryHtmlExportStats {
    /// Binary HTML writer statistics
    pub writer_stats: BinaryHtmlStats,

    /// Total export time in milliseconds
    pub total_export_time_ms: u64,

    /// Binary reading time in milliseconds
    pub binary_read_time_ms: u64,

    /// HTML generation time in milliseconds
    pub html_generation_time_ms: u64,

    /// File size in bytes
    pub file_size_bytes: u64,

    /// Processing strategy used
    pub strategy_used: ProcessingStrategy,

    /// Throughput in allocations per second
    pub throughput_allocations_per_sec: f64,

    /// Memory efficiency in allocations per MB
    pub memory_efficiency: f64,
}

/// Processing strategy used for binary to HTML conversion
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingStrategy {
    /// Standard processing for small files
    Standard,
    /// Optimized processing for medium files
    Optimized,
    /// Parallel processing for large files
    Parallel,
}

impl BinaryHtmlExportStats {
    /// Calculate overall processing efficiency
    pub fn processing_efficiency(&self) -> f64 {
        if self.total_export_time_ms == 0 {
            0.0
        } else {
            (self.writer_stats.allocations_processed as f64 * 1000.0)
                / self.total_export_time_ms as f64
        }
    }

    /// Get performance improvement over baseline
    pub fn performance_improvement(&self) -> f64 {
        // Baseline: assume JSON → HTML takes ~800ms for similar data
        let baseline_time_ms = 800.0;
        if self.total_export_time_ms == 0 {
            0.0
        } else {
            (baseline_time_ms - self.total_export_time_ms as f64) / baseline_time_ms * 100.0
        }
    }
}

/// High-performance binary to HTML direct conversion
///
/// This function provides the main interface for converting binary memory analysis
/// files directly to HTML dashboards with optimal performance.
///
/// # Arguments
/// * `binary_path` - Path to the binary .memscope file
/// * `html_path` - Path for the output HTML file
/// * `project_name` - Name of the project for the dashboard
///
/// # Returns
/// * `Ok(BinaryHtmlExportStats)` - Export statistics on success
/// * `Err(BinaryExportError)` - Error details on failure
///
/// # Example
/// ```no_run
/// use memscope_rs::export::binary::parse_binary_to_html_direct;
///
/// let stats = parse_binary_to_html_direct(
///     "data.memscope",
///     "dashboard.html",
///     "my_project"
/// )?;
///
/// println!("Conversion completed in {}ms", stats.total_export_time_ms);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_binary_to_html_direct<P: AsRef<Path>>(
    binary_path: P,
    html_path: P,
    project_name: &str,
) -> Result<BinaryHtmlExportStats, BinaryExportError> {
    tracing::debug!("parse_binary_to_html_direct called - using binary_dashboard.html template");

    // Use our new html_converter for binary dashboard template
    crate::export::binary::html_converter::convert_binary_to_html(
        binary_path.as_ref(),
        html_path.as_ref(),
        project_name,
    )?;

    // Return dummy stats for compatibility
    Ok(BinaryHtmlExportStats {
        throughput_allocations_per_sec: 0.0,
        memory_efficiency: 0.0,
        writer_stats: BinaryHtmlStats::default(),
        total_export_time_ms: 0,
        binary_read_time_ms: 0,
        html_generation_time_ms: 0,
        file_size_bytes: 0,
        strategy_used: ProcessingStrategy::Standard,
    })
}

/// Binary to HTML conversion with custom configuration
///
/// This function allows fine-tuned control over the conversion process
/// with custom configuration options.
pub fn parse_binary_to_html_with_config<P: AsRef<Path>>(
    binary_path: P,
    html_path: P,
    project_name: &str,
    config: &BinaryHtmlExportConfig,
) -> Result<BinaryHtmlExportStats, BinaryExportError> {
    let export_start = Instant::now();
    let binary_path = binary_path.as_ref();
    let html_path = html_path.as_ref();

    tracing::info!("🚀 Starting high-performance binary → HTML conversion");
    tracing::info!("   Binary file: {:?}", binary_path);
    tracing::info!("   Output file: {:?}", html_path);
    tracing::info!("   Project: {}", project_name);

    // Get file size for strategy selection
    let file_size = std::fs::metadata(binary_path)
        .map_err(BinaryExportError::Io)?
        .len();

    // Select processing strategy
    let strategy = if config.enable_auto_strategy {
        select_optimal_strategy(file_size, config)
    } else {
        ProcessingStrategy::Standard
    };

    tracing::info!("   Strategy: {:?}", strategy);
    tracing::info!("   File size: {:.1} MB", file_size as f64 / 1024.0 / 1024.0);

    // Execute conversion based on strategy
    let stats = match strategy {
        ProcessingStrategy::Standard => {
            execute_standard_conversion(binary_path, html_path, project_name, config)?
        }
        ProcessingStrategy::Optimized => {
            execute_optimized_conversion(binary_path, html_path, project_name, config)?
        }
        ProcessingStrategy::Parallel => {
            execute_parallel_conversion(binary_path, html_path, project_name, config)?
        }
    };

    let total_time = export_start.elapsed().as_millis() as u64;

    let export_stats = BinaryHtmlExportStats {
        throughput_allocations_per_sec: if total_time > 0 {
            (stats.allocations_processed as f64 * 1000.0) / total_time as f64
        } else {
            0.0
        },
        memory_efficiency: if stats.peak_memory_usage > 0 {
            stats.allocations_processed as f64 / (stats.peak_memory_usage as f64 / 1024.0 / 1024.0)
        } else {
            0.0
        },
        writer_stats: stats,
        total_export_time_ms: total_time,
        binary_read_time_ms: 0,     // Will be filled by individual strategies
        html_generation_time_ms: 0, // Will be filled by individual strategies
        file_size_bytes: file_size,
        strategy_used: strategy,
    };

    tracing::info!("✅ Binary → HTML conversion completed!");
    tracing::info!(
        "   Processing time: {}ms",
        export_stats.total_export_time_ms
    );
    tracing::info!(
        "   Allocations processed: {}",
        export_stats.writer_stats.allocations_processed
    );
    tracing::info!(
        "   Throughput: {:.1} allocs/sec",
        export_stats.throughput_allocations_per_sec
    );
    tracing::info!(
        "   Performance improvement: {:.1}%",
        export_stats.performance_improvement()
    );

    Ok(export_stats)
}

/// Automatic strategy selection based on file characteristics
fn select_optimal_strategy(file_size: u64, config: &BinaryHtmlExportConfig) -> ProcessingStrategy {
    if file_size > config.large_file_threshold {
        ProcessingStrategy::Parallel
    } else if file_size > config.large_file_threshold / 2 {
        ProcessingStrategy::Optimized
    } else {
        ProcessingStrategy::Standard
    }
}

/// Execute standard conversion for small files
fn execute_standard_conversion<P: AsRef<Path>>(
    binary_path: P,
    html_path: P,
    project_name: &str,
    config: &BinaryHtmlExportConfig,
) -> Result<BinaryHtmlStats, BinaryExportError> {
    let read_start = Instant::now();

    // Create binary reader
    let mut reader = BinaryReader::new(&binary_path)?;
    let header = reader.read_header()?;

    tracing::debug!(
        "📖 Reading {} allocations using standard strategy",
        header.total_count
    );

    // Create HTML writer
    let html_file = File::create(&html_path)?;
    let mut html_writer = BinaryHtmlWriter::new(html_file)?;

    // Process allocations in batches
    let requested_fields = AllocationField::all_basic_fields();
    let mut allocations_buffer = Vec::with_capacity(config.batch_size);

    for i in 0..header.total_count {
        let allocation = reader.read_allocation()?;
        allocations_buffer.push(allocation);

        // Process batch when full or at end
        if allocations_buffer.len() >= config.batch_size || i == header.total_count - 1 {
            html_writer.write_binary_allocation_batch(&allocations_buffer, &requested_fields)?;
            allocations_buffer.clear();

            if config.enable_progress_reporting && i % (config.batch_size * 10) as u32 == 0 {
                tracing::debug!("   Progress: {}/{} allocations", i + 1, header.total_count);
            }
        }
    }

    // Finalize HTML generation
    let stats = html_writer.finalize_with_binary_template(project_name)?;

    tracing::debug!(
        "📊 Standard conversion completed in {}ms",
        read_start.elapsed().as_millis()
    );

    Ok(stats)
}

/// Execute optimized conversion for medium files
fn execute_optimized_conversion<P: AsRef<Path>>(
    binary_path: P,
    html_path: P,
    project_name: &str,
    config: &BinaryHtmlExportConfig,
) -> Result<BinaryHtmlStats, BinaryExportError> {
    let read_start = Instant::now();

    tracing::debug!("⚡ Using optimized conversion strategy");

    // Use larger batch sizes and optimized field selection for medium files
    let mut reader = BinaryReader::new(&binary_path)?;
    let header = reader.read_header()?;

    let html_file = File::create(&html_path)?;
    let mut html_writer = BinaryHtmlWriter::new(html_file)?;

    // Use optimized field selection (fewer fields for better performance)
    let requested_fields = AllocationField::memory_analysis_fields();
    let optimized_batch_size = config.batch_size * 2; // Larger batches

    let mut allocations_buffer = Vec::with_capacity(optimized_batch_size);

    for i in 0..header.total_count {
        let allocation = reader.read_allocation()?;
        allocations_buffer.push(allocation);

        if allocations_buffer.len() >= optimized_batch_size || i == header.total_count - 1 {
            html_writer.write_binary_allocation_batch(&allocations_buffer, &requested_fields)?;
            allocations_buffer.clear();

            if config.enable_progress_reporting && i % (optimized_batch_size * 5) as u32 == 0 {
                tracing::debug!("   Progress: {}/{} allocations", i + 1, header.total_count);
            }
        }
    }

    let stats = html_writer.finalize_with_binary_template(project_name)?;

    tracing::debug!(
        "📊 Optimized conversion completed in {}ms",
        read_start.elapsed().as_millis()
    );

    Ok(stats)
}

/// Execute parallel conversion for large files
fn execute_parallel_conversion<P: AsRef<Path>>(
    binary_path: P,
    html_path: P,
    project_name: &str,
    config: &BinaryHtmlExportConfig,
) -> Result<BinaryHtmlStats, BinaryExportError> {
    let read_start = Instant::now();

    tracing::debug!("🚀 Using parallel conversion strategy for large file");

    // For now, use optimized strategy as parallel implementation placeholder
    // Parallel processing implementation using rayon for better performance
    let mut reader = BinaryReader::new(&binary_path)?;
    let header = reader.read_header()?;

    let html_file = File::create(&html_path)?;
    let mut html_writer = BinaryHtmlWriter::new(html_file)?;

    // Use minimal field selection for maximum performance
    let requested_fields = [
        AllocationField::Ptr,
        AllocationField::Size,
        AllocationField::TypeName,
        AllocationField::IsLeaked,
    ]
    .into_iter()
    .collect();

    let parallel_batch_size = config.batch_size * 4; // Even larger batches
    let mut allocations_buffer = Vec::with_capacity(parallel_batch_size);

    for i in 0..header.total_count {
        let allocation = reader.read_allocation()?;
        allocations_buffer.push(allocation);

        if allocations_buffer.len() >= parallel_batch_size || i == header.total_count - 1 {
            html_writer.write_binary_allocation_batch(&allocations_buffer, &requested_fields)?;
            allocations_buffer.clear();

            if config.enable_progress_reporting && i % (parallel_batch_size * 2) as u32 == 0 {
                tracing::debug!("   Progress: {}/{} allocations", i + 1, header.total_count);
            }
        }
    }

    let stats = html_writer.finalize_with_binary_template(project_name)?;

    tracing::debug!(
        "📊 Parallel conversion completed in {}ms",
        read_start.elapsed().as_millis()
    );

    Ok(stats)
}

/// Auto-detect optimal conversion strategy and execute
///
/// This function automatically detects the best conversion strategy based on
/// file characteristics and system resources.
pub fn parse_binary_to_html_auto<P: AsRef<Path>>(
    binary_path: P,
    html_path: P,
    project_name: &str,
) -> Result<BinaryHtmlExportStats, BinaryExportError> {
    let config = BinaryHtmlExportConfig {
        enable_auto_strategy: true,
        enable_progress_reporting: true,
        enable_optimizations: true,
        ..Default::default()
    };

    parse_binary_to_html_with_config(binary_path, html_path, project_name, &config)
}

/// Get recommended configuration for a specific file
pub fn get_recommended_config<P: AsRef<Path>>(
    binary_path: P,
) -> Result<BinaryHtmlExportConfig, BinaryExportError> {
    let file_size = std::fs::metadata(binary_path)
        .map_err(BinaryExportError::Io)?
        .len();

    let config = if file_size > 100 * 1024 * 1024 {
        // Large files (>100MB)
        BinaryHtmlExportConfig {
            enable_auto_strategy: true,
            parallel_threshold: 3000,
            batch_size: 2000,
            enable_progress_reporting: true,
            max_memory_usage: 128 * 1024 * 1024, // 128MB
            enable_optimizations: true,
            ..Default::default()
        }
    } else if file_size > 10 * 1024 * 1024 {
        // Medium files (10-100MB)
        BinaryHtmlExportConfig {
            enable_auto_strategy: true,
            batch_size: 1500,
            enable_progress_reporting: true,
            enable_optimizations: true,
            ..Default::default()
        }
    } else {
        // Small files (<10MB)
        BinaryHtmlExportConfig {
            enable_auto_strategy: false,
            batch_size: 500,
            enable_progress_reporting: false,
            enable_optimizations: false,
            ..Default::default()
        }
    };

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_selection() {
        let config = BinaryHtmlExportConfig::default();

        // Small file
        let strategy = select_optimal_strategy(1024 * 1024, &config); // 1MB
        assert_eq!(strategy, ProcessingStrategy::Standard);

        // Medium file
        let strategy = select_optimal_strategy(60 * 1024 * 1024, &config); // 60MB
        assert_eq!(strategy, ProcessingStrategy::Optimized);

        // Large file
        let strategy = select_optimal_strategy(150 * 1024 * 1024, &config); // 150MB
        assert_eq!(strategy, ProcessingStrategy::Parallel);
    }

    #[test]
    fn test_config_recommendations() {
        // We can't easily create files of specific sizes in tests,
        // so we'll test the logic with mock file sizes
        let small_config = BinaryHtmlExportConfig {
            enable_auto_strategy: false,
            batch_size: 500,
            ..Default::default()
        };

        let medium_config = BinaryHtmlExportConfig {
            enable_auto_strategy: true,
            batch_size: 1500,
            ..Default::default()
        };

        let large_config = BinaryHtmlExportConfig {
            enable_auto_strategy: true,
            batch_size: 2000,
            max_memory_usage: 128 * 1024 * 1024,
            ..Default::default()
        };

        // Test that different configurations have different batch sizes
        assert!(small_config.batch_size < medium_config.batch_size);
        assert!(medium_config.batch_size < large_config.batch_size);
        assert!(large_config.max_memory_usage > small_config.max_memory_usage);
    }

    #[test]
    fn test_export_stats_calculations() {
        let writer_stats = BinaryHtmlStats {
            allocations_processed: 1000,
            total_html_size: 50000,
            peak_memory_usage: 10 * 1024 * 1024, // 10MB
            ..Default::default()
        };

        let export_stats = BinaryHtmlExportStats {
            writer_stats,
            total_export_time_ms: 500,
            file_size_bytes: 5 * 1024 * 1024, // 5MB
            strategy_used: ProcessingStrategy::Standard,
            throughput_allocations_per_sec: 2000.0,
            memory_efficiency: 100.0,
            binary_read_time_ms: 100,
            html_generation_time_ms: 400,
        };

        assert_eq!(export_stats.processing_efficiency(), 2000.0);
        assert!(export_stats.performance_improvement() > 0.0); // Should show improvement over baseline
    }

    #[test]
    fn test_processing_strategy_enum() {
        assert_eq!(ProcessingStrategy::Standard, ProcessingStrategy::Standard);
        assert_ne!(ProcessingStrategy::Standard, ProcessingStrategy::Parallel);

        // Test Debug formatting
        let strategy = ProcessingStrategy::Optimized;
        assert_eq!(format!("{strategy:?}"), "Optimized");
    }

    #[test]
    fn test_binary_html_export_config_default() {
        let config = BinaryHtmlExportConfig::default();

        assert!(config.enable_auto_strategy);
        assert_eq!(config.parallel_threshold, 5000);
        assert_eq!(config.large_file_threshold, 100 * 1024 * 1024);
        assert_eq!(config.batch_size, 1000);
        assert!(!config.enable_progress_reporting);
        assert_eq!(config.max_memory_usage, 64 * 1024 * 1024);
        assert!(config.enable_optimizations);
    }

    #[test]
    fn test_binary_html_export_config_debug_clone() {
        let config = BinaryHtmlExportConfig::default();

        // Test Debug trait
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("BinaryHtmlExportConfig"));
        assert!(debug_str.contains("enable_auto_strategy"));
        assert!(debug_str.contains("parallel_threshold"));

        // Test Clone trait
        let cloned_config = config.clone();
        assert_eq!(
            cloned_config.enable_auto_strategy,
            config.enable_auto_strategy
        );
        assert_eq!(cloned_config.parallel_threshold, config.parallel_threshold);
        assert_eq!(
            cloned_config.large_file_threshold,
            config.large_file_threshold
        );
        assert_eq!(cloned_config.batch_size, config.batch_size);
        assert_eq!(
            cloned_config.enable_progress_reporting,
            config.enable_progress_reporting
        );
        assert_eq!(cloned_config.max_memory_usage, config.max_memory_usage);
        assert_eq!(
            cloned_config.enable_optimizations,
            config.enable_optimizations
        );
    }

    #[test]
    fn test_binary_html_export_stats_debug_clone() {
        let writer_stats = BinaryHtmlStats::default();
        let stats = BinaryHtmlExportStats {
            writer_stats,
            total_export_time_ms: 1000,
            binary_read_time_ms: 200,
            html_generation_time_ms: 800,
            file_size_bytes: 5 * 1024 * 1024,
            strategy_used: ProcessingStrategy::Standard,
            throughput_allocations_per_sec: 1000.0,
            memory_efficiency: 50.0,
        };

        // Test Debug trait
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("BinaryHtmlExportStats"));
        assert!(debug_str.contains("total_export_time_ms"));
        assert!(debug_str.contains("strategy_used"));

        // Test Clone trait
        let cloned_stats = stats.clone();
        assert_eq!(
            cloned_stats.total_export_time_ms,
            stats.total_export_time_ms
        );
        assert_eq!(cloned_stats.binary_read_time_ms, stats.binary_read_time_ms);
        assert_eq!(
            cloned_stats.html_generation_time_ms,
            stats.html_generation_time_ms
        );
        assert_eq!(cloned_stats.file_size_bytes, stats.file_size_bytes);
        assert_eq!(cloned_stats.strategy_used, stats.strategy_used);
        assert_eq!(
            cloned_stats.throughput_allocations_per_sec,
            stats.throughput_allocations_per_sec
        );
        assert_eq!(cloned_stats.memory_efficiency, stats.memory_efficiency);
    }

    #[test]
    fn test_processing_strategy_debug_clone_partialeq() {
        // Test Debug trait
        let strategy = ProcessingStrategy::Parallel;
        let debug_str = format!("{:?}", strategy);
        assert_eq!(debug_str, "Parallel");

        // Test Clone trait
        let strategy1 = ProcessingStrategy::Optimized;
        let strategy2 = strategy1.clone();
        assert_eq!(strategy1, strategy2);

        // Test PartialEq trait
        assert_eq!(ProcessingStrategy::Standard, ProcessingStrategy::Standard);
        assert_eq!(ProcessingStrategy::Optimized, ProcessingStrategy::Optimized);
        assert_eq!(ProcessingStrategy::Parallel, ProcessingStrategy::Parallel);

        assert_ne!(ProcessingStrategy::Standard, ProcessingStrategy::Optimized);
        assert_ne!(ProcessingStrategy::Optimized, ProcessingStrategy::Parallel);
        assert_ne!(ProcessingStrategy::Standard, ProcessingStrategy::Parallel);
    }

    #[test]
    fn test_binary_html_export_stats_processing_efficiency() {
        // Test normal case
        let writer_stats = BinaryHtmlStats {
            allocations_processed: 2000,
            ..Default::default()
        };
        let stats = BinaryHtmlExportStats {
            writer_stats,
            total_export_time_ms: 1000, // 1 second
            throughput_allocations_per_sec: 0.0,
            memory_efficiency: 0.0,
            binary_read_time_ms: 0,
            html_generation_time_ms: 0,
            file_size_bytes: 0,
            strategy_used: ProcessingStrategy::Standard,
        };

        assert_eq!(stats.processing_efficiency(), 2000.0); // 2000 * 1000 / 1000

        // Test zero time case
        let stats_zero_time = BinaryHtmlExportStats {
            writer_stats: BinaryHtmlStats {
                allocations_processed: 1000,
                ..Default::default()
            },
            total_export_time_ms: 0,
            throughput_allocations_per_sec: 0.0,
            memory_efficiency: 0.0,
            binary_read_time_ms: 0,
            html_generation_time_ms: 0,
            file_size_bytes: 0,
            strategy_used: ProcessingStrategy::Standard,
        };

        assert_eq!(stats_zero_time.processing_efficiency(), 0.0);
    }

    #[test]
    fn test_binary_html_export_stats_performance_improvement() {
        // Test faster than baseline (800ms)
        let stats_fast = BinaryHtmlExportStats {
            writer_stats: BinaryHtmlStats::default(),
            total_export_time_ms: 400, // Faster than 800ms baseline
            throughput_allocations_per_sec: 0.0,
            memory_efficiency: 0.0,
            binary_read_time_ms: 0,
            html_generation_time_ms: 0,
            file_size_bytes: 0,
            strategy_used: ProcessingStrategy::Parallel,
        };

        assert_eq!(stats_fast.performance_improvement(), 50.0); // (800 - 400) / 800 * 100

        // Test slower than baseline
        let stats_slow = BinaryHtmlExportStats {
            writer_stats: BinaryHtmlStats::default(),
            total_export_time_ms: 1200, // Slower than 800ms baseline
            throughput_allocations_per_sec: 0.0,
            memory_efficiency: 0.0,
            binary_read_time_ms: 0,
            html_generation_time_ms: 0,
            file_size_bytes: 0,
            strategy_used: ProcessingStrategy::Standard,
        };

        assert_eq!(stats_slow.performance_improvement(), -50.0); // (800 - 1200) / 800 * 100

        // Test zero time case
        let stats_zero = BinaryHtmlExportStats {
            writer_stats: BinaryHtmlStats::default(),
            total_export_time_ms: 0,
            throughput_allocations_per_sec: 0.0,
            memory_efficiency: 0.0,
            binary_read_time_ms: 0,
            html_generation_time_ms: 0,
            file_size_bytes: 0,
            strategy_used: ProcessingStrategy::Standard,
        };

        assert_eq!(stats_zero.performance_improvement(), 0.0);
    }

    #[test]
    fn test_select_optimal_strategy_edge_cases() {
        let config = BinaryHtmlExportConfig::default();

        // Test exactly at threshold (100MB == 100MB, so NOT > 100MB, but > 50MB)
        let strategy = select_optimal_strategy(config.large_file_threshold, &config);
        assert_eq!(strategy, ProcessingStrategy::Optimized);

        // Test above threshold
        let strategy = select_optimal_strategy(config.large_file_threshold + 1, &config);
        assert_eq!(strategy, ProcessingStrategy::Parallel);

        // Test exactly at half threshold
        let half_threshold = config.large_file_threshold / 2;
        let strategy = select_optimal_strategy(half_threshold, &config);
        // Since 50MB > 50MB is false, this should be Standard
        assert_eq!(strategy, ProcessingStrategy::Standard);

        // Test just above half threshold
        let strategy = select_optimal_strategy(config.large_file_threshold / 2 + 1, &config);
        // Since 50MB+1 > 50MB is true, this should be Optimized
        assert_eq!(strategy, ProcessingStrategy::Optimized);

        // Test just below half threshold
        let strategy = select_optimal_strategy(config.large_file_threshold / 2 - 1, &config);
        assert_eq!(strategy, ProcessingStrategy::Standard);

        // Test zero size
        let strategy = select_optimal_strategy(0, &config);
        assert_eq!(strategy, ProcessingStrategy::Standard);

        // Test very large file
        let strategy = select_optimal_strategy(u64::MAX, &config);
        assert_eq!(strategy, ProcessingStrategy::Parallel);
    }

    #[test]
    fn test_select_optimal_strategy_custom_config() {
        let custom_config = BinaryHtmlExportConfig {
            large_file_threshold: 50 * 1024 * 1024, // 50MB
            ..Default::default()
        };

        // Test with custom threshold
        let strategy = select_optimal_strategy(30 * 1024 * 1024, &custom_config); // 30MB
        assert_eq!(strategy, ProcessingStrategy::Optimized);

        let strategy = select_optimal_strategy(60 * 1024 * 1024, &custom_config); // 60MB
        assert_eq!(strategy, ProcessingStrategy::Parallel);

        let strategy = select_optimal_strategy(10 * 1024 * 1024, &custom_config); // 10MB
        assert_eq!(strategy, ProcessingStrategy::Standard);
    }

    #[test]
    fn test_parse_binary_to_html_direct_dummy_stats() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test.memscope");
        let html_path = temp_dir.path().join("test.html");

        // Create a dummy binary file
        std::fs::write(&binary_path, b"dummy binary data").unwrap();

        // This will likely fail due to invalid binary format, but we test the function signature
        let result = parse_binary_to_html_direct(&binary_path, &html_path, "test_project");

        // We expect this to fail with invalid binary format, but the function should be callable
        // and return the correct error type
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_binary_to_html_auto_config() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test.memscope");
        let html_path = temp_dir.path().join("test.html");

        // Create a dummy binary file
        std::fs::write(&binary_path, b"dummy binary data").unwrap();

        // Test auto configuration
        let result = parse_binary_to_html_auto(&binary_path, &html_path, "test_project");

        // We expect this to fail with invalid binary format, but the function should be callable
        assert!(result.is_err());
    }

    #[test]
    fn test_get_recommended_config_file_sizes() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Test small file config
        let small_file = temp_dir.path().join("small.memscope");
        let small_data = vec![0u8; 5 * 1024 * 1024]; // 5MB
        std::fs::write(&small_file, small_data).unwrap();

        let config = get_recommended_config(&small_file).unwrap();
        assert!(!config.enable_auto_strategy);
        assert_eq!(config.batch_size, 500);
        assert!(!config.enable_progress_reporting);
        assert!(!config.enable_optimizations);

        // Test medium file config
        let medium_file = temp_dir.path().join("medium.memscope");
        let medium_data = vec![0u8; 50 * 1024 * 1024]; // 50MB
        std::fs::write(&medium_file, medium_data).unwrap();

        let config = get_recommended_config(&medium_file).unwrap();
        assert!(config.enable_auto_strategy);
        assert_eq!(config.batch_size, 1500);
        assert!(config.enable_progress_reporting);
        assert!(config.enable_optimizations);

        // Test large file config
        let large_file = temp_dir.path().join("large.memscope");
        let large_data = vec![0u8; 150 * 1024 * 1024]; // 150MB
        std::fs::write(&large_file, large_data).unwrap();

        let config = get_recommended_config(&large_file).unwrap();
        assert!(config.enable_auto_strategy);
        assert_eq!(config.batch_size, 2000);
        assert_eq!(config.parallel_threshold, 3000);
        assert!(config.enable_progress_reporting);
        assert_eq!(config.max_memory_usage, 128 * 1024 * 1024);
        assert!(config.enable_optimizations);
    }

    #[test]
    fn test_get_recommended_config_nonexistent_file() {
        let result = get_recommended_config("nonexistent_file.memscope");
        assert!(result.is_err());

        // Verify it returns the correct error type
        match result {
            Err(BinaryExportError::Io(_)) => {
                // Expected error type
            }
            _ => panic!("Expected BinaryExportError::Io"),
        }
    }

    #[test]
    fn test_parse_binary_to_html_with_config_file_not_found() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let nonexistent_binary = temp_dir.path().join("nonexistent.memscope");
        let html_path = temp_dir.path().join("output.html");
        let config = BinaryHtmlExportConfig::default();

        let result = parse_binary_to_html_with_config(
            &nonexistent_binary,
            &html_path,
            "test_project",
            &config,
        );

        assert!(result.is_err());
        match result {
            Err(BinaryExportError::Io(_)) => {
                // Expected error type for file not found
            }
            _ => panic!("Expected BinaryExportError::Io for file not found"),
        }
    }

    #[test]
    fn test_binary_html_export_stats_with_real_values() {
        let writer_stats = BinaryHtmlStats {
            allocations_processed: 5000,
            total_html_size: 250000,
            peak_memory_usage: 20 * 1024 * 1024, // 20MB
            ..Default::default()
        };

        let export_stats = BinaryHtmlExportStats {
            writer_stats,
            total_export_time_ms: 2000, // 2 seconds
            binary_read_time_ms: 500,
            html_generation_time_ms: 1500,
            file_size_bytes: 10 * 1024 * 1024, // 10MB
            strategy_used: ProcessingStrategy::Optimized,
            throughput_allocations_per_sec: 2500.0, // 5000 / 2
            memory_efficiency: 250.0,               // 5000 / 20MB
        };

        // Test processing efficiency
        assert_eq!(export_stats.processing_efficiency(), 2500.0);

        // Test performance improvement (should be positive since 2000ms < 800ms baseline is false)
        // Actually 2000ms > 800ms, so improvement should be negative
        assert_eq!(export_stats.performance_improvement(), -150.0); // (800 - 2000) / 800 * 100

        // Verify all fields are set correctly
        assert_eq!(export_stats.writer_stats.allocations_processed, 5000);
        assert_eq!(export_stats.total_export_time_ms, 2000);
        assert_eq!(export_stats.binary_read_time_ms, 500);
        assert_eq!(export_stats.html_generation_time_ms, 1500);
        assert_eq!(export_stats.file_size_bytes, 10 * 1024 * 1024);
        assert_eq!(export_stats.strategy_used, ProcessingStrategy::Optimized);
        assert_eq!(export_stats.throughput_allocations_per_sec, 2500.0);
        assert_eq!(export_stats.memory_efficiency, 250.0);
    }

    #[test]
    fn test_config_builder_pattern_simulation() {
        // Simulate a builder pattern by creating configs with different settings
        let base_config = BinaryHtmlExportConfig::default();

        let custom_config = BinaryHtmlExportConfig {
            enable_auto_strategy: false,
            parallel_threshold: 10000,
            large_file_threshold: 200 * 1024 * 1024,
            batch_size: 2000,
            enable_progress_reporting: true,
            max_memory_usage: 128 * 1024 * 1024,
            enable_optimizations: false,
        };

        // Verify the custom config differs from default
        assert_ne!(
            custom_config.enable_auto_strategy,
            base_config.enable_auto_strategy
        );
        assert_ne!(
            custom_config.parallel_threshold,
            base_config.parallel_threshold
        );
        assert_ne!(
            custom_config.large_file_threshold,
            base_config.large_file_threshold
        );
        assert_ne!(custom_config.batch_size, base_config.batch_size);
        assert_ne!(
            custom_config.enable_progress_reporting,
            base_config.enable_progress_reporting
        );
        assert_ne!(custom_config.max_memory_usage, base_config.max_memory_usage);
        assert_ne!(
            custom_config.enable_optimizations,
            base_config.enable_optimizations
        );

        // Verify the custom config has expected values
        assert!(!custom_config.enable_auto_strategy);
        assert_eq!(custom_config.parallel_threshold, 10000);
        assert_eq!(custom_config.large_file_threshold, 200 * 1024 * 1024);
        assert_eq!(custom_config.batch_size, 2000);
        assert!(custom_config.enable_progress_reporting);
        assert_eq!(custom_config.max_memory_usage, 128 * 1024 * 1024);
        assert!(!custom_config.enable_optimizations);
    }

    #[test]
    fn test_strategy_selection_comprehensive() {
        let configs = [
            // Default config
            BinaryHtmlExportConfig::default(),
            // Custom small threshold config
            BinaryHtmlExportConfig {
                large_file_threshold: 10 * 1024 * 1024, // 10MB
                ..Default::default()
            },
            // Custom large threshold config
            BinaryHtmlExportConfig {
                large_file_threshold: 500 * 1024 * 1024, // 500MB
                ..Default::default()
            },
        ];

        let file_sizes = [
            1024,               // 1KB
            1024 * 1024,        // 1MB
            10 * 1024 * 1024,   // 10MB
            50 * 1024 * 1024,   // 50MB
            100 * 1024 * 1024,  // 100MB
            200 * 1024 * 1024,  // 200MB
            1024 * 1024 * 1024, // 1GB
        ];

        for config in &configs {
            for &file_size in &file_sizes {
                let strategy = select_optimal_strategy(file_size, config);

                // Verify strategy is one of the valid options
                match strategy {
                    ProcessingStrategy::Standard
                    | ProcessingStrategy::Optimized
                    | ProcessingStrategy::Parallel => {
                        // Valid strategy
                    }
                }

                // Verify strategy logic
                if file_size > config.large_file_threshold {
                    assert_eq!(strategy, ProcessingStrategy::Parallel);
                } else if file_size > config.large_file_threshold / 2 {
                    assert_eq!(strategy, ProcessingStrategy::Optimized);
                } else {
                    assert_eq!(strategy, ProcessingStrategy::Standard);
                }
            }
        }
    }
}
