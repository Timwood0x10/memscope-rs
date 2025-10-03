# Core Tracking Modules

memscope-rs provides three specialized tracking modules designed for different concurrency scenarios, plus a hybrid mode that combines their capabilities.

## 🎯 Module Overview

| Module | Use Case | Performance | Precision | Best For |
|--------|----------|-------------|-----------|----------|
| **Single-threaded** | Basic tracking | Zero overhead | Exact | Development, debugging |
| **Multi-threaded (lockfree)** | High concurrency | Sampling-based | Approximate | Production, 20+ threads |
| **Async Tasks** | Task-centric | < 5ns overhead | Task-level | async/await applications |
| **Hybrid** | Mixed scenarios | Adaptive | Combined | Complex applications |

## 📦 1. Single-threaded Module (Default)

### Features
- **Zero-overhead tracking** with `track_var!` macro
- **Precise lifecycle management** with `track_var_owned!`
- **Smart type detection** with `track_var_smart!`
- **Real-time analysis** and interactive HTML reports

### API Usage
```rust
use memscope_rs::{track_var, track_var_smart, track_var_owned};

fn main() {
    memscope_rs::init();
    
    // Zero-overhead tracking (recommended)
    let data = vec![1, 2, 3, 4, 5];
    track_var!(data);
    
    // Smart tracking (automatic optimization)
    let number = 42i32;        // Copy type - copied
    let text = String::new();  // Non-copy - tracked by reference
    track_var_smart!(number);
    track_var_smart!(text);
    
    // Ownership tracking (precise lifecycle)
    let owned_data = vec![1, 2, 3];
    let tracked = track_var_owned!(owned_data);
    println!("Data: {:?}", tracked.get());
    
    // Export analysis
    let tracker = memscope_rs::get_tracker();
    tracker.export_to_json("analysis.json").unwrap();
}
```

### Example: Basic Usage
```bash
cargo run --example basic_usage
```

**Generated files:**
- `MemoryAnalysis/basic_usage.json` - Raw tracking data
- `MemoryAnalysis/basic_usage.html` - Interactive dashboard

## 🔀 2. Multi-threaded Module (Lockfree)

### Features
- **Thread-local tracking** with zero shared state
- **Lock-free design** for high concurrency (100+ threads)
- **Intelligent sampling** for performance optimization
- **Binary format** for efficient data storage
- **Comprehensive platform metrics** (CPU, GPU, I/O)

### API Usage
```rust
use memscope_rs::lockfree::{trace_all, stop_tracing, export_comprehensive_analysis};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start tracking all threads
    trace_all("./MemoryAnalysis")?;
    
    // Spawn multiple threads
    let handles: Vec<_> = (0..30).map(|i| {
        thread::spawn(move || {
            // Thread-local tracking happens automatically
            let data = vec![0u8; 1024 * 1024]; // 1MB allocation
            thread::sleep(std::time::Duration::from_millis(100));
            
            // Simulate work
            for j in 0..1000 {
                let temp = vec![i, j];
                drop(temp);
            }
        })
    }).collect();
    
    // Wait for threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Stop tracking and export
    stop_tracing()?;
    export_comprehensive_analysis("./MemoryAnalysis", "comprehensive_analysis")?;
    
    Ok(())
}
```

### Example: Complex Multi-thread
```bash
cargo run --example complex_multithread_showcase
```

**Generated files:**
- `MemoryAnalysis/complex_showcase_dashboard.html` - Comprehensive dashboard
- `MemoryAnalysis/*.bin` - Binary tracking data (high performance)

## ⚡ 3. Async Module

### Features
- **Task-centric tracking** for async/await applications
- **Zero-overhead task identification** using waker addresses
- **Lock-free event buffering** with quality monitoring
- **Production-grade reliability** with data integrity monitoring

### API Usage
```rust
use memscope_rs::async_memory::{initialize, spawn_tracked, get_memory_snapshot};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize async tracking
    initialize().await?;
    
    // Spawn tracked tasks
    let task1 = spawn_tracked(async {
        let data = vec![0u8; 1024 * 1024]; // 1MB allocation
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        data.len()
    });
    
    let task2 = spawn_tracked(async {
        let mut results = Vec::new();
        for i in 0..1000 {
            results.push(format!("Task {}", i));
            tokio::task::yield_now().await;
        }
        results.len()
    });
    
    // Execute tasks
    let (result1, result2) = tokio::try_join!(task1, task2)?;
    println!("Results: {}, {}", result1, result2);
    
    // Get memory snapshot
    let snapshot = get_memory_snapshot();
    println!("Active tasks: {}", snapshot.active_task_count());
    println!("Total memory: {} bytes", snapshot.total_memory_usage());
    
    Ok(())
}
```

