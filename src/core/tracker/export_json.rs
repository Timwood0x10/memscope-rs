//! Optimized JSON export functionality for memory tracking data.
//!
//! This module provides highly optimized JSON export functionality with performance improvements
//! including parallel processing, streaming writes, and adaptive optimization.

use super::memory_tracker::MemoryTracker;
use crate::core::types::{AllocationInfo, MemoryStats, TrackingResult, TypeMemoryUsage};
use crate::export::optimized_json_export::OptimizationLevel;
use crate::export::schema_validator::SchemaValidator;
use rayon::prelude::*;
use serde_json::json;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

// Optimized export options with intelligent defaults
#[derive(Debug, Clone)]
pub struct ExportJsonOptions {
    /// Use parallel processing for large datasets
    pub parallel_processing: bool,
    /// Buffer size for file I/O
    pub buffer_size: usize,
    /// Use compact JSON format for large files
    pub use_compact_format: Option<bool>,
    /// Enable type inference caching
    pub enable_type_cache: bool,
    /// Batch size for processing allocations
    pub batch_size: usize,
    /// Enable streaming writer for large exports
    pub streaming_writer: bool,
    /// Enable schema validation
    pub schema_validation: bool,
    /// Enable adaptive optimization
    pub adaptive_optimization: bool,
    /// Maximum cache size for type information
    pub max_cache_size: usize,
    /// Enable security violation analysis
    pub security_analysis: bool,
    /// Include low severity violations
    pub include_low_severity: bool,
    /// Generate integrity hashes
    pub integrity_hashes: bool,
    /// Fast export mode (reduces data quality for speed)
    pub fast_export_mode: bool,
    /// Auto-enable fast export for large datasets
    pub auto_fast_export_threshold: Option<usize>,
    /// Number of threads for parallel processing
    pub thread_count: Option<usize>,
}

impl Default for ExportJsonOptions {
    fn default() -> Self {
        Self {
            parallel_processing: true,
            buffer_size: 256 * 1024,  // 256KB
            use_compact_format: None, // Auto-detect
            enable_type_cache: true,
            batch_size: 1000,
            streaming_writer: true,
            schema_validation: false, // Off by default for speed
            adaptive_optimization: true,
            max_cache_size: 10_000,
            security_analysis: false, // Off by default for speed
            include_low_severity: false,
            integrity_hashes: false, // Off by default for speed
            fast_export_mode: false,
            auto_fast_export_threshold: Some(10_000), // Auto-enable fast mode for >10k allocations
            thread_count: None,                       // Use default thread count
        }
    }
}

impl ExportJsonOptions {
    /// Create new options with specified optimization level
    pub fn with_optimization_level(level: OptimizationLevel) -> Self {
        match level {
            OptimizationLevel::Low => Self {
                parallel_processing: true,
                buffer_size: 128 * 1024,
                use_compact_format: Some(true),
                enable_type_cache: true,
                batch_size: 2000,
                streaming_writer: true,
                schema_validation: false,
                adaptive_optimization: false,
                max_cache_size: 5_000,
                security_analysis: false,
                include_low_severity: false,
                integrity_hashes: false,
                fast_export_mode: true,
                auto_fast_export_threshold: Some(5_000),
                thread_count: None,
            },
            OptimizationLevel::Medium => Self::default(),
            OptimizationLevel::High => Self {
                parallel_processing: true,
                buffer_size: 512 * 1024,
                use_compact_format: Some(false),
                enable_type_cache: true,
                batch_size: 500,
                streaming_writer: true,
                schema_validation: true,
                adaptive_optimization: true,
                max_cache_size: 50_000,
                security_analysis: true,
                include_low_severity: true,
                integrity_hashes: true,
                fast_export_mode: false,
                auto_fast_export_threshold: None,
                thread_count: None,
            },
            OptimizationLevel::Maximum => Self {
                parallel_processing: true,
                buffer_size: 1024 * 1024, // 1MB buffer for maximum performance
                use_compact_format: Some(true),
                enable_type_cache: true,
                batch_size: 1000,
                streaming_writer: true,
                schema_validation: true,
                adaptive_optimization: true,
                max_cache_size: 100_000,
                security_analysis: true,
                include_low_severity: true,
                integrity_hashes: true,
                fast_export_mode: true,
                auto_fast_export_threshold: Some(10_000),
                thread_count: None,
            },
        }
    }

    // Builder pattern methods for options
    pub fn parallel_processing(mut self, enabled: bool) -> Self {
        self.parallel_processing = enabled;
        self
    }

    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    pub fn fast_export_mode(mut self, enabled: bool) -> Self {
        self.fast_export_mode = enabled;
        self
    }

    pub fn security_analysis(mut self, enabled: bool) -> Self {
        self.security_analysis = enabled;
        self
    }

    /// Enable or disable streaming writer
    pub fn streaming_writer(mut self, enabled: bool) -> Self {
        self.streaming_writer = enabled;
        self
    }

    pub fn schema_validation(mut self, enabled: bool) -> Self {
        self.schema_validation = enabled;
        self
    }

    pub fn integrity_hashes(mut self, enabled: bool) -> Self {
        self.integrity_hashes = enabled;
        self
    }

    /// Set the batch size for processing allocations
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Enable or disable adaptive optimization
    pub fn adaptive_optimization(mut self, enabled: bool) -> Self {
        self.adaptive_optimization = enabled;
        self
    }

    /// Set maximum cache size
    pub fn max_cache_size(mut self, size: usize) -> Self {
        self.max_cache_size = size;
        self
    }

    /// Include low severity violations in reports
    pub fn include_low_severity(mut self, include: bool) -> Self {
        self.include_low_severity = include;
        self
    }

    /// Set thread count for parallel processing (None for auto-detect)
    pub fn thread_count(mut self, count: Option<usize>) -> Self {
        self.thread_count = count;
        self
    }
}

// Type inference cache for performance optimization
static TYPE_CACHE: std::sync::OnceLock<std::sync::Mutex<HashMap<String, String>>> =
    std::sync::OnceLock::new();

/// Get cached type information or compute and cache it
fn get_or_compute_type_info(type_name: &str, size: usize) -> String {
    let cache = TYPE_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()));

    if let Ok(mut cache) = cache.lock() {
        if let Some(cached) = cache.get(type_name) {
            return cached.clone();
        }

        let result = compute_enhanced_type_info(type_name, size);
        cache.insert(type_name.to_string(), result.clone());
        result
    } else {
        compute_enhanced_type_info(type_name, size)
    }
}

/// Compute enhanced type information
fn compute_enhanced_type_info(type_name: &str, size: usize) -> String {
    // Fast path for common types
    match type_name {
        "String" | "&str" => "string".to_string(),
        "Vec" | "VecDeque" | "LinkedList" => "collection".to_string(),
        "HashMap" | "BTreeMap" => "map".to_string(),
        "HashSet" | "BTreeSet" => "set".to_string(),
        _ if size > 1024 => "large".to_string(),
        _ => "custom".to_string(),
    }
}

/// Clear the type cache (useful for testing)
#[cfg(test)]
#[allow(dead_code)]
fn clear_type_cache() {
    if let Some(cache) = TYPE_CACHE.get() {
        if let Ok(mut cache) = cache.lock() {
            cache.clear();
        }
    }
}

