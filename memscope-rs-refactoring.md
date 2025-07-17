# memscope-rs Project Refactoring Suggestions

## Project Simplification and Consolidation Suggestions

### 1. Core Module Consolidation

1. **Consolidate Tracker Related Files**:
   - Merge `src/tracker.rs` and `src/scope_tracker.rs` into a single `src/tracking.rs` file
   - These two files are functionally related, both concerning memory allocation and scope tracking. Consolidating them can reduce cross-file references

2. **Consolidate Visualization Related Files**:
   - Merge `src/visualization.rs`, `src/advanced_charts.rs`, and `src/html_export.rs` into a single `src/visualization.rs` file
   - Also merge `src/optimized_html_export.rs` into the above file, as they are all related to data visualization

3. **Consolidate Export Functionality**:
   - Merge `src/export_enhanced.rs` and `src/report_generator.rs` into a single `src/export.rs` file
   - These files are all related to data export and report generation, with overlapping functionality

4. **Consolidate Analysis Functionality**:
   - Merge `src/advanced_analysis.rs` and `src/unsafe_ffi_visualization.rs` into a single `src/analysis.rs` file
   - These are all related to data analysis functionality

### 2. Simplify Type Definitions

1. **Streamline Type Definitions**:
   - The `src/types.rs` file is too large (1326 lines) and contains too many type definitions
   - It's recommended to split the types into smaller files by functionality:
     - `src/types/core.rs`: Core types and error handling
     - `src/types/allocation.rs`: Allocation-related types
     - `src/types/visualization.rs`: Visualization-related types
     - `src/types/analysis.rs`: Analysis-related types

2. **Reduce Duplicate Data Structures**:
   - There are currently multiple similar report structures, such as `ComprehensiveReport`, `MemoryOverview`, etc.
   - These structures can be consolidated to reduce redundancy

### 3. Consolidate Example Files

1. **Integrate Example Files**:
   - There are multiple similar example files in the `examples` directory, such as `test_html_dashboard.rs`, `test_fast_dashboard.rs`, etc.
   - It's recommended to merge them into fewer, more representative examples, with each example demonstrating different features

2. **Create Categorized Examples**:
   - Basic usage examples: Demonstrate core functionality
   - Advanced usage examples: Show advanced analysis features
   - Visualization examples: Display different visualization options

### 4. Simplify Module Structure

1. **Reduce Module Hierarchy**:
   - Currently, `lib.rs` exports too many modules, making the API overly complex
   - It's recommended to group related functionality into fewer public modules and hide implementation details

2. **Reorganize Public API**:
   - Focus core functionality in a few main modules:
     - `tracking`: Memory tracking functionality
     - `analysis`: Analysis functionality
     - `visualization`: Visualization functionality
     - `export`: Export functionality

### 5. Resolve Duplicate Export Issues

1. **Fix Duplicate Exports**:
   - There are duplicate export issues in `lib.rs`, such as `MemoryTracker` being defined multiple times
   - The exports need to be organized to ensure each type is only exported once

### 6. Specific File Consolidation Suggestions

```text
src/
├── tracking.rs (merge tracker.rs and scope_tracker.rs)
├── analysis.rs (merge advanced_analysis.rs and unsafe_ffi_tracker.rs)
├── visualization.rs (merge visualization.rs, advanced_charts.rs, html_export.rs and optimized_html_export.rs)
├── export.rs (merge export_enhanced.rs and report_generator.rs)
├── allocator.rs (keep as is)
├── types/
│   ├── core.rs
│   ├── allocation.rs
│   ├── visualization.rs
│   └── analysis.rs
└── utils.rs (keep as is)
```

### 7. Streamline Examples Directory

```text
examples/
├── basic_usage.rs (basic usage examples)
├── advanced_analysis.rs (consolidate complex analysis examples)
├── visualization_showcase.rs (consolidate visualization examples)
└── unsafe_ffi_demo.rs (keep as is, specific functionality)
```

## Refactored Module Structure

### 1. Core Tracking Module (`src/tracking.rs`)

This module will contain all memory tracking related functionality:

- Memory allocation tracking
- Scope tracking
- Variable association
- Statistics collection

### 2. Analysis Module (`src/analysis.rs`)

This module will contain all data analysis related functionality:

- Memory usage analysis
- Type analysis
- Unsafe code analysis
- FFI call analysis
- Performance analysis

### 3. Visualization Module (`src/visualization.rs`)

This module will contain all data visualization related functionality:

- SVG chart generation
- HTML dashboard
- Timeline visualization
- Interactive reports

### 4. Export Module (`src/export.rs`)

This module will contain all data export related functionality:

- JSON export
- HTML report generation
- SVG export
- Enhanced JSON export

### 5. Types Module (`src/types/`)

Split large type definition files into smaller, focused files:

- `core.rs`: Error types, result types, basic interfaces
- `allocation.rs`: Allocation information, memory statistics
- `visualization.rs`: Visualization-related types
- `analysis.rs`: Analysis-related types

## Refactored Public API

```rust
// lib.rs
pub mod tracking;
pub mod analysis;
pub mod visualization;
pub mod export;
pub mod types;
pub mod allocator;
pub mod utils;

// Re-export core functionality for backward compatibility
pub use allocator::TrackingAllocator;
pub use tracking::{get_global_tracker, MemoryTracker, track_var, init};
pub use types::core::{TrackingError, TrackingResult};
pub use visualization::{export_memory_analysis, export_lifecycle_timeline};
pub use export::generate_interactive_html_report;

// Set up global allocator
#[cfg(feature = "tracking-allocator")]
#[global_allocator]
pub static GLOBAL: TrackingAllocator = TrackingAllocator::new();
```

## Benefits of Refactoring

1. **Reduced File Count**: From 20+ files to about 10 core files
2. **Improved Code Organization**: Related functionality grouped together
3. **Simplified API**: Clearer module structure and exports
4. **Reduced Duplication**: Consolidated similar functionality
5. **Better Maintainability**: Easier to understand and modify the code
6. **Improved Documentation**: More focused functionality is easier to document

## Implementation Steps

1. Create the new file structure
2. Merge related functionality into the new files
3. Update internal reference paths
4. Update public API exports
5. Update documentation and examples
6. Run tests to ensure functionality remains intact

This refactoring maintains all existing functionality while significantly simplifying the code structure, making the project more maintainable and easier to understand.
