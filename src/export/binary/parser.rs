//! Binary file parser and converter for transforming binary data to other formats

use crate::core::types::AllocationInfo;
use crate::export::binary::error::BinaryExportError;
use crate::export::binary::reader::BinaryReader;
use std::path::Path;

/// Binary parser for file conversion and data loading
pub struct BinaryParser;

impl BinaryParser {
    /// Convert binary file to standard JSON files (5 categorized files)
    pub fn to_standard_json_files<P: AsRef<Path>>(
        binary_path: P, 
        base_name: &str
    ) -> Result<(), BinaryExportError> {
        let allocations = Self::load_allocations(binary_path)?;
        
        // Create output directory structure matching existing system
        let base_memory_analysis_dir = std::path::Path::new("MemoryAnalysis");
        let project_dir = base_memory_analysis_dir.join(base_name);
        std::fs::create_dir_all(&project_dir)?;
        
        // Generate the 5 standard JSON files
        let files_to_generate = [
            ("memory_analysis", Self::generate_memory_analysis_json(&allocations)?),
            ("lifetime", Self::generate_lifetime_json(&allocations)?),
            ("performance", Self::generate_performance_json(&allocations)?),
            ("unsafe_ffi", Self::generate_unsafe_ffi_json(&allocations)?),
            ("complex_types", Self::generate_complex_types_json(&allocations)?),
        ];
        
        for (file_type, json_content) in files_to_generate {
            let file_path = project_dir.join(format!("{}_{}.json", base_name, file_type));
            std::fs::write(file_path, json_content)?;
        }
        
        Ok(())
    }
    
    /// Convert binary file to single JSON format (legacy compatibility)
    pub fn to_json<P: AsRef<Path>>(
        binary_path: P, 
        json_path: P
    ) -> Result<(), BinaryExportError> {
        let allocations = Self::load_allocations(binary_path)?;
        
        // Serialize to JSON using serde
        let json_data = serde_json::to_string_pretty(&allocations)
            .map_err(|e| BinaryExportError::CorruptedData(format!("JSON serialization failed: {}", e)))?;
        
        std::fs::write(json_path, json_data)?;
        Ok(())
    }
    
    /// Convert binary file to HTML format
    pub fn to_html<P: AsRef<Path>>(
        binary_path: P, 
        html_path: P
    ) -> Result<(), BinaryExportError> {
        let allocations = Self::load_allocations(binary_path)?;
        
        // Generate HTML report using existing template
        let html_content = Self::generate_html_report(&allocations)?;
        
        std::fs::write(html_path, html_content)?;
        Ok(())
    }
    
    /// Load allocation data from binary file
    pub fn load_allocations<P: AsRef<Path>>(
        binary_path: P
    ) -> Result<Vec<AllocationInfo>, BinaryExportError> {
        let mut reader = BinaryReader::new(binary_path)?;
        let allocations = reader.read_all()?;
        
        // Validate data integrity
        Self::validate_allocations(&allocations)?;
        
        Ok(allocations)
    }
    
    /// Generate HTML report using existing template
    fn generate_html_report(allocations: &[AllocationInfo]) -> Result<String, BinaryExportError> {
        // Read the existing HTML template
        let template_content = match std::fs::read_to_string("templates/dashboard.html") {
            Ok(content) => content,
            Err(_) => {
                // Fallback to simple HTML if template not found
                return Self::generate_simple_html_report(allocations);
            }
        };
        
        // Read CSS and JS files
        let css_content = std::fs::read_to_string("templates/styles.css").unwrap_or_default();
        let js_content = std::fs::read_to_string("templates/script.js").unwrap_or_default();
        
        // Prepare data for template
        let json_data = serde_json::to_string(allocations)
            .map_err(|e| BinaryExportError::CorruptedData(format!("JSON serialization failed: {}", e)))?;
        
        // Replace template placeholders
        let html = template_content
            .replace("{{ json_data }}", &json_data)
            .replace("{{CSS_CONTENT}}", &css_content)
            .replace("{{JS_CONTENT}}", &js_content);
        
        Ok(html)
    }
    
    /// Generate simple HTML report as fallback
    fn generate_simple_html_report(allocations: &[AllocationInfo]) -> Result<String, BinaryExportError> {
        let mut html = String::new();
        
        // HTML header with basic styling
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\"><head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str("<title>Memory Analysis Report</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }\n");
        html.push_str(".container { max-width: 1200px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }\n");
        html.push_str("h1 { color: #333; border-bottom: 2px solid #007acc; padding-bottom: 10px; }\n");
        html.push_str("h2 { color: #555; margin-top: 30px; }\n");
        html.push_str(".summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin: 20px 0; }\n");
        html.push_str(".stat-card { background: #007acc; color: white; padding: 20px; border-radius: 6px; text-align: center; }\n");
        html.push_str(".stat-value { font-size: 2em; font-weight: bold; margin: 10px 0; }\n");
        html.push_str("table { width: 100%; border-collapse: collapse; margin-top: 20px; }\n");
        html.push_str("th, td { padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }\n");
        html.push_str("th { background-color: #f8f9fa; font-weight: bold; }\n");
        html.push_str("tr:hover { background-color: #f5f5f5; }\n");
        html.push_str(".address { font-family: monospace; color: #666; }\n");
        html.push_str(".size { text-align: right; }\n");
        html.push_str("</style>\n");
        html.push_str("</head><body>\n");
        html.push_str("<div class=\"container\">\n");
        
