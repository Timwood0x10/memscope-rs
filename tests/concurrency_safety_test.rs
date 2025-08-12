//! Concurrency safety tests
//! 
//! Tests thread safety, concurrent access patterns, and data race prevention
//! in multi-threaded memory tracking scenarios.

use memscope_rs::{track_var, get_global_tracker};
use std::thread;
use std::sync::{Arc, Barrier};
use std::time::Duration;

#[test]
fn test_concurrent_allocation_tracking() {
    // Test concurrent allocation tracking across multiple threads
    let thread_count = 8;
    let allocations_per_thread = 50;
    let barrier = Arc::new(Barrier::new(thread_count));
    
    let thread_handles: Vec<_> = (0..thread_count).map(|thread_id| {
        let barrier_clone = Arc::clone(&barrier);
        
        thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();
            
            // Perform concurrent allocations
            for allocation_index in 0..allocations_per_thread {
                let thread_data = vec![thread_id as u8; 64 + allocation_index];
                track_var!(thread_data);
                
                // Small delay to increase chance of race conditions
                thread::sleep(Duration::from_micros(10));
                
                drop(thread_data);
            }
        })
    }).collect();
    
    // Wait for all threads to complete
    for (index, handle) in thread_handles.into_iter().enumerate() {
        match handle.join() {
            Ok(_) => {},
            Err(e) => {
                panic!("Concurrent allocation thread {} failed: {:?}", index, e);
            }
        }
    }
    
    // Verify concurrent tracking worked correctly
    let tracker = get_global_tracker();
    match tracker.get_stats() {
        Ok(stats) => {
            let expected_min = thread_count * allocations_per_thread;
            if (stats.total_allocations as usize) < expected_min {
                panic!("Concurrent tracking lost allocations: {} < {}", 
                       stats.total_allocations, expected_min);
            }
        }
        Err(e) => {
            panic!("Failed to get stats after concurrent test: {:?}", e);
        }
    }
}

#[test]
fn test_concurrent_binary_export() {
    // Test concurrent binary export operations
    let export_thread_count = 4;
    
    let export_handles: Vec<_> = (0..export_thread_count).map(|thread_id| {
        thread::spawn(move || {
            // Create thread-specific allocations
            for i in 0..20 {
                let export_data = vec![thread_id as u8; 128 + i];
                track_var!(export_data);
                drop(export_data);
            }
            
            // Ensure directory exists
            if let Err(e) = std::fs::create_dir_all("MemoryAnalysis") {
                panic!("Thread {} failed to create directory: {:?}", thread_id, e);
            }
            
            // Perform concurrent export
            let tracker = get_global_tracker();
            let export_file = format!("MemoryAnalysis/concurrent_export_{}.memscope", thread_id);
            
            match tracker.export_to_binary(&export_file) {
                Ok(_) => {
                    // Verify export file exists
                    if !std::path::Path::new(&export_file).exists() {
                        panic!("Thread {} export file not created", thread_id);
                    }
                }
                Err(e) => {
                    panic!("Thread {} export failed: {:?}", thread_id, e);
                }
            }
            
            // Cleanup
            std::fs::remove_file(&export_file).ok();
        })
    }).collect();
    
    // Wait for all export threads
    for (index, handle) in export_handles.into_iter().enumerate() {
        match handle.join() {
            Ok(_) => {},
            Err(e) => {
                panic!("Concurrent export thread {} failed: {:?}", index, e);
            }
        }
    }
}

#[test]
fn test_concurrent_statistics_access() {
    // Test concurrent access to statistics
    let stats_thread_count = 6;
    let barrier = Arc::new(Barrier::new(stats_thread_count));
    
    // Create some initial allocations
    for i in 0..10 {
        let initial_data = vec![i as u8; 256];
        track_var!(initial_data);
        drop(initial_data);
    }
    
    let stats_handles: Vec<_> = (0..stats_thread_count).map(|thread_id| {
        let barrier_clone = Arc::clone(&barrier);
        
        thread::spawn(move || {
            barrier_clone.wait();
            
            // Concurrent statistics access
            for iteration in 0..100 {
                let tracker = get_global_tracker();
                match tracker.get_stats() {
                    Ok(stats) => {
                        // Verify stats are reasonable
                        if stats.total_allocations == 0 {
                            panic!("Thread {} iteration {} got zero allocations", thread_id, iteration);
                        }
                    }
                    Err(e) => {
                        panic!("Thread {} stats access failed: {:?}", thread_id, e);
                    }
                }
                
                // Small delay between accesses
                thread::sleep(Duration::from_micros(1));
            }
        })
    }).collect();
    
    // Wait for all stats threads
    for (index, handle) in stats_handles.into_iter().enumerate() {
        match handle.join() {
            Ok(_) => {},
            Err(e) => {
                panic!("Concurrent stats thread {} failed: {:?}", index, e);
            }
        }
    }
}

#[test]
fn test_thread_local_isolation() {
    // Test that thread-local data is properly isolated
    let isolation_thread_count = 4;
    
    let isolation_handles: Vec<_> = (0..isolation_thread_count).map(|thread_id| {
        thread::spawn(move || {
            // Create thread-specific pattern
            let pattern_value = (thread_id + 1) * 10;
            
            for i in 0..30 {
                let isolated_data = vec![pattern_value as u8; 64 + i];
                track_var!(isolated_data);
                
                // Verify our data maintains expected pattern
                if isolated_data[0] != pattern_value as u8 {
                    panic!("Thread {} data corruption detected", thread_id);
                }
                
                drop(isolated_data);
            }
            
            // Return thread ID for verification
            thread_id
        })
    }).collect();
    
    // Collect results and verify isolation
    let mut completed_threads = Vec::new();
    for handle in isolation_handles {
        match handle.join() {
            Ok(thread_id) => {
                completed_threads.push(thread_id);
            }
            Err(e) => {
                panic!("Thread isolation test failed: {:?}", e);
            }
        }
    }
    
    // Verify all threads completed
    completed_threads.sort();
    let expected_threads: Vec<_> = (0..isolation_thread_count).collect();
    if completed_threads != expected_threads {
        panic!("Thread isolation incomplete: {:?} != {:?}", completed_threads, expected_threads);
    }
}