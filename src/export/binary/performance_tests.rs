//! Performance tests for binary serialization vs JSON serialization

#[cfg(test)]
mod tests {
    use crate::core::types::{
        AllocationInfo, FieldLayoutInfo, LayoutEfficiency, MemoryLayoutInfo, OptimizationPotential,
        PaddingAnalysis, PaddingLocation, PaddingReason, RefCountSnapshot, SmartPointerInfo,
        SmartPointerType,
    };
    use crate::export::binary::config::BinaryExportConfig;
    use crate::export::binary::reader::BinaryReader;
    use crate::export::binary::serializable::BinarySerializable;
    use crate::export::binary::writer::BinaryWriter;
    use std::time::Instant;
    use tempfile::NamedTempFile;

    fn create_complex_allocation_info() -> AllocationInfo {
        // Create a complex SmartPointerInfo
        let smart_pointer_info = SmartPointerInfo {
            data_ptr: 0x1000,
            cloned_from: Some(0x2000),
            clones: vec![0x3000, 0x4000, 0x5000],
            ref_count_history: vec![
                RefCountSnapshot {
                    timestamp: 1000,
                    strong_count: 1,
                    weak_count: 0,
                },
                RefCountSnapshot {
                    timestamp: 2000,
                    strong_count: 3,
                    weak_count: 1,
                },
                RefCountSnapshot {
                    timestamp: 3000,
                    strong_count: 2,
                    weak_count: 2,
                },
            ],
            weak_count: Some(2),
            is_weak_reference: false,
            is_data_owner: true,
            is_implicitly_deallocated: false,
            pointer_type: SmartPointerType::Rc,
        };

        // Create a complex MemoryLayoutInfo
        let memory_layout_info = MemoryLayoutInfo {
            total_size: 64,
            alignment: 8,
            field_layout: vec![
                FieldLayoutInfo {
                    field_name: "field1".to_string(),
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
                    field_name: "field2".to_string(),
                    field_type: "u64".to_string(),
                    offset: 8,
                    size: 8,
                    alignment: 8,
                    is_padding: false,
                },
            ],
            padding_info: PaddingAnalysis {
                total_padding_bytes: 4,
                padding_locations: vec![PaddingLocation {
                    start_offset: 4,
                    size: 4,
                    reason: PaddingReason::FieldAlignment,
                }],
                padding_ratio: 0.0625, // 4/64
                optimization_suggestions: vec![
                    "Reorder fields to reduce padding".to_string(),
                    "Use packed struct attribute".to_string(),
                ],
            },
            layout_efficiency: LayoutEfficiency {
                memory_utilization: 0.9375, // 60/64
                cache_friendliness: 85.0,
                alignment_waste: 4,
                optimization_potential: OptimizationPotential::Minor {
                    potential_savings: 4,
                },
            },
            container_analysis: None, // Keep simple for now
        };

        AllocationInfo {
            ptr: 0x10000,
            size: 1024,
            var_name: Some("complex_allocation".to_string()),
            type_name: Some("ComplexStruct".to_string()),
            scope_name: Some("test_function".to_string()),
            timestamp_alloc: 1000000000,
            timestamp_dealloc: Some(1000001500),
            thread_id: "main".to_string(),
            borrow_count: 3,
            stack_trace: Some(vec![
                "main".to_string(),
                "test_function".to_string(),
                "allocate_complex".to_string(),
            ]),
            is_leaked: false,
            lifetime_ms: Some(1500),
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
        }
    }

    #[test]
    fn test_binary_serialization_performance() {
        let allocations: Vec<AllocationInfo> = (0..1000)
            .map(|_| create_complex_allocation_info())
            .collect();

        // Test binary serialization performance
        let temp_file = NamedTempFile::new().unwrap();
        let config = BinaryExportConfig::performance_first();

        let start = Instant::now();
        let mut writer = BinaryWriter::new_with_config(temp_file.path(), &config).unwrap();
        writer.write_header(allocations.len() as u32).unwrap();

        for allocation in &allocations {
            writer.write_allocation(allocation).unwrap();
        }

        writer.finish().unwrap();
        let binary_write_time = start.elapsed();

        // Test binary deserialization performance
        let start = Instant::now();
        let mut reader = BinaryReader::new(temp_file.path()).unwrap();
        let read_allocations = reader.read_all().unwrap();
        let binary_read_time = start.elapsed();

        // Verify data integrity
        assert_eq!(read_allocations.len(), allocations.len());
        assert_eq!(read_allocations[0].ptr, allocations[0].ptr);
        assert_eq!(
            read_allocations[0]
                .smart_pointer_info
                .as_ref()
                .unwrap()
                .data_ptr,
            allocations[0].smart_pointer_info.as_ref().unwrap().data_ptr
        );

        // Get file size
        let binary_file_size = std::fs::metadata(temp_file.path()).unwrap().len();

        println!("Binary serialization performance:");
        println!("  Write time: {:?}", binary_write_time);
        println!("  Read time: {:?}", binary_read_time);
        println!("  File size: {} bytes", binary_file_size);
        println!("  Total time: {:?}", binary_write_time + binary_read_time);

        // Performance assertions
        assert!(
            binary_write_time.as_millis() < 1000,
            "Binary write should be fast"
        );
        assert!(
            binary_read_time.as_millis() < 1000,
            "Binary read should be fast"
        );
    }

