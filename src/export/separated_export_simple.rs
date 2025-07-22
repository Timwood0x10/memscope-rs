//! Simplified Separated JSON Export Implementation
//!
//! This is a working implementation of the separated export functionality
//! that focuses on the core requirements without complex dependencies.

use crate::core::tracker::MemoryTracker;
use crate::core::types::{AllocationInfo, MemoryStats, TrackingResult, TypeMemoryUsage};
use crate::variable_registry::VariableRegistry;
// Removed unused import
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Result of separated export operation
#[derive(Debug, Clone)]
pub struct SeparatedExportResult {
    /// Path to variable relationships JSON file
    pub variable_relationships_path: PathBuf,
    /// Path to memory analysis JSON file
    pub memory_analysis_path: PathBuf,
    /// Path to lifetime analysis JSON file
    pub lifetime_analysis_path: PathBuf,
    /// Path to unsafe/FFI analysis JSON file
    pub unsafe_ffi_analysis_path: PathBuf,
    /// Export time
    pub export_time: Duration,
    /// Total allocations processed
    pub total_allocations_processed: usize,
}

/// Main entry point for simplified separated JSON export
pub fn export_separated_json_simple<P: AsRef<Path>>(
    tracker: &MemoryTracker,
    base_path: P,
) -> TrackingResult<SeparatedExportResult> {
    let base_path = base_path.as_ref();
    let start_time = Instant::now();

    tracing::info!(
        "🚀 Starting simplified separated JSON export to: {}",
        base_path.display()
    );

    // Create base directory if it doesn't exist
    if let Some(parent) = base_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Extract data from tracker
    let active_allocations = tracker.get_active_allocations()?;
    let allocation_history = tracker.get_allocation_history()?;
    let memory_stats = tracker.get_stats()?;
    let memory_by_type = tracker.get_memory_by_type()?;
    let variable_registry = VariableRegistry::get_all_variables();

    let total_allocations = active_allocations.len() + allocation_history.len();

    // Get base filename without extension
    let base_name = base_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("analysis");
    let parent_dir = base_path.parent().unwrap_or(Path::new("."));

    // Generate the four specialized JSON files sequentially
    let relationships_path = generate_variable_relationships_simple(
        &active_allocations,
        &allocation_history,
        &variable_registry,
        parent_dir,
        base_name,
    )?;

    let memory_path = generate_memory_analysis_simple(
        &memory_stats,
        &memory_by_type,
        &active_allocations,
        &allocation_history,
        parent_dir,
        base_name,
    )?;

    let lifetime_path = generate_lifetime_analysis_simple(
        &active_allocations,
        &allocation_history,
        parent_dir,
        base_name,
    )?;

    let unsafe_path = generate_unsafe_ffi_analysis_simple(
        &active_allocations,
        &allocation_history,
        parent_dir,
        base_name,
    )?;

    let total_time = start_time.elapsed();

    tracing::info!("✅ Separated JSON export completed in {:?}", total_time);
    tracing::info!("📁 Generated files:");
    tracing::info!(
        "  🔗 Variable Relationships: {}",
        relationships_path.display()
    );
    tracing::info!("  📊 Memory Analysis: {}", memory_path.display());
    tracing::info!("  ⏱️ Lifetime Analysis: {}", lifetime_path.display());
    tracing::info!("  ⚠️ Unsafe/FFI Analysis: {}", unsafe_path.display());

    Ok(SeparatedExportResult {
        variable_relationships_path: relationships_path,
        memory_analysis_path: memory_path,
        lifetime_analysis_path: lifetime_path,
        unsafe_ffi_analysis_path: unsafe_path,
        export_time: total_time,
        total_allocations_processed: total_allocations,
    })
}

