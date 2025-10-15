/// Complex Multi-Thread Memory Tracking Showcase
///
/// This example demonstrates advanced multi-thread memory tracking with:
/// - 100 threads with different workload patterns
/// - Complex memory allocation strategies
/// - Realistic workload simulation
/// - Comprehensive resource monitoring
///
/// Generated outputs:
/// - JSON: ./Memoryanalysis/complex_showcase_comprehensive.json
/// - HTML: ./Memoryanalysis/complex_showcase_dashboard.html
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use memscope_rs::lockfree::{
    export_comprehensive_analysis, finalize_thread_tracker, init_thread_tracker,
    track_allocation_lockfree, track_deallocation_lockfree, IntegratedProfilingSession,
    SamplingConfig,
};

/// Thread workload types for realistic simulation
#[derive(Clone, Copy, Debug)]
enum WorkloadType {
    DataProcessing,   // High memory, medium CPU
    ComputeIntensive, // High CPU, low memory
    IoSimulation,     // Medium memory, high I/O simulation
    BatchProcessing,  // Burst memory allocation
    StreamProcessing, // Continuous small allocations
    CacheWorker,      // Memory caching patterns
}

/// Thread execution statistics
#[derive(Debug, Clone)]
struct ThreadStats {
    #[allow(dead_code)]
    thread_id: usize,
    workload_type: WorkloadType,
    total_allocations: usize,
    peak_memory_mb: f64,
    execution_time_ms: u64,
    operations_completed: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Complex Multi-Thread Memory Tracking Showcase");
    println!("📊 Configuration: 100 threads, 6 workload types, selective tracking");

    let output_dir = PathBuf::from("./Memoryanalysis");
    std::fs::create_dir_all(&output_dir)?;

    // Initialize platform monitoring
    let mut session = IntegratedProfilingSession::new(&output_dir)?;
    session.start_profiling()?;
    let start_time = Instant::now();

    // Shared statistics collection
    let completed_operations = Arc::new(AtomicUsize::new(0));
    let thread_stats = Arc::new(Mutex::new(Vec::<ThreadStats>::new()));

    println!("🧵 Launching 100 threads with diverse workload patterns...");

    // Launch 100 threads with different workload patterns
    let handles: Vec<_> = (0..100)
        .map(|thread_id| {
            let output_dir = output_dir.clone();
            let completed_ops = completed_operations.clone();
            let stats_collector = thread_stats.clone();

            thread::spawn(move || {
                let workload_type = determine_workload_type(thread_id);
                let should_track = should_track_thread(thread_id, workload_type);

                let result = execute_complex_workload(
                    thread_id,
                    workload_type,
                    should_track,
                    &output_dir,
                    &completed_ops,
                );

                // Collect thread statistics
                if let Ok(ref stats) = result {
                    if let Ok(mut stats_vec) = stats_collector.lock() {
                        stats_vec.push(stats.clone());
                    }
                }

                result
            })
        })
        .collect();

    // Wait for all threads to complete
    println!("⏳ Waiting for thread completion...");
    let mut success_count = 0;
    let mut error_count = 0;

    for (i, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(_)) => {
                success_count += 1;
                if (i + 1) % 10 == 0 {
                    println!("✅ Completed {} threads", i + 1);
                }
            }
            Ok(Err(e)) => {
                error_count += 1;
                println!("❌ Thread {i} error: {e}");
            }
            Err(_) => {
                error_count += 1;
                println!("💥 Thread {i} panicked");
            }
        }
    }

    // Monitoring handled automatically
    let total_time = start_time.elapsed();

    // Print execution summary
    print_execution_summary(
        success_count,
        error_count,
        total_time,
        completed_operations.load(Ordering::Relaxed),
        &thread_stats,
    )?;

    // Export comprehensive analysis
    println!("📊 Exporting comprehensive analysis...");
    let analysis = session.stop_profiling_and_analyze()?;
    export_comprehensive_analysis(&analysis, &output_dir, "complex_showcase")?;

    println!("\n🎉 Complex Multi-Thread Showcase Complete!");
    println!("📄 Generated files:");
    println!("   📊 JSON: ./Memoryanalysis/complex_showcase_comprehensive.json");
    println!("   🌐 HTML: ./Memoryanalysis/complex_showcase_dashboard.html");
    println!("   📈 Rankings: ./Memoryanalysis/complex_showcase_resource_rankings.json");
    println!("\n🌐 Open the HTML dashboard to explore interactive analysis!");

    Ok(())
}

