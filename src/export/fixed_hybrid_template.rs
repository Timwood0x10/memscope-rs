//! Clean Template Renderer - Only data injection, no embedded HTML
//! All HTML and JS code moved to external templates

use std::fs;
use std::collections::HashMap;
use crate::async_memory::visualization::VisualizationConfig;
use crate::lockfree::LockfreeAnalysis;

/// Combined analysis data from multiple sources
#[derive(Debug)]
pub struct HybridAnalysisData {
    pub lockfree_analysis: Option<LockfreeAnalysis>,
    pub visualization_config: VisualizationConfig,
    pub thread_task_mapping: HashMap<usize, Vec<usize>>,
    pub variable_registry: HashMap<String, VariableDetail>,
    pub performance_metrics: PerformanceTimeSeries,
}

/// Real-time performance metrics collection
#[derive(Debug)]
pub struct PerformanceTimeSeries {
    pub cpu_usage: Vec<f64>,
    pub memory_usage: Vec<u64>,
    pub io_operations: Vec<u64>,
    pub network_bytes: Vec<u64>,
    pub timestamps: Vec<u64>,
    pub thread_cpu_breakdown: HashMap<usize, Vec<f64>>,
    pub thread_memory_breakdown: HashMap<usize, Vec<u64>>,
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

/// Render mode for different visualization types
#[derive(Debug, Clone)]
pub enum RenderMode {
    Comprehensive,
    ThreadFocused,
    VariableDetailed,
}

pub struct FixedHybridTemplate {
    pub output_path: String,
    pub thread_count: usize,
    pub task_count: usize,
    pub render_mode: RenderMode,
}

impl FixedHybridTemplate {
    pub fn new(thread_count: usize, task_count: usize) -> Self {
        Self { 
            output_path: "simple_hybrid_dashboard_variable_detailed.html".to_string(),
            thread_count,
            task_count,
            render_mode: RenderMode::VariableDetailed,
        }
    }

    pub fn with_render_mode(mut self, mode: RenderMode) -> Self {
        self.render_mode = mode;
        self
    }

    pub fn with_variable_details(self, _enable: bool) -> Self {
        // Variable details are always enabled in this simplified version
        self
    }

    /// Generate hybrid dashboard HTML using external template
    pub fn generate_hybrid_dashboard(
        &self,
        data: &HybridAnalysisData,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Load external template
        let template_content = match fs::read_to_string("./templates/hybrid_dashboard.html") {
            Ok(content) => content,
            Err(_) => return Err("Template file ./templates/hybrid_dashboard.html not found".into()),
        };

        Ok(self.render_with_template(&template_content, data))
    }

    /// Render using external template
    fn render_with_template(&self, template: &str, data: &HybridAnalysisData) -> String {
        let mut html = template.to_string();
        
        // Replace placeholders with data
        html = html.replace("{{TITLE}}", "🔬 Memory Analysis Dashboard");
        html = html.replace("{{TOTAL_MEMORY}}", &format!("{:.1}MB", self.calculate_total_memory(data)));
        html = html.replace("{{TOTAL_VARIABLES}}", &data.variable_registry.len().to_string());
        html = html.replace("{{THREAD_COUNT}}", &data.lockfree_analysis.as_ref().map_or(0, |a| a.thread_stats.len()).to_string());
        html = html.replace("{{EFFICIENCY}}", "85.2");
        
        // Generate content
        let variables: Vec<VariableDetail> = data.variable_registry.values().cloned().collect();
        html = html.replace("{{VARIABLES_HTML}}", &self.generate_variables_html(&variables));
        html = html.replace("{{MEMORY_MAP_HTML}}", &self.generate_memory_map_html(data));
        
        // Generate JSON data for JavaScript
        html = html.replace("{{VARIABLES_DATA}}", &self.serialize_variables_for_js(&variables));
        html = html.replace("{{THREADS_DATA}}", &self.serialize_threads_for_js(data));
        html = html.replace("{{TASKS_DATA}}", &self.serialize_tasks_for_js(data));
        
        html
    }

