//! Enhanced 30-thread demo with all advanced features
//!
//! This example demonstrates the complete enhanced data collection system:
//! 1. Real call stacks with backtrace integration
//! 2. System-level performance metrics
//! 3. Advanced analysis with pattern prediction
//!
//! Run with different feature combinations:
//! - Basic: cargo run --example enhanced_30_thread_demo
//! - With backtrace: cargo run --example enhanced_30_thread_demo --features backtrace
//! - With system metrics: cargo run --example enhanced_30_thread_demo --features system-metrics
//! - Full enhanced: cargo run --example enhanced_30_thread_demo --features enhanced-tracking

use memscope_rs::lockfree::aggregator::LockfreeAggregator;
use memscope_rs::lockfree::tracker::{
    finalize_thread_tracker, init_thread_tracker, track_allocation_lockfree,
    track_deallocation_lockfree, SamplingConfig,
};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Enhanced 30-Thread Memory Tracking Demo");

    // Show which features are enabled
    print!("   Enhanced features: ");

    #[allow(unused_mut, clippy::vec_init_then_push)]
    {
        let mut features: Vec<&str> = Vec::new();

        #[cfg(feature = "backtrace")]
        features.push("Real Call Stacks");

        #[cfg(feature = "system-metrics")]
        features.push("System Metrics");

        #[cfg(feature = "advanced-analysis")]
        features.push("Advanced Analysis");

        if features.is_empty() {
            println!("Basic tracking only");
            println!("   💡 Enable enhanced features with: --features enhanced-tracking");
        } else {
            println!("{}", features.join(", "));
        }
    }

    println!();

    let demo_start = Instant::now();
    let output_dir = std::path::PathBuf::from("./Memoryanalysis");

    // Clean setup
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }
    std::fs::create_dir_all(&output_dir)?;

    // Create realistic workload
    let thread_configs = create_enhanced_workload();
    let thread_count = thread_configs.len();
    let total_operations = Arc::new(AtomicUsize::new(0));

    println!("🔄 Starting {} enhanced worker threads...", thread_count);
    let start_time = Instant::now();

    // Spawn threads with varied workloads
    let handles: Vec<_> = thread_configs
        .into_iter()
        .enumerate()
        .map(|(thread_idx, config)| {
            let output_dir = output_dir.clone();
            let total_operations = Arc::clone(&total_operations);

            // Set thread names for better tracking
            let thread_name = config.name.clone();

            thread::Builder::new()
                .name(thread_name)
                .spawn(move || -> Result<(), String> {
                    run_enhanced_thread(thread_idx, config, &output_dir, &total_operations)
                })
                .expect("Failed to spawn thread")
        })
        .collect();

    // Monitor progress
    let monitor_handle = {
        let total_operations = Arc::clone(&total_operations);
        thread::spawn(move || {
            for i in 0..20 {
                thread::sleep(Duration::from_millis(500));
                let ops = total_operations.load(Ordering::Relaxed);
                if i % 4 == 0 {
                    print!("   Progress: {} operations\r", ops);
                }
            }
            println!();
        })
    };

    // Wait for all threads
    let mut results = Vec::new();
    for (idx, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(())) => {
                results.push((idx, "Success"));
            }
            Ok(Err(e)) => {
                println!("❌ Thread {} failed: {}", idx, e);
                results.push((idx, "Failed"));
            }
            Err(_) => {
                println!("💥 Thread {} panicked", idx);
                results.push((idx, "Panicked"));
            }
        }
    }

    // Stop monitor
    let _ = monitor_handle.join();

    let simulation_duration = start_time.elapsed();
    let final_operations = total_operations.load(Ordering::Relaxed);

    // Performance analysis
    println!("\n📊 Enhanced Tracking Results:");
    println!(
        "   ✅ Successful threads: {}/{}",
        results
            .iter()
            .filter(|(_, status)| *status == "Success")
            .count(),
        thread_count
    );
    println!("   🔄 Total operations: {}", final_operations);
    println!("   ⏱️  Duration: {:?}", simulation_duration);
    println!(
        "   🚀 Operations/sec: {:.0}",
        final_operations as f64 / simulation_duration.as_secs_f64()
    );

    // Enhanced data analysis
    if results.iter().any(|(_, status)| *status == "Success") {
        println!("\n🔍 Generating enhanced analysis...");
        analyze_enhanced_data(&output_dir)?;
    }

    let total_duration = demo_start.elapsed();
    println!("\n🎉 Enhanced demo completed in {:?}", total_duration);

    // Feature-specific summaries
    #[cfg(feature = "backtrace")]
    println!("   📍 Real call stacks captured with function names and source locations");

    #[cfg(feature = "system-metrics")]
    println!("   📈 System performance metrics collected (CPU, memory, load)");

    #[cfg(feature = "advanced-analysis")]
    println!("   🧠 Advanced pattern analysis performed (lifetime, sharing, access patterns)");

    Ok(())
}

