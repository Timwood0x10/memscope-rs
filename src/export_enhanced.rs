//! Enhanced export functionality for memory tracking data.

use crate::tracker::MemoryTracker;
use crate::types::{AllocationInfo, MemoryStats, TrackingResult, TypeMemoryUsage};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use svg::node::element::{Circle, Rectangle, Text as SvgText};
use svg::Document;

/// Enhanced type information processing
fn enhance_type_information(memory_by_type: &[TypeMemoryUsage]) -> Vec<EnhancedTypeInfo> {
    memory_by_type
        .iter()
        .filter_map(|usage| {
            // Skip unknown types
            if usage.type_name == "Unknown" {
                return None;
            }

            // Simplify and categorize type names
            let (simplified_name, category) = simplify_type_name(&usage.type_name);

            Some(EnhancedTypeInfo {
                simplified_name,
                category,
                total_size: usage.total_size,
                allocation_count: usage.allocation_count,
            })
        })
        .collect()
}

/// Categorize allocations for better visualization
fn categorize_allocations(allocations: &[AllocationInfo]) -> Vec<AllocationCategory> {
    let mut categories: HashMap<String, AllocationCategory> = HashMap::new();

    for allocation in allocations {
        // Skip allocations without variable names or with unknown types
        if allocation.var_name.is_none()
            || allocation.type_name.as_ref().is_none_or(|t| t == "Unknown")
        {
            continue;
        }

        let type_name = allocation.type_name.as_ref().unwrap();
        let (_, category_name) = simplify_type_name(type_name);

        let category =
            categories
                .entry(category_name.clone())
                .or_insert_with(|| AllocationCategory {
                    name: category_name.clone(),
                    allocations: Vec::new(),
                    total_size: 0,
                    color: get_category_color(&category_name),
                });

        category.allocations.push(allocation.clone());
        category.total_size += allocation.size;
    }

    let mut result: Vec<_> = categories.into_values().collect();
    result.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    result
}

/// Simplify Rust type names for better readability
fn simplify_type_name(type_name: &str) -> (String, String) {
    if type_name.starts_with("alloc::vec::Vec<") || type_name.starts_with("std::vec::Vec<") {
        let inner = extract_generic_type(type_name, "Vec");
        (format!("Vec<{}>", inner), "Collections".to_string())
    } else if type_name.starts_with("alloc::string::String") || type_name == "String" {
        ("String".to_string(), "Text".to_string())
    } else if type_name.starts_with("alloc::boxed::Box<")
        || type_name.starts_with("std::boxed::Box<")
    {
        let inner = extract_generic_type(type_name, "Box");
        (format!("Box<{}>", inner), "Smart Pointers".to_string())
    } else if type_name.starts_with("alloc::rc::Rc<") || type_name.starts_with("std::rc::Rc<") {
        let inner = extract_generic_type(type_name, "Rc");
        (format!("Rc<{}>", inner), "Reference Counted".to_string())
    } else if type_name.starts_with("alloc::sync::Arc<") || type_name.starts_with("std::sync::Arc<")
    {
        let inner = extract_generic_type(type_name, "Arc");
        (format!("Arc<{}>", inner), "Thread-Safe Shared".to_string())
    } else if type_name.contains("HashMap") {
        ("HashMap".to_string(), "Collections".to_string())
    } else if type_name.contains("BTreeMap") {
        ("BTreeMap".to_string(), "Collections".to_string())
    } else if type_name.contains("VecDeque") {
        ("VecDeque".to_string(), "Collections".to_string())
    } else {
        // For other types, try to extract the last component
        let simplified = type_name
            .split("::")
            .last()
            .unwrap_or(type_name)
            .to_string();
        (simplified, "Other".to_string())
    }
}

/// Extract generic type parameter for display
fn extract_generic_type(type_name: &str, container: &str) -> String {
    if let Some(start) = type_name.find(&format!("{}<", container)) {
        let start = start + container.len() + 1;
        if let Some(end) = type_name[start..].rfind('>') {
            let inner = &type_name[start..start + end];
            // Simplify the inner type too
            return inner.split("::").last().unwrap_or(inner).to_string();
        }
    }
    "?".to_string()
}

/// Get color for category
fn get_category_color(category: &str) -> String {
    match category {
        "Collections" => "#3498db".to_string(),        // Blue
        "Text" => "#2ecc71".to_string(),               // Green
        "Smart Pointers" => "#e74c3c".to_string(),     // Red
        "Reference Counted" => "#f39c12".to_string(),  // Orange
        "Thread-Safe Shared" => "#9b59b6".to_string(), // Purple
        _ => "#95a5a6".to_string(),                    // Gray
    }
}

