//! Unsafe Rust & FFI Memory Analysis - New API
//!
//! This example demonstrates unsafe Rust and FFI memory tracking with memory passport export.
//! Also demonstrates variable relationships for the relationship graph.

use memscope_rs::{analyzer, global_tracker, init_global_tracking, track, MemScopeResult};
use std::alloc::{alloc, dealloc, Layout};
use std::sync::Arc;
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

    println!("\n2. Variable Relationships Demo");

    // Owner relationship: Box contains pointer to heap allocation
    let boxed_value = Box::new(42u64);
    track!(tracker, boxed_value);
    println!("   Box<u64> - Owner relationship");

    // Shared relationship: Arc with multiple references
    let shared_data = Arc::new(vec![1u64, 2, 3, 4, 5]);
    track!(tracker, shared_data);
    let shared_clone1 = Arc::clone(&shared_data);
    let shared_clone2 = Arc::clone(&shared_data);
    println!("   Arc<Vec<u64>> - Shared relationship (3 references)");

    // Clone relationship: same type, size, stack
    let original_vec = vec![10u64, 20, 30, 40, 50];
    track!(tracker, original_vec);
    let cloned_vec = original_vec.clone();
    track!(tracker, cloned_vec);
    println!("   Cloned Vec - Clone relationship");

    // Slice relationship: slice pointing into another allocation
    let large_array = vec![100u64, 200, 300, 400, 500, 600, 700, 800];
    track!(tracker, large_array);
    let slice_ref: &[u64] = &large_array[2..6];
    println!("   Slice reference - Slice relationship");

    // Nested structures with pointers
    let outer = Box::new(vec![1000u64, 2000, 3000]);
    track!(tracker, outer);
    println!("   Box<Vec<u64>> - Nested ownership");

    println!("\n3. Unsafe Rust Allocations");
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

    println!("\n4. FFI Memory Operations");
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

    // Keep some allocations alive for relationship detection
    // NOTE: std::mem::forget is used here intentionally for demonstration purposes.
    // This prevents the Arc clones from being dropped, keeping the reference count
    // alive so that the relationship graph can show shared ownership relationships.
    // In production code, you would use proper lifecycle management instead.
    println!("\n5. Keeping allocations alive for analysis...");
    std::mem::forget(shared_clone1);
    std::mem::forget(shared_clone2);
    // Suppress unused variable warning for slice reference (intentionally kept alive)
    let _ = slice_ref;

    let duration = start_time.elapsed();

    println!("\n6. Leak Detection");
    let leak_result = tracker.passport_tracker().detect_leaks_at_shutdown();
    let stats = tracker.get_stats();
    println!("   Total passports created: {}", stats.passport_count);
    println!("   Leaks detected: {}", leak_result.total_leaks);

    println!("\n7. Memory Analysis");
    println!("   Total allocations: {}", stats.total_allocations);
    println!("   Active allocations: {}", stats.active_allocations);
    println!("   Peak memory: {} bytes", stats.peak_memory_bytes);

    // Use the unified Analyzer API
    println!("\n=== Unified Analyzer API ===\n");
    let mut az = analyzer(&tracker)?;

    // Full analysis
    let report = az.analyze();
    println!("Analysis Report:");
    println!("  Allocations: {}", report.stats.allocation_count);
    println!("  Total Bytes: {}", report.stats.total_bytes);
    println!("  Peak Bytes: {}", report.stats.peak_bytes);

    // Leak detection
    let leaks = az.detect().leaks();
    println!("\nLeak Detection:");
    println!("  Leak Count: {}", leaks.leak_count);
    println!("  Leaked Bytes: {}", leaks.total_leaked_bytes);

    // Metrics
    let metrics = az.metrics().summary();
    println!("\nMetrics:");
    println!("  Types: {}", metrics.by_type.len());

    println!("\n8. Exporting memory snapshot...");
    let output_path = "MemoryAnalysis/unsafe_ffi_new_api";
    tracker.export_json(output_path)?;
    println!("   memory_snapshots.json");
    println!("   memory_passports.json");
    println!("   leak_detection.json");
    println!("   unsafe_ffi_analysis.json");
    println!("   system_resources.json");
    println!("   async_analysis.json");

    // Export HTML dashboard
    println!("\n9. Exporting HTML dashboard...");
    tracker.export_html(output_path)?;
    println!("   dashboard.html");

    println!("\n============================================");
    println!("Duration: {:.2}ms", duration.as_secs_f64() * 1000.0);
    println!("\nNote: Variable Relationship Graph should now show:");
    println!("  - Owner relationships (Box<T>)");
    println!("  - Shared relationships (Arc<T>)");
    println!("  - Clone relationships (vec.clone())");
    println!("  - Slice relationships (&[T])");

    Ok(())
}
