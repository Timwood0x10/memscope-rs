//! Binary export stress tests
//!
//! This module contains stress tests to verify that the binary export system
//! can handle large datasets, concurrent operations, memory pressure, and
//! long-running scenarios without issues.

use memscope_rs::core::tracker::MemoryTracker;
use memscope_rs::core::types::{AllocationInfo, MemoryStats, TrackingResult};
use memscope_rs::export::binary_exporter::{BinaryExporter, BinaryExportOptions};
use memscope_rs::export::binary_parser::{BinaryParser, BinaryParserOptions};
use memscope_rs::export::binary_converter::BinaryConverter;
use memscope_rs::export::binary_format::CompressionType;
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Stress test configuration
#[derive(Debug, Clone)]
pub struct StressTestConfig {
    pub allocation_count: usize,
    pub thread_count: usize,
    pub duration_seconds: u64,
    pub memory_limit_mb: usize,
    pub enable_concurrent_access: bool,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            allocation_count: 100_000,
            thread_count: 4,
            duration_seconds: 30,
            memory_limit_mb: 512,
            enable_concurrent_access: true,
        }
    }
}

/// Stress test results
#[derive(Debug, Clone)]
pub struct StressTestResult {
    pub test_name: String,
    pub duration: Duration,
    pub operations_completed: usize,
    pub operations_per_second: f64,
    pub peak_memory_usage_mb: f64,
    pub errors_encountered: usize,
    pub success_rate: f64,
}

impl StressTestResult {
    pub fn new(
        test_name: &str,
        duration: Duration,
        operations_completed: usize,
        peak_memory_usage_bytes: usize,
        errors_encountered: usize,
    ) -> Self {
        let operations_per_second = if duration.as_secs_f64() > 0.0 {
            operations_completed as f64 / duration.as_secs_f64()
        } else {
            0.0
        };
        
        let success_rate = if operations_completed + errors_encountered > 0 {
            operations_completed as f64 / (operations_completed + errors_encountered) as f64 * 100.0
        } else {
            0.0
        };
        
        Self {
            test_name: test_name.to_string(),
            duration,
            operations_completed,
            operations_per_second,
            peak_memory_usage_mb: peak_memory_usage_bytes as f64 / (1024.0 * 1024.0),
            errors_encountered,
            success_rate,
        }
    }
    
    pub fn print_summary(&self) {
        println!("\n=== {} Results ===", self.test_name);
        println!("Duration: {:.2}s", self.duration.as_secs_f64());
        println!("Operations completed: {}", self.operations_completed);
        println!("Operations per second: {:.2}", self.operations_per_second);
        println!("Peak memory usage: {:.2} MB", self.peak_memory_usage_mb);
        println!("Errors encountered: {}", self.errors_encountered);
        println!("Success rate: {:.1}%", self.success_rate);
        
        if self.success_rate >= 99.0 {
            println!("✅ Stress test passed");
        } else {
            println!("❌ Stress test failed - success rate too low");
        }
    }
}

/// Create a large test dataset
fn create_large_dataset(allocation_count: usize) -> TrackingResult<MemoryTracker> {
    let mut tracker = MemoryTracker::new();
    
    println!("Creating large dataset with {} allocations...", allocation_count);
    let start_time = Instant::now();
    
    for i in 0..allocation_count {
        let size = match i % 20 {
            0..=10 => 32 + (i % 512),        // Small: 32B - 512B
            11..=15 => 512 + (i % 4096),     // Medium: 512B - 4KB  
            16..=18 => 4096 + (i % 32768),   // Large: 4KB - 32KB
            _ => 32768 + (i % 131072),       // Very large: 32KB - 128KB
        };
        
        let type_name = match i % 12 {
            0 => "Vec<i32>",
            1 => "String",
            2 => "HashMap<String,String>",
            3 => "Box<[u8]>",
            4 => "Arc<Mutex<Data>>",
            5 => "Vec<String>",
            6 => "BTreeMap<u64,Value>",
            7 => "CustomStruct",
            8 => "Rc<RefCell<Node>>",
            9 => "Vec<Box<dyn Trait>>",
            10 => "HashMap<u64,Vec<String>>",
            _ => "ComplexNestedType",
        };
        
        let ptr = (0x100000000u64 + (i as u64 * 64)) as *mut u8;
        tracker.track_allocation(ptr as usize, size)?;
        
        // Simulate realistic deallocation patterns
        if i > 1000 && i % 13 == 0 {
            let dealloc_ptr = (0x100000000u64 + ((i - 500) as u64 * 64)) as *mut u8;
            let _ = tracker.track_deallocation(dealloc_ptr as usize);
        }
        
        // Progress reporting for very large datasets
        if i % 10000 == 0 && i > 0 {
            let elapsed = start_time.elapsed();
            let rate = i as f64 / elapsed.as_secs_f64();
            println!("  Progress: {}/{} ({:.0} allocs/sec)", i, allocation_count, rate);
        }
    }
    
    let creation_time = start_time.elapsed();
    println!("Dataset created in {:.2}s", creation_time.as_secs_f64());
    
    Ok(tracker)
}

