//! Binary compatibility tests
//!
//! This module contains comprehensive compatibility tests to ensure that:
//! 1. Binary export produces equivalent data to JSON export
//! 2. API compatibility is maintained for existing code
//! 3. Version compatibility works correctly
//! 4. Cross-platform format consistency is maintained

use memscope::core::tracker::MemoryTracker;
use memscope::core::types::{AllocationInfo, MemoryStats, TrackingResult};
use memscope::export::binary_exporter::{BinaryExporter, BinaryExportOptions};
use memscope::export::binary_parser::BinaryParser;
use memscope::export::binary_converter::BinaryConverter;
use memscope::export::optimized_json_export::OptimizedExportOptions;
use serde_json;
use std::collections::{HashMap, HashSet};
use tempfile::TempDir;

/// Data equivalence checker for comparing binary and JSON outputs
#[derive(Debug)]
pub struct DataEquivalenceChecker {
    pub tolerance: f64,
    pub ignore_fields: HashSet<String>,
}

impl Default for DataEquivalenceChecker {
    fn default() -> Self {
        let mut ignore_fields = HashSet::new();
        // Fields that may legitimately differ between formats
        ignore_fields.insert("export_timestamp".to_string());
        ignore_fields.insert("format_version".to_string());
        ignore_fields.insert("export_duration_ms".to_string());
        
        Self {
            tolerance: 0.001, // 0.1% tolerance for floating point comparisons
            ignore_fields,
        }
    }
}

impl DataEquivalenceChecker {
    pub fn compare_json_outputs(
        &self,
        binary_json: &str,
        direct_json: &str,
    ) -> Result<EquivalenceResult, Box<dyn std::error::Error>> {
        let binary_data: serde_json::Value = serde_json::from_str(binary_json)?;
        let direct_data: serde_json::Value = serde_json::from_str(direct_json)?;
        
        let mut result = EquivalenceResult::new();
        self.compare_values(&binary_data, &direct_data, "", &mut result);
        
        Ok(result)
    }
    
    fn compare_values(
        &self,
        binary_val: &serde_json::Value,
        direct_val: &serde_json::Value,
        path: &str,
        result: &mut EquivalenceResult,
    ) {
        match (binary_val, direct_val) {
            (serde_json::Value::Object(b_obj), serde_json::Value::Object(d_obj)) => {
                self.compare_objects(b_obj, d_obj, path, result);
            }
            (serde_json::Value::Array(b_arr), serde_json::Value::Array(d_arr)) => {
                self.compare_arrays(b_arr, d_arr, path, result);
            }
            (serde_json::Value::Number(b_num), serde_json::Value::Number(d_num)) => {
                self.compare_numbers(b_num, d_num, path, result);
            }
            (b_val, d_val) if b_val == d_val => {
                // Values are identical
            }
            _ => {
                result.add_difference(path, format!("Value mismatch: {:?} != {:?}", binary_val, direct_val));
            }
        }
    }
}    
 
   fn compare_objects(
        &self,
        binary_obj: &serde_json::Map<String, serde_json::Value>,
        direct_obj: &serde_json::Map<String, serde_json::Value>,
        path: &str,
        result: &mut EquivalenceResult,
    ) {
        // Check all keys from both objects
        let mut all_keys: HashSet<String> = binary_obj.keys().cloned().collect();
        all_keys.extend(direct_obj.keys().cloned());
        
        for key in all_keys {
            let current_path = if path.is_empty() { key.clone() } else { format!("{}.{}", path, key) };
            
            // Skip ignored fields
            if self.ignore_fields.contains(&key) {
                continue;
            }
            
            match (binary_obj.get(&key), direct_obj.get(&key)) {
                (Some(b_val), Some(d_val)) => {
                    self.compare_values(b_val, d_val, &current_path, result);
                }
                (Some(_), None) => {
                    result.add_difference(&current_path, "Key exists in binary but not in direct JSON".to_string());
                }
                (None, Some(_)) => {
                    result.add_difference(&current_path, "Key exists in direct but not in binary JSON".to_string());
                }
                (None, None) => unreachable!(),
            }
        }
    }
    
    fn compare_arrays(
        &self,
        binary_arr: &[serde_json::Value],
        direct_arr: &[serde_json::Value],
        path: &str,
        result: &mut EquivalenceResult,
    ) {
        if binary_arr.len() != direct_arr.len() {
            result.add_difference(path, format!("Array length mismatch: {} != {}", binary_arr.len(), direct_arr.len()));
            return;
        }
        
        for (i, (b_val, d_val)) in binary_arr.iter().zip(direct_arr.iter()).enumerate() {
            let current_path = format!("{}[{}]", path, i);
            self.compare_values(b_val, d_val, &current_path, result);
        }
    }
    
    fn compare_numbers(
        &self,
        binary_num: &serde_json::Number,
        direct_num: &serde_json::Number,
        path: &str,
        result: &mut EquivalenceResult,
    ) {
        if let (Some(b_f64), Some(d_f64)) = (binary_num.as_f64(), direct_num.as_f64()) {
            let diff = (b_f64 - d_f64).abs();
            let relative_diff = if d_f64.abs() > 0.0 { diff / d_f64.abs() } else { diff };
            
            if relative_diff > self.tolerance {
                result.add_difference(path, format!("Number difference too large: {} vs {} (diff: {:.6})", b_f64, d_f64, relative_diff));
            }
        } else if binary_num != direct_num {
            result.add_difference(path, format!("Number mismatch: {:?} != {:?}", binary_num, direct_num));
        }
    }
}

