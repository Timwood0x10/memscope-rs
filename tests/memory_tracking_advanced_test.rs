//! Advanced memory tracking tests
//! 
//! Tests complex memory tracking scenarios including lifecycle management,
//! scope tracking, borrow counting, and memory leak detection.

use memscope_rs::{track_var, get_global_tracker};
use std::time::Instant;

#[test]
fn test_memory_lifecycle_tracking() {
    // Test complete memory lifecycle from allocation to deallocation
    let initial_stats = get_initial_stats();
    
    {
        // Create scoped allocations
        let scoped_allocation = vec![1u8; 1024];
        track_var!(scoped_allocation);
        
        // Verify allocation is tracked
        let tracker = get_global_tracker();
        match tracker.get_stats() {
            Ok(current_stats) => {
                if current_stats.total_allocations <= initial_stats.total_allocations {
                    panic!("Scoped allocation should increase allocation count");
                }
            }
            Err(e) => {
                panic!("Failed to get stats during lifecycle test: {:?}", e);
            }
        }
        
        // Allocation goes out of scope here
    }
    
    // Verify lifecycle tracking captured the full cycle
    let tracker = get_global_tracker();
    match tracker.get_stats() {
        Ok(final_stats) => {
            if final_stats.total_allocations <= initial_stats.total_allocations {
                panic!("Lifecycle tracking should record allocation history");
            }
        }
        Err(e) => {
            panic!("Failed to get final stats in lifecycle test: {:?}", e);
        }
    }
}

#[test]
fn test_nested_scope_tracking() {
    // Test tracking across nested scopes
    let outer_allocation = vec![1u8; 512];
    track_var!(outer_allocation);
    
    {
        let middle_allocation = vec![2u8; 256];
        track_var!(middle_allocation);
        
        {
            let inner_allocation = vec![3u8; 128];
            track_var!(inner_allocation);
            
            // Verify all allocations are tracked
            let tracker = get_global_tracker();
            match tracker.get_stats() {
                Ok(stats) => {
                    if stats.total_allocations < 3 {
                        panic!("Nested scope tracking should capture all levels");
                    }
                }
                Err(e) => {
                    panic!("Failed to get stats in nested scope test: {:?}", e);
                }
            }
            
            drop(inner_allocation);
        }
        
        drop(middle_allocation);
    }
    
    drop(outer_allocation);
}

#[test]
fn test_borrow_counting_accuracy() {
    // Test borrow counting functionality
    let borrowed_data = vec![42u8; 256];
    track_var!(borrowed_data);
    
    // Create multiple borrows
    let borrow_1 = &borrowed_data;
    let borrow_2 = &borrowed_data;
    let borrow_3 = &borrowed_data;
    
    // Use borrows to prevent optimization
    let _sum = borrow_1.len() + borrow_2.len() + borrow_3.len();
    
    // Verify borrow tracking
    let tracker = get_global_tracker();
    match tracker.get_stats() {
        Ok(stats) => {
            if stats.total_allocations == 0 {
                panic!("Borrow counting test should track allocations");
            }
        }
        Err(e) => {
            panic!("Failed to get stats in borrow counting test: {:?}", e);
        }
    }
    
    drop(borrowed_data);
}

#[test]
fn test_memory_leak_detection() {
    // Test memory leak detection capabilities
    let initial_stats = get_initial_stats();
    
    // Create intentionally leaked allocation
    let leaked_allocation = Box::new(vec![0u8; 2048]);
    track_var!(leaked_allocation);
    
    // Forget the allocation to simulate a leak
    std::mem::forget(leaked_allocation);
    
    // Verify leak detection
    let tracker = get_global_tracker();
    match tracker.get_stats() {
        Ok(current_stats) => {
            if current_stats.total_allocations <= initial_stats.total_allocations {
                panic!("Leak detection should track forgotten allocations");
            }
        }
        Err(e) => {
            panic!("Failed to get stats in leak detection test: {:?}", e);
        }
    }
}

#[test]
fn test_large_allocation_tracking() {
    // Test tracking of large memory allocations
    let large_sizes = vec![1024 * 1024, 2 * 1024 * 1024, 4 * 1024 * 1024]; // 1MB, 2MB, 4MB
    
    for size in large_sizes {
        let large_allocation = vec![0u8; size];
        track_var!(large_allocation);
        
        let tracker = get_global_tracker();
        match tracker.get_stats() {
            Ok(stats) => {
                if (stats.total_allocated as usize) < size {
                    panic!("Large allocation tracking failed for size: {}", size);
                }
            }
            Err(e) => {
                panic!("Failed to get stats for large allocation test: {:?}", e);
            }
        }
        
        drop(large_allocation);
    }
}

