//! Minimal demonstration of lock-free memory tracking
//!
//! This example shows the basic usage of the lock-free tracking system
//! with a simple working implementation.

use memscope_rs::lockfree::{
    finalize_thread_tracker, init_thread_tracker, track_allocation_lockfree,
    track_deallocation_lockfree, SamplingConfig,
};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Lock-free Memory Tracking Demo");

    let output_dir = std::env::temp_dir().join("minimal_lockfree_demo");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }
    std::fs::create_dir_all(&output_dir)?;

    let thread_count = 5;
    println!("Spawning {} threads for demonstration...", thread_count);

    let handles: Vec<_> = (0..thread_count)
        .map(|thread_idx| {
            let output_dir = output_dir.clone();

            thread::spawn(move || -> Result<(), String> {
                println!("Thread {} starting", thread_idx);

                // Initialize lock-free tracker for this thread
                let config = SamplingConfig::default();
                init_thread_tracker(&output_dir, Some(config))
                    .map_err(|e| format!("Thread {} init failed: {}", thread_idx, e))?;

                // Perform some allocations
                for i in 0..50 {
                    let ptr = (thread_idx * 1000 + i * 8) as usize;
                    let size = 64 + (i % 8) * 32;
                    let call_stack = vec![0x1000 + thread_idx, 0x2000 + i];

                    track_allocation_lockfree(ptr, size, &call_stack)
                        .map_err(|e| format!("Thread {} allocation failed: {}", thread_idx, e))?;

                    // Some deallocations
                    if i % 4 == 0 && i > 0 {
                        let dealloc_ptr = (thread_idx * 1000 + (i - 1) * 8) as usize;
                        track_deallocation_lockfree(dealloc_ptr, &call_stack).map_err(|e| {
                            format!("Thread {} deallocation failed: {}", thread_idx, e)
                        })?;
                    }
                }

                // Finalize tracking
                finalize_thread_tracker()
                    .map_err(|e| format!("Thread {} finalize failed: {}", thread_idx, e))?;

                println!("Thread {} completed successfully", thread_idx);
                Ok(())
            })
        })
        .collect();

    // Wait for all threads
    let mut successful = 0;
    for (idx, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(())) => {
                successful += 1;
            }
            Ok(Err(e)) => {
                println!("Thread {} failed: {}", idx, e);
            }
            Err(e) => {
                println!("Thread {} panicked: {:?}", idx, e);
            }
        }
    }

    println!("\n📊 Demo Results:");
    println!("✅ Successful threads: {}/{}", successful, thread_count);

    // Check output files
    let mut file_count = 0;
    for entry in std::fs::read_dir(&output_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with("memscope_thread_") {
                file_count += 1;
                let file_size = std::fs::metadata(&path)?.len();
                println!("📄 Created: {} ({} bytes)", file_name, file_size);
            }
        }
    }

    println!("📁 Total files generated: {}", file_count);

    // Cleanup
    std::fs::remove_dir_all(&output_dir)?;

    if successful == thread_count {
        println!("\n🎉 SUCCESS: Lock-free tracking works perfectly!");
        println!("   No fatal runtime errors, all threads completed independently.");
    } else {
        println!("\n⚠️  Some threads had issues, but no system-level failures occurred.");
    }

    Ok(())
}