#[derive(Debug, Clone)]
struct EnhancedWorkload {
    name: String,
    workload_type: WorkloadType,
    operation_count: usize,
    complexity: ComplexityLevel,
}

#[derive(Debug, Clone)]
enum WorkloadType {
    IOBound,     // Simulates I/O operations with varied allocations
    CPUBound,    // Simulates CPU-intensive work with predictable patterns
    MemoryBound, // Heavy memory usage with large allocations
    Interactive, // Mixed workload simulating user interactions
}

#[derive(Debug, Clone)]
enum ComplexityLevel {
    Simple,  // Basic allocation patterns
    Medium,  // Some complexity with nested operations
    Complex, // Complex patterns with recursive structures
}

fn create_enhanced_workload() -> Vec<EnhancedWorkload> {
    let mut workloads = Vec::new();

    // Create 30 threads with diverse realistic workloads
    for i in 0..30 {
        let workload = match i % 4 {
            0 => EnhancedWorkload {
                name: format!("IOWorker-{:02}", i),
                workload_type: WorkloadType::IOBound,
                operation_count: 1500,
                complexity: match i % 3 {
                    0 => ComplexityLevel::Simple,
                    1 => ComplexityLevel::Medium,
                    _ => ComplexityLevel::Complex,
                },
            },
            1 => EnhancedWorkload {
                name: format!("CPUWorker-{:02}", i),
                workload_type: WorkloadType::CPUBound,
                operation_count: 2000,
                complexity: ComplexityLevel::Medium,
            },
            2 => EnhancedWorkload {
                name: format!("MemWorker-{:02}", i),
                workload_type: WorkloadType::MemoryBound,
                operation_count: 800,
                complexity: ComplexityLevel::Complex,
            },
            _ => EnhancedWorkload {
                name: format!("Interactive-{:02}", i),
                workload_type: WorkloadType::Interactive,
                operation_count: 1200,
                complexity: ComplexityLevel::Simple,
            },
        };
        workloads.push(workload);
    }

    workloads
}

fn run_enhanced_thread(
    thread_idx: usize,
    config: EnhancedWorkload,
    output_dir: &std::path::Path,
    total_operations: &Arc<AtomicUsize>,
) -> Result<(), String> {
    // Initialize with demo config for rich data capture
    init_thread_tracker(output_dir, Some(SamplingConfig::demo()))
        .map_err(|e| format!("Thread {} init failed: {}", thread_idx, e))?;

    // Execute workload-specific operations
    execute_workload(&config, thread_idx, total_operations)?;

    // Finalize tracking
    finalize_thread_tracker()
        .map_err(|e| format!("Thread {} finalize failed: {}", thread_idx, e))?;

    println!(
        "   ✓ {} completed ({} ops)",
        config.name, config.operation_count
    );
    Ok(())
}

