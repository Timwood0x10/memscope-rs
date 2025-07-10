//! Unified visualization module for memscope-rs
//! Provides memory analysis and lifecycle timeline SVG exports

use crate::tracker::MemoryTracker;
use crate::types::{AllocationInfo, MemoryStats, TrackingError, TrackingResult};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use svg::node::element::{Circle, Line, Rectangle, Style, Text as SvgText};
use svg::Document;

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

    let active_allocations = tracker.get_active_allocations()?;
    let stats = tracker.get_stats()?;

    let document = create_memory_analysis_svg(&active_allocations, &stats)?;

    let mut file = File::create(path)?;
    svg::write(&mut file, &document)
        .map_err(|e| TrackingError::SerializationError(format!("Failed to write SVG: {e}")))?;

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

    let active_allocations = tracker.get_active_allocations()?;
    let stats = tracker.get_stats()?;

    let document = create_lifecycle_timeline_svg(&active_allocations, &stats)?;

    let mut file = File::create(path)?;
    svg::write(&mut file, &document)
        .map_err(|e| TrackingError::SerializationError(format!("Failed to write SVG: {e}")))?;

    tracing::info!("Successfully exported lifecycle timeline SVG");
    Ok(())
}

/// Create memory analysis SVG with enhanced variable information
fn create_memory_analysis_svg(
    allocations: &[AllocationInfo],
    stats: &MemoryStats,
) -> TrackingResult<Document> {
    let width = 1400;
    let height = 1000;

    let mut document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", width)
        .set("height", height)
        .set("style", "background: linear-gradient(135deg, #2C3E50 0%, #34495E 100%); font-family: 'Segoe UI', Arial, sans-serif;");

    // Title
    let title = SvgText::new("Memory Analysis - Variable Usage Report")
        .set("x", width / 2)
        .set("y", 40)
        .set("text-anchor", "middle")
        .set("font-size", 28)
        .set("font-weight", "bold")
        .set("fill", "#ECF0F1");
    document = document.add(title);

    // Filter tracked variables
    let tracked_vars: Vec<_> = allocations
        .iter()
        .filter(|a| a.var_name.is_some())
        .collect();

    if tracked_vars.is_empty() {
        let no_data =
            SvgText::new("No tracked variables found - Use track_var! macro to track variables")
                .set("x", width / 2)
                .set("y", height / 2)
                .set("text-anchor", "middle")
                .set("font-size", 16)
                .set("fill", "#E74C3C");
        document = document.add(no_data);
        return Ok(document);
    }

    // Add call stack analysis
    document = add_call_stack_analysis(document, allocations, 100, 200)?;

    // Add memory timeline
    document = add_memory_timeline(document, &tracked_vars, 100, 450)?;

    // Add categorized allocations
    document = add_categorized_allocations(document, &tracked_vars, 100, 700)?;

    // Add summary
    let summary_text = format!(
        "Total: {} tracked variables using {} memory",
        tracked_vars.len(),
        format_bytes(stats.active_memory)
    );
    let summary = SvgText::new(summary_text)
        .set("x", width / 2)
        .set("y", height - 30)
        .set("text-anchor", "middle")
        .set("font-size", 16)
        .set("font-weight", "bold")
        .set("fill", "#ECF0F1");
    document = document.add(summary);

    Ok(document)
}

