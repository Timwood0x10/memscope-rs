//! Unsafe Rust & FFI Memory Analysis - New API
//!
//! This example demonstrates unsafe Rust and FFI memory tracking with memory passport export.

use memscope_rs::{global_tracker, init_global_tracking, track, MemScopeResult};
use std::alloc::{alloc, dealloc, Layout};
use std::time::Instant;

fn main() -> MemScopeResult<()> {
    println!("Unsafe Rust & FFI Memory Analysis - New API");
    println!("============================================\n");

    let start_time = Instant::now();

    init_global_tracking()?;

    let tracker = global_tracker()?;

    println!("1. Safe Rust Allocations");
    let safe_vec = vec![1, 2, 3, 4, 5];
    track!(tracker, safe_vec);
    println!("   Tracked Vec with {} elements", 5);

    let safe_string = String::from("Hello, safe Rust!");
    track!(tracker, safe_string);
    println!("   Tracked String with {} chars", safe_string.len());

    println!("\n2. Unsafe Rust Allocations");
    unsafe {
        let layout = Layout::new::<[i32; 10]>();
        let ptr = alloc(layout);

        if !ptr.is_null() {
            let passport_id = tracker
                .create_passport(
                    ptr as usize,
                    layout.size(),
                    "unsafe_rust_allocation".to_string(),
                )
                .map_err(|e| {
                    memscope_rs::MemScopeError::error("unsafe_ffi_demo", "main", e.to_string())
                })?;

            let slice = std::slice::from_raw_parts_mut(ptr as *mut i32, 10);
            for (i, item) in slice.iter_mut().enumerate() {
                *item = i as i32 * i as i32;
            }

            println!("   Passport {}: {} bytes", passport_id, layout.size());
            dealloc(ptr, layout);
        }
    }

    println!("\n3. FFI Memory Operations");
    extern "C" {
        fn malloc(size: usize) -> *mut std::ffi::c_void;
        fn free(ptr: *mut std::ffi::c_void);
        fn calloc(nmemb: usize, size: usize) -> *mut std::ffi::c_void;
    }

    for i in 0..5 {
        let size = 256 * (i + 1);
        let ffi_ptr = if i % 2 == 0 {
            unsafe { malloc(size) }
        } else {
            unsafe { calloc(size / 8, 8) }
        };

        if !ffi_ptr.is_null() {
            let passport_id = tracker
                .create_passport(ffi_ptr as usize, size, format!("ffi_alloc_{}", i))
                .map_err(|e| {
                    memscope_rs::MemScopeError::error("unsafe_ffi_demo", "main", e.to_string())
                })?;

            tracker.record_handover(
                ffi_ptr as usize,
                "foreign_function".to_string(),
                format!("ffi_call_{}", i),
            );

            unsafe {
                std::ptr::write_bytes(ffi_ptr as *mut u8, (0x40 + i) as u8, size);
                free(ffi_ptr);
            }

            println!(
                "   Passport {}: FFI {} bytes (leaked intentionally)",
                passport_id, size
            );
        }
    }

    let duration = start_time.elapsed();

    println!("\n4. Leak Detection");
    let leak_result = tracker.passport_tracker().detect_leaks_at_shutdown();
    let stats = tracker.get_stats();
    println!("   Total passports created: {}", stats.passport_count);
    println!("   Leaks detected: {}", leak_result.total_leaks);

    println!("\n5. Memory Analysis");
    println!("   Total allocations: {}", stats.total_allocations);
    println!("   Active allocations: {}", stats.active_allocations);
    println!("   Peak memory: {} bytes", stats.peak_memory_bytes);

    println!("\n6. Exporting memory snapshot...");
    let output_path = "MemoryAnalysis/unsafe_ffi_new_api";
    tracker.export_json(output_path)?;
    println!("   memory_snapshots.json");
    println!("   memory_passports.json");
    println!("   leak_detection.json");
    println!("   unsafe_ffi_analysis.json");
    println!("   system_resources.json");
    println!("   async_analysis.json");

    // Export HTML dashboard
    println!("\n7. Exporting HTML dashboard...");
    tracker.export_html(output_path)?;
    println!("   dashboard.html");

    println!("\n============================================");
    println!("Duration: {:.2}ms", duration.as_secs_f64() * 1000.0);

    Ok(())
}
