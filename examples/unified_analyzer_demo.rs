//! Example: Unified Analyzer API
//!
//! Demonstrates the new unified analyzer interface using memscope_rs::start().

use memscope_rs::{analyzer, track, MemScopeResult};

fn main() -> MemScopeResult<()> {
    println!("=== Unified Analyzer API Demo ===\n");

    // 1. One-line start (logging + global tracker + auto-export hooks)
    let guard = memscope_rs::start()?;

    // 2. Track some variables
    let data = vec![1, 2, 3, 4, 5];
    track!(guard, data);

    let map = std::collections::HashMap::<String, i32>::new();
    track!(guard, map);

    let string = String::from("Hello, memscope!");
    track!(guard, string);

    // 3. Create analyzer (single entry point)
    // Note: analyzer() now returns Result<Analyzer, MemScopeError>
    let mut az = match analyzer(&guard) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to create analyzer: {e}");
            return Ok(());
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

    // Auto-export: MemScopeGuard's Drop writes the dashboard + JSON
    // to ./memscope-report/ on exit.
    println!("=== Demo Complete ===");
    Ok(())
}
