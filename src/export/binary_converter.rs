//! Binary converter for transforming binary format to JSON/HTML
//!
//! This module implements conversion functions that read binary format memory tracking data
//! and convert it to JSON or HTML formats, maintaining compatibility with existing tools.

use crate::core::types::{AllocationInfo, MemoryStats, TrackingResult, TypeMemoryUsage};
use crate::export::binary_parser::{BinaryParser, BinaryParserOptions};
use crate::export::conversion_validator::{ConversionValidator, ValidationOptions, ValidationResult};
use crate::export::html_export::export_interactive_html;
use crate::export::optimized_json_export::OptimizedExportOptions;
use rayon::prelude::*;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Conversion options for binary to other formats
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// Parser options for reading binary data
    pub parser_options: BinaryParserOptions,
    /// JSON export options when converting to JSON
    pub json_options: OptimizedExportOptions,
    /// Enable progress reporting during conversion
    pub enable_progress_reporting: bool,
    /// Buffer size for output operations
    pub output_buffer_size: usize,
    /// Validate output against original data
    pub validate_output: bool,
    /// Include metadata in converted output
    pub include_metadata: bool,
    /// Validation options for comprehensive validation
    pub validation_options: Option<ValidationOptions>,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            parser_options: BinaryParserOptions::default(),
            json_options: OptimizedExportOptions::default(),
            enable_progress_reporting: false,
            output_buffer_size: 256 * 1024, // 256KB
            validate_output: false,
            include_metadata: true,
            validation_options: None,
        }
    }
}

impl ConversionOptions {
    /// Create fast conversion options (minimal validation, maximum speed)
    pub fn fast() -> Self {
        Self {
            parser_options: BinaryParserOptions::fast(),
            json_options: OptimizedExportOptions::fast(),
            enable_progress_reporting: false,
            output_buffer_size: 512 * 1024, // Larger buffer for speed
            validate_output: false,
            include_metadata: false,
            validation_options: None,
        }
    }

    /// Create comprehensive conversion options (full validation, all features)
    pub fn comprehensive() -> Self {
        Self {
            parser_options: BinaryParserOptions::strict(),
            json_options: OptimizedExportOptions::comprehensive(),
            enable_progress_reporting: true,
            output_buffer_size: 256 * 1024,
            validate_output: true,
            include_metadata: true,
            validation_options: Some(ValidationOptions::default()),
        }
    }

    /// Create production conversion options (balanced performance and reliability)
    pub fn production() -> Self {
        Self {
            parser_options: BinaryParserOptions::default(),
            json_options: OptimizedExportOptions::production(),
            enable_progress_reporting: true,
            output_buffer_size: 256 * 1024,
            validate_output: false, // Skip for performance in production
            include_metadata: true,
            validation_options: None,
        }
    }
}

/// Conversion result with statistics
#[derive(Debug, Clone)]
pub struct ConversionResult {
    /// Path to the converted file
    pub output_path: String,
    /// Original binary file size
    pub input_size: usize,
    /// Converted file size
    pub output_size: usize,
    /// Conversion duration
    pub conversion_duration: std::time::Duration,
    /// Number of allocations converted
    pub allocations_converted: usize,
    /// Whether validation was performed and passed
    pub validation_passed: Option<bool>,
    /// Detailed validation result (if comprehensive validation was performed)
    pub validation_result: Option<ValidationResult>,
    /// Conversion format
    pub output_format: String,
}

impl ConversionResult {
    /// Calculate size ratio (output/input)
    pub fn size_ratio(&self) -> f64 {
        if self.input_size > 0 {
            self.output_size as f64 / self.input_size as f64
        } else {
            0.0
        }
    }

    /// Calculate conversion speed in MB/s
    pub fn conversion_speed_mbps(&self) -> f64 {
        let seconds = self.conversion_duration.as_secs_f64();
        if seconds > 0.0 {
            (self.input_size as f64) / (1024.0 * 1024.0) / seconds
        } else {
            0.0
        }
    }
}

/// Error information for failed conversions
#[derive(Debug, Clone)]
pub struct ConversionError {
    /// Path to the file that failed to convert
    pub file_path: String,
    /// Error message
    pub error_message: String,
    /// Error timestamp
    pub timestamp: std::time::SystemTime,
    /// Error category
    pub error_category: ErrorCategory,
}

/// Categories of conversion errors
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// File I/O errors (file not found, permission denied, etc.)
    IoError,
    /// Binary format parsing errors
    ParseError,
    /// Output generation errors
    OutputError,
    /// Validation errors
    ValidationError,
    /// Unknown/other errors
    Unknown,
}

/// Progress information for batch conversions
#[derive(Debug, Clone)]
pub struct ConversionProgress {
    /// Total number of files to convert
    pub total_files: usize,
    /// Number of files completed (successful + failed)
    pub completed_files: usize,
    /// Number of successful conversions
    pub successful_conversions: usize,
    /// Number of failed conversions
    pub failed_conversions: usize,
    /// Current file being processed
    pub current_file: Option<String>,
    /// Elapsed time since start
    pub elapsed_time: Duration,
    /// Estimated time remaining
    pub estimated_remaining: Option<Duration>,
    /// Current conversion speed (files/second)
    pub conversion_rate: f64,
    /// Total bytes processed
    pub bytes_processed: usize,
    /// Processing speed (MB/s)
    pub processing_speed_mbps: f64,
}

