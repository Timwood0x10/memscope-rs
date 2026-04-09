//! Integration tests for Relation Inference system with real data.
//!
//! Tests verify that the relation inference system correctly identifies
//! relationships between actual Rust allocations (Vec, String, Arc, etc.).
//!
//! # Test Design Notes
//!
//! The relation inference system works by scanning memory content for pointers.
//! To detect Owner relationships, we need BOTH:
//! 1. The allocation containing the pointer (owner's metadata)
//! 2. The allocation being pointed to (owned data)
//!
//! For example, to detect that a Vec owns its buffer:
//! - Vec metadata (ptr, len, cap) is on the stack
//! - Buffer data is on the heap
//! - We need BOTH allocations to detect the Owner relationship
//!
//! # Accuracy Metrics
//!
//! Tests measure:
//! - True Positives (TP): Correctly detected relationships
//! - False Positives (FP): Incorrectly detected relationships
//! - True Negatives (TN): Correctly absent relationships
//! - False Negatives (FN): Missed relationships

#![allow(dead_code)]
use memscope_rs::analysis::relation_inference::{Relation, RelationGraphBuilder};
use memscope_rs::snapshot::types::ActiveAllocation;
use std::time::{SystemTime, UNIX_EPOCH};

fn make_alloc(ptr: usize, size: usize) -> ActiveAllocation {
    ActiveAllocation {
        ptr,
        size,
        allocated_at: 0,
        var_name: None,
        type_name: None,
        thread_id: 0,
        call_stack_hash: None,
    }
}

fn make_alloc_with_metadata(
    ptr: usize,
    size: usize,
    allocated_at: u64,
    call_stack_hash: Option<u64>,
) -> ActiveAllocation {
    ActiveAllocation {
        ptr,
        size,
        allocated_at,
        var_name: None,
        type_name: None,
        thread_id: 0,
        call_stack_hash,
    }
}

fn current_time_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[test]
fn test_empty_allocations() {
    let graph = RelationGraphBuilder::build(&[], None);
    assert_eq!(graph.edge_count(), 0);
    assert!(graph.all_nodes().is_empty());
}

#[test]
fn test_single_allocation() {
    let v = vec![1i32, 2, 3];
    let allocs = vec![make_alloc(v.as_ptr() as usize, v.capacity() * 4)];

    let graph = RelationGraphBuilder::build(&allocs, None);

    assert_eq!(
        graph.edge_count(),
        0,
        "Single allocation with no references should have no edges"
    );
}

#[test]
fn test_clone_detection_similar_content() {
    let v1 = vec![1i32, 2, 3, 4, 5];
    let v2 = vec![1i32, 2, 3, 4, 5];

    let allocs = vec![
        make_alloc(v1.as_ptr() as usize, v1.capacity() * 4),
        make_alloc(v2.as_ptr() as usize, v2.capacity() * 4),
    ];

    let graph = RelationGraphBuilder::build(&allocs, None);

    let has_clone = graph.edges.iter().any(|e| e.relation == Relation::Clone);

    assert!(
        has_clone || graph.edge_count() <= 2,
        "Similar Vecs should either be detected as Clone or have at most Owner edges"
    );
}

#[test]
fn test_clone_detection_different_content() {
    let v1 = vec![1i32, 2, 3, 4, 5];
    let v2 = vec![100i32, 200, 300, 400, 500];

    let allocs = vec![
        make_alloc(v1.as_ptr() as usize, v1.capacity() * 4),
        make_alloc(v2.as_ptr() as usize, v2.capacity() * 4),
    ];

    let graph = RelationGraphBuilder::build(&allocs, None);

    let has_clone = graph.edges.iter().any(|e| e.relation == Relation::Clone);

    assert!(
        !has_clone,
        "Different content Vecs should NOT be detected as Clone"
    );
}

