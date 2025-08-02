//! Binary export performance benchmark tests
//!
//! This module contains comprehensive performance tests to verify that the binary export system
//! meets the performance targets of 3-5x speed improvement and 60-80% file size reduction
//! compared to JSON export.

use memscope_rs::core::tracker::{MemoryTracker, ExportOptions};
use memscope_rs::core::types::{AllocationInfo, MemoryStats, TrackingResult};
use memscope_rs::export::binary_exporter::{BinaryExporter, BinaryExportOptions};
use memscope_rs::export::binary_parser::BinaryParser;
use memscope_rs::export::binary_converter::BinaryConverter;
use memscope_rs::export::binary_format::CompressionType;
use memscope_rs::export::optimized_json_export::OptimizedExportOptions;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Performance benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub operation: String,
    pub format: String,
    pub duration: Duration,
    pub file_size: usize,
    pub throughput_mbps: f64,
    pub allocations_processed: usize,
}

impl BenchmarkResult {
    pub fn new(
        operation: &str,
        format: &str,
        duration: Duration,
        file_size: usize,
        allocations_processed: usize,
    ) -> Self {
        let throughput_mbps = if duration.as_secs_f64() > 0.0 {
            (file_size as f64) / (1024.0 * 1024.0) / duration.as_secs_f64()
        } else {
            0.0
        };

        Self {
            operation: operation.to_string(),
            format: format.to_string(),
            duration,
            file_size,
            throughput_mbps,
            allocations_processed,
        }
    }
}

/// Performance comparison results
#[derive(Debug, Clone)]
pub struct PerformanceComparison {
    pub binary_result: BenchmarkResult,
    pub json_result: BenchmarkResult,
    pub speed_improvement: f64,
    pub size_reduction_percent: f64,
    pub meets_speed_target: bool,
    pub meets_size_target: bool,
}

impl PerformanceComparison {
    pub fn new(binary_result: BenchmarkResult, json_result: BenchmarkResult) -> Self {
        let speed_improvement = if json_result.duration.as_secs_f64() > 0.0 {
            json_result.duration.as_secs_f64() / binary_result.duration.as_secs_f64()
        } else {
            0.0
        };

        let size_reduction_percent = if json_result.file_size > 0 {
            (1.0 - (binary_result.file_size as f64 / json_result.file_size as f64)) * 100.0
        } else {
            0.0
        };

        let meets_speed_target = speed_improvement >= 3.0; // Target: 3-5x improvement
        let meets_size_target = size_reduction_percent >= 60.0; // Target: 60-80% reduction

        Self {
            binary_result,
            json_result,
            speed_improvement,
            size_reduction_percent,
            meets_speed_target,
            meets_size_target,
        }
    }

    pub fn print_summary(&self) {
        println!("\n=== Performance Comparison Results ===");
        println!("Operation: {}", self.binary_result.operation);
        println!();
        
        println!("Binary Export:");
        println!("  Duration: {:.3}s", self.binary_result.duration.as_secs_f64());
        println!("  File Size: {} bytes ({:.2} MB)", 
            self.binary_result.file_size, 
            self.binary_result.file_size as f64 / (1024.0 * 1024.0));
        println!("  Throughput: {:.2} MB/s", self.binary_result.throughput_mbps);
        
        println!();
        println!("JSON Export:");
        println!("  Duration: {:.3}s", self.json_result.duration.as_secs_f64());
        println!("  File Size: {} bytes ({:.2} MB)", 
            self.json_result.file_size, 
            self.json_result.file_size as f64 / (1024.0 * 1024.0));
        println!("  Throughput: {:.2} MB/s", self.json_result.throughput_mbps);
        
        println!();
        println!("Performance Improvements:");
        println!("  Speed Improvement: {:.2}x {}", 
            self.speed_improvement,
            if self.meets_speed_target { "✅" } else { "❌" });
        println!("  Size Reduction: {:.1}% {}", 
            self.size_reduction_percent,
            if self.meets_size_target { "✅" } else { "❌" });
        
        println!();
        if self.meets_speed_target && self.meets_size_target {
            println!("🎉 All performance targets met!");
        } else {
            println!("⚠️  Some performance targets not met:");
            if !self.meets_speed_target {
                println!("   - Speed improvement {:.2}x < 3.0x target", self.speed_improvement);
            }
            if !self.meets_size_target {
                println!("   - Size reduction {:.1}% < 60% target", self.size_reduction_percent);
            }
        }
    }
}

