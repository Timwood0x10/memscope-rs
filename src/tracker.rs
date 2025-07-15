//! Memory allocation tracking functionality.

use crate::types::{AllocationInfo, MemoryStats, TrackingResult, TypeMemoryUsage};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Global memory tracker instance
static GLOBAL_TRACKER: OnceLock<Arc<MemoryTracker>> = OnceLock::new();

/// Get the global memory tracker instance.
///
/// This function returns a reference to the singleton memory tracker
/// that is used throughout the application.
pub fn get_global_tracker() -> Arc<MemoryTracker> {
    GLOBAL_TRACKER
        .get_or_init(|| Arc::new(MemoryTracker::new()))
        .clone()
}

/// Core memory tracking functionality.
///
/// The MemoryTracker maintains records of all memory allocations and deallocations,
/// provides statistics, and supports exporting data in various formats.
pub struct MemoryTracker {
    /// Active allocations (ptr -> allocation info)
    active_allocations: Mutex<HashMap<usize, AllocationInfo>>,
    /// Complete allocation history (for analysis)
    allocation_history: Mutex<Vec<AllocationInfo>>,
    /// Memory usage statistics
    stats: Mutex<MemoryStats>,
}

impl MemoryTracker {
    /// Create a new memory tracker.
    pub fn new() -> Self {
        Self {
            active_allocations: Mutex::new(HashMap::new()),
            allocation_history: Mutex::new(Vec::new()),
            stats: Mutex::new(MemoryStats::default()),
        }
    }

    /// Track a new memory allocation.
    pub fn track_allocation(&self, ptr: usize, size: usize) -> TrackingResult<()> {
        // Create allocation info first (no locks needed)
        let allocation = AllocationInfo::new(ptr, size);

        // Use try_lock to avoid blocking during high allocation activity
        match (self.active_allocations.try_lock(), self.stats.try_lock()) {
            (Ok(mut active), Ok(mut stats)) => {
                // Add to active allocations
                active.insert(ptr, allocation.clone());

                // Update statistics with overflow protection
                stats.total_allocations = stats.total_allocations.saturating_add(1);
                stats.total_allocated = stats.total_allocated.saturating_add(size);
                stats.active_allocations = stats.active_allocations.saturating_add(1);
                stats.active_memory = stats.active_memory.saturating_add(size);

                // Update peaks
                if stats.active_allocations > stats.peak_allocations {
                    stats.peak_allocations = stats.active_allocations;
                }
                if stats.active_memory > stats.peak_memory {
                    stats.peak_memory = stats.active_memory;
                }

                // Release locks before adding to history
                drop(stats);
                drop(active);

                // Add to history with separate try_lock (optional, skip if busy)
                if let Ok(mut history) = self.allocation_history.try_lock() {
                    history.push(allocation);
                }

                Ok(())
            }
            _ => {
                // If we can't get locks immediately, skip tracking to avoid deadlock
                // This is acceptable as we prioritize program stability over complete tracking
                Ok(())
            }
        }
    }

    /// Track a memory deallocation.
    pub fn track_deallocation(&self, ptr: usize) -> TrackingResult<()> {
        // Use try_lock to avoid blocking during high deallocation activity
        match (self.active_allocations.try_lock(), self.stats.try_lock()) {
            (Ok(mut active), Ok(mut stats)) => {
                if let Some(allocation) = active.remove(&ptr) {
                    // Update statistics with overflow protection
                    stats.total_deallocations = stats.total_deallocations.saturating_add(1);
                    stats.total_deallocated =
                        stats.total_deallocated.saturating_add(allocation.size);
                    stats.active_allocations = stats.active_allocations.saturating_sub(1);
                    stats.active_memory = stats.active_memory.saturating_sub(allocation.size);
                }
                Ok(())
            }
            _ => {
                // If we can't get locks immediately, skip tracking to avoid deadlock
                Ok(())
            }
        }
    }

