//! 🚀 Performance Stress Test - Lite Version
//!
//! Tests our ultra-high performance optimization solution under different data scales

use memscope_rs::export::binary::BinaryParser;
use memscope_rs::{get_global_tracker, init, track_var};
use std::time::Instant;

/// Test configuration for different scale scenarios
struct TestConfig {
    name: &'static str,
    allocation_count: usize,
    target_time_ms: u128,
    description: &'static str,
}

/// Test result containing performance metrics
#[derive(Debug, Clone)]
struct TestResult {
    parse_time_ms: u128,
    binary_size_kb: f64,
    json_size_kb: f64,
    #[allow(dead_code)]
    creation_time_ms: u128,
    #[allow(dead_code)]
    export_time_ms: u128,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    println!("🚀 Performance Stress Test Started!");
    println!("{}", "=".repeat(60));
    println!("Testing ultra-high performance optimization across different data scales");
    println!("{}", "=".repeat(60));

    // Progressive testing with different scales
    let test_configs = vec![
        TestConfig {
            name: "Small Scale",
            allocation_count: 2_500, // 2.5K allocations
            target_time_ms: 75,
            description: "Basic performance validation with small dataset",
        },
        TestConfig {
            name: "Medium Scale",
            allocation_count: 15_000, // 15K allocations
            target_time_ms: 150,
            description: "Standard workload simulation",
        },
        TestConfig {
            name: "Large Scale",
            allocation_count: 75_000, // 75K allocations
            target_time_ms: 400,
            description: "High-throughput scenario testing",
        },
        TestConfig {
            name: "Extra Large Scale",
            allocation_count: 150_000, // 150K allocations
            target_time_ms: 750,
            description: "Maximum capacity stress testing",
        },
        TestConfig {
            name: "Enterprise Scale",
            allocation_count: 250_000, // 250K allocations
            target_time_ms: 1200,
            description: "Enterprise-level workload simulation",
        },
    ];

    let mut test_results = Vec::new();
    let total_start = Instant::now();

