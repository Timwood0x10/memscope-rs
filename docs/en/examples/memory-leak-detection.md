# Memory Leak Detection

This guide demonstrates how to use memscope-rs to detect and analyze memory leaks in Rust programs, including circular references, forgotten resource releases, and long-lived objects.

## 🎯 Learning Objectives

- Identify different types of memory leaks
- Use memscope-rs to detect circular references
- Analyze abnormal object lifecycles
- Understand common causes of memory leaks in Rust
- Generate memory leak analysis reports

## 🚨 Memory Leak Types

| Leak Type | Cause | Detection Method | Severity |
|-----------|-------|------------------|----------|
| **Circular References** | Rc/Arc cycles | Reference count analysis | High |
| **Forgotten Resources** | Manual resource management | Lifecycle tracking | Medium |
| **Long-lived Objects** | Global/static variables | Lifetime analysis | Low |
| **Async Leaks** | Incomplete Futures | Async state tracking | Medium |

## 🚀 Complete Detection Example

```rust
use memscope_rs::{init, track_var, get_global_tracker};
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    println!("🔍 Starting memory leak detection...");
    
    // 1. Circular reference detection
    detect_circular_references();
    
    // 2. Long-lived object detection
    detect_long_lived_objects();
    
    // 3. Resource leak detection
    detect_resource_leaks();
    
    // 4. Async memory leak detection
    detect_async_leaks();
    
    // 5. Export analysis results
    let tracker = get_global_tracker();
    tracker.export_to_binary("memory_leak_detection")?;
    
    println!("✅ Memory leak detection complete!");
    println!("Run: make html DIR=MemoryAnalysis/memory_leak_detection BASE=memory_leak_detection");
    
    Ok(())
}
```

## 🔄 Circular Reference Detection

### Classic Circular Reference Example

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct Parent {
    name: String,
    children: RefCell<Vec<Rc<Child>>>,
}

#[derive(Debug)]
struct Child {
    name: String,
    parent: RefCell<Option<Rc<Parent>>>, // This creates a circular reference!
}

fn detect_circular_references() {
    println!("🔄 Detecting circular references...");
    
    // Create parent node
    let parent = Rc::new(Parent {
        name: "Parent".to_string(),
        children: RefCell::new(vec![]),
    });
    track_var!(parent);
    
    // Create child node
    let child = Rc::new(Child {
        name: "Child".to_string(),
        parent: RefCell::new(None),
    });
    track_var!(child);
    
    // Establish circular reference
    parent.children.borrow_mut().push(Rc::clone(&child));
    *child.parent.borrow_mut() = Some(Rc::clone(&parent));
    
    println!("  Parent reference count: {}", Rc::strong_count(&parent));
    println!("  Child reference count: {}", Rc::strong_count(&child));
    
    // Note: This creates a memory leak!
    println!("  ⚠️ Circular reference detected - Memory leak!");
}
```

### Correct Circular Reference Solution

```rust
#[derive(Debug)]
struct SafeParent {
    name: String,
    children: RefCell<Vec<Rc<SafeChild>>>,
}

#[derive(Debug)]
struct SafeChild {
    name: String,
    parent: RefCell<Option<Weak<SafeParent>>>, // Use Weak to break the cycle
}

fn demonstrate_safe_references() {
    println!("✅ Demonstrating safe reference patterns...");
    
    let parent = Rc::new(SafeParent {
        name: "SafeParent".to_string(),
        children: RefCell::new(vec![]),
    });
    track_var!(parent);
    
    let child = Rc::new(SafeChild {
        name: "SafeChild".to_string(),
        parent: RefCell::new(None),
    });
    track_var!(child);
    
    // Establish safe parent-child relationship
    parent.children.borrow_mut().push(Rc::clone(&child));
    *child.parent.borrow_mut() = Some(Rc::downgrade(&parent));
    
    println!("  Parent reference count: {}", Rc::strong_count(&parent));
    println!("  Child reference count: {}", Rc::strong_count(&child));
    println!("  Parent weak reference count: {}", Rc::weak_count(&parent));
    
    println!("  ✅ No circular reference - Memory safe!");
}
```

## ⏰ Long-lived Object Detection

### Simulating Long-lived Objects

```rust
use std::time::{Duration, Instant};
use std::thread;

