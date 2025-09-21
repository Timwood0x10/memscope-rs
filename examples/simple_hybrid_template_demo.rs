//! Simple Fixed Hybrid Template Demo - 5 Threads × 6 Tasks Showcase
//!
//! Simplified demonstration of the fixed hybrid template system focusing
//! on HTML generation and variable visualization without complex tracking.

use memscope_rs::export::fixed_hybrid_template::{
    create_sample_hybrid_data, FixedHybridTemplate, RenderMode
};
use std::time::Instant;

const THREAD_COUNT: usize = 24;
const TASK_COUNT: usize = 36;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Simple Fixed Hybrid Template Demo");
    println!("Configuration: {} Threads × {} Tasks", THREAD_COUNT, TASK_COUNT);
    
    let demo_start = Instant::now();
    
    // Phase 1: Generate sample data
    println!("Phase 1: Generating sample hybrid data...");
    let hybrid_data = create_sample_hybrid_data(THREAD_COUNT, TASK_COUNT);
    
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