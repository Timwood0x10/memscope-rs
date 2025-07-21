//! Test command implementation
//!
//! This module provides the test subcommand functionality.

use clap::ArgMatches;
use std::error::Error;

/// Run the test command
pub fn run_test(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let output_path = matches
        .get_one::<String>("output")
        .map(|s| s.as_str())
        .unwrap_or("enhanced_memory_test");

    println!("🧪 Running enhanced memory tests...");
    println!("Output path: {}", output_path);

    // Initialize memory tracking
    crate::init();

    // Run the test
    run_enhanced_memory_test()?;

    Ok(())
}

fn run_enhanced_memory_test() -> Result<(), Box<dyn Error>> {
    use crate::core::tracker::get_global_tracker;

    println!("Creating test allocations...");

    // Create some test allocations
    let vec1 = track_var!(vec![1, 2, 3, 4, 5]);
    let vec2 = track_var!(vec![6, 7, 8, 9, 10]);
    let string1 = track_var!(String::from("Hello, World!"));
    let boxed1 = track_var!(Box::new(42));

    // Create some temporary objects
    for i in 0..10 {
        let _temp = track_var!(vec![i; i]);
    }

    // Get all allocations
    let tracker = get_global_tracker();
    let allocations = match tracker.get_active_allocations() {
        Ok(allocs) => allocs,
        Err(e) => {
            println!("Error getting allocations: {}", e);
            Vec::new()
        }
    };

    println!("Enhanced Memory Analysis Summary:");
    println!("--------------------------------");
    println!("Total active allocations: {}", allocations.len());

    // Keep variables alive until the end
    println!("Vec1 length: {}", vec1.len());
    println!("Vec2 length: {}", vec2.len());
    println!("String1 length: {}", string1.len());
    println!("Boxed1 value: {}", *boxed1);

    Ok(())
}

fn _original_main() {
    // Create some test allocations
    let vec1 = track_var!(vec![1, 2, 3, 4, 5]);
    let vec2 = track_var!(vec![6, 7, 8, 9, 10]);
    let string1 = track_var!(String::from("Hello, World!"));
    let boxed1 = track_var!(Box::new(42));

    // Create some temporary objects
    for i in 0..10 {
        let _temp = track_var!(vec![i; i]);
    }

    // Get all allocations
    let tracker = memscope_rs::get_global_tracker();
    let allocations = match tracker.get_active_allocations() {
        Ok(allocs) => allocs,
        Err(e) => {
            println!("Error getting allocations: {}", e);
            Vec::new()
        }
    };

    // Run enhanced analysis
    let report = analyze_memory_with_enhanced_features(&allocations);

    // Print summary
    println!("Enhanced Memory Analysis Summary:");
    println!("--------------------------------");
    println!("Analysis duration: {} ms", report.analysis_duration_ms);
    println!(
        "Stack allocations: {}",
        report.stack_heap_analysis.stack_allocations.len()
    );
    println!(
        "Heap allocations: {}",
        report.stack_heap_analysis.heap_allocations.len()
    );
    println!(
        "Temporary objects: {}",
        report.temp_object_analysis.temporary_objects.len()
    );
    println!(
        "Fragmentation level: {:.2}%",
        report
            .fragmentation_analysis
            .current_fragmentation
            .total_fragmentation_ratio
            * 100.0
    );
    println!(
        "Generic instantiations: {}",
        report.generic_analysis.instantiation_analysis.len()
    );
    println!(
        "Lifecycle reports: {}",
        report.lifecycle_analysis.lifecycle_reports.len()
    );
    println!(
        "Overall recommendations: {}",
        report.overall_recommendations.len()
    );

    // Keep variables alive until the end
    println!("Vec1 length: {}", vec1.len());
    println!("Vec2 length: {}", vec2.len());
    println!("String1 length: {}", string1.len());
    println!("Boxed1 value: {}", *boxed1);
}
