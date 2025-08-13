# Analysis API Reference

Memory analysis functions and analyzers in memscope-rs.

## Overview

This document describes the analysis API that provides advanced memory analysis capabilities including fragmentation analysis, circular reference detection, unsafe/FFI tracking, and comprehensive memory pattern analysis.

## Analysis Manager

### AnalysisManager

Main analysis interface that consolidates all analysis functionality.

**Module:** `memscope_rs::analysis`

**Source:** `src/analysis/mod.rs`

```rust
use memscope_rs::analysis::AnalysisManager;

let manager = AnalysisManager::new();
let allocations = tracker.get_active_allocations()?;
let stats = tracker.get_stats()?;

let report = manager.perform_comprehensive_analysis(&allocations, &stats);
```

#### Methods

##### perform_comprehensive_analysis

```rust
pub fn perform_comprehensive_analysis(
    &self,
    allocations: &[AllocationInfo],
    stats: &MemoryStats,
) -> ComprehensiveAnalysisReport
```

Perform comprehensive analysis combining all analysis types.

**Parameters:**
- `allocations`: `&[AllocationInfo]` - Memory allocations to analyze
- `stats`: `&MemoryStats` - Memory statistics

**Returns:** `ComprehensiveAnalysisReport` - Complete analysis results

## Memory Analysis

### EnhancedMemoryAnalyzer

Advanced memory analysis with pattern recognition.

**Module:** `memscope_rs::analysis::enhanced_memory_analysis`

**Source:** `src/analysis/enhanced_memory_analysis.rs`

```rust
use memscope_rs::EnhancedMemoryAnalyzer;

let analyzer = EnhancedMemoryAnalyzer::new();
let allocations = tracker.get_active_allocations()?;
let analysis = analyzer.analyze_memory_patterns(&allocations);
```

#### Methods

##### analyze_memory_patterns

```rust
pub fn analyze_memory_patterns(&self, allocations: &[AllocationInfo]) -> MemoryPatternAnalysis
```

Analyze memory usage patterns and identify anomalies.

##### detect_fragmentation

```rust
pub fn detect_fragmentation(&self, allocations: &[AllocationInfo]) -> FragmentationAnalysis
```

Analyze memory fragmentation patterns.

##### identify_leaks

```rust
pub fn identify_leaks(&self, allocations: &[AllocationInfo]) -> Vec<PotentialLeak>
```

Identify potential memory leaks.

## Unsafe/FFI Analysis

### UnsafeFFITracker

Tracks unsafe operations and FFI boundary crossings.

**Module:** `memscope_rs::analysis::unsafe_ffi_tracker`

**Source:** `src/analysis/unsafe_ffi_tracker.rs`

```rust
use memscope_rs::{get_global_unsafe_ffi_tracker, UnsafeFFITracker};

let tracker = get_global_unsafe_ffi_tracker();
let stats = tracker.get_stats();
```

#### Methods

##### track_unsafe_allocation

```rust
pub fn track_unsafe_allocation(&self, ptr: usize, size: usize, context: &str)
```

Track an unsafe memory allocation.

##### track_ffi_call

```rust
pub fn track_ffi_call(&self, function_name: &str, ptr: usize)
```

Track an FFI function call that involves memory.

##### get_stats

```rust
pub fn get_stats(&self) -> UnsafeFFIStats
```

Get unsafe/FFI operation statistics.

### Global Functions

```rust
// Get global unsafe/FFI tracker
let tracker = get_global_unsafe_ffi_tracker();

// Track unsafe allocation
track_unsafe_alloc!(ptr, size, "manual allocation");

// Track FFI allocation  
track_ffi_alloc!(ptr, "malloc");
```

## Circular Reference Analysis

### CircularReferenceAnalysis

Detects circular references in smart pointers.

**Module:** `memscope_rs::analysis::circular_reference`

**Source:** `src/analysis/circular_reference.rs`

```rust
use memscope_rs::analysis::detect_circular_references;

let allocations = tracker.get_active_allocations()?;
let analysis = detect_circular_references(&allocations);

for circular_ref in analysis.circular_references {
    println!("Circular reference detected: {:?}", circular_ref);
}
```

#### Functions

##### detect_circular_references

```rust
pub fn detect_circular_references(allocations: &[AllocationInfo]) -> CircularReferenceAnalysis
```

Detect circular references in smart pointer allocations.

**Parameters:**
- `allocations`: `&[AllocationInfo]` - Allocations to analyze

