//! Core functionality tests
//! 
//! Tests essential memscope-rs functionality to ensure proper operation.
//! Validates basic tracking, memory statistics, and binary export capabilities.

use memscope_rs::{track_var, get_global_tracker};
use std::path::Path;

#[test]
fn test_basic_variable_tracking() {
    // Test fundamental variable tracking functionality
    let test_data = vec![1, 2, 3, 4, 5];
    track_var!(test_data);
    
    // Verify tracker captures allocation data
    let tracker = get_global_tracker();
    let stats_result = tracker.get_stats();
    
    match stats_result {
        Ok(stats) => {
            assert!(stats.total_allocations > 0, "Should record allocations");
        }
        Err(e) => {
            panic!("Failed to retrieve tracking statistics: {:?}", e);
        }
    }
    
    drop(test_data);
}

#[test]
fn test_multiple_allocation_tracking() {
    // Test tracking across multiple allocations
    for iteration in 0..10 {
        let allocation_data = vec![iteration; 100];
        track_var!(allocation_data);
        drop(allocation_data);
    }
    
    // Verify multiple allocations are tracked
    let tracker = get_global_tracker();
    match tracker.get_stats() {
        Ok(stats) => {
            assert!(stats.total_allocations >= 10, "Should track multiple allocations");
        }
        Err(e) => {
            panic!("Statistics retrieval failed: {:?}", e);
        }
    }
}

#[test]
fn test_diverse_data_type_tracking() {
    // Test tracking of various data types
    let string_allocation = String::from("test string data");
    track_var!(string_allocation);
    
    let vector_allocation = vec![1u32, 2, 3, 4];
    track_var!(vector_allocation);
    
    let buffer_allocation = vec![0u8; 1024];
    track_var!(buffer_allocation);
    
    // Verify all data types are tracked
    let tracker = get_global_tracker();
    match tracker.get_stats() {
        Ok(stats) => {
            assert!(stats.total_allocations >= 3, "Should track diverse data types");
        }
        Err(e) => {
            panic!("Failed to get tracking statistics: {:?}", e);
        }
    }
    
    drop(string_allocation);
    drop(vector_allocation);
    drop(buffer_allocation);
}

#[test]
fn test_binary_export_functionality() {
    // Test binary export capability
    let export_test_data = vec![42u8; 256];
    track_var!(export_test_data);
    drop(export_test_data);
    
    // Ensure output directory exists
    if let Err(e) = std::fs::create_dir_all("MemoryAnalysis") {
        panic!("Failed to create output directory: {:?}", e);
    }
    
    let tracker = get_global_tracker();
    let binary_file_path = "MemoryAnalysis/test_export.memscope";
    
    // Perform binary export
    match tracker.export_to_binary(binary_file_path) {
        Ok(_) => {
            // Verify exported file exists
            if !Path::new(binary_file_path).exists() {
                panic!("Binary export file was not created: {}", binary_file_path);
            }
        }
        Err(e) => {
            panic!("Binary export operation failed: {:?}", e);
        }
    }
    
    // Cleanup test file
    std::fs::remove_file(binary_file_path).ok();
}

#[test]
fn test_memory_statistics_accuracy() {
    // Test memory statistics calculation accuracy
    let large_allocation = vec![0u8; 10240]; // 10KB allocation
    track_var!(large_allocation);
    
    let tracker = get_global_tracker();
    match tracker.get_stats() {
        Ok(stats) => {
            assert!(stats.total_allocated > 10000, "Should track large allocation size");
            assert!(stats.total_allocations > 0, "Should record allocation count");
        }
        Err(e) => {
            panic!("Memory statistics calculation failed: {:?}", e);
        }
    }
    
    drop(large_allocation);
}