#[derive(Debug, Clone)]
struct EnhancedTypeInfo {
    simplified_name: String,
    category: String,
    total_size: usize,
    allocation_count: usize,
}

#[derive(Debug, Clone)]
struct AllocationCategory {
    name: String,
    allocations: Vec<AllocationInfo>,
    total_size: usize,
    color: String,
}

/// Format bytes in human readable format
fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Enhanced SVG export with better visualization
pub fn export_enhanced_svg<P: AsRef<Path>>(tracker: &MemoryTracker, path: P) -> TrackingResult<()> {
    let path = path.as_ref();

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let active_allocations = tracker.get_active_allocations()?;
    let memory_by_type = tracker.get_memory_by_type()?;
    let stats = tracker.get_stats()?;

    // Filter out unknown types and enhance type information
    let enhanced_memory_by_type = enhance_type_information(&memory_by_type);
    let categorized_allocations = categorize_allocations(&active_allocations);

    // Create larger SVG document for better visualization
    let mut document = Document::new()
        .set("viewBox", (0, 0, 1200, 800))
        .set("width", 1200)
        .set("height", 800)
        .set(
            "style",
            "background-color: #f8f9fa; font-family: 'Segoe UI', Arial, sans-serif;",
        );

    // Add enhanced title with statistics
    document = add_enhanced_header(document, &stats)?;

    // Create enhanced memory usage by type chart
    if !enhanced_memory_by_type.is_empty() {
        document = add_enhanced_type_chart(document, &enhanced_memory_by_type)?;
    }

    // Create categorized allocation visualization
    if !categorized_allocations.is_empty() {
        document = add_categorized_allocations(document, &categorized_allocations)?;
    }

    // Add memory timeline/lifecycle view
    document = add_memory_timeline(document, &active_allocations, &stats)?;

    // Write SVG to file
    let mut file = File::create(path)?;
    write!(file, "{}", document)?;

    Ok(())
}

/// Add enhanced header with statistics
fn add_enhanced_header(mut document: Document, stats: &MemoryStats) -> TrackingResult<Document> {
    // Main title
    let title = SvgText::new("Rust Memory Usage Analysis")
        .set("x", 600)
        .set("y", 40)
        .set("text-anchor", "middle")
        .set("font-size", 24)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");

    document = document.add(title);

    // Statistics panel
    let stats_bg = Rectangle::new()
        .set("x", 50)
        .set("y", 60)
        .set("width", 1100)
        .set("height", 80)
        .set("fill", "#ecf0f1")
        .set("stroke", "#bdc3c7")
        .set("stroke-width", 1)
        .set("rx", 5);

    document = document.add(stats_bg);

    // Statistics text
    let stats_text = [
        format!("Active Allocations: {}", stats.active_allocations),
        format!("Active Memory: {}", format_bytes(stats.active_memory)),
        format!("Peak Memory: {}", format_bytes(stats.peak_memory)),
        format!("Total Allocations: {}", stats.total_allocations),
    ];

    for (i, text) in stats_text.iter().enumerate() {
        let x = 80 + (i * 270);
        let stat_text = SvgText::new(text)
            .set("x", x)
            .set("y", 105)
            .set("font-size", 14)
            .set("font-weight", "600")
            .set("fill", "#34495e");

        document = document.add(stat_text);
    }

    Ok(document)
}

