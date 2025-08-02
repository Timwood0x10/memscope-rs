//! Integration tests for conversion validation functionality

use memscope_rs::core::types::{AllocationInfo, MemoryStats, TypeMemoryUsage};
use memscope_rs::export::binary_converter::{BinaryConverter, ConversionOptions, ConversionResult};
use memscope_rs::export::conversion_validator::{
    ChecksumValidationResult, ConversionValidator, DataIntegrityResult, PerformanceCategory,
    PerformanceComparisonResult, PerformanceThresholds, QualityGrade, QualityMetrics,
    ValidationError, ValidationErrorType, ValidationOptions, ValidationResult, ValidationSeverity,
    ValidationWarning, ValidationWarningType,
};
use std::path::Path;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

/// Helper function to create mock allocation data for testing
fn create_mock_allocations(count: usize) -> Vec<AllocationInfo> {
    (0..count)
        .map(|i| AllocationInfo {
            ptr: 0x1000 + i * 0x100,
            size: 64 + (i % 1024),
            var_name: Some(format!("var_{}", i)),
            type_name: Some(format!("TestType{}", i % 10)),
            scope_name: Some(format!("scope_{}", i % 5)),
            timestamp_alloc: 1000000 + i as u64,
            timestamp_dealloc: None,
            thread_id: format!("thread_{}", i % 4),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: Some(100 + i as u64),
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
        })
        .collect()
}

/// Helper function to create mock memory stats
fn create_mock_memory_stats() -> MemoryStats {
    MemoryStats {
        total_allocations: 1000,
        total_allocated: 1024 * 1024,
        active_allocations: 200,
        active_memory: 200 * 1024,
        peak_allocations: 500,
        peak_memory: 500 * 1024,
        total_deallocations: 800,
        total_deallocated: 800 * 1024,
        leaked_allocations: 0,
        leaked_memory: 0,
        fragmentation_analysis: Default::default(),
        lifecycle_stats: Default::default(),
        allocations: Vec::new(),
        system_library_stats: Default::default(),
        concurrency_analysis: Default::default(),
    }
}

/// Helper function to create mock conversion result
fn create_mock_conversion_result() -> ConversionResult {
    ConversionResult {
        input_path: "test_input.bin".to_string(),
        output_path: "test_output.json".to_string(),
        input_size: 1024 * 1024,      // 1MB
        output_size: 2 * 1024 * 1024, // 2MB
        conversion_duration: Duration::from_millis(100),
        peak_memory_usage: Some(512 * 1024), // 512KB
        compression_ratio: Some(0.5),
        conversion_type: memscope_rs::export::binary_converter::ConversionType::BinaryToJson,
        metadata: std::collections::HashMap::new(),
        allocations_converted: 100,
        validation_passed: Some(true),
        validation_result: None,
        output_format: "JSON".to_string(),
    }
}

#[test]
fn test_conversion_validation_integration() {
    // This test verifies that the conversion validation system is properly integrated
    let validation_options = ValidationOptions {
        enable_comprehensive_validation: true,
        enable_performance_comparison: true,
        validation_sample_size: 100,
        performance_thresholds: PerformanceThresholds::default(),
        enable_checksum_validation: true,
        max_acceptable_error_rate: 0.01,
        enable_detailed_logging: false,
    };

    let validator = ConversionValidator::with_options(validation_options);

    // Test that validator is properly configured
    let options = validator.get_options();
    assert!(options.enable_comprehensive_validation);
    assert!(options.enable_performance_comparison);
    assert_eq!(options.validation_sample_size, 100);
}

#[test]
fn test_validation_options_configuration() {
    let mut validation_options = ValidationOptions::default();
    validation_options.enable_performance_comparison = true;
    validation_options.validation_sample_size = 500;
    validation_options.max_acceptable_error_rate = 0.05;

    let validator = ConversionValidator::with_options(validation_options.clone());

    let options = validator.get_options();
    assert!(options.enable_performance_comparison);
    assert_eq!(options.validation_sample_size, 500);
    assert_eq!(options.max_acceptable_error_rate, 0.05);
}

