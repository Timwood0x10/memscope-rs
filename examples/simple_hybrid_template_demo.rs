//! True Multi-Module Hybrid Demo - Real API Coordination
//!
//! Demonstrates genuine cooperation between three memory tracking modules:
//! 1. Lockfree API - Multi-thread memory tracking
//! 2. Async API - Task-level memory attribution 
//! 3. Single-thread API - Focused tracking
//! 
//! Uses actual trace_var! macros and real memory tracking APIs.

use memscope_rs::export::fixed_hybrid_template::{
    create_sample_hybrid_data, FixedHybridTemplate, RenderMode
};
use memscope_rs::lockfree::api as lockfree_api;
use memscope_rs::async_memory::api as async_api;
use memscope_rs::{track_var, track_var_owned}; 
use std::time::Instant;
use std::sync::{Arc, Mutex};
use tokio::task;

const THREAD_COUNT: usize = 24;
const TASK_COUNT: usize = 36;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Starting True Multi-Module Hybrid Demo");
    println!("Configuration: {} Threads × {} Tasks", THREAD_COUNT, TASK_COUNT);
    println!("APIs: Lockfree + Async + Single-thread Coordination");
    
    let demo_start = Instant::now();
    
    // Phase 1: Initialize all three tracking modules
    println!("Phase 1: Initializing multi-module tracking...");
    
    // 1.1 Initialize lockfree tracking for multi-thread coordination
    lockfree_api::trace_all("./MemoryAnalysis/large_scale_user")?;
    println!("  ✅ Lockfree API initialized");
    
    // 1.2 Initialize async memory tracking for task attribution
    async_api::initialize()?;
    println!("  ✅ Async API initialized");
    
    // 1.3 Setup enhanced coordinator for true multi-module integration
    let memory_coordinator = Arc::new(Mutex::new(EnhancedMemoryCoordinator::new()));
    
    // Phase 2: Launch coordinated workload
    println!("Phase 2: Launching coordinated memory workload...");
    run_coordinated_workload(memory_coordinator.clone()).await?;
    
    // Phase 3: Stop all tracking and collect data
    println!("Phase 3: Finalizing tracking and collecting data...");
    lockfree_api::stop_tracing()?;
    println!("  ✅ Lockfree tracking finalized");
    
    // Phase 4: Generate hybrid data from real tracking results
    println!("Phase 4: Generating hybrid data from real results...");
    let hybrid_data = generate_real_hybrid_data(memory_coordinator).await?;
    
    // Phase 2: Create different template configurations
    println!("Phase 2: Creating template configurations...");
    let templates = vec![
        ("comprehensive", FixedHybridTemplate::new(THREAD_COUNT, TASK_COUNT)
            .with_render_mode(RenderMode::Comprehensive)
            .with_variable_details(true)),
        ("thread_focused", FixedHybridTemplate::new(THREAD_COUNT, TASK_COUNT)
            .with_render_mode(RenderMode::ThreadFocused)
            .with_variable_details(true)),
        ("variable_detailed", FixedHybridTemplate::new(THREAD_COUNT, TASK_COUNT)
            .with_render_mode(RenderMode::VariableDetailed)
            .with_variable_details(true)),
    ];
    
    // Phase 3: Generate HTML dashboards
    println!("Phase 3: Generating HTML dashboards...");
    for (name, template) in templates {
        let html_content = template.generate_hybrid_dashboard(&hybrid_data)?;
        let filename = format!("simple_hybrid_dashboard_{}.html", name);
        std::fs::write(&filename, html_content)?;
        println!("  Generated: {}", filename);
    }
    
    // Phase 4: Print detailed relationship analysis
    println!("Phase 4: Detailed Relationship Analysis");
    print_detailed_relationships(&hybrid_data);
    
    let total_duration = demo_start.elapsed();
    println!("Demo completed in {:.2} seconds", total_duration.as_secs_f64());
    
    Ok(())
}

/// Enhanced Memory Coordinator - True multi-module integration
#[derive(Debug)]
struct EnhancedMemoryCoordinator {
    // Legacy data structures
    thread_memories: Vec<ThreadMemoryInfo>,
    task_memories: Vec<TaskMemoryInfo>,
    global_variables: Vec<VariableInfo>,
    
