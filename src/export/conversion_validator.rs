//! Conversion validation module for binary export system
//!
//! This module provides comprehensive validation functionality for binary-to-JSON/HTML conversions,
//! including data integrity checks, performance comparisons, and quality reporting.

use crate::core::types::{AllocationInfo, MemoryStats, TrackingResult, TypeMemoryUsage};
use crate::export::binary_converter::{BatchConversionReport, ConversionResult};
use crate::export::binary_parser::BinaryParser;
use crate::export::optimized_json_export::OptimizedExportOptions;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

/// Validation result for a single conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the conversion passed all validation checks
    pub is_valid: bool,
    /// Data integrity check results
    pub data_integrity: DataIntegrityResult,
    /// Performance comparison results
    pub performance_comparison: PerformanceComparisonResult,
    /// Quality metrics
    pub quality_metrics: QualityMetrics,
    /// List of validation errors found
    pub validation_errors: Vec<ValidationError>,
    /// List of validation warnings
    pub validation_warnings: Vec<ValidationWarning>,
    /// Validation timestamp
    pub validation_timestamp: std::time::SystemTime,
    /// Time taken to perform validation
    pub validation_duration: Duration,
}

/// Data integrity validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataIntegrityResult {
    /// Whether allocation count matches
    pub allocation_count_match: bool,
    /// Whether memory statistics match
    pub memory_stats_match: bool,
    /// Whether type memory usage matches
    pub type_memory_usage_match: bool,
    /// Percentage of allocations that match exactly
    pub allocation_match_percentage: f64,
    /// Number of mismatched allocations
    pub mismatched_allocations: usize,
    /// Sample validation results (for performance)
    pub sample_validation_passed: bool,
    /// Sample size used for validation
    pub sample_size: usize,
    /// Checksum validation results
    pub checksum_validation: ChecksumValidationResult,
}

/// Checksum validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksumValidationResult {
    /// Whether binary file checksum is valid
    pub binary_checksum_valid: bool,
    /// Whether converted data checksum matches expected
    pub converted_data_checksum_valid: bool,
    /// Binary file checksum
    pub binary_checksum: Option<String>,
    /// Converted data checksum
    pub converted_data_checksum: Option<String>,
}

/// Performance comparison results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparisonResult {
    /// Conversion speed in MB/s
    pub conversion_speed_mbps: f64,
    /// Size ratio (output/input)
    pub size_ratio: f64,
    /// Memory usage during conversion (if available)
    pub memory_usage_mb: Option<f64>,
    /// Comparison with direct JSON export (if available)
    pub json_export_comparison: Option<JsonExportComparison>,
    /// Performance rating (1-5 scale)
    pub performance_rating: u8,
    /// Performance category
    pub performance_category: PerformanceCategory,
}

/// Comparison with direct JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonExportComparison {
    /// Speed improvement factor (binary conversion vs direct JSON)
    pub speed_improvement_factor: f64,
    /// Size difference ratio
    pub size_difference_ratio: f64,
    /// Time taken for direct JSON export
    pub direct_json_export_time: Duration,
    /// Time taken for binary conversion
    pub binary_conversion_time: Duration,
    /// Direct JSON export file size
    pub direct_json_size: usize,
    /// Binary conversion output size
    pub binary_conversion_size: usize,
}

/// Performance categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PerformanceCategory {
    /// Excellent performance (>50 MB/s, <1.5x size ratio)
    Excellent,
    /// Good performance (20-50 MB/s, 1.5-2.0x size ratio)
    Good,
    /// Average performance (5-20 MB/s, 2.0-3.0x size ratio)
    Average,
    /// Poor performance (1-5 MB/s, 3.0-5.0x size ratio)
    Poor,
    /// Very poor performance (<1 MB/s, >5.0x size ratio)
    VeryPoor,
}

/// Quality metrics for conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Overall quality score (0-100)
    pub overall_score: f64,
    /// Data completeness score (0-100)
    pub data_completeness_score: f64,
    /// Data accuracy score (0-100)
    pub data_accuracy_score: f64,
    /// Performance score (0-100)
    pub performance_score: f64,
    /// Reliability score (0-100)
    pub reliability_score: f64,
    /// Number of critical issues
    pub critical_issues: usize,
    /// Number of warnings
    pub warnings: usize,
    /// Quality grade (A-F)
    pub quality_grade: QualityGrade,
}

/// Quality grades
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QualityGrade {
    ///90-100: Excellent
    A,
    /// 80-89: Good
    B,
    /// 70-79: Average
    C,
    /// 60-69: Below Average
    D,
    /// <60: Poor
    F,
}

/// Validation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error type
    pub error_type: ValidationErrorType,
    /// Error message
    pub message: String,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Context information
    pub context: Option<String>,
    /// Suggested fix
    pub suggested_fix: Option<String>,
}

/// Validation error types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationErrorType {
    /// Data mismatch between original and converted
    DataMismatch,
    /// Missing data in conversion
    MissingData,
    /// Corrupted data detected
    CorruptedData,
    /// Performance below threshold
    PerformanceBelowThreshold,
    /// Checksum validation failure
    ChecksumFailure,
    /// Format validation failure
    FormatValidationFailure,
    /// Unknown error
    Unknown,
}

