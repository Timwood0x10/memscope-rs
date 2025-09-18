//! Pure lock-free demo that completely avoids any global tracker dependencies
//!
//! This demo only uses the lockfree module components and avoids importing
//! anything from the main lib.rs that might trigger global tracker access.

// Only import the specific lockfree components we need
use memscope_rs::lockfree::tracker::{
    init_thread_tracker, track_allocation_lockfree, track_deallocation_lockfree,
    finalize_thread_tracker, SamplingConfig
};
use memscope_rs::lockfree::aggregator::LockfreeAggregator;

use std::thread;
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Pure Lock-free Multi-threaded Demo (30 threads)");
    println!("   Completely avoiding any global tracker dependencies...\n");
    
    let demo_start = Instant::now();
    let output_dir = std::path::PathBuf::from("./Memoryanalysis");
    
    // Clean and setup output directory
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }
    std::fs::create_dir_all(&output_dir)?;
    
    // Configuration for 30 threads with different patterns
    let thread_configs = create_thread_configurations();
    let thread_count = thread_configs.len();
    let total_operations = Arc::new(AtomicUsize::new(0));
    
    println!("🔄 Starting {} worker threads...", thread_count);
    println!("   Expected total operations: ~180,000");
    
    let start_time = Instant::now();
    
    // Spawn threads
    let handles: Vec<_> = thread_configs.into_iter().enumerate()
        .map(|(thread_idx, config)| {
            let output_dir = output_dir.clone();
            let total_operations = Arc::clone(&total_operations);
            
            thread::spawn(move || -> Result<(), String> {
                run_worker_thread(thread_idx, config, &output_dir, &total_operations)
            })
        })
        .collect();
    
    // Monitor progress
    let progress_monitor = {
        let total_operations = Arc::clone(&total_operations);
        thread::spawn(move || {
            let mut last_count = 0;
            for i in 0..120 {
                thread::sleep(Duration::from_millis(500));
                let current = total_operations.load(Ordering::Relaxed);
                if current != last_count {
                    print!(".");
                    if i % 10 == 0 && current > 0 {
                        print!(" {}k ", current / 1000);
                    }
                    std::io::Write::flush(&mut std::io::stdout()).ok();
                    last_count = current;
                } else if i > 20 && current > 0 {
                    break;
                }
            }
            println!();
        })
    };
    
    // Wait for all threads
    let mut successful_threads = 0;
    let mut failed_threads = 0;
    
    for (idx, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(())) => {
                successful_threads += 1;
            }
            Ok(Err(e)) => {
                println!("❌ Thread {} failed: {}", idx, e);
                failed_threads += 1;
            }
            Err(e) => {
                println!("💥 Thread {} panicked: {:?}", idx, e);
                failed_threads += 1;
            }
        }
    }
    
    let _ = progress_monitor.join();
    
    let simulation_duration = start_time.elapsed();
    let final_operations = total_operations.load(Ordering::Relaxed);
    
    println!("\n📊 Pure Lock-free Results:");
    println!("   ✅ Successful threads: {}/{}", successful_threads, thread_count);
    println!("   ❌ Failed threads: {}", failed_threads);
    println!("   🔄 Total operations: {}", final_operations);
    println!("   ⏱️  Duration: {:?}", simulation_duration);
    println!("   🚀 Operations/sec: {:.0}", final_operations as f64 / simulation_duration.as_secs_f64());
    
    // Generate analysis
    if successful_threads > 0 {
        println!("\n🔍 Generating analysis...");
        generate_pure_analysis(&output_dir)?;
    }
    
    let total_duration = demo_start.elapsed();
    println!("\n🎉 Pure lock-free demo completed in {:?}", total_duration);
    println!("📁 Data files: {}", output_dir.display());
    
    if successful_threads == thread_count {
        println!("✨ SUCCESS: 30-thread lock-free tracking works perfectly!");
    } else {
        println!("⚠️  Partial success: {}/{} threads completed", successful_threads, thread_count);
    }
    
    Ok(())
}

/// Thread configuration for different allocation patterns
#[derive(Debug, Clone)]
struct ThreadConfig {
    name: String,
    allocation_sizes: Vec<usize>,
    operation_count: usize,
    sampling_config: SamplingConfig,
    deallocation_frequency: usize, // Every N allocations
}