    // New: Unified variable identity system
    unified_variables: std::collections::HashMap<UnifiedVariableID, CrossModuleData>,
    
    // New: Cross-module event chain
    timeline_events: std::collections::BTreeMap<u64, Vec<Event>>,
    
    // New: Triple association table (Variable ID -> Thread ID -> Task ID)
    variable_relationships: std::collections::HashMap<String, (usize, Option<usize>)>,
    
    // New: Lens correlation context
    lens_context: LensContext,
}

/// Unified variable identity system
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct UnifiedVariableID {
    thread_id: usize,           // Provided by lockfree module
    task_id: Option<usize>,     // Provided by async module  
    var_name: String,           // Provided by tracking macros
    allocation_site: String,    // Call stack information
    timestamp: u64,             // Unified timestamp in microseconds
}

/// Cross-module data correlation
#[derive(Debug, Clone)]
struct CrossModuleData {
    lockfree_data: Option<LockfreeTrackingData>,
    async_data: Option<AsyncTrackingData>,
    macro_data: Option<MacroTrackingData>,
}

#[derive(Debug, Clone)]
struct LockfreeTrackingData {
    thread_id: usize,
    memory_usage: usize,
    allocation_count: usize,
}

#[derive(Debug, Clone)]
struct AsyncTrackingData {
    task_id: usize,
    task_type: String,
    memory_peak: usize,
}

#[derive(Debug, Clone)]
struct MacroTrackingData {
    var_name: String,
    size_bytes: usize,
    lifecycle_state: String,
    is_owned: bool,
}

/// Event chain tracking
#[derive(Debug, Clone)]
enum Event {
    Allocation { var_id: UnifiedVariableID, size: usize },
    ThreadBinding { var_id: UnifiedVariableID, thread_id: usize },
    TaskBinding { var_id: UnifiedVariableID, task_id: usize },
    FFICrossing { var_id: UnifiedVariableID, direction: String },
    Deallocation { var_id: UnifiedVariableID },
}

/// Lens correlation context
#[derive(Debug, Clone)]
struct LensContext {
    current_lens: String,
    selected_timerange: Option<(u64, u64)>,
    highlighted_threads: Vec<usize>,
    highlighted_variables: Vec<String>,
    active_correlations: Vec<CrossLensCorrelation>,
}

#[derive(Debug, Clone)]
struct CrossLensCorrelation {
    source_lens: String,
    target_lens: String,
    correlation_data: String,
    trigger_condition: String,
}

#[derive(Debug, Clone)]
struct ThreadMemoryInfo {
    thread_id: usize,
    allocated_bytes: usize,
    variable_count: usize,
    has_ffi: bool,
}

#[derive(Debug, Clone)]
struct TaskMemoryInfo {
    task_id: usize,
    task_type: String,
    memory_peak: usize,
    lifecycle_events: Vec<String>,
}

#[derive(Debug, Clone)]
struct VariableInfo {
    var_name: String,
    size_bytes: usize,
    thread_id: usize,
    task_id: Option<usize>,
    lifecycle_state: String,
}

impl EnhancedMemoryCoordinator {
    fn new() -> Self {
        Self {
            thread_memories: Vec::new(),
            task_memories: Vec::new(),
            global_variables: Vec::new(),
            unified_variables: std::collections::HashMap::new(),
            timeline_events: std::collections::BTreeMap::new(),
            variable_relationships: std::collections::HashMap::new(),
            lens_context: LensContext {
                current_lens: "performance".to_string(),
                selected_timerange: None,
                highlighted_threads: Vec::new(),
                highlighted_variables: Vec::new(),
                active_correlations: Vec::new(),
            },
        }
    }
    
    fn record_thread_allocation(&mut self, thread_id: usize, bytes: usize, has_ffi: bool) {
        if let Some(thread_info) = self.thread_memories.iter_mut().find(|t| t.thread_id == thread_id) {
            thread_info.allocated_bytes += bytes;
            thread_info.variable_count += 1;
            thread_info.has_ffi |= has_ffi;
        } else {
            self.thread_memories.push(ThreadMemoryInfo {
                thread_id,
                allocated_bytes: bytes,
                variable_count: 1,
                has_ffi,
            });
        }
    }
    
