# MemScope Unified Backend User Guide

## Overview

The MemScope Unified Backend provides intelligent memory tracking across different runtime environments, automatically selecting optimal strategies for single-threaded, multi-threaded, async, and hybrid applications.

## Quick Start

### Installation

```bash
cargo install memscope-rs
```

### Basic Usage

```bash
# Analyze with automatic strategy selection
memscope analyze --mode auto your_program

# Run with unified tracking
memscope run --track-async cargo run --bin your_async_app
```

## Command Reference

### Analyze Command

Enhanced analysis with unified backend support:

```bash
memscope analyze [OPTIONS] <COMMAND>...
```

#### Options

- `--mode <MODE>`: Tracking mode
  - `unified`: Use unified backend (recommended)
  - `legacy`: Use legacy tracking system
  - `auto`: Automatic detection (default)

- `--strategy <STRATEGY>`: Backend strategy
  - `single-thread`: Optimized for single-threaded applications
  - `thread-local`: Multi-threaded with thread-local storage
  - `async`: Async-aware tracking for Tokio/async-std
  - `hybrid`: Adaptive strategy for complex environments
  - `auto`: Automatic selection (default)

- `--sample-rate <RATE>`: Sampling rate (0.0-1.0, default: 1.0)
- `-e, --export <FORMAT>`: Export format (json, html, svg)
- `-o, --output <FILE>`: Output file path

#### Examples

```bash
# Analyze a Rust program with automatic detection
memscope analyze --mode auto cargo run --bin my_app

# Use specific strategy for async application
memscope analyze --strategy async --export html tokio_app

# High-frequency sampling for detailed analysis
memscope analyze --sample-rate 1.0 --export json intensive_app

# Multi-threaded application analysis
memscope analyze --strategy thread-local parallel_app --jobs 4
```

### Run Command

Execute programs with unified memory tracking:

```bash
memscope run [OPTIONS] <COMMAND>...
```

#### Options

- `--track-async`: Enable async task tracking
- `--detailed-tracking`: Enable detailed allocation tracking
- `--performance-monitoring`: Enable performance monitoring
- `--max-overhead <MB>`: Maximum memory overhead in MB (default: 64)
- `-e, --export <FORMAT>`: Export format (json, html, svg)
- `-o, --output <FILE>`: Output file path

#### Examples

```bash
# Run async application with full tracking
memscope run --track-async --detailed-tracking cargo run --bin async_server

# Performance monitoring mode
memscope run --performance-monitoring --max-overhead 32 cargo test

# Complete feature set
memscope run --track-async --detailed-tracking --performance-monitoring my_program
```

## API Reference

### Core Types

#### UnifiedBackend

Main orchestrator for memory tracking:

```rust
use memscope_rs::unified::{UnifiedBackend, BackendConfig};

let config = BackendConfig {
    auto_detect: true,
    force_strategy: None,
    sample_rate: 1.0,
    max_overhead_percent: 5.0,
};

let backend = UnifiedBackend::initialize(config)?;
```

#### BackendConfig

Configuration for unified backend:

```rust
pub struct BackendConfig {
    /// Enable automatic strategy detection
    pub auto_detect: bool,
    /// Force specific strategy (overrides auto_detect)
    pub force_strategy: Option<TrackingStrategy>,
    /// Sampling rate for performance optimization
    pub sample_rate: f64,
    /// Maximum memory overhead percentage
    pub max_overhead_percent: f64,
}
```

#### TrackingSession

Handle for active tracking sessions:

```rust
let session = backend.start_tracking()?;
println!("Session ID: {}", session.session_id());

// ... run your application ...

let analysis_data = backend.collect_data()?;
```

### Environment Detection

#### Automatic Environment Detection

```rust
use memscope_rs::unified::{detect_environment, RuntimeEnvironment};

let environment = detect_environment()?;
match environment {
    RuntimeEnvironment::SingleThreaded => println!("Single-threaded environment"),
    RuntimeEnvironment::MultiThreaded => println!("Multi-threaded environment"),
    RuntimeEnvironment::AsyncRuntime(_) => println!("Async runtime detected"),
    RuntimeEnvironment::Hybrid => println!("Hybrid environment"),
}
```

#### Manual Strategy Selection

```rust
use memscope_rs::unified::{UnifiedBackend, BackendConfig, TrackingStrategy};

let config = BackendConfig {
    auto_detect: false,
    force_strategy: Some(TrackingStrategy::AsyncOptimized),
    sample_rate: 0.8,
    max_overhead_percent: 3.0,
};

let backend = UnifiedBackend::initialize(config)?;
```

## Integration Examples

### Basic Integration

```rust
use memscope_rs::unified::{unified_quick_start, test_unified_system};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Quick start for immediate use
    let mut backend = unified_quick_start()?;
    
    // Start tracking
    let session = backend.start_tracking()?;
    
    // Your application code here
    let data = vec![1, 2, 3, 4, 5];
    let processed = data.iter().map(|x| x * 2).collect::<Vec<_>>();
    
    // Collect results
    let analysis = backend.collect_data()?;
    println!("Tracked {} bytes", analysis.raw_data.len());
    
    Ok(())
}
```

### Async Application Integration

