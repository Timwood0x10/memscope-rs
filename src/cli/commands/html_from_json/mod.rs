//! HTML generation from JSON files command
//!
//! This module provides functionality to generate interactive HTML reports
//! from exported JSON data files, significantly faster than direct tracker export.

use clap::ArgMatches;
use rayon::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::time::Instant;

mod data_integrator;
pub mod data_normalizer;
pub mod debug_logger;
pub mod error_handler;
pub mod json_file_discovery;
pub mod large_file_optimizer;

use data_integrator::DataIntegrator;
use data_normalizer::DataNormalizer;
use debug_logger::{DebugConfig, DebugLogger, LogLevel};
use error_handler::{ErrorRecoveryContext, HtmlErrorHandler};
use json_file_discovery::{JsonFileConfig, JsonFileDiscovery};
use large_file_optimizer::{LargeFileConfig, LargeFileOptimizer};

pub mod direct_json_template;

/// Run the HTML from JSON generation command
pub fn run_html_from_json(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let input_dir = matches
        .get_one::<String>("input-dir")
        .ok_or("Input directory is required")?;
    let validate_only = matches.get_flag("validate-only");
    let default_output = "validation_only.html".to_string();
    let output_file = if validate_only {
        matches
            .get_one::<String>("output")
            .unwrap_or(&default_output)
    } else {
        matches
            .get_one::<String>("output")
            .ok_or("Output HTML file is required")?
    };
    let base_name = matches
        .get_one::<String>("base-name")
        .map(|s| s.as_str())
        .unwrap_or("snapshot");

    // Configure debug logging based on command line options
    let verbose = matches.get_flag("verbose");
    let debug_mode = matches.get_flag("debug");
    let performance_mode = matches.get_flag("performance");

    let debug_config = DebugConfig {
        log_level: if debug_mode {
            LogLevel::Debug
        } else if verbose {
            LogLevel::Info
        } else {
            LogLevel::Info
        },
        enable_timing: performance_mode || verbose,
        enable_progress: verbose || debug_mode,
        enable_memory_tracking: performance_mode || debug_mode,
        enable_file_ops: debug_mode,
        enable_json_details: debug_mode,
        progress_interval_ms: if debug_mode { 500 } else { 2000 },
        include_timestamps: debug_mode || performance_mode,
    };

    let logger = DebugLogger::with_config(debug_config);

    logger.info("🚀 Generating HTML report from JSON files...");
    logger.info(&format!("📁 Input directory: {input_dir}"));
    logger.info(&format!("📄 Output file: {output_file}"));
    logger.info(&format!("🏷️  Base name: {base_name}"));

    if verbose {
        logger.info(&format!(
            "🔧 Debug mode: {debug_mode}, Verbose: {verbose}, Performance: {performance_mode}",
        ));
    }

    // Start overall progress tracking
    logger.start_progress(5, "Initializing HTML generation");

    // 🎯 Load JSON files
    let discovery_timing = logger.start_timing("json_file_discovery");
    logger.next_progress_step("Loading JSON files", 1);
    let json_data = load_json_files_with_logging(input_dir, base_name, &logger)?;
    let discovery_time = logger.end_timing(&discovery_timing).unwrap_or(0);
    logger.update_stats(|stats| stats.discovery_time_ms = discovery_time);

    // 🔄 Normalize data
    logger.next_progress_step("Normalizing data", 1);
    let normalization_timing = logger.start_timing("data_normalization");
    let normalizer = DataNormalizer::new();
    let mut unified_data = normalizer.normalize(&json_data)?;
    let normalization_time = logger.end_timing(&normalization_timing).unwrap_or(0);
    logger.update_stats(|stats| stats.normalization_time_ms = normalization_time);
    logger.debug(&format!(
        "📊 Normalized {} allocations",
        unified_data.allocations.len()
    ));

    // Check if we should only validate and exit early
    if validate_only {
        logger.info("✅ JSON validation completed successfully!");
        logger.info(&format!("📊 Validation results:"));
        logger.info(&format!("   - Files loaded: {}", json_data.len()));
        logger.info(&format!(
            "   - Allocations found: {}",
            unified_data.allocations.len()
        ));
        logger.info(&format!(
            "   - Lifecycle events: {}",
            unified_data.lifecycle.lifecycle_events.len()
        ));
        logger.info("- Performance data: Available");
        logger.info(&format!(
            "   - Security violations: {}",
            unified_data.security.total_violations
        ));
        logger.info(&format!(
            "   - Complex types: {}",
            unified_data.complex_types.summary.total_complex_types
        ));
        logger.info(&format!(
            "   - Active memory: {} bytes",
            unified_data.stats.active_memory
        ));
        logger.info(&format!(
            "   - Peak memory: {} bytes",
            unified_data.stats.peak_memory
        ));
        return Ok(());
    }

    // 🔗 Integrate multiple data sources
    logger.next_progress_step("Integrating data sources", 1);
    let integration_timing = logger.start_timing("data_integration");
    let integrator = DataIntegrator::new();
    let integration_stats = integrator.integrate(&mut unified_data)?;
    let integration_time = logger.end_timing(&integration_timing).unwrap_or(0);
    logger.update_stats(|stats| stats.integration_time_ms = integration_time);

    logger.debug(&format!(
        "🔗 Integration completed: {} cross-references, {} conflicts resolved",
        integration_stats.cross_references_found, integration_stats.conflicts_resolved
    ));

    tracing::info!("📊 Integration Statistics:");
    tracing::info!(
        "   Cross-references found: {}",
        integration_stats.cross_references_found
    );
    tracing::info!(
        "   Conflicts resolved: {}",
        integration_stats.conflicts_resolved
    );
    tracing::info!(
        "   Data enrichments: {}",
        integration_stats.enrichments_performed
    );
    tracing::info!(
        "   Index build time: {}ms",
        integration_stats.index_build_time_ms
    );
    tracing::info!(
        "   Total integration time: {}ms",
        integration_stats.integration_time_ms
    );

    {
        // 🎨 Generate static HTML report - using direct JSON data template
        logger.next_progress_step("Generating HTML template", 1);
        let template_timing = logger.start_timing("template_generation");
        logger.info("🎨 Using direct JSON data template with charts...");
        let html_content = direct_json_template::generate_direct_html(&json_data)?;
        let template_time = logger.end_timing(&template_timing).unwrap_or(0);
        logger.update_stats(|stats| stats.template_time_ms = template_time);

        // Simple template statistics
        let template_size_bytes = html_content.len();
        let generation_time_ms = template_time;

        tracing::info!("🎨 Template Generation Statistics:");
        tracing::info!(
            "   Template size: {:.1} KB",
            template_size_bytes as f64 / 1024.0
        );
        tracing::info!(
            "   Total generation time: {}ms",
            generation_time_ms
        );

        // Determine output path - if output is just a filename, put it in the input directory
        let output_path = if Path::new(output_file).is_absolute() || output_file.contains('/') {
            // Use the provided path as-is
            output_file.to_string()
        } else {
            // Put the HTML file in the input directory
            format!("{}/{}", input_dir.trim_end_matches('/'), output_file)
        };

        logger.next_progress_step("Writing HTML file", 1);
        let write_timing = logger.start_timing("file_write");
        logger.info(&format!("📁 Writing HTML file to: {output_path}"));

        // Write HTML file
        fs::write(&output_path, &html_content)?;
        logger.end_timing(&write_timing);

        logger.log_file_operation("written", &output_path, Some(html_content.len()));

        // Update the output message
        logger.info("✅ HTML report generated successfully!");
        logger.info(&format!(
            "🌐 Open {output_path} in your browser to view the interactive report",
        ));

        // Print comprehensive performance report if requested
        if performance_mode || debug_mode {
            logger.print_performance_report();
            logger.print_memory_info();
        }
    }

    Ok(())
}

