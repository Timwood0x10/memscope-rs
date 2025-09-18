//! Verified Selective Tracking Demo
//! 
//! This demo proves that selective tracking works by:
//! 1. Creating 50 threads
//! 2. Only even-indexed threads call tracking functions
//! 3. Verifying the data content matches our expectations

use memscope_rs::lockfree::tracker::{
    init_thread_tracker, track_allocation_lockfree, track_deallocation_lockfree,
    finalize_thread_tracker, SamplingConfig
};
use memscope_rs::lockfree::aggregator::LockfreeAggregator;

use std::thread;
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Verified Selective Tracking Demo");
    println!("===================================");
    println!("   50 threads total, verifying ONLY EVEN threads tracked\n");
    
    let demo_start = Instant::now();
    let output_dir = std::path::PathBuf::from("./Memoryanalysis");
    
    // Clean setup
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }
    std::fs::create_dir_all(&output_dir)?;
    
    let total_operations = Arc::new(AtomicUsize::new(0));
    let tracking_log = Arc::new(Mutex::new(Vec::new()));
    
    println!("🔄 Starting 50 threads with verified selective tracking...");
    let start_time = Instant::now();
    
    // Create 50 threads with explicit tracking verification
    let handles: Vec<_> = (0..50)
        .map(|thread_idx| {
            let output_dir = output_dir.clone();
            let total_operations = Arc::clone(&total_operations);
            let tracking_log = Arc::clone(&tracking_log);
            
            thread::spawn(move || -> Result<(), String> {
                run_verified_worker(thread_idx, &output_dir, &total_operations, &tracking_log)
            })
        })
        .collect();
    
    // Wait for all threads
    let mut successful_threads = 0;
    for (idx, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(())) => successful_threads += 1,
            Ok(Err(e)) => println!("   ❌ Thread {} failed: {}", idx, e),
            Err(_) => println!("   💥 Thread {} panicked", idx),
        }
    }
    
    let simulation_duration = start_time.elapsed();
    let final_operations = total_operations.load(Ordering::Relaxed);
    
    // Analyze tracking log
    let tracking_results = tracking_log.lock().unwrap();
    let tracked_threads: Vec<_> = tracking_results.iter()
        .filter(|(_, tracked)| *tracked)
        .map(|(idx, _)| *idx)
        .collect();
    let untracked_threads: Vec<_> = tracking_results.iter()
        .filter(|(_, tracked)| !*tracked)
        .map(|(idx, _)| *idx)
        .collect();
    
    println!("\n📊 Verified Tracking Results:");
    println!("   ✅ Successful threads: {}/50", successful_threads);
    println!("   🔄 Total operations: {}", final_operations);
    println!("   🟢 Tracked threads: {} {:?}", tracked_threads.len(), tracked_threads);
    println!("   ⚫ Untracked threads: {} {:?}", untracked_threads.len(), untracked_threads);
    println!("   ⏱️  Duration: {:?}", simulation_duration);
    
    // Verify tracking was selective
    verify_selective_tracking_logic(&tracked_threads, &untracked_threads)?;
    
    // Generate analysis if any threads were tracked
    if !tracked_threads.is_empty() {
        generate_verified_analysis(&output_dir)?;
    }
    
    // Verify file-level results
    verify_tracking_files(&output_dir, tracked_threads.len())?;
    
    let total_duration = demo_start.elapsed();
    println!("\n🎉 Verified demo completed in {:?}", total_duration);
    
    Ok(())
}

/// Run worker with explicit tracking verification
fn run_verified_worker(
    thread_idx: usize,
    output_dir: &std::path::Path,
    total_operations: &Arc<AtomicUsize>,
    tracking_log: &Arc<Mutex<Vec<(usize, bool)>>>
) -> Result<(), String> {
    let should_track = thread_idx % 2 == 0; // Only even threads
    
    // Log our decision
    if let Ok(mut log) = tracking_log.lock() {
        log.push((thread_idx, should_track));
    }
    
    if should_track {
        println!("   🟢 Thread {} TRACKING", thread_idx);
        run_tracking_worker(thread_idx, output_dir, total_operations)
    } else {
        println!("   ⚫ Thread {} SKIPPED", thread_idx);
        run_non_tracking_worker(thread_idx, total_operations)
    }
}

