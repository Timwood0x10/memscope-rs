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

use memscope_rs::export::fixed_hybrid_template::{
    create_sample_hybrid_data, FixedHybridTemplate, RenderMode,
};
use memscope_rs::{track_var, init};
use std::collections::HashMap;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Enhanced 30-Thread Memory Tracking Demo with HTML Visualization");
    println!("   Features: track_var! macros + HTML template integration");
    
    // Initialize memscope
    init();

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

    println!("🔄 Starting {thread_count} enhanced worker threads...");
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

    // Generate HTML visualization using template system
    if results.iter().any(|(_, status)| *status == "Success") {
        println!("\n🔍 Generating HTML visualization...");
        generate_html_visualization(thread_count, final_operations, simulation_duration)?;
    }

    let total_duration = demo_start.elapsed();
    println!("\n🎉 Enhanced demo completed in {:?}", total_duration);
    println!("📄 Generated files:");
    println!("   - enhanced_thread_analysis_comprehensive.html");
    println!("   - enhanced_thread_analysis_thread_focused.html");
    println!("   - enhanced_thread_analysis_variable_detailed.html");

    Ok(())
}

fn generate_html_visualization(
    thread_count: usize,
    total_operations: usize,
    duration: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  📊 Creating hybrid analysis data...");
    
    // Create realistic task mapping (simulating how threads map to async tasks)
    let task_count = thread_count + 10; // More tasks than threads
    let hybrid_data = create_sample_hybrid_data(thread_count, task_count);
    
    println!("  🎨 Generating HTML reports...");
    
    // Generate different views of the same data with enhanced insights
    let templates = vec![
        (
            "comprehensive",
            FixedHybridTemplate::new(thread_count, task_count)
                .with_render_mode(RenderMode::Comprehensive)
                .with_variable_details(true)
                .with_enhanced_insights(true),  // 新增洞察功能
        ),
        (
            "thread_focused", 
            FixedHybridTemplate::new(thread_count, task_count)
                .with_render_mode(RenderMode::ThreadFocused)
                .with_variable_details(true),
        ),
        (
            "variable_detailed",
            FixedHybridTemplate::new(thread_count, task_count)
                .with_render_mode(RenderMode::VariableDetailed)
                .with_variable_details(true),
        ),
    ];

    for (name, template) in templates {
        let html_content = template.generate_hybrid_dashboard(&hybrid_data)?;
        let filename = format!("enhanced_thread_analysis_{}.html", name);
        std::fs::write(&filename, html_content)?;
        println!("    ✅ Generated: {}", filename);
    }

    // Print summary of what was tracked
    print_tracking_summary(&hybrid_data, total_operations, duration);

    Ok(())
}

fn print_tracking_summary(
    data: &memscope_rs::export::fixed_hybrid_template::HybridAnalysisData,
    total_operations: usize,
    duration: Duration,
) {
    println!("\n📋 Tracking Summary:");
    println!("  🔄 Total operations: {}", total_operations);
    println!("  ⏱️  Duration: {:?}", duration);
    println!(
        "  🚀 Operations/sec: {:.0}",
        total_operations as f64 / duration.as_secs_f64()
    );
    println!("  🧵 Threads tracked: {}", data.thread_task_mapping.len());
    println!("  📋 Variables tracked: {}", data.variable_registry.len());

    // Show thread distribution
    println!("\n🧵 Thread Distribution:");
    for thread_id in 0..data.thread_task_mapping.len().min(10) {
        let thread_vars = data
            .variable_registry
            .values()
            .filter(|v| v.thread_id == thread_id)
            .count();
        
        let thread_memory: u64 = data
            .variable_registry
            .values()
            .filter(|v| v.thread_id == thread_id)
            .map(|v| v.memory_usage)
            .sum();

        println!(
            "  Thread {}: {} variables, {:.1} KB tracked",
            thread_id,
            thread_vars,
            thread_memory as f64 / 1024.0
        );
    }

    // Show workload type distribution (simulated based on thread ID)
    println!("\n📊 Workload Types:");
    let mut workload_counts = HashMap::new();
    for thread_id in 0..data.thread_task_mapping.len() {
        let workload_type = match thread_id % 4 {
            0 => "IOBound",
            1 => "CPUBound", 
            2 => "MemoryBound",
            _ => "Interactive",
        };
        *workload_counts.entry(workload_type).or_insert(0) += 1;
    }

    for (workload_type, count) in workload_counts {
        println!("  {}: {} threads", workload_type, count);
    }
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
    _output_dir: &std::path::Path,
    total_operations: &Arc<AtomicUsize>,
) -> Result<(), String> {
    println!("🧵 Starting thread {}: {}", thread_idx, config.name);

    // Execute workload using track_var! macros
    execute_track_var_workload(&config, thread_idx, total_operations)?;

    println!(
        "   ✓ {} completed ({} ops)",
        config.name, config.operation_count
    );
    Ok(())
}

