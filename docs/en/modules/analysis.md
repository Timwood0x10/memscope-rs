# Analysis Module

## Overview

The analysis module provides advanced memory analysis capabilities including leak detection, safety analysis, lifecycle analysis, and pattern detection. It implements various detectors and analyzers to identify memory issues and provide insights.

## Components

### 1. Detectors

**Files**: `src/analysis/detectors/`

**Purpose**: Specialized detectors for common memory issues.

#### LeakDetector

Detects potential memory leaks:

```rust
pub struct LeakDetector {
    config: LeakDetectorConfig,
}

impl Detector for LeakDetector {
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        // Detect allocations without corresponding deallocations
        let leaked: Vec<_> = allocations.iter()
            .filter(|a| a.timestamp_dealloc.is_none())
            .collect();

        DetectionResult {
            detector_name: "LeakDetector".to_string(),
            issues: leaked.len(),
            details: leaked.iter().map(|a| format!("0x{:x}: {} bytes", a.ptr, a.size)).collect(),
        }
    }
}
```

#### UafDetector

Detects use-after-free issues:

```rust
pub struct UafDetector {
    config: UafDetectorConfig,
}

impl Detector for UafDetector {
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        // Detect accesses after deallocation
        // ... implementation
    }
}
```

#### OverflowDetector

Detects buffer overflow issues:

```rust
pub struct OverflowDetector {
    config: OverflowDetectorConfig,
}

impl Detector for OverflowDetector {
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        // Detect buffer overflows
        // ... implementation
    }
}
```

#### SafetyDetector

Detects safety violations:

```rust
pub struct SafetyDetector {
    config: SafetyDetectorConfig,
}

impl Detector for SafetyDetector {
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        // Detect safety violations
        // ... implementation
    }
}
```

#### LifecycleDetector

Detects lifecycle issues:

```rust
pub struct LifecycleDetector {
    config: LifecycleDetectorConfig,
}

impl Detector for LifecycleDetector {
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        // Detect lifecycle issues
        // ... implementation
    }
}
```

### 2. Specialized Analyzers

#### MemoryPassportTracker

Tracks memory lifecycle with "passports":

```rust
pub struct MemoryPassport {
    pub ptr: usize,
    pub size: usize,
    pub created_at: u64,
    pub status: PassportStatus,
    pub events: Vec<PassportEvent>,
}
```

#### CircularReferenceDetector

Detects circular references in smart pointers:

```rust
pub fn detect_circular_references(allocations: &[AllocationInfo]) -> CircularReferenceAnalysis {
    // Detect cycles in reference graphs
    // ... implementation
}
```

#### VariableRelationshipAnalyzer

Analyzes variable relationships:

```rust
pub fn build_variable_relationship_graph(allocations: &[AllocationInfo]) -> VariableRelationshipGraph {
    // Build relationship graph
    // ... implementation
}
```

### 3. Pattern Analyzers

#### AsyncAnalyzer

Analyzes async patterns:

```rust
pub struct AsyncAnalyzer {
    // Analyzes Future state machines
}
```

#### BorrowAnalyzer

Analyzes borrow patterns:

```rust
pub struct BorrowAnalyzer {
    // Analyzes borrow checker patterns
}
```

#### GenericAnalyzer

Analyzes generic type usage:

```rust
pub struct GenericAnalyzer {
    // Analyzes generic instantiations
}
```

#### ClosureAnalyzer

Analyzes closure captures:

```rust
pub struct ClosureAnalyzer {
    // Analyzes closure lifetime patterns
}
```

### 4. AnalysisManager

**File**: `src/analysis/mod.rs`

**Purpose**: Consolidates all analysis functionality.

**Core Implementation**:

```rust
pub struct AnalysisManager {
    // Consolidates all analysis functionality
}

impl AnalysisManager {
    /// Analyze memory fragmentation
    pub fn analyze_fragmentation(&self, allocations: &[AllocationInfo]) -> FragmentationAnalysis {
        // ... implementation
    }

    /// Analyze system library usage
    pub fn analyze_system_libraries(&self, allocations: &[AllocationInfo]) -> SystemLibraryStats {
        // ... implementation
    }

    /// Analyze concurrency safety
    pub fn analyze_concurrency_safety(&self, allocations: &[AllocationInfo]) -> ConcurrencyAnalysis {
        // ... implementation
    }

    /// Perform comprehensive analysis
    pub fn perform_comprehensive_analysis(
        &self,
        allocations: &[AllocationInfo],
        stats: &MemoryStats,
    ) -> ComprehensiveAnalysisReport {
        // ... implementation
    }
}
```

## Usage Examples

### Using Detectors

```rust
use memscope::analysis::detectors::{LeakDetector, LeakDetectorConfig};

// Create leak detector
let detector = LeakDetector::new(LeakDetectorConfig::default());

// Run detection
let result = detector.detect(&allocations);

println!("Detected {} potential leaks", result.issues);
for detail in &result.details {
    println!("  {}", detail);
}
```

### Comprehensive Analysis

```rust
use memscope::analysis::AnalysisManager;

let manager = AnalysisManager::new();

// Perform comprehensive analysis
let report = manager.perform_comprehensive_analysis(&allocations, &stats);

println!("Fragmentation ratio: {}", report.fragmentation_analysis.fragmentation_ratio);
println!("Unsafe FFI operations: {}", report.unsafe_ffi_stats.total_operations);
```

