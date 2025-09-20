//! Classified Async Task Monitor
//!
//! This example demonstrates monitoring 15 async tasks with detailed classification:
//! - CPU Intensive Tasks (5 tasks)
//! - IO Intensive Tasks (4 tasks) 
//! - Network Intensive Tasks (3 tasks)
//! - Memory Intensive Tasks (2 tasks)
//! - Mixed Workload Task (1 task)
//!
//! Features:
//! - Real-time task classification and monitoring
//! - Dark mode HTML report with task categorization
//! - Performance analysis per category
//! - Resource utilization patterns analysis

use memscope_rs::async_memory::{
    self, TaskType, TaskId, AsyncResourceMonitor, TaskResourceProfile,
    resource_monitor::SourceLocation
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone)]
struct ClassifiedTaskConfig {
    name: String,
    task_type: TaskType,
    category: TaskCategory,
    description: String,
    intensity_level: IntensityLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TaskCategory {
    CpuHeavy,
    IoHeavy,
    NetworkHeavy,
    MemoryHeavy,
    Balanced,
}

impl TaskCategory {
    fn category_name(&self) -> &'static str {
        match self {
            TaskCategory::CpuHeavy => "cpu_heavy",
            TaskCategory::IoHeavy => "io_heavy", 
            TaskCategory::NetworkHeavy => "network_heavy",
            TaskCategory::MemoryHeavy => "memory_heavy",
            TaskCategory::Balanced => "balanced",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum IntensityLevel {
    Light,
    Medium,
    Heavy,
    Extreme,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Classified Async Task Monitor");
    println!("================================");
    
    // Initialize the async memory tracking system
    async_memory::initialize()?;
    
    // Create shared resource monitor
    let monitor = Arc::new(Mutex::new(AsyncResourceMonitor::new()));
    
    // Create classified task configurations
    let task_configs = create_classified_tasks();
    println!("📋 Created {} classified tasks for monitoring", task_configs.len());
    
    // Print task categories summary
    print_task_categories(&task_configs);
    
    let mut task_handles = Vec::new();
    
    // Spawn all tasks with staggered start times
    for (i, config) in task_configs.iter().enumerate() {
        let task_id = (2000 + i) as TaskId; // Start from 2000
        let monitor_clone = Arc::clone(&monitor);
        let config_clone = config.clone();
        
        // Create source location for this task
        let source_location = SourceLocation {
            file_path: "examples/classified_async_tasks.rs".to_string(),
            line_number: 150 + (i as u32 * 5),
            function_name: format!("execute_{}_task", config.category.category_name().to_lowercase()),
            module_path: "classified_async_tasks".to_string(),
            crate_name: "memscope_rs".to_string(),
        };

        // Start monitoring this task with source location
        {
            let mut mon = monitor_clone.lock().unwrap();
            mon.start_monitoring_with_location(task_id, config.name.clone(), config.task_type.clone(), Some(source_location));
        }
        
        let handle = tokio::spawn(async move {
            execute_classified_task(task_id, config_clone, monitor_clone).await
        });
        
        task_handles.push((task_id, config.category.clone(), handle));
        
        println!("🚀 Started Task {}: {} [{:?}] ({:?})", 
                 task_id, config.name, config.category, config.intensity_level);
        
        // Stagger task starts to observe different phases
        sleep(Duration::from_millis(200)).await;
    }
    
    // Monitor tasks for 8 seconds
    let monitoring_duration = Duration::from_secs(8);
    println!("\n⏱️  Monitoring {} tasks for {} seconds...", 
             task_handles.len(), monitoring_duration.as_secs());
    
    // Start monitoring reporter
    let monitoring_handle = {
        let monitor_clone = Arc::clone(&monitor);
        tokio::spawn(async move {
            monitor_and_classify_tasks(monitor_clone, monitoring_duration).await;
        })
    };
    
    // Wait for all tasks to complete or timeout
    let mut completed_by_category: HashMap<TaskCategory, Vec<String>> = HashMap::new();
    
    for (task_id, category, handle) in task_handles {
        match tokio::time::timeout(monitoring_duration, handle).await {
            Ok(Ok(result)) => {
                println!("✅ Task {} [{:?}] completed: {:?}", task_id, category, result);
                completed_by_category.entry(category).or_default().push(format!("Task {}", task_id));
                
                let mut mon = monitor.lock().unwrap();
                mon.finish_monitoring(task_id);
            }
            Ok(Err(e)) => {
                println!("❌ Task {} [{:?}] failed: {}", task_id, category, e);
            }
            Err(_) => {
                println!("⏰ Task {} [{:?}] timed out", task_id, category);
                let mut mon = monitor.lock().unwrap();
                mon.finish_monitoring(task_id);
            }
        }
    }
    
    // Wait for monitoring to complete
    let _ = monitoring_handle.await;
    
    // Print completion summary
    print_completion_summary(&completed_by_category);
    
    // Generate classified HTML report
    println!("\n📊 Generating classified task analysis report...");
    generate_classified_html_report(&monitor).await?;
    
    println!("🎉 Classification analysis completed!");
    println!("📄 Check 'classified_async_analysis.html' for detailed results");
    
    Ok(())
}

/// Create 15 classified tasks across different categories
fn create_classified_tasks() -> Vec<ClassifiedTaskConfig> {
    vec![
        // CPU Heavy Tasks (5 tasks)
        ClassifiedTaskConfig {
            name: "Prime Number Generator".to_string(),
            task_type: TaskType::CpuIntensive,
            category: TaskCategory::CpuHeavy,
            description: "Generates prime numbers using trial division".to_string(),
            intensity_level: IntensityLevel::Heavy,
        },
        ClassifiedTaskConfig {
            name: "Matrix Calculator".to_string(),
            task_type: TaskType::CpuIntensive,
            category: TaskCategory::CpuHeavy,
            description: "Performs intensive matrix operations".to_string(),
            intensity_level: IntensityLevel::Extreme,
        },
        ClassifiedTaskConfig {
            name: "Hash Computer".to_string(),
            task_type: TaskType::CpuIntensive,
            category: TaskCategory::CpuHeavy,
            description: "Computes cryptographic hashes in loops".to_string(),
            intensity_level: IntensityLevel::Medium,
        },
        ClassifiedTaskConfig {
            name: "Mathematical Processor".to_string(),
            task_type: TaskType::CpuIntensive,
            category: TaskCategory::CpuHeavy,
            description: "Performs complex mathematical calculations".to_string(),
            intensity_level: IntensityLevel::Heavy,
        },
        ClassifiedTaskConfig {
            name: "Algorithm Solver".to_string(),
            task_type: TaskType::CpuIntensive,
            category: TaskCategory::CpuHeavy,
            description: "Solves computational algorithms".to_string(),
            intensity_level: IntensityLevel::Medium,
        },

        // IO Heavy Tasks (4 tasks)
        ClassifiedTaskConfig {
            name: "File Stream Processor".to_string(),
            task_type: TaskType::IoIntensive,
            category: TaskCategory::IoHeavy,
            description: "Processes large file streams".to_string(),
            intensity_level: IntensityLevel::Heavy,
        },
        ClassifiedTaskConfig {
            name: "Database Writer".to_string(),
            task_type: TaskType::IoIntensive,
            category: TaskCategory::IoHeavy,
            description: "Writes data to storage systems".to_string(),
            intensity_level: IntensityLevel::Medium,
        },
        ClassifiedTaskConfig {
            name: "Log Analyzer".to_string(),
            task_type: TaskType::IoIntensive,
            category: TaskCategory::IoHeavy,
            description: "Analyzes system log files".to_string(),
            intensity_level: IntensityLevel::Light,
        },
        ClassifiedTaskConfig {
            name: "Backup Manager".to_string(),
            task_type: TaskType::IoIntensive,
            category: TaskCategory::IoHeavy,
            description: "Creates and manages file backups".to_string(),
            intensity_level: IntensityLevel::Heavy,
        },

        // Network Heavy Tasks (3 tasks)
        ClassifiedTaskConfig {
            name: "API Gateway".to_string(),
            task_type: TaskType::NetworkIntensive,
            category: TaskCategory::NetworkHeavy,
            description: "Handles multiple API requests".to_string(),
            intensity_level: IntensityLevel::Heavy,
        },
        ClassifiedTaskConfig {
            name: "Web Crawler".to_string(),
            task_type: TaskType::NetworkIntensive,
            category: TaskCategory::NetworkHeavy,
            description: "Crawls web pages for data extraction".to_string(),
            intensity_level: IntensityLevel::Medium,
        },
        ClassifiedTaskConfig {
            name: "Data Synchronizer".to_string(),
            task_type: TaskType::NetworkIntensive,
            category: TaskCategory::NetworkHeavy,
            description: "Synchronizes data across network nodes".to_string(),
            intensity_level: IntensityLevel::Heavy,
        },

        // Memory Heavy Tasks (2 tasks)
        ClassifiedTaskConfig {
            name: "Cache Builder".to_string(),
            task_type: TaskType::MemoryIntensive,
            category: TaskCategory::MemoryHeavy,
            description: "Builds large in-memory caches".to_string(),
            intensity_level: IntensityLevel::Extreme,
        },
        ClassifiedTaskConfig {
            name: "Data Structure Manager".to_string(),
            task_type: TaskType::MemoryIntensive,
            category: TaskCategory::MemoryHeavy,
            description: "Manages complex data structures in memory".to_string(),
            intensity_level: IntensityLevel::Heavy,
        },

        // Balanced Task (1 task)
        ClassifiedTaskConfig {
            name: "Full-Stack Processor".to_string(),
            task_type: TaskType::Mixed,
            category: TaskCategory::Balanced,
            description: "Balanced workload across all resources".to_string(),
            intensity_level: IntensityLevel::Medium,
        },
    ]
}

fn print_task_categories(configs: &[ClassifiedTaskConfig]) {
    let mut category_counts: HashMap<TaskCategory, usize> = HashMap::new();
    
    for config in configs {
        *category_counts.entry(config.category.clone()).or_insert(0) += 1;
    }
    
    println!("\n📊 Task Category Distribution:");
    for (category, count) in &category_counts {
        let emoji = match category {
            TaskCategory::CpuHeavy => "🔥",
            TaskCategory::IoHeavy => "💾",
            TaskCategory::NetworkHeavy => "🌐",
            TaskCategory::MemoryHeavy => "🧠",
            TaskCategory::Balanced => "⚖️",
        };
        println!("   {} {:?}: {} tasks", emoji, category, count);
    }
    println!();
}

async fn execute_classified_task(
    task_id: TaskId,
    config: ClassifiedTaskConfig,
    monitor: Arc<Mutex<AsyncResourceMonitor>>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    
    // Update metrics during execution
    let update_handle = {
        let monitor_clone = Arc::clone(&monitor);
        tokio::spawn(async move {
            for _ in 0..8 {
                {
                    let mut mon = monitor_clone.lock().unwrap();
                    mon.update_metrics(task_id);
                }
                sleep(Duration::from_millis(500)).await;
            }
        })
    };
    
    // Execute task based on category and intensity
    let result = match (&config.category, &config.intensity_level) {
        (TaskCategory::CpuHeavy, intensity) => execute_cpu_heavy_task(&config.name, intensity).await,
        (TaskCategory::IoHeavy, intensity) => execute_io_heavy_task(&config.name, intensity).await,
        (TaskCategory::NetworkHeavy, intensity) => execute_network_heavy_task(&config.name, intensity).await,
        (TaskCategory::MemoryHeavy, intensity) => execute_memory_heavy_task(&config.name, intensity).await,
        (TaskCategory::Balanced, intensity) => execute_balanced_task(&config.name, intensity).await,
    };
    
    let _ = update_handle.await;
    
    match result {
        Ok(msg) => Ok(format!("{} [{}] completed: {}", config.name, config.description, msg)),
        Err(e) => Err(e),
    }
}

async fn execute_cpu_heavy_task(name: &str, intensity: &IntensityLevel) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let iterations = match intensity {
        IntensityLevel::Light => 500_000,
        IntensityLevel::Medium => 1_000_000,
        IntensityLevel::Heavy => 2_000_000,
        IntensityLevel::Extreme => 3_000_000,
    };
    
    match name {
        "Prime Number Generator" => {
            let mut count = 0;
            for n in 10_000..(10_000 + iterations / 100) {
                if is_prime(n as u64) {
                    count += 1;
                }
                if count % 100 == 0 {
                    tokio::task::yield_now().await;
                }
            }
            Ok(format!("found {} primes", count))
        },
        "Matrix Calculator" => {
            let size = match intensity {
                IntensityLevel::Light => 100,
                IntensityLevel::Medium => 200,
                IntensityLevel::Heavy => 300,
                IntensityLevel::Extreme => 400,
            };
            
            let matrix_a = vec![vec![1.0f32; size]; size];
            let matrix_b = vec![vec![2.0f32; size]; size];
            let mut result = vec![vec![0.0f32; size]; size];
            
            for i in 0..size {
                for j in 0..size {
                    for k in 0..size {
                        result[i][j] += matrix_a[i][k] * matrix_b[k][j];
                    }
                }
                if i % 20 == 0 {
                    tokio::task::yield_now().await;
                }
            }
            Ok(format!("{}x{} matrix multiplication", size, size))
        },
        _ => {
            // Generic CPU work
            for i in 0..iterations {
                let _ = (i as u64).wrapping_mul(i as u64) % 12345;
                if i % 50000 == 0 {
                    tokio::task::yield_now().await;
                }
            }
            Ok("CPU computation completed".to_string())
        }
    }
}

async fn execute_io_heavy_task(name: &str, intensity: &IntensityLevel) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let file_size_kb = match intensity {
        IntensityLevel::Light => 512,     // 512KB
        IntensityLevel::Medium => 2048,   // 2MB
        IntensityLevel::Heavy => 8192,    // 8MB
        IntensityLevel::Extreme => 16384, // 16MB
    };
    
