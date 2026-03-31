//! Async Memory Tracking - New API
//!
//! This example demonstrates async memory tracking using the new unified API.
//!
//! ## Limitations vs Old API
//!
//! The new API has the following limitations in async scenarios:
//!
//! | Feature | Old API | New API | Status |
//! |---------|---------|---------|--------|
//! | Task ID tracking | ✅ | ❌ | Not implemented |
//! | Task type classification | ✅ | ❌ | Not implemented |
//! | Per-task metrics | ✅ | ❌ | Not implemented |
//! | Task lifecycle management | ✅ | ❌ | Not implemented |
//! | Efficiency scoring | ✅ | ❌ | Not implemented |
//! | Bottleneck analysis | ✅ | ❌ | Not implemented |
//! | Optimization recommendations | ✅ | ❌ | Not implemented |
//! | Source location tracking | ✅ | ❌ | Not implemented |
//! | Variable name capture | ❌ | ✅ | **New feature** |
//! | Type name capture | ❌ | ✅ | **New feature** |
//! | Simple API | ❌ | ✅ | **New feature** |
//!
//! ## What New API Can Do
//!
//! - Track memory allocations across async tasks
//! - Capture variable names automatically
//! - Export data to JSON
//! - Simple, unified interface
//!
//! ## What New API Cannot Do (Yet)
//!
//! - Distinguish between different tasks
//! - Track task-specific metrics
//! - Provide efficiency scores
//! - Generate optimization recommendations
//! - Analyze bottlenecks per task

use memscope_rs::{track, tracker};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Task configuration for async workload
#[derive(Debug, Clone)]
struct TaskConfig {
    name: String,
    task_type: TaskType,
    intensity: IntensityLevel,
    duration_secs: u64,
}

#[derive(Debug, Clone)]
enum TaskType {
    CpuIntensive,
    IoIntensive,
    NetworkIntensive,
    MemoryIntensive,
    Streaming,
    Background,
    Mixed,
}

#[derive(Debug, Clone)]
enum IntensityLevel {
    Light,
    Moderate,
    Heavy,
    Extreme,
}

/// Task execution result
#[derive(Debug, Clone)]
struct TaskResult {
    task_name: String,
    task_type: String,
    allocations: usize,
    peak_memory_mb: f64,
    execution_time_ms: u64,
    status: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Async Memory Tracking (New API)");
    println!("===================================");

    // Create tracker with system monitoring
    let tracker = tracker!().with_system_monitoring();

    // Create task configurations
    let task_configs = create_task_configs();

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

    println!("\n🎯 Starting {} async tasks...", task_configs.len());

    let start_time = Instant::now();
    let results = Arc::new(Mutex::new(Vec::new()));

    // Launch all tasks
    let mut handles = Vec::new();

    for (index, config) in task_configs.into_iter().enumerate() {
        let tracker_clone = tracker!(); // Each task gets its own tracker view
        let results_clone = results.clone();

        let handle = tokio::spawn(async move {
            let result = execute_task(index, config, tracker_clone).await;
            if let Ok(task_result) = result {
                results_clone.lock().unwrap().push(task_result);
            }
        });

        handles.push(handle);

        // Stagger task starts
        sleep(Duration::from_millis(100)).await;
    }

    // Wait for all tasks
    println!("\n⏳ Waiting for tasks to complete...");
    for handle in handles {
        handle.await.ok();
    }

    let total_time = start_time.elapsed();

    // Print results
    println!("\n📊 Task Results:");
    println!("================");

    if let Ok(results) = results.lock() {
        for result in results.iter() {
            println!(
                "  {} ({}) - {} allocations, {:.1} MB peak, {} ms - {}",
                result.task_name,
                result.task_type,
                result.allocations,
                result.peak_memory_mb,
                result.execution_time_ms,
                result.status
            );
        }
    }

    // Generate analysis report
    println!("\n🔍 Generating analysis report...");
    let report = tracker.analyze();

