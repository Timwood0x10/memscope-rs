//! Conversion validation module for binary export system
//!
//! This module provides comprehensive validation functionality for binary-to-JSON/HTML conversions,
//! including data integrity checks, performance comparisons, and quality reporting.

use crate::core::types::{AllocationInfo, MemoryStats, TrackingResult, TypeMemoryUsage};
use crate::export::binary_converter::{ConversionResult, BatchConversionReport};
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
    A, // 90-100: Excellent
    B, // 80-89: Good
    C, // 70-79: Average
    D, // 60-69: Below Average
    F, // <60: Poor
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
            validation_sample_size: 1000, // Validate 1000 allocations by default
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
            || validation_errors.iter().all(|e| e.severity >= ValidationSeverity::Minor);

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
                if validation_result.is_valid { "PASSED" } else { "FAILED" },
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
        let memory_stats_match = self.validate_memory_stats(&parsed, original_stats, validation_errors);

        // Validate sample of allocations
        let sample_size = if self.options.validation_sample_size == 0 {
            original_allocations.len()
        } else {
            self.options.validation_sample_size.min(original_allocations.len())
        };

        let (sample_validation_passed, allocation_match_percentage, mismatched_allocations) = 
            self.validate_allocation_sample(
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
            memory_stats_match: true, // Cannot validate from HTML
            type_memory_usage_match: true, // Cannot validate from HTML
            allocation_match_percentage: if contains_allocation_count { 100.0 } else { 0.0 },
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
                if peak_memory.as_u64() != Some(original_stats.lifecycle_stats.peak_memory_usage as u64) {
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
                            i,
                            original_type_name,
                            json_type_name
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

        let sample_validation_passed = match_percentage >= (100.0 - self.options.max_acceptable_error_rate * 100.0);

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

        Ok((sample_validation_passed, match_percentage, mismatched_allocations))
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
        if conversion_speed_mbps < self.options.performance_thresholds.min_conversion_speed_mbps {
            validation_warnings.push(ValidationWarning {
                warning_type: ValidationWarningType::SlowConversionSpeed,
                message: format!(
                    "Conversion speed {:.2} MB/s is below threshold {:.2} MB/s",
                    conversion_speed_mbps,
                    self.options.performance_thresholds.min_conversion_speed_mbps
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
                    size_ratio,
                    self.options.performance_thresholds.max_size_ratio
                ),
                context: Some("size_ratio".to_string()),
                recommendation: Some("Consider using compression or optimizing output format".to_string()),
            });
        }

        // Determine performance category
        let performance_category = self.categorize_performance(conversion_speed_mbps, size_ratio);
        let performance_rating = self.calculate_performance_rating(&performance_category);

        // Optional: Compare with direct JSON export
        let json_export_comparison = if self.options.enable_performance_comparison {
            Some(self.perform_json_export_comparison(binary_path, conversion_result)?)
        } else {
            None
        };

        Ok(PerformanceComparisonResult {
            conversion_speed_mbps,
            size_ratio,
            memory_usage_mb: None, // TODO: Implement memory tracking
            json_export_comparison,
            performance_rating,
            performance_category,
        })
    }

    /// Perform comparison with direct JSON export
    fn perform_json_export_comparison(
        &self,
        binary_path: &Path,
        conversion_result: &ConversionResult,
    ) -> TrackingResult<JsonExportComparison> {
        // Load data from binary file
        let mut parser = BinaryParser::default();
        parser.load_from_file(binary_path)?;
        let allocations = parser.load_allocations()?;
        let stats = parser.load_memory_stats()?;
        let type_usage = parser.load_type_memory_usage()?;

        // Create temporary JSON file for direct export
        let temp_json_path = std::env::temp_dir().join("temp_direct_export.json");
        
        // Perform direct JSON export
        let direct_export_start = Instant::now();
        let json_options = OptimizedExportOptions::default();
        
        // Create JSON data directly (simulating direct export)
        let json_data = self.create_direct_json_export(&allocations, &stats, &type_usage, &json_options)?;
        std::fs::write(&temp_json_path, &json_data).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to write temporary JSON file: {e}"
            ))
        })?;
        
        let direct_json_export_time = direct_export_start.elapsed();
        let direct_json_size = json_data.len();

        // Clean up temporary file
        let _ = std::fs::remove_file(&temp_json_path);

        // Calculate comparison metrics
        let speed_improvement_factor = if direct_json_export_time.as_secs_f64() > 0.0 {
            direct_json_export_time.as_secs_f64() / conversion_result.conversion_duration.as_secs_f64()
        } else {
            1.0
        };

        let size_difference_ratio = conversion_result.output_size as f64 / direct_json_size as f64;

        Ok(JsonExportComparison {
            speed_improvement_factor,
            size_difference_ratio,
            direct_json_export_time,
            binary_conversion_time: conversion_result.conversion_duration,
            direct_json_size,
            binary_conversion_size: conversion_result.output_size,
        })
    }

    /// Create direct JSON export (simplified)
    fn create_direct_json_export(
        &self,
        allocations: &[AllocationInfo],
        stats: &MemoryStats,
        type_usage: &[TypeMemoryUsage],
        _options: &OptimizedExportOptions,
    ) -> TrackingResult<String> {
        use serde_json::json;

        let json_data = json!({
            "allocations": allocations,
            "stats": stats,
            "memoryByType": type_usage.iter().map(|usage| {
                (usage.type_name.clone(), (usage.total_size, usage.allocation_count))
            }).collect::<HashMap<_, _>>(),
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });

        serde_json::to_string_pretty(&json_data).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to serialize direct JSON export: {e}"
            ))
        })
    }

    /// Categorize performance based on speed and size ratio
    fn categorize_performance(&self, speed_mbps: f64, size_ratio: f64) -> PerformanceCategory {
        match (speed_mbps, size_ratio) {
            (s, r) if s > 50.0 && r < 1.5 => PerformanceCategory::Excellent,
            (s, r) if s > 20.0 && r < 2.0 => PerformanceCategory::Good,
            (s, r) if s > 5.0 && r < 3.0 => PerformanceCategory::Average,
            (s, r) if s > 1.0 && r < 5.0 => PerformanceCategory::Poor,
            _ => PerformanceCategory::VeryPoor,
        }
    }

    /// Calculate performance rating (1-5)
    fn calculate_performance_rating(&self, category: &PerformanceCategory) -> u8 {
        match category {
            PerformanceCategory::Excellent => 5,
            PerformanceCategory::Good => 4,
            PerformanceCategory::Average => 3,
            PerformanceCategory::Poor => 2,
            PerformanceCategory::VeryPoor => 1,
        }
    }

    /// Calculate overall quality metrics
    fn calculate_quality_metrics(
        &self,
        data_integrity: &DataIntegrityResult,
        performance: &PerformanceComparisonResult,
        validation_errors: &[ValidationError],
        validation_warnings: &[ValidationWarning],
    ) -> QualityMetrics {
        // Calculate individual scores
        let data_completeness_score = if data_integrity.allocation_count_match 
            && data_integrity.memory_stats_match 
            && data_integrity.type_memory_usage_match {
            100.0
        } else {
            50.0
        };

        let data_accuracy_score = data_integrity.allocation_match_percentage;

        let performance_score = match performance.performance_category {
            PerformanceCategory::Excellent => 100.0,
            PerformanceCategory::Good => 80.0,
            PerformanceCategory::Average => 60.0,
            PerformanceCategory::Poor => 40.0,
            PerformanceCategory::VeryPoor => 20.0,
        };

        // Calculate reliability score based on errors
        let critical_issues = validation_errors.iter()
            .filter(|e| e.severity == ValidationSeverity::Critical)
            .count();
        let major_issues = validation_errors.iter()
            .filter(|e| e.severity == ValidationSeverity::Major)
            .count();

        let reliability_score = if critical_issues > 0 {
            0.0
        } else if major_issues > 0 {
            50.0 - (major_issues as f64 * 10.0).min(50.0)
        } else {
            100.0 - (validation_errors.len() as f64 * 5.0).min(50.0)
        };

        // Calculate overall score
        let overall_score = (data_completeness_score * 0.3 
            + data_accuracy_score * 0.3 
            + performance_score * 0.2 
            + reliability_score * 0.2).min(100.0);

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
            warnings: validation_warnings.len(),
            quality_grade,
        }
    }

    /// Generate a comprehensive quality report
    pub fn generate_quality_report(&self, validation_result: &ValidationResult) -> String {
        let mut report = String::new();
        
        report.push_str("=".repeat(80).as_str());
        report.push_str("\n                    CONVERSION VALIDATION REPORT\n");
        report.push_str("=".repeat(80).as_str());
        report.push('\n');

        // Overall status
        report.push_str(&format!(
            "\nOVERALL STATUS: {}\n",
            if validation_result.is_valid { "✅ PASSED" } else { "❌ FAILED" }
        ));
        
        report.push_str(&format!(
            "Quality Score: {:.1}/100 (Grade: {:?})\n",
            validation_result.quality_metrics.overall_score,
            validation_result.quality_metrics.quality_grade
        ));

        report.push_str(&format!(
            "Validation Duration: {:.2}s\n",
            validation_result.validation_duration.as_secs_f64()
        ));

        // Data Integrity Section
        report.push_str("\n");
        report.push_str("-".repeat(40).as_str());
        report.push_str("\n           DATA INTEGRITY\n");
        report.push_str("-".repeat(40).as_str());
        report.push('\n');

        let integrity = &validation_result.data_integrity;
        report.push_str(&format!(
            "Allocation Count Match: {}\n",
            if integrity.allocation_count_match { "✅" } else { "❌" }
        ));
        report.push_str(&format!(
            "Memory Stats Match: {}\n",
            if integrity.memory_stats_match { "✅" } else { "❌" }
        ));
        report.push_str(&format!(
            "Allocation Match Rate: {:.1}%\n",
            integrity.allocation_match_percentage
        ));
        report.push_str(&format!(
            "Sample Size: {} allocations\n",
            integrity.sample_size
        ));
        if integrity.mismatched_allocations > 0 {
            report.push_str(&format!(
                "Mismatched Allocations: {}\n",
                integrity.mismatched_allocations
            ));
        }

        // Performance Section
        report.push_str("\n");
        report.push_str("-".repeat(40).as_str());
        report.push_str("\n           PERFORMANCE\n");
        report.push_str("-".repeat(40).as_str());
        report.push('\n');

        let perf = &validation_result.performance_comparison;
        report.push_str(&format!(
            "Conversion Speed: {:.2} MB/s\n",
            perf.conversion_speed_mbps
        ));
        report.push_str(&format!(
            "Size Ratio: {:.2}x\n",
            perf.size_ratio
        ));
        report.push_str(&format!(
            "Performance Category: {:?}\n",
            perf.performance_category
        ));
        report.push_str(&format!(
            "Performance Rating: {}/5\n",
            perf.performance_rating
        ));

        if let Some(ref comparison) = perf.json_export_comparison {
            report.push_str(&format!(
                "Speed Improvement vs Direct JSON: {:.1}x\n",
                comparison.speed_improvement_factor
            ));
            report.push_str(&format!(
                "Size vs Direct JSON: {:.2}x\n",
                comparison.size_difference_ratio
            ));
        }

        // Quality Metrics Section
        report.push_str("\n");
        report.push_str("-".repeat(40).as_str());
        report.push_str("\n         QUALITY METRICS\n");
        report.push_str("-".repeat(40).as_str());
        report.push('\n');

        let quality = &validation_result.quality_metrics;
        report.push_str(&format!(
            "Data Completeness: {:.1}/100\n",
            quality.data_completeness_score
        ));
        report.push_str(&format!(
            "Data Accuracy: {:.1}/100\n",
            quality.data_accuracy_score
        ));
        report.push_str(&format!(
            "Performance Score: {:.1}/100\n",
            quality.performance_score
        ));
        report.push_str(&format!(
            "Reliability Score: {:.1}/100\n",
            quality.reliability_score
        ));

        // Issues Section
        if !validation_result.validation_errors.is_empty() || !validation_result.validation_warnings.is_empty() {
            report.push_str("\n");
            report.push_str("-".repeat(40).as_str());
            report.push_str("\n            ISSUES\n");
            report.push_str("-".repeat(40).as_str());
            report.push('\n');

            if !validation_result.validation_errors.is_empty() {
                report.push_str(&format!("\nERRORS ({}):\n", validation_result.validation_errors.len()));
                for (i, error) in validation_result.validation_errors.iter().enumerate() {
                    report.push_str(&format!(
                        "  {}. [{:?}] {}\n",
                        i + 1,
                        error.severity,
                        error.message
                    ));
                    if let Some(ref context) = error.context {
                        report.push_str(&format!("     Context: {}\n", context));
                    }
                    if let Some(ref fix) = error.suggested_fix {
                        report.push_str(&format!("     Suggested Fix: {}\n", fix));
                    }
                }
            }

            if !validation_result.validation_warnings.is_empty() {
                report.push_str(&format!("\nWARNINGS ({}):\n", validation_result.validation_warnings.len()));
                for (i, warning) in validation_result.validation_warnings.iter().enumerate() {
                    report.push_str(&format!(
                        "  {}. {}\n",
                        i + 1,
                        warning.message
                    ));
                    if let Some(ref recommendation) = warning.recommendation {
                        report.push_str(&format!("     Recommendation: {}\n", recommendation));
                    }
                }
            }
        }

        report.push_str("\n");
        report.push_str("=".repeat(80).as_str());
        report.push_str("\n                    END OF REPORT\n");
        report.push_str("=".repeat(80).as_str());
        report.push('\n');

        report
    }

    /// Validate batch conversion results
    pub fn validate_batch_conversion(
        &self,
        batch_report: &BatchConversionReport,
    ) -> TrackingResult<BatchValidationReport> {
        let start_time = Instant::now();

        // Calculate batch-level metrics
        let overall_success_rate = batch_report.success_rate();
        let overall_speed = batch_report.overall_speed_mbps();
        let overall_size_ratio = batch_report.overall_size_ratio();

        // Categorize batch performance
        let batch_performance_category = self.categorize_performance(overall_speed, overall_size_ratio);

        // Calculate quality score for the batch
        let batch_quality_score = self.calculate_batch_quality_score(batch_report);

        let validation_duration = start_time.elapsed();

        Ok(BatchValidationReport {
            total_files: batch_report.successful_conversions + batch_report.failed_conversions,
            successful_validations: batch_report.successful_conversions,
            failed_validations: batch_report.failed_conversions,
            overall_success_rate,
            overall_speed_mbps: overall_speed,
            overall_size_ratio,
            batch_performance_category,
            batch_quality_score,
            validation_duration,
            validation_timestamp: std::time::SystemTime::now(),
        })
    }

    /// Calculate batch quality score
    fn calculate_batch_quality_score(&self, batch_report: &BatchConversionReport) -> f64 {
        let success_rate = batch_report.success_rate();
        let speed_score = match batch_report.overall_speed_mbps() {
            s if s > 50.0 => 100.0,
            s if s > 20.0 => 80.0,
            s if s > 5.0 => 60.0,
            s if s > 1.0 => 40.0,
            _ => 20.0,
        };

        let size_score = match batch_report.overall_size_ratio() {
            r if r < 1.5 => 100.0,
            r if r < 2.0 => 80.0,
            r if r < 3.0 => 60.0,
            r if r < 5.0 => 40.0,
            _ => 20.0,
        };

        // Weighted average
        (success_rate * 100.0 * 0.5 + speed_score * 0.3 + size_score * 0.2).min(100.0)
    }
}