/// Result of loading a single JSON file
#[derive(Debug)]
pub struct JsonLoadResult {
    /// File suffix
    pub suffix: String,
    /// Whether loading was successful
    pub success: bool,
    /// Loaded JSON data
    pub data: Option<Value>,
    /// Error message if failed
    pub error: Option<String>,
    /// File size in bytes
    pub file_size: usize,
    /// Load time in milliseconds
    pub load_time_ms: u64,
}

/// Statistics for the JSON loading process
#[derive(Debug)]
pub struct JsonLoadStats {
    /// Total files attempted
    pub total_files_attempted: usize,
    /// Files successfully loaded
    pub files_loaded: usize,
    /// Files skipped
    pub files_skipped: usize,
    /// Files failed to load
    pub files_failed: usize,
    /// Total size in bytes
    pub total_size_bytes: usize,
    /// Total load time in milliseconds
    pub total_load_time_ms: u64,
    /// Whether parallel loading was used
    pub parallel_loading_used: bool,
}

/// Collection of JSON data from different analysis files
type JsonDataCollection = HashMap<String, Value>;

/// Optimized JSON file loader with parallel processing and monitoring
fn load_json_files_with_logging(
    input_dir: &str,
    base_name: &str,
    logger: &DebugLogger,
) -> Result<JsonDataCollection, Box<dyn Error>> {
    let start_time = Instant::now();

    logger.debug("🚀 Starting optimized JSON file loading with comprehensive error handling...");
    logger.debug(&format!("📁 Directory: {input_dir}"));
    logger.debug(&format!("🏷️  Base name: {base_name}"));

    // Initialize error handler with recovery context
    let recovery_context = ErrorRecoveryContext {
        attempt_recovery: true,
        max_retries: 3,
        allow_partial_data: true,
        use_fallbacks: true,
        verbose_errors: false,
    };
    let mut error_handler = HtmlErrorHandler::with_context(recovery_context);

    // Use the new JSON file discovery system with error handling
    let discovery_timing = logger.start_timing("file_discovery");
    let discovery = JsonFileDiscovery::new(input_dir.to_string(), base_name.to_string());
    let discovery_result = match discovery.discover_files() {
        Ok(result) => {
            logger.debug(&format!(
                "📊 Discovery completed: {} files found",
                result.found_files.len()
            ));
            result
        }
        Err(e) => {
            match error_handler.handle_file_discovery_error(input_dir, base_name, Box::new(e)) {
                Ok(alternatives) => {
                    logger.warn(&format!(
                        "🔄 Found alternative directories: {alternatives:?}",
                    ));
                    return Err("JSON file discovery failed after attempting recovery".into());
                }
                Err(handled_error) => {
                    logger.error(&format!("{handled_error}"));
                    return Err(handled_error.into());
                }
            }
        }
    };
    logger.end_timing(&discovery_timing);

    // Convert discovered files to the format expected by the loader
    let mut valid_files = Vec::new();
    let mut total_size = 0usize;

    for file_info in &discovery_result.found_files {
        let file_path = file_info.path.to_string_lossy().to_string();
        let file_size = file_info.size_bytes as usize;

        logger.log_file_operation("discovered", &file_path, Some(file_size));

        total_size += file_size;
        valid_files.push((file_info.config.clone(), file_path, file_size));
    }

    if valid_files.is_empty() {
        let error_msg =
            "No valid JSON files found! Please check the input directory and base name.";
        logger.error(error_msg);
        return Err(error_msg.into());
    }

    logger.info(&format!(
        "📊 Found {} valid files, total size: {:.1} MB",
        valid_files.len(),
        total_size as f64 / 1024.0 / 1024.0
    ));

    // Update progress with file count
    logger.update_progress_items(valid_files.len());

    // Intelligent decision for parallel loading based on file count, size, and system resources
    let has_large_files = valid_files
        .iter()
        .any(|(_, _, size)| *size > 20 * 1024 * 1024);
    let use_parallel = valid_files.len() >= 3 || total_size >= 10 * 1024 * 1024 || has_large_files;

    if use_parallel {
        logger.info(&format!(
            "⚡ Using parallel loading for {} files (total: {:.1} MB, has large files: {})",
            valid_files.len(),
            total_size as f64 / 1024.0 / 1024.0,
            has_large_files
        ));
    } else {
        logger.info(&format!(
            "📝 Using sequential loading for {} files (total: {:.1} MB)",
            valid_files.len(),
            total_size as f64 / 1024.0 / 1024.0
        ));
    }

    // load files
    let loading_timing = logger.start_timing("file_loading");
    let results = if use_parallel {
        load_files_parallel_with_logging(&valid_files, logger)?
    } else {
        load_files_sequential_with_logging(&valid_files, logger)?
    };
    let loading_time = logger.end_timing(&loading_timing).unwrap_or(0);

    // process results
    let mut data = JsonDataCollection::new();
    let mut stats = JsonLoadStats {
        total_files_attempted: valid_files.len(),
        files_loaded: 0,
        files_skipped: 0,
        files_failed: 0,
        total_size_bytes: 0,
        total_load_time_ms: start_time.elapsed().as_millis() as u64,
        parallel_loading_used: use_parallel,
    };

    for result in results {
        if result.success {
            if let Some(json_data) = result.data {
                data.insert(result.suffix.clone(), json_data);
                stats.files_loaded += 1;
                stats.total_size_bytes += result.file_size;
                logger.debug(&format!(
                    "✅ Loaded {} ({:.1} KB in {}ms)",
                    result.suffix,
                    result.file_size as f64 / 1024.0,
                    result.load_time_ms
                ));
            }
        } else {
            stats.files_failed += 1;
            logger.error(&format!(
                "❌ Failed to load {}: {}",
                result.suffix,
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ));
        }
    }

    // print statistics
    print_load_statistics_with_logging(&stats, logger);

    // Print error recovery summary
    error_handler.print_recovery_summary();

    // Update performance stats
    logger.update_stats(|perf_stats| {
        perf_stats.loading_time_ms = loading_time;
        perf_stats.files_processed = stats.files_loaded;
        perf_stats.data_size_bytes = stats.total_size_bytes;
    });

    if data.is_empty() {
        let error_msg = "No JSON files were successfully loaded!";
        logger.error(error_msg);
        return Err(error_msg.into());
    }

    Ok(data)
}

