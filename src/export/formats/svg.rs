//! SVG export functionality for memory visualization
//! Provides memory analysis and lifecycle timeline SVG exports

use crate::core::tracker::MemoryTracker;
use crate::core::types::{AllocationInfo, MemoryStats, TrackingError, TrackingResult};
use std::path::Path;

/// Export memory analysis visualization showing variable names, types, and usage
pub fn export_memory_analysis<P: AsRef<Path>>(
    tracker: &MemoryTracker,
    path: P,
) -> TrackingResult<()> {
    let path = path.as_ref();
    tracing::info!("Exporting memory analysis to: {}", path.display());

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let active_allocations = tracker.get_all_active_allocations()?;
    let stats = tracker.get_memory_stats()?;

    let svg_content = create_memory_analysis_svg(&active_allocations, &stats)?;

    std::fs::write(path, svg_content)
        .map_err(|e| TrackingError::IoError(e.to_string()))?;

    tracing::info!("Successfully exported memory analysis SVG");
    Ok(())
}

/// Export interactive lifecycle timeline showing variable lifecycles and relationships
pub fn export_lifecycle_timeline<P: AsRef<Path>>(
    tracker: &MemoryTracker,
    path: P,
) -> TrackingResult<()> {
    let path = path.as_ref();
    tracing::info!("Exporting lifecycle timeline to: {}", path.display());

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let active_allocations = tracker.get_all_active_allocations()?;
    let stats = tracker.get_memory_stats()?;

    let svg_content = create_lifecycle_timeline_svg(&active_allocations, &stats)?;

    std::fs::write(path, svg_content)
        .map_err(|e| TrackingError::IoError(e.to_string()))?;

    tracing::info!("Successfully exported lifecycle timeline SVG");
    Ok(())
}