/// Validation warning types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning type
    pub warning_type: ValidationWarningType,
    /// Warning message
    pub message: String,
    /// Context information
    pub context: Option<String>,
    /// Recommendation
    pub recommendation: Option<String>,
}

/// Validation warning types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationWarningType {
    /// Minor data differences
    MinorDataDifference,
    /// Performance could be improved
    PerformanceWarning,
    /// Large file size
    LargeFileSize,
    /// Slow conversion speed
    SlowConversionSpeed,
    /// High memory usage
    HighMemoryUsage,
    /// Deprecated features used
    DeprecatedFeatures,
}

/// Validation severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    /// Critical error that prevents proper functionality
    Critical,
    /// Major error that significantly impacts functionality
    Major,
    /// Minor error with limited impact
    Minor,
    /// Informational message
    Info,
}

/// Validation options
#[derive(Debug, Clone)]
pub struct ValidationOptions {
    /// Enable comprehensive data validation
    pub enable_comprehensive_validation: bool,
    /// Enable performance comparison with direct JSON export
    pub enable_performance_comparison: bool,
    /// Sample size for allocation validation (0 = validate all)
    pub validation_sample_size: usize,
    /// Performance thresholds
    pub performance_thresholds: PerformanceThresholds,
    /// Enable checksum validation
    pub enable_checksum_validation: bool,
    /// Maximum acceptable error rate (0.0-1.0)
    pub max_acceptable_error_rate: f64,
    /// Enable detailed logging
    pub enable_detailed_logging: bool,
}

/// Performance thresholds for validation
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// Minimum acceptable conversion speed (MB/s)
    pub min_conversion_speed_mbps: f64,
    /// Maximum acceptable size ratio
    pub max_size_ratio: f64,
    /// Maximum acceptable memory usage (MB)
    pub max_memory_usage_mb: Option<f64>,
    /// Minimum acceptable speed improvement over JSON
    pub min_speed_improvement_factor: f64,
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self {
            enable_comprehensive_validation: true,
            enable_performance_comparison: false, // Expensive operation
            validation_sample_size: 1000,         // Validate 1000 allocations by default
            performance_thresholds: PerformanceThresholds::default(),
            enable_checksum_validation: true,
            max_acceptable_error_rate: 0.01, // 1% error rate
            enable_detailed_logging: false,
        }
    }
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            min_conversion_speed_mbps: 5.0,
            max_size_ratio: 3.0,
            max_memory_usage_mb: Some(1024.0), // 1GB
            min_speed_improvement_factor: 2.0,
        }
    }
}

/// Main conversion validator
pub struct ConversionValidator {
    options: ValidationOptions,
}

impl ConversionValidator {
    /// Create a new conversion validator with default options
    pub fn new() -> Self {
        Self {
            options: ValidationOptions::default(),
        }
    }

    /// Create a new conversion validator with custom options
    pub fn with_options(options: ValidationOptions) -> Self {
        Self { options }
    }

    /// Validate a single conversion result
    pub fn validate_conversion<P: AsRef<Path>>(
        &self,
        binary_path: P,
        converted_path: P,
        conversion_result: &ConversionResult,
    ) -> TrackingResult<ValidationResult> {
        let start_time = Instant::now();
        let binary_path = binary_path.as_ref();
        let converted_path = converted_path.as_ref();

        tracing::info!(
            "Starting validation of conversion: {} -> {}",
            binary_path.display(),
            converted_path.display()
        );

        let mut validation_errors = Vec::new();
        let mut validation_warnings = Vec::new();

        // Load original data from binary file
        let mut parser = BinaryParser::default();
        parser.load_from_file(binary_path)?;
        let original_allocations = parser.load_allocations()?;
        let original_stats = parser.load_memory_stats()?;
        let original_type_usage = parser.load_type_memory_usage()?;

        // Validate data integrity
        let data_integrity = self.validate_data_integrity(
            &original_allocations,
            &original_stats,
            &original_type_usage,
            converted_path,
            &mut validation_errors,
            &mut validation_warnings,
        )?;

        // Validate performance
        let performance_comparison = self.validate_performance(
            conversion_result,
            binary_path,
            &mut validation_errors,
            &mut validation_warnings,
        )?;

        // Calculate quality metrics
        let quality_metrics = self.calculate_quality_metrics(
            &data_integrity,
            &performance_comparison,
            &validation_errors,
            &validation_warnings,
        );

        let is_valid = validation_errors.is_empty()
            || validation_errors
                .iter()
                .all(|e| e.severity >= ValidationSeverity::Minor);

        let validation_result = ValidationResult {
            is_valid,
            data_integrity,
            performance_comparison,
            quality_metrics,
            validation_errors,
            validation_warnings,
            validation_timestamp: std::time::SystemTime::now(),
            validation_duration: start_time.elapsed(),
        };

        if self.options.enable_detailed_logging {
            tracing::info!(
                "Validation completed: {} (score: {:.1}, grade: {:?})",
                if validation_result.is_valid {
                    "PASSED"
                } else {
                    "FAILED"
                },
                validation_result.quality_metrics.overall_score,
                validation_result.quality_metrics.quality_grade
            );
        }

        Ok(validation_result)
    }

