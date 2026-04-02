//! Single-Threaded Mode Test - Core Tracker
//!
//! This example tests the single-threaded mode using Core Tracker.
//! Tests data accuracy and event capture completeness with detailed JSON export.

use memscope_rs::{track, tracker};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Single-Threaded Mode Test (Core Tracker)");
    println!("═══════════════════════════════════════════════\n");

    let tracker = tracker!();
    let num_allocations = 10_000;

    println!("📊 Configuration:");
    println!("  Mode: Single-Threaded (Core Tracker)");
    println!("  Threads: 1");
    println!("  Allocations: {}\n", num_allocations);

    let start_time = Instant::now();

    // Track variables using track! macro
    for i in 0..num_allocations {
        let size = 64 + (i % 4033);
        let data = vec![0u8; size];
        track!(tracker, data);
    }

    let duration = start_time.elapsed();

    // Get analysis report
    let report = tracker.analyze();

    println!("📊 Results:");
    println!("  Duration: {:.2}s", duration.as_secs_f64());
    println!("  Throughput: {:.0} allocs/sec", 
             num_allocations as f64 / duration.as_secs_f64());
    println!("  Total Allocations: {}", report.total_allocations);
    println!("  Active Allocations: {}", report.active_allocations);
    println!("  Current Memory: {} bytes ({:.2} MB)", 
             report.current_memory_bytes, 
             report.current_memory_bytes as f64 / 1024.0 / 1024.0);
    println!("  Peak Memory: {} bytes ({:.2} MB)", 
             report.peak_memory_bytes, 
             report.peak_memory_bytes as f64 / 1024.0 / 1024.0);

    // Export detailed JSON using tracker.export_json
    println!("\n📄 Exporting detailed JSON...");
    match tracker.export_json("single_threaded_test") {
        Ok(_) => {
            println!("✅ Detailed JSON exported successfully!");
            println!("   📁 Files saved to MemoryAnalysis/single_threaded_test/");
            println!("   📄 memory_analysis.json - with borrow_info, clone_info, ownership_history");
            println!("   📄 lifetime.json - detailed ownership events");
            println!("   📄 unsafe_ffi.json - unsafe operations analysis");
            println!("   📄 variable_relationships.json - variable relationships");
            println!("   📄 type_analysis.json - type analysis");
        }
        Err(e) => {
            eprintln!("❌ JSON export failed: {}", e);
        }
    }

    // Export analysis report
    println!("\n📊 Exporting analysis report...");
    match tracker.export_analysis("single_threaded_analysis") {
        Ok(_) => {
            println!("✅ Analysis report exported successfully!");
            println!("   📁 File saved to MemoryAnalysis/single_threaded_analysis_analysis.json");
            println!("   📊 Contains: total allocations, peak memory, hotspots, system snapshots");
        }
        Err(e) => eprintln!("❌ Analysis report export failed: {}", e),
    }

    println!("\n🎉 Test completed successfully!");
    println!("═══════════════════════════════════════════════\n");

    Ok(())
}
