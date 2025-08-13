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
use memscope_rs::{get_global_tracker, track_var, init};

fn main() {
    init();
    
    let data = vec![1, 2, 3, 4, 5];
    track_var!(data);
    
    let tracker = get_global_tracker();
    
    // Export to JSON (generates 5 categorized files)
    if let Err(e) = tracker.export_to_json("my_analysis") {
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
    "export_version": "0.1.4",
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

### Custom JSON Export
```rust
use memscope_rs::{get_global_tracker, ExportOptions};

let tracker = get_global_tracker();
let options = ExportOptions::new()
    .include_system_allocations(true);  // Include system allocations (slow but detailed)

// Note: Including system allocations significantly reduces performance (5-10x slower)
tracker.export_to_json_with_options("detailed_analysis", options)?;
```

### Performance Mode Selection

```rust
// Fast mode (recommended) - only tracks user variables
tracker.export_to_json("fast_analysis")?;

// Detailed mode - includes all system allocations (slow)
let detailed_options = ExportOptions::new()
    .include_system_allocations(true);
tracker.export_to_json_with_options("detailed_analysis", detailed_options)?;
```

## 🎨 SVG Export - Static Visualization

### Features
- **Vector graphics** - Scalable without quality loss
- **Lightweight** - Small files, fast loading
- **Embed-friendly** - Can be directly embedded in web pages and documents

### Basic Usage
```rust
use memscope_rs::{get_global_tracker, track_var, init};

fn main() {
    init();
    
    // Create some interesting memory patterns
    let vec1 = vec![1; 100];
    track_var!(vec1);
    
    let vec2 = vec![2; 200];
    track_var!(vec2);
    
    let boxed = Box::new(vec![3; 50]);
    track_var!(boxed);
    
    let tracker = get_global_tracker();
    
    // Export memory usage chart
    if let Err(e) = tracker.export_memory_analysis("memory_chart.svg") {
        eprintln!("SVG export failed: {}", e);
    } else {
        println!("✅ SVG export successful");
        // File location: MemoryAnalysis/memory_chart.svg
    }
}
```

### SVG Chart Types

**Memory Usage Timeline**
```rust
// Generate memory usage over time chart
tracker.export_memory_timeline("timeline.svg")?;
```

**Allocation Type Distribution**
```rust
// Generate memory distribution chart by type
tracker.export_type_distribution("distribution.svg")?;
```

**Lifecycle Analysis**
```rust
// Generate variable lifecycle visualization
use memscope_rs::export_lifecycle_timeline;
export_lifecycle_timeline("lifecycle.svg", &allocations)?;
```

## 🌐 HTML Export - Interactive Dashboard

### Features
- **Interactive** - Clickable, filterable, zoomable
- **Real-time analysis** - Dynamic calculation and display
- **Beautiful interface** - Professional data visualization
- **Two methods** - Direct export or via make command

### Method 1: Direct HTML Export
```rust
use memscope_rs::{get_global_tracker, track_var, init};
use std::rc::Rc;

fn main() {
    init();
    
    // Create complex memory scenarios
    let data1 = vec![1; 1000];
    track_var!(data1);
    
    let shared = Rc::new(String::from("shared data"));
    track_var!(shared);
    
    let clone1 = Rc::clone(&shared);
    track_var!(clone1);
    
    let tracker = get_global_tracker();
    
    // Export interactive HTML dashboard
    if let Err(e) = tracker.export_interactive_dashboard("interactive_report.html") {
        eprintln!("HTML export failed: {}", e);
    } else {
        println!("✅ HTML export successful");
        println!("Open in browser: MemoryAnalysis/interactive_report.html");
    }
}
```

### Method 2: Using make Command (Recommended)
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

### Custom HTML Themes
```rust
use memscope_rs::HtmlExportOptions;

let html_options = HtmlExportOptions::new()
    .with_theme("dark")              // Dark theme
    .with_charts(true)               // Include charts
    .with_detailed_tables(true)      // Detailed tables
    .with_performance_metrics(true); // Performance metrics

tracker.export_to_html_with_options("custom_report.html", &html_options)?;
```

## ⚡ Binary Export - High Performance Choice

### Features
- **Smallest files** - Compact binary format (saves 34% space)
- **Fastest speed** - 80x faster export performance than JSON
- **Complete data** - Preserves all analysis information
- **Convertible** - Can convert to JSON or HTML formats

### Basic Usage
```rust
use memscope_rs::{get_global_tracker, track_var, init};

fn main() {
    init();
    
    // Large dataset scenario
    for i in 0..1000 {
        let data = vec![i; 100];
        track_var!(data);
    }
    
    let tracker = get_global_tracker();
    
    // Export binary format (.memscope extension)
    if let Err(e) = tracker.export_to_binary("large_dataset") {
        eprintln!("Binary export failed: {}", e);
    } else {
        println!("✅ Binary export successful");
        // File location: MemoryAnalysis/large_dataset/large_dataset.memscope
    }
}
```

### Binary → JSON Conversion
```rust
use memscope_rs::MemoryTracker;

// Convert binary file to standard 5 JSON files
MemoryTracker::parse_binary_to_standard_json(
    "data.memscope", 
    "converted_data"
)?;

// Or convert to single JSON file
MemoryTracker::parse_binary_to_json(
    "data.memscope", 
    "single_file.json"
)?;
```

### Binary → HTML Conversion
```rust
use memscope_rs::MemoryTracker;

// Generate HTML report directly from binary
MemoryTracker::parse_binary_to_html(
    "data.memscope", 
    "report.html"
)?;
```

### Binary Format Configuration
```rust
use memscope_rs::BinaryExportConfig;

let config = BinaryExportConfig::new()
    .with_compression(true)          // Enable compression
    .with_string_deduplication(true) // String deduplication
    .with_fast_mode(true);           // Fast mode

tracker.export_to_binary_with_config("optimized.memscope", &config)?;
```

### Reading Binary Files
```rust
use memscope_rs::BinaryReader;

// Read binary file
let reader = BinaryReader::from_file("data.memscope")?;
let allocations = reader.read_allocations()?;
let stats = reader.read_stats()?;

// Convert to other formats
reader.export_to_json("converted.json")?;
reader.export_to_html("converted.html")?;
```

## 🔧 Batch Export

### Export All Formats
```rust
use memscope_rs::{get_global_tracker, ExportOptions};

fn export_all_formats(base_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tracker = get_global_tracker();
    
    // JSON data
    tracker.export_to_json(base_name)?;
    
    // SVG charts
    tracker.export_memory_analysis(&format!("{}.svg", base_name))?;
    
    // HTML dashboard
    tracker.export_to_html(&format!("{}.html", base_name))?;
    
    // Binary data
    tracker.export_to_binary(&format!("{}.memscope", base_name))?;
    
    println!("✅ All formats exported successfully");
    Ok(())
}

// Usage
export_all_formats("complete_analysis")?;
```

### Performance Optimized Export
```rust
use memscope_rs::ExportOptions;

// Fast export (suitable for large datasets)
let fast_options = ExportOptions::new()
    .with_fast_mode(true)
    .with_minimal_analysis(true)
    .with_compression(true);

tracker.export_to_json_with_options("fast_export", &fast_options)?;

// Detailed export (suitable for deep analysis)
let detailed_options = ExportOptions::new()
    .with_detailed_analysis(true)
    .with_stack_traces(true)
    .with_thread_info(true)
    .with_circular_reference_detection(true);

tracker.export_to_json_with_options("detailed_export", &detailed_options)?;
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

### Custom Output Directory
```rust
use memscope_rs::ExportOptions;

let options = ExportOptions::new()
    .with_output_directory("custom_reports")
    .with_create_subdirectory(false);

tracker.export_to_json_with_options("analysis", &options)?;
// Output to: custom_reports/analysis_memory_analysis.json
```

## 🎯 Usage Recommendations

### Development Phase
```rust
// Quick iteration - use SVG
tracker.export_memory_analysis("debug.svg")?;
```

### Detailed Analysis
```rust
// Deep analysis - use HTML
tracker.export_to_html("detailed_analysis.html")?;
```

### Automated Processing
```rust
// Data processing - use JSON
tracker.export_to_json("automated_analysis")?;
```

### Performance Critical
```rust
// Large datasets - use Binary
tracker.export_to_binary("performance_data.memscope")?;
```

Choose the right export format for efficient memory analysis! 🚀