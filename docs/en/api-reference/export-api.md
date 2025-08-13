# Export API Reference

Data export functionality and configuration options in memscope-rs.

## Overview

This document describes the export API that provides multiple output formats (JSON, SVG, HTML, Binary) with comprehensive configuration options for memory analysis data export.

## Export Methods

### JSON Export

#### export_to_json

```rust
pub fn export_to_json(&self, base_name: &str) -> TrackingResult<()>
```

Export memory analysis to JSON format (5 categorized files).

**Module:** `memscope_rs::MemoryTracker`

**Source:** `src/core/tracker/export_json.rs`

**Parameters:**
- `base_name`: `&str` - Base name for output files

**Returns:** `TrackingResult<()>`

Generates 5 specialized JSON files:
- `{base_name}_memory_analysis.json` - Basic memory analysis
- `{base_name}_lifetime.json` - Variable lifecycle data
- `{base_name}_performance.json` - Performance metrics
- `{base_name}_unsafe_ffi.json` - Unsafe/FFI tracking
- `{base_name}_complex_types.json` - Complex type analysis

```rust
use memscope_rs::get_global_tracker;

let tracker = get_global_tracker();
tracker.export_to_json("my_analysis")?;
// Generates: MemoryAnalysis/my_analysis/*_*.json
```

#### export_to_json_with_options

```rust
pub fn export_to_json_with_options(
    &self,
    base_name: &str,
    options: ExportOptions,
) -> TrackingResult<()>
```

Export JSON with custom configuration options.

**Parameters:**
- `base_name`: `&str` - Base name for output files
- `options`: `ExportOptions` - Export configuration

```rust
use memscope_rs::{get_global_tracker, ExportOptions};

let options = ExportOptions::new()
    .include_system_allocations(false)
    .verbose_logging(true)
    .buffer_size(128 * 1024);

let tracker = get_global_tracker();
tracker.export_to_json_with_options("detailed_analysis", options)?;
```

### Binary Export

#### export_to_binary

```rust
pub fn export_to_binary(&self, base_name: &str) -> TrackingResult<()>
```

Export to high-performance binary format.

**Module:** `memscope_rs::MemoryTracker`

**Source:** `src/export/binary/mod.rs`

**Parameters:**
- `base_name`: `&str` - Base name for output file

**Returns:** `TrackingResult<()>`

```rust
let tracker = get_global_tracker();
tracker.export_to_binary("fast_export")?;
// Generates: MemoryAnalysis/fast_export/fast_export.memscope
```

#### export_to_binary_with_mode

```rust
pub fn export_to_binary_with_mode(
    &self,
    base_name: &str,
    mode: BinaryExportMode,
) -> TrackingResult<()>
```

Export binary with specific mode.

**Parameters:**
- `base_name`: `&str` - Base name for output file
- `mode`: `BinaryExportMode` - Export mode (UserOnly or Full)

```rust
use memscope_rs::BinaryExportMode;

let tracker = get_global_tracker();
tracker.export_to_binary_with_mode("user_only", BinaryExportMode::UserOnly)?;
tracker.export_to_binary_with_mode("complete", BinaryExportMode::Full)?;
```

### SVG Export

#### export_memory_analysis

```rust
pub fn export_memory_analysis(&self, filename: &str) -> TrackingResult<()>
```

Export memory usage visualization to SVG.

**Module:** `memscope_rs::MemoryTracker`

**Source:** `src/export/visualization.rs`

**Parameters:**
- `filename`: `&str` - Output SVG filename

```rust
let tracker = get_global_tracker();
tracker.export_memory_analysis("memory_chart.svg")?;
// Generates: MemoryAnalysis/memory_chart.svg
```

#### export_lifecycle_timeline

```rust
pub fn export_lifecycle_timeline(
    filename: &str,
    allocations: &[AllocationInfo],
) -> TrackingResult<()>
```

Export variable lifecycle timeline to SVG.

**Parameters:**
- `filename`: `&str` - Output filename
- `allocations`: `&[AllocationInfo]` - Allocations to visualize

```rust
use memscope_rs::export_lifecycle_timeline;

let allocations = tracker.get_active_allocations()?;
export_lifecycle_timeline("lifecycle.svg", &allocations)?;
```

### HTML Export

#### export_to_html

```rust
pub fn export_to_html(&self, filename: &str) -> TrackingResult<()>
```

Export interactive HTML dashboard.

**Module:** `memscope_rs::MemoryTracker`

**Source:** `src/export/html_export.rs`

**Parameters:**
- `filename`: `&str` - Output HTML filename

```rust
let tracker = get_global_tracker();
tracker.export_to_html("interactive_report.html")?;
// Generates: MemoryAnalysis/interactive_report.html
```

#### export_interactive_dashboard

```rust
pub fn export_interactive_dashboard(&self, filename: &str) -> TrackingResult<()>
```

Export enhanced interactive dashboard with advanced features.

```rust
let tracker = get_global_tracker();
tracker.export_interactive_dashboard("dashboard.html")?;
```

## Configuration Types