    match name {
        "File Stream Processor" => {
            let file_path = "tmp_classified_stream.dat";
            let data = vec![0u8; file_size_kb * 1024];
            
            tokio::fs::write(file_path, &data).await?;
            
            let mut file = File::open(file_path).await?;
            let mut buffer = vec![0u8; 8192]; // 8KB buffer
            let mut total_read = 0;
            
            while let Ok(bytes_read) = file.read(&mut buffer).await {
                if bytes_read == 0 { break; }
                total_read += bytes_read;
                tokio::task::yield_now().await;
            }
            
            tokio::fs::remove_file(file_path).await.ok();
            Ok(format!("processed {} bytes", total_read))
        },
        "Database Writer" => {
            let file_path = "tmp_classified_db.dat";
            let mut file = File::create(file_path).await?;
            
            let records = file_size_kb * 10; // 10 records per KB
            for i in 0..records {
                let record = format!("Record-{:06}: {}\n", i, "X".repeat(100));
                file.write_all(record.as_bytes()).await?;
                
                if i % 100 == 0 {
                    file.flush().await?;
                    tokio::task::yield_now().await;
                }
            }
            
            tokio::fs::remove_file(file_path).await.ok();
            Ok(format!("wrote {} records", records))
        },
        _ => {
            // Generic IO work
            for i in 0..(file_size_kb / 10) {
                let file_path = format!("tmp_classified_io_{}.dat", i);
                let data = vec![i as u8; 1024]; // 1KB files
                tokio::fs::write(&file_path, &data).await?;
                let _ = tokio::fs::read(&file_path).await?;
                tokio::fs::remove_file(&file_path).await.ok();
                tokio::task::yield_now().await;
            }
            Ok("IO operations completed".to_string())
        }
    }
}

async fn execute_network_heavy_task(_name: &str, intensity: &IntensityLevel) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let request_count = match intensity {
        IntensityLevel::Light => 3,
        IntensityLevel::Medium => 6,
        IntensityLevel::Heavy => 10,
        IntensityLevel::Extreme => 15,
    };
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()?;
    
