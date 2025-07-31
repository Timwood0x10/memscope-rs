//! Binary file validation utilities
//!
//! This module provides comprehensive validation for binary export files including:
//! - File format validation and integrity checking
//! - Corruption detection and recovery assessment
//! - Metadata validation and consistency checks
//! - Performance and size analysis

use crate::core::types::TrackingResult;
use crate::export::formats::binary_export::BinaryExportData;
use std::path::Path;

/// Comprehensive validation result with detailed analysis
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Overall validation status
    pub is_valid: bool,
    /// File format version detected
    pub format_version: Option<String>,
    /// Compression algorithm detected
    pub compression_detected: Option<String>,
    /// File size information
    pub file_info: FileInfo,
    /// Data consistency checks
    pub consistency_checks: ConsistencyChecks,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// All validation errors found
    pub errors: Vec<ValidationError>,
    /// Non-critical warnings
    pub warnings: Vec<String>,
    /// Recovery assessment
    pub recovery_assessment: RecoveryAssessment,
}

/// File size and compression information
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// File size in bytes
    pub file_size: u64,
    /// Estimated uncompressed size
    pub uncompressed_size: Option<u64>,
    /// Compression ratio if compressed
    pub compression_ratio: Option<f64>,
    /// File creation timestamp
    pub created_at: Option<u64>,
}

/// Data consistency validation results
#[derive(Debug, Clone)]
pub struct ConsistencyChecks {
    /// Allocation count matches actual allocations
    pub allocation_count_consistent: bool,
    /// Memory totals are reasonable
    pub memory_totals_valid: bool,
    /// Timestamps are in valid ranges
    pub timestamps_valid: bool,
    /// Metadata is consistent with data
    pub metadata_consistent: bool,
    /// No duplicate allocations found
    pub no_duplicates: bool,
}

/// Performance and efficiency metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Estimated parsing time
    pub estimated_parse_time: std::time::Duration,
    /// Memory usage estimate
    pub estimated_memory_usage: u64,
    /// Compression efficiency rating
    pub compression_efficiency: CompressionEfficiency,
    /// Data density score
    pub data_density_score: f64,
}

/// Compression efficiency rating
#[derive(Debug, Clone)]
pub enum CompressionEfficiency {
    /// No compression used
    None,
    /// Poor compression (< 10% reduction)
    Poor,
    /// Fair compression (10-30% reduction)
    Fair,
    /// Good compression (30-60% reduction)
    Good,
    /// Excellent compression (> 60% reduction)
    Excellent,
}

/// Recovery assessment for corrupted files
#[derive(Debug, Clone)]
pub struct RecoveryAssessment {
    /// Whether any recovery is possible
    pub recoverable: bool,
    /// Estimated percentage of data that can be recovered
    pub recovery_percentage: f64,
    /// Specific recovery strategies available
    pub recovery_strategies: Vec<RecoveryStrategy>,
    /// Estimated effort required for recovery
    pub recovery_effort: RecoveryEffort,
}

/// Available recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Partial deserialization of valid sections
    PartialDeserialization,
    /// Metadata reconstruction from partial data
    MetadataReconstruction,
    /// Allocation data recovery from fragments
    AllocationRecovery,
    /// Statistical reconstruction based on patterns
    StatisticalReconstruction,
}

/// Effort level required for recovery
#[derive(Debug, Clone)]
pub enum RecoveryEffort {
    /// Automatic recovery possible
    Automatic,
    /// Minimal manual intervention required
    Minimal,
    /// Moderate effort required
    Moderate,
    /// Significant manual work needed
    Significant,
    /// Recovery not feasible
    NotFeasible,
}

/// Specific validation error with context
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error category
    pub category: ErrorCategory,
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Human-readable error message
    pub message: String,
    /// Location in file where error occurred (if applicable)
    pub location: Option<u64>,
    /// Suggested fix or workaround
    pub suggestion: Option<String>,
}

/// Error categories for classification
#[derive(Debug, Clone)]
pub enum ErrorCategory {
    /// File format or structure errors
    Format,
    /// Data corruption or integrity issues
    Corruption,
    /// Compression or decompression problems
    Compression,
    /// Metadata inconsistencies
    Metadata,
    /// Performance or size concerns
    Performance,
}

/// Error severity levels
#[derive(Debug, Clone)]
pub enum ErrorSeverity {
    /// Critical error preventing parsing
    Critical,
    /// Major error affecting data integrity
    Major,
    /// Minor issue that may affect performance
    Minor,
    /// Informational notice
    Info,
}

