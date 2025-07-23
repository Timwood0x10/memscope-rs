//! Optimized JSON export implementation with performance improvements
//! 
//! This module provides highly optimized JSON export functionality that addresses
//! the main performance bottlenecks identified in the current implementation.

use crate::core::tracker::MemoryTracker;
use crate::core::types::{AllocationInfo, TrackingResult};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use rayon::prelude::*;
use std::sync::LazyLock;

/// Json file types 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonFileType {
    /// memory_analysis.json
    MemoryAnalysis,
    /// lifetime.json
    Lifetime,
    /// unsafe_ffi.json
    UnsafeFfi,
    /// performance.json
    Performance,
    /// complex_types.json
    ComplexTypes,
    // AsyncAnalysis,    // 异步分析
    // ThreadSafety,     // 线程安全分析
    // MemoryLeaks,      // 内存泄漏分析
    // TypeInference,    // 类型推断分析
}

impl JsonFileType {
    /// get standard four files
    pub fn standard_four() -> Vec<JsonFileType> {
        vec![
            JsonFileType::MemoryAnalysis,
            JsonFileType::Lifetime,
            JsonFileType::UnsafeFfi,
            JsonFileType::Performance,
        ]
    }
    
    /// get standard five files
    pub fn standard_five() -> Vec<JsonFileType> {
        vec![
            JsonFileType::MemoryAnalysis,
            JsonFileType::Lifetime,
            JsonFileType::UnsafeFfi,
            JsonFileType::Performance,
            JsonFileType::ComplexTypes,
        ]
    }
    
    /// get file suffix
    pub fn file_suffix(&self) -> &'static str {
        match self {
            JsonFileType::MemoryAnalysis => "memory_analysis",
            JsonFileType::Lifetime => "lifetime",
            JsonFileType::UnsafeFfi => "unsafe_ffi",
            JsonFileType::Performance => "performance",
            JsonFileType::ComplexTypes => "complex_types",
        }
    }
}

/// Optimized export options with intelligent defaults
#[derive(Debug, Clone)]
pub struct OptimizedExportOptions {
    /// Use parallel processing for large datasets (default: auto-detect)
    pub use_parallel_processing: Option<bool>,
    /// Buffer size for file I/O (default: 256KB for better performance)
    pub buffer_size: usize,
    /// Use compact JSON format for large files (default: auto-detect)
    pub use_compact_format: Option<bool>,
    /// Enable type inference caching (default: true)
    pub enable_type_cache: bool,
    /// Batch size for processing allocations (default: 1000)
    pub batch_size: usize,
    /// Enable async I/O for large files (default: auto-detect)
    pub use_async_io: Option<bool>,
}

impl Default for OptimizedExportOptions {
    fn default() -> Self {
        Self {
            use_parallel_processing: None, // Auto-detect based on data size
            buffer_size: 256 * 1024,      // 256KB buffer
            use_compact_format: None,      // Auto-detect based on file size
            enable_type_cache: true,
            batch_size: 1000,
            use_async_io: None,            // Auto-detect based on file size
        }
    }
}

/// Type inference cache for performance optimization
static TYPE_CACHE: LazyLock<std::sync::Mutex<HashMap<String, String>>> = LazyLock::new(|| {
    std::sync::Mutex::new(HashMap::new())
});

/// Clear the type cache (useful for testing)
pub fn clear_type_cache() {
    if let Ok(mut cache) = TYPE_CACHE.lock() {
        cache.clear();
    }
}

/// Get cached type information or compute and cache it
fn get_or_compute_type_info(type_name: &str, size: usize) -> String {
    if let Ok(mut cache) = TYPE_CACHE.lock() {
        let key = format!("{}:{}", type_name, size);
        
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
        
        // Compute type info
        let type_info = compute_enhanced_type_info(type_name, size);
        cache.insert(key, type_info.clone());
        type_info
    } else {
        // Fallback if cache is unavailable
        compute_enhanced_type_info(type_name, size)
    }
}

/// Compute enhanced type information
fn compute_enhanced_type_info(type_name: &str, size: usize) -> String {
    // Enhanced type analysis with better categorization
    if type_name.contains("Vec<") {
        extract_vec_inner_type(type_name)
    } else if type_name.contains("HashMap") {
        "HashMap<K,V>".to_string()
    } else if type_name.contains("String") {
        "String".to_string()
    } else if type_name.contains("Box<") {
        extract_box_inner_type(type_name)
    } else {
        // Size-based inference for unknown types
        match size {
            1..=8 => "Primitive".to_string(),
            9..=32 => "SmallStruct".to_string(),
            33..=128 => "MediumStruct".to_string(),
            129..=1024 => "LargeStruct".to_string(),
            _ => "Buffer".to_string(),
        }
    }
}