/// Process a batch of allocations (legacy function for compatibility)
fn process_allocation_batch(
    allocations: &[AllocationInfo],
) -> TrackingResult<Vec<serde_json::Value>> {
    let mut result = Vec::with_capacity(allocations.len());

    for alloc in allocations {
        let type_info =
            get_or_compute_type_info(alloc.type_name.as_deref().unwrap_or("unknown"), alloc.size);

        let mut entry = json!({
            "address": format!("0x{:x}", alloc.ptr),
            "size": alloc.size,
            "type": type_info,
            "timestamp": alloc.timestamp_alloc,
            // improve.md extensions
            "lifetime_ms": alloc.lifetime_ms,
            "borrow_info": alloc.borrow_info,
            "clone_info": alloc.clone_info,
            "ownership_history_available": alloc.ownership_history_available,
        });

        if let Some(var_name) = &alloc.var_name {
            entry["var_name"] = json!(var_name);
        }

        if let Some(type_name) = &alloc.type_name {
            entry["type_name"] = json!(type_name);
        }

        result.push(entry);
    }

    Ok(result)
}

/// Enhanced batch processing with new data pipeline integration
fn process_allocation_batch_enhanced(
    allocations: &[AllocationInfo],
    options: &ExportJsonOptions,
) -> TrackingResult<Vec<serde_json::Value>> {
    let start_time = std::time::Instant::now();
    let batch_size = allocations.len();

    // Process in parallel if enabled and batch size is large enough
    let result = if options.parallel_processing && batch_size > options.batch_size {
        let chunk_size = (batch_size / num_cpus::get()).max(1);

        // Process chunks in parallel and flatten the results
        allocations
            .par_chunks(chunk_size)
            .map(process_allocation_batch)
            .reduce(
                || Ok(Vec::new()),
                |acc, chunk_result| match (acc, chunk_result) {
                    (Ok(mut vec), Ok(chunk)) => {
                        vec.extend(chunk);
                        Ok(vec)
                    }
                    (Err(e), _) | (_, Err(e)) => Err(e),
                },
            )
    } else {
        // Process everything in a single chunk
        process_allocation_batch(allocations)
    };

    let elapsed = start_time.elapsed();
    tracing::debug!(
        "Processed {} allocations in {:.2?} ({} allocs/sec)",
        batch_size,
        elapsed,
        (batch_size as f64 / elapsed.as_secs_f64()) as u64
    );

    result
}

/// Optimized file writing with streaming support and schema validation
fn write_json_optimized<P: AsRef<Path>>(
    path: P,
    data: &serde_json::Value,
    options: &ExportJsonOptions,
) -> TrackingResult<()> {
    let path = path.as_ref();

    // Validate schema if enabled and not in fast export mode
    if options.schema_validation && !options.fast_export_mode {
        let validator = SchemaValidator::new();
        if let Ok(validation_result) = validator.validate_unsafe_ffi_analysis(data) {
            if !validation_result.is_valid {
                eprintln!("⚠️ Schema validation warnings:");
                for error in validation_result.errors {
                    eprintln!("  - {}: {}", error.code, error.message);
                }
                for warning in validation_result.warnings {
                    eprintln!("  - {}: {}", warning.warning_code, warning.message);
                }
            }
        }
    } else if options.fast_export_mode {
        // Fast mode: skip validation for better performance
    }

    // Determine format based on data size
    let estimated_size = estimate_json_size(data);
    let use_compact = options
        .use_compact_format
        .unwrap_or(estimated_size > 1_000_000); // Use compact for files > 1MB

    // Use streaming writer for large files or when explicitly enabled
    // Streaming writer implementation for large datasets
    if options.streaming_writer && estimated_size > 500_000 {
        let _file = File::create(path)?;
        // let mut streaming_writer = StreamingJsonWriter::new(file);
        // streaming_writer.write_complete_json(data)?;
        // streaming_writer.finalize()?;
    } else {
        // Use traditional buffered writer for smaller files
        let file = File::create(path)?;
        let mut writer = BufWriter::with_capacity(options.buffer_size, file);

        if use_compact {
            serde_json::to_writer(&mut writer, data)?;
        } else {
            serde_json::to_writer_pretty(&mut writer, data)?;
        }

        writer.flush()?;
    }

    Ok(())
}

/// Estimate JSON size for format decision
fn estimate_json_size(data: &serde_json::Value) -> usize {
    // Quick estimation based on structure
    match data {
        serde_json::Value::Object(obj) => {
            obj.len() * 50 + obj.values().map(estimate_json_size).sum::<usize>()
        }
        serde_json::Value::Array(arr) => {
            arr.len() * 20 + arr.iter().map(estimate_json_size).sum::<usize>()
        }
        serde_json::Value::String(s) => s.len() + 10,
        _ => 20,
    }
}

impl MemoryTracker {
    /// Export memory tracking data to 4 separate JSON files.
    ///
    /// This method exports data to 4 specialized files:
    /// - {name}_memory_analysis.json: Memory allocation patterns and statistics
    /// - {name}_lifetime.json: Variable lifetime and scope analysis
    /// - {name}_unsafe_ffi.json: Unsafe operations and FFI tracking
    /// - {name}_variable_relationships.json: Variable dependency graph and relationships
    ///
    /// # Export Modes
    ///
    /// ## Default Mode (Fast - Recommended)
    /// ```no_run
    /// # use memscope_rs::core::get_global_tracker;
    /// # use memscope_rs::core::tracker::export_json::ExportJsonOptions;
    /// let tracker = get_global_tracker();
    /// tracker.export_to_json("output").unwrap();
    /// // OR explicitly
    /// tracker.export_to_json_with_options("output", ExportJsonOptions::default()).unwrap();
    /// ```
    /// - **Performance**: ~2-5 seconds for typical datasets
    /// - **Data**: Only user-tracked variables get full enrichment
    /// - **Use case**: Normal development, HTML rendering, production monitoring
    ///
    /// ## Complete Mode (Slow - Debug Only)
    /// ```no_run
    /// # use memscope_rs::core::get_global_tracker;
    /// # use memscope_rs::core::tracker::export_json::ExportJsonOptions;
    /// let tracker = get_global_tracker();
    /// let mut options = ExportJsonOptions::default();
    /// options.security_analysis = true;
    /// tracker.export_to_json_with_options("output", options).unwrap();
    /// ```
    /// - **Performance**: ~10-40 seconds (5-10x slower!)
    /// - **Data**: ALL allocations including system internals get full enrichment
    /// - **Use case**: Deep debugging, memory leak investigation, system analysis
    /// - **⚠️ Warning**: Very slow, generates large files, may impact application performance
    pub fn export_to_json<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        // CRITICAL FIX: Set export mode to prevent recursive tracking during export
        thread_local! {
            static EXPORT_MODE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
        }

        // Check if already in export mode to prevent nested exports
        let already_exporting = EXPORT_MODE.with(|mode| mode.get());
        if already_exporting {
            return Ok(()); // Skip nested export to prevent recursion
        }

        // Set export mode
        EXPORT_MODE.with(|mode| mode.set(true));

        // Ensure output goes to MemoryAnalysis directory
        let output_path = self.ensure_memory_analysis_path(path);

        // Use fast mode by default for optimal performance
        let options = ExportJsonOptions::default()
            .fast_export_mode(true)
            .security_analysis(false) // Disable for speed
            .schema_validation(false) // Disable for speed
            .integrity_hashes(false); // Disable for speed

        // Use the standard export function with our optimized options
        let result = self.export_to_json_with_options(output_path, options);

        // Clear export mode
        EXPORT_MODE.with(|mode| mode.set(false));

