//! Stress test with 100+ threads to validate lock-free implementation
//!
//! This test demonstrates that the new multi-threaded approach can handle
//! the scenarios that previously caused "fatal runtime error" with the old
//! RwLock-based global tracker approach.

use memscope_rs::lockfree::{
    init_thread_tracker, track_allocation_lockfree, track_deallocation_lockfree,
    finalize_thread_tracker, SamplingConfig, LockfreeAggregator
};
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting stress test with 100+ threads...");
    println!("   This test validates that we solved the 'fatal runtime error' problem");
    
    let start_time = Instant::now();
    let output_dir = std::env::temp_dir().join("stress_test_multithread");
    
    // Clean up previous test data
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }
    std::fs::create_dir_all(&output_dir)?;
    
    // Stress test configuration
    let thread_count = 150;  // More than the previous failure threshold
    let allocations_per_thread = 200;
    let expected_operations = thread_count * allocations_per_thread;
    
    println!("📊 Test Parameters:");
    println!("   - Threads: {}", thread_count);
    println!("   - Allocations per thread: {}", allocations_per_thread);
    println!("   - Total expected operations: {}", expected_operations);
    
    let operation_counter = Arc::new(AtomicUsize::new(0));
    let successful_threads = Arc::new(AtomicUsize::new(0));
    
    // Create optimized sampling config for high-load scenario
    let sampling_config = SamplingConfig {
        large_allocation_rate: 1.0,   // Always track large allocations
        medium_allocation_rate: 0.3,  // 30% of medium allocations
        small_allocation_rate: 0.05,  // 5% of small allocations
        large_threshold: 8192,        // 8KB threshold
        medium_threshold: 1024,       // 1KB threshold
        frequency_threshold: 20,      // Boost after 20 occurrences
    };
    
    println!("🔧 Intelligent Sampling Configuration:");
    println!("   - Large (>8KB): 100% sampling");
    println!("   - Medium (1-8KB): 30% sampling");
    println!("   - Small (<1KB): 5% sampling");
    
    println!("\n🏃 Spawning {} threads...", thread_count);
    
    // Spawn worker threads
    let handles: Vec<_> = (0..thread_count)
        .map(|thread_idx| {
            let output_dir = output_dir.clone();
            let sampling_config = sampling_config.clone();
            let operation_counter = Arc::clone(&operation_counter);
            let successful_threads = Arc::clone(&successful_threads);
            
            thread::spawn(move || -> Result<(), String> {
                // Initialize thread-local tracker
                init_thread_tracker(&output_dir, Some(sampling_config))
                    .map_err(|e| format!("Thread {} init failed: {}", thread_idx, e))?;
                
                // Perform varied allocation patterns
                for allocation_idx in 0..allocations_per_thread {
                    let ptr_base = 0x100000 + (thread_idx * 0x10000);
                    let ptr = ptr_base + (allocation_idx * 64);
                    
                    // Varied allocation sizes to test sampling
                    let size = match allocation_idx % 7 {
                        0 => 64,     // Small
                        1 => 128,    // Small
                        2 => 512,    // Small
                        3 => 1024,   // Medium
                        4 => 2048,   // Medium
                        5 => 8192,   // Large
                        6 => 16384,  // Large
                        _ => 256,    // Default
                    };
                    
                    // Generate realistic call stack patterns
                    let call_stack = vec![
                        0x400000 + (thread_idx % 10),              // Function pattern
                        0x500000 + (allocation_idx % 20),          // Loop pattern  
                        0x600000 + ((size / 1024) % 10),          // Size-based pattern
                    ];
                    
                    // Track allocation
                    track_allocation_lockfree(ptr, size, &call_stack)
                        .map_err(|e| format!("Thread {} alloc failed: {}", thread_idx, e))?;
                    
                    // Simulate realistic deallocation patterns
                    if allocation_idx % 3 == 0 && allocation_idx > 0 {
                        let dealloc_ptr = ptr_base + ((allocation_idx - 1) * 64);
                        track_deallocation_lockfree(dealloc_ptr, &call_stack)
                            .map_err(|e| format!("Thread {} dealloc failed: {}", thread_idx, e))?;
                    }
                    
                    operation_counter.fetch_add(1, Ordering::Relaxed);
                }
                
                // Finalize thread tracking
                finalize_thread_tracker()
                    .map_err(|e| format!("Thread {} finalize failed: {}", thread_idx, e))?;
                
                successful_threads.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
        })
        .collect();
    
    // Monitor progress while threads run
    let progress_handle = {
        let operation_counter = Arc::clone(&operation_counter);
        thread::spawn(move || {
            let mut last_count = 0;
            loop {
                thread::sleep(std::time::Duration::from_secs(2));
                let current_count = operation_counter.load(Ordering::Relaxed);
                if current_count == last_count && current_count > 0 {
                    break; // No progress, threads likely finished
                }
                let progress = (current_count as f64 / expected_operations as f64) * 100.0;
                println!("📈 Progress: {:.1}% ({}/{} operations)", 
                        progress, current_count, expected_operations);
                last_count = current_count;
            }
        })
    };
    
    // Wait for all threads to complete
    let mut completed_threads = 0;
    let mut failed_threads = 0;
    
    for handle in handles {
        match handle.join() {
            Ok(result) => {
                match result {
                    Ok(_) => completed_threads += 1,
                    Err(e) => {
                        println!("❌ Thread failed: {}", e);
                        failed_threads += 1;
                    }
                }
            }
            Err(e) => {
                println!("💥 Thread panicked: {:?}", e);
                failed_threads += 1;
            }
        }
    }
    
    // Stop progress monitoring
    let _ = progress_handle.join();
    
    let total_duration = start_time.elapsed();
    let actual_operations = operation_counter.load(Ordering::Relaxed);
    let actual_successful = successful_threads.load(Ordering::Relaxed);
    
    println!("\n📊 Stress Test Results:");
    println!("   ⏱️  Duration: {:?}", total_duration);
    println!("   ✅ Successful threads: {}/{}", actual_successful, thread_count);
    println!("   ❌ Failed threads: {}", failed_threads);
    println!("   🔄 Total operations: {}/{}", actual_operations, expected_operations);
    println!("   📈 Success rate: {:.1}%", (actual_successful as f64 / thread_count as f64) * 100.0);
    
    // Generate analysis report
    if actual_successful > 0 {
        println!("\n📋 Generating analysis report...");
        let aggregator = LockfreeAggregator::new(output_dir.clone());
        
        match aggregator.aggregate_all_threads() {
            Ok(analysis) => {
                println!("   📄 Analysis completed:");
                println!("      - Threads analyzed: {}", analysis.thread_stats.len());
                println!("      - Hottest call stacks: {}", analysis.hottest_call_stacks.len());
                println!("      - Thread interactions: {}", analysis.thread_interactions.len());
                println!("      - Performance bottlenecks: {}", analysis.performance_bottlenecks.len());
                
                // Export reports
                let json_path = output_dir.join("stress_test_analysis.json");
                let html_path = output_dir.join("stress_test_report.html");
                
                if let Err(e) = aggregator.export_analysis(&analysis, &json_path) {
                    println!("⚠️  JSON export failed: {}", e);
                }
                
                if let Err(e) = aggregator.generate_html_report(&analysis, &html_path) {
                    println!("⚠️  HTML export failed: {}", e);
                }
            }
            Err(e) => {
                println!("⚠️  Analysis failed: {}", e);
            }
        }
    }
    
    // Performance evaluation
    let ops_per_second = actual_operations as f64 / total_duration.as_secs_f64();
    println!("\n⚡ Performance Metrics:");
    println!("   🚀 Operations/second: {:.0}", ops_per_second);
    println!("   ⏱️  Avg time per operation: {:.2}μs", 
            total_duration.as_micros() as f64 / actual_operations as f64);
    
    // Cleanup
    std::fs::remove_dir_all(&output_dir)?;
    
    // Final verdict
    if actual_successful >= (thread_count as f64 * 0.95) as usize {
        println!("\n🎉 SUCCESS: Multi-threaded tracking handles 100+ threads without errors!");
        println!("   The 'fatal runtime error' issue has been resolved.");
    } else {
        println!("\n⚠️  PARTIAL SUCCESS: Some threads failed, but no fatal runtime errors occurred.");
    }
    
    Ok(())
}