/// Extract Vec inner type efficiently
fn extract_vec_inner_type(type_name: &str) -> String {
    if let Some(start) = type_name.find("Vec<") {
        if let Some(end) = type_name[start..].find('>') {
            let inner = &type_name[start + 4..start + end];
            return format!("Vec<{}>", inner.trim());
        }
    }
    "Vec<T>".to_string()
}

/// Extract Box inner type efficiently
fn extract_box_inner_type(type_name: &str) -> String {
    if let Some(start) = type_name.find("Box<") {
        if let Some(end) = type_name[start..].find('>') {
            let inner = &type_name[start + 4..start + end];
            return format!("Box<{}>", inner.trim());
        }
    }
    "Box<T>".to_string()
}

/// Optimized allocation processing with intelligent batching
fn process_allocations_optimized(
    allocations: &[AllocationInfo],
    options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    let use_parallel = options.use_parallel_processing
        .unwrap_or(allocations.len() > 1000);
    
    let processed_allocations = if use_parallel && allocations.len() > options.batch_size {
        // Use parallel processing for large datasets
        allocations
            .par_chunks(options.batch_size)
            .map(|chunk| process_allocation_batch(chunk))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect()
    } else {
        // Use sequential processing for smaller datasets
        process_allocation_batch(allocations)?
    };
    
    Ok(serde_json::json!({
        "allocations": processed_allocations,
        "total_count": allocations.len(),
        "processing_mode": if use_parallel { "parallel" } else { "sequential" }
    }))
}

/// Process a batch of allocations
fn process_allocation_batch(allocations: &[AllocationInfo]) -> TrackingResult<Vec<serde_json::Value>> {
    let mut processed = Vec::with_capacity(allocations.len());
    
    for alloc in allocations {
        let enhanced_type = if let Some(type_name) = &alloc.type_name {
            get_or_compute_type_info(type_name, alloc.size)
        } else {
            compute_enhanced_type_info("Unknown", alloc.size)
        };
        
        processed.push(serde_json::json!({
            "ptr": format!("0x{:x}", alloc.ptr),
            "size": alloc.size,
            "type_name": enhanced_type,
            "var_name": alloc.var_name.as_deref().unwrap_or("unnamed"),
            "scope": alloc.scope_name.as_deref().unwrap_or("global"),
            "timestamp": alloc.timestamp_alloc
        }));
    }
    
    Ok(processed)
}

/// Optimized file writing with intelligent buffering
fn write_json_optimized<P: AsRef<Path>>(
    path: P,
    data: &serde_json::Value,
    options: &OptimizedExportOptions,
) -> TrackingResult<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::with_capacity(options.buffer_size, file);
    
    // Determine format based on data size
    let estimated_size = estimate_json_size(data);
    let use_compact = options.use_compact_format
        .unwrap_or(estimated_size > 1_000_000); // Use compact for files > 1MB
    
    if use_compact {
        serde_json::to_writer(&mut writer, data)?;
    } else {
        serde_json::to_writer_pretty(&mut writer, data)?;
    }
    
    writer.flush()?;
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

/// Ultra-fast export implementation
impl MemoryTracker {
    /// Optimized export to standard 4 JSON files (replaces export_separated_json_simple)
    pub fn export_optimized_json_files<P: AsRef<Path>>(
        &self,
        base_path: P,
    ) -> TrackingResult<()> {
        let options = OptimizedExportOptions::default();
        self.export_optimized_json_files_with_options(base_path, options)
    }
    
    /// Export to 5 JSON files including complex types analysis
    pub fn export_optimized_json_files_with_complex_types<P: AsRef<Path>>(
        &self,
        base_path: P,
    ) -> TrackingResult<()> {
        let options = OptimizedExportOptions::default();
        self.export_extensible_json_files_with_options(base_path, &JsonFileType::standard_five(), options)
    }
    