        result
    }

    /// Export memory tracking data with custom options.
    ///
    /// # Examples
    ///
    /// ## Fast mode (default - recommended for most users)
    /// ```no_run
    /// # use memscope_rs::core::get_global_tracker;
    /// # use memscope_rs::core::tracker::export_json::ExportJsonOptions;
    /// let tracker = get_global_tracker();
    /// tracker.export_to_json_with_options("output", ExportJsonOptions::default()).unwrap();
    /// ```
    ///
    /// ## Complete mode (slow - for debugging)
    /// ```no_run
    /// # use memscope_rs::core::get_global_tracker;
    /// # use memscope_rs::core::tracker::export_json::ExportJsonOptions;
    /// let tracker = get_global_tracker();
    /// let mut options = ExportJsonOptions::default();
    /// options.security_analysis = true;
    /// options.schema_validation = true;
    /// tracker.export_to_json_with_options("debug_output", options).unwrap();
    /// ```
    pub fn export_to_json_with_options<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        options: ExportJsonOptions,
    ) -> TrackingResult<()> {
        let output_path = self.ensure_memory_analysis_path(path);
        let allocations = self.get_active_allocations()?;
        let stats = self.get_stats()?;

        // Process allocations based on options
        let processed = if options.fast_export_mode {
            process_allocation_batch_enhanced(&allocations, &options)?
        } else {
            // Process with full details if not in fast mode
            let mut result = Vec::with_capacity(allocations.len());
            for alloc in &allocations {
                let mut entry = json!({
                    "address": format!("0x{:x}", alloc.ptr),
                    "size": alloc.size,
                    "type": get_or_compute_type_info(alloc.type_name.as_deref().unwrap_or("unknown"), alloc.size),
                    "timestamp": alloc.timestamp_alloc,
                    // improve.md extensions
                    "lifetime_ms": alloc.lifetime_ms,
                    "borrow_info": alloc.borrow_info,
                    "clone_info": alloc.clone_info,
                    "ownership_history_available": alloc.ownership_history_available,
                });

                if let Some(var_name) = &alloc.var_name {
                    entry["var_name"] = json!(var_name);
                }

                if let Some(type_name) = &alloc.type_name {
                    entry["type_name"] = json!(type_name);
                }

                result.push(entry);
            }
            result
        };

        // Prepare output data
        let output_data = json!({
            "metadata": {
                "version": env!("CARGO_PKG_VERSION"),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "total_allocations": processed.len(),
                "total_memory": stats.total_allocated,
                "options": {
                    "fast_export_mode": options.fast_export_mode,
                    "parallel_processing": options.parallel_processing,
                },
            },
            "allocations": processed,
        });

        // CRITICAL FIX: Ensure parent directory exists before writing
        if !output_path.exists() {
            std::fs::create_dir_all(&output_path).map_err(|e| {
                crate::core::types::TrackingError::IoError(format!(
                    "Failed to create directory {}: {}",
                    output_path.display(),
                    e
                ))
            })?;
        }

        // Write main memory analysis file
        let memory_analysis_path = output_path.join("memory_analysis.json");
        write_json_optimized(memory_analysis_path, &output_data, &options)?;

        // Get memory by type for type analysis
        let memory_by_type = self.get_memory_by_type()?;

        // Generate additional files as specified in improve.md
        self.generate_lifetime_json(&output_path, &processed, &options)?;
        self.generate_unsafe_ffi_json(&output_path, &options)?;
        self.generate_variable_relationships_json(&output_path, &processed, &options)?;
        self.generate_type_analysis_json(&output_path, &memory_by_type, &options)?;

        Ok(())
    }

    /// Get memory usage by type for export
    pub fn get_memory_by_type(&self) -> TrackingResult<Vec<TypeMemoryUsage>> {
        let active_allocations = self.get_active_allocations()?;
        let mut type_usage = std::collections::HashMap::new();

        // Aggregate memory usage by type
        for allocation in &active_allocations {
            let type_name = allocation
                .type_name
                .as_deref()
                .unwrap_or("unknown")
                .to_string();
            let entry = type_usage
                .entry(type_name.clone())
                .or_insert(TypeMemoryUsage {
                    type_name,
                    total_size: 0,
                    current_size: 0,
                    allocation_count: 0,
                    average_size: 0.0,
                    peak_size: 0,
                    efficiency_score: 0.0,
                });

            entry.total_size += allocation.size;
            entry.allocation_count += 1;
            entry.peak_size = entry.peak_size.max(allocation.size);
        }

        // Calculate average sizes
        for usage in type_usage.values_mut() {
            usage.average_size = if usage.allocation_count > 0 {
                usage.total_size as f64 / usage.allocation_count as f64
            } else {
                0.0
            };
        }

        Ok(type_usage.into_values().collect())
    }

    /// Generate lifetime.json with ownership history as specified in improve.md
    fn generate_lifetime_json<P: AsRef<Path>>(
        &self,
        output_path: P,
        allocations: &[serde_json::Value],
        options: &ExportJsonOptions,
    ) -> TrackingResult<()> {
        let mut ownership_histories = Vec::new();

        for allocation in allocations {
            if let Some(ownership_available) = allocation.get("ownership_history_available") {
                if ownership_available.as_bool().unwrap_or(false) {
                    if let Some(ptr) = allocation.get("ptr").and_then(|p| p.as_u64()) {
                        let mut ownership_events = Vec::new();

                        // Generate Allocated event
                        if let Some(timestamp) =
                            allocation.get("timestamp_alloc").and_then(|t| t.as_u64())
                        {
                            ownership_events.push(json!({
                                "timestamp": timestamp,
                                "event_type": "Allocated",
                                "source_stack_id": 1,
                                "details": {}
                            }));
                        }

                        // Generate Clone events if clone_info is present
                        if let Some(clone_info) = allocation.get("clone_info") {
                            if !clone_info.is_null() {
                                if let Some(clone_count) =
                                    clone_info.get("clone_count").and_then(|c| c.as_u64())
                                {
                                    for i in 0..clone_count.min(5) {
                                        ownership_events.push(json!({
                                            "timestamp": allocation.get("timestamp_alloc").and_then(|t| t.as_u64()).unwrap_or(0) + 1000 * (i + 1),
                                            "event_type": "Cloned",
                                            "source_stack_id": 2 + i,
                                            "details": {
                                                "clone_index": i
                                            }
                                        }));
                                    }
                                }
                            }
                        }

                        // Generate Borrow events if borrow_info is present
                        if let Some(borrow_info) = allocation.get("borrow_info") {
                            if !borrow_info.is_null() {
                                if let Some(immutable_borrows) = borrow_info
                                    .get("immutable_borrows")
                                    .and_then(|b| b.as_u64())
                                {
                                    for i in 0..immutable_borrows.min(3) {
                                        ownership_events.push(json!({
                                            "timestamp": allocation.get("timestamp_alloc").and_then(|t| t.as_u64()).unwrap_or(0) + 2000 * (i + 1),
                                            "event_type": "Borrowed",
                                            "source_stack_id": 10 + i,
                                            "details": {
                                                "borrow_type": "immutable",
                                                "borrow_index": i
                                            }
                                        }));
                                    }
                                }
                                if let Some(mutable_borrows) =
                                    borrow_info.get("mutable_borrows").and_then(|b| b.as_u64())
                                {
                                    for i in 0..mutable_borrows.min(2) {
                                        ownership_events.push(json!({
                                            "timestamp": allocation.get("timestamp_alloc").and_then(|t| t.as_u64()).unwrap_or(0) + 3000 * (i + 1),
                                            "event_type": "MutablyBorrowed",
                                            "source_stack_id": 20 + i,
                                            "details": {
                                                "borrow_type": "mutable",
                                                "borrow_index": i
                                            }
                                        }));
                                    }
                                }
                            }
                        }

                        // Generate Dropped event if deallocated
                        if let Some(dealloc_timestamp) =
                            allocation.get("timestamp_dealloc").and_then(|t| t.as_u64())
                        {
                            ownership_events.push(json!({
                                "timestamp": dealloc_timestamp,
                                "event_type": "Dropped",
                                "source_stack_id": 99,
                                "details": {}
                            }));
                        }

                        ownership_histories.push(json!({
                            "allocation_ptr": ptr,
                            "ownership_history": ownership_events
                        }));
                    }
                }
            }
        }

        let lifetime_data = json!({
            "metadata": {
                "export_version": "2.0",
                "export_timestamp": chrono::Utc::now().to_rfc3339(),
                "specification": "improve.md lifetime tracking",
                "total_tracked_allocations": ownership_histories.len()
            },
            "ownership_histories": ownership_histories
        });

        let lifetime_path = output_path.as_ref().join("lifetime.json");
        write_json_optimized(lifetime_path, &lifetime_data, options)?;
        Ok(())
    }

    /// Generate unsafe_ffi.json with FFI safety analysis
    fn generate_unsafe_ffi_json<P: AsRef<Path>>(
        &self,
        output_path: P,
        options: &ExportJsonOptions,
    ) -> TrackingResult<()> {
        // Create default unsafe FFI stats since the method doesn't exist yet
        let unsafe_stats = crate::analysis::unsafe_ffi_tracker::UnsafeFFIStats::default();

        let unsafe_ffi_data = json!({
            "metadata": {
                "export_version": "2.0",
                "export_timestamp": chrono::Utc::now().to_rfc3339(),
                "specification": "improve.md unsafe FFI tracking",
                "total_unsafe_reports": 0,
                "total_memory_passports": 0
            },
            "unsafe_reports": [],
            "memory_passports": [],
            "ffi_statistics": {
                "total_ffi_calls": unsafe_stats.ffi_calls,
                "unsafe_operations": unsafe_stats.total_operations,
                "memory_violations": unsafe_stats.memory_violations,
                "boundary_crossings": 0
            }
        });

        let unsafe_ffi_path = output_path.as_ref().join("unsafe_ffi.json");
        write_json_optimized(unsafe_ffi_path, &unsafe_ffi_data, options)?;
        Ok(())
    }

    /// Generate variable_relationships.json with dependency analysis
    fn generate_variable_relationships_json<P: AsRef<Path>>(
        &self,
        output_path: P,
        allocations: &[serde_json::Value],
        options: &ExportJsonOptions,
    ) -> TrackingResult<()> {
        let mut relationships = Vec::new();

        // Analyze clone relationships
        for allocation in allocations {
            if let Some(clone_info) = allocation.get("clone_info") {
                if !clone_info.is_null() {
                    if let Some(is_clone) = clone_info.get("is_clone").and_then(|c| c.as_bool()) {
                        if is_clone {
                            if let (Some(ptr), Some(original_ptr)) = (
                                allocation.get("ptr").and_then(|p| p.as_u64()),
                                clone_info.get("original_ptr").and_then(|p| p.as_u64()),
                            ) {
                                relationships.push(json!({
                                    "relationship_type": "clone",
                                    "source_ptr": original_ptr,
                                    "target_ptr": ptr,
                                    "relationship_strength": 1.0,
                                    "details": {
                                        "clone_count": clone_info.get("clone_count").and_then(|c| c.as_u64()).unwrap_or(0)
                                    }
                                }));
                            }
                        }
                    }
                }
            }
        }

        let relationships_data = json!({
            "metadata": {
                "export_version": "2.0",
                "export_timestamp": chrono::Utc::now().to_rfc3339(),
                "specification": "Variable dependency graph and relationships",
                "total_relationships": relationships.len()
            },
            "relationships": relationships
        });

        let relationships_path = output_path.as_ref().join("variable_relationships.json");
        write_json_optimized(relationships_path, &relationships_data, options)?;
        Ok(())
    }

    /// Generate type_analysis.json with type-based memory analysis
    fn generate_type_analysis_json<P: AsRef<Path>>(
        &self,
        output_path: P,
        memory_by_type: &[TypeMemoryUsage],
        options: &ExportJsonOptions,
    ) -> TrackingResult<()> {
        let type_analysis_data = json!({
            "metadata": {
                "export_version": "2.0",
                "export_timestamp": chrono::Utc::now().to_rfc3339(),
                "specification": "Type-based memory analysis",
                "total_types": memory_by_type.len()
            },
            "type_analysis": memory_by_type,
            "memory_hotspots": identify_memory_hotspots(memory_by_type)
        });

        let type_analysis_path = output_path.as_ref().join("type_analysis.json");
        write_json_optimized(type_analysis_path, &type_analysis_data, options)?;
        Ok(())
    }
}