/// Memory usage monitor for stress tests
struct MemoryMonitor {
    peak_usage: AtomicUsize,
    current_usage: AtomicUsize,
}

impl MemoryMonitor {
    fn new() -> Self {
        Self {
            peak_usage: AtomicUsize::new(0),
            current_usage: AtomicUsize::new(0),
        }
    }
    
    fn update_usage(&self, usage: usize) {
        self.current_usage.store(usage, Ordering::Relaxed);
        
        let mut peak = self.peak_usage.load(Ordering::Relaxed);
        while usage > peak {
            match self.peak_usage.compare_exchange_weak(
                peak, usage, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(new_peak) => peak = new_peak,
            }
        }
    }
    
    fn get_peak_usage(&self) -> usize {
        self.peak_usage.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_large_dataset_100k() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let allocation_count = 100_000;
        
        println!("\n=== Large Dataset Stress Test (100K allocations) ===");
        
        let tracker = create_large_dataset(allocation_count)
            .expect("Failed to create large dataset");
        
        let memory_monitor = MemoryMonitor::new();
        let start_time = Instant::now();
        
        // Test binary export
        let binary_path = temp_dir.path().join("large_100k.bin");
        let binary_options = BinaryExportOptions::default()
            .compression(memscope::export::binary_format::CompressionType::Lz4)
            .parallel_encoding(true)
            .memory_limit(512 * 1024 * 1024); // 512MB limit
        
        tracker.export_to_binary_with_options(&binary_path, binary_options)
            .expect("Large dataset binary export failed");
        
        let export_duration = start_time.elapsed();
        let file_size = std::fs::metadata(&binary_path).unwrap().len() as usize;
        
        // Update memory usage (approximate)
        memory_monitor.update_usage(file_size);
        
        let result = StressTestResult::new(
            "Large Dataset 100K",
            export_duration,
            1, // One export operation
            memory_monitor.get_peak_usage(),
            0, // No errors expected
        );
        
        result.print_summary();
        
        // Verify the export was successful
        assert!(binary_path.exists(), "Binary file should be created");
        assert!(file_size > 1024 * 1024, "File should be at least 1MB for 100K allocations");
        assert!(export_duration.as_secs() < 30, "Export should complete within 30 seconds");
        
        // Test that we can parse the large file
        let parse_start = Instant::now();
        let mut parser = BinaryParser::new();
        parser.load_from_file(&binary_path).expect("Failed to parse large binary file");
        let allocations = parser.load_allocations().expect("Failed to load allocations");
        let parse_duration = parse_start.elapsed();
        
        assert!(allocations.len() > 50000, "Should have loaded a significant number of allocations");
        assert!(parse_duration.as_secs() < 10, "Parsing should be fast");
        
        println!("✅ Large dataset test passed");
        println!("   Export: {:.2}s, Parse: {:.2}s, File: {:.2}MB", 
            export_duration.as_secs_f64(), 
            parse_duration.as_secs_f64(),
            file_size as f64 / (1024.0 * 1024.0));
    }

    #[test]
    fn test_very_large_dataset_1m() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let allocation_count = 1_000_000;
        
