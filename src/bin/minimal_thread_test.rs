//! Minimal test to isolate the multi-threading issue
//!
//! This test uses only the new lock-free components without any
//! global tracker access to identify the root cause.

use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting minimal thread isolation test...");
    
    let counter = Arc::new(AtomicUsize::new(0));
    let thread_count = 200; // Test with high thread count
    
    println!("Spawning {} threads with minimal operations...", thread_count);
    
    // Test pure thread spawning without any memscope operations
    let handles: Vec<_> = (0..thread_count)
        .map(|thread_idx| {
            let counter = Arc::clone(&counter);
            
            thread::spawn(move || {
                // Minimal operations - no memscope calls
                for i in 0..100 {
                    let _dummy = thread_idx * 1000 + i;
                    counter.fetch_add(1, Ordering::Relaxed);
                }
                println!("Thread {} completed", thread_idx);
            })
        })
        .collect();
    
    // Wait for completion
    let mut completed = 0;
    for handle in handles {
        if handle.join().is_ok() {
            completed += 1;
        }
    }
    
    let total_ops = counter.load(Ordering::Relaxed);
    println!("Test completed:");
    println!("  - Threads completed: {}/{}", completed, thread_count);
    println!("  - Total operations: {}", total_ops);
    
    if completed == thread_count {
        println!("✅ Pure threading works - issue is in memscope components");
    } else {
        println!("❌ Basic threading failed - system issue");
    }
    
    Ok(())
}