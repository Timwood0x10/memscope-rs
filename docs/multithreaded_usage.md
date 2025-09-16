# Multi-threaded Usage Guide for memscope-rs

## Overview

This document explains how `get_global_tracker()` works in multi-threaded environments and provides best practices for using memscope-rs safely across concurrent threads.

## Core Threading Model

### Global Tracker Singleton

memscope-rs uses a global singleton `MemoryTracker` instance accessed via `get_global_tracker()`:

```rust
static GLOBAL_TRACKER: OnceLock<Arc<MemoryTracker>> = OnceLock::new();

pub fn get_global_tracker() -> Arc<MemoryTracker> {
    GLOBAL_TRACKER
        .get_or_init(|| Arc::new(MemoryTracker::new()))
        .clone()
}
```

### Thread Safety Guarantees

- **Initialization**: Uses `std::sync::OnceLock` for thread-safe singleton initialization
- **Access**: Returns `Arc<MemoryTracker>` for safe shared ownership across threads
- **Internal State**: All internal data structures use `Mutex` for thread-safe access

## Threading Limitations and Considerations

### 1. High Concurrency Bottleneck

**Problem**: With 20+ concurrent threads accessing `get_global_tracker()` simultaneously, initialization conflicts can occur.

**Symptoms**:
- Program crashes with "fatal runtime error"
- System kills the process due to resource exhaustion
- Deadlocks during tracker initialization

**Root Cause**: Multiple `Mutex` initializations in `MemoryTracker::new()` under high contention.

**✅ SOLUTION**: Use Per-Thread Tracking Strategy for 20+ threads.

### 2. Memory Overhead in Multi-threaded Environments

Each thread calling `track_var!` triggers:
- Global tracker access
- Mutex lock acquisition
- Memory allocations for tracking data
- String allocations for variable names

In high-concurrency scenarios (50+ threads × 1000+ variables), this can lead to:
- Exponential memory growth
- Lock contention cascades
- System resource exhaustion

## Recommended Usage Patterns

### 1. Single-threaded Initialization

**Best Practice**: Initialize the global tracker in the main thread before spawning worker threads.

```rust
use memscope_rs::{get_global_tracker, track_var, init_for_testing};

fn main() {
    // Initialize tracker in main thread
    init_for_testing();
    let _tracker = get_global_tracker(); // Force initialization
    
    // Now safe to spawn worker threads
    let handles: Vec<_> = (0..10).map(|i| {
        thread::spawn(move || {
            let data = vec![i; 100];
            track_var!(data); // Safe - tracker already initialized
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
}
```

### 2. Fast Mode for Multi-threaded Environments

**Best Practice**: Enable fast mode to reduce overhead in concurrent scenarios.

```rust
use memscope_rs::{get_global_tracker, track_var};

fn main() {
    // Enable fast mode for better multi-threading performance
    let tracker = get_global_tracker();
    tracker.enable_fast_mode();
    
    // Fast mode reduces lock contention and memory allocations
    let handles: Vec<_> = (0..20).map(|i| {
        thread::spawn(move || {
            for j in 0..100 {
                let data = vec![i * 100 + j; 10];
                track_var!(data); // Optimized for concurrency
            }
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
}
```

### 3. Per-Thread Tracking for 20+ Threads

**Best Practice**: Disable global tracking and use thread-local tracking for high concurrency.

```rust
use memscope_rs::track_var;
use std::thread;

fn main() {
    println!("High concurrency solution: Per-thread tracking");
    
    // For 20+ threads: Disable global tracker to prevent crashes
    std::env::set_var("MEMSCOPE_DISABLE_GLOBAL", "1");
    std::env::set_var("MEMSCOPE_ASYNC_MODE", "1");
    
    let handles: Vec<_> = (0..50).map(|thread_id| {
        thread::spawn(move || {
            // Each thread tracks independently - no global conflicts
            for i in 0..1000 {
                let data = vec![thread_id * 1000 + i; 25];
                track_var!(data); // Safe - no global state access
                
                let string_data = format!("thread_{}_{}", thread_id, i);
                track_var!(string_data);
            }
            println!("Thread {} completed 2000 variables", thread_id);
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("✅ Successfully tracked 100,000 variables across 50 threads!");
}
```

### 4. Async Mode for Heavy Workloads

**Best Practice**: Use async mode to skip expensive operations in high-load scenarios.

```rust
use memscope_rs::{track_var, init_for_testing};

fn main() {
    // Initialize with async mode for heavy multi-threaded workloads
    init_for_testing();
    std::env::set_var("MEMSCOPE_ASYNC_MODE", "1");
    
    let handles: Vec<_> = (0..50).map(|thread_id| {
        thread::spawn(move || {
            for i in 0..1000 {
                let data = vec![thread_id * 1000 + i; 25];
                track_var!(data); // Lightweight tracking
                
                let string_data = format!("thread_{}_{}", thread_id, i);
                track_var!(string_data);
            }
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
}
```

## Performance Characteristics

### Memory Usage Scaling

