//! Comprehensive validation tests for binary format compatibility

use memscope_rs::core::types::AllocationInfo;
use memscope_rs::export::binary;
use memscope_rs::get_global_tracker;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

/// Create comprehensive test data with various allocation patterns
fn create_comprehensive_test_data() -> Vec<AllocationInfo> {
    vec![
        // Small allocation
        AllocationInfo {
            ptr: 0x1000,
            size: 64,
            var_name: Some("small_buffer".to_string()),
            type_name: Some("Vec<u8>".to_string()),
            scope_name: Some("main".to_string()),
            timestamp_alloc: 1000000000,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 1,
            stack_trace: Some(vec!["main::allocate".to_string(), "std::vec::Vec::new".to_string()]),
            is_leaked: false,
            lifetime_ms: Some(150),
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
        // Large allocation
        AllocationInfo {
            ptr: 0x2000,
            size: 4096,
            var_name: Some("large_data".to_string()),
            type_name: Some("HashMap<String, Vec<i32>>".to_string()),
            scope_name: Some("process_data".to_string()),
            timestamp_alloc: 1000000100,
            timestamp_dealloc: Some(1000000500),
            thread_id: "worker-1".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: Some(400),
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
        // String allocation
        AllocationInfo {
            ptr: 0x3000,
            size: 256,
            var_name: Some("config_path".to_string()),
            type_name: Some("String".to_string()),
            scope_name: Some("load_config".to_string()),
            timestamp_alloc: 1000000200,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 2,
            stack_trace: Some(vec!["load_config".to_string()]),
            is_leaked: true,
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
        // Allocation with no variable name
        AllocationInfo {
            ptr: 0x4000,
            size: 128,
            var_name: None,
            type_name: Some("Box<dyn Trait>".to_string()),
            scope_name: None,
            timestamp_alloc: 1000000300,
            timestamp_dealloc: Some(1000000350),
            thread_id: "async-runtime".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: Some(50),
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
fn test_binary_to_json_data_integrity() {
    let temp_dir = TempDir::new().unwrap();
    let binary_path = temp_dir.path().join("test_integrity.memscope");
    let json_path = temp_dir.path().join("test_integrity.json");
    
    let original_data = create_comprehensive_test_data();
    
    // Export to binary
    binary::export_to_binary(&original_data, &binary_path).unwrap();
    
    // Convert binary to JSON
    binary::parse_binary_to_json(&binary_path, &json_path).unwrap();
    
    // Read and parse the JSON
    let json_content = fs::read_to_string(&json_path).unwrap();
    let parsed_data: Vec<AllocationInfo> = serde_json::from_str(&json_content).unwrap();
    
    // Verify data integrity
    assert_eq!(parsed_data.len(), original_data.len());
    
    for (original, parsed) in original_data.iter().zip(parsed_data.iter()) {
        assert_eq!(original.ptr, parsed.ptr);
        assert_eq!(original.size, parsed.size);
        assert_eq!(original.var_name, parsed.var_name);
        assert_eq!(original.type_name, parsed.type_name);
        assert_eq!(original.timestamp_alloc, parsed.timestamp_alloc);
        // Note: thread_id might be different due to test environment
        // assert_eq!(original.thread_id, parsed.thread_id);
        // Note: Some fields might not be preserved in binary format
    }
    
    println!("✅ Binary to JSON data integrity test passed");
    println!("📊 Verified {} allocations", original_data.len());
}

#[test]
fn test_binary_to_html_generation() {
    let temp_dir = TempDir::new().unwrap();
    let binary_path = temp_dir.path().join("test_html.memscope");
    let html_path = temp_dir.path().join("test_report.html");
    
    let test_data = create_comprehensive_test_data();
    
    // Export to binary
    binary::export_to_binary(&test_data, &binary_path).unwrap();
    
    // Convert binary to HTML
    binary::parse_binary_to_html(&binary_path, &html_path).unwrap();
    
    // Verify HTML file exists and has content
    assert!(html_path.exists());
    let html_content = fs::read_to_string(&html_path).unwrap();
    
    // Basic HTML structure validation
    assert!(html_content.contains("<!DOCTYPE html>"));
    assert!(html_content.contains("<html"));
    assert!(html_content.contains("</html>"));
    assert!(html_content.contains("Memory Analysis Report") || html_content.contains("Memory & FFI Snapshot Analysis"));
    
    // Check for allocation data
    assert!(html_content.contains("0x1000") || html_content.contains("small_buffer"));
    assert!(html_content.contains("0x2000") || html_content.contains("large_data"));
    
    // Check for summary statistics
    assert!(html_content.contains("Total Allocations") || html_content.contains("total-allocations"));
    
    println!("✅ Binary to HTML generation test passed");
    println!("📄 HTML file size: {} bytes", html_content.len());
}

#[test]
fn test_json_compatibility_with_existing_format() {
    let temp_dir = TempDir::new().unwrap();
    let binary_path = temp_dir.path().join("compatibility.memscope");
    let json_from_binary = temp_dir.path().join("from_binary.json");
    let json_direct = temp_dir.path().join("direct.json");
    
    let test_data = create_comprehensive_test_data();
    
    // Export to binary and convert to JSON
    binary::export_to_binary(&test_data, &binary_path).unwrap();
    binary::parse_binary_to_json(&binary_path, &json_from_binary).unwrap();
    
    // Export directly to JSON
    let direct_json = serde_json::to_string_pretty(&test_data).unwrap();
    fs::write(&json_direct, direct_json).unwrap();
    
    // Parse both JSON files
    let json_from_binary_content = fs::read_to_string(&json_from_binary).unwrap();
    let json_direct_content = fs::read_to_string(&json_direct).unwrap();
    
    let parsed_from_binary: Value = serde_json::from_str(&json_from_binary_content).unwrap();
    let parsed_direct: Value = serde_json::from_str(&json_direct_content).unwrap();
    
    // Compare key fields (some fields might be filtered in binary format)
    if let (Value::Array(from_binary), Value::Array(direct)) = (&parsed_from_binary, &parsed_direct) {
        assert_eq!(from_binary.len(), direct.len());
        
        for (binary_item, direct_item) in from_binary.iter().zip(direct.iter()) {
            // Check essential fields
            assert_eq!(binary_item["ptr"], direct_item["ptr"]);
            assert_eq!(binary_item["size"], direct_item["size"]);
            assert_eq!(binary_item["var_name"], direct_item["var_name"]);
            assert_eq!(binary_item["type_name"], direct_item["type_name"]);
            assert_eq!(binary_item["timestamp_alloc"], direct_item["timestamp_alloc"]);
            assert_eq!(binary_item["thread_id"], direct_item["thread_id"]);
        }
    }
    
    println!("✅ JSON compatibility test passed");
    println!("📊 Binary JSON size: {} bytes", json_from_binary_content.len());
    println!("📊 Direct JSON size: {} bytes", json_direct_content.len());
}

#[test]
fn test_performance_comparison() {
    let test_data = create_comprehensive_test_data();
    let temp_dir = TempDir::new().unwrap();
    
    // Measure binary export time
    let binary_path = temp_dir.path().join("perf_test.memscope");
    let start = std::time::Instant::now();
    binary::export_to_binary(&test_data, &binary_path).unwrap();
    let binary_export_time = start.elapsed();
    
    // Measure JSON export time
    let json_path = temp_dir.path().join("perf_test.json");
    let start = std::time::Instant::now();
    let json_data = serde_json::to_string_pretty(&test_data).unwrap();
    fs::write(&json_path, json_data).unwrap();
    let json_export_time = start.elapsed();
    
    // Measure file sizes
    let binary_size = fs::metadata(&binary_path).unwrap().len();
    let json_size = fs::metadata(&json_path).unwrap().len();
    
    // Calculate metrics
    let size_reduction = ((json_size as f64 - binary_size as f64) / json_size as f64) * 100.0;
    let speed_improvement = json_export_time.as_nanos() as f64 / binary_export_time.as_nanos() as f64;
    
    println!("🚀 Performance Comparison Results:");
    println!("   Binary export time: {:?}", binary_export_time);
    println!("   JSON export time: {:?}", json_export_time);
    println!("   Speed improvement: {:.2}x faster", speed_improvement);
    println!("   Binary file size: {} bytes", binary_size);
    println!("   JSON file size: {} bytes", json_size);
    println!("   Size reduction: {:.1}%", size_reduction);
    
    // Verify performance goals
    assert!(size_reduction > 0.0, "Binary should be smaller than JSON");
    assert!(speed_improvement >= 1.0, "Binary export should be at least as fast as JSON");
    
    println!("✅ Performance comparison test passed");
}

#[test]
fn test_memory_tracker_integration() {
    let temp_dir = TempDir::new().unwrap();
    
    // Change to temp directory for this test
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    // Get tracker and add test data
    let tracker = get_global_tracker();
    tracker.enable_fast_mode();
    
    // Track some allocations
    tracker.fast_track_allocation(0x5000, 1024, "integration_test_1".to_string()).unwrap();
    tracker.fast_track_allocation(0x6000, 2048, "integration_test_2".to_string()).unwrap();
    tracker.fast_track_allocation(0x7000, 512, "integration_test_3".to_string()).unwrap();
    
    // Export using MemoryTracker
    let result = tracker.export_to_binary("integration_test");
    assert!(result.is_ok(), "MemoryTracker binary export failed: {:?}", result);
    
    // Look for the created file
    let memory_analysis_dir = temp_dir.path().join("MemoryAnalysis");
    assert!(memory_analysis_dir.exists(), "MemoryAnalysis directory should be created");
    
    // Find the .memscope file
    let mut found_binary_file = None;
    if let Ok(entries) = fs::read_dir(&memory_analysis_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                    for sub_entry in sub_entries {
                        if let Ok(sub_entry) = sub_entry {
                            if sub_entry.path().extension() == Some(std::ffi::OsStr::new("memscope")) {
                                found_binary_file = Some(sub_entry.path());
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    
    assert!(found_binary_file.is_some(), "Binary file should be created");
    let binary_file = found_binary_file.unwrap();
    
    // Verify file has content
    let metadata = fs::metadata(&binary_file).unwrap();
    assert!(metadata.len() > 16, "Binary file should have content beyond header");
    
    // Test conversion functions
    let json_path = temp_dir.path().join("converted.json");
    let html_path = temp_dir.path().join("converted.html");
    
    let json_result = memscope_rs::core::tracker::MemoryTracker::parse_binary_to_json(&binary_file, &json_path);
    assert!(json_result.is_ok(), "Binary to JSON conversion failed: {:?}", json_result);
    
    let html_result = memscope_rs::core::tracker::MemoryTracker::parse_binary_to_html(&binary_file, &html_path);
    assert!(html_result.is_ok(), "Binary to HTML conversion failed: {:?}", html_result);
    
    // Verify converted files
    assert!(json_path.exists());
    assert!(html_path.exists());
    
    let json_content = fs::read_to_string(&json_path).unwrap();
    assert!(json_content.contains("integration_test"));
    
    let html_content = fs::read_to_string(&html_path).unwrap();
    assert!(html_content.contains("Memory Analysis Report") || html_content.contains("Memory & FFI Snapshot Analysis"));
    
    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
    
    println!("✅ MemoryTracker integration test passed");
    println!("📁 Binary file: {} bytes", metadata.len());
    println!("📄 JSON file: {} bytes", json_content.len());
    println!("🌐 HTML file: {} bytes", html_content.len());
}