/// Original load function for backward compatibility
#[allow(dead_code)]
fn load_json_files(input_dir: &str, base_name: &str) -> Result<JsonDataCollection, Box<dyn Error>> {
    let logger = DebugLogger::new();
    load_json_files_with_logging(input_dir, base_name, &logger)
}

/// Load files in parallel using rayon with error handling and logging
fn load_files_parallel_with_logging(
    files: &[(JsonFileConfig, String, usize)],
    logger: &DebugLogger,
) -> Result<Vec<JsonLoadResult>, Box<dyn Error>> {
    logger.debug("⚡ Starting parallel file loading");

    let results: Vec<JsonLoadResult> = files
        .par_iter()
        .enumerate()
        .map(|(index, (config, file_path, file_size))| {
            let file_timing = format!("load_file_{index}");
            let timing_id = logger.start_timing(&file_timing);

            logger.log_file_operation("loading", file_path, Some(*file_size));
            let result = load_single_file_with_recovery(config, file_path, *file_size);

            logger.end_timing(&timing_id);
            result
        })
        .collect();

    logger.debug("✅ Parallel file loading completed");
    Ok(results)
}

/// Load files in parallel using rayon with error handling (backward compatibility)
#[allow(dead_code)]
fn load_files_parallel(
    files: &[(JsonFileConfig, String, usize)],
) -> Result<Vec<JsonLoadResult>, Box<dyn Error>> {
    let logger = DebugLogger::new();
    load_files_parallel_with_logging(files, &logger)
}

