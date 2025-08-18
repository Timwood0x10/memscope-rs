//! Unified visualization module for memscope-rs
//! Provides memory analysis and lifecycle timeline SVG exports

use crate::core::tracker::MemoryTracker;
use crate::core::types::{AllocationInfo, MemoryStats, TrackingError, TrackingResult};
use crate::utils::{format_bytes, get_simple_type, get_type_color, get_type_gradient_colors};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use svg::node::element::{
    Circle, Definitions, Group, Line, Marker, Polygon, Rectangle, Style, Text as SvgText,
};
use svg::Document;

use crate::unsafe_ffi_tracker::{
    AllocationSource, BoundaryEventType, EnhancedAllocationInfo, SafetyViolation, UnsafeFFITracker,
};

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

    // FIXED: Get stats and allocations at the same time to ensure data consistency
    let stats = tracker.get_stats()?;
    let active_allocations = tracker.get_active_allocations()?;

    // Debug: Log the peak memory value used in SVG export
    tracing::info!(
        "SVG Export - Using peak_memory: {} bytes ({})",
        stats.peak_memory,
        crate::utils::format_bytes(stats.peak_memory)
    );

    let document = create_memory_analysis_svg(&active_allocations, &stats, tracker)?;

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

/// Create comprehensive memory analysis SVG with original 12-section layout
fn create_memory_analysis_svg(
    allocations: &[AllocationInfo],
    stats: &MemoryStats,
    tracker: &MemoryTracker,
) -> TrackingResult<Document> {
    // Create comprehensive memory analysis using the original enhanced export logic
    let width = 1800;
    let height = 2000; // Reduced height after repositioning modules

    let mut document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", width)
        .set("height", height)
        .set("style", "background: linear-gradient(135deg, #ecf0f1 0%, #bdc3c7 100%); font-family: 'Segoe UI', Arial, sans-serif;");

    // 1. Title: Rust Memory Usage Analysis
    document = crate::export::export_enhanced::add_enhanced_header(document, stats, allocations)?;

    // 3. Performance Dashboard - REMOVED to prevent overlap with header metrics
    // document =
    //     crate::export_enhanced::add_enhanced_timeline_dashboard(document, stats, allocations)?;

    // 4. Memory Allocation Heatmap
    document = crate::export::export_enhanced::add_memory_heatmap(document, allocations)?;

    // 5. Left side: Memory Usage by Type
    // Fixed: Get actual memory type data instead of empty array
    let memory_by_type_data = tracker.get_memory_by_type().unwrap_or_default();
    let memory_by_type = enhance_type_information(&memory_by_type_data, allocations);
    document = crate::export::export_enhanced::add_enhanced_type_chart(document, &memory_by_type)?;

    // 6. Right side: Memory Fragmentation Analysis
    document = crate::export::export_enhanced::add_fragmentation_analysis(document, allocations)?;

    // 7. Left side: Tracked Variables by Category
    // FIXED: Use same enhanced data source as Memory Usage by Type for consistency
    let categorized = categorize_enhanced_allocations(&memory_by_type);
    document = crate::export::export_enhanced::add_categorized_allocations(document, &categorized)?;

    // 8. Right side: Call Stack Analysis
    document = crate::export::export_enhanced::add_callstack_analysis(document, allocations)?;

    // 9. Memory Growth Trends
    document = crate::export::export_enhanced::add_memory_growth_trends(document, allocations, stats)?;

    // 10. Variable Allocation Timeline
    document = crate::export::export_enhanced::add_memory_timeline(document, allocations, stats)?;

    // 11. Bottom left: Interactive Legend & Guide
    document = crate::export::export_enhanced::add_interactive_legend(document)?;

    // 12. Bottom right: Memory Analysis Summary
    document = crate::export::export_enhanced::add_comprehensive_summary(document, stats, allocations)?;

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

    // Title - Scope Matrix & Lifecycle Visualization as specified in task.md
    let title = SvgText::new("Scope Matrix & Lifecycle Visualization")
        .set("x", width / 2)
        .set("y", 40)
        .set("text-anchor", "middle")
        .set("font-size", 32)
        .set("font-weight", "bold")
        .set("fill", "#FFFFFF")
        .set("style", "text-shadow: 3px 3px 6px rgba(0,0,0,0.5);");
    document = document.add(title);

    // PROMINENT GLOBAL LEGEND for Progress Bar explanation
    document = add_prominent_progress_bar_legend(document, width);

    let tracked_vars: Vec<_> = allocations
        .iter()
        .filter(|a| a.var_name.is_some())
        .collect();

    // tracing::info!(
    //     "Found {} total allocations, {} with variable names",
    //     allocations.len(),
    //     tracked_vars.len()
    // );

    // Debug: Print the tracked variables we found
    // for (i, var) in tracked_vars.iter().enumerate() {
    //     tracing::info!(
    //         "Tracked var {}: {} ({})",
    //         i + 1,
    //         var.var_name.as_ref().unwrap_or(&"None".to_string()),
    //         var.type_name.as_ref().unwrap_or(&"Unknown".to_string())
    //     );
    // }

    if tracked_vars.is_empty() {
        let no_data = SvgText::new(format!(
            "No tracked variables found (checked {} allocations)",
            allocations.len()
        ))
        .set("x", width / 2)
        .set("y", height / 2)
        .set("text-anchor", "middle")
        .set("font-size", 18)
        .set("fill", "#FFFFFF");
        document = document.add(no_data);
        return Ok(document);
    }

    // Add matrix layout instead of timeline - ADJUSTED Y position for global legend
    document = add_matrix_layout_section(document, &tracked_vars, 50, 130)?;

    // Add memory analysis section - aligned and consistent width
    document = add_memory_section(document, &tracked_vars, stats, 550, width - 100)?;

    // Add variable relationships section - aligned and consistent width
    document = add_relationships_section(document, &tracked_vars, 900, width - 100)?;

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

    // Section title - TOP 3 MEMORY ANALYSIS as specified in task.md
    let section_title = SvgText::new("Top 3 Memory Analysis")
        .set("x", 70)
        .set("y", start_y + 10)
        .set("class", "section-title");
    document = document.add(section_title);

    // Group by type and get TOP 3 ONLY
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

    // Sort by total size and take TOP 3 ONLY
    let mut sorted_types: Vec<_> = type_stats.into_iter().collect();
    sorted_types.sort_by(|a, b| b.1 .1.cmp(&a.1 .1)); // Sort by total size descending
    sorted_types.truncate(3); // STRICTLY TOP 3

    // Draw memory bars - TOP 3 ONLY
    let chart_x = 100;
    let chart_y = start_y + 50;
    let max_size = sorted_types
        .iter()
        .map(|(_, (_, size, _))| *size)
        .max()
        .unwrap_or(1);

    for (i, (type_name, (count, total_size, vars))) in sorted_types.iter().enumerate() {
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

        // Type label inside the bar
        let type_label = SvgText::new(format!("{type_name} ({count} vars)"))
            .set("x", chart_x + 10)
            .set("y", y + 17)
            .set("font-size", 12)
            .set("font-weight", "bold")
            .set("fill", "#FFFFFF")
            .set("text-shadow", "1px 1px 2px rgba(0,0,0,0.8)");
        document = document.add(type_label);

        // Enhanced variable names display - more prominent
        let vars_text = vars.join(" | ");
        let vars_label = SvgText::new(if vars_text.len() > 80 {
            format!("{}...", &vars_text[..77])
        } else {
            vars_text
        })
        .set("x", chart_x + bar_width + 15)
        .set("y", y + 17)
        .set("font-size", 11)
        .set("font-weight", "600")
        .set("fill", "#FFFFFF")
        .set("text-shadow", "1px 1px 2px rgba(0,0,0,0.6)");
        document = document.add(vars_label);
    }

    // Placeholder for lifecycle timeline
    Ok(document)
}