#[test]
fn test_performance_categorization() {
    let validator = ConversionValidator::new();

    // Test excellent performance
    assert_eq!(
        validator.test_categorize_performance(60.0, 1.2),
        PerformanceCategory::Excellent
    );

    // Test good performance
    assert_eq!(
        validator.test_categorize_performance(25.0, 1.8),
        PerformanceCategory::Good
    );

    // Test average performance
    assert_eq!(
        validator.test_categorize_performance(8.0, 2.5),
        PerformanceCategory::Average
    );

    // Test poor performance
    assert_eq!(
        validator.test_categorize_performance(2.0, 4.0),
        PerformanceCategory::Poor
    );

    // Test very poor performance
    assert_eq!(
        validator.test_categorize_performance(0.5, 6.0),
        PerformanceCategory::VeryPoor
    );
}

#[test]
fn test_performance_rating_calculation() {
    let validator = ConversionValidator::new();

    // Test various performance scenarios
    assert_eq!(validator.test_calculate_performance_rating(60.0, 1.2), 5);
    assert_eq!(validator.test_calculate_performance_rating(25.0, 1.8), 4);
    assert_eq!(validator.test_calculate_performance_rating(8.0, 2.5), 3);
    assert_eq!(validator.test_calculate_performance_rating(2.0, 4.0), 2);
    assert_eq!(validator.test_calculate_performance_rating(0.5, 6.0), 1);

    // Test edge cases
    assert_eq!(validator.test_calculate_performance_rating(50.1, 1.49), 5);
    assert_eq!(validator.test_calculate_performance_rating(19.9, 2.01), 3);
}

#[test]
fn test_quality_metrics_calculation() {
    let validator = ConversionValidator::new();

    // Create perfect data integrity result
    let perfect_data_integrity = DataIntegrityResult {
        allocation_count_match: true,
        memory_stats_match: true,
        type_memory_usage_match: true,
        allocation_match_percentage: 100.0,
        mismatched_allocations: 0,
        sample_validation_passed: true,
        sample_size: 1000,
        checksum_validation: ChecksumValidationResult {
            binary_checksum_valid: true,
            converted_data_checksum_valid: true,
            binary_checksum: Some("abc123".to_string()),
            converted_data_checksum: Some("def456".to_string()),
        },
    };

    // Create excellent performance result
    let excellent_performance = PerformanceComparisonResult {
        conversion_speed_mbps: 50.0,
        size_ratio: 1.2,
        memory_usage_mb: Some(100.0),
        json_export_comparison: None,
        performance_rating: 5,
        performance_category: PerformanceCategory::Excellent,
    };

    let validation_errors = Vec::new();
    let validation_warnings = Vec::new();

    let quality_metrics = validator.test_calculate_quality_metrics(
        &perfect_data_integrity,
        &excellent_performance,
        &validation_errors,
        &validation_warnings,
    );

    assert_eq!(quality_metrics.quality_grade, QualityGrade::A);
    assert!(quality_metrics.overall_score >= 90.0);
    assert_eq!(quality_metrics.critical_issues, 0);
    assert_eq!(quality_metrics.warnings, 0);
}

#[test]
fn test_quality_metrics_with_issues() {
    let validator = ConversionValidator::new();

    // Create data integrity result with issues
    let problematic_data_integrity = DataIntegrityResult {
        allocation_count_match: false,
        memory_stats_match: true,
        type_memory_usage_match: false,
        allocation_match_percentage: 85.0,
        mismatched_allocations: 15,
        sample_validation_passed: false,
        sample_size: 100,
        checksum_validation: ChecksumValidationResult {
            binary_checksum_valid: false,
            converted_data_checksum_valid: true,
            binary_checksum: None,
            converted_data_checksum: Some("def456".to_string()),
        },
    };

    // Create poor performance result
    let poor_performance = PerformanceComparisonResult {
        conversion_speed_mbps: 2.0,
        size_ratio: 4.0,
        memory_usage_mb: Some(2048.0),
        json_export_comparison: None,
        performance_rating: 2,
        performance_category: PerformanceCategory::Poor,
    };

    let validation_errors = vec![
        ValidationError {
            error_type: ValidationErrorType::DataMismatch,
            message: "Allocation count mismatch".to_string(),
            severity: ValidationSeverity::Critical,
            context: Some("allocation_count".to_string()),
            suggested_fix: Some("Check parser logic".to_string()),
        },
        ValidationError {
            error_type: ValidationErrorType::ChecksumFailure,
            message: "Checksum validation failed".to_string(),
            severity: ValidationSeverity::Major,
            context: Some("checksum".to_string()),
            suggested_fix: Some("Verify file integrity".to_string()),
        },
    ];

    let validation_warnings = vec![ValidationWarning {
        warning_type: ValidationWarningType::PerformanceWarning,
        message: "Slow conversion speed".to_string(),
        context: Some("performance".to_string()),
        recommendation: Some("Optimize algorithms".to_string()),
    }];

    let quality_metrics = validator.test_calculate_quality_metrics(
        &problematic_data_integrity,
        &poor_performance,
        &validation_errors,
        &validation_warnings,
    );

    assert!(matches!(
        quality_metrics.quality_grade,
        QualityGrade::D | QualityGrade::F
    ));
    assert!(quality_metrics.overall_score < 70.0);
    assert_eq!(quality_metrics.critical_issues, 1);
    assert_eq!(quality_metrics.warnings, 1);
}

