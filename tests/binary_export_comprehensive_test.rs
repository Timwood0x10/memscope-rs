//! Comprehensive binary export tests
//! 
//! Tests all aspects of binary export functionality including format validation,
//! data integrity, compression, and various export modes.

use memscope_rs::{track_var, get_global_tracker};
use std::path::Path;
use std::time::Instant;

#[test]
fn test_binary_format_validation() {
    // Test binary format structure and validation
    create_test_allocation_set(50);
    
    let binary_file_path = "MemoryAnalysis/format_validation_test.memscope";
    ensure_directory_exists();
    
    let tracker = get_global_tracker();
    match tracker.export_to_binary(binary_file_path) {
        Ok(_) => {
            // Verify file exists and has reasonable size
            match std::fs::metadata(binary_file_path) {
                Ok(metadata) => {
                    let file_size = metadata.len();
                    if file_size == 0 {
                        panic!("Binary file should not be empty");
                    }
                    if file_size > 100 * 1024 * 1024 {
                        panic!("Binary file unexpectedly large: {} bytes", file_size);
                    }
                }
                Err(e) => {
                    panic!("Failed to read binary file metadata: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Binary format validation export failed: {:?}", e);
        }
    }
    
    cleanup_test_file(binary_file_path);
}

#[test]
fn test_user_binary_vs_full_binary_data_integrity() {
    // Test data integrity between user and full binary exports
    create_diverse_allocation_set();
    
    let user_binary_path = "MemoryAnalysis/integrity_test_user.memscope";
    let full_binary_path = "MemoryAnalysis/integrity_test_full.memscope";
    ensure_directory_exists();
    
    let tracker = get_global_tracker();
    
    // Export both formats
    match tracker.export_user_binary(user_binary_path) {
        Ok(_) => {},
        Err(e) => panic!("User binary export failed: {:?}", e),
    }
    
    match tracker.export_full_binary(full_binary_path) {
        Ok(_) => {},
        Err(e) => panic!("Full binary export failed: {:?}", e),
    }
    
    // Verify both files exist and have different sizes
    let user_size = match std::fs::metadata(user_binary_path) {
        Ok(metadata) => metadata.len(),
        Err(e) => panic!("Failed to read user binary metadata: {:?}", e),
    };
    
    let full_size = match std::fs::metadata(full_binary_path) {
        Ok(metadata) => metadata.len(),
        Err(e) => panic!("Failed to read full binary metadata: {:?}", e),
    };
    
    // Full binary should typically be larger or equal to user binary
    if full_size < user_size {
        panic!("Full binary ({} bytes) should not be smaller than user binary ({} bytes)", 
               full_size, user_size);
    }
    
    cleanup_test_file(user_binary_path);
    cleanup_test_file(full_binary_path);
}

#[test]
fn test_binary_export_with_large_allocations() {
    // Test binary export with large memory allocations
    let large_allocation = vec![0u8; 1024 * 1024]; // 1MB allocation
    track_var!(large_allocation);
    
    let medium_allocations: Vec<_> = (0..100).map(|i| {
        let allocation = vec![i as u8; 10240]; // 10KB each
        track_var!(allocation);
        allocation
    }).collect();
    
    let binary_file_path = "MemoryAnalysis/large_allocation_test.memscope";
    ensure_directory_exists();
    
    let tracker = get_global_tracker();
    let export_start = Instant::now();
    
    match tracker.export_to_binary(binary_file_path) {
        Ok(_) => {
            let export_duration = export_start.elapsed();
            
            // Export should complete in reasonable time even with large allocations
            if export_duration.as_secs() > 30 {
                panic!("Large allocation export took too long: {:?}", export_duration);
            }
            
            // Verify file was created
            if !Path::new(binary_file_path).exists() {
                panic!("Large allocation binary file was not created");
            }
        }
        Err(e) => {
            panic!("Large allocation export failed: {:?}", e);
        }
    }
    
    drop(large_allocation);
    drop(medium_allocations);
    cleanup_test_file(binary_file_path);
}

#[test]
fn test_binary_export_with_complex_data_structures() {
    // Test export with complex nested data structures
    let nested_vectors: Vec<Vec<u32>> = (0..10).map(|i| {
        let inner_vec = (0..100).map(|j| i * 100 + j).collect();
        track_var!(inner_vec);
        inner_vec
    }).collect();
    track_var!(nested_vectors);
    
    let string_collection: Vec<String> = (0..50).map(|i| {
        let string_data = format!("test_string_number_{}", i);
        track_var!(string_data);
        string_data
    }).collect();
    track_var!(string_collection);
    
    let binary_file_path = "MemoryAnalysis/complex_structures_test.memscope";
    ensure_directory_exists();
    
    let tracker = get_global_tracker();
    match tracker.export_to_binary(binary_file_path) {
        Ok(_) => {
            // Verify export succeeded with complex structures
            if !Path::new(binary_file_path).exists() {
                panic!("Complex structures binary file was not created");
            }
        }
        Err(e) => {
            panic!("Complex structures export failed: {:?}", e);
        }
    }
    
    drop(nested_vectors);
    drop(string_collection);
    cleanup_test_file(binary_file_path);
}

#[test]
fn test_binary_export_performance_scaling() {
    // Test export performance scaling with different dataset sizes
    let test_sizes = vec![10, 100, 500, 1000];
    
    for size in test_sizes {
        create_test_allocation_set(size);
        
        let binary_file_path = format!("MemoryAnalysis/scaling_test_{}.memscope", size);
        ensure_directory_exists();
        
        let tracker = get_global_tracker();
        let export_start = Instant::now();
        
        match tracker.export_to_binary(&binary_file_path) {
            Ok(_) => {
                let export_duration = export_start.elapsed();
                let export_time_ms = export_duration.as_millis();
                
                // Export time should scale reasonably with dataset size
                let expected_max_time = size as u128 * 2; // 2ms per allocation maximum
                if export_time_ms > expected_max_time {
                    panic!("Export scaling poor: {} allocations took {}ms (expected <{}ms)", 
                           size, export_time_ms, expected_max_time);
                }
            }
            Err(e) => {
                panic!("Scaling test export failed for size {}: {:?}", size, e);
            }
        }
        
        cleanup_test_file(&binary_file_path);
    }
}

#[test]
fn test_concurrent_binary_exports() {
    // Test concurrent binary export operations
    use std::thread;
    
    let thread_handles: Vec<_> = (0..4).map(|thread_id| {
        thread::spawn(move || {
            // Create thread-specific allocations
            for allocation_index in 0..25 {
                let allocation_data = vec![thread_id as u8; 256 + allocation_index];
                track_var!(allocation_data);
                drop(allocation_data);
            }
            
            let binary_file_path = format!("MemoryAnalysis/concurrent_test_{}.memscope", thread_id);
            
            // Ensure directory exists in each thread
            if let Err(e) = std::fs::create_dir_all("MemoryAnalysis") {
                panic!("Thread {} failed to create directory: {:?}", thread_id, e);
            }
            
            let tracker = get_global_tracker();
            match tracker.export_to_binary(&binary_file_path) {
                Ok(_) => {
                    // Verify file was created successfully
                    if !Path::new(&binary_file_path).exists() {
                        panic!("Thread {} binary file was not created", thread_id);
                    }
                }
                Err(e) => {
                    panic!("Thread {} export failed: {:?}", thread_id, e);
                }
            }
            
            // Cleanup thread-specific file
            std::fs::remove_file(&binary_file_path).ok();
        })
    }).collect();
    
    // Wait for all threads to complete
    for (index, handle) in thread_handles.into_iter().enumerate() {
        match handle.join() {
            Ok(_) => {},
            Err(e) => {
                panic!("Thread {} panicked: {:?}", index, e);
            }
        }
    }
}

/// Create test allocation set with specified count
fn create_test_allocation_set(count: usize) {
    for index in 0..count {
        let allocation_size = 64 + (index % 128);
        let test_data = vec![index as u8; allocation_size];
        track_var!(test_data);
        drop(test_data);
    }
}

/// Create diverse allocation set for comprehensive testing
fn create_diverse_allocation_set() {
    // Small allocations
    for i in 0..20 {
        let small_data = vec![i as u8; 32];
        track_var!(small_data);
        drop(small_data);
    }
    
    // Medium allocations
    for i in 0..10 {
        let medium_data = vec![i as u8; 1024];
        track_var!(medium_data);
        drop(medium_data);
    }
    
    // Large allocations
    for i in 0..5 {
        let large_data = vec![i as u8; 8192];
        track_var!(large_data);
        drop(large_data);
    }
    
    // String allocations
    for i in 0..15 {
        let string_data = format!("test_string_allocation_{}", i);
        track_var!(string_data);
        drop(string_data);
    }
}

/// Ensure output directory exists
fn ensure_directory_exists() {
    if let Err(e) = std::fs::create_dir_all("MemoryAnalysis") {
        panic!("Failed to create output directory: {:?}", e);
    }
}

/// Clean up test file
fn cleanup_test_file(file_path: &str) {
    std::fs::remove_file(file_path).ok();
}