# Export Formats Guide

memscope-rs supports multiple export formats, each with specific purposes and advantages. This guide will help you choose the most suitable export method.

## 📊 Format Comparison

| Format | File Size | Generation Speed | Interactivity | Use Case |
|--------|-----------|------------------|---------------|----------|
| **JSON** | Medium | Medium | None | Data analysis, automation |
| **SVG** | Small | Fast | Basic | Report embedding, static visualization |
| **HTML** | Large | Slow | High | Interactive analysis, presentations |
| **Binary** | Smallest | **Fastest** | None | Large datasets, performance-critical |

### Performance Comparison (Real Test Data)

Based on actual test results from `advanced_metrics_demo` example:

- **Binary export**: 211ms, 480KB file
- **JSON export**: 17.1s, 728KB files (5 categorized files)
- **Speed improvement**: Binary is **80.72x faster** than JSON
- **Space savings**: Binary saves **34.0%** space compared to JSON

## 📄 JSON Export - Data Analysis First Choice

### Features
- **Categorized data** - 5 specialized JSON files for analyzing different aspects
- **Structured data** - Complete memory allocation information
- **Machine readable** - Easy for automated analysis and processing
- **Standard format** - Can integrate with other tools

### 5 Categorized Files

JSON export generates 5 specialized files:

1. **`*_memory_analysis.json`** - Basic memory analysis data
2. **`*_lifetime.json`** - Variable lifecycle information
3. **`*_performance.json`** - Performance-related data
4. **`*_unsafe_ffi.json`** - Unsafe code and FFI tracking
5. **`*_complex_types.json`** - Complex type analysis

### Basic Usage
```rust
use memscope_rs::track_var;

fn main() {
    let memscope = memscope_rs::MemScope::new();

    let data = vec![1, 2, 3, 4, 5];
    track_var!(data);

    // Export to JSON (generates 5 categorized files)
    if let Err(e) = memscope.export_json("my_analysis") {
        eprintln!("Export failed: {}", e);
    } else {
        println!("✅ JSON export successful");
        // Files location: MemoryAnalysis/my_analysis/
        // - my_analysis_memory_analysis.json
        // - my_analysis_lifetime.json
        // - my_analysis_performance.json
        // - my_analysis_unsafe_ffi.json
        // - my_analysis_complex_types.json
    }
}
```

### JSON Data Structure
```json
{
  "metadata": {
    "export_timestamp": 1691234567890,
    "export_version": "0.1.10",
    "total_allocations": 3,
    "active_allocations": 3,
    "peak_memory": 1024
  },
  "memory_stats": {
    "active_allocations": 3,
    "active_memory": 512,
    "total_allocations": 3,
    "total_deallocations": 0,
    "peak_memory": 512,
    "peak_allocations": 3
  },
  "allocations": [
    {
      "ptr": 140712345678912,
      "size": 40,
      "var_name": "data",
      "type_name": "Vec<i32>",
      "timestamp_alloc": 1691234567123,
      "thread_id": "ThreadId(1)",
      "is_leaked": false
    }
  ],
  "analysis": {
    "fragmentation_analysis": {...},
    "circular_references": [...],
    "unsafe_ffi_stats": {...}
  }
}
```

## 🎨 SVG Export - Static Visualization

### Features
- **Vector graphics** - Scalable without quality loss
- **Lightweight** - Small files, fast loading
- **Embed-friendly** - Can be directly embedded in web pages and documents

### Basic Usage
```rust
use memscope_rs::track_var;

fn main() {
    let memscope = memscope_rs::MemScope::new();

    // Create some interesting memory patterns
    let vec1 = vec![1; 100];
    track_var!(vec1);

    let vec2 = vec![2; 200];
    track_var!(vec2);

    let boxed = Box::new(vec![3; 50]);
    track_var!(boxed);

    // Export memory usage chart
    if let Err(e) = memscope.export_svg("memory_chart.svg") {
        eprintln!("SVG export failed: {}", e);
    } else {
        println!("✅ SVG export successful");
        // File location: MemoryAnalysis/memory_chart.svg
    }
}
```