/// Creates configurations for 30 different threads
fn create_thread_configurations() -> Vec<ThreadConfig> {
    let mut configs = Vec::new();
    
    // 8 Network handler threads (small, frequent allocations)
    for i in 0..8 {
        configs.push(ThreadConfig {
            name: format!("NetworkHandler-{}", i),
            allocation_sizes: vec![64, 128, 256, 512, 768, 1024],
            operation_count: 3000, // 增加工作量
            sampling_config: SamplingConfig::demo(),
            deallocation_frequency: 4,
        });
    }
    
    // 10 Data processor threads (medium allocations)
    for i in 0..10 {
        configs.push(ThreadConfig {
            name: format!("DataProcessor-{}", i),
            allocation_sizes: vec![1024, 2048, 4096, 8192, 16384],
            operation_count: 2000, // 增加工作量
            sampling_config: SamplingConfig::demo(),
            deallocation_frequency: 6,
        });
    }
    
    // 6 Cache manager threads (large allocations with some leaks)
    for i in 0..6 {
        configs.push(ThreadConfig {
            name: format!("CacheManager-{}", i),
            allocation_sizes: vec![16384, 32768, 65536, 131072, 262144],
            operation_count: 800, // 增加工作量
            sampling_config: SamplingConfig::demo(),
            deallocation_frequency: 8, // 减少释放频率以产生一些内存泄漏
        });
    }
    
    // 6 Log writer threads (very small, very frequent)
    for i in 0..6 {
        configs.push(ThreadConfig {
            name: format!("LogWriter-{}", i),
            allocation_sizes: vec![32, 64, 128, 192, 256],
            operation_count: 4000, // 增加工作量
            sampling_config: SamplingConfig::demo(),
            deallocation_frequency: 3,
        });
    }
    
    configs
}

/// Runs a single worker thread with the given configuration
fn run_worker_thread(
    thread_idx: usize,
    config: ThreadConfig,
    output_dir: &std::path::Path,
    total_operations: &Arc<AtomicUsize>
) -> Result<(), String> {
    // Initialize tracker
    init_thread_tracker(output_dir, Some(config.sampling_config))
        .map_err(|e| format!("Thread {} init failed: {}", thread_idx, e))?;
    
    let mut allocated_ptrs = Vec::new();
    let mut operation_count = 0;
    
    // Perform allocations according to config
    for i in 0..config.operation_count {
        let size_idx = i % config.allocation_sizes.len();
        let size = config.allocation_sizes[size_idx];
        
        // Generate REALISTIC memory address - simulate actual heap allocation
        let ptr = thread_idx * 10000 + i * 64 + size_idx;  // Much smaller, realistic range
        
        // Create REAL call stack using actual function pointers
        let call_stack = vec![
            run_worker_thread as *const () as usize,      // Real function address
            create_thread_configurations as *const () as usize, // Real function address  
            main as *const () as usize,                   // Real main function
        ];
        
        // Track allocation
        track_allocation_lockfree(ptr, size, &call_stack)
            .map_err(|e| format!("Thread {} allocation failed: {}", thread_idx, e))?;
        
        allocated_ptrs.push((ptr, call_stack.clone()));
        operation_count += 1;
        total_operations.fetch_add(1, Ordering::Relaxed);
        
        // Deallocate according to pattern
        if i % config.deallocation_frequency == 0 && !allocated_ptrs.is_empty() {
            let (dealloc_ptr, dealloc_stack) = allocated_ptrs.remove(0);
            track_deallocation_lockfree(dealloc_ptr, &dealloc_stack)
                .map_err(|e| format!("Thread {} deallocation failed: {}", thread_idx, e))?;
            
            operation_count += 1;
            total_operations.fetch_add(1, Ordering::Relaxed);
        }
        
        // Add varied timing patterns to simulate realistic workloads
        match thread_idx % 4 {
            0 => { // Network threads: burst patterns
                if i % 200 == 0 {
                    thread::sleep(Duration::from_millis(2));
                }
            }
            1 => { // Data processor threads: steady processing
                if i % 150 == 0 {
                    thread::sleep(Duration::from_millis(1));
                }
            }
            2 => { // Cache threads: occasional heavy work
                if i % 300 == 0 {
                    thread::sleep(Duration::from_millis(3));
                }
            }
            _ => { // Log threads: very frequent, very quick
                if i % 50 == 0 {
                    thread::sleep(Duration::from_nanos(500_000)); // 0.5ms
                }
            }
        }
    }
    
    // REAL cleanup with proper deallocation tracking
    let cleanup_ratio = match config.name.split('-').next().unwrap_or("") {
        "NetworkHandler" => 0.8,   // Network handlers clean up most allocations
        "DataProcessor" => 0.6,    // Data processors keep some data cached  
        "CacheManager" => 0.3,     // Cache managers intentionally leak some memory
        "LogWriter" => 0.9,        // Log writers clean up aggressively
        _ => 0.5,
    };
    
    let cleanup_count = (allocated_ptrs.len() as f64 * cleanup_ratio) as usize;
    let mut deallocated_count = 0;
    
    for (ptr, call_stack) in allocated_ptrs.into_iter().take(cleanup_count) {
        // Track deallocation with REAL call stack
        let dealloc_call_stack = vec![
            run_worker_thread as *const () as usize,
            main as *const () as usize,
        ];
        
        track_deallocation_lockfree(ptr, &dealloc_call_stack)
            .map_err(|e| format!("Thread {} cleanup failed: {}", thread_idx, e))?;
        
        operation_count += 1;
        deallocated_count += 1;
        total_operations.fetch_add(1, Ordering::Relaxed);
    }
    
    let total_allocations = operation_count / 2; // Half operations are allocations, half deallocations
    println!("   Thread {}: {} allocs, {} deallocs, {:.1}% cleanup rate",
             thread_idx, 
             total_allocations,
             deallocated_count,
             if total_allocations > 0 { 
                 deallocated_count as f64 / total_allocations as f64 * 100.0 
             } else { 
                 0.0 
             });
    
    // Finalize tracker
    finalize_thread_tracker()
        .map_err(|e| format!("Thread {} finalize failed: {}", thread_idx, e))?;
    
    println!("   ✓ Thread {} ({}) completed {} operations", 
             thread_idx, config.name, operation_count);
    
    Ok(())
}