/// Advanced binary file validator
pub struct BinaryValidator {
    /// Enable deep validation checks
    deep_validation: bool,
    /// Maximum time to spend on validation
    max_validation_time: std::time::Duration,
    /// Enable performance profiling during validation
    enable_profiling: bool,
}

impl BinaryValidator {
    /// Create a new validator with default settings
    pub fn new() -> Self {
        Self {
            deep_validation: true,
            max_validation_time: std::time::Duration::from_secs(30),
            enable_profiling: false,
        }
    }

    /// Create a fast validator for quick checks
    pub fn fast() -> Self {
        Self {
            deep_validation: false,
            max_validation_time: std::time::Duration::from_secs(5),
            enable_profiling: false,
        }
    }

    /// Create a thorough validator for comprehensive analysis
    pub fn thorough() -> Self {
        Self {
            deep_validation: true,
            max_validation_time: std::time::Duration::from_secs(120),
            enable_profiling: true,
        }
    }

    /// Perform comprehensive validation of a binary file
    pub fn validate_file<P: AsRef<Path>>(&self, path: P) -> TrackingResult<ValidationReport> {
        let path_str = path.as_ref().to_string_lossy();
        let start_time = std::time::Instant::now();
        
        println!("🔍 Starting comprehensive binary file validation: {path_str}");
        println!("📋 Validation settings: deep={}, max_time={:?}, profiling={}", 
                 self.deep_validation, self.max_validation_time, self.enable_profiling);

        let mut report = ValidationReport {
            is_valid: false,
            format_version: None,
            compression_detected: None,
            file_info: FileInfo {
                file_size: 0,
                uncompressed_size: None,
                compression_ratio: None,
                created_at: None,
            },
            consistency_checks: ConsistencyChecks {
                allocation_count_consistent: false,
                memory_totals_valid: false,
                timestamps_valid: false,
                metadata_consistent: false,
                no_duplicates: false,
            },
            performance_metrics: PerformanceMetrics {
                estimated_parse_time: std::time::Duration::from_millis(0),
                estimated_memory_usage: 0,
                compression_efficiency: CompressionEfficiency::None,
                data_density_score: 0.0,
            },
            errors: Vec::new(),
            warnings: Vec::new(),
            recovery_assessment: RecoveryAssessment {
                recoverable: false,
                recovery_percentage: 0.0,
                recovery_strategies: Vec::new(),
                recovery_effort: RecoveryEffort::NotFeasible,
            },
        };

        // Step 1: Basic file validation
        if let Err(e) = self.validate_file_access(&path, &mut report) {
            report.errors.push(ValidationError {
                category: ErrorCategory::Format,
                severity: ErrorSeverity::Critical,
                message: format!("File access failed: {e}"),
                location: None,
                suggestion: Some("Check file permissions and path".to_string()),
            });
            return Ok(report);
        }

        // Step 2: Format and compression detection
        self.detect_format_and_compression(&path, &mut report)?;

        // Step 3: Data structure validation
        if self.deep_validation {
            self.validate_data_structure(&path, &mut report)?;
        }

        // Step 4: Performance analysis
        if self.enable_profiling {
            self.analyze_performance(&path, &mut report)?;
        }

        // Step 5: Recovery assessment
        self.assess_recovery_options(&mut report);

        // Final validation status
        report.is_valid = report.errors.iter().all(|e| !matches!(e.severity, ErrorSeverity::Critical));

        let validation_time = start_time.elapsed();
        println!("✅ Validation completed in {validation_time:?}");
        println!("📊 Validation summary:");
        println!("   - Status: {}", if report.is_valid { "VALID" } else { "INVALID" });
        println!("   - Errors: {} (Critical: {})", 
                 report.errors.len(),
                 report.errors.iter().filter(|e| matches!(e.severity, ErrorSeverity::Critical)).count());
        println!("   - Warnings: {}", report.warnings.len());
        println!("   - Recovery possible: {}", report.recovery_assessment.recoverable);

        Ok(report)
    }

    /// Validate basic file access and properties
    fn validate_file_access<P: AsRef<Path>>(&self, path: P, report: &mut ValidationReport) -> TrackingResult<()> {
        let metadata = std::fs::metadata(&path)
            .map_err(|e| crate::core::types::TrackingError::IoError(
                format!("Cannot access file: {e}")
            ))?;

        report.file_info.file_size = metadata.len();
        
        if let Ok(created) = metadata.created() {
            if let Ok(duration) = created.duration_since(std::time::UNIX_EPOCH) {
                report.file_info.created_at = Some(duration.as_secs());
            }
        }

        // Check for reasonable file size
        if report.file_info.file_size == 0 {
            report.errors.push(ValidationError {
                category: ErrorCategory::Format,
                severity: ErrorSeverity::Critical,
                message: "File is empty".to_string(),
                location: Some(0),
                suggestion: Some("Ensure the export completed successfully".to_string()),
            });
        } else if report.file_info.file_size > 10 * 1024 * 1024 * 1024 { // 10GB
            report.warnings.push("File is very large (>10GB), parsing may be slow".to_string());
        }

        println!("   - File size: {} bytes", report.file_info.file_size);
        Ok(())
    }

