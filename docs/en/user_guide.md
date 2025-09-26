# MemScope User Guide

## Overview

MemScope is a high-performance Rust memory tracking and analysis tool that provides three complementary tracking modes and interactive HTML dashboard generation for comprehensive memory analysis.

## Three Tracking Modes

### 1. Real-time Memory Tracking (Live Mode)
**Purpose**: Real-time memory allocation tracking during program execution  
**Best for**: Development, debugging, immediate feedback

```rust
use memscope::core::tracker::MemoryTracker;
use std::sync::Arc;

// Initialize tracker
let tracker = Arc::new(MemoryTracker::new());

// Track memory allocations in real-time
tracker.track_allocation(ptr, size, Some("my_variable".to_string()))?;
```

### 2. Sampling-based Tracking (Sampling Mode)
**Purpose**: High-performance sampling with minimal runtime overhead  
**Best for**: Production environments, performance-critical applications

```rust
use memscope::core::sampling_tracker::SamplingTracker;

// Initialize sampling tracker
let tracker = SamplingTracker::new("./output");

// Track variable with reduced overhead
tracker.track_variable(ptr, size, "var_name".to_string(), "String".to_string())?;
```

### 3. Binary Analysis (Post-mortem Mode)
**Purpose**: Analyze pre-generated binary files from previous tracking sessions  
**Best for**: Offline analysis, CI/CD pipelines, detailed investigation

```rust
use memscope::export::api::Exporter;

// Convert binary to HTML dashboard
Exporter::binary_to_html("memory_data.memscope", "analysis.html")?;
```

## HTML Dashboard Generation

### Main Export APIs

#### 1. Direct HTML Export (Live Data)
```rust
use memscope::export::clean_unified_api::export_html;

// Export current tracker state to HTML
export_html(tracker, "dashboard.html")?;
```

#### 2. Binary to HTML Conversion
```rust
use memscope::core::tracker::memory_tracker::MemoryTracker;

// Convert existing binary file to interactive HTML
MemoryTracker::export_binary_to_html("data.memscope", "dashboard.html")?;
```

#### 3. Unified Export API
```rust
use memscope::export::api::Exporter;

// Create exporter with custom configuration
let exporter = Exporter::new(allocations, stats, config);
let stats = exporter.export_html("output.html")?;
```

## Dashboard Features

The generated HTML dashboard includes:

### 1. Overview Panel
- Total memory usage statistics
- Active/deallocated allocation counts
- Performance metrics and trends
- Memory efficiency indicators

### 2. Thread Variables (50/50) Cards
Interactive cards showing:
- Variable name and lifecycle stage (🟢 Active, 🟡 Allocated, 🔄 Shared, ⚫ Deallocated)
- Memory size and allocation count
- Thread information
- Performance category (CPU/IO/Memory/Async intensive)

### 3. Detailed Variable Inspector
Click any variable card to access:
- **Overview Tab**: Basic information and lifecycle
- **Lifecycle Tab**: Detailed allocation timeline
- **FFI Passport Tab**: Foreign Function Interface boundary tracking
- **Optimization Tab**: Performance recommendations

### 4. Memory Map Visualization
- Thread-based memory layout
- Visual representation of variable sizes
- Memory hotspot identification

### 5. Enhanced Diagnostics
- Real-time problem detection
- Memory leak pattern recognition
- Performance bottleneck identification
- Root cause analysis

## Trackable Data Types

### Core Allocation Information
```rust
pub struct AllocationInfo {
    pub ptr: usize,                    // Memory address
    pub size: usize,                   // Allocation size in bytes
    pub var_name: Option<String>,      // Variable name
    pub type_name: Option<String>,     // Rust type name
    pub thread_id: String,             // Thread identifier
    pub timestamp_alloc: u64,          // Allocation timestamp
    pub timestamp_dealloc: Option<u64>, // Deallocation timestamp
    pub borrow_count: usize,           // Active borrow count
    pub stack_trace: Option<Vec<String>>, // Call stack
    pub is_leaked: bool,               // Leak detection flag
    pub lifetime_ms: Option<u64>,      // Variable lifetime
    // ... extensive metadata fields
}
```

### Memory Statistics
```rust
pub struct MemoryStats {
    pub total_allocations: usize,      // Total allocation count
    pub active_allocations: usize,     // Currently active
    pub peak_memory: usize,            // Peak memory usage
    pub leaked_allocations: usize,     // Detected leaks
    pub fragmentation_analysis: FragmentationAnalysis,
    pub lifecycle_stats: ScopeLifecycleMetrics,
    // ... comprehensive statistics
}
```

## Mode Coordination Workflow

### Development Phase
1. **Live Mode**: Use real-time tracking for immediate feedback
2. **Export to Binary**: Save session data for later analysis
3. **Generate HTML**: Create interactive dashboards for detailed investigation

### Production Phase
1. **Sampling Mode**: Deploy with minimal overhead
2. **Collect Binary Data**: Gather performance data over time
3. **Offline Analysis**: Convert to HTML for post-mortem analysis

### Example Complete Workflow
```rust
// 1. Live tracking during development
let tracker = Arc::new(MemoryTracker::new());
// ... track allocations ...

// 2. Export to binary for archival
tracker.export_user_binary("session.memscope")?;

// 3. Generate interactive HTML dashboard
MemoryTracker::export_binary_to_html("session.memscope", "analysis.html")?;

// 4. Later: sampling mode for production
let sampler = SamplingTracker::new("./prod_data");
// ... production tracking ...

// 5. Analyze production data
Exporter::binary_to_html("prod_data.memscope", "production_analysis.html")?;
```

## Usage Recommendations

### Performance Considerations
- **Development Environment**: Use live mode for maximum visibility
- **Testing Environment**: Use sampling mode to balance performance and data quality
- **Production Environment**: Use sampling mode to ensure minimal performance impact

### Data Management
- Regularly export binary files to prevent data loss
- Use version control to track memory analysis reports
- Establish memory performance baselines and alert thresholds

### Troubleshooting
- Use detailed variable inspector for deep analysis of specific memory issues
- Leverage enhanced diagnostics for automatic problem detection
- Combine multiple tracking sessions for trend analysis

## Quick Start Example

```rust
use memscope::core::tracker::MemoryTracker;
use memscope::export::clean_unified_api::export_html;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize tracker
    let tracker = Arc::new(MemoryTracker::new());
    
    // 2. Your application code with memory tracking
    let data = vec![1, 2, 3, 4, 5];
    tracker.track_allocation(
        data.as_ptr() as usize,
        data.len() * std::mem::size_of::<i32>(),
        Some("my_vector".to_string())
    )?;
    
    // 3. Export to interactive HTML dashboard
    export_html(tracker, "memory_analysis.html")?;
    
    println!("Memory analysis saved to memory_analysis.html");
    Ok(())
}
```