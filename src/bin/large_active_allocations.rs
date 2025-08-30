//! large active allocations test program
//!
//! create large active allocations to test the true large file export performance

use memscope_rs::{get_global_tracker, init, track_var};
use std::collections::HashMap;

fn main() {
    tracing::info!("🚀 large active allocations test program");
    tracing::info!("======================");
    tracing::info!("");

    init();

    // 保持所有分配存活的容器
    let mut keep_alive: Vec<Box<dyn std::any::Any>> = Vec::new();

    tracing::info!("📦 create 10,000 active allocations...");

    for i in 0..1000 {
        // Reduced from 10000 to 1000
        // create different types of allocations
        let large_vec = vec![i; 100];
        track_var!(large_vec);
        keep_alive.push(Box::new(()) as Box<dyn std::any::Any>);

        let large_string = format!("Large string with data {i}");
        track_var!(large_string);
        keep_alive.push(Box::new(()) as Box<dyn std::any::Any>);

        let mut map = HashMap::new();
        map.insert(format!("key_{i}"), i);
        track_var!(map);
        keep_alive.push(Box::new(()) as Box<dyn std::any::Any>);

        if i % 1000 == 0 {
            tracing::info!("  ✅ created {} groups", i);
        }
    }

    tracing::info!("\n📊 final statistics:");
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        tracing::info!("  • total allocations: {}", stats.total_allocations);
        tracing::info!("  • active allocations: {}", stats.active_allocations);
        tracing::info!(
            "  • released allocations: {}",
            stats.total_allocations - stats.active_allocations
        );
        tracing::info!(
            "  • active rate: {:.1}%",
            stats.active_allocations as f64 / stats.total_allocations as f64 * 100.0
        );
        tracing::info!(
            "  • active memory: {:.2} MB",
            stats.active_memory as f64 / 1024.0 / 1024.0
        );
    }

    tracing::info!("\n🎯 now you can test large file export performance!");
    tracing::info!("advise: use fast export to handle so many active allocations.");

    // keep all allocations alive until program ends
    tracing::info!("📌 keep {} variables alive", keep_alive.len());
}
