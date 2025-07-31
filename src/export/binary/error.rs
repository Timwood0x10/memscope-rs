//! Error handling and recovery for binary export operations
//!
//! This module provides comprehensive error handling with intelligent recovery
//! strategies for various failure scenarios during binary export operations.

use std::fmt;
use std::path::PathBuf;

/// Comprehensive error types for binary export operations
#[derive(Debug, Clone)]
pub enum BinaryExportError {
    /// File system related errors
    IoError(std::io::ErrorKind),
    /// File not found at specified path
    FileNotFound(PathBuf),
    /// Insufficient permissions to read/write file
    PermissionDenied(PathBuf),
    /// Disk space insufficient for export
    InsufficientDiskSpace { required: u64, available: u64 },
    
    /// Data serialization/deserialization errors
    SerializationError(String),
    /// Data compression/decompression errors
    CompressionError(String),
    /// Invalid binary format or corrupted data
    InvalidFormat(String),
    /// Unsupported format version
    UnsupportedVersion { found: u32, supported: u32 },
    
    /// Memory related errors
    OutOfMemory { requested: usize, available: usize },
    /// Memory allocation failed
    AllocationFailed(String),
    /// Memory limit exceeded during operation
    MemoryLimitExceeded { limit: usize, usage: usize },
    
    /// Operation timeout
    Timeout { operation: String, timeout_secs: u64 },
    /// Operation was cancelled by user
    Cancelled,
    /// No data available to export
    NoDataToExport,
    
    /// Data validation errors
    ValidationFailed(String),
    /// Checksum mismatch
    ChecksumMismatch { expected: String, actual: String },
    /// Data integrity check failed
    IntegrityCheckFailed(String),
    
    /// Configuration errors
    InvalidConfiguration(String),
    /// Feature not supported in current configuration
    UnsupportedFeature(String),
    
    /// Internal errors that shouldn't normally occur
    InternalError(String),
}

impl fmt::Display for BinaryExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryExportError::IoError(kind) => {
                write!(f, "I/O error: {:?}", kind)
            }
            BinaryExportError::FileNotFound(path) => {
                write!(f, "File not found: {}", path.display())
            }
            BinaryExportError::PermissionDenied(path) => {
                write!(f, "Permission denied: {}", path.display())
            }
            BinaryExportError::InsufficientDiskSpace { required, available } => {
                write!(f, "Insufficient disk space: need {} bytes, have {} bytes", required, available)
            }
            BinaryExportError::SerializationError(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
            BinaryExportError::CompressionError(msg) => {
                write!(f, "Compression error: {}", msg)
            }
            BinaryExportError::InvalidFormat(msg) => {
                write!(f, "Invalid format: {}", msg)
            }
            BinaryExportError::UnsupportedVersion { found, supported } => {
                write!(f, "Unsupported version: found {}, supported {}", found, supported)
            }
            BinaryExportError::OutOfMemory { requested, available } => {
                write!(f, "Out of memory: requested {} bytes, available {} bytes", requested, available)
            }
            BinaryExportError::AllocationFailed(msg) => {
                write!(f, "Memory allocation failed: {}", msg)
            }
            BinaryExportError::MemoryLimitExceeded { limit, usage } => {
                write!(f, "Memory limit exceeded: limit {} bytes, usage {} bytes", limit, usage)
            }
            BinaryExportError::Timeout { operation, timeout_secs } => {
                write!(f, "Operation '{}' timed out after {} seconds", operation, timeout_secs)
            }
            BinaryExportError::Cancelled => {
                write!(f, "Operation was cancelled")
            }
            BinaryExportError::NoDataToExport => {
                write!(f, "No data available to export")
            }
            BinaryExportError::ValidationFailed(msg) => {
                write!(f, "Validation failed: {}", msg)
            }
            BinaryExportError::ChecksumMismatch { expected, actual } => {
                write!(f, "Checksum mismatch: expected {}, got {}", expected, actual)
            }
            BinaryExportError::IntegrityCheckFailed(msg) => {
                write!(f, "Integrity check failed: {}", msg)
            }
            BinaryExportError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
            BinaryExportError::UnsupportedFeature(feature) => {
                write!(f, "Unsupported feature: {}", feature)
            }
            BinaryExportError::InternalError(msg) => {
                write!(f, "Internal error: {}", msg)
            }
        }
    }
}

