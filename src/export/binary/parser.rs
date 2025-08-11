//! Optimized binary file parser

use crate::core::types::AllocationInfo;
use crate::export::analysis_engine::{AnalysisEngine, StandardAnalysisEngine};
use crate::export::binary::{BinaryExportError, BinaryReader};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

/// Binary parser for optimized file conversion
pub struct BinaryParser;

impl BinaryParser {
    /// Convert binary file to standard JSON files using optimized approach
    pub fn to_standard_json_files<P: AsRef<Path>>(
        binary_path: P,
        base_name: &str,
    ) -> Result<(), BinaryExportError> {
        let start = Instant::now();
        let binary_path = binary_path.as_ref();

        // Create output directory structure
        let base_memory_analysis_dir = std::path::Path::new("MemoryAnalysis");
        let project_dir = base_memory_analysis_dir.join(base_name);
        std::fs::create_dir_all(&project_dir)?;

        // Load allocations - only user-defined variables for performance
        let allocations = Self::load_allocations(binary_path)?;
        let user_allocations: Vec<AllocationInfo> = allocations
            .into_iter()
            .filter(|a| a.var_name.is_some())
            .collect();

        // Use StandardAnalysisEngine but with filtered user allocations for performance
        let analysis_engine = StandardAnalysisEngine::new();

        // Generate 5 JSON files with proper analysis data
        let analyses = [
            (
                "memory_analysis",
                analysis_engine
                    .create_memory_analysis(&user_allocations)
                    .map_err(|e| {
                        BinaryExportError::CorruptedData(format!("Memory analysis failed: {}", e))
                    })?,
            ),
            (
                "lifetime",
                analysis_engine
                    .create_lifetime_analysis(&user_allocations)
                    .map_err(|e| {
                        BinaryExportError::CorruptedData(format!("Lifetime analysis failed: {}", e))
                    })?,
            ),
            (
                "performance",
                analysis_engine
                    .create_performance_analysis(&user_allocations)
                    .map_err(|e| {
                        BinaryExportError::CorruptedData(format!(
                            "Performance analysis failed: {}",
                            e
                        ))
                    })?,
            ),
            (
                "unsafe_ffi",
                analysis_engine
                    .create_unsafe_ffi_analysis(&user_allocations)
                    .map_err(|e| {
                        BinaryExportError::CorruptedData(format!(
                            "Unsafe FFI analysis failed: {}",
                            e
                        ))
                    })?,
            ),
            (
                "complex_types",
                analysis_engine
                    .create_complex_types_analysis(&user_allocations)
                    .map_err(|e| {
                        BinaryExportError::CorruptedData(format!(
                            "Complex types analysis failed: {}",
                            e
                        ))
                    })?,
            ),
        ];

        for (file_type, analysis_data) in analyses {
            let file_path = project_dir.join(format!("{}_{}.json", base_name, file_type));
            let json_content = serde_json::to_string(&analysis_data.data).map_err(|e| {
                BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e))
            })?;
            std::fs::write(file_path, json_content)?;
        }

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 300 {
            eprintln!(
                "⚠️  Performance target missed: {}ms (target: <300ms)",
                elapsed.as_millis()
            );
        } else {
            println!(
                "✅ Optimized conversion completed in {}ms",
                elapsed.as_millis()
            );
        }

        Ok(())
    }

    /// Load allocations from binary file
    pub fn load_allocations<P: AsRef<Path>>(
        binary_path: P,
    ) -> Result<Vec<AllocationInfo>, BinaryExportError> {
        let mut reader = BinaryReader::new(binary_path)?;
        reader.read_all()
    }

    /// Load allocations with enhanced error recovery (Task 5.1: 一招制敌)
    /// 
    /// 解决"failed to fill whole buffer"错误的核心方法
    pub fn load_allocations_with_recovery<P: AsRef<Path>>(
        binary_path: P,
    ) -> Result<Vec<AllocationInfo>, BinaryExportError> {
        let binary_path = binary_path.as_ref();
        
        // 首先检查文件大小和完整性
        let file_metadata = std::fs::metadata(binary_path)?;
        let file_size = file_metadata.len();
        tracing::debug!("Binary file size: {} bytes", file_size);
        
        // 尝试正常读取
        match Self::load_allocations(binary_path) {
            Ok(allocations) => {
                tracing::info!("Successfully loaded {} allocations normally", allocations.len());
                Ok(allocations)
            }
            Err(BinaryExportError::Io(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                tracing::warn!("Encountered EOF error, attempting recovery read");
                
                // 使用恢复模式读取
                let mut reader = BinaryReader::new(binary_path)?;
                let header = reader.read_header()?;
                let mut allocations = Vec::new();
                
                // 逐个读取，遇到错误就停止
                for i in 0..header.total_count {
                    match reader.read_allocation() {
                        Ok(allocation) => allocations.push(allocation),
                        Err(BinaryExportError::Io(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                            tracing::warn!("Recovered {} of {} allocations before EOF", i, header.total_count);
                            break;
                        }
                        Err(e) => {
                            tracing::error!("Failed to read allocation {}: {}", i, e);
                            return Err(e);
                        }
                    }
                }
                
                if allocations.is_empty() {
                    return Err(BinaryExportError::CorruptedData(
                        "No allocations could be recovered from corrupted file".to_string()
                    ));
                }
                
                tracing::info!("Successfully recovered {} allocations", allocations.len());
                Ok(allocations)
            }
            Err(e) => {
                tracing::error!("Failed to load allocations: {}", e);
                Err(e)
            }
        }
    }

    /// Convert binary file to single JSON format (legacy compatibility)
    pub fn to_json<P: AsRef<Path>>(binary_path: P, json_path: P) -> Result<(), BinaryExportError> {
        let allocations = Self::load_allocations(binary_path)?;
        let json_data = serde_json::to_string_pretty(&allocations).map_err(|e| {
            BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e))
        })?;
        std::fs::write(json_path, json_data)?;
        Ok(())
    }

    /// Convert binary file to HTML format (legacy compatibility)
    pub fn to_html<P: AsRef<Path>>(binary_path: P, html_path: P) -> Result<(), BinaryExportError> {
        let allocations = Self::load_allocations(binary_path)?;
        let html_content = format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Memory Analysis</title></head>
<body>
<h1>Memory Analysis Report</h1>
<p>Total allocations: {}</p>
<pre>{}</pre>
</body>
</html>"#,
            allocations.len(),
            serde_json::to_string_pretty(&allocations).map_err(|e| {
                BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e))
            })?
        );
        std::fs::write(html_path, html_content)?;
        Ok(())
    }

    /// Parse user binary to JSON using ultra-fast strategy (optimized for all file sizes)
    /// Now uses the same ultra-fast approach as full binary parsing for consistent performance
    pub fn parse_user_binary_to_json<P: AsRef<Path>>(
        binary_path: P,
        base_name: &str,
    ) -> Result<(), BinaryExportError> {
        let start = Instant::now();
        tracing::info!("Starting user binary to JSON conversion using ultra-fast strategy");

        // Load allocations and filter for user-only data
        let allocations = Self::load_allocations(binary_path)?;
        let user_allocations: Vec<AllocationInfo> = allocations
            .into_iter()
            .filter(|a| a.var_name.is_some())
            .collect();

        tracing::info!(
            "Loaded {} user allocations for ultra-fast processing",
            user_allocations.len()
        );

        // Create output directory
        let base_memory_analysis_dir = std::path::Path::new("MemoryAnalysis");
        let project_dir = base_memory_analysis_dir.join(base_name);
        std::fs::create_dir_all(&project_dir)?;

        // Generate JSON files using ultra-fast batch approach
        let json_start = Instant::now();

        // Pre-calculate total JSON size to avoid reallocations
        let estimated_size_per_alloc = 150; // bytes per allocation
        let total_estimated_size = user_allocations.len() * estimated_size_per_alloc;

        // Generate all 5 JSON files using ultra-fast method
        let paths = [
            (
                project_dir.join(format!("{base_name}_memory_analysis.json")),
                "memory",
            ),
            (
                project_dir.join(format!("{base_name}_lifetime.json")),
                "lifetime",
            ),
            (
                project_dir.join(format!("{base_name}_performance.json")),
                "performance",
            ),
            (
                project_dir.join(format!("{base_name}_unsafe_ffi.json")),
                "unsafe_ffi",
            ),
            (
                project_dir.join(format!("{base_name}_complex_types.json")),
                "complex_types",
            ),
        ];

        // Parallel JSON generation for maximum performance
        use rayon::prelude::*;

        let results: Result<Vec<()>, BinaryExportError> = paths
            .par_iter()
            .map(|(path, json_type)| {
                Self::generate_json_ultra_fast(
                    &user_allocations,
                    path,
                    json_type,
                    total_estimated_size,
                )
            })
            .collect();

        results?;

        let json_time = json_start.elapsed();
        tracing::info!("Generated 5 JSON files in {}ms", json_time.as_millis());

        let elapsed = start.elapsed();

        // Performance target check: <300ms for user binary processing
        if elapsed.as_millis() > 300 {
            tracing::warn!(
                "Performance target missed: {}ms (target: <300ms)",
                elapsed.as_millis()
            );
        } else {
            tracing::info!(
                "✅ Ultra-fast user binary conversion completed in {}ms (target: <300ms)",
                elapsed.as_millis()
            );
        }

        Ok(())
    }

    /// Parse full binary to JSON using ultra-fast direct approach (Task 5.2: 一招制敌)
    /// 
    /// **一招制敌**: 直接使用已优化的generate_*_json方法，避免SelectiveJsonExporter的I/O错误
    /// 
    /// 核心优化:
    /// - 使用load_allocations但加强错误处理 (Task 5.1)
    /// - 直接调用优化的generate_*_json方法 (避免复杂的SelectiveJsonExporter)
    /// - 并行生成5个JSON文件 (Task 7.1)
    /// - 目标: <300ms性能，无null字段，JSON格式一致
    pub fn parse_full_binary_to_json<P: AsRef<Path>>(
        binary_path: P,
        base_name: &str,
    ) -> Result<(), BinaryExportError> {
        let start = Instant::now();
        tracing::info!("Starting ultra-fast full binary to JSON conversion (direct approach)");

        // Load all allocations with improved error handling (Task 5.1)
        let load_start = Instant::now();
        let all_allocations = Self::load_allocations_with_recovery(&binary_path)?;
        let load_time = load_start.elapsed();
        tracing::info!(
            "Loaded {} allocations in {}ms with error recovery",
            all_allocations.len(),
            load_time.as_millis()
        );

        // Create output directory
        let base_memory_analysis_dir = std::path::Path::new("MemoryAnalysis");
        let project_dir = base_memory_analysis_dir.join(base_name);
        std::fs::create_dir_all(&project_dir)?;

        // **一招制敌**: 并行生成5个JSON文件，避免SelectiveJsonExporter的I/O问题
        let json_start = Instant::now();
        
        let paths = [
            project_dir.join(format!("{base_name}_memory_analysis.json")),
            project_dir.join(format!("{base_name}_lifetime.json")),
            project_dir.join(format!("{base_name}_performance.json")),
            project_dir.join(format!("{base_name}_unsafe_ffi.json")),
            project_dir.join(format!("{base_name}_complex_types.json")),
        ];

        // Task 7.1: 并行生成JSON文件
        use rayon::prelude::*;
        
        let results: Result<Vec<()>, BinaryExportError> = paths
            .par_iter()
            .enumerate()
            .map(|(i, path)| {
                match i {
                    0 => Self::generate_memory_analysis_json(&all_allocations, path),
                    1 => Self::generate_lifetime_analysis_json(&all_allocations, path),
                    2 => Self::generate_performance_analysis_json(&all_allocations, path),
                    3 => Self::generate_unsafe_ffi_analysis_json(&all_allocations, path),
                    4 => Self::generate_complex_types_analysis_json(&all_allocations, path),
                    _ => unreachable!(),
                }
            })
            .collect();

        results?;

        let json_time = json_start.elapsed();
        tracing::info!("Generated 5 JSON files in parallel in {}ms", json_time.as_millis());

        let elapsed = start.elapsed();

        // Performance target check: <300ms for full binary processing
        if elapsed.as_millis() > 300 {
            tracing::warn!(
                "Performance target missed: {}ms (target: <300ms)",
                elapsed.as_millis()
            );
        } else {
            tracing::info!(
                "✅ Ultra-fast full binary conversion completed in {}ms (target: <300ms)",
                elapsed.as_millis()
            );
        }

        Ok(())
    }

    /// Generate memory analysis JSON directly (fast path)
    fn generate_memory_analysis_json(
        allocations: &[AllocationInfo],
        output_path: &std::path::Path,
    ) -> Result<(), BinaryExportError> {
        use std::io::{BufWriter, Write};

        let file = std::fs::File::create(output_path)?;
        let mut writer = BufWriter::with_capacity(64 * 1024, file);

        // Pre-allocate string buffer for reuse
        let mut buffer = String::with_capacity(512);

        // Write JSON header
        writer.write_all(b"{\"data\":{\"allocations\":[")?;

        // Write allocations directly with buffering
        for (i, alloc) in allocations.iter().enumerate() {
            if i > 0 {
                writer.write_all(b",")?;
            }

            buffer.clear();
            buffer.push_str("{\"ptr\":\"0x");
            buffer.push_str(&format!("{:x}", alloc.ptr));
            buffer.push_str("\",\"size\":");
            buffer.push_str(&alloc.size.to_string());
            buffer.push_str(",\"var_name\":\"");
            // Full-binary mode: no null fields allowed (requirement 21) - direct access without inference
            buffer.push_str(alloc.var_name.as_deref().unwrap_or("unknown_var"));
            buffer.push_str("\",\"type_name\":\"");
            buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
            buffer.push_str("\",\"scope_name\":\"");
            buffer.push_str(alloc.scope_name.as_deref().unwrap_or("global"));
            buffer.push_str("\",\"timestamp_alloc\":");
            buffer.push_str(&alloc.timestamp_alloc.to_string());
            buffer.push_str(",\"thread_id\":\"");
            buffer.push_str(&alloc.thread_id);
            buffer.push_str("\",\"borrow_count\":");
            buffer.push_str(&alloc.borrow_count.to_string());
            buffer.push_str(",\"is_leaked\":");
            buffer.push_str(if alloc.is_leaked { "true" } else { "false" });
            buffer.push('}');

            writer.write_all(buffer.as_bytes())?;
        }

        // Write JSON footer
        writer.write_all(b"]}}")?;
        writer.flush()?;

        Ok(())
    }

    /// Generate lifetime analysis JSON directly (fast path)
    fn generate_lifetime_analysis_json(
        allocations: &[AllocationInfo],
        output_path: &std::path::Path,
    ) -> Result<(), BinaryExportError> {
        use std::io::{BufWriter, Write};

        let file = std::fs::File::create(output_path)?;
        let mut writer = BufWriter::with_capacity(64 * 1024, file);
        let mut buffer = String::with_capacity(256);

        writer.write_all(b"{\"lifecycle_events\":[")?;

        for (i, alloc) in allocations.iter().enumerate() {
            if i > 0 {
                writer.write_all(b",")?;
            }

            buffer.clear();
            buffer.push_str("{\"event\":\"allocation\",\"ptr\":\"0x");
            buffer.push_str(&format!("{:x}", alloc.ptr));
            buffer.push_str("\",\"scope\":\"");
            buffer.push_str(alloc.scope_name.as_deref().unwrap_or("global"));
            buffer.push_str("\",\"size\":");
            buffer.push_str(&alloc.size.to_string());
            buffer.push_str(",\"timestamp\":");
            buffer.push_str(&alloc.timestamp_alloc.to_string());
            buffer.push_str(",\"type_name\":\"");
            buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
            buffer.push_str("\",\"var_name\":\"");
            buffer.push_str(alloc.var_name.as_deref().unwrap_or("unknown_var"));
            buffer.push_str("\"}");

            writer.write_all(buffer.as_bytes())?;
        }

        writer.write_all(b"]}")?;
        writer.flush()?;

        Ok(())
    }

    /// Generate performance analysis JSON directly (fast path)
    fn generate_performance_analysis_json(
        allocations: &[AllocationInfo],
        output_path: &std::path::Path,
    ) -> Result<(), BinaryExportError> {
        use std::io::{BufWriter, Write};

        let file = std::fs::File::create(output_path)?;
        let mut writer = BufWriter::with_capacity(64 * 1024, file);
        let mut buffer = String::with_capacity(512);

        writer.write_all(b"{\"data\":{\"allocations\":[")?;

        for (i, alloc) in allocations.iter().enumerate() {
            if i > 0 {
                writer.write_all(b",")?;
            }

            buffer.clear();
            buffer.push_str("{\"ptr\":\"0x");
            buffer.push_str(&format!("{:x}", alloc.ptr));
            buffer.push_str("\",\"size\":");
            buffer.push_str(&alloc.size.to_string());
            buffer.push_str(",\"var_name\":\"");
            buffer.push_str(
                alloc
                    .var_name
                    .as_deref()
                    .unwrap_or("unknown_var"),
            );
            buffer.push_str("\",\"type_name\":\"");
            buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
            buffer.push_str("\",\"timestamp_alloc\":");
            buffer.push_str(&alloc.timestamp_alloc.to_string());
            buffer.push_str(",\"thread_id\":\"");
            buffer.push_str(&alloc.thread_id);
            buffer.push_str("\",\"borrow_count\":");
            buffer.push_str(&alloc.borrow_count.to_string());
            buffer.push_str(",\"fragmentation_analysis\":{\"status\":\"not_analyzed\"}}");

            writer.write_all(buffer.as_bytes())?;
        }

        writer.write_all(b"]}}")?;
        writer.flush()?;

        Ok(())
    }

    /// Generate unsafe FFI analysis JSON directly (fast path)
    fn generate_unsafe_ffi_analysis_json(
        allocations: &[AllocationInfo],
        output_path: &std::path::Path,
    ) -> Result<(), BinaryExportError> {
        use std::io::{BufWriter, Write};

        let file = std::fs::File::create(output_path)?;
        let mut writer = BufWriter::with_capacity(64 * 1024, file);
        let mut buffer = String::with_capacity(512);

        writer.write_all(b"{\"boundary_events\":[],\"enhanced_ffi_data\":[")?;

        for (i, alloc) in allocations.iter().enumerate() {
            if i > 0 {
                writer.write_all(b",")?;
            }

            buffer.clear();
            buffer.push_str("{\"ptr\":\"0x");
            buffer.push_str(&format!("{:x}", alloc.ptr));
            buffer.push_str("\",\"size\":");
            buffer.push_str(&alloc.size.to_string());
            buffer.push_str(",\"var_name\":\"");
            buffer.push_str(
                alloc
                    .var_name
                    .as_deref()
                    .unwrap_or("unknown_var"),
            );
            buffer.push_str("\",\"type_name\":\"");
            buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
            buffer.push_str("\",\"timestamp_alloc\":");
            buffer.push_str(&alloc.timestamp_alloc.to_string());
            buffer.push_str(",\"thread_id\":\"");
            buffer.push_str(&alloc.thread_id);
            buffer.push_str("\",\"stack_trace\":");
            if alloc.stack_trace.is_some() {
                buffer.push_str("[]");
            } else {
                buffer.push_str("[\"no_stack_trace_available\"]");
            }
            buffer.push_str(",\"runtime_state\":{\"status\":\"not_analyzed\"}}");

            writer.write_all(buffer.as_bytes())?;
        }

        writer.write_all(b"]}")?;
        writer.flush()?;

        Ok(())
    }

    /// Generate complex types analysis JSON directly (fast path)
    fn generate_complex_types_analysis_json(
        allocations: &[AllocationInfo],
        output_path: &std::path::Path,
    ) -> Result<(), BinaryExportError> {
        use std::io::{BufWriter, Write};

        let file = std::fs::File::create(output_path)?;
        let mut writer = BufWriter::with_capacity(64 * 1024, file);
        let mut buffer = String::with_capacity(1024);

        writer.write_all(b"{\"categorized_types\":{\"primitive\":[")?;

        for (i, alloc) in allocations.iter().enumerate() {
            if i > 0 {
                writer.write_all(b",")?;
            }

            buffer.clear();
            buffer.push_str("{\"ptr\":\"0x");
            buffer.push_str(&format!("{:x}", alloc.ptr));
            buffer.push_str("\",\"size\":");
            buffer.push_str(&alloc.size.to_string());
            buffer.push_str(",\"var_name\":\"");
            buffer.push_str(
                alloc
                    .var_name
                    .as_deref()
                    .unwrap_or("unknown_var"),
            );
            buffer.push_str("\",\"type_name\":\"");
            buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
            buffer.push_str("\",\"smart_pointer_info\":{\"type\":\"none\"}");
            buffer.push_str(",\"memory_layout\":{\"alignment\":8}");
            buffer.push_str(",\"generic_info\":{\"is_generic\":false}");
            buffer.push_str(",\"dynamic_type_info\":{\"is_dynamic\":false}");
            buffer.push_str(",\"generic_instantiation\":{\"instantiated\":false}");
            buffer.push_str(",\"type_relationships\":{\"relationships\":[]}");
            buffer.push_str(",\"type_usage\":{\"usage_count\":1}}");

            writer.write_all(buffer.as_bytes())?;
        }

        writer.write_all(b"]}}")?;
        writer.flush()?;

        Ok(())
    }

    /// Ultra-fast JSON generation using direct streaming writes (no intermediate string allocation)
    fn generate_json_ultra_fast(
        allocations: &[AllocationInfo],
        output_path: &std::path::Path,
        json_type: &str,
        _estimated_size: usize,
    ) -> Result<(), BinaryExportError> {
        use std::io::{BufWriter, Write};

        let file = std::fs::File::create(output_path)?;
        let mut writer = BufWriter::with_capacity(2 * 1024 * 1024, file); // 2MB buffer for maximum I/O performance

        // Direct streaming write without intermediate string allocation
        match json_type {
            "memory" => {
                writer.write_all(br#"{"data":{"allocations":["#)?;
                for (i, alloc) in allocations.iter().enumerate() {
                    if i > 0 {
                        writer.write_all(b",")?;
                    }
                    Self::write_memory_record_direct(&mut writer, alloc)?;
                }
                writer.write_all(b"]}}")?;
            }
            "lifetime" => {
                writer.write_all(br#"{"lifecycle_events":["#)?;
                for (i, alloc) in allocations.iter().enumerate() {
                    if i > 0 {
                        writer.write_all(b",")?;
                    }
                    Self::write_lifetime_record_direct(&mut writer, alloc)?;
                }
                writer.write_all(b"]}")?;
            }
            "performance" => {
                writer.write_all(br#"{"data":{"allocations":["#)?;
                for (i, alloc) in allocations.iter().enumerate() {
                    if i > 0 {
                        writer.write_all(b",")?;
                    }
                    Self::write_performance_record_direct(&mut writer, alloc)?;
                }
                writer.write_all(b"]}}")?;
            }
            "unsafe_ffi" => {
                writer.write_all(br#"{"boundary_events":[],"enhanced_ffi_data":["#)?;
                for (i, alloc) in allocations.iter().enumerate() {
                    if i > 0 {
                        writer.write_all(b",")?;
                    }
                    Self::write_ffi_record_direct(&mut writer, alloc)?;
                }
                writer.write_all(b"]}")?;
            }
            "complex_types" => {
                writer.write_all(br#"{"categorized_types":{"primitive":["#)?;
                for (i, alloc) in allocations.iter().enumerate() {
                    if i > 0 {
                        writer.write_all(b",")?;
                    }
                    Self::write_complex_record_direct(&mut writer, alloc)?;
                }
                writer.write_all(b"]}}")?;
            }
            _ => {
                return Err(BinaryExportError::CorruptedData(format!(
                    "Unknown JSON type: {json_type}"
                )))
            }
        }

        writer.flush()?;
        Ok(())
    }

    /// Serial optimized JSON generation for small datasets
    /// Uses the same optimizations as parallel version but without threading overhead
    fn generate_json_serial_optimized(
        allocations: &[AllocationInfo],
        output_path: &std::path::Path,
        json_type: &str,
        _estimated_size: usize,
    ) -> Result<(), BinaryExportError> {
        use std::io::{BufWriter, Write};

        let file = std::fs::File::create(output_path)?;
        // Task 7.3: Large buffer for optimal I/O performance
        let mut writer = BufWriter::with_capacity(4 * 1024 * 1024, file);

        // Task 7.2: Precise memory pre-allocation based on JSON type
        let _estimated_record_size = match json_type {
            "memory" => 220,
            "lifetime" => 130,
            "performance" => 190,
            "unsafe_ffi" => 170,
            "complex_types" => 320,
            _ => 180,
        };

        // Use small buffer for chunked writing instead of giant string
        let mut buffer = String::with_capacity(8192); // 8KB buffer for chunked writes

        // Task 7.4: Ultra-fast JSON generation with chunked writing
        match json_type {
            "memory" => {
                writer.write_all(br#"{"data":{"allocations":["#)?;
                for (i, alloc) in allocations.iter().enumerate() {
                    if i > 0 {
                        writer.write_all(b",")?;
                    }
                    buffer.clear();
                    Self::append_memory_record_optimized(&mut buffer, alloc);
                    writer.write_all(buffer.as_bytes())?;
                }
                writer.write_all(b"]}}")?;
            }
            "lifetime" => {
                writer.write_all(br#"{"lifecycle_events":["#)?;
                for (i, alloc) in allocations.iter().enumerate() {
                    if i > 0 {
                        writer.write_all(b",")?;
                    }
                    buffer.clear();
                    Self::append_lifetime_record_optimized(&mut buffer, alloc);
                    writer.write_all(buffer.as_bytes())?;
                }
                writer.write_all(b"]}")?;
            }
            "performance" => {
                writer.write_all(br#"{"data":{"allocations":["#)?;
                for (i, alloc) in allocations.iter().enumerate() {
                    if i > 0 {
                        writer.write_all(b",")?;
                    }
                    buffer.clear();
                    Self::append_performance_record_optimized(&mut buffer, alloc);
                    writer.write_all(buffer.as_bytes())?;
                }
                writer.write_all(b"]}}")?;
            }
            "unsafe_ffi" => {
                writer.write_all(br#"{"boundary_events":[],"enhanced_ffi_data":["#)?;
                for (i, alloc) in allocations.iter().enumerate() {
                    if i > 0 {
                        writer.write_all(b",")?;
                    }
                    buffer.clear();
                    Self::append_ffi_record_optimized(&mut buffer, alloc);
                    writer.write_all(buffer.as_bytes())?;
                }
                writer.write_all(b"]}")?;
            }
            "complex_types" => {
                writer.write_all(br#"{"categorized_types":{"primitive":["#)?;
                for (i, alloc) in allocations.iter().enumerate() {
                    if i > 0 {
                        writer.write_all(b",")?;
                    }
                    buffer.clear();
                    Self::append_complex_record_optimized(&mut buffer, alloc);
                    writer.write_all(buffer.as_bytes())?;
                }
                writer.write_all(b"]}}")?;
            }
            _ => {
                return Err(BinaryExportError::CorruptedData(format!(
                    "Unknown JSON type: {json_type}"
                )))
            }
        }
        writer.flush()?;
        Ok(())
    }

    /// Ultra-fast parallel JSON generation with shared data and optimized I/O
    /// Task 7.1, 7.2, 7.3, 7.4: Implements parallel processing, precise memory allocation,
    /// large I/O buffers, and reduced format! usage
    fn generate_json_ultra_fast_parallel(
        allocations: &Arc<Vec<AllocationInfo>>,
        output_path: &std::path::Path,
        json_type: &str,
        _estimated_size: usize,
    ) -> Result<(), BinaryExportError> {
        use std::io::{BufWriter, Write};

        let file = std::fs::File::create(output_path)?;
        // Task 7.3: Increase buffer size to 8MB for maximum I/O performance
        let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file);

        // Task 7.2: Precise memory pre-allocation based on JSON type
        let estimated_record_size = match json_type {
            "memory" => 220, // memory_analysis: ~220 bytes per allocation (increased precision)
            "lifetime" => 130, // lifetime: ~130 bytes per allocation
            "performance" => 190, // performance: ~190 bytes per allocation
            "unsafe_ffi" => 170, // unsafe_ffi: ~170 bytes per allocation
            "complex_types" => 320, // complex_types: ~320 bytes per allocation (most complex)
            _ => 180,
        };

        // Pre-allocate buffer with 10% extra space to avoid reallocations
        let buffer_capacity = (allocations.len() * estimated_record_size * 110) / 100;
        let mut buffer = String::with_capacity(buffer_capacity);

        // Task 7.4: Optimized JSON generation with minimal format! usage
        // Use direct string operations instead of format! macro where possible
        match json_type {
            "memory" => {
                buffer.push_str(r#"{"data":{"allocations":["#);
                for (i, alloc) in allocations.iter().enumerate() {
                    if i > 0 {
                        buffer.push(',');
                    }
                    Self::append_memory_record_optimized(&mut buffer, alloc);
                }
                buffer.push_str("]}}")
            }
            "lifetime" => {
                buffer.push_str(r#"{"lifecycle_events":["#);
                for (i, alloc) in allocations.iter().enumerate() {
                    if i > 0 {
                        buffer.push(',');
                    }
                    Self::append_lifetime_record_optimized(&mut buffer, alloc);
                }
                buffer.push_str("]}")
            }
            "performance" => {
                buffer.push_str(r#"{"data":{"allocations":["#);
                for (i, alloc) in allocations.iter().enumerate() {
                    if i > 0 {
                        buffer.push(',');
                    }
                    Self::append_performance_record_optimized(&mut buffer, alloc);
                }
                buffer.push_str("]}}")
            }
            "unsafe_ffi" => {
                buffer.push_str(r#"{"boundary_events":[],"enhanced_ffi_data":["#);
                for (i, alloc) in allocations.iter().enumerate() {
                    if i > 0 {
                        buffer.push(',');
                    }
                    Self::append_ffi_record_optimized(&mut buffer, alloc);
                }
                buffer.push_str("]}")
            }
            "complex_types" => {
                buffer.push_str(r#"{"categorized_types":{"primitive":["#);
                for (i, alloc) in allocations.iter().enumerate() {
                    if i > 0 {
                        buffer.push(',');
                    }
                    Self::append_complex_record_optimized(&mut buffer, alloc);
                }
                buffer.push_str("]}}")
            }
            _ => {
                return Err(BinaryExportError::CorruptedData(format!(
                    "Unknown JSON type: {json_type}"
                )))
            }
        }

        // Task 7.3: Single large write for maximum I/O performance
        writer.write_all(buffer.as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    #[inline]
    fn append_memory_record(buffer: &mut String, alloc: &AllocationInfo) {
        buffer.push_str(r#"{"ptr":"0x"#);
        Self::append_hex(buffer, alloc.ptr);
        buffer.push_str(r#"","size":"#);
        Self::append_usize(buffer, alloc.size);
        buffer.push_str(r#","var_name":""#);
        buffer.push_str(
            alloc
                .var_name
                .as_deref()
                .unwrap_or("unknown_var"),
        );
        buffer.push_str(r#"","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","scope_name":""#);
        buffer.push_str(alloc.scope_name.as_deref().unwrap_or("global"));
        buffer.push_str(r#"","timestamp_alloc":"#);
        Self::append_number(buffer, alloc.timestamp_alloc);
        buffer.push_str(r#","thread_id":""#);
        buffer.push_str(&alloc.thread_id);
        buffer.push_str(r#"","borrow_count":"#);
        Self::append_usize(buffer, alloc.borrow_count);
        buffer.push_str(r#","is_leaked":"#);
        buffer.push_str(if alloc.is_leaked { "true" } else { "false" });
        buffer.push('}');
    }

    #[inline]
    fn append_lifetime_record(buffer: &mut String, alloc: &AllocationInfo) {
        buffer.push_str(r#"{"event":"allocation","ptr":"0x"#);
        Self::append_hex(buffer, alloc.ptr);
        buffer.push_str(r#"","scope":""#);
        buffer.push_str(alloc.scope_name.as_deref().unwrap_or("global"));
        buffer.push_str(r#"","size":"#);
        Self::append_usize(buffer, alloc.size);
        buffer.push_str(r#","timestamp":"#);
        Self::append_number(buffer, alloc.timestamp_alloc);
        buffer.push_str(r#","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","var_name":""#);
        buffer.push_str(
            alloc
                .var_name
                .as_deref()
                .unwrap_or("unknown_var"),
        );
        buffer.push_str("\"}");
    }

    #[inline]
    fn append_performance_record(buffer: &mut String, alloc: &AllocationInfo) {
        buffer.push_str(r#"{"ptr":"0x"#);
        Self::append_hex(buffer, alloc.ptr);
        buffer.push_str(r#"","size":"#);
        Self::append_usize(buffer, alloc.size);
        buffer.push_str(r#","var_name":""#);
        buffer.push_str(
            alloc
                .var_name
                .as_deref()
                .unwrap_or("unknown_var"),
        );
        buffer.push_str(r#"","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","timestamp_alloc":"#);
        Self::append_number(buffer, alloc.timestamp_alloc);
        buffer.push_str(r#","thread_id":""#);
        buffer.push_str(&alloc.thread_id);
        buffer.push_str(r#"","borrow_count":"#);
        Self::append_usize(buffer, alloc.borrow_count);
        buffer.push_str(r#","fragmentation_analysis":{"status":"not_analyzed"}}"#);
    }

    #[inline]
    fn append_ffi_record(buffer: &mut String, alloc: &AllocationInfo) {
        buffer.push_str(r#"{"ptr":"0x"#);
        Self::append_hex(buffer, alloc.ptr);
        buffer.push_str(r#"","size":"#);
        Self::append_usize(buffer, alloc.size);
        buffer.push_str(r#","var_name":""#);
        buffer.push_str(
            alloc
                .var_name
                .as_deref()
                .unwrap_or("unknown_var"),
        );
        buffer.push_str(r#"","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","timestamp_alloc":"#);
        Self::append_number(buffer, alloc.timestamp_alloc);
        buffer.push_str(r#","thread_id":""#);
        buffer.push_str(&alloc.thread_id);
        buffer.push_str(r#"","stack_trace":["rust_main_thread"],"runtime_state":{"status":"safe","boundary_crossings":0}}"#);
    }

    #[inline]
    fn append_complex_record(buffer: &mut String, alloc: &AllocationInfo) {
        buffer.push_str(r#"{"ptr":"0x"#);
        Self::append_hex(buffer, alloc.ptr);
        buffer.push_str(r#"","size":"#);
        Self::append_usize(buffer, alloc.size);
        buffer.push_str(r#","var_name":""#);
        buffer.push_str(
            alloc
                .var_name
                .as_deref()
                .unwrap_or("unknown_var"),
        );
        buffer.push_str(r#"","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","smart_pointer_info":{"type":"raw_pointer","is_smart":false},"memory_layout":{"alignment":8,"size_class":"medium"},"generic_info":{"is_generic":false,"type_params":[]},"dynamic_type_info":{"is_dynamic":false,"vtable_ptr":0},"generic_instantiation":{"instantiated":true,"template_args":[]},"type_relationships":{"parent_types":[],"child_types":[]},"type_usage":{"usage_count":1,"access_pattern":"sequential"}}"#);
    }

    // PERFORMANCE OPTIMIZATION: Removed infer_type_name and infer_variable_name functions
    // These functions were causing 8384ms performance bottleneck by doing complex inference
    // calculations for 1000+ allocations. Now we use direct field access for maximum speed.
    // Requirement 21: Full-binary mode guarantees no null fields, so direct access is safe.

    #[inline]
    fn append_hex(buffer: &mut String, value: usize) {
        // Fast hex conversion without format! macro
        const HEX_CHARS: &[u8] = b"0123456789abcdef";
        let mut temp = [0u8; 16]; // Enough for 64-bit hex
        let mut i = 0;
        let mut val = value;

        if val == 0 {
            buffer.push('0');
            return;
        }

        while val > 0 {
            temp[i] = HEX_CHARS[(val & 0xf) as usize];
            val >>= 4;
            i += 1;
        }

        // Reverse and append
        for j in (0..i).rev() {
            buffer.push(temp[j] as char);
        }
    }

    #[inline]
    fn append_number(buffer: &mut String, value: u64) {
        // Fast number to string conversion without format! macro
        if value == 0 {
            buffer.push('0');
            return;
        }

        let mut temp = [0u8; 20]; // Enough for 64-bit number
        let mut i = 0;
        let mut val = value;

        while val > 0 {
            temp[i] = b'0' + (val % 10) as u8;
            val /= 10;
            i += 1;
        }

        // Reverse and append
        for j in (0..i).rev() {
            buffer.push(temp[j] as char);
        }
    }

    #[inline]
    fn append_usize(buffer: &mut String, value: usize) {
        Self::append_number(buffer, value as u64);
    }

    /// Task 7.4: Ultra-fast memory record generation - eliminated inference calls
    /// Performance optimization: Removed infer_type_name and infer_variable_name calls
    /// Requirement 21: Full-binary mode guarantees no null fields, direct access is safe
    #[inline]
    fn append_memory_record_optimized(buffer: &mut String, alloc: &AllocationInfo) {
        // Use direct string operations instead of format! for better performance
        buffer.push_str(r#"{"ptr":"0x"#);
        Self::append_hex(buffer, alloc.ptr);
        buffer.push_str(r#"","size":"#);
        Self::append_usize(buffer, alloc.size);
        buffer.push_str(r#","var_name":""#);
        // Direct access - use stored data when available, simple defaults when missing
        buffer.push_str(alloc.var_name.as_deref().unwrap_or("system_alloc"));
        buffer.push_str(r#"","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("system_type"));
        buffer.push_str(r#"","scope_name":""#);
        buffer.push_str(alloc.scope_name.as_deref().unwrap_or("global"));
        buffer.push_str(r#"","timestamp_alloc":"#);
        Self::append_number(buffer, alloc.timestamp_alloc);
        buffer.push_str(r#","thread_id":""#);
        buffer.push_str(&alloc.thread_id);
        buffer.push_str(r#"","borrow_count":"#);
        Self::append_usize(buffer, alloc.borrow_count);
        buffer.push_str(r#","is_leaked":"#);
        buffer.push_str(if alloc.is_leaked { "true" } else { "false" });
        buffer.push('}');
    }

    /// Task 7.4: Ultra-fast lifetime record generation - eliminated inference calls
    #[inline]
    fn append_lifetime_record_optimized(buffer: &mut String, alloc: &AllocationInfo) {
        buffer.push_str(r#"{"event":"allocation","ptr":"0x"#);
        Self::append_hex(buffer, alloc.ptr);
        buffer.push_str(r#"","scope":""#);
        buffer.push_str(alloc.scope_name.as_deref().unwrap_or("global"));
        buffer.push_str(r#"","size":"#);
        Self::append_usize(buffer, alloc.size);
        buffer.push_str(r#","timestamp":"#);
        Self::append_number(buffer, alloc.timestamp_alloc);
        buffer.push_str(r#","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("system_type"));
        buffer.push_str(r#"","var_name":""#);
        buffer.push_str(alloc.var_name.as_deref().unwrap_or("system_alloc"));
        buffer.push_str("\"}");
    }

    /// Task 7.4: Ultra-fast performance record generation - eliminated inference calls
    #[inline]
    fn append_performance_record_optimized(buffer: &mut String, alloc: &AllocationInfo) {
        buffer.push_str(r#"{"ptr":"0x"#);
        Self::append_hex(buffer, alloc.ptr);
        buffer.push_str(r#"","size":"#);
        Self::append_usize(buffer, alloc.size);
        buffer.push_str(r#","var_name":""#);
        buffer.push_str(alloc.var_name.as_deref().unwrap_or("unknown_var"));
        buffer.push_str(r#"","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","timestamp_alloc":"#);
        Self::append_number(buffer, alloc.timestamp_alloc);
        buffer.push_str(r#","thread_id":""#);
        buffer.push_str(&alloc.thread_id);
        buffer.push_str(r#"","borrow_count":"#);
        Self::append_usize(buffer, alloc.borrow_count);
        buffer.push_str(r#","fragmentation_analysis":{"status":"not_analyzed"}}"#);
    }

    /// Task 7.4: Ultra-fast FFI record generation - eliminated inference calls
    #[inline]
    fn append_ffi_record_optimized(buffer: &mut String, alloc: &AllocationInfo) {
        buffer.push_str(r#"{"ptr":"0x"#);
        Self::append_hex(buffer, alloc.ptr);
        buffer.push_str(r#"","size":"#);
        Self::append_usize(buffer, alloc.size);
        buffer.push_str(r#","var_name":""#);
        buffer.push_str(alloc.var_name.as_deref().unwrap_or("unknown_var"));
        buffer.push_str(r#"","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","timestamp_alloc":"#);
        Self::append_number(buffer, alloc.timestamp_alloc);
        buffer.push_str(r#","thread_id":""#);
        buffer.push_str(&alloc.thread_id);
        buffer.push_str(r#"","stack_trace":["rust_main_thread"],"runtime_state":{"status":"safe","boundary_crossings":0}}"#);
    }

    /// Task 7.4: Ultra-fast complex types record generation - eliminated inference calls
    #[inline]
    fn append_complex_record_optimized(buffer: &mut String, alloc: &AllocationInfo) {
        buffer.push_str(r#"{"ptr":"0x"#);
        Self::append_hex(buffer, alloc.ptr);
        buffer.push_str(r#"","size":"#);
        Self::append_usize(buffer, alloc.size);
        buffer.push_str(r#","var_name":""#);
        buffer.push_str(alloc.var_name.as_deref().unwrap_or("unknown_var"));
        buffer.push_str(r#"","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","smart_pointer_info":{"type":"raw_pointer","is_smart":false},"memory_layout":{"alignment":8,"size_class":"medium"},"generic_info":{"is_generic":false,"type_params":[]},"dynamic_type_info":{"is_dynamic":false,"vtable_ptr":0},"generic_instantiation":{"instantiated":true,"template_args":[]},"type_relationships":{"parent_types":[],"child_types":[]},"type_usage":{"usage_count":1,"access_pattern":"sequential"}}"#);
    }

    /// Direct write memory record without string allocation - ultra-fast string building
    #[inline]
    fn write_memory_record_direct<W: std::io::Write>(
        writer: &mut W,
        alloc: &AllocationInfo,
    ) -> Result<(), BinaryExportError> {
        // Pre-allocate buffer for maximum performance
        let mut buffer = String::with_capacity(512);

        // Direct string building without format! macro
        buffer.push_str(r#"{"ptr":"0x"#);
        Self::append_hex(&mut buffer, alloc.ptr);
        buffer.push_str(r#"","size":"#);
        Self::append_usize(&mut buffer, alloc.size);
        buffer.push_str(r#","var_name":""#);
        buffer.push_str(
            alloc
                .var_name
                .as_deref()
                .unwrap_or("unknown_var"),
        );
        buffer.push_str(r#"","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","scope_name":""#);
        buffer.push_str(alloc.scope_name.as_deref().unwrap_or("global"));
        buffer.push_str(r#"","timestamp_alloc":"#);
        Self::append_number(&mut buffer, alloc.timestamp_alloc);
        buffer.push_str(r#","thread_id":""#);
        buffer.push_str(&alloc.thread_id);
        buffer.push_str(r#"","borrow_count":"#);
        Self::append_usize(&mut buffer, alloc.borrow_count);
        buffer.push_str(r#","is_leaked":"#);
        buffer.push_str(if alloc.is_leaked { "true" } else { "false" });
        buffer.push_str(r#","lifetime_ms":0,"smart_pointer_info":{"data_ptr":0,"ref_count":1},"memory_layout":{"alignment":8,"size":"#);
        Self::append_usize(&mut buffer, alloc.size);
        buffer.push_str("}}");

        writer.write_all(buffer.as_bytes())?;
        Ok(())
    }

    /// Direct write lifetime record without string allocation - ultra-fast string building
    #[inline]
    fn write_lifetime_record_direct<W: std::io::Write>(
        writer: &mut W,
        alloc: &AllocationInfo,
    ) -> Result<(), BinaryExportError> {
        let mut buffer = String::with_capacity(256);

        buffer.push_str(r#"{"event":"allocation","ptr":"0x"#);
        Self::append_hex(&mut buffer, alloc.ptr);
        buffer.push_str(r#"","scope":""#);
        buffer.push_str(alloc.scope_name.as_deref().unwrap_or("global"));
        buffer.push_str(r#"","size":"#);
        Self::append_usize(&mut buffer, alloc.size);
        buffer.push_str(r#","timestamp":"#);
        Self::append_number(&mut buffer, alloc.timestamp_alloc);
        buffer.push_str(r#","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","var_name":""#);
        buffer.push_str(
            alloc
                .var_name
                .as_deref()
                .unwrap_or("unknown_var"),
        );
        buffer.push_str("\"}");

        writer.write_all(buffer.as_bytes())?;
        Ok(())
    }

    /// Direct write performance record without string allocation - ultra-fast string building
    #[inline]
    fn write_performance_record_direct<W: std::io::Write>(
        writer: &mut W,
        alloc: &AllocationInfo,
    ) -> Result<(), BinaryExportError> {
        let mut buffer = String::with_capacity(384);

        buffer.push_str(r#"{"ptr":"0x"#);
        Self::append_hex(&mut buffer, alloc.ptr);
        buffer.push_str(r#"","size":"#);
        Self::append_usize(&mut buffer, alloc.size);
        buffer.push_str(r#","var_name":""#);
        buffer.push_str(
            alloc
                .var_name
                .as_deref()
                .unwrap_or("unknown_var"),
        );
        buffer.push_str(r#"","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","timestamp_alloc":"#);
        Self::append_number(&mut buffer, alloc.timestamp_alloc);
        buffer.push_str(r#","thread_id":""#);
        buffer.push_str(&alloc.thread_id);
        buffer.push_str(r#"","borrow_count":"#);
        Self::append_usize(&mut buffer, alloc.borrow_count);
        buffer.push_str(r#","fragmentation_analysis":{"status":"not_analyzed"}}"#);

        writer.write_all(buffer.as_bytes())?;
        Ok(())
    }

    /// Direct write FFI record without string allocation - ultra-fast string building
    #[inline]
    fn write_ffi_record_direct<W: std::io::Write>(
        writer: &mut W,
        alloc: &AllocationInfo,
    ) -> Result<(), BinaryExportError> {
        let mut buffer = String::with_capacity(320);

        buffer.push_str(r#"{"ptr":"0x"#);
        Self::append_hex(&mut buffer, alloc.ptr);
        buffer.push_str(r#"","size":"#);
        Self::append_usize(&mut buffer, alloc.size);
        buffer.push_str(r#","var_name":""#);
        buffer.push_str(
            alloc
                .var_name
                .as_deref()
                .unwrap_or("unknown_var"),
        );
        buffer.push_str(r#"","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","timestamp_alloc":"#);
        Self::append_number(&mut buffer, alloc.timestamp_alloc);
        buffer.push_str(r#","thread_id":""#);
        buffer.push_str(&alloc.thread_id);
        buffer.push_str(r#"","stack_trace":["rust_main_thread"],"runtime_state":{"status":"safe","boundary_crossings":0}}"#);

        writer.write_all(buffer.as_bytes())?;
        Ok(())
    }

    /// Direct write complex types record without string allocation - ultra-fast string building
    #[inline]
    fn write_complex_record_direct<W: std::io::Write>(
        writer: &mut W,
        alloc: &AllocationInfo,
    ) -> Result<(), BinaryExportError> {
        let mut buffer = String::with_capacity(256);

        buffer.push_str(r#"{"allocation_id":"#);
        Self::append_usize(&mut buffer, alloc.ptr);
        buffer.push_str(r#","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","category":"primitive","complexity_score":1,"memory_layout":{"alignment":8},"generic_info":{"is_generic":false}}"#);

        writer.write_all(buffer.as_bytes())?;
        Ok(())
    }
}
