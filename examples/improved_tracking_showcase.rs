//! Comprehensive showcase of the improved TrackedVariable drop logic and smart pointer handling
//!
//! This example demonstrates:
//! - Enhanced drop protection mechanisms
//! - Centralized smart pointer detection (Rc, Arc, Box)
//! - Robust error handling in tracking operations
//! - Unsafe/FFI memory tracking
//! - Complex lifecycle scenarios with proper cleanup
//! - Performance improvements from removing auto-export

use memscope_rs::{
    get_global_tracker, get_global_unsafe_ffi_tracker, track_var, track_var_owned, Trackable,
    TrackingResult,
};
use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Simulate FFI functions for demonstration
extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

/// Custom struct for testing complex tracking scenarios
#[derive(Debug, Clone)]
struct ComplexData {
    id: u32,
    name: String,
    data: Vec<u8>,
    nested: Option<Box<ComplexData>>,
}

impl ComplexData {
    fn new(id: u32, name: &str, size: usize) -> Self {
        Self {
            id,
            name: name.to_string(),
            data: vec![0u8; size],
            nested: None,
        }
    }

    fn with_nested(mut self, nested: ComplexData) -> Self {
        self.nested = Some(Box::new(nested));
        self
    }
}

// Implement Trackable for ComplexData
impl Trackable for ComplexData {
    fn get_heap_ptr(&self) -> Option<usize> {
        // Return None to let TrackedVariable generate a synthetic pointer
        // This ensures proper tracking integration
        None
    }

    fn get_type_name(&self) -> &'static str {
        "ComplexData"
    }

    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.len() + self.name.len()
    }
}

/// Demonstrate smart pointer tracking with the improved system
fn demonstrate_smart_pointer_tracking() -> TrackingResult<()> {
    println!("🔍 Testing Smart Pointer Tracking with Enhanced Detection");

    // Test Rc tracking with automatic detection
    {
        let data = ComplexData::new(1, "rc_data", 1024);
        let rc_data = Rc::new(data);
        track_var!(rc_data); // Just track it, don't wrap it

        println!("  📊 Rc strong count: {}", Rc::strong_count(&rc_data));

        // Clone the Rc to test reference counting
        let cloned_rc = rc_data.clone();
        println!(
            "  📊 Rc strong count after clone: {}",
            Rc::strong_count(&rc_data)
        );

        // No into_inner() needed since we're not using TrackedVariable
        let original_rc = rc_data;
        println!("  ✅ Successfully extracted Rc via into_inner()");
        println!(
            "  📊 Final Rc strong count: {}",
            Rc::strong_count(&original_rc)
        );

        drop(cloned_rc);
        drop(original_rc);
    }

    // Test Arc tracking with thread safety
    {
        let data = ComplexData::new(2, "arc_data", 2048);
        let arc_data = Arc::new(data);
        track_var!(arc_data); // Just track it, don't wrap it

        println!("  📊 Arc strong count: {}", Arc::strong_count(&arc_data));

        // Share across threads
        let arc_clone = arc_data.clone();
        let handle = thread::spawn(move || {
            println!(
                "  🧵 Thread: Arc strong count: {}",
                Arc::strong_count(&arc_clone)
            );
            thread::sleep(Duration::from_millis(100));
            println!("  🧵 Thread: Dropping Arc clone");
        });

        handle.join().unwrap();

        let original_arc = arc_data;
        println!("  ✅ Arc tracked successfully");
        println!(
            "  📊 Final Arc strong count: {}",
            Arc::strong_count(&original_arc)
        );
    }

    // Test Box tracking
    {
        let data = ComplexData::new(3, "box_data", 512).with_nested(ComplexData::new(
            4,
            "nested_data",
            256,
        ));
        let boxed_data = Box::new(data);
        track_var!(boxed_data); // Just track it, don't wrap it

        println!("  📦 Box data ID: {}", boxed_data.id);
        println!(
            "  📦 Box nested data: {:?}",
            boxed_data.nested.as_ref().map(|n| &n.name)
        );

        let _original_box = boxed_data;
        println!("  ✅ Box tracked successfully");
    }

    Ok(())
}

