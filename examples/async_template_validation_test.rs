use memscope_rs::async_memory::{
    initialize, spawn_tracked, VisualizationGenerator, AsyncResourceMonitor, TaskType,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

/// Async template validation test using exact same API as comprehensive_async_showcase.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Async Template Validation Test...");
    
    let output_dir = "memoryanalysis";
    std::fs::create_dir_all(output_dir)?;
    
    // Initialize async memory tracking
    initialize()?;
    
    // Create async resource monitor like in comprehensive_async_showcase.rs
    let monitor = Arc::new(Mutex::new(AsyncResourceMonitor::new()));
    
    // Run various async workloads to test template
    run_template_validation_workloads(monitor.clone()).await?;
    
    // Generate HTML using the correct API
    let profiles = {
        let mon = monitor.lock().unwrap();
        mon.get_all_profiles().clone()
    };
    
    let viz_generator = VisualizationGenerator::new();
    let html_content = viz_generator.generate_html_report(&profiles)?;
    
    let output_path = format!("{}/async_template_validation_dashboard.html", output_dir);
    tokio::fs::write(&output_path, html_content).await?;
    
    println!("✅ Async template validation test completed!");
    println!("📊 Profiles collected: {} tasks", profiles.len());
    println!("📄 HTML Report: {}", output_path);
    
    Ok(())
}

async fn run_template_validation_workloads(monitor: Arc<Mutex<AsyncResourceMonitor>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Running template validation workloads...");
    
    // Workload 1: CPU-intensive tasks with varying memory patterns
    cpu_intensive_workload(monitor.clone()).await?;
    
    // Workload 2: I/O simulation with buffer management
    io_simulation_workload(monitor.clone()).await?;
    
    // Workload 3: Producer-consumer pattern
    producer_consumer_workload(monitor.clone()).await?;
    
    // Workload 4: Burst allocation pattern
    burst_allocation_workload(monitor.clone()).await?;
    
    // Update all task metrics to get realistic data
    println!("📊 Updating task metrics...");
    {
        let mut mon = monitor.lock().unwrap();
        let task_ids: Vec<_> = mon.get_all_profiles().keys().cloned().collect();
        for task_id in task_ids {
            mon.update_metrics(task_id);
            mon.finish_monitoring(task_id);
        }
    }
    
    println!("✅ All workloads completed with metrics updated");
    Ok(())
}

/// CPU-intensive workload with different allocation sizes
async fn cpu_intensive_workload(_monitor: Arc<Mutex<AsyncResourceMonitor>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 CPU-intensive workload...");
    
    let tasks = (1..=8).map(|task_id| {
        let monitor = _monitor.clone();
        spawn_tracked(async move {
            // Register this task with the monitor
            {
                let mut mon = monitor.lock().unwrap();
                mon.start_monitoring(task_id as u128, format!("cpu_task_{}", task_id), TaskType::CpuIntensive);
            }
            
            for iteration in 0..25 {
                // Simulate different CPU work with memory allocation
                let work_size = match task_id % 4 {
                    0 => 1024 + (iteration * 64),      // Small, growing
                    1 => 4096 + (iteration * 128),     // Medium, growing  
                    2 => 8192 - (iteration * 32),      // Large, shrinking
                    _ => 2048 + (iteration % 10) * 256, // Variable
                };
                
                let _work_buffer = vec![task_id as u8; work_size];
                
                // Simulate CPU computation
                let mut sum = 0u64;
                for i in 0..1000 {
                    sum += (i * task_id as u64) % 997;
                }
                
                // Memory for intermediate results
                let _result_buffer = vec![sum as u8; 512];
                
                sleep(Duration::from_millis(10 + (task_id % 5) as u64)).await;
            }
            
            task_id
        })
    }).collect::<Vec<_>>();
    
    let results = futures::future::join_all(tasks).await;
    println!("✅ CPU workload: {} tasks completed", results.len());
    
    Ok(())
}

/// I/O simulation with realistic buffer patterns
async fn io_simulation_workload(_monitor: Arc<Mutex<AsyncResourceMonitor>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("💾 I/O simulation workload...");
    
    let io_tasks = (1..=6).map(|io_id| {
        let monitor = _monitor.clone();
        spawn_tracked(async move {
            // Register this task with the monitor
            {
                let mut mon = monitor.lock().unwrap();
                mon.start_monitoring(io_id as u128 + 100, format!("io_task_{}", io_id), TaskType::IoIntensive);
            }
            
            for operation in 0..15 {
                // Simulate different I/O operations
                match operation % 3 {
                    0 => {
                        // File read simulation
                        let read_size = 2048 + (operation * 128);
                        let _read_buffer = vec![0u8; read_size];
                        sleep(Duration::from_millis(8)).await;
                    }
                    1 => {
                        // File write simulation  
                        let write_size = 1536 + (operation * 96);
                        let _write_buffer = vec![io_id as u8; write_size];
                        sleep(Duration::from_millis(12)).await;
                    }
                    _ => {
                        // Network operation simulation
                        let packet_size = 512 + (operation * 64);
                        let header_size = 128;
                        let _packet = vec![0u8; packet_size];
                        let _header = vec![io_id as u8; header_size];
                        sleep(Duration::from_millis(6)).await;
                    }
                }
            }
            
            io_id
        })
    }).collect::<Vec<_>>();
    
    let results = futures::future::join_all(io_tasks).await;
    println!("✅ I/O workload: {} tasks completed", results.len());
    
    Ok(())
}

