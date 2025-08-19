//! Integration test for complex type analysis in binary HTML export
//!
//! This test verifies that the complex type analyzer is properly integrated
//! with the binary HTML writer and produces expected results.

#[cfg(test)]
mod tests {
    use crate::core::types::AllocationInfo;
    use crate::export::binary::binary_html_writer::BinaryHtmlWriter;
    use crate::export::binary::complex_type_analyzer::{ComplexTypeAnalyzer, TypeCategory};
    use crate::export::binary::selective_reader::AllocationField;
    use std::io::Cursor;

    fn create_test_allocation_with_type(
        type_name: &str,
        size: usize,
        ptr: usize,
    ) -> AllocationInfo {
        AllocationInfo {
            ptr,
            size,
            type_name: Some(type_name.to_string()),
            var_name: Some(format!("var_{}", ptr)),
            scope_name: Some("test_scope".to_string()),
            timestamp_alloc: 1000,
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
            drop_chain_analysis: None,
        }
    }

    #[test]
    fn test_complex_type_analysis_integration() {
        // Create test allocations with various types
        let allocations = vec![
            create_test_allocation_with_type("i32", 4, 0x1000),
            create_test_allocation_with_type("Vec<String>", 24, 0x2000),
            create_test_allocation_with_type("HashMap<String, i32>", 48, 0x3000),
            create_test_allocation_with_type("Box<Vec<u8>>", 16, 0x4000),
            create_test_allocation_with_type("Arc<Mutex<HashMap<String, Vec<i32>>>>", 64, 0x5000),
            create_test_allocation_with_type("MyCustomStruct", 32, 0x6000),
        ];

        // Test direct analysis
        let analysis = ComplexTypeAnalyzer::analyze_allocations(&allocations)
            .expect("Failed to get test value");

        // Verify analysis results
        assert_eq!(analysis.summary.total_types, 6);
        assert_eq!(analysis.summary.primitive_count, 1); // i32
        assert_eq!(analysis.summary.collection_count, 2); // Vec<String>, HashMap<String, i32>
        assert_eq!(analysis.summary.smart_pointer_count, 2); // Box<Vec<u8>>, Arc<Mutex<HashMap<String, Vec<i32>>>>
                                                             // MyCustomStruct should be in custom_types
        assert_eq!(analysis.categorized_types.custom_types.len(), 1);

        // Verify complexity scores
        assert_eq!(analysis.complexity_scores.get("i32"), Some(&1));
        assert_eq!(analysis.complexity_scores.get("Vec<String>"), Some(&4));
        assert_eq!(
            analysis
                .complexity_scores
                .get("Arc<Mutex<HashMap<String, Vec<i32>>>>"),
            Some(&10)
        );

        // Verify categorization
        let primitives = &analysis.categorized_types.primitives;
        assert_eq!(primitives.len(), 1);
        assert_eq!(primitives[0].name, "i32");
        assert_eq!(primitives[0].category, TypeCategory::Primitive);

        let collections = &analysis.categorized_types.collections;
        assert_eq!(collections.len(), 2);
        assert!(collections.iter().any(|t| t.name == "Vec<String>"));
        assert!(collections.iter().any(|t| t.name == "HashMap<String, i32>"));

        let smart_pointers = &analysis.categorized_types.smart_pointers;
        assert_eq!(smart_pointers.len(), 2);
        assert!(smart_pointers.iter().any(|t| t.name == "Box<Vec<u8>>"));
        assert!(smart_pointers.iter().any(|t| t.name.contains("Arc<Mutex<")));

        // Generic analysis tracks base types separately from categorization
        // Types like Vec<String> are categorized as Collections, not Generics
        // So generic_analysis may be empty while still having generic parameters in other categories
    }

    #[test]
    fn test_binary_html_writer_with_complex_types() {
        // Create test allocations
        let allocations = vec![
            create_test_allocation_with_type("Vec<String>", 24, 0x1000),
            create_test_allocation_with_type("HashMap<String, i32>", 48, 0x2000),
            create_test_allocation_with_type("Box<i32>", 8, 0x3000),
        ];

        // Create binary HTML writer
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = BinaryHtmlWriter::new(cursor).expect("Failed to get test value");

        // Write allocations
        let fields = AllocationField::all_basic_fields();
        for allocation in &allocations {
            writer
                .write_binary_allocation(allocation, &fields)
                .expect("Test operation failed");
        }

        // Finalize and get stats
        let stats = writer
            .finalize_with_binary_template("test_project")
            .expect("Test operation failed");

        // Verify that allocations were processed
        assert_eq!(stats.allocations_processed, 3);
        assert!(stats.total_html_size > 0);
    }