    println!("\n📈 Analysis Report:");
    println!("===================");
    println!("Total allocations: {}", report.total_allocations);
    println!(
        "Peak memory: {:.2} MB",
        report.peak_memory_bytes as f64 / 1_048_576.0
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
    tracker.export_json("async_new_api")?;
    tracker.export_analysis("async_new_api")?;

    println!(
        "\n⏱️  Total execution time: {:.2}s",
        total_time.as_secs_f64()
    );

    println!("\n🎉 Async showcase completed!");
    println!("📄 Check 'async_new_api.json' and 'async_new_api_analysis.json'");

    Ok(())
}

fn create_task_configs() -> Vec<TaskConfig> {
    vec![
        TaskConfig {
            name: "Matrix Multiplication".to_string(),
            task_type: TaskType::CpuIntensive,
            intensity: IntensityLevel::Heavy,
            duration_secs: 15,
        },
        TaskConfig {
            name: "Data Stream Processing".to_string(),
            task_type: TaskType::Streaming,
            intensity: IntensityLevel::Moderate,
            duration_secs: 20,
        },
        TaskConfig {
            name: "File System Scanner".to_string(),
            task_type: TaskType::IoIntensive,
            intensity: IntensityLevel::Heavy,
            duration_secs: 12,
        },
        TaskConfig {
            name: "Web Scraper".to_string(),
            task_type: TaskType::NetworkIntensive,
            intensity: IntensityLevel::Moderate,
            duration_secs: 18,
        },
        TaskConfig {
            name: "Memory Cache Manager".to_string(),
            task_type: TaskType::MemoryIntensive,
            intensity: IntensityLevel::Extreme,
            duration_secs: 10,
        },
        TaskConfig {
            name: "Background Maintenance".to_string(),
            task_type: TaskType::Background,
            intensity: IntensityLevel::Light,
            duration_secs: 25,
        },
        TaskConfig {
            name: "Hybrid Workload".to_string(),
            task_type: TaskType::Mixed,
            intensity: IntensityLevel::Heavy,
            duration_secs: 20,
        },
    ]
}

async fn execute_task(
    task_id: usize,
    config: TaskConfig,
    tracker: memscope_rs::tracker::Tracker,
) -> Result<TaskResult, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = Instant::now();
    let mut allocations = 0;

    println!("🚀 Starting Task {}: {}", task_id, config.name);

    let result = match config.task_type {
        TaskType::CpuIntensive => execute_cpu_intensive(&config, &tracker, &mut allocations).await,
        TaskType::IoIntensive => execute_io_intensive(&config, &tracker, &mut allocations).await,
        TaskType::NetworkIntensive => {
            execute_network_intensive(&config, &tracker, &mut allocations).await
        }
        TaskType::MemoryIntensive => {
            execute_memory_intensive(&config, &tracker, &mut allocations).await
        }
        TaskType::Streaming => execute_streaming(&config, &tracker, &mut allocations).await,
        TaskType::Background => execute_background(&config, &tracker, &mut allocations).await,
        TaskType::Mixed => execute_mixed(&config, &tracker, &mut allocations).await,
    };

    let execution_time = start_time.elapsed();

    Ok(TaskResult {
        task_name: config.name,
        task_type: format!("{:?}", config.task_type),
        allocations,
        peak_memory_mb: 0.0, // Would need to track this separately
        execution_time_ms: execution_time.as_millis() as u64,
        status: if result.is_ok() {
            "Completed".to_string()
        } else {
            "Failed".to_string()
        },
    })
}

async fn execute_cpu_intensive(
    config: &TaskConfig,
    tracker: &memscope_rs::tracker::Tracker,
    allocations: &mut usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let iterations = match config.intensity {
        IntensityLevel::Light => 1_000_000,
        IntensityLevel::Moderate => 5_000_000,
        IntensityLevel::Heavy => 15_000_000,
        IntensityLevel::Extreme => 30_000_000,
    };

    let mut result = 0u64;
    for i in 0..iterations {
        // Simulate CPU work
        let val = (i as f64).sqrt().sin().cos();
        result = result.wrapping_add((val * 1000.0) as u64);

        // Yield periodically
        if i % 100_000 == 0 {
            tokio::task::yield_now().await;
        }
    }

    Ok(())
}

