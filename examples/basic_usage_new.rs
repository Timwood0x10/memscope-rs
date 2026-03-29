//! Basic usage example using the new unified tracker API.
//!
//! This demonstrates the simplified API compared to the legacy track_var! approach.
//!
//! Comparison with old API (basic_usage.rs):
//! - Old: 242 lines, 5+ imports, complex export setup
//! - New: ~70 lines, 2 imports, simple one-liner export

use memscope_rs::{track, tracker};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

fn main() {
    let total_start = Instant::now();

    // Initialize with new API - single line!
    let tracker = tracker!();
    println!("memscope-rs initialized with new API.");

    // Track variables - unified syntax for all types
    println!("\nTracking variables...");

    let numbers_vec = vec![1, 2, 3, 4, 5];
    track!(tracker, numbers_vec);
    println!("  Tracked 'numbers_vec'");

    let text_string = String::from("Hello, Trace Tools!");
    track!(tracker, text_string);
    println!("  Tracked 'text_string'");

    let boxed_value = Box::new(100i32);
    track!(tracker, boxed_value);
    println!("  Tracked 'boxed_value'");

    let boxed_value2 = Box::new(200i32);
    track!(tracker, boxed_value2);
    println!("  Tracked 'boxed_value2'");

    let rc_data = Rc::new(vec![10, 20, 30]);
    track!(tracker, rc_data);
    println!("  Tracked 'rc_data'");

    let arc_data = Arc::new(String::from("Shared data"));
    track!(tracker, arc_data);
    println!("  Tracked 'arc_data'");

    // Use variables normally
    println!("\nUsing tracked variables:");
    println!("  Sum of numbers_vec: {}", numbers_vec.iter().sum::<i32>());
    println!("  Length of text_string: {}", text_string.len());
    println!("  Value in boxed_value: {}", *boxed_value);
    println!("  Value in boxed_value2: {}", *boxed_value2);
    println!("  First element of rc_data: {}", rc_data[0]);
    println!("  Content of arc_data: {}", *arc_data);

    // Get statistics
    let stats = tracker.snapshot();
    println!("\nMemory Statistics:");
    println!("  Active allocations: {}", stats.active_allocations);
    println!("  Active memory: {} bytes", stats.active_memory);
    println!("  Total allocations: {}", stats.total_allocations);
    println!("  Peak memory: {} bytes", stats.peak_memory);

    // Export - much simpler than old API!
    println!("\nExporting...");
    match tracker.export_json("basic_usage_new") {
        Ok(_) => println!("✅ Export successful to MemoryAnalysis/basic_usage_new/"),
        Err(e) => println!("❌ Export failed: {}", e),
    }

    let total_time = total_start.elapsed();
    println!("\n========================================");
    println!("Total execution time: {:?}", total_time);
    println!("========================================");
}
