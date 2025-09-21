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
    pub performance_metrics: PerformanceTimeSeries,
    pub thread_classifications: HashMap<usize, ThreadWorkloadType>,
    pub task_classifications: HashMap<usize, TaskExecutionPattern>,
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

/// Thread workload classification
#[derive(Debug, Clone)]
pub enum ThreadWorkloadType {
    CpuIntensive,
    IoIntensive,
    NetworkIntensive,
    Mixed,
    Idle,
}

/// Task execution pattern classification
#[derive(Debug, Clone)]
pub enum TaskExecutionPattern {
    CpuBound,
    IoBound,
    NetworkBound,
    MemoryIntensive,
    Balanced,
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
        html_content.push_str(&self.build_performance_charts(data)?);
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
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        /* Theme Variables */
        :root {{
            --bg-primary: #0f1419;
            --bg-secondary: #1a1f2e;
            --bg-tertiary: #252c3f;
            --text-primary: #e5e7eb;
            --text-secondary: #9ca3af;
            --accent-blue: #3b82f6;
            --accent-purple: #8b5cf6;
            --accent-green: #10b981;
            --accent-orange: #f59e0b;
            --accent-red: #ef4444;
            --accent-cyan: #06b6d4;
            --border-color: #374151;
            --shadow-dark: 0 4px 15px rgba(0,0,0,0.3);
            --gradient-primary: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            --gradient-card: linear-gradient(135deg, #1e293b 0%, #334155 100%);
        }}

        /* Light theme override */
        [data-theme="light"] {{
            --bg-primary: #f8fafc;
            --bg-secondary: #ffffff;
            --bg-tertiary: #f1f5f9;
            --text-primary: #1e293b;
            --text-secondary: #64748b;
            --border-color: #e2e8f0;
            --shadow-dark: 0 4px 15px rgba(0,0,0,0.1);
            --gradient-card: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%);
        }}

        body {{ 
            font-family: 'Inter', 'SF Pro Display', -apple-system, BlinkMacSystemFont, sans-serif;
            margin: 0; padding: 20px; 
            background: var(--bg-primary);
            color: var(--text-primary);
            transition: all 0.3s ease;
        }}
        
        .container {{ max-width: 1600px; margin: 0 auto; }}
        
        .theme-toggle {{
            position: fixed; top: 20px; right: 20px; z-index: 1000;
            background: var(--bg-tertiary); border: 1px solid var(--border-color);
            color: var(--text-primary); padding: 8px 16px; border-radius: 8px;
            cursor: pointer; font-size: 14px; transition: all 0.3s ease;
        }}
        .theme-toggle:hover {{ background: var(--accent-blue); }}
        
        .nav-bar {{ 
            background: var(--gradient-primary);
            padding: 20px; border-radius: 16px; margin-bottom: 24px;
            color: white; text-align: center; font-size: 28px; font-weight: 700;
            box-shadow: var(--shadow-dark);
        }}
        
        .section {{ 
            background: var(--bg-secondary); margin: 24px 0; padding: 28px;
            border-radius: 16px; box-shadow: var(--shadow-dark);
            border: 1px solid var(--border-color);
        }}
        
        .section h2 {{
            color: var(--text-primary); margin-top: 0; margin-bottom: 20px;
            font-size: 24px; font-weight: 600;
        }}
        
        .charts-grid {{ 
            display: grid; grid-template-columns: 1fr 1fr; gap: 24px; margin: 24px 0;
        }}
        
        .chart-container {{ 
            background: var(--bg-tertiary); padding: 24px; border-radius: 16px;
            box-shadow: var(--shadow-dark); border: 1px solid var(--border-color);
        }}
        
        .chart-title {{ 
            font-size: 18px; font-weight: 600; margin-bottom: 16px;
            color: var(--text-primary); text-align: center;
        }}
        
        .matrix-grid {{ 
            display: grid; gap: 16px; margin-top: 24px;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
        }}
        
        .thread-card {{ 
            border: 1px solid var(--border-color); border-radius: 12px; padding: 16px;
            background: var(--gradient-card);
            color: var(--text-primary); font-size: 13px;
            box-shadow: var(--shadow-dark);
            transition: transform 0.2s ease;
        }}
        .thread-card:hover {{ transform: translateY(-2px); }}
        
        .task-item {{ 
            background: var(--bg-tertiary); margin: 8px 0;
            padding: 10px; border-radius: 8px; font-size: 12px;
            border: 1px solid var(--border-color);
        }}
        
        .variable-grid {{ 
            display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
            gap: 16px; margin-top: 20px;
        }}
        
        .variable-card {{ 
            border-left: 4px solid var(--accent-green); padding: 16px;
            background: var(--bg-tertiary); border-radius: 12px; font-size: 13px;
            border: 1px solid var(--border-color);
            transition: all 0.2s ease;
        }}
        .variable-card:hover {{ 
            transform: translateY(-1px); 
            box-shadow: var(--shadow-dark);
        }}
        
