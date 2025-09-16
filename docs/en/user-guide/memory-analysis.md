# Memory Analysis Guide

memscope-rs provides powerful memory analysis capabilities to help you understand program memory usage patterns, identify potential issues, and optimize performance.

## 🎯 Analysis Overview

### Core Analysis Capabilities

| Analysis Type | Function | Use Case | Output Format |
|---------------|----------|----------|---------------|
| **Basic Statistics** | Memory usage, allocation counts | Daily monitoring | Real-time data |
| **Lifecycle Analysis** | Object creation to destruction tracking | Memory leak detection | Timeline charts |
| **Type Analysis** | Memory usage by data type | Structure optimization | Pie/bar charts |
| **Reference Analysis** | Smart pointer reference counting | Circular reference detection | Relationship graphs |
| **Hotspot Analysis** | High-frequency allocation locations | Performance optimization | Heat maps |

## 📊 Basic Statistics Analysis

### Real-time Memory Statistics

```rust
use memscope_rs::{get_global_tracker, track_var, init};

fn basic_statistics_demo() {
    init();
    
    // Create test data
    let data1 = vec![1; 1000];
    track_var!(data1);
    
    let data2 = String::from("Hello, Analysis!");
    track_var!(data2);
    
    let data3 = Box::new(vec![0u8; 2048]);
    track_var!(data3);
    
    // Get detailed statistics
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("📈 Memory Statistics Report:");
        println!("  Active allocations: {}", stats.active_allocations);
        println!("  Active memory: {} bytes ({:.2} KB)", 
                stats.active_memory, 
                stats.active_memory as f64 / 1024.0);
        println!("  Total allocations: {}", stats.total_allocations);
        println!("  Total deallocations: {}", stats.total_deallocations);
        println!("  Peak memory: {} bytes ({:.2} KB)", 
                stats.peak_memory,
                stats.peak_memory as f64 / 1024.0);
        println!("  Average allocation size: {:.2} bytes", 
                stats.active_memory as f64 / stats.active_allocations as f64);
    }
}
```

### Memory Efficiency Analysis

```rust
use memscope_rs::{get_global_tracker, track_var, init};

fn memory_efficiency_analysis() {
    init();
    let tracker = get_global_tracker();
    
    // Record baseline
    let baseline = tracker.get_stats().unwrap();
    
    // Simulate different memory usage patterns
    
    // Pattern 1: Large contiguous allocations
    println!("🔍 Analyzing Pattern 1: Large contiguous allocations");
    {
        let large_blocks: Vec<Vec<u8>> = (0..10)
            .map(|i| {
                let block = vec![i as u8; 10240]; // 10KB each
                track_var!(block);
                block
            })
            .collect();
        
        let after_large = tracker.get_stats().unwrap();
        println!("  Large allocation efficiency: {:.2}%", 
                calculate_efficiency(&baseline, &after_large));
    }
    
    // Pattern 2: Small frequent allocations
    println!("🔍 Analyzing Pattern 2: Small frequent allocations");
    {
        let small_blocks: Vec<Vec<u8>> = (0..1000)
            .map(|i| {
                let block = vec![i as u8; 100]; // 100B each
                track_var!(block);
                block
            })
            .collect();
        
        let after_small = tracker.get_stats().unwrap();
        println!("  Small allocation efficiency: {:.2}%", 
                calculate_efficiency(&baseline, &after_small));
    }
}

fn calculate_efficiency(baseline: &memscope_rs::MemoryStats, current: &memscope_rs::MemoryStats) -> f64 {
    let allocated_memory = current.active_memory - baseline.active_memory;
    let allocation_count = current.active_allocations - baseline.active_allocations;
    
    if allocation_count == 0 {
        return 100.0;
    }
    
    let average_size = allocated_memory as f64 / allocation_count as f64;
    let efficiency = (average_size / 1024.0).min(1.0) * 100.0; // Assume 1KB as ideal size
    efficiency
}
```

## 🔄 Lifecycle Analysis

### Object Lifecycle Tracking