/// Load files sequentially with error handling and logging
fn load_files_sequential_with_logging(
    files: &[(JsonFileConfig, String, usize)],
    logger: &DebugLogger,
) -> Result<Vec<JsonLoadResult>, Box<dyn Error>> {
    logger.debug("📝 Starting sequential file loading");
    let mut results = Vec::new();

    for (index, (config, file_path, file_size)) in files.iter().enumerate() {
        let file_timing = format!("load_file_{index}");
        let timing_id = logger.start_timing(&file_timing);

        logger.log_file_operation("loading", file_path, Some(*file_size));
        let result = load_single_file_with_recovery(config, file_path, *file_size);

        logger.end_timing(&timing_id);
        results.push(result);

        // Update progress
        logger.update_progress_items(index + 1);
    }

    logger.debug("✅ Sequential file loading completed");
    Ok(results)
}

/// Load files sequentially with error handling (backward compatibility)
#[allow(dead_code)]
fn load_files_sequential(
    files: &[(JsonFileConfig, String, usize)],
) -> Result<Vec<JsonLoadResult>, Box<dyn Error>> {
    let logger = DebugLogger::new();
    load_files_sequential_with_logging(files, &logger)
}

/// Load a single JSON file with comprehensive error handling and recovery
fn load_single_file_with_recovery(
    config: &JsonFileConfig,
    file_path: &str,
    file_size: usize,
) -> JsonLoadResult {
    // Create a local error handler for this file
    let mut local_error_handler = HtmlErrorHandler::new();

    match load_single_file_internal(config, file_path, file_size, &mut local_error_handler) {
        Ok(result) => result,
        Err(e) => {
            // Convert error to JsonLoadResult format
            JsonLoadResult {
                suffix: config.suffix.to_string(),
                success: false,
                data: None,
                error: Some(e.to_string()),
                file_size,
                load_time_ms: 0,
            }
        }
    }
}

