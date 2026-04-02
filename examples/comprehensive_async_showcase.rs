//! Comprehensive Async Memory Showcase - New API
//!
//! This example demonstrates async memory tracking using the new unified API.
//! Uses the global tracker with tokio for async task tracking.

use memscope_rs::render_engine::export::{export_snapshot_to_json, ExportJsonOptions};
use memscope_rs::snapshot::MemorySnapshot;
use memscope_rs::{track, tracker};
use std::sync::Arc;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Comprehensive Async Showcase - New API");
    println!("======================================\n");

    let start_time = Instant::now();

    println!("Starting async tasks with global tracker...\n");

    // Track async tasks using global tracker
    let tracker = Arc::new(tracker!());

    // Task 1: Light computation
    let tracker1 = tracker.clone();
    let handle1 = tokio::spawn(async move {
        let data = vec![0i32; 100];
        track!(tracker1, data);
        println!("Task 1 (Light): allocated {} bytes", 100 * 4);
        data.len()
    });

    // Task 2: Heavy computation
    let tracker2 = tracker.clone();
    let handle2 = tokio::spawn(async move {
        let mut data = Vec::with_capacity(1000);
        for i in 0..1000 {
            data.push(format!("Item {}: {}", i, i * i));
        }
        track!(tracker2, data);
        println!("Task 2 (Heavy): allocated ~{} bytes", 1000 * std::mem::size_of::<String>());
        data.len()
    });

    // Task 3: IO simulation
    let tracker3 = tracker.clone();
    let handle3 = tokio::spawn(async move {
        let data = String::from("IO task data");
        track!(tracker3, data);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        println!("Task 3 (IO): allocated {} bytes", data.len());
        data.len()
    });

    // Task 4: Mixed workload
    let tracker4 = tracker.clone();
    let handle4 = tokio::spawn(async move {
        let vec_data = vec![1u64; 500];
        let vec_size = 500 * 8;
        track!(tracker4, vec_data);

        let string_data = format!("Processed {} items", 500);
        track!(tracker4, string_data);

        println!("Task 4 (Mixed): allocated {} bytes", vec_size + string_data.len());
        vec_size + string_data.len()
    });

    // Wait for all tasks
    futures::future::join_all([handle1, handle2, handle3, handle4]).await;

    let duration = start_time.elapsed();

    // Get analysis
    let report = tracker.analyze();

    println!("\n======================================");
    println!("Async Memory Analysis Results:");
    println!("  Active tasks: 4");
    println!("  Total allocations: {}", report.total_allocations);
    println!("  Active allocations: {}", report.active_allocations);
    println!("  Peak memory: {} bytes", report.peak_memory_bytes);
    println!("  Duration: {:.2}ms", duration.as_secs_f64() * 1000.0);

    // Export
    println!("\nExporting memory data...");
    let allocations = tracker.inner().get_active_allocations().unwrap_or_default();
    let snapshot = MemorySnapshot::from_allocation_infos(allocations);

    let export_options = ExportJsonOptions::default();
    let output_path = "MemoryAnalysis/async_showcase_new_api";

    export_snapshot_to_json(&snapshot, output_path.as_ref(), &export_options)?;

    println!("Export successful!");
    println!("Files saved to {}/", output_path);
    println!("  📄 memory_analysis.json");
    println!("  📄 lifetime.json");
    println!("  📄 thread_analysis.json");
    println!("  📄 variable_relationships.json");

    Ok(())
}