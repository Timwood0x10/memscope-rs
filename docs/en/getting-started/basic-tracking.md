# Basic Tracking Guide

Learn the core tracking functionality of memscope-rs and master the three tracking macros.

## 🎯 Core Concepts

### What is Memory Tracking?

Memory tracking in memscope-rs monitors:
- **Heap allocations** - Vec, String, Box, HashMap, etc.
- **Smart pointers** - Rc, Arc reference counting
- **Variable lifecycles** - Creation to destruction timing
- **Memory patterns** - Usage trends and anomalies

### Zero-Cost Philosophy

memscope-rs is designed with zero-cost abstraction:
- `track_var!` adds **no runtime overhead**
- Variables work **exactly the same** after tracking
- **No ownership changes** or performance impact
- Safe for **production use**

## 🚀 The Three Tracking Macros

### 1. track_var! - The Recommended Choice

**Best for**: 95% of use cases, production monitoring

```rust
use memscope_rs::{init, track_var};

fn main() {
    init();
    
    // Track any heap-allocated type
    let my_vec = vec![1, 2, 3, 4, 5];
    track_var!(my_vec);
    
    let my_string = String::from("Hello, world!");
    track_var!(my_string);
    
    let my_box = Box::new(42);
    track_var!(my_box);
    
    // Variables work completely normally
    println!("Vec: {:?}", my_vec);
    println!("String: {}", my_string);
    println!("Box: {}", *my_box);
}
```

**Key Features**:
- ✅ Zero performance overhead
- ✅ No ownership changes
- ✅ Works with all types
- ✅ Production-safe

### 2. track_var_smart! - The Convenient Choice

**Best for**: Mixed types, rapid prototyping

```rust
use memscope_rs::{init, track_var_smart};

fn main() {
    init();
    
    // Automatically optimizes based on type
    let number = track_var_smart!(42i32);        // Copy type
    let text = track_var_smart!(String::from("hello")); // Non-Copy type
    let boxed = track_var_smart!(Box::new(100)); // Smart pointer
    
    // All variables remain usable
    println!("{}, {}, {}", number, text, *boxed);
}
```

**Key Features**:
- ✅ Automatic type optimization
- ✅ Returns original value
- ✅ Method chaining support
- ✅ Minimal overhead

### 3. track_var_owned! - The Precise Choice

**Best for**: Detailed lifecycle analysis, debugging

```rust
use memscope_rs::{init, track_var_owned};

fn main() {
    init();
    
    let data = vec![1, 2, 3, 4, 5];
    let tracked = track_var_owned!(data); // Takes ownership
    
    // Transparent usage through Deref
    println!("Length: {}", tracked.len());
    println!("First: {}", tracked[0]);
    
    // Can retrieve original if needed
    let original = tracked.into_inner();
} // Precise lifecycle timing recorded here
```

**Key Features**:
- ✅ Precise timing measurements
- ✅ Automatic cleanup detection
- ✅ Enhanced smart pointer analysis
- ⚠️ Takes ownership

## 📊 Smart Pointer Tracking

### Reference Counting Analysis

```rust
use memscope_rs::{init, track_var};
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    init();
    
    // Rc tracking
    let rc_data = Rc::new(vec![1, 2, 3]);
    track_var!(rc_data);
    println!("Initial Rc count: {}", Rc::strong_count(&rc_data));
    
    let rc_clone = Rc::clone(&rc_data);
    track_var!(rc_clone);
    println!("After clone: {}", Rc::strong_count(&rc_data));
    
    // Arc tracking (thread-safe)
    let arc_data = Arc::new(String::from("shared"));
    track_var!(arc_data);
    
    let arc_clone = Arc::clone(&arc_data);
    track_var!(arc_clone);
    
    // Reference count changes are automatically tracked
}
```

### Box and Unique Ownership

```rust
use memscope_rs::{init, track_var};

fn main() {
    init();
    
    // Box tracking
    let boxed_data = Box::new(vec![1, 2, 3, 4, 5]);
    track_var!(boxed_data);
    
    // Box provides unique ownership
    println!("Boxed data: {:?}", *boxed_data);
    
    // Moving Box transfers ownership
    let moved_box = boxed_data; // Ownership transferred
    track_var!(moved_box);
}
```

## 🔄 Collection Types

### Standard Collections

```rust
use memscope_rs::{init, track_var};
use std::collections::{HashMap, HashSet, VecDeque};

fn main() {
    init();
    
    // Vector
    let mut vec = Vec::with_capacity(100);
    vec.extend(0..50);
    track_var!(vec);
    
    // HashMap
    let mut map = HashMap::new();
    map.insert("key1", "value1");
    map.insert("key2", "value2");
    track_var!(map);
    
    // HashSet
    let mut set = HashSet::new();
    set.insert(1);
    set.insert(2);
    track_var!(set);
    
    // VecDeque
    let mut deque = VecDeque::new();
    deque.push_back(1);
    deque.push_front(0);
    track_var!(deque);
}
```

### Nested Collections