/// Internal file loading with error handler
fn load_single_file_internal(
    config: &JsonFileConfig,
    file_path: &str,
    file_size: usize,
    error_handler: &mut HtmlErrorHandler,
) -> Result<JsonLoadResult, Box<dyn Error>> {
    let start_time = Instant::now();

    // Use large file optimizer for files > 50MB or if specified in config
    let use_large_file_optimizer = file_size > 50 * 1024 * 1024
        || config.max_size_mb.is_some_and(|max_mb| file_size > max_mb * 1024 * 1024 / 2);

    if use_large_file_optimizer {
        // Use optimized large file processing
        let large_file_config = LargeFileConfig {
            max_memory_bytes: 256 * 1024 * 1024, // 256MB limit for large files
            stream_chunk_size: 128 * 1024,       // 128KB chunks
            enable_memory_monitoring: true,
            enable_progress_reporting: true,
            max_file_size_bytes: config.max_size_mb.unwrap_or(500) * 1024 * 1024,
        };

        let optimizer = LargeFileOptimizer::new(large_file_config);

        match optimizer.process_file(file_path, config.suffix) {
            Ok((json_value, processing_stats)) => {
                tracing::info!(
                    "📊 Large file processing stats for {}: {:.1} MB/s, {} objects, streaming: {}",
                    config.suffix,
                    processing_stats.throughput_mb_per_sec,
                    processing_stats.objects_processed,
                    processing_stats.streaming_mode_used
                );

                Ok(JsonLoadResult {
                    suffix: config.suffix.to_string(),
                    success: true,
                    data: Some(json_value),
                    error: None,
                    file_size,
                    load_time_ms: processing_stats.processing_time_ms,
                })
            }
            Err(e) => {
                // Handle large file processing error with recovery
                let file_path_buf = std::path::PathBuf::from(file_path);
                match error_handler.handle_file_loading_error(
                    file_path_buf,
                    config.suffix,
                    file_size,
                    Box::new(e),
                ) {
                    Ok(Some(recovered_data)) => {
                        tracing::info!("✅ Recovered data for {} using fallback", config.suffix);
                        Ok(JsonLoadResult {
                            suffix: config.suffix.to_string(),
                            success: true,
                            data: Some(recovered_data),
                            error: None,
                            file_size,
                            load_time_ms: start_time.elapsed().as_millis() as u64,
                        })
                    }
                    Ok(None) => Err(format!(
                        "Failed to load {} and no fallback available",
                        config.suffix
                    )
                    .into()),
                    Err(handled_error) => Err(handled_error.into()),
                }
            }
        }
    } else {
        // Use standard processing for smaller files with error handling
        match std::fs::read_to_string(file_path) {
            Ok(content) => {
                match serde_json::from_str::<Value>(&content) {
                    Ok(json_value) => {
                        // Validate JSON structure
                        if let Err(validation_error) =
                            validate_json_structure(&json_value, config.suffix)
                        {
                            let validation_err = error_handler.handle_validation_error(
                                std::path::PathBuf::from(file_path),
                                config.suffix,
                                &validation_error,
                                &json_value,
                            );

                            tracing::error!("{}", validation_err);

                            // Try to continue with partial data if allowed
                            let allow_partial = {
                                let stats = error_handler.get_stats();
                                stats.total_errors < 5 // Allow partial data if not too many errors
                            };
                            if allow_partial {
                                tracing::info!(
                                    "⚠️  Continuing with potentially invalid data for {}",
                                    config.suffix
                                );
                                Ok(JsonLoadResult {
                                    suffix: config.suffix.to_string(),
                                    success: true,
                                    data: Some(json_value),
                                    error: Some(format!("Validation warning: {validation_error}")),
                                    file_size,
                                    load_time_ms: start_time.elapsed().as_millis() as u64,
                                })
                            } else {
                                Err(validation_err.into())
                            }
                        } else {
                            Ok(JsonLoadResult {
                                suffix: config.suffix.to_string(),
                                success: true,
                                data: Some(json_value),
                                error: None,
                                file_size,
                                load_time_ms: start_time.elapsed().as_millis() as u64,
                            })
                        }
                    }
                    Err(e) => {
                        let parsing_err = error_handler.handle_json_parsing_error(
                            std::path::PathBuf::from(file_path),
                            &e.to_string(),
                        );

                        tracing::error!("{}", parsing_err);
                        Err(parsing_err.into())
                    }
                }
            }
            Err(e) => {
                // Handle file reading error with recovery
                let file_path_buf = std::path::PathBuf::from(file_path);
                match error_handler.handle_file_loading_error(
                    file_path_buf,
                    config.suffix,
                    file_size,
                    Box::new(e),
                ) {
                    Ok(Some(recovered_data)) => {
                        tracing::info!("✅ Recovered data for {} using fallback", config.suffix);
                        Ok(JsonLoadResult {
                            suffix: config.suffix.to_string(),
                            success: true,
                            data: Some(recovered_data),
                            error: None,
                            file_size,
                            load_time_ms: start_time.elapsed().as_millis() as u64,
                        })
                    }
                    Ok(None) => Err(format!(
                        "Failed to load {} and no fallback available",
                        config.suffix
                    )
                    .into()),
                    Err(handled_error) => Err(handled_error.into()),
                }
            }
        }
    }
}

