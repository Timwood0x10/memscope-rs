//! Simple test for lock-free multi-threaded tracking validation
//!
//! This test verifies that the new thread-local approach works correctly
//! without any shared state or lock contention issues.

use memscope_rs::lockfree::{
    finalize_thread_tracker, init_thread_tracker, track_allocation_lockfree,
    track_deallocation_lockfree, SamplingConfig,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting simple multi-threaded tracking test...");

    // Create output directory
    let output_dir = std::env::temp_dir().join("simple_multithread_test");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }
    std::fs::create_dir_all(&output_dir)?;

    // Shared counter for tracking progress
    let counter = Arc::new(AtomicUsize::new(0));
    let thread_count = 10;
    let allocations_per_thread = 100;

    println!(
        "Spawning {} threads with {} allocations each",
        thread_count, allocations_per_thread
    );

    // Spawn worker threads
    let handles: Vec<_> = (0..thread_count)
        .map(|thread_idx| {
            let output_dir = output_dir.clone();
            let counter = Arc::clone(&counter);

            thread::spawn(move || -> Result<(), String> {
                // Initialize tracker for this thread
                let config = SamplingConfig::default();
                init_thread_tracker(&output_dir, Some(config))
                    .map_err(|e| format!("Failed to init tracker: {}", e))?;

                // Perform allocations
                for i in 0..allocations_per_thread {
                    let ptr = (thread_idx * 10000 + i * 8) as usize;
                    let size = 64 + (i % 10) * 64;
                    let call_stack = vec![0x1000 + thread_idx, 0x2000 + i];

                    track_allocation_lockfree(ptr, size, &call_stack)
                        .map_err(|e| format!("Failed to track allocation: {}", e))?;

                    // Track some deallocations
                    if i % 5 == 0 && i > 0 {
                        let dealloc_ptr = (thread_idx * 10000 + (i - 1) * 8) as usize;
                        track_deallocation_lockfree(dealloc_ptr, &call_stack)
                            .map_err(|e| format!("Failed to track deallocation: {}", e))?;
                    }

                    counter.fetch_add(1, Ordering::Relaxed);
                }

                // Finalize tracker
                finalize_thread_tracker()
                    .map_err(|e| format!("Failed to finalize tracker: {}", e))?;

                println!("Thread {} completed successfully", thread_idx);
                Ok(())
            })
        })
        .collect();

    // Wait for all threads to complete
    let mut successful_threads = 0;
    for (idx, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(result) => match result {
                Ok(_) => {
                    successful_threads += 1;
                }
                Err(e) => {
                    println!("Thread {} failed: {}", idx, e);
                }
            },
            Err(e) => {
                println!("Thread {} panicked: {:?}", idx, e);
            }
        }
    }

    let total_operations = counter.load(Ordering::Relaxed);
    println!("Test completed:");
    println!(
        "  - Successful threads: {}/{}",
        successful_threads, thread_count
    );
    println!("  - Total operations: {}", total_operations);

    // Verify output files were created
    let mut file_count = 0;
    for entry in std::fs::read_dir(&output_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with("memscope_thread_") {
                file_count += 1;
                println!("Created file: {}", file_name);
            }
        }
    }

    println!("Generated {} thread files", file_count);

    // Cleanup
    std::fs::remove_dir_all(&output_dir)?;

    if successful_threads == thread_count {
        println!("✅ All threads completed successfully - multi-threaded tracking works!");
    } else {
        println!("❌ Some threads failed - need to investigate");
    }

    Ok(())
}
