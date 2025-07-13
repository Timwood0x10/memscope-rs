//! Enhanced export functionality for memory tracking data.

use crate::tracker::MemoryTracker;
use crate::types::{AllocationInfo, MemoryStats, TrackingResult, TypeMemoryUsage};
use crate::utils::{format_bytes, get_category_color, simplify_type_name};

/// Calculate real median and P95 percentiles from allocation sizes
/// Returns (median_size, p95_size)
fn calculate_allocation_percentiles(allocations: &[AllocationInfo]) -> (usize, usize) {
    if allocations.is_empty() {
        return (0, 0);
    }

    // Collect all allocation sizes
    let mut sizes: Vec<usize> = allocations.iter().map(|a| a.size).collect();
    sizes.sort_unstable();

    let len = sizes.len();
    
    // Calculate median (50th percentile)
    let median = if len % 2 == 0 {
        (sizes[len / 2 - 1] + sizes[len / 2]) / 2
    } else {
        sizes[len / 2]
    };

    // Calculate P95 (95th percentile)
    let p95_index = ((len as f64) * 0.95) as usize;
    let p95 = if p95_index >= len {
        sizes[len - 1]
    } else {
        sizes[p95_index]
    };

    (median, p95)
}
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use svg::node::element::{Circle, Rectangle, Text as SvgText};
use svg::Document;

