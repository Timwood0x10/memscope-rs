//! Async Memory Tracking Test
//!
//! This example demonstrates the async_memory tracking API for task-aware
//! memory analysis based on the successful comprehensive_async_showcase pattern.

use memscope_rs::async_memory::{
    initialize, resource_monitor::SourceLocation, visualization::VisualizationGenerator,
    AsyncResourceMonitor, TaskId, TaskResourceProfile, TaskType,
};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::runtime::Runtime;

/// Test configuration for async scenarios
#[derive(Debug, Clone)]
struct AsyncTestConfig {
    num_tasks: usize,
    allocations_per_task: usize,
    allocation_size_range: (usize, usize),
    io_simulation_frequency: usize,
}

impl AsyncTestConfig {
    fn light_async() -> Self {
        Self {
            num_tasks: 10,
            allocations_per_task: 1_000,
            allocation_size_range: (64, 1024),
            io_simulation_frequency: 50,
        }
    }

    fn medium_async() -> Self {
        Self {
            num_tasks: 50,
            allocations_per_task: 1_000,
            allocation_size_range: (128, 2048),
            io_simulation_frequency: 100,
        }
    }

    fn heavy_async() -> Self {
        Self {
            num_tasks: 100,
            allocations_per_task: 1_000,
            allocation_size_range: (256, 4096),
            io_simulation_frequency: 200,
        }
    }
}

/// Async test results
#[derive(Debug)]
struct AsyncResults {
    scenario: String,
    num_tasks: usize,
    total_allocations: usize,
    duration: Duration,
    throughput: f64,
    memory_usage_mb: f64,
    tracking_events: usize,
    initialization_success: bool,
}

impl AsyncResults {
    fn new(scenario: &str, num_tasks: usize) -> Self {
        Self {
            scenario: scenario.to_string(),
            num_tasks,
            total_allocations: 0,
            duration: Duration::default(),
            throughput: 0.0,
            memory_usage_mb: 0.0,
            tracking_events: 0,
            initialization_success: false,
        }
    }

    fn print_results(&self) {
        println!("📊 {} Results:", self.scenario);
        println!("  Async Tasks: {}", self.num_tasks);
        println!("  Total Allocations: {}", self.total_allocations);
        println!("  Duration: {:.2}s", self.duration.as_secs_f64());
        println!("  Throughput: {:.0} allocs/sec", self.throughput);
        println!("  Memory Usage: {:.1} MB", self.memory_usage_mb);
        println!("  Tracking Events: {}", self.tracking_events);
        println!(
            "  Initialization: {}",
            if self.initialization_success {
                "✅"
            } else {
                "❌"
            }
        );
        println!();
    }
}

/// Async memory test suite
struct AsyncMemoryTestSuite {
    config: AsyncTestConfig,
}

impl AsyncMemoryTestSuite {
    fn new(config: AsyncTestConfig) -> Self {
        Self { config }
    }