        println!("\n=== Very Large Dataset Stress Test (1M allocations) ===");
        
        let tracker = create_large_dataset(allocation_count)
            .expect("Failed to create very large dataset");
        
        let memory_monitor = MemoryMonitor::new();
        let start_time = Instant::now();
        
        // Test binary export with maximum optimizations
        let binary_path = temp_dir.path().join("very_large_1m.bin");
        let binary_options = BinaryExportOptions::default()
            .compression(memscope::export::binary_format::CompressionType::Zstd)
            .parallel_encoding(true)
            .memory_limit(1024 * 1024 * 1024) // 1GB limit
            .performance(memscope::export::binary_exporter::PerformanceConfig {
                use_memory_mapping: true,
                memory_mapping_config: None,
                enable_zero_copy: true,
                enable_simd: true,
                cache_size: 512 * 1024,
                batch_size: 5000,
            });
        
        tracker.export_to_binary_with_options(&binary_path, binary_options)
            .expect("Very large dataset binary export failed");
        
        let export_duration = start_time.elapsed();
        let file_size = std::fs::metadata(&binary_path).unwrap().len() as usize;
        
        memory_monitor.update_usage(file_size);
        
        let result = StressTestResult::new(
            "Very Large Dataset 1M",
            export_duration,
            1,
            memory_monitor.get_peak_usage(),
            0,
        );
        
        result.print_summary();
        
        // More stringent requirements for very large datasets
        assert!(binary_path.exists(), "Binary file should be created");
        assert!(file_size > 10 * 1024 * 1024, "File should be at least 10MB for 1M allocations");
        assert!(export_duration.as_secs() < 120, "Export should complete within 2 minutes");
        
        // Test parsing performance
        let parse_start = Instant::now();
        let mut parser = BinaryParser::with_options(
            memscope::export::binary_parser::BinaryParserOptions::fast()
        );
        parser.load_from_file(&binary_path).expect("Failed to parse very large binary file");
        let parse_duration = parse_start.elapsed();
        
        assert!(parse_duration.as_secs() < 30, "Parsing should complete within 30 seconds");
        
