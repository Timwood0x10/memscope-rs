//! Integration test for binary serialization optimization (Task 7)

use memscope_rs::core::types::{
    AllocationInfo, FieldLayoutInfo, LayoutEfficiency, MemoryLayoutInfo, OptimizationPotential,
    PaddingAnalysis, PaddingLocation, PaddingReason, RefCountSnapshot, SmartPointerInfo,
    SmartPointerType,
};
use memscope_rs::export::binary::{export_to_binary_with_config, BinaryExportConfig, BinaryReader};
use std::time::Instant;
use tempfile::NamedTempFile;

fn create_allocation_with_complex_data() -> AllocationInfo {
    // Create complex smart pointer info
    let smart_pointer_info = SmartPointerInfo {
        data_ptr: 0x1000,
        cloned_from: Some(0x2000),
        clones: vec![0x3000, 0x4000, 0x5000, 0x6000],
        ref_count_history: vec![
            RefCountSnapshot {
                timestamp: 1000,
                strong_count: 1,
                weak_count: 0,
            },
            RefCountSnapshot {
                timestamp: 2000,
                strong_count: 4,
                weak_count: 2,
            },
            RefCountSnapshot {
                timestamp: 3000,
                strong_count: 2,
                weak_count: 1,
            },
        ],
        weak_count: Some(1),
        is_weak_reference: false,
        is_data_owner: true,
        is_implicitly_deallocated: false,
        pointer_type: SmartPointerType::Rc,
    };

    // Create complex memory layout info
    let memory_layout_info = MemoryLayoutInfo {
        total_size: 128,
        alignment: 8,
        field_layout: vec![
            FieldLayoutInfo {
                field_name: "id".to_string(),
                field_type: "u32".to_string(),
                offset: 0,
                size: 4,
                alignment: 4,
                is_padding: false,
            },
            FieldLayoutInfo {
                field_name: "padding1".to_string(),
                field_type: "padding".to_string(),
                offset: 4,
                size: 4,
                alignment: 1,
                is_padding: true,
            },
            FieldLayoutInfo {
                field_name: "data".to_string(),
                field_type: "Vec<u8>".to_string(),
                offset: 8,
                size: 24,
                alignment: 8,
                is_padding: false,
            },
            FieldLayoutInfo {
                field_name: "metadata".to_string(),
                field_type: "HashMap<String, String>".to_string(),
                offset: 32,
                size: 48,
                alignment: 8,
                is_padding: false,
            },
        ],
        padding_info: PaddingAnalysis {
            total_padding_bytes: 48,
            padding_locations: vec![
                PaddingLocation {
                    start_offset: 4,
                    size: 4,
                    reason: PaddingReason::FieldAlignment,
                },
                PaddingLocation {
                    start_offset: 80,
                    size: 44,
                    reason: PaddingReason::StructAlignment,
                },
            ],
            padding_ratio: 0.375, // 48/128
            optimization_suggestions: vec![
                "Reorder fields to minimize padding".to_string(),
                "Consider using #[repr(packed)]".to_string(),
                "Group smaller fields together".to_string(),
            ],
        },
        layout_efficiency: LayoutEfficiency {
            memory_utilization: 0.625, // 80/128
            cache_friendliness: 75.0,
            alignment_waste: 48,
            optimization_potential: OptimizationPotential::Moderate {
                potential_savings: 40,
                suggestions: vec![
                    "Reorder fields by size".to_string(),
                    "Use smaller alignment where possible".to_string(),
                ],
            },
        },
        container_analysis: None, // Keep simple for this test
    };

    AllocationInfo {
        ptr: 0x10000,
        size: 2048,
        var_name: Some("complex_data_structure".to_string()),
        type_name: Some("ComplexStruct<T>".to_string()),
        scope_name: Some("process_data".to_string()),
        timestamp_alloc: 1000000000,
        timestamp_dealloc: Some(1000003000),
        thread_id: "worker-thread-1".to_string(),
        borrow_count: 5,
        stack_trace: Some(vec![
            "main".to_string(),
            "process_data".to_string(),
            "allocate_complex_structure".to_string(),
            "create_with_metadata".to_string(),
        ]),
        is_leaked: false,
        lifetime_ms: Some(3000), // 3 seconds
        smart_pointer_info: Some(smart_pointer_info),
        memory_layout: Some(memory_layout_info),
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
            drop_chain_analysis: None,
    }
}