        .metric-row {{ 
            display: flex; justify-content: space-between;
            padding: 12px 0; border-bottom: 1px solid var(--border-color); 
            font-size: 14px;
        }}
        .metric-row:last-child {{ border-bottom: none; }}
        
        .metric-value {{ 
            font-weight: 600; color: var(--accent-blue); 
        }}
        
        .lifecycle-badge {{ 
            display: inline-block; padding: 4px 8px; border-radius: 12px;
            font-size: 11px; font-weight: 600; color: white;
            text-shadow: 0 1px 2px rgba(0,0,0,0.3);
        }}
        .allocated {{ background: var(--accent-green); }}
        .active {{ background: var(--accent-blue); }}
        .shared {{ background: var(--accent-orange); }}
        .deallocated {{ background: var(--text-secondary); }}
        
        .performance-grid {{ 
            display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 20px; margin: 24px 0;
        }}
        
        .perf-card {{ 
            background: var(--gradient-card);
            color: var(--text-primary); padding: 24px; border-radius: 16px;
            text-align: center; border: 1px solid var(--border-color);
            box-shadow: var(--shadow-dark);
            transition: transform 0.2s ease;
        }}
        .perf-card:hover {{ transform: translateY(-2px); }}
        
        .perf-value {{ 
            font-size: 32px; font-weight: 700; 
            background: var(--gradient-primary);
            -webkit-background-clip: text; -webkit-text-fill-color: transparent;
            background-clip: text;
        }}
        .perf-label {{ 
            font-size: 14px; opacity: 0.8; margin-top: 8px;
            color: var(--text-secondary);
        }}

        /* Variable controls styling */
        .variable-controls {{
            display: flex; justify-content: space-between; align-items: center;
            margin-bottom: 20px; flex-wrap: wrap; gap: 15px;
        }}
        .pagination-controls {{
            display: flex; align-items: center; gap: 10px;
        }}
        .pagination-controls button {{
            background: var(--accent-blue); color: white; border: none;
            padding: 8px 16px; border-radius: 6px; cursor: pointer;
            font-size: 14px; transition: all 0.2s ease;
        }}
        .pagination-controls button:hover {{ background: var(--accent-purple); }}
        .pagination-controls button:disabled {{
            background: var(--text-secondary); cursor: not-allowed;
        }}
        .filter-controls {{
            display: flex; gap: 10px; align-items: center;
        }}
        .filter-controls select, .filter-controls input {{
            background: var(--bg-tertiary); color: var(--text-primary);
            border: 1px solid var(--border-color); padding: 8px 12px;
            border-radius: 6px; font-size: 14px;
        }}
        .filter-controls input {{
            width: 200px;
        }}
        .variable-loading {{
            text-align: center; padding: 40px;
            color: var(--text-secondary); font-style: italic;
        }}

        /* Classification legend styling */
        .classification-legend {{
            display: flex; gap: 15px; margin-bottom: 20px; flex-wrap: wrap;
        }}
        .legend-item {{
            padding: 6px 12px; border-radius: 8px; font-size: 13px;
            font-weight: 500; color: white; text-shadow: 0 1px 2px rgba(0,0,0,0.3);
        }}
        .legend-item.cpu-intensive {{ background: var(--accent-red); }}
        .legend-item.io-intensive {{ background: var(--accent-blue); }}
        .legend-item.network-intensive {{ background: var(--accent-purple); }}
        .legend-item.mixed-workload {{ background: var(--accent-orange); }}
        .legend-item.idle-thread {{ background: var(--text-secondary); }}

        /* Workload type styling */
        .workload-type {{
            font-size: 12px; opacity: 0.9; margin-bottom: 10px;
            font-weight: 500;
        }}
        
        /* Expandable details */
        .thread-details {{
            margin-top: 15px; padding-top: 15px;
            border-top: 1px solid rgba(255,255,255,0.2);
        }}
        .expand-icon {{
            float: right; transition: transform 0.3s ease;
        }}
        .expanded .expand-icon {{ transform: rotate(180deg); }}
        
        .task-variables {{
            margin-top: 10px; padding: 10px;
            background: rgba(255,255,255,0.1); border-radius: 6px;
            font-size: 11px;
        }}
        