static mut GLOBAL_CACHE: Option<HashMap<String, Vec<u8>>> = None;

fn detect_long_lived_objects() {
    println!("⏰ Detecting long-lived objects...");
    
    // 1. Global cache (potential memory leak source)
    unsafe {
        GLOBAL_CACHE = Some(HashMap::new());
        if let Some(ref mut cache) = GLOBAL_CACHE {
            // Add lots of data to global cache
            for i in 0..1000 {
                let key = format!("key_{}", i);
                let value = vec![i as u8; 1024]; // 1KB per entry
                cache.insert(key, value);
            }
            track_var!(cache);
        }
    }
    
    // 2. Long-lived large object
    let long_lived_data = vec![0u8; 10 * 1024 * 1024]; // 10MB
    track_var!(long_lived_data);
    
    // 3. Simulate some short-term objects
    for i in 0..100 {
        let temp_data = vec![i; 100];
        track_var!(temp_data);
        // temp_data is dropped here
    }
    
    // 4. Medium-lived object
    let medium_lived = Arc::new(Mutex::new(vec![0; 1000]));
    track_var!(medium_lived);
    
    println!("  ✅ Long-lived object detection complete");
    
    // Note: long_lived_data and GLOBAL_CACHE will live until program end
}
```

### Lifetime Analysis

```rust
fn analyze_object_lifetimes() {
    let start_time = Instant::now();
    
    // Create objects with different lifetimes
    let short_lived = {
        let data = vec![1; 1000];
        track_var!(data);
        data
    }; // data should be dropped here, but we returned it
    
    thread::sleep(Duration::from_millis(100));
    
    let medium_lived = vec![2; 1000];
    track_var!(medium_lived);
    
    thread::sleep(Duration::from_millis(200));
    
    let long_lived = Box::leak(Box::new(vec![3; 1000])); // Intentional leak!
    track_var!(long_lived);
    
    println!("  Object creation time: {:?}", start_time.elapsed());
    println!("  ⚠️ Intentional memory leak detected");
}
```

## 💧 Resource Leak Detection

### File Handle Leaks

```rust
use std::fs::File;
use std::io::Read;

fn detect_resource_leaks() {
    println!("💧 Detecting resource leaks...");
    
    // 1. File handle leak example
    let mut leaked_files = Vec::new();
    for i in 0..10 {
        match File::open("Cargo.toml") {
            Ok(file) => {
                leaked_files.push(file); // File handles held but may not be properly closed
            }
            Err(_) => continue,
        }
    }
    track_var!(leaked_files);
    
    // 2. Memory allocation leak
    let mut leaked_memory = Vec::new();
    for i in 0..100 {
        let data = Box::leak(Box::new(vec![i; 1000])); // Intentional memory leak
        leaked_memory.push(data as *const Vec<i32>);
    }
    track_var!(leaked_memory);
    
    // 3. Thread handle leak
    let mut thread_handles = Vec::new();
    for i in 0..5 {
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(10)); // Long-running
            println!("Thread {} finished", i);
        });
        thread_handles.push(handle);
    }
    track_var!(thread_handles);
    // Note: If join() is not called, thread resources may leak
    
    println!("  ⚠️ Multiple resource leaks detected");
}
```

### Proper Resource Management

```rust
fn demonstrate_proper_resource_management() {
    println!("✅ Demonstrating proper resource management...");
    
    // 1. Use RAII for automatic file management
    {
        let _file = File::open("Cargo.toml").expect("Failed to open file");
        // File automatically closed when scope ends
    }
    
    // 2. Use Drop trait for automatic cleanup
    struct ManagedResource {
        data: Vec<u8>,
    }
    
    impl Drop for ManagedResource {
        fn drop(&mut self) {
            println!("  Cleaning up resource: {} bytes", self.data.len());
        }
    }
    
    {
        let resource = ManagedResource {
            data: vec![0; 1000],
        };
        track_var!(resource);
        // resource automatically calls drop here
    }
    
    // 3. Properly handle threads
    let handles: Vec<_> = (0..3).map(|i| {
        std::thread::spawn(move || {
            println!("Worker thread {} completed", i);
        })
    }).collect();
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("  ✅ All resources properly cleaned up");
}
```

## 🔮 Async Memory Leak Detection

### Async Task Leaks

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct LeakyFuture {
    data: Vec<u8>,
    completed: bool,
}

impl Future for LeakyFuture {
    type Output = ();
    
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.completed {
            Poll::Ready(())
        } else {
            // Simulate a Future that never completes
            Poll::Pending
        }
    }
}

fn detect_async_leaks() {
    println!("🔮 Detecting async memory leaks...");
    
    // 1. Create a Future that never completes
    let leaky_future = LeakyFuture {
        data: vec![0; 10000], // 10KB data
        completed: false,
    };
    track_var!(leaky_future);
    
    // 2. Create many async tasks but don't await completion
    let mut pending_futures = Vec::new();
    for i in 0..100 {
        let future = LeakyFuture {
            data: vec![i as u8; 1000],
            completed: false,
        };
        pending_futures.push(Box::pin(future));
    }
    track_var!(pending_futures);
    
    println!("  ⚠️ Async task leaks detected");
    
    // Note: These Futures will never complete, causing memory leaks
}
```

