//! JSON output validation tests
//! 
//! Tests JSON generation, format compliance, data accuracy, and compatibility
//! with HTML rendering requirements.

use memscope_rs::{track_var, get_global_tracker};
use memscope_rs::export::binary::BinaryParser;
use std::path::Path;

#[test]
fn test_json_output_file_generation() {
    // Test that all required JSON files are generated
    create_comprehensive_test_dataset();
    
    let binary_file_path = "MemoryAnalysis/json_generation_test.memscope";
    ensure_output_directory();
    
    let tracker = get_global_tracker();
    match tracker.export_to_binary(binary_file_path) {
        Ok(_) => {
            // Parse to JSON using optimized method
            match BinaryParser::parse_full_binary_to_json_with_existing_optimizations(
                binary_file_path,
                "json_generation_test"
            ) {
                Ok(_) => {
                    // Verify all expected JSON files exist
                    let expected_json_files = [
                        "MemoryAnalysis/json_generation_test/json_generation_test_memory_analysis.json",
                        "MemoryAnalysis/json_generation_test/json_generation_test_lifetime.json",
                        "MemoryAnalysis/json_generation_test/json_generation_test_performance.json",
                        "MemoryAnalysis/json_generation_test/json_generation_test_unsafe_ffi.json",
                        "MemoryAnalysis/json_generation_test/json_generation_test_complex_types.json",
                    ];
                    
                    for json_file_path in &expected_json_files {
                        if !Path::new(json_file_path).exists() {
                            panic!("Required JSON file missing: {}", json_file_path);
                        }
                    }
                }
                Err(e) => {
                    panic!("JSON generation failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Binary export for JSON test failed: {:?}", e);
        }
    }
    
    cleanup_test_artifacts(binary_file_path, "json_generation_test");
}

#[test]
fn test_json_format_validity() {
    // Test that generated JSON files are valid JSON format
    create_comprehensive_test_dataset();
    
    let binary_file_path = "MemoryAnalysis/json_validity_test.memscope";
    ensure_output_directory();
    
    let tracker = get_global_tracker();
    match tracker.export_to_binary(binary_file_path) {
        Ok(_) => {
            match BinaryParser::parse_full_binary_to_json_with_existing_optimizations(
                binary_file_path,
                "json_validity_test"
            ) {
                Ok(_) => {
                    // Validate JSON format for each file
                    let json_files = [
                        "MemoryAnalysis/json_validity_test/json_validity_test_memory_analysis.json",
                        "MemoryAnalysis/json_validity_test/json_validity_test_lifetime.json",
                        "MemoryAnalysis/json_validity_test/json_validity_test_performance.json",
                        "MemoryAnalysis/json_validity_test/json_validity_test_unsafe_ffi.json",
                        "MemoryAnalysis/json_validity_test/json_validity_test_complex_types.json",
                    ];
                    
                    for json_file_path in &json_files {
                        validate_json_file_format(json_file_path);
                    }
                }
                Err(e) => {
                    panic!("JSON validity test parsing failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Binary export for JSON validity test failed: {:?}", e);
        }
    }
    
    cleanup_test_artifacts(binary_file_path, "json_validity_test");
}

#[test]
fn test_json_data_accuracy() {
    // Test accuracy of data in generated JSON files
    create_known_allocation_pattern();
    
    let binary_file_path = "MemoryAnalysis/json_accuracy_test.memscope";
    ensure_output_directory();
    
    let tracker = get_global_tracker();
    match tracker.export_to_binary(binary_file_path) {
        Ok(_) => {
            match BinaryParser::parse_full_binary_to_json_with_existing_optimizations(
                binary_file_path,
                "json_accuracy_test"
            ) {
                Ok(_) => {
                    // Verify data accuracy in memory analysis JSON
                    let memory_json_path = "MemoryAnalysis/json_accuracy_test/json_accuracy_test_memory_analysis.json";
                    validate_memory_analysis_data_accuracy(memory_json_path);
                    
                    // Verify data accuracy in lifetime JSON
                    let lifetime_json_path = "MemoryAnalysis/json_accuracy_test/json_accuracy_test_lifetime.json";
                    validate_lifetime_data_accuracy(lifetime_json_path);
                }
                Err(e) => {
                    panic!("JSON accuracy test parsing failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Binary export for JSON accuracy test failed: {:?}", e);
        }
    }
    
    cleanup_test_artifacts(binary_file_path, "json_accuracy_test");
}

#[test]
fn test_json_size_optimization() {
    // Test JSON file size optimization and compression
    let test_sizes = vec![50, 200, 500];
    
    for allocation_count in test_sizes {
        create_test_allocation_set(allocation_count);
        
        let binary_file_path = format!("MemoryAnalysis/json_size_test_{}.memscope", allocation_count);
        ensure_output_directory();
        
        let tracker = get_global_tracker();
        match tracker.export_to_binary(&binary_file_path) {
            Ok(_) => {
                match BinaryParser::parse_full_binary_to_json_with_existing_optimizations(
                    &binary_file_path,
                    &format!("json_size_test_{}", allocation_count)
                ) {
                    Ok(_) => {
                        // Calculate total JSON size
                        let total_json_size = calculate_total_json_size(&format!("json_size_test_{}", allocation_count));
                        
                        // JSON size should scale reasonably with allocation count
                        let expected_max_size = allocation_count * 1000; // 1KB per allocation maximum
                        if total_json_size > expected_max_size {
                            panic!("JSON size not optimized: {} allocations produced {} bytes (expected <{} bytes)", 
                                   allocation_count, total_json_size, expected_max_size);
                        }
                    }
                    Err(e) => {
                        panic!("JSON size test parsing failed for {} allocations: {:?}", allocation_count, e);
                    }
                }
            }
            Err(e) => {
                panic!("Binary export for JSON size test failed: {:?}", e);
            }
        }
        
        cleanup_test_artifacts(&binary_file_path, &format!("json_size_test_{}", allocation_count));
    }
}

#[test]
fn test_json_compatibility_with_reference_format() {
    // Test compatibility with reference JSON format for HTML rendering
    create_comprehensive_test_dataset();
    
    let binary_file_path = "MemoryAnalysis/json_compatibility_test.memscope";
    ensure_output_directory();
    
    let tracker = get_global_tracker();
    match tracker.export_to_binary(binary_file_path) {
        Ok(_) => {
            match BinaryParser::parse_full_binary_to_json_with_existing_optimizations(
                binary_file_path,
                "json_compatibility_test"
            ) {
                Ok(_) => {
                    // Verify JSON structure compatibility
                    let memory_json_path = "MemoryAnalysis/json_compatibility_test/json_compatibility_test_memory_analysis.json";
                    validate_memory_json_structure(memory_json_path);
                    
                    let ffi_json_path = "MemoryAnalysis/json_compatibility_test/json_compatibility_test_unsafe_ffi.json";
                    validate_ffi_json_structure(ffi_json_path);
                }
                Err(e) => {
                    panic!("JSON compatibility test parsing failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Binary export for JSON compatibility test failed: {:?}", e);
        }
    }
    
    cleanup_test_artifacts(binary_file_path, "json_compatibility_test");
}

/// Create comprehensive test dataset with various allocation types
fn create_comprehensive_test_dataset() {
    // Various sized allocations
    for size in [64, 256, 1024, 4096] {
        let allocation = vec![0u8; size];
        track_var!(allocation);
        drop(allocation);
    }
    
    // String allocations
    for i in 0..10 {
        let string_allocation = format!("test_string_{}", i);
        track_var!(string_allocation);
        drop(string_allocation);
    }
    
    // Vector allocations
    for i in 0..5 {
        let vector_allocation: Vec<u32> = (0..100).map(|j| i * 100 + j).collect();
        track_var!(vector_allocation);
        drop(vector_allocation);
    }
}

/// Create known allocation pattern for accuracy testing
fn create_known_allocation_pattern() {
    // Create exactly 10 allocations of known sizes
    for i in 0..10 {
        let known_size = 128 * (i + 1);
        let known_allocation = vec![i as u8; known_size];
        track_var!(known_allocation);
        drop(known_allocation);
    }
}

/// Create test allocation set with specified count
fn create_test_allocation_set(count: usize) {
    for index in 0..count {
        let allocation_size = 100 + (index % 50);
        let test_data = vec![index as u8; allocation_size];
        track_var!(test_data);
        drop(test_data);
    }
}

/// Validate JSON file format
fn validate_json_file_format(json_file_path: &str) {
    match std::fs::read_to_string(json_file_path) {
        Ok(json_content) => {
            if json_content.is_empty() {
                panic!("JSON file is empty: {}", json_file_path);
            }
            
            // Attempt to parse as JSON
            match serde_json::from_str::<serde_json::Value>(&json_content) {
                Ok(_) => {
                    // JSON is valid
                }
                Err(e) => {
                    panic!("Invalid JSON format in {}: {:?}", json_file_path, e);
                }
            }
        }
        Err(e) => {
            panic!("Failed to read JSON file {}: {:?}", json_file_path, e);
        }
    }
}

/// Validate memory analysis data accuracy
fn validate_memory_analysis_data_accuracy(json_file_path: &str) {
    match std::fs::read_to_string(json_file_path) {
        Ok(json_content) => {
            match serde_json::from_str::<serde_json::Value>(&json_content) {
                Ok(json_data) => {
                    // Verify allocations array exists
                    if let Some(allocations) = json_data.get("allocations") {
                        if let Some(allocations_array) = allocations.as_array() {
                            if allocations_array.is_empty() {
                                panic!("Memory analysis should contain allocation data");
                            }
                        } else {
                            panic!("Allocations should be an array in memory analysis JSON");
                        }
                    } else {
                        panic!("Memory analysis JSON should contain allocations field");
                    }
                }
                Err(e) => {
                    panic!("Failed to parse memory analysis JSON: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Failed to read memory analysis JSON: {:?}", e);
        }
    }
}

/// Validate lifetime data accuracy
fn validate_lifetime_data_accuracy(json_file_path: &str) {
    match std::fs::read_to_string(json_file_path) {
        Ok(json_content) => {
            match serde_json::from_str::<serde_json::Value>(&json_content) {
                Ok(json_data) => {
                    // Verify lifecycle_events array exists
                    if let Some(events) = json_data.get("lifecycle_events") {
                        if let Some(events_array) = events.as_array() {
                            if events_array.is_empty() {
                                panic!("Lifetime analysis should contain lifecycle events");
                            }
                        } else {
                            panic!("Lifecycle events should be an array in lifetime JSON");
                        }
                    } else {
                        panic!("Lifetime JSON should contain lifecycle_events field");
                    }
                }
                Err(e) => {
                    panic!("Failed to parse lifetime JSON: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Failed to read lifetime JSON: {:?}", e);
        }
    }
}

/// Validate memory JSON structure compatibility
fn validate_memory_json_structure(json_file_path: &str) {
    match std::fs::read_to_string(json_file_path) {
        Ok(json_content) => {
            match serde_json::from_str::<serde_json::Value>(&json_content) {
                Ok(json_data) => {
                    // Verify required top-level fields exist
                    let required_fields = ["allocations", "memory_stats", "metadata"];
                    for field in &required_fields {
                        if !json_data.get(field).is_some() {
                            panic!("Memory JSON missing required field: {}", field);
                        }
                    }
                }
                Err(e) => {
                    panic!("Failed to parse memory JSON structure: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Failed to read memory JSON for structure validation: {:?}", e);
        }
    }
}

/// Validate FFI JSON structure compatibility
fn validate_ffi_json_structure(json_file_path: &str) {
    match std::fs::read_to_string(json_file_path) {
        Ok(json_content) => {
            match serde_json::from_str::<serde_json::Value>(&json_content) {
                Ok(json_data) => {
                    // FFI JSON should be an array (matching snapshot_unsafe_ffi.json format)
                    if !json_data.is_array() {
                        panic!("FFI JSON should be an array format");
                    }
                }
                Err(e) => {
                    panic!("Failed to parse FFI JSON structure: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Failed to read FFI JSON for structure validation: {:?}", e);
        }
    }
}

/// Calculate total JSON size for optimization testing
fn calculate_total_json_size(output_name: &str) -> usize {
    let json_files = [
        format!("MemoryAnalysis/{}/{}_memory_analysis.json", output_name, output_name),
        format!("MemoryAnalysis/{}/{}_lifetime.json", output_name, output_name),
        format!("MemoryAnalysis/{}/{}_performance.json", output_name, output_name),
        format!("MemoryAnalysis/{}/{}_unsafe_ffi.json", output_name, output_name),
        format!("MemoryAnalysis/{}/{}_complex_types.json", output_name, output_name),
    ];
    
    let mut total_size = 0;
    for json_file_path in &json_files {
        if let Ok(metadata) = std::fs::metadata(json_file_path) {
            total_size += metadata.len() as usize;
        }
    }
    
    total_size
}

/// Ensure output directory exists
fn ensure_output_directory() {
    if let Err(e) = std::fs::create_dir_all("MemoryAnalysis") {
        panic!("Failed to create output directory: {:?}", e);
    }
}

/// Clean up test artifacts
fn cleanup_test_artifacts(binary_file_path: &str, output_name: &str) {
    std::fs::remove_file(binary_file_path).ok();
    std::fs::remove_dir_all(format!("MemoryAnalysis/{}", output_name)).ok();
}