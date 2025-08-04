//! Integration test for binary format advanced metrics functionality

use memscope_rs::core::types::AllocationInfo;
use memscope_rs::export::binary::{
    export_to_binary_with_config, AdvancedMetricsLevel, BinaryExportConfig, BinaryReader,
};

use tempfile::NamedTempFile;

fn create_test_allocations() -> Vec<AllocationInfo> {
    vec![
        AllocationInfo {
            ptr: 0x1000,
            size: 1024,
            var_name: Some("test_vec".to_string()),
            type_name: Some("Vec<i32>".to_string()),
            scope_name: Some("main".to_string()),
            timestamp_alloc: 1000000000,
            timestamp_dealloc: Some(1000001500),
            thread_id: "main".to_string(),
            borrow_count: 2,
            stack_trace: Some(vec!["main".to_string(), "allocate_vec".to_string()]),
            is_leaked: false,
            lifetime_ms: Some(1500), // 1.5 seconds
            smart_pointer_info: None,
            memory_layout: None, // Simplified for testing
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None, // Simplified for testing
            function_call_tracking: None,
            lifecycle_tracking: None, // Simplified for testing
            access_tracking: None,
        },
        AllocationInfo {
            ptr: 0x2000,
            size: 256,
            var_name: Some("test_string".to_string()),
            type_name: Some("String".to_string()),
            scope_name: Some("helper".to_string()),
            timestamp_alloc: 1000000500,
            timestamp_dealloc: Some(1000000800),
            thread_id: "worker-1".to_string(),
            borrow_count: 1,
            stack_trace: Some(vec!["helper".to_string(), "create_string".to_string()]),
            is_leaked: false,
            lifetime_ms: Some(300), // 0.3 seconds
            smart_pointer_info: None,
            memory_layout: None, // Simplified for testing
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None, // Simplified for testing
            function_call_tracking: None,
            lifecycle_tracking: None, // Simplified for testing
            access_tracking: None,
        },
    ]
}

#[test]
fn test_binary_export_with_comprehensive_advanced_metrics() {
    let temp_file = NamedTempFile::new().unwrap();
    let allocations = create_test_allocations();

    // Create comprehensive configuration
    let config = BinaryExportConfig::debug_comprehensive();

    // Export with advanced metrics
    export_to_binary_with_config(&allocations, temp_file.path(), &config)
        .expect("Failed to export binary with advanced metrics");

    // Read back and verify
    let mut reader = BinaryReader::new(temp_file.path()).expect("Failed to create binary reader");

    let read_allocations = reader.read_all().expect("Failed to read allocations");

    // Verify basic allocation data
    assert_eq!(read_allocations.len(), 2);
    assert_eq!(read_allocations[0].ptr, 0x1000);
    assert_eq!(read_allocations[0].size, 1024);
    assert_eq!(read_allocations[0].lifetime_ms, Some(1500));
    assert_eq!(read_allocations[1].ptr, 0x2000);
    assert_eq!(read_allocations[1].size, 256);
    assert_eq!(read_allocations[1].lifetime_ms, Some(300));

    // Verify advanced metrics were read
    let advanced_metrics = reader
        .get_advanced_metrics()
        .expect("Advanced metrics should be present");

    // Check lifecycle metrics
    assert_eq!(advanced_metrics.lifecycle_metrics.len(), 2);
    assert!(advanced_metrics.lifecycle_metrics.contains_key(&0x1000));
    assert!(advanced_metrics.lifecycle_metrics.contains_key(&0x2000));

    let lifecycle_1 = &advanced_metrics.lifecycle_metrics[&0x1000];
    assert_eq!(lifecycle_1.lifetime_ms, 1500);
    assert!(lifecycle_1.lifecycle_tracking.is_none()); // Set to None in test data

    let lifecycle_2 = &advanced_metrics.lifecycle_metrics[&0x2000];
    assert_eq!(lifecycle_2.lifetime_ms, 300);
    assert!(lifecycle_2.lifecycle_tracking.is_none()); // Set to None in test data

    // Check container metrics (should be empty since we set memory_layout to None)
    assert_eq!(advanced_metrics.container_metrics.len(), 0);

    // Check type usage metrics (should be empty since we set type_usage to None)
    assert_eq!(advanced_metrics.type_usage_metrics.len(), 0);
}