```rust
use memscope_rs::{track_var_owned, get_global_tracker, init};
use std::rc::Rc;

fn lifecycle_analysis_demo() {
    init();
    println!("🔄 Object Lifecycle Analysis");
    
    // Phase 1: Creation phase
    println!("  📦 Phase 1: Object creation");
    let creation_time = std::time::Instant::now();
    
    let long_lived_data = track_var_owned!(vec![1; 5000]);
    let short_lived_data = track_var_owned!(String::from("temporary data"));
    
    println!("    Creation time: {:?}", creation_time.elapsed());
    
    // Phase 2: Usage phase
    println!("  🔧 Phase 2: Object usage");
    let usage_time = std::time::Instant::now();
    
    // Simulate data usage
    let _sum: i32 = long_lived_data.iter().sum();
    let _length = short_lived_data.len();
    
    println!("    Usage time: {:?}", usage_time.elapsed());
    
    // Phase 3: Partial cleanup
    println!("  🧹 Phase 3: Partial cleanup");
    drop(short_lived_data); // Explicitly release short-term data
    
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("    Active memory after cleanup: {} bytes", stats.active_memory);
    }
    
    // Phase 4: Complete cleanup
    println!("  🗑️ Phase 4: Complete cleanup");
    drop(long_lived_data);
    
    if let Ok(stats) = tracker.get_stats() {
        println!("    Final active memory: {} bytes", stats.active_memory);
    }
}
```

### Smart Pointer Lifecycle

```rust
use memscope_rs::{track_var, get_global_tracker, init};
use std::rc::Rc;
use std::sync::Arc;

fn smart_pointer_lifecycle() {
    init();
    println!("🔗 Smart Pointer Lifecycle Analysis");
    
    // Rc reference counting analysis
    println!("  📊 Rc Reference Counting Analysis:");
    {
        let original = Rc::new(vec![1, 2, 3, 4, 5]);
        track_var!(original);
        println!("    Initial reference count: {}", Rc::strong_count(&original));
        
        let clone1 = Rc::clone(&original);
        track_var!(clone1);
        println!("    After first clone: {}", Rc::strong_count(&original));
        
        let clone2 = Rc::clone(&original);
        track_var!(clone2);
        println!("    After second clone: {}", Rc::strong_count(&original));
        
        // Analyze memory usage
        let tracker = get_global_tracker();
        if let Ok(stats) = tracker.get_stats() {
            println!("    Current active allocations: {}", stats.active_allocations);
        }
        
        drop(clone1);
        println!("    After dropping clone1: {}", Rc::strong_count(&original));
        
        drop(clone2);
        println!("    After dropping clone2: {}", Rc::strong_count(&original));
    }
    
    // Arc thread-safe analysis
    println!("  🧵 Arc Thread-Safe Analysis:");
    {
        let shared_data = Arc::new(vec![1; 1000]);
        track_var!(shared_data);
        
        let handles: Vec<_> = (0..3)
            .map(|i| {
                let data_clone = Arc::clone(&shared_data);
                track_var!(data_clone);
                
                std::thread::spawn(move || {
                    println!("    Thread {} accessing data length: {}", i, data_clone.len());
                    std::thread::sleep(std::time::Duration::from_millis(100));
                })
            })
            .collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        println!("    Final reference count: {}", Arc::strong_count(&shared_data));
    }
}
```

## 📈 Type Analysis

### Data Type Memory Usage Analysis

