//! Memory tracking example application

use trace_tools::*;
use tracing::info;

mod export;
// mod procmacros;  // Commented out to fix compilation
mod tracker;
mod types;

/// Example function that allocates some memory
fn allocate_memory() -> Vec<u8> {
    let mut vec = Vec::with_capacity(1024);
    vec.extend(0..100);
    vec
}

fn main() {
    // Initialize the memory tracking system
    trace_tools::init();
    info!("Starting memory tracking example");

    // Allocate some memory
    let data = allocate_memory();
    track_var!(data);

    // Get memory statistics
    let stats = get_global_tracker().get_stats();
    info!(
        "Memory stats - Allocations: {}, Total bytes: {}",
        stats.total_allocations, stats.total_memory
    );

    // Export to JSON
    if let Err(e) = get_global_tracker().export_to_json("memory_snapshot.json") {
        eprintln!("Failed to export to JSON: {}", e);
    }

    // Export to SVG
    if let Err(e) = get_global_tracker().export_to_svg("memory_usage.svg") {
        eprintln!("Failed to export to SVG: {}", e);
    }

    info!("Example completed. Check memory_snapshot.json and memory_usage.svg for results.");

    // Print active allocations at the end
    let active = get_global_tracker().get_active_allocations();
    if !active.is_empty() {
        info!("--- Active Allocations ---");
        for alloc in active {
            info!(
                "Allocation: ptr=0x{:x}, size={}, var={:?}, type={:?}",
                alloc.ptr,
                alloc.size,
                alloc.var_name.unwrap_or_else(|| "<unknown>".to_string()),
                alloc.type_name.unwrap_or_else(|| "<unknown>".to_string())
            );
        }
    } else {
        info!("No active allocations at program end");
    }
}