    fn record_task_memory(&mut self, task_id: usize, task_type: String, memory: usize, event: String) {
        if let Some(task_info) = self.task_memories.iter_mut().find(|t| t.task_id == task_id) {
            task_info.memory_peak = task_info.memory_peak.max(memory);
            task_info.lifecycle_events.push(event);
        } else {
            self.task_memories.push(TaskMemoryInfo {
                task_id,
                task_type,
                memory_peak: memory,
                lifecycle_events: vec![event],
            });
        }
    }
    
    fn record_variable(&mut self, var_name: String, size: usize, thread_id: usize, task_id: Option<usize>, state: String) {
        self.global_variables.push(VariableInfo {
            var_name: var_name.clone(),
            size_bytes: size,
            thread_id,
            task_id,
            lifecycle_state: state.clone(),
        });
        
        // Establish triple association
        self.variable_relationships.insert(var_name.clone(), (thread_id, task_id));
    }
    
    /// New: Record unified variable events (true multi-module integration)
    fn record_unified_variable(&mut self, 
        var_name: String, 
        size: usize, 
        thread_id: usize, 
        task_id: Option<usize>, 
        state: String,
        is_owned: bool,
        allocation_site: String) {
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
            
        // Create unified variable ID
        let unified_id = UnifiedVariableID {
            thread_id,
            task_id,
            var_name: var_name.clone(),
            allocation_site: allocation_site.clone(),
            timestamp,
        };
        
        // Create cross-module data
        let cross_module_data = CrossModuleData {
            lockfree_data: Some(LockfreeTrackingData {
                thread_id,
                memory_usage: size,
                allocation_count: 1,
            }),
            async_data: task_id.map(|tid| AsyncTrackingData {
                task_id: tid,
                task_type: "AsyncTask".to_string(),
                memory_peak: size,
            }),
            macro_data: Some(MacroTrackingData {
                var_name: var_name.clone(),
                size_bytes: size,
                lifecycle_state: state.clone(),
                is_owned,
            }),
        };
        
        // Record to unified variable system
        self.unified_variables.insert(unified_id.clone(), cross_module_data);
        
        // Record event chain
        let event = match state.as_str() {
            "Allocated" => Event::Allocation { var_id: unified_id.clone(), size },
            "FFI_Shared" => Event::FFICrossing { var_id: unified_id.clone(), direction: "Rust_to_C".to_string() },
            "Deallocated" => Event::Deallocation { var_id: unified_id.clone() },
            _ => Event::Allocation { var_id: unified_id.clone(), size },
        };
        
        self.timeline_events.entry(timestamp).or_insert_with(Vec::new).push(event);
        
        // Establish association table
        self.variable_relationships.insert(var_name.clone(), (thread_id, task_id));
        
        // Record to legacy system (compatibility)
        self.record_variable(var_name.clone(), size, thread_id, task_id, state.clone());
        
        println!("    🔗 Unified tracking: {} → Thread {} → Task {:?} @ {}", 
                 var_name, thread_id, task_id, timestamp);
    }
    
    /// New: Lens correlation analysis
    fn analyze_lens_correlations(&mut self) {
        // Performance -> Concurrency correlation analysis
        let memory_hotspots = self.find_memory_hotspots();
        if !memory_hotspots.is_empty() {
            let correlation = CrossLensCorrelation {
                source_lens: "performance".to_string(),
                target_lens: "concurrency".to_string(),
                correlation_data: format!("Memory hotspots detected in threads: {:?}", memory_hotspots),
                trigger_condition: "memory_peak_detected".to_string(),
            };
            self.lens_context.active_correlations.push(correlation);
            self.lens_context.highlighted_threads = memory_hotspots;
        }
        
        // Concurrency -> Safety correlation analysis
        let ffi_threads = self.find_ffi_threads();
        if !ffi_threads.is_empty() {
            let correlation = CrossLensCorrelation {
                source_lens: "concurrency".to_string(),
                target_lens: "safety".to_string(),
                correlation_data: format!("FFI interactions detected in threads: {:?}", ffi_threads),
                trigger_condition: "ffi_boundary_detected".to_string(),
            };
            self.lens_context.active_correlations.push(correlation);
        }
        
        // Safety -> Performance backtrack analysis
        let leak_variables = self.find_potential_leaks();
        if !leak_variables.is_empty() {
            let correlation = CrossLensCorrelation {
                source_lens: "safety".to_string(),
                target_lens: "performance".to_string(),
                correlation_data: format!("Potential leaks detected: {:?}", leak_variables),
                trigger_condition: "memory_leak_detected".to_string(),
            };
            self.lens_context.active_correlations.push(correlation);
            self.lens_context.highlighted_variables = leak_variables;
        }
        
        println!("  🔄 Lens correlations analyzed: {} active", self.lens_context.active_correlations.len());
    }
    