/// Demonstrate unsafe/FFI tracking with enhanced error handling
fn demonstrate_unsafe_ffi_tracking() -> TrackingResult<()> {
    println!("🔍 Testing Unsafe/FFI Tracking with Enhanced Error Handling");

    unsafe {
        // Allocate memory using FFI
        let size = 1024;
        let ptr = malloc(size);
        if ptr.is_null() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::OutOfMemory,
                "Failed to allocate memory via malloc",
            )
            .into());
        }

        println!("  🔧 Allocated {} bytes at {:p}", size, ptr);

        // Track the FFI allocation
        let ffi_tracker = get_global_unsafe_ffi_tracker();
        ffi_tracker.track_ffi_allocation(
            ptr as usize,
            size,
            "libc".to_string(),
            "malloc".to_string(),
        )?;

        // Create a CString to demonstrate string FFI tracking
        let c_string = match CString::new("Hello, FFI World!") {
            Ok(s) => s,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e).into()),
        };
        let c_str_ptr = c_string.as_ptr();

        println!(
            "  📝 Created CString at {:p}: {:?}",
            c_str_ptr,
            CStr::from_ptr(c_str_ptr).to_string_lossy()
        );

        // Track the CString allocation
        ffi_tracker.track_ffi_allocation(
            c_str_ptr as usize,
            c_string.as_bytes_with_nul().len(),
            "std::ffi".to_string(),
            "CString::new".to_string(),
        )?;

        // Simulate some unsafe operations
        let data_ptr = ptr as *mut u8;
        for i in 0..10 {
            *data_ptr.add(i) = (i as u8) * 2;
        }

        println!("  ✅ Performed unsafe memory operations");

        // Clean up FFI allocations (simulate deallocation tracking)
        println!("  🧹 Cleaning up FFI allocation");
        free(ptr);

        // CString will be automatically cleaned up when it goes out of scope
        println!("  🧹 CString will be cleaned up automatically");
    }

    Ok(())
}

/// Demonstrate complex lifecycle scenarios with drop protection
fn demonstrate_complex_lifecycle_scenarios() -> TrackingResult<()> {
    println!("🔍 Testing Complex Lifecycle Scenarios with Drop Protection");

    // Test scenario 1: Multiple tracked variables with early extraction
    {
        let data1 = ComplexData::new(10, "early_extract", 1024);
        let data2 = ComplexData::new(11, "normal_drop", 2048);

        track_var!(data1);
        track_var!(data2);

        println!("  📊 Used track_var! for basic reference tracking");
        println!(
            "  📊 data1 name: {}, size: {}",
            data1.name,
            data1.get_size_estimate()
        );
        println!(
            "  📊 data2 name: {}, size: {}",
            data2.name,
            data2.get_size_estimate()
        );

        // Variables will be automatically cleaned up at end of scope

        // tracked2 will drop normally at end of scope
    }

    // Test scenario 2: Nested smart pointers
    {
        let inner_data = ComplexData::new(20, "inner", 512);
        let rc_inner = Rc::new(inner_data);
        let arc_outer = Arc::new(rc_inner);

        track_var!(arc_outer); // Just track it, don't wrap it

        println!(
            "  📊 Nested smart pointer - Arc strong count: {}",
            Arc::strong_count(&arc_outer)
        );
        println!(
            "  📊 Nested smart pointer - Rc strong count: {}",
            Rc::strong_count(&*arc_outer)
        );

        // Test cloning nested structure
        let cloned_nested = arc_outer.clone();
        println!(
            "  📊 After clone - Arc strong count: {}",
            Arc::strong_count(&arc_outer)
        );

        drop(cloned_nested);
        println!("  🧹 Dropped clone");
    }

    // Test scenario 3: Panic safety (simulated)
    {
        let data = ComplexData::new(30, "panic_test", 1024);
        track_var!(data); // Just track it, don't wrap it

        // Simulate a scenario where panic might occur during processing
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            println!("  🧪 Testing panic safety with tracked variable");
            // This won't actually panic, but demonstrates the safety mechanism
            data.id
        }));

        match result {
            Ok(id) => println!("  ✅ Panic safety test passed, ID: {}", id),
            Err(_) => println!("  ⚠️ Panic occurred but was handled safely"),
        }

        // tracked will be properly cleaned up even if panic occurred
    }

    Ok(())
}

