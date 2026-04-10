//! Export functionality for the render engine
//!
//! This module provides export functionality for memory tracking data,
//! including JSON export, lifetime analysis, and variable relationships.

use tracing::{debug, warn};

use crate::analysis::is_virtual_pointer;
use crate::analysis::memory_passport_tracker::MemoryPassportTracker;
use crate::analysis::ownership_graph::{EdgeKind, ObjectId, OwnershipGraph, OwnershipOp};
use crate::capture::platform::memory_info::PlatformMemoryInfo;
use crate::core::{MemScopeError, MemScopeResult};
use crate::render_engine::dashboard::DashboardRenderer;
use crate::snapshot::{ActiveAllocation, MemorySnapshot, ThreadMemoryStats};
use crate::tracker::Tracker;
use rayon::prelude::*;
use serde_json::json;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    sync::Arc,
};

/// Optimization level for JSON export
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptimizationLevel {
    /// Low optimization - maximum compatibility
    Low,
    /// Medium optimization - balanced
    #[default]
    Medium,
    /// High optimization - maximum performance
    High,
    /// Maximum optimization - aggressive optimization
    Maximum,
}

/// Schema validator for JSON export
#[derive(Debug, Clone, Default)]
pub struct SchemaValidator {
    strict_mode: bool,
}