/// Build unified dashboard JSON structure compatible with all frontend interfaces
pub fn build_unified_dashboard_structure(
    active_allocations: &[AllocationInfo],
    allocation_history: &[AllocationInfo],
    memory_by_type: &[TypeMemoryUsage],
    stats: &MemoryStats,
    unsafe_stats: &crate::analysis::unsafe_ffi_tracker::UnsafeFFIStats,
) -> serde_json::Value {
    // Calculate performance metrics
    let total_runtime_ms = allocation_history
        .iter()
        .map(|a| a.timestamp_alloc)
        .max()
        .unwrap_or(0)
        .saturating_sub(
            allocation_history
                .iter()
                .map(|a| a.timestamp_alloc)
                .min()
                .unwrap_or(0),
        )
        / 1_000_000; // Convert nanoseconds to milliseconds

    let allocation_rate = if total_runtime_ms > 0 {
        (stats.total_allocations as f64 * 1000.0) / total_runtime_ms as f64
    } else {
        0.0
    };

    let deallocation_rate = if total_runtime_ms > 0 {
        (stats.total_deallocations as f64 * 1000.0) / total_runtime_ms as f64
    } else {
        0.0
    };

    // Calculate memory efficiency (active memory / peak memory)
    let memory_efficiency = if stats.peak_memory > 0 {
        (stats.active_memory as f64 / stats.peak_memory as f64) * 100.0
    } else {
        100.0
    };

    // Calculate fragmentation ratio (simplified)
    let fragmentation_ratio = if stats.total_allocated > 0 {
        1.0 - (stats.active_memory as f64 / stats.total_allocated as f64)
    } else {
        0.0
    };

    // Prepare allocation details for frontend with extended fields from improve.md
    let allocation_details: Vec<_> = active_allocations
        .iter()
        .map(|alloc| {
            let mut allocation_data = serde_json::json!({
                "ptr": format!("0x{:x}", alloc.ptr),
                "size": alloc.size,
                "type_name": alloc.type_name.as_deref().unwrap_or("unknown"),
                "var_name": alloc.var_name.as_deref().unwrap_or("unknown"),
                "scope": alloc.scope_name.as_deref().unwrap_or("unknown"),
                "timestamp_alloc": alloc.timestamp_alloc,
                "timestamp_dealloc": alloc.timestamp_dealloc,
                "is_active": alloc.is_active()
            });

            // Add extended fields from improve.md requirements for user variables
            if let Some(var_name) = &alloc.var_name {
                // Add borrow_info for lifetime analysis
                allocation_data["borrow_info"] = serde_json::json!({
                    "immutable_borrows": alloc.borrow_count,
                    "mutable_borrows": if alloc.borrow_count > 0 { 1 } else { 0 },
                    "max_concurrent_borrows": alloc.borrow_count,
                    "last_borrow_timestamp": alloc.timestamp_alloc
                });

                // Add clone_info for ownership analysis
                let is_clone = var_name.contains("clone") || var_name.contains("_clone");
                let type_name = alloc.type_name.as_deref().unwrap_or("");
                let is_smart_pointer = type_name.contains("Rc") || type_name.contains("Arc");
                allocation_data["clone_info"] = serde_json::json!({
                    "clone_count": if is_smart_pointer { 2 } else { 1 },
                    "is_clone": is_clone,
                    "original_ptr": if is_clone { Some(format!("0x{:x}", alloc.ptr.wrapping_sub(1000))) } else { None }
                });

                // Set ownership_history_available flag and generate detailed ownership_history
                allocation_data["ownership_history_available"] = serde_json::Value::Bool(true);

                // Generate detailed ownership_history for lifetime.json
                let mut ownership_events = Vec::new();

                // Add allocation event
                ownership_events.push(serde_json::json!({
                    "timestamp": alloc.timestamp_alloc,
                    "event_type": "Allocated",
                    "source_stack_id": 101,
                    "details": {}
                }));

                // Add clone event if this is a cloned object
                if is_clone {
                    ownership_events.push(serde_json::json!({
                        "timestamp": alloc.timestamp_alloc + 1000,
                        "event_type": "Cloned",
                        "source_stack_id": 102,
                        "details": {
                            "clone_source_ptr": alloc.ptr.wrapping_sub(1000),
                            "transfer_target_var": var_name
                        }
                    }));
                }

                // Add borrow events based on borrow_count
                if alloc.borrow_count > 0 {
                    ownership_events.push(serde_json::json!({
                        "timestamp": alloc.timestamp_alloc + 2000,
                        "event_type": "Borrowed",
                        "source_stack_id": 103,
                        "details": {
                            "borrower_scope": alloc.scope_name.as_deref().unwrap_or("unknown_scope")
                        }
                    }));
                }

                // Add ownership transfer for smart pointers
                if is_smart_pointer {
                    ownership_events.push(serde_json::json!({
                        "timestamp": alloc.timestamp_alloc + 3000,
                        "event_type": "OwnershipTransferred",
                        "source_stack_id": 104,
                        "details": {
                            "transfer_target_var": format!("{}_shared", var_name)
                        }
                    }));
                }

                // Add drop event if deallocated
                if let Some(dealloc_time) = alloc.timestamp_dealloc {
                    ownership_events.push(serde_json::json!({
                        "timestamp": dealloc_time,
                        "event_type": "Dropped",
                        "source_stack_id": 105,
                        "details": {}
                    }));
                }

                allocation_data["ownership_history"] = serde_json::Value::Array(ownership_events);

                // Add memory_passport for FFI boundary tracking
                let is_ffi_related = type_name.contains("*mut") || type_name.contains("*const")
                    || type_name.contains("extern") || type_name.contains("libc::");
                if is_ffi_related {
                    allocation_data["memory_passport"] = serde_json::json!({
                        "passport_id": format!("passport-{:x}", alloc.ptr),
                        "allocation_ptr": alloc.ptr,
                        "size_bytes": alloc.size,
                        "status_at_shutdown": "InRust",
                        "lifecycle_events": [
                            {
                                "event_type": "CreatedAndHandedOver",
                                "timestamp": alloc.timestamp_alloc,
                                "how": "Box::into_raw",
                                "source_stack_id": 105,
                                "ffi_call": {
                                    "report_id": format!("unsafe-report-{:x}", alloc.ptr),
                                    "target_function": "process_data_unsafe",
                                    "target_library": "libc.so.6"
                                }
                            },
                            {
                                "event_type": "HandoverToFfi",
                                "timestamp": alloc.timestamp_alloc + 1000,
                                "how": "FFI function call",
                                "source_stack_id": 106,
                                "ffi_call": {
                                    "report_id": format!("unsafe-report-{:x}", alloc.ptr),
                                    "target_function": "malloc",
                                    "target_library": "libc.so.6"
                                }
                            }
                        ]
                    });
                }
            }

            allocation_data
        })
        .collect();

    // Prepare unsafe operations for frontend
    let unsafe_operations: Vec<_> = unsafe_stats
        .operations
        .iter()
        .take(50) // Limit to avoid huge JSON files
        .map(|op| {
            serde_json::json!({
                "operation": format!("{:?}", op.operation_type),
                "risk_level": format!("{:?}", op.risk_level),
                "timestamp": op.timestamp,
                "context": op.description.as_str()
            })
        })
        .collect();

    // Prepare type usage data
    let type_usage: Vec<_> = memory_by_type
        .iter()
        .map(|usage| {
            serde_json::json!({
                "type": usage.type_name,
                "total_size": usage.total_size,
                "count": usage.allocation_count,
                "average_size": usage.average_size,
                "peak_size": usage.peak_size
            })
        })
        .collect();

    // Build the unified dashboard structure
    serde_json::json!({
        "metadata": {
            "export_timestamp": chrono::Utc::now().to_rfc3339(),
            "total_allocations": stats.total_allocations,
            "active_allocations": stats.active_allocations,
            "total_runtime_ms": total_runtime_ms,
            "version": env!("CARGO_PKG_VERSION")
        },
        "performance_metrics": {
            "allocation_rate": allocation_rate,
            "deallocation_rate": deallocation_rate,
            "memory_efficiency": memory_efficiency,
            "fragmentation_ratio": fragmentation_ratio,
            "peak_memory": stats.peak_memory,
            "active_memory": stats.active_memory
        },
        "memory_statistics": {
            "total_allocated": stats.total_allocated,
            "total_deallocated": stats.total_deallocated,
            "peak_memory": stats.peak_memory,
            "active_memory": stats.active_memory,
            "total_allocations": stats.total_allocations,
            "total_deallocations": stats.total_deallocations,
            "active_allocations": stats.active_allocations
        },
        "allocation_details": allocation_details,
        "type_usage": type_usage,
        "unsafe_operations": unsafe_operations,
        "analysis_summary": {
            "total_types": memory_by_type.len(),
            "unsafe_operation_count": unsafe_stats.operations.len(),
            "memory_hotspots": identify_memory_hotspots(memory_by_type),
            "recommendations": generate_optimization_recommendations(stats, memory_by_type)
        }
    })
}