## 🌐 HTML Export - Interactive Dashboard

### Features
- **Interactive** - Clickable, filterable, zoomable
- **Real-time analysis** - Dynamic calculation and display
- **Beautiful interface** - Professional data visualization

### Basic Usage
```rust
use memscope_rs::track_var;
use std::rc::Rc;

fn main() {
    let memscope = memscope_rs::MemScope::new();

    // Create complex memory scenarios
    let data1 = vec![1; 1000];
    track_var!(data1);

    let shared = Rc::new(String::from("shared data"));
    track_var!(shared);

    let clone1 = Rc::clone(&shared);
    track_var!(clone1);

    // Export interactive HTML dashboard
    if let Err(e) = memscope.export_html("interactive_report.html") {
        eprintln!("HTML export failed: {}", e);
    } else {
        println!("✅ HTML export successful");
        println!("Open in browser: MemoryAnalysis/interactive_report.html");
    }
}
```

### Using make Command (Recommended)
```bash
# 1. First run program to generate JSON data
cargo run --example your_program

# 2. Use make command to generate enhanced HTML report
make html DIR=MemoryAnalysis/your_analysis BASE=your_analysis

# 3. Open generated report
open memory_report.html
```

This method generates HTML reports with richer functionality and more interactive charts.

### HTML Dashboard Features

**Memory Overview**
- Real-time memory statistics
- Allocation trend charts
- Type distribution pie charts

**Detailed Analysis**
- Filterable allocation lists
- Smart pointer relationship graphs
- Memory leak detection results

**Interactive Features**
- Click to view detailed information
- Filter by type/thread/time
- Zoom and pan charts

## ⚡ Binary Export - High Performance Choice

### Features
- **Smallest files** - Compact binary format (saves 34% space)
- **Fastest speed** - 80x faster export performance than JSON
- **Complete data** - Preserves all analysis information
- **Convertible** - Can convert to JSON or HTML formats

### Basic Usage
```rust
use memscope_rs::track_var;

fn main() {
    let memscope = memscope_rs::MemScope::new();

    // Large dataset scenario
    for i in 0..1000 {
        let data = vec![i; 100];
        track_var!(data);
    }

    // Export binary format (.memscope extension)
    if let Err(e) = memscope.export_binary("large_dataset") {
        eprintln!("Binary export failed: {}", e);
    } else {
        println!("✅ Binary export successful");
        // File location: MemoryAnalysis/large_dataset/large_dataset.memscope
    }
}
```

## 🔧 Batch Export

### Export All Formats
```rust
use memscope_rs::track_var;

fn export_all_formats(base_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let memscope = memscope_rs::MemScope::new();

    // ... tracking code ...

    // JSON data
    memscope.export_json(base_name)?;

    // SVG charts
    memscope.export_svg(&format!("{}.svg", base_name))?;

    // HTML dashboard
    memscope.export_html(&format!("{}.html", base_name))?;

    // Binary data
    memscope.export_binary(&format!("{}.memscope", base_name))?;

    println!("✅ All formats exported successfully");
    Ok(())
}

// Usage
export_all_formats("complete_analysis")?;
```

## 📁 File Organization

### Default Directory Structure
```
MemoryAnalysis/
├── my_analysis/
│   ├── my_analysis_memory_analysis.json
│   ├── my_analysis.svg
│   ├── my_analysis.html
│   └── my_analysis.memscope
├── performance_test/
│   └── ...
└── debug_session/
    └── ...
```

## 🎯 Usage Recommendations

### Development Phase
```rust
// Quick iteration - use SVG
memscope.export_svg("debug.svg")?;
```

### Detailed Analysis
```rust
// Deep analysis - use HTML
memscope.export_html("detailed_analysis.html")?;
```

### Automated Processing
```rust
// Data processing - use JSON
memscope.export_json("automated_analysis")?;
```

### Performance Critical
```rust
// Large datasets - use Binary
memscope.export_binary("performance_data.memscope")?;
```

Choose the right export format for efficient memory analysis! 🚀