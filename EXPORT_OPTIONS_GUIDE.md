# JSON Export Options Guide

## Overview

The memscope-rs library provides flexible JSON export options that let you control the balance between **performance** and **data completeness**. By default, the export is optimized for speed and focuses on user-tracked variables.

## Quick Start

### Default Export (Recommended)
```rust
use memscope_rs::get_global_tracker;

let tracker = get_global_tracker();

// Fast export - only user-tracked variables get full enrichment
tracker.export_to_json("my_analysis")?;
```

**Result**: 4 files created in ~2-5 seconds
- `my_analysis_memory_analysis.json`
- `my_analysis_lifetime.json` 
- `my_analysis_unsafe_ffi.json`
- `my_analysis_variable_relationships.json`

### Complete Export (Debug Mode)
```rust
use memscope_rs::{get_global_tracker, ExportOptions};

let tracker = get_global_tracker();

// ⚠️ SLOW export - ALL allocations get full enrichment
let options = ExportOptions::new().include_system_allocations(true);
tracker.export_to_json_with_options("debug_analysis", options)?;
```

**Result**: 4 files created in ~10-40 seconds (much larger files)

## Export Options Reference

### `ExportOptions` Structure

```rust
pub struct ExportOptions {
    /// Include system allocations in full enrichment (default: false)
    pub include_system_allocations: bool,
    
    /// Enable verbose logging during export (default: false)
    pub verbose_logging: bool,
    
    /// Buffer size for file I/O in bytes (default: 64KB)
    pub buffer_size: usize,
    
    /// Enable data compression (default: false)
    pub compress_output: bool,
}
```

### Option Details

#### `include_system_allocations`

**Default**: `false` (recommended for most users)

| Setting | Performance | Data Scope | Use Case |
|---------|-------------|------------|----------|
| `false` | **Fast** (~2-5 sec) | User variables only | Development, HTML rendering, production |
| `true` | **Slow** (~10-40 sec) | All allocations | Deep debugging, memory leak investigation |

**⚠️ Performance Impact**: Setting to `true` makes export **5-10x slower** and generates **much larger files**.

#### `verbose_logging`

**Default**: `false`

```rust
let options = ExportOptions::new().verbose_logging(true);
```

Enables detailed progress logging during export:
```
📊 Processing user allocations: 1000/5000 (20%)
🔍 Enriching variable: user_vector (Vec<i32>)
💾 Writing memory_analysis.json: 2.5MB
```

#### `buffer_size`

**Default**: `64KB`

```rust
let options = ExportOptions::new().buffer_size(128 * 1024); // 128KB
```

Controls file I/O buffer size. Larger buffers may improve performance for very large datasets.

#### `compress_output`

**Default**: `false` (experimental)

```rust
let options = ExportOptions::new().compress_output(true);
```

Enables output compression to reduce file sizes (experimental feature).

## Usage Examples

### Example 1: Fast Development Export
```rust
// For normal development and HTML rendering
tracker.export_to_json("app_memory_snapshot")?;
```

### Example 2: Verbose Fast Export
```rust
let options = ExportOptions::new().verbose_logging(true);
tracker.export_to_json_with_options("verbose_snapshot", options)?;
```

### Example 3: Complete Debug Export
```rust
// For deep debugging - includes ALL system allocations
let options = ExportOptions::new()
    .include_system_allocations(true)
    .verbose_logging(true);
    
tracker.export_to_json_with_options("complete_debug", options)?;
```

### Example 4: High-Performance Export
```rust
// Optimized for large datasets
let options = ExportOptions::new()
    .buffer_size(256 * 1024)  // 256KB buffer
    .compress_output(true);   // Reduce file size
    
tracker.export_to_json_with_options("optimized_export", options)?;
```

## Performance Comparison

### Typical Dataset (5000 allocations)

| Mode | Time | File Size | User Data | System Data |
|------|------|-----------|-----------|-------------|
| **Default** | 2-5 sec | 2-5 MB | ✅ Full | ⚡ Minimal |
| **Complete** | 10-40 sec | 10-50 MB | ✅ Full | ✅ Full |