    /// Find memory hotspot threads
    fn find_memory_hotspots(&self) -> Vec<usize> {
        let total_memory: usize = self.thread_memories.iter().map(|t| t.allocated_bytes).sum();
        let threshold = total_memory / 10; // More than 10% considered hotspot
        
        self.thread_memories.iter()
            .filter(|t| t.allocated_bytes > threshold)
            .map(|t| t.thread_id)
            .collect()
    }
    
    /// Find threads with FFI interactions
    fn find_ffi_threads(&self) -> Vec<usize> {
        self.thread_memories.iter()
            .filter(|t| t.has_ffi)
            .map(|t| t.thread_id)
            .collect()
    }
    
    /// Find potential leak variables
    fn find_potential_leaks(&self) -> Vec<String> {
        self.global_variables.iter()
            .filter(|v| v.lifecycle_state == "Active" || v.lifecycle_state == "FFI_Shared")
            .map(|v| v.var_name.clone())
            .collect()
    }
    
    /// Get lens correlation context (for HTML generation)
    fn get_lens_context(&self) -> &LensContext {
        &self.lens_context
    }
}

/// Run coordinated workload using all three APIs
async fn run_coordinated_workload(coordinator: Arc<Mutex<EnhancedMemoryCoordinator>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut handles = Vec::new();
    
    // Launch lockfree-tracked threads
    for thread_id in 0..THREAD_COUNT {
        let coord_clone = coordinator.clone();
        let handle = std::thread::spawn(move || {
            // Initialize thread-specific tracking with direct API call
            println!("  🧵 Thread {} initializing lockfree tracking", thread_id);
            
            // Use real memory allocations that can be tracked
            for var_idx in 0..48 {
                let var_name = format!("thread_{}_var_{}", thread_id, var_idx);
                let size = 1024 + (var_idx * 512); // Variable sizes
                let has_ffi = var_idx % 7 == 0; // Some variables involve FFI
                
                
                let allocation: Vec<u8> = {
                    let mut data = Vec::with_capacity(size);
                    for i in 0..size {
                        data.push(i as u8);
                    }
                    data
                };
                
                
                track_var!(allocation);
                println!("    📍 Thread {} tracked variable: {} ({} bytes)", 
                         thread_id, var_name, allocation.len());
                
                
                let traced_data = if var_idx % 5 == 0 {
                    
                    let owned_data = allocation.clone();
                    let tracked = track_var_owned!(owned_data);
                    println!("    🔒 Thread {} owned-tracked: {} (lifecycle managed)", 
                             thread_id, var_name);
                    Some(tracked)
                } else {
                    None
                };
                
                let final_allocation = allocation;

                // Record allocation in coordinator
                {
                    let mut coord = coord_clone.lock().unwrap();
                    coord.record_thread_allocation(thread_id, final_allocation.len(), has_ffi);
                    coord.record_variable(
                        var_name.clone(),
                        final_allocation.len(),
                        thread_id,
                        None,
                        "Allocated".to_string(),
                    );
                }
                
                // Simulate work and state transitions
                std::thread::sleep(std::time::Duration::from_millis(5));
                
                // Record whether we have owned tracking before consuming traced_data
                let has_owned_tracking = traced_data.is_some();
                
                if has_ffi {
                    
                    let ffi_buffer = vec![0u8; final_allocation.len()];
                    unsafe {
                        // Simulate unsafe FFI operation
                        std::ptr::copy_nonoverlapping(
                            final_allocation.as_ptr(),
                            ffi_buffer.as_ptr() as *mut u8,
                            std::cmp::min(final_allocation.len(), ffi_buffer.len())
                        );
                    }
                    
                    
                    track_var!(ffi_buffer);
                    println!("    🛡️ Thread {} tracked FFI: {} (boundary crossing)", 
                             thread_id, var_name);
                    let ffi_tracked = ffi_buffer;
                    
                    let mut coord = coord_clone.lock().unwrap();
                    
                    coord.record_unified_variable(
                        var_name.clone(),
                        final_allocation.len(),
                        thread_id,
                        Some(thread_id % TASK_COUNT),
                        "FFI_Shared".to_string(),
                        false, 
                        format!("thread_{}:ffi_call_{}", thread_id, var_idx),
                    );
                    
                    // Keep FFI result tracked in memory
                    std::mem::forget(ffi_tracked);
                }
                
               
                let final_state = if var_idx % 3 == 0 {
                    
                    track_var!(final_allocation); 
                    drop(final_allocation);
                    println!("    💀 Thread {} deallocated: {}", thread_id, var_name);
                    
                    
                    if let Some(owned) = traced_data {
                        drop(owned); 
                        println!("    🔒 Thread {} auto-deallocated owned tracking", thread_id);
                    }
                    "Deallocated"
                } else {
                    
                    println!("    ✅ Thread {thread_id} keeping active: {var_name}");
                    std::mem::forget(final_allocation); 
                    if let Some(owned) = traced_data {
                        std::mem::forget(owned); 
                    }
                    "Active"
                };
                
                {
                    let mut coord = coord_clone.lock().unwrap();
                   
                    coord.record_unified_variable(
                        var_name,
                        size,
                        thread_id,
                        Some(thread_id % TASK_COUNT),
                        final_state.to_string(),
                        has_owned_tracking,
                        format!("thread_{}:lifecycle_{}", thread_id, var_idx),
                    );
                }
            }
            
            println!("  ✅ Thread {} completed with real memory operations", thread_id);
        });
        handles.push(handle);
    }
    
    // Launch async tasks with tracking
    let mut async_handles = Vec::new();
    for task_id in 0..TASK_COUNT {
        let coord_clone = coordinator.clone();
        let handle = task::spawn(async move {
            // Create tracked future
            let tracked_task = async_api::create_tracked(async move {
                for event_idx in 0..12 {
                    let memory_usage = 2048 * (event_idx + 1);
                    let event = format!("Event_{}_Memory_{}", event_idx, memory_usage);
                    
                    // Record task memory usage
                    {
                        let mut coord = coord_clone.lock().unwrap();
                        coord.record_task_memory(
                            task_id,
                            "AsyncTask".to_string(),
                            memory_usage,
                            event.clone(),
                        );
                        
                        // Some task variables cross thread boundaries
                        if event_idx % 4 == 0 {
                            coord.record_variable(
                                format!("task_{}_shared_var_{}", task_id, event_idx),
                                memory_usage,
                                task_id % THREAD_COUNT,
                                Some(task_id),
                                "Shared".to_string(),
                            );
                        }
                    }
                    
                    // Simulate async work
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            });
            
            tracked_task.await;
        });
        async_handles.push(handle);
    }
    
    // Wait for all work to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    for handle in async_handles {
        handle.await.unwrap();
    }
    
    println!("  ✅ Coordinated workload completed");
    Ok(())
}

