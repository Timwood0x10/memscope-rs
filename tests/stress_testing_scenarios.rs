//! Stress testing scenarios
//! 
//! Tests system behavior under extreme conditions including high allocation
//! rates, memory pressure, and resource exhaustion scenarios.

use memscope_rs::{track_var, get_global_tracker};
use std::time::Instant;

#[test]
fn test_high_allocation_rate_stress() {
    // Test system under high allocation rate stress
    let allocation_count = 10000;
    let stress_start = Instant::now();
    
    for iteration in 0..allocation_count {
        let stress_data = vec![iteration as u8; 32];
        track_var!(stress_data);
        drop(stress_data);
        
        // Check for reasonable performance every 1000 iterations
        if iteration % 1000 == 0 && iteration > 0 {
            let elapsed = stress_start.elapsed();
            let rate = iteration as f64 / elapsed.as_secs_f64();
            
            if rate < 100.0 {
                panic!("High allocation rate too slow: {:.1} allocs/sec at iteration {}", rate, iteration);
            }
        }
    }
    
    let total_time = stress_start.elapsed();
    let final_rate = allocation_count as f64 / total_time.as_secs_f64();
    
    if final_rate < 500.0 {
        panic!("Final allocation rate too slow: {:.1} allocs/sec", final_rate);
    }
    
    // Verify all allocations were tracked
    let tracker = get_global_tracker();
    match tracker.get_stats() {
        Ok(stats) => {
            if (stats.total_allocations as usize) < allocation_count {
                panic!("High rate stress lost allocations: {} < {}", 
                       stats.total_allocations, allocation_count);
            }
        }
        Err(e) => {
            panic!("Failed to get stats after high rate stress: {:?}", e);
        }
    }
}

#[test]
fn test_memory_pressure_stress() {
    // Test system under memory pressure
    let large_allocation_count = 100;
    let allocation_size = 1024 * 1024; // 1MB each
    let mut large_allocations = Vec::new();
    
    let pressure_start = Instant::now();
    
    // Create memory pressure
    for iteration in 0..large_allocation_count {
        let large_data = vec![iteration as u8; allocation_size];
        track_var!(large_data);
        large_allocations.push(large_data);
        
        // Check memory usage periodically
        if iteration % 10 == 0 {
            let tracker = get_global_tracker();
            match tracker.get_stats() {
                Ok(stats) => {
                    let expected_min_size = (iteration + 1) * allocation_size;
                    if (stats.total_allocated as usize) < expected_min_size {
                        panic!("Memory pressure tracking inaccurate at iteration {}", iteration);
                    }
                }
                Err(e) => {
                    panic!("Stats access failed under memory pressure: {:?}", e);
                }
            }
        }
    }
    
    let pressure_time = pressure_start.elapsed();
    
    // Verify system remained responsive under pressure
    if pressure_time.as_secs() > 30 {
        panic!("Memory pressure test took too long: {:?}", pressure_time);
    }
    
    // Clean up large allocations
    for allocation in large_allocations {
        drop(allocation);
    }
}

#[test]
fn test_rapid_export_stress() {
    // Test rapid binary export operations
    let export_count = 20;
    let export_start = Instant::now();
    
    // Ensure directory exists
    if let Err(e) = std::fs::create_dir_all("MemoryAnalysis") {
        panic!("Failed to create export directory: {:?}", e);
    }
    
    for export_iteration in 0..export_count {
        // Create data for each export
        for data_iteration in 0..50 {
            let export_data = vec![(export_iteration * 50 + data_iteration) as u8; 128];
            track_var!(export_data);
            drop(export_data);
        }
        
        // Perform rapid export
        let tracker = get_global_tracker();
        let export_file = format!("MemoryAnalysis/rapid_export_{}.memscope", export_iteration);
        
        match tracker.export_to_binary(&export_file) {
            Ok(_) => {
                // Verify export completed
                if !std::path::Path::new(&export_file).exists() {
                    panic!("Rapid export {} failed to create file", export_iteration);
                }
            }
            Err(e) => {
                panic!("Rapid export {} failed: {:?}", export_iteration, e);
            }
        }
        
        // Clean up immediately
        std::fs::remove_file(&export_file).ok();
    }
    
    let export_time = export_start.elapsed();
    let export_rate = export_count as f64 / export_time.as_secs_f64();
    
    if export_rate < 1.0 {
        panic!("Rapid export rate too slow: {:.1} exports/sec", export_rate);
    }
}

