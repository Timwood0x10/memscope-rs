//! Demo: Render the unified MemScope dashboard (merged multi-mode template).
//!
//! The merged template embeds all eight dashboard modes in a single HTML
//! file: Overview, Threads, Async Tasks, Task Graph, Variables, Passports,
//! FFI Bridge, and Unsafe/Time Travel. A left side-bar switches between
//! modes via CSS `display` toggling — no extra HTTP requests or templates.
use memscope_rs::analysis::memory_passport_tracker::{
    MemoryPassportTracker, PassportTrackerConfig,
};
use memscope_rs::render_engine::DashboardRenderer;
use memscope_rs::tracker::Tracker;
use std::sync::Arc;

fn main() {
    println!("=== MemScope Unified Dashboard Demo ===\n");

    let tracker = Tracker::new();
    let passport_tracker = Arc::new(MemoryPassportTracker::new(PassportTrackerConfig::default()));

    let output_dir = std::path::Path::new("demo_output");
    std::fs::create_dir_all(output_dir).ok();

    let renderer = DashboardRenderer::new().unwrap();
    let ctx = renderer
        .build_context_from_tracker(&tracker, &passport_tracker)
        .unwrap();

    // Single merged template — render_unified_dashboard and render_final_dashboard
    // both delegate to the same dashboard_unified template now.
    println!("Rendering unified dashboard...");
    let html = renderer.render_unified_dashboard(&ctx).unwrap();
    let path = output_dir.join("unified_dashboard.html");
    std::fs::write(&path, &html).unwrap();
    println!(
        "  ✅ {} ({:.1} KB)",
        path.display(),
        std::fs::metadata(&path).unwrap().len() as f64 / 1024.0
    );

    println!("\n✅ Dashboard exported to: {:?}", output_dir);
    println!("\nOpen unified_dashboard.html in your browser to see the merged template!");
    println!("\nMerged template modes (left side-bar switches between them):");
    println!("  • Overview     — 4 KPI cards, type intelligence, heap lattice, flamegraph, allocations stream");
    println!("  • Threads      — thread affinity map, hardware core allocation, event log, thread details");
    println!("  • Async Tasks  — task counters, waker efficiency heatgrid, poll latency, execution timeline");
    println!("  • Task Graph   — task topology tree, streaming stats, trace log window");
    println!(
        "  • Variables    — variable dependency graph, node detail, neighbor density histogram"
    );
    println!("  • Passports    — clean/active/leaked/FFI summary cards, passport cards grid");
    println!(
        "  • FFI Bridge   — call mapping topology, stack integrity, symbol table, thread timeline"
    );
    println!("  • Unsafe/Time  — unsafe ops + FFI crossings, ownership graph, time travel chart");
    println!("\nAll dashboard data fields are bound via Handlebars to DashboardContext,");
    println!("so live tracker data renders correctly in every mode.");
}
