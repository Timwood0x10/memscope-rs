//! Demo: Template-based dashboard rendering with Kinetic Engineering style
use memscope_rs::analysis::memory_passport_tracker::{
    MemoryPassportTracker, PassportTrackerConfig,
};
use memscope_rs::render_engine::{DashboardContext, DashboardRenderer};
use memscope_rs::tracker::Tracker;
use std::sync::Arc;

fn main() {
    println!("=== MemScope Unified Super Template Demo ===\n");

    let tracker = Tracker::new();
    let passport_tracker = Arc::new(MemoryPassportTracker::new(PassportTrackerConfig::default()));

    let output_dir = std::path::Path::new("demo_output");
    std::fs::create_dir_all(output_dir).ok();

    // 1. Unified (Kinetic Engineering style)
    println!("Rendering Unified Super Template...");
    let renderer = DashboardRenderer::new().unwrap();
    let ctx = renderer
        .build_context_from_tracker(&tracker, &passport_tracker)
        .unwrap();
    let html = renderer.render_unified_dashboard(&ctx).unwrap();
    let path = output_dir.join("unified_super_template.html");
    std::fs::write(&path, &html).unwrap();
    println!(
        "  ✅ {} ({:.1} KB)",
        path.display(),
        std::fs::metadata(&path).unwrap().len() as f64 / 1024.0
    );

    // 2. Final (Investigation Console - Indigo/Slate)
    println!("Rendering Investigation Console...");
    let html2 = renderer.render_final_dashboard(&ctx).unwrap();
    let path2 = output_dir.join("investigation_console.html");
    std::fs::write(&path2, &html2).unwrap();
    println!(
        "  ✅ {} ({:.1} KB)",
        path2.display(),
        std::fs::metadata(&path2).unwrap().len() as f64 / 1024.0
    );

    println!("\n✅ All dashboards exported to: {:?}", output_dir);
    println!("\nOpen unified_super_template.html in your browser to see the Kinetic Engineering mega-template!");
    println!("\nTemplate features:");
    println!("  • Hero KPI strip (Health Score, Allocations, Memory, Leaks)");
    println!(
        "  • Overview mode: diagnosis, type intelligence, heap lattice, flamegraph, alloc stream"
    );
    println!("  • Thread mode: relationship graph, heatmap, affinity grid, event log");
    println!("  • Task mode: waker efficiency heatgrid, poll latency, execution timeline");
    println!("  • Task graph mode: topology tree, streaming stats, trace log");
    println!("  • Variable mode: D3 force graph, neighbor density histogram, node detail panel");
    println!("  • Passport mode: clean/active/leaked/FFI summary cards");
    println!("  • FFI Bridge mode: call mapping topology, symbol table, thread timeline");
    println!("  • Unsafe mode: unsafe ops + FFI crossings side-by-side");
    println!("  • Time travel mode: timeline chart");
    println!("  • Footer status bar with clock");
}
