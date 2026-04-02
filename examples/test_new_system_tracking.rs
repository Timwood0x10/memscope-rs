//! New System Tracking Example - Using tracker! and track! macros
//!
//! This example demonstrates the new system's tracker! and track! macros
//! which export the same detailed JSON format with borrow_info, clone_info, etc.

use memscope_rs::{track, tracker};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 New System Tracking Example");
    println!("═══════════════════════════════════════\n");

    // Create a tracker using the new tracker! macro
    let tracker = tracker!();
    println!("✅ Tracker created using tracker!() macro");

    // Allocate and track variables using track! macro
    println!("\n📊 Allocating and tracking variables...");

    // Track simple types
    let numbers_vec = vec![1, 2, 3, 4, 5];
    track!(tracker, numbers_vec);
    println!("✅ Tracked 'numbers_vec' (Vec<i32>)");

    let mut text_string = String::from("Hello, MemScope!");
    track!(tracker, text_string);
    println!("✅ Tracked 'text_string' (String)");

    let boxed_value = Box::new(100i32);
    track!(tracker, boxed_value);
    println!("✅ Tracked 'boxed_value' (Box<i32>)");

    // Track reference-counted types
    use std::rc::Rc;
    use std::sync::Arc;

    let rc_data = Rc::new(vec![10, 20, 30]);
    track!(tracker, rc_data);
    println!("✅ Tracked 'rc_data' (Rc<Vec<i32>>)");

    let arc_data = Arc::new(String::from("Shared data"));
    track!(tracker, arc_data);
    println!("✅ Tracked 'arc_data' (Arc<String>)");

    // Clone Rc to show shared ownership
    let rc_data_clone = Rc::clone(&rc_data);
    track!(tracker, rc_data_clone);
    println!("✅ Tracked 'rc_data_clone' (Rc<Vec<i32>> - shared ownership)");

    // Create variable relationships
    println!("\n🔗 Creating variable relationships...");
    
    // Create a vector that contains references to other variables
    let mut container = Vec::new();
    container.push(arc_data.clone());
    container.push(arc_data.clone());
    track!(tracker, container);
    println!("✅ Tracked 'container' (Vec<Arc<String>>) - contains arc_data clones");

    // Create nested structures
    let nested = vec![rc_data.clone(), rc_data.clone()];
    track!(tracker, nested);
    println!("✅ Tracked 'nested' (Vec<Rc<Vec<i32>>>) - contains rc_data clones");

    // Create another nested structure
    let deep_nested = vec![container.clone(), container.clone()];
    track!(tracker, deep_nested);
    println!("✅ Tracked 'deep_nested' (Vec<Vec<Arc<String>>>) - contains container clones");

    // Perform some operations to generate borrow info
    println!("\n🔧 Performing operations to generate borrow info...");
    
    // Immutable borrows
    let _immutable_borrow = &numbers_vec;
    let _immutable_borrow2 = &numbers_vec;
    let _immutable_borrow3 = &text_string;
    
    // Mutable borrow
    let current_value = text_string.clone();
    let mutable_borrow = &mut text_string;
    *mutable_borrow = format!("{} - Modified", current_value);
    
    // More immutable borrows
    let _immutable_borrow4 = &numbers_vec;
    let _immutable_borrow5 = &text_string;

    println!("✅ Operations completed - borrow info generated");

    // Export detailed JSON using the new system
    println!("\n📄 Exporting detailed JSON using new system...");
    
    match tracker.export_json("new_system_tracking") {
        Ok(_) => {
            println!("✅ New system JSON export successful!");
            println!("   📁 Files saved to MemoryAnalysis/new_system_tracking_snapshot_analysis/");
            println!("   📄 memory_analysis.json - with borrow_info, clone_info, ownership_history");
            println!("   📄 lifetime.json - detailed ownership events");
            println!("   📄 unsafe_ffi.json - unsafe operations analysis");
            println!("   📄 variable_relationships.json - variable relationships");
            println!("   📄 type_analysis.json - type analysis");
        }
        Err(e) => eprintln!("❌ New system JSON export failed: {e}"),
    }

    // Also export analysis report
    println!("\n📊 Exporting analysis report...");
    match tracker.export_analysis("new_system_analysis") {
        Ok(_) => {
            println!("✅ Analysis report exported successfully!");
            println!("   📁 File saved to MemoryAnalysis/new_system_analysis_analysis.json");
            println!("   📊 Contains: total allocations, peak memory, hotspots, system snapshots");
        }
        Err(e) => eprintln!("❌ Analysis report export failed: {e}"),
    }

    // Show system information
    println!("\n🖥️  System Information:");
    let snapshot = tracker.current_system_snapshot();
    println!("   CPU Usage: {:.2}%", snapshot.cpu_usage_percent);
    println!("   Memory Usage: {:.2}%", snapshot.memory_usage_percent);
    println!("   Thread Count: {}", snapshot.thread_count);
    println!("   GPU Usage: {:.2}%", snapshot.gpu_usage_percent);

    println!("\n🎉 Test completed successfully!");
    println!("\n📁 Check the exported files in:");
    println!("   MemoryAnalysis/new_system_tracking_snapshot_analysis/");
    println!("   MemoryAnalysis/new_system_analysis_analysis.json");
    println!("\nThese files contain the detailed data you're familiar with:");
    println!("   • borrow_info: immutable and mutable borrow counts");
    println!("   • clone_info: clone operations");
    println!("   • type_name: detailed type information");
    println!("   • var_name: variable names");
    println!("   • lifetime_ms: variable lifetimes");
    println!("   • ownership_history_available: detailed ownership tracking");
    println!("   • system_snapshots: CPU, memory, GPU usage");
    println!("   • hotspots: allocation hotspots");
    println!("═══════════════════════════════════════════\n");

    Ok(())
}