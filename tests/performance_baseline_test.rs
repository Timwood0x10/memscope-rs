//! Performance baseline testing framework
//!
//! This module provides comprehensive performance testing to ensure no regression
//! and compare different configuration levels.

use memscope_rs::core::tracker::MemoryTracker;
use memscope_rs::export::binary::BinaryExportConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;

/// Performance baseline metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerformanceBaseline {
    /// Allocation operations per second
    allocation_throughput: f64,
    /// Deallocation operations per second
    deallocation_throughput: f64,
    /// Binary export time in milliseconds
    export_binary_time_ms: f64,
    /// JSON export time in milliseconds
    export_json_time_ms: f64,
    /// Statistics generation time in milliseconds
    stats_generation_time_ms: f64,
    /// Timestamp when baseline was measured
    timestamp: u64,
}

/// Configuration-specific performance metrics
#[derive(Debug, Clone)]
struct ConfigPerformance {
    config_name: String,
    export_time: f64,
    throughput: f64,
    file_size: usize,
}

const PERFORMANCE_REGRESSION_THRESHOLD: f64 = 0.10; // 10%
const BASELINE_FILE_PATH: &str = "tests/performance_baseline.json";
const NUM_TEST_ALLOCATIONS: usize = 1_000;

/// Test performance baseline and detect regressions
#[test]
fn test_performance_baseline() {
    println!("🚀 Starting performance baseline test...");

    let baseline = measure_current_performance();

    // Compare with historical baseline if exists
    if let Some(historical) = load_baseline_from_file() {
        println!("📊 Comparing with historical baseline...");
        assert_no_performance_regression(&baseline, &historical);
    } else {
        println!("📝 No historical baseline found, creating new baseline");
    }

    // Save current baseline
    save_baseline_to_file(&baseline);

    print_baseline_results(&baseline);

    println!("✅ Performance baseline test completed successfully");
}

/// Test performance comparison across different configurations
#[test]
fn test_configuration_performance_comparison() {
    println!("📈 Starting configuration performance comparison...");

    let configs = [
        ("minimal", BinaryExportConfig::minimal()),
        ("performance_first", BinaryExportConfig::performance_first()),
        (
            "debug_comprehensive",
            BinaryExportConfig::debug_comprehensive(),
        ),
    ];

    let mut results = HashMap::new();

    for (name, config) in configs {
        println!("   Testing configuration: {}", name);
        let perf = measure_config_performance(name, &config);
        results.insert(name, perf);
    }

    // Verify performance expectations
    assert_config_performance_expectations(&results);

    // Generate comparison report
    print_performance_comparison(&results);

    println!("✅ Configuration performance comparison completed");
}