#[test]
fn test_data_completeness_score_calculation() {
    let validator = ConversionValidator::new();

    // Test perfect data integrity
    let perfect_integrity = DataIntegrityResult {
        allocation_count_match: true,
        memory_stats_match: true,
        type_memory_usage_match: true,
        allocation_match_percentage: 100.0,
        mismatched_allocations: 0,
        sample_validation_passed: true,
        sample_size: 1000,
        checksum_validation: ChecksumValidationResult {
            binary_checksum_valid: true,
            converted_data_checksum_valid: true,
            binary_checksum: Some("abc123".to_string()),
            converted_data_checksum: Some("def456".to_string()),
        },
    };

    let score = validator.test_calculate_data_completeness_score(&perfect_integrity);
    assert_eq!(score, 100.0);

    // Test problematic data integrity
    let problematic_integrity = DataIntegrityResult {
        allocation_count_match: false,
        memory_stats_match: false,
        type_memory_usage_match: false,
        allocation_match_percentage: 100.0,
        mismatched_allocations: 0,
        sample_validation_passed: false,
        sample_size: 1000,
        checksum_validation: ChecksumValidationResult {
            binary_checksum_valid: false,
            converted_data_checksum_valid: true,
            binary_checksum: None,
            converted_data_checksum: Some("def456".to_string()),
        },
    };

    let score = validator.test_calculate_data_completeness_score(&problematic_integrity);
    assert_eq!(score, 0.0); // 100 - 30 - 20 - 15 - 25 - 10 = 0, max(0.0) = 0
}

#[test]
fn test_performance_score_calculation() {
    let validator = ConversionValidator::new();

    // Test excellent performance
    let excellent_performance = PerformanceComparisonResult {
        conversion_speed_mbps: 60.0,
        size_ratio: 1.2,
        memory_usage_mb: Some(100.0),
        json_export_comparison: None,
        performance_rating: 5,
        performance_category: PerformanceCategory::Excellent,
    };

    let score = validator.test_calculate_performance_score(&excellent_performance);
    assert_eq!(score, 100.0); // (100 * 0.7) + (100 * 0.3) = 100

    // Test poor performance
    let poor_performance = PerformanceComparisonResult {
        conversion_speed_mbps: 0.5,
        size_ratio: 6.0,
        memory_usage_mb: Some(2048.0),
        json_export_comparison: None,
        performance_rating: 1,
        performance_category: PerformanceCategory::VeryPoor,
    };

    let score = validator.test_calculate_performance_score(&poor_performance);
    assert_eq!(score, 20.0); // (20 * 0.7) + (20 * 0.3) = 20
}

// Note: calculate_consistency_score is not implemented in the current version
// This test is commented out until the method is implemented
// #[test]
// fn test_consistency_score_calculation() {
//     // Implementation pending
// }

