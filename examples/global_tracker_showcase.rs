//! Global Tracker Showcase - New API
//!
//! This example demonstrates how to use the global tracker across all execution modes:
//! - Single-threaded mode
//! - Multi-threaded mode
//! - Async mode
//! - Unsafe/FFI mode

use memscope_rs::capture::backends::global_tracking::{
    export_to_json, get_stats, global_passport_tracker, global_tracker, init_global_tracking,
};
use memscope_rs::render_engine::export::export_dashboard_html;
use memscope_rs::{track, tracker};
use std::alloc::{alloc, dealloc, Layout};
use std::thread;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║        Global Tracker Showcase - New Unified API           ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    init_global_tracking()?;
    println!("✓ Global tracking initialized (Tracker + MemoryPassport)\n");

    println!("📦 Section 1: Single-Threaded Mode\n");
    let single_start = Instant::now();
    {
        let tracker = global_tracker()?;
        track!(tracker, vec![1i32, 2, 3, 4, 5]);
        track!(tracker, String::from("Hello, global tracking!"));
        track!(tracker, Box::new(42i64));
        println!("✓ Tracked 3 allocations");
    }
    println!(
        "  Duration: {:.2}ms\n",
        single_start.elapsed().as_secs_f64() * 1000.0
    );

    println!("📦 Section 2: Multi-Threaded Mode\n");
    let multi_start = Instant::now();
    let handles: Vec<_> = (0..4)
        .map(|id| {
            thread::spawn(move || {
                let tracker = global_tracker().unwrap();
                for i in 0..100 {
                    track!(tracker, vec![i as i32; 16]);
                }
                println!("  Thread {}: tracked 100 allocations", id);
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    println!("✓ Multi-threaded completed");
    println!(
        "  Duration: {:.2}ms\n",
        multi_start.elapsed().as_secs_f64() * 1000.0
    );

    println!("📦 Section 3: Async Mode\n");
    let async_start = Instant::now();
    run_async_mode()?;
    println!("✓ Async completed");
    println!(
        "  Duration: {:.2}ms\n",
        async_start.elapsed().as_secs_f64() * 1000.0
    );

    println!("📦 Section 4: Unsafe/FFI Mode\n");
    let unsafe_start = Instant::now();
    run_unsafe_ffi_mode()?;
    println!("✓ Unsafe/FFI completed");
    println!(
        "  Duration: {:.2}ms\n",
        unsafe_start.elapsed().as_secs_f64() * 1000.0
    );

    println!("📦 Section 5: Statistics\n");
    let stats = get_stats()?;
    println!("✓ Total allocations: {}", stats.total_allocations);
    println!("✓ Active allocations: {}", stats.active_allocations);
    println!(
        "✓ Peak memory: {:.2} MB",
        stats.peak_memory_bytes as f64 / 1024.0 / 1024.0
    );
    println!("✓ Memory passports: {}", stats.passport_count);

    println!("\n📦 Section 6: Export (8 files)\n");
    let output_path = "MemoryAnalysis/global_tracker_showcase";
    export_to_json(output_path)?;

    // Also export HTML dashboard
    println!("Exporting HTML dashboard...");
    use memscope_rs::capture::backends::global_tracking::global_tracker;
    let tracker = global_tracker()?;
    let passport_tracker = global_passport_tracker()?;
    export_dashboard_html(output_path, &tracker, &passport_tracker)?;

    // Export SVG visualizations
    println!("\nExporting SVG visualizations...");
    use memscope_rs::render_engine::export::export_svg;
    export_svg(output_path, &tracker)?;

    println!("✓ Export successful!");
    println!("  📄 memory_analysis.json");
    println!("  📄 lifetime.json");
    println!("  📄 thread_analysis.json");
    println!("  📄 variable_relationships.json");
    println!("  📄 memory_passports.json");
    println!("  📄 leak_detection.json");
    println!("  📄 unsafe_ffi.json");
    println!("  📄 dashboard.html");
    println!("  📄 memory_analysis.svg");
    println!("  📄 lifecycle_timeline.svg");

    println!("\n✓ All modes completed successfully!");
    println!(
        "\nOpen {}/dashboard.html in your browser to view the dashboard!",
        output_path
    );
    Ok(())
}

#[tokio::main]
async fn run_async_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("Spawning 4 async tasks...");

    let tasks = (0..4).map(|i| async move {
        let tracker = global_tracker().unwrap();
        track!(tracker, vec![0u64; 50]);
        track!(tracker, format!("Async task: {}", i));
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        println!("  Task-{}: tracked 2 allocations", i);
    });

    futures::future::join_all(tasks).await;
    Ok(())
}

fn run_unsafe_ffi_mode() -> Result<(), Box<dyn std::error::Error>> {
    let passport_tracker = global_passport_tracker()?;

    println!("Spawning unsafe/FFI operations...");

    // Unsafe Rust allocations
    for i in 0..5 {
        unsafe {
            let layout = Layout::new::<[i32; 64]>();
            let ptr = alloc(layout);

            if !ptr.is_null() {
                passport_tracker.create_passport(
                    ptr as usize,
                    layout.size(),
                    format!("unsafe_vec_{}", i),
                )?;

                let slice = std::slice::from_raw_parts_mut(ptr as *mut i32, 64);
                for (j, item) in slice.iter_mut().enumerate() {
                    *item = (i * 100 + j) as i32;
                }

                println!("  Unsafe-{}: {} bytes", i, layout.size());
                dealloc(ptr, layout);
            }
        }
    }

    // FFI operations
    extern "C" {
        fn malloc(size: usize) -> *mut std::ffi::c_void;
        fn free(ptr: *mut std::ffi::c_void);
    }

    for i in 0..5 {
        let size = 256 * (i + 1);
        let ffi_ptr = unsafe { malloc(size) };

        if !ffi_ptr.is_null() {
            passport_tracker.create_passport(ffi_ptr as usize, size, format!("ffi_alloc_{}", i))?;

            passport_tracker.record_handover_to_ffi(
                ffi_ptr as usize,
                "foreign_function".to_string(),
                format!("ffi_call_{}", i),
            )?;

            unsafe {
                std::ptr::write_bytes(ffi_ptr as *mut u8, (0x40 + i) as u8, size);
                free(ffi_ptr);
            }

            println!("  FFI-{}: {} bytes", i, size);
        }
    }

    Ok(())
}