/// Original load single file function (kept for compatibility)
#[allow(dead_code)]
fn load_single_file(config: &JsonFileConfig, file_path: &str, file_size: usize) -> JsonLoadResult {
    let start_time = Instant::now();

    // Use large file optimizer for files > 50MB or if specified in config
    let use_large_file_optimizer = file_size > 50 * 1024 * 1024
        || config.max_size_mb.is_some_and(|max_mb| file_size > max_mb * 1024 * 1024 / 2);

    let result = if use_large_file_optimizer {
        // Use optimized large file processing
        let large_file_config = LargeFileConfig {
            max_memory_bytes: 256 * 1024 * 1024, // 256MB limit for large files
            stream_chunk_size: 128 * 1024,       // 128KB chunks
            enable_memory_monitoring: true,
            enable_progress_reporting: true,
            max_file_size_bytes: config.max_size_mb.unwrap_or(500) * 1024 * 1024,
        };

        let optimizer = LargeFileOptimizer::new(large_file_config);

        match optimizer.process_file(file_path, config.suffix) {
            Ok((json_value, processing_stats)) => {
                tracing::info!(
                    "📊 Large file processing stats for {}: {:.1} MB/s, {} objects, streaming: {}",
                    config.suffix,
                    processing_stats.throughput_mb_per_sec,
                    processing_stats.objects_processed,
                    processing_stats.streaming_mode_used
                );

                JsonLoadResult {
                    suffix: config.suffix.to_string(),
                    success: true,
                    data: Some(json_value),
                    error: None,
                    file_size,
                    load_time_ms: processing_stats.processing_time_ms,
                }
            }
            Err(e) => JsonLoadResult {
                suffix: config.suffix.to_string(),
                success: false,
                data: None,
                error: Some(format!("Large file processing error: {e}")),
                file_size,
                load_time_ms: start_time.elapsed().as_millis() as u64,
            },
        }
    } else {
        // Use standard processing for smaller files
        match fs::read_to_string(file_path) {
            Ok(content) => {
                match serde_json::from_str::<Value>(&content) {
                    Ok(json_value) => {
                        // validate JSON structure
                        if let Err(validation_error) =
                            validate_json_structure(&json_value, config.suffix)
                        {
                            JsonLoadResult {
                                suffix: config.suffix.to_string(),
                                success: false,
                                data: None,
                                error: Some(format!("Validation error: {validation_error}")),
                                file_size,
                                load_time_ms: start_time.elapsed().as_millis() as u64,
                            }
                        } else {
                            JsonLoadResult {
                                suffix: config.suffix.to_string(),
                                success: true,
                                data: Some(json_value),
                                error: None,
                                file_size,
                                load_time_ms: start_time.elapsed().as_millis() as u64,
                            }
                        }
                    }
                    Err(e) => JsonLoadResult {
                        suffix: config.suffix.to_string(),
                        success: false,
                        data: None,
                        error: Some(format!("JSON parsing error: {e}")),
                        file_size,
                        load_time_ms: start_time.elapsed().as_millis() as u64,
                    },
                }
            }
            Err(e) => JsonLoadResult {
                suffix: config.suffix.to_string(),
                success: false,
                data: None,
                error: Some(format!("File read error: {e}")),
                file_size,
                load_time_ms: start_time.elapsed().as_millis() as u64,
            },
        }
    };

    result
}

