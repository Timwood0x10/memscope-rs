//! Unified visualization module for memscope-rs
//! Provides memory analysis and lifecycle timeline SVG exports

use crate::tracker::MemoryTracker;
use crate::types::{AllocationInfo, MemoryStats, TrackingError, TrackingResult};
use std::fs::File;
use std::path::Path;
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
    let memory_by_type = crate::export_enhanced::enhance_type_information(&memory_by_type_data, allocations);
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
    _allocations: &[AllocationInfo],
    _stats: &MemoryStats,
) -> TrackingResult<Document> {
    let width = 1600;
    let height = 1200;

    let document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", width)
        .set("height", height)
        .set("style", "background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); font-family: 'Inter', 'Segoe UI', sans-serif;");

    // Placeholder for lifecycle timeline
    Ok(document)
}