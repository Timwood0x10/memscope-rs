//! Unsafe Rust & FFI Memory Analysis - New API
//!
//! This example demonstrates unsafe Rust and FFI memory tracking with memory passport export.

use memscope_rs::analysis::memory_passport_tracker::{
    initialize_global_passport_tracker, PassportTrackerConfig,
};
use memscope_rs::capture::backends::global_tracking::export_to_json;
use memscope_rs::{track, tracker};
use std::alloc::{alloc, dealloc, Layout};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Unsafe Rust & FFI Memory Analysis - New API");
    println!("============================================\n");

    let start_time = Instant::now();

    let config = PassportTrackerConfig {
        detailed_logging: true,
        max_events_per_passport: 100,
        enable_leak_detection: true,
        enable_validation: true,
        max_passports: 10000,
        track_rust_internal_stack: false,
        user_code_prefixes: vec!["examples/".to_string()],
    };
    let _passport_tracker = initialize_global_passport_tracker(config);

    let tracker = tracker!();

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
            let passport_id = _passport_tracker.create_passport(
                ptr as usize,
                layout.size(),
                "unsafe_rust_allocation".to_string(),
            )?;

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
            let passport_id = _passport_tracker.create_passport(
                ffi_ptr as usize,
                size,
                format!("ffi_alloc_{}", i),
            )?;

            _passport_tracker.record_handover_to_ffi(
                ffi_ptr as usize,
                "foreign_function".to_string(),
                format!("ffi_call_{}", i),
            )?;

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
    let leak_result = _passport_tracker.detect_leaks_at_shutdown();
    println!(
        "   Total passports created: {}",
        _passport_tracker.get_stats().total_passports_created
    );
    println!("   Leaks detected: {}", leak_result.total_leaks);

    let report = tracker.analyze();
    println!("\n5. Memory Analysis");
    println!("   Total allocations: {}", report.total_allocations);
    println!("   Active allocations: {}", report.active_allocations);
    println!("   Peak memory: {} bytes", report.peak_memory_bytes);

    println!("\n6. Exporting memory snapshot (7 files)...");
    let output_path = "MemoryAnalysis/unsafe_ffi_new_api";
    export_to_json(output_path)?;
    println!("   📄 memory_analysis.json");
    println!("   📄 lifetime.json");
    println!("   📄 thread_analysis.json");
    println!("   📄 variable_relationships.json");
    println!("   📄 memory_passports.json");
    println!("   📄 leak_detection.json");
    println!("   📄 unsafe_ffi.json");

    // Export HTML dashboard
    println!("\n7. Exporting HTML dashboard...");
    use memscope_rs::analysis::memory_passport_tracker::{MemoryPassportTracker, PassportTrackerConfig};
    use memscope_rs::render_engine::export::export_dashboard_html;
    use std::sync::Arc;

    let passport_tracker = Arc::new(MemoryPassportTracker::new(PassportTrackerConfig::default()));
    export_dashboard_html(output_path, &tracker, &passport_tracker)?;
    println!("   📄 dashboard.html");

    // Export SVG visualizations
    println!("\n8. Exporting SVG visualizations...");
    use memscope_rs::render_engine::export::export_svg;
    export_svg(output_path, &tracker)?;
    println!("   📄 memory_analysis.svg");
    println!("   📄 lifecycle_timeline.svg");

    println!("\n============================================");
    println!("Duration: {:.2}ms", duration.as_secs_f64() * 1000.0);

    Ok(())
}
