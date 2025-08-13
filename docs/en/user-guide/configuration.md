# Configuration Options Guide

memscope-rs provides comprehensive configuration options to customize memory tracking behavior and performance according to your specific needs.

## 🎯 Configuration Overview

| Configuration Category | Purpose | Performance Impact |
|------------------------|---------|-------------------|
| **Tracking Config** | Control tracking behavior | Low to Medium |
| **Export Config** | Customize export formats | Medium |
| **Performance Config** | Optimize performance | Significant |
| **Analysis Config** | Adjust analysis depth | Medium to High |

## 🔧 Basic Configuration

### Initialization Configuration

```rust
use memscope_rs::{init_with_config, TrackingConfig};

fn main() {
    // Use default configuration
    memscope_rs::init();
    
    // Or use custom configuration
    let config = TrackingConfig {
        enable_stack_traces: true,
        max_tracked_allocations: 10000,
        enable_lifecycle_tracking: true,
        ..Default::default()
    };
    
    init_with_config(config);
}
```

### TrackingConfig Options

```rust
pub struct TrackingConfig {
    /// Enable stack traces (impacts performance)
    pub enable_stack_traces: bool,
    
    /// Maximum number of tracked allocations (0 = unlimited)
    pub max_tracked_allocations: usize,
    
    /// Enable lifecycle tracking
    pub enable_lifecycle_tracking: bool,
    
    /// Enable borrow checker analysis
    pub enable_borrow_analysis: bool,
    
    /// Enable circular reference detection
    pub enable_circular_reference_detection: bool,
    
    /// Enable async analysis
    pub enable_async_analysis: bool,
    
    /// Enable FFI tracking
    pub enable_ffi_tracking: bool,
    
    /// Memory threshold (bytes), allocations smaller than this may be ignored
    pub memory_threshold: usize,
    
    /// Sampling rate (0.0-1.0), 1.0 = track all allocations
    pub sampling_rate: f64,
    
    /// Enable real-time statistics
    pub enable_real_time_stats: bool,
}
```

## 📊 Export Configuration

### ExportOptions Configuration

```rust
use memscope_rs::{get_global_tracker, ExportOptions};

fn export_with_config() {
    let tracker = get_global_tracker();
    
    let options = ExportOptions {
        include_stack_traces: true,
        include_lifecycle_data: true,
        include_type_analysis: true,
        compress_output: true,
        max_entries: Some(5000),
        filter_small_allocations: true,
        min_allocation_size: 64,
        ..Default::default()
    };
    
    tracker.export_to_json_with_options("analysis", &options).unwrap();
}
```

### Export Options Details

```rust
pub struct ExportOptions {
    /// Include stack trace information
    pub include_stack_traces: bool,
    
    /// Include lifecycle data
    pub include_lifecycle_data: bool,
    
    /// Include type analysis information
    pub include_type_analysis: bool,
    
    /// Compress output files
    pub compress_output: bool,
    
    /// Maximum number of export entries
    pub max_entries: Option<usize>,
    
    /// Filter small allocations
    pub filter_small_allocations: bool,
    
    /// Minimum allocation size threshold
    pub min_allocation_size: usize,
    
    /// Include performance metrics
    pub include_performance_metrics: bool,
    
    /// Include memory layout information
    pub include_memory_layout: bool,
    
    /// Export format version
    pub format_version: String,
    
    /// Custom metadata
    pub custom_metadata: std::collections::HashMap<String, String>,
}
```

## ⚡ Performance Configuration

### High-Performance Configuration

```rust
use memscope_rs::TrackingConfig;

// Production configuration - minimal overhead
let production_config = TrackingConfig {
    enable_stack_traces: false,          // Disable stack traces
    max_tracked_allocations: 1000,       // Limit tracking count
    enable_lifecycle_tracking: false,    // Disable lifecycle tracking
    enable_borrow_analysis: false,       // Disable borrow analysis
    enable_circular_reference_detection: false,
    enable_async_analysis: false,
    enable_ffi_tracking: false,
    memory_threshold: 1024,              // Only track >1KB allocations
    sampling_rate: 0.1,                  // 10% sampling rate
    enable_real_time_stats: false,
};

// Development configuration - full features
let development_config = TrackingConfig {
    enable_stack_traces: true,
    max_tracked_allocations: 50000,
    enable_lifecycle_tracking: true,
    enable_borrow_analysis: true,
    enable_circular_reference_detection: true,
    enable_async_analysis: true,
    enable_ffi_tracking: true,
    memory_threshold: 0,                 // Track all allocations
    sampling_rate: 1.0,                  // 100% sampling rate
    enable_real_time_stats: true,
};

// Debug configuration - most detailed information
let debug_config = TrackingConfig {
    enable_stack_traces: true,
    max_tracked_allocations: 0,          // Unlimited
    enable_lifecycle_tracking: true,
    enable_borrow_analysis: true,
    enable_circular_reference_detection: true,
    enable_async_analysis: true,
    enable_ffi_tracking: true,
    memory_threshold: 0,
    sampling_rate: 1.0,
    enable_real_time_stats: true,
};
```

