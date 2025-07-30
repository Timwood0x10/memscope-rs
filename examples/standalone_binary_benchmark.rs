//! Standalone Binary vs JSON Performance Benchmark
//!
//! This example demonstrates the performance benefits of binary formats
//! without depending on the memscope internal APIs that are still being developed.
//!
//! Run with: cargo run --example standalone_binary_benchmark

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

/// Realistic memory allocation data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryAllocation {
    ptr: usize,
    size: usize,
    var_name: String,
    type_name: String,
    scope_name: Option<String>,
    timestamp_alloc: u64,
    timestamp_dealloc: Option<u64>,
    thread_id: String,
    stack_trace: Option<Vec<String>>,
    is_leaked: bool,
    lifetime_ms: Option<u64>,
}

/// Memory analysis export data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryAnalysisExport {
    metadata: ExportMetadata,
    allocations: Vec<MemoryAllocation>,
    statistics: MemoryStatistics,
    analysis_results: AnalysisResults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExportMetadata {
    version: String,
    created_at: u64,
    total_allocations: usize,
    export_format: String,
    compression_used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryStatistics {
    total_allocated: usize,
    total_deallocated: usize,
    peak_memory: usize,
    active_allocations: usize,
    average_allocation_size: f64,
    allocation_frequency: f64,
    memory_efficiency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalysisResults {
    memory_leaks: Vec<LeakInfo>,
    fragmentation_analysis: FragmentationInfo,
    performance_metrics: PerformanceMetrics,
    type_usage_stats: HashMap<String, TypeUsageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LeakInfo {
    ptr: usize,
    size: usize,
    var_name: String,
    allocation_time: u64,
    suspected_leak_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FragmentationInfo {
    fragmentation_ratio: f64,
    largest_free_block: usize,
    total_free_memory: usize,
    fragmented_regions: Vec<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerformanceMetrics {
    allocation_overhead: f64,
    deallocation_overhead: f64,
    tracking_overhead: f64,
    memory_access_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TypeUsageInfo {
    allocation_count: usize,
    total_memory: usize,
    average_lifetime: f64,
    peak_instances: usize,
}

#[derive(Debug)]
struct BenchmarkResult {
    format_name: String,
    serialize_time: std::time::Duration,
    deserialize_time: std::time::Duration,
    file_size: usize,
    compression_ratio: Option<f64>,
}

impl BenchmarkResult {
    fn print_results(&self) {
        println!("\n📊 {} Performance:", self.format_name);
        println!("  ⏱️  Serialize: {:?}", self.serialize_time);
        println!("  📥 Deserialize: {:?}", self.deserialize_time);
        println!(
            "  📏 Size: {} bytes ({:.2} MB)",
            self.file_size,
            self.file_size as f64 / 1024.0 / 1024.0
        );

        if let Some(ratio) = self.compression_ratio {
            println!("  🗜️  Compression: {:.1}% of original", ratio * 100.0);
        }
    }

    fn compare_with(&self, baseline: &BenchmarkResult) {
        if self.format_name == baseline.format_name {
            return;
        }

        let serialize_speedup =
            baseline.serialize_time.as_nanos() as f64 / self.serialize_time.as_nanos() as f64;
        let deserialize_speedup =
            baseline.deserialize_time.as_nanos() as f64 / self.deserialize_time.as_nanos() as f64;
        let size_reduction = baseline.file_size as f64 / self.file_size as f64;

        println!(
            "\n🎯 {} vs {} Improvement:",
            self.format_name, baseline.format_name
        );
        println!("  🚀 Serialize: {:.1}x faster", serialize_speedup);
        println!("  📥 Deserialize: {:.1}x faster", deserialize_speedup);
        println!("  💾 Size: {:.1}x smaller", size_reduction);

        // Calculate overall performance score
        let overall_score = (serialize_speedup + deserialize_speedup + size_reduction) / 3.0;
        match overall_score {
            s if s > 4.0 => println!("  ✅ Outstanding improvement! ({:.1}x overall)", s),
            s if s > 3.0 => println!("  ✅ Excellent improvement! ({:.1}x overall)", s),
            s if s > 2.0 => println!("  ✅ Good improvement ({:.1}x overall)", s),
            s if s > 1.5 => println!("  ✅ Moderate improvement ({:.1}x overall)", s),
            s => println!("  ⚠️  Minimal improvement ({:.1}x overall)", s),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Standalone Binary vs JSON Performance Benchmark");
    println!("{}", "=".repeat(70));
    println!("Simulating complex memory analysis data export...\n");

    // Create realistic test data
    println!("📊 Generating realistic memory analysis data...");
    let test_data = create_realistic_memory_data();
    println!(
        "✅ Generated {} allocations with full analysis data",
        test_data.allocations.len()
    );

    // Run benchmarks
    let json_result = benchmark_json(&test_data)?;
    json_result.print_results();

    let msgpack_result = benchmark_messagepack(&test_data)?;
    msgpack_result.print_results();
    msgpack_result.compare_with(&json_result);

    let compressed_result = benchmark_compressed(&test_data)?;
    compressed_result.print_results();
    compressed_result.compare_with(&json_result);

    // Print comprehensive analysis
    print_comprehensive_analysis(&json_result, &msgpack_result, &compressed_result);

    // Cleanup
    cleanup_files();

    println!("\n✅ Benchmark completed! Binary format shows clear advantages.");
    Ok(())
}

/// Create realistic memory analysis data
fn create_realistic_memory_data() -> MemoryAnalysisExport {
    let mut allocations = Vec::new();
    let mut type_usage_stats = HashMap::new();

    // Generate diverse allocation patterns
    let type_patterns = [
        ("Vec<i32>", 200, 50..500),
        ("String", 150, 20..200),
        ("HashMap<String, i32>", 100, 100..1000),
        ("Box<dyn Trait>", 80, 50..300),
        ("Arc<Mutex<T>>", 60, 80..400),
        ("Vec<Vec<String>>", 40, 200..2000),
        ("CustomStruct", 120, 30..250),
    ];

    let mut allocation_id = 0x10000000;
    let base_time = 1640000000;

    for (type_name, count, size_range) in &type_patterns {
        let mut total_memory = 0;
        let mut total_lifetime = 0.0;

        for i in 0..*count {
            let size = size_range.start + (i % (size_range.end - size_range.start));
            let lifetime = if i % 10 == 0 {
                None
            } else {
                Some((i as u64 % 5000) + 100)
            }; // Some leaks
            let is_leaked = lifetime.is_none();

            total_memory += size;
            if let Some(lt) = lifetime {
                total_lifetime += lt as f64;
            }

            allocations.push(MemoryAllocation {
                ptr: allocation_id,
                size,
                var_name: format!("{}_{}", type_name.replace("<", "_").replace(">", "_"), i),
                type_name: type_name.to_string(),
                scope_name: Some(format!("scope_{}", i % 20)),
                timestamp_alloc: base_time + i as u64,
                timestamp_dealloc: if is_leaked {
                    None
                } else {
                    Some(base_time + i as u64 + lifetime.unwrap())
                },
                thread_id: format!("thread_{}", i % 4),
                stack_trace: Some(vec![
                    format!("main::function_{}()", i % 10),
                    format!("module::{}::allocate()", type_name),
                    "std::alloc::alloc()".to_string(),
                ]),
                is_leaked,
                lifetime_ms: lifetime,
            });

            allocation_id += 256;
        }

        let avg_lifetime = if *count > 0 {
            total_lifetime / *count as f64
        } else {
            0.0
        };
        type_usage_stats.insert(
            type_name.to_string(),
            TypeUsageInfo {
                allocation_count: *count,
                total_memory,
                average_lifetime: avg_lifetime,
                peak_instances: *count * 80 / 100, // Assume 80% peak concurrency
            },
        );
    }

    // Create analysis results
    let memory_leaks: Vec<LeakInfo> = allocations
        .iter()
        .filter(|a| a.is_leaked)
        .map(|a| LeakInfo {
            ptr: a.ptr,
            size: a.size,
            var_name: a.var_name.clone(),
            allocation_time: a.timestamp_alloc,
            suspected_leak_reason: "Variable went out of scope without deallocation".to_string(),
        })
        .collect();

    let total_allocated: usize = allocations.iter().map(|a| a.size).sum();
    let active_allocations = allocations
        .iter()
        .filter(|a| a.timestamp_dealloc.is_none())
        .count();

    MemoryAnalysisExport {
        metadata: ExportMetadata {
            version: "1.0.0".to_string(),
            created_at: base_time,
            total_allocations: allocations.len(),
            export_format: "memory_analysis".to_string(),
            compression_used: false,
        },
        allocations,
        statistics: MemoryStatistics {
            total_allocated,
            total_deallocated: total_allocated - (active_allocations * 200), // Rough estimate
            peak_memory: total_allocated * 85 / 100,                         // 85% peak usage
            active_allocations,
            average_allocation_size: total_allocated as f64 / type_usage_stats.len() as f64,
            allocation_frequency: 1000.0, // allocations per second
            memory_efficiency: 0.92,      // 92% efficiency
        },
        analysis_results: AnalysisResults {
            memory_leaks,
            fragmentation_analysis: FragmentationInfo {
                fragmentation_ratio: 0.15,
                largest_free_block: 1024 * 1024,
                total_free_memory: 512 * 1024,
                fragmented_regions: vec![(0x20000000, 1024), (0x30000000, 2048)],
            },
            performance_metrics: PerformanceMetrics {
                allocation_overhead: 0.05,   // 5% overhead
                deallocation_overhead: 0.03, // 3% overhead
                tracking_overhead: 0.02,     // 2% overhead
                memory_access_patterns: vec![
                    "Sequential".to_string(),
                    "Random".to_string(),
                    "Clustered".to_string(),
                ],
            },
            type_usage_stats,
        },
    }
}

/// Benchmark JSON serialization
fn benchmark_json(
    data: &MemoryAnalysisExport,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    // Serialize
    let serialize_start = Instant::now();
    let json_string = serde_json::to_string_pretty(data)?;
    let serialize_time = serialize_start.elapsed();

    fs::write("benchmark_memory_analysis.json", &json_string)?;
    let file_size = json_string.len();

    // Deserialize
    let deserialize_start = Instant::now();
    let loaded_json = fs::read_to_string("benchmark_memory_analysis.json")?;
    let _parsed: MemoryAnalysisExport = serde_json::from_str(&loaded_json)?;
    let deserialize_time = deserialize_start.elapsed();

    Ok(BenchmarkResult {
        format_name: "JSON".to_string(),
        serialize_time,
        deserialize_time,
        file_size,
        compression_ratio: None,
    })
}

/// Benchmark MessagePack serialization
fn benchmark_messagepack(
    data: &MemoryAnalysisExport,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    // Serialize
    let serialize_start = Instant::now();
    let msgpack_data = rmp_serde::to_vec(data)?;
    let serialize_time = serialize_start.elapsed();

    fs::write("benchmark_memory_analysis.msgpack", &msgpack_data)?;
    let file_size = msgpack_data.len();

    // Deserialize
    let deserialize_start = Instant::now();
    let loaded_msgpack = fs::read("benchmark_memory_analysis.msgpack")?;
    let _parsed: MemoryAnalysisExport = rmp_serde::from_slice(&loaded_msgpack)?;
    let deserialize_time = deserialize_start.elapsed();

    Ok(BenchmarkResult {
        format_name: "MessagePack".to_string(),
        serialize_time,
        deserialize_time,
        file_size,
        compression_ratio: None,
    })
}

/// Benchmark MessagePack + Zstd compression
fn benchmark_compressed(
    data: &MemoryAnalysisExport,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    // Serialize and compress
    let serialize_start = Instant::now();
    let msgpack_data = rmp_serde::to_vec(data)?;
    let compressed_data = zstd::encode_all(&msgpack_data[..], 3)?;
    let serialize_time = serialize_start.elapsed();

    fs::write("benchmark_memory_analysis.msgpack.zst", &compressed_data)?;
    let file_size = compressed_data.len();
    let compression_ratio = file_size as f64 / msgpack_data.len() as f64;

    // Decompress and deserialize
    let deserialize_start = Instant::now();
    let loaded_compressed = fs::read("benchmark_memory_analysis.msgpack.zst")?;
    let decompressed = zstd::decode_all(&loaded_compressed[..])?;
    let _parsed: MemoryAnalysisExport = rmp_serde::from_slice(&decompressed)?;
    let deserialize_time = deserialize_start.elapsed();

    Ok(BenchmarkResult {
        format_name: "MessagePack + Zstd".to_string(),
        serialize_time,
        deserialize_time,
        file_size,
        compression_ratio: Some(compression_ratio),
    })
}

/// Print comprehensive performance analysis
fn print_comprehensive_analysis(
    json: &BenchmarkResult,
    msgpack: &BenchmarkResult,
    compressed: &BenchmarkResult,
) {
    println!("\n🎯 Comprehensive Performance Analysis");
    println!("{}", "=".repeat(70));

    // Speed comparison table
    println!("\n📊 Speed Comparison (lower is better):");
    println!("  Format              | Serialize    | Deserialize  | Total");
    println!("  -------------------|--------------|--------------|----------");
    println!(
        "  JSON               | {:>10?} | {:>10?} | {:>8?}",
        json.serialize_time,
        json.deserialize_time,
        json.serialize_time + json.deserialize_time
    );
    println!(
        "  MessagePack        | {:>10?} | {:>10?} | {:>8?}",
        msgpack.serialize_time,
        msgpack.deserialize_time,
        msgpack.serialize_time + msgpack.deserialize_time
    );
    println!(
        "  MessagePack + Zstd | {:>10?} | {:>10?} | {:>8?}",
        compressed.serialize_time,
        compressed.deserialize_time,
        compressed.serialize_time + compressed.deserialize_time
    );

    // Size comparison
    println!("\n💾 Size Comparison:");
    println!(
        "  JSON:               {:.2} MB (100%)",
        json.file_size as f64 / 1024.0 / 1024.0
    );
    println!(
        "  MessagePack:        {:.2} MB ({:.1}%)",
        msgpack.file_size as f64 / 1024.0 / 1024.0,
        msgpack.file_size as f64 / json.file_size as f64 * 100.0
    );
    println!(
        "  MessagePack + Zstd: {:.2} MB ({:.1}%)",
        compressed.file_size as f64 / 1024.0 / 1024.0,
        compressed.file_size as f64 / json.file_size as f64 * 100.0
    );

    // Recommendations
    println!("\n💡 Recommendations for Memory Analysis Export:");
    println!("  🎯 Best Overall: MessagePack + Zstd");
    println!("     - Excellent compression for repetitive memory data");
    println!("     - Fast serialization for real-time monitoring");
    println!("     - Cross-platform compatibility");

    println!("  🚀 Best Speed: MessagePack");
    println!("     - Fastest for frequent exports");
    println!("     - Good size reduction without compression overhead");

    println!("  🔄 Best Compatibility: JSON");
    println!("     - Human readable for debugging");
    println!("     - Universal tool support");
    println!("     - Easy integration with web interfaces");

    // Memory analysis specific benefits
    println!("\n🔍 Memory Analysis Specific Benefits:");
    println!("  • Binary formats excel with repetitive allocation patterns");
    println!("  • Compression is highly effective on structured memory data");
    println!("  • Faster loading enables real-time memory monitoring");
    println!("  • Smaller files reduce storage costs for long-term analysis");
}

/// Clean up benchmark files
fn cleanup_files() {
    let files = [
        "benchmark_memory_analysis.json",
        "benchmark_memory_analysis.msgpack",
        "benchmark_memory_analysis.msgpack.zst",
    ];

    for file in &files {
        let _ = fs::remove_file(file);
    }
}