/// Determine workload type based on thread ID
fn determine_workload_type(thread_id: usize) -> WorkloadType {
    match thread_id % 6 {
        0 => WorkloadType::DataProcessing,
        1 => WorkloadType::ComputeIntensive,
        2 => WorkloadType::IoSimulation,
        3 => WorkloadType::BatchProcessing,
        4 => WorkloadType::StreamProcessing,
        5 => WorkloadType::CacheWorker,
        _ => unreachable!(),
    }
}

/// Determine if thread should be tracked (selective tracking strategy)
fn should_track_thread(thread_id: usize, workload_type: WorkloadType) -> bool {
    match workload_type {
        // Track all data processing and batch processing threads
        WorkloadType::DataProcessing | WorkloadType::BatchProcessing => true,
        // Track every 3rd compute intensive thread
        WorkloadType::ComputeIntensive => thread_id.is_multiple_of(3),
        // Track half of I/O simulation threads
        WorkloadType::IoSimulation => thread_id.is_multiple_of(2),
        // Track every 4th stream processing thread
        WorkloadType::StreamProcessing => thread_id.is_multiple_of(4),
        // Track all cache workers
        WorkloadType::CacheWorker => true,
    }
}

/// Execute complex workload based on thread type
fn execute_complex_workload(
    thread_id: usize,
    workload_type: WorkloadType,
    should_track: bool,
    output_dir: &Path,
    completed_ops: &Arc<AtomicUsize>,
) -> Result<ThreadStats, String> {
    let start_time = Instant::now();
    let mut allocation_count = 0;
    let mut peak_memory_bytes = 0u64;
    let mut operations_completed = 0;

    // Initialize tracking if needed
    if should_track {
        init_thread_tracker(output_dir, Some(SamplingConfig::demo()))
            .map_err(|e| format!("Thread {} init failed: {}", thread_id, e))?;
    }

    // Execute workload based on type
    match workload_type {
        WorkloadType::DataProcessing => {
            let result = execute_data_processing_workload(
                thread_id,
                should_track,
                &mut allocation_count,
                &mut peak_memory_bytes,
                &mut operations_completed,
                completed_ops,
            );
            if let Err(e) = result {
                return Err(format!("Data processing failed: {}", e));
            }
        }
        WorkloadType::ComputeIntensive => {
            let result = execute_compute_intensive_workload(
                thread_id,
                should_track,
                &mut allocation_count,
                &mut peak_memory_bytes,
                &mut operations_completed,
                completed_ops,
            );
            if let Err(e) = result {
                return Err(format!("Compute intensive failed: {}", e));
            }
        }
        WorkloadType::IoSimulation => {
            let result = execute_io_simulation_workload(
                thread_id,
                should_track,
                &mut allocation_count,
                &mut peak_memory_bytes,
                &mut operations_completed,
                completed_ops,
            );
            if let Err(e) = result {
                return Err(format!("I/O simulation failed: {}", e));
            }
        }
        WorkloadType::BatchProcessing => {
            let result = execute_batch_processing_workload(
                thread_id,
                should_track,
                &mut allocation_count,
                &mut peak_memory_bytes,
                &mut operations_completed,
                completed_ops,
            );
            if let Err(e) = result {
                return Err(format!("Batch processing failed: {}", e));
            }
        }
        WorkloadType::StreamProcessing => {
            let result = execute_stream_processing_workload(
                thread_id,
                should_track,
                &mut allocation_count,
                &mut peak_memory_bytes,
                &mut operations_completed,
                completed_ops,
            );
            if let Err(e) = result {
                return Err(format!("Stream processing failed: {}", e));
            }
        }
        WorkloadType::CacheWorker => {
            let result = execute_cache_worker_workload(
                thread_id,
                should_track,
                &mut allocation_count,
                &mut peak_memory_bytes,
                &mut operations_completed,
                completed_ops,
            );
            if let Err(e) = result {
                return Err(format!("Cache worker failed: {}", e));
            }
        }
    }

    // Finalize tracking if needed
    if should_track {
        finalize_thread_tracker()
            .map_err(|e| format!("Thread {} finalize failed: {}", thread_id, e))?;
    }

    let execution_time = start_time.elapsed();

    Ok(ThreadStats {
        thread_id,
        workload_type,
        total_allocations: allocation_count,
        peak_memory_mb: peak_memory_bytes as f64 / 1024.0 / 1024.0,
        execution_time_ms: execution_time.as_millis() as u64,
        operations_completed,
    })
}