    /// Optimized export to standard 4 JSON files with custom options
    pub fn export_optimized_json_files_with_options<P: AsRef<Path>>(
        &self,
        base_path: P,
        options: OptimizedExportOptions,
    ) -> TrackingResult<()> {
        let start_time = std::time::Instant::now();
        println!("🚀 Starting optimized 4-file JSON export...");
        
        let base_path = base_path.as_ref();
        let base_name = base_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("export");
        let parent_dir = base_path.parent().unwrap_or(Path::new("."));
        
        // Get data once for all files
        let allocations = self.get_active_allocations()?;
        let stats = self.get_stats()?;
        
        println!("📊 Processing {} allocations across 4 standard files...", allocations.len());
        
        // 1. Memory Analysis JSON (标准文件1)
        let memory_path = parent_dir.join(format!("{}_memory_analysis.json", base_name));
        let memory_data = create_optimized_memory_analysis(&allocations, &stats, &options)?;
        write_json_optimized(&memory_path, &memory_data, &options)?;
        
        // 2. Lifetime Analysis JSON (标准文件2)
        let lifetime_path = parent_dir.join(format!("{}_lifetime.json", base_name));
        let lifetime_data = create_optimized_lifetime_analysis(&allocations, &options)?;
        write_json_optimized(&lifetime_path, &lifetime_data, &options)?;
        
        // 3. Unsafe FFI Analysis JSON (标准文件3)
        let unsafe_path = parent_dir.join(format!("{}_unsafe_ffi.json", base_name));
        let unsafe_data = create_optimized_unsafe_ffi_analysis(&allocations, &options)?;
        write_json_optimized(&unsafe_path, &unsafe_data, &options)?;
        
        // 4. Performance Analysis JSON (标准文件4)
        let perf_path = parent_dir.join(format!("{}_performance.json", base_name));
        let perf_data = create_optimized_performance_analysis(&allocations, &stats, start_time, &options)?;
        write_json_optimized(&perf_path, &perf_data, &options)?;
        
        let total_duration = start_time.elapsed();
        println!("✅ Optimized 4-file export completed in {:?}", total_duration);
        println!("📁 Generated standard files:");
        println!("   1. {}_memory_analysis.json", base_name);
        println!("   2. {}_lifetime.json", base_name);
        println!("   3. {}_unsafe_ffi.json", base_name);
        println!("   4. {}_performance.json", base_name);
        
        // 显示优化效果
        if options.use_parallel_processing.unwrap_or(allocations.len() > 1000) {
            println!("💡 Applied parallel processing optimization");
        }
        if options.enable_type_cache {
            println!("💡 Applied type inference caching");
        }
        println!("💡 Applied optimized buffering ({} KB)", options.buffer_size / 1024);
        
        Ok(())
    }
    
    /// A generic export method reserved for future expansion. can easily add a 5th and 6th JSON file
    pub fn export_extensible_json_files<P: AsRef<Path>>(
        &self,
        base_path: P,
        file_types: &[JsonFileType],
    ) -> TrackingResult<()> {
        let options = OptimizedExportOptions::default();
        self.export_extensible_json_files_with_options(base_path, file_types, options)
    }
    
    /// A generic export method reserved for future expansion. can easily add a 5th and 6th JSON file
    pub fn export_extensible_json_files_with_options<P: AsRef<Path>>(
        &self,
        base_path: P,
        file_types: &[JsonFileType],
        options: OptimizedExportOptions,
    ) -> TrackingResult<()> {
        let start_time = std::time::Instant::now();
        println!("🚀 Starting extensible JSON export for {} files...", file_types.len());
        
        let base_path = base_path.as_ref();
        let base_name = base_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("export");
        let parent_dir = base_path.parent().unwrap_or(Path::new("."));
        
        // Get data once for all files
        let allocations = self.get_active_allocations()?;
        let stats = self.get_stats()?;
        
        println!("📊 Processing {} allocations...", allocations.len());
        
        // genearte files
        for file_type in file_types {
            let (filename, data) = match file_type {
                JsonFileType::MemoryAnalysis => {
                    let filename = format!("{}_memory_analysis.json", base_name);
                    let data = create_optimized_memory_analysis(&allocations, &stats, &options)?;
                    (filename, data)
                }
                JsonFileType::Lifetime => {
                    let filename = format!("{}_lifetime.json", base_name);
                    let data = create_optimized_lifetime_analysis(&allocations, &options)?;
                    (filename, data)
                }
                JsonFileType::UnsafeFfi => {
                    let filename = format!("{}_unsafe_ffi.json", base_name);
                    let data = create_optimized_unsafe_ffi_analysis(&allocations, &options)?;
                    (filename, data)
                }
                JsonFileType::Performance => {
                    let filename = format!("{}_performance.json", base_name);
                    let data = create_optimized_performance_analysis(&allocations, &stats, start_time, &options)?;
                    (filename, data)
                }
                JsonFileType::ComplexTypes => {
                    let filename = format!("{}_complex_types.json", base_name);
                    let data = create_optimized_complex_types_analysis(&allocations, &options)?;
                    (filename, data)
                }
                // future can easily add new file types
                // JsonFileType::AsyncAnalysis => { ... }
                // JsonFileType::ThreadSafety => { ... }
            };
            
            let file_path = parent_dir.join(filename);
            write_json_optimized(&file_path, &data, &options)?;
            println!("   ✅ Generated: {}", file_path.file_name().unwrap().to_string_lossy());
        }
        
        let total_duration = start_time.elapsed();
        println!("✅ Extensible export completed in {:?}", total_duration);
        
        Ok(())
    }
}

