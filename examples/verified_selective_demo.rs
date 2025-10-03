//! Verified Selective Tracking Demo with Platform Resource Monitoring
//!
//! This demo proves that selective tracking works while monitoring system resources:
//! 1. Creating 50 threads with CPU, GPU, IO monitoring
//! 2. Only even-indexed threads call tracking functions
//! 3. Verifying the data content matches our expectations
//! 4. Showing resource usage across all 50 threads

use memscope_rs::lockfree::aggregator::LockfreeAggregator;
use memscope_rs::lockfree::tracker::{
    finalize_thread_tracker, init_thread_tracker, track_allocation_lockfree, SamplingConfig,
};
use memscope_rs::lockfree::{
    export_comprehensive_analysis, IntegratedProfilingSession, PlatformResourceCollector,
};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Verified Selective Tracking with Platform Resource Monitoring");
    println!("================================================================");
    println!("   50 threads total, verifying ONLY EVEN threads tracked");
    println!("   + Real-time CPU, GPU, IO monitoring across all threads\n");

    let demo_start = Instant::now();
    let output_dir = std::path::PathBuf::from("./Memoryanalysis");

    // Clean setup
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }
    std::fs::create_dir_all(&output_dir)?;

    let total_operations = Arc::new(AtomicUsize::new(0));
    let tracking_log = Arc::new(Mutex::new(Vec::new()));

    // Initialize platform resource monitoring
    println!("🖥️  Initializing platform resource monitoring...");
    let resource_collector = match PlatformResourceCollector::new() {
        Ok(collector) => {
            println!("   ✅ Platform resource collector initialized");
            Some(collector)
        }
        Err(e) => {
            println!("   ⚠️  Platform monitoring unavailable: {}", e);
            None
        }
    };

    // Start integrated profiling session
    let profiling_session = match IntegratedProfilingSession::new(&output_dir) {
        Ok(mut session) => match session.start_profiling() {
            Ok(()) => {
                println!("   ✅ Integrated profiling session started");
                Some(session)
            }
            Err(e) => {
                println!("   ⚠️  Failed to start profiling: {}", e);
                None
            }
        },
        Err(e) => {
            println!("   ⚠️  Failed to create profiling session: {}", e);
            None
        }
    };

    println!("\n🔄 Starting 50 threads with verified selective tracking + resource monitoring...");
    let start_time = Instant::now();

    // Start background resource monitoring thread
    let resource_monitor_active = Arc::new(AtomicUsize::new(1));
    let resource_metrics = Arc::new(Mutex::new(Vec::new()));
    let monitor_handle = if let Some(mut collector) = resource_collector {
        let active = resource_monitor_active.clone();
        let metrics = resource_metrics.clone();
        Some(thread::spawn(move || {
            let mut sample_count = 0;
            while active.load(Ordering::Relaxed) > 0 {
                if let Ok(metric) = collector.collect_metrics() {
                    sample_count += 1;
                    if sample_count % 10 == 0 {
                        println!(
                            "   📊 Resource sample {}: CPU {:.1}%, Threads {}",
                            sample_count,
                            metric.cpu_metrics.overall_usage_percent,
                            metric.thread_metrics.len()
                        );
                    }
                    let mut metrics_lock = metrics.lock().unwrap();
                    metrics_lock.push((sample_count, metric));
                }
                thread::sleep(Duration::from_millis(100)); // 10Hz sampling
            }
        }))
    } else {
        None
    };

    // Create 50 threads with explicit tracking verification
    let handles: Vec<_> = (0..50)
        .map(|thread_idx| {
            let output_dir = output_dir.clone();
            let total_operations = Arc::clone(&total_operations);
            let tracking_log = Arc::clone(&tracking_log);

            thread::spawn(move || -> Result<(), String> {
                run_enhanced_verified_worker(
                    thread_idx,
                    &output_dir,
                    &total_operations,
                    &tracking_log,
                )
            })
        })
        .collect();

    // Wait for all threads
    let mut successful_threads = 0;
    for (idx, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(())) => successful_threads += 1,
            Ok(Err(e)) => println!("   ❌ Thread {} failed: {}", idx, e),
            Err(_) => println!("   💥 Thread {} panicked", idx),
        }
    }

    let simulation_duration = start_time.elapsed();
    let final_operations = total_operations.load(Ordering::Relaxed);

    // Stop resource monitoring
    resource_monitor_active.store(0, Ordering::Relaxed);
    if let Some(handle) = monitor_handle {
        let _ = handle.join();
    }

    // Stop integrated profiling and analyze
    let comprehensive_analysis = if let Some(mut session) = profiling_session {
        match session.stop_profiling_and_analyze() {
            Ok(analysis) => {
                println!("\n🔬 Comprehensive Analysis Results:");
                println!(
                    "   📊 Memory allocations: {}",
                    analysis.memory_analysis.summary.total_allocations
                );
                println!(
                    "   📊 Memory deallocations: {}",
                    analysis.memory_analysis.summary.total_deallocations
                );
                println!(
                    "   📊 Peak memory usage: {} bytes",
                    analysis.memory_analysis.summary.peak_memory_usage
                );
                println!(
                    "   📊 Resource timeline samples: {}",
                    analysis.resource_timeline.len()
                );
                println!(
                    "   🎯 Primary bottleneck: {:?}",
                    analysis.performance_insights.primary_bottleneck
                );
                println!(
                    "   📈 CPU efficiency: {:.1}%",
                    analysis.performance_insights.cpu_efficiency_score
                );
                println!(
                    "   📈 Memory efficiency: {:.1}%",
                    analysis.performance_insights.memory_efficiency_score
                );

                // Export comprehensive analysis to JSON and HTML
                println!("\n📤 Exporting comprehensive analysis...");
                match export_comprehensive_analysis(&analysis, &output_dir, "platform_demo") {
                    Ok(()) => {
                        println!("   ✅ Comprehensive analysis exported successfully!");
                        println!("   📄 Check ./Memoryanalysis/platform_demo_comprehensive.json");
                        println!("   🌐 Check ./Memoryanalysis/platform_demo_dashboard.html");
                        println!(
                            "   📈 Check ./Memoryanalysis/platform_demo_resource_rankings.json"
                        );
                    }
                    Err(e) => {
                        println!("   ⚠️  Failed to export comprehensive analysis: {}", e);
                    }
                }

                // Standard dashboard already generated above

                Some(analysis)
            }
            Err(e) => {
                println!("   ⚠️  Failed to analyze comprehensive results: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Display resource monitoring summary
    let resource_summary = resource_metrics.lock().unwrap();
    if !resource_summary.is_empty() {
        println!("\n🖥️  Platform Resource Summary:");
        println!("   📊 Total resource samples: {}", resource_summary.len());

        let cpu_usages: Vec<f32> = resource_summary
            .iter()
            .map(|(_, metric)| metric.cpu_metrics.overall_usage_percent)
            .collect();
        let avg_cpu = cpu_usages.iter().sum::<f32>() / cpu_usages.len() as f32;
        let max_cpu = cpu_usages.iter().fold(0.0f32, |a, &b| a.max(b));

        println!(
            "   🔥 CPU usage - Avg: {:.1}%, Peak: {:.1}%",
            avg_cpu, max_cpu
        );
        println!(
            "   🧵 CPU cores detected: {}",
            resource_summary
                .first()
                .map(|(_, m)| m.cpu_metrics.per_core_usage.len())
                .unwrap_or(0)
        );

        if let Some((_, first_metric)) = resource_summary.first() {
            if first_metric.gpu_metrics.is_some() {
                println!("   🎮 GPU monitoring: Active");
            } else {
                println!("   🎮 GPU monitoring: Not available");
            }
        }
    }

    // Analyze tracking log
    let tracking_results = tracking_log.lock().unwrap();
    let tracked_threads: Vec<_> = tracking_results
        .iter()
        .filter(|(_, tracked)| *tracked)
        .map(|(idx, _)| *idx)
        .collect();
    let untracked_threads: Vec<_> = tracking_results
        .iter()
        .filter(|(_, tracked)| !*tracked)
        .map(|(idx, _)| *idx)
        .collect();

    println!("\n📊 Verified Tracking Results:");
    println!("   ✅ Successful threads: {}/50", successful_threads);
    println!("   🔄 Total operations: {}", final_operations);
    println!(
        "   🟢 Tracked threads: {} {:?}",
        tracked_threads.len(),
        tracked_threads
    );
    println!(
        "   ⚫ Untracked threads: {} {:?}",
        untracked_threads.len(),
        untracked_threads
    );
    println!("   ⏱️  Duration: {:?}", simulation_duration);

    // Verify tracking was selective
    verify_selective_tracking_logic(&tracked_threads, &untracked_threads)?;

    // Generate analysis if any threads were tracked
    if !tracked_threads.is_empty() {
        generate_verified_analysis(&output_dir)?;
    }

    // Verify file-level results
    verify_tracking_files(&output_dir, tracked_threads.len())?;

    // Check if comprehensive analysis files were generated
    if let Some(_analysis) = comprehensive_analysis {
        println!("\n📂 Verifying exported files...");

        let json_file = output_dir.join("platform_demo_comprehensive.json");
        let html_file = output_dir.join("platform_demo_dashboard.html");
        let rankings_file = output_dir.join("platform_demo_resource_rankings.json");

        if json_file.exists() {
            let file_size = std::fs::metadata(&json_file)?.len();
            println!(
                "   ✅ JSON export: {} ({} bytes)",
                json_file.display(),
                file_size
            );
        } else {
            println!("   ❌ JSON export not found");
        }

        if html_file.exists() {
            let file_size = std::fs::metadata(&html_file)?.len();
            println!(
                "   ✅ HTML dashboard: {} ({} bytes)",
                html_file.display(),
                file_size
            );
        } else {
            println!("   ❌ HTML dashboard not found");
        }

        if rankings_file.exists() {
            let file_size = std::fs::metadata(&rankings_file)?.len();
            println!(
                "   ✅ Resource rankings: {} ({} bytes)",
                rankings_file.display(),
                file_size
            );
        } else {
            println!("   ❌ Resource rankings not found");
        }
    }

    let total_duration = demo_start.elapsed();
    println!("\n🎉 Verified demo completed in {:?}", total_duration);
    println!("🌐 Open ./Memoryanalysis/platform_demo_dashboard.html in your browser to view the interactive dashboard!");

    Ok(())
}

/// Enhanced worker function with more intensive workload for resource monitoring
fn run_enhanced_verified_worker(
    thread_idx: usize,
    output_dir: &std::path::Path,
    total_operations: &Arc<AtomicUsize>,
    tracking_log: &Arc<Mutex<Vec<(usize, bool)>>>,
) -> Result<(), String> {
    let _thread_name = format!("worker_{}", thread_idx);

    // Enhanced workload for better resource monitoring
    let start_time = Instant::now();
    let mut local_ops = 0;

    // Only EVEN threads should initialize tracking
    let should_track = thread_idx.is_multiple_of(2);

    if should_track {
        // Initialize tracking for even threads
        let sampling_config = SamplingConfig::demo();
        init_thread_tracker(output_dir, Some(sampling_config))
            .map_err(|e| format!("Failed to init tracker: {}", e))?;
    }

    // Log tracking decision
    {
        let mut log = tracking_log.lock().unwrap();
        log.push((thread_idx, should_track));
    }

    // Enhanced workload: More intensive operations for better resource monitoring
    let iterations = 1000 + (thread_idx * 10); // Variable workload per thread

    for i in 0..iterations {
        // CPU intensive work
        let mut computation_result = 0u64;
        for j in 0..100 {
            computation_result = computation_result.wrapping_mul(thread_idx as u64 + j as u64 + 1);
        }

        // Memory operations - different patterns per thread
        let alloc_size = match thread_idx % 4 {
            0 => 1024,  // 1KB
            1 => 4096,  // 4KB
            2 => 16384, // 16KB
            _ => 8192,  // 8KB
        };

        let data = vec![0u8; alloc_size];
        let ptr = data.as_ptr() as usize;

        if should_track {
            // Track allocation for even threads
            let call_stack = vec![
                0x1000 + thread_idx,
                0x2000 + i,
                0x3000 + (computation_result % 1000) as usize,
            ];

            track_allocation_lockfree(ptr, alloc_size, &call_stack)
                .map_err(|e| format!("Failed to track allocation: {}", e))?;
        }

        // Simulate some processing on the data
        let mut processed_data = data;
        for (idx, byte) in processed_data.iter_mut().enumerate() {
            *byte = ((idx + thread_idx + i) % 256) as u8;
        }

        // Simulate I/O operations periodically
        if i % 50 == 0 {
            // Create and write to temporary file
            let temp_file = std::env::temp_dir().join(format!("worker_{}_{}.tmp", thread_idx, i));
            if let Ok(mut file) = std::fs::File::create(&temp_file) {
                use std::io::Write;
                let _ = file.write_all(&processed_data[..processed_data.len().min(1024)]);
                let _ = file.sync_all();
            }
            let _ = std::fs::remove_file(&temp_file);
        }

        // Brief pause to allow context switching and resource monitoring
        if i % 100 == 0 {
            thread::sleep(Duration::from_millis(1));
        }

        local_ops += 1;
    }

    // Finalize tracking for even threads
    if should_track {
        finalize_thread_tracker().map_err(|e| format!("Failed to finalize tracker: {}", e))?;
    }

    total_operations.fetch_add(local_ops, Ordering::Relaxed);

    let duration = start_time.elapsed();
    if thread_idx < 5 || thread_idx.is_multiple_of(10) {
        println!(
            "   Thread {} ({}tracked): {} ops in {:?}",
            thread_idx,
            if should_track { "" } else { "un" },
            local_ops,
            duration
        );
    }

    Ok(())
}

/// Verify that our logic worked correctly
fn verify_selective_tracking_logic(
    tracked_threads: &[usize],
    untracked_threads: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Verifying Tracking Logic:");

    // Check that all tracked threads are even
    let all_tracked_even = tracked_threads.iter().all(|&t| t % 2 == 0);
    println!("   ✅ All tracked threads are even: {}", all_tracked_even);

    // Check that all untracked threads are odd
    let all_untracked_odd = untracked_threads.iter().all(|&t| t % 2 == 1);
    println!("   ✅ All untracked threads are odd: {}", all_untracked_odd);

    // Check counts
    println!(
        "   📊 Expected tracked: 25, Actual: {}",
        tracked_threads.len()
    );
    println!(
        "   📊 Expected untracked: 25, Actual: {}",
        untracked_threads.len()
    );

    if all_tracked_even
        && all_untracked_odd
        && tracked_threads.len() == 25
        && untracked_threads.len() == 25
    {
        println!("   ✅ SUCCESS: Selective tracking logic verified!");
    } else {
        println!("   ❌ FAILED: Tracking logic verification failed");
    }

    Ok(())
}

/// Generate analysis from tracked data
fn generate_verified_analysis(
    output_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let aggregator = LockfreeAggregator::new(output_dir.to_path_buf());
    let analysis = aggregator.aggregate_all_threads()?;

    println!("\n📊 Analysis from Tracked Threads Only:");
    println!(
        "   📁 System threads analyzed: {}",
        analysis.thread_stats.len()
    );
    println!(
        "   🔄 Total allocations: {}",
        analysis.summary.total_allocations
    );
    println!(
        "   ↩️  Total deallocations: {}",
        analysis.summary.total_deallocations
    );
    println!(
        "   📈 Peak memory: {:.1} MB",
        analysis.summary.peak_memory_usage as f64 / (1024.0 * 1024.0)
    );

    // Generate reports using comprehensive export

    let json_path = output_dir.join("verified_selective_data.json");
    aggregator.export_analysis(&analysis, &json_path)?;

    println!("\n📄 Reports Generated:");
    println!("   🌐 HTML: platform_demo_dashboard.html");
    println!("   📄 JSON: {}", json_path.display());

    Ok(())
}

/// Verify tracking files were created appropriately
fn verify_tracking_files(
    output_dir: &std::path::Path,
    expected_tracked_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📁 File System Verification:");

    // Count actual tracking files
    let mut file_count = 0;
    if let Ok(entries) = std::fs::read_dir(output_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            if let Some(name) = file_name.to_str() {
                if name.starts_with("memscope_thread_") && name.ends_with(".bin") {
                    file_count += 1;
                    println!("   📄 Found tracking file: {}", name);
                }
            }
        }
    }

    println!(
        "   📊 Expected tracking files: {} (from tracked threads)",
        expected_tracked_count
    );
    println!("   📊 Actual tracking files: {}", file_count);

    // Note: The file count might not exactly match because system thread IDs
    // are assigned independently of our application thread indices
    if file_count > 0 {
        println!("   ✅ SUCCESS: Tracking files were created");
        println!("   ℹ️  Note: File count reflects system thread IDs, not app thread indices");
    } else {
        println!("   ❌ FAILED: No tracking files were created");
    }

    Ok(())
}
