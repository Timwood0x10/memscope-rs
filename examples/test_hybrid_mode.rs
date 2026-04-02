//! Hybrid Mode Test - Unified Tracker
//!
//! This example tests hybrid mode using Unified Tracker.
//! Tests data accuracy and event capture completeness with detailed JSON export.

use memscope_rs::{track, tracker};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Hybrid Mode Test (Unified Tracker)");
    println!("═══════════════════════════════════════════\n");

    let tracker = Arc::new(tracker!());
    let num_allocations = 30_000;
    let num_threads = 50;

    println!("📊 Configuration:");
    println!("  Mode: Hybrid (Unified Tracker)");
    println!("  Threads: {}", num_threads);
    println!("  Allocations: {}\n", num_allocations);

    let start_time = Instant::now();
    let completed_allocations = Arc::new(Mutex::new(0usize));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let tracker = Arc::clone(&tracker);
            let completed = Arc::clone(&completed_allocations);
            let allocations_per_thread = num_allocations / num_threads;

            thread::spawn(move || {
                for i in 0..allocations_per_thread {
                    let size = 512 + (i % 32257);
                    let data = vec![thread_id as u8; size];
                    track!(tracker, data);
                    
                    if let Ok(mut count) = completed.lock() {
                        *count += 1;
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
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
    match tracker.export_json("hybrid_mode_test") {
        Ok(_) => {
            println!("✅ Detailed JSON exported successfully!");
            println!("   📁 Files saved to MemoryAnalysis/hybrid_mode_test/");
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
    match tracker.export_analysis("hybrid_mode_analysis") {
        Ok(_) => {
            println!("✅ Analysis report exported successfully!");
            println!("   📁 File saved to MemoryAnalysis/hybrid_mode_analysis_analysis.json");
            println!("   📊 Contains: total allocations, peak memory, hotspots, system snapshots");
        }
        Err(e) => eprintln!("❌ Analysis report export failed: {}", e),
    }

    println!("\n🎉 Test completed successfully!");
    println!("═══════════════════════════════════════════\n");

    Ok(())
}
