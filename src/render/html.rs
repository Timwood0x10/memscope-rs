//! HTML rendering implementation
//!
//! Provides HTML rendering for all tracking strategies with
//! strategy-specific dashboards.

use super::renderer::Renderer;
use crate::data::{TrackingSnapshot, TrackingStrategy, RenderResult};

/// HTML renderer
///
/// Generates strategy-specific HTML dashboards for memory tracking data.
pub struct HtmlRenderer;

impl HtmlRenderer {
    /// Create a new HTML renderer
    pub fn new() -> Self {
        Self
    }

    /// Generate Core strategy HTML
    fn render_core(&self, snapshot: &TrackingSnapshot) -> String {
        let allocations_json = serde_json::to_string(&snapshot.allocations).unwrap_or_default();
        let stats_json = serde_json::to_string(&snapshot.stats).unwrap_or_default();
        
        let html_template = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Core Memory Tracking Dashboard</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { 
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }
        .container { 
            max-width: 1400px; 
            margin: 0 auto; 
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            border-radius: 20px;
            padding: 40px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
        }
        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 40px;
            padding-bottom: 20px;
            border-bottom: 2px solid #e0e0e0;
        }
        .header h1 {
            font-size: 36px;
            font-weight: 700;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }
        .stats-grid { 
            display: grid; 
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); 
            gap: 20px; 
            margin-bottom: 40px; 
        }
        .stat-card {
            background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
            padding: 25px;
            border-radius: 15px;
            box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }
        .stat-card:hover {
            transform: translateY(-5px);
            box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
        }
        .stat-label { 
            font-size: 14px; 
            color: #666; 
            margin-bottom: 8px;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        .stat-value { 
            font-size: 32px; 
            font-weight: 700; 
            color: #333;
        }
        .stat-value.highlight {
            color: #667eea;
        }
        .section {
            margin-bottom: 40px;
        }
        .section h2 {
            font-size: 24px;
            font-weight: 600;
            margin-bottom: 20px;
            color: #333;
        }
        .allocation-table {
            width: 100%;
            border-collapse: collapse;
            background: white;
            border-radius: 10px;
            overflow: hidden;
            box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
        }
        .allocation-table th {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 15px;
            text-align: left;
            font-weight: 600;
        }
        .allocation-table td {
            padding: 12px 15px;
            border-bottom: 1px solid #e0e0e0;
        }
        .allocation-table tr:hover {
            background: #f5f5f5;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Core Memory Tracking</h1>
            <div style="color: #666; font-size: 14px;">TIMESTAMP</div>
        </div>

        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-label">Total Allocated</div>
                <div class="stat-value">TOTAL_ALLOCATED</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Current Allocated</div>
                <div class="stat-value highlight">CURRENT_ALLOCATED</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Peak Memory</div>
                <div class="stat-value">PEAK_MEMORY</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Fragmentation</div>
                <div class="stat-value">FRAGMENTATION%</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Average Size</div>
                <div class="stat-value">AVERAGE_SIZE</div>
            </div>
        </div>

        <div class="section">
            <h2>Active Allocations (ALLOCATIONS_COUNT)</h2>
            <table class="allocation-table">
                <thead>
                    <tr>
                        <th>Address</th>
                        <th>Size</th>
                        <th>Thread</th>
                        <th>Timestamp</th>
                    </tr>
                </thead>
                <tbody id="allocations-body">
                </tbody>
            </table>
        </div>
    </div>

    <script>
        const allocations = ALLOCATIONS_JSON;
        const stats = STATS_JSON;

        const allocationsBody = document.getElementById('allocations-body');
        allocations.forEach((alloc) => {
            const row = document.createElement('tr');
            row.innerHTML = `
                <td>0x${alloc.ptr.toString(16).padStart(16, '0')}</td>
                <td>${formatBytes(alloc.size)}</td>
                <td>${alloc.thread_id}</td>
                <td>${alloc.timestamp}</td>
            `;
            allocationsBody.appendChild(row);
        });

        function formatBytes(bytes) {
            if (bytes === 0) return '0 B';
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }
    </script>
</body>
</html>"#;

        html_template
            .replace("TIMESTAMP", &format_timestamp(snapshot.timestamp))
            .replace("TOTAL_ALLOCATED", &format_bytes(snapshot.stats.total_allocated))
            .replace("CURRENT_ALLOCATED", &format_bytes(snapshot.stats.current_allocated))
            .replace("PEAK_MEMORY", &format_bytes(snapshot.stats.peak_memory))
            .replace("FRAGMENTATION", &format!("{:.2}", snapshot.stats.fragmentation))
            .replace("AVERAGE_SIZE", &format_bytes(snapshot.stats.average_allocation_size))
            .replace("ALLOCATIONS_COUNT", &snapshot.allocations.len().to_string())
            .replace("ALLOCATIONS_JSON", &allocations_json)
            .replace("STATS_JSON", &stats_json)
    }

    /// Generate Lockfree strategy HTML
    fn render_lockfree(&self, snapshot: &TrackingSnapshot) -> String {
        let events_json = serde_json::to_string(&snapshot.events).unwrap_or_default();
        let stats_json = serde_json::to_string(&snapshot.stats).unwrap_or_default();

        let html_template = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Lockfree Memory Tracking Dashboard</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { 
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #11998e 0%, #38ef7d 100%);
            min-height: 100vh;
            padding: 20px;
        }
        .container { 
            max-width: 1400px; 
            margin: 0 auto; 
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            border-radius: 20px;
            padding: 40px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
        }
        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 40px;
            padding-bottom: 20px;
            border-bottom: 2px solid #e0e0e0;
        }
        .header h1 {
            font-size: 36px;
            font-weight: 700;
            background: linear-gradient(135deg, #11998e 0%, #38ef7d 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }
        .stats-grid { 
            display: grid; 
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); 
            gap: 20px; 
            margin-bottom: 40px; 
        }
        .stat-card {
            background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
            padding: 25px;
            border-radius: 15px;
            box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }
        .stat-card:hover {
            transform: translateY(-5px);
            box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
        }
        .stat-label { 
            font-size: 14px; 
            color: #666; 
            margin-bottom: 8px;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        .stat-value { 
            font-size: 32px; 
            font-weight: 700; 
            color: #333;
        }
        .section {
            margin-bottom: 40px;
        }
        .section h2 {
            font-size: 24px;
            font-weight: 600;
            margin-bottom: 20px;
            color: #333;
        }
        .event-timeline {
            position: relative;
            padding-left: 30px;
        }
        .event-timeline::before {
            content: '';
            position: absolute;
            left: 10px;
            top: 0;
            bottom: 0;
            width: 2px;
            background: linear-gradient(135deg, #11998e 0%, #38ef7d 100%);
        }
        .event-item {
            position: relative;
            padding: 15px 20px;
            margin-bottom: 15px;
            background: white;
            border-radius: 10px;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
            transition: transform 0.3s ease;
        }
        .event-item:hover {
            transform: translateX(5px);
        }
        .event-item::before {
            content: '';
            position: absolute;
            left: -24px;
            top: 50%;
            transform: translateY(-50%);
            width: 12px;
            height: 12px;
            border-radius: 50%;
            background: linear-gradient(135deg, #11998e 0%, #38ef7d 100%);
        }
        .event-type-alloc {
            border-left: 4px solid #38ef7d;
        }
        .event-type-dealloc {
            border-left: 4px solid #ff6b6b;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Lockfree Memory Tracking</h1>
            <div style="color: #666; font-size: 14px;">TIMESTAMP</div>
        </div>

        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-label">Total Allocations</div>
                <div class="stat-value">TOTAL_ALLOCATIONS</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Total Deallocations</div>
                <div class="stat-value">TOTAL_DEALLOCATIONS</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Active Memory</div>
                <div class="stat-value">ACTIVE_MEMORY</div>
            </div>
        </div>

        <div class="section">
            <h2>Event Timeline</h2>
            <div class="event-timeline" id="events-timeline">
            </div>
        </div>
    </div>

    <script>
        const events = EVENTS_JSON;
        const stats = STATS_JSON;

        const eventsTimeline = document.getElementById('events-timeline');
        events.forEach((event) => {
            const item = document.createElement('div');
            item.className = 'event-item event-type-' + event.event_type.toLowerCase();
            item.innerHTML = `
                <div style="display: flex; justify-content: space-between; margin-bottom: 5px;">
                    <strong>${event.event_type}</strong>
                    <span style="color: #666;">${event.timestamp}</span>
                </div>
                <div style="color: #666;">
                    Thread: ${event.thread_id} | Size: ${formatBytes(event.size)}
                </div>
            `;
            eventsTimeline.appendChild(item);
        });

        function formatBytes(bytes) {
            if (bytes === 0) return '0 B';
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }
    </script>
</body>
</html>"#;

        html_template
            .replace("TIMESTAMP", &format_timestamp(snapshot.timestamp))
            .replace("TOTAL_ALLOCATIONS", &snapshot.stats.allocation_count.to_string())
            .replace("TOTAL_DEALLOCATIONS", &snapshot.stats.deallocation_count.to_string())
            .replace("ACTIVE_MEMORY", &format_bytes(snapshot.stats.current_allocated))
            .replace("EVENTS_JSON", &events_json)
            .replace("STATS_JSON", &stats_json)
    }

    /// Generate Async strategy HTML
    fn render_async(&self, snapshot: &TrackingSnapshot) -> String {
        let tasks_json = serde_json::to_string(&snapshot.tasks).unwrap_or_default();
        let stats_json = serde_json::to_string(&snapshot.stats).unwrap_or_default();

        let html_template = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Async Memory Tracking Dashboard</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { 
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #fa709a 0%, #fee140 100%);
            min-height: 100vh;
            padding: 20px;
        }
        .container { 
            max-width: 1400px; 
            margin: 0 auto; 
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            border-radius: 20px;
            padding: 40px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
        }
        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 40px;
            padding-bottom: 20px;
            border-bottom: 2px solid #e0e0e0;
        }
        .header h1 {
            font-size: 36px;
            font-weight: 700;
            background: linear-gradient(135deg, #fa709a 0%, #fee140 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }
        .stats-grid { 
            display: grid; 
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); 
            gap: 20px; 
            margin-bottom: 40px; 
        }
        .stat-card {
            background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
            padding: 25px;
            border-radius: 15px;
            box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }
        .stat-card:hover {
            transform: translateY(-5px);
            box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
        }
        .stat-label { 
            font-size: 14px; 
            color: #666; 
            margin-bottom: 8px;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        .stat-value { 
            font-size: 32px; 
            font-weight: 700; 
            color: #333;
        }
        .stat-value.highlight {
            color: #fa709a;
        }
        .section {
            margin-bottom: 40px;
        }
        .section h2 {
            font-size: 24px;
            font-weight: 600;
            margin-bottom: 20px;
            color: #333;
        }
        .task-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
        }
        .task-card {
            background: white;
            padding: 25px;
            border-radius: 15px;
            box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
            transition: transform 0.3s ease;
        }
        .task-card:hover {
            transform: translateY(-5px);
        }
        .task-card.running {
            border-top: 4px solid #fa709a;
        }
        .task-card.completed {
            border-top: 4px solid #38ef7d;
        }
        .task-card.leaking {
            border-top: 4px solid #ff6b6b;
        }
        .task-name {
            font-size: 20px;
            font-weight: 600;
            margin-bottom: 15px;
            color: #333;
        }
        .task-stats {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 10px;
        }
        .task-stat {
            display: flex;
            flex-direction: column;
        }
        .task-stat-value {
            font-size: 18px;
            font-weight: 600;
            color: #667eea;
        }
        .task-stat-label {
            font-size: 12px;
            color: #666;
            margin-top: 5px;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Async Memory Tracking</h1>
            <div style="color: #666; font-size: 14px;">TIMESTAMP</div>
        </div>

        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-label">Total Tasks</div>
                <div class="stat-value">TOTAL_TASKS</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Leaking Tasks</div>
                <div class="stat-value highlight">LEAKING_TASKS</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Peak Memory</div>
                <div class="stat-value">PEAK_MEMORY</div>
            </div>
        </div>

        <div class="section">
            <h2>Task Memory Usage</h2>
            <div class="task-grid" id="tasks-grid">
            </div>
        </div>
    </div>

    <script>
        const tasks = TASKS_JSON;
        const stats = STATS_JSON;

        const tasksGrid = document.getElementById('tasks-grid');
        tasks.forEach((task) => {
            const card = document.createElement('div');
            card.className = 'task-card ' + task.status.toLowerCase();
            if (task.has_leak && task.has_leak()) {
                card.classList.add('leaking');
            }
            card.innerHTML = `
                <div class="task-name">${task.task_name}</div>
                <div class="task-stats">
                    <div class="task-stat">
                        <div class="task-stat-value">${formatBytes(task.memory_usage)}</div>
                        <div class="task-stat-label">Memory Usage</div>
                    </div>
                    <div class="task-stat">
                        <div class="task-stat-value">${task.allocation_count}</div>
                        <div class="task-stat-label">Allocations</div>
                    </div>
                    <div class="task-stat">
                        <div class="task-stat-value">${task.status}</div>
                        <div class="task-stat-label">Status</div>
                    </div>
                    <div class="task-stat">
                        <div class="task-stat-value">${(task.memory_efficiency() * 100).toFixed(1)}%</div>
                        <div class="task-stat-label">Efficiency</div>
                    </div>
                </div>
            `;
            tasksGrid.appendChild(card);
        });

        function formatBytes(bytes) {
            if (bytes === 0) return '0 B';
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }
    </script>
</body>
</html>"#;

        html_template
            .replace("TIMESTAMP", &format_timestamp(snapshot.timestamp))
            .replace("TOTAL_TASKS", &snapshot.tasks.len().to_string())
            .replace("LEAKING_TASKS", &snapshot.tasks.iter().filter(|t| t.has_leak()).count().to_string())
            .replace("PEAK_MEMORY", &format_bytes(snapshot.stats.peak_memory))
            .replace("TASKS_JSON", &tasks_json)
            .replace("STATS_JSON", &stats_json)
    }

    /// Generate Unified strategy HTML
    fn render_unified(&self, snapshot: &TrackingSnapshot) -> String {
        let allocations_json = serde_json::to_string(&snapshot.allocations).unwrap_or_default();
        let events_json = serde_json::to_string(&snapshot.events).unwrap_or_default();
        let tasks_json = serde_json::to_string(&snapshot.tasks).unwrap_or_default();
        let stats_json = serde_json::to_string(&snapshot.stats).unwrap_or_default();

        let html_template = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Unified Memory Tracking Dashboard</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { 
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }
        .container { 
            max-width: 1400px; 
            margin: 0 auto; 
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            border-radius: 20px;
            padding: 40px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
        }
        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 40px;
            padding-bottom: 20px;
            border-bottom: 2px solid #e0e0e0;
        }
        .header h1 {
            font-size: 36px;
            font-weight: 700;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }
        .stats-grid { 
            display: grid; 
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); 
            gap: 20px; 
            margin-bottom: 40px; 
        }
        .stat-card {
            background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
            padding: 25px;
            border-radius: 15px;
            box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }
        .stat-card:hover {
            transform: translateY(-5px);
            box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
        }
        .stat-label { 
            font-size: 14px; 
            color: #666; 
            margin-bottom: 8px;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        .stat-value { 
            font-size: 32px; 
            font-weight: 700; 
            color: #333;
        }
        .stat-value.highlight {
            color: #667eea;
        }
        .section {
            margin-bottom: 40px;
        }
        .section h2 {
            font-size: 24px;
            font-weight: 600;
            margin-bottom: 20px;
            color: #333;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Unified Memory Tracking</h1>
            <div style="color: #666; font-size: 14px;">TIMESTAMP</div>
        </div>

        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-label">Total Allocations</div>
                <div class="stat-value">TOTAL_ALLOCATIONS</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Peak Memory</div>
                <div class="stat-value">PEAK_MEMORY</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Active Memory</div>
                <div class="stat-value">ACTIVE_MEMORY</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Leaked Allocations</div>
                <div class="stat-value highlight">LEAKED_ALLOCATIONS</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Total Tasks</div>
                <div class="stat-value">TOTAL_TASKS</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Active Allocations</div>
                <div class="stat-value">ACTIVE_ALLOCATIONS</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Total Events</div>
                <div class="stat-value">TOTAL_EVENTS</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Total Tasks</div>
                <div class="stat-value">TOTAL_TASKS_2</div>
            </div>
        </div>

        <div class="section">
            <h2>Data Summary</h2>
            <p>Tracking snapshot with allocations, events, and tasks</p>
        </div>
    </div>

    <script>
        const allocations = ALLOCATIONS_JSON;
        const events = EVENTS_JSON;
        const tasks = TASKS_JSON;
        const stats = STATS_JSON;

        function formatBytes(bytes) {
            if (bytes === 0) return '0 B';
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }
    </script>
</body>
</html>"#;

        html_template
            .replace("TIMESTAMP", &format_timestamp(snapshot.timestamp))
            .replace("TOTAL_ALLOCATIONS", &snapshot.stats.allocation_count.to_string())
            .replace("PEAK_MEMORY", &format_bytes(snapshot.stats.peak_memory))
            .replace("ACTIVE_MEMORY", &format_bytes(snapshot.stats.current_allocated))
            .replace("LEAKED_ALLOCATIONS", &snapshot.allocations.iter().filter(|a| a.is_active).count().to_string())
            .replace("TOTAL_TASKS", &snapshot.tasks.len().to_string())
            .replace("ACTIVE_ALLOCATIONS", &snapshot.allocations.len().to_string())
            .replace("TOTAL_EVENTS", &snapshot.events.len().to_string())
            .replace("TOTAL_TASKS_2", &snapshot.tasks.len().to_string())
            .replace("ALLOCATIONS_JSON", &allocations_json)
            .replace("EVENTS_JSON", &events_json)
            .replace("TASKS_JSON", &tasks_json)
            .replace("STATS_JSON", &stats_json)
    }
}

impl Renderer for HtmlRenderer {
    fn format(&self) -> crate::data::ExportFormat {
        crate::data::ExportFormat::Html
    }

    fn render(&self, snapshot: &TrackingSnapshot) -> RenderResult<String> {
        let html = match snapshot.strategy {
            TrackingStrategy::Core => self.render_core(snapshot),
            TrackingStrategy::Lockfree => self.render_lockfree(snapshot),
            TrackingStrategy::Async => self.render_async(snapshot),
            TrackingStrategy::Unified => self.render_unified(snapshot),
        };
        Ok(html)
    }
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Format timestamp to human-readable string
fn format_timestamp(timestamp: u64) -> String {
    let secs = timestamp / 1_000_000;
    let millis = (timestamp % 1_000_000) / 1_000;
    let micros = timestamp % 1_000;
    format!("{}.{:03}.{:03}s", secs, millis, micros)
}

/// Format bytes to human-readable string
fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{TrackingStrategy, AllocationRecord, TrackingStats};

    #[test]
    fn test_html_renderer_creation() {
        let renderer = HtmlRenderer::new();
        assert_eq!(renderer.format(), crate::data::ExportFormat::Html);
    }

    #[test]
    fn test_html_render_core() {
        let renderer = HtmlRenderer::new();
        let snapshot = TrackingSnapshot {
            strategy: TrackingStrategy::Core,
            allocations: vec![],
            events: vec![],
            tasks: vec![],
            stats: TrackingStats::default(),
            timestamp: 0,
        };

        let result = renderer.render(&snapshot);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("Core Memory Tracking"));
        assert!(html.contains("<html>"));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp(0), "0.0.0s");
        assert_eq!(format_timestamp(1_000_000), "1.0.0s");
        assert_eq!(format_timestamp(1_500_500), "1.500.500s");
    }
}