//! Error resilience tests
//! 
//! Tests error handling capabilities and system resilience under various failure conditions.
//! Validates graceful error recovery without system crashes or data corruption.

use memscope_rs::export::binary::BinaryParser;
use std::path::Path;

#[test]
fn test_nonexistent_file_handling() {
    // Test handling of nonexistent binary files
    let nonexistent_file_path = "nonexistent_directory/missing_file.memscope";
    
    let parse_result = BinaryParser::parse_full_binary_to_json_with_existing_optimizations(
        nonexistent_file_path,
        "test_nonexistent"
    );
    
    // Should fail gracefully without panic
    match parse_result {
        Ok(_) => {
            panic!("Should not succeed with nonexistent file");
        }
        Err(_) => {
            // Expected behavior - graceful error handling
        }
    }
}

#[test]
fn test_corrupted_binary_file_handling() {
    // Test handling of corrupted binary files
    let corrupted_file_path = "MemoryAnalysis/test_corrupted.memscope";
    
    // Ensure output directory exists
    if let Err(e) = std::fs::create_dir_all("MemoryAnalysis") {
        panic!("Failed to create test directory: {:?}", e);
    }
    
    // Create corrupted file with invalid binary data
    if let Err(e) = std::fs::write(corrupted_file_path, b"invalid_binary_content") {
        panic!("Failed to create corrupted test file: {:?}", e);
    }
    
    let parse_result = BinaryParser::parse_full_binary_to_json_with_existing_optimizations(
        corrupted_file_path,
        "test_corrupted"
    );
    
    // Should handle corruption gracefully
    match parse_result {
        Ok(_) => {
            panic!("Should not succeed with corrupted file");
        }
        Err(_) => {
            // Expected behavior - graceful error handling
        }
    }
    
    // Cleanup test file
    std::fs::remove_file(corrupted_file_path).ok();
}

#[test]
fn test_empty_binary_file_handling() {
    // Test handling of empty binary files
    let empty_file_path = "MemoryAnalysis/test_empty.memscope";
    
    // Ensure output directory exists
    if let Err(e) = std::fs::create_dir_all("MemoryAnalysis") {
        panic!("Failed to create test directory: {:?}", e);
    }
    
    // Create empty file
    if let Err(e) = std::fs::write(empty_file_path, b"") {
        panic!("Failed to create empty test file: {:?}", e);
    }
    
    let parse_result = BinaryParser::parse_full_binary_to_json_with_existing_optimizations(
        empty_file_path,
        "test_empty"
    );
    
    // Should handle empty files gracefully
    match parse_result {
        Ok(_) => {
            panic!("Should not succeed with empty file");
        }
        Err(_) => {
            // Expected behavior - graceful error handling
        }
    }
    
    // Cleanup test file
    std::fs::remove_file(empty_file_path).ok();
}

#[test]
fn test_invalid_output_path_handling() {
    // Test handling of invalid output paths
    use memscope_rs::{track_var, get_global_tracker};
    
    // Create test data for export
    let test_data = vec![1, 2, 3];
    track_var!(test_data);
    drop(test_data);
    
    // Ensure output directory exists
    if let Err(e) = std::fs::create_dir_all("MemoryAnalysis") {
        panic!("Failed to create test directory: {:?}", e);
    }
    
    let tracker = get_global_tracker();
    let binary_file_path = "MemoryAnalysis/test_invalid_output.memscope";
    
    match tracker.export_to_binary(binary_file_path) {
        Ok(_) => {
            // Test parsing with problematic output path
            let parse_result = BinaryParser::parse_full_binary_to_json_with_existing_optimizations(
                binary_file_path,
                "invalid/path/with/problematic:characters"
            );
            
            // Should handle invalid paths gracefully
            match parse_result {
                Ok(_) => {
                    // May succeed on some systems - acceptable
                }
                Err(_) => {
                    // Expected behavior on systems with strict path validation
                }
            }
        }
        Err(e) => {
            panic!("Test setup failed - binary export error: {:?}", e);
        }
    }
    
    // Cleanup test file
    std::fs::remove_file(binary_file_path).ok();
}

#[test]
fn test_large_dataset_error_recovery() {
    // Test error recovery capabilities with larger datasets
    use memscope_rs::{track_var, get_global_tracker};
    
    // Create substantial test dataset
    for iteration in 0..1000 {
        let allocation_data = vec![iteration as u8; 512];
        track_var!(allocation_data);
        drop(allocation_data);
    }
    
    // Ensure output directory exists
    if let Err(e) = std::fs::create_dir_all("MemoryAnalysis") {
        panic!("Failed to create test directory: {:?}", e);
    }
    
    let tracker = get_global_tracker();
    let binary_file_path = "MemoryAnalysis/test_large_dataset.memscope";
    
    // Export large dataset
    match tracker.export_to_binary(binary_file_path) {
        Ok(_) => {
            // Verify file was created successfully
            if !Path::new(binary_file_path).exists() {
                panic!("Large dataset export file was not created");
            }
            
            // Test parsing large dataset
            let parse_result = BinaryParser::parse_full_binary_to_json_with_existing_optimizations(
                binary_file_path,
                "test_large_dataset"
            );
            
            match parse_result {
                Ok(_) => {
                    // Success - verify output files exist
                    let output_directory = "MemoryAnalysis/test_large_dataset";
                    if !Path::new(output_directory).exists() {
                        panic!("Output directory was not created for large dataset");
                    }
                }
                Err(e) => {
                    panic!("Large dataset parsing failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Large dataset export failed: {:?}", e);
        }
    }
    
    // Cleanup test artifacts
    std::fs::remove_file(binary_file_path).ok();
    std::fs::remove_dir_all("MemoryAnalysis/test_large_dataset").ok();
}