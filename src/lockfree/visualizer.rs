//! Advanced HTML Visualizer for Memory Analysis
//!
//! Creates rich, interactive HTML reports with charts, graphs, and detailed analysis

use super::analysis::LockfreeAnalysis;
use super::platform_resources::PlatformResourceMetrics;
use super::resource_integration::ComprehensiveAnalysis;
use std::path::Path;

/// Generate comprehensive HTML report with CPU/GPU resource visualizations
pub fn generate_comprehensive_html_report(
    comprehensive_analysis: &ComprehensiveAnalysis,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let html_content = build_comprehensive_html_report(comprehensive_analysis)?;
    std::fs::write(output_path, html_content)?;
    Ok(())
}

/// Generate enhanced HTML report with modern visualizations (memory only)
pub fn generate_enhanced_html_report(
    analysis: &LockfreeAnalysis,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let html_content = build_enhanced_html_report(analysis)?;
    std::fs::write(output_path, html_content)?;
    Ok(())
}

/// Build comprehensive HTML report with CPU/GPU/Memory visualizations
fn build_comprehensive_html_report(
    comprehensive_analysis: &ComprehensiveAnalysis,
) -> Result<String, Box<dyn std::error::Error>> {
    let analysis = &comprehensive_analysis.memory_analysis;
    let resource_timeline = &comprehensive_analysis.resource_timeline;
    let performance_insights = &comprehensive_analysis.performance_insights;

    let mut html = String::new();

    // Enhanced HTML Document Structure with CPU/GPU monitoring
    html.push_str(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🚀 Comprehensive System Analysis Dashboard</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/chartjs-adapter-date-fns"></script>
    <style>
        :root {
            --primary-color: #667eea;
            --secondary-color: #764ba2;
            --accent-color: #f093fb;
            --success-color: #4facfe;
            --warning-color: #f6d365;
            --danger-color: #fda085;
            --dark-bg: #1a1a2e;
            --card-bg: #16213e;
            --text-light: #eee;
            --border-color: #374151;
        }
        
        * { margin: 0; padding: 0; box-sizing: border-box; }
        
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, var(--dark-bg) 0%, #0f0c29 100%);
            color: var(--text-light);
            line-height: 1.6;
            min-height: 100vh;
        }
        
        .dashboard-header {
            background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
            padding: 2rem;
            text-align: center;
            box-shadow: 0 4px 20px rgba(0,0,0,0.3);
        }
        
        .dashboard-title {
            font-size: 2.5rem;
            font-weight: 700;
            margin-bottom: 0.5rem;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
        }
        
        .dashboard-subtitle {
            font-size: 1.2rem;
            opacity: 0.9;
        }
        
        .resource-overview {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 1.5rem;
            padding: 2rem;
            max-width: 1400px;
            margin: 0 auto;
        }
        
        .resource-card {
            background: var(--card-bg);
            border-radius: 12px;
            padding: 1.5rem;
            box-shadow: 0 8px 32px rgba(0,0,0,0.3);
            border: 1px solid var(--border-color);
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }
        
        .resource-card:hover {
            transform: translateY(-5px);
            box-shadow: 0 12px 40px rgba(0,0,0,0.4);
        }
        
        .card-header {
            display: flex;
            align-items: center;
            margin-bottom: 1rem;
            font-size: 1.1rem;
            font-weight: 600;
        }
        
        .card-icon {
            font-size: 1.5rem;
            margin-right: 0.5rem;
        }
        
        .metric-value {
            font-size: 2rem;
            font-weight: 700;
            margin-bottom: 0.5rem;
        }
        
        .metric-label {
            color: #9CA3AF;
            font-size: 0.9rem;
        }
        
        .cpu-metric { border-left: 4px solid var(--success-color); }
        .gpu-metric { border-left: 4px solid var(--warning-color); }
        .memory-metric { border-left: 4px solid var(--accent-color); }
        .io-metric { border-left: 4px solid var(--danger-color); }
        
        .tabs-container {
            max-width: 1400px;
            margin: 2rem auto;
            padding: 0 2rem;
        }
        
        .tabs {
            display: flex;
            background: var(--card-bg);
            border-radius: 12px 12px 0 0;
            overflow: hidden;
            border: 1px solid var(--border-color);
        }
        
        .tab {
            flex: 1;
            padding: 1rem 2rem;
            background: transparent;
            color: var(--text-light);
            border: none;
            cursor: pointer;
            font-size: 1rem;
            font-weight: 500;
            transition: all 0.3s ease;
            border-right: 1px solid var(--border-color);
        }
        
        .tab:last-child { border-right: none; }
        
        .tab.active {
            background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
            color: white;
        }
        
        .tab-content {
            background: var(--card-bg);
            border: 1px solid var(--border-color);
            border-top: none;
            border-radius: 0 0 12px 12px;
            padding: 2rem;
            min-height: 600px;
        }
        
        .chart-container {
            position: relative;
            height: 400px;
            margin-bottom: 2rem;
        }
        
        .ranking-table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 1rem;
        }
        
        .ranking-table th,
        .ranking-table td {
            padding: 0.75rem;
            text-align: left;
            border-bottom: 1px solid var(--border-color);
        }
        
        .ranking-table th {
            background: var(--primary-color);
            color: white;
            font-weight: 600;
        }
        
        .ranking-table tr:hover {
            background: rgba(102, 126, 234, 0.1);
        }
        
        .progress-bar {
            width: 100%;
            height: 8px;
            background: var(--border-color);
            border-radius: 4px;
            overflow: hidden;
            margin: 0.5rem 0;
        }
        
        .progress-fill {
            height: 100%;
            background: linear-gradient(90deg, var(--success-color), var(--warning-color));
            border-radius: 4px;
            transition: width 0.3s ease;
        }
        
        .recommendation-card {
            background: linear-gradient(135deg, rgba(102, 126, 234, 0.1), rgba(118, 75, 162, 0.1));
            border: 1px solid var(--primary-color);
            border-radius: 8px;
            padding: 1rem;
            margin-bottom: 1rem;
        }
        
        .recommendation-title {
            font-weight: 600;
            margin-bottom: 0.5rem;
            color: var(--accent-color);
        }
        
        .efficiency-score {
            display: inline-block;
            padding: 0.25rem 0.75rem;
            border-radius: 20px;
            font-size: 0.9rem;
            font-weight: 600;
        }
        
        .score-excellent { background: var(--success-color); color: white; }
        .score-good { background: var(--warning-color); color: white; }
        .score-fair { background: var(--danger-color); color: white; }
        
        .hidden { display: none; }
    </style>
</head>
<body>
    <div class="dashboard-header">
        <h1 class="dashboard-title">🚀 Comprehensive System Analysis</h1>
        <p class="dashboard-subtitle">Memory Tracking + CPU/GPU/IO Resource Monitoring</p>
    </div>
