//! Optimized binary file parser

use crate::core::types::AllocationInfo;
use crate::export::binary::{BinaryExportError, BinaryReader};
use std::path::Path;
use std::time::Instant;

/// Binary parser for optimized file conversion
pub struct BinaryParser;

impl BinaryParser {
    /// Convert binary file to standard JSON files using simple optimized approach
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

        // Load allocations - only user-defined variables
        let allocations = Self::load_allocations(binary_path)?;
        let user_allocations: Vec<_> = allocations.iter()
            .filter(|a| a.var_name.is_some())
            .collect();

        // Generate 5 JSON files with minimal data
        Self::generate_memory_analysis(&project_dir, base_name, &user_allocations)?;
        Self::generate_lifetime_analysis(&project_dir, base_name, &user_allocations)?;
        Self::generate_performance_analysis(&project_dir, base_name, &user_allocations)?;
        Self::generate_unsafe_ffi_analysis(&project_dir, base_name, &user_allocations)?;
        Self::generate_complex_types_analysis(&project_dir, base_name, &user_allocations)?;

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 300 {
            eprintln!("⚠️  Performance target missed: {}ms (target: <300ms)", elapsed.as_millis());
        } else {
            println!("✅ Optimized conversion completed in {}ms", elapsed.as_millis());
        }
        