    /// Associate a variable name and type with an allocation.
    pub fn associate_var(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
    ) -> TrackingResult<()> {
        // Use try_lock to avoid blocking if the allocator is currently tracking
        match self.active_allocations.try_lock() {
            Ok(mut active) => {
                if let Some(allocation) = active.get_mut(&ptr) {
                    allocation.var_name = Some(var_name.clone());
                    allocation.type_name = Some(type_name.clone());
                    tracing::debug!(
                        "Associated variable '{}' with existing allocation at {:x}",
                        var_name,
                        ptr
                    );
                    Ok(())
                } else {
                    // For smart pointers and other complex types, create a synthetic allocation entry
                    // This ensures we can track variables even when the exact pointer isn't in our allocator
                    let mut synthetic_allocation = AllocationInfo::new(ptr, 0); // Size will be estimated
                    synthetic_allocation.var_name = Some(var_name.clone());
                    synthetic_allocation.type_name = Some(type_name.clone());

                    // Estimate size based on type
                    let estimated_size = estimate_type_size(&type_name);
                    synthetic_allocation.size = estimated_size;

                    // Add to active allocations for tracking
                    active.insert(ptr, synthetic_allocation);
                    tracing::debug!("Created synthetic allocation for variable '{}' at {:x} (estimated size: {})", 
                                   var_name, ptr, estimated_size);
                    Ok(())
                }
            }
            Err(_) => {
                // If we can't get the lock immediately, it's likely the allocator is busy
                // We'll just skip the association to avoid deadlock
                // tracing::warn!("Failed to associate variable '{}' - tracker busy", var_name);
                Ok(())
            }
        }
    }

    /// Get current memory usage statistics.
    pub fn get_stats(&self) -> TrackingResult<MemoryStats> {
        match self.stats.lock() {
            Ok(stats) => Ok(stats.clone()),
            Err(poisoned) => {
                // Handle poisoned lock by recovering the data
                let stats = poisoned.into_inner();
                Ok(stats.clone())
            }
        }
    }

    /// Get all currently active allocations.
    pub fn get_active_allocations(&self) -> TrackingResult<Vec<AllocationInfo>> {
        match self.active_allocations.lock() {
            Ok(active) => Ok(active.values().cloned().collect()),
            Err(poisoned) => {
                // Handle poisoned lock by recovering the data
                let active = poisoned.into_inner();
                Ok(active.values().cloned().collect())
            }
        }
    }

    /// Get the complete allocation history.
    pub fn get_allocation_history(&self) -> TrackingResult<Vec<AllocationInfo>> {
        match self.allocation_history.lock() {
            Ok(history) => Ok(history.clone()),
            Err(poisoned) => {
                // Handle poisoned lock by recovering the data
                let history = poisoned.into_inner();
                Ok(history.clone())
            }
        }
    }

    /// Get memory usage grouped by type.
    pub fn get_memory_by_type(&self) -> TrackingResult<Vec<TypeMemoryUsage>> {
        // Clone the active allocations to avoid holding the lock for too long
        let active_clone = {
            match self.active_allocations.lock() {
                Ok(active) => active.values().cloned().collect::<Vec<_>>(),
                Err(poisoned) => {
                    // Handle poisoned lock by recovering the data
                    let active = poisoned.into_inner();
                    active.values().cloned().collect::<Vec<_>>()
                }
            }
        };

        let mut type_usage: HashMap<String, (usize, usize)> = HashMap::new();

        for allocation in active_clone {
            let type_name = allocation
                .type_name
                .unwrap_or_else(|| "Unknown".to_string());

            let (total_size, count) = type_usage.entry(type_name).or_insert((0, 0));
            *total_size = total_size.saturating_add(allocation.size);
            *count = count.saturating_add(1);
        }

        let mut result: Vec<TypeMemoryUsage> = type_usage
            .into_iter()
            .map(
                |(type_name, (total_size, allocation_count))| TypeMemoryUsage {
                    type_name,
                    total_size,
                    allocation_count,
                },
            )
            .collect();

        // Sort by total size descending
        result.sort_by(|a, b| b.total_size.cmp(&a.total_size));

        Ok(result)
    }