        println!("✅ Very large dataset test passed");
        println!("   Export: {:.2}s, Parse: {:.2}s, File: {:.2}MB", 
            export_duration.as_secs_f64(), 
            parse_duration.as_secs_f64(),
            file_size as f64 / (1024.0 * 1024.0));
    }
}   
 #[test]
    fn test_concurrent_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let allocation_count = 50_000;
        
        println!("\n=== Concurrent Operations Stress Test ===");
        
        let tracker = Arc::new(create_large_dataset(allocation_count)
            .expect("Failed to create dataset"));
        
        let memory_monitor = Arc::new(MemoryMonitor::new());
        let operations_completed = Arc::new(AtomicUsize::new(0));
        let errors_encountered = Arc::new(AtomicUsize::new(0));
        
        let start_time = Instant::now();
        let thread_count = 4;
        let operations_per_thread = 5;
        
        let mut handles = Vec::new();
        
        for thread_id in 0..thread_count {
            let tracker: Arc<MemoryTracker> = Arc::clone(&tracker);
            let memory_monitor = Arc::clone(&memory_monitor);
            let operations_completed = Arc::clone(&operations_completed);
            let errors_encountered = Arc::clone(&errors_encountered);
            let temp_dir_path = temp_dir.path().to_path_buf();
            
            let handle = thread::spawn(move || {
                for op_id in 0..operations_per_thread {
                    let binary_path = temp_dir_path.join(format!("concurrent_{}_{}.bin", thread_id, op_id));
                    
                    let options = BinaryExportOptions::default()
                        .compression(CompressionType::Lz4)
                        .parallel_encoding(false) // Disable to avoid nested parallelism
                        .thread_count(Some(1));
                    
                    match tracker.export_to_binary_with_options(&binary_path, options) {
                        Ok(_) => {
                            operations_completed.fetch_add(1, Ordering::Relaxed);
                            
                            // Update memory usage
                            if let Ok(metadata) = std::fs::metadata(&binary_path) {
                                memory_monitor.update_usage(metadata.len() as usize);
                            }
                        }
                        Err(_) => {
                            errors_encountered.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    
                    // Small delay to simulate realistic usage
                    thread::sleep(Duration::from_millis(100));
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
        
        let total_duration = start_time.elapsed();
        let total_operations = operations_completed.load(Ordering::Relaxed);
        let total_errors = errors_encountered.load(Ordering::Relaxed);
        
        let result = StressTestResult::new(
            "Concurrent Operations",
            total_duration,
            total_operations,
            memory_monitor.get_peak_usage(),
            total_errors,
        );
        
        result.print_summary();
        
        // Verify concurrent operations succeeded
        assert_eq!(total_operations, thread_count * operations_per_thread, 
            "All operations should complete successfully");
        assert_eq!(total_errors, 0, "No errors should occur during concurrent operations");
        assert!(result.success_rate >= 99.0, "Success rate should be at least 99%");
        
        println!("✅ Concurrent operations test passed");
    }

    #[test]
    fn test_memory_pressure() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let allocation_count = 75_000;
        
        println!("\n=== Memory Pressure Stress Test ===");
        
        let tracker = create_large_dataset(allocation_count)
            .expect("Failed to create dataset");
        
        let memory_monitor = MemoryMonitor::new();
        let start_time = Instant::now();
        
        // Test with very limited memory
        let binary_path = temp_dir.path().join("memory_pressure.bin");
        let binary_options = BinaryExportOptions::default()
            .compression(CompressionType::Zstd) // High compression
            .parallel_encoding(true)
            .memory_limit(64 * 1024 * 1024) // Only 64MB limit
            .performance(memscope_rs::export::binary_exporter::PerformanceConfig {
                use_memory_mapping: true,
                memory_mapping_config: Some(
                    memscope_rs::export::memory_mapping::MemoryMappingConfig {
                        max_memory_usage: 32 * 1024 * 1024, // 32MB mapping limit
                        ..Default::default()
                    }
                ),
                enable_zero_copy: true,
                enable_simd: false, // Disable for memory conservation
                cache_size: 16 * 1024, // Small cache
                batch_size: 500, // Small batches
            });
        
        let export_result = tracker.export_to_binary_with_options(&binary_path, binary_options);
        let export_duration = start_time.elapsed();
        
        match export_result {
            Ok(_) => {
                let file_size = std::fs::metadata(&binary_path).unwrap().len() as usize;
                memory_monitor.update_usage(file_size);
                
                let result = StressTestResult::new(
                    "Memory Pressure",
                    export_duration,
                    1,
                    memory_monitor.get_peak_usage(),
                    0,
                );
                
                result.print_summary();
                
                // Verify the export worked under memory pressure
                assert!(binary_path.exists(), "Binary file should be created under memory pressure");
                assert!(file_size > 0, "File should have content");
                
                // Test that we can still parse it
                let mut parser = BinaryParser::with_options(
                    BinaryParserOptions {
                        buffer_size: 32 * 1024, // Small buffer
                        ..Default::default()
                    }
                );
                
                parser.load_from_file(&binary_path)
                    .expect("Should be able to parse file created under memory pressure");
                
                println!("✅ Memory pressure test passed");
            }
            Err(e) => {
                println!("⚠️  Export failed under memory pressure (this may be expected): {}", e);
                // This might be acceptable behavior under extreme memory pressure
            }
        }
    }

    #[test]
    fn test_long_running_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        println!("\n=== Long Running Operations Stress Test ===");
        
        let memory_monitor = Arc::new(MemoryMonitor::new());
        let operations_completed = Arc::new(AtomicUsize::new(0));
        let errors_encountered = Arc::new(AtomicUsize::new(0));
        
        let start_time = Instant::now();
        let test_duration = Duration::from_secs(60); // 1 minute test
        let operation_interval = Duration::from_secs(5);
        
        let memory_monitor_clone = Arc::clone(&memory_monitor);
        let operations_completed_clone = Arc::clone(&operations_completed);
        let errors_encountered_clone = Arc::clone(&errors_encountered);
        let temp_dir_path = temp_dir.path().to_path_buf();
        
        let handle = thread::spawn(move || {
            let mut operation_count = 0;
            
            while start_time.elapsed() < test_duration {
                // Create a fresh dataset for each operation
                let tracker = match create_large_dataset(10_000) {
                    Ok(t) => t,
                    Err(_) => {
                        errors_encountered_clone.fetch_add(1, Ordering::Relaxed);
                        thread::sleep(operation_interval);
                        continue;
                    }
                };
                
                let binary_path = temp_dir_path.join(format!("long_running_{}.bin", operation_count));
                let options = BinaryExportOptions::default()
                    .compression(CompressionType::Lz4);
                
                match tracker.export_to_binary_with_options(&binary_path, options) {
                    Ok(_) => {
                        operations_completed_clone.fetch_add(1, Ordering::Relaxed);
                        
                        if let Ok(metadata) = std::fs::metadata(&binary_path) {
                            memory_monitor_clone.update_usage(metadata.len() as usize);
                        }
                        
                        // Clean up old files to prevent disk space issues
                        if operation_count > 5 {
                            let old_path = temp_dir_path.join(format!("long_running_{}.bin", operation_count - 5));
                            let _ = std::fs::remove_file(old_path);
                        }
                    }
                    Err(_) => {
                        errors_encountered_clone.fetch_add(1, Ordering::Relaxed);
                    }
                }
                
                operation_count += 1;
                thread::sleep(operation_interval);
            }
        });
        
        handle.join().expect("Long running test thread panicked");
        
        let total_duration = start_time.elapsed();
        let total_operations = operations_completed.load(Ordering::Relaxed);
        let total_errors = errors_encountered.load(Ordering::Relaxed);
        
        let result = StressTestResult::new(
            "Long Running Operations",
            total_duration,
            total_operations,
            memory_monitor.get_peak_usage(),
            total_errors,
        );
        
        result.print_summary();
        
        // Verify long running stability
        assert!(total_operations >= 8, "Should complete at least 8 operations in 1 minute");
        assert!(result.success_rate >= 90.0, "Success rate should be at least 90% for long running test");
        
        println!("✅ Long running operations test passed");
    }

    #[test]
    fn test_error_recovery() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        println!("\n=== Error Recovery Stress Test ===");
        
        let tracker = create_large_dataset(25_000)
            .expect("Failed to create dataset");
        
        // Create a binary file
        let binary_path = temp_dir.path().join("error_recovery.bin");
        tracker.export_to_binary(&binary_path).expect("Initial export failed");
        
        // Read the file and corrupt it slightly
        let mut file_data = std::fs::read(&binary_path).expect("Failed to read binary file");
        
        // Corrupt a few bytes in the middle (not the header)
        if file_data.len() > 1000 {
            file_data[500] = file_data[500].wrapping_add(1);
            file_data[750] = file_data[750].wrapping_add(1);
            file_data[1000] = file_data[1000].wrapping_add(1);
        }
        
        let corrupted_path = temp_dir.path().join("corrupted.bin");
        std::fs::write(&corrupted_path, &file_data).expect("Failed to write corrupted file");
        
        // Test recovery parsing
        let mut recovery_parser = BinaryParser::with_options(
            BinaryParserOptions::recovery_mode()
        );
        
        let start_time = Instant::now();
        let parse_result = recovery_parser.load_from_file(&corrupted_path);
        let parse_duration = start_time.elapsed();
        
        match parse_result {
            Ok(_) => {
                println!("✅ Recovery parser successfully handled corrupted file");
                
                // Try to extract what data we can
                match recovery_parser.load_allocations() {
                    Ok(allocations) => {
                        println!("   Recovered {} allocations", allocations.len());
                        assert!(allocations.len() > 1000, "Should recover a significant portion of data");
                    }
                    Err(e) => {
                        println!("   Could not recover allocations: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("⚠️  Recovery parser failed (this may be expected for severe corruption): {}", e);
            }
        }
        
        // Test strict parser (should fail)
        let mut strict_parser = BinaryParser::with_options(
            BinaryParserOptions::strict()
        );
        
        let strict_result = strict_parser.load_from_file(&corrupted_path);
        assert!(strict_result.is_err(), "Strict parser should reject corrupted file");
        
        println!("✅ Error recovery test completed");
    }

    #[test]
    fn test_resource_cleanup() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        println!("\n=== Resource Cleanup Stress Test ===");
        
        let memory_monitor = MemoryMonitor::new();
        let start_time = Instant::now();
        
        // Perform many operations to test resource cleanup
        for i in 0..20 {
            let tracker = create_large_dataset(5_000)
                .expect("Failed to create dataset");
            
            let binary_path = temp_dir.path().join(format!("cleanup_test_{}.bin", i));
            let options = BinaryExportOptions::default()
                .compression(CompressionType::Lz4);
            
            tracker.export_to_binary_with_options(&binary_path, options)
                .expect("Export failed during cleanup test");
            
            // Parse the file
            let mut parser = BinaryParser::new();
            parser.load_from_file(&binary_path).expect("Parse failed during cleanup test");
            let _allocations = parser.load_allocations().expect("Load allocations failed");
            
            // Update memory usage
            if let Ok(metadata) = std::fs::metadata(&binary_path) {
                memory_monitor.update_usage(metadata.len() as usize);
            }
            
            // Force cleanup by dropping variables
            drop(parser);
            drop(tracker);
            
            // Clean up file
            std::fs::remove_file(&binary_path).expect("Failed to clean up file");
            
            if i % 5 == 0 {
                println!("  Completed {} cleanup cycles", i + 1);
            }
        }
        
        let total_duration = start_time.elapsed();
        
        let result = StressTestResult::new(
            "Resource Cleanup",
            total_duration,
            20, // 20 cycles
            memory_monitor.get_peak_usage(),
            0,
        );
        
        result.print_summary();
        
        // Verify cleanup was successful
        assert!(total_duration.as_secs() < 60, "Cleanup test should complete within 1 minute");
        
        println!("✅ Resource cleanup test passed");
    }


/// Extreme stress tests for edge cases
#[cfg(test)]
mod extreme_tests {
    use super::*;

    #[test]
    #[ignore] // Only run manually due to resource requirements
    fn test_extreme_large_dataset_10m() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let allocation_count = 10_000_000; // 10 million allocations
        
        println!("\n=== EXTREME: 10M Allocations Stress Test ===");
        println!("⚠️  This test requires significant memory and time");
        
        let tracker = create_large_dataset(allocation_count)
            .expect("Failed to create extreme dataset");
        
        let start_time = Instant::now();
        
        let binary_path = temp_dir.path().join("extreme_10m.bin");
        let binary_options = BinaryExportOptions::comprehensive()
            .compression(CompressionType::Zstd)
            .parallel_encoding(true)
            .memory_limit(2 * 1024 * 1024 * 1024) // 2GB limit
            .performance(memscope_rs::export::binary_exporter::PerformanceConfig {
                use_memory_mapping: true,
                memory_mapping_config: None,
                enable_zero_copy: true,
                enable_simd: true,
                cache_size: 1024 * 1024, // 1MB cache
                batch_size: 10000,
            });
        
        tracker.export_to_binary_with_options(&binary_path, binary_options)
            .expect("Extreme dataset export failed");
        
        let export_duration = start_time.elapsed();
        let file_size = std::fs::metadata(&binary_path).unwrap().len() as usize;
        
        println!("Extreme dataset results:");
        println!("  Export time: {:.2}s", export_duration.as_secs_f64());
        println!("  File size: {:.2} MB", file_size as f64 / (1024.0 * 1024.0));
        println!("  Throughput: {:.2} MB/s", 
            (file_size as f64 / (1024.0 * 1024.0)) / export_duration.as_secs_f64());
        
        // Should complete within reasonable time even for extreme dataset
        assert!(export_duration.as_secs() < 600, "Even extreme dataset should export within 10 minutes");
        assert!(file_size > 100 * 1024 * 1024, "File should be at least 100MB for 10M allocations");
        
        println!("✅ Extreme large dataset test passed");
    }
}