        // Header
        html.push_str("<h1>🔍 Memory Analysis Report</h1>\n");
        html.push_str("<p>Generated from binary export data</p>\n");
        
        // Summary statistics
        let total_size: usize = allocations.iter().map(|a| a.size).sum();
        let active_count = allocations.len();
        let avg_size = if active_count > 0 { total_size / active_count } else { 0 };
        
        html.push_str("<h2>📊 Summary</h2>\n");
        html.push_str("<div class=\"summary\">\n");
        html.push_str(&format!("<div class=\"stat-card\"><div>Total Allocations</div><div class=\"stat-value\">{}</div></div>\n", active_count));
        html.push_str(&format!("<div class=\"stat-card\"><div>Total Memory</div><div class=\"stat-value\">{} bytes</div></div>\n", total_size));
        html.push_str(&format!("<div class=\"stat-card\"><div>Average Size</div><div class=\"stat-value\">{} bytes</div></div>\n", avg_size));
        html.push_str("</div>\n");
        
        // Allocation table
        html.push_str("<h2>📋 Memory Allocations</h2>\n");
        html.push_str("<table>\n");
        html.push_str("<thead><tr><th>Address</th><th>Size</th><th>Variable</th><th>Type</th><th>Thread</th><th>Timestamp</th></tr></thead>\n");
        html.push_str("<tbody>\n");
        
        for alloc in allocations {
            html.push_str("<tr>");
            html.push_str(&format!("<td class=\"address\">0x{:x}</td>", alloc.ptr));
            html.push_str(&format!("<td class=\"size\">{}</td>", alloc.size));
            html.push_str(&format!("<td>{}</td>", 
                alloc.var_name.as_deref().unwrap_or("N/A")));
            html.push_str(&format!("<td>{}</td>", 
                alloc.type_name.as_deref().unwrap_or("N/A")));
            html.push_str(&format!("<td>{}</td>", alloc.thread_id));
            html.push_str(&format!("<td>{}</td>", alloc.timestamp_alloc));
            html.push_str("</tr>\n");
        }
        
        html.push_str("</tbody></table>\n");
        
        // Footer
        html.push_str("<div style=\"margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; text-align: center; color: #666;\">\n");
        html.push_str("<p>Generated by MemScope-rs Binary Export Module</p>\n");
        html.push_str("</div>\n");
        
        html.push_str("</div></body></html>\n");
        