### Example: Comprehensive Async
```bash
cargo run --example comprehensive_async_showcase
```

**Generated files:**
- `AsyncAnalysis/async_dashboard.html` - Task-centric analysis
- `AsyncAnalysis/task_profiles.json` - Individual task metrics

## 🔄 4. Hybrid Module

### Features
- **Combined analysis** from all three modules
- **Unified dashboard** with cross-module insights
- **Automatic optimization** based on workload patterns
- **Rich visualization** with performance correlations

### API Usage
```rust
use memscope_rs::export::fixed_hybrid_template::{
    FixedHybridTemplate, create_sample_hybrid_data, RenderMode
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create hybrid analysis combining all modules
    let thread_count = 30;
    let task_count = 100;
    
    // Generate comprehensive hybrid data
    let hybrid_data = create_sample_hybrid_data(thread_count, task_count);
    
    // Create HTML dashboard
    let template = FixedHybridTemplate::new(RenderMode::Complete);
    let html_content = template.generate_hybrid_dashboard(&hybrid_data)?;
    
    // Write comprehensive dashboard
    std::fs::write("hybrid_dashboard.html", html_content)?;
    
    println!("✅ Hybrid analysis complete: hybrid_dashboard.html");
    
    Ok(())
}
```

### Example: Enhanced 30-Thread Demo
```bash
cargo run --example enhanced_30_thread_demo
```

**Generated files:**
- `hybrid_dashboard.html` - Unified analysis dashboard
- Combines thread, task, and single-threaded insights

## 🎛️ Configuration Options

### Single-threaded Configuration
```rust
// Fast mode for testing
std::env::set_var("MEMSCOPE_TEST_MODE", "1");

// Auto-export on program exit
memscope_rs::enable_auto_export(Some("final_analysis"));
```

### Multi-threaded Configuration
```rust
use memscope_rs::lockfree::SamplingConfig;

// Custom sampling configuration
let config = SamplingConfig {
    sample_rate: 0.1,        // 10% sampling rate
    max_events: 1000000,     // 1M events per thread
    buffer_size: 64 * 1024,  // 64KB buffer
};
```

### Async Configuration
```rust
use memscope_rs::async_memory::VisualizationConfig;

let config = VisualizationConfig {
    max_tracked_tasks: 10000,
    buffer_size: 1024 * 1024,  // 1MB per thread
    enable_task_hierarchy: true,
};
```

## 📊 Performance Characteristics

### Export Performance (Real Test Data)

| Module | Export Time | File Size | Use Case |
|--------|-------------|-----------|----------|
| Single-threaded | 1.3s | 1.2MB | Development analysis |
| Multi-threaded | 211ms | 480KB | Production monitoring |
| Async | 800ms | 800KB | Task performance analysis |
| Hybrid | 2.1s | 2.5MB | Comprehensive analysis |

*Based on actual test results from example applications*

### Memory Overhead

| Module | Per-thread Overhead | Tracking Overhead | Runtime Impact |
|--------|-------------------|------------------|----------------|
| Single-threaded | ~100KB | Zero (reference-based) | < 0.1% |
| Multi-threaded | ~64KB | Sampling-based | < 0.5% |
| Async | ~1MB | < 5ns per allocation | < 0.1% |
| Hybrid | Variable | Adaptive | < 1% |

## 🔧 Choosing the Right Module

### Use Single-threaded when:
- ✅ Development and debugging
- ✅ Single-threaded applications
- ✅ Need exact precision
- ✅ Real-time analysis required

### Use Multi-threaded when:
- ✅ High concurrency (20+ threads)
- ✅ Performance is critical
- ✅ Production monitoring
- ✅ Approximate tracking acceptable

### Use Async when:
- ✅ async/await applications
- ✅ Task-level analysis needed
- ✅ Complex async patterns
- ✅ Need task hierarchy insights

### Use Hybrid when:
- ✅ Complex applications with mixed patterns
- ✅ Need comprehensive analysis
- ✅ Comparing different approaches
- ✅ Advanced performance optimization

## 🚀 Quick Start Commands

```bash
# Try each module:
cargo run --example basic_usage                    # Single-threaded
cargo run --example complex_multithread_showcase   # Multi-threaded  
cargo run --example comprehensive_async_showcase   # Async
cargo run --example enhanced_30_thread_demo        # Hybrid

# Generate HTML reports:
make html DIR=MemoryAnalysis BASE=basic_usage
```

---

**Next:** [Single-threaded Module Details](single-threaded.md) | [Multi-threaded Module Details](multithread.md) | [Async Module Details](async.md)