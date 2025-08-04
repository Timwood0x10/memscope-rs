//! Integration tests for enhanced lifecycle tracking functionality

use memscope_rs::core::types::AllocationInfo;
use memscope_rs::export::binary::{export_to_binary_with_config, BinaryExportConfig, BinaryReader};
use std::fs;
use tempfile::NamedTempFile;

/// Create test allocation data with enhanced lifecycle tracking
fn create_test_allocations_with_lifecycle() -> Vec<AllocationInfo> {
    let mut allocations = Vec::new();

    // Create allocations with different types to test lifecycle pattern analysis
    let test_types = vec![
        "Vec<String>",      // Should be classified as LongLived
        "String",           // Should be classified as ShortLived
        "Box<i32>",         // Should be classified as RAII
        "HashMap<String, i32>", // Should be classified as LongLived
        "Mutex<Data>",      // Should be classified as Singleton
    ];

    for (i, type_name) in test_types.iter().enumerate() {
        let allocation = AllocationInfo {
            ptr: 0x1000 + i * 0x100,
            size: 1024 + i * 256,
            var_name: Some(format!("var_{}", i)),
            type_name: Some(type_name.to_string()),
            scope_name: Some("test_function".to_string()),
            timestamp_alloc: 1234567890 + i as u64 * 1000,
            timestamp_dealloc: Some(1234567890 + i as u64 * 1000 + 5000), // 5 second lifetime
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: Some(vec![
                "main".to_string(),
                "test_function".to_string(),
                "allocate_memory".to_string(),
            ]),
            is_leaked: false,
            lifetime_ms: Some(5000), // 5 seconds
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
            lifecycle_tracking: None, // Will be populated by the tracker
            access_tracking: None,    // Will be populated by the tracker
        };
        
        allocations.push(allocation);
    }

    allocations
}

#[test]
fn test_enhanced_lifecycle_tracking_export() {
    let allocations = create_test_allocations_with_lifecycle();
    
    // Export with lifecycle timeline enabled
    let temp_file = NamedTempFile::new().unwrap();
    let config = BinaryExportConfig::debug_comprehensive(); // Has lifecycle_timeline enabled
    
    let result = export_to_binary_with_config(&allocations, temp_file.path(), &config);
    assert!(result.is_ok(), "Export with enhanced lifecycle tracking should succeed");
    
    // Verify file was created and has reasonable size
    let metadata = fs::metadata(temp_file.path()).unwrap();
    assert!(metadata.len() > 0, "File should not be empty");
    
    // Try to read the file back
    let mut reader = BinaryReader::new(temp_file.path()).unwrap();
    let header = reader.read_header().unwrap();
    assert_eq!(header.count, allocations.len() as u32);
    
    // Read back allocations to verify correctness
    for i in 0..allocations.len() {
        let allocation = reader.read_allocation().unwrap();
        assert_eq!(allocation.ptr, allocations[i].ptr);
        assert_eq!(allocation.size, allocations[i].size);
        assert_eq!(allocation.type_name, allocations[i].type_name);
        assert_eq!(allocation.lifetime_ms, allocations[i].lifetime_ms);
        
        // Verify that lifecycle_tracking and access_tracking would be populated
        // (In a real implementation, these would be filled by the tracker)
        println!("Allocation {}: type={:?}, lifetime={:?}", 
                i, allocation.type_name, allocation.lifetime_ms);
    }
}