    #[test]
    fn test_json_vs_binary_comparison() {
        let allocations: Vec<AllocationInfo> =
            (0..100).map(|_| create_complex_allocation_info()).collect();

        // Test JSON serialization (for comparison)
        let json_temp_file = NamedTempFile::new().unwrap();
        let start = Instant::now();
        let json_data = serde_json::to_string(&allocations).unwrap();
        std::fs::write(json_temp_file.path(), &json_data).unwrap();
        let json_write_time = start.elapsed();

        let start = Instant::now();
        let json_content = std::fs::read_to_string(json_temp_file.path()).unwrap();
        let _: Vec<AllocationInfo> = serde_json::from_str(&json_content).unwrap();
        let json_read_time = start.elapsed();

        let json_file_size = std::fs::metadata(json_temp_file.path()).unwrap().len();

        // Test binary serialization
        let binary_temp_file = NamedTempFile::new().unwrap();
        let config = BinaryExportConfig::performance_first();

        let start = Instant::now();
        let mut writer = BinaryWriter::new_with_config(binary_temp_file.path(), &config).unwrap();
        writer.write_header(allocations.len() as u32).unwrap();

        for allocation in &allocations {
            writer.write_allocation(allocation).unwrap();
        }

        writer.finish().unwrap();
        let binary_write_time = start.elapsed();

        let start = Instant::now();
        let mut reader = BinaryReader::new(binary_temp_file.path()).unwrap();
        let _read_allocations = reader.read_all().unwrap();
        let binary_read_time = start.elapsed();

        let binary_file_size = std::fs::metadata(binary_temp_file.path()).unwrap().len();

        println!("\nPerformance Comparison (100 complex allocations):");
        println!("JSON:");
        println!("  Write time: {:?}", json_write_time);
        println!("  Read time: {:?}", json_read_time);
        println!("  File size: {} bytes", json_file_size);
        println!("  Total time: {:?}", json_write_time + json_read_time);

        println!("Binary:");
        println!("  Write time: {:?}", binary_write_time);
        println!("  Read time: {:?}", binary_read_time);
        println!("  File size: {} bytes", binary_file_size);
        println!("  Total time: {:?}", binary_write_time + binary_read_time);

        let json_total_time = json_write_time + json_read_time;
        let binary_total_time = binary_write_time + binary_read_time;

        println!("\nImprovement:");
        println!(
            "  Speed improvement: {:.2}x",
            json_total_time.as_nanos() as f64 / binary_total_time.as_nanos() as f64
        );
        println!(
            "  Size reduction: {:.1}%",
            (1.0 - binary_file_size as f64 / json_file_size as f64) * 100.0
        );

        // Performance assertions
        assert!(
            binary_total_time < json_total_time,
            "Binary should be faster than JSON"
        );
        assert!(
            binary_file_size < json_file_size,
            "Binary should be smaller than JSON"
        );
    }

    #[test]
    fn test_smart_pointer_serialization_performance() {
        let smart_pointer = SmartPointerInfo {
            data_ptr: 0x1000,
            cloned_from: Some(0x2000),
            clones: (0..100).map(|i| 0x3000 + i).collect(), // 100 clones
            ref_count_history: (0..50)
                .map(|i| RefCountSnapshot {
                    timestamp: 1000 + i as u64,
                    strong_count: (i % 10) + 1,
                    weak_count: i % 5,
                })
                .collect(),
            weak_count: Some(5),
            is_weak_reference: false,
            is_data_owner: true,
            is_implicitly_deallocated: false,
            pointer_type: SmartPointerType::Rc,
        };

        // Test binary serialization
        let mut binary_buffer = Vec::new();
        let start = Instant::now();
        smart_pointer.write_binary(&mut binary_buffer).unwrap();
        let binary_write_time = start.elapsed();

        let start = Instant::now();
        let mut cursor = std::io::Cursor::new(&binary_buffer);
        let _read_smart_pointer = SmartPointerInfo::read_binary(&mut cursor).unwrap();
        let binary_read_time = start.elapsed();

        // Test JSON serialization
        let start = Instant::now();
        let json_data = serde_json::to_string(&smart_pointer).unwrap();
        let json_write_time = start.elapsed();

        let start = Instant::now();
        let _: SmartPointerInfo = serde_json::from_str(&json_data).unwrap();
        let json_read_time = start.elapsed();

        println!("\nSmartPointerInfo Serialization Comparison:");
        println!("JSON:");
        println!("  Write time: {:?}", json_write_time);
        println!("  Read time: {:?}", json_read_time);
        println!("  Size: {} bytes", json_data.len());

        println!("Binary:");
        println!("  Write time: {:?}", binary_write_time);
        println!("  Read time: {:?}", binary_read_time);
        println!("  Size: {} bytes", binary_buffer.len());

        let json_total_time = json_write_time + json_read_time;
        let binary_total_time = binary_write_time + binary_read_time;

        println!("Improvement:");
        println!(
            "  Speed improvement: {:.2}x",
            json_total_time.as_nanos() as f64 / binary_total_time.as_nanos() as f64
        );
        println!(
            "  Size reduction: {:.1}%",
            (1.0 - binary_buffer.len() as f64 / json_data.len() as f64) * 100.0
        );

        // Performance assertions
        assert!(
            binary_total_time <= json_total_time,
            "Binary should be at least as fast as JSON"
        );
        assert!(
            binary_buffer.len() <= json_data.len(),
            "Binary should be at least as compact as JSON"
        );
    }
}