/// Create optimized memory analysis
fn create_optimized_memory_analysis(
    allocations: &[AllocationInfo],
    stats: &crate::core::types::MemoryStats,
    options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    let processed_allocations = process_allocations_optimized(allocations, options)?;
    
    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "memory_analysis_optimized",
            "optimization_level": "high",
            "total_allocations": allocations.len(),
            "export_version": "2.0",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "memory_stats": {
            "total_allocated": stats.total_allocated,
            "active_memory": stats.active_memory,
            "peak_memory": stats.peak_memory,
            "total_allocations": stats.total_allocations
        },
        "allocations": processed_allocations
    }))
}

/// Create optimized lifetime analysis
fn create_optimized_lifetime_analysis(
    allocations: &[AllocationInfo],
    options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    // 生命周期分析：按作用域分组分析
    let mut scope_analysis: HashMap<String, (usize, usize, Vec<usize>)> = HashMap::new();
    
    for alloc in allocations {
        let scope = alloc.scope_name.as_deref().unwrap_or("global");
        let entry = scope_analysis.entry(scope.to_string()).or_insert((0, 0, Vec::new()));
        entry.0 += alloc.size;  // 总大小
        entry.1 += 1;          // 分配数量
        entry.2.push(alloc.size); // 大小列表用于统计
    }
    
    // 转换为JSON格式
    let mut scope_stats: Vec<_> = scope_analysis.into_iter()
        .map(|(scope, (total_size, count, sizes))| {
            let avg_size = if count > 0 { total_size / count } else { 0 };
            let max_size = sizes.iter().max().copied().unwrap_or(0);
            let min_size = sizes.iter().min().copied().unwrap_or(0);
            
            serde_json::json!({
                "scope_name": scope,
                "total_size": total_size,
                "allocation_count": count,
                "average_size": avg_size,
                "max_size": max_size,
                "min_size": min_size
            })
        })
        .collect();
    
    // 按总大小排序
    scope_stats.sort_by(|a, b| {
        b["total_size"].as_u64().unwrap_or(0)
            .cmp(&a["total_size"].as_u64().unwrap_or(0))
    });
    
    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "lifetime_analysis_optimized",
            "optimization_level": "high",
            "total_scopes": scope_stats.len(),
            "export_version": "2.0",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "scope_analysis": scope_stats,
        "summary": {
            "total_allocations": allocations.len(),
            "unique_scopes": scope_stats.len()
        }
    }))
}

/// Create optimized unsafe FFI analysis
fn create_optimized_unsafe_ffi_analysis(
    allocations: &[AllocationInfo],
    _options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    // Analyze possible unsafe operations and FFI-related allocations
    let mut unsafe_indicators = Vec::new();
    let mut ffi_patterns = Vec::new();
    
    for alloc in allocations {
        // Check for unsafe patterns in type names
        if let Some(type_name) = &alloc.type_name {
            if type_name.contains("*mut") || type_name.contains("*const") {
                unsafe_indicators.push(serde_json::json!({
                    "ptr": format!("0x{:x}", alloc.ptr),
                    "type": "raw_pointer",
                    "type_name": type_name,
                    "size": alloc.size,
                    "risk_level": "high"
                }));
            } else if type_name.contains("extern") || type_name.contains("libc::") {
                ffi_patterns.push(serde_json::json!({
                    "ptr": format!("0x{:x}", alloc.ptr),
                    "type": "ffi_related",
                    "type_name": type_name,
                    "size": alloc.size,
                    "risk_level": "medium"
                }));
            }
        }
        
        // Check for unsafe patterns in variable names
        if let Some(var_name) = &alloc.var_name {
            if var_name.contains("unsafe") || var_name.contains("raw") {
                unsafe_indicators.push(serde_json::json!({
                    "ptr": format!("0x{:x}", alloc.ptr),
                    "type": "unsafe_variable",
                    "var_name": var_name,
                    "size": alloc.size,
                    "risk_level": "medium"
                }));
            }
        }
    }
    
    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "unsafe_ffi_analysis_optimized",
            "optimization_level": "high",
            "total_allocations_analyzed": allocations.len(),
            "export_version": "2.0",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "unsafe_indicators": unsafe_indicators,
        "ffi_patterns": ffi_patterns,
        "summary": {
            "unsafe_count": unsafe_indicators.len(),
            "ffi_count": ffi_patterns.len(),
            "total_risk_items": unsafe_indicators.len() + ffi_patterns.len(),
            "risk_assessment": if unsafe_indicators.len() + ffi_patterns.len() > 10 {
                "high"
            } else if unsafe_indicators.len() + ffi_patterns.len() > 5 {
                "medium"
            } else {
                "low"
            }
        }
    }))
}

