//! Enhanced export functionality for memory tracking data.

use crate::core::tracker::MemoryTracker;
use crate::core::types::{AllocationInfo, MemoryStats, TrackingResult, TypeMemoryUsage};
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

/// Enhanced type information processing with variable names and inner type extraction
pub fn enhance_type_information(
    memory_by_type: &[TypeMemoryUsage],
    allocations: &[AllocationInfo],
) -> Vec<EnhancedTypeInfo> {
    let mut enhanced_types = Vec::new();
    let mut inner_type_stats: std::collections::HashMap<String, (usize, usize, Vec<String>)> =
        std::collections::HashMap::new();

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
                        Some(var_name.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .take(5) // Limit to 5 variable names
            .collect();

        // Add the main type with subcategory information
        enhanced_types.push(EnhancedTypeInfo {
            simplified_name: simplified_name,
            category: category,
            subcategory: subcategory,
            total_size: usage.total_size,
            allocation_count: usage.allocation_count,
            variable_names,
        });

        // Extract and accumulate inner types (e.g., i32 from Vec<i32>, u8 from Vec<u8>)
        extract_and_accumulate_inner_types_enhanced(
            &usage.type_name,
            usage.total_size,
            usage.allocation_count,
            &mut inner_type_stats,
        );
    }

    // Add accumulated inner types as separate entries
    for (inner_type, (total_size, allocation_count, var_names)) in inner_type_stats {
        let (simplified_name, category, subcategory) =
            analyze_type_with_detailed_subcategory(&inner_type);
        // tracing::info!("Adding inner type: '{}' -> '{}' ({}), size: {}, category: {}, subcategory: {}",
        //               inner_type, simplified_name, allocation_count, total_size, category, subcategory);
        enhanced_types.push(EnhancedTypeInfo {
            simplified_name,
            category,
            subcategory,
            total_size,
            allocation_count,
            variable_names: var_names.into_iter().take(5).collect(),
        });
    }

    // Debug: Print final enhanced types
    // tracing::info!("Final enhanced types count: {}", enhanced_types.len());
    // for (i, t) in enhanced_types.iter().enumerate() {
    //     tracing::info!("Type {}: {} ({} -> {}) - {} bytes", i, t.simplified_name, t.category, t.subcategory, t.total_size);
    // }

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

    // Collections analysis with precise subcategorization
    if clean_type.contains("Vec<") || clean_type.contains("vec::Vec") {
        let inner = extract_generic_inner_type(clean_type, "Vec");
        return (
            format!("Vec<{inner}>"),
            "Collections".to_string(),
            "Vec<T>".to_string(),
        );
    }

    if clean_type.contains("HashMap") || clean_type.contains("hash_map") {
        return (
            "HashMap<K,V>".to_string(),
            "Collections".to_string(),
            "HashMap<K,V>".to_string(),
        );
    }

    if clean_type.contains("HashSet") || clean_type.contains("hash_set") {
        return (
            "HashSet<T>".to_string(),
            "Collections".to_string(),
            "HashSet<T>".to_string(),
        );
    }

    if clean_type.contains("BTreeMap") || clean_type.contains("btree_map") {
        return (
            "BTreeMap<K,V>".to_string(),
            "Collections".to_string(),
            "BTreeMap<K,V>".to_string(),
        );
    }

    if clean_type.contains("BTreeSet") || clean_type.contains("btree_set") {
        return (
            "BTreeSet<T>".to_string(),
            "Collections".to_string(),
            "BTreeSet<T>".to_string(),
        );
    }

    if clean_type.contains("VecDeque") || clean_type.contains("vec_deque") {
        return (
            "VecDeque<T>".to_string(),
            "Collections".to_string(),
            "VecDeque<T>".to_string(),
        );
    }

    if clean_type.contains("LinkedList") {
        return (
            "LinkedList<T>".to_string(),
            "Collections".to_string(),
            "LinkedList<T>".to_string(),
        );
    }

    // Basic Types analysis with precise subcategorization
    if clean_type.contains("String") || clean_type.contains("string::String") {
        return (
            "String".to_string(),
            "Basic Types".to_string(),
            "Strings".to_string(),
        );
    }

    if clean_type.contains("&str") || clean_type == "str" {
        return (
            "&str".to_string(),
            "Basic Types".to_string(),
            "Strings".to_string(),
        );
    }

    // Integer types - exact matching
    if clean_type == "i32" || clean_type.ends_with("::i32") {
        return (
            "i32".to_string(),
            "Basic Types".to_string(),
            "Integers".to_string(),
        );
    }
    if clean_type == "i64" || clean_type.ends_with("::i64") {
        return (
            "i64".to_string(),
            "Basic Types".to_string(),
            "Integers".to_string(),
        );
    }
    if clean_type == "u32" || clean_type.ends_with("::u32") {
        return (
            "u32".to_string(),
            "Basic Types".to_string(),
            "Integers".to_string(),
        );
    }
    if clean_type == "u64" || clean_type.ends_with("::u64") {
        return (
            "u64".to_string(),
            "Basic Types".to_string(),
            "Integers".to_string(),
        );
    }
    if clean_type == "usize" || clean_type.ends_with("::usize") {
        return (
            "usize".to_string(),
            "Basic Types".to_string(),
            "Integers".to_string(),
        );
    }
    if clean_type == "isize" || clean_type.ends_with("::isize") {
        return (
            "isize".to_string(),
            "Basic Types".to_string(),
            "Integers".to_string(),
        );
    }
    if clean_type == "i8" || clean_type.ends_with("::i8") {
        return (
            "i8".to_string(),
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
    if clean_type == "i16" || clean_type.ends_with("::i16") {
        return (
            "i16".to_string(),
            "Basic Types".to_string(),
            "Integers".to_string(),
        );
    }
    if clean_type == "u16" || clean_type.ends_with("::u16") {
        return (
            "u16".to_string(),
            "Basic Types".to_string(),
            "Integers".to_string(),
        );
    }

    // Float types
    if clean_type == "f32" || clean_type.ends_with("::f32") {
        return (
            "f32".to_string(),
            "Basic Types".to_string(),
            "Floats".to_string(),
        );
    }
    if clean_type == "f64" || clean_type.ends_with("::f64") {
        return (
            "f64".to_string(),
            "Basic Types".to_string(),
            "Floats".to_string(),
        );
    }

    // Other basic types
    if clean_type == "bool" || clean_type.ends_with("::bool") {
        return (
            "bool".to_string(),
            "Basic Types".to_string(),
            "Booleans".to_string(),
        );
    }
    if clean_type == "char" || clean_type.ends_with("::char") {
        return (
            "char".to_string(),
            "Basic Types".to_string(),
            "Characters".to_string(),
        );
    }

    // Smart Pointers
    if clean_type.contains("Box<") {
        let inner = extract_generic_inner_type(clean_type, "Box");
        return (
            format!("Box<{inner}>"),
            "Smart Pointers".to_string(),
            "Box<T>".to_string(),
        );
    }

    if clean_type.contains("Rc<") {
        let inner = extract_generic_inner_type(clean_type, "Rc");
        return (
            format!("Rc<{inner}>"),
            "Smart Pointers".to_string(),
            "Rc<T>".to_string(),
        );
    }

    if clean_type.contains("Arc<") {
        let inner = extract_generic_inner_type(clean_type, "Arc");
        return (
            format!("Arc<{inner}>"),
            "Smart Pointers".to_string(),
            "Arc<T>".to_string(),
        );
    }

    // Fall back to original logic for other types
    let (simplified_name, category) = simplify_type_name(clean_type);
    (simplified_name, category, "Other".to_string())
}

/// Extract generic inner type for display
fn extract_generic_inner_type(type_name: &str, container: &str) -> String {
    if let Some(start) = type_name.find(&format!("{container}<")) {
        let start = start + container.len() + 1;
        if let Some(end) = type_name[start..].rfind('>') {
            let inner = &type_name[start..start + end];
            return inner.split("::").last().unwrap_or(inner).to_string();
        }
    }
    "?".to_string()
}

/// Extract inner types from complex types and accumulate their statistics (enhanced version)
fn extract_and_accumulate_inner_types_enhanced(
    type_name: &str,
    size: usize,
    count: usize,
    stats: &mut std::collections::HashMap<String, (usize, usize, Vec<String>)>,
) {
    // Extract inner types from generic containers
    let inner_types = extract_inner_primitive_types_enhanced(type_name);

    // Debug: Print what we found
    // if !inner_types.is_empty() {
    // tracing::info!("Found inner types in '{}': {:?}", type_name, inner_types);
    // }

    for inner_type in inner_types {
        let entry = stats.entry(inner_type).or_insert((0, 0, Vec::new()));
        entry.0 += size / 4; // Rough estimation of inner type contribution
        entry.1 += count;
        entry.2.push(format!("from {type_name}"));
        // tracing::info!("Accumulated {} bytes for inner type '{}' from '{}'", size / 4, inner_type, type_name);
    }
}

/// Extract primitive types from complex type signatures (enhanced version)
fn extract_inner_primitive_types_enhanced(type_name: &str) -> Vec<String> {
    let mut inner_types = Vec::new();

    // Common primitive type patterns
    let primitives = [
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64", "bool", "char",
    ];

    for primitive in &primitives {
        if type_name.contains(primitive) {
            // Make sure it's actually the primitive type, not part of another word
            if type_name.contains(&format!("{primitive}>"))
                || type_name.contains(&format!("{primitive},"))
                || type_name.contains(&format!(" {primitive}"))
                || type_name.ends_with(primitive)
            {
                inner_types.push(primitive.to_string());
            }
        }
    }

    inner_types
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
        let category_name_clone = category_name.clone();

        let category =
            categories
                .entry(category_name.clone())
                .or_insert_with(|| AllocationCategory {
                    name: category_name_clone,
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

/// Categorize enhanced type information for consistent visualization
/// This ensures "Tracked Variables by Category" uses the same data as "Memory Usage by Type"
pub fn categorize_enhanced_allocations(
    enhanced_types: &[EnhancedTypeInfo],
) -> Vec<AllocationCategory> {
    let mut categories: HashMap<String, AllocationCategory> = HashMap::new();

    for enhanced_type in enhanced_types {
        // Skip unknown types
        if enhanced_type.simplified_name == "Unknown" {
            continue;
        }

        let category_name = &enhanced_type.category;

        let category = categories
            .entry(category_name.to_string())
            .or_insert_with(|| AllocationCategory {
                name: category_name.to_string(),
                allocations: Vec::new(),
                total_size: 0,
                color: get_category_color(category_name),
            });

        // Create synthetic allocation info for display
        let mut synthetic_allocation = AllocationInfo::new(0, enhanced_type.total_size);
        synthetic_allocation.var_name = Some(enhanced_type.variable_names.join(", "));
        synthetic_allocation.type_name = Some(enhanced_type.simplified_name.clone());

        category.allocations.push(synthetic_allocation);
        category.total_size += enhanced_type.total_size;
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
    /// Subcategory for fine-grained classification
    pub subcategory: String,
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
        (
            "Active Memory",
            format_bytes(active_memory),
            (active_memory as f64 / peak_memory.max(1) as f64 * 100.0).min(100.0),
            "#3498db",
        ),
        (
            "Peak Memory",
            {
                let formatted = format_bytes(peak_memory);
                tracing::info!(
                    "SVG add_enhanced_header - Formatting peak_memory: {} bytes -> {}",
                    peak_memory,
                    formatted
                );
                formatted
            },
            100.0,
            "#e74c3c",
        ),
        (
            "Active Allocs",
            format!("{active_allocations}"),
            (active_allocations as f64 / 1000.0 * 100.0).min(100.0),
            "#2ecc71",
        ),
        (
            "Reclamation",
            format!("{memory_reclamation_rate:.1}%"),
            memory_reclamation_rate,
            "#f39c12",
        ),
        (
            "Efficiency",
            format!("{allocator_efficiency:.1}%"),
            allocator_efficiency,
            "#9b59b6",
        ),
        (
            "Median Size",
            format_bytes(median_alloc_size),
            (median_alloc_size as f64 / 1024.0 * 100.0).min(100.0),
            "#1abc9c",
        ),
        (
            "P95 Size",
            format_bytes(p95_alloc_size),
            (p95_alloc_size as f64 / 4096.0 * 100.0).min(100.0),
            "#e67e22",
        ),
        (
            "Fragmentation",
            format!("{memory_fragmentation:.1}%"),
            memory_fragmentation,
            "#95a5a6",
        ),
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
            .set(
                "stroke-dasharray",
                format!("{circumference} {circumference}"),
            )
            .set("stroke-dashoffset", progress_offset)
            .set(
                "transform",
                format!("rotate(-90 {ring_center_x} {ring_center_y})"),
            )
            .set("style", "transition: stroke-dashoffset 0.5s ease;");
        document = document.add(progress_ring);

        // Percentage text in center of ring
        let percent_text = SvgText::new(format!("{percentage:.0}%"))
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
            "#e74c3c" // Red for high values
        } else if *percentage >= 50.0 {
            "#f39c12" // Orange for medium values
        } else {
            "#27ae60" // Green for low values
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

/// Add enhanced treemap chart with real data
pub fn add_enhanced_type_chart(
    mut document: Document,
    types: &[EnhancedTypeInfo],
) -> TrackingResult<Document> {
    // Optimized position to avoid overlap with other modules
    let chart_x = 50;
    let chart_y = 320; // Moved up significantly after removing timeline dashboard
    let chart_width = 850;
    let chart_height = 300; // Reduced height to prevent overlap

    // Chart title
    let title = SvgText::new("Memory Usage by Type - Treemap Visualization")
        .set("x", chart_x + chart_width / 2)
        .set("y", chart_y - 10)
        .set("text-anchor", "middle")
        .set("font-size", 16)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");

    document = document.add(title);

    // Add treemap styles
    let styles = svg::node::element::Style::new(
        r#"
        .integrated-treemap-rect { 
            transition: all 0.3s ease; 
            cursor: pointer; 
            stroke: #ffffff; 
            stroke-width: 2; 
        }
        .integrated-treemap-rect:hover { 
            stroke: #2c3e50; 
            stroke-width: 3; 
            filter: brightness(1.1); 
        }
        .integrated-treemap-label { 
            fill: #ffffff; 
            font-weight: 700; 
            text-anchor: middle; 
            dominant-baseline: middle; 
            pointer-events: none;
            text-shadow: 1px 1px 2px rgba(0,0,0,0.8);
        }
        .integrated-treemap-percentage { 
            fill: #f8f9fa; 
            font-weight: 600;
            text-anchor: middle; 
            dominant-baseline: middle; 
            pointer-events: none;
            text-shadow: 1px 1px 2px rgba(0,0,0,0.6);
        }
    "#,
    );
    document = document.add(styles);

    // Create integrated treemap layout
    let treemap_area = IntegratedTreemapArea {
        x: chart_x as f64,
        y: chart_y as f64,
        width: chart_width as f64,
        height: chart_height as f64,
    };

    // Build and render real data treemap
    document = render_real_data_treemap(document, treemap_area, types)?;

    // Note: Legend is now integrated within the treemap rendering, no separate legend needed

    Ok(document)
}

/// Integrated treemap area structure
#[derive(Debug, Clone)]
struct IntegratedTreemapArea {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// Render integrated treemap with real data - placeholder function
/// Render treemap with real memory data matching task.md layout
fn render_real_data_treemap(
    mut document: Document,
    area: IntegratedTreemapArea,
    types: &[EnhancedTypeInfo],
) -> TrackingResult<Document> {
    if types.is_empty() {
        let no_data_rect = Rectangle::new()
            .set("x", area.x)
            .set("y", area.y)
            .set("width", area.width)
            .set("height", area.height)
            .set("fill", "#f8f9fa")
            .set("stroke", "#dee2e6")
            .set("stroke-width", 2)
            .set("rx", 10);
        document = document.add(no_data_rect);

        let no_data_text = SvgText::new("No Memory Type Data Available")
            .set("x", area.x + area.width / 2.0)
            .set("y", area.y + area.height / 2.0)
            .set("text-anchor", "middle")
            .set("font-size", 16)
            .set("font-weight", "bold")
            .set("fill", "#6c757d");
        document = document.add(no_data_text);

        return Ok(document);
    }

    // Calculate total memory for percentage calculations
    let total_memory: usize = types.iter().map(|t| t.total_size).sum();
    if total_memory == 0 {
        return Ok(document);
    }

    // Group types by category to match task.md structure
    let mut collections_types = Vec::new();
    let mut basic_types = Vec::new();
    let mut smart_pointers_types = Vec::new();
    let mut other_types = Vec::new();

    for type_info in types {
        match type_info.category.as_str() {
            "Collections" => collections_types.push(type_info),
            "Basic Types" => basic_types.push(type_info),
            "Strings" => basic_types.push(type_info), // Legacy support - redirect to Basic Types
            "Smart Pointers" => smart_pointers_types.push(type_info),
            _ => other_types.push(type_info),
        }
    }

    // Calculate category totals
    let _collections_total: usize = collections_types.iter().map(|t| t.total_size).sum();
    let _basic_types_total: usize = basic_types.iter().map(|t| t.total_size).sum();
    let _smart_pointers_total: usize = smart_pointers_types.iter().map(|t| t.total_size).sum();
    let _other_total: usize = other_types.iter().map(|t| t.total_size).sum();

    // Analyze data distribution for intelligent layout decision
    let layout_strategy = analyze_data_distribution(
        &collections_types,
        &basic_types,
        &smart_pointers_types,
        &other_types,
        total_memory,
    );

    // Build treemap structure based on intelligent analysis
    let treemap_data = build_adaptive_treemap_data(
        collections_types,
        basic_types,
        smart_pointers_types,
        other_types,
        total_memory,
        &layout_strategy,
    );

    // Render treemap using adaptive algorithm
    document = render_adaptive_treemap(document, area, &treemap_data, &layout_strategy)?;

    Ok(document)
}

/// Treemap data structure for hierarchical visualization
#[derive(Debug, Clone)]
struct TreemapNode {
    name: String,
    size: usize,
    percentage: f64,
    color: String,
    children: Vec<TreemapNode>,
}

/// Layout strategy for treemap rendering
#[derive(Debug, Clone)]
enum TreemapLayoutStrategy {
    /// Full treemap with all categories and subcategories
    FullLayout,
    /// Simplified layout focusing on dominant category
    DominantCategoryLayout { dominant_category: String },
    /// Minimal layout for simple programs
    MinimalLayout,
    /// Collections-only layout
    CollectionsOnlyLayout,
    /// Basic types only layout
    BasicTypesOnlyLayout,
}

/// Analyze data distribution to determine optimal layout strategy
fn analyze_data_distribution(
    collections_types: &[&EnhancedTypeInfo],
    basic_types: &[&EnhancedTypeInfo],
    smart_pointers_types: &[&EnhancedTypeInfo],
    _other_types: &[&EnhancedTypeInfo],
    total_memory: usize,
) -> TreemapLayoutStrategy {
    let collections_total: usize = collections_types.iter().map(|t| t.total_size).sum();
    let basic_types_total: usize = basic_types.iter().map(|t| t.total_size).sum();
    let smart_pointers_total: usize = smart_pointers_types.iter().map(|t| t.total_size).sum();

    let collections_percentage = if total_memory > 0 {
        (collections_total as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };
    let basic_types_percentage = if total_memory > 0 {
        (basic_types_total as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };
    let smart_pointers_percentage = if total_memory > 0 {
        (smart_pointers_total as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };

    // Count non-zero categories
    let active_categories = [
        (collections_total > 0, "Collections"),
        (basic_types_total > 0, "Basic Types"),
        (smart_pointers_total > 0, "Smart Pointers"),
    ]
    .iter()
    .filter(|(active, _)| *active)
    .count();

    match active_categories {
        0 => TreemapLayoutStrategy::MinimalLayout,
        1 => {
            // Single dominant category
            if collections_percentage > 80.0 {
                TreemapLayoutStrategy::CollectionsOnlyLayout
            } else if basic_types_percentage > 80.0 {
                TreemapLayoutStrategy::BasicTypesOnlyLayout
            } else {
                TreemapLayoutStrategy::DominantCategoryLayout {
                    dominant_category: if collections_total > basic_types_total
                        && collections_total > smart_pointers_total
                    {
                        "Collections".to_string()
                    } else if basic_types_total > smart_pointers_total {
                        "Basic Types".to_string()
                    } else {
                        "Smart Pointers".to_string()
                    },
                }
            }
        }
        2 => {
            // Two categories - use simplified layout
            if collections_percentage > 70.0
                || basic_types_percentage > 70.0
                || smart_pointers_percentage > 70.0
            {
                TreemapLayoutStrategy::DominantCategoryLayout {
                    dominant_category: if collections_total > basic_types_total
                        && collections_total > smart_pointers_total
                    {
                        "Collections".to_string()
                    } else if basic_types_total > smart_pointers_total {
                        "Basic Types".to_string()
                    } else {
                        "Smart Pointers".to_string()
                    },
                }
            } else {
                TreemapLayoutStrategy::FullLayout
            }
        }
        _ => TreemapLayoutStrategy::FullLayout, // 3+ categories - use full layout
    }
}

/// Build adaptive treemap data structure based on layout strategy
fn build_adaptive_treemap_data(
    collections_types: Vec<&EnhancedTypeInfo>,
    basic_types: Vec<&EnhancedTypeInfo>,
    smart_pointers_types: Vec<&EnhancedTypeInfo>,
    _other_types: Vec<&EnhancedTypeInfo>,
    total_memory: usize,
    strategy: &TreemapLayoutStrategy,
) -> Vec<TreemapNode> {
    let mut treemap_nodes = Vec::new();

    match strategy {
        TreemapLayoutStrategy::MinimalLayout => {
            // Show a simple "No significant data" message
            treemap_nodes.push(TreemapNode {
                name: "No Significant Memory Usage".to_string(),
                size: total_memory.max(1),
                percentage: 100.0,
                color: "#95a5a6".to_string(),
                children: Vec::new(),
            });
        }
        TreemapLayoutStrategy::CollectionsOnlyLayout => {
            // Show only Collections with detailed subcategories
            let collections_total: usize = collections_types.iter().map(|t| t.total_size).sum();
            if collections_total > 0 {
                treemap_nodes.push(build_collections_node(&collections_types, total_memory));
            }
        }
        TreemapLayoutStrategy::BasicTypesOnlyLayout => {
            // Show only Basic Types with detailed subcategories
            let basic_types_total: usize = basic_types.iter().map(|t| t.total_size).sum();
            if basic_types_total > 0 {
                treemap_nodes.push(build_basic_types_node(&basic_types, total_memory));
            }
        }
        TreemapLayoutStrategy::DominantCategoryLayout { dominant_category } => {
            // Show dominant category with details, others simplified
            match dominant_category.as_str() {
                "Collections" => {
                    let collections_total: usize =
                        collections_types.iter().map(|t| t.total_size).sum();
                    if collections_total > 0 {
                        treemap_nodes
                            .push(build_collections_node(&collections_types, total_memory));
                    }
                    // Add other categories as simple nodes
                    add_simple_categories(
                        &mut treemap_nodes,
                        &basic_types,
                        &smart_pointers_types,
                        total_memory,
                    );
                }
                "Basic Types" => {
                    let basic_types_total: usize = basic_types.iter().map(|t| t.total_size).sum();
                    if basic_types_total > 0 {
                        treemap_nodes.push(build_basic_types_node(&basic_types, total_memory));
                    }
                    // Add other categories as simple nodes
                    add_simple_categories_alt(
                        &mut treemap_nodes,
                        &collections_types,
                        &smart_pointers_types,
                        total_memory,
                    );
                }
                _ => {
                    // Smart Pointers dominant - use full layout
                    build_full_layout(
                        &mut treemap_nodes,
                        &collections_types,
                        &basic_types,
                        &smart_pointers_types,
                        total_memory,
                    );
                }
            }
        }
        TreemapLayoutStrategy::FullLayout => {
            // Show all categories with full details
            build_full_layout(
                &mut treemap_nodes,
                &collections_types,
                &basic_types,
                &smart_pointers_types,
                total_memory,
            );
        }
    }

    treemap_nodes
}

/// Build Collections node with comprehensive subcategories using enhanced type info
fn build_collections_node(
    collections_types: &[&EnhancedTypeInfo],
    total_memory: usize,
) -> TreemapNode {
    let collections_total: usize = collections_types.iter().map(|t| t.total_size).sum();

    // Debug: Print what collections we received
    // tracing::info!("build_collections_node received {} types, total: {} bytes", collections_types.len(), collections_total);
    // 2

    if collections_total == 0 {
        return TreemapNode {
            name: "Collections".to_string(),
            size: 1,
            percentage: 0.0,
            color: "#ecf0f1".to_string(),
            children: Vec::new(),
        };
    }

    // Group by subcategory for more accurate classification
    let mut subcategory_groups: std::collections::HashMap<String, Vec<&EnhancedTypeInfo>> =
        std::collections::HashMap::new();

    for collection_type in collections_types {
        subcategory_groups
            .entry(collection_type.subcategory.clone())
            .or_default()
            .push(collection_type);
    }

    let mut collections_children = Vec::new();

    // Define colors for each subcategory
    let subcategory_colors = [
        ("Vec<T>", "#e74c3c"),
        ("HashMap<K,V>", "#3498db"),
        ("HashSet<T>", "#9b59b6"),
        ("BTreeMap<K,V>", "#2ecc71"),
        ("BTreeSet<T>", "#27ae60"),
        ("VecDeque<T>", "#f39c12"),
        ("LinkedList<T>", "#e67e22"),
        ("Other", "#95a5a6"),
    ]
    .iter()
    .cloned()
    .collect::<std::collections::HashMap<&str, &str>>();

    for (subcategory, types) in subcategory_groups {
        let category_total: usize = types.iter().map(|t| t.total_size).sum();

        // Debug: Print subcategory details
        // tracing::info!("Collections Subcategory '{}': {} types, total: {} bytes",
        //               subcategory, types.len(), category_total);
        // for t in &types {
        //     tracing::info!("  - '{}' - {} bytes", t.simplified_name, t.total_size);
        // }

        if category_total > 0 {
            let relative_percentage = (category_total as f64 / collections_total as f64) * 100.0;
            let color = subcategory_colors
                .get(subcategory.as_str())
                .unwrap_or(&"#95a5a6")
                .to_string();

            collections_children.push(TreemapNode {
                name: format!("{subcategory} ({relative_percentage:.1}%)"),
                size: category_total,
                percentage: (category_total as f64 / total_memory as f64) * 100.0,
                color,
                children: Vec::new(),
            });
        }
    }

    // Sort children by size (largest first) for better visual hierarchy
    collections_children.sort_by(|a, b| b.size.cmp(&a.size));

    TreemapNode {
        name: format!(
            "Collections ({:.1}%)",
            (collections_total as f64 / total_memory as f64) * 100.0
        ),
        size: collections_total,
        percentage: (collections_total as f64 / total_memory as f64) * 100.0,
        color: "#ecf0f1".to_string(),
        children: collections_children,
    }
}

/// Build Basic Types node with comprehensive subcategories using enhanced type info
fn build_basic_types_node(basic_types: &[&EnhancedTypeInfo], total_memory: usize) -> TreemapNode {
    let basic_types_total: usize = basic_types.iter().map(|t| t.total_size).sum();

    // Debug: Print what basic types we received
    // tracing::info!("build_basic_types_node received {} types, total: {} bytes", basic_types.len(), basic_types_total);
    // for (i, bt) in basic_types.iter().enumerate() {
    //     tracing::info!("  Basic type {}: '{}' (subcategory: '{}') - {} bytes", i, bt.simplified_name, bt.subcategory, bt.total_size);
    // }

    if basic_types_total == 0 {
        return TreemapNode {
            name: "Basic Types".to_string(),
            size: 1,
            percentage: 0.0,
            color: "#ecf0f1".to_string(),
            children: Vec::new(),
        };
    }

    // Group by subcategory for more accurate classification
    let mut subcategory_groups: std::collections::HashMap<String, Vec<&EnhancedTypeInfo>> =
        std::collections::HashMap::new();

    for basic_type in basic_types {
        subcategory_groups
            .entry(basic_type.subcategory.clone())
            .or_default()
            .push(basic_type);
    }

    let mut basic_types_children = Vec::new();

    // Define colors for each subcategory
    let subcategory_colors = [
        ("Strings", "#2ecc71"),
        ("Integers", "#3498db"),
        ("Floats", "#e74c3c"),
        ("Booleans", "#f39c12"),
        ("Characters", "#9b59b6"),
        ("Arrays", "#1abc9c"),
        ("References", "#e67e22"),
        ("Other", "#95a5a6"),
    ]
    .iter()
    .cloned()
    .collect::<std::collections::HashMap<&str, &str>>();

    for (subcategory, types) in subcategory_groups {
        let category_total: usize = types.iter().map(|t| t.total_size).sum();

        // Debug: Print subcategory details
        // tracing::info!("Basic Types Subcategory '{}': {} types, total: {} bytes",
        //               subcategory, types.len(), category_total);
        // for t in &types {
        //     tracing::info!("  - '{}' - {} bytes", t.simplified_name, t.total_size);
        // }

        if category_total > 0 {
            let relative_percentage = (category_total as f64 / basic_types_total as f64) * 100.0;
            let color = subcategory_colors
                .get(subcategory.as_str())
                .unwrap_or(&"#95a5a6")
                .to_string();

            basic_types_children.push(TreemapNode {
                name: format!("{subcategory} ({relative_percentage:.1}%)"),
                size: category_total,
                percentage: (category_total as f64 / total_memory as f64) * 100.0,
                color,
                children: Vec::new(),
            });
        }
    }

    // Sort children by size (largest first) for better visual hierarchy
    basic_types_children.sort_by(|a, b| b.size.cmp(&a.size));

    // Debug: Print final children
    // tracing::info!("Basic Types node will have {} children:", basic_types_children.len());
    // for (i, child) in basic_types_children.iter().enumerate() {
    //     tracing::info!("  Child {}: '{}' - {} bytes", i, child.name, child.size);
    // }

    TreemapNode {
        name: format!(
            "Basic Types ({:.1}%)",
            (basic_types_total as f64 / total_memory as f64) * 100.0
        ),
        size: basic_types_total,
        percentage: (basic_types_total as f64 / total_memory as f64) * 100.0,
        color: "#ecf0f1".to_string(),
        children: basic_types_children,
    }
}

/// Add simple category nodes without subcategories
fn add_simple_categories(
    treemap_nodes: &mut Vec<TreemapNode>,
    basic_types: &[&EnhancedTypeInfo],
    smart_pointers_types: &[&EnhancedTypeInfo],
    total_memory: usize,
) {
    let basic_types_total: usize = basic_types.iter().map(|t| t.total_size).sum();
    if basic_types_total > 0 {
        treemap_nodes.push(TreemapNode {
            name: "Basic Types".to_string(),
            size: basic_types_total,
            percentage: (basic_types_total as f64 / total_memory as f64) * 100.0,
            color: "#f39c12".to_string(),
            children: Vec::new(),
        });
    }

    let smart_pointers_total: usize = smart_pointers_types.iter().map(|t| t.total_size).sum();
    if smart_pointers_total > 0 {
        treemap_nodes.push(TreemapNode {
            name: "Smart Pointers".to_string(),
            size: smart_pointers_total,
            percentage: (smart_pointers_total as f64 / total_memory as f64) * 100.0,
            color: "#9b59b6".to_string(),
            children: Vec::new(),
        });
    }
}

/// Add simple category nodes (alternative version)
fn add_simple_categories_alt(
    treemap_nodes: &mut Vec<TreemapNode>,
    collections_types: &[&EnhancedTypeInfo],
    smart_pointers_types: &[&EnhancedTypeInfo],
    total_memory: usize,
) {
    let collections_total: usize = collections_types.iter().map(|t| t.total_size).sum();
    if collections_total > 0 {
        treemap_nodes.push(TreemapNode {
            name: "Collections".to_string(),
            size: collections_total,
            percentage: (collections_total as f64 / total_memory as f64) * 100.0,
            color: "#3498db".to_string(),
            children: Vec::new(),
        });
    }

    let smart_pointers_total: usize = smart_pointers_types.iter().map(|t| t.total_size).sum();
    if smart_pointers_total > 0 {
        treemap_nodes.push(TreemapNode {
            name: "Smart Pointers".to_string(),
            size: smart_pointers_total,
            percentage: (smart_pointers_total as f64 / total_memory as f64) * 100.0,
            color: "#9b59b6".to_string(),
            children: Vec::new(),
        });
    }
}

/// Build full layout with all categories
fn build_full_layout(
    treemap_nodes: &mut Vec<TreemapNode>,
    collections_types: &[&EnhancedTypeInfo],
    basic_types: &[&EnhancedTypeInfo],
    smart_pointers_types: &[&EnhancedTypeInfo],
    total_memory: usize,
) {
    let collections_total: usize = collections_types.iter().map(|t| t.total_size).sum();
    if collections_total > 0 {
        treemap_nodes.push(build_collections_node(collections_types, total_memory));
    }

    let basic_types_total: usize = basic_types.iter().map(|t| t.total_size).sum();
    if basic_types_total > 0 {
        treemap_nodes.push(build_basic_types_node(basic_types, total_memory));
    }

    let smart_pointers_total: usize = smart_pointers_types.iter().map(|t| t.total_size).sum();
    if smart_pointers_total > 0 {
        treemap_nodes.push(TreemapNode {
            name: "Smart Pointers".to_string(),
            size: smart_pointers_total,
            percentage: (smart_pointers_total as f64 / total_memory as f64) * 100.0,
            color: "#9b59b6".to_string(),
            children: Vec::new(),
        });
    }
}

/// Render adaptive treemap based on layout strategy
fn render_adaptive_treemap(
    document: Document,
    area: IntegratedTreemapArea,
    treemap_data: &[TreemapNode],
    _strategy: &TreemapLayoutStrategy,
) -> TrackingResult<Document> {
    // Use the existing render_squarified_treemap function
    render_squarified_treemap(document, area, treemap_data)
}

/// Render intelligent horizontal-band treemap layout (like basic_usage_graph.svg)
fn render_squarified_treemap(
    mut document: Document,
    area: IntegratedTreemapArea,
    nodes: &[TreemapNode],
) -> TrackingResult<Document> {
    if nodes.is_empty() {
        return render_empty_treemap(document, area);
    }

    // Container background
    let container_bg = Rectangle::new()
        .set("class", "integrated-treemap-container")
        .set("x", area.x)
        .set("y", area.y)
        .set("width", area.width)
        .set("height", area.height)
        .set("fill", "#ecf0f1")
        .set("stroke", "#bdc3c7")
        .set("stroke-width", 2)
        .set("rx", 20);
    document = document.add(container_bg);

    // Calculate total size for proportional height allocation
    let total_size: usize = nodes.iter().map(|n| n.size).sum();
    if total_size == 0 {
        return render_empty_treemap(document, area);
    }

    // Layout parameters
    let padding = 15.0;
    let band_spacing = 10.0;
    let available_width = area.width - (padding * 2.0);
    let available_height =
        area.height - (padding * 2.0) - (band_spacing * (nodes.len() - 1) as f64);

    let mut current_y = area.y + padding;

    // Sort nodes by size (largest first) for better visual hierarchy
    let mut sorted_nodes = nodes.to_vec();
    sorted_nodes.sort_by(|a, b| b.size.cmp(&a.size));

    for node in &sorted_nodes {
        // Calculate proportional height for this band
        let band_height = (node.size as f64 / total_size as f64) * available_height;

        // Render the band based on whether it has children
        if !node.children.is_empty() {
            document = render_smart_horizontal_band(
                document,
                area.x + padding,
                current_y,
                available_width,
                band_height,
                node,
            )?;
        } else {
            document = render_simple_horizontal_band(
                document,
                area.x + padding,
                current_y,
                available_width,
                band_height,
                node,
            )?;
        }

        current_y += band_height + band_spacing;
    }

    // Note: No separate legend needed - information is embedded in treemap labels

    Ok(document)
}

/// Render empty treemap when no data available
fn render_empty_treemap(
    mut document: Document,
    area: IntegratedTreemapArea,
) -> TrackingResult<Document> {
    let container_bg = Rectangle::new()
        .set("class", "integrated-treemap-container")
        .set("x", area.x)
        .set("y", area.y)
        .set("width", area.width)
        .set("height", area.height)
        .set("fill", "#f8f9fa")
        .set("stroke", "#dee2e6")
        .set("stroke-width", 2)
        .set("rx", 20);
    document = document.add(container_bg);

    let no_data_text = SvgText::new("No Memory Type Data Available")
        .set("x", area.x + area.width / 2.0)
        .set("y", area.y + area.height / 2.0)
        .set("text-anchor", "middle")
        .set("font-size", 16)
        .set("font-weight", "bold")
        .set("fill", "#6c757d");
    document = document.add(no_data_text);

    Ok(document)
}

/// Render horizontal band with children (Collections, Basic Types)
fn render_smart_horizontal_band(
    mut document: Document,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    node: &TreemapNode,
) -> TrackingResult<Document> {
    // Band background
    let band_bg = Rectangle::new()
        .set("class", "integrated-treemap-rect")
        .set("x", x)
        .set("y", y)
        .set("width", width)
        .set("height", height)
        .set("fill", node.color.as_str())
        .set("rx", 18);
    document = document.add(band_bg);

    // Band title
    let title_y = y + 20.0;
    let title = SvgText::new(format!("{} - {:.0}%", node.name, node.percentage))
        .set("class", "integrated-treemap-label")
        .set("x", x + width / 2.0)
        .set("y", title_y)
        .set("font-size", 16);
    document = document.add(title);

    // Children layout area
    let children_y = title_y + 10.0;
    let children_height = height - 30.0;
    let children_padding = 10.0;
    let available_children_width = width - (children_padding * 2.0);

    // Calculate total children size for proportional width allocation
    let total_children_size: usize = node.children.iter().map(|c| c.size).sum();
    if total_children_size == 0 {
        return Ok(document);
    }

    let mut current_x = x + children_padding;

    for child in &node.children {
        // Calculate proportional width for this child
        let child_width =
            (child.size as f64 / total_children_size as f64) * available_children_width;

        // Child rectangle
        let child_rect = Rectangle::new()
            .set("class", "integrated-treemap-rect")
            .set("x", current_x)
            .set("y", children_y)
            .set("width", child_width - 5.0) // Small gap between children
            .set("height", children_height - 10.0)
            .set("fill", child.color.as_str())
            .set("rx", 18);
        document = document.add(child_rect);

        // Child label (extract percentage from name if present)
        let (child_name, child_relative_percentage) = extract_name_and_percentage(&child.name);

        let child_label = SvgText::new(child_name)
            .set("class", "integrated-treemap-label")
            .set("x", current_x + child_width / 2.0)
            .set("y", children_y + children_height / 2.0 - 5.0)
            .set("font-size", calculate_font_size(child_width))
            .set("font-weight", "bold");
        document = document.add(child_label);

        // Child percentage
        let percentage_text = if let Some(pct) = child_relative_percentage {
            format!("({pct})")
        } else {
            format!(
                "({:.0}%)",
                (child.size as f64 / total_children_size as f64) * 100.0
            )
        };

        let child_percentage = SvgText::new(percentage_text)
            .set("class", "integrated-treemap-percentage")
            .set("x", current_x + child_width / 2.0)
            .set("y", children_y + children_height / 2.0 + 15.0)
            .set("font-size", calculate_font_size(child_width) - 2);
        document = document.add(child_percentage);

        current_x += child_width;
    }

    Ok(document)
}

/// Render simple horizontal band without children (Smart Pointers)
fn render_simple_horizontal_band(
    mut document: Document,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    node: &TreemapNode,
) -> TrackingResult<Document> {
    // Simple band rectangle
    let band_rect = Rectangle::new()
        .set("class", "integrated-treemap-rect")
        .set("x", x)
        .set("y", y)
        .set("width", width)
        .set("height", height)
        .set("fill", node.color.as_str())
        .set("rx", 18);
    document = document.add(band_rect);

    // Band label
    let label = SvgText::new(&node.name)
        .set("class", "integrated-treemap-label")
        .set("x", x + width / 2.0)
        .set("y", y + height / 2.0 - 5.0)
        .set("font-size", 16)
        .set("font-weight", "bold");
    document = document.add(label);

    // Band percentage
    let percentage_label = SvgText::new(format!("{:.0}%", node.percentage))
        .set("class", "integrated-treemap-percentage")
        .set("x", x + width / 2.0)
        .set("y", y + height / 2.0 + 15.0)
        .set("font-size", 12);
    document = document.add(percentage_label);

    Ok(document)
}

/// Extract name and percentage from formatted strings like "Vec<T> (100%)"
fn extract_name_and_percentage(formatted_name: &str) -> (&str, Option<&str>) {
    if let Some(open_paren) = formatted_name.find(" (") {
        if let Some(close_paren) = formatted_name.find(')') {
            let name = &formatted_name[..open_paren];
            let percentage = &formatted_name[open_paren + 2..close_paren];
            return (name, Some(percentage));
        }
    }
    (formatted_name, None)
}

/// Calculate appropriate font size based on available width
fn calculate_font_size(width: f64) -> i32 {
    if width > 200.0 {
        15
    } else if width > 100.0 {
        13
    } else if width > 50.0 {
        11
    } else {
        9
    }
}

/// Add categorized allocations visualization
pub fn add_categorized_allocations(
    mut document: Document,
    categories: &[AllocationCategory],
) -> TrackingResult<Document> {
    let chart_x = 50;
    let chart_y = 680; // Adjusted to provide better spacing (320 + 300 + 60 margin)
    let chart_width = 850;
    let chart_height = 280; // Slightly reduced height

    // Chart background with rounded corners
    let bg = Rectangle::new()
        .set("x", chart_x)
        .set("y", chart_y)
        .set("width", chart_width)
        .set("height", chart_height)
        .set("fill", "white")
        .set("stroke", "#bdc3c7")
        .set("stroke-width", 2)
        .set("rx", 20); // Even more rounded corners for natural look

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

        // Bar with rounded corners
        let bar = Rectangle::new()
            .set("x", chart_x + 150)
            .set("y", y)
            .set("width", bar_width)
            .set("height", bar_height - 5)
            .set("fill", category.color.as_str())
            .set("stroke", "#34495e")
            .set("stroke-width", 1)
            .set("rx", 12); // More rounded bar corners for natural look

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
                        var_name.to_string()
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
    let chart_y = 1380; // Moved up from 1780
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

    // Filter and sort tracked allocations by memory usage and lifecycle
    let mut tracked_allocs: Vec<_> = allocations
        .iter()
        .filter(|a| a.var_name.is_some())
        .collect();

    // Sort by memory size (descending) and then by lifecycle duration (descending)
    tracked_allocs.sort_by(|a, b| {
        let size_cmp = b.size.cmp(&a.size);
        if size_cmp == std::cmp::Ordering::Equal {
            // If sizes are equal, sort by lifecycle duration (timestamp difference as proxy)
            let a_duration =
                a.timestamp_dealloc.unwrap_or(a.timestamp_alloc + 1000) - a.timestamp_alloc;
            let b_duration =
                b.timestamp_dealloc.unwrap_or(b.timestamp_alloc + 1000) - b.timestamp_alloc;
            b_duration.cmp(&a_duration)
        } else {
            size_cmp
        }
    });

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
        .iter()
        .map(|a| a.timestamp_alloc)
        .min()
        .unwrap_or(0);
    let max_time = tracked_allocs
        .iter()
        .map(|a| a.timestamp_alloc)
        .max()
        .unwrap_or(min_time + 1);
    let _time_range = (max_time - min_time).max(1);

    // Optimized layout parameters to prevent overflow
    let label_width = 500; // More space for variable names
    let timeline_width = chart_width - label_width - 80; // Better margin
    let max_items = 10; // Show top 10 variables with highest memory usage and longest lifecycles

    // Calculate optimal row height based on available space
    let available_height = chart_height - 120; // Reserve space for title, axis, and note
    let row_height = (available_height / max_items as i32).min(22).max(18);

    // Draw timeline for tracked variables with optimized spacing
    for (i, allocation) in tracked_allocs.iter().take(max_items).enumerate() {
        // Use time-based positioning for better timeline representation
        let time_ratio = if max_time > min_time {
            let alloc_time = allocation.timestamp_alloc.max(min_time); // Prevent underflow
            (alloc_time - min_time) as f64 / (max_time - min_time) as f64
        } else {
            0.5 // Center if no time range
        };
        let x = chart_x + 30 + (time_ratio * timeline_width as f64) as i32;
        let y = chart_y + 40 + (i as i32 * row_height);

        // Ensure x position stays within timeline bounds
        let x = x.min(chart_x + timeline_width - 10).max(chart_x + 30);

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

        // Draw connecting line to label area with better positioning
        let label_start_x = chart_x + timeline_width + 40;
        let line = svg::node::element::Line::new()
            .set("x1", x + 5)
            .set("y1", y)
            .set("x2", label_start_x - 5)
            .set("y2", y)
            .set("stroke", "#bdc3c7")
            .set("stroke-width", 1)
            .set("stroke-dasharray", "2,3");

        document = document.add(line);

        // Optimized variable label with smart truncation to prevent overflow
        if let Some(var_name) = &allocation.var_name {
            let type_name = allocation.type_name.as_deref().unwrap_or("Unknown");
            let (simplified_type, _) = simplify_type_name(type_name);
            let size_info = format_bytes(allocation.size);

            // Smart truncation based on available space
            let max_var_name_length = 20;
            let max_type_length = 12;

            let truncated_var_name = if var_name.len() > max_var_name_length {
                format!("{}...", &var_name[..max_var_name_length - 3])
            } else {
                var_name.to_string()
            };

            let truncated_type = if simplified_type.len() > max_type_length {
                format!("{}...", &simplified_type[..max_type_length - 3])
            } else {
                simplified_type
            };

            let label_text = format!("{truncated_var_name} ({truncated_type}) {size_info}");

            let label = SvgText::new(label_text)
                .set("x", label_start_x)
                .set("y", y + 4)
                .set("font-size", 9) // Smaller font for better fit
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

    // Add note about data limitation
    let total_tracked = tracked_allocs.len();
    if total_tracked > max_items {
        let note_text = format!("Note: Showing top {max_items} variables with highest memory usage and longest lifecycles (out of {total_tracked} total tracked variables)");
        let note = SvgText::new(note_text)
            .set("x", chart_x + 20)
            .set("y", chart_y + chart_height - 10)
            .set("font-size", 11)
            .set("font-style", "italic")
            .set("fill", "#7f8c8d");
        document = document.add(note);
    }

    Ok(document)
}

/// Add fragmentation analysis chart
pub fn add_fragmentation_analysis(
    mut document: Document,
    allocations: &[AllocationInfo],
) -> TrackingResult<Document> {
    let chart_x = 950;
    let chart_y = 320; // Fix: adjust position to avoid overlap with header
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
    let chart_y = 680; // Fix: adjust position to avoid overlap
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
    let chart_y = 1040; // Moved up from 1430
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
    let legend_y = 1720; // Moved up from 2130
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
    let summary_y = 1720; // Moved up from 2130
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
/// Add enhanced memory allocation timeline with multiple visualization options
pub fn add_enhanced_timeline_dashboard(
    mut document: Document,
    stats: &MemoryStats,
    allocations: &[AllocationInfo],
) -> TrackingResult<Document> {
    // First, add the performance dashboard with circular progress indicators
    let dashboard_y = 120;
    let dashboard_height = 200;

    // Calculate performance metrics
    let memory_efficiency = if stats.peak_memory > 0 {
        (stats.active_memory as f64 / stats.peak_memory as f64) * 100.0
    } else {
        0.0
    };

    let allocation_efficiency = if stats.total_allocations > 0 {
        (stats.active_allocations as f64 / stats.total_allocations as f64) * 100.0
    } else {
        0.0
    };

    let fragmentation_ratio = if stats.total_allocated > 0 {
        (1.0 - (stats.active_memory as f64 / stats.total_allocated as f64)) * 100.0
    } else {
        0.0
    };

    // Performance Dashboard Section Background
    let dashboard_bg = Rectangle::new()
        .set("x", 50)
        .set("y", dashboard_y)
        .set("width", 1700)
        .set("height", dashboard_height)
        .set("fill", "rgba(255,255,255,0.1)")
        .set("stroke", "rgba(255,255,255,0.2)")
        .set("stroke-width", 1)
        .set("rx", 12);
    document = document.add(dashboard_bg);

    // Dashboard Title
    let dashboard_title = SvgText::new("Performance Dashboard")
        .set("x", 900)
        .set("y", dashboard_y + 25)
        .set("text-anchor", "middle")
        .set("font-size", 18)
        .set("font-weight", "bold")
        .set("fill", "#FFFFFF");
    document = document.add(dashboard_title);

    // Create circular progress indicators
    let metrics = [
        ("Active Memory", memory_efficiency, "#3498db"),
        ("Allocation Efficiency", allocation_efficiency, "#2ecc71"),
        (
            "Memory Fragmentation",
            100.0 - fragmentation_ratio,
            "#e74c3c",
        ),
        (
            "Peak Usage",
            if stats.peak_memory > 0 {
                (stats.active_memory as f64 / stats.peak_memory as f64) * 100.0
            } else {
                0.0
            },
            "#f39c12",
        ),
    ];

    for (i, (label, percentage, color)) in metrics.iter().enumerate() {
        let x = 200 + i * 350;
        let y = dashboard_y + 100;
        let radius = 40;

        // Background circle
        let bg_circle = Circle::new()
            .set("cx", x)
            .set("cy", y)
            .set("r", radius)
            .set("fill", "none")
            .set("stroke", "rgba(255,255,255,0.2)")
            .set("stroke-width", 8);
        document = document.add(bg_circle);

        // Progress circle
        let circumference = 2.0 * std::f64::consts::PI * radius as f64;
        let progress = circumference * (percentage / 100.0);
        let dash_array = format!("{} {}", progress, circumference - progress);

        let progress_circle = Circle::new()
            .set("cx", x)
            .set("cy", y)
            .set("r", radius)
            .set("fill", "none")
            .set("stroke", *color)
            .set("stroke-width", 8)
            .set("stroke-dasharray", dash_array)
            .set("stroke-dashoffset", 0)
            .set("transform", format!("rotate(-90 {x} {y})"))
            .set("stroke-linecap", "round");
        document = document.add(progress_circle);

        // Percentage text
        let percentage_text = SvgText::new(format!("{percentage:.0}%"))
            .set("x", x)
            .set("y", y + 5)
            .set("text-anchor", "middle")
            .set("font-size", 16)
            .set("font-weight", "bold")
            .set("fill", "#FFFFFF");
        document = document.add(percentage_text);

        // Label
        let label_text = SvgText::new(*label)
            .set("x", x)
            .set("y", y + 65)
            .set("text-anchor", "middle")
            .set("font-size", 12)
            .set("fill", "#FFFFFF");
        document = document.add(label_text);
    }

    // Now add the timeline chart below the dashboard
    let chart_x = 50;
    let chart_y = 350; // Move down to avoid overlapping with dashboard
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

    // Group tracked variables by scope for better visualization
    let tracked_vars: Vec<&AllocationInfo> = allocations
        .iter()
        .filter(|a| a.var_name.is_some())
        .collect();

    if tracked_vars.is_empty() {
        let no_data = SvgText::new("No tracked variables found for timeline visualization")
            .set("x", chart_x + chart_width / 2)
            .set("y", chart_y + chart_height / 2)
            .set("text-anchor", "middle")
            .set("font-size", 14)
            .set("fill", "#7f8c8d");
        document = document.add(no_data);
        return Ok(document);
    }

    // Sort tracked variables by memory size and lifecycle duration, then limit to top 10
    let mut sorted_tracked_vars: Vec<_> = tracked_vars.clone();
    sorted_tracked_vars.sort_by(|a, b| {
        let size_cmp = b.size.cmp(&a.size);
        if size_cmp == std::cmp::Ordering::Equal {
            // If sizes are equal, sort by lifecycle duration (timestamp difference as proxy)
            let a_duration =
                a.timestamp_dealloc.unwrap_or(a.timestamp_alloc + 1000) - a.timestamp_alloc;
            let b_duration =
                b.timestamp_dealloc.unwrap_or(b.timestamp_alloc + 1000) - b.timestamp_alloc;
            b_duration.cmp(&a_duration)
        } else {
            size_cmp
        }
    });

    // Limit to top 10 variables to prevent overflow
    let max_vars_to_show = 10;
    let limited_tracked_vars: Vec<_> = sorted_tracked_vars
        .into_iter()
        .take(max_vars_to_show)
        .collect();

    // Group limited variables by scope (extract scope from variable names)
    let mut scope_groups: std::collections::HashMap<String, Vec<&AllocationInfo>> =
        std::collections::HashMap::new();
    for var in &limited_tracked_vars {
        let scope_name = extract_scope_name(var.var_name.as_ref().unwrap());
        scope_groups.entry(scope_name).or_default().push(*var);
    }

    // Sort scopes for consistent display
    let mut sorted_scopes: Vec<_> = scope_groups.into_iter().collect();
    sorted_scopes.sort_by(|a, b| a.0.cmp(&b.0));

    // Calculate time ranges for limited tracked variables
    let max_time = limited_tracked_vars
        .iter()
        .map(|a| a.timestamp_alloc)
        .max()
        .unwrap_or(1000) as f64;
    let min_time = limited_tracked_vars
        .iter()
        .map(|a| a.timestamp_alloc)
        .min()
        .unwrap_or(0) as f64;
    let time_range = (max_time - min_time).max(1.0);

    // Plot area dimensions - Gantt chart layout
    let plot_x = chart_x + 200; // More space for variable names
    let plot_y = chart_y + 50;
    let plot_width = chart_width - 350; // Space for names and legend
    let plot_height = chart_height - 100;

    let row_height = (plot_height as f64 / sorted_scopes.len().max(1) as f64)
        .min(40.0)
        .max(25.0); // Ensure minimum spacing for scopes

    // Time axis (horizontal)
    let time_axis = svg::node::element::Line::new()
        .set("x1", plot_x)
        .set("y1", plot_y + plot_height)
        .set("x2", plot_x + plot_width)
        .set("y2", plot_y + plot_height)
        .set("stroke", "#34495e")
        .set("stroke-width", 2);
    document = document.add(time_axis);

    // Add time labels with better formatting - use relative time units
    let time_units = ["0ms", "0.25ms", "0.5ms", "0.75ms", "1ms"];
    for i in 0..=4 {
        let x = plot_x + (i * plot_width / 4);

        let tick = svg::node::element::Line::new()
            .set("x1", x)
            .set("y1", plot_y + plot_height)
            .set("x2", x)
            .set("y2", plot_y + plot_height + 5)
            .set("stroke", "#34495e")
            .set("stroke-width", 1);
        document = document.add(tick);

        // Better formatted time labels
        let time_label = time_units[i];
        let label = SvgText::new(time_label)
            .set("x", x)
            .set("y", plot_y + plot_height + 18)
            .set("text-anchor", "middle")
            .set("font-size", 10)
            .set("font-weight", "500")
            .set("fill", "#2c3e50");
        document = document.add(label);
    }

    // X-axis label - clearly indicate horizontal axis meaning
    let x_label = SvgText::new("Execution Time (milliseconds)")
        .set("x", plot_x + plot_width / 2)
        .set("y", plot_y + plot_height + 40)
        .set("text-anchor", "middle")
        .set("font-size", 12)
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");
    document = document.add(x_label);

    // Remove Y-axis label to save space and reduce clutter

    // Add Y-axis scale markers for better readability
    let scale_markers = [
        (16, "16B"),
        (256, "256B"),
        (1024, "1KB"),
        (4096, "4KB"),
        (16384, "16KB"),
    ];

    for (size, label) in scale_markers.iter() {
        // Skip size filtering for now - just show all scale markers
        {
            let log_size = (*size as f64).ln();
            let log_min = 1.0_f64.ln();
            let log_max = 16384.0_f64.ln();
            let log_range = log_max - log_min;

            if log_range > 0.0 {
                let y_pos = plot_y + plot_height
                    - ((log_size - log_min) / log_range * plot_height as f64) as i32;

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

    // Add X-axis time scale markers with better formatting
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

        // Format time more readably
        let formatted_time = if time_point < 1000.0 {
            format!("{time_point:.1}ms")
        } else if time_point < 60000.0 {
            format!("{:.2}s", time_point / 1000.0)
        } else {
            format!("{:.1}m", time_point / 60000.0)
        };

        let time_label = SvgText::new(formatted_time)
            .set("x", x_pos)
            .set("y", plot_y + plot_height + 20)
            .set("text-anchor", "middle")
            .set("font-size", 10)
            .set("font-weight", "500")
            .set("fill", "#2c3e50");
        document = document.add(time_label);
    }

    // Categorize allocations and get colors
    let categorized = categorize_allocations(allocations);
    let mut category_colors: HashMap<String, String> = HashMap::new();
    for category in &categorized {
        category_colors.insert(category.name.clone(), category.color.clone());
    }

    // Calculate P95 threshold for larger dots
    let mut sizes: Vec<usize> = allocations
        .iter()
        .map(|a| a.size)
        .filter(|&s| s > 0)
        .collect();
    sizes.sort_unstable();
    let _p95_threshold = if !sizes.is_empty() {
        let p95_index = (sizes.len() as f64 * 0.95) as usize;
        sizes[p95_index.min(sizes.len() - 1)]
    } else {
        0
    };

    // Draw scope-based Gantt chart - move scopes up slightly
    for (scope_index, (scope_name, scope_vars)) in sorted_scopes.iter().enumerate() {
        let scope_y = plot_y + 10 + (scope_index as f64 * row_height) as i32; // Moved up by 10px

        // Get scope color (different color for each scope)
        let scope_colors = [
            "#3498db", "#e74c3c", "#2ecc71", "#f39c12", "#9b59b6", "#1abc9c",
        ];
        let scope_color = scope_colors[scope_index % scope_colors.len()];

        // Draw scope background bar (full timeline)
        let scope_bg = Rectangle::new()
            .set("x", plot_x)
            .set("y", scope_y)
            .set("width", plot_width)
            .set("height", (row_height * 0.8) as i32)
            .set("fill", scope_color)
            .set("opacity", 0.2)
            .set("rx", 5);
        document = document.add(scope_bg);

        // Scope name label
        let scope_label = SvgText::new(format!("Scope: {scope_name}"))
            .set("x", plot_x - 15)
            .set("y", scope_y + (row_height * 0.4) as i32)
            .set("text-anchor", "end")
            .set("font-size", 11)
            .set("font-weight", "bold")
            .set("fill", scope_color);
        document = document.add(scope_label);

        // Draw variables within this scope
        for (var_index, alloc) in scope_vars.iter().enumerate() {
            let var_name = alloc.var_name.as_ref().unwrap();

            // Calculate variable bar position
            let start_x = plot_x as i32
                + ((alloc.timestamp_alloc as f64 - min_time) / time_range * plot_width as f64)
                    as i32;
            let var_y = scope_y + 5 + (var_index as f64 * 8.0) as i32; // Stack variables within scope

            // Get variable type color
            let (_, category) = if let Some(type_name) = &alloc.type_name {
                simplify_type_name(type_name)
            } else {
                (
                    "Unknown".to_string(),
                    "Runtime/System Allocation".to_string(),
                )
            };
            let var_color = get_category_color(&category);

            // Variable bar width based on memory size
            let max_size = scope_vars.iter().map(|a| a.size).max().unwrap_or(1024) as f64;
            let min_size = scope_vars
                .iter()
                .map(|a| a.size)
                .filter(|&s| s > 0)
                .min()
                .unwrap_or(1) as f64;
            let size_ratio = if max_size > min_size {
                ((alloc.size as f64).ln() - min_size.ln()) / (max_size.ln() - min_size.ln())
            } else {
                0.5
            };
            let bar_width = ((plot_width as f64 * 0.4 * size_ratio) + 30.0) as i32;

            // Draw variable bar
            let plot_x_i32 = plot_x as i32;
            let plot_width_i32 = plot_width as i32;
            let var_bar = Rectangle::new()
                .set("x", start_x.max(plot_x_i32))
                .set("y", var_y)
                .set(
                    "width",
                    bar_width.min(plot_x_i32 + plot_width_i32 - start_x.max(plot_x_i32)),
                )
                .set("height", 6)
                .set("fill", var_color)
                .set("stroke", "#ffffff")
                .set("stroke-width", 1)
                .set("rx", 2)
                .set("opacity", 0.9);
            document = document.add(var_bar);

            // Variable info label
            let var_info = format!("{}: {}", var_name, format_bytes(alloc.size));
            let var_label = SvgText::new(if var_info.len() > 25 {
                format!("{}...", &var_info[..22])
            } else {
                var_info
            })
            .set("x", start_x.max(plot_x_i32) + 5)
            .set("y", var_y + 4)
            .set("font-size", 7)
            .set("font-weight", "500")
            .set("fill", "#2c3e50");
            document = document.add(var_label);
        }
    }

    // Move legend to top-left corner with proportional scaling
    let legend_x = chart_x + 20;
    let legend_y = chart_y + 20;
    let legend_width = 200; // Scaled proportionally
    let legend_height = 120; // Scaled proportionally

    // Legend background box - completely transparent with no border
    let legend_bg = Rectangle::new()
        .set("x", legend_x - 10)
        .set("y", legend_y - 15)
        .set("width", legend_width)
        .set("height", legend_height)
        .set("fill", "rgba(255,255,255,0.0)") // Transparent background
        .set("stroke", "none") // No border
        .set("stroke-width", 0)
        .set("rx", 5);
    document = document.add(legend_bg);

    let legend_title = SvgText::new("Type Categories")
        .set("x", legend_x)
        .set("y", legend_y)
        .set("font-size", 10) // Smaller title
        .set("font-weight", "bold")
        .set("fill", "#2c3e50");
    document = document.add(legend_title);

    // Add compact system allocation info
    let unknown_count = allocations
        .iter()
        .filter(|a| {
            if let Some(type_name) = &a.type_name {
                let (_, category) = simplify_type_name(type_name);
                category == "Unknown"
            } else {
                true // No type_name at all
            }
        })
        .count();

    let unknown_legend_y = legend_y + 15;

    // System allocation color square - smaller
    let unknown_color_square = Rectangle::new()
        .set("x", legend_x)
        .set("y", unknown_legend_y - 6)
        .set("width", 8)
        .set("height", 8)
        .set("fill", "#95a5a6");
    document = document.add(unknown_color_square);

    // Compact system allocation label
    let unknown_label = if unknown_count > 0 {
        format!("System ({unknown_count} allocs)")
    } else {
        "No System Allocs".to_string()
    };

    let unknown_text = SvgText::new(unknown_label)
        .set("x", legend_x + 12)
        .set("y", unknown_legend_y - 1)
        .set("font-size", 8)
        .set("fill", "#2c3e50");
    document = document.add(unknown_text);

    for (i, category) in categorized.iter().take(4).enumerate() {
        // Reduce to 4 items for compact legend
        let legend_item_y = legend_y + 30 + (i as i32) * 15; // Reduce spacing

        // Color square - smaller size
        let color_square = Rectangle::new()
            .set("x", legend_x)
            .set("y", legend_item_y - 6)
            .set("width", 8)
            .set("height", 8)
            .set("fill", category.color.as_str());
        document = document.add(color_square);

        // Category name - more compact
        let category_name = if category.name.len() > 15 {
            format!("{}...", &category.name[..12])
        } else {
            category.name.clone()
        };

        let category_text = SvgText::new(category_name)
            .set("x", legend_x + 12)
            .set("y", legend_item_y - 1)
            .set("font-size", 8) // Smaller font
            .set("fill", "#2c3e50");
        document = document.add(category_text);
    }

    // Add note about data limitation for Memory Allocation Timeline
    let total_tracked = tracked_vars.len();
    if total_tracked > max_vars_to_show {
        let note_text = format!("Note: Showing top {max_vars_to_show} variables with highest memory usage and longest lifecycles (out of {total_tracked} total tracked variables)");
        let note = SvgText::new(note_text)
            .set("x", chart_x + 20)
            .set("y", chart_y + chart_height - 10)
            .set("font-size", 11)
            .set("font-style", "italic")
            .set("fill", "#7f8c8d");
        document = document.add(note);
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

/// Extract scope name from variable name - use actual program scopes
fn extract_scope_name(var_name: &str) -> String {
    // Extract meaningful scope information based on actual program structure
    if var_name.contains("global") {
        "Global Scope".to_string()
    } else if var_name.contains("static") {
        "Static Scope".to_string()
    } else if var_name.contains("boxed") {
        "Boxed Allocations".to_string()
    } else if var_name.contains("shared") || var_name.contains("arc") || var_name.contains("rc") {
        "Shared References".to_string()
    } else if var_name.contains("node") {
        "Graph Nodes".to_string()
    } else if var_name.contains("mutable") {
        "Mutable Data".to_string()
    } else if var_name.contains("_") {
        // Use prefix before first underscore as scope
        let prefix = var_name.split('_').next().unwrap_or("Unknown");
        format!("{} Scope", prefix.to_ascii_uppercase())
    } else {
        // Group by variable type/pattern
        if var_name.len() > 6 {
            format!("{} Scope", &var_name[..6].to_ascii_uppercase())
        } else {
            "Main Scope".to_string()
        }
    }
}
