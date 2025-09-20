//! Async Task Performance Visualization Module
//!
//! This module provides comprehensive visualization capabilities for async task performance data.
//! It generates interactive HTML reports with charts, baselines, rankings, and detailed analytics.

use super::{TaskId, TaskResourceProfile, TaskType};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Visualization configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub title: String,
    pub theme: Theme,
    pub include_charts: bool,
    pub include_baselines: bool,
    pub include_rankings: bool,
    pub include_efficiency_breakdown: bool,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            title: "Async Task Performance Analysis".to_string(),
            theme: Theme::Dark,
            include_charts: true,
            include_baselines: true,
            include_rankings: true,
            include_efficiency_breakdown: true,
        }
    }
}

/// UI Theme options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
}

/// Baseline performance metrics for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaselines {
    pub avg_cpu_percent: f64,
    pub avg_memory_mb: f64,
    pub avg_io_mbps: f64,
    pub avg_network_mbps: f64,
    pub avg_efficiency_score: f64,
}

/// Task ranking within its category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRanking {
    pub rank: usize,
    pub total_in_category: usize,
    pub category_name: String,
}

/// Performance comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparison {
    pub value: f64,
    pub baseline: f64,
    pub difference_percent: f64,
    pub comparison_type: ComparisonType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonType {
    AboveAverage,
    BelowAverage,
    NearAverage,
}

/// Main visualization generator
pub struct VisualizationGenerator {
    config: VisualizationConfig,
}

impl VisualizationGenerator {
    /// Create a new visualization generator with default config
    pub fn new() -> Self {
        Self {
            config: VisualizationConfig::default(),
        }
    }

    /// Create a new visualization generator with custom config
    pub fn with_config(config: VisualizationConfig) -> Self {
        Self { config }
    }

    /// Generate complete HTML report from task profiles
    pub fn generate_html_report(
        &self,
        profiles: &HashMap<TaskId, TaskResourceProfile>,
    ) -> Result<String, VisualizationError> {
        let analytics = self.analyze_profiles(profiles)?;
        self.build_html_report(&analytics, profiles)
    }

    /// Generate analytics data from profiles
    pub fn analyze_profiles(
        &self,
        profiles: &HashMap<TaskId, TaskResourceProfile>,
    ) -> Result<PerformanceAnalytics, VisualizationError> {
        if profiles.is_empty() {
            return Err(VisualizationError::NoDataAvailable);
        }

        let baselines = self.calculate_baselines(profiles);
        let rankings = self.calculate_rankings(profiles);
        let comparisons = self.calculate_comparisons(profiles, &baselines);

        Ok(PerformanceAnalytics {
            baselines,
            rankings,
            comparisons,
            total_tasks: profiles.len(),
        })
    }

    /// Calculate baseline metrics
    fn calculate_baselines(&self, profiles: &HashMap<TaskId, TaskResourceProfile>) -> PerformanceBaselines {
        let total = profiles.len() as f64;
        let mut totals = (0.0, 0.0, 0.0, 0.0, 0.0);

        for profile in profiles.values() {
            totals.0 += profile.cpu_metrics.usage_percent;
            totals.1 += profile.memory_metrics.current_bytes as f64 / 1_048_576.0;
            totals.2 += profile.io_metrics.bandwidth_mbps;
            totals.3 += profile.network_metrics.throughput_mbps;
            totals.4 += profile.efficiency_score;
        }

        PerformanceBaselines {
            avg_cpu_percent: totals.0 / total,
            avg_memory_mb: totals.1 / total,
            avg_io_mbps: totals.2 / total,
            avg_network_mbps: totals.3 / total,
            avg_efficiency_score: totals.4 / total,
        }
    }