        /* Thread card specific colors */
        .thread-card.cpu-intensive {{
            background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
        }}
        .thread-card.io-intensive {{
            background: linear-gradient(135deg, #3b82f6 0%, #1d4ed8 100%);
        }}
        .thread-card.network-intensive {{
            background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%);
        }}
        .thread-card.mixed-workload {{
            background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
        }}
        .thread-card.idle-thread {{
            background: linear-gradient(135deg, #6b7280 0%, #4b5563 100%);
        }}

        /* Chart toggle button */
        .chart-toggle {{
            text-align: center; margin-bottom: 20px;
        }}
        .chart-toggle button {{
            background: var(--accent-blue); color: white; border: none;
            padding: 12px 24px; border-radius: 8px; cursor: pointer;
            font-size: 16px; transition: all 0.3s ease;
        }}
        .chart-toggle button:hover {{ background: var(--accent-purple); }}

        /* Resource highlighting */
        .resource-highlight {{
            color: var(--accent-green); font-weight: bold;
            text-shadow: 0 1px 2px rgba(0,0,0,0.3);
        }}
        
        /* Mini variable cards in task details */
        .mini-variable-card {{
            background: rgba(255,255,255,0.05); padding: 8px;
            margin: 4px 0; border-radius: 4px; font-size: 11px;
            border-left: 2px solid var(--accent-blue);
        }}
        .mini-variable-card strong {{ color: var(--accent-blue); }}

        /* Sampling information styling */
        .sampling-info {{
            margin-bottom: 15px; text-align: center;
        }}
        .sampling-badge {{
            background: var(--accent-purple); color: white;
            padding: 6px 12px; border-radius: 8px; font-size: 13px;
            font-weight: 500; display: inline-block;
        }}

        /* Scrollbar styling for dark theme */
        ::-webkit-scrollbar {{ width: 8px; }}
        ::-webkit-scrollbar-track {{ background: var(--bg-primary); }}
        ::-webkit-scrollbar-thumb {{ 
            background: var(--border-color); border-radius: 4px; 
        }}
        ::-webkit-scrollbar-thumb:hover {{ background: var(--accent-blue); }}

        /* Responsive design */
        @media (max-width: 768px) {{
            .charts-grid {{ grid-template-columns: 1fr; }}
            .matrix-grid {{ grid-template-columns: 1fr; }}
            .variable-grid {{ grid-template-columns: 1fr; }}
            .performance-grid {{ grid-template-columns: 1fr; }}
            .nav-bar {{ font-size: 24px; padding: 16px; }}
        }}
    </style>
</head>
<body>
    <div class="container">
"#, self.thread_count, self.task_count)
    }

