# Rust Memory Execution Visualizer (trace_tools)

## Overview
`trace_tools` is a Rust utility designed to help developers understand memory allocation patterns and variable lifecycles within their applications. It provides a custom global allocator (optional, feature-gated) to track heap memory operations and offers mechanisms to associate these operations with source code variables. The collected data can be exported into JSON format for detailed analysis or as an SVG image for a visual timeline of memory allocation lifecycles.

## Features
*   **Memory Allocation Tracking:** Uses an optional custom global allocator (`TrackingAllocator`) to monitor `alloc` and `dealloc` operations when the `tracking-allocator` feature is enabled (default).
*   **Variable Association:** Allows associating specific variable names with their memory allocations using the `track_var!` macro.
*   **Data Export:**
    *   Exports detailed memory snapshots (active allocations) to JSON.
    *   Generates SVG visualizations of memory allocation lifecycles, showing when each tracked allocation was active.
*   **Backtraces:** Captures backtraces for allocations to help pinpoint their origin (requires `backtrace` feature).
*   **Thread Information:** Records thread IDs for allocations.
*   **Easy Initialization:** Simple setup using `trace_tools::init()`.

## Installation
Add this to your `Cargo.toml`:

```toml
[dependencies]
trace_tools = "0.1.0" # Or specify git/path: { git = "..." } or { path = "..." }
```

By default, the `tracking-allocator` feature is enabled. If you wish to manage features explicitly:
```toml
[dependencies]
trace_tools = { version = "0.1.0", default-features = false, features = ["your_desired_features"] }
```
The `tracking-allocator` feature enables the custom global allocator. Without it, `track_var!` can still be used to associate names with pointers, but the allocations themselves must be reported to the `MemoryTracker` manually (e.g., by a different allocator or custom hooks).

## Usage

### 1. Initialize the Tracker
In your `main.rs` or early in your application startup:
```rust
fn main() {
    trace_tools::init(); // Initializes tracing and the memory tracker setup.
                         // If `tracking-allocator` feature is active, this also sets up the global allocator.
    // Your application code
}
```

### 2. Ensure the Tracking Allocator is Active (if used)
If you are relying on the built-in global allocator for tracking, ensure the `tracking-allocator` feature is enabled (it is by default). The allocator is set up in `src/lib.rs`:
```rust
// In src/lib.rs (already present in trace_tools)
#[cfg(feature = "tracking-allocator")]
#[global_allocator]
pub static GLOBAL: TrackingAllocator = TrackingAllocator;
```

### 3. Track Variables
Use the `track_var!` macro to associate a variable name with its underlying heap allocation. This should be called after the variable is initialized.
The `track_var!` macro works with types that implement the `Trackable` trait. Common types like `Box<T>`, `Vec<T>`, `String`, `Rc<T>`, and `Arc<T>` have `Trackable` implementations provided by `trace_tools`. Users may need to `use trace_tools::Trackable;` if they are implementing the trait for their own custom types or calling `Trackable` methods directly.