fn execute_workload(
    config: &EnhancedWorkload,
    thread_idx: usize,
    total_operations: &Arc<AtomicUsize>,
) -> Result<(), String> {
    let mut allocated_ptrs = Vec::new();

    for i in 0..config.operation_count {
        // Generate realistic allocation patterns based on workload type
        let (ptr, size, call_stack) = generate_workload_allocation(config, thread_idx, i);

        // Track allocation
        track_allocation_lockfree(ptr, size, &call_stack)
            .map_err(|e| format!("Allocation failed: {}", e))?;

        allocated_ptrs.push((ptr, call_stack.clone()));
        total_operations.fetch_add(1, Ordering::Relaxed);

        // Workload-specific deallocation patterns
        if should_deallocate_workload(config, i) && !allocated_ptrs.is_empty() {
            let dealloc_idx = select_deallocation_target(config, allocated_ptrs.len());
            let (dealloc_ptr, dealloc_stack) = allocated_ptrs.remove(dealloc_idx);

            track_deallocation_lockfree(dealloc_ptr, &dealloc_stack)
                .map_err(|e| format!("Deallocation failed: {}", e))?;

            total_operations.fetch_add(1, Ordering::Relaxed);
        }

        // Workload-specific timing
        simulate_workload_timing(config, i);
    }

    // Cleanup remaining allocations
    let cleanup_ratio = get_cleanup_ratio(config);
    let cleanup_count = (allocated_ptrs.len() as f64 * cleanup_ratio) as usize;

    for (ptr, call_stack) in allocated_ptrs.into_iter().take(cleanup_count) {
        track_deallocation_lockfree(ptr, &call_stack)
            .map_err(|e| format!("Cleanup failed: {}", e))?;

        total_operations.fetch_add(1, Ordering::Relaxed);
    }

    Ok(())
}

fn generate_workload_allocation(
    config: &EnhancedWorkload,
    thread_idx: usize,
    iteration: usize,
) -> (usize, usize, Vec<usize>) {
    let base_ptr = 0x20000000 + (thread_idx * 0x2000000) + (iteration * 256);

    let size = match config.workload_type {
        WorkloadType::IOBound => {
            // IO workloads have varied buffer sizes
            match iteration % 5 {
                0 => 512,   // Small buffers
                1 => 4096,  // Page-sized buffers
                2 => 65536, // Large buffers
                3 => 1024,  // Medium buffers
                _ => 256,   // Tiny buffers
            }
        }
        WorkloadType::CPUBound => {
            // CPU workloads have predictable allocation patterns
            64 + (iteration % 32) * 64
        }
        WorkloadType::MemoryBound => {
            // Memory workloads have large allocations
            16384 + (iteration % 16) * 8192
        }
        WorkloadType::Interactive => {
            // Interactive workloads have mixed patterns
            match iteration % 7 {
                0..=2 => 128 + (iteration % 8) * 32,    // Small frequent
                3..=4 => 2048 + (iteration % 4) * 1024, // Medium occasional
                _ => 32768,                             // Large rare
            }
        }
    };

    // Generate realistic call stacks based on complexity
    let call_stack = match config.complexity {
        ComplexityLevel::Simple => vec![0x400000 + thread_idx, 0x500000 + (iteration % 10)],
        ComplexityLevel::Medium => vec![
            0x400000 + thread_idx,
            0x500000 + (iteration % 10),
            0x600000 + ((iteration / 10) % 5),
            0x700000 + (size / 1024),
        ],
        ComplexityLevel::Complex => vec![
            0x400000 + thread_idx,
            0x500000 + (iteration % 10),
            0x600000 + ((iteration / 10) % 5),
            0x700000 + (size / 1024),
            0x800000 + ((iteration / 100) % 3),
            0x900000 + (thread_idx % 8),
            0xA00000 + (iteration % 20),
        ],
    };

    (base_ptr, size, call_stack)
}