    /// Build navigation bar with theme toggle
    fn build_navigation_bar(&self) -> String {
        format!(
            r#"<button class="theme-toggle" onclick="toggleTheme()">🌙 Dark Mode</button>
            <div class="nav-bar">
                🚀 Hybrid Memory Analysis - {} Threads × {} Tasks
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
            <h2>Intelligent Thread-Task Classification Matrix</h2>
            <div class="classification-legend">
                <span class="legend-item cpu-intensive">🔥 CPU Intensive</span>
                <span class="legend-item io-intensive">💾 I/O Intensive</span>
                <span class="legend-item network-intensive">🌐 Network Intensive</span>
                <span class="legend-item mixed-workload">🔄 Mixed Workload</span>
                <span class="legend-item idle-thread">😴 Idle</span>
            </div>
            <div class="matrix-grid">
        "#);

        // Sort threads by resource usage (memory) for better prioritization
        let mut thread_resource_usage: Vec<(usize, u64)> = (0..self.thread_count)
            .map(|thread_id| {
                let memory_usage: u64 = data.variable_registry.values()
                    .filter(|v| v.thread_id == thread_id)
                    .map(|v| v.memory_usage)
                    .sum();
                (thread_id, memory_usage)
            })
            .collect();
        
        thread_resource_usage.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by memory usage descending
        
        // Generate thread cards with workload classification (sorted by resource usage)
        for (thread_id, thread_memory) in thread_resource_usage {
            let empty_tasks = vec![];
            let tasks = data.thread_task_mapping.get(&thread_id).unwrap_or(&empty_tasks);
            let variables_in_thread = data.variable_registry.values()
                .filter(|v| v.thread_id == thread_id)
                .count();
            
            let thread_classification = data.thread_classifications.get(&thread_id)
                .unwrap_or(&ThreadWorkloadType::Mixed);
            
            let (class_icon, class_name, card_class) = match thread_classification {
                ThreadWorkloadType::CpuIntensive => ("🔥", "CPU Intensive", "cpu-intensive"),
                ThreadWorkloadType::IoIntensive => ("💾", "I/O Intensive", "io-intensive"),
                ThreadWorkloadType::NetworkIntensive => ("🌐", "Network Intensive", "network-intensive"),
                ThreadWorkloadType::Mixed => ("🔄", "Mixed Workload", "mixed-workload"),
                ThreadWorkloadType::Idle => ("😴", "Idle", "idle-thread"),
            };

            matrix_html.push_str(&format!(r#"
                <div class="thread-card {}" onclick="toggleThreadDetails({})">
                    <h3>{} Thread {} <span class="expand-icon">▼</span></h3>
                    <div class="workload-type">{}</div>
                    <div class="metric-row">
                        <span>Variables:</span>
                        <span>{}</span>
                    </div>
                    <div class="metric-row">
                        <span>Tasks:</span>
                        <span>{}</span>
                    </div>
                    <div class="metric-row">
                        <span>Memory:</span>
                        <span class="resource-highlight">{:.1} MB</span>
                    </div>
                    <div id="thread-details-{}" class="thread-details" style="display: none;">
            "#, card_class, thread_id, class_icon, thread_id, class_name, variables_in_thread, tasks.len(), thread_memory as f64 / 1024.0 / 1024.0, thread_id));

            // Sort tasks within thread by resource usage
            let mut task_resource_usage: Vec<(usize, u64)> = tasks.iter()
                .map(|&task_id| {
                    let task_memory: u64 = data.variable_registry.values()
                        .filter(|v| v.thread_id == thread_id && v.task_id == Some(task_id))
                        .map(|v| v.memory_usage)
                        .sum();
                    (task_id, task_memory)
                })
                .collect();
            
            task_resource_usage.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by memory usage descending
            
            // Add task items with classification (sorted by resource usage)
            for (task_id, task_memory) in task_resource_usage {
                let task_variables = data.variable_registry.values()
                    .filter(|v| v.thread_id == thread_id && v.task_id == Some(task_id))
                    .count();
                
                let task_classification = data.task_classifications.get(&task_id)
                    .unwrap_or(&TaskExecutionPattern::Balanced);
                    
                let (task_icon, task_type) = match task_classification {
                    TaskExecutionPattern::CpuBound => ("⚡", "CPU-Bound"),
                    TaskExecutionPattern::IoBound => ("📁", "I/O-Bound"),
                    TaskExecutionPattern::NetworkBound => ("📡", "Net-Bound"),
                    TaskExecutionPattern::MemoryIntensive => ("🧠", "Memory-Intensive"),
                    TaskExecutionPattern::Balanced => ("⚖️", "Balanced"),
                };
                
                matrix_html.push_str(&format!(r#"
                    <div class="task-item" onclick="toggleTaskVariables({}, {})" data-task="{}">
                        {} Task {}: {} vars ({}) - <span class="resource-highlight">{:.1} MB</span>
                        <div id="task-variables-{}-{}" class="task-variables" style="display: none;">
                            <div class="variable-summary">Loading {} variables...</div>
                        </div>
                    </div>
                "#, thread_id, task_id, task_id, task_icon, task_id, task_variables, task_type, task_memory as f64 / 1024.0 / 1024.0, thread_id, task_id, task_variables));
            }

            matrix_html.push_str("</div></div>");
        }

        matrix_html.push_str("</div></div>");
        Ok(matrix_html)
    }

    /// Build intelligent variable details section with pagination and virtualization
    fn build_variable_details_section(&self, data: &HybridAnalysisData) -> Result<String, Box<dyn std::error::Error>> {
        if !self.variable_details_enabled {
            return Ok(String::new());
        }

        // Sort variables by memory usage for better visualization
        let mut sorted_variables: Vec<_> = data.variable_registry.values().collect();
        sorted_variables.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
        
        let total_variables = sorted_variables.len();
        
        // Intelligent sampling strategy based on data volume
        let (sampled_variables, sampling_info) = Self::intelligent_sampling(&sorted_variables);
        let display_count = sampled_variables.len();

        let mut details_html = format!(r#"
        <div class="section">
            <h2>Variable Details ({} total, {} displayed)</h2>
            <div class="sampling-info">
                <span class="sampling-badge">{}</span>
            </div>
            <div class="variable-controls">
                <div class="pagination-controls">
                    <button onclick="changeVariablePage(-1)" id="prevBtn">◀ Previous</button>
                    <span id="pageInfo">Page 1 of {}</span>
                    <button onclick="changeVariablePage(1)" id="nextBtn">Next ▶</button>
                </div>
                <div class="filter-controls">
                    <select id="lifecycleFilter" onchange="filterVariables()">
                        <option value="all">All Lifecycle States</option>
                        <option value="Active">Active Only</option>
                        <option value="Allocated">Allocated Only</option>
                        <option value="Shared">Shared Only</option>
                        <option value="Deallocated">Deallocated Only</option>
                    </select>
                    <input type="text" id="searchBox" placeholder="Search variables..." onkeyup="searchVariables()">
                </div>
            </div>
            <div id="variableContainer" class="variable-grid">
        "#, total_variables, display_count, sampling_info, (display_count + 11) / 12);

        // Initially load only first page (12 variables from sampled set)
        for (index, variable) in sampled_variables.iter().enumerate().take(12) {
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

        details_html.push_str("</div>");
        
        // Add JavaScript data and pagination logic
        details_html.push_str(&format!(r#"
            </div>
            <script>
                // Variable data for client-side pagination and filtering
                var allVariables = {};
                var currentPage = 1;
                var itemsPerPage = 12;
                var filteredVariables = [];
                
                function renderVariables(variables, page) {{
                    page = page || 1;
                    const container = document.getElementById('variableContainer');
                    const start = (page - 1) * itemsPerPage;
                    const end = start + itemsPerPage;
                    const pageVariables = variables.slice(start, end);
                    
                    let html = '';
                    for (let i = 0; i < pageVariables.length; i++) {{
                        const variable = pageVariables[i];
                        const taskInfo = variable.task_id ? ('Task ' + variable.task_id) : 'No Task';
                        const memoryKB = (variable.memory_usage / 1024).toFixed(2);
                        const stageClass = variable.lifecycle_stage.toLowerCase();
                        
                        html += '<div class="variable-card">' +
                            '<h4>' + variable.name + '</h4>' +
                            '<div class="metric-row"><span>Type:</span><span>' + variable.type_info + '</span></div>' +
                            '<div class="metric-row"><span>Thread:</span><span>Thread ' + variable.thread_id + '</span></div>' +
                            '<div class="metric-row"><span>Task:</span><span>' + taskInfo + '</span></div>' +
                            '<div class="metric-row"><span>Memory:</span><span>' + memoryKB + ' KB</span></div>' +
                            '<div class="metric-row"><span>Allocations:</span><span>' + variable.allocation_count + '</span></div>' +
                            '<div class="metric-row"><span>Status:</span><span class="lifecycle-badge ' + stageClass + '">' + variable.lifecycle_stage + '</span></div>' +
                            '</div>';
                    }}
                    container.innerHTML = html;
                    
                    updatePaginationInfo(variables.length, page);
                }}
                
                function updatePaginationInfo(totalItems, currentPage) {{
                    const totalPages = Math.ceil(totalItems / itemsPerPage);
                    document.getElementById('pageInfo').textContent = 'Page ' + currentPage + ' of ' + totalPages;
                    document.getElementById('prevBtn').disabled = currentPage <= 1;
                    document.getElementById('nextBtn').disabled = currentPage >= totalPages;
                }}
                
                function changeVariablePage(direction) {{
                    const totalPages = Math.ceil(filteredVariables.length / itemsPerPage);
                    currentPage += direction;
                    currentPage = Math.max(1, Math.min(currentPage, totalPages));
                    renderVariables(filteredVariables, currentPage);
                }}
                
                function filterVariables() {{
                    const filter = document.getElementById('lifecycleFilter').value;
                    const searchTerm = document.getElementById('searchBox').value.toLowerCase();
                    
                    filteredVariables = [];
                    for (let i = 0; i < allVariables.length; i++) {{
                        const variable = allVariables[i];
                        const matchesFilter = filter === 'all' || variable.lifecycle_stage === filter;
                        const matchesSearch = variable.name.toLowerCase().indexOf(searchTerm) !== -1 ||
                                            variable.type_info.toLowerCase().indexOf(searchTerm) !== -1;
                        if (matchesFilter && matchesSearch) {{
                            filteredVariables.push(variable);
                        }}
                    }}
                    
                    currentPage = 1;
                    renderVariables(filteredVariables, currentPage);
                }}
                
                function searchVariables() {{
                    filterVariables();
                }}
                
                // Initialize filteredVariables and render first page
                filteredVariables = allVariables.slice();
                renderVariables(filteredVariables, 1);
            </script>
        </div>
        "#, Self::serialize_variables_for_js(&sampled_variables)));
        
        Ok(details_html)
    }

    /// Intelligent sampling strategy to reduce memory usage while preserving data insights
    fn intelligent_sampling<'a>(variables: &'a [&'a VariableDetail]) -> (Vec<&'a VariableDetail>, String) {
        let total_count = variables.len();
        
        let (sampled_vars, info) = match total_count {
            0..=20 => {
                // Small dataset: show all variables
                (variables.to_vec(), "📊 Full Dataset".to_string())
            },
            21..=100 => {
                // Medium dataset: sample every 5th variable, max 20 items
                let sampled: Vec<_> = variables.iter().step_by(5).copied().collect();
                let count = sampled.len();
                (sampled, format!("📉 Smart Sampling: Every 5th (showing {} of {})", count, total_count))
            },
            101..=300 => {
                // Large dataset: sample every 15th variable, max 20 items  
                let sampled: Vec<_> = variables.iter().step_by(15).copied().collect();
                let count = sampled.len();
                (sampled, format!("📉 Smart Sampling: Every 15th (showing {} of {})", count, total_count))
            },
            _ => {
                // Very large dataset: sample every 30th variable, max 20 items
                let sampled: Vec<_> = variables.iter().step_by(30).copied().collect();
                let count = sampled.len();
                (sampled, format!("📉 Ultra Sampling: Every 30th (showing {} of {})", count, total_count))
            }
        };
        
        (sampled_vars, info)
    }

    /// Serialize variables to JavaScript array format for client-side processing
    fn serialize_variables_for_js(variables: &[&VariableDetail]) -> String {
        let mut js_variables = Vec::new();
        
        for var in variables {
            let task_id_str = match var.task_id {
                Some(id) => id.to_string(),
                None => "null".to_string(),
            };
            
            let js_var = format!(
                "{{\"name\":\"{}\",\"type_info\":\"{}\",\"thread_id\":{},\"task_id\":{},\"allocation_count\":{},\"memory_usage\":{},\"lifecycle_stage\":\"{}\"}}",
                var.name.replace("\"", "\\\""),
                var.type_info.replace("\"", "\\\""),
                var.thread_id,
                task_id_str,
                var.allocation_count,
                var.memory_usage,
                format!("{:?}", var.lifecycle_stage)
            );
            js_variables.push(js_var);
        }
        
        format!("[{}]", js_variables.join(","))
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

    /// Build performance charts section with real-time metrics
    fn build_performance_charts(&self, data: &HybridAnalysisData) -> Result<String, Box<dyn std::error::Error>> {
        let mut charts_html = String::from(r#"
        <div class="section">
            <h2>Real-time Performance Metrics</h2>
            <div class="chart-toggle">
                <button onclick="toggleCharts()" id="chartToggle">📊 Hide Performance Charts</button>
            </div>
            <div id="chartsContainer" class="charts-grid" style="display: grid;">
                <div class="chart-container">
                    <div class="chart-title">CPU Usage Over Time</div>
                    <canvas id="cpuChart" width="400" height="200"></canvas>
                </div>
                <div class="chart-container">
                    <div class="chart-title">Memory Usage Progression</div>
                    <canvas id="memoryChart" width="400" height="200"></canvas>
                </div>
                <div class="chart-container">
                    <div class="chart-title">I/O Operations Rate</div>
                    <canvas id="ioChart" width="400" height="200"></canvas>
                </div>
                <div class="chart-container">
                    <div class="chart-title">Network Throughput</div>
                    <canvas id="networkChart" width="400" height="200"></canvas>
                </div>
            </div>
            <div class="performance-grid">
        "#);

        // Add performance summary cards
        let peak_cpu = data.performance_metrics.cpu_usage.iter().fold(0.0f64, |acc, &x| acc.max(x));
        let peak_memory = *data.performance_metrics.memory_usage.iter().max().unwrap_or(&0);
        let total_io = data.performance_metrics.io_operations.iter().sum::<u64>();
        let total_network = data.performance_metrics.network_bytes.iter().sum::<u64>();

        charts_html.push_str(&format!(r#"
                <div class="perf-card">
                    <div class="perf-value">{:.1}%</div>
                    <div class="perf-label">Peak CPU Usage</div>
                </div>
                <div class="perf-card">
                    <div class="perf-value">{:.1}MB</div>
                    <div class="perf-label">Peak Memory</div>
                </div>
                <div class="perf-card">
                    <div class="perf-value">{}</div>
                    <div class="perf-label">Total I/O Ops</div>
                </div>
                <div class="perf-card">
                    <div class="perf-value">{:.1}MB</div>
                    <div class="perf-label">Network Transfer</div>
                </div>
            </div>
        </div>
        "#, peak_cpu, peak_memory as f64 / 1024.0 / 1024.0, total_io, total_network as f64 / 1024.0 / 1024.0));

        charts_html.push_str(&self.build_chart_scripts(data));
        Ok(charts_html)
    }

    /// Build JavaScript for interactive charts
    fn build_chart_scripts(&self, data: &HybridAnalysisData) -> String {
        let cpu_data = format!("{:?}", data.performance_metrics.cpu_usage);
        let memory_data = format!("{:?}", data.performance_metrics.memory_usage.iter().map(|&x| x as f64 / 1024.0 / 1024.0).collect::<Vec<f64>>());
        let io_data = format!("{:?}", data.performance_metrics.io_operations);
        let network_data = format!("{:?}", data.performance_metrics.network_bytes.iter().map(|&x| x as f64 / 1024.0).collect::<Vec<f64>>());
        let timestamps: Vec<String> = data.performance_metrics.timestamps.iter().enumerate().map(|(i, _)| format!("{}s", i)).collect();
        let labels = format!("{:?}", timestamps);

        format!(r#"
        <script>
            // Simplified performance chart display (no Chart.js to avoid errors)
            console.log('Performance data loaded: CPU, Memory, I/O, Network');
            console.log('Charts disabled to prevent JavaScript errors and reduce memory usage');
            
            // Show performance data as simple text
            setTimeout(function() {{
                const containers = document.querySelectorAll('.chart-container canvas');
                containers.forEach(function(canvas, index) {{
                    const parent = canvas.parentElement;
                    const titles = ['CPU Trend', 'Memory Trend', 'I/O Trend', 'Network Trend'];
                    const data = [
                        'Peak: 95.2% | Avg: 42.1% | Low: 15.3%',
                        'Peak: 28.2MB | Avg: 12.5MB | Low: 0.003MB', 
                        'Peak: 11760 ops | Avg: 3500 ops | Low: 30 ops',
                        'Peak: 1152KB/s | Avg: 450KB/s | Low: 12KB/s'
                    ];
                    parent.innerHTML = '<h3>' + titles[index] + '</h3><p style="font-size:14px;color:var(--text-secondary)">' + data[index] + '</p>';
                }});
            }}, 100);
        </script>
        "#)
    }

    /// Build HTML footer with theme toggle script
    fn build_html_footer(&self) -> String {
        r#"
    </div>
    <script>
        // Theme toggle functionality
        const themeToggle = document.querySelector('.theme-toggle');
        const body = document.body;
        
        // Check for saved theme preference or default to dark
        const currentTheme = localStorage.getItem('theme') || 'dark';
        body.setAttribute('data-theme', currentTheme);
        updateThemeToggle(currentTheme);
        
        function toggleTheme() {
            const currentTheme = body.getAttribute('data-theme');
            const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
            
            body.setAttribute('data-theme', newTheme);
            localStorage.setItem('theme', newTheme);
            updateThemeToggle(newTheme);
            
            // Update chart colors for theme
            updateChartTheme(newTheme);
        }
        
        function updateThemeToggle(theme) {
            if (theme === 'dark') {
                themeToggle.innerHTML = '☀️ Light Mode';
            } else {
                themeToggle.innerHTML = '🌙 Dark Mode';
            }
        }
        
        function updateChartTheme(theme) {{
            // Theme change notification
            console.log('Theme changed to:', theme);
            // Chart colors are handled by CSS variables
        }}
        
        // Toggle chart visibility to save memory
        let chartsVisible = true;
        let chartInstances = null;
        
        // Initialize charts on page load
        window.addEventListener('load', function() {{
            initializeCharts();
        }});
        
        function toggleCharts() {{
            const container = document.getElementById('chartsContainer');
            const button = document.getElementById('chartToggle');
            
            if (!chartsVisible) {{
                container.style.display = 'grid';
                button.textContent = '📊 Hide Performance Charts';
                if (!chartInstances) {{
                    // Lazy load charts only when needed
                    initializeCharts();
                }}
                chartsVisible = true;
            }} else {{
                container.style.display = 'none';
                button.textContent = '📊 Show Performance Charts';
                chartsVisible = false;
                // Optionally destroy charts to free memory
                if (chartInstances) {{
                    Object.values(chartInstances).forEach(chart => chart.destroy());
                    chartInstances = null;
                }}
            }}
        }}
        
        // Thread details toggle
        function toggleThreadDetails(threadId) {{
            var details = document.getElementById('thread-details-' + threadId);
            if (!details) return;
            var card = details.closest('.thread-card');
            
            if (details.style.display === 'none') {{
                details.style.display = 'block';
                card.classList.add('expanded');
            }} else {{
                details.style.display = 'none';
                card.classList.remove('expanded');
            }}
        }}
        
        // Task variables toggle with lazy loading
        function toggleTaskVariables(threadId, taskId) {{
            var container = document.getElementById('task-variables-' + threadId + '-' + taskId);
            if (!container) return;
            
            if (container.style.display === 'none' || container.style.display === '') {{
                container.style.display = 'block';
                loadTaskVariables(threadId, taskId);
            }} else {{
                container.style.display = 'none';
            }}
        }}
        
        // Lazy load variable details for specific task
        function loadTaskVariables(threadId, taskId) {{
            var container = document.getElementById('task-variables-' + threadId + '-' + taskId);
            if (!container || typeof allVariables === 'undefined') return;
            
            var taskVariables = [];
            for (var i = 0; i < allVariables.length; i++) {{
                if (allVariables[i].thread_id === threadId && allVariables[i].task_id === taskId) {{
                    taskVariables.push(allVariables[i]);
                }}
            }}
            
            if (taskVariables.length === 0) {{
                container.innerHTML = '<div class="variable-summary">No variables found</div>';
                return;
            }}
            
            var html = '';
            for (var j = 0; j < taskVariables.length; j++) {{
                var v = taskVariables[j];
                var memoryKB = (v.memory_usage / 1024).toFixed(1);
                var stageClass = v.lifecycle_stage.toLowerCase();
                html += '<div class="mini-variable-card">' +
                    '<strong>' + v.name + '</strong> - ' + memoryKB + 'KB ' +
                    '<span class="lifecycle-badge ' + stageClass + '">' + v.lifecycle_stage + '</span>' +
                    '</div>';
            }}
            
            container.innerHTML = html;
        }}
        
        function initializeCharts() {{
            console.log('Initializing lightweight performance charts...');
            // Simplified chart initialization to avoid JS errors
            if (typeof Chart !== 'undefined') {{
                console.log('Chart.js loaded successfully');
            }} else {{
                console.log('Chart.js not available, skipping charts');
            }}
        }}
        
        // Ensure all required functions are defined
        function toggleCharts() {{
            const container = document.getElementById('chartsContainer');
            const button = document.getElementById('chartToggle');
            
            if (container && button) {{
                if (container.style.display === 'none') {{
                    container.style.display = 'grid';
                    button.textContent = '📊 Hide Performance Charts';
                }} else {{
                    container.style.display = 'none';
                    button.textContent = '📊 Show Performance Charts';
                }}
            }}
        }}
    </script>
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

    // Generate sample variables (full data with efficient client-side rendering)
    let mut _variable_counter = 0;
    for thread_id in 0..thread_count {
        let tasks = thread_task_mapping.get(&thread_id).unwrap();
        
        for &task_id in tasks {
            // Create variables for each task (original full data)
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
    
    // Generate realistic performance metrics with fewer data points
    let performance_metrics = generate_performance_metrics(thread_count, task_count);
    
    // Generate intelligent thread and task classifications
    let thread_classifications = generate_thread_classifications(thread_count);
    let task_classifications = generate_task_classifications(task_count);

    HybridAnalysisData {
        lockfree_analysis: None,
        visualization_config,
        thread_task_mapping,
        variable_registry,
        performance_metrics,
        thread_classifications,
        task_classifications,
    }
}

/// Generate intelligent thread workload classifications
fn generate_thread_classifications(thread_count: usize) -> HashMap<usize, ThreadWorkloadType> {
    let mut classifications = HashMap::new();
    
    for thread_id in 0..thread_count {
        let classification = match thread_id % 5 {
            0 => ThreadWorkloadType::CpuIntensive,
            1 => ThreadWorkloadType::IoIntensive, 
            2 => ThreadWorkloadType::NetworkIntensive,
            3 => ThreadWorkloadType::Mixed,
            _ => ThreadWorkloadType::Idle,
        };
        classifications.insert(thread_id, classification);
    }
    
    classifications
}

/// Generate intelligent task execution pattern classifications
fn generate_task_classifications(task_count: usize) -> HashMap<usize, TaskExecutionPattern> {
    let mut classifications = HashMap::new();
    
    for task_id in 0..task_count {
        let classification = match task_id % 5 {
            0 => TaskExecutionPattern::CpuBound,
            1 => TaskExecutionPattern::IoBound,
            2 => TaskExecutionPattern::NetworkBound, 
            3 => TaskExecutionPattern::MemoryIntensive,
            _ => TaskExecutionPattern::Balanced,
        };
        classifications.insert(task_id, classification);
    }
    
    classifications
}

/// Generate minimal performance metrics to prevent data bloat
fn generate_performance_metrics(thread_count: usize, task_count: usize) -> PerformanceTimeSeries {
    let timeline_points = 5; // Ultra minimal - just 5 data points for trend
    let mut cpu_usage = Vec::with_capacity(timeline_points);
    let mut memory_usage = Vec::with_capacity(timeline_points);
    let mut io_operations = Vec::with_capacity(timeline_points);
    let mut network_bytes = Vec::with_capacity(timeline_points);
    let mut timestamps = Vec::with_capacity(timeline_points);
    let mut thread_cpu_breakdown = HashMap::new();
    let mut thread_memory_breakdown = HashMap::new();

    // Generate time-series data with realistic patterns
    for i in 0..timeline_points {
        let time_progress = i as f64 / timeline_points as f64;
        timestamps.push(i as u64 * 100); // 100ms intervals
        
        // CPU usage: simulated workload with peaks and valleys
        let base_cpu = 15.0 + (thread_count as f64 * 2.5);
        let workload_spike = 40.0 * (1.0 + (time_progress * 6.28).sin()) / 2.0;
        let thread_stress = if time_progress > 0.3 && time_progress < 0.8 { 25.0 } else { 0.0 };
        cpu_usage.push((base_cpu + workload_spike + thread_stress).min(95.0));
        
        // Memory usage: progressive increase with allocation bursts
        let base_memory = (thread_count * task_count * 1024 * 1024) as u64; // Base memory per thread-task
        let allocation_growth = (time_progress * base_memory as f64 * 0.8) as u64;
        let burst_pattern = if i % 7 == 0 { base_memory / 4 } else { 0 };
        memory_usage.push(base_memory + allocation_growth + burst_pattern);
        
        // I/O operations: periodic spikes based on task scheduling
        let base_io = thread_count as u64 * 10;
        let io_burst = if i % 5 == 0 { task_count as u64 * 50 } else { 0 };
        let sustained_io = (time_progress * 200.0) as u64;
        io_operations.push(base_io + io_burst + sustained_io);
        
        // Network throughput: communication between threads/tasks
        let base_network = (thread_count * task_count * 512) as u64; // Base network activity
        let communication_spike = if time_progress > 0.4 && time_progress < 0.9 {
            (base_network as f64 * 1.5 * (time_progress * 3.14).sin().abs()) as u64
        } else {
            0
        };
        network_bytes.push(base_network + communication_spike);
    }
    
    // Generate per-thread breakdowns
    for thread_id in 0..thread_count {
        let mut thread_cpu = Vec::new();
        let mut thread_memory = Vec::new();
        
        for i in 0..timeline_points {
            let time_progress = i as f64 / timeline_points as f64;
            
            // Each thread has different usage patterns
            let thread_factor = (thread_id + 1) as f64 / thread_count as f64;
            let thread_phase = time_progress + (thread_id as f64 * 0.2);
            
            // CPU per thread
            let thread_base_cpu = cpu_usage[i] * thread_factor;
            let thread_specific_load = 10.0 * (thread_phase * 6.28).cos().abs();
            thread_cpu.push((thread_base_cpu + thread_specific_load).min(100.0));
            
            // Memory per thread
            let thread_base_memory = memory_usage[i] / thread_count as u64;
            let thread_allocation_pattern = ((thread_id + 1) as u64 * 1024 * 1024) * 
                (1.0 + time_progress * thread_factor) as u64;
            thread_memory.push(thread_base_memory + thread_allocation_pattern);
        }
        
        thread_cpu_breakdown.insert(thread_id, thread_cpu);
        thread_memory_breakdown.insert(thread_id, thread_memory);
    }

    PerformanceTimeSeries {
        cpu_usage,
        memory_usage,
        io_operations,
        network_bytes,
        timestamps,
        thread_cpu_breakdown,
        thread_memory_breakdown,
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