/// Data Processing Workload: Large memory allocations for data transformation
fn execute_data_processing_workload(
    thread_id: usize,
    should_track: bool,
    allocation_count: &mut usize,
    peak_memory_bytes: &mut u64,
    operations_completed: &mut usize,
    completed_ops: &Arc<AtomicUsize>,
) -> Result<(), String> {
    let mut allocated_ptrs = Vec::new();
    let base_iterations = 800 + (thread_id % 10) * 50; // 800-1300 iterations

    for i in 0..base_iterations {
        // Large data chunks (simulating data processing)
        let size = 32768 + (i % 8) * 16384; // 32KB - 144KB
        let ptr = (thread_id as u64) << 32 | (i as u64) << 16 | size as u64;

        if should_track {
            let call_stack = vec![
                execute_data_processing_workload as *const () as usize,
                execute_complex_workload as *const () as usize,
                main as *const () as usize,
            ];

            track_allocation_lockfree(ptr as usize, size, &call_stack)
                .map_err(|e| format!("Track allocation failed: {}", e))?;
        }

        allocated_ptrs.push((ptr as usize, size));
        *allocation_count += 1;
        *peak_memory_bytes += size as u64;

        // Simulate data processing work
        thread::sleep(Duration::from_micros(100 + (i % 50) as u64));

        // Periodic cleanup (every 10 allocations)
        if i % 10 == 0 && !allocated_ptrs.is_empty() {
            let (old_ptr, old_size) = allocated_ptrs.remove(0);

            if should_track {
                let call_stack = vec![
                    execute_data_processing_workload as *const () as usize,
                    execute_complex_workload as *const () as usize,
                ];

                track_deallocation_lockfree(old_ptr, &call_stack)
                    .map_err(|e| format!("Track deallocation failed: {}", e))?;
            }

            *peak_memory_bytes = peak_memory_bytes.saturating_sub(old_size as u64);
        }

        *operations_completed += 1;
        completed_ops.fetch_add(1, Ordering::Relaxed);
    }

    // Clean up remaining allocations
    for (ptr, _) in allocated_ptrs {
        if should_track {
            let call_stack = vec![execute_data_processing_workload as *const () as usize];
            track_deallocation_lockfree(ptr, &call_stack)
                .map_err(|e| format!("Cleanup failed: {}", e))?;
        }
    }

    Ok(())
}

/// Compute Intensive Workload: Minimal memory, heavy computation
fn execute_compute_intensive_workload(
    thread_id: usize,
    should_track: bool,
    allocation_count: &mut usize,
    peak_memory_bytes: &mut u64,
    operations_completed: &mut usize,
    completed_ops: &Arc<AtomicUsize>,
) -> Result<(), String> {
    let base_iterations = 500 + (thread_id % 5) * 100; // 500-900 iterations

    for i in 0..base_iterations {
        // Small memory allocations for computation buffers
        let size = 1024 + (i % 4) * 512; // 1KB - 2.5KB
        let ptr = (thread_id as u64) << 32 | (i as u64) << 8 | size as u64;

        if should_track {
            let call_stack = vec![
                execute_compute_intensive_workload as *const () as usize,
                execute_complex_workload as *const () as usize,
            ];

            track_allocation_lockfree(ptr as usize, size, &call_stack)
                .map_err(|e| format!("Track allocation failed: {}", e))?;
        }

        *allocation_count += 1;
        *peak_memory_bytes += size as u64;

        // Simulate intensive computation (more CPU, less memory)
        let mut result = 0u64;
        for j in 0..1000 {
            result = result.wrapping_add((i * j) as u64);
        }

        // Quick deallocation (compute workloads don't hold much memory)
        if should_track {
            let call_stack = vec![execute_compute_intensive_workload as *const () as usize];
            track_deallocation_lockfree(ptr as usize, &call_stack)
                .map_err(|e| format!("Track deallocation failed: {}", e))?;
        }

        *peak_memory_bytes = peak_memory_bytes.saturating_sub(size as u64);

        thread::sleep(Duration::from_micros(50));
        *operations_completed += 1;
        completed_ops.fetch_add(1, Ordering::Relaxed);
    }

    Ok(())
}

