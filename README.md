# memscope-rs - Advanced Rust Memory Analysis Toolkit

[![Rust](https://img.shields.io/badge/rust-1.82+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/memscope-rs.svg)](https://crates.io/crates/memscope-rs)

**A comprehensive memory analysis toolkit with specialized tracking strategies for single-threaded, multi-threaded, and async Rust applications.**

---

## 🎯 Four Specialized Tracking Strategies

memscope-rs provides **four intelligent tracking strategies** automatically selected based on your application patterns:

| Strategy | Use Case | Performance | Best For |
|----------|----------|-------------|----------|
| 🧩 **Core Tracker** | Development & debugging | Zero overhead | Precise analysis with `track_var!` macros |
| 🔀 **Lock-free Multi-threaded** | High concurrency (100+ threads) | Thread-local sampling | Production monitoring, zero contention |
| ⚡ **Async Task-aware** | async/await applications | < 5ns per allocation | Context-aware async task tracking |
| 🔄 **Unified Backend** | Complex hybrid applications | Adaptive routing | Automatic strategy selection and switching |

## 🚀 Quick Start Examples

### 🧩 Core Tracking (Zero Overhead)
```rust
use memscope_rs::{track_var, track_var_smart, track_var_owned};

fn main() {
    // Zero-overhead reference tracking (recommended)
    let data = vec![1, 2, 3, 4, 5];
    track_var!(data);
    
    // Smart tracking (automatic strategy selection)
    let number = 42i32;        // Copy type - copied
    let text = String::new();  // Non-copy - tracked by reference
    track_var_smart!(number);
    track_var_smart!(text);
    
    // Ownership tracking (precise lifecycle analysis)
    let tracked = track_var_owned!(vec![1, 2, 3]);
    
    // Export with multiple formats
    memscope_rs::export_user_variables_json("analysis.json").unwrap();
    memscope_rs::export_user_variables_binary("analysis.memscope").unwrap();
}
```

### 🔀 Lock-free Multi-threaded (100+ Threads)
```rust
use memscope_rs::lockfree;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize lock-free tracking
    lockfree::initialize_lockfree_tracking()?;
    
    // Spawn many threads (scales to 100+ threads)
    let handles: Vec<_> = (0..100).map(|i| {
        std::thread::spawn(move || {
            // Thread-local tracking with intelligent sampling
            for j in 0..1000 {
                let data = vec![i; j % 100 + 1];
                lockfree::track_allocation(&data, &format!("data_{}_{}", i, j));
            }
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Aggregate and analyze all threads
    let analysis = lockfree::aggregate_all_threads()?;
    lockfree::export_analysis(&analysis, "lockfree_analysis")?;
    
    Ok(())
}
```

### ⚡ Async Task-aware Tracking
```rust
use memscope_rs::async_memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize async-aware tracking
    async_memory::initialize().await?;
    
    // Track memory across async tasks
    let tasks: Vec<_> = (0..50).map(|i| {
        tokio::spawn(async move {
            let data = vec![i; 1000];
            async_memory::track_in_task(&data, &format!("async_data_{}", i)).await;
            
            // Simulate async work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        })
    }).collect();
    
    futures::future::join_all(tasks).await;
    
    // Export task-aware analysis
    let analysis = async_memory::generate_analysis().await?;
    async_memory::export_visualization(&analysis, "async_analysis").await?;
    
    Ok(())
}
```

### 🔄 Unified Backend (Automatic Strategy Selection)
```rust
use memscope_rs::unified::{UnifiedBackend, BackendConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize unified backend with automatic detection
    let mut backend = UnifiedBackend::initialize(BackendConfig::default())?;
    
    // Backend automatically detects environment and selects optimal strategy:
    // - Single-threaded: Core tracker
    // - Multi-threaded: Lock-free tracker  
    // - Async runtime: Async-aware tracker
    // - Mixed: Hybrid strategy
    
    let session = backend.start_tracking()?;
    
    // Your application logic here - tracking happens transparently
    let data = vec![1, 2, 3, 4, 5];
    // Backend handles tracking automatically
    
    // Collect comprehensive analysis
    let analysis = session.collect_data()?;
    let final_data = session.end_session()?;
    
    // Export unified analysis
    backend.export_analysis(&final_data, "unified_analysis")?;
    
    Ok(())
}
```

## 🔥 Key Features

### 📊 Advanced Export Formats
- **JSON Export**: Human-readable with interactive HTML dashboards
- **Binary Export**: High-performance format (5-10x faster, 60-80% smaller)
- **Streaming Export**: Memory-efficient for large datasets
- **HTML Dashboards**: Interactive real-time visualization

### 🛡️ Smart Pointer Support
- **Automatic Detection**: Rc, Arc, Box, and custom smart pointers
- **Reference Counting**: Accurate ref count tracking
- **Lifecycle Analysis**: Comprehensive ownership history
- **Memory Safety**: Enhanced safety analysis and validation

### 🔧 Production-Ready Features
- **Zero Overhead**: Reference tracking with no runtime cost
- **Thread Safety**: Robust multi-threading support up to 100+ threads
- **Sampling Support**: Configurable sampling for production environments
- **Error Recovery**: Panic-safe error handling and graceful degradation

### 🎯 Advanced Analysis
- **FFI Boundary Tracking**: C/C++ interop memory analysis
- **Container Analysis**: Vec, HashMap, BTreeMap specialized tracking
- **Drop Chain Analysis**: Complex destructor chain analysis
- **Memory Passport**: Detailed allocation lifecycle tracking

## 📊 Performance Benchmarks

### 🚀 Tracking Overhead
| Strategy | Overhead | Best Use Case |
|----------|----------|---------------|
| **Reference Tracking** | ~0% (zero-cost) | Development debugging |
| **Ownership Tracking** | ~5-10% | Precise lifecycle analysis |
| **Lock-free Multi-threaded** | ~2-8% (adaptive sampling) | High concurrency production |
| **Async Task-aware** | < 5ns per allocation | Async applications |

### 📈 Export Performance  
| Format | Speed vs JSON | Size vs JSON | Use Case |
|--------|---------------|--------------|----------|
| **Binary Export** | 5-10x faster | 60-80% smaller | Production, large datasets |
| **JSON Export** | Baseline | Baseline | Development, debugging |
| **Streaming Export** | Memory-efficient | Variable | Large datasets, limited memory |

### 🔧 Scalability
| Metric | Single-threaded | Multi-threaded | Async |
|--------|----------------|----------------|-------|
| **Concurrency** | 1 thread | 100+ threads | 50+ tasks |
| **Variables** | 1M+ variables | 100K+ per thread | 10K+ per task |
| **Memory Usage** | ~50KB + 100B/var | Thread-local pools | Task-local buffers |

### 📊 Export Performance (Real Test Data)

| Module | Export Time | File Size | Use Case |
|--------|-------------|-----------|----------|
| Single-threaded | 1.3s | 1.2MB | Development analysis |
| Multi-threaded | 211ms | 480KB | Production monitoring |
| Async | 800ms | 800KB | Task performance analysis |
| Hybrid | 2.1s | 2.5MB | Comprehensive analysis |

*Based on actual test results from example applications*

### 🎮 Interactive HTML Dashboards

All modules generate rich, interactive HTML dashboards:

- **Memory Timeline**: Real-time allocation/deallocation patterns
- **Thread Analysis**: Per-thread memory usage and performance metrics
- **Task Insights**: Async task lifecycle and resource usage
- **Smart Pointer Tracking**: Reference counting and relationship analysis
- **Leak Detection**: Automatic identification of potential memory leaks
- **Performance Bottlenecks**: CPU, I/O, and memory correlation analysis

## 🚀 Try It Now

```bash
# Clone the repository
git clone https://github.com/TimWood0x10/memscope-rs
cd memscope-rs

# Try each module:
cargo run --example basic_usage                    # 🧩 Single-threaded
cargo run --example complex_multithread_showcase   # 🔀 Multi-threaded  
cargo run --example comprehensive_async_showcase   # ⚡ Async
cargo run --example enhanced_30_thread_demo        # 🔄 Hybrid

# Generate HTML reports:
make html DIR=MemoryAnalysis BASE=basic_usage
```

## 📚 Documentation

### 🎯 Core Tracking Modules

- **[Core Modules Overview](docs/en/core-modules.md)** - Complete comparison of all four tracking strategies
- **[Single-threaded Module](docs/en/single-threaded.md)** - Zero-overhead `track_var!` macros with examples
- **[Multi-threaded Module](docs/en/multithread.md)** - Lock-free high-concurrency tracking for 20+ threads
- **[Async Module](docs/en/async.md)** - Task-centric memory analysis for async/await applications
- **[Hybrid Module](docs/en/hybrid.md)** - Comprehensive cross-module analysis and visualization

### 📖 Complete Documentation

- **[Getting Started](docs/en/getting-started/)** - Installation, quick start, and basic tutorials
- **[User Guide](docs/en/user-guide/)** - Tracking macros, analysis, export formats, CLI tools
- **[API Reference](docs/en/api-reference/)** - Complete API documentation with examples
- **[Examples](docs/en/examples/)** - Real-world usage examples and integration guides
- **[Advanced Features](docs/en/advanced/)** - Binary format, custom allocators, performance optimization

### 🌍 Multi-language Documentation

- **[English Documentation](docs/en/)** - Complete English documentation
- **[中文文档](docs/zh/)** - 完整的中文文档

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

- **JSON export**: Export detailed memory allocation data for programmatic analysis
- **Binary export**: Efficient binary format for large datasets with faster I/O
- **SVG visualization**: Generate memory usage charts and timelines (pretty pictures!)
- **🎯 HTML Interactive Dashboard**: Full-featured web-based dashboard with clickable charts, filterable data, and real-time analysis
  - **Binary → HTML**: Convert binary snapshots directly to interactive HTML dashboards
  - **JSON → HTML**: Transform JSON analysis data into rich web visualizations
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

# Comprehensive memory analysis showcase
cargo run --example comprehensive_memory_analysis

# Complex lifecycle showcase
cargo run --example comprehensive_binary_to_html_demo

# Memory stress test (warning: may stress your computer too)
cargo run --example heavy_workload_test

# Multi-threaded stress test
cargo run --example multithreaded_stress_test

# Performance test
cargo run --example performance_benchmark_demo

# Realistic usage with extensions
cargo run --example realistic_usage_with_extensions

# Large-scale binary comparison
cargo run --example large_scale_binary_comparison

# Unsafe/FFI safety demo (for the brave souls)
cargo run --example unsafe_ffi_demo

# Async basic test
cargo run --example async_basic_test

# Simple binary test
cargo run --example simple_binary_test

# JSON export test
cargo run --example test_binary_to_json
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

## Performance Testing & Benchmarks

### 🎯 Quick Start Commands

```bash
# Clone and setup
git clone https://github.com/TimWood0x10/memscope-rs
cd memscope-rs

# Build and test basic functionality
make build
make run-basic

# Generate HTML report
make html DIR=MemoryAnalysis/basic_usage BASE=user OUTPUT=memory_report.html VERBOSE=1 
open ./MemoryAnalysis/basic_usage/memory_report.html

```

### 📊 Available Benchmarks

```bash
# Fast benchmarks (recommended)
make benchmark-main          # ~2 minutes

# Comprehensive benchmarks
make run-benchmark           # Full performance analysis
make run-core-performance    # Core system evaluation
make run-simple-benchmark    # Quick validation

# Stress testing
cargo run --example heavy_workload_test
cargo run --example multithreaded_stress_test
```

## Build & Installation

### System Requirements

- **Rust**: 1.82 or later (required for icu_properties_data compatibility)
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
make run-basic
├── complex_lifecycle_snapshot_complex_types.json
├── complex_lifecycle_snapshot_lifetime.json
├── complex_lifecycle_snapshot_memory_analysis.json
├── complex_lifecycle_snapshot_performance.json
├── complex_lifecycle_snapshot_security_violations.json
├── complex_lifecycle_snapshot_unsafe_ffi.json


# Export to different formats
make html DIR=MemoryAnalysis/basic_usage OUTPUT=memory_report.html  # JSON → HTML
cargo run --example comprehensive_binary_to_html_demo              # Binary → HTML
cargo run --example large_scale_binary_comparison              # Binary format comparison demo

# View generated dashboards
open memory_report.html                    # From JSON conversion
open comprehensive_report.html             # From binary conversion

# You can view the HTML interface examples in ./images/*.html
```

### From Crates.io

```bash
# Add to your project
cargo add memscope-rs

# Or manually add to Cargo.toml
[dependencies]
memscope-rs = "0.1.5"
```

### Feature Flags

```toml
[dependencies]
memscope-rs = { version = "0.1.5" }
```

Available features:

- `backtrace` - Enable stack trace collection (adds overhead, but gives you the full story)
- `derive` - Enable derive macro support (experimental, use at your own risk)
- `tracking-allocator` - Custom allocator support (enabled by default)

## Output File Structure & Interactive Dashboard

After running programs, you'll find analysis results in the `MemoryAnalysis/` directory:

```
├── basic_usage_memory_analysis.json     // comprehensive memory data
├── basic_usage_lifetime.json            // variable lifetime info
├── basic_usage_performance.json         // performance metrics 
├── basic_usage_security_violations.json // security analysis
├── basic_usage_unsafe_ffi.json          // unsafe && ffi info
├── basic_usage_complex_types.json       // complex types data
└── memory_report.html                   // interactive dashboard
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

## Current Development Status 


## What works reliably:

- ✅ **Single-threaded variable tracking**: Core functionality works well in single-threaded environments
- ✅ **Multi-format data export**:
  - JSON export with complete allocation data
  - Binary export for efficient large dataset handling
  - Direct binary → HTML conversion with interactive dashboards
  - JSON → HTML transformation with rich visualizations
- ✅ **Interactive HTML dashboard**: Feature-rich visualization with clickable elements, variable relationship graphs, 3D memory layout
- ✅ **Smart pointer support**: Full Rc, Arc, Box tracking with reference counting
- ✅ **Memory analysis**: Basic leak detection and pattern analysis
- ✅ **CLI tools and examples**: All demonstration programs run successfully

#### Known critical issues (honest assessment):

- ⚠️ **Multi-threading deadlocks**: Global tracker with multiple mutexes causes deadlocks(20 threads limit or be killed)
- ⚠️ **Performance overhead**: 5-15% runtime overhead, degrades significantly with large datasets
- ⚠️ **934 unsafe `unwrap()` calls**: Potential panic points that need proper error handling
  - **Risk**: Application can panic unexpectedly during memory tracking operations
  - **Mitigation**: Use `MEMSCOPE_TEST_MODE=1` for safer fallback behavior
  - **Status**: Active work in progress to replace with safe alternatives
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

**Application hangs or deadlocks in multi-threaded code:**

```bash
# Use one of these environment variables:
export MEMSCOPE_DISABLE_GLOBAL=1   # Completely disable global tracking
export MEMSCOPE_TEST_MODE=1        # Use simplified tracking logic
export MEMSCOPE_ASYNC_MODE=1       # Skip heavy operations
export RUST_TEST_THREADS=1         # Force single-threaded execution
```

**Panic with "unwrap() called on None":**

```bash
# Enable safer fallback behavior
export MEMSCOPE_TEST_MODE=1
# Or disable specific features
export MEMSCOPE_DISABLE_BACKTRACE=1
```

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

**Performance degradation:**

```bash
# Use fast mode with reduced tracking
export MEMSCOPE_FAST_MODE=1
# Or disable expensive operations
export MEMSCOPE_DISABLE_ANALYSIS=1
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

## 🛠️ Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
memscope-rs = "0.1.6"

# Optional features
[features]
default = ["parking-lot"]
derive = ["memscope-rs/derive"]           # Derive macros
enhanced-tracking = ["memscope-rs/enhanced-tracking"]  # Advanced analysis
system-metrics = ["memscope-rs/system-metrics"]        # System monitoring
```

## 🔧 CLI Tools

memscope-rs includes powerful command-line tools:

```bash
# Analyze existing memory data
cargo run --bin memscope-analyze -- analysis.json

# Generate comprehensive reports
cargo run --bin memscope-report -- --input analysis.memscope --format html

# Run performance benchmarks
cargo run --bin memscope-benchmark -- --threads 50 --allocations 10000
```

## 📚 Documentation

- **[API Documentation](https://docs.rs/memscope-rs)** - Complete API reference
- **[User Guide](docs/user_guide.md)** - Step-by-step tutorials
- **[Examples](examples/)** - Real-world usage examples
- **[Performance Guide](docs/performance.md)** - Optimization tips

## 🙏 Help Me Improve This Project

**I need your feedback!** While memscope-rs has comprehensive functionality, I believe it can be even better with your help.

### 🐛 **Found a Bug? Please Tell Me!**

I've put tremendous effort into testing, but complex software inevitably has edge cases I haven't encountered. Your real-world usage scenarios are invaluable:

- **Performance issues** in your specific use case
- **Compatibility problems** with certain crates or Rust versions  
- **Unexpected behavior** that doesn't match documentation
- **Missing features** that would make your workflow easier

### 💡 **How You Can Help**

1. **Create Issues**: [Open an issue](https://github.com/TimWood0x10/memscope-rs/issues/new) - no matter how small!
2. **Share Use Cases**: Tell me how you're using memscope-rs
3. **Report Performance**: Let me know if tracking overhead is higher than expected
4. **Documentation Gaps**: Point out anything confusing or unclear

### 🚀 **Your Experience Matters**

Every issue report helps make memscope-rs more robust for the entire Rust community. I'm committed to:
- **Quick responses** to reported issues
- **Transparent communication** about fixes and improvements  
- **Recognition** for your contributions

**Together, we can build the best memory analysis tool for Rust!** 🦀

---

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Running Tests
```bash
make test        # Run all tests
make check       # Check code quality
make benchmark   # Run performance benchmarks
```

## 📄 License

This project is licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

---

**Made with ❤️ and 🦀 by developers who care about memory (maybe too much) \**
