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
        let user_allocations: Vec<AllocationInfo> = allocations.into_iter()
            .filter(|a| a.var_name.is_some())
            .collect();

        // Use StandardAnalysisEngine but with filtered user allocations for performance
        let analysis_engine = StandardAnalysisEngine::new();
        
        // Generate 5 JSON files with proper analysis data
        let analyses = [
            ("memory_analysis", analysis_engine.create_memory_analysis(&user_allocations)
                .map_err(|e| BinaryExportError::CorruptedData(format!("Memory analysis failed: {}", e)))?),
            ("lifetime", analysis_engine.create_lifetime_analysis(&user_allocations)
                .map_err(|e| BinaryExportError::CorruptedData(format!("Lifetime analysis failed: {}", e)))?),
            ("performance", analysis_engine.create_performance_analysis(&user_allocations)
                .map_err(|e| BinaryExportError::CorruptedData(format!("Performance analysis failed: {}", e)))?),
            ("unsafe_ffi", analysis_engine.create_unsafe_ffi_analysis(&user_allocations)
                .map_err(|e| BinaryExportError::CorruptedData(format!("Unsafe FFI analysis failed: {}", e)))?),
            ("complex_types", analysis_engine.create_complex_types_analysis(&user_allocations)
                .map_err(|e| BinaryExportError::CorruptedData(format!("Complex types analysis failed: {}", e)))?),
        ];

        for (file_type, analysis_data) in analyses {
            let file_path = project_dir.join(format!("{}_{}.json", base_name, file_type));
            let json_content = serde_json::to_string(&analysis_data.data)
                .map_err(|e| BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e)))?;
            std::fs::write(file_path, json_content)?;
        }

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
}