```rust
use memscope_rs::{init, track_var};

fn main() {
    init();
    
    // Vector of vectors
    let nested_vec = vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ];
    track_var!(nested_vec);
    
    // HashMap with Vec values
    let mut complex_map = std::collections::HashMap::new();
    complex_map.insert("numbers", vec![1, 2, 3]);
    complex_map.insert("letters", vec![4, 5, 6]);
    track_var!(complex_map);
}
```

## 🎯 Tracking Strategies

### Selective Tracking

```rust
use memscope_rs::{init, track_var};

fn main() {
    init();
    
    // Track important allocations
    let important_data = vec![0; 1024 * 1024]; // 1MB
    track_var!(important_data);
    
    // Don't track small/temporary data
    let temp = vec![1, 2, 3]; // Small, temporary
    // No tracking needed
    
    // Track long-lived data
    let persistent = String::with_capacity(10000);
    track_var!(persistent);
}
```

### Function-Level Tracking

```rust
use memscope_rs::{init, track_var};

fn process_data(input: Vec<i32>) -> Vec<i32> {
    track_var!(input); // Track input parameter
    
    let mut result = Vec::with_capacity(input.len());
    track_var!(result); // Track result buffer
    
    for item in input {
        result.push(item * 2);
    }
    
    result
}

fn main() {
    init();
    
    let data = vec![1, 2, 3, 4, 5];
    let processed = process_data(data);
    track_var!(processed); // Track final result
}
```

### Conditional Tracking

```rust
use memscope_rs::{init, track_var};

// Only track in debug builds
#[cfg(debug_assertions)]
macro_rules! debug_track {
    ($var:expr) => { track_var!($var) };
}

#[cfg(not(debug_assertions))]
macro_rules! debug_track {
    ($var:expr) => {};
}

fn main() {
    init();
    
    let data = vec![1, 2, 3];
    debug_track!(data); // Only tracks in debug mode
}
```

## 📈 Getting Statistics

### Real-Time Statistics

```rust
use memscope_rs::{init, track_var, get_global_tracker};

fn main() {
    init();
    
    // Create some tracked data
    let data1 = vec![1; 1000];
    track_var!(data1);
    
    let data2 = String::from("Hello, world!".repeat(100));
    track_var!(data2);
    
    // Get current statistics
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("📊 Memory Statistics:");
        println!("  Active allocations: {}", stats.active_allocations);
        println!("  Active memory: {} bytes", stats.active_memory);
        println!("  Peak memory: {} bytes", stats.peak_memory);
        println!("  Total allocations: {}", stats.total_allocations);
    }
}
```

### Allocation Details

```rust
use memscope_rs::{init, track_var, get_global_tracker};

fn main() {
    init();
    
    let vec1 = vec![1; 100];
    track_var!(vec1);
    
    let vec2 = vec![2; 200];
    track_var!(vec2);
    
    let tracker = get_global_tracker();
    if let Ok(allocations) = tracker.get_active_allocations() {
        println!("📋 Active Allocations:");
        for alloc in allocations {
            if let Some(var_name) = &alloc.var_name {
                println!("  {}: {} bytes", var_name, alloc.size);
            }
        }
    }
}
```

## 🔧 Best Practices

### 1. Initialize Early

```rust
// ✅ Good
fn main() {
    memscope_rs::init(); // First thing in main
    
    // Your program logic...
}

// ❌ Avoid
fn some_function() {
    memscope_rs::init(); // Don't initialize in functions
}
```

### 2. Track Strategically

```rust
// ✅ Track heap allocations
let heap_data = vec![1, 2, 3];
track_var!(heap_data);

// ✅ Track large allocations
let large_buffer = vec![0; 1024 * 1024];
track_var!(large_buffer);

// ❌ Don't track stack primitives
let stack_int = 42; // No need to track
```

### 3. Use Appropriate Macro

```rust
// ✅ Most cases: zero-cost tracking
track_var!(data);

// ✅ Mixed types: smart tracking
let result = track_var_smart!(process_data());

// ✅ Precise analysis: owned tracking
let tracked = track_var_owned!(critical_data);
```

### 4. Export Results

```rust
use memscope_rs::get_global_tracker;

fn main() {
    // ... tracking code ...
    
    // Export at program end
    let tracker = get_global_tracker();
    tracker.export_to_json("analysis").unwrap();
    tracker.export_memory_analysis("chart.svg").unwrap();
}
```

## 🚀 Next Steps

Now that you understand basic tracking:

1. **[First Analysis](first-analysis.md)** - Learn to interpret results
2. **[Memory Analysis Guide](../user-guide/memory-analysis.md)** - Advanced analysis techniques
3. **[Export Formats](../user-guide/export-formats.md)** - Choose the right output format

## 💡 Key Takeaways

- **Use `track_var!` for most cases** - Zero overhead, production-safe
- **Track heap allocations** - Vec, String, Box, HashMap, etc.
- **Smart pointers work automatically** - Rc/Arc reference counting tracked
- **Variables remain normal** - No behavior changes after tracking
- **Initialize once** - Call `init()` at program start
- **Export results** - Generate reports for analysis

Start with `track_var!` and expand from there! 🎯