### Pattern Analysis

```rust
// Analyze async patterns
let async_analyzer = get_global_async_analyzer();
let async_analysis = async_analyzer.analyze_async_patterns();

// Analyze borrow patterns
let borrow_analyzer = get_global_borrow_analyzer();
let borrow_analysis = borrow_analyzer.analyze_borrow_patterns();

// Analyze closure patterns
let closure_analyzer = get_global_closure_analyzer();
let closure_analysis = closure_analyzer.analyze_closure_patterns(&allocations);
```

## Design Principles

### 1. Modularity
Detectors and analyzers are independent:
- **Benefits**: Easy to add new analyses
- **Trade-off**: More complex to coordinate

### 2. Extensibility
Easy to add custom detectors:
- **Benefits**: Flexible for specific use cases
- **Trade-off**: API complexity

### 3. Performance
Optimized for efficient analysis:
- **Benefits**: Fast analysis
- **Trade-off**: May use more memory

## Best Practices

1. **Detector Selection**: Use appropriate detectors for your use case
2. **Configuration**: Customize detector configurations for better results
3. **Error Handling**: Always handle analysis errors
4. **Performance**: Cache analysis results when possible

## Limitations

1. **False Positives**: Detectors may report false positives
2. **Performance**: Complex analyses may be slow
3. **Memory Usage**: Analysis may consume significant memory
4. **Context**: Some analyses require additional context

## Future Improvements

1. **Better Accuracy**: Reduce false positives
2. **More Detectors**: Add more specialized detectors
3. **Performance**: Improve analysis performance
4. **Machine Learning**: Use ML for pattern detection
5. **Real-time Analysis**: Support real-time analysis

## Relation Inference Engine

### Overview

The relation inference engine automatically detects semantic relationships between memory allocations, helping to understand program memory structure and ownership models.

### Supported Relationship Types

| Relationship | Description | Detection Method |
|--------------|-------------|------------------|
| **Owner** | A owns or points to B | Pointer scanning - find pointer to B in A's memory |
| **Slice** | A is a sub-view of B | Address range detection - A's pointer is inside B |
| **Clone** | A is a clone of B | Content similarity + time window + call stack matching |
| **Shared** | A and B share ownership | Arc/Rc control block pattern recognition |

### Usage

```rust
use memscope_rs::analysis::relation_inference::{RelationGraphBuilder, Relation};

// Build relation graph from active allocations
let graph = RelationGraphBuilder::build(&allocations, None);

// Query relationships
for edge in &graph.edges {
    println!("{:?}: {} -> {}", edge.relation, edge.from, edge.to);
}

// Detect circular references
let cycles = graph.detect_cycles();
if !cycles.is_empty() {
    println!("Detected {} circular references", cycles.len());
}

// Get all nodes
let nodes = graph.all_nodes();
```

### Accuracy Metrics

Based on real test data:

```
=== Clone Detection Accuracy ===
Precision: 100.00%
Recall: 100.00%
F1 Score: 100.00%

=== Owner Detection Accuracy ===
✅ Box<Vec> relationship correctly detected
✅ Independent Vec no false positives

=== Performance ===
1000 allocations build time: ~230ms
```

### Configuration Options

```rust
use memscope_rs::analysis::relation_inference::{GraphBuilderConfig, CloneConfig};

let config = GraphBuilderConfig {
    clone_config: CloneConfig {
        min_similarity: 0.8,              // Minimum similarity threshold
        min_similarity_no_stack_hash: 0.95, // Stricter threshold without call stack
        max_time_diff_ns: 10_000_000,     // 10ms time window
        max_clone_edges_per_node: 10,     // Max clone edges per node
        ..Default::default()
    },
};

let graph = RelationGraphBuilder::build(&allocations, Some(config));
```

### Detection Algorithm Details

#### Owner Detection

Scans pointer values in allocation memory. If a pointer to another allocation is found, an Owner relationship is established.

```rust
// Detection flow
1. Read allocation memory content
2. Scan each 8-byte aligned position
3. Parse as pointer value
4. Look up target allocation in RangeMap
5. Create Owner edge
```

#### Slice Detection

Detects if one allocation's pointer falls inside another allocation (not at the start).

```rust
// Detection conditions
1. A.ptr is within B's address range (B.ptr < A.ptr < B.ptr + B.size)
2. A.size <= 256 (Slice metadata is usually small)
3. A's full range is within B
```

#### Clone Detection

Detects clone relationships based on content similarity and time window.

```rust
// Detection conditions
1. Same TypeKind and size
2. Same call_stack_hash (or both None)
3. Allocation time difference within window
4. Content similarity >= threshold
```

#### Shared Detection

Detects Arc/Rc shared ownership.

```rust
// Detection conditions
1. Multiple Owner edges point to the same target
2. Target memory layout matches ArcInner structure
3. strong_count and weak_count are within reasonable range
```

### Notes

1. **Owner Detection**: Requires metadata on heap (e.g., `Box<Vec>`), stack metadata cannot be scanned
2. **Clone Detection**: Relies on call stack hash, allocations with same call stack are grouped for comparison
3. **Shared Detection**: Depends on Owner relationships, requires Owner detection first
4. **Performance**: Uses sliding time window to avoid O(n²) complexity