/// Generate variable_relationships.json with simplified relationship detection
fn generate_variable_relationships_simple(
    active_allocations: &[AllocationInfo],
    allocation_history: &[AllocationInfo],
    registry: &HashMap<usize, crate::variable_registry::VariableInfo>,
    parent_dir: &Path,
    base_name: &str,
) -> TrackingResult<PathBuf> {
    let start_time = Instant::now();
    tracing::debug!("🔗 Generating variable relationships JSON...");

    // Build simplified nodes from allocations
    let mut nodes = Vec::new();
    let mut relationships = Vec::new();

    // Process all allocations
    for alloc in active_allocations.iter().chain(allocation_history.iter()) {
        let node_id = format!("0x{:x}", alloc.ptr);

        // Get variable info from registry or infer
        let (name, type_name, category) = if let Some(var_info) = registry.get(&alloc.ptr) {
            (
                var_info.var_name.clone(),
                var_info.type_name.clone(),
                "user_variable",
            )
        } else if let (Some(var_name), Some(type_name)) = (&alloc.var_name, &alloc.type_name) {
            (var_name.clone(), type_name.clone(), "user_variable")
        } else {
            let (inferred_name, inferred_type) =
                crate::variable_registry::VariableRegistry::infer_allocation_info_cached(alloc);
            (inferred_name, inferred_type, "system_allocation")
        };

        let scope = alloc.scope_name.as_deref().unwrap_or("main").to_string();

        nodes.push(serde_json::json!({
            "id": node_id,
            "name": name,
            "type": type_name,
            "size": alloc.size,
            "scope": scope,
            "is_active": alloc.timestamp_dealloc.is_none(),
            "category": category,
            "created_at": alloc.timestamp_alloc,
            "destroyed_at": alloc.timestamp_dealloc
        }));

        // Simple relationship detection based on smart pointers
        if let Some(smart_ptr_info) = &alloc.smart_pointer_info {
            // Add clone relationships
            for &clone_addr in &smart_ptr_info.clones {
                let clone_id = format!("0x{:x}", clone_addr);
                relationships.push(serde_json::json!({
                    "source": node_id,
                    "target": clone_id,
                    "type": "clones",
                    "weight": 1.0
                }));
            }

            // Add parent relationship if cloned from another
            if let Some(parent_addr) = smart_ptr_info.cloned_from {
                let parent_id = format!("0x{:x}", parent_addr);
                relationships.push(serde_json::json!({
                    "source": parent_id,
                    "target": node_id,
                    "type": "clones",
                    "weight": 1.0
                }));
            }
        }
    }

    // Group by scope for clusters
    let mut scope_clusters = HashMap::new();
    for node in &nodes {
        let scope = node["scope"].as_str().unwrap_or("unknown");
        scope_clusters
            .entry(scope.to_string())
            .or_insert_with(Vec::new)
            .push(node["id"].clone());
    }

    let clusters: Vec<_> = scope_clusters
        .into_iter()
        .map(|(scope, variables)| {
            serde_json::json!({
                "id": format!("scope_{}", scope),
                "type": "scope",
                "variables": variables,
                "metadata": {
                    "scope_name": scope
                }
            })
        })
        .collect();

    let json_data = serde_json::json!({
        "relationship_graph": {
            "nodes": nodes,
            "relationships": relationships,
            "clusters": clusters,
            "statistics": {
                "total_nodes": nodes.len(),
                "total_relationships": relationships.len(),
                "total_clusters": clusters.len()
            }
        },
        "metadata": {
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "processing_time_ms": start_time.elapsed().as_millis(),
            "export_type": "variable_relationships"
        }
    });

    let file_path = parent_dir.join(format!("{}_variable_relationships.json", base_name));
    let mut file = File::create(&file_path)?;
    file.write_all(serde_json::to_string_pretty(&json_data)?.as_bytes())?;

    tracing::debug!(
        "✅ Variable relationships JSON generated in {:?}",
        start_time.elapsed()
    );
    Ok(file_path)
}

/// Generate memory_analysis.json with core memory statistics
fn generate_memory_analysis_simple(
    memory_stats: &MemoryStats,
    memory_by_type: &[TypeMemoryUsage],
    active_allocations: &[AllocationInfo],
    allocation_history: &[AllocationInfo],
    parent_dir: &Path,
    base_name: &str,
) -> TrackingResult<PathBuf> {
    let start_time = Instant::now();
    tracing::debug!("📊 Generating memory analysis JSON...");

    let json_data = serde_json::json!({
        "memory_statistics": {
            "total_allocated": memory_stats.total_allocated,
            "active_memory": memory_stats.active_memory,
            "peak_memory": memory_stats.peak_memory,
            "total_allocations": memory_stats.total_allocations,
            "active_allocations": memory_stats.active_allocations,
            "peak_allocations": memory_stats.peak_allocations
        },
        "memory_by_type": memory_by_type,
        "allocation_summary": {
            "active_count": active_allocations.len(),
            "history_count": allocation_history.len(),
            "total_count": active_allocations.len() + allocation_history.len()
        },
        "metadata": {
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "processing_time_ms": start_time.elapsed().as_millis(),
            "export_type": "memory_analysis"
        }
    });

    let file_path = parent_dir.join(format!("{}_memory_analysis.json", base_name));
    let mut file = File::create(&file_path)?;
    file.write_all(serde_json::to_string_pretty(&json_data)?.as_bytes())?;

    tracing::debug!(
        "✅ Memory analysis JSON generated in {:?}",
        start_time.elapsed()
    );
    Ok(file_path)
}

