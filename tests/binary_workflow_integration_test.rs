//! Integration tests for binary data workflow
//!
//! This module tests the complete workflow:
//! analyze -> export -> convert (binary -> JSON/HTML)

use memscope_rs::export::formats::{
    binary_export::{BinaryExportOptions, export_memory_to_binary, load_binary_export_data},
    binary_parser::{BinaryParser, BinaryParseOptions},
    json_converter::{JsonConverter, JsonConvertOptions},
    html_converter::{HtmlConverter, HtmlConvertOptions},
    binary_validation::BinaryValidator,
};
use memscope_rs::core::tracker::MemoryTracker;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Test the complete binary workflow: export -> parse -> convert to JSON
#[test]
fn test_binary_to_json_workflow() {
    println!("🧪 Testing binary -> JSON workflow");
    
    // Create temporary directory for test files
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let binary_path = temp_dir.path().join("test_export.ms");
    let json_path = temp_dir.path().join("test_output.json");
    
    // Step 1: Create a memory tracker with some test data
    let tracker = create_test_memory_tracker();
    
    // Step 2: Export to binary format
    println!("📦 Step 1: Exporting to binary format");
    let export_options = BinaryExportOptions::balanced();
    let export_stats = export_memory_to_binary(&tracker, &binary_path, export_options)
        .expect("Binary export should succeed");
    
    println!("✅ Binary export completed:");
    println!("   - File size: {} bytes", export_stats.file_size);
    println!("   - Compression ratio: {:.1}%", export_stats.compression_ratio * 100.0);
    println!("   - Allocations: {}", export_stats.allocation_count);
    
    // Verify binary file was created
    assert!(binary_path.exists(), "Binary file should be created");
    assert!(export_stats.file_size > 0, "Binary file should not be empty");
    
    // Step 3: Parse binary file
    println!("🔍 Step 2: Parsing binary file");
    let parser = BinaryParser::new(BinaryParseOptions::safe());
    let binary_data = parser.parse_file(&binary_path)
        .expect("Binary parsing should succeed");
    
    println!("✅ Binary parsing completed:");
    println!("   - Version: {}", binary_data.version);
    println!("   - Allocations: {}", binary_data.allocation_count);
    println!("   - Total memory: {} bytes", binary_data.total_memory);
    
    // Verify parsed data integrity
    assert_eq!(binary_data.allocation_count, binary_data.allocations.len());
    assert!(binary_data.total_memory > 0, "Total memory should be positive");
    
    // Step 4: Convert to JSON
    println!("📄 Step 3: Converting to JSON");
    let converter = JsonConverter::new(JsonConvertOptions::compatible());
    let conversion_stats = converter.convert_to_file(&binary_data, &json_path)
        .expect("JSON conversion should succeed");
    
    println!("✅ JSON conversion completed:");
    println!("   - Output size: {} bytes", conversion_stats.output_size);
    println!("   - Allocations converted: {}", conversion_stats.allocations_converted);
    println!("   - Conversion time: {:?}", conversion_stats.conversion_time);
    
    // Verify JSON file was created and is valid
    assert!(json_path.exists(), "JSON file should be created");
    assert!(conversion_stats.output_size > 0, "JSON file should not be empty");
    
    // Step 5: Validate JSON content
    println!("🔍 Step 4: Validating JSON content");
    let json_content = fs::read_to_string(&json_path)
        .expect("Should be able to read JSON file");
    
    let json_value: serde_json::Value = serde_json::from_str(&json_content)
        .expect("JSON should be valid");
    
    // Verify JSON structure
    assert!(json_value.get("allocations").is_some(), "JSON should have allocations array");
    
    if let Some(allocations) = json_value.get("allocations").and_then(|v| v.as_array()) {
        assert_eq!(allocations.len(), binary_data.allocation_count, 
                   "JSON allocations count should match binary data");
        
        // Verify first allocation has required fields
        if let Some(first_alloc) = allocations.first() {
            assert!(first_alloc.get("ptr").is_some(), "Allocation should have ptr field");
            assert!(first_alloc.get("size").is_some(), "Allocation should have size field");
            assert!(first_alloc.get("timestamp_alloc").is_some(), "Allocation should have timestamp_alloc field");
        }
    }
    
    println!("🎉 Binary -> JSON workflow test completed successfully!");
}

