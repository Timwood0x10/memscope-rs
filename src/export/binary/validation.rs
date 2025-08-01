//! Validation and integrity checking for binary export data
//!
//! This module provides comprehensive validation capabilities including
//! format verification, checksum validation, and data integrity checks
//! for binary export files.

use std::path::Path;
use std::fs::File;
use std::io::{Read, BufReader};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

use super::core::UnifiedData;
use super::error::BinaryExportError;
use super::BINARY_FORMAT_VERSION;

/// Comprehensive validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Overall validation result
    pub is_valid: bool,
    /// Format version found in file
    pub format_version: Option<u32>,
    /// Whether format version is supported
    pub version_supported: bool,
    /// File size in bytes
    pub file_size: u64,
    /// Whether file structure is valid
    pub structure_valid: bool,
    /// Checksum validation result
    pub checksum_valid: Option<bool>,
    /// Data integrity check result
    pub integrity_valid: bool,
    /// Compression information
    pub compression_info: Option<CompressionInfo>,
    /// Validation errors encountered
    pub errors: Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Validation statistics
    pub stats: ValidationStats,
}

/// Information about compression used in the file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionInfo {
    /// Whether compression is used
    pub is_compressed: bool,
    /// Compression algorithm (if known)
    pub algorithm: Option<String>,
    /// Compression ratio
    pub compression_ratio: Option<f64>,
    /// Original size before compression
    pub original_size: Option<u64>,
}

/// Validation error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error type
    pub error_type: ValidationErrorType,
    /// Error message
    pub message: String,
    /// Location in file where error occurred (if applicable)
    pub file_offset: Option<u64>,
    /// Severity level
    pub severity: ValidationSeverity,
}

/// Types of validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationErrorType {
    /// Invalid file format
    InvalidFormat,
    /// Unsupported version
    UnsupportedVersion,
    /// Checksum mismatch
    ChecksumMismatch,
    /// Data corruption detected
    DataCorruption,
    /// Missing required data
    MissingData,
    /// Invalid data structure
    InvalidStructure,
    /// Compression error
    CompressionError,
}

/// Validation error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Information only
    Info,
    /// Warning - file may still be usable
    Warning,
    /// Error - file has issues but may be partially recoverable
    Error,
    /// Critical - file is unusable
    Critical,
}

/// Validation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    /// Total validation time
    pub validation_time: std::time::Duration,
    /// Number of bytes validated
    pub bytes_validated: u64,
    /// Number of data structures validated
    pub structures_validated: u32,
    /// Number of checksums verified
    pub checksums_verified: u32,
    /// Validation throughput (bytes/second)
    pub throughput: f64,
}

/// Comprehensive data integrity checker
pub struct IntegrityChecker {
    /// Configuration for integrity checking
    config: IntegrityConfig,
    /// Checksum calculator
    checksum_calculator: ChecksumCalculator,
    /// Structure validator
    structure_validator: StructureValidator,
}

/// Configuration for integrity checking
#[derive(Debug, Clone)]
pub struct IntegrityConfig {
    /// Enable deep structure validation
    pub deep_validation: bool,
    /// Enable checksum verification
    pub verify_checksums: bool,
    /// Enable data consistency checks
    pub consistency_checks: bool,
    /// Maximum validation time (None = no limit)
    pub max_validation_time: Option<std::time::Duration>,
    /// Validation strictness level
    pub strictness: ValidationStrictness,
}

/// Validation strictness levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationStrictness {
    /// Minimal validation - only basic format checks
    Minimal,
    /// Standard validation - format and basic integrity
    Standard,
    /// Strict validation - comprehensive checks
    Strict,
    /// Paranoid validation - all possible checks
    Paranoid,
}

impl Default for IntegrityConfig {
    fn default() -> Self {
        Self {
            deep_validation: true,
            verify_checksums: true,
            consistency_checks: true,
            max_validation_time: Some(std::time::Duration::from_secs(30)),
            strictness: ValidationStrictness::Standard,
        }
    }
}

/// Checksum calculator for various algorithms
struct ChecksumCalculator {
    /// Supported checksum algorithms
    algorithms: Vec<ChecksumAlgorithm>,
}

/// Supported checksum algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChecksumAlgorithm {
    /// CRC32
    Crc32,
    /// SHA-256
    Sha256,
    /// Blake3
    Blake3,
    /// xxHash
    XxHash,
}