/// Create a test memory tracker with sample data
fn create_test_tracker(allocation_count: usize) -> TrackingResult<MemoryTracker> {
    let mut tracker = MemoryTracker::new();
    
    // Generate sample allocations with realistic patterns
    for i in 0..allocation_count {
        let size = match i % 10 {
            0..=5 => 64 + (i % 1024),      // Small allocations (64B - 1KB)
            6..=8 => 1024 + (i % 8192),    // Medium allocations (1KB - 8KB)
            _ => 8192 + (i % 65536),       // Large allocations (8KB - 64KB)
        };
        
        let type_name = match i % 8 {
            0 => "Vec<i32>",
            1 => "String",
            2 => "HashMap<String,String>",
            3 => "Box<[u8]>",
            4 => "Arc<Mutex<Data>>",
            5 => "Vec<String>",
            6 => "BTreeMap<u64,Value>",
            _ => "CustomStruct",
        };
        
        let ptr = (0x1000000 + i * 8) as *mut u8;
        tracker.track_allocation(ptr as usize, size)?;
        
        // Simulate some deallocations to create realistic patterns
        if i > 100 && i % 7 == 0 {
            let dealloc_ptr = (0x1000000 + (i - 50) * 8) as *mut u8;
            let _ = tracker.track_deallocation(dealloc_ptr as usize);
        }
    }
    
    Ok(tracker)
}

/// Benchmark binary export performance
fn benchmark_binary_export(
    tracker: &MemoryTracker,
    temp_dir: &TempDir,
    options: BinaryExportOptions,
) -> TrackingResult<BenchmarkResult> {
    let output_path = temp_dir.path().join("benchmark.bin");
    
    let start_time = Instant::now();
    tracker.export_to_binary_with_options(&output_path, options)?;
    let duration = start_time.elapsed();
    
    let file_size = std::fs::metadata(&output_path)?.len() as usize;
    let allocations_processed = tracker.get_stats().unwrap().total_allocations;
    
    Ok(BenchmarkResult::new(
        "Export",
        "Binary",
        duration,
        file_size,
        allocations_processed,
    ))
}

/// Benchmark JSON export performance
fn benchmark_json_export(
    tracker: &MemoryTracker,
    temp_dir: &TempDir,
    options: OptimizedExportOptions,
) -> TrackingResult<BenchmarkResult> {
    let output_path = temp_dir.path().join("benchmark.json");
    
    let start_time = Instant::now();
    tracker.export_to_json_with_options(&output_path, ExportOptions::default())?;
    let duration = start_time.elapsed();
    
    let file_size = std::fs::metadata(&output_path)?.len() as usize;
    let allocations_processed = tracker.get_stats().unwrap().total_allocations;
    
    Ok(BenchmarkResult::new(
        "Export",
        "JSON",
        duration,
        file_size,
        allocations_processed,
    ))
}

