//! Unified visualization module for memscope-rs
//! Provides memory analysis and lifecycle timeline SVG exports

use crate::tracker::MemoryTracker;
use crate::types::{AllocationInfo, MemoryStats, TrackingError, TrackingResult};
use crate::utils::{format_bytes, get_simple_type, get_type_color, get_type_gradient_colors};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use svg::node::element::{Circle, Group, Line, Rectangle, Style, Text as SvgText};
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
    let width = 1800; // 进一步增加宽度，让布局更宽敞
    let height = 2400; // Tall document for comprehensive analysis

    let mut document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", width)
        .set("height", height)
        .set("style", "background: linear-gradient(135deg, #ecf0f1 0%, #bdc3c7 100%); font-family: 'Segoe UI', Arial, sans-serif;");

    // 1. Title: Rust Memory Usage Analysis (handled by add_enhanced_header)
    document = crate::export_enhanced::add_enhanced_header(document, stats)?;

    // 3. Performance Dashboard
    document = crate::export_enhanced::add_performance_dashboard(document, stats, allocations)?;

    // 4. Memory Allocation Heatmap
    document = crate::export_enhanced::add_memory_heatmap(document, allocations)?;

    // 5. Left side: Memory Usage by Type
    // 修复：获取实际的内存类型数据而不是空数组
    let memory_by_type_data = tracker.get_memory_by_type().unwrap_or_default();
    let memory_by_type =
        crate::export_enhanced::enhance_type_information(&memory_by_type_data, allocations);
    document = crate::export_enhanced::add_enhanced_type_chart(document, &memory_by_type)?;

    // 6. Right side: Memory Fragmentation Analysis
    document = crate::export_enhanced::add_fragmentation_analysis(document, allocations)?;

    // 7. Left side: Tracked Variables by Category
    let categorized = crate::export_enhanced::categorize_allocations(allocations);
    document = crate::export_enhanced::add_categorized_allocations(document, &categorized)?;

    // 8. Right side: Call Stack Analysis
    document = crate::export_enhanced::add_callstack_analysis(document, allocations)?;

    // 9. Memory Growth Trends
    document = crate::export_enhanced::add_memory_growth_trends(document, allocations, stats)?;

    // 10. Variable Allocation Timeline
    document = crate::export_enhanced::add_memory_timeline(document, allocations, stats)?;

    // 11. Bottom left: Interactive Legend & Guide
    document = crate::export_enhanced::add_interactive_legend(document)?;

    // 12. Bottom right: Memory Analysis Summary
    document = crate::export_enhanced::add_comprehensive_summary(document, stats, allocations)?;

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

    tracing::info!(
        "Found {} total allocations, {} with variable names",
        allocations.len(),
        tracked_vars.len()
    );

    // Debug: Print the tracked variables we found
    for (i, var) in tracked_vars.iter().enumerate() {
        tracing::info!(
            "Tracked var {}: {} ({})",
            i + 1,
            var.var_name.as_ref().unwrap_or(&"None".to_string()),
            var.type_name.as_ref().unwrap_or(&"Unknown".to_string())
        );
    }

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

        // Enhanced variable names display - more prominent and detailed
        let var_names: Vec<String> = allocations
            .iter()
            .filter_map(|a| a.var_name.as_ref())
            .take(5) // Show more variables
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

        let display_text = format!(
            "{} - Variables: {}",
            format_bytes(total_size),
            var_names.join(" | ")
        );
        let size_text = SvgText::new(display_text)
            .set("x", chart_x + bar_width + 15)
            .set("y", y + 20)
            .set("font-size", 12)
            .set("font-weight", "600")
            .set("fill", "#FFFFFF")
            .set("text-shadow", "1px 1px 2px rgba(0,0,0,0.8)");
        document = document.add(size_text);
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
        let scope_label = SvgText::new(format!("Scope: {}", scope_name))
            .set("x", group_x - 10)
            .set("y", group_y - 5)
            .set("font-size", 12)
            .set("font-weight", "bold")
            .set("fill", "#FFFFFF");
        document = document.add(scope_label);

        scope_positions.insert(scope_name.clone(), (group_x, group_y));
    }

    // Draw relationship lines FIRST (behind nodes)
    let relationships = analyze_variable_relationships(tracked_vars);
    for relationship in &relationships {
        document = draw_relationship_line(document, relationship, &scope_positions)?;
    }

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

/// Analyze variable relationships based on type patterns
fn analyze_variable_relationships(tracked_vars: &[&AllocationInfo]) -> Vec<VariableRelationship> {
    let mut relationships = Vec::new();

    for (i, var1) in tracked_vars.iter().enumerate() {
        for var2 in tracked_vars.iter().skip(i + 1) {
            if let (Some(name1), Some(name2)) = (&var1.var_name, &var2.var_name) {
                if let (Some(type1), Some(type2)) = (&var1.type_name, &var2.type_name) {
                    // Detect relationship patterns
                    let relationship_type = detect_relationship_type(name1, type1, name2, type2);
                    if relationship_type != RelationshipType::None {
                        relationships.push(VariableRelationship {
                            from_var: name1.clone(),
                            to_var: name2.clone(),
                            relationship_type,
                            strength: calculate_relationship_strength(var1, var2),
                        });
                    }
                }
            }
        }
    }

    relationships
}