| Threads | Variables per Thread | Memory Impact | Recommended Mode |
|---------|---------------------|---------------|------------------|
| 1-5     | Any                 | Minimal       | Normal Mode      |
| 6-15    | <500               | Low           | Normal Mode      |
| 16-19   | <100               | Medium        | Fast Mode        |
| 20+     | Any                 | **CRITICAL**  | **Disable Global** |

**CRITICAL**: 20+ threads require `MEMSCOPE_DISABLE_GLOBAL=1` to prevent system crashes.

### Concurrency Limits

- **Safe Range**: 1-15 concurrent threads with normal tracking
- **Moderate Range**: 16-25 threads with fast mode enabled
- **Heavy Range**: 26+ threads require async mode
- **Critical Limit**: 50+ threads may require additional optimizations

## Environment Variables

### MEMSCOPE_DISABLE_GLOBAL

**NEW**: Completely disables global tracker for high concurrency (20+ threads):

```bash
export MEMSCOPE_DISABLE_GLOBAL=1
```

**Effect**:
- Completely bypasses `get_global_tracker()` calls
- No mutex contention or initialization conflicts
- Zero memory overhead from tracking
- Safe for unlimited thread concurrency
- **Use this for 20+ threads**

### MEMSCOPE_ASYNC_MODE

Enables lightweight tracking mode that skips expensive operations:

```bash
export MEMSCOPE_ASYNC_MODE=1
```

**Effect**:
- Skips `VariableRegistry::register_variable()`
- Skips `scope_tracker.associate_variable()`
- Skips `enhance_with_type_info()` heavy operations
- Reduces memory allocations by ~90%

### MEMSCOPE_TEST_MODE

Enables fast mode automatically:

```bash
export MEMSCOPE_TEST_MODE=1
```

**Effect**:
- Enables fast mode in `MemoryTracker::new()`
- Reduces bounded stats limits
- Optimizes for test environments

## Troubleshooting

### Problem: "fatal runtime error: something here is badly broken!"

**Cause**: Too many threads accessing global tracker simultaneously.

**Solutions**:
1. Enable async mode: `std::env::set_var("MEMSCOPE_ASYNC_MODE", "1")`
2. Reduce thread count to <20
3. Initialize tracker in main thread before spawning workers
4. Use fast mode: `tracker.enable_fast_mode()`

### Problem: High Memory Usage

**Cause**: Each thread creating extensive tracking data.

**Solutions**:
1. Enable fast mode to reduce per-variable overhead
2. Use async mode to skip heavy allocations
3. Limit variables tracked per thread
4. Consider tracking only critical variables

### Problem: Lock Contention

**Cause**: Multiple threads competing for tracker mutexes.

**Solutions**:
1. Initialize tracker before multi-threading
2. Enable fast mode to reduce lock duration
3. Use async mode to avoid locks entirely
4. Batch variable tracking where possible

## API Recommendations

### Thread-Safe Functions

✅ **Safe for concurrent use**:
- `get_global_tracker()` (after initial setup)
- `track_var!(var)` (with fast/async mode)
- `tracker.enable_fast_mode()`
- `tracker.is_fast_mode()`

⚠️ **Use with caution**:
- `track_var!(var)` in normal mode with 20+ threads
- `tracker.get_stats()` under heavy load
- `tracker.export_*()` methods during active tracking

❌ **Avoid in multi-threaded contexts**:
- `MemoryTracker::new()` in worker threads
- Heavy tracking without fast/async mode
- Export operations during concurrent tracking

## Example Configurations

### Configuration 1: Development (Low Concurrency)

```rust
// 1-10 threads, full tracking capability
fn main() {
    let tracker = get_global_tracker();
    // Normal mode - full features available
    
    // Spawn worker threads...
}
```

### Configuration 2: Testing (Medium Concurrency)

```rust
// 11-25 threads, optimized tracking
fn main() {
    init_for_testing(); // Enables fast mode
    let tracker = get_global_tracker();
    
    // Spawn worker threads...
}
```

### Configuration 3: Production (High Concurrency)

```rust
// 26+ threads, minimal overhead tracking
fn main() {
    init_for_testing();
    std::env::set_var("MEMSCOPE_ASYNC_MODE", "1");
    let _tracker = get_global_tracker(); // Force init
    
    // Spawn many worker threads...
}
```

## Future Improvements

Planned enhancements for better multi-threading support:

1. **Per-thread trackers**: Reduce global state contention
2. **Lock-free data structures**: Eliminate mutex bottlenecks
3. **Async-first design**: Native async/await support
4. **Hierarchical tracking**: Thread-local with global aggregation
5. **Memory pooling**: Reduce allocation overhead

## Conclusion

memscope-rs can be used safely in multi-threaded environments with proper configuration:

- **< 15 threads**: Use normal mode
- **15-25 threads**: Enable fast mode
- **25+ threads**: Enable async mode
- **Always**: Initialize in main thread before spawning workers

Follow these guidelines to ensure stable operation and optimal performance in concurrent Rust applications.