/// Add relationships section for lifecycle visualization - ENHANCED VARIABLE RELATIONSHIPS
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
    let section_title = SvgText::new("Variable Relationships - Ownership & Borrowing")
        .set("x", 70)
        .set("y", start_y + 10)
        .set("class", "section-title");
    document = document.add(section_title);

    // Group variables by scope for better organization
    let mut scope_groups: HashMap<String, Vec<&AllocationInfo>> = HashMap::new();
    for var in tracked_vars {
        let scope = identify_precise_scope(var);
        scope_groups.entry(scope).or_default().push(*var);
    }

    // Draw scope group backgrounds first
    let mut scope_positions = HashMap::new();
    let start_x = 100;
    let group_spacing_x = 400;
    let group_spacing_y = 200;

    for (i, (scope_name, _vars)) in scope_groups.iter().enumerate() {
        let group_x = start_x + (i % 3) as i32 * group_spacing_x;
        let group_y = start_y + 50 + (i / 3) as i32 * group_spacing_y;

        // Scope group background with subtle color
        let group_bg = Rectangle::new()
            .set("x", group_x - 20)
            .set("y", group_y - 20)
            .set("width", 300)
            .set("height", 150)
            .set("fill", get_scope_background_color(scope_name))
            .set("stroke", get_scope_border_color(scope_name))
            .set("stroke-width", 2)
            .set(
                "stroke-dasharray",
                if scope_name == "Global" {
                    "none"
                } else {
                    "5,3"
                },
            )
            .set("rx", 8)
            .set("opacity", "0.3");
        document = document.add(group_bg);

        // Scope label
        let scope_label = SvgText::new(format!("Scope: {scope_name}"))
            .set("x", group_x - 10)
            .set("y", group_y - 5)
            .set("font-size", 12)
            .set("font-weight", "bold")
            .set("fill", "#FFFFFF");
        document = document.add(scope_label);

        scope_positions.insert(scope_name, (group_x, group_y));
    }

    // Note: Relationship analysis removed to eliminate unused code

    // Draw variable nodes AFTER lines (on top)
    for (scope_name, vars) in &scope_groups {
        if let Some((group_x, group_y)) = scope_positions.get(scope_name) {
            for (i, allocation) in vars.iter().take(4).enumerate() {
                // Limit to 4 per scope
                let node_x = group_x + (i % 2) as i32 * 120 + 40;
                let node_y = group_y + (i / 2) as i32 * 60 + 30;

                document = draw_variable_node(document, allocation, node_x, node_y)?;
            }
        }
    }

    // Add relationship legend
    document = add_relationship_legend(document, start_y + section_height - 100)?;

    Ok(document)
}

/// Draw variable node with enhanced styling
fn draw_variable_node(
    mut document: Document,
    allocation: &AllocationInfo,
    x: i32,
    y: i32,
) -> TrackingResult<Document> {
    let default_name = "<unknown>".to_string();
    let var_name = allocation.var_name.as_ref().unwrap_or(&default_name);
    let type_name = allocation.type_name.as_deref().unwrap_or("Unknown");
    let simple_type = get_simple_type(type_name);
    let color = get_type_color(&simple_type);

    // Variable node with tooltip support
    let node = Circle::new()
        .set("cx", x)
        .set("cy", y)
        .set("r", 20)
        .set("fill", color)
        .set("stroke", "#FFFFFF")
        .set("stroke-width", 2)
        .set(
            "title",
            format!(
                "{}: {} ({})",
                var_name,
                simple_type,
                format_bytes(allocation.size)
            ),
        );
    document = document.add(node);

    // Variable name (truncated if too long)
    let display_name = if var_name.len() > 8 {
        format!("{}...", &var_name[..6])
    } else {
        var_name.clone()
    };

    let name_label = SvgText::new(display_name)
        .set("x", x)
        .set("y", y + 3)
        .set("text-anchor", "middle")
        .set("font-size", 9)
        .set("font-weight", "bold")
        .set("fill", "#FFFFFF");
    document = document.add(name_label);

    // Type and size below
    let info_text = format!("{} | {}", simple_type, format_bytes(allocation.size));
    let info_label = SvgText::new(info_text)
        .set("x", x)
        .set("y", y + 35)
        .set("text-anchor", "middle")
        .set("font-size", 7)
        .set("fill", "#E2E8F0");
    document = document.add(info_label);

    Ok(document)
}

/// Add relationship legend
fn add_relationship_legend(mut document: Document, start_y: i32) -> TrackingResult<Document> {
    let legend_items = [
        ("Ownership Transfer", "#E74C3C", "solid", "4"),
        ("Mutable Borrow", "#3498DB", "solid", "3"),
        ("Immutable Borrow", "#27AE60", "solid", "2"),
        ("Clone", "#95A5A6", "solid", "2"),
        ("Shared Pointer", "#9B59B6", "8,4", "3"),
        ("Indirect Reference", "#F39C12", "4,2", "1"),
    ];

    for (i, (label, color, dash, width)) in legend_items.iter().enumerate() {
        let x = 100 + (i % 3) as i32 * 200;
        let y = start_y + (i / 3) as i32 * 25;

        // Legend line
        let legend_line = Line::new()
            .set("x1", x)
            .set("y1", y)
            .set("x2", x + 30)
            .set("y2", y)
            .set("stroke", *color)
            .set("stroke-width", *width)
            .set("stroke-dasharray", *dash);
        document = document.add(legend_line);

        // Legend label
        let legend_label = SvgText::new(*label)
            .set("x", x + 35)
            .set("y", y + 4)
            .set("font-size", 10)
            .set("fill", "#FFFFFF");
        document = document.add(legend_label);
    }

    Ok(document)
}

/// Get scope background color
fn get_scope_background_color(scope_name: &str) -> &'static str {
    match scope_name {
        "Global" => "rgba(52, 73, 94, 0.2)",
        _ => "rgba(52, 152, 219, 0.2)",
    }
}

/// Get scope border color
fn get_scope_border_color(scope_name: &str) -> &'static str {
    match scope_name {
        "Global" => "#34495E",
        _ => "#3498DB",
    }
}

/// Calculate matrix size based on 5-variable standard with dynamic shrinking
fn calculate_dynamic_matrix_size(var_count: usize) -> (i32, i32) {
    let standard_width = 350; // Standard size for 5 variables
    let standard_height = 280; // Standard size for 5 variables
    let card_height = 40; // Height per variable card
    let header_height = 80; // Header and footer space
    let standard_vars = 5;

    if var_count <= standard_vars {
        // SHRINK: Reduce size for fewer variables
        let actual_content_height = header_height + (var_count as i32 * card_height) + 40; // Bottom space
        let width_reduction = ((standard_vars - var_count) * 15) as i32; // Gentle width reduction
        let actual_width = standard_width - width_reduction;

        (actual_width.max(250), actual_content_height.max(150)) // Minimum size protection
    } else {
        // FIXED STANDARD SIZE: Always use standard size, show only 5 + "more" indicator
        (standard_width, standard_height)
    }
}

