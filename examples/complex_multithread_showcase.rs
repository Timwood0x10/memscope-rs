//! Complex Multi-Thread Memory Tracking Showcase - New API
//!
//! This example demonstrates multi-thread memory tracking using the new unified API.

use memscope_rs::capture::backends::global_tracking::{
    export_to_json, global_tracker, init_global_tracking,
};
use std::thread;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Complex Multi-Thread Showcase - New API");
    println!("========================================\n");

    let num_threads = 8;
    let allocations_per_thread = 500;

    println!("Configuration:");
    println!("  Threads: {}", num_threads);
    println!("  Allocations per thread: {}", allocations_per_thread);

    let start_time = Instant::now();

    init_global_tracking()?;
    println!("✓ Global tracking initialized\n");

    println!("Starting multi-threaded allocations...\n");

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            thread::spawn(move || {
                let tracker = global_tracker().unwrap();

                for i in 0..allocations_per_thread / 2 {
                    let data = vec![0i32; 64];
                    memscope_rs::track!(tracker, data);
                }

                for i in 0..allocations_per_thread / 2 {
                    let data = vec![0i64; 256];
                    memscope_rs::track!(tracker, data);
                }

                println!(
                    "Thread {} completed {} allocations",
                    thread_id, allocations_per_thread
                );
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread failed");
    }

    let duration = start_time.elapsed();
    let total_allocations = num_threads * allocations_per_thread;
    let throughput = total_allocations as f64 / duration.as_secs_f64();

    let tracker = global_tracker()?;
    let report = tracker.analyze();

    println!("\n========================================");
    println!("Memory Analysis Results:");
    println!("  Total allocations: {}", report.total_allocations);
    println!("  Active allocations: {}", report.active_allocations);
    println!(
        "  Peak memory: {} bytes ({:.2} MB)",
        report.peak_memory_bytes,
        report.peak_memory_bytes as f64 / 1024.0 / 1024.0
    );
    println!("  Duration: {:.2}ms", duration.as_secs_f64() * 1000.0);
    println!("  Throughput: {:.0} allocs/sec", throughput);

    println!("\nExporting memory snapshot (7 files)...");
    let output_path = "MemoryAnalysis/multithread_new_api";
    export_to_json(output_path)?;
    println!("  📄 memory_analysis.json");
    println!("  📄 lifetime.json");
    println!("  📄 thread_analysis.json");
    println!("  📄 variable_relationships.json");
    println!("  📄 memory_passports.json");
    println!("  📄 leak_detection.json");
    println!("  📄 unsafe_ffi.json");

    // Export HTML dashboard
    println!("\nExporting HTML dashboard...");
    use memscope_rs::analysis::memory_passport_tracker::{MemoryPassportTracker, PassportTrackerConfig};
    use memscope_rs::render_engine::export::export_dashboard_html;
    use std::sync::Arc;

    let passport_tracker = Arc::new(MemoryPassportTracker::new(PassportTrackerConfig::default()));
    export_dashboard_html(output_path, &tracker, &passport_tracker)?;
    println!("  📄 dashboard.html");

    // Export SVG visualizations
    println!("\nExporting SVG visualizations...");
    use memscope_rs::render_engine::export::export_svg;
    export_svg(output_path, &tracker)?;
    println!("  📄 memory_analysis.svg");
    println!("  📄 lifecycle_timeline.svg");

    Ok(())
}
