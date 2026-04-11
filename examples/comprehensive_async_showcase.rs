//! Comprehensive Async Memory Showcase - New API
//!
//! This example demonstrates async memory tracking using the new unified API.
//! Uses the global tracker with tokio for async task tracking.
//!
//! ## New APIs Demonstrated
//!
//! - `spawn_tracked()`: Spawns a task with automatic task context management
//! - `TrackerContext::capture()`: Captures current thread + task context
//! - `track_in_tokio_task()`: Wraps a future with automatic task tracking
//! - `detect_zombie_tasks()`: Detects tasks that never completed
//! - `zombie_task_stats()`: Gets zombie task statistics

use memscope_rs::{
    analyzer,
    capture::backends::async_tracker::{spawn_tracked, TrackerContext},
    global_tracker, init_global_tracking, track, MemScopeResult,
};

use std::time::Instant;

#[tokio::main]
async fn main() -> MemScopeResult<()> {
    println!("Comprehensive Async Showcase - New API");
    println!("======================================\n");

    let start_time = Instant::now();

    init_global_tracking()?;
    println!("✓ Global tracking initialized\n");

    // Demonstrate TrackerContext capture
    println!("=== TrackerContext Demo ===\n");
    let ctx = TrackerContext::capture();
    println!(
        "Main context - Thread: {}, Task: {:?}, Tokio: {:?}",
        ctx.thread_id, ctx.task_id, ctx.tokio_task_id
    );

    println!("Starting async tasks with spawn_tracked()...\n");

    // Spawn multiple tracked tasks
    let handles: Vec<_> = (0..4)
        .map(|i| {
            spawn_tracked(async move {
                let tracker = global_tracker().unwrap();
                let ctx = TrackerContext::capture();

                println!(
                    "Task {} started - Thread: {}, Task: {:?}",
                    i, ctx.thread_id, ctx.task_id
                );

                let result = match i {
                    0 => {
                        let data = vec![0i32; 100];
                        track!(tracker, data);
                        println!("Task {} (Light): allocated {} bytes", i, 100 * 4);
                        100 * 4
                    }
                    1 => {
                        let mut data = Vec::with_capacity(1000);
                        for j in 0..1000 {
                            data.push(format!("Item {}: {}", j, j * j));
                        }
                        track!(tracker, data);
                        println!(
                            "Task {} (Heavy): allocated ~{} bytes",
                            i,
                            1000 * std::mem::size_of::<String>()
                        );
                        1000 * std::mem::size_of::<String>()
                    }
                    2 => {
                        let data = String::from("IO task data");
                        track!(tracker, data);
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        println!("Task {} (IO): allocated {} bytes", i, data.len());
                        data.len()
                    }
                    _ => {
                        let vec_data = vec![1u64; 500];
                        let vec_size = 500 * 8;
                        track!(tracker, vec_data);

                        let string_data = format!("Processed {} items", 500);
                        track!(tracker, string_data);

                        println!(
                            "Task {} (Mixed): allocated {} bytes",
                            i,
                            vec_size + string_data.len()
                        );
                        vec_size + string_data.len()
                    }
                };

                // Capture context at end
                let end_ctx = TrackerContext::capture();
                println!(
                    "Task {} completed - Thread: {}, Task: {:?}",
                    i, end_ctx.thread_id, end_ctx.task_id
                );

                result
            })
        })
        .collect();

    // Wait for all tasks to complete
    for (i, handle) in handles.into_iter().enumerate() {
        if let Ok(result) = handle.await {
            println!("Task {} returned: {} bytes", i, result);
        }
    }

    let duration = start_time.elapsed();

    let tracker = global_tracker()?;
    let stats = tracker.get_stats();

    // Demonstrate zombie task detection
    println!("\n=== Zombie Task Detection ===\n");
    let zombies = tracker.async_tracker().detect_zombie_tasks();
    let (zombie_count, total) = tracker.async_tracker().zombie_task_stats();

    if zombies.is_empty() {
        println!("✓ No zombie tasks detected");
    } else {
        println!(
            "⚠ {} zombie task(s) detected out of {} total:",
            zombie_count, total
        );
        for task_id in zombies {
            println!("  - Zombie task ID: {}", task_id);
        }
    }

    println!("\n======================================");
    println!("Async Memory Analysis Results:");
    println!("  Active tasks: {}", total);
    println!("  Zombie tasks: {}", zombie_count);
    println!("  Total allocations: {}", stats.total_allocations);
    println!("  Active allocations: {}", stats.active_allocations);
    println!("  Peak memory: {} bytes", stats.peak_memory_bytes);
    println!("  Duration: {:.2}ms", duration.as_secs_f64() * 1000.0);

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