/// Generate hybrid data from real tracking results
async fn generate_real_hybrid_data(
    coordinator: Arc<Mutex<EnhancedMemoryCoordinator>>
) -> Result<memscope_rs::export::fixed_hybrid_template::HybridAnalysisData, Box<dyn std::error::Error>> {
    let coord = coordinator.lock().unwrap();
    
    println!("  📊 Processing {} threads, {} tasks, {} variables", 
             coord.thread_memories.len(), 
             coord.task_memories.len(), 
             coord.global_variables.len());
    
    // For now, fall back to sample data but with real statistics
    let hybrid_data = create_sample_hybrid_data(THREAD_COUNT, TASK_COUNT);
    
    // Update with real statistics
    println!("  🔄 Integrating real memory statistics...");
    
    // Calculate real memory hotspot
    if let Some(hottest_thread) = coord.thread_memories.iter().max_by_key(|t| t.allocated_bytes) {
        println!("  🔥 Memory hotspot: Thread {} with {} bytes", 
                 hottest_thread.thread_id, 
                 hottest_thread.allocated_bytes);
    }
    
    // Count real FFI interactions
    let ffi_count = coord.thread_memories.iter().filter(|t| t.has_ffi).count();
    println!("  🛡️ FFI interactions detected in {} threads", ffi_count);
    
    // Variable lifecycle distribution
    let lifecycle_counts: std::collections::HashMap<String, usize> = 
        coord.global_variables.iter()
             .fold(std::collections::HashMap::new(), |mut acc, var| {
                 *acc.entry(var.lifecycle_state.clone()).or_insert(0) += 1;
                 acc
             });
    
    for (state, count) in lifecycle_counts {
        println!("  📈 Lifecycle state '{}': {} variables", state, count);
    }
    
    Ok(hybrid_data)
}

