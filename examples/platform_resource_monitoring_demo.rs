//! Platform-specific resource monitoring demonstration
//! 
//! Shows how to use the new CPU, GPU, and IO monitoring capabilities
//! on macOS and Linux platforms in multi-threaded environments

use memscope_rs::lockfree::{
    PlatformResourceCollector,
    IntegratedProfilingSession,
    comprehensive_profile_execution,
};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Platform Resource Monitoring Demo ===");
    println!("Monitoring CPU, GPU, and IO resources across platforms");
    
    // Test basic platform resource collection
    test_basic_resource_collection()?;
    
    // Test comprehensive profiling with memory tracking
    test_comprehensive_profiling()?;
    
    // Test multi-threaded resource monitoring
    test_multithreaded_monitoring()?;
    
    println!("\n=== Demo completed successfully ===");
    Ok(())
}

/// Test basic platform resource collection capabilities
fn test_basic_resource_collection() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Testing Basic Resource Collection ---");
    
    match PlatformResourceCollector::new() {
        Ok(mut collector) => {
            println!("✓ Platform resource collector initialized successfully");
            
            // Collect metrics for a few seconds
            for i in 0..5 {
                match collector.collect_metrics() {
                    Ok(metrics) => {
                        println!("Sample {}: CPU: {:.1}%, Cores: {}, GPU: {}", 
                                i + 1,
                                metrics.cpu_metrics.overall_usage_percent,
                                metrics.cpu_metrics.per_core_usage.len(),
                                if metrics.gpu_metrics.is_some() { "Available" } else { "N/A" });
                    }
                    Err(e) => {
                        println!("⚠️  Warning: Failed to collect metrics: {}", e);
                    }
                }
                
                // Generate some CPU load for demonstration
                cpu_intensive_work(Duration::from_millis(100));
                
                thread::sleep(Duration::from_millis(500));
            }
            
            let optimal_interval = collector.get_optimal_collection_interval();
            println!("✓ Optimal collection interval: {:?}", optimal_interval);
        }
        Err(e) => {
            println!("⚠️  Platform resource collection not available: {}", e);
            println!("This is expected on unsupported platforms");
        }
    }
    
    Ok(())
}

/// Test comprehensive profiling that combines memory and resource tracking
fn test_comprehensive_profiling() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Testing Comprehensive Profiling ---");
    
    let output_dir = std::env::temp_dir().join("memscope_platform_demo");
    std::fs::create_dir_all(&output_dir)?;
    
    let result = comprehensive_profile_execution(&output_dir, || {
        // Simulate workload with memory allocations and system resource usage
        simulate_comprehensive_workload()
    });
    
    match result {
        Ok((work_result, analysis)) => {
            println!("✓ Comprehensive profiling completed");
            println!("  Work result: {} operations completed", work_result);
            println!("  Memory allocations: {}", analysis.memory_analysis.summary.total_allocations);
            println!("  Memory deallocations: {}", analysis.memory_analysis.summary.total_deallocations);
            println!("  Peak memory usage: {} bytes", analysis.memory_analysis.summary.peak_memory_usage);
            println!("  Resource timeline samples: {}", analysis.resource_timeline.len());
            
            // Display correlation insights
            println!("  Performance Insights:");
            println!("    Primary bottleneck: {:?}", analysis.performance_insights.primary_bottleneck);
            println!("    CPU efficiency: {:.1}%", analysis.performance_insights.cpu_efficiency_score);
            println!("    Memory efficiency: {:.1}%", analysis.performance_insights.memory_efficiency_score);
            println!("    IO efficiency: {:.1}%", analysis.performance_insights.io_efficiency_score);
            
            if !analysis.performance_insights.recommendations.is_empty() {
                println!("  Recommendations:");
                for (i, rec) in analysis.performance_insights.recommendations.iter().enumerate() {
                    println!("    {}. {}", i + 1, rec);
                }
            }
            
            // Clean up
            let _ = std::fs::remove_dir_all(&output_dir);
        }
        Err(e) => {
            println!("⚠️  Comprehensive profiling failed: {}", e);
            println!("This may be expected on platforms without full resource monitoring support");
        }
    }
    
    Ok(())
}