impl ConversionProgress {
    /// Calculate completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.total_files > 0 {
            (self.completed_files as f64 / self.total_files as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.completed_files > 0 {
            (self.successful_conversions as f64 / self.completed_files as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Check if conversion is complete
    pub fn is_complete(&self) -> bool {
        self.completed_files >= self.total_files
    }
}

/// Batch conversion report with detailed statistics and error information
#[derive(Debug, Clone)]
pub struct BatchConversionReport {
    /// Individual conversion results
    pub results: Vec<ConversionResult>,
    /// Conversion errors
    pub errors: Vec<ConversionError>,
    /// Total conversion time
    pub total_duration: Duration,
    /// Number of successful conversions
    pub successful_conversions: usize,
    /// Number of failed conversions
    pub failed_conversions: usize,
    /// Total input size
    pub total_input_size: usize,
    /// Total output size
    pub total_output_size: usize,
    /// Conversion was cancelled
    pub was_cancelled: bool,
    /// Peak memory usage during conversion (if available)
    pub peak_memory_usage: Option<usize>,
    /// Number of parallel workers used
    pub parallel_workers: usize,
    /// Average conversion speed per file
    pub average_speed_per_file: f64,
    /// Files processed per second
    pub files_per_second: f64,
}

impl BatchConversionReport {
    /// Calculate overall success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.successful_conversions + self.failed_conversions;
        if total > 0 {
            self.successful_conversions as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Calculate overall size ratio
    pub fn overall_size_ratio(&self) -> f64 {
        if self.total_input_size > 0 {
            self.total_output_size as f64 / self.total_input_size as f64
        } else {
            0.0
        }
    }

    /// Calculate overall conversion speed
    pub fn overall_speed_mbps(&self) -> f64 {
        let seconds = self.total_duration.as_secs_f64();
        if seconds > 0.0 {
            (self.total_input_size as f64) / (1024.0 * 1024.0) / seconds
        } else {
            0.0
        }
    }

    /// Get errors by category
    pub fn errors_by_category(&self) -> HashMap<ErrorCategory, Vec<&ConversionError>> {
        let mut categorized = HashMap::new();
        for error in &self.errors {
            categorized
                .entry(error.error_category.clone())
                .or_insert_with(Vec::new)
                .push(error);
        }
        categorized
    }

    /// Generate a summary report as a formatted string
    pub fn generate_summary(&self) -> String {
        let total_files = self.successful_conversions + self.failed_conversions;
        let success_rate = self.success_rate() * 100.0;
        let size_ratio = self.overall_size_ratio();
        let speed_mbps = self.overall_speed_mbps();

        let mut summary = format!(
            "Batch Conversion Report\n\
             ======================\n\
             Total Files: {}\n\
             Successful: {} ({:.1}%)\n\
             Failed: {}\n\
             Total Duration: {:.2}s\n\
             Size Ratio: {:.2}x\n\
             Speed: {:.2} MB/s\n\
             Files/sec: {:.2}\n",
            total_files,
            self.successful_conversions,
            success_rate,
            self.failed_conversions,
            self.total_duration.as_secs_f64(),
            size_ratio,
            speed_mbps,
            self.files_per_second
        );

        if self.was_cancelled {
            summary.push_str("Status: CANCELLED\n");
        }

        if !self.errors.is_empty() {
            summary.push_str("\nErrors by Category:\n");
            let errors_by_cat = self.errors_by_category();
            for (category, errors) in errors_by_cat {
                summary.push_str(&format!("  {:?}: {} errors\n", category, errors.len()));
            }
        }

        summary
    }
}

/// Progress callback function type
pub type ProgressCallback = Arc<dyn Fn(&ConversionProgress) + Send + Sync>;

/// Cancellation token for batch operations
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Create a new cancellation token
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Cancel the operation
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Check if the operation is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for batch conversion operations
#[derive(Clone)]
pub struct BatchConversionOptions {
    /// Base conversion options
    pub conversion_options: ConversionOptions,
    /// Enable parallel processing
    pub enable_parallel: bool,
    /// Number of parallel workers (None = auto-detect)
    pub parallel_workers: Option<usize>,
    /// Progress callback function
    pub progress_callback: Option<ProgressCallback>,
    /// Cancellation token
    pub cancellation_token: Option<CancellationToken>,
    /// Continue on error (don't stop batch on first error)
    pub continue_on_error: bool,
    /// Maximum memory usage limit (bytes)
    pub memory_limit: Option<usize>,
    /// Progress update interval (in processed files)
    pub progress_update_interval: usize,
}

impl Default for BatchConversionOptions {
    fn default() -> Self {
        Self {
            conversion_options: ConversionOptions::default(),
            enable_parallel: true,
            parallel_workers: None, // Auto-detect
            progress_callback: None,
            cancellation_token: None,
            continue_on_error: true,
            memory_limit: None,
            progress_update_interval: 1,
        }
    }
}

impl BatchConversionOptions {
    /// Create fast batch conversion options
    pub fn fast() -> Self {
        Self {
            conversion_options: ConversionOptions::fast(),
            enable_parallel: true,
            parallel_workers: None,
            progress_callback: None,
            cancellation_token: None,
            continue_on_error: true,
            memory_limit: None,
            progress_update_interval: 5, // Less frequent updates for speed
        }
    }

    /// Create comprehensive batch conversion options
    pub fn comprehensive() -> Self {
        Self {
            conversion_options: ConversionOptions::comprehensive(),
            enable_parallel: true,
            parallel_workers: None,
            progress_callback: None,
            cancellation_token: None,
            continue_on_error: false, // Stop on errors for comprehensive mode
            memory_limit: Some(2 * 1024 * 1024 * 1024), // 2GB limit
            progress_update_interval: 1,
        }
    }

    /// Set progress callback
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&ConversionProgress) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Arc::new(callback));
        self
    }

    /// Set cancellation token
    pub fn with_cancellation_token(mut self, token: CancellationToken) -> Self {
        self.cancellation_token = Some(token);
        self
    }

    /// Set parallel workers count
    pub fn with_parallel_workers(mut self, workers: usize) -> Self {
        self.parallel_workers = Some(workers);
        self
    }
}

impl ConversionOptions {
    /// Enable comprehensive validation with default options
    pub fn with_comprehensive_validation(mut self) -> Self {
        self.validate_output = true;
        self.validation_options = Some(ValidationOptions::default());
        self
    }

    /// Enable comprehensive validation with custom options
    pub fn with_validation_options(mut self, validation_options: ValidationOptions) -> Self {
        self.validate_output = true;
        self.validation_options = Some(validation_options);
        self
    }

    /// Enable performance comparison in validation
    pub fn with_performance_comparison(mut self) -> Self {
        if let Some(ref mut validation_options) = self.validation_options {
            validation_options.enable_performance_comparison = true;
        } else {
            let mut validation_options = ValidationOptions::default();
            validation_options.enable_performance_comparison = true;
            self.validation_options = Some(validation_options);
            self.validate_output = true;
        }
        self
    }
}

/// Main binary converter struct
pub struct BinaryConverter;

impl BinaryConverter {
    /// Convert binary file to JSON format
    pub fn binary_to_json<P: AsRef<Path>>(
        binary_path: P,
        json_path: P,
    ) -> TrackingResult<ConversionResult> {
        Self::binary_to_json_with_options(
            binary_path,
            json_path,
            ConversionOptions::default(),
        )
    }

