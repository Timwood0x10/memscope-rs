//! Integration test for FFI safety analysis in binary HTML export
//!
//! This test verifies that the FFI safety analyzer is properly integrated
//! with the binary HTML writer and produces expected results.

#[cfg(test)]
mod tests {
    use crate::core::types::AllocationInfo;
    use crate::export::binary::binary_html_writer::BinaryHtmlWriter;
    use crate::export::binary::ffi_safety_analyzer::{
        FfiSafetyAnalyzer, RiskLevel, UnsafeOperationType,
    };
    use crate::export::binary::selective_reader::AllocationField;
    use std::io::Cursor;

    fn create_test_allocation_with_ffi(
        ptr: usize,
        size: usize,
        type_name: Option<&str>,
        scope_name: Option<&str>,
        stack_trace: Option<Vec<String>>,
        is_leaked: bool,
    ) -> AllocationInfo {
        AllocationInfo {
            ptr,
            size,
            type_name: type_name.map(|s| s.to_string()),
            var_name: Some(format!("var_{}", ptr)),
            scope_name: scope_name.map(|s| s.to_string()),
            timestamp_alloc: 1000,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace,
            is_leaked,
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
    fn test_ffi_safety_analysis_integration() {
        // Create test allocations with various FFI safety scenarios
        let allocations = vec![
            // Safe allocation
            create_test_allocation_with_ffi(
                0x1000,
                1024,
                Some("Vec<u8>"),
                Some("safe_function"),
                Some(vec![
                    "main::safe_function".to_string(),
                    "std::vec::Vec::new".to_string(),
                ]),
                false,
            ),
            // FFI boundary allocation
            create_test_allocation_with_ffi(
                0x2000,
                2048,
                Some("c_void"),
                Some("ffi_module"),
                Some(vec![
                    "main::ffi_call".to_string(),
                    "module::ffi::unsafe_function".to_string(),
                    "libc::malloc".to_string(),
                ]),
                false,
            ),
            // Raw pointer allocation
            create_test_allocation_with_ffi(
                0x3000,
                512,
                Some("*mut u8"),
                Some("unsafe_block"),
                Some(vec![
                    "unsafe { transmute(ptr) }".to_string(),
                    "Vec::from_raw_parts".to_string(),
                ]),
                false,
            ),
            // Leaked allocation (potential use-after-free)
            create_test_allocation_with_ffi(
                0x4000,
                4096,
                Some("CString"),
                Some("ffi_string"),
                Some(vec!["CString::new".to_string(), "libc::strdup".to_string()]),
                true,
            ),
            // Large allocation (potential buffer overflow)
            create_test_allocation_with_ffi(
                0x5000,
                2 * 1024 * 1024, // 2MB
                Some("Vec<u8>"),
                Some("large_buffer"),
                Some(vec!["allocate_large_buffer".to_string()]),
                false,
            ),
        ];

        // Test direct analysis
        let analysis =
            FfiSafetyAnalyzer::analyze_allocations(&allocations).expect("Failed to get test value");

        // Verify analysis results
        assert_eq!(analysis.summary.total_allocations, 5);
        assert!(analysis.summary.unsafe_operations_count > 0);
        assert!(analysis.summary.ffi_allocations_count > 0);
        assert!(analysis.summary.safety_score <= 100);

        // Verify unsafe operations detection
        assert!(!analysis.unsafe_operations.is_empty());

        // Check for specific unsafe operation types
        let has_raw_pointer = analysis
            .unsafe_operations
            .iter()
            .any(|op| op.operation_type == UnsafeOperationType::RawPointerDeref);
        let has_use_after_free = analysis
            .unsafe_operations
            .iter()
            .any(|op| op.operation_type == UnsafeOperationType::UseAfterFree);
        let has_buffer_overflow = analysis
            .unsafe_operations
            .iter()
            .any(|op| op.operation_type == UnsafeOperationType::BufferOverflow);

        assert!(has_raw_pointer);
        assert!(has_use_after_free);
        assert!(has_buffer_overflow);

        // Verify FFI hotspots
        assert!(!analysis.ffi_hotspots.is_empty());
        let ffi_hotspot = analysis
            .ffi_hotspots
            .iter()
            .find(|h| h.name.contains("ffi") || h.name.contains("libc"));
        assert!(ffi_hotspot.is_some());

        // Verify risk assessment
        assert!(analysis.risk_assessment.memory_safety.issue_count > 0);
        assert!(!analysis.risk_assessment.risk_distribution.is_empty());

        // Verify call graph
        assert!(!analysis.call_graph.nodes.is_empty());
        assert_eq!(
            analysis.call_graph.statistics.node_count,
            analysis.call_graph.nodes.len()
        );
    }

    #[test]
    fn test_binary_html_writer_with_ffi_analysis() {
        // Create test allocations with FFI patterns
        let allocations = vec![
            create_test_allocation_with_ffi(
                0x1000,
                1024,
                Some("CString"),
                Some("ffi_module"),
                Some(vec!["module::ffi::function".to_string()]),
                false,
            ),
            create_test_allocation_with_ffi(
                0x2000,
                2048,
                Some("*const c_char"),
                Some("unsafe_block"),
                Some(vec!["unsafe { transmute(ptr) }".to_string()]),
                false,
            ),
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
        assert_eq!(stats.allocations_processed, 2);
        assert!(stats.total_html_size > 0);
    }

    #[test]
    fn test_ffi_safety_edge_cases() {
        // Test with empty allocations
        let empty_allocations = vec![];
        let result = FfiSafetyAnalyzer::analyze_allocations(&empty_allocations);
        assert!(result.is_ok());
        let analysis = result.expect("Test operation failed");
        assert_eq!(analysis.summary.total_allocations, 0);
        assert_eq!(analysis.summary.unsafe_operations_count, 0);

        // Test with allocation without stack trace
        let no_stack_trace = vec![create_test_allocation_with_ffi(
            0x1000,
            1024,
            Some("Vec<u8>"),
            Some("safe_function"),
            None,
            false,
        )];
        let result = FfiSafetyAnalyzer::analyze_allocations(&no_stack_trace);
        assert!(result.is_ok());

        // Test with allocation without type name
        let no_type_name = vec![create_test_allocation_with_ffi(
            0x1000,
            1024,
            None,
            Some("unknown_function"),
            Some(vec!["unknown_function".to_string()]),
            false,
        )];
        let result = FfiSafetyAnalyzer::analyze_allocations(&no_type_name);
        assert!(result.is_ok());
    }

    #[test]
    fn test_risk_level_assessment() {
        // Test high-risk scenario
        let high_risk_allocations = vec![
            create_test_allocation_with_ffi(
                0x1000,
                1024,
                Some("*mut c_void"),
                Some("unsafe_block"),
                Some(vec!["unsafe { transmute(ptr) }".to_string()]),
                true, // leaked
            ),
            create_test_allocation_with_ffi(
                0x2000,
                2048,
                Some("*const c_char"),
                Some("unsafe_block"),
                Some(vec!["Vec::from_raw_parts".to_string()]),
                false,
            ),
        ];

        let analysis = FfiSafetyAnalyzer::analyze_allocations(&high_risk_allocations)
            .expect("Failed to get test value");

        // Should detect multiple high-risk operations
        assert!(analysis.summary.unsafe_operations_count >= 2);
        assert!(matches!(
            analysis.summary.risk_level,
            RiskLevel::High | RiskLevel::Critical
        ));
        assert!(analysis.summary.safety_score < 80); // Should be penalized for unsafe operations
    }

    #[test]
    fn test_ffi_hotspot_detection() {
        let allocations = vec![
            // Multiple allocations from same FFI function
            create_test_allocation_with_ffi(
                0x1000,
                1024,
                Some("c_int"),
                Some("ffi_module"),
                Some(vec!["module::ffi::allocate_buffer".to_string()]),
                false,
            ),
            create_test_allocation_with_ffi(
                0x2000,
                2048,
                Some("c_char"),
                Some("ffi_module"),
                Some(vec!["module::ffi::allocate_buffer".to_string()]),
                false,
            ),
            create_test_allocation_with_ffi(
                0x3000,
                512,
                Some("c_void"),
                Some("ffi_module"),
                Some(vec!["module::ffi::allocate_buffer".to_string()]),
                false,
            ),
        ];

        let analysis =
            FfiSafetyAnalyzer::analyze_allocations(&allocations).expect("Failed to get test value");

        // Should detect hotspot for repeated FFI function
        assert!(!analysis.ffi_hotspots.is_empty());
        let hotspot = &analysis.ffi_hotspots[0];
        assert!(hotspot.call_count > 1);
        assert!(hotspot.total_memory > 1024);
        assert!(hotspot.average_size > 0.0);
    }

    #[test]
    fn test_call_graph_generation() {
        let allocations = vec![create_test_allocation_with_ffi(
            0x1000,
            1024,
            Some("CString"),
            Some("main"),
            Some(vec![
                "main::function".to_string(),
                "module::ffi::wrapper".to_string(),
                "libc::malloc".to_string(),
            ]),
            false,
        )];

        let analysis =
            FfiSafetyAnalyzer::analyze_allocations(&allocations).expect("Failed to get test value");

        // Should generate call graph with nodes
        assert!(!analysis.call_graph.nodes.is_empty());
        assert_eq!(
            analysis.call_graph.statistics.node_count,
            analysis.call_graph.nodes.len()
        );

        // Nodes should have proper classification
        let has_rust_function = analysis.call_graph.nodes.iter().any(|node| {
            node.node_type == crate::export::binary::ffi_safety_analyzer::NodeType::RustFunction
        });
        let has_external_lib = analysis.call_graph.nodes.iter().any(|node| {
            node.node_type == crate::export::binary::ffi_safety_analyzer::NodeType::ExternalLibrary
        });

        assert!(has_rust_function || has_external_lib);
    }

    #[test]
    fn test_memory_safety_analysis() {
        let allocations = vec![
            // Large allocation (potential buffer overflow)
            create_test_allocation_with_ffi(
                0x1000,
                5 * 1024 * 1024, // 5MB
                Some("Vec<u8>"),
                Some("large_buffer"),
                Some(vec!["allocate_huge_buffer".to_string()]),
                false,
            ),
            // Leaked allocation
            create_test_allocation_with_ffi(
                0x2000,
                1024,
                Some("Box<i32>"),
                Some("leaked_function"),
                Some(vec!["Box::new".to_string()]),
                true,
            ),
        ];

        let analysis =
            FfiSafetyAnalyzer::analyze_allocations(&allocations).expect("Failed to get test value");

        // Should detect memory safety issues
        assert!(analysis.risk_assessment.memory_safety.issue_count > 0);
        assert!(analysis.risk_assessment.data_integrity.issue_count > 0);

        // Should have buffer overflow and use-after-free operations
        let has_buffer_overflow = analysis
            .unsafe_operations
            .iter()
            .any(|op| op.operation_type == UnsafeOperationType::BufferOverflow);
        let has_use_after_free = analysis
            .unsafe_operations
            .iter()
            .any(|op| op.operation_type == UnsafeOperationType::UseAfterFree);

        assert!(has_buffer_overflow);
        assert!(has_use_after_free);
    }
}