impl std::error::Error for BinaryExportError {}

impl From<std::io::Error> for BinaryExportError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => {
                // Try to extract path from error if available
                BinaryExportError::FileNotFound(PathBuf::from("unknown"))
            }
            std::io::ErrorKind::PermissionDenied => {
                BinaryExportError::PermissionDenied(PathBuf::from("unknown"))
            }
            kind => BinaryExportError::IoError(kind),
        }
    }
}

/// Recovery strategies for different types of errors
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// Retry the operation with the same parameters
    Retry { max_attempts: u32, delay_ms: u64 },
    /// Retry with reduced memory usage
    RetryWithLessMemory { memory_reduction_factor: f64 },
    /// Retry with different compression settings
    RetryWithDifferentCompression { fallback_level: i32 },
    /// Fallback to uncompressed export
    FallbackToUncompressed,
    /// Use streaming mode for large datasets
    FallbackToStreaming,
    /// Skip problematic data and continue
    SkipAndContinue,
    /// Abort the operation
    Abort,
}

/// Error recovery system that provides intelligent fallback strategies
pub struct ErrorRecovery {
    /// Maximum number of retry attempts for any operation
    max_retries: u32,
    /// Whether to enable automatic fallback strategies
    enable_fallback: bool,
    /// Recovery strategies for different error types
    strategies: std::collections::HashMap<String, RecoveryStrategy>,
}

impl ErrorRecovery {
    /// Create a new error recovery system with default strategies
    pub fn new() -> Self {
        let mut strategies = std::collections::HashMap::new();
        
        // Default recovery strategies
        strategies.insert(
            "memory".to_string(),
            RecoveryStrategy::RetryWithLessMemory { memory_reduction_factor: 0.5 }
        );
        strategies.insert(
            "compression".to_string(),
            RecoveryStrategy::RetryWithDifferentCompression { fallback_level: 1 }
        );
        strategies.insert(
            "io".to_string(),
            RecoveryStrategy::Retry { max_attempts: 3, delay_ms: 1000 }
        );
        
        Self {
            max_retries: 3,
            enable_fallback: true,
            strategies,
        }
    }

    /// Get recovery strategy for a specific error
    pub fn get_strategy(&self, error: &BinaryExportError) -> Option<RecoveryStrategy> {
        if !self.enable_fallback {
            return None;
        }

        match error {
            BinaryExportError::OutOfMemory { .. } |
            BinaryExportError::MemoryLimitExceeded { .. } |
            BinaryExportError::AllocationFailed(_) => {
                self.strategies.get("memory").cloned()
            }
            BinaryExportError::CompressionError(_) => {
                self.strategies.get("compression").cloned()
            }
            BinaryExportError::IoError(_) |
            BinaryExportError::InsufficientDiskSpace { .. } => {
                self.strategies.get("io").cloned()
            }
            BinaryExportError::Timeout { .. } => {
                Some(RecoveryStrategy::Retry { max_attempts: 1, delay_ms: 0 })
            }
            BinaryExportError::InvalidFormat(_) |
            BinaryExportError::UnsupportedVersion { .. } => {
                Some(RecoveryStrategy::Abort)
            }
            _ => None,
        }
    }