/// Add enhanced type chart with categories
fn add_enhanced_type_chart(
    mut document: Document,
    types: &[EnhancedTypeInfo],
) -> TrackingResult<Document> {
    let chart_x = 50;
    let chart_y = 170;
    let chart_width = 500;
    let chart_height = 300;

    // Chart background
    let bg = Rectangle::new()
        .set("x", chart_x)
        .set("y", chart_y)
        .set("width", chart_width)
        .set("height", chart_height)
        .set("fill", "white")
        .set("stroke", "#bdc3c7")
        .set("stroke-width", 1)
        .set("rx", 5);

    document = document.add(bg);

    // Chart title
    let title = SvgText::new("Memory Usage by Type")
        .set("x", chart_x + chart_width / 2)
        .set("y", chart_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 16)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");

    document = document.add(title);

    if types.is_empty() {
        let no_data = SvgText::new("No type information available")
            .set("x", chart_x + chart_width / 2)
            .set("y", chart_y + chart_height / 2)
            .set("text-anchor", "middle")
            .set("font-size", 14)
            .set("fill", "#7f8c8d");

        document = document.add(no_data);
        return Ok(document);
    }

    let max_size = types.iter().map(|t| t.total_size).max().unwrap_or(1);
    let bar_height = (chart_height - 40) / types.len().min(10);

    for (i, type_info) in types.iter().take(10).enumerate() {
        let y = chart_y + 20 + i * bar_height;
        let bar_width =
            ((type_info.total_size as f64 / max_size as f64) * (chart_width - 200) as f64) as i32;

        // Bar
        let bar = Rectangle::new()
            .set("x", chart_x + 150)
            .set("y", y)
            .set("width", bar_width)
            .set("height", bar_height - 5)
            .set("fill", get_category_color(&type_info.category))
            .set("stroke", "#34495e")
            .set("stroke-width", 1);

        document = document.add(bar);

        // Type name
        let name_text = SvgText::new(&type_info.simplified_name)
            .set("x", chart_x + 10)
            .set("y", y + bar_height / 2 + 4)
            .set("font-size", 11)
            .set("font-weight", "600")
            .set("fill", "#2c3e50");

        document = document.add(name_text);

        // Size and count
        let size_text = SvgText::new(format!(
            "{} ({} allocs)",
            format_bytes(type_info.total_size),
            type_info.allocation_count
        ))
        .set("x", chart_x + 160)
        .set("y", y + bar_height / 2 + 4)
        .set("font-size", 10)
        .set("fill", "white");

        document = document.add(size_text);
    }

    Ok(document)
}

/// Add categorized allocations visualization
fn add_categorized_allocations(
    mut document: Document,
    categories: &[AllocationCategory],
) -> TrackingResult<Document> {
    let chart_x = 600;
    let chart_y = 170;
    let chart_width = 550;
    let chart_height = 300;

    // Chart background
    let bg = Rectangle::new()
        .set("x", chart_x)
        .set("y", chart_y)
        .set("width", chart_width)
        .set("height", chart_height)
        .set("fill", "white")
        .set("stroke", "#bdc3c7")
        .set("stroke-width", 1)
        .set("rx", 5);

    document = document.add(bg);

    // Chart title
    let title = SvgText::new("Tracked Variables by Category")
        .set("x", chart_x + chart_width / 2)
        .set("y", chart_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 16)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");

    document = document.add(title);

    if categories.is_empty() {
        let no_data = SvgText::new("No tracked variables found")
            .set("x", chart_x + chart_width / 2)
            .set("y", chart_y + chart_height / 2)
            .set("text-anchor", "middle")
            .set("font-size", 14)
            .set("fill", "#7f8c8d");

        document = document.add(no_data);
        return Ok(document);
    }

    // Create simple bar chart for categories
    let max_size = categories.iter().map(|c| c.total_size).max().unwrap_or(1);
    let bar_height = (chart_height - 60) / categories.len().min(8);

    for (i, category) in categories.iter().take(8).enumerate() {
        let y = chart_y + 30 + i * bar_height;
        let bar_width =
            ((category.total_size as f64 / max_size as f64) * (chart_width - 200) as f64) as i32;

        // Bar
        let bar = Rectangle::new()
            .set("x", chart_x + 150)
            .set("y", y)
            .set("width", bar_width)
            .set("height", bar_height - 5)
            .set("fill", category.color.as_str())
            .set("stroke", "#34495e")
            .set("stroke-width", 1);

        document = document.add(bar);

        // Category name
        let name_text = SvgText::new(&category.name)
            .set("x", chart_x + 10)
            .set("y", y + bar_height / 2 + 4)
            .set("font-size", 12)
            .set("font-weight", "600")
            .set("fill", "#2c3e50");

        document = document.add(name_text);

        // Size and count
        let size_text = SvgText::new(format!(
            "{} ({} vars)",
            format_bytes(category.total_size),
            category.allocations.len()
        ))
        .set("x", chart_x + 160)
        .set("y", y + bar_height / 2 + 4)
        .set("font-size", 10)
        .set("fill", "white");

        document = document.add(size_text);
    }

    Ok(document)
}

