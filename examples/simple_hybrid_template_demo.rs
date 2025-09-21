//! Simple Fixed Hybrid Template Demo - 5 Threads × 6 Tasks Showcase
//!
//! Simplified demonstration of the fixed hybrid template system focusing
//! on HTML generation and variable visualization without complex tracking.

use memscope_rs::export::fixed_hybrid_template::{
    create_sample_hybrid_data, FixedHybridTemplate, RenderMode
};
use std::time::Instant;

const THREAD_COUNT: usize = 5;
const TASK_COUNT: usize = 6;

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
    
    // Phase 4: Print analysis summary
    println!("Phase 4: Analysis Summary");
    print_analysis_summary(&hybrid_data);
    
    let total_duration = demo_start.elapsed();
    println!("Demo completed in {:.2} seconds", total_duration.as_secs_f64());
    
    Ok(())
}

/// Print comprehensive analysis summary
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