/// Generates analysis using only the lockfree aggregator
fn generate_pure_analysis(output_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let aggregator = LockfreeAggregator::new(output_dir.to_path_buf());
    let analysis = aggregator.aggregate_all_threads()?;
    
    println!("📊 Analysis Summary:");
    println!("   📁 Threads analyzed: {}", analysis.thread_stats.len());
    println!("   🔄 Total allocations: {}", analysis.summary.total_allocations);
    println!("   ↩️  Total deallocations: {}", analysis.summary.total_deallocations);
    println!("   📈 Peak memory: {:.2} MB", 
             analysis.summary.peak_memory_usage as f64 / (1024.0 * 1024.0));
    println!("   📊 Unique call stacks: {}", analysis.summary.unique_call_stacks);
    
    // Top threads by allocation
    let mut thread_allocs: Vec<_> = analysis.thread_stats.iter()
        .map(|(id, stats)| (*id, stats.total_allocations))
        .collect();
    thread_allocs.sort_by(|a, b| b.1.cmp(&a.1));
    
    println!("\n🔥 Most Active Threads:");
    for (i, (thread_id, allocs)) in thread_allocs.iter().take(5).enumerate() {
        let stats = &analysis.thread_stats[thread_id];
        println!("   {}. Thread {}: {} allocs, {:.1}KB peak, avg {:.0}B",
                i + 1, thread_id, allocs, 
                stats.peak_memory as f64 / 1024.0,
                stats.avg_allocation_size);
    }
    
    // Hot call stacks
    if !analysis.hottest_call_stacks.is_empty() {
        println!("\n🔥 Hottest Call Stacks:");
        for (i, hot_stack) in analysis.hottest_call_stacks.iter().take(3).enumerate() {
            println!("   {}. Hash 0x{:x}: {} times, {:.1}KB total",
                    i + 1, hot_stack.call_stack_hash, hot_stack.total_frequency,
                    hot_stack.total_size as f64 / 1024.0);
        }
    }
    
    // Generate reports
    let json_path = output_dir.join("pure_lockfree_analysis.json");
    aggregator.export_analysis(&analysis, &json_path)?;
    
    let html_path = output_dir.join("pure_lockfree_report.html");
    aggregator.generate_html_report(&analysis, &html_path)?;
    
    println!("\n📄 Reports generated:");
    println!("   🌐 HTML: {}", html_path.display());
    println!("   📄 JSON: {}", json_path.display());
    
    Ok(())
}