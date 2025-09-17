//! Comprehensive 30-thread validation test
//!
//! This example validates the complete data collection pipeline for 30 threads,
//! ensuring all requirements from nextstep_v2.md are met with full data capture.

use memscope_rs::lockfree::tracker::{
    init_thread_tracker, track_allocation_lockfree, track_deallocation_lockfree,
    finalize_thread_tracker, SamplingConfig
};
use memscope_rs::lockfree::aggregator::LockfreeAggregator;

use std::thread;
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Comprehensive 30-Thread Validation");
    println!("   Validating complete data collection pipeline...\n");
    
    let demo_start = Instant::now();
    let output_dir = std::path::PathBuf::from("./Memoryanalysis");
    
    // Clean setup
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }
    std::fs::create_dir_all(&output_dir)?;
    
    // Comprehensive thread configurations
    let thread_configs = create_validation_configurations();
    let thread_count = thread_configs.len();
    let total_operations = Arc::new(AtomicUsize::new(0));
    
    assert_eq!(thread_count, 30, "Must test exactly 30 threads");
    
    println!("🔄 Starting {} validation threads...", thread_count);
    let start_time = Instant::now();
    
    // Execute all threads
    let handles: Vec<_> = thread_configs.into_iter().enumerate()
        .map(|(thread_idx, config)| {
            let output_dir = output_dir.clone();
            let total_operations = Arc::clone(&total_operations);
            
            thread::spawn(move || -> Result<ValidationResult, String> {
                run_validation_thread(thread_idx, config, &output_dir, &total_operations)
            })
        })
        .collect();
    
    // Collect results
    let mut thread_results = Vec::new();
    let mut successful_threads = 0;
    
    for (idx, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(result)) => {
                let (_tid, ops, allocs, _deallocs) = result.get_stats();
                if ops > 0 && allocs > 0 {
                    thread_results.push(result);
                    successful_threads += 1;
                }
            }
            Ok(Err(e)) => {
                println!("❌ Thread {} failed: {}", idx, e);
            }
            Err(e) => {
                println!("💥 Thread {} panicked: {:?}", idx, e);
            }
        }
    }
    
    let simulation_duration = start_time.elapsed();
    let final_operations = total_operations.load(Ordering::Relaxed);
    
    // Validate results
    println!("\n📊 Validation Results:");
    println!("   ✅ Successful threads: {}/{}", successful_threads, thread_count);
    println!("   🔄 Total operations: {}", final_operations);
    println!("   ⏱️  Duration: {:?}", simulation_duration);
    println!("   🚀 Operations/sec: {:.0}", final_operations as f64 / simulation_duration.as_secs_f64());
    
    // Validate file creation
    let bin_files = std::fs::read_dir(&output_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "bin"))
        .count();
    
    let freq_files = std::fs::read_dir(&output_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "freq"))
        .count();
    
    println!("\n📁 File Validation:");
    println!("   📄 Binary files: {}", bin_files);
    println!("   📊 Frequency files: {}", freq_files);
    
    // Generate and validate analysis
    if successful_threads == thread_count {
        println!("\n🔍 Generating comprehensive analysis...");
        let analysis_result = validate_analysis(&output_dir)?;
        
        let total_duration = demo_start.elapsed();
        println!("\n🎉 Validation completed in {:?}", total_duration);
        
        // Final validation checks
        validate_success_criteria(&analysis_result, thread_count, final_operations)?;
        
        println!("✨ SUCCESS: 30-thread comprehensive validation passed!");
        println!("📈 All data collection requirements from nextstep_v2.md fulfilled!");
    } else {
        println!("⚠️  Validation failed: {}/{} threads completed", successful_threads, thread_count);
    }
    
    Ok(())
}

#[derive(Debug)]
struct ValidationConfig {
    name: String,
    allocation_pattern: AllocationPattern,
    operation_count: usize,
}

#[derive(Debug)]
enum AllocationPattern {
    SmallFrequent,    // Many small allocations
    MediumBurst,      // Medium allocations in bursts
    LargeSparse,      // Few large allocations
    Mixed,            // Mixed pattern
}

#[derive(Debug)]
struct ValidationResult {
    thread_id: usize,
    operations_completed: usize,
    allocations_made: usize,
    deallocations_made: usize,
}

impl ValidationResult {
    /// Get summary statistics for this thread's validation
    fn get_stats(&self) -> (usize, usize, usize, usize) {
        (self.thread_id, self.operations_completed, self.allocations_made, self.deallocations_made)
    }
}