```rust
use memscope_rs::{track_var, get_global_tracker, init};
use std::collections::{HashMap, BTreeMap, HashSet};

fn type_analysis_demo() {
    init();
    println!("📊 Data Type Memory Analysis");
    
    let tracker = get_global_tracker();
    let baseline = tracker.get_stats().unwrap();
    
    // String type analysis
    println!("  📝 String Types:");
    let string_data = String::from("This is a test string for analyzing memory usage");
    track_var!(string_data);
    let after_string = tracker.get_stats().unwrap();
    println!("    String memory growth: {} bytes", 
            after_string.active_memory - baseline.active_memory);
    
    // Vector type analysis
    println!("  📋 Vector Types:");
    let vec_data = vec![1i32; 1000];
    track_var!(vec_data);
    let after_vec = tracker.get_stats().unwrap();
    println!("    Vec<i32> memory growth: {} bytes", 
            after_vec.active_memory - after_string.active_memory);
    
    // HashMap analysis
    println!("  🗂️ HashMap Types:");
    let mut map_data = HashMap::new();
    for i in 0..100 {
        map_data.insert(format!("key_{}", i), i);
    }
    track_var!(map_data);
    let after_map = tracker.get_stats().unwrap();
    println!("    HashMap memory growth: {} bytes", 
            after_map.active_memory - after_vec.active_memory);
    
    // BTreeMap comparison analysis
    println!("  🌳 BTreeMap Types:");
    let mut btree_data = BTreeMap::new();
    for i in 0..100 {
        btree_data.insert(format!("key_{}", i), i);
    }
    track_var!(btree_data);
    let after_btree = tracker.get_stats().unwrap();
    println!("    BTreeMap memory growth: {} bytes", 
            after_btree.active_memory - after_map.active_memory);
    
    // Generate type analysis report
    generate_type_analysis_report();
}

fn generate_type_analysis_report() {
    let tracker = get_global_tracker();
    
    // Export detailed type analysis
    if let Err(e) = tracker.export_to_json("type_analysis") {
        eprintln!("Type analysis export failed: {}", e);
    } else {
        println!("  ✅ Type analysis report generated: MemoryAnalysis/type_analysis/");
    }
}
```

## 🔍 Hotspot Analysis

### Memory Allocation Hotspot Identification

```rust
use memscope_rs::{track_var, get_global_tracker, init};

fn hotspot_analysis_demo() {
    init();
    println!("🔥 Memory Allocation Hotspot Analysis");
    
    // Simulate different allocation patterns
    
    // Hotspot 1: Loop allocations
    println!("  🔄 Hotspot 1: Loop allocations");
    for i in 0..100 {
        let data = vec![i; 50];
        track_var!(data);
        
        // Simulate processing time
        if i % 10 == 0 {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    
    // Hotspot 2: Recursive allocations
    println!("  🌀 Hotspot 2: Recursive allocations");
    recursive_allocation(5, 100);
    
    // Hotspot 3: Conditional allocations
    println!("  🎯 Hotspot 3: Conditional allocations");
    for i in 0..50 {
        if i % 3 == 0 {
            let large_data = vec![0u8; 1024];
            track_var!(large_data);
        } else {
            let small_data = vec![i as u8; 10];
            track_var!(small_data);
        }
    }
    
    // Generate hotspot analysis report
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("  📊 Hotspot Analysis Results:");
        println!("    Total allocations: {}", stats.total_allocations);
        println!("    Current active allocations: {}", stats.active_allocations);
        println!("    Allocation efficiency: {:.2}%", 
                (stats.active_allocations as f64 / stats.total_allocations as f64) * 100.0);
    }
    
    // Export hotspot analysis
    if let Err(e) = tracker.export_to_html("hotspot_analysis.html") {
        eprintln!("Hotspot analysis export failed: {}", e);
    } else {
        println!("  ✅ Hotspot analysis report: MemoryAnalysis/hotspot_analysis/");
    }
}

fn recursive_allocation(depth: usize, size: usize) {
    if depth == 0 {
        return;
    }
    
    let data = vec![depth; size];
    track_var!(data);
    
    recursive_allocation(depth - 1, size / 2);
}
```

## 🔗 Reference Relationship Analysis

### Circular Reference Detection