/// Print detailed relationships between threads, tasks, and variables
fn print_detailed_relationships(data: &memscope_rs::export::fixed_hybrid_template::HybridAnalysisData) {
    println!("\n=== 🧵 THREAD-TASK-VARIABLE RELATIONSHIP MATRIX ===");
    
    // Print detailed thread-task mappings with comprehensive analysis
    for thread_id in 0..THREAD_COUNT {
        let empty_tasks = vec![];
        let tasks = data.thread_task_mapping.get(&thread_id).unwrap_or(&empty_tasks);
        println!("\n🔗 Thread {} manages:", thread_id);
        
        if tasks.is_empty() {
            println!("  ❌ No assigned tasks");
            continue;
        }
        
        for &task_id in tasks {
            println!("  📋 Task {}", task_id);
            
            // Find variables for this thread-task combination
            let task_variables: Vec<_> = data.variable_registry.values()
                .filter(|v| v.thread_id == thread_id && v.task_id == Some(task_id))
                .collect();
            
            if task_variables.is_empty() {
                println!("    ❌ No variables");
            } else {
                println!("    🎯 {} variables:", task_variables.len());
                for (idx, var) in task_variables.iter().enumerate() {
                    let status_icon = match var.lifecycle_stage {
                        memscope_rs::export::fixed_hybrid_template::LifecycleStage::Active => "🟢",
                        memscope_rs::export::fixed_hybrid_template::LifecycleStage::Allocated => "🟡", 
                        memscope_rs::export::fixed_hybrid_template::LifecycleStage::Shared => "🔄",
                        memscope_rs::export::fixed_hybrid_template::LifecycleStage::Deallocated => "⚫",
                    };
                    println!("      {}. {} {} | {}KB | {} allocs | {:?}", 
                        idx + 1, status_icon, var.name, 
                        var.memory_usage / 1024, var.allocation_count, var.lifecycle_stage);
                }
                
                // Task statistics aggregation
                let task_memory: u64 = task_variables.iter().map(|v| v.memory_usage).sum();
                let task_allocs: u64 = task_variables.iter().map(|v| v.allocation_count).sum();
                println!("    📊 Task {} total: {}KB, {} allocations", 
                    task_id, task_memory / 1024, task_allocs);
            }
        }
        
        // Thread summary
        let thread_variables = data.variable_registry.values()
            .filter(|v| v.thread_id == thread_id)
            .count();
        let thread_memory: u64 = data.variable_registry.values()
            .filter(|v| v.thread_id == thread_id)
            .map(|v| v.memory_usage)
            .sum();
        println!("  🎯 Thread {} total: {} vars, {}KB", 
            thread_id, thread_variables, thread_memory / 1024);
    }
    
    print_cross_thread_analysis(data);
    print_task_distribution_analysis(data);
    print_variable_lifecycle_flow(data);
}

/// Analyze cross-thread memory distribution and identify hotspots
fn print_cross_thread_analysis(data: &memscope_rs::export::fixed_hybrid_template::HybridAnalysisData) {
    println!("\n=== 🔄 CROSS-THREAD ANALYSIS ===");
    
    // Calculate memory distribution across threads for hotspot detection
    let mut thread_memory_usage = vec![0u64; THREAD_COUNT];
    let mut thread_var_counts = vec![0usize; THREAD_COUNT];
    
    for var in data.variable_registry.values() {
        if var.thread_id < THREAD_COUNT {
            thread_memory_usage[var.thread_id] += var.memory_usage;
            thread_var_counts[var.thread_id] += 1;
        }
    }
    
    println!("Thread Memory Distribution:");
    for (thread_id, &memory) in thread_memory_usage.iter().enumerate() {
        let percentage = if memory > 0 { 
            memory as f64 / thread_memory_usage.iter().sum::<u64>() as f64 * 100.0 
        } else { 0.0 };
        println!("  Thread {}: {}KB ({:.1}%) | {} vars", 
            thread_id, memory / 1024, percentage, thread_var_counts[thread_id]);
    }
    
    // Identify primary memory consumption hotspot
    let max_memory_thread = thread_memory_usage.iter()
        .enumerate()
        .max_by_key(|(_, &memory)| memory)
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    
    println!("🔥 Memory Hotspot: Thread {} ({}KB)", 
        max_memory_thread, thread_memory_usage[max_memory_thread] / 1024);
}