    /// Execute an operation with automatic error recovery
    pub fn execute_with_recovery<T, F>(
        &self,
        operation: F,
        operation_name: &str,
    ) -> Result<T, BinaryExportError>
    where
        F: Fn() -> Result<T, BinaryExportError>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.max_retries {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    attempts += 1;
                    last_error = Some(error.clone());

                    if let Some(strategy) = self.get_strategy(&error) {
                        match strategy {
                            RecoveryStrategy::Retry { max_attempts, delay_ms } => {
                                if attempts < max_attempts {
                                    if delay_ms > 0 {
                                        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                                    }
                                    continue;
                                }
                            }
                            RecoveryStrategy::Abort => {
                                return Err(error);
                            }
                            _ => {
                                // Other strategies require external handling
                                return Err(error);
                            }
                        }
                    }

                    // If no strategy or strategy failed, continue to next attempt
                    if attempts >= self.max_retries {
                        break;
                    }
                }
            }
        }

        // All attempts failed
        Err(last_error.unwrap_or_else(|| {
            BinaryExportError::InternalError(
                format!("Operation '{}' failed after {} attempts", operation_name, attempts)
            )
        }))
    }

    /// Check if an error is recoverable
    pub fn is_recoverable(&self, error: &BinaryExportError) -> bool {
        matches!(error,
            BinaryExportError::IoError(_) |
            BinaryExportError::OutOfMemory { .. } |
            BinaryExportError::MemoryLimitExceeded { .. } |
            BinaryExportError::CompressionError(_) |
            BinaryExportError::Timeout { .. } |
            BinaryExportError::InsufficientDiskSpace { .. }
        )
    }

    /// Get human-readable error message with recovery suggestions
    pub fn get_error_message_with_suggestions(&self, error: &BinaryExportError) -> String {
        let base_message = error.to_string();
        
        let suggestion = match error {
            BinaryExportError::OutOfMemory { .. } => {
                "Try reducing memory usage by enabling compression or using streaming mode."
            }
            BinaryExportError::InsufficientDiskSpace { .. } => {
                "Free up disk space or choose a different output location."
            }
            BinaryExportError::CompressionError(_) => {
                "Try disabling compression or using a lower compression level."
            }
            BinaryExportError::PermissionDenied(_) => {
                "Check file permissions or run with appropriate privileges."
            }
            BinaryExportError::Timeout { .. } => {
                "Increase timeout value or try with a smaller dataset."
            }
            BinaryExportError::UnsupportedVersion { .. } => {
                "Update to a newer version or use a compatible format."
            }
            _ => "Check the error details and try again.",
        };

        format!("{}\n\nSuggestion: {}", base_message, suggestion)
    }
}

impl Default for ErrorRecovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = BinaryExportError::FileNotFound(PathBuf::from("/test/path"));
        assert!(error.to_string().contains("File not found"));
    }

    #[test]
    fn test_recovery_strategy_selection() {
        let recovery = ErrorRecovery::new();
        
        let memory_error = BinaryExportError::OutOfMemory { requested: 1000, available: 500 };
        let strategy = recovery.get_strategy(&memory_error);
        assert!(matches!(strategy, Some(RecoveryStrategy::RetryWithLessMemory { .. })));
        
        let io_error = BinaryExportError::IoError(std::io::ErrorKind::TimedOut);
        let strategy = recovery.get_strategy(&io_error);
        assert!(matches!(strategy, Some(RecoveryStrategy::Retry { .. })));
    }

    #[test]
    fn test_error_recoverability() {
        let recovery = ErrorRecovery::new();
        
        let recoverable = BinaryExportError::OutOfMemory { requested: 1000, available: 500 };
        assert!(recovery.is_recoverable(&recoverable));
        
        let non_recoverable = BinaryExportError::InvalidFormat("bad format".to_string());
        assert!(!recovery.is_recoverable(&non_recoverable));
    }

    #[test]
    fn test_error_suggestions() {
        let recovery = ErrorRecovery::new();
        let error = BinaryExportError::OutOfMemory { requested: 1000, available: 500 };
        let message = recovery.get_error_message_with_suggestions(&error);
        assert!(message.contains("Suggestion:"));
        assert!(message.contains("compression"));
    }
}