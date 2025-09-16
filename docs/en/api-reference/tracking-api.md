# Tracking API Reference

Functions and methods for memory tracking in memscope-rs.

## Overview

This document describes the tracking API that provides core functionality for monitoring memory allocations, managing the tracking system, and retrieving statistics.

## Global Functions

### get_global_tracker

```rust
pub fn get_global_tracker() -> Arc<MemoryTracker>
```

Get the global memory tracker instance.

**Returns:** `Arc<MemoryTracker>` - Shared reference to the global tracker

```rust
use memscope_rs::get_global_tracker;

let tracker = get_global_tracker();
match tracker.get_stats() {
    Ok(stats) => println!("Active allocations: {}", stats.active_allocations),
    Err(e) => eprintln!("Failed to get stats: {}", e),
}
```

### init

```rust
pub fn init()
```

Initialize the memory tracking system. Must be called once at program start.

```rust
use memscope_rs::init;

fn main() {
    init(); // Initialize tracking system
    
    let my_vec = vec![1, 2, 3, 4, 5];
    memscope_rs::track_var!(my_vec);
}
```

## MemoryTracker Methods

### get_stats

```rust
pub fn get_stats(&self) -> TrackingResult<MemoryStats>
```

Get current memory statistics with analysis.

**Returns:** `TrackingResult<MemoryStats>` - Current memory statistics

```rust
let tracker = get_global_tracker();
match tracker.get_stats() {
    Ok(stats) => {
        println!("Total allocations: {}", stats.total_allocations);
        println!("Peak memory: {} bytes", stats.peak_memory);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### get_active_allocations

```rust
pub fn get_active_allocations(&self) -> TrackingResult<Vec<AllocationInfo>>
```

Get all currently active allocations.

**Returns:** `TrackingResult<Vec<AllocationInfo>>` - List of active allocations

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

**Parameters:**
- `ptr`: `usize` - Memory address
- `size`: `usize` - Size in bytes
- `var_name`: `Option<String>` - Variable name
- `type_name`: `Option<String>` - Type name

## Export Functions

### export_to_json

```rust
pub fn export_to_json(&self, base_name: &str) -> TrackingResult<()>
```

Export memory analysis to JSON format (5 categorized files).

```rust
let tracker = get_global_tracker();
tracker.export_to_json("my_analysis")?;
// Generates: MemoryAnalysis/my_analysis/*_*.json
```

### export_to_binary

```rust
pub fn export_to_binary(&self, base_name: &str) -> TrackingResult<()>
```

Export to high-performance binary format.

```rust
tracker.export_to_binary("fast_export")?;
// Generates: MemoryAnalysis/fast_export/fast_export.memscope
```

### export_memory_analysis

```rust
pub fn export_memory_analysis(&self, filename: &str) -> TrackingResult<()>
```

Export memory visualization to SVG.

```rust
tracker.export_memory_analysis("memory_chart.svg")?;
```

## Tracking Macros

### track_var!

```rust
track_var!(variable)
```

**[RECOMMENDED]** Zero-cost tracking without ownership changes.

```rust
let my_vec = vec![1, 2, 3, 4, 5];
track_var!(my_vec);
// my_vec can still be used normally
```

### track_var_owned!

```rust
track_var_owned!(variable)
```

**[ADVANCED]** Precise lifecycle tracking with ownership transfer.

```rust
let my_vec = vec![1, 2, 3, 4, 5];
let tracked = track_var_owned!(my_vec);
// tracked provides lifecycle management
```

### track_var_smart!

```rust
track_var_smart!(variable)
```

**[SMART]** Automatic optimization based on type.

```rust
let number = track_var_smart!(42i32);     // Copy type
let vec = track_var_smart!(vec![1,2,3]);  // Reference tracking
```

## Configuration Types

### ExportOptions

```rust
pub struct ExportOptions {
    pub include_system_allocations: bool,
    pub verbose_logging: bool,
    pub buffer_size: usize,
    // ... more options
}
```

```rust
let options = ExportOptions::new()
    .include_system_allocations(false)
    .verbose_logging(true)
    .buffer_size(128 * 1024);

tracker.export_to_json_with_options("analysis", options)?;
```

### BinaryExportMode

```rust
pub enum BinaryExportMode {
    UserOnly,  // Only user variables (faster)
    Full,      // All allocations (complete)
}
```

## Complete Example

```rust
use memscope_rs::{init, track_var, get_global_tracker, ExportOptions};
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize
    init();
    
    // Track variables
    let numbers = vec![1, 2, 3, 4, 5];
    track_var!(numbers);
    
    let shared = Rc::new(vec!["a", "b", "c"]);
    track_var!(shared);
    
    // Get statistics
    let tracker = get_global_tracker();
    let stats = tracker.get_stats()?;
    println!("Active memory: {} bytes", stats.active_memory);
    
    // Export results
    tracker.export_to_json("complete_example")?;
    tracker.export_to_binary("complete_example")?;
    tracker.export_memory_analysis("visualization.svg")?;
    
    Ok(())
}
```

## See Also

- [Core Types Reference](core-types.md) - Data structures and types
- [Analysis API Reference](analysis-api.md) - Memory analysis functions
- [Export API Reference](export-api.md) - Data export functionality