    /// Detect file format and compression
    fn detect_format_and_compression<P: AsRef<Path>>(&self, path: P, report: &mut ValidationReport) -> TrackingResult<()> {
        println!("🔍 Detecting format and compression...");
        
        // Read first few bytes for format detection
        let _sample_size = std::cmp::min(1024, report.file_info.file_size as usize);
        let sample = std::fs::read(&path)
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;

        // Check for zstd compression
        if sample.len() >= 4 {
            let magic = u32::from_le_bytes([sample[0], sample[1], sample[2], sample[3]]);
            if magic == 0x28B52FFD {
                report.compression_detected = Some("zstd".to_string());
                println!("   - Compression detected: zstd");
                
                // Try to get uncompressed size
                if let Ok(decompressed) = zstd::bulk::decompress(&sample, 100 * 1024 * 1024) {
                    report.file_info.uncompressed_size = Some(decompressed.len() as u64);
                    report.file_info.compression_ratio = Some(
                        sample.len() as f64 / decompressed.len() as f64
                    );
                    
                    // Assess compression efficiency
                    if let Some(ratio) = report.file_info.compression_ratio {
                        report.performance_metrics.compression_efficiency = match ratio {
                            r if r > 0.9 => CompressionEfficiency::Poor,
                            r if r > 0.7 => CompressionEfficiency::Fair,
                            r if r > 0.4 => CompressionEfficiency::Good,
                            _ => CompressionEfficiency::Excellent,
                        };
                    }
                }
            }
        }

        // Try to detect format version
        let data_to_check = if report.compression_detected.is_some() {
            match zstd::bulk::decompress(&sample, 10 * 1024 * 1024) {
                Ok(decompressed) => decompressed,
                Err(_) => sample,
            }
        } else {
            sample
        };

        // Try to parse as BinaryExportData to get version
        match rmp_serde::from_slice::<BinaryExportData>(&data_to_check) {
            Ok(data) => {
                report.format_version = Some(data.version.clone());
                println!("   - Format version: {}", data.version);
            }
            Err(e) => {
                report.errors.push(ValidationError {
                    category: ErrorCategory::Format,
                    severity: ErrorSeverity::Major,
                    message: format!("Cannot parse MessagePack format: {e}"),
                    location: None,
                    suggestion: Some("File may be corrupted or in wrong format".to_string()),
                });
            }
        }

        Ok(())
    }

    /// Validate data structure consistency
    fn validate_data_structure<P: AsRef<Path>>(&self, path: P, report: &mut ValidationReport) -> TrackingResult<()> {
        println!("🔍 Performing deep data structure validation...");
        
        // Try to fully parse the file
        let data = std::fs::read(&path)
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;

        let parsed_data = if report.compression_detected.is_some() {
            let decompressed = zstd::bulk::decompress(&data, 1024 * 1024 * 1024)
                .map_err(|e| crate::core::types::TrackingError::SerializationError(
                    format!("Decompression failed: {e}")
                ))?;
            rmp_serde::from_slice::<BinaryExportData>(&decompressed)
        } else {
            rmp_serde::from_slice::<BinaryExportData>(&data)
        };

        match parsed_data {
            Ok(export_data) => {
                self.validate_export_data(&export_data, report);
            }
            Err(e) => {
                report.errors.push(ValidationError {
                    category: ErrorCategory::Corruption,
                    severity: ErrorSeverity::Critical,
                    message: format!("Data structure validation failed: {e}"),
                    location: None,
                    suggestion: Some("File appears to be corrupted".to_string()),
                });
                
                // Assess recovery potential
                report.recovery_assessment.recoverable = true;
                report.recovery_assessment.recovery_percentage = 25.0;
                report.recovery_assessment.recovery_strategies.push(RecoveryStrategy::PartialDeserialization);
                report.recovery_assessment.recovery_effort = RecoveryEffort::Moderate;
            }
        }

        Ok(())
    }