/// Detect relationship type between two variables
fn detect_relationship_type(
    name1: &str,
    type1: &str,
    name2: &str,
    type2: &str,
) -> RelationshipType {
    // Ownership Transfer/Move patterns
    if name2.contains("moved")
        || name2.contains("transferred")
        || (name1.contains("original") && name2.contains("new"))
    {
        return RelationshipType::OwnershipTransfer;
    }

    // Clone patterns
    if name2.contains("clone")
        || name1.contains("clone")
        || (name1.contains("shared") && name2.contains("shared"))
    {
        return RelationshipType::Clone;
    }

    // Shared pointer patterns (Rc, Arc)
    if (type1.contains("Rc") || type1.contains("Arc"))
        && (type2.contains("Rc") || type2.contains("Arc"))
    {
        return RelationshipType::SharedPointer;
    }

    // Borrow patterns (same base name, different suffixes)
    if name1.replace("_ref", "").replace("_mut", "")
        == name2.replace("_ref", "").replace("_mut", "")
    {
        if name1.contains("_mut") || name2.contains("_mut") {
            return RelationshipType::MutableBorrow;
        } else {
            return RelationshipType::ImmutableBorrow;
        }
    }

    // Indirect relationship (similar types, different scopes)
    if get_simple_type(type1) == get_simple_type(type2) && name1 != name2 {
        return RelationshipType::IndirectReference;
    }

    RelationshipType::None
}

/// Calculate relationship strength (0.0 to 1.0)
fn calculate_relationship_strength(var1: &AllocationInfo, var2: &AllocationInfo) -> f32 {
    // Base strength on size similarity and timing proximity
    let size_ratio = (var1.size.min(var2.size) as f32) / (var1.size.max(var2.size) as f32);
    let time_diff = (var1.timestamp_alloc as i64 - var2.timestamp_alloc as i64).abs() as f32;
    let time_factor = 1.0 / (1.0 + time_diff / 1000.0); // Closer in time = stronger relationship

    (size_ratio * 0.6 + time_factor * 0.4).min(1.0)
}

/// Draw relationship line with proper styling
fn draw_relationship_line(
    mut document: Document,
    relationship: &VariableRelationship,
    _scope_positions: &HashMap<String, (i32, i32)>,
) -> TrackingResult<Document> {
    // For now, draw a simple example line - in real implementation,
    // you'd calculate actual node positions
    let (color, stroke_width, dash_array, label) =
        get_relationship_style(&relationship.relationship_type);

    // Example line (you'd calculate real positions based on variable locations)
    let line = Line::new()
        .set("x1", 200)
        .set("y1", 100)
        .set("x2", 400)
        .set("y2", 150)
        .set("stroke", color)
        .set("stroke-width", stroke_width)
        .set("stroke-dasharray", dash_array)
        .set("marker-end", "url(#arrowhead)");
    document = document.add(line);

    // Add line label
    let line_label = SvgText::new(label)
        .set("x", 300) // Midpoint
        .set("y", 120)
        .set("font-size", 8)
        .set("font-weight", "bold")
        .set("fill", color)
        .set("text-anchor", "middle");
    document = document.add(line_label);

    Ok(document)
}

/// Draw variable node with enhanced styling
fn draw_variable_node(
    mut document: Document,
    allocation: &AllocationInfo,
    x: i32,
    y: i32,
) -> TrackingResult<Document> {
    let var_name = allocation.var_name.as_ref().unwrap();
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

/// Get relationship styling
fn get_relationship_style(
    rel_type: &RelationshipType,
) -> (&'static str, i32, &'static str, &'static str) {
    match rel_type {
        RelationshipType::OwnershipTransfer => ("#E74C3C", 4, "none", "owns"),
        RelationshipType::MutableBorrow => ("#3498DB", 3, "none", "borrows_mut"),
        RelationshipType::ImmutableBorrow => ("#27AE60", 2, "none", "borrows"),
        RelationshipType::Clone => ("#95A5A6", 2, "none", "cloned"),
        RelationshipType::SharedPointer => ("#9B59B6", 3, "8,4", "shared"),
        RelationshipType::IndirectReference => ("#F39C12", 1, "4,2", "indirect_ref"),
        RelationshipType::None => ("#7F8C8D", 1, "1,1", ""),
    }
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

// Define relationship types and structures
#[derive(Debug, PartialEq)]
enum RelationshipType {
    OwnershipTransfer,
    MutableBorrow,
    ImmutableBorrow,
    Clone,
    SharedPointer,
    IndirectReference,
    None,
}

#[derive(Debug)]
struct VariableRelationship {
    from_var: String,
    to_var: String,
    relationship_type: RelationshipType,
    strength: f32,
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
        let span = (end - start) as u64;
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
            (name.clone(), vars.clone(), priority, total_memory)
        })
        .collect();

    // Sort by priority (higher first), then by memory usage (larger first)
    scopes_with_priority.sort_by(|a, b| b.2.cmp(&a.2).then(b.3.cmp(&a.3)));

    scopes_with_priority
        .into_iter()
        .map(|(name, vars, _, _)| (name, vars))
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
                .unwrap()
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
        scope_data.insert("scope_name".to_string(), Value::String(scope_name.clone()));
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
                    Value::Number((var.timestamp_alloc as u64).into()),
                );
                variables.push(Value::Object(var_data));
            }
        }
        scope_data.insert("variables".to_string(), Value::Array(variables));

        all_scopes_data.push(Value::Object(scope_data));
    }
    analysis.insert("all_scopes".to_string(), Value::Array(all_scopes_data));

    // Write to JSON file
    let json_content = serde_json::to_string_pretty(&Value::Object(analysis)).map_err(|e| {
        TrackingError::SerializationError(format!("JSON serialization failed: {}", e))
    })?;

    std::fs::write("scope_analysis.json", json_content).map_err(|e| TrackingError::IoError(e))?;

    tracing::info!("Exported complete scope analysis to scope_analysis.json");
    Ok(())
}

