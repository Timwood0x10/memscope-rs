//! Lifecycle tracking example for trace_tools.
//! Demonstrates how variable lifecycles across different scopes are tracked.

use trace_tools::{get_global_tracker, init, track_var};

/// Function that allocates memory internally
fn create_and_drop_string() {
    println!("Entering create_and_drop_string()...");
    let local_string = String::from("This string is local to create_and_drop_string");
    track_var!(local_string).expect("Failed to track local_string");
    println!("Tracked 'local_string' with value: \"{}\"", local_string);
    println!("Exiting create_and_drop_string()...");
    // local_string goes out of scope here and memory is deallocated
}

/// Function that allocates memory and returns it (transferring ownership)
fn create_vec_on_heap() -> Vec<i32> {
    println!("\nEntering create_vec_on_heap()...");
    let heap_vec = vec![100, 200, 300];
    track_var!(heap_vec).expect("Failed to track heap_vec");
    println!("Tracked 'heap_vec' with content: {:?}", heap_vec);
    println!("Exiting create_vec_on_heap()...");
    heap_vec // Ownership transferred to caller
}

fn main() {
    // Initialize the memory tracking system
    init();
    println!("trace_tools initialized for lifecycle demo.");

    // Track a variable in the main scope
    println!("\nAllocating 'main_scope_vec'...");
    let main_scope_vec = vec![1, 2, 3];
    track_var!(main_scope_vec).expect("Failed to track main_scope_vec");
    println!("Tracked 'main_scope_vec'");

    // Call a function that allocates and deallocates internally
    println!("\nCalling create_and_drop_string()...");
    create_and_drop_string();
    println!("Returned from create_and_drop_string(). 'local_string' should be deallocated.");

    // Call a function that returns a heap-allocated variable
    println!("\nCalling create_vec_on_heap()...");
    let transferred_vec = create_vec_on_heap();
    // Re-associate the pointer with the new variable name
    track_var!(transferred_vec).expect("Failed to track transferred_vec");
    println!(
        "Tracked 'transferred_vec' (originally 'heap_vec') after ownership transfer: {:?}",
        transferred_vec
    );
    println!("Returned from create_vec_on_heap().");

    // Create variables within a loop to see multiple short-lived allocations
    println!("\nCreating variables inside a loop...");
    for i in 0..3 {
        let loop_string = format!("Loop string #{}", i);
        track_var!(loop_string).expect("Failed to track loop_string");
        println!("Tracked 'loop_string' iteration {}: \"{}\"", i, loop_string);
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

    // Export data
    println!("\nExporting memory snapshot to lifecycles_snapshot.json...");
    if let Err(e) = tracker.export_to_json("lifecycles_snapshot.json") {
        eprintln!("Failed to export JSON: {}", e);
    } else {
        println!("Successfully exported JSON.");
    }

    println!("\nExporting memory usage visualization to lifecycles_graph.svg...");
    if let Err(e) = tracker.export_to_svg("lifecycles_graph.svg") {
        eprintln!("Failed to export SVG: {}", e);
    } else {
        println!("Successfully exported SVG.");
    }

    println!(
        "\nLifecycle demo finished. Check 'lifecycles_snapshot.json' and 'lifecycles_graph.svg'."
    );
    println!("In the exports, observe how:");
    println!("- 'local_string' was allocated and deallocated within the function");
    println!("- 'heap_vec'/'transferred_vec' had its name updated after ownership transfer");
    println!("- Multiple 'loop_string' instances were created and destroyed");
    println!("- 'main_scope_vec' and 'transferred_vec' remain active until program end");

    // All remaining tracked variables go out of scope here
}
