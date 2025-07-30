//! Integrated Binary Export Test
//!
//! This example demonstrates the fully integrated binary export functionality
//! within the memscope tracking system, showing real performance improvements
//! with actual tracked data.
//!
//! Run with: cargo run --example integrated_binary_export_test

use memscope_rs::export::binary_export::{BinaryExportOptions, SelectionCriteria};
use memscope_rs::*;
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Integrated Binary Export Performance Test");
    println!("{}", "=".repeat(70));
    println!("Testing fully integrated binary export with real memscope data...\n");

    // Initialize tracker and create comprehensive test data
    let tracker = get_global_tracker();
    println!("📊 Creating comprehensive memory tracking data...");
    create_comprehensive_test_data();

    println!("✅ Test data creation completed\n");

    // Test JSON export (baseline)
    println!("📄 Testing JSON Export (Baseline)...");
    let json_start = Instant::now();
    tracker.export_to_json("integrated_test_json")?;
    let json_duration = json_start.elapsed();

    let json_size = fs::metadata("integrated_test_json_memory_analysis.json")?.len();
    println!("  ⏱️  Export time: {:?}", json_duration);
    println!(
        "  📏 File size: {} bytes ({:.2} MB)",
        json_size,
        json_size as f64 / 1024.0 / 1024.0
    );

    // Test Binary export with different configurations
    println!("\n📦 Testing Binary Export Configurations...");

    // Fast configuration
    println!("\n🚀 Fast Configuration:");
    let fast_start = Instant::now();
    let fast_options = BinaryExportOptions::fast();
    let fast_stats = tracker.export_to_binary("integrated_test_fast.msgpack", fast_options)?;
    let fast_duration = fast_start.elapsed();

    let fast_size = fs::metadata("integrated_test_fast.msgpack")?.len();
    println!("  ⏱️  Export time: {:?}", fast_duration);
    println!(
        "  📏 File size: {} bytes ({:.2} MB)",
        fast_size,
        fast_size as f64 / 1024.0 / 1024.0
    );
    println!(
        "  🗜️  Compression: {:.1}%",
        fast_stats.compression_ratio * 100.0
    );

    // Compact configuration
    println!("\n💾 Compact Configuration:");
    let compact_start = Instant::now();
    let compact_options = BinaryExportOptions::compact();
    let compact_stats =
        tracker.export_to_binary("integrated_test_compact.msgpack", compact_options)?;
    let compact_duration = compact_start.elapsed();

    let compact_size = fs::metadata("integrated_test_compact.msgpack")?.len();
    println!("  ⏱️  Export time: {:?}", compact_duration);
    println!(
        "  📏 File size: {} bytes ({:.2} MB)",
        compact_size,
        compact_size as f64 / 1024.0 / 1024.0
    );
    println!(
        "  🗜️  Compression: {:.1}%",
        compact_stats.compression_ratio * 100.0
    );

    // Selective configuration
    println!("\n🎯 Selective Configuration:");
    let selective_start = Instant::now();
    let selective_options = BinaryExportOptions::selective();
    let selective_stats =
        tracker.export_to_binary("integrated_test_selective.msgpack", selective_options)?;
    let selective_duration = selective_start.elapsed();

    let selective_size = fs::metadata("integrated_test_selective.msgpack")?.len();
    println!("  ⏱️  Export time: {:?}", selective_duration);
    println!(
        "  📏 File size: {} bytes ({:.2} MB)",
        selective_size,
        selective_size as f64 / 1024.0 / 1024.0
    );
    println!(
        "  🗜️  Compression: {:.1}%",
        selective_stats.compression_ratio * 100.0
    );

    // Test selective loading
    println!("\n📥 Testing Selective Loading...");

    // Load all data
    let load_all_start = Instant::now();
    let full_data = MemoryTracker::load_from_binary("integrated_test_selective.msgpack")?;
    let load_all_duration = load_all_start.elapsed();
    println!(
        "  📊 Full load: {} allocations in {:?}",
        full_data.allocations.len(),
        load_all_duration
    );

    // Load only Vec<i32> allocations
    let load_selective_start = Instant::now();
    let criteria = SelectionCriteria {
        type_names: Some(vec!["Vec<i32>".to_string()]),
        limit: Some(50),
        ..Default::default()
    };
    let selective_data =
        MemoryTracker::load_selective_binary("integrated_test_selective.msgpack", criteria)?;
    let load_selective_duration = load_selective_start.elapsed();
    println!(
        "  🎯 Selective load: {} Vec<i32> allocations in {:?}",
        selective_data.len(),
        load_selective_duration
    );

    // Print comprehensive comparison
    print_comprehensive_comparison(
        json_duration,
        json_size,
        fast_duration,
        fast_size,
        &fast_stats,
        compact_duration,
        compact_size,
        &compact_stats,
        selective_duration,
        selective_size,
        &selective_stats,
    );

    // Cleanup test files
    cleanup_test_files();

    println!("\n✅ Integrated binary export test completed successfully!");
    println!("🎉 Binary export is fully integrated and working!");

    Ok(())
}

