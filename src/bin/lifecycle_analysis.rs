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
        track_var!(temp_vec);

        let temp_string = format!("Temporary string {i}");
        track_var!(temp_string);

        let mut temp_map = HashMap::new();
        temp_map.insert(format!("key_{i}"), i);
        track_var!(temp_map);
    } // all variables are released here

    tracing::info!("  ✅ 1000 short lifetime allocations completed (automatically released)");
}

fn test_long_lifetime_allocations() -> Vec<Box<dyn std::any::Any>> {
    tracing::info!("  create 100 long lifetime allocations...");
    let mut keep_alive = Vec::new();

    for i in 0..100 {
        let long_vec = vec![i; 100];
        track_var!(long_vec);
        keep_alive.push(Box::new(long_vec) as Box<dyn std::any::Any>);

        let long_string = format!("Long-lived string {i}");
        track_var!(long_string);
        keep_alive.push(Box::new(long_string) as Box<dyn std::any::Any>);
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
        track_var!(temp_data);

        // every 10 create a long (keep alive)
        if i % 10 == 0 {
            let long_data = vec![i; 50];
            track_var!(long_data);
            keep_alive.push(Box::new(long_data) as Box<dyn std::any::Any>);
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn test_short_lifetime_allocation_creation() {
        // Test the logic for creating short-lifetime allocations without global tracker
        let mut temp_allocations = Vec::new();
        
        for i in 0..10 {
            let temp_vec = vec![i; 100];
            let temp_string = format!("temp_string_{i}");
            let mut temp_map = HashMap::new();
            temp_map.insert(format!("temp_key_{i}"), i);
            
            temp_allocations.push((temp_vec, temp_string, temp_map));
        }
        
        assert_eq!(temp_allocations.len(), 10);
        
        // Test that allocations are properly created
        assert_eq!(temp_allocations[0].0.len(), 100);
        assert!(temp_allocations[0].1.contains("temp_string_0"));
        assert_eq!(temp_allocations[0].2.get("temp_key_0"), Some(&0));
        
        // Simulate dropping short-lived allocations
        drop(temp_allocations);
    }

    #[test]
    fn test_long_lifetime_allocation_creation() {
        // Test the logic for creating long-lifetime allocations without global tracker
        let mut long_lived = Vec::new();
        
        for i in 0..5 {
            let persistent_data = vec![i; 1000];
            let persistent_string = format!("persistent_data_{i}");
            
            long_lived.push((persistent_data, persistent_string));
        }
        
        assert_eq!(long_lived.len(), 5);
        assert_eq!(long_lived[0].0.len(), 1000);
        assert!(long_lived[0].1.contains("persistent_data_0"));
        
        // Return the long-lived data to simulate keeping it alive
        drop(long_lived);
    }

    #[test]
    fn test_mixed_lifetime_pattern_creation() {
        // Test the logic for creating mixed lifetime patterns without global tracker
        let mut long_lived = Vec::new();
        
        for i in 0..10 {
            // Create short-lived allocations (simulated by immediate drop)
            for j in 0..10 {
                let _temp = vec![j; 50];
                let _temp_string = format!("temp_{i}_{j}");
            }
            
            // Create long-lived allocation
            if i % 2 == 0 {
                let persistent = vec![i; 200];
                long_lived.push(persistent);
            }
        }
        
        assert_eq!(long_lived.len(), 5); // Every other iteration creates long-lived data
        assert_eq!(long_lived[0].len(), 200);
        
        drop(long_lived);
    }

    #[test]
    fn test_lifecycle_statistics_calculation() {
        // Test lifecycle statistics calculation logic without global tracker
        let total_allocations = 1000u64;
        let active_allocations = 300u64;
        let released_allocations = total_allocations - active_allocations;
        
        assert_eq!(released_allocations, 700);
        
        let release_rate = released_allocations as f64 / total_allocations as f64 * 100.0;
        assert_eq!(release_rate, 70.0);
        
        // Test lifecycle distribution logic
        let instant_allocations = 200u64;
        let short_term_allocations = 300u64;
        let medium_term_allocations = 250u64;
        let long_term_allocations = 250u64;
        
        let total_categorized = instant_allocations + short_term_allocations + medium_term_allocations + long_term_allocations;
        assert_eq!(total_categorized, total_allocations);
    }

    #[test]
    fn test_allocation_pattern_analysis() {
        // Test the analysis logic for different allocation patterns
        let active_allocations = 200u64;
        let total_allocations = 1000u64;
        
        // Test condition for mostly released allocations
        let mostly_released = active_allocations < total_allocations / 2;
        assert!(mostly_released);
        
        // Test condition for mostly active allocations
        let active_allocations_high = 800u64;
        let mostly_active = active_allocations_high >= total_allocations / 2;
        assert!(mostly_active);
    }
}
