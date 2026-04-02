//! Complex Multi-Thread Memory Tracking Showcase - New API
//!
//! This example demonstrates multi-thread memory tracking using the new unified API.

use memscope_rs::render_engine::export::{export_snapshot_to_json, ExportJsonOptions};
use memscope_rs::snapshot::MemorySnapshot;
use memscope_rs::{track, tracker};
use std::sync::Arc;
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

    println!("\nStarting multi-threaded allocations...\n");

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            thread::spawn(move || {
                let tracker = tracker!();

                // Phase 1: Small allocations
                for i in 0..allocations_per_thread / 2 {
                    let data = vec![0i32; 64];
                    track!(tracker, data);
                }

                // Phase 2: Large allocations
                for i in 0..allocations_per_thread / 2 {
                    let data = vec![0i64; 256];
                    track!(tracker, data);
                }

                println!("Thread {} completed {} allocations", thread_id, allocations_per_thread);
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread failed");
    }

    let duration = start_time.elapsed();
    let total_allocations = num_threads * allocations_per_thread;
    let throughput = total_allocations as f64 / duration.as_secs_f64();

    // Get final analysis
    let tracker = tracker!();
    let report = tracker.analyze();

    println!("\n========================================");
    println!("Memory Analysis Results:");
    println!("  Total allocations: {}", report.total_allocations);
    println!("  Active allocations: {}", report.active_allocations);
    println!("  Peak memory: {} bytes ({:.2} MB)",
             report.peak_memory_bytes,
             report.peak_memory_bytes as f64 / 1024.0 / 1024.0);
    println!("  Duration: {:.2}ms", duration.as_secs_f64() * 1000.0);
    println!("  Throughput: {:.0} allocs/sec", throughput);

    // Export using new API
    println!("\nExporting memory snapshot using new API...");

    let allocations = tracker.inner().get_active_allocations().unwrap_or_default();
    let snapshot = MemorySnapshot::from_allocation_infos(allocations);

    let export_options = ExportJsonOptions::default();
    let output_path = "MemoryAnalysis/multithread_new_api";

    export_snapshot_to_json(&snapshot, output_path.as_ref(), &export_options)?;

    println!("Export successful!");
    println!("Files saved to {}/", output_path);
    println!("  memory_analysis.json");
    println!("  lifetime.json");
    println!("  thread_analysis.json");
    println!("  variable_relationships.json");

    Ok(())
}