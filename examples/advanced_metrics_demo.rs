//! Advanced Memory Metrics Demo
//!
//! This example demonstrates advanced memory analysis capabilities including:
//! - Complex data structures and relationships
//! - Unsafe code and FFI interactions
//! - Smart pointer analysis
//! - Memory layout optimization
//! - Lifecycle tracking
//! - Performance profiling
//!
//! The demo creates files in ./MemoryAnalysis/advanced_metrics_demo/ directory:
//! - advanced_metrics_demo.memscope (binary format)
//! - advanced_metrics_demo_*.json (5 categorized JSON files)
//!
//! Use 'make html DIR=MemoryAnalysis/advanced_metrics_demo' to generate HTML reports

use memscope_rs::{core::tracker::MemoryTracker, get_global_tracker, track_var};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced Memory Metrics Demo");
    println!("===============================");

    // Create output directory
    let output_dir = Path::new("./MemoryAnalysis/advanced_metrics_demo");
    fs::create_dir_all(output_dir)?;
    println!("📁 Created output directory: {}", output_dir.display());

    // Get the global memory tracker
    let tracker = get_global_tracker();

    // Create advanced memory allocations with complex relationships
    println!("\n📊 Creating advanced memory scenarios...");

    // 1. Complex nested data structures
    create_complex_data_structures()?;

    // 2. Smart pointer relationships and circular references
    create_smart_pointer_scenarios()?;

    // 3. Unsafe code and FFI demonstrations
    create_unsafe_ffi_scenarios()?;

    // 4. Multi-threaded shared data
    create_multithreaded_scenarios()?;

    // 5. Memory layout optimization examples
    create_layout_optimization_examples()?;

    // 6. Performance-critical allocations
    create_performance_critical_allocations()?;

    // Add some simple main-thread allocations with clear variable names
    println!("   Creating main-thread allocations with clear variable names...");

    let main_thread_buffer = vec![42u8; 1024];
    track_var!(main_thread_buffer);

    let main_thread_map = {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), vec![1, 2, 3, 4, 5]);
        map.insert("key2".to_string(), vec![6, 7, 8, 9, 10]);
        map
    };
    track_var!(main_thread_map);

    let main_thread_string = "This is a main thread string for testing".to_string();
    track_var!(main_thread_string);

    let main_thread_tuple = (
        vec!["tuple_item_1".to_string(), "tuple_item_2".to_string()],
        vec![100, 200, 300],
        vec!["metadata".to_string()],
    );
    track_var!(main_thread_tuple);

    println!("✅ Created advanced allocation scenarios with rich metadata");

    // Simulate some complex operations and deallocations
    std::thread::sleep(std::time::Duration::from_millis(50));
    println!("✅ Simulated complex memory operations");

    // Export to binary format using MemoryTracker
    println!("\n💾 Exporting to binary format...");
    let start_time = std::time::Instant::now();
    tracker.export_to_binary("advanced_metrics_demo")?;
    let binary_export_time = start_time.elapsed();

    // Find the created binary file
    let binary_file = find_binary_file("MemoryAnalysis")?;
    let binary_size = fs::metadata(&binary_file)?.len();

    println!("✅ Binary export completed in {:?}", binary_export_time);
    println!(
        "📁 Binary file: {} ({} bytes)",
        binary_file.display(),
        binary_size
    );

    // Convert binary to standard JSON files (5 categorized files)
    println!("\n🔄 Converting binary to standard JSON files...");
    let start_time = std::time::Instant::now();
    MemoryTracker::parse_binary_to_standard_json(&binary_file, "advanced_metrics_demo")?;
    let json_conversion_time = start_time.elapsed();

    // Check the generated JSON files
    let json_files = [
        "advanced_metrics_demo_memory_analysis.json",
        "advanced_metrics_demo_lifetime.json",
        "advanced_metrics_demo_performance.json",
        "advanced_metrics_demo_unsafe_ffi.json",
        "advanced_metrics_demo_complex_types.json",
    ];

    let mut total_json_size = 0;
    println!(
        "✅ Standard JSON conversion completed in {:?}",
        json_conversion_time
    );
    println!("📄 Generated JSON files:");
    for json_file_name in &json_files {
        let json_file_path = output_dir.join(json_file_name);
        if json_file_path.exists() {
            let size = fs::metadata(&json_file_path)?.len();
            total_json_size += size;
            println!("  • {} ({} bytes)", json_file_name, size);
        }
    }

    println!("\n🌐 JSON files ready for HTML generation...");

    // Performance analysis
    println!("\n📈 Advanced Performance Analysis:");
    println!("=================================");

    // Export using standard JSON method for comparison
    let start_time = std::time::Instant::now();
    tracker.export_to_json("advanced_metrics_direct")?;
    let json_direct_time = start_time.elapsed();

    // Calculate performance metrics
    let speed_improvement =
        json_direct_time.as_nanos() as f64 / binary_export_time.as_nanos() as f64;
    let size_reduction =
        ((total_json_size as f64 - binary_size as f64) / total_json_size as f64) * 100.0;

    println!("Binary vs Standard JSON Export Performance:");
    println!("  📊 Binary export time:     {:?}", binary_export_time);
    println!("  📊 Standard JSON time:     {:?}", json_direct_time);
    println!(
        "  🚀 Speed improvement:      {:.2}x faster",
        speed_improvement
    );
    println!("  📁 Binary file size:       {} bytes", binary_size);
    println!(
        "  📁 JSON files size:        {} bytes (5 files)",
        total_json_size
    );
    println!("  💾 Size reduction:         {:.1}%", size_reduction);

    println!("\nConversion Performance:");
    println!("  🔄 Binary → 5 JSON files:  {:?}", json_conversion_time);

    // Advanced analysis of generated data
    println!("\n🔍 Advanced Memory Analysis:");
    println!("============================");

    // Analyze the most detailed JSON file (memory_analysis)
    let memory_analysis_file = output_dir.join("advanced_metrics_demo_memory_analysis.json");
    if memory_analysis_file.exists() {
        let json_content = fs::read_to_string(&memory_analysis_file)?;
        analyze_advanced_metrics(&json_content)?;
    }

    println!("\n🎉 Advanced demo completed successfully!");
    println!("📁 All files generated in: {}", output_dir.display());
    println!("📋 Generated files:");
    println!(
        "  • {} (binary format - {} bytes)",
        binary_file.display(),
        binary_size
    );
    for json_file_name in &json_files {
        let json_file_path = output_dir.join(json_file_name);
        if json_file_path.exists() {
            let size = fs::metadata(&json_file_path).map(|m| m.len()).unwrap_or(0);
            println!("  • {} ({} bytes)", json_file_name, size);
        }
    }

    println!("\n💡 Advanced Analysis Features Demonstrated:");
    println!("  ✅ Complex nested data structures");
    println!("  ✅ Smart pointer relationships");
    println!("  ✅ Unsafe code and FFI tracking");
    println!("  ✅ Multi-threaded memory sharing");
    println!("  ✅ Memory layout optimization");
    println!("  ✅ Performance-critical allocations");
    println!("  ✅ Binary → JSON pipeline");

    println!("\n🌐 Next steps:");
    println!("  1. Generate HTML report: make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo");
    println!("  2. Examine JSON files for detailed allocation data");
    println!("  3. Open the generated HTML file (memory_report.html) in your browser for interactive analysis");

    Ok(())
}