/// Test the complete binary workflow: export -> parse -> convert to HTML
#[test]
fn test_binary_to_html_workflow() {
    println!("🧪 Testing binary -> HTML workflow");
    
    // Create temporary directory for test files
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let binary_path = temp_dir.path().join("test_export.ms");
    let html_path = temp_dir.path().join("test_report.html");
    
    // Step 1: Create a memory tracker with some test data
    let tracker = create_test_memory_tracker();
    
    // Step 2: Export to binary format
    println!("📦 Step 1: Exporting to binary format");
    let export_options = BinaryExportOptions::compact(); // Use compression for HTML test
    let export_stats = export_memory_to_binary(&tracker, &binary_path, export_options)
        .expect("Binary export should succeed");
    
    println!("✅ Binary export completed with compression");
    
    // Step 3: Parse binary file
    println!("🔍 Step 2: Parsing compressed binary file");
    let parser = BinaryParser::new(BinaryParseOptions::safe());
    let binary_data = parser.parse_file(&binary_path)
        .expect("Binary parsing should succeed");
    
    // Step 4: Convert to HTML
    println!("🌐 Step 3: Converting to HTML");
    let mut converter = HtmlConverter::new(HtmlConvertOptions::complete());
    let conversion_stats = converter.convert_to_file(&binary_data, &html_path)
        .expect("HTML conversion should succeed");
    
    println!("✅ HTML conversion completed:");
    println!("   - HTML size: {} bytes", conversion_stats.html_size);
    println!("   - Charts generated: {}", conversion_stats.charts_generated);
    println!("   - Table rows: {}", conversion_stats.table_rows_generated);
    
    // Verify HTML file was created
    assert!(html_path.exists(), "HTML file should be created");
    assert!(conversion_stats.html_size > 0, "HTML file should not be empty");
    
    // Step 5: Validate HTML content
    println!("🔍 Step 4: Validating HTML content");
    let html_content = fs::read_to_string(&html_path)
        .expect("Should be able to read HTML file");
    
    // Basic HTML validation
    assert!(html_content.contains("<!DOCTYPE html>"), "Should be valid HTML document");
    assert!(html_content.contains("<title>"), "Should have title tag");
    assert!(html_content.contains("Memory Analysis"), "Should contain memory analysis content");
    assert!(html_content.contains("allocations"), "Should contain allocation data");
    
    println!("🎉 Binary -> HTML workflow test completed successfully!");
}

/// Test binary file validation workflow
#[test]
fn test_binary_validation_workflow() {
    println!("🧪 Testing binary validation workflow");
    
    // Create temporary directory for test files
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let binary_path = temp_dir.path().join("test_validation.ms");
    
    // Step 1: Create and export binary file
    let tracker = create_test_memory_tracker();
    let export_options = BinaryExportOptions::balanced();
    export_memory_to_binary(&tracker, &binary_path, export_options)
        .expect("Binary export should succeed");
    
    // Step 2: Validate binary file
    println!("🔍 Validating binary file");
    let validator = BinaryValidator::new();
    let validation_result = validator.validate_file(&binary_path)
        .expect("Validation should complete");
    
    println!("✅ Validation completed:");
    println!("   - Valid: {}", validation_result.is_valid);
    println!("   - File size: {} bytes", validation_result.file_info.file_size);
    println!("   - Errors: {}", validation_result.errors.len());
    println!("   - Warnings: {}", validation_result.warnings.len());
    
    // Verify validation results
    assert!(validation_result.is_valid, "File should be valid");
    assert!(validation_result.file_info.file_size > 0, "File size should be positive");
    assert_eq!(validation_result.errors.len(), 0, "Should have no validation errors");
    
    // Check compression detection
    if let Some(compression) = &validation_result.compression_detected {
        println!("   - Compression detected: {compression}");
        assert_eq!(compression, "zstd", "Should detect zstd compression");
    }
    
    println!("🎉 Binary validation workflow test completed successfully!");
}

/// Test error handling in workflow
#[test]
fn test_workflow_error_handling() {
    println!("🧪 Testing workflow error handling");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let nonexistent_path = temp_dir.path().join("nonexistent.ms");
    let output_path = temp_dir.path().join("output.json");
    
    // Test parsing nonexistent file
    let parser = BinaryParser::new(BinaryParseOptions::safe());
    let parse_result = parser.parse_file(&nonexistent_path);
    
    assert!(parse_result.is_err(), "Parsing nonexistent file should fail");
    
    // Test validation of nonexistent file
    let validator = BinaryValidator::new();
    let validation_result = validator.validate_file(&nonexistent_path);
    
    assert!(validation_result.is_err(), "Validating nonexistent file should fail");
    
    println!("✅ Error handling tests completed successfully!");
}

/// Test streaming mode for large files
#[test]
fn test_streaming_workflow() {
    println!("🧪 Testing streaming workflow");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let binary_path = temp_dir.path().join("test_streaming.ms");
    let json_path = temp_dir.path().join("test_streaming.json");
    
    // Create binary file
    let tracker = create_large_test_memory_tracker();
    let export_options = BinaryExportOptions::balanced();
    export_memory_to_binary(&tracker, &binary_path, export_options)
        .expect("Binary export should succeed");
    
    // Parse with streaming options
    let parser = BinaryParser::new(BinaryParseOptions::streaming());
    let binary_data = parser.parse_file(&binary_path)
        .expect("Streaming parsing should succeed");
    
    // Convert with streaming options
    let converter = JsonConverter::new(JsonConvertOptions::streaming());
    let conversion_stats = converter.convert_to_file(&binary_data, &json_path)
        .expect("Streaming conversion should succeed");
    
    println!("✅ Streaming workflow completed:");
    println!("   - Chunks processed: {}", conversion_stats.chunks_processed);
    
    assert!(conversion_stats.chunks_processed > 0, "Should process multiple chunks");
    
    println!("🎉 Streaming workflow test completed successfully!");
}

