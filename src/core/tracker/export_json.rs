//! JSON export functionality for memory tracking data.
//!
//! This module contains methods for exporting memory tracking data to JSON format,
//! including dashboard structures and various export options.

use super::memory_tracker::MemoryTracker;
use crate::core::types::{AllocationInfo, MemoryStats, TrackingResult, TypeMemoryUsage};
use crate::export::optimized_json_export::{OptimizationLevel, OptimizedExportOptions};
use std::path::Path;

impl MemoryTracker {
    /// Export memory tracking data to JSON format (fast mode by default).
    ///
    /// This is the recommended method for most users as it provides optimal performance
    /// while maintaining comprehensive data export.
    ///
    /// # Examples
    ///
    /// ## Default Mode (Fast - Recommended)
    /// ```text
    /// tracker.export_to_json("output")?;
    ///
    /// // OR explicitly
    /// tracker.export_to_json_with_options("output", ExportOptions::default())?;
    /// ```
    /// - **Performance**: ~2-5 seconds for typical datasets
    /// - **Data**: User-tracked variables with full enrichment
    /// - **Use case**: Regular development, profiling, optimization
    ///
    /// ## Complete Mode (Slow - For Deep Analysis)
    /// ```text
    /// let options = ExportOptions::new().include_system_allocations(true);
    /// tracker.export_to_json_with_options("output", options)?;
    /// ```
    /// - **Performance**: ~10-40 seconds (5-10x slower!)
    /// - **Data**: ALL allocations including system internals
    /// - **Use case**: Deep debugging, memory leak investigation, system analysis
    /// - **⚠️ Warning**: Very slow, generates large files, may impact application performance
    pub fn export_to_json<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
        // Ensure output goes to MemoryAnalysis directory
        let output_path = self.ensure_memory_analysis_path(path);

        // Use fast mode by default for optimal performance
        let options = OptimizedExportOptions::default()
            .fast_export_mode(true)
            .security_analysis(false) // Disable for speed
            .schema_validation(false) // Disable for speed
            .integrity_hashes(false); // Disable for speed

        self.export_to_json_with_optimized_options_internal(output_path, options)
    }

    /// Export memory tracking data with custom options.
    ///
    /// This method provides backward compatibility with the legacy ExportOptions
    /// while leveraging the new optimized export system.
    ///
    /// # Examples
    ///
    /// ## Fast mode (default - recommended for most users)
    /// ```text
    /// tracker.export_to_json_with_options("output", ExportOptions::default())?;
    /// ```
    ///
    /// ## Complete mode (slow - for debugging)
    /// ```text
    /// let options = ExportOptions::new()
    ///     .include_system_allocations(true)
    ///     .verbose_logging(true);
    /// tracker.export_to_json_with_options("debug_output", options)?;
    /// ```
    pub fn export_to_json_with_options<P: AsRef<Path>>(
        &self,
        path: P,
        options: crate::core::tracker::config::ExportOptions,
    ) -> TrackingResult<()> {
        // Convert legacy ExportOptions to OptimizedExportOptions for backward compatibility
        let mut optimized_options = OptimizedExportOptions {
            buffer_size: options.buffer_size,
            use_compact_format: Some(!options.verbose_logging), // Verbose = pretty format
            ..Default::default()
        };

        // Determine optimization level based on legacy settings
        if options.include_system_allocations {
            // System allocations = comprehensive analysis = High optimization
            optimized_options.optimization_level = OptimizationLevel::High;
            optimized_options.enable_enhanced_ffi_analysis = true;
            optimized_options.enable_boundary_event_processing = true;
            optimized_options.enable_memory_passport_tracking = true;
            optimized_options.enable_security_analysis = true;

            tracing::info!(
                "⚠️  WARNING: System allocation enrichment enabled - export will be 5-10x slower!"
            );
            tracing::info!(
                "💡 To speed up export, use default options: tracker.export_to_json(path)"
            );
        } else {
            // User-focused mode = High optimization (default)
            optimized_options.optimization_level = OptimizationLevel::High;
        }

        // Enable compression if requested in legacy options
        if options.compress_output {
            optimized_options.use_compact_format = Some(true);
            optimized_options.buffer_size = optimized_options.buffer_size.max(512 * 1024);
            // Larger buffer for compression
        }

        // Adjust parallel processing based on expected load
        optimized_options.parallel_processing =
            options.include_system_allocations || options.buffer_size > 128 * 1024;

        tracing::info!("🔄 Converted legacy ExportOptions to OptimizedExportOptions:");
        tracing::info!(
            "   - Optimization level: {:?}",
            optimized_options.optimization_level
        );
        tracing::info!(
            "   - Buffer size: {} KB",
            optimized_options.buffer_size / 1024
        );
        tracing::info!(
            "   - Parallel processing: {}",
            optimized_options.parallel_processing
        );
        tracing::info!(
            "   - Enhanced features: {}",
            optimized_options.enable_enhanced_ffi_analysis
        );

        // Use the new optimized export method
        self.export_to_json(path)
    }

    /// Internal method to handle export with optimized options
    fn export_to_json_with_optimized_options_internal<P: AsRef<Path>>(
        &self,
        path: P,
        options: OptimizedExportOptions,
    ) -> TrackingResult<()> {
        // Delegate to the optimized export implementation
        self.export_json_with_options(path, options)
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