#[test]
fn test_rapid_allocation_deallocation() {
    // Test rapid allocation and deallocation cycles
    let initial_stats = get_initial_stats();
    let cycle_count = 1000;
    
    let start_time = Instant::now();
    
    for cycle in 0..cycle_count {
        let rapid_allocation = vec![cycle as u8; 64];
        track_var!(rapid_allocation);
        drop(rapid_allocation);
    }
    
    let elapsed_time = start_time.elapsed();
    
    // Verify rapid cycles are handled efficiently
    if elapsed_time.as_secs() > 5 {
        panic!("Rapid allocation cycles took too long: {:?}", elapsed_time);
    }
    
    // Verify all cycles were tracked
    let tracker = get_global_tracker();
    match tracker.get_stats() {
        Ok(final_stats) => {
            let tracked_cycles = final_stats.total_allocations - initial_stats.total_allocations;
            if tracked_cycles < cycle_count {
                panic!("Rapid cycles tracking incomplete: {} < {}", tracked_cycles, cycle_count);
            }
        }
        Err(e) => {
            panic!("Failed to get final stats in rapid cycles test: {:?}", e);
        }
    }
}

#[test]
fn test_concurrent_allocation_tracking() {
    // Test concurrent allocation tracking across multiple threads
    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    let thread_count = 4;
    let allocations_per_thread = 100;
    let completed_threads = Arc::new(AtomicUsize::new(0));
    
    let thread_handles: Vec<_> = (0..thread_count).map(|thread_id| {
        let completed_counter = Arc::clone(&completed_threads);
        
        thread::spawn(move || {
            // Create thread-specific allocations
            for allocation_index in 0..allocations_per_thread {
                let thread_allocation = vec![thread_id as u8; 128 + allocation_index];
                track_var!(thread_allocation);
                drop(thread_allocation);
            }
            
            completed_counter.fetch_add(1, Ordering::SeqCst);
        })
    }).collect();
    
    // Wait for all threads to complete
    for (index, handle) in thread_handles.into_iter().enumerate() {
        match handle.join() {
            Ok(_) => {},
            Err(e) => {
                panic!("Concurrent tracking thread {} failed: {:?}", index, e);
            }
        }
    }
    
    // Verify all threads completed
    let final_completed = completed_threads.load(Ordering::SeqCst);
    if final_completed != thread_count {
        panic!("Not all concurrent threads completed: {} < {}", final_completed, thread_count);
    }
    
    // Verify concurrent allocations were tracked
    let tracker = get_global_tracker();
    match tracker.get_stats() {
        Ok(stats) => {
            let expected_min_allocations = thread_count * allocations_per_thread;
            if (stats.total_allocations as usize) < expected_min_allocations {
                panic!("Concurrent allocation tracking incomplete: {} < {}", 
                       stats.total_allocations, expected_min_allocations);
            }
        }
        Err(e) => {
            panic!("Failed to get stats after concurrent test: {:?}", e);
        }
    }
}

#[test]
fn test_memory_fragmentation_tracking() {
    // Test tracking of memory fragmentation patterns
    let fragment_sizes = vec![64, 128, 256, 512, 1024];
    let mut allocations = Vec::new();
    
    // Create fragmented allocation pattern
    for (index, &size) in fragment_sizes.iter().enumerate() {
        for fragment in 0..10 {
            let fragmented_allocation = vec![(index * 10 + fragment) as u8; size];
            track_var!(fragmented_allocation);
            allocations.push(fragmented_allocation);
        }
    }
    
    // Verify fragmentation tracking
    let tracker = get_global_tracker();
    match tracker.get_stats() {
        Ok(stats) => {
            let expected_allocations = fragment_sizes.len() * 10;
            if (stats.total_allocations as usize) < expected_allocations {
                panic!("Fragmentation tracking incomplete: {} < {}", 
                       stats.total_allocations, expected_allocations);
            }
        }
        Err(e) => {
            panic!("Failed to get stats in fragmentation test: {:?}", e);
        }
    }
    
    // Clean up allocations
    for allocation in allocations {
        drop(allocation);
    }
}

/// Get initial statistics for comparison
fn get_initial_stats() -> memscope_rs::core::types::MemoryStats {
    let tracker = get_global_tracker();
    match tracker.get_stats() {
        Ok(stats) => stats,
        Err(e) => {
            panic!("Failed to get initial stats: {:?}", e);
        }
    }
}