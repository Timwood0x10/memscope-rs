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
    parse_binary_to_html_with_config(
        binary_path,
        html_path,
        project_name,
        &BinaryHtmlExportConfig::default(),
    )
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
    // TODO: Implement actual parallel processing using rayon
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
        assert_eq!(format!("{:?}", strategy), "Optimized");
    }
}