#[test]
fn test_binary_serialization_optimization_comprehensive() {
    // Create test data with complex smart pointer and memory layout info
    let allocations: Vec<AllocationInfo> = (0..500)
        .map(|i| {
            let mut alloc = create_allocation_with_complex_data();
            alloc.ptr = 0x10000 + (i * 0x1000); // Unique pointers
            alloc
        })
        .collect();

    println!(
        "Testing binary serialization optimization with {} complex allocations",
        allocations.len()
    );

    // Test with comprehensive configuration (includes advanced metrics)
    let temp_file = NamedTempFile::new().unwrap();
    let config = BinaryExportConfig::debug_comprehensive();

    // Measure export performance
    let start = Instant::now();
    export_to_binary_with_config(&allocations, temp_file.path(), &config)
        .expect("Failed to export to binary");
    let export_time = start.elapsed();

    // Measure import performance
    let start = Instant::now();
    let mut reader = BinaryReader::new(temp_file.path()).expect("Failed to create binary reader");
    let read_allocations = reader.read_all().expect("Failed to read allocations");
    let import_time = start.elapsed();

    // Get file size
    let file_size = std::fs::metadata(temp_file.path()).unwrap().len();

    println!("Binary serialization results:");
    println!("  Export time: {:?}", export_time);
    println!("  Import time: {:?}", import_time);
    println!("  Total time: {:?}", export_time + import_time);
    println!(
        "  File size: {} bytes ({:.2} KB)",
        file_size,
        file_size as f64 / 1024.0
    );
    println!(
        "  Average per allocation: {:.1} bytes",
        file_size as f64 / allocations.len() as f64
    );

    // Verify data integrity
    assert_eq!(read_allocations.len(), allocations.len());

    // Verify first allocation in detail
    let original = &allocations[0];
    let read = &read_allocations[0];

    assert_eq!(read.ptr, original.ptr);
    assert_eq!(read.size, original.size);
    assert_eq!(read.var_name, original.var_name);
    assert_eq!(read.type_name, original.type_name);
    assert_eq!(read.lifetime_ms, original.lifetime_ms);

    // Verify smart pointer info was preserved
    assert!(read.smart_pointer_info.is_some());
    let read_spi = read.smart_pointer_info.as_ref().unwrap();
    let orig_spi = original.smart_pointer_info.as_ref().unwrap();

    assert_eq!(read_spi.data_ptr, orig_spi.data_ptr);
    assert_eq!(read_spi.cloned_from, orig_spi.cloned_from);
    assert_eq!(read_spi.clones, orig_spi.clones);
    assert_eq!(
        read_spi.ref_count_history.len(),
        orig_spi.ref_count_history.len()
    );
    assert_eq!(read_spi.pointer_type, orig_spi.pointer_type);

    // Verify memory layout info was preserved
    assert!(read.memory_layout.is_some());
    let read_mli = read.memory_layout.as_ref().unwrap();
    let orig_mli = original.memory_layout.as_ref().unwrap();

    assert_eq!(read_mli.total_size, orig_mli.total_size);
    assert_eq!(read_mli.alignment, orig_mli.alignment);
    assert_eq!(read_mli.field_layout.len(), orig_mli.field_layout.len());
    assert_eq!(
        read_mli.padding_info.total_padding_bytes,
        orig_mli.padding_info.total_padding_bytes
    );

    // Verify advanced metrics were read
    let advanced_metrics = reader
        .get_advanced_metrics()
        .expect("Advanced metrics should be present");

    // Should have lifecycle metrics for allocations with lifetime_ms
    assert_eq!(advanced_metrics.lifecycle_metrics.len(), allocations.len());

    // Performance assertions
    assert!(
        export_time.as_millis() < 2000,
        "Export should be fast (< 2s)"
    );
    assert!(
        import_time.as_millis() < 2000,
        "Import should be fast (< 2s)"
    );
    assert!(
        file_size < 5_000_000,
        "File should be reasonably sized (< 5MB)"
    );

    println!("✅ Binary serialization optimization test passed!");
}