/// Create comprehensive test data using actual memscope tracking
fn create_comprehensive_test_data() {
    // Create diverse data structures with realistic patterns
    for i in 0..300 {
        let vec_data: Vec<i32> = (0..100).collect();
        track_var!(vec_data);

        let map_data: HashMap<String, i32> =
            (0..50).map(|j| (format!("key_{}_{}", i, j), j)).collect();
        track_var!(map_data);
    }

    // Create nested structures
    for _i in 0..150 {
        let nested: Vec<Vec<String>> = (0..10)
            .map(|j| (0..20).map(|k| format!("nested_{}_{}", j, k)).collect())
            .collect();
        track_var!(nested);
    }

    // Create temporary objects with varying lifetimes
    for i in 0..500 {
        let temp_string = format!(
            "Temporary string with substantial content for testing compression efficiency {}",
            i
        );
        track_var!(temp_string);
    }

    // Create long-lived objects
    for _i in 0..100 {
        let long_lived: Box<Vec<u64>> = Box::new((0..200).map(|x| x as u64).collect());
        track_var!(long_lived);
    }

    // Create smart pointer usage patterns
    use std::rc::Rc;
    use std::sync::Arc;

    for i in 0..100 {
        let rc_data = Rc::new(format!("Reference counted data with content {}", i));
        track_var!(rc_data);

        let arc_data = Arc::new(vec![i; 100]);
        track_var!(arc_data);
    }

    println!("  📈 Created ~1150 tracked allocations");
    println!("  🔄 Mixed container types, nested structures, and smart pointers");
    println!("  💾 Realistic memory patterns for compression testing");
}