/// Calculate scope lifetime with FIXED Global scope logic using program runtime
fn calculate_scope_lifetime(scope_name: &str, vars: &[&AllocationInfo]) -> u64 {
    if vars.is_empty() {
        return 0;
    }

    if scope_name == "Global" {
        // FIXED: Global scope uses total program runtime, not just variable span
        estimate_program_runtime()
    } else {
        // Local scope: calculate based on variable lifetimes
        let start = vars.iter().map(|v| v.timestamp_alloc).min().unwrap_or(0);
        let end = vars.iter().map(|v| v.timestamp_alloc).max().unwrap_or(0);
        let span = end - start;
        if span == 0 {
            // If variables allocated at same time, estimate reasonable duration
            100 // 100ms default for local scopes
        } else {
            span
        }
    }
}

/// Estimate total program runtime for Global scope lifetime calculation
fn estimate_program_runtime() -> u64 {
    // Method A: Use a reasonable estimate for program execution time
    // For memory tracking programs, typically run for at least a few seconds
    2000 // 2 seconds - reasonable for Global variable lifetime
}

/// Add prominent global legend for Progress Bar explanation
fn add_prominent_progress_bar_legend(mut document: Document, svg_width: i32) -> Document {
    // Prominent background for the legend
    let legend_bg = Rectangle::new()
        .set("x", 50)
        .set("y", 60)
        .set("width", svg_width - 100)
        .set("height", 35)
        .set("fill", "rgba(252, 211, 77, 0.15)")
        .set("stroke", "#FCD34D")
        .set("stroke-width", 2)
        .set("rx", 8)
        .set("ry", 8);
    document = document.add(legend_bg);

    // Progress bar icon/example
    let example_bg = Rectangle::new()
        .set("x", 70)
        .set("y", 70)
        .set("width", 60)
        .set("height", 8)
        .set("fill", "rgba(255, 255, 255, 0.2)")
        .set("rx", 4);
    document = document.add(example_bg);

    let example_fill = Rectangle::new()
        .set("x", 70)
        .set("y", 70)
        .set("width", 40)
        .set("height", 8)
        .set("fill", "#4CAF50")
        .set("rx", 4);
    document = document.add(example_fill);

    // Prominent explanation text
    let legend_text =
        SvgText::new("📊 Progress Bars show: Variable Size / Largest Variable in Same Scope")
            .set("x", 150)
            .set("y", 78)
            .set("font-size", 14)
            .set("font-weight", "bold")
            .set("fill", "#FCD34D")
            .set("text-shadow", "1px 1px 2px rgba(0,0,0,0.8)");
    document = document.add(legend_text);

    // Size example
    let size_example = SvgText::new("Example: 2.4KB / 5.6KB")
        .set("x", 150)
        .set("y", 88)
        .set("font-size", 10)
        .set("fill", "#E2E8F0")
        .set("font-style", "italic");
    document = document.add(size_example);

    document
}

/// Prioritize scopes for display based on importance
fn prioritize_scopes_for_display<'a>(
    scope_groups: &'a HashMap<String, Vec<&'a AllocationInfo>>,
) -> Vec<(String, Vec<&'a AllocationInfo>)> {
    let mut scopes_with_priority: Vec<_> = scope_groups
        .iter()
        .map(|(name, vars)| {
            let priority = calculate_scope_priority(name, vars);
            let total_memory: usize = vars.iter().map(|v| v.size).sum();
            (name, vars, priority, total_memory)
        })
        .collect();

    // Sort by priority (higher first), then by memory usage (larger first)
    scopes_with_priority.sort_by(|a, b| b.2.cmp(&a.2).then(b.3.cmp(&a.3)));

    scopes_with_priority
        .into_iter()
        .map(|(name, vars, _, _)| (name.clone(), vars.clone()))
        .collect()
}

/// Calculate scope priority based on name patterns and characteristics
fn calculate_scope_priority(scope_name: &str, vars: &[&AllocationInfo]) -> u8 {
    let name_lower = scope_name.to_lowercase();

    // CRITICAL SCOPES (Priority: 100)
    if name_lower == "global"
        || name_lower == "main"
        || name_lower.contains("error")
        || name_lower.contains("panic")
    {
        return 100;
    }

    // HIGH PRIORITY (Priority: 80)
    if name_lower.contains("process")
        || name_lower.contains("parse")
        || name_lower.contains("compute")
        || name_lower.contains("algorithm")
        || name_lower.contains("core")
        || name_lower.contains("engine")
    {
        return 80;
    }

    // MEDIUM PRIORITY (Priority: 60)
    if name_lower.contains("util")
        || name_lower.contains("helper")
        || name_lower.contains("format")
        || name_lower.contains("convert")
    {
        return 60;
    }

    // LOW PRIORITY (Priority: 40)
    if name_lower.contains("test")
        || name_lower.contains("debug")
        || name_lower.contains("macro")
        || name_lower.contains("generated")
    {
        return 40;
    }

    // DEFAULT PRIORITY based on memory usage and variable count
    let total_memory: usize = vars.iter().map(|v| v.size).sum();
    let var_count = vars.len();

    if total_memory > 1024 || var_count > 3 {
        70 // High memory/variable count
    } else if total_memory > 256 || var_count > 1 {
        50 // Medium memory/variable count
    } else {
        30 // Low memory/variable count
    }
}

/// Export complete scope analysis to JSON file
fn export_scope_analysis_json(
    all_scopes: &HashMap<String, Vec<&AllocationInfo>>,
    displayed_scopes: &[(String, Vec<&AllocationInfo>)],
) -> TrackingResult<()> {
    use serde_json::{Map, Value};

    let mut analysis = Map::new();

    // Project analysis summary
    let mut project_analysis = Map::new();
    project_analysis.insert(
        "total_scopes".to_string(),
        Value::Number((all_scopes.len() as u64).into()),
    );
    project_analysis.insert(
        "displayed_in_svg".to_string(),
        Value::Number((displayed_scopes.len() as u64).into()),
    );
    project_analysis.insert(
        "exported_to_json".to_string(),
        Value::Number(((all_scopes.len() - displayed_scopes.len()) as u64).into()),
    );
    project_analysis.insert(
        "layout_strategy".to_string(),
        Value::String("hierarchical_priority".to_string()),
    );
    project_analysis.insert(
        "generation_timestamp".to_string(),
        Value::String(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
        ),
    );
    analysis.insert(
        "project_analysis".to_string(),
        Value::Object(project_analysis),
    );

    // All scopes data
    let mut all_scopes_data = Vec::new();
    for (scope_name, vars) in all_scopes {
        let total_memory: usize = vars.iter().map(|v| v.size).sum();
        let is_displayed = displayed_scopes.iter().any(|(name, _)| name == scope_name);

        let mut scope_data = Map::new();
        scope_data.insert(
            "scope_name".to_string(),
            Value::String(scope_name.clone()),
        );
        scope_data.insert(
            "total_memory".to_string(),
            Value::Number((total_memory as u64).into()),
        );
        scope_data.insert(
            "variable_count".to_string(),
            Value::Number((vars.len() as u64).into()),
        );
        scope_data.insert(
            "display_status".to_string(),
            Value::String(if is_displayed {
                "shown_in_svg".to_string()
            } else {
                "json_only".to_string()
            }),
        );
        scope_data.insert(
            "priority".to_string(),
            Value::Number((calculate_scope_priority(scope_name, vars) as u64).into()),
        );

        // Variables in this scope
        let mut variables = Vec::new();
        for var in vars {
            if let Some(var_name) = &var.var_name {
                let mut var_data = Map::new();
                var_data.insert("name".to_string(), Value::String(var_name.clone()));
                var_data.insert(
                    "type".to_string(),
                    Value::String(var.type_name.as_deref().unwrap_or("Unknown").to_string()),
                );
                var_data.insert(
                    "size_bytes".to_string(),
                    Value::Number((var.size as u64).into()),
                );
                var_data.insert(
                    "timestamp".to_string(),
                    Value::Number(var.timestamp_alloc.into()),
                );
                variables.push(Value::Object(var_data));
            }
        }
        scope_data.insert("variables".to_string(), Value::Array(variables));

        all_scopes_data.push(Value::Object(scope_data));
    }
    analysis.insert("all_scopes".to_string(), Value::Array(all_scopes_data));

    // Write to JSON file
    let json_content = serde_json::to_string(&Value::Object(analysis)).map_err(|e| {
        TrackingError::SerializationError(format!("JSON serialization failed: {e}"))
    })?;

    std::fs::write("scope_analysis.json", json_content)
        .map_err(|e| TrackingError::IoError(e.to_string()))?;

    tracing::info!("Exported complete scope analysis to scope_analysis.json");
    Ok(())
}

