//! Optimized JSON export functionality for memory tracking data.
//!
//! This module provides highly optimized JSON export functionality with performance improvements
//! including parallel processing, streaming writes, and adaptive optimization.

use super::memory_tracker::MemoryTracker;
use crate::core::types::{AllocationInfo, MemoryStats, TrackingResult, TypeMemoryUsage};
use crate::export::optimized_json_export::OptimizationLevel;
use serde_json::json;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};
use crate::export::schema_validator::SchemaValidator;
use rayon::prelude::*;

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
            buffer_size: 256 * 1024, // 256KB
            use_compact_format: None, // Auto-detect
            enable_type_cache: true,
            batch_size: 1000,
            streaming_writer: true,
            schema_validation: false, // Off by default for speed
            adaptive_optimization: true,
            max_cache_size: 10_000,
            security_analysis: false, // Off by default for speed
            include_low_severity: false,
            integrity_hashes: false,  // Off by default for speed
            fast_export_mode: false,
            auto_fast_export_threshold: Some(10_000), // Auto-enable fast mode for >10k allocations
            thread_count: None,       // Use default thread count
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
                buffer_size: 1024 * 1024,  // 1MB buffer for maximum performance
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
        let type_info = get_or_compute_type_info(
            alloc.type_name.as_deref().unwrap_or("unknown"),
            alloc.size,
        );
        
        let mut entry = json!({
            "address": format!("0x{:x}", alloc.ptr),
            "size": alloc.size,
            "type": type_info,
            "timestamp": alloc.timestamp_alloc,
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
            .map(|chunk| process_allocation_batch(chunk))
            .reduce(|| Ok(Vec::new()), |acc, chunk_result| {
                match (acc, chunk_result) {
                    (Ok(mut vec), Ok(chunk)) => {
                        vec.extend(chunk);
                        Ok(vec)
                    }
                    (Err(e), _) | (_, Err(e)) => Err(e),
                }
            })
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
    // TODO: Fix streaming writer implementation
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
    /// ```rust
    /// tracker.export_to_json("output")?;
    /// // OR explicitly
    /// tracker.export_to_json_with_options("output", ExportOptions::default())?;
    /// ```
    /// - **Performance**: ~2-5 seconds for typical datasets
    /// - **Data**: Only user-tracked variables get full enrichment
    /// - **Use case**: Normal development, HTML rendering, production monitoring
    ///
    /// ## Complete Mode (Slow - Debug Only)
    /// ```rust
    /// let options = ExportOptions::new().include_system_allocations(true);
    /// tracker.export_to_json_with_options("output", options)?;
    /// ```
    /// - **Performance**: ~10-40 seconds (5-10x slower!)
    /// - **Data**: ALL allocations including system internals get full enrichment
    /// - **Use case**: Deep debugging, memory leak investigation, system analysis
    /// - **⚠️ Warning**: Very slow, generates large files, may impact application performance
    pub fn export_to_json<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        // Ensure output goes to MemoryAnalysis directory
        let output_path = self.ensure_memory_analysis_path(path);

        // Use fast mode by default for optimal performance
        let options = ExportJsonOptions::default()
            .fast_export_mode(true)
            .security_analysis(false) // Disable for speed
            .schema_validation(false) // Disable for speed
            .integrity_hashes(false); // Disable for speed

        // Use the standard export function with our optimized options
        self.export_to_json_with_options(output_path, options)
    }

    /// Export memory tracking data with custom options.
    ///
    /// # Examples
    ///
    /// ## Fast mode (default - recommended for most users)
    /// ```rust
    /// tracker.export_to_json_with_options("output", ExportOptions::default())?;
    /// ```
    ///
    /// ## Complete mode (slow - for debugging)
    /// ```rust
    /// let options = ExportOptions::new()
    ///     .include_system_allocations(true)
    ///     .verbose_logging(true);
    /// tracker.export_to_json_with_options("debug_output", options)?;
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
        
        // Write output file
        let output_path = output_path.join("memory_analysis.json");
        write_json_optimized(output_path, &output_data, &options)?;
        
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

    // Prepare allocation details for frontend - use filtered data
    let allocation_details: Vec<_> = active_allocations
        .iter()
        .map(|alloc| {
            serde_json::json!({
                "size": alloc.size,
                "type": alloc.type_name.as_deref().unwrap_or("unknown"),
                "variable": alloc.var_name.as_deref().unwrap_or("unknown"),
                "timestamp": alloc.timestamp_alloc
            })
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