/// I/O Simulation Workload: Medium memory with I/O patterns
fn execute_io_simulation_workload(
    thread_id: usize,
    should_track: bool,
    allocation_count: &mut usize,
    peak_memory_bytes: &mut u64,
    operations_completed: &mut usize,
    completed_ops: &Arc<AtomicUsize>,
) -> Result<(), String> {
    let mut io_buffers = Vec::new();
    let base_iterations = 600 + (thread_id % 8) * 75; // 600-1125 iterations

    for i in 0..base_iterations {
        // I/O buffer allocations
        let size = 8192 + (i % 6) * 4096; // 8KB - 28KB
        let ptr = (thread_id as u64) << 32 | (i as u64) << 12 | size as u64;

        if should_track {
            let call_stack = vec![
                execute_io_simulation_workload as *const () as usize,
                execute_complex_workload as *const () as usize,
            ];

            track_allocation_lockfree(ptr as usize, size, &call_stack)
                .map_err(|e| format!("Track allocation failed: {}", e))?;
        }

        io_buffers.push((ptr as usize, size));
        *allocation_count += 1;
        *peak_memory_bytes += size as u64;

        // Simulate I/O wait time
        thread::sleep(Duration::from_micros(200 + (i % 100) as u64));

        // Buffer rotation (keep last 5 buffers)
        if io_buffers.len() > 5 {
            let (old_ptr, old_size) = io_buffers.remove(0);

            if should_track {
                let call_stack = vec![execute_io_simulation_workload as *const () as usize];
                track_deallocation_lockfree(old_ptr, &call_stack)
                    .map_err(|e| format!("Track deallocation failed: {}", e))?;
            }

            *peak_memory_bytes = peak_memory_bytes.saturating_sub(old_size as u64);
        }

        *operations_completed += 1;
        completed_ops.fetch_add(1, Ordering::Relaxed);
    }

    // Clean up remaining buffers
    for (ptr, _) in io_buffers {
        if should_track {
            let call_stack = vec![execute_io_simulation_workload as *const () as usize];
            track_deallocation_lockfree(ptr, &call_stack)
                .map_err(|e| format!("Cleanup failed: {}", e))?;
        }
    }

    Ok(())
}

/// Batch Processing Workload: Burst allocations
fn execute_batch_processing_workload(
    thread_id: usize,
    should_track: bool,
    allocation_count: &mut usize,
    peak_memory_bytes: &mut u64,
    operations_completed: &mut usize,
    completed_ops: &Arc<AtomicUsize>,
) -> Result<(), String> {
    let batch_count = 8 + (thread_id % 4); // 8-11 batches

    for batch in 0..batch_count {
        let mut batch_allocations = Vec::new();
        let batch_size = 50 + (batch % 3) * 25; // 50-100 allocations per batch

        // Burst allocation phase
        for i in 0..batch_size {
            let size = 16384 + (i % 12) * 8192; // 16KB - 104KB
            let ptr =
                (thread_id as u64) << 32 | (batch as u64) << 16 | (i as u64) << 8 | size as u64;

            if should_track {
                let call_stack = vec![
                    execute_batch_processing_workload as *const () as usize,
                    execute_complex_workload as *const () as usize,
                ];

                track_allocation_lockfree(ptr as usize, size, &call_stack)
                    .map_err(|e| format!("Track allocation failed: {}", e))?;
            }

            batch_allocations.push((ptr as usize, size));
            *allocation_count += 1;
            *peak_memory_bytes += size as u64;
        }

        // Processing phase (hold memory)
        thread::sleep(Duration::from_millis(10 + (batch % 5) as u64));

        // Burst deallocation phase
        for (ptr, size) in batch_allocations {
            if should_track {
                let call_stack = vec![execute_batch_processing_workload as *const () as usize];
                track_deallocation_lockfree(ptr, &call_stack)
                    .map_err(|e| format!("Track deallocation failed: {}", e))?;
            }

            *peak_memory_bytes = peak_memory_bytes.saturating_sub(size as u64);
        }

        *operations_completed += batch_size;
        completed_ops.fetch_add(batch_size, Ordering::Relaxed);

        // Inter-batch pause
        thread::sleep(Duration::from_millis(5));
    }

    Ok(())
}