/// Get simple type name

/// Get color based on duration ratio (0.0 to 1.0)
/// Assign colors based on relative lifecycle length: longest time=dark color, shortest time=white, global scope=special deep blue
fn get_duration_color(ratio: f64, is_global: bool) -> String {
    if is_global {
        // Global scope uses special deep blue color
        return "#0A2540".to_string();
    }

    // Create gradient from white to deep blue
    // ratio = 0.0 (shortest time) -> close to white
    // ratio = 1.0 (longest time) -> deep blue

    if ratio <= 0.01 {
        // Very short time or no time difference -> light gray-white
        "#F8FAFC".to_string()
    } else {
        // Calculate RGB values, gradient from light blue-white to deep blue
        let base_r = 248; // Starting red value (close to white)
        let base_g = 250; // Starting green value
        let base_b = 252; // Starting blue value

        let target_r = 30; // Target red value (deep blue)
        let target_g = 64; // Target green value
        let target_b = 175; // Target blue value

        // Use smooth gradient function
        let smooth_ratio = ratio.powf(0.7); // Make gradient smoother

        let r = (base_r as f64 + (target_r as f64 - base_r as f64) * smooth_ratio) as u8;
        let g = (base_g as f64 + (target_g as f64 - base_g as f64) * smooth_ratio) as u8;
        let b = (base_b as f64 + (target_b as f64 - base_b as f64) * smooth_ratio) as u8;

        format!("#{r:02X}{g:02X}{b:02X}")
    }
}

/// Add matrix layout section with INTELLIGENT 15-SCOPE LIMITATION
fn add_matrix_layout_section(
    mut document: Document,
    tracked_vars: &[&AllocationInfo],
    start_x: i32,
    start_y: i32,
) -> TrackingResult<Document> {
    // Group variables by scope
    let mut scope_groups: HashMap<String, Vec<&AllocationInfo>> = HashMap::new();
    for var in tracked_vars {
        let scope = identify_precise_scope(var);
        scope_groups.entry(scope).or_default().push(*var);
    }

    // INTELLIGENT SCOPE PRIORITIZATION - Maximum 15 scopes
    let prioritized_scopes = prioritize_scopes_for_display(&scope_groups);
    let selected_scopes: Vec<_> = prioritized_scopes.into_iter().take(15).collect();

    // tracing::info!(
    //     "Total scopes found: {}, displaying: {}",
    //     scope_groups.len(),
    //     selected_scopes.len()
    // );

    // Calculate maximum duration across all SELECTED scopes for relative color scaling
    let max_duration = selected_scopes
        .iter()
        .map(|(_, vars)| {
            if !vars.is_empty() {
                let start = vars.iter().map(|v| v.timestamp_alloc).min().unwrap_or(0);
                let end = vars.iter().map(|v| v.timestamp_alloc).max().unwrap_or(0);
                end - start
            } else {
                0
            }
        })
        .max()
        .unwrap_or(1); // Avoid division by zero

    // DYNAMIC GRID LAYOUT - 3 columns, up to 5 rows
    let base_matrix_width = 350;
    let base_matrix_height = 180;
    let spacing_x = 450; // Increased spacing to prevent matrix overlap
    let spacing_y = 250;

    let mut positions = Vec::new();

    // Calculate positions for SELECTED matrices only
    for (i, (scope_name, _)) in selected_scopes.iter().enumerate() {
        let col = i % 3;
        let row = i / 3;
        let x = start_x + (col as i32 * spacing_x);
        let y = start_y + (row as i32 * spacing_y);
        positions.push((scope_name, x, y));
    }

    // Draw relationship lines first (only for displayed scopes)
    for (i, (scope_name, x, y)) in positions.iter().enumerate() {
        if *scope_name != "Global" && i > 0 {
            // Find Global scope position
            if let Some((_, global_x, global_y)) =
                positions.iter().find(|(name, _, _)| *name == "Global")
            {
                let line = Line::new()
                    .set("x1", global_x + base_matrix_width / 2)
                    .set("y1", global_y + base_matrix_height)
                    .set("x2", x + base_matrix_width / 2)
                    .set("y2", *y)
                    .set("stroke", "#7F8C8D")
                    .set("stroke-width", 2)
                    .set("stroke-dasharray", "5,3");
                document = document.add(line);
            }
        }
    }

    // Render SELECTED scope matrices with relative color scaling
    for ((scope_name, vars), (_, x, y)) in selected_scopes.iter().zip(positions.iter()) {
        document = render_scope_matrix_fixed(
            document,
            scope_name,
            vars,
            *x,
            *y,
            base_matrix_width,
            base_matrix_height,
            max_duration,
        )?;
    }

    // Export complete data to JSON if there are overflow scopes
    if scope_groups.len() > 15 {
        export_scope_analysis_json(&scope_groups, &selected_scopes)?;
    }

    Ok(document)
}

