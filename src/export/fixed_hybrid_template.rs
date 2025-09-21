//! Fixed Hybrid Template System for Multi-threaded and Async Memory Analysis
//!
//! This module provides a unified template system that combines lockfree multi-threaded
//! tracking data with async memory analysis, creating comprehensive visualizations
//! that showcase variable details across multiple threads and tasks.

use crate::async_memory::visualization::VisualizationConfig;
use crate::lockfree::{LockfreeAnalysis};
use std::collections::HashMap;

/// Fixed hybrid template configuration for rendering complex data
#[derive(Debug)]
pub struct FixedHybridTemplate {
    thread_count: usize,
    task_count: usize,
    variable_details_enabled: bool,
    render_mode: RenderMode,
}

/// Rendering mode for different visualization approaches
#[derive(Debug, Clone)]
pub enum RenderMode {
    Comprehensive,
    ThreadFocused,
    TaskFocused,
    VariableDetailed,
}

/// Combined analysis data from multiple sources
#[derive(Debug)]
pub struct HybridAnalysisData {
    pub lockfree_analysis: Option<LockfreeAnalysis>,
    pub visualization_config: VisualizationConfig,
    pub thread_task_mapping: HashMap<usize, Vec<usize>>,
    pub variable_registry: HashMap<String, VariableDetail>,
}

/// Detailed variable information for template rendering
#[derive(Debug, Clone)]
pub struct VariableDetail {
    pub name: String,
    pub type_info: String,
    pub thread_id: usize,
    pub task_id: Option<usize>,
    pub allocation_count: u64,
    pub memory_usage: u64,
    pub lifecycle_stage: LifecycleStage,
}

/// Variable lifecycle tracking stages
#[derive(Debug, Clone)]
pub enum LifecycleStage {
    Allocated,
    Active,
    Shared,
    Deallocated,
}

impl FixedHybridTemplate {
    /// Create new fixed hybrid template with specified configuration
    pub fn new(thread_count: usize, task_count: usize) -> Self {
        Self {
            thread_count,
            task_count,
            variable_details_enabled: true,
            render_mode: RenderMode::Comprehensive,
        }
    }

    /// Configure rendering mode for template output
    pub fn with_render_mode(mut self, mode: RenderMode) -> Self {
        self.render_mode = mode;
        self
    }

    /// Enable or disable detailed variable tracking
    pub fn with_variable_details(mut self, enabled: bool) -> Self {
        self.variable_details_enabled = enabled;
        self
    }

    /// Generate comprehensive HTML dashboard with hybrid data
    pub fn generate_hybrid_dashboard(
        &self,
        data: &HybridAnalysisData,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut html_content = String::with_capacity(50000);
        
        // Build HTML structure
        html_content.push_str(&self.build_html_header());
        html_content.push_str(&self.build_navigation_bar());
        html_content.push_str(&self.build_overview_section(data)?);
        html_content.push_str(&self.build_thread_task_matrix(data)?);
        html_content.push_str(&self.build_variable_details_section(data)?);
        html_content.push_str(&self.build_performance_metrics(data)?);
        html_content.push_str(&self.build_html_footer());

        Ok(html_content)
    }