### Data Quality Comparison

#### Default Mode (`include_system_allocations: false`)
```json
{
  "active_allocations": [
    {
      "var_name": "user_vector",           // ✅ Real name
      "type_name": "Vec<i32>",            // ✅ Accurate type
      "size": 1024,
      "scope_name": "main_function",      // ✅ Real scope
      "stack_trace": ["user_code", "..."] // ✅ Meaningful trace
    },
    {
      "var_name": "system_alloc",         // ⚡ Simple name
      "type_name": "system",              // ⚡ Generic type
      "size": 8,
      "scope_name": "system",             // ⚡ Generic scope
      "stack_trace": ["system"]           // ⚡ Minimal trace
    }
  ]
}
```

#### Complete Mode (`include_system_allocations: true`)
```json
{
  "active_allocations": [
    {
      "var_name": "user_vector",                    // ✅ Real name
      "type_name": "Vec<i32>",                     // ✅ Accurate type
      "size": 1024,
      "scope_name": "main_function",               // ✅ Real scope
      "stack_trace": ["user_code", "..."]          // ✅ Meaningful trace
    },
    {
      "var_name": "std_internal_buffer_0x7ff123", // ✅ Detailed name
      "type_name": "std::collections::HashMap",   // ✅ Accurate type
      "size": 8,
      "scope_name": "std::collections",           // ✅ Real scope
      "stack_trace": ["std::collections::...", "..."] // ✅ Full trace
    }
  ]
}
```

## When to Use Each Mode

### Use Default Mode When:
- ✅ Normal development and testing
- ✅ HTML rendering and visualization
- ✅ Production monitoring
- ✅ Performance is important
- ✅ You care about user variables only

### Use Complete Mode When:
- 🐛 Debugging memory leaks
- 🔍 Investigating system-level issues
- 📊 Analyzing library/framework memory usage
- 🧪 Research and detailed analysis
- ⚠️ You can tolerate slow export times

## Best Practices

### 1. Start with Default Mode
Always start with the default fast mode:
```rust
tracker.export_to_json("analysis")?;
```

### 2. Use Complete Mode Sparingly
Only enable system allocation enrichment when you specifically need it:
```rust
// Only for debugging
let debug_options = ExportOptions::new().include_system_allocations(true);
tracker.export_to_json_with_options("debug", debug_options)?;
```

### 3. Enable Verbose Logging for Long Exports
For complete mode exports, enable verbose logging to track progress:
```rust
let options = ExportOptions::new()
    .include_system_allocations(true)
    .verbose_logging(true);
```

### 4. Consider File Sizes
Complete mode can generate very large files. Ensure you have adequate disk space:
- Default mode: ~2-5 MB for typical applications
- Complete mode: ~10-50 MB for the same application

## Troubleshooting

### Export is Too Slow
```rust
// Switch to default mode
tracker.export_to_json("fast_export")?;
```

### Missing System Data
```rust
// Enable complete mode (accept slower performance)
let options = ExportOptions::new().include_system_allocations(true);
tracker.export_to_json_with_options("complete_export", options)?;
```

### Large File Sizes
```rust
// Enable compression
let options = ExportOptions::new().compress_output(true);
tracker.export_to_json_with_options("compressed_export", options)?;
```

### Want Progress Updates
```rust
// Enable verbose logging
let options = ExportOptions::new().verbose_logging(true);
tracker.export_to_json_with_options("verbose_export", options)?;
```

## API Reference

### Methods

#### `export_to_json(path)`
Default fast export - recommended for most users.

#### `export_to_json_with_options(path, options)`
Custom export with user-specified options.

### Types

#### `ExportOptions`
Configuration structure for export behavior.

#### `ExportOptions::new()`
Create new options with default settings (fast mode).

#### `ExportOptions::default()`
Same as `new()` - creates fast mode options.

---

**💡 Recommendation**: Use the default `export_to_json()` method for 95% of use cases. Only use `include_system_allocations: true` when you specifically need to debug system-level memory issues.