/// Enhanced Multi-Thread Memory Tracking Showcase (New API)
///
/// This example demonstrates the enhanced new API with:
/// - System monitoring
/// - Hotspot analysis
/// - Sampling configuration
/// - Analysis report export
///
/// Comparison with old API (complex_multithread_showcase.rs):
/// - Old: 712 lines, complex init/finalize, manual pointer management
/// - New: ~200 lines, simple tracker!() and track!() macros, same features
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use memscope_rs::tracker::SamplingConfig;
use memscope_rs::{track, tracker};

/// Thread workload types
#[derive(Clone, Copy, Debug)]
enum WorkloadType {
    DataProcessing,
    ComputeIntensive,
    IoSimulation,
    BatchProcessing,
    StreamProcessing,
    CacheWorker,
}

/// Thread execution statistics
#[derive(Debug, Clone)]
struct ThreadStats {
    #[allow(dead_code)]
    thread_id: usize,
    workload_type: WorkloadType,
    total_allocations: usize,
    peak_memory_bytes: u64,
    execution_time_ms: u64,
    operations_completed: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Enhanced Multi-Thread Memory Tracking (New API)");
    println!("📊 Configuration: 100 threads, 6 workload types, system monitoring enabled");

    let start_time = Instant::now();
    let completed_operations = Arc::new(AtomicUsize::new(0));
    let thread_stats = Arc::new(Mutex::new(Vec::<ThreadStats>::new()));

    // Create tracker with enhanced features
    let tracker = tracker!()
        .with_system_monitoring()
        .with_sampling(SamplingConfig::demo());

    println!("🧵 Launching 100 threads with diverse workload patterns...");

    let handles: Vec<_> = (0..100)
        .map(|thread_id| {
            let completed_ops = completed_operations.clone();
            let stats_collector = thread_stats.clone();

            thread::spawn(move || {
                let workload_type = match thread_id % 6 {
                    0 => WorkloadType::DataProcessing,
                    1 => WorkloadType::ComputeIntensive,
                    2 => WorkloadType::IoSimulation,
                    3 => WorkloadType::BatchProcessing,
                    4 => WorkloadType::StreamProcessing,
                    _ => WorkloadType::CacheWorker,
                };

                let result = execute_workload(thread_id, workload_type, &completed_ops);

                if let Ok(ref stats) = result {
                    if let Ok(mut stats_vec) = stats_collector.lock() {
                        stats_vec.push(stats.clone());
                    }
                }

                result
            })
        })
        .collect();

    // Wait for all threads
    println!("⏳ Waiting for thread completion...");
    let mut success_count = 0;
    let mut error_count = 0;

    for (i, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(_)) => {
                success_count += 1;
                if (i + 1) % 20 == 0 {
                    println!("✅ Completed {} threads", i + 1);
                }
            }
            Ok(Err(e)) => {
                error_count += 1;
                if error_count <= 5 {
                    println!("❌ Thread {i} error: {e}");
                }
            }
            Err(_) => {
                error_count += 1;
                if error_count <= 5 {
                    println!("💥 Thread {i} panicked");
                }
            }
        }
    }

    let total_time = start_time.elapsed();

    // Print summary
    println!("\n📊 ========== EXECUTION SUMMARY ==========");
    println!(
        "⏱️  Total execution time: {:.2} seconds",
        total_time.as_secs_f64()
    );
    println!("✅ Successful threads: {success_count}");
    println!("❌ Failed threads: {error_count}");
    println!(
        "🔄 Total operations: {}",
        completed_operations.load(Ordering::Relaxed)
    );
    println!(
        "📈 Operations per second: {:.0}",
        completed_operations.load(Ordering::Relaxed) as f64 / total_time.as_secs_f64()
    );

    // Print workload breakdown
    if let Ok(stats) = thread_stats.lock() {
        println!("\n📋 ========== WORKLOAD BREAKDOWN ==========");
        let mut workload_summary: HashMap<String, (usize, usize, u64, f64, usize)> = HashMap::new();
        for stat in stats.iter() {
            let entry = workload_summary
                .entry(format!("{:?}", stat.workload_type))
                .or_insert((0, 0, 0, 0.0, 0));
            entry.0 += 1;
            entry.1 += stat.total_allocations;
            entry.2 += stat.peak_memory_bytes;
            entry.3 += stat.execution_time_ms as f64;
            entry.4 += stat.operations_completed;
        }

        for (workload_type, (count, allocs, memory, time, ops)) in workload_summary {
            println!("🔹 {workload_type}: {count} threads");
            println!(
                "   └─ Allocations: {allocs} ({:.1} avg)",
                allocs as f64 / count as f64
            );
            println!(
                "   └─ Peak Memory: {:.1} MB ({:.1} MB avg)",
                memory as f64 / 1024.0 / 1024.0,
                memory as f64 / 1024.0 / 1024.0 / count as f64
            );
            println!(
                "   └─ Execution: {time:.1} ms ({:.1} ms avg)",
                time / count as f64
            );
            println!(
                "   └─ Operations: {ops} ({:.1} avg)",
                ops as f64 / count as f64
            );
        }
    }

    // Generate analysis report
    println!("\n🔍 Generating analysis report...");
    let report = tracker.analyze();

    println!("\n📊 ========== ANALYSIS REPORT ==========");
    println!("Total allocations: {}", report.total_allocations);
    println!("Active allocations: {}", report.active_allocations);
    println!(
        "Peak memory: {:.2} MB",
        report.peak_memory_bytes as f64 / 1024.0 / 1024.0
    );
    println!(
        "Allocation rate: {:.0} ops/sec",
        report.allocation_rate_per_sec
    );

    println!("\n🔥 Top Allocation Hotspots:");
    let mut sorted_hotspots = report.hotspots.clone();
    sorted_hotspots.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    for (i, hotspot) in sorted_hotspots.iter().take(10).enumerate() {
        println!(
            "  {}. {} ({}): {} bytes, {} allocations",
            i + 1,
            hotspot.var_name,
            hotspot.type_name,
            hotspot.total_size,
            hotspot.allocation_count
        );
    }

    // Export data
    println!("\n📤 Exporting data...");
    tracker.export_json("enhanced_multithread")?;
    println!("✅ Exported JSON to MemoryAnalysis/enhanced_multithread/");

    // Export analysis report
    tracker.export_analysis("enhanced_multithread")?;
    println!("✅ Exported analysis to MemoryAnalysis/enhanced_multithread_analysis.json");

    println!("\n🎉 Enhanced Multi-Thread Showcase Complete!");
    println!("📄 Generated files:");
    println!("   📊 JSON: MemoryAnalysis/enhanced_multithread/");
    println!("   📈 Analysis: MemoryAnalysis/enhanced_multithread_analysis.json");

    Ok(())
}

