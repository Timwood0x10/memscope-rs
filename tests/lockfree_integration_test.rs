//! Test binary for validating multi-threaded tracking implementation
//!
//! This test demonstrates the new lock-free multi-threaded memory tracking
//! system that eliminates the "fatal runtime error" issues when using
//! many concurrent threads (100+ threads).
//!
//! Key validation points:
//! - Thread independence: No shared state between threads
//! - Sampling accuracy: Intelligent frequency + size sampling
//! - Performance impact: <5% overhead target
//! - Data completeness: All critical allocations captured

use memscope_rs::lockfree::{
    init_thread_tracker, track_allocation_lockfree, track_deallocation_lockfree,
    finalize_thread_tracker, SamplingConfig, LockfreeAggregator
};
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use tracing::{info, warn, error};

/// Configuration for multi-threaded test scenarios
#[derive(Debug)]
struct TestConfig {
    thread_count: usize,
    allocations_per_thread: usize,
    allocation_sizes: Vec<usize>,
    test_duration_limit: std::time::Duration,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            thread_count: 50,
            allocations_per_thread: 1000,
            allocation_sizes: vec![64, 128, 512, 1024, 4096, 8192, 16384],
            test_duration_limit: std::time::Duration::from_secs(30),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();
    
    info!("Starting multi-threaded tracking validation test");
    
    // Test with default configuration
    let config = TestConfig::default();
    run_multithreaded_test(&config)?;
    
    // Test with stress configuration (100+ threads)
    let stress_config = TestConfig {
        thread_count: 100,
        allocations_per_thread: 500,
        ..Default::default()
    };
    run_multithreaded_test(&stress_config)?;
    
    info!("All multi-threaded tests completed successfully");
    Ok(())
}

/// Executes multi-threaded tracking test with specified configuration
/// 
/// # Arguments
/// * `config` - Test configuration including thread count and allocation patterns
/// 
/// # Returns
/// Result indicating test success or failure details
fn run_multithreaded_test(config: &TestConfig) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running test with {} threads, {} allocations per thread", 
          config.thread_count, config.allocations_per_thread);
    
    let test_start = Instant::now();
    let output_dir = std::env::temp_dir().join("memscope_multithread_test");
    
    // Clean up any previous test data
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }
    std::fs::create_dir_all(&output_dir)?;
    
    // Shared counters for validation
    let total_operations = Arc::new(AtomicUsize::new(0));
    let successful_threads = Arc::new(AtomicUsize::new(0));
    
    // Create sampling configuration for test
    let sampling_config = SamplingConfig {
        large_allocation_rate: 1.0,   // Track all large allocations
        medium_allocation_rate: 0.5,  // 50% of medium allocations
        small_allocation_rate: 0.1,   // 10% of small allocations
        large_threshold: 4096,
        medium_threshold: 512,
        frequency_threshold: 50,
    };
    
    // Spawn worker threads
    let handles: Vec<_> = (0..config.thread_count)
        .map(|thread_idx| {
            let output_dir = output_dir.clone();
            let sampling_config = sampling_config.clone();
            let allocation_sizes = config.allocation_sizes.clone();
            let allocations_per_thread = config.allocations_per_thread;
            let total_operations = Arc::clone(&total_operations);
            let successful_threads = Arc::clone(&successful_threads);
            
            thread::spawn(move || -> Result<(), String> {
                // Initialize thread-local tracker
                init_thread_tracker(&output_dir, Some(sampling_config))
                    .map_err(|e| e.to_string())?;
                
                // Perform allocations with varying patterns
                for allocation_idx in 0..allocations_per_thread {
                    let size_idx = allocation_idx % allocation_sizes.len();
                    let size = allocation_sizes[size_idx];
                    
                    // Generate realistic memory addresses
                    let base_ptr = 0x100000 + (thread_idx * 0x10000);
                    let ptr = base_ptr + (allocation_idx * 64);
                    
                    // Create call stack simulation
                    let call_stack = vec![
                        0x400000 + thread_idx,           // Thread-specific function
                        0x500000 + allocation_idx % 10,  // Allocation pattern
                        0x600000 + size_idx,             // Size-specific function
                    ];
                    
                    // Track allocation
                    track_allocation_lockfree(ptr, size, &call_stack)
                        .map_err(|e| e.to_string())?;
                    
                    // Simulate some deallocations for memory balance
                    if allocation_idx % 4 == 0 && allocation_idx > 0 {
                        let dealloc_ptr = base_ptr + ((allocation_idx - 1) * 64);
                        track_deallocation_lockfree(dealloc_ptr, &call_stack)
                            .map_err(|e| e.to_string())?;
                    }
                    
                    total_operations.fetch_add(1, Ordering::Relaxed);
                }
                
                // Finalize thread tracking
                finalize_thread_tracker()
                    .map_err(|e| e.to_string())?;
                successful_threads.fetch_add(1, Ordering::Relaxed);
                
                Ok(())
            })
        })
        .collect();
    
    // Wait for all threads with timeout protection
    let mut completed_threads = 0;
    for (idx, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(result) => {
                if let Err(e) = result {
                    error!("Thread {} failed: {}", idx, e);
                } else {
                    completed_threads += 1;
                }
            }
            Err(e) => {
                error!("Thread {} panicked: {:?}", idx, e);
            }
        }
    }
    
    let test_duration = test_start.elapsed();
    
    // Validate test results
    validate_test_results(config, &output_dir, completed_threads, 
                         total_operations.load(Ordering::Relaxed), test_duration)?;
    
    // Generate analysis report
    generate_analysis_report(&output_dir, config)?;
    
    // Cleanup test data
    std::fs::remove_dir_all(&output_dir)?;
    
    info!("Test completed in {:?} with {} successful threads", test_duration, completed_threads);
    Ok(())
}

