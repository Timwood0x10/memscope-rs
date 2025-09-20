//! Simple Classified Async Task Monitor
//!
//! A simplified version without complex JavaScript to avoid compilation issues

use memscope_rs::async_memory::{
    self, TaskType, TaskId, AsyncResourceMonitor, TaskResourceProfile,
    resource_monitor::SourceLocation
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

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

#[derive(Debug, Clone)]
struct TaskConfig {
    name: String,
    task_type: TaskType,
    category: TaskCategory,
    description: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Simple Classified Async Task Monitor");
    println!("=====================================");
    
    // Initialize the async memory tracking system
    async_memory::initialize()?;
    
    // Create shared resource monitor
    let monitor = Arc::new(Mutex::new(AsyncResourceMonitor::new()));
    
    // Create 12 classified tasks
    let task_configs = create_simple_tasks();
    println!("📋 Created {} classified tasks for monitoring", task_configs.len());
    
    let mut task_handles = Vec::new();
    
    // Spawn all tasks
    for (i, config) in task_configs.iter().enumerate() {
        let task_id = (3000 + i) as TaskId; // Start from 3000
        let monitor_clone = Arc::clone(&monitor);
        let config_clone = config.clone();
        
        // Create source location
        let source_location = SourceLocation {
            file_path: "examples/simple_classified_tasks.rs".to_string(),
            line_number: 100 + (i as u32 * 5),
            function_name: format!("execute_{}_task", config.category.category_name()),
            module_path: "simple_classified_tasks".to_string(),
            crate_name: "memscope_rs".to_string(),
        };

        // Start monitoring this task with source location
        {
            let mut mon = monitor_clone.lock().unwrap();
            mon.start_monitoring_with_location(task_id, config.name.clone(), config.task_type.clone(), Some(source_location));
        }
        
        let handle = tokio::spawn(async move {
            execute_simple_task(task_id, config_clone, monitor_clone).await
        });
        
        task_handles.push((task_id, config.category.clone(), handle));
        
        println!("🚀 Started Task {}: {} [{:?}]", 
                 task_id, config.name, config.category);
        
        sleep(Duration::from_millis(100)).await;
    }
    
    // Monitor for 6 seconds
    let monitoring_duration = Duration::from_secs(6);
    println!("\n⏱️  Monitoring {} tasks for {} seconds...", 
             task_handles.len(), monitoring_duration.as_secs());
    
    // Wait for all tasks
    for (task_id, category, handle) in task_handles {
        match tokio::time::timeout(monitoring_duration, handle).await {
            Ok(Ok(result)) => {
                println!("✅ Task {} [{:?}] completed: {:?}", task_id, category, result);
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
    
    // Generate simple HTML report
    println!("\n📊 Generating simple HTML report...");
    generate_simple_html_report(&monitor).await?;
    
    println!("🎉 Simple classification completed!");
    println!("📄 Check 'simple_classified_analysis.html' for results");
    
    Ok(())
}

fn create_simple_tasks() -> Vec<TaskConfig> {
    vec![
        // CPU Heavy Tasks (4 tasks)
        TaskConfig {
            name: "Prime Calculator".to_string(),
            task_type: TaskType::CpuIntensive,
            category: TaskCategory::CpuHeavy,
            description: "Calculates prime numbers".to_string(),
        },
        TaskConfig {
            name: "Matrix Processor".to_string(),
            task_type: TaskType::CpuIntensive,
            category: TaskCategory::CpuHeavy,
            description: "Matrix multiplication".to_string(),
        },
        TaskConfig {
            name: "Hash Generator".to_string(),
            task_type: TaskType::CpuIntensive,
            category: TaskCategory::CpuHeavy,
            description: "Cryptographic hashing".to_string(),
        },
        TaskConfig {
            name: "Algorithm Solver".to_string(),
            task_type: TaskType::CpuIntensive,
            category: TaskCategory::CpuHeavy,
            description: "Complex algorithms".to_string(),
        },

        // IO Heavy Tasks (3 tasks)
        TaskConfig {
            name: "File Processor".to_string(),
            task_type: TaskType::IoIntensive,
            category: TaskCategory::IoHeavy,
            description: "File operations".to_string(),
        },
        TaskConfig {
            name: "Database Writer".to_string(),
            task_type: TaskType::IoIntensive,
            category: TaskCategory::IoHeavy,
            description: "Database operations".to_string(),
        },
        TaskConfig {
            name: "Log Analyzer".to_string(),
            task_type: TaskType::IoIntensive,
            category: TaskCategory::IoHeavy,
            description: "Log processing".to_string(),
        },

        // Network Heavy Tasks (3 tasks)
        TaskConfig {
            name: "API Client".to_string(),
            task_type: TaskType::NetworkIntensive,
            category: TaskCategory::NetworkHeavy,
            description: "API requests".to_string(),
        },
        TaskConfig {
            name: "Web Scraper".to_string(),
            task_type: TaskType::NetworkIntensive,
            category: TaskCategory::NetworkHeavy,
            description: "Web data extraction".to_string(),
        },
        TaskConfig {
            name: "Data Sync".to_string(),
            task_type: TaskType::NetworkIntensive,
            category: TaskCategory::NetworkHeavy,
            description: "Network synchronization".to_string(),
        },

        // Memory Heavy Tasks (1 task)
        TaskConfig {
            name: "Cache Builder".to_string(),
            task_type: TaskType::MemoryIntensive,
            category: TaskCategory::MemoryHeavy,
            description: "Memory cache building".to_string(),
        },

        // Balanced Task (1 task)
        TaskConfig {
            name: "Full Pipeline".to_string(),
            task_type: TaskType::Mixed,
            category: TaskCategory::Balanced,
            description: "Balanced workload".to_string(),
        },
    ]
}

async fn execute_simple_task(
    task_id: TaskId,
    config: TaskConfig,
    monitor: Arc<Mutex<AsyncResourceMonitor>>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    
    // Update metrics periodically
    let update_handle = {
        let monitor_clone = Arc::clone(&monitor);
        tokio::spawn(async move {
            for _ in 0..6 {
                {
                    let mut mon = monitor_clone.lock().unwrap();
                    mon.update_metrics(task_id);
                }
                sleep(Duration::from_millis(500)).await;
            }
        })
    };
    
    // Execute different types of work based on category
    match config.category {
        TaskCategory::CpuHeavy => {
            for i in 0..2000000u32 {
                let _ = i.wrapping_mul(i) % 12345;
                if i % 200000 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        }
        TaskCategory::IoHeavy => {
            for i in 0..20 {
                let data = vec![0u8; 50000]; // 50KB
                let file_path = format!("tmp_simple_io_{}.dat", i);
                tokio::fs::write(&file_path, &data).await.ok();
                let _ = tokio::fs::read(&file_path).await;
                tokio::fs::remove_file(&file_path).await.ok();
                tokio::task::yield_now().await;
            }
        }
        TaskCategory::NetworkHeavy => {
            if let Ok(client) = reqwest::Client::builder().timeout(Duration::from_secs(2)).build() {
                for _ in 0..5 {
                    let _ = client.get("https://httpbin.org/get").send().await;
                    sleep(Duration::from_millis(300)).await;
                }
            }
        }
        TaskCategory::MemoryHeavy => {
            let mut data = Vec::new();
            for i in 0..2000 {
                let block = vec![i as u8; 5000]; // 5KB blocks
                data.push(block);
                if i % 200 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        }
        TaskCategory::Balanced => {
            // Mixed workload
            for i in 0..500000u32 {
                let _ = i.wrapping_mul(i) % 123;
            }
            
            let data = vec![0u8; 10000];
            tokio::fs::write("tmp_balanced.dat", &data).await.ok();
            tokio::fs::remove_file("tmp_balanced.dat").await.ok();
            
            let _memory_block = vec![0u8; 1000000]; // 1MB
        }
    }
    
    let _ = update_handle.await;
    Ok(format!("{} task completed successfully", config.name))
}

async fn generate_simple_html_report(monitor: &Arc<Mutex<AsyncResourceMonitor>>) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = {
        let mon = monitor.lock().unwrap();
        mon.get_all_profiles().clone()
    };
    
    let html_content = create_simple_html(&profiles)?;
    tokio::fs::write("simple_classified_analysis.html", html_content).await?;
    
    println!("📄 Simple HTML report generated: simple_classified_analysis.html");
    Ok(())
}

fn create_simple_html(profiles: &HashMap<TaskId, TaskResourceProfile>) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = String::new();
    
    // Calculate baselines and rankings
    let baselines = calculate_baselines(profiles);
    let rankings = calculate_category_rankings(profiles);
    
    html.push_str(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Enhanced Classified Task Analysis</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            margin: 0;
            padding: 20px;
            background: #0d1117;
            color: #f0f6fc;
            line-height: 1.6;
        }
        
        .container {
            max-width: 1400px;
            margin: 0 auto;
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 12px;
            overflow: hidden;
        }
        
        .header {
            background: linear-gradient(135deg, #58a6ff 0%, #a5a5ff 100%);
            padding: 2rem;
            text-align: center;
            color: white;
        }
        
        .header h1 {
            margin: 0;
            font-size: 2.5rem;
            font-weight: 700;
        }
        
        .summary {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1.5rem;
            padding: 2rem;
            background: #21262d;
        }
        
        .summary-card {
            background: #161b22;
            border: 1px solid #30363d;
            padding: 1.5rem;
            border-radius: 8px;
            text-align: center;
        }
        
        .summary-card h3 {
            margin: 0 0 0.5rem 0;
            color: #8b949e;
            font-size: 0.9rem;
            text-transform: uppercase;
        }
        
        .summary-card .value {
            font-size: 2rem;
            font-weight: 700;
            color: #58a6ff;
        }
        
        .tasks-section {
            padding: 2rem;
        }
        
        .section-title {
            margin: 0 0 2rem 0;
            font-size: 1.5rem;
            font-weight: 600;
            color: #f0f6fc;
            text-align: center;
        }
        
        .tasks-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
            gap: 1.5rem;
        }
        
        .task-card {
            background: #21262d;
            border: 1px solid #30363d;
            border-radius: 10px;
            overflow: hidden;
            transition: transform 0.2s ease;
            position: relative;
        }
        
        .task-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 16px rgba(0, 0, 0, 0.3);
        }
        
        .ranking-badge {
            position: absolute;
            top: 10px;
            right: 10px;
            background: linear-gradient(135deg, #f9c513, #ffd700);
            color: #000;
            padding: 0.25rem 0.5rem;
            border-radius: 12px;
            font-size: 0.7rem;
            font-weight: 700;
            z-index: 10;
            box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
        }
        
        .ranking-badge.rank-1 { background: linear-gradient(135deg, #ffd700, #ffed4e); }
        .ranking-badge.rank-2 { background: linear-gradient(135deg, #c0c0c0, #e8e8e8); }
        .ranking-badge.rank-3 { background: linear-gradient(135deg, #cd7f32, #daa520); }
        
        .task-header {
            padding: 1.5rem;
            background: #161b22;
            border-bottom: 1px solid #30363d;
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
        
        .task-header.cpu::before { background: #f85149; }
        .task-header.io::before { background: #58a6ff; }
        .task-header.network::before { background: #3fb950; }
        .task-header.memory::before { background: #a5a5ff; }
        .task-header.mixed::before { background: #f9c513; }
        
        .task-name {
            margin: 0 0 0.5rem 0;
            font-size: 1.125rem;
            font-weight: 600;
            color: #f0f6fc;
        }
        
        .task-badge {
            display: inline-block;
            padding: 0.25rem 0.75rem;
            border-radius: 12px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
        }
        
        .task-badge.cpu { background: #f85149; color: white; }
        .task-badge.io { background: #58a6ff; color: white; }
        .task-badge.network { background: #3fb950; color: white; }
        .task-badge.memory { background: #a5a5ff; color: white; }
        .task-badge.mixed { background: #f9c513; color: black; }
        
        .task-content {
            padding: 1.5rem;
        }
        
        .metrics-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 1rem;
            margin-bottom: 1rem;
        }
        
        .metric-item {
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 6px;
            padding: 1rem;
            text-align: center;
        }
        
        .metric-label {
            font-size: 0.75rem;
            color: #8b949e;
            margin-bottom: 0.25rem;
            text-transform: uppercase;
        }
        
        .metric-value {
            font-size: 1.25rem;
            font-weight: 700;
            color: #f0f6fc;
        }
        
        .metric-comparison {
            font-size: 0.7rem;
            color: #8b949e;
            margin-top: 0.25rem;
        }
        
        .comparison-above {
            color: #f85149;
        }
        
        .comparison-below {
            color: #3fb950;
        }
        
        .comparison-average {
            color: #f9c513;
        }
        
        .efficiency-section {
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 6px;
            padding: 1rem;
            margin-top: 1rem;
        }
        
        .efficiency-title {
            font-size: 0.875rem;
            color: #8b949e;
            margin-bottom: 0.5rem;
            text-transform: uppercase;
        }
        
        .efficiency-bar {
            width: 100%;
            height: 8px;
            background: #30363d;
            border-radius: 4px;
            overflow: hidden;
            margin-bottom: 0.5rem;
        }
        
        .efficiency-fill {
            height: 100%;
            background: linear-gradient(90deg, #3fb950, #f9c513, #f85149);
            border-radius: 4px;
        }
        
        .efficiency-score {
            font-size: 1rem;
            font-weight: 700;
            color: #58a6ff;
            text-align: right;
            position: relative;
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
        }
        
        .info-icon {
            cursor: help;
            background: #30363d;
            color: #8b949e;
            border-radius: 50%;
            width: 16px;
            height: 16px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 0.7rem;
            font-weight: bold;
        }
        
        .info-icon:hover {
            background: #58a6ff;
            color: white;
        }
        
        .tooltip {
            position: absolute;
            bottom: 100%;
            right: 0;
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 6px;
            padding: 0.75rem;
            min-width: 250px;
            font-size: 0.8rem;
            color: #f0f6fc;
            box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
            z-index: 1000;
            opacity: 0;
            visibility: hidden;
            transition: opacity 0.3s ease, visibility 0.3s ease;
        }
        
        .info-icon:hover .tooltip {
            opacity: 1;
            visibility: visible;
        }
        
        .tooltip::after {
            content: '';
            position: absolute;
            top: 100%;
            right: 10px;
            border: 5px solid transparent;
            border-top-color: #30363d;
        }
        
        .efficiency-breakdown {
            margin-top: 0.5rem;
        }
        
        .efficiency-component {
            display: flex;
            justify-content: space-between;
            margin-bottom: 0.25rem;
            font-size: 0.75rem;
        }
        
        .component-name {
            color: #8b949e;
        }
        
        .component-score {
            color: #f0f6fc;
            font-weight: 600;
        }
        
        .charts-section {
            padding: 2rem;
            background: #161b22;
            border-top: 1px solid #30363d;
        }
        
        .charts-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 2rem;
            margin-top: 1.5rem;
        }
        
        .chart-container {
            background: #21262d;
            border: 1px solid #30363d;
            border-radius: 8px;
            padding: 1.5rem;
        }
        
        .chart-title {
            margin: 0 0 1rem 0;
            font-size: 1rem;
            font-weight: 600;
            color: #f0f6fc;
            text-align: center;
        }
        
        .chart-canvas {
            width: 100%;
            height: 200px;
        }
        
        .source-info {
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 6px;
            padding: 1rem;
            margin-top: 1rem;
        }
        
        .source-title {
            font-size: 0.875rem;
            color: #8b949e;
            margin-bottom: 0.5rem;
            text-transform: uppercase;
        }
        
        .source-detail {
            display: flex;
            justify-content: space-between;
            margin-bottom: 0.25rem;
            font-size: 0.8rem;
        }
        
        .source-label {
            color: #8b949e;
        }
        
        .source-value {
            color: #f0f6fc;
            font-family: 'Courier New', monospace;
        }
        
        @media (max-width: 768px) {
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
            <h1>📊 Enhanced Classified Task Monitor</h1>
            <p>Advanced performance analysis with baselines, rankings, and trends</p>
        </div>
"#);

    // Calculate summary statistics
    let total_tasks = profiles.len();
    let mut avg_cpu = 0.0;
    let mut avg_memory_mb = 0.0;
    let mut avg_efficiency = 0.0;

    for profile in profiles.values() {
        avg_cpu += profile.cpu_metrics.usage_percent;
        avg_memory_mb += profile.memory_metrics.current_bytes as f64 / 1_048_576.0;
        avg_efficiency += profile.efficiency_score;
    }

    if total_tasks > 0 {
        avg_cpu /= total_tasks as f64;
        avg_memory_mb /= total_tasks as f64;
        avg_efficiency /= total_tasks as f64;
    }

    // Summary section
    html.push_str(&format!(r#"
        <div class="summary">
            <div class="summary-card">
                <h3>Total Tasks</h3>
                <div class="value">{}</div>
            </div>
            <div class="summary-card">
                <h3>Avg CPU Usage</h3>
                <div class="value">{:.1}%</div>
            </div>
            <div class="summary-card">
                <h3>Avg Memory</h3>
                <div class="value">{:.0}MB</div>
            </div>
            <div class="summary-card">
                <h3>Avg Efficiency</h3>
                <div class="value">{:.0}%</div>
            </div>
        </div>

        <div class="tasks-section">
            <h2 class="section-title">Task Performance Details</h2>
            <div class="tasks-grid">
"#, total_tasks, avg_cpu, avg_memory_mb, avg_efficiency * 100.0));

    // Sort tasks by efficiency score
    let mut sorted_profiles: Vec<_> = profiles.iter().collect();
    sorted_profiles.sort_by(|a, b| b.1.efficiency_score.partial_cmp(&a.1.efficiency_score).unwrap_or(std::cmp::Ordering::Equal));

    // Task cards
    for (task_id, profile) in sorted_profiles {
        let task_type_class = format!("{:?}", profile.task_type).to_lowercase();
        let ranking = rankings.get(task_id).cloned().unwrap_or(CategoryRanking {
            rank: 1,
            total_in_category: 1,
            category: TaskCategory::Balanced,
        });
        
        let cpu_value = profile.cpu_metrics.usage_percent;
        let memory_value = profile.memory_metrics.current_bytes as f64 / 1_048_576.0;
        let io_value = profile.io_metrics.bandwidth_mbps;
        let network_value = profile.network_metrics.throughput_mbps;
        
        let rank_class = match ranking.rank {
            1 => "rank-1",
            2 => "rank-2", 
            3 => "rank-3",
            _ => "",
        };
        
        html.push_str(&format!(r#"
                <div class="task-card">
                    <div class="ranking-badge {}">#{}/{}</div>
                    <div class="task-header {}">
                        <h3 class="task-name">{}</h3>
                        <span class="task-badge {}">{:?}</span>
                    </div>
                    <div class="task-content">
                        <div class="metrics-grid">
                            <div class="metric-item">
                                <div class="metric-label">CPU Usage</div>
                                <div class="metric-value">{:.1}%</div>
                                <div class="metric-comparison {}">{}</div>
                            </div>
                            <div class="metric-item">
                                <div class="metric-label">Memory</div>
                                <div class="metric-value">{:.0}MB</div>
                                <div class="metric-comparison {}">{}</div>
                            </div>
                            <div class="metric-item">
                                <div class="metric-label">IO Bandwidth</div>
                                <div class="metric-value">{:.1}MB/s</div>
                                <div class="metric-comparison {}">{}</div>
                            </div>
                            <div class="metric-item">
                                <div class="metric-label">Network</div>
                                <div class="metric-value">{:.1}Mbps</div>
                                <div class="metric-comparison {}">{}</div>
                            </div>
                        </div>"#, 
            rank_class, ranking.rank, ranking.total_in_category,
            task_type_class,
            profile.task_name,
            task_type_class,
            profile.task_type,
            cpu_value,
            get_comparison_class(cpu_value, baselines.avg_cpu),
            format_comparison(cpu_value, baselines.avg_cpu, "%"),
            memory_value,
            get_comparison_class(memory_value, baselines.avg_memory_mb),
            format_comparison(memory_value, baselines.avg_memory_mb, "MB"),
            io_value,
            get_comparison_class(io_value, baselines.avg_io_mbps),
            format_comparison(io_value, baselines.avg_io_mbps, "MB/s"),
            network_value,
            get_comparison_class(network_value, baselines.avg_network_mbps),
            format_comparison(network_value, baselines.avg_network_mbps, "Mbps")
        ));
                        
                        <div class="efficiency-section">
                            <div class="efficiency-title">Efficiency Score</div>
                            <div class="efficiency-bar">
                                <div class="efficiency-fill" style="width: {:.1}%"></div>
                            </div>
                            <div class="efficiency-score">
                                {:.1}%
                                <div class="info-icon">?
                                    <div class="tooltip">
                                        <strong>Efficiency Score Breakdown:</strong>
                                        <div class="efficiency-breakdown">
                                            <div class="efficiency-component">
                                                <span class="component-name">CPU Efficiency:</span>
                                                <span class="component-score">{:.1}%</span>
                                            </div>
                                            <div class="efficiency-component">
                                                <span class="component-name">Memory Efficiency:</span>
                                                <span class="component-score">{:.1}%</span>
                                            </div>
                                            <div class="efficiency-component">
                                                <span class="component-name">IO Performance:</span>
                                                <span class="component-score">{:.1}%</span>
                                            </div>
                                            <div class="efficiency-component">
                                                <span class="component-name">Network Performance:</span>
                                                <span class="component-score">{:.1}%</span>
                                            </div>
                                        </div>
                                        <hr style="border: 1px solid #30363d; margin: 0.5rem 0;">
                                        <small>Hover over metrics to see detailed analysis</small>
                                    </div>
                                </div>
                            </div>
                        </div>"#, 
            profile.efficiency_score * 100.0,
            profile.efficiency_score * 100.0,
            profile.efficiency_explanation.component_scores.cpu_efficiency * 100.0,
            profile.efficiency_explanation.component_scores.memory_efficiency * 100.0,
            profile.efficiency_explanation.component_scores.io_efficiency * 100.0,
            profile.efficiency_explanation.component_scores.network_efficiency * 100.0
        ));
        
        // Add source location information
        html.push_str(&format!(r#"
                        
                        <div class="source-info">
                            <div class="source-title">Source Location</div>
                            <div class="source-detail">
                                <span class="source-label">File:</span>
                                <span class="source-value">{}</span>
                            </div>
                            <div class="source-detail">
                                <span class="source-label">Line:</span>
                                <span class="source-value">{}</span>
                            </div>
                            <div class="source-detail">
                                <span class="source-label">Function:</span>
                                <span class="source-value">{}</span>
                            </div>
                        </div>
                    </div>
                </div>
"#, 
            profile.source_location.file_path,
            profile.source_location.line_number,
            profile.source_location.function_name
        ));
    }

    html.push_str(r#"
            </div>
        </div>

        <!-- Trends and Analytics Charts -->
        <div class="charts-section">
            <h2 class="section-title">📈 Performance Trends & Analytics</h2>
            <div class="charts-grid">
                <div class="chart-container">
                    <h3 class="chart-title">CPU Usage Distribution</h3>
                    <canvas id="cpuChart" class="chart-canvas"></canvas>
                </div>
                <div class="chart-container">
                    <h3 class="chart-title">Memory Usage Distribution</h3>
                    <canvas id="memoryChart" class="chart-canvas"></canvas>
                </div>
                <div class="chart-container">
                    <h3 class="chart-title">IO Performance Distribution</h3>
                    <canvas id="ioChart" class="chart-canvas"></canvas>
                </div>
                <div class="chart-container">
                    <h3 class="chart-title">Network Throughput Distribution</h3>
                    <canvas id="networkChart" class="chart-canvas"></canvas>
                </div>
            </div>
        </div>
    </div>

    <script>
        // Chart.js configuration
        Chart.defaults.color = "#f0f6fc";
        Chart.defaults.borderColor = "#30363d";
        Chart.defaults.backgroundColor = "#21262d";

        // Prepare data for charts"#);

    // Generate JavaScript data for charts
    let mut task_names = Vec::new();
    let mut cpu_data = Vec::new();
    let mut memory_data = Vec::new();
    let mut io_data = Vec::new();
    let mut network_data = Vec::new();

    for (_, profile) in profiles {
        task_names.push(format!("'{}'", profile.task_name.replace("'", "\\'")));
        cpu_data.push(format!("{:.1}", profile.cpu_metrics.usage_percent));
        memory_data.push(format!("{:.1}", profile.memory_metrics.current_bytes as f64 / 1_048_576.0));
        io_data.push(format!("{:.1}", profile.io_metrics.bandwidth_mbps));
        network_data.push(format!("{:.1}", profile.network_metrics.throughput_mbps));
    }

    html.push_str(&format!(r#"
        const taskData = {{
            labels: [{}],
            cpu: [{}],
            memory: [{}],
            io: [{}],
            network: [{}]
        }};"#, 
        task_names.join(", "),
        cpu_data.join(", "),
        memory_data.join(", "),
        io_data.join(", "),
        network_data.join(", ")
    ));

    html.push_str(r#"

        // CPU Chart
        new Chart(document.getElementById('cpuChart'), {
            type: 'bar',
            data: {
                labels: taskData.labels,
                datasets: [{
                    label: 'CPU Usage (%)',
                    data: taskData.cpu,
                    backgroundColor: 'rgba(248, 81, 73, 0.6)',
                    borderColor: 'rgba(248, 81, 73, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        max: 100
                    }
                }
            }
        });

        // Memory Chart
        new Chart(document.getElementById('memoryChart'), {
            type: 'bar',
            data: {
                labels: taskData.labels,
                datasets: [{
                    label: 'Memory Usage (MB)',
                    data: taskData.memory,
                    backgroundColor: 'rgba(165, 165, 255, 0.6)',
                    borderColor: 'rgba(165, 165, 255, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });

        // IO Chart
        new Chart(document.getElementById('ioChart'), {
            type: 'bar',
            data: {
                labels: taskData.labels,
                datasets: [{
                    label: 'IO Bandwidth (MB/s)',
                    data: taskData.io,
                    backgroundColor: 'rgba(88, 166, 255, 0.6)',
                    borderColor: 'rgba(88, 166, 255, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });

        // Network Chart
        new Chart(document.getElementById('networkChart'), {
            type: 'bar',
            data: {
                labels: taskData.labels,
                datasets: [{
                    label: 'Network Throughput (Mbps)',
                    data: taskData.network,
                    backgroundColor: 'rgba(63, 185, 80, 0.6)',
                    borderColor: 'rgba(63, 185, 80, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });
    </script>
</body>
</html>
"#);

    Ok(html)
}

#[derive(Debug, Clone)]
struct Baselines {
    avg_cpu: f64,
    avg_memory_mb: f64,
    avg_io_mbps: f64,
    avg_network_mbps: f64,
    avg_efficiency: f64,
}

#[derive(Debug, Clone)]
struct CategoryRanking {
    rank: usize,
    total_in_category: usize,
    category: TaskCategory,
}

fn calculate_baselines(profiles: &HashMap<TaskId, TaskResourceProfile>) -> Baselines {
    let total = profiles.len() as f64;
    if total == 0.0 {
        return Baselines {
            avg_cpu: 0.0,
            avg_memory_mb: 0.0,
            avg_io_mbps: 0.0,
            avg_network_mbps: 0.0,
            avg_efficiency: 0.0,
        };
    }

    let mut total_cpu = 0.0;
    let mut total_memory = 0.0;
    let mut total_io = 0.0;
    let mut total_network = 0.0;
    let mut total_efficiency = 0.0;

    for profile in profiles.values() {
        total_cpu += profile.cpu_metrics.usage_percent;
        total_memory += profile.memory_metrics.current_bytes as f64 / 1_048_576.0;
        total_io += profile.io_metrics.bandwidth_mbps;
        total_network += profile.network_metrics.throughput_mbps;
        total_efficiency += profile.efficiency_score;
    }

    Baselines {
        avg_cpu: total_cpu / total,
        avg_memory_mb: total_memory / total,
        avg_io_mbps: total_io / total,
        avg_network_mbps: total_network / total,
        avg_efficiency: total_efficiency / total,
    }
}

fn calculate_category_rankings(profiles: &HashMap<TaskId, TaskResourceProfile>) -> HashMap<TaskId, CategoryRanking> {
    let mut rankings = HashMap::new();
    
    // Group tasks by category
    let mut category_groups: HashMap<String, Vec<(TaskId, &TaskResourceProfile)>> = HashMap::new();
    
    for (task_id, profile) in profiles {
        let category_key = format!("{:?}", profile.task_type);
        category_groups.entry(category_key).or_default().push(*task_id, profile);
    }
    
    // Calculate rankings within each category
    for (category_key, mut tasks) in category_groups {
        // Sort by efficiency score (descending)
        tasks.sort_by(|a, b| b.1.efficiency_score.partial_cmp(&a.1.efficiency_score).unwrap_or(std::cmp::Ordering::Equal));
        
        let total_in_category = tasks.len();
        
        for (rank, (task_id, profile)) in tasks.iter().enumerate() {
            let category = match profile.task_type {
                TaskType::CpuIntensive => TaskCategory::CpuHeavy,
                TaskType::IoIntensive => TaskCategory::IoHeavy,
                TaskType::NetworkIntensive => TaskCategory::NetworkHeavy,
                TaskType::MemoryIntensive => TaskCategory::MemoryHeavy,
                TaskType::Mixed => TaskCategory::Balanced,
                _ => TaskCategory::Balanced,
            };
            
            rankings.insert(*task_id, CategoryRanking {
                rank: rank + 1,
                total_in_category,
                category,
            });
        }
    }
    
    rankings
}

fn format_comparison(value: f64, baseline: f64, unit: &str) -> String {
    let diff_percent = ((value - baseline) / baseline * 100.0).abs();
    
    if (value - baseline).abs() < baseline * 0.05 {
        format!("(≈ avg {})", unit)
    } else if value > baseline {
        format!("(+{:.1}% vs avg)", diff_percent)
    } else {
        format!("(-{:.1}% vs avg)", diff_percent)
    }
}

fn get_comparison_class(value: f64, baseline: f64) -> &'static str {
    if (value - baseline).abs() < baseline * 0.05 {
        "comparison-average"
    } else if value > baseline {
        "comparison-above"
    } else {
        "comparison-below"
    }
}

fn generate_efficiency_explanation(profile: &TaskResourceProfile) -> String {
    let cpu_score = (100.0 - profile.cpu_metrics.usage_percent.min(100.0)) / 100.0 * 100.0;
    let memory_score = 100.0 - (profile.memory_metrics.current_bytes as f64 / 1_048_576.0 / 100.0).min(100.0);
    let io_score = profile.io_metrics.bandwidth_mbps.min(100.0);
    let network_score = profile.network_metrics.throughput_mbps.min(100.0);
    
    format!(
        "CPU Efficiency: {:.1}% | Memory Efficiency: {:.1}% | IO Performance: {:.1}% | Network Performance: {:.1}% | Overall: {:.1}%",
        cpu_score, memory_score, io_score, network_score, profile.efficiency_score * 100.0
    )
}