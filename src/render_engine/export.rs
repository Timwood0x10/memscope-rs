//! Export functionality for the render engine
//!
//! This module provides export functionality for memory tracking data,
//! including JSON export, lifetime analysis, and variable relationships.

use crate::snapshot::{ActiveAllocation, MemorySnapshot, ThreadMemoryStats};
use rayon::prelude::*;
use serde_json::json;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};


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
    std::fs::create_dir_all(output_path)?;

    let allocations: Vec<&ActiveAllocation> = snapshot.active_allocations.values().collect();
    let processed = process_allocations(&allocations, options)?;

    generate_memory_analysis_json(output_path, &processed, options)?;
    generate_lifetime_json(output_path, &processed, options)?;
    generate_thread_analysis_json(output_path, &snapshot.thread_stats, options)?;
    generate_variable_relationships_json(output_path, &processed, options)?;

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
    allocations
        .iter()
        .map(|alloc| {
            let type_info = get_or_compute_type_info(
                alloc.type_name.as_deref().unwrap_or("unknown"),
                alloc.size,
            );

            let mut entry = json!({
                "address": format!("0x{:x}", alloc.ptr),
                "size": alloc.size,
                "type": type_info,
                "timestamp": alloc.allocated_at,
                "thread_id": alloc.thread_id,
                "lifetime_ms": 0,
                "borrow_info": {
                    "immutable_borrows": 0,
                    "mutable_borrows": 0,
                    "max_concurrent_borrows": 0,
                },
                "clone_info": {
                    "clone_count": 0,
                    "is_clone": false,
                    "original_ptr": null,
                },
                "ownership_history_available": false,
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
    if type_name.contains("Vec") || type_name.contains("vec::Vec") {
        "dynamic_array".to_string()
    } else if type_name.contains("String") || type_name.contains("str") {
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

fn generate_variable_relationships_json<P: AsRef<Path>>(
    output_path: P,
    allocations: &[serde_json::Value],
    options: &ExportJsonOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut relationships = Vec::new();

    fn parse_address(addr_str: &str) -> Option<usize> {
        if addr_str.starts_with("0x") || addr_str.starts_with("0X") {
            usize::from_str_radix(&addr_str[2..], 16).ok()
        } else {
            usize::from_str_radix(addr_str, 16)
                .ok()
                .or_else(|| addr_str.parse::<usize>().ok())
        }
    }

    for allocation in allocations {
        if let Some(clone_info) = allocation.get("clone_info") {
            if !clone_info.is_null() {
                if let Some(is_clone) = clone_info.get("is_clone").and_then(|c| c.as_bool()) {
                    if is_clone {
                        if let (Some(address), Some(original_ptr)) = (
                            allocation.get("address").and_then(|a| a.as_str()),
                            clone_info.get("original_ptr").and_then(|p| p.as_u64()),
                        ) {
                            if let Some(ptr) = parse_address(address) {
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
    }

    let mut type_map: HashMap<String, Vec<&serde_json::Value>> = HashMap::new();
    for allocation in allocations {
        if let Some(type_name) = allocation.get("type_name").and_then(|t| t.as_str()) {
            type_map.entry(type_name.to_string()).or_default().push(allocation);
        }
    }

    for (type_name, type_allocations) in type_map {
        if type_allocations.len() > 1 {
            let max_pairs = std::cmp::min(type_allocations.len() * 2, 100);
            let mut pair_count = 0;

            for i in 0..type_allocations.len() {
                if pair_count >= max_pairs {
                    break;
                }
                for j in i + 1..type_allocations.len() {
                    if pair_count >= max_pairs {
                        break;
                    }
                    if let (Some(addr1), Some(addr2)) = (
                        type_allocations[i].get("address").and_then(|a| a.as_str()),
                        type_allocations[j].get("address").and_then(|a| a.as_str()),
                    ) {
                        if let (Some(ptr1), Some(ptr2)) = (parse_address(addr1), parse_address(addr2)) {
                            relationships.push(json!({
                                "relationship_type": "type_similarity",
                                "source_ptr": ptr1,
                                "target_ptr": ptr2,
                                "relationship_strength": 0.5,
                                "details": {
                                    "type_name": type_name
                                }
                            }));
                            pair_count += 1;
                        }
                    }
                }
            }
        }
    }

    let mut size_map: HashMap<usize, Vec<&serde_json::Value>> = HashMap::new();
    for allocation in allocations {
        if let Some(size) = allocation.get("size").and_then(|s| s.as_u64()) {
            let size_range = (size as usize / 1024) * 1024;
            size_map.entry(size_range).or_default().push(allocation);
        }
    }

    for (size_range, size_allocations) in size_map {
        if size_allocations.len() > 1 {
            let max_pairs = std::cmp::min(size_allocations.len() * 2, 100);
            let mut pair_count = 0;

            for i in 0..size_allocations.len() {
                if pair_count >= max_pairs {
                    break;
                }
                for j in i + 1..size_allocations.len() {
                    if pair_count >= max_pairs {
                        break;
                    }
                    if let (Some(addr1), Some(addr2)) = (
                        size_allocations[i].get("address").and_then(|a| a.as_str()),
                        size_allocations[j].get("address").and_then(|a| a.as_str()),
                    ) {
                        if let (Some(ptr1), Some(ptr2)) = (parse_address(addr1), parse_address(addr2)) {
                            relationships.push(json!({
                                "relationship_type": "size_similarity",
                                "source_ptr": ptr1,
                                "target_ptr": ptr2,
                                "relationship_strength": 0.3,
                                "details": {
                                    "size_range": format!("{}-{} bytes", size_range, size_range + 1023)
                                }
                            }));
                            pair_count += 1;
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

        let result = if use_compact {
            serde_json::to_writer(&mut writer, data)
        } else {
            serde_json::to_writer_pretty(&mut writer, data)
        };

        result?;
        writer.flush()?;
    } else {
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

fn compute_enhanced_type_info(type_name: &str, size: usize) -> String {
    if type_name.contains("Vec") || type_name.contains("vec::Vec") {
        "dynamic_array".to_string()
    } else if type_name.contains("String") || type_name.contains("str") {
        "string".to_string()
    } else if type_name.contains("Box") {
        "boxed".to_string()
    } else if type_name.contains("Rc") || type_name.contains("Arc") {
        "reference_counted".to_string()
    } else if type_name.contains("Cell") || type_name.contains("RefCell") {
        "cell".to_string()
    } else if type_name.contains("[") {
        "array".to_string()
    } else if type_name.contains("u8") && size > 0 {
        "byte".to_string()
    } else {
        "heap".to_string()
    }
}