## 📊 Leak Analysis Reports

### Generate Detailed Reports

```bash
# Run detection
cargo run --example memory_leak_detection

# Generate HTML report
make html DIR=MemoryAnalysis/memory_leak_detection BASE=memory_leak_detection

# Analyze JSON data
cat MemoryAnalysis/memory_leak_detection/memory_leak_detection_memory_analysis.json | jq '.allocations[] | select(.is_leaked == true)'
```

### Key Metrics Interpretation

1. **Abnormal Reference Counts**
   ```json
   {
     "var_name": "circular_parent",
     "type_name": "alloc::rc::Rc<Parent>",
     "reference_count": 2,
     "expected_count": 1,
     "is_leaked": true
   }
   ```

2. **Long-lived Objects**
   ```json
   {
     "var_name": "long_lived_data",
     "lifetime_ms": 300000,
     "size": 10485760,
     "leak_probability": "high"
   }
   ```

3. **Resource Handle Leaks**
   ```json
   {
     "resource_type": "file_handle",
     "count": 10,
     "status": "not_closed"
   }
   ```

## 🛠️ Leak Detection Tools

### Automated Detection Functions

```rust
use memscope_rs::analysis::detect_memory_leaks;

fn automated_leak_detection() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = get_global_tracker();
    let allocations = tracker.get_active_allocations()?;
    
    // Detect objects alive for more than 5 seconds
    let potential_leaks = detect_memory_leaks(&allocations, 5000);
    
    if !potential_leaks.is_empty() {
        println!("🚨 Found {} potential memory leaks:", potential_leaks.len());
        
        for leak in &potential_leaks {
            println!("  - {} bytes, alive for {}ms", leak.size, leak.lifetime_ms);
            if let Some(name) = &leak.var_name {
                println!("    Variable name: {}", name);
            }
        }
    } else {
        println!("✅ No memory leaks found");
    }
    
    Ok(())
}
```

### Circular Reference Detector

```rust
use memscope_rs::analysis::analyze_circular_references;

fn automated_circular_reference_detection() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = get_global_tracker();
    let allocations = tracker.get_active_allocations()?;
    
    let circular_refs = analyze_circular_references(&allocations)?;
    
    if !circular_refs.is_empty() {
        println!("🔄 Found {} circular references:", circular_refs.len());
        
        for circular_ref in &circular_refs {
            println!("  - Cycle length: {}", circular_ref.cycle_length);
            println!("    Involved allocations: {:?}", circular_ref.involved_allocations);
            println!("    Severity: {:?}", circular_ref.severity);
        }
    } else {
        println!("✅ No circular references found");
    }
    
    Ok(())
}
```

## 🔧 Prevention and Fix Strategies

### 1. Use Weak References

```rust
// ❌ Prone to circular references
struct BadNode {
    children: Vec<Rc<BadNode>>,
    parent: Option<Rc<BadNode>>,
}

// ✅ Use Weak to break cycles
struct GoodNode {
    children: Vec<Rc<GoodNode>>,
    parent: Option<Weak<GoodNode>>,
}
```

### 2. Implement Drop Trait

```rust
struct ResourceManager {
    resources: Vec<File>,
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        println!("Cleaning up {} resources", self.resources.len());
        // Resources are automatically cleaned up
    }
}
```