    #[test]
    fn test_type_complexity_edge_cases() {
        // Test empty type name
        let empty_allocation = create_test_allocation_with_type("", 4, 0x1000);
        let result = ComplexTypeAnalyzer::analyze_allocations(&[empty_allocation]);
        assert!(result.is_ok());
        let analysis = result.expect("Test operation failed");
        assert_eq!(analysis.summary.total_types, 0); // Empty types should be skipped

        // Test unknown type
        let unknown_allocation = create_test_allocation_with_type("Unknown", 4, 0x1000);
        let result = ComplexTypeAnalyzer::analyze_allocations(&[unknown_allocation]);
        assert!(result.is_ok());
        let analysis = result.expect("Test operation failed");
        assert_eq!(analysis.summary.total_types, 0); // Unknown types should be skipped

        // Test very complex nested type
        let complex_allocation = create_test_allocation_with_type(
            "Arc<RwLock<HashMap<String, Vec<Box<dyn Trait>>>>>",
            64,
            0x1000,
        );
        let result = ComplexTypeAnalyzer::analyze_allocations(&[complex_allocation]);
        assert!(result.is_ok());
        let analysis = result.expect("Test operation failed");
        assert_eq!(analysis.summary.total_types, 1);
        // The complexity should be capped at 10
        assert_eq!(
            analysis
                .complexity_scores
                .get("Arc<RwLock<HashMap<String, Vec<Box<dyn Trait>>>>>"),
            Some(&10)
        );
    }

    #[test]
    fn test_generic_parameter_parsing_edge_cases() {
        // Test nested generics through analysis
        let nested_allocation =
            create_test_allocation_with_type("HashMap<String, Vec<Option<i32>>>", 64, 0x1000);
        let result = ComplexTypeAnalyzer::analyze_allocations(&[nested_allocation]);
        assert!(result.is_ok());
        let analysis = result.expect("Test operation failed");

        // Verify the type was analyzed
        assert_eq!(analysis.summary.total_types, 1);
        // The type should have generic parameters even if not categorized as Generic
        let type_info = &analysis.categorized_types.collections[0];
        assert!(!type_info.generic_parameters.is_empty());

        // Test multiple parameter types
        let multi_param_allocation =
            create_test_allocation_with_type("Result<String, Error>", 32, 0x2000);
        let result = ComplexTypeAnalyzer::analyze_allocations(&[multi_param_allocation]);
        assert!(result.is_ok());
        let analysis = result.expect("Test operation failed");
        assert_eq!(analysis.summary.total_types, 1);
    }

    #[test]
    fn test_memory_efficiency_calculation() {
        let allocations = vec![
            create_test_allocation_with_type("i32", 4, 0x1000),
            create_test_allocation_with_type("i32", 4, 0x2000),
            create_test_allocation_with_type("Vec<String>", 100, 0x3000),
        ];

        let analysis = ComplexTypeAnalyzer::analyze_allocations(&allocations)
            .expect("Failed to get test value");

        // Find i32 type info
        let i32_info = analysis
            .categorized_types
            .primitives
            .iter()
            .find(|t| t.name == "i32")
            .expect("Test operation failed");

        assert_eq!(i32_info.allocation_count, 2);
        assert_eq!(i32_info.total_memory, 8);
        assert_eq!(i32_info.average_size, 4.0);
        assert!(i32_info.memory_efficiency > 0.0);

        // Find Vec<String> type info
        let vec_info = analysis
            .categorized_types
            .collections
            .iter()
            .find(|t| t.name == "Vec<String>")
            .expect("Test operation failed");

        assert_eq!(vec_info.allocation_count, 1);
        assert_eq!(vec_info.total_memory, 100);
        assert_eq!(vec_info.average_size, 100.0);
    }
}