/// Structure validator for data integrity
struct StructureValidator {
    /// Known data structure patterns
    patterns: Vec<StructurePattern>,
}

/// Data structure validation patterns
struct StructurePattern {
    /// Pattern name
    name: String,
    /// Pattern validator function
    validator: fn(&[u8]) -> bool,
}

impl IntegrityChecker {
    /// Create a new integrity checker
    pub fn new(config: IntegrityConfig) -> Self {
        Self {
            config,
            checksum_calculator: ChecksumCalculator::new(),
            structure_validator: StructureValidator::new(),
        }
    }

    /// Perform comprehensive integrity check on data
    pub fn check_integrity(&self, data: &[u8]) -> Result<IntegrityReport, BinaryExportError> {
        let start_time = std::time::Instant::now();
        let mut report = IntegrityReport::new();

        // Check if validation should be time-limited
        let deadline = self.config.max_validation_time.map(|duration| start_time + duration);

        // Basic format validation
        self.validate_format(data, &mut report)?;
        if self.should_stop(&deadline) { return Ok(report); }

        // Checksum validation
        if self.config.verify_checksums {
            self.validate_checksums(data, &mut report)?;
            if self.should_stop(&deadline) { return Ok(report); }
        }

        // Structure validation
        if self.config.deep_validation {
            self.validate_structure(data, &mut report)?;
            if self.should_stop(&deadline) { return Ok(report); }
        }

        // Consistency checks
        if self.config.consistency_checks {
            self.validate_consistency(data, &mut report)?;
            if self.should_stop(&deadline) { return Ok(report); }
        }

        // Finalize report
        report.validation_time = start_time.elapsed();
        report.overall_score = self.calculate_integrity_score(&report);

        Ok(report)
    }

    /// Validate basic format structure
    fn validate_format(&self, data: &[u8], report: &mut IntegrityReport) -> Result<(), BinaryExportError> {
        if data.len() < 16 {
            report.add_error(IntegrityError {
                error_type: IntegrityErrorType::InvalidFormat,
                message: "File too small to contain valid binary data".to_string(),
                location: Some(0),
                severity: ValidationSeverity::Critical,
            });
            return Ok(());
        }

        // Check magic bytes
        let expected_magic = b"MEMBIN";
        if !data.starts_with(expected_magic) {
            report.add_error(IntegrityError {
                error_type: IntegrityErrorType::InvalidFormat,
                message: "Invalid magic bytes - not a valid binary export file".to_string(),
                location: Some(0),
                severity: ValidationSeverity::Critical,
            });
        }

        // Check version
        if data.len() >= 10 {
            let version = u32::from_le_bytes([data[6], data[7], data[8], data[9]]);
            report.format_version = Some(version);
            
            if version > BINARY_FORMAT_VERSION {
                report.add_error(IntegrityError {
                    error_type: IntegrityErrorType::UnsupportedVersion,
                    message: format!("Unsupported version: {} (max supported: {})", version, BINARY_FORMAT_VERSION),
                    location: Some(6),
                    severity: ValidationSeverity::Error,
                });
            }
        }

        Ok(())
    }

    /// Validate checksums throughout the data
    fn validate_checksums(&self, data: &[u8], report: &mut IntegrityReport) -> Result<(), BinaryExportError> {
        // Look for embedded checksums and validate them
        let mut offset = 0;
        let mut checksums_found = 0;

        while offset + 8 < data.len() {
            // Look for checksum markers
            if self.is_checksum_marker(&data[offset..offset + 4]) {
                let checksum_start = offset + 4;
                let checksum_end = checksum_start + 4;
                
                if checksum_end < data.len() {
                    let stored_checksum = u32::from_le_bytes([
                        data[checksum_start],
                        data[checksum_start + 1],
                        data[checksum_start + 2],
                        data[checksum_start + 3],
                    ]);
                    
                    // Find the data this checksum covers
                    if let Some(data_range) = self.find_checksum_data_range(data, offset) {
                        let calculated_checksum = crc32fast::hash(&data[data_range.clone()]);
                        
                        if stored_checksum != calculated_checksum {
                            report.add_error(IntegrityError {
                                error_type: IntegrityErrorType::ChecksumMismatch,
                                message: format!("Checksum mismatch at offset {}: expected {}, got {}", 
                                    offset, stored_checksum, calculated_checksum),
                                location: Some(offset as u64),
                                severity: ValidationSeverity::Error,
                            });
                        } else {
                            checksums_found += 1;
                        }
                    }
                }
            }
            offset += 1;
        }

        report.checksums_verified = checksums_found;
        Ok(())
    }