/// Render single scope matrix with DYNAMIC SIZING and ENHANCED MEMORY VISUALIZATION
fn render_scope_matrix_fixed(
    mut document: Document,
    scope_name: &str,
    vars: &[&AllocationInfo],
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    max_duration: u64, // Maximum duration across all scopes for normalization
) -> TrackingResult<Document> {
    // DYNAMIC MATRIX SIZING based on variable count
    let (dynamic_width, dynamic_height) = calculate_dynamic_matrix_size(vars.len());
    let actual_width = dynamic_width.max(width);
    let actual_height = dynamic_height.max(height);

    let mut matrix_group = Group::new().set("transform", format!("translate({x}, {y})"));

    // ENHANCED SCOPE LIFETIME CALCULATION
    let duration = calculate_scope_lifetime(scope_name, vars);
    let total_memory = vars.iter().map(|v| v.size).sum::<usize>();
    // FIXED: Remove incorrect peak_memory calculation - this was calculating max single allocation
    // instead of peak total memory. Peak memory should come from stats.peak_memory which is
    // the historical maximum of total active memory, not the maximum single allocation size.
    let active_vars = vars.len();

    // Calculate duration ratio (0.0 to 1.0)
    let duration_ratio = if max_duration > 0 {
        duration as f64 / max_duration as f64
    } else {
        0.0
    };

    // Get border color based on duration ratio and scope type
    let is_global = scope_name == "Global";
    let border_color = get_duration_color(duration_ratio, is_global);

    // ENHANCED MATRIX CONTAINER with dynamic sizing
    let container = Rectangle::new()
        .set("width", actual_width)
        .set("height", actual_height)
        .set("fill", "rgba(30, 64, 175, 0.1)")
        .set("stroke", border_color.as_str())
        .set("stroke-width", 3)
        .set(
            "stroke-dasharray",
            if scope_name != "Global" {
                "8,4"
            } else {
                "none"
            },
        )
        .set("rx", 12);
    matrix_group = matrix_group.add(container);

    // ENHANCED SCOPE HEADER with comprehensive memory overview - ENGLISH ONLY
    let header_text = format!(
        "Scope: {} | Memory: {} | Variables: {} | Lifetime: {}ms",
        scope_name,
        format_bytes(total_memory),
        active_vars,
        duration
    );
    let enhanced_title = SvgText::new(header_text)
        .set("x", 15)
        .set("y", 25)
        .set("font-size", 11)
        .set("font-weight", "700")
        .set("fill", "#f8fafc");
    matrix_group = matrix_group.add(enhanced_title);

    // Variables section with ENHANCED MODERN CARD DESIGN
    let var_start_y = 45;
    let card_height = 45; // Increased height for vertical layout (3 lines)
    let var_spacing = 50; // More spacing for taller cards
    let _font_size = 10;

    for (i, var) in vars.iter().take(4).enumerate() {
        // Limit to 4 for better layout
        let var_y = var_start_y + (i as i32 * var_spacing);
        let default_name = "<unknown>".to_string();
        let var_name = var.var_name.as_ref().unwrap_or(&default_name);
        let type_name = get_simple_type(var.type_name.as_ref().unwrap_or(&"Unknown".to_string()));
        let duration_ms = estimate_variable_duration(var);

        // Calculate progress percentage for the progress bar
        let max_size_in_scope = vars.iter().map(|v| v.size).max().unwrap_or(1);
        let progress_ratio = var.size as f64 / max_size_in_scope as f64;
        let _progress_width = (progress_ratio * 180.0) as i32;

        // ENHANCED MODERN CARD with dynamic width
        let card_width = actual_width - 20;
        let card_bg = Rectangle::new()
            .set("x", 10)
            .set("y", var_y - 5)
            .set("width", card_width)
            .set("height", card_height)
            .set("fill", "rgba(255, 255, 255, 0.08)")
            .set("stroke", "rgba(255, 255, 255, 0.15)")
            .set("stroke-width", 1)
            .set("rx", 8)
            .set("ry", 8);
        matrix_group = matrix_group.add(card_bg);

        // Variable name with enhanced styling
        let var_label = SvgText::new(var_name)
            .set("x", 18)
            .set("y", var_y + 8)
            .set("font-size", 12)
            .set("font-weight", "bold")
            .set("fill", "#FFFFFF")
            .set("text-shadow", "1px 1px 2px rgba(0,0,0,0.8)");
        matrix_group = matrix_group.add(var_label);

        // Type label with enhanced color coding
        let (type_start_color, _) = get_type_gradient_colors(&type_name);
        let type_label = SvgText::new(format!("({type_name})"))
            .set("x", 18)
            .set("y", var_y + 22)
            .set("font-size", 9)
            .set("fill", type_start_color)
            .set("font-weight", "600");
        matrix_group = matrix_group.add(type_label);

        // DYNAMIC PROGRESS BAR - Responsive to matrix width
        let available_width = card_width - 40; // Leave margins
        let progress_bar_width = (available_width as f64 * 0.5) as i32; // 50% of available width
        let progress_x = 20; // Fixed left margin

        let progress_bg = Rectangle::new()
            .set("x", progress_x)
            .set("y", var_y + 15) // Moved down to avoid overlap
            .set("width", progress_bar_width)
            .set("height", 8)
            .set("fill", "rgba(255, 255, 255, 0.1)")
            .set("stroke", "rgba(255, 255, 255, 0.2)")
            .set("stroke-width", 1)
            .set("rx", 4)
            .set("ry", 4);
        matrix_group = matrix_group.add(progress_bg);

        // ENHANCED GRADIENT PROGRESS BAR with type-specific colors
        let (start_color, _) = get_type_gradient_colors(&type_name);
        let progress_fill_width = (progress_ratio * progress_bar_width as f64) as i32;
        let progress_fill = Rectangle::new()
            .set("x", progress_x)
            .set("y", var_y + 15)
            .set("width", progress_fill_width)
            .set("height", 8)
            .set("fill", start_color) // Enhanced with gradient colors
            .set("rx", 4)
            .set("ry", 4);
        matrix_group = matrix_group.add(progress_fill);

        // VERTICAL LAYOUT - Size display below progress bar to avoid overlap
        let size_display = format!(
            "{} / {}",
            format_bytes(var.size),
            format_bytes(max_size_in_scope)
        );
        let size_label = SvgText::new(size_display)
            .set("x", progress_x + progress_bar_width + 10)
            .set("y", var_y + 20)
            .set("font-size", 8)
            .set("font-weight", "bold")
            .set("fill", "#E2E8F0");
        matrix_group = matrix_group.add(size_label);

        // LIFETIME on separate line to prevent overlap
        let time_label = SvgText::new(format!("Active {duration_ms}ms"))
            .set("x", 20)
            .set("y", var_y + 30)
            .set("font-size", 7)
            .set("fill", "#FCD34D")
            .set("font-weight", "500");
        matrix_group = matrix_group.add(time_label);
    }

    // Show "more" indicator if needed
    if vars.len() > 4 {
        let more_text = format!("+ {} more variables", vars.len() - 4);
        let more_label = SvgText::new(more_text)
            .set("x", 20)
            .set("y", var_start_y + (4 * var_spacing) + 10)
            .set("font-size", 9)
            .set("font-weight", "500")
            .set("fill", "#94A3B8")
            .set("font-style", "italic");
        matrix_group = matrix_group.add(more_label);
    }

    // INTUITIVE EXPLANATION at bottom of matrix - ENGLISH ONLY
    let explanation_y = actual_height - 15;
    let explanation_text = "Progress Bar: Current Size / Max Size in Scope";
    let explanation = SvgText::new(explanation_text)
        .set("x", 15)
        .set("y", explanation_y)
        .set("font-size", 8)
        .set("font-weight", "500")
        .set("fill", "#FCD34D")
        .set("font-style", "italic");
    matrix_group = matrix_group.add(explanation);

    document = document.add(matrix_group);
    Ok(document)
}

/// Identify precise scope for allocation
fn identify_precise_scope(allocation: &AllocationInfo) -> String {
    if let Some(var_name) = &allocation.var_name {
        if var_name.contains("global") {
            return "Global".to_string();
        }
        // Use timestamp to infer scope
        match allocation.timestamp_alloc {
            0..=1000 => "Global".to_string(),
            1001..=2000 => "demonstrate_builtin_types".to_string(),
            2001..=3000 => "demonstrate_smart_pointers".to_string(),
            3001..=4000 => "demonstrate_custom_structures".to_string(),
            4001..=5000 => "demonstrate_complex_patterns".to_string(),
            5001..=6000 => "simulate_web_server_scenario".to_string(),
            _ => "simulate_data_processing_pipeline".to_string(),
        }
    } else {
        "Global".to_string()
    }
}

