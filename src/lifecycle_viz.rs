//! Independent lifecycle visualization module
//! Creates simple, intuitive SVG showing variable birth, life, and death

use crate::tracker::MemoryTracker;
use crate::types::{AllocationInfo, MemoryStats, TrackingResult, TrackingError};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use svg::node::element::{Circle, Rectangle, Text as SvgText, Line};
use svg::Document;

/// Export comprehensive lifecycle visualization to SVG format
/// This creates a detailed lifecycle visualization showing variable birth, life, death, and hierarchy
pub fn export_comprehensive_lifecycle<P: AsRef<Path>>(tracker: &MemoryTracker, path: P) -> TrackingResult<()> {
    let path = path.as_ref();
    tracing::info!("Exporting comprehensive lifecycle visualization to: {}", path.display());

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Gather data
    let active_allocations = tracker.get_active_allocations()?;
    let stats = tracker.get_stats()?;
    
    // Create comprehensive lifecycle SVG document
    let document = create_comprehensive_lifecycle_svg(&active_allocations, &stats)?;
    
    // Write to file
    let mut file = File::create(path)?;
    svg::write(&mut file, &document)
        .map_err(|e| TrackingError::SerializationError(format!("Failed to write SVG: {}", e)))?;

    tracing::info!("Successfully exported comprehensive lifecycle visualization to SVG");
    Ok(())
}

/// Export simple lifecycle visualization to SVG format
pub fn export_simple_lifecycle<P: AsRef<Path>>(tracker: &MemoryTracker, path: P) -> TrackingResult<()> {
    let path = path.as_ref();
    tracing::info!("Exporting simple lifecycle visualization to: {}", path.display());

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Gather data
    let active_allocations = tracker.get_active_allocations()?;
    let stats = tracker.get_stats()?;
    
    // Create simple SVG document
    let document = create_simple_lifecycle_svg(&active_allocations, &stats)?;
    
    // Write to file
    let mut file = File::create(path)?;
    svg::write(&mut file, &document)
        .map_err(|e| TrackingError::SerializationError(format!("Failed to write SVG: {}", e)))?;

    tracing::info!("Successfully exported simple lifecycle visualization to SVG");
    Ok(())
}