    /// Calculate category rankings
    fn calculate_rankings(&self, profiles: &HashMap<TaskId, TaskResourceProfile>) -> HashMap<TaskId, CategoryRanking> {
        let mut rankings = HashMap::new();
        let mut category_groups: HashMap<String, Vec<(TaskId, &TaskResourceProfile)>> = HashMap::new();

        // Group by task type
        for (task_id, profile) in profiles {
            let category = format!("{:?}", profile.task_type);
            category_groups.entry(category).or_default().push((*task_id, profile));
        }

        // Calculate rankings within each category
        for (category_name, mut tasks) in category_groups {
            tasks.sort_by(|a, b| {
                b.1.efficiency_score
                    .partial_cmp(&a.1.efficiency_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let total_in_category = tasks.len();
            for (rank, (task_id, _)) in tasks.iter().enumerate() {
                rankings.insert(
                    *task_id,
                    CategoryRanking {
                        rank: rank + 1,
                        total_in_category,
                        category_name: category_name.clone(),
                    },
                );
            }
        }

        rankings
    }

    /// Calculate performance comparisons
    fn calculate_comparisons(
        &self,
        profiles: &HashMap<TaskId, TaskResourceProfile>,
        baselines: &PerformanceBaselines,
    ) -> HashMap<TaskId, TaskComparisons> {
        let mut comparisons = HashMap::new();

        for (task_id, profile) in profiles {
            let cpu_comp = self.compare_to_baseline(
                profile.cpu_metrics.usage_percent,
                baselines.avg_cpu_percent,
            );
            let memory_comp = self.compare_to_baseline(
                profile.memory_metrics.current_bytes as f64 / 1_048_576.0,
                baselines.avg_memory_mb,
            );
            let io_comp = self.compare_to_baseline(
                profile.io_metrics.bandwidth_mbps,
                baselines.avg_io_mbps,
            );
            let network_comp = self.compare_to_baseline(
                profile.network_metrics.throughput_mbps,
                baselines.avg_network_mbps,
            );

            comparisons.insert(
                *task_id,
                TaskComparisons {
                    cpu: cpu_comp,
                    memory: memory_comp,
                    io: io_comp,
                    network: network_comp,
                },
            );
        }

        comparisons
    }

    /// Compare a value to its baseline
    fn compare_to_baseline(&self, value: f64, baseline: f64) -> PerformanceComparison {
        let difference_percent = if baseline != 0.0 {
            ((value - baseline) / baseline) * 100.0
        } else {
            0.0
        };

        let comparison_type = if difference_percent.abs() < 5.0 {
            ComparisonType::NearAverage
        } else if difference_percent > 0.0 {
            ComparisonType::AboveAverage
        } else {
            ComparisonType::BelowAverage
        };

        PerformanceComparison {
            value,
            baseline,
            difference_percent,
            comparison_type,
        }
    }
}

/// Complete analytics data
#[derive(Debug, Clone)]
pub struct PerformanceAnalytics {
    pub baselines: PerformanceBaselines,
    pub rankings: HashMap<TaskId, CategoryRanking>,
    pub comparisons: HashMap<TaskId, TaskComparisons>,
    pub total_tasks: usize,
}

/// Task-specific comparison data
#[derive(Debug, Clone)]
pub struct TaskComparisons {
    pub cpu: PerformanceComparison,
    pub memory: PerformanceComparison,
    pub io: PerformanceComparison,
    pub network: PerformanceComparison,
}

/// Visualization errors
#[derive(Debug, thiserror::Error)]
pub enum VisualizationError {
    #[error("No data available for visualization")]
    NoDataAvailable,
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Template generation error: {0}")]
    TemplateError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl Default for VisualizationGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualizationGenerator {
    /// Build complete HTML report
    fn build_html_report(
        &self,
        analytics: &PerformanceAnalytics,
        profiles: &HashMap<TaskId, TaskResourceProfile>,
    ) -> Result<String, VisualizationError> {
        let mut html = String::new();
        
        // HTML Header
        html.push_str(&self.generate_html_header());
        
        // Summary Section
        html.push_str(&self.generate_summary_section(analytics));
        
        // Charts Section (if enabled) - moved to top
        if self.config.include_charts {
            html.push_str(&self.generate_charts_section(profiles)?);
        }
        
        // Tasks Section
        html.push_str(&self.generate_tasks_section(analytics, profiles)?);
        
        // HTML Footer
        html.push_str(&self.generate_html_footer());
        
        Ok(html)
    }

    /// Generate HTML header with styles
    fn generate_html_header(&self) -> String {
        let theme_styles = match self.config.theme {
            Theme::Dark => self.get_dark_theme_styles(),
            Theme::Light => self.get_light_theme_styles(),
        };

        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        {}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>📊 {}</h1>
            <p>Advanced performance analysis with baselines, rankings, and trends</p>
        </div>
"#, self.config.title, theme_styles, self.config.title)
    }

    /// Generate summary statistics section
    fn generate_summary_section(&self, analytics: &PerformanceAnalytics) -> String {
        format!(r#"
        <div class="summary">
            <div class="summary-card">
                <h3>Total Tasks</h3>
                <div class="value">{}</div>
            </div>
            <div class="summary-card">
                <h3>Avg CPU Usage</h3>
                <div class="value">{:.1}%</div>
            </div>
            <div class="summary-card">
                <h3>Avg Memory</h3>
                <div class="value">{:.0}MB</div>
            </div>
            <div class="summary-card">
                <h3>Avg Efficiency</h3>
                <div class="value">{:.0}%</div>
            </div>
        </div>
"#, 
            analytics.total_tasks,
            analytics.baselines.avg_cpu_percent,
            analytics.baselines.avg_memory_mb,
            analytics.baselines.avg_efficiency_score * 100.0
        )
    }

    /// Generate tasks section with cards
    fn generate_tasks_section(
        &self,
        analytics: &PerformanceAnalytics,
        profiles: &HashMap<TaskId, TaskResourceProfile>,
    ) -> Result<String, VisualizationError> {
        let mut html = String::new();
        
        html.push_str(r#"
        <div class="tasks-section">
            <h2 class="section-title">Task Performance Details</h2>
            <div class="tasks-grid">
"#);

        // Sort tasks by efficiency score
        let mut sorted_profiles: Vec<_> = profiles.iter().collect();
        sorted_profiles.sort_by(|a, b| {
            b.1.efficiency_score
                .partial_cmp(&a.1.efficiency_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (task_id, profile) in sorted_profiles {
            html.push_str(&self.generate_task_card(*task_id, profile, analytics)?);
        }

        html.push_str(r#"
            </div>
        </div>
"#);

        Ok(html)
    }

    /// Generate individual task card
    fn generate_task_card(
        &self,
        task_id: TaskId,
        profile: &TaskResourceProfile,
        analytics: &PerformanceAnalytics,
    ) -> Result<String, VisualizationError> {
        let ranking = analytics.rankings.get(&task_id);
        let comparisons = analytics.comparisons.get(&task_id);
        
        let task_type_class = format!("{:?}", profile.task_type).to_lowercase();
        
        let rank_info = if let Some(ranking) = ranking {
            let rank_class = match ranking.rank {
                1 => "rank-1",
                2 => "rank-2",
                3 => "rank-3",
                _ => "",
            };
            format!(r#"<div class="ranking-badge {}">#{}/{}</div>"#, 
                rank_class, ranking.rank, ranking.total_in_category)
        } else {
            String::new()
        };

        let comparison_info = if let Some(comp) = comparisons {
            self.generate_comparison_info(comp)
        } else {
            String::new()
        };

        let efficiency_tooltip = if self.config.include_efficiency_breakdown {
            format!(r#"
                <div class="info-icon">?
                    <div class="tooltip">
                        <strong>Efficiency Breakdown:</strong><br>
                        CPU: {:.1}%<br>
                        Memory: {:.1}%<br>
                        IO: {:.1}%<br>
                        Network: {:.1}%<br>
                        Overall: {:.1}%
                    </div>
                </div>
"#, 
                profile.efficiency_explanation.component_scores.cpu_efficiency * 100.0,
                profile.efficiency_explanation.component_scores.memory_efficiency * 100.0,
                profile.efficiency_explanation.component_scores.io_efficiency * 100.0,
                profile.efficiency_explanation.component_scores.network_efficiency * 100.0,
                profile.efficiency_score * 100.0
            )
        } else {
            String::new()
        };

        Ok(format!(r#"
                <div class="task-card">
                    {}
                    <div class="task-header {}">
                        <h3 class="task-name">{}</h3>
                        <span class="task-badge {}">{:?}</span>
                    </div>
                    <div class="task-content">
                        <div class="metrics-grid">
                            <div class="metric-item">
                                <div class="metric-label">CPU Usage</div>
                                <div class="metric-value">{:.1}%</div>
                                {}
                            </div>
                            <div class="metric-item">
                                <div class="metric-label">Memory</div>
                                <div class="metric-value">{:.0}MB</div>
                                {}
                            </div>
                            <div class="metric-item">
                                <div class="metric-label">IO Bandwidth</div>
                                <div class="metric-value">{:.1}MB/s</div>
                                {}
                            </div>
                            <div class="metric-item">
                                <div class="metric-label">Network</div>
                                <div class="metric-value">{:.1}Mbps</div>
                                {}
                            </div>
                        </div>
                        
                        <div class="efficiency-section">
                            <div class="efficiency-title">
                                Efficiency Score
                                {}
                            </div>
                            <div class="efficiency-bar">
                                <div class="efficiency-fill" style="width: {:.1}%"></div>
                            </div>
                            <div class="efficiency-score">{:.1}%</div>
                        </div>
                        
                        <div class="source-info">
                            <div class="source-title">Source Location</div>
                            <div class="source-detail">
                                <span class="source-label">File:</span>
                                <span class="source-value">{}</span>
                            </div>
                            <div class="source-detail">
                                <span class="source-label">Line:</span>
                                <span class="source-value">{}</span>
                            </div>
                            <div class="source-detail">
                                <span class="source-label">Function:</span>
                                <span class="source-value">{}</span>
                            </div>
                        </div>
                    </div>
                </div>
"#, 
            rank_info,
            task_type_class,
            profile.task_name,
            task_type_class,
            profile.task_type,
            profile.cpu_metrics.usage_percent,
            if let Some(comp) = comparisons { format!("<div class=\"metric-comparison {}\">{}</div>", self.get_comparison_class(&comp.cpu), self.format_comparison(&comp.cpu)) } else { String::new() },
            profile.memory_metrics.current_bytes as f64 / 1_048_576.0,
            if let Some(comp) = comparisons { format!("<div class=\"metric-comparison {}\">{}</div>", self.get_comparison_class(&comp.memory), self.format_comparison(&comp.memory)) } else { String::new() },
            profile.io_metrics.bandwidth_mbps,
            if let Some(comp) = comparisons { format!("<div class=\"metric-comparison {}\">{}</div>", self.get_comparison_class(&comp.io), self.format_comparison(&comp.io)) } else { String::new() },
            profile.network_metrics.throughput_mbps,
            if let Some(comp) = comparisons { format!("<div class=\"metric-comparison {}\">{}</div>", self.get_comparison_class(&comp.network), self.format_comparison(&comp.network)) } else { String::new() },
            efficiency_tooltip,
            profile.efficiency_score * 100.0,
            profile.efficiency_score * 100.0,
            profile.source_location.file_path,
            profile.source_location.line_number,
            profile.source_location.function_name
        ))
    }

    /// Generate comparison information display
    fn generate_comparison_info(&self, comparisons: &TaskComparisons) -> String {
        // Implementation for comparison display
        String::new()
    }

    /// Format comparison for display
    fn format_comparison(&self, comparison: &PerformanceComparison) -> String {
        match comparison.comparison_type {
            ComparisonType::NearAverage => "(≈ avg)".to_string(),
            ComparisonType::AboveAverage => format!("(+{:.1}% vs avg)", comparison.difference_percent.abs()),
            ComparisonType::BelowAverage => format!("(-{:.1}% vs avg)", comparison.difference_percent.abs()),
        }
    }

    /// Get CSS class for comparison
    fn get_comparison_class(&self, comparison: &PerformanceComparison) -> &'static str {
        match comparison.comparison_type {
            ComparisonType::NearAverage => "comparison-average",
            ComparisonType::AboveAverage => "comparison-above",
            ComparisonType::BelowAverage => "comparison-below",
        }
    }

    /// Generate charts section
    fn generate_charts_section(&self, profiles: &HashMap<TaskId, TaskResourceProfile>) -> Result<String, VisualizationError> {
        let mut html = String::new();
        
        html.push_str(r#"
        <div class="charts-section">
            <h2 class="section-title">📈 Performance Trends</h2>
"#);

        // Generate simple CSS charts
        let chart_html = self.generate_chart_scripts(profiles)?;
        html.push_str(&chart_html);
        
        html.push_str(r#"
        </div>
"#);

        Ok(html)
    }

    /// Generate simple CSS charts (no JavaScript)
    fn generate_chart_scripts(&self, profiles: &HashMap<TaskId, TaskResourceProfile>) -> Result<String, VisualizationError> {
        let mut cpu_bars = String::new();
        let mut memory_bars = String::new();
        
        // Find max values for scaling
        let max_cpu = profiles.values().map(|p| p.cpu_metrics.usage_percent).fold(0.0, f64::max).max(100.0);
        let max_memory = profiles.values().map(|p| p.memory_metrics.current_bytes as f64 / 1_048_576.0).fold(0.0, f64::max);

        for profile in profiles.values() {
            let cpu_percent = profile.cpu_metrics.usage_percent;
            let memory_mb = profile.memory_metrics.current_bytes as f64 / 1_048_576.0;
            
            let cpu_width = (cpu_percent / max_cpu * 100.0).min(100.0);
            let memory_width = if max_memory > 0.0 { (memory_mb / max_memory * 100.0).min(100.0) } else { 0.0 };
            
            cpu_bars.push_str(&format!(r#"
                <div class="chart-bar">
                    <div class="bar-label">{}</div>
                    <div class="bar-container">
                        <div class="bar-fill cpu-bar" style="width: {:.1}%"></div>
                        <div class="bar-value">{:.1}%</div>
                    </div>
                </div>
"#, profile.task_name, cpu_width, cpu_percent));

            memory_bars.push_str(&format!(r#"
                <div class="chart-bar">
                    <div class="bar-label">{}</div>
                    <div class="bar-container">
                        <div class="bar-fill memory-bar" style="width: {:.1}%"></div>
                        <div class="bar-value">{:.1}MB</div>
                    </div>
                </div>
"#, profile.task_name, memory_width, memory_mb));
        }

        // Generate network bars
        let mut network_bars = String::new();
        let max_network = profiles.values().map(|p| p.network_metrics.throughput_mbps).fold(0.0, f64::max);
        
        for profile in profiles.values() {
            let network_mbps = profile.network_metrics.throughput_mbps;
            let network_width = if max_network > 0.0 { (network_mbps / max_network * 100.0).min(100.0) } else { 0.0 };
            
            network_bars.push_str(&format!(r#"
                <div class="chart-bar">
                    <div class="bar-label">{}</div>
                    <div class="bar-container">
                        <div class="bar-fill network-bar" style="width: {:.1}%"></div>
                        <div class="bar-value">{:.1}Mbps</div>
                    </div>
                </div>
"#, profile.task_name, network_width, network_mbps));
        }

        Ok(format!(r#"
        <div class="simple-charts">
            <div class="simple-chart">
                <h4>CPU Usage Distribution</h4>
                <div class="chart-bars">
                    {}
                </div>
            </div>
            <div class="simple-chart">
                <h4>Memory Usage Distribution</h4>
                <div class="chart-bars">
                    {}
                </div>
            </div>
            <div class="simple-chart">
                <h4>Network Throughput Distribution</h4>
                <div class="chart-bars">
                    {}
                </div>
            </div>
        </div>
"#, cpu_bars, memory_bars, network_bars))
    }

    /// Generate HTML footer
    fn generate_html_footer(&self) -> String {
        r#"
    </div>
</body>
</html>
"#.to_string()
    }

    /// Get dark theme CSS styles
    fn get_dark_theme_styles(&self) -> &'static str {
        r#"
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            margin: 0;
            padding: 20px;
            background: #0d1117;
            color: #f0f6fc;
            line-height: 1.6;
        }
        
        .container {
            max-width: 1400px;
            margin: 0 auto;
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 12px;
            overflow: hidden;
        }
        
        .header {
            background: linear-gradient(135deg, #58a6ff 0%, #a5a5ff 100%);
            padding: 2rem;
            text-align: center;
            color: white;
        }
        
        .header h1 {
            margin: 0;
            font-size: 2.5rem;
            font-weight: 700;
        }
        
        .summary {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1.5rem;
            padding: 2rem;
            background: #21262d;
        }
        
        .summary-card {
            background: #161b22;
            border: 1px solid #30363d;
            padding: 1.5rem;
            border-radius: 8px;
            text-align: center;
        }
        
        .summary-card h3 {
            margin: 0 0 0.5rem 0;
            color: #8b949e;
            font-size: 0.9rem;
            text-transform: uppercase;
        }
        
        .summary-card .value {
            font-size: 2rem;
            font-weight: 700;
            color: #58a6ff;
        }
        
        .tasks-section {
            padding: 2rem;
        }
        
        .section-title {
            margin: 0 0 2rem 0;
            font-size: 1.5rem;
            font-weight: 600;
            color: #f0f6fc;
            text-align: center;
        }
        
        .tasks-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(450px, 1fr));
            gap: 1.5rem;
        }
        
        .task-card {
            background: #21262d;
            border: 1px solid #30363d;
            border-radius: 10px;
            overflow: hidden;
            transition: transform 0.2s ease;
            position: relative;
        }
        
        .task-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 16px rgba(0, 0, 0, 0.3);
        }
        
        .ranking-badge {
            position: absolute;
            top: 10px;
            right: 10px;
            background: linear-gradient(135deg, #f9c513, #ffd700);
            color: #000;
            padding: 0.25rem 0.5rem;
            border-radius: 12px;
            font-size: 0.7rem;
            font-weight: 700;
            z-index: 10;
        }
        
        .ranking-badge.rank-1 { background: linear-gradient(135deg, #ffd700, #ffed4e); }
        .ranking-badge.rank-2 { background: linear-gradient(135deg, #c0c0c0, #e8e8e8); }
        .ranking-badge.rank-3 { background: linear-gradient(135deg, #cd7f32, #daa520); }
        
        .task-header {
            padding: 1.5rem;
            background: #161b22;
            border-bottom: 1px solid #30363d;
            position: relative;
        }
        
        .task-header::before {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 3px;
        }
        
        .task-header.cpuintensive::before { background: #f85149; }
        .task-header.iointensive::before { background: #58a6ff; }
        .task-header.networkintensive::before { background: #3fb950; }
        .task-header.memoryintensive::before { background: #a5a5ff; }
        .task-header.mixed::before { background: #f9c513; }
        
        .task-name {
            margin: 0 0 0.5rem 0;
            font-size: 1.125rem;
            font-weight: 600;
            color: #f0f6fc;
        }
        
        .task-badge {
            display: inline-block;
            padding: 0.25rem 0.75rem;
            border-radius: 12px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
        }
        
        .task-badge.cpuintensive { background: #f85149; color: white; }
        .task-badge.iointensive { background: #58a6ff; color: white; }
        .task-badge.networkintensive { background: #3fb950; color: white; }
        .task-badge.memoryintensive { background: #a5a5ff; color: white; }
        .task-badge.mixed { background: #f9c513; color: black; }
        
        .task-content {
            padding: 1.5rem;
        }
        
        .metrics-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 1rem;
            margin-bottom: 1rem;
        }
        
        .metric-item {
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 6px;
            padding: 1rem;
            text-align: center;
        }
        
        .metric-label {
            font-size: 0.75rem;
            color: #8b949e;
            margin-bottom: 0.25rem;
            text-transform: uppercase;
        }
        
        .metric-value {
            font-size: 1.25rem;
            font-weight: 700;
            color: #f0f6fc;
        }
        
        .metric-comparison {
            font-size: 0.7rem;
            margin-top: 0.25rem;
        }
        
        .comparison-above { color: #f85149; }
        .comparison-below { color: #3fb950; }
        .comparison-average { color: #f9c513; }
        
        .efficiency-section {
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 6px;
            padding: 1rem;
            margin-top: 1rem;
        }
        
        .efficiency-title {
            font-size: 0.875rem;
            color: #8b949e;
            margin-bottom: 0.5rem;
            text-transform: uppercase;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }
        
        .info-icon {
            cursor: help;
            background: #30363d;
            color: #8b949e;
            border-radius: 50%;
            width: 16px;
            height: 16px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 0.7rem;
            font-weight: bold;
            position: relative;
        }
        
        .info-icon:hover {
            background: #58a6ff;
            color: white;
        }
        
        .tooltip {
            position: absolute;
            bottom: 100%;
            right: 0;
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 6px;
            padding: 0.75rem;
            min-width: 200px;
            font-size: 0.8rem;
            color: #f0f6fc;
            box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
            z-index: 1000;
            opacity: 0;
            visibility: hidden;
            transition: opacity 0.3s ease;
        }
        
        .info-icon:hover .tooltip {
            opacity: 1;
            visibility: visible;
        }
        
        .efficiency-bar {
            width: 100%;
            height: 8px;
            background: #30363d;
            border-radius: 4px;
            overflow: hidden;
            margin-bottom: 0.5rem;
        }
        
        .efficiency-fill {
            height: 100%;
            background: linear-gradient(90deg, #3fb950, #f9c513, #f85149);
            border-radius: 4px;
        }
        
        .efficiency-score {
            font-size: 1rem;
            font-weight: 700;
            color: #58a6ff;
            text-align: right;
        }
        
        .source-info {
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 6px;
            padding: 1rem;
            margin-top: 1rem;
        }
        
        .source-title {
            font-size: 0.875rem;
            color: #8b949e;
            margin-bottom: 0.5rem;
            text-transform: uppercase;
        }
        
        .source-detail {
            display: flex;
            justify-content: space-between;
            margin-bottom: 0.25rem;
            font-size: 0.8rem;
        }
        
        .source-label {
            color: #8b949e;
        }
        
        .source-value {
            color: #f0f6fc;
            font-family: 'Courier New', monospace;
        }
        
        .charts-section {
            padding: 2rem;
            background: #161b22;
            border-top: 1px solid #30363d;
        }
        
        .charts-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 2rem;
            margin-top: 1.5rem;
        }
        
        .chart-container {
            background: #21262d;
            border: 1px solid #30363d;
            border-radius: 8px;
            padding: 1.5rem;
        }
        
        .chart-title {
            margin: 0 0 1rem 0;
            font-size: 1rem;
            font-weight: 600;
            color: #f0f6fc;
            text-align: center;
        }
        
        /* Simple CSS Charts */
        .simple-charts {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 2rem;
            padding: 2rem;
        }
        
        .simple-chart {
            background: #21262d;
            border: 1px solid #30363d;
            border-radius: 8px;
            padding: 1.5rem;
        }
        
        .simple-chart h4 {
            margin: 0 0 1rem 0;
            font-size: 1rem;
            font-weight: 600;
            color: #f0f6fc;
            text-align: center;
        }
        
        .chart-bars {
            display: flex;
            flex-direction: column;
            gap: 0.75rem;
        }
        
        .chart-bar {
            display: flex;
            align-items: center;
            gap: 1rem;
        }
        
        .bar-label {
            min-width: 120px;
            font-size: 0.8rem;
            color: #8b949e;
            text-align: right;
        }
        
        .bar-container {
            flex: 1;
            display: flex;
            align-items: center;
            gap: 0.5rem;
            position: relative;
        }
        
        .bar-fill {
            height: 20px;
            border-radius: 4px;
            transition: width 0.6s ease;
            position: relative;
        }
        
        .cpu-bar {
            background: linear-gradient(90deg, #f85149, #ff6b6b);
        }
        
        .memory-bar {
            background: linear-gradient(90deg, #a5a5ff, #b39ddb);
        }
        
        .network-bar {
            background: linear-gradient(90deg, #3fb950, #a5d6a7);
        }
        
        .bar-value {
            font-size: 0.8rem;
            color: #f0f6fc;
            font-weight: 600;
            min-width: 50px;
        }
        
        @media (max-width: 1024px) {
            .simple-charts {
                grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            }
            
            .bar-label {
                min-width: 100px;
                font-size: 0.75rem;
            }
        }
        
        @media (max-width: 768px) {
            .tasks-grid {
                grid-template-columns: 1fr;
            }
            .metrics-grid {
                grid-template-columns: 1fr;
            }
        }
        "#
    }

    /// Get light theme CSS styles
    fn get_light_theme_styles(&self) -> &'static str {
        r#"
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            margin: 0;
            padding: 20px;
            background: #ffffff;
            color: #24292f;
            line-height: 1.6;
        }
        
        .container {
            max-width: 1400px;
            margin: 0 auto;
            background: #f6f8fa;
            border: 1px solid #d0d7de;
            border-radius: 12px;
            overflow: hidden;
        }
        
        .header {
            background: linear-gradient(135deg, #0969da 0%, #8250df 100%);
            padding: 2rem;
            text-align: center;
            color: white;
        }
        
        .summary {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1.5rem;
            padding: 2rem;
            background: #ffffff;
        }
        
        .summary-card {
            background: #f6f8fa;
            border: 1px solid #d0d7de;
            padding: 1.5rem;
            border-radius: 8px;
            text-align: center;
        }
        
        .task-card {
            background: #ffffff;
            border: 1px solid #d0d7de;
            border-radius: 10px;
            overflow: hidden;
            transition: transform 0.2s ease;
            position: relative;
        }
        
        .task-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 16px rgba(0, 0, 0, 0.1);
        }
        
        /* Add more light theme styles as needed */
        "#
    }
}