fn execute_workload(
    thread_id: usize,
    workload_type: WorkloadType,
    completed_ops: &Arc<AtomicUsize>,
) -> Result<ThreadStats, String> {
    let start_time = Instant::now();
    let mut allocation_count = 0;
    let mut peak_memory_bytes = 0u64;
    let mut operations_completed = 0;

    // Create per-thread tracker (shares global state)
    let tracker = tracker!();

    match workload_type {
        WorkloadType::DataProcessing => {
            let iterations = 800 + (thread_id % 10) * 50;
            for i in 0..iterations {
                let size = 32768 + (i % 8) * 16384;
                let data = vec![i as u8; size];
                track!(tracker, data);

                allocation_count += 1;
                peak_memory_bytes += size as u64;
                operations_completed += 1;
                completed_ops.fetch_add(1, Ordering::Relaxed);

                thread::sleep(Duration::from_micros(100 + (i % 50) as u64));
            }
        }
        WorkloadType::ComputeIntensive => {
            let iterations = 500 + (thread_id % 5) * 100;
            for i in 0..iterations {
                let buffer = vec![i as u8; 1024 + (i % 4) * 512];
                track!(tracker, buffer);

                allocation_count += 1;
                peak_memory_bytes += buffer.len() as u64;

                // Heavy computation
                let mut result = 0u64;
                for j in 0..1000 {
                    result = result.wrapping_add((i * j) as u64);
                }

                operations_completed += 1;
                completed_ops.fetch_add(1, Ordering::Relaxed);

                thread::sleep(Duration::from_micros(50));
            }
        }
        WorkloadType::IoSimulation => {
            let iterations = 600 + (thread_id % 8) * 75;
            for i in 0..iterations {
                let buffer = vec![i as u8; 8192 + (i % 6) * 4096];
                track!(tracker, buffer);

                allocation_count += 1;
                peak_memory_bytes += buffer.len() as u64;
                operations_completed += 1;
                completed_ops.fetch_add(1, Ordering::Relaxed);

                thread::sleep(Duration::from_micros(200 + (i % 100) as u64));
            }
        }
        WorkloadType::BatchProcessing => {
            let batch_count = 8 + (thread_id % 4);
            for batch in 0..batch_count {
                let batch_size = 50 + (batch % 3) * 25;
                for i in 0..batch_size {
                    let data = vec![i as u8; 16384 + (i % 12) * 8192];
                    track!(tracker, data);

                    allocation_count += 1;
                    peak_memory_bytes += data.len() as u64;
                }

                operations_completed += batch_size;
                completed_ops.fetch_add(batch_size, Ordering::Relaxed);

                thread::sleep(Duration::from_millis(10));
            }
        }
        WorkloadType::StreamProcessing => {
            let iterations = 1200 + (thread_id % 15) * 80;
            for i in 0..iterations {
                let data = vec![i as u8; 512 + (i % 8) * 256];
                track!(tracker, data);

                allocation_count += 1;
                peak_memory_bytes += data.len() as u64;
                operations_completed += 1;
                completed_ops.fetch_add(1, Ordering::Relaxed);

                thread::sleep(Duration::from_micros(10));
            }
        }
        WorkloadType::CacheWorker => {
            let iterations = 400 + (thread_id % 6) * 50;
            let mut cache: HashMap<usize, Vec<u8>> = HashMap::new();

            for i in 0..iterations {
                let key = i % 50;
                if let std::collections::hash_map::Entry::Vacant(e) = cache.entry(key) {
                    let data = vec![i as u8; 4096 + (key % 10) * 2048];
                    track!(tracker, data);
                    e.insert(data.clone());

                    allocation_count += 1;
                    peak_memory_bytes += data.len() as u64;
                }

                operations_completed += 1;
                completed_ops.fetch_add(1, Ordering::Relaxed);

                thread::sleep(Duration::from_micros(30));

                // Evict old entries
                if cache.len() > 30 {
                    let evict_key = (i.saturating_sub(30)) % 50;
                    cache.remove(&evict_key);
                }
            }
        }
    }

    let execution_time = start_time.elapsed();

    Ok(ThreadStats {
        thread_id,
        workload_type,
        total_allocations: allocation_count,
        peak_memory_bytes,
        execution_time_ms: execution_time.as_millis() as u64,
        operations_completed,
    })
}