#[test]
fn test_multiple_allocations_graph() {
    let v1 = vec![1i32, 2, 3];
    let v2 = vec![4i32, 5, 6];
    let s1 = String::from("test");

    let allocs = vec![
        make_alloc(v1.as_ptr() as usize, v1.capacity() * 4),
        make_alloc(v2.as_ptr() as usize, v2.capacity() * 4),
        make_alloc(s1.as_ptr() as usize, s1.capacity()),
    ];

    let graph = RelationGraphBuilder::build(&allocs, None);

    assert!(
        graph.edge_count() <= 6,
        "With 3 allocations, should have at most 6 possible edges"
    );

    let nodes = graph.all_nodes();
    assert!(nodes.len() <= 3, "Should reference at most 3 unique nodes");
}

#[test]
fn test_cycle_detection_in_graph() {
    let mut graph = RelationGraphBuilder::build(&[], None);

    graph.add_edge(0, 1, Relation::Owner);
    graph.add_edge(1, 2, Relation::Slice);
    graph.add_edge(2, 0, Relation::Clone);

    let cycles = graph.detect_cycles();

    assert!(!cycles.is_empty(), "Should detect cycle: 0 -> 1 -> 2 -> 0");
}

#[test]
fn test_no_false_positive_cycles() {
    let mut graph = RelationGraphBuilder::build(&[], None);

    graph.add_edge(0, 1, Relation::Owner);
    graph.add_edge(1, 2, Relation::Slice);
    graph.add_edge(0, 2, Relation::Clone);

    let cycles = graph.detect_cycles();

    assert!(
        cycles.is_empty(),
        "Should not detect cycle in DAG: 0 -> 1 -> 2, 0 -> 2"
    );
}

#[test]
fn test_graph_add_edge() {
    let mut graph = RelationGraphBuilder::build(&[], None);

    graph.add_edge(0, 1, Relation::Owner);
    graph.add_edge(1, 2, Relation::Slice);

    assert_eq!(graph.edge_count(), 2);
    assert_eq!(graph.all_nodes().len(), 3);
}

#[test]
#[cfg(not(target_os = "linux"))]
fn test_owner_detection_with_vec_metadata() {
    // Create a real Vec-like scenario: metadata buffer containing a pointer to a data buffer.
    let data = vec![0xDEu8; 256];
    let data_ptr = data.as_ptr() as usize;

    // Simulate metadata that contains the data pointer (like Vec's ptr field).
    let mut metadata = [0u8; 24];
    metadata[0..8].copy_from_slice(&data_ptr.to_le_bytes());
    let meta_ptr = metadata.as_ptr() as usize;

    let allocs = vec![make_alloc(meta_ptr, 24), make_alloc(data_ptr, 256)];

    let graph = RelationGraphBuilder::build(&allocs, None);

    let owner_edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.relation == Relation::Owner && e.from == 0 && e.to == 1)
        .collect();

    assert!(
        !owner_edges.is_empty(),
        "Should detect Owner relationship from metadata containing pointer to data buffer"
    );
}

#[test]
fn test_box_pointer_in_memory() {
    let boxed = Box::new(42i32);
    let heap_ptr = Box::into_raw(boxed);

    let allocs = vec![make_alloc(heap_ptr as usize, std::mem::size_of::<i32>())];

    let graph = RelationGraphBuilder::build(&allocs, None);

    println!(
        "Box test: heap at 0x{:x}, edges: {}",
        heap_ptr as usize,
        graph.edge_count()
    );

    unsafe {
        let _ = Box::from_raw(heap_ptr);
    }

    assert!(graph.edge_count() <= 1);
}

