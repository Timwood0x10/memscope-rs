//! Error types for memory tracking operations.
//!
//! This module defines the unified error type for all tracking operations
//! within the memscope-rs library.

use std::fmt;

/// Result type for tracking operations.
pub type TrackingResult<T> = Result<T, TrackingError>;

/// Comprehensive error type for the tracking system.
///
/// This enum covers all possible error conditions that can occur
/// during memory tracking, analysis, and export operations.
#[derive(Debug)]
pub enum TrackingError {
    /// Memory allocation operation failed.
    AllocationFailed(String),
    /// Memory deallocation operation failed.
    DeallocationFailed(String),
    /// Memory tracking is currently disabled.
    TrackingDisabled,
    /// The provided pointer is invalid or null.
    InvalidPointer(String),
    /// Error occurred during data serialization.
    SerializationError(String),
    /// Error occurred during visualization generation.
    VisualizationError(String),
    /// Thread safety violation detected.
    ThreadSafetyError(String),
    /// Configuration parameter is invalid.
    ConfigurationError(String),
    /// Error occurred during memory analysis.
    AnalysisError(String),
    /// Error occurred during data export.
    ExportError(String),
    /// Memory corruption detected.
    MemoryCorruption(String),
    /// Unsafe operation detected and flagged.
    UnsafeOperationDetected(String),
    /// Foreign Function Interface error.
    FFIError(String),
    /// Scope management error.
    ScopeError(String),
    /// Borrow checker violation detected.
    BorrowCheckError(String),
    /// Lifetime management error.
    LifetimeError(String),
    /// Type inference failed.
    TypeInferenceError(String),
    /// Performance threshold exceeded.
    PerformanceError(String),
    /// System resources exhausted.
    ResourceExhausted(String),
    /// Internal system error.
    InternalError(String),
    /// Input/output operation failed.
    IoError(String),
    /// Lock acquisition failed due to contention.
    LockContention(String),
    /// Lock operation error.
    LockError(String),
    /// Channel communication error.
    ChannelError(String),
    /// Thread operation error.
    ThreadError(String),
    /// Initialization error.
    InitializationError(String),
    /// Feature not implemented.
    NotImplemented(String),
    /// Invalid operation.
    InvalidOperation(String),
    /// Validation error.
    ValidationError(String),
    /// Data operation error.
    DataError(String),
}

impl Clone for TrackingError {
    fn clone(&self) -> Self {
        match self {
            TrackingError::AllocationFailed(s) => TrackingError::AllocationFailed(s.clone()),
            TrackingError::DeallocationFailed(s) => TrackingError::DeallocationFailed(s.clone()),
            TrackingError::TrackingDisabled => TrackingError::TrackingDisabled,
            TrackingError::InvalidPointer(s) => TrackingError::InvalidPointer(s.clone()),
            TrackingError::SerializationError(s) => TrackingError::SerializationError(s.clone()),
            TrackingError::VisualizationError(s) => TrackingError::VisualizationError(s.clone()),
            TrackingError::ThreadSafetyError(s) => TrackingError::ThreadSafetyError(s.clone()),
            TrackingError::ConfigurationError(s) => TrackingError::ConfigurationError(s.clone()),
            TrackingError::AnalysisError(s) => TrackingError::AnalysisError(s.clone()),
            TrackingError::ExportError(s) => TrackingError::ExportError(s.clone()),
            TrackingError::MemoryCorruption(s) => TrackingError::MemoryCorruption(s.clone()),
            TrackingError::UnsafeOperationDetected(s) => {
                TrackingError::UnsafeOperationDetected(s.clone())
            }
            TrackingError::FFIError(s) => TrackingError::FFIError(s.clone()),
            TrackingError::ScopeError(s) => TrackingError::ScopeError(s.clone()),
            TrackingError::BorrowCheckError(s) => TrackingError::BorrowCheckError(s.clone()),
            TrackingError::LifetimeError(s) => TrackingError::LifetimeError(s.clone()),
            TrackingError::TypeInferenceError(s) => TrackingError::TypeInferenceError(s.clone()),
            TrackingError::PerformanceError(s) => TrackingError::PerformanceError(s.clone()),
            TrackingError::ResourceExhausted(s) => TrackingError::ResourceExhausted(s.clone()),
            TrackingError::InternalError(s) => TrackingError::InternalError(s.clone()),
            TrackingError::IoError(s) => TrackingError::IoError(s.clone()),
            TrackingError::LockError(s) => TrackingError::LockError(s.clone()),
            TrackingError::ChannelError(s) => TrackingError::ChannelError(s.clone()),
            TrackingError::ThreadError(s) => TrackingError::ThreadError(s.clone()),
            TrackingError::InitializationError(s) => TrackingError::InitializationError(s.clone()),
            TrackingError::NotImplemented(s) => TrackingError::NotImplemented(s.clone()),
            TrackingError::ValidationError(s) => TrackingError::ValidationError(s.clone()),
            TrackingError::InvalidOperation(s) => TrackingError::InvalidOperation(s.clone()),
            TrackingError::LockContention(s) => TrackingError::LockContention(s.clone()),
            TrackingError::DataError(s) => TrackingError::DataError(s.clone()),
        }
    }
}