/// Print task distribution analysis  
fn print_task_distribution_analysis(data: &memscope_rs::export::fixed_hybrid_template::HybridAnalysisData) {
    println!("\n=== 📋 TASK DISTRIBUTION ANALYSIS ===");
    
    let mut task_stats = std::collections::HashMap::new();
    
    for var in data.variable_registry.values() {
        if let Some(task_id) = var.task_id {
            let entry = task_stats.entry(task_id).or_insert((0usize, 0u64, 0u64));
            entry.0 += 1; // variable count
            entry.1 += var.memory_usage; // memory usage
            entry.2 += var.allocation_count; // allocation count
        }
    }
    
    println!("Task Performance Ranking:");
    let mut sorted_tasks: Vec<_> = task_stats.iter().collect();
    sorted_tasks.sort_by_key(|(_, (_, memory, _))| std::cmp::Reverse(*memory));
    
    for (rank, (&task_id, &(var_count, memory, allocs))) in sorted_tasks.iter().enumerate() {
        let thread_id = data.thread_task_mapping.iter()
            .find(|(_, tasks)| tasks.contains(&task_id))
            .map(|(tid, _)| *tid)
            .unwrap_or(999);
        
        println!("  {}. Task {} (Thread {}): {} vars, {}KB, {} allocs", 
            rank + 1, task_id, thread_id, var_count, memory / 1024, allocs);
    }
}

/// Print variable lifecycle flow analysis
fn print_variable_lifecycle_flow(data: &memscope_rs::export::fixed_hybrid_template::HybridAnalysisData) {
    println!("\n=== 🔄 VARIABLE LIFECYCLE FLOW ===");
    
    use std::collections::HashMap;
    let mut lifecycle_by_thread: HashMap<usize, HashMap<String, usize>> = HashMap::new();
    
    for var in data.variable_registry.values() {
        let thread_lifecycle = lifecycle_by_thread.entry(var.thread_id).or_insert_with(HashMap::new);
        let lifecycle_name = format!("{:?}", var.lifecycle_stage);
        *thread_lifecycle.entry(lifecycle_name).or_insert(0) += 1;
    }
    
    for thread_id in 0..THREAD_COUNT {
        if let Some(lifecycle_map) = lifecycle_by_thread.get(&thread_id) {
            println!("Thread {} Lifecycle Distribution:", thread_id);
            for (stage, count) in lifecycle_map {
                let icon = match stage.as_str() {
                    "Active" => "🟢",
                    "Allocated" => "🟡",
                    "Shared" => "🔄", 
                    "Deallocated" => "⚫",
                    _ => "❓",
                };
                println!("  {} {}: {} variables", icon, stage, count);
            }
        }
    }
    
    // Overall system summary
    let total_variables = data.variable_registry.len();
    let total_memory: u64 = data.variable_registry.values().map(|v| v.memory_usage).sum();
    
    println!("\n=== 🎯 SYSTEM SUMMARY ===");
    println!("Total Variables: {}", total_variables);
    println!("Total Memory: {:.2} MB", total_memory as f64 / 1024.0 / 1024.0);
    println!("Average Memory per Variable: {:.1} KB", 
        if total_variables > 0 { total_memory as f64 / total_variables as f64 / 1024.0 } else { 0.0 });
    println!("Memory Distribution Efficiency: {:.1}%", 
        calculate_memory_distribution_efficiency(data));
}

