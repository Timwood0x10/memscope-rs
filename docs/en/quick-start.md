# Quick Start

## Installation

Add memscope-rs to your `Cargo.toml`:

```toml
[dependencies]
memscope-rs = "0.2"
```

## Basic Usage

### 1. Initialize Tracker

```rust
use memscope_rs::{global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    // Initialize global tracker
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // Your code...

    Ok(())
}
```

### 2. Track Variables

```rust
use memscope_rs::track;

fn main() -> memscope_rs::MemScopeResult<()> {
    let tracker = global_tracker()?;

    // Track allocations
    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);

    let string_data = String::from("Hello, world!");
    track!(tracker, string_data);

    Ok(())
}
```

### 3. Analyze Memory Usage

```rust
use memscope_rs::analyzer;

fn main() -> memscope_rs::MemScopeResult<()> {
    let tracker = global_tracker()?;

    let data = vec![1, 2, 3];
    track!(tracker, data);

    // Analyze memory usage
    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    println!("Total allocations: {}", report.stats.allocation_count);
    println!("Total bytes: {}", report.stats.total_bytes);
    println!("Peak memory: {} bytes", report.stats.peak_bytes);

    Ok(())
}
```

### 4. Export Reports

```rust
fn main() -> memscope_rs::MemScopeResult<()> {
    let tracker = global_tracker()?;

    let data = vec![1, 2, 3];
    track!(tracker, data);

    // Export JSON report
    tracker.export_all_json("output")?;

    // Export HTML Dashboard
    tracker.render_unified_dashboard("output/dashboard.html")?;

    Ok(())
}
```

## Smart Pointer Tracking

### Auto Detection

```rust
use std::rc::Rc;
use std::sync::Arc;

fn main() -> memscope_rs::MemScopeResult<()> {
    let tracker = global_tracker()?;

    // Auto-detect smart pointer types
    let rc_data = Rc::new(vec![1, 2, 3]);
    track!(tracker, rc_data);

    let arc_data = Arc::new(vec![4, 5, 6]);
    track!(tracker, arc_data);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    println!("Smart pointers: {}", report.circular_references.total_smart_pointers);
    println!("Circular refs: {}", report.circular_references.count);

    Ok(())
}
```

### Circular Reference Detection

```rust
use std::cell::RefCell;
use std::rc::Rc;

struct Node {
    data: i32,
    next: Option<Rc<RefCell<Node>>>,
}

fn main() -> memscope_rs::MemScopeResult<()> {
    let tracker = global_tracker()?;

    // Create circular reference
    let node1 = Rc::new(RefCell::new(Node { data: 1, next: None }));
    let node2 = Rc::new(RefCell::new(Node { data: 2, next: None }));

    node1.borrow_mut().next = Some(node2.clone());
    node2.borrow_mut().next = Some(node1.clone());

    track!(tracker, node1);
    track!(tracker, node2);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    if report.circular_references.has_cycles {
        println!("Circular references detected!");
        println!("Leaked memory: {} bytes", report.circular_references.total_leaked_memory);
    }

    Ok(())
}
```

## Lifecycle Analysis

### Auto Tracking

```rust
fn main() -> memscope_rs::MemScopeResult<()> {
    let tracker = global_tracker()?;

    {
        let temporary = vec![1, 2, 3];
        track!(tracker, temporary);
        // temporary goes out of scope here, lifecycle auto-recorded
    }

    let long_lived = vec![1; 1000];
    track!(tracker, long_lived);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    // View lifetimes
    for alloc in &report.allocations {
        println!("{}: lifetime_ms = {}", alloc.type_name, alloc.lifetime_ms);
    }

    Ok(())
}
```

## Performance Optimization

### Sampling Mode

```rust
use memscope_rs::{tracker, tracker::SamplingConfig};

fn main() -> memscope_rs::MemScopeResult<()> {
    // Use high-performance sampling mode (1% sample rate)
    let tracker = tracker!()
        .with_sampling(SamplingConfig::high_performance());

    for i in 0..10000 {
        let data = vec![i; 100];
        track!(tracker, data);
    }

    Ok(())
}
```

## Complete Example

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};
use std::rc::Rc;
use std::sync::Arc;

fn main() -> memscope_rs::MemScopeResult<()> {
    // 1. Initialize
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // 2. Track various types
    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);

    let string_data = String::from("Hello");
    track!(tracker, string_data);

    let rc_data = Rc::new(vec![1.0, 2.0, 3.0]);
    track!(tracker, rc_data);

    let arc_data = Arc::new(vec![4.0, 5.0, 6.0]);
    track!(tracker, arc_data);

    // 3. Analyze
    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    println!("=== Memory Analysis Report ===");
    println!("Total allocations: {}", report.stats.allocation_count);
    println!("Total bytes: {}", report.stats.total_bytes);
    println!("Peak memory: {} bytes", report.stats.peak_bytes);
    println!("Threads: {}", report.stats.thread_count);
    println!("\n=== Smart Pointers ===");
    println!("Smart pointers: {}", report.circular_references.total_smart_pointers);
    println!("Circular refs: {}", report.circular_references.count);

    // 4. Export
    tracker.export_all_json("output")?;
    tracker.render_unified_dashboard("output/dashboard.html")?;

    println!("\nReports exported to output/ directory");

    Ok(())
}
```

## Next Steps

- Read [Compile-time Semantic Enhancement](./compile-time-enhancement.md) for detailed features
- Read [Smart Pointer Tracking](./smart-pointer-tracking.md) for circular reference detection
- Read [API Documentation](./api.md) for complete API reference
