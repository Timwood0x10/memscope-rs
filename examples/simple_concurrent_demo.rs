//! 🚀 Simple Concurrent Memory Tracing Demo
//!
//! Basic demonstration of memory tracking in concurrent scenarios

use memscope_rs::{get_global_tracker, init, track_var};
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    println!("🚀 Simple Concurrent Memory Tracing Demo");
    println!("{}", "=".repeat(50));

    let start = Instant::now();

    // Test 1: Basic multi-threading
    println!("\n🧵 Test 1: Multi-threading");
    test_multithreading()?;

    // Test 2: Thread communication
    println!("\n📡 Test 2: Thread Communication");
    test_thread_communication()?;

    // Test 3: Parallel processing
    println!("\n⚡ Test 3: Parallel Processing");
    test_parallel_processing()?;

    let total_time = start.elapsed();

    // Show final stats
    show_final_stats(total_time)?;

    println!("\n🏁 Demo Completed!");
    Ok(())
}

fn test_multithreading() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    let mut handles = Vec::new();

    // Spawn 4 threads
    for thread_id in 0..4 {
        let handle = thread::spawn(move || {
            for i in 0..1000 {
                // Use simple trackable types
                let data = vec![(thread_id + i) as u8; 256 + (i % 128)];
                track_var!(data);

                // Create some strings
                let text = format!("thread_{}_item_{}", thread_id, i);
                track_var!(text);

                // Simulate some work
                if i % 100 == 0 {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = start.elapsed();
    println!(
        "✅ Multi-threading completed in {:.2}ms",
        elapsed.as_secs_f64() * 1000.0
    );

    Ok(())
}

fn test_thread_communication() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();

    // Producer thread
    let producer = thread::spawn(move || {
        for i in 0..1000 {
            let data = vec![i as u8; 128 + (i % 64)];
            track_var!(data);

            if tx.send(data).is_err() {
                break;
            }

            if i % 100 == 0 {
                thread::sleep(Duration::from_micros(500));
            }
        }
    });

    // Consumer thread
    let consumer = thread::spawn(move || {
        let mut count = 0;
        while let Ok(received_data) = rx.recv() {
            let processed = received_data
                .iter()
                .map(|&x| x.wrapping_mul(2))
                .collect::<Vec<_>>();
            track_var!(processed);
            count += 1;
        }
        count
    });

    producer.join().unwrap();
    let processed_count = consumer.join().unwrap();

    let elapsed = start.elapsed();
    println!(
        "✅ Thread communication completed in {:.2}ms, processed {} items",
        elapsed.as_secs_f64() * 1000.0,
        processed_count
    );

    Ok(())
}

fn test_parallel_processing() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    let mut handles = Vec::new();

    // Create multiple worker threads
    for worker_id in 0..6 {
        let handle = thread::spawn(move || {
            for batch in 0..10 {
                // Process a batch of data
                for item in 0..100 {
                    let size = 64 + (item % 32);
                    let data = vec![(worker_id + batch + item) as u8; size];
                    track_var!(data);

                    // Simulate processing
                    let result_text = format!("worker_{}_batch_{}_item_{}", worker_id, batch, item);
                    track_var!(result_text);
                }

                // Small delay between batches
                thread::sleep(Duration::from_micros(100));
            }
        });
        handles.push(handle);
    }

    // Wait for all workers
    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = start.elapsed();
    println!(
        "✅ Parallel processing completed in {:.2}ms",
        elapsed.as_secs_f64() * 1000.0
    );

    Ok(())
}

fn show_final_stats(total_time: Duration) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Final Memory Statistics");
    println!("{}", "=".repeat(40));

    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("• Total allocations: {}", stats.total_allocations);
        println!("• Active allocations: {}", stats.active_allocations);
        println!(
            "• Peak memory: {:.2}MB",
            stats.peak_memory as f64 / 1024.0 / 1024.0
        );
        println!(
            "• Active memory: {:.2}MB",
            stats.active_memory as f64 / 1024.0 / 1024.0
        );

        let rate = stats.total_allocations as f64 / total_time.as_secs_f64();
        println!("• Allocation rate: {:.0} allocs/sec", rate);

        // Export data
        println!("\n💾 Exporting trace data...");
        let export_start = Instant::now();
        tracker.export_to_binary("simple_concurrent_demo")?;
        let export_time = export_start.elapsed();
        println!(
            "✅ Export completed in {:.2}ms",
            export_time.as_secs_f64() * 1000.0
        );
    }

    Ok(())
}