```rust
use memscope_rs::{track_var, get_global_tracker, init};
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct Node {
    value: i32,
    children: RefCell<Vec<Rc<Node>>>,
    parent: RefCell<Weak<Node>>,
}

impl Node {
    fn new(value: i32) -> Rc<Self> {
        Rc::new(Node {
            value,
            children: RefCell::new(Vec::new()),
            parent: RefCell::new(Weak::new()),
        })
    }
    
    fn add_child(&self, child: Rc<Node>) {
        child.parent.borrow_mut().clone_from(&Rc::downgrade(&Rc::new(self.clone())));
        self.children.borrow_mut().push(child);
    }
}

fn reference_analysis_demo() {
    init();
    println!("🔗 Reference Relationship Analysis");
    
    let tracker = get_global_tracker();
    let baseline = tracker.get_stats().unwrap();
    
    // Create node tree structure
    println!("  🌳 Creating node tree:");
    let root = Node::new(0);
    track_var!(root);
    
    let child1 = Node::new(1);
    track_var!(child1);
    
    let child2 = Node::new(2);
    track_var!(child2);
    
    let grandchild = Node::new(3);
    track_var!(grandchild);
    
    // Establish parent-child relationships
    root.add_child(child1.clone());
    root.add_child(child2.clone());
    child1.add_child(grandchild.clone());
    
    let after_tree = tracker.get_stats().unwrap();
    println!("    Tree structure memory usage: {} bytes", 
            after_tree.active_memory - baseline.active_memory);
    
    // Analyze reference counts
    println!("  📊 Reference Count Analysis:");
    println!("    root reference count: {}", Rc::strong_count(&root));
    println!("    child1 reference count: {}", Rc::strong_count(&child1));
    println!("    child2 reference count: {}", Rc::strong_count(&child2));
    println!("    grandchild reference count: {}", Rc::strong_count(&grandchild));
    
    // Detect potential cycles
    detect_potential_cycles(&root);
    
    // Cleanup analysis
    drop(grandchild);
    drop(child2);
    drop(child1);
    drop(root);
    
    let after_cleanup = tracker.get_stats().unwrap();
    println!("  🧹 Memory after cleanup: {} bytes", 
            after_cleanup.active_memory - baseline.active_memory);
}

fn detect_potential_cycles(node: &Rc<Node>) {
    println!("  🔍 Circular Reference Detection:");
    
    // Simple circular reference detection logic
    let strong_refs = Rc::strong_count(node);
    let weak_refs = Rc::weak_count(node);
    
    println!("    Node {} - Strong refs: {}, Weak refs: {}", 
            node.value, strong_refs, weak_refs);
    
    if strong_refs > 2 {
        println!("    ⚠️ Warning: Node {} may have circular references", node.value);
    }
    
    // Recursively check child nodes
    for child in node.children.borrow().iter() {
        detect_potential_cycles(child);
    }
}
```

## 📊 Advanced Analysis Techniques

### Memory Usage Pattern Recognition

```rust
use memscope_rs::{get_global_tracker, track_var, init};
use std::time::{Duration, Instant};

fn pattern_analysis_demo() {
    init();
    println!("🎯 Memory Usage Pattern Recognition");
    
    let tracker = get_global_tracker();
    
    // Pattern 1: Steady growth pattern
    println!("  📈 Pattern 1: Steady growth");
    analyze_steady_growth_pattern();
    
    // Pattern 2: Fluctuation pattern
    println!("  🌊 Pattern 2: Fluctuation usage");
    analyze_fluctuation_pattern();
    
    // Pattern 3: Peak pattern
    println!("  ⛰️ Pattern 3: Peak usage");
    analyze_peak_pattern();
    
    // Generate pattern analysis report
    if let Err(e) = tracker.export_to_html("pattern_analysis.html") {
        eprintln!("Pattern analysis export failed: {}", e);
    } else {
        println!("  ✅ Pattern analysis report: MemoryAnalysis/pattern_analysis/");
    }
}

fn analyze_steady_growth_pattern() {
    let mut data_store = Vec::new();
    
    for i in 0..20 {
        let data = vec![i; 100 * (i + 1)]; // Gradually increasing
        track_var!(data);
        data_store.push(data);
        
        std::thread::sleep(Duration::from_millis(50));
    }
    
    println!("    Steady growth pattern complete");
}

fn analyze_fluctuation_pattern() {
    for i in 0..30 {
        let size = if i % 2 == 0 { 1000 } else { 100 };
        let data = vec![i; size];
        track_var!(data);
        
        std::thread::sleep(Duration::from_millis(30));
    }
    
    println!("    Fluctuation pattern complete");
}

fn analyze_peak_pattern() {
    // Normal usage
    for i in 0..10 {
        let data = vec![i; 100];
        track_var!(data);
    }
    
    // Sudden large allocation (peak)
    let peak_data = vec![0u8; 50000];
    track_var!(peak_data);
    
    // Return to normal usage
    for i in 10..20 {
        let data = vec![i; 100];
        track_var!(data);
    }
    
    println!("    Peak pattern complete");
}
```

