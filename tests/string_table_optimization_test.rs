//! Integration tests for string table optimization in binary export

use memscope_rs::core::types::AllocationInfo;
use memscope_rs::export::binary::{export_to_binary_with_config, BinaryExportConfig, BinaryReader};
use std::fs;
use tempfile::NamedTempFile;

/// Create test allocation data with repeated strings
fn create_test_allocations_with_repeated_strings() -> Vec<AllocationInfo> {
    let mut allocations = Vec::new();

    // Create allocations with repeated type names and function names
    let common_types = vec!["Vec<String>", "HashMap<String, i32>", "Box<dyn Trait>", "Arc<Mutex<T>>"];
    let common_functions = vec!["main", "process_data", "allocate_memory", "cleanup"];

    for i in 0..100 {
        let type_name = common_types[i % common_types.len()].to_string();
        let function_name = common_functions[i % common_functions.len()].to_string();
        
        let allocation = AllocationInfo {
            ptr: 0x1000 + i * 0x100,
            size: 1024 + i * 64,
            var_name: Some(format!("var_{}", i)),
            type_name: Some(type_name),
            scope_name: Some(function_name.clone()),
            timestamp_alloc: 1234567890 + i as u64,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: Some(vec![
                function_name,
                "std::alloc::alloc".to_string(),
                "core::ptr::write".to_string(),
            ]),
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
    }

    allocations
}

#[test]
fn test_string_table_optimization_enabled() {
    let allocations = create_test_allocations_with_repeated_strings();
    
    // Export with string table optimization enabled
    let temp_file = NamedTempFile::new().unwrap();
    let config = BinaryExportConfig::performance_first(); // Has string table optimization enabled
    
    let result = export_to_binary_with_config(&allocations, temp_file.path(), &config);
    assert!(result.is_ok(), "Export with string table should succeed");
    
    // Verify file was created and has reasonable size
    let metadata = fs::metadata(temp_file.path()).unwrap();
    assert!(metadata.len() > 0, "File should not be empty");
    
    // Try to read the file back
    let mut reader = BinaryReader::new(temp_file.path()).unwrap();
    let header = reader.read_header().unwrap();
    assert_eq!(header.count, allocations.len() as u32);
    
    // Read back some allocations to verify correctness
    for i in 0..5 {
        let allocation = reader.read_allocation().unwrap();
        assert_eq!(allocation.ptr, allocations[i].ptr);
        assert_eq!(allocation.size, allocations[i].size);
        assert_eq!(allocation.type_name, allocations[i].type_name);
        assert_eq!(allocation.scope_name, allocations[i].scope_name);
    }
}

#[test]
fn test_string_table_optimization_disabled() {
    let allocations = create_test_allocations_with_repeated_strings();
    
    // Export with string table optimization disabled
    let temp_file = NamedTempFile::new().unwrap();
    let config = BinaryExportConfig::minimal(); // Has string table optimization disabled
    
    let result = export_to_binary_with_config(&allocations, temp_file.path(), &config);
    assert!(result.is_ok(), "Export without string table should succeed");
    
    // Verify file was created
    let metadata = fs::metadata(temp_file.path()).unwrap();
    assert!(metadata.len() > 0, "File should not be empty");
    
    // Try to read the file back
    let mut reader = BinaryReader::new(temp_file.path()).unwrap();
    let header = reader.read_header().unwrap();
    assert_eq!(header.count, allocations.len() as u32);
    
    // Read back some allocations to verify correctness
    for i in 0..5 {
        let allocation = reader.read_allocation().unwrap();
        assert_eq!(allocation.ptr, allocations[i].ptr);
        assert_eq!(allocation.size, allocations[i].size);
        assert_eq!(allocation.type_name, allocations[i].type_name);
        assert_eq!(allocation.scope_name, allocations[i].scope_name);
    }
}

#[test]
fn test_string_table_compression_effectiveness() {
    let allocations = create_test_allocations_with_repeated_strings();
    
    // Export with string table optimization
    let temp_file_optimized = NamedTempFile::new().unwrap();
    let config_optimized = BinaryExportConfig::performance_first();
    export_to_binary_with_config(&allocations, temp_file_optimized.path(), &config_optimized).unwrap();
    
    // Export without string table optimization
    let temp_file_unoptimized = NamedTempFile::new().unwrap();
    let config_unoptimized = BinaryExportConfig::minimal();
    export_to_binary_with_config(&allocations, temp_file_unoptimized.path(), &config_unoptimized).unwrap();
    
    // Compare file sizes
    let optimized_size = fs::metadata(temp_file_optimized.path()).unwrap().len();
    let unoptimized_size = fs::metadata(temp_file_unoptimized.path()).unwrap().len();
    
    println!("Optimized size: {} bytes", optimized_size);
    println!("Unoptimized size: {} bytes", unoptimized_size);
    
    // With repeated strings, the optimized version should be smaller or similar size
    // (It might not always be smaller due to the overhead of the string table itself)
    // But it should not be significantly larger
    let size_ratio = optimized_size as f64 / unoptimized_size as f64;
    assert!(size_ratio < 1.5, "Optimized file should not be more than 50% larger than unoptimized");
}

#[test]
fn test_string_table_with_unique_strings() {
    // Create allocations with mostly unique strings (string table should not be beneficial)
    let mut allocations = Vec::new();
    
    for i in 0..20 {
        let allocation = AllocationInfo {
            ptr: 0x1000 + i * 0x100,
            size: 1024,
            var_name: Some(format!("unique_var_{}", i)),
            type_name: Some(format!("UniqueType{}", i)),
            scope_name: Some(format!("unique_function_{}", i)),
            timestamp_alloc: 1234567890 + i as u64,
            timestamp_dealloc: None,
            thread_id: format!("thread_{}", i),
            borrow_count: 0,
            stack_trace: Some(vec![format!("unique_frame_{}", i)]),
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
    }
    
    // Export with string table optimization enabled
    let temp_file = NamedTempFile::new().unwrap();
    let config = BinaryExportConfig::performance_first();
    
    let result = export_to_binary_with_config(&allocations, temp_file.path(), &config);
    assert!(result.is_ok(), "Export should succeed even with unique strings");
    
    // Verify we can read the data back correctly
    let mut reader = BinaryReader::new(temp_file.path()).unwrap();
    let header = reader.read_header().unwrap();
    assert_eq!(header.count, allocations.len() as u32);
    
    // Read back all allocations to verify correctness
    for i in 0..allocations.len() {
        let allocation = reader.read_allocation().unwrap();
        assert_eq!(allocation.ptr, allocations[i].ptr);
        assert_eq!(allocation.type_name, allocations[i].type_name);
        assert_eq!(allocation.var_name, allocations[i].var_name);
        assert_eq!(allocation.scope_name, allocations[i].scope_name);
        assert_eq!(allocation.thread_id, allocations[i].thread_id);
    }
}

#[test]
fn test_empty_and_none_strings() {
    // Test with empty strings and None values
    let allocation = AllocationInfo {
        ptr: 0x1000,
        size: 1024,
        var_name: None,
        type_name: Some("".to_string()), // Empty string
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
    };
    
    let allocations = vec![allocation];
    
    // Export with string table optimization
    let temp_file = NamedTempFile::new().unwrap();
    let config = BinaryExportConfig::performance_first();
    
    let result = export_to_binary_with_config(&allocations, temp_file.path(), &config);
    assert!(result.is_ok(), "Export should handle empty/None strings");
    
    // Verify we can read the data back correctly
    let mut reader = BinaryReader::new(temp_file.path()).unwrap();
    let header = reader.read_header().unwrap();
    assert_eq!(header.count, 1);
    
    let read_allocation = reader.read_allocation().unwrap();
    assert_eq!(read_allocation.var_name, None);
    assert_eq!(read_allocation.type_name, Some("".to_string()));
    assert_eq!(read_allocation.scope_name, None);
}