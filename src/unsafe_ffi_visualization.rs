//! Specialized SVG visualizations for unsafe Rust and FFI memory analysis

use crate::unsafe_ffi_tracker::{
    AllocationSource, BoundaryEventType, EnhancedAllocationInfo, 
    SafetyViolation, UnsafeFFITracker
};
use crate::types::{TrackingError, TrackingResult};
use crate::utils::format_bytes;
use std::path::Path;
use std::fs::File;
use svg::node::element::{
    Circle, Line, Path as SvgPath, Rectangle, Text as SvgText, 
    Definitions, LinearGradient, Stop, Marker, Polygon
};
use svg::Document;

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
                .set("fill", "#e74c3c")
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
    let unsafe_count = allocations.iter()
        .filter(|a| matches!(a.source, AllocationSource::UnsafeRust { .. }))
        .count();
    let ffi_count = allocations.iter()
        .filter(|a| matches!(a.source, AllocationSource::FfiC { .. }))
        .count();
    let cross_boundary_events: usize = allocations.iter()
        .map(|a| a.cross_boundary_events.len())
        .sum();
    let total_unsafe_memory: usize = allocations.iter()
        .filter(|a| !matches!(a.source, AllocationSource::RustSafe))
        .map(|a| a.base.size)
        .sum();

    // Metrics cards
    let metrics = vec![
        ("Unsafe Allocations", unsafe_count.to_string(), "#e74c3c"),
        ("FFI Allocations", ffi_count.to_string(), "#3498db"),
        ("Boundary Crossings", cross_boundary_events.to_string(), "#f39c12"),
        ("Safety Violations", violations.len().to_string(), "#e67e22"),
        ("Unsafe Memory", format_bytes(total_unsafe_memory), "#9b59b6"),
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
        let value_text = SvgText::new(value.clone())
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
    let sources = vec![
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
            let count_text = SvgText::new(count.to_string())
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
    let bg_color = if violations.is_empty() { "#27ae60" } else { "#e74c3c" };
    let bg = Rectangle::new()
        .set("x", start_x)
        .set("y", start_y)
        .set("width", width)
        .set("height", height)
        .set("fill", format!("{}20", bg_color))
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

        let safe_desc = SvgText::new("All unsafe operations and FFI calls appear to be memory-safe")
            .set("x", start_x + width / 2)
            .set("y", start_y + 180)
            .set("text-anchor", "middle")
            .set("font-size", 12)
            .set("fill", "#2ecc71");
        document = document.add(safe_desc);
    } else {
        // Violations detected
        let violation_text = SvgText::new(format!("{} Safety Violations Detected", violations.len()))
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

            let violation_item = SvgText::new(format!("• {}", description))
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

        let count_text = SvgText::new(rust_to_ffi.to_string())
            .set("x", start_x + 300)
            .set("y", start_y + 75)
            .set("text-anchor", "middle")
            .set("font-size", 12)
            .set("font-weight", "bold")
            .set("fill", "#e74c3c");
        document = document.add(count_text);
    }

    if ffi_to_rust > 0 {
        let count_text = SvgText::new(ffi_to_rust.to_string())
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
    let unsafe_allocations: Vec<_> = allocations.iter()
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