impl Default for ConversionValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchValidationReport {
    /// Total number of files processed
    pub total_files: usize,
    /// Number of successful validations
    pub successful_validations: usize,
    /// Number of failed validations
    pub failed_validations: usize,
    /// Overall success rate (0.0-1.0)
    pub overall_success_rate: f64,
    /// Overall conversion speed (MB/s)
    pub overall_speed_mbps: f64,
    /// Overall size ratio
    pub overall_size_ratio: f64,
    /// Batch performance category
    pub batch_performance_category: PerformanceCategory,
    /// Batch quality score (0-100)
    pub batch_quality_score: f64,
    /// Time taken for validation
    pub validation_duration: Duration,
    /// Validation timestamp
    pub validation_timestamp: std::time::SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;

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
            validator.categorize_performance(30.0, 1.8),
            PerformanceCategory::Good
        );
        assert_eq!(
            validator.categorize_performance(10.0, 2.5),
            PerformanceCategory::Average
        );
        assert_eq!(
            validator.categorize_performance(3.0, 4.0),
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
        
        assert_eq!(validator.calculate_performance_rating(&PerformanceCategory::Excellent), 5);
        assert_eq!(validator.calculate_performance_rating(&PerformanceCategory::Good), 4);
        assert_eq!(validator.calculate_performance_rating(&PerformanceCategory::Average), 3);
        assert_eq!(validator.calculate_performance_rating(&PerformanceCategory::Poor), 2);
        assert_eq!(validator.calculate_performance_rating(&PerformanceCategory::VeryPoor), 1);
    }

    #[test]
    fn test_quality_grade_calculation() {
        let validator = ConversionValidator::new();
        
        // Test data integrity with perfect scores
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

        let performance = PerformanceComparisonResult {
            conversion_speed_mbps: 60.0,
            size_ratio: 1.2,
            memory_usage_mb: None,
            json_export_comparison: None,
            performance_rating: 5,
            performance_category: PerformanceCategory::Excellent,
        };

        let quality = validator.calculate_quality_metrics(
            &data_integrity,
            &performance,
            &[],
            &[]
        );

        assert_eq!(quality.quality_grade, QualityGrade::A);
        assert!(quality.overall_score >= 90.0);
    }
}