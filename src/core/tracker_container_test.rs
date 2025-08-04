//! Tests for container analysis functionality

#[cfg(test)]
mod tests {
    use crate::core::types::*;
    use crate::MemoryTracker;

    #[test]
    fn test_vec_container_analysis() {
        let tracker = MemoryTracker::new();

        // Test Vec<i32> analysis
        let layout = tracker.analyze_memory_layout("Vec<i32>", 64).unwrap();

        assert_eq!(layout.total_size, 64);
        assert!(layout.container_analysis.is_some());

        let container_analysis = layout.container_analysis.unwrap();
        match container_analysis.container_type {
            ContainerType::Vec {
                element_type,
                element_size,
            } => {
                assert_eq!(element_type, "i32");
                assert_eq!(element_size, 4);
            }
            _ => panic!("Expected Vec container type"),
        }

        // Check capacity utilization
        assert!(container_analysis.capacity_utilization.current_capacity > 0);
        assert!(container_analysis.capacity_utilization.utilization_ratio >= 0.0);
        assert!(container_analysis.capacity_utilization.utilization_ratio <= 1.0);

        // Check reallocation patterns
        assert_eq!(
            container_analysis.reallocation_patterns.growth_pattern,
            GrowthPattern::Exponential
        );

        // Check efficiency metrics
        assert!(container_analysis.efficiency_metrics.health_score >= 0.0);
        assert!(container_analysis.efficiency_metrics.health_score <= 100.0);
    }

    #[test]
    fn test_hashmap_container_analysis() {
        let tracker = MemoryTracker::new();

        // Test HashMap<String, i32> analysis
        let layout = tracker
            .analyze_memory_layout("HashMap<String, i32>", 128)
            .unwrap();

        assert!(layout.container_analysis.is_some());

        let container_analysis = layout.container_analysis.unwrap();
        match container_analysis.container_type {
            ContainerType::HashMap {
                key_type,
                value_type,
                key_size,
                value_size,
            } => {
                assert_eq!(key_type, "String");
                assert_eq!(value_type, "i32");
                assert!(key_size > 0);
                assert_eq!(value_size, 4);
            }
            _ => panic!("Expected HashMap container type"),
        }

        // HashMap should have random access efficiency
        assert_eq!(
            container_analysis.efficiency_metrics.access_efficiency,
            AccessEfficiency::Random
        );
    }

    #[test]
    fn test_box_container_analysis() {
        let tracker = MemoryTracker::new();

        // Test Box<String> analysis
        let layout = tracker.analyze_memory_layout("Box<String>", 8).unwrap();

        assert!(layout.container_analysis.is_some());

        let container_analysis = layout.container_analysis.unwrap();
        match container_analysis.container_type {
            ContainerType::Box {
                boxed_type,
                boxed_size,
            } => {
                assert_eq!(boxed_type, "String");
                assert!(boxed_size > 0);
            }
            _ => panic!("Expected Box container type"),
        }

        // Box should have single allocation pattern
        assert_eq!(
            container_analysis.reallocation_patterns.growth_pattern,
            GrowthPattern::SingleAllocation
        );
        assert_eq!(
            container_analysis
                .reallocation_patterns
                .frequency_assessment,
            ReallocationFrequency::None
        );
    }

    #[test]
    fn test_string_container_analysis() {
        let tracker = MemoryTracker::new();

        // Test String analysis
        let layout = tracker.analyze_memory_layout("String", 48).unwrap();

        assert!(layout.container_analysis.is_some());

        let container_analysis = layout.container_analysis.unwrap();
        assert_eq!(container_analysis.container_type, ContainerType::String);

        // String should have sequential access efficiency
        assert_eq!(
            container_analysis.efficiency_metrics.access_efficiency,
            AccessEfficiency::Sequential
        );

        // String should have exponential growth pattern
        assert_eq!(
            container_analysis.reallocation_patterns.growth_pattern,
            GrowthPattern::Exponential
        );
    }

    #[test]
    fn test_capacity_utilization_efficiency() {
        let tracker = MemoryTracker::new();

        // Test different utilization scenarios
        let layout_small = tracker.analyze_memory_layout("Vec<u8>", 32).unwrap();
        let layout_large = tracker.analyze_memory_layout("Vec<u8>", 1024).unwrap();

        let small_analysis = layout_small.container_analysis.unwrap();
        let large_analysis = layout_large.container_analysis.unwrap();

        // Both should have valid utilization ratios
        assert!(small_analysis.capacity_utilization.utilization_ratio >= 0.0);
        assert!(small_analysis.capacity_utilization.utilization_ratio <= 1.0);
        assert!(large_analysis.capacity_utilization.utilization_ratio >= 0.0);
        assert!(large_analysis.capacity_utilization.utilization_ratio <= 1.0);

        // Larger containers might have different efficiency assessments
        match small_analysis.capacity_utilization.efficiency_assessment {
            UtilizationEfficiency::Excellent
            | UtilizationEfficiency::Good
            | UtilizationEfficiency::Fair
            | UtilizationEfficiency::Poor { .. } => {
                // All variants are valid
            }
        }
    }

    #[test]
    fn test_reallocation_pattern_detection() {
        let tracker = MemoryTracker::new();

        // Test Vec reallocation estimation
        let layout = tracker.analyze_memory_layout("Vec<i64>", 256).unwrap();
        let container_analysis = layout.container_analysis.unwrap();

        // Should detect some reallocations for a reasonably sized Vec
        assert!(
            container_analysis
                .reallocation_patterns
                .estimated_reallocations
                > 0
        );

        // Should have optimization suggestions
        assert!(!container_analysis
            .reallocation_patterns
            .optimization_suggestions
            .is_empty());

        // Should classify frequency appropriately
        match container_analysis
            .reallocation_patterns
            .frequency_assessment
        {
            ReallocationFrequency::None
            | ReallocationFrequency::Low
            | ReallocationFrequency::Moderate
            | ReallocationFrequency::High { .. } => {
                // All variants are valid
            }
        }
    }

    #[test]
    fn test_non_container_type() {
        let tracker = MemoryTracker::new();

        // Test non-container type
        let layout = tracker.analyze_memory_layout("i32", 4).unwrap();

        // Should not have container analysis for primitive types
        assert!(layout.container_analysis.is_none());
    }

    #[test]
    fn test_container_efficiency_metrics() {
        let tracker = MemoryTracker::new();

        let layout = tracker.analyze_memory_layout("Vec<f64>", 128).unwrap();
        let container_analysis = layout.container_analysis.unwrap();

        let metrics = &container_analysis.efficiency_metrics;

        // All metrics should be within valid ranges
        assert!(metrics.memory_overhead >= 0.0);
        assert!(metrics.cache_efficiency >= 0.0);
        assert!(metrics.cache_efficiency <= 100.0);
        assert!(metrics.health_score >= 0.0);
        assert!(metrics.health_score <= 100.0);

        // Vec should have sequential access efficiency
        assert_eq!(metrics.access_efficiency, AccessEfficiency::Sequential);
    }
}
