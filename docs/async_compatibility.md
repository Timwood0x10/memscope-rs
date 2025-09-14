# Async Compatibility Guide for memscope-rs

## Overview

This document explains how memscope-rs works with async/await and tokio runtime environments, providing solutions for the common conflicts between synchronous memory tracking and asynchronous execution models.

## The Core Problem

### Standard Tracking vs Async Runtime Conflict

memscope-rs uses synchronous primitives that can conflict with async runtimes:

```rust
// This can cause issues in async contexts
#[tokio::main]
async fn main() {
    let data = vec![1, 2, 3];
    track_var!(data); // May block tokio runtime!
    
    tokio::time::sleep(Duration::from_millis(10)).await;
}
```

**Root Cause**: The tracking chain involves synchronous operations that don't yield to the async runtime:
- `std::sync::Mutex` locks (or `parking_lot::Mutex`)
- Heavy memory allocations
- String formatting and cloning
- HashMap operations

## Solution: Async-Safe Tracking

### Method 1: Async Mode Environment Variable

**Quick Solution**: Enable async mode globally:

```rust
#[tokio::main]
async fn main() {
    // Enable async-safe mode
    std::env::set_var("MEMSCOPE_ASYNC_MODE", "1");
    
    let data = vec![1, 2, 3];
    track_var!(data); // Now async-safe!
    
    tokio::time::sleep(Duration::from_millis(10)).await;
}
```

### Method 2: Init Function

**Recommended**: Use the dedicated async initialization:

```rust
use memscope_rs::{init_async, track_var};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize async compatibility
    init_async().await;
    
    let data = vec![1, 2, 3];
    track_var!(data);
    
    // Async operations work normally
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    Ok(())
}
```

### Method 3: Async-Specific Macro

**For explicit async contexts**: Use the dedicated async macro:

```rust
use memscope_rs::track_var_async;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = vec![1, 2, 3];
    track_var_async!(data).await?;
    
    // Continue with async operations
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    Ok(())
}
```

## Technical Implementation

### How Async Mode Works

When `MEMSCOPE_ASYNC_MODE` is enabled, the tracking system:

1. **Skips Heavy Registry Operations**:
   ```rust
   // These expensive operations are bypassed:
   // - VariableRegistry::register_variable()
   // - scope_tracker.associate_variable()
   // - AllocationInfo::enhance_with_type_info()
   ```

2. **Uses Lightweight Logging**:
   ```rust
   // Instead of full tracking, just logs:
   tracing::debug!("Tracked: {} ({})", var_name, type_name);
   ```

3. **Avoids Memory Allocations**:
   - No HashMap insertions
   - No String cloning for registry
   - No complex type analysis
   - No borrow/clone tracking structures

### Performance Comparison

| Mode | Memory Overhead | CPU Overhead | Async Safety | Features |
|------|----------------|--------------|--------------|----------|
| Normal | High | High | ❌ | Full tracking |
| Fast Mode | Medium | Medium | ⚠️ | Reduced tracking |
| Async Mode | Low | Low | ✅ | Basic tracking |

## Async Patterns

### Pattern 1: Web Server with Request Tracking

```rust
use memscope_rs::{init_async, track_var};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_async().await;
    
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    
    loop {
        let (socket, _) = listener.accept().await?;
        
        tokio::spawn(async move {
            let mut buffer = vec![0; 1024];
            track_var!(buffer); // Safe in async context
            
            // Handle request...
        });
    }
}
```

### Pattern 2: Concurrent Data Processing

```rust
use memscope_rs::{init_async, track_var};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_async().await;
    
    let mut handles = Vec::new();
    
    for i in 0..100 {
        let handle = tokio::spawn(async move {
            let data = vec![i; 1000];
            track_var!(data); // Async-safe
            
            // Process data asynchronously
            tokio::time::sleep(Duration::from_millis(10)).await;
            
            data.len()
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let result = handle.await?;
        println!("Processed {} items", result);
    }
    
    Ok(())
}
```

### Pattern 3: Stream Processing

```rust
use memscope_rs::{init_async, track_var};
use tokio_stream::{self as stream, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_async().await;
    
    let mut stream = stream::iter(0..1000);
    
    while let Some(item) = stream.next().await {
        let processed = vec![item * 2; 100];
        track_var!(processed); // Safe in async stream
        
        // Process item asynchronously
        if item % 100 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    Ok(())
}
```

