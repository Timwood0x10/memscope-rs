//! Basic usage example for memscope-rs memory visualizer.

use memscope_rs::{get_global_tracker, init, track_var};
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    // Initialize the memory tracking system
    init();
    println!("memscope-rs initialized. Tracking memory allocations...");

    // Allocate and track simple types
    println!("\nAllocating and tracking variables...");

    let numbers_vec = vec![1, 2, 3, 4, 5];
    let tracked_numbers_vec = track_var!(numbers_vec);
    println!("Tracked 'numbers_vec'");

    let text_string = String::from("Hello, Trace Tools!");
    let tracked_text_string = track_var!(text_string);
    println!("Tracked 'text_string'");

    let boxed_value = Box::new(100i32);
    let tracked_boxed_value = track_var!(boxed_value);
    println!("Tracked 'boxed_value'");

    let boxed_value2 = Box::new(200i32);
    let tracked_boxed_value2 = track_var!(boxed_value2);
    println!("Tracked 'boxed_value2'");

    // Track reference-counted types
    let rc_data = Rc::new(vec![10, 20, 30]);
    let tracked_rc_data = track_var!(rc_data);
    println!("Tracked 'rc_data'");

    let arc_data = Arc::new(String::from("Shared data"));
    let tracked_arc_data = track_var!(arc_data);
    println!("Tracked 'arc_data'");

    // Clone Rc to show shared ownership
    let rc_data_clone = Rc::clone(&tracked_rc_data);
    let tracked_rc_data_clone = track_var!(rc_data_clone);
    println!("Tracked 'rc_data_clone' (shares allocation with 'rc_data')");

    // Perform some operations
    let sum_of_vec = tracked_numbers_vec.iter().sum::<i32>();
    println!("\nSum of 'numbers_vec': {sum_of_vec}");
    println!("Length of 'text_string': {}", tracked_text_string.len());
    println!("Value in 'boxed_value': {}", *tracked_boxed_value);
    println!("Value in 'boxed_value2': {}", *tracked_boxed_value2);
    println!("First element of 'rc_data': {}", tracked_rc_data[0]);
    println!("Content of 'arc_data': {}", *tracked_arc_data);

    // Get memory statistics
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("\nMemory Statistics:");
        println!("  Active allocations: {}", stats.active_allocations);
        println!("  Active memory: {} bytes", stats.active_memory);
        println!("  Total allocations: {}", stats.total_allocations);
        println!("  Peak memory: {} bytes", stats.peak_memory);
    }

    // Export memory snapshot to JSON
    println!("\nExporting memory snapshot to basic_usage_snapshot.json...");
    if let Err(e) = tracker.export_to_json("basic_usage_snapshot.json") {
        eprintln!("Failed to export JSON: {e}");
    } else {
        println!("Successfully exported JSON.");
    }

    // Export memory usage visualization to SVG
    println!("\nExporting memory usage visualization to basic_usage_graph.svg...");
    if let Err(e) = tracker.export_memory_analysis("basic_usage_graph.svg") {
        eprintln!("Failed to export SVG: {e}");
    } else {
        println!("Successfully exported SVG.");
    }

    println!("\nExample finished. Check 'basic_usage_snapshot.json' and 'basic_usage_graph.svg'.");
    println!("The SVG shows memory usage by type and individual allocations.");
}