**Returns:** `CircularReferenceAnalysis` - Analysis results

## Advanced Type Analysis

### AsyncAnalyzer

Analyzes async/await memory patterns.

**Module:** `memscope_rs::analysis::async_analysis`

**Source:** `src/analysis/async_analysis.rs`

```rust
use memscope_rs::analysis::{get_global_async_analyzer, AsyncAnalyzer};

let analyzer = get_global_async_analyzer();
let analysis = analyzer.analyze_async_patterns();
```

#### Methods

##### analyze_async_patterns

```rust
pub fn analyze_async_patterns(&self) -> AsyncPatternAnalysis
```

Analyze async/await memory usage patterns.

##### get_async_statistics

```rust
pub fn get_async_statistics(&self) -> AsyncStatistics
```

Get async-related memory statistics.

### BorrowAnalyzer

Analyzes borrow checker integration and lifetime tracking.

**Module:** `memscope_rs::analysis::borrow_analysis`

**Source:** `src/analysis/borrow_analysis.rs`

```rust
use memscope_rs::analysis::{get_global_borrow_analyzer, BorrowAnalyzer};

let analyzer = get_global_borrow_analyzer();
let analysis = analyzer.analyze_borrow_patterns();
```

### GenericAnalyzer

Analyzes generic type usage and constraints.

**Module:** `memscope_rs::analysis::generic_analysis`

**Source:** `src/analysis/generic_analysis.rs`

```rust
use memscope_rs::analysis::{get_global_generic_analyzer, GenericAnalyzer};

let analyzer = get_global_generic_analyzer();
let stats = analyzer.get_generic_statistics();
```

### ClosureAnalyzer

Analyzes closure captures and lifetime relationships.

**Module:** `memscope_rs::analysis::closure_analysis`

**Source:** `src/analysis/closure_analysis.rs`

```rust
use memscope_rs::analysis::{get_global_closure_analyzer, ClosureAnalyzer};

let analyzer = get_global_closure_analyzer();
let allocations = tracker.get_active_allocations()?;
let report = analyzer.analyze_closure_patterns(&allocations);
```

### LifecycleAnalyzer

Analyzes lifecycle patterns including Drop trait and RAII.

**Module:** `memscope_rs::analysis::lifecycle_analysis`

**Source:** `src/analysis/lifecycle_analysis.rs`

```rust
use memscope_rs::analysis::{get_global_lifecycle_analyzer, LifecycleAnalyzer};

let analyzer = get_global_lifecycle_analyzer();
let report = analyzer.get_lifecycle_report();
```

## Analysis Results Types

### ComprehensiveAnalysisReport

Complete analysis report combining all analysis types.

```rust
pub struct ComprehensiveAnalysisReport {
    pub fragmentation_analysis: FragmentationAnalysis,
    pub system_library_stats: SystemLibraryStats,
    pub concurrency_analysis: ConcurrencyAnalysis,
    pub unsafe_ffi_stats: UnsafeFFIStats,
    pub circular_reference_analysis: CircularReferenceAnalysis,
    pub advanced_type_analysis: AdvancedTypeAnalysisReport,
    pub borrow_analysis: BorrowPatternAnalysis,
    pub generic_analysis: GenericStatistics,
    pub async_analysis: AsyncPatternAnalysis,
    pub closure_analysis: ClosureAnalysisReport,
    pub lifecycle_analysis: LifecycleAnalysisReport,
    pub memory_stats: MemoryStats,
    pub analysis_timestamp: u64,
}
```

### UnsafeFFIStats

Statistics for unsafe operations and FFI calls.

```rust
pub struct UnsafeFFIStats {
    pub unsafe_allocations: usize,
    pub ffi_calls: usize,
    pub potential_violations: Vec<SafetyViolation>,
    pub boundary_crossings: usize,
}
```

### CircularReferenceAnalysis

Results of circular reference detection.

```rust
pub struct CircularReferenceAnalysis {
    pub circular_references: Vec<CircularReference>,
    pub potential_cycles: Vec<PotentialCycle>,
    pub reference_graph: ReferenceGraph,
}
```

## Convenience Functions

### Global Analysis Functions

