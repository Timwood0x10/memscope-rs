# memscope-rs - Rust Memory Tracking & Analysis Library

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/memscope-rs.svg)](https://crates.io/crates/memscope-rs)


## What is this thing?

memscope-rs is a Rust library for tracking memory allocations and generating analysis reports. Think of it as a friendly neighborhood memory detective 🕵️ that helps you understand what your variables are up to when you're not looking.

It provides simple macros for variable tracking and exports data in JSON and SVG formats. Perfect for those "wait, where did all my memory go?" moments.

## Core Features

### 1. Variable Tracking
- **Non-intrusive tracking**: Use `track_var!` macro to track variables without breaking your existing code (we promise!)
- **Smart pointer support**: Full support for `Rc<T>`, `Arc<T>`, `Box<T>` - because Rust loves its smart pointers
- **Lifecycle analysis**: Automatic recording of variable lifecycles from birth to... well, drop
- **Reference count monitoring**: Real-time tracking of smart pointer reference count changes (watch those Rc clones!)

### 2. Memory Analysis
- **Memory leak detection**: Find those sneaky leaks hiding in your code
- **Fragmentation analysis**: Basic heap fragmentation reporting
- **Usage pattern detection**: Simple memory usage pattern recognition
- **Performance issue identification**: Spot memory-related bottlenecks

### 3. Data Export & Interactive Visualization
- **JSON format**: Export detailed memory allocation data for programmatic analysis
- **SVG visualization**: Generate memory usage charts and timelines (pretty pictures!)
- **🎯 HTML Interactive Dashboard**: Full-featured web-based dashboard with clickable charts, filterable data, and real-time analysis
- **Multiple export modes**: Fast mode, detailed mode, and "let the computer decide" mode

### 4. Safety Analysis
- **FFI boundary tracking**: Monitor memory interactions between Rust and C/C++ code
- **Security violation detection**: Identify potential memory safety issues
- **Use-after-free detection**: Catch those "oops, I used it after freeing it" moments

## Available Commands and Tools

### Example Programs
```bash
# Basic usage demonstration
cargo run --example basic_usage

# Ownership patterns demo (prepare to be amazed by Rust's ownership system)
cargo run --example ownership_demo

# Complex lifecycle showcase
cargo run --example complex_lifecycle_showcase

# Memory stress test (warning: may stress your computer too)
cargo run --example memory_stress_test

# Performance test
cargo run --example speed_test

# Unsafe/FFI safety demo (for the brave souls)
cargo run --example unsafe_ffi_demo
```

## Usage Examples

### Basic Usage
```rust
use memscope_rs::{init, track_var, get_global_tracker};

fn main() {
    // Initialize memory tracking (don't forget this, or nothing will work!)
    init();
    
    // Create and track variables
    let my_vec = vec![1, 2, 3, 4, 5];
    track_var!(my_vec);
    
    let my_string = String::from("Hello, memscope!");
    track_var!(my_string);
    
    let my_box = Box::new(42); // The answer to everything
    track_var!(my_box);
    
    // Variables work normally (tracking is invisible, like a good spy)
    println!("Vector: {:?}", my_vec);
    println!("String: {}", my_string);
    println!("Box: {}", *my_box);
    
    // Export analysis results
    let tracker = get_global_tracker();
    if let Err(e) = tracker.export_to_json("my_analysis") {
        eprintln!("Export failed: {} (this shouldn't happen, but computers...)", e);
    }
}
```

### Smart Pointer Tracking
```rust
use std::rc::Rc;
use std::sync::Arc;

// Track reference counted pointers
let rc_data = Rc::new(vec![1, 2, 3]);
track_var!(rc_data);

// Track atomic reference counted pointers (for when you need thread safety)
let arc_data = Arc::new(String::from("shared data"));
track_var!(arc_data);

// Cloning operations are also tracked (watch the ref count go up!)
let rc_clone = Rc::clone(&rc_data);
track_var!(rc_clone);
```

### Export Configuration
```rust
use memscope_rs::ExportOptions;

let options = ExportOptions::new()
    .include_system_allocations(false)  // Fast mode (recommended)
    .verbose_logging(true)              // For when you want ALL the details
    .buffer_size(128 * 1024);           // 128KB buffer (because bigger is better, right?)

if let Err(e) = tracker.export_to_json_with_options("detailed_analysis", options) {
    eprintln!("Export failed: {}", e);
}
```

## Build & Installation

### System Requirements
- **Rust**: 1.70 or later (we're not asking for much)
- **OS**: Linux, macOS, Windows (basically everywhere Rust runs)
- **Memory**: At least 4GB RAM recommended (for analyzing large projects)

### From Source
```bash
# Clone the repository
git clone https://github.com/TimWood0x10/memscope-rs.git
cd memscope-rs

# Build the project (grab a coffee, this might take a moment)
make build 

# Run tests (cross your fingers)
cargo test

# Try an example
make run complex_lifecycle_showcase
├── complex_lifecycle_snapshot_complex_types.json
├── complex_lifecycle_snapshot_lifetime.json
├── complex_lifecycle_snapshot_memory_analysis.json
├── complex_lifecycle_snapshot_performance.json
├── complex_lifecycle_snapshot_security_violations.json
├── complex_lifecycle_snapshot_unsafe_ffi.json


# export to html
make html DIR=MemoryAnalysis/complex_lifecycle BASE=complex_lifecycle OUTPUT=improved_tracking_final_v3.html

open improved_tracking_final_v3.html

# You can view the HTML interface I output in ./imsges/*.html
```

### From Crates.io
```bash
# Add to your project
cargo add memscope-rs

# Or manually add to Cargo.toml
[dependencies]
memscope-rs = "0.1.3"
```

### Feature Flags
```toml
[dependencies]
memscope-rs = { version = "0.1.3" }
```

Available features:
- `backtrace` - Enable stack trace collection (adds overhead, but gives you the full story)
- `derive` - Enable derive macro support (experimental, use at your own risk)
- `tracking-allocator` - Custom allocator support (enabled by default)



## Output File Structure & Interactive Dashboard

After running programs, you'll find analysis results in the `MemoryAnalysis/` directory:

```
MemoryAnalysis/
├── basic_usage/
│   ├── basic_usage_snapshot.json      # JSON memory data (the raw truth)
│   ├── basic_usage_graph.svg          # SVG memory usage chart (the pretty version)
│   ├── memory_timeline.svg            # Memory timeline graph
│   └── dashboard.html                 # 🎯 Interactive dashboard (click all the things!)
├── complex_lifecycle/
│   ├── allocations.json               # Allocation details
│   ├── lifecycle_analysis.json        # Lifecycle analysis
│   ├── performance_metrics.json       # Performance metrics
│   └── security_violations.json       # Security issue reports (hopefully empty)
└── benchmark_results/
    ├── benchmark_results.json         # Benchmark results
    ├── performance_report.md          # Performance report
    └── comparison_charts.svg          # Comparison charts
```

### 🌟 Interactive HTML Dashboard Features

The generated `dashboard.html` provides a rich, interactive experience:

- **📊 Interactive Charts**: Click and zoom on memory usage graphs
- **🔍 Filterable Data Tables**: Search and filter allocations by type, size, or lifetime
- **📈 Real-time Statistics**: Live updating memory metrics and trends
- **🎯 Variable Drill-down**: Click on any variable to see detailed lifecycle information
- **📱 Responsive Design**: Works on desktop, tablet, and mobile browsers
- **🔗 Cross-references**: Navigate between related allocations and smart pointer relationships

**To view the dashboard:**
```bash
# After running your tracked program
open MemoryAnalysis/your_analysis_name/dashboard.html
# Or simply double-click the HTML file in your file manager
```

## Project Highlights

### 1. Non-intrusive Design
- Use macros for tracking without changing your code structure
- Variables work normally after tracking (no weird side effects)
- Selective tracking of key variables instead of global tracking (because sometimes less is more)

### 2. Smart Analysis
- Automatic identification of memory usage patterns and anomalies
- Smart pointer reference count change tracking
- Variable relationship analysis and dependency graph generation

### 3. Diverse Output Formats
- JSON data for programmatic processing and integration
- SVG charts for intuitive visualization
- HTML dashboard for interactive analysis (with actual buttons to click!)

### 4. Performance Optimization
- Fast export mode to reduce performance overhead
- Parallel processing support for large datasets
- Configurable buffer sizes for I/O optimization

### 5. Safety Analysis
- FFI boundary memory safety checks
- Automatic detection of potential security vulnerabilities
- Memory access pattern safety assessment

## Comparison with Other Tools

| Feature | memscope-rs | Valgrind | Heaptrack | jemalloc |
|---------|-------------|----------|-----------|----------|
| **Rust Native** | ✅ | ❌ | ❌ | ⚠️ |
| **Variable Names** | ✅ | ❌ | ❌ | ❌ |
| **Smart Pointer Analysis** | ✅ | ⚠️ | ⚠️ | ❌ |
| **Visual Reports** | ✅ | ⚠️ | ✅ | ❌ |
| **Production Ready** | ⚠️ | ✅ | ✅ | ✅ |
| **Interactive Timeline** | ✅ | ❌ | ⚠️ | ❌ |
| **Real-time Tracking** | ⚠️ | ✅ | ✅ | ✅ |
| **Low Overhead** | ⚠️ | ⚠️ | ✅ | ✅ |
| **Mature Ecosystem** | ❌ | ✅ | ✅ | ✅ |

### Honest Assessment

**memscope-rs (this project)**
- ✅ **Strengths**: Rust native, variable name tracking, smart pointer analysis, interactive visualization
- ⚠️ **Current status**: Experimental tool, good for development debugging, noticeable performance overhead
- ❌ **Limitations**: Not mature enough, not suitable for production, relatively limited functionality

**Valgrind**
- ✅ **Strengths**: Industry standard, battle-tested, comprehensive features, production-grade
- ⚠️ **Limitations**: Not Rust native, significant performance overhead, steep learning curve
- 🎯 **Best for**: Deep memory debugging, complex problem troubleshooting

**Heaptrack**
- ✅ **Strengths**: Mature profiling tool, good visualization, relatively low overhead
- ⚠️ **Limitations**: Mainly for C/C++, limited Rust-specific features
- 🎯 **Best for**: Performance analysis, memory usage optimization

**jemalloc**
- ✅ **Strengths**: Production-grade allocator, excellent performance, built-in analysis features
- ⚠️ **Limitations**: Mainly an allocator, basic analysis functionality
- 🎯 **Best for**: Production environments, performance optimization

### When to Use memscope-rs

**Good scenarios:**
- 🔍 **Rust project development debugging** - Want to understand specific variable memory usage
- 📚 **Learning Rust memory management** - Visualize ownership and borrowing concepts
- 🧪 **Prototype validation** - Quickly verify memory usage patterns
- 🎯 **Smart pointer analysis** - Deep dive into Rc/Arc reference count changes

**Not recommended scenarios:**
- 🚫 **Production monitoring** - Use mature tools instead
- 🚫 **High-performance requirements** - Tracking overhead might be unacceptable
- 🚫 **Complex memory issues** - Valgrind and friends are better
- 🚫 **Large project comprehensive analysis** - Functionality and stability not sufficient yet

## Performance Characteristics

Based on actual testing (not marketing numbers):

### Tracking Overhead
- **Small programs**: ~5-15% runtime overhead (not too bad!)
- **Memory usage**: ~10-20% additional memory for tracking data
- **Large datasets**: Performance degrades significantly (we're working on it)

### Export Performance
- **Small datasets** (< 1000 allocations): < 100ms (blink and you'll miss it)
- **Medium datasets** (1000-10000 allocations): 100ms - 1s (time for a sip of coffee)
- **Large datasets** (> 10000 allocations): Several seconds (time for a full coffee break)

### Known Limitations
- **Thread safety**: Basic support, may have issues under heavy concurrency
- **Memory leaks**: Tracking itself may leak memory in some scenarios (ironic, we know)
- **Platform support**: Limited testing on different platforms
- **Error handling**: Some errors are silently ignored (we're working on being more vocal)

## Current Development Status

### What works:
- ✅ Basic variable tracking
- ✅ JSON export with allocation data
- ✅ Simple SVG charts
- ✅ Smart pointer support (Rc, Arc, Box)
- ✅ Basic CLI tools
- ✅ Examples run successfully

### Known issues (we're honest about our problems):
- ⚠️ Performance overhead in large applications
- ⚠️ 154 `unwrap()` calls that could panic (we counted them)
- ⚠️ Lock contention in multi-threaded scenarios
- ⚠️ Inconsistent API design across modules
- ⚠️ Memory usage by tracker itself
- ⚠️ Thread safety not thoroughly tested

### Planned improvements:
- 🔄 Replace dangerous `unwrap()` calls with proper error handling
- 🔄 Optimize lock usage and reduce contention
- 🔄 Better error handling and reporting
- 🔄 Performance optimization for large datasets
- 🔄 More comprehensive testing
- 🔄 API consistency improvements

## Use Cases

- **Development debugging**: Track memory usage during development
- **Performance optimization**: Identify memory bottlenecks and optimization opportunities
- **Memory leak troubleshooting**: Locate and fix memory leak issues
- **Code review**: Analyze code memory usage patterns
- **Educational demos**: Demonstrate Rust memory management mechanisms

## Technical Architecture

The project uses a modular design:

- **core/**: Core tracking functionality and type definitions
- **analysis/**: Memory analysis algorithms and pattern recognition
- **export/**: Data export and visualization generation
- **cli/**: Command-line tools and user interface
- **bin/**: Executable analysis tools

## Troubleshooting

### Common Issues

**Export fails with large datasets:**
```rust
// Use smaller buffer or exclude system allocations
let options = ExportOptions::new()
    .include_system_allocations(false)
    .buffer_size(32 * 1024);
```

**High memory usage:**
```bash
# Disable backtrace collection
cargo run --no-default-features --features tracking-allocator
```

**Permission errors on output:**
```bash
# Ensure write permissions
mkdir -p MemoryAnalysis
chmod 755 MemoryAnalysis
```

## Contributing

This is experimental software, but we welcome contributions! Please:

1. **Test thoroughly** - Make sure your changes don't break existing functionality
2. **Document limitations** - Be honest about what doesn't work
3. **Performance test** - Measure the impact of your changes
4. **Keep it simple** - Avoid over-engineering (we have enough complexity already)

```bash
# Development workflow
git clone https://github.com/TimWood0x10/memscope-rs
cd memscope-rs

make build
make run-basic
```

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Disclaimer

This software is provided "as is" without warranty. Use at your own risk. Not suitable for production environments yet, but we're working on it! Performance claims are based on limited testing and may not reflect real-world usage.

Remember: with great power comes great responsibility... and sometimes memory leaks. 🦀

---

**Made with ❤️ and 🦀 by developers who care about memory (maybe too much)**