impl SchemaValidator {
    pub fn new() -> Self {
        Self { strict_mode: false }
    }

    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    pub fn validate(&self, data: &serde_json::Value) -> Result<(), String> {
        if !data.is_object() {
            return Err("Export data must be a JSON object".to_string());
        }

        let obj = data.as_object().ok_or("Invalid JSON object")?;

        if self.strict_mode {
            let required_fields = ["timestamp", "allocations", "stats"];
            for field in &required_fields {
                if !obj.contains_key(*field) {
                    return Err(format!("Missing required field: {}", field));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ExportJsonOptions {
    pub parallel_processing: bool,
    pub buffer_size: usize,
    pub use_compact_format: Option<bool>,
    pub enable_type_cache: bool,
    pub batch_size: usize,
    pub streaming_writer: bool,
    pub schema_validation: bool,
    pub adaptive_optimization: bool,
    pub max_cache_size: usize,
    pub security_analysis: bool,
    pub include_low_severity: bool,
    pub integrity_hashes: bool,
    pub fast_export_mode: bool,
    pub auto_fast_export_threshold: Option<usize>,
    pub thread_count: Option<usize>,
}

impl Default for ExportJsonOptions {
    fn default() -> Self {
        Self {
            parallel_processing: true,
            buffer_size: 256 * 1024,
            use_compact_format: None,
            enable_type_cache: true,
            batch_size: 1000,
            streaming_writer: true,
            schema_validation: false,
            adaptive_optimization: true,
            max_cache_size: 10_000,
            security_analysis: false,
            include_low_severity: false,
            integrity_hashes: false,
            fast_export_mode: false,
            auto_fast_export_threshold: Some(10_000),
            thread_count: None,
        }
    }
}

impl ExportJsonOptions {
    pub fn fast_export_mode(mut self, enabled: bool) -> Self {
        self.fast_export_mode = enabled;
        self
    }

    pub fn security_analysis(mut self, enabled: bool) -> Self {
        self.security_analysis = enabled;
        self
    }

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

    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    pub fn adaptive_optimization(mut self, enabled: bool) -> Self {
        self.adaptive_optimization = enabled;
        self
    }

    pub fn max_cache_size(mut self, size: usize) -> Self {
        self.max_cache_size = size;
        self
    }

    pub fn include_low_severity(mut self, include: bool) -> Self {
        self.include_low_severity = include;
        self
    }

    pub fn thread_count(mut self, count: Option<usize>) -> Self {
        self.thread_count = count;
        self
    }
}

pub fn export_snapshot_to_json(
    snapshot: &MemorySnapshot,
    output_path: &Path,
    options: &ExportJsonOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let allocations: Vec<&ActiveAllocation> = snapshot.active_allocations.values().collect();
    let processed = process_allocations(&allocations, options)?;

    // Use output_path as the base directory for generated files
    let output_dir = if output_path.extension().is_some() {
        // If output_path has an extension, treat it as a file and use its parent
        output_path.parent().unwrap_or(Path::new("."))
    } else {
        output_path
    };

    generate_memory_analysis_json(output_dir, &processed, options)?;
    generate_lifetime_json(output_dir, &processed, options)?;
    generate_thread_analysis_json(output_dir, &snapshot.thread_stats, options)?;

    Ok(())
}

fn process_allocations(
    allocations: &[&ActiveAllocation],
    options: &ExportJsonOptions,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    if options.parallel_processing && allocations.len() > options.batch_size {
        let chunk_size = (allocations.len() / num_cpus::get()).max(1);
        Ok(allocations
            .par_chunks(chunk_size)
            .flat_map(process_allocation_batch)
            .collect())
    } else {
        Ok(process_allocation_batch(allocations))
    }
}

fn process_allocation_batch(allocations: &[&ActiveAllocation]) -> Vec<serde_json::Value> {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    allocations
        .iter()
        .map(|alloc| {
            let type_info = get_or_compute_type_info(
                alloc.type_name.as_deref().unwrap_or("unknown"),
                alloc.size,
            );

            let lifetime_ms = if alloc.allocated_at > 0 {
                (current_time.saturating_sub(alloc.allocated_at)) / 1_000_000
            } else {
                0
            };

            let address = match alloc.ptr {
                Some(ptr) => format!("0x{:x}", ptr),
                None => "N/A".to_string(),
            };

            let mut entry = json!({
                "address": address,
                "size": alloc.size,
                "type": type_info,
                "timestamp": alloc.allocated_at,
                "thread_id": alloc.thread_id,
                "lifetime_ms": lifetime_ms,
            });

            if let Some(ref var_name) = alloc.var_name {
                entry["var_name"] = serde_json::json!(var_name);
            }

            if let Some(ref type_name) = alloc.type_name {
                entry["type_name"] = serde_json::json!(type_name);
            }

            entry
        })
        .collect()
}

fn get_or_compute_type_info(type_name: &str, size: usize) -> String {
    // Check for Vec but not VecDeque
    if (type_name.contains("Vec<") || type_name.contains("vec::Vec<"))
        && !type_name.contains("VecDeque")
    {
        "dynamic_array".to_string()
    } else if type_name == "str"
        || type_name == "String"
        || type_name.contains("&str")
        || type_name.contains("alloc::string::String")
    {
        "string".to_string()
    } else if type_name.contains("Box") || type_name.contains("Rc") || type_name.contains("Arc") {
        "smart_pointer".to_string()
    } else if type_name.contains("[") && type_name.contains("u8") {
        "byte_array".to_string()
    } else if size > 1024 * 1024 {
        "large_buffer".to_string()
    } else {
        "custom".to_string()
    }
}

fn generate_memory_analysis_json<P: AsRef<Path>>(
    output_path: P,
    allocations: &[serde_json::Value],
    options: &ExportJsonOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let total_size: usize = allocations
        .iter()
        .filter_map(|a| a.get("size").and_then(|s| s.as_u64()))
        .map(|s| s as usize)
        .sum();

    let type_distribution: HashMap<String, usize> = {
        let mut dist = HashMap::new();
        for alloc in allocations {
            if let Some(t) = alloc.get("type").and_then(|t| t.as_str()) {
                *dist.entry(t.to_string()).or_insert(0) += 1;
            }
        }
        dist
    };

    let data = json!({
        "metadata": {
            "export_version": "2.0",
            "export_timestamp": chrono::Utc::now().to_rfc3339(),
            "specification": "memscope-rs memory analysis",
            "total_allocations": allocations.len(),
            "total_size_bytes": total_size
        },
        "allocations": allocations,
        "statistics": {
            "total_allocations": allocations.len(),
            "total_size_bytes": total_size,
            "average_size_bytes": if allocations.is_empty() { 0 } else { total_size / allocations.len() }
        },
        "type_distribution": type_distribution
    });

    let path = output_path.as_ref().join("memory_analysis.json");
    write_json_optimized(path, &data, options)?;
    Ok(())
}

fn generate_lifetime_json<P: AsRef<Path>>(
    output_path: P,
    allocations: &[serde_json::Value],
    options: &ExportJsonOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let ownership_histories: Vec<serde_json::Value> = allocations
        .iter()
        .map(|alloc| {
            json!({
                "address": alloc.get("address"),
                "var_name": alloc.get("var_name"),
                "type_name": alloc.get("type_name"),
                "size": alloc.get("size"),
                "timestamp_alloc": alloc.get("timestamp"),
                "timestamp_dealloc": null,
                "lifetime_ms": alloc.get("lifetime_ms"),
                "events": [
                    {
                        "event_type": "Created",
                        "timestamp": alloc.get("timestamp"),
                        "context": "initial_allocation"
                    }
                ]
            })
        })
        .collect();

    let lifetime_data = json!({
        "metadata": {
            "export_version": "2.0",
            "export_timestamp": chrono::Utc::now().to_rfc3339(),
            "specification": "memscope-rs lifetime tracking",
            "total_tracked_allocations": ownership_histories.len()
        },
        "ownership_histories": ownership_histories
    });

    let lifetime_path = output_path.as_ref().join("lifetime.json");
    write_json_optimized(lifetime_path, &lifetime_data, options)?;
    Ok(())
}

fn generate_thread_analysis_json<P: AsRef<Path>>(
    output_path: P,
    thread_stats: &HashMap<u64, ThreadMemoryStats>,
    options: &ExportJsonOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let thread_analysis: Vec<serde_json::Value> = thread_stats
        .values()
        .map(|stats| {
            json!({
                "thread_id": stats.thread_id,
                "allocation_count": stats.allocation_count,
                "total_allocated": stats.total_allocated,
                "current_memory": stats.current_memory,
                "peak_memory": stats.peak_memory,
            })
        })
        .collect();

    let data = json!({
        "metadata": {
            "export_version": "2.0",
            "export_timestamp": chrono::Utc::now().to_rfc3339(),
            "specification": "thread analysis",
            "total_threads": thread_analysis.len()
        },
        "thread_analysis": thread_analysis
    });

    let path = output_path.as_ref().join("thread_analysis.json");
    write_json_optimized(path, &data, options)?;
    Ok(())
}

fn write_json_optimized<P: AsRef<Path>>(
    path: P,
    data: &serde_json::Value,
    options: &ExportJsonOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = path.as_ref();

    let estimated_size = estimate_json_size(data);
    let use_compact = options
        .use_compact_format
        .unwrap_or(estimated_size > 1_000_000);

    if options.streaming_writer && estimated_size > 500_000 {
        let file = File::create(path)?;
        let mut writer = BufWriter::with_capacity(options.buffer_size, file);

        if use_compact {
            serde_json::to_writer(&mut writer, data)?;
        } else {
            serde_json::to_writer_pretty(&mut writer, data)?;
        }

        writer.flush()?;
    } else {
        let json_string = if use_compact {
            serde_json::to_string(data)?
        } else {
            serde_json::to_string_pretty(data)?
        };
        std::fs::write(path, json_string)?;
    }

    Ok(())
}

fn estimate_json_size(data: &serde_json::Value) -> usize {
    match data {
        serde_json::Value::Object(map) => {
            map.values().map(estimate_json_size).sum::<usize>() + map.len() * 20
        }
        serde_json::Value::Array(arr) => {
            arr.iter().map(estimate_json_size).sum::<usize>() + arr.len() * 10
        }
        serde_json::Value::String(s) => s.len(),
        serde_json::Value::Number(n) => n.to_string().len(),
        _ => 10,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Export failed: {0}")]
    ExportFailed(String),
}

pub fn export_all_json<P: AsRef<Path>>(
    path: P,
    tracker: &Tracker,
    passport_tracker: &Arc<MemoryPassportTracker>,
    async_tracker: &Arc<crate::capture::backends::async_tracker::AsyncTracker>,
) -> MemScopeResult<()> {
    let path_ref = path.as_ref();

    // Use event_store as unified data source (includes both HeapOwner and Container allocations)
    let events = tracker.event_store().snapshot();
    let allocations = DashboardRenderer::rebuild_allocations_from_events(&events);
    let snapshot = MemorySnapshot::from_allocation_infos(allocations.clone());
    let options = ExportJsonOptions::default();

    std::fs::create_dir_all(path_ref)
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Starting export_snapshot_to_json");

    export_snapshot_to_json(&snapshot, path_ref, &options)
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Completed export_snapshot_to_json");

    debug!("Starting export_memory_passports_json");

    export_memory_passports_json(path_ref, passport_tracker)
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Completed export_memory_passports_json");

    debug!("Starting export_leak_detection_json");

    export_leak_detection_json(path_ref, passport_tracker)
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Completed export_leak_detection_json");

    debug!("Starting export_unsafe_ffi_json");

    export_unsafe_ffi_json(path_ref, passport_tracker)
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Completed export_unsafe_ffi_json");

    debug!("Starting export_system_resources_json");

    export_system_resources_json(path_ref)
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Completed export_system_resources_json");

    debug!("Starting export_async_analysis_json");

    export_async_analysis_json(path_ref, async_tracker)
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Completed export_async_analysis_json");

    debug!("Starting export_ownership_graph_json");

    let typed_allocations: Vec<crate::capture::types::AllocationInfo> =
        allocations.clone().into_iter().map(|a| a.into()).collect();

    debug!(
        allocations = typed_allocations.len(),
        "Converted allocations to typed format"
    );

    export_ownership_graph_json(path_ref, &typed_allocations, tracker.event_store())
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Completed export_ownership_graph_json");

    debug!("All exports completed successfully");

    Ok(())
}

/// Export async task analysis to JSON
pub fn export_async_analysis_json<P: AsRef<Path>>(
    path: P,
    async_tracker: &Arc<crate::capture::backends::async_tracker::AsyncTracker>,
) -> MemScopeResult<()> {
    let path_ref = path.as_ref();
    let stats = async_tracker.get_stats();
    let profiles = async_tracker.get_all_profiles();
    let snapshot = async_tracker.snapshot();

    let async_data = json!({
        "summary": {
            "total_tasks": stats.total_tasks,
            "active_tasks": stats.active_tasks,
            "total_allocations": stats.total_allocations,
            "total_memory_bytes": stats.total_memory,
            "active_memory_bytes": stats.active_memory,
            "peak_memory_bytes": stats.peak_memory,
        },
        "task_profiles": profiles.iter().map(|p| json!({
            "task_id": p.task_id,
            "task_name": p.task_name,
            "task_type": format!("{:?}", p.task_type),
            "created_at_ms": p.created_at_ms,
            "completed_at_ms": p.completed_at_ms,
            "total_bytes": p.total_bytes,
            "current_memory": p.current_memory,
            "peak_memory": p.peak_memory,
            "total_allocations": p.total_allocations,
            "total_deallocations": p.total_deallocations,
            "duration_ns": p.duration_ns,
            "allocation_rate": p.allocation_rate,
            "efficiency_score": p.efficiency_score,
            "average_allocation_size": p.average_allocation_size,
            "is_completed": p.is_completed(),
            "has_potential_leak": p.has_potential_leak(),
        })).collect::<Vec<_>>(),
        "allocations": snapshot.allocations.iter().map(|a| json!({
            "ptr": format!("0x{:x}", a.ptr),
            "size": a.size,
            "timestamp": a.timestamp,
            "task_id": a.task_id,
            "var_name": a.var_name,
            "type_name": a.type_name,
        })).collect::<Vec<_>>(),
    });

    let async_path = path_ref.join("async_analysis.json");
    let file = File::create(async_path)
        .map_err(|e| MemScopeError::error("export", "export_async_analysis_json", e.to_string()))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &async_data)
        .map_err(|e| MemScopeError::error("export", "export_async_analysis_json", e.to_string()))?;
    writer
        .flush()
        .map_err(|e| MemScopeError::error("export", "export_async_analysis_json", e.to_string()))?;

    Ok(())
}

/// Dashboard template type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DashboardTemplate {
    /// Unified dashboard (multi-mode in single HTML)
    #[default]
    Unified,
    /// Final dashboard (new investigation console)
    // #[default]
    Final,
}

impl std::fmt::Display for DashboardTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DashboardTemplate::Unified => write!(f, "dashboard_unified"),
            DashboardTemplate::Final => write!(f, "dashboard_final"),
        }
    }
}

/// Export HTML dashboard from tracker data
///
/// This function generates a complete HTML dashboard from the tracker data,
/// including memory analysis, variable relationships, unsafe/FFI tracking,
/// and system resources. The dashboard is rendered using Handlebars templates.
pub fn export_dashboard_html<P: AsRef<Path>>(
    path: P,
    tracker: &Tracker,
    passport_tracker: &Arc<MemoryPassportTracker>,
) -> MemScopeResult<()> {
    export_dashboard_html_with_template(
        path,
        tracker,
        passport_tracker,
        DashboardTemplate::default(),
        None,
    )
}

/// Export HTML dashboard with async tracker support
pub fn export_dashboard_html_with_async<P: AsRef<Path>>(
    path: P,
    tracker: &Tracker,
    passport_tracker: &Arc<MemoryPassportTracker>,
    async_tracker: &Arc<crate::capture::backends::async_tracker::AsyncTracker>,
) -> MemScopeResult<()> {
    export_dashboard_html_with_template(
        path,
        tracker,
        passport_tracker,
        DashboardTemplate::default(),
        Some(async_tracker),
    )
}

/// Export HTML dashboard with specific template
///
/// This function generates a complete HTML dashboard from the tracker data
/// using the specified template type.
pub fn export_dashboard_html_with_template<P: AsRef<Path>>(
    path: P,
    tracker: &Tracker,
    passport_tracker: &Arc<MemoryPassportTracker>,
    template: DashboardTemplate,
    async_tracker: Option<&Arc<crate::capture::backends::async_tracker::AsyncTracker>>,
) -> MemScopeResult<()> {
    let path_ref = path.as_ref();

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(path_ref).map_err(|e| {
        MemScopeError::error(
            "export",
            "export_dashboard_html_with_template",
            format!("Failed to create output directory: {}", e),
        )
    })?;

    // Create dashboard renderer
    let renderer = DashboardRenderer::new().map_err(|e| {
        MemScopeError::error(
            "export",
            "export_dashboard_html_with_template",
            format!("Failed to create dashboard renderer: {}", e),
        )
    })?;

    // Render HTML from tracker data using selected template
    let context = renderer
        .build_context_from_tracker_with_async(tracker, passport_tracker, async_tracker)
        .map_err(|e| {
            MemScopeError::error(
                "export",
                "export_dashboard_html_with_template",
                format!("Failed to build context: {}", e),
            )
        })?;

    let html_content = match template {
        DashboardTemplate::Final => renderer.render_final_dashboard(&context).map_err(|e| {
            MemScopeError::error(
                "export",
                "export_dashboard_html_with_template",
                format!("Failed to render final dashboard: {}", e),
            )
        })?,
        DashboardTemplate::Unified => renderer.render_unified_dashboard(&context).map_err(|e| {
            MemScopeError::error(
                "export",
                "export_dashboard_html_with_template",
                format!("Failed to render dashboard: {}", e),
            )
        })?,
    };

    // Write HTML to file
    let output_file = path_ref.join(format!("{}_dashboard.html", template));
    std::fs::write(&output_file, html_content).map_err(|e| {
        MemScopeError::error(
            "export",
            "export_dashboard_html_with_template",
            format!("Failed to write HTML file: {}", e),
        )
    })?;

    tracing::info!("✅ Dashboard HTML exported to: {:?}", output_file);

    Ok(())
}

pub fn export_memory_passports_json<P: AsRef<Path>>(
    base_path: P,
    passport_tracker: &Arc<MemoryPassportTracker>,
) -> MemScopeResult<()> {
    let base_path = base_path.as_ref();
    let passports = passport_tracker.get_all_passports();

    let passport_data: Vec<_> = passports
        .values()
        .map(|p| {
            serde_json::json!({
                "passport_id": p.passport_id,
                "allocation_ptr": format!("0x{:x}", p.allocation_ptr),
                "size_bytes": p.size_bytes,
                "created_at": p.created_at,
                "lifecycle_events": p.lifecycle_events.len(),
                "status": format!("{:?}", p.status_at_shutdown),
            })
        })
        .collect();

    let json_data = serde_json::json!({
        "metadata": {
            "export_version": "2.0",
            "specification": "memory passport tracking",
            "total_passports": passports.len()
        },
        "memory_passports": passport_data,
    });

    let file_path = base_path.join("memory_passports.json");
    let json_string = serde_json::to_string_pretty(&json_data).map_err(|e| {
        MemScopeError::error("export", "export_memory_passports_json", e.to_string())
    })?;
    std::fs::write(&file_path, json_string).map_err(|e| {
        MemScopeError::error("export", "export_memory_passports_json", e.to_string())
    })?;

    Ok(())
}

pub fn export_leak_detection_json<P: AsRef<Path>>(
    base_path: P,
    passport_tracker: &Arc<MemoryPassportTracker>,
) -> MemScopeResult<()> {
    let base_path = base_path.as_ref();
    let leak_result = passport_tracker.detect_leaks_at_shutdown();

    let leak_details: Vec<_> = leak_result
        .leak_details
        .iter()
        .map(|detail| {
            serde_json::json!({
                "passport_id": detail.passport_id,
                "memory_address": format!("0x{:x}", detail.memory_address),
                "size_bytes": detail.size_bytes,
                "lifecycle_summary": detail.lifecycle_summary,
            })
        })
        .collect();

    let json_data = serde_json::json!({
        "metadata": {
            "export_version": "2.0",
            "specification": "leak detection",
            "leaks_detected": leak_result.total_leaks
        },
        "leak_detection": {
            "total_leaks": leak_result.total_leaks,
            "leak_details": leak_details
        }
    });

    let file_path = base_path.join("leak_detection.json");
    let json_string = serde_json::to_string_pretty(&json_data)
        .map_err(|e| MemScopeError::error("export", "export_leak_detection_json", e.to_string()))?;
    std::fs::write(&file_path, json_string)
        .map_err(|e| MemScopeError::error("export", "export_leak_detection_json", e.to_string()))?;

    Ok(())
}

pub fn export_unsafe_ffi_json<P: AsRef<Path>>(
    base_path: P,
    passport_tracker: &Arc<MemoryPassportTracker>,
) -> MemScopeResult<()> {
    use crate::analysis::memory_passport_tracker::PassportStatus;

    let base_path = base_path.as_ref();
    let passports = passport_tracker.get_all_passports();

    let ffi_reports: Vec<_> = passports
        .values()
        .filter(|p| {
            matches!(
                p.status_at_shutdown,
                PassportStatus::HandoverToFfi
                    | PassportStatus::InForeignCustody
                    | PassportStatus::FreedByForeign
            )
        })
        .map(|p| {
            serde_json::json!({
                "passport_id": p.passport_id,
                "allocation_ptr": format!("0x{:x}", p.allocation_ptr),
                "size_bytes": p.size_bytes,
                "status": format!("{:?}", p.status_at_shutdown),
                "created_at": p.created_at,
                "boundary_events": p.lifecycle_events.iter().map(|e| {
                    serde_json::json!({
                        "timestamp": e.timestamp,
                        "event_type": format!("{:?}", e.event_type),
                        "context": e.context,
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect();

    let json_data = serde_json::json!({
        "metadata": {
            "export_version": "2.0",
            "specification": "unsafe FFI tracking",
            "total_ffi_reports": ffi_reports.len(),
            "total_memory_passports": passports.len()
        },
        "unsafe_reports": ffi_reports,
        "memory_passports": passports.len()
    });

    let file_path = base_path.join("unsafe_ffi.json");
    let json_string = serde_json::to_string_pretty(&json_data)
        .map_err(|e| MemScopeError::error("export", "export_unsafe_ffi_json", e.to_string()))?;
    std::fs::write(&file_path, json_string)
        .map_err(|e| MemScopeError::error("export", "export_unsafe_ffi_json", e.to_string()))?;

    Ok(())
}

/// Export system resource monitoring data to JSON
///
/// This function exports comprehensive system resource statistics including:
/// - Memory statistics (virtual, physical, process, system)
/// - CPU information (cores, cache, system info)
/// - Memory pressure indicators
pub fn export_system_resources_json<P: AsRef<Path>>(base_path: P) -> MemScopeResult<()> {
    let base_path = base_path.as_ref();

    // Collect memory statistics
    let mut memory_info = PlatformMemoryInfo::new();
    let _ = memory_info.initialize();

    let memory_stats = match memory_info.collect_stats() {
        Ok(stats) => stats,
        Err(e) => {
            warn!(error = %e, "Failed to collect memory stats");
            return Err(MemScopeError::error(
                "export",
                "export_system_resources_json",
                e.to_string(),
            ));
        }
    };

    // Collect system information
    let system_info = match memory_info.get_system_info() {
        Ok(info) => info,
        Err(e) => {
            warn!(error = %e, "Failed to collect system info");
            return Err(MemScopeError::error(
                "export",
                "export_system_resources_json",
                e.to_string(),
            ));
        }
    };

    // Build JSON structure
    let json_data = serde_json::json!({
        "metadata": {
            "export_version": "2.0",
            "specification": "system resource monitoring",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "system_info": {
            "os_name": system_info.os_name,
            "os_version": system_info.os_version,
            "architecture": system_info.architecture,
            "cpu_cores": system_info.cpu_cores,
            "page_size": system_info.page_size,
            "large_page_size": system_info.large_page_size,
            "cpu_cache": {
                "l1_cache_size": system_info.cpu_cache.l1_cache_size,
                "l2_cache_size": system_info.cpu_cache.l2_cache_size,
                "l3_cache_size": system_info.cpu_cache.l3_cache_size,
                "cache_line_size": system_info.cpu_cache.cache_line_size
            },
            "mmu_info": {
                "virtual_address_bits": system_info.mmu_info.virtual_address_bits,
                "physical_address_bits": system_info.mmu_info.physical_address_bits,
                "aslr_enabled": system_info.mmu_info.aslr_enabled,
                "nx_bit_supported": system_info.mmu_info.nx_bit_supported
            }
        },
        "memory_stats": {
            "virtual_memory": {
                "total_virtual": memory_stats.virtual_memory.total_virtual,
                "available_virtual": memory_stats.virtual_memory.available_virtual,
                "used_virtual": memory_stats.virtual_memory.used_virtual,
                "reserved": memory_stats.virtual_memory.reserved,
                "committed": memory_stats.virtual_memory.committed
            },
            "physical_memory": {
                "total_physical": memory_stats.physical_memory.total_physical,
                "available_physical": memory_stats.physical_memory.available_physical,
                "used_physical": memory_stats.physical_memory.used_physical,
                "cached": memory_stats.physical_memory.cached,
                "buffers": memory_stats.physical_memory.buffers,
                "swap": {
                    "total_swap": memory_stats.physical_memory.swap.total_swap,
                    "used_swap": memory_stats.physical_memory.swap.used_swap,
                    "available_swap": memory_stats.physical_memory.swap.available_swap,
                    "swap_in_rate": memory_stats.physical_memory.swap.swap_in_rate,
                    "swap_out_rate": memory_stats.physical_memory.swap.swap_out_rate
                }
            },
            "process_memory": {
                "virtual_size": memory_stats.process_memory.virtual_size,
                "resident_size": memory_stats.process_memory.resident_size,
                "shared_size": memory_stats.process_memory.shared_size,
                "private_size": memory_stats.process_memory.private_size,
                "heap_size": memory_stats.process_memory.heap_size,
                "stack_size": memory_stats.process_memory.stack_size,
                "mapped_files": memory_stats.process_memory.mapped_files,
                "peak_usage": memory_stats.process_memory.peak_usage
            },
            "system_memory": {
                "allocation_count": memory_stats.system_memory.allocation_count,
                "deallocation_count": memory_stats.system_memory.deallocation_count,
                "active_allocations": memory_stats.system_memory.active_allocations,
                "total_allocated": memory_stats.system_memory.total_allocated,
                "total_deallocated": memory_stats.system_memory.total_deallocated,
                "fragmentation_level": memory_stats.system_memory.fragmentation_level,
                "large_pages": {
                    "supported": memory_stats.system_memory.large_pages.supported,
                    "total_large_pages": memory_stats.system_memory.large_pages.total_large_pages,
                    "used_large_pages": memory_stats.system_memory.large_pages.used_large_pages,
                    "page_size": memory_stats.system_memory.large_pages.page_size
                }
            },
            "pressure_indicators": {
                "pressure_level": format!("{:?}", memory_stats.pressure_indicators.pressure_level),
                "low_memory": memory_stats.pressure_indicators.low_memory,
                "swapping_active": memory_stats.pressure_indicators.swapping_active,
                "allocation_failure_rate": memory_stats.pressure_indicators.allocation_failure_rate,
                "gc_pressure": memory_stats.pressure_indicators.gc_pressure
            }
        }
    });

    let file_path = base_path.join("system_resources.json");
    let json_string = serde_json::to_string_pretty(&json_data).map_err(|e| {
        MemScopeError::error("export", "export_system_resources_json", e.to_string())
    })?;
    std::fs::write(&file_path, json_string).map_err(|e| {
        MemScopeError::error("export", "export_system_resources_json", e.to_string())
    })?;

    Ok(())
}

/// Export ownership graph analysis to JSON
///
/// This function exports the ownership graph including:
/// - Node information (objects with their types and sizes)
/// - Edge information (clone relationships)
/// - Detected cycles (Rc/Arc retain cycles)
/// - Diagnostics (clone storms, cycle warnings)
pub fn export_ownership_graph_json<P: AsRef<Path>>(
    base_path: P,
    allocations: &[crate::capture::types::AllocationInfo],
    event_store: &crate::event_store::EventStore,
) -> MemScopeResult<()> {
    let base_path = base_path.as_ref();

    // Build ownership graph from allocations
    let graph = build_ownership_graph_from_allocations(allocations, event_store);

    // Get diagnostics
    let diagnostics = graph.diagnostics(50);

    // Convert nodes to JSON
    let nodes_json: Vec<_> = graph
        .nodes
        .iter()
        .map(|node| {
            json!({
                "id": format!("0x{:x}", node.id.0),
                "type_name": node.type_name,
                "size": node.size,
            })
        })
        .collect();

    // Convert edges to JSON
    let edges_json: Vec<_> = graph
        .edges
        .iter()
        .map(|edge| {
            json!({
                "from": format!("0x{:x}", edge.from.0),
                "to": format!("0x{:x}", edge.to.0),
                "kind": match edge.op {
                    EdgeKind::Owns => "Owns",
                    EdgeKind::Contains => "Contains",
                    EdgeKind::Borrows => "Borrows",
                    EdgeKind::RcClone => "RcClone",
                    EdgeKind::ArcClone => "ArcClone",
                },
            })
        })
        .collect();

    // Convert cycles to JSON
    let cycles_json: Vec<_> = graph
        .cycles
        .iter()
        .map(|cycle| {
            let nodes: Vec<_> = cycle.iter().map(|id| format!("0x{:x}", id.0)).collect();
            json!({
                "nodes": nodes,
            })
        })
        .collect();

    // Convert issues to JSON
    let issues_json: Vec<_> = diagnostics
        .issues
        .iter()
        .map(|issue| match issue {
            crate::analysis::ownership_graph::DiagnosticIssue::RcCycle { nodes, cycle_type } => {
                json!({
                    "type": "RcCycle",
                    "cycle_type": format!("{:?}", cycle_type),
                    "nodes": nodes.iter().map(|id| format!("0x{:x}", id.0)).collect::<Vec<_>>(),
                    "severity": "error",
                })
            }
            crate::analysis::ownership_graph::DiagnosticIssue::ArcCloneStorm {
                clone_count,
                threshold,
            } => {
                json!({
                    "type": "ArcCloneStorm",
                    "clone_count": clone_count,
                    "threshold": threshold,
                    "severity": "warning",
                })
            }
        })
        .collect();

    // Build root cause info
    let root_cause_json = graph.find_root_cause().map(|rc| {
        json!({
            "cause": match rc.root_cause {
                crate::analysis::ownership_graph::RootCause::ArcCloneStorm => "ArcCloneStorm",
                crate::analysis::ownership_graph::RootCause::RcCycle => "RcCycle",
            },
            "description": rc.description,
            "impact": rc.impact,
        })
    });

    let json_data = json!({
        "metadata": {
            "export_version": "2.0",
            "specification": "ownership graph analysis",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "summary": {
            "total_nodes": graph.nodes.len(),
            "total_edges": graph.edges.len(),
            "total_cycles": graph.cycles.len(),
            "rc_clone_count": diagnostics.rc_clone_count,
            "arc_clone_count": diagnostics.arc_clone_count,
            "has_issues": diagnostics.has_issues(),
        },
        "nodes": nodes_json,
        "edges": edges_json,
        "cycles": cycles_json,
        "diagnostics": {
            "issues": issues_json,
            "root_cause": root_cause_json,
        },
    });

    let file_path = base_path.join("ownership_graph.json");
    let json_string = serde_json::to_string_pretty(&json_data).map_err(|e| {
        MemScopeError::error("export", "export_ownership_graph_json", e.to_string())
    })?;
    std::fs::write(&file_path, json_string).map_err(|e| {
        MemScopeError::error("export", "export_ownership_graph_json", e.to_string())
    })?;

    Ok(())
}

/// Build ownership graph from allocation data
fn build_ownership_graph_from_allocations(
    allocations: &[crate::capture::types::AllocationInfo],
    event_store: &crate::event_store::EventStore,
) -> OwnershipGraph {
    debug!(
        allocations = allocations.len(),
        "Starting build_ownership_graph"
    );
    use crate::analysis::relation_inference::{detect_containers, Relation, RelationGraphBuilder};
    use crate::event_store::MemoryEventType;

    // Convert allocations to passport format for graph building
    // IMPORTANT: For allocations with ptr=0, assign virtual unique IDs to avoid collisions
    debug!("Converting allocations to passports");
    let passports: Vec<(
        ObjectId,
        String,
        usize,
        Vec<crate::analysis::ownership_graph::OwnershipEvent>,
    )> = allocations
        .iter()
        .enumerate()
        .map(|(idx, alloc)| {
            let unique_ptr = if alloc.ptr == 0 {
                crate::analysis::VIRTUAL_PTR_BASE + idx
            } else {
                alloc.ptr
            };
            let id = ObjectId::from_ptr(unique_ptr);
            let type_name = alloc
                .type_name
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            let size = alloc.size;

            // Generate ownership events from allocation info
            let events = vec![crate::analysis::ownership_graph::OwnershipEvent::new(
                alloc.timestamp_alloc,
                OwnershipOp::Create,
                id,
                None,
            )];

            // Detect smart pointer clones from type name
            if type_name.contains("Arc<") || type_name.contains("Rc<") {
                // Check for clone patterns
                // Note: In real tracking, clone events would be recorded at runtime
                // Here we infer from type and allocation patterns
            }

            (id, type_name, size, events)
        })
        .collect();
    debug!(passports = passports.len(), "Created passports");

    debug!("Building initial ownership graph");
    let mut graph = OwnershipGraph::build(&passports);
    debug!(
        nodes = graph.nodes.len(),
        edges = graph.edges.len(),
        "Initial graph built"
    );

    // Build separate lists for HeapOwner and Container allocations
    // with their original indices preserved for correct edge mapping

    // Step 1: Collect HeapOwner allocations
    let _heap_owner_original_indices: Vec<usize> = allocations
        .iter()
        .enumerate()
        .filter(|(_, a)| a.timestamp_dealloc.is_none())
        .map(|(i, _)| i)
        .collect();

    let heap_owner_allocations: Vec<ActiveAllocation> = allocations
        .iter()
        .enumerate()
        .filter(|(_, a)| a.timestamp_dealloc.is_none())
        .filter_map(|(_idx, a)| {
            // Skip Container types (ptr is 0 or virtual pointer)
            if a.ptr == 0 || is_virtual_pointer(a.ptr) {
                return None;
            }

            Some(ActiveAllocation {
                ptr: Some(a.ptr),
                kind: crate::core::types::TrackKind::HeapOwner {
                    ptr: a.ptr,
                    size: a.size,
                },
                size: a.size,
                allocated_at: a.timestamp_alloc,
                var_name: a.var_name.clone(),
                type_name: a.type_name.clone(),
                thread_id: a.thread_id_u64,
                call_stack_hash: None,
            })
        })
        .collect();

    // Step 2: Collect Container events from event_store
    // IMPORTANT: Only include containers that match HeapOwner thread_id
    let valid_thread_ids: std::collections::HashSet<u64> =
        heap_owner_allocations.iter().map(|a| a.thread_id).collect();

    let container_events: Vec<_> = event_store
        .snapshot()
        .into_iter()
        .filter(|e| e.event_type == MemoryEventType::Metadata)
        .filter(|e| valid_thread_ids.contains(&e.thread_id))
        .filter_map(|e| {
            let type_name = e.type_name.clone().unwrap_or_default();
            let var_name = e.var_name.clone().unwrap_or_default();
            let is_container = type_name.contains("HashMap")
                || type_name.contains("BTreeMap")
                || type_name.contains("VecDeque")
                || type_name.contains("RefCell")
                || type_name.contains("RwLock");
            if is_container {
                return Some((e, type_name, var_name));
            }
            None
        })
        .collect();

    // Assign virtual pointers to Container allocations to avoid ptr=0 collisions
    let container_allocations: Vec<ActiveAllocation> = container_events
        .iter()
        .enumerate()
        .map(|(idx, (e, type_name, var_name))| {
            let virtual_ptr = 0x300000000u64 as usize + idx;
            ActiveAllocation {
                ptr: Some(virtual_ptr),
                kind: crate::core::types::TrackKind::Container,
                size: e.size.max(1),
                allocated_at: e.timestamp,
                var_name: Some(var_name.clone()),
                type_name: Some(type_name.clone()),
                thread_id: e.thread_id,
                call_stack_hash: None,
            }
        })
        .collect();

    // Step 3: Combine for container detection: first HeapOwner, then Container
    let mut all_for_relation: Vec<ActiveAllocation> = Vec::new();
    all_for_relation.extend(heap_owner_allocations.clone());
    all_for_relation.extend(container_allocations.clone());

    // Step 4: Run container detection with appropriate config
    debug!("Running container detection");
    let container_config = crate::analysis::relation_inference::ContainerConfig {
        time_window_ns: 10_000_000, // 10ms (more permissive for test scenarios)
        size_ratio: 10000,          // Allow very large ratio for containers with small size
        lookahead: 10,              // Look at more candidates
    };

    let container_edges = detect_containers(&all_for_relation, Some(container_config));
    debug!(
        container_edges = container_edges.len(),
        "Container detection completed"
    );

    // Step 5: Run RelationGraphBuilder for HeapOwner edges (Owns, Shares, Clone)
    debug!("Running RelationGraphBuilder");
    let relation_graph = RelationGraphBuilder::build(&heap_owner_allocations, None);
    debug!(
        edges = relation_graph.edges.len(),
        "RelationGraphBuilder completed"
    );

    // Step 6: Add Container nodes to graph
    let heap_owner_count = graph.nodes.len();
    for (e, type_name, _var_name) in container_events.iter() {
        let node_id = ObjectId(e.timestamp);
        graph.nodes.push(crate::analysis::ownership_graph::Node {
            id: node_id,
            type_name: type_name.clone(),
            size: e.size,
        });
    }

    // Step 7: Add HeapOwner edges (Owns, Shares, Clone)
    for edge in &relation_graph.edges {
        let from_id = graph.nodes[edge.from].id;
        let to_id = graph.nodes[edge.to].id;

        let edge_kind = match edge.relation {
            Relation::Owns => EdgeKind::Owns,
            Relation::Contains => EdgeKind::Contains,
            Relation::Slice => EdgeKind::Borrows,
            Relation::Clone => EdgeKind::RcClone,
            Relation::Shares => EdgeKind::ArcClone,
            Relation::Evolution => EdgeKind::Contains,
        };

        graph.edges.push(crate::analysis::ownership_graph::Edge {
            from: from_id,
            to: to_id,
            op: edge_kind,
        });
    }

    // Step 8: Add Container → HeapOwner edges (Contains)
    for edge in &container_edges {
        // edge.from is Container index in all_for_relation
        // edge.to is HeapOwner index in all_for_relation
        let from_all_idx = edge.from;
        let to_all_idx = edge.to;

        // Validate indices
        if from_all_idx < heap_owner_count || to_all_idx >= heap_owner_count {
            continue; // Invalid edge indices
        }

        // Convert from container index to graph node index
        let container_graph_idx = from_all_idx - heap_owner_count;
        if container_graph_idx >= container_events.len() {
            continue; // Invalid container index
        }

        let from_id = graph.nodes[heap_owner_count + container_graph_idx].id;
        let to_id = graph.nodes[to_all_idx].id;

        graph.edges.push(crate::analysis::ownership_graph::Edge {
            from: from_id,
            to: to_id,
            op: EdgeKind::Contains,
        });
    }

    debug!(
        nodes = graph.nodes.len(),
        edges = graph.edges.len(),
        "Final ownership graph built"
    );
    graph
}