fn create_validation_configurations() -> Vec<ValidationConfig> {
    let mut configs = Vec::new();
    
    // Create 30 threads with diverse patterns
    for i in 0..30 {
        let pattern = match i % 4 {
            0 => AllocationPattern::SmallFrequent,
            1 => AllocationPattern::MediumBurst, 
            2 => AllocationPattern::LargeSparse,
            _ => AllocationPattern::Mixed,
        };
        
        let operation_count = match pattern {
            AllocationPattern::SmallFrequent => 2000,
            AllocationPattern::MediumBurst => 1000,
            AllocationPattern::LargeSparse => 500,
            AllocationPattern::Mixed => 1500,
        };
        
        configs.push(ValidationConfig {
            name: format!("Validator-{:02}-{:?}", i, pattern),
            allocation_pattern: pattern,
            operation_count,
        });
    }
    
    configs
}

fn run_validation_thread(
    thread_idx: usize,
    config: ValidationConfig,
    output_dir: &std::path::Path,
    total_operations: &Arc<AtomicUsize>
) -> Result<ValidationResult, String> {
    // Initialize with demo config for maximum data capture
    init_thread_tracker(output_dir, Some(SamplingConfig::demo()))
        .map_err(|e| format!("Thread {} init failed: {}", thread_idx, e))?;
    
    let mut allocations_made = 0;
    let mut deallocations_made = 0;
    let mut allocated_ptrs = Vec::new();
    
    // Execute allocation pattern
    for i in 0..config.operation_count {
        let (ptr, size) = generate_allocation_data(&config.allocation_pattern, thread_idx, i);
        let call_stack = generate_call_stack(thread_idx, i, &config.allocation_pattern);
        
        // Track allocation
        track_allocation_lockfree(ptr, size, &call_stack)
            .map_err(|e| format!("Thread {} allocation failed: {}", thread_idx, e))?;
        
        allocated_ptrs.push((ptr, call_stack.clone()));
        allocations_made += 1;
        total_operations.fetch_add(1, Ordering::Relaxed);
        
        // Pattern-specific deallocation strategy
        if should_deallocate(&config.allocation_pattern, i) && !allocated_ptrs.is_empty() {
            let (dealloc_ptr, dealloc_stack) = allocated_ptrs.remove(0);
            track_deallocation_lockfree(dealloc_ptr, &dealloc_stack)
                .map_err(|e| format!("Thread {} deallocation failed: {}", thread_idx, e))?;
            
            deallocations_made += 1;
            total_operations.fetch_add(1, Ordering::Relaxed);
        }
        
        // Realistic timing simulation
        if i % 100 == 0 {
            thread::sleep(Duration::from_micros(100));
        }
    }
    
    // Final cleanup pattern
    let cleanup_count = allocated_ptrs.len() / 2;
    for (ptr, call_stack) in allocated_ptrs.into_iter().take(cleanup_count) {
        track_deallocation_lockfree(ptr, &call_stack)
            .map_err(|e| format!("Thread {} cleanup failed: {}", thread_idx, e))?;
        
        deallocations_made += 1;
        total_operations.fetch_add(1, Ordering::Relaxed);
    }
    
    // Finalize tracking
    finalize_thread_tracker()
        .map_err(|e| format!("Thread {} finalize failed: {}", thread_idx, e))?;
    
    println!("   ✓ {} completed: {} allocs, {} deallocs", 
             config.name, allocations_made, deallocations_made);
    
    Ok(ValidationResult {
        thread_id: thread_idx,
        operations_completed: allocations_made + deallocations_made,
        allocations_made,
        deallocations_made,
    })
}

fn generate_allocation_data(pattern: &AllocationPattern, thread_idx: usize, iteration: usize) -> (usize, usize) {
    let base_ptr = 0x10000000 + (thread_idx * 0x1000000) + (iteration * 128);
    
    let size = match pattern {
        AllocationPattern::SmallFrequent => 64 + (iteration % 8) * 32,
        AllocationPattern::MediumBurst => 1024 + (iteration % 16) * 512,
        AllocationPattern::LargeSparse => 16384 + (iteration % 4) * 8192,
        AllocationPattern::Mixed => {
            match iteration % 3 {
                0 => 128,
                1 => 2048,
                _ => 32768,
            }
        }
    };
    
    (base_ptr, size)
}