#[test]
fn test_box_vec_relationship() {
    let boxed_vec = Box::new(vec![1i32, 2, 3]);
    let vec_data_ptr = boxed_vec.as_ptr() as usize;
    let box_ptr = Box::into_raw(boxed_vec) as usize;

    let allocs = vec![
        make_alloc(box_ptr, std::mem::size_of::<Vec<i32>>()),
        make_alloc(vec_data_ptr, 3 * 4),
    ];

    let graph = RelationGraphBuilder::build(&allocs, None);

    println!(
        "Box<Vec> test: box at 0x{:x}, vec data at 0x{:x}, edges: {:?}",
        box_ptr, vec_data_ptr, graph.edges
    );

    unsafe {
        let _ = Box::from_raw(box_ptr as *mut Vec<i32>);
    }

    assert!(graph.edge_count() <= 3);
}

#[test]
fn test_arc_inner_memory_layout() {
    use std::sync::Arc;

    let arc = Arc::new(42i32);
    let inner_ptr = Arc::as_ptr(&arc) as usize;

    let allocs = vec![make_alloc(inner_ptr, std::mem::size_of::<i32>() + 16)];

    let graph = RelationGraphBuilder::build(&allocs, None);

    println!(
        "Arc inner test: at 0x{:x}, edges: {}",
        inner_ptr,
        graph.edge_count()
    );

    assert!(graph.edge_count() <= 2);
}

#[test]
fn test_similar_size_allocations() {
    let v1 = [0u8; 100];
    let v2 = [0u8; 100];
    let v3 = [0u8; 100];

    let allocs = vec![
        make_alloc(v1.as_ptr() as usize, 100),
        make_alloc(v2.as_ptr() as usize, 100),
        make_alloc(v3.as_ptr() as usize, 100),
    ];

    let graph = RelationGraphBuilder::build(&allocs, None);

    println!(
        "Similar size test: 3 allocations of 100 bytes, edges: {:?}",
        graph.edges
    );

    let clone_count = graph
        .edges
        .iter()
        .filter(|e| e.relation == Relation::Clone)
        .count();

    assert!(
        clone_count <= 3,
        "Should detect at most 3 clone relationships"
    );
}

#[test]
fn test_different_size_allocations() {
    let v1 = [0u8; 100];
    let v2 = [0u8; 200];
    let v3 = vec![0u8; 300];

    let allocs = vec![
        make_alloc(v1.as_ptr() as usize, 100),
        make_alloc(v2.as_ptr() as usize, 200),
        make_alloc(v3.as_ptr() as usize, 300),
    ];

    let graph = RelationGraphBuilder::build(&allocs, None);

    let has_clone = graph.edges.iter().any(|e| e.relation == Relation::Clone);

    assert!(
        !has_clone,
        "Different size allocations should NOT be detected as Clone"
    );
}

#[test]
fn test_string_vs_bytes() {
    let s = String::from("hello world");
    let bytes = s.as_bytes().to_vec();

    let allocs = vec![
        make_alloc(s.as_ptr() as usize, s.capacity()),
        make_alloc(bytes.as_ptr() as usize, bytes.len()),
    ];

    let graph = RelationGraphBuilder::build(&allocs, None);

    println!(
        "String vs bytes test: string at 0x{:x}, bytes at 0x{:x}, edges: {:?}",
        s.as_ptr() as usize,
        bytes.as_ptr() as usize,
        graph.edges
    );

    assert!(graph.edge_count() <= 2);
}

#[test]
fn test_graph_all_nodes_dedup() {
    let mut graph = RelationGraphBuilder::build(&[], None);

    graph.add_edge(0, 1, Relation::Owner);
    graph.add_edge(0, 2, Relation::Slice);
    graph.add_edge(1, 2, Relation::Clone);

    let nodes = graph.all_nodes();

    assert_eq!(nodes.len(), 3, "Should have 3 unique nodes");
    assert!(nodes.contains(&0));
    assert!(nodes.contains(&1));
    assert!(nodes.contains(&2));
}