/// Create comprehensive lifecycle visualization showing detailed variable lifecycles
fn create_comprehensive_lifecycle_svg(
    allocations: &[AllocationInfo],
    stats: &MemoryStats,
) -> TrackingResult<Document> {
    let width = 1400;
    let height = 1000;
    
    let mut document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", width)
        .set("height", height)
        .set("style", "background: white; font-family: 'Segoe UI', Arial, sans-serif;");

    // Title
    let title = SvgText::new("Variable Lifecycle Visualization - Complete Timeline")
        .set("x", width / 2)
        .set("y", 30)
        .set("text-anchor", "middle")
        .set("font-size", 24)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");
    document = document.add(title);

    // Subtitle
    let subtitle = SvgText::new("Shows creation, lifetime, and destruction timeline of all variables")
        .set("x", width / 2)
        .set("y", 55)
        .set("text-anchor", "middle")
        .set("font-size", 14)
        .set("fill", "#7f8c8d");
    document = document.add(subtitle);

    // Filter tracked variables
    let mut tracked_vars: Vec<_> = allocations
        .iter()
        .filter(|a| a.var_name.is_some())
        .collect();
    
    if tracked_vars.is_empty() {
        let no_data = SvgText::new("No tracked variables found - Use track_var! macro to track variables")
            .set("x", width / 2)
            .set("y", height / 2)
            .set("text-anchor", "middle")
            .set("font-size", 16)
            .set("fill", "#e74c3c");
        document = document.add(no_data);
        return Ok(document);
    }

    // Sort by allocation time
    tracked_vars.sort_by_key(|a| a.timestamp_alloc);

    // Timeline setup
    let timeline_start_x = 100;
    let timeline_end_x = width - 100;
    let timeline_y = 120;
    let timeline_width = timeline_end_x - timeline_start_x;
    
    // Draw main timeline
    let timeline = Line::new()
        .set("x1", timeline_start_x)
        .set("y1", timeline_y)
        .set("x2", timeline_end_x)
        .set("y2", timeline_y)
        .set("stroke", "#34495e")
        .set("stroke-width", 3);
    document = document.add(timeline);

    // Timeline labels
    let start_label = SvgText::new("Program Start")
        .set("x", timeline_start_x)
        .set("y", timeline_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 12)
        .set("font-weight", "bold")
        .set("fill", "#27ae60");
    document = document.add(start_label);

    let end_label = SvgText::new("Current Time")
        .set("x", timeline_end_x)
        .set("y", timeline_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 12)
        .set("font-weight", "bold")
        .set("fill", "#e74c3c");
    document = document.add(end_label);

    // Calculate time range
    let min_time = tracked_vars.first().map(|a| a.timestamp_alloc).unwrap_or(0);
    let max_time = tracked_vars.last().map(|a| a.timestamp_alloc).unwrap_or(min_time + 1);
    let time_range = (max_time - min_time).max(1);

    // Draw variables with detailed lifecycle information
    let var_start_y = 180;
    let var_spacing = 80;
    
    for (i, allocation) in tracked_vars.iter().take(10).enumerate() {
        let y = var_start_y + i * var_spacing;
        
        // Calculate position on timeline
        let time_pos = if time_range > 0 {
            ((allocation.timestamp_alloc - min_time) as f64 / time_range as f64 * timeline_width as f64) as i32
        } else {
            timeline_width / 2
        };
        let x = timeline_start_x + time_pos;

        let var_name = allocation.var_name.as_ref().unwrap();
        let type_name = allocation.type_name.as_deref().unwrap_or("Unknown");
        let simple_type = get_simple_type(type_name);

        // Variable lifecycle box
        let box_width = 300;
        let box_height = 60;
        let box_x = 50;

        // Variable info box
        let var_box = Rectangle::new()
            .set("x", box_x)
            .set("y", y - 25)
            .set("width", box_width)
            .set("height", box_height)
            .set("fill", "#ecf0f1")
            .set("stroke", "#3498db")
            .set("stroke-width", 2)
            .set("rx", 8);
        document = document.add(var_box);

        // Variable name and type
        let var_info = format!("{}({})", var_name, simple_type);
        let var_text = SvgText::new(var_info)
            .set("x", box_x + 10)
            .set("y", y - 5)
            .set("font-size", 14)
            .set("font-weight", "bold")
            .set("fill", "#2c3e50");
        document = document.add(var_text);

        // Memory size
        let size_text = format!("Memory: {}", format_size(allocation.size));
        let size_label = SvgText::new(size_text)
            .set("x", box_x + 10)
            .set("y", y + 15)
            .set("font-size", 12)
            .set("fill", "#7f8c8d");
        document = document.add(size_label);

        // Status
        let status_text = SvgText::new("Status: ALIVE")
            .set("x", box_x + 10)
            .set("y", y + 30)
            .set("font-size", 11)
            .set("font-weight", "bold")
            .set("fill", "#27ae60");
        document = document.add(status_text);

        // Connection line from box to timeline
        let connection = Line::new()
            .set("x1", box_x + box_width)
            .set("y1", y)
            .set("x2", x - 10)
            .set("y2", timeline_y)
            .set("stroke", "#bdc3c7")
            .set("stroke-width", 2)
            .set("stroke-dasharray", "5,5");
        document = document.add(connection);

        // Birth marker on timeline
        let birth_circle = Circle::new()
            .set("cx", x)
            .set("cy", timeline_y)
            .set("r", 8)
            .set("fill", "#27ae60")
            .set("stroke", "white")
            .set("stroke-width", 3);
        document = document.add(birth_circle);

        // Birth label
        let birth_label = SvgText::new("BORN")
            .set("x", x)
            .set("y", timeline_y + 25)
            .set("text-anchor", "middle")
            .set("font-size", 10)
            .set("font-weight", "bold")
            .set("fill", "#27ae60");
        document = document.add(birth_label);

        // Lifetime line (from birth to current)
        let lifetime_line = Line::new()
            .set("x1", x)
            .set("y1", timeline_y)
            .set("x2", timeline_end_x - 20)
            .set("y2", timeline_y)
            .set("stroke", "#3498db")
            .set("stroke-width", 4)
            .set("opacity", "0.7");
        document = document.add(lifetime_line);

        // Current status marker
        let current_marker = Circle::new()
            .set("cx", timeline_end_x - 20)
            .set("cy", timeline_y)
            .set("r", 6)
            .set("fill", "#3498db")
            .set("stroke", "white")
            .set("stroke-width", 2);
        document = document.add(current_marker);
    }

    // Legend
    let legend_y = height - 150;
    
    // Legend background
    let legend_bg = Rectangle::new()
        .set("x", 50)
        .set("y", legend_y - 20)
        .set("width", width - 100)
        .set("height", 120)
        .set("fill", "#f8f9fa")
        .set("stroke", "#dee2e6")
        .set("stroke-width", 1)
        .set("rx", 8);
    document = document.add(legend_bg);

    let legend_title = SvgText::new("Legend:")
        .set("x", 70)
        .set("y", legend_y)
        .set("font-size", 16)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");
    document = document.add(legend_title);

    // Legend items
    let legend_items = [
        ("Green Circle = Variable Birth (allocation time)", 70, legend_y + 25),
        ("Blue Line = Variable Lifetime (from birth to current)", 70, legend_y + 45),
        ("Blue Circle = Current Status (still alive)", 70, legend_y + 65),
        ("Gray Box = Variable Details (name, type, memory)", 70, legend_y + 85),
    ];

    for (text, x, y) in legend_items {
        let item_text = SvgText::new(text)
            .set("x", x)
            .set("y", y)
            .set("font-size", 12)
            .set("fill", "#495057");
        document = document.add(item_text);
    }

    // Summary
    let summary_text = format!(
        "Total: {} variables using {} memory",
        tracked_vars.len(),
        format_size(stats.active_memory)
    );
    let summary_label = SvgText::new(summary_text)
        .set("x", width / 2)
        .set("y", height - 20)
        .set("text-anchor", "middle")
        .set("font-size", 14)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");
    document = document.add(summary_label);

    Ok(document)
}

