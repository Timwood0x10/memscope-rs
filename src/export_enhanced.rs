//! Enhanced export functionality for memory tracking data.

use crate::tracker::MemoryTracker;
use crate::types::{AllocationInfo, MemoryStats, TrackingResult, TypeMemoryUsage};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use svg::node::element::{Circle, Rectangle, Text as SvgText};
use svg::Document;

/// Enhanced type information processing with variable names
pub fn enhance_type_information(memory_by_type: &[TypeMemoryUsage], allocations: &[AllocationInfo]) -> Vec<EnhancedTypeInfo> {
    memory_by_type
        .iter()
        .filter_map(|usage| {
            // Skip unknown types
            if usage.type_name == "Unknown" {
                return None;
            }

            // Simplify and categorize type names
            let (simplified_name, category) = simplify_type_name(&usage.type_name);

            // Collect variable names for this type
            let variable_names: Vec<String> = allocations
                .iter()
                .filter_map(|alloc| {
                    if let (Some(var_name), Some(type_name)) = (&alloc.var_name, &alloc.type_name) {
                        let (alloc_simplified, _) = simplify_type_name(type_name);
                        if alloc_simplified == simplified_name {
                            Some(var_name.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .take(5) // Limit to 5 variable names
                .collect();

            Some(EnhancedTypeInfo {
                simplified_name,
                category,
                total_size: usage.total_size,
                allocation_count: usage.allocation_count,
                variable_names,
            })
        })
        .collect()
}

/// Categorize allocations for better visualization
pub fn categorize_allocations(allocations: &[AllocationInfo]) -> Vec<AllocationCategory> {
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
        (format!("Vec<{inner}>"), "Collections".to_string())
    } else if type_name.starts_with("alloc::string::String") || type_name == "String" {
        ("String".to_string(), "Text".to_string())
    } else if type_name.starts_with("alloc::boxed::Box<")
        || type_name.starts_with("std::boxed::Box<")
    {
        let inner = extract_generic_type(type_name, "Box");
        (format!("Box<{inner}>"), "Smart Pointers".to_string())
    } else if type_name.starts_with("alloc::rc::Rc<") || type_name.starts_with("std::rc::Rc<") {
        let inner = extract_generic_type(type_name, "Rc");
        (format!("Rc<{inner}>"), "Reference Counted".to_string())
    } else if type_name.starts_with("alloc::sync::Arc<") || type_name.starts_with("std::sync::Arc<")
    {
        let inner = extract_generic_type(type_name, "Arc");
        (format!("Arc<{inner}>"), "Thread-Safe Shared".to_string())
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
    if let Some(start) = type_name.find(&format!("{container}<")) {
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
pub struct EnhancedTypeInfo {
    simplified_name: String,
    category: String,
    total_size: usize,
    allocation_count: usize,
    variable_names: Vec<String>, // Add variable names
}

#[derive(Debug, Clone)]
pub struct AllocationCategory {
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

/// Enhanced SVG export with comprehensive visualization
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
    let enhanced_memory_by_type = enhance_type_information(&memory_by_type, &active_allocations);
    let _categorized_allocations = categorize_allocations(&active_allocations);

    // Create COMPACT SVG document - REDUCED HEIGHT for space efficiency
    let mut document = Document::new()
        .set("viewBox", (0, 0, 1800, 400))
        .set("width", 1800)
        .set("height", 400)
        .set(
            "style",
            "background: linear-gradient(135deg, #2C3E50 0%, #34495E 100%); font-family: 'Segoe UI', Arial, sans-serif;",
        );

    // COMPACT LAYOUT - Only essential components for 400px height
    
    // Title (y: 0-50)
    let title = SvgText::new("Top 3 Memory Analysis - Compact View")
        .set("x", 700)
        .set("y", 30)
        .set("text-anchor", "middle")
        .set("font-size", 24)
        .set("font-weight", "bold")
        .set("fill", "#FFFFFF");
    document = document.add(title);

    // Only show TOP 3 type chart (y: 60-350)
    if !enhanced_memory_by_type.is_empty() {
        document = add_compact_type_chart(document, &enhanced_memory_by_type)?;
    }

    // Compact summary (y: 350-400)
    document = add_compact_summary(document, &stats, &active_allocations)?;

    // Write SVG to file
    let mut file = File::create(path)?;
    write!(file, "{document}")?;

    Ok(())
}

/// Add compact type chart - TOP 3 ONLY with progress bars
fn add_compact_type_chart(
    mut document: Document,
    types: &[EnhancedTypeInfo],
) -> TrackingResult<Document> {
    let chart_x = 100;
    let chart_y = 60;
    let chart_width = 1200;
    
    if types.is_empty() {
        let no_data = SvgText::new("No tracked variables found")
            .set("x", chart_x + chart_width / 2)
            .set("y", 200)
            .set("text-anchor", "middle")
            .set("font-size", 16)
            .set("fill", "#E74C3C");
        document = document.add(no_data);
        return Ok(document);
    }

    let max_size = types.iter().map(|t| t.total_size).max().unwrap_or(1);
    
    // Show TOP 3 ONLY
    for (i, type_info) in types.iter().take(3).enumerate() {
        let y = chart_y + (i as i32) * 80;
        
        // Progress bar background
        let bg_bar = Rectangle::new()
            .set("x", chart_x)
            .set("y", y)
            .set("width", 600)
            .set("height", 30)
            .set("fill", "#34495E")
            .set("stroke", "#ECF0F1")
            .set("stroke-width", 1)
            .set("rx", 6);
        document = document.add(bg_bar);
        
        // Progress bar fill
        let bar_width = ((type_info.total_size as f64 / max_size as f64) * 600.0) as i32;
        let color = get_category_color(&type_info.category);
        let progress_bar = Rectangle::new()
            .set("x", chart_x)
            .set("y", y)
            .set("width", bar_width)
            .set("height", 30)
            .set("fill", color)
            .set("rx", 6);
        document = document.add(progress_bar);
        
        // Type and size info
        let content_text = format!(
            "{} ({} vars) | Total: {}",
            type_info.simplified_name, 
            type_info.allocation_count, 
            format_bytes(type_info.total_size)
        );
        
        let content_label = SvgText::new(content_text)
            .set("x", chart_x + 10)
            .set("y", y + 20)
            .set("font-size", 12)
            .set("font-weight", "600")
            .set("fill", "#FFFFFF")
            .set("text-shadow", "1px 1px 2px rgba(0,0,0,0.8)");
        document = document.add(content_label);
        
        // Progress percentage
        let percentage = (type_info.total_size as f64 / max_size as f64 * 100.0) as i32;
        let percent_label = SvgText::new(format!("{}%", percentage))
            .set("x", chart_x + 720)
            .set("y", y + 20)
            .set("font-size", 14)
            .set("font-weight", "bold")
            .set("fill", "#ECF0F1");
        document = document.add(percent_label);
        
        // Variable names below
        let var_names_text = if type_info.variable_names.is_empty() {
            "no tracked vars".to_string()
        } else {
            format!("Variables: {}", type_info.variable_names.join(", "))
        };
        
        let vars_label = SvgText::new(var_names_text)
            .set("x", chart_x + 10)
            .set("y", y + 45)
            .set("font-size", 9)
            .set("fill", "#94A3B8")
            .set("font-style", "italic");
        document = document.add(vars_label);
    }

    Ok(document)
}

/// Add compact summary
fn add_compact_summary(
    mut document: Document,
    stats: &MemoryStats,
    allocations: &[AllocationInfo],
) -> TrackingResult<Document> {
    let tracked_vars = allocations.iter().filter(|a| a.var_name.is_some()).count();
    
    let summary_text = format!(
        "Showing TOP 3 memory-consuming types | Total tracked: {} variables | Active memory: {}",
        tracked_vars,
        format_bytes(stats.active_memory)
    );
    
    let summary = SvgText::new(summary_text)
        .set("x", 700)
        .set("y", 370)
        .set("text-anchor", "middle")
        .set("font-size", 14)
        .set("font-weight", "bold")
        .set("fill", "#ECF0F1");
    document = document.add(summary);

    Ok(document)
}

/// Add enhanced header with statistics
pub fn add_enhanced_header(mut document: Document, stats: &MemoryStats) -> TrackingResult<Document> {
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
        .set("width", 1700) // 适应1800px宽度
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
pub fn add_enhanced_type_chart(
    mut document: Document,
    types: &[EnhancedTypeInfo],
) -> TrackingResult<Document> {
    let chart_x = 50;
    let chart_y = 670;
    let chart_width = 850;
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

        // Size and count with variable names - exactly what you wanted!
        let var_names_text = if type_info.variable_names.is_empty() {
            "no tracked vars".to_string()
        } else {
            type_info.variable_names.join(", ")
        };
        
        let size_text = SvgText::new(format!(
            "{} ({} allocs) - Variables: {}",
            format_bytes(type_info.total_size),
            type_info.allocation_count,
            var_names_text
        ))
        .set("x", chart_x + 160)
        .set("y", y + bar_height / 2 + 4)
        .set("font-size", 12)
        .set("font-weight", "bold")
        .set("fill", "#FFFFFF")
        .set("text-shadow", "1px 1px 2px rgba(0,0,0,0.8)");

        document = document.add(size_text);
    }

    Ok(document)
}

/// Add categorized allocations visualization
pub fn add_categorized_allocations(
    mut document: Document,
    categories: &[AllocationCategory],
) -> TrackingResult<Document> {
    let chart_x = 50;
    let chart_y = 1020;
    let chart_width = 850;
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

        // Enhanced variable names display - 优化文字溢出问题
        let var_names: Vec<String> = category.allocations.iter()
            .filter_map(|a| {
                if let Some(var_name) = &a.var_name {
                    let type_name = a.type_name.as_deref().unwrap_or("Unknown");
                    let (simplified_type, _) = simplify_type_name(type_name);
                    // 缩短变量名显示，避免溢出
                    let short_var = if var_name.len() > 12 {
                        format!("{}...", &var_name[..9])
                    } else {
                        var_name.clone()
                    };
                    Some(format!("{}({})", short_var, simplified_type))
                } else {
                    None
                }
            })
            .take(3) // 减少显示的变量数量，避免溢出
            .collect();
        
        let mut display_text = if var_names.is_empty() {
            format!("{} ({} vars)", format_bytes(category.total_size), category.allocations.len())
        } else {
            format!("{} - Vars: {}", format_bytes(category.total_size), var_names.join(", "))
        };
        
        // 动态截断文字，确保不超出图表边界
        let max_text_width = chart_width - 180; // 预留边距
        let estimated_char_width = 7; // 估算每个字符宽度
        let max_chars = (max_text_width / estimated_char_width) as usize;
        
        if display_text.len() > max_chars {
            display_text = format!("{}...", &display_text[..max_chars.saturating_sub(3)]);
        }
        
        let size_text = SvgText::new(display_text)
            .set("x", chart_x + 160)
            .set("y", y + bar_height / 2 + 4)
            .set("font-size", 11) // 稍微减小字体避免溢出
            .set("font-weight", "bold")
            .set("fill", "#FFFFFF")
            .set("text-shadow", "1px 1px 2px rgba(0,0,0,0.8)");

        document = document.add(size_text);
    }

    Ok(document)
}

/// Add memory timeline visualization
pub fn add_memory_timeline(
    mut document: Document,
    allocations: &[AllocationInfo],
    _stats: &MemoryStats,
) -> TrackingResult<Document> {
    let chart_x = 50;
    let chart_y = 1720;
    let chart_width = 1700;
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

    // Calculate layout parameters for better alignment and prevent text overflow
    let label_width = 400; // Increased reserved space for labels to prevent overflow
    let timeline_width = chart_width - label_width - 60; // More margin
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

        // Add variable name with type in dedicated label area - prevent overflow
        if let Some(var_name) = &allocation.var_name {
            let type_name = allocation.type_name.as_deref().unwrap_or("Unknown");
            let (simplified_type, _) = simplify_type_name(type_name);
            let mut label_text = format!("{}({}) memory: {}", var_name, simplified_type, format_bytes(allocation.size));
            
            // Truncate text if too long to prevent overflow
            if label_text.len() > 45 {
                label_text = format!("{}...", &label_text[..42]);
            }
            
            let label = SvgText::new(label_text)
                .set("x", label_start_x + 5)
                .set("y", y + 4)
                .set("font-size", 10) // Slightly smaller font
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

/// Add fragmentation analysis chart
pub fn add_fragmentation_analysis(
    mut document: Document,
    allocations: &[AllocationInfo],
) -> TrackingResult<Document> {
    let chart_x = 950;
    let chart_y = 650; // 修复：向上移动20px，避免下移问题
    let chart_width = 750; // 修复：适应新的1600px宽度
    let chart_height = 300;

    // Chart background
    let bg = Rectangle::new()
        .set("x", chart_x)
        .set("y", chart_y)
        .set("width", chart_width)
        .set("height", chart_height)
        .set("fill", "white")
        .set("stroke", "#f39c12")
        .set("stroke-width", 2)
        .set("rx", 10);

    document = document.add(bg);

    // Chart title
    let title = SvgText::new("Memory Fragmentation Analysis")
        .set("x", chart_x + chart_width / 2)
        .set("y", chart_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 18)
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

    // Create size distribution histogram
    let size_buckets = [
        (0, 64, "Tiny (0-64B)"),
        (65, 256, "Small (65-256B)"),
        (257, 1024, "Medium (257B-1KB)"),
        (1025, 4096, "Large (1-4KB)"),
        (4097, 16384, "XLarge (4-16KB)"),
        (16385, usize::MAX, "Huge (>16KB)"),
    ];

    let mut bucket_counts = vec![0; size_buckets.len()];

    for allocation in allocations {
        for (i, &(min, max, _)) in size_buckets.iter().enumerate() {
            if allocation.size >= min && allocation.size <= max {
                bucket_counts[i] += 1;
                break;
            }
        }
    }

    let max_count = bucket_counts.iter().max().copied().unwrap_or(1);
    let bar_width = (chart_width - 100) / size_buckets.len();

    // Draw histogram bars
    for (i, (&(_, _, label), &count)) in size_buckets.iter().zip(bucket_counts.iter()).enumerate() {
        let x = chart_x + 50 + i * bar_width;
        let bar_height = if max_count > 0 {
            (count as f64 / max_count as f64 * (chart_height - 80) as f64) as i32
        } else {
            0
        };
        let y = chart_y + chart_height - 40 - bar_height;

        // Color based on fragmentation level
        let color = match i {
            0..=1 => "#27ae60", // Green for small allocations
            2..=3 => "#f39c12", // Orange for medium
            _ => "#e74c3c",     // Red for large (potential fragmentation)
        };

        let bar = Rectangle::new()
            .set("x", x)
            .set("y", y)
            .set("width", bar_width - 5)
            .set("height", bar_height)
            .set("fill", color)
            .set("stroke", "#2c3e50")
            .set("stroke-width", 1);

        document = document.add(bar);

        // Count label
        let count_text = SvgText::new(count.to_string())
            .set("x", x + bar_width / 2)
            .set("y", y - 5)
            .set("text-anchor", "middle")
            .set("font-size", 12)
            .set("font-weight", "bold")
            .set("fill", "#2c3e50");

        document = document.add(count_text);

        // Size label
        let size_text = SvgText::new(label)
            .set("x", x + bar_width / 2)
            .set("y", chart_y + chart_height - 10)
            .set("text-anchor", "middle")
            .set("font-size", 10)
            .set("fill", "#7f8c8d");

        document = document.add(size_text);
    }

    Ok(document)
}

/// Add call stack analysis visualization
pub fn add_callstack_analysis(
    mut document: Document,
    allocations: &[AllocationInfo],
) -> TrackingResult<Document> {
    let chart_x = 950;
    let chart_y = 1000; // 修复：向上移动20px，避免下移问题
    let chart_width = 750; // 修复：适应新的1600px宽度
    let chart_height = 300;

    // Chart background
    let bg = Rectangle::new()
        .set("x", chart_x)
        .set("y", chart_y)
        .set("width", chart_width)
        .set("height", chart_height)
        .set("fill", "white")
        .set("stroke", "#9b59b6")
        .set("stroke-width", 2)
        .set("rx", 10);

    document = document.add(bg);

    // Chart title
    let title = SvgText::new("Call Stack Analysis")
        .set("x", chart_x + chart_width / 2)
        .set("y", chart_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 18)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");

    document = document.add(title);

    // Group allocations by variable name and type with better categorization
    let mut source_stats: HashMap<String, (usize, usize)> = HashMap::new();

    for allocation in allocations {
        // Create a more descriptive key that helps identify allocations
        let source_key = if let Some(var_name) = &allocation.var_name {
            // Tracked variables - show variable name and type
            if let Some(type_name) = &allocation.type_name {
                let (simplified_type, _) = simplify_type_name(type_name);
                format!("{}({}) memory: {}", var_name, simplified_type, format_bytes(allocation.size))
            } else {
                format!("{}(Unknown Type) memory: {}", var_name, format_bytes(allocation.size))
            }
        } else if let Some(type_name) = &allocation.type_name {
            // Untracked allocations with known type - categorize by type and source
            let (simplified_type, _) = simplify_type_name(type_name);
            
            // Try to identify the source of untracked allocations
            if type_name.contains("std::") || type_name.contains("alloc::") {
                format!("System/Runtime {} (untracked)", simplified_type)
            } else if simplified_type.contains("Vec") {
                "Internal Vec allocations (untracked)".to_string()
            } else if simplified_type.contains("String") {
                "Internal String allocations (untracked)".to_string()
            } else if simplified_type.contains("HashMap") {
                "Internal HashMap allocations (untracked)".to_string()
            } else {
                format!("Internal {} allocations (untracked)", simplified_type)
            }
        } else {
            // Completely unknown allocations
            "System/Runtime allocations (no type info)".to_string()
        };

        let entry = source_stats.entry(source_key).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += allocation.size;
    }

    if source_stats.is_empty() {
        let no_data = SvgText::new("No call stack data available")
            .set("x", chart_x + chart_width / 2)
            .set("y", chart_y + chart_height / 2)
            .set("text-anchor", "middle")
            .set("font-size", 14)
            .set("fill", "#7f8c8d");

        document = document.add(no_data);
        return Ok(document);
    }

    // Sort by total size and take top 10
    let mut sorted_sources: Vec<_> = source_stats.iter().collect();
    sorted_sources.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));

    let max_size = sorted_sources
        .first()
        .map(|(_, (_, size))| *size)
        .unwrap_or(1);

    // Draw tree-like visualization
    for (i, (source, (count, total_size))) in sorted_sources.iter().take(10).enumerate() {
        let y = chart_y + 40 + i * 25;
        let node_size = ((*total_size as f64 / max_size as f64) * 15.0 + 5.0) as i32;

        // Draw node
        let colors = ["#e74c3c", "#f39c12", "#27ae60", "#3498db", "#9b59b6"];
        let color = colors[i % colors.len()];

        let node = Circle::new()
            .set("cx", chart_x + 50)
            .set("cy", y)
            .set("r", node_size)
            .set("fill", color)
            .set("stroke", "#2c3e50")
            .set("stroke-width", 2);

        document = document.add(node);

        // Source label
        let source_text = format!("{source} ({count} allocs, {total_size} bytes)");

        let label = SvgText::new(source_text)
            .set("x", chart_x + 80)
            .set("y", y + 5)
            .set("font-size", 11)
            .set("font-weight", "500")
            .set("fill", "#2c3e50");

        document = document.add(label);
    }

    Ok(document)
}

/// Add memory growth trends visualization
pub fn add_memory_growth_trends(
    mut document: Document,
    allocations: &[AllocationInfo],
    stats: &MemoryStats,
) -> TrackingResult<Document> {
    let chart_x = 50;
    let chart_y = 1370;
    let chart_width = 1700;
    let chart_height = 300;

    // Chart background
    let bg = Rectangle::new()
        .set("x", chart_x)
        .set("y", chart_y)
        .set("width", chart_width)
        .set("height", chart_height)
        .set("fill", "white")
        .set("stroke", "#27ae60")
        .set("stroke-width", 2)
        .set("rx", 10);

    document = document.add(bg);

    // Chart title
    let title = SvgText::new("Memory Growth Trends")
        .set("x", chart_x + chart_width / 2)
        .set("y", chart_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 18)
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

    // Create simplified trend visualization
    let time_points = 10;
    let point_width = (chart_width - 100) / time_points;

    for i in 0..time_points {
        let x = chart_x + 50 + i * point_width;
        let simulated_memory = stats.active_memory / time_points * (i + 1);
        let y = chart_y + chart_height
            - 50
            - ((simulated_memory as f64 / stats.peak_memory as f64) * (chart_height - 100) as f64)
                as i32;

        // Draw data points
        let point = Circle::new()
            .set("cx", x)
            .set("cy", y)
            .set("r", 4)
            .set("fill", "#27ae60")
            .set("stroke", "#2c3e50")
            .set("stroke-width", 1);

        document = document.add(point);

        // Connect with lines
        if i > 0 {
            let prev_x = chart_x + 50 + (i - 1) * point_width;
            let prev_memory = stats.active_memory / time_points * i;
            let prev_y = chart_y + chart_height
                - 50
                - ((prev_memory as f64 / stats.peak_memory as f64) * (chart_height - 100) as f64)
                    as i32;

            let line = svg::node::element::Line::new()
                .set("x1", prev_x)
                .set("y1", prev_y)
                .set("x2", x)
                .set("y2", y)
                .set("stroke", "#27ae60")
                .set("stroke-width", 2);

            document = document.add(line);
        }
    }

    // Add peak memory indicator
    let peak_y = chart_y + 50;
    let peak_line = svg::node::element::Line::new()
        .set("x1", chart_x + 50)
        .set("y1", peak_y)
        .set("x2", chart_x + chart_width - 50)
        .set("y2", peak_y)
        .set("stroke", "#e74c3c")
        .set("stroke-width", 2)
        .set("stroke-dasharray", "10,5");

    document = document.add(peak_line);

    let peak_label = SvgText::new(format!("Peak: {} bytes", stats.peak_memory))
        .set("x", chart_x + chart_width - 100)
        .set("y", peak_y - 10)
        .set("font-size", 12)
        .set("font-weight", "bold")
        .set("fill", "#e74c3c");

    document = document.add(peak_label);

    Ok(document)
}

/// Add interactive legend
pub fn add_interactive_legend(mut document: Document) -> TrackingResult<Document> {
    let legend_x = 50;
    let legend_y = 2070;
    let legend_width = 850;
    let legend_height = 250;

    // Legend background
    let bg = Rectangle::new()
        .set("x", legend_x)
        .set("y", legend_y)
        .set("width", legend_width)
        .set("height", legend_height)
        .set("fill", "white")
        .set("stroke", "#34495e")
        .set("stroke-width", 2)
        .set("rx", 10);

    document = document.add(bg);

    // Legend title
    let title = SvgText::new("Interactive Legend & Guide")
        .set("x", legend_x + legend_width / 2)
        .set("y", legend_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 18)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");

    document = document.add(title);

    // Legend items
    let legend_items = [
        ("#e74c3c", "High Memory Usage / Critical"),
        ("#f39c12", "Medium Usage / Warning"),
        ("#27ae60", "Low Usage / Good"),
        ("#3498db", "Performance Metrics"),
        ("#9b59b6", "Call Stack Data"),
        ("#34495e", "General Information"),
    ];

    for (i, (color, description)) in legend_items.iter().enumerate() {
        let x = legend_x + 30 + (i % 3) * 220;
        let y = legend_y + 40 + (i / 3) * 40;

        // Color swatch
        let swatch = Rectangle::new()
            .set("x", x)
            .set("y", y - 10)
            .set("width", 20)
            .set("height", 15)
            .set("fill", *color)
            .set("stroke", "#2c3e50")
            .set("stroke-width", 1);

        document = document.add(swatch);

        // Description
        let desc_text = SvgText::new(*description)
            .set("x", x + 30)
            .set("y", y)
            .set("font-size", 12)
            .set("fill", "#2c3e50");

        document = document.add(desc_text);
    }

    Ok(document)
}

/// Add comprehensive summary
pub fn add_comprehensive_summary(
    mut document: Document,
    stats: &MemoryStats,
    allocations: &[AllocationInfo],
) -> TrackingResult<Document> {
    let summary_x = 950;
    let summary_y = 2070;
    let summary_width = 800;
    let summary_height = 250;

    // Summary background
    let bg = Rectangle::new()
        .set("x", summary_x)
        .set("y", summary_y)
        .set("width", summary_width)
        .set("height", summary_height)
        .set("fill", "white")
        .set("stroke", "#2c3e50")
        .set("stroke-width", 2)
        .set("rx", 10);

    document = document.add(bg);

    // Summary title
    let title = SvgText::new("Memory Analysis Summary")
        .set("x", summary_x + summary_width / 2)
        .set("y", summary_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 18)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");

    document = document.add(title);

    // Calculate summary metrics
    let tracked_vars = allocations.iter().filter(|a| a.var_name.is_some()).count();
    let avg_size = if !allocations.is_empty() {
        allocations.iter().map(|a| a.size).sum::<usize>() / allocations.len()
    } else {
        0
    };

    let summary_items = [
        format!("Total Active Allocations: {}", stats.active_allocations),
        format!(
            "Tracked Variables: {} ({:.1}%)",
            tracked_vars,
            if stats.active_allocations > 0 {
                tracked_vars as f64 / stats.active_allocations as f64 * 100.0
            } else {
                0.0
            }
        ),
        format!("Average Allocation Size: {avg_size} bytes"),
        format!(
            "Memory Efficiency: {:.1}%",
            if stats.total_allocations > 0 {
                stats.total_deallocations as f64 / stats.total_allocations as f64 * 100.0
            } else {
                0.0
            }
        ),
        format!(
            "Peak vs Current: {} vs {} bytes",
            stats.peak_memory, stats.active_memory
        ),
    ];

    for (i, item) in summary_items.iter().enumerate() {
        let summary_text = SvgText::new(item)
            .set("x", summary_x + 30)
            .set("y", summary_y + 40 + i * 25)
            .set("font-size", 13)
            .set("font-weight", "500")
            .set("fill", "#2c3e50");

        document = document.add(summary_text);
    }

    Ok(document)
}

/// Add CSS styles for interactive elements
fn add_css_styles(mut document: Document) -> TrackingResult<Document> {
    let style = svg::node::element::Style::new(
        r#"
        .tooltip { opacity: 0; transition: opacity 0.3s; }
        .chart-element:hover .tooltip { opacity: 1; }
        .interactive-bar:hover { opacity: 0.8; cursor: pointer; }
        .legend-item:hover { background-color: #ecf0f1; }
        .heatmap-cell:hover { stroke-width: 2; }
        .trend-line { stroke-dasharray: 5,5; animation: dash 1s linear infinite; }
        @keyframes dash { to { stroke-dashoffset: -10; } }
        .performance-gauge { filter: drop-shadow(2px 2px 4px rgba(0,0,0,0.3)); }
        .callstack-node:hover { transform: scale(1.1); transform-origin: center; }
    "#,
    );

    document = document.add(style);
    Ok(document)
}

/// Add performance dashboard with key metrics
pub fn add_performance_dashboard(
    mut document: Document,
    stats: &MemoryStats,
    _allocations: &[AllocationInfo],
) -> TrackingResult<Document> {
    let dashboard_x = 50;
    let dashboard_y = 170;
    let dashboard_width = 1700;
    let dashboard_height = 200;

    // Dashboard background
    let bg = Rectangle::new()
        .set("x", dashboard_x)
        .set("y", dashboard_y)
        .set("width", dashboard_width)
        .set("height", dashboard_height)
        .set("fill", "white")
        .set("stroke", "#3498db")
        .set("stroke-width", 2)
        .set("rx", 10)
        .set("class", "performance-gauge");

    document = document.add(bg);

    // Dashboard title
    let title = SvgText::new("Performance Dashboard")
        .set("x", dashboard_x + dashboard_width / 2)
        .set("y", dashboard_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 18)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");

    document = document.add(title);

    // Calculate performance metrics
    let efficiency = if stats.total_allocations > 0 {
        stats.total_deallocations as f64 / stats.total_allocations as f64 * 100.0
    } else {
        0.0
    };

    let avg_allocation_size = if stats.total_allocations > 0 {
        stats.active_memory / stats.total_allocations
    } else {
        0
    };

    let memory_utilization = if stats.peak_memory > 0 {
        stats.active_memory as f64 / stats.peak_memory as f64 * 100.0
    } else {
        0.0
    };

    // Performance gauges - 优化布局，让饼图填充更完整
    let gauges = [
        ("Memory Efficiency", efficiency, "%", "#e74c3c"),
        ("Avg Alloc Size", avg_allocation_size as f64, "B", "#f39c12"),
        ("Memory Utilization", memory_utilization, "%", "#27ae60"),
        (
            "Active Allocs",
            stats.active_allocations as f64,
            "",
            "#9b59b6",
        ),
    ];

    for (i, (label, value, unit, color)) in gauges.iter().enumerate() {
        let gauge_x = dashboard_x + 100 + (i * 400); // 增加间距，让饼图更大
        let gauge_y = dashboard_y + 100; // 居中显示

        // Gauge background circle - 增大饼图尺寸，填充更完整
        let bg_circle = Circle::new()
            .set("cx", gauge_x)
            .set("cy", gauge_y)
            .set("r", 60) // 增大半径
            .set("fill", "none")
            .set("stroke", "#ecf0f1")
            .set("stroke-width", 12); // 增加线宽

        document = document.add(bg_circle);

        // Gauge value arc (simplified as partial circle)
        let normalized_value = if *unit == "%" {
            value.min(100.0) / 100.0
        } else {
            (value / 1000.0).min(1.0) // Normalize large values
        };

        let arc_length = normalized_value * 2.0 * std::f64::consts::PI * 60.0; // 适应新半径
        let gauge_arc = Circle::new()
            .set("cx", gauge_x)
            .set("cy", gauge_y)
            .set("r", 60) // 增大半径
            .set("fill", "none")
            .set("stroke", *color)
            .set("stroke-width", 12) // 增加线宽
            .set("stroke-dasharray", format!("{} {}", arc_length, 400.0)) // 调整虚线长度
            .set("transform", format!("rotate(-90 {gauge_x} {gauge_y})"));

        document = document.add(gauge_arc);

        // Gauge value text
        let value_text = if *unit == "B" && *value > 1024.0 {
            format!("{:.1}K", value / 1024.0)
        } else {
            format!("{value:.1}{unit}")
        };

        let text = SvgText::new(value_text)
            .set("x", gauge_x)
            .set("y", gauge_y + 5)
            .set("text-anchor", "middle")
            .set("font-size", 16) // 增大数值字体
            .set("font-weight", "bold")
            .set("fill", *color);

        document = document.add(text);

        // Gauge label - 调整位置适应新的饼图大小
        let label_text = SvgText::new(*label)
            .set("x", gauge_x)
            .set("y", gauge_y + 85) // 向下移动适应更大的饼图
            .set("text-anchor", "middle")
            .set("font-size", 12) // 增大字体
            .set("font-weight", "600")
            .set("fill", "#2c3e50");

        document = document.add(label_text);
    }

    Ok(document)
}

/// Add memory heatmap visualization
pub fn add_memory_heatmap(
    mut document: Document,
    allocations: &[AllocationInfo],
) -> TrackingResult<Document> {
    let heatmap_x = 50;
    let heatmap_y = 420;
    let heatmap_width = 1700; // 适应新的1800px宽度
    let heatmap_height = 200;

    // Heatmap background
    let bg = Rectangle::new()
        .set("x", heatmap_x)
        .set("y", heatmap_y)
        .set("width", heatmap_width)
        .set("height", heatmap_height)
        .set("fill", "white")
        .set("stroke", "#e74c3c")
        .set("stroke-width", 2)
        .set("rx", 10);

    document = document.add(bg);

    // Heatmap title
    let title = SvgText::new("Memory Allocation Heatmap")
        .set("x", heatmap_x + heatmap_width / 2)
        .set("y", heatmap_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 18)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");

    document = document.add(title);

    if allocations.is_empty() {
        let no_data = SvgText::new("No allocation data available")
            .set("x", heatmap_x + heatmap_width / 2)
            .set("y", heatmap_y + heatmap_height / 2)
            .set("text-anchor", "middle")
            .set("font-size", 14)
            .set("fill", "#7f8c8d");

        document = document.add(no_data);
        return Ok(document);
    }

    // Create heatmap grid - 动态调整网格大小以适应不同项目规模
    let allocation_count = allocations.len();
    let (grid_cols, grid_rows) = if allocation_count > 1000 {
        // 大型项目：更细粒度的网格
        (25, 10)
    } else if allocation_count > 100 {
        // 中型项目：标准网格
        (20, 8)
    } else {
        // 小型项目：较小网格
        (15, 6)
    };
    
    let cell_width = (heatmap_width - 40) / grid_cols;
    let cell_height = (heatmap_height - 40) / grid_rows;

    // Calculate allocation density per cell
    let mut density_grid = vec![vec![0; grid_cols]; grid_rows];
    let max_size = allocations.iter().map(|a| a.size).max().unwrap_or(1);

    for allocation in allocations {
        // Map allocation to grid position based on size and timestamp
        let size_ratio = allocation.size as f64 / max_size as f64;
        let time_ratio = (allocation.timestamp_alloc % 1000) as f64 / 1000.0;

        let col = ((size_ratio * (grid_cols - 1) as f64) as usize).min(grid_cols - 1);
        let row = ((time_ratio * (grid_rows - 1) as f64) as usize).min(grid_rows - 1);

        density_grid[row][col] += 1;
    }

    // Find max density for color scaling
    let max_density = density_grid
        .iter()
        .flat_map(|row| row.iter())
        .max()
        .copied()
        .unwrap_or(1);

    // Draw heatmap cells
    for (row, row_data) in density_grid.iter().enumerate() {
        for (col, &density) in row_data.iter().enumerate() {
            let x = heatmap_x + 20 + col * cell_width;
            let y = heatmap_y + 20 + row * cell_height;

            // Calculate color intensity based on density
            let intensity = if max_density > 0 {
                density as f64 / max_density as f64
            } else {
                0.0
            };

            let color = if intensity == 0.0 {
                "#f8f9fa".to_string()
            } else {
                // Heat colors from blue (cold) to red (hot)
                let red = (255.0 * intensity) as u8;
                let blue = (255.0 * (1.0 - intensity)) as u8;
                format!("rgb({red}, 100, {blue})")
            };

            let cell = Rectangle::new()
                .set("x", x)
                .set("y", y)
                .set("width", cell_width - 1)
                .set("height", cell_height - 1)
                .set("fill", color)
                .set("stroke", "#bdc3c7")
                .set("stroke-width", 0.5)
                .set("class", "heatmap-cell");

            document = document.add(cell);

            // Add density text for non-zero cells
            if density > 0 {
                let density_text = SvgText::new(density.to_string())
                    .set("x", x + cell_width / 2)
                    .set("y", y + cell_height / 2 + 3)
                    .set("text-anchor", "middle")
                    .set("font-size", 8)
                    .set("fill", if intensity > 0.5 { "white" } else { "black" });

                document = document.add(density_text);
            }
        }
    }

    // Add heatmap legend
    let legend_y = heatmap_y + heatmap_height - 15;
    let legend_text = SvgText::new("Size →")
        .set("x", heatmap_x + 20)
        .set("y", legend_y)
        .set("font-size", 10)
        .set("fill", "#7f8c8d");
    document = document.add(legend_text);

    let legend_text2 = SvgText::new("↑ Time")
        .set("x", heatmap_x + 10)
        .set("y", heatmap_y + 40)
        .set("font-size", 10)
        .set("fill", "#7f8c8d")
        .set(
            "transform",
            format!("rotate(-90 {} {})", heatmap_x + 10, heatmap_y + 40),
        );
    document = document.add(legend_text2);

    Ok(document)
}
