//! Basic usage example for trace_tools memory visualizer.

use std::rc::Rc;
use std::sync::Arc;
use trace_tools::{get_global_tracker, init, track_var};

fn main() {
    // Initialize the memory tracking system
    init();
    println!("trace_tools initialized. Tracking memory allocations...");

    // Allocate and track simple types
    println!("\nAllocating and tracking variables...");

    let numbers_vec = vec![1, 2, 3, 4, 5];
    track_var!(numbers_vec).expect("Failed to track numbers_vec");
    println!("Tracked 'numbers_vec'");

    let text_string = String::from("Hello, Trace Tools!");
    track_var!(text_string).expect("Failed to track text_string");
    println!("Tracked 'text_string'");

    let boxed_value = Box::new(100i32);
    track_var!(boxed_value).expect("Failed to track boxed_value");
    println!("Tracked 'boxed_value'");

    let boxed_value2 = Box::new(200i32);
    track_var!(boxed_value2).expect("Failed to track boxed_value2");
    println!("Tracked 'boxed_value2'");

    // Track reference-counted types
    let rc_data = Rc::new(vec![10, 20, 30]);
    track_var!(rc_data).expect("Failed to track rc_data");
    println!("Tracked 'rc_data'");

    let arc_data = Arc::new(String::from("Shared data"));
    track_var!(arc_data).expect("Failed to track arc_data");
    println!("Tracked 'arc_data'");

    // Clone Rc to show shared ownership
    let rc_data_clone = Rc::clone(&rc_data);
    track_var!(rc_data_clone).expect("Failed to track rc_data_clone");
    println!("Tracked 'rc_data_clone' (shares allocation with 'rc_data')");

    // Perform some operations
    let sum_of_vec = numbers_vec.iter().sum::<i32>();
    println!("\nSum of 'numbers_vec': {}", sum_of_vec);
    println!("Length of 'text_string': {}", text_string.len());
    println!("Value in 'boxed_value': {}", *boxed_value);
    println!("Value in 'boxed_value2': {}", *boxed_value2);
    println!("First element of 'rc_data': {}", rc_data[0]);
    println!("Content of 'arc_data': {}", *arc_data);

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
        eprintln!("Failed to export JSON: {}", e);
    } else {
        println!("Successfully exported JSON.");
    }

    // Export memory usage visualization to SVG
    println!("\nExporting memory usage visualization to basic_usage_graph.svg...");
    if let Err(e) = tracker.export_to_svg("basic_usage_graph.svg") {
        eprintln!("Failed to export SVG: {}", e);
    } else {
        println!("Successfully exported SVG.");
    }

    println!("\nExample finished. Check 'basic_usage_snapshot.json' and 'basic_usage_graph.svg'.");
    println!("The SVG shows memory usage by type and individual allocations.");
}