### ExportOptions

Main configuration struct for export operations.

**Module:** `memscope_rs::core::tracker::config`

**Source:** `src/core/tracker/config.rs`

```rust
pub struct ExportOptions {
    pub include_system_allocations: bool,
    pub verbose_logging: bool,
    pub buffer_size: usize,
    pub include_stack_traces: bool,
    pub include_lifecycle_data: bool,
    pub include_type_analysis: bool,
    pub compress_output: bool,
    pub max_entries: Option<usize>,
    pub filter_small_allocations: bool,
    pub min_allocation_size: usize,
    // ... more options
}
```

#### Methods

##### new

```rust
pub fn new() -> Self
```

Create new export options with default settings.

##### include_system_allocations

```rust
pub fn include_system_allocations(mut self, include: bool) -> Self
```

Set whether to include system allocations (slower but more complete).

##### verbose_logging

```rust
pub fn verbose_logging(mut self, verbose: bool) -> Self
```

Enable verbose logging during export.

##### buffer_size

```rust
pub fn buffer_size(mut self, size: usize) -> Self
```

Set buffer size for export operations.

##### include_stack_traces

```rust
pub fn include_stack_traces(mut self, include: bool) -> Self
```

Include stack trace information in export.

##### compress_output

```rust
pub fn compress_output(mut self, compress: bool) -> Self
```

Enable output compression.

##### max_entries

```rust
pub fn max_entries(mut self, max: Option<usize>) -> Self
```

Set maximum number of entries to export.

##### filter_small_allocations

```rust
pub fn filter_small_allocations(mut self, filter: bool) -> Self
```

Filter out small allocations.

##### min_allocation_size

```rust
pub fn min_allocation_size(mut self, size: usize) -> Self
```

Set minimum allocation size threshold.

#### Example Usage

```rust
use memscope_rs::ExportOptions;

// Fast export configuration
let fast_options = ExportOptions::new()
    .include_system_allocations(false)
    .verbose_logging(false)
    .buffer_size(64 * 1024)
    .compress_output(true);

// Detailed export configuration
let detailed_options = ExportOptions::new()
    .include_system_allocations(true)
    .verbose_logging(true)
    .include_stack_traces(true)
    .include_lifecycle_data(true)
    .buffer_size(256 * 1024);

// Filtered export configuration
let filtered_options = ExportOptions::new()
    .filter_small_allocations(true)
    .min_allocation_size(1024)
    .max_entries(Some(5000));
```

### BinaryExportMode

Export mode for binary format.

**Module:** `memscope_rs::core::tracker::memory_tracker`

**Source:** `src/core/tracker/memory_tracker.rs`

```rust
pub enum BinaryExportMode {
    /// Export only user-defined variables (faster, smaller files)
    UserOnly,
    /// Export all allocations including system allocations (complete data)
    Full,
}
```

### HtmlExportOptions

Configuration for HTML export.

```rust
pub struct HtmlExportOptions {
    pub theme: String,
    pub include_charts: bool,
    pub include_detailed_tables: bool,
    pub include_performance_metrics: bool,
    pub interactive_features: bool,
}
```

```rust
use memscope_rs::HtmlExportOptions;

let html_options = HtmlExportOptions::new()
    .with_theme("dark")
    .with_charts(true)
    .with_detailed_tables(true)
    .with_performance_metrics(true);

tracker.export_to_html_with_options("custom_report.html", &html_options)?;
```

## Binary Format Utilities

### Binary File Conversion

#### parse_binary_to_json

```rust
pub fn parse_binary_to_json(
    binary_file: &str,
    output_file: &str,
) -> TrackingResult<()>
```

Convert binary file to single JSON file.

**Module:** `memscope_rs::MemoryTracker`

```rust
use memscope_rs::MemoryTracker;

MemoryTracker::parse_binary_to_json("data.memscope", "output.json")?;
```

#### parse_binary_to_standard_json

```rust
pub fn parse_binary_to_standard_json(
    binary_file: &str,
    base_name: &str,
) -> TrackingResult<()>
```

Convert binary file to 5 categorized JSON files.

```rust
MemoryTracker::parse_binary_to_standard_json("data.memscope", "converted")?;
// Generates: converted_memory_analysis.json, converted_lifetime.json, etc.
```

#### parse_binary_to_html

```rust
pub fn parse_binary_to_html(
    binary_file: &str,
    output_file: &str,
) -> TrackingResult<()>
```

Convert binary file directly to HTML report.

```rust
MemoryTracker::parse_binary_to_html("data.memscope", "report.html")?;
```

### BinaryReader

Low-level binary file reader.

**Module:** `memscope_rs::export::binary`

**Source:** `src/export/binary/reader.rs`

```rust
use memscope_rs::BinaryReader;

let reader = BinaryReader::from_file("data.memscope")?;
let allocations = reader.read_allocations()?;
let stats = reader.read_stats()?;

// Convert to other formats
reader.export_to_json("converted.json")?;
reader.export_to_html("converted.html")?;
```

## Batch Export Operations

### export_all_formats

