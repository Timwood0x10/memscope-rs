//! Comprehensive Async Memory Showcase
//!
//! This example demonstrates all aspects of the async memory tracking system:
//! - Different task types with realistic workloads
//! - Resource monitoring and performance analysis
//! - HTML report generation with detailed metrics
//! - Advanced features like hotspot analysis and efficiency scoring

use memscope_rs::async_memory::{
    resource_monitor::SourceLocation, visualization::VisualizationGenerator, AsyncResourceMonitor,
    TaskId, TaskType,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
struct ComprehensiveTaskConfig {
    name: String,
    task_type: TaskType,
    intensity: IntensityLevel,
    duration_secs: u64,
    _description: String,
}

#[derive(Debug, Clone)]
enum IntensityLevel {
    Light,
    Moderate,
    Heavy,
    Extreme,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Comprehensive Async Memory Showcase");
    println!("======================================");

    // Initialize the async memory tracking system
    memscope_rs::async_memory::initialize()?;

    let monitor = Arc::new(Mutex::new(AsyncResourceMonitor::new()));
    let mut task_handles = Vec::new();

    // Create diverse task configurations
    let task_configs = create_comprehensive_task_configs();

    println!("📋 Task Configuration:");
    for (i, config) in task_configs.iter().enumerate() {
        println!(
            "  {}. {} [{:?}] - {:?} intensity",
            i + 1,
            config.name,
            config.task_type,
            config.intensity
        );
    }

    println!(
        "\n🎯 Starting {} tasks with comprehensive monitoring...",
        task_configs.len()
    );

    // Launch all tasks
    for (index, config) in task_configs.into_iter().enumerate() {
        let task_id = index as TaskId;
        let config_clone = config.clone();
        let monitor_clone = Arc::clone(&monitor);

        // Start monitoring with detailed source location
        {
            let mut mon = monitor.lock().unwrap();
            let source_location = SourceLocation {
                file_path: "examples/comprehensive_async_showcase.rs".to_string(),
                line_number: 60 + (index as u32 * 5),
                function_name: format!(
                    "execute_{}_task",
                    config.name.to_lowercase().replace(" ", "_")
                ),
                module_path: "comprehensive_async_showcase".to_string(),
                crate_name: "memscope_rs".to_string(),
            };

            mon.start_monitoring_with_location(
                task_id,
                config.name.clone(),
                config.task_type.clone(),
                Some(source_location),
            );
        }

        let handle = tokio::spawn(async move {
            execute_comprehensive_task(task_id, config_clone, monitor_clone).await
        });

        task_handles.push((task_id, config.name.clone(), handle));

        println!("🚀 Started Task {}: {}", task_id, config.name);

        // Stagger task starts
        sleep(Duration::from_millis(200)).await;
    }

    // Monitor all tasks for their specified durations
    println!("\n⏱️  Monitoring tasks with real-time metrics...");

    // Start a monitoring loop
    let monitor_clone = Arc::clone(&monitor);
    let metrics_handle = tokio::spawn(async move {
        for _ in 0..60 {
            // Monitor for 60 seconds
            {
                let mut mon = monitor_clone.lock().unwrap();
                let task_ids: Vec<TaskId> = mon.get_all_profiles().keys().copied().collect();
                for task_id in task_ids {
                    mon.update_metrics(task_id);
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    });

    // Wait for all tasks to complete
    for (task_id, task_name, handle) in task_handles {
        match tokio::time::timeout(Duration::from_secs(30), handle).await {
            Ok(Ok(result)) => {
                println!(
                    "✅ Task {} ({}) completed: {:?}",
                    task_id, task_name, result
                );
                let mut mon = monitor.lock().unwrap();
                mon.finish_monitoring(task_id);
            }
            Ok(Err(e)) => {
                println!("❌ Task {} ({}) failed: {}", task_id, task_name, e);
            }
            Err(_) => {
                println!("⏰ Task {} ({}) timed out", task_id, task_name);
                let mut mon = monitor.lock().unwrap();
                mon.finish_monitoring(task_id);
            }
        }
    }

    // Stop metrics monitoring
    metrics_handle.abort();

    // Generate comprehensive reports
    println!("\n📊 Generating comprehensive analysis reports...");
    generate_json_report(&monitor).await?;
    generate_advanced_html_report(&monitor).await?;

    // Print summary statistics
    print_performance_summary(&monitor).await?;

    println!("\n🎉 Comprehensive showcase completed!");
    println!("📄 Check 'comprehensive_async_analysis.json' for detailed metrics");
    println!("📄 Check 'comprehensive_async_dashboard.html' for interactive analysis");

    Ok(())
}

fn create_comprehensive_task_configs() -> Vec<ComprehensiveTaskConfig> {
    vec![
        ComprehensiveTaskConfig {
            name: "Matrix Multiplication".to_string(),
            task_type: TaskType::CpuIntensive,
            intensity: IntensityLevel::Heavy,
            duration_secs: 15,
            _description: "Large matrix operations with SIMD optimization".to_string(),
        },
        ComprehensiveTaskConfig {
            name: "Data Stream Processing".to_string(),
            task_type: TaskType::Streaming,
            intensity: IntensityLevel::Moderate,
            duration_secs: 20,
            _description: "Real-time data processing with backpressure handling".to_string(),
        },
        ComprehensiveTaskConfig {
            name: "File System Scanner".to_string(),
            task_type: TaskType::IoIntensive,
            intensity: IntensityLevel::Heavy,
            duration_secs: 12,
            _description: "Recursive directory scanning with async file operations".to_string(),
        },
        ComprehensiveTaskConfig {
            name: "Web Scraper".to_string(),
            task_type: TaskType::NetworkIntensive,
            intensity: IntensityLevel::Moderate,
            duration_secs: 18,
            _description: "Concurrent HTTP requests with rate limiting".to_string(),
        },
        ComprehensiveTaskConfig {
            name: "Memory Cache Manager".to_string(),
            task_type: TaskType::MemoryIntensive,
            intensity: IntensityLevel::Extreme,
            duration_secs: 10,
            _description: "Large-scale in-memory data structures with LRU eviction".to_string(),
        },
        ComprehensiveTaskConfig {
            name: "Background Maintenance".to_string(),
            task_type: TaskType::Background,
            intensity: IntensityLevel::Light,
            duration_secs: 25,
            _description: "Low-priority cleanup and maintenance tasks".to_string(),
        },
        ComprehensiveTaskConfig {
            name: "Hybrid Workload".to_string(),
            task_type: TaskType::Mixed,
            intensity: IntensityLevel::Heavy,
            duration_secs: 20,
            _description: "Mixed CPU, IO, and network operations".to_string(),
        },
    ]
}

async fn execute_comprehensive_task(
    task_id: TaskId,
    config: ComprehensiveTaskConfig,
    monitor: Arc<Mutex<AsyncResourceMonitor>>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Start periodic metrics updates
    let monitor_clone = Arc::clone(&monitor);
    let update_handle = tokio::spawn(async move {
        for _ in 0..(config.duration_secs * 2) {
            // Update every 500ms
            {
                let mut mon = monitor_clone.lock().unwrap();
                mon.update_metrics(task_id);
            }
            sleep(Duration::from_millis(500)).await;
        }
    });

    // Execute task based on type and intensity
    let result = match config.task_type {
        TaskType::CpuIntensive => execute_cpu_intensive_task(&config).await,
        TaskType::IoIntensive => execute_io_intensive_task(&config).await,
        TaskType::NetworkIntensive => execute_network_intensive_task(&config).await,
        TaskType::MemoryIntensive => execute_memory_intensive_task(&config).await,
        TaskType::Streaming => execute_streaming_task(&config).await,
        TaskType::Background => execute_background_task(&config).await,
        TaskType::Mixed => execute_mixed_task(&config).await,
        TaskType::GpuCompute => execute_gpu_compute_task(&config).await,
    };

    // Wait for metrics updates to complete
    update_handle.abort();

    result
}

async fn execute_cpu_intensive_task(
    config: &ComprehensiveTaskConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let iterations = match config.intensity {
        IntensityLevel::Light => 1_000_000,
        IntensityLevel::Moderate => 5_000_000,
        IntensityLevel::Heavy => 15_000_000,
        IntensityLevel::Extreme => 30_000_000,
    };

    println!("🔥 Starting CPU intensive task: {} iterations", iterations);

    // Simulate matrix multiplication
    let mut result = 0u64;
    for i in 0..iterations {
        // Complex mathematical operations
        let val = (i as f64).sqrt().sin().cos();
        result = result.wrapping_add((val * 1000.0) as u64);

        // Yield periodically to allow other tasks
        if i % 100_000 == 0 {
            tokio::task::yield_now().await;
        }
    }

    Ok(format!(
        "CPU task completed with result checksum: {}",
        result
    ))
}

async fn execute_io_intensive_task(
    config: &ComprehensiveTaskConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let file_count = match config.intensity {
        IntensityLevel::Light => 10,
        IntensityLevel::Moderate => 50,
        IntensityLevel::Heavy => 100,
        IntensityLevel::Extreme => 200,
    };

    let file_size = match config.intensity {
        IntensityLevel::Light => 1024,      // 1KB
        IntensityLevel::Moderate => 102400, // 100KB
        IntensityLevel::Heavy => 1048576,   // 1MB
        IntensityLevel::Extreme => 5242880, // 5MB
    };

    println!(
        "💾 Starting IO intensive task: {} files of {}KB each",
        file_count,
        file_size / 1024
    );

    let mut total_bytes = 0u64;

    for i in 0..file_count {
        let filename = format!("tmp_rovodev_io_test_{}.dat", i);

        // Create test data
        let data = vec![((i * 37) % 256) as u8; file_size];

        // Write file
        tokio::fs::write(&filename, &data).await?;
        total_bytes += data.len() as u64;

        // Read and verify
        let read_data = tokio::fs::read(&filename).await?;
        if read_data.len() != data.len() {
            return Err("File size mismatch".into());
        }

        // Cleanup
        tokio::fs::remove_file(&filename).await.ok();

        // Small delay between operations
        if i % 10 == 0 {
            tokio::task::yield_now().await;
        }
    }

    Ok(format!(
        "IO task processed {} files, {} MB total",
        file_count,
        total_bytes / 1048576
    ))
}

async fn execute_network_intensive_task(
    config: &ComprehensiveTaskConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let request_count = match config.intensity {
        IntensityLevel::Light => 10,
        IntensityLevel::Moderate => 50,
        IntensityLevel::Heavy => 100,
        IntensityLevel::Extreme => 200,
    };

    println!(
        "🌐 Starting network intensive task: {} simulated requests",
        request_count
    );

    let mut total_bytes = 0u64;
    let mut successful_requests = 0;

    // Simulate concurrent HTTP requests
    let mut handles = Vec::new();

    for i in 0..request_count {
        let handle = tokio::spawn(async move {
            // Simulate network latency
            let latency = Duration::from_millis(50 + (i % 100) as u64);
            sleep(latency).await;

            // Simulate response data
            let response_size = 1024 + (i * 100) % 5000; // Variable response sizes
            let response_data = vec![0u8; response_size];

            Ok::<(usize, Vec<u8>), Box<dyn std::error::Error + Send + Sync>>((i, response_data))
        });

        handles.push(handle);

        // Add some delay between request starts
        if i % 10 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }

    // Wait for all requests to complete
    for handle in handles {
        if let Ok(Ok((_, response_data))) = handle.await {
            total_bytes += response_data.len() as u64;
            successful_requests += 1;
        } // Ignore failures for this demo
    }

    Ok(format!(
        "Network task completed {} requests, {} KB transferred",
        successful_requests,
        total_bytes / 1024
    ))
}

async fn execute_memory_intensive_task(
    config: &ComprehensiveTaskConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let allocation_size = match config.intensity {
        IntensityLevel::Light => 10 * 1024 * 1024,    // 10MB
        IntensityLevel::Moderate => 50 * 1024 * 1024, // 50MB
        IntensityLevel::Heavy => 100 * 1024 * 1024,   // 100MB
        IntensityLevel::Extreme => 250 * 1024 * 1024, // 250MB
    };

    println!(
        "🧠 Starting memory intensive task: {} MB allocation",
        allocation_size / 1048576
    );

    let mut allocations = Vec::new();
    let chunk_size = 1024 * 1024; // 1MB chunks
    let num_chunks = allocation_size / chunk_size;

    // Allocate memory in chunks
    for i in 0..num_chunks {
        let mut chunk = vec![0u8; chunk_size];

        // Fill with pattern to prevent optimization
        for (j, byte) in chunk.iter_mut().enumerate() {
            *byte = ((i + j) % 256) as u8;
        }

        allocations.push(chunk);

        // Yield occasionally during large allocations
        if i % 10 == 0 {
            tokio::task::yield_now().await;
        }
    }

    // Simulate some processing on the allocated memory
    let mut checksum = 0u64;
    for chunk in &allocations {
        for &byte in chunk.iter().step_by(1024) {
            // Sample every 1KB
            checksum = checksum.wrapping_add(byte as u64);
        }
        tokio::task::yield_now().await;
    }

    Ok(format!(
        "Memory task allocated {} MB with checksum {}",
        allocations.len(),
        checksum
    ))
}

async fn execute_streaming_task(
    config: &ComprehensiveTaskConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let stream_duration = Duration::from_secs(config.duration_secs);
    let batch_size = match config.intensity {
        IntensityLevel::Light => 100,
        IntensityLevel::Moderate => 500,
        IntensityLevel::Heavy => 1000,
        IntensityLevel::Extreme => 2000,
    };

    println!(
        "📡 Starting streaming task: {} items/batch for {}s",
        batch_size, config.duration_secs
    );

    let start_time = std::time::Instant::now();
    let mut total_processed = 0;
    let mut batch_count = 0;

    while start_time.elapsed() < stream_duration {
        // Simulate processing a batch of streaming data
        let mut batch_data = Vec::with_capacity(batch_size);

        for i in 0..batch_size {
            let data_item = format!("stream_item_{}_{}", batch_count, i);
            let processed_item = data_item.chars().rev().collect::<String>(); // Simple processing
            batch_data.push(processed_item);
        }

        total_processed += batch_data.len();
        batch_count += 1;

        // Simulate backpressure handling
        sleep(Duration::from_millis(100)).await;
    }

    Ok(format!(
        "Streaming task processed {} items in {} batches",
        total_processed, batch_count
    ))
}

async fn execute_background_task(
    config: &ComprehensiveTaskConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    println!(
        "🔧 Starting background maintenance task for {}s",
        config.duration_secs
    );

    let end_time = std::time::Instant::now() + Duration::from_secs(config.duration_secs);
    let mut maintenance_cycles = 0;

    while std::time::Instant::now() < end_time {
        // Simulate various maintenance operations

        // Memory cleanup simulation
        let _temp_data = vec![0u8; 1024]; // Small allocation

        // Log rotation simulation
        let _log_entry = format!(
            "Maintenance cycle {} at {:?}",
            maintenance_cycles,
            std::time::Instant::now()
        );

        // Configuration check simulation
        for i in 0..100 {
            let _ = (i as f64).sqrt();
        }

        maintenance_cycles += 1;

        // Background tasks should be low-impact
        sleep(Duration::from_millis(1000)).await;
    }

    Ok(format!(
        "Background task completed {} maintenance cycles",
        maintenance_cycles
    ))
}

async fn execute_mixed_task(
    config: &ComprehensiveTaskConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    println!("🔀 Starting mixed workload task");

    let duration = Duration::from_secs(config.duration_secs);
    let start_time = std::time::Instant::now();

    let mut cpu_work = 0u64;
    let mut io_files = 0;
    let mut network_requests = 0;
    let mut memory_allocations = 0;

    while start_time.elapsed() < duration {
        // Rotate between different types of work
        let work_type = (start_time.elapsed().as_millis() / 1000) % 4;

        match work_type {
            0 => {
                // CPU work
                for i in 0..50000 {
                    cpu_work = cpu_work.wrapping_add((i as f64).sin() as u64);
                }
            }
            1 => {
                // IO work
                let filename = format!("tmp_rovodev_mixed_{}.dat", io_files);
                let data = vec![io_files as u8; 1024];
                tokio::fs::write(&filename, &data).await.ok();
                tokio::fs::remove_file(&filename).await.ok();
                io_files += 1;
            }
            2 => {
                // Network simulation
                sleep(Duration::from_millis(10)).await; // Simulate network latency
                network_requests += 1;
            }
            3 => {
                // Memory work
                let _allocation = vec![memory_allocations as u8; 4096];
                memory_allocations += 1;
            }
            _ => unreachable!(),
        }

        tokio::task::yield_now().await;
    }

    Ok(format!(
        "Mixed task: {} CPU ops, {} IO ops, {} network ops, {} memory ops",
        cpu_work % 1000,
        io_files,
        network_requests,
        memory_allocations
    ))
}

async fn execute_gpu_compute_task(
    config: &ComprehensiveTaskConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    println!("🎮 Starting simulated GPU compute task");

    // Since we don't have actual GPU compute, simulate with CPU work that represents GPU offloading
    let compute_iterations = match config.intensity {
        IntensityLevel::Light => 100_000,
        IntensityLevel::Moderate => 500_000,
        IntensityLevel::Heavy => 1_000_000,
        IntensityLevel::Extreme => 2_000_000,
    };

    let mut compute_result = 0.0f64;

    for i in 0..compute_iterations {
        // Simulate GPU-like parallel operations
        let x = (i as f64) / 1000.0;
        compute_result += (x.sin() * x.cos()).powi(2);

        if i % 50_000 == 0 {
            tokio::task::yield_now().await;
        }
    }

    Ok(format!(
        "GPU compute task completed {} operations, result: {:.6}",
        compute_iterations, compute_result
    ))
}

async fn generate_json_report(
    monitor: &Arc<Mutex<AsyncResourceMonitor>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = {
        let mon = monitor.lock().unwrap();
        mon.get_all_profiles().clone()
    };

    let json_output = serde_json::to_string_pretty(&profiles)?;
    tokio::fs::write("comprehensive_async_analysis.json", json_output).await?;

    println!("📄 JSON report generated: comprehensive_async_analysis.json");
    Ok(())
}

async fn generate_advanced_html_report(
    monitor: &Arc<Mutex<AsyncResourceMonitor>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = {
        let mon = monitor.lock().unwrap();
        mon.get_all_profiles().clone()
    };

    // 使用内置的VisualizationGenerator生成HTML报告
    let viz_generator = VisualizationGenerator::new();
    let html_content = viz_generator.generate_html_report(&profiles)?;
    tokio::fs::write("comprehensive_async_dashboard.html", html_content).await?;

    println!("📄 Advanced HTML dashboard generated using built-in template: comprehensive_async_dashboard.html");
    Ok(())
}

async fn print_performance_summary(
    monitor: &Arc<Mutex<AsyncResourceMonitor>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = {
        let mon = monitor.lock().unwrap();
        mon.get_all_profiles().clone()
    };

    println!("\n📈 Performance Summary:");
    println!("=====================");

    let total_tasks = profiles.len();
    if total_tasks == 0 {
        println!("No tasks were monitored.");
        return Ok(());
    }

    // Calculate statistics
    let total_duration: f64 = profiles.values().filter_map(|p| p.duration_ms).sum::<f64>() / 1000.0;

    let avg_efficiency =
        profiles.values().map(|p| p.efficiency_score).sum::<f64>() / total_tasks as f64;

    let total_memory_mb = profiles
        .values()
        .map(|p| p.memory_metrics.peak_bytes as f64 / 1_048_576.0)
        .sum::<f64>();

    let total_io_mb = profiles
        .values()
        .map(|p| (p.io_metrics.bytes_read + p.io_metrics.bytes_written) as f64 / 1_048_576.0)
        .sum::<f64>();

    let total_network_mb = profiles
        .values()
        .map(|p| {
            (p.network_metrics.bytes_sent + p.network_metrics.bytes_received) as f64 / 1_048_576.0
        })
        .sum::<f64>();

    println!("📊 Total Tasks: {}", total_tasks);
    println!("⏱️  Total Execution Time: {:.2}s", total_duration);
    println!("⚡ Average Efficiency: {:.1}%", avg_efficiency * 100.0);
    println!("🧠 Total Memory Used: {:.1} MB", total_memory_mb);
    println!("💾 Total I/O: {:.1} MB", total_io_mb);
    println!("🌐 Total Network: {:.1} MB", total_network_mb);

    // Find best and worst performing tasks
    let best_task = profiles
        .values()
        .max_by(|a, b| a.efficiency_score.partial_cmp(&b.efficiency_score).unwrap());

    let worst_task = profiles
        .values()
        .min_by(|a, b| a.efficiency_score.partial_cmp(&b.efficiency_score).unwrap());

    if let Some(best) = best_task {
        println!(
            "\n🏆 Best Performing Task: {} ({:.1}% efficiency)",
            best.task_name,
            best.efficiency_score * 100.0
        );
    }

    if let Some(worst) = worst_task {
        println!(
            "🔧 Needs Optimization: {} ({:.1}% efficiency)",
            worst.task_name,
            worst.efficiency_score * 100.0
        );
    }

    // Bottleneck analysis
    let bottleneck_counts =
        profiles
            .values()
            .fold(std::collections::HashMap::new(), |mut acc, profile| {
                let bottleneck = format!("{:?}", profile.bottleneck_type);
                *acc.entry(bottleneck).or_insert(0) += 1;
                acc
            });

    println!("\n🔍 Bottleneck Analysis:");
    for (bottleneck, count) in bottleneck_counts {
        println!("  {}: {} tasks", bottleneck, count);
    }

    println!("\n🎯 Top Recommendations:");
    let mut all_recommendations = Vec::new();
    for profile in profiles.values() {
        for rec in &profile.efficiency_explanation.recommendations {
            all_recommendations.push((rec, profile.task_name.clone()));
        }
    }

    // Sort by estimated improvement
    all_recommendations.sort_by(|a, b| {
        b.0.estimated_improvement
            .partial_cmp(&a.0.estimated_improvement)
            .unwrap()
    });

    for (i, (rec, task_name)) in all_recommendations.iter().take(3).enumerate() {
        println!(
            "  {}. {} ({}): {} ({:.1}% improvement)",
            i + 1,
            rec.category,
            task_name,
            rec.description,
            rec.estimated_improvement
        );
    }

    Ok(())
}