/// Generate lifetime_analysis.json with temporal patterns
fn generate_lifetime_analysis_simple(
    active_allocations: &[AllocationInfo],
    allocation_history: &[AllocationInfo],
    parent_dir: &Path,
    base_name: &str,
) -> TrackingResult<PathBuf> {
    let start_time = Instant::now();
    tracing::debug!("⏱️ Generating lifetime analysis JSON...");

    // Create timeline events
    let mut timeline_events = Vec::new();

    for alloc in active_allocations.iter().chain(allocation_history.iter()) {
        timeline_events.push(serde_json::json!({
            "type": "allocation",
            "timestamp": alloc.timestamp_alloc,
            "ptr": alloc.ptr,
            "size": alloc.size
        }));

        if let Some(dealloc_time) = alloc.timestamp_dealloc {
            timeline_events.push(serde_json::json!({
                "type": "deallocation",
                "timestamp": dealloc_time,
                "ptr": alloc.ptr,
                "lifetime_ms": (dealloc_time - alloc.timestamp_alloc) / 1_000_000
            }));
        }
    }

    // Sort by timestamp
    timeline_events.sort_by_key(|event| event["timestamp"].as_u64().unwrap_or(0));

    // Calculate lifecycle patterns
    let lifetimes: Vec<u64> = allocation_history
        .iter()
        .filter_map(|a| {
            a.timestamp_dealloc
                .map(|d| (d - a.timestamp_alloc) / 1_000_000)
        })
        .collect();

    let avg_lifetime = if !lifetimes.is_empty() {
        lifetimes.iter().sum::<u64>() / lifetimes.len() as u64
    } else {
        0
    };

    let json_data = serde_json::json!({
        "timeline_events": timeline_events,
        "lifecycle_patterns": {
            "short_lived": allocation_history.iter().filter(|a| {
                a.timestamp_dealloc.map_or(false, |d| (d - a.timestamp_alloc) < 1_000_000)
            }).count(),
            "long_lived": active_allocations.len(),
            "average_lifetime_ms": avg_lifetime,
            "max_lifetime_ms": lifetimes.iter().max().copied().unwrap_or(0),
            "min_lifetime_ms": lifetimes.iter().min().copied().unwrap_or(0)
        },
        "allocation_timeline": active_allocations.iter().chain(allocation_history.iter())
            .map(|alloc| serde_json::json!({
                "ptr": alloc.ptr,
                "start": alloc.timestamp_alloc,
                "end": alloc.timestamp_dealloc,
                "size": alloc.size,
                "is_active": alloc.timestamp_dealloc.is_none()
            }))
            .collect::<Vec<_>>(),
        "metadata": {
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "processing_time_ms": start_time.elapsed().as_millis(),
            "export_type": "lifetime_analysis"
        }
    });

    let file_path = parent_dir.join(format!("{}_lifetime_analysis.json", base_name));
    let mut file = File::create(&file_path)?;
    file.write_all(serde_json::to_string_pretty(&json_data)?.as_bytes())?;

    tracing::debug!(
        "✅ Lifetime analysis JSON generated in {:?}",
        start_time.elapsed()
    );
    Ok(file_path)
}

/// Generate unsafe_ffi_analysis.json with safety analysis
fn generate_unsafe_ffi_analysis_simple(
    active_allocations: &[AllocationInfo],
    allocation_history: &[AllocationInfo],
    parent_dir: &Path,
    base_name: &str,
) -> TrackingResult<PathBuf> {
    let start_time = Instant::now();
    tracing::debug!("⚠️ Generating unsafe/FFI analysis JSON...");

    // Analyze circular references using existing functionality
    let all_allocations: Vec<_> = active_allocations
        .iter()
        .chain(allocation_history.iter())
        .cloned()
        .collect();

    let circular_analysis =
        crate::analysis::circular_reference::detect_circular_references(&all_allocations);

    let json_data = serde_json::json!({
        "circular_references": {
            "detected_cycles": circular_analysis.circular_references,
            "total_smart_pointers": circular_analysis.total_smart_pointers,
            "pointers_in_cycles": circular_analysis.pointers_in_cycles,
            "total_leaked_memory": circular_analysis.total_leaked_memory,
            "statistics": circular_analysis.statistics
        },
        "safety_analysis": {
            "potential_leaks": active_allocations.len(),
            "deallocated_count": allocation_history.iter()
                .filter(|a| a.timestamp_dealloc.is_some())
                .count(),
            "smart_pointer_count": all_allocations.iter()
                .filter(|a| a.smart_pointer_info.is_some())
                .count()
        },
        "risk_assessment": {
            "overall_risk_level": if !circular_analysis.circular_references.is_empty() {
                "HIGH"
            } else if active_allocations.len() > 100 {
                "MEDIUM"
            } else {
                "LOW"
            },
            "recommendations": if !circular_analysis.circular_references.is_empty() {
                vec!["Consider using Weak references to break circular dependencies"]
            } else {
                vec!["Memory usage appears normal"]
            }
        },
        "metadata": {
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "processing_time_ms": start_time.elapsed().as_millis(),
            "export_type": "unsafe_ffi_analysis"
        }
    });

    let file_path = parent_dir.join(format!("{}_unsafe_ffi_analysis.json", base_name));
    let mut file = File::create(&file_path)?;
    file.write_all(serde_json::to_string_pretty(&json_data)?.as_bytes())?;

    tracing::debug!(
        "✅ Unsafe/FFI analysis JSON generated in {:?}",
        start_time.elapsed()
    );
    Ok(file_path)
}
