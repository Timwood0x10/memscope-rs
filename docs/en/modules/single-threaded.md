# Single-threaded Module: Zero-overhead Memory Tracking

The single-threaded module is the **recommended starting point** for most applications. It provides precise, zero-overhead memory tracking using the `track_var!` family of macros.

## 🎯 When to Use

**✅ Perfect for:**
- Development and debugging
- Single-threaded applications
- Applications with < 10 threads
- When you need precise tracking data
- Learning and experimenting with memscope-rs

**❌ Consider alternatives for:**
- High-concurrency applications (20+ threads)
- Performance-critical production systems
- When approximate data is sufficient

## 🧩 Core Tracking Macros

The single-threaded module provides three specialized tracking macros:

### 1. `track_var!` - **[RECOMMENDED]**

Zero-cost tracking by reference. The variable remains fully usable.

```rust
use memscope_rs::{init, track_var, get_global_tracker};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    // Create and track variables
    let my_vec = vec![1, 2, 3, 4, 5];
    track_var!(my_vec);  // Zero-cost tracking
    
    let my_string = String::from("Hello, memscope!");
    track_var!(my_string);
    
    let my_box = Box::new(42);
    track_var!(my_box);
    
    // Variables work normally - tracking is invisible
    println!("Vector: {:?}", my_vec);
    println!("String: {}", my_string);
    println!("Box: {}", *my_box);
    
    // Export analysis
    let tracker = get_global_tracker();
    tracker.export_to_json("analysis")?;
    tracker.export_to_html("analysis.html")?;
    
    Ok(())
}
```

**Performance:** Truly zero-overhead - no cloning, no wrappers, no ownership changes.

### 2. `track_var_smart!` - **[INTELLIGENT]**

Automatically chooses the best tracking strategy based on type:

```rust
use memscope_rs::{track_var_smart, init};
use std::rc::Rc;

fn main() {
    init();
    
    // Copy types - copied automatically (cheap)
    let number = 42i32;
    track_var_smart!(number);
    
    // Non-copy types - tracked by reference (zero cost)
    let text = String::from("Hello");
    track_var_smart!(text);
    
    // Smart pointers - clones the pointer (cheap reference increment)
    let rc_data = Rc::new(vec![1, 2, 3]);
    track_var_smart!(rc_data);
    
    // All variables remain fully usable!
    println!("{}, {}, {:?}", number, text, rc_data);
}
```

**Intelligence:**
- `Copy` types (i32, f64, bool): Creates copy
- Non-`Copy` types: Reference tracking
- Smart pointers (Rc, Arc): Clones pointer

### 3. `track_var_owned!` - **[ADVANCED]**

Full lifecycle management with ownership transfer:

```rust
use memscope_rs::{track_var_owned, init};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    let data = vec![1, 2, 3, 4, 5];
    let tracked = track_var_owned!(data);  // Takes ownership
    
    // Access through wrapper methods
    println!("Length: {}", tracked.len());
    println!("First: {}", tracked[0]);
    
    // Extract the original value when needed
    let original = tracked.into_inner();
    println!("Extracted: {:?}", original);
    
    Ok(())
}
```

**Features:**
- Precise lifecycle tracking
- Automatic cleanup detection
- Drop protection against duplicates
- Smart pointer detection

## 📊 Smart Pointer Support

All tracking macros have special handling for Rust's smart pointers:

```rust
use memscope_rs::{track_var, init};
use std::rc::Rc;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    // Reference counted pointers
    let rc_data = Rc::new(vec![1, 2, 3]);
    track_var!(rc_data);
    
    // Cloning operations are tracked
    let rc_clone = Rc::clone(&rc_data);
    track_var!(rc_clone);
    
    // Atomic reference counted (thread-safe)
    let arc_data = Arc::new(String::from("shared"));
    track_var!(arc_data);
    
    // Heap allocated
    let boxed = Box::new(42);
    track_var!(boxed);
    
    // Export with smart pointer analysis
    let tracker = get_global_tracker();
    tracker.export_to_json("smart_pointers")?;
    
    Ok(())
}
```

## 🔧 Export and Analysis

### JSON Export - Detailed Analysis

