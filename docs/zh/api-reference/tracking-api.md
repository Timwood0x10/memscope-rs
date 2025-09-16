# Tracking API Reference

Functions and methods for memory tracking in memscope-rs.

## Overview

This document describes the tracking API that provides the core functionality for monitoring memory allocations, managing the tracking system, and retrieving statistics. The API is designed to be simple to use while providing comprehensive tracking capabilities.

## Global Functions

### get_global_tracker

```rust
pub fn get_global_tracker() -> Arc<MemoryTracker>
```

Get the global memory tracker instance.

**Module:** `memscope_rs::core::tracker`

**Source:** `src/core/tracker/memory_tracker.rs`

Returns a reference to the singleton memory tracker that is used throughout the application. This is the primary entry point for accessing tracking functionality.

**Returns:** `Arc<MemoryTracker>` - Shared reference to the global tracker

#### Example Usage

```rust
use memscope_rs::get_global_tracker;

fn main() {
    let tracker = get_global_tracker();
    
    // Get current statistics
    match tracker.get_stats() {
        Ok(stats) => {
            println!("Active allocations: {}", stats.active_allocations);
            println!("Active memory: {} bytes", stats.active_memory);
        }
        Err(e) => eprintln!("Failed to get stats: {}", e),
    }
}
```

### init

```rust
pub fn init()
```

Initialize the memory tracking system.

**Module:** `memscope_rs`

**Source:** `src/lib.rs`

Initializes the global memory tracker and sets up the tracking infrastructure. This function should be called once at the beginning of your program before using any tracking functionality.

#### Example Usage

```rust
use memscope_rs::init;

fn main() {
    // Initialize tracking system
    init();
    
    // Now you can use tracking macros
    let my_vec = vec![1, 2, 3, 4, 5];
    memscope_rs::track_var!(my_vec);
}
```

## MemoryTracker Methods

### new

```rust
pub fn new() -> Self
```

Create a new memory tracker instance.

**Module:** `memscope_rs::MemoryTracker`

**Source:** `src/core/tracker/memory_tracker.rs`

Creates a new memory tracker with empty allocation records and default statistics. Automatically detects test mode for reduced overhead.

**Returns:** `MemoryTracker`

### get_stats

```rust
pub fn get_stats(&self) -> TrackingResult<MemoryStats>
```

Get current memory statistics with advanced analysis.

**Module:** `memscope_rs::MemoryTracker`

**Source:** `src/core/tracker/memory_tracker.rs`

Retrieves comprehensive memory statistics including allocation counts, memory usage, peak values, and analysis results.

**Returns:** `TrackingResult<MemoryStats>` - Current memory statistics

#### Example Usage

```rust
use memscope_rs::get_global_tracker;

let tracker = get_global_tracker();
match tracker.get_stats() {
    Ok(stats) => {
        println!("Total allocations: {}", stats.total_allocations);
        println!("Peak memory: {} bytes", stats.peak_memory);
        println!("Leaked allocations: {}", stats.leaked_allocations);
    }
    Err(e) => eprintln!("Error getting stats: {}", e),
}
```

### get_active_allocations

```rust
pub fn get_active_allocations(&self) -> TrackingResult<Vec<AllocationInfo>>
```

Get all currently active allocations.

**Module:** `memscope_rs::MemoryTracker`

**Source:** `src/core/tracker/memory_tracker.rs`

Returns a list of all allocations that are currently active (not yet deallocated).

**Returns:** `TrackingResult<Vec<AllocationInfo>>` - List of active allocations

#### Example Usage

```rust
use memscope_rs::get_global_tracker;

let tracker = get_global_tracker();
match tracker.get_active_allocations() {
    Ok(allocations) => {
        println!("Found {} active allocations", allocations.len());
        for alloc in allocations {
            if let Some(var_name) = &alloc.var_name {
                println!("  {}: {} bytes", var_name, alloc.size);
            }
        }
    }
    Err(e) => eprintln!("Error getting allocations: {}", e),
}
```

### track_allocation

```rust
pub fn track_allocation(
    &self,
    ptr: usize,
    size: usize,
    var_name: Option<String>,
    type_name: Option<String>,
) -> TrackingResult<()>
```

Track a new memory allocation.

**Module:** `memscope_rs::MemoryTracker`

**Source:** `src/core/tracker/allocation_tracking.rs`

Records a new memory allocation with the specified parameters. This is typically called automatically by the tracking macros.

**Parameters:**
- `ptr`: `usize` - Memory address of the allocation
- `size`: `usize` - Size of the allocation in bytes
- `var_name`: `Option<String>` - Optional variable name
- `type_name`: `Option<String>` - Optional type name

**Returns:** `TrackingResult<()>`

### track_deallocation

```rust
pub fn track_deallocation(&self, ptr: usize) -> TrackingResult<()>
```

Track a memory deallocation.

**Module:** `memscope_rs::MemoryTracker`

**Source:** `src/core/tracker/allocation_tracking.rs`