## 🚀 Analysis Results Application

### Optimization Recommendations Based on Analysis

```rust
use memscope_rs::{get_global_tracker, MemoryStats};

fn generate_optimization_suggestions(stats: &MemoryStats) {
    println!("🎯 Optimization Recommendations:");
    
    // Memory usage efficiency analysis
    let efficiency = (stats.active_memory as f64 / stats.peak_memory as f64) * 100.0;
    
    if efficiency < 70.0 {
        println!("  ⚠️ Low memory usage efficiency ({:.1}%)", efficiency);
        println!("     Suggestion: Consider using object pools or reducing temporary allocations");
    }
    
    // Allocation frequency analysis
    let avg_allocation_size = stats.active_memory as f64 / stats.active_allocations as f64;
    
    if avg_allocation_size < 100.0 {
        println!("  ⚠️ Small average allocation size ({:.1} bytes)", avg_allocation_size);
        println!("     Suggestion: Consider batch allocations or using larger buffers");
    }
    
    // Memory leak risk assessment
    let deallocation_ratio = stats.total_deallocations as f64 / stats.total_allocations as f64;
    
    if deallocation_ratio < 0.8 {
        println!("  🚨 Potential memory leak risk (deallocation rate: {:.1}%)", deallocation_ratio * 100.0);
        println!("     Suggestion: Check long-lived objects and circular references");
    }
    
    // Performance optimization suggestions
    if stats.total_allocations > 10000 {
        println!("  🏃 High-frequency allocations detected ({} times)", stats.total_allocations);
        println!("     Suggestion: Consider pre-allocation or using stack allocation");
    }
}

fn comprehensive_analysis_example() {
    init();
    let tracker = get_global_tracker();
    
    // Run some memory operations
    simulate_application_workload();
    
    // Get final statistics
    if let Ok(stats) = tracker.get_stats() {
        println!("\n📊 Comprehensive Analysis Report:");
        println!("  Active allocations: {}", stats.active_allocations);
        println!("  Active memory: {} KB", stats.active_memory / 1024);
        println!("  Peak memory: {} KB", stats.peak_memory / 1024);
        println!("  Total allocations: {}", stats.total_allocations);
        
        // Generate optimization suggestions
        generate_optimization_suggestions(&stats);
    }
    
    // Export complete analysis report
    let _ = tracker.export_to_html("comprehensive_analysis.html");
    println!("\n✅ Complete analysis report generated!");
}

fn simulate_application_workload() {
    // Simulate real application memory usage patterns
    for _ in 0..100 {
        let data = vec![0u8; 1024];
        track_var!(data);
    }
}
```

## 🚀 Next Steps

Now that you've mastered memscope-rs advanced analysis features, continue learning:

- **[Export Formats Guide](export-formats.md)** - Choose the best report format
- **[CLI Tools](cli-tools.md)** - Use command line for batch analysis
- **[Performance Optimization Guide](../advanced/performance-optimization.md)** - Systematic optimization methods

## 💡 Key Points

- **Multi-dimensional analysis** - Combine statistics, lifecycle, type, and other perspectives
- **Pattern recognition** - Identify common memory usage patterns and issues
- **Real data** - Continuously track memory state during program execution
- **Visual reports** - Use charts and dashboards to intuitively display analysis results
- **Optimization guidance** - Generate targeted optimization recommendations based on analysis results

Master these analysis techniques to make your Rust programs more memory-efficient! 🎯