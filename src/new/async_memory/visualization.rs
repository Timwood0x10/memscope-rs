//! Async Task Performance Visualization Module
//!
//! This module provides comprehensive visualization capabilities for async task performance data.
//! It generates interactive HTML reports with charts, baselines, rankings, and detailed analytics.

use super::{TaskId, TaskResourceProfile};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Task type metrics for categorization
#[derive(Debug, Clone)]
struct TaskTypeMetrics {
    pub cpu_count: usize,
    pub cpu_efficiency: f64,
    pub memory_count: usize,
    pub memory_efficiency: f64,
    pub io_count: usize,
    pub io_efficiency: f64,
    pub network_count: usize,
    pub network_efficiency: f64,
}

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
        // Try template-based generation first, fallback to hardcoded if template not found
        match self.generate_templated_html_report(profiles) {
            Ok(html) => Ok(html),
            Err(_) => self.generate_hardcoded_html_report(profiles), // Fallback to original
        }
    }

    fn get_html_template() -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🦀 {{title}} - Rust Async Performance Dashboard</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/chartjs-adapter-date-fns"></script>
    <style>
        :root {
            --rust-orange: #ff6b35;
            --rust-brown: #8b4513;
            --async-blue: #4c9aff;
            --async-cyan: #00d9ff;
            --async-purple: #8b5cf6;
            --success-green: #10b981;
            --warning-yellow: #f59e0b;
            --error-red: #ef4444;
            --dark-bg: #0c0c0c;
            --card-bg: #1a1a1a;
            --surface-bg: #262626;
            --text-primary: #ffffff;
            --text-secondary: #a3a3a3;
            --border-color: #404040;
            --glow-rust: rgba(255, 107, 53, 0.3);
            --glow-async: rgba(76, 154, 255, 0.3);
        }
        
        * { margin: 0; padding: 0; box-sizing: border-box; }
        
        body {
            font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', 'Consolas', monospace;
            background: linear-gradient(135deg, var(--dark-bg) 0%, #111111 50%, var(--dark-bg) 100%);
            color: var(--text-primary);
            line-height: 1.6;
            min-height: 100vh;
            overflow-x: hidden;
        }
        
        /* Rust-themed animated background */
        .rust-bg-pattern {
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            z-index: -1;
            background-image: 
                radial-gradient(circle at 25% 25%, var(--glow-rust) 0%, transparent 50%),
                radial-gradient(circle at 75% 75%, var(--glow-async) 0%, transparent 50%),
                radial-gradient(circle at 50% 10%, rgba(139, 92, 246, 0.1) 0%, transparent 50%);
            animation: rustFlow 30s ease-in-out infinite;
        }
        
        @keyframes rustFlow {
            0%, 100% { transform: translateY(0px) rotate(0deg); opacity: 0.7; }
            25% { transform: translateY(-10px) rotate(0.5deg); opacity: 0.9; }
            50% { transform: translateY(-5px) rotate(-0.5deg); opacity: 0.8; }
            75% { transform: translateY(-15px) rotate(0.3deg); opacity: 0.6; }
        }
        
        .dashboard-container {
            max-width: 1800px;
            margin: 0 auto;
            padding: 20px;
            position: relative;
        }
        
        /* Enhanced Rust-themed header */
        .rust-header {
            background: linear-gradient(135deg, var(--rust-orange), var(--rust-brown), var(--async-blue));
            padding: 3rem 2rem;
            text-align: center;
            border-radius: 20px;
            margin-bottom: 3rem;
            box-shadow: 0 20px 60px var(--glow-rust), inset 0 1px 0 rgba(255,255,255,0.1);
            position: relative;
            overflow: hidden;
        }
        
        .rust-header::before {
            content: '🦀';
            position: absolute;
            top: 20px;
            left: 30px;
            font-size: 3rem;
            opacity: 0.3;
            animation: rustCrab 5s ease-in-out infinite;
        }
        
        .rust-header::after {
            content: '⚡';
            position: absolute;
            top: 20px;
            right: 30px;
            font-size: 3rem;
            opacity: 0.3;
            animation: asyncBolt 3s ease-in-out infinite;
        }
        
        @keyframes rustCrab {
            0%, 100% { transform: translateX(0px) rotate(0deg); }
            50% { transform: translateX(10px) rotate(5deg); }
        }
        
        @keyframes asyncBolt {
            0%, 100% { transform: scale(1); opacity: 0.3; }
            50% { transform: scale(1.1); opacity: 0.6; }
        }
        
        .rust-title {
            font-size: 3.5rem;
            font-weight: 800;
            margin-bottom: 1rem;
            background: linear-gradient(45deg, #ffffff, var(--async-cyan), var(--rust-orange));
            background-clip: text;
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            text-shadow: 0 0 30px rgba(255, 107, 53, 0.5);
            position: relative;
            z-index: 1;
        }
        
        .rust-subtitle {
            font-size: 1.3rem;
            opacity: 0.9;
            font-weight: 400;
            position: relative;
            z-index: 1;
            text-shadow: 0 1px 3px rgba(0,0,0,0.5);
        }
        
        /* Performance metrics grid */
        .performance-metrics {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
            gap: 2rem;
            margin-bottom: 3rem;
        }
        
        .metric-card {
            background: var(--card-bg);
            border: 2px solid var(--border-color);
            border-radius: 16px;
            padding: 2rem;
            position: relative;
            transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
            overflow: hidden;
        }
        
        .metric-card::before {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 4px;
            background: var(--metric-color, linear-gradient(90deg, var(--rust-orange), var(--async-blue)));
        }
        
        .metric-card:hover {
            transform: translateY(-8px) scale(1.02);
            box-shadow: 0 25px 80px var(--metric-glow, var(--glow-rust));
            border-color: var(--metric-border, var(--rust-orange));
        }
        
        .metric-icon {
            font-size: 2.8rem;
            margin-bottom: 1rem;
            display: block;
            filter: drop-shadow(0 0 10px currentColor);
        }
        
        .metric-value {
            font-size: 2.8rem;
            font-weight: 700;
            margin-bottom: 0.5rem;
            background: linear-gradient(45deg, var(--metric-color, var(--rust-orange)), var(--async-cyan));
            background-clip: text;
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        
        .metric-label {
            color: var(--text-secondary);
            font-size: 1.1rem;
            text-transform: uppercase;
            letter-spacing: 0.1em;
            font-weight: 600;
        }
        
        .metric-details {
            margin-top: 1rem;
            padding-top: 1rem;
            border-top: 1px solid var(--border-color);
            font-size: 0.9rem;
            color: var(--text-secondary);
        }
        
        /* Task type specific colors */
        .cpu-intensive { --metric-color: #ef4444; --metric-glow: rgba(239, 68, 68, 0.3); --metric-border: #ef4444; }
        .memory-intensive { --metric-color: #8b5cf6; --metric-glow: rgba(139, 92, 246, 0.3); --metric-border: #8b5cf6; }
        .io-intensive { --metric-color: #10b981; --metric-glow: rgba(16, 185, 129, 0.3); --metric-border: #10b981; }
        .network-intensive { --metric-color: #f59e0b; --metric-glow: rgba(245, 158, 11, 0.3); --metric-border: #f59e0b; }
        .gpu-compute { --metric-color: #ec4899; --metric-glow: rgba(236, 72, 153, 0.3); --metric-border: #ec4899; }
        .mixed { --metric-color: #6366f1; --metric-glow: rgba(99, 102, 241, 0.3); --metric-border: #6366f1; }
        .streaming { --metric-color: #14b8a6; --metric-glow: rgba(20, 184, 166, 0.3); --metric-border: #14b8a6; }
        .background { --metric-color: #64748b; --metric-glow: rgba(100, 116, 139, 0.3); --metric-border: #64748b; }
        
        /* Task Flow Section */
        .async-task-flow {
            background: var(--card-bg);
            border-radius: 20px;
            padding: 2rem;
            margin-bottom: 3rem;
            border: 1px solid var(--border-color);
            box-shadow: 0 10px 40px rgba(0,0,0,0.3);
        }
        
        .flow-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 3rem;
            padding-bottom: 1rem;
            border-bottom: 2px solid var(--border-color);
        }
        
        .flow-title {
            font-size: 2rem;
            font-weight: 700;
            background: linear-gradient(45deg, var(--rust-orange), var(--async-blue));
            background-clip: text;
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        
        .flow-stats {
            display: flex;
            gap: 2rem;
            flex-wrap: wrap;
        }
        
        .stat-item {
            font-size: 0.9rem;
            color: var(--text-secondary);
            background: var(--surface-bg);
            padding: 0.5rem 1rem;
            border-radius: 8px;
            border: 1px solid var(--border-color);
        }
        
        /* Task Categories */
        .task-categories {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 2rem;
        }
        
        .category-lane {
            background: var(--surface-bg);
            border-radius: 16px;
            padding: 1.5rem;
            border: 2px solid var(--border-color);
            transition: all 0.3s ease;
        }
        
        .category-lane:hover {
            border-color: var(--metric-border);
            box-shadow: 0 8px 32px var(--metric-glow);
        }
        
        .lane-header {
            display: flex;
            align-items: center;
            gap: 1rem;
            margin-bottom: 1.5rem;
            padding-bottom: 1rem;
            border-bottom: 1px solid var(--border-color);
        }
        
        .lane-icon {
            font-size: 1.5rem;
            filter: drop-shadow(0 0 8px currentColor);
        }
        
        .lane-title {
            font-weight: 600;
            font-size: 1.1rem;
            flex: 1;
        }
        
        .lane-count {
            background: var(--metric-color);
            color: var(--dark-bg);
            padding: 0.25rem 0.75rem;
            border-radius: 12px;
            font-weight: 700;
            font-size: 0.8rem;
        }
        
        .lane-efficiency {
            font-size: 0.8rem;
            color: var(--text-secondary);
        }
        
        /* Task Cards */
        .tasks-container {
            display: flex;
            flex-direction: column;
            gap: 1rem;
            max-height: 500px;
            overflow-y: auto;
            padding-right: 0.5rem;
        }
        
        .tasks-container::-webkit-scrollbar {
            width: 6px;
        }
        
        .tasks-container::-webkit-scrollbar-track {
            background: var(--border-color);
            border-radius: 3px;
        }
        
        .tasks-container::-webkit-scrollbar-thumb {
            background: var(--metric-color);
            border-radius: 3px;
        }
        
        .task-card {
            background: var(--card-bg);
            border-radius: 12px;
            padding: 1.5rem;
            border: 1px solid var(--border-color);
            border-left: 4px solid var(--metric-color);
            transition: all 0.3s ease;
            cursor: pointer;
            position: relative;
            overflow: hidden;
        }
        
        .task-card::before {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: linear-gradient(45deg, transparent 30%, var(--metric-glow) 50%, transparent 70%);
            opacity: 0;
            transition: opacity 0.3s ease;
        }
        
        .task-card:hover::before {
            opacity: 1;
        }
        
        .task-card:hover {
            transform: translateY(-3px) scale(1.02);
            box-shadow: 0 12px 40px var(--metric-glow);
            border-color: var(--metric-color);
        }
        
        .task-card.selected {
            border-color: var(--rust-orange);
            box-shadow: 0 0 20px var(--glow-rust);
            transform: scale(1.05);
        }
        
        .task-header {
            margin-bottom: 1rem;
        }
        
        .task-name {
            font-weight: 600;
            font-size: 1rem;
            margin-bottom: 0.25rem;
            color: var(--text-primary);
        }
        
        .task-source {
            font-size: 0.8rem;
            color: var(--text-secondary);
            font-family: 'Courier New', monospace;
            background: var(--surface-bg);
            padding: 0.25rem 0.5rem;
            border-radius: 4px;
            display: inline-block;
        }
        
        .task-metrics {
            margin-bottom: 1rem;
        }
        
        .metric-row {
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 0.5rem;
            font-size: 0.85rem;
        }
        
        .metric-label {
            color: var(--text-secondary);
            min-width: 80px;
        }
        
        .metric-value {
            font-weight: 600;
            color: var(--text-primary);
            min-width: 60px;
            text-align: right;
        }
        
        .metric-bar {
            flex: 1;
            height: 4px;
            background: var(--border-color);
            border-radius: 2px;
            margin: 0 0.5rem;
            overflow: hidden;
        }
        
        .metric-fill {
            height: 100%;
            background: linear-gradient(90deg, var(--metric-color), color-mix(in srgb, var(--metric-color) 70%, white));
            border-radius: 2px;
            transition: width 0.6s ease;
        }
        
        .task-status {
            display: flex;
            justify-content: space-between;
            align-items: center;
            font-size: 0.8rem;
        }
        
        .status-badge {
            padding: 0.25rem 0.5rem;
            border-radius: 4px;
            font-weight: 600;
            text-transform: uppercase;
        }
        
        .status-badge.completed {
            background: rgba(16, 185, 129, 0.2);
            color: var(--success-green);
            border: 1px solid var(--success-green);
        }
        
        .status-badge.running {
            background: rgba(76, 154, 255, 0.2);
            color: var(--async-blue);
            border: 1px solid var(--async-blue);
        }
        
        .status-badge.pending {
            background: rgba(245, 158, 11, 0.2);
            color: var(--warning-yellow);
            border: 1px solid var(--warning-yellow);
        }
        
        .status-badge.failed {
            background: rgba(239, 68, 68, 0.2);
            color: var(--error-red);
            border: 1px solid var(--error-red);
        }
        
        .duration {
            color: var(--text-secondary);
            font-weight: 500;
        }
        
        /* Responsive Design */
        @media (max-width: 1200px) {
            .task-categories {
                grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
            }
            
            .flow-header {
                flex-direction: column;
                gap: 1rem;
                align-items: flex-start;
            }
            
            .flow-stats {
                gap: 1rem;
            }
        }
        
        @media (max-width: 768px) {
            .task-categories {
                grid-template-columns: 1fr;
            }
            
            .performance-metrics {
                grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            }
            
            .rust-title {
                font-size: 2.5rem;
            }
            
            .flow-stats {
                flex-direction: column;
                gap: 0.5rem;
            }
            
            .lane-header {
                flex-wrap: wrap;
                gap: 0.5rem;
            }
        }
    </style>
</head>
<body>
    <div class="rust-bg-pattern"></div>
    
    <div class="dashboard-container">
        <div class="rust-header">
            <h1 class="rust-title">🦀 {{title}}</h1>
            <p class="rust-subtitle">{{subtitle}}</p>
        </div>
        
        <div class="performance-metrics">
            <div class="metric-card">
                <span class="metric-icon">🚀</span>
                <div class="metric-value">{{total_tasks}}</div>
                <div class="metric-label">Async Tasks</div>
                <div class="metric-details">
                    Active: {{active_tasks}} | Completed: {{completed_tasks}} | Failed: {{failed_tasks}}
                </div>
            </div>
            
            <div class="metric-card cpu-intensive">
                <span class="metric-icon">🔥</span>
                <div class="metric-value">{{cpu_usage_avg}}%</div>
                <div class="metric-label">CPU Usage</div>
                <div class="metric-details">
                    Peak: {{cpu_usage_peak}}% | Cores: {{cpu_cores}} | Context Switches: {{context_switches}}
                </div>
            </div>
            
            <div class="metric-card memory-intensive">
                <span class="metric-icon">💾</span>
                <div class="metric-value">{{total_memory_mb}}MB</div>
                <div class="metric-label">Memory Usage</div>
                <div class="metric-details">
                    Peak: {{peak_memory_mb}}MB | Allocations: {{total_allocations}} | Efficiency: {{memory_efficiency}}%
                </div>
            </div>
            
            <div class="metric-card io-intensive">
                <span class="metric-icon">⚡</span>
                <div class="metric-value">{{io_throughput}}MB/s</div>
                <div class="metric-label">I/O Throughput</div>
                <div class="metric-details">
                    Read: {{total_read_mb}}MB | Write: {{total_write_mb}}MB | Ops: {{total_io_ops}}
                </div>
            </div>
            
            <div class="metric-card network-intensive">
                <span class="metric-icon">🌐</span>
                <div class="metric-value">{{network_throughput}}Mbps</div>
                <div class="metric-label">Network Throughput</div>
                <div class="metric-details">
                    Sent: {{total_sent_mb}}MB | Received: {{total_received_mb}}MB | Latency: {{avg_latency}}ms
                </div>
            </div>
            
            <div class="metric-card">
                <span class="metric-icon">⚖️</span>
                <div class="metric-value">{{efficiency_score}}%</div>
                <div class="metric-label">Overall Efficiency</div>
                <div class="metric-details">
                    Balance: {{resource_balance}}% | Bottlenecks: {{bottleneck_count}} | Optimization: {{optimization_potential}}%
                </div>
            </div>
        </div>
        
        <!-- Task Flow Visualization with Rust Async characteristics -->
        <div class="async-task-flow">
            <div class="flow-header">
                <h2 class="flow-title">🦀 Rust Async Task Flow Analysis</h2>
                <div class="flow-stats">
                    <span class="stat-item">📊 Futures: {{futures_count}}</span>
                    <span class="stat-item">🔄 Polled: {{total_polls}}</span>
                    <span class="stat-item">⏱️ Avg Poll Time: {{avg_poll_time}}μs</span>
                    <span class="stat-item">🎯 Ready Rate: {{ready_rate}}%</span>
                </div>
            </div>
            
            <!-- Task Type Categories -->
            <div class="task-categories">
                <div class="category-lane cpu-intensive">
                    <div class="lane-header">
                        <span class="lane-icon">🔥</span>
                        <span class="lane-title">CPU Intensive Tasks</span>
                        <span class="lane-count">{{cpu_intensive_count}}</span>
                        <span class="lane-efficiency">{{cpu_avg_efficiency}}% efficiency</span>
                    </div>
                    <div class="tasks-container">
                        {{#each cpu_intensive_tasks}}
                        <div class="task-card cpu-intensive" data-task-id="{{task_id}}" onclick="event.stopPropagation(); selectTask('{{task_id}}'); return false;">
                            <div class="task-header">
                                <div class="task-name">{{task_name}}</div>
                                <div class="task-source">{{source_file}}:{{source_line}}</div>
                            </div>
                            <div class="task-metrics">
                                <div class="metric-row">
                                    <span class="metric-label">CPU:</span>
                                    <span class="metric-value">{{cpu_usage}}%</span>
                                    <div class="metric-bar">
                                        <div class="metric-fill" style="width: {{cpu_usage}}%"></div>
                                    </div>
                                </div>
                                <div class="metric-row">
                                    <span class="metric-label">Cycles:</span>
                                    <span class="metric-value">{{cpu_cycles}}M</span>
                                </div>
                                <div class="metric-row">
                                    <span class="metric-label">Instructions:</span>
                                    <span class="metric-value">{{instructions}}M</span>
                                </div>
                                <div class="metric-row">
                                    <span class="metric-label">Cache Misses:</span>
                                    <span class="metric-value">{{cache_misses}}K</span>
                                </div>
                            </div>
                            <div class="task-status">
                                <span class="status-badge {{status_class}}">{{status}}</span>
                                <span class="duration">{{duration_ms}}ms</span>
                            </div>
                        </div>
                        {{/each}}
                    </div>
                </div>
                
                <div class="category-lane memory-intensive">
                    <div class="lane-header">
                        <span class="lane-icon">💾</span>
                        <span class="lane-title">Memory Intensive Tasks</span>
                        <span class="lane-count">{{memory_intensive_count}}</span>
                        <span class="lane-efficiency">{{memory_avg_efficiency}}% efficiency</span>
                    </div>
                    <div class="tasks-container">
                        {{#each memory_intensive_tasks}}
                        <div class="task-card memory-intensive" data-task-id="{{task_id}}" onclick="event.stopPropagation(); selectTask('{{task_id}}'); return false;">
                            <div class="task-header">
                                <div class="task-name">{{task_name}}</div>
                                <div class="task-source">{{source_file}}:{{source_line}}</div>
                            </div>
                            <div class="task-metrics">
                                <div class="metric-row">
                                    <span class="metric-label">Allocated:</span>
                                    <span class="metric-value">{{allocated_mb}}MB</span>
                                    <div class="metric-bar">
                                        <div class="metric-fill" style="width: {{memory_usage_percent}}%"></div>
                                    </div>
                                </div>
                                <div class="metric-row">
                                    <span class="metric-label">Peak:</span>
                                    <span class="metric-value">{{peak_memory_mb}}MB</span>
                                </div>
                                <div class="metric-row">
                                    <span class="metric-label">Allocations:</span>
                                    <span class="metric-value">{{allocation_count}}</span>
                                </div>
                                <div class="metric-row">
                                    <span class="metric-label">Fragmentation:</span>
                                    <span class="metric-value">{{heap_fragmentation}}%</span>
                                </div>
                            </div>
                            <div class="task-status">
                                <span class="status-badge {{status_class}}">{{status}}</span>
                                <span class="duration">{{duration_ms}}ms</span>
                            </div>
                        </div>
                        {{/each}}
                    </div>
                </div>
                
                <div class="category-lane io-intensive">
                    <div class="lane-header">
                        <span class="lane-icon">⚡</span>
                        <span class="lane-title">I/O Intensive Tasks</span>
                        <span class="lane-count">{{io_intensive_count}}</span>
                        <span class="lane-efficiency">{{io_avg_efficiency}}% efficiency</span>
                    </div>
                    <div class="tasks-container">
                        {{#each io_intensive_tasks}}
                        <div class="task-card io-intensive" data-task-id="{{task_id}}" onclick="event.stopPropagation(); selectTask('{{task_id}}'); return false;">
                            <div class="task-header">
                                <div class="task-name">{{task_name}}</div>
                                <div class="task-source">{{source_file}}:{{source_line}}</div>
                            </div>
                            <div class="task-metrics">
                                <div class="metric-row">
                                    <span class="metric-label">Read:</span>
                                    <span class="metric-value">{{bytes_read_mb}}MB</span>
                                    <div class="metric-bar">
                                        <div class="metric-fill" style="width: {{io_usage_percent}}%"></div>
                                    </div>
                                </div>
                                <div class="metric-row">
                                    <span class="metric-label">Write:</span>
                                    <span class="metric-value">{{bytes_written_mb}}MB</span>
                                </div>
                                <div class="metric-row">
                                    <span class="metric-label">Latency:</span>
                                    <span class="metric-value">{{avg_latency_us}}μs</span>
                                </div>
                                <div class="metric-row">
                                    <span class="metric-label">Queue Depth:</span>
                                    <span class="metric-value">{{queue_depth}}</span>
                                </div>
                            </div>
                            <div class="task-status">
                                <span class="status-badge {{status_class}}">{{status}}</span>
                                <span class="duration">{{duration_ms}}ms</span>
                            </div>
                        </div>
                        {{/each}}
                    </div>
                </div>
                
                <div class="category-lane network-intensive">
                    <div class="lane-header">
                        <span class="lane-icon">🌐</span>
                        <span class="lane-title">Network Intensive Tasks</span>
                        <span class="lane-count">{{network_intensive_count}}</span>
                        <span class="lane-efficiency">{{network_avg_efficiency}}% efficiency</span>
                    </div>
                    <div class="tasks-container">
                        {{#each network_intensive_tasks}}
                        <div class="task-card network-intensive" data-task-id="{{task_id}}" onclick="event.stopPropagation(); selectTask('{{task_id}}'); return false;">
                            <div class="task-header">
                                <div class="task-name">{{task_name}}</div>
                                <div class="task-source">{{source_file}}:{{source_line}}</div>
                            </div>
                            <div class="task-metrics">
                                <div class="metric-row">
                                    <span class="metric-label">Sent:</span>
                                    <span class="metric-value">{{bytes_sent_mb}}MB</span>
                                    <div class="metric-bar">
                                        <div class="metric-fill" style="width: {{network_usage_percent}}%"></div>
                                    </div>
                                </div>
                                <div class="metric-row">
                                    <span class="metric-label">Received:</span>
                                    <span class="metric-value">{{bytes_received_mb}}MB</span>
                                </div>
                                <div class="metric-row">
                                    <span class="metric-label">Connections:</span>
                                    <span class="metric-value">{{active_connections}}</span>
                                </div>
                                <div class="metric-row">
                                    <span class="metric-label">Latency:</span>
                                    <span class="metric-value">{{avg_latency_ms}}ms</span>
                                </div>
                            </div>
                            <div class="task-status">
                                <span class="status-badge {{status_class}}">{{status}}</span>
                                <span class="duration">{{duration_ms}}ms</span>
                            </div>
                        </div>
                        {{/each}}
                    </div>
                </div>
            </div>
        </div>
        
        <!-- Advanced Analytics Section -->
        <div class="analytics-section">
            <div class="analytics-header">
                <h2 class="analytics-title">🔬 Advanced Rust Async Analytics</h2>
            </div>
            
            <div class="analytics-grid">
                <div class="analytics-card">
                    <h3>🚀 Future Polling Insights</h3>
                    <div class="insight-metrics">
                        <div class="insight-metric">
                            <span class="label">Total Polls:</span>
                            <span class="value">{{total_polls}}</span>
                        </div>
                        <div class="insight-metric">
                            <span class="label">Avg Poll Duration:</span>
                            <span class="value">{{avg_poll_duration}}μs</span>
                        </div>
                        <div class="insight-metric">
                            <span class="label">Ready Immediately:</span>
                            <span class="value">{{immediate_ready_percent}}%</span>
                        </div>
                        <div class="insight-metric">
                            <span class="label">Waker Efficiency:</span>
                            <span class="value">{{waker_efficiency}}%</span>
                        </div>
                    </div>
                </div>
                
                <div class="analytics-card">
                    <h3>🧠 Memory Management</h3>
                    <div class="insight-metrics">
                        <div class="insight-metric">
                            <span class="label">Total Allocations:</span>
                            <span class="value">{{total_allocations}}</span>
                        </div>
                        <div class="insight-metric">
                            <span class="label">Peak Allocation Rate:</span>
                            <span class="value">{{peak_alloc_rate}}/s</span>
                        </div>
                        <div class="insight-metric">
                            <span class="label">Memory Fragmentation:</span>
                            <span class="value">{{avg_fragmentation}}%</span>
                        </div>
                        <div class="insight-metric">
                            <span class="label">GC Pressure:</span>
                            <span class="value">{{gc_pressure}}%</span>
                        </div>
                    </div>
                </div>
                
                <div class="analytics-card">
                    <h3>⚡ Async Runtime Health</h3>
                    <div class="insight-metrics">
                        <div class="insight-metric">
                            <span class="label">Executor Utilization:</span>
                            <span class="value">{{executor_utilization}}%</span>
                        </div>
                        <div class="insight-metric">
                            <span class="label">Task Queue Length:</span>
                            <span class="value">{{avg_queue_length}}</span>
                        </div>
                        <div class="insight-metric">
                            <span class="label">Blocking Tasks:</span>
                            <span class="value">{{blocking_tasks_count}}</span>
                        </div>
                        <div class="insight-metric">
                            <span class="label">Deadlock Risk:</span>
                            <span class="value">{{deadlock_risk}}%</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script>
        // Enhanced JavaScript for Rust Async Dashboard
        
        let selectedTaskId = null;
        let animationFrameId = null;
        
        // Initialize dashboard
        document.addEventListener('DOMContentLoaded', function() {
            initializeDashboard();
            startMetricAnimations();
            setupTaskInteractions();
        });
        
        function initializeDashboard() {
            console.log('🦀 Rust Async Performance Dashboard Initialized');
            
            // Add pulse animation to metric cards
            const metricCards = document.querySelectorAll('.metric-card');
            metricCards.forEach((card, index) => {
                setTimeout(() => {
                    card.style.opacity = '0';
                    card.style.transform = 'translateY(20px)';
                    card.style.transition = 'all 0.6s cubic-bezier(0.175, 0.885, 0.32, 1.275)';
                    
                    setTimeout(() => {
                        card.style.opacity = '1';
                        card.style.transform = 'translateY(0)';
                    }, 100);
                }, index * 100);
            });
        }
        
        function selectTask(taskId) {
            // Check if event exists to prevent errors
            if (typeof event !== 'undefined') {
                event.preventDefault();
                event.stopPropagation();
            }
            
            console.log(`🦀 Selecting Rust async task: ${taskId}`);
            
            // Remove previous selection
            document.querySelectorAll('.task-card').forEach(card => {
                card.classList.remove('selected');
            });
            
            // Add selection to clicked task
            const taskCard = document.querySelector(`[data-task-id="${taskId}"]`);
            if (taskCard) {
                taskCard.classList.add('selected');
                selectedTaskId = taskId;
                
                // Show task details with delay to ensure DOM is ready
                setTimeout(() => {
                    showTaskDetails(taskId);
                }, 100);
                
                // Scroll task into view
                taskCard.scrollIntoView({ 
                    behavior: 'smooth', 
                    block: 'center' 
                });
            }
        }
        
        function showTaskDetails(taskId) {
            // Create or update task details panel
            let detailsPanel = document.getElementById('task-details-panel');
            if (!detailsPanel) {
                detailsPanel = createTaskDetailsPanel();
                document.body.appendChild(detailsPanel);
            }
            
            // Populate with task data
            const taskCard = document.querySelector(`[data-task-id="${taskId}"]`);
            if (taskCard) {
                const taskName = taskCard.querySelector('.task-name').textContent;
                const taskSource = taskCard.querySelector('.task-source').textContent;
                const taskStatus = taskCard.querySelector('.status-badge').textContent;
                const taskDuration = taskCard.querySelector('.duration').textContent;
                
                // Get task type from card classes
                const taskType = Array.from(taskCard.classList).find(cls => cls.includes('-intensive')) || 'unknown';
                const taskTypeDisplay = taskType.replace('-intensive', '').replace('-', ' ').toUpperCase();
                
                // Build comprehensive task details
                detailsPanel.innerHTML = `
                    <div class="details-header">
                        <div class="header-content">
                            <h3>🦀 Rust Async Task Details</h3>
                            <div class="task-title-info">
                                <span class="task-title">${taskName}</span>
                                <span class="task-type-badge ${taskType}">${taskTypeDisplay} TASK</span>
                            </div>
                        </div>
                        <button onclick="closeTaskDetails()" class="close-btn">✕</button>
                    </div>
                    <div class="details-content">
                        <div class="detail-section">
                            <h4>📍 Source Location & Context</h4>
                            <div class="source-info">
                                <p class="source-code">${taskSource}</p>
                                <div class="execution-info">
                                    <span class="info-item">Status: <strong class="status-${taskStatus.toLowerCase()}">${taskStatus}</strong></span>
                                    <span class="info-item">Duration: <strong>${taskDuration}</strong></span>
                                    <span class="info-item">Task ID: <strong>${taskId}</strong></span>
                                </div>
                            </div>
                        </div>
                        
                        <div class="detail-section">
                            <h4>📊 Detailed Performance Metrics</h4>
                            <div class="metrics-grid">
                                ${Array.from(taskCard.querySelectorAll('.metric-row')).map(row => {
                                    const label = row.querySelector('.metric-label').textContent;
                                    const value = row.querySelector('.metric-value').textContent;
                                    const bar = row.querySelector('.metric-fill');
                                    const percentage = bar ? bar.style.width : '0%';
                                    
                                    return `<div class="metric-detail-card">
                                        <div class="metric-header">
                                            <span class="metric-name">${label}</span>
                                            <span class="metric-val">${value}</span>
                                        </div>
                                        <div class="metric-bar-container">
                                            <div class="metric-bar-bg">
                                                <div class="metric-bar-fill ${taskType}" style="width: ${percentage}"></div>
                                            </div>
                                            <span class="metric-percentage">${percentage}</span>
                                        </div>
                                    </div>`;
                                }).join('')}
                            </div>
                        </div>
                        
                        <div class="detail-section">
                            <h4>🔬 Async Runtime Analysis</h4>
                            <div class="analysis-grid">
                                <div class="analysis-card">
                                    <div class="analysis-icon">⚡</div>
                                    <div class="analysis-content">
                                        <h5>Future Polling</h5>
                                        <p>This task represents a Rust Future that gets polled by the async executor. Monitor polling frequency and ready states for optimization.</p>
                                    </div>
                                </div>
                                <div class="analysis-card">
                                    <div class="analysis-icon">🧠</div>
                                    <div class="analysis-content">
                                        <h5>Resource Usage</h5>
                                        <p>Track ${taskTypeDisplay.toLowerCase()} resource consumption patterns to identify bottlenecks and optimization opportunities.</p>
                                    </div>
                                </div>
                                <div class="analysis-card">
                                    <div class="analysis-icon">🎯</div>
                                    <div class="analysis-content">
                                        <h5>Efficiency Score</h5>
                                        <p>Overall task efficiency based on resource utilization, context switches, and async runtime behavior.</p>
                                    </div>
                                </div>
                            </div>
                        </div>
                        
                        <div class="detail-section">
                            <h4>🔧 ${taskTypeDisplay} Task Optimization Suggestions</h4>
                            <div class="suggestions-container">
                                ${getTaskSpecificSuggestions(taskType)}
                            </div>
                        </div>
                        
                        <div class="detail-section">
                            <h4>📈 Performance Timeline</h4>
                            <div class="timeline-placeholder">
                                <p>📊 Detailed performance timeline visualization would appear here in a full implementation</p>
                                <div class="timeline-mock">
                                    <div class="timeline-item">Start → Resource Allocation → Execution → Completion</div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="details-footer">
                        <button onclick="exportTaskData('${taskId}')" class="action-btn export-btn">📤 Export Task Data</button>
                        <button onclick="analyzeTaskPerformance('${taskId}')" class="action-btn analyze-btn">🔍 Deep Analysis</button>
                        <button onclick="closeTaskDetails()" class="action-btn close-action-btn">Close</button>
                    </div>
                `;
                
                // Show panel with animation
                detailsPanel.style.display = 'block';
                setTimeout(() => {
                    detailsPanel.classList.add('visible');
                    // Add backdrop blur effect
                    document.body.classList.add('modal-open');
                }, 10);
                
                console.log(`🦀 Opened detailed view for Rust async task: ${taskName} (${taskId})`);
            }
        }
        
        function createTaskDetailsPanel() {
            const panel = document.createElement('div');
            panel.id = 'task-details-panel';
            panel.className = 'task-details-panel';
            panel.style.cssText = `
                position: fixed;
                top: 50%;
                left: 50%;
                transform: translate(-50%, -50%) scale(0.9);
                width: 90%;
                max-width: 600px;
                max-height: 80vh;
                background: var(--card-bg);
                border: 2px solid var(--rust-orange);
                border-radius: 16px;
                box-shadow: 0 20px 60px rgba(0,0,0,0.5);
                z-index: 1000;
                display: none;
                opacity: 0;
                transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
                overflow-y: auto;
                color: var(--text-primary);
            `;
            
            return panel;
        }
        
        function closeTaskDetails() {
            const panel = document.getElementById('task-details-panel');
            if (panel) {
                panel.classList.remove('visible');
                document.body.classList.remove('modal-open');
                setTimeout(() => {
                    panel.style.display = 'none';
                }, 300);
            }
            
            // Also remove selection from task cards
            document.querySelectorAll('.task-card.selected').forEach(card => {
                card.classList.remove('selected');
            });
            selectedTaskId = null;
            console.log('🦀 Closed task details panel');
        }
        
        // Get task-specific optimization suggestions
        function getTaskSpecificSuggestions(taskType) {
            const suggestions = {
                'cpu-intensive': [
                    'Use rayon for parallel computation where applicable',
                    'Consider async-friendly CPU-bound algorithms',
                    'Implement work-stealing for better load distribution',
                    'Profile hot paths and optimize critical sections',
                    'Use tokio::task::yield_now() to prevent blocking the executor'
                ],
                'memory-intensive': [
                    'Implement memory pooling to reduce allocations',
                    'Use async streams for large data processing',
                    'Consider zero-copy techniques where possible',
                    'Monitor heap fragmentation and optimize allocation patterns',
                    'Use Arc and Rc judiciously to minimize cloning'
                ],
                'io-intensive': [
                    'Use async I/O operations with proper buffering',
                    'Implement connection pooling for database operations',
                    'Consider batch operations to reduce I/O overhead',
                    'Use tokio::fs for async file operations',
                    'Optimize buffer sizes based on workload patterns'
                ],
                'network-intensive': [
                    'Implement connection reuse and keep-alive',
                    'Use async HTTP clients with connection pooling',
                    'Consider implementing backpressure for streaming data',
                    'Optimize serialization/deserialization performance',
                    'Use compression for large data transfers'
                ]
            };
            
            const taskSuggestions = suggestions[taskType] || suggestions['cpu-intensive'];
            return taskSuggestions.map(suggestion => 
                `<div class="suggestion-item">
                    <div class="suggestion-icon">💡</div>
                    <div class="suggestion-text">${suggestion}</div>
                </div>`
            ).join('');
        }
        
        // Export task data functionality
        function exportTaskData(taskId) {
            const taskCard = document.querySelector(`[data-task-id="${taskId}"]`);
            if (taskCard) {
                const taskData = {
                    taskId: taskId,
                    taskName: taskCard.querySelector('.task-name').textContent,
                    sourceLocation: taskCard.querySelector('.task-source').textContent,
                    status: taskCard.querySelector('.status-badge').textContent,
                    duration: taskCard.querySelector('.duration').textContent,
                    metrics: Array.from(taskCard.querySelectorAll('.metric-row')).map(row => ({
                        label: row.querySelector('.metric-label').textContent,
                        value: row.querySelector('.metric-value').textContent
                    })),
                    exportTimestamp: new Date().toISOString()
                };
                
                const blob = new Blob([JSON.stringify(taskData, null, 2)], { type: 'application/json' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = `rust-async-task-${taskId}-${Date.now()}.json`;
                a.click();
                URL.revokeObjectURL(url);
                
                console.log(`📤 Exported data for task: ${taskId}`);
            }
        }
        
        // Deep analysis functionality (placeholder)
        function analyzeTaskPerformance(taskId) {
            console.log(`🔍 Starting deep analysis for task: ${taskId}`);
            
            // Create analysis notification
            const notification = document.createElement('div');
            notification.className = 'analysis-notification';
            notification.innerHTML = `
                <div class="notification-content">
                    <span class="notification-icon">🔬</span>
                    <span class="notification-text">Deep analysis started for task ${taskId}</span>
                    <span class="notification-progress">Analyzing...</span>
                </div>
            `;
            
            document.body.appendChild(notification);
            
            // Simulate analysis process
            setTimeout(() => {
                notification.querySelector('.notification-progress').textContent = 'Analysis complete!';
                notification.classList.add('success');
                
                setTimeout(() => {
                    document.body.removeChild(notification);
                }, 2000);
            }, 3000);
        }
        
        function startMetricAnimations() {
            // Animate metric bars
            const metricFills = document.querySelectorAll('.metric-fill');
            metricFills.forEach(fill => {
                const width = fill.style.width;
                fill.style.width = '0%';
                setTimeout(() => {
                    fill.style.width = width;
                }, Math.random() * 1000 + 500);
            });
            
            // Pulse animation for real-time feeling
            setInterval(() => {
                const activeCards = document.querySelectorAll('.task-card');
                const randomCard = activeCards[Math.floor(Math.random() * activeCards.length)];
                if (randomCard && !randomCard.classList.contains('selected')) {
                    randomCard.style.transform = 'scale(1.01)';
                    setTimeout(() => {
                        randomCard.style.transform = '';
                    }, 200);
                }
            }, 3000);
        }
        
        function setupTaskInteractions() {
            // Add keyboard navigation
            document.addEventListener('keydown', function(e) {
                if (e.key === 'Escape') {
                    closeTaskDetails();
                    // Deselect all tasks
                    document.querySelectorAll('.task-card').forEach(card => {
                        card.classList.remove('selected');
                    });
                    selectedTaskId = null;
                }
            });
            
            // Add click outside to close details - but only on backdrop, not the panel itself
            document.addEventListener('click', function(e) {
                const panel = document.getElementById('task-details-panel');
                if (panel && panel.classList.contains('visible')) {
                    // Only close if clicking directly on the backdrop, not the panel content
                    if (e.target === panel) {
                        closeTaskDetails();
                    }
                }
            });
        }
        
        // Utility functions for enhanced interactivity
        function filterTasksByType(taskType) {
            const allCards = document.querySelectorAll('.task-card');
            allCards.forEach(card => {
                if (taskType === 'all' || card.classList.contains(taskType)) {
                    card.style.display = 'block';
                    card.style.opacity = '1';
                } else {
                    card.style.opacity = '0.3';
                }
            });
        }
        
        function exportDashboardData() {
            const data = {
                timestamp: new Date().toISOString(),
                tasks: Array.from(document.querySelectorAll('.task-card')).map(card => ({
                    id: card.dataset.taskId,
                    name: card.querySelector('.task-name').textContent,
                    type: Array.from(card.classList).find(c => c.includes('-intensive')),
                    metrics: Array.from(card.querySelectorAll('.metric-row')).map(row => ({
                        label: row.querySelector('.metric-label').textContent,
                        value: row.querySelector('.metric-value').textContent
                    }))
                }))
            };
            
            const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `rust-async-performance-${Date.now()}.json`;
            a.click();
            URL.revokeObjectURL(url);
        }
        
        // Performance monitoring
        function monitorPerformance() {
            if ('performance' in window) {
                const navigation = performance.getEntriesByType('navigation')[0];
                console.log(`🦀 Dashboard Load Time: ${navigation.loadEventEnd - navigation.loadEventStart}ms`);
                
                // Monitor memory usage if available
                if ('memory' in performance) {
                    const memory = performance.memory;
                    console.log(`💾 Memory Usage: ${(memory.usedJSHeapSize / 1048576).toFixed(2)}MB`);
                }
            }
        }
        
        // Initialize performance monitoring
        window.addEventListener('load', monitorPerformance);
        
        // CSS for task details panel
        const detailsStyles = `
            /* Enhanced Task Details Panel */
            .task-details-panel {
                position: fixed;
                top: 50%;
                left: 50%;
                transform: translate(-50%, -50%) scale(0.9);
                width: 95%;
                max-width: 900px;
                max-height: 90vh;
                background: var(--card-bg);
                border: 2px solid var(--rust-orange);
                border-radius: 20px;
                box-shadow: 0 25px 80px rgba(0,0,0,0.7), 0 0 0 1000px rgba(0,0,0,0.5);
                z-index: 1000;
                display: none;
                opacity: 0;
                transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
                overflow: hidden;
                color: var(--text-primary);
            }
            
            .task-details-panel.visible {
                opacity: 1 !important;
                transform: translate(-50%, -50%) scale(1) !important;
            }
            
            .body.modal-open {
                overflow: hidden;
            }
            
            .details-header {
                background: linear-gradient(135deg, var(--rust-orange), var(--async-blue));
                padding: 2rem;
                display: flex;
                justify-content: space-between;
                align-items: flex-start;
                border-bottom: 2px solid var(--border-color);
            }
            
            .header-content h3 {
                font-size: 1.5rem;
                margin-bottom: 1rem;
                color: white;
                text-shadow: 0 2px 4px rgba(0,0,0,0.3);
            }
            
            .task-title-info {
                display: flex;
                flex-direction: column;
                gap: 0.75rem;
            }
            
            .task-title {
                font-size: 1.3rem;
                font-weight: 700;
                color: white;
                text-shadow: 0 1px 3px rgba(0,0,0,0.5);
            }
            
            .task-type-badge {
                display: inline-block;
                padding: 0.5rem 1rem;
                border-radius: 25px;
                font-size: 0.8rem;
                font-weight: 700;
                text-transform: uppercase;
                letter-spacing: 0.05em;
                background: rgba(255,255,255,0.2);
                border: 1px solid rgba(255,255,255,0.3);
                color: white;
                backdrop-filter: blur(10px);
            }
            
            .details-content {
                padding: 2rem;
                overflow-y: auto;
                max-height: 60vh;
            }
            
            .details-content::-webkit-scrollbar {
                width: 8px;
            }
            
            .details-content::-webkit-scrollbar-track {
                background: var(--surface-bg);
                border-radius: 4px;
            }
            
            .details-content::-webkit-scrollbar-thumb {
                background: var(--rust-orange);
                border-radius: 4px;
            }
            
            .detail-section {
                margin-bottom: 2.5rem;
                padding-bottom: 1.5rem;
                border-bottom: 1px solid var(--border-color);
            }
            
            .detail-section:last-child {
                border-bottom: none;
            }
            
            .detail-section h4 {
                color: var(--rust-orange);
                margin-bottom: 1.5rem;
                font-size: 1.2rem;
                font-weight: 600;
                display: flex;
                align-items: center;
                gap: 0.5rem;
            }
            
            .source-info {
                background: var(--surface-bg);
                padding: 1.5rem;
                border-radius: 12px;
                border: 1px solid var(--border-color);
            }
            
            .source-code {
                background: var(--dark-bg);
                padding: 1rem;
                border-radius: 8px;
                font-family: 'Courier New', monospace;
                font-size: 0.9rem;
                color: var(--async-cyan);
                border-left: 4px solid var(--rust-orange);
                margin-bottom: 1rem;
            }
            
            .execution-info {
                display: flex;
                gap: 2rem;
                flex-wrap: wrap;
            }
            
            .info-item {
                font-size: 0.9rem;
                color: var(--text-secondary);
            }
            
            .info-item strong {
                color: var(--text-primary);
                font-weight: 600;
            }
            
            .metrics-grid {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
                gap: 1rem;
            }
            
            .metric-detail-card {
                background: var(--surface-bg);
                padding: 1.5rem;
                border-radius: 12px;
                border: 1px solid var(--border-color);
                transition: all 0.3s ease;
            }
            
            .metric-detail-card:hover {
                border-color: var(--rust-orange);
                box-shadow: 0 4px 16px rgba(255, 107, 53, 0.2);
            }
            
            .metric-header {
                display: flex;
                justify-content: space-between;
                align-items: center;
                margin-bottom: 1rem;
            }
            
            .metric-name {
                color: var(--text-secondary);
                font-size: 0.9rem;
                font-weight: 500;
            }
            
            .metric-val {
                color: var(--text-primary);
                font-weight: 700;
                font-size: 1.1rem;
            }
            
            .metric-bar-container {
                display: flex;
                align-items: center;
                gap: 1rem;
            }
            
            .metric-bar-bg {
                flex: 1;
                height: 8px;
                background: var(--border-color);
                border-radius: 4px;
                overflow: hidden;
            }
            
            .metric-bar-fill {
                height: 100%;
                border-radius: 4px;
                transition: width 0.6s ease;
                background: linear-gradient(90deg, var(--metric-color), color-mix(in srgb, var(--metric-color) 70%, white));
            }
            
            .metric-percentage {
                font-size: 0.8rem;
                color: var(--text-secondary);
                min-width: 40px;
                text-align: right;
            }
            
            .analysis-grid {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
                gap: 1.5rem;
            }
            
            .analysis-card {
                background: var(--surface-bg);
                padding: 1.5rem;
                border-radius: 12px;
                border: 1px solid var(--border-color);
                display: flex;
                gap: 1rem;
                transition: all 0.3s ease;
            }
            
            .analysis-card:hover {
                border-color: var(--async-blue);
                box-shadow: 0 4px 16px rgba(76, 154, 255, 0.2);
            }
            
            .analysis-icon {
                font-size: 2rem;
                filter: drop-shadow(0 0 8px currentColor);
            }
            
            .analysis-content h5 {
                margin-bottom: 0.5rem;
                color: var(--text-primary);
                font-weight: 600;
            }
            
            .analysis-content p {
                font-size: 0.9rem;
                color: var(--text-secondary);
                line-height: 1.5;
            }
            
            .suggestions-container {
                display: flex;
                flex-direction: column;
                gap: 1rem;
            }
            
            .suggestion-item {
                display: flex;
                align-items: flex-start;
                gap: 1rem;
                background: var(--surface-bg);
                padding: 1rem;
                border-radius: 8px;
                border-left: 4px solid var(--async-blue);
                transition: all 0.3s ease;
            }
            
            .suggestion-item:hover {
                background: color-mix(in srgb, var(--surface-bg) 80%, var(--async-blue));
            }
            
            .suggestion-icon {
                font-size: 1.2rem;
                margin-top: 0.2rem;
            }
            
            .suggestion-text {
                color: var(--text-primary);
                font-size: 0.9rem;
                line-height: 1.5;
            }
            
            .timeline-placeholder {
                background: var(--surface-bg);
                padding: 2rem;
                border-radius: 12px;
                border: 1px dashed var(--border-color);
                text-align: center;
            }
            
            .timeline-mock {
                margin-top: 1rem;
                padding: 1rem;
                background: var(--dark-bg);
                border-radius: 8px;
                font-family: monospace;
                color: var(--async-cyan);
            }
            
            .details-footer {
                background: var(--surface-bg);
                padding: 1.5rem 2rem;
                border-top: 1px solid var(--border-color);
                display: flex;
                gap: 1rem;
                justify-content: flex-end;
                flex-wrap: wrap;
            }
            
            .action-btn {
                padding: 0.75rem 1.5rem;
                border: none;
                border-radius: 8px;
                font-weight: 600;
                cursor: pointer;
                transition: all 0.3s ease;
                display: flex;
                align-items: center;
                gap: 0.5rem;
                font-size: 0.9rem;
            }
            
            .export-btn {
                background: var(--success-green);
                color: white;
            }
            
            .export-btn:hover {
                background: #059669;
                transform: translateY(-2px);
            }
            
            .analyze-btn {
                background: var(--async-blue);
                color: white;
            }
            
            .analyze-btn:hover {
                background: #2563eb;
                transform: translateY(-2px);
            }
            
            .close-action-btn {
                background: var(--error-red);
                color: white;
            }
            
            .close-action-btn:hover {
                background: #dc2626;
                transform: translateY(-2px);
            }
            
            .close-btn {
                background: rgba(255,255,255,0.2);
                color: white;
                border: 1px solid rgba(255,255,255,0.3);
                padding: 0.75rem 1rem;
                border-radius: 8px;
                cursor: pointer;
                font-weight: 600;
                backdrop-filter: blur(10px);
                transition: all 0.3s ease;
            }
            
            .close-btn:hover {
                background: rgba(255,255,255,0.3);
                transform: scale(1.05);
            }
            
            /* Analysis notification */
            .analysis-notification {
                position: fixed;
                top: 2rem;
                right: 2rem;
                background: var(--card-bg);
                border: 2px solid var(--async-blue);
                border-radius: 12px;
                padding: 1rem 1.5rem;
                box-shadow: 0 8px 32px rgba(0,0,0,0.5);
                z-index: 1001;
                color: var(--text-primary);
                transition: all 0.3s ease;
            }
            
            .analysis-notification.success {
                border-color: var(--success-green);
            }
            
            .notification-content {
                display: flex;
                align-items: center;
                gap: 1rem;
            }
            
            .notification-icon {
                font-size: 1.5rem;
            }
            
            .notification-text {
                font-weight: 600;
            }
            
            .notification-progress {
                color: var(--text-secondary);
                font-size: 0.9rem;
            }
            .analytics-section {
                background: var(--card-bg);
                border-radius: 20px;
                padding: 2rem;
                margin-bottom: 3rem;
                border: 1px solid var(--border-color);
            }
            .analytics-title {
                font-size: 1.8rem;
                font-weight: 700;
                margin-bottom: 2rem;
                background: linear-gradient(45deg, var(--rust-orange), var(--async-blue));
                background-clip: text;
                -webkit-background-clip: text;
                -webkit-text-fill-color: transparent;
            }
            .analytics-grid {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
                gap: 2rem;
            }
            .analytics-card {
                background: var(--surface-bg);
                border-radius: 12px;
                padding: 1.5rem;
                border: 1px solid var(--border-color);
            }
            .analytics-card h3 {
                margin-bottom: 1rem;
                color: var(--text-primary);
            }
            .insight-metrics {
                display: flex;
                flex-direction: column;
                gap: 0.75rem;
            }
            .insight-metric {
                display: flex;
                justify-content: space-between;
                padding: 0.5rem;
                background: var(--card-bg);
                border-radius: 6px;
                font-size: 0.9rem;
            }
            .insight-metric .label {
                color: var(--text-secondary);
            }
            .insight-metric .value {
                font-weight: 600;
                color: var(--rust-orange);
            }
        `;
        
        // Inject styles
        const styleSheet = document.createElement('style');
        styleSheet.textContent = detailsStyles;
        document.head.appendChild(styleSheet);
        
    </script>
</body>
</html>
        "#.to_string()
    }

    /// Generate HTML using Handlebars template (NEW)
    fn generate_templated_html_report(
        &self,
        profiles: &HashMap<TaskId, TaskResourceProfile>,
    ) -> Result<String, VisualizationError> {
        let template_content = Self::get_html_template();

        // Create Handlebars registry
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("async_dashboard", template_content)
            .map_err(|e| {
                VisualizationError::TemplateError(format!("Failed to register template: {}", e))
            })?;

        // Build template data
        let template_data = self.build_template_data(profiles)?;

        // Render template
        let rendered = handlebars
            .render("async_dashboard", &template_data)
            .map_err(|e| {
                VisualizationError::TemplateError(format!("Failed to render template: {}", e))
            })?;

        Ok(rendered)
    }

    /// Generate hardcoded HTML (ORIGINAL method)
    fn generate_hardcoded_html_report(
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
    fn calculate_baselines(
        &self,
        profiles: &HashMap<TaskId, TaskResourceProfile>,
    ) -> PerformanceBaselines {
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
    fn calculate_rankings(
        &self,
        profiles: &HashMap<TaskId, TaskResourceProfile>,
    ) -> HashMap<TaskId, CategoryRanking> {
        let mut rankings = HashMap::new();
        let mut category_groups: HashMap<String, Vec<(TaskId, &TaskResourceProfile)>> =
            HashMap::new();

        // Group by task type
        for (task_id, profile) in profiles {
            let category = format!("{:?}", profile.task_type);
            category_groups
                .entry(category)
                .or_default()
                .push((*task_id, profile));
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
            let cpu_comp = self
                .compare_to_baseline(profile.cpu_metrics.usage_percent, baselines.avg_cpu_percent);
            let memory_comp = self.compare_to_baseline(
                profile.memory_metrics.current_bytes as f64 / 1_048_576.0,
                baselines.avg_memory_mb,
            );
            let io_comp =
                self.compare_to_baseline(profile.io_metrics.bandwidth_mbps, baselines.avg_io_mbps);
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

        format!(
            r#"<!DOCTYPE html>
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
"#,
            self.config.title, theme_styles, self.config.title
        )
    }

    /// Generate summary statistics section
    fn generate_summary_section(&self, analytics: &PerformanceAnalytics) -> String {
        format!(
            r#"
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

        html.push_str(
            r#"
        <div class="tasks-section">
            <h2 class="section-title">Task Performance Details</h2>
            <div class="tasks-grid">
"#,
        );

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

        html.push_str(
            r#"
            </div>
        </div>
"#,
        );

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
            format!(
                r#"<div class="ranking-badge {}">#{}/{}</div>"#,
                rank_class, ranking.rank, ranking.total_in_category
            )
        } else {
            String::new()
        };

        if let Some(comp) = comparisons {
            self.generate_comparison_info(comp)
        } else {
            String::new()
        };

        let efficiency_tooltip = if self.config.include_efficiency_breakdown {
            format!(
                r#"
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
                profile
                    .efficiency_explanation
                    .component_scores
                    .cpu_efficiency
                    * 100.0,
                profile
                    .efficiency_explanation
                    .component_scores
                    .memory_efficiency
                    * 100.0,
                profile
                    .efficiency_explanation
                    .component_scores
                    .io_efficiency
                    * 100.0,
                profile
                    .efficiency_explanation
                    .component_scores
                    .network_efficiency
                    * 100.0,
                profile.efficiency_score * 100.0
            )
        } else {
            String::new()
        };

        Ok(format!(
            r#"
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
            if let Some(comp) = comparisons {
                format!(
                    "<div class=\"metric-comparison {}\">{}</div>",
                    self.get_comparison_class(&comp.cpu),
                    self.format_comparison(&comp.cpu)
                )
            } else {
                String::new()
            },
            profile.memory_metrics.current_bytes as f64 / 1_048_576.0,
            if let Some(comp) = comparisons {
                format!(
                    "<div class=\"metric-comparison {}\">{}</div>",
                    self.get_comparison_class(&comp.memory),
                    self.format_comparison(&comp.memory)
                )
            } else {
                String::new()
            },
            profile.io_metrics.bandwidth_mbps,
            if let Some(comp) = comparisons {
                format!(
                    "<div class=\"metric-comparison {}\">{}</div>",
                    self.get_comparison_class(&comp.io),
                    self.format_comparison(&comp.io)
                )
            } else {
                String::new()
            },
            profile.network_metrics.throughput_mbps,
            if let Some(comp) = comparisons {
                format!(
                    "<div class=\"metric-comparison {}\">{}</div>",
                    self.get_comparison_class(&comp.network),
                    self.format_comparison(&comp.network)
                )
            } else {
                String::new()
            },
            efficiency_tooltip,
            profile.efficiency_score * 100.0,
            profile.efficiency_score * 100.0,
            profile.source_location.file_path,
            profile.source_location.line_number,
            profile.source_location.function_name
        ))
    }

    /// Generate comparison information display
    fn generate_comparison_info(&self, _comparisons: &TaskComparisons) -> String {
        // Implementation for comparison display
        String::new()
    }

    /// Format comparison for display
    fn format_comparison(&self, comparison: &PerformanceComparison) -> String {
        match comparison.comparison_type {
            ComparisonType::NearAverage => "(≈ avg)".to_string(),
            ComparisonType::AboveAverage => {
                format!("(+{:.1}% vs avg)", comparison.difference_percent.abs())
            }
            ComparisonType::BelowAverage => {
                format!("(-{:.1}% vs avg)", comparison.difference_percent.abs())
            }
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
    fn generate_charts_section(
        &self,
        profiles: &HashMap<TaskId, TaskResourceProfile>,
    ) -> Result<String, VisualizationError> {
        let mut html = String::new();

        html.push_str(
            r#"
        <div class="charts-section">
            <h2 class="section-title">📈 Performance Trends</h2>
"#,
        );

        // Generate simple CSS charts
        let chart_html = self.generate_chart_scripts(profiles)?;
        html.push_str(&chart_html);

        html.push_str(
            r#"
        </div>
"#,
        );

        Ok(html)
    }

    /// Build enhanced template data from task profiles with comprehensive Rust async metrics
    fn build_template_data(
        &self,
        profiles: &HashMap<TaskId, TaskResourceProfile>,
    ) -> Result<serde_json::Value, VisualizationError> {
        if profiles.is_empty() {
            return Err(VisualizationError::NoDataAvailable);
        }

        // Calculate aggregated metrics
        let total_tasks = profiles.len();
        let cpu_usage_avg = profiles
            .values()
            .map(|p| p.cpu_metrics.usage_percent)
            .sum::<f64>()
            / total_tasks as f64;
        let cpu_usage_peak = profiles
            .values()
            .map(|p| p.cpu_metrics.usage_percent)
            .fold(0.0f64, |a, b| a.max(b));
        let total_memory_mb = profiles
            .values()
            .map(|p| p.memory_metrics.allocated_bytes as f64 / 1024.0 / 1024.0)
            .sum::<f64>();
        let peak_memory_mb = profiles
            .values()
            .map(|p| p.memory_metrics.peak_bytes as f64 / 1024.0 / 1024.0)
            .fold(0.0f64, |a, b| a.max(b));

        // Enhanced async-specific metrics
        let total_context_switches = profiles
            .values()
            .map(|p| p.cpu_metrics.context_switches)
            .sum::<u64>();
        let total_allocations = profiles
            .values()
            .map(|p| p.memory_metrics.allocation_count)
            .sum::<u64>();
        let avg_efficiency =
            profiles.values().map(|p| p.efficiency_score).sum::<f64>() / total_tasks as f64;

        // Network and I/O totals
        let total_io_ops = profiles
            .values()
            .map(|p| p.io_metrics.read_operations + p.io_metrics.write_operations)
            .sum::<u64>();
        let total_read_mb = profiles
            .values()
            .map(|p| p.io_metrics.bytes_read as f64 / 1024.0 / 1024.0)
            .sum::<f64>();
        let total_write_mb = profiles
            .values()
            .map(|p| p.io_metrics.bytes_written as f64 / 1024.0 / 1024.0)
            .sum::<f64>();
        let io_throughput = if total_tasks > 0 {
            (total_read_mb + total_write_mb) / total_tasks as f64
        } else {
            0.0
        };

        let total_sent_mb = profiles
            .values()
            .map(|p| p.network_metrics.bytes_sent as f64 / 1024.0 / 1024.0)
            .sum::<f64>();
        let total_received_mb = profiles
            .values()
            .map(|p| p.network_metrics.bytes_received as f64 / 1024.0 / 1024.0)
            .sum::<f64>();
        let network_throughput = if total_tasks > 0 {
            profiles
                .values()
                .map(|p| p.network_metrics.throughput_mbps)
                .sum::<f64>()
                / total_tasks as f64
        } else {
            0.0
        };
        let avg_latency = if total_tasks > 0 {
            profiles
                .values()
                .map(|p| p.network_metrics.latency_avg_ms)
                .sum::<f64>()
                / total_tasks as f64
        } else {
            0.0
        };

        // Count task types and calculate efficiency by type
        let task_type_counts = self.calculate_task_type_metrics(profiles);

        // Build categorized task data
        let cpu_intensive_tasks =
            self.build_task_category_data(profiles, &crate::async_memory::TaskType::CpuIntensive);
        let memory_intensive_tasks = self
            .build_task_category_data(profiles, &crate::async_memory::TaskType::MemoryIntensive);
        let io_intensive_tasks =
            self.build_task_category_data(profiles, &crate::async_memory::TaskType::IoIntensive);
        let network_intensive_tasks = self
            .build_task_category_data(profiles, &crate::async_memory::TaskType::NetworkIntensive);

        // Async-specific metrics for Rust
        let futures_count = total_tasks; // Each task represents a Future
        let total_polls = total_context_switches; // Context switches approximate polling
        let avg_poll_time = if total_polls > 0 {
            cpu_usage_avg * 10.0
        } else {
            0.0
        }; // Estimated poll time in microseconds
        let ready_rate = if total_tasks > 0 {
            avg_efficiency * 100.0
        } else {
            0.0
        };

        // Build template data in smaller chunks to avoid recursion limit
        let mut template_data = serde_json::Map::new();

        // Basic info
        template_data.insert(
            "title".to_string(),
            serde_json::Value::String("Rust Async Performance Analysis".to_string()),
        );
        template_data.insert("subtitle".to_string(), serde_json::Value::String(format!("Advanced analysis of {} Rust async tasks with detailed performance metrics and Future polling insights", total_tasks)));

        // Core metrics
        template_data.insert(
            "total_tasks".to_string(),
            serde_json::Value::Number(serde_json::Number::from(total_tasks)),
        );
        template_data.insert(
            "active_tasks".to_string(),
            serde_json::Value::Number(serde_json::Number::from(0)),
        );
        template_data.insert(
            "completed_tasks".to_string(),
            serde_json::Value::Number(serde_json::Number::from(total_tasks)),
        );
        template_data.insert(
            "failed_tasks".to_string(),
            serde_json::Value::Number(serde_json::Number::from(0)),
        );

        // CPU metrics
        template_data.insert(
            "cpu_usage_avg".to_string(),
            serde_json::Value::String(format!("{:.1}", cpu_usage_avg)),
        );
        template_data.insert(
            "cpu_usage_peak".to_string(),
            serde_json::Value::String(format!("{:.1}", cpu_usage_peak)),
        );
        template_data.insert(
            "cpu_cores".to_string(),
            serde_json::Value::Number(serde_json::Number::from(8)),
        );
        template_data.insert(
            "context_switches".to_string(),
            serde_json::Value::Number(serde_json::Number::from(total_context_switches)),
        );

        // Memory metrics
        template_data.insert(
            "total_memory_mb".to_string(),
            serde_json::Value::String(format!("{:.1}", total_memory_mb)),
        );
        template_data.insert(
            "peak_memory_mb".to_string(),
            serde_json::Value::String(format!("{:.1}", peak_memory_mb)),
        );
        template_data.insert(
            "total_allocations".to_string(),
            serde_json::Value::Number(serde_json::Number::from(total_allocations)),
        );
        template_data.insert(
            "memory_efficiency".to_string(),
            serde_json::Value::String(format!("{:.1}", avg_efficiency * 100.0)),
        );

        // I/O metrics
        template_data.insert(
            "io_throughput".to_string(),
            serde_json::Value::String(format!("{:.1}", io_throughput)),
        );
        template_data.insert(
            "total_read_mb".to_string(),
            serde_json::Value::String(format!("{:.1}", total_read_mb)),
        );
        template_data.insert(
            "total_write_mb".to_string(),
            serde_json::Value::String(format!("{:.1}", total_write_mb)),
        );
        template_data.insert(
            "total_io_ops".to_string(),
            serde_json::Value::Number(serde_json::Number::from(total_io_ops)),
        );

        // Network metrics
        template_data.insert(
            "network_throughput".to_string(),
            serde_json::Value::String(format!("{:.1}", network_throughput)),
        );
        template_data.insert(
            "total_sent_mb".to_string(),
            serde_json::Value::String(format!("{:.1}", total_sent_mb)),
        );
        template_data.insert(
            "total_received_mb".to_string(),
            serde_json::Value::String(format!("{:.1}", total_received_mb)),
        );
        template_data.insert(
            "avg_latency".to_string(),
            serde_json::Value::String(format!("{:.1}", avg_latency)),
        );

        // Overall efficiency
        let resource_balance =
            profiles.values().map(|p| p.resource_balance).sum::<f64>() / total_tasks as f64 * 100.0;
        let bottleneck_count = profiles
            .values()
            .filter(|p| {
                !matches!(
                    p.bottleneck_type,
                    crate::async_memory::BottleneckType::Balanced
                )
            })
            .count();

        template_data.insert(
            "efficiency_score".to_string(),
            serde_json::Value::String(format!("{:.1}", avg_efficiency * 100.0)),
        );
        template_data.insert(
            "resource_balance".to_string(),
            serde_json::Value::String(format!("{:.1}", resource_balance)),
        );
        template_data.insert(
            "bottleneck_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(bottleneck_count)),
        );
        template_data.insert(
            "optimization_potential".to_string(),
            serde_json::Value::String(format!("{:.1}", (1.0 - avg_efficiency) * 100.0)),
        );

        // Async-specific Rust metrics
        template_data.insert(
            "futures_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(futures_count)),
        );
        template_data.insert(
            "total_polls".to_string(),
            serde_json::Value::Number(serde_json::Number::from(total_polls)),
        );
        template_data.insert(
            "avg_poll_time".to_string(),
            serde_json::Value::String(format!("{:.1}", avg_poll_time)),
        );
        template_data.insert(
            "ready_rate".to_string(),
            serde_json::Value::String(format!("{:.1}", ready_rate)),
        );

        // Task type counts and efficiency
        template_data.insert(
            "cpu_intensive_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(task_type_counts.cpu_count)),
        );
        template_data.insert(
            "cpu_avg_efficiency".to_string(),
            serde_json::Value::String(format!("{:.1}", task_type_counts.cpu_efficiency)),
        );
        template_data.insert(
            "memory_intensive_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(task_type_counts.memory_count)),
        );
        template_data.insert(
            "memory_avg_efficiency".to_string(),
            serde_json::Value::String(format!("{:.1}", task_type_counts.memory_efficiency)),
        );
        template_data.insert(
            "io_intensive_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(task_type_counts.io_count)),
        );
        template_data.insert(
            "io_avg_efficiency".to_string(),
            serde_json::Value::String(format!("{:.1}", task_type_counts.io_efficiency)),
        );
        template_data.insert(
            "network_intensive_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(task_type_counts.network_count)),
        );
        template_data.insert(
            "network_avg_efficiency".to_string(),
            serde_json::Value::String(format!("{:.1}", task_type_counts.network_efficiency)),
        );

        // Categorized task data
        template_data.insert(
            "cpu_intensive_tasks".to_string(),
            serde_json::Value::Array(cpu_intensive_tasks),
        );
        template_data.insert(
            "memory_intensive_tasks".to_string(),
            serde_json::Value::Array(memory_intensive_tasks),
        );
        template_data.insert(
            "io_intensive_tasks".to_string(),
            serde_json::Value::Array(io_intensive_tasks),
        );
        template_data.insert(
            "network_intensive_tasks".to_string(),
            serde_json::Value::Array(network_intensive_tasks),
        );

        // Advanced analytics data
        let avg_fragmentation = profiles
            .values()
            .map(|p| p.memory_metrics.heap_fragmentation)
            .sum::<f64>()
            / total_tasks as f64
            * 100.0;
        let blocking_tasks_count = profiles
            .values()
            .filter(|p| p.cpu_metrics.usage_percent > 80.0)
            .count();

        template_data.insert(
            "avg_poll_duration".to_string(),
            serde_json::Value::String(format!("{:.1}", avg_poll_time)),
        );
        template_data.insert(
            "immediate_ready_percent".to_string(),
            serde_json::Value::String(format!("{:.1}", ready_rate * 0.8)),
        );
        template_data.insert(
            "waker_efficiency".to_string(),
            serde_json::Value::String(format!("{:.1}", avg_efficiency * 95.0)),
        );
        template_data.insert(
            "peak_alloc_rate".to_string(),
            serde_json::Value::String(format!("{}", total_allocations / total_tasks as u64)),
        );
        template_data.insert(
            "avg_fragmentation".to_string(),
            serde_json::Value::String(format!("{:.1}", avg_fragmentation)),
        );
        template_data.insert(
            "gc_pressure".to_string(),
            serde_json::Value::String(format!("{:.1}", (1.0 - avg_efficiency) * 50.0)),
        );
        template_data.insert(
            "executor_utilization".to_string(),
            serde_json::Value::String(format!("{:.1}", cpu_usage_avg * 0.8)),
        );
        template_data.insert(
            "avg_queue_length".to_string(),
            serde_json::Value::String(format!("{:.1}", total_tasks as f64 * 0.1)),
        );
        template_data.insert(
            "blocking_tasks_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(blocking_tasks_count)),
        );
        template_data.insert(
            "deadlock_risk".to_string(),
            serde_json::Value::String(format!(
                "{:.1}",
                if total_tasks > 10 {
                    total_tasks as f64 * 0.05
                } else {
                    0.0
                }
            )),
        );

        let template_data = serde_json::Value::Object(template_data);

        Ok(template_data)
    }

    /// Calculate task type metrics and efficiency (legacy method for compatibility)
    fn calculate_task_type_metrics(
        &self,
        profiles: &HashMap<TaskId, TaskResourceProfile>,
    ) -> TaskTypeMetrics {
        let mut cpu_count = 0;
        let mut cpu_efficiency_sum = 0.0;
        let mut memory_count = 0;
        let mut memory_efficiency_sum = 0.0;
        let mut io_count = 0;
        let mut io_efficiency_sum = 0.0;
        let mut network_count = 0;
        let mut network_efficiency_sum = 0.0;

        for profile in profiles.values() {
            match profile.task_type {
                crate::async_memory::TaskType::CpuIntensive => {
                    cpu_count += 1;
                    cpu_efficiency_sum += profile.efficiency_score;
                }
                crate::async_memory::TaskType::MemoryIntensive => {
                    memory_count += 1;
                    memory_efficiency_sum += profile.efficiency_score;
                }
                crate::async_memory::TaskType::IoIntensive => {
                    io_count += 1;
                    io_efficiency_sum += profile.efficiency_score;
                }
                crate::async_memory::TaskType::NetworkIntensive => {
                    network_count += 1;
                    network_efficiency_sum += profile.efficiency_score;
                }
                _ => {} // Handle other types as needed
            }
        }

        TaskTypeMetrics {
            cpu_count,
            cpu_efficiency: if cpu_count > 0 {
                cpu_efficiency_sum / cpu_count as f64 * 100.0
            } else {
                0.0
            },
            memory_count,
            memory_efficiency: if memory_count > 0 {
                memory_efficiency_sum / memory_count as f64 * 100.0
            } else {
                0.0
            },
            io_count,
            io_efficiency: if io_count > 0 {
                io_efficiency_sum / io_count as f64 * 100.0
            } else {
                0.0
            },
            network_count,
            network_efficiency: if network_count > 0 {
                network_efficiency_sum / network_count as f64 * 100.0
            } else {
                0.0
            },
        }
    }

    /// Build task data for specific category
    fn build_task_category_data(
        &self,
        profiles: &HashMap<TaskId, TaskResourceProfile>,
        task_type: &crate::async_memory::TaskType,
    ) -> Vec<serde_json::Value> {
        profiles
            .iter()
            .filter(|(_, profile)| {
                std::mem::discriminant(&profile.task_type) == std::mem::discriminant(task_type)
            })
            .map(|(task_id, profile)| {
                let mut task_data = serde_json::Map::new();

                // Basic task info
                task_data.insert(
                    "task_id".to_string(),
                    serde_json::Value::String(task_id.to_string()),
                );
                task_data.insert(
                    "task_name".to_string(),
                    serde_json::Value::String(profile.task_name.clone()),
                );
                task_data.insert(
                    "source_file".to_string(),
                    serde_json::Value::String(
                        profile
                            .source_location
                            .file_path
                            .split('/')
                            .next_back()
                            .unwrap_or("unknown.rs")
                            .to_string(),
                    ),
                );
                task_data.insert(
                    "source_line".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(
                        profile.source_location.line_number,
                    )),
                );
                task_data.insert(
                    "status".to_string(),
                    serde_json::Value::String("completed".to_string()),
                );
                task_data.insert(
                    "status_class".to_string(),
                    serde_json::Value::String("completed".to_string()),
                );
                task_data.insert(
                    "duration_ms".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(profile.duration_ms.unwrap_or(0.0))
                            .unwrap_or(serde_json::Number::from(0)),
                    ),
                );

                // CPU-specific metrics
                task_data.insert(
                    "cpu_usage".to_string(),
                    serde_json::Value::String(format!("{:.1}", profile.cpu_metrics.usage_percent)),
                );
                task_data.insert(
                    "cpu_cycles".to_string(),
                    serde_json::Value::String(format!(
                        "{:.1}",
                        profile.cpu_metrics.cpu_cycles as f64 / 1_000_000.0
                    )),
                );
                task_data.insert(
                    "instructions".to_string(),
                    serde_json::Value::String(format!(
                        "{:.1}",
                        profile.cpu_metrics.instructions as f64 / 1_000_000.0
                    )),
                );
                task_data.insert(
                    "cache_misses".to_string(),
                    serde_json::Value::String(format!(
                        "{:.1}",
                        profile.cpu_metrics.cache_misses as f64 / 1_000.0
                    )),
                );

                // Memory-specific metrics
                task_data.insert(
                    "allocated_mb".to_string(),
                    serde_json::Value::String(format!(
                        "{:.1}",
                        profile.memory_metrics.allocated_bytes as f64 / 1024.0 / 1024.0
                    )),
                );
                task_data.insert(
                    "peak_memory_mb".to_string(),
                    serde_json::Value::String(format!(
                        "{:.1}",
                        profile.memory_metrics.peak_bytes as f64 / 1024.0 / 1024.0
                    )),
                );
                task_data.insert(
                    "allocation_count".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(
                        profile.memory_metrics.allocation_count,
                    )),
                );
                task_data.insert(
                    "heap_fragmentation".to_string(),
                    serde_json::Value::String(format!(
                        "{:.1}",
                        profile.memory_metrics.heap_fragmentation * 100.0
                    )),
                );
                task_data.insert(
                    "memory_usage_percent".to_string(),
                    serde_json::Value::String(format!(
                        "{:.1}",
                        (profile.memory_metrics.current_bytes as f64
                            / profile.memory_metrics.allocated_bytes.max(1) as f64)
                            * 100.0
                    )),
                );

                // I/O-specific metrics
                task_data.insert(
                    "bytes_read_mb".to_string(),
                    serde_json::Value::String(format!(
                        "{:.1}",
                        profile.io_metrics.bytes_read as f64 / 1024.0 / 1024.0
                    )),
                );
                task_data.insert(
                    "bytes_written_mb".to_string(),
                    serde_json::Value::String(format!(
                        "{:.1}",
                        profile.io_metrics.bytes_written as f64 / 1024.0 / 1024.0
                    )),
                );
                task_data.insert(
                    "avg_latency_us".to_string(),
                    serde_json::Value::String(format!("{:.1}", profile.io_metrics.avg_latency_us)),
                );
                task_data.insert(
                    "queue_depth".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(
                        profile.io_metrics.queue_depth,
                    )),
                );
                task_data.insert(
                    "io_usage_percent".to_string(),
                    serde_json::Value::String(format!("{:.1}", profile.io_metrics.io_wait_percent)),
                );

                // Network-specific metrics
                task_data.insert(
                    "bytes_sent_mb".to_string(),
                    serde_json::Value::String(format!(
                        "{:.1}",
                        profile.network_metrics.bytes_sent as f64 / 1024.0 / 1024.0
                    )),
                );
                task_data.insert(
                    "bytes_received_mb".to_string(),
                    serde_json::Value::String(format!(
                        "{:.1}",
                        profile.network_metrics.bytes_received as f64 / 1024.0 / 1024.0
                    )),
                );
                task_data.insert(
                    "active_connections".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(
                        profile.network_metrics.connections_active,
                    )),
                );
                task_data.insert(
                    "avg_latency_ms".to_string(),
                    serde_json::Value::String(format!(
                        "{:.1}",
                        profile.network_metrics.latency_avg_ms
                    )),
                );
                task_data.insert(
                    "network_usage_percent".to_string(),
                    serde_json::Value::String(format!(
                        "{:.1}",
                        (profile.network_metrics.throughput_mbps / 100.0).min(100.0)
                    )),
                );

                serde_json::Value::Object(task_data)
            })
            .collect()
    }

    /// Generate simple CSS charts (no JavaScript)
    fn generate_chart_scripts(
        &self,
        profiles: &HashMap<TaskId, TaskResourceProfile>,
    ) -> Result<String, VisualizationError> {
        let mut cpu_bars = String::new();
        let mut memory_bars = String::new();

        // Find max values for scaling
        let max_cpu = profiles
            .values()
            .map(|p| p.cpu_metrics.usage_percent)
            .fold(0.0, f64::max)
            .max(100.0);
        let max_memory = profiles
            .values()
            .map(|p| p.memory_metrics.current_bytes as f64 / 1_048_576.0)
            .fold(0.0, f64::max);

        for profile in profiles.values() {
            let cpu_percent = profile.cpu_metrics.usage_percent;
            let memory_mb = profile.memory_metrics.current_bytes as f64 / 1_048_576.0;

            let cpu_width = (cpu_percent / max_cpu * 100.0).min(100.0);
            let memory_width = if max_memory > 0.0 {
                (memory_mb / max_memory * 100.0).min(100.0)
            } else {
                0.0
            };

            cpu_bars.push_str(&format!(
                r#"
                <div class="chart-bar">
                    <div class="bar-label">{}</div>
                    <div class="bar-container">
                        <div class="bar-fill cpu-bar" style="width: {:.1}%"></div>
                        <div class="bar-value">{:.1}%</div>
                    </div>
                </div>
"#,
                profile.task_name, cpu_width, cpu_percent
            ));

            memory_bars.push_str(&format!(
                r#"
                <div class="chart-bar">
                    <div class="bar-label">{}</div>
                    <div class="bar-container">
                        <div class="bar-fill memory-bar" style="width: {:.1}%"></div>
                        <div class="bar-value">{:.1}MB</div>
                    </div>
                </div>
"#,
                profile.task_name, memory_width, memory_mb
            ));
        }

        // Generate network bars
        let mut network_bars = String::new();
        let max_network = profiles
            .values()
            .map(|p| p.network_metrics.throughput_mbps)
            .fold(0.0, f64::max);

        for profile in profiles.values() {
            let network_mbps = profile.network_metrics.throughput_mbps;
            let network_width = if max_network > 0.0 {
                (network_mbps / max_network * 100.0).min(100.0)
            } else {
                0.0
            };

            network_bars.push_str(&format!(
                r#"
                <div class="chart-bar">
                    <div class="bar-label">{}</div>
                    <div class="bar-container">
                        <div class="bar-fill network-bar" style="width: {:.1}%"></div>
                        <div class="bar-value">{:.1}Mbps</div>
                    </div>
                </div>
"#,
                profile.task_name, network_width, network_mbps
            ));
        }

        Ok(format!(
            r#"
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
"#,
            cpu_bars, memory_bars, network_bars
        ))
    }

    /// Generate HTML footer
    fn generate_html_footer(&self) -> String {
        r#"
    </div>
</body>
</html>
"#
        .to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::async_memory::resource_monitor::{
        BottleneckType, ComponentScores, CpuMetrics, CriticalPathAnalysis, EfficiencyExplanation,
        HotMetrics, IoMetrics, MemoryMetrics, NetworkMetrics, SourceLocation, TaskResourceProfile,
        TaskType,
    };

    /// Helper function to create a test task profile
    fn create_test_profile(
        task_name: &str,
        task_type: TaskType,
        cpu_usage: f64,
        memory_bytes: u64,
        efficiency: f64,
    ) -> TaskResourceProfile {
        TaskResourceProfile {
            task_id: 1u128,
            task_name: task_name.to_string(),
            task_type,
            start_time: 1000,
            end_time: Some(2000),
            duration_ms: Some(1000.0),
            cpu_metrics: CpuMetrics {
                usage_percent: cpu_usage,
                time_user_ms: 100.0,
                time_kernel_ms: 50.0,
                context_switches: 10,
                cpu_cycles: 1000000,
                instructions: 500000,
                cache_misses: 100,
                branch_misses: 50,
                core_affinity: vec![0],
            },
            memory_metrics: MemoryMetrics {
                allocated_bytes: memory_bytes,
                peak_bytes: memory_bytes + 1024,
                current_bytes: memory_bytes,
                allocation_count: 5,
                deallocation_count: 3,
                page_faults: 10,
                heap_fragmentation: 0.1,
                memory_bandwidth_mbps: 1000.0,
            },
            io_metrics: IoMetrics {
                bytes_read: 1024,
                bytes_written: 512,
                read_operations: 10,
                write_operations: 5,
                sync_operations: 3,
                async_operations: 12,
                avg_latency_us: 100.0,
                bandwidth_mbps: 10.0,
                queue_depth: 4,
                io_wait_percent: 5.0,
            },
            network_metrics: NetworkMetrics {
                bytes_sent: 2048,
                bytes_received: 1536,
                packets_sent: 100,
                packets_received: 95,
                connections_active: 5,
                connections_established: 10,
                connection_errors: 1,
                latency_avg_ms: 10.0,
                throughput_mbps: 5.0,
                retransmissions: 2,
            },
            gpu_metrics: None,
            efficiency_score: efficiency,
            resource_balance: 0.8,
            bottleneck_type: BottleneckType::Balanced,
            source_location: SourceLocation {
                file_path: "test.rs".to_string(),
                line_number: 42,
                function_name: "test_function".to_string(),
                module_path: "test::module".to_string(),
                crate_name: "test_crate".to_string(),
            },
            hot_metrics: HotMetrics {
                cpu_hotspots: vec![],
                memory_hotspots: vec![],
                io_hotspots: vec![],
                network_hotspots: vec![],
                critical_path_analysis: CriticalPathAnalysis {
                    total_execution_time_ms: 1000.0,
                    critical_path_time_ms: 800.0,
                    parallelization_potential: 0.5,
                    blocking_operations: vec![],
                },
            },
            efficiency_explanation: EfficiencyExplanation {
                overall_score: efficiency,
                component_scores: ComponentScores {
                    cpu_efficiency: efficiency,
                    memory_efficiency: efficiency,
                    io_efficiency: efficiency,
                    network_efficiency: efficiency,
                    resource_balance: 0.8,
                },
                recommendations: vec![],
                bottleneck_analysis: "No bottlenecks detected".to_string(),
                optimization_potential: 0.2,
            },
        }
    }

    #[test]
    fn test_visualization_config_default() {
        let config = VisualizationConfig::default();

        assert_eq!(config.title, "Async Task Performance Analysis");
        assert!(matches!(config.theme, Theme::Dark));
        assert!(config.include_charts);
        assert!(config.include_baselines);
        assert!(config.include_rankings);
        assert!(config.include_efficiency_breakdown);
    }

    #[test]
    fn test_visualization_generator_new() {
        let generator = VisualizationGenerator::new();
        assert_eq!(generator.config.title, "Async Task Performance Analysis");
    }

    #[test]
    fn test_visualization_generator_with_config() {
        let custom_config = VisualizationConfig {
            title: "Custom Analysis".to_string(),
            theme: Theme::Light,
            include_charts: false,
            include_baselines: false,
            include_rankings: false,
            include_efficiency_breakdown: false,
        };

        let generator = VisualizationGenerator::with_config(custom_config.clone());
        assert_eq!(generator.config.title, "Custom Analysis");
        assert!(matches!(generator.config.theme, Theme::Light));
        assert!(!generator.config.include_charts);
    }

    #[test]
    fn test_calculate_baselines() {
        let generator = VisualizationGenerator::new();
        let mut profiles = HashMap::new();

        profiles.insert(
            1u128,
            create_test_profile("task1", TaskType::CpuIntensive, 50.0, 1024 * 1024, 0.8),
        );
        profiles.insert(
            2u128,
            create_test_profile("task2", TaskType::IoIntensive, 30.0, 2048 * 1024, 0.6),
        );

        let baselines = generator.calculate_baselines(&profiles);

        assert_eq!(baselines.avg_cpu_percent, 40.0);
        assert_eq!(baselines.avg_memory_mb, 1.5);
        assert_eq!(baselines.avg_io_mbps, 10.0);
        assert_eq!(baselines.avg_network_mbps, 5.0);
        assert_eq!(baselines.avg_efficiency_score, 0.7);
    }

    #[test]
    fn test_calculate_rankings() {
        let generator = VisualizationGenerator::new();
        let mut profiles = HashMap::new();

        // Create tasks with different efficiency scores in same category
        profiles.insert(
            1u128,
            create_test_profile("task1", TaskType::CpuIntensive, 50.0, 1024 * 1024, 0.9),
        );
        profiles.insert(
            2u128,
            create_test_profile("task2", TaskType::CpuIntensive, 30.0, 2048 * 1024, 0.7),
        );
        profiles.insert(
            3u128,
            create_test_profile("task3", TaskType::IoIntensive, 20.0, 512 * 1024, 0.8),
        );

        let rankings = generator.calculate_rankings(&profiles);

        // Check CPU intensive tasks ranking
        let task1_ranking = rankings.get(&1u128).expect("Task 1 should have ranking");
        let task2_ranking = rankings.get(&2u128).expect("Task 2 should have ranking");

        assert_eq!(task1_ranking.rank, 1); // Higher efficiency should rank first
        assert_eq!(task1_ranking.total_in_category, 2);
        assert_eq!(task1_ranking.category_name, "CpuIntensive");

        assert_eq!(task2_ranking.rank, 2);
        assert_eq!(task2_ranking.total_in_category, 2);

        // Check IO intensive task ranking
        let task3_ranking = rankings.get(&3u128).expect("Task 3 should have ranking");
        assert_eq!(task3_ranking.rank, 1);
        assert_eq!(task3_ranking.total_in_category, 1);
        assert_eq!(task3_ranking.category_name, "IoIntensive");
    }

    #[test]
    fn test_compare_to_baseline() {
        let generator = VisualizationGenerator::new();

        // Test above average
        let comp = generator.compare_to_baseline(110.0, 100.0);
        assert!(matches!(comp.comparison_type, ComparisonType::AboveAverage));
        assert_eq!(comp.difference_percent, 10.0);

        // Test below average
        let comp = generator.compare_to_baseline(90.0, 100.0);
        assert!(matches!(comp.comparison_type, ComparisonType::BelowAverage));
        assert_eq!(comp.difference_percent, -10.0);

        // Test near average
        let comp = generator.compare_to_baseline(102.0, 100.0);
        assert!(matches!(comp.comparison_type, ComparisonType::NearAverage));
        assert_eq!(comp.difference_percent, 2.0);

        // Test zero baseline
        let comp = generator.compare_to_baseline(50.0, 0.0);
        assert_eq!(comp.difference_percent, 0.0);
    }

    #[test]
    fn test_analyze_profiles_empty() {
        let generator = VisualizationGenerator::new();
        let profiles = HashMap::new();

        let result = generator.analyze_profiles(&profiles);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VisualizationError::NoDataAvailable
        ));
    }

    #[test]
    fn test_format_comparison() {
        let generator = VisualizationGenerator::new();

        let comp_above = PerformanceComparison {
            value: 110.0,
            baseline: 100.0,
            difference_percent: 10.5,
            comparison_type: ComparisonType::AboveAverage,
        };
        assert_eq!(generator.format_comparison(&comp_above), "(+10.5% vs avg)");

        let comp_below = PerformanceComparison {
            value: 85.0,
            baseline: 100.0,
            difference_percent: -15.0,
            comparison_type: ComparisonType::BelowAverage,
        };
        assert_eq!(generator.format_comparison(&comp_below), "(-15.0% vs avg)");

        let comp_average = PerformanceComparison {
            value: 102.0,
            baseline: 100.0,
            difference_percent: 2.0,
            comparison_type: ComparisonType::NearAverage,
        };
        assert_eq!(generator.format_comparison(&comp_average), "(≈ avg)");
    }

    #[test]
    fn test_get_comparison_class() {
        let generator = VisualizationGenerator::new();

        let comp_above = PerformanceComparison {
            value: 110.0,
            baseline: 100.0,
            difference_percent: 10.0,
            comparison_type: ComparisonType::AboveAverage,
        };
        assert_eq!(
            generator.get_comparison_class(&comp_above),
            "comparison-above"
        );

        let comp_below = PerformanceComparison {
            value: 90.0,
            baseline: 100.0,
            difference_percent: -10.0,
            comparison_type: ComparisonType::BelowAverage,
        };
        assert_eq!(
            generator.get_comparison_class(&comp_below),
            "comparison-below"
        );

        let comp_average = PerformanceComparison {
            value: 102.0,
            baseline: 100.0,
            difference_percent: 2.0,
            comparison_type: ComparisonType::NearAverage,
        };
        assert_eq!(
            generator.get_comparison_class(&comp_average),
            "comparison-average"
        );
    }

    #[test]
    fn test_theme_styles() {
        let generator = VisualizationGenerator::new();

        let dark_styles = generator.get_dark_theme_styles();
        assert!(dark_styles.contains("background: #0d1117"));
        assert!(dark_styles.contains("color: #f0f6fc"));

        let light_styles = generator.get_light_theme_styles();
        assert!(light_styles.contains("background: #ffffff"));
        assert!(light_styles.contains("color: #24292f"));
    }

    #[test]
    fn test_visualization_error_display() {
        let err = VisualizationError::NoDataAvailable;
        assert_eq!(format!("{}", err), "No data available for visualization");

        let err = VisualizationError::InvalidConfiguration("test error".to_string());
        assert_eq!(format!("{}", err), "Invalid configuration: test error");

        let err = VisualizationError::TemplateError("template failed".to_string());
        assert_eq!(
            format!("{}", err),
            "Template generation error: template failed"
        );
    }
}
