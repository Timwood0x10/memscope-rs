# memscope-rs - Rust Memory Tracking & Analysis Library

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/memscope-rs.svg)](https://crates.io/crates/memscope-rs)

## What is this thing?

memscope-rs is a Rust library for tracking memory allocations and generating analysis reports. Think of it as a friendly neighborhood memory detective 🕵️ that helps you understand what your variables are up to when you're not looking.

It provides simple macros for variable tracking and exports data in JSON and SVG formats. Perfect for those "wait, where did all my memory go?" moments.

## ⚠️ Important: Multi-threading Support Status 

**Current Status: Single-threaded environments work reliably. Multi-threaded scenarios require additional design and optimization.**

#### ✅ What Works Well (Single-threaded)

- Variable tracking with `track_var!` macro
- Memory allocation and deallocation monitoring
- Smart pointer reference counting (Rc, Arc, Box)
- JSON/SVG/HTML export functionality
- Memory leak detection and analysis
- Interactive dashboard generation

#### ⚠️ Multi-threading Limitations

The current implementation has known issues in multi-threaded environments:

**Root Cause:** The global `MemoryTracker` uses multiple `Mutex<T>` instances that can lead to deadlocks:

```rust
pub struct MemoryTracker {
    active_allocations: Mutex<HashMap<usize, AllocationInfo>>,
    bounded_stats: Mutex<BoundedMemoryStats>,
    history_manager: Mutex<AllocationHistoryManager>,
    ownership_history: Mutex<OwnershipHistoryRecorder>,
    stats: Mutex<MemoryStats>,
    // Multiple mutexes = potential deadlock risk
}
```

**Symptoms you might encounter:**

- Application hangs when using `track_var!` in multi-threaded code
- Deadlocks during memory tracking operations
- Inconsistent tracking results
- Performance degradation under high concurrency

#### 🛠️ Current Workarounds

If you need to use memscope-rs in multi-threaded applications, we provide several mitigation options:

**Option 1: Disable Global Tracking**

```bash
export MEMSCOPE_DISABLE_GLOBAL=1
# This completely bypasses global state, safe but limited functionality
```

**Option 2: Enable Fast Mode**

```bash
export MEMSCOPE_TEST_MODE=1
# Uses simplified tracking logic with reduced lock contention
```

**Option 3: Async-Compatible Mode**

```bash
export MEMSCOPE_ASYNC_MODE=1
# Skips heavy operations that can cause deadlocks
```

**Option 4: Limit Test Concurrency**

```bash
export RUST_TEST_THREADS=1
# Force single-threaded execution for testing
```

#### 🔮 Future Plans

We are actively working on a lock-free multi-threaded implementation that will include:

- Lock-free data structures using atomic operations
- Per-thread local buffers with periodic aggregation
- Sampling-based tracking similar to profiling tools like `perf`
- Better separation between fast-path operations and analysis

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
├── complex_lifecycle_snapshot_complex_types.json  // complex types info
├── complex_lifecycle_snapshot_lifetime.json // varable lifetiem infos
├── complex_lifecycle_snapshot_memory_analysis.json  // all things 
├── complex_lifecycle_snapshot_performance.json   // performances 
├── complex_lifecycle_snapshot_security_violations.json  // security violations
├── complex_lifecycle_snapshot_unsafe_ffi.json  // unsafe && ffi infos
└── improved_tracking_final_v3.html // output html
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
# output html 
make html DIR=YOUR_JSON_DIR BASE=complex_lifecycle OUTPUT=improved_tracking_final.html

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

| Feature                          | memscope-rs | Valgrind | Heaptrack | jemalloc |
| -------------------------------- | ----------- | -------- | --------- | -------- |
| **Rust Native**            | ✅          | ❌       | ❌        | ⚠️     |
| **Variable Names**         | ✅          | ❌       | ❌        | ❌       |
| **Smart Pointer Analysis** | ✅          | ⚠️     | ⚠️      | ❌       |
| **Visual Reports**         | ✅          | ⚠️     | ✅        | ❌       |
| **Production Ready**       | ⚠️        | ✅       | ✅        | ✅       |
| **Interactive Timeline**   | ✅          | ❌       | ⚠️      | ❌       |
| **Real-time Tracking**     | ⚠️        | ✅       | ✅        | ✅       |
| **Low Overhead**           | ⚠️        | ⚠️     | ✅        | ✅       |
| **Mature Ecosystem**       | ❌          | ✅       | ✅        | ✅       |

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

## Current Development Status | 当前开发状态

### English Assessment

#### What works reliably:

- ✅ **Single-threaded variable tracking**: Core functionality works well in single-threaded environments
- ✅ **JSON export with allocation data**: Complete and accurate data export
- ✅ **Interactive HTML dashboard**: Feature-rich visualization with clickable elements
- ✅ **Smart pointer support**: Full Rc, Arc, Box tracking with reference counting
- ✅ **Memory analysis**: Basic leak detection and pattern analysis
- ✅ **CLI tools and examples**: All demonstration programs run successfully

#### Known critical issues (honest assessment):

- ⚠️ **Multi-threading deadlocks**: Global tracker with multiple mutexes causes deadlocks(20 threads limit or be killed)
- ⚠️ **Performance overhead**: 5-15% runtime overhead, degrades significantly with large datasets
- ⚠️ **154 unsafe `unwrap()` calls**: Potential panic points that need proper error handling
- ⚠️ **Thread safety**: Basic support only, not thoroughly tested under concurrency
- ⚠️ **Memory leaks in tracker itself**: Tracking system can leak memory (ironic but true)
- ⚠️ **Inconsistent API design**: Some modules use different patterns and conventions
- ⚠️ **Limited platform testing**: Mainly tested on specific development environments

#### Production readiness:

- 🚫 **Not suitable for production**: Current status is experimental/development tool only
- 🚫 **No stability guarantees**: APIs may change, memory safety not fully validated
- ✅ **Good for development debugging**: Excellent for understanding memory patterns during development
- ✅ **Educational value**: Great for learning Rust memory management concepts

### Planned improvements :

#### High Priority :

- 🔄 **Multi-threading safety**: Implement lock-free tracking architecture
- 🔄 **Replace dangerous `unwrap()` calls**: 154 calls need proper error handling
- 🔄 **Performance optimization**: Reduce overhead for large datasets
- 🔄 **Memory leak fixes**: Fix tracker's own memory leaks

#### Medium Priority :

- 🔄 **API consistency**: Standardize interfaces across modules
- 🔄 **Better error handling**: Comprehensive error reporting system
- 🔄 **Cross-platform testing**: Validate on Windows, macOS, Linux
- 🔄 **Documentation improvements**: More examples and use cases

#### Future Goals :

- 🔄 **Production readiness**: Stability and performance validation
- 🔄 **Advanced analysis**: ML-based memory pattern detection
- 🔄 **Integration support**: IDE plugins and CI/CD integration
- 🔄 **Real-time monitoring**: Live memory tracking dashboard

## Use Cases

### ✅ Recommended Use Cases

#### Single-threaded Applications

#### **Development debugging** : Track memory usage during development

- **Performance optimization** : Identify memory bottlenecks and optimization opportunities
- **Memory leak troubleshooting** : Locate and fix memory leak issues
- **Code review** : Analyze code memory usage patterns
- **Educational demos** : Demonstrate Rust memory management mechanisms
- **Algorithm analysis** : Understand memory behavior of data structures and algorithms

### ⚠️ Use with Caution

### Multi-threaded Applications

- **Only with workarounds** : Use environment variables to disable problematic features
- **Testing environments** : Single-threaded test execution with `RUST_TEST_THREADS=1`
- **Development debugging** : Limited tracking with `MEMSCOPE_DISABLE_GLOBAL=1`

**Required precautions for multi-threaded use:**

```bash
# Choose one of these approaches:
export MEMSCOPE_DISABLE_GLOBAL=1   # Safest: disables global tracking
export MEMSCOPE_ASYNC_MODE=1       # Skips heavy operations
export MEMSCOPE_TEST_MODE=1        # Uses simplified tracking
export RUST_TEST_THREADS=1         # Forces single-threaded execution
```

### 🚫 Not Recommended

- **Production environments**: Not stable enough, use mature tools instead
- **High-performance applications :** Tracking overhead may be unacceptable
- **Critical systems** : Potential deadlocks and memory leaks in tracker itself
- **Large-scale applications** : Performance degrades significantly with large datasets
- **Concurrent servers** : Multi-threading limitations make it unsuitable

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

**This software is experimental and not production-ready.** Please read carefully:

- ⚠️ **Multi-threading deadlock risk**: Known deadlock issues in concurrent environments
- ⚠️ **Memory safety not guaranteed**: 154 unsafe `unwrap()` calls could cause panics
- ⚠️ **Performance impact**: 5-15% runtime overhead, significant with large datasets
- ⚠️ **Memory leaks**: The tracker itself may leak memory during operation
- ⚠️ **Limited testing**: Not thoroughly tested across different platforms and scenarios
- ⚠️ **API instability**: Interfaces may change in future versions

**Use at your own risk.** This tool is suitable for development debugging and educational purposes only. For production monitoring, consider mature alternatives like Valgrind, Heaptrack, or jemalloc's built-in profiling.

---

**Made with ❤️ and 🦀 by developers who care about memory (maybe too much) \**
