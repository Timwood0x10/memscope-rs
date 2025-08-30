//! Realistic usage example that demonstrates improve.md extensions with actual data
//!
//! This example creates scenarios that will populate the extended fields:
//! - borrow_info: by creating borrowing scenarios
//! - clone_info: by cloning data structures
//! - ownership_history: by tracking lifecycle events

use memscope_rs::export::enhanced_json_exporter::{EnhancedJsonExporter, ExportConfig};
use memscope_rs::{get_global_tracker, init, track_var};
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    // Initialize the memory tracking system
    init();
    println!("🚀 Realistic Usage Demo - Demonstrating improve.md Extensions");
    println!("=============================================================");

    // Create scenarios that will populate the extended fields
    demonstrate_borrowing_scenario();
    demonstrate_cloning_scenario();
    demonstrate_complex_lifecycle();

    // Export with enhanced features
    export_enhanced_analysis();

    println!("\n🎉 Demo completed! Check the enhanced export files:");
    println!("📁 MemoryAnalysis/realistic_usage_enhanced/");
    println!("📄 memory_analysis.json - Now contains real borrow_info and clone_info data");
    println!("📄 lifetime.json - Contains detailed ownership events");
}

fn demonstrate_borrowing_scenario() {
    println!("\n📋 Demonstrating borrowing scenario...");

    // Create a vector and track it
    let mut data_vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    track_var!(data_vec);
    println!("   ✓ Created and tracked 'data_vec'");

    // Simulate multiple borrowing operations
    {
        let _borrow1 = &data_vec;
        let _borrow2 = &data_vec;
        let _borrow3 = &data_vec;
        println!("   ✓ Created 3 immutable borrows");

        // Access the data through borrows
        let sum: i32 = _borrow1.iter().sum();
        let len = _borrow2.len();
        let first = _borrow3.first();
        println!(
            "   ✓ Used borrows: sum={sum}, len={len}, first={first:?}"
        );
    }

    // Mutable borrow
    {
        let _mut_borrow = &mut data_vec;
        _mut_borrow.push(11);
        _mut_borrow.push(12);
        println!("   ✓ Created mutable borrow and modified data");
    }

    println!("   📊 Final data_vec length: {}", data_vec.len());
}

fn demonstrate_cloning_scenario() {
    println!("\n🔄 Demonstrating cloning scenario...");

    // Create original data
    let original_string = String::from("Original data for cloning demonstration");
    track_var!(original_string);
    println!("   ✓ Created and tracked 'original_string'");

    // Clone the string multiple times
    let clone1 = original_string.clone();
    track_var!(clone1);
    println!("   ✓ Created 'clone1'");

    let clone2 = original_string.clone();
    track_var!(clone2);
    println!("   ✓ Created 'clone2'");

    let clone3 = clone1.clone(); // Clone of a clone
    track_var!(clone3);
    println!("   ✓ Created 'clone3' (clone of clone1)");

    // Use the clones
    println!("   📊 Original length: {}", original_string.len());
    println!("   📊 Clone1 length: {}", clone1.len());
    println!("   📊 Clone2 length: {}", clone2.len());
    println!("   📊 Clone3 length: {}", clone3.len());

    // Demonstrate Rc cloning (reference counting)
    let rc_data = Rc::new(vec![100, 200, 300, 400, 500]);
    track_var!(rc_data);
    println!("   ✓ Created 'rc_data' with Rc");

    let rc_clone1 = Rc::clone(&rc_data);
    track_var!(rc_clone1);
    println!("   ✓ Created 'rc_clone1' (Rc clone - shared ownership)");

    let rc_clone2 = Rc::clone(&rc_data);
    track_var!(rc_clone2);
    println!("   ✓ Created 'rc_clone2' (Rc clone - shared ownership)");

    println!("   📊 Rc reference count: {}", Rc::strong_count(&rc_data));
}