```rust
use memscope_rs::{get_global_tracker, ExportOptions};

fn export_detailed_analysis() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = get_global_tracker();
    
    // Basic export
    tracker.export_to_json("basic_analysis")?;
    
    // Configured export
    let options = ExportOptions::new()
        .include_system_allocations(false)  // Skip system allocations
        .verbose_logging(true)              // Detailed logging
        .buffer_size(128 * 1024);           // 128KB buffer
    
    tracker.export_to_json_with_options("detailed_analysis", options)?;
    
    // Optimized export (best performance)
    let result = tracker.export_to_json_optimized("optimized_analysis")?;
    println!("Export completed in {:.2}ms", result.export_stats.export_time_ms);
    
    Ok(())
}
```

### HTML Dashboard - Interactive Visualization

```rust
use memscope_rs::get_global_tracker;

fn generate_html_dashboard() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = get_global_tracker();
    
    // Generate interactive HTML dashboard
    tracker.export_to_html("memory_dashboard.html")?;
    
    println!("📊 Interactive dashboard generated: memory_dashboard.html");
    println!("   - Memory timeline charts");
    println!("   - Variable lifecycle analysis");
    println!("   - Smart pointer reference tracking");
    println!("   - Memory leak detection");
    
    Ok(())
}
```

## ⚡ Performance Characteristics

### Tracking Overhead

| Macro | Overhead | Use Case |
|-------|----------|----------|
| `track_var!` | **Zero** | Production recommended |
| `track_var_smart!` | **Minimal** | Mixed types |
| `track_var_owned!` | **Wrapper** | Precise analysis |

### Export Performance (Real Data)

Based on tracking 1000+ variables in actual test:

| Format | Export Time | File Size | Features |
|--------|-------------|-----------|----------|
| **JSON** | 1.3s | 1.2MB | Detailed analysis, readable |
| **HTML** | 800ms | 2.1MB | Interactive dashboard |
| **Binary** | 211ms | 480KB | High performance |

## 🛡️ Safety Features

### Automatic Type Detection

```rust
use memscope_rs::track_var;

fn test_type_detection() {
    // Primitive types
    let number = 42i32;
    track_var!(number);  // Synthetic pointer generated
    
    // Heap-allocated types
    let vector = vec![1, 2, 3];
    track_var!(vector);  // Real heap pointer used
    
    // Smart pointers
    let rc = Rc::new(vector);
    track_var!(rc);      // Smart pointer tracking
}
```

### Error Handling

```rust
use memscope_rs::{get_global_tracker, TrackingResult};

fn robust_tracking() -> TrackingResult<()> {
    let tracker = get_global_tracker();
    
    // Fast mode for testing
    tracker.enable_fast_mode();
    
    let data = vec![1, 2, 3];
    track_var!(data);
    
    // Export with error handling
    match tracker.export_to_json("analysis") {
        Ok(_) => println!("✅ Export successful"),
        Err(e) => eprintln!("❌ Export failed: {}", e),
    }
    
    Ok(())
}
```

## 🎮 Complete Example

```rust
use memscope_rs::{init, track_var, track_var_smart, track_var_owned, get_global_tracker};
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracking
    init();
    
    // Different tracking strategies
    let basic_data = vec![1, 2, 3, 4, 5];
    track_var!(basic_data);  // Zero-cost reference tracking
    
    let smart_data = String::from("Hello, World!");
    track_var_smart!(smart_data);  // Intelligent tracking
    
    let owned_data = vec![10, 20, 30];
    let tracked = track_var_owned!(owned_data);  // Full lifecycle
    
    // Smart pointer tracking
    let rc_data = Rc::new(vec![100, 200, 300]);
    track_var!(rc_data);
    let rc_clone = Rc::clone(&rc_data);
    track_var!(rc_clone);
    
    // Use all variables normally
    println!("Basic: {:?}", basic_data);
    println!("Smart: {}", smart_data);
    println!("Tracked: {:?}", *tracked);
    println!("RC count: {}", Rc::strong_count(&rc_data));
    
    // Export comprehensive analysis
    let tracker = get_global_tracker();
    tracker.export_to_json("comprehensive_analysis")?;
    tracker.export_to_html("dashboard.html")?;
    
    println!("🎯 Analysis complete!");
    println!("📁 JSON: comprehensive_analysis.json");
    println!("📊 Dashboard: dashboard.html");
    
    Ok(())
}
```

## 🔗 Next Steps

- **[Multi-threaded Module](multithread.md)** - High-concurrency tracking
- **[Async Module](async.md)** - Task-centric analysis
- **[API Reference](api-reference/tracking-api.md)** - Complete API documentation
- **[Examples](examples/basic-usage.md)** - More detailed examples