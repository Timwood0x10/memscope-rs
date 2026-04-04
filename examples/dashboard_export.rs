//! Dashboard Export Example
//!
//! This example demonstrates how to export memory tracking data
//! using the new simplified export API:
//! - export_json: Exports 8 JSON files
//! - export_html: Automatically detects program characteristics and exports appropriate HTML

use memscope_rs::analysis::memory_passport_tracker::{
    MemoryPassportTracker, PassportTrackerConfig,
};
use memscope_rs::{track, tracker};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Dashboard HTML Export Example");
    println!("==============================\n");

    let start_time = Instant::now();

    // Create tracker
    let tracker = tracker!();

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
    track!(tracker, arc_data.clone()); // Track Arc clone
    track!(tracker, arc_data);
    println!("✓ Tracked Arc<Vec<f64>> with clone");

    let boxed_data = Box::new(42i32);
    track!(tracker, boxed_data);
    println!("✓ Tracked Box<i32>");

    // Create more complex data structures
    let mut complex_data = HashMap::new();
    for i in 0..10 {
        complex_data.insert(format!("key_{}", i), vec![i, i * 2, i * 3]);
    }
    track!(tracker, complex_data);
    println!("✓ Tracked HashMap<String, Vec<i32>> with 10 entries");

    // Create passport tracker
    let passport_tracker = Arc::new(MemoryPassportTracker::new(PassportTrackerConfig::default()));

    // Get statistics
    let report = tracker.analyze();
    println!("\nMemory Analysis Results:");
    println!("  Total allocations: {}", report.total_allocations);
    println!("  Active allocations: {}", report.active_allocations);
    println!("  Peak memory: {} bytes", report.peak_memory_bytes);

    // Export with new simplified API
    let output_path = "MemoryAnalysis/dashboard_export";

    // Export HTML dashboard (auto-detects template based on program characteristics)
    println!("\nExporting HTML dashboard (auto-detecting template)...");
    // Note: In this example, we're using the old tracker API for demonstration
    // The new MemScope API would be: memscope.export_html(&output_path)?;
    use memscope_rs::render_engine::export::export_dashboard_html;
    export_dashboard_html(output_path, &tracker, &passport_tracker)?;
    println!("✓ HTML dashboard exported (template auto-selected)");

    // Export all JSON files
    println!("\nExporting JSON data files (8 files)...");
    use memscope_rs::render_engine::export::export_all_json;
    export_all_json(output_path, &tracker, &passport_tracker)?;
    println!("✓ All JSON files exported");

    let duration = start_time.elapsed();

    println!("\n✅ Export successful!");
    println!("Files saved to {}/", output_path);
    println!("\nExported files:");
    println!("  - HTML dashboard (auto-selected template based on program characteristics)");
    println!("  - 8 JSON files:");
    println!("    • memory_analysis.json (complete memory allocation analysis)");
    println!("    • lifetime.json (ownership and lifetime tracking)");
    println!("    • variable_relationships.json (variable dependency graph)");
    println!("    • system_resources.json (system resource monitoring)");
    println!("    • thread_analysis.json (thread-specific memory stats)");
    println!("    • unsafe_ffi.json (unsafe FFI boundary tracking)");
    println!("    • memory_passports.json (memory lifecycle passports)");
    println!("    • leak_detection.json (potential memory leaks)");

    println!(
        "\nExample finished in {:.2}ms",
        duration.as_secs_f64() * 1000.0
    );
    println!("\nOpen any dashboard in your browser to view the results!");
    println!("Recommended: binary_dashboard.html for comprehensive analysis");

    Ok(())
}