/// Create complex nested data structures
fn create_complex_data_structures() -> Result<(), Box<dyn std::error::Error>> {
    println!("   Creating complex nested data structures...");

    // Complex nested HashMap with multiple levels
    let mut complex_map = HashMap::new();
    for i in 0..10 {
        let mut inner_map = BTreeMap::new();
        for j in 0..5 {
            let data = vec![format!("data_{}_{}", i, j); 10];
            inner_map.insert(format!("key_{}", j), data);
        }
        complex_map.insert(format!("outer_key_{}", i), inner_map);
    }
    track_var!(complex_map);

    // Multi-dimensional vector structure
    let matrix: Vec<Vec<Vec<f64>>> = (0..5)
        .map(|i| {
            (0..5)
                .map(|j| (0..5).map(|k| (i * j * k) as f64).collect())
                .collect()
        })
        .collect();
    track_var!(matrix);

    // Complex data structure instead of enum
    let complex_data = {
        let mut map = HashMap::new();
        for i in 0..8 {
            map.insert(format!("complex_key_{}", i), vec![i as u8; 50]);
        }
        map
    };
    track_var!(complex_data);

    // Queue with complex elements
    let mut complex_queue = VecDeque::new();
    for i in 0..15 {
        let element = (
            format!("queue_item_{}", i),
            vec![i as f32; 20],
            HashMap::from([(format!("meta_{}", i), i * 2)]),
        );
        complex_queue.push_back(element);
    }
    track_var!(complex_queue);

    Ok(())
}

