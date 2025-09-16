# Smart Pointer Memory Analysis

This guide demonstrates how to use memscope-rs to analyze memory usage patterns of Rust smart pointers, including tracking and analysis of `Box`, `Rc`, `Arc`, `RefCell`, and more.

## 🎯 Learning Objectives

- Track memory allocation of different smart pointer types
- Analyze reference count change patterns
- Detect circular references and memory leaks
- Understand performance impact of smart pointers
- Generate analysis reports for smart pointer usage

## 📦 Smart Pointer Types Overview

| Smart Pointer | Purpose | Thread Safe | Reference Counted |
|---------------|---------|-------------|-------------------|
| `Box<T>` | Heap allocation | ❌ | ❌ |
| `Rc<T>` | Shared ownership | ❌ | ✅ |
| `Arc<T>` | Thread-safe sharing | ✅ | ✅ |
| `RefCell<T>` | Interior mutability | ❌ | ❌ |
| `Mutex<T>` | Thread-safe mutability | ✅ | ❌ |

## 🚀 Complete Example

### Basic Smart Pointer Tracking

```rust
use memscope_rs::{init, track_var, get_global_tracker};
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    // 1. Box pointer analysis
    analyze_box_pointers();
    
    // 2. Rc reference counting analysis
    analyze_rc_pointers();
    
    // 3. Arc thread-safe analysis
    analyze_arc_pointers();
    
    // 4. RefCell interior mutability analysis
    analyze_refcell_patterns();
    
    // 5. Export analysis results
    let tracker = get_global_tracker();
    tracker.export_to_binary("smart_pointer_analysis")?;
    
    println!("✅ Smart pointer analysis complete!");
    println!("Run: make html DIR=MemoryAnalysis/smart_pointer_analysis BASE=smart_pointer_analysis");
    
    Ok(())
}
```

## 📦 Box Pointer Analysis

### Basic Box Usage

```rust
fn analyze_box_pointers() {
    println!("📦 Analyzing Box pointers...");
    
    // Create different sized Boxes
    let small_box = Box::new(42i32);
    track_var!(small_box);
    
    let large_box = Box::new(vec![0; 10000]);
    track_var!(large_box);
    
    let string_box = Box::new(String::from("Hello, Box!"));
    track_var!(string_box);
    
    // Nested Box
    let nested_box = Box::new(Box::new(Box::new(100)));
    track_var!(nested_box);
    
    println!("  ✅ Box analysis complete");
}
```

### Box Performance Patterns

```rust
fn analyze_box_performance() {
    // Many small Boxes (potentially inefficient)
    let mut small_boxes = Vec::new();
    for i in 0..1000 {
        let boxed = Box::new(i);
        small_boxes.push(boxed);
    }
    track_var!(small_boxes);
    
    // Single large Box (more efficient)
    let large_data = vec![0; 1000];
    let large_box = Box::new(large_data);
    track_var!(large_box);
}
```

## 🔄 Rc Reference Counting Analysis

### Basic Rc Usage

```rust
fn analyze_rc_pointers() {
    println!("🔄 Analyzing Rc reference counting...");
    
    // Create original Rc
    let original = Rc::new(vec![1, 2, 3, 4, 5]);
    track_var!(original);
    println!("  Reference count: {}", Rc::strong_count(&original));
    
    // Create clones
    let clone1 = Rc::clone(&original);
    track_var!(clone1);
    println!("  Reference count: {}", Rc::strong_count(&original));
    
    let clone2 = Rc::clone(&original);
    track_var!(clone2);
    println!("  Reference count: {}", Rc::strong_count(&original));
    
    // Weak reference
    let weak_ref = Rc::downgrade(&original);
    track_var!(weak_ref);
    println!("  Strong: {}, Weak: {}", 
             Rc::strong_count(&original), 
             Rc::weak_count(&original));
    
    println!("  ✅ Rc analysis complete");
}
```

### Rc Circular Reference Detection

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct Node {
    value: i32,
    children: RefCell<Vec<Rc<Node>>>,
    parent: RefCell<Weak<Node>>,
}

fn analyze_circular_references() {
    println!("🔄 Detecting circular references...");
    
    let parent = Rc::new(Node {
        value: 1,
        children: RefCell::new(vec![]),
        parent: RefCell::new(Weak::new()),
    });
    track_var!(parent);
    
    let child = Rc::new(Node {
        value: 2,
        children: RefCell::new(vec![]),
        parent: RefCell::new(Rc::downgrade(&parent)),
    });
    track_var!(child);
    
    // Establish parent-child relationship
    parent.children.borrow_mut().push(Rc::clone(&child));
    
    println!("  Parent reference count: {}", Rc::strong_count(&parent));
    println!("  Child reference count: {}", Rc::strong_count(&child));
    
    // Note: No circular reference here because we used Weak
    println!("  ✅ No circular reference detection complete");
}
```

## 🧵 Arc Thread-Safe Analysis

### Multi-threaded Arc Usage

```rust
use std::sync::Arc;
use std::thread;

