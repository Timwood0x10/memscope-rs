//! Basic usage example for memscope-rs memory visualizer.
//!
//! This example demonstrates the new unified API with:
//! - tracker!() and track!() macros
//! - MemorySnapshot for snapshot building
//! - global_tracking::export_all_json for data export

use memscope_rs::analysis::memory_passport_tracker::PassportTrackerConfig;
use memscope_rs::capture::backends::global_tracking::export_all_json;
use memscope_rs::{track, tracker};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Basic Usage Example - New API");
    println!("================================\n");

    let start_time = Instant::now();

    let tracker = tracker!();

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

    let report = tracker.analyze();
    println!("\nMemory Analysis Results:");
    println!("  Active allocations: {}", report.active_allocations);
    println!("  Total allocations: {}", report.total_allocations);
    println!("  Peak memory: {} bytes", report.peak_memory_bytes);

    println!("\nExporting memory snapshot using new API...");

    let allocations = tracker.inner().get_active_allocations().unwrap_or_default();

    let output_path = "MemoryAnalysis/basic_usage_new_api";
    let passport_tracker = Arc::new(
        memscope_rs::analysis::memory_passport_tracker::MemoryPassportTracker::new(
            PassportTrackerConfig::default(),
        ),
    );

    export_all_json(output_path, &tracker, &passport_tracker)?;

    // Also export HTML dashboard
    println!("\nExporting HTML dashboard...");
    use memscope_rs::render_engine::export::export_dashboard_html;
    export_dashboard_html(output_path, &tracker, &passport_tracker)?;

    // Export SVG visualizations
    println!("\nExporting SVG visualizations...");
    use memscope_rs::render_engine::export::export_svg;
    export_svg(output_path, &tracker)?;

    println!("Export successful!");
    println!("Files saved to {}/", output_path);
    println!("  memory_analysis.json");
    println!("  lifetime.json");
    println!("  thread_analysis.json");
    println!("  variable_relationships.json");
    println!("  memory_passports.json");
    println!("  leak_detection.json");
    println!("  unsafe_ffi.json");
    println!("  system_resources.json");
    println!("  dashboard.html");
    println!("  memory_analysis.svg");
    println!("  lifecycle_timeline.svg");

    println!(
        "\nExample finished in {:.2}ms",
        duration.as_secs_f64() * 1000.0
    );

    Ok(())
}