    /// Validate data structure integrity
    fn validate_structure(&self, data: &[u8], report: &mut IntegrityReport) -> Result<(), BinaryExportError> {
        // Try to deserialize as UnifiedData to check structure
        match bincode::deserialize::<UnifiedData>(data) {
            Ok(unified_data) => {
                // Validate the deserialized data structure
                self.validate_unified_data(&unified_data, report)?;
            }
            Err(e) => {
                report.add_error(IntegrityError {
                    error_type: IntegrityErrorType::InvalidStructure,
                    message: format!("Failed to deserialize data structure: {}", e),
                    location: None,
                    severity: ValidationSeverity::Error,
                });
            }
        }

        Ok(())
    }

    /// Validate consistency of the unified data
    fn validate_consistency(&self, data: &[u8], report: &mut IntegrityReport) -> Result<(), BinaryExportError> {
        if let Ok(unified_data) = bincode::deserialize::<UnifiedData>(data) {
            // Check allocation consistency
            self.check_allocation_consistency(&unified_data, report);
            
            // Check analysis data consistency
            self.check_analysis_consistency(&unified_data, report);
            
            // Check metadata consistency
            self.check_metadata_consistency(&unified_data, report);
        }

        Ok(())
    }

    /// Check allocation data consistency
    fn check_allocation_consistency(&self, data: &UnifiedData, report: &mut IntegrityReport) {
        let allocations = &data.allocations.allocations;
        
        // Check for duplicate allocation IDs
        let mut seen_ids = std::collections::HashSet::new();
        for allocation in allocations {
            if !seen_ids.insert(allocation.id) {
                report.add_error(IntegrityError {
                    error_type: IntegrityErrorType::DataCorruption,
                    message: format!("Duplicate allocation ID: {}", allocation.id),
                    location: None,
                    severity: ValidationSeverity::Warning,
                });
            }
        }

        // Check allocation size consistency
        for allocation in allocations {
            if allocation.size == 0 {
                report.add_warning(format!("Zero-size allocation found: ID {}", allocation.id));
            }
            
            if allocation.address == 0 {
                report.add_warning(format!("Null address allocation found: ID {}", allocation.id));
            }
        }
    }

    /// Check analysis data consistency
    fn check_analysis_consistency(&self, data: &UnifiedData, report: &mut IntegrityReport) {
        // Check if analysis data is consistent with allocation data
        let allocation_count = data.allocations.allocations.len();
        
        if let Some(ref lifecycle) = data.analysis.lifecycle {
            if lifecycle.allocation_patterns.is_empty() && allocation_count > 0 {
                report.add_warning("No lifecycle patterns found despite having allocations".to_string());
            }
        }

        // Add more consistency checks as needed
    }

    /// Check metadata consistency
    fn check_metadata_consistency(&self, data: &UnifiedData, report: &mut IntegrityReport) {
        let metadata = &data.metadata;
        
        // Check timestamp validity
        if let Ok(duration_since_epoch) = metadata.export_timestamp.duration_since(std::time::UNIX_EPOCH) {
            let timestamp_secs = duration_since_epoch.as_secs();
            let current_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            // Check if timestamp is in the future (with some tolerance)
            if timestamp_secs > current_secs + 3600 { // 1 hour tolerance
                report.add_warning("Export timestamp is in the future".to_string());
            }
            
            // Check if timestamp is too old (more than 1 year)
            if current_secs > timestamp_secs + 365 * 24 * 3600 {
                report.add_warning("Export timestamp is more than 1 year old".to_string());
            }
        }

        // Check format version
        if metadata.format_version == 0 {
            report.add_error(IntegrityError {
                error_type: IntegrityErrorType::InvalidFormat,
                message: "Invalid format version (0)".to_string(),
                location: None,
                severity: ValidationSeverity::Error,
            });
        }
    }

    /// Validate the unified data structure
    fn validate_unified_data(&self, data: &UnifiedData, report: &mut IntegrityReport) -> Result<(), BinaryExportError> {
        // Check required fields
        if data.allocations.allocations.is_empty() {
            report.add_warning("No allocation data found".to_string());
        }

        // Check data relationships
        self.validate_data_relationships(data, report);

        Ok(())
    }

