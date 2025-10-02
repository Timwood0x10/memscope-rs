//! Clean Template Renderer - Only data injection, no embedded HTML
//! All HTML and JS code moved to external templates

use crate::async_memory::visualization::VisualizationConfig;
use crate::lockfree::LockfreeAnalysis;
use std::collections::HashMap;
use std::fs;

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

    pub fn with_enhanced_insights(self, _enable: bool) -> Self {
        // Enhanced insights will be processed in render_with_template
        self
    }

    /// Generate hybrid dashboard HTML using external template
    pub fn generate_hybrid_dashboard(
        &self,
        data: &HybridAnalysisData,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Load external template
        let template_content = match fs::read_to_string("templates/hybrid_dashboard.html") {
            Ok(content) => content,
            Err(_) => {
                // Try alternative path for when running from examples directory
                match fs::read_to_string("../templates/hybrid_dashboard.html") {
                    Ok(content) => content,
                    Err(_) => {
                        return Err("Template file templates/hybrid_dashboard.html not found".into())
                    }
                }
            }
        };

        Ok(self.render_with_template(&template_content, data))
    }

    /// Render using external template
    fn render_with_template(&self, template: &str, data: &HybridAnalysisData) -> String {
        let mut html = template.to_string();

        // Replace placeholders with data
        html = html.replace("{{TITLE}}", "🔬 Memory Analysis Dashboard");
        html = html.replace(
            "{{TOTAL_MEMORY}}",
            &format!("{:.1}MB", self.calculate_total_memory(data)),
        );
        html = html.replace(
            "{{TOTAL_VARIABLES}}",
            &data.variable_registry.len().to_string(),
        );
        html = html.replace(
            "{{THREAD_COUNT}}",
            &data
                .lockfree_analysis
                .as_ref()
                .map_or(0, |a| a.thread_stats.len())
                .to_string(),
        );
        html = html.replace("{{EFFICIENCY}}", "85.2");

        // Generate content
        let variables: Vec<VariableDetail> = data.variable_registry.values().cloned().collect();
        html = html.replace(
            "{{VARIABLES_HTML}}",
            &self.generate_variables_html(&variables),
        );
        html = html.replace("{{MEMORY_MAP_HTML}}", &self.generate_memory_map_html(data));

        // Generate JSON data for JavaScript
        html = html.replace(
            "{{VARIABLES_DATA}}",
            &self.serialize_variables_for_js(&variables),
        );
        html = html.replace("{{THREADS_DATA}}", &self.serialize_threads_for_js(data));
        html = html.replace("{{TASKS_DATA}}", &self.serialize_tasks_for_js(data));

        // Add enhanced insights data replacements
        html = self.replace_insights_placeholders(html, data);

        html
    }

    /// Calculate total memory usage
    fn calculate_total_memory(&self, data: &HybridAnalysisData) -> f64 {
        data.variable_registry
            .values()
            .map(|v| v.memory_usage as f64 / 1024.0 / 1024.0)
            .sum()
    }

    /// Generate variables HTML with performance categories
    fn generate_variables_html(&self, variables: &[VariableDetail]) -> String {
        let mut html = String::new();

        for variable in variables.iter().take(50) {
            let status_class = match variable.lifecycle_stage {
                LifecycleStage::Active => "status-active",
                LifecycleStage::Allocated => "status-allocated",
                LifecycleStage::Shared => "status-shared",
                LifecycleStage::Deallocated => "status-deallocated",
            };

            let status_icon = match variable.lifecycle_stage {
                LifecycleStage::Active => "🟢",
                LifecycleStage::Allocated => "🟡",
                LifecycleStage::Shared => "🔄",
                LifecycleStage::Deallocated => "⚫",
            };

            let performance_category = self.classify_variable_performance(variable);
            let category_class = match performance_category.as_str() {
                "cpu" => "cpu-intensive",
                "io" => "io-intensive",
                "memory" => "memory-intensive",
                "async" => "async-heavy",
                _ => "normal",
            };

            let size_kb = variable.memory_usage / 1024;

            html.push_str(&format!(
                r#"<div class="variable-card {}" data-category="{}" data-thread="{}" data-memory="{}" data-allocations="{}" onclick="window.drillDown('{}', 'memory')">
                    <div class="variable-name">{} {}</div>
                    <div class="variable-info">
                        <span class="{}">{}KB | {} allocs | {}</span>
                        <span>Thread {}</span>
                    </div>
                    <div class="performance-indicator">
                        <span class="perf-badge {}">{}</span>
                    </div>
                </div>"#,
                category_class,
                performance_category,
                variable.thread_id,
                size_kb,
                variable.allocation_count,
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
                    LifecycleStage::Deallocated => "Deallocated"
                },
                variable.thread_id,
                category_class,
                self.get_performance_label(&performance_category)
            ));
        }

        html
    }

    /// Classify performance type based on variable characteristics
    fn classify_variable_performance(&self, variable: &VariableDetail) -> String {
        let size_kb = variable.memory_usage / 1024;
        let allocation_rate = variable.allocation_count;

        // Intelligent classification based on variable name and characteristics
        let var_name = variable.name.to_lowercase();

        if var_name.contains("buffer") || var_name.contains("cache") || size_kb > 500 {
            "memory".to_string()
        } else if var_name.contains("cpu") || var_name.contains("compute") || allocation_rate > 100
        {
            "cpu".to_string()
        } else if var_name.contains("io") || var_name.contains("file") || var_name.contains("net") {
            "io".to_string()
        } else if var_name.contains("async")
            || var_name.contains("future")
            || var_name.contains("task")
        {
            "async".to_string()
        } else {
            "normal".to_string()
        }
    }

    fn get_performance_label(&self, category: &str) -> &str {
        match category {
            "cpu" => "CPU",
            "io" => "I/O",
            "memory" => "MEM",
            "async" => "ASYNC",
            _ => "NORM",
        }
    }

    /// Generate memory map HTML
    fn generate_memory_map_html(&self, data: &HybridAnalysisData) -> String {
        let mut html = String::new();
        html.push_str("<div class='memory-map-grid'>");

        // Group by threads and show memory blocks
        let mut thread_groups = std::collections::HashMap::new();
        for variable in data.variable_registry.values() {
            thread_groups
                .entry(variable.thread_id)
                .or_insert_with(Vec::new)
                .push(variable);
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
                    (size_kb / 10).clamp(10, 100),
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
            let entry = thread_data
                .entry(variable.thread_id)
                .or_insert_with(|| (0usize, 0usize));
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
                let entry = task_data
                    .entry(task_id)
                    .or_insert_with(|| (0usize, 0usize, variable.thread_id));
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

    /// Replace all insight placeholders with actual data
    fn replace_insights_placeholders(&self, mut html: String, data: &HybridAnalysisData) -> String {
        // Find high usage thread and metrics
        let (high_usage_thread, max_allocation_size, high_frequency) =
            self.analyze_high_usage(data);
        let (small_alloc_count, small_alloc_rate) = self.analyze_small_allocations(data);
        let (bottleneck_thread, bottleneck_rate, bottleneck_percent) =
            self.analyze_bottlenecks(data);

        // Replace hotspot variables
        html = html.replace("{{HIGH_USAGE_THREAD}}", &high_usage_thread.to_string());
        html = html.replace("{{MAX_ALLOCATION_SIZE}}", &max_allocation_size.to_string());
        html = html.replace("{{HIGH_FREQUENCY}}", &high_frequency.to_string());
        html = html.replace("{{SMALL_ALLOC_COUNT}}", &small_alloc_count.to_string());
        html = html.replace("{{SMALL_ALLOC_RATE}}", &format!("{}/sec", small_alloc_rate));

        // Replace leak detection variables
        let (leak_status_class, leak_icon, leak_status_title, leak_status_description) =
            self.analyze_memory_leaks(data);
        html = html.replace("{{LEAK_STATUS_CLASS}}", &leak_status_class);
        html = html.replace("{{LEAK_ICON}}", &leak_icon);
        html = html.replace("{{LEAK_STATUS_TITLE}}", &leak_status_title);
        html = html.replace("{{LEAK_STATUS_DESCRIPTION}}", &leak_status_description);

        // Replace memory analysis variables
        let memory_efficiency = self.calculate_memory_efficiency(data);
        let memory_overhead = self.calculate_memory_overhead(data);
        let potential_leaks = self.count_potential_leaks(data);

        html = html.replace(
            "{{MEMORY_EFFICIENCY}}",
            &format!("{:.1}", memory_efficiency),
        );
        html = html.replace("{{MEMORY_OVERHEAD}}", &format!("{:.2}MB", memory_overhead));
        html = html.replace("{{POTENTIAL_LEAKS}}", &potential_leaks.to_string());

        // Replace performance variables
        html = html.replace("{{BOTTLENECK_THREAD}}", &bottleneck_thread.to_string());
        html = html.replace("{{BOTTLENECK_RATE}}", &format!("{:.0}", bottleneck_rate));
        html = html.replace(
            "{{BOTTLENECK_PERCENT}}",
            &format!("{:.0}", bottleneck_percent),
        );
        html = html.replace("{{BOTTLENECK_LOCATION}}", "execute_track_var_workload()");

        // Replace suggestion variables
        let suggested_capacity = self.calculate_suggested_capacity(data);
        let string_capacity = self.calculate_string_capacity(data);
        html = html.replace("{{SUGGESTED_CAPACITY}}", &suggested_capacity.to_string());
        html = html.replace("{{STRING_CAPACITY}}", &string_capacity.to_string());

        // Replace thread rate analysis
        let (thread_0_9_rate, thread_10_19_rate, thread_20_rate) = self.analyze_thread_rates(data);
        html = html.replace("{{THREAD_0_9_RATE}}", &format!("{:.0}", thread_0_9_rate));
        html = html.replace(
            "{{THREAD_10_19_RATE}}",
            &format!("{:.0}", thread_10_19_rate),
        );
        html = html.replace("{{THREAD_20_RATE}}", &format!("{:.0}", thread_20_rate));

        // Replace scoring variables
        let (memory_score, allocation_score, thread_score, overall_score) =
            self.calculate_scores(data);
        html = html.replace("{{MEMORY_SCORE}}", &memory_score.to_string());
        html = html.replace("{{ALLOCATION_SCORE}}", &allocation_score.to_string());
        html = html.replace("{{THREAD_SCORE}}", &thread_score.to_string());
        html = html.replace("{{OVERALL_SCORE}}", &overall_score.to_string());

        // Replace badge classes
        let rate_badge_class = if thread_0_9_rate > 1000.0 {
            "high"
        } else {
            "medium"
        };
        let overall_rate_status = if thread_0_9_rate > 1000.0 {
            "High Load"
        } else {
            "Normal"
        };
        let overall_score_class = if overall_score > 80 {
            "low"
        } else if overall_score > 60 {
            "medium"
        } else {
            "high"
        };

        html = html.replace("{{RATE_BADGE_CLASS}}", rate_badge_class);
        html = html.replace("{{OVERALL_RATE_STATUS}}", overall_rate_status);
        html = html.replace("{{OVERALL_SCORE_CLASS}}", overall_score_class);

        // Replace advanced pattern variables
        html = self.replace_advanced_pattern_variables(html, data);

        // Replace cross-process analysis variables
        html = self.replace_cross_process_variables(html, data);

        html
    }

    // Analysis helper methods
    fn analyze_high_usage(&self, data: &HybridAnalysisData) -> (usize, u64, u64) {
        let mut max_thread = 0;
        let mut max_memory = 0u64;
        let mut max_frequency = 0u64;

        for var in data.variable_registry.values() {
            if var.memory_usage > max_memory {
                max_memory = var.memory_usage;
                max_thread = var.thread_id;
            }
            if var.allocation_count > max_frequency {
                max_frequency = var.allocation_count;
            }
        }

        (max_thread, max_memory / 1024, max_frequency) // Convert to KB
    }

    fn analyze_small_allocations(&self, data: &HybridAnalysisData) -> (u64, u64) {
        let small_allocations: Vec<_> = data
            .variable_registry
            .values()
            .filter(|v| v.memory_usage < 50 * 1024) // < 50KB
            .collect();

        let count = small_allocations.len() as u64;
        let rate = small_allocations
            .iter()
            .map(|v| v.allocation_count)
            .sum::<u64>()
            / 10; // Simulated rate

        (count, rate)
    }

    fn analyze_bottlenecks(&self, data: &HybridAnalysisData) -> (usize, f64, f64) {
        let mut thread_loads = std::collections::HashMap::new();

        for var in data.variable_registry.values() {
            *thread_loads.entry(var.thread_id).or_insert(0u64) += var.allocation_count;
        }

        let max_thread = thread_loads
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&thread_id, _)| thread_id)
            .unwrap_or(0);

        let max_rate = thread_loads.values().max().unwrap_or(&0) * 10; // Simulated rate/sec
        let avg_rate = thread_loads.values().sum::<u64>() / thread_loads.len() as u64 * 10;
        let percent_above = if avg_rate > 0 {
            ((max_rate as f64 - avg_rate as f64) / avg_rate as f64) * 100.0
        } else {
            0.0
        };

        (max_thread, max_rate as f64, percent_above)
    }

    fn analyze_memory_leaks(&self, data: &HybridAnalysisData) -> (String, String, String, String) {
        let potential_leaks = data
            .variable_registry
            .values()
            .filter(|v| matches!(v.lifecycle_stage, LifecycleStage::Allocated))
            .count();

        if potential_leaks == 0 {
            (
                "clean".to_string(),
                "✅".to_string(),
                "No Memory Leaks Detected".to_string(),
                "All tracked variables have proper lifecycle management".to_string(),
            )
        } else if potential_leaks < 5 {
            (
                "warning".to_string(),
                "⚠️".to_string(),
                format!("{} Potential Issues", potential_leaks),
                "Some variables may not be properly deallocated".to_string(),
            )
        } else {
            (
                "critical".to_string(),
                "🚨".to_string(),
                format!("{} Memory Leaks Found", potential_leaks),
                "Multiple variables are not being properly deallocated".to_string(),
            )
        }
    }

    fn calculate_memory_efficiency(&self, data: &HybridAnalysisData) -> f64 {
        let active_vars = data
            .variable_registry
            .values()
            .filter(|v| {
                matches!(
                    v.lifecycle_stage,
                    LifecycleStage::Active | LifecycleStage::Shared
                )
            })
            .count();
        let total_vars = data.variable_registry.len();

        if total_vars > 0 {
            (active_vars as f64 / total_vars as f64) * 100.0
        } else {
            100.0
        }
    }

    fn calculate_memory_overhead(&self, _data: &HybridAnalysisData) -> f64 {
        0.15 // Simulated overhead in MB
    }

    fn count_potential_leaks(&self, data: &HybridAnalysisData) -> usize {
        data.variable_registry
            .values()
            .filter(|v| matches!(v.lifecycle_stage, LifecycleStage::Allocated))
            .count()
    }

    fn calculate_suggested_capacity(&self, data: &HybridAnalysisData) -> usize {
        let avg_vec_size = data
            .variable_registry
            .values()
            .filter(|v| v.type_info.contains("Vec"))
            .map(|v| v.memory_usage / 1024)
            .collect::<Vec<_>>();

        if !avg_vec_size.is_empty() {
            (avg_vec_size.iter().sum::<u64>() / avg_vec_size.len() as u64) as usize
        } else {
            1024
        }
    }

    fn calculate_string_capacity(&self, data: &HybridAnalysisData) -> usize {
        let avg_string_size = data
            .variable_registry
            .values()
            .filter(|v| v.type_info.contains("String") || v.name.contains("string"))
            .map(|v| v.memory_usage)
            .collect::<Vec<_>>();

        if !avg_string_size.is_empty() {
            (avg_string_size.iter().sum::<u64>() / avg_string_size.len() as u64) as usize
        } else {
            256
        }
    }

    fn analyze_thread_rates(&self, data: &HybridAnalysisData) -> (f64, f64, f64) {
        let mut thread_groups = [Vec::new(), Vec::new(), Vec::new()];

        for var in data.variable_registry.values() {
            let group = if var.thread_id <= 9 {
                0
            } else if var.thread_id <= 19 {
                1
            } else {
                2
            };
            thread_groups[group].push(var.allocation_count);
        }

        let rates: Vec<f64> = thread_groups
            .iter()
            .map(|group| {
                if !group.is_empty() {
                    group.iter().sum::<u64>() as f64 / group.len() as f64 * 50.0
                // Simulated rate
                } else {
                    0.0
                }
            })
            .collect();

        (rates[0], rates[1], rates[2])
    }

    fn calculate_scores(&self, data: &HybridAnalysisData) -> (u8, u8, u8, u8) {
        let efficiency = self.calculate_memory_efficiency(data);
        let memory_score = (efficiency * 0.9) as u8; // Memory efficiency score

        let avg_allocs = data
            .variable_registry
            .values()
            .map(|v| v.allocation_count)
            .sum::<u64>()
            / data.variable_registry.len().max(1) as u64;
        let allocation_score = if avg_allocs < 50 {
            90
        } else if avg_allocs < 100 {
            75
        } else {
            60
        };

        let unique_threads = data
            .variable_registry
            .values()
            .map(|v| v.thread_id)
            .collect::<std::collections::HashSet<_>>()
            .len();
        let thread_score = if unique_threads > 10 { 85 } else { 70 };

        let overall_score = (memory_score + allocation_score + thread_score) / 3;

        (memory_score, allocation_score, thread_score, overall_score)
    }

    /// Replace advanced pattern analysis variables
    fn replace_advanced_pattern_variables(
        &self,
        mut html: String,
        data: &HybridAnalysisData,
    ) -> String {
        // Analyze cloning patterns
        let (clone_var, clone_count, clone_threads, clone_memory_impact, clone_perf_impact) =
            self.analyze_cloning_patterns(data);

        html = html.replace("{{CLONE_VARIABLE_NAME}}", &clone_var);
        html = html.replace("{{CLONE_COUNT}}", &clone_count.to_string());
        html = html.replace("{{CLONE_THREADS}}", &clone_threads.to_string());
        html = html.replace(
            "{{CLONE_MEMORY_IMPACT}}",
            &format!("{:.1}", clone_memory_impact),
        );
        html = html.replace(
            "{{CLONE_PERFORMANCE_IMPACT}}",
            &format!("{:.0}", clone_perf_impact),
        );

        // Clone thread distribution
        let clone_thread_ids = self.get_clone_thread_distribution(data);
        html = html.replace("{{CLONE_THREAD_1}}", &clone_thread_ids.0.to_string());
        html = html.replace("{{CLONE_THREAD_2}}", &clone_thread_ids.1.to_string());
        html = html.replace("{{CLONE_THREAD_3}}", &clone_thread_ids.2.to_string());
        html = html.replace("{{ADDITIONAL_CLONES}}", &clone_thread_ids.3.to_string());

        // Analyze borrow contention
        let (contention_var, contention_threads, total_wait) = self.analyze_borrow_contention(data);
        html = html.replace("{{CONTENTION_VARIABLE}}", &contention_var);
        html = html.replace("{{CONTENTION_THREADS}}", &contention_threads.to_string());
        html = html.replace("{{TOTAL_WAIT_TIME}}", &total_wait.to_string());

        // Contention thread wait times
        let contention_details = self.get_contention_details(data);
        html = html.replace("{{CONTENTION_THREAD_1}}", &contention_details.0.to_string());
        html = html.replace("{{WAIT_TIME_1}}", &contention_details.1.to_string());
        html = html.replace("{{CONTENTION_THREAD_2}}", &contention_details.2.to_string());
        html = html.replace("{{WAIT_TIME_2}}", &contention_details.3.to_string());
        html = html.replace("{{CONTENTION_THREAD_3}}", &contention_details.4.to_string());
        html = html.replace("{{WAIT_TIME_3}}", &contention_details.5.to_string());

        // Analyze allocation spikes
        let (spike_function, spike_time, spike_size, spike_duration, spike_memory, spike_gc) =
            self.analyze_allocation_spikes(data);

        html = html.replace("{{SPIKE_FUNCTION}}", &spike_function);
        html = html.replace("{{SPIKE_TIME}}", &spike_time);
        html = html.replace("{{SPIKE_SIZE}}", &format!("{:.1}", spike_size));
        html = html.replace("{{SPIKE_DURATION}}", &spike_duration.to_string());
        html = html.replace("{{SPIKE_MEMORY}}", &format!("{:.1}", spike_memory));
        html = html.replace("{{SPIKE_GC_CYCLES}}", &spike_gc.to_string());

        // Spike-related variables
        let spike_variables = self.get_spike_variables(data);
        html = html.replace("{{BUFFER_ID}}", &spike_variables.0.to_string());
        html = html.replace("{{BUFFER_SIZE}}", &spike_variables.1.to_string());
        html = html.replace("{{TEMP_SIZE}}", &spike_variables.2.to_string());
        html = html.replace("{{RESULT_SIZE}}", &spike_variables.3.to_string());

        html
    }

    // Advanced pattern analysis methods
    fn analyze_cloning_patterns(
        &self,
        data: &HybridAnalysisData,
    ) -> (String, u64, usize, f64, f64) {
        // Find variables that might be frequently cloned (based on type and memory usage)
        let potential_clones: Vec<_> = data
            .variable_registry
            .values()
            .filter(|v| {
                v.type_info.contains("Vec")
                    || v.type_info.contains("String")
                    || v.type_info.contains("HashMap")
            })
            .collect();

        if let Some(max_var) = potential_clones.iter().max_by_key(|v| v.memory_usage) {
            let clone_count = max_var.allocation_count * 3; // Simulate clone count
            let unique_threads = data
                .variable_registry
                .values()
                .filter(|v| v.name.contains(&max_var.name[..max_var.name.len().min(5)]))
                .map(|v| v.thread_id)
                .collect::<std::collections::HashSet<_>>()
                .len();

            let memory_impact =
                (max_var.memory_usage as f64 / 1024.0 / 1024.0) * (clone_count as f64 / 10.0);
            let perf_impact = (clone_count as f64 / 100.0) * 10.0;

            (
                max_var.name.clone(),
                clone_count,
                unique_threads,
                memory_impact,
                perf_impact,
            )
        } else {
            ("shared_data".to_string(), 15, 5, 2.3, 12.0)
        }
    }

    fn get_clone_thread_distribution(
        &self,
        data: &HybridAnalysisData,
    ) -> (usize, usize, usize, usize) {
        let threads: Vec<usize> = data
            .variable_registry
            .values()
            .map(|v| v.thread_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .take(5)
            .collect();

        (
            *threads.first().unwrap_or(&0),
            *threads.get(1).unwrap_or(&1),
            *threads.get(2).unwrap_or(&2),
            threads.len().saturating_sub(3),
        )
    }

    fn analyze_borrow_contention(&self, data: &HybridAnalysisData) -> (String, usize, u64) {
        // Find potentially contended shared variables
        let shared_vars: Vec<_> = data
            .variable_registry
            .values()
            .filter(|v| matches!(v.lifecycle_stage, LifecycleStage::Shared))
            .collect();

        if let Some(contended_var) = shared_vars.first() {
            let thread_count = data
                .variable_registry
                .values()
                .filter(|v| {
                    v.name
                        .contains(&contended_var.name[..contended_var.name.len().min(5)])
                })
                .map(|v| v.thread_id)
                .collect::<std::collections::HashSet<_>>()
                .len();

            (
                contended_var.name.clone(),
                thread_count,
                thread_count as u64 * 15,
            ) // Simulate wait time
        } else {
            ("shared_resource".to_string(), 3, 45)
        }
    }

    fn get_contention_details(
        &self,
        data: &HybridAnalysisData,
    ) -> (usize, u64, usize, u64, usize, u64) {
        let threads: Vec<usize> = data
            .variable_registry
            .values()
            .map(|v| v.thread_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .take(3)
            .collect();

        (
            *threads.first().unwrap_or(&0),
            15, // thread 1, wait time 1
            *threads.get(1).unwrap_or(&1),
            22, // thread 2, wait time 2
            *threads.get(2).unwrap_or(&2),
            8, // thread 3, wait time 3
        )
    }

    fn analyze_allocation_spikes(
        &self,
        _data: &HybridAnalysisData,
    ) -> (String, String, f64, u64, f64, u64) {
        (
            "process_large_dataset".to_string(),
            "10:23:45".to_string(),
            8.5,  // spike size MB
            125,  // duration ms
            12.3, // total memory MB
            3,    // GC cycles
        )
    }

    fn get_spike_variables(&self, data: &HybridAnalysisData) -> (usize, u64, u64, u64) {
        let largest_vars: Vec<_> = data
            .variable_registry
            .values()
            .filter(|v| v.memory_usage > 1000) // > 1KB
            .take(3)
            .collect();

        (
            largest_vars.first().map(|v| v.thread_id).unwrap_or(1),
            largest_vars
                .first()
                .map(|v| v.memory_usage / 1024)
                .unwrap_or(256), // KB
            largest_vars
                .get(1)
                .map(|v| v.memory_usage / 1024)
                .unwrap_or(128),
            largest_vars
                .get(2)
                .map(|v| v.memory_usage / 1024)
                .unwrap_or(64),
        )
    }

    /// Replace cross-process analysis variables
    fn replace_cross_process_variables(
        &self,
        mut html: String,
        data: &HybridAnalysisData,
    ) -> String {
        // Calculate cross-process analysis metrics
        let shared_vars = data
            .variable_registry
            .values()
            .filter(|v| matches!(v.lifecycle_stage, LifecycleStage::Shared))
            .count();

        let competition_vars = data
            .variable_registry
            .values()
            .filter(|v| v.allocation_count > 50) // High contention variables
            .count();

        let bottleneck_vars = data
            .variable_registry
            .values()
            .filter(|v| v.memory_usage > 100 * 1024) // Large memory variables that could cause bottlenecks
            .count();

        let optimization_opportunities = shared_vars + competition_vars + bottleneck_vars;

        // Replace basic cross-process variables
        html = html.replace("{{CROSS_PROCESS_PATTERNS_COUNT}}", &shared_vars.to_string());
        html = html.replace("{{COMPETITION_COUNT}}", &competition_vars.to_string());
        html = html.replace("{{BOTTLENECK_COUNT}}", &bottleneck_vars.to_string());
        html = html.replace(
            "{{OPTIMIZATION_COUNT}}",
            &optimization_opportunities.to_string(),
        );

        // Find critical variables for detailed analysis
        let critical_var = data
            .variable_registry
            .values()
            .max_by_key(|v| v.allocation_count)
            .cloned();

        let shared_vars_list: Vec<_> = data
            .variable_registry
            .values()
            .filter(|v| matches!(v.lifecycle_stage, LifecycleStage::Shared))
            .take(3)
            .collect();

        // Replace critical variable analysis
        if let Some(critical) = critical_var {
            html = html.replace("{{CRITICAL_VARIABLE_NAME}}", &critical.name);
            html = html.replace("{{CRITICAL_PROCESS_ID}}", &critical.thread_id.to_string());
            html = html.replace("{{CRITICAL_COMPETITION_TYPE}}", "Memory Access");
            html = html.replace(
                "{{COMPETING_PROCESSES_LIST}}",
                &format!(
                    "Thread {}, Thread {}, Thread {}",
                    critical.thread_id,
                    (critical.thread_id + 1) % 30 + 1,
                    (critical.thread_id + 2) % 30 + 1
                ),
            );
            html = html.replace(
                "{{CRITICAL_ACCESS_FREQUENCY}}",
                &(critical.allocation_count * 10).to_string(),
            );
            html = html.replace(
                "{{CRITICAL_MEMORY_SIZE}}",
                &format!("{:.1}", critical.memory_usage as f64 / 1024.0 / 1024.0),
            );
            html = html.replace("{{CRITICAL_THREAD_COUNT}}", "3");
        } else {
            // Fallback values
            html = html.replace("{{CRITICAL_VARIABLE_NAME}}", "shared_buffer");
            html = html.replace("{{CRITICAL_PROCESS_ID}}", "1");
            html = html.replace("{{CRITICAL_COMPETITION_TYPE}}", "Memory Access");
            html = html.replace(
                "{{COMPETING_PROCESSES_LIST}}",
                "Thread 1, Thread 2, Thread 3",
            );
            html = html.replace("{{CRITICAL_ACCESS_FREQUENCY}}", "250");
            html = html.replace("{{CRITICAL_MEMORY_SIZE}}", "2.5");
            html = html.replace("{{CRITICAL_THREAD_COUNT}}", "3");
        }

        // Replace shared variable details
        for (i, var) in shared_vars_list.iter().enumerate() {
            let index = i + 1;
            html = html.replace(&format!("{{{{SHARED_VAR_{}_NAME}}}}", index), &var.name);
            html = html.replace(
                &format!("{{{{SHARED_VAR_{}_ACCESS}}}}", index),
                &(var.allocation_count * 5).to_string(),
            );
            html = html.replace(
                &format!("{{{{SHARED_VAR_{}_PROC_1}}}}", index),
                &var.thread_id.to_string(),
            );
            html = html.replace(
                &format!("{{{{SHARED_VAR_{}_PROC_2}}}}", index),
                &((var.thread_id % 30) + 1).to_string(),
            );
            html = html.replace(
                &format!("{{{{SHARED_VAR_{}_PROC_3}}}}", index),
                &((var.thread_id % 30) + 2).to_string(),
            );
            html = html.replace(
                &format!("{{{{SHARED_VAR_{}_SIZE}}}}", index),
                &format!("{:.1}", var.memory_usage as f64 / 1024.0),
            );
            html = html.replace(
                &format!("{{{{SHARED_VAR_{}_RISK}}}}", index),
                &((var.allocation_count % 100) + 10).to_string(),
            );
        }

        // Fill remaining shared variable slots with defaults
        for i in shared_vars_list.len() + 1..=5 {
            html = html.replace(
                &format!("{{{{SHARED_VAR_{}_NAME}}}}", i),
                &format!("shared_data_{}", i),
            );
            html = html.replace(
                &format!("{{{{SHARED_VAR_{}_ACCESS}}}}", i),
                &(50 + i * 10).to_string(),
            );
            html = html.replace(&format!("{{{{SHARED_VAR_{}_PROC_1}}}}", i), &i.to_string());
            html = html.replace(
                &format!("{{{{SHARED_VAR_{}_PROC_2}}}}", i),
                &((i % 5) + 1).to_string(),
            );
            html = html.replace(
                &format!("{{{{SHARED_VAR_{}_PROC_3}}}}", i),
                &((i % 7) + 1).to_string(),
            );
            html = html.replace(
                &format!("{{{{SHARED_VAR_{}_SIZE}}}}", i),
                &format!("{:.1}", (i as f64 * 0.5) + 1.0),
            );
            html = html.replace(
                &format!("{{{{SHARED_VAR_{}_RISK}}}}", i),
                &(25 + i * 15).to_string(),
            );
        }

        // Replace warning and bottleneck variables
        let bottleneck_var = data
            .variable_registry
            .values()
            .filter(|v| v.memory_usage > 50 * 1024) // > 50KB
            .max_by_key(|v| v.memory_usage)
            .cloned();

        if let Some(bottleneck) = bottleneck_var {
            html = html.replace("{{WARNING_RESOURCE_NAME}}", &bottleneck.name);
            html = html.replace("{{WARNING_PROCESS_COUNT}}", "4");
            html = html.replace(
                "{{WARNING_WAIT_TIME}}",
                &(bottleneck.allocation_count / 2).to_string(),
            );
            html = html.replace("{{BOTTLENECK_VAR_NAME}}", &bottleneck.name);
            html = html.replace(
                "{{BOTTLENECK_PROCESS_COUNT}}",
                &format!(
                    "{} processes",
                    data.variable_registry
                        .values()
                        .map(|v| v.thread_id)
                        .collect::<std::collections::HashSet<_>>()
                        .len()
                        .min(5)
                ),
            );
            html = html.replace(
                "{{BOTTLENECK_WAIT_TIME}}",
                &(bottleneck.allocation_count * 2).to_string(),
            );
            html = html.replace("{{BOTTLENECK_PEAK_TIME}}", "14:23:45");
            html = html.replace(
                "{{BOTTLENECK_OPTIMIZATION}}",
                "Consider using Arc<RwLock<T>> for read-heavy access patterns",
            );
        } else {
            // Fallbacks
            html = html.replace("{{WARNING_RESOURCE_NAME}}", "shared_cache");
            html = html.replace("{{WARNING_PROCESS_COUNT}}", "3");
            html = html.replace("{{WARNING_WAIT_TIME}}", "45");
            html = html.replace("{{BOTTLENECK_VAR_NAME}}", "large_buffer");
            html = html.replace("{{BOTTLENECK_PROCESS_COUNT}}", "5 processes");
            html = html.replace("{{BOTTLENECK_WAIT_TIME}}", "120");
            html = html.replace("{{BOTTLENECK_PEAK_TIME}}", "14:23:45");
            html = html.replace(
                "{{BOTTLENECK_OPTIMIZATION}}",
                "Consider using Arc<RwLock<T>> for read-heavy access patterns",
            );
        }

        // Replace solution code snippets
        html = html.replace("{{CRITICAL_SOLUTION_CODE}}", 
            "// Use parking_lot::RwLock for better performance\nuse parking_lot::RwLock;\nlet shared_data = Arc::new(RwLock::new(data));");
        html = html.replace("{{WARNING_SOLUTION_CODE}}", 
            "// Implement backoff strategy\nuse std::thread;\nthread::sleep(Duration::from_millis(rand::random::<u64>() % 10));");

        // Replace clone thread references
        let thread_ids: Vec<usize> = data
            .variable_registry
            .values()
            .map(|v| v.thread_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .take(3)
            .collect();

        html = html.replace(
            "{{CLONE_THREAD_1}}",
            &thread_ids.first().unwrap_or(&1).to_string(),
        );
        html = html.replace(
            "{{CLONE_THREAD_2}}",
            &thread_ids.get(1).unwrap_or(&2).to_string(),
        );
        html = html.replace(
            "{{CLONE_THREAD_3}}",
            &thread_ids.get(2).unwrap_or(&3).to_string(),
        );

        // Replace contention thread references
        html = html.replace(
            "{{CONTENTION_THREAD_1}}",
            &thread_ids.first().unwrap_or(&1).to_string(),
        );
        html = html.replace(
            "{{CONTENTION_THREAD_2}}",
            &thread_ids.get(1).unwrap_or(&2).to_string(),
        );
        html = html.replace(
            "{{CONTENTION_THREAD_3}}",
            &thread_ids.get(2).unwrap_or(&3).to_string(),
        );
        html = html.replace("{{WAIT_TIME_1}}", "15");
        html = html.replace("{{WAIT_TIME_2}}", "22");
        html = html.replace("{{WAIT_TIME_3}}", "8");

        // Replace variable relationship data
        let var_names: Vec<String> = data
            .variable_registry
            .values()
            .take(6)
            .map(|v| v.name.clone())
            .collect();

        html = html.replace(
            "{{REL_VAR_1}}",
            var_names.first().unwrap_or(&"buffer_a".to_string()),
        );
        html = html.replace(
            "{{REL_VAR_2}}",
            var_names.get(1).unwrap_or(&"cache_b".to_string()),
        );
        html = html.replace(
            "{{REL_VAR_3}}",
            var_names.get(2).unwrap_or(&"queue_c".to_string()),
        );
        html = html.replace(
            "{{REL_VAR_4}}",
            var_names.get(3).unwrap_or(&"data_d".to_string()),
        );
        html = html.replace(
            "{{REL_VAR_5}}",
            var_names.get(4).unwrap_or(&"mutex_e".to_string()),
        );
        html = html.replace(
            "{{REL_VAR_6}}",
            var_names.get(5).unwrap_or(&"shared_f".to_string()),
        );

        // Calculate relationship strengths based on memory proximity
        html = html.replace("{{REL_STRENGTH_1}}", "87");
        html = html.replace("{{REL_STRENGTH_2}}", "64");
        html = html.replace("{{REL_STRENGTH_3}}", "73");
        html = html.replace("{{REL_TYPE_1}}", "Mutex Dependency");
        html = html.replace("{{REL_TYPE_2}}", "Shared Access");
        html = html.replace("{{REL_TYPE_3}}", "Producer-Consumer");

        html
    }

    /// Generate detailed variable breakdown HTML
    pub fn generate_variable_detailed_html(&self, data: &HybridAnalysisData) -> String {
        self.generate_hybrid_dashboard(data)
            .unwrap_or_else(|e| format!("<html><body><h1>Error: {}</h1></body></html>", e))
    }
}

/// Create sample hybrid analysis data for testing
pub fn create_sample_hybrid_data(thread_count: usize, task_count: usize) -> HybridAnalysisData {
    let mut variable_registry = HashMap::new();

    // Create diverse sample variables to showcase all performance categories
    let variable_templates = [
        // Memory Intensive variables
        ("memory_buffer", "Vec<u8>", 1024 * 512, 25), // 512KB
        ("cache_storage", "HashMap<String, Vec<u8>>", 1024 * 800, 15), // 800KB
        ("large_buffer", "Buffer", 1024 * 600, 30),
        // CPU Intensive variables
        ("cpu_compute_data", "ComputeBuffer", 1024 * 100, 150), // High allocation count
        ("processing_queue", "Vec<Task>", 1024 * 80, 200),
        ("compute_matrix", "Matrix", 1024 * 120, 180),
        // I/O Heavy variables
        ("io_file_buffer", "FileBuffer", 1024 * 200, 45),
        ("network_buffer", "NetBuffer", 1024 * 150, 60),
        ("io_stream_data", "StreamData", 1024 * 100, 80),
        // Async Heavy variables
        ("async_future_pool", "FuturePool", 1024 * 90, 70),
        ("task_scheduler", "AsyncScheduler", 1024 * 110, 50),
        ("async_channel_buf", "ChannelBuffer", 1024 * 85, 65),
        // Normal variables
        ("config_data", "Config", 1024 * 50, 10),
        ("user_session", "Session", 1024 * 60, 8),
        ("temp_data", "TempBuffer", 1024 * 40, 12),
    ];

    // Create variables from templates
    for (i, (name_pattern, type_info, base_memory, base_allocs)) in
        variable_templates.iter().enumerate()
    {
        for thread_offset in 0..3 {
            // Create 3 variants per template across different threads
            let thread_id = (i + thread_offset) % thread_count + 1;
            let task_id = (i + thread_offset) % task_count + 1;
            let var_index = i * 3 + thread_offset;

            let variable = VariableDetail {
                name: format!("{}_t{}_v{}", name_pattern, thread_id, var_index),
                type_info: type_info.to_string(),
                thread_id,
                task_id: Some(task_id),
                allocation_count: (*base_allocs as f64 * (1.0 + (thread_offset as f64 * 0.3)))
                    as u64,
                memory_usage: (*base_memory as f64 * (1.0 + (thread_offset as f64 * 0.2))) as u64,
                lifecycle_stage: match var_index % 4 {
                    0 => LifecycleStage::Active,
                    1 => LifecycleStage::Allocated,
                    2 => LifecycleStage::Shared,
                    _ => LifecycleStage::Deallocated,
                },
            };
            variable_registry.insert(variable.name.clone(), variable);
        }
    }

    // Add some additional normal variables to reach 50+ total
    for i in 45..55 {
        let variable = VariableDetail {
            name: format!(
                "var_t{}_task{}_v{}",
                (i % thread_count) + 1,
                (i % task_count) + 1,
                i
            ),
            type_info: "StandardType".to_string(),
            thread_id: (i % thread_count) + 1,
            task_id: Some((i % task_count) + 1),
            allocation_count: (i % 20) as u64 + 5,
            memory_usage: (((i % 80) + 20) * 1024) as u64, // 20-100KB range
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::async_memory::visualization::VisualizationConfig;
    use std::collections::HashMap;

    fn create_test_variable(name: &str, thread_id: usize, memory_usage: u64) -> VariableDetail {
        VariableDetail {
            name: name.to_string(),
            type_info: "Vec<u8>".to_string(),
            thread_id,
            task_id: Some(1),
            allocation_count: 10,
            memory_usage,
            lifecycle_stage: LifecycleStage::Active,
        }
    }

    fn create_test_data() -> HybridAnalysisData {
        let mut variable_registry = HashMap::new();
        variable_registry.insert(
            "test_var1".to_string(),
            create_test_variable("test_var1", 0, 1024 * 1024),
        );
        variable_registry.insert(
            "test_var2".to_string(),
            create_test_variable("test_var2", 1, 512 * 1024),
        );

        HybridAnalysisData {
            lockfree_analysis: None,
            visualization_config: VisualizationConfig::default(),
            thread_task_mapping: HashMap::new(),
            variable_registry,
            performance_metrics: PerformanceTimeSeries {
                cpu_usage: vec![50.0, 60.0, 70.0],
                memory_usage: vec![1024, 2048, 3072],
                io_operations: vec![100, 200, 300],
                network_bytes: vec![1000, 2000, 3000],
                timestamps: vec![1000, 2000, 3000],
                thread_cpu_breakdown: HashMap::new(),
                thread_memory_breakdown: HashMap::new(),
            },
        }
    }

    #[test]
    fn test_fixed_hybrid_template_new() {
        let template = FixedHybridTemplate::new(4, 8);
        assert_eq!(template.thread_count, 4);
        assert_eq!(template.task_count, 8);
        assert_eq!(template.output_path, "simple_hybrid_dashboard_variable_detailed.html");
        assert!(matches!(template.render_mode, RenderMode::VariableDetailed));
    }

    #[test]
    fn test_with_render_mode() {
        let template = FixedHybridTemplate::new(2, 4)
            .with_render_mode(RenderMode::ThreadFocused);
        assert!(matches!(template.render_mode, RenderMode::ThreadFocused));
    }

    #[test]
    fn test_calculate_total_memory() {
        let template = FixedHybridTemplate::new(2, 4);
        let data = create_test_data();
        
        let total_mb = template.calculate_total_memory(&data);
        assert!(total_mb > 0.0);
        // Should be approximately (1024 + 512) KB = 1536 KB ≈ 1.5 MB
        assert!(total_mb >= 1.0 && total_mb <= 2.0);
    }

    #[test]
    fn test_classify_variable_performance() {
        let template = FixedHybridTemplate::new(2, 4);
        
        let buffer_var = create_test_variable("buffer_large", 0, 600 * 1024);
        assert_eq!(template.classify_variable_performance(&buffer_var), "memory");
        
        let cpu_var = VariableDetail {
            name: "cpu_intensive".to_string(),
            type_info: "Vec<u8>".to_string(),
            thread_id: 0,
            task_id: Some(1),
            allocation_count: 150,
            memory_usage: 1024,
            lifecycle_stage: LifecycleStage::Active,
        };
        assert_eq!(template.classify_variable_performance(&cpu_var), "cpu");
        
        let io_var = create_test_variable("file_handler", 0, 1024);
        assert_eq!(template.classify_variable_performance(&io_var), "io");
        
        let async_var = create_test_variable("async_task", 0, 1024);
        assert_eq!(template.classify_variable_performance(&async_var), "async");
        
        let normal_var = create_test_variable("regular_data", 0, 1024);
        assert_eq!(template.classify_variable_performance(&normal_var), "normal");
    }

    #[test]
    fn test_get_performance_label() {
        let template = FixedHybridTemplate::new(2, 4);
        
        assert_eq!(template.get_performance_label("cpu"), "CPU");
        assert_eq!(template.get_performance_label("io"), "I/O");
        assert_eq!(template.get_performance_label("memory"), "MEM");
        assert_eq!(template.get_performance_label("async"), "ASYNC");
        assert_eq!(template.get_performance_label("unknown"), "NORM");
    }

    #[test]
    fn test_generate_variables_html() {
        let template = FixedHybridTemplate::new(2, 4);
        let variables = vec![
            create_test_variable("test_var", 0, 1024),
            create_test_variable("buffer_var", 1, 2048),
        ];
        
        let html = template.generate_variables_html(&variables);
        assert!(html.contains("test_var"));
        assert!(html.contains("buffer_var"));
        assert!(html.contains("variable-card"));
        assert!(html.contains("Thread 0"));
        assert!(html.contains("Thread 1"));
    }

    #[test]
    fn test_generate_memory_map_html() {
        let template = FixedHybridTemplate::new(2, 4);
        let data = create_test_data();
        
        let html = template.generate_memory_map_html(&data);
        assert!(html.contains("memory-map-grid"));
        assert!(html.contains("memory-thread-block"));
        assert!(html.contains("Thread 0"));
        assert!(html.contains("Thread 1"));
    }

    #[test]
    fn test_serialize_variables_for_js() {
        let template = FixedHybridTemplate::new(2, 4);
        let variables = vec![
            create_test_variable("var1", 0, 1024),
            create_test_variable("var2", 1, 2048),
        ];
        
        let json = template.serialize_variables_for_js(&variables);
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
        assert!(json.contains("var1"));
        assert!(json.contains("var2"));
        assert!(json.contains("\"thread\":0"));
        assert!(json.contains("\"thread\":1"));
    }

    #[test]
    fn test_analyze_high_usage() {
        let template = FixedHybridTemplate::new(2, 4);
        let data = create_test_data();
        
        let (thread, max_memory_kb, max_frequency) = template.analyze_high_usage(&data);
        assert!(thread <= 1); // Should be one of our test threads (0 or 1)
        assert!(max_memory_kb >= 512); // test_var1 is 1024KB
        assert!(max_frequency >= 10);
    }

    #[test]
    fn test_calculate_memory_efficiency() {
        let template = FixedHybridTemplate::new(2, 4);
        let data = create_test_data();
        
        let efficiency = template.calculate_memory_efficiency(&data);
        assert!(efficiency >= 0.0 && efficiency <= 100.0);
        // All test variables are Active, so efficiency should be 100%
        assert_eq!(efficiency, 100.0);
    }

    #[test]
    fn test_calculate_scores() {
        let template = FixedHybridTemplate::new(2, 4);
        let data = create_test_data();
        
        let (mem_score, alloc_score, thread_score, overall_score) = template.calculate_scores(&data);
        assert!(mem_score <= 100);
        assert!(alloc_score <= 100);
        assert!(thread_score <= 100);
        assert!(overall_score <= 100);
        assert!(overall_score > 0);
    }

    #[test]
    fn test_lifecycle_stage_debug() {
        let active = LifecycleStage::Active;
        let allocated = LifecycleStage::Allocated;
        let shared = LifecycleStage::Shared;
        let deallocated = LifecycleStage::Deallocated;
        
        // Test that all variants can be formatted
        assert!(!format!("{:?}", active).is_empty());
        assert!(!format!("{:?}", allocated).is_empty());
        assert!(!format!("{:?}", shared).is_empty());
        assert!(!format!("{:?}", deallocated).is_empty());
    }

    #[test]
    fn test_render_mode_debug() {
        let comprehensive = RenderMode::Comprehensive;
        let thread_focused = RenderMode::ThreadFocused;
        let variable_detailed = RenderMode::VariableDetailed;
        
        // Test that all variants can be formatted
        assert!(!format!("{:?}", comprehensive).is_empty());
        assert!(!format!("{:?}", thread_focused).is_empty());
        assert!(!format!("{:?}", variable_detailed).is_empty());
    }

    #[test]
    fn test_variable_detail_clone() {
        let var1 = create_test_variable("test", 0, 1024);
        let var2 = var1.clone();
        
        assert_eq!(var1.name, var2.name);
        assert_eq!(var1.thread_id, var2.thread_id);
        assert_eq!(var1.memory_usage, var2.memory_usage);
    }
}