## Compatibility Matrix

### Supported Async Runtimes

| Runtime | Compatibility | Notes |
|---------|---------------|-------|
| tokio | ✅ Full | Primary target runtime |
| async-std | ✅ Full | Works with async mode |
| smol | ✅ Partial | Requires async mode |
| futures | ✅ Full | Executor-agnostic |

### Supported Async Libraries

| Library | Compatibility | Recommendation |
|---------|---------------|----------------|
| hyper | ✅ | Use `init_async()` |
| reqwest | ✅ | Enable async mode |
| tonic (gRPC) | ✅ | Use async mode |
| sqlx | ✅ | Enable async mode |
| serde_json | ✅ | Works normally |
| tracing | ✅ | Native integration |

## Best Practices

### 1. Early Initialization

**Do**:
```rust
#[tokio::main]
async fn main() {
    init_async().await; // First thing
    // Rest of async code...
}
```

**Don't**:
```rust
#[tokio::main]
async fn main() {
    // Some async work first
    init_async().await; // Too late!
}
```

### 2. Environment Setup

**Development**:
```rust
// Full async compatibility
std::env::set_var("MEMSCOPE_ASYNC_MODE", "1");
std::env::set_var("RUST_LOG", "memscope_rs=debug");
```

**Production**:
```rust
// Lightweight async mode
std::env::set_var("MEMSCOPE_ASYNC_MODE", "1");
std::env::set_var("RUST_LOG", "error");
```

### 3. Error Handling

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_async().await;
    
    // Handle tracking errors gracefully
    let data = vec![1, 2, 3];
    if let Err(e) = track_var_async!(data).await {
        eprintln!("Tracking failed: {}", e);
        // Continue execution - tracking failure shouldn't crash app
    }
    
    Ok(())
}
```

## Common Issues and Solutions

### Issue 1: Deadlock in Async Context

**Symptoms**: Application hangs when using `track_var!` in async functions.

**Solution**:
```rust
// Before
async fn process_data() {
    let data = vec![1, 2, 3];
    track_var!(data); // May deadlock
}

// After
async fn process_data() {
    std::env::set_var("MEMSCOPE_ASYNC_MODE", "1");
    let data = vec![1, 2, 3];
    track_var!(data); // Safe
}
```

### Issue 2: Performance Degradation

**Symptoms**: Async application becomes slow when tracking is enabled.

**Solution**:
```rust
// Enable async mode to reduce overhead
init_async().await;

// Or use the fast async macro
track_var_async!(data).await?;
```

### Issue 3: Memory Growth in Long-Running Async Applications

**Symptoms**: Memory usage keeps growing in server applications.

**Solution**:
```rust
#[tokio::main]
async fn main() {
    // Use lightweight async mode
    std::env::set_var("MEMSCOPE_ASYNC_MODE", "1");
    init_async().await;
    
    // Memory tracking won't accumulate indefinitely
}
```

## Migration Guide

### From Sync to Async

**Step 1**: Add async initialization
```rust
// Add to main function
init_async().await;
```

**Step 2**: Update tracking calls (optional)
```rust
// Option A: Keep existing syntax (recommended)
track_var!(data);

// Option B: Use explicit async syntax
track_var_async!(data).await?;
```

**Step 3**: Set environment for production
```rust
std::env::set_var("MEMSCOPE_ASYNC_MODE", "1");
```

## Future Roadmap

Planned improvements for async compatibility:

1. **Native async traits**: `AsyncTrackable` trait for futures
2. **Stream integration**: Direct support for `Stream` and `Sink`
3. **Async export**: Non-blocking export operations
4. **Runtime detection**: Automatic async mode detection
5. **Backpressure handling**: Respect async runtime limits

## Conclusion

memscope-rs provides full async compatibility through:

- **Simple setup**: `init_async()` or environment variables
- **Zero overhead**: Async mode eliminates performance bottlenecks  
- **Runtime safety**: No conflicts with tokio or other async runtimes
- **Backward compatibility**: Existing code works with minimal changes

Enable async mode for any tokio-based application to ensure smooth operation and optimal performance.