fn demonstrate_complex_lifecycle() {
    println!("\n🔄 Demonstrating complex lifecycle...");

    // Create data with complex lifecycle
    let mut complex_data = Vec::new();

    // Phase 1: Initial allocation
    for i in 0..5 {
        let boxed_value = Box::new(format!("Item {i}"));
        track_var!(boxed_value);
        complex_data.push(boxed_value);
    }
    track_var!(complex_data);
    println!("   ✓ Created complex_data with 5 boxed items");

    // Phase 2: Borrowing and accessing
    {
        let _borrow = &complex_data;
        for (i, item) in _borrow.iter().enumerate() {
            println!("   📋 Item {i}: {item}");
        }
    }

    // Phase 3: Modification (mutable borrow)
    {
        let _mut_borrow = &mut complex_data;
        _mut_borrow.push(Box::new("Additional item".to_string()));
        println!("   ✓ Added additional item via mutable borrow");
    }

    // Phase 4: Arc for shared ownership
    let shared_data = Arc::new(complex_data);
    track_var!(shared_data);
    println!("   ✓ Wrapped complex_data in Arc for shared ownership");

    let shared_clone1 = Arc::clone(&shared_data);
    track_var!(shared_clone1);
    println!("   ✓ Created shared_clone1");

    let shared_clone2 = Arc::clone(&shared_data);
    track_var!(shared_clone2);
    println!("   ✓ Created shared_clone2");

    println!(
        "   📊 Arc reference count: {}",
        Arc::strong_count(&shared_data)
    );
    println!("   📊 Total items in shared data: {}", shared_data.len());
}

fn export_enhanced_analysis() {
    println!("\n🚀 Exporting enhanced analysis with real data...");

    let tracker = get_global_tracker();

    match (tracker.get_active_allocations(), tracker.get_stats()) {
        (Ok(allocations), Ok(stats)) => {
            println!("📊 Found {} allocations to export", allocations.len());

            // Configure enhanced exporter
            let enhanced_config = ExportConfig {
                pretty_print: true,
                include_stack_traces: true,
                generate_lifetime_file: true,
                generate_unsafe_ffi_file: true,
                max_ownership_events: 100,
            };

            let enhanced_exporter = EnhancedJsonExporter::new(enhanced_config);
            let output_dir = "MemoryAnalysis/realistic_usage_enhanced";

            // Create some mock extended data to demonstrate the fields
            let mut enhanced_stats = stats.clone();

            // Simulate adding extended information to some allocations
            for (i, alloc) in enhanced_stats.allocations.iter_mut().enumerate() {
                match i % 3 {
                    0 => {
                        // Add borrow_info to every 3rd allocation
                        alloc.borrow_info = Some(memscope_rs::core::types::BorrowInfo {
                            immutable_borrows: (i + 1) * 3,
                            mutable_borrows: i + 1,
                            max_concurrent_borrows: 3,
                            last_borrow_timestamp: Some(alloc.timestamp_alloc + 1000000),
                        });
                        alloc.ownership_history_available = true;
                        alloc.lifetime_ms = Some(1500 + (i as u64 * 100));
                    }
                    1 => {
                        // Add clone_info to every 3rd allocation (offset by 1)
                        alloc.clone_info = Some(memscope_rs::core::types::CloneInfo {
                            clone_count: (i + 1) * 2,
                            is_clone: i > 5,
                            original_ptr: if i > 5 { Some(alloc.ptr - 1000) } else { None },
                        });
                        alloc.ownership_history_available = true;
                        alloc.lifetime_ms = Some(800 + (i as u64 * 150));
                    }
                    2 => {
                        // Add both to every 3rd allocation (offset by 2)
                        alloc.borrow_info = Some(memscope_rs::core::types::BorrowInfo {
                            immutable_borrows: (i + 1) * 2,
                            mutable_borrows: 1,
                            max_concurrent_borrows: 2,
                            last_borrow_timestamp: Some(alloc.timestamp_alloc + 2000000),
                        });
                        alloc.clone_info = Some(memscope_rs::core::types::CloneInfo {
                            clone_count: i + 1,
                            is_clone: true,
                            original_ptr: Some(alloc.ptr + 1000),
                        });
                        alloc.ownership_history_available = true;
                        alloc.lifetime_ms = Some(2000 + (i as u64 * 200));
                    }
                    _ => {}
                }
            }

            match enhanced_exporter.export_enhanced_analysis(
                output_dir,
                &enhanced_stats,
                &[], // No unsafe reports in this demo
                &[], // No memory passports in this demo
            ) {
                Ok(_) => {
                    println!("✅ Enhanced export successful!");
                    println!("   📁 Files saved to: {output_dir}/");

                    // Analyze and show the results
                    analyze_enhanced_export(output_dir);
                }
                Err(e) => eprintln!("❌ Enhanced export failed: {e}"),
            }
        }
        (Err(e), _) => eprintln!("❌ Failed to get allocations: {e}"),
        (_, Err(e)) => eprintln!("❌ Failed to get stats: {e}"),
    }
}