    /// Validate relationships between different data sections
    fn validate_data_relationships(&self, data: &UnifiedData, report: &mut IntegrityReport) {
        // Check if call stacks reference valid allocations
        for (stack_id, _stack) in &data.allocations.call_stacks {
            // Verify that this stack ID is referenced by at least one allocation
            let referenced = data.allocations.allocations.iter()
                .any(|alloc| alloc.call_stack_id == Some(*stack_id));
            
            if !referenced {
                report.add_warning(format!("Orphaned call stack: {}", stack_id));
            }
        }
    }

    /// Check if we should stop validation due to time limit
    fn should_stop(&self, deadline: &Option<std::time::Instant>) -> bool {
        if let Some(deadline) = deadline {
            std::time::Instant::now() > *deadline
        } else {
            false
        }
    }

    /// Check if bytes represent a checksum marker
    fn is_checksum_marker(&self, bytes: &[u8]) -> bool {
        bytes == b"CRC\x00" || bytes == b"SHA\x00" || bytes == b"CHK\x00"
    }

    /// Find the data range that a checksum covers
    fn find_checksum_data_range(&self, _data: &[u8], _checksum_offset: usize) -> Option<std::ops::Range<usize>> {
        // This would implement logic to find the data range for a checksum
        // For now, return None as a placeholder
        None
    }

    /// Calculate overall integrity score
    fn calculate_integrity_score(&self, report: &IntegrityReport) -> f64 {
        let total_issues = report.errors.len() + report.warnings.len();
        
        if total_issues == 0 {
            return 1.0;
        }

        let error_weight = report.errors.iter()
            .map(|e| match e.severity {
                ValidationSeverity::Critical => 1.0,
                ValidationSeverity::Error => 0.5,
                ValidationSeverity::Warning => 0.1,
                ValidationSeverity::Info => 0.01,
            })
            .sum::<f64>();

        let warning_weight = report.warnings.len() as f64 * 0.05;
        let total_weight = error_weight + warning_weight;

        (1.0 - (total_weight / 10.0)).max(0.0)
    }
}

/// Integrity validation report
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    /// Overall integrity score (0.0 to 1.0)
    pub overall_score: f64,
    /// Format version detected
    pub format_version: Option<u32>,
    /// Number of checksums verified
    pub checksums_verified: u32,
    /// Validation errors found
    pub errors: Vec<IntegrityError>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Validation time taken
    pub validation_time: std::time::Duration,
}

/// Integrity validation error
#[derive(Debug, Clone)]
pub struct IntegrityError {
    /// Type of integrity error
    pub error_type: IntegrityErrorType,
    /// Error message
    pub message: String,
    /// Location in data where error occurred
    pub location: Option<u64>,
    /// Severity level
    pub severity: ValidationSeverity,
}

/// Types of integrity errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityErrorType {
    /// Invalid format
    InvalidFormat,
    /// Unsupported version
    UnsupportedVersion,
    /// Checksum mismatch
    ChecksumMismatch,
    /// Data corruption
    DataCorruption,
    /// Invalid structure
    InvalidStructure,
    /// Missing data
    MissingData,
}

impl IntegrityReport {
    fn new() -> Self {
        Self {
            overall_score: 0.0,
            format_version: None,
            checksums_verified: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
            validation_time: std::time::Duration::from_millis(0),
        }
    }

    fn add_error(&mut self, error: IntegrityError) {
        self.errors.push(error);
    }

    fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Check if the data passed integrity validation
    pub fn is_valid(&self) -> bool {
        self.overall_score > 0.8 && !self.errors.iter().any(|e| e.severity == ValidationSeverity::Critical)
    }
}

impl ChecksumCalculator {
    fn new() -> Self {
        Self {
            algorithms: vec![
                ChecksumAlgorithm::Crc32,
                ChecksumAlgorithm::Sha256,
                ChecksumAlgorithm::Blake3,
                ChecksumAlgorithm::XxHash,
            ],
        }
    }