/// Create optimized performance analysis
fn create_optimized_performance_analysis(
    allocations: &[AllocationInfo],
    stats: &crate::core::types::MemoryStats,
    start_time: std::time::Instant,
    options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    let processing_time = start_time.elapsed();
    let allocations_per_second = if processing_time.as_secs() > 0 {
        allocations.len() as f64 / processing_time.as_secs_f64()
    } else {
        allocations.len() as f64 / 0.001 // 假设最少1ms
    };
    
    // 分析分配大小分布
    let mut size_distribution = HashMap::new();
    for alloc in allocations {
        let category = match alloc.size {
            0..=64 => "tiny",
            65..=256 => "small", 
            257..=1024 => "medium",
            1025..=4096 => "large",
            4097..=16384 => "huge",
            _ => "massive",
        };
        *size_distribution.entry(category).or_insert(0) += 1;
    }
    
    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "performance_analysis_optimized",
            "optimization_level": "high",
            "export_version": "2.0",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "export_performance": {
            "total_processing_time_ms": processing_time.as_millis(),
            "allocations_processed": allocations.len(),
            "processing_rate": {
                "allocations_per_second": allocations_per_second,
                "performance_class": if allocations_per_second > 10000.0 {
                    "excellent"
                } else if allocations_per_second > 1000.0 {
                    "good"
                } else {
                    "needs_optimization"
                }
            }
        },
        "memory_performance": {
            "total_allocated": stats.total_allocated,
            "active_memory": stats.active_memory,
            "peak_memory": stats.peak_memory,
            "memory_efficiency": if stats.peak_memory > 0 {
                (stats.active_memory as f64 / stats.peak_memory as f64 * 100.0) as u64
            } else {
                100
            }
        },
        "allocation_distribution": size_distribution,
        "optimization_status": {
            "type_caching": options.enable_type_cache,
            "parallel_processing": options.use_parallel_processing.unwrap_or(allocations.len() > 1000),
            "buffer_size_kb": options.buffer_size / 1024,
            "batch_size": options.batch_size
        }
    }))
}

/// Create optimized complex types analysis
fn create_optimized_complex_types_analysis(
    allocations: &[AllocationInfo],
    options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    // 复杂类型分析：识别和分析各种复杂的Rust类型
    let mut complex_type_stats: HashMap<String, ComplexTypeInfo> = HashMap::new();
    let mut generic_types = Vec::new();
    let mut trait_objects = Vec::new();
    let mut smart_pointers = Vec::new();
    let mut collections = Vec::new();
    
    // 使用并行处理分析复杂类型
    let use_parallel = options.use_parallel_processing.unwrap_or(allocations.len() > 1000);
    
    if use_parallel {
        // 并行分析复杂类型
        let results: Vec<_> = allocations
            .par_chunks(options.batch_size)
            .map(|chunk| analyze_complex_types_batch(chunk))
            .collect();
        
        // 合并结果
        for batch_result in results {
            for (type_name, info) in batch_result.type_stats {
                let entry = complex_type_stats.entry(type_name).or_insert_with(|| ComplexTypeInfo::new());
                entry.merge(info);
            }
            generic_types.extend(batch_result.generic_types);
            trait_objects.extend(batch_result.trait_objects);
            smart_pointers.extend(batch_result.smart_pointers);
            collections.extend(batch_result.collections);
        }
    } else {
        // 串行分析
        let batch_result = analyze_complex_types_batch(allocations);
        complex_type_stats = batch_result.type_stats;
        generic_types = batch_result.generic_types;
        trait_objects = batch_result.trait_objects;
        smart_pointers = batch_result.smart_pointers;
        collections = batch_result.collections;
    }
    
    // 转换为JSON格式并排序
    let mut type_analysis: Vec<_> = complex_type_stats.into_iter()
        .map(|(type_name, info)| {
            serde_json::json!({
                "type_name": type_name,
                "category": info.category,
                "total_size": info.total_size,
                "allocation_count": info.allocation_count,
                "average_size": if info.allocation_count > 0 { 
                    info.total_size / info.allocation_count 
                } else { 
                    0 
                },
                "max_size": info.max_size,
                "complexity_score": info.complexity_score,
                "memory_efficiency": calculate_memory_efficiency(&type_name, info.total_size, info.allocation_count),
                "optimization_suggestions": generate_optimization_suggestions(&type_name, &info)
            })
        })
        .collect();
    
    // 按复杂度分数和总大小排序
    type_analysis.sort_by(|a, b| {
        let score_cmp = b["complexity_score"].as_u64().unwrap_or(0)
            .cmp(&a["complexity_score"].as_u64().unwrap_or(0));
        if score_cmp == std::cmp::Ordering::Equal {
            b["total_size"].as_u64().unwrap_or(0)
                .cmp(&a["total_size"].as_u64().unwrap_or(0))
        } else {
            score_cmp
        }
    });
    
    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "complex_types_analysis_optimized",
            "optimization_level": "high",
            "total_allocations_analyzed": allocations.len(),
            "unique_complex_types": type_analysis.len(),
            "export_version": "2.0",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "processing_mode": if use_parallel { "parallel" } else { "sequential" }
        },
        "complex_type_analysis": type_analysis,
        "categorized_types": {
            "generic_types": generic_types,
            "trait_objects": trait_objects,
            "smart_pointers": smart_pointers,
            "collections": collections
        },
        "summary": {
            "total_complex_types": type_analysis.len(),
            "generic_type_count": generic_types.len(),
            "trait_object_count": trait_objects.len(),
            "smart_pointer_count": smart_pointers.len(),
            "collection_count": collections.len(),
            "complexity_distribution": calculate_complexity_distribution(&type_analysis)
        },
        "optimization_recommendations": generate_global_optimization_recommendations(&type_analysis)
    }))
}