/// Create smart pointer scenarios with relationships
fn create_smart_pointer_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    println!("   Creating smart pointer relationships...");

    // Rc with multiple references
    let shared_data = Rc::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    let ref1 = Rc::clone(&shared_data);
    let ref2 = Rc::clone(&shared_data);
    let ref3 = Rc::clone(&shared_data);
    track_var!(shared_data);
    track_var!(ref1);
    track_var!(ref2);
    track_var!(ref3);

    // Arc for thread-safe sharing
    let thread_safe_data = Arc::new(Mutex::new(HashMap::new()));
    for i in 0..20 {
        let mut guard = thread_safe_data.lock().unwrap();
        guard.insert(format!("thread_key_{}", i), vec![i as u8; 30]);
    }
    let arc_ref1 = Arc::clone(&thread_safe_data);
    let arc_ref2 = Arc::clone(&thread_safe_data);
    track_var!(thread_safe_data);
    track_var!(arc_ref1);
    track_var!(arc_ref2);

    // RefCell for interior mutability
    let mutable_data = Rc::new(RefCell::new(vec![String::new(); 25]));
    {
        let mut borrowed = mutable_data.borrow_mut();
        for i in 0..25 {
            borrowed[i] = format!("mutable_item_{}", i);
        }
    }
    let refcell_ref = Rc::clone(&mutable_data);
    track_var!(mutable_data);
    track_var!(refcell_ref);

    // Box with nested structures
    let boxed_complex = Box::new((
        (0..15)
            .map(|i| format!("boxed_string_{}", i))
            .collect::<Vec<_>>(),
        HashMap::from([
            ("nested_vec".to_string(), vec![42u32; 40]),
            ("another_vec".to_string(), vec![84u32; 35]),
        ]),
    ));
    track_var!(boxed_complex);

    Ok(())
}

/// Create unsafe code and FFI scenarios
fn create_unsafe_ffi_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    println!("   Creating unsafe code and FFI scenarios...");

    // Raw pointer manipulation
    let raw_data = vec![0u8; 1024];
    let _raw_ptr = raw_data.as_ptr();
    track_var!(raw_data);

    // Simulate FFI-like allocation
    let ffi_buffer = unsafe {
        let layout = std::alloc::Layout::from_size_align(2048, 8).unwrap();
        let ptr = std::alloc::alloc(layout);
        if ptr.is_null() {
            return Err("FFI allocation failed".into());
        }
        // Initialize with some data
        std::ptr::write_bytes(ptr, 0xAB, 2048);
        Vec::from_raw_parts(ptr, 2048, 2048)
    };
    track_var!(ffi_buffer);

    // Unsafe data manipulation with trackable types
    let unsafe_data = unsafe {
        let mut data = vec![0u8; 8];
        let int_value: i64 = 0x123456789ABCDEF0;
        std::ptr::copy_nonoverlapping(&int_value as *const i64 as *const u8, data.as_mut_ptr(), 8);
        data[0] = 0xFF;
        data
    };
    track_var!(unsafe_data);

    // Manual memory management simulation
    let manual_buffer = unsafe {
        let size = 4096;
        let layout = std::alloc::Layout::from_size_align(size, 16).unwrap();
        let ptr = std::alloc::alloc_zeroed(layout) as *mut u64;
        if ptr.is_null() {
            return Err("Manual allocation failed".into());
        }

        // Fill with pattern
        for i in 0..(size / 8) {
            *ptr.add(i) = (i as u64).wrapping_mul(0x0123456789ABCDEF);
        }

        Vec::from_raw_parts(ptr as *mut u8, size, size)
    };
    track_var!(manual_buffer);

    Ok(())
}