/// Result of data equivalence comparison
#[derive(Debug, Clone)]
pub struct EquivalenceResult {
    pub differences: Vec<(String, String)>,
    pub is_equivalent: bool,
}

impl EquivalenceResult {
    pub fn new() -> Self {
        Self {
            differences: Vec::new(),
            is_equivalent: true,
        }
    }
    
    pub fn add_difference(&mut self, path: &str, message: String) {
        self.differences.push((path.to_string(), message));
        self.is_equivalent = false;
    }
    
    pub fn print_summary(&self) {
        if self.is_equivalent {
            println!("✅ Data equivalence check passed");
        } else {
            println!("❌ Data equivalence check failed with {} differences:", self.differences.len());
            for (path, message) in &self.differences {
                println!("  {}: {}", path, message);
            }
        }
    }
}/// Cr
eate a test tracker with deterministic data for compatibility testing
fn create_deterministic_tracker() -> TrackingResult<MemoryTracker> {
    let mut tracker = MemoryTracker::new();
    
    // Create predictable allocations for consistent testing
    let test_data = vec![
        (0x1000, 1024, "Vec<i32>"),
        (0x2000, 2048, "String"),
        (0x3000, 512, "HashMap<String,String>"),
        (0x4000, 4096, "Box<[u8]>"),
        (0x5000, 256, "Arc<Mutex<Data>>"),
        (0x6000, 8192, "Vec<String>"),
        (0x7000, 128, "BTreeMap<u64,Value>"),
        (0x8000, 1536, "CustomStruct"),
    ];
    
    for (addr, size, type_name) in test_data {
        let ptr = addr as *mut u8;
        tracker.track_allocation(ptr, size, Some(type_name.to_string()))?;
    }
    
    // Deallocate some to create realistic patterns
    tracker.track_deallocation(0x3000 as *mut u8)?;
    tracker.track_deallocation(0x7000 as *mut u8)?;
    
    Ok(tracker)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_output_equivalence() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_deterministic_tracker().expect("Failed to create test tracker");
        
        println!("\n=== JSON Output Equivalence Test ===");
        
        // Export directly to JSON
        let direct_json_path = temp_dir.path().join("direct.json");
        tracker.export_to_json(&direct_json_path).expect("Direct JSON export failed");
        
        // Export to binary, then convert to JSON
        let binary_path = temp_dir.path().join("test.bin");
        let binary_json_path = temp_dir.path().join("from_binary.json");
        
        tracker.export_to_binary(&binary_path).expect("Binary export failed");
        BinaryConverter::binary_to_json(&binary_path, &binary_json_path)
            .expect("Binary to JSON conversion failed");
        
        // Compare the JSON outputs
        let direct_json = std::fs::read_to_string(&direct_json_path).expect("Failed to read direct JSON");
        let binary_json = std::fs::read_to_string(&binary_json_path).expect("Failed to read binary JSON");
        
        let checker = DataEquivalenceChecker::default();
        let result = checker.compare_json_outputs(&binary_json, &direct_json)
            .expect("Failed to compare JSON outputs");
        
        result.print_summary();
        
        assert!(result.is_equivalent, "JSON outputs should be equivalent");
        
        // Additional verification: parse both as JSON and compare key metrics
        let direct_data: serde_json::Value = serde_json::from_str(&direct_json).unwrap();
        let binary_data: serde_json::Value = serde_json::from_str(&binary_json).unwrap();
        
        // Check that both have the same structure
        assert_eq!(direct_data.get("memory_stats").is_some(), binary_data.get("memory_stats").is_some());
        assert_eq!(direct_data.get("allocations").is_some(), binary_data.get("allocations").is_some());
        assert_eq!(direct_data.get("type_memory_usage").is_some(), binary_data.get("type_memory_usage").is_some());
    }

    #[test]
    fn test_api_compatibility() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_deterministic_tracker().expect("Failed to create test tracker");
        
        println!("\n=== API Compatibility Test ===");
        
        // Test that existing JSON API still works
        let json_path = temp_dir.path().join("api_test.json");
        tracker.export_to_json(&json_path).expect("JSON export should still work");
        
        // Test that new binary API works
        let binary_path = temp_dir.path().join("api_test.bin");
        tracker.export_to_binary(&binary_path).expect("Binary export should work");
        
        // Test that options-based APIs work
        let json_options = OptimizedExportOptions::default();
        let json_with_options_path = temp_dir.path().join("api_test_options.json");
        tracker.export_to_json_with_options(&json_with_options_path, json_options)
            .expect("JSON export with options should work");
        
        let binary_options = BinaryExportOptions::default();
        let binary_with_options_path = temp_dir.path().join("api_test_options.bin");
        tracker.export_to_binary_with_options(&binary_with_options_path, binary_options)
            .expect("Binary export with options should work");
        
        // Test convenience methods
        let fast_binary_path = temp_dir.path().join("api_test_fast.bin");
        tracker.export_to_binary_fast(&fast_binary_path)
            .expect("Fast binary export should work");
        
        let comprehensive_binary_path = temp_dir.path().join("api_test_comprehensive.bin");
        tracker.export_to_binary_comprehensive(&comprehensive_binary_path)
            .expect("Comprehensive binary export should work");
        
        // Verify all files were created
        assert!(json_path.exists(), "JSON file should be created");
        assert!(binary_path.exists(), "Binary file should be created");
        assert!(json_with_options_path.exists(), "JSON with options file should be created");
        assert!(binary_with_options_path.exists(), "Binary with options file should be created");
        assert!(fast_binary_path.exists(), "Fast binary file should be created");
        assert!(comprehensive_binary_path.exists(), "Comprehensive binary file should be created");
        
        println!("✅ All API methods work correctly");
    }

    #[test]
    fn test_version_compatibility() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_deterministic_tracker().expect("Failed to create test tracker");
        
        println!("\n=== Version Compatibility Test ===");
        
        // Create binary file with current version
        let binary_path = temp_dir.path().join("version_test.bin");
        tracker.export_to_binary(&binary_path).expect("Binary export failed");
        
        // Test parsing with different parser options
        let strict_parser = BinaryParser::with_options(
            memscope::export::binary_parser::BinaryParserOptions::strict()
        );
        
        let recovery_parser = BinaryParser::with_options(
            memscope::export::binary_parser::BinaryParserOptions::recovery_mode()
        );
        
        let fast_parser = BinaryParser::with_options(
            memscope::export::binary_parser::BinaryParserOptions::fast()
        );
        
        // All parsers should be able to read the current format
        let mut strict_parser = strict_parser;
        let mut recovery_parser = recovery_parser;
        let mut fast_parser = fast_parser;
        
        strict_parser.load_from_file(&binary_path).expect("Strict parser should work");
        recovery_parser.load_from_file(&binary_path).expect("Recovery parser should work");
        fast_parser.load_from_file(&binary_path).expect("Fast parser should work");
        
        // Test that all parsers can extract the same data
        let strict_allocations = strict_parser.load_allocations().expect("Strict parser should load allocations");
        let recovery_allocations = recovery_parser.load_allocations().expect("Recovery parser should load allocations");
        let fast_allocations = fast_parser.load_allocations().expect("Fast parser should load allocations");
        
        assert_eq!(strict_allocations.len(), recovery_allocations.len());
        assert_eq!(strict_allocations.len(), fast_allocations.len());
        
        println!("✅ Version compatibility maintained across parser modes");
    }
}   
 #[test]
    fn test_cross_platform_consistency() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_deterministic_tracker().expect("Failed to create test tracker");
        
        println!("\n=== Cross-Platform Consistency Test ===");
        
        // Test different endianness handling by creating binary with different options
        let little_endian_path = temp_dir.path().join("little_endian.bin");
        let options = BinaryExportOptions::default();
        tracker.export_to_binary_with_options(&little_endian_path, options)
            .expect("Little endian export failed");
        
        // Parse the binary file and verify data integrity
        let mut parser = BinaryParser::new();
        parser.load_from_file(&little_endian_path).expect("Failed to parse binary file");
        
        let allocations = parser.load_allocations().expect("Failed to load allocations");
        let stats = parser.load_memory_stats().expect("Failed to load stats");
        
        // Verify that the data makes sense (basic sanity checks)
        assert!(!allocations.is_empty(), "Should have allocations");
        assert!(stats.total_allocations > 0, "Should have allocation count");
        assert!(stats.total_allocated > 0, "Should have allocated bytes");
        
        // Test that we can convert back to JSON and get consistent results
        let json_path = temp_dir.path().join("cross_platform.json");
        BinaryConverter::binary_to_json(&little_endian_path, &json_path)
            .expect("Binary to JSON conversion failed");
        
        let json_content = std::fs::read_to_string(&json_path).expect("Failed to read JSON");
        let json_data: serde_json::Value = serde_json::from_str(&json_content)
            .expect("Failed to parse JSON");
        
        // Verify JSON structure is correct
        assert!(json_data.get("memory_stats").is_some(), "JSON should have memory_stats");
        assert!(json_data.get("allocations").is_some(), "JSON should have allocations");
        
        println!("✅ Cross-platform consistency maintained");
    }

    #[test]
    fn test_data_integrity_after_conversion() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_deterministic_tracker().expect("Failed to create test tracker");
        
        println!("\n=== Data Integrity After Conversion Test ===");
        
        // Get original data from tracker
        let original_stats = tracker.get_stats();
        let original_allocations = tracker.get_active_allocations();
        let original_type_usage = tracker.get_type_memory_usage();
        
        // Export to binary and convert back to JSON
        let binary_path = temp_dir.path().join("integrity_test.bin");
        let json_path = temp_dir.path().join("integrity_test.json");
        
        tracker.export_to_binary(&binary_path).expect("Binary export failed");
        BinaryConverter::binary_to_json(&binary_path, &json_path)
            .expect("Binary to JSON conversion failed");
        
        // Parse the converted JSON and verify data integrity
        let json_content = std::fs::read_to_string(&json_path).expect("Failed to read JSON");
        let json_data: serde_json::Value = serde_json::from_str(&json_content)
            .expect("Failed to parse JSON");
        
        // Check memory stats
        if let Some(stats_obj) = json_data.get("memory_stats") {
            let total_allocations = stats_obj.get("total_allocations")
                .and_then(|v| v.as_u64())
                .expect("Should have total_allocations") as usize;
            let active_allocations = stats_obj.get("active_allocations")
                .and_then(|v| v.as_u64())
                .expect("Should have active_allocations") as usize;
            let total_allocated = stats_obj.get("total_allocated")
                .and_then(|v| v.as_u64())
                .expect("Should have total_allocated") as usize;
            
            assert_eq!(total_allocations, original_stats.total_allocations);
            assert_eq!(active_allocations, original_stats.active_allocations);
            assert_eq!(total_allocated, original_stats.total_allocated);
        } else {
            panic!("JSON should contain memory_stats");
        }
        
        // Check allocations array
        if let Some(allocations_array) = json_data.get("allocations").and_then(|v| v.as_array()) {
            assert_eq!(allocations_array.len(), original_allocations.len());
            
            // Verify first few allocations in detail
            for (i, allocation) in allocations_array.iter().take(3).enumerate() {
                let size = allocation.get("size")
                    .and_then(|v| v.as_u64())
                    .expect("Allocation should have size") as usize;
                let type_name = allocation.get("type_name")
                    .and_then(|v| v.as_str());
                
                assert!(size > 0, "Allocation size should be positive");
                if let Some(original_alloc) = original_allocations.get(i) {
                    assert_eq!(size, original_alloc.size);
                    assert_eq!(type_name, original_alloc.type_name.as_deref());
                }
            }
        } else {
            panic!("JSON should contain allocations array");
        }
        
        // Check type memory usage
        if let Some(type_usage_array) = json_data.get("type_memory_usage").and_then(|v| v.as_array()) {
            assert_eq!(type_usage_array.len(), original_type_usage.len());
        }
        
        println!("✅ Data integrity maintained after conversion");
    }

    #[test]
    fn test_backward_compatibility() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_deterministic_tracker().expect("Failed to create test tracker");
        
        println!("\n=== Backward Compatibility Test ===");
        
        // Create binary file with current version
        let binary_path = temp_dir.path().join("backward_compat.bin");
        tracker.export_to_binary(&binary_path).expect("Binary export failed");
        
        // Test that we can read it with different compatibility settings
        let mut parser = BinaryParser::with_options(
            memscope::export::binary_parser::BinaryParserOptions {
                strict_validation: false,
                enable_recovery: true,
                verify_checksums: true,
                ..Default::default()
            }
        );
        
        let parse_result = parser.load_from_file(&binary_path)
            .expect("Should be able to parse with backward compatibility");
        
        // Verify we can extract all expected data
        let allocations = parser.load_allocations().expect("Should load allocations");
        let stats = parser.load_memory_stats().expect("Should load stats");
        let type_usage = parser.load_type_memory_usage().expect("Should load type usage");
        
        assert!(!allocations.is_empty());
        assert!(stats.total_allocations > 0);
        assert!(!type_usage.is_empty());
        
        println!("✅ Backward compatibility maintained");
    }

    #[test]
    fn test_format_validation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_deterministic_tracker().expect("Failed to create test tracker");
        
        println!("\n=== Format Validation Test ===");
        
        // Create binary file
        let binary_path = temp_dir.path().join("format_validation.bin");
        tracker.export_to_binary(&binary_path).expect("Binary export failed");
        
        // Read raw file and verify format structure
        let file_data = std::fs::read(&binary_path).expect("Failed to read binary file");
        
        // Check magic number (first 8 bytes should be "MEMSCOPE")
        assert_eq!(&file_data[0..8], b"MEMSCOPE", "Magic number should be correct");
        
        // Check that file is not empty and has reasonable size
        assert!(file_data.len() > 64, "File should be larger than just the header");
        assert!(file_data.len() < 10 * 1024 * 1024, "File should not be unreasonably large for test data");
        
        // Test that parser can validate the format
        let mut parser = BinaryParser::with_options(
            memscope::export::binary_parser::BinaryParserOptions::strict()
        );
        
        parser.load_from_file(&binary_path).expect("Strict parser should validate format correctly");
        
        println!("✅ Format validation passed");
    }
}

