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
                tracing::debug!("Failed to associate variable '{}' - tracker busy", var_name);
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

    /// Export memory data to hierarchical JSON format organized by scopes.
    /// This creates a structured JSON with scopes as top-level containers,
    /// variables within scopes, and relationships between them.
    pub fn export_to_json<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        use std::fs::File;
        use std::collections::HashMap;
        
        let path = path.as_ref();
        let active_allocations = self.get_active_allocations()?;
        let stats = self.get_stats()?;

        // Group allocations by scope
        let mut scopes: HashMap<String, Vec<&crate::types::AllocationInfo>> = HashMap::new();
        
        for allocation in &active_allocations {
            let scope_name = allocation.scope_name
                .as_ref()
                .unwrap_or(&"Global".to_string())
                .clone();
            scopes.entry(scope_name).or_insert_with(Vec::new).push(allocation);
        }

        // Build hierarchical structure
        let mut scope_data = Vec::new();
        
        for (scope_name, allocations) in scopes {
            // Calculate scope-level statistics
            let total_memory: usize = allocations.iter().map(|a| a.size).sum();
            let variable_count = allocations.len();
            let avg_lifetime = if allocations.is_empty() {
                0.0
            } else {
                let total_lifetime: u128 = allocations.iter()
                    .filter_map(|a| a.lifetime_ms())
                    .sum();
                total_lifetime as f64 / allocations.len() as f64
            };

            // Group variables by type within scope
            let mut variables_by_type: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
            
            for allocation in &allocations {
                let type_name = allocation.type_name
                    .as_ref()
                    .unwrap_or(&"Unknown".to_string())
                    .clone();
                
                let variable_data = serde_json::json!({
                    "name": allocation.var_name.as_ref().unwrap_or(&format!("ptr_{:x}", allocation.ptr)),
                    "ptr": format!("0x{:x}", allocation.ptr),
                    "size": allocation.size,
                    "peak_size": allocation.peak_size,
                    "timestamp_alloc": allocation.timestamp_alloc,
                    "timestamp_dealloc": allocation.timestamp_dealloc,
                    "lifetime_ms": allocation.lifetime_ms(),
                    "thread_id": allocation.thread_id,
                    "growth_events": allocation.growth_events,
                    "ownership_pattern": allocation.ownership_pattern,
                    "risk_level": allocation.risk_level,
                    "efficiency_score": allocation.efficiency_score,
                    "borrow_count": allocation.borrow_count,
                    "mut_borrow_count": allocation.mut_borrow_count,
                    "transfer_count": allocation.transfer_count,
                    "metadata_tags": allocation.metadata_tags,
                    "memory_growth_factor": allocation.memory_growth_factor()
                });
                
                variables_by_type.entry(type_name).or_insert_with(Vec::new).push(variable_data);
            }

            // Calculate relationships within scope
            let mut relationships = Vec::new();
            
            // Find shared ownership relationships (Rc/Arc clones)
            for allocation in &allocations {
                if let Some(type_name) = &allocation.type_name {
                    if type_name.contains("Rc<") || type_name.contains("Arc<") {
                        // Look for other variables with the same type (potential clones)
                        for other in &allocations {
                            if other.ptr != allocation.ptr 
                                && other.type_name.as_ref() == Some(type_name) {
                                relationships.push(serde_json::json!({
                                    "type": "shared_ownership",
                                    "from": allocation.var_name.as_ref().unwrap_or(&format!("ptr_{:x}", allocation.ptr)),
                                    "to": other.var_name.as_ref().unwrap_or(&format!("ptr_{:x}", other.ptr)),
                                    "relationship": "clone",
                                    "strength": 0.9
                                }));
                            }
                        }
                    }
                }
            }

            // Find size-based relationships (containers and their contents)
            for allocation in &allocations {
                if let Some(type_name) = &allocation.type_name {
                    if type_name.contains("Box<") {
                        // Box contains another allocation
                        relationships.push(serde_json::json!({
                            "type": "containment",
                            "from": allocation.var_name.as_ref().unwrap_or(&format!("ptr_{:x}", allocation.ptr)),
                            "to": "heap_data",
                            "relationship": "owns",
                            "strength": 1.0
                        }));
                    }
                }
            }

            let scope_info = serde_json::json!({
                "scope_name": scope_name,
                "summary": {
                    "variable_count": variable_count,
                    "total_memory_bytes": total_memory,
                    "avg_lifetime_ms": avg_lifetime,
                    "types_count": variables_by_type.len()
                },
                "variables_by_type": variables_by_type,
                "relationships": relationships,
                "scope_metrics": {
                    "memory_efficiency": if total_memory > 0 {
                        allocations.iter()
                            .filter_map(|a| a.efficiency_score)
                            .sum::<f64>() / allocations.len() as f64
                    } else { 1.0 },
                    "growth_events_total": allocations.iter().map(|a| a.growth_events).sum::<usize>(),
                    "ownership_transfers_total": allocations.iter().map(|a| a.transfer_count).sum::<usize>(),
                    "borrow_events_total": allocations.iter().map(|a| a.borrow_count + a.mut_borrow_count).sum::<usize>()
                }
            });
            
            scope_data.push(scope_info);
        }

        // Sort scopes by memory usage (largest first)
        scope_data.sort_by(|a, b| {
            let a_memory = a["summary"]["total_memory_bytes"].as_u64().unwrap_or(0);
            let b_memory = b["summary"]["total_memory_bytes"].as_u64().unwrap_or(0);
            b_memory.cmp(&a_memory)
        });

        // Build final hierarchical structure
        let hierarchical_data = serde_json::json!({
            "metadata": {
                "export_timestamp": chrono::Utc::now(),
                "format_version": "1.0",
                "description": "Hierarchical memory analysis organized by scopes and variable relationships"
            },
            "global_stats": stats,
            "scope_summary": {
                "total_scopes": scope_data.len(),
                "total_variables": active_allocations.len(),
                "total_memory": active_allocations.iter().map(|a| a.size).sum::<usize>(),
                "scopes_by_memory": scope_data.iter().map(|s| serde_json::json!({
                    "name": s["scope_name"],
                    "memory": s["summary"]["total_memory_bytes"],
                    "variables": s["summary"]["variable_count"]
                })).collect::<Vec<_>>()
            },
            "scopes": scope_data,
            "cross_scope_relationships": {
                "description": "Relationships that span across different scopes",
                "global_shared_types": {
                    "description": "Types that appear in multiple scopes",
                    "types": self.analyze_cross_scope_types(&active_allocations)
                }
            }
        });

        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &hierarchical_data).map_err(|e| {
            crate::types::TrackingError::SerializationError(format!("Hierarchical JSON export failed: {e}"))
        })?;
        Ok(())
    }


    /// Analyze types that appear across multiple scopes
    fn analyze_cross_scope_types(&self, allocations: &[crate::types::AllocationInfo]) -> serde_json::Value {
        use std::collections::HashMap;
        
        let mut type_scope_map: HashMap<String, std::collections::HashSet<String>> = HashMap::new();
        
        for allocation in allocations {
            if let Some(type_name) = &allocation.type_name {
                let scope_name = allocation.scope_name
                    .as_ref()
                    .unwrap_or(&"Global".to_string())
                    .clone();
                
                type_scope_map.entry(type_name.clone())
                    .or_insert_with(std::collections::HashSet::new)
                    .insert(scope_name);
            }
        }
        
        let cross_scope_types: Vec<_> = type_scope_map.iter()
            .filter(|(_, scopes)| scopes.len() > 1)
            .map(|(type_name, scopes)| serde_json::json!({
                "type_name": type_name,
                "scopes": scopes.iter().collect::<Vec<_>>(),
                "scope_count": scopes.len()
            }))
            .collect();
            
        serde_json::json!(cross_scope_types)
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