/// Producer-consumer pattern with channels
async fn producer_consumer_workload(_monitor: Arc<Mutex<AsyncResourceMonitor>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Producer-consumer workload...");
    
    let (tx, rx) = tokio::sync::mpsc::channel::<DataMessage>(100);
    let rx = std::sync::Arc::new(tokio::sync::Mutex::new(rx));
    
    // Start 3 producers
    let producers = (1..=3).map(|producer_id| {
        let tx = tx.clone();
        let monitor = _monitor.clone();
        spawn_tracked(async move {
            // Register this task with the monitor
            {
                let mut mon = monitor.lock().unwrap();
                mon.start_monitoring(producer_id as u128 + 200, format!("producer_{}", producer_id), TaskType::NetworkIntensive);
            }
            
            for msg_id in 0..20 {
                // Create message with varying payload
                let payload_size = 256 + (msg_id * 32) + (producer_id as usize * 64);
                let payload = vec![producer_id as u8; payload_size];
                
                // Create metadata
                let metadata_size = 64 + (msg_id * 4);
                let metadata = vec![msg_id as u8; metadata_size];
                
                let message = DataMessage {
                    id: producer_id * 1000 + msg_id as u32,
                    payload,
                    metadata,
                };
                
                let _ = tx.send(message).await;
                sleep(Duration::from_millis(15)).await;
            }
            
            producer_id
        })
    }).collect::<Vec<_>>();
    
    // Start 2 consumers
    let consumers = (1..=2).map(|consumer_id| {
        let rx = rx.clone();
        let monitor = _monitor.clone();
        spawn_tracked(async move {
            // Register this task with the monitor
            {
                let mut mon = monitor.lock().unwrap();
                mon.start_monitoring(consumer_id as u128 + 300, format!("consumer_{}", consumer_id), TaskType::NetworkIntensive);
            }
            
            let mut processed = 0;
            while processed < 30 {
                let message = {
                    let mut rx_guard = rx.lock().await;
                    rx_guard.recv().await
                };
                
                if let Some(message) = message {
                    // Process message (allocate processing buffer)
                    let processing_size = message.payload.len() + 256;
                    let _processing_buffer = vec![consumer_id as u8; processing_size];
                    
                    // Simulate processing time
                    sleep(Duration::from_millis(5)).await;
                    
                    processed += 1;
                } else {
                    break;
                }
            }
            
            consumer_id
        })
    }).collect::<Vec<_>>();
    
    drop(tx); // Close channel
    
    let producer_results = futures::future::join_all(producers).await;
    let consumer_results = futures::future::join_all(consumers).await;
    
    println!("✅ Producer-consumer: {} producers, {} consumers", 
             producer_results.len(), consumer_results.len());
    
    Ok(())
}

/// Burst allocation pattern to test peak memory tracking
async fn burst_allocation_workload(_monitor: Arc<Mutex<AsyncResourceMonitor>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("💥 Burst allocation workload...");
    
    let burst_tasks = (1..=4).map(|burst_id| {
        let monitor = _monitor.clone();
        spawn_tracked(async move {
            // Register this task with the monitor
            {
                let mut mon = monitor.lock().unwrap();
                mon.start_monitoring(burst_id as u128 + 400, format!("burst_task_{}", burst_id), TaskType::MemoryIntensive);
            }
            
            // Phase 1: Small allocations
            let mut small_buffers = Vec::new();
            for i in 0..50 {
                let buffer = vec![burst_id as u8; 128 + (i * 8)];
                small_buffers.push(buffer);
                
                if i % 10 == 0 {
                    sleep(Duration::from_millis(2)).await;
                }
            }
            
            sleep(Duration::from_millis(20)).await;
            
            // Phase 2: Burst of large allocations
            let mut large_buffers = Vec::new();
            for i in 0..10 {
                let large_size = 8192 + (i * 1024);
                let buffer = vec![(burst_id + i) as u8; large_size];
                large_buffers.push(buffer);
                sleep(Duration::from_millis(5)).await;
            }
            
            sleep(Duration::from_millis(30)).await;
            
            // Phase 3: Release half and allocate different sizes
            large_buffers.truncate(5);
            
            for i in 0..15 {
                let variable_size = if i % 3 == 0 { 512 } else if i % 3 == 1 { 2048 } else { 4096 };
                let _temp_buffer = vec![burst_id as u8; variable_size];
                sleep(Duration::from_millis(3)).await;
            }
            
            burst_id
        })
    }).collect::<Vec<_>>();
    
    let results = futures::future::join_all(burst_tasks).await;
    println!("✅ Burst workload: {} tasks completed", results.len());
    
    Ok(())
}

#[derive(Clone, Debug)]
struct DataMessage {
    id: u32,
    payload: Vec<u8>,
    metadata: Vec<u8>,
}
