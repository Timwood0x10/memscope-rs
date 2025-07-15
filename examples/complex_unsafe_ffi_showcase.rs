//! Moderate complexity unsafe Rust and FFI memory analysis showcase
//! 
//! This example demonstrates realistic scenarios with:
//! - Multiple FFI libraries (libc, image processing, database)
//! - Various unsafe memory operations
//! - Cross-boundary memory transfers
//! - Safety violations and leak detection

use memscope_rs::{init, get_global_tracker};
use memscope_rs::unsafe_ffi_tracker::{get_global_unsafe_ffi_tracker, BoundaryEventType};
use memscope_rs::unsafe_ffi_visualization::export_unsafe_ffi_dashboard;
use std::alloc::{alloc, dealloc, Layout, GlobalAlloc, System};
use std::ffi::{CString, c_void, c_char, c_int};
use std::slice;

// Mock FFI functions
#[no_mangle]
pub extern "C" fn mock_malloc(size: usize) -> *mut c_void {
    unsafe {
        let layout = Layout::from_size_align(size, 8).unwrap_or_else(|_| Layout::new::<u8>());
        System.alloc(layout) as *mut c_void
    }
}

#[no_mangle]
pub extern "C" fn mock_free(ptr: *mut c_void) {
    unsafe {
        if !ptr.is_null() {
            let layout = Layout::new::<u8>();
            System.dealloc(ptr as *mut u8, layout);
        }
    }
}

#[no_mangle]
pub extern "C" fn image_alloc_buffer(width: c_int, height: c_int) -> *mut c_void {
    let size = (width * height * 4) as usize; // RGBA
    mock_malloc(size)
}

#[no_mangle]
pub extern "C" fn image_free_buffer(ptr: *mut c_void) {
    mock_free(ptr);
}

#[no_mangle]
pub extern "C" fn db_alloc_record(size: usize) -> *mut c_void {
    mock_malloc(size + 32) // Extra metadata
}

