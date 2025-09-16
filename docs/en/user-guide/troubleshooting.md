# Troubleshooting Guide

This guide covers the most common issues encountered when using memscope-rs and their solutions.

## 🚨 Compilation Errors

### Issue 1: "cannot find macro `track_var` in this scope"

**Error Message**:
```
error: cannot find macro `track_var` in this scope
 --> src/main.rs:5:5
  |
5 |     track_var!(my_vec);
  |     ^^^^^^^^^
```

**Cause**: Macro not properly imported

**Solution**:
```rust
// ✅ Correct import
use memscope_rs::{track_var, init, get_global_tracker};

// Or use full path
memscope_rs::track_var!(my_vec);
```

### Issue 2: "no global memory allocator found"

**Error Message**:
```
error: no global memory allocator found but one is required; 
link to std or add `extern crate alloc` and `#[global_allocator]` as appropriate
```

**Cause**: Using in `no_std` environment or allocator configuration conflict

**Solution**:
```rust
// Solution 1: Ensure using std
// Don't set default-features = false in Cargo.toml

// Solution 2: If must use no_std, disable tracking-allocator feature
[dependencies]
memscope-rs = { version = "0.1.4", default-features = false }
```

### Issue 3: "feature `tracking-allocator` is required"

**Error Message**:
```
error: the feature `tracking-allocator` is required to use the global allocator
```

**Solution**:
```toml
# Ensure feature is enabled in Cargo.toml
[dependencies]
memscope-rs = { version = "0.1.4", features = ["tracking-allocator"] }

# Or use default features (includes tracking-allocator)
[dependencies]
memscope-rs = "0.1.4"
```

## 🏃‍♂️ Runtime Errors

### Issue 4: "failed to initialize memory tracker"

**Error Message**:
```
thread 'main' panicked at 'failed to initialize memory tracker: AlreadyInitialized'
```

**Cause**: Multiple calls to `init()`

**Solution**:
```rust
// ✅ Call only once at program start
fn main() {
    memscope_rs::init(); // Call only once
    
    // Program logic...
}

// ❌ Avoid repeated initialization
fn some_function() {
    // memscope_rs::init(); // Don't call here
}
```

### Issue 5: "export directory creation failed"

**Error Message**:
```
Error: export directory creation failed: Permission denied (os error 13)
```

**Cause**: No write permissions or directory in use

**Solution**:
```rust
// Solution 1: Check current directory permissions
// Ensure program has write access to current directory

// Solution 2: Specify custom output directory
use memscope_rs::ExportOptions;

let options = ExportOptions::new()
    .with_output_directory("/tmp/memscope_analysis") // Use directory with permissions
    .with_create_subdirectory(true);

tracker.export_to_json_with_options("analysis", &options)?;

// Solution 3: Use relative path
let options = ExportOptions::new()
    .with_output_directory("./reports");
```

### Issue 6: "memory tracking not working"

**Symptoms**: Calling `get_stats()` returns all zeros

**Possible Causes and Solutions**:

```rust
// Cause 1: Forgot to call init()
fn main() {
    memscope_rs::init(); // ← Must call this
    
    let data = vec![1, 2, 3];
    memscope_rs::track_var!(data);
}

// Cause 2: tracking-allocator feature not enabled
// Check Cargo.toml:
[dependencies]
memscope-rs = { version = "0.1.4", features = ["tracking-allocator"] }

// Cause 3: In no_std environment
// Use manual tracking mode:
#[cfg(not(feature = "tracking-allocator"))]
fn manual_tracking_example() {
    use memscope_rs::MemoryTracker;
    
    let tracker = MemoryTracker::new();
    // Manually record allocations...
}
```

## 📊 Performance Issues

### Issue 7: "Program running slowly"

**Symptoms**: Program noticeably slower after enabling memscope-rs

**Diagnosis and Solutions**:

```rust
// Check 1: Ensure using zero-overhead macros
// ✅ Zero overhead
track_var!(data);

// ❌ Has overhead
let tracked = track_var_owned!(data);

// Check 2: Avoid excessive tracking
// ✅ Only track important allocations
let important_data = vec![1; 1000000];
track_var!(important_data);

// ❌ Avoid tracking many small objects
for i in 0..10000 {
    let small_data = vec![i]; // Don't track each one
    // track_var!(small_data); // Avoid this
}

// Check 3: Use fast export mode
use memscope_rs::ExportOptions;

let fast_options = ExportOptions::new()
    .with_fast_mode(true)
    .with_minimal_analysis(true);

