# Binary Export Format Guide

The binary format (`.memscope`) of memscope-rs is a high-performance memory data storage format designed for large-scale memory analysis.

## 🚀 Performance Advantages

### Real Performance Comparison

Based on actual test results from `advanced_metrics_demo` example:

| Metric | Binary Format | JSON Format | Performance Gain |
|--------|---------------|-------------|------------------|
| **Export Time** | 211ms | 17.1s | **80.72x faster** |
| **File Size** | 480KB | 728KB | **34.0% smaller** |
| **Memory Usage** | Low | High | Significantly lower |

### Why So Fast?

1. **Binary serialization** - Direct memory layout writing, no text conversion
2. **Compact format** - Optimized data structures, reduced redundancy
3. **Batch writing** - Fewer system calls
4. **No formatting overhead** - No JSON beautification or SVG rendering needed

## 📁 Basic Usage

### Export to Binary Format

```rust
use memscope_rs::{get_global_tracker, track_var, init};

fn main() {
    init();
    
    // Create some data
    let data = vec![1, 2, 3, 4, 5];
    track_var!(data);
    
    let tracker = get_global_tracker();
    
    // Export to binary format (recommended for large datasets)
    if let Err(e) = tracker.export_to_binary("my_analysis") {
        eprintln!("Binary export failed: {}", e);
    } else {
        println!("✅ Binary export successful");
        // File location: MemoryAnalysis/my_analysis/my_analysis.memscope
    }
}
```

### File Naming Rules

```
Input: "my_analysis"
Output: MemoryAnalysis/my_analysis/my_analysis.memscope
```

## 🔄 Format Conversion

### Binary → JSON Conversion

```rust
use memscope_rs::MemoryTracker;

// Convert to 5 categorized JSON files
MemoryTracker::parse_binary_to_standard_json(
    "data.memscope", 
    "converted_data"
)?;

// Generated files:
// - converted_data_memory_analysis.json
// - converted_data_lifetime.json
// - converted_data_performance.json
// - converted_data_unsafe_ffi.json
// - converted_data_complex_types.json
```

### Binary → Single JSON File

```rust
use memscope_rs::MemoryTracker;

// Convert to single JSON file
MemoryTracker::parse_binary_to_json(
    "data.memscope", 
    "single_output.json"
)?;
```

### Binary → HTML Report

```rust
use memscope_rs::MemoryTracker;

// Generate HTML report directly from binary
MemoryTracker::parse_binary_to_html(
    "data.memscope", 
    "report.html"
)?;
```

## 🎯 Real Usage Examples

### Example 1: High-Performance Data Export

```bash
# Run advanced example (generates large amount of data)
cargo run --example advanced_metrics_demo

# Check generated binary file
ls -la MemoryAnalysis/advanced_metrics_demo/
# -rw-r--r-- 1 user staff 480737 Aug  5 10:30 advanced_metrics_demo.memscope

# Convert to JSON for analysis
make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo
```

### Example 2: Binary Export Dedicated Example

```bash
# Run binary export example
cargo run --example binary_export_demo

# Check performance comparison results
# Binary vs Standard JSON Export Performance:
#   📊 Binary export time:     14ms
#   📊 Standard JSON time:     4.2s  
#   🚀 Speed improvement:      291.80x faster
#   📁 Binary file size:       480KB
#   📁 JSON files size:        728KB
#   💾 Size reduction:         34.0%
```

## 🔧 Advanced Usage

### Batch Conversion

```rust
use memscope_rs::MemoryTracker;
use std::fs;

fn batch_convert_binary_to_json(input_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension() == Some(std::ffi::OsStr::new("memscope")) {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let output_base = format!("{}_converted", stem);
            
            println!("Converting: {} → {}", path.display(), output_base);
            
            MemoryTracker::parse_binary_to_standard_json(&path, &output_base)?;
        }
    }
    
    Ok(())
}

// Usage
batch_convert_binary_to_json("MemoryAnalysis/")?;
```

### Performance Benchmarking

```rust
use memscope_rs::{get_global_tracker, track_var, init};
use std::time::Instant;

fn performance_comparison() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    // Create large amount of data
    for i in 0..1000 {
        let data = vec![i; 100];
        track_var!(data);
    }
    
    let tracker = get_global_tracker();
    
    // Test Binary export performance
    let start = Instant::now();
    tracker.export_to_binary("perf_test_binary")?;
    let binary_time = start.elapsed();
    
    // Test JSON export performance
    let start = Instant::now();
    tracker.export_to_json("perf_test_json")?;
    let json_time = start.elapsed();
    
    println!("Performance comparison:");
    println!("  Binary export: {:?}", binary_time);
    println!("  JSON export:   {:?}", json_time);
    println!("  Speed improvement: {:.2}x", json_time.as_nanos() as f64 / binary_time.as_nanos() as f64);
    
    Ok(())
}
```