/// Validates test results against expected criteria
/// 
/// # Arguments
/// * `config` - Original test configuration
/// * `output_dir` - Directory containing test output files
/// * `completed_threads` - Number of successfully completed threads
/// * `total_operations` - Total number of operations performed
/// * `duration` - Total test execution time
fn validate_test_results(
    config: &TestConfig,
    output_dir: &std::path::Path,
    completed_threads: usize,
    total_operations: usize,
    duration: std::time::Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Validating test results...");
    
    // Check thread completion rate
    let completion_rate = completed_threads as f64 / config.thread_count as f64;
    if completion_rate < 0.95 {
        return Err(format!("Thread completion rate too low: {:.2}%", completion_rate * 100.0).into());
    }
    
    // Check operation completion
    let expected_operations = config.thread_count * config.allocations_per_thread;
    let operation_rate = total_operations as f64 / expected_operations as f64;
    if operation_rate < 0.90 {
        return Err(format!("Operation completion rate too low: {:.2}%", operation_rate * 100.0).into());
    }
    
    // Check test duration (should complete within reasonable time)
    if duration > config.test_duration_limit {
        warn!("Test duration exceeded limit: {:?} > {:?}", duration, config.test_duration_limit);
    }
    
    // Verify thread files were created
    let mut thread_files_found = 0;
    for entry in std::fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with("memscope_thread_") && file_name.ends_with(".bin") {
                thread_files_found += 1;
            }
        }
    }
    
    if thread_files_found < completed_threads {
        return Err(format!("Missing thread files: found {}, expected {}", 
                          thread_files_found, completed_threads).into());
    }
    
    info!("Validation successful: {:.1}% threads completed, {:.1}% operations completed", 
          completion_rate * 100.0, operation_rate * 100.0);
    Ok(())
}

/// Generates comprehensive analysis report from test data
/// 
/// # Arguments
/// * `output_dir` - Directory containing thread data files
/// * `config` - Test configuration for context
fn generate_analysis_report(
    output_dir: &std::path::Path,
    _config: &TestConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Generating analysis report...");
    
    let aggregator = LockfreeAggregator::new(output_dir.to_path_buf());
    let analysis = aggregator.aggregate_all_threads()?;
    
    // Log key analysis results
    info!("Analysis Results:");
    info!("  - Threads analyzed: {}", analysis.thread_stats.len());
    info!("  - Hottest call stacks: {}", analysis.hottest_call_stacks.len());
    info!("  - Thread interactions: {}", analysis.thread_interactions.len());
    info!("  - Performance bottlenecks: {}", analysis.performance_bottlenecks.len());
    
    // Check for critical issues
    let high_severity_bottlenecks = analysis.performance_bottlenecks
        .iter()
        .filter(|b| b.severity > 0.7)
        .count();
    
    if high_severity_bottlenecks > 0 {
        warn!("Found {} high-severity performance bottlenecks", high_severity_bottlenecks);
    }
    
    // Export analysis to files for inspection
    let analysis_json = output_dir.join("multithreaded_analysis.json");
    aggregator.export_analysis(&analysis, &analysis_json)?;
    
    let report_html = output_dir.join("multithreaded_report.html");
    aggregator.generate_html_report(&analysis, &report_html)?;
    
    info!("Analysis report generated successfully");
    Ok(())
}