tracker.export_to_json_with_options("fast_export", &fast_options)?;
```

### Issue 8: "Excessive memory usage"

**Symptoms**: Program memory usage growing abnormally

**Solutions**:

```rust
// Solution 1: Periodically clean tracking data
let tracker = get_global_tracker();
tracker.clear_deallocated_entries(); // Clean up deallocated entries

// Solution 2: Use sampling tracking
static mut TRACK_COUNTER: usize = 0;

macro_rules! sample_track {
    ($var:expr) => {
        unsafe {
            TRACK_COUNTER += 1;
            if TRACK_COUNTER % 100 == 0 { // Only track 1% of allocations
                track_var!($var);
            }
        }
    };
}

// Solution 3: Limit tracking by data size
fn should_track<T>(data: &T) -> bool {
    std::mem::size_of_val(data) > 1024 // Only track >1KB allocations
}

let large_data = vec![0; 2048];
if should_track(&large_data) {
    track_var!(large_data);
}
```

## 🔧 Export Issues

### Issue 9: "JSON export files too large"

**Solutions**:

```rust
use memscope_rs::ExportOptions;

// Solution 1: Enable compression
let options = ExportOptions::new()
    .with_compression(true)
    .with_minimal_analysis(true);

// Solution 2: Filter small allocations
let options = ExportOptions::new()
    .with_size_threshold(1024) // Only export >1KB allocations
    .with_exclude_system_allocations(true);

// Solution 3: Use binary format
tracker.export_to_binary("compact_data.memscope")?;
```

### Issue 10: "HTML report won't open"

**Symptoms**: Generated HTML file shows blank in browser

**Solutions**:

```rust
// Check 1: Ensure file is completely generated
use std::fs;

let html_path = "MemoryAnalysis/report.html";
if let Ok(metadata) = fs::metadata(html_path) {
    if metadata.len() == 0 {
        println!("HTML file is empty, regenerating...");
        tracker.export_to_html("report.html")?;
    }
} else {
    println!("HTML file doesn't exist");
}

// Check 2: Use absolute path
let current_dir = std::env::current_dir()?;
let html_path = current_dir.join("MemoryAnalysis/report.html");
println!("HTML file location: {}", html_path.display());

// Check 3: Verify browser compatibility
// Use modern browsers (Chrome, Firefox, Safari, Edge)
```

## 🧵 Multi-threading Issues

### Issue 11: "Data inconsistency in multi-threaded environment"

**Solution**:

```rust
use std::sync::Arc;
use std::thread;