/// Enhanced type information processing with variable names
pub fn enhance_type_information(
    memory_by_type: &[TypeMemoryUsage],
    allocations: &[AllocationInfo],
) -> Vec<EnhancedTypeInfo> {
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

/// Enhanced type information with variable names and categorization
#[derive(Debug, Clone)]
pub struct EnhancedTypeInfo {
    /// Simplified type name for display
    pub simplified_name: String,
    /// Category this type belongs to
    pub category: String,
    /// Total memory size used by this type
    pub total_size: usize,
    /// Number of allocations of this type
    pub allocation_count: usize,
    /// Variable names associated with this type
    pub variable_names: Vec<String>,
}

/// Allocation category for grouping related allocations
#[derive(Debug, Clone)]
pub struct AllocationCategory {
    /// Category name
    pub name: String,
    /// Allocations in this category
    pub allocations: Vec<AllocationInfo>,
    /// Total size of all allocations in this category
    pub total_size: usize,
    /// Color used for visualization
    pub color: String,
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
        let percent_label = SvgText::new(format!("{percentage}%"))
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

/// Add enhanced header with 8 core metrics - Modern Dashboard Style
pub fn add_enhanced_header(
    mut document: Document,
    stats: &MemoryStats,
    allocations: &[AllocationInfo],
) -> TrackingResult<Document> {
    // Main title with modern styling
    let title = SvgText::new("Rust Memory Usage Analysis")
        .set("x", 900)
        .set("y", 40)
        .set("text-anchor", "middle")
        .set("font-size", 28)
        .set("font-weight", "300")
        .set("fill", "#2c3e50")
        .set("style", "letter-spacing: 1px;");

    document = document.add(title);

    // Calculate 8 core metrics with percentage values for progress rings
    let active_memory = stats.active_memory;
    let peak_memory = stats.peak_memory;
    let active_allocations = stats.active_allocations;
    
    let memory_reclamation_rate = if stats.total_allocated > 0 {
        (stats.total_deallocated as f64 / stats.total_allocated as f64) * 100.0
    } else {
        0.0
    };

    let allocator_efficiency = if stats.peak_memory > 0 {
        (stats.active_memory as f64 / stats.peak_memory as f64) * 100.0
    } else {
        0.0
    };
    
    let (median_alloc_size, p95_alloc_size) = calculate_allocation_percentiles(allocations);
    
    let memory_fragmentation = if stats.peak_memory > 0 {
        ((stats.peak_memory - stats.active_memory) as f64 / stats.peak_memory as f64) * 100.0
    } else {
        0.0
    };

    // Define metrics with their display values and progress percentages
    let metrics = [
        ("Active Memory", format_bytes(active_memory), 
         (active_memory as f64 / peak_memory.max(1) as f64 * 100.0).min(100.0), "#3498db"),
        ("Peak Memory", format_bytes(peak_memory), 100.0, "#e74c3c"),
        ("Active Allocs", format!("{}", active_allocations), 
         (active_allocations as f64 / 1000.0 * 100.0).min(100.0), "#2ecc71"),
        ("Reclamation", format!("{:.1}%", memory_reclamation_rate), 
         memory_reclamation_rate, "#f39c12"),
        ("Efficiency", format!("{:.1}%", allocator_efficiency), 
         allocator_efficiency, "#9b59b6"),
        ("Median Size", format_bytes(median_alloc_size), 
         (median_alloc_size as f64 / 1024.0 * 100.0).min(100.0), "#1abc9c"),
        ("P95 Size", format_bytes(p95_alloc_size), 
         (p95_alloc_size as f64 / 4096.0 * 100.0).min(100.0), "#e67e22"),
        ("Fragmentation", format!("{:.1}%", memory_fragmentation), 
         memory_fragmentation, "#95a5a6"),
    ];

    // Single row layout parameters
    let card_width = 200;
    let card_height = 120;
    let start_x = 50;
    let start_y = 130;
    let spacing_x = 220;

    // Add single section header for all metrics
    let header = SvgText::new("KEY PERFORMANCE METRICS")
        .set("x", 900)
        .set("y", start_y - 20)
        .set("text-anchor", "middle")
        .set("font-size", 11)
        .set("font-weight", "600")
        .set("fill", "#7f8c8d")
        .set("style", "letter-spacing: 2px;");
    document = document.add(header);

    // Render all 8 metrics in a single row
    for (i, (title, value, percentage, color)) in metrics.iter().enumerate() {
        let x = start_x + i * spacing_x;
        let y = start_y;

        // Modern card background with gradient and shadow
        let card_bg = Rectangle::new()
            .set("x", x)
            .set("y", y)
            .set("width", card_width)
            .set("height", card_height)
            .set("fill", "#ffffff")
            .set("stroke", "none")
            .set("rx", 12)
            .set("style", "filter: drop-shadow(0 4px 12px rgba(0,0,0,0.15));");

        // Use simple solid color instead of gradient to avoid SVG compatibility issues
        if i == 0 {
            // Skip gradient definition for now - use solid colors
        }

        document = document.add(card_bg);

        // Progress ring background
        let ring_center_x = x + 40;
        let ring_center_y = y + 60;
        let ring_radius = 25;
        
        let ring_bg = Circle::new()
            .set("cx", ring_center_x)
            .set("cy", ring_center_y)
            .set("r", ring_radius)
            .set("fill", "none")
            .set("stroke", "#ecf0f1")
            .set("stroke-width", 6);
        document = document.add(ring_bg);

        // Progress ring foreground
        let circumference = 2.0 * std::f64::consts::PI * ring_radius as f64;
        let progress_offset = circumference * (1.0 - percentage / 100.0);
        
        let progress_ring = Circle::new()
            .set("cx", ring_center_x)
            .set("cy", ring_center_y)
            .set("r", ring_radius)
            .set("fill", "none")
            .set("stroke", *color)
            .set("stroke-width", 6)
            .set("stroke-linecap", "round")
            .set("stroke-dasharray", format!("{} {}", circumference, circumference))
            .set("stroke-dashoffset", progress_offset)
            .set("transform", format!("rotate(-90 {} {})", ring_center_x, ring_center_y))
            .set("style", "transition: stroke-dashoffset 0.5s ease;");
        document = document.add(progress_ring);

        // Percentage text in center of ring
        let percent_text = SvgText::new(format!("{:.0}%", percentage))
            .set("x", ring_center_x)
            .set("y", ring_center_y + 4)
            .set("text-anchor", "middle")
            .set("font-size", 12)
            .set("font-weight", "bold")
            .set("fill", *color);
        document = document.add(percent_text);

        // Metric title
        let title_text = SvgText::new(*title)
            .set("x", x + 90)
            .set("y", y + 35)
            .set("font-size", 12)
            .set("font-weight", "600")
            .set("fill", "#2c3e50");
        document = document.add(title_text);

        // Metric value with larger, prominent display
        let value_text = SvgText::new(value)
            .set("x", x + 90)
            .set("y", y + 55)
            .set("font-size", 16)
            .set("font-weight", "bold")
            .set("fill", "#2c3e50");
        document = document.add(value_text);

        // Status indicator based on percentage
        let status_color = if *percentage >= 80.0 {
            "#e74c3c"  // Red for high values
        } else if *percentage >= 50.0 {
            "#f39c12"  // Orange for medium values
        } else {
            "#27ae60"  // Green for low values
        };

        let status_dot = Circle::new()
            .set("cx", x + 90)
            .set("cy", y + 75)
            .set("r", 4)
            .set("fill", status_color);
        document = document.add(status_dot);

        // Status text
        let status_text = if *percentage >= 80.0 {
            "HIGH"
        } else if *percentage >= 50.0 {
            "MEDIUM"
        } else {
            "OPTIMAL"
        };

        let status_label = SvgText::new(status_text)
            .set("x", x + 105)
            .set("y", y + 79)
            .set("font-size", 9)
            .set("font-weight", "600")
            .set("fill", status_color);
        document = document.add(status_label);
    }

    Ok(document)
}

/// Add enhanced type chart with categories
pub fn add_enhanced_type_chart(
    mut document: Document,
    types: &[EnhancedTypeInfo],
) -> TrackingResult<Document> {
    let chart_x = 50;
    let chart_y = 730;
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
    let chart_y = 1080;
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

        // Enhanced variable names display - optimize text overflow issues
        let var_names: Vec<String> = category
            .allocations
            .iter()
            .filter_map(|a| {
                if let Some(var_name) = &a.var_name {
                    let type_name = a.type_name.as_deref().unwrap_or("Unknown");
                    let (simplified_type, _) = simplify_type_name(type_name);
                    // Shorten variable name display to avoid overflow
                    let short_var = if var_name.len() > 12 {
                        format!("{}...", &var_name[..9])
                    } else {
                        var_name.clone()
                    };
                    Some(format!("{short_var}({simplified_type})"))
                } else {
                    None
                }
            })
            .take(3) // Reduce number of displayed variables to avoid overflow
            .collect();

        let mut display_text = if var_names.is_empty() {
            format!(
                "{} ({} vars)",
                format_bytes(category.total_size),
                category.allocations.len()
            )
        } else {
            format!(
                "{} - Vars: {}",
                format_bytes(category.total_size),
                var_names.join(", ")
            )
        };

        // Dynamically truncate text to ensure it doesn't exceed chart boundaries
        let max_text_width = chart_width - 180; // Reserve margin
        let estimated_char_width = 7; // Estimate character width
        let max_chars = (max_text_width / estimated_char_width) as usize;

        if display_text.len() > max_chars {
            display_text = format!("{}...", &display_text[..max_chars.saturating_sub(3)]);
        }

        let size_text = SvgText::new(display_text)
            .set("x", chart_x + 160)
            .set("y", y + bar_height / 2 + 4)
            .set("font-size", 11) // Slightly reduce font size to avoid overflow
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
    let chart_y = 1780;
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
            let mut label_text = format!(
                "{}({}) memory: {}",
                var_name,
                simplified_type,
                format_bytes(allocation.size)
            );

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
    let chart_y = 710; // Fix: adjust position to avoid overlap
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
    let chart_y = 1060; // Fix: adjust position to avoid overlap
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
            // Untracked allocations with known type - categorize by type and source
            let (simplified_type, _) = simplify_type_name(type_name);

            // Try to identify the source of untracked allocations
            if type_name.contains("std::") || type_name.contains("alloc::") {
                format!("System/Runtime {simplified_type} (untracked)")
            } else if simplified_type.contains("Vec") {
                "Internal Vec allocations (untracked)".to_string()
            } else if simplified_type.contains("String") {
                "Internal String allocations (untracked)".to_string()
            } else if simplified_type.contains("HashMap") {
                "Internal HashMap allocations (untracked)".to_string()
            } else {
                format!("Internal {simplified_type} allocations (untracked)")
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
    let chart_y = 1430;
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
    let legend_y = 2130;
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
    let summary_y = 2130;
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

/// Add memory allocation timeline as categorized scatter plot
pub fn add_performance_dashboard(
    mut document: Document,
    _stats: &MemoryStats,
    allocations: &[AllocationInfo],
) -> TrackingResult<Document> {
    let chart_x = 50;
    let chart_y = 300; // Move down to avoid overlapping with header section
    let chart_width = 1700;
    let chart_height = 350; // Reduce height to avoid being blocked by modules below

    // Chart background
    let bg = Rectangle::new()
        .set("x", chart_x)
        .set("y", chart_y)
        .set("width", chart_width)
        .set("height", chart_height)
        .set("fill", "white")
        .set("stroke", "#bdc3c7")
        .set("stroke-width", 1)
        .set("rx", 8);

    document = document.add(bg);

    // Chart title
    let title = SvgText::new("Memory Allocation Timeline")
        .set("x", chart_x + chart_width / 2)
        .set("y", chart_y - 15)
        .set("text-anchor", "middle")
        .set("font-size", 16)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");

    document = document.add(title);

    // Calculate time and size ranges for scaling
    let max_time = allocations.iter()
        .map(|a| a.timestamp_alloc)
        .max()
        .unwrap_or(1000) as f64;
    let min_time = allocations.iter()
        .map(|a| a.timestamp_alloc)
        .min()
        .unwrap_or(0) as f64;
    let time_range = max_time - min_time;

    let max_size = allocations.iter().map(|a| a.size).max().unwrap_or(1024) as f64;
    let min_size = allocations.iter().map(|a| a.size).filter(|&s| s > 0).min().unwrap_or(1) as f64;

    // Plot area dimensions - reserve space for right-side legend to prevent overflow
    let plot_x = chart_x + 80;
    let plot_y = chart_y + 30;
    let plot_width = chart_width - 250; // Increase right margin to leave space for legend
    let plot_height = chart_height - 80;

    // X-axis (Time)
    let x_axis = svg::node::element::Line::new()
        .set("x1", plot_x)
        .set("y1", plot_y + plot_height)
        .set("x2", plot_x + plot_width)
        .set("y2", plot_y + plot_height)
        .set("stroke", "#34495e")
        .set("stroke-width", 2);
    document = document.add(x_axis);

    // Y-axis (Size - Logarithmic)
    let y_axis = svg::node::element::Line::new()
        .set("x1", plot_x)
        .set("y1", plot_y)
        .set("x2", plot_x)
        .set("y2", plot_y + plot_height)
        .set("stroke", "#34495e")
        .set("stroke-width", 2);
    document = document.add(y_axis);

    // X-axis label - clearly indicate horizontal axis meaning
    let x_label = SvgText::new("Execution Time (milliseconds)")
        .set("x", plot_x + plot_width / 2)
        .set("y", plot_y + plot_height + 40)
        .set("text-anchor", "middle")
        .set("font-size", 12)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");
    document = document.add(x_label);

    // Y-axis label - clearly indicate vertical axis meaning
    let y_label = SvgText::new("Memory Allocation Size (Bytes)")
        .set("x", plot_x - 60)
        .set("y", plot_y + plot_height / 2)
        .set("text-anchor", "middle")
        .set("font-size", 12)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50")
        .set("transform", format!("rotate(-90 {} {})", plot_x - 60, plot_y + plot_height / 2));
    document = document.add(y_label);

    // Add Y-axis scale markers for better readability
    let scale_markers = [
        (16, "16B"),
        (256, "256B"), 
        (1024, "1KB"),
        (4096, "4KB"),
        (16384, "16KB"),
    ];
    
    for (size, label) in scale_markers.iter() {
        if *size >= min_size as usize && *size <= max_size as usize {
            let log_size = (*size as f64).ln();
            let log_min = min_size.ln();
            let log_max = max_size.ln();
            let log_range = log_max - log_min;
            
            if log_range > 0.0 {
                let y_pos = plot_y + plot_height - ((log_size - log_min) / log_range * plot_height as f64) as i32;
                
                // Scale marker line
                let marker_line = svg::node::element::Line::new()
                    .set("x1", plot_x - 5)
                    .set("y1", y_pos)
                    .set("x2", plot_x + 5)
                    .set("y2", y_pos)
                    .set("stroke", "#7f8c8d")
                    .set("stroke-width", 1);
                document = document.add(marker_line);
                
                // Scale marker label
                let marker_label = SvgText::new(*label)
                    .set("x", plot_x - 15)
                    .set("y", y_pos + 3)
                    .set("text-anchor", "end")
                    .set("font-size", 10)
                    .set("fill", "#7f8c8d");
                document = document.add(marker_label);
            }
        }
    }

    // Add X-axis time scale markers - allow users to easily see time data
    let time_markers = 5;
    for i in 0..=time_markers {
        let time_point = min_time + (time_range * i as f64 / time_markers as f64);
        let x_pos = plot_x + (i * plot_width / time_markers);
        
        // Time marker line
        let time_marker_line = svg::node::element::Line::new()
            .set("x1", x_pos)
            .set("y1", plot_y + plot_height - 5)
            .set("x2", x_pos)
            .set("y2", plot_y + plot_height + 5)
            .set("stroke", "#7f8c8d")
            .set("stroke-width", 1);
        document = document.add(time_marker_line);
        
        // Time marker label
        let time_label = SvgText::new(format!("{:.0}ms", time_point))
            .set("x", x_pos)
            .set("y", plot_y + plot_height + 20)
            .set("text-anchor", "middle")
            .set("font-size", 9)
            .set("fill", "#7f8c8d");
        document = document.add(time_label);
    }

    // Categorize allocations and get colors
    let categorized = categorize_allocations(allocations);
    let mut category_colors: HashMap<String, String> = HashMap::new();
    for category in &categorized {
        category_colors.insert(category.name.clone(), category.color.clone());
    }

    // Calculate P95 threshold for larger dots
    let mut sizes: Vec<usize> = allocations.iter().map(|a| a.size).filter(|&s| s > 0).collect();
    sizes.sort_unstable();
    let p95_threshold = if !sizes.is_empty() {
        let p95_index = (sizes.len() as f64 * 0.95) as usize;
        sizes[p95_index.min(sizes.len() - 1)]
    } else {
        0
    };

    // Plot data points with ENHANCED VISIBILITY
    for allocation in allocations.iter().take(500) { // Increased from 200 to 500 points
        if allocation.size > 0 {
            // Calculate position
            let timestamp = allocation.timestamp_alloc;
            let x_pos = if time_range > 0.0 {
                plot_x + ((timestamp as f64 - min_time) / time_range * plot_width as f64) as i32
            } else {
                plot_x + plot_width / 2
            };

            // Logarithmic Y scaling
            let log_size = (allocation.size as f64).ln();
            let log_min = min_size.ln();
            let log_max = max_size.ln();
            let log_range = log_max - log_min;
            
            let y_pos = if log_range > 0.0 {
                plot_y + plot_height - ((log_size - log_min) / log_range * plot_height as f64) as i32
            } else {
                plot_y + plot_height / 2
            };

            // Get category color - USE SAME LOGIC AS LEGEND
            let color = if let Some(type_name) = &allocation.type_name {
                // Use the same categorization logic as the legend
                let (_, category) = simplify_type_name(type_name);
                get_category_color(&category)
            } else {
                "#95a5a6".to_string() // Gray for unknown types
            };

            // LARGER DOTS with P95+ emphasis
            let radius = if allocation.size >= p95_threshold {
                8 // P95+ allocations get 8px radius
            } else {
                6  // Regular allocations get 6px radius (doubled from 3px)
            };

            // Draw point with enhanced visibility
            let point = Circle::new()
                .set("cx", x_pos)
                .set("cy", y_pos)
                .set("r", radius)
                .set("fill", color)
                .set("stroke", "#2c3e50")
                .set("stroke-width", 1)
                .set("opacity", 0.8); // Increased opacity

            document = document.add(point);
        }
    }

    // Optimized legend - prevent right border overflow
    let legend_x = plot_x + plot_width + 20;
    let legend_y = plot_y + 20;
    let legend_width = 140; // Limit legend width
    
    // Legend background box to prevent content overflow
    let legend_bg = Rectangle::new()
        .set("x", legend_x - 10)
        .set("y", legend_y - 15)
        .set("width", legend_width)
        .set("height", 200)
        .set("fill", "rgba(255,255,255,0.9)")
        .set("stroke", "#bdc3c7")
        .set("stroke-width", 1)
        .set("rx", 5);
    document = document.add(legend_bg);
    
    let legend_title = SvgText::new("Type Categories")
        .set("x", legend_x)
        .set("y", legend_y)
        .set("font-size", 12)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");
    document = document.add(legend_title);

    // Add gray dot explanation
    // Count Unknown Type allocations for more precise information
    let unknown_count = allocations.iter()
        .filter(|a| a.type_name.as_ref().map_or(true, |t| t == "Unknown" || t.is_empty()))
        .count();
    
    let unknown_legend_y = legend_y + 20;
    
    // Unknown Type color square - following legend style
    let unknown_color_square = Rectangle::new()
        .set("x", legend_x)
        .set("y", unknown_legend_y - 8)
        .set("width", 10)
        .set("height", 10)
        .set("fill", "#95a5a6");
    document = document.add(unknown_color_square);
    
    // More precise Unknown Type label showing count and possible causes
    let unknown_label = if unknown_count > 0 {
        format!("Unknown ({} allocs)", unknown_count)
    } else {
        "No Unknown Types".to_string()
    };
    
    let unknown_text = SvgText::new(unknown_label)
        .set("x", legend_x + 15)
        .set("y", unknown_legend_y - 1)
        .set("font-size", 9)
        .set("fill", "#2c3e50");
    document = document.add(unknown_text);

    for (i, category) in categorized.iter().take(6).enumerate() {
        let legend_item_y = legend_y + 40 + (i as i32) * 18;
        
        // Color square
        let color_square = Rectangle::new()
            .set("x", legend_x)
            .set("y", legend_item_y - 8)
            .set("width", 10)
            .set("height", 10)
            .set("fill", category.color.as_str());
        document = document.add(color_square);

        // Category name - truncate long names to avoid overflow
        let category_name = if category.name.len() > 12 {
            format!("{}...", &category.name[..9])
        } else {
            category.name.clone()
        };
        
        let category_text = SvgText::new(category_name)
            .set("x", legend_x + 15)
            .set("y", legend_item_y - 1)
            .set("font-size", 9)
            .set("fill", "#2c3e50");
        document = document.add(category_text);
    }

    Ok(document)
}

/// Add memory heatmap visualization (placeholder for now)
pub fn add_memory_heatmap(
    document: Document,
    _allocations: &[AllocationInfo],
) -> TrackingResult<Document> {
    // For now, just return the document as-is
    // This will be replaced with actual heatmap implementation later
    Ok(document)
}
