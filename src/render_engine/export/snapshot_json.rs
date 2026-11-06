//! JSON projections of memory snapshots and their export options.

use crate::snapshot::{ActiveAllocation, MemorySnapshot, ThreadMemoryStats};
use rayon::prelude::*;
use serde_json::json;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
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
    export_snapshot_with_containers(snapshot, &[], output_path, options)
}

/// Export a snapshot plus the container/stack metadata observed in the same capture.
///
/// Containers carry no heap bytes and are therefore not part of the counters; they are
/// written to their own array so their metadata survives without inflating the live
/// numbers (plan A04 follow-up).
pub fn export_snapshot_with_containers(
    snapshot: &MemorySnapshot,
    containers: &[crate::capture::types::AllocationInfo],
    output_path: &Path,
    options: &ExportJsonOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    // Every live identity, not just the newest binding per address: an overlapping
    // generation (a missed release) must appear in the file it is counted in.
    let allocations: Vec<&ActiveAllocation> = snapshot.allocations().collect();
    let mut processed = process_allocations(&allocations, snapshot.timestamp, options)?;
    // `process_allocations` preserves order, so the identity of each row can be taken
    // from the snapshot's binding list when it carries one.
    if let Some(bindings) = snapshot
        .active_bindings
        .as_ref()
        .filter(|bindings| bindings.len() == processed.len())
    {
        for (entry, binding) in processed.iter_mut().zip(bindings.iter()) {
            entry["generation_id"] = serde_json::json!(binding.generation_id);
            entry["binding_serial"] = serde_json::json!(binding.binding_serial);
        }
    }
    // Use output_path as the base directory for generated files
    let output_dir = if output_path.extension().is_some() {
        // If output_path has an extension, treat it as a file and use its parent
        output_path.parent().unwrap_or(Path::new("."))
    } else {
        output_path
    };
    generate_memory_analysis_json(
        output_dir,
        &processed,
        containers,
        &snapshot.stats,
        snapshot.active_bindings.is_some(),
        options,
        snapshot.timestamp,
    )?;
    generate_lifetime_json(output_dir, &processed, options, snapshot.timestamp)?;
    generate_thread_analysis_json(
        output_dir,
        &snapshot.thread_stats,
        options,
        snapshot.timestamp,
    )?;
    Ok(())
}

fn process_allocations(
    allocations: &[&ActiveAllocation],
    captured_at: u64,
    options: &ExportJsonOptions,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    if options.parallel_processing && allocations.len() > options.batch_size {
        let chunk_size = (allocations.len() / num_cpus::get()).max(1);
        Ok(allocations
            .par_chunks(chunk_size)
            .flat_map(|batch| process_allocation_batch(batch, captured_at))
            .collect())
    } else {
        Ok(process_allocation_batch(allocations, captured_at))
    }
}

fn process_allocation_batch(
    allocations: &[&ActiveAllocation],
    captured_at: u64,
) -> Vec<serde_json::Value> {
    allocations
        .iter()
        .map(|alloc| {
            let type_info = get_or_compute_type_info(
                alloc.type_name.as_deref().unwrap_or("unknown"),
                alloc.size,
            );
            // `lifetime_ms` describes a *complete* lifetime and is only filled when the
            // release time is known. A live object reports `observed_age_ms` instead:
            // "how long it had been alive when the snapshot was taken", never an invented
            // total lifetime (plan §4.5).
            let observed_age_ms = if alloc.allocated_at > 0 {
                (captured_at.saturating_sub(alloc.allocated_at)) / 1_000_000
            } else {
                0
            };
            // This projection only contains allocations that are still live, so a
            // complete lifetime is never known here: `lifetime_ms` stays null and the
            // consumer is pointed at `observed_age_ms` instead of mistaking an open-ended
            // age for the object's total lifetime (plan §4.5).
            let lifetime_ms: Option<u64> = None;
            let address = match alloc.ptr {
                Some(ptr) => format!("0x{:x}", ptr),
                None => "N/A".to_string(),
            };
            let mut entry = json!({
                "address": address, "size": alloc.size, "type": type_info,
                "timestamp": alloc.allocated_at, "thread_id": alloc.thread_id,
                "lifetime_ms": lifetime_ms, "observed_age_ms": observed_age_ms,
                "lifetime_complete": lifetime_ms.is_some(),
            });
            if let Some(ref var_name) = alloc.var_name {
                entry["var_name"] = serde_json::json!(var_name);
            }
            if let Some(ref type_name) = alloc.type_name {
                entry["type_name"] = serde_json::json!(type_name);
            }
            if let Some(ref module_path) = alloc.module_path {
                // Filter out library paths, only show user project paths
                if !is_library_module_path(module_path) {
                    entry["module_path"] = serde_json::json!(module_path);
                }
            }
            entry
        })
        .collect()
}

/// Check if a module path belongs to a library (std, core, alloc, or external crates)
fn is_library_module_path(module_path: &str) -> bool {
    // Filter out standard library paths
    if module_path.starts_with("std::")
        || module_path.starts_with("core::")
        || module_path.starts_with("alloc::")
    {
        return true;
    }
    // Filter out memscope_rs library paths
    if module_path.starts_with("memscope_rs::") {
        return true;
    }
    // Filter out other common library prefixes
    let library_prefixes = [
        "tokio::",
        "serde::",
        "async_trait::",
        "futures::",
        "log::",
        "tracing::",
        "chrono::",
        "indexmap::",
        "rustc_hash::",
        "parking_lot::",
        "crossbeam::",
        "rayon::",
        "dashmap::",
        "ahash::",
        "hashbrown::",
    ];
    for prefix in library_prefixes.iter() {
        if module_path.starts_with(prefix) {
            return true;
        }
    }
    false
}