// ✅ Correct multi-threaded tracking
fn multithreaded_tracking() {
    memscope_rs::init();
    
    let handles: Vec<_> = (0..4).map(|i| {
        thread::spawn(move || {
            let data = vec![i; 1000];
            track_var!(data); // Thread-safe tracking
            
            // Process data...
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Export in main thread
    let tracker = get_global_tracker();
    tracker.export_to_html("multithreaded_analysis.html").unwrap();
}
```

### Issue 12: "Arc/Rc reference counting tracking anomalies"

**Solution**:

```rust
use std::sync::Arc;
use std::rc::Rc;

// ✅ Correct shared pointer tracking
fn shared_pointer_tracking() {
    // Arc - thread-safe
    let arc_data = Arc::new(vec![1, 2, 3]);
    track_var!(arc_data);
    
    let arc_clone = Arc::clone(&arc_data);
    track_var!(arc_clone); // Automatically tracks reference count changes
    
    // Rc - single-threaded
    let rc_data = Rc::new(String::from("test"));
    track_var!(rc_data);
    
    let rc_clone = Rc::clone(&rc_data);
    track_var!(rc_clone); // Automatically tracks reference count changes
}
```

## 🔍 Debugging Techniques

### Enable Verbose Logging
```rust
// Set log level at program start
std::env::set_var("RUST_LOG", "memscope_rs=debug");
env_logger::init();

// Or use tracing
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

### Verify Tracking Status
```rust
fn debug_tracking_status() {
    let tracker = get_global_tracker();
    
    // Check tracker status
    if let Ok(stats) = tracker.get_stats() {
        println!("Tracker status:");
        println!("  Active allocations: {}", stats.active_allocations);
        println!("  Total allocations: {}", stats.total_allocations);
        println!("  Peak memory: {}", stats.peak_memory);
    } else {
        println!("⚠️ Tracker not properly initialized");
    }
    
    // Check feature enablement status
    #[cfg(feature = "tracking-allocator")]
    println!("✅ tracking-allocator feature enabled");
    
    #[cfg(not(feature = "tracking-allocator"))]
    println!("⚠️ tracking-allocator feature not enabled");
}
```

### Minimal Reproduction Example
```rust
// Create minimal problem reproduction example
fn minimal_reproduction() {
    println!("Starting minimal reproduction test...");
    
    // 1. Initialize
    memscope_rs::init();
    println!("✅ Initialization complete");
    
    // 2. Simple tracking
    let test_data = vec![1, 2, 3];
    memscope_rs::track_var!(test_data);
    println!("✅ Tracking complete");
    
    // 3. Get statistics
    let tracker = memscope_rs::get_global_tracker();
    match tracker.get_stats() {
        Ok(stats) => {
            println!("✅ Statistics retrieved successfully: {} allocations", stats.active_allocations);
        }
        Err(e) => {
            println!("❌ Statistics retrieval failed: {}", e);
        }
    }
    
    // 4. Export test
    match tracker.export_to_json("test_export") {
        Ok(_) => println!("✅ Export successful"),
        Err(e) => println!("❌ Export failed: {}", e),
    }
}
```

## 🔧 Common Configuration Issues

### Issue 13: "Features not working as expected"

**Check Configuration**:

```rust
// Verify feature flags in Cargo.toml
[dependencies]
memscope-rs = { 
    version = "0.1.4", 
    features = [
        "tracking-allocator",  // Required for automatic tracking
        "analysis",           // Required for advanced analysis
        "export-html",        // Required for HTML exports
        "export-binary"       // Required for binary exports
    ]
}

// Check runtime configuration
use memscope_rs::TrackingConfig;

let config = TrackingConfig {
    enable_stack_traces: true,
    enable_lifecycle_tracking: true,
    enable_circular_reference_detection: true,
    ..Default::default()
};

memscope_rs::init_with_config(config);
```

### Issue 14: "Binary export/import failures"

**Solutions**:

```rust
// Check 1: Ensure binary feature is enabled
[dependencies]
memscope-rs = { version = "0.1.4", features = ["export-binary"] }

// Check 2: Verify file permissions and disk space
use std::fs;

fn check_export_prerequisites() -> Result<(), Box<dyn std::error::Error>> {
    // Check current directory is writable
    let test_file = "test_write.tmp";
    fs::write(test_file, "test")?;
    fs::remove_file(test_file)?;
    
    // Check available disk space
    let metadata = fs::metadata(".")?;
    println!("Directory metadata: {:?}", metadata);
    
    Ok(())
}

// Check 3: Use error handling for exports
fn robust_export() {
    let tracker = get_global_tracker();
    
    match tracker.export_to_binary("analysis") {
        Ok(_) => println!("Binary export successful"),
        Err(e) => {
            eprintln!("Binary export failed: {}", e);
            // Fallback to JSON export
            if let Err(json_err) = tracker.export_to_json("analysis_fallback") {
                eprintln!("JSON fallback also failed: {}", json_err);
            }
        }
    }
}
```

## 📊 Performance Profiling

### Profile memscope-rs Impact

```rust
use std::time::Instant;

fn profile_tracking_overhead() {
    // Baseline measurement (without tracking)
    let start = Instant::now();
    let mut data = Vec::new();
    for i in 0..10000 {
        data.push(vec![i; 100]);
    }
    let baseline_duration = start.elapsed();
    println!("Baseline (no tracking): {:?}", baseline_duration);
    
    // With tracking measurement
    memscope_rs::init();
    let start = Instant::now();
    let mut tracked_data = Vec::new();
    for i in 0..10000 {
        let item = vec![i; 100];
        track_var!(item);
        tracked_data.push(item);
    }
    let tracking_duration = start.elapsed();
    println!("With tracking: {:?}", tracking_duration);
    
    let overhead = tracking_duration.as_nanos() as f64 / baseline_duration.as_nanos() as f64;
    println!("Tracking overhead: {:.2}x", overhead);
}
```

## 📞 Getting Help

If none of the above solutions solve your problem:

1. **Check Version Compatibility** - Ensure you're using the latest version
2. **Review Example Code** - Check working examples in the `examples/` directory
3. **Submit an Issue** - Provide a minimal reproduction example on GitHub
4. **Read Documentation** - Check the [API documentation](https://docs.rs/memscope-rs)

### Creating a Good Bug Report

```rust
// Include this information in bug reports:
fn bug_report_info() {
    println!("memscope-rs version: {}", env!("CARGO_PKG_VERSION"));
    println!("Rust version: {}", std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string()));
    println!("Target: {}", std::env::consts::ARCH);
    println!("OS: {}", std::env::consts::OS);
    
    #[cfg(feature = "tracking-allocator")]
    println!("tracking-allocator: enabled");
    #[cfg(not(feature = "tracking-allocator"))]
    println!("tracking-allocator: disabled");
    
    // Include minimal reproduction code
    // Include error messages
    // Include expected vs actual behavior
}
```

Remember: Most issues have simple solutions! 🎯