    let mut successful_requests = 0;
    
    for i in 0..request_count {
        match client.get("https://httpbin.org/get").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    successful_requests += 1;
                }
            },
            Err(_) => {
                // Simulate fallback with delay
                sleep(Duration::from_millis(100)).await;
            }
        }
        
        if i < request_count - 1 {
            sleep(Duration::from_millis(200)).await;
        }
    }
    
    Ok(format!("{}/{} requests successful", successful_requests, request_count))
}

async fn execute_memory_heavy_task(_name: &str, intensity: &IntensityLevel) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let memory_blocks = match intensity {
        IntensityLevel::Light => 100,     // ~100MB
        IntensityLevel::Medium => 200,    // ~200MB
        IntensityLevel::Heavy => 500,     // ~500MB
        IntensityLevel::Extreme => 1000,  // ~1GB
    };
    
    let mut allocated_memory = Vec::new();
    
    for i in 0..memory_blocks {
        let block = vec![i as u8; 1024 * 1024]; // 1MB blocks
        allocated_memory.push(block);
        
        if i % 50 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    // Do some work with the memory
    let mut checksum = 0u64;
    for (i, block) in allocated_memory.iter().enumerate() {
        checksum = checksum.wrapping_add(block[0] as u64);
        if i % 100 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    Ok(format!("allocated {} MB, checksum: {}", memory_blocks, checksum))
}

async fn execute_balanced_task(_name: &str, _intensity: &IntensityLevel) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // CPU work
    for i in 0..500_000u32 {
        let _ = i.wrapping_mul(i) % 12345;
    }
    
    // IO work
    let file_path = "tmp_classified_balanced.dat";
    let data = vec![42u8; 1024 * 1024]; // 1MB
    tokio::fs::write(file_path, &data).await?;
    let _ = tokio::fs::read(file_path).await?;
    tokio::fs::remove_file(file_path).await.ok();
    
    // Memory work
    let _memory_block = vec![0u8; 10 * 1024 * 1024]; // 10MB
    
    // Network work (if available)
    if let Ok(client) = reqwest::Client::builder().timeout(Duration::from_secs(1)).build() {
        let _ = client.get("https://httpbin.org/get").send().await;
    }
    
    Ok("balanced workload completed".to_string())
}

fn is_prime(n: u64) -> bool {
    if n < 2 { return false; }
    if n == 2 { return true; }
    if n % 2 == 0 { return false; }
    
    for i in (3..=(n as f64).sqrt() as u64).step_by(2) {
        if n % i == 0 { return false; }
    }
    true
}

async fn monitor_and_classify_tasks(monitor: Arc<Mutex<AsyncResourceMonitor>>, duration: Duration) {
    let start = std::time::Instant::now();
    let mut update_count = 0;
    
    while start.elapsed() < duration {
        {
            let mon = monitor.lock().unwrap();
            let profiles = mon.get_all_profiles();
            
            if update_count % 8 == 0 && !profiles.is_empty() {
                println!("\n📊 Real-time Monitoring Update #{}", update_count / 8 + 1);
                
                // Group by task type for monitoring
                let mut type_groups: HashMap<String, Vec<&TaskResourceProfile>> = HashMap::new();
                for profile in profiles.values() {
                    let task_type = format!("{:?}", profile.task_type);
                    type_groups.entry(task_type).or_default().push(profile);
                }
                
                for (task_type, group) in type_groups {
                    let avg_cpu: f64 = group.iter().map(|p| p.cpu_metrics.usage_percent).sum::<f64>() / group.len() as f64;
                    let avg_memory: f64 = group.iter().map(|p| p.memory_metrics.current_bytes as f64).sum::<f64>() / group.len() as f64 / 1_048_576.0;
                    
                    println!("   {} ({} tasks): CPU {:.1}%, Memory {:.1}MB", 
                             task_type, group.len(), avg_cpu, avg_memory);
                }
            }
        }
        
        update_count += 1;
        sleep(Duration::from_millis(500)).await;
    }
    
    println!("\n🔚 Monitoring completed after {} updates", update_count);
}

fn print_completion_summary(completed_by_category: &HashMap<TaskCategory, Vec<String>>) {
    println!("\n✅ Task Completion Summary:");
    for (category, tasks) in completed_by_category {
        let emoji = match category {
            TaskCategory::CpuHeavy => "🔥",
            TaskCategory::IoHeavy => "💾",
            TaskCategory::NetworkHeavy => "🌐",
            TaskCategory::MemoryHeavy => "🧠",
            TaskCategory::Balanced => "⚖️",
        };
        println!("   {} {:?}: {} completed", emoji, category, tasks.len());
    }
}

async fn generate_classified_html_report(monitor: &Arc<Mutex<AsyncResourceMonitor>>) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = {
        let mon = monitor.lock().unwrap();
        mon.get_all_profiles().clone()
    };
    
    let html_content = generate_classified_html_content(&profiles)?;
    tokio::fs::write("classified_async_analysis.html", html_content).await?;
    
    println!("📄 Classified HTML report generated: classified_async_analysis.html");
    Ok(())
}

