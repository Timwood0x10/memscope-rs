# MemScope-rs Examples

This directory contains comprehensive examples demonstrating different tracking strategies and use cases for memscope-rs.

## 📁 Available Examples

### Core Single-threaded Examples
| Example | Description | Key Features |
|---------|-------------|--------------|
| [`basic_usage.rs`](basic_usage.rs) | Simple single-threaded tracking | Zero-overhead reference tracking |

### Lock-free Multi-threaded Examples  
| Example | Description | Key Features |
|---------|-------------|--------------|
| [`complex_multithread_showcase.rs`](complex_multithread_showcase.rs) | High-concurrency demonstration (100+ threads) | Thread-local tracking, automatic cleanup |
| [`enhanced_30_thread_demo.rs`](enhanced_30_thread_demo.rs) | ⭐ **Interactive HTML Dashboard** | Real-time visualization with JavaScript |

### Async Examples
| Example | Description | Key Features |
|---------|-------------|--------------|
| [`comprehensive_async_showcase.rs`](comprehensive_async_showcase.rs) | Task-aware async memory tracking | Context-based task tracking |

### Lifecycle & Analysis Examples
| Example | Description | Key Features |
|---------|-------------|--------------|
| [`complex_lifecycle_showcase.rs`](complex_lifecycle_showcase.rs) | Variable lifecycle analysis | Drop tracking, reference counting |
| [`performance_test_visualization.rs`](performance_test_visualization.rs) | Performance benchmarking | Timing analysis, bottleneck detection |

### Advanced Features
| Example | Description | Key Features |
|---------|-------------|--------------|
| [`unsafe_ffi_demo.rs`](unsafe_ffi_demo.rs) | FFI boundary tracking | C/C++ interop analysis |
| [`large_binary.rs`](large_binary.rs) | Large dataset handling | Binary export, streaming |
| [`verified_selective_demo.rs`](verified_selective_demo.rs) | Selective tracking strategies | Custom filtering |
| [`comprehensive_binary_to_html_demo.rs`](comprehensive_binary_to_html_demo.rs) | Binary to HTML conversion | Format conversion pipeline |

## 🌟 Featured Example: Enhanced 30-Thread Demo

The [`enhanced_30_thread_demo.rs`](enhanced_30_thread_demo.rs) example creates an **interactive HTML dashboard** with real-time data visualization.

### ⚠️ **Important Setup Requirement**

For full interactivity, the generated HTML file **must be placed in the same directory** as the JavaScript files:

```bash
# Run the example
cargo run --example enhanced_30_thread_demo

# This generates: enhanced_30_thread_demo.html

# REQUIRED: Copy JavaScript dependencies to the same directory
cp ./templates/hybrid_dashboard.js ./enhanced_30_thread_demo_files/
cp ./templates/enhanced_diagnostics.js ./enhanced_30_thread_demo_files/

# Now open enhanced_30_thread_demo.html in your browser
```

### Interactive Features (with JavaScript)
- 📊 **Real-time Charts**: Dynamic memory usage visualization
- 🔍 **Drill-down Analysis**: Click to explore detailed data
- 📈 **Performance Metrics**: Live performance monitoring
- 🧭 **Navigation**: Interactive timeline and filtering

### Static View (without JavaScript)
- 📋 **Summary Dashboard**: Basic statistics and overview
- 📄 **Static Charts**: Non-interactive visualizations
- 📝 **Text Reports**: Detailed textual analysis

## 🚀 Quick Start

### Single-threaded (Zero Overhead)
```bash
cargo run --example basic_usage
```

### Multi-threaded (High Concurrency)
```bash
# Simple multi-threading
cargo run --example complex_multithread_showcase

# Interactive dashboard (remember to copy JS files!)
cargo run --example enhanced_30_thread_demo
```

### Async Task Tracking
```bash
cargo run --example comprehensive_async_showcase
```

## 📊 Output Formats

### JSON Export (Human-readable)
```json
{
  "tracking_strategy": "lockfree",
  "total_allocations": 12500,
  "peak_memory": "2.4MB",
  "threads_tracked": 30,
  "analysis_duration": "15ms"
}
```

### Binary Export (High Performance)
- **Speed**: 5-10x faster than JSON
- **Size**: 60-80% smaller files  
- **Use case**: Large datasets, production monitoring

### HTML Dashboard (Interactive)
- **Real-time visualization**: Dynamic charts and graphs
- **Detailed analysis**: Drill-down capabilities
- **Cross-platform**: Works in any modern browser