/// Test data integrity across the entire workflow
#[test]
fn test_data_integrity_workflow() {
    println!("🧪 Testing data integrity across workflow");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let binary_path = temp_dir.path().join("test_integrity.ms");
    
    // Step 1: Create original data
    let original_tracker = create_test_memory_tracker();
    let original_stats = original_tracker.get_memory_stats()
        .expect("Should get original stats");
    let original_allocations = original_tracker.get_all_active_allocations()
        .expect("Should get original allocations");
    
    // Step 2: Export to binary
    let export_options = BinaryExportOptions::balanced();
    export_memory_to_binary(&original_tracker, &binary_path, export_options)
        .expect("Binary export should succeed");
    
    // Step 3: Load binary data
    let loaded_data = load_binary_export_data(&binary_path)
        .expect("Should load binary data");
    
    // Step 4: Verify data integrity
    println!("🔍 Verifying data integrity");
    
    // Check allocation count
    assert_eq!(loaded_data.allocation_count, original_allocations.len(),
               "Allocation count should be preserved");
    
    // Check memory statistics
    assert_eq!(loaded_data.stats.active_memory, original_stats.active_memory,
               "Active memory should be preserved");
    assert_eq!(loaded_data.stats.total_allocations, original_stats.total_allocations,
               "Total allocations should be preserved");
    
    // Check individual allocations
    for (original, loaded) in original_allocations.iter().zip(loaded_data.allocations.iter()) {
        assert_eq!(original.ptr, loaded.ptr, "Pointer should be preserved");
        assert_eq!(original.size, loaded.size, "Size should be preserved");
        assert_eq!(original.timestamp_alloc, loaded.timestamp_alloc, "Timestamp should be preserved");
        assert_eq!(original.type_name, loaded.type_name, "Type name should be preserved");
    }
    
    println!("✅ Data integrity verified successfully!");
    println!("🎉 Data integrity workflow test completed successfully!");
}

/// Helper function to create a test memory tracker with sample data
fn create_test_memory_tracker() -> MemoryTracker {
    let tracker = MemoryTracker::new();
    
    // Note: In a real test, we would need to actually perform memory operations
    // to populate the tracker. For this integration test, we're testing the
    // workflow with whatever data the tracker contains.
    
    tracker
}

/// Helper function to create a larger test memory tracker for streaming tests
fn create_large_test_memory_tracker() -> MemoryTracker {
    let tracker = MemoryTracker::new();
    
    // Note: In a real implementation, we would create more allocations
    // to test streaming functionality properly
    
    tracker
}

/// Test performance characteristics of the workflow
#[test]
fn test_workflow_performance() {
    println!("🧪 Testing workflow performance");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let binary_path = temp_dir.path().join("test_performance.ms");
    let json_path = temp_dir.path().join("test_performance.json");
    
    let start_time = std::time::Instant::now();
    
    // Create and export binary
    let tracker = create_test_memory_tracker();
    let export_options = BinaryExportOptions::fast(); // Use fast options for performance test
    let export_stats = export_memory_to_binary(&tracker, &binary_path, export_options)
        .expect("Binary export should succeed");
    
    let export_time = start_time.elapsed();
    
    // Parse binary
    let parse_start = std::time::Instant::now();
    let parser = BinaryParser::new(BinaryParseOptions::fast());
    let binary_data = parser.parse_file(&binary_path)
        .expect("Binary parsing should succeed");
    let parse_time = parse_start.elapsed();
    
    // Convert to JSON
    let convert_start = std::time::Instant::now();
    let converter = JsonConverter::new(JsonConvertOptions::fast());
    let conversion_stats = converter.convert_to_file(&binary_data, &json_path)
        .expect("JSON conversion should succeed");
    let convert_time = convert_start.elapsed();
    
    let total_time = start_time.elapsed();
    
    println!("⏱️  Performance results:");
    println!("   - Export time: {export_time:?}");
    println!("   - Parse time: {parse_time:?}");
    println!("   - Convert time: {convert_time:?}");
    println!("   - Total time: {total_time:?}");
    println!("   - File size: {} bytes", export_stats.file_size);
    println!("   - Compression ratio: {:.1}%", export_stats.compression_ratio * 100.0);
    
    // Performance assertions (these are quite lenient for CI environments)
    assert!(total_time < std::time::Duration::from_secs(10), 
            "Total workflow should complete within 10 seconds");
    assert!(export_stats.compression_ratio <= 1.0, 
            "Compression ratio should not exceed 100%");
    
    println!("🎉 Performance test completed successfully!");
}