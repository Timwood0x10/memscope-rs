# 5-Minute Quick Start

This guide will help you start using memscope-rs for memory tracking and analysis in 5 minutes.

## 1. Add Dependency (30 seconds)

Add to your `Cargo.toml`:

```toml
[dependencies]
memscope-rs = "0.1.4"
```

## 2. Basic Usage (2 minutes)

Create a simple example:

```rust
use memscope_rs::{track_var, get_global_tracker, init};
use std::rc::Rc;

fn main() {
    // Initialize memory tracking
    init();
    
    // Create and track variables
    let my_vec = vec![1, 2, 3, 4, 5];
    track_var!(my_vec);  // Zero-cost tracking
    
    let my_string = String::from("Hello, memscope!");
    track_var!(my_string);
    
    let boxed_data = Box::new(42);
    track_var!(boxed_data);
    
    // Smart pointer tracking
    let rc_data = Rc::new(vec![10, 20, 30]);
    track_var!(rc_data);
    
    // Variables can still be used normally
    println!("Vector: {:?}", my_vec);
    println!("String: {}", my_string);
    println!("Boxed: {}", *boxed_data);
    println!("RC data: {:?}", *rc_data);
    
    // Get memory statistics
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("Active allocations: {}", stats.active_allocations);
        println!("Active memory: {} bytes", stats.active_memory);
        println!("Total allocations: {}", stats.total_allocations);
        println!("Peak memory: {} bytes", stats.peak_memory);
    }
}
```

## 3. Generate Analysis Reports (2 minutes)

Add export functionality:

```rust
use memscope_rs::{track_var, get_global_tracker, init};
use std::rc::Rc;

fn main() {
    init();
    
    // Track various types of data
    let numbers = vec![1, 2, 3, 4, 5];
    track_var!(numbers);
    
    let shared_data = Rc::new(vec!["a", "b", "c"]);
    track_var!(shared_data);
    
    let shared_clone = Rc::clone(&shared_data);
    track_var!(shared_clone);
    
    let tracker = get_global_tracker();
    
    // 1. Export JSON data (5 categorized files)
    if let Err(e) = tracker.export_to_json("my_analysis") {
        eprintln!("JSON export failed: {}", e);
    } else {
        println!("✅ JSON export successful: MemoryAnalysis/my_analysis/");
    }
    
    // 2. Export SVG visualization
    if let Err(e) = tracker.export_memory_analysis("my_analysis.svg") {
        eprintln!("SVG export failed: {}", e);
    } else {
        println!("✅ SVG export successful: MemoryAnalysis/my_analysis/");
    }
    
    // 3. Export HTML interactive dashboard
    if let Err(e) = tracker.export_interactive_dashboard("my_analysis.html") {
        eprintln!("HTML export failed: {}", e);
    } else {
        println!("✅ HTML export successful: MemoryAnalysis/my_analysis/");
    }
    
    // 4. Export binary format (high performance)
    if let Err(e) = tracker.export_to_binary("my_analysis") {
        eprintln!("Binary export failed: {}", e);
    } else {
        println!("✅ Binary export successful: MemoryAnalysis/my_analysis/");
    }
}
```

## 4. View Results (30 seconds)

After running the program, check generated files:

```bash
# Run the program
cargo run

# Check generated files
ls MemoryAnalysis/my_analysis/
# You'll see:
# - my_analysis_memory_analysis.json  (memory analysis data)
# - my_analysis_lifetime.json         (lifecycle data)
# - my_analysis_performance.json      (performance data)
# - my_analysis_unsafe_ffi.json       (unsafe/FFI data)
# - my_analysis_complex_types.json    (complex types data)
# - my_analysis.svg                   (memory usage chart)
# - my_analysis.html                  (interactive dashboard)
# - my_analysis.memscope              (binary format)
```

### Generate Enhanced Reports with make html

```bash
# Use make command to generate richer HTML reports
make html DIR=MemoryAnalysis/my_analysis BASE=my_analysis

# Open the generated report
open memory_report.html  # macOS
# Or open memory_report.html in your browser
```

## 🎯 What You Just Learned

✅ **Zero-cost tracking**: `track_var!` macro doesn't affect program performance  
✅ **Multiple data types**: Vec, String, Box, Rc, Arc, etc. can all be tracked  
✅ **Real-time statistics**: Get current memory usage and peak values  
✅ **Multiple export formats**: JSON data, SVG charts, HTML dashboard, binary format  
✅ **Categorized data**: 5 specialized JSON files for analyzing different aspects  
✅ **Variables remain usable**: Variables work completely normally after tracking  
✅ **High-performance binary**: Export format 80x faster than JSON  

## 🚀 Next Steps

Now that you've mastered the basics, continue learning:

- **[Basic Tracking](basic-tracking.md)** - Deep dive into the three tracking macros
- **[First Analysis](first-analysis.md)** - Learn to interpret analysis reports
- **[Tracking Macros Guide](../user-guide/tracking-macros.md)** - Choose the best tracking method

## 💡 Quick Tips

- **Performance**: `track_var!` is zero-cost, safe for production use
- **Smart pointers**: Rc/Arc automatically track reference count changes
- **File location**: All export files are in the `MemoryAnalysis/` directory
- **HTML reports**: Include clickable charts and filtering features
- **Binary format**: Uses `.memscope` extension, convertible to JSON or HTML
- **Make commands**: Use `make html` to generate enhanced interactive reports
- **Multi-threading**: Supports memory tracking and analysis for multi-threaded programs

## 🔥 Advanced Examples

Want to see more complex usage? Run these examples:

```bash
# Basic usage example
cargo run --example basic_usage

# Binary export example
cargo run --example binary_export_demo

# Advanced multi-threaded example
cargo run --example advanced_metrics_demo

# Then generate HTML reports
make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo
```

Start enjoying efficient memory analysis! 🎉