    /// Convert binary file to JSON format with custom options
    pub fn binary_to_json_with_options<P: AsRef<Path>>(
        binary_path: P,
        json_path: P,
        options: ConversionOptions,
    ) -> TrackingResult<ConversionResult> {
        let start_time = std::time::Instant::now();
        let binary_path = binary_path.as_ref();
        let json_path = json_path.as_ref();

        tracing::info!(
            "Converting binary file {} to JSON {}",
            binary_path.display(),
            json_path.display()
        );

        // Get input file size
        let input_size = std::fs::metadata(binary_path)
            .map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to read binary file metadata: {e}"
                ))
            })?
            .len() as usize;

        // Parse binary file
        let mut parser = BinaryParser::with_options(options.parser_options.clone());
        
        // Add progress callback if enabled
        if options.enable_progress_reporting {
            parser = parser.with_progress_callback(|progress| {
                tracing::info!(
                    "Parsing progress: {:.1}% ({} sections parsed)",
                    progress.completion_percentage(),
                    progress.sections_parsed
                );
            });
        }

        let _parse_result = parser.load_from_file(binary_path)?;

        // Extract data from parser
        let allocations = parser.load_allocations()?;
        let stats = parser.load_memory_stats()?;
        let memory_by_type = parser.load_type_memory_usage()?;

        tracing::info!(
            "Loaded {} allocations, {} types from binary file",
            allocations.len(),
            memory_by_type.len()
        );

        // Convert to JSON format
        let json_data = Self::create_json_data(&allocations, &stats, &memory_by_type, &options)?;

        // Write JSON file
        let output_file = File::create(json_path).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to create JSON file: {e}"
            ))
        })?;

        let mut writer = BufWriter::with_capacity(options.output_buffer_size, output_file);
        writer.write_all(json_data.as_bytes()).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to write JSON data: {e}"
            ))
        })?;
        writer.flush().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to flush JSON data: {e}"
            ))
        })?;

        // Get output file size
        let output_size = std::fs::metadata(json_path)
            .map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to read JSON file metadata: {e}"
                ))
            })?
            .len() as usize;

        let conversion_duration = start_time.elapsed();

        // Perform validation if requested
        let (validation_passed, validation_result) = if options.validate_output {
            if let Some(ref validation_options) = options.validation_options {
                // Perform comprehensive validation
                let validator = ConversionValidator::with_options(validation_options.clone());
                let temp_result = ConversionResult {
                    output_path: json_path.to_string_lossy().to_string(),
                    input_size,
                    output_size,
                    conversion_duration,
                    allocations_converted: allocations.len(),
                    validation_passed: None,
                    validation_result: None,
                    output_format: "JSON".to_string(),
                };
                
                match validator.validate_conversion(binary_path, json_path, &temp_result) {
                    Ok(validation_result) => {
                        (Some(validation_result.is_valid), Some(validation_result))
                    }
                    Err(e) => {
                        tracing::warn!("Comprehensive validation failed: {}", e);
                        // Fall back to basic validation
                        (Some(Self::validate_json_output(&json_data, &allocations)?), None)
                    }
                }
            } else {
                // Basic validation only
                (Some(Self::validate_json_output(&json_data, &allocations)?), None)
            }
        } else {
            (None, None)
        };

        let result = ConversionResult {
            output_path: json_path.to_string_lossy().to_string(),
            input_size,
            output_size,
            conversion_duration,
            allocations_converted: allocations.len(),
            validation_passed,
            validation_result,
            output_format: "JSON".to_string(),
        };

        tracing::info!(
            "Successfully converted binary to JSON: {} -> {} ({:.1}x size ratio, {:.1} MB/s)",
            binary_path.display(),
            json_path.display(),
            result.size_ratio(),
            result.conversion_speed_mbps()
        );

        Ok(result)
    }

    /// Convert binary file to HTML format
    pub fn binary_to_html<P: AsRef<Path>>(
        binary_path: P,
        html_path: P,
    ) -> TrackingResult<ConversionResult> {
        Self::binary_to_html_with_options(
            binary_path,
            html_path,
            ConversionOptions::default(),
        )
    }

    /// Convert binary file to HTML format with custom options
    pub fn binary_to_html_with_options<P: AsRef<Path>>(
        binary_path: P,
        html_path: P,
        options: ConversionOptions,
    ) -> TrackingResult<ConversionResult> {
        let start_time = std::time::Instant::now();
        let binary_path = binary_path.as_ref();
        let html_path = html_path.as_ref();

        tracing::info!(
            "Converting binary file {} to HTML {}",
            binary_path.display(),
            html_path.display()
        );

        // Get input file size
        let input_size = std::fs::metadata(binary_path)
            .map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to read binary file metadata: {e}"
                ))
            })?
            .len() as usize;

        // Parse binary file
        let mut parser = BinaryParser::with_options(options.parser_options.clone());
        
        if options.enable_progress_reporting {
            parser = parser.with_progress_callback(|progress| {
                tracing::info!(
                    "Parsing progress: {:.1}% ({} sections parsed)",
                    progress.completion_percentage(),
                    progress.sections_parsed
                );
            });
        }

        let _parse_result = parser.load_from_file(binary_path)?;

        // Create a temporary MemoryTracker with the parsed data
        let temp_tracker = Self::create_memory_tracker_from_parser(&parser)?;

        // Use existing HTML export functionality
        export_interactive_html(&temp_tracker, None, html_path)?;

        // Get output file size
        let output_size = std::fs::metadata(html_path)
            .map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to read HTML file metadata: {e}"
                ))
            })?
            .len() as usize;

        let conversion_duration = start_time.elapsed();
        let allocations = parser.load_allocations()?;

        // Perform validation if requested (limited for HTML)
        let (validation_passed, validation_result) = if options.validate_output {
            if let Some(ref validation_options) = options.validation_options {
                // Perform comprehensive validation for HTML
                let validator = ConversionValidator::with_options(validation_options.clone());
                let temp_result = ConversionResult {
                    output_path: html_path.to_string_lossy().to_string(),
                    input_size,
                    output_size,
                    conversion_duration,
                    allocations_converted: allocations.len(),
                    validation_passed: None,
                    validation_result: None,
                    output_format: "HTML".to_string(),
                };
                
                match validator.validate_conversion(binary_path, html_path, &temp_result) {
                    Ok(validation_result) => {
                        (Some(validation_result.is_valid), Some(validation_result))
                    }
                    Err(e) => {
                        tracing::warn!("HTML validation failed: {}", e);
                        (Some(false), None)
                    }
                }
            } else {
                // Basic HTML validation (just check if file exists and has content)
                let has_content = std::fs::metadata(html_path)
                    .map(|m| m.len() > 0)
                    .unwrap_or(false);
                (Some(has_content), None)
            }
        } else {
            (None, None)
        };

        let result = ConversionResult {
            output_path: html_path.to_string_lossy().to_string(),
            input_size,
            output_size,
            conversion_duration,
            allocations_converted: allocations.len(),
            validation_passed,
            validation_result,
            output_format: "HTML".to_string(),
        };

        tracing::info!(
            "Successfully converted binary to HTML: {} -> {} ({:.1}x size ratio, {:.1} MB/s)",
            binary_path.display(),
            html_path.display(),
            result.size_ratio(),
            result.conversion_speed_mbps()
        );

        Ok(result)
    }

    /// Batch convert multiple binary files
    pub fn batch_convert<P: AsRef<Path>>(
        input_dir: P,
        output_dir: P,
        output_format: OutputFormat,
    ) -> TrackingResult<BatchConversionReport> {
        Self::batch_convert_with_options(
            input_dir,
            output_dir,
            output_format,
            BatchConversionOptions::default(),
        )
    }

    /// Batch convert multiple binary files with custom options
    pub fn batch_convert_with_options<P: AsRef<Path>>(
        input_dir: P,
        output_dir: P,
        output_format: OutputFormat,
        options: BatchConversionOptions,
    ) -> TrackingResult<BatchConversionReport> {
        let start_time = Instant::now();
        let input_dir = input_dir.as_ref();
        let output_dir = output_dir.as_ref();

        tracing::info!(
            "Starting batch conversion from {} to {} (format: {:?}, parallel: {})",
            input_dir.display(),
            output_dir.display(),
            output_format,
            options.enable_parallel
        );

        // Create output directory if it doesn't exist
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir).map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to create output directory: {e}"
                ))
            })?;
        }

        // Find all binary files in input directory
        let binary_files = Self::find_binary_files(input_dir)?;
        
        if binary_files.is_empty() {
            tracing::warn!("No binary files found in {}", input_dir.display());
            return Ok(BatchConversionReport {
                results: Vec::new(),
                errors: Vec::new(),
                total_duration: start_time.elapsed(),
                successful_conversions: 0,
                failed_conversions: 0,
                total_input_size: 0,
                total_output_size: 0,
                was_cancelled: false,
                peak_memory_usage: None,
                parallel_workers: 1,
                average_speed_per_file: 0.0,
                files_per_second: 0.0,
            });
        }

        tracing::info!("Found {} binary files to convert", binary_files.len());

        // Set up parallel processing
        let num_workers = if options.enable_parallel {
            options.parallel_workers.unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(|p| p.get())
                    .unwrap_or(4)
                    .min(binary_files.len())
                    .max(1)
            })
        } else {
            1
        };

        tracing::info!("Using {} parallel workers", num_workers);

        // Set up progress tracking
        let progress_data = Arc::new(Mutex::new(ConversionProgress {
            total_files: binary_files.len(),
            completed_files: 0,
            successful_conversions: 0,
            failed_conversions: 0,
            current_file: None,
            elapsed_time: Duration::from_secs(0),
            estimated_remaining: None,
            conversion_rate: 0.0,
            bytes_processed: 0,
            processing_speed_mbps: 0.0,
        }));

        let results = Arc::new(Mutex::new(Vec::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));
        let completed_count = Arc::new(AtomicUsize::new(0));
        let success_count = Arc::new(AtomicUsize::new(0));
        let total_input_size = Arc::new(AtomicUsize::new(0));
        let total_output_size = Arc::new(AtomicUsize::new(0));
        let was_cancelled = Arc::new(AtomicBool::new(false));

        // Configure thread pool if using parallel processing
        if options.enable_parallel && num_workers > 1 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(num_workers)
                .build_global()
                .map_err(|e| {
                    crate::core::types::TrackingError::ExportError(format!(
                        "Failed to configure thread pool: {e}"
                    ))
                })?;
        }

        // Process files (parallel or sequential based on options)
        let processing_result = if options.enable_parallel && num_workers > 1 {
            Self::process_files_parallel(
                &binary_files,
                output_dir,
                output_format,
                &options,
                start_time,
                &progress_data,
                &results,
                &errors,
                &completed_count,
                &success_count,
                &total_input_size,
                &total_output_size,
                &was_cancelled,
            )
        } else {
            Self::process_files_sequential(
                &binary_files,
                output_dir,
                output_format,
                &options,
                start_time,
                &progress_data,
                &results,
                &errors,
                &completed_count,
                &success_count,
                &total_input_size,
                &total_output_size,
                &was_cancelled,
            )
        };

        // Handle processing result
        if let Err(e) = processing_result {
            tracing::error!("Batch conversion failed: {}", e);
            return Err(e);
        }

        let total_duration = start_time.elapsed();
        let final_results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
        let final_errors = Arc::try_unwrap(errors).unwrap().into_inner().unwrap();
        let successful_conversions = success_count.load(Ordering::Relaxed);
        let failed_conversions = final_errors.len();
        let final_input_size = total_input_size.load(Ordering::Relaxed);
        let final_output_size = total_output_size.load(Ordering::Relaxed);
        let cancelled = was_cancelled.load(Ordering::Relaxed);

        let files_per_second = if total_duration.as_secs_f64() > 0.0 {
            (successful_conversions + failed_conversions) as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        let average_speed_per_file = if successful_conversions > 0 {
            final_input_size as f64 / successful_conversions as f64 / (1024.0 * 1024.0)
        } else {
            0.0
        };

        let report = BatchConversionReport {
            results: final_results,
            errors: final_errors,
            total_duration,
            successful_conversions,
            failed_conversions,
            total_input_size: final_input_size,
            total_output_size: final_output_size,
            was_cancelled: cancelled,
            peak_memory_usage: None, // TODO: Implement memory tracking
            parallel_workers: num_workers,
            average_speed_per_file,
            files_per_second,
        };

        let status = if cancelled { "CANCELLED" } else { "COMPLETED" };
        tracing::info!(
            "Batch conversion {}: {}/{} successful ({:.1}% success rate, {:.1} MB/s overall, {:.2} files/s)",
            status,
            report.successful_conversions,
            report.successful_conversions + report.failed_conversions,
            report.success_rate() * 100.0,
            report.overall_speed_mbps(),
            report.files_per_second
        );

        Ok(report)
    }

    // ============================================================================
    // Private Helper Methods
    // ============================================================================

    /// Process files in parallel
    fn process_files_parallel(
        binary_files: &[std::path::PathBuf],
        output_dir: &Path,
        output_format: OutputFormat,
        options: &BatchConversionOptions,
        start_time: Instant,
        progress_data: &Arc<Mutex<ConversionProgress>>,
        results: &Arc<Mutex<Vec<ConversionResult>>>,
        errors: &Arc<Mutex<Vec<ConversionError>>>,
        completed_count: &Arc<AtomicUsize>,
        success_count: &Arc<AtomicUsize>,
        total_input_size: &Arc<AtomicUsize>,
        total_output_size: &Arc<AtomicUsize>,
        was_cancelled: &Arc<AtomicBool>,
    ) -> TrackingResult<()> {
        tracing::info!("Starting parallel conversion of {} files", binary_files.len());

        binary_files
            .par_iter()
            .enumerate()
            .try_for_each(|(_index, binary_file)| -> TrackingResult<()> {
                // Check for cancellation
                if let Some(ref token) = options.cancellation_token {
                    if token.is_cancelled() {
                        was_cancelled.store(true, Ordering::Relaxed);
                        return Ok(()); // Exit gracefully
                    }
                }

                // Update current file in progress
                if let Some(ref _callback) = options.progress_callback {
                    if let Ok(mut progress) = progress_data.lock() {
                        progress.current_file = Some(
                            binary_file
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string(),
                        );
                        progress.elapsed_time = start_time.elapsed();
                        
                        // Calculate estimated remaining time
                        let completed = completed_count.load(Ordering::Relaxed);
                        if completed > 0 {
                            let rate = completed as f64 / progress.elapsed_time.as_secs_f64();
                            let remaining_files = binary_files.len() - completed;
                            progress.estimated_remaining = Some(Duration::from_secs_f64(
                                remaining_files as f64 / rate,
                            ));
                            progress.conversion_rate = rate;
                        }
                    }
                }

                let result = Self::convert_single_file(
                    binary_file,
                    output_dir,
                    output_format,
                    &options.conversion_options,
                );

                // Update counters and collections
                let completed = completed_count.fetch_add(1, Ordering::Relaxed) + 1;

                match result {
                    Ok(conversion_result) => {
                        success_count.fetch_add(1, Ordering::Relaxed);
                        total_input_size.fetch_add(conversion_result.input_size, Ordering::Relaxed);
                        total_output_size.fetch_add(conversion_result.output_size, Ordering::Relaxed);
                        
                        if let Ok(mut results_guard) = results.lock() {
                            results_guard.push(conversion_result);
                        }

                        tracing::debug!(
                            "Successfully converted {} ({}/{})",
                            binary_file.display(),
                            completed,
                            binary_files.len()
                        );
                    }
                    Err(e) => {
                        let error = ConversionError {
                            file_path: binary_file.to_string_lossy().to_string(),
                            error_message: e.to_string(),
                            timestamp: std::time::SystemTime::now(),
                            error_category: Self::categorize_error(&e),
                        };

                        if let Ok(mut errors_guard) = errors.lock() {
                            errors_guard.push(error);
                        }

                        tracing::error!(
                            "Failed to convert {} ({}/{}): {}",
                            binary_file.display(),
                            completed,
                            binary_files.len(),
                            e
                        );

                        if !options.continue_on_error {
                            return Err(e);
                        }
                    }
                }

                // Update progress and call callback
                if completed % options.progress_update_interval == 0 {
                    if let Some(ref callback) = options.progress_callback {
                        if let Ok(mut progress) = progress_data.lock() {
                            progress.completed_files = completed;
                            progress.successful_conversions = success_count.load(Ordering::Relaxed);
                            progress.failed_conversions = completed - progress.successful_conversions;
                            progress.elapsed_time = start_time.elapsed();
                            progress.bytes_processed = total_input_size.load(Ordering::Relaxed);
                            
                            if progress.elapsed_time.as_secs_f64() > 0.0 {
                                progress.processing_speed_mbps = progress.bytes_processed as f64
                                    / (1024.0 * 1024.0)
                                    / progress.elapsed_time.as_secs_f64();
                            }

                            callback(&progress);
                        }
                    }
                }

                Ok(())
            })?;

        Ok(())
    }

    /// Process files sequentially
    fn process_files_sequential(
        binary_files: &[std::path::PathBuf],
        output_dir: &Path,
        output_format: OutputFormat,
        options: &BatchConversionOptions,
        start_time: Instant,
        progress_data: &Arc<Mutex<ConversionProgress>>,
        results: &Arc<Mutex<Vec<ConversionResult>>>,
        errors: &Arc<Mutex<Vec<ConversionError>>>,
        completed_count: &Arc<AtomicUsize>,
        success_count: &Arc<AtomicUsize>,
        total_input_size: &Arc<AtomicUsize>,
        total_output_size: &Arc<AtomicUsize>,
        was_cancelled: &Arc<AtomicBool>,
    ) -> TrackingResult<()> {
        tracing::info!("Starting sequential conversion of {} files", binary_files.len());

        for (index, binary_file) in binary_files.iter().enumerate() {
            // Check for cancellation
            if let Some(ref token) = options.cancellation_token {
                if token.is_cancelled() {
                    was_cancelled.store(true, Ordering::Relaxed);
                    break;
                }
            }

            // Update current file in progress
            if let Some(ref _callback) = options.progress_callback {
                if let Ok(mut progress) = progress_data.lock() {
                    progress.current_file = Some(
                        binary_file
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                    );
                    progress.elapsed_time = start_time.elapsed();
                    
                    // Calculate estimated remaining time
                    if index > 0 {
                        let rate = index as f64 / progress.elapsed_time.as_secs_f64();
                        let remaining_files = binary_files.len() - index;
                        progress.estimated_remaining = Some(Duration::from_secs_f64(
                            remaining_files as f64 / rate,
                        ));
                        progress.conversion_rate = rate;
                    }
                }
            }

            let result = Self::convert_single_file(
                binary_file,
                output_dir,
                output_format,
                &options.conversion_options,
            );

            let completed = completed_count.fetch_add(1, Ordering::Relaxed) + 1;

            match result {
                Ok(conversion_result) => {
                    success_count.fetch_add(1, Ordering::Relaxed);
                    total_input_size.fetch_add(conversion_result.input_size, Ordering::Relaxed);
                    total_output_size.fetch_add(conversion_result.output_size, Ordering::Relaxed);
                    
                    if let Ok(mut results_guard) = results.lock() {
                        results_guard.push(conversion_result);
                    }

                    tracing::debug!(
                        "Successfully converted {} ({}/{})",
                        binary_file.display(),
                        completed,
                        binary_files.len()
                    );
                }
                Err(e) => {
                    let error = ConversionError {
                        file_path: binary_file.to_string_lossy().to_string(),
                        error_message: e.to_string(),
                        timestamp: std::time::SystemTime::now(),
                        error_category: Self::categorize_error(&e),
                    };

                    if let Ok(mut errors_guard) = errors.lock() {
                        errors_guard.push(error);
                    }

                    tracing::error!(
                        "Failed to convert {} ({}/{}): {}",
                        binary_file.display(),
                        completed,
                        binary_files.len(),
                        e
                    );

                    if !options.continue_on_error {
                        return Err(e);
                    }
                }
            }

            // Update progress and call callback
            if completed % options.progress_update_interval == 0 {
                if let Some(ref callback) = options.progress_callback {
                    if let Ok(mut progress) = progress_data.lock() {
                        progress.completed_files = completed;
                        progress.successful_conversions = success_count.load(Ordering::Relaxed);
                        progress.failed_conversions = completed - progress.successful_conversions;
                        progress.elapsed_time = start_time.elapsed();
                        progress.bytes_processed = total_input_size.load(Ordering::Relaxed);
                        
                        if progress.elapsed_time.as_secs_f64() > 0.0 {
                            progress.processing_speed_mbps = progress.bytes_processed as f64
                                / (1024.0 * 1024.0)
                                / progress.elapsed_time.as_secs_f64();
                        }

                        callback(&progress);
                    }
                }
            }
        }

        Ok(())
    }

    /// Convert a single file
    fn convert_single_file(
        binary_file: &Path,
        output_dir: &Path,
        output_format: OutputFormat,
        options: &ConversionOptions,
    ) -> TrackingResult<ConversionResult> {
        let file_name = binary_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let output_file = output_dir.join(format!(
            "{}.{}",
            file_name,
            output_format.extension()
        ));

        match output_format {
            OutputFormat::Json => {
                Self::binary_to_json_with_options(binary_file, &output_file, options.clone())
            }
            OutputFormat::Html => {
                Self::binary_to_html_with_options(binary_file, &output_file, options.clone())
            }
        }
    }

    /// Categorize error for reporting
    fn categorize_error(error: &crate::core::types::TrackingError) -> ErrorCategory {
        match error {
            crate::core::types::TrackingError::IoError(_) => ErrorCategory::IoError,
            crate::core::types::TrackingError::ExportError(msg) => {
                if msg.contains("parse") || msg.contains("binary") || msg.contains("format") {
                    ErrorCategory::ParseError
                } else if msg.contains("validation") {
                    ErrorCategory::ValidationError
                } else if msg.contains("output") || msg.contains("write") {
                    ErrorCategory::OutputError
                } else {
                    ErrorCategory::Unknown
                }
            }
            _ => ErrorCategory::Unknown,
        }
    }

    /// Create JSON data from parsed binary data
    fn create_json_data(
        allocations: &[AllocationInfo],
        stats: &MemoryStats,
        memory_by_type: &[TypeMemoryUsage],
        options: &ConversionOptions,
    ) -> TrackingResult<String> {
        use serde_json::json;
        use std::time::{SystemTime, UNIX_EPOCH};

        // Convert allocations to JSON format
        let json_allocations: Vec<serde_json::Value> = allocations
            .iter()
            .map(|alloc| {
                json!({
                    "ptr": format!("0x{:x}", alloc.ptr),
                    "size": alloc.size,
                    "timestamp_alloc": alloc.timestamp_alloc,
                    "timestamp_dealloc": alloc.timestamp_dealloc,
                    "var_name": alloc.var_name,
                    "type_name": alloc.type_name,
                    "scope_name": alloc.scope_name,
                    "stack_trace": alloc.stack_trace,
                    "borrow_count": alloc.borrow_count,
                    "is_leaked": alloc.is_leaked,
                    "lifetime_ms": alloc.lifetime_ms,
                    "smart_pointer_info": alloc.smart_pointer_info,
                    "memory_layout": alloc.memory_layout,
                    "generic_info": alloc.generic_info,
                    "dynamic_type_info": alloc.dynamic_type_info,
                    "runtime_state": alloc.runtime_state,
                    "stack_allocation": alloc.stack_allocation,
                    "temporary_object": alloc.temporary_object,
                    "fragmentation_analysis": alloc.fragmentation_analysis,
                    "generic_instantiation": alloc.generic_instantiation,
                    "type_relationships": alloc.type_relationships,
                    "type_usage": alloc.type_usage,
                    "function_call_tracking": alloc.function_call_tracking,
                    "lifecycle_tracking": alloc.lifecycle_tracking,
                    "access_tracking": alloc.access_tracking
                })
            })
            .collect();

        // Convert memory by type to map format
        let memory_by_type_map: HashMap<String, (usize, usize)> = memory_by_type
            .iter()
            .map(|usage| {
                (
                    usage.type_name.clone(),
                    (usage.total_size, usage.allocation_count),
                )
            })
            .collect();

        // Create main JSON object
        let mut json_obj = json!({
            "allocations": json_allocations,
            "stats": stats,
            "memoryByType": memory_by_type_map,
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "version": env!("CARGO_PKG_VERSION"),
        });

        // Add metadata if requested
        if options.include_metadata {
            json_obj["metadata"] = json!({
                "source_format": "binary",
                "conversion_tool": "memscope-rs binary_converter",
                "conversion_timestamp": SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "original_allocation_count": allocations.len(),
                "parser_options": {
                    "strict_validation": options.parser_options.strict_validation,
                    "enable_recovery": options.parser_options.enable_recovery,
                    "verify_checksums": options.parser_options.verify_checksums,
                }
            });
        }

        serde_json::to_string_pretty(&json_obj).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to serialize JSON: {e}"
            ))
        })
    }

    /// Create a temporary MemoryTracker from parsed data for HTML export
    fn create_memory_tracker_from_parser(
        _parser: &BinaryParser,
    ) -> TrackingResult<crate::core::tracker::MemoryTracker> {
        // This is a simplified implementation
        // In a real scenario, we would need to properly reconstruct the MemoryTracker
        // For now, we'll create a basic tracker and populate it with the parsed data
        
        let tracker = crate::core::tracker::MemoryTracker::new();
        
        // Note: This is a placeholder implementation
        // The actual implementation would need to properly restore the tracker state
        // from the parsed binary data, which would require additional methods
        // in both BinaryParser and MemoryTracker
        
        tracing::warn!("Using simplified MemoryTracker reconstruction for HTML export");
        tracing::warn!("Some features may not be available in the generated HTML");
        
        Ok(tracker)
    }

    /// Validate JSON output against original data
    fn validate_json_output(
        json_data: &str,
        original_allocations: &[AllocationInfo],
    ) -> TrackingResult<bool> {
        // Parse the JSON back
        let parsed: serde_json::Value = serde_json::from_str(json_data).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to parse generated JSON for validation: {e}"
            ))
        })?;

        // Check allocation count
        let json_allocations = parsed["allocations"].as_array().ok_or_else(|| {
            crate::core::types::TrackingError::ExportError(
                "JSON missing allocations array".to_string(),
            )
        })?;

        if json_allocations.len() != original_allocations.len() {
            tracing::error!(
                "Allocation count mismatch: JSON has {}, original has {}",
                json_allocations.len(),
                original_allocations.len()
            );
            return Ok(false);
        }

        // Validate a sample of allocations
        let sample_size = (original_allocations.len() / 10).clamp(1, 100);
        for i in (0..original_allocations.len()).step_by(original_allocations.len() / sample_size) {
            let original = &original_allocations[i];
            let json_alloc = &json_allocations[i];

            // Check key fields
            if json_alloc["size"].as_u64() != Some(original.size as u64) {
                tracing::error!("Size mismatch at index {}", i);
                return Ok(false);
            }

            if json_alloc["timestamp_alloc"].as_u64() != Some(original.timestamp_alloc) {
                tracing::error!("Timestamp mismatch at index {}", i);
                return Ok(false);
            }
        }

        tracing::info!("JSON validation passed");
        Ok(true)
    }

    /// Generate a quality report for a conversion result
    pub fn generate_quality_report(conversion_result: &ConversionResult) -> TrackingResult<String> {
        if let Some(ref validation_result) = conversion_result.validation_result {
            let validator = ConversionValidator::new();
            Ok(validator.generate_quality_report(validation_result))
        } else {
            // Generate a basic report without detailed validation
            Ok(Self::generate_basic_quality_report(conversion_result))
        }
    }

    /// Generate a basic quality report without detailed validation
    fn generate_basic_quality_report(conversion_result: &ConversionResult) -> String {
        let separator = "=".repeat(80);
        let validation_status = match conversion_result.validation_passed {
            Some(true) => "✅ PASSED",
            Some(false) => "❌ FAILED",
            None => "⚪ NOT PERFORMED",
        };

        format!(
            "{}\n\
             {}BASIC CONVERSION REPORT\n\
             {}\n\n\
             File: {}\n\
             Format: {}\n\
             Input Size: {} bytes\n\
             Output Size: {} bytes\n\
             Size Ratio: {:.2}x\n\
             Conversion Speed: {:.2} MB/s\n\
             Duration: {:.2}s\n\
             Allocations Converted: {}\n\
             Validation: {}\n\n\
             {}\n\
             {}END OF REPORT\n\
             {}\n",
            separator,
            " ".repeat(20),
            separator,
            conversion_result.output_path,
            conversion_result.output_format,
            conversion_result.input_size,
            conversion_result.output_size,
            conversion_result.size_ratio(),
            conversion_result.conversion_speed_mbps(),
            conversion_result.conversion_duration.as_secs_f64(),
            conversion_result.allocations_converted,
            validation_status,
            separator,
            " ".repeat(20),
            separator
        )
    }

    /// Validate batch conversion results
    pub fn validate_batch_conversion(
        batch_report: &BatchConversionReport,
        validation_options: Option<ValidationOptions>,
    ) -> TrackingResult<crate::export::conversion_validator::BatchValidationReport> {
        let validator = if let Some(options) = validation_options {
            ConversionValidator::with_options(options)
        } else {
            ConversionValidator::new()
        };

        validator.validate_batch_conversion(batch_report)
    }

    /// Generate comprehensive batch quality report
    pub fn generate_batch_quality_report(
        batch_report: &BatchConversionReport,
        validation_options: Option<ValidationOptions>,
    ) -> TrackingResult<String> {
        let batch_validation = Self::validate_batch_conversion(batch_report, validation_options)?;
        
        let mut report = String::new();
        
        report.push_str(&"=".repeat(80));
        report.push_str("\n                    BATCH CONVERSION QUALITY REPORT\n");
        report.push_str(&"=".repeat(80));
        report.push('\n');

        // Summary
        report.push_str(&format!(
            "\nBATCH SUMMARY:\n\
             Total Files: {}\n\
             Successful: {} ({:.1}%)\n\
             Failed: {}\n\
             Overall Speed: {:.2} MB/s\n\
             Overall Size Ratio: {:.2}x\n\
             Batch Quality Score: {:.1}/100\n\
             Performance Category: {:?}\n\
             Total Duration: {:.2}s\n\
             Files per Second: {:.2}\n",
            batch_validation.total_files,
            batch_validation.successful_validations,
            batch_validation.overall_success_rate * 100.0,
            batch_validation.failed_validations,
            batch_validation.overall_speed_mbps,
            batch_validation.overall_size_ratio,
            batch_validation.batch_quality_score,
            batch_validation.batch_performance_category,
            batch_report.total_duration.as_secs_f64(),
            batch_report.files_per_second
        ));

        // Error breakdown
        if !batch_report.errors.is_empty() {
            report.push_str("\nERROR BREAKDOWN:\n");
            let errors_by_category = batch_report.errors_by_category();
            for (category, errors) in errors_by_category {
                report.push_str(&format!("  {:?}: {} errors\n", category, errors.len()));
            }
        }

        // Performance analysis
        report.push_str("\nPERFORMANCE ANALYSIS:\n");
        if !batch_report.results.is_empty() {
            let speeds: Vec<f64> = batch_report.results.iter()
                .map(|r| r.conversion_speed_mbps())
                .collect();
            let size_ratios: Vec<f64> = batch_report.results.iter()
                .map(|r| r.size_ratio())
                .collect();

            let avg_speed = speeds.iter().sum::<f64>() / speeds.len() as f64;
            let avg_size_ratio = size_ratios.iter().sum::<f64>() / size_ratios.len() as f64;
            let min_speed = speeds.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_speed = speeds.iter().fold(0.0f64, |a, &b| a.max(b));

            report.push_str(&format!(
                "  Average Speed: {:.2} MB/s\n\
                 Speed Range: {:.2} - {:.2} MB/s\n\
                 Average Size Ratio: {:.2}x\n\
                 Parallel Workers: {}\n",
                avg_speed,
                min_speed,
                max_speed,
                avg_size_ratio,
                batch_report.parallel_workers
            ));
        }

        report.push_str("\n");
        report.push_str(&"=".repeat(80));
        report.push_str("\n                    END OF BATCH REPORT\n");
        report.push_str(&"=".repeat(80));
        report.push('\n');

        Ok(report)
    }

    /// Find all binary files in a directory
    fn find_binary_files<P: AsRef<Path>>(dir: P) -> TrackingResult<Vec<std::path::PathBuf>> {
        let dir = dir.as_ref();
        let mut binary_files = Vec::new();

        let entries = std::fs::read_dir(dir).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to read directory {}: {e}",
                dir.display()
            ))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to read directory entry: {e}"
                ))
            })?;

            let path = entry.path();
            
            // Check if it's a binary file (by extension or magic number)
            if Self::is_binary_file(&path)? {
                binary_files.push(path);
            }
        }

        binary_files.sort();
        Ok(binary_files)
    }

    /// Check if a file is a binary format file
    fn is_binary_file<P: AsRef<Path>>(path: P) -> TrackingResult<bool> {
        let path = path.as_ref();

        // Check file extension first
        if let Some(extension) = path.extension() {
            if extension == "memscope" || extension == "bin" {
                return Ok(true);
            }
        }

        // Check magic number
        if let Ok(mut file) = std::fs::File::open(path) {
            let mut magic = [0u8; 8];
            if let Ok(8) = std::io::Read::read(&mut file, &mut magic) {
                return Ok(magic == crate::export::binary_format::BINARY_MAGIC);
            }
        }

        Ok(false)
    }
}

