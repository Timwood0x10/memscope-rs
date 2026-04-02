//! Basic usage example for memscope-rs memory visualizer.
//!
//! This example demonstrates the new unified API with:
//! - tracker!() and track!() macros
//! - MemorySnapshot for snapshot building
//! - render_engine::export for data export

use memscope_rs::render_engine::export::{export_snapshot_to_json, ExportJsonOptions};
use memscope_rs::snapshot::MemorySnapshot;
use memscope_rs::{track, tracker};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Basic Usage Example - New API");
    println!("================================\n");

    let start_time = Instant::now();

    // Initialize tracker using new API
    let tracker = tracker!();
    println!("Tracker initialized");

    // Track simple types using new track! macro
    println!("\nAllocating and tracking variables...");

    let numbers_vec = vec![1, 2, 3, 4, 5];
    track!(tracker, numbers_vec);
    println!("Tracked 'numbers_vec'");

    let text_string = String::from("Hello, Trace Tools!");
    track!(tracker, text_string);
    println!("Tracked 'text_string'");

    let boxed_value = Box::new(100i32);
    track!(tracker, boxed_value);
    println!("Tracked 'boxed_value'");

    let boxed_value2 = Box::new(200i32);
    track!(tracker, boxed_value2);
    println!("Tracked 'boxed_value2'");

    // Track reference-counted types
    let rc_data = Rc::new(vec![10, 20, 30]);
    track!(tracker, rc_data);
    println!("Tracked 'rc_data'");

    let arc_data = Arc::new(String::from("Shared data"));
    track!(tracker, arc_data);
    println!("Tracked 'arc_data'");

    let rc_data_clone = Rc::clone(&rc_data);
    track!(tracker, rc_data_clone);
    println!("Tracked 'rc_data_clone'");

    // Operations
    let sum_of_vec = numbers_vec.iter().sum::<i32>();
    println!("\nSum of 'numbers_vec': {sum_of_vec}");

    let duration = start_time.elapsed();

    // Get analysis report using new API
    let report = tracker.analyze();
    println!("\nMemory Analysis Results:");
    println!("  Active allocations: {}", report.active_allocations);
    println!("  Total allocations: {}", report.total_allocations);
    println!("  Peak memory: {} bytes", report.peak_memory_bytes);

    // Export using new render_engine
    println!("\nExporting memory snapshot using new API...");

    let allocations = tracker.inner().get_active_allocations().unwrap_or_default();
    let snapshot = MemorySnapshot::from_allocation_infos(allocations);

    let export_options = ExportJsonOptions::default();
    let output_path = "MemoryAnalysis/basic_usage_new_api";

    export_snapshot_to_json(&snapshot, output_path.as_ref(), &export_options)?;

    println!("Export successful!");
    println!("Files saved to {}/", output_path);
    println!("  memory_analysis.json");
    println!("  lifetime.json");
    println!("  thread_analysis.json");
    println!("  variable_relationships.json");

    println!("\nExample finished in {:.2}ms", duration.as_secs_f64() * 1000.0);

    Ok(())
}