/// 复杂类型信息结构
#[derive(Debug, Clone)]
struct ComplexTypeInfo {
    category: String,
    total_size: usize,
    allocation_count: usize,
    max_size: usize,
    complexity_score: u64,
}

impl ComplexTypeInfo {
    fn new() -> Self {
        Self {
            category: String::new(),
            total_size: 0,
            allocation_count: 0,
            max_size: 0,
            complexity_score: 0,
        }
    }
    
    fn merge(&mut self, other: ComplexTypeInfo) {
        self.total_size += other.total_size;
        self.allocation_count += other.allocation_count;
        self.max_size = self.max_size.max(other.max_size);
        self.complexity_score = self.complexity_score.max(other.complexity_score);
        if self.category.is_empty() {
            self.category = other.category;
        }
    }
}

/// 批量分析结果
struct ComplexTypeBatchResult {
    type_stats: HashMap<String, ComplexTypeInfo>,
    generic_types: Vec<serde_json::Value>,
    trait_objects: Vec<serde_json::Value>,
    smart_pointers: Vec<serde_json::Value>,
    collections: Vec<serde_json::Value>,
}

/// 批量分析复杂类型
fn analyze_complex_types_batch(allocations: &[AllocationInfo]) -> ComplexTypeBatchResult {
    let mut type_stats: HashMap<String, ComplexTypeInfo> = HashMap::new();
    let mut generic_types = Vec::new();
    let mut trait_objects = Vec::new();
    let mut smart_pointers = Vec::new();
    let mut collections = Vec::new();
    
    for alloc in allocations {
        if let Some(type_name) = &alloc.type_name {
            let normalized_type = normalize_type_name(type_name);
            let category = categorize_complex_type(type_name);
            let complexity = calculate_type_complexity(type_name);
            
            // 更新类型统计
            let entry = type_stats.entry(normalized_type.clone()).or_insert_with(|| {
                let mut info = ComplexTypeInfo::new();
                info.category = category.clone();
                info.complexity_score = complexity;
                info
            });
            entry.total_size += alloc.size;
            entry.allocation_count += 1;
            entry.max_size = entry.max_size.max(alloc.size);
            
            // 分类收集
            let type_info = serde_json::json!({
                "ptr": format!("0x{:x}", alloc.ptr),
                "type_name": type_name,
                "normalized_type": normalized_type,
                "size": alloc.size,
                "var_name": alloc.var_name.as_deref().unwrap_or("unnamed"),
                "complexity_score": complexity
            });
            
            match category.as_str() {
                "Generic" => generic_types.push(type_info),
                "TraitObject" => trait_objects.push(type_info),
                "SmartPointer" => smart_pointers.push(type_info),
                "Collection" => collections.push(type_info),
                _ => {} // 其他类型不特别收集
            }
        }
    }
    
    ComplexTypeBatchResult {
        type_stats,
        generic_types,
        trait_objects,
        smart_pointers,
        collections,
    }
}

/// 标准化类型名称
fn normalize_type_name(type_name: &str) -> String {
    // 移除具体的泛型参数，保留结构
    if type_name.contains('<') {
        if let Some(base) = type_name.split('<').next() {
            format!("{}<T>", base)
        } else {
            type_name.to_string()
        }
    } else {
        type_name.to_string()
    }
}

/// 分类复杂类型
fn categorize_complex_type(type_name: &str) -> String {
    if type_name.contains("dyn ") {
        "TraitObject".to_string()
    } else if type_name.starts_with("Box<") || type_name.starts_with("Rc<") || 
              type_name.starts_with("Arc<") || type_name.starts_with("RefCell<") {
        "SmartPointer".to_string()
    } else if type_name.starts_with("Vec<") || type_name.starts_with("HashMap<") || 
              type_name.starts_with("BTreeMap<") || type_name.starts_with("HashSet<") {
        "Collection".to_string()
    } else if type_name.contains('<') && type_name.contains('>') {
        "Generic".to_string()
    } else if type_name.contains("::") {
        "ModulePath".to_string()
    } else {
        "Simple".to_string()
    }
}