pub(super) fn get_or_compute_type_info(type_name: &str, size: usize) -> String {
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
    containers: &[crate::capture::types::AllocationInfo],
    stats: &crate::snapshot::types::MemoryStats,
    has_transition_history: bool,
    options: &ExportJsonOptions,
    captured_at: u64,
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
    // Two different questions are answered here and must not be confused:
    //   * `live_*`        – what is still allocated at this cut (sum over the rows);
    //   * `cumulative_*`  – what the capture observed over the whole run, including
    //                       blocks that were released before the cut.
    // `total_size_bytes` / `total_allocations` stay as the live values for backwards
    // compatibility; `counters_scope` says which is which.
    // Without a transition history the snapshot only knows what is live, so the
    // cumulative counters must stay null instead of echoing the live values.
    let (cumulative_allocated, cumulative_deallocated, peak) = if has_transition_history {
        (
            serde_json::json!(stats.total_allocated),
            serde_json::json!(stats.total_deallocated),
            serde_json::json!(stats.peak_memory),
        )
    } else {
        (
            serde_json::Value::Null,
            serde_json::Value::Null,
            serde_json::Value::Null,
        )
    };
    let counters_scope = if has_transition_history {
        "live counters describe this cut, cumulative counters describe the whole capture"
    } else {
        "live only: this snapshot has no transition history, cumulative counters are unknown"
    };
    let data = json!({
        "metadata": {
            "export_version": "2.0", "export_timestamp": snapshot_timestamp(captured_at),
            "snapshot_timestamp_ns": captured_at,
            "specification": "memscope-rs memory analysis", "counters_scope": counters_scope,
            "total_allocations": allocations.len(), "total_size_bytes": total_size,
            "live_size_bytes": total_size, "degraded_releases": stats.degraded_releases,
            "cumulative_allocated_bytes": cumulative_allocated, "cumulative_deallocated_bytes": cumulative_deallocated, "peak_bytes": peak
        },
        "allocations": allocations,
        // Stack-side container metadata: listed for context, never counted as heap bytes.
        "containers": containers.iter().map(|container| json!({
            "address": if container.ptr == 0 { "N/A".to_string() } else { format!("0x{:x}", container.ptr) },
            "size": container.size,
            "type_name": container.type_name.clone().unwrap_or_else(|| "unknown".to_string()),
            "var_name": container.var_name.clone(), "thread_id": container.thread_id_u64, "counted_in_statistics": false,
        })).collect::<Vec<_>>(),
        "statistics": {
            "total_allocations": allocations.len(), "total_size_bytes": total_size,
            "average_size_bytes": if allocations.is_empty() { 0 } else { total_size / allocations.len() },
            "live_size_bytes": total_size, "allocation_count": stats.total_allocations,
            "deallocation_count": stats.total_deallocations, "degraded_releases": stats.degraded_releases,
            "cumulative_allocated_bytes": cumulative_allocated, "cumulative_deallocated_bytes": cumulative_deallocated,
            "peak_bytes": peak, "active_bindings": stats.active_allocations
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
    captured_at: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let ownership_histories: Vec<serde_json::Value> = allocations.iter().map(|alloc| {
        json!({
            "address": alloc.get("address"), "var_name": alloc.get("var_name"),
            "type_name": alloc.get("type_name"), "size": alloc.get("size"),
            "timestamp_alloc": alloc.get("timestamp"), "timestamp_dealloc": null,
            // The projection only sees live allocations, so a complete lifetime is
            // unknown: report the observed age and say so explicitly (plan §4.5).
            "lifetime_ms": null, "observed_age_ms": alloc.get("observed_age_ms"), "lifetime_complete": false,
            "events": [{ "event_type": "Created", "timestamp": alloc.get("timestamp"), "context": "initial_allocation" }]
        })
    }).collect();
    let lifetime_data = json!({
        "metadata": {
            "export_version": "2.0", "export_timestamp": snapshot_timestamp(captured_at),
            "snapshot_timestamp_ns": captured_at,
            "specification": "memscope-rs lifetime tracking", "total_tracked_allocations": ownership_histories.len()
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
    captured_at: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut thread_analysis: Vec<serde_json::Value> = thread_stats.values().map(|stats| {
        json!({
            "thread_id": stats.thread_id, "allocation_count": stats.allocation_count,
            "total_allocated": stats.total_allocated, "current_memory": stats.current_memory, "peak_memory": stats.peak_memory,
        })
    }).collect();
    thread_analysis.sort_by_key(|row| row["thread_id"].as_u64());
    let data = json!({
        "metadata": {
            "export_version": "2.0", "export_timestamp": snapshot_timestamp(captured_at),
            "snapshot_timestamp_ns": captured_at,
            "specification": "thread analysis", "total_threads": thread_analysis.len()
        },
        "thread_analysis": thread_analysis
    });
    let path = output_path.as_ref().join("thread_analysis.json");
    write_json_optimized(path, &data, options)?;
    Ok(())
}

/// Keep the legacy timestamp field stable when one snapshot is exported again.
fn snapshot_timestamp(captured_at: u64) -> String {
    chrono::DateTime::from_timestamp(
        (captured_at / 1_000_000_000) as i64,
        (captured_at % 1_000_000_000) as u32,
    )
    .expect("a u64 nanosecond timestamp is within chrono's supported range")
    .to_rfc3339()
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

pub(super) fn estimate_json_size(data: &serde_json::Value) -> usize {
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