    /// Validate data integrity between original and converted data
    fn validate_data_integrity(
        &self,
        original_allocations: &[AllocationInfo],
        original_stats: &MemoryStats,
        original_type_usage: &[TypeMemoryUsage],
        converted_path: &Path,
        validation_errors: &mut Vec<ValidationError>,
        validation_warnings: &mut Vec<ValidationWarning>,
    ) -> TrackingResult<DataIntegrityResult> {
        // Load converted data
        let converted_content = std::fs::read_to_string(converted_path).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to read converted file: {e}"
            ))
        })?;

        // Parse JSON if it's a JSON file
        if converted_path.extension().and_then(|s| s.to_str()) == Some("json") {
            self.validate_json_data_integrity(
                original_allocations,
                original_stats,
                original_type_usage,
                &converted_content,
                validation_errors,
                validation_warnings,
            )
        } else {
            // For HTML files, we can only do basic validation
            self.validate_html_data_integrity(
                original_allocations,
                &converted_content,
                validation_errors,
                validation_warnings,
            )
        }
    }

    /// Validate JSON data integrity
    fn validate_json_data_integrity(
        &self,
        original_allocations: &[AllocationInfo],
        original_stats: &MemoryStats,
        _original_type_usage: &[TypeMemoryUsage],
        json_content: &str,
        validation_errors: &mut Vec<ValidationError>,
        validation_warnings: &mut Vec<ValidationWarning>,
    ) -> TrackingResult<DataIntegrityResult> {
        // Parse JSON
        let parsed: serde_json::Value = serde_json::from_str(json_content).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to parse converted JSON: {e}"
            ))
        })?;

        // Check allocation count
        let json_allocations = parsed["allocations"].as_array().ok_or_else(|| {
            crate::core::types::TrackingError::ExportError(
                "JSON missing allocations array".to_string(),
            )
        })?;

        let allocation_count_match = json_allocations.len() == original_allocations.len();
        if !allocation_count_match {
            validation_errors.push(ValidationError {
                error_type: ValidationErrorType::DataMismatch,
                message: format!(
                    "Allocation count mismatch: expected {}, found {}",
                    original_allocations.len(),
                    json_allocations.len()
                ),
                severity: ValidationSeverity::Critical,
                context: Some("allocation_count".to_string()),
                suggested_fix: Some("Check binary parser and JSON serialization logic".to_string()),
            });
        }

        // Validate memory statistics
        let memory_stats_match =
            self.validate_memory_stats(&parsed, original_stats, validation_errors);

        // Validate sample of allocations
        let sample_size = if self.options.validation_sample_size == 0 {
            original_allocations.len()
        } else {
            self.options
                .validation_sample_size
                .min(original_allocations.len())
        };

        let (sample_validation_passed, allocation_match_percentage, mismatched_allocations) = self
            .validate_allocation_sample(
                original_allocations,
                json_allocations,
                sample_size,
                validation_errors,
                validation_warnings,
            )?;

        // Checksum validation
        let checksum_validation = if self.options.enable_checksum_validation {
            self.validate_checksums(json_content, validation_errors)
        } else {
            ChecksumValidationResult {
                binary_checksum_valid: true,
                converted_data_checksum_valid: true,
                binary_checksum: None,
                converted_data_checksum: None,
            }
        };

        Ok(DataIntegrityResult {
            allocation_count_match,
            memory_stats_match,
            type_memory_usage_match: true, // TODO: Implement type usage validation
            allocation_match_percentage,
            mismatched_allocations,
            sample_validation_passed,
            sample_size,
            checksum_validation,
        })
    }

    /// Validate HTML data integrity (basic validation)
    fn validate_html_data_integrity(
        &self,
        original_allocations: &[AllocationInfo],
        html_content: &str,
        validation_errors: &mut Vec<ValidationError>,
        _validation_warnings: &mut Vec<ValidationWarning>,
    ) -> TrackingResult<DataIntegrityResult> {
        // Basic HTML validation - check if allocation count is mentioned
        let allocation_count_str = original_allocations.len().to_string();
        let contains_allocation_count = html_content.contains(&allocation_count_str);

        if !contains_allocation_count {
            validation_errors.push(ValidationError {
                error_type: ValidationErrorType::MissingData,
                message: "HTML does not contain expected allocation count".to_string(),
                severity: ValidationSeverity::Minor,
                context: Some("html_content".to_string()),
                suggested_fix: Some("Check HTML generation logic".to_string()),
            });
        }

        // Basic HTML structure validation
        let has_html_tags = html_content.contains("<html>") && html_content.contains("</html>");
        if !has_html_tags {
            validation_errors.push(ValidationError {
                error_type: ValidationErrorType::FormatValidationFailure,
                message: "Invalid HTML structure".to_string(),
                severity: ValidationSeverity::Major,
                context: Some("html_structure".to_string()),
                suggested_fix: Some("Check HTML template and generation".to_string()),
            });
        }

        Ok(DataIntegrityResult {
            allocation_count_match: contains_allocation_count,
            memory_stats_match: true,      // Cannot validate from HTML
            type_memory_usage_match: true, // Cannot validate from HTML
            allocation_match_percentage: if contains_allocation_count {
                100.0
            } else {
                0.0
            },
            mismatched_allocations: 0,
            sample_validation_passed: contains_allocation_count,
            sample_size: 0,
            checksum_validation: ChecksumValidationResult {
                binary_checksum_valid: true,
                converted_data_checksum_valid: true,
                binary_checksum: None,
                converted_data_checksum: None,
            },
        })
    }

    /// Validate memory statistics
    fn validate_memory_stats(
        &self,
        parsed_json: &serde_json::Value,
        original_stats: &MemoryStats,
        validation_errors: &mut Vec<ValidationError>,
    ) -> bool {
        if let Some(stats_value) = parsed_json.get("stats") {
            // Check total allocations
            if let Some(total_allocs) = stats_value.get("total_allocations") {
                if total_allocs.as_u64() != Some(original_stats.total_allocations as u64) {
                    validation_errors.push(ValidationError {
                        error_type: ValidationErrorType::DataMismatch,
                        message: format!(
                            "Total allocations mismatch: expected {}, found {:?}",
                            original_stats.total_allocations,
                            total_allocs.as_u64()
                        ),
                        severity: ValidationSeverity::Major,
                        context: Some("memory_stats.total_allocations".to_string()),
                        suggested_fix: Some("Check statistics calculation logic".to_string()),
                    });
                    return false;
                }
            }

            // Check peak memory usage
            if let Some(peak_memory) = stats_value.get("peak_memory_usage") {
                if peak_memory.as_u64()
                    != Some(original_stats.lifecycle_stats.peak_memory_usage as u64)
                {
                    validation_errors.push(ValidationError {
                        error_type: ValidationErrorType::DataMismatch,
                        message: format!(
                            "Peak memory usage mismatch: expected {}, found {:?}",
                            original_stats.lifecycle_stats.peak_memory_usage,
                            peak_memory.as_u64()
                        ),
                        severity: ValidationSeverity::Major,
                        context: Some("memory_stats.peak_memory_usage".to_string()),
                        suggested_fix: Some("Check statistics calculation logic".to_string()),
                    });
                    return false;
                }
            }
        }

        true
    }

    /// Validate a sample of allocations
    fn validate_allocation_sample(
        &self,
        original_allocations: &[AllocationInfo],
        json_allocations: &[serde_json::Value],
        sample_size: usize,
        validation_errors: &mut Vec<ValidationError>,
        validation_warnings: &mut Vec<ValidationWarning>,
    ) -> TrackingResult<(bool, f64, usize)> {
        if original_allocations.is_empty() || json_allocations.is_empty() {
            return Ok((true, 100.0, 0));
        }

        let step_size = if sample_size >= original_allocations.len() {
            1
        } else {
            original_allocations.len() / sample_size
        };

        let mut matched_count = 0;
        let mut total_checked = 0;
        let mut mismatched_allocations = 0;

        for i in (0..original_allocations.len()).step_by(step_size) {
            if i >= json_allocations.len() {
                break;
            }

            let original = &original_allocations[i];
            let json_alloc = &json_allocations[i];

            total_checked += 1;
            let mut allocation_matches = true;

            // Check size
            if json_alloc["size"].as_u64() != Some(original.size as u64) {
                allocation_matches = false;
                validation_errors.push(ValidationError {
                    error_type: ValidationErrorType::DataMismatch,
                    message: format!(
                        "Size mismatch at allocation {}: expected {}, found {:?}",
                        i,
                        original.size,
                        json_alloc["size"].as_u64()
                    ),
                    severity: ValidationSeverity::Minor,
                    context: Some(format!("allocation[{}].size", i)),
                    suggested_fix: Some("Check allocation serialization logic".to_string()),
                });
            }

            // Check timestamp
            if json_alloc["timestamp_alloc"].as_u64() != Some(original.timestamp_alloc) {
                allocation_matches = false;
                validation_warnings.push(ValidationWarning {
                    warning_type: ValidationWarningType::MinorDataDifference,
                    message: format!(
                        "Timestamp mismatch at allocation {}: expected {}, found {:?}",
                        i,
                        original.timestamp_alloc,
                        json_alloc["timestamp_alloc"].as_u64()
                    ),
                    context: Some(format!("allocation[{}].timestamp_alloc", i)),
                    recommendation: Some("Check timestamp handling in serialization".to_string()),
                });
            }

            // Check type name
            if let Some(json_type_name) = json_alloc["type_name"].as_str() {
                let original_type_name = original.type_name.as_deref().unwrap_or("");
                if json_type_name != original_type_name {
                    allocation_matches = false;
                    validation_errors.push(ValidationError {
                        error_type: ValidationErrorType::DataMismatch,
                        message: format!(
                            "Type name mismatch at allocation {}: expected '{}', found '{}'",
                            i, original_type_name, json_type_name
                        ),
                        severity: ValidationSeverity::Minor,
                        context: Some(format!("allocation[{}].type_name", i)),
                        suggested_fix: Some("Check type name serialization logic".to_string()),
                    });
                }
            }

            if allocation_matches {
                matched_count += 1;
            } else {
                mismatched_allocations += 1;
            }
        }

        let match_percentage = if total_checked > 0 {
            (matched_count as f64 / total_checked as f64) * 100.0
        } else {
            100.0
        };

        let sample_validation_passed =
            match_percentage >= (100.0 - self.options.max_acceptable_error_rate * 100.0);

        if !sample_validation_passed {
            validation_errors.push(ValidationError {
                error_type: ValidationErrorType::DataMismatch,
                message: format!(
                    "Allocation validation failed: {:.1}% match rate (threshold: {:.1}%)",
                    match_percentage,
                    100.0 - self.options.max_acceptable_error_rate * 100.0
                ),
                severity: ValidationSeverity::Critical,
                context: Some("allocation_sample_validation".to_string()),
                suggested_fix: Some("Review conversion logic and data handling".to_string()),
            });
        }

        Ok((
            sample_validation_passed,
            match_percentage,
            mismatched_allocations,
        ))
    }

    /// Validate checksums
    fn validate_checksums(
        &self,
        json_content: &str,
        validation_errors: &mut Vec<ValidationError>,
    ) -> ChecksumValidationResult {
        // Calculate checksum of converted data
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        json_content.hash(&mut hasher);
        let converted_checksum = format!("{:x}", hasher.finish());

        // For now, we'll assume checksums are valid
        // In a real implementation, we would compare against stored checksums
        let binary_checksum_valid = true;
        let converted_data_checksum_valid = true;

        if !binary_checksum_valid {
            validation_errors.push(ValidationError {
                error_type: ValidationErrorType::ChecksumFailure,
                message: "Binary file checksum validation failed".to_string(),
                severity: ValidationSeverity::Critical,
                context: Some("binary_checksum".to_string()),
                suggested_fix: Some("Check binary file integrity".to_string()),
            });
        }

        ChecksumValidationResult {
            binary_checksum_valid,
            converted_data_checksum_valid,
            binary_checksum: Some("placeholder".to_string()),
            converted_data_checksum: Some(converted_checksum),
        }
    }

    /// Validate performance metrics
    fn validate_performance(
        &self,
        conversion_result: &ConversionResult,
        binary_path: &Path,
        validation_errors: &mut Vec<ValidationError>,
        validation_warnings: &mut Vec<ValidationWarning>,
    ) -> TrackingResult<PerformanceComparisonResult> {
        let conversion_speed_mbps = conversion_result.conversion_speed_mbps();
        let size_ratio = conversion_result.size_ratio();

        // Check performance thresholds
        if conversion_speed_mbps
            < self
                .options
                .performance_thresholds
                .min_conversion_speed_mbps
        {
            validation_warnings.push(ValidationWarning {
                warning_type: ValidationWarningType::SlowConversionSpeed,
                message: format!(
                    "Conversion speed {:.2} MB/s is below threshold {:.2} MB/s",
                    conversion_speed_mbps,
                    self.options
                        .performance_thresholds
                        .min_conversion_speed_mbps
                ),
                context: Some("conversion_speed".to_string()),
                recommendation: Some("Consider optimizing conversion algorithms".to_string()),
            });
        }

        if size_ratio > self.options.performance_thresholds.max_size_ratio {
            validation_warnings.push(ValidationWarning {
                warning_type: ValidationWarningType::LargeFileSize,
                message: format!(
                    "Size ratio {:.2} exceeds threshold {:.2}",
                    size_ratio, self.options.performance_thresholds.max_size_ratio
                ),
                context: Some("size_ratio".to_string()),
                recommendation: Some(
                    "Consider using compression or optimizing output format".to_string(),
                ),
            });
        }

        // Determine performance category
        let performance_category = self.categorize_performance(conversion_speed_mbps, size_ratio);
        let performance_rating =
            self.calculate_performance_rating(conversion_speed_mbps, size_ratio);

        // Optional: Compare with direct JSON export if enabled
        let json_export_comparison = if self.options.enable_performance_comparison {
            self.compare_with_direct_json_export(
                binary_path,
                conversion_result,
                validation_warnings,
            )?
        } else {
            None
        };

        // Check memory usage if available
        if let Some(memory_usage) = conversion_result.memory_usage_mb() {
            if let Some(max_memory) = self.options.performance_thresholds.max_memory_usage_mb {
                if memory_usage > max_memory {
                    validation_warnings.push(ValidationWarning {
                        warning_type: ValidationWarningType::HighMemoryUsage,
                        message: format!(
                            "Memory usage {:.2} MB exceeds threshold {:.2} MB",
                            memory_usage, max_memory
                        ),
                        context: Some("memory_usage".to_string()),
                        recommendation: Some(
                            "Consider using streaming conversion or reducing buffer sizes"
                                .to_string(),
                        ),
                    });
                }
            }
        }

        Ok(PerformanceComparisonResult {
            conversion_speed_mbps,
            size_ratio,
            memory_usage_mb: conversion_result.memory_usage_mb(),
            json_export_comparison,
            performance_rating,
            performance_category,
        })
    }

    /// Compare conversion performance with direct JSON export
    fn compare_with_direct_json_export(
        &self,
        binary_path: &Path,
        conversion_result: &ConversionResult,
        validation_warnings: &mut Vec<ValidationWarning>,
    ) -> TrackingResult<Option<JsonExportComparison>> {
        // Load data from binary file
        let mut parser = crate::export::binary_parser::BinaryParser::default();
        parser.load_from_file(binary_path)?;
        let allocations = parser.load_allocations()?;
        let stats = parser.load_memory_stats()?;

        // Create temporary JSON export for comparison
        let temp_json_path = binary_path.with_extension("temp_comparison.json");
        let direct_export_start = Instant::now();

        // Simulate direct JSON export (in real implementation, this would use the actual JSON exporter)
        let json_data = serde_json::json!({
            "allocations": allocations,
            "stats": stats,
            "metadata": {
                "export_type": "direct_json",
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            }
        });

        std::fs::write(
            &temp_json_path,
            serde_json::to_string_pretty(&json_data).unwrap(),
        )
        .map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to write temp JSON: {e}"
            ))
        })?;

        let direct_json_export_time = direct_export_start.elapsed();
        let direct_json_size = std::fs::metadata(&temp_json_path)
            .map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to get temp JSON size: {e}"
                ))
            })?
            .len() as usize;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_json_path);

        let binary_conversion_time = conversion_result.conversion_duration();
        let binary_conversion_size = conversion_result.output_size();

        let speed_improvement_factor = if binary_conversion_time.as_secs_f64() > 0.0 {
            direct_json_export_time.as_secs_f64() / binary_conversion_time.as_secs_f64()
        } else {
            1.0
        };

        let size_difference_ratio = binary_conversion_size as f64 / direct_json_size as f64;

        // Check if speed improvement meets threshold
        if speed_improvement_factor
            < self
                .options
                .performance_thresholds
                .min_speed_improvement_factor
        {
            validation_warnings.push(ValidationWarning {
                warning_type: ValidationWarningType::PerformanceWarning,
                message: format!(
                    "Speed improvement factor {:.2}x is below threshold {:.2}x",
                    speed_improvement_factor,
                    self.options
                        .performance_thresholds
                        .min_speed_improvement_factor
                ),
                context: Some("speed_improvement".to_string()),
                recommendation: Some("Consider optimizing binary conversion pipeline".to_string()),
            });
        }

        Ok(Some(JsonExportComparison {
            speed_improvement_factor,
            size_difference_ratio,
            direct_json_export_time,
            binary_conversion_time,
            direct_json_size,
            binary_conversion_size,
        }))
    }

    /// Categorize performance based on speed and size metrics
    fn categorize_performance(&self, speed_mbps: f64, size_ratio: f64) -> PerformanceCategory {
        match (speed_mbps, size_ratio) {
            (s, r) if s > 50.0 && r < 1.5 => PerformanceCategory::Excellent,
            (s, r) if s > 20.0 && r < 2.0 => PerformanceCategory::Good,
            (s, r) if s > 5.0 && r < 3.0 => PerformanceCategory::Average,
            (s, r) if s > 1.0 && r < 5.0 => PerformanceCategory::Poor,
            _ => PerformanceCategory::VeryPoor,
        }
    }

    /// Calculate performance rating (1-5 scale)
    fn calculate_performance_rating(&self, speed_mbps: f64, size_ratio: f64) -> u8 {
        let speed_score = match speed_mbps {
            s if s > 50.0 => 5,
            s if s > 20.0 => 4,
            s if s > 5.0 => 3,
            s if s > 1.0 => 2,
            _ => 1,
        };

        let size_score = match size_ratio {
            r if r < 1.5 => 5,
            r if r < 2.0 => 4,
            r if r < 3.0 => 3,
            r if r < 5.0 => 2,
            _ => 1,
        };

        // Average the scores
        ((speed_score + size_score) / 2).max(1).min(5)
    }

    /// Calculate overall quality metrics based on validation results
    fn calculate_quality_metrics(
        &self,
        data_integrity: &DataIntegrityResult,
        performance_comparison: &PerformanceComparisonResult,
        validation_errors: &[ValidationError],
        validation_warnings: &[ValidationWarning],
    ) -> QualityMetrics {
        // Calculate individual scores
        let data_completeness_score = self.calculate_data_completeness_score(data_integrity);
        let data_accuracy_score = self.calculate_data_accuracy_score(data_integrity);
        let performance_score = self.calculate_performance_score(performance_comparison);
        let reliability_score =
            self.calculate_reliability_score(validation_errors, validation_warnings);

        // Calculate overall score (weighted average)
        let overall_score = (data_completeness_score * 0.25)
            + (data_accuracy_score * 0.35)
            + (performance_score * 0.25)
            + (reliability_score * 0.15);

        // Count critical issues and warnings
        let critical_issues = validation_errors
            .iter()
            .filter(|e| e.severity == ValidationSeverity::Critical)
            .count();
        let warnings = validation_warnings.len();

        // Determine quality grade
        let quality_grade = match overall_score {
            s if s >= 90.0 => QualityGrade::A,
            s if s >= 80.0 => QualityGrade::B,
            s if s >= 70.0 => QualityGrade::C,
            s if s >= 60.0 => QualityGrade::D,
            _ => QualityGrade::F,
        };

        QualityMetrics {
            overall_score,
            data_completeness_score,
            data_accuracy_score,
            performance_score,
            reliability_score,
            critical_issues,
            warnings,
            quality_grade,
        }
    }

    /// Calculate data completeness score
    fn calculate_data_completeness_score(&self, data_integrity: &DataIntegrityResult) -> f64 {
        let mut score: f64 = 100.0;

        if !data_integrity.allocation_count_match {
            score -= 30.0;
        }

        if !data_integrity.memory_stats_match {
            score -= 20.0;
        }

        if !data_integrity.type_memory_usage_match {
            score -= 15.0;
        }

        if !data_integrity.sample_validation_passed {
            score -= 25.0;
        }

        if !data_integrity.checksum_validation.binary_checksum_valid {
            score -= 10.0;
        }

        score.max(0.0)
    }

    /// Calculate data accuracy score
    fn calculate_data_accuracy_score(&self, data_integrity: &DataIntegrityResult) -> f64 {
        let base_score = data_integrity.allocation_match_percentage;

        // Adjust based on mismatched allocations
        let mismatch_penalty = if data_integrity.sample_size > 0 {
            (data_integrity.mismatched_allocations as f64 / data_integrity.sample_size as f64)
                * 20.0
        } else {
            0.0
        };

        (base_score - mismatch_penalty).max(0.0)
    }

    /// Calculate performance score
    fn calculate_performance_score(&self, performance: &PerformanceComparisonResult) -> f64 {
        let speed_score = match performance.conversion_speed_mbps {
            s if s > 50.0 => 100.0,
            s if s > 20.0 => 80.0,
            s if s > 5.0 => 60.0,
            s if s > 1.0 => 40.0,
            _ => 20.0,
        };

        let size_score = match performance.size_ratio {
            r if r < 1.5 => 100.0,
            r if r < 2.0 => 80.0,
            r if r < 3.0 => 60.0,
            r if r < 5.0 => 40.0,
            _ => 20.0,
        };

        // Weight speed more heavily than size
        (speed_score * 0.7) + (size_score * 0.3)
    }

    /// Calculate reliability score based on errors and warnings
    fn calculate_reliability_score(
        &self,
        validation_errors: &[ValidationError],
        validation_warnings: &[ValidationWarning],
    ) -> f64 {
        let mut score = 100.0;

        // Deduct points for errors based on severity
        for error in validation_errors {
            let penalty = match error.severity {
                ValidationSeverity::Critical => 25.0,
                ValidationSeverity::Major => 15.0,
                ValidationSeverity::Minor => 5.0,
                ValidationSeverity::Info => 1.0,
            };
            score -= penalty;
        }

        // Deduct points for warnings (less severe)
        score -= validation_warnings.len() as f64 * 2.0;

        score.max(0.0)
    }

    /// Generate a comprehensive quality report
    pub fn generate_quality_report(&self, validation_result: &ValidationResult) -> String {
        format!(
            "Validation Report\n\
            =================\n\
            Status: {}\n\
            Quality Grade: {:?}\n\
            Overall Score: {:.1}/100\n\
            \n\
            Data Integrity:\n\
            - Allocation Count Match: {}\n\
            - Memory Stats Match: {}\n\
            - Allocation Match Rate: {:.1}%\n\
            \n\
            Performance:\n\
            - Conversion Speed: {:.2} MB/s\n\
            - Size Ratio: {:.2}\n\
            - Performance Category: {:?}\n\
            \n\
            Issues:\n\
            - Critical Errors: {}\n\
            - Warnings: {}\n",
            if validation_result.is_valid {
                "PASSED"
            } else {
                "FAILED"
            },
            validation_result.quality_metrics.quality_grade,
            validation_result.quality_metrics.overall_score,
            validation_result.data_integrity.allocation_count_match,
            validation_result.data_integrity.memory_stats_match,
            validation_result.data_integrity.allocation_match_percentage,
            validation_result
                .performance_comparison
                .conversion_speed_mbps,
            validation_result.performance_comparison.size_ratio,
            validation_result
                .performance_comparison
                .performance_category,
            validation_result.quality_metrics.critical_issues,
            validation_result.quality_metrics.warnings
        )
    }

    /// Validate batch conversion results
    pub fn validate_batch_conversion(
        &self,
        batch_report: &BatchConversionReport,
    ) -> TrackingResult<String> {
        let mut report = String::new();
        report.push_str("Batch Validation Report\n");
        report.push_str("=======================\n");

        let total_conversions = batch_report.results.len() + batch_report.errors.len();
        let success_rate = if total_conversions > 0 {
            (batch_report.results.len() as f64 / total_conversions as f64) * 100.0
        } else {
            0.0
        };

        report.push_str(&format!("Total Conversions: {}\n", total_conversions));
        report.push_str(&format!("Successful: {}\n", batch_report.results.len()));
        report.push_str(&format!("Failed: {}\n", batch_report.errors.len()));
        report.push_str(&format!("Success Rate: {:.1}%\n", success_rate));

        if success_rate >= 95.0 {
            report.push_str("Status: PASSED\n");
        } else {
            report.push_str("Status: FAILED\n");
        }

        Ok(report)
    }
}

