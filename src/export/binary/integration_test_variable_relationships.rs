//! Integration test for variable relationship analysis in binary HTML export
//!
//! This test verifies that the variable relationship analyzer is properly integrated
//! with the binary HTML writer and produces expected D3.js compatible results.

#[cfg(test)]
mod tests {
    use crate::core::types::AllocationInfo;
    use crate::export::binary::binary_html_writer::BinaryHtmlWriter;
    use crate::export::binary::selective_reader::AllocationField;
    use crate::export::binary::variable_relationship_analyzer::{
        EdgeDirection, LifetimeCategory, NodeCategory, OwnershipStatus, RelationshipType,
        VariableRelationshipAnalyzer,
    };
    use std::io::Cursor;

    fn create_test_allocation_with_relationships(
        ptr: usize,
        size: usize,
        var_name: Option<&str>,
        type_name: Option<&str>,
        scope_name: Option<&str>,
        timestamp: u64,
        lifetime_ms: Option<u64>,
    ) -> AllocationInfo {
        AllocationInfo {
            ptr,
            size,
            var_name: var_name.map(|s| s.to_string()),
            type_name: type_name.map(|s| s.to_string()),
            scope_name: scope_name.map(|s| s.to_string()),
            timestamp_alloc: timestamp,
            timestamp_dealloc: lifetime_ms.map(|l| timestamp + l * 1_000_000), // Convert ms to ns
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms,
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
    fn test_variable_relationship_analysis_integration() {
        // Create test allocations with various relationship scenarios
        let allocations = vec![
            // Stack variable
            create_test_allocation_with_relationships(
                0x1000,
                4,
                Some("counter"),
                Some("i32"),
                Some("main"),
                1000,
                Some(100),
            ),
            // Heap allocation (Vec)
            create_test_allocation_with_relationships(
                0x2000,
                1024,
                Some("data"),
                Some("Vec<u8>"),
                Some("main"),
                1100,
                Some(500),
            ),
            // Smart pointer
            create_test_allocation_with_relationships(
                0x3000,
                8,
                Some("boxed_value"),
                Some("Box<i64>"),
                Some("main"),
                1200,
                Some(300),
            ),
            // Reference
            create_test_allocation_with_relationships(
                0x4000,
                8,
                Some("reference"),
                Some("&str"),
                Some("helper"),
                1300,
                Some(50),
            ),
            // Adjacent memory allocation
            create_test_allocation_with_relationships(
                0x2000 + 1024, // Adjacent to data
                512,
                Some("buffer"),
                Some("Vec<u8>"),
                Some("main"),
                1400,
                Some(200),
            ),
        ];

        // Test direct analysis
        let analysis = VariableRelationshipAnalyzer::analyze_allocations(&allocations)
            .expect("Failed to get test value");

        // Verify graph structure
        assert_eq!(analysis.graph.nodes.len(), 5);
        assert_eq!(analysis.summary.total_variables, 5);
        assert!(analysis.summary.total_relationships > 0);

        // Verify node categories
        let stack_nodes = analysis
            .graph
            .nodes
            .iter()
            .filter(|n| n.category == NodeCategory::Stack || n.category == NodeCategory::Primitive)
            .count();
        let collection_nodes = analysis
            .graph
            .nodes
            .iter()
            .filter(|n| n.category == NodeCategory::Collection)
            .count();
        let smart_pointer_nodes = analysis
            .graph
            .nodes
            .iter()
            .filter(|n| n.category == NodeCategory::SmartPointer)
            .count();
        let reference_nodes = analysis
            .graph
            .nodes
            .iter()
            .filter(|n| n.category == NodeCategory::Reference)
            .count();

        assert!(stack_nodes > 0);
        assert!(collection_nodes > 0);
        assert!(smart_pointer_nodes > 0);
        assert!(reference_nodes > 0);

        // Verify ownership status distribution
        assert!(!analysis.summary.ownership_distribution.is_empty());
        assert!(analysis
            .summary
            .ownership_distribution
            .contains_key(&OwnershipStatus::Owner));

        // Verify relationship types
        assert!(!analysis.summary.relationship_distribution.is_empty());

        // Should have temporal relationships (consecutive allocations)
        let has_temporal = analysis
            .graph
            .links
            .iter()
            .any(|edge| edge.relationship == RelationshipType::Temporal);
        assert!(has_temporal);

        // Should have type similarity relationships (multiple Vec<u8>)
        let has_type_similarity = analysis
            .graph
            .links
            .iter()
            .any(|edge| edge.relationship == RelationshipType::TypeSimilarity);
        assert!(has_type_similarity);

        // Should have memory adjacency relationships
        let has_memory_adjacency = analysis
            .graph
            .links
            .iter()
            .any(|edge| edge.relationship == RelationshipType::MemoryAdjacency);
        assert!(has_memory_adjacency);

        // Verify D3.js compatibility
        for node in &analysis.graph.nodes {
            assert!(!node.id.is_empty());
            assert!(!node.name.is_empty());
            assert!(node.visual.radius > 0.0);
            assert!(!node.visual.color.is_empty());
            assert!(node.visual.opacity > 0.0 && node.visual.opacity <= 1.0);
        }

        for edge in &analysis.graph.links {
            assert!(!edge.source.is_empty());
            assert!(!edge.target.is_empty());
            assert!(edge.strength >= 0.0 && edge.strength <= 1.0);
            assert!(edge.visual.width > 0.0);
            assert!(!edge.visual.color.is_empty());
        }

        // Verify graph metadata for D3.js
        let metadata = &analysis.graph.metadata;
        assert_eq!(metadata.node_count, 5);
        assert_eq!(metadata.edge_count, analysis.graph.links.len());
        assert!(metadata.density >= 0.0 && metadata.density <= 1.0);
        assert_eq!(metadata.layout.algorithm, "force");
        assert!(metadata.layout.viewport.width > 0.0);
        assert!(metadata.layout.viewport.height > 0.0);

        // Verify performance optimization
        assert!(!analysis.optimization.lod_levels.is_empty());
        assert!(analysis.optimization.rendering.target_fps > 0);
        assert!(analysis.optimization.rendering.memory_budget > 0);
    }

    #[test]
    fn test_binary_html_writer_with_relationship_analysis() {
        // Create test allocations with relationships
        let allocations = vec![
            create_test_allocation_with_relationships(
                0x1000,
                1024,
                Some("vector"),
                Some("Vec<String>"),
                Some("main"),
                1000,
                Some(100),
            ),
            create_test_allocation_with_relationships(
                0x2000,
                8,
                Some("smart_ptr"),
                Some("Box<Vec<String>>"),
                Some("main"),
                1100,
                Some(150),
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
    fn test_node_categorization() {
        let allocations = vec![
            create_test_allocation_with_relationships(
                0x1000,
                4,
                Some("int_val"),
                Some("i32"),
                Some("main"),
                1000,
                Some(10),
            ),
            create_test_allocation_with_relationships(
                0x2000,
                1024,
                Some("vec_val"),
                Some("Vec<u8>"),
                Some("main"),
                1100,
                Some(100),
            ),
            create_test_allocation_with_relationships(
                0x3000,
                8,
                Some("box_val"),
                Some("Box<i32>"),
                Some("main"),
                1200,
                Some(50),
            ),
            create_test_allocation_with_relationships(
                0x4000,
                8,
                Some("ref_val"),
                Some("&str"),
                Some("main"),
                1300,
                Some(20),
            ),
            create_test_allocation_with_relationships(
                0x5000,
                8,
                Some("ptr_val"),
                Some("*mut u8"),
                Some("main"),
                1400,
                Some(30),
            ),
        ];

        let analysis = VariableRelationshipAnalyzer::analyze_allocations(&allocations)
            .expect("Failed to get test value");

        // Find nodes by type and verify categories
        let int_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "int_val")
            .expect("Test operation failed");
        assert_eq!(int_node.category, NodeCategory::Primitive);

        let vec_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "vec_val")
            .expect("Test operation failed");
        assert_eq!(vec_node.category, NodeCategory::Collection);

        let box_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "box_val")
            .expect("Test operation failed");
        assert_eq!(box_node.category, NodeCategory::SmartPointer);

        let ref_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "ref_val")
            .expect("Test operation failed");
        assert_eq!(ref_node.category, NodeCategory::Reference);

        let ptr_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "ptr_val")
            .expect("Test operation failed");
        assert_eq!(ptr_node.category, NodeCategory::RawPointer);
    }