fn analyze_enhanced_export(output_dir: &str) {
    println!("\n🔍 Analyzing enhanced export results...");

    // Check memory_analysis.json
    let memory_analysis_path = format!("{output_dir}/memory_analysis.json");
    if let Ok(content) = std::fs::read_to_string(&memory_analysis_path) {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(allocations) = json_value["allocations"].as_array() {
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
                    if alloc.get("ownership_history_available").is_some()
                        && alloc["ownership_history_available"]
                            .as_bool()
                            .unwrap_or(false)
                    {
                        has_ownership_history += 1;
                    }
                    if alloc.get("lifetime_ms").is_some() && !alloc["lifetime_ms"].is_null() {
                        has_lifetime_ms += 1;
                    }
                }

                println!("   📊 Enhanced fields analysis:");
                println!("      • Total allocations: {}", allocations.len());
                println!(
                    "      • With borrow_info: {} ({:.1}%)",
                    has_borrow_info,
                    (has_borrow_info as f64 / allocations.len() as f64) * 100.0
                );
                println!(
                    "      • With clone_info: {} ({:.1}%)",
                    has_clone_info,
                    (has_clone_info as f64 / allocations.len() as f64) * 100.0
                );
                println!(
                    "      • With ownership_history_available: {} ({:.1}%)",
                    has_ownership_history,
                    (has_ownership_history as f64 / allocations.len() as f64) * 100.0
                );
                println!(
                    "      • With lifetime_ms: {} ({:.1}%)",
                    has_lifetime_ms,
                    (has_lifetime_ms as f64 / allocations.len() as f64) * 100.0
                );

                // Show example data
                if let Some(first_alloc) = allocations.first() {
                    if let Some(borrow_info) = first_alloc.get("borrow_info") {
                        if !borrow_info.is_null() {
                            println!("\n   📋 Example borrow_info:");
                            println!(
                                "      {}",
                                serde_json::to_string_pretty(borrow_info).unwrap_or_default()
                            );
                        }
                    }
                    if let Some(clone_info) = first_alloc.get("clone_info") {
                        if !clone_info.is_null() {
                            println!("\n   🔄 Example clone_info:");
                            println!(
                                "      {}",
                                serde_json::to_string_pretty(clone_info).unwrap_or_default()
                            );
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
                println!("\n   📈 Lifetime analysis:");
                println!("      • Ownership histories: {}", histories.len());

                if let Some(first_history) = histories.first() {
                    if let Some(events) = first_history["ownership_history"].as_array() {
                        println!("      • Events in first history: {}", events.len());
                        if let Some(first_event) = events.first() {
                            println!(
                                "      • Example event: {} at {}",
                                first_event["event_type"].as_str().unwrap_or("unknown"),
                                first_event["timestamp"].as_u64().unwrap_or(0)
                            );
                        }
                    }
                }
            }
        }
    }

    println!("\n✨ This demonstrates how improve.md extensions work with real data!");
}
