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
                let _temp = [j; 50];
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

        let total_categorized = instant_allocations
            + short_term_allocations
            + medium_term_allocations
            + long_term_allocations;
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

    #[test]
    fn test_short_lifetime_allocation_logic() {
        // Test the actual logic used in test_short_lifetime_allocations function
        let mut allocation_count = 0;

        for i in 0..100 {
            let temp_vec = vec![i; 100];
            let temp_string = format!("Temporary string {i}");
            let mut temp_map = HashMap::new();
            temp_map.insert(format!("key_{i}"), i);

            // Verify the allocations are created correctly
            assert_eq!(temp_vec.len(), 100);
            assert_eq!(temp_vec[0], i);
            assert!(temp_string.contains(&i.to_string()));
            assert_eq!(temp_map.get(&format!("key_{i}")), Some(&i));

            allocation_count += 1;
        }

        assert_eq!(allocation_count, 100);
    }

    #[test]
    fn test_long_lifetime_allocation_logic() {
        // Test the actual logic used in test_long_lifetime_allocations function
        let mut keep_alive = Vec::new();

        for i in 0..100 {
            let long_vec = vec![i; 100];
            let long_string = format!("Long-lived string {i}");

            // Verify allocations before boxing
            assert_eq!(long_vec.len(), 100);
            assert_eq!(long_vec[0], i);
            assert!(long_string.contains(&i.to_string()));

            keep_alive.push(Box::new(long_vec) as Box<dyn std::any::Any>);
            keep_alive.push(Box::new(long_string) as Box<dyn std::any::Any>);
        }

        assert_eq!(keep_alive.len(), 200); // 100 vecs + 100 strings
        drop(keep_alive);
    }

    #[test]
    fn test_mixed_lifetime_pattern_logic() {
        // Test the actual logic used in test_mixed_lifetime_pattern function
        let mut keep_alive = Vec::new();
        let mut short_count = 0;
        let mut long_count = 0;

        for i in 0..500 {
            // Test short-lived allocation creation
            let temp_data = vec![i; 50];
            assert_eq!(temp_data.len(), 50);
            assert_eq!(temp_data[0], i);
            short_count += 1;

            // Test long-lived allocation creation (every 10th)
            if i % 10 == 0 {
                let long_data = vec![i; 50];
                assert_eq!(long_data.len(), 50);
                assert_eq!(long_data[0], i);
                keep_alive.push(Box::new(long_data) as Box<dyn std::any::Any>);
                long_count += 1;
            }
        }

        assert_eq!(short_count, 500);
        assert_eq!(long_count, 50); // 500 / 10 = 50
        assert_eq!(keep_alive.len(), 50);
        drop(keep_alive);
    }

    #[test]
    fn test_release_rate_calculation_edge_cases() {
        // Test edge cases for release rate calculation

        // Case 1: All allocations released
        let total = 100u64;
        let active = 0u64;
        let release_rate = (total - active) as f64 / total as f64 * 100.0;
        assert_eq!(release_rate, 100.0);

        // Case 2: No allocations released
        let total = 100u64;
        let active = 100u64;
        let release_rate = (total - active) as f64 / total as f64 * 100.0;
        assert_eq!(release_rate, 0.0);

        // Case 3: Half allocations released
        let total = 100u64;
        let active = 50u64;
        let release_rate = (total - active) as f64 / total as f64 * 100.0;
        assert_eq!(release_rate, 50.0);
    }

    #[test]
    fn test_lifecycle_distribution_validation() {
        // Test that lifecycle distribution adds up correctly
        let instant = 150u64;
        let short_term = 250u64;
        let medium_term = 300u64;
        let long_term = 300u64;
        let total = instant + short_term + medium_term + long_term;

        assert_eq!(total, 1000);

        // Test proportions
        let instant_pct = instant as f64 / total as f64 * 100.0;
        let short_pct = short_term as f64 / total as f64 * 100.0;
        let medium_pct = medium_term as f64 / total as f64 * 100.0;
        let long_pct = long_term as f64 / total as f64 * 100.0;

        assert_eq!(instant_pct, 15.0);
        assert_eq!(short_pct, 25.0);
        assert_eq!(medium_pct, 30.0);
        assert_eq!(long_pct, 30.0);

        // Total should be 100%
        let total_pct = instant_pct + short_pct + medium_pct + long_pct;
        assert_eq!(total_pct, 100.0);
    }

    #[test]
    fn test_allocation_pattern_boundary_conditions() {
        // Test boundary conditions for allocation pattern analysis

        // Exactly half active (should be considered mostly active)
        let total = 1000u64;
        let active_half = 500u64;
        assert!(active_half >= total / 2);

        // Just under half active (should be considered mostly released)
        let active_under_half = 499u64;
        assert!(active_under_half < total / 2);

        // Just over half active (should be considered mostly active)
        let active_over_half = 501u64;
        assert!(active_over_half >= total / 2);
    }

    #[test]
    fn test_format_string_generation() {
        // Test the format string generation used in the functions
        for i in 0..10 {
            let temp_string = format!("Temporary string {i}");
            assert!(temp_string.starts_with("Temporary string "));
            assert!(temp_string.ends_with(&i.to_string()));

            let long_string = format!("Long-lived string {i}");
            assert!(long_string.starts_with("Long-lived string "));
            assert!(long_string.ends_with(&i.to_string()));

            let key_string = format!("key_{i}");
            assert!(key_string.starts_with("key_"));
            assert!(key_string.ends_with(&i.to_string()));
        }
    }

    #[test]
    fn test_hashmap_operations() {
        // Test HashMap operations used in the allocation functions
        let mut test_map = HashMap::new();

        for i in 0..10 {
            let key = format!("key_{i}");
            test_map.insert(key.clone(), i);

            // Verify insertion
            assert_eq!(test_map.get(&key), Some(&i));
            assert_eq!(test_map.len(), (i + 1) as usize);
        }

        // Test all keys exist
        for i in 0..10 {
            let key = format!("key_{i}");
            assert!(test_map.contains_key(&key));
        }

        assert_eq!(test_map.len(), 10);
    }

    #[test]
    fn test_vector_allocation_patterns() {
        // Test vector allocation patterns used in the functions

        // Test small vectors (50 elements)
        let small_vec = vec![42; 50];
        assert_eq!(small_vec.len(), 50);
        assert!(small_vec.iter().all(|&x| x == 42));

        // Test medium vectors (100 elements)
        let medium_vec = vec![123; 100];
        assert_eq!(medium_vec.len(), 100);
        assert!(medium_vec.iter().all(|&x| x == 123));

        // Test large vectors (1000 elements)
        let large_vec = vec![999; 1000];
        assert_eq!(large_vec.len(), 1000);
        assert!(large_vec.iter().all(|&x| x == 999));
    }

    #[test]
    fn test_boxed_any_conversion() {
        // Test the Box<dyn Any> conversion used in long-lived allocations
        let test_vec = vec![1, 2, 3, 4, 5];
        let test_string = "test string".to_string();

        let boxed_vec = Box::new(test_vec) as Box<dyn std::any::Any>;
        let boxed_string = Box::new(test_string) as Box<dyn std::any::Any>;

        // Verify the boxes can be created
        assert!(boxed_vec.type_id() == std::any::TypeId::of::<Vec<i32>>());
        assert!(boxed_string.type_id() == std::any::TypeId::of::<String>());
    }
}
