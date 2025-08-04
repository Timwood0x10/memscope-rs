//! Integration test for container analysis functionality

use memscope_rs::{
    core::tracker::MemoryTracker,
    export::binary::{BinaryExportConfig, BinaryExportConfigBuilder},
    core::types::*,
};

#[test]
fn test_container_analysis_integration() {
    let tracker = MemoryTracker::new();
    
    // Test that container analysis is properly integrated
    let layout = tracker.analyze_memory_layout("Vec<i32>", 128).unwrap();
    
    // Verify basic layout info
    assert_eq!(layout.total_size, 128);
    assert!(layout.alignment > 0);
    
    // Verify container analysis is present
    assert!(layout.container_analysis.is_some());
    
    let container_analysis = layout.container_analysis.unwrap();
    
    // Verify container type classification
    match container_analysis.container_type {
        ContainerType::Vec { element_type, element_size } => {
            assert_eq!(element_type, "i32");
            assert_eq!(element_size, 4);
        }
        _ => panic!("Expected Vec container type"),
    }
    
    // Verify capacity utilization analysis
    let utilization = &container_analysis.capacity_utilization;
    assert!(utilization.current_capacity > 0);
    assert!(utilization.utilization_ratio >= 0.0);
    assert!(utilization.utilization_ratio <= 1.0);
    
    // Verify reallocation pattern detection
    let patterns = &container_analysis.reallocation_patterns;
    assert_eq!(patterns.growth_pattern, GrowthPattern::Exponential);
    assert!(patterns.estimated_reallocations > 0);
    
    // Verify efficiency metrics
    let metrics = &container_analysis.efficiency_metrics;
    assert!(metrics.health_score >= 0.0);
    assert!(metrics.health_score <= 100.0);
    assert_eq!(metrics.access_efficiency, AccessEfficiency::Sequential);
}

#[test]
fn test_binary_export_config_container_analysis() {
    // Test that BinaryExportConfig properly controls container analysis
    let config_with_container_analysis = BinaryExportConfigBuilder::new()
        .container_analysis(true)
        .build();
    
    assert!(config_with_container_analysis.container_analysis);
    
    let config_without_container_analysis = BinaryExportConfigBuilder::new()
        .container_analysis(false)
        .build();
    
    assert!(!config_without_container_analysis.container_analysis);
    
    // Test preset configurations
    let performance_config = BinaryExportConfig::performance_first();
    assert!(performance_config.container_analysis); // Should be enabled for high value
    
    let debug_config = BinaryExportConfig::debug_comprehensive();
    assert!(debug_config.container_analysis); // Should be enabled for comprehensive analysis
    
    let minimal_config = BinaryExportConfig::minimal();
    assert!(!minimal_config.container_analysis); // Should be disabled for minimal overhead
}

#[test]
fn test_container_analysis_different_types() {
    let tracker = MemoryTracker::new();
    
    // Test different container types
    let test_cases = vec![
        ("Vec<u8>", 64, ContainerType::Vec { element_type: "u8".to_string(), element_size: 1 }),
        ("HashMap<i32, String>", 256, ContainerType::HashMap { 
            key_type: "i32".to_string(), 
            value_type: "String".to_string(), 
            key_size: 4, 
            value_size: 24 
        }),
        ("Box<f64>", 8, ContainerType::Box { 
            boxed_type: "f64".to_string(), 
            boxed_size: 8 
        }),
        ("String", 48, ContainerType::String),
    ];
    
    for (type_name, size, expected_container_type) in test_cases {
        let layout = tracker.analyze_memory_layout(type_name, size).unwrap();
        
        assert!(layout.container_analysis.is_some(), "Container analysis missing for {}", type_name);
        
        let container_analysis = layout.container_analysis.unwrap();
        
        // Compare container types (using Debug format for easier comparison)
        assert_eq!(
            format!("{:?}", container_analysis.container_type),
            format!("{:?}", expected_container_type),
            "Container type mismatch for {}",
            type_name
        );
    }
}

#[test]
fn test_non_container_types_no_analysis() {
    let tracker = MemoryTracker::new();
    
    // Test that non-container types don't get container analysis
    let non_container_types = vec![
        ("i32", 4),
        ("f64", 8),
        ("bool", 1),
        ("CustomStruct", 32),
    ];
    
    for (type_name, size) in non_container_types {
        let layout = tracker.analyze_memory_layout(type_name, size).unwrap();
        
        // Non-container types should not have container analysis
        assert!(layout.container_analysis.is_none(), 
            "Container analysis should be None for non-container type: {}", type_name);
    }
}

#[test]
fn test_capacity_utilization_efficiency_assessment() {
    let tracker = MemoryTracker::new();
    
    // Test different Vec sizes to verify utilization efficiency assessment
    let test_cases = vec![
        (32, "small Vec"),
        (128, "medium Vec"),
        (512, "large Vec"),
        (2048, "very large Vec"),
    ];
    
    for (size, description) in test_cases {
        let layout = tracker.analyze_memory_layout("Vec<i32>", size).unwrap();
        let container_analysis = layout.container_analysis.unwrap();
        
        let utilization = &container_analysis.capacity_utilization;
        
        // Verify utilization ratio is valid
        assert!(utilization.utilization_ratio >= 0.0, 
            "Invalid utilization ratio for {}: {}", description, utilization.utilization_ratio);
        assert!(utilization.utilization_ratio <= 1.0, 
            "Invalid utilization ratio for {}: {}", description, utilization.utilization_ratio);
        
        // Verify efficiency assessment is valid
        match utilization.efficiency_assessment {
            UtilizationEfficiency::Excellent | 
            UtilizationEfficiency::Good | 
            UtilizationEfficiency::Fair | 
            UtilizationEfficiency::Poor { .. } => {
                // All variants are valid
            }
        }
        
        // Verify wasted space calculation
        if utilization.current_capacity > utilization.current_length {
            assert!(utilization.wasted_space > 0, 
                "Expected wasted space for {}", description);
        }
    }
}

#[test]
fn test_reallocation_pattern_optimization_suggestions() {
    let tracker = MemoryTracker::new();
    
    // Test that reallocation patterns include optimization suggestions
    let layout = tracker.analyze_memory_layout("Vec<i64>", 512).unwrap();
    let container_analysis = layout.container_analysis.unwrap();
    
    let patterns = &container_analysis.reallocation_patterns;
    
    // Should have optimization suggestions for Vec
    assert!(!patterns.optimization_suggestions.is_empty(), 
        "Expected optimization suggestions for Vec");
    
    // Should contain relevant suggestions
    let suggestions_text = patterns.optimization_suggestions.join(" ");
    assert!(suggestions_text.contains("with_capacity") || 
            suggestions_text.contains("SmallVec") ||
            suggestions_text.contains("capacity"),
        "Expected relevant optimization suggestions, got: {:?}", 
        patterns.optimization_suggestions);
}