/// Stream Processing Workload: Continuous small allocations
fn execute_stream_processing_workload(
    thread_id: usize,
    should_track: bool,
    allocation_count: &mut usize,
    peak_memory_bytes: &mut u64,
    operations_completed: &mut usize,
    completed_ops: &Arc<AtomicUsize>,
) -> Result<(), String> {
    let mut stream_buffer = Vec::new();
    let base_iterations = 1200 + (thread_id % 15) * 80;

    for i in 0..base_iterations {
        let size = 512 + (i % 8) * 256;
        let ptr = (thread_id as u64) << 32 | (i as u64) << 4 | size as u64;

        if should_track {
            let call_stack = vec![
                execute_stream_processing_workload as *const () as usize,
                execute_complex_workload as *const () as usize,
            ];
            track_allocation_lockfree(ptr as usize, size, &call_stack)
                .map_err(|e| format!("Track allocation failed: {}", e))?;
        }

        stream_buffer.push((ptr as usize, size));
        *allocation_count += 1;
        *peak_memory_bytes += size as u64;

        thread::sleep(Duration::from_micros(10));

        if stream_buffer.len() > 20 {
            let (old_ptr, old_size) = stream_buffer.remove(0);
            if should_track {
                let call_stack = vec![execute_stream_processing_workload as *const () as usize];
                track_deallocation_lockfree(old_ptr, &call_stack)
                    .map_err(|e| format!("Track deallocation failed: {}", e))?;
            }
            *peak_memory_bytes = peak_memory_bytes.saturating_sub(old_size as u64);
        }

        *operations_completed += 1;
        completed_ops.fetch_add(1, Ordering::Relaxed);
    }

    for (ptr, _) in stream_buffer {
        if should_track {
            let call_stack = vec![execute_stream_processing_workload as *const () as usize];
            track_deallocation_lockfree(ptr, &call_stack)
                .map_err(|e| format!("Cleanup failed: {}", e))?;
        }
    }
    Ok(())
}

fn execute_cache_worker_workload(
    thread_id: usize,
    should_track: bool,
    allocation_count: &mut usize,
    peak_memory_bytes: &mut u64,
    operations_completed: &mut usize,
    completed_ops: &Arc<AtomicUsize>,
) -> Result<(), String> {
    let mut cache_entries = HashMap::new();
    let base_iterations = 400 + (thread_id % 6) * 50;

    for i in 0..base_iterations {
        let cache_key = i % 50;
        if let std::collections::hash_map::Entry::Vacant(e) = cache_entries.entry(cache_key) {
            let size = 4096 + (cache_key % 10) * 2048;
            let ptr = (thread_id as u64) << 32 | (cache_key as u64) << 16 | size as u64;

            if should_track {
                let call_stack = vec![
                    execute_cache_worker_workload as *const () as usize,
                    execute_complex_workload as *const () as usize,
                ];
                track_allocation_lockfree(ptr as usize, size, &call_stack)
                    .map_err(|e| format!("Track allocation failed: {}", e))?;
            }

            e.insert((ptr as usize, size));
            *allocation_count += 1;
            *peak_memory_bytes += size as u64;
        }

        thread::sleep(Duration::from_micros(30 + (i % 20) as u64));

        if cache_entries.len() > 30 {
            let evict_key = (i.saturating_sub(30)) % 50;
            if let Some((ptr, size)) = cache_entries.remove(&evict_key) {
                if should_track {
                    let call_stack = vec![execute_cache_worker_workload as *const () as usize];
                    track_deallocation_lockfree(ptr, &call_stack)
                        .map_err(|e| format!("Track deallocation failed: {}", e))?;
                }
                *peak_memory_bytes = peak_memory_bytes.saturating_sub(size as u64);
            }
        }

        *operations_completed += 1;
        completed_ops.fetch_add(1, Ordering::Relaxed);
    }

    for (_, (ptr, _)) in cache_entries {
        if should_track {
            let call_stack = vec![execute_cache_worker_workload as *const () as usize];
            track_deallocation_lockfree(ptr, &call_stack)
                .map_err(|e| format!("Cleanup failed: {}", e))?;
        }
    }
    Ok(())
}

fn print_execution_summary(
    success_count: usize,
    error_count: usize,
    total_time: Duration,
    total_operations: usize,
    thread_stats: &Arc<Mutex<Vec<ThreadStats>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 ========== EXECUTION SUMMARY ==========");
    println!(
        "⏱️  Total execution time: {:.2} seconds",
        total_time.as_secs_f64()
    );
    println!("✅ Successful threads: {success_count}");
    println!("❌ Failed threads: {error_count}");
    println!("🔄 Total operations: {total_operations}");
    println!(
        "📈 Operations per second: {:.1}",
        total_operations as f64 / total_time.as_secs_f64()
    );

    if let Ok(stats) = thread_stats.lock() {
        println!("\n📋 ========== WORKLOAD BREAKDOWN ==========");
        let mut workload_summary = HashMap::new();
        for stat in stats.iter() {
            let entry = workload_summary
                .entry(format!("{:?}", stat.workload_type))
                .or_insert((0, 0, 0.0, 0.0, 0));
            entry.0 += 1;
            entry.1 += stat.total_allocations;
            entry.2 += stat.peak_memory_mb;
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
                "   └─ Peak Memory: {memory:.1} MB ({:.1} MB avg)",
                memory / count as f64
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
    Ok(())
}
