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

impl From<crate::capture::backends::core_types::TrackingError> for TrackingError {
    fn from(error: crate::capture::backends::core_types::TrackingError) -> Self {
        match error {
            crate::capture::backends::core_types::TrackingError::LockError(msg) => {
                TrackingError::LockError(msg)
            }
            crate::capture::backends::core_types::TrackingError::InvalidPointer(msg) => {
                TrackingError::InvalidPointer(msg)
            }
            crate::capture::backends::core_types::TrackingError::ExportError(msg) => {
                TrackingError::ExportError(msg)
            }
            crate::capture::backends::core_types::TrackingError::SerializationError(msg) => {
                TrackingError::SerializationError(msg)
            }
        }
    }
}

impl From<crate::core::types::TrackingError> for TrackingError {
    fn from(error: crate::core::types::TrackingError) -> Self {
        // Direct conversion since both have compatible variants
        // core::types::TrackingError has the same structure as capture::types::error::TrackingError
        match error {
            crate::core::types::TrackingError::AllocationFailed(msg) => {
                TrackingError::AllocationFailed(msg)
            }
            crate::core::types::TrackingError::DeallocationFailed(msg) => {
                TrackingError::DeallocationFailed(msg)
            }
            crate::core::types::TrackingError::TrackingDisabled => TrackingError::TrackingDisabled,
            crate::core::types::TrackingError::InvalidPointer(msg) => {
                TrackingError::InvalidPointer(msg)
            }
            crate::core::types::TrackingError::SerializationError(msg) => {
                TrackingError::SerializationError(msg)
            }
            crate::core::types::TrackingError::VisualizationError(msg) => {
                TrackingError::VisualizationError(msg)
            }
            crate::core::types::TrackingError::ThreadSafetyError(msg) => {
                TrackingError::ThreadSafetyError(msg)
            }
            crate::core::types::TrackingError::ConfigurationError(msg) => {
                TrackingError::ConfigurationError(msg)
            }
            crate::core::types::TrackingError::AnalysisError(msg) => {
                TrackingError::AnalysisError(msg)
            }
            crate::core::types::TrackingError::ExportError(msg) => TrackingError::ExportError(msg),
            crate::core::types::TrackingError::MemoryCorruption(msg) => {
                TrackingError::MemoryCorruption(msg)
            }
            crate::core::types::TrackingError::UnsafeOperationDetected(msg) => {
                TrackingError::UnsafeOperationDetected(msg)
            }
            crate::core::types::TrackingError::FFIError(msg) => TrackingError::FFIError(msg),
            crate::core::types::TrackingError::ScopeError(msg) => TrackingError::ScopeError(msg),
            crate::core::types::TrackingError::BorrowCheckError(msg) => {
                TrackingError::BorrowCheckError(msg)
            }
            crate::core::types::TrackingError::LifetimeError(msg) => {
                TrackingError::LifetimeError(msg)
            }
            crate::core::types::TrackingError::TypeInferenceError(msg) => {
                TrackingError::TypeInferenceError(msg)
            }
            crate::core::types::TrackingError::PerformanceError(msg) => {
                TrackingError::PerformanceError(msg)
            }
            crate::core::types::TrackingError::ResourceExhausted(msg) => {
                TrackingError::ResourceExhausted(msg)
            }
            crate::core::types::TrackingError::InternalError(msg) => {
                TrackingError::InternalError(msg)
            }
            crate::core::types::TrackingError::IoError(msg) => TrackingError::IoError(msg),
            crate::core::types::TrackingError::LockError(msg) => TrackingError::LockError(msg),
            crate::core::types::TrackingError::ChannelError(msg) => {
                TrackingError::ChannelError(msg)
            }
            crate::core::types::TrackingError::ThreadError(msg) => TrackingError::ThreadError(msg),
            crate::core::types::TrackingError::InitializationError(msg) => {
                TrackingError::InitializationError(msg)
            }
            crate::core::types::TrackingError::NotImplemented(msg) => {
                TrackingError::NotImplemented(msg)
            }
            crate::core::types::TrackingError::ValidationError(msg) => {
                TrackingError::ValidationError(msg)
            }
            crate::core::types::TrackingError::InvalidOperation(msg) => {
                TrackingError::InvalidOperation(msg)
            }
            crate::core::types::TrackingError::LockContention(msg) => {
                TrackingError::LockContention(msg)
            }
            crate::core::types::TrackingError::DataError(msg) => TrackingError::DataError(msg),
        }
    }
}