    /// Export memory data to JSON format with unified dashboard structure.
    pub fn export_to_json<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        use std::fs::File;
        let path = path.as_ref();
        let active_allocations = self.get_active_allocations()?;
        let memory_by_type = self.get_memory_by_type()?;
        let stats = self.get_stats()?;
        let allocation_history = self.get_allocation_history()?;

        // Get unsafe/FFI data if available
        let unsafe_stats = crate::unsafe_ffi_tracker::get_global_unsafe_ffi_tracker()
            .get_stats();

        // Build unified dashboard structure
        let dashboard_data = build_unified_dashboard_structure(
            &active_allocations,
            &allocation_history,
            &memory_by_type,
            &stats,
            &unsafe_stats,
        );

        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &dashboard_data).map_err(|e| {
            crate::types::TrackingError::SerializationError(format!("JSON export failed: {e}"))
        })?;
        Ok(())
    }

    /// Export memory analysis visualization showing variable names, types, and usage patterns.
    /// This creates a comprehensive memory analysis with call stack analysis, timeline, and categorization.
    ///
    /// # Arguments
    /// * `path` - Output path for the memory analysis SVG file (recommended: "program_name_memory_analysis.svg")
    pub fn export_memory_analysis<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        crate::visualization::export_memory_analysis(self, path)
    }

    /// Export interactive lifecycle timeline showing variable lifecycles and relationships.
    /// This creates an advanced timeline with variable birth, life, death, and cross-section interactivity.
    ///
    /// # Arguments
    /// * `path` - Output path for the lifecycle timeline SVG file (recommended: "program_name_lifecycle.svg")
    pub fn export_lifecycle_timeline<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> TrackingResult<()> {
        crate::visualization::export_lifecycle_timeline(self, path)
    }

    /// Legacy export method for backward compatibility.
    /// Redirects to the new memory analysis export.
    ///
    /// # Arguments
    /// * `path` - Output path for the SVG file
    #[deprecated(since = "0.1.0", note = "Use export_memory_analysis instead")]
    pub fn export_to_svg<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        self.export_memory_analysis(path)
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Estimate the size of a type based on its name
/// This is used for synthetic allocations when we can't get the exact size
fn estimate_type_size(type_name: &str) -> usize {
    if type_name.contains("Box<") {
        // Box typically contains a pointer (8 bytes) plus the size of the contained type
        if type_name.contains("Vec") {
            64 // Vec has capacity, length, and pointer
        } else if type_name.contains("String") {
            48 // String has capacity, length, and pointer
        } else if type_name.contains("HashMap") {
            128 // HashMap has more complex internal structure
        } else {
            32 // Generic Box overhead
        }
    } else if type_name.contains("Rc<") || type_name.contains("Arc<") {
        // Reference counted types have additional overhead
        if type_name.contains("RefCell") {
            72 // Rc<RefCell<T>> has extra indirection
        } else {
            56 // Basic Rc/Arc overhead
        }
    } else if type_name.contains("Vec<") {
        // Direct Vec allocation
        48 // Vec struct size (capacity, length, pointer)
    } else if type_name.contains("String") {
        // Direct String allocation
        32 // String struct size (capacity, length, pointer)
    } else if type_name.contains("HashMap") {
        // Direct HashMap allocation
        96 // HashMap has complex internal structure
    } else {
        // Default estimate for unknown types
        24
    }
}