/// Create lifecycle timeline SVG with interactive features
fn create_lifecycle_timeline_svg(
    allocations: &[AllocationInfo],
    stats: &MemoryStats,
) -> TrackingResult<Document> {
    let width = 1600;
    let height = 1200;

    let mut document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", width)
        .set("height", height)
        .set("style", "background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); font-family: 'Inter', 'Segoe UI', sans-serif;");

    // Add interactive styles
    let styles = Style::new(
        r#"
        .timeline-bar { transition: all 0.3s ease; cursor: pointer; }
        .timeline-bar:hover { stroke: #FFFFFF; stroke-width: 3; filter: drop-shadow(0 0 12px rgba(255,255,255,0.8)); }
        .variable-label { fill: #FFFFFF; font-size: 13px; font-weight: 600; text-shadow: 1px 1px 2px rgba(0,0,0,0.5); }
        .memory-label { fill: #E2E8F0; font-size: 11px; text-shadow: 1px 1px 2px rgba(0,0,0,0.5); }
        .section-title { fill: #FFFFFF; font-size: 20px; font-weight: 700; text-shadow: 2px 2px 4px rgba(0,0,0,0.5); }
        .section-bg { fill: rgba(255,255,255,0.1); stroke: rgba(255,255,255,0.2); stroke-width: 1; rx: 12; }
    "#,
    );
    document = document.add(styles);

    // Title
    let title = SvgText::new("Variable Lifecycle Timeline")
        .set("x", width / 2)
        .set("y", 40)
        .set("text-anchor", "middle")
        .set("font-size", 32)
        .set("font-weight", "bold")
        .set("fill", "#FFFFFF")
        .set("style", "text-shadow: 3px 3px 6px rgba(0,0,0,0.5);");
    document = document.add(title);

    let tracked_vars: Vec<_> = allocations
        .iter()
        .filter(|a| a.var_name.is_some())
        .collect();

    if tracked_vars.is_empty() {
        let no_data = SvgText::new("No tracked variables found")
            .set("x", width / 2)
            .set("y", height / 2)
            .set("text-anchor", "middle")
            .set("font-size", 18)
            .set("fill", "#FFFFFF");
        document = document.add(no_data);
        return Ok(document);
    }

    // Add timeline section
    document = add_timeline_section(document, &tracked_vars, 100, 400)?;

    // Add memory analysis section
    document = add_memory_section(document, &tracked_vars, stats, 600, 300)?;

    // Add variable relationships
    document = add_relationships_section(document, &tracked_vars, 920, 200)?;

    Ok(document)
}

/// Add call stack analysis section
fn add_call_stack_analysis(
    mut document: Document,
    allocations: &[AllocationInfo],
    start_x: i32,
    start_y: i32,
) -> TrackingResult<Document> {
    // Section title
    let title = SvgText::new("Call Stack Analysis - Memory Sources")
        .set("x", start_x)
        .set("y", start_y)
        .set("font-size", 18)
        .set("font-weight", "bold")
        .set("fill", "#ECF0F1");
    document = document.add(title);

    // Group allocations by variable name and type
    let mut source_stats: HashMap<String, (usize, usize)> = HashMap::new();

    for allocation in allocations {
        let source_key = if let Some(var_name) = &allocation.var_name {
            if let Some(type_name) = &allocation.type_name {
                let simplified_type = get_simple_type(type_name);
                format!(
                    "{}({}) memory: {}",
                    var_name,
                    simplified_type,
                    format_bytes(allocation.size)
                )
            } else {
                format!(
                    "{}(Unknown Type) memory: {}",
                    var_name,
                    format_bytes(allocation.size)
                )
            }
        } else if let Some(type_name) = &allocation.type_name {
            let simplified_type = get_simple_type(type_name);
            format!("System/Runtime {simplified_type} allocations (untracked)")
        } else {
            "System/Runtime allocations (no type info)".to_string()
        };

        let entry = source_stats.entry(source_key).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += allocation.size;
    }

    // Sort and display top sources
    let mut sorted_sources: Vec<_> = source_stats.iter().collect();
    sorted_sources.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));

    for (i, (source, (count, total_size))) in sorted_sources.iter().take(8).enumerate() {
        let y = start_y + 30 + (i as i32) * 25;
        let node_size = ((*total_size as f64 / sorted_sources[0].1 .1 as f64) * 15.0 + 5.0) as i32;

        let colors = [
            "#E74C3C", "#F39C12", "#27AE60", "#3498DB", "#9B59B6", "#1ABC9C", "#E67E22", "#34495E",
        ];
        let color = colors[i % colors.len()];

        let node = Circle::new()
            .set("cx", start_x + 30)
            .set("cy", y)
            .set("r", node_size)
            .set("fill", color)
            .set("stroke", "#ECF0F1")
            .set("stroke-width", 2);
        document = document.add(node);

        let label = SvgText::new(format!("{source} ({count} allocs)"))
            .set("x", start_x + 60)
            .set("y", y + 5)
            .set("font-size", 11)
            .set("font-weight", "500")
            .set("fill", "#ECF0F1");
        document = document.add(label);
    }

    Ok(document)
}

/// Add memory timeline section
fn add_memory_timeline(
    mut document: Document,
    tracked_vars: &[&AllocationInfo],
    start_x: i32,
    start_y: i32,
) -> TrackingResult<Document> {
    let title = SvgText::new("Variable Allocation Timeline")
        .set("x", start_x)
        .set("y", start_y)
        .set("font-size", 18)
        .set("font-weight", "bold")
        .set("fill", "#ECF0F1");
    document = document.add(title);

    let timeline_width = 1000;
    let timeline_y = start_y + 40;

    // Draw timeline axis
    let axis = Line::new()
        .set("x1", start_x)
        .set("y1", timeline_y)
        .set("x2", start_x + timeline_width)
        .set("y2", timeline_y)
        .set("stroke", "#BDC3C7")
        .set("stroke-width", 2);
    document = document.add(axis);

    // Calculate time range
    let min_time = tracked_vars
        .first()
        .map(|a| a.timestamp_alloc as u64)
        .unwrap_or(0);
    let max_time = tracked_vars
        .iter()
        .map(|a| a.timestamp_alloc as u64)
        .max()
        .unwrap_or(min_time + 1000);
    let time_range = (max_time - min_time).max(1000);

    // Add variables to timeline
    for (i, allocation) in tracked_vars.iter().take(10).enumerate() {
        let y = timeline_y + 30 + (i as i32) * 20;
        let x_pos = start_x
            + ((allocation.timestamp_alloc as u64).saturating_sub(min_time) as f64
                / time_range as f64
                * timeline_width as f64) as i32;

        let var_name = allocation.var_name.as_ref().unwrap();
        let type_name = allocation.type_name.as_deref().unwrap_or("Unknown");
        let simple_type = get_simple_type(type_name);

        // Variable marker
        let marker = Circle::new()
            .set("cx", x_pos)
            .set("cy", y)
            .set("r", 6)
            .set("fill", get_type_color(&simple_type))
            .set("stroke", "#ECF0F1")
            .set("stroke-width", 2);
        document = document.add(marker);

        // Variable label
        let label_text = format!(
            "{}({}) memory: {}",
            var_name,
            simple_type,
            format_bytes(allocation.size)
        );
        let label = SvgText::new(label_text)
            .set("x", start_x + timeline_width + 20)
            .set("y", y + 4)
            .set("font-size", 11)
            .set("font-weight", "500")
            .set("fill", "#ECF0F1");
        document = document.add(label);
    }

    Ok(document)
}

/// Add categorized allocations section
fn add_categorized_allocations(
    mut document: Document,
    tracked_vars: &[&AllocationInfo],
    start_x: i32,
    start_y: i32,
) -> TrackingResult<Document> {
    let title = SvgText::new("Tracked Variables by Category")
        .set("x", start_x)
        .set("y", start_y)
        .set("font-size", 18)
        .set("font-weight", "bold")
        .set("fill", "#ECF0F1");
    document = document.add(title);

    // Group by type
    let mut categories: HashMap<String, Vec<&AllocationInfo>> = HashMap::new();
    for allocation in tracked_vars {
        let type_name = allocation.type_name.as_deref().unwrap_or("Unknown");
        let simple_type = get_simple_type(type_name);
        categories.entry(simple_type).or_default().push(allocation);
    }

    let chart_x = start_x;
    let chart_y = start_y + 40;
    let bar_height = 30;

    for (i, (category_name, allocations)) in categories.iter().enumerate() {
        let y = chart_y + (i as i32) * 40;
        let total_size: usize = allocations.iter().map(|a| a.size).sum();
        let bar_width = (total_size as f64 / 1024.0 * 2.0) as i32 + 50; // Scale for visibility

        // Category bar
        let bar = Rectangle::new()
            .set("x", chart_x)
            .set("y", y)
            .set("width", bar_width)
            .set("height", bar_height)
            .set("fill", get_type_color(category_name))
            .set("rx", 4);
        document = document.add(bar);

        // Category label
        let category_label = SvgText::new(category_name)
            .set("x", chart_x + 10)
            .set("y", y + 20)
            .set("font-size", 12)
            .set("font-weight", "bold")
            .set("fill", "#FFFFFF");
        document = document.add(category_label);

        // Variable names and memory usage
        let var_names: Vec<String> = allocations
            .iter()
            .filter_map(|a| a.var_name.as_ref())
            .take(3)
            .map(|name| {
                format!(
                    "{}({})",
                    name,
                    format_bytes(
                        allocations
                            .iter()
                            .find(|a| a.var_name.as_ref() == Some(name))
                            .map(|a| a.size)
                            .unwrap_or(0)
                    )
                )
            })
            .collect();

        let display_text = format!("{} - {}", format_bytes(total_size), var_names.join(", "));
        let size_text = SvgText::new(display_text)
            .set("x", chart_x + bar_width + 10)
            .set("y", y + 20)
            .set("font-size", 10)
            .set("fill", "#ECF0F1");
        document = document.add(size_text);
    }

    Ok(document)
}

/// Add timeline section for lifecycle visualization
fn add_timeline_section(
    mut document: Document,
    tracked_vars: &[&AllocationInfo],
    start_y: i32,
    section_height: i32,
) -> TrackingResult<Document> {
    let margin_left = 300;
    let timeline_width = 1000;

    // Section background
    let section_bg = Rectangle::new()
        .set("x", 50)
        .set("y", start_y - 20)
        .set("width", 1500)
        .set("height", section_height)
        .set("class", "section-bg");
    document = document.add(section_bg);

    // Section title
    let section_title = SvgText::new("Interactive Timeline - Variable Lifecycles")
        .set("x", 70)
        .set("y", start_y + 10)
        .set("class", "section-title");
    document = document.add(section_title);

    // Calculate time range
    let min_time = tracked_vars
        .first()
        .map(|a| a.timestamp_alloc as u64)
        .unwrap_or(0);
    let max_time = tracked_vars
        .iter()
        .map(|a| a.timestamp_alloc as u64)
        .max()
        .unwrap_or(min_time + 1000);
    let time_range = (max_time - min_time).max(1000);

    // Draw time axis
    let timeline_y = start_y + 60;
    let axis = Line::new()
        .set("x1", margin_left)
        .set("y1", timeline_y)
        .set("x2", margin_left + timeline_width)
        .set("y2", timeline_y)
        .set("stroke", "rgba(255,255,255,0.5)")
        .set("stroke-width", 2);
    document = document.add(axis);

    // Draw variable lanes
    for (i, allocation) in tracked_vars.iter().take(6).enumerate() {
        let lane_y = timeline_y + 40 + (i as i32) * 45;

        let var_name = allocation.var_name.as_ref().unwrap();
        let type_name = allocation.type_name.as_deref().unwrap_or("Unknown");
        let simple_type = get_simple_type(type_name);

        // Variable label
        let var_label = SvgText::new(format!("{var_name} ({simple_type})"))
            .set("x", margin_left - 10)
            .set("y", lane_y + 5)
            .set("text-anchor", "end")
            .set("class", "variable-label");
        document = document.add(var_label);

        // Timeline bar
        let start_x = margin_left
            + ((allocation.timestamp_alloc as u64 - min_time) as f64 / time_range as f64
                * timeline_width as f64) as i32;
        let bar_width = timeline_width / 8; // Assume still alive

        let color = get_type_color(&simple_type);
        let timeline_bar = Rectangle::new()
            .set("x", start_x)
            .set("y", lane_y - 6)
            .set("width", bar_width)
            .set("height", 12)
            .set("fill", color)
            .set("rx", 6)
            .set("class", "timeline-bar");
        document = document.add(timeline_bar);

        // Birth and alive indicators
        let birth_circle = Circle::new()
            .set("cx", start_x)
            .set("cy", lane_y)
            .set("r", 4)
            .set("fill", color);
        document = document.add(birth_circle);

        let alive_circle = Circle::new()
            .set("cx", start_x + bar_width)
            .set("cy", lane_y)
            .set("r", 4)
            .set("fill", "#FF6B6B");
        document = document.add(alive_circle);
    }

    Ok(document)
}

/// Add memory section for lifecycle visualization
fn add_memory_section(
    mut document: Document,
    tracked_vars: &[&AllocationInfo],
    _stats: &MemoryStats,
    start_y: i32,
    section_height: i32,
) -> TrackingResult<Document> {
    // Section background
    let section_bg = Rectangle::new()
        .set("x", 50)
        .set("y", start_y - 20)
        .set("width", 1500)
        .set("height", section_height)
        .set("class", "section-bg");
    document = document.add(section_bg);

    // Section title
    let section_title = SvgText::new("Memory Usage Analysis")
        .set("x", 70)
        .set("y", start_y + 10)
        .set("class", "section-title");
    document = document.add(section_title);

    // Group by type
    let mut type_stats: HashMap<String, (usize, usize, Vec<String>)> = HashMap::new();
    for allocation in tracked_vars {
        if let Some(var_name) = &allocation.var_name {
            let type_name = allocation.type_name.as_deref().unwrap_or("Unknown");
            let simple_type = get_simple_type(type_name);

            let entry = type_stats.entry(simple_type).or_insert((0, 0, Vec::new()));
            entry.0 += 1;
            entry.1 += allocation.size;
            entry
                .2
                .push(format!("{}({})", var_name, format_bytes(allocation.size)));
        }
    }

    // Draw memory bars
    let chart_x = 100;
    let chart_y = start_y + 50;
    let max_size = type_stats
        .values()
        .map(|(_, size, _)| *size)
        .max()
        .unwrap_or(1);

    for (i, (type_name, (count, total_size, vars))) in type_stats.iter().enumerate() {
        let y = chart_y + (i as i32) * 40;
        let bar_width = ((*total_size as f64 / max_size as f64) * 400.0) as i32;

        let color = get_type_color(type_name);
        let memory_bar = Rectangle::new()
            .set("x", chart_x)
            .set("y", y)
            .set("width", bar_width)
            .set("height", 25)
            .set("fill", color)
            .set("rx", 4);
        document = document.add(memory_bar);

        // Labels
        let type_label = SvgText::new(format!("{type_name} ({count} vars)"))
            .set("x", chart_x + 10)
            .set("y", y + 17)
            .set("font-size", 11)
            .set("font-weight", "bold")
            .set("fill", "#FFFFFF");
        document = document.add(type_label);

        let vars_text = vars.join(", ");
        let vars_label = SvgText::new(if vars_text.len() > 50 {
            format!("{}...", &vars_text[..47])
        } else {
            vars_text
        })
        .set("x", chart_x + bar_width + 10)
        .set("y", y + 17)
        .set("font-size", 10)
        .set("fill", "#E2E8F0");
        document = document.add(vars_label);
    }

    Ok(document)
}

/// Add relationships section for lifecycle visualization
fn add_relationships_section(
    mut document: Document,
    tracked_vars: &[&AllocationInfo],
    start_y: i32,
    section_height: i32,
) -> TrackingResult<Document> {
    // Section background
    let section_bg = Rectangle::new()
        .set("x", 50)
        .set("y", start_y - 20)
        .set("width", 1500)
        .set("height", section_height)
        .set("class", "section-bg");
    document = document.add(section_bg);

    // Section title
    let section_title = SvgText::new("Variable Relationships")
        .set("x", 70)
        .set("y", start_y + 10)
        .set("class", "section-title");
    document = document.add(section_title);

    // Draw variables as connected nodes
    let start_x = 150;
    let node_y = start_y + 60;
    let node_spacing = 200;

    for (i, allocation) in tracked_vars.iter().take(6).enumerate() {
        let x = start_x + ((i % 3) as i32) * node_spacing;
        let y = node_y + ((i / 3) as i32) * 80;

        let var_name = allocation.var_name.as_ref().unwrap();
        let type_name = allocation.type_name.as_deref().unwrap_or("Unknown");
        let simple_type = get_simple_type(type_name);
        let color = get_type_color(&simple_type);

        // Variable node
        let node = Circle::new()
            .set("cx", x)
            .set("cy", y)
            .set("r", 25)
            .set("fill", color)
            .set("stroke", "#FFFFFF")
            .set("stroke-width", 2);
        document = document.add(node);

        // Variable name
        let name_label = SvgText::new(var_name)
            .set("x", x)
            .set("y", y + 4)
            .set("text-anchor", "middle")
            .set("font-size", 10)
            .set("font-weight", "bold")
            .set("fill", "#FFFFFF");
        document = document.add(name_label);

        // Type and size below
        let info_text = format!("{} | {}", simple_type, format_bytes(allocation.size));
        let info_label = SvgText::new(info_text)
            .set("x", x)
            .set("y", y + 45)
            .set("text-anchor", "middle")
            .set("font-size", 9)
            .set("fill", "#E2E8F0");
        document = document.add(info_label);
    }

    Ok(document)
}

/// Get color for variable type
fn get_type_color(type_name: &str) -> &'static str {
    match type_name {
        "String" => "#4ECDC4",
        "Vec" => "#45B7D1",
        "Box" => "#FF6B6B",
        "Rc" | "Arc" => "#FFA07A",
        "HashMap" => "#98D8C8",
        _ => "#95A5A6",
    }
}

/// Get simple type name
fn get_simple_type(type_name: &str) -> String {
    if type_name.contains("String") {
        "String".to_string()
    } else if type_name.contains("Vec") {
        "Vec".to_string()
    } else if type_name.contains("Box") {
        "Box".to_string()
    } else if type_name.contains("Rc") {
        "Rc".to_string()
    } else if type_name.contains("Arc") {
        "Arc".to_string()
    } else if type_name.contains("HashMap") {
        "HashMap".to_string()
    } else {
        type_name
            .split("::")
            .last()
            .unwrap_or("Unknown")
            .to_string()
    }
}

/// Format bytes
fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes}B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