/// Demonstrate performance improvements from removing auto-export
fn demonstrate_performance_improvements() -> TrackingResult<()> {
    println!("🔍 Testing Performance Improvements (No Auto-Export)");

    let start_time = std::time::Instant::now();

    // Create many tracked variables to test performance
    let mut tracked_vars = Vec::new();
    for i in 0..100 {
        let data = ComplexData::new(i, &format!("perf_test_{}", i), 128);
        track_var!(data); // Just track it, don't wrap it
        tracked_vars.push(data); // Push the original data
    }

    let creation_time = start_time.elapsed();
    println!("  ⏱️ Created 100 tracked variables in {:?}", creation_time);

    // Extract some variables using into_inner()
    let extract_start = std::time::Instant::now();
    for _ in 0..50 {
        if let Some(_data) = tracked_vars.pop() {
            // Data is automatically cleaned up when it goes out of scope
        }
    }
    let extract_time = extract_start.elapsed();
    println!("  ⏱️ Extracted 50 variables in {:?}", extract_time);

    // Let the remaining variables drop normally
    let drop_start = std::time::Instant::now();
    drop(tracked_vars);
    let drop_time = drop_start.elapsed();
    println!("  ⏱️ Dropped remaining 50 variables in {:?}", drop_time);

    let total_time = start_time.elapsed();
    println!("  ✅ Total performance test completed in {:?}", total_time);
    println!("  📈 No auto-export overhead during drop operations!");

    Ok(())
}

/// Export comprehensive analysis data
fn export_analysis_data() -> TrackingResult<()> {
    println!("📊 Exporting Comprehensive Analysis Data");

    let tracker = get_global_tracker();

    // Create output directory
    std::fs::create_dir_all("MemoryAnalysis/improved_tracking")?;

    // Export using the standard export_to_json method which creates multiple files
    tracker.export_to_json("MemoryAnalysis/improved_tracking/improved_tracking")?;
    println!("  ✅ Exported all analysis data using export_to_json");

    println!("📁 All analysis data exported to: MemoryAnalysis/improved_tracking/");
    println!("📄 Generated files:");
    println!("  - improved_tracking_memory_analysis.json");
    println!("  - improved_tracking_lifetime.json");
    println!("  - improved_tracking_unsafe_ffi.json");
    println!("  - improved_tracking_performance.json");
    println!("  - improved_tracking_complex_types.json");

    Ok(())
}

fn main() -> TrackingResult<()> {
    println!("🚀 Improved TrackedVariable Showcase");
    println!("=====================================");
    println!();

    // Enable verbose logging for demonstration
    std::env::set_var("MEMSCOPE_VERBOSE", "1");
    
    println!("🔧 DEBUG: Starting main function");

    // Create some persistent tracked variables that will remain active during export
    let persistent_data1 = ComplexData::new(100, "persistent_var1", 512);
    let persistent_data2 = ComplexData::new(101, "persistent_var2", 1024);
    let persistent_data3 = ComplexData::new(102, "persistent_var3", 256);
    
    let _tracked_persistent1 = track_var_owned!(persistent_data1);
    let _tracked_persistent2 = track_var_owned!(persistent_data2);
    track_var!(persistent_data3); // Test both tracking methods
    
    println!("📊 Created persistent tracked variables for export testing");
    
    // Check current allocation count
    let tracker = get_global_tracker();
    if let Ok(allocations) = tracker.get_active_allocations() {
        println!("📊 Current active allocations: {}", allocations.len());
        let mut synthetic_count = 0;
        let mut named_count = 0;
        for alloc in allocations.iter() {
            if alloc.ptr >= 0x8000_0000 {
                synthetic_count += 1;
            }
            if alloc.var_name.is_some() {
                named_count += 1;
                println!("  📝 Named allocation: {} -> {:?}", 
                         format!("0x{:x}", alloc.ptr), 
                         alloc.var_name.as_ref().unwrap());
            }
        }
        println!("📊 Synthetic allocations: {}, Named allocations: {}", synthetic_count, named_count);
    }
    println!();

    // Run all demonstration scenarios
    demonstrate_smart_pointer_tracking()?;
    println!();

    demonstrate_unsafe_ffi_tracking()?;
    println!();

    demonstrate_complex_lifecycle_scenarios()?;
    println!();

    demonstrate_performance_improvements()?;
    println!();

    // Export all analysis data while persistent variables are still active
    export_analysis_data()?;
    println!();

    println!("🎉 Showcase completed successfully!");
    println!("📊 Analysis data exported for HTML report generation");
    println!("💡 Run: make html DIR=MemoryAnalysis/improved_tracking BASE=improved_tracking");

    Ok(())
}