/// Add memory timeline visualization
fn add_memory_timeline(
    mut document: Document,
    allocations: &[AllocationInfo],
    _stats: &MemoryStats,
) -> TrackingResult<Document> {
    let chart_x = 50;
    let chart_y = 500;
    let chart_width = 1100;
    let chart_height = 250;

    // Chart background
    let bg = Rectangle::new()
        .set("x", chart_x)
        .set("y", chart_y)
        .set("width", chart_width)
        .set("height", chart_height)
        .set("fill", "white")
        .set("stroke", "#bdc3c7")
        .set("stroke-width", 1)
        .set("rx", 5);

    document = document.add(bg);

    // Chart title
    let title = SvgText::new("Variable Allocation Timeline")
        .set("x", chart_x + chart_width / 2)
        .set("y", chart_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 16)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");

    document = document.add(title);

    if allocations.is_empty() {
        let no_data = SvgText::new("No allocation data available")
            .set("x", chart_x + chart_width / 2)
            .set("y", chart_y + chart_height / 2)
            .set("text-anchor", "middle")
            .set("font-size", 14)
            .set("fill", "#7f8c8d");

        document = document.add(no_data);
        return Ok(document);
    }

    // Filter and sort tracked allocations
    let mut tracked_allocs: Vec<_> = allocations
        .iter()
        .filter(|a| a.var_name.is_some())
        .collect();
    tracked_allocs.sort_by_key(|a| a.timestamp_alloc);

    if tracked_allocs.is_empty() {
        let no_data = SvgText::new("No tracked variables found")
            .set("x", chart_x + chart_width / 2)
            .set("y", chart_y + chart_height / 2)
            .set("text-anchor", "middle")
            .set("font-size", 14)
            .set("fill", "#7f8c8d");

        document = document.add(no_data);
        return Ok(document);
    }

    let min_time = tracked_allocs
        .first()
        .map(|a| a.timestamp_alloc)
        .unwrap_or(0);
    let max_time = tracked_allocs
        .last()
        .map(|a| a.timestamp_alloc)
        .unwrap_or(min_time + 1);
    let _time_range = (max_time - min_time).max(1);

    // Calculate layout parameters for better alignment
    let label_width = 200; // Reserved space for labels
    let timeline_width = chart_width - label_width - 40;
    let max_items = 8; // Limit items to prevent overcrowding
    
    // Draw timeline for tracked variables with proper spacing
    for (i, allocation) in tracked_allocs.iter().take(max_items).enumerate() {
        // Distribute items evenly across timeline instead of by timestamp
        let x = chart_x + 20 + (i * timeline_width / max_items.max(1));
        let y = chart_y + 50 + (i * 25); // Increased vertical spacing
        
        // Ensure x position stays within timeline bounds
        let x = x.min(chart_x + timeline_width).max(chart_x + 20);

        // Get color based on type category
        let color = if let Some(type_name) = &allocation.type_name {
            let (_, category) = simplify_type_name(type_name);
            get_category_color(&category)
        } else {
            "#95a5a6".to_string()
        };

        // Draw allocation point
        let point = Circle::new()
            .set("cx", x)
            .set("cy", y)
            .set("r", 5)
            .set("fill", color)
            .set("stroke", "#2c3e50")
            .set("stroke-width", 2);

        document = document.add(point);

        // Draw connecting line to label area
        let label_start_x = chart_x + timeline_width + 20;
        let line = svg::node::element::Line::new()
            .set("x1", x + 5)
            .set("y1", y)
            .set("x2", label_start_x)
            .set("y2", y)
            .set("stroke", "#bdc3c7")
            .set("stroke-width", 1)
            .set("stroke-dasharray", "3,3");

        document = document.add(line);

        // Add variable name in dedicated label area
        if let Some(var_name) = &allocation.var_name {
            let label_text = format!("{} ({})", var_name, format_bytes(allocation.size));
            let label = SvgText::new(label_text)
                .set("x", label_start_x + 5)
                .set("y", y + 4)
                .set("font-size", 11)
                .set("font-weight", "500")
                .set("fill", "#2c3e50");

            document = document.add(label);
        }
    }
    
    // Add timeline axis
    let axis_y = chart_y + chart_height - 40;
    let axis_line = svg::node::element::Line::new()
        .set("x1", chart_x + 20)
        .set("y1", axis_y)
        .set("x2", chart_x + timeline_width)
        .set("y2", axis_y)
        .set("stroke", "#34495e")
        .set("stroke-width", 2);

    document = document.add(axis_line);

    // Add axis labels
    let start_label = SvgText::new("Timeline")
        .set("x", chart_x + 20)
        .set("y", axis_y + 20)
        .set("font-size", 12)
        .set("font-weight", "600")
        .set("fill", "#7f8c8d");

    document = document.add(start_label);

    Ok(document)
}