/// Create extremely simple lifecycle visualization - like a family tree
fn create_simple_lifecycle_svg(
    allocations: &[AllocationInfo],
    stats: &MemoryStats,
) -> TrackingResult<Document> {
    let width = 1000;
    let height = 700;
    
    let mut document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", width)
        .set("height", height)
        .set("style", "background: white; font-family: Arial, sans-serif;");

    // Super simple title
    let title = SvgText::new("Variable Family Tree - Who's Born, Who's Alive, Who's Gone")
        .set("x", width / 2)
        .set("y", 30)
        .set("text-anchor", "middle")
        .set("font-size", 22)
        .set("font-weight", "bold")
        .set("fill", "black");
    document = document.add(title);

    // Simple explanation
    let explanation = SvgText::new("Green Circle = Just Born | Blue Box = Living | Red X = Dead")
        .set("x", width / 2)
        .set("y", 55)
        .set("text-anchor", "middle")
        .set("font-size", 14)
        .set("fill", "gray");
    document = document.add(explanation);

    // Group variables
    let mut variables: HashMap<String, Vec<&AllocationInfo>> = HashMap::new();
    for alloc in allocations {
        if let Some(var_name) = &alloc.var_name {
            variables.entry(var_name.clone()).or_default().push(alloc);
        }
    }

    if variables.is_empty() {
        let no_data = SvgText::new("No variables to show - your program is very quiet!")
            .set("x", width / 2)
            .set("y", height / 2)
            .set("text-anchor", "middle")
            .set("font-size", 16)
            .set("fill", "gray");
        document = document.add(no_data);
        return Ok(document);
    }

    // Draw variables in a simple grid
    let start_x = 50;
    let start_y = 100;
    let box_width = 180;
    let box_height = 60;
    let spacing_x = 200;
    let spacing_y = 80;
    let cols = 4;

    for (i, (var_name, var_allocs)) in variables.iter().take(16).enumerate() {
        let col = i % cols;
        let row = i / cols;
        let x = start_x + (col * spacing_x);
        let y = start_y + (row * spacing_y);

        if let Some(first_alloc) = var_allocs.first() {
            let type_name = first_alloc.type_name.as_deref().unwrap_or("Unknown");
            let simple_type = get_simple_type(type_name);
            let total_size = var_allocs.iter().map(|a| a.size).sum::<usize>();

            // Variable box (always blue for "alive")
            let var_box = Rectangle::new()
                .set("x", x)
                .set("y", y)
                .set("width", box_width)
                .set("height", box_height)
                .set("fill", "#e3f2fd")
                .set("stroke", "#1976d2")
                .set("stroke-width", 2)
                .set("rx", 5);
            document = document.add(var_box);

            // Variable name (big)
            let name_text = SvgText::new(var_name)
                .set("x", x + box_width / 2)
                .set("y", y + 20)
                .set("text-anchor", "middle")
                .set("font-size", 14)
                .set("font-weight", "bold")
                .set("fill", "black");
            document = document.add(name_text);

            // Type and size
            let info_text = format!("{} ({})", simple_type, format_size(total_size));
            let info_label = SvgText::new(info_text)
                .set("x", x + box_width / 2)
                .set("y", y + 40)
                .set("text-anchor", "middle")
                .set("font-size", 11)
                .set("fill", "gray");
            document = document.add(info_label);

            // Status
            let status_text = SvgText::new("ALIVE")
                .set("x", x + box_width / 2)
                .set("y", y + 55)
                .set("text-anchor", "middle")
                .set("font-size", 10)
                .set("font-weight", "bold")
                .set("fill", "#1976d2");
            document = document.add(status_text);

            // Birth indicator (green circle above)
            let birth_circle = Circle::new()
                .set("cx", x + box_width / 2)
                .set("cy", y - 15)
                .set("r", 8)
                .set("fill", "#4caf50")
                .set("stroke", "white")
                .set("stroke-width", 2);
            document = document.add(birth_circle);

            // "BORN" label
            let born_label = SvgText::new("BORN")
                .set("x", x + box_width / 2)
                .set("y", y - 25)
                .set("text-anchor", "middle")
                .set("font-size", 9)
                .set("font-weight", "bold")
                .set("fill", "#4caf50");
            document = document.add(born_label);

            // Connection line from birth to variable
            let connection = Line::new()
                .set("x1", x + box_width / 2)
                .set("y1", y - 7)
                .set("x2", x + box_width / 2)
                .set("y2", y)
                .set("stroke", "#4caf50")
                .set("stroke-width", 2);
            document = document.add(connection);
        }
    }

    // Simple legend at bottom
    let legend_y = height - 100;
    
    // Legend background
    let legend_bg = Rectangle::new()
        .set("x", 50)
        .set("y", legend_y - 20)
        .set("width", width - 100)
        .set("height", 80)
        .set("fill", "#f5f5f5")
        .set("stroke", "#ddd")
        .set("stroke-width", 1)
        .set("rx", 5);
    document = document.add(legend_bg);

    let legend_title = SvgText::new("What you're seeing:")
        .set("x", 70)
        .set("y", legend_y)
        .set("font-size", 14)
        .set("font-weight", "bold")
        .set("fill", "black");
    document = document.add(legend_title);

    // Legend items
    let items = [
        ("Green circle = Variable was born (allocated memory)", 70, legend_y + 20),
        ("Blue box = Variable is currently alive and using memory", 70, legend_y + 35),
        ("All variables shown are STILL ALIVE in your program", 70, legend_y + 50),
    ];

    for (text, x, y) in items {
        let item_text = SvgText::new(text)
            .set("x", x)
            .set("y", y)
            .set("font-size", 12)
            .set("fill", "#333");
        document = document.add(item_text);
    }

    // Summary
    let summary_text = format!(
        "Total: {} variables using {} of memory",
        variables.len(),
        format_size(stats.active_memory)
    );
    let summary_label = SvgText::new(summary_text)
        .set("x", width / 2)
        .set("y", height - 20)
        .set("text-anchor", "middle")
        .set("font-size", 16)
        .set("font-weight", "bold")
        .set("fill", "#1976d2");
    document = document.add(summary_label);

    Ok(document)
}

/// Get simple type name
fn get_simple_type(type_name: &str) -> String {
    if type_name.contains("String") {
        "String".to_string()
    } else if type_name.contains("Vec") {
        "Vec".to_string()
    } else if type_name.contains("Box") {
        "Box".to_string()
    } else if type_name.contains("HashMap") {
        "HashMap".to_string()
    } else {
        type_name.split("::").last().unwrap_or("Unknown").to_string()
    }
}

/// Format memory size
fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} bytes", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}