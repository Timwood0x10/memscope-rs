//! Test command implementation
//!
//! This module provides the test subcommand functionality.

use crate::core::tracker::get_global_tracker;
use crate::track_var;
use clap::ArgMatches;
use std::error::Error;

/// Run the test command
pub fn run_test(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let output_path = matches
        .get_one::<String>("output")
        .map(|s| s.as_str())
        .unwrap_or("enhanced_memory_test");

    tracing::info!("🧪 Running enhanced memory tests...");
    tracing::info!("Output path: {}", output_path);

    // Initialize memory tracking
    crate::init();

    // Run the test
    run_enhanced_memory_test()?;

    Ok(())
}

fn run_enhanced_memory_test() -> Result<(), Box<dyn Error>> {
    use crate::core::tracker::get_global_tracker;

    tracing::info!("Creating test allocations...");

    // Create some test allocations
    let vec1 = vec![1, 2, 3, 4, 5];
    let vec2 = vec![6, 7, 8, 9, 10];
    let string1 = String::from("Hello, World!");
    let boxed1 = Box::new(42);

    track_var!(vec1);
    track_var!(vec2);
    track_var!(string1);
    track_var!(boxed1);

    // Create some temporary objects
    for i in 0..10 {
        let temp = vec![i; i];
        track_var!(temp);
    }

    // Get all allocations
    let tracker = get_global_tracker();
    let _allocations = match tracker.get_active_allocations() {
        Ok(allocs) => allocs,
        Err(e) => {
            tracing::info!("Error getting allocations: {}", e);
            Vec::new()
        }
    };

    tracing::info!("Enhanced Memory Analysis Summary:");
    tracing::info!("--------------------------------");
    tracing::info!("Total active allocations: {}", _allocations.len());

    // Keep variables alive until the end
    tracing::info!("Vec1 length: {}", vec1.len());
    tracing::info!("Vec2 length: {}", vec2.len());
    tracing::info!("String1 length: {}", string1.len());
    tracing::info!("Boxed1 value: {}", *boxed1);

    Ok(())
}

fn _test_enhanced_memory_analysis() {
    // Create some test allocations
    let vec1 = vec![1, 2, 3, 4, 5];
    let vec2 = vec![6, 7, 8, 9, 10];
    let string1 = String::from("Hello, World!");
    let boxed1 = Box::new(42);

    track_var!(vec1);
    track_var!(vec2);
    track_var!(string1);
    track_var!(boxed1);

    // Create some temporary objects
    for i in 0..10 {
        let temp = vec![i; i];
        track_var!(temp);
    }

    // Get all allocations
    let tracker = get_global_tracker();
    let _allocations = match tracker.get_active_allocations() {
        Ok(allocs) => allocs,
        Err(e) => {
            tracing::info!("Error getting allocations: {}", e);
            Vec::new()
        }
    };

    // Run enhanced analysis
    let report = crate::analysis::analyze_memory_with_enhanced_features()
        .unwrap_or_else(|e| format!("Error: {}", e));

    // Print summary
    tracing::info!("Enhanced Memory Analysis Summary:");
    tracing::info!("--------------------------------");
    tracing::info!("Report: {}", report);

    // Keep variables alive until the end
    tracing::info!("Vec1 length: {}", vec1.len());
    tracing::info!("Vec2 length: {}", vec2.len());
    tracing::info!("String1 length: {}", string1.len());
    tracing::info!("Boxed1 value: {}", *boxed1);
}