/// Benchmark binary parsing performance
fn benchmark_binary_parsing(
    binary_path: &std::path::Path,
    temp_dir: &TempDir,
) -> TrackingResult<BenchmarkResult> {
    let start_time = Instant::now();
    
    let mut parser = BinaryParser::new();
    let _result = parser.load_from_file(binary_path)?;
    let allocations = parser.load_allocations()?;
    
    let duration = start_time.elapsed();
    let file_size = std::fs::metadata(binary_path)?.len() as usize;
    
    Ok(BenchmarkResult::new(
        "Parse",
        "Binary",
        duration,
        file_size,
        allocations.len(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_dataset_performance() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_test_tracker(1000).expect("Failed to create test tracker");
        
        println!("\n=== Small Dataset Performance Test (1K allocations) ===");
        
        // Test binary export
        let binary_options = BinaryExportOptions::default();
        let binary_result = benchmark_binary_export(&tracker, &temp_dir, binary_options)
            .expect("Binary export failed");
        
        // Test JSON export
        let json_options = OptimizedExportOptions::default();
        let json_result = benchmark_json_export(&tracker, &temp_dir, json_options)
            .expect("JSON export failed");
        
        let comparison = PerformanceComparison::new(binary_result, json_result);
        comparison.print_summary();
        
        // Verify basic performance expectations
        assert!(comparison.speed_improvement > 1.0, "Binary should be faster than JSON");
        assert!(comparison.size_reduction_percent > 0.0, "Binary should be smaller than JSON");
    }

    #[test]
    fn test_medium_dataset_performance() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_test_tracker(10000).expect("Failed to create test tracker");
        
        println!("\n=== Medium Dataset Performance Test (10K allocations) ===");
        
        // Test binary export with LZ4 compression
        let binary_options = BinaryExportOptions::default()
            .compression(CompressionType::Lz4)
            .parallel_encoding(true);
        let binary_result = benchmark_binary_export(&tracker, &temp_dir, binary_options)
            .expect("Binary export failed");
        
        // Test JSON export with optimization
        let json_options = crate::core::tracker::ExportOptions::default();
        let json_result = benchmark_json_export(&tracker, &temp_dir, json_options)
            .expect("JSON export failed");
        
        let comparison = PerformanceComparison::new(binary_result, json_result);
        comparison.print_summary();
        
        // More stringent performance expectations for medium datasets
        assert!(comparison.speed_improvement >= 2.0, 
            "Binary should be at least 2x faster for medium datasets, got {:.2}x", 
            comparison.speed_improvement);
        assert!(comparison.size_reduction_percent >= 40.0, 
            "Binary should achieve at least 40% size reduction, got {:.1}%", 
            comparison.size_reduction_percent);
    }

    #[test]
    fn test_large_dataset_performance() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_test_tracker(100000).expect("Failed to create test tracker");
        
        println!("\n=== Large Dataset Performance Test (100K allocations) ===");
        
        // Test binary export with Zstd compression and all optimizations
        let binary_options = BinaryExportOptions::comprehensive()
            .compression(CompressionType::Zstd)
            .parallel_encoding(true)
            .performance(memscope::export::binary_exporter::PerformanceConfig {
                use_memory_mapping: true,
                memory_mapping_config: None,
                enable_zero_copy: true,
                enable_simd: true,
                cache_size: 256 * 1024,
                batch_size: 2000,
            });
        let binary_result = benchmark_binary_export(&tracker, &temp_dir, binary_options)
            .expect("Binary export failed");
        
        // Test JSON export with maximum optimization
        let json_options = crate::core::tracker::ExportOptions::default();
        let json_result = benchmark_json_export(&tracker, &temp_dir, json_options)
            .expect("JSON export failed");
        
        let comparison = PerformanceComparison::new(binary_result, json_result);
        comparison.print_summary();
        
        // Target performance expectations for large datasets
        assert!(comparison.speed_improvement >= 3.0, 
            "Binary should be at least 3x faster for large datasets, got {:.2}x", 
            comparison.speed_improvement);
        assert!(comparison.size_reduction_percent >= 60.0, 
            "Binary should achieve at least 60% size reduction, got {:.1}%", 
            comparison.size_reduction_percent);
    }

    #[test]
    fn test_parsing_performance() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_test_tracker(50000).expect("Failed to create test tracker");
        
        println!("\n=== Binary Parsing Performance Test ===");
        
        // First create a binary file
        let binary_path = temp_dir.path().join("parse_test.bin");
        let binary_options = BinaryExportOptions::default()
            .compression(CompressionType::Lz4);
        tracker.export_to_binary_with_options(&binary_path, binary_options)
            .expect("Failed to create binary file");
        
        // Benchmark parsing
        let parse_result = benchmark_binary_parsing(&binary_path, &temp_dir)
            .expect("Binary parsing failed");
        
        println!("Binary Parsing Results:");
        println!("  Duration: {:.3}s", parse_result.duration.as_secs_f64());
        println!("  File Size: {} bytes ({:.2} MB)", 
            parse_result.file_size, 
            parse_result.file_size as f64 / (1024.0 * 1024.0));
        println!("  Throughput: {:.2} MB/s", parse_result.throughput_mbps);
        println!("  Allocations Parsed: {}", parse_result.allocations_processed);
        
        // Parsing should be very fast
        assert!(parse_result.throughput_mbps > 10.0, 
            "Binary parsing should achieve at least 10 MB/s, got {:.2} MB/s", 
            parse_result.throughput_mbps);
    }

    #[test]
    fn test_compression_comparison() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_test_tracker(25000).expect("Failed to create test tracker");
        
        println!("\n=== Compression Comparison Test ===");
        
        let compression_types = vec![
            ("None", CompressionType::None),
            ("LZ4", CompressionType::Lz4),
            ("Zstd", CompressionType::Zstd),
        ];
        
        let mut results = Vec::new();
        
        for (name, compression) in compression_types {
            let options = BinaryExportOptions::default()
                .compression(compression)
                .parallel_encoding(true);
            
            let result = benchmark_binary_export(&tracker, &temp_dir, options)
                .expect("Binary export failed");
            
            println!("{} Compression:", name);
            println!("  Duration: {:.3}s", result.duration.as_secs_f64());
            println!("  File Size: {} bytes ({:.2} MB)", 
                result.file_size, 
                result.file_size as f64 / (1024.0 * 1024.0));
            println!("  Throughput: {:.2} MB/s", result.throughput_mbps);
            println!();
            
            results.push((name, result));
        }
        
        // Verify compression effectiveness
        let none_size = results[0].1.file_size;
        let lz4_size = results[1].1.file_size;
        let zstd_size = results[2].1.file_size;
        
        assert!(lz4_size < none_size, "LZ4 should compress better than no compression");
        assert!(zstd_size < lz4_size, "Zstd should compress better than LZ4");
        
        let lz4_reduction = (1.0 - (lz4_size as f64 / none_size as f64)) * 100.0;
        let zstd_reduction = (1.0 - (zstd_size as f64 / none_size as f64)) * 100.0;
        
        println!("Compression Effectiveness:");
        println!("  LZ4 reduction: {:.1}%", lz4_reduction);
        println!("  Zstd reduction: {:.1}%", zstd_reduction);
        
        assert!(lz4_reduction > 20.0, "LZ4 should achieve at least 20% compression");
        assert!(zstd_reduction > 30.0, "Zstd should achieve at least 30% compression");
    }

    #[test]
    fn test_memory_usage_monitoring() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_test_tracker(20000).expect("Failed to create test tracker");
        
        println!("\n=== Memory Usage Monitoring Test ===");
        
        // Test with memory monitoring enabled
        let options = BinaryExportOptions::default()
            .performance(memscope::export::binary_exporter::PerformanceConfig {
                use_memory_mapping: true,
                memory_mapping_config: Some(
                    // MemoryMappingConfig { // Commented out for now
                        max_memory_usage: 100 * 1024 * 1024, // 100MB limit
                        ..Default::default()
                    }
                ),
                enable_zero_copy: true,
                enable_simd: false, // Disable for consistent testing
                cache_size: 64 * 1024,
                batch_size: 1000,
            })
            .memory_limit(50 * 1024 * 1024); // 50MB limit
        
        let result = benchmark_binary_export(&tracker, &temp_dir, options)
            .expect("Binary export with memory monitoring failed");
        
        println!("Memory-Monitored Export:");
        println!("  Duration: {:.3}s", result.duration.as_secs_f64());
        println!("  File Size: {} bytes ({:.2} MB)", 
            result.file_size, 
            result.file_size as f64 / (1024.0 * 1024.0));
        println!("  Throughput: {:.2} MB/s", result.throughput_mbps);
        
        // Should still achieve reasonable performance with memory monitoring
        assert!(result.throughput_mbps > 5.0, 
            "Memory-monitored export should still achieve at least 5 MB/s");
    }

    #[test]
    fn test_conversion_roundtrip_performance() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_test_tracker(15000).expect("Failed to create test tracker");
        
        println!("\n=== Conversion Roundtrip Performance Test ===");
        
        // Export to binary
        let binary_path = temp_dir.path().join("roundtrip.bin");
        let export_start = Instant::now();
        tracker.export_to_binary(&binary_path).expect("Binary export failed");
        let export_duration = export_start.elapsed();
        
        // Convert binary to JSON
        let json_path = temp_dir.path().join("roundtrip.json");
        let convert_start = Instant::now();
        BinaryConverter::binary_to_json(&binary_path, &json_path)
            .expect("Binary to JSON conversion failed");
        let convert_duration = convert_start.elapsed();
        
        let binary_size = std::fs::metadata(&binary_path).unwrap().len() as usize;
        let json_size = std::fs::metadata(&json_path).unwrap().len() as usize;
        
        println!("Roundtrip Performance:");
        println!("  Binary Export: {:.3}s", export_duration.as_secs_f64());
        println!("  Binary to JSON: {:.3}s", convert_duration.as_secs_f64());
        println!("  Total Time: {:.3}s", (export_duration + convert_duration).as_secs_f64());
        println!("  Binary Size: {} bytes ({:.2} MB)", binary_size, binary_size as f64 / (1024.0 * 1024.0));
        println!("  JSON Size: {} bytes ({:.2} MB)", json_size, json_size as f64 / (1024.0 * 1024.0));
        
        // Conversion should be fast
        let conversion_throughput = (binary_size as f64) / (1024.0 * 1024.0) / convert_duration.as_secs_f64();
        assert!(conversion_throughput > 20.0, 
            "Binary to JSON conversion should achieve at least 20 MB/s, got {:.2} MB/s", 
            conversion_throughput);
    }
}

