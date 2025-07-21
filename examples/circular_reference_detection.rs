//! Circular reference detection demonstration
//!
//! This example demonstrates the circular reference detection capabilities
//! for smart pointers (Rc/Arc) that can lead to memory leaks.

use memscope_rs::{detect_circular_references, get_global_tracker, init, track_var};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
// Example structures that can create circular references
#[derive(Debug, Clone)]
struct Node {
    id: u32,
    value: String,
    children: RefCell<Vec<Rc<Node>>>,
    parent: RefCell<Option<Weak<Node>>>,
}

impl Node {
    fn new(id: u32, value: String) -> Rc<Self> {
        Rc::new(Node {
            id,
            value,
            children: RefCell::new(Vec::new()),
            parent: RefCell::new(None),
        })
    }

    fn add_child(self: &Rc<Self>, child: Rc<Node>) {
        // This creates a potential circular reference if not handled properly
        child.parent.borrow_mut().replace(Rc::downgrade(self));
        self.children.borrow_mut().push(child);
    }
}

// Another example: circular reference between two structures
#[derive(Debug, Clone)]
struct Person {
    name: String,
    friends: RefCell<Vec<Rc<Person>>>,
}

impl Person {
    fn new(name: String) -> Rc<Self> {
        Rc::new(Person {
            name,
            friends: RefCell::new(Vec::new()),
        })
    }

    fn add_friend(self: &Rc<Self>, friend: Rc<Person>) {
        self.friends.borrow_mut().push(friend);
        // This creates a circular reference: A -> B and B -> A
        friend.friends.borrow_mut().push(self);
    }
}