/// 计算类型复杂度
fn calculate_type_complexity(type_name: &str) -> u64 {
    let mut score = 0u64;
    
    // 基础分数
    score += 1;
    
    // 泛型参数增加复杂度
    score += type_name.matches('<').count() as u64 * 2;
    
    // 嵌套层级增加复杂度
    let nesting_level = type_name.chars().filter(|&c| c == '<').count();
    score += nesting_level as u64 * 3;
    
    // 特殊类型增加复杂度
    if type_name.contains("dyn ") { score += 5; }
    if type_name.contains("impl ") { score += 4; }
    if type_name.contains("async") { score += 3; }
    if type_name.contains("Future") { score += 3; }
    
    // 智能指针增加复杂度
    if type_name.contains("Box<") { score += 2; }
    if type_name.contains("Rc<") { score += 3; }
    if type_name.contains("Arc<") { score += 4; }
    if type_name.contains("RefCell<") { score += 3; }
    
    score
}

/// Calculate memory efficiency based on type and average size
fn calculate_memory_efficiency(type_name: &str, total_size: usize, count: usize) -> u64 {
    if count == 0 { return 100; }
    
    let avg_size = total_size / count;
    
    // Calculate efficiency based on type and average size
    let efficiency = if type_name.contains("Vec<") {
        // Vec efficiency depends on capacity utilization
        if avg_size < 64 { 60 } else { 85 }
    } else if type_name.contains("HashMap<") {
        // HashMap has additional overhead
        if avg_size < 128 { 50 } else { 75 }
    } else if type_name.contains("Box<") {
        // Box is usually very efficient
        90
    } else if type_name.contains("Arc<") || type_name.contains("Rc<") {
        // Reference counting has overhead
        80
    } else {
        // Default efficiency
        85
    };
    
    efficiency
}

/// Generate optimization suggestions based on type and allocation information
fn generate_optimization_suggestions(type_name: &str, info: &ComplexTypeInfo) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    if info.allocation_count > 100 {
        suggestions.push("Consider using object pooling for frequently allocated types".to_string());
    }
    
    if type_name.contains("Vec<") && info.total_size > 1024 * 1024 {
        suggestions.push("Consider pre-allocating Vec capacity to reduce reallocations".to_string());
    }
    
    if type_name.contains("HashMap<") && info.allocation_count > 50 {
        suggestions.push("Consider using FxHashMap for better performance".to_string());
    }
    
    if type_name.contains("Box<") && info.allocation_count > 200 {
        suggestions.push("Consider using arena allocation for many small Box allocations".to_string());
    }
    
    if info.complexity_score > 10 {
        suggestions.push("High complexity type - consider simplifying or using type aliases".to_string());
    }
    
    suggestions
}

/// Calculate complexity distribution
fn calculate_complexity_distribution(type_analysis: &[serde_json::Value]) -> serde_json::Value {
    let mut low = 0;
    let mut medium = 0;
    let mut high = 0;
    let mut very_high = 0;
    
    for analysis in type_analysis {
        if let Some(score) = analysis["complexity_score"].as_u64() {
            match score {
                0..=3 => low += 1,
                4..=7 => medium += 1,
                8..=15 => high += 1,
                _ => very_high += 1,
            }
        }
    }
    
    serde_json::json!({
        "low_complexity": low,
        "medium_complexity": medium,
        "high_complexity": high,
        "very_high_complexity": very_high
    })
}

/// Generate global optimization recommendations based on type analysis
fn generate_global_optimization_recommendations(type_analysis: &[serde_json::Value]) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    let total_types = type_analysis.len();
    let high_complexity_count = type_analysis.iter()
        .filter(|t| t["complexity_score"].as_u64().unwrap_or(0) > 10)
        .count();
    
    if high_complexity_count > total_types / 4 {
        recommendations.push("Consider refactoring high-complexity types to improve maintainability".to_string());
    }
    
    let large_allocation_count = type_analysis.iter()
        .filter(|t| t["allocation_count"].as_u64().unwrap_or(0) > 100)
        .count();
    
    if large_allocation_count > 5 {
        recommendations.push("Multiple types with high allocation frequency - consider object pooling".to_string());
    }
    
    recommendations.push("Use 'cargo clippy' to identify additional optimization opportunities".to_string());
    recommendations.push("Consider profiling with 'perf' or 'valgrind' for detailed performance analysis".to_string());
    
    recommendations
}