    /// Validate parsed export data for consistency
    fn validate_export_data(&self, data: &BinaryExportData, report: &mut ValidationReport) {
        // Check allocation count consistency
        report.consistency_checks.allocation_count_consistent = 
            data.allocations.len() == data.allocation_count;
        
        if !report.consistency_checks.allocation_count_consistent {
            report.errors.push(ValidationError {
                category: ErrorCategory::Corruption,
                severity: ErrorSeverity::Major,
                message: format!(
                    "Allocation count mismatch: expected {}, found {}",
                    data.allocation_count, data.allocations.len()
                ),
                location: None,
                suggestion: Some("Data may be partially corrupted".to_string()),
            });
        }

        // Check memory totals
        let calculated_total: usize = data.allocations.iter().map(|a| a.size).sum();
        report.consistency_checks.memory_totals_valid = 
            calculated_total as u64 <= data.total_memory * 2; // Allow some variance
        
        if !report.consistency_checks.memory_totals_valid {
            report.warnings.push(format!(
                "Memory total inconsistency: calculated {} vs reported {}",
                calculated_total, data.total_memory
            ));
        }

        // Check for duplicates
        let mut seen_ptrs = std::collections::HashSet::new();
        let mut duplicates = 0;
        for allocation in &data.allocations {
            if !seen_ptrs.insert(allocation.ptr) {
                duplicates += 1;
            }
        }
        
        report.consistency_checks.no_duplicates = duplicates == 0;
        if duplicates > 0 {
            report.warnings.push(format!("Found {} duplicate allocations", duplicates));
        }

        // Validate metadata consistency
        if let Some(metadata) = &data.metadata {
            report.consistency_checks.metadata_consistent = 
                !metadata.export_format_version.is_empty();
            
            if !report.consistency_checks.metadata_consistent {
                report.warnings.push("Metadata appears incomplete".to_string());
            }
        }

        println!("   - Allocation count consistent: {}", report.consistency_checks.allocation_count_consistent);
        println!("   - Memory totals valid: {}", report.consistency_checks.memory_totals_valid);
        println!("   - No duplicates: {}", report.consistency_checks.no_duplicates);
    }

    /// Analyze performance characteristics
    fn analyze_performance<P: AsRef<Path>>(&self, _path: P, report: &mut ValidationReport) -> TrackingResult<()> {
        println!("📊 Analyzing performance characteristics...");
        
        // Estimate parsing time based on file size
        let base_time_ms = (report.file_info.file_size / (1024 * 1024)) * 10; // ~10ms per MB
        report.performance_metrics.estimated_parse_time = 
            std::time::Duration::from_millis(base_time_ms);

        // Estimate memory usage (roughly 2-3x file size for decompression + parsing)
        report.performance_metrics.estimated_memory_usage = 
            report.file_info.file_size * 3;

        // Calculate data density score
        if let Some(uncompressed_size) = report.file_info.uncompressed_size {
            report.performance_metrics.data_density_score = 
                report.file_info.file_size as f64 / uncompressed_size as f64;
        }

        println!("   - Estimated parse time: {:?}", report.performance_metrics.estimated_parse_time);
        println!("   - Estimated memory usage: {} MB", 
                 report.performance_metrics.estimated_memory_usage / (1024 * 1024));

        Ok(())
    }

    /// Assess recovery options for corrupted files
    fn assess_recovery_options(&self, report: &mut ValidationReport) {
        let critical_errors = report.errors.iter()
            .filter(|e| matches!(e.severity, ErrorSeverity::Critical))
            .count();

        if critical_errors == 0 {
            // File is valid, no recovery needed
            report.recovery_assessment.recoverable = true;
            report.recovery_assessment.recovery_percentage = 100.0;
            report.recovery_assessment.recovery_effort = RecoveryEffort::Automatic;
        } else if critical_errors <= 2 {
            // Minor corruption, good recovery prospects
            report.recovery_assessment.recoverable = true;
            report.recovery_assessment.recovery_percentage = 75.0;
            report.recovery_assessment.recovery_strategies.extend([
                RecoveryStrategy::PartialDeserialization,
                RecoveryStrategy::MetadataReconstruction,
            ]);
            report.recovery_assessment.recovery_effort = RecoveryEffort::Minimal;
        } else if critical_errors <= 5 {
            // Moderate corruption
            report.recovery_assessment.recoverable = true;
            report.recovery_assessment.recovery_percentage = 50.0;
            report.recovery_assessment.recovery_strategies.extend([
                RecoveryStrategy::AllocationRecovery,
                RecoveryStrategy::StatisticalReconstruction,
            ]);
            report.recovery_assessment.recovery_effort = RecoveryEffort::Moderate;
        } else {
            // Severe corruption
            report.recovery_assessment.recoverable = false;
            report.recovery_assessment.recovery_percentage = 10.0;
            report.recovery_assessment.recovery_effort = RecoveryEffort::NotFeasible;
        }
    }
}

impl Default for BinaryValidator {
    fn default() -> Self {
        Self::new()
    }
}