/// Output format for conversions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// JSON format
    Json,
    /// HTML format
    Html,
}

impl OutputFormat {
    /// Get file extension for the format
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Json => "json",
            OutputFormat::Html => "html",
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_options_defaults() {
        let options = ConversionOptions::default();
        assert_eq!(options.output_buffer_size, 256 * 1024);
        assert!(!options.enable_progress_reporting);
        assert!(!options.validate_output);
        assert!(options.include_metadata);
        assert!(options.validation_options.is_none());
    }

    #[test]
    fn test_conversion_options_fast() {
        let options = ConversionOptions::fast();
        assert_eq!(options.output_buffer_size, 512 * 1024);
        assert!(!options.validate_output);
        assert!(!options.include_metadata);
        assert!(options.validation_options.is_none());
    }

    #[test]
    fn test_conversion_result_calculations() {
        let result = ConversionResult {
            output_path: "test.json".to_string(),
            input_size: 1000,
            output_size: 2000,
            conversion_duration: Duration::from_secs(1),
            allocations_converted: 100,
            validation_passed: Some(true),
            validation_result: None,
            output_format: "JSON".to_string(),
        };

        assert_eq!(result.size_ratio(), 2.0);
        assert!((result.conversion_speed_mbps() - 0.00095367).abs() < 0.00001);
    }