## 📊 Format Specification

### File Structure

```
.memscope file structure:
┌─────────────────┐
│ Header          │ - Magic number, version, metadata
├─────────────────┤
│ String Table    │ - Deduplicated string data
├─────────────────┤
│ Allocation Records │ - Memory allocation information
├─────────────────┤
│ Statistics Data │ - Summary statistics
├─────────────────┤
│ Extended Data   │ - Advanced analysis data
└─────────────────┘
```

### Data Integrity

Binary format preserves all information:
- ✅ Variable names and type information
- ✅ Timestamps and thread IDs
- ✅ Memory addresses and sizes
- ✅ Lifecycle data
- ✅ Performance metrics
- ✅ Complex type analysis
- ✅ Unsafe/FFI tracking

## 🛠️ Troubleshooting

### Common Issues

1. **File Corruption**
   ```rust
   // Verify file integrity
   match MemoryTracker::parse_binary_to_json("data.memscope", "test.json") {
       Ok(_) => println!("File is intact"),
       Err(e) => println!("File may be corrupted: {}", e),
   }
   ```

2. **Version Compatibility**
   ```rust
   // Binary format is backward compatible
   // Newer versions can read older version files
   // But older versions cannot read newer version files
   ```

3. **Large File Handling**
   ```bash
   # For very large files, convert in batches
   # Use streaming processing to avoid memory issues
   ```

### Performance Tuning

```rust
// For large datasets, prioritize binary format
if data_size > 1_000_000 {
    tracker.export_to_binary("large_dataset")?;
} else {
    tracker.export_to_json("small_dataset")?;
}
```

## 🔗 Comparison with Other Formats

### Usage Scenario Recommendations

| Scenario | Recommended Format | Reason |
|----------|-------------------|--------|
| **Large Dataset Analysis** | Binary | 80x speed improvement |
| **Automated Processing** | Binary → JSON | Fast export first, convert as needed |
| **Interactive Analysis** | Binary → HTML | Direct visualization report generation |
| **Data Archiving** | Binary | Small files, good integrity |
| **Quick Debugging** | SVG | Immediate visualization |

### Recommended Workflow

```bash
# Recommended workflow
# 1. Development phase - Use Binary for fast export
cargo run --example your_program
# → Generates .memscope file

# 2. Analysis phase - Convert as needed
make html DIR=MemoryAnalysis/your_data BASE=your_data
# → Generates interactive HTML report

# 3. Data processing - Convert to JSON
MemoryTracker::parse_binary_to_standard_json("data.memscope", "analysis")
# → Generates 5 categorized JSON files
```

## 💡 Best Practices

### 1. Naming Conventions

```rust
// ✅ Use descriptive names
tracker.export_to_binary("user_session_analysis")?;
tracker.export_to_binary("performance_benchmark_2024")?;

// ❌ Avoid generic names
tracker.export_to_binary("data")?;
tracker.export_to_binary("test")?;
```

### 2. File Management

```bash
# Recommended directory structure
MemoryAnalysis/
├── daily_reports/
│   ├── 2024-08-05.memscope
│   └── 2024-08-06.memscope
├── benchmarks/
│   ├── baseline.memscope
│   └── optimized.memscope
└── debugging/
    ├── issue_123.memscope
    └── crash_analysis.memscope
```

### 3. Automation Scripts

```bash
#!/bin/bash
# Automated binary analysis script

BINARY_FILE="$1"
OUTPUT_NAME="$2"

if [ -z "$BINARY_FILE" ] || [ -z "$OUTPUT_NAME" ]; then
    echo "Usage: $0 <binary_file> <output_name>"
    exit 1
fi

echo "🔄 Converting binary file: $BINARY_FILE"

# Convert to JSON
echo "Generating JSON files..."
./target/release/memscope-rs parse-binary-to-json "$BINARY_FILE" "${OUTPUT_NAME}.json"

# Generate HTML report
echo "Generating HTML report..."
./target/release/memscope-rs parse-binary-to-html "$BINARY_FILE" "${OUTPUT_NAME}.html"

echo "✅ Conversion complete!"
echo "📄 JSON: ${OUTPUT_NAME}.json"
echo "🌐 HTML: ${OUTPUT_NAME}.html"
```

## 🎉 Summary

Binary format is one of the core advantages of memscope-rs:

✅ **Ultimate Performance** - 80x faster than JSON  
✅ **Space Efficiency** - 34% space savings  
✅ **Complete Data** - Preserves all analysis information  
✅ **Flexible Conversion** - Can convert to any other format  
✅ **Backward Compatibility** - Worry-free version upgrades  

For any scenario requiring high-performance memory analysis, binary format is the best choice! 🚀