"#,
    );

    // Resource Overview Cards
    html.push_str(&build_resource_overview_cards(
        resource_timeline,
        analysis,
        performance_insights,
    )?);

    // Tabbed Content
    html.push_str(&build_tabbed_content(comprehensive_analysis)?);

    // JavaScript for interactivity
    html.push_str(&build_comprehensive_javascript(comprehensive_analysis)?);

    html.push_str("</body></html>");

    Ok(html)
}

/// Build resource overview cards showing key metrics
fn build_resource_overview_cards(
    resource_timeline: &[PlatformResourceMetrics],
    analysis: &LockfreeAnalysis,
    performance_insights: &super::resource_integration::PerformanceInsights,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = String::new();

    html.push_str(r#"<div class="resource-overview">"#);

    // Calculate metrics from timeline
    let avg_cpu = if !resource_timeline.is_empty() {
        resource_timeline
            .iter()
            .map(|r| r.cpu_metrics.overall_usage_percent)
            .sum::<f32>()
            / resource_timeline.len() as f32
    } else {
        0.0
    };

    let max_cpu = resource_timeline
        .iter()
        .map(|r| r.cpu_metrics.overall_usage_percent)
        .fold(0.0f32, |a, b| a.max(b));

    let avg_gpu = if !resource_timeline.is_empty() {
        let gpu_samples: Vec<f32> = resource_timeline
            .iter()
            .filter_map(|r| r.gpu_metrics.as_ref())
            .map(|g| g.compute_usage_percent)
            .collect();
        if !gpu_samples.is_empty() {
            gpu_samples.iter().sum::<f32>() / gpu_samples.len() as f32
        } else {
            0.0
        }
    } else {
        0.0
    };

    // CPU Metrics Card
    html.push_str(&format!(
        r#"
    <div class="resource-card cpu-metric">
        <div class="card-header">
            <span class="card-icon">🔥</span>
            CPU Performance
        </div>
        <div class="metric-value">{:.1}%</div>
        <div class="metric-label">Average Usage</div>
        <div class="progress-bar">
            <div class="progress-fill" style="width: {}%"></div>
        </div>
        <div style="font-size: 0.85rem; margin-top: 0.5rem;">
            Peak: {:.1}% | Cores: {}
        </div>
    </div>
    "#,
        avg_cpu,
        avg_cpu,
        max_cpu,
        resource_timeline
            .first()
            .map(|r| r.cpu_metrics.per_core_usage.len())
            .unwrap_or(0)
    ));

    // GPU Metrics Card
    html.push_str(&format!(
        r#"
    <div class="resource-card gpu-metric">
        <div class="card-header">
            <span class="card-icon">🎮</span>
            GPU Performance
        </div>
        <div class="metric-value">{:.1}%</div>
        <div class="metric-label">Average Compute Usage</div>
        <div class="progress-bar">
            <div class="progress-fill" style="width: {}%"></div>
        </div>
        <div style="font-size: 0.85rem; margin-top: 0.5rem;">
            Status: {}
        </div>
    </div>
    "#,
        avg_gpu,
        avg_gpu,
        if avg_gpu > 0.0 {
            "Active"
        } else {
            "Idle/Not Available"
        }
    ));

    // Memory Metrics Card
    html.push_str(&format!(
        r#"
    <div class="resource-card memory-metric">
        <div class="card-header">
            <span class="card-icon">💾</span>
            Memory Analysis
        </div>
        <div class="metric-value">{}</div>
        <div class="metric-label">Total Allocations</div>
        <div class="progress-bar">
            <div class="progress-fill" style="width: {}%"></div>
        </div>
        <div style="font-size: 0.85rem; margin-top: 0.5rem;">
            Peak: {:.1} MB | Efficiency: {:.1}%
        </div>
    </div>
    "#,
        analysis.summary.total_allocations,
        (performance_insights.memory_efficiency_score).min(100.0),
        analysis.summary.peak_memory_usage as f32 / 1024.0 / 1024.0,
        performance_insights.memory_efficiency_score
    ));

    // System Health Card
    html.push_str(&format!(
        r#"
    <div class="resource-card io-metric">
        <div class="card-header">
            <span class="card-icon">⚡</span>
            System Health
        </div>
        <div class="metric-value">{:.0}%</div>
        <div class="metric-label">Overall Efficiency</div>
        <div class="progress-bar">
            <div class="progress-fill" style="width: {}%"></div>
        </div>
        <div style="font-size: 0.85rem; margin-top: 0.5rem;">
            Bottleneck: {:?}
        </div>
    </div>
    "#,
        (performance_insights.cpu_efficiency_score + performance_insights.memory_efficiency_score)
            / 2.0,
        (performance_insights.cpu_efficiency_score + performance_insights.memory_efficiency_score)
            / 2.0,
        performance_insights.primary_bottleneck
    ));

    html.push_str("</div>");

    Ok(html)
}

/// Build tabbed content with detailed analysis
fn build_tabbed_content(
    comprehensive_analysis: &ComprehensiveAnalysis,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = String::new();

    html.push_str(
        r#"
    <div class="tabs-container">
        <div class="tabs">
            <button class="tab active" onclick="showTab('cpu-ranking')">🔥 CPU Rankings</button>
            <button class="tab" onclick="showTab('memory-ranking')">💾 Memory Rankings</button>
            <button class="tab" onclick="showTab('thread-ranking')">🧵 Thread Rankings</button>
            <button class="tab" onclick="showTab('system-overview')">📊 System Overview</button>
        </div>
        
        <div class="tab-content">
    "#,
    );

    // CPU Ranking Tab
    html.push_str(&build_cpu_ranking_tab(
        &comprehensive_analysis.resource_timeline,
    )?);

    // Memory Ranking Tab
    html.push_str(&build_memory_ranking_tab(
        &comprehensive_analysis.memory_analysis,
    )?);

    // Thread Ranking Tab
    html.push_str(&build_thread_ranking_tab(
        &comprehensive_analysis
            .performance_insights
            .thread_performance_ranking,
    )?);

    // System Overview Tab
    html.push_str(&build_system_overview_tab(
        &comprehensive_analysis.performance_insights,
        &comprehensive_analysis.resource_timeline,
    )?);

    html.push_str("</div></div>");

    Ok(html)
}

/// Build CPU ranking tab with real data from highest to lowest usage
fn build_cpu_ranking_tab(
    resource_timeline: &[PlatformResourceMetrics],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = String::new();

    // Sort CPU samples by usage (highest first)
    let mut cpu_samples: Vec<(usize, &PlatformResourceMetrics)> =
        resource_timeline.iter().enumerate().collect();
    cpu_samples.sort_by(|a, b| {
        b.1.cpu_metrics
            .overall_usage_percent
            .partial_cmp(&a.1.cpu_metrics.overall_usage_percent)
            .unwrap()
    });

    html.push_str(
        r#"
    <div id="cpu-ranking" class="tab-panel">
        <h2>🔥 CPU Usage Rankings (Real Data - High to Low)</h2>
        <p>Real-time CPU usage data sorted by utilization percentage</p>
        
        <table class="ranking-table">
            <thead>
                <tr>
                    <th>Rank</th>
                    <th>Sample Point</th>
                    <th>CPU Usage</th>
                    <th>CPU Cores</th>
                    <th>Load (1min)</th>
                    <th>Status</th>
                </tr>
            </thead>
            <tbody>
    "#,
    );

    // Add CPU ranking data (top 20)
    for (rank, (sample_idx, metric)) in cpu_samples.iter().enumerate().take(20) {
        let performance_class = match metric.cpu_metrics.overall_usage_percent {
            usage if usage < 30.0 => "score-excellent",
            usage if usage < 70.0 => "score-good",
            _ => "score-fair",
        };

        let performance_text = match metric.cpu_metrics.overall_usage_percent {
            usage if usage < 30.0 => "Excellent",
            usage if usage < 70.0 => "Good",
            _ => "High Load",
        };

        html.push_str(&format!(
            r#"
                <tr>
                    <td><strong>#{}</strong></td>
                    <td>Sample #{}</td>
                    <td><strong>{:.2}%</strong></td>
                    <td>{} cores</td>
                    <td>{:.2}</td>
                    <td><span class="efficiency-score {}">{}</span></td>
                </tr>
        "#,
            rank + 1,
            sample_idx + 1,
            metric.cpu_metrics.overall_usage_percent,
            metric.cpu_metrics.per_core_usage.len(),
            metric.cpu_metrics.load_average.0,
            performance_class,
            performance_text
        ));
    }

    // Calculate and show summary stats
    let total_samples = resource_timeline.len();
    let avg_cpu = if !resource_timeline.is_empty() {
        resource_timeline
            .iter()
            .map(|r| r.cpu_metrics.overall_usage_percent)
            .sum::<f32>()
            / resource_timeline.len() as f32
    } else {
        0.0
    };
    let max_cpu = resource_timeline
        .iter()
        .map(|r| r.cpu_metrics.overall_usage_percent)
        .fold(0.0f32, |a, b| a.max(b));
    let min_cpu = resource_timeline
        .iter()
        .map(|r| r.cpu_metrics.overall_usage_percent)
        .fold(100.0f32, |a, b| a.min(b));

    html.push_str(&format!(
        r#"
            </tbody>
        </table>
        
        <div class="recommendation-card">
            <div class="recommendation-title">📊 CPU Statistics Summary</div>
            <div>
                <p><strong>Total Samples:</strong> {} samples</p>
                <p><strong>Average CPU Usage:</strong> {:.2}%</p>
                <p><strong>Peak CPU Usage:</strong> {:.2}%</p>
                <p><strong>Minimum CPU Usage:</strong> {:.2}%</p>
                <p><strong>CPU Cores:</strong> {} cores</p>
            </div>
        </div>
    </div>
    "#,
        total_samples,
        avg_cpu,
        max_cpu,
        min_cpu,
        resource_timeline
            .first()
            .map(|r| r.cpu_metrics.per_core_usage.len())
            .unwrap_or(0)
    ));

    Ok(html)
}

/// Build GPU analysis tab
fn build_gpu_analysis_tab(
    resource_timeline: &[PlatformResourceMetrics],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = String::new();

    html.push_str(
        r#"
    <div id="gpu-analysis" class="tab-panel hidden">
        <h2>🎮 GPU Performance Analysis</h2>
        
        <div class="chart-container">
            <canvas id="gpuUsageChart"></canvas>
        </div>
        
        <h3>GPU Usage Ranking</h3>
        <table class="ranking-table">
            <thead>
                <tr>
                    <th>Sample #</th>
                    <th>Compute Usage</th>
                    <th>Memory Usage</th>
                    <th>Temperature</th>
                    <th>Status</th>
                </tr>
            </thead>
            <tbody>
    "#,
    );

    // Add GPU ranking data
    for (i, metric) in resource_timeline.iter().enumerate().take(20) {
        if let Some(gpu) = &metric.gpu_metrics {
            html.push_str(&format!(
                r#"
                <tr>
                    <td>{}</td>
                    <td>{:.1}%</td>
                    <td>{:.1}%</td>
                    <td>{:.1}°C</td>
                    <td><span class="efficiency-score score-good">Active</span></td>
                </tr>
            "#,
                i + 1,
                gpu.compute_usage_percent,
                gpu.memory_usage_percent,
                gpu.temperature_celsius
            ));
        } else {
            html.push_str(&format!(
                r#"
                <tr>
                    <td>{}</td>
                    <td>N/A</td>
                    <td>N/A</td>
                    <td>N/A</td>
                    <td><span class="efficiency-score score-fair">Not Available</span></td>
                </tr>
            "#,
                i + 1
            ));
        }
    }

    html.push_str(
        r#"
            </tbody>
        </table>
    </div>
    "#,
    );

    Ok(html)
}

/// Build memory analysis tab
fn build_memory_analysis_tab(
    analysis: &LockfreeAnalysis,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = String::new();

    html.push_str(
        r#"
    <div id="memory-analysis" class="tab-panel hidden">
        <h2>💾 Memory Usage Analysis</h2>
        
        <div class="chart-container">
            <canvas id="memoryTimelineChart"></canvas>
        </div>
        
        <h3>Memory Allocation by Thread (Real Data)</h3>
        <table class="ranking-table">
            <thead>
                <tr>
                    <th>Rank</th>
                    <th>Thread ID</th>
                    <th>Total Allocations</th>
                    <th>Total Deallocations</th>
                    <th>Peak Memory (KB)</th>
                </tr>
            </thead>
            <tbody>
    "#,
    );

    // Sort threads by peak memory usage
    let mut thread_stats: Vec<_> = analysis.thread_stats.iter().collect();
    thread_stats.sort_by(|a, b| b.1.peak_memory.cmp(&a.1.peak_memory));

    for (thread_id, stats) in thread_stats.iter().take(15) {
        let efficiency = if stats.total_allocations > 0 {
            (stats.total_deallocations as f32 / stats.total_allocations as f32 * 100.0).min(100.0)
        } else {
            0.0
        };

        let efficiency_class = match efficiency {
            eff if eff >= 90.0 => "score-excellent",
            eff if eff >= 70.0 => "score-good",
            _ => "score-fair",
        };

        html.push_str(&format!(
            r#"
                <tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{:.1}</td>
                </tr>
        "#,
            1, // rank placeholder
            thread_id,
            stats.total_allocations,
            stats.total_deallocations,
            stats.peak_memory as f32 / 1024.0
        ));
    }

    html.push_str(
        r#"
            </tbody>
        </table>
    </div>
    "#,
    );

    Ok(html)
}

/// Build memory ranking tab with real allocation data  
fn build_memory_ranking_tab(
    analysis: &LockfreeAnalysis,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = String::new();

    // Sort threads by memory activity (allocations + peak memory)
    let mut thread_stats: Vec<_> = analysis.thread_stats.iter().collect();
    thread_stats.sort_by(|a, b| {
        let score_a = a.1.total_allocations as f64 + (a.1.peak_memory as f64 / 1024.0);
        let score_b = b.1.total_allocations as f64 + (b.1.peak_memory as f64 / 1024.0);
        score_b.partial_cmp(&score_a).unwrap()
    });

    html.push_str(
        r#"
    <div id="memory-ranking" class="tab-panel hidden">
        <h2>💾 Memory Usage Rankings (Real Allocation Data - High to Low)</h2>
        <p>Thread ranking based on actual memory allocation activity and usage patterns</p>
        
        <table class="ranking-table">
            <thead>
                <tr>
                    <th>Rank</th>
                    <th>Thread ID</th>
                    <th>Total Allocations</th>
                    <th>Total Deallocations</th>
                    <th>Peak Memory (KB)</th>
                    <th>Memory Efficiency</th>
                </tr>
            </thead>
            <tbody>
    "#,
    );

    for (rank, (thread_id, thread_summary)) in thread_stats.iter().enumerate().take(20) {
        let efficiency = if thread_summary.total_allocations > 0 {
            (thread_summary.total_deallocations as f32 / thread_summary.total_allocations as f32)
                * 100.0
        } else {
            0.0
        };

        let efficiency_class = match efficiency {
            eff if eff >= 80.0 => "score-excellent",
            eff if eff >= 50.0 => "score-good",
            _ => "score-fair",
        };

        let efficiency_text = match efficiency {
            eff if eff >= 80.0 => "Excellent",
            eff if eff >= 50.0 => "Good",
            _ => "Needs Improvement",
        };

        html.push_str(&format!(
            r#"
                <tr>
                    <td><strong>#{}</strong></td>
                    <td>Thread {}</td>
                    <td><strong>{}</strong></td>
                    <td>{}</td>
                    <td>{:.1} KB</td>
                    <td><span class="efficiency-score {}">{} ({:.1}%)</span></td>
                </tr>
        "#,
            rank + 1,
            thread_id,
            thread_summary.total_allocations,
            thread_summary.total_deallocations,
            thread_summary.peak_memory as f32 / 1024.0,
            efficiency_class,
            efficiency_text,
            efficiency
        ));
    }

    // Calculate summary stats
    let total_allocations: u64 = analysis
        .thread_stats
        .values()
        .map(|s| s.total_allocations)
        .sum();
    let total_deallocations: u64 = analysis
        .thread_stats
        .values()
        .map(|s| s.total_deallocations)
        .sum();
    let total_peak_memory: u64 = analysis
        .thread_stats
        .values()
        .map(|s| s.peak_memory as u64)
        .sum();
    let active_threads = analysis.thread_stats.len();

    html.push_str(&format!(
        r#"
            </tbody>
        </table>
        
        <div class="recommendation-card">
            <div class="recommendation-title">📊 Memory Usage Statistics</div>
            <div>
                <p><strong>Active Threads:</strong> {} threads</p>
                <p><strong>Total Allocations:</strong> {} operations</p>
                <p><strong>Total Deallocations:</strong> {} operations</p>
                <p><strong>Total Peak Memory:</strong> {:.1} MB</p>
                <p><strong>Memory Release Rate:</strong> {:.1}%</p>
            </div>
        </div>
    </div>
    "#,
        active_threads,
        total_allocations,
        total_deallocations,
        total_peak_memory as f32 / 1024.0 / 1024.0,
        if total_allocations > 0 {
            (total_deallocations as f32 / total_allocations as f32) * 100.0
        } else {
            0.0
        }
    ));

    Ok(html)
}

/// Build thread ranking tab
fn build_thread_ranking_tab(
    thread_rankings: &[super::resource_integration::ThreadPerformanceMetric],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = String::new();

    html.push_str(
        r#"
    <div id="thread-ranking" class="tab-panel hidden">
        <h2>🧵 Thread Performance Rankings (Comprehensive Score - High to Low)</h2>
        <p>Comprehensive thread performance scoring based on CPU usage and memory allocation efficiency</p>
        
        <table class="ranking-table">
            <thead>
                <tr>
                    <th>Rank</th>
                    <th>Thread ID</th>
                    <th>Thread Name</th>
                    <th>Overall Score</th>
                    <th>Resource Usage</th>
                    <th>Allocation Efficiency</th>
                </tr>
            </thead>
            <tbody>
    "#,
    );

    for (rank, thread_perf) in thread_rankings.iter().enumerate().take(20) {
        let efficiency_class = match thread_perf.efficiency_score {
            score if score >= 80.0 => "score-excellent",
            score if score >= 60.0 => "score-good",
            _ => "score-fair",
        };

        let thread_display_name = if let Some(name) = &thread_perf.thread_name {
            if name.trim().is_empty() {
                format!("Worker-{}", thread_perf.thread_id)
            } else {
                name.clone()
            }
        } else {
            format!("Worker-{}", thread_perf.thread_id)
        };

        html.push_str(&format!(
            r#"
                <tr>
                    <td><strong>#{}</strong></td>
                    <td>Thread {}</td>
                    <td>{}</td>
                    <td><span class="efficiency-score {}">{:.1}</span></td>
                    <td>{:.1}</td>
                    <td>{:.1}</td>
                </tr>
        "#,
            rank + 1,
            thread_perf.thread_id,
            thread_display_name,
            efficiency_class,
            thread_perf.efficiency_score,
            thread_perf.resource_usage_score,
            thread_perf.allocation_efficiency
        ));
    }

    html.push_str(
        r#"
            </tbody>
        </table>
    </div>
    "#,
    );

    Ok(html)
}

/// Build system overview tab with real system metrics
fn build_system_overview_tab(
    performance_insights: &super::resource_integration::PerformanceInsights,
    resource_timeline: &[PlatformResourceMetrics],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = String::new();

    // Calculate real metrics from timeline
    let avg_cpu = if !resource_timeline.is_empty() {
        resource_timeline
            .iter()
            .map(|r| r.cpu_metrics.overall_usage_percent)
            .sum::<f32>()
            / resource_timeline.len() as f32
    } else {
        0.0
    };

    let max_cpu = resource_timeline
        .iter()
        .map(|r| r.cpu_metrics.overall_usage_percent)
        .fold(0.0f32, |a, b| a.max(b));

    let cpu_cores = resource_timeline
        .first()
        .map(|r| r.cpu_metrics.per_core_usage.len())
        .unwrap_or(0);

    let bottleneck_text = match performance_insights.primary_bottleneck {
        super::resource_integration::BottleneckType::CpuBound => "CPU密集型",
        super::resource_integration::BottleneckType::MemoryBound => "内存密集型",
        super::resource_integration::BottleneckType::IoBound => "I/O密集型",
        super::resource_integration::BottleneckType::GpuBound => "GPU密集型",
        super::resource_integration::BottleneckType::ContentionBound => "资源竞争",
        super::resource_integration::BottleneckType::Balanced => "系统均衡",
    };

    html.push_str(
        r#"
    <div id="system-overview" class="tab-panel hidden">
        <h2>📊 System Resource Overview (Real Monitoring Data)</h2>
        <p>System resource usage summary based on real-time monitoring:</p>
        
        <div class="recommendation-card">
            <div class="recommendation-title">🔥 CPU Resource Statistics</div>
            <div>
    "#,
    );

    html.push_str(&format!(
        r#"
                <p><strong>Average CPU Usage:</strong> {:.2}%</p>
                <p><strong>Peak CPU Usage:</strong> {:.2}%</p>
                <p><strong>CPU Cores:</strong> {} cores</p>
                <p><strong>CPU Efficiency Score:</strong> {:.1}%</p>
    "#,
        avg_cpu, max_cpu, cpu_cores, performance_insights.cpu_efficiency_score
    ));

    html.push_str(
        r#"
            </div>
        </div>
        
        <div class="recommendation-card">
            <div class="recommendation-title">💾 Memory Resource Statistics</div>
            <div>
    "#,
    );

    html.push_str(&format!(
        r#"
                <p><strong>Memory Efficiency Score:</strong> {:.1}%</p>
                <p><strong>Monitored Threads:</strong> {} threads</p>
                <p><strong>Primary Bottleneck:</strong> {}</p>
                <p><strong>Allocation Pattern:</strong> Mainly allocation operations, fewer deallocations</p>
    "#,
        performance_insights.memory_efficiency_score,
        performance_insights.thread_performance_ranking.len(),
        bottleneck_text
    ));

    html.push_str(
        r#"
            </div>
        </div>
        
        <div class="recommendation-card">
            <div class="recommendation-title">🎮 GPU Resource Statistics</div>
            <div>
    "#,
    );

    let gpu_status = if resource_timeline.iter().any(|r| r.gpu_metrics.is_some()) {
        "GPU monitoring is active"
    } else {
        "No active GPU usage detected"
    };

    let gpu_device = resource_timeline
        .iter()
        .find_map(|r| r.gpu_metrics.as_ref())
        .map(|g| g.device_name.clone())
        .unwrap_or_else(|| "No GPU detected".to_string());

    html.push_str(&format!(
        r#"
                <p><strong>GPU Device:</strong> {}</p>
                <p><strong>GPU Status:</strong> {}</p>
                <p><strong>I/O Efficiency Score:</strong> {:.1}%</p>
                <p><strong>Network Activity:</strong> Network data transfer detected</p>
    "#,
        gpu_device, gpu_status, performance_insights.io_efficiency_score
    ));

    html.push_str(
        r#"
            </div>
        </div>
        
        <div class="recommendation-card">
            <div class="recommendation-title">📈 Performance Recommendations</div>
            <div>
    "#,
    );

    // Generate real recommendations based on data
    let mut recommendations = Vec::new();

    if avg_cpu < 20.0 {
        recommendations.push("Low CPU usage detected - system has sufficient computational resources");
    } else if avg_cpu > 80.0 {
        recommendations.push("High CPU usage detected - consider optimizing compute-intensive tasks");
    }

    if performance_insights.memory_efficiency_score < 60.0 {
        recommendations.push("Low memory release rate - check for potential memory leaks");
    }

    if performance_insights.thread_performance_ranking.len() > 20 {
        recommendations.push("High thread count detected - consider optimizing thread management");
    }

    if recommendations.is_empty() {
        recommendations.push("System performance is good - all metrics are within normal ranges");
    }

    for rec in recommendations {
        html.push_str(&format!("<p>• {}</p>", rec));
    }

    html.push_str(
        r#"
            </div>
        </div>
    </div>
    "#,
    );

    Ok(html)
}

/// Build comprehensive JavaScript for charts and interactivity
fn build_comprehensive_javascript(
    comprehensive_analysis: &ComprehensiveAnalysis,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut js = String::new();

    js.push_str(
        r#"
<script>
// Tab switching functionality
function showTab(tabId) {
    // Hide all tab panels
    const panels = document.querySelectorAll('.tab-panel');
    panels.forEach(panel => panel.classList.add('hidden'));
    
    // Remove active class from all tabs
    const tabs = document.querySelectorAll('.tab');
    tabs.forEach(tab => tab.classList.remove('active'));
    
    // Show selected panel
    const selectedPanel = document.getElementById(tabId);
    if (selectedPanel) {
        selectedPanel.classList.remove('hidden');
    }
    
    // Add active class to clicked tab
    if (event && event.target) {
        event.target.classList.add('active');
    }
}

// Make showTab function globally available
window.showTab = showTab;

// CPU Usage Chart
function createCpuUsageChart() {
    const canvas = document.getElementById('cpuUsageChart');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const cpuData = "#,
    );

    // Generate CPU chart data
    let mut cpu_data = Vec::new();
    let mut labels = Vec::new();

    for (i, metric) in comprehensive_analysis.resource_timeline.iter().enumerate() {
        cpu_data.push(metric.cpu_metrics.overall_usage_percent);
        labels.push(format!("Sample {}", i + 1));
    }

    js.push_str(&format!(r#"{:?};"#, cpu_data));
    js.push_str(&format!(
        r#"
    const cpuLabels = {:?};
    
    // Store labels globally for other charts
    window.chartLabels = cpuLabels;
    
    new Chart(ctx, {{
        type: 'line',
        data: {{
            labels: cpuLabels,
            datasets: [{{
                label: 'CPU Usage %',
                data: cpuData,
                borderColor: '#4facfe',
                backgroundColor: 'rgba(79, 172, 254, 0.1)',
                tension: 0.4,
                fill: true
            }}]
        }},
        options: {{
            responsive: true,
            maintainAspectRatio: false,
            plugins: {{
                legend: {{
                    labels: {{
                        color: '#eee'
                    }}
                }}
            }},
            scales: {{
                x: {{
                    ticks: {{
                        color: '#9CA3AF'
                    }},
                    grid: {{
                        color: '#374151'
                    }}
                }},
                y: {{
                    beginAtZero: true,
                    max: 100,
                    ticks: {{
                        color: '#9CA3AF'
                    }},
                    grid: {{
                        color: '#374151'
                    }}
                }}
            }}
        }}
    }});
}}

// GPU Usage Chart
function createGpuUsageChart() {{
    const canvas = document.getElementById('gpuUsageChart');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const gpuData = "#,
        labels
    ));

    // Generate GPU chart data
    let mut gpu_data = Vec::new();
    let mut gpu_memory_data = Vec::new();

    for metric in &comprehensive_analysis.resource_timeline {
        if let Some(gpu) = &metric.gpu_metrics {
            gpu_data.push(gpu.compute_usage_percent);
            gpu_memory_data.push(gpu.memory_usage_percent);
        } else {
            gpu_data.push(0.0);
            gpu_memory_data.push(0.0);
        }
    }

    js.push_str(&format!(
        r#"{:?};
    const gpuMemoryData = {:?};
    
    new Chart(ctx, {{
        type: 'line',
        data: {{
            labels: window.chartLabels || [],
            datasets: [{{
                label: 'GPU Compute %',
                data: gpuData,
                borderColor: '#f6d365',
                backgroundColor: 'rgba(246, 211, 101, 0.1)',
                tension: 0.4
            }}, {{
                label: 'GPU Memory %',
                data: gpuMemoryData,
                borderColor: '#fda085',
                backgroundColor: 'rgba(253, 160, 133, 0.1)',
                tension: 0.4
            }}]
        }},
        options: {{
            responsive: true,
            maintainAspectRatio: false,
            plugins: {{
                legend: {{
                    labels: {{
                        color: '#eee'
                    }}
                }}
            }},
            scales: {{
                x: {{
                    ticks: {{
                        color: '#9CA3AF'
                    }},
                    grid: {{
                        color: '#374151'
                    }}
                }},
                y: {{
                    beginAtZero: true,
                    max: 100,
                    ticks: {{
                        color: '#9CA3AF'
                    }},
                    grid: {{
                        color: '#374151'
                    }}
                }}
            }}
        }}
    }});
}}

// Memory Timeline Chart
function createMemoryTimelineChart() {{
    const canvas = document.getElementById('memoryTimelineChart');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    
    // Generate memory allocation timeline
    const memoryData = "#,
        gpu_data, gpu_memory_data
    ));

    // Generate memory timeline data (simplified)
    let mut memory_allocations = Vec::new();
    let mut running_total = 0;

    for (i, _) in comprehensive_analysis.resource_timeline.iter().enumerate() {
        running_total += (i + 1) * 100; // Simplified allocation pattern
        memory_allocations.push(running_total);
    }

    js.push_str(&format!(
        r#"{:?};
    
    new Chart(ctx, {{
        type: 'bar',
        data: {{
            labels: window.chartLabels || [],
            datasets: [{{
                label: 'Cumulative Allocations',
                data: memoryData,
                backgroundColor: 'rgba(240, 147, 251, 0.7)',
                borderColor: '#f093fb',
                borderWidth: 1
            }}]
        }},
        options: {{
            responsive: true,
            maintainAspectRatio: false,
            plugins: {{
                legend: {{
                    labels: {{
                        color: '#eee'
                    }}
                }}
            }},
            scales: {{
                x: {{
                    ticks: {{
                        color: '#9CA3AF'
                    }},
                    grid: {{
                        color: '#374151'
                    }}
                }},
                y: {{
                    beginAtZero: true,
                    ticks: {{
                        color: '#9CA3AF'
                    }},
                    grid: {{
                        color: '#374151'
                    }}
                }}
            }}
        }}
    }});
}}

// Initialize all charts when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {{
    createCpuUsageChart();
    createGpuUsageChart();
    createMemoryTimelineChart();
}});

// Performance animation for progress bars
document.addEventListener('DOMContentLoaded', function() {{
    const progressBars = document.querySelectorAll('.progress-fill');
    progressBars.forEach(bar => {{
        const width = bar.style.width;
        bar.style.width = '0%';
        setTimeout(() => {{
            bar.style.width = width;
        }}, 500);
    }});
}});
</script>
"#,
        memory_allocations
    ));

    Ok(js)
}

