//! Unsafe Rust & FFI Memory Analysis - New API
//!
//! This example demonstrates unsafe Rust and FFI memory tracking with memory passport export.

use memscope_rs::analysis::memory_passport_tracker::{
    initialize_global_passport_tracker, PassportTrackerConfig,
};
use memscope_rs::render_engine::export::{export_snapshot_to_json, ExportJsonOptions};
use memscope_rs::snapshot::MemorySnapshot;
use memscope_rs::{track, tracker};
use std::alloc::{alloc, dealloc, Layout};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Unsafe Rust & FFI Memory Analysis - New API");
    println!("============================================\n");

    let start_time = Instant::now();

    // Initialize Memory Passport
    let config = PassportTrackerConfig {
        detailed_logging: true,
        max_events_per_passport: 100,
        enable_leak_detection: true,
        enable_validation: true,
        max_passports: 10000,
        track_rust_internal_stack: false,
        user_code_prefixes: vec!["examples/".to_string()],
    };
    let passport_tracker = initialize_global_passport_tracker(config);

    // Initialize tracker
    let tracker = tracker!();

    // 1. Safe Rust allocations
    println!("1. Safe Rust Allocations");
    let safe_vec = vec![1, 2, 3, 4, 5];
    track!(tracker, safe_vec);
    println!("   Tracked Vec with {} elements", 5);

    let safe_string = String::from("Hello, safe Rust!");
    track!(tracker, safe_string);
    println!("   Tracked String with {} chars", safe_string.len());

    // 2. Unsafe Rust allocations
    println!("\n2. Unsafe Rust Allocations");
    unsafe {
        let layout = Layout::new::<[i32; 10]>();
        let ptr = alloc(layout);

        if !ptr.is_null() {
            let passport_id = passport_tracker.create_passport(
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

    // 3. FFI memory operations
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
            let passport_id = passport_tracker.create_passport(
                ffi_ptr as usize,
                size,
                format!("ffi_alloc_{}", i),
            )?;

            passport_tracker.record_handover_to_ffi(
                ffi_ptr as usize,
                "foreign_function".to_string(),
                format!("ffi_call_{}", i),
            )?;

            unsafe {
                std::ptr::write_bytes(ffi_ptr as *mut u8, (0x40 + i) as u8, size);
                free(ffi_ptr);
            }

            println!("   Passport {}: FFI {} bytes (leaked intentionally)", passport_id, size);
        }
    }

    let duration = start_time.elapsed();

    // Leak detection
    println!("\n4. Leak Detection");
    let leak_result = passport_tracker.detect_leaks_at_shutdown();
    println!("   Total passports created: {}", passport_tracker.get_stats().total_passports_created);
    println!("   Leaks detected: {}", leak_result.total_leaks);

    // Memory analysis
    let report = tracker.analyze();
    println!("\n5. Memory Analysis");
    println!("   Total allocations: {}", report.total_allocations);
    println!("   Active allocations: {}", report.active_allocations);
    println!("   Peak memory: {} bytes", report.peak_memory_bytes);

    // Export using new API
    println!("\n6. Exporting memory snapshot...");
    let allocations = tracker.inner().get_active_allocations().unwrap_or_default();
    let snapshot = MemorySnapshot::from_allocation_infos(allocations);

    let export_options = ExportJsonOptions::default();
    let output_path = "MemoryAnalysis/unsafe_ffi_new_api";

    export_snapshot_to_json(&snapshot, output_path.as_ref(), &export_options)?;
    println!("   📄 memory_analysis.json");
    println!("   📄 lifetime.json");
    println!("   📄 thread_analysis.json");
    println!("   📄 variable_relationships.json");

    // Export Memory Passport data
    println!("\n7. Exporting Memory Passport data...");
    export_memory_passport_json(&passport_tracker, &leak_result, output_path)?;

    println!("\n============================================");
    println!("Duration: {:.2}ms", duration.as_secs_f64() * 1000.0);

    Ok(())
}

fn export_memory_passport_json(
    passport_tracker: &memscope_rs::analysis::memory_passport_tracker::MemoryPassportTracker,
    leak_result: &memscope_rs::analysis::memory_passport_tracker::LeakDetectionResult,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let passports = passport_tracker.get_all_passports();

    let passport_data: Vec<_> = passports.values().map(|p| {
        serde_json::json!({
            "passport_id": p.passport_id,
            "allocation_ptr": format!("0x{:x}", p.allocation_ptr),
            "size_bytes": p.size_bytes,
            "created_at": p.created_at,
            "updated_at": p.updated_at,
            "lifecycle_events": p.lifecycle_events.len(),
            "status": format!("{:?}", p.status_at_shutdown),
            "metadata": p.metadata,
        })
    }).collect();

    let leak_details: Vec<_> = leak_result.leak_details.iter().map(|detail| {
        serde_json::json!({
            "passport_id": detail.passport_id,
            "memory_address": format!("0x{:x}", detail.memory_address),
            "size_bytes": detail.size_bytes,
            "lifecycle_summary": detail.lifecycle_summary,
        })
    }).collect();

    let json_data = serde_json::json!({
        "metadata": {
            "export_version": "2.0",
            "export_timestamp": chrono::Utc::now().to_rfc3339(),
            "specification": "memory passport tracking",
            "total_passports": passports.len(),
            "leaks_detected": leak_result.total_leaks
        },
        "memory_passports": passport_data,
        "leak_detection": {
            "total_leaks": leak_result.total_leaks,
            "leak_details": leak_details
        }
    });

    let file_path = format!("{}/memory_passports.json", output_path);
    let json_string = serde_json::to_string_pretty(&json_data)?;
    std::fs::write(&file_path, json_string)?;

    println!("   📄 memory_passports.json - {} passports, {} leaks",
             passports.len(), leak_result.total_leaks);

    Ok(())
}