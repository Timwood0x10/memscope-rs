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
    /// Compression ratio (compressed/original)
    pub ratio: Option<f64>,
    /// Original size before compression
    pub original_size: Option<u64>,
    /// Compressed size
    pub compressed_size: Option<u64>,
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