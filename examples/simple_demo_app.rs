//! Simple Demo Application
//!
//! A simple application that demonstrates different memory allocation patterns
//! to showcase the unified backend's tracking capabilities.

use std::thread;
use std::time::Duration;

fn main() {
    println!("🎯 MemScope Demo Application");
    println!("============================");

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "single" => demo_single_threaded(),
        "multi" => demo_multi_threaded(),
        "async" => demo_async_simulation(),
        "memory-intensive" => demo_memory_intensive(),
        "leak-simulation" => demo_leak_simulation(),
        _ => {
            println!("❌ Unknown demo type: {}", args[1]);
            print_usage();
        }
    }

    println!("✅ Demo completed!");
}

fn print_usage() {
    println!("Usage: simple_demo_app <demo_type>");
    println!();
    println!("Demo types:");
    println!("  single           - Single-threaded memory operations");
    println!("  multi            - Multi-threaded memory operations");
    println!("  async            - Simulated async memory patterns");
    println!("  memory-intensive - Heavy memory allocation/deallocation");
    println!("  leak-simulation  - Simulated memory leak patterns");
    println!();
    println!("Example usage with MemScope:");
    println!("  memscope analyze --mode unified simple_demo_app single");
    println!("  memscope run --track-async simple_demo_app async");
    println!("  memscope analyze --strategy thread-local simple_demo_app multi");
}

/// Demo 1: Single-threaded operations
fn demo_single_threaded() {
    println!("🧵 Running single-threaded demo...");

    // Create and manipulate various data structures
    let mut vectors = Vec::new();

    for i in 0..100 {
        let data = vec![i; i % 10 + 1];
        vectors.push(data);

        if i % 20 == 0 {
            // Simulate some processing
            thread::sleep(Duration::from_millis(1));
        }
    }

    // Process the data
    let total_elements: usize = vectors.iter().map(|v| v.len()).sum();
    println!(
        "📊 Processed {} vectors with {} total elements",
        vectors.len(),
        total_elements
    );

    // Create some larger allocations
    for size in [1024, 2048, 4096] {
        let large_data = vec![0u8; size];
        println!("🔧 Created buffer of size: {} bytes", large_data.len());
        thread::sleep(Duration::from_millis(5));
    }
}

/// Demo 2: Multi-threaded operations
fn demo_multi_threaded() {
    println!("🔀 Running multi-threaded demo...");

    let num_threads = 4;
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let handle = thread::spawn(move || {
            println!("🧵 Thread {} starting...", thread_id);

            // Thread-specific memory operations
            let mut thread_data = Vec::new();

            for i in 0..50 {
                let data = vec![thread_id * 1000 + i; 20];
                thread_data.push(data);

                if i % 10 == 0 {
                    thread::sleep(Duration::from_millis(2));
                }
            }

            // Process data
            let sum: usize = thread_data.iter().flat_map(|v| v.iter()).sum();

            println!("🧵 Thread {} completed with sum: {}", thread_id, sum);
            sum
        });

        handles.push(handle);
    }

    // Wait for all threads and collect results
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    let total_sum: usize = results.iter().sum();
    println!("📊 All threads completed. Total sum: {}", total_sum);
}

/// Demo 3: Simulated async operations
fn demo_async_simulation() {
    println!("⚡ Running async simulation demo...");

    // Simulate async-like behavior with delays and task switching
    let tasks = 8;

    for task_id in 0..tasks {
        println!("📋 Starting task {}", task_id);

        // Simulate task setup
        let task_data = vec![task_id; 100];

        // Simulate async yield points
        for step in 0..5 {
            let step_data = [step; 50];
            thread::sleep(Duration::from_millis(3)); // Simulate async yield

            // Process step data
            let _processed: Vec<_> = step_data
                .iter()
                .zip(task_data.iter())
                .map(|(a, b)| a + b)
                .collect();

            if step % 2 == 0 {
                println!("  ⏱️  Task {} step {} completed", task_id, step);
            }
        }

        println!("✅ Task {} finished", task_id);
    }
}

/// Demo 4: Memory-intensive operations
fn demo_memory_intensive() {
    println!("💾 Running memory-intensive demo...");

    let mut large_allocations = Vec::new();

    // Create progressively larger allocations
    for i in 1..=10 {
        let size = i * 1024 * 100; // 100KB, 200KB, ..., 1MB
        let data = vec![i as u8; size];

        println!("🔧 Allocated {} KB", size / 1024);
        large_allocations.push(data);

        // Simulate some processing time
        thread::sleep(Duration::from_millis(10));

        // Occasionally drop some allocations
        if i % 3 == 0 && !large_allocations.is_empty() {
            large_allocations.remove(0);
            println!("🗑️  Freed an allocation");
        }
    }

    // Final processing
    let total_memory: usize = large_allocations.iter().map(|v| v.len()).sum();
    println!("📊 Final memory usage: {} KB", total_memory / 1024);

    // Batch deallocation
    large_allocations.clear();
    println!("🧹 Cleared all allocations");
}

/// Demo 5: Simulated memory leak patterns
fn demo_leak_simulation() {
    println!("🚨 Running leak simulation demo (controlled)...");

    let mut growing_collections = Vec::new();

    // Simulate a growing collection that might indicate a leak
    for cycle in 1..=20 {
        let mut cycle_data = Vec::new();

        // Each cycle adds more data than the previous
        for item in 0..(cycle * 10) {
            let item_data = vec![item; cycle];
            cycle_data.push(item_data);
        }

        // Some cycles "leak" by not properly cleaning up
        if cycle % 4 != 0 {
            growing_collections.push(cycle_data);
        } else {
            // Occasionally clean up to show the pattern
            println!("🧹 Cleanup cycle {}", cycle);
            if !growing_collections.is_empty() {
                growing_collections.remove(0);
            }
        }

        let current_items: usize = growing_collections
            .iter()
            .map(|collection| collection.len())
            .sum();

        println!(
            "📈 Cycle {}: {} total items in memory",
            cycle, current_items
        );
        thread::sleep(Duration::from_millis(20));
    }

    // Final cleanup to show we can detect the "fix"
    println!("🔧 Applying leak fix...");
    growing_collections.clear();
    println!("✅ All leaked memory cleaned up");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_threaded_demo() {
        demo_single_threaded();
        // Should complete without panicking
    }

    #[test]
    fn test_multi_threaded_demo() {
        demo_multi_threaded();
        // Should complete without panicking
    }

    #[test]
    fn test_async_simulation_demo() {
        demo_async_simulation();
        // Should complete without panicking
    }

    #[test]
    fn test_memory_intensive_demo() {
        demo_memory_intensive();
        // Should complete without panicking
    }

    #[test]
    fn test_leak_simulation_demo() {
        demo_leak_simulation();
        // Should complete without panicking
    }
}
