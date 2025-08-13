# Concurrent Code Memory Analysis

This guide demonstrates how to use memscope-rs to analyze memory usage patterns in multi-threaded programs, including shared variable tracking, race condition detection, and performance analysis.

## 🎯 Learning Objectives

- Track shared variables in multi-threaded programs
- Analyze memory usage in producer-consumer patterns
- Detect load balancing in work-stealing queues
- Understand memory impact of atomic operations
- Generate memory analysis reports for concurrent programs

## 🚀 Complete Example

Run our provided advanced example:

```bash
# Run advanced multi-threaded memory analysis example
cargo run --example advanced_metrics_demo

# Generate interactive HTML report
make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo

# Open report to view results
open memory_report.html
```

## 📊 Example Output

After running the example, you'll see output similar to:

```text
🚀 Advanced Memory Metrics Demo
===============================
📊 Creating advanced memory scenarios...
   Creating complex nested data structures...
   Creating smart pointer relationships...
   Creating unsafe code and FFI scenarios...
   Creating multi-threaded scenarios with shared variable tracking...
   ✅ Complex multi-threaded scenarios with shared variable tracking completed
   Creating memory layout optimization examples...
   Creating performance-critical allocations...
   Creating main-thread allocations with clear variable names...
✅ Created advanced allocation scenarios with rich metadata

💾 Exporting to binary format...
✅ Binary export completed in 211ms
📁 Binary file: MemoryAnalysis/advanced_metrics_demo/advanced_metrics_demo.memscope (480KB)

🔄 Converting binary to standard JSON files...
✅ Standard JSON conversion completed in 17.1s
📄 Generated JSON files:
  • advanced_metrics_demo_memory_analysis.json (84KB)
  • advanced_metrics_demo_lifetime.json (69KB)
  • advanced_metrics_demo_performance.json (125KB)
  • advanced_metrics_demo_unsafe_ffi.json (118KB)
  • advanced_metrics_demo_complex_types.json (330KB)

📈 Advanced Performance Analysis:
  📊 Binary export time:     211ms
  📊 Standard JSON time:     17.1s
  🚀 Speed improvement:      80.72x faster
  📁 Binary file size:       480KB
  📁 JSON files size:        728KB (5 files)
  💾 Size reduction:         34.0%

🔍 Advanced Memory Analysis:
  • Total allocations: 289
  • Smart pointer usage: 20
  • Unsafe operations: 0
  • Multi-threaded allocations: 294
  • Complex data structures: 78
```

## 🧵 Multi-threaded Scenario Analysis

### 1. Producer-Consumer Pattern

The example implements a complex producer-consumer scenario:

- **3 producer threads** - Add data to shared buffer
- **2 consumer threads** - Remove data from shared buffer
- **Shared statistics** - Track production and consumption counts

```rust
// Core data structures (from example)
let shared_buffer = Arc<Mutex<VecDeque<String>>>;
let buffer_stats = Arc<Mutex<(usize, usize)>>; // (produced, consumed)

// Each thread's results are tracked separately
let consumer_data = (vec![consumer_id], consumed_items, vec![stats]);
track_var!(consumer_data);
```

### 2. Read-Write Lock Cache Access

Simulates a high-concurrency cache system:

- **2 writer threads** - Update cache data
- **4 reader threads** - Concurrent cache reads
- **Access statistics** - Track reads, writes, and cache misses

```rust
let shared_cache = Arc<RwLock<HashMap<String, Vec<u8>>>>;
let cache_metrics = Arc<Mutex<(usize, usize, usize)>>; // (reads, writes, misses)
```

### 3. Work-Stealing Queue

Implements a work-stealing algorithm:

- **4 worker threads** - Each with its own work queue
- **Task stealing** - Idle threads steal tasks from others
- **Load statistics** - Track work completed by each thread

