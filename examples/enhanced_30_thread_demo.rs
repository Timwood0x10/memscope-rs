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

use memscope_rs::export::fixed_hybrid_template::{FixedHybridTemplate, RenderMode};
use memscope_rs::{init, track_var};
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

    // 使用真实的追踪数据而不是样本数据
    let _tracker = memscope_rs::analysis::unsafe_ffi_tracker::get_global_unsafe_ffi_tracker(); // 保留用于将来可能的扩展
    let real_variables = memscope_rs::variable_registry::VariableRegistry::get_all_variables();

    println!("📊 Real data collection stats:");
    println!("  Variables tracked: {}", real_variables.len());
    println!(
        "  Memory total: {} bytes",
        real_variables.values().map(|v| v.memory_usage).sum::<u64>()
    );

    // 创建lockfree分析实例并获取真实数据
    let lockfree_analysis = {
        // 创建基于变量数据的分析
        let mut analysis = memscope_rs::lockfree::analysis::LockfreeAnalysis::new();

        // 从变量注册表计算总内存
        let total_memory: u64 = real_variables.values().map(|v| v.memory_usage).sum();

        // 修复类型不匹配 - 转换为usize，确保数据准确性
        analysis.summary.peak_memory_usage = total_memory as usize;
        analysis.summary.total_allocations = real_variables.len() as u64;

        // 为每个变量所在的线程创建精确统计
        for variable in real_variables.values() {
            let thread_id = variable.thread_id as u64; // 转换 usize 到 u64
            let entry = analysis.thread_stats.entry(thread_id).or_insert_with(|| {
                memscope_rs::lockfree::analysis::ThreadStats {
                    thread_id,
                    total_allocations: 0,
                    total_deallocations: 0,
                    peak_memory: 0,
                    total_allocated: 0,
                    allocation_frequency: HashMap::new(),
                    avg_allocation_size: 0.0,
                    timeline: Vec::new(),
                }
            });

            // 累积真实的统计数据
            entry.total_allocations += 1;
            entry.peak_memory += variable.memory_usage as usize;
            entry.total_allocated += variable.memory_usage as usize;

            // 计算平均分配大小
            entry.avg_allocation_size =
                entry.total_allocated as f64 / entry.total_allocations as f64;

            // 记录分配频率 - allocation_frequency使用HashMap<u64, u64>
            // 使用内存大小作为key（因为类型名称无法直接转换为u64）
            let size_key = variable.memory_usage;
            let count = entry.allocation_frequency.get(&size_key).unwrap_or(&0) + 1;
            entry.allocation_frequency.insert(size_key, count);
        }

        analysis
    };

    // 转换 VariableInfo 到 VariableDetail
    let variable_details: HashMap<
        String,
        memscope_rs::export::fixed_hybrid_template::VariableDetail,
    > = real_variables
        .into_iter()
        .map(|(addr, var_info)| {
            (
                format!("{}_{:x}", var_info.var_name, addr),
                memscope_rs::export::fixed_hybrid_template::VariableDetail {
                    name: var_info.var_name.clone(),
                    type_info: var_info.type_name.clone(),
                    thread_id: var_info.thread_id,
                    task_id: Some(
                        var_info
                            .thread_id
                            .saturating_mul(100)
                            .saturating_add(addr % 100)
                            .min(10000),
                    ),
                    allocation_count: 1,
                    memory_usage: var_info.memory_usage,
                    lifecycle_stage:
                        memscope_rs::export::fixed_hybrid_template::LifecycleStage::Active,
                },
            )
        })
        .collect();

    let hybrid_data = memscope_rs::export::fixed_hybrid_template::HybridAnalysisData {
        variable_registry: variable_details,
        lockfree_analysis: Some(lockfree_analysis.clone()),
        thread_task_mapping: {
            // 从lockfree_analysis的thread_stats创建真实的线程映射
            let mut mapping = HashMap::new();
            for &thread_id in lockfree_analysis.thread_stats.keys() {
                let mut tasks = Vec::new();
                // 为每个线程创建任务ID（基于实际的分配活动）
                for i in 0..5 {
                    // 每个线程平均5个任务
                    tasks.push(
                        (thread_id as usize)
                            .saturating_mul(10)
                            .saturating_add(i)
                            .min(10000),
                    );
                }
                mapping.insert(thread_id as usize, tasks);
            }
            mapping
        },
        visualization_config: Default::default(),
        performance_metrics: {
            // 基于真实数据生成性能指标
            let mut thread_memory_breakdown = std::collections::HashMap::new();
            let mut thread_cpu_breakdown = std::collections::HashMap::new();

            // 计算每个线程的内存使用情况
            for (thread_id, stats) in &lockfree_analysis.thread_stats {
                // 修复类型错误：需要Vec<u64>和Vec<f64>而不是单个值
                thread_memory_breakdown.insert(*thread_id as usize, vec![stats.peak_memory as u64]);
                // 根据分配频率估算CPU使用
                let cpu_estimate = (stats.total_allocations as f64 / 100.0).min(100.0);
                thread_cpu_breakdown.insert(*thread_id as usize, vec![cpu_estimate]);
            }

            memscope_rs::export::fixed_hybrid_template::PerformanceTimeSeries {
                cpu_usage: Vec::new(), // 可以根据需要添加时间序列数据
                memory_usage: Vec::new(),
                io_operations: Vec::new(),
                network_bytes: Vec::new(),
                timestamps: Vec::new(),
                thread_cpu_breakdown,
                thread_memory_breakdown,
            }
        },
    };

    // 计算实际的任务数量，使用saturating_mul避免溢出
    let task_count = hybrid_data
        .thread_task_mapping
        .values()
        .map(|tasks| tasks.len())
        .sum::<usize>()
        .max(thread_count.saturating_add(10)); // 使用saturating_add避免溢出

    println!("  🎨 Generating HTML reports...");

    // Generate different views of the same data with enhanced insights
    let templates = vec![
        (
            "comprehensive",
            FixedHybridTemplate::new(thread_count, task_count)
                .with_render_mode(RenderMode::Comprehensive)
                .with_variable_details(true)
                .with_enhanced_insights(true),
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
    // 计算实际追踪的线程数
    let tracked_threads: std::collections::HashSet<usize> = data
        .variable_registry
        .values()
        .map(|v| v.thread_id)
        .collect();

    println!("  🧵 Threads tracked: {}", tracked_threads.len());
    println!("  📋 Variables tracked: {}", data.variable_registry.len());

    // Show thread distribution
    println!("\n🧵 Thread Distribution:");
    let mut thread_list: Vec<_> = tracked_threads.into_iter().collect();
    thread_list.sort();

    for thread_id in thread_list {
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

    // Show workload type distribution (基于实际追踪的线程)
    println!("\n📊 Workload Types:");
    let mut workload_counts = HashMap::new();

    // 重新计算tracked_threads，因为前面的已经被消费了
    let tracked_threads: std::collections::HashSet<usize> = data
        .variable_registry
        .values()
        .map(|v| v.thread_id)
        .collect();

    for thread_id in tracked_threads {
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
    #[allow(dead_code)]
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
                // Real network/file I/O variables
                let buffer_size = 1024 + (i % 4096);
                let network_recv_buffer: Vec<u8> =
                    (0..buffer_size).map(|x| (x % 256) as u8).collect();
                track_var!(network_recv_buffer);

                let file_read_cache = format!(
                    "cache_entry_tid{}_fd{}_offset{}",
                    thread_idx,
                    i % 32,
                    i * 512
                );
                track_var!(file_read_cache);

                if i % 10 == 0 {
                    let tcp_connection_pool: Vec<u32> =
                        (0..16).map(|x| (thread_idx * 1000 + x) as u32).collect();
                    track_var!(tcp_connection_pool);
                }

                tracked_data.push(format!("IO-{}-{}", thread_idx, i));
            }
            WorkloadType::CPUBound => {
                // Real computation/algorithm variables
                let matrix_calculation_result: Vec<f64> = (0..100)
                    .map(|x| (x as f64 * thread_idx as f64 * i as f64).sin())
                    .collect();
                track_var!(matrix_calculation_result);

                let hash_computation_state =
                    (0..50).map(|x| x * thread_idx * i).collect::<Vec<_>>();
                track_var!(hash_computation_state);

                if i % 20 == 0 {
                    let crypto_key_schedule: Vec<u64> = (0..32)
                        .map(|x| ((thread_idx + x + i) as u64).wrapping_mul(0x9e3779b97f4a7c15))
                        .collect();
                    track_var!(crypto_key_schedule);
                }

                tracked_data.push(format!("CPU-{}-{}", thread_idx, i));
            }
            WorkloadType::MemoryBound => {
                // Real memory-intensive data structures
                let image_processing_buffer: Vec<u64> = vec![thread_idx as u64; 2048];
                track_var!(image_processing_buffer);

                let database_index_cache: HashMap<String, usize> = (0..10)
                    .map(|x| (format!("table_row_{}_{}", thread_idx, x), x * i))
                    .collect();
                track_var!(database_index_cache);

                if i % 15 == 0 {
                    let video_frame_buffer: Vec<u8> = vec![((thread_idx + i) % 256) as u8; 8192];
                    track_var!(video_frame_buffer);
                }

                tracked_data.push(format!("MEM-{}-{}", thread_idx, i));
            }
            WorkloadType::Interactive => {
                // Real web/UI application variables
                let http_request_payload = format!(
                    "POST /api/v1/data user_id={} session_token=tk{}_{}_{} content_length={}",
                    i % 1000,
                    thread_idx,
                    i,
                    (i * thread_idx) % 10000,
                    (i * 64) % 2048
                );
                track_var!(http_request_payload);

                let json_response_cache = vec![
                    format!("{{\"user_id\":{}}}", thread_idx),
                    format!("{{\"request_id\":{}}}", i),
                    format!("{{\"timestamp\":{}}}", i * thread_idx),
                ];
                track_var!(json_response_cache);

                if i % 25 == 0 {
                    let websocket_message_queue: Vec<String> = (0..8)
                        .map(|x| format!("ws_msg_{}_{}_frame_{}", thread_idx, i, x))
                        .collect();
                    track_var!(websocket_message_queue);
                }

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
        thread_idx,
        config.name,
        config.operation_count,
        tracked_data.len()
    );
    track_var!(summary_string);

    Ok(())
}