        Ok(())
    }

    /// Load allocations from binary file
    pub fn load_allocations<P: AsRef<Path>>(binary_path: P) -> Result<Vec<AllocationInfo>, BinaryExportError> {
        let mut reader = BinaryReader::new(binary_path)?;
        reader.read_all()
    }

    /// Simple binary to JSON conversion - one method does it all
    pub fn binary_to_json<P: AsRef<Path>>(binary_path: P, output_dir: P) -> Result<(), BinaryExportError> {
        let binary_path = binary_path.as_ref();
        let output_dir = output_dir.as_ref();
        
        // Create output directory
        std::fs::create_dir_all(output_dir)?;
        
        // Load allocations - only user variables
        let allocations = Self::load_allocations(binary_path)?;
        let user_allocations: Vec<_> = allocations.iter()
            .filter(|a| a.var_name.is_some())
            .collect();

        println!("🔄 Converting {} user variables from binary to JSON", user_allocations.len());

        // Generate 5 JSON files directly
        let json_files = [
            ("memory_analysis.json", &user_allocations[..]),
            ("lifetime.json", &user_allocations[..]),
            ("performance.json", &user_allocations.iter().filter(|a| a.size >= 64).cloned().collect::<Vec<_>>()[..]),
            ("unsafe_ffi.json", &user_allocations[..]),
            ("complex_types.json", &user_allocations.iter().filter(|a| a.type_name.is_some()).cloned().collect::<Vec<_>>()[..]),
        ];

        for (filename, data) in json_files {
            let file_path = output_dir.join(filename);
            
            let json_data = serde_json::json!({
                "allocations": data.iter().map(|a| serde_json::json!({
                    "ptr": format!("0x{:x}", a.ptr),
                    "size": a.size,
                    "var_name": a.var_name.as_deref().unwrap_or("unknown"),
                    "type_name": a.type_name.as_deref().unwrap_or("unknown"),
                    "thread_id": a.thread_id,
                    "timestamp_alloc": a.timestamp_alloc,
                    "is_leaked": a.is_leaked
                })).collect::<Vec<_>>()
            });
            
            let json_content = serde_json::to_string(&json_data)
                .map_err(|e| BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e)))?;
            
            std::fs::write(&file_path, json_content)?;
            
            let size = std::fs::metadata(&file_path)?.len();
            println!("  ✅ {} ({:.1}KB)", filename, size as f64 / 1024.0);
        }

        Ok(())
    }

    /// Convert binary file to single JSON format (legacy compatibility)
    pub fn to_json<P: AsRef<Path>>(binary_path: P, json_path: P) -> Result<(), BinaryExportError> {
        let allocations = Self::load_allocations(binary_path)?;
        let json_data = serde_json::to_string_pretty(&allocations)
            .map_err(|e| BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e)))?;
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
            serde_json::to_string_pretty(&allocations)
                .map_err(|e| BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e)))?
        );
        std::fs::write(html_path, html_content)?;
        Ok(())
    }

    fn generate_memory_analysis<P: AsRef<Path>>(
        project_dir: P,
        base_name: &str,
        allocations: &[&AllocationInfo],
    ) -> Result<(), BinaryExportError> {
        let file_path = project_dir.as_ref().join(format!("snapshot_memory_analysis.json"));
        
        let json_data = serde_json::json!({
            "allocations": allocations.iter().map(|a| serde_json::json!({
                "ptr": format!("0x{:x}", a.ptr),
                "size": a.size,
                "var_name": a.var_name.as_deref().unwrap_or("unknown"),
                "type_name": a.type_name.as_deref().unwrap_or("unknown"),
                "thread_id": a.thread_id,
                "timestamp_alloc": a.timestamp_alloc,
                "is_leaked": a.is_leaked,
                "borrow_count": a.borrow_count
            })).collect::<Vec<_>>()
        });
        
        let json_content = serde_json::to_string(&json_data)
            .map_err(|e| BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e)))?;
        
        std::fs::write(file_path, json_content)?;
        Ok(())
    }

    fn generate_lifetime_analysis<P: AsRef<Path>>(
        project_dir: P,
        base_name: &str,
        allocations: &[&AllocationInfo],
    ) -> Result<(), BinaryExportError> {
        let file_path = project_dir.as_ref().join(format!("snapshot_lifetime.json"));
        
        // Include all user allocations for lifetime analysis (not just deallocated ones)
        let lifetime_allocations: Vec<_> = allocations.iter().collect();
        
        let json_data = serde_json::json!({
            "allocations": lifetime_allocations.iter().map(|a| serde_json::json!({
                "ptr": format!("0x{:x}", a.ptr),
                "var_name": a.var_name.as_deref().unwrap_or("unknown"),
                "timestamp_alloc": a.timestamp_alloc,
                "timestamp_dealloc": a.timestamp_dealloc,
                "lifetime_ms": a.lifetime_ms,
                "scope_name": a.scope_name.as_deref().unwrap_or("global")
            })).collect::<Vec<_>>()
        });
        
        let json_content = serde_json::to_string(&json_data)
            .map_err(|e| BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e)))?;
        
        std::fs::write(file_path, json_content)?;
        Ok(())
    }

    fn generate_performance_analysis<P: AsRef<Path>>(
        project_dir: P,
        base_name: &str,
        allocations: &[&AllocationInfo],
    ) -> Result<(), BinaryExportError> {
        let file_path = project_dir.as_ref().join(format!("snapshot_performance.json"));
        
        // Include allocations >= 64 bytes (more reasonable threshold for user variables)
        let large_allocations: Vec<_> = allocations.iter()
            .filter(|a| a.size >= 64)
            .collect();
        
        let json_data = serde_json::json!({
            "allocations": large_allocations.iter().map(|a| serde_json::json!({
                "ptr": format!("0x{:x}", a.ptr),
                "size": a.size,
                "timestamp_alloc": a.timestamp_alloc,
                "thread_id": a.thread_id,
                "borrow_count": a.borrow_count
            })).collect::<Vec<_>>()
        });
        
        let json_content = serde_json::to_string(&json_data)
            .map_err(|e| BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e)))?;
        
        std::fs::write(file_path, json_content)?;
        Ok(())
    }

    fn generate_unsafe_ffi_analysis<P: AsRef<Path>>(
        project_dir: P,
        base_name: &str,
        allocations: &[&AllocationInfo],
    ) -> Result<(), BinaryExportError> {
        let file_path = project_dir.as_ref().join(format!("snapshot_unsafe_ffi.json"));
        
        // Include all user allocations for FFI analysis (stack trace is optional)
        let ffi_allocations: Vec<_> = allocations.iter().collect();
        
        let json_data = serde_json::json!({
            "allocations": ffi_allocations.iter().map(|a| serde_json::json!({
                "ptr": format!("0x{:x}", a.ptr),
                "var_name": a.var_name.as_deref().unwrap_or("unknown"),
                "type_name": a.type_name.as_deref().unwrap_or("unknown"),
                "thread_id": a.thread_id,
                "stack_trace": a.stack_trace
            })).collect::<Vec<_>>()
        });
        
        let json_content = serde_json::to_string(&json_data)
            .map_err(|e| BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e)))?;
        
        std::fs::write(file_path, json_content)?;
        Ok(())
    }

    fn generate_complex_types_analysis<P: AsRef<Path>>(
        project_dir: P,
        base_name: &str,
        allocations: &[&AllocationInfo],
    ) -> Result<(), BinaryExportError> {
        let file_path = project_dir.as_ref().join(format!("snapshot_complex_types.json"));
        
        // Include all user allocations with type names (no size restriction)
        let complex_allocations: Vec<_> = allocations.iter()
            .filter(|a| a.type_name.is_some())
            .collect();
        
        let json_data = serde_json::json!({
            "allocations": complex_allocations.iter().map(|a| serde_json::json!({
                "ptr": format!("0x{:x}", a.ptr),
                "size": a.size,
                "var_name": a.var_name.as_deref().unwrap_or("unknown"),
                "type_name": a.type_name.as_deref().unwrap_or("unknown")
            })).collect::<Vec<_>>()
        });
        
        let json_content = serde_json::to_string(&json_data)
            .map_err(|e| BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e)))?;
        
        std::fs::write(file_path, json_content)?;
        Ok(())
    }
}