/// Validate JSON structure based on file type
fn validate_json_structure(json: &Value, file_type: &str) -> Result<(), String> {
    match file_type {
        "memory_analysis" => {
            if !json.is_object() {
                return Err("Memory analysis JSON must be an object".to_string());
            }
            // can add more specific validation
        }
        "performance" => {
            if !json.is_object() {
                return Err("Performance JSON must be an object".to_string());
            }
        }
        _ => {
            // basic validation: ensure it's a valid JSON object or array
            if !json.is_object() && !json.is_array() {
                return Err("JSON must be an object or array".to_string());
            }
        }
    }
    Ok(())
}

/// Print loading statistics with logging
fn print_load_statistics_with_logging(stats: &JsonLoadStats, logger: &DebugLogger) {
    logger.info("\n📈 JSON Loading Statistics:");
    logger.info(&format!(
        "   Files attempted: {}",
        stats.total_files_attempted
    ));
    logger.info(&format!("   Files loaded: {}", stats.files_loaded));
    logger.info(&format!("   Files failed: {}", stats.files_failed));
    logger.info(&format!(
        "   Total size: {:.1} MB",
        stats.total_size_bytes as f64 / 1024.0 / 1024.0
    ));
    logger.info(&format!("   Total time: {}ms", stats.total_load_time_ms));
    logger.info(&format!(
        "   Parallel loading: {}",
        if stats.parallel_loading_used {
            "Yes"
        } else {
            "No"
        }
    ));

    if stats.files_loaded > 0 {
        let avg_time = stats.total_load_time_ms / stats.files_loaded as u64;
        let throughput = if stats.total_load_time_ms > 0 {
            (stats.total_size_bytes as f64 / 1024.0 / 1024.0)
                / (stats.total_load_time_ms as f64 / 1000.0)
        } else {
            0.0
        };
        logger.info(&format!("   Average time per file: {avg_time}ms"));
        logger.info(&format!("   Throughput: {throughput:.1} MB/s"));

        // Memory efficiency information
        let memory_efficiency = if stats.total_size_bytes > 0 {
            // Estimate memory efficiency based on file sizes and processing time
            let estimated_peak_memory = stats.total_size_bytes as f64 * 1.5; // Assume 1.5x overhead
            let efficiency = (stats.total_size_bytes as f64 / estimated_peak_memory) * 100.0;
            format!("{efficiency:.1}%")
        } else {
            "N/A".to_string()
        };
        logger.info(&format!("   Memory efficiency: {memory_efficiency}"));
    }
    logger.info("");
}
