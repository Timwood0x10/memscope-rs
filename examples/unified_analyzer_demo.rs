//! Example: Unified Analyzer API
//!
//! Demonstrates the new unified analyzer interface.

use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};

fn main() {
    println!("=== Unified Analyzer API Demo ===\n");

    // 1. Initialize
    init_global_tracking().unwrap();
    let tracker = global_tracker().unwrap();

    // 2. Track some variables
    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);

    let map = std::collections::HashMap::<String, i32>::new();
    track!(tracker, map);

    let string = String::from("Hello, memscope!");
    track!(tracker, string);

    // 3. Create analyzer (single entry point)
    // Note: analyzer() now returns Result<Analyzer, MemScopeError>
    let mut az = match analyzer(&tracker) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to create analyzer: {}", e);
            return;
        }
    };

    // 4. Full analysis
    let report = az.analyze();
    println!("Analysis Report:");
    println!("  Allocations: {}", report.stats.allocation_count);
    println!("  Total Bytes: {}", report.stats.total_bytes);
    println!("  Peak Bytes: {}", report.stats.peak_bytes);
    println!("  Threads: {}", report.stats.thread_count);
    println!();

    // 5. Leak detection
    let leaks = az.detect().leaks();
    println!("Leak Detection:");
    println!("  Leak Count: {}", leaks.leak_count);
    println!("  Leaked Bytes: {}", leaks.total_leaked_bytes);
    println!();

    // 6. Metrics
    let metrics = az.metrics().summary();
    println!("Metrics:");
    println!("  Allocation Count: {}", metrics.allocation_count);
    println!("  Total Bytes: {}", metrics.total_bytes);
    println!("  Types: {}", metrics.by_type.len());
    println!();

    // 7. Top allocations
    let top = az.metrics().top_by_size(5);
    println!("Top 5 Allocations by Size:");
    for (i, a) in top.iter().enumerate() {
        println!(
            "  {}: {} bytes ({})",
            i + 1,
            a.size,
            a.type_name.as_deref().unwrap_or("unknown")
        );
    }
    println!();

    // 8. Export
    println!("Export:");
    match az.export().to_json() {
        Ok(json) => println!("  JSON length: {} bytes", json.len()),
        Err(e) => println!("  JSON error: {}", e),
    }
    println!();

    println!("=== Demo Complete ===");
}
