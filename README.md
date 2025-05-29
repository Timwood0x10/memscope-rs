# Rust Memory Execution Visualizer (trace_tools)

## Overview
`trace_tools` is a Rust utility designed to help developers understand memory allocation patterns and variable lifecycles within their applications. It provides a custom global allocator to track heap memory operations and offers mechanisms to associate these operations with source code variables. The collected data can be exported into JSON format for detailed analysis or as an SVG image for a visual overview of memory usage.

## Features
*   **Memory Allocation Tracking:** Uses a custom global allocator (`TrackingAllocator`) to monitor `alloc` and `dealloc` operations.
*   **Variable Association:** Allows associating specific variable names with their memory allocations using the `track_var!` macro.
*   **Data Export:**
    *   Exports detailed memory snapshots to JSON.
    *   Generates SVG visualizations of memory usage (currently shows active allocations).
*   **Backtraces:** Captures backtraces for allocations to help pinpoint their origin.
*   **Thread Information:** Records thread IDs for allocations.
*   **Easy Initialization:** Simple setup using `trace_tools::init()`.

## Installation
Add this to your `Cargo.toml`:

```toml
[dependencies]
trace_tools = { version = "0.1.0", path = "..." } # Or from crates.io if published
```

And enable the tracking allocator feature (enabled by default):
If you want to explicitly control it, you might use:
```toml
trace_tools = { version = "0.1.0", path = "...", features = ["tracking-allocator"] }
```
Ensure the `tracking-allocator` feature is enabled (it is by default) to use the custom global allocator.

## Usage

### 1. Initialize the Tracker
In your `main.rs` or early in your application startup:
```rust
fn main() {
    trace_tools::init(); // Initializes tracing and the memory tracker setup

    // Your application code
}
```

### 2. Enable the Tracking Allocator
The `TrackingAllocator` is set as the `#[global_allocator]` in `src/lib.rs` and enabled by the `tracking-allocator` feature (which is part of the default features). Make sure this feature is active.

```rust
// In src/lib.rs (already present)
#[cfg(feature = "tracking-allocator")]
#[global_allocator]
pub static GLOBAL: TrackingAllocator = TrackingAllocator;
```

### 3. Track Variables
Use the `track_var!` macro to associate a variable name with its underlying heap allocation. This should be called after the variable is initialized.

```rust
use trace_tools::track_var;

let my_data = vec![1, 2, 3, 4, 5];
track_var!(my_data); // Associates the name "my_data" with the Vec's allocation

let my_string = String::from("hello world");
track_var!(my_string);
```
The `track_var!` macro works with types that implement the `Trackable` trait (e.g., `Box<T>`, `Vec<T>`, `String`).

### 4. Exporting Data
You can export the collected memory data at any point:

```rust
use trace_tools::get_global_tracker;

// ... later in your code, perhaps before shutdown ...
let tracker = get_global_tracker();

if let Err(e) = tracker.export_to_json("memory_snapshot.json") {
    eprintln!("Failed to export JSON: {}", e);
}

if let Err(e) = tracker.export_to_svg("memory_usage.svg") {
    eprintln!("Failed to export SVG: {}", e);
}
```

## Interpreting Output

*   **`memory_snapshot.json`**: Contains a detailed list of all allocations (both active and those that have been deallocated by the time of export, if the log includes them - currently focuses on active for snapshot). Each entry includes pointer address, size, allocation/deallocation timestamps, associated variable name (if tracked), type name, and backtrace.
*   **`memory_usage.svg`**: Provides a visual representation of memory usage. The current version displays a bar chart of the top N largest active allocations. (This will be enhanced to show lifecycles).

## Examples
See the `examples/` directory for runnable code. Here's a quick look:

```rust
// examples/basic_usage.rs (will be updated to reflect proper usage)
use trace_tools::{init, track_var, get_global_tracker};

fn main() {
    init();

    let numbers = vec![0u8; 1024];
    track_var!(numbers);

    let text = String::from("This is a test string.");
    track_var!(text);

    // Do more work...

    let tracker = get_global_tracker();
    tracker.export_to_json("example_snapshot.json").expect("JSON export failed");
    tracker.export_to_svg("example_usage.svg").expect("SVG export failed");

    println!("Example finished. Check example_snapshot.json and example_usage.svg.");
}
```
*(Note: The example snippet above is illustrative and might need adjustments based on actual example code improvements planned).*

## Building the Project
```bash
cargo build
```

## Running Examples
```bash
cargo run --example basic_usage 
```
*(Assuming `basic_usage.rs` is updated as per the plan)*

## Contributing
Contributions are welcome! Please feel free to submit pull requests or open issues for bugs, feature requests, or improvements.
(Further details can be added here, like coding style, testing requirements etc.)

## License
This project is licensed under the MIT License (Please verify from `Cargo.toml` or add a `LICENSE` file if not present. Assuming MIT based on common Rust practice).

Copyright (c) 2025 TimWood

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

[http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0)