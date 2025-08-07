//! Multithreading Memory Analysis Performance Test - Binary Export Format
//! 
//! This example demonstrates memory analysis performance in a multithreaded environment,
//! using binary format for data export and measuring binary to JSON conversion performance

use memscope_rs::{track_var, get_global_tracker};
use memscope_rs::export::binary::{export_to_binary, parse_binary_to_json};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Shared data structure for multithreading tests
#[derive(Clone)]
struct SharedData {
    counter: Arc<Mutex<u64>>,
    data_store: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl SharedData {
    fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            data_store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn increment_and_store(&self, thread_id: usize, data: Vec<u8>) {
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;
        
        let mut store = self.data_store.lock().unwrap();
        store.insert(format!("thread_{}_data_{}", thread_id, *counter), data);
    }
}

/// Simulate CPU-intensive workload
fn cpu_intensive_work(iterations: usize) -> Vec<u64> {
    let mut results = Vec::with_capacity(iterations);
    for i in 0..iterations {
        // 模拟一些计算
        let value = (i as u64).wrapping_mul(17).wrapping_add(23) % 1000;
        results.push(value);
    }
    results
}

/// Multithreading worker function
fn worker_thread(thread_id: usize, shared_data: SharedData, work_size: usize) {
    println!("🧵 Thread {} starting work", thread_id);
    
    // Create some local data structures
    let _local_buffer = track_var!(vec![0u8; 1024 * thread_id + 512]);
    let _local_map = track_var!(HashMap::<String, i32>::new());
    
    for iteration in 0..work_size {
        // CPU-intensive work
        let _computation_result = track_var!(cpu_intensive_work(100));
        
        // Create some data and store it in shared structure
        let data = vec![thread_id as u8; 256 + iteration % 100];
        let _tracked_data = track_var!(data.clone());
        shared_data.increment_and_store(thread_id, data);
        
        // Simulate some string operations
        let _temp_string = track_var!(format!("thread_{}_iteration_{}", thread_id, iteration));
        
        // Brief sleep to simulate I/O
        if iteration % 10 == 0 {
            thread::sleep(Duration::from_millis(1));
        }
    }
    
    println!("✅ Thread {} completed work", thread_id);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Multithreading Memory Analysis Performance Test Started");
    println!("📊 Using Binary Export Format for Performance Analysis\n");

    // Get global memory tracker
    let tracker = get_global_tracker();

    // Test configuration
    let num_threads = 4;
    let work_per_thread = 50;
    
    println!("⚙️  Test Configuration:");
    println!("   • Number of threads: {}", num_threads);
    println!("   • Work per thread: {}", work_per_thread);
    println!();

    // Create shared data
    let shared_data = SharedData::new();

    // Record start time
    let start_time = Instant::now();
    println!("⏱️  Starting multithreaded execution...");

    // Launch multiple worker threads
    let mut handles = Vec::new();
    
    for thread_id in 0..num_threads {
        let shared_data_clone = shared_data.clone();
        let handle = thread::spawn(move || {
            worker_thread(thread_id, shared_data_clone, work_per_thread);
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for (i, handle) in handles.into_iter().enumerate() {
        handle.join().unwrap();
        println!("🔄 Thread {} joined main thread", i);
    }

    let execution_time = start_time.elapsed();
    println!("\n✨ Multithreaded execution completed!");
    println!("   • Total execution time: {:?}", execution_time);

    // Get memory analysis data
    println!("\n📈 Collecting memory analysis data...");
    let allocations = tracker.get_active_allocations()?;
    println!("   • Total allocations: {}", allocations.len());
    
    // Calculate some statistics
    let total_memory: usize = allocations.iter().map(|a| a.size).sum();
    let avg_allocation_size = if !allocations.is_empty() {
        total_memory / allocations.len()
    } else {
        0
    };
    
    println!("   • Total memory usage: {} bytes", total_memory);
    println!("   • Average allocation size: {} bytes", avg_allocation_size);

    // Statistics by thread
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
    let project_name = "multithreading_performance_test";
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
    println!("   • Multithreaded execution: {:?}", execution_time);
    println!("   • Binary export: {:?}", export_time);
    println!("   • Binary→JSON conversion: {:?}", conversion_time);
    println!("   • Total processing time: {:?}", execution_time + export_time + conversion_time);
    
    // Calculate throughput
    let allocations_per_second = allocations.len() as f64 / (export_time.as_secs_f64() + conversion_time.as_secs_f64());
    println!("   • Processing throughput: {:.0} allocations/sec", allocations_per_second);

    println!("\n📁 Generated files:");
    println!("   • Binary format: {}", binary_path);
    println!("   • JSON format: {}", json_path);
    println!("   • Check ./MemoryAnalysis/{}/", project_name);

    println!("\n🎉 Multithreading performance test completed!");
    
    Ok(())
}