fn analyze_arc_pointers() {
    println!("🧵 Analyzing Arc thread-safe pointers...");
    
    let shared_data = Arc::new(vec![1, 2, 3, 4, 5]);
    track_var!(shared_data);
    
    let mut handles = vec![];
    
    // Share data across multiple threads
    for thread_id in 0..4 {
        let data_clone = Arc::clone(&shared_data);
        
        let handle = thread::spawn(move || {
            // Track clone in each thread
            track_var!(data_clone);
            
            println!("  Thread {} accessing data: {:?}", thread_id, data_clone);
            
            // Simulate some work
            thread::sleep(std::time::Duration::from_millis(100));
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("  ✅ Arc analysis complete");
}
```

### Arc + Mutex Pattern

```rust
use std::sync::{Arc, Mutex};

fn analyze_arc_mutex_pattern() {
    let shared_counter = Arc::new(Mutex::new(0));
    track_var!(shared_counter);
    
    let mut handles = vec![];
    
    for _ in 0..4 {
        let counter_clone = Arc::clone(&shared_counter);
        
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                let mut num = counter_clone.lock().unwrap();
                *num += 1;
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let final_count = *shared_counter.lock().unwrap();
    println!("  Final count: {}", final_count);
}
```

## 🔄 RefCell Interior Mutability Analysis

### Basic RefCell Usage

```rust
use std::cell::RefCell;

fn analyze_refcell_patterns() {
    println!("🔄 Analyzing RefCell interior mutability...");
    
    let data = RefCell::new(vec![1, 2, 3]);
    track_var!(data);
    
    // Immutable borrow
    {
        let borrowed = data.borrow();
        println!("  Data length: {}", borrowed.len());
        track_var!(borrowed);
    }
    
    // Mutable borrow
    {
        let mut borrowed_mut = data.borrow_mut();
        borrowed_mut.push(4);
        track_var!(borrowed_mut);
    }
    
    println!("  ✅ RefCell analysis complete");
}
```

### Rc + RefCell Combination Pattern

```rust
fn analyze_rc_refcell_combination() {
    let shared_data = Rc::new(RefCell::new(vec![1, 2, 3]));
    track_var!(shared_data);
    
    let clone1 = Rc::clone(&shared_data);
    let clone2 = Rc::clone(&shared_data);
    
    // Modify data through different clones
    clone1.borrow_mut().push(4);
    clone2.borrow_mut().push(5);
    
    println!("  Final data: {:?}", shared_data.borrow());
    println!("  Reference count: {}", Rc::strong_count(&shared_data));
}
```

## 📊 Performance Analysis and Optimization

### Smart Pointer Performance Comparison

```rust
use std::time::Instant;

fn benchmark_smart_pointers() {
    let iterations = 100000;
    
    // Box performance test
    let start = Instant::now();
    for i in 0..iterations {
        let boxed = Box::new(i);
        std::hint::black_box(boxed);
    }
    let box_time = start.elapsed();
    
    // Rc performance test
    let start = Instant::now();
    let rc_data = Rc::new(0);
    for _ in 0..iterations {
        let cloned = Rc::clone(&rc_data);
        std::hint::black_box(cloned);
    }
    let rc_time = start.elapsed();
    
    println!("📊 Performance comparison:");
    println!("  Box creation: {:?}", box_time);
    println!("  Rc cloning: {:?}", rc_time);
}
```

### Memory Usage Pattern Analysis

```rust
fn analyze_memory_patterns() {
    // Pattern 1: Deep nesting
    let deep_nested = Box::new(Box::new(Box::new(Box::new(42))));
    track_var!(deep_nested);
    
    // Pattern 2: Wide sharing
    let shared = Rc::new(vec![1; 1000]);
    let mut clones = Vec::new();
    for _ in 0..10 {
        clones.push(Rc::clone(&shared));
    }
    track_var!(clones);
    
    // Pattern 3: Mixed usage
    let mixed = Arc::new(Mutex::new(RefCell::new(Box::new(vec![1, 2, 3]))));
    track_var!(mixed);
}
```

## 🔍 Analysis Report Interpretation

### Generate Detailed Reports

```bash
# Export all formats
cargo run --example smart_pointer_analysis
memscope analyze --export all ./target/debug/examples/smart_pointer_analysis

# Generate HTML report
make html DIR=MemoryAnalysis/smart_pointer_analysis BASE=smart_pointer_analysis

# View JSON data
cat MemoryAnalysis/smart_pointer_analysis/smart_pointer_analysis_memory_analysis.json | jq .
```

### Key Metrics Interpretation

1. **Reference Count Changes**
   ```json
   {
     "var_name": "shared_rc",
     "type_name": "alloc::rc::Rc<alloc::vec::Vec<i32>>",
     "reference_count": 3,
     "weak_count": 1
   }
   ```

2. **Memory Distribution**
   - Box: Direct heap allocation
   - Rc: Reference count + data
   - Arc: Atomic reference count + data

3. **Lifecycle Patterns**
   - Short-term: Temporary Box
   - Medium-term: Shared Rc
   - Long-term: Global Arc

## 🛠️ Best Practices

### 1. Choose the Right Smart Pointer

```rust
// Single ownership -> Box
let unique_data = Box::new(expensive_computation());

// Single-threaded sharing -> Rc
let shared_config = Rc::new(load_configuration());

// Multi-threaded sharing -> Arc
let thread_safe_data = Arc::new(Mutex::new(shared_state));

// Interior mutability -> RefCell
let mutable_in_immutable = RefCell::new(counter);
```

### 2. Avoid Common Pitfalls

```rust
// ❌ Avoid: Unnecessary Box
let unnecessary = Box::new(42); // Just use i32 directly

// ✅ Recommended: Only use Box when needed
let necessary = Box::new(large_struct);

// ❌ Avoid: Circular references
// parent -> child -> parent (using Rc)

// ✅ Recommended: Use Weak to break cycles
// parent -> child, child -> Weak<parent>
```

### 3. Performance Optimization Tips

```rust
// Pre-allocate capacity
let mut data = Vec::with_capacity(1000);
let boxed_data = Box::new(data);

// Batch operations
let batch_data = (0..1000).collect::<Vec<_>>();
let shared_batch = Rc::new(batch_data);

// Reduce cloning
let data = Rc::new(expensive_data);
// Pass reference instead of cloning
process_data(&data);
```

## 🔧 Troubleshooting

### Common Issues

1. **Reference count not decreasing**
   ```rust
   // Check for circular references
   println!("Strong references: {}", Rc::strong_count(&data));
   println!("Weak references: {}", Rc::weak_count(&data));
   ```

2. **High memory usage**
   ```rust
   // Check for memory leaks
   let tracker = get_global_tracker();
   let stats = tracker.get_stats()?;
   println!("Active allocations: {}", stats.active_allocations);
   ```

3. **Performance issues**
   ```rust
   // Use Arc instead of Mutex<Rc<T>>
   // ❌ Inefficient
   let bad = Mutex::new(Rc::new(data));
   
   // ✅ Efficient
   let good = Arc::new(Mutex::new(data));
   ```

## 🔍 Advanced Smart Pointer Patterns

### Custom Smart Pointers

```rust
use std::ops::{Deref, DerefMut};

struct TrackedBox<T> {
    inner: Box<T>,
    id: usize,
}

impl<T> TrackedBox<T> {
    fn new(value: T, id: usize) -> Self {
        let boxed = Box::new(value);
        let tracked = Self { inner: boxed, id };
        track_var!(tracked);
        tracked
    }
}

impl<T> Deref for TrackedBox<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for TrackedBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
```

### Smart Pointer Composition

```rust
// Complex composition patterns
type SharedMutableData<T> = Arc<Mutex<RefCell<Box<T>>>>;
type ThreadSafeCache<K, V> = Arc<RwLock<HashMap<K, Arc<V>>>>;

fn analyze_complex_compositions() {
    // Shared mutable data across threads
    let complex_data: SharedMutableData<Vec<i32>> = 
        Arc::new(Mutex::new(RefCell::new(Box::new(vec![1, 2, 3]))));
    track_var!(complex_data);
    
    // Thread-safe cache with shared values
    let cache: ThreadSafeCache<String, Vec<u8>> = 
        Arc::new(RwLock::new(HashMap::new()));
    track_var!(cache);
    
    // Insert shared data
    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    cache.write().unwrap().insert("key1".to_string(), data);
}
```

## 📊 Memory Layout Analysis

### Understanding Memory Overhead

```rust
fn analyze_memory_overhead() {
    use std::mem;
    
    // Direct value
    let direct = vec![1, 2, 3];
    println!("Direct Vec size: {}", mem::size_of_val(&direct));
    
    // Boxed value
    let boxed = Box::new(vec![1, 2, 3]);
    println!("Box overhead: {}", mem::size_of_val(&boxed));
    track_var!(boxed);
    
    // Rc value
    let rc_value = Rc::new(vec![1, 2, 3]);
    println!("Rc overhead: {}", mem::size_of_val(&rc_value));
    track_var!(rc_value);
    
    // Arc value
    let arc_value = Arc::new(vec![1, 2, 3]);
    println!("Arc overhead: {}", mem::size_of_val(&arc_value));
    track_var!(arc_value);
}
```

## 🎉 Summary

Through this smart pointer analysis example, you learned:

✅ **Smart pointer tracking** - Track Box, Rc, Arc, RefCell, etc.  
✅ **Reference count analysis** - Understand reference count change patterns  
✅ **Circular reference detection** - Identify and avoid memory leaks  
✅ **Performance optimization** - Choose appropriate smart pointer types  
✅ **Best practices** - Avoid common pitfalls and performance issues  

Now you can effectively analyze and optimize smart pointer usage in your Rust programs! 🚀