        Ok(html)
    }
    
    /// Generate memory analysis JSON data
    fn generate_memory_analysis_json(allocations: &[AllocationInfo]) -> Result<String, BinaryExportError> {
        // Focus on core memory allocation data
        let memory_data: Vec<serde_json::Value> = allocations.iter().map(|alloc| {
            serde_json::json!({
                "ptr": format!("0x{:x}", alloc.ptr),
                "size": alloc.size,
                "var_name": alloc.var_name.as_deref().unwrap_or("N/A"),
                "type_name": alloc.type_name.as_deref().unwrap_or("N/A"),
                "thread_id": alloc.thread_id,
                "timestamp_alloc": alloc.timestamp_alloc,
                "is_leaked": alloc.is_leaked,
                "borrow_count": alloc.borrow_count
            })
        }).collect();
        
        let result = serde_json::json!({
            "memory_analysis": {
                "total_allocations": allocations.len(),
                "total_memory": allocations.iter().map(|a| a.size).sum::<usize>(),
                "allocations": memory_data
            }
        });
        
        serde_json::to_string_pretty(&result)
            .map_err(|e| BinaryExportError::CorruptedData(format!("Memory analysis JSON serialization failed: {}", e)))
    }
    
    /// Generate lifetime analysis JSON data
    fn generate_lifetime_json(allocations: &[AllocationInfo]) -> Result<String, BinaryExportError> {
        // Focus on lifetime and temporal data
        let lifetime_data: Vec<serde_json::Value> = allocations.iter().map(|alloc| {
            serde_json::json!({
                "ptr": format!("0x{:x}", alloc.ptr),
                "timestamp_alloc": alloc.timestamp_alloc,
                "timestamp_dealloc": alloc.timestamp_dealloc,
                "lifetime_ms": alloc.lifetime_ms,
                "is_leaked": alloc.is_leaked,
                "var_name": alloc.var_name.as_deref().unwrap_or("N/A"),
                "scope_name": alloc.scope_name.as_deref().unwrap_or("N/A")
            })
        }).collect();
        
        let result = serde_json::json!({
            "lifetime_analysis": {
                "total_allocations": allocations.len(),
                "leaked_count": allocations.iter().filter(|a| a.is_leaked).count(),
                "allocations": lifetime_data
            }
        });
        
        serde_json::to_string_pretty(&result)
            .map_err(|e| BinaryExportError::CorruptedData(format!("Lifetime JSON serialization failed: {}", e)))
    }
    
    /// Generate performance analysis JSON data
    fn generate_performance_json(allocations: &[AllocationInfo]) -> Result<String, BinaryExportError> {
        // Focus on performance metrics
        let total_size: usize = allocations.iter().map(|a| a.size).sum();
        let avg_size = if !allocations.is_empty() { total_size / allocations.len() } else { 0 };
        let max_size = allocations.iter().map(|a| a.size).max().unwrap_or(0);
        let min_size = allocations.iter().map(|a| a.size).min().unwrap_or(0);
        
        let performance_data: Vec<serde_json::Value> = allocations.iter().map(|alloc| {
            serde_json::json!({
                "ptr": format!("0x{:x}", alloc.ptr),
                "size": alloc.size,
                "timestamp_alloc": alloc.timestamp_alloc,
                "thread_id": alloc.thread_id,
                "borrow_count": alloc.borrow_count,
                "fragmentation_analysis": alloc.fragmentation_analysis
            })
        }).collect();
        
        let result = serde_json::json!({
            "performance_analysis": {
                "summary": {
                    "total_allocations": allocations.len(),
                    "total_memory": total_size,
                    "average_size": avg_size,
                    "max_size": max_size,
                    "min_size": min_size
                },
                "allocations": performance_data
            }
        });
        
        serde_json::to_string_pretty(&result)
            .map_err(|e| BinaryExportError::CorruptedData(format!("Performance JSON serialization failed: {}", e)))
    }
    
    /// Generate unsafe FFI analysis JSON data
    fn generate_unsafe_ffi_json(allocations: &[AllocationInfo]) -> Result<String, BinaryExportError> {
        // Focus on unsafe operations and FFI-related data
        let unsafe_data: Vec<serde_json::Value> = allocations.iter().map(|alloc| {
            serde_json::json!({
                "ptr": format!("0x{:x}", alloc.ptr),
                "size": alloc.size,
                "var_name": alloc.var_name.as_deref().unwrap_or("N/A"),
                "type_name": alloc.type_name.as_deref().unwrap_or("N/A"),
                "thread_id": alloc.thread_id,
                "stack_trace": alloc.stack_trace,
                "runtime_state": alloc.runtime_state
            })
        }).collect();
        
        let result = serde_json::json!({
            "unsafe_ffi_analysis": {
                "total_allocations": allocations.len(),
                "allocations": unsafe_data
            }
        });
        
        serde_json::to_string_pretty(&result)
            .map_err(|e| BinaryExportError::CorruptedData(format!("Unsafe FFI JSON serialization failed: {}", e)))
    }
    
    /// Generate complex types analysis JSON data
    fn generate_complex_types_json(allocations: &[AllocationInfo]) -> Result<String, BinaryExportError> {
        // Focus on complex type information
        let complex_data: Vec<serde_json::Value> = allocations.iter().map(|alloc| {
            serde_json::json!({
                "ptr": format!("0x{:x}", alloc.ptr),
                "size": alloc.size,
                "var_name": alloc.var_name.as_deref().unwrap_or("N/A"),
                "type_name": alloc.type_name.as_deref().unwrap_or("N/A"),
                "smart_pointer_info": alloc.smart_pointer_info,
                "memory_layout": alloc.memory_layout,
                "generic_info": alloc.generic_info,
                "dynamic_type_info": alloc.dynamic_type_info,
                "generic_instantiation": alloc.generic_instantiation,
                "type_relationships": alloc.type_relationships,
                "type_usage": alloc.type_usage
            })
        }).collect();
        
        let result = serde_json::json!({
            "complex_types_analysis": {
                "total_allocations": allocations.len(),
                "allocations": complex_data
            }
        });
        
        serde_json::to_string_pretty(&result)
            .map_err(|e| BinaryExportError::CorruptedData(format!("Complex types JSON serialization failed: {}", e)))
    }
    
    /// Validate allocation data for consistency
    fn validate_allocations(allocations: &[AllocationInfo]) -> Result<(), BinaryExportError> {
        for (i, alloc) in allocations.iter().enumerate() {
            // Check for null pointers
            if alloc.ptr == 0 {
                return Err(BinaryExportError::CorruptedData(
                    format!("Null pointer in allocation {}", i)
                ));
            }
            
            // Check for zero-sized allocations
            if alloc.size == 0 {
                return Err(BinaryExportError::CorruptedData(
                    format!("Zero-sized allocation {}", i)
                ));
            }
            
            // Check timestamp validity
            if alloc.timestamp_alloc == 0 {
                return Err(BinaryExportError::CorruptedData(
                    format!("Invalid timestamp in allocation {}", i)
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::binary::writer::BinaryWriter;
    use tempfile::NamedTempFile;
    
    fn create_test_allocations() -> Vec<AllocationInfo> {
        vec![
            AllocationInfo {
                ptr: 0x1000,
                size: 1024,
                var_name: Some("buffer".to_string()),
                type_name: Some("Vec<u8>".to_string()),
                scope_name: None,
                timestamp_alloc: 1234567890,
                timestamp_dealloc: None,
                thread_id: "main".to_string(),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: None,
                smart_pointer_info: None,
                memory_layout: None,
                generic_info: None,
                dynamic_type_info: None,
                runtime_state: None,
                stack_allocation: None,
                temporary_object: None,
                fragmentation_analysis: None,
                generic_instantiation: None,
                type_relationships: None,
                type_usage: None,
                function_call_tracking: None,
                lifecycle_tracking: None,
                access_tracking: None,
            },
            AllocationInfo {
                ptr: 0x2000,
                size: 512,
                var_name: Some("data".to_string()),
                type_name: Some("String".to_string()),
                scope_name: None,
                timestamp_alloc: 1234567891,
                timestamp_dealloc: None,
                thread_id: "worker".to_string(),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: None,
                smart_pointer_info: None,
                memory_layout: None,
                generic_info: None,
                dynamic_type_info: None,
                runtime_state: None,
                stack_allocation: None,
                temporary_object: None,
                fragmentation_analysis: None,
                generic_instantiation: None,
                type_relationships: None,
                type_usage: None,
                function_call_tracking: None,
                lifecycle_tracking: None,
                access_tracking: None,
            },
        ]
    }
    
    #[test]
    fn test_load_allocations() {
        let temp_file = NamedTempFile::new().unwrap();
        let test_allocations = create_test_allocations();
        
        // Write test data
        {
            let mut writer = BinaryWriter::new(temp_file.path()).unwrap();
            writer.write_header(test_allocations.len() as u32).unwrap();
            for alloc in &test_allocations {
                writer.write_allocation(alloc).unwrap();
            }
            writer.finish().unwrap();
        }
        
        // Load and verify
        let loaded = BinaryParser::load_allocations(temp_file.path()).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].ptr, 0x1000);
        assert_eq!(loaded[1].ptr, 0x2000);
    }
    
    #[test]
    fn test_json_conversion() {
        let temp_binary = NamedTempFile::new().unwrap();
        let temp_json = NamedTempFile::new().unwrap();
        let test_allocations = create_test_allocations();
        
        // Write binary data
        {
            let mut writer = BinaryWriter::new(temp_binary.path()).unwrap();
            writer.write_header(test_allocations.len() as u32).unwrap();
            for alloc in &test_allocations {
                writer.write_allocation(alloc).unwrap();
            }
            writer.finish().unwrap();
        }
        
        // Convert to JSON
        let result = BinaryParser::to_json(temp_binary.path(), temp_json.path());
        assert!(result.is_ok());
        
        // Verify JSON file exists and has content
        let json_content = std::fs::read_to_string(temp_json.path()).unwrap();
        assert!(!json_content.is_empty());
        assert!(json_content.contains("buffer"));
        assert!(json_content.contains("Vec<u8>"));
    }
    
    #[test]
    fn test_html_conversion() {
        let temp_binary = NamedTempFile::new().unwrap();
        let temp_html = NamedTempFile::new().unwrap();
        let test_allocations = create_test_allocations();
        
        // Write binary data
        {
            let mut writer = BinaryWriter::new(temp_binary.path()).unwrap();
            writer.write_header(test_allocations.len() as u32).unwrap();
            for alloc in &test_allocations {
                writer.write_allocation(alloc).unwrap();
            }
            writer.finish().unwrap();
        }
        
        // Convert to HTML
        let result = BinaryParser::to_html(temp_binary.path(), temp_html.path());
        assert!(result.is_ok());
        
        // Verify HTML file exists and has content
        let html_content = std::fs::read_to_string(temp_html.path()).unwrap();
        assert!(!html_content.is_empty());
        assert!(html_content.contains("<html>"));
        assert!(html_content.contains("Memory Analysis Report"));
        assert!(html_content.contains("0x1000"));
    }
}