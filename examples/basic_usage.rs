//! Basic usage example for memscope-rs memory visualizer.
//!
//! This example demonstrates the new unified API with:
//! - Safe error handling (no unwrap)
//! - Clean export interface
//! - User variables only export (recommended)

use memscope_rs::export::enhanced_json_exporter::{EnhancedJsonExporter, ExportConfig};
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

    // Export using new enhanced API with improve.md extensions
    println!("\n🚀 Exporting memory snapshot using enhanced API with improve.md extensions...");

    // Get allocations and stats for export
    match (tracker.get_active_allocations(), tracker.get_stats()) {
        (Ok(allocations), Ok(stats)) => {
            // First, export using legacy API for comparison
            println!("📋 Exporting user variables to JSON (legacy)...");
            match export_user_variables_json(
                allocations.clone(),
                stats.clone(),
                "basic_usage_snapshot",
            ) {
                Ok(export_stats) => {
                    println!("✅ Legacy JSON export successful!");
                    println!(
                        "   📊 Processed {} allocations in {}ms",
                        export_stats.allocations_processed, export_stats.processing_time_ms
                    );
                    println!("   📁 Files saved to MemoryAnalysis/basic_usage_snapshot_analysis/");
                }
                Err(e) => eprintln!("❌ Legacy JSON export failed: {e}"),
            }

            // Now export using enhanced API with improve.md extensions
            println!("\n🆕 Exporting with improve.md extensions (borrow_info, clone_info, ownership_history)...");
            let enhanced_config = ExportConfig {
                pretty_print: true,
                include_stack_traces: true,
                generate_lifetime_file: true,
                generate_unsafe_ffi_file: true,
                max_ownership_events: 100,
            };

            let enhanced_exporter = EnhancedJsonExporter::new(enhanced_config);
            let enhanced_output_dir = "MemoryAnalysis/basic_usage_enhanced";

            match enhanced_exporter.export_enhanced_analysis(
                enhanced_output_dir,
                &stats,
                &[], // No unsafe reports in basic usage
                &[], // No memory passports in basic usage
            ) {
                Ok(_) => {
                    println!("✅ Enhanced JSON export successful!");
                    println!("   📁 Enhanced files saved to: {enhanced_output_dir}/");
                    println!("   📄 memory_analysis.json - with borrow_info, clone_info, ownership_history_available");
                    println!(
                        "   📄 lifetime.json - detailed ownership events and lifecycle tracking"
                    );

                    // Show what's different in the enhanced export
                    show_enhanced_features(enhanced_output_dir);
                }
                Err(e) => eprintln!("❌ Enhanced JSON export failed: {e}"),
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
                Err(e) => eprintln!("❌ Binary export failed: {e}"),
            }

            // Legacy export for comparison (deprecated but still works)
            println!("\n🔄 Legacy export for comparison...");
            if let Err(e) = tracker.export_memory_analysis("basic_usage_graph.svg") {
                eprintln!("⚠️  Legacy SVG export failed: {e}");
            } else {
                println!("✅ Legacy SVG exported to MemoryAnalysis/basic_usage/");
            }
        }
        (Err(e), _) => eprintln!("❌ Failed to get allocations: {e}"),
        (_, Err(e)) => eprintln!("❌ Failed to get stats: {e}"),
    }

    println!("\nExample finished. Check both legacy and enhanced exports:");
    println!("📁 Legacy: MemoryAnalysis/basic_usage_snapshot_analysis/");
    println!("📁 Enhanced: MemoryAnalysis/basic_usage_enhanced/");
    println!("🔍 Compare the files to see improve.md extensions in action!");
}

/// Show the enhanced features in the exported files
fn show_enhanced_features(output_dir: &str) {
    println!("\n🔍 Analyzing enhanced export features...");

    // Check memory_analysis.json for improve.md extensions
    let memory_analysis_path = format!("{output_dir}/memory_analysis.json");
    if let Ok(content) = std::fs::read_to_string(&memory_analysis_path) {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(allocations) = json_value["allocations"].as_array() {
                println!(
                    "   📊 Found {} allocations in enhanced export",
                    allocations.len()
                );

                // Check for improve.md extensions
                let mut has_borrow_info = 0;
                let mut has_clone_info = 0;
                let mut has_ownership_history = 0;
                let mut has_lifetime_ms = 0;

                for alloc in allocations {
                    if alloc.get("borrow_info").is_some() && !alloc["borrow_info"].is_null() {
                        has_borrow_info += 1;
                    }
                    if alloc.get("clone_info").is_some() && !alloc["clone_info"].is_null() {
                        has_clone_info += 1;
                    }
                    if alloc.get("ownership_history_available").is_some() {
                        has_ownership_history += 1;
                    }
                    if alloc.get("lifetime_ms").is_some() && !alloc["lifetime_ms"].is_null() {
                        has_lifetime_ms += 1;
                    }
                }

                println!("   ✅ improve.md extensions found:");
                println!("      • borrow_info: {has_borrow_info} allocations");
                println!("      • clone_info: {has_clone_info} allocations");
                println!(
                    "      • ownership_history_available: {has_ownership_history} allocations"
                );
                println!("      • lifetime_ms: {has_lifetime_ms} allocations");

                // Show example of borrow_info if available
                if let Some(first_alloc) = allocations.first() {
                    if let Some(borrow_info) = first_alloc.get("borrow_info") {
                        if !borrow_info.is_null() {
                            println!("   📋 Example borrow_info: {borrow_info}");
                        }
                    }
                }
            }
        }
    }

    // Check lifetime.json
    let lifetime_path = format!("{output_dir}/lifetime.json");
    if let Ok(content) = std::fs::read_to_string(&lifetime_path) {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(histories) = json_value["ownership_histories"].as_array() {
                println!(
                    "   📈 Found {} ownership histories in lifetime.json",
                    histories.len()
                );

                if let Some(first_history) = histories.first() {
                    if let Some(events) = first_history["ownership_history"].as_array() {
                        println!(
                            "      • First allocation has {} ownership events",
                            events.len()
                        );
                        if let Some(first_event) = events.first() {
                            println!(
                                "      • Example event: {} at timestamp {}",
                                first_event["event_type"].as_str().unwrap_or("unknown"),
                                first_event["timestamp"].as_u64().unwrap_or(0)
                            );
                        }
                    }
                }
            }
        }
    }

    println!(
        "   🎯 These extensions provide detailed borrowing, cloning, and lifecycle information!"
    );
}
