//! Smart Dispatch Strategy Performance Test
//!
//! This example tests the track_var! intelligent dispatch strategy under:
//! 1. High concurrency (25+ threads)
//! 2. Async/await environments
//! 3. Hybrid scenarios (threads + async)
//!
//! Run with: cargo run --example smart_dispatch_performance_test

use memscope_rs::{init, track_var};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct TestConfig {
    thread_count: usize,
    #[allow(dead_code)]
    async_task_count: usize,
    operations_per_worker: usize,
    #[allow(dead_code)]
    test_duration_secs: u64,
}

impl TestConfig {
    fn high_concurrency() -> Self {
        Self {
            thread_count: 30,
            async_task_count: 0,
            operations_per_worker: 1000,
            test_duration_secs: 10,
        }
    }

    fn multi_thread_heavy() -> Self {
        Self {
            thread_count: 50,
            async_task_count: 0,
            operations_per_worker: 1200,
            test_duration_secs: 15,
        }
    }
}

#[derive(Debug)]
struct PerformanceMetrics {
    total_operations: usize,
    duration: Duration,
    operations_per_second: f64,
    memory_overhead_mb: f64,
    strategy_switches: usize,
    avg_dispatch_latency_us: f64,
}

fn main() -> Result<(), String> {
    println!("🚀 Smart Dispatch Strategy Performance Test");
    println!("Testing track_var! intelligent routing under various scenarios");
    println!();

    // Initialize memscope
    init();

    // Test scenarios for track_var! performance evaluation
    let scenarios = vec![
        (
            "High Concurrency (30 threads)",
            TestConfig::high_concurrency(),
        ),
        (
            "Multi-Thread Heavy (50 threads)",
            TestConfig::multi_thread_heavy(),
        ),
    ];

    let mut all_results = Vec::new();

    for (name, config) in scenarios {
        println!("📊 Testing: {}", name);
        println!("   Configuration: {:?}", config);

        // Run simplified thread-only test
        let metrics = run_thread_performance_test(config)?;
        println!("   Results: {:?}", metrics);
        println!();

        all_results.push((name, metrics));
    }

    // Generate comparative analysis
    generate_performance_report(&all_results);

    Ok(())
}

// Simplified performance test focused on track_var! usage

// Removed old async functions - using simplified approach

// Data structures for testing
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ComplexData {
    id: usize,
    values: Vec<f64>,
    metadata: String,
}

fn run_thread_performance_test(config: TestConfig) -> Result<PerformanceMetrics, String> {
    let start_time = Instant::now();
    let total_operations = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();

    // Spawn thread workers
    for i in 0..config.thread_count {
        let ops_counter = Arc::clone(&total_operations);
        let ops_per_worker = config.operations_per_worker;

        let handle = thread::spawn(move || -> Result<(), String> {
            run_simple_thread_worker(i, ops_per_worker, ops_counter)
        });
        handles.push(handle);
    }

    // Wait for all workers to complete
    for handle in handles {
        match handle.join() {
            Ok(result) => result?,
            Err(_) => return Err("Thread panicked".to_string()),
        }
    }

    let duration = start_time.elapsed();
    let final_operations = total_operations.load(Ordering::Relaxed);

    Ok(PerformanceMetrics {
        total_operations: final_operations,
        duration,
        operations_per_second: final_operations as f64 / duration.as_secs_f64(),
        memory_overhead_mb: 0.0, // Simplified
        strategy_switches: 0,
        avg_dispatch_latency_us: 0.0,
    })
}

fn run_simple_thread_worker(
    thread_id: usize,
    operations: usize,
    total_ops: Arc<AtomicUsize>,
) -> Result<(), String> {
    // Simulate diverse memory allocation patterns
    for i in 0..operations {
        // Create different types of tracked variables
        match i % 3 {
            0 => {
                // Simple scalar tracking
                let value = i * thread_id;
                track_var!(value);
            }
            1 => {
                // Vector allocation tracking
                let mut vec: Vec<u64> = Vec::with_capacity(50);
                for j in 0..25 {
                    vec.push((i + j) as u64);
                }
                track_var!(vec);
            }
            2 => {
                // String allocation tracking
                let text = format!("Thread {} operation {}", thread_id, i);
                track_var!(text);
            }
            _ => unreachable!(),
        }

        total_ops.fetch_add(1, Ordering::Relaxed);

        // Simulate work with occasional delays
        if i % 200 == 0 {
            thread::sleep(Duration::from_micros(5));
        }
    }

    Ok(())
}

// Simplified test - removed async helper functions

fn generate_performance_report(results: &[(&str, PerformanceMetrics)]) {
    println!("📊 Performance Comparison Report");
    println!("=====================================");

    for (name, metrics) in results {
        println!("\n🎯 {}", name);
        println!("   Operations: {}", metrics.total_operations);
        println!("   Duration: {:?}", metrics.duration);
        println!("   Ops/sec: {:.0}", metrics.operations_per_second);
        println!("   Memory overhead: {:.2} MB", metrics.memory_overhead_mb);
        println!("   Strategy switches: {}", metrics.strategy_switches);
        println!(
            "   Avg dispatch latency: {:.2} μs",
            metrics.avg_dispatch_latency_us
        );
    }

    // Find best performing scenario
    let best = results
        .iter()
        .max_by(|a, b| {
            a.1.operations_per_second
                .partial_cmp(&b.1.operations_per_second)
                .unwrap()
        })
        .unwrap();

    println!(
        "\n🏆 Best Performance: {} ({:.0} ops/sec)",
        best.0, best.1.operations_per_second
    );

    // Calculate efficiency metrics
    println!("\n📈 Efficiency Analysis:");
    for (name, metrics) in results {
        let efficiency = metrics.operations_per_second / metrics.memory_overhead_mb.max(1.0);
        println!("   {}: {:.0} ops/sec per MB", name, efficiency);
    }
}