/// Integration tests that verify compatibility across different scenarios
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_large_dataset_compatibility() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mut tracker = MemoryTracker::new();
        
        // Create a larger dataset for more comprehensive testing
        for i in 0..5000 {
            let size = 64 + (i % 4096);
            let type_name = match i % 5 {
                0 => "Vec<i32>",
                1 => "String", 
                2 => "HashMap<String,String>",
                3 => "Box<[u8]>",
                _ => "CustomType",
            };
            
            let ptr = (0x10000000 + i * 16) as *mut u8;
            tracker.track_allocation(ptr, size, Some(type_name.to_string()))
                .expect("Allocation tracking failed");
        }
        
        // Export both formats
        let binary_path = temp_dir.path().join("large_dataset.bin");
        let json_path = temp_dir.path().join("large_dataset.json");
        let converted_json_path = temp_dir.path().join("large_dataset_converted.json");
        
        tracker.export_to_binary(&binary_path).expect("Binary export failed");
        tracker.export_to_json(&json_path).expect("JSON export failed");
        BinaryConverter::binary_to_json(&binary_path, &converted_json_path)
            .expect("Binary to JSON conversion failed");
        
        // Compare the outputs
        let direct_json = std::fs::read_to_string(&json_path).expect("Failed to read direct JSON");
        let converted_json = std::fs::read_to_string(&converted_json_path).expect("Failed to read converted JSON");
        
        let checker = DataEquivalenceChecker::default();
        let result = checker.compare_json_outputs(&converted_json, &direct_json)
            .expect("Failed to compare JSON outputs");
        
        assert!(result.is_equivalent, "Large dataset should maintain equivalence");
        
        println!("✅ Large dataset compatibility verified");
    }
}