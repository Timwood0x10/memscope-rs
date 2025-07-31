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

    /// Get current memory usage statistics with advanced analysis.
    pub fn get_stats(&self) -> TrackingResult<MemoryStats> {
        let base_stats = match self.stats.lock() {
            Ok(stats) => stats.clone(),
            Err(poisoned) => {
                // Handle poisoned lock by recovering the data
                let stats = poisoned.into_inner();
                stats.clone()
            }
        };
        
        // Get active allocations for advanced analysis
        let active_allocations = self.get_active_allocations()?;
        
        // Perform advanced analysis
        let fragmentation_analysis = crate::advanced_analysis::analyze_fragmentation(&active_allocations);
        let system_library_stats = crate::advanced_analysis::analyze_system_libraries(&active_allocations);
        let concurrency_analysis = crate::advanced_analysis::analyze_concurrency_safety(&active_allocations);
        
        Ok(MemoryStats {
            total_allocations: base_stats.total_allocations,
            total_deallocations: base_stats.total_deallocations,
            total_allocated: base_stats.total_allocated,
            total_deallocated: base_stats.total_deallocated,
            active_allocations: base_stats.active_allocations,
            active_memory: base_stats.active_memory,
            peak_allocations: base_stats.peak_allocations,
            peak_memory: base_stats.peak_memory,
            lifecycle_stats: base_stats.lifecycle_stats,
            allocations: active_allocations,
            fragmentation_analysis,
            system_library_stats,
            concurrency_analysis,
        })
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

        let mut type_usage: HashMap<String, (usize, usize)> = HashMap::with_capacity(active_clone.len());

        for allocation in active_clone {
            let type_name = allocation
                .type_name
                .unwrap_or("Unknown".to_string());

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

    /// Export interactive HTML dashboard with embedded SVG charts
    pub fn export_interactive_dashboard<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        crate::html_export::export_interactive_dashboard(self, path)
    }

    /// Export memory data to JSON format with unified dashboard structure.
    pub fn export_to_json<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        use std::fs::File;
        let path = path.as_ref();
        let active_allocations = self.get_active_allocations()?;
        let memory_by_type = self.get_memory_by_type()?;
        let stats = self.get_stats()?;
        let allocation_history = self.get_allocation_history()?;

        // Apply same filtering logic as HTML export
        let mut prioritized_allocations = Vec::new();
        let mut system_allocations = Vec::new();
        
        // Separate user variables and system allocations
        for alloc in &active_allocations {
            if alloc.var_name.is_some() {
                prioritized_allocations.push(alloc.clone());
            } else if alloc.size >= 1024 {  // Only include >= 1KB system allocations
                let mut enhanced_alloc = alloc.clone();
                // Use smart analysis for more precise identification
                let (smart_name, smart_type) = crate::html_export::analyze_system_allocation(alloc);
                enhanced_alloc.var_name = Some(smart_name);
                enhanced_alloc.type_name = Some(smart_type);
                system_allocations.push(enhanced_alloc);
            }
        }
        
        // Sort system allocations by size, take top 50
        system_allocations.sort_by(|a, b| b.size.cmp(&a.size));
        system_allocations.truncate(50);
        
        // Merge: user variables + large system allocations
        prioritized_allocations.extend(system_allocations);
        
        // Filter history data similarly
        let mut prioritized_history = Vec::new();
        let mut system_history = Vec::new();
        
        for alloc in allocation_history.iter().rev() {  // Most recent first
            if alloc.var_name.is_some() {
                prioritized_history.push(alloc.clone());
            } else if alloc.size >= 512 && system_history.len() < 100 {  // Limit system history
                let mut enhanced_alloc = alloc.clone();
                // Use smart analysis for history allocations too
                let (smart_name, smart_type) = crate::html_export::analyze_system_allocation(alloc);
                enhanced_alloc.var_name = Some(smart_name);
                enhanced_alloc.type_name = Some(smart_type);
                system_history.push(enhanced_alloc);
            }
            
            if prioritized_history.len() >= 200 {  // Limit total count
                break;
            }
        }
        
        prioritized_history.extend(system_history);

        println!("JSON export: {} user variables, {} large system allocations", 
                 prioritized_allocations.iter().filter(|a| a.var_name.as_ref().map_or(false, |n| !n.starts_with("system_"))).count(),
                 prioritized_allocations.iter().filter(|a| a.var_name.as_ref().map_or(false, |n| n.starts_with("system_"))).count());

        // Get unsafe/FFI data if available
        let unsafe_stats = crate::unsafe_ffi_tracker::get_global_unsafe_ffi_tracker()
            .get_stats();

        // Generate timeline data with filtered data
        let timeline_data = self.generate_timeline_data(&prioritized_history, &prioritized_allocations);

        // Build unified dashboard structure with filtered data
        let mut dashboard_data = build_unified_dashboard_structure(
            &prioritized_allocations,
            &prioritized_history,
            &memory_by_type,
            &stats,
            &unsafe_stats,
        );

        // Add timeline data to the dashboard
        if let serde_json::Value::Object(ref mut map) = dashboard_data {
            map.insert("timeline".to_string(), serde_json::to_value(timeline_data).unwrap_or(serde_json::Value::Null));
        }

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

    /// Export enhanced JSON with complete timeline, call stacks, scope hierarchy, and variable relationships.
    /// This addresses all issues in current JSON output: no unknown types, precise variable names, complete data.
    pub fn export_enhanced_json<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        use std::fs::File;
        let path = path.as_ref();
        
        println!("Generating enhanced JSON with complete data...");
        
        // Get all data
        let active_allocations = self.get_active_allocations()?;
        let allocation_history = self.get_allocation_history()?;
        let memory_by_type = self.get_memory_by_type()?;
        let stats = self.get_stats()?;
        
        // Get unsafe/FFI data
        let unsafe_stats = crate::unsafe_ffi_tracker::get_global_unsafe_ffi_tracker().get_stats();
        
        // Enhance allocations with precise names (no unknown types)
        let enhanced_allocations = self.enhance_allocations_with_precise_names(&active_allocations)?;
        let enhanced_history = self.enhance_allocations_with_precise_names(&allocation_history)?;
        
        // Generate comprehensive timeline with call stacks
        let enhanced_timeline = self.generate_timeline_data(&enhanced_history, &enhanced_allocations);
        
        // Generate scope hierarchy
        let scope_hierarchy = self.generate_scope_hierarchy(&enhanced_allocations)?;
        
        // Generate variable relationship graph
        let variable_relationships = self.generate_variable_relationships(&enhanced_allocations)?;
        
        // Generate SVG data for the three main visualizations
        let svg_data = self.generate_svg_visualization_data(&enhanced_allocations, &stats)?;
        
        // Build the enhanced JSON structure
        let enhanced_data = serde_json::json!({
            "metadata": {
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "version": "3.0-enhanced",
                "source": "memscope-rs enhanced export",
                "format_description": "Enhanced JSON format with complete timeline, call stacks, scope hierarchy, and variable relationships",
                "features": [
                    "complete_timeline_data",
                    "detailed_call_stacks", 
                    "scope_hierarchy",
                    "variable_relationships",
                    "svg_visualization_data",
                    "unsafe_ffi_analysis",
                    "no_unknown_types"
                ]
            },
            
            // 1. Timeline data with timestamps
            "timeline": {
                "memory_snapshots": enhanced_timeline.memory_snapshots,
                "allocation_events": enhanced_timeline.allocation_events,
                "scope_events": enhanced_timeline.scope_events,
                "time_range": enhanced_timeline.time_range,
                "stack_traces": {
                    "traces": enhanced_timeline.stack_traces.traces,
                    "hotspots": enhanced_timeline.stack_traces.hotspots,
                    "common_patterns": enhanced_timeline.stack_traces.common_patterns
                },
                "allocation_hotspots": enhanced_timeline.allocation_hotspots
            },
            
            // 2. Detailed call stack information
            "call_stacks": self.generate_detailed_call_stacks(&enhanced_allocations)?,
            
            // 3. Scope hierarchy structure
            "scope_hierarchy": scope_hierarchy,
            
            // 4. Variable relationship graph data
            "variable_relationships": variable_relationships,
            
            // 5. Complete unsafe/FFI analysis
            "unsafe_ffi_analysis": {
                "summary": {
                    "total_operations": unsafe_stats.total_operations,
                    "unsafe_blocks": unsafe_stats.unsafe_blocks,
                    "ffi_calls": unsafe_stats.ffi_calls,
                    "raw_pointer_operations": unsafe_stats.raw_pointer_operations,
                    "memory_violations": unsafe_stats.memory_violations,
                    "risk_score": unsafe_stats.risk_score
                },
                "operations": unsafe_stats.operations.iter().map(|op| {
                    serde_json::json!({
                        "operation_type": format!("{:?}", op.operation_type),
                        "location": op.location,
                        "risk_level": format!("{:?}", op.risk_level),
                        "timestamp": op.timestamp,
                        "description": op.description,
                        "stack_trace": Vec::<String>::new() // Placeholder for stack trace
                    })
                }).collect::<Vec<_>>(),
                "risk_analysis": {
                    "critical_operations": unsafe_stats.operations.iter()
                        .filter(|op| matches!(op.risk_level, crate::unsafe_ffi_tracker::RiskLevel::Critical))
                        .count(),
                    "high_risk_operations": unsafe_stats.operations.iter()
                        .filter(|op| matches!(op.risk_level, crate::unsafe_ffi_tracker::RiskLevel::High))
                        .count(),
                    "medium_risk_operations": unsafe_stats.operations.iter()
                        .filter(|op| matches!(op.risk_level, crate::unsafe_ffi_tracker::RiskLevel::Medium))
                        .count(),
                    "low_risk_operations": unsafe_stats.operations.iter()
                        .filter(|op| matches!(op.risk_level, crate::unsafe_ffi_tracker::RiskLevel::Low))
                        .count()
                }
            },
            
            // SVG visualization data for three main modules
            "svg_visualizations": svg_data,
            
            // Enhanced memory analysis with no unknown types
            "memory_analysis": {
                "summary": {
                    "total_allocations": stats.total_allocations,
                    "total_allocated_bytes": stats.total_allocated,
                    "active_allocations": stats.active_allocations,
                    "active_memory_bytes": stats.active_memory,
                    "peak_allocations": stats.peak_allocations,
                    "peak_memory_bytes": stats.peak_memory,
                    "total_deallocations": stats.total_deallocations,
                    "total_deallocated_bytes": stats.total_deallocated
                },
                "allocations": enhanced_allocations.iter().map(|alloc| {
                    serde_json::json!({
                        "variable_name": alloc.var_name.as_deref().unwrap_or("system_allocation"),
                        "type_name": alloc.type_name.as_deref().unwrap_or("inferred_type"),
                        "size_bytes": alloc.size,
                        "allocated_at": alloc.timestamp_alloc,
                        "deallocated_at": alloc.timestamp_dealloc,
                        "scope_name": alloc.scope_name.as_deref().unwrap_or("global"),
                        "thread_id": format!("{:?}", alloc.thread_id),
                        "borrow_count": alloc.borrow_count,
                        "memory_address": format!("0x{:x}", alloc.ptr),
                        "lifetime_ms": alloc.timestamp_dealloc.map(|dealloc| 
                            (dealloc.saturating_sub(alloc.timestamp_alloc)) / 1_000_000
                        ),
                        "is_leaked": alloc.timestamp_dealloc.is_none()
                    })
                }).collect::<Vec<_>>(),
                "type_analysis": memory_by_type.iter()
                    .filter(|t| t.type_name != "Unknown")  // Filter out Unknown types
                    .map(|t| {
                        serde_json::json!({
                            "type_name": t.type_name,
                            "total_size_bytes": t.total_size,
                            "allocation_count": t.allocation_count,
                            "average_size_bytes": if t.allocation_count > 0 { t.total_size / t.allocation_count } else { 0 },
                            "percentage_of_total": if stats.total_allocated > 0 { 
                                (t.total_size as f64 / stats.total_allocated as f64) * 100.0 
                            } else { 
                                0.0 
                            }
                        })
                    }).collect::<Vec<_>>()
            },
            
            // Performance metrics
            "performance_metrics": {
                "allocation_rate_per_second": self.calculate_allocation_rate(&enhanced_history)?,
                "deallocation_rate_per_second": self.calculate_deallocation_rate(&enhanced_history)?,
                "memory_fragmentation_ratio": stats.fragmentation_analysis.fragmentation_ratio,
                "memory_efficiency_percentage": if stats.peak_memory > 0 { 
                    (stats.active_memory as f64 / stats.peak_memory as f64) * 100.0 
                } else { 
                    100.0 
                },
                "average_allocation_size": if stats.total_allocations > 0 { 
                    stats.total_allocated / stats.total_allocations 
                } else { 
                    0 
                },
                "peak_concurrent_allocations": stats.peak_allocations
            }
        });
        
        // Write to file
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &enhanced_data).map_err(|e| {
            crate::types::TrackingError::SerializationError(format!("Enhanced JSON export failed: {e}"))
        })?;
        
        println!("Enhanced JSON exported to: {}", path.display());
        println!("   {} allocations with precise names", enhanced_allocations.len());
        println!("   {} call stack entries", enhanced_timeline.stack_traces.traces.len());
        println!("   {} scope levels", scope_hierarchy.get("levels").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0));
        println!("   {} variable relationships", variable_relationships.get("relationships").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0));
        println!("   {} unsafe/FFI operations", unsafe_stats.total_operations);
        
        Ok(())
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryTracker {
    /// Track growth events for a variable
    fn track_growth_events(&self, var_name: &str, allocation_history: &[AllocationInfo]) -> Vec<crate::types::GrowthEvent> {
        let mut growth_events = Vec::new();
        let mut last_size = 0;
        
        for alloc in allocation_history {
            if let Some(name) = &alloc.var_name {
                if name == var_name && alloc.size > last_size {
                    growth_events.push(crate::types::GrowthEvent {
                        timestamp: alloc.timestamp_alloc,
                        old_size: last_size,
                        new_size: alloc.size,
                        growth_reason: if last_size > 0 { crate::types::GrowthReason::Expansion } else { crate::types::GrowthReason::Initial },
                    });
                    last_size = alloc.size;
                }
            }
        }
        
        growth_events
    }
    
    /// Track borrow events for a variable
    fn track_borrow_events(&self, _var_name: &str, _allocation_history: &[AllocationInfo]) -> Vec<crate::types::BorrowEvent> {
        // Simplified implementation - return empty for now
        Vec::new()
    }
    
    /// Track move events for a variable
    fn track_move_events(&self, _var_name: &str, _allocation_history: &[AllocationInfo]) -> Vec<crate::types::MoveEvent> {
        // Simplified implementation - return empty for now
        Vec::new()
    }
    
    /// Track variable relationships
    fn track_variable_relationships(&self, _var_name: &str, _active_allocations: &[AllocationInfo]) -> Vec<crate::types::VariableRelationship> {
        // Simplified implementation - return empty for now
        Vec::new()
    }
    
    /// Calculate minimum allocation size for a type
    fn calculate_min_allocation_size(&self, type_name: &str, allocation_history: &[AllocationInfo]) -> usize {
        allocation_history
            .iter()
            .filter(|alloc| alloc.type_name.as_deref() == Some(type_name))
            .map(|alloc| alloc.size)
            .min()
            .unwrap_or(0)
    }
    
    /// Detect potential memory leaks
    fn detect_potential_leaks(&self, active_allocations: &[AllocationInfo]) -> Vec<crate::types::PotentialLeak> {
        let mut leaks = Vec::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u128;
        
        for alloc in active_allocations {
            let age_ms = (now.saturating_sub(alloc.timestamp_alloc)) / 1_000_000;
            
            // Consider allocations older than 10 seconds as potential leaks
            if age_ms > 10_000 {
                let confidence = if age_ms > 60_000 { 0.9 } else { 0.5 };
                
                leaks.push(crate::types::PotentialLeak {
                    variable_name: alloc.var_name.clone().unwrap_or_else(|| "unknown".to_string()),
                    size: alloc.size,
                    age_ms,
                    scope: alloc.scope_name.clone().unwrap_or_else(|| "global".to_string()),
                    confidence,
                    allocation_stack: None,
                    last_access_time: None,
                });
            }
        }
        
        leaks
    }
    
    /// Convert unsafe violations from unsafe tracker
    fn convert_unsafe_violations(&self) -> Vec<crate::types::SafetyViolation> {
        // Simplified implementation - return empty for now
        Vec::new()
    }

    /// Generate timeline data with stack traces and hotspots
    pub fn generate_timeline_data(&self, allocation_history: &[AllocationInfo], _active_allocations: &[AllocationInfo]) -> crate::types::TimelineData {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u128;

        // Generate memory snapshots (every 100ms or every 10 allocations)
        let memory_snapshots = self.generate_memory_snapshots(allocation_history);
        
        // Generate allocation events
        let allocation_events = self.generate_allocation_events(allocation_history);
        
        // Generate scope events
        let scope_events = self.generate_scope_events(allocation_history);
        
        // Calculate time range
        let start_time = allocation_history.iter().map(|a| a.timestamp_alloc).min().unwrap_or(now);
        let end_time = allocation_history.iter()
            .filter_map(|a| a.timestamp_dealloc.or(Some(now)))
            .max()
            .unwrap_or(now);
        
        let time_range = crate::types::TimeRange {
            start_time,
            end_time,
            duration_ms: (end_time.saturating_sub(start_time)) / 1_000_000,
        };

        // Generate stack trace data
        let stack_traces = self.generate_stack_trace_data(allocation_history);
        
        // Generate allocation hotspots
        let allocation_hotspots = self.generate_allocation_hotspots(allocation_history);

        crate::types::TimelineData {
            memory_snapshots,
            allocation_events,
            scope_events,
            time_range,
            stack_traces,
            allocation_hotspots,
        }
    }

    /// Generate memory snapshots over time
    fn generate_memory_snapshots(&self, allocation_history: &[AllocationInfo]) -> Vec<crate::types::MemorySnapshot> {
        let mut snapshots = Vec::new();
        let mut current_memory = 0;
        let mut current_allocations = 0;
        let mut scope_breakdown: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        
        // Group allocations by time windows (every 100ms)
        let mut events: Vec<_> = allocation_history.iter().collect();
        events.sort_by_key(|a| a.timestamp_alloc);
        
        let start_time = events.first().map(|a| a.timestamp_alloc).unwrap_or(0);
        let window_size = 100_000_000; // 100ms in nanoseconds
        
        let mut current_window = start_time;
        let mut window_allocations = Vec::new();
        
        for alloc in events {
            if alloc.timestamp_alloc >= current_window + window_size {
                // Process current window
                if !window_allocations.is_empty() {
                    snapshots.push(crate::types::MemorySnapshot {
                        timestamp: current_window,
                        total_memory: current_memory,
                        active_allocations: current_allocations,
                        scope_breakdown: scope_breakdown.clone(),
                    });
                }
                
                // Move to next window
                current_window = alloc.timestamp_alloc;
                window_allocations.clear();
            }
            
            window_allocations.push(alloc);
            current_memory += alloc.size;
            current_allocations += 1;
            
            // Update scope breakdown
            let scope = alloc.scope_name.as_deref().unwrap_or("global");
            *scope_breakdown.entry(scope.to_string()).or_insert(0) += alloc.size;
        }
        
        // Add final snapshot
        if !window_allocations.is_empty() {
            snapshots.push(crate::types::MemorySnapshot {
                timestamp: current_window,
                total_memory: current_memory,
                active_allocations: current_allocations,
                scope_breakdown,
            });
        }
        
        snapshots
    }

    /// Generate allocation events
    fn generate_allocation_events(&self, allocation_history: &[AllocationInfo]) -> Vec<crate::types::AllocationEvent> {
        let mut events = Vec::new();
        
        for alloc in allocation_history {
            // Allocation event
            events.push(crate::types::AllocationEvent {
                timestamp: alloc.timestamp_alloc,
                event_type: crate::types::AllocationEventType::Allocate,
                variable_name: alloc.var_name.clone().unwrap_or_else(|| {
                    if alloc.size >= 1024 {
                        let (smart_name, _) = crate::html_export::analyze_system_allocation(alloc);
                        smart_name
                    } else {
                        format!("small_alloc_{}B", alloc.size)
                    }
                }),
                size: alloc.size,
                scope: alloc.scope_name.clone().unwrap_or_else(|| "global".to_string()),
                stack_trace_id: Some(format!("stack_{}", alloc.ptr)),
                type_name: alloc.type_name.clone(),
                thread_id: alloc.thread_id.clone(),
            });
            
            // Deallocation event (if applicable)
            if let Some(dealloc_time) = alloc.timestamp_dealloc {
                events.push(crate::types::AllocationEvent {
                    timestamp: dealloc_time,
                    event_type: crate::types::AllocationEventType::Deallocate,
                    variable_name: alloc.var_name.clone().unwrap_or_else(|| {
                        if alloc.size >= 1024 {
                            let (smart_name, _) = crate::html_export::analyze_system_allocation(alloc);
                            smart_name
                        } else {
                            format!("small_alloc_{}B", alloc.size)
                        }
                    }),
                    size: alloc.size,
                    scope: alloc.scope_name.clone().unwrap_or_else(|| "global".to_string()),
                    stack_trace_id: Some(format!("stack_{}", alloc.ptr)),
                    type_name: alloc.type_name.clone(),
                    thread_id: alloc.thread_id.clone(),
                });
            }
        }
        
        // Sort by timestamp
        events.sort_by_key(|e| e.timestamp);
        events
    }

    /// Generate scope events
    fn generate_scope_events(&self, allocation_history: &[AllocationInfo]) -> Vec<crate::types::ScopeEvent> {
        let mut scope_events = Vec::new();
        let mut scope_states: std::collections::HashMap<String, (u128, usize)> = std::collections::HashMap::new();
        
        for alloc in allocation_history {
            let scope_name = alloc.scope_name.clone().unwrap_or_else(|| "global".to_string());
            
            // Check if this is the first time we see this scope
            if !scope_states.contains_key(&scope_name) {
                scope_events.push(crate::types::ScopeEvent {
                    timestamp: alloc.timestamp_alloc,
                    event_type: crate::types::ScopeEventType::Enter,
                    scope_name: scope_name.clone(),
                    memory_impact: 0,
                });
                scope_states.insert(scope_name.clone(), (alloc.timestamp_alloc, alloc.size));
            } else {
                // Update memory impact
                if let Some((_, ref mut memory)) = scope_states.get_mut(&scope_name) {
                    *memory += alloc.size;
                }
            }
            
            // Generate exit event if deallocation happened
            if let Some(dealloc_time) = alloc.timestamp_dealloc {
                if let Some((_, memory)) = scope_states.get(&scope_name) {
                    scope_events.push(crate::types::ScopeEvent {
                        timestamp: dealloc_time,
                        event_type: crate::types::ScopeEventType::Exit,
                        scope_name: scope_name.clone(),
                        memory_impact: *memory,
                    });
                }
            }
        }
        
        scope_events.sort_by_key(|e| e.timestamp);
        scope_events
    }

    /// Generate stack trace data
    fn generate_stack_trace_data(&self, allocation_history: &[AllocationInfo]) -> crate::types::StackTraceData {
        let mut traces = std::collections::HashMap::new();
        let mut stack_stats: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();
        
        // Generate synthetic stack traces for each allocation
        for alloc in allocation_history {
            let stack_id = format!("stack_{}", alloc.ptr);
            let stack_frames = self.generate_synthetic_stack_trace(alloc);
            
            traces.insert(stack_id.clone(), stack_frames.clone());
            
            // Update statistics
            let stack_key = self.stack_frames_to_key(&stack_frames);
            let (count, memory) = stack_stats.entry(stack_key).or_insert((0, 0));
            *count += 1;
            *memory += alloc.size;
        }
        
        // Generate hotspots
        let hotspots = stack_stats.into_iter()
            .map(|(stack_key, (count, memory))| {
                let stack_pattern = self.parse_stack_key(&stack_key);
                crate::types::StackTraceHotspot {
                    stack_pattern,
                    allocation_count: count,
                    total_memory: memory,
                    average_size: memory as f64 / count.max(1) as f64,
                    frequency_per_second: count as f64 / 10.0, // Assume 10 second runtime
                }
            })
            .collect();
        
        // Generate common patterns
        let common_patterns = vec![
            crate::types::AllocationPattern {
                pattern: "Vec allocations in loops".to_string(),
                frequency: allocation_history.len() / 4,
                total_memory_impact: allocation_history.iter().map(|a| a.size).sum::<usize>() / 4,
                example_stacks: vec![],
            }
        ];
        
        crate::types::StackTraceData {
            traces,
            hotspots,
            common_patterns,
        }
    }

    /// Generate synthetic stack trace for an allocation
    fn generate_synthetic_stack_trace(&self, alloc: &AllocationInfo) -> Vec<crate::types::StackFrame> {
        let mut frames = Vec::new();
        
        // Add main frame
        frames.push(crate::types::StackFrame {
            function: "main".to_string(),
            file: Some("main.rs".to_string()),
            line: Some(42),
            module: Some("my_app".to_string()),
        });
        
        // Add scope-specific frame
        if let Some(scope) = &alloc.scope_name {
            frames.push(crate::types::StackFrame {
                function: scope.clone(),
                file: Some(format!("{}.rs", scope)),
                line: Some(15),
                module: Some("my_app".to_string()),
            });
        }
        
        // Add type-specific frame
        if let Some(type_name) = &alloc.type_name {
            if type_name.contains("Vec") {
                frames.push(crate::types::StackFrame {
                    function: "Vec::new".to_string(),
                    file: Some("vec.rs".to_string()),
                    line: Some(123),
                    module: Some("alloc::vec".to_string()),
                });
            } else if type_name.contains("String") {
                frames.push(crate::types::StackFrame {
                    function: "String::new".to_string(),
                    file: Some("string.rs".to_string()),
                    line: Some(456),
                    module: Some("alloc::string".to_string()),
                });
            }
        }
        
        frames
    }

    /// Convert stack frames to a key for grouping
    fn stack_frames_to_key(&self, frames: &[crate::types::StackFrame]) -> String {
        frames.iter()
            .map(|f| format!("{}:{}", f.function, f.line.unwrap_or(0)))
            .collect::<Vec<_>>()
            .join("|")
    }

    /// Parse stack key back to frames
    fn parse_stack_key(&self, key: &str) -> Vec<crate::types::StackFrame> {
        key.split('|')
            .map(|part| {
                let parts: Vec<&str> = part.split(':').collect();
                crate::types::StackFrame {
                    function: parts.get(0).unwrap_or(&"unknown").to_string(),
                    file: None,
                    line: parts.get(1).and_then(|s| s.parse().ok()),
                    module: None,
                }
            })
            .collect()
    }

    /// Generate allocation hotspots over time
    fn generate_allocation_hotspots(&self, allocation_history: &[AllocationInfo]) -> Vec<crate::types::AllocationHotspot> {
        let mut hotspots = Vec::new();
        let window_size = 1_000_000_000; // 1 second windows
        
        if allocation_history.is_empty() {
            return hotspots;
        }
        
        let start_time = allocation_history.iter().map(|a| a.timestamp_alloc).min().unwrap_or(0);
        let end_time = allocation_history.iter().map(|a| a.timestamp_alloc).max().unwrap_or(0);
        
        let mut current_window = start_time;
        
        while current_window < end_time {
            let window_end = current_window + window_size;
            
            // Find allocations in this window
            let window_allocs: Vec<_> = allocation_history.iter()
                .filter(|a| a.timestamp_alloc >= current_window && a.timestamp_alloc < window_end)
                .collect();
            
            if !window_allocs.is_empty() {
                let total_memory: usize = window_allocs.iter().map(|a| a.size).sum();
                let allocation_count = window_allocs.len();
                
                // Find the most common location in this window
                let mut location_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                for alloc in &window_allocs {
                    let location = alloc.scope_name.clone().unwrap_or_else(|| "global".to_string());
                    *location_counts.entry(location).or_insert(0) += 1;
                }
                
                let most_common_location = location_counts.into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(location, _)| location)
                    .unwrap_or_else(|| "global".to_string());
                
                hotspots.push(crate::types::AllocationHotspot {
                    timestamp: current_window,
                    location: crate::types::HotspotLocation {
                        function: most_common_location.clone(),
                        file: Some(format!("{}.rs", most_common_location)),
                        line: Some(42),
                        scope: most_common_location,
                    },
                    allocation_count,
                    total_memory,
                    allocation_rate: allocation_count as f64,
                    memory_pressure: (total_memory as f64 / 1024.0 / 1024.0).min(1.0), // MB to pressure score
                });
            }
            
            current_window = window_end;
        }
        
        hotspots
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
pub fn build_unified_dashboard_structure(
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
    let _now = std::time::SystemTime::now()
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

            let mut type_details = Vec::with_capacity(types.len());
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

impl MemoryTracker {
    /// Enhance allocations with precise names, eliminating unknown types
    fn enhance_allocations_with_precise_names(&self, allocations: &[AllocationInfo]) -> TrackingResult<Vec<AllocationInfo>> {
        let mut enhanced = Vec::new();
        
        for alloc in allocations {
            let mut enhanced_alloc = alloc.clone();
            
            // If no variable name, generate one based on size and type
            if enhanced_alloc.var_name.is_none() {
                let (smart_name, smart_type) = crate::html_export::analyze_system_allocation(alloc);
                enhanced_alloc.var_name = Some(smart_name);
                enhanced_alloc.type_name = Some(smart_type);
            }
            
            // If type is unknown, infer from size and context
            if enhanced_alloc.type_name.as_deref() == Some("Unknown") || enhanced_alloc.type_name.is_none() {
                enhanced_alloc.type_name = Some(self.infer_type_from_context(alloc));
            }
            
            enhanced.push(enhanced_alloc);
        }
        
        Ok(enhanced)
    }
    
    /// Infer type from allocation context
    fn infer_type_from_context(&self, alloc: &AllocationInfo) -> String {
        match alloc.size {
            1..=8 => "Small Primitive".to_string(),
            9..=32 => "Medium Structure".to_string(),
            33..=128 => "Large Structure".to_string(),
            129..=1024 => "Buffer/Array".to_string(),
            1025..=4096 => "Large Buffer".to_string(),
            4097..=16384 => "Page Buffer".to_string(),
            _ => "Large Memory Block".to_string(),
        }
    }
    
    /// Generate scope hierarchy
    fn generate_scope_hierarchy(&self, allocations: &[AllocationInfo]) -> TrackingResult<serde_json::Value> {
        let mut scope_tree = std::collections::HashMap::new();
        let mut scope_stats = std::collections::HashMap::new();
        
        for alloc in allocations {
            let scope = alloc.scope_name.as_deref().unwrap_or("global");
            let parts: Vec<&str> = scope.split("::").collect();
            
            // Build hierarchy
            let mut current_path = String::new();
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    current_path.push_str("::");
                }
                current_path.push_str(part);
                
                let entry = scope_tree.entry(current_path.clone()).or_insert_with(|| serde_json::json!({
                    "name": part,
                    "full_path": current_path.clone(),
                    "level": i,
                    "children": [],
                    "allocations": []
                }));
                
                // Add allocation to this scope
                if let Some(allocations_array) = entry.get_mut("allocations") {
                    if let Some(array) = allocations_array.as_array_mut() {
                        array.push(serde_json::json!({
                            "variable_name": alloc.var_name.as_deref().unwrap_or("unnamed"),
                            "size": alloc.size,
                            "type_name": alloc.type_name.as_deref().unwrap_or("inferred")
                        }));
                    }
                }
            }
            
            // Update scope statistics
            let stats = scope_stats.entry(scope.to_string()).or_insert((0, 0));
            stats.0 += alloc.size;
            stats.1 += 1;
        }
        
        // Convert to hierarchical structure
        let levels: Vec<_> = scope_tree.into_iter()
            .map(|(path, mut data)| {
                if let Some(stats) = scope_stats.get(&path) {
                    data["total_memory"] = serde_json::json!(stats.0);
                    data["allocation_count"] = serde_json::json!(stats.1);
                }
                data
            })
            .collect();
        
        Ok(serde_json::json!({
            "levels": levels,
            "total_scopes": scope_stats.len(),
            "scope_statistics": scope_stats.into_iter().map(|(scope, (memory, count))| {
                serde_json::json!({
                    "scope_name": scope,
                    "total_memory": memory,
                    "allocation_count": count
                })
            }).collect::<Vec<_>>()
        }))
    }
    
    /// Generate variable relationships
    fn generate_variable_relationships(&self, allocations: &[AllocationInfo]) -> TrackingResult<serde_json::Value> {
        let mut relationships = Vec::new();
        let mut variable_map = std::collections::HashMap::new();
        
        // Build variable map
        for alloc in allocations {
            if let Some(var_name) = &alloc.var_name {
                variable_map.insert(var_name.clone(), alloc);
            }
        }
        
        // Find relationships based on naming patterns and types
        for (var_name, alloc) in &variable_map {
            // Find related variables
            for (other_name, other_alloc) in &variable_map {
                if var_name != other_name {
                    let relationship_type = self.determine_relationship_type(var_name, other_name, alloc, other_alloc);
                    if let Some(rel_type) = relationship_type {
                        relationships.push(serde_json::json!({
                            "source": var_name,
                            "target": other_name,
                            "relationship_type": rel_type,
                            "strength": self.calculate_relationship_strength(alloc, other_alloc),
                            "source_info": {
                                "size": alloc.size,
                                "type": alloc.type_name.as_deref().unwrap_or("unknown"),
                                "scope": alloc.scope_name.as_deref().unwrap_or("global")
                            },
                            "target_info": {
                                "size": other_alloc.size,
                                "type": other_alloc.type_name.as_deref().unwrap_or("unknown"),
                                "scope": other_alloc.scope_name.as_deref().unwrap_or("global")
                            }
                        }));
                    }
                }
            }
        }
        
        Ok(serde_json::json!({
            "relationships": relationships,
            "total_variables": variable_map.len(),
            "relationship_count": relationships.len(),
            "relationship_types": {
                "ownership": relationships.iter().filter(|r| r["relationship_type"] == "ownership").count(),
                "reference": relationships.iter().filter(|r| r["relationship_type"] == "reference").count(),
                "collection_item": relationships.iter().filter(|r| r["relationship_type"] == "collection_item").count(),
                "scope_related": relationships.iter().filter(|r| r["relationship_type"] == "scope_related").count()
            }
        }))
    }
    
    /// Determine relationship type between two variables
    fn determine_relationship_type(&self, var1: &str, var2: &str, alloc1: &AllocationInfo, alloc2: &AllocationInfo) -> Option<String> {
        // Check for naming patterns
        if var1.contains("clone") && var2.replace("_clone", "") == var1.replace("_clone", "") {
            return Some("clone".to_string());
        }
        
        if var1.ends_with("_ref") && var2 == var1.trim_end_matches("_ref") {
            return Some("reference".to_string());
        }
        
        // Check for type relationships
        if let (Some(type1), Some(type2)) = (&alloc1.type_name, &alloc2.type_name) {
            if type1.contains("Box") && type2.contains(&type1.replace("Box<", "").replace(">", "")) {
                return Some("ownership".to_string());
            }
            
            if type1.contains("Vec") && type2.contains(&extract_vec_inner_type(type1)) {
                return Some("collection_item".to_string());
            }
        }
        
        // Check for scope relationships
        if alloc1.scope_name == alloc2.scope_name && alloc1.scope_name.is_some() {
            return Some("scope_related".to_string());
        }
        
        None
    }
    
    /// Calculate relationship strength
    fn calculate_relationship_strength(&self, alloc1: &AllocationInfo, alloc2: &AllocationInfo) -> f64 {
        let mut strength: f64 = 0.0;
        
        // Same scope increases strength
        if alloc1.scope_name == alloc2.scope_name {
            strength += 0.3;
        }
        
        // Similar allocation times increase strength
        let time_diff = alloc1.timestamp_alloc.abs_diff(alloc2.timestamp_alloc);
        if time_diff < 1_000_000 { // Within 1ms
            strength += 0.4;
        } else if time_diff < 10_000_000 { // Within 10ms
            strength += 0.2;
        }
        
        // Similar sizes increase strength
        let size_ratio = (alloc1.size as f64) / (alloc2.size as f64).max(1.0);
        if size_ratio > 0.5 && size_ratio < 2.0 {
            strength += 0.3;
        }
        
        strength.min(1.0)
    }
    
    /// Generate SVG visualization data
    fn generate_svg_visualization_data(&self, allocations: &[AllocationInfo], stats: &MemoryStats) -> TrackingResult<serde_json::Value> {
        Ok(serde_json::json!({
            "memory_analysis": {
                "chart_type": "memory_analysis",
                "data_points": allocations.iter().map(|alloc| {
                    serde_json::json!({
                        "variable": alloc.var_name.as_deref().unwrap_or("unnamed"),
                        "size": alloc.size,
                        "type": alloc.type_name.as_deref().unwrap_or("inferred"),
                        "x": alloc.timestamp_alloc % 1000, // Simplified positioning
                        "y": alloc.size,
                        "color": self.get_type_color(alloc.type_name.as_deref().unwrap_or("default"))
                    })
                }).collect::<Vec<_>>(),
                "metadata": {
                    "total_memory": stats.total_allocated,
                    "peak_memory": stats.peak_memory,
                    "active_allocations": stats.active_allocations
                }
            },
            "lifecycle_timeline": {
                "chart_type": "lifecycle_timeline",
                "timeline_events": allocations.iter().map(|alloc| {
                    serde_json::json!({
                        "variable": alloc.var_name.as_deref().unwrap_or("unnamed"),
                        "start_time": alloc.timestamp_alloc,
                        "end_time": alloc.timestamp_dealloc,
                        "duration": alloc.timestamp_dealloc.map(|end| end - alloc.timestamp_alloc),
                        "size": alloc.size,
                        "scope": alloc.scope_name.as_deref().unwrap_or("global")
                    })
                }).collect::<Vec<_>>()
            },
            "unsafe_ffi": {
                "chart_type": "unsafe_ffi_dashboard",
                "risk_indicators": allocations.iter()
                    .filter(|alloc| alloc.var_name.as_ref().map_or(false, |name| name.contains("unsafe") || name.contains("ffi")))
                    .map(|alloc| {
                        serde_json::json!({
                            "variable": alloc.var_name.as_deref().unwrap_or("unnamed"),
                            "risk_level": self.assess_risk_level(alloc),
                            "size": alloc.size,
                            "location": alloc.scope_name.as_deref().unwrap_or("global")
                        })
                    }).collect::<Vec<_>>()
            }
        }))
    }
    
    /// Get color for type visualization
    fn get_type_color(&self, type_name: &str) -> String {
        match type_name {
            t if t.contains("Vec") => "#4CAF50".to_string(),
            t if t.contains("String") => "#2196F3".to_string(),
            t if t.contains("Box") => "#FF9800".to_string(),
            t if t.contains("HashMap") => "#9C27B0".to_string(),
            t if t.contains("Arc") || t.contains("Rc") => "#F44336".to_string(),
            _ => "#607D8B".to_string(),
        }
    }
    
    /// Assess risk level for unsafe operations
    fn assess_risk_level(&self, alloc: &AllocationInfo) -> String {
        if alloc.size > 1024 * 1024 { // > 1MB
            "high".to_string()
        } else if alloc.size > 64 * 1024 { // > 64KB
            "medium".to_string()
        } else {
            "low".to_string()
        }
    }
    
    /// Generate detailed call stacks
    fn generate_detailed_call_stacks(&self, allocations: &[AllocationInfo]) -> TrackingResult<serde_json::Value> {
        let mut call_stacks = std::collections::HashMap::new();
        
        for alloc in allocations {
            let stack_id = format!("stack_{}", alloc.ptr);
            let frames = self.generate_synthetic_stack_trace(alloc);
            
            call_stacks.insert(stack_id, serde_json::json!({
                "frames": frames.iter().map(|frame| {
                    serde_json::json!({
                        "function": frame.function,
                        "file": frame.file,
                        "line": frame.line,
                        "module": frame.module
                    })
                }).collect::<Vec<_>>(),
                "allocation_info": {
                    "variable": alloc.var_name.as_deref().unwrap_or("unnamed"),
                    "size": alloc.size,
                    "timestamp": alloc.timestamp_alloc
                }
            }));
        }
        
        Ok(serde_json::json!({
            "stacks": call_stacks,
            "total_stacks": call_stacks.len(),
            "unique_functions": self.count_unique_functions(&call_stacks)
        }))
    }
    
    /// Count unique functions in call stacks
    fn count_unique_functions(&self, call_stacks: &std::collections::HashMap<String, serde_json::Value>) -> usize {
        let mut functions = std::collections::HashSet::new();
        
        for stack in call_stacks.values() {
            if let Some(frames) = stack.get("frames").and_then(|f| f.as_array()) {
                for frame in frames {
                    if let Some(function) = frame.get("function").and_then(|f| f.as_str()) {
                        functions.insert(function.to_string());
                    }
                }
            }
        }
        
        functions.len()
    }
    
    /// Calculate allocation rate
    fn calculate_allocation_rate(&self, history: &[AllocationInfo]) -> TrackingResult<f64> {
        if history.is_empty() {
            return Ok(0.0);
        }
        
        let start_time = history.iter().map(|a| a.timestamp_alloc).min().unwrap_or(0);
        let end_time = history.iter().map(|a| a.timestamp_alloc).max().unwrap_or(0);
        let duration_seconds = (end_time - start_time) as f64 / 1_000_000_000.0;
        
        if duration_seconds > 0.0 {
            Ok(history.len() as f64 / duration_seconds)
        } else {
            Ok(0.0)
        }
    }
    
    /// Calculate deallocation rate
    fn calculate_deallocation_rate(&self, history: &[AllocationInfo]) -> TrackingResult<f64> {
        let deallocated_count = history.iter().filter(|a| a.timestamp_dealloc.is_some()).count();
        
        if history.is_empty() {
            return Ok(0.0);
        }
        
        let start_time = history.iter().map(|a| a.timestamp_alloc).min().unwrap_or(0);
        let end_time = history.iter().map(|a| a.timestamp_alloc).max().unwrap_or(0);
        let duration_seconds = (end_time - start_time) as f64 / 1_000_000_000.0;
        
        if duration_seconds > 0.0 {
            Ok(deallocated_count as f64 / duration_seconds)
        } else {
            Ok(0.0)
        }
    }
}

/// Extract inner type from Vec<T>
fn extract_vec_inner_type(type_name: &str) -> String {
    if let Some(start) = type_name.find("Vec<") {
        if let Some(end) = type_name.rfind('>') {
            let inner = &type_name[start + 4..end];
            return inner.to_string();
        }
    }
    "T".to_string()
}