/// Create comprehensive memory analysis SVG
fn create_memory_analysis_svg(
    allocations: &[AllocationInfo],
    stats: &MemoryStats,
) -> TrackingResult<String> {
    let width = 1200;
    let height = 800;
    
    let total_memory = allocations.iter().map(|a| a.size).sum::<usize>();
    let allocation_count = allocations.len();
    
    let svg_content = format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
            <defs>
                <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" style="stop-color:#f8f9fa;stop-opacity:1" />
                    <stop offset="100%" style="stop-color:#e9ecef;stop-opacity:1" />
                </linearGradient>
                <style>
                    .title {{ font: bold 24px Arial; fill: #2c3e50; }}
                    .subtitle {{ font: 16px Arial; fill: #34495e; }}
                    .label {{ font: 12px Arial; fill: #2c3e50; }}
                    .value {{ font: bold 14px Arial; fill: #e74c3c; }}
                </style>
            </defs>
            
            <rect width="100%" height="100%" fill="url(#bg)"/>
            
            <!-- Title -->
            <text x="600" y="40" text-anchor="middle" class="title">Memory Analysis Dashboard</text>
            <text x="600" y="65" text-anchor="middle" class="subtitle">Active Allocations: {} | Total Memory: {} bytes</text>
            
            <!-- Statistics Panel -->
            <rect x="50" y="100" width="300" height="200" fill="white" stroke="rgb(189,195,199)" stroke-width="2" rx="10"/>
            <text x="200" y="130" text-anchor="middle" class="subtitle">Memory Statistics</text>
            
            <text x="70" y="160" class="label">Active Allocations:</text>
            <text x="250" y="160" class="value">{}</text>
            
            <text x="70" y="180" class="label">Total Memory:</text>
            <text x="250" y="180" class="value">{} bytes</text>
            
            <text x="70" y="200" class="label">Peak Memory:</text>
            <text x="250" y="200" class="value">{} bytes</text>
            
            <text x="70" y="220" class="label">Total Allocations:</text>
            <text x="250" y="220" class="value">{}</text>
            
            <!-- Memory Usage Chart -->
            <rect x="400" y="100" width="750" height="400" fill="white" stroke="rgb(189,195,199)" stroke-width="2" rx="10"/>
            <text x="775" y="130" text-anchor="middle" class="subtitle">Memory Usage by Variable</text>
            
            {}
            
            <!-- Legend -->
            <rect x="50" y="550" width="1100" height="200" fill="white" stroke="rgb(189,195,199)" stroke-width="2" rx="10"/>
            <text x="600" y="580" text-anchor="middle" class="subtitle">Variable Details</text>
            
            {}
        </svg>"#,
        width, height,
        allocation_count, total_memory,
        stats.active_allocations,
        stats.active_memory,
        stats.peak_memory,
        stats.total_allocations,
        generate_memory_bars(allocations, 420, 150, 710, 300),
        generate_variable_legend(allocations, 70, 600, 1060, 140)
    );
    
    Ok(svg_content)
}

/// Create lifecycle timeline SVG
fn create_lifecycle_timeline_svg(
    allocations: &[AllocationInfo],
    _stats: &MemoryStats,
) -> TrackingResult<String> {
    let width = 1400;
    let height = 600;
    
    let svg_content = format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
            <defs>
                <linearGradient id="timeline-bg" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" style="stop-color:#3498db;stop-opacity:0.1" />
                    <stop offset="100%" style="stop-color:#e74c3c;stop-opacity:0.1" />
                </linearGradient>
                <style>
                    .title {{ font: bold 20px Arial; fill: #2c3e50; }}
                    .timeline-label {{ font: 12px Arial; fill: #34495e; }}
                    .var-name {{ font: bold 11px Arial; fill: #2c3e50; }}
                </style>
            </defs>
            
            <rect width="100%" height="100%" fill="white"/>
            <rect x="0" y="0" width="100%" height="60" fill="url(#timeline-bg)"/>
            
            <!-- Title -->
            <text x="700" y="35" text-anchor="middle" class="title">Memory Allocation Timeline</text>
            
            <!-- Timeline -->
            {}
            
        </svg>"#,
        width, height,
        generate_timeline_bars(allocations, 50, 80, 1300, 480)
    );
    
    Ok(svg_content)
}

/// Generate memory usage bars for the chart
fn generate_memory_bars(allocations: &[AllocationInfo], x: i32, y: i32, width: i32, height: i32) -> String {
    if allocations.is_empty() {
        return String::new();
    }
    
    let max_size = allocations.iter().map(|a| a.size).max().unwrap_or(1);
    let bar_width = (width / allocations.len().min(20) as i32).max(10);
    let mut bars = String::new();
    
    for (i, allocation) in allocations.iter().take(20).enumerate() {
        let bar_height = (allocation.size as f64 / max_size as f64 * height as f64) as i32;
        let bar_x = x + (i as i32 * bar_width);
        let bar_y = y + height - bar_height;
        
        let color = match allocation.size {
            s if s > 10000 => "rgb(231,76,60)",
            s if s > 1000 => "rgb(243,156,18)", 
            _ => "rgb(39,174,96)"
        };
        
        bars.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" opacity="0.8"/>
               <text x="{}" y="{}" class="label" transform="rotate(-45 {} {})">{}</text>"#,
            bar_x, bar_y, bar_width - 2, bar_height, color,
            bar_x + bar_width/2, y + height + 15, bar_x + bar_width/2, y + height + 15,
            allocation.var_name.as_deref().unwrap_or("unknown")
        ));
    }
    
    bars
}

/// Generate variable legend
fn generate_variable_legend(allocations: &[AllocationInfo], x: i32, y: i32, width: i32, height: i32) -> String {
    let mut legend = String::new();
    let cols = 3;
    let rows = (allocations.len().min(15) + cols - 1) / cols;
    let col_width = width / cols as i32;
    let row_height = height / rows.max(1) as i32;
    
    for (i, allocation) in allocations.iter().take(15).enumerate() {
        let col = i % cols;
        let row = i / cols;
        let item_x = x + (col as i32 * col_width);
        let item_y = y + (row as i32 * row_height);
        
        legend.push_str(&format!(
            r#"<text x="{}" y="{}" class="label">{}: {} bytes ({})</text>"#,
            item_x, item_y + 20,
            allocation.var_name.as_deref().unwrap_or("unknown"),
            allocation.size,
            allocation.type_name.as_deref().unwrap_or("unknown")
        ));
    }
    
    legend
}

/// Generate timeline bars showing allocation lifetimes
fn generate_timeline_bars(allocations: &[AllocationInfo], x: i32, y: i32, width: i32, height: i32) -> String {
    if allocations.is_empty() {
        return String::new();
    }
    
    let min_time = allocations.iter().map(|a| a.timestamp_alloc).min().unwrap_or(0);
    let max_time = allocations.iter()
        .map(|a| a.timestamp_dealloc.unwrap_or(a.timestamp_alloc + 1000000))
        .max().unwrap_or(min_time + 1000000);
    
    let time_range = (max_time - min_time).max(1);
    let bar_height = (height / allocations.len().min(20) as i32).max(15);
    let mut timeline = String::new();
    
    for (i, allocation) in allocations.iter().take(20).enumerate() {
        let start_x = x + ((allocation.timestamp_alloc - min_time) as f64 / time_range as f64 * width as f64) as i32;
        let end_time = allocation.timestamp_dealloc.unwrap_or(max_time);
        let end_x = x + ((end_time - min_time) as f64 / time_range as f64 * width as f64) as i32;
        let bar_y = y + (i as i32 * bar_height);
        let bar_width = (end_x - start_x).max(5);
        
        let color = if allocation.timestamp_dealloc.is_some() { "rgb(52,152,219)" } else { "rgb(231,76,60)" };
        
        timeline.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" opacity="0.7"/>
               <text x="{}" y="{}" class="var-name">{}</text>"#,
            start_x, bar_y, bar_width, bar_height - 2, color,
            start_x + 5, bar_y + bar_height/2 + 4,
            allocation.var_name.as_deref().unwrap_or("unknown")
        ));
    }
    
    timeline
}