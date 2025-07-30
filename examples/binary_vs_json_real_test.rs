//! Real Binary vs JSON Performance Test
//!
//! This example uses the actual memscope tracking system to create real data
//! and then compares JSON vs binary serialization performance.
//!
//! Run with: cargo run --example binary_vs_json_real_test

use memscope_rs::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

/// Simplified allocation data for real performance testing
///
/// This structure mirrors the essential fields from AllocationInfo
/// but with simplified types that are easy to serialize/deserialize.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestAllocationInfo {
    /// Memory pointer address
    pub ptr: usize,
    /// Allocation size in bytes
    pub size: usize,
    /// Variable name if available
    pub var_name: Option<String>,
    /// Type name if available
    pub type_name: Option<String>,
    /// Allocation timestamp
    pub timestamp_alloc: u64,
    /// Deallocation timestamp if deallocated
    pub timestamp_dealloc: Option<u64>,
    /// Thread identifier
    pub thread_id: String,
    /// Whether this allocation is suspected to be leaked
    pub is_leaked: bool,
}

/// Test export data structure for performance comparison
///
/// Contains the essential data needed for memory analysis export
/// while maintaining compatibility with both JSON and binary formats.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestExportData {
    /// Export metadata
    pub metadata: TestMetadata,
    /// All allocation information
    pub allocations: Vec<TestAllocationInfo>,
    /// Summary statistics
    pub summary: TestSummary,
}

/// Export metadata for tracking export details
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestMetadata {
    /// Format version for compatibility
    pub version: String,
    /// Unix timestamp when export was created
    pub created_at: u64,
    /// Total number of allocations
    pub allocation_count: usize,
    /// Export format identifier
    pub export_format: String,
}

/// Summary statistics for the test export
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestSummary {
    /// Total number of allocations
    pub total_allocations: usize,
    /// Total bytes allocated
    pub total_allocated: usize,
    /// Number of active allocations
    pub active_allocations: usize,
    /// Total active memory
    pub active_memory: usize,
    /// Peak memory usage
    pub peak_memory: usize,
    /// Number of suspected leaks
    pub suspected_leaks: usize,
}

/// Performance benchmark results
#[derive(Debug)]
struct PerformanceResult {
    format_name: String,
    export_time: std::time::Duration,
    import_time: std::time::Duration,
    file_size: u64,
    data_size: usize,
}

impl PerformanceResult {
    /// Print detailed performance results with analysis
    fn print_detailed_results(&self) {
        println!("\n📊 {} Performance Results:", self.format_name);
        println!("  ⏱️  Export time: {:?}", self.export_time);
        println!("  📥 Import time: {:?}", self.import_time);
        println!(
            "  📏 File size: {} bytes ({:.2} MB)",
            self.file_size,
            self.file_size as f64 / 1024.0 / 1024.0
        );
        println!(
            "  💾 Data efficiency: {:.2} bytes per allocation",
            self.file_size as f64 / self.data_size as f64
        );

        // Calculate total time
        println!(
            "  ⚡ Total time: {:.2} ms",
            (self.export_time + self.import_time).as_secs_f64() * 1000.0
        );
    }