/// Create optimized type analysis with caching
fn create_optimized_type_analysis(
    allocations: &[AllocationInfo],
    options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    let mut type_stats: HashMap<String, (usize, usize, usize)> = HashMap::new();
    
    // Use parallel processing for type analysis if beneficial
    let use_parallel = options.use_parallel_processing
        .unwrap_or(allocations.len() > 1000);
    
    if use_parallel {
        // Parallel type analysis
        let type_results: Vec<_> = allocations
            .par_chunks(options.batch_size)
            .map(|chunk| {
                let mut local_stats: HashMap<String, (usize, usize, usize)> = HashMap::new();
                for alloc in chunk {
                    let type_name = if let Some(name) = &alloc.type_name {
                        get_or_compute_type_info(name, alloc.size)
                    } else {
                        compute_enhanced_type_info("Unknown", alloc.size)
                    };
                    
                    let entry = local_stats.entry(type_name).or_insert((0, 0, 0));
                    entry.0 += alloc.size;  // total size
                    entry.1 += 1;          // count
                    entry.2 = entry.2.max(alloc.size); // max size
                }
                local_stats
            })
            .collect();
        
        // Merge results
        for local_stats in type_results {
            for (type_name, (size, count, max_size)) in local_stats {
                let entry = type_stats.entry(type_name).or_insert((0, 0, 0));
                entry.0 += size;
                entry.1 += count;
                entry.2 = entry.2.max(max_size);
            }
        }
    } else {
        // Sequential type analysis
        for alloc in allocations {
            let type_name = if let Some(name) = &alloc.type_name {
                get_or_compute_type_info(name, alloc.size)
            } else {
                compute_enhanced_type_info("Unknown", alloc.size)
            };
            
            let entry = type_stats.entry(type_name).or_insert((0, 0, 0));
            entry.0 += alloc.size;
            entry.1 += 1;
            entry.2 = entry.2.max(alloc.size);
        }
    }
    
    // Convert to sorted JSON
    let mut type_list: Vec<_> = type_stats.into_iter()
        .map(|(type_name, (total_size, count, max_size))| {
            serde_json::json!({
                "type_name": type_name,
                "total_size": total_size,
                "allocation_count": count,
                "max_allocation_size": max_size,
                "average_size": if count > 0 { total_size / count } else { 0 }
            })
        })
        .collect();
    
    // Sort by total size (descending)
    type_list.sort_by(|a, b| {
        b["total_size"].as_u64().unwrap_or(0)
            .cmp(&a["total_size"].as_u64().unwrap_or(0))
    });
    
    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "type_analysis_optimized",
            "processing_mode": if use_parallel { "parallel" } else { "sequential" },
            "cache_enabled": options.enable_type_cache,
            "unique_types": type_list.len()
        },
        "type_statistics": type_list
    }))
}

/// Create fast allocation summary
fn create_fast_allocation_summary(
    allocations: &[AllocationInfo],
    stats: &crate::core::types::MemoryStats,
) -> TrackingResult<serde_json::Value> {
    // Quick summary without heavy processing
    let total_size: usize = allocations.iter().map(|a| a.size).sum();
    let avg_size = if !allocations.is_empty() { 
        total_size / allocations.len() 
    } else { 
        0 
    };
    
    // Size distribution (fast calculation)
    let mut small_count = 0;
    let mut medium_count = 0;
    let mut large_count = 0;
    
    for alloc in allocations {
        match alloc.size {
            0..=256 => small_count += 1,
            257..=4096 => medium_count += 1,
            _ => large_count += 1,
        }
    }
    
    Ok(serde_json::json!({
        "metadata": {
            "summary_type": "fast_allocation_summary",
            "generation_time": "minimal"
        },
        "overview": {
            "total_allocations": allocations.len(),
            "total_size": total_size,
            "average_size": avg_size,
            "active_memory": stats.active_memory,
            "peak_memory": stats.peak_memory
        },
        "size_distribution": {
            "small_allocations": {
                "count": small_count,
                "size_range": "0-256 bytes"
            },
            "medium_allocations": {
                "count": medium_count,
                "size_range": "257-4096 bytes"
            },
            "large_allocations": {
                "count": large_count,
                "size_range": ">4096 bytes"
            }
        }
    }))
}

/// Create performance metrics
fn create_performance_metrics(
    allocations: &[AllocationInfo],
    start_time: std::time::Instant,
) -> TrackingResult<serde_json::Value> {
    let processing_time = start_time.elapsed();
    let allocations_per_second = if processing_time.as_secs() > 0 {
        allocations.len() as f64 / processing_time.as_secs_f64()
    } else {
        allocations.len() as f64 / 0.001 // Assume 1ms minimum
    };
    
    Ok(serde_json::json!({
        "metadata": {
            "metrics_type": "performance_optimized",
            "measurement_time": processing_time.as_millis()
        },
        "performance": {
            "total_processing_time_ms": processing_time.as_millis(),
            "allocations_processed": allocations.len(),
            "processing_rate": {
                "allocations_per_second": allocations_per_second,
                "performance_class": if allocations_per_second > 10000.0 {
                    "excellent"
                } else if allocations_per_second > 1000.0 {
                    "good"
                } else {
                    "needs_optimization"
                }
            }
        },
        "optimization_status": {
            "type_caching": "enabled",
            "parallel_processing": "auto-detected",
            "buffer_optimization": "enabled",
            "format_optimization": "auto-detected"
        }
    }))
}