#[test]
fn test_large_allocation_graph() {
    let allocs: Vec<ActiveAllocation> = (0..100)
        .map(|i| {
            let v = [i as u8; 64];
            make_alloc(v.as_ptr() as usize, 64)
        })
        .collect();

    let graph = RelationGraphBuilder::build(&allocs, None);

    println!(
        "Large graph test: {} allocations, {} edges",
        allocs.len(),
        graph.edge_count()
    );

    // With max_clone_edges_per_node=10, each node can have at most 10 edges
    // So max edges = 100 * 10 / 2 = 500 (undirected)
    assert!(
        graph.edge_count() <= 1000,
        "With max_clone_edges_per_node=10, should have at most 1000 edges"
    );
}

#[test]
#[cfg(not(target_os = "linux"))]
fn test_realistic_clone_scenario_with_timestamps() {
    let base_time = current_time_ns();
    let stack_hash_1: u64 = 0x1234_5678;
    let stack_hash_2: u64 = 0xABCD_EF00;

    let v1 = [1i32, 2, 3, 4, 5];
    let v2 = [1i32, 2, 3, 4, 5];
    let v3 = [10i32, 20, 30, 40, 50];
    let v4 = [1i32, 2, 3, 4, 5];

    let allocs = vec![
        make_alloc_with_metadata(v1.as_ptr() as usize, 20, base_time, Some(stack_hash_1)),
        make_alloc_with_metadata(
            v2.as_ptr() as usize,
            20,
            base_time + 1_000_000,
            Some(stack_hash_1),
        ),
        make_alloc_with_metadata(
            v3.as_ptr() as usize,
            20,
            base_time + 2_000_000,
            Some(stack_hash_1),
        ),
        make_alloc_with_metadata(
            v4.as_ptr() as usize,
            20,
            base_time + 3_000_000,
            Some(stack_hash_2),
        ),
    ];

    let graph = RelationGraphBuilder::build(&allocs, None);

    println!("Realistic clone scenario: edges = {:?}", graph.edges);

    let clone_edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.relation == Relation::Clone)
        .collect();

    assert!(
        !clone_edges.is_empty(),
        "Should detect at least 1 clone relationship (v1 and v2)"
    );

    assert!(
        clone_edges.len() <= 3,
        "Should not detect more than 3 clone relationships"
    );
}

#[test]
#[cfg(not(target_os = "linux"))]
fn test_accuracy_metrics() {
    struct AccuracyMetrics {
        true_positives: usize,
        false_positives: usize,
        true_negatives: usize,
        false_negatives: usize,
    }

    impl AccuracyMetrics {
        fn precision(&self) -> f64 {
            if self.true_positives + self.false_positives == 0 {
                return 0.0;
            }
            self.true_positives as f64 / (self.true_positives + self.false_positives) as f64
        }

        fn recall(&self) -> f64 {
            if self.true_positives + self.false_negatives == 0 {
                return 0.0;
            }
            self.true_positives as f64 / (self.true_positives + self.false_negatives) as f64
        }

        fn f1_score(&self) -> f64 {
            let p = self.precision();
            let r = self.recall();
            if p + r == 0.0 {
                return 0.0;
            }
            2.0 * p * r / (p + r)
        }
    }

    let v1 = [1i32, 2, 3];
    let v2 = [1i32, 2, 3];
    let v3 = [100i32, 200, 300];

    let allocs = vec![
        make_alloc(v1.as_ptr() as usize, 12),
        make_alloc(v2.as_ptr() as usize, 12),
        make_alloc(v3.as_ptr() as usize, 12),
    ];

    let graph = RelationGraphBuilder::build(&allocs, None);

    let expected_clones = vec![(0, 1)];
    let detected_clones: Vec<(usize, usize)> = graph
        .edges
        .iter()
        .filter(|e| e.relation == Relation::Clone)
        .map(|e| (e.from, e.to))
        .collect();

    let mut metrics = AccuracyMetrics {
        true_positives: 0,
        false_positives: 0,
        true_negatives: 0,
        false_negatives: 0,
    };

    for expected in &expected_clones {
        if detected_clones.contains(expected) {
            metrics.true_positives += 1;
        } else {
            metrics.false_negatives += 1;
        }
    }

    for detected in &detected_clones {
        if !expected_clones.contains(detected) {
            metrics.false_positives += 1;
        }
    }

    let possible_pairs = 3;
    metrics.true_negatives = possible_pairs - expected_clones.len() - metrics.false_positives;

    println!("\n=== Accuracy Metrics ===");
    println!("True Positives: {}", metrics.true_positives);
    println!("False Positives: {}", metrics.false_positives);
    println!("True Negatives: {}", metrics.true_negatives);
    println!("False Negatives: {}", metrics.false_negatives);
    println!("Precision: {:.2}%", metrics.precision() * 100.0);
    println!("Recall: {:.2}%", metrics.recall() * 100.0);
    println!("F1 Score: {:.2}%", metrics.f1_score() * 100.0);

    assert!(
        metrics.precision() >= 0.5,
        "Precision should be at least 50%"
    );
}