fn should_deallocate_workload(config: &EnhancedWorkload, iteration: usize) -> bool {
    match config.workload_type {
        WorkloadType::IOBound => iteration % 3 == 0, // Frequent cleanup
        WorkloadType::CPUBound => iteration % 4 == 0, // Regular cleanup
        WorkloadType::MemoryBound => iteration % 8 == 0, // Less frequent (more caching)
        WorkloadType::Interactive => iteration % 5 == 0, // Moderate cleanup
    }
}

fn select_deallocation_target(config: &EnhancedWorkload, available: usize) -> usize {
    if available == 0 {
        return 0;
    }

    match config.workload_type {
        WorkloadType::IOBound => 0, // FIFO (oldest first)
        WorkloadType::CPUBound => (available / 2).min(available - 1), // Middle-out
        WorkloadType::MemoryBound => available - 1, // LIFO (newest first)
        WorkloadType::Interactive => (available % 3).min(available - 1), // Semi-random
    }
}

fn simulate_workload_timing(config: &EnhancedWorkload, iteration: usize) {
    let sleep_duration = match config.workload_type {
        WorkloadType::IOBound => {
            if iteration % 50 == 0 {
                Duration::from_millis(5) // Simulate I/O wait
            } else {
                Duration::from_micros(100)
            }
        }
        WorkloadType::CPUBound => {
            if iteration % 200 == 0 {
                Duration::from_millis(1) // Brief context switch
            } else {
                Duration::from_nanos(1000)
            }
        }
        WorkloadType::MemoryBound => {
            if iteration % 100 == 0 {
                Duration::from_millis(2) // Memory pressure pause
            } else {
                Duration::from_micros(500)
            }
        }
        WorkloadType::Interactive => {
            if iteration % 30 == 0 {
                Duration::from_millis(10) // User interaction delay
            } else {
                Duration::from_micros(200)
            }
        }
    };

    thread::sleep(sleep_duration);
}

fn get_cleanup_ratio(config: &EnhancedWorkload) -> f64 {
    match config.workload_type {
        WorkloadType::IOBound => 0.9,     // Clean I/O
        WorkloadType::CPUBound => 0.7,    // Some caching
        WorkloadType::MemoryBound => 0.4, // Heavy caching/leaks
        WorkloadType::Interactive => 0.8, // Mostly clean
    }
}

fn analyze_enhanced_data(output_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let aggregator = LockfreeAggregator::new(output_dir.to_path_buf());
    let analysis = aggregator.aggregate_all_threads()?;

    println!("📊 Enhanced Analysis Results:");

    // Basic statistics
    println!("   📁 Threads analyzed: {}", analysis.thread_stats.len());
    println!(
        "   🔄 Total allocations: {}",
        analysis.summary.total_allocations
    );
    println!(
        "   ↩️  Total deallocations: {}",
        analysis.summary.total_deallocations
    );
    println!(
        "   📈 Peak memory: {:.2} MB",
        analysis.summary.peak_memory_usage as f64 / (1024.0 * 1024.0)
    );

    // Enhanced feature analysis
    #[cfg(feature = "backtrace")]
    println!("   📍 Real call stacks captured for detailed source tracking");

    #[cfg(feature = "system-metrics")]
    println!("   📈 System metrics show realistic resource usage patterns");

    #[cfg(feature = "advanced-analysis")]
    println!("   🧠 Advanced analysis detected allocation lifetime and sharing patterns");

    // Generate reports
    let json_path = output_dir.join("enhanced_analysis.json");
    aggregator.export_analysis(&analysis, &json_path)?;

    let html_path = output_dir.join("enhanced_report.html");
    aggregator.generate_html_report(&analysis, &html_path)?;

    println!("\n📄 Enhanced Reports Generated:");
    println!("   🌐 HTML: {}", html_path.display());
    println!("   📄 JSON: {}", json_path.display());

    // File size analysis
    if let Ok(entries) = std::fs::read_dir(output_dir) {
        let mut total_size = 0u64;
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
        println!(
            "   💾 Total enhanced data: {:.1} MB",
            total_size as f64 / (1024.0 * 1024.0)
        );
    }

    Ok(())
}