/// Create multi-threaded scenarios with complex shared variable tracking
fn create_multithreaded_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    println!("   Creating multi-threaded scenarios with shared variable tracking...");

    // 1. Producer-Consumer pattern with shared buffer
    let shared_buffer = Arc::new(Mutex::new(VecDeque::new()));
    let buffer_stats = Arc::new(Mutex::new((0usize, 0usize))); // (produced, consumed)

    // Track the shared structures before threading
    let buffer_for_tracking = Arc::clone(&shared_buffer);
    let stats_for_tracking = Arc::clone(&buffer_stats);
    track_var!(buffer_for_tracking);
    track_var!(stats_for_tracking);

    let mut producer_handles = vec![];
    let mut consumer_handles = vec![];

    // Create multiple producers
    for producer_id in 0..3 {
        let buffer_clone = Arc::clone(&shared_buffer);
        let stats_clone = Arc::clone(&buffer_stats);

        let handle = std::thread::spawn(move || {
            for i in 0..15 {
                let item = format!("producer_{}_item_{}", producer_id, i);

                // Add to shared buffer
                {
                    let mut buffer = buffer_clone.lock().unwrap();
                    buffer.push_back(item);
                }

                // Update stats
                {
                    let mut stats = stats_clone.lock().unwrap();
                    stats.0 += 1; // increment produced count
                }

                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        });
        producer_handles.push(handle);
    }

    // Create multiple consumers
    for consumer_id in 0..2 {
        let buffer_clone = Arc::clone(&shared_buffer);
        let stats_clone = Arc::clone(&buffer_stats);

        let handle = std::thread::spawn(move || {
            let mut consumed_items = Vec::new();

            for _ in 0..20 {
                let item = {
                    let mut buffer = buffer_clone.lock().unwrap();
                    buffer.pop_front()
                };

                if let Some(item) = item {
                    consumed_items.push(format!("consumer_{}_got_{}", consumer_id, item));

                    // Update stats
                    {
                        let mut stats = stats_clone.lock().unwrap();
                        stats.1 += 1; // increment consumed count
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(3));
            }

            // Track consumer-specific data
            let consumer_data = (
                vec![consumer_id],
                consumed_items,
                vec![format!("consumer_{}_stats", consumer_id)],
            );
            track_var!(consumer_data);
        });
        consumer_handles.push(handle);
    }

    // Wait for all threads
    for handle in producer_handles {
        handle.join().unwrap();
    }
    for handle in consumer_handles {
        handle.join().unwrap();
    }

    // 2. Shared cache with read-write access patterns
    let shared_cache = Arc::new(std::sync::RwLock::new(HashMap::new()));
    let cache_metrics = Arc::new(Mutex::new((0usize, 0usize, 0usize))); // (reads, writes, misses)

    track_var!(shared_cache);
    track_var!(cache_metrics);

    let mut cache_handles = vec![];

    // Writer threads
    for writer_id in 0..2 {
        let cache_clone = Arc::clone(&shared_cache);
        let metrics_clone = Arc::clone(&cache_metrics);

        let handle = std::thread::spawn(move || {
            for i in 0..10 {
                let key = format!("key_{}", i % 5); // Limited key space for conflicts
                let value = vec![writer_id as u8; 50 + i * 10];

                {
                    let mut cache = cache_clone.write().unwrap();
                    cache.insert(key, value);
                }

                {
                    let mut metrics = metrics_clone.lock().unwrap();
                    metrics.1 += 1; // increment writes
                }

                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        });
        cache_handles.push(handle);
    }

    // Reader threads
    for reader_id in 0..4 {
        let cache_clone = Arc::clone(&shared_cache);
        let metrics_clone = Arc::clone(&cache_metrics);

        let handle = std::thread::spawn(move || {
            let mut read_data = Vec::new();

            for i in 0..15 {
                let key = format!("key_{}", i % 7); // Slightly different key space

                let value = {
                    let cache = cache_clone.read().unwrap();
                    cache.get(&key).cloned()
                };

                {
                    let mut metrics = metrics_clone.lock().unwrap();
                    metrics.0 += 1; // increment reads
                    if value.is_none() {
                        metrics.2 += 1; // increment misses
                    }
                }

                if let Some(data) = value {
                    read_data.push((key, data.len()));
                }

                std::thread::sleep(std::time::Duration::from_millis(1));
            }

            let reader_results = (
                vec![reader_id],
                read_data,
                vec![format!("reader_{}_stats", reader_id)],
            );
            track_var!(reader_results);
        });
        cache_handles.push(handle);
    }

    for handle in cache_handles {
        handle.join().unwrap();
    }

    // 3. Work-stealing queue simulation
    let work_queues: Vec<Arc<Mutex<VecDeque<String>>>> = (0..4)
        .map(|_| Arc::new(Mutex::new(VecDeque::new())))
        .collect();

    let work_stats = Arc::new(Mutex::new(vec![0usize; 4])); // work done per thread

    // Track work queues and stats
    for (_i, queue) in work_queues.iter().enumerate() {
        let queue_for_tracking = Arc::clone(queue);
        track_var!(queue_for_tracking);
    }
    track_var!(work_stats);

    let mut worker_handles = vec![];

    for worker_id in 0..4 {
        let queues_clone = work_queues.clone();
        let stats_clone = Arc::clone(&work_stats);

        let handle = std::thread::spawn(move || {
            // Initially populate own queue
            {
                let mut my_queue = queues_clone[worker_id].lock().unwrap();
                for i in 0..8 {
                    my_queue.push_back(format!("worker_{}_task_{}", worker_id, i));
                }
            }

            let mut completed_work = Vec::new();

            // Work loop with stealing
            for _ in 0..12 {
                // Try own queue first
                let mut work_item = {
                    let mut my_queue = queues_clone[worker_id].lock().unwrap();
                    my_queue.pop_front()
                };

                // If no work, try to steal from others
                if work_item.is_none() {
                    for other_id in 0..4 {
                        if other_id != worker_id {
                            let mut other_queue = queues_clone[other_id].lock().unwrap();
                            if let Some(stolen_work) = other_queue.pop_back() {
                                work_item =
                                    Some(format!("stolen_from_{}: {}", other_id, stolen_work));
                                break;
                            }
                        }
                    }
                }

                if let Some(work) = work_item {
                    completed_work.push(work);

                    // Update stats
                    {
                        let mut stats = stats_clone.lock().unwrap();
                        stats[worker_id] += 1;
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(2));
            }

            let worker_results = (
                vec![worker_id],
                completed_work,
                vec![format!("worker_{}_stats", worker_id)],
            );
            track_var!(worker_results);
        });
        worker_handles.push(handle);
    }

    for handle in worker_handles {
        handle.join().unwrap();
    }

    // 4. Atomic operations and lock-free structures
    let atomic_counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let atomic_flags = Arc::new([
        std::sync::atomic::AtomicBool::new(false),
        std::sync::atomic::AtomicBool::new(false),
        std::sync::atomic::AtomicBool::new(false),
        std::sync::atomic::AtomicBool::new(false),
    ]);

    track_var!(atomic_counter);
    track_var!(atomic_flags);

    let mut atomic_handles = vec![];

    for thread_id in 0..4 {
        let counter_clone = Arc::clone(&atomic_counter);
        let flags_clone = Arc::clone(&atomic_flags);

        let handle = std::thread::spawn(move || {
            let mut operations = Vec::new();

            for i in 0..20 {
                // Atomic increment
                let old_value = counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                operations.push(format!("increment: {} -> {}", old_value, old_value + 1));

                // Toggle flag
                let flag_index = i % 4;
                let old_flag =
                    flags_clone[flag_index].swap(i % 2 == 0, std::sync::atomic::Ordering::SeqCst);
                operations.push(format!(
                    "flag[{}]: {} -> {}",
                    flag_index,
                    old_flag,
                    i % 2 == 0
                ));

                std::thread::sleep(std::time::Duration::from_millis(1));
            }

            let atomic_results = (
                vec![thread_id],
                operations,
                vec![format!("atomic_{}_stats", thread_id)],
            );
            track_var!(atomic_results);
        });
        atomic_handles.push(handle);
    }

    for handle in atomic_handles {
        handle.join().unwrap();
    }

    println!("   ✅ Complex multi-threaded scenarios with shared variable tracking completed");
    Ok(())
}

/// Create memory layout optimization examples
fn create_layout_optimization_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("   Creating memory layout optimization examples...");

    // Optimized data structure using 3-tuple (which is trackable)
    let optimized_data = (
        vec![0x123456789ABCDEF0u64; 16], // large_field
        vec![0x12345678u32; 8],          // medium_field
        vec![0x1234u16; 4],              // small_field
    );
    track_var!(optimized_data);

    // Packed data using 3-tuple (space-efficient representation)
    let packed_data = (
        vec![0x12u8],
        vec![0x123456789ABCDEF0u64],
        vec![0x1234u16, 0x5678u16],
    );
    track_var!(packed_data);

    // Aligned data using vector
    let aligned_data = vec![0xABu8; 1024];
    track_var!(aligned_data);

    // Cache-friendly data structure using 3-tuple
    let cache_friendly = (
        vec![1u64, 2, 3, 4, 5, 6, 7, 8], // hot_data - frequently accessed
        (0..25)
            .map(|i| format!("cold_data_{}", i))
            .collect::<Vec<String>>(), // cold_data_1
        (25..50)
            .map(|i| format!("cold_data_{}", i))
            .collect::<Vec<String>>(), // cold_data_2
    );
    track_var!(cache_friendly);

    Ok(())
}

/// Create performance-critical allocations
fn create_performance_critical_allocations() -> Result<(), Box<dyn std::error::Error>> {
    println!("   Creating performance-critical allocations...");

    // Large contiguous buffer
    let large_buffer = vec![0u8; 1024 * 1024]; // 1MB
    track_var!(large_buffer);

    // Pre-allocated capacity vectors
    let mut preallocated_vec = Vec::with_capacity(10000);
    for i in 0..5000 {
        preallocated_vec.push(format!("preallocated_item_{}", i));
    }
    track_var!(preallocated_vec);

    // HashMap with pre-allocated capacity
    let mut preallocated_map = HashMap::with_capacity(1000);
    for i in 0..500 {
        preallocated_map.insert(format!("perf_key_{}", i), vec![i as u8; 100]);
    }
    track_var!(preallocated_map);

    // Ring buffer simulation using 3-tuple
    let mut ring_buffer_data = vec![0u64; 1024];
    let mut head = 0usize;
    let mut tail = 0usize;
    let capacity = 1024usize;

    // Fill ring buffer
    for i in 0..2048 {
        ring_buffer_data[tail] = i;
        tail = (tail + 1) % capacity;
        if tail == head {
            head = (head + 1) % capacity;
        }
    }

    let ring_buffer = (ring_buffer_data, vec![head], vec![tail, capacity]);
    track_var!(ring_buffer);

    // Memory pool simulation
    let memory_pool: Vec<Vec<u8>> = (0..100).map(|i| vec![i as u8; 256]).collect();
    track_var!(memory_pool);

    Ok(())
}

/// Analyze advanced metrics from JSON content
fn analyze_advanced_metrics(json_content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let allocation_count = json_content.matches("\"ptr\":").count();
    let smart_pointer_count = json_content.matches("smart_pointer").count();
    let unsafe_count = json_content.matches("unsafe").count();
    let thread_count = json_content.matches("thread").count();
    let complex_type_count = json_content.matches("HashMap").count()
        + json_content.matches("Vec").count()
        + json_content.matches("BTreeMap").count();

    println!("📊 Advanced Metrics Analysis:");
    println!("  • Total allocations: {}", allocation_count);
    println!("  • Smart pointer usage: {}", smart_pointer_count);
    println!("  • Unsafe operations: {}", unsafe_count);
    println!("  • Multi-threaded allocations: {}", thread_count);
    println!("  • Complex data structures: {}", complex_type_count);

    // Check for advanced features
    if json_content.contains("memory_layout") {
        println!("  ✅ Memory layout analysis available");
    }
    if json_content.contains("lifecycle_tracking") {
        println!("  ✅ Lifecycle tracking available");
    }
    if json_content.contains("performance_metrics") {
        println!("  ✅ Performance metrics available");
    }

    Ok(())
}

/// Find the binary file in the MemoryAnalysis directory
fn find_binary_file(base_dir: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let memory_analysis_dir = std::path::Path::new(base_dir);

    if !memory_analysis_dir.exists() {
        return Err("MemoryAnalysis directory not found".into());
    }

    // Look for .memscope files
    for entry in fs::read_dir(memory_analysis_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            for sub_entry in fs::read_dir(entry.path())? {
                let sub_entry = sub_entry?;
                if sub_entry.path().extension() == Some(std::ffi::OsStr::new("memscope")) {
                    return Ok(sub_entry.path());
                }
            }
        }
    }

    Err("No .memscope file found".into())
}
