//! lifecycle analysis program
//!
//! analyze why most allocations in complex_lifecycle_showcase are released

use memscope_rs::{get_global_tracker, init, track_var};
use std::collections::HashMap;

fn main() {
    tracing::info!("🔍 lifecycle analysis program");
    tracing::info!("==================");
    tracing::info!("");

    init();

    tracing::info!("📊 initial state:");
    print_stats("after initialization");

    tracing::info!("\n🧪 test 1: short lifetime allocations");
    test_short_lifetime_allocations();
    print_stats("after short lifetime test");

    tracing::info!("\n🧪 test 2: long lifetime allocations");
    let _long_lived = test_long_lifetime_allocations();
    print_stats("after long lifetime test");

    tracing::info!("\n🧪 test 3: mixed lifetime pattern");
    let _mixed_lived = test_mixed_lifetime_pattern();
    print_stats("after mixed lifetime test");

    tracing::info!("\n📊 final analysis:");
    analyze_lifecycle_patterns();
}

fn test_short_lifetime_allocations() {
    tracing::info!("  create 1000 short lifetime allocations...");

    for i in 0..100 {
        // Reduced from 1000 to 100
        let temp_vec = vec![i; 100];
        let _tracked = track_var!(temp_vec);

        let temp_string = format!("Temporary string {i}");
        let _tracked_string = track_var!(temp_string);

        let mut temp_map = HashMap::new();
        temp_map.insert(format!("key_{i}"), i);
        let _tracked_map = track_var!(temp_map);
    } // all variables are released here

    tracing::info!("  ✅ 1000 short lifetime allocations completed (automatically released)");
}

fn test_long_lifetime_allocations() -> Vec<Box<dyn std::any::Any>> {
    tracing::info!("  create 100 long lifetime allocations...");
    let mut keep_alive = Vec::new();

    for i in 0..100 {
        let long_vec = vec![i; 100];
        let tracked_vec = track_var!(long_vec);
        keep_alive.push(Box::new(tracked_vec) as Box<dyn std::any::Any>);

        let long_string = format!("Long-lived string {i}");
        let tracked_string = track_var!(long_string);
        keep_alive.push(Box::new(tracked_string) as Box<dyn std::any::Any>);
    }

    tracing::info!("  ✅ 100 long lifetime allocations completed (keep alive)");
    keep_alive
}

fn test_mixed_lifetime_pattern() -> Vec<Box<dyn std::any::Any>> {
    tracing::info!("  create mixed lifetime pattern...");
    let mut keep_alive = Vec::new();

    // create 500 short + 50 long
    for i in 0..500 {
        // short (will be released)
        let temp_data = vec![i; 50];
        let _tracked_temp = track_var!(temp_data);

        // every 10 create a long (keep alive)
        if i % 10 == 0 {
            let long_data = vec![i; 50];
            let tracked_long = track_var!(long_data);
            keep_alive.push(Box::new(tracked_long) as Box<dyn std::any::Any>);
        }
    }

    tracing::info!("  ✅ mixed pattern: 500 short + 50 long");
    keep_alive
}

fn print_stats(phase: &str) {
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        tracing::info!(
            "  📊 {}: total allocations={}, active allocations={}, released={}",
            phase,
            stats.total_allocations,
            stats.active_allocations,
            stats.total_allocations - stats.active_allocations
        );
    }
}

fn analyze_lifecycle_patterns() {
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        tracing::info!("🔍 lifecycle pattern analysis:");
        tracing::info!("  • total allocations: {}", stats.total_allocations);
        tracing::info!("  • active allocations: {}", stats.active_allocations);
        tracing::info!(
            "  • released allocations: {}",
            stats.total_allocations - stats.active_allocations
        );
        tracing::info!(
            "  • release rate: {:.1}%",
            (stats.total_allocations - stats.active_allocations) as f64
                / stats.total_allocations as f64
                * 100.0
        );

        let lifecycle = &stats.lifecycle_stats;
        tracing::info!("\n📈 lifecycle distribution:");
        tracing::info!(
            "  • instant allocations (< 1ms): {}",
            lifecycle.instant_allocations
        );
        tracing::info!(
            "  • short term allocations (1-100ms): {}",
            lifecycle.short_term_allocations
        );
        tracing::info!(
            "  • medium term allocations (100ms-1s): {}",
            lifecycle.medium_term_allocations
        );
        tracing::info!(
            "  • long term allocations (> 1s): {}",
            lifecycle.long_term_allocations
        );

        tracing::info!("\n💡 conclusion:");
        if stats.active_allocations < stats.total_allocations / 2 {
            tracing::info!("  ⚠️  most allocations have been released, explaining why the export file size doesn't grow with total allocations");
            tracing::info!("  📝 the export system only exports active allocations, released allocations won't appear in the file");
        } else {
            tracing::info!("  ✅ most allocations are still active, file size should be proportional to the number of allocations");
        }
    }
}