```rust
let work_queues: Vec<Arc<Mutex<VecDeque<String>>>> = (0..4)
    .map(|_| Arc::new(Mutex::new(VecDeque::new())))
    .collect();
```

### 4. Atomic Operations and Lock-Free Structures

Demonstrates atomic operation memory tracking:

- **Atomic counters** - Thread-safe counting
- **Atomic flags** - Inter-thread state synchronization
- **Operation history** - Record each atomic operation

```rust
let atomic_counter = Arc<AtomicUsize>;
let atomic_flags = Arc<[AtomicBool; 4]>;
```

## 📈 Performance Analysis Results

### Export Performance Comparison

| Format | Export Time | File Size | Speed Improvement |
|--------|-------------|-----------|-------------------|
| Binary | 211ms | 480KB | Baseline |
| JSON | 17.1s | 728KB | 80.72x slower |

### Memory Usage Statistics

- **Total allocations**: 289
- **Smart pointers**: 20 (Arc, Rc, etc.)
- **Multi-threaded allocations**: 294
- **Complex data structures**: 78

## 🔍 Analysis Report Interpretation

### JSON File Contents

The generated 5 JSON files contain different aspects of data:

1. **memory_analysis.json** - Basic allocation information

   ```json
   {
     "var_name": "main_thread_buffer",
     "type_name": "alloc::vec::Vec<u8>",
     "size": 1024,
     "thread_id": "ThreadId(1)"
   }
   ```

2. **performance.json** - Performance-related data
3. **complex_types.json** - Complex type analysis
4. **unsafe_ffi.json** - Unsafe code tracking
5. **lifetime.json** - Lifecycle information

### HTML Report Features

The interactive report generated with `make html` includes:

- **Memory usage timeline** - Shows memory growth trends
- **Thread analysis** - Memory usage grouped by thread
- **Type distribution** - Memory usage by different data types
- **Variable relationship graph** - Smart pointer reference relationships

## 🛠️ Custom Concurrent Analysis

### Create Your Own Multi-threaded Analysis

```rust
use memscope_rs::{get_global_tracker, track_var, init};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    // 1. Create shared data structures
    let shared_data = Arc::new(Mutex::new(Vec::new()));
    track_var!(shared_data);
    
    // 2. Start multiple threads
    let mut handles = vec![];
    for thread_id in 0..4 {
        let data_clone = Arc::clone(&shared_data);
        
        let handle = thread::spawn(move || {
            // 3. Track local data in each thread
            let local_data = vec![thread_id; 100];
            track_var!(local_data);
            
            // 4. Operate on shared data
            {
                let mut data = data_clone.lock().unwrap();
                data.extend_from_slice(&local_data);
            }
        });
        handles.push(handle);
    }
    
    // 5. Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // 6. Export analysis results
    let tracker = get_global_tracker();
    tracker.export_to_binary("my_concurrent_analysis")?;
    
    println!("✅ Concurrent analysis complete!");
    println!("Run: make html DIR=MemoryAnalysis/my_concurrent_analysis BASE=my_concurrent_analysis");
    
    Ok(())
}
```

### Analyze Specific Concurrent Patterns

```rust
// Analyze Channel communication
use std::sync::mpsc;

let (sender, receiver) = mpsc::channel();
track_var!(sender);
track_var!(receiver);

// Analyze Barrier synchronization
use std::sync::Barrier;

let barrier = Arc::new(Barrier::new(4));
track_var!(barrier);

// Analyze Condvar waiting
use std::sync::Condvar;

let condvar = Arc::new(Condvar::new());
track_var!(condvar);
```

## 🎯 Best Practices

### 1. Tracking Strategy

- **Main thread variables** - Ensure clear variable names
- **Shared data** - Start tracking at creation time
- **Thread-local data** - Track within each thread

### 2. Performance Considerations

- **Use Binary format** - For large amounts of data, Binary is 80x faster than JSON
- **Batch analysis** - Avoid tracking too many variables at once
- **Selective tracking** - Only track critical shared data