/// Test multi-threaded resource monitoring
fn test_multithreaded_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Testing Multi-threaded Monitoring ---");
    
    let output_dir = std::env::temp_dir().join("memscope_multithread_demo");
    std::fs::create_dir_all(&output_dir)?;
    
    match IntegratedProfilingSession::new(&output_dir) {
        Ok(mut session) => {
            println!("✓ Integrated profiling session created");
            
            // Start profiling
            if let Err(e) = session.start_profiling() {
                println!("⚠️  Failed to start profiling: {}", e);
                return Ok(());
            }
            
            println!("✓ Profiling started, launching worker threads...");
            
            // Create multiple worker threads with different resource patterns
            let stop_signal = Arc::new(AtomicBool::new(false));
            let mut handles = Vec::new();
            
            // CPU-intensive thread
            let stop = stop_signal.clone();
            handles.push(thread::Builder::new()
                .name("cpu_worker".to_string())
                .spawn(move || cpu_intensive_thread(stop))?);
            
            // Memory-intensive thread
            let stop = stop_signal.clone();
            handles.push(thread::Builder::new()
                .name("memory_worker".to_string())
                .spawn(move || memory_intensive_thread(stop))?);
            
            // IO-intensive thread
            let stop = stop_signal.clone();
            handles.push(thread::Builder::new()
                .name("io_worker".to_string())
                .spawn(move || io_intensive_thread(stop))?);
            
            // Let threads run for a few seconds
            thread::sleep(Duration::from_secs(3));
            
            // Stop all threads
            stop_signal.store(true, Ordering::Relaxed);
            for handle in handles {
                let _ = handle.join();
            }
            
            println!("✓ Worker threads completed, analyzing results...");
            
            // Stop profiling and get analysis
            match session.stop_profiling_and_analyze() {
                Ok(analysis) => {
                    println!("✓ Multi-threaded analysis completed");
                    println!("  Total threads analyzed: {}", analysis.memory_analysis.thread_stats.len());
                    println!("  Resource samples collected: {}", analysis.resource_timeline.len());
                    
                    // Show thread performance ranking
                    if !analysis.performance_insights.thread_performance_ranking.is_empty() {
                        println!("  Thread Performance Ranking:");
                        for (i, thread_perf) in analysis.performance_insights.thread_performance_ranking.iter().take(3).enumerate() {
                            println!("    {}. Thread {} - Efficiency: {:.1}%", 
                                    i + 1, 
                                    thread_perf.thread_id, 
                                    thread_perf.efficiency_score);
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️  Analysis failed: {}", e);
                }
            }
            
            // Clean up
            let _ = std::fs::remove_dir_all(&output_dir);
        }
        Err(e) => {
            println!("⚠️  Failed to create profiling session: {}", e);
            println!("This may be expected on platforms without full resource monitoring support");
        }
    }
    
    Ok(())
}

/// Simulate comprehensive workload with mixed resource usage
fn simulate_comprehensive_workload() -> usize {
    let mut operation_count = 0;
    
    // Mix of CPU, memory, and IO operations
    for _iteration in 0..10 {
        // CPU intensive computation
        cpu_intensive_work(Duration::from_millis(50));
        operation_count += 1;
        
        // Memory allocations
        let mut memory_blocks = Vec::new();
        for _i in 0..100 {
            let block = vec![0u8; 1024]; // 1KB blocks
            memory_blocks.push(block);
        }
        operation_count += memory_blocks.len();
        
        // Simulate some processing
        for block in &mut memory_blocks {
            for (i, byte) in block.iter_mut().enumerate() {
                *byte = (i % 256) as u8;
            }
        }
        
        // IO simulation (create and delete temporary files)
        for i in 0..5 {
            let temp_file = std::env::temp_dir().join(format!("memscope_demo_{}.tmp", i));
            if let Ok(mut file) = std::fs::File::create(&temp_file) {
                use std::io::Write;
                let _ = file.write_all(&vec![42u8; 1024]);
                let _ = file.sync_all();
                operation_count += 1;
            }
            let _ = std::fs::remove_file(&temp_file);
        }
        
        // Brief pause to allow resource monitoring
        thread::sleep(Duration::from_millis(10));
    }
    
    operation_count
}

/// CPU-intensive work function
fn cpu_intensive_work(duration: Duration) {
    let start = std::time::Instant::now();
    let mut counter = 0u64;
    
    while start.elapsed() < duration {
        // Perform CPU-intensive computation
        counter = counter.wrapping_mul(17).wrapping_add(1);
        counter = counter.wrapping_mul(19).wrapping_add(3);
        counter = counter.wrapping_mul(23).wrapping_add(7);
    }
    
    // Prevent optimization from removing the computation
    if counter == u64::MAX {
        println!("Unlikely result: {}", counter);
    }
}

/// CPU-intensive thread worker
fn cpu_intensive_thread(stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Relaxed) {
        cpu_intensive_work(Duration::from_millis(100));
        thread::sleep(Duration::from_millis(10));
    }
}

/// Memory-intensive thread worker
fn memory_intensive_thread(stop: Arc<AtomicBool>) {
    let mut memory_pool = Vec::new();
    
    while !stop.load(Ordering::Relaxed) {
        // Allocate memory blocks
        for _i in 0..50 {
            let block = vec![0u8; 2048]; // 2KB blocks
            memory_pool.push(block);
        }
        
        // Process some blocks
        for block in memory_pool.iter_mut().take(20) {
            for byte in block.iter_mut() {
                *byte = (*byte).wrapping_add(1);
            }
        }
        
        // Clean up some memory periodically
        if memory_pool.len() > 500 {
            memory_pool.drain(0..100);
        }
        
        thread::sleep(Duration::from_millis(20));
    }
}

/// IO-intensive thread worker
fn io_intensive_thread(stop: Arc<AtomicBool>) {
    let mut file_counter = 0;
    
    while !stop.load(Ordering::Relaxed) {
        // Create temporary files
        for i in 0..3 {
            let temp_file = std::env::temp_dir().join(format!("memscope_io_{}_{}.tmp", file_counter, i));
            
            // Write data
            if let Ok(mut file) = std::fs::File::create(&temp_file) {
                use std::io::Write;
                let data = vec![42u8; 4096]; // 4KB
                let _ = file.write_all(&data);
                let _ = file.sync_all();
            }
            
            // Read data back
            if let Ok(mut file) = std::fs::File::open(&temp_file) {
                use std::io::Read;
                let mut buffer = Vec::new();
                let _ = file.read_to_end(&mut buffer);
            }
            
            // Clean up
            let _ = std::fs::remove_file(&temp_file);
        }
        
        file_counter += 1;
        thread::sleep(Duration::from_millis(50));
    }
}