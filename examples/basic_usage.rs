//! examples/basic_usage.rs
//! Demonstrates basic usage of the trace_tools memory visualizer.

use std::rc::Rc;
use std::sync::Arc;
use trace_tools::{get_global_tracker, init, track_var};

fn main() {
    // 1. Initialize the memory tracking system.
    // This sets up the tracing subscriber and prepares the global tracker.
    // If the `tracking-allocator` feature is enabled (default), this also
    // ensures our custom global allocator is in place.
    init();
    println!("trace_tools initialized. Tracking memory allocations...");

    // 2. Allocate some simple types and track them.
    println!(
        "
Allocating and tracking variables..."
    );

    let numbers_vec = vec![1, 2, 3, 4, 5];
    track_var!(numbers_vec); // Track the Vec
    println!("Tracked 'numbers_vec'");

    let text_string = String::from("Hello, Trace Tools!");
    track_var!(text_string); // Track the String
    println!("Tracked 'text_string'");

    let boxed_value = Box::new(100i32);
    track_var!(boxed_value); // Track the Box
    println!("Tracked 'boxed_value'");

    let boxed_value2 = Box::new(200i32);
    track_var!(boxed_value2);
    println!("Tracked 'boxed_value2'");

    // 3. Allocate and track reference-counted types.
    let rc_data = Rc::new(vec![10, 20, 30]);
    track_var!(rc_data); // Track the Rc (points to the Vec's data)
    println!("Tracked 'rc_data'");

    let arc_data = Arc::new(String::from("Shared data"));
    track_var!(arc_data); // Track the Arc (points to the String's data)
    println!("Tracked 'arc_data'");

    // Create another Rc to show shared ownership (both point to same allocation)
    let rc_data_clone = Rc::clone(&rc_data);
    // Note: track_var! on rc_data_clone will try to associate "rc_data_clone"
    // with the same underlying pointer as rc_data. The tracker will update the var_name
    // if called multiple times for the same pointer, or you might implement more complex logic.
    // For now, the last call to track_var for a given pointer "wins" the name.
    // Or, if the pointer is already known, associate_var might not update it.
    // Let's track it to see the behavior (current associate_var updates var_name).
    track_var!(rc_data_clone);
    println!("Tracked 'rc_data_clone' (shares allocation with 'rc_data')");

    // 4. Perform some operations (optional, just to make the program do something)
    let sum_of_vec = numbers_vec.iter().sum::<i32>();
    println!(
        "
Sum of 'numbers_vec': {}",
        sum_of_vec
    );
    println!("Length of 'text_string': {}", text_string.len());
    println!("Value in 'boxed_value': {}", *boxed_value);
    println!("Value in 'boxed_value2': {}", *boxed_value2);
    println!("First element of 'rc_data': {}", rc_data[0]);
    println!("Content of 'arc_data': {}", *arc_data);

    // 5. Get a reference to the global tracker to export data.
    let tracker = get_global_tracker();

    // 6. Export memory snapshot to JSON.
    println!(
        "
Exporting memory snapshot to basic_usage_snapshot.json..."
    );
    if let Err(e) = tracker.export_to_json("basic_usage_snapshot.json", true) { // Enable sync for reliable file writing
        eprintln!("Failed to export JSON: {}", e);
    } else {
        println!("Successfully exported JSON.");
    }

    // 7. Export memory usage visualization to SVG.
    println!(
        "
Exporting memory usage visualization to basic_usage_graph.svg..."
    );
    if let Err(e) = tracker.export_to_svg("basic_usage_graph.svg", true) { // Enable sync for reliable file writing
        eprintln!("Failed to export SVG: {}", e);
    } else {
        println!("Successfully exported SVG.");
    }

    println!(
        "
Example finished. Check 'basic_usage_snapshot.json' and 'basic_usage_graph.svg'."
    );
    println!("Observe the SVG to see the lifecycles of the tracked variables.");

    // Variables go out of scope here, and their memory will be deallocated.
    // The TrackingAllocator will record these deallocations.
    // The exported SVG should reflect their entire lifecycle.
}