/// Measure current system performance
fn measure_current_performance() -> PerformanceBaseline {
    let tracker = MemoryTracker::new();

    println!("   Measuring allocation throughput...");
    let allocation_throughput = measure_allocation_throughput(&tracker);

    println!("   Measuring deallocation throughput...");
    let deallocation_throughput = measure_deallocation_throughput(&tracker);

    println!("   Measuring export performance...");
    let (export_binary_time, export_json_time) = measure_export_performance(&tracker);

    println!("   Measuring stats generation performance...");
    let stats_time = measure_stats_generation_performance(&tracker);

    PerformanceBaseline {
        allocation_throughput,
        deallocation_throughput,
        export_binary_time_ms: export_binary_time.as_millis() as f64,
        export_json_time_ms: export_json_time.as_millis() as f64,
        stats_generation_time_ms: stats_time.as_millis() as f64,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}

/// Measure allocation throughput (operations per second)
fn measure_allocation_throughput(tracker: &MemoryTracker) -> f64 {
    let num_operations = NUM_TEST_ALLOCATIONS;
    let start = Instant::now();

    for i in 0..num_operations {
        let ptr = (i + 1) * 0x1000;
        tracker.track_allocation(ptr, 1024).unwrap();

        // Associate variable for more realistic test
        if i % 10 == 0 {
            tracker
                .associate_var(ptr, format!("test_var_{}", i), "TestType".to_string())
                .unwrap();
        }
    }

    let duration = start.elapsed();
    num_operations as f64 / duration.as_secs_f64()
}

/// Measure deallocation throughput (operations per second)
fn measure_deallocation_throughput(tracker: &MemoryTracker) -> f64 {
    // First create allocations
    let num_operations = NUM_TEST_ALLOCATIONS;
    let mut ptrs = Vec::new();

    for i in 0..num_operations {
        let ptr = (i + 1) * 0x2000; // Different range to avoid conflicts
        tracker.track_allocation(ptr, 1024).unwrap();
        ptrs.push(ptr);
    }

    // Now measure deallocation performance
    let start = Instant::now();

    for ptr in ptrs {
        tracker.track_deallocation(ptr).unwrap();
    }

    let duration = start.elapsed();
    num_operations as f64 / duration.as_secs_f64()
}

/// Measure export performance for both binary and JSON formats
fn measure_export_performance(tracker: &MemoryTracker) -> (Duration, Duration) {
    // Create smaller test data for faster testing
    create_test_allocations(tracker, 100);

    let stats = tracker.get_stats().unwrap();
    let allocations = &stats.allocations;

    // Measure binary export
    let binary_temp = NamedTempFile::new().unwrap();
    let binary_start = Instant::now();

    let config = BinaryExportConfig::performance_first();
    memscope_rs::export::binary::export_to_binary_with_config(
        allocations,
        binary_temp.path(),
        &config,
    )
    .unwrap();

    let binary_duration = binary_start.elapsed();

    // Measure JSON export (simplified)
    let json_start = Instant::now();
    let _json_data = serde_json::to_string(allocations).unwrap(); // Use compact format
    let json_duration = json_start.elapsed();

    (binary_duration, json_duration)
}

/// Measure statistics generation performance
fn measure_stats_generation_performance(tracker: &MemoryTracker) -> Duration {
    // Create smaller test data
    create_test_allocations(tracker, 100);

    let start = Instant::now();
    let _stats = tracker.get_stats().unwrap();
    start.elapsed()
}

/// Create test allocations for performance testing
fn create_test_allocations(tracker: &MemoryTracker, count: usize) {
    for i in 0..count {
        let ptr = (i + 1) * 0x3000; // Different range
        tracker.track_allocation(ptr, 1024).unwrap(); // Fixed size for simplicity

        // Reduce variable association frequency
        if i % 10 == 0 {
            tracker
                .associate_var(
                    ptr,
                    format!("perf_test_var_{i}"),
                    "PerfTestType".to_string(), // Fixed type for simplicity
                )
                .unwrap();
        }

        // Simplified deallocation pattern
        if i > 20 && i % 5 == 0 {
            let dealloc_ptr = ((i - 20) + 1) * 0x3000;
            tracker.track_deallocation(dealloc_ptr).unwrap();
        }
    }
}

/// Measure performance for a specific configuration
fn measure_config_performance(name: &str, config: &BinaryExportConfig) -> ConfigPerformance {
    let tracker = MemoryTracker::new();
    create_test_allocations(&tracker, 100); // Reduced test data

    let stats = tracker.get_stats().unwrap();
    let allocations = &stats.allocations;

    let temp_file = NamedTempFile::new().unwrap();
    let start = Instant::now();

    memscope_rs::export::binary::export_to_binary_with_config(
        allocations,
        temp_file.path(),
        config,
    )
    .unwrap();

    let export_time = start.elapsed().as_millis() as f64;
    let file_size = fs::metadata(temp_file.path()).unwrap().len() as usize;
    let throughput = allocations.len() as f64 / (export_time / 1000.0).max(0.001); // Avoid division by zero

    ConfigPerformance {
        config_name: name.to_string(),
        export_time,
        throughput,
        file_size,
    }
}

/// Assert no performance regression compared to baseline
fn assert_no_performance_regression(current: &PerformanceBaseline, baseline: &PerformanceBaseline) {
    // Check allocation performance
    let allocation_change = (baseline.allocation_throughput - current.allocation_throughput)
        / baseline.allocation_throughput;

    assert!(allocation_change < PERFORMANCE_REGRESSION_THRESHOLD,
           "❌ ALLOCATION PERFORMANCE REGRESSION: {:.2}% slower than baseline ({:.0} vs {:.0} ops/sec)", 
           allocation_change * 100.0, current.allocation_throughput, baseline.allocation_throughput);

    // Check deallocation performance
    let deallocation_change = (baseline.deallocation_throughput - current.deallocation_throughput)
        / baseline.deallocation_throughput;

    assert!(deallocation_change < PERFORMANCE_REGRESSION_THRESHOLD,
           "❌ DEALLOCATION PERFORMANCE REGRESSION: {:.2}% slower than baseline ({:.0} vs {:.0} ops/sec)", 
           deallocation_change * 100.0, current.deallocation_throughput, baseline.deallocation_throughput);

    // Check binary export performance
    let export_change = (current.export_binary_time_ms - baseline.export_binary_time_ms)
        / baseline.export_binary_time_ms;

    assert!(
        export_change < PERFORMANCE_REGRESSION_THRESHOLD,
        "❌ BINARY EXPORT PERFORMANCE REGRESSION: {:.2}% slower than baseline ({:.2}ms vs {:.2}ms)",
        export_change * 100.0,
        current.export_binary_time_ms,
        baseline.export_binary_time_ms
    );

    // Check stats generation performance
    let stats_change = (current.stats_generation_time_ms - baseline.stats_generation_time_ms)
        / baseline.stats_generation_time_ms;

    assert!(stats_change < PERFORMANCE_REGRESSION_THRESHOLD,
           "❌ STATS GENERATION PERFORMANCE REGRESSION: {:.2}% slower than baseline ({:.2}ms vs {:.2}ms)", 
           stats_change * 100.0, current.stats_generation_time_ms, baseline.stats_generation_time_ms);

    println!("✅ No performance regression detected (all metrics within 10% threshold)");
}

/// Assert configuration performance meets expectations
fn assert_config_performance_expectations(results: &HashMap<&str, ConfigPerformance>) {
    let minimal = &results["minimal"];
    let performance = &results["performance_first"];
    let comprehensive = &results["debug_comprehensive"];

    // Minimal should be fastest or close to performance_first
    assert!(
        minimal.export_time <= performance.export_time * 1.1,
        "Minimal config should be fastest: {}ms vs {}ms",
        minimal.export_time,
        performance.export_time
    );

    // Comprehensive should be slowest but not excessively slow
    assert!(
        comprehensive.export_time <= minimal.export_time * 10.0,
        "Comprehensive config too slow: {}ms vs {}ms (>10x slower)",
        comprehensive.export_time,
        minimal.export_time
    );

    // Performance_first should be reasonable middle ground
    assert!(
        performance.export_time <= minimal.export_time * 5.0,
        "Performance_first config too slow: {}ms vs {}ms",
        performance.export_time,
        minimal.export_time
    );

    println!("✅ All configuration performance expectations met");
}

/// Load baseline from file
fn load_baseline_from_file() -> Option<PerformanceBaseline> {
    if Path::new(BASELINE_FILE_PATH).exists() {
        let content = fs::read_to_string(BASELINE_FILE_PATH).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

/// Save baseline to file
fn save_baseline_to_file(baseline: &PerformanceBaseline) {
    if let Some(parent) = Path::new(BASELINE_FILE_PATH).parent() {
        fs::create_dir_all(parent).ok();
    }

    let json = serde_json::to_string_pretty(baseline).unwrap();
    fs::write(BASELINE_FILE_PATH, json).unwrap();
}

/// Print baseline results
fn print_baseline_results(baseline: &PerformanceBaseline) {
    println!("\n📊 Performance Baseline Results:");
    println!(
        "   Allocation throughput: {:.0} ops/sec",
        baseline.allocation_throughput
    );
    println!(
        "   Deallocation throughput: {:.0} ops/sec",
        baseline.deallocation_throughput
    );
    println!(
        "   Binary export time: {:.2}ms",
        baseline.export_binary_time_ms
    );
    println!("   JSON export time: {:.2}ms", baseline.export_json_time_ms);
    println!(
        "   Stats generation time: {:.2}ms",
        baseline.stats_generation_time_ms
    );
    println!(
        "   Binary vs JSON speedup: {:.1}x",
        baseline.export_json_time_ms / baseline.export_binary_time_ms
    );
}

/// Print performance comparison results
fn print_performance_comparison(results: &HashMap<&str, ConfigPerformance>) {
    println!("\n📈 Configuration Performance Comparison:");

    let mut sorted_results: Vec<_> = results.iter().collect();
    sorted_results.sort_by(|a, b| a.1.export_time.partial_cmp(&b.1.export_time).unwrap());

    for (name, perf) in sorted_results {
        println!(
            "   {:<18} Export: {:.2}ms, Throughput: {:.0} items/sec, Size: {} bytes",
            format!("{}:", name),
            perf.export_time,
            perf.throughput,
            perf.file_size
        );
    }

    // Calculate relative performance
    let fastest = results
        .values()
        .min_by(|a, b| a.export_time.partial_cmp(&b.export_time).unwrap())
        .unwrap();
    println!("\n📊 Relative Performance (vs fastest):");

    for (name, perf) in results {
        let relative = perf.export_time / fastest.export_time;
        println!("   {:<18} {:.1}x slower", format!("{}:", name), relative);
    }
}