fn generate_classified_html_content(profiles: &HashMap<TaskId, TaskResourceProfile>) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = String::new();
    
    html.push_str(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Classified Async Task Monitor</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        :root {
            --bg-primary: #0a0c10;
            --bg-secondary: #1a1d23;
            --bg-tertiary: #252830;
            --bg-quaternary: #2d3139;
            --border-color: #3d4349;
            --text-primary: #f8fafc;
            --text-secondary: #94a3b8;
            --text-muted: #64748b;
            --accent-cpu: #ef4444;
            --accent-io: #3b82f6;
            --accent-network: #10b981;
            --accent-memory: #8b5cf6;
            --accent-mixed: #f59e0b;
            --accent-gpu: #ec4899;
            --gradient-primary: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            --gradient-cpu: linear-gradient(135deg, #ff6b6b 0%, #ee5a24 100%);
            --gradient-io: linear-gradient(135deg, #74b9ff 0%, #0984e3 100%);
            --gradient-network: linear-gradient(135deg, #00b894 0%, #00a085 100%);
            --gradient-memory: linear-gradient(135deg, #a29bfe 0%, #6c5ce7 100%);
            --gradient-mixed: linear-gradient(135deg, #fdcb6e 0%, #e17055 100%);
        }
        
        * {
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
            margin: 0;
            padding: 0;
            background: var(--bg-primary);
            color: var(--text-primary);
            line-height: 1.6;
            overflow-x: hidden;
        }
        
        .container {
            min-height: 100vh;
            background: var(--bg-primary);
        }
        
        .header {
            background: var(--gradient-primary);
            padding: 3rem 2rem;
            text-align: center;
            position: relative;
            overflow: hidden;
        }
        
        .header::before {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><defs><pattern id="grid" width="10" height="10" patternUnits="userSpaceOnUse"><path d="M 10 0 L 0 0 0 10" fill="none" stroke="rgba(255,255,255,0.05)" stroke-width="0.5"/></pattern></defs><rect width="100" height="100" fill="url(%23grid)"/></svg>');
            z-index: 1;
        }
        
        .header-content {
            position: relative;
            z-index: 2;
        }
        
        .header h1 {
            margin: 0 0 1rem 0;
            font-size: 3rem;
            font-weight: 700;
            color: white;
            text-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
        }
        
        .header p {
            margin: 0;
            font-size: 1.25rem;
            color: rgba(255, 255, 255, 0.9);
            font-weight: 300;
        }
        
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 2rem;
            padding: 3rem 2rem;
            background: var(--bg-secondary);
            border-bottom: 1px solid var(--border-color);
        }
        
        .stat-card {
            background: var(--bg-tertiary);
            border: 1px solid var(--border-color);
            border-radius: 16px;
            padding: 2rem;
            text-align: center;
            position: relative;
            overflow: hidden;
            transition: all 0.3s ease;
        }
        
        .stat-card:hover {
            transform: translateY(-4px);
            box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
            border-color: var(--accent-cpu);
        }
        
        .stat-card::before {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 4px;
            background: var(--gradient-primary);
        }
        
        .stat-card.cpu::before { background: var(--gradient-cpu); }
        .stat-card.io::before { background: var(--gradient-io); }
        .stat-card.network::before { background: var(--gradient-network); }
        .stat-card.memory::before { background: var(--gradient-memory); }
        .stat-card.mixed::before { background: var(--gradient-mixed); }
        
        .stat-icon {
            font-size: 3rem;
            margin-bottom: 1rem;
            display: block;
        }
        
        .stat-title {
            margin: 0 0 0.5rem 0;
            font-size: 0.875rem;
            font-weight: 600;
            color: var(--text-secondary);
            text-transform: uppercase;
            letter-spacing: 1px;
        }
        
        .stat-value {
            font-size: 2.5rem;
            font-weight: 800;
            color: var(--text-primary);
            margin: 0;
            text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        }
        
        .content {
            padding: 3rem 2rem;
            max-width: 1400px;
            margin: 0 auto;
        }
        
        .section {
            margin-bottom: 4rem;
        }
        
        .section-header {
            display: flex;
            align-items: center;
            gap: 1rem;
            margin-bottom: 2rem;
        }
        
        .section-title {
            font-size: 2rem;
            font-weight: 700;
            color: var(--text-primary);
            margin: 0;
        }
        
        .section-subtitle {
            color: var(--text-secondary);
            font-size: 1rem;
            margin: 0.5rem 0 0 0;
        }
        
        .charts-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 3rem;
            margin-bottom: 4rem;
        }
        
        .chart-container {
            background: var(--bg-tertiary);
            border: 1px solid var(--border-color);
            border-radius: 16px;
            padding: 2rem;
            position: relative;
        }
        
        .chart-title {
            font-size: 1.25rem;
            font-weight: 600;
            color: var(--text-primary);
            margin: 0 0 1.5rem 0;
            text-align: center;
        }
        
        .tasks-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(450px, 1fr));
            gap: 1.5rem;
        }
        
        .task-card {
            background: var(--bg-tertiary);
            border: 1px solid var(--border-color);
            border-radius: 16px;
            overflow: hidden;
            transition: all 0.3s ease;
            position: relative;
        }
        
        .task-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 12px 24px rgba(0, 0, 0, 0.2);
        }
        
        .task-header {
            padding: 1.5rem;
            background: var(--bg-quaternary);
            border-bottom: 1px solid var(--border-color);
            position: relative;
        }
        
        .task-header::before {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 3px;
        }
        
        .task-header.cpu::before { background: var(--gradient-cpu); }
        .task-header.io::before { background: var(--gradient-io); }
        .task-header.network::before { background: var(--gradient-network); }
        .task-header.memory::before { background: var(--gradient-memory); }
        .task-header.mixed::before { background: var(--gradient-mixed); }
        
        .task-name {
            font-size: 1.125rem;
            font-weight: 600;
            color: var(--text-primary);
            margin: 0 0 0.75rem 0;
        }
        
        .task-meta {
            display: flex;
            gap: 0.75rem;
            flex-wrap: wrap;
        }
        
        .task-badge {
            padding: 0.25rem 0.75rem;
            border-radius: 12px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        
        .task-badge.cpu { background: var(--accent-cpu); color: white; }
        .task-badge.io { background: var(--accent-io); color: white; }
        .task-badge.network { background: var(--accent-network); color: white; }
        .task-badge.memory { background: var(--accent-memory); color: white; }
        .task-badge.mixed { background: var(--accent-mixed); color: white; }
        
        .task-metrics {
            padding: 1.5rem;
        }
        
        .metrics-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 1rem;
            margin-bottom: 1.5rem;
        }
        
        .metric-item {
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 1rem;
            text-align: center;
        }
        
        .metric-label {
            font-size: 0.75rem;
            font-weight: 500;
            color: var(--text-muted);
            text-transform: uppercase;
            letter-spacing: 0.5px;
            margin-bottom: 0.5rem;
        }
        
        .metric-value {
            font-size: 1.25rem;
            font-weight: 700;
            color: var(--text-primary);
        }
        
        .performance-bar {
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 1rem;
        }
        
        .performance-label {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 0.5rem;
        }
        
        .performance-label span {
            font-size: 0.875rem;
            font-weight: 500;
            color: var(--text-secondary);
        }
        
        .performance-label .value {
            font-weight: 700;
            color: var(--text-primary);
        }
        
        .progress-track {
            width: 100%;
            height: 6px;
            background: var(--border-color);
            border-radius: 3px;
            overflow: hidden;
        }
        
        .progress-fill {
            height: 100%;
            background: linear-gradient(90deg, var(--accent-network), var(--accent-mixed), var(--accent-cpu));
            border-radius: 3px;
            transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1);
        }
        
        .bottleneck-indicator {
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.5rem 1rem;
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 20px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            margin-top: 1rem;
        }
        
        .bottleneck-indicator.cpu { border-color: var(--accent-cpu); color: var(--accent-cpu); }
        .bottleneck-indicator.memory { border-color: var(--accent-memory); color: var(--accent-memory); }
        .bottleneck-indicator.io { border-color: var(--accent-io); color: var(--accent-io); }
        .bottleneck-indicator.network { border-color: var(--accent-network); color: var(--accent-network); }
        .bottleneck-indicator.balanced { border-color: var(--accent-mixed); color: var(--accent-mixed); }
        
        /* Enhanced feature styles */
        .source-location, .deep-dive-section {
            margin: 1rem 0;
            border: 1px solid var(--border-color);
            border-radius: 8px;
            background: var(--bg-secondary);
        }
        
        .section-toggle {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0.75rem 1rem;
            background: var(--bg-quaternary);
            border-radius: 8px 8px 0 0;
            transition: background-color 0.2s ease;
        }
        
        .section-toggle:hover {
            background: var(--bg-tertiary);
        }
        
        .toggle-icon {
            font-size: 1rem;
            margin-right: 0.5rem;
        }
        
        .toggle-label {
            font-weight: 600;
            color: var(--text-primary);
            flex-grow: 1;
        }
        
        .chevron {
            font-size: 0.8rem;
            color: var(--text-secondary);
            transition: transform 0.3s ease;
        }
        
        .chevron.expanded {
            transform: rotate(180deg);
        }
        
        .collapsible-content {
            overflow: hidden;
            transition: max-height 0.3s ease;
        }
        
        /* Source Location Styles */
        .location-info {
            padding: 1rem;
            background: var(--bg-tertiary);
        }
        
        .location-item {
            display: flex;
            justify-content: space-between;
            margin-bottom: 0.5rem;
            padding: 0.25rem 0;
            border-bottom: 1px solid var(--border-color);
        }
        
        .location-item:last-child {
            border-bottom: none;
            margin-bottom: 0;
        }
        
        .location-label {
            font-weight: 500;
            color: var(--text-secondary);
            min-width: 80px;
        }
        
        .location-value {
            font-family: 'Courier New', monospace;
            color: var(--text-primary);
            font-size: 0.9rem;
            text-align: right;
            flex-grow: 1;
            margin-left: 1rem;
        }
        
        /* Hot Metrics Styles */
        .hotspots-grid {
            padding: 1rem;
            background: var(--bg-tertiary);
        }
        
        .hotspot-category {
            margin-bottom: 1.5rem;
        }
        
        .hotspot-category:last-child {
            margin-bottom: 0;
        }
        
        .hotspot-category h5 {
            margin: 0 0 0.75rem 0;
            color: var(--text-primary);
            font-size: 0.9rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        
        .hotspot-list {
            display: flex;
            flex-direction: column;
            gap: 0.5rem;
        }
        
        .hotspot-item {
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 6px;
            padding: 0.75rem;
        }
        
        .hotspot-name {
            font-weight: 600;
            color: var(--text-primary);
            font-size: 0.9rem;
            margin-bottom: 0.25rem;
        }
        
        .hotspot-stats {
            display: flex;
            justify-content: space-between;
            font-size: 0.8rem;
            color: var(--text-secondary);
        }
        
        /* Efficiency Analysis Styles */
        .efficiency-breakdown {
            padding: 1rem;
            background: var(--bg-tertiary);
        }
        
        .component-scores {
            margin-bottom: 1.5rem;
        }
        
        .component-scores h5 {
            margin: 0 0 1rem 0;
            color: var(--text-primary);
            font-size: 0.9rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        
        .score-item {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 0.5rem;
            padding: 0.5rem;
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 6px;
        }
        
        .score-value {
            font-weight: 700;
            color: var(--accent-blue);
        }
        
        .bottleneck-analysis {
            margin-bottom: 1.5rem;
        }
        
        .bottleneck-analysis h5 {
            margin: 0 0 0.75rem 0;
            color: var(--text-primary);
            font-size: 0.9rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        
        .analysis-text {
            margin: 0;
            padding: 1rem;
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 6px;
            color: var(--text-secondary);
            line-height: 1.5;
            font-size: 0.9rem;
        }
        
        .recommendations {
            margin-bottom: 1.5rem;
        }
        
        .recommendations h5 {
            margin: 0 0 1rem 0;
            color: var(--text-primary);
            font-size: 0.9rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        
        .recommendations-list {
            display: flex;
            flex-direction: column;
            gap: 0.75rem;
        }
        
        .recommendation-item {
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 6px;
            padding: 1rem;
        }
        
        .rec-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 0.5rem;
        }
        
        .rec-category {
            font-weight: 600;
            color: var(--text-primary);
            font-size: 0.9rem;
        }
        
        .rec-impact {
            padding: 0.25rem 0.5rem;
            border-radius: 12px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
        }
        
        .rec-impact.impact-high {
            background: var(--accent-orange);
            color: white;
        }
        
        .rec-impact.impact-medium {
            background: var(--accent-yellow);
            color: black;
        }
        
        .rec-impact.impact-low {
            background: var(--accent-green);
            color: white;
        }
        
        .rec-description {
            color: var(--text-secondary);
            line-height: 1.4;
            margin-bottom: 0.5rem;
            font-size: 0.9rem;
        }
        
        .rec-improvement {
            color: var(--accent-blue);
            font-weight: 600;
            font-size: 0.8rem;
        }
        
        .optimization-potential h5 {
            margin: 0 0 0.75rem 0;
            color: var(--text-primary);
            font-size: 0.9rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        
        .potential-bar {
            position: relative;
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            height: 40px;
            overflow: hidden;
        }
        
        .potential-fill {
            height: 100%;
            background: linear-gradient(90deg, var(--accent-green), var(--accent-yellow));
            transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1);
        }
        
        .potential-text {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            font-weight: 600;
            color: var(--text-primary);
            font-size: 0.9rem;
            text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
        }
        
        /* Deep Dive Analysis Styles */
        .deep-dive-content {
            padding: 1.5rem;
            background: var(--bg-tertiary);
        }
        
        .efficiency-overview {
            margin-bottom: 2rem;
        }
        
        .efficiency-overview h5 {
            margin: 0 0 1rem 0;
            color: var(--text-primary);
            font-size: 1rem;
            font-weight: 600;
        }
        
        .efficiency-grid {
            display: grid;
            gap: 0.75rem;
        }
        
        .efficiency-metric {
            display: flex;
            align-items: center;
            gap: 1rem;
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 0.75rem;
        }
        
        .metric-name {
            min-width: 60px;
            font-weight: 600;
            color: var(--text-secondary);
            font-size: 0.9rem;
        }
        
        .metric-bar {
            flex-grow: 1;
            height: 8px;
            background: var(--border-color);
            border-radius: 4px;
            overflow: hidden;
        }
        
        .metric-fill {
            height: 100%;
            border-radius: 4px;
            transition: width 0.6s ease;
        }
        
        .cpu-fill { background: linear-gradient(90deg, var(--accent-orange), #ff8a80); }
        .memory-fill { background: linear-gradient(90deg, var(--accent-purple), #b39ddb); }
        .io-fill { background: linear-gradient(90deg, var(--accent-blue), #90caf9); }
        .network-fill { background: linear-gradient(90deg, var(--accent-green), #a5d6a7); }
        
        .metric-percent {
            min-width: 45px;
            text-align: right;
            font-weight: 700;
            color: var(--text-primary);
            font-size: 0.9rem;
        }
        
        /* Hotspots Analysis Styles */
        .hotspots-analysis {
            margin-bottom: 2rem;
        }
        
        .hotspots-analysis h5 {
            margin: 0 0 1rem 0;
            color: var(--text-primary);
            font-size: 1rem;
            font-weight: 600;
        }
        
        .hotspots-tabs {
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            overflow: hidden;
        }
        
        .tab-buttons {
            display: flex;
            background: var(--bg-quaternary);
            border-bottom: 1px solid var(--border-color);
        }
        
        .tab-btn {
            flex: 1;
            padding: 0.75rem 1rem;
            background: transparent;
            border: none;
            color: var(--text-secondary);
            font-weight: 500;
            cursor: pointer;
            transition: all 0.2s ease;
        }
        
        .tab-btn:hover {
            background: var(--bg-tertiary);
            color: var(--text-primary);
        }
        
        .tab-btn.active {
            background: var(--accent-blue);
            color: white;
        }
        
        .tab-content {
            display: none;
            padding: 1rem;
        }
        
        .tab-content.active {
            display: block;
        }
        
        .hotspot-detail {
            background: var(--bg-quaternary);
            border: 1px solid var(--border-color);
            border-radius: 6px;
            padding: 1rem;
            margin-bottom: 0.75rem;
        }
        
        .hotspot-detail:last-child {
            margin-bottom: 0;
        }
        
        .hotspot-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 0.5rem;
        }
        
        .hotspot-function {
            font-weight: 600;
            color: var(--text-primary);
            font-size: 0.9rem;
        }
        
        .hotspot-impact {
            font-weight: 700;
            color: var(--accent-orange);
            font-size: 0.9rem;
        }
        
        .hotspot-metrics {
            display: flex;
            gap: 1rem;
            flex-wrap: wrap;
            font-size: 0.8rem;
            color: var(--text-secondary);
        }
        
        .hotspot-path {
            margin-top: 0.5rem;
            font-family: 'Courier New', monospace;
            font-size: 0.75rem;
            color: var(--text-muted);
            padding: 0.25rem 0.5rem;
            background: var(--bg-secondary);
            border-radius: 4px;
        }
        
        /* Intelligent Diagnosis Styles */
        .intelligent-diagnosis h5 {
            margin: 0 0 1rem 0;
            color: var(--text-primary);
            font-size: 1rem;
            font-weight: 600;
        }
        
        .diagnosis-content {
            display: flex;
            flex-direction: column;
            gap: 1.5rem;
        }
        
        .bottleneck-insight {
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 1rem;
        }
        
        .insight-header {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            margin-bottom: 0.75rem;
        }
        
        .insight-icon {
            font-size: 1.2rem;
        }
        
        .insight-title {
            font-weight: 600;
            color: var(--text-primary);
        }
        
        .insight-text {
            color: var(--text-secondary);
            line-height: 1.5;
            font-size: 0.9rem;
        }
        
        .optimization-suggestions {
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 1rem;
        }
        
        .suggestions-header {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            margin-bottom: 1rem;
        }
        
        .suggestions-icon {
            font-size: 1.2rem;
        }
        
        .suggestions-title {
            font-weight: 600;
            color: var(--text-primary);
        }
        
        .suggestions-list {
            display: flex;
            flex-direction: column;
            gap: 0.75rem;
        }
        
        .suggestion-item {
            background: var(--bg-tertiary);
            border: 1px solid var(--border-color);
            border-radius: 6px;
            padding: 0.75rem;
        }
        
        .suggestion-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 0.5rem;
        }
        
        .suggestion-category {
            font-weight: 600;
            color: var(--text-primary);
            font-size: 0.9rem;
        }
        
        .suggestion-impact {
            padding: 0.2rem 0.5rem;
            border-radius: 10px;
            font-size: 0.7rem;
            font-weight: 600;
            text-transform: uppercase;
        }
        
        .suggestion-text {
            color: var(--text-secondary);
            line-height: 1.4;
            font-size: 0.85rem;
            margin-bottom: 0.5rem;
        }
        
        .suggestion-improvement {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            color: var(--accent-green);
            font-weight: 600;
            font-size: 0.8rem;
        }
        
        .improvement-icon {
            font-size: 0.9rem;
        }
        
        .optimization-summary {
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 1rem;
        }
        
        .summary-header {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            margin-bottom: 1rem;
        }
        
        .summary-icon {
            font-size: 1.2rem;
        }
        
        .summary-title {
            font-weight: 600;
            color: var(--text-primary);
        }
        
        .potential-display {
            display: flex;
            align-items: center;
            gap: 1rem;
        }
        
        .potential-bar {
            flex-grow: 1;
            height: 12px;
            background: var(--border-color);
            border-radius: 6px;
            overflow: hidden;
        }
        
        .potential-fill {
            height: 100%;
            background: linear-gradient(90deg, var(--accent-green), var(--accent-yellow));
            border-radius: 6px;
            transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1);
        }
        
        .potential-percentage {
            font-weight: 700;
            color: var(--accent-green);
            font-size: 1rem;
            min-width: 50px;
            text-align: right;
        }
        
        @media (max-width: 1400px) {
            .tasks-grid {
                grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
            }
        }
        
        @media (max-width: 1024px) {
            .charts-grid {
                grid-template-columns: 1fr;
            }
            .tasks-grid {
                grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
            }
        }
        
        @media (max-width: 768px) {
            .header {
                padding: 2rem 1rem;
            }
            
            .header h1 {
                font-size: 2rem;
            }
            
            .stats-grid {
                grid-template-columns: 1fr;
                padding: 2rem 1rem;
            }
            
            .content {
                padding: 2rem 1rem;
            }
            
            .tasks-grid {
                grid-template-columns: 1fr;
            }
            
            .metrics-grid {
                grid-template-columns: 1fr;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="header-content">
                <h1>⚡ Classified Task Monitor</h1>
                <p>Advanced performance analysis of categorized async tasks</p>
            </div>
        </div>
"#);

    // Calculate statistics by category
    let _category_stats = calculate_category_statistics(profiles);
    let total_tasks = profiles.len();
    
    // Generate summary statistics
    let avg_cpu = profiles.values().map(|p| p.cpu_metrics.usage_percent).sum::<f64>() / total_tasks as f64;
    let avg_memory = profiles.values().map(|p| p.memory_metrics.current_bytes as f64).sum::<f64>() / total_tasks as f64 / 1_048_576.0;
    let avg_efficiency = profiles.values().map(|p| p.efficiency_score).sum::<f64>() / total_tasks as f64;
    
    // Count by task type
    let mut type_counts = HashMap::new();
    let mut bottleneck_counts = HashMap::new();
    
    for profile in profiles.values() {
        let task_type = format!("{:?}", profile.task_type);
        *type_counts.entry(task_type).or_insert(0) += 1;
        
        let bottleneck = format!("{:?}", profile.bottleneck_type);
        *bottleneck_counts.entry(bottleneck).or_insert(0) += 1;
    }

    // Stats grid
    html.push_str(&format!(r#"
        <div class="stats-grid">
            <div class="stat-card">
                <span class="stat-icon">📊</span>
                <h3 class="stat-title">Total Tasks</h3>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card cpu">
                <span class="stat-icon">🔥</span>
                <h3 class="stat-title">Avg CPU Usage</h3>
                <div class="stat-value">{:.1}%</div>
            </div>
            <div class="stat-card memory">
                <span class="stat-icon">🧠</span>
                <h3 class="stat-title">Avg Memory</h3>
                <div class="stat-value">{:.0}MB</div>
            </div>
            <div class="stat-card mixed">
                <span class="stat-icon">⚡</span>
                <h3 class="stat-title">Avg Efficiency</h3>
                <div class="stat-value">{:.0}%</div>
            </div>
        </div>
"#, total_tasks, avg_cpu, avg_memory, avg_efficiency * 100.0));

    // Charts section
    html.push_str(r#"
        <div class="content">
            <div class="section">
                <div class="section-header">
                    <h2 class="section-title">📈 Performance Analytics</h2>
                </div>
                <div class="charts-grid">
                    <div class="chart-container">
                        <h3 class="chart-title">Task Type Distribution</h3>
                        <canvas id="taskTypeChart"></canvas>
                    </div>
                    <div class="chart-container">
                        <h3 class="chart-title">Resource Bottlenecks</h3>
                        <canvas id="bottleneckChart"></canvas>
                    </div>
                </div>
            </div>
"#);

    // Task details section
    html.push_str(r#"
            <div class="section">
                <div class="section-header">
                    <h2 class="section-title">🎯 Task Performance Details</h2>
                    <p class="section-subtitle">Individual task metrics and resource utilization patterns</p>
                </div>
                <div class="tasks-grid">
"#);

    // Sort tasks by efficiency score (descending)
    let mut sorted_profiles: Vec<_> = profiles.iter().collect();
    sorted_profiles.sort_by(|a, b| b.1.efficiency_score.partial_cmp(&a.1.efficiency_score).unwrap_or(std::cmp::Ordering::Equal));

    for (task_id, profile) in sorted_profiles {
        let task_type_class = format!("{:?}", profile.task_type).to_lowercase();
        let bottleneck_class = format!("{:?}", profile.bottleneck_type).to_lowercase();
        
        html.push_str(&format!(r#"
                    <div class="task-card">
                        <div class="task-header {}">
                            <h4 class="task-name">{}</h4>
                            <div class="task-meta">
                                <span class="task-badge {}">{:?}</span>
                                <span class="task-badge" style="background: var(--bg-secondary); color: var(--text-secondary);">ID: {}</span>
                            </div>
                        </div>
                        
                        <!-- Source Location Section -->
                        <div class="source-location">
                            <div class="section-toggle" onclick="toggleSection('source-{}')" style="cursor: pointer;">
                                <span class="toggle-icon">📍</span>
                                <span class="toggle-label">Source Location</span>
                                <span class="chevron">▼</span>
                            </div>
                            <div id="source-{}" class="collapsible-content">
                                <div class="location-info">
                                    <div class="location-item">
                                        <span class="location-label">File:</span>
                                        <span class="location-value">{}</span>
                                    </div>
                                    <div class="location-item">
                                        <span class="location-label">Line:</span>
                                        <span class="location-value">{}</span>
                                    </div>
                                    <div class="location-item">
                                        <span class="location-label">Function:</span>
                                        <span class="location-value">{}</span>
                                    </div>
                                    <div class="location-item">
                                        <span class="location-label">Module:</span>
                                        <span class="location-value">{}</span>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="task-metrics">
                            <div class="metrics-grid">
                                <div class="metric-item">
                                    <div class="metric-label">CPU Usage</div>
                                    <div class="metric-value">{:.1}%</div>
                                </div>
                                <div class="metric-item">
                                    <div class="metric-label">Memory</div>
                                    <div class="metric-value">{:.0}MB</div>
                                </div>
                                <div class="metric-item">
                                    <div class="metric-label">IO Bandwidth</div>
                                    <div class="metric-value">{:.1}MB/s</div>
                                </div>
                                <div class="metric-item">
                                    <div class="metric-label">Network</div>
                                    <div class="metric-value">{:.1}Mbps</div>
                                </div>
                            </div>
                            
                            <!-- Deep Dive Analysis Section (合并热点和效率分析) -->
                            <div class="deep-dive-section">
                                <div class="section-toggle" onclick="toggleSection('deepdive-{}')" style="cursor: pointer;">
                                    <span class="toggle-icon">🔬</span>
                                    <span class="toggle-label">Deep Dive Analysis</span>
                                    <span class="chevron">▼</span>
                                </div>
                                <div id="deepdive-{}" class="collapsible-content">
                                    <div class="hotspots-grid">
                                        <div class="hotspot-category">
                                            <h5>CPU Hotspots</h5>
                                            <div class="hotspot-list">
"#, 
            task_type_class,
            profile.task_name,
            task_type_class,
            profile.task_type,
            task_id,
            task_id, task_id,
            profile.source_location.file_path,
            profile.source_location.line_number,
            profile.source_location.function_name,
            profile.source_location.module_path,
            profile.cpu_metrics.usage_percent,
            profile.memory_metrics.current_bytes as f64 / 1_048_576.0,
            profile.io_metrics.bandwidth_mbps,
            profile.network_metrics.throughput_mbps,
            task_id, task_id
        ));

        // 整合的深度分析内容
        html.push_str(&format!(r#"
                                    <div class="deep-dive-content">
                                        <!-- 综合效率概览 -->
                                        <div class="efficiency-overview">
                                            <h5>📊 Performance Overview</h5>
                                            <div class="efficiency-grid">
                                                <div class="efficiency-metric">
                                                    <span class="metric-name">CPU</span>
                                                    <div class="metric-bar">
                                                        <div class="metric-fill cpu-fill" style="width: {:.1}%"></div>
                                                    </div>
                                                    <span class="metric-percent">{:.1}%</span>
                                                </div>
                                                <div class="efficiency-metric">
                                                    <span class="metric-name">Memory</span>
                                                    <div class="metric-bar">
                                                        <div class="metric-fill memory-fill" style="width: {:.1}%"></div>
                                                    </div>
                                                    <span class="metric-percent">{:.1}%</span>
                                                </div>
                                                <div class="efficiency-metric">
                                                    <span class="metric-name">IO</span>
                                                    <div class="metric-bar">
                                                        <div class="metric-fill io-fill" style="width: {:.1}%"></div>
                                                    </div>
                                                    <span class="metric-percent">{:.1}%</span>
                                                </div>
                                                <div class="efficiency-metric">
                                                    <span class="metric-name">Network</span>
                                                    <div class="metric-bar">
                                                        <div class="metric-fill network-fill" style="width: {:.1}%"></div>
                                                    </div>
                                                    <span class="metric-percent">{:.1}%</span>
                                                </div>
                                            </div>
                                        </div>

                                        <!-- 热点分析 -->
                                        <div class="hotspots-analysis">
                                            <h5>🔥 Performance Hotspots</h5>
                                            <div class="hotspots-tabs">
                                                <div class="tab-buttons">
                                                    <button class="tab-btn active" onclick="switchTab('cpu-{}')" data-tab="cpu-{}">CPU</button>
                                                    <button class="tab-btn" onclick="switchTab('memory-{}')" data-tab="memory-{}">Memory</button>
                                                    <button class="tab-btn" onclick="switchTab('io-{}')" data-tab="io-{}">IO</button>
                                                    <button class="tab-btn" onclick="switchTab('network-{}')" data-tab="network-{}">Network</button>
                                                </div>
                                                
                                                <div id="cpu-{}" class="tab-content active">
"#, 
            profile.efficiency_explanation.component_scores.cpu_efficiency * 100.0,
            profile.efficiency_explanation.component_scores.cpu_efficiency * 100.0,
            profile.efficiency_explanation.component_scores.memory_efficiency * 100.0,
            profile.efficiency_explanation.component_scores.memory_efficiency * 100.0,
            profile.efficiency_explanation.component_scores.io_efficiency * 100.0,
            profile.efficiency_explanation.component_scores.io_efficiency * 100.0,
            profile.efficiency_explanation.component_scores.network_efficiency * 100.0,
            profile.efficiency_explanation.component_scores.network_efficiency * 100.0,
            task_id, task_id, task_id, task_id, task_id, task_id, task_id, task_id, task_id
        ));

        // CPU热点
        for hotspot in &profile.hot_metrics.cpu_hotspots {
            html.push_str(&format!(r#"
                                                    <div class="hotspot-detail">
                                                        <div class="hotspot-header">
                                                            <span class="hotspot-function">{}</span>
                                                            <span class="hotspot-impact">{:.1}%</span>
                                                        </div>
                                                        <div class="hotspot-metrics">
                                                            <span>⏱️ {:.1}ms total</span>
                                                            <span>📞 {} calls</span>
                                                            <span>⚡ {:.2}ms/call</span>
                                                        </div>
                                                    </div>
"#, hotspot.function_name, hotspot.percentage_of_total, hotspot.cpu_time_ms, hotspot.call_count, hotspot.avg_time_per_call));
        }

        html.push_str(&format!(r#"
                                                </div>
                                                
                                                <div id="memory-{}" class="tab-content">
"#, task_id));

        // Memory热点
        for hotspot in &profile.hot_metrics.memory_hotspots {
            html.push_str(&format!(r#"
                                                    <div class="hotspot-detail">
                                                        <div class="hotspot-header">
                                                            <span class="hotspot-function">{}</span>
                                                            <span class="hotspot-impact">{:.1}MB</span>
                                                        </div>
                                                        <div class="hotspot-metrics">
                                                            <span>📦 {} allocations</span>
                                                            <span>📈 {:.1}MB peak</span>
                                                            <span>⏳ {:.0}ms lifetime</span>
                                                        </div>
                                                    </div>
"#, hotspot.allocation_site, hotspot.bytes_allocated as f64 / 1_048_576.0, hotspot.allocation_count, hotspot.peak_usage as f64 / 1_048_576.0, hotspot.lifetime_ms));
        }

        html.push_str(&format!(r#"
                                                </div>
                                                
                                                <div id="io-{}" class="tab-content">
"#, task_id));

        // IO热点
        for hotspot in &profile.hot_metrics.io_hotspots {
            html.push_str(&format!(r#"
                                                    <div class="hotspot-detail">
                                                        <div class="hotspot-header">
                                                            <span class="hotspot-function">{}</span>
                                                            <span class="hotspot-impact">{:.1}MB</span>
                                                        </div>
                                                        <div class="hotspot-metrics">
                                                            <span>🔄 {} ops</span>
                                                            <span>⏱️ {:.1}ms total</span>
                                                            <span>📊 {:.1}ms avg</span>
                                                        </div>
                                                        <div class="hotspot-path">{}</div>
                                                    </div>
"#, hotspot.operation_type, hotspot.bytes_processed as f64 / 1_048_576.0, hotspot.operation_count, hotspot.total_time_ms, hotspot.avg_latency_ms, hotspot.file_path));
        }

        html.push_str(&format!(r#"
                                                </div>
                                                
                                                <div id="network-{}" class="tab-content">
"#, task_id));

        // Network热点
        for hotspot in &profile.hot_metrics.network_hotspots {
            html.push_str(&format!(r#"
                                                    <div class="hotspot-detail">
                                                        <div class="hotspot-header">
                                                            <span class="hotspot-function">{}</span>
                                                            <span class="hotspot-impact">{:.1}MB</span>
                                                        </div>
                                                        <div class="hotspot-metrics">
                                                            <span>📡 {} requests</span>
                                                            <span>⏱️ {:.1}ms avg</span>
                                                            <span>❌ {:.1}% errors</span>
                                                        </div>
                                                    </div>
"#, hotspot.endpoint, hotspot.bytes_transferred as f64 / 1_048_576.0, hotspot.request_count, hotspot.avg_response_time_ms, hotspot.error_rate * 100.0));
        }

        html.push_str(r#"
                                                </div>
                                            </div>
                                        </div>

                                        <!-- 智能诊断和建议 -->
                                        <div class="intelligent-diagnosis">
                                            <h5>🎯 Intelligent Diagnosis</h5>
                                            <div class="diagnosis-content">
                                                <div class="bottleneck-insight">
                                                    <div class="insight-header">
                                                        <span class="insight-icon">🔍</span>
                                                        <span class="insight-title">Primary Bottleneck</span>
                                                    </div>
                                                    <div class="insight-text">"#);

        html.push_str(&profile.efficiency_explanation.bottleneck_analysis);

        html.push_str(&format!(r#"</div>
                                                </div>
                                                
                                                <div class="optimization-suggestions">
                                                    <div class="suggestions-header">
                                                        <span class="suggestions-icon">💡</span>
                                                        <span class="suggestions-title">Optimization Recommendations</span>
                                                    </div>
                                                    <div class="suggestions-list">
"#));

        // Add recommendations  
        for rec in &profile.efficiency_explanation.recommendations {
            html.push_str(&format!(r#"
                                                        <div class="suggestion-item">
                                                            <div class="suggestion-header">
                                                                <span class="suggestion-category">{}</span>
                                                                <span class="suggestion-impact impact-{}">{}</span>
                                                            </div>
                                                            <div class="suggestion-text">{}</div>
                                                            <div class="suggestion-improvement">
                                                                <span class="improvement-icon">📈</span>
                                                                <span>+{:.1}% potential improvement</span>
                                                            </div>
                                                        </div>
"#, rec.category, rec.impact.to_lowercase(), rec.impact, rec.description, rec.estimated_improvement));
        }

        html.push_str(&format!(r#"
                                                    </div>
                                                </div>
                                                
                                                <div class="optimization-summary">
                                                    <div class="summary-header">
                                                        <span class="summary-icon">🚀</span>
                                                        <span class="summary-title">Optimization Potential</span>
                                                    </div>
                                                    <div class="potential-display">
                                                        <div class="potential-bar">
                                                            <div class="potential-fill" style="width: {:.1}%"></div>
                                                        </div>
                                                        <span class="potential-percentage">{:.1}%</span>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div class="performance-bar">
                                <div class="performance-label">
                                    <span>Overall Efficiency Score</span>
                                    <span class="value">{:.1}%</span>
                                </div>
                                <div class="progress-track">
                                    <div class="progress-fill" style="width: {:.1}%"></div>
                                </div>
                            </div>
                            
                            <div class="bottleneck-indicator {}">
                                <span>🎯</span>
                                <span>Bottleneck: {:?}</span>
                            </div>
                        </div>
                    </div>
"#, 
            profile.efficiency_explanation.optimization_potential,
            profile.efficiency_explanation.optimization_potential,
            profile.efficiency_score * 100.0,
            profile.efficiency_score * 100.0,
            bottleneck_class,
            profile.bottleneck_type
        ));
    }

    html.push_str(r#"
                </div>
            </div>
        </div>
    </div>

    <script>
        // Task Type Chart
        const taskTypeCtx = document.getElementById('taskTypeChart').getContext('2d');
        new Chart(taskTypeCtx, {
            type: 'doughnut',
            data: {
                labels: ["#);
    
    let type_labels: Vec<String> = type_counts.keys().cloned().collect();
    let type_values: Vec<String> = type_labels.iter()
        .map(|label| type_counts.get(label).unwrap_or(&0).to_string())
        .collect();
    
    html.push_str(&type_labels.join("\", \""));
    
    html.push_str(&format!(r#""],
                datasets: [{{
                    data: [{}],
                    backgroundColor: ['#ef4444', '#3b82f6', '#10b981', '#8b5cf6', '#f59e0b'],
                    borderWidth: 2,
                    borderColor: '#1a1d23'
                }}]
            }},
            options: {{
                responsive: true,
                plugins: {{
                    legend: {{
                        position: 'bottom',
                        labels: {{
                            color: '#f8fafc',
                            font: {{ size: 12 }}
                        }}
                    }}
                }}
            }}
        }});"#, type_values.join(", ")));

        // Bottleneck Chart
        const bottleneckCtx = document.getElementById('bottleneckChart').getContext('2d');
        new Chart(bottleneckCtx, {{
            type: 'bar',
            data: {{
                labels: ["#);
    
    let bottleneck_labels: Vec<String> = bottleneck_counts.keys().cloned().collect();
    let bottleneck_values: Vec<String> = bottleneck_labels.iter()
        .map(|label| bottleneck_counts.get(label).unwrap_or(&0).to_string())
        .collect();
    
    html.push_str(&bottleneck_labels.join("\", \""));
    
    html.push_str(&format!(r#""],
                datasets: [{{
                    label: 'Task Count',
                    data: [{}],
                    backgroundColor: ['#ef4444', '#8b5cf6', '#3b82f6', '#10b981', '#f59e0b'],
                    borderWidth: 1,
                    borderColor: '#3d4349'
                }}]
            }},
            options: {{
                responsive: true,
                scales: {{
                    y: {{
                        beginAtZero: true,
                        ticks: {{
                            color: '#94a3b8',
                            stepSize: 1
                        }},
                        grid: {{
                            color: '#3d4349'
                        }}
                    }},
                    x: {{
                        ticks: {{
                            color: '#94a3b8'
                        }},
                        grid: {{
                            color: '#3d4349'
                        }}
                    }}
                }},
                plugins: {{
                    legend: {{
                        display: false
                    }}
                }}
            }}
        }});"#, bottleneck_values.join(", ")));
        
        // Toggle section functionality
        function toggleSection(sectionId) {{
            const content = document.getElementById(sectionId);
            const chevron = content.previousElementSibling.querySelector('.chevron');
            
            if (content.style.maxHeight === '0px' || content.style.maxHeight === '') {{
                content.style.maxHeight = content.scrollHeight + 'px';
                chevron.classList.add('expanded');
            }} else {{
                content.style.maxHeight = '0px';
                chevron.classList.remove('expanded');
            }}
        }}
        
        // Switch tab functionality for hotspots
        function switchTab(targetId) {{
            // Get the parent tab container
            const tabContainer = document.getElementById(targetId).closest('.hotspots-tabs');
            
            // Hide all tab contents in this container
            const tabContents = tabContainer.querySelectorAll('.tab-content');
            tabContents.forEach(content => {{
                content.classList.remove('active');
            }});
            
            // Remove active class from all tab buttons in this container
            const tabButtons = tabContainer.querySelectorAll('.tab-btn');
            tabButtons.forEach(btn => {{
                btn.classList.remove('active');
            }});
            
            // Show target tab content
            document.getElementById(targetId).classList.add('active');
            
            // Add active class to clicked button
            event.target.classList.add('active');
        }}
        
        // Initialize page
        document.addEventListener('DOMContentLoaded', function() {{
            // Initialize all collapsible content as closed
            const collapsibles = document.querySelectorAll('.collapsible-content');
            collapsibles.forEach(content => {{
                content.style.maxHeight = '0px';
                content.style.overflow = 'hidden';
                content.style.transition = 'max-height 0.3s ease';
            }});
            
            // Add click handlers to section toggles
            const toggles = document.querySelectorAll('.section-toggle');
            toggles.forEach(toggle => {{
                toggle.addEventListener('click', function() {{
                    const onclickAttr = this.getAttribute('onclick');
                    if (onclickAttr) {{
                        const match = onclickAttr.match(/toggleSection\\('([^']+)'\\)/);
                        if (match) {{
                            toggleSection(match[1]);
                        }}
                    }}
                }});
            }});
        }});
    </script>
</body>
</html>
"#);

    Ok(html)
}

fn calculate_category_statistics(profiles: &HashMap<TaskId, TaskResourceProfile>) -> HashMap<String, (usize, f64, f64)> {
    let mut stats = HashMap::new();
    
    for profile in profiles.values() {
        let category = format!("{:?}", profile.task_type);
        let entry = stats.entry(category).or_insert((0, 0.0, 0.0));
        
        entry.0 += 1; // count
        entry.1 += profile.cpu_metrics.usage_percent; // total cpu
        entry.2 += profile.memory_metrics.current_bytes as f64; // total memory
    }
    
    // Convert totals to averages
    for (_, (count, cpu_total, memory_total)) in stats.iter_mut() {
        if *count > 0 {
            *cpu_total /= *count as f64;
            *memory_total /= *count as f64;
        }
    }
    
    stats
}