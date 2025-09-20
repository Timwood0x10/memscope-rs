//! Enhanced Simple Classified Async Task Monitor
//!
//! This version includes all the requested features:
//! - Baseline comparisons
//! - Category rankings  
//! - Efficiency score explanations
//! - Performance trend charts

use memscope_rs::async_memory::{
    self, TaskType, TaskId, AsyncResourceMonitor, TaskResourceProfile,
    resource_monitor::SourceLocation, VisualizationGenerator, VisualizationConfig, Theme
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Enhanced Simple Classified Task Monitor");
    println!("==========================================");
    
    // Initialize the async memory tracking system
    async_memory::initialize()?;
    
    // Create shared resource monitor
    let monitor = Arc::new(Mutex::new(AsyncResourceMonitor::new()));
    
    // Create 10 classified tasks
    let task_configs = create_enhanced_tasks();
    println!("📋 Created {} classified tasks for monitoring", task_configs.len());
    
    let mut task_handles = Vec::new();
    
    // Spawn all tasks
    for (i, config) in task_configs.iter().enumerate() {
        let task_id = (4000 + i) as TaskId;
        let monitor_clone = Arc::clone(&monitor);
        let config_clone = config.clone();
        
        // Create source location
        let source_location = SourceLocation {
            file_path: "examples/enhanced_simple_tasks.rs".to_string(),
            line_number: 150 + (i as u32 * 10),
            function_name: format!("execute_{}_task", config.category.category_name()),
            module_path: "enhanced_simple_tasks".to_string(),
            crate_name: "memscope_rs".to_string(),
        };

        // Start monitoring
        {
            let mut mon = monitor_clone.lock().unwrap();
            mon.start_monitoring_with_location(task_id, config.name.clone(), config.task_type.clone(), Some(source_location));
        }
        
        let handle = tokio::spawn(async move {
            execute_enhanced_task(task_id, config_clone, monitor_clone).await
        });
        
        task_handles.push((task_id, config.category.clone(), handle));
        
        println!("🚀 Started Task {}: {} [{:?}]", 
                 task_id, config.name, config.category);
        
        sleep(Duration::from_millis(100)).await;
    }
    
    // Monitor for 5 seconds
    let monitoring_duration = Duration::from_secs(5);
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
    
    // Generate enhanced HTML report
    println!("\n📊 Generating enhanced HTML report...");
    generate_enhanced_html_report(&monitor).await?;
    
    println!("🎉 Enhanced analysis completed!");
    println!("📄 Check 'enhanced_simple_analysis.html' for results");
    
    Ok(())
}

fn create_enhanced_tasks() -> Vec<TaskConfig> {
    vec![
        // CPU Heavy Tasks (4 tasks)
        TaskConfig {
            name: "Prime Calculator Pro".to_string(),
            task_type: TaskType::CpuIntensive,
            category: TaskCategory::CpuHeavy,
        },
        TaskConfig {
            name: "Matrix Processor".to_string(),
            task_type: TaskType::CpuIntensive,
            category: TaskCategory::CpuHeavy,
        },
        TaskConfig {
            name: "Hash Generator".to_string(),
            task_type: TaskType::CpuIntensive,
            category: TaskCategory::CpuHeavy,
        },
        TaskConfig {
            name: "Algorithm Solver".to_string(),
            task_type: TaskType::CpuIntensive,
            category: TaskCategory::CpuHeavy,
        },

        // IO Heavy Tasks (3 tasks)
        TaskConfig {
            name: "File Stream Handler".to_string(),
            task_type: TaskType::IoIntensive,
            category: TaskCategory::IoHeavy,
        },
        TaskConfig {
            name: "Database Engine".to_string(),
            task_type: TaskType::IoIntensive,
            category: TaskCategory::IoHeavy,
        },
        TaskConfig {
            name: "Log Processor".to_string(),
            task_type: TaskType::IoIntensive,
            category: TaskCategory::IoHeavy,
        },

        // Network Heavy Tasks (2 tasks)
        TaskConfig {
            name: "API Gateway".to_string(),
            task_type: TaskType::NetworkIntensive,
            category: TaskCategory::NetworkHeavy,
        },
        TaskConfig {
            name: "Data Sync Engine".to_string(),
            task_type: TaskType::NetworkIntensive,
            category: TaskCategory::NetworkHeavy,
        },

        // Balanced Task (1 task)
        TaskConfig {
            name: "Full Stack Pipeline".to_string(),
            task_type: TaskType::Mixed,
            category: TaskCategory::Balanced,
        },
    ]
}

async fn execute_enhanced_task(
    task_id: TaskId,
    config: TaskConfig,
    monitor: Arc<Mutex<AsyncResourceMonitor>>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    
    // Update metrics periodically
    let update_handle = {
        let monitor_clone = Arc::clone(&monitor);
        tokio::spawn(async move {
            for _ in 0..5 {
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
            for i in 0..1500000u32 {
                let _ = i.wrapping_mul(i) % 12345;
                if i % 150000 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        }
        TaskCategory::IoHeavy => {
            for i in 0..15 {
                let data = vec![0u8; 100000]; // 100KB
                let file_path = format!("tmp_enhanced_io_{}.dat", i);
                tokio::fs::write(&file_path, &data).await.ok();
                let _ = tokio::fs::read(&file_path).await;
                tokio::fs::remove_file(&file_path).await.ok();
                tokio::task::yield_now().await;
            }
        }
        TaskCategory::NetworkHeavy => {
            if let Ok(client) = reqwest::Client::builder().timeout(Duration::from_secs(2)).build() {
                for _ in 0..3 {
                    let _ = client.get("https://httpbin.org/get").send().await;
                    sleep(Duration::from_millis(400)).await;
                }
            }
        }
        TaskCategory::MemoryHeavy => {
            let mut data = Vec::new();
            for i in 0..1500 {
                let block = vec![i as u8; 10000]; // 10KB blocks
                data.push(block);
                if i % 150 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        }
        TaskCategory::Balanced => {
            // Mixed workload
            for i in 0..300000u32 {
                let _ = i.wrapping_mul(i) % 123;
            }
            
            let data = vec![0u8; 50000];
            tokio::fs::write("tmp_enhanced_balanced.dat", &data).await.ok();
            tokio::fs::remove_file("tmp_enhanced_balanced.dat").await.ok();
            
            let _memory_block = vec![0u8; 5000000]; // 5MB
        }
    }
    
    let _ = update_handle.await;
    Ok(format!("{} task completed successfully", config.name))
}

async fn generate_enhanced_html_report(monitor: &Arc<Mutex<AsyncResourceMonitor>>) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = {
        let mon = monitor.lock().unwrap();
        mon.get_all_profiles().clone()
    };
    
    // Create visualization configuration
    let config = VisualizationConfig {
        title: "Enhanced Async Task Performance Analysis".to_string(),
        theme: Theme::Dark,
        include_charts: true,
        include_baselines: true,
        include_rankings: true,
        include_efficiency_breakdown: true,
    };
    
    // Generate HTML using the visualization module
    let visualizer = VisualizationGenerator::with_config(config);
    let html_content = visualizer.generate_html_report(&profiles)?;
    
    tokio::fs::write("enhanced_simple_analysis.html", html_content).await?;
    
    println!("📄 Enhanced HTML report generated using visualization module: enhanced_simple_analysis.html");
    Ok(())
}
