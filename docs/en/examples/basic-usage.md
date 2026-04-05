# Basic Usage Example

This document provides detailed usage instructions and best practices based on `examples/basic_usage.rs`.

## 🎯 Complete Example Analysis

### Basic Setup
```rust
use memscope_rs::{track_var};
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    // 1. Create MemScope instance
    let memscope = memscope_rs::MemScope::new();
    println!("memscope-rs initialized. Tracking memory allocations...");
```

**Key Points**:
- `memscope_rs::MemScope::new()` creates a new tracking instance
- Only needs to be called once, usually at the start of `main()` function
- After initialization, the tracking system is ready

### Basic Type Tracking
```rust
    // 2. Allocate and track simple types
    println!("\nAllocating and tracking variables...");

    let numbers_vec = vec![1, 2, 3, 4, 5];
    track_var!(numbers_vec);
    println!("Tracked 'numbers_vec'");

    let text_string = String::from("Hello, Trace Tools!");
    track_var!(text_string);
    println!("Tracked 'text_string'");

    let boxed_value = Box::new(100i32);
    track_var!(boxed_value);
    println!("Tracked 'boxed_value'");
```

**Explanation**:
- `Vec<T>` - Dynamic array, allocates data on heap
- `String` - Dynamic string, content stored on heap
- `Box<T>` - Smart pointer, allocates data to heap

**Memory Layout**:
```
Stack                   Heap
┌─────────────┐    ┌─────────────────┐
│ numbers_vec │───▶│ [1, 2, 3, 4, 5] │
├─────────────┤    ├─────────────────┤
│ text_string │───▶│ "Hello, Trace..." │
├─────────────┤    ├─────────────────┤
│ boxed_value │───▶│      100        │
└─────────────┘    └─────────────────┘
```

### Smart Pointer Tracking
```rust
    // 3. Track reference-counted types
    let rc_data = Rc::new(vec![10, 20, 30]);
    track_var!(rc_data);
    println!("Tracked 'rc_data'");

    let arc_data = Arc::new(String::from("Shared data"));
    track_var!(arc_data);
    println!("Tracked 'arc_data'");

    // Clone Rc to show shared ownership
    let rc_data_clone = Rc::clone(&rc_data);
    track_var!(rc_data_clone);
    println!("Tracked 'rc_data_clone' (shares allocation with 'rc_data')");
```

**Smart Pointer Features**:
- `Rc<T>` - Single-threaded reference-counted smart pointer
- `Arc<T>` - Thread-safe reference-counted smart pointer
- Cloning only increases reference count, doesn't copy data

**Reference Count Tracking**:
```
Initial state: rc_data (ref count: 1)
After clone:   rc_data (ref count: 2) ←─┐
              rc_data_clone ─────────────┘
              (sharing same heap memory)
```

### Normal Variable Usage
```rust
    // 4. Perform some operations (variables remain fully usable)
    let sum_of_vec = numbers_vec.iter().sum::<i32>();
    println!("\nSum of 'numbers_vec': {sum_of_vec}");
    println!("Length of 'text_string': {}", text_string.len());
    println!("Value in 'boxed_value': {}", *boxed_value);
    println!("First element of 'rc_data': {}", rc_data[0]);
    println!("Content of 'arc_data': {}", *arc_data);
```

**Important Features**:
- Variables work completely normally after tracking
- Zero performance overhead
- No ownership changes

### Get Memory Statistics
```rust
    // 5. Get memory statistics
    if let Ok(stats) = memscope.summary() {
        println!("\nMemory Statistics:");
        println!("  Active allocations: {}", stats.active_allocations);
        println!("  Active memory: {} bytes", stats.active_memory);
        println!("  Total allocations: {}", stats.total_allocations);
        println!("  Peak memory: {} bytes", stats.peak_memory);
    }
```

**Statistics Explanation**:
- `active_allocations` - Number of currently active allocations
- `active_memory` - Total amount of memory currently in use
- `total_allocations` - Total number of allocations during program execution
- `peak_memory` - Peak memory usage

### Export Analysis Results
```rust
    // 6. Export memory snapshot to JSON
    println!("\nExporting memory snapshot to MemoryAnalysis/basic_usage/...");
    if let Err(e) = memscope.export_json("basic_usage_snapshot") {
        eprintln!("Failed to export JSON: {e}");
    } else {
        println!("Successfully exported JSON to MemoryAnalysis/basic_usage/");
    }

    // 7. Export memory usage visualization to SVG
    println!("\nExporting memory usage visualization to MemoryAnalysis/basic_usage/...");
    if let Err(e) = memscope.export_svg("basic_usage_graph.svg") {
        eprintln!("Failed to export SVG: {e}");
    } else {
        println!("Successfully exported SVG to MemoryAnalysis/basic_usage/");
    }
}
```

## 🔍 Running Results Analysis

### Console Output Example
```
memscope-rs initialized. Tracking memory allocations...

Allocating and tracking variables...
Tracked 'numbers_vec'
Tracked 'text_string'
Tracked 'boxed_value'
Tracked 'rc_data'
Tracked 'arc_data'
Tracked 'rc_data_clone' (shares allocation with 'rc_data')

Sum of 'numbers_vec': 15
Length of 'text_string': 17
Value in 'boxed_value': 100
First element of 'rc_data': 10
Content of 'arc_data': Shared data

Memory Statistics:
  Active allocations: 6
  Active memory: 321 bytes
  Total allocations: 6
  Peak memory: 321 bytes

Exporting memory snapshot to MemoryAnalysis/basic_usage/...
Successfully exported JSON to MemoryAnalysis/basic_usage/

Exporting memory usage visualization to MemoryAnalysis/basic_usage/...
Successfully exported SVG to MemoryAnalysis/basic_usage/
```