```rust
use memscope_rs::analysis::*;

// Analyze fragmentation
let fragmentation = analyze_fragmentation(&allocations);

// Analyze system libraries
let system_stats = analyze_system_libraries(&allocations);

// Analyze concurrency safety
let concurrency = analyze_concurrency_safety(&allocations);

// Get unsafe/FFI stats
let unsafe_stats = get_unsafe_ffi_stats();

// Perform comprehensive analysis
let report = perform_comprehensive_analysis(&allocations, &stats);
```

## Complete Example

```rust
use memscope_rs::{
    init, track_var, get_global_tracker,
    analysis::{AnalysisManager, get_global_unsafe_ffi_tracker}
};
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    // Create and track various data types
    let vec_data = vec![1, 2, 3, 4, 5];
    track_var!(vec_data);
    
    let rc_data = Rc::new(String::from("shared data"));
    track_var!(rc_data);
    
    let rc_clone = Rc::clone(&rc_data);
    track_var!(rc_clone);
    
    // Get allocations and stats
    let tracker = get_global_tracker();
    let allocations = tracker.get_active_allocations()?;
    let stats = tracker.get_stats()?;
    
    // Perform comprehensive analysis
    let manager = AnalysisManager::new();
    let report = manager.perform_comprehensive_analysis(&allocations, &stats);
    
    // Print analysis results
    println!("📊 Analysis Results:");
    println!("  Fragmentation ratio: {:.2}%", report.fragmentation_analysis.fragmentation_ratio * 100.0);
    println!("  Circular references: {}", report.circular_reference_analysis.circular_references.len());
    println!("  Unsafe operations: {}", report.unsafe_ffi_stats.unsafe_allocations);
    
    // Get specific analyzer results
    let unsafe_tracker = get_global_unsafe_ffi_tracker();
    let unsafe_stats = unsafe_tracker.get_stats();
    println!("  FFI calls: {}", unsafe_stats.ffi_calls);
    
    // Export comprehensive analysis
    tracker.export_to_json("comprehensive_analysis")?;
    
    Ok(())
}
```

## Advanced Usage

### Custom Analysis Pipeline

```rust
use memscope_rs::analysis::*;

fn custom_analysis_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = get_global_tracker();
    let allocations = tracker.get_active_allocations()?;
    
    // Step 1: Basic fragmentation analysis
    let fragmentation = analyze_fragmentation(&allocations);
    if fragmentation.fragmentation_ratio > 0.3 {
        println!("⚠️ High fragmentation detected: {:.1}%", fragmentation.fragmentation_ratio * 100.0);
    }
    
    // Step 2: Circular reference detection
    let circular_refs = detect_circular_references(&allocations);
    if !circular_refs.circular_references.is_empty() {
        println!("🔄 Circular references found: {}", circular_refs.circular_references.len());
    }
    
    // Step 3: Async pattern analysis
    let async_analyzer = get_global_async_analyzer();
    let async_analysis = async_analyzer.analyze_async_patterns();
    println!("🔄 Async allocations: {}", async_analysis.future_allocations);
    
    // Step 4: Generic type analysis
    let generic_analyzer = get_global_generic_analyzer();
    let generic_stats = generic_analyzer.get_generic_statistics();
    println!("📋 Generic instantiations: {}", generic_stats.total_instantiations);
    
    Ok(())
}
```

### Filtering and Targeted Analysis

```rust
use memscope_rs::analysis::*;

fn targeted_analysis() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = get_global_tracker();
    let all_allocations = tracker.get_active_allocations()?;
    
    // Filter large allocations
    let large_allocations: Vec<_> = all_allocations
        .into_iter()
        .filter(|alloc| alloc.size > 1024)
        .collect();
    
    // Analyze only large allocations
    let fragmentation = analyze_fragmentation(&large_allocations);
    println!("Large allocation fragmentation: {:.2}%", fragmentation.fragmentation_ratio * 100.0);
    
    // Filter smart pointer allocations
    let smart_ptr_allocations: Vec<_> = all_allocations
        .into_iter()
        .filter(|alloc| {
            alloc.type_name.as_ref()
                .map(|name| name.contains("Rc<") || name.contains("Arc<"))
                .unwrap_or(false)
        })
        .collect();
    
    // Analyze circular references in smart pointers only
    let circular_analysis = detect_circular_references(&smart_ptr_allocations);
    println!("Smart pointer circular references: {}", circular_analysis.circular_references.len());
    
    Ok(())
}
```

## See Also

- [Core Types Reference](core-types.md) - Data structures and types
- [Tracking API Reference](tracking-api.md) - Memory tracking functions
- [Export API Reference](export-api.md) - Data export functionality