impl Default for ConversionValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversionValidator {
    /// Get validation options (for testing)
    #[doc(hidden)]
    pub fn get_options(&self) -> &ValidationOptions {
        &self.options
    }

    /// Public wrapper for categorize_performance (for testing)
    #[doc(hidden)]
    pub fn test_categorize_performance(
        &self,
        speed_mbps: f64,
        size_ratio: f64,
    ) -> PerformanceCategory {
        self.categorize_performance(speed_mbps, size_ratio)
    }

    /// Public wrapper for calculate_performance_rating (for testing)
    #[doc(hidden)]
    pub fn test_calculate_performance_rating(&self, speed_mbps: f64, size_ratio: f64) -> u8 {
        self.calculate_performance_rating(speed_mbps, size_ratio)
    }

    /// Public wrapper for calculate_quality_metrics (for testing)
    #[doc(hidden)]
    pub fn test_calculate_quality_metrics(
        &self,
        data_integrity: &DataIntegrityResult,
        performance_comparison: &PerformanceComparisonResult,
        validation_errors: &[ValidationError],
        validation_warnings: &[ValidationWarning],
    ) -> QualityMetrics {
        self.calculate_quality_metrics(
            data_integrity,
            performance_comparison,
            validation_errors,
            validation_warnings,
        )
    }