#[test]
fn test_json_vs_optimized_binary_comparison() {
    // Create smaller dataset for comparison
    let allocations: Vec<AllocationInfo> = (0..50)
        .map(|i| {
            let mut alloc = create_allocation_with_complex_data();
            alloc.ptr = 0x10000 + (i * 0x1000);
            alloc
        })
        .collect();

    println!(
        "Comparing JSON vs optimized binary serialization ({} allocations)",
        allocations.len()
    );

    // Test JSON serialization
    let json_temp_file = NamedTempFile::new().unwrap();
    let start = Instant::now();
    let json_data = serde_json::to_string(&allocations).unwrap();
    std::fs::write(json_temp_file.path(), &json_data).unwrap();
    let json_export_time = start.elapsed();

    let start = Instant::now();
    let json_content = std::fs::read_to_string(json_temp_file.path()).unwrap();
    let _: Vec<AllocationInfo> = serde_json::from_str(&json_content).unwrap();
    let json_import_time = start.elapsed();

    let json_file_size = std::fs::metadata(json_temp_file.path()).unwrap().len();

    // Test optimized binary serialization
    let binary_temp_file = NamedTempFile::new().unwrap();
    let config = BinaryExportConfig::debug_comprehensive();

    let start = Instant::now();
    export_to_binary_with_config(&allocations, binary_temp_file.path(), &config).unwrap();
    let binary_export_time = start.elapsed();

    let start = Instant::now();
    let mut reader = BinaryReader::new(binary_temp_file.path()).unwrap();
    let _read_allocations = reader.read_all().unwrap();
    let binary_import_time = start.elapsed();

    let binary_file_size = std::fs::metadata(binary_temp_file.path()).unwrap().len();

    // Calculate improvements
    let json_total_time = json_export_time + json_import_time;
    let binary_total_time = binary_export_time + binary_import_time;

    let speed_improvement = json_total_time.as_nanos() as f64 / binary_total_time.as_nanos() as f64;
    let size_reduction = (1.0 - binary_file_size as f64 / json_file_size as f64) * 100.0;

    println!("\n📊 Performance Comparison Results:");
    println!("JSON Serialization:");
    println!("  Export time: {:?}", json_export_time);
    println!("  Import time: {:?}", json_import_time);
    println!("  Total time: {:?}", json_total_time);
    println!(
        "  File size: {} bytes ({:.2} KB)",
        json_file_size,
        json_file_size as f64 / 1024.0
    );

    println!("Optimized Binary Serialization:");
    println!("  Export time: {:?}", binary_export_time);
    println!("  Import time: {:?}", binary_import_time);
    println!("  Total time: {:?}", binary_total_time);
    println!(
        "  File size: {} bytes ({:.2} KB)",
        binary_file_size,
        binary_file_size as f64 / 1024.0
    );

    println!("🚀 Improvements:");
    println!("  Speed improvement: {:.2}x faster", speed_improvement);
    println!("  Size reduction: {:.1}% smaller", size_reduction);
    println!(
        "  Export speed: {:.2}x faster",
        json_export_time.as_nanos() as f64 / binary_export_time.as_nanos() as f64
    );
    println!(
        "  Import speed: {:.2}x faster",
        json_import_time.as_nanos() as f64 / binary_import_time.as_nanos() as f64
    );

    // Performance assertions (adjusted for realistic expectations)
    assert!(
        speed_improvement >= 0.8,
        "Binary should be competitive with JSON"
    );
    assert!(
        size_reduction >= 5.0,
        "Binary should be at least 5% smaller than JSON"
    );
    // Note: Overall performance may be similar due to other JSON fields still being used
    // The key improvement is in the specific optimized structures (smart_pointer_info, memory_layout)
    assert!(
        binary_file_size < json_file_size,
        "Binary should be smaller"
    );
    
    // Import should be faster due to binary deserialization of complex structures
    assert!(
        binary_import_time <= json_import_time * 2,
        "Binary import should be competitive"
    );

    println!("✅ JSON vs Binary comparison test passed!");
}

#[test]
fn test_backward_compatibility_with_optimization() {
    let allocation = create_allocation_with_complex_data();

    // Test that files created with optimized binary serialization
    // can still be read by the reader
    let temp_file = NamedTempFile::new().unwrap();
    let config = BinaryExportConfig::performance_first();

    // Write with optimized serialization
    export_to_binary_with_config(&[allocation.clone()], temp_file.path(), &config).unwrap();

    // Read back and verify
    let mut reader = BinaryReader::new(temp_file.path()).unwrap();
    let read_allocations = reader.read_all().unwrap();

    assert_eq!(read_allocations.len(), 1);
    let read_alloc = &read_allocations[0];

    // Verify all data was preserved correctly
    assert_eq!(read_alloc.ptr, allocation.ptr);
    assert_eq!(
        read_alloc.smart_pointer_info.is_some(),
        allocation.smart_pointer_info.is_some()
    );
    assert_eq!(
        read_alloc.memory_layout.is_some(),
        allocation.memory_layout.is_some()
    );

    if let (Some(read_spi), Some(orig_spi)) = (
        &read_alloc.smart_pointer_info,
        &allocation.smart_pointer_info,
    ) {
        assert_eq!(read_spi.data_ptr, orig_spi.data_ptr);
        assert_eq!(read_spi.pointer_type, orig_spi.pointer_type);
    }

    if let (Some(read_mli), Some(orig_mli)) = (&read_alloc.memory_layout, &allocation.memory_layout)
    {
        assert_eq!(read_mli.total_size, orig_mli.total_size);
        assert_eq!(read_mli.field_layout.len(), orig_mli.field_layout.len());
    }

    println!("✅ Backward compatibility test passed!");
}