fn generate_call_stack(thread_idx: usize, iteration: usize, pattern: &AllocationPattern) -> Vec<usize> {
    let pattern_id = match pattern {
        AllocationPattern::SmallFrequent => 1,
        AllocationPattern::MediumBurst => 2,
        AllocationPattern::LargeSparse => 3,
        AllocationPattern::Mixed => 4,
    };
    
    vec![
        0x500000 + thread_idx,
        0x600000 + pattern_id,
        0x700000 + (iteration % 20),
        0x800000 + (iteration / 100),
    ]
}

fn should_deallocate(pattern: &AllocationPattern, iteration: usize) -> bool {
    match pattern {
        AllocationPattern::SmallFrequent => iteration % 3 == 0,
        AllocationPattern::MediumBurst => iteration % 5 == 0,
        AllocationPattern::LargeSparse => iteration % 8 == 0,
        AllocationPattern::Mixed => iteration % 4 == 0,
    }
}

fn validate_analysis(output_dir: &std::path::Path) -> Result<AnalysisValidation, Box<dyn std::error::Error>> {
    let aggregator = LockfreeAggregator::new(output_dir.to_path_buf());
    let analysis = aggregator.aggregate_all_threads()?;
    
    println!("📊 Analysis Validation:");
    println!("   📁 Threads analyzed: {}", analysis.thread_stats.len());
    println!("   🔄 Total allocations: {}", analysis.summary.total_allocations);
    println!("   ↩️  Total deallocations: {}", analysis.summary.total_deallocations);
    println!("   📈 Peak memory: {:.2} MB", 
             analysis.summary.peak_memory_usage as f64 / (1024.0 * 1024.0));
    println!("   📊 Unique call stacks: {}", analysis.summary.unique_call_stacks);
    
    // Generate reports
    let json_path = output_dir.join("validation_analysis.json");
    aggregator.export_analysis(&analysis, &json_path)?;
    
    let html_path = output_dir.join("validation_report.html");
    aggregator.generate_html_report(&analysis, &html_path)?;
    
    println!("\n📄 Reports generated:");
    println!("   🌐 HTML: {}", html_path.display());
    println!("   📄 JSON: {}", json_path.display());
    
    Ok(AnalysisValidation {
        threads_analyzed: analysis.thread_stats.len(),
        total_allocations: analysis.summary.total_allocations,
        total_deallocations: analysis.summary.total_deallocations,
        peak_memory_mb: analysis.summary.peak_memory_usage as f64 / (1024.0 * 1024.0),
        unique_call_stacks: analysis.summary.unique_call_stacks as usize,
    })
}

#[derive(Debug)]
struct AnalysisValidation {
    threads_analyzed: usize,
    total_allocations: u64,
    total_deallocations: u64,
    peak_memory_mb: f64,
    unique_call_stacks: usize,
}

fn validate_success_criteria(
    analysis: &AnalysisValidation,
    expected_threads: usize,
    total_operations: usize
) -> Result<(), Box<dyn std::error::Error>> {
    
    println!("\n🎯 Success Criteria Validation:");
    
    // Criterion 1: All threads analyzed
    if analysis.threads_analyzed != expected_threads {
        return Err(format!("Expected {} threads, got {}", expected_threads, analysis.threads_analyzed).into());
    }
    println!("   ✅ Thread coverage: {}/{}", analysis.threads_analyzed, expected_threads);
    
    // Criterion 2: Substantial allocations captured
    if analysis.total_allocations < (total_operations as u64 / 4) {
        return Err(format!("Too few allocations captured: {}", analysis.total_allocations).into());
    }
    println!("   ✅ Allocation capture: {} allocations", analysis.total_allocations);
    
    // Criterion 3: Memory tracking active
    if analysis.peak_memory_mb < 1.0 {
        return Err(format!("Peak memory too low: {:.2} MB", analysis.peak_memory_mb).into());
    }
    println!("   ✅ Memory tracking: {:.2} MB peak", analysis.peak_memory_mb);
    
    // Criterion 4: Call stack diversity
    if analysis.unique_call_stacks < 1000 {
        return Err(format!("Too few unique call stacks: {}", analysis.unique_call_stacks).into());
    }
    println!("   ✅ Call stack diversity: {} unique patterns", analysis.unique_call_stacks);
    
    // Criterion 5: Deallocation tracking
    if analysis.total_deallocations == 0 {
        return Err("No deallocations tracked".into());
    }
    println!("   ✅ Deallocation tracking: {} deallocations", analysis.total_deallocations);
    
    println!("\n🏆 ALL SUCCESS CRITERIA MET!");
    
    Ok(())
}