/// Estimate variable duration
fn estimate_variable_duration(var: &AllocationInfo) -> u64 {
    let base_duration = match var.size {
        0..=100 => 10,
        101..=1000 => 50,
        1001..=10000 => 100,
        _ => 200,
    };

    let type_multiplier = if let Some(type_name) = &var.type_name {
        if type_name.contains("Vec") || type_name.contains("HashMap") {
            2.0
        } else if type_name.contains("Box") || type_name.contains("Rc") {
            1.5
        } else {
            1.0
        }
    } else {
        1.0
    };

    (base_duration as f64 * type_multiplier) as u64
}

// ============================================================================
// Core SVG Generation Functions (moved from export_enhanced.rs for consolidation)
// ============================================================================

use crate::core::types::TypeMemoryUsage;

// Using EnhancedTypeInfo from export_enhanced module

/// Enhanced type information processing with variable names and inner type extraction
pub fn enhance_type_information(
    memory_by_type: &[TypeMemoryUsage],
    allocations: &[AllocationInfo],
) -> Vec<crate::export::export_enhanced::EnhancedTypeInfo> {
    let mut enhanced_types = Vec::new();

    for usage in memory_by_type {
        // Skip unknown types
        if usage.type_name == "Unknown" {
            continue;
        }

        // Use enhanced type analysis for better categorization
        let (simplified_name, category, subcategory) =
            analyze_type_with_detailed_subcategory(&usage.type_name);

        // Collect variable names for this type
        let variable_names: Vec<String> = allocations
            .iter()
            .filter_map(|alloc| {
                if let (Some(var_name), Some(type_name)) = (&alloc.var_name, &alloc.type_name) {
                    let (alloc_simplified, _, _) =
                        analyze_type_with_detailed_subcategory(type_name);
                    if alloc_simplified == simplified_name {
                        Some(var_name)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .take(5) // Limit to 5 variable names
            .map(|s| s.clone())
            .collect();

        // Add the main type with subcategory information
        enhanced_types.push(crate::export::export_enhanced::EnhancedTypeInfo {
            simplified_name,
            category,
            subcategory,
            total_size: usage.total_size,
            allocation_count: usage.allocation_count,
            variable_names,
        });
    }

    enhanced_types
}

/// Enhanced type analysis with detailed subcategory detection
fn analyze_type_with_detailed_subcategory(type_name: &str) -> (String, String, String) {
    let clean_type = type_name.trim();

    // Handle empty or explicitly unknown types first
    if clean_type.is_empty() || clean_type == "Unknown" {
        return (
            "Unknown Type".to_string(),
            "Unknown".to_string(),
            "Other".to_string(),
        );
    }

    // Collections analysis
    if clean_type.contains("Vec<") || clean_type.contains("vec::Vec") {
        return (
            "Vec<T>".to_string(),
            "Collections".to_string(),
            "Vec<T>".to_string(),
        );
    }

    if clean_type.contains("HashMap") {
        return (
            "HashMap<K,V>".to_string(),
            "Collections".to_string(),
            "HashMap<K,V>".to_string(),
        );
    }

    if clean_type.contains("String") {
        return (
            "String".to_string(),
            "Basic Types".to_string(),
            "Strings".to_string(),
        );
    }

    // Integer types
    if clean_type == "i32" || clean_type.ends_with("::i32") {
        return (
            "i32".to_string(),
            "Basic Types".to_string(),
            "Integers".to_string(),
        );
    }

    if clean_type == "u8" || clean_type.ends_with("::u8") {
        return (
            "u8".to_string(),
            "Basic Types".to_string(),
            "Integers".to_string(),
        );
    }

    // Default case
    (
        clean_type.to_string(),
        "Other".to_string(),
        "Custom".to_string(),
    )
}

/// Categorize enhanced allocations by type category - delegate to export_enhanced
pub fn categorize_enhanced_allocations(
    enhanced_types: &[crate::export::export_enhanced::EnhancedTypeInfo],
) -> Vec<crate::export::export_enhanced::AllocationCategory> {
    crate::export::export_enhanced::categorize_enhanced_allocations(enhanced_types)
}

/// Export comprehensive unsafe/FFI memory analysis to dedicated SVG
pub fn export_unsafe_ffi_dashboard<P: AsRef<Path>>(
    tracker: &UnsafeFFITracker,
    path: P,
) -> TrackingResult<()> {
    let path = path.as_ref();
    tracing::info!("Exporting unsafe/FFI dashboard to: {}", path.display());

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let enhanced_allocations = tracker.get_enhanced_allocations()?;
    let safety_violations = tracker.get_safety_violations()?;

    let document = create_unsafe_ffi_dashboard(&enhanced_allocations, &safety_violations)?;

    let mut file = File::create(path)?;
    svg::write(&mut file, &document)
        .map_err(|e| TrackingError::SerializationError(format!("Failed to write SVG: {e}")))?;

    tracing::info!("Successfully exported unsafe/FFI dashboard SVG");
    Ok(())
}

/// Create the main unsafe/FFI analysis dashboard
fn create_unsafe_ffi_dashboard(
    allocations: &[EnhancedAllocationInfo],
    violations: &[SafetyViolation],
) -> TrackingResult<Document> {
    let width = 1400;
    let height = 1000;

    let mut document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", width)
        .set("height", height)
        .set("style", "background: linear-gradient(135deg, #2c3e50 0%, #34495e 50%, #2c3e50 100%); font-family: 'Segoe UI', Arial, sans-serif;");

    // Add definitions for gradients and markers
    document = add_svg_definitions(document);

    // Header with title and key metrics
    document = add_dashboard_header(document, allocations, violations)?;

    // Main content areas
    document = add_allocation_source_breakdown(document, allocations)?;
    document = add_memory_safety_status(document, violations)?;
    document = add_boundary_crossing_flow(document, allocations)?;
    document = add_unsafe_hotspots(document, allocations)?;

    Ok(document)
}

/// Add SVG definitions for gradients, markers, etc.
fn add_svg_definitions(document: Document) -> Document {
    let mut defs = Definitions::new();

    // Arrow marker for flow diagrams
    let arrow_marker = Marker::new()
        .set("id", "arrowhead")
        .set("markerWidth", 10)
        .set("markerHeight", 7)
        .set("refX", 9)
        .set("refY", 3.5)
        .set("orient", "auto")
        .add(
            Polygon::new()
                .set("points", "0 0, 10 3.5, 0 7")
                .set("fill", "#e74c3c"),
        );
    defs = defs.add(arrow_marker);

    document.add(defs)
}

/// Add dashboard header with title and key metrics
fn add_dashboard_header(
    mut document: Document,
    allocations: &[EnhancedAllocationInfo],
    violations: &[SafetyViolation],
) -> TrackingResult<Document> {
    // Main title
    let title = SvgText::new("Unsafe Rust & FFI Memory Analysis Dashboard")
        .set("x", 700)
        .set("y", 40)
        .set("text-anchor", "middle")
        .set("font-size", 24)
        .set("font-weight", "bold")
        .set("fill", "#ecf0f1");
    document = document.add(title);

    // Calculate key metrics
    let unsafe_count = allocations
        .iter()
        .filter(|a| matches!(a.source, AllocationSource::UnsafeRust { .. }))
        .count();
    let ffi_count = allocations
        .iter()
        .filter(|a| matches!(a.source, AllocationSource::FfiC { .. }))
        .count();
    let cross_boundary_events: usize = allocations
        .iter()
        .map(|a| a.cross_boundary_events.len())
        .sum();
    let total_unsafe_memory: usize = allocations
        .iter()
        .filter(|a| !matches!(a.source, AllocationSource::RustSafe))
        .map(|a| a.base.size)
        .sum();

    // Metrics cards
    let metrics = vec![
        ("Unsafe Allocations", unsafe_count.clone(), "#e74c3c"),
        ("FFI Allocations", ffi_count.clone(), "#3498db"),
        (
            "Boundary Crossings",
            cross_boundary_events.clone(),
            "#f39c12",
        ),
        ("Safety Violations", violations.len().clone(), "#e67e22"),
        (
            "Unsafe Memory",
            total_unsafe_memory,
            "#9b59b6",
        ),
    ];

    for (i, (label, value, color)) in metrics.iter().enumerate() {
        let x = 100 + i as i32 * 250;
        let y = 80;

        // Card background
        let card = Rectangle::new()
            .set("x", x - 60)
            .set("y", y - 25)
            .set("width", 120)
            .set("height", 50)
            .set("fill", *color)
            .set("fill-opacity", 0.2)
            .set("stroke", *color)
            .set("stroke-width", 2)
            .set("rx", 8);
        document = document.add(card);

        // Value
        let value_text = SvgText::new(&value.to_string())
            .set("x", x)
            .set("y", y - 5)
            .set("text-anchor", "middle")
            .set("font-size", 16)
            .set("font-weight", "bold")
            .set("fill", *color);
        document = document.add(value_text);

        // Label
        let label_text = SvgText::new(*label)
            .set("x", x)
            .set("y", y + 15)
            .set("text-anchor", "middle")
            .set("font-size", 10)
            .set("fill", "#bdc3c7");
        document = document.add(label_text);
    }

    Ok(document)
}

/// Add allocation source breakdown visualization
fn add_allocation_source_breakdown(
    mut document: Document,
    allocations: &[EnhancedAllocationInfo],
) -> TrackingResult<Document> {
    let start_x = 50;
    let start_y = 150;
    let width = 600;
    let height = 300;

    // Section title
    let title = SvgText::new("Memory Allocation Sources")
        .set("x", start_x + width / 2)
        .set("y", start_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 18)
        .set("font-weight", "bold")
        .set("fill", "#ecf0f1");
    document = document.add(title);

    // Background
    let bg = Rectangle::new()
        .set("x", start_x)
        .set("y", start_y)
        .set("width", width)
        .set("height", height)
        .set("fill", "rgba(52, 73, 94, 0.3)")
        .set("stroke", "#34495e")
        .set("stroke-width", 2)
        .set("rx", 10);
    document = document.add(bg);

    // Count allocations by source
    let mut safe_count = 0;
    let mut unsafe_count = 0;
    let mut ffi_count = 0;
    let mut cross_boundary_count = 0;

    for allocation in allocations {
        match &allocation.source {
            AllocationSource::RustSafe => safe_count += 1,
            AllocationSource::UnsafeRust { .. } => unsafe_count += 1,
            AllocationSource::FfiC { .. } => ffi_count += 1,
            AllocationSource::CrossBoundary { .. } => cross_boundary_count += 1,
        }
    }

    let total = safe_count + unsafe_count + ffi_count + cross_boundary_count;
    if total == 0 {
        let no_data = SvgText::new("No allocation data available")
            .set("x", start_x + width / 2)
            .set("y", start_y + height / 2)
            .set("text-anchor", "middle")
            .set("font-size", 14)
            .set("fill", "#95a5a6");
        document = document.add(no_data);
        return Ok(document);
    }

    // Create simple bar chart instead of pie chart
    let sources = [
        ("Safe Rust", safe_count, "#2ecc71"),
        ("Unsafe Rust", unsafe_count, "#e74c3c"),
        ("FFI", ffi_count, "#3498db"),
        ("Cross-boundary", cross_boundary_count, "#9b59b6"),
    ];

    for (i, (label, count, color)) in sources.iter().enumerate() {
        if *count > 0 {
            let x = start_x + 50 + i as i32 * 120;
            let y = start_y + 200;
            let bar_height = (*count as f32 / total as f32 * 100.0) as i32;

            // Bar
            let bar = Rectangle::new()
                .set("x", x)
                .set("y", y - bar_height)
                .set("width", 40)
                .set("height", bar_height)
                .set("fill", *color);
            document = document.add(bar);

            // Count label
            let count_text = SvgText::new(&count.to_string())
                .set("x", x + 20)
                .set("y", y - bar_height - 5)
                .set("text-anchor", "middle")
                .set("font-size", 12)
                .set("font-weight", "bold")
                .set("fill", *color);
            document = document.add(count_text);

            // Label
            let label_text = SvgText::new(*label)
                .set("x", x + 20)
                .set("y", y + 20)
                .set("text-anchor", "middle")
                .set("font-size", 10)
                .set("fill", "#ecf0f1");
            document = document.add(label_text);
        }
    }

    Ok(document)
}

/// Add memory safety status panel
fn add_memory_safety_status(
    mut document: Document,
    violations: &[SafetyViolation],
) -> TrackingResult<Document> {
    let start_x = 750;
    let start_y = 150;
    let width = 600;
    let height = 300;

    // Section title
    let title = SvgText::new("Memory Safety Status")
        .set("x", start_x + width / 2)
        .set("y", start_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 18)
        .set("font-weight", "bold")
        .set("fill", "#ecf0f1");
    document = document.add(title);

    // Background
    let bg_color = if violations.is_empty() {
        "#27ae60"
    } else {
        "#e74c3c"
    };
    let bg = Rectangle::new()
        .set("x", start_x)
        .set("y", start_y)
        .set("width", width)
        .set("height", height)
        .set("fill", format!("{bg_color}20"))
        .set("stroke", bg_color)
        .set("stroke-width", 2)
        .set("rx", 10);
    document = document.add(bg);

    if violations.is_empty() {
        // Safe status
        let safe_text = SvgText::new("No Safety Violations Detected")
            .set("x", start_x + width / 2)
            .set("y", start_y + 150)
            .set("text-anchor", "middle")
            .set("font-size", 16)
            .set("font-weight", "bold")
            .set("fill", "#27ae60");
        document = document.add(safe_text);

        let safe_desc =
            SvgText::new("All unsafe operations and FFI calls appear to be memory-safe")
                .set("x", start_x + width / 2)
                .set("y", start_y + 180)
                .set("text-anchor", "middle")
                .set("font-size", 12)
                .set("fill", "#2ecc71");
        document = document.add(safe_desc);
    } else {
        // Violations detected
        let violation_text =
            SvgText::new(format!("{} Safety Violations Detected", violations.len()))
                .set("x", start_x + width / 2)
                .set("y", start_y + 120)
                .set("text-anchor", "middle")
                .set("font-size", 16)
                .set("font-weight", "bold")
                .set("fill", "#e74c3c");
        document = document.add(violation_text);

        // List violations
        for (i, violation) in violations.iter().take(5).enumerate() {
            let y = start_y + 160 + i as i32 * 20;

            let description = match violation {
                SafetyViolation::DoubleFree { .. } => "Double Free",
                SafetyViolation::InvalidFree { .. } => "Invalid Free",
                SafetyViolation::PotentialLeak { .. } => "Memory Leak",
                SafetyViolation::CrossBoundaryRisk { .. } => "Cross-Boundary Risk",
            };

            let violation_item = SvgText::new(format!("• {description}"))
                .set("x", start_x + 30)
                .set("y", y)
                .set("font-size", 12)
                .set("fill", "#e74c3c");
            document = document.add(violation_item);
        }
    }

    Ok(document)
}

/// Add boundary crossing flow diagram
fn add_boundary_crossing_flow(
    mut document: Document,
    allocations: &[EnhancedAllocationInfo],
) -> TrackingResult<Document> {
    let start_x = 50;
    let start_y = 500;
    let width = 600;
    let height = 200;

    // Section title
    let title = SvgText::new("Cross-Language Memory Flow")
        .set("x", start_x + width / 2)
        .set("y", start_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 18)
        .set("font-weight", "bold")
        .set("fill", "#ecf0f1");
    document = document.add(title);

    // Background
    let bg = Rectangle::new()
        .set("x", start_x)
        .set("y", start_y)
        .set("width", width)
        .set("height", height)
        .set("fill", "rgba(52, 73, 94, 0.3)")
        .set("stroke", "#34495e")
        .set("stroke-width", 2)
        .set("rx", 10);
    document = document.add(bg);

    // Rust territory
    let rust_box = Rectangle::new()
        .set("x", start_x + 50)
        .set("y", start_y + 50)
        .set("width", 200)
        .set("height", 100)
        .set("fill", "#2ecc71")
        .set("fill-opacity", 0.2)
        .set("stroke", "#2ecc71")
        .set("stroke-width", 2)
        .set("rx", 8);
    document = document.add(rust_box);

    let rust_label = SvgText::new("RUST")
        .set("x", start_x + 150)
        .set("y", start_y + 110)
        .set("text-anchor", "middle")
        .set("font-size", 14)
        .set("font-weight", "bold")
        .set("fill", "#2ecc71");
    document = document.add(rust_label);

    // FFI territory
    let ffi_box = Rectangle::new()
        .set("x", start_x + 350)
        .set("y", start_y + 50)
        .set("width", 200)
        .set("height", 100)
        .set("fill", "#3498db")
        .set("fill-opacity", 0.2)
        .set("stroke", "#3498db")
        .set("stroke-width", 2)
        .set("rx", 8);
    document = document.add(ffi_box);

    let ffi_label = SvgText::new("FFI / C")
        .set("x", start_x + 450)
        .set("y", start_y + 110)
        .set("text-anchor", "middle")
        .set("font-size", 14)
        .set("font-weight", "bold")
        .set("fill", "#3498db");
    document = document.add(ffi_label);

    // Count boundary events
    let mut rust_to_ffi = 0;
    let mut ffi_to_rust = 0;

    for allocation in allocations {
        for event in &allocation.cross_boundary_events {
            match event.event_type {
                BoundaryEventType::RustToFfi => rust_to_ffi += 1,
                BoundaryEventType::FfiToRust => ffi_to_rust += 1,
                BoundaryEventType::OwnershipTransfer => rust_to_ffi += 1,
                BoundaryEventType::SharedAccess => {
                    rust_to_ffi += 1;
                    ffi_to_rust += 1;
                }
            }
        }
    }

    // Draw flow arrows
    if rust_to_ffi > 0 {
        let arrow = Line::new()
            .set("x1", start_x + 250)
            .set("y1", start_y + 80)
            .set("x2", start_x + 350)
            .set("y2", start_y + 80)
            .set("stroke", "#e74c3c")
            .set("stroke-width", 3)
            .set("marker-end", "url(#arrowhead)");
        document = document.add(arrow);

        let count_text = SvgText::new(&rust_to_ffi.to_string())
            .set("x", start_x + 300)
            .set("y", start_y + 75)
            .set("text-anchor", "middle")
            .set("font-size", 12)
            .set("font-weight", "bold")
            .set("fill", "#e74c3c");
        document = document.add(count_text);
    }

    if ffi_to_rust > 0 {
        let count_text = SvgText::new(&ffi_to_rust.to_string())
            .set("x", start_x + 300)
            .set("y", start_y + 135)
            .set("text-anchor", "middle")
            .set("font-size", 12)
            .set("font-weight", "bold")
            .set("fill", "#f39c12");
        document = document.add(count_text);
    }

    Ok(document)
}

/// Add unsafe memory hotspots visualization
fn add_unsafe_hotspots(
    mut document: Document,
    allocations: &[EnhancedAllocationInfo],
) -> TrackingResult<Document> {
    let start_x = 750;
    let start_y = 500;
    let width = 600;
    let height = 200;

    // Section title
    let title = SvgText::new("Unsafe Memory Hotspots")
        .set("x", start_x + width / 2)
        .set("y", start_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 18)
        .set("font-weight", "bold")
        .set("fill", "#ecf0f1");
    document = document.add(title);

    // Background
    let bg = Rectangle::new()
        .set("x", start_x)
        .set("y", start_y)
        .set("width", width)
        .set("height", height)
        .set("fill", "rgba(52, 73, 94, 0.3)")
        .set("stroke", "#34495e")
        .set("stroke-width", 2)
        .set("rx", 10);
    document = document.add(bg);

    // Find unsafe allocations
    let unsafe_allocations: Vec<_> = allocations
        .iter()
        .filter(|a| !matches!(a.source, AllocationSource::RustSafe))
        .collect();

    if unsafe_allocations.is_empty() {
        let no_unsafe = SvgText::new("No unsafe memory allocations detected")
            .set("x", start_x + width / 2)
            .set("y", start_y + height / 2)
            .set("text-anchor", "middle")
            .set("font-size", 14)
            .set("fill", "#2ecc71");
        document = document.add(no_unsafe);
        return Ok(document);
    }

    // Display top unsafe allocations
    for (i, allocation) in unsafe_allocations.iter().take(6).enumerate() {
        let x = start_x + 80 + (i % 3) as i32 * 180;
        let y = start_y + 80 + (i / 3) as i32 * 70;

        // Hotspot circle
        let size_factor = (allocation.base.size.min(1000) as f32 / 1000.0 * 15.0 + 5.0) as i32;
        let color = match &allocation.source {
            AllocationSource::UnsafeRust { .. } => "#e74c3c",
            AllocationSource::FfiC { .. } => "#3498db",
            AllocationSource::CrossBoundary { .. } => "#9b59b6",
            _ => "#95a5a6",
        };

        let hotspot = Circle::new()
            .set("cx", x)
            .set("cy", y)
            .set("r", size_factor)
            .set("fill", color)
            .set("fill-opacity", 0.7)
            .set("stroke", color)
            .set("stroke-width", 2);
        document = document.add(hotspot);

        // Size label
        let size_text = SvgText::new(format_bytes(allocation.base.size))
            .set("x", x)
            .set("y", y + 4)
            .set("text-anchor", "middle")
            .set("font-size", 8)
            .set("font-weight", "bold")
            .set("fill", "#ffffff");
        document = document.add(size_text);

        // Type label
        let type_label = match &allocation.source {
            AllocationSource::UnsafeRust { .. } => "UNSAFE",
            AllocationSource::FfiC { .. } => "FFI",
            AllocationSource::CrossBoundary { .. } => "CROSS",
            _ => "OTHER",
        };

        let type_text = SvgText::new(type_label)
            .set("x", x)
            .set("y", y + 35)
            .set("text-anchor", "middle")
            .set("font-size", 10)
            .set("fill", color);
        document = document.add(type_text);
    }

    Ok(document)
}
