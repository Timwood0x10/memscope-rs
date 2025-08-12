//! Performance validation tests
//! 
//! Validates ultra-high performance binary parsing optimization.
//! Tests millisecond-level performance targets for different dataset sizes.

use memscope_rs::{track_var, get_global_tracker};
use memscope_rs::export::binary::BinaryParser;
use std::time::Instant;

#[test]
fn test_small_dataset_performance_target() {
    // Test small dataset performance (target: <100ms)
    create_test_allocation_dataset(100);
    
    let binary_file_path = "MemoryAnalysis/perf_test_small.memscope";
    ensure_output_directory_exists();
    
    let tracker = get_global_tracker();
    match tracker.export_to_binary(binary_file_path) {
        Ok(_) => {
            // Measure parsing performance
            let parse_start_time = Instant::now();
            let parse_result = BinaryParser::parse_full_binary_to_json_with_existing_optimizations(
                binary_file_path,
                "perf_test_small"
            );
            let parse_duration = parse_start_time.elapsed();
            
            match parse_result {
                Ok(_) => {
                    let parse_time_ms = parse_duration.as_millis();
                    if parse_time_ms > 100 {
                        panic!("Performance target missed: {}ms > 100ms", parse_time_ms);
                    }
                }
                Err(e) => {
                    panic!("Small dataset parsing failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Binary export failed: {:?}", e);
        }
    }
    
    cleanup_test_artifacts(binary_file_path, "perf_test_small");
}

#[test]
fn test_medium_dataset_performance_target() {
    // Test medium dataset performance (target: <500ms)
    create_test_allocation_dataset(1000);
    
    let binary_file_path = "MemoryAnalysis/perf_test_medium.memscope";
    ensure_output_directory_exists();
    
    let tracker = get_global_tracker();
    match tracker.export_to_binary(binary_file_path) {
        Ok(_) => {
            // Measure parsing performance
            let parse_start_time = Instant::now();
            let parse_result = BinaryParser::parse_full_binary_to_json_with_existing_optimizations(
                binary_file_path,
                "perf_test_medium"
            );
            let parse_duration = parse_start_time.elapsed();
            
            match parse_result {
                Ok(_) => {
                    let parse_time_ms = parse_duration.as_millis();
                    if parse_time_ms > 500 {
                        panic!("Performance target missed: {}ms > 500ms", parse_time_ms);
                    }
                }
                Err(e) => {
                    panic!("Medium dataset parsing failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Binary export failed: {:?}", e);
        }
    }
    
    cleanup_test_artifacts(binary_file_path, "perf_test_medium");
}

#[test]
fn test_user_vs_full_binary_performance_consistency() {
    // Test performance consistency between user and full binary modes
    create_test_allocation_dataset(500);
    
    let user_binary_path = "MemoryAnalysis/consistency_test_user.memscope";
    let full_binary_path = "MemoryAnalysis/consistency_test_full.memscope";
    ensure_output_directory_exists();
    
    let tracker = get_global_tracker();
    
    // Export both binary formats
    if let Err(e) = tracker.export_user_binary(user_binary_path) {
        panic!("User binary export failed: {:?}", e);
    }
    
    if let Err(e) = tracker.export_full_binary(full_binary_path) {
        panic!("Full binary export failed: {:?}", e);
    }
    
    // Measure user binary parsing performance
    let user_parse_start = Instant::now();
    let user_parse_result = BinaryParser::parse_user_binary_to_json(
        user_binary_path, 
        "consistency_test_user"
    );
    let user_parse_duration = user_parse_start.elapsed();
    
    // Measure full binary parsing performance
    let full_parse_start = Instant::now();
    let full_parse_result = BinaryParser::parse_full_binary_to_json_with_existing_optimizations(
        full_binary_path, 
        "consistency_test_full"
    );
    let full_parse_duration = full_parse_start.elapsed();
    
    // Validate both operations succeeded
    if let Err(e) = user_parse_result {
        panic!("User binary parsing failed: {:?}", e);
    }
    
    if let Err(e) = full_parse_result {
        panic!("Full binary parsing failed: {:?}", e);
    }
    
    // Verify performance consistency (should be similar due to unified optimization)
    let user_time_ms = user_parse_duration.as_millis();
    let full_time_ms = full_parse_duration.as_millis();
    let performance_ratio = if user_time_ms > 0 {
        full_time_ms as f64 / user_time_ms as f64
    } else {
        1.0
    };
    
    if performance_ratio > 10.0 {
        panic!("Performance inconsistency: full/user ratio {:.1}x too high", performance_ratio);
    }
    
    cleanup_test_artifacts(user_binary_path, "consistency_test_user");
    cleanup_test_artifacts(full_binary_path, "consistency_test_full");
}

/// Create test allocation dataset with specified size
fn create_test_allocation_dataset(allocation_count: usize) {
    for index in 0..allocation_count {
        let allocation_size = 64 + (index % 256);
        let test_allocation = vec![index as u8; allocation_size];
        track_var!(test_allocation);
        drop(test_allocation);
    }
}

/// Ensure output directory exists for test artifacts
fn ensure_output_directory_exists() {
    if let Err(e) = std::fs::create_dir_all("MemoryAnalysis") {
        panic!("Failed to create output directory: {:?}", e);
    }
}

/// Clean up test artifacts after test completion
fn cleanup_test_artifacts(binary_file_path: &str, output_name: &str) {
    std::fs::remove_file(binary_file_path).ok();
    std::fs::remove_dir_all(format!("MemoryAnalysis/{}", output_name)).ok();
}