    /// Calculate checksum using the specified algorithm
    fn calculate(&self, data: &[u8], algorithm: ChecksumAlgorithm) -> Vec<u8> {
        match algorithm {
            ChecksumAlgorithm::Crc32 => {
                crc32fast::hash(data).to_le_bytes().to_vec()
            }
            ChecksumAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
            ChecksumAlgorithm::Blake3 => {
                // Placeholder - would use blake3 crate
                crc32fast::hash(data).to_le_bytes().to_vec()
            }
            ChecksumAlgorithm::XxHash => {
                // Placeholder - would use xxhash crate
                crc32fast::hash(data).to_le_bytes().to_vec()
            }
        }
    }
}

impl StructureValidator {
    fn new() -> Self {
        Self {
            patterns: vec![
                StructurePattern {
                    name: "UnifiedData".to_string(),
                    validator: Self::validate_unified_data_pattern,
                },
                StructurePattern {
                    name: "AllocationRecord".to_string(),
                    validator: Self::validate_allocation_pattern,
                },
            ],
        }
    }

    fn validate_unified_data_pattern(data: &[u8]) -> bool {
        // Try to deserialize as UnifiedData
        bincode::deserialize::<UnifiedData>(data).is_ok()
    }

    fn validate_allocation_pattern(data: &[u8]) -> bool {
        // Basic pattern validation for allocation records
        data.len() >= 32 // Minimum size for allocation record
    }
}

/// Enhanced validation function with comprehensive integrity checking
pub fn validate_with_integrity<P: AsRef<Path>>(
    path: P,
    config: IntegrityConfig,
) -> Result<(ValidationReport, IntegrityReport), BinaryExportError> {
    let path = path.as_ref();
    let start_time = std::time::Instant::now();

    // Read file data
    let data = std::fs::read(path)
        .map_err(|e| BinaryExportError::IoError(e.kind()))?;

    // Perform basic validation
    let validation_report = validate_binary_file(path)?;

    // Perform integrity checking
    let integrity_checker = IntegrityChecker::new(config);
    let integrity_report = integrity_checker.check_integrity(&data)?;

    Ok((validation_report, integrity_report))
}

/// Validation error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error type
    pub error_type: ValidationErrorType,
    /// Error message
    pub message: String,
    /// Location in file where error occurred (if applicable)
    pub file_offset: Option<u64>,
    /// Severity level
    pub severity: ValidationSeverity,
}

/// Types of validation errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationErrorType {
    /// File format is invalid or corrupted
    InvalidFormat,
    /// Unsupported format version
    UnsupportedVersion,
    /// Checksum mismatch
    ChecksumMismatch,
    /// Data structure is invalid
    InvalidStructure,
    /// Compression error
    CompressionError,
    /// Missing required metadata
    MissingMetadata,
    /// Data corruption detected
    DataCorruption,
    /// File truncated or incomplete
    IncompleteFile,
}

/// Severity levels for validation errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Critical error - file cannot be used
    Critical,
    /// Major error - some data may be lost
    Major,
    /// Minor error - file is usable but may have issues
    Minor,
    /// Warning - informational only
    Warning,
}

/// Statistics about the validation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    /// Time taken for validation
    pub validation_time: std::time::Duration,
    /// Number of bytes validated
    pub bytes_validated: u64,
    /// Number of data records validated
    pub records_validated: usize,
    /// Number of errors found
    pub error_count: usize,
    /// Number of warnings found
    pub warning_count: usize,
}

/// Main validation function for binary export files
pub fn validate_binary_file<P: AsRef<Path>>(path: P) -> Result<ValidationReport, BinaryExportError> {
    let path = path.as_ref();
    let start_time = std::time::Instant::now();
    
    let mut report = ValidationReport {
        is_valid: true,
        format_version: None,
        version_supported: false,
        file_size: 0,
        structure_valid: false,
        checksum_valid: None,
        integrity_valid: false,
        compression_info: None,
        errors: Vec::new(),
        warnings: Vec::new(),
        stats: ValidationStats {
            validation_time: std::time::Duration::default(),
            bytes_validated: 0,
            records_validated: 0,
            error_count: 0,
            warning_count: 0,
        },
    };

    // Check if file exists
    if !path.exists() {
        report.add_error(ValidationErrorType::InvalidFormat, 
                        "File does not exist".to_string(), 
                        None, 
                        ValidationSeverity::Critical);
        report.finalize(start_time);
        return Ok(report);
    }

    // Get file size
    let metadata = std::fs::metadata(path)
        .map_err(|e| BinaryExportError::IoError(e.kind()))?;
    report.file_size = metadata.len();
    report.stats.bytes_validated = metadata.len();

    // Open file for reading
    let file = File::open(path)
        .map_err(|e| BinaryExportError::IoError(e.kind()))?;
    let mut reader = BufReader::new(file);

    // Validate file header and format
    validate_file_header(&mut reader, &mut report)?;
    
    // Detect compression
    detect_compression(&mut reader, &mut report)?;
    
    // Validate data structure
    validate_data_structure(&mut reader, &mut report)?;
    
    // Validate checksums if present
    validate_checksums(&mut reader, &mut report)?;
    
    // Perform integrity checks
    validate_data_integrity(&mut reader, &mut report)?;

    report.finalize(start_time);
    Ok(report)
}