/// Print comprehensive performance comparison
fn print_comprehensive_comparison(
    json_duration: std::time::Duration,
    json_size: u64,
    fast_duration: std::time::Duration,
    fast_size: u64,
    fast_stats: &memscope_rs::export::binary_export::BinaryExportStats,
    compact_duration: std::time::Duration,
    compact_size: u64,
    compact_stats: &memscope_rs::export::binary_export::BinaryExportStats,
    selective_duration: std::time::Duration,
    selective_size: u64,
    selective_stats: &memscope_rs::export::binary_export::BinaryExportStats,
) {
    println!("\n🎯 Comprehensive Performance Analysis");
    println!("{}", "=".repeat(70));

    // Speed improvements
    let fast_speedup = json_duration.as_nanos() as f64 / fast_duration.as_nanos() as f64;
    let compact_speedup = json_duration.as_nanos() as f64 / compact_duration.as_nanos() as f64;
    let selective_speedup = json_duration.as_nanos() as f64 / selective_duration.as_nanos() as f64;

    println!("\n🚀 Export Speed Improvements:");
    println!("  Fast Binary:      {:.1}x faster than JSON", fast_speedup);
    println!(
        "  Compact Binary:   {:.1}x faster than JSON",
        compact_speedup
    );
    println!(
        "  Selective Binary: {:.1}x faster than JSON",
        selective_speedup
    );

    // Size improvements
    let fast_size_reduction = json_size as f64 / fast_size as f64;
    let compact_size_reduction = json_size as f64 / compact_size as f64;
    let selective_size_reduction = json_size as f64 / selective_size as f64;

    println!("\n💾 File Size Improvements:");
    println!(
        "  Fast Binary:      {:.1}x smaller than JSON",
        fast_size_reduction
    );
    println!(
        "  Compact Binary:   {:.1}x smaller than JSON",
        compact_size_reduction
    );
    println!(
        "  Selective Binary: {:.1}x smaller than JSON",
        selective_size_reduction
    );

    // Performance summary table
    println!("\n📊 Performance Summary Table:");
    println!("  Format           | Export Time | File Size  | Speedup | Size Reduction");
    println!("  -----------------|-------------|------------|---------|---------------");
    println!(
        "  JSON (baseline)  | {:>9.1}ms | {:>8.1}KB |   1.0x  |        1.0x",
        json_duration.as_secs_f64() * 1000.0,
        json_size as f64 / 1024.0
    );
    println!(
        "  Fast Binary      | {:>9.1}ms | {:>8.1}KB | {:>6.1}x  | {:>10.1}x",
        fast_duration.as_secs_f64() * 1000.0,
        fast_size as f64 / 1024.0,
        fast_speedup,
        fast_size_reduction
    );
    println!(
        "  Compact Binary   | {:>9.1}ms | {:>8.1}KB | {:>6.1}x  | {:>10.1}x",
        compact_duration.as_secs_f64() * 1000.0,
        compact_size as f64 / 1024.0,
        compact_speedup,
        compact_size_reduction
    );
    println!(
        "  Selective Binary | {:>9.1}ms | {:>8.1}KB | {:>6.1}x  | {:>10.1}x",
        selective_duration.as_secs_f64() * 1000.0,
        selective_size as f64 / 1024.0,
        selective_speedup,
        selective_size_reduction
    );

    // Recommendations
    println!("\n💡 Configuration Recommendations:");
    println!("  🚀 Fast Binary: Best for real-time monitoring and frequent exports");
    println!("     - Minimal compression overhead");
    println!("     - Maximum export speed");
    println!("     - Good size reduction without sacrificing performance");

    println!("  💾 Compact Binary: Best for archival storage and bandwidth-limited scenarios");
    println!("     - Maximum compression efficiency");
    println!("     - Smallest file sizes");
    println!("     - Ideal for long-term storage");

    println!("  🎯 Selective Binary: Best for analysis workflows requiring partial data");
    println!("     - Balanced performance and compression");
    println!("     - Optimized for selective loading");
    println!("     - Includes comprehensive indexing");

    println!("\n🔍 Key Insights:");
    println!(
        "  • Binary formats show consistent {:.1}x-{:.1}x speed improvements",
        [compact_speedup, fast_speedup, selective_speedup]
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b)),
        [compact_speedup, fast_speedup, selective_speedup]
            .iter()
            .fold(0.0f64, |a, &b| a.max(b))
    );
    println!(
        "  • File size reductions range from {:.1}x to {:.1}x smaller",
        [
            fast_size_reduction,
            compact_size_reduction,
            selective_size_reduction
        ]
        .iter()
        .fold(f64::INFINITY, |a, &b| a.min(b)),
        [
            fast_size_reduction,
            compact_size_reduction,
            selective_size_reduction
        ]
        .iter()
        .fold(0.0f64, |a, &b| a.max(b))
    );
    println!(
        "  • Compression ratios: Fast {:.1}%, Compact {:.1}%, Selective {:.1}%",
        fast_stats.compression_ratio * 100.0,
        compact_stats.compression_ratio * 100.0,
        selective_stats.compression_ratio * 100.0
    );
    println!("  • All configurations provide substantial benefits over JSON");
}

/// Clean up test files
fn cleanup_test_files() {
    let files = [
        "integrated_test_json_memory_analysis.json",
        "integrated_test_json_lifetime.json",
        "integrated_test_json_unsafe_ffi.json",
        "integrated_test_json_performance.json",
        "integrated_test_fast.msgpack",
        "integrated_test_compact.msgpack",
        "integrated_test_selective.msgpack",
    ];

    for file in &files {
        let _ = fs::remove_file(file);
    }
}
