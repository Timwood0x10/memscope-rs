# 🦀 memscope-rs - Advanced Rust Memory Analysis & Visualization

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Safety](https://img.shields.io/badge/safety-tested-green.svg)](#safety--security)

[**memscope-rs**](https://github.com/Timwood0x10/memscope-rs.git) is a comprehensive Rust memory analysis toolkit that provides real-time tracking, visualization, and analysis of memory allocations in Rust applications. It features a custom global allocator, intuitive variable tracking, and beautiful SVG visualizations with enhanced layout and comprehensive memory insights.

## 🌟 Key Features

### 🔍 **Advanced Memory Tracking**
- **Custom Global Allocator**: Tracks every heap allocation/deallocation automatically
- **Variable Association**: Link memory allocations to source code variables using `track_var!` macro
- **Thread-Safe**: Full multi-threading support with deadlock prevention
- **Type Recognition**: Intelligent Rust type detection and categorization

### 📊 **Rich Visualizations**
- **Enhanced SVG Reports**: Beautiful, professional memory usage charts with performance dashboards
- **Lifecycle Timeline**: Visual timeline showing variable lifecycles and scope relationships
- **Type Categorization**: Groups allocations by Collections, Text, Smart Pointers, etc.
- **Dual SVG Output**: Memory analysis + lifecycle timeline for comprehensive insights
- **Human-Readable Formats**: Displays "1.2 KB", "5.4 MB" instead of raw bytes

### 🛡️ **Production Ready**
- **Deadlock-Free**: Advanced lock ordering and `try_lock` strategies
- **Performance Optimized**: Minimal overhead with graceful degradation
- **Error Resilient**: Comprehensive error handling and recovery
- **Memory Safe**: Extensive safety testing and validation

### 📈 **Export & Analysis**
- **JSON Export**: Detailed memory snapshots for programmatic analysis
- **Dual SVG Output**: Memory analysis + lifecycle timeline visualizations
- **Statistics**: Peak memory, allocation counts, type breakdowns, lifecycle metrics
- **Lifecycle Tracking**: Variable creation, destruction, and scope relationship patterns
- **Flexible Naming**: Recommended `program_name_memory_analysis.svg` and `program_name_lifecycle_timeline.svg` format

## 🎨 Lifecycle Timeline Visualization

The lifecycle timeline SVG provides a visual representation of memory allocation events over time, showing when variables are created and how they relate to each other in terms of scope and lifetime.

![Lifecycle Timeline Example](demo_lifecycle_timeline.svg)

### 📊 Lifecycle Timeline Analysis

The lifecycle timeline visualization offers several key insights:

**🔍 Timeline Structure:**
- **Horizontal Timeline**: Shows the chronological progression of memory allocation events
- **Event Markers**: Green circles represent allocation events for tracked variables
- **Variable Labels**: Each allocation shows the variable name and its Rust type
- **Scope Relationships**: Visual positioning indicates variable scope levels and relationships

**📈 Key Metrics Displayed:**
- **Total Allocations**: Complete count of memory allocation events
- **Peak Memory**: Maximum memory usage reached during execution
- **Active Memory**: Current memory usage at the time of export
- **Timeline Span**: Duration from first to last allocation event

**🎯 Use Cases:**
- **Memory Leak Detection**: Identify variables that persist longer than expected
- **Scope Analysis**: Understand variable lifetime patterns and scope relationships
- **Performance Optimization**: Spot allocation hotspots and memory usage patterns
- **Debugging**: Trace memory allocation sequences and identify problematic patterns

**🔧 Interpretation Guide:**
- **Dense Clusters**: Indicate rapid allocation sequences (loops, bulk operations)
- **Isolated Events**: Show individual variable allocations
- **Vertical Positioning**: Represents different execution contexts or scope levels
- **Color Coding**: Green markers for allocations (red for deallocations when available)

This visualization complements the memory analysis SVG by focusing on the temporal aspects of memory usage rather than just the final state.

## 🚀 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
memscope-rs = "0.1.0"

# Optional: Enable backtrace support
memscope-rs = { version = "0.1.0", features = ["backtrace"] }
```

### Basic Usage

```rust
use memscope_rs::{init, track_var, get_global_tracker};

fn main() {
    // Initialize the memory tracking system
    init();

    // Create and track variables
    let user_data = vec![1, 2, 3, 4, 5];
    track_var!(user_data).expect("Failed to track user_data");

    let config = String::from("app_config=production");
    track_var!(config).expect("Failed to track config");

    let cache = Box::new([0u8; 1024]);
    track_var!(cache).expect("Failed to track cache");

    // Get memory statistics
    let tracker = get_global_tracker();
    let stats = tracker.get_stats().expect("Failed to get stats");
    
    println!("Memory Usage:");
    println!("  Active allocations: {}", stats.active_allocations);
    println!("  Active memory: {} bytes", stats.active_memory);
    println!("  Peak memory: {} bytes", stats.peak_memory);

    // Export detailed analysis with recommended naming
    tracker.export_to_json("my_program_data.json").expect("JSON export failed");
    tracker.export_memory_analysis("my_program_memory_analysis.svg").expect("Memory analysis export failed");
    tracker.export_lifecycle_timeline("my_program_lifecycle_timeline.svg").expect("Lifecycle timeline export failed");

    println!("Analysis exported! Check the generated files:");
    println!("📊 my_program_memory_analysis.svg - Memory usage analysis");
    println!("⏱️  my_program_lifecycle_timeline.svg - Lifecycle timeline");
    println!("📄 my_program_data.json - Detailed data");
}
```


## 📖 Comprehensive Guide

### Supported Types

The `track_var!` macro works with these Rust types:

```rust
// Collections
let numbers = vec![1, 2, 3, 4, 5];
track_var!(numbers).ok();

// Text
let message = String::from("Hello, memscope-rs!");
track_var!(message).ok();

// Smart Pointers
let boxed_data = Box::new(42);
track_var!(boxed_data).ok();

// Reference Counted
let shared_data = std::rc::Rc::new(vec![1, 2, 3]);
track_var!(shared_data).ok();

// Thread-Safe Shared
let arc_data = std::sync::Arc::new(String::from("Shared"));
track_var!(arc_data).ok();
```

### Advanced Usage

#### Memory Lifecycle Tracking

```rust
fn process_user_request() -> Vec<u8> {
    let request_data = vec![0u8; 1024];
    track_var!(request_data).ok();
    
    // Process data...
    request_data // Ownership transferred
}

fn main() {
    init();
    
    let response = process_user_request();
    track_var!(response).ok(); // Track the transferred data
    
    // Analyze memory patterns
    let tracker = get_global_tracker();
    let memory_by_type = tracker.get_memory_by_type().expect("Failed to get memory by type");
    
    for type_info in memory_by_type {
        println!("{}: {} bytes ({} allocations)", 
                 type_info.type_name, 
                 type_info.total_size, 
                 type_info.allocation_count);
    }
}
```

#### Concurrent Applications

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    init();
    
    let shared_config = Arc::new(String::from("shared_configuration"));
    track_var!(shared_config).ok();
    
    let handles: Vec<_> = (0..4).map(|i| {
        let config = Arc::clone(&shared_config);
        thread::spawn(move || {
            let thread_data = vec![i; 1000];
            track_var!(thread_data).unwrap();
            
            // Thread processing...
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Analyze cross-thread memory usage
    let tracker = get_global_tracker();
    tracker.export_memory_analysis("concurrent_memory_analysis.svg").expect("Memory analysis export failed");
    tracker.export_lifecycle_timeline("concurrent_lifecycle_timeline.svg").expect("Lifecycle timeline export failed");
}
```

### Understanding the Visualizations

#### SVG Output Features

![Example Output](./images/memory_analysis.svg)

## 📊 Eight Core Memory Metrics

Our visualization provides eight key metrics to comprehensively analyze your Rust application's memory usage:

### 🔵 Basic Metrics (Central & Ring Display)
1. **Active Memory** - Current memory in use by your application
   - `stats.active_memory` (bytes)
   - Displayed in central blue circle

2. **Peak Memory** - Maximum memory usage reached during execution  
   - `stats.peak_memory` (bytes)
   - Displayed in central blue circle

3. **Active Allocations** - Number of currently active memory allocations
   - `stats.active_allocations` (count)
   - Displayed in red satellite circle

4. **Memory Reclamation Rate** - Percentage of allocated memory that has been freed
   - `(stats.total_deallocated / stats.total_allocated) × 100%`
   - Displayed in green satellite circle

### 🟠 Advanced Metrics (Ring & Linear Display)
5. **Allocator Efficiency** - How well memory is being utilized vs peak usage
   - `(stats.active_memory / stats.peak_memory) × 100%`
   - Displayed in orange satellite circle

6. **Median Allocation Size** - Middle value of all allocation sizes (50th percentile)
   - Calculated from sorted allocation sizes: `sizes[len/2]`
   - Displayed as blue linear bar

7. **P95 Allocation Size** - 95th percentile of allocation sizes (large allocations)
   - Calculated as: `sizes[(len × 0.95) as usize]`
   - Displayed as orange linear bar (full length)

8. **Memory Fragmentation** - Percentage of peak memory not currently in use
   - `((stats.peak_memory - stats.active_memory) / stats.peak_memory) × 100%`
   - Displayed in purple satellite circle

### 🎯 Visualization Layout
- **Central Circle**: Most critical metrics (Active/Peak Memory)
- **Satellite Circles**: Performance and efficiency metrics with connecting lines
- **Linear Bars**: Size comparison metrics for intuitive understanding

The enhanced SVG visualization includes:

1. **Header Statistics Panel**
   - Active allocations count
   - Current memory usage (human-readable)
   - Peak memory usage
   - Total allocation count

2. **Memory Usage by Type Chart**
   - Bar chart showing memory consumption per type
   - Color-coded by category (Collections=Blue, Text=Green, etc.)
   - Shows both size and allocation count

3. **Tracked Variables by Category**
   - Groups your tracked variables by type category
   - Shows which variables consume the most memory
   - Helps identify memory hotspots

4. **Allocation Timeline**
   - Visual timeline of when variables were allocated
   - Shows variable names and sizes
   - Helps understand allocation patterns

![Example Output](./images/lifecycle_timeline.svg)

### 🎯 Lifecycle Timeline SVG Detailed Analysis

The lifecycle timeline visualization provides a comprehensive view of variable lifecycles and scope relationships in your Rust application:

#### **📊 Timeline Structure & Components**

**🔝 Header Section:**
- **Title**: "Scope Matrix & Lifecycle Visualization" - Professional gradient styling
- **Global Legend**: Prominent progress bar explanation showing lifecycle progression patterns
- **Scope Information**: Total scopes found and variables being displayed

**🎨 Matrix Layout Section:**
- **Variable Nodes**: Each tracked variable displayed as colored circles with variable names
- **Type Information**: Shows complete Rust type information (e.g., `alloc::boxed::Box<Vec<u8>>`)
- **Scope Grouping**: Variables organized by their scope context (Global, Function, Block)
- **Visual Hierarchy**: Indentation and positioning indicate scope relationships

**📈 Top 3 Memory Analysis:**
- **Memory Bars**: Horizontal bars showing relative memory usage by type
- **Variable Details**: Each bar shows variable names and their memory consumption
- **Type Categorization**: Groups similar types together (Collections, Smart Pointers, etc.)
- **Size Comparison**: Visual comparison of memory usage across different variable types

**🔗 Variable Relationships Section:**
- **Ownership & Borrowing**: Visual representation of Rust ownership patterns
- **Scope Backgrounds**: Different colored backgrounds for different scopes
- **Relationship Lines**: Connecting lines showing variable dependencies and references
- **Interactive Elements**: Hover effects and visual feedback for better understanding

#### **🎨 Visual Design Features**

**Color Coding System:**
- **Blue Gradient**: Represents different variable lifetimes (darker = longer lived)
- **Scope Colors**: Different background colors for Global, Function, and Block scopes
- **Type Colors**: Consistent color scheme matching the memory analysis SVG
- **Relationship Lines**: Dotted and solid lines indicating different relationship types

**Layout Optimization:**
- **Responsive Design**: Adapts to different numbers of variables and scopes
- **Overflow Prevention**: Smart text truncation and layout adjustments
- **Professional Styling**: Modern shadows, gradients, and typography
- **Clear Hierarchy**: Visual separation between different analysis sections

#### **🔧 Interpretation Guide**

**Understanding Variable Lifecycles:**
- **Node Position**: Higher positions indicate earlier allocation times
- **Node Size**: Larger nodes represent variables with more memory usage
- **Color Intensity**: Darker colors indicate longer-lived variables
- **Scope Grouping**: Variables in the same scope are visually grouped together

**Relationship Analysis:**
- **Ownership Transfer**: Solid lines showing move semantics
- **Borrowing Patterns**: Dashed lines indicating reference relationships
- **Shared Ownership**: Special indicators for `Rc` and `Arc` patterns
- **Scope Boundaries**: Clear visual separation between different scopes

**Practical Applications:**
- Understanding variable scope relationships in complex Rust applications
- Identifying memory ownership patterns and potential optimizations
- Visualizing the lifecycle of smart pointers and reference-counted data
- Debugging scope-related memory issues and lifetime conflicts

#### JSON Output Structure

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "total_allocations": 150,
  "total_allocated": 2048576,
  "active_allocations": [
    {
      "ptr": 140234567890,
      "size": 1024,
      "timestamp_alloc": 1705312200000,
      "var_name": "user_data",
      "type_name": "Vec<i32>",
      "thread_id": "ThreadId(1)"
    }
  ],
  "memory_by_type": [
    {
      "type_name": "Vec<i32>",
      "total_size": 4096,
      "allocation_count": 4
    }
  ],
  "stats": {
    "total_allocations": 150,
    "active_allocations": 45,
    "peak_memory": 3145728
  }
}
```

## 🛡️ Safety & Security

### Security Analysis

We've conducted comprehensive security analysis covering:

- **Memory Safety**: Extensive testing of unsafe allocator code
- **Thread Safety**: Deadlock prevention and race condition testing
- **Resource Management**: Memory leak detection and bounds checking
- **Error Handling**: Graceful failure modes and recovery

See [SECURITY_ANALYSIS.md](SECURITY_ANALYSIS.md) for detailed analysis.

### Performance Characteristics

- **Allocation Overhead**: < 5% in typical applications
- **Memory Overhead**: ~50-100 bytes per tracked allocation
- **Lock Contention**: Minimized with `try_lock` strategies
- **Export Performance**: < 10 seconds for 10,000+ allocations

### Production Considerations

```rust
# Disable tracking in release builds
#[cfg(debug_assertions)]
memscope_rs::init();

# Or use conditional compilation
#[cfg(feature = "memory-tracking")]
memscope_rs::init();
```

## 🧪 Testing

### Running Tests

```bash
# Basic tests
cargo test

# Stress tests
cargo test --test stress_test

# Safety tests
cargo test --test safety_test

# Performance benchmarks
cargo test --test performance_test --release

# Edge cases
cargo test --test edge_cases_test

# Comprehensive integration tests
cargo test --test comprehensive_integration_test

// or 
make test
make run-stress-test
make run-main
```

### Test Coverage

- **Unit Tests**: Core functionality testing
- **Integration Tests**: Real-world usage scenarios
- **Stress Tests**: High-load and concurrent scenarios
- **Safety Tests**: Memory safety and error handling
- **Performance Tests**: Overhead and bottleneck analysis
- **Edge Cases**: Unusual inputs and boundary conditions

## 📊 Visual Memory Analysis

memscope-rs generates **two comprehensive SVG visualizations** that provide deep insights into your application's memory usage patterns:

### 🎯 **Dual SVG Output System**

#### 1. **Memory Analysis SVG** (`program_name_memory_analysis.svg`)
Comprehensive memory usage analysis including:
- **Performance Dashboard**: Real-time memory metrics and efficiency indicators
- **Memory Usage by Type**: Categorized breakdown with `varType(Type)` format
- **Variable Allocation Timeline**: Chronological allocation patterns
- **Call Stack Analysis**: Specific variable tracking (no more "Unknown"!)
- **Memory Fragmentation**: Visual fragmentation analysis
- **Hot Spots**: Memory allocation frequency analysis

#### 2. **Lifecycle Timeline SVG** (`program_name_lifecycle_timeline.svg`) 
Beautiful timeline visualization showing:
- **Variable Lifecycles**: Each variable displayed as `varName(Type)` with colored bars
- **Scope Hierarchy**: Indented display showing function/scope relationships
- **Time Progression**: Horizontal timeline with precise timestamps
- **Active Status**: Red "LIVE" indicators for variables still in memory
- **Relationship Lines**: Dotted lines connecting variables to their parent scopes
- **Modern Design**: White background, shadows, and professional styling

![Memory Analysis Visualization](stress_test_visualization.svg)

### 🎯 Performance Dashboard (Top Section)
Four key performance gauges displaying:
- **Memory Efficiency**: Allocation/deallocation ratio (35.7% in example)
- **Average Allocation Size**: Mean size per allocation (1.0K bytes)
- **Memory Utilization**: Current vs peak memory usage (100.0%)
- **Active Allocations**: Number of currently tracked allocations (25.0K)

### 🔥 Memory Allocation Heatmap (Second Section)
A 20x8 grid showing allocation density patterns:
- **X-axis**: Allocation size (small to large)
- **Y-axis**: Time progression
- **Color intensity**: Number of allocations (blue=cold, red=hot)
- **Numbers in cells**: Exact allocation counts

### 📊 Memory Usage by Type & Fragmentation Analysis (Third Row)
**Left side - Type Usage Chart**: Pie chart showing memory distribution by data types
**Right side - Fragmentation Analysis**: Histogram of allocation sizes:
- **Green bars**: Small allocations (good for performance)
- **Orange bars**: Medium allocations (moderate impact)
- **Red bars**: Large allocations (potential fragmentation risk)

### 🔍 Categorized Allocations & Call Stack Analysis (Fourth Row)
**Left side - Categorized Allocations**: Memory usage grouped by allocation categories
**Right side - Call Stack Analysis**: Tree visualization showing:
- **Colored nodes**: Different source locations
- **Node size**: Proportional to memory usage
- **Labels**: Source location with allocation count and total bytes

### 📈 Memory Growth Trends (Fifth Section)
Time-series visualization showing:
- **Green trend line**: Memory usage progression over time
- **Data points**: Specific measurement points
- **Red dashed line**: Peak memory usage indicator
- **Trend analysis**: Growth patterns and memory behavior

### 📱 Memory Timeline (Sixth Section)
Detailed timeline showing:
- **Variable lifecycles**: When variables are allocated and deallocated
- **Memory blocks**: Visual representation of active allocations
- **Time progression**: Left to right temporal flow

### 🎨 Interactive Legend & Summary (Bottom Section)
**Left side - Legend**: Color coding explanation for all chart elements
**Right side - Summary**: Key statistics including:
- Total active allocations
- Tracked variables percentage
- Average allocation size
- Memory efficiency metrics
- Peak vs current memory comparison

## 📊 Examples & Use Cases

The `examples/` directory contains comprehensive demonstration programs showcasing different memory usage patterns:

### 🚀 Basic Examples
- **`basic_usage.rs`** - Simple tracking example showing fundamental usage
- **`lifecycles.rs`** - Variable lifecycle tracking with scope management

### 💪 Advanced Examples  
- **`heavy_workload.rs`** - Complex application simulation with:
  - Web server session management (1,000 sessions)
  - Data pipeline processing (10,000+ records)
  - LRU cache system with hit/miss patterns
  - Concurrent worker pool (8 threads, 2,000 tasks)

- **`memory_stress_test.rs`** - Extreme stress testing scenarios including:
  - **Massive Allocation Burst**: 50,000+ rapid allocations
  - **Memory Fragmentation**: Complex fragmentation patterns
  - **Concurrent Storm**: 16 threads with 80,000+ allocations
  - **Large Object Stress**: Objects up to 10MB each
  - **Rapid Cycles**: 100,000+ allocation/deallocation cycles

### 🎯 Running Examples
```bash
# Basic usage demonstration
cargo run --example basic_usage

# Variable lifecycle tracking
cargo run --example lifecycles

# Complex workload simulation
cargo run --example heavy_workload

# Extreme stress testing (generates rich visualizations)
cargo run --example memory_stress_test
```

Each example generates detailed JSON snapshots and beautiful SVG visualizations showing memory usage patterns, performance metrics, and allocation analysis.

### Example 1: Web Server Memory Analysis

```rust
use memscope_rs::{init, track_var, get_global_tracker};

struct WebServer {
    connections: Vec<String>,
    cache: std::collections::HashMap<String, Vec<u8>>,
}

fn main() {
    init();
    
    let mut server = WebServer {
        connections: Vec::new(),
        cache: std::collections::HashMap::new(),
    };
    
    // Simulate handling requests
    for i in 0..100 {
        let connection = format!("Connection {}", i);
        track_var!(connection).ok();
        server.connections.push(connection);
        
        let response_data = vec![0u8; 1024];
        track_var!(response_data).ok();
        server.cache.insert(format!("key_{}", i), response_data);
    }
    
    // Analyze server memory usage
    let tracker = get_global_tracker();
    tracker.export_memory_analysis("webserver_memory_analysis.svg").expect("Memory analysis export failed");
    tracker.export_lifecycle_timeline("webserver_lifecycle_timeline.svg").expect("Lifecycle timeline export failed");
    
    println!("Web server memory analysis exported!");
}
```

### Example 2: Data Processing Pipeline

```rust
use memscope_rs::{init, track_var, get_global_tracker};

fn process_data_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    // Stage 1: Load raw data
    let raw_data = vec![0u8; 1_000_000]; // 1MB of raw data
    track_var!(raw_data).ok();
    
    // Stage 2: Parse into structured data
    let parsed_data: Vec<i32> = raw_data.chunks(4)
        .map(|chunk| i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();
    track_var!(parsed_data).ok();
    
    // Stage 3: Process and filter
    let processed_data: Vec<i32> = parsed_data.into_iter()
        .filter(|&x| x > 0)
        .map(|x| x * 2)
        .collect();
    track_var!(processed_data).ok();
    
    // Stage 4: Generate results
    let results = processed_data.iter()
        .map(|&x| format!("Result: {}", x))
        .collect::<Vec<_>>();
    track_var!(results).ok();
    
    // Analyze pipeline memory usage
    let tracker = get_global_tracker();
    let stats = tracker.get_stats().expect("Failed to get stats");
    
    println!("Pipeline Memory Usage:");
    println!("  Peak memory: {} bytes", stats.peak_memory);
    println!("  Active allocations: {}", stats.active_allocations);
    
    tracker.export_to_json("pipeline_analysis.json").expect("Export failed");
    tracker.export_to_svg("pipeline_visualization.svg").expect("Export failed");
    
    Ok(())
}
```

## 🔧 Configuration

### Features

```toml
[dependencies]
memscope-rs = { version = "0.1.0", features = ["backtrace"] }
```

Available features:
- `backtrace`: Enable backtrace capture for allocations (requires `backtrace` crate)
- `tracking-allocator`: Enable custom global allocator (default)

### Environment Variables

```bash
# Set logging level
RUST_LOG=memscope_rs=debug cargo run

# Disable tracking at runtime
TRACE_TOOLS_DISABLED=1 cargo run
```

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines:

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Add tests** for your changes
4. **Run** the full test suite (`cargo test`)
5. **Commit** your changes (`git commit -m 'Add amazing feature'`)
6. **Push** to the branch (`git push origin feature/amazing-feature`)
7. **Open** a Pull Request

### Development Setup

```bash
git clone https://github.com/Timwood0x10/memscope-rs.git
cd memscope-rs
cargo build
cargo test
```

## 📄 License

This project is licensed under the Apache License, Version 2.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Rust community for excellent tooling and libraries
- Contributors to the `serde`, `svg`, and `tracing` crates
- Memory analysis research and best practices from the systems programming community

## 📞 Support

- **Documentation**: Run `cargo doc --open` to view local documentation
- **Issues**: [GitHub Issues](https://github.com/Timwood0x10/memscope-rs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Timwood0x10/memscope-rs/discussions)

---

**Made with ❤️ and 🦀 by the Rust community**