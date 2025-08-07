//! Async Memory Analysis Performance Test - Binary Export Format
//! 
//! This example demonstrates memory analysis performance in an async environment,
//! using binary format for data export and measuring binary to JSON conversion performance

use memscope_rs::{track_var, get_global_tracker};
use memscope_rs::export::binary::{export_to_binary, parse_binary_to_json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;


/// Async data processor
#[allow(dead_code)]
struct AsyncDataProcessor {
    id: usize,
    processed_count: Arc<Mutex<usize>>,
}

impl AsyncDataProcessor {
    fn new(id: usize) -> Self {
        Self {
            id,
            processed_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Async data processing
    async fn process_data(&self, data: Vec<u8>) -> Vec<u8> {
        // Simulate async I/O operation
        sleep(Duration::from_millis(10)).await;
        
        // Create processed data
        let mut processed = Vec::with_capacity(data.len() * 2);
        let _tracked_processed = track_var!(processed.clone());
        
        // Simulate data processing
        for &byte in &data {
            processed.push(byte);
            processed.push(byte.wrapping_add(1));
        }
        
        // Update counter
        {
            let mut count = self.processed_count.lock().unwrap();
            *count += 1;
        }
        
        processed
    }

    /// Get processing count
    fn get_processed_count(&self) -> usize {
        *self.processed_count.lock().unwrap()
    }
}

/// Async network simulator
struct AsyncNetworkSimulator {
    latency_ms: u64,
}

impl AsyncNetworkSimulator {
    fn new(latency_ms: u64) -> Self {
        Self { latency_ms }
    }

    /// Simulate async network request
    async fn fetch_data(&self, size: usize) -> Vec<u8> {
        // Simulate network latency
        sleep(Duration::from_millis(self.latency_ms)).await;
        
        // Create simulated data
        let _data = track_var!(vec![0u8; size]);
        
        // Fill with some simulated data
        let mut result = Vec::with_capacity(size);
        let _tracked_result = track_var!(result.clone());
        for i in 0..size {
            result.push((i % 256) as u8);
        }
        
        result
    }

    /// Simulate async data upload
    async fn upload_data(&self, data: &[u8]) -> bool {
        // Simulate upload latency
        sleep(Duration::from_millis(self.latency_ms / 2)).await;
        
        // Create upload confirmation data
        let _confirmation = track_var!(format!("uploaded_{}_bytes", data.len()));
        
        true
    }
}

/// Async task manager
struct AsyncTaskManager {
    tasks_completed: Arc<Mutex<usize>>,
    total_data_processed: Arc<Mutex<usize>>,
}

impl AsyncTaskManager {
    fn new() -> Self {
        Self {
            tasks_completed: Arc::new(Mutex::new(0)),
            total_data_processed: Arc::new(Mutex::new(0)),
        }
    }

    /// Execute async task
    async fn execute_task(&self, task_id: usize, processor: &AsyncDataProcessor, network: &AsyncNetworkSimulator) {
        println!("🔄 Task {} starting execution", task_id);
        
        // Create task local data
        let _task_buffer = track_var!(vec![task_id as u8; 512]);
        let _task_metadata = track_var!(HashMap::<String, String>::new());
        
        // Step 1: Fetch data
        let raw_data = network.fetch_data(256 + task_id % 100).await;
        let _tracked_raw = track_var!(raw_data.clone());
        
        // Step 2: Process data
        let processed_data = processor.process_data(raw_data).await;
        let _tracked_processed = track_var!(processed_data.clone());
        
        // Step 3: Upload results
        let _upload_success = network.upload_data(&processed_data).await;
        
        // Step 4: Create result summary
        let _summary = track_var!(format!("task_{}_completed_size_{}", task_id, processed_data.len()));
        
        // Update statistics
        {
            let mut completed = self.tasks_completed.lock().unwrap();
            *completed += 1;
            
            let mut total_data = self.total_data_processed.lock().unwrap();
            *total_data += processed_data.len();
        }
        
        println!("✅ Task {} completed", task_id);
    }

    /// Get statistics
    fn get_stats(&self) -> (usize, usize) {
        let completed = *self.tasks_completed.lock().unwrap();
        let total_data = *self.total_data_processed.lock().unwrap();
        (completed, total_data)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Async Memory Analysis Performance Test Started");
    println!("📊 Using Binary Export Format for Performance Analysis\n");

    // Get global memory tracker
    let tracker = get_global_tracker();

    // Test configuration
    let num_tasks = 20;
    let num_processors = 3;
    let network_latency = 5; // ms
    
    println!("⚙️  Test Configuration:");
    println!("   • Number of async tasks: {}", num_tasks);
    println!("   • Number of data processors: {}", num_processors);
    println!("   • Simulated network latency: {}ms", network_latency);
    println!();

    // Create async components
    let task_manager = AsyncTaskManager::new();
    
    let mut processors = Vec::new();
    for i in 0..num_processors {
        processors.push(AsyncDataProcessor::new(i));
    }
    
    let network = AsyncNetworkSimulator::new(network_latency);

    // Record start time
    let start_time = Instant::now();
    println!("⏱️  Starting async execution...");

    // Create concurrent tasks
    let mut task_handles = Vec::new();
    
    for task_id in 0..num_tasks {
        let processor_index = task_id % num_processors;
        let processor = &processors[processor_index];
        
        let task_handle = task_manager.execute_task(task_id, processor, &network);
        task_handles.push(task_handle);
    }

    // Wait for all tasks to complete
    for task_handle in task_handles {
        task_handle.await;
    }

    let execution_time = start_time.elapsed();
    println!("\n✨ Async execution completed!");
    println!("   • Total execution time: {:?}", execution_time);

    // Get task statistics
    let (completed_tasks, total_data_processed) = task_manager.get_stats();
    println!("   • Completed tasks: {}", completed_tasks);
    println!("   • Total data processed: {} bytes", total_data_processed);

    // Get processor statistics
    println!("\n📊 Processor Statistics:");
    for (i, processor) in processors.iter().enumerate() {
        println!("   • Processor {}: {} processing operations", i, processor.get_processed_count());
    }

    // Get memory analysis data
    println!("\n📈 Collecting memory analysis data...");
    let allocations = tracker.get_active_allocations()?;
    println!("   • Total allocations: {}", allocations.len());
    
    // Calculate statistics
    let total_memory: usize = allocations.iter().map(|a| a.size).sum();
    let avg_allocation_size = if !allocations.is_empty() {
        total_memory / allocations.len()
    } else {
        0
    };
    
    println!("   • Total memory usage: {} bytes", total_memory);
    println!("   • Average allocation size: {} bytes", avg_allocation_size);

    // Statistics by thread (may have multiple tokio threads in async environment)
    let mut thread_stats = HashMap::new();
    for alloc in &allocations {
        let entry = thread_stats.entry(&alloc.thread_id).or_insert((0, 0));
        entry.0 += 1; // count
        entry.1 += alloc.size; // total size
    }
    
    println!("\n📊 Statistics by thread:");
    for (thread_id, (count, size)) in thread_stats {
        println!("   • {}: {} allocations, {} bytes", thread_id, count, size);
    }

    // Export to Binary format and measure performance
    println!("\n💾 Exporting to Binary format...");
    let project_name = "async_performance_test";
    let binary_path = format!("./MemoryAnalysis/{}/{}.memscope", project_name, project_name);
    let json_path = format!("./MemoryAnalysis/{}/{}.json", project_name, project_name);
    
    // Create output directory
    std::fs::create_dir_all(format!("./MemoryAnalysis/{}", project_name))?;
    
    let export_start = Instant::now();
    export_to_binary(&allocations, &binary_path)?;
    let export_time = export_start.elapsed();
    
    println!("   • Binary export time: {:?}", export_time);
    
    // Get file size
    let binary_size = std::fs::metadata(&binary_path)?.len();
    println!("   • Binary file size: {} bytes", binary_size);

    // Convert Binary to JSON and measure performance
    println!("\n🔄 Converting Binary to JSON...");
    let conversion_start = Instant::now();
    match parse_binary_to_json(&binary_path, &json_path) {
        Ok(_) => println!("   • Conversion successful"),
        Err(e) => {
            println!("   • Conversion encountered issue: {}", e);
            println!("   • Using backup method to export JSON...");
            // Backup method: export JSON directly from tracker
            tracker.export_to_json(project_name)?;
            let backup_json = format!("./MemoryAnalysis/{}/{}_memory_analysis.json", project_name, project_name);
            std::fs::copy(&backup_json, &json_path).ok();
        }
    }
    let conversion_time = conversion_start.elapsed();
    
    println!("   • Conversion time: {:?}", conversion_time);
    
    // Get JSON file size for comparison
    let json_size = std::fs::metadata(&json_path)?.len();
    println!("   • JSON file size: {} bytes", json_size);
    
    // Calculate compression ratio
    let compression_ratio = if json_size > 0 {
        (json_size as f64 - binary_size as f64) / json_size as f64 * 100.0
    } else {
        0.0
    };
    println!("   • Compression ratio: {:.1}% (Binary smaller than JSON)", compression_ratio);

    // Performance summary
    println!("\n🎯 Performance Summary:");
    println!("   • Async execution: {:?}", execution_time);
    println!("   • Binary export: {:?}", export_time);
    println!("   • Binary→JSON conversion: {:?}", conversion_time);
    println!("   • Total processing time: {:?}", execution_time + export_time + conversion_time);
    
    // Calculate throughput
    let tasks_per_second = num_tasks as f64 / execution_time.as_secs_f64();
    let allocations_per_second = allocations.len() as f64 / (export_time.as_secs_f64() + conversion_time.as_secs_f64());
    
    println!("   • Task throughput: {:.1} tasks/sec", tasks_per_second);
    println!("   • Processing throughput: {:.0} allocations/sec", allocations_per_second);

    // Async-specific performance metrics
    let avg_task_time = execution_time.as_millis() as f64 / num_tasks as f64;
    println!("   • Average task time: {:.1}ms", avg_task_time);

    println!("\n📁 Generated files:");
    println!("   • Binary format: {}", binary_path);
    println!("   • JSON format: {}", json_path);
    println!("   • Check ./MemoryAnalysis/{}/", project_name);

    println!("\n🎉 Async performance test completed!");
    
    Ok(())
}