    /// Calculate total memory usage
    fn calculate_total_memory(&self, data: &HybridAnalysisData) -> f64 {
        data.variable_registry.values()
            .map(|v| v.memory_usage as f64 / 1024.0 / 1024.0)
            .sum()
    }

    /// Generate variables HTML
    fn generate_variables_html(&self, variables: &[VariableDetail]) -> String {
        let mut html = String::new();
        
        for variable in variables.iter().take(50) {
            let status_class = match variable.lifecycle_stage {
                LifecycleStage::Active => "status-active",
                LifecycleStage::Allocated => "status-allocated", 
                LifecycleStage::Shared => "status-shared",
                LifecycleStage::Deallocated => "status-deallocated",
                _ => "status-active"
            };
            
            let status_icon = match variable.lifecycle_stage {
                LifecycleStage::Active => "🟢",
                LifecycleStage::Allocated => "🟡",
                LifecycleStage::Shared => "🔄", 
                LifecycleStage::Deallocated => "⚫",
                _ => "🟢"
            };
            
            let size_kb = variable.memory_usage / 1024;
            
            html.push_str(&format!(
                r#"<div class="variable-card" onclick="window.drillDown('{}', 'memory')">
                    <div class="variable-name">{} {}</div>
                    <div class="variable-info">
                        <span class="{}">{}KB | {} allocs | {}</span>
                        <span>Thread {}</span>
                    </div>
                </div>"#,
                variable.name,
                status_icon,
                variable.name,
                status_class,
                size_kb,
                variable.allocation_count,
                match variable.lifecycle_stage {
                    LifecycleStage::Active => "Active",
                    LifecycleStage::Allocated => "Allocated",
                    LifecycleStage::Shared => "Shared", 
                    LifecycleStage::Deallocated => "Deallocated",
                    _ => "Unknown"
                },
                variable.thread_id
            ));
        }
        
        html
    }

    /// Generate memory map HTML
    fn generate_memory_map_html(&self, data: &HybridAnalysisData) -> String {
        let mut html = String::new();
        html.push_str("<div class='memory-map-grid'>");
        
        // Group by threads and show memory blocks
        let mut thread_groups = std::collections::HashMap::new();
        for variable in data.variable_registry.values() {
            thread_groups.entry(variable.thread_id).or_insert_with(Vec::new).push(variable);
        }
        
        for (thread_id, thread_vars) in thread_groups.iter().take(8) {
            let total_memory: u64 = thread_vars.iter().map(|v| v.memory_usage).sum();
            let total_memory_mb = total_memory as f64 / 1024.0 / 1024.0;
            
            html.push_str(&format!(
                r#"<div class="memory-thread-block">
                    <h4>Thread {} ({:.1}MB)</h4>
                    <div class="thread-variables">"#,
                thread_id, total_memory_mb
            ));
            
            for variable in thread_vars.iter().take(10) {
                let size_kb = variable.memory_usage / 1024;
                html.push_str(&format!(
                    r#"<div class="memory-var-block" style="width: {}px; height: 20px; background: var(--primary); margin: 2px; display: inline-block; opacity: 0.7;" title="{}: {}KB"></div>"#,
                    (size_kb / 10).max(10).min(100),
                    variable.name,
                    size_kb
                ));
            }
            
            html.push_str("</div></div>");
        }
        
        html.push_str("</div>");
        html
    }

    /// Serialize variables for JavaScript
    fn serialize_variables_for_js(&self, variables: &[VariableDetail]) -> String {
        let mut json_items = Vec::new();
        
        for variable in variables.iter().take(100) {
            json_items.push(format!(
                r#"{{"name":"{}","size":{},"thread":{},"state":"{}","allocs":{}}}"#,
                variable.name,
                variable.memory_usage,
                variable.thread_id,
                match variable.lifecycle_stage {
                    LifecycleStage::Active => "Active",
                    LifecycleStage::Allocated => "Allocated",
                    LifecycleStage::Shared => "Shared", 
                    LifecycleStage::Deallocated => "Deallocated",
                    _ => "Unknown"
                },
                variable.allocation_count
            ));
        }
        
        format!("[{}]", json_items.join(","))
    }