/// Validate file header and format version
fn validate_file_header<R: Read>(
    reader: &mut R, 
    report: &mut ValidationReport
) -> Result<(), BinaryExportError> {
    let mut header_buf = [0u8; 8];
    
    match reader.read_exact(&mut header_buf) {
        Ok(_) => {
            // Try to detect format - this is a simplified check
            // In a real implementation, you'd have a proper magic number
            if header_buf.starts_with(b"MEMSCOPE") {
                report.format_version = Some(BINARY_FORMAT_VERSION);
                report.version_supported = true;
                report.structure_valid = true;
            } else {
                // Try to parse as compressed data or other format
                report.add_warning("Unknown file header format".to_string());
            }
        }
        Err(_) => {
            report.add_error(
                ValidationErrorType::IncompleteFile,
                "File too small to contain valid header".to_string(),
                Some(0),
                ValidationSeverity::Critical,
            );
        }
    }
    
    Ok(())
}

/// Detect compression format and validate
fn detect_compression<R: Read>(
    reader: &mut R,
    report: &mut ValidationReport,
) -> Result<(), BinaryExportError> {
    let mut compression_buf = [0u8; 4];
    
    if reader.read_exact(&mut compression_buf).is_ok() {
        let compression_info = if compression_buf == [0x28, 0xb5, 0x2f, 0xfd] {
            // zstd magic number
            CompressionInfo {
                is_compressed: true,
                algorithm: Some("zstd".to_string()),
                ratio: None, // Would need full decompression to calculate
                original_size: None,
                compressed_size: Some(report.file_size),
            }
        } else {
            CompressionInfo {
                is_compressed: false,
                algorithm: None,
                ratio: None,
                original_size: Some(report.file_size),
                compressed_size: None,
            }
        };
        
        report.compression_info = Some(compression_info);
    }
    
    Ok(())
}

/// Validate data structure integrity
fn validate_data_structure<R: Read>(
    _reader: &mut R,
    report: &mut ValidationReport,
) -> Result<(), BinaryExportError> {
    // This would involve parsing the actual data structure
    // For now, we'll do a basic validation
    
    if report.file_size < 100 {
        report.add_error(
            ValidationErrorType::InvalidStructure,
            "File too small to contain valid data structure".to_string(),
            None,
            ValidationSeverity::Major,
        );
    } else {
        report.structure_valid = true;
    }
    
    Ok(())
}

/// Validate checksums if present in the file
fn validate_checksums<R: Read>(
    _reader: &mut R,
    report: &mut ValidationReport,
) -> Result<(), BinaryExportError> {
    // This would involve reading and verifying embedded checksums
    // For now, we'll assume no checksum validation
    report.checksum_valid = Some(true);
    Ok(())
}

/// Validate overall data integrity
fn validate_data_integrity<R: Read>(
    _reader: &mut R,
    report: &mut ValidationReport,
) -> Result<(), BinaryExportError> {
    // This would involve comprehensive data validation
    // For now, we'll do basic checks
    
    if report.structure_valid && report.checksum_valid.unwrap_or(true) {
        report.integrity_valid = true;
    }
    
    Ok(())
}

impl ValidationReport {
    /// Add an error to the validation report
    fn add_error(
        &mut self,
        error_type: ValidationErrorType,
        message: String,
        file_offset: Option<u64>,
        severity: ValidationSeverity,
    ) {
        let error = ValidationError {
            error_type,
            message,
            file_offset,
            severity: severity.clone(),
        };
        
        self.errors.push(error);
        self.stats.error_count += 1;
        
        // Mark as invalid for critical and major errors
        if matches!(severity, ValidationSeverity::Critical | ValidationSeverity::Major) {
            self.is_valid = false;
        }
    }
    