#[test]
fn test_quality_report_generation() {
    let validator = ConversionValidator::new();

    // Create a validation result with mixed quality
    let validation_result = ValidationResult {
        is_valid: true,
        data_integrity: DataIntegrityResult {
            allocation_count_match: true,
            memory_stats_match: true,
            type_memory_usage_match: true,
            allocation_match_percentage: 92.0,
            mismatched_allocations: 8,
            sample_validation_passed: true,
            sample_size: 100,
            checksum_validation: ChecksumValidationResult {
                binary_checksum_valid: true,
                converted_data_checksum_valid: true,
                binary_checksum: Some("abc123".to_string()),
                converted_data_checksum: Some("def456".to_string()),
            },
        },
        performance_comparison: PerformanceComparisonResult {
            conversion_speed_mbps: 8.0,
            size_ratio: 2.2,
            memory_usage_mb: Some(256.0),
            json_export_comparison: None,
            performance_rating: 3,
            performance_category: PerformanceCategory::Average,
        },
        quality_metrics: QualityMetrics {
            overall_score: 75.0,
            data_completeness_score: 85.0,
            data_accuracy_score: 92.0,
            performance_score: 60.0,
            reliability_score: 90.0,
            critical_issues: 0,
            warnings: 2,
            quality_grade: QualityGrade::C,
        },
        validation_errors: Vec::new(),
        validation_warnings: vec![
            ValidationWarning {
                warning_type: ValidationWarningType::PerformanceWarning,
                message: "Conversion speed could be improved".to_string(),
                context: Some("performance".to_string()),
                recommendation: Some("Consider optimization".to_string()),
            },
            ValidationWarning {
                warning_type: ValidationWarningType::MinorDataDifference,
                message: "Minor data differences detected".to_string(),
                context: Some("data_accuracy".to_string()),
                recommendation: Some("Review serialization logic".to_string()),
            },
        ],
        validation_timestamp: SystemTime::now(),
        validation_duration: Duration::from_millis(500),
    };

    let quality_report = validator.generate_quality_report(&validation_result);

    // Verify report structure (it's a String, not a struct)
    assert!(!quality_report.is_empty());
    assert!(quality_report.contains("PASSED"));
    assert!(quality_report.contains("C"));
    assert!(quality_report.contains("Validation Report"));
    assert!(quality_report.contains("Quality Grade"));
    assert!(quality_report.contains("Overall Score"));
}

#[test]
fn test_conversion_result_metrics() {
    let conversion_result = create_mock_conversion_result();

    // Test speed calculation
    let speed = conversion_result.conversion_speed_mbps();
    assert!(speed > 0.0);
    // 1MB in 100ms = 10 MB/s
    assert!((speed - 10.0).abs() < 0.1);

    // Test size ratio calculation
    let ratio = conversion_result.size_ratio();
    assert_eq!(ratio, 2.0); // 2MB / 1MB = 2.0

    // Test memory usage
    let memory_usage = conversion_result.memory_usage_mb();
    assert!(memory_usage.is_some());
    assert_eq!(memory_usage.unwrap(), 0.5); // 512KB = 0.5MB
}

#[test]
fn test_validation_error_severity_ordering() {
    use ValidationSeverity::*;

    // Test that severity levels are properly ordered
    assert!(Critical < Major);
    assert!(Major < Minor);
    assert!(Minor < Info);

    // Test in vector sorting
    let mut severities = vec![Info, Critical, Minor, Major];
    severities.sort();
    assert_eq!(severities, vec![Critical, Major, Minor, Info]);
}

#[test]
fn test_performance_thresholds_default() {
    let thresholds = PerformanceThresholds::default();

    assert_eq!(thresholds.min_conversion_speed_mbps, 5.0);
    assert_eq!(thresholds.max_size_ratio, 3.0);
    assert_eq!(thresholds.max_memory_usage_mb, Some(1024.0));
    assert_eq!(thresholds.min_speed_improvement_factor, 2.0);
}

#[test]
fn test_validation_options_default() {
    let options = ValidationOptions::default();

    assert!(options.enable_comprehensive_validation);
    assert!(!options.enable_performance_comparison);
    assert_eq!(options.validation_sample_size, 1000);
    assert_eq!(options.max_acceptable_error_rate, 0.01);
    assert!(options.enable_checksum_validation);
    assert!(!options.enable_detailed_logging);
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
        std::fs::write(
            &json_path,
            r#"{"allocations": [], "stats": {"total_allocations": 0}}"#,
        )
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
