# memscope-rs API Usage Guide

## Quick Start

memscope-rs provides a simple API to track memory usage in Rust applications.

### Basic Usage

```rust
use memscope_rs::{global_tracker, init_global_tracking, track};

// Initialize (optional, auto-initializes on first use)
init_global_tracking().unwrap();

// Get the global tracker
let tracker = global_tracker().unwrap();

// Track variables
let data = vec![1, 2, 3, 4, 5];
track!(tracker, data);

let text = String::from("Hello, world!");
track!(tracker, text);

// Export data
tracker.export_json("output").unwrap();
tracker.export_html("output").unwrap();
```

## Core API

### 1. Initialization

```rust
use memscope_rs::init_global_tracking;

// Initialize with default config
init_global_tracking().unwrap();

// Initialize with custom config
use memscope_rs::GlobalTrackerConfig;
let config = GlobalTrackerConfig {
    tracker: memscope_rs::TrackerConfig {
        max_allocations: 1000000,
        enable_statistics: true,
    },
    ..Default::default()
};
init_global_tracking_with_config(config).unwrap();
```

### 2. Get Tracker

```rust
use memscope_rs::global_tracker;

// Get the global tracker instance
let tracker = global_tracker().unwrap();
```

### 3. Track Variables

#### Simple Tracking

```rust
use memscope_rs::track;

// Auto-track with track! macro
let data = vec![1, 2, 3];
track!(tracker, data);

let text = String::from("Hello");
track!(tracker, text);
```

#### Named Tracking

```rust
use memscope_rs::track;

// Track with variable name and location
let important_data = vec![1, 2, 3];
track!(tracker, important_data, "important_data", "my_file.rs", 42);
```

#### Direct Methods

```rust
// Use tracker.track() method
let data = vec![1, 2, 3];
tracker.track(&data);

// Use tracker.track_as() to specify details
let data = vec![1, 2, 3];
tracker.track_as(&data, "my_data", "my_file.rs", 42);
```

### 4. Export Data

#### Export JSON

```rust
// Export all JSON files to specified directory
tracker.export_json("output")?;

// Generates the following files:
// - memory_snapshots.json        - Memory snapshot data
// - memory_passports.json        - Memory passport tracking
// - leak_detection.json          - Memory leak detection results
// - unsafe_ffi_analysis.json    - Unsafe/FFI tracking data
// - system_resources.json       - System resource monitoring
// - async_analysis.json         - Async task memory analysis
```

#### Export HTML

```rust
// Export HTML dashboard
tracker.export_html("output")?;

// Generates an interactive HTML dashboard
```

### 5. Get Statistics

```rust
// Get tracking statistics
let stats = tracker.get_stats();

println!("Total allocations: {}", stats.total_allocations);
println!("Active allocations: {}", stats.active_allocations);
println!("Peak memory: {} bytes", stats.peak_memory_bytes);
println!("Current memory: {} bytes", stats.current_memory_bytes);
println!("Memory passports: {}", stats.passport_count);
println!("Active passports: {}", stats.active_passports);
println!("Leaks detected: {}", stats.leaks_detected);
println!("Async tasks: {}", stats.async_task_count);
println!("Active async tasks: {}", stats.active_async_tasks);
println!("Uptime: {:?}", stats.uptime);
```

## Advanced Usage

### Unsafe Code Tracking

```rust
use memscope_rs::global_tracker;

let tracker = global_tracker().unwrap();

// Track unsafe allocations
use std::alloc::{alloc, Layout};
unsafe {
    let layout = Layout::new::<[u32; 100]>();
    let ptr = alloc(layout);

    if !ptr.is_null() {
        // Create memory passport (returns passport ID)
        let passport_id = tracker
            .create_passport(ptr as usize, layout.size(), "unsafe_alloc".to_string())
            .expect("Failed to create passport");
        println!("Created passport: {}", passport_id);

        // Record cross-boundary event
        tracker.record_handover(ptr as usize, "ffi_context".to_string(), "malloc".to_string());

        // Use memory...
        std::ptr::write_bytes(ptr as *mut u8, 0, layout.size());

        // Record free event
        tracker.record_free(ptr as usize, "ffi_context".to_string(), "free".to_string());

        std::alloc::dealloc(ptr, layout);
    }
}
```

### Leak Detection

```rust
use memscope_rs::global_tracker;

let tracker = global_tracker().unwrap();

// Execute leak detection (returns LeakDetectionResult)
let leak_result = tracker.passport_tracker().detect_leaks_at_shutdown();

println!("Total leaks detected: {}", leak_result.total_leaks);
println!("Active passports: {}", leak_result.active_passports);
println!("Total passports: {}", leak_result.total_passports);
```

### Access Internal Trackers

For lower-level tracking functionality:

```rust
use memscope_rs::global_tracker;

let tracker = global_tracker().unwrap();

// Access base tracker
let base_tracker = tracker.tracker();
let report = base_tracker.analyze();

// Access passport tracker
let passport_tracker = tracker.passport_tracker();
let passport_stats = passport_tracker.get_stats();

// Access async tracker
let async_tracker = tracker.async_tracker();
let async_stats = async_tracker.get_stats();
```

## Supported Types

The following types are automatically supported for tracking:

- `Vec<T>`
- `String`
- `Box<T>`
- `Rc<T>`
- `Arc<T>`
- `HashMap<K, V>`
- `BTreeMap<K, V>`
- `VecDeque<T>`
- `RefCell<T>`
- `RwLock<T>`

## Complete Example

```rust
use memscope_rs::{global_tracker, init_global_tracking, track};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize
    init_global_tracking()?;

    // Get tracker
    let tracker = global_tracker()?;

    // Track various types
    let vec_data = vec![1, 2, 3, 4, 5];
    track!(tracker, vec_data);

    let string_data = String::from("Hello, memscope!");
    track!(tracker, string_data);

    let boxed_data = Box::new(42);
    track!(tracker, boxed_data);

    // Get statistics
    let stats = tracker.get_stats();
    println!("Tracking stats: {:?}", stats);

    // Export data
    tracker.export_json("output")?;
    tracker.export_html("output")?;

    println!("Data exported to output/ directory");

    Ok(())
}
```

## Notes

1. **Thread Safety**: The global tracker is thread-safe and can be used in multi-threaded environments
2. **Performance Impact**: Tracking introduces some overhead, recommended for development and debugging only
3. **Memory Overhead**: Tracking itself consumes memory, monitor memory usage
4. **Initialization**: While auto-initialization works, recommended to explicitly call `init_global_tracking()` at program start

## Troubleshooting

### Error: AlreadyInitialized

This error occurs when `init_global_tracking()` is called more than once. The global tracker is designed to be initialized only once. To reset for testing:

```rust
#[cfg(test)]
use memscope_rs::reset_global_tracking;

#[test]
fn test_tracking() {
    reset_global_tracking(); // Reset before test
    init_global_tracking().unwrap();
    // ...
}
```

### Error: NotInitialized

This error occurs when trying to access `global_tracker()` before initialization. Either call `init_global_tracking()` first, or let it auto-initialize on first use.

### Performance Issues

If tracking causes significant performance degradation:

1. Reduce `max_allocations` in config
2. Disable statistics collection with `enable_statistics: false`
3. Use selective tracking instead of global tracking