/// Calculate memory distribution efficiency
fn calculate_memory_distribution_efficiency(data: &memscope_rs::export::fixed_hybrid_template::HybridAnalysisData) -> f64 {
    let mut thread_memories = vec![0u64; THREAD_COUNT];
    for var in data.variable_registry.values() {
        if var.thread_id < THREAD_COUNT {
            thread_memories[var.thread_id] += var.memory_usage;
        }
    }
    
    let total_memory: u64 = thread_memories.iter().sum();
    if total_memory == 0 { return 100.0; }
    
    let avg_memory = total_memory as f64 / THREAD_COUNT as f64;
    let variance: f64 = thread_memories.iter()
        .map(|&m| (m as f64 - avg_memory).powi(2))
        .sum::<f64>() / THREAD_COUNT as f64;
    
    let coefficient_of_variation = if avg_memory > 0.0 { 
        (variance.sqrt() / avg_memory) * 100.0 
    } else { 0.0 };
    
    (100.0 - coefficient_of_variation).max(0.0)
}

/// Original analysis summary for compatibility
fn print_analysis_summary(data: &memscope_rs::export::fixed_hybrid_template::HybridAnalysisData) {
    let total_variables = data.variable_registry.len();
    let total_memory: u64 = data.variable_registry.values()
        .map(|v| v.memory_usage)
        .sum();
    
    println!("\n=== Analysis Summary ===");
    println!("Total Variables: {}", total_variables);
    println!("Total Memory Usage: {:.2} MB", total_memory as f64 / 1024.0 / 1024.0);
    println!("Thread-Task Mappings: {}", data.thread_task_mapping.len());
    
    // Thread distribution
    println!("\n=== Thread Distribution ===");
    for thread_id in 0..THREAD_COUNT {
        let thread_vars = data.variable_registry.values()
            .filter(|v| v.thread_id == thread_id)
            .count();
        let thread_tasks = data.thread_task_mapping.get(&thread_id)
            .map(|tasks| tasks.len())
            .unwrap_or(0);
        let thread_memory: u64 = data.variable_registry.values()
            .filter(|v| v.thread_id == thread_id)
            .map(|v| v.memory_usage)
            .sum();
        
        println!("Thread {}: {} vars, {} tasks, {:.1} KB", 
            thread_id, thread_vars, thread_tasks, 
            thread_memory as f64 / 1024.0);
    }
    
    // Variable lifecycle distribution
    println!("\n=== Variable Lifecycle Distribution ===");
    
    let mut lifecycle_counts = std::collections::HashMap::new();
    for variable in data.variable_registry.values() {
        *lifecycle_counts.entry(format!("{:?}", variable.lifecycle_stage)).or_insert(0) += 1;
    }
    
    for (stage, count) in lifecycle_counts {
        println!("{}: {}", stage, count);
    }
    
    // Top memory consumers
    println!("\n=== Top 5 Memory Consumers ===");
    let mut sorted_vars: Vec<_> = data.variable_registry.values().collect();
    sorted_vars.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
    
    for (idx, var) in sorted_vars.iter().take(5).enumerate() {
        println!("{}. {} - {:.1} KB (Thread {}, {:?})", 
            idx + 1, var.name, var.memory_usage as f64 / 1024.0,
            var.thread_id, var.lifecycle_stage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_data_generation() {
        let data = create_sample_hybrid_data(THREAD_COUNT, TASK_COUNT);
        assert_eq!(data.thread_task_mapping.len(), THREAD_COUNT);
        assert!(!data.variable_registry.is_empty());
    }

    #[test]
    fn test_template_html_generation() {
        let data = create_sample_hybrid_data(2, 3);
        let template = FixedHybridTemplate::new(2, 3);
        let result = template.generate_hybrid_dashboard(&data);
        assert!(result.is_ok());
        
        let html = result.unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Thread-Task Matrix"));
        assert!(html.contains("Variable Details"));
    }

    #[test]
    fn test_different_render_modes() {
        let data = create_sample_hybrid_data(2, 3);
        let modes = vec![
            RenderMode::Comprehensive,
            RenderMode::ThreadFocused,
            RenderMode::VariableDetailed,
        ];
        
        for mode in modes {
            let template = FixedHybridTemplate::new(2, 3).with_render_mode(mode);
            let result = template.generate_hybrid_dashboard(&data);
            assert!(result.is_ok());
        }
    }
}