## 📊 Understanding the Output

### Memory Breakdown

Based on the example output:

| Variable | Type | Approx. Size | Notes |
|----------|------|--------------|-------|
| `numbers_vec` | `Vec<i32>` | 40 bytes | 5 × 8 bytes |
| `text_string` | `String` | 17 bytes | Length |
| `boxed_value` | `Box<i32>` | 8 bytes | Single value |
| `rc_data` | `Rc<Vec<i32>>` | 24 bytes | 3 × 8 bytes |
| `arc_data` | `Arc<String>` | 10 bytes | Shared string |
| `rc_data_clone` | `Rc<Vec<i32>>` | 8 bytes | Ref count only |
| **Total** | | **~107 bytes** | Data + overhead |

### File Structure

After running the example:

```
MemoryAnalysis/
└── basic_usage_snapshot/
    ├── basic_usage_snapshot_memory_analysis.json
    ├── basic_usage_snapshot_lifetime.json
    ├── basic_usage_snapshot_performance.json
    ├── basic_usage_snapshot_unsafe_ffi.json
    ├── basic_usage_snapshot_complex_types.json
    └── basic_usage_graph.svg
```

## 🎯 Best Practices

### 1. Initialize at Program Start
```rust
// ✅ Good
fn main() {
    let memscope = memscope_rs::MemScope::new(); // First line
    // ... rest of program
}

// ❌ Avoid
fn main() {
    some_function();
    // ... later
    let memscope = memscope_rs::MemScope::new(); // Too late
}
```

### 2. Track Key Variables
```rust
// ✅ Track important allocations
let large_data = vec![0; 1024 * 1024]; // 1MB
track_var!(large_data);

// ✅ Track shared references
let shared = Arc::new(String::from("important"));
track_var!(shared);

// ❌ Don't track small stack variables
let x = 42; // No need to track
```

### 3. Export Before Program Exit
```rust
fn main() {
    let memscope = memscope_rs::MemScope::new();

    // ... your code ...

    // Export before exit
    memscope.export_json("final_analysis").unwrap();
}
```

## 🚀 Advanced Usage

### Conditional Tracking
```rust
#[cfg(debug_assertions)]
macro_rules! debug_track {
    ($var:expr) => { track_var!($var) };
}

#[cfg(not(debug_assertions))]
macro_rules! debug_track {
    ($var:expr) => {};
}

fn main() {
    let memscope = memscope_rs::MemScope::new();

    let data = vec![1, 2, 3];
    debug_track!(data); // Only tracks in debug mode
}
```

### Function-Level Tracking
```rust
fn process_data(input: Vec<i32>) -> Vec<i32> {
    track_var!(input); // Track input

    let mut result = Vec::with_capacity(input.len());
    track_var!(result); // Track output buffer

    for item in input {
        result.push(item * 2);
    }

    result
}
```

## 💡 Common Patterns

### Pattern 1: Resource Tracking
```rust
fn main() {
    let memscope = memscope_rs::MemScope::new();

    // Track resources
    let buffer = vec![0u8; 4096];
    track_var!(buffer);

    let cache = std::collections::HashMap::new();
    track_var!(cache);

    // Use resources
    // ...

    memscope.export_json("resource_analysis").unwrap();
}
```

### Pattern 2: Performance Profiling
```rust
fn main() {
    let memscope = memscope_rs::MemScope::new();

    let start = std::time::Instant::now();

    // Run workload
    for i in 0..1000 {
        let data = vec![i; 100];
        track_var!(data);
    }

    let duration = start.elapsed();
    println!("Completed in {:?}", duration);

    memscope.export_json("performance_profile").unwrap();
}
```

## 🔧 Troubleshooting

### Issue: No Output Files
**Cause**: Export not called or export failed

**Solution**:
```rust
// Check export result
if let Err(e) = memscope.export_json("analysis") {
    eprintln!("Export failed: {}", e);
}
```

### Issue: Variables Not Tracked
**Cause**: Forgot to call `track_var!`

**Solution**:
```rust
// ✅ Correct
let data = vec![1, 2, 3];
track_var!(data);

// ❌ Missing
let data = vec![1, 2, 3];
// track_var!(data); // Forgot this
```

### Issue: No Memory Statistics
**Cause**: Query before any allocations

**Solution**:
```rust
let memscope = memscope_rs::MemScope::new();

// Track something first
let data = vec![1, 2, 3];
track_var!(data);

// Now query
let stats = memscope.summary()?;
```

## 📚 Next Steps

Now that you understand basic usage:

1. **[Memory Analysis Guide](../user-guide/memory-analysis.md)** - Learn advanced analysis
2. **[Export Formats](../user-guide/export-formats.md)** - Understand output formats
3. **[Advanced Examples](performance-profiling.md)** - Explore complex scenarios
4. **[Smart Pointers](smart-pointers.md)** - Deep dive into Rc/Arc tracking

## 💡 Key Takeaways

- **Create MemScope instance** - Use `memscope_rs::MemScope::new()` at program start
- **Track with macros** - Use `track_var!` for zero-cost tracking
- **Variables work normally** - No behavior changes after tracking
- **Export for analysis** - Generate JSON and SVG reports
- **Check statistics** - Use `memscope.summary()` for real-time stats
- **Smart pointers work** - Rc/Arc reference counting tracked automatically

Start with the basic example and expand from there! 🎯