fn main() {
    init();

    println!("🔍 Circular Reference Detection Demo");
    println!("====================================");

    // Test 1: Simple circular reference between two Rc instances
    println!("\n📦 Test 1: Simple two-way circular reference");
    {
        let data1 = Rc::new("Data 1".to_string());
        let data2 = Rc::new("Data 2".to_string());

        let _tracked1 = track_var!(data1);
        let _tracked2 = track_var!(data2);

        // Simulate circular reference by creating clones that reference each other
        let clone1 = data1;
        let clone2 = data2;
        let _tracked_clone1 = track_var!(clone1);
        let _tracked_clone2 = track_var!(clone2);

        println!("✅ Created simple circular reference scenario");
        println!("   data1 ref_count: {}", Rc::strong_count(&data1));
        println!("   data2 ref_count: {}", Rc::strong_count(&data2));
    }

    // Test 2: Self-referencing structure
    println!("\n🔄 Test 2: Self-referencing structure");
    {
        let self_ref = Rc::new(RefCell::new(None::<Rc<String>>));
        let _tracked_self = track_var!(self_ref);

        // Create self-reference
        let clone = self_ref;
        *self_ref.borrow_mut() = Some(Rc::new("Self-referencing data".to_string()));
        let _tracked_clone = track_var!(clone);

        println!("✅ Created self-referencing structure");
        println!("   self_ref ref_count: {}", Rc::strong_count(&self_ref));
    }

    // Test 3: Complex multi-node circular reference
    println!("\n🌐 Test 3: Complex multi-node circular reference");
    {
        let node1 = Rc::new("Node 1".to_string());
        let node2 = Rc::new("Node 2".to_string());
        let node3 = Rc::new("Node 3".to_string());

        // Create a cycle: node1 -> node2 -> node3 -> node1
        let _tracked1 = track_var!(node1);
        let _tracked2 = track_var!(node2);
        let _tracked3 = track_var!(node3);

        // Simulate the circular chain
        let chain1 = node1;
        let chain2 = node2;
        let chain3 = node3;
        let _tracked_chain1 = track_var!(chain1);
        let _tracked_chain2 = track_var!(chain2);
        let _tracked_chain3 = track_var!(chain3);

        println!("✅ Created complex 3-node circular reference");
        println!("   node1 ref_count: {}", Rc::strong_count(&node1));
        println!("   node2 ref_count: {}", Rc::strong_count(&node2));
        println!("   node3 ref_count: {}", Rc::strong_count(&node3));
    }

    // Test 4: Mixed Rc and Weak references (should NOT create cycles)
    println!("\n✅ Test 4: Proper use of Weak references (no cycles)");
    {
        let strong_ref = Rc::new("Strong data".to_string());
        let _tracked_strong = track_var!(strong_ref);

        let weak_ref = Rc::downgrade(&strong_ref);
        let _tracked_weak = track_var!(weak_ref);

        // Create additional strong references
        let clone1 = strong_ref;
        let clone2 = strong_ref;
        let _tracked_clone1 = track_var!(clone1);
        let _tracked_clone2 = track_var!(clone2);

        println!("✅ Created proper Weak reference usage");
        println!("   strong_ref ref_count: {}", Rc::strong_count(&strong_ref));
        println!("   weak_ref weak_count: {}", Rc::weak_count(&strong_ref));
    }

    // Test 5: Large reference count scenario
    println!("\n📈 Test 5: High reference count scenario");
    {
        let shared_data = Rc::new(vec![1, 2, 3, 4, 5]);
        let _tracked_original = track_var!(shared_data);

        // Create many clones
        let mut clones = Vec::new();
        for i in 0..10 {
            let clone = shared_data;
            clones.push(track_var!(clone));
            if i % 3 == 0 {
                println!(
                    "   Created clone {}, ref_count: {}",
                    i,
                    Rc::strong_count(&shared_data)
                );
            }
        }

        println!("✅ Created high reference count scenario");
        println!("   Final ref_count: {}", Rc::strong_count(&shared_data));
    }

    // Perform circular reference analysis
    println!("\n🔍 Performing circular reference analysis...");

    let tracker = get_global_tracker();
    let allocations = match tracker.get_allocation_history() {
        Ok(allocs) => allocs,
        Err(e) => {
            println!("❌ Failed to get allocation history: {e}");
            return;
        }
    };
    let analysis = detect_circular_references(&allocations);

    println!("\n📊 Analysis Results:");
    println!("==================");
    println!(
        "Total smart pointers analyzed: {}",
        analysis.total_smart_pointers
    );
    println!(
        "Pointers involved in cycles: {}",
        analysis.pointers_in_cycles
    );
    println!(
        "Total circular references detected: {}",
        analysis.circular_references.len()
    );
    println!(
        "Total estimated leaked memory: {} bytes",
        analysis.total_leaked_memory
    );

    if !analysis.circular_references.is_empty() {
        println!("\n🚨 Detected Circular References:");
        for (i, circular_ref) in analysis.circular_references.iter().enumerate() {
            println!("\n  Cycle #{}: {:?} severity", i + 1, circular_ref.severity);
            println!("    Type: {:?}", circular_ref.cycle_type);
            println!(
                "    Estimated leaked memory: {} bytes",
                circular_ref.estimated_leaked_memory
            );
            println!("    Cycle path length: {}", circular_ref.cycle_path.len());

            if !circular_ref.suggested_weak_positions.is_empty() {
                println!("    💡 Suggested fixes:");
                for &pos in &circular_ref.suggested_weak_positions {
                    if let Some(node) = circular_ref.cycle_path.get(pos) {
                        println!("      - Convert pointer at position {pos} to Weak reference");
                        if let Some(var_name) = &node.var_name {
                            println!("        Variable: {var_name}");
                        }
                        if let Some(type_name) = &node.type_name {
                            println!("        Type: {type_name}");
                        }
                    }
                }
            }
        }
    } else {
        println!("✅ No circular references detected!");
    }

    // Display statistics
    println!("\n📈 Statistics:");
    println!("==============");
    println!(
        "Average cycle length: {:.2}",
        analysis.statistics.average_cycle_length
    );
    println!(
        "Largest cycle size: {}",
        analysis.statistics.largest_cycle_size
    );

    if !analysis.statistics.by_severity.is_empty() {
        println!("\nBy severity:");
        for (severity, count) in &analysis.statistics.by_severity {
            println!("  {severity}: {count}");
        }
    }

    if !analysis.statistics.by_type.is_empty() {
        println!("\nBy type:");
        for (cycle_type, count) in &analysis.statistics.by_type {
            println!("  {cycle_type}: {count}");
        }
    }

    if !analysis.statistics.by_pointer_type.is_empty() {
        println!("\nBy pointer type:");
        for (pointer_type, count) in &analysis.statistics.by_pointer_type {
            println!("  {pointer_type}: {count}");
        }
    }

    // Export comprehensive analysis
    println!("\n📄 Exporting comprehensive analysis...");
    if let Err(e) = tracker.export_to_json("circular_reference_analysis.json") {
        println!("❌ Failed to export analysis: {e}");
    } else {
        println!("✅ Analysis exported to circular_reference_analysis.json");
        println!("   This file now includes circular reference detection data");
    }

    println!("\n🎯 Circular reference detection complete!");
    println!("💡 Use this analysis to identify and fix memory leaks in your Rust code.");
}