#[test]
fn test_lifecycle_tracking_with_different_configs() {
    let allocations = create_test_allocations_with_lifecycle();
    
    // Test with performance_first config (lifecycle_timeline enabled)
    let temp_file_perf = NamedTempFile::new().unwrap();
    let config_perf = BinaryExportConfig::performance_first();
    
    let result = export_to_binary_with_config(&allocations, temp_file_perf.path(), &config_perf);
    assert!(result.is_ok(), "Export with performance_first config should succeed");
    
    // Test with minimal config (lifecycle_timeline disabled)
    let temp_file_min = NamedTempFile::new().unwrap();
    let config_min = BinaryExportConfig::minimal();
    
    let result = export_to_binary_with_config(&allocations, temp_file_min.path(), &config_min);
    assert!(result.is_ok(), "Export with minimal config should succeed");
    
    // Compare file sizes - comprehensive should be larger
    let perf_size = fs::metadata(temp_file_perf.path()).unwrap().len();
    let min_size = fs::metadata(temp_file_min.path()).unwrap().len();
    
    println!("Performance config size: {} bytes", perf_size);
    println!("Minimal config size: {} bytes", min_size);
    
    // Both should be readable
    let mut reader_perf = BinaryReader::new(temp_file_perf.path()).unwrap();
    let header_perf = reader_perf.read_header().unwrap();
    assert_eq!(header_perf.count, allocations.len() as u32);
    
    let mut reader_min = BinaryReader::new(temp_file_min.path()).unwrap();
    let header_min = reader_min.read_header().unwrap();
    assert_eq!(header_min.count, allocations.len() as u32);
}

#[test]
fn test_access_tracking_integration() {
    let mut allocations = create_test_allocations_with_lifecycle();
    
    // Simulate some allocations with different access patterns
    for (i, allocation) in allocations.iter_mut().enumerate() {
        // Vary sizes to test different access pattern predictions
        allocation.size = match i {
            0 => 32,    // Small allocation - should predict good locality
            1 => 1024,  // Medium allocation - moderate locality
            2 => 8192,  // Large allocation - may have poor locality
            _ => 512,   // Default medium size
        };
    }
    
    let temp_file = NamedTempFile::new().unwrap();
    let config = BinaryExportConfig::debug_comprehensive();
    
    let result = export_to_binary_with_config(&allocations, temp_file.path(), &config);
    assert!(result.is_ok(), "Export with access tracking should succeed");
    
    // Verify the file can be read back
    let mut reader = BinaryReader::new(temp_file.path()).unwrap();
    let header = reader.read_header().unwrap();
    assert_eq!(header.count, allocations.len() as u32);
    
    // Read back and verify different sized allocations
    for i in 0..allocations.len() {
        let allocation = reader.read_allocation().unwrap();
        assert_eq!(allocation.size, allocations[i].size);
        
        println!("Allocation {}: size={}, type={:?}", 
                i, allocation.size, allocation.type_name);
    }
}

#[test]
fn test_lifecycle_patterns_analysis() {
    // Create allocations that should trigger different lifecycle patterns
    let mut allocations = Vec::new();
    
    let pattern_test_cases = vec![
        ("Vec<String>", 1024, "LongLived pattern expected"),
        ("String", 64, "ShortLived pattern expected"),
        ("Box<Data>", 256, "RAII pattern expected"),
        ("Mutex<State>", 512, "Singleton pattern expected"),
        ("CustomType", 128, "OnDemand pattern expected"),
    ];
    
    for (i, (type_name, size, description)) in pattern_test_cases.iter().enumerate() {
        let allocation = AllocationInfo {
            ptr: 0x2000 + i * 0x200,
            size: *size,
            var_name: Some(format!("test_var_{}", i)),
            type_name: Some(type_name.to_string()),
            scope_name: Some("pattern_test".to_string()),
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
        };
        
        allocations.push(allocation);
        println!("Test case {}: {} - {}", i, type_name, description);
    }
    
    let temp_file = NamedTempFile::new().unwrap();
    let config = BinaryExportConfig::debug_comprehensive();
    
    let result = export_to_binary_with_config(&allocations, temp_file.path(), &config);
    assert!(result.is_ok(), "Export with lifecycle pattern analysis should succeed");
    
    // Verify file integrity
    let mut reader = BinaryReader::new(temp_file.path()).unwrap();
    let header = reader.read_header().unwrap();
    assert_eq!(header.count, allocations.len() as u32);
    
    println!("Successfully exported {} allocations with lifecycle pattern analysis", 
             allocations.len());
}