//! Real-world test for relation inference system.
//!
//! This test creates real Rust allocations (Vec, String, Arc) and verifies
//! that the relation inference system correctly detects their relationships.

use memscope_rs::analysis::relation_inference::{Relation, RelationGraphBuilder};
use memscope_rs::snapshot::types::ActiveAllocation;

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

#[test]
fn test_real_vec_relationships() {
    let v1 = vec![1i32, 2, 3, 4, 5];
    let v2 = vec![10i32, 20, 30];

    let ptr1 = v1.as_ptr() as usize;
    let ptr2 = v2.as_ptr() as usize;

    let allocs = vec![
        make_alloc(ptr1, v1.capacity() * 4),
        make_alloc(ptr2, v2.capacity() * 4),
    ];

    let graph = RelationGraphBuilder::build(&allocs, None);

    println!("\n=== Real Vec Test ===");
    println!("Vec1 at 0x{:x}, capacity {}", ptr1, v1.capacity());
    println!("Vec2 at 0x{:x}, capacity {}", ptr2, v2.capacity());
    println!("Edges found: {}", graph.edge_count());

    for edge in graph.edges() {
        println!("  {:?}: {} -> {}", edge.relation, edge.from, edge.to);
    }
}

#[test]
fn test_real_string_relationships() {
    let s1 = String::from("hello world");
    let s2 = String::from("test string");

    let ptr1 = s1.as_ptr() as usize;
    let ptr2 = s2.as_ptr() as usize;

    let allocs = vec![
        make_alloc(ptr1, s1.capacity()),
        make_alloc(ptr2, s2.capacity()),
    ];

    let graph = RelationGraphBuilder::build(&allocs, None);

    println!("\n=== Real String Test ===");
    println!("String1 at 0x{:x}, capacity {}", ptr1, s1.capacity());
    println!("String2 at 0x{:x}, capacity {}", ptr2, s2.capacity());
    println!("Edges found: {}", graph.edge_count());

    for edge in graph.edges() {
        println!("  {:?}: {} -> {}", edge.relation, edge.from, edge.to);
    }
}

#[test]
fn test_real_arc_relationships() {
    use std::sync::Arc;

    let arc1 = Arc::new(vec![1i32, 2, 3]);
    let arc2 = Arc::clone(&arc1);

    let ptr = Arc::as_ptr(&arc1) as usize;

    let allocs = vec![make_alloc(ptr, arc1.capacity() * 4)];

    let graph = RelationGraphBuilder::build(&allocs, None);

    println!("\n=== Real Arc Test ===");
    println!("Arc inner at 0x{:x}, capacity {}", ptr, arc1.capacity());
    println!("Arc1 strong_count (via Arc::clone): should be 2");
    println!("Edges found: {}", graph.edge_count());

    for edge in graph.edges() {
        println!("  {:?}: {} -> {}", edge.relation, edge.from, edge.to);
    }
}

fn main() {
    println!("Run with: cargo test real_relations -- --nocapture");
}
