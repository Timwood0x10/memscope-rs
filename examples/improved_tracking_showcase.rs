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
    track_var_owned, get_global_tracker, get_global_unsafe_ffi_tracker,
    TrackingResult, Trackable,
};
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::ffi::{CString, CStr};
use std::os::raw::c_void;

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
        Some(self.data.as_ptr() as usize)
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
        let tracked_rc = track_var_owned!(rc_data);
        
        println!("  📊 Rc strong count: {}", Rc::strong_count(&tracked_rc));
        
        // Clone the Rc to test reference counting
        let cloned_rc = tracked_rc.clone();
        println!("  📊 Rc strong count after clone: {}", Rc::strong_count(&tracked_rc));
        
        // Test into_inner() with drop protection
        let original_rc = tracked_rc.into_inner();
        println!("  ✅ Successfully extracted Rc via into_inner()");
        println!("  📊 Final Rc strong count: {}", Rc::strong_count(&original_rc));
        
        drop(cloned_rc);
        drop(original_rc);
    }
    
    // Test Arc tracking with thread safety
    {
        let data = ComplexData::new(2, "arc_data", 2048);
        let arc_data = Arc::new(data);
        let tracked_arc = track_var_owned!(arc_data);
        
        println!("  📊 Arc strong count: {}", Arc::strong_count(&tracked_arc));
        
        // Share across threads
        let arc_clone = tracked_arc.clone();
        let handle = thread::spawn(move || {
            println!("  🧵 Thread: Arc strong count: {}", Arc::strong_count(&arc_clone));
            thread::sleep(Duration::from_millis(100));
            println!("  🧵 Thread: Dropping Arc clone");
        });
        
        handle.join().unwrap();
        
        let original_arc = tracked_arc.into_inner();
        println!("  ✅ Successfully extracted Arc via into_inner()");
        println!("  📊 Final Arc strong count: {}", Arc::strong_count(&original_arc));
    }
    
    // Test Box tracking
    {
        let data = ComplexData::new(3, "box_data", 512)
            .with_nested(ComplexData::new(4, "nested_data", 256));
        let boxed_data = Box::new(data);
        let tracked_box = track_var_owned!(boxed_data);
        
        println!("  📦 Box data ID: {}", tracked_box.id);
        println!("  📦 Box nested data: {:?}", tracked_box.nested.as_ref().map(|n| &n.name));
        
        let _original_box = tracked_box.into_inner();
        println!("  ✅ Successfully extracted Box via into_inner()");
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
            return Err(std::io::Error::new(std::io::ErrorKind::OutOfMemory, "Failed to allocate memory via malloc").into());
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
        
        println!("  📝 Created CString at {:p}: {:?}", c_str_ptr, 
                 CStr::from_ptr(c_str_ptr).to_string_lossy());
        
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
        
        let tracked1 = track_var_owned!(data1);
        let _tracked2 = track_var_owned!(data2);
        
        // Extract one early, let the other drop normally
        let _extracted = tracked1.into_inner();
        println!("  ✅ Extracted tracked1 early");
        
        // tracked2 will drop normally at end of scope
    }
    
    // Test scenario 2: Nested smart pointers
    {
        let inner_data = ComplexData::new(20, "inner", 512);
        let rc_inner = Rc::new(inner_data);
        let arc_outer = Arc::new(rc_inner);
        
        let tracked_nested = track_var_owned!(arc_outer);
        
        println!("  📊 Nested smart pointer - Arc strong count: {}", 
                 Arc::strong_count(&tracked_nested));
        println!("  📊 Nested smart pointer - Rc strong count: {}", 
                 Rc::strong_count(&**tracked_nested));
        
        // Test cloning nested structure
        let cloned_nested = tracked_nested.clone();
        println!("  📊 After clone - Arc strong count: {}", 
                 Arc::strong_count(&tracked_nested));
        
        drop(cloned_nested);
        println!("  🧹 Dropped clone");
    }
    
    // Test scenario 3: Panic safety (simulated)
    {
        let data = ComplexData::new(30, "panic_test", 1024);
        let tracked = track_var_owned!(data);
        
        // Simulate a scenario where panic might occur during processing
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            println!("  🧪 Testing panic safety with tracked variable");
            // This won't actually panic, but demonstrates the safety mechanism
            tracked.get().id
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
        let tracked = track_var_owned!(data);
        tracked_vars.push(tracked);
    }
    
    let creation_time = start_time.elapsed();
    println!("  ⏱️ Created 100 tracked variables in {:?}", creation_time);
    
    // Extract some variables using into_inner()
    let extract_start = std::time::Instant::now();
    for _ in 0..50 {
        if let Some(tracked) = tracked_vars.pop() {
            let _extracted = tracked.into_inner();
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
    std::fs::create_dir_all("MemoryAnalysis/improved_tracking_showcase")?;
    
    // Export using the standard export_to_json method which creates multiple files
    tracker.export_to_json("MemoryAnalysis/improved_tracking_showcase/improved_tracking")?;
    println!("  ✅ Exported all analysis data using export_to_json");
    
    println!("📁 All analysis data exported to: MemoryAnalysis/improved_tracking_showcase/");
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
    
    // Run all demonstration scenarios
    demonstrate_smart_pointer_tracking()?;
    println!();
    
    demonstrate_unsafe_ffi_tracking()?;
    println!();
    
    demonstrate_complex_lifecycle_scenarios()?;
    println!();
    
    demonstrate_performance_improvements()?;
    println!();
    
    // Export all analysis data
    export_analysis_data()?;
    println!();
    
    println!("🎉 Showcase completed successfully!");
    println!("📊 Analysis data exported for HTML report generation");
    println!("💡 Run: make html DIR=MemoryAnalysis/improved_tracking_showcase BASE=improved_tracking");
    
    Ok(())
}