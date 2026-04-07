//! Dashboard Export Example
//!
//! This example demonstrates how to export memory tracking data
//! using the new simplified export API.

use memscope_rs::{global_tracker, init_global_tracking, track};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Dashboard HTML Export Example");
    println!("==============================\n");

    let start_time = Instant::now();

    init_global_tracking()?;
    let tracker = global_tracker()?;

    // Track some allocations
    println!("Tracking allocations...\n");

    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);
    println!("✓ Tracked Vec<i32>: {} elements", data.len());

    let string_data = String::from("Hello, MemScope Dashboard!");
    track!(tracker, string_data);
    println!("✓ Tracked String: {} bytes", string_data.len());

    let rc_data = std::rc::Rc::new(vec![1.0, 2.0, 3.0]);
    track!(tracker, rc_data);
    println!("✓ Tracked Rc<Vec<f64>>");

    let arc_data = std::sync::Arc::new(vec![1.0, 2.0, 3.0]);
    track!(tracker, arc_data.clone());
    track!(tracker, arc_data);
    println!("✓ Tracked Arc<Vec<f64>> with clone");

    let boxed_data = Box::new(42i32);
    track!(tracker, boxed_data);
    println!("✓ Tracked Box<i32>");

    // Create more complex data structures
    let mut complex_data = std::collections::HashMap::new();
    for i in 0..10 {
        complex_data.insert(format!("key_{}", i), vec![i, i * 2, i * 3]);
    }
    track!(tracker, complex_data);
    println!("✓ Tracked HashMap<String, Vec<i32>> with 10 entries");

    // Get statistics
    let stats = tracker.get_stats();
    println!("\nMemory Analysis Results:");
    println!("  Total allocations: {}", stats.total_allocations);
    println!("  Active allocations: {}", stats.active_allocations);
    println!("  Peak memory: {} bytes", stats.peak_memory_bytes);

    // Export with new simplified API
    let output_path = "MemoryAnalysis/dashboard_export";

    // Export HTML dashboard
    println!("\nExporting HTML dashboard...");
    tracker.export_html(output_path)?;
    println!("✓ HTML dashboard exported");

    // Export all JSON files
    println!("\nExporting JSON data files...");
    tracker.export_json(output_path)?;
    println!("✓ All JSON files exported");

    let duration = start_time.elapsed();

    println!("\n✅ Export successful!");
    println!("Files saved to {}/", output_path);
    println!("\nExported files:");
    println!("  - dashboard.html (interactive HTML dashboard)");
    println!("  - 6 JSON files:");
    println!("    • memory_snapshots.json");
    println!("    • memory_passports.json");
    println!("    • leak_detection.json");
    println!("    • unsafe_ffi_analysis.json");
    println!("    • system_resources.json");
    println!("    • async_analysis.json");

    println!(
        "\nExample finished in {:.2}ms",
        duration.as_secs_f64() * 1000.0
    );

    Ok(())
}