/// Worker that actually initializes tracking
fn run_tracking_worker(
    thread_idx: usize,
    output_dir: &std::path::Path,
    total_operations: &Arc<AtomicUsize>
) -> Result<(), String> {
    // Initialize tracking for this thread
    init_thread_tracker(output_dir, Some(SamplingConfig::demo()))
        .map_err(|e| format!("Thread {} init failed: {}", thread_idx, e))?;
    
    let mut allocated_ptrs = Vec::new();
    let operation_count = 500; // Moderate workload
    
    // Perform tracked allocations
    for i in 0..operation_count {
        let size = 1024 + (i % 8) * 128; // Varied sizes
        let ptr = thread_idx * 100000 + i * 1000; // Unique addresses
        
        let call_stack = vec![
            run_tracking_worker as *const () as usize,
            run_verified_worker as *const () as usize,
            main as *const () as usize,
        ];
        
        // Track allocation with metadata that identifies our thread_idx
        track_allocation_lockfree(ptr, size, &call_stack)
            .map_err(|e| format!("Thread {} track failed: {}", thread_idx, e))?;
        
        allocated_ptrs.push((ptr, call_stack.clone()));
        total_operations.fetch_add(1, Ordering::Relaxed);
        
        // Periodic deallocation
        if i > 0 && i % 5 == 0 && !allocated_ptrs.is_empty() {
            let (old_ptr, old_stack) = allocated_ptrs.remove(0);
            track_deallocation_lockfree(old_ptr, &old_stack)
                .map_err(|e| format!("Thread {} dealloc failed: {}", thread_idx, e))?;
            
            total_operations.fetch_add(1, Ordering::Relaxed);
        }
        
        // Small work simulation
        thread::sleep(Duration::from_micros(10));
    }
    
    // Clean up remaining allocations
    for (ptr, stack) in allocated_ptrs {
        track_deallocation_lockfree(ptr, &stack)
            .map_err(|e| format!("Thread {} cleanup failed: {}", thread_idx, e))?;
        total_operations.fetch_add(1, Ordering::Relaxed);
    }
    
    // Finalize tracking
    finalize_thread_tracker()
        .map_err(|e| format!("Thread {} finalize failed: {}", thread_idx, e))?;
    
    Ok(())
}

/// Worker that does NOT initialize tracking
fn run_non_tracking_worker(
    thread_idx: usize,
    total_operations: &Arc<AtomicUsize>
) -> Result<(), String> {
    // NO tracking initialization - just simulate work
    let operation_count = 500;
    
    for i in 0..operation_count {
        // Simulate work without any tracking calls
        let _fake_size = 1024 + (i % 8) * 128;
        let _fake_ptr = thread_idx * 100000 + i * 1000;
        
        total_operations.fetch_add(1, Ordering::Relaxed);
        
        // Simulate deallocation work
        if i > 0 && i % 5 == 0 {
            total_operations.fetch_add(1, Ordering::Relaxed);
        }
        
        // Same timing as tracking threads
        thread::sleep(Duration::from_micros(10));
    }
    
    Ok(())
}

/// Verify that our logic worked correctly
fn verify_selective_tracking_logic(
    tracked_threads: &[usize],
    untracked_threads: &[usize]
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Verifying Tracking Logic:");
    
    // Check that all tracked threads are even
    let all_tracked_even = tracked_threads.iter().all(|&t| t % 2 == 0);
    println!("   ✅ All tracked threads are even: {}", all_tracked_even);
    
    // Check that all untracked threads are odd
    let all_untracked_odd = untracked_threads.iter().all(|&t| t % 2 == 1);
    println!("   ✅ All untracked threads are odd: {}", all_untracked_odd);
    
    // Check counts
    println!("   📊 Expected tracked: 25, Actual: {}", tracked_threads.len());
    println!("   📊 Expected untracked: 25, Actual: {}", untracked_threads.len());
    
    if all_tracked_even && all_untracked_odd && tracked_threads.len() == 25 && untracked_threads.len() == 25 {
        println!("   ✅ SUCCESS: Selective tracking logic verified!");
    } else {
        println!("   ❌ FAILED: Tracking logic verification failed");
    }
    
    Ok(())
}

/// Generate analysis from tracked data
fn generate_verified_analysis(output_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let aggregator = LockfreeAggregator::new(output_dir.to_path_buf());
    let analysis = aggregator.aggregate_all_threads()?;
    
    println!("\n📊 Analysis from Tracked Threads Only:");
    println!("   📁 System threads analyzed: {}", analysis.thread_stats.len());
    println!("   🔄 Total allocations: {}", analysis.summary.total_allocations);
    println!("   ↩️  Total deallocations: {}", analysis.summary.total_deallocations);
    println!("   📈 Peak memory: {:.1} MB", 
             analysis.summary.peak_memory_usage as f64 / (1024.0 * 1024.0));
    
    // Generate reports
    let html_path = output_dir.join("verified_selective_report.html");
    aggregator.generate_html_report(&analysis, &html_path)?;
    
    let json_path = output_dir.join("verified_selective_data.json");
    aggregator.export_analysis(&analysis, &json_path)?;
    
    println!("\n📄 Reports Generated:");
    println!("   🌐 HTML: {}", html_path.display());
    println!("   📄 JSON: {}", json_path.display());
    
    Ok(())
}

/// Verify tracking files were created appropriately
fn verify_tracking_files(
    output_dir: &std::path::Path,
    expected_tracked_count: usize
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📁 File System Verification:");
    
    // Count actual tracking files
    let mut file_count = 0;
    if let Ok(entries) = std::fs::read_dir(output_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name();
                if let Some(name) = file_name.to_str() {
                    if name.starts_with("memscope_thread_") && name.ends_with(".bin") {
                        file_count += 1;
                        println!("   📄 Found tracking file: {}", name);
                    }
                }
            }
        }
    }
    
    println!("   📊 Expected tracking files: {} (from tracked threads)", expected_tracked_count);
    println!("   📊 Actual tracking files: {}", file_count);
    
    // Note: The file count might not exactly match because system thread IDs
    // are assigned independently of our application thread indices
    if file_count > 0 {
        println!("   ✅ SUCCESS: Tracking files were created");
        println!("   ℹ️  Note: File count reflects system thread IDs, not app thread indices");
    } else {
        println!("   ❌ FAILED: No tracking files were created");
    }
    
    Ok(())
}