## 🌍 Environment Variable Configuration

Override configuration with environment variables:

```bash
# Basic configuration
export MEMSCOPE_ENABLE_STACK_TRACES=true
export MEMSCOPE_MAX_TRACKED_ALLOCATIONS=10000
export MEMSCOPE_MEMORY_THRESHOLD=1024
export MEMSCOPE_SAMPLING_RATE=0.5

# Feature toggles
export MEMSCOPE_ENABLE_LIFECYCLE_TRACKING=true
export MEMSCOPE_ENABLE_BORROW_ANALYSIS=false
export MEMSCOPE_ENABLE_ASYNC_ANALYSIS=true
export MEMSCOPE_ENABLE_FFI_TRACKING=false

# Export configuration
export MEMSCOPE_EXPORT_COMPRESS=true
export MEMSCOPE_EXPORT_MAX_ENTRIES=5000
export MEMSCOPE_EXPORT_MIN_SIZE=64

# Performance configuration
export MEMSCOPE_REAL_TIME_STATS=false
export MEMSCOPE_BACKGROUND_ANALYSIS=true
```

Reading environment variables in code:

```rust
use memscope_rs::TrackingConfig;

fn config_from_env() -> TrackingConfig {
    TrackingConfig {
        enable_stack_traces: std::env::var("MEMSCOPE_ENABLE_STACK_TRACES")
            .map(|v| v.parse().unwrap_or(false))
            .unwrap_or(false),
            
        max_tracked_allocations: std::env::var("MEMSCOPE_MAX_TRACKED_ALLOCATIONS")
            .map(|v| v.parse().unwrap_or(10000))
            .unwrap_or(10000),
            
        sampling_rate: std::env::var("MEMSCOPE_SAMPLING_RATE")
            .map(|v| v.parse().unwrap_or(1.0))
            .unwrap_or(1.0),
            
        ..Default::default()
    }
}
```

## 📁 Configuration Files

### TOML Configuration File

Create `memscope.toml`:

```toml
[tracking]
enable_stack_traces = true
max_tracked_allocations = 10000
enable_lifecycle_tracking = true
enable_borrow_analysis = true
enable_circular_reference_detection = true
enable_async_analysis = false
enable_ffi_tracking = false
memory_threshold = 512
sampling_rate = 1.0
enable_real_time_stats = true

[export]
include_stack_traces = true
include_lifecycle_data = true
include_type_analysis = true
compress_output = false
max_entries = 5000
filter_small_allocations = true
min_allocation_size = 64
include_performance_metrics = true

[analysis.circular_reference]
enabled = true
max_depth = 10
check_interval_ms = 1000

[analysis.lifecycle]
enabled = true
track_drop_order = true
analyze_scope_relationships = true

[analysis.async]
enabled = false
track_future_states = false
analyze_await_points = false

[analysis.ffi]
enabled = false
track_c_allocations = false
validate_pointer_safety = false
```

Loading configuration file:

```rust
use memscope_rs::config::load_config_from_file;

fn main() {
    let config = load_config_from_file("memscope.toml")
        .unwrap_or_else(|_| TrackingConfig::default());
    
    memscope_rs::init_with_config(config);
}
```

## 🎛️ Runtime Configuration

### Dynamic Configuration Adjustment

```rust
use memscope_rs::get_global_tracker;

fn adjust_runtime_config() {
    let tracker = get_global_tracker();
    
    // Dynamically adjust sampling rate
    tracker.set_sampling_rate(0.1);
    
    // Dynamically adjust memory threshold
    tracker.set_memory_threshold(2048);
    
    // Enable/disable specific features
    tracker.enable_stack_traces(false);
    tracker.enable_lifecycle_tracking(true);
    
    // Clean up old data
    tracker.cleanup_old_allocations(Duration::from_secs(300));
}
```