#[test]
fn test_binary_export_with_minimal_config() {
    let temp_file = NamedTempFile::new().unwrap();
    let allocations = create_test_allocations();

    // Create minimal configuration (no advanced metrics)
    let config = BinaryExportConfig::minimal();

    // Export without advanced metrics
    export_to_binary_with_config(&allocations, temp_file.path(), &config)
        .expect("Failed to export binary with minimal config");

    // Read back and verify
    let mut reader = BinaryReader::new(temp_file.path()).expect("Failed to create binary reader");

    let read_allocations = reader.read_all().expect("Failed to read allocations");

    // Verify basic allocation data is still present
    assert_eq!(read_allocations.len(), 2);
    assert_eq!(read_allocations[0].ptr, 0x1000);
    assert_eq!(read_allocations[0].size, 1024);

    // Verify no advanced metrics were written
    let advanced_metrics = reader.get_advanced_metrics();
    assert!(
        advanced_metrics.is_none(),
        "Advanced metrics should not be present with minimal config"
    );
}

#[test]
fn test_binary_export_with_performance_first_config() {
    let temp_file = NamedTempFile::new().unwrap();
    let allocations = create_test_allocations();

    // Create performance-first configuration (selective advanced metrics)
    let config = BinaryExportConfig::performance_first();

    // Export with selective advanced metrics
    export_to_binary_with_config(&allocations, temp_file.path(), &config)
        .expect("Failed to export binary with performance-first config");

    // Read back and verify
    let mut reader = BinaryReader::new(temp_file.path()).expect("Failed to create binary reader");

    let read_allocations = reader.read_all().expect("Failed to read allocations");

    // Verify basic allocation data
    assert_eq!(read_allocations.len(), 2);

    // Verify selective advanced metrics were written
    let advanced_metrics = reader
        .get_advanced_metrics()
        .expect("Some advanced metrics should be present with performance-first config");

    // Performance-first config should include lifecycle analysis
    assert_eq!(advanced_metrics.lifecycle_metrics.len(), 2);
    // Container and type usage metrics should be empty since we set the fields to None
    assert_eq!(advanced_metrics.container_metrics.len(), 0);
    assert_eq!(advanced_metrics.type_usage_metrics.len(), 0);
}

#[test]
fn test_configuration_validation() {
    // Test that configuration validation works
    let mut config = BinaryExportConfig::new();
    config.buffer_size = 100; // Too small
    config.compression_level = 15; // Too high
    config.advanced_metrics_level = AdvancedMetricsLevel::None;
    config.source_analysis = true; // Conflicting with None level

    let warnings = config.validate_and_fix();

    // Should have warnings about buffer size, compression level, and conflicting settings
    assert!(!warnings.is_empty());
    assert!(warnings.iter().any(|w| w.contains("Buffer size")));
    assert!(warnings.iter().any(|w| w.contains("Compression level")));
    assert!(warnings
        .iter()
        .any(|w| w.contains("Advanced metrics level is None")));

    // Values should be fixed
    assert_eq!(config.buffer_size, 1024); // Fixed to minimum
    assert_eq!(config.compression_level, 9); // Fixed to maximum
    assert!(!config.source_analysis); // Disabled due to conflict
}

#[test]
fn test_performance_impact_estimation() {
    let minimal = BinaryExportConfig::minimal();
    let performance = BinaryExportConfig::performance_first();
    let comprehensive = BinaryExportConfig::debug_comprehensive();

    let minimal_impact = minimal.estimated_performance_impact();
    let performance_impact = performance.estimated_performance_impact();
    let comprehensive_impact = comprehensive.estimated_performance_impact();

    // Impact should increase with more features
    assert!(minimal_impact < performance_impact);
    assert!(performance_impact < comprehensive_impact);

    // Minimal should have very low impact
    assert!(minimal_impact < 0.1);

    // Comprehensive should have significant impact
    assert!(comprehensive_impact > 0.5);
}