    #[test]
    fn test_ownership_status_detection() {
        let allocations = vec![
            create_test_allocation_with_relationships(
                0x1000,
                4,
                Some("owned"),
                Some("i32"),
                Some("main"),
                1000,
                Some(100),
            ),
            create_test_allocation_with_relationships(
                0x2000,
                8,
                Some("borrowed"),
                Some("&i32"),
                Some("main"),
                1100,
                Some(50),
            ),
            create_test_allocation_with_relationships(
                0x3000,
                8,
                Some("mut_borrowed"),
                Some("&mut i32"),
                Some("main"),
                1200,
                Some(30),
            ),
            create_test_allocation_with_relationships(
                0x4000,
                8,
                Some("shared"),
                Some("Rc<i32>"),
                Some("main"),
                1300,
                Some(200),
            ),
            create_test_allocation_with_relationships(
                0x5000,
                8,
                Some("arc_shared"),
                Some("Arc<i32>"),
                Some("main"),
                1400,
                Some(300),
            ),
        ];

        let analysis = VariableRelationshipAnalyzer::analyze_allocations(&allocations)
            .expect("Failed to get test value");

        let owned_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "owned")
            .expect("Test operation failed");
        assert_eq!(owned_node.ownership, OwnershipStatus::Owner);

        let borrowed_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "borrowed")
            .expect("Test operation failed");
        assert_eq!(borrowed_node.ownership, OwnershipStatus::BorrowedImmutable);

        let mut_borrowed_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "mut_borrowed")
            .expect("Test operation failed");
        assert_eq!(
            mut_borrowed_node.ownership,
            OwnershipStatus::BorrowedMutable
        );

        let shared_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "shared")
            .expect("Test operation failed");
        assert_eq!(shared_node.ownership, OwnershipStatus::Shared);

        let arc_shared_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "arc_shared")
            .expect("Test operation failed");
        assert_eq!(arc_shared_node.ownership, OwnershipStatus::Shared);
    }

    #[test]
    fn test_lifetime_categorization() {
        let allocations = vec![
            create_test_allocation_with_relationships(
                0x1000,
                4,
                Some("instant"),
                Some("i32"),
                Some("main"),
                1000,
                Some(0),
            ),
            create_test_allocation_with_relationships(
                0x2000,
                4,
                Some("short"),
                Some("i32"),
                Some("main"),
                1100,
                Some(50),
            ),
            create_test_allocation_with_relationships(
                0x3000,
                4,
                Some("medium"),
                Some("i32"),
                Some("main"),
                1200,
                Some(500),
            ),
            create_test_allocation_with_relationships(
                0x4000,
                4,
                Some("long"),
                Some("i32"),
                Some("main"),
                1300,
                Some(5000),
            ),
            create_test_allocation_with_relationships(
                0x5000,
                4,
                Some("persistent"),
                Some("i32"),
                Some("main"),
                1400,
                Some(15000),
            ),
            create_test_allocation_with_relationships(
                0x6000,
                4,
                Some("active"),
                Some("i32"),
                Some("main"),
                1500,
                None,
            ),
        ];

        let analysis = VariableRelationshipAnalyzer::analyze_allocations(&allocations)
            .expect("Failed to get test value");

        let instant_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "instant")
            .expect("Test operation failed");
        assert_eq!(instant_node.lifetime.category, LifetimeCategory::Instant);

        let short_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "short")
            .expect("Test operation failed");
        assert_eq!(short_node.lifetime.category, LifetimeCategory::Short);

        let medium_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "medium")
            .expect("Test operation failed");
        assert_eq!(medium_node.lifetime.category, LifetimeCategory::Medium);

        let long_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "long")
            .expect("Test operation failed");
        assert_eq!(long_node.lifetime.category, LifetimeCategory::Long);

        let persistent_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "persistent")
            .expect("Test operation failed");
        assert_eq!(
            persistent_node.lifetime.category,
            LifetimeCategory::Persistent
        );

        let active_node = analysis
            .graph
            .nodes
            .iter()
            .find(|n| n.name == "active")
            .expect("Test operation failed");
        assert_eq!(active_node.lifetime.category, LifetimeCategory::Active);
        assert!(active_node.lifetime.is_active);
    }

    #[test]
    fn test_relationship_detection() {
        // Create allocations that should have specific relationships
        let allocations = vec![
            // Same type allocations (should have type similarity)
            create_test_allocation_with_relationships(
                0x1000,
                1024,
                Some("vec1"),
                Some("Vec<u8>"),
                Some("main"),
                1000,
                Some(100),
            ),
            create_test_allocation_with_relationships(
                0x2000,
                2048,
                Some("vec2"),
                Some("Vec<u8>"),
                Some("main"),
                1100,
                Some(200),
            ),
            // Adjacent memory allocations (should have memory adjacency)
            create_test_allocation_with_relationships(
                0x3000,
                512,
                Some("buf1"),
                Some("Vec<u8>"),
                Some("main"),
                1200,
                Some(150),
            ),
            create_test_allocation_with_relationships(
                0x3000 + 512,
                256,
                Some("buf2"),
                Some("Vec<u8>"),
                Some("main"),
                1300,
                Some(100),
            ),
            // Same scope allocations (should have containment)
            create_test_allocation_with_relationships(
                0x4000,
                4,
                Some("var1"),
                Some("i32"),
                Some("helper"),
                1400,
                Some(50),
            ),
            create_test_allocation_with_relationships(
                0x5000,
                4,
                Some("var2"),
                Some("f64"),
                Some("helper"),
                1500,
                Some(60),
            ),
        ];

        let analysis = VariableRelationshipAnalyzer::analyze_allocations(&allocations)
            .expect("Failed to get test value");

        // Should have type similarity relationships
        let type_similarity_count = analysis
            .graph
            .links
            .iter()
            .filter(|edge| edge.relationship == RelationshipType::TypeSimilarity)
            .count();
        assert!(type_similarity_count > 0);

        // Should have memory adjacency relationships
        let memory_adjacency_count = analysis
            .graph
            .links
            .iter()
            .filter(|edge| edge.relationship == RelationshipType::MemoryAdjacency)
            .count();
        assert!(memory_adjacency_count > 0);

        // Should have containment relationships (same scope)
        let containment_count = analysis
            .graph
            .links
            .iter()
            .filter(|edge| edge.relationship == RelationshipType::Containment)
            .count();
        assert!(containment_count > 0);

        // Should have temporal relationships (consecutive allocations)
        let temporal_count = analysis
            .graph
            .links
            .iter()
            .filter(|edge| edge.relationship == RelationshipType::Temporal)
            .count();
        assert!(temporal_count > 0);
    }

    #[test]
    fn test_d3js_compatibility() {
        let allocations = vec![
            create_test_allocation_with_relationships(
                0x1000,
                1024,
                Some("node1"),
                Some("Vec<u8>"),
                Some("main"),
                1000,
                Some(100),
            ),
            create_test_allocation_with_relationships(
                0x2000,
                2048,
                Some("node2"),
                Some("Box<i32>"),
                Some("main"),
                1100,
                Some(200),
            ),
        ];

        let analysis = VariableRelationshipAnalyzer::analyze_allocations(&allocations)
            .expect("Failed to get test value");

        // Verify D3.js node structure
        for node in &analysis.graph.nodes {
            // Required D3.js node properties
            assert!(!node.id.is_empty());
            assert!(node.visual.radius > 0.0);
            assert!(!node.visual.color.is_empty());
            assert!(node.visual.opacity >= 0.0 && node.visual.opacity <= 1.0);

            // Color should be valid hex
            assert!(node.visual.color.starts_with('#'));
            assert_eq!(node.visual.color.len(), 7);
        }

        // Verify D3.js edge structure
        for edge in &analysis.graph.links {
            // Required D3.js edge properties
            assert!(!edge.source.is_empty());
            assert!(!edge.target.is_empty());
            assert!(edge.strength >= 0.0 && edge.strength <= 1.0);
            assert!(edge.visual.width > 0.0);
            assert!(!edge.visual.color.is_empty());

            // Direction should be valid
            assert!(matches!(
                edge.direction,
                EdgeDirection::Directed | EdgeDirection::Undirected | EdgeDirection::Bidirectional
            ));
        }

        // Verify force simulation parameters
        let force_config = &analysis.graph.metadata.layout.force;
        assert!(force_config.link_strength >= 0.0 && force_config.link_strength <= 1.0);
        assert!(force_config.charge_strength < 0.0); // Should be negative for repulsion
        assert!(force_config.alpha_decay > 0.0 && force_config.alpha_decay < 1.0);
        assert!(force_config.velocity_decay > 0.0 && force_config.velocity_decay <= 1.0);

        // Verify viewport configuration
        let viewport = &analysis.graph.metadata.layout.viewport;
        assert!(viewport.width > 0.0);
        assert!(viewport.height > 0.0);
        assert!(viewport.zoom > 0.0);
    }

    #[test]
    fn test_performance_optimization() {
        // Create a smaller dataset to test performance optimizations (reduced from 150 to 20)
        let mut allocations = Vec::new();
        for i in 0..20 {
            allocations.push(create_test_allocation_with_relationships(
                0x1000 + i * 1024,
                1024,
                Some(&format!("var_{}", i)),
                Some("Vec<u8>"),
                Some("main"),
                1000 + i as u64,
                Some(100),
            ));
        }

        let analysis = VariableRelationshipAnalyzer::analyze_allocations(&allocations)
            .expect("Failed to get test value");

        // Should have performance optimizations configured
        let performance = &analysis.graph.metadata.performance;
        assert!(performance.max_visible_nodes > 0);
        assert!(performance.clustering_threshold > 0.0 && performance.clustering_threshold <= 1.0);

        // Should have level-of-detail configurations
        assert!(!analysis.optimization.lod_levels.is_empty());
        for lod in &analysis.optimization.lod_levels {
            assert!(lod.zoom_threshold > 0.0);
            assert!(lod.max_nodes > 0);
            assert!(lod.edge_simplification >= 0.0 && lod.edge_simplification <= 1.0);
        }

        // Should have rendering hints
        let rendering = &analysis.optimization.rendering;
        assert!(rendering.batch_size > 0);
        assert!(rendering.target_fps > 0);
        assert!(rendering.memory_budget > 0);
    }

    #[test]
    fn test_memory_insights() {
        let allocations = vec![
            create_test_allocation_with_relationships(
                0x1000,
                1024,
                Some("small"),
                Some("i32"),
                Some("main"),
                1000,
                Some(10),
            ),
            create_test_allocation_with_relationships(
                0x2000,
                2048,
                Some("medium"),
                Some("Vec<u8>"),
                Some("main"),
                1100,
                Some(500),
            ),
            create_test_allocation_with_relationships(
                0x3000,
                4096,
                Some("large"),
                Some("HashMap<String, i32>"),
                Some("main"),
                1200,
                Some(10000),
            ),
        ];

        let analysis = VariableRelationshipAnalyzer::analyze_allocations(&allocations)
            .expect("Failed to get test value");

        let insights = &analysis.summary.memory_insights;

        // Should calculate total memory correctly
        assert_eq!(insights.total_memory, 1024 + 2048 + 4096);

        // Should have lifetime distribution
        assert!(!insights.lifetime_distribution.is_empty());
        assert!(insights
            .lifetime_distribution
            .contains_key(&LifetimeCategory::Short));
        assert!(insights
            .lifetime_distribution
            .contains_key(&LifetimeCategory::Medium));
        assert!(insights
            .lifetime_distribution
            .contains_key(&LifetimeCategory::Persistent));

        // Should have optimization suggestions
        assert!(!insights.optimizations.is_empty());

        // Scores should be in valid ranges
        assert!(insights.fragmentation_score >= 0.0 && insights.fragmentation_score <= 1.0);
        assert!(insights.sharing_efficiency >= 0.0 && insights.sharing_efficiency <= 1.0);
    }
}
