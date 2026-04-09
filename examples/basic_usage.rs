//! Basic usage example for memscope-rs memory visualizer.
//!
//! This example demonstrates the new unified API with:
//! - tracker!() and track!() macros
//! - MemorySnapshot for snapshot building
//! - global_tracking::export_all_json for data export

use memscope_rs::{global_tracker, init_global_tracking, track, MemScopeResult};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

fn main() -> MemScopeResult<()> {
    println!("Basic Usage Example - New API");
    println!("================================\n");

    let start_time = Instant::now();
    init_global_tracking()?;
    let tracker = global_tracker()?;

    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);

    let string_data = String::from("Hello, world!");
    track!(tracker, string_data);

    let rc_data = Rc::new(vec![1.0, 2.0, 3.0]);
    track!(tracker, rc_data);

    let arc_data = Arc::new(vec![1.0, 2.0, 3.0]);
    track!(tracker, arc_data);

    let boxed_data = Box::new(42);
    track!(tracker, boxed_data);

    let duration = start_time.elapsed();

    let stats = tracker.get_stats();
    println!("\nMemory Analysis Results:");
    println!("  Active allocations: {}", stats.active_allocations);
    println!("  Total allocations: {}", stats.total_allocations);
    println!("  Peak memory: {} bytes", stats.peak_memory_bytes);

    println!("\nExporting memory snapshot using new API...");

    let output_path = "MemoryAnalysis/basic_usage_new_api";

    tracker.export_json(output_path)?;

    // Also export HTML dashboard
    println!("\nExporting HTML dashboard...");
    tracker.export_html(output_path)?;

    println!("Export successful!");
    println!("Files saved to {}/", output_path);
    println!("  memory_snapshots.json");
    println!("  memory_passports.json");
    println!("  leak_detection.json");
    println!("  unsafe_ffi_analysis.json");
    println!("  system_resources.json");
    println!("  async_analysis.json");
    println!("  dashboard.html");

    println!(
        "\nExample finished in {:.2}ms",
        duration.as_secs_f64() * 1000.0
    );

    Ok(())
}
