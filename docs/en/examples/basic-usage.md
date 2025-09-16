# Basic Usage Example

This document provides detailed usage instructions and best practices based on `examples/basic_usage.rs`.

## 🎯 Complete Example Analysis

### Basic Setup
```rust
use memscope_rs::{get_global_tracker, init, track_var};
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    // 1. Initialize memory tracking system
    init();
    println!("memscope-rs initialized. Tracking memory allocations...");
```

**Key Points**:
- `init()` must be called before any tracking operations
- Only needs to be called once, usually at the start of `main()` function
- After initialization, the global allocator starts working

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
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
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
    if let Err(e) = tracker.export_to_json("basic_usage_snapshot") {
        eprintln!("Failed to export JSON: {e}");
    } else {
        println!("Successfully exported JSON to MemoryAnalysis/basic_usage/");
    }

    // 7. Export memory usage visualization to SVG
    println!("\nExporting memory usage visualization to MemoryAnalysis/basic_usage/...");
    if let Err(e) = tracker.export_memory_analysis("basic_usage_graph.svg") {
        eprintln!("Failed to export SVG: {e}");
    } else {
        println!("Successfully exported SVG to MemoryAnalysis/basic_usage/");
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
Tracked 'boxed_value2'
Tracked 'rc_data'
Tracked 'arc_data'
Tracked 'rc_data_clone' (shares allocation with 'rc_data')

Sum of 'numbers_vec': 15
Length of 'text_string': 19
Value in 'boxed_value': 100
Value in 'boxed_value2': 200
First element of 'rc_data': 10
Content of 'arc_data': Shared data

Memory Statistics:
  Active allocations: 7
  Active memory: 234 bytes
  Total allocations: 7
  Peak memory: 234 bytes

Exporting memory snapshot to MemoryAnalysis/basic_usage/...
Successfully exported JSON to MemoryAnalysis/basic_usage/

Exporting memory usage visualization to MemoryAnalysis/basic_usage/...
Successfully exported SVG to MemoryAnalysis/basic_usage/

Example finished. Check 'basic_usage_snapshot.json' and 'basic_usage_graph.svg'.
The SVG shows memory usage by type and individual allocations.
```

### Generated Files
```
MemoryAnalysis/basic_usage/
├── basic_usage_snapshot_memory_analysis.json  # Basic memory analysis
├── basic_usage_snapshot_lifetime.json         # Lifecycle data
├── basic_usage_snapshot_performance.json      # Performance data
├── basic_usage_snapshot_unsafe_ffi.json       # Unsafe/FFI data
├── basic_usage_snapshot_complex_types.json    # Complex type analysis
└── basic_usage_graph.svg                      # Visualization chart
```

### Generate HTML Report with make Command
```bash
# Run example
cargo run --example basic_usage

# Generate HTML report
make html DIR=MemoryAnalysis/basic_usage BASE=basic_usage_snapshot

# Open report
open memory_report.html
```

## 📊 Memory Analysis Details

### JSON Data Structure
Generated JSON files contain:

```json
{
  "metadata": {
    "export_timestamp": 1691234567890,
    "total_allocations": 5,
    "active_allocations": 5
  },
  "allocations": [
    {
      "ptr": 140712345678912,
      "size": 40,
      "var_name": "numbers_vec",
      "type_name": "Vec<i32>",
      "timestamp_alloc": 1691234567123,
      "is_leaked": false
    },
    {
      "ptr": 140712345678952,
      "size": 19,
      "var_name": "text_string", 
      "type_name": "String",
      "timestamp_alloc": 1691234567124,
      "is_leaked": false
    }
    // ... more allocation info
  ]
}
```

### SVG Visualization
Generated SVG charts show:
- Memory usage distribution by type
- Allocation timeline
- Memory size comparison

## 🚀 Extended Examples

### Add More Tracking
```rust
use memscope_rs::{track_var, init, get_global_tracker};
use std::collections::{HashMap, VecDeque};

fn extended_example() {
    init();
    
    // Collection types
    let mut map = HashMap::new();
    map.insert("key1", "value1");
    map.insert("key2", "value2");
    track_var!(map);
    
    let mut deque = VecDeque::new();
    deque.push_back(1);
    deque.push_back(2);
    track_var!(deque);
    
    // Nested structures
    let nested = vec![vec![1, 2], vec![3, 4, 5]];
    track_var!(nested);
    
    // Large allocations
    let large_buffer = vec![0u8; 1024 * 1024]; // 1MB
    track_var!(large_buffer);
    
    // Export detailed analysis
    let tracker = get_global_tracker();
    tracker.export_to_html("extended_analysis.html").unwrap();
}
```

### Function-Level Tracking
```rust
fn process_data(input: Vec<i32>) -> Vec<i32> {
    track_var!(input);
    
    let mut result = Vec::with_capacity(input.len());
    track_var!(result);
    
    for item in input {
        result.push(item * 2);
    }
    
    result
}

fn main() {
    init();
    
    let data = vec![1, 2, 3, 4, 5];
    let processed = process_data(data);
    track_var!(processed);
    
    let tracker = get_global_tracker();
    tracker.export_to_json("function_level_tracking").unwrap();
}
```

### Lifecycle Analysis
```rust
fn lifecycle_example() {
    init();
    
    {
        let short_lived = vec![1, 2, 3];
        track_var!(short_lived);
        // short_lived is destroyed here
    }
    
    let long_lived = vec![4, 5, 6];
    track_var!(long_lived);
    
    // Export shows different lifecycle patterns
    let tracker = get_global_tracker();
    tracker.export_to_html("lifecycle_analysis.html").unwrap();
}
```

## 💡 Best Practices

### 1. Initialization Timing
```rust
// ✅ Good practice
fn main() {
    memscope_rs::init(); // Initialize at program start
    
    // Your program logic...
}

// ❌ Avoid
fn some_function() {
    memscope_rs::init(); // Don't repeatedly initialize in functions
}
```

### 2. Tracking Strategy
```rust
// ✅ Track important heap allocations
let important_data = vec![1, 2, 3];
track_var!(important_data);

// ✅ Track large allocations
let large_buffer = vec![0; 1024 * 1024];
track_var!(large_buffer);

// ❌ No need to track simple stack values
let simple_int = 42; // No need to track
```

### 3. Export Timing
```rust
// ✅ Export before program ends
fn main() {
    init();
    
    // Program logic...
    
    // Export analysis results
    let tracker = get_global_tracker();
    tracker.export_to_html("final_analysis.html").unwrap();
}
```

### 4. Error Handling
```rust
// ✅ Proper error handling
let tracker = get_global_tracker();
match tracker.export_to_json("analysis") {
    Ok(_) => println!("Export successful"),
    Err(e) => eprintln!("Export failed: {}", e),
}
```

This basic example provides you with a complete starting point for using memscope-rs. From here, you can explore more advanced features! 🎯