Records the deallocation of a previously tracked memory allocation.

**Parameters:**
- `ptr`: `usize` - Memory address being deallocated

**Returns:** `TrackingResult<()>`

### track_smart_pointer_clone

```rust
pub fn track_smart_pointer_clone(
    &self,
    clone_ptr: usize,
    source_ptr: usize,
    data_ptr: usize,
    new_ref_count: usize,
    weak_count: usize,
) -> TrackingResult<()>
```

Track smart pointer clone operations.

**Module:** `memscope_rs::MemoryTracker`

**Source:** `src/core/tracker/allocation_tracking.rs`

Records when a smart pointer (Rc, Arc) is cloned, tracking the relationship between the original and clone.

**Parameters:**
- `clone_ptr`: `usize` - Address of the new clone
- `source_ptr`: `usize` - Address of the source pointer
- `data_ptr`: `usize` - Address of the shared data
- `new_ref_count`: `usize` - New reference count after cloning
- `weak_count`: `usize` - Current weak reference count

**Returns:** `TrackingResult<()>`

### update_smart_pointer_ref_count

```rust
pub fn update_smart_pointer_ref_count(
    &self,
    ptr: usize,
    strong_count: usize,
    weak_count: usize,
) -> TrackingResult<()>
```

Update smart pointer reference counts.

**Module:** `memscope_rs::MemoryTracker`

**Source:** `src/core/tracker/allocation_tracking.rs`

Updates the reference count information for a smart pointer allocation.

**Parameters:**
- `ptr`: `usize` - Address of the smart pointer
- `strong_count`: `usize` - Current strong reference count
- `weak_count`: `usize` - Current weak reference count

**Returns:** `TrackingResult<()>`

## Export Functions

### export_to_json

```rust
pub fn export_to_json(&self, base_name: &str) -> TrackingResult<()>
```

Export memory analysis to JSON format.

**Module:** `memscope_rs::MemoryTracker`

**Source:** `src/core/tracker/export_json.rs`

Exports comprehensive memory analysis data to JSON files in the `MemoryAnalysis/` directory.

**Parameters:**
- `base_name`: `&str` - Base name for the output files

**Returns:** `TrackingResult<()>`

#### Example Usage

```rust
use memscope_rs::get_global_tracker;

let tracker = get_global_tracker();
if let Err(e) = tracker.export_to_json("my_analysis") {
    eprintln!("Export failed: {}", e);
} else {
    println!("Analysis exported to MemoryAnalysis/my_analysis/");
}
```

### export_to_json_with_options

```rust
pub fn export_to_json_with_options(
    &self,
    base_name: &str,
    options: ExportOptions,
) -> TrackingResult<()>
```

Export memory analysis to JSON with custom options.

**Module:** `memscope_rs::MemoryTracker`

**Source:** `src/core/tracker/export_json.rs`

Exports memory analysis data with customizable export options for fine-tuned control over the output.

**Parameters:**
- `base_name`: `&str` - Base name for the output files
- `options`: `ExportOptions` - Export configuration options

**Returns:** `TrackingResult<()>`

#### Example Usage

```rust
use memscope_rs::{get_global_tracker, ExportOptions};

let tracker = get_global_tracker();
let options = ExportOptions::new()
    .include_system_allocations(false)
    .verbose_logging(true)
    .buffer_size(128 * 1024);

if let Err(e) = tracker.export_to_json_with_options("detailed_analysis", options) {
    eprintln!("Export failed: {}", e);
}
```

### export_to_binary

```rust
pub fn export_to_binary(
    &self,
    base_name: &str,
    mode: BinaryExportMode,
) -> TrackingResult<()>
```

Export memory analysis to binary format.

**Module:** `memscope_rs::MemoryTracker`

**Source:** `src/export/binary/mod.rs`

Exports memory analysis data to efficient binary format for faster processing and smaller file sizes.

**Parameters:**
- `base_name`: `&str` - Base name for the output files
- `mode`: `BinaryExportMode` - Export mode (UserOnly or Full)

**Returns:** `TrackingResult<()>`

#### Example Usage

```rust
use memscope_rs::{get_global_tracker, BinaryExportMode};

let tracker = get_global_tracker();
if let Err(e) = tracker.export_to_binary("fast_export", BinaryExportMode::UserOnly) {
    eprintln!("Binary export failed: {}", e);
} else {
    println!("Binary data exported successfully");
}
```

### export_memory_analysis

```rust
pub fn export_memory_analysis(&self, filename: &str) -> TrackingResult<()>
```

Export memory usage visualization to SVG.

**Module:** `memscope_rs::MemoryTracker`

**Source:** `src/export/visualization.rs`

Generates SVG visualization of memory usage patterns and saves it to the specified file.

**Parameters:**
- `filename`: `&str` - Name of the output SVG file

**Returns:** `TrackingResult<()>`

#### Example Usage