async fn execute_io_intensive(
    config: &TaskConfig,
    tracker: &memscope_rs::tracker::Tracker,
    allocations: &mut usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        let filename = format!("tmp_async_io_{}.dat", i);

        // Create test data
        let data = vec![((i * 37) % 256) as u8; file_size];
        track!(tracker, data);
        *allocations += 1;

        // Write file
        tokio::fs::write(&filename, &data).await?;

        // Read file
        let _ = tokio::fs::read(&filename).await?;

        // Cleanup
        tokio::fs::remove_file(&filename).await.ok();

        total_bytes += file_size as u64;

        if i % 10 == 0 {
            tokio::task::yield_now().await;
        }
    }

    Ok(())
}

async fn execute_network_intensive(
    config: &TaskConfig,
    tracker: &memscope_rs::tracker::Tracker,
    allocations: &mut usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
            track!(tracker, response_data);
            *allocations += 1;
            total_bytes += response_data.len() as u64;
            successful_requests += 1;
        }
    }

    Ok(())
}

async fn execute_memory_intensive(
    config: &TaskConfig,
    tracker: &memscope_rs::tracker::Tracker,
    allocations: &mut usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let size_mb = match config.intensity {
        IntensityLevel::Light => 5,
        IntensityLevel::Moderate => 10,
        IntensityLevel::Heavy => 20,
        IntensityLevel::Extreme => 50,
    };

    let mut data_chunks = Vec::new();

    for i in 0..size_mb {
        let chunk = vec![i as u8; 1024 * 1024]; // 1MB chunk
        track!(tracker, chunk);
        *allocations += 1;
        data_chunks.push(chunk);

        if i % 5 == 0 {
            tokio::task::yield_now().await;
        }
    }

    // Process data
    let mut checksum = 0u64;
    for chunk in &data_chunks {
        for &byte in chunk.iter().step_by(1024) {
            checksum = checksum.wrapping_add(byte as u64);
        }
    }

    Ok(())
}

async fn execute_streaming(
    config: &TaskConfig,
    tracker: &memscope_rs::tracker::Tracker,
    allocations: &mut usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let stream_duration = Duration::from_secs(config.duration_secs);
    let batch_size = match config.intensity {
        IntensityLevel::Light => 100,
        IntensityLevel::Moderate => 500,
        IntensityLevel::Heavy => 1000,
        IntensityLevel::Extreme => 2000,
    };

    println!(
        "📡 Starting streaming task: {} items/batch for {} seconds",
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

    Ok(())
}

async fn execute_background(
    config: &TaskConfig,
    tracker: &memscope_rs::tracker::Tracker,
    allocations: &mut usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cycles = config.duration_secs as usize;

    for i in 0..cycles {
        // Small allocations
        let temp_data = vec![i as u8; 1024];
        track!(tracker, temp_data);
        *allocations += 1;

        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

async fn execute_mixed(
    config: &TaskConfig,
    tracker: &memscope_rs::tracker::Tracker,
    allocations: &mut usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let duration = Duration::from_secs(config.duration_secs);
    let start_time = Instant::now();

    let mut work_type = 0;

    while start_time.elapsed() < duration {
        match work_type % 4 {
            0 => {
                // CPU work
                let mut result = 0u64;
                for i in 0..10000 {
                    result = result.wrapping_add((i as f64).sin() as u64);
                }
            }
            1 => {
                // Memory work
                let data = vec![work_type as u8; 4096];
                track!(tracker, data);
                *allocations += 1;
            }
            2 => {
                // Simulated IO
                sleep(Duration::from_millis(10)).await;
            }
            3 => {
                // Simulated network
                sleep(Duration::from_millis(5)).await;
            }
            _ => unreachable!(),
        }

        work_type += 1;
        tokio::task::yield_now().await;
    }

    Ok(())
}