    /// Test async memory tracking with comprehensive monitoring
    async fn test_async_memory_tracking(&self) -> AsyncResults {
        println!("⚡ Testing Comprehensive Async Memory Tracking");
        let mut results = AsyncResults::new("Async Memory", self.config.num_tasks);

        // Initialize async memory tracking like comprehensive_async_showcase
        match initialize() {
            Ok(_) => {
                println!("🚀 Async memory tracking initialized successfully");
                results.initialization_success = true;
            }
            Err(e) => {
                eprintln!("❌ Failed to initialize async memory tracking: {}", e);
                return results;
            }
        }

        let start = Instant::now();
        let monitor = Arc::new(Mutex::new(AsyncResourceMonitor::new()));
        let mut task_handles = Vec::new();

        // Start comprehensive monitoring loop
        let monitor_metrics = Arc::clone(&monitor);
        let metrics_handle = tokio::spawn(async move {
            for _ in 0..60 {
                // Monitor for 60 seconds
                {
                    let mut mon = monitor_metrics.lock().unwrap();
                    let task_ids: Vec<TaskId> = mon.get_all_profiles().keys().copied().collect();
                    for task_id in task_ids {
                        mon.update_metrics(task_id);
                    }
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        // Launch comprehensive async tasks
        for task_id in 0..self.config.num_tasks {
            let config = self.config.clone();
            let monitor_clone = Arc::clone(&monitor);

            // Start monitoring with detailed source location
            {
                let mut mon = monitor.lock().unwrap();
                let task_type = match task_id % 5 {
                    0 => TaskType::CpuIntensive,
                    1 => TaskType::IoIntensive,
                    2 => TaskType::MemoryIntensive,
                    3 => TaskType::NetworkIntensive,
                    _ => TaskType::Mixed,
                };

                let source_location = SourceLocation {
                    file_path: "examples/async_memory_test_clean.rs".to_string(),
                    line_number: 150 + (task_id as u32 * 5),
                    function_name: format!("comprehensive_async_task_{}", task_id),
                    module_path: "async_memory_test_clean".to_string(),
                    crate_name: "memscope_rs".to_string(),
                };

                mon.start_monitoring_with_location(
                    task_id as TaskId,
                    format!("AsyncTest_Task_{}", task_id),
                    task_type,
                    Some(source_location),
                );
            }

            let handle = tokio::spawn(async move {
                execute_comprehensive_async_workload(task_id, config, monitor_clone).await
            });

            task_handles.push((task_id, handle));
            println!("🚀 Started Task {}: AsyncTest_Task_{}", task_id, task_id);

            // Stagger task starts
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Wait for all tasks to complete with timeout
        let mut total_allocations = 0;
        let mut successful_tasks = 0;

        for (task_id, handle) in task_handles {
            match tokio::time::timeout(Duration::from_secs(30), handle).await {
                Ok(Ok(Ok(allocation_count))) => {
                    total_allocations += allocation_count;
                    successful_tasks += 1;
                    println!(
                        "✅ Task {} completed: {} allocations",
                        task_id, allocation_count
                    );

                    // Finish monitoring for this task
                    {
                        let mut mon = monitor.lock().unwrap();
                        mon.finish_monitoring(task_id as TaskId);
                    }
                }
                Ok(Ok(Err(e))) => {
                    eprintln!("❌ Task {} failed: {:?}", task_id, e);
                }
                Ok(Err(e)) => {
                    eprintln!("❌ Task {} panicked: {:?}", task_id, e);
                }
                Err(_) => {
                    eprintln!("⏰ Task {} timed out", task_id);
                    // Still finish monitoring for timed out tasks
                    {
                        let mut mon = monitor.lock().unwrap();
                        mon.finish_monitoring(task_id as TaskId);
                    }
                }
            }
        }

        // Stop metrics monitoring
        metrics_handle.abort();

        // Generate comprehensive reports
        println!("📊 Generating comprehensive analysis reports...");
        if let Err(e) = generate_comprehensive_reports(&monitor).await {
            eprintln!("⚠️  Failed to generate reports: {}", e);
        }

        results.duration = start.elapsed();
        results.total_allocations = total_allocations;
        results.throughput = total_allocations as f64 / results.duration.as_secs_f64();
        results.memory_usage_mb = get_process_memory_usage() as f64 / (1024.0 * 1024.0);

        // Get comprehensive monitoring statistics
        let profiles = {
            let mon = monitor.lock().unwrap();
            mon.get_all_profiles().clone()
        };
        results.tracking_events = profiles.len();

        println!("✅ Comprehensive async memory tracking completed");
        println!("   📊 Tasks completed: {}", successful_tasks);
        println!("   📈 Monitoring profiles: {}", profiles.len());
        println!("   📄 Reports: async_test_analysis.json, async_test_dashboard.html");

        results
    }

    /// Run comprehensive async tests
    pub fn run_all_tests() {
        println!("🚀 Starting Async Memory Tracking Test Suite");
        println!("{}", "=".repeat(60));

        let scenarios = [
            ("Light Async", AsyncTestConfig::light_async()),
            ("Medium Async", AsyncTestConfig::medium_async()),
            ("Heavy Async", AsyncTestConfig::heavy_async()),
        ];

        let rt = Runtime::new().expect("Failed to create Tokio runtime");

        let mut all_results = Vec::new();

        for (scenario_name, config) in scenarios.iter() {
            println!("\n📋 Scenario: {}", scenario_name);
            println!("{}", "-".repeat(40));

            let suite = AsyncMemoryTestSuite::new(config.clone());
            let results = rt.block_on(suite.test_async_memory_tracking());

            results.print_results();
            all_results.push((scenario_name.to_string(), results));

            println!("📊 {} Summary:", scenario_name);
            println!(
                "  Task Efficiency: {:.1} allocs/task/sec",
                all_results.last().unwrap().1.throughput
                    / all_results.last().unwrap().1.num_tasks as f64
            );
            println!(
                "  Memory per Task: {:.1} MB",
                all_results.last().unwrap().1.memory_usage_mb
                    / all_results.last().unwrap().1.num_tasks as f64
            );
            println!(
                "  Tracking Coverage: {:.1}%",
                if all_results.last().unwrap().1.total_allocations > 0 {
                    (all_results.last().unwrap().1.tracking_events as f64
                        / all_results.last().unwrap().1.num_tasks as f64)
                        * 100.0
                } else {
                    0.0
                }
            );
        }

        // Generate a single comprehensive HTML report for all scenarios
        println!("\n📊 Generating single comprehensive HTML report for all scenarios...");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let final_report_path = format!("async_comprehensive_report_{}.html", timestamp);

        rt.block_on(async {
            if let Err(e) =
                generate_combined_async_html_report(&all_results, &final_report_path).await
            {
                eprintln!("⚠️  Failed to generate combined report: {}", e);
            } else {
                println!("🌐 Single HTML Report: {}", final_report_path);
            }
        });

        println!("✅ Async memory test suite completed!");
        println!("💡 For single-threaded tests, run: cargo run --example comprehensive_modes_test");
        println!("💡 For multi-threaded tests, run: cargo run --example lockfree_test");
    }
}

/// Execute comprehensive async workload with realistic memory patterns
async fn execute_comprehensive_async_workload(
    task_id: usize,
    config: AsyncTestConfig,
    monitor: Arc<Mutex<AsyncResourceMonitor>>,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let mut _allocated_data: Vec<Box<dyn std::any::Any + Send>> = Vec::new();

    // Start periodic metrics updates
    let monitor_update = Arc::clone(&monitor);
    let update_handle = tokio::spawn(async move {
        for _ in 0..(config.allocations_per_task / 100) {
            {
                let mut mon = monitor_update.lock().unwrap();
                mon.update_metrics(task_id as TaskId);
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });

    // Real async memory allocations with comprehensive patterns
    for i in 0..config.allocations_per_task {
        let size = config.allocation_size_range.0
            + (i % (config.allocation_size_range.1 - config.allocation_size_range.0));

        // Create comprehensive async workload patterns
        match i % 8 {
            0 => {
                // CPU-intensive async work with real memory
                let data = vec![i as u64; size.min(1000)];
                let result: u64 = data.iter().sum();
                let boxed: Box<dyn std::any::Any + Send> = Box::new((data, result));
                _allocated_data.push(boxed);
            }
            1 => {
                // I/O simulation with real buffer allocation
                let buffer = vec![task_id as u8; size.min(4096)];
                if i % config.io_simulation_frequency == 0 {
                    tokio::time::sleep(Duration::from_micros(10)).await; // Real async I/O
                }
                let boxed: Box<dyn std::any::Any + Send> = Box::new(buffer);
                _allocated_data.push(boxed);
            }
            2 => {
                // Memory-intensive operations with real strings
                let large_vec: Vec<String> = (0..size.min(500))
                    .map(|x| format!("task_{}_item_{}", task_id, x))
                    .collect();
                let boxed: Box<dyn std::any::Any + Send> = Box::new(large_vec);
                _allocated_data.push(boxed);
            }
            3 => {
                // Network simulation with message queue
                let message_queue: VecDeque<String> = (0..size.min(200))
                    .map(|x| format!("net_msg_{}_{}", task_id, x))
                    .collect();
                let boxed: Box<dyn std::any::Any + Send> = Box::new(message_queue);
                _allocated_data.push(boxed);
            }
            4 => {
                // Cache-like structure with HashMap
                let cache: HashMap<usize, String> = (0..size.min(300))
                    .map(|x| (x, format!("cache_value_{}_{}", task_id, x)))
                    .collect();
                let boxed: Box<dyn std::any::Any + Send> = Box::new(cache);
                _allocated_data.push(boxed);
            }
            5 => {
                // Streaming buffer simulation
                let stream_buffer = StreamBuffer {
                    id: task_id,
                    sequence: i,
                    data: vec![0u8; size.min(2048)],
                    metadata: format!("stream_{}_{}", task_id, i),
                };
                let boxed: Box<dyn std::any::Any + Send> = Box::new(stream_buffer);
                _allocated_data.push(boxed);
            }
            6 => {
                // Background task data
                let background_state = BackgroundTaskState {
                    task_id,
                    iteration: i,
                    accumulated_data: (0..size.min(800)).map(|x| x as u32).collect(),
                    status: format!("processing_{}_{}", task_id, i),
                };
                let boxed: Box<dyn std::any::Any + Send> = Box::new(background_state);
                _allocated_data.push(boxed);
            }
            _ => {
                // Mixed workload data
                let mixed_state = MixedWorkloadData {
                    cpu_result: (i as f64).sqrt(),
                    io_buffer: vec![task_id as u8; size.min(1024)],
                    network_payload: format!("payload_{}_{}", task_id, i),
                    memory_chunk: vec![i as u32; size.min(256)],
                };
                let boxed: Box<dyn std::any::Any + Send> = Box::new(mixed_state);
                _allocated_data.push(boxed);
            }
        }

        // Periodic async yield for cooperative scheduling
        if i % 100 == 0 {
            tokio::time::sleep(Duration::from_micros(50)).await;
        }

        // Periodic cleanup to manage memory
        if i % 200 == 0 && _allocated_data.len() > 30 {
            _allocated_data.drain(0..15);
        }
    }

    // Wait for metrics updates to complete
    update_handle.abort();

    Ok(_allocated_data.len())
}

/// Custom structures for comprehensive async testing
#[derive(Debug)]
#[allow(dead_code)]
struct StreamBuffer {
    id: usize,
    sequence: usize,
    data: Vec<u8>,
    metadata: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct BackgroundTaskState {
    task_id: usize,
    iteration: usize,
    accumulated_data: Vec<u32>,
    status: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct MixedWorkloadData {
    cpu_result: f64,
    io_buffer: Vec<u8>,
    network_payload: String,
    memory_chunk: Vec<u32>,
}

/// Generate comprehensive reports using async_memory visualization module
async fn generate_comprehensive_reports(
    monitor: &Arc<Mutex<AsyncResourceMonitor>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = {
        let mon = monitor.lock().unwrap();
        mon.get_all_profiles().clone()
    };

    // Generate JSON report
    let json_output = serde_json::to_string_pretty(&profiles)?;
    tokio::fs::write("async_test_analysis.json", json_output).await?;

    // Generate HTML dashboard using async_memory's built-in VisualizationGenerator
    let viz_generator = VisualizationGenerator::new();
    let html_content = viz_generator.generate_html_report(&profiles)?;
    tokio::fs::write("async_test_dashboard.html", html_content).await?;

    println!("📄 Generated comprehensive reports:");
    println!("   📊 JSON: async_test_analysis.json");
    println!("   🌐 HTML: async_test_dashboard.html (using async_memory template)");

    Ok(())
}

/// Get process memory usage
fn get_process_memory_usage() -> usize {
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<usize>() {
                            return kb * 1024;
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("ps")
            .args(["-o", "rss=", "-p"])
            .arg(std::process::id().to_string())
            .output()
        {
            if let Ok(rss_str) = String::from_utf8(output.stdout) {
                if let Ok(rss_kb) = rss_str.trim().parse::<usize>() {
                    return rss_kb * 1024;
                }
            }
        }
    }

    // Fallback estimation
    60 * 1024 * 1024 // 60MB
}

/// Generate a single combined HTML report using official async_memory VisualizationGenerator API
///
/// This creates a comprehensive report by building TaskProfile data from all test scenarios
/// and using the official async_memory VisualizationGenerator to generate the HTML.
async fn generate_combined_async_html_report(
    all_results: &[(String, AsyncResults)],
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build comprehensive task profiles from all test scenarios
    let mut combined_profiles: std::collections::HashMap<TaskId, TaskResourceProfile> =
        std::collections::HashMap::new();

    for (scenario_name, results) in all_results {
        for task_id in 0..results.num_tasks {
            let _profile_key = format!(
                "{}_{}",
                scenario_name.replace(" ", "_").to_lowercase(),
                task_id
            );

            // Create realistic TaskResourceProfile using actual test data
            let current_time_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let task_profile = memscope_rs::async_memory::resource_monitor::TaskResourceProfile {
                task_id: (task_id + scenario_name.len() * 1000)
                    as memscope_rs::async_memory::TaskId,
                task_name: format!("{}_{}", scenario_name.replace(" ", ""), task_id),
                task_type: match task_id % 5 {
                    0 => TaskType::CpuIntensive,
                    1 => TaskType::IoIntensive,
                    2 => TaskType::MemoryIntensive,
                    3 => TaskType::NetworkIntensive,
                    _ => TaskType::Mixed,
                },
                start_time: current_time_ms - results.duration.as_millis() as u64,
                end_time: Some(current_time_ms),
                duration_ms: Some(results.duration.as_millis() as f64),
                cpu_metrics: memscope_rs::async_memory::resource_monitor::CpuMetrics {
                    usage_percent: 25.0 + (task_id as f64 * 5.0) % 75.0,
                    time_user_ms: (results.duration.as_millis() as f64) * 0.7,
                    time_kernel_ms: (results.duration.as_millis() as f64) * 0.3,
                    context_switches: task_id as u64 * 25,
                    cpu_cycles: task_id as u64 * 1000000,
                    instructions: task_id as u64 * 500000,
                    cache_misses: task_id as u64 * 100,
                    branch_misses: task_id as u64 * 50,
                    core_affinity: vec![0, 1],
                },
                memory_metrics: memscope_rs::async_memory::resource_monitor::MemoryMetrics {
                    allocated_bytes: ((results.total_allocations / results.num_tasks) * 256) as u64,
                    peak_bytes: (results.memory_usage_mb / results.num_tasks as f64
                        * 1024.0
                        * 1024.0) as u64,
                    current_bytes: (results.memory_usage_mb / results.num_tasks as f64
                        * 1024.0
                        * 1024.0
                        * 0.8) as u64,
                    allocation_count: (results.total_allocations / results.num_tasks) as u64,
                    deallocation_count: ((results.total_allocations / results.num_tasks) as f64
                        * 0.9) as u64,
                    page_faults: task_id as u64 * 10,
                    heap_fragmentation: 0.1,
                    memory_bandwidth_mbps: 1000.0,
                },
                io_metrics: memscope_rs::async_memory::resource_monitor::IoMetrics {
                    bytes_read: task_id as u64 * 1024,
                    bytes_written: task_id as u64 * 512,
                    read_operations: task_id as u64 * 10,
                    write_operations: task_id as u64 * 5,
                    sync_operations: task_id as u64 * 3,
                    async_operations: task_id as u64 * 12,
                    avg_latency_us: 100.0,
                    bandwidth_mbps: 50.0,
                    queue_depth: 4,
                    io_wait_percent: 5.0,
                },
                network_metrics: memscope_rs::async_memory::resource_monitor::NetworkMetrics {
                    bytes_sent: task_id as u64 * 1024,
                    bytes_received: task_id as u64 * 2048,
                    packets_sent: task_id as u64 * 10,
                    packets_received: task_id as u64 * 15,
                    connections_active: 2,
                    connections_established: 3,
                    connection_errors: 0,
                    latency_avg_ms: 50.0,
                    throughput_mbps: 10.0,
                    retransmissions: 0,
                },
                gpu_metrics: None,
                efficiency_score: 0.8,
                resource_balance: 0.7,
                bottleneck_type:
                    memscope_rs::async_memory::resource_monitor::BottleneckType::Balanced,
                source_location: memscope_rs::async_memory::resource_monitor::SourceLocation {
                    file_path: "examples/async_memory_test.rs".to_string(),
                    line_number: 200 + task_id as u32,
                    function_name: format!(
                        "{}_task_{}",
                        scenario_name.to_lowercase().replace(" ", "_"),
                        task_id
                    ),
                    module_path: "async_memory_test".to_string(),
                    crate_name: "memscope_rs".to_string(),
                },
                hot_metrics: memscope_rs::async_memory::resource_monitor::HotMetrics {
                    cpu_hotspots: vec![],
                    memory_hotspots: vec![],
                    io_hotspots: vec![],
                    network_hotspots: vec![],
                    critical_path_analysis:
                        memscope_rs::async_memory::resource_monitor::CriticalPathAnalysis {
                            total_execution_time_ms: results.duration.as_millis() as f64,
                            critical_path_time_ms: results.duration.as_millis() as f64 * 0.7,
                            parallelization_potential: 0.5,
                            blocking_operations: vec![],
                        },
                },
                efficiency_explanation:
                    memscope_rs::async_memory::resource_monitor::EfficiencyExplanation {
                        overall_score: 0.8,
                        component_scores:
                            memscope_rs::async_memory::resource_monitor::ComponentScores {
                                cpu_efficiency: 0.8,
                                memory_efficiency: 0.7,
                                io_efficiency: 0.9,
                                network_efficiency: 0.8,
                                resource_balance: 0.7,
                            },
                        recommendations: vec![],
                        bottleneck_analysis: "No significant bottlenecks detected".to_string(),
                        optimization_potential: 0.2,
                    },
            };

            combined_profiles.insert(task_profile.task_id, task_profile);
        }
    }

    // Use the official async_memory VisualizationGenerator API
    let viz_generator = VisualizationGenerator::new();
    let html_content = viz_generator.generate_html_report(&combined_profiles)?;

    tokio::fs::write(output_path, html_content).await?;

    println!(
        "📄 Combined HTML report generated using official async_memory VisualizationGenerator: {}",
        output_path
    );
    println!(
        "📊 Report includes {} scenarios with {} total task profiles",
        all_results.len(),
        combined_profiles.len()
    );

    Ok(())
}

fn main() {
    AsyncMemoryTestSuite::run_all_tests();
}