## 🎯 Tracking Strategies

### 🧩 Core (Single-threaded)
```rust
use memscope_rs::{track_var, export_user_variables_json};

fn main() {
    let data = vec![1, 2, 3];
    track_var!(data);
    export_user_variables_json("analysis.json").unwrap();
}
```
- **Overhead**: ~0% (zero-cost)
- **Best for**: Development, debugging, precise analysis

### 🔀 Lock-free (Multi-threaded)  
```rust
use memscope_rs::lockfree;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    lockfree::initialize_lockfree_tracking()?;
    
    // Spawn 100+ threads - scales efficiently
    // ... your multi-threaded code ...
    
    let analysis = lockfree::aggregate_all_threads()?;
    lockfree::export_analysis(&analysis, "analysis")?;
    Ok(())
}
```
- **Overhead**: ~2-8% (adaptive sampling)
- **Scalability**: 100+ threads with zero contention
- **Best for**: High-concurrency production applications

### ⚡ Async (Task-aware)
```rust
use memscope_rs::async_memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    async_memory::initialize().await?;
    
    // Track memory across async tasks
    // ... your async code ...
    
    let analysis = async_memory::generate_analysis().await?;
    async_memory::export_visualization(&analysis, "async_analysis").await?;
    Ok(())
}
```
- **Overhead**: <5ns per allocation
- **Context-aware**: Tracks memory per async task
- **Best for**: async/await applications, microservices

## 🔧 Advanced Usage

### Custom Configuration
```rust
use memscope_rs::lockfree::{TrackerConfig, SamplingMode};

let config = TrackerConfig {
    sampling_mode: SamplingMode::Adaptive,
    enable_advanced_analysis: true,
    max_threads: 50,
    ..Default::default()
};

lockfree::initialize_with_config(config)?;
```

### Performance Monitoring
```rust
// Check overhead impact
let stats = lockfree::get_performance_stats()?;
println!("CPU overhead: {:.2}%", stats.cpu_overhead_percent);
println!("Memory overhead: {}MB", stats.memory_overhead_mb);
```

## 📈 Performance Comparison

| Strategy | Memory Overhead | CPU Overhead | Concurrency | Best Use Case |
|----------|-----------------|--------------|-------------|---------------|
| **Core** | ~100 bytes/var | ~0% | Single thread | Development debugging |
| **Lock-free** | Thread-local | ~2-8% | 100+ threads | Production monitoring |
| **Async** | Task-local | <5ns/alloc | 50+ tasks | Async applications |

## 🛠️ Development Workflow

### 1. Choose Your Strategy
- **Debugging**: Start with `basic_usage.rs`
- **Multi-threading**: Try `complex_multithread_showcase.rs`  
- **Async**: Use `comprehensive_async_showcase.rs`
- **Interactive analysis**: Run `enhanced_30_thread_demo.rs`

### 2. Customize Configuration
- Adjust sampling rates for performance
- Enable/disable advanced features
- Configure export formats

### 3. Analyze Results
- Use HTML dashboards for interactive exploration
- JSON for integration with other tools
- Binary for high-performance scenarios

## 💡 Best Practices

### Performance
- **High-frequency allocations**: Enable adaptive sampling
- **Memory-constrained environments**: Use binary export
- **Real-time applications**: Monitor overhead continuously

### Development
- **Start simple**: Begin with single-threaded examples
- **Scale gradually**: Add complexity as needed
- **Test thoroughly**: Verify tracking overhead in your specific use case

### Production
- **Monitor overhead**: Set acceptable performance thresholds
- **Use sampling**: Reduce overhead with intelligent sampling
- **Automate cleanup**: Enable automatic file cleanup for long-running apps

## 🤝 Contributing

Want to add a new example?

1. **Follow naming convention**: `purpose_description.rs`
2. **Add comprehensive docs**: Explain use case and expected output
3. **Test across platforms**: Ensure compatibility
4. **Update this README**: Add your example to the table above

## 📚 Additional Resources

- **[API Documentation](https://docs.rs/memscope-rs)**: Complete API reference
- **[User Guide](../docs/user_guide.md)**: Detailed usage instructions  
- **[Performance Guide](../docs/performance.md)**: Optimization techniques
- **[GitHub Repository](https://github.com/TimWood0x10/memscope-rs)**: Source code and issues

---

**Happy memory tracking! 🚀🦀**