#[no_mangle]
pub extern "C" fn db_free_record(ptr: *mut c_void) {
    mock_free(ptr);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize memory tracking
    init();
    println!("🦀 Starting Moderate Complexity Unsafe Rust & FFI Memory Analysis");

    let tracker = get_global_tracker();
    let unsafe_ffi_tracker = get_global_unsafe_ffi_tracker();

    // 1. Image processing scenario
    println!("\n🖼️  1. Image Processing Operations (FFI)");
    
    let mut image_buffers = Vec::new();
    for i in 0..3 {
        let width = 800 + i * 200;
        let height = 600 + i * 100;
        
        unsafe {
            let buffer = image_alloc_buffer(width, height);
            if !buffer.is_null() {
                let size = (width * height * 4) as usize;
                
                // Track FFI allocation
                memscope_rs::track_ffi_alloc!(buffer, size, "libimage", "image_alloc_buffer");
                
                // Record boundary event
                let _ = unsafe_ffi_tracker.record_boundary_event(
                    buffer as usize,
                    BoundaryEventType::FfiToRust,
                    "libimage".to_string(),
                    "rust_image_processor".to_string(),
                );
                
                // Process the image data
                let slice = slice::from_raw_parts_mut(buffer as *mut u8, size);
                for j in 0..size.min(1000) {
                    slice[j] = ((i as usize + j) % 256) as u8;
                }
                
                image_buffers.push(buffer);
                println!("   ✅ Created {}x{} image buffer ({} bytes)", width, height, size);
            }
        }
    }

    // 2. Database record management
    println!("\n🗄️  2. Database Operations (FFI)");
    
    let mut db_records = Vec::new();
    for i in 0..5 {
        let record_size = 128 + i * 64;
        
        unsafe {
            let record = db_alloc_record(record_size);
            if !record.is_null() {
                let total_size = record_size + 32;
                
                // Track FFI allocation
                memscope_rs::track_ffi_alloc!(record, total_size, "libdb", "db_alloc_record");
                
                // Record cross-boundary event
                let _ = unsafe_ffi_tracker.record_boundary_event(
                    record as usize,
                    BoundaryEventType::FfiToRust,
                    "libdb".to_string(),
                    "rust_db_manager".to_string(),
                );
                
                // Write data to record
                let slice = slice::from_raw_parts_mut(record as *mut u8, record_size);
                for j in 0..record_size {
                    slice[j] = ((i * 10 + j) % 256) as u8;
                }
                
                db_records.push(record);
                println!("   ✅ Allocated database record {} ({} bytes)", i, total_size);
            }
        }
    }

    // 3. Unsafe memory operations
    println!("\n⚠️  3. Unsafe Memory Operations");
    
    let mut unsafe_allocations = Vec::new();
    unsafe {
        for i in 0..4 {
            let size = 256 + i * 128;
            let layout = Layout::from_size_align(size, 8).unwrap();
            let ptr = alloc(layout);
            
            if !ptr.is_null() {
                // Track unsafe allocation
                memscope_rs::track_unsafe_alloc!(ptr, size);
                
                // Record boundary event
                let _ = unsafe_ffi_tracker.record_boundary_event(
                    ptr as usize,
                    BoundaryEventType::OwnershipTransfer,
                    "unsafe_rust_block".to_string(),
                    "memory_manager".to_string(),
                );
                
                // Initialize memory with pattern
                let slice = slice::from_raw_parts_mut(ptr, size);
                for j in 0..size {
                    slice[j] = ((i + j) % 256) as u8;
                }
                
                unsafe_allocations.push((ptr, layout));
                println!("   ✅ Unsafe allocation {} ({} bytes)", i, size);
            }
        }
    }

    // 4. String operations across FFI boundary
    println!("\n📝 4. String Operations (FFI)");
    
    let strings = vec![
        "Hello, FFI world!",
        "Complex memory operations",
        "Cross-boundary transfers",
        "Unsafe Rust analysis",
    ];
    
    let mut c_strings = Vec::new();
    for (i, rust_str) in strings.iter().enumerate() {
        unsafe {
            let c_string = CString::new(*rust_str).unwrap();
            let len = c_string.as_bytes().len() + 1;
            
            let c_ptr = mock_malloc(len) as *mut c_char;
            if !c_ptr.is_null() {
                // Track FFI allocation
                memscope_rs::track_ffi_alloc!(c_ptr, len, "libc", "malloc");
                
                // Copy string data
                std::ptr::copy_nonoverlapping(c_string.as_ptr(), c_ptr, len);
                
                // Record boundary crossing
                let _ = unsafe_ffi_tracker.record_boundary_event(
                    c_ptr as usize,
                    BoundaryEventType::RustToFfi,
                    "rust_string".to_string(),
                    "c_string".to_string(),
                );
                
                c_strings.push(c_ptr);
                println!("   ✅ Transferred string {} to C memory ({} bytes)", i, len);
            }
        }
    }

    // 5. Memory reallocation operations
    println!("\n🔄 5. Memory Reallocation (FFI)");
    
    let mut dynamic_buffers = Vec::new();
    unsafe {
        for i in 0..3 {
            let initial_size = 512;
            let mut ptr = mock_malloc(initial_size);
            
            if !ptr.is_null() {
                // Track initial allocation
                memscope_rs::track_ffi_alloc!(ptr, initial_size, "libc", "malloc");
                
                // Simulate growth
                for growth in 1..=2 {
                    let new_size = initial_size * (growth + 1);
                    let new_ptr = mock_malloc(new_size);
                    
                    if !new_ptr.is_null() {
                        // Copy old data
                        std::ptr::copy_nonoverlapping(ptr as *const u8, new_ptr as *mut u8, initial_size);
                        
                        // Free old pointer
                        mock_free(ptr);
                        
                        // Track new allocation
                        memscope_rs::track_ffi_alloc!(new_ptr, new_size, "libc", "realloc_simulation");
                        
                        // Record ownership transfer
                        let _ = unsafe_ffi_tracker.record_boundary_event(
                            new_ptr as usize,
                            BoundaryEventType::OwnershipTransfer,
                            "old_buffer".to_string(),
                            "new_buffer".to_string(),
                        );
                        
                        ptr = new_ptr;
                        println!("   ✅ Reallocated buffer {} to {} bytes", i, new_size);
                    }
                }
                
                dynamic_buffers.push(ptr);
            }
        }
    }

    // 6. Safety violation testing
    println!("\n🚨 6. Safety Violation Detection");
    
    // Test double free
    let test_ptr = mock_malloc(64);
    if !test_ptr.is_null() {
        memscope_rs::track_ffi_alloc!(test_ptr, 64, "libc", "malloc");
        
        // First free
        let _ = unsafe_ffi_tracker.track_enhanced_deallocation(test_ptr as usize);
        mock_free(test_ptr);
        
        // Attempt second free
        match unsafe_ffi_tracker.track_enhanced_deallocation(test_ptr as usize) {
            Ok(_) => println!("   ❌ Double-free not detected"),
            Err(e) => println!("   ✅ Double-free detected: {}", e),
        }
    }
    
    // Test invalid free
    let fake_ptr = 0x12345678 as *mut c_void;
    match unsafe_ffi_tracker.track_enhanced_deallocation(fake_ptr as usize) {
        Ok(_) => println!("   ❌ Invalid free not detected"),
        Err(e) => println!("   ✅ Invalid free detected: {}", e),
    }

    // 7. Create some intentional leaks
    println!("\n💧 7. Memory Leak Simulation");
    
    for i in 0..2 {
        let leak_size = 512 + i * 256;
        let leak_ptr = mock_malloc(leak_size);
        if !leak_ptr.is_null() {
            memscope_rs::track_ffi_alloc!(leak_ptr, leak_size, "libc", "malloc");
            println!("   ⚠️  Created intentional leak {} ({} bytes)", i, leak_size);
            // Not freeing these intentionally
        }
    }

    // 8. Cleanup (most allocations)
    println!("\n🧹 8. Cleanup Operations");
    
    // Clean up image buffers
    for buffer in image_buffers {
        image_free_buffer(buffer);
    }
    
    // Clean up database records
    for record in db_records {
        db_free_record(record);
    }
    
    // Clean up unsafe allocations
    for (ptr, layout) in unsafe_allocations {
        unsafe { dealloc(ptr, layout); }
    }
    
    // Clean up C strings
    for c_str in c_strings {
        mock_free(c_str as *mut c_void);
    }
    
    // Clean up dynamic buffers
    for buffer in dynamic_buffers {
        mock_free(buffer);
    }

    // 9. Analysis and reporting
    println!("\n📊 9. Final Analysis");
    
    // Check for leaks
    let leaks = unsafe_ffi_tracker.detect_leaks(50)?; // Short threshold
    println!("   📊 Potential leaks detected: {}", leaks.len());
    
    // Get statistics
    let stats = tracker.get_stats()?;
    let enhanced_allocations = unsafe_ffi_tracker.get_enhanced_allocations()?;
    let violations = unsafe_ffi_tracker.get_safety_violations()?;
    
    println!("   📊 Total allocations: {}", stats.total_allocations);
    println!("   📊 Enhanced allocations tracked: {}", enhanced_allocations.len());
    println!("   📊 Safety violations: {}", violations.len());
    
    // Count by source and library
    let mut library_counts = std::collections::HashMap::new();
    let mut source_counts = std::collections::HashMap::new();
    let mut total_boundary_events = 0;
    
    for allocation in &enhanced_allocations {
        let source_name = match &allocation.source {
            memscope_rs::unsafe_ffi_tracker::AllocationSource::RustSafe => "Safe Rust",
            memscope_rs::unsafe_ffi_tracker::AllocationSource::UnsafeRust { .. } => "Unsafe Rust",
            memscope_rs::unsafe_ffi_tracker::AllocationSource::FfiC { library_name, .. } => {
                *library_counts.entry(library_name.clone()).or_insert(0) += 1;
                "FFI"
            },
            memscope_rs::unsafe_ffi_tracker::AllocationSource::CrossBoundary { .. } => "Cross-boundary",
        };
        *source_counts.entry(source_name).or_insert(0) += 1;
        total_boundary_events += allocation.cross_boundary_events.len();
    }
    
    println!("\n   📈 Allocation Sources:");
    for (source, count) in source_counts {
        println!("      • {}: {}", source, count);
    }
    
    println!("\n   📈 FFI Libraries:");
    for (library, count) in library_counts {
        println!("      • {}: {}", library, count);
    }
    
    println!("   📈 Total boundary events: {}", total_boundary_events);

    // 10. Generate reports
    println!("\n📁 10. Generating Reports");
    
    tracker.export_memory_analysis("moderate_unsafe_ffi_memory_analysis.svg")?;
    println!("   ✅ Standard memory analysis exported");
    
    tracker.export_lifecycle_timeline("moderate_unsafe_ffi_lifecycle_timeline.svg")?;
    println!("   ✅ Lifecycle timeline exported");
    
    export_unsafe_ffi_dashboard(&unsafe_ffi_tracker, "moderate_unsafe_ffi_dashboard.svg")?;
    println!("   ✅ Moderate complexity unsafe/FFI dashboard exported");
    
    tracker.export_to_json("moderate_unsafe_ffi_memory_snapshot.json")?;
    println!("   ✅ JSON snapshot exported");

    println!("\n🎉 Moderate Complexity Unsafe Rust & FFI Memory Analysis Complete!");
    println!("📁 Generated files:");
    println!("   • moderate_unsafe_ffi_memory_analysis.svg - Standard memory analysis");
    println!("   • moderate_unsafe_ffi_lifecycle_timeline.svg - Variable lifecycle timeline");
    println!("   • moderate_unsafe_ffi_dashboard.svg - 🎯 MODERATE COMPLEXITY ANALYSIS");
    println!("   • moderate_unsafe_ffi_memory_snapshot.json - Raw data export");
    
    println!("\n🔍 Summary:");
    println!("   • FFI Libraries: libc, libimage, libdb");
    println!("   • Operations: image processing, database records, string transfers");
    println!("   • Safety violations: {} detected", violations.len());
    println!("   • Cross-boundary events: {} total", total_boundary_events);
    println!("   • Memory leaks: {} potential leaks", leaks.len());

    Ok(())
}