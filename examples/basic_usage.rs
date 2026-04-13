//! Basic usage example for memscope-rs memory visualizer.
//!
//! This example demonstrates the new unified API with:
//! - tracker!() and track!() macros
//! - Analyzer for unified analysis
//! - Export for data visualization

use memscope_rs::{analyzer, global_tracker, init_global_tracking, track, MemScopeResult};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

fn main() -> MemScopeResult<()> {
    println!("Basic Usage Example - Unified API");
    println!("==================================\n");

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

    // Use the unified Analyzer API
    println!("\n=== Unified Analyzer API ===\n");

    let mut az = analyzer(&tracker)?;

    // Full analysis
    let report = az.analyze();
    println!("Analysis Report:");
    println!("  Allocations: {}", report.stats.allocation_count);
    println!("  Total Bytes: {}", report.stats.total_bytes);
    println!("  Peak Bytes: {}", report.stats.peak_bytes);
    println!("  Threads: {}", report.stats.thread_count);
    println!();

    // Leak detection
    let leaks = az.detect().leaks();
    println!("Leak Detection:");
    println!("  Leak Count: {}", leaks.leak_count);
    println!("  Leaked Bytes: {}", leaks.total_leaked_bytes);
    println!();

    // Metrics
    let metrics = az.metrics().summary();
    println!("Metrics:");
    println!("  Allocation Count: {}", metrics.allocation_count);
    println!("  Total Bytes: {}", metrics.total_bytes);
    println!("  Types: {}", metrics.by_type.len());
    println!();

    // Export
    println!("Export:");
    match az.export().to_json() {
        Ok(json) => println!("  JSON length: {} bytes", json.len()),
        Err(e) => println!("  JSON error: {}", e),
    }
    println!();

    // Export to files
    let output_path = "MemoryAnalysis/basic_usage_unified";
    tracker.export_json(output_path)?;
    tracker.export_html(output_path)?;

    println!("Export successful!");
    println!("Files saved to {}/", output_path);

    println!(
        "\nExample finished in {:.2}ms",
        duration.as_secs_f64() * 1000.0
    );

    Ok(())
}