    #[test]
    fn test_output_format_extension() {
        assert_eq!(OutputFormat::Json.extension(), "json");
        assert_eq!(OutputFormat::Html.extension(), "html");
    }

    #[test]
    fn test_batch_conversion_report_calculations() {
        let report = BatchConversionReport {
            results: Vec::new(),
            errors: Vec::new(),
            total_duration: Duration::from_secs(2),
            successful_conversions: 8,
            failed_conversions: 2,
            total_input_size: 10000,
            total_output_size: 15000,
            was_cancelled: false,
            peak_memory_usage: None,
            parallel_workers: 4,
            average_speed_per_file: 1.25,
            files_per_second: 5.0,
        };

        assert_eq!(report.success_rate(), 0.8);
        assert_eq!(report.overall_size_ratio(), 1.5);
        assert!((report.overall_speed_mbps() - 0.00476837).abs() < 0.00001);
    }

    #[test]
    fn test_batch_conversion_options_defaults() {
        let options = BatchConversionOptions::default();
        assert!(options.enable_parallel);
        assert!(options.continue_on_error);
        assert_eq!(options.progress_update_interval, 1);
        assert!(options.parallel_workers.is_none());
    }

    #[test]
    fn test_batch_conversion_options_fast() {
        let options = BatchConversionOptions::fast();
        assert!(options.enable_parallel);
        assert!(options.continue_on_error);
        assert_eq!(options.progress_update_interval, 5);
    }

