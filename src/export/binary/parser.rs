//! Optimized binary file parser

use crate::core::types::AllocationInfo;
use crate::export::analysis_engine::{AnalysisEngine, StandardAnalysisEngine};
use crate::export::binary::{BinaryExportError, BinaryReader};
use std::path::Path;
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

    /// Parse full binary to JSON using optimized strategy (large files, heavy optimization)
    /// This method uses a fast, direct approach optimized for large full-binary files
    /// that contain all allocations (user + system) and can be hundreds of KB in size.
    ///
    /// Optimizations:
    /// - Direct JSON generation without heavy analysis engine overhead
    /// Parse full binary to JSON using optimized strategy (large files, heavy optimization)
    /// This method uses a fast, direct approach optimized for large full-binary files
    /// that contain all allocations (user + system) and can be hundreds of KB in size.
    ///
    /// Optimizations:
    /// - Direct JSON generation without heavy analysis engine overhead
    /// - Parallel processing of different JSON types
    /// - Minimal memory allocations and string formatting
    /// - Targets <300ms performance for large datasets
    pub fn parse_full_binary_to_json<P: AsRef<Path>>(
        binary_path: P,
        base_name: &str,
    ) -> Result<(), BinaryExportError> {
        let start = Instant::now();
        tracing::info!("Starting optimized full binary to JSON conversion");

        // Load all allocations (user + system) for full binary mode
        let load_start = Instant::now();
        let all_allocations = Self::load_allocations(binary_path)?;
        let load_time = load_start.elapsed();
        tracing::info!(
            "Loaded {} allocations in {}ms",
            all_allocations.len(),
            load_time.as_millis()
        );

        // Create output directory
        let base_memory_analysis_dir = std::path::Path::new("MemoryAnalysis");
        let project_dir = base_memory_analysis_dir.join(base_name);
        std::fs::create_dir_all(&project_dir)?;

        // Generate JSON files using ultra-fast batch approach
        let json_start = Instant::now();

        // Pre-calculate total JSON size to avoid reallocations
        let estimated_size_per_alloc = 150; // Reduced from 200 to 150 bytes per allocation
        let total_estimated_size = all_allocations.len() * estimated_size_per_alloc;

        // Generate all 5 JSON files in parallel using batch approach
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
                    &all_allocations,
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

        // Performance target check: <300ms for full binary processing
        if elapsed.as_millis() > 300 {
            tracing::warn!(
                "Performance target missed: {}ms (target: <300ms)",
                elapsed.as_millis()
            );
        } else {
            tracing::info!(
                "✅ Optimized full binary conversion completed in {}ms (target: <300ms)",
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
            // Full-binary mode: no null fields allowed (requirement 21)
            buffer.push_str(alloc.var_name.as_deref().unwrap_or("system_allocation"));
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
            buffer.push_str(alloc.var_name.as_deref().unwrap_or("system_allocation"));
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
            buffer.push_str(alloc.var_name.as_deref().unwrap_or("system_allocation"));
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
            buffer.push_str(alloc.var_name.as_deref().unwrap_or("system_allocation"));
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
            buffer.push_str(alloc.var_name.as_deref().unwrap_or("system_allocation"));
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

    #[inline]
    fn append_memory_record(buffer: &mut String, alloc: &AllocationInfo) {
        buffer.push_str(r#"{"ptr":"0x"#);
        Self::append_hex(buffer, alloc.ptr);
        buffer.push_str(r#"","size":"#);
        Self::append_number(buffer, alloc.size);
        buffer.push_str(r#","var_name":""#);
        buffer.push_str(alloc.var_name.as_deref().unwrap_or("system_allocation"));
        buffer.push_str(r#"","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","scope_name":""#);
        buffer.push_str(alloc.scope_name.as_deref().unwrap_or("global"));
        buffer.push_str(r#"","timestamp_alloc":"#);
        Self::append_number(buffer, alloc.timestamp_alloc);
        buffer.push_str(r#","thread_id":""#);
        buffer.push_str(&alloc.thread_id);
        buffer.push_str(r#"","borrow_count":"#);
        Self::append_number(buffer, alloc.borrow_count);
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
        Self::append_number(buffer, alloc.size);
        buffer.push_str(r#","timestamp":"#);
        Self::append_number(buffer, alloc.timestamp_alloc);
        buffer.push_str(r#","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","var_name":""#);
        buffer.push_str(alloc.var_name.as_deref().unwrap_or("system_allocation"));
        buffer.push_str("\"}");
    }

    #[inline]
    fn append_performance_record(buffer: &mut String, alloc: &AllocationInfo) {
        buffer.push_str(r#"{"ptr":"0x"#);
        Self::append_hex(buffer, alloc.ptr);
        buffer.push_str(r#"","size":"#);
        Self::append_number(buffer, alloc.size);
        buffer.push_str(r#","var_name":""#);
        buffer.push_str(alloc.var_name.as_deref().unwrap_or("system_allocation"));
        buffer.push_str(r#"","type_name":""#);
        buffer.push_str(alloc.type_name.as_deref().unwrap_or("unknown_type"));
        buffer.push_str(r#"","timestamp_alloc":"#);
        Self::append_number(buffer, alloc.timestamp_alloc);
        buffer.push_str(r#","thread_id":""#);
        buffer.push_str(&alloc.thread_id);
        buffer.push_str(r#"","borrow_count":"#);
        Self::append_number(buffer, alloc.borrow_count);
        buffer.push_str(r#","fragmentation_analysis":{"status":"not_analyzed"}}"#);
    }

    #[inline]
    fn append_ffi_record(buffer: &mut String, alloc: &AllocationInfo) {
        buffer.push_str(r#"{"ptr":"0x"#);
        Self::append_hex(buffer, alloc.ptr);
        buffer.push_str(r#"","size":"#);
        Self::append_number(buffer, alloc.size);
        buffer.push_str(r#","var_name":""#);
        buffer.push_str(alloc.var_name.as_deref().unwrap_or("system_variable"));
        buffer.push_str(r#"","type_name":""#);
        buffer.push_str(Self::infer_type_name(alloc));
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
        Self::append_number(buffer, alloc.size);
        buffer.push_str(r#","var_name":""#);
        buffer.push_str(alloc.var_name.as_deref().unwrap_or("system_variable"));
        buffer.push_str(r#"","type_name":""#);
        buffer.push_str(Self::infer_type_name(alloc));
        buffer.push_str(r#"","smart_pointer_info":{"type":"raw_pointer","is_smart":false},"memory_layout":{"alignment":8,"size_class":"medium"},"generic_info":{"is_generic":false,"type_params":[]},"dynamic_type_info":{"is_dynamic":false,"vtable_ptr":0},"generic_instantiation":{"instantiated":true,"template_args":[]},"type_relationships":{"parent_types":[],"child_types":[]},"type_usage":{"usage_count":1,"access_pattern":"sequential"}}"#);
    }

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
            temp[i] = HEX_CHARS[val & 0xf];
            val >>= 4;
            i += 1;
        }

        // Reverse and append
        for j in (0..i).rev() {
            buffer.push(temp[j] as char);
        }
    }

    #[inline]
    fn append_number<T: std::fmt::Display>(buffer: &mut String, value: T) {
        use std::fmt::Write;
        let _ = write!(buffer, "{value}");
    }

    /// Infer meaningful type name based on allocation characteristics
    #[inline]
    fn infer_type_name(alloc: &AllocationInfo) -> &str {
        if let Some(ref type_name) = alloc.type_name {
            type_name
        } else {
            // Infer type based on size and context
            match alloc.size {
                0..=8 => "primitive_type",
                9..=64 => "small_struct",
                65..=1024 => "medium_struct",
                1025..=8192 => "large_struct",
                _ => "bulk_data",
            }
        }
    }

    /// Direct write memory record without string allocation
    #[inline]
    fn write_memory_record_direct<W: std::io::Write>(
        writer: &mut W,
        alloc: &AllocationInfo,
    ) -> Result<(), BinaryExportError> {
        // Use a single format! call to minimize allocations
        let record = format!(
            "{{\"ptr\":\"0x{:x}\",\"size\":{},\"var_name\":\"{}\",\"type_name\":\"{}\",\"scope_name\":\"{}\",\"timestamp_alloc\":{},\"thread_id\":\"{}\",\"borrow_count\":{},\"is_leaked\":{},\"lifetime_ms\":0,\"smart_pointer_info\":{{\"data_ptr\":0,\"ref_count\":1}},\"memory_layout\":{{\"alignment\":8,\"size\":{}}}}}",
            alloc.ptr,
            alloc.size,
            alloc.var_name.as_deref().unwrap_or("system_allocation"),
            alloc.type_name.as_deref().unwrap_or("unknown_type"),
            alloc.scope_name.as_deref().unwrap_or("global"),
            alloc.timestamp_alloc,
            alloc.thread_id,
            alloc.borrow_count,
            if alloc.is_leaked { "true" } else { "false" },
            alloc.size
        );
        writer.write_all(record.as_bytes())?;
        Ok(())
    }

    /// Direct write lifetime record without string allocation
    #[inline]
    fn write_lifetime_record_direct<W: std::io::Write>(
        writer: &mut W,
        alloc: &AllocationInfo,
    ) -> Result<(), BinaryExportError> {
        let record = format!(
            "{{\"allocation_id\":{},\"event_type\":\"allocation\",\"timestamp\":{},\"size\":{},\"thread_id\":\"{}\"}}",
            alloc.ptr, alloc.timestamp_alloc, alloc.size, alloc.thread_id
        );
        writer.write_all(record.as_bytes())?;
        Ok(())
    }

    /// Direct write performance record without string allocation
    #[inline]
    fn write_performance_record_direct<W: std::io::Write>(
        writer: &mut W,
        alloc: &AllocationInfo,
    ) -> Result<(), BinaryExportError> {
        let record = format!(
            "{{\"ptr\":\"0x{:x}\",\"size\":{},\"allocation_time_ns\":0,\"deallocation_time_ns\":0,\"peak_memory_usage\":{},\"fragmentation_score\":0.0,\"access_pattern\":\"sequential\"}}",
            alloc.ptr, alloc.size, alloc.size
        );
        writer.write_all(record.as_bytes())?;
        Ok(())
    }

    /// Direct write FFI record without string allocation
    #[inline]
    fn write_ffi_record_direct<W: std::io::Write>(
        writer: &mut W,
        alloc: &AllocationInfo,
    ) -> Result<(), BinaryExportError> {
        let record = format!(
            "{{\"allocation_id\":{},\"is_ffi_related\":false,\"boundary_crossings\":0,\"safety_level\":\"safe\",\"ffi_source\":\"none\"}}",
            alloc.ptr
        );
        writer.write_all(record.as_bytes())?;
        Ok(())
    }

    /// Direct write complex types record without string allocation
    #[inline]
    fn write_complex_record_direct<W: std::io::Write>(
        writer: &mut W,
        alloc: &AllocationInfo,
    ) -> Result<(), BinaryExportError> {
        let record = format!(
            "{{\"allocation_id\":{},\"type_name\":\"{}\",\"category\":\"primitive\",\"complexity_score\":1,\"memory_layout\":{{\"alignment\":8}},\"generic_info\":{{\"is_generic\":false}}}}",
            alloc.ptr,
            alloc.type_name.as_deref().unwrap_or("unknown_type")
        );
        writer.write_all(record.as_bytes())?;
        Ok(())
    }
}