/// Performance regression detection
#[cfg(test)]
mod regression_tests {
    use super::*;

    /// Baseline performance metrics (update these when making performance improvements)
    const BASELINE_SMALL_SPEED_IMPROVEMENT: f64 = 2.0;
    const BASELINE_MEDIUM_SPEED_IMPROVEMENT: f64 = 3.0;
    const BASELINE_LARGE_SPEED_IMPROVEMENT: f64 = 4.0;
    const BASELINE_SIZE_REDUCTION: f64 = 50.0;

    #[test]
    fn test_performance_regression_small() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_test_tracker(1000).expect("Failed to create test tracker");
        
        let binary_result = benchmark_binary_export(&tracker, &temp_dir, BinaryExportOptions::default())
            .expect("Binary export failed");
        let json_result = benchmark_json_export(&tracker, &temp_dir, OptimizedExportOptions::default())
            .expect("JSON export failed");
        
        let comparison = PerformanceComparison::new(binary_result, json_result);
        
        assert!(comparison.speed_improvement >= BASELINE_SMALL_SPEED_IMPROVEMENT,
            "Performance regression detected: speed improvement {:.2}x < baseline {:.2}x",
            comparison.speed_improvement, BASELINE_SMALL_SPEED_IMPROVEMENT);
    }

    #[test]
    fn test_performance_regression_medium() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let tracker = create_test_tracker(10000).expect("Failed to create test tracker");
        
        let binary_result = benchmark_binary_export(&tracker, &temp_dir, BinaryExportOptions::default())
            .expect("Binary export failed");
        let json_result = benchmark_json_export(&tracker, &temp_dir, OptimizedExportOptions::default())
            .expect("JSON export failed");
        
        let comparison = PerformanceComparison::new(binary_result, json_result);
        
        assert!(comparison.speed_improvement >= BASELINE_MEDIUM_SPEED_IMPROVEMENT,
            "Performance regression detected: speed improvement {:.2}x < baseline {:.2}x",
            comparison.speed_improvement, BASELINE_MEDIUM_SPEED_IMPROVEMENT);
        assert!(comparison.size_reduction_percent >= BASELINE_SIZE_REDUCTION,
            "Size regression detected: reduction {:.1}% < baseline {:.1}%",
            comparison.size_reduction_percent, BASELINE_SIZE_REDUCTION);
    }
}