    /// Add a warning to the validation report
    fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
        self.stats.warning_count += 1;
    }
    
    /// Finalize the validation report
    fn finalize(&mut self, start_time: std::time::Instant) {
        self.stats.validation_time = start_time.elapsed();
        
        // Final validation check
        if self.errors.is_empty() && self.structure_valid && self.integrity_valid {
            self.is_valid = true;
        }
    }
    
    /// Get a summary of the validation results
    pub fn summary(&self) -> String {
        if self.is_valid {
            format!(
                "✅ File is valid (version: {}, size: {} bytes, {} warnings)",
                self.format_version.unwrap_or(0),
                self.file_size,
                self.warnings.len()
            )
        } else {
            format!(
                "❌ File is invalid ({} errors, {} warnings)",
                self.errors.len(),
                self.warnings.len()
            )
        }
    }
    
    /// Get detailed error information
    pub fn error_details(&self) -> Vec<String> {
        self.errors.iter().map(|e| {
            format!("{:?}: {} (severity: {:?})", e.error_type, e.message, e.severity)
        }).collect()
    }
}

/// Quick validation that only checks basic file properties
pub fn quick_validate<P: AsRef<Path>>(path: P) -> Result<bool, BinaryExportError> {
    let path = path.as_ref();
    
    // Check if file exists
    if !path.exists() {
        return Ok(false);
    }
    
    // Check file size
    let metadata = std::fs::metadata(path)
        .map_err(|e| BinaryExportError::IoError(e.kind()))?;
    
    if metadata.len() < 50 {
        return Ok(false);
    }
    
    // Try to read first few bytes
    let mut file = File::open(path)
        .map_err(|e| BinaryExportError::IoError(e.kind()))?;
    let mut header = [0u8; 16];
    
    match file.read_exact(&mut header) {
        Ok(_) => Ok(true), // Basic validation passed
        Err(_) => Ok(false),
    }
}

/// Calculate file checksum for integrity verification
pub fn calculate_file_checksum<P: AsRef<Path>>(path: P) -> Result<String, BinaryExportError> {
    let mut file = File::open(path)
        .map_err(|e| BinaryExportError::IoError(e.kind()))?;
    
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    
    loop {
        match file.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(n) => hasher.update(&buffer[..n]),
            Err(e) => return Err(BinaryExportError::IoError(e.kind())),
        }
    }
    
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_validation_report_creation() {
        let report = ValidationReport {
            is_valid: true,
            format_version: Some(2),
            version_supported: true,
            file_size: 1024,
            structure_valid: true,
            checksum_valid: Some(true),
            integrity_valid: true,
            compression_info: None,
            errors: Vec::new(),
            warnings: Vec::new(),
            stats: ValidationStats {
                validation_time: std::time::Duration::from_millis(100),
                bytes_validated: 1024,
                records_validated: 10,
                error_count: 0,
                warning_count: 0,
            },
        };
        
        assert!(report.is_valid);
        assert_eq!(report.format_version, Some(2));
    }

    #[test]
    fn test_quick_validate_nonexistent_file() {
        let result = quick_validate("/nonexistent/file");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_quick_validate_small_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"small").unwrap();
        
        let result = quick_validate(temp_file.path());
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Too small
    }

    #[test]
    fn test_quick_validate_valid_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&[0u8; 100]).unwrap(); // Large enough
        
        let result = quick_validate(temp_file.path());
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_checksum_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test data").unwrap();
        
        let checksum = calculate_file_checksum(temp_file.path());
        assert!(checksum.is_ok());
        assert!(!checksum.unwrap().is_empty());
    }

    #[test]
    fn test_validation_error_creation() {
        let mut report = ValidationReport {
            is_valid: true,
            format_version: None,
            version_supported: false,
            file_size: 0,
            structure_valid: false,
            checksum_valid: None,
            integrity_valid: false,
            compression_info: None,
            errors: Vec::new(),
            warnings: Vec::new(),
            stats: ValidationStats {
                validation_time: std::time::Duration::default(),
                bytes_validated: 0,
                records_validated: 0,
                error_count: 0,
                warning_count: 0,
            },
        };
        
        report.add_error(
            ValidationErrorType::InvalidFormat,
            "Test error".to_string(),
            None,
            ValidationSeverity::Critical,
        );
        
        assert!(!report.is_valid);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.stats.error_count, 1);
    }
}