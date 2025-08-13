# memscope-rs Documentation

Welcome to memscope-rs documentation! A high-performance Rust memory analysis tool.

## 🌍 Language / 语言

Choose your preferred language:

### 📖 Documentation Languages

- **[🇨🇳 中文文档](zh/)** - 完整的中文文档
- **[🇺🇸 English Documentation](en/)** - Complete English documentation

## 🚀 Quick Start

```rust
use memscope_rs::{init, track_var, get_global_tracker};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize memory tracking
    init();
    
    // Track variables
    let data = vec![1, 2, 3, 4, 5];
    track_var!(data);
    
    // Export analysis results
    let tracker = get_global_tracker();
    tracker.export_to_html("memory_analysis.html")?;
    
    println!("Memory analysis complete! Check memory_analysis.html");
    Ok(())
}
```

## 📊 Features

- ✅ **Zero-overhead tracking** - Production-ready low-overhead memory tracking
- ✅ **Multiple export formats** - JSON, HTML, SVG, binary formats
- ✅ **Smart pointer support** - Automatic tracking of Rc, Arc, Box, etc.
- ✅ **Concurrency safe** - Multi-threaded program analysis support
- ✅ **Visual reports** - Interactive HTML reports and charts
- ✅ **CLI tools** - Powerful command-line analysis tools
- ✅ **High performance** - Binary format 80x faster than JSON

## 📚 Documentation Structure

Both Chinese and English documentation include:

- **Getting Started** - Installation, quick start, basic tracking
- **User Guide** - Tracking macros, analysis, export formats, CLI tools
- **API Reference** - Core types, tracking API, analysis API, export API
- **Examples** - Basic usage, concurrency, smart pointers, leak detection
- **Advanced** - Binary format, custom allocators, unsafe/FFI, async analysis

## 🔗 Links

- [GitHub Repository](https://github.com/your-org/memscope-rs)
- [API Documentation](https://docs.rs/memscope-rs)
- [Example Code](https://github.com/your-org/memscope-rs/tree/main/examples)
- [Issue Tracker](https://github.com/your-org/memscope-rs/issues)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.