### 3. Report Generation

```bash
# Quick view - Use SVG
tracker.export_memory_analysis("quick_view.svg")?;

# Detailed analysis - Use HTML
make html DIR=MemoryAnalysis/your_analysis BASE=your_analysis

# Data processing - Use JSON
tracker.export_to_json("data_analysis")?;
```

## 🔧 Troubleshooting

### Common Issues

1. **Variable names show as "unknown"**
   - Ensure explicitly named variables in main thread
   - Use `track_var!(variable_name)` instead of anonymous expressions

2. **HTML charts display errors**
   - Ensure correct BASE name: `make html BASE=your_actual_base_name`
   - Check if JSON files are generated correctly

3. **Performance issues**
   - Prioritize Binary format export
   - Avoid tracking too many temporary variables

### Debugging Tips

```rust
// Enable verbose logging
std::env::set_var("MEMSCOPE_VERBOSE", "1");

// Enable test mode (more accurate tracking)
std::env::set_var("MEMSCOPE_TEST_MODE", "1");

// Enable accurate tracking (for testing)
std::env::set_var("MEMSCOPE_ACCURATE_TRACKING", "1");
```

## 🔍 Advanced Concurrent Patterns

### Async/Await Memory Tracking

```rust
use tokio;
use memscope_rs::{track_var, init};

#[tokio::main]
async fn main() {
    init();
    
    // Track async data structures
    let async_data = Arc::new(Mutex::new(Vec::new()));
    track_var!(async_data);
    
    // Spawn async tasks
    let mut tasks = vec![];
    for i in 0..4 {
        let data_clone = Arc::clone(&async_data);
        let task = tokio::spawn(async move {
            let local_data = vec![i; 100];
            track_var!(local_data);
            
            // Simulate async work
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            let mut data = data_clone.lock().unwrap();
            data.extend_from_slice(&local_data);
        });
        tasks.push(task);
    }
    
    // Wait for all tasks
    for task in tasks {
        task.await.unwrap();
    }
    
    println!("✅ Async analysis complete!");
}
```

### Lock-Free Data Structures

```rust
use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};
use std::sync::Arc;

fn analyze_lockfree_structures() {
    init();
    
    // Track atomic operations
    let counter = Arc::new(AtomicUsize::new(0));
    track_var!(counter);
    
    let data_ptr = Arc::new(AtomicPtr::new(std::ptr::null_mut()));
    track_var!(data_ptr);
    
    // Simulate lock-free operations
    let handles: Vec<_> = (0..4).map(|_| {
        let counter_clone = Arc::clone(&counter);
        thread::spawn(move || {
            for _ in 0..1000 {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            }
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final counter value: {}", counter.load(Ordering::Relaxed));
}
```

## 📊 Memory Pattern Analysis

### Identifying Memory Hotspots

The HTML report helps identify:

1. **Thread contention** - Threads competing for shared resources
2. **Memory leaks** - Variables that aren't properly cleaned up
3. **Allocation patterns** - Frequent allocation/deallocation cycles
4. **Cache efficiency** - Data locality in multi-threaded access

### Performance Optimization Tips

Based on analysis results:

- **Reduce lock contention** - Use more granular locking
- **Optimize data structures** - Choose appropriate concurrent collections
- **Memory pooling** - Reuse allocations to reduce overhead
- **NUMA awareness** - Consider thread-to-core affinity

## 🎉 Summary

Through this concurrent analysis example, you learned:

✅ **Multi-threaded memory tracking** - Track shared variables and thread-local data  
✅ **Performance optimization** - Use Binary format for 80x speed improvement  
✅ **Complex scenario analysis** - Producer-consumer, work-stealing, atomic operations  
✅ **Interactive reporting** - Generate professional HTML analysis reports  
✅ **Data categorization** - 5 specialized JSON files for deep analysis  

Now you can analyze memory usage patterns in any complex multi-threaded Rust program! 🚀