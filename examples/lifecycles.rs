//! examples/lifecycles.rs
//! Demonstrates how variable lifecycles across different scopes are tracked and visualized.

use trace_tools::{get_global_tracker, init, track_var}; // Removed Trackable

// Function that allocates memory internally
fn create_and_drop_string() {
    println!("Entering create_and_drop_string()...");
    let local_string = String::from("This string is local to create_and_drop_string");
    track_var!(local_string);
    println!("Tracked 'local_string' with value: \"{}\"", local_string);
    // local_string goes out of scope here, and its memory should be deallocated.
    println!("Exiting create_and_drop_string()...");
}

// Function that allocates memory and returns it (transferring ownership)
fn create_vec_on_heap() -> Vec<i32> {
    println!("\nEntering create_vec_on_heap()...");
    let heap_vec = vec![100, 200, 300];
    track_var!(heap_vec); // Tracking the Vec in this function's scope initially
    println!("Tracked 'heap_vec' with content: {:?}", heap_vec);
    println!("Exiting create_vec_on_heap()...");
    heap_vec // Ownership transferred to caller
}

fn main() {
    // 1. Initialize the memory tracking system.
    init();
    println!("trace_tools initialized for lifecycle demo.");

    // 2. Track a variable in the main scope.
    println!("\nAllocating 'main_scope_vec'...");
    let main_scope_vec = vec![1, 2, 3];
    track_var!(main_scope_vec);
    println!("Tracked 'main_scope_vec'");

    // 3. Call a function that allocates and deallocates internally.
    println!("\nCalling create_and_drop_string()...");
    create_and_drop_string();
    println!("Returned from create_and_drop_string(). 'local_string' should be deallocated.");

    // 4. Call a function that returns a heap-allocated variable.
    println!("\nCalling create_vec_on_heap()...");
    let transferred_vec = create_vec_on_heap();
    // Note: `track_var!(transferred_vec)` here would re-associate the pointer
    // (originally named 'heap_vec') with the name 'transferred_vec'.
    // This is generally the desired behavior when ownership is transferred and the variable name changes.
    track_var!(transferred_vec);
    println!(
        "Tracked 'transferred_vec' (originally 'heap_vec') after ownership transfer: {:?}",
        transferred_vec
    );
    println!("Returned from create_vec_on_heap().");

    // 5. Create variables within a loop to see multiple short-lived allocations.
    println!("\nCreating variables inside a loop...");
    for i in 0..3 {
        let loop_string = format!("Loop string #{}", i);
        track_var!(loop_string);
        println!("Tracked 'loop_string' iteration {}: \"{}\"", i, loop_string);
        // loop_string is deallocated at the end of each iteration.
    }
    println!("Finished loop.");

    // (main_scope_vec and transferred_vec are still alive here)

    // 6. Export data.
    let tracker = get_global_tracker();
    println!("\nExporting memory snapshot to lifecycles_snapshot.json...");
    if let Err(e) = tracker.export_to_json("lifecycles_snapshot.json", true) {
        // Enable sync for reliable file writing
        eprintln!("Failed to export JSON: {}", e);
    } else {
        println!("Successfully exported JSON.");
    }

    println!("\nExporting memory usage visualization to lifecycles_graph.svg...");
    if let Err(e) = tracker.export_to_svg("lifecycles_graph.svg", true) {
        // Enable sync for reliable file writing
        eprintln!("Failed to export SVG: {}", e);
    } else {
        println!("Successfully exported SVG.");
    }

    println!(
        "\nLifecycle demo finished. Check 'lifecycles_snapshot.json' and 'lifecycles_graph.svg'."
    );
    println!("In the SVG, observe how 'local_string' has a short lifecycle,");
    println!("'heap_vec'/'transferred_vec' has its name updated and persists longer,");
    println!("and multiple 'loop_string' instances appear and disappear.");

    // All remaining tracked variables (main_scope_vec, transferred_vec) go out of scope here.
}