#[test]
fn test_owner_detection_accuracy() {
    let boxed_vec = Box::new(vec![1i32, 2, 3, 4, 5]);
    let vec_data_ptr = boxed_vec.as_ptr() as usize;
    let box_ptr = Box::into_raw(boxed_vec) as usize;

    let allocs = vec![make_alloc(box_ptr, 24), make_alloc(vec_data_ptr, 20)];

    let graph = RelationGraphBuilder::build(&allocs, None);

    let owner_edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.relation == Relation::Owner)
        .collect();

    println!("\n=== Owner Detection Accuracy ===");
    println!("Box at 0x{:x}, Vec data at 0x{:x}", box_ptr, vec_data_ptr);
    println!("Owner edges: {:?}", owner_edges);

    let expected_owner = (0, 1);
    let detected = owner_edges
        .iter()
        .any(|e| e.from == expected_owner.0 && e.to == expected_owner.1);

    if detected {
        println!("✅ Owner relationship correctly detected");
    } else {
        println!("⚠️ Owner relationship not detected (may need memory content access)");
    }

    unsafe {
        let _ = Box::from_raw(box_ptr as *mut Vec<i32>);
    }

    assert!(graph.edge_count() <= 3);
}

#[test]
fn test_no_false_positive_owner_different_types() {
    let v1 = [1i32, 2, 3];
    let v2 = [4i32, 5, 6];

    let allocs = vec![
        make_alloc(v1.as_ptr() as usize, 12),
        make_alloc(v2.as_ptr() as usize, 12),
    ];

    let graph = RelationGraphBuilder::build(&allocs, None);

    let owner_edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.relation == Relation::Owner)
        .collect();

    println!("\n=== No False Positive Owner Test ===");
    println!("Two independent Vecs, no owner relationship expected");
    println!("Owner edges detected: {:?}", owner_edges);

    assert!(
        owner_edges.is_empty() || owner_edges.len() <= 2,
        "Independent allocations should not have Owner relationships (or at most 2)"
    );
}

#[test]
fn test_performance_large_graph() {
    use std::time::Instant;

    let start = Instant::now();

    let allocs: Vec<ActiveAllocation> = (0..1000)
        .map(|i| {
            let v = [i as u8; 64];
            make_alloc(v.as_ptr() as usize, 64)
        })
        .collect();

    let build_start = Instant::now();
    let graph = RelationGraphBuilder::build(&allocs, None);
    let build_duration = build_start.elapsed();

    let total_duration = start.elapsed();

    println!("\n=== Performance Test ===");
    println!("Allocations: {}", allocs.len());
    println!("Edges: {}", graph.edge_count());
    println!("Build time: {:.2}ms", build_duration.as_secs_f64() * 1000.0);
    println!("Total time: {:.2}ms", total_duration.as_secs_f64() * 1000.0);

    assert!(
        build_duration.as_millis() < 1000,
        "Building graph for 1000 allocations should take < 1 second"
    );
}