```rust
use memscope_rs::unified::{UnifiedBackend, BackendConfig, TrackingStrategy};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = BackendConfig {
        auto_detect: false,
        force_strategy: Some(TrackingStrategy::AsyncOptimized),
        sample_rate: 1.0,
        max_overhead_percent: 5.0,
    };
    
    let mut backend = UnifiedBackend::initialize(config)?;
    let session = backend.start_tracking()?;
    
    // Async workload
    let tasks = (0..10).map(|i| {
        tokio::spawn(async move {
            let data = vec![0; 1024 * i];
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            data.len()
        })
    });
    
    let results = futures::future::try_join_all(tasks).await?;
    println!("Completed {} tasks", results.len());
    
    let analysis = backend.collect_data()?;
    println!("Analysis: {} bytes collected", analysis.raw_data.len());
    
    Ok(())
}
```

### Multi-threaded Application Integration

```rust
use memscope_rs::unified::{UnifiedBackend, BackendConfig, TrackingStrategy};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = BackendConfig {
        auto_detect: false,
        force_strategy: Some(TrackingStrategy::ThreadLocal),
        sample_rate: 1.0,
        max_overhead_percent: 5.0,
    };
    
    let mut backend = UnifiedBackend::initialize(config)?;
    let session = backend.start_tracking()?;
    
    // Multi-threaded workload
    let handles: Vec<_> = (0..4).map(|i| {
        thread::spawn(move || {
            let data = vec![i; 1024];
            data.into_iter().sum::<usize>()
        })
    }).collect();
    
    let results: Vec<_> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    
    println!("Thread results: {:?}", results);
    
    let analysis = backend.collect_data()?;
    println!("Analysis completed: {} bytes", analysis.raw_data.len());
    
    Ok(())
}
```

## Output Formats

### JSON Export

```json
{
  "session_id": "session_12345",
  "duration_ms": 1250,
  "strategy_used": "AsyncOptimized",
  "total_allocations": 142,
  "peak_memory_mb": 8.5,
  "data_size_bytes": 2048
}
```

### HTML Report

Generated HTML includes:
- Interactive memory usage charts
- Allocation timeline
- Strategy performance metrics
- Detailed allocation breakdown

### SVG Visualization

Vector graphics showing:
- Memory usage over time
- Allocation patterns
- Performance hotspots

## Performance Tuning

### Sampling Rate Optimization

```bash
# High precision (100% sampling)
memscope analyze --sample-rate 1.0 performance_critical_app

# Balanced performance (80% sampling)
memscope analyze --sample-rate 0.8 regular_app

# Low overhead (20% sampling)
memscope analyze --sample-rate 0.2 production_app
```

### Memory Overhead Control

```bash
# Strict memory limits
memscope run --max-overhead 16 memory_constrained_app

# Relaxed limits for detailed analysis
memscope run --max-overhead 128 analysis_target_app
```

## Troubleshooting

### Common Issues

1. **High Memory Overhead**
   - Reduce sampling rate: `--sample-rate 0.5`
   - Lower overhead limit: `--max-overhead 32`

2. **Async Tracking Issues**
   - Ensure `--track-async` flag is set
   - Use `--strategy async` for async-heavy applications

3. **Multi-threading Problems**
   - Use `--strategy thread-local` for better thread support
   - Enable detailed tracking: `--detailed-tracking`

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug memscope analyze your_program

# Trace level for maximum detail
RUST_LOG=trace memscope run --detailed-tracking your_program
```

## Best Practices

1. **Strategy Selection**
   - Use `auto` mode for general applications
   - Use `async` for Tokio/async-std applications
   - Use `thread-local` for CPU-intensive multi-threaded apps

2. **Performance Optimization**
   - Start with default sampling (1.0)
   - Reduce sampling for production monitoring
   - Increase overhead limits for development analysis

3. **Export Format Choice**
   - Use `html` for interactive analysis
   - Use `json` for programmatic processing
   - Use `svg` for presentations and reports

## Advanced Features

### Environment Analysis

```rust
use memscope_rs::unified::{detect_environment_detailed, DetectionConfig};

let config = DetectionConfig {
    deep_analysis: true,
    confidence_threshold: 0.8,
    timeout_ms: 5000,
};

let analysis = detect_environment_detailed(config)?;
println!("Confidence: {:.2}", analysis.confidence);
println!("Recommended strategy: {:?}", analysis.recommended_strategy);
```

### Custom Strategy Integration

```rust
use memscope_rs::unified::strategies::{StrategyFactory, StrategyPerformance};

let mut factory = StrategyFactory::new();

let performance = StrategyPerformance {
    avg_overhead_percent: 2.1,
    avg_init_time_us: 150.0,
    success_rate: 0.98,
    satisfaction_score: 0.92,
    session_count: 1,
};

factory.record_performance("custom_strategy", performance);
```

## Migration Guide

### From Legacy to Unified

1. **Update CLI commands**:
   ```bash
   # Old
   memscope analyze my_program
   
   # New
   memscope analyze --mode unified my_program
   ```

2. **Update API calls**:
   ```rust
   // Old
   use memscope_rs::core::{MemoryTracker, quick_start};
   
   // New
   use memscope_rs::unified::{UnifiedBackend, unified_quick_start};
   ```

3. **Configuration migration**:
   ```rust
   // Old configuration
   let tracker = MemoryTracker::new()?;
   
   // New configuration
   let backend = unified_quick_start()?;
   ```

The unified backend maintains full backward compatibility while providing enhanced features and performance.