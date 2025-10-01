# memscope-rs Documentation

Welcome to memscope-rs - A Rust memory tracking and analysis library!

## 🚀 Quick Start

If you're new to memscope-rs, we recommend reading in this order:

1. **[Core Modules Overview](modules/README.md)** - Understand the four tracking modules
2. **[Installation](getting-started/installation.md)** - Project integration and environment setup
3. **[5-Minute Quick Start](getting-started/quick-start.md)** - Get started immediately
4. **[Basic Tracking](getting-started/basic-tracking.md)** - Learn core functionality
5. **[First Analysis](getting-started/first-analysis.md)** - Generate your first analysis report

## 📚 Documentation Navigation

### Getting Started

- [Installation](getting-started/installation.md) - Project integration and environment setup
- [Quick Start](getting-started/quick-start.md) - 5-minute getting started guide
- [Basic Tracking](getting-started/basic-tracking.md) - Core tracking functionality
- [First Analysis](getting-started/first-analysis.md) - Generate analysis reports

### Core Modules

- **[Core Modules Overview](modules/README.md)** - Complete comparison of all four modules
- **[Single-threaded Module](modules/single-threaded.md)** - Zero-overhead `track_var!` macros
- **[Multi-threaded Module](modules/multithread.md)** - Lock-free high-concurrency tracking
- **[Async Module](modules/async.md)** - Task-centric memory analysis
- **[Hybrid Module](modules/hybrid.md)** - Comprehensive cross-module analysis

### User Guide

- [Tracking Macros](user-guide/tracking-macros.md) - Usage of `track_var!` series macros
- [Memory Analysis](user-guide/memory-analysis.md) - Analysis features and report interpretation
- [Export Formats](user-guide/export-formats.md) - JSON/SVG/HTML/Binary export
- [CLI Tools](user-guide/cli-tools.md) - CLI tools usage guide
- [Configuration](user-guide/configuration.md) - Detailed configuration options
- [Troubleshooting](user-guide/troubleshooting.md) - Problem diagnosis and solutions

### Advanced Features

- [Custom Allocator](advanced/custom-allocator.md) - TrackingAllocator deep dive
- [Unsafe/FFI Tracking](advanced/unsafe-ffi-tracking.md) - FFI and unsafe code analysis
- [Async Analysis](advanced/async-analysis.md) - async/await memory pattern analysis
- [Binary Format](advanced/binary-format.md) - High-performance binary export
- [Performance Optimization](advanced/performance-optimization.md) - Best practices and optimization tips
- [Extending Analysis](advanced/extending-analysis.md) - Custom analyzers

### Examples and Tutorials

- [Basic Usage](examples/basic-usage.md) - Detailed explanation from examples/basic_usage.rs
- [Smart Pointer Tracking](examples/smart-pointers.md) - Rc/Arc/Box tracking techniques
- [Concurrent Analysis](examples/concurrent-analysis.md) - Multi-threaded memory analysis
- [Memory Leak Detection](examples/memory-leak-detection.md) - Leak detection and prevention
- [Performance Profiling](examples/performance-profiling.md) - Performance bottleneck identification
- [Integration Examples](examples/integration-examples.md) - Integration into existing projects

### API Reference

- [Core Types](api-reference/core-types.md) - AllocationInfo, MemoryStats, etc.
- [Tracking API](api-reference/tracking-api.md) - MemoryTracker and related interfaces
- [Analysis API](api-reference/analysis-api.md) - Analyzers and analysis results
- [Export API](api-reference/export-api.md) - Export functionality and configuration
- [CLI API](api-reference/cli-api.md) - Command-line interface reference

## 🎯 Common Scenarios

### I want to

**Quickly check memory usage** → [Quick Start](getting-started/quick-start.md)

**Analyze multi-threaded programs** → [Concurrent Analysis](examples/concurrent-analysis.md)

**Use high-performance export** → [Export Formats](user-guide/export-formats.md)

**Generate HTML reports** → [CLI Tools](user-guide/cli-tools.md)

**Understand binary format** → [Binary Format](advanced/binary-format.md)

**Integrate into existing project** → [Basic Usage](examples/basic-usage.md)

**Run actual examples** → Check `examples/` directory

### 🚀 Try It Now

```bash
# Run basic example
cargo run --example basic_usage

# Run advanced multi-threaded example  
cargo run --example advanced_metrics_demo

# Run binary export example
cargo run --example binary_export_demo

# Generate HTML report
make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo
```

## 📊 Performance Reference

### Tracking Performance

| Tracking Method | Performance Overhead | Use Case |
|----------------|---------------------|----------|
| `track_var!` | Zero overhead | Production monitoring |
| `track_var_smart!` | Minimal overhead | Mixed type tracking |
| `track_var_owned!` | Wrapper overhead | Precise lifecycle analysis |

### Export Performance (Real Test Data)

| Export Format | Export Time | File Size | Speed Comparison |
|--------------|-------------|-----------|------------------|
| **Binary** | 211ms | 480KB | Baseline |
| **JSON** | 17.1s | 728KB | 80.72x slower |
| **HTML** | 1.3s | 1.2MB | 6.15x slower |

*Based on actual test results from `advanced_metrics_demo` example*

## 🤝 Getting Help

- **Documentation Issues**: Check [Troubleshooting](user-guide/troubleshooting.md)
- **Feature Requests**: Submit GitHub Issue
- **Bug Reports**: Provide minimal reproduction example
- **Contributing Code**: See [Contributing Guide](contributing/development-setup.md)

## 🔧 Tools and Utilities

- **Online Documentation**: [docs.rs/memscope-rs](https://docs.rs/memscope-rs)
- **Source Repository**: [GitHub](https://github.com/TimWood0x10/memscope-rs)
- **Example Code**: Check `examples/` directory
- **Test Cases**: Check `tests/` directory for more usage patterns

---

**Start your memory analysis journey** → [5-Minute Quick Start](getting-started/quick-start.md) 🚀