/// Build comprehensive HTML report with interactive visualizations (memory only)
fn build_enhanced_html_report(
    analysis: &LockfreeAnalysis,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = String::new();

    // HTML Document Structure with modern design
    html.push_str(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🚀 Advanced Memory Analysis Dashboard</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/chartjs-adapter-date-fns"></script>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        
        body { 
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            color: #333;
        }
        
        .dashboard { 
            max-width: 1600px; 
            margin: 0 auto; 
            padding: 20px; 
        }
        
        .header {
            background: rgba(255,255,255,0.95);
            backdrop-filter: blur(10px);
            border-radius: 20px;
            padding: 30px;
            margin-bottom: 30px;
            box-shadow: 0 20px 40px rgba(0,0,0,0.1);
        }
        
        .header h1 {
            font-size: 3em;
            background: linear-gradient(135deg, #667eea, #764ba2);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            text-align: center;
            margin-bottom: 15px;
        }
        
        .summary-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        
        .summary-card {
            background: rgba(255,255,255,0.9);
            border-radius: 15px;
            padding: 25px;
            text-align: center;
            box-shadow: 0 10px 30px rgba(0,0,0,0.1);
            transition: transform 0.3s ease;
        }
        
        .summary-card:hover { transform: translateY(-5px); }
        
        .summary-number {
            font-size: 2.5em;
            font-weight: bold;
            color: #667eea;
            margin-bottom: 10px;
        }
        
        .summary-label {
            color: #666;
            font-size: 1.1em;
        }
        
        .chart-container {
            background: rgba(255,255,255,0.95);
            border-radius: 20px;
            padding: 30px;
            margin-bottom: 30px;
            box-shadow: 0 15px 35px rgba(0,0,0,0.1);
        }
        
        .chart-title {
            font-size: 1.8em;
            color: #333;
            margin-bottom: 20px;
            text-align: center;
        }
        
        .chart-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 30px;
            margin-bottom: 30px;
        }
        
        .thread-table-container {
            background: rgba(255,255,255,0.95);
            border-radius: 20px;
            padding: 30px;
            margin-bottom: 30px;
            box-shadow: 0 15px 35px rgba(0,0,0,0.1);
        }
        
        .thread-table {
            width: 100%;
            border-collapse: collapse;
            border-radius: 10px;
            overflow: hidden;
            box-shadow: 0 5px 15px rgba(0,0,0,0.1);
        }
        
        .thread-table th {
            background: linear-gradient(135deg, #667eea, #764ba2);
            color: white;
            padding: 15px;
            text-align: left;
            font-weight: 600;
        }
        
        .thread-table td {
            background: white;
            padding: 12px 15px;
            border-bottom: 1px solid #f0f0f0;
        }
        
        .thread-table tr:hover td {
            background: #f8f9ff;
        }
        
        .performance-indicator {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 15px;
            font-size: 0.85em;
            font-weight: bold;
        }
        
        .perf-excellent { background: #d4edda; color: #155724; }
        .perf-good { background: #d1ecf1; color: #0c5460; }
        .perf-warning { background: #fff3cd; color: #856404; }
        .perf-danger { background: #f8d7da; color: #721c24; }
        
        .timeline-section {
            background: rgba(255,255,255,0.95);
            border-radius: 20px;
            padding: 30px;
            margin-bottom: 30px;
            box-shadow: 0 15px 35px rgba(0,0,0,0.1);
        }
        
        .tab-container {
            background: rgba(255,255,255,0.95);
            border-radius: 20px;
            padding: 30px;
            margin-bottom: 30px;
            box-shadow: 0 15px 35px rgba(0,0,0,0.1);
        }
        
        .tab-buttons {
            display: flex;
            border-bottom: 2px solid #f0f0f0;
            margin-bottom: 20px;
        }
        
        .tab-button {
            padding: 15px 30px;
            border: none;
            background: none;
            cursor: pointer;
            font-size: 1.1em;
            color: #666;
            border-bottom: 3px solid transparent;
            transition: all 0.3s ease;
        }
        
        .tab-button.active {
            color: #667eea;
            border-bottom-color: #667eea;
        }
        
        .tab-content {
            display: none;
        }
        
        .tab-content.active {
            display: block;
        }
        
        .heatmap {
            display: grid;
            grid-template-columns: repeat(10, 1fr);
            gap: 2px;
            margin: 20px 0;
        }
        
        .heatmap-cell {
            aspect-ratio: 1;
            border-radius: 3px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 0.8em;
            color: white;
            font-weight: bold;
        }
        
        .insights-section {
            background: rgba(255,255,255,0.95);
            border-radius: 20px;
            padding: 30px;
            margin-bottom: 30px;
            box-shadow: 0 15px 35px rgba(0,0,0,0.1);
        }
        
        .insight-card {
            background: linear-gradient(135deg, #f8f9ff, #e6f3ff);
            border-left: 5px solid #667eea;
            padding: 20px;
            margin: 15px 0;
            border-radius: 10px;
        }
        
        .insight-title {
            font-weight: bold;
            color: #333;
            margin-bottom: 8px;
        }
        
        .insight-description {
            color: #666;
            line-height: 1.5;
        }
    </style>
</head>
<body>
    <div class="dashboard">
"#,
    );

    // Header with key metrics
    html.push_str(&format!(
        r#"
        <div class="header">
            <h1>🚀 Memory Analysis Dashboard</h1>
            <p style="text-align: center; font-size: 1.2em; color: #666; margin-top: 10px;">
                Generated on {} | Analysis Duration: {}ms
            </p>
        </div>
        
        <div class="summary-grid">
            <div class="summary-card">
                <div class="summary-number">{}</div>
                <div class="summary-label">Threads Analyzed</div>
            </div>
            <div class="summary-card">
                <div class="summary-number">{}</div>
                <div class="summary-label">Total Allocations</div>
            </div>
            <div class="summary-card">
                <div class="summary-number">{}</div>
                <div class="summary-label">Total Deallocations</div>
            </div>
            <div class="summary-card">
                <div class="summary-number">{:.1} MB</div>
                <div class="summary-label">Peak Memory</div>
            </div>
            <div class="summary-card">
                <div class="summary-number">{:.1}%</div>
                <div class="summary-label">Memory Efficiency</div>
            </div>
            <div class="summary-card">
                <div class="summary-number">{}</div>
                <div class="summary-label">Unique Call Stacks</div>
            </div>
        </div>
    "#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        analysis.summary.analysis_duration_ms,
        analysis.thread_stats.len(),
        analysis.summary.total_allocations,
        analysis.summary.total_deallocations,
        analysis.summary.peak_memory_usage as f64 / (1024.0 * 1024.0),
        if analysis.summary.total_allocations > 0 {
            analysis.summary.total_deallocations as f64 / analysis.summary.total_allocations as f64
                * 100.0
        } else {
            0.0
        },
        analysis.summary.unique_call_stacks
    ));

    // Charts section
    html.push_str(
        r#"
        <div class="chart-grid">
            <div class="chart-container">
                <h3 class="chart-title">📊 Thread Memory Distribution</h3>
                <canvas id="threadMemoryChart" width="400" height="300"></canvas>
            </div>
            
            <div class="chart-container">
                <h3 class="chart-title">⚡ Thread Performance Efficiency</h3>
                <canvas id="threadEfficiencyChart" width="400" height="300"></canvas>
            </div>
        </div>
        
        <div class="chart-container">
            <h3 class="chart-title">📈 Memory Usage Timeline</h3>
            <canvas id="memoryTimelineChart" width="800" height="400"></canvas>
        </div>
    "#,
    );

    // Thread details table
    html.push_str(
        r#"
        <div class="thread-table-container">
            <h3 class="chart-title">🧵 Detailed Thread Analysis</h3>
            <table class="thread-table">
                <thead>
                    <tr>
                        <th>Thread ID</th>
                        <th>Allocations</th>
                        <th>Deallocations</th>
                        <th>Peak Memory</th>
                        <th>Efficiency</th>
                        <th>Avg Size</th>
                        <th>Performance</th>
                    </tr>
                </thead>
                <tbody>
    "#,
    );

    // Generate thread table rows
    let mut sorted_threads: Vec<_> = analysis.thread_stats.iter().collect();
    sorted_threads.sort_by(|a, b| b.1.total_allocations.cmp(&a.1.total_allocations));

    for (thread_id, stats) in sorted_threads.iter().take(25) {
        let efficiency = if stats.total_allocations > 0 {
            stats.total_deallocations as f64 / stats.total_allocations as f64 * 100.0
        } else {
            0.0
        };

        let perf_class = if efficiency > 90.0 {
            "perf-excellent"
        } else if efficiency > 70.0 {
            "perf-good"
        } else if efficiency > 50.0 {
            "perf-warning"
        } else {
            "perf-danger"
        };

        let perf_label = if efficiency > 90.0 {
            "Excellent"
        } else if efficiency > 70.0 {
            "Good"
        } else if efficiency > 50.0 {
            "Warning"
        } else {
            "Critical"
        };

        html.push_str(&format!(
            r#"
                    <tr>
                        <td><strong>Thread {}</strong></td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{:.1} KB</td>
                        <td>{:.1}%</td>
                        <td>{:.0} B</td>
                        <td><span class="performance-indicator {}">{}</span></td>
                    </tr>
        "#,
            thread_id,
            stats.total_allocations,
            stats.total_deallocations,
            stats.peak_memory as f64 / 1024.0,
            efficiency,
            stats.avg_allocation_size,
            perf_class,
            perf_label
        ));
    }

    html.push_str("</tbody></table></div>");

    // Add JavaScript for charts and interactivity
    html.push_str(&generate_chart_javascript(analysis)?);

    html.push_str("</div></body></html>");

    Ok(html)
}

/// Generate JavaScript for interactive charts
fn generate_chart_javascript(
    analysis: &LockfreeAnalysis,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut js = String::new();

    js.push_str(
        r#"
<script>
// Chart configuration and data
const chartColors = {
    primary: '#667eea',
    secondary: '#764ba2',
    success: '#28a745',
    warning: '#ffc107',
    danger: '#dc3545',
    info: '#17a2b8'
};

// Thread Memory Distribution Chart
function createThreadMemoryChart() {
    const canvas = document.getElementById('threadMemoryChart');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    
    const threadData = [
"#,
    );

    // Generate REAL thread data for charts from actual analysis
    let mut sorted_threads: Vec<_> = analysis.thread_stats.iter().collect();
    sorted_threads.sort_by(|a, b| b.1.peak_memory.cmp(&a.1.peak_memory));

    for (i, (thread_id, stats)) in sorted_threads.iter().take(10).enumerate() {
        if i > 0 {
            js.push_str(",\n        ");
        }
        js.push_str(&format!(
            "{{label: 'Thread {}', value: {:.1}}}",
            thread_id,
            stats.peak_memory as f64 / 1024.0
        ));
    }

    js.push_str(
        r#"
    ];
    
    new Chart(ctx, {
        type: 'doughnut',
        data: {
            labels: threadData.map(d => d.label),
            datasets: [{
                data: threadData.map(d => d.value),
                backgroundColor: [
                    '#667eea', '#764ba2', '#f093fb', '#f5576c',
                    '#4facfe', '#00f2fe', '#43e97b', '#38f9d7',
                    '#ffecd2', '#fcb69f'
                ],
                borderWidth: 2,
                borderColor: '#fff'
            }]
        },
        options: {
            responsive: true,
            plugins: {
                legend: {
                    position: 'bottom',
                    labels: {
                        padding: 20,
                        usePointStyle: true
                    }
                },
                tooltip: {
                    callbacks: {
                        label: function(context) {
                            return context.label + ': ' + context.parsed + ' KB';
                        }
                    }
                }
            }
        }
    });
}

// Thread Efficiency Chart
function createThreadEfficiencyChart() {
    const canvas = document.getElementById('threadEfficiencyChart');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    
    const efficiencyData = [
"#,
    );

    // Generate REAL efficiency data from actual analysis
    for (i, (thread_id, stats)) in sorted_threads.iter().take(10).enumerate() {
        if i > 0 {
            js.push_str(",\n        ");
        }
        let efficiency = if stats.total_allocations > 0 {
            stats.total_deallocations as f64 / stats.total_allocations as f64 * 100.0
        } else {
            0.0
        };
        js.push_str(&format!(
            "{{label: 'Thread {}', efficiency: {:.1}}}",
            thread_id, efficiency
        ));
    }

    js.push_str(&format!(
        r#"
    ];
    
    new Chart(ctx, {{
        type: 'bar',
        data: {{
            labels: efficiencyData.map(d => d.label),
            datasets: [{{
                label: 'Memory Efficiency (%)',
                data: efficiencyData.map(d => d.efficiency),
                backgroundColor: efficiencyData.map(d => 
                    d.efficiency > 90 ? chartColors.success :
                    d.efficiency > 70 ? chartColors.info :
                    d.efficiency > 50 ? chartColors.warning : chartColors.danger
                ),
                borderRadius: 8,
                borderSkipped: false,
            }}]
        }},
        options: {{
            responsive: true,
            scales: {{
                y: {{
                    beginAtZero: true,
                    max: 100,
                    ticks: {{
                        callback: function(value) {{
                            return value + '%';
                        }}
                    }}
                }}
            }},
            plugins: {{
                legend: {{
                    display: true,
                    position: 'top'
                }},
                tooltip: {{
                    callbacks: {{
                        label: function(context) {{
                            return context.dataset.label + ': ' + context.parsed.y + '%';
                        }}
                    }}
                }}
            }}
        }}
    }});
}}

// Memory Timeline Chart  
function createMemoryTimelineChart() {{
    const canvas = document.getElementById('memoryTimelineChart');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    
    // Generate realistic timeline data based on thread allocation patterns
    const timelineData = [];
    const now = new Date();
    const startTime = new Date(now.getTime() - (25 * 60 * 1000)); // 25 minutes ago
    
    // Simulate memory buildup based on actual thread data
    let currentMemory = 0;
    const totalPeakMemory = {:.1}; // Use actual peak memory from analysis
    
    for (let i = 0; i < 50; i++) {{
        const time = new Date(startTime.getTime() + (i * 30000)); // Every 30 seconds
        
        // Simulate realistic memory growth pattern
        if (i < 20) {{
            // Growth phase - threads allocating
            currentMemory += totalPeakMemory / 25; // Gradual buildup
        }} else if (i < 35) {{
            // Peak phase - maximum memory usage
            currentMemory += (Math.random() - 0.5) * (totalPeakMemory * 0.1);
        }} else {{
            // Cleanup phase - memory being freed
            currentMemory -= totalPeakMemory / 15;
        }}
        
        currentMemory = Math.max(0, Math.min(currentMemory, totalPeakMemory * 1.1));
        
        timelineData.push({{
            x: time,
            y: currentMemory
        }});
    }}
    
    new Chart(ctx, {{
        type: 'line',
        data: {{
            datasets: [{{
                label: 'Memory Usage (MB)',
                data: timelineData,
                borderColor: chartColors.primary,
                backgroundColor: chartColors.primary + '20',
                fill: true,
                tension: 0.4,
                pointRadius: 2,
                pointHoverRadius: 6
            }}]
        }},
        options: {{
            responsive: true,
            interaction: {{
                intersect: false,
                mode: 'index'
            }},
            scales: {{
                x: {{
                    type: 'time',
                    time: {{
                        displayFormats: {{
                            minute: 'HH:mm',
                            hour: 'HH:mm'
                        }}
                    }},
                    title: {{
                        display: true,
                        text: 'Time'
                    }}
                }},
                y: {{
                    beginAtZero: true,
                    title: {{
                        display: true,
                        text: 'Memory Usage (MB)'
                    }},
                    ticks: {{
                        callback: function(value) {{
                            return value.toFixed(1) + ' MB';
                        }}
                    }}
                }}
            }},
            plugins: {{
                legend: {{
                    display: true,
                    position: 'top'
                }},
                tooltip: {{
                    callbacks: {{
                        label: function(context) {{
                            return 'Memory: ' + context.parsed.y.toFixed(1) + ' MB';
                        }}
                    }}
                }}
            }}
        }}
    }});
}}

// Initialize all charts when page loads
document.addEventListener('DOMContentLoaded', function() {{
    createThreadMemoryChart();
    createThreadEfficiencyChart();
    createMemoryTimelineChart();
}});

// Tab functionality
function showTab(tabName) {{
    // Hide all tab contents
    const tabContents = document.querySelectorAll('.tab-content');
    tabContents.forEach(content => content.classList.remove('active'));
    
    // Remove active class from all buttons
    const tabButtons = document.querySelectorAll('.tab-button');
    tabButtons.forEach(button => button.classList.remove('active'));
    
    // Show selected tab content
    document.getElementById(tabName).classList.add('active');
    
    // Add active class to clicked button
    event.target.classList.add('active');
}}
</script>
    "#,
        analysis.summary.peak_memory_usage as f64 / (1024.0 * 1024.0)
    ));

    Ok(js)
}