impl fmt::Display for TrackingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrackingError::AllocationFailed(msg) => write!(f, "Allocation failed: {msg}"),
            TrackingError::DeallocationFailed(msg) => write!(f, "Deallocation failed: {msg}"),
            TrackingError::TrackingDisabled => write!(f, "Memory tracking is disabled"),
            TrackingError::InvalidPointer(msg) => write!(f, "Invalid pointer: {msg}"),
            TrackingError::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            TrackingError::VisualizationError(msg) => write!(f, "Visualization error: {msg}"),
            TrackingError::ThreadSafetyError(msg) => write!(f, "Thread safety error: {msg}"),
            TrackingError::ConfigurationError(msg) => write!(f, "Configuration error: {msg}"),
            TrackingError::AnalysisError(msg) => write!(f, "Analysis error: {msg}"),
            TrackingError::ExportError(msg) => write!(f, "Export error: {msg}"),
            TrackingError::MemoryCorruption(msg) => write!(f, "Memory corruption detected: {msg}"),
            TrackingError::UnsafeOperationDetected(msg) => {
                write!(f, "Unsafe operation detected: {msg}")
            }
            TrackingError::FFIError(msg) => write!(f, "FFI error: {msg}"),
            TrackingError::ScopeError(msg) => write!(f, "Scope error: {msg}"),
            TrackingError::BorrowCheckError(msg) => write!(f, "Borrow check error: {msg}"),
            TrackingError::LifetimeError(msg) => write!(f, "Lifetime error: {msg}"),
            TrackingError::TypeInferenceError(msg) => write!(f, "Type inference error: {msg}"),
            TrackingError::PerformanceError(msg) => write!(f, "Performance error: {msg}"),
            TrackingError::ResourceExhausted(msg) => write!(f, "Resource exhausted: {msg}"),
            TrackingError::InternalError(msg) => write!(f, "Internal error: {msg}"),
            TrackingError::IoError(msg) => write!(f, "IO error: {msg}"),
            TrackingError::LockError(msg) => write!(f, "Lock error: {msg}"),
            TrackingError::ChannelError(msg) => write!(f, "Channel error: {msg}"),
            TrackingError::ThreadError(msg) => write!(f, "Thread error: {msg}"),
            TrackingError::InitializationError(msg) => write!(f, "Initialization error: {msg}"),
            TrackingError::NotImplemented(msg) => write!(f, "Not implemented: {msg}"),
            TrackingError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
            TrackingError::InvalidOperation(msg) => write!(f, "Invalid operation: {msg}"),
            TrackingError::LockContention(msg) => write!(f, "Lock contention: {msg}"),
            TrackingError::DataError(msg) => write!(f, "Data error: {msg}"),
        }
    }
}

impl std::error::Error for TrackingError {}

impl From<std::io::Error> for TrackingError {
    fn from(error: std::io::Error) -> Self {
        TrackingError::IoError(error.to_string())
    }
}

impl From<serde_json::Error> for TrackingError {
    fn from(error: serde_json::Error) -> Self {
        TrackingError::SerializationError(format!("JSON error: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracking_error_creation() {
        let error = TrackingError::AllocationFailed("test error".to_string());
        assert!(error.to_string().contains("test error"));

        let error2 = TrackingError::TrackingDisabled;
        assert!(error2.to_string().contains("disabled"));
    }

    #[test]
    fn test_tracking_error_clone() {
        let original = TrackingError::InvalidPointer("null pointer".to_string());
        let cloned = original.clone();

        assert_eq!(original.to_string(), cloned.to_string());
    }

    #[test]
    fn test_tracking_result_type() {
        let success: TrackingResult<i32> = Ok(42);
        let failure: TrackingResult<i32> = Err(TrackingError::TrackingDisabled);

        assert!(success.is_ok());
        assert!(failure.is_err());
    }

    #[test]
    fn test_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let tracking_error: TrackingError = io_error.into();

        assert!(matches!(tracking_error, TrackingError::IoError(_)));
    }

    #[test]
    fn test_from_serde_error() {
        let json_error = serde_json::from_str::<i32>("not a number").unwrap_err();
        let tracking_error: TrackingError = json_error.into();

        assert!(matches!(
            tracking_error,
            TrackingError::SerializationError(_)
        ));
    }
}