/// Build unified dashboard JSON structure compatible with all frontend interfaces
fn build_unified_dashboard_structure(
    active_allocations: &[AllocationInfo],
    allocation_history: &[AllocationInfo],
    memory_by_type: &[crate::types::TypeMemoryUsage],
    stats: &crate::types::MemoryStats,
    unsafe_stats: &crate::unsafe_ffi_tracker::UnsafeFFIStats,
) -> serde_json::Value {
    use std::collections::HashMap;

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
                .unwrap_or(0)
        ) / 1_000_000; // Convert nanoseconds to milliseconds

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

    // Prepare allocation details for frontend
    let allocation_details: Vec<_> = active_allocations
        .iter()
        .take(100) // Limit to avoid huge JSON files
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
    let unsafe_operations: Vec<_> = unsafe_stats.operations
        .iter()
        .take(50) // Limit to avoid huge JSON files
        .map(|op| {
            serde_json::json!({
                "type": format!("{:?}", op.operation_type),
                "location": op.location,
                "risk_level": format!("{:?}", op.risk_level),
                "timestamp": op.timestamp,
                "description": op.description
            })
        })
        .collect();

    // Calculate lifecycle statistics
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u128;

    let mut lifetimes: Vec<u128> = allocation_history
        .iter()
        .filter_map(|alloc| {
            if let Some(dealloc_time) = alloc.timestamp_dealloc {
                if dealloc_time > 0 {
                    Some(dealloc_time.saturating_sub(alloc.timestamp_alloc))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    lifetimes.sort_unstable();
    let average_lifetime_ms = if !lifetimes.is_empty() {
        lifetimes.iter().sum::<u128>() / lifetimes.len() as u128 / 1_000_000
    } else {
        0
    };

    // Categorize objects by lifetime
    let short_lived = lifetimes.iter().filter(|&&lt| lt < 1_000_000_000).count(); // < 1 second
    let medium_lived = lifetimes.iter().filter(|&&lt| lt >= 1_000_000_000 && lt < 10_000_000_000).count(); // 1-10 seconds
    let long_lived = lifetimes.iter().filter(|&&lt| lt >= 10_000_000_000).count(); // > 10 seconds

    // Build hierarchical memory structure for backward compatibility
    let enhanced_types = crate::export_enhanced::enhance_type_information(memory_by_type, active_allocations);
    let memory_hierarchy = build_legacy_hierarchy(&enhanced_types, active_allocations, stats);

    // Build the unified dashboard structure
    serde_json::json!({
        "memory_stats": {
            "total_allocations": stats.total_allocations,
            "total_size_bytes": stats.total_allocated,
            "peak_memory_usage": stats.peak_memory,
            "current_memory_usage": stats.active_memory,
            "allocation_rate": allocation_rate,
            "deallocation_rate": deallocation_rate,
            "memory_efficiency": memory_efficiency,
            "fragmentation_ratio": fragmentation_ratio,
            "allocations": allocation_details
        },
        "unsafe_stats": {
            "total_operations": unsafe_stats.total_operations,
            "unsafe_blocks": unsafe_stats.unsafe_blocks,
            "ffi_calls": unsafe_stats.ffi_calls,
            "raw_pointer_operations": unsafe_stats.raw_pointer_operations,
            "memory_violations": unsafe_stats.memory_violations,
            "risk_score": unsafe_stats.risk_score,
            "operations": unsafe_operations
        },
        "performance_metrics": {
            "allocation_time_avg_ns": if stats.total_allocations > 0 { 
                total_runtime_ms * 1_000_000 / stats.total_allocations as u128 
            } else { 
                0 
            },
            "allocation_time_max_ns": total_runtime_ms * 1_000_000, // Simplified
            "memory_throughput_mb_s": if total_runtime_ms > 0 {
                (stats.total_allocated as f64 / 1_048_576.0) / (total_runtime_ms as f64 / 1000.0)
            } else {
                0.0
            },
            "gc_pressure": fragmentation_ratio
        },
        "lifecycle_stats": {
            "short_lived_objects": short_lived,
            "medium_lived_objects": medium_lived,
            "long_lived_objects": long_lived,
            "average_lifetime_ms": average_lifetime_ms,
            "memory_leaks_detected": stats.active_allocations.saturating_sub(
                allocation_history.iter().filter(|a| a.timestamp_dealloc.is_some()).count()
            )
        },
        "metadata": {
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "version": "2.0",
            "source": "memscope-rs unified dashboard export",
            "total_runtime_ms": total_runtime_ms,
            "format_description": "Unified dashboard format compatible with all frontend interfaces"
        },
        // Keep legacy hierarchy for backward compatibility
        "memory_hierarchy": memory_hierarchy,
        // Summary for legacy compatibility
        "summary": {
            "total_memory_bytes": stats.total_allocated,
            "total_allocations": stats.total_allocations,
            "active_allocations": stats.active_allocations,
            "active_memory_bytes": stats.active_memory,
            "peak_memory_bytes": stats.peak_memory
        }
    })
}

/// Build legacy hierarchical structure for backward compatibility
fn build_legacy_hierarchy(
    enhanced_types: &[crate::export_enhanced::EnhancedTypeInfo],
    active_allocations: &[AllocationInfo],
    stats: &crate::types::MemoryStats,
) -> serde_json::Value {
    use std::collections::HashMap;

    // Group enhanced types by category and subcategory
    let mut categories: HashMap<
        String,
        HashMap<String, Vec<&crate::export_enhanced::EnhancedTypeInfo>>,
    > = HashMap::new();

    for enhanced_type in enhanced_types {
        categories
            .entry(enhanced_type.category.clone())
            .or_insert_with(HashMap::new)
            .entry(enhanced_type.subcategory.clone())
            .or_insert_with(Vec::new)
            .push(enhanced_type);
    }

    // Build hierarchical structure
    let mut category_data = serde_json::Map::new();
    let total_memory: usize = enhanced_types.iter().map(|t| t.total_size).sum();

    for (category_name, subcategories) in categories {
        let category_total: usize = subcategories
            .values()
            .flat_map(|types| types.iter())
            .map(|t| t.total_size)
            .sum();

        let category_percentage = if total_memory > 0 {
            (category_total as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };

        let mut subcategory_data = serde_json::Map::new();
        let subcategory_count = subcategories.len();

        for (subcategory_name, types) in subcategories {
            let subcategory_total: usize = types.iter().map(|t| t.total_size).sum();
            let subcategory_percentage = if category_total > 0 {
                (subcategory_total as f64 / category_total as f64) * 100.0
            } else {
                0.0
            };

            let mut type_details = Vec::new();
            let type_count = types.len();
            for type_info in &types {
                let type_percentage = if subcategory_total > 0 {
                    (type_info.total_size as f64 / subcategory_total as f64) * 100.0
                } else {
                    0.0
                };

                // Find allocations for this specific type
                let type_allocations: Vec<_> = active_allocations
                    .iter()
                    .filter(|alloc| {
                        if let Some(type_name) = &alloc.type_name {
                            alloc.var_name.as_ref().map_or(false, |var_name| {
                                type_info.variable_names.contains(var_name)
                            }) || type_name.contains(&type_info.simplified_name)
                        } else {
                            false
                        }
                    })
                    .map(|alloc| {
                        serde_json::json!({
                            "variable_name": alloc.var_name,
                            "size_bytes": alloc.size,
                            "allocation_time": alloc.timestamp_alloc,
                            "type_name": alloc.type_name
                        })
                    })
                    .collect();

                type_details.push(serde_json::json!({
                    "type_name": type_info.simplified_name,
                    "size_bytes": type_info.total_size,
                    "allocation_count": type_info.allocation_count,
                    "percentage_of_subcategory": format!("{:.1}%", type_percentage),
                    "percentage_of_total": format!("{:.1}%", (type_info.total_size as f64 / total_memory as f64) * 100.0),
                    "variable_names": type_info.variable_names,
                    "allocations": type_allocations
                }));
            }

            subcategory_data.insert(subcategory_name, serde_json::json!({
                "summary": {
                    "total_size_bytes": subcategory_total,
                    "percentage_of_category": format!("{:.1}%", subcategory_percentage),
                    "percentage_of_total": format!("{:.1}%", (subcategory_total as f64 / total_memory as f64) * 100.0),
                    "type_count": type_count
                },
                "types": type_details
            }));
        }

        category_data.insert(
            category_name,
            serde_json::json!({
                "summary": {
                    "total_size_bytes": category_total,
                    "percentage_of_total": format!("{:.1}%", category_percentage),
                    "subcategory_count": subcategory_count
                },
                "subcategories": subcategory_data
            }),
        );
    }

    serde_json::Value::Object(category_data)
}
