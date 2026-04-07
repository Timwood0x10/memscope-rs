//! Variable Relationships Showcase
//!
//! This example demonstrates all types of variable relationships that memscope can capture:
//! - Clone relationships
//! - Smart pointer relationships (Arc/Rc)
//! - Ownership transfers
//! - Type-based relationships
//! - Borrow relationships

use memscope_rs::{global_tracker, init_global_tracking, track};
use std::rc::Rc;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Variable Relationships Showcase");
    println!("===================================\n");

    init_global_tracking()?;
    let tracker = global_tracker()?;

    // 1. Clone relationships - same type and size
    println!("1. Clone Relationships (same type Vec<i32>)");
    let mut vec_clones = Vec::new();
    for i in 0..10 {
        let vec: Vec<i32> = vec![i, i * 2, i * 3];
        track!(tracker, vec);
        vec_clones.push(vec);
    }
    println!("✓ Created {} Vec<i32> clones", vec_clones.len());

    // 2. Smart pointer relationships - Arc
    println!("\n2. Smart Pointer Relationships (Arc)");
    let data = vec![1, 2, 3, 4, 5];
    let arc1 = Arc::new(data);
    track!(tracker, arc1.clone());

    for _i in 0..8 {
        let arc_clone = Arc::clone(&arc1);
        track!(tracker, arc_clone);
    }
    println!("✓ Created Arc with {} clones", 9);

    // 3. Smart pointer relationships - Rc
    println!("\n3. Smart Pointer Relationships (Rc)");
    let rc_data = "Hello, Rc!".to_string();
    let rc1 = Rc::new(rc_data);
    track!(tracker, rc1.clone());

    for _i in 0..6 {
        let rc_clone = Rc::clone(&rc1);
        track!(tracker, rc_clone);
    }
    println!("✓ Created Rc with {} clones", 7);

    // 4. String type relationships
    println!("\n4. String Type Relationships");
    let mut strings = Vec::new();
    for i in 0..12 {
        let s = format!("String number {}", i);
        track!(tracker, s);
        strings.push(s);
    }
    println!("✓ Created {} String instances", strings.len());

    // 5. HashMap type relationships
    println!("\n5. HashMap Type Relationships");
    let mut maps = Vec::new();
    for i in 0..5 {
        let mut map = std::collections::HashMap::new();
        for j in 0..10 {
            map.insert(format!("key_{}_{}", i, j), j * i);
        }
        track!(tracker, map);
        maps.push(map);
    }
    println!("✓ Created {} HashMap instances", maps.len());

    // 6. BTreeMap type relationships
    println!("\n6. BTreeMap Type Relationships");
    let mut btree_maps = Vec::new();
    for i in 0..4 {
        let mut map = std::collections::BTreeMap::new();
        for j in 0..8 {
            map.insert(j, format!("value_{}_{}", i, j));
        }
        track!(tracker, map);
        btree_maps.push(map);
    }
    println!("✓ Created {} BTreeMap instances", btree_maps.len());

    // 7. Vector<u8> type relationships (different from Vec<i32>)
    println!("\n7. Vector<u8> Type Relationships");
    let mut byte_vecs = Vec::new();
    for i in 0..8 {
        let vec: Vec<u8> = (0..128).map(|j| ((i * 128 + j) % 256) as u8).collect();
        track!(tracker, vec);
        byte_vecs.push(vec);
    }
    println!("✓ Created {} Vec<u8> instances", byte_vecs.len());

    // 8. Box type relationships
    println!("\n8. Box Type Relationships");
    let mut boxes = Vec::new();
    for i in 0..6 {
        let boxed: Box<i32> = Box::new(i * 100);
        track!(tracker, boxed);
        boxes.push(boxed);
    }
    println!("✓ Created {} Box<i32> instances", boxes.len());

    // Get statistics
    let stats = tracker.get_stats();
    println!("\n📊 Statistics:");
    println!("  Total allocations: {}", stats.total_allocations);
    println!("  Active allocations: {}", stats.active_allocations);
    println!("  Peak memory: {} bytes", stats.peak_memory_bytes);

    // Export results
    let output_path = "MemoryAnalysis/variable_relationships_showcase";
    tracker.export_html(output_path)?;
    tracker.export_json(output_path)?;

    println!("\n✅ Export successful!");
    println!("📁 Results saved to: {}", output_path);
    println!("\nExpected relationships in visualization:");
    println!("  🟢 Green: Vec<i32>, Vec<u8>, String, Box");
    println!("  🟣 Purple: Arc, Rc");
    println!("  🟠 Yellow: HashMap, BTreeMap");
    println!("  🔵 Blue: Other relationships");
    println!("  🔴 Red: Cycles (if any)");

    Ok(())
}
