# JSON Export Optimization

## Overview

This document explains the JSON export optimization implemented in the memscope-rs project. The optimization addresses the issue of long JSON generation times by splitting the export into multiple specialized files and implementing various performance improvements.

## Implementation Details

### 1. Optimized Export Architecture

The optimized JSON export splits the data into four specialized files:

- **variable_relationships.json**: Contains variable relationship graph data
- **memory_analysis.json**: Contains memory usage analysis data
- **lifetime_analysis.json**: Contains object lifecycle analysis data
- **unsafe_ffi_analysis.json**: Contains unsafe code and FFI analysis data

### 2. Performance Optimizations

- **Parallel Processing**: Uses Rayon to process and generate each file in parallel
- **Type Inference Caching**: Caches type inference results to avoid redundant calculations
- **Batch Processing**: Processes allocations in batches to reduce memory pressure
- **Optimized Data Structures**: Uses specialized data structures for each file type

### 3. Backward Compatibility

The original `export_to_json` method has been completely reimplemented to use the optimized export internally while maintaining backward compatibility:

- It combines the separate JSON files into one comprehensive file
- This approach is much faster than the old implementation because it uses optimized data generation
- All existing code that calls `export_to_json` will automatically benefit from the optimization

## Usage

### Standard Usage (Backward Compatible)

```rust
// This now uses the optimized implementation internally
tracker.export_to_json("output.json");
```

### Direct Access to Separated Files

```rust
// Get direct access to the separated files and performance metrics
let result = memscope_rs::export_separated_json_simple(&tracker, "output");

// Access the file paths
println!("Variable relationships: {}", result.variable_relationships_path.display());
println!("Memory analysis: {}", result.memory_analysis_path.display());
println!("Lifetime analysis: {}", result.lifetime_analysis_path.display());
println!("Unsafe/FFI analysis: {}", result.unsafe_ffi_analysis_path.display());

// Access performance metrics
println!("Export time: {:?}", result.export_time);
println!("Cache hit rate: {:.2}%", result.performance_metrics.type_inference_cache_hit_rate * 100.0);
```

## Performance Comparison

The optimized export implementation provides significant performance improvements:

1. **Reduced Export Time**: By processing data in parallel and using specialized data structures
2. **Lower Memory Usage**: Through batch processing and streaming serialization
3. **Improved Scalability**: Performance scales better with larger datasets
4. **Better User Experience**: Faster export times and more detailed performance metrics

## Implementation Notes

1. The old slow `generate_comprehensive_export` method has been deprecated and marked with a warning
2. The HTML export has been updated to use the optimized implementation
3. All code paths now use the optimized implementation, ensuring consistent performance

## Future Improvements

1. **Compression Support**: Add optional GZIP compression for exported files
2. **Incremental Export**: Support for incremental updates to existing export files
3. **Custom Export Configuration**: Allow users to configure which files to generate
4. **Further Parallelization**: Explore additional opportunities for parallel processing