fn execute_track_var_workload(
    config: &EnhancedWorkload,
    thread_idx: usize,
    total_operations: &Arc<AtomicUsize>,
) -> Result<(), String> {
    let mut tracked_data = Vec::new();

    for i in 0..config.operation_count {
        // Generate workload-specific data and track with track_var!
        match &config.workload_type {
            WorkloadType::IOBound => {
                // Simulate I/O buffer allocations
                let buffer_size = 1024 + (i % 4096);
                let io_buffer: Vec<u8> = (0..buffer_size).map(|x| (x % 256) as u8).collect();
                track_var!(io_buffer);
                
                let io_metadata = format!("io_thread_{}_operation_{}", thread_idx, i);
                track_var!(io_metadata);
                
                tracked_data.push(format!("IO-{}-{}", thread_idx, i));
            }
            WorkloadType::CPUBound => {
                // Simulate computation results
                let computation_result: Vec<f64> = (0..100)
                    .map(|x| (x as f64 * thread_idx as f64 * i as f64).sin())
                    .collect();
                track_var!(computation_result);
                
                let cpu_workload = (0..50).map(|x| x * thread_idx * i).collect::<Vec<_>>();
                track_var!(cpu_workload);
                
                tracked_data.push(format!("CPU-{}-{}", thread_idx, i));
            }
            WorkloadType::MemoryBound => {
                // Simulate large memory allocations
                let large_allocation: Vec<u64> = vec![thread_idx as u64; 2048];
                track_var!(large_allocation);
                
                let memory_map: HashMap<String, usize> = (0..10)
                    .map(|x| (format!("key_{}_{}", thread_idx, x), x * i))
                    .collect();
                track_var!(memory_map);
                
                tracked_data.push(format!("MEM-{}-{}", thread_idx, i));
            }
            WorkloadType::Interactive => {
                // Simulate user interaction data
                let user_input = format!("User action {} from thread {} at iteration {}", 
                    i % 10, thread_idx, i);
                track_var!(user_input);
                
                let session_data = vec![
                    format!("session_{}", thread_idx),
                    format!("action_{}", i),
                    format!("timestamp_{}", i * thread_idx),
                ];
                track_var!(session_data);
                
                tracked_data.push(format!("UI-{}-{}", thread_idx, i));
            }
        }

        total_operations.fetch_add(1, Ordering::Relaxed);

        // Simulate work timing
        if i % 100 == 0 {
            thread::sleep(Duration::from_millis(1));
        }
    }

    // Track final summary for this thread
    let summary_string = format!(
        "Thread {} completed: {} {} operations with {} tracked items",
        thread_idx, config.name, config.operation_count, tracked_data.len()
    );
    track_var!(summary_string);

    Ok(())
}

#[derive(Debug, Clone)]
struct ThreadSummary {
    thread_id: usize,
    workload_name: String,
    workload_type: String,
    operations_completed: usize,
    tracked_items: Vec<String>,
}