```rust
pub fn export_all_formats(&self, base_name: &str) -> TrackingResult<()>
```

Export to all supported formats.

```rust
let tracker = get_global_tracker();
tracker.export_all_formats("complete_analysis")?;
// Generates: JSON, SVG, HTML, and Binary files
```

### Batch Processing Example

```rust
use memscope_rs::{get_global_tracker, ExportOptions, BinaryExportMode};

fn batch_export_example() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = get_global_tracker();
    let base_name = "batch_analysis";
    
    // 1. Fast binary export
    tracker.export_to_binary_with_mode(base_name, BinaryExportMode::UserOnly)?;
    
    // 2. Detailed JSON export
    let detailed_options = ExportOptions::new()
        .include_system_allocations(true)
        .include_stack_traces(true)
        .verbose_logging(true);
    
    tracker.export_to_json_with_options(base_name, detailed_options)?;
    
    // 3. Visualization exports
    tracker.export_memory_analysis(&format!("{}.svg", base_name))?;
    tracker.export_interactive_dashboard(&format!("{}.html", base_name))?;
    
    println!("✅ Batch export complete!");
    Ok(())
}
```

## Performance Optimization

### Fast Export Configuration

```rust
// Optimized for speed
let fast_options = ExportOptions::new()
    .include_system_allocations(false)  // Skip system allocations
    .verbose_logging(false)             // Reduce logging overhead
    .buffer_size(256 * 1024)            // Large buffer
    .compress_output(false)             // Skip compression
    .max_entries(Some(10000));          // Limit entries

tracker.export_to_json_with_options("fast_export", fast_options)?;
```

### Memory-Efficient Export

```rust
// Optimized for memory usage
let memory_efficient = ExportOptions::new()
    .buffer_size(32 * 1024)             // Small buffer
    .filter_small_allocations(true)     // Filter small allocations
    .min_allocation_size(64)            // Skip tiny allocations
    .compress_output(true);             // Compress output

tracker.export_to_json_with_options("memory_efficient", memory_efficient)?;
```

## Complete Example

```rust
use memscope_rs::{
    init, track_var, get_global_tracker,
    ExportOptions, BinaryExportMode
};
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    // Create and track data
    let numbers = vec![1, 2, 3, 4, 5];
    track_var!(numbers);
    
    let shared = Rc::new(String::from("shared data"));
    track_var!(shared);
    
    let shared_clone = Rc::clone(&shared);
    track_var!(shared_clone);
    
    let tracker = get_global_tracker();
    
    // 1. Fast binary export (recommended for large datasets)
    tracker.export_to_binary_with_mode("example", BinaryExportMode::UserOnly)?;
    println!("✅ Binary export complete (fastest)");
    
    // 2. Standard JSON export (5 categorized files)
    tracker.export_to_json("example")?;
    println!("✅ JSON export complete");
    
    // 3. Custom JSON export with options
    let custom_options = ExportOptions::new()
        .include_stack_traces(true)
        .include_lifecycle_data(true)
        .verbose_logging(true);
    
    tracker.export_to_json_with_options("example_detailed", custom_options)?;
    println!("✅ Detailed JSON export complete");
    
    // 4. SVG visualization
    tracker.export_memory_analysis("example_chart.svg")?;
    println!("✅ SVG visualization complete");
    
    // 5. Interactive HTML dashboard
    tracker.export_interactive_dashboard("example_dashboard.html")?;
    println!("✅ HTML dashboard complete");
    
    // 6. Convert binary to other formats
    use memscope_rs::MemoryTracker;
    MemoryTracker::parse_binary_to_html("MemoryAnalysis/example/example.memscope", "from_binary.html")?;
    println!("✅ Binary to HTML conversion complete");
    
    println!("\n📁 Generated files:");
    println!("  - MemoryAnalysis/example/ (JSON files)");
    println!("  - MemoryAnalysis/example_chart.svg");
    println!("  - MemoryAnalysis/example_dashboard.html");
    println!("  - MemoryAnalysis/example/example.memscope");
    println!("  - from_binary.html");
    
    Ok(())
}
```

## Error Handling

### Common Export Errors

```rust
use memscope_rs::{get_global_tracker, TrackingError};

fn handle_export_errors() {
    let tracker = get_global_tracker();
    
    match tracker.export_to_json("analysis") {
        Ok(_) => println!("Export successful"),
        Err(TrackingError::ExportError(msg)) => {
            eprintln!("Export failed: {}", msg);
        }
        Err(TrackingError::IoError(msg)) => {
            eprintln!("File I/O error: {}", msg);
        }
        Err(TrackingError::SerializationError(msg)) => {
            eprintln!("Serialization error: {}", msg);
        }
        Err(e) => eprintln!("Other error: {}", e),
    }
}
```

## See Also

- [Core Types Reference](core-types.md) - Data structures and types
- [Tracking API Reference](tracking-api.md) - Memory tracking functions
- [Analysis API Reference](analysis-api.md) - Memory analysis functions
- [Export Formats Guide](../user-guide/export-formats.md) - Detailed format comparison