impl From<TrackingError> for crate::core::types::TrackingError {
    fn from(error: TrackingError) -> Self {
        // Reverse conversion
        match error {
            TrackingError::AllocationFailed(msg) => {
                crate::core::types::TrackingError::AllocationFailed(msg)
            }
            TrackingError::DeallocationFailed(msg) => {
                crate::core::types::TrackingError::DeallocationFailed(msg)
            }
            TrackingError::TrackingDisabled => crate::core::types::TrackingError::TrackingDisabled,
            TrackingError::InvalidPointer(msg) => {
                crate::core::types::TrackingError::InvalidPointer(msg)
            }
            TrackingError::SerializationError(msg) => {
                crate::core::types::TrackingError::SerializationError(msg)
            }
            TrackingError::VisualizationError(msg) => {
                crate::core::types::TrackingError::VisualizationError(msg)
            }
            TrackingError::ThreadSafetyError(msg) => {
                crate::core::types::TrackingError::ThreadSafetyError(msg)
            }
            TrackingError::ConfigurationError(msg) => {
                crate::core::types::TrackingError::ConfigurationError(msg)
            }
            TrackingError::AnalysisError(msg) => {
                crate::core::types::TrackingError::AnalysisError(msg)
            }
            TrackingError::ExportError(msg) => crate::core::types::TrackingError::ExportError(msg),
            TrackingError::MemoryCorruption(msg) => {
                crate::core::types::TrackingError::MemoryCorruption(msg)
            }
            TrackingError::UnsafeOperationDetected(msg) => {
                crate::core::types::TrackingError::UnsafeOperationDetected(msg)
            }
            TrackingError::FFIError(msg) => crate::core::types::TrackingError::FFIError(msg),
            TrackingError::ScopeError(msg) => crate::core::types::TrackingError::ScopeError(msg),
            TrackingError::BorrowCheckError(msg) => {
                crate::core::types::TrackingError::BorrowCheckError(msg)
            }
            TrackingError::LifetimeError(msg) => {
                crate::core::types::TrackingError::LifetimeError(msg)
            }
            TrackingError::TypeInferenceError(msg) => {
                crate::core::types::TrackingError::TypeInferenceError(msg)
            }
            TrackingError::PerformanceError(msg) => {
                crate::core::types::TrackingError::PerformanceError(msg)
            }
            TrackingError::ResourceExhausted(msg) => {
                crate::core::types::TrackingError::ResourceExhausted(msg)
            }
            TrackingError::InternalError(msg) => {
                crate::core::types::TrackingError::InternalError(msg)
            }
            TrackingError::IoError(msg) => crate::core::types::TrackingError::IoError(msg),
            TrackingError::LockError(msg) => crate::core::types::TrackingError::LockError(msg),
            TrackingError::ChannelError(msg) => {
                crate::core::types::TrackingError::ChannelError(msg)
            }
            TrackingError::ThreadError(msg) => crate::core::types::TrackingError::ThreadError(msg),
            TrackingError::InitializationError(msg) => {
                crate::core::types::TrackingError::InitializationError(msg)
            }
            TrackingError::NotImplemented(msg) => {
                crate::core::types::TrackingError::NotImplemented(msg)
            }
            TrackingError::ValidationError(msg) => {
                crate::core::types::TrackingError::ValidationError(msg)
            }
            TrackingError::InvalidOperation(msg) => {
                crate::core::types::TrackingError::InvalidOperation(msg)
            }
            TrackingError::LockContention(msg) => {
                crate::core::types::TrackingError::LockContention(msg)
            }
            TrackingError::DataError(msg) => crate::core::types::TrackingError::DataError(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify TrackingError creation with various variants
    /// Invariants: Each variant should format correctly
    #[test]
    fn test_tracking_error_creation() {
        let error = TrackingError::AllocationFailed("test error".to_string());
        assert!(
            error.to_string().contains("test error"),
            "Should contain error message"
        );
        assert!(
            error.to_string().contains("Allocation failed"),
            "Should contain error type"
        );

        let error2 = TrackingError::TrackingDisabled;
        assert!(
            error2.to_string().contains("disabled"),
            "Should indicate tracking disabled"
        );
    }

    /// Objective: Verify TrackingError clone functionality
    /// Invariants: Cloned error should have same string representation
    #[test]
    fn test_tracking_error_clone() {
        let original = TrackingError::InvalidPointer("null pointer".to_string());
        let cloned = original.clone();

        assert_eq!(
            original.to_string(),
            cloned.to_string(),
            "Cloned error should have same string representation"
        );
    }

    /// Objective: Verify TrackingResult type alias
    /// Invariants: Should work as Result<T, TrackingError>
    #[test]
    fn test_tracking_result_type() {
        let success: TrackingResult<i32> = Ok(42);
        let failure: TrackingResult<i32> = Err(TrackingError::TrackingDisabled);

        assert!(success.is_ok(), "Should be Ok");
        assert!(failure.is_err(), "Should be Err");
    }

    /// Objective: Verify From<std::io::Error> conversion
    /// Invariants: IO error should convert to IoError variant
    #[test]
    fn test_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let tracking_error: TrackingError = io_error.into();

        assert!(
            matches!(tracking_error, TrackingError::IoError(_)),
            "Should convert to IoError"
        );
        assert!(
            tracking_error.to_string().contains("file not found"),
            "Should preserve error message"
        );
    }

    /// Objective: Verify From<serde_json::Error> conversion
    /// Invariants: JSON error should convert to SerializationError variant
    #[test]
    fn test_from_serde_error() {
        let json_error = serde_json::from_str::<i32>("not a number").unwrap_err();
        let tracking_error: TrackingError = json_error.into();

        assert!(
            matches!(tracking_error, TrackingError::SerializationError(_)),
            "Should convert to SerializationError"
        );
        assert!(
            tracking_error.to_string().contains("JSON error"),
            "Should indicate JSON error"
        );
    }

    /// Objective: Verify Display implementation for all error variants
    /// Invariants: Each variant should have meaningful display output
    #[test]
    fn test_display_all_variants() {
        let errors = vec![
            (
                TrackingError::AllocationFailed("alloc".into()),
                "Allocation failed",
            ),
            (
                TrackingError::DeallocationFailed("dealloc".into()),
                "Deallocation failed",
            ),
            (TrackingError::TrackingDisabled, "disabled"),
            (
                TrackingError::InvalidPointer("ptr".into()),
                "Invalid pointer",
            ),
            (
                TrackingError::SerializationError("ser".into()),
                "Serialization error",
            ),
            (
                TrackingError::VisualizationError("viz".into()),
                "Visualization error",
            ),
            (
                TrackingError::ThreadSafetyError("thread".into()),
                "Thread safety error",
            ),
            (
                TrackingError::ConfigurationError("config".into()),
                "Configuration error",
            ),
            (
                TrackingError::AnalysisError("analysis".into()),
                "Analysis error",
            ),
            (TrackingError::ExportError("export".into()), "Export error"),
            (
                TrackingError::MemoryCorruption("corrupt".into()),
                "Memory corruption",
            ),
            (
                TrackingError::UnsafeOperationDetected("unsafe".into()),
                "Unsafe operation",
            ),
            (TrackingError::FFIError("ffi".into()), "FFI error"),
            (TrackingError::ScopeError("scope".into()), "Scope error"),
            (
                TrackingError::BorrowCheckError("borrow".into()),
                "Borrow check error",
            ),
            (
                TrackingError::LifetimeError("lifetime".into()),
                "Lifetime error",
            ),
            (
                TrackingError::TypeInferenceError("type".into()),
                "Type inference error",
            ),
            (
                TrackingError::PerformanceError("perf".into()),
                "Performance error",
            ),
            (
                TrackingError::ResourceExhausted("resource".into()),
                "Resource exhausted",
            ),
            (
                TrackingError::InternalError("internal".into()),
                "Internal error",
            ),
            (TrackingError::IoError("io".into()), "IO error"),
            (TrackingError::LockError("lock".into()), "Lock error"),
            (
                TrackingError::ChannelError("channel".into()),
                "Channel error",
            ),
            (TrackingError::ThreadError("thread".into()), "Thread error"),
            (
                TrackingError::InitializationError("init".into()),
                "Initialization error",
            ),
            (
                TrackingError::NotImplemented("not impl".into()),
                "Not implemented",
            ),
            (
                TrackingError::ValidationError("validation".into()),
                "Validation error",
            ),
            (
                TrackingError::InvalidOperation("invalid".into()),
                "Invalid operation",
            ),
            (
                TrackingError::LockContention("contention".into()),
                "Lock contention",
            ),
            (TrackingError::DataError("data".into()), "Data error"),
        ];

        for (error, expected_fragment) in errors {
            let display = error.to_string();
            assert!(
                display.contains(expected_fragment),
                "Error {:?} should contain '{}', got: {}",
                error,
                expected_fragment,
                display
            );
        }
    }

    /// Objective: Verify Clone implementation for all error variants
    /// Invariants: All variants should be cloneable
    #[test]
    fn test_clone_all_variants() {
        let errors: Vec<TrackingError> = vec![
            TrackingError::AllocationFailed("test".into()),
            TrackingError::DeallocationFailed("test".into()),
            TrackingError::TrackingDisabled,
            TrackingError::InvalidPointer("test".into()),
            TrackingError::SerializationError("test".into()),
            TrackingError::VisualizationError("test".into()),
            TrackingError::ThreadSafetyError("test".into()),
            TrackingError::ConfigurationError("test".into()),
            TrackingError::AnalysisError("test".into()),
            TrackingError::ExportError("test".into()),
            TrackingError::MemoryCorruption("test".into()),
            TrackingError::UnsafeOperationDetected("test".into()),
            TrackingError::FFIError("test".into()),
            TrackingError::ScopeError("test".into()),
            TrackingError::BorrowCheckError("test".into()),
            TrackingError::LifetimeError("test".into()),
            TrackingError::TypeInferenceError("test".into()),
            TrackingError::PerformanceError("test".into()),
            TrackingError::ResourceExhausted("test".into()),
            TrackingError::InternalError("test".into()),
            TrackingError::IoError("test".into()),
            TrackingError::LockError("test".into()),
            TrackingError::ChannelError("test".into()),
            TrackingError::ThreadError("test".into()),
            TrackingError::InitializationError("test".into()),
            TrackingError::NotImplemented("test".into()),
            TrackingError::ValidationError("test".into()),
            TrackingError::InvalidOperation("test".into()),
            TrackingError::LockContention("test".into()),
            TrackingError::DataError("test".into()),
        ];

        for error in errors {
            let cloned = error.clone();
            assert_eq!(
                error.to_string(),
                cloned.to_string(),
                "Cloned error should match original"
            );
        }
    }

    /// Objective: Verify std::error::Error implementation
    /// Invariants: TrackingError should implement Error trait
    #[test]
    fn test_error_trait() {
        let error = TrackingError::InvalidPointer("test".into());
        let _: &dyn std::error::Error = &error;
    }

    /// Objective: Verify Debug implementation
    /// Invariants: Debug output should contain variant name
    #[test]
    fn test_debug_implementation() {
        let error = TrackingError::AllocationFailed("test".into());
        let debug_str = format!("{:?}", error);
        assert!(
            debug_str.contains("AllocationFailed"),
            "Debug should contain variant name"
        );
    }

    /// Objective: Verify error chain with source()
    /// Invariants: TrackingError should not have a source
    #[test]
    fn test_error_source() {
        let error = TrackingError::InternalError("test".into());
        assert!(
            std::error::Error::source(&error).is_none(),
            "TrackingError should not have a source"
        );
    }

    /// Objective: Verify error can be used in Result context
    /// Invariants: Should work with ? operator
    #[test]
    fn test_result_context() -> TrackingResult<()> {
        fn inner_function() -> TrackingResult<()> {
            Err(TrackingError::NotImplemented("test".into()))
        }

        let result = inner_function();
        assert!(result.is_err(), "Should propagate error");

        let error = result.unwrap_err();
        assert!(
            matches!(error, TrackingError::NotImplemented(_)),
            "Should be NotImplemented error"
        );
        Ok(())
    }

    /// Objective: Verify error comparison through string representation
    /// Invariants: Same errors should have same string representation
    #[test]
    fn test_error_equality_via_string() {
        let error1 = TrackingError::InvalidPointer("null".into());
        let error2 = TrackingError::InvalidPointer("null".into());

        assert_eq!(
            error1.to_string(),
            error2.to_string(),
            "Same errors should have same string representation"
        );
    }

    /// Objective: Verify error message preservation
    /// Invariants: Error message should be preserved through conversions
    #[test]
    fn test_message_preservation() {
        let msg = "custom error message with special chars: \n\t\"quotes\"";
        let error = TrackingError::AnalysisError(msg.to_string());
        let display = error.to_string();

        assert!(
            display.contains(msg),
            "Original message should be preserved in display"
        );
    }

    /// Objective: Verify empty error message handling
    /// Invariants: Should handle empty messages gracefully
    #[test]
    fn test_empty_message() {
        let error = TrackingError::InvalidPointer("".into());
        let display = error.to_string();

        assert!(
            display.contains("Invalid pointer"),
            "Should still show error type even with empty message"
        );
    }

    /// Objective: Verify error with long message
    /// Invariants: Should handle long messages
    #[test]
    fn test_long_message() {
        let long_msg = "x".repeat(1000);
        let error = TrackingError::InternalError(long_msg.clone());
        let display = error.to_string();

        assert!(display.contains(&long_msg), "Should preserve long message");
    }

    /// Objective: Verify error with unicode message
    /// Invariants: Should handle unicode characters
    #[test]
    fn test_unicode_message() {
        let unicode_msg = "错误信息 🎯 日本語 العربية";
        let error = TrackingError::ValidationError(unicode_msg.to_string());
        let display = error.to_string();

        assert!(
            display.contains(unicode_msg),
            "Should preserve unicode characters"
        );
    }
}