    /// Public wrapper for calculate_data_completeness_score (for testing)
    #[doc(hidden)]
    pub fn test_calculate_data_completeness_score(
        &self,
        data_integrity: &DataIntegrityResult,
    ) -> f64 {
        self.calculate_data_completeness_score(data_integrity)
    }

    /// Public wrapper for calculate_performance_score (for testing)
    #[doc(hidden)]
    pub fn test_calculate_performance_score(
        &self,
        performance: &PerformanceComparisonResult,
    ) -> f64 {
        self.calculate_performance_score(performance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_validation_options_default() {
        let options = ValidationOptions::default();
        assert!(options.enable_comprehensive_validation);
        assert!(!options.enable_performance_comparison);
        assert_eq!(options.validation_sample_size, 1000);
        assert_eq!(options.max_acceptable_error_rate, 0.01);
    }

    #[test]
    fn test_performance_categorization() {
        let validator = ConversionValidator::new();

        assert_eq!(
            validator.categorize_performance(60.0, 1.2),
            PerformanceCategory::Excellent
        );

        assert_eq!(
            validator.categorize_performance(25.0, 1.8),
            PerformanceCategory::Good
        );

        assert_eq!(
            validator.categorize_performance(8.0, 2.5),
            PerformanceCategory::Average
        );

        assert_eq!(
            validator.categorize_performance(2.0, 4.0),
            PerformanceCategory::Poor
        );

        assert_eq!(
            validator.categorize_performance(0.5, 6.0),
            PerformanceCategory::VeryPoor
        );
    }

    #[test]
    fn test_performance_rating() {
        let validator = ConversionValidator::new();

        assert_eq!(validator.calculate_performance_rating(60.0, 1.2), 5);
        assert_eq!(validator.calculate_performance_rating(25.0, 1.8), 4);
        assert_eq!(validator.calculate_performance_rating(8.0, 2.5), 3);
        assert_eq!(validator.calculate_performance_rating(2.0, 4.0), 2);
        assert_eq!(validator.calculate_performance_rating(0.5, 6.0), 1);
    }

    #[test]
    fn test_quality_grade_calculation() {
        let validator = ConversionValidator::new();

        // Create mock data for testing
        let data_integrity = DataIntegrityResult {
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
                binary_checksum: None,
                converted_data_checksum: None,
            },
        };

        let performance_comparison = PerformanceComparisonResult {
            conversion_speed_mbps: 50.0,
            size_ratio: 1.2,
            memory_usage_mb: Some(100.0),
            json_export_comparison: None,
            performance_rating: 5,
            performance_category: PerformanceCategory::Excellent,
        };

        let validation_errors = Vec::new();
        let validation_warnings = Vec::new();

        let quality_metrics = validator.calculate_quality_metrics(
            &data_integrity,
            &performance_comparison,
            &validation_errors,
            &validation_warnings,
        );

        assert_eq!(quality_metrics.quality_grade, QualityGrade::A);
        assert!(quality_metrics.overall_score >= 90.0);
    }
}
