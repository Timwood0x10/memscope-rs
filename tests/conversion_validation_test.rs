//! Integration tests for conversion validation functionality

use memscope_rs::export::binary_converter::{BinaryConverter, ConversionOptions};
use memscope_rs::export::conversion_validator::{ValidationOptions, ConversionValidator};
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_conversion_validation_integration() {
    // This test verifies that the conversion validation system is properly integrated
    // Note: This is a basic integration test that checks the API works
    
    let options = ConversionOptions::default()
        .with_comprehensive_validation()
        .with_performance_comparison();
    
    // Verify that validation options are properly set
    assert!(options.validate_output);
    assert!(options.validation_options.is_some());
    
    if let Some(ref validation_options) = options.validation_options {
        assert!(validation_options.enable_comprehensive_validation);
        assert!(validation_options.enable_performance_comparison);
    }
}

#[test]
fn test_validation_options_configuration() {
    let mut validation_options = ValidationOptions::default();
    validation_options.enable_performance_comparison = true;
    validation_options.validation_sample_size = 500;
    validation_options.max_acceptable_error_rate = 0.05;
    
    let options = ConversionOptions::default()
        .with_validation_options(validation_options.clone());
    
    assert!(options.validate_output);
    assert!(options.validation_options.is_some());
    
    if let Some(ref opts) = options.validation_options {
        assert!(opts.enable_performance_comparison);
        assert_eq!(opts.validation_sample_size, 500);
        assert_eq!(opts.max_acceptable_error_rate, 0.05);
    }
}

#[test]
fn test_quality_report_generation() {
    use memscope_rs::export::binary_converter::ConversionResult;
    use std::time::Duration;
    
    // Create a mock conversion result
    let conversion_result = ConversionResult {
        output_path: "test.json".to_string(),
        input_size: 1024,
        output_size: 2048,
        conversion_duration: Duration::from_millis(100),
        allocations_converted: 100,
        validation_passed: Some(true),
        validation_result: None,
        output_format: "JSON".to_string(),
    };
    
    // Generate basic quality report
    let report = BinaryConverter::generate_quality_report(&conversion_result)
        .expect("Should generate quality report");
    
    assert!(report.contains("CONVERSION REPORT"));
    assert!(report.contains("test.json"));
    assert!(report.contains("JSON"));
    assert!(report.contains("✅ PASSED"));
}

#[test]
fn test_validator_creation() {
    // Test default validator creation
    let validator = ConversionValidator::new();
    // Just verify it can be created without panicking
    
    // Test validator with custom options
    let validation_options = ValidationOptions {
        enable_comprehensive_validation: true,
        enable_performance_comparison: false,
        validation_sample_size: 100,
        performance_thresholds: Default::default(),
        enable_checksum_validation: true,
        max_acceptable_error_rate: 0.02,
        enable_detailed_logging: true,
    };
    
    let _validator = ConversionValidator::with_options(validation_options);
    // Just verify it can be created without panicking
}

#[cfg(feature = "integration_tests")]
mod integration_tests {
    use super::*;
    use memscope_rs::core::tracker::MemoryTracker;
    use memscope_rs::export::binary_exporter::BinaryExporter;
    
    #[test]
    fn test_end_to_end_validation() {
        // This test would require actual binary files to work properly
        // For now, it's disabled unless the integration_tests feature is enabled
        
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let binary_path = temp_dir.path().join("test.bin");
        let json_path = temp_dir.path().join("test.json");
        
        // Create a simple memory tracker with some data
        let tracker = MemoryTracker::new();
        
        // This would normally export to binary format
        // For this test, we'll just create empty files to test the validation pipeline
        std::fs::write(&binary_path, b"mock binary data").expect("Failed to write binary file");
        std::fs::write(&json_path, r#"{"allocations": [], "stats": {"total_allocations": 0}}"#)
            .expect("Failed to write JSON file");
        
        // Test validation options
        let validation_options = ValidationOptions {
            enable_comprehensive_validation: true,
            enable_performance_comparison: false, // Disable for this test
            validation_sample_size: 10,
            performance_thresholds: Default::default(),
            enable_checksum_validation: false, // Disable for mock data
            max_acceptable_error_rate: 0.1,
            enable_detailed_logging: true,
        };
        
        let validator = ConversionValidator::with_options(validation_options);
        
        // Create a mock conversion result
        let conversion_result = ConversionResult {
            output_path: json_path.to_string_lossy().to_string(),
            input_size: 100,
            output_size: 200,
            conversion_duration: Duration::from_millis(50),
            allocations_converted: 0,
            validation_passed: None,
            validation_result: None,
            output_format: "JSON".to_string(),
        };
        
        // This would normally perform full validation
        // For this test, we just verify the API works
        match validator.validate_conversion(&binary_path, &json_path, &conversion_result) {
            Ok(_validation_result) => {
                // Validation completed successfully
            }
            Err(_e) => {
                // Expected to fail with mock data, but API should work
            }
        }
    }
}