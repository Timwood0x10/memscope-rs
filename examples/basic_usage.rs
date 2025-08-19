//! Basic usage example for memscope-rs memory visualizer.
//!
//! This example demonstrates the new unified API with:
//! - Safe error handling (no unwrap)
//! - Clean export interface
//! - User variables only export (recommended)

use memscope_rs::export::{export_user_variables_binary, export_user_variables_json};
use memscope_rs::{get_global_tracker, init, track_var};
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    // Initialize the memory tracking system
    init();
    println!("memscope-rs initialized. Tracking memory allocations...");

    // Allocate and track simple types
    println!("\nAllocating and tracking variables...");

    let numbers_vec = vec![1, 2, 3, 4, 5];
    track_var!(numbers_vec);
    println!("Tracked 'numbers_vec'");

    let text_string = String::from("Hello, Trace Tools!");
    track_var!(text_string);
    println!("Tracked 'text_string'");

    let boxed_value = Box::new(100i32);
    track_var!(boxed_value);
    println!("Tracked 'boxed_value'");

    let boxed_value2 = Box::new(200i32);
    track_var!(boxed_value2);
    println!("Tracked 'boxed_value2'");

    // Track reference-counted types
    let rc_data = Rc::new(vec![10, 20, 30]);
    track_var!(rc_data);
    println!("Tracked 'rc_data'");

    let arc_data = Arc::new(String::from("Shared data"));
    track_var!(arc_data);
    println!("Tracked 'arc_data'");

    // Clone Rc to show shared ownership
    let rc_data_clone = Rc::clone(&rc_data);
    track_var!(rc_data_clone);
    println!("Tracked 'rc_data_clone' (shares allocation with 'rc_data')");

    // Perform some operations (variables remain fully usable)
    let sum_of_vec = numbers_vec.iter().sum::<i32>();
    println!("\nSum of 'numbers_vec': {sum_of_vec}");
    println!("Length of 'text_string': {}", text_string.len());
    println!("Value in 'boxed_value': {}", *boxed_value);
    println!("Value in 'boxed_value2': {}", *boxed_value2);
    println!("First element of 'rc_data': {}", rc_data[0]);
    println!("Content of 'arc_data': {}", *arc_data);

    // Get memory statistics
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("\nMemory Statistics:");
        println!("  Active allocations: {}", stats.active_allocations);
        println!("  Active memory: {} bytes", stats.active_memory);
        println!("  Total allocations: {}", stats.total_allocations);
        println!("  Peak memory: {} bytes", stats.peak_memory);
    }

    // Export using new unified API - user variables only (recommended)
    println!("\n🚀 Exporting memory snapshot using new unified API...");

    // Get allocations and stats for export
    match (tracker.get_active_allocations(), tracker.get_stats()) {
        (Ok(allocations), Ok(stats)) => {
            // Export to JSON (user variables only - cleaner and faster)
            println!("📋 Exporting user variables to JSON...");
            match export_user_variables_json(
                allocations.clone(),
                stats.clone(),
                "basic_usage_snapshot",
            ) {
                Ok(export_stats) => {
                    println!("✅ JSON export successful!");
                    println!(
                        "   📊 Processed {} allocations in {}ms",
                        export_stats.allocations_processed, export_stats.processing_time_ms
                    );
                    println!("   📁 Files saved to MemoryAnalysis/basic_usage_snapshot_analysis/");
                }
                Err(e) => eprintln!("❌ JSON export failed: {}", e),
            }

            // Export to binary format (efficient for large datasets)
            println!("\n💾 Exporting user variables to binary...");
            match export_user_variables_binary(allocations, stats, "basic_usage.memscope") {
                Ok(export_stats) => {
                    println!("✅ Binary export successful!");
                    println!(
                        "   📊 Processed {} allocations in {}ms",
                        export_stats.allocations_processed, export_stats.processing_time_ms
                    );
                    println!("   📁 Binary file: MemoryAnalysis/basic_usage.memscope");
                }
                Err(e) => eprintln!("❌ Binary export failed: {}", e),
            }

            // Legacy export for comparison (deprecated but still works)
            println!("\n🔄 Legacy export for comparison...");
            if let Err(e) = tracker.export_memory_analysis("basic_usage_graph.svg") {
                eprintln!("⚠️  Legacy SVG export failed: {e}");
            } else {
                println!("✅ Legacy SVG exported to MemoryAnalysis/basic_usage/");
            }
        }
        (Err(e), _) => eprintln!("❌ Failed to get allocations: {}", e),
        (_, Err(e)) => eprintln!("❌ Failed to get stats: {}", e),
    }

    println!("\nExample finished. Check 'basic_usage_snapshot.json' and 'basic_usage_graph.svg'.");
    println!("The SVG shows memory usage by type and individual allocations.");
}