    #[test]
    fn test_batch_conversion_options_comprehensive() {
        let options = BatchConversionOptions::comprehensive();
        assert!(options.enable_parallel);
        assert!(!options.continue_on_error);
        assert_eq!(options.progress_update_interval, 1);
        assert!(options.memory_limit.is_some());
    }

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_conversion_progress_calculations() {
        let progress = ConversionProgress {
            total_files: 100,
            completed_files: 25,
            successful_conversions: 20,
            failed_conversions: 5,
            current_file: Some("test.bin".to_string()),
            elapsed_time: Duration::from_secs(10),
            estimated_remaining: Some(Duration::from_secs(30)),
            conversion_rate: 2.5,
            bytes_processed: 1024 * 1024,
            processing_speed_mbps: 0.1,
        };

        assert_eq!(progress.completion_percentage(), 25.0);
        assert_eq!(progress.success_rate(), 80.0);
        assert!(!progress.is_complete());
    }

    #[test]
    fn test_error_categorization() {
        let io_error = crate::core::types::TrackingError::IoError(
            "File not found".to_string()
        );
        assert_eq!(BinaryConverter::categorize_error(&io_error), ErrorCategory::IoError);

        let parse_error = crate::core::types::TrackingError::ExportError(
            "Failed to parse binary format".to_string()
        );
        assert_eq!(BinaryConverter::categorize_error(&parse_error), ErrorCategory::ParseError);

        let validation_error = crate::core::types::TrackingError::ExportError(
            "validation failed".to_string()
        );
        assert_eq!(BinaryConverter::categorize_error(&validation_error), ErrorCategory::ValidationError);
    }

    #[test]
    fn test_batch_report_summary_generation() {
        let report = BatchConversionReport {
            results: Vec::new(),
            errors: vec![
                ConversionError {
                    file_path: "test1.bin".to_string(),
                    error_message: "Parse error".to_string(),
                    timestamp: std::time::SystemTime::now(),
                    error_category: ErrorCategory::ParseError,
                },
                ConversionError {
                    file_path: "test2.bin".to_string(),
                    error_message: "IO error".to_string(),
                    timestamp: std::time::SystemTime::now(),
                    error_category: ErrorCategory::IoError,
                },
            ],
            total_duration: Duration::from_secs(10),
            successful_conversions: 8,
            failed_conversions: 2,
            total_input_size: 10000,
            total_output_size: 8000,
            was_cancelled: false,
            peak_memory_usage: None,
            parallel_workers: 4,
            average_speed_per_file: 1.25,
            files_per_second: 1.0,
        };

        let summary = report.generate_summary();
        assert!(summary.contains("Total Files: 10"));
        assert!(summary.contains("Successful: 8"));
        assert!(summary.contains("Failed: 2"));
        assert!(summary.contains("ParseError: 1 errors"));
        assert!(summary.contains("IoError: 1 errors"));
    }
}