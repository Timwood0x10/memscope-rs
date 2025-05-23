//! Memory tracking data export functionality

use serde::Serialize;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use svg::node::element::{Rectangle, Text as SvgText};
use svg::Document;

use crate::tracker::MemoryTracker;
use crate::types::AllocationInfo;

/// A snapshot of memory usage at a point in time
#[derive(Serialize, Debug, Clone)]
pub struct MemorySnapshot {
    /// ISO 8601 timestamp of when the snapshot was taken
    pub timestamp: String,
    
    /// Total number of active allocations
    pub total_allocations: usize,
    
    /// Total number of bytes allocated
    pub total_allocated: usize,
    
    /// Detailed information about active allocations
    pub active_allocations: Vec<AllocationInfo>,
}

/// Export memory usage data to a JSON file
///
/// # Arguments
/// * `tracker` - Reference to the memory tracker
/// * `path` - Path where the JSON file should be saved
///
/// # Errors
/// Returns `io::Error` if there's a problem creating or writing to the file
pub fn export_to_json<P: AsRef<Path>>(tracker: &MemoryTracker, path: P) -> io::Result<()> {
    let snapshot = create_snapshot(tracker);
    let json = serde_json::to_string_pretty(&snapshot)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())
}

/// Export memory usage data to an SVG visualization
///
/// # Arguments
/// * `tracker` - Reference to the memory tracker
/// * `path` - Path where the SVG file should be saved
///
/// # Errors
/// Returns `io::Error` if there's a problem creating or writing to the file
pub fn export_to_svg<P: AsRef<Path>>(tracker: &MemoryTracker, path: P) -> io::Result<()> {
    let snapshot = create_snapshot(tracker);
    let doc = create_svg_document(&snapshot);
    
    svg::save(path, &doc)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

/// Create a memory snapshot from the current state of the tracker
fn create_snapshot(tracker: &MemoryTracker) -> MemorySnapshot {
    let stats = tracker.get_stats();
    
    MemorySnapshot {
        timestamp: chrono::Local::now().to_rfc3339(),
        total_allocations: stats.total_allocations,
        total_allocated: stats.total_memory,
        active_allocations: tracker.get_active_allocations().into_iter().map(|a| a.into()).collect(),
    }
}

/// Create an SVG document visualizing memory usage
fn create_svg_document(snapshot: &MemorySnapshot) -> Document {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;
    const MARGIN: u32 = 50;
    const BAR_WIDTH: u32 = 40;
    const BAR_SPACING: u32 = 10;
    
    let mut doc = Document::new()
        .set("width", WIDTH)
        .set("height", HEIGHT)
        .set("viewBox", (0, 0, WIDTH, HEIGHT));
    
    // Add title
    let title = SvgText::new("Memory Usage Snapshot")
        .set("x", WIDTH / 2)
        .set("y", MARGIN / 2)
        .set("text-anchor", "middle")
        .set("font-size", 16);
    
    doc = doc.add(title);
    
    // Add timestamp
    let timestamp = SvgText::new(&snapshot.timestamp)
        .set("x", WIDTH / 2)
        .set("y", MARGIN / 2 + 20)
        .set("text-anchor", "middle")
        .set("font-size", 12);
    
    doc = doc.add(timestamp);
    
    // Add stats
    let stats_text = format!(
        "Total Allocations: {} | Total Memory: {} bytes",
        snapshot.total_allocations, snapshot.total_allocated
    );
    
    let stats = SvgText::new(stats_text)
        .set("x", WIDTH / 2)
        .set("y", MARGIN / 2 + 40)
        .set("text-anchor", "middle")
        .set("font-size", 12);
    
    doc = doc.add(stats);
    
    // Add memory bars for top allocations
    let max_allocations = 10;
    let mut allocations = snapshot.active_allocations.clone();
    allocations.sort_by(|a, b| b.size.cmp(&a.size));
    let top_allocations = allocations.iter().take(max_allocations).collect::<Vec<_>>();
    
    if !top_allocations.is_empty() {
        let max_size = top_allocations[0].size as f64;
        let chart_height = (HEIGHT - MARGIN * 2 - 60) as f64;
        let bar_width = (WIDTH as f64 / (top_allocations.len() as f64 * 1.5)).min(100.0);
        
        for (i, alloc) in top_allocations.iter().enumerate() {
            let bar_height = (alloc.size as f64 / max_size * chart_height) as u32;
            let x = MARGIN as f64 + i as f64 * (bar_width + 10.0);
            let y = (HEIGHT - MARGIN - bar_height) as f64;
            
            let bar = Rectangle::new()
                .set("x", x)
                .set("y", y)
                .set("width", bar_width)
                .set("height", bar_height)
                .set("fill", "steelblue")
                .set("stroke", "black");
            
            doc = doc.add(bar);
            
            // Add label
            let label = format!(
                "{}: {}B",
                alloc.var_name.as_deref().unwrap_or("<unknown>"),
                alloc.size
            );
            
            let label_text = SvgText::new(label)
                 .set("x", x + bar_width / 2.0)
                .set("y", y - 5.0)
                .set("text-anchor", "middle")
                .set("font-size", 10)
                .set("transform", format!("rotate(-45, {}, {})", x + bar_width / 2.0, y - 5.0));
            
            doc = doc.add(label_text);
        }
    }
    
    doc
}