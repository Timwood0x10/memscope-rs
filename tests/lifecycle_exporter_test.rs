use memscope_rs::core::types::AllocationInfo;
use memscope_rs::export::{LifecycleExporter, LifecycleExportConfig};
use std::fs;
use std::path::Path;

#[test]
fn test_lifecycle_export() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for test output
    let temp_dir = tempfile::tempdir()?;
    let output_path = temp_dir.path().join("lifecycle_test.json");

    // Create test allocations
    let allocations = vec![
        AllocationInfo {
            ptr: 0x1000,
            size: 1024,
            timestamp_alloc: 1000,
            timestamp_dealloc: Some(2000),
            type_name: Some("TestType".to_string()),
            var_name: Some("test_var".to_string()),
            stack_trace: Vec::new(),
            scope_name: None,
        },
        AllocationInfo {
            ptr: 0x2000,
            size: 2048,
            timestamp_alloc: 1500,
            timestamp_dealloc: None,
            type_name: Some("AnotherType".to_string()),
            var_name: Some("another_var".to_string()),
            stack_trace: Vec::new(),
            scope_name: None,
        },
    ];

    // Configure and run exporter
    let config = LifecycleExportConfig {
        include_system_allocations: true,
        pretty_print: true,
        batch_size: 100,
    };

    let exporter = LifecycleExporter::new(config);
    let stats = exporter.export_lifecycle_data(&allocations, &output_path)?;

    // Verify export stats
    assert_eq!(stats.objects_exported, 2);
    assert!(stats.processing_time.as_millis() < 100); // Should be fast
    assert!(stats.output_size > 0);

    // Verify file content
    let content = fs::read_to_string(&output_path)?;
    assert!(content.contains("test_var"));
    assert!(content.contains("TestType"));
    assert!(content.contains("another_var"));
    assert!(content.contains("AnotherType"));
    assert!(content.contains("allocation_ptr"));
    assert!(content.contains("ownership_history"));

    // Clean up
    temp_dir.close()?;
    Ok(())
}