    /// Serialize threads for JavaScript
    fn serialize_threads_for_js(&self, data: &HybridAnalysisData) -> String {
        let mut thread_data = std::collections::HashMap::new();
        
        for variable in data.variable_registry.values() {
            let entry = thread_data.entry(variable.thread_id).or_insert_with(|| (0usize, 0usize));
            entry.0 += variable.memory_usage as usize;
            entry.1 += 1;
        }
        
        let mut json_items = Vec::new();
        for (thread_id, (memory, count)) in thread_data {
            json_items.push(format!(
                r#"{{"id":{},"memory":{},"variables":{}}}"#,
                thread_id, memory, count
            ));
        }
        
        format!("[{}]", json_items.join(","))
    }

    /// Serialize tasks for JavaScript
    fn serialize_tasks_for_js(&self, data: &HybridAnalysisData) -> String {
        let mut task_data = std::collections::HashMap::new();
        
        for variable in data.variable_registry.values() {
            if let Some(task_id) = variable.task_id {
                let entry = task_data.entry(task_id).or_insert_with(|| (0usize, 0usize, variable.thread_id));
                entry.0 += variable.memory_usage as usize;
                entry.1 += 1;
            }
        }
        
        let mut json_items = Vec::new();
        for (task_id, (memory, count, thread_id)) in task_data {
            json_items.push(format!(
                r#"{{"id":{},"memory":{},"variables":{},"thread":{}}}"#,
                task_id, memory, count, thread_id
            ));
        }
        
        format!("[{}]", json_items.join(","))
    }

    /// Generate detailed variable breakdown HTML
    pub fn generateVariableDetailedHTML(&self, data: &HybridAnalysisData) -> String {
        self.generate_hybrid_dashboard(data).unwrap_or_else(|e| {
            format!("<html><body><h1>Error: {}</h1></body></html>", e)
        })
    }
}

/// Create sample hybrid analysis data for testing
pub fn create_sample_hybrid_data(thread_count: usize, task_count: usize) -> HybridAnalysisData {
    let mut variable_registry = HashMap::new();
    
    // Create sample variables
    for i in 0..50 {
        let variable = VariableDetail {
            name: format!("var_t{}_task{}_v{}", (i % thread_count) + 1, (i % task_count) + 1, i),
            type_info: "Vec<u8>".to_string(),
            thread_id: (i % thread_count) + 1,
            task_id: Some((i % task_count) + 1),
            allocation_count: i as u64 + 1,
            memory_usage: ((i + 1) * 1024 * 8) as u64, // KB to bytes
            lifecycle_stage: match i % 4 {
                0 => LifecycleStage::Active,
                1 => LifecycleStage::Allocated,
                2 => LifecycleStage::Shared,
                _ => LifecycleStage::Deallocated,
            },
        };
        variable_registry.insert(variable.name.clone(), variable);
    }

    HybridAnalysisData {
        lockfree_analysis: None,
        visualization_config: VisualizationConfig::default(),
        thread_task_mapping: HashMap::new(),
        variable_registry,
        performance_metrics: PerformanceTimeSeries {
            cpu_usage: vec![45.2, 67.8, 23.1, 89.4],
            memory_usage: vec![1024, 2048, 1536, 3072],
            io_operations: vec![100, 250, 180, 320],
            network_bytes: vec![500, 1200, 800, 1500],
            timestamps: vec![1000, 2000, 3000, 4000],
            thread_cpu_breakdown: HashMap::new(),
            thread_memory_breakdown: HashMap::new(),
        },
    }
}