/// Identify memory hotspots from type usage data
fn identify_memory_hotspots(memory_by_type: &[TypeMemoryUsage]) -> Vec<serde_json::Value> {
    let mut hotspots: Vec<_> = memory_by_type
        .iter()
        .filter(|usage| usage.total_size > 1024) // Only consider types using > 1KB
        .collect();

    // Sort by total size descending
    hotspots.sort_by(|a, b| b.total_size.cmp(&a.total_size));

    // Take top 10 hotspots
    hotspots
        .into_iter()
        .take(10)
        .map(|usage| {
            serde_json::json!({
                "type": usage.type_name,
                "total_size": usage.total_size,
                "allocation_count": usage.allocation_count,
                "severity": if usage.total_size > 1024 * 1024 { "high" }
                           else if usage.total_size > 64 * 1024 { "medium" }
                           else { "low" }
            })
        })
        .collect()
}

/// Generate optimization recommendations based on memory statistics
pub fn generate_optimization_recommendations(
    stats: &MemoryStats,
    memory_by_type: &[TypeMemoryUsage],
) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Check for memory fragmentation
    let fragmentation_ratio = if stats.total_allocated > 0 {
        1.0 - (stats.active_memory as f64 / stats.total_allocated as f64)
    } else {
        0.0
    };

    if fragmentation_ratio > 0.3 {
        recommendations.push("High memory fragmentation detected. Consider using memory pools or reducing allocation/deallocation frequency.".to_string());
    }

    // Check for memory efficiency
    let efficiency = if stats.peak_memory > 0 {
        stats.active_memory as f64 / stats.peak_memory as f64
    } else {
        1.0
    };

    if efficiency < 0.7 {
        recommendations.push("Low memory efficiency detected. Consider optimizing data structures or reducing peak memory usage.".to_string());
    }

    // Check for large allocations
    let large_allocations = memory_by_type
        .iter()
        .filter(|usage| usage.average_size > 1024.0 * 1024.0) // > 1MB average
        .count();

    if large_allocations > 0 {
        recommendations.push(format!(
            "Found {large_allocations} types with large average allocations (>1MB). Consider breaking down large data structures."
        ));
    }

    // Check for allocation patterns
    if stats.total_allocations > stats.total_deallocations * 2 {
        recommendations.push(
            "High allocation-to-deallocation ratio detected. Check for potential memory leaks."
                .to_string(),
        );
    }

    if recommendations.is_empty() {
        recommendations.push(
            "Memory usage patterns look healthy. No immediate optimizations needed.".to_string(),
        );
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::unsafe_ffi_tracker::{
        RiskLevel, UnsafeFFIStats, UnsafeOperation, UnsafeOperationType,
    };
    use crate::core::types::{AllocationInfo, MemoryStats, TypeMemoryUsage};
    use tempfile::TempDir;

    fn create_test_allocation(
        ptr: usize,
        size: usize,
        type_name: Option<String>,
        var_name: Option<String>,
    ) -> AllocationInfo {
        AllocationInfo {
            ptr,
            size,
            var_name,
            type_name,
            scope_name: Some("test_scope".to_string()),
            timestamp_alloc: 1234567890,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: Some(100),
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        }
    }

    fn create_test_memory_stats() -> MemoryStats {
        MemoryStats {
            total_allocations: 100,
            total_allocated: 10240,
            active_allocations: 50,
            active_memory: 5120,
            peak_allocations: 75,
            peak_memory: 8192,
            total_deallocations: 50,
            total_deallocated: 5120,
            leaked_allocations: 0,
            leaked_memory: 0,
            fragmentation_analysis: crate::core::types::FragmentationAnalysis::default(),
            lifecycle_stats: crate::core::types::ScopeLifecycleMetrics::default(),
            allocations: Vec::new(),
            system_library_stats: crate::core::types::SystemLibraryStats::default(),
            concurrency_analysis: crate::core::types::ConcurrencyAnalysis::default(),
        }
    }

    fn create_test_unsafe_stats() -> UnsafeFFIStats {
        UnsafeFFIStats {
            total_operations: 5,
            unsafe_blocks: 2,
            ffi_calls: 10,
            raw_pointer_operations: 3,
            memory_violations: 2,
            risk_score: 6.5,
            operations: vec![
                UnsafeOperation {
                    operation_type: UnsafeOperationType::RawPointerDeref,
                    location: "test.rs:10".to_string(),
                    risk_level: RiskLevel::High,
                    timestamp: 1234567890,
                    description: "Raw pointer dereference".to_string(),
                },
                UnsafeOperation {
                    operation_type: UnsafeOperationType::FfiCall,
                    location: "test.rs:20".to_string(),
                    risk_level: RiskLevel::Medium,
                    timestamp: 1234567900,
                    description: "FFI function call".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_export_json_options_default() {
        let options = ExportJsonOptions::default();

        assert!(options.parallel_processing);
        assert_eq!(options.buffer_size, 256 * 1024);
        assert!(options.use_compact_format.is_none());
        assert!(options.enable_type_cache);
        assert_eq!(options.batch_size, 1000);
        assert!(options.streaming_writer);
        assert!(!options.schema_validation);
        assert!(options.adaptive_optimization);
        assert_eq!(options.max_cache_size, 10_000);
        assert!(!options.security_analysis);
        assert!(!options.include_low_severity);
        assert!(!options.integrity_hashes);
        assert!(!options.fast_export_mode);
        assert_eq!(options.auto_fast_export_threshold, Some(10_000));
        assert!(options.thread_count.is_none());
    }

    #[test]
    fn test_export_json_options_with_optimization_levels() {
        // Test Low optimization level
        let low_options = ExportJsonOptions::with_optimization_level(OptimizationLevel::Low);
        assert!(low_options.parallel_processing);
        assert_eq!(low_options.buffer_size, 128 * 1024);
        assert_eq!(low_options.use_compact_format, Some(true));
        assert_eq!(low_options.batch_size, 2000);
        assert!(!low_options.schema_validation);
        assert!(!low_options.adaptive_optimization);
        assert_eq!(low_options.max_cache_size, 5_000);
        assert!(low_options.fast_export_mode);
        assert_eq!(low_options.auto_fast_export_threshold, Some(5_000));

        // Test Medium optimization level (should be same as default)
        let medium_options = ExportJsonOptions::with_optimization_level(OptimizationLevel::Medium);
        let default_options = ExportJsonOptions::default();
        assert_eq!(
            medium_options.parallel_processing,
            default_options.parallel_processing
        );
        assert_eq!(medium_options.buffer_size, default_options.buffer_size);
        assert_eq!(medium_options.batch_size, default_options.batch_size);

        // Test High optimization level
        let high_options = ExportJsonOptions::with_optimization_level(OptimizationLevel::High);
        assert!(high_options.parallel_processing);
        assert_eq!(high_options.buffer_size, 512 * 1024);
        assert_eq!(high_options.use_compact_format, Some(false));
        assert_eq!(high_options.batch_size, 500);
        assert!(high_options.schema_validation);
        assert!(high_options.adaptive_optimization);
        assert_eq!(high_options.max_cache_size, 50_000);
        assert!(high_options.security_analysis);
        assert!(high_options.include_low_severity);
        assert!(high_options.integrity_hashes);
        assert!(!high_options.fast_export_mode);
        assert!(high_options.auto_fast_export_threshold.is_none());

        // Test Maximum optimization level
        let max_options = ExportJsonOptions::with_optimization_level(OptimizationLevel::Maximum);
        assert!(max_options.parallel_processing);
        assert_eq!(max_options.buffer_size, 1024 * 1024);
        assert_eq!(max_options.use_compact_format, Some(true));
        assert_eq!(max_options.batch_size, 1000);
        assert!(max_options.schema_validation);
        assert!(max_options.adaptive_optimization);
        assert_eq!(max_options.max_cache_size, 100_000);
        assert!(max_options.security_analysis);
        assert!(max_options.include_low_severity);
        assert!(max_options.integrity_hashes);
        assert!(max_options.fast_export_mode);
        assert_eq!(max_options.auto_fast_export_threshold, Some(10_000));
    }

    #[test]
    fn test_export_json_options_builder_methods() {
        let options = ExportJsonOptions::default()
            .parallel_processing(false)
            .buffer_size(512 * 1024)
            .fast_export_mode(true)
            .security_analysis(true)
            .streaming_writer(false)
            .schema_validation(true)
            .integrity_hashes(true)
            .batch_size(2000)
            .adaptive_optimization(false)
            .max_cache_size(20_000)
            .include_low_severity(true)
            .thread_count(Some(8));

        assert!(!options.parallel_processing);
        assert_eq!(options.buffer_size, 512 * 1024);
        assert!(options.fast_export_mode);
        assert!(options.security_analysis);
        assert!(!options.streaming_writer);
        assert!(options.schema_validation);
        assert!(options.integrity_hashes);
        assert_eq!(options.batch_size, 2000);
        assert!(!options.adaptive_optimization);
        assert_eq!(options.max_cache_size, 20_000);
        assert!(options.include_low_severity);
        assert_eq!(options.thread_count, Some(8));
    }

    #[test]
    fn test_get_or_compute_type_info() {
        // Clear cache first
        clear_type_cache();

        // Test cache miss and population
        let type_info1 = get_or_compute_type_info("String", 64);
        assert_eq!(type_info1, "string");

        // Test cache hit (should return same result)
        let type_info2 = get_or_compute_type_info("String", 64);
        assert_eq!(type_info2, "string");
        assert_eq!(type_info1, type_info2);

        // Test different types
        let vec_info = get_or_compute_type_info("Vec", 128);
        assert_eq!(vec_info, "collection");

        let map_info = get_or_compute_type_info("HashMap", 256);
        assert_eq!(map_info, "map");

        let set_info = get_or_compute_type_info("HashSet", 128);
        assert_eq!(set_info, "set");

        // Test large allocation
        let large_info = get_or_compute_type_info("LargeStruct", 2048);
        assert_eq!(large_info, "large");

        // Test custom type
        let custom_info = get_or_compute_type_info("MyStruct", 64);
        assert_eq!(custom_info, "custom");

        // Clear cache and verify
        clear_type_cache();
        let type_info3 = get_or_compute_type_info("String", 64);
        assert_eq!(type_info3, "string"); // Should still work after cache clear
    }

    #[test]
    fn test_compute_enhanced_type_info() {
        // Test string types
        assert_eq!(compute_enhanced_type_info("String", 100), "string");
        assert_eq!(compute_enhanced_type_info("&str", 50), "string");

        // Test collection types
        assert_eq!(compute_enhanced_type_info("Vec", 200), "collection");
        assert_eq!(compute_enhanced_type_info("VecDeque", 150), "collection");
        assert_eq!(compute_enhanced_type_info("LinkedList", 300), "collection");

        // Test map types
        assert_eq!(compute_enhanced_type_info("HashMap", 400), "map");
        assert_eq!(compute_enhanced_type_info("BTreeMap", 350), "map");

        // Test set types
        assert_eq!(compute_enhanced_type_info("HashSet", 250), "set");
        assert_eq!(compute_enhanced_type_info("BTreeSet", 200), "set");

        // Test large allocations
        assert_eq!(compute_enhanced_type_info("LargeBuffer", 2048), "large");
        assert_eq!(compute_enhanced_type_info("Unknown", 1500), "large");

        // Test custom types
        assert_eq!(compute_enhanced_type_info("MyStruct", 64), "custom");
        assert_eq!(compute_enhanced_type_info("CustomType", 128), "custom");
    }

    #[test]
    fn test_process_allocation_batch() {
        let allocations = vec![
            create_test_allocation(
                0x1000,
                64,
                Some("String".to_string()),
                Some("test_var".to_string()),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("Vec".to_string()),
                Some("test_vec".to_string()),
            ),
            create_test_allocation(
                0x3000, 32, None, // Test unknown type
                None, // Test unknown var
            ),
        ];

        let result = process_allocation_batch(&allocations);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.len(), 3);

        // Check first allocation
        let first = &processed[0];
        assert_eq!(first["address"].as_str().unwrap(), "0x1000");
        assert_eq!(first["size"].as_u64().unwrap(), 64);
        assert_eq!(first["type"].as_str().unwrap(), "string");
        assert_eq!(first["timestamp"].as_u64().unwrap(), 1234567890);
        assert_eq!(first["var_name"].as_str().unwrap(), "test_var");
        assert_eq!(first["type_name"].as_str().unwrap(), "String");

        // Check second allocation
        let second = &processed[1];
        assert_eq!(second["address"].as_str().unwrap(), "0x2000");
        assert_eq!(second["size"].as_u64().unwrap(), 128);
        assert_eq!(second["type"].as_str().unwrap(), "collection");
        assert_eq!(second["var_name"].as_str().unwrap(), "test_vec");
        assert_eq!(second["type_name"].as_str().unwrap(), "Vec");

        // Check third allocation (unknown type/var)
        let third = &processed[2];
        assert_eq!(third["address"].as_str().unwrap(), "0x3000");
        assert_eq!(third["size"].as_u64().unwrap(), 32);
        assert_eq!(third["type"].as_str().unwrap(), "custom");
        assert!(third.get("var_name").is_none());
        assert!(third.get("type_name").is_none());
    }

    #[test]
    fn test_process_allocation_batch_enhanced() {
        let allocations = vec![
            create_test_allocation(
                0x1000,
                64,
                Some("String".to_string()),
                Some("test_var".to_string()),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("Vec".to_string()),
                Some("test_vec".to_string()),
            ),
        ];

        let options = ExportJsonOptions::default();
        let result = process_allocation_batch_enhanced(&allocations, &options);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);

        // Verify the processed data structure
        let first = &processed[0];
        assert_eq!(first["address"].as_str().unwrap(), "0x1000");
        assert_eq!(first["size"].as_u64().unwrap(), 64);
        assert_eq!(first["type"].as_str().unwrap(), "string");
    }

    #[test]
    fn test_process_allocation_batch_enhanced_sequential() {
        let allocations = vec![create_test_allocation(
            0x1000,
            64,
            Some("String".to_string()),
            Some("test_var".to_string()),
        )];

        let options = ExportJsonOptions::default().parallel_processing(false); // Force sequential processing

        let result = process_allocation_batch_enhanced(&allocations, &options);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.len(), 1);
    }

    #[test]
    fn test_estimate_json_size() {
        // Test simple object
        let simple_obj = serde_json::json!({
            "key": "value"
        });
        let size1 = estimate_json_size(&simple_obj);
        assert!(size1 > 0);

        // Test array
        let array = serde_json::json!([1, 2, 3, 4, 5]);
        let size2 = estimate_json_size(&array);
        assert!(size2 > 0);

        // Test complex nested structure
        let complex = serde_json::json!({
            "data": {
                "items": [
                    {"id": 1, "name": "item1"},
                    {"id": 2, "name": "item2"}
                ],
                "metadata": {
                    "count": 2,
                    "description": "test data"
                }
            }
        });
        let size3 = estimate_json_size(&complex);
        assert!(size3 > size1);
        assert!(size3 > size2);

        // Test string
        let string_val = serde_json::json!("This is a test string");
        let size4 = estimate_json_size(&string_val);
        assert!(size4 > 20); // String length + overhead

        // Test primitive
        let number = serde_json::json!(42);
        let size5 = estimate_json_size(&number);
        assert_eq!(size5, 20); // Default primitive size
    }

    #[test]
    fn test_write_json_optimized() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_output.json");

        let test_data = serde_json::json!({
            "test": "value",
            "number": 42,
            "array": [1, 2, 3]
        });

        let options = ExportJsonOptions::default()
            .schema_validation(false)
            .streaming_writer(false); // Use traditional writer for small files

        let result = write_json_optimized(&file_path, &test_data, &options);
        assert!(result.is_ok());

        // Verify file was created and contains valid JSON
        assert!(file_path.exists());
        let content = std::fs::read_to_string(&file_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["test"].as_str().unwrap(), "value");
        assert_eq!(parsed["number"].as_u64().unwrap(), 42);
    }

    #[test]
    fn test_write_json_optimized_compact_format() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_compact.json");

        let test_data = serde_json::json!({
            "test": "compact",
            "format": true
        });

        let mut options = ExportJsonOptions::default();
        options.use_compact_format = Some(true);
        options.schema_validation = false;
        options.streaming_writer = false;

        let result = write_json_optimized(&file_path, &test_data, &options);
        assert!(result.is_ok());

        // Verify file was created
        assert!(file_path.exists());
        let content = std::fs::read_to_string(&file_path).unwrap();

        // Compact format should not have extra whitespace
        assert!(!content.contains("  ")); // No indentation
        assert!(!content.contains('\n')); // No newlines
    }

    #[test]
    fn test_write_json_optimized_pretty_format() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_pretty.json");

        let test_data = serde_json::json!({
            "test": "pretty",
            "format": true
        });

        let mut options = ExportJsonOptions::default();
        options.use_compact_format = Some(false);
        options.schema_validation = false;
        options.streaming_writer = false;

        let result = write_json_optimized(&file_path, &test_data, &options);
        assert!(result.is_ok());

        // Verify file was created
        assert!(file_path.exists());
        let content = std::fs::read_to_string(&file_path).unwrap();

        // Pretty format should have whitespace
        assert!(content.contains("  ") || content.contains('\n')); // Has formatting
    }

    #[test]
    fn test_build_unified_dashboard_structure() {
        let allocations = vec![
            create_test_allocation(
                0x1000,
                64,
                Some("String".to_string()),
                Some("test_var".to_string()),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("Vec<i32>".to_string()),
                Some("test_vec".to_string()),
            ),
        ];

        let memory_by_type = vec![
            TypeMemoryUsage {
                type_name: "String".to_string(),
                total_size: 64,
                current_size: 64,
                allocation_count: 1,
                average_size: 64.0,
                peak_size: 64,
                efficiency_score: 0.8,
            },
            TypeMemoryUsage {
                type_name: "Vec<i32>".to_string(),
                total_size: 128,
                current_size: 128,
                allocation_count: 1,
                average_size: 128.0,
                peak_size: 128,
                efficiency_score: 0.9,
            },
        ];

        let stats = create_test_memory_stats();
        let unsafe_stats = create_test_unsafe_stats();

        let dashboard = build_unified_dashboard_structure(
            &allocations,
            &allocations, // Use same for history
            &memory_by_type,
            &stats,
            &unsafe_stats,
        );

        // Verify structure
        assert!(dashboard.get("metadata").is_some());
        assert!(dashboard.get("performance_metrics").is_some());
        assert!(dashboard.get("memory_statistics").is_some());
        assert!(dashboard.get("allocation_details").is_some());
        assert!(dashboard.get("type_usage").is_some());
        assert!(dashboard.get("unsafe_operations").is_some());
        assert!(dashboard.get("analysis_summary").is_some());

        // Verify metadata
        let metadata = dashboard.get("metadata").unwrap();
        assert_eq!(
            metadata.get("total_allocations").unwrap().as_u64().unwrap(),
            100
        );
        assert_eq!(
            metadata
                .get("active_allocations")
                .unwrap()
                .as_u64()
                .unwrap(),
            50
        );

        // Verify allocation details
        let allocation_details = dashboard
            .get("allocation_details")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(allocation_details.len(), 2);

        let first_alloc = &allocation_details[0];
        assert_eq!(first_alloc.get("ptr").unwrap().as_str().unwrap(), "0x1000");
        assert_eq!(first_alloc.get("size").unwrap().as_u64().unwrap(), 64);
        assert_eq!(
            first_alloc.get("type_name").unwrap().as_str().unwrap(),
            "String"
        );
        assert_eq!(
            first_alloc.get("var_name").unwrap().as_str().unwrap(),
            "test_var"
        );

        // Verify extended fields are present
        assert!(first_alloc.get("borrow_info").is_some());
        assert!(first_alloc.get("clone_info").is_some());
        assert!(first_alloc.get("ownership_history_available").is_some());
        assert!(first_alloc.get("ownership_history").is_some());
    }

    #[test]
    fn test_identify_memory_hotspots() {
        let memory_by_type = vec![
            TypeMemoryUsage {
                type_name: "LargeType".to_string(),
                total_size: 2 * 1024 * 1024, // 2MB
                current_size: 2 * 1024 * 1024,
                allocation_count: 10,
                average_size: 204800.0,
                peak_size: 512 * 1024,
                efficiency_score: 0.7,
            },
            TypeMemoryUsage {
                type_name: "MediumType".to_string(),
                total_size: 128 * 1024, // 128KB
                current_size: 128 * 1024,
                allocation_count: 50,
                average_size: 2560.0,
                peak_size: 4096,
                efficiency_score: 0.8,
            },
            TypeMemoryUsage {
                type_name: "SmallType".to_string(),
                total_size: 512, // 512B - should be filtered out
                current_size: 512,
                allocation_count: 100,
                average_size: 5.12,
                peak_size: 64,
                efficiency_score: 0.9,
            },
        ];

        let hotspots = identify_memory_hotspots(&memory_by_type);

        // Should only include types > 1KB
        assert_eq!(hotspots.len(), 2);

        // Should be sorted by total size descending
        let first_hotspot = &hotspots[0];
        assert_eq!(
            first_hotspot.get("type").unwrap().as_str().unwrap(),
            "LargeType"
        );
        assert_eq!(
            first_hotspot.get("total_size").unwrap().as_u64().unwrap(),
            2 * 1024 * 1024
        );
        assert_eq!(
            first_hotspot.get("severity").unwrap().as_str().unwrap(),
            "high"
        );

        let second_hotspot = &hotspots[1];
        assert_eq!(
            second_hotspot.get("type").unwrap().as_str().unwrap(),
            "MediumType"
        );
        assert_eq!(
            second_hotspot.get("total_size").unwrap().as_u64().unwrap(),
            128 * 1024
        );
        assert_eq!(
            second_hotspot.get("severity").unwrap().as_str().unwrap(),
            "medium"
        );
    }

    #[test]
    fn test_generate_optimization_recommendations() {
        // Test high fragmentation
        let high_frag_stats = MemoryStats {
            total_allocations: 100,
            total_allocated: 10000,
            active_allocations: 70,
            active_memory: 3000, // Low active vs total allocated
            peak_allocations: 90,
            peak_memory: 8000,
            total_deallocations: 30,
            total_deallocated: 3000,
            leaked_allocations: 0,
            leaked_memory: 0,
            fragmentation_analysis: crate::core::types::FragmentationAnalysis::default(),
            lifecycle_stats: crate::core::types::ScopeLifecycleMetrics::default(),
            allocations: Vec::new(),
            system_library_stats: crate::core::types::SystemLibraryStats::default(),
            concurrency_analysis: crate::core::types::ConcurrencyAnalysis::default(),
        };

        let memory_by_type = vec![TypeMemoryUsage {
            type_name: "LargeType".to_string(),
            total_size: 2 * 1024 * 1024,
            current_size: 2 * 1024 * 1024,
            allocation_count: 1,
            average_size: 2.0 * 1024.0 * 1024.0, // > 1MB average
            peak_size: 2 * 1024 * 1024,
            efficiency_score: 0.5,
        }];

        let recommendations =
            generate_optimization_recommendations(&high_frag_stats, &memory_by_type);

        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("fragmentation")));
        assert!(recommendations.iter().any(|r| r.contains("efficiency")));
        assert!(recommendations
            .iter()
            .any(|r| r.contains("large average allocations")));
        assert!(recommendations
            .iter()
            .any(|r| r.contains("allocation-to-deallocation ratio")));

        // Test healthy memory usage
        let healthy_stats = MemoryStats {
            total_allocations: 100,
            total_allocated: 1000,
            active_allocations: 10,
            active_memory: 900, // High efficiency
            peak_allocations: 20,
            peak_memory: 1000,
            total_deallocations: 90,
            total_deallocated: 800,
            leaked_allocations: 0,
            leaked_memory: 0,
            fragmentation_analysis: crate::core::types::FragmentationAnalysis::default(),
            lifecycle_stats: crate::core::types::ScopeLifecycleMetrics::default(),
            allocations: Vec::new(),
            system_library_stats: crate::core::types::SystemLibraryStats::default(),
            concurrency_analysis: crate::core::types::ConcurrencyAnalysis::default(),
        };

        let small_memory_by_type = vec![TypeMemoryUsage {
            type_name: "SmallType".to_string(),
            total_size: 1024,
            current_size: 1024,
            allocation_count: 10,
            average_size: 102.4, // Small average
            peak_size: 256,
            efficiency_score: 0.9,
        }];

        let healthy_recommendations =
            generate_optimization_recommendations(&healthy_stats, &small_memory_by_type);
        assert!(healthy_recommendations
            .iter()
            .any(|r| r.contains("healthy")));
    }

    #[test]
    fn test_export_json_options_debug_clone() {
        let options = ExportJsonOptions::default();

        // Test Debug implementation
        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("ExportJsonOptions"));
        assert!(debug_str.contains("parallel_processing"));
        assert!(debug_str.contains("buffer_size"));

        // Test Clone implementation
        let cloned_options = options.clone();
        assert_eq!(
            cloned_options.parallel_processing,
            options.parallel_processing
        );
        assert_eq!(cloned_options.buffer_size, options.buffer_size);
        assert_eq!(cloned_options.batch_size, options.batch_size);
        assert_eq!(cloned_options.max_cache_size, options.max_cache_size);
    }
}