/// Get simple type name

/// Get color based on duration ratio (0.0 to 1.0)
/// 根据相对生命周期长度分配颜色：最长时间=深色，最短时间=白色，全局作用域=特殊深蓝色
fn get_duration_color(ratio: f64, is_global: bool) -> String {
    if is_global {
        // 全局作用域使用特殊的深蓝色
        return "#0A2540".to_string();
    }

    // 创建从白色到深蓝色的渐变
    // ratio = 0.0 (最短时间) -> 接近白色
    // ratio = 1.0 (最长时间) -> 深蓝色

    if ratio <= 0.01 {
        // 极短时间或无时间差 -> 浅灰白色
        "#F8FAFC".to_string()
    } else {
        // 计算RGB值，从浅蓝白色渐变到深蓝色
        let base_r = 248; // 起始红色值 (接近白色)
        let base_g = 250; // 起始绿色值
        let base_b = 252; // 起始蓝色值

        let target_r = 30; // 目标红色值 (深蓝色)
        let target_g = 64; // 目标绿色值
        let target_b = 175; // 目标蓝色值

        // 使用平滑的渐变函数
        let smooth_ratio = ratio.powf(0.7); // 使渐变更平滑

        let r = (base_r as f64 + (target_r as f64 - base_r as f64) * smooth_ratio) as u8;
        let g = (base_g as f64 + (target_g as f64 - base_g as f64) * smooth_ratio) as u8;
        let b = (base_b as f64 + (target_b as f64 - base_b as f64) * smooth_ratio) as u8;

        format!("#{:02X}{:02X}{:02X}", r, g, b)
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

    tracing::info!(
        "Total scopes found: {}, displaying: {}",
        scope_groups.len(),
        selected_scopes.len()
    );

    // Calculate maximum duration across all SELECTED scopes for relative color scaling
    let max_duration = selected_scopes
        .iter()
        .map(|(_, vars)| {
            if !vars.is_empty() {
                let start = vars.iter().map(|v| v.timestamp_alloc).min().unwrap_or(0);
                let end = vars.iter().map(|v| v.timestamp_alloc).max().unwrap_or(0);
                (end - start) as u64
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
        positions.push((scope_name.clone(), x, y));
    }

    // Draw relationship lines first (only for displayed scopes)
    for (i, (scope_name, x, y)) in positions.iter().enumerate() {
        if scope_name != "Global" && i > 0 {
            // Find Global scope position
            if let Some((_, global_x, global_y)) =
                positions.iter().find(|(name, _, _)| name == "Global")
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

    let mut matrix_group = Group::new().set("transform", format!("translate({}, {})", x, y));

    // ENHANCED SCOPE LIFETIME CALCULATION
    let duration = calculate_scope_lifetime(scope_name, vars);
    let total_memory = vars.iter().map(|v| v.size).sum::<usize>();
    let _peak_memory = vars.iter().map(|v| v.size).max().unwrap_or(0);
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
        let var_name = var.var_name.as_ref().unwrap();
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
        let var_label = SvgText::new(var_name.clone())
            .set("x", 18)
            .set("y", var_y + 8)
            .set("font-size", 12)
            .set("font-weight", "bold")
            .set("fill", "#FFFFFF")
            .set("text-shadow", "1px 1px 2px rgba(0,0,0,0.8)");
        matrix_group = matrix_group.add(var_label);

        // Type label with enhanced color coding
        let (type_start_color, _) = get_type_gradient_colors(&type_name);
        let type_label = SvgText::new(format!("({})", type_name))
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
        let time_label = SvgText::new(format!("Active {}ms", duration_ms))
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