    for (index, config) in test_configs.iter().enumerate() {
        println!(
            "\n🔥 Starting Test {}/{}: {} ({} allocations)",
            index + 1,
            test_configs.len(),
            config.name,
            config.allocation_count
        );
        println!("📝 Description: {}", config.description);

        match run_performance_test(&config) {
            Ok(test_result) => {
                println!("✅ {} Completed Successfully!", config.name);
                println!("📊 Parse Time: {}ms", test_result.parse_time_ms);
                println!("💾 Binary Size: {:.2}KB", test_result.binary_size_kb);
                println!("📄 JSON Size: {:.2}KB", test_result.json_size_kb);

                if test_result.parse_time_ms <= config.target_time_ms {
                    println!(
                        "🎉 Performance Target ACHIEVED: {}ms <= {}ms",
                        test_result.parse_time_ms, config.target_time_ms
                    );
                } else {
                    println!(
                        "⚠️  Performance Target MISSED: {}ms > {}ms",
                        test_result.parse_time_ms, config.target_time_ms
                    );
                }

                // Calculate throughput
                let throughput =
                    config.allocation_count as f64 / (test_result.parse_time_ms as f64 / 1000.0);
                println!(
                    "🚀 Processing Throughput: {:.0} allocations/sec",
                    throughput
                );

                // Calculate compression ratio
                let compression_ratio = test_result.json_size_kb / test_result.binary_size_kb;
                println!(
                    "📦 Compression Ratio: {:.2}x (JSON/Binary)",
                    compression_ratio
                );

                test_results.push((config, test_result));
            }
            Err(e) => {
                println!("❌ {} FAILED: {}", config.name, e);
            }
        }

        println!("{}", "-".repeat(60));

        // Add a small delay between tests to prevent resource exhaustion
        if index < test_configs.len() - 1 {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    let total_time = total_start.elapsed();

    // Print comprehensive summary
    print_performance_summary(&test_results, total_time);

    println!("\n🏁 Performance Stress Test Completed!");
    Ok(())
}

/// Run individual performance test
fn run_performance_test(config: &TestConfig) -> Result<TestResult, Box<dyn std::error::Error>> {
    // Step 1: Create test data
    println!(
        "📊 Creating {} allocation records...",
        config.allocation_count
    );
    let creation_start = Instant::now();

    create_test_data(config.allocation_count)?;

    let creation_time = creation_start.elapsed();
    let creation_time_ms = creation_time.as_millis();
    println!("✅ Data creation completed: {}ms", creation_time_ms);

    // Step 2: Export to binary
    println!("💾 Exporting to binary file...");
    let export_start = Instant::now();

    let tracker = get_global_tracker();
    let binary_file = format!(
        "MemoryAnalysis/perf_test_{}.memscope",
        config.allocation_count
    );
    tracker.export_to_binary(&binary_file)?;

    let export_time = export_start.elapsed();
    let export_time_ms = export_time.as_millis();
    let file_size = std::fs::metadata(&binary_file)?.len();
    let binary_size_kb = file_size as f64 / 1024.0;
    println!(
        "✅ Binary export completed: {}ms, File size: {:.2}KB",
        export_time_ms, binary_size_kb
    );

    // Step 3: Ultra-high performance parsing to JSON
    println!("🚀 Ultra-fast parsing to JSON...");
    let parse_start = Instant::now();

    let output_name = format!("perf_test_{}", config.allocation_count);
    BinaryParser::parse_user_binary_to_json(&binary_file, &output_name)?;

    let parse_time = parse_start.elapsed();
    let parse_time_ms = parse_time.as_millis();

    // Step 4: Calculate JSON file sizes
    let json_size = calculate_json_size(&output_name)?;
    let json_size_kb = json_size as f64 / 1024.0;
    println!("✅ JSON files total size: {:.2}KB", json_size_kb);

    // Clean up test files
    cleanup_test_files(&binary_file, &output_name)?;

    Ok(TestResult {
        parse_time_ms,
        binary_size_kb,
        json_size_kb,
        creation_time_ms,
        export_time_ms,
    })
}

/// Create test data with various object sizes
fn create_test_data(count: usize) -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..count {
        // Create different sized test data to simulate real-world scenarios
        let size = match i % 6 {
            0 => 32,    // Tiny objects (primitives)
            1 => 128,   // Small objects (small structs)
            2 => 512,   // Medium objects (collections)
            3 => 2048,  // Large objects (buffers)
            4 => 8192,  // Very large objects (big data structures)
            5 => 16384, // Extra large objects (massive arrays)
            _ => 64,
        };

        // Create more realistic data patterns
        let data = if i % 10 == 0 {
            // Simulate string data
            format!(
                "test_string_data_{}_with_content_{}",
                i,
                "x".repeat(size / 20)
            )
            .into_bytes()
        } else {
            // Simulate binary data
            vec![(i % 256) as u8; size]
        };

        track_var!(data);

        // Simulate different lifetime patterns
        if i % 3 == 0 {
            // Short-lived objects (immediately dropped)
            drop(data);
        } else if i % 7 == 0 {
            // Medium-lived objects (stored temporarily)
            let _temp_storage = data;
            // Implicitly dropped at end of scope
        } else {
            // Long-lived objects
            drop(data);
        }
    }

    Ok(())
}

/// Calculate total JSON file sizes
fn calculate_json_size(output_name: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let json_files = [
        format!(
            "MemoryAnalysis/{}/{}_memory_analysis.json",
            output_name, output_name
        ),
        format!(
            "MemoryAnalysis/{}/{}_lifetime.json",
            output_name, output_name
        ),
        format!(
            "MemoryAnalysis/{}/{}_performance.json",
            output_name, output_name
        ),
        format!(
            "MemoryAnalysis/{}/{}_unsafe_ffi.json",
            output_name, output_name
        ),
        format!(
            "MemoryAnalysis/{}/{}_complex_types.json",
            output_name, output_name
        ),
    ];

    let mut total_size = 0u64;
    for file_path in &json_files {
        if let Ok(metadata) = std::fs::metadata(file_path) {
            total_size += metadata.len();
        }
    }

    Ok(total_size)
}

/// Clean up test files to prevent disk space issues
fn cleanup_test_files(
    binary_file: &str,
    output_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Remove binary file
    if std::path::Path::new(binary_file).exists() {
        std::fs::remove_file(binary_file)?;
    }

    // Remove JSON output directory
    let output_dir = format!("MemoryAnalysis/{}", output_name);
    if std::path::Path::new(&output_dir).exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    Ok(())
}

/// Print comprehensive performance summary
fn print_performance_summary(
    test_results: &[(&TestConfig, TestResult)],
    total_time: std::time::Duration,
) {
    println!("\n📊 COMPREHENSIVE PERFORMANCE SUMMARY");
    println!("{}", "=".repeat(80));

    if test_results.is_empty() {
        println!("❌ No successful test results to summarize");
        return;
    }

    // Calculate aggregate statistics
    let mut total_allocations = 0;
    let mut total_parse_time = 0;
    let mut total_binary_size = 0.0;
    let mut total_json_size = 0.0;
    let mut passed_tests = 0;

    println!("Individual Test Results:");
    println!("{:-<80}", "");
    println!(
        "{:<20} {:<12} {:<12} {:<12} {:<12} {:<8}",
        "Test Name", "Allocations", "Parse(ms)", "Binary(KB)", "JSON(KB)", "Status"
    );
    println!("{:-<80}", "");

    for (config, result) in test_results {
        let status = if result.parse_time_ms <= config.target_time_ms {
            "PASS"
        } else {
            "FAIL"
        };
        if result.parse_time_ms <= config.target_time_ms {
            passed_tests += 1;
        }

        println!(
            "{:<20} {:<12} {:<12} {:<12.1} {:<12.1} {:<8}",
            config.name,
            config.allocation_count,
            result.parse_time_ms,
            result.binary_size_kb,
            result.json_size_kb,
            status
        );

        total_allocations += config.allocation_count;
        total_parse_time += result.parse_time_ms;
        total_binary_size += result.binary_size_kb;
        total_json_size += result.json_size_kb;
    }

    println!("{:-<80}", "");

    // Performance metrics
    let avg_throughput = total_allocations as f64 / (total_parse_time as f64 / 1000.0);
    let overall_compression_ratio = total_json_size / total_binary_size;
    let success_rate = (passed_tests as f64 / test_results.len() as f64) * 100.0;

    println!("\nAggregate Performance Metrics:");
    println!("• Total Allocations Processed: {}", total_allocations);
    println!("• Total Parse Time: {}ms", total_parse_time);
    println!(
        "• Average Throughput: {:.0} allocations/sec",
        avg_throughput
    );
    println!("• Total Binary Size: {:.1}KB", total_binary_size);
    println!("• Total JSON Size: {:.1}KB", total_json_size);
    println!(
        "• Overall Compression Ratio: {:.2}x",
        overall_compression_ratio
    );
    println!(
        "• Test Success Rate: {:.1}% ({}/{})",
        success_rate,
        passed_tests,
        test_results.len()
    );
    println!("• Total Test Runtime: {:.2}s", total_time.as_secs_f64());

    // Performance analysis
    println!("\nPerformance Analysis:");
    if success_rate >= 80.0 {
        println!("🎉 EXCELLENT: High success rate indicates robust performance");
    } else if success_rate >= 60.0 {
        println!("⚠️  GOOD: Moderate success rate, some optimization opportunities");
    } else {
        println!("❌ NEEDS IMPROVEMENT: Low success rate indicates performance issues");
    }

    if avg_throughput >= 100_000.0 {
        println!("🚀 OUTSTANDING: Ultra-high throughput achieved");
    } else if avg_throughput >= 50_000.0 {
        println!("✅ EXCELLENT: High throughput performance");
    } else if avg_throughput >= 10_000.0 {
        println!("👍 GOOD: Solid throughput performance");
    } else {
        println!("⚠️  MODERATE: Throughput could be improved");
    }

    if overall_compression_ratio <= 2.0 {
        println!("📦 EXCELLENT: Superior compression efficiency");
    } else if overall_compression_ratio <= 3.0 {
        println!("👍 GOOD: Solid compression performance");
    } else {
        println!("⚠️  MODERATE: Compression could be optimized");
    }
}
