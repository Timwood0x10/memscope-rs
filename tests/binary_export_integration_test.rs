//! Integration test for binary export functionality

use memscope_rs::core::types::AllocationInfo;
use memscope_rs::export::binary;
use memscope_rs::get_global_tracker;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_binary_export_direct() {
    let temp_dir = TempDir::new().unwrap();
    let binary_path = temp_dir.path().join("test_direct.memscope");
    let json_path = temp_dir.path().join("test_direct.json");
    
    // Create test allocation data directly
    let allocations = vec![
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
    ];
    
    // Export to binary using direct function
    let result = binary::export_to_binary(&allocations, &binary_path);
    assert!(result.is_ok(), "Binary export failed: {:?}", result);
    
    // Verify binary file exists and has content
    assert!(binary_path.exists());
    let metadata = fs::metadata(&binary_path).unwrap();
    assert!(metadata.len() > 16, "Binary file too small: {} bytes", metadata.len());
    
    // Convert binary to JSON
    let result = binary::parse_binary_to_json(&binary_path, &json_path);
    assert!(result.is_ok(), "Binary to JSON conversion failed: {:?}", result);
    
    // Verify JSON file exists and has content
    assert!(json_path.exists());
    let json_content = fs::read_to_string(&json_path).unwrap();
    assert!(!json_content.is_empty());
    assert!(json_content.contains("buffer") || json_content.contains("data"));
    
    println!("✅ Binary export direct test passed");
    println!("📁 Binary file size: {} bytes", metadata.len());
    println!("📄 JSON content preview: {}", &json_content[..json_content.len().min(200)]);
}

#[test]
fn test_binary_export_integration() {
    let temp_dir = TempDir::new().unwrap();
    
    // Use a simple filename that won't trigger complex path handling
    let result = std::env::set_current_dir(&temp_dir);
    assert!(result.is_ok(), "Failed to change directory");
    
    // Get tracker and add some test data
    let tracker = get_global_tracker();
    tracker.enable_fast_mode();
    
    // Track some allocations
    tracker.fast_track_allocation(0x1000, 1024, "buffer".to_string()).unwrap();
    tracker.fast_track_allocation(0x2000, 512, "data".to_string()).unwrap();
    
    // Export to binary with simple name
    let result = tracker.export_to_binary("simple_test");
    assert!(result.is_ok(), "Binary export failed: {:?}", result);
    
    // Look for the created file in MemoryAnalysis directory
    let memory_analysis_dir = temp_dir.path().join("MemoryAnalysis");
    println!("Looking for MemoryAnalysis directory at: {:?}", memory_analysis_dir);
    
    if memory_analysis_dir.exists() {
        println!("MemoryAnalysis directory found");
        if let Ok(entries) = fs::read_dir(&memory_analysis_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("Found subdirectory: {:?}", entry.path());
                    if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                        for sub_entry in sub_entries {
                            if let Ok(sub_entry) = sub_entry {
                                println!("Found file: {:?}", sub_entry.path());
                                if sub_entry.path().extension() == Some(std::ffi::OsStr::new("memscope")) {
                                    let metadata = fs::metadata(sub_entry.path()).unwrap();
                                    println!("✅ Found .memscope file with {} bytes", metadata.len());
                                    assert!(metadata.len() > 16);
                                    return; // Test passed
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    panic!("No .memscope file found in MemoryAnalysis directory");
}

#[test]
fn test_binary_to_html_conversion() {
    let temp_dir = TempDir::new().unwrap();
    let binary_path = temp_dir.path().join("test_html.memscope");
    let html_path = temp_dir.path().join("test_report.html");
    
    // Create test allocation data directly and export using direct function
    let allocations = vec![
        AllocationInfo {
            ptr: 0x3000,
            size: 2048,
            var_name: Some("html_test".to_string()),
            type_name: Some("Vec<u8>".to_string()),
            scope_name: None,
            timestamp_alloc: 1234567892,
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
    ];
    
    // Export to binary using direct function
    binary::export_to_binary(&allocations, &binary_path).unwrap();
    
    // Convert binary to HTML using direct function
    let result = binary::parse_binary_to_html(&binary_path, &html_path);
    assert!(result.is_ok(), "Binary to HTML conversion failed: {:?}", result);
    
    // Verify HTML file exists and has content
    assert!(html_path.exists());
    let html_content = fs::read_to_string(&html_path).unwrap();
    assert!(!html_content.is_empty());
    assert!(html_content.contains("<html>"));
    assert!(html_content.contains("Memory Analysis Report"));
    assert!(html_content.contains("0x3000"));
    
    println!("✅ Binary to HTML conversion test passed");
}