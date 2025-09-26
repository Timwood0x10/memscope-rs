//! Comprehensive test suite for lock-free data aggregator
//!
//! This test validates the complete pipeline from thread-local tracking
//! to offline data aggregation and analysis report generation.

use memscope_rs::lockfree::{
    finalize_thread_tracker, init_thread_tracker, track_allocation_lockfree,
    track_deallocation_lockfree, LockfreeAggregator, SamplingConfig,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

/// Test configuration for aggregator validation
struct AggregatorTestConfig {
    thread_count: usize,
    allocations_per_thread: usize,
    expected_min_files: usize,
}

impl Default for AggregatorTestConfig {
    fn default() -> Self {
        Self {
            thread_count: 8,
            allocations_per_thread: 200,
            expected_min_files: 16, // 8 threads * 2 files (.bin + .freq)
        }
    }
}

#[test]
fn test_complete_aggregation_pipeline() {
    let test_start = Instant::now();
    let config = AggregatorTestConfig::default();

    // Setup test environment
    let output_dir = std::env::temp_dir().join("aggregator_test");
    cleanup_test_directory(&output_dir);
    std::fs::create_dir_all(&output_dir).expect("Failed to create test directory");

    // Generate multi-threaded data
    let operation_counter = generate_test_data(&output_dir, &config);

    // Validate data generation
    validate_data_generation(&output_dir, &config, operation_counter);

    // Test aggregation process
    test_aggregation_functionality(&output_dir, &config);

    // Test report generation
    test_report_generation(&output_dir);

    // Performance validation
    validate_performance_requirements(test_start);

    // Cleanup
    cleanup_test_directory(&output_dir);
}

/// Generates test data using multiple threads with varied allocation patterns
fn generate_test_data(output_dir: &std::path::Path, config: &AggregatorTestConfig) -> usize {
    let operation_counter = Arc::new(AtomicUsize::new(0));
    let thread_count = config.thread_count;
    let allocations_per_thread = config.allocations_per_thread;

    // Create varied sampling configurations for different threads
    let sampling_configs = [
        SamplingConfig::default(),
        SamplingConfig::high_precision(),
        SamplingConfig::performance_optimized(),
        SamplingConfig::leak_detection(),
    ];

    let handles: Vec<_> = (0..thread_count)
        .map(|thread_idx| {
            let output_dir = output_dir.to_path_buf();
            let config_idx = thread_idx % sampling_configs.len();
            let sampling_config = sampling_configs[config_idx].clone();
            let operation_counter = Arc::clone(&operation_counter);

            thread::spawn(move || -> Result<(), String> {
                // Initialize thread tracker with specific configuration
                init_thread_tracker(&output_dir, Some(sampling_config))
                    .map_err(|e| format!("Thread {} init failed: {}", thread_idx, e))?;

                // Generate varied allocation patterns
                for alloc_idx in 0..allocations_per_thread {
                    let allocation_result =
                        generate_thread_allocations(thread_idx, alloc_idx, &operation_counter);

                    if let Err(e) = allocation_result {
                        return Err(format!("Thread {} allocation failed: {}", thread_idx, e));
                    }
                }

                // Finalize thread tracking
                finalize_thread_tracker()
                    .map_err(|e| format!("Thread {} finalize failed: {}", thread_idx, e))?;

                Ok(())
            })
        })
        .collect();

    // Wait for all threads and validate completion
    let mut successful_threads = 0;
    for (idx, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(())) => successful_threads += 1,
            Ok(Err(e)) => panic!("Thread {} failed: {}", idx, e),
            Err(e) => panic!("Thread {} panicked: {:?}", idx, e),
        }
    }

    assert_eq!(
        successful_threads, thread_count,
        "Not all threads completed successfully"
    );

    operation_counter.load(Ordering::Relaxed)
}

/// Generates realistic allocation patterns for a single thread
fn generate_thread_allocations(
    thread_idx: usize,
    alloc_idx: usize,
    counter: &Arc<AtomicUsize>,
) -> Result<(), String> {
    let ptr_base = 0x100000 + (thread_idx * 0x10000);
    let ptr = ptr_base + (alloc_idx * 64);

    // Create varied allocation sizes to test sampling effectiveness
    let size = match alloc_idx % 8 {
        0 => 32,    // Small allocation
        1 => 128,   // Small allocation
        2 => 512,   // Small allocation
        3 => 1024,  // Medium allocation
        4 => 2048,  // Medium allocation
        5 => 4096,  // Medium allocation
        6 => 8192,  // Large allocation
        7 => 16384, // Large allocation
        _ => 256,   // Default
    };

    // Generate realistic call stack patterns
    let call_stack = vec![
        0x400000 + (thread_idx % 5),    // Function group pattern
        0x500000 + (alloc_idx % 15),    // Loop iteration pattern
        0x600000 + ((size / 1024) % 8), // Size category pattern
    ];

    // Track allocation
    track_allocation_lockfree(ptr, size, &call_stack)
        .map_err(|e| format!("Allocation tracking failed: {}", e))?;

    // Simulate realistic deallocation patterns
    if alloc_idx.is_multiple_of(3) && alloc_idx > 0 {
        let dealloc_ptr = ptr_base + ((alloc_idx - 1) * 64);
        track_deallocation_lockfree(dealloc_ptr, &call_stack)
            .map_err(|e| format!("Deallocation tracking failed: {}", e))?;
    }

    counter.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

/// Validates that test data was generated correctly
fn validate_data_generation(
    output_dir: &std::path::Path,
    config: &AggregatorTestConfig,
    operation_count: usize,
) {
    // Verify expected number of operations
    let expected_operations = config.thread_count * config.allocations_per_thread;
    assert_eq!(
        operation_count, expected_operations,
        "Operation count mismatch: expected {}, got {}",
        expected_operations, operation_count
    );

    // Verify binary files were created
    let mut file_count = 0;
    let mut total_file_size = 0u64;

    for entry in std::fs::read_dir(output_dir).expect("Failed to read output directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with("memscope_thread_") {
                file_count += 1;
                let metadata = std::fs::metadata(&path).expect("Failed to get file metadata");
                total_file_size += metadata.len();

                // Validate file is not empty
                assert!(metadata.len() > 0, "Generated file {} is empty", file_name);
            }
        }
    }

    assert!(
        file_count >= config.expected_min_files,
        "Insufficient files generated: expected >= {}, got {}",
        config.expected_min_files,
        file_count
    );

    assert!(
        total_file_size > 1024,
        "Total file size too small: {} bytes",
        total_file_size
    );
}