    /// Build HTML header with styles and scripts
    fn build_html_header(&self) -> String {
        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hybrid Memory Analysis - {} Threads, {} Tasks</title>
    <style>
        body {{ 
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0; padding: 20px; background: #f5f7fa;
        }}
        .container {{ max-width: 1400px; margin: 0 auto; }}
        .nav-bar {{ 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            padding: 15px; border-radius: 10px; margin-bottom: 20px;
            color: white; text-align: center; font-size: 24px; font-weight: bold;
        }}
        .section {{ 
            background: white; margin: 20px 0; padding: 25px;
            border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.1);
        }}
        .matrix-grid {{ 
            display: grid; gap: 15px; margin-top: 20px;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
        }}
        .thread-card {{ 
            border: 2px solid #e1e8ed; border-radius: 8px; padding: 15px;
            background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
            color: white;
        }}
        .task-item {{ 
            background: rgba(255,255,255,0.2); margin: 8px 0;
            padding: 10px; border-radius: 6px; font-size: 14px;
        }}
        .variable-grid {{ 
            display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 15px; margin-top: 15px;
        }}
        .variable-card {{ 
            border-left: 4px solid #4CAF50; padding: 15px;
            background: #f8f9fa; border-radius: 6px;
        }}
        .metric-row {{ 
            display: flex; justify-content: space-between;
            padding: 12px 0; border-bottom: 1px solid #eee;
        }}
        .metric-value {{ font-weight: bold; color: #2c3e50; }}
        .lifecycle-badge {{ 
            display: inline-block; padding: 4px 8px; border-radius: 12px;
            font-size: 12px; font-weight: bold; color: white;
        }}
        .allocated {{ background: #28a745; }}
        .active {{ background: #007bff; }}
        .shared {{ background: #ffc107; color: #212529; }}
        .deallocated {{ background: #6c757d; }}
    </style>
</head>
<body>
    <div class="container">
"#, self.thread_count, self.task_count)
    }

    /// Build navigation bar
    fn build_navigation_bar(&self) -> String {
        format!(
            r#"<div class="nav-bar">
                Fixed Hybrid Template Analysis - {} Threads × {} Tasks
            </div>"#,
            self.thread_count, self.task_count
        )
    }

    /// Build overview section with summary statistics
    fn build_overview_section(&self, data: &HybridAnalysisData) -> Result<String, Box<dyn std::error::Error>> {
        let total_variables = data.variable_registry.len();
        let total_memory = data.variable_registry.values()
            .map(|v| v.memory_usage)
            .sum::<u64>();
        let active_variables = data.variable_registry.values()
            .filter(|v| matches!(v.lifecycle_stage, LifecycleStage::Active))
            .count();

        Ok(format!(r#"
        <div class="section">
            <h2>System Overview</h2>
            <div class="metric-row">
                <span>Total Variables Tracked:</span>
                <span class="metric-value">{}</span>
            </div>
            <div class="metric-row">
                <span>Active Variables:</span>
                <span class="metric-value">{}</span>
            </div>
            <div class="metric-row">
                <span>Total Memory Usage:</span>
                <span class="metric-value">{:.2} MB</span>
            </div>
            <div class="metric-row">
                <span>Thread-Task Mappings:</span>
                <span class="metric-value">{}</span>
            </div>
        </div>
        "#, total_variables, active_variables, total_memory as f64 / 1024.0 / 1024.0, data.thread_task_mapping.len()))
    }

    /// Build thread-task matrix visualization
    fn build_thread_task_matrix(&self, data: &HybridAnalysisData) -> Result<String, Box<dyn std::error::Error>> {
        let mut matrix_html = String::from(r#"
        <div class="section">
            <h2>Thread-Task Matrix</h2>
            <div class="matrix-grid">
        "#);

        // Generate thread cards with associated tasks
        for thread_id in 0..self.thread_count {
            let empty_tasks = vec![];
            let tasks = data.thread_task_mapping.get(&thread_id).unwrap_or(&empty_tasks);
            let variables_in_thread = data.variable_registry.values()
                .filter(|v| v.thread_id == thread_id)
                .count();

            matrix_html.push_str(&format!(r#"
                <div class="thread-card">
                    <h3>Thread {}</h3>
                    <div class="metric-row">
                        <span>Variables:</span>
                        <span>{}</span>
                    </div>
                    <div class="metric-row">
                        <span>Tasks:</span>
                        <span>{}</span>
                    </div>
            "#, thread_id, variables_in_thread, tasks.len()));

            // Add task items
            for &task_id in tasks {
                let task_variables = data.variable_registry.values()
                    .filter(|v| v.thread_id == thread_id && v.task_id == Some(task_id))
                    .count();
                
                matrix_html.push_str(&format!(r#"
                    <div class="task-item">
                        Task {}: {} variables
                    </div>
                "#, task_id, task_variables));
            }

            matrix_html.push_str("</div>");
        }

        matrix_html.push_str("</div></div>");
        Ok(matrix_html)
    }

    /// Build detailed variable information section
    fn build_variable_details_section(&self, data: &HybridAnalysisData) -> Result<String, Box<dyn std::error::Error>> {
        if !self.variable_details_enabled {
            return Ok(String::new());
        }

        let mut details_html = String::from(r#"
        <div class="section">
            <h2>Variable Details</h2>
            <div class="variable-grid">
        "#);

        // Sort variables by memory usage for better visualization
        let mut sorted_variables: Vec<_> = data.variable_registry.values().collect();
        sorted_variables.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));

        // Display top variables with detailed information
        for variable in sorted_variables.iter().take(20) {
            let lifecycle_class = match variable.lifecycle_stage {
                LifecycleStage::Allocated => "allocated",
                LifecycleStage::Active => "active",
                LifecycleStage::Shared => "shared",
                LifecycleStage::Deallocated => "deallocated",
            };

            let task_info = variable.task_id
                .map(|id| format!("Task {}", id))
                .unwrap_or_else(|| "No Task".to_string());

            details_html.push_str(&format!(r#"
                <div class="variable-card">
                    <h4>{}</h4>
                    <div class="metric-row">
                        <span>Type:</span>
                        <span>{}</span>
                    </div>
                    <div class="metric-row">
                        <span>Thread:</span>
                        <span>Thread {}</span>
                    </div>
                    <div class="metric-row">
                        <span>Task:</span>
                        <span>{}</span>
                    </div>
                    <div class="metric-row">
                        <span>Memory:</span>
                        <span>{:.2} KB</span>
                    </div>
                    <div class="metric-row">
                        <span>Allocations:</span>
                        <span>{}</span>
                    </div>
                    <div class="metric-row">
                        <span>Status:</span>
                        <span class="lifecycle-badge {}">
                            {:?}
                        </span>
                    </div>
                </div>
            "#, 
            variable.name, 
            variable.type_info, 
            variable.thread_id, 
            task_info,
            variable.memory_usage as f64 / 1024.0,
            variable.allocation_count,
            lifecycle_class,
            variable.lifecycle_stage
            ));
        }

        details_html.push_str("</div></div>");
        Ok(details_html)
    }

    /// Build performance metrics section
    fn build_performance_metrics(&self, data: &HybridAnalysisData) -> Result<String, Box<dyn std::error::Error>> {
        let thread_metrics = self.calculate_thread_metrics(data);
        let task_metrics = self.calculate_task_metrics(data);

        Ok(format!(r#"
        <div class="section">
            <h2>Performance Metrics</h2>
            <div class="metric-row">
                <span>Average Variables per Thread:</span>
                <span class="metric-value">{:.1}</span>
            </div>
            <div class="metric-row">
                <span>Average Memory per Thread:</span>
                <span class="metric-value">{:.2} MB</span>
            </div>
            <div class="metric-row">
                <span>Average Variables per Task:</span>
                <span class="metric-value">{:.1}</span>
            </div>
            <div class="metric-row">
                <span>Memory Efficiency:</span>
                <span class="metric-value">{:.1}%</span>
            </div>
        </div>
        "#, 
        thread_metrics.avg_variables_per_thread,
        thread_metrics.avg_memory_per_thread / 1024.0 / 1024.0,
        task_metrics.avg_variables_per_task,
        task_metrics.memory_efficiency * 100.0
        ))
    }

    /// Build HTML footer
    fn build_html_footer(&self) -> String {
        r#"
    </div>
</body>
</html>"#.to_string()
    }

    /// Calculate thread-level performance metrics
    fn calculate_thread_metrics(&self, data: &HybridAnalysisData) -> ThreadMetrics {
        let total_variables = data.variable_registry.len() as f64;
        let total_memory: u64 = data.variable_registry.values()
            .map(|v| v.memory_usage)
            .sum();

        ThreadMetrics {
            avg_variables_per_thread: total_variables / self.thread_count as f64,
            avg_memory_per_thread: total_memory as f64 / self.thread_count as f64,
        }
    }

    /// Calculate task-level performance metrics
    fn calculate_task_metrics(&self, data: &HybridAnalysisData) -> TaskMetrics {
        let total_variables = data.variable_registry.len() as f64;
        let active_variables = data.variable_registry.values()
            .filter(|v| matches!(v.lifecycle_stage, LifecycleStage::Active))
            .count() as f64;

        TaskMetrics {
            avg_variables_per_task: total_variables / self.task_count as f64,
            memory_efficiency: if total_variables > 0.0 { active_variables / total_variables } else { 0.0 },
        }
    }
}

/// Thread performance metrics
#[derive(Debug)]
struct ThreadMetrics {
    avg_variables_per_thread: f64,
    avg_memory_per_thread: f64,
}

/// Task performance metrics
#[derive(Debug)]
struct TaskMetrics {
    avg_variables_per_task: f64,
    memory_efficiency: f64,
}

/// Create sample hybrid analysis data for demonstration
pub fn create_sample_hybrid_data(thread_count: usize, task_count: usize) -> HybridAnalysisData {
    let mut variable_registry = HashMap::new();
    let mut thread_task_mapping = HashMap::new();

    // Generate thread-task mappings
    for thread_id in 0..thread_count {
        let tasks_per_thread = (task_count / thread_count).max(1);
        let start_task = thread_id * tasks_per_thread;
        let end_task = ((thread_id + 1) * tasks_per_thread).min(task_count);
        let tasks: Vec<usize> = (start_task..end_task).collect();
        thread_task_mapping.insert(thread_id, tasks);
    }

    // Generate sample variables
    let mut _variable_counter = 0;
    for thread_id in 0..thread_count {
        let tasks = thread_task_mapping.get(&thread_id).unwrap();
        
        for &task_id in tasks {
            // Create variables for each task
            for var_idx in 0..((thread_id + 1) * 2) {
                let variable_name = format!("var_t{}_task{}_v{}", thread_id, task_id, var_idx);
                let variable = VariableDetail {
                    name: variable_name.clone(),
                    type_info: format!("Type{}", var_idx % 4),
                    thread_id,
                    task_id: Some(task_id),
                    allocation_count: (var_idx as u64 + 1) * 10,
                    memory_usage: (var_idx as u64 + 1) * 1024 * (thread_id as u64 + 1),
                    lifecycle_stage: match var_idx % 4 {
                        0 => LifecycleStage::Active,
                        1 => LifecycleStage::Allocated,
                        2 => LifecycleStage::Shared,
                        _ => LifecycleStage::Deallocated,
                    },
                };
                variable_registry.insert(variable_name, variable);
                _variable_counter += 1;
            }
        }
    }

    // Create visualization config
    let visualization_config = VisualizationConfig::default();

    HybridAnalysisData {
        lockfree_analysis: None,
        visualization_config,
        thread_task_mapping,
        variable_registry,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let template = FixedHybridTemplate::new(5, 6);
        assert_eq!(template.thread_count, 5);
        assert_eq!(template.task_count, 6);
        assert!(template.variable_details_enabled);
    }

    #[test]
    fn test_sample_data_generation() {
        let data = create_sample_hybrid_data(3, 4);
        assert_eq!(data.thread_task_mapping.len(), 3);
        assert!(!data.variable_registry.is_empty());
    }

    #[test]
    fn test_html_generation() {
        let template = FixedHybridTemplate::new(2, 3);
        let data = create_sample_hybrid_data(2, 3);
        let result = template.generate_hybrid_dashboard(&data);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("Thread-Task Matrix"));
        assert!(html.contains("Variable Details"));
    }
}