```rust
// main.rs or your relevant module
use trace_tools::{init, track_var, get_global_tracker};
// Use this if you need to call Trackable trait methods directly (e.g. for manual simulation)
// use trace_tools::Trackable; 

fn main() {
    init();

    let my_data = vec![1, 2, 3, 4, 5];
    // If using the `tracking-allocator` feature, the allocation is automatically reported.
    // If not, or for testing purposes where the allocator might not be fully active,
    // you would need to ensure the allocation is known to the tracker:
    //
    // if let Some(ptr) = my_data.get_trackable_raw_ptr() { // Trackable must be in scope for this method
    //     get_global_tracker().track_allocation(
    //         ptr,
    //         std::mem::size_of_val(my_data.as_slice()),
    //         Some(my_data.get_type_name()) // get_type_name also from Trackable
    //     ).expect("Simulated allocation tracking failed");
    // }
    track_var!(my_data); // Associates "my_data" with the Vec's allocation

    let my_string = String::from("hello world");
    // Similar simulation for my_string if needed:
    // if let Some(ptr) = my_string.get_trackable_raw_ptr() {
    //     get_global_tracker().track_allocation(ptr, my_string.capacity(), Some(my_string.get_type_name())).unwrap();
    // }
    track_var!(my_string);
    
    // ... other variables ...

    // Example cleanup for tests/non-allocator scenarios (not typically needed in apps)
    // if let Some(ptr) = my_data.get_trackable_raw_ptr() { let _ = get_global_tracker().track_deallocation(ptr); }
    // if let Some(ptr) = my_string.get_trackable_raw_ptr() { let _ = get_global_tracker().track_deallocation(ptr); }
}
```
**Note on Complex Collections:** For types like `HashMap` or `HashSet`, `track_var!` applied to the collection variable will associate the name with the main structure of the collection (if it's directly heap-allocated and `Trackable`). However, the internal buffers or individual elements within these collections are managed by the collection's own logic. While the `TrackingAllocator` (if enabled) will see these internal allocations, `track_var!` on the collection variable itself doesn't individually name these internal heap segments.

### 4. Exporting Data
You can export the collected memory data at any point, typically before your application exits:

```rust
use trace_tools::get_global_tracker;

fn export_data() { // Encapsulate in a function for clarity
    let tracker = get_global_tracker();

    if let Err(e) = tracker.export_to_json("memory_snapshot.json") {
        eprintln!("Failed to export JSON: {}", e);
    }

    if let Err(e) = tracker.export_to_svg("memory_usage.svg") {
        eprintln!("Failed to export SVG: {}", e);
    }
}
```

## Interpreting Output

*   **`memory_snapshot.json`**: Contains a detailed list of *active* allocations at the time of export. Each entry includes pointer address, size, allocation timestamp, associated variable name (if tracked by `track_var!`), type name, and backtrace (if enabled).
*   **`memory_usage.svg`**: Provides a timeline visualization of memory allocations. Each bar represents an allocation recorded in the *deallocation log*, showing its start time (allocation) and end time (deallocation). This helps in understanding the lifecycle of variables and identifying allocations that persist longer than expected.

## Examples
See the `examples/` directory for runnable code. The `basic_usage.rs` example demonstrates core features:

```rust
// From examples/basic_usage.rs
use std::rc::Rc;
use std::sync::Arc;
use trace_tools::{get_global_tracker, init, track_var};
// For direct calls to Trackable methods like get_trackable_raw_ptr in example setup:
// use trace_tools::Trackable; 

fn main() {
    init(); // Initialize trace_tools

    let numbers_vec = vec![1, 2, 3, 4, 5];
    // In a real app with the `tracking-allocator` feature, track_allocation is automatic.
    // The examples often simulate this for environments where the allocator isn't fully hooked.
    // For example:
    // if cfg!(not(feature = "tracking-allocator")) { // Or for specific test setups
    //     if let Some(ptr) = numbers_vec.get_trackable_raw_ptr() { // Trackable needs to be in scope
    //         use trace_tools::Trackable; 
    //         get_global_tracker().track_allocation(ptr, std::mem::size_of_val(numbers_vec.as_slice()), Some(numbers_vec.get_type_name())).unwrap();
    //     }
    // }
    track_var!(numbers_vec);

    let text_string = String::from("Hello, Trace Tools!");
    // Similar manual tracking simulation if needed for text_string
    track_var!(text_string);

    // ... other tracked variables like Box, Rc, Arc ...

    let tracker = get_global_tracker();
    tracker.export_to_json("basic_usage_snapshot.json").expect("JSON export failed");
    tracker.export_to_svg("basic_usage_graph.svg").expect("SVG export failed");

    println!("Example finished. Check basic_usage_snapshot.json and basic_usage_graph.svg.");
}
```

## Building the Project
```bash
cargo build
```

## Running Examples
```bash
cargo run --example basic_usage
cargo run --example lifecycles
```

## Contributing
Contributions are welcome! Please feel free to submit pull requests or open issues for bugs, feature requests, or improvements.

## License
This project is licensed under the MIT License (as per `Cargo.toml`).
```