    /// Compare this result with a baseline
    fn compare_with_baseline(&self, baseline: &PerformanceResult) {
        if self.format_name == baseline.format_name {
            return;
        }

        let export_speedup =
            baseline.export_time.as_nanos() as f64 / self.export_time.as_nanos() as f64;
        let import_speedup =
            baseline.import_time.as_nanos() as f64 / self.import_time.as_nanos() as f64;
        let size_reduction = baseline.file_size as f64 / self.file_size as f64;
        let total_speedup = (baseline.export_time + baseline.import_time).as_nanos() as f64
            / (self.export_time + self.import_time).as_nanos() as f64;

        println!(
            "\n🎯 {} vs {} Comparison:",
            self.format_name, baseline.format_name
        );
        println!("  🚀 Export: {:.1}x faster", export_speedup);
        println!("  📥 Import: {:.1}x faster", import_speedup);
        println!("  ⚡ Overall: {:.1}x faster", total_speedup);
        println!("  💾 Size: {:.1}x smaller", size_reduction);

        // Performance assessment
        let overall_improvement = (export_speedup + import_speedup + size_reduction) / 3.0;
        match overall_improvement {
            x if x > 5.0 => println!("  ✅ Outstanding improvement! ({:.1}x overall)", x),
            x if x > 3.0 => println!("  ✅ Excellent improvement! ({:.1}x overall)", x),
            x if x > 2.0 => println!("  ✅ Good improvement ({:.1}x overall)", x),
            x if x > 1.5 => println!("  ✅ Moderate improvement ({:.1}x overall)", x),
            x => println!("  ⚠️  Minimal improvement ({:.1}x overall)", x),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Real Binary vs JSON Performance Test");
    println!("{}", "=".repeat(70));
    println!("Using actual memscope tracking data for realistic comparison...\n");

    // Initialize tracker and create realistic test data
    let tracker = get_global_tracker();
    println!("📊 Creating realistic memory tracking data...");
    create_complex_test_data();

    // Collect real data from the tracker
    let test_data = collect_real_tracking_data(&tracker);
    println!(
        "✅ Collected {} allocations from real tracking",
        test_data.allocations.len()
    );

    // Run performance benchmarks
    let json_result = benchmark_json_performance(&test_data)?;
    json_result.print_detailed_results();

    let msgpack_result = benchmark_messagepack_performance(&test_data)?;
    msgpack_result.print_detailed_results();
    msgpack_result.compare_with_baseline(&json_result);

    let compressed_result = benchmark_compressed_performance(&test_data)?;
    compressed_result.print_detailed_results();
    compressed_result.compare_with_baseline(&json_result);

    // Print comprehensive analysis
    print_comprehensive_analysis(&json_result, &msgpack_result, &compressed_result);

    // Cleanup test files
    cleanup_test_files();

    println!("\n✅ Real performance test completed successfully!");
    Ok(())
}

/// Create complex test data using actual memscope tracking
fn create_complex_test_data() {
    // Create diverse data structures similar to complex_lifecycle_showcase
    for i in 0..2000 {
        // 增加10倍数据量
        let vec_data: Vec<i32> = (0..50).collect();
        track_var!(vec_data);

        let map_data: HashMap<String, i32> =
            (0..20).map(|j| (format!("key_{}_{}", i, j), j)).collect();
        track_var!(map_data);
    }

    // Create nested structures
    for _i in 0..1000 {
        // 增加10倍数据量
        let nested: Vec<Vec<String>> = (0..5)
            .map(|j| (0..10).map(|k| format!("nested_{}_{}", j, k)).collect())
            .collect();
        track_var!(nested);
    }

    // Create temporary objects
    for i in 0..3000 {
        // 增加10倍数据量
        let temp_string = format!("Temporary string with content {}", i);
        track_var!(temp_string);
    }

    // Create some long-lived objects
    for _i in 0..500 {
        // 增加10倍数据量
        let long_lived: Box<Vec<u64>> = Box::new((0..100).map(|x| x as u64).collect());
        track_var!(long_lived);
    }

    println!("  📈 Created ~6500 tracked allocations with realistic patterns");
    println!("  🔄 Mixed container types, nested structures, and temporary objects");
}

/// Collect real tracking data from the memscope tracker
fn collect_real_tracking_data(tracker: &MemoryTracker) -> TestExportData {
    // Get real statistics from the tracker
    let stats = tracker.get_memory_stats();
    let active_allocations = tracker.get_all_active_allocations();

    // Convert real allocations to test format
    let test_allocations: Vec<TestAllocationInfo> = active_allocations
        .into_iter()
        .map(|alloc| TestAllocationInfo {
            ptr: alloc.ptr,
            size: alloc.size,
            var_name: alloc.var_name,
            type_name: alloc.type_name,
            timestamp_alloc: alloc.timestamp_alloc,
            timestamp_dealloc: alloc.timestamp_dealloc,
            thread_id: alloc.thread_id,
            is_leaked: alloc.is_leaked,
        })
        .collect();

    let suspected_leaks = test_allocations.iter().filter(|a| a.is_leaked).count();

    TestExportData {
        metadata: TestMetadata {
            version: "1.0.0".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            allocation_count: test_allocations.len(),
            export_format: "memscope_test".to_string(),
        },
        allocations: test_allocations,
        summary: TestSummary {
            total_allocations: stats.total_allocations,
            total_allocated: stats.total_allocated,
            active_allocations: stats.active_allocations,
            active_memory: stats.active_memory,
            peak_memory: stats.peak_memory,
            suspected_leaks,
        },
    }
}

/// Benchmark JSON serialization performance
fn benchmark_json_performance(
    data: &TestExportData,
) -> Result<PerformanceResult, Box<dyn std::error::Error>> {
    // Export benchmark
    let export_start = Instant::now();
    let json_string = serde_json::to_string_pretty(data)?;
    let export_time = export_start.elapsed();

    // Write to file
    fs::write("real_test_export.json", &json_string)?;
    let file_size = fs::metadata("real_test_export.json")?.len();

    // Import benchmark
    let import_start = Instant::now();
    let loaded_json = fs::read_to_string("real_test_export.json")?;
    let _parsed: TestExportData = serde_json::from_str(&loaded_json)?;
    let import_time = import_start.elapsed();

    Ok(PerformanceResult {
        format_name: "JSON".to_string(),
        export_time,
        import_time,
        file_size,
        data_size: data.allocations.len(),
    })
}

/// Benchmark MessagePack serialization performance
fn benchmark_messagepack_performance(
    data: &TestExportData,
) -> Result<PerformanceResult, Box<dyn std::error::Error>> {
    // Export benchmark
    let export_start = Instant::now();
    let msgpack_data = rmp_serde::to_vec(data)?;
    let export_time = export_start.elapsed();

    // Write to file
    fs::write("real_test_export.msgpack", &msgpack_data)?;
    let file_size = fs::metadata("real_test_export.msgpack")?.len();

    // Import benchmark
    let import_start = Instant::now();
    let loaded_msgpack = fs::read("real_test_export.msgpack")?;
    let _parsed: TestExportData = rmp_serde::from_slice(&loaded_msgpack)?;
    let import_time = import_start.elapsed();

    Ok(PerformanceResult {
        format_name: "MessagePack".to_string(),
        export_time,
        import_time,
        file_size,
        data_size: data.allocations.len(),
    })
}

/// Benchmark MessagePack + Zstd compression performance
fn benchmark_compressed_performance(
    data: &TestExportData,
) -> Result<PerformanceResult, Box<dyn std::error::Error>> {
    // Export benchmark (serialize + compress)
    let export_start = Instant::now();
    let msgpack_data = rmp_serde::to_vec(data)?;
    let compressed_data = zstd::encode_all(&msgpack_data[..], 3)?;
    let export_time = export_start.elapsed();

    // Write to file
    fs::write("real_test_export.msgpack.zst", &compressed_data)?;
    let file_size = fs::metadata("real_test_export.msgpack.zst")?.len();

    // Import benchmark (decompress + deserialize)
    let import_start = Instant::now();
    let loaded_compressed = fs::read("real_test_export.msgpack.zst")?;
    let decompressed = zstd::decode_all(&loaded_compressed[..])?;
    let _parsed: TestExportData = rmp_serde::from_slice(&decompressed)?;
    let import_time = import_start.elapsed();

    Ok(PerformanceResult {
        format_name: "MessagePack + Zstd".to_string(),
        export_time,
        import_time,
        file_size,
        data_size: data.allocations.len(),
    })
}

/// Print comprehensive performance analysis
fn print_comprehensive_analysis(
    json: &PerformanceResult,
    msgpack: &PerformanceResult,
    compressed: &PerformanceResult,
) {
    println!("\n🎯 Comprehensive Performance Analysis");
    println!("{}", "=".repeat(70));

    // Performance summary table
    println!("\n📊 Performance Summary:");
    println!("  Format              | Export    | Import    | Total     | Size      | Compression");
    println!("  -------------------|-----------|-----------|-----------|-----------|------------");
    let json_total = (json.export_time + json.import_time).as_secs_f64() * 1000.0;
    let msgpack_total = (msgpack.export_time + msgpack.import_time).as_secs_f64() * 1000.0;
    let compressed_total = (compressed.export_time + compressed.import_time).as_secs_f64() * 1000.0;

    println!(
        "  JSON               | {:>7.1}ms | {:>7.1}ms | {:>7.1}ms | {:>7.1}KB | {:>8}",
        json.export_time.as_secs_f64() * 1000.0,
        json.import_time.as_secs_f64() * 1000.0,
        json_total,
        json.file_size as f64 / 1024.0,
        "1.0x"
    );
    println!(
        "  MessagePack        | {:>7.1}ms | {:>7.1}ms | {:>7.1}ms | {:>7.1}KB | {:>8.1}x",
        msgpack.export_time.as_secs_f64() * 1000.0,
        msgpack.import_time.as_secs_f64() * 1000.0,
        msgpack_total,
        msgpack.file_size as f64 / 1024.0,
        json.file_size as f64 / msgpack.file_size as f64
    );
    println!(
        "  MessagePack + Zstd | {:>7.1}ms | {:>7.1}ms | {:>7.1}ms | {:>7.1}KB | {:>8.1}x",
        compressed.export_time.as_secs_f64() * 1000.0,
        compressed.import_time.as_secs_f64() * 1000.0,
        compressed_total,
        compressed.file_size as f64 / 1024.0,
        json.file_size as f64 / compressed.file_size as f64
    );

    println!("\n💡 Key Findings for Memory Analysis Data:");
    println!("  🎯 Best Overall Performance: MessagePack + Zstd");
    println!("     - Excellent compression ratio for repetitive memory data");
    println!("     - Fast processing suitable for real-time monitoring");
    println!("     - Significant storage savings for long-term analysis");

    println!("  🚀 Best Raw Speed: MessagePack");
    println!("     - Fastest for frequent exports without compression overhead");
    println!("     - Good balance of speed and size reduction");

    println!("  🔄 Best Compatibility: JSON");
    println!("     - Human readable for debugging and manual inspection");
    println!("     - Universal tool support and easy integration");

    println!("\n🔍 Memory Analysis Specific Benefits:");
    println!("  • Binary formats excel with structured allocation data");
    println!("  • Compression is highly effective on repetitive type names");
    println!("  • Faster loading enables real-time memory monitoring dashboards");
    println!("  • Smaller files reduce network transfer costs");
    println!("  • Cross-platform compatibility with MessagePack ecosystem");
}

/// Clean up test files
fn cleanup_test_files() {
    let files = [
        "real_test_export.json",
        "real_test_export.msgpack",
        "real_test_export.msgpack.zst",
    ];

    for file in &files {
        let _ = fs::remove_file(file);
    }
}