### 3. Use Scope Management

```rust
fn scoped_resource_management() {
    // Use scopes to limit object lifetimes
    {
        let temp_data = vec![0; 1000];
        track_var!(temp_data);
        // temp_data automatically released here
    }
    
    // Use RAII pattern
    let _guard = std::fs::File::open("temp.txt");
    // File automatically closed when _guard is destroyed
}
```

### 4. Periodic Cleanup

```rust
fn periodic_cleanup() {
    static mut CLEANUP_COUNTER: usize = 0;
    
    unsafe {
        CLEANUP_COUNTER += 1;
        if CLEANUP_COUNTER % 1000 == 0 {
            // Clean up every 1000 operations
            if let Some(ref mut cache) = GLOBAL_CACHE {
                cache.clear();
                println!("Cleaned global cache");
            }
        }
    }
}
```

## 🎯 Best Practices

### 1. Design Principles

- **Prefer stack allocation** - Avoid unnecessary heap allocation
- **Clear ownership** - Use Rust's ownership system
- **Limit lifetimes** - Use scopes to control object lifetimes
- **Avoid global state** - Reduce use of global variables

### 2. Detection Strategy

- **Regular detection** - Run leak detection regularly during development
- **Automated testing** - Integrate memory leak detection in CI/CD
- **Performance monitoring** - Monitor memory usage in production

### 3. Fix Process

1. **Identify leaks** - Use memscope-rs to identify leak locations
2. **Analyze causes** - Understand root causes of leaks
3. **Design fixes** - Choose appropriate fix strategies
4. **Verify fixes** - Confirm no more leaks after fixes

## 🔍 Advanced Leak Detection

### Custom Leak Detectors

```rust
struct LeakDetector {
    tracked_objects: HashMap<usize, ObjectInfo>,
    leak_threshold_ms: u64,
}

struct ObjectInfo {
    creation_time: Instant,
    size: usize,
    type_name: String,
}

impl LeakDetector {
    fn new(threshold_ms: u64) -> Self {
        Self {
            tracked_objects: HashMap::new(),
            leak_threshold_ms: threshold_ms,
        }
    }
    
    fn track_object(&mut self, id: usize, size: usize, type_name: String) {
        self.tracked_objects.insert(id, ObjectInfo {
            creation_time: Instant::now(),
            size,
            type_name,
        });
    }
    
    fn check_for_leaks(&self) -> Vec<usize> {
        let now = Instant::now();
        self.tracked_objects
            .iter()
            .filter(|(_, info)| {
                now.duration_since(info.creation_time).as_millis() as u64 
                    > self.leak_threshold_ms
            })
            .map(|(id, _)| *id)
            .collect()
    }
}
```

### Memory Pattern Analysis

```rust
fn analyze_allocation_patterns() {
    let tracker = get_global_tracker();
    
    // Analyze allocation frequency
    let mut allocation_times = Vec::new();
    let mut last_check = Instant::now();
    
    for _ in 0..1000 {
        let data = vec![0; 1000];
        track_var!(data);
        
        let now = Instant::now();
        allocation_times.push(now.duration_since(last_check));
        last_check = now;
    }
    
    // Calculate statistics
    let avg_interval: Duration = allocation_times.iter().sum::<Duration>() / allocation_times.len() as u32;
    let max_interval = allocation_times.iter().max().unwrap();
    let min_interval = allocation_times.iter().min().unwrap();
    
    println!("📊 Allocation Pattern Analysis:");
    println!("  Average interval: {:?}", avg_interval);
    println!("  Max interval: {:?}", max_interval);
    println!("  Min interval: {:?}", min_interval);
    
    if avg_interval < Duration::from_millis(1) {
        println!("  ⚠️ High-frequency allocations detected - potential performance issue");
    }
}
```

## 🎉 Summary

Through this memory leak detection example, you learned:

✅ **Leak type identification** - Circular references, resource leaks, long-lived objects  
✅ **Automated detection tools** - Using memscope-rs analysis features  
✅ **Prevention strategies** - Weak references, RAII, scope management  
✅ **Fix techniques** - Drop trait, periodic cleanup, ownership design  
✅ **Best practices** - Design principles, detection strategies, fix processes  

Now you can effectively detect and fix memory leaks in your Rust programs! 🚀