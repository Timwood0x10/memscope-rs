//! Comprehensive Async Memory Showcase - New API
//!
//! This example demonstrates async memory tracking using the new unified API.
//! Uses the global tracker with tokio for async task tracking.

use memscope_rs::{global_tracker, init_global_tracking, track};

use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Comprehensive Async Showcase - New API");
    println!("======================================\n");

    let start_time = Instant::now();

    init_global_tracking()?;
    println!("✓ Global tracking initialized\n");

    println!("Starting async tasks with global tracker...\n");

    let tasks = (0..4).map(|i| async move {
        let tracker = global_tracker().unwrap();
        match i {
            0 => {
                let data = vec![0i32; 100];
                track!(tracker, data);
                println!("Task 1 (Light): allocated {} bytes", 100 * 4);
                100 * 4
            }
            1 => {
                let mut data = Vec::with_capacity(1000);
                for j in 0..1000 {
                    data.push(format!("Item {}: {}", j, j * j));
                }
                track!(tracker, data);
                println!(
                    "Task 2 (Heavy): allocated ~{} bytes",
                    1000 * std::mem::size_of::<String>()
                );
                1000 * std::mem::size_of::<String>()
            }
            2 => {
                let data = String::from("IO task data");
                track!(tracker, data);
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                println!("Task 3 (IO): allocated {} bytes", data.len());
                data.len()
            }
            _ => {
                let vec_data = vec![1u64; 500];
                let vec_size = 500 * 8;
                track!(tracker, vec_data);

                let string_data = format!("Processed {} items", 500);
                track!(tracker, string_data);

                println!(
                    "Task 4 (Mixed): allocated {} bytes",
                    vec_size + string_data.len()
                );
                vec_size + string_data.len()
            }
        }
    });

    futures::future::join_all(tasks).await;

    let duration = start_time.elapsed();

    let tracker = global_tracker()?;
    let stats = tracker.get_stats();

    println!("\n======================================");
    println!("Async Memory Analysis Results:");
    println!("  Active tasks: 4");
    println!("  Total allocations: {}", stats.total_allocations);
    println!("  Active allocations: {}", stats.active_allocations);
    println!("  Peak memory: {} bytes", stats.peak_memory_bytes);
    println!("  Duration: {:.2}ms", duration.as_secs_f64() * 1000.0);

    println!("\nExporting memory data...");
    let output_path = "MemoryAnalysis/async_showcase_new_api";
    tracker.export_json(output_path)?;
    println!("  memory_snapshots.json");
    println!("  memory_passports.json");
    println!("  leak_detection.json");
    println!("  unsafe_ffi_analysis.json");
    println!("  system_resources.json");
    println!("  async_analysis.json");

    // Export HTML dashboard
    println!("\nExporting HTML dashboard...");
    tracker.export_html(output_path)?;
    println!("  dashboard.html");

    Ok(())
}
