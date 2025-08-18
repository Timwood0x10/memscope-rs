//! 🧵 Multithreaded Memory Demo
//!
//! Demonstrates memory tracking across multiple threads

use memscope_rs::{init, track_var};
use std::thread;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    println!("🧵 Multithreaded Memory Demo Started!");
    println!("{}", "=".repeat(50));

    let total_start = Instant::now();

    // Create multiple threads that allocate memory
    println!("\n📦 Creating data across {} threads...", 4);

    let mut handles = Vec::new();

    for thread_id in 0..4 {
        let handle = thread::spawn(move || {
            for i in 0..250 {
                let data = vec![(thread_id * 100 + i) as u8; 128];
                track_var!(data);
            }
            println!("Thread {} completed", thread_id);
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let total_time = total_start.elapsed();

    println!("\n📊 Multithreaded Demo Report");
    println!("{}", "=".repeat(35));
    println!("Total execution time: {:.2}ms", total_time.as_millis());
    println!("All threads completed successfully");

    println!("\n🏁 Multithreaded Demo Completed!");

    Ok(())
}
