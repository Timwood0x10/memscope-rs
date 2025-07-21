//! Simplified lifecycle tracking example for memscope-rs.
//! Only exports the two essential SVG files.

use memscope_rs::{get_global_tracker, init, track_var};

/// Function that allocates memory internally
fn create_and_drop_string() {
    println!("Entering create_and_drop_string()...");
    let local_string = String::from("This string is local to create_and_drop_string");
    println!("Tracked 'local_string' with value: \"{local_string}\"");
    let _tracked_local_string = track_var!(local_string);
    println!("Exiting create_and_drop_string()...");
    // local_string goes out of scope here and memory is deallocated
}

/// Function that allocates memory and returns it (transferring ownership)
fn create_vec_on_heap() -> Vec<i32> {
    println!("\nEntering create_vec_on_heap()...");
    let heap_vec = vec![100, 200, 300];
    println!("Tracked 'heap_vec' with content: {heap_vec:?}");
    let _tracked_heap_vec = track_var!(heap_vec);
    println!("Exiting create_vec_on_heap()...");
    heap_vec // Ownership transferred to caller
}

fn main() {
    // Initialize the memory tracking system
    init();
    println!("memscope-rs initialized for lifecycle demo.");

    // Track a variable in the main scope
    println!("\nAllocating 'main_scope_vec'...");
    let main_scope_vec = vec![1, 2, 3];
    let _tracked_main_scope_vec = track_var!(main_scope_vec);
    println!("Tracked 'main_scope_vec'");

    // Call a function that allocates and deallocates internally
    println!("\nCalling create_and_drop_string()...");
    create_and_drop_string();
    println!("Returned from create_and_drop_string(). 'local_string' should be deallocated.");

    // Call a function that returns a heap-allocated variable
    println!("\nCalling create_vec_on_heap()...");
    let transferred_vec = create_vec_on_heap();
    // Re-associate the pointer with the new variable name
    println!(
        "Tracked 'transferred_vec' (originally 'heap_vec') after ownership transfer: {transferred_vec:?}"
    );
    let _tracked_transferred_vec = track_var!(transferred_vec);
    println!("Returned from create_vec_on_heap().");

    // Create variables within a loop to see multiple short-lived allocations
    println!("\nCreating variables inside a loop...");
    for i in 0..3 {
        let loop_string = format!("Loop string #{i}");
        println!("Tracked 'loop_string' iteration {i}: \"{loop_string}\"");
        let _tracked_loop_string = track_var!(loop_string);
        // loop_string is deallocated at the end of each iteration
    }
    println!("Finished loop.");

    // Show current memory state
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("\nCurrent Memory Statistics:");
        println!("  Active allocations: {}", stats.active_allocations);
        println!("  Active memory: {} bytes", stats.active_memory);
        println!("  Total allocations: {}", stats.total_allocations);
        println!("  Total deallocations: {}", stats.total_deallocations);
    }

    // Export only the two essential SVG files
    println!("\n=== Exporting Essential Visualizations ===");

    // Export 1: Memory Analysis - Shows variable names, types, and memory usage
    println!("1. Exporting memory analysis to program_memory_analysis.svg...");
    if let Err(e) = tracker.export_memory_analysis("program_memory_analysis.svg") {
        eprintln!("Failed to export memory analysis: {e}");
    } else {
        println!("   Memory analysis SVG exported successfully.");
    }

    // Export 2: Lifecycle Timeline - Interactive timeline with variable lifecycles
    println!("2. Exporting lifecycle timeline to program_lifecycle.svg...");
    if let Err(e) = tracker.export_lifecycle_timeline("program_lifecycle.svg") {
        eprintln!("Failed to export lifecycle timeline: {e}");
    } else {
        println!("   Lifecycle timeline SVG exported successfully.");
    }

    println!("\n=== Export Complete ===");
    println!("Generated files:");
    println!("  - program_memory_analysis.svg: Memory usage with variable names and types");
    println!("  - program_lifecycle.svg: Interactive timeline showing variable lifecycles");
    println!("\nLifecycle demo finished. Check the two SVG files for complete analysis.");

    // All remaining tracked variables go out of scope here
}