#[test]
fn test_fragmented_allocation_stress() {
    // Test system with highly fragmented allocation patterns
    let fragment_patterns = vec![
        (64, 100),    // 100 x 64-byte allocations
        (256, 50),    // 50 x 256-byte allocations
        (1024, 25),   // 25 x 1KB allocations
        (4096, 10),   // 10 x 4KB allocations
        (16384, 5),   // 5 x 16KB allocations
    ];
    
    let fragmentation_start = Instant::now();
    let mut all_allocations = Vec::new();
    
    // Create fragmented pattern
    for (size, count) in fragment_patterns {
        for iteration in 0..count {
            let fragmented_data = vec![(size / 64) as u8; size];
            track_var!(fragmented_data);
            all_allocations.push(fragmented_data);
            
            // Interleave with small allocations
            let small_data = vec![iteration as u8; 32];
            track_var!(small_data);
            all_allocations.push(small_data);
        }
    }
    
    let fragmentation_time = fragmentation_start.elapsed();
    
    // Verify fragmentation handling
    if fragmentation_time.as_secs() > 10 {
        panic!("Fragmentation stress took too long: {:?}", fragmentation_time);
    }
    
    // Verify tracking accuracy under fragmentation
    let tracker = get_global_tracker();
    match tracker.get_stats() {
        Ok(stats) => {
            let expected_allocations = all_allocations.len();
            if (stats.total_allocations as usize) < expected_allocations {
                panic!("Fragmentation stress lost allocations: {} < {}", 
                       stats.total_allocations, expected_allocations);
            }
        }
        Err(e) => {
            panic!("Stats access failed under fragmentation: {:?}", e);
        }
    }
    
    // Clean up fragmented allocations
    for allocation in all_allocations {
        drop(allocation);
    }
}

#[test]
fn test_concurrent_stress_combination() {
    // Test combination of concurrent access and high allocation rate
    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    
    let stress_duration = std::time::Duration::from_secs(2); // Reduced from 5 to 2 seconds
    let stop_flag = Arc::new(AtomicBool::new(false));
    let thread_count = 4;
    
    let stress_handles: Vec<_> = (0..thread_count).map(|thread_id| {
        let stop_flag_clone = Arc::clone(&stop_flag);
        
        thread::spawn(move || {
            let mut iteration_count = 0;
            
            while !stop_flag_clone.load(Ordering::Relaxed) {
                // High-rate allocations
                for batch in 0..10 {
                    let stress_data = vec![thread_id as u8; 64 + batch];
                    track_var!(stress_data);
                    drop(stress_data);
                }
                
                iteration_count += 1;
                
                // Brief pause to allow other threads
                std::thread::sleep(std::time::Duration::from_micros(100));
            }
            
            iteration_count
        })
    }).collect();
    
    // Run stress test for specified duration
    std::thread::sleep(stress_duration);
    stop_flag.store(true, Ordering::Relaxed);
    
    // Collect results
    let mut total_iterations = 0;
    for (index, handle) in stress_handles.into_iter().enumerate() {
        match handle.join() {
            Ok(iterations) => {
                total_iterations += iterations;
                if iterations == 0 {
                    panic!("Stress thread {} made no progress", index);
                }
            }
            Err(e) => {
                panic!("Concurrent stress thread {} failed: {:?}", index, e);
            }
        }
    }
    
    // Verify reasonable progress was made
    if total_iterations < 100 {
        panic!("Concurrent stress made insufficient progress: {}", total_iterations);
    }
    
    // Verify system remained functional
    let tracker = get_global_tracker();
    match tracker.get_stats() {
        Ok(stats) => {
            if stats.total_allocations == 0 {
                panic!("Concurrent stress lost all allocation tracking");
            }
        }
        Err(e) => {
            panic!("System non-functional after concurrent stress: {:?}", e);
        }
    }
}