```rust
use memscope_rs::get_global_tracker;

let tracker = get_global_tracker();
if let Err(e) = tracker.export_memory_analysis("memory_usage.svg") {
    eprintln!("SVG export failed: {}", e);
} else {
    println!("Memory visualization saved to memory_usage.svg");
}
```

## Configuration Types

### ExportOptions

Configuration options for data export.

**Module:** `memscope_rs::core::tracker::config`

**Source:** `src/core/tracker/config.rs`

#### Methods

##### new

```rust
pub fn new() -> Self
```

Create new export options with default settings.

##### include_system_allocations

```rust
pub fn include_system_allocations(mut self, include: bool) -> Self
```

Set whether to include system allocations in export.

##### verbose_logging

```rust
pub fn verbose_logging(mut self, verbose: bool) -> Self
```

Enable or disable verbose logging during export.

##### buffer_size

```rust
pub fn buffer_size(mut self, size: usize) -> Self
```

Set the buffer size for export operations.

#### Example Usage

```rust
use memscope_rs::ExportOptions;

let options = ExportOptions::new()
    .include_system_allocations(false)
    .verbose_logging(true)
    .buffer_size(64 * 1024);
```

### BinaryExportMode

Export mode for binary format.

**Module:** `memscope_rs::core::tracker::memory_tracker`

**Source:** `src/core/tracker/memory_tracker.rs`

#### Variants

- **UserOnly** - Export only user-defined variables (smaller files, faster processing)
- **Full** - Export all allocations including system allocations (complete data)

## Tracking Macros

### track_var!

```rust
track_var!(variable)
```

**[RECOMMENDED]** Track a variable's memory allocation without taking ownership.

**Module:** `memscope_rs`

**Source:** `src/lib.rs`

Zero-cost tracking macro that monitors memory usage without affecting variable ownership or performance.

#### Example Usage

```rust
use memscope_rs::track_var;

let my_vec = vec![1, 2, 3, 4, 5];
track_var!(my_vec);
// my_vec can still be used normally
println!("Vector: {:?}", my_vec);
```

### track_var_owned!

```rust
track_var_owned!(variable)
```

**[ADVANCED]** Track a variable with full lifecycle management and ownership transfer.

**Module:** `memscope_rs`

**Source:** `src/lib.rs`

Advanced tracking macro that takes ownership and provides precise lifecycle tracking with automatic cleanup detection.

#### Example Usage

```rust
use memscope_rs::track_var_owned;

let my_vec = vec![1, 2, 3, 4, 5];
let tracked_vec = track_var_owned!(my_vec);
// tracked_vec behaves like my_vec but with lifecycle tracking
println!("Length: {}", tracked_vec.len());
```

### track_var_smart!

```rust
track_var_smart!(variable)
```

**[SMART]** Intelligent tracking that automatically chooses the best strategy.

**Module:** `memscope_rs`

**Source:** `src/lib.rs`

Smart tracking macro that automatically detects variable type and chooses optimal tracking approach.

#### Example Usage

```rust
use memscope_rs::track_var_smart;

let number = 42i32;           // Copy type - will be copied
let my_vec = vec![1, 2, 3];   // Non-Copy - will be tracked by reference
let rc_data = std::rc::Rc::new(vec![]); // Smart pointer - will clone the Rc

track_var_smart!(number);
track_var_smart!(my_vec);
track_var_smart!(rc_data);
```

## Complete Example

```rust
use memscope_rs::{init, track_var, get_global_tracker, ExportOptions};
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracking system
    init();
    
    // Track various types of allocations
    let my_vec = vec![1, 2, 3, 4, 5];
    track_var!(my_vec);
    
    let my_string = String::from("Hello, memscope!");
    track_var!(my_string);
    
    let rc_data = Rc::new(vec![10, 20, 30]);
    track_var!(rc_data);
    
    let rc_clone = Rc::clone(&rc_data);
    track_var!(rc_clone);
    
    // Get tracker and statistics
    let tracker = get_global_tracker();
    let stats = tracker.get_stats()?;
    
    println!("Memory Statistics:");
    println!("  Active allocations: {}", stats.active_allocations);
    println!("  Active memory: {} bytes", stats.active_memory);
    println!("  Peak memory: {} bytes", stats.peak_memory);
    
    // Export analysis results
    tracker.export_to_json("complete_example")?;
    
    // Export with custom options
    let options = ExportOptions::new()
        .include_system_allocations(false)
        .verbose_logging(true);
    
    tracker.export_to_json_with_options("detailed_example", options)?;
    
    // Export visualization
    tracker.export_memory_analysis("example_visualization.svg")?;
    
    println!("Analysis complete! Check MemoryAnalysis/ directory for results.");
    
    Ok(())
}
```

## See Also

- [Core Types Reference](core-types.md) - Data structures and types
- [Analysis API Reference](analysis-api.md) - Memory analysis functions
- [Export API Reference](export-api.md) - Data export functionality
- [User Guide: Tracking Macros](../user-guide/tracking-macros.md) - Detailed macro usage guide