## 🔧 Best Practices

### 1. Layered Configuration Strategy

```rust
// Base configuration
let base_config = TrackingConfig::default();

// Environment-specific configuration
let env_config = match std::env::var("RUST_ENV").as_deref() {
    Ok("production") => production_overrides(),
    Ok("development") => development_overrides(),
    Ok("testing") => testing_overrides(),
    _ => TrackingConfig::default(),
};

// Merge configurations
let final_config = merge_configs(base_config, env_config);
```

### 2. Performance Monitoring Configuration

```rust
// Monitor configuration impact on performance
fn monitor_config_impact() {
    let start = std::time::Instant::now();
    
    // Perform some operations
    perform_operations();
    
    let duration = start.elapsed();
    println!("Operation took: {:?}", duration);
    
    // Adjust configuration based on performance
    if duration > Duration::from_millis(100) {
        // Reduce tracking precision to improve performance
        adjust_for_performance();
    }
}
```

## 📊 Configuration Presets

### Quick Configuration Presets

```rust
use memscope_rs::TrackingConfig;

impl TrackingConfig {
    /// Minimal configuration for production use
    pub fn minimal() -> Self {
        Self {
            enable_stack_traces: false,
            max_tracked_allocations: 1000,
            sampling_rate: 0.01,
            memory_threshold: 4096,
            ..Default::default()
        }
    }
    
    /// Balanced configuration for development
    pub fn balanced() -> Self {
        Self {
            enable_stack_traces: true,
            max_tracked_allocations: 10000,
            sampling_rate: 0.5,
            memory_threshold: 512,
            enable_lifecycle_tracking: true,
            ..Default::default()
        }
    }
    
    /// Maximum configuration for debugging
    pub fn maximum() -> Self {
        Self {
            enable_stack_traces: true,
            max_tracked_allocations: 0,
            sampling_rate: 1.0,
            memory_threshold: 0,
            enable_lifecycle_tracking: true,
            enable_borrow_analysis: true,
            enable_circular_reference_detection: true,
            enable_async_analysis: true,
            enable_ffi_tracking: true,
            enable_real_time_stats: true,
        }
    }
}

// Usage
fn main() {
    // Quick setup for different scenarios
    memscope_rs::init_with_config(TrackingConfig::balanced());
}
```

## 🎯 Configuration Examples by Use Case

### Web Server Configuration

```rust
let web_server_config = TrackingConfig {
    enable_stack_traces: false,        // Reduce overhead
    max_tracked_allocations: 5000,     // Limit memory usage
    sampling_rate: 0.05,               // 5% sampling
    memory_threshold: 4096,            // Only large allocations
    enable_lifecycle_tracking: false,  // Reduce complexity
    enable_real_time_stats: true,      // Monitor live performance
    ..Default::default()
};
```

### Desktop Application Configuration

```rust
let desktop_config = TrackingConfig {
    enable_stack_traces: true,         // Detailed debugging
    max_tracked_allocations: 20000,    // More tracking capacity
    sampling_rate: 0.8,                // High sampling rate
    memory_threshold: 256,             // Track smaller allocations
    enable_lifecycle_tracking: true,   // Full lifecycle analysis
    enable_borrow_analysis: true,      // Ownership analysis
    enable_real_time_stats: true,
    ..Default::default()
};
```

### Testing Configuration

```rust
let testing_config = TrackingConfig {
    enable_stack_traces: true,         // Full debugging info
    max_tracked_allocations: 0,        // Unlimited tracking
    sampling_rate: 1.0,                // Track everything
    memory_threshold: 0,               // Track all sizes
    enable_lifecycle_tracking: true,
    enable_borrow_analysis: true,
    enable_circular_reference_detection: true,
    enable_async_analysis: true,
    enable_ffi_tracking: true,
    enable_real_time_stats: true,
};
```

## 🔗 Related Documentation

- [Performance Optimization Guide](../advanced/performance-optimization.md) - Performance tuning techniques
- [Troubleshooting](troubleshooting.md) - Configuration-related issue resolution
- [CLI Tools](cli-tools.md) - Command-line configuration options

---

Proper configuration is the foundation of efficient memory analysis! ⚙️