//! Export functionality for memory tracking data.

use crate::tracker::MemoryTracker;
use crate::types::{AllocationInfo, TrackingError, TrackingResult};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use svg::node::element::{Rectangle, Text as SvgText};
use svg::Document;

/// Memory snapshot for JSON export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// Timestamp when the snapshot was taken
    pub timestamp: String,
    /// Total number of active allocations
    pub total_allocations: usize,
    /// Total bytes allocated
    pub total_allocated: usize,
    /// Active allocations at the time of snapshot
    pub active_allocations: Vec<AllocationInfo>,
    /// Memory usage by type
    pub memory_by_type: Vec<crate::types::TypeMemoryUsage>,
    /// Memory usage statistics
    pub stats: crate::types::MemoryStats,
}

/// Export memory data to JSON format.
pub fn export_to_json<P: AsRef<Path>>(tracker: &MemoryTracker, path: P) -> TrackingResult<()> {
    let path = path.as_ref();
    tracing::info!("Exporting memory snapshot to: {}", path.display());

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Gather data
    let active_allocations = tracker.get_active_allocations()?;
    let memory_by_type = tracker.get_memory_by_type()?;
    let stats = tracker.get_stats()?;

    let snapshot = MemorySnapshot {
        timestamp: chrono::Utc::now().to_rfc3339(),
        total_allocations: active_allocations.len(),
        total_allocated: stats.active_memory,
        active_allocations,
        memory_by_type,
        stats,
    };

    // Write to file
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, &snapshot)
        .map_err(|e| TrackingError::SerializationError(e.to_string()))?;

    tracing::info!("Successfully exported memory snapshot to JSON");
    Ok(())
}

/// Export memory visualization to SVG format.
pub fn export_to_svg<P: AsRef<Path>>(tracker: &MemoryTracker, path: P) -> TrackingResult<()> {
    // Use the enhanced SVG export
    crate::export_enhanced::export_enhanced_svg(tracker, path)
}

/// Add memory usage by type chart to the SVG document.
#[allow(dead_code)]
fn add_type_usage_chart(
    mut document: Document,
    memory_by_type: &[crate::types::TypeMemoryUsage],
) -> TrackingResult<Document> {
    let chart_x = 50;
    let chart_y = 60;
    let chart_width = 700;
    let chart_height = 200;

    // Add chart title
    let chart_title = SvgText::new("Memory Usage by Type")
        .set("x", chart_x + chart_width / 2)
        .set("y", chart_y - 10)
        .set("text-anchor", "middle")
        .set("font-family", "Arial, sans-serif")
        .set("font-size", 14)
        .set("font-weight", "bold");

    document = document.add(chart_title);

    // Find max size for scaling
    let max_size = memory_by_type
        .iter()
        .map(|t| t.total_size)
        .max()
        .unwrap_or(1);

    // Draw bars for each type (limit to top 10)
    let types_to_show = memory_by_type.iter().take(10);
    let bar_height = chart_height / 10.min(memory_by_type.len());

    for (i, type_usage) in types_to_show.enumerate() {
        let y = chart_y + i * bar_height;
        let bar_width =
            (type_usage.total_size as f64 / max_size as f64 * chart_width as f64) as i32;

        // Color based on size
        let color = match i {
            0 => "#ff6b6b", // Red for largest
            1 => "#ffa500", // Orange
            2 => "#ffff00", // Yellow
            _ => "#4ecdc4", // Teal for others
        };

        // Draw bar
        let bar = Rectangle::new()
            .set("x", chart_x)
            .set("y", y)
            .set("width", bar_width)
            .set("height", bar_height - 5)
            .set("fill", color)
            .set("stroke", "#333")
            .set("stroke-width", 1);

        document = document.add(bar);

        // Add type name
        let type_name = if type_usage.type_name.len() > 30 {
            format!("{}...", &type_usage.type_name[..27])
        } else {
            type_usage.type_name.clone()
        };

        let label = SvgText::new(format!("{} ({} bytes)", type_name, type_usage.total_size))
            .set("x", chart_x + 5)
            .set("y", y + bar_height / 2 + 4)
            .set("font-family", "Arial, sans-serif")
            .set("font-size", 10)
            .set("fill", "white");

        document = document.add(label);
    }

    Ok(document)
}

/// Add allocation bars to the SVG document.
#[allow(dead_code)]
fn add_allocation_bars(
    mut document: Document,
    allocations: &[AllocationInfo],
) -> TrackingResult<Document> {
    let chart_x = 50;
    let chart_y = 300;
    let chart_width = 700;
    let chart_height = 250;

    // Add chart title
    let chart_title = SvgText::new("Active Allocations")
        .set("x", chart_x + chart_width / 2)
        .set("y", chart_y - 10)
        .set("text-anchor", "middle")
        .set("font-family", "Arial, sans-serif")
        .set("font-size", 14)
        .set("font-weight", "bold");

    document = document.add(chart_title);

    // Sort allocations by size and take top 20
    let mut sorted_allocs = allocations.to_vec();
    sorted_allocs.sort_by(|a, b| b.size.cmp(&a.size));
    let allocs_to_show = sorted_allocs.iter().take(20);

    let max_size = sorted_allocs.first().map(|a| a.size).unwrap_or(1);
    let bar_height = chart_height / 20.min(allocations.len());

    for (i, allocation) in allocs_to_show.enumerate() {
        let y = chart_y + i * bar_height;
        let bar_width = (allocation.size as f64 / max_size as f64 * chart_width as f64) as i32;

        // Color based on variable name availability
        let color = if allocation.var_name.is_some() {
            "#4ecdc4" // Teal for tracked variables
        } else {
            "#95a5a6" // Gray for untracked
        };

        // Draw bar
        let bar = Rectangle::new()
            .set("x", chart_x)
            .set("y", y)
            .set("width", bar_width)
            .set("height", bar_height - 2)
            .set("fill", color)
            .set("stroke", "#333")
            .set("stroke-width", 1);

        document = document.add(bar);

        // Add label
        let label_text = if let Some(var_name) = &allocation.var_name {
            format!("{} ({} bytes)", var_name, allocation.size)
        } else {
            format!("0x{:x} ({} bytes)", allocation.ptr, allocation.size)
        };

        let label = SvgText::new(label_text)
            .set("x", chart_x + 5)
            .set("y", y + bar_height / 2 + 3)
            .set("font-family", "Arial, sans-serif")
            .set("font-size", 9)
            .set("fill", "white");

        document = document.add(label);
    }

    Ok(document)
}