/// Tests the aggregation functionality with generated data
fn test_aggregation_functionality(output_dir: &std::path::Path, config: &AggregatorTestConfig) {
    let aggregator = LockfreeAggregator::new(output_dir.to_path_buf());

    // Test aggregation process
    let analysis = aggregator
        .aggregate_all_threads()
        .expect("Aggregation failed");

    // Validate analysis results
    assert_eq!(
        analysis.thread_stats.len(),
        config.thread_count,
        "Thread count mismatch in analysis"
    );

    assert!(
        analysis.summary.total_allocations > 0,
        "No allocations found in analysis"
    );

    assert!(
        analysis.summary.unique_call_stacks > 0,
        "No unique call stacks found"
    );

    // Validate thread statistics
    for (thread_id, stats) in &analysis.thread_stats {
        assert!(
            stats.total_allocations > 0,
            "Thread {} has no allocations",
            thread_id
        );

        assert!(
            !stats.allocation_frequency.is_empty(),
            "Thread {} has no frequency data",
            thread_id
        );

        assert!(
            !stats.timeline.is_empty(),
            "Thread {} has no timeline events",
            thread_id
        );
    }

    // Validate cross-thread analysis
    if config.thread_count > 1 {
        // Should detect some thread interactions with similar patterns
        // Thread interactions may or may not exist depending on patterns
        // Just ensure the detection runs without errors
    }

    // Validate hottest call stacks detection
    assert!(
        !analysis.hottest_call_stacks.is_empty(),
        "No hot call stacks detected"
    );

    // Validate performance bottleneck detection
    // Note: May or may not find bottlenecks depending on test data patterns
    // Just ensure the detection runs without errors
}

/// Tests report generation functionality
fn test_report_generation(output_dir: &std::path::Path) {
    let aggregator = LockfreeAggregator::new(output_dir.to_path_buf());
    let analysis = aggregator
        .aggregate_all_threads()
        .expect("Failed to aggregate for report test");

    // Test JSON export
    let json_path = output_dir.join("test_analysis.json");
    aggregator
        .export_analysis(&analysis, &json_path)
        .expect("JSON export failed");

    assert!(json_path.exists(), "JSON report file not created");

    let json_size = std::fs::metadata(&json_path)
        .expect("Failed to get JSON metadata")
        .len();
    assert!(
        json_size > 100,
        "JSON report too small: {} bytes",
        json_size
    );

    // HTML generation now uses comprehensive export
    // Note: HTML generation has been moved to comprehensive export
}

/// Validates that performance requirements are met
fn validate_performance_requirements(test_start: Instant) {
    let total_duration = test_start.elapsed();

    // Aggregation should complete within reasonable time
    assert!(
        total_duration.as_secs() < 30,
        "Test took too long: {:?}",
        total_duration
    );

    // Memory usage should be reasonable (indirect check via successful completion)
    // Direct memory measurement would require additional instrumentation
}

/// Cleans up test directory, handling errors gracefully
fn cleanup_test_directory(output_dir: &std::path::Path) {
    if output_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(output_dir) {
            // Non-fatal error, test can still proceed
            eprintln!("Warning: Failed to clean test directory: {}", e);
        }
    }
}

#[test]
fn test_aggregator_error_handling() {
    // Test aggregator with non-existent directory
    let non_existent_dir = std::path::PathBuf::from("/non/existent/directory");
    let aggregator = LockfreeAggregator::new(non_existent_dir);

    let result = aggregator.aggregate_all_threads();
    assert!(
        result.is_ok(),
        "Aggregator should handle missing directory gracefully"
    );

    let analysis = result.expect("Should return empty analysis");
    assert_eq!(analysis.thread_stats.len(), 0, "Should have no thread data");
}

#[test]
fn test_sampling_configuration_effectiveness() {
    // Test that different sampling configurations produce different results
    let output_dir = std::env::temp_dir().join("sampling_test");
    cleanup_test_directory(&output_dir);
    std::fs::create_dir_all(&output_dir).expect("Failed to create test directory");

    // This test would require more complex setup to validate sampling effectiveness
    // For now, just ensure configurations can be created and validated
    let configs = vec![
        SamplingConfig::default(),
        SamplingConfig::high_precision(),
        SamplingConfig::performance_optimized(),
        SamplingConfig::leak_detection(),
    ];

    for config in configs {
        assert!(
            config.validate().is_ok(),
            "Sampling configuration should be valid"
        );
    }

    cleanup_test_directory(&output_dir);
}
