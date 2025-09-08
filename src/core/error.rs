//! Unified error handling system for memscope-rs
//!
//! This module provides a simplified, efficient error handling system that:
//! - Reduces string cloning overhead using `Arc<str>`
//! - Maintains all existing error information
//! - Provides error recovery mechanisms
//! - Ensures backward compatibility

use std::fmt;
use std::sync::Arc;

/// Unified error type for the entire memscope-rs system
///
/// This replaces the complex TrackingError with a simpler, more efficient design
/// while maintaining all existing error information and backward compatibility.
#[derive(Debug, Clone)]
pub enum MemScopeError {
    /// Memory tracking operations (allocation, deallocation, association)
    Memory {
        operation: MemoryOperation,
        message: Arc<str>,
        context: Option<Arc<str>>,
    },

    /// Analysis operations (fragmentation, lifecycle, etc.)
    Analysis {
        analyzer: Arc<str>,
        message: Arc<str>,
        recoverable: bool,
    },

    /// Export operations (JSON, binary, HTML, etc.)
    Export {
        format: Arc<str>,
        message: Arc<str>,
        partial_success: bool,
    },

    /// Configuration and initialization errors
    Configuration {
        component: Arc<str>,
        message: Arc<str>,
    },

    /// System-level errors (IO, threading, etc.)
    System {
        error_type: SystemErrorType,
        message: Arc<str>,
        source_message: Option<Arc<str>>,
    },

    /// Internal errors that should not normally occur
    Internal {
        message: Arc<str>,
        location: Option<Arc<str>>,
    },
}

/// Memory operation types for better error categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryOperation {
    Allocation,
    Deallocation,
    Association,
    Tracking,
    Validation,
}

/// System error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemErrorType {
    Io,
    Threading,
    Locking,
    Channel,
    Serialization,
    Network,
    FileSystem,
}

/// Result type for all memscope operations
pub type Result<T> = std::result::Result<T, MemScopeError>;

impl MemScopeError {
    /// Create a memory operation error
    pub fn memory(operation: MemoryOperation, message: impl Into<Arc<str>>) -> Self {
        Self::Memory {
            operation,
            message: message.into(),
            context: None,
        }
    }

    /// Create a memory operation error with context
    pub fn memory_with_context(
        operation: MemoryOperation,
        message: impl Into<Arc<str>>,
        context: impl Into<Arc<str>>,
    ) -> Self {
        Self::Memory {
            operation,
            message: message.into(),
            context: Some(context.into()),
        }
    }

    /// Create an analysis error
    pub fn analysis(analyzer: impl Into<Arc<str>>, message: impl Into<Arc<str>>) -> Self {
        Self::Analysis {
            analyzer: analyzer.into(),
            message: message.into(),
            recoverable: true,
        }
    }

    /// Create a non-recoverable analysis error
    pub fn analysis_fatal(analyzer: impl Into<Arc<str>>, message: impl Into<Arc<str>>) -> Self {
        Self::Analysis {
            analyzer: analyzer.into(),
            message: message.into(),
            recoverable: false,
        }
    }

    /// Create an export error
    pub fn export(format: impl Into<Arc<str>>, message: impl Into<Arc<str>>) -> Self {
        Self::Export {
            format: format.into(),
            message: message.into(),
            partial_success: false,
        }
    }

    /// Create an export error with partial success
    pub fn export_partial(format: impl Into<Arc<str>>, message: impl Into<Arc<str>>) -> Self {
        Self::Export {
            format: format.into(),
            message: message.into(),
            partial_success: true,
        }
    }

    /// Create a configuration error
    pub fn config(component: impl Into<Arc<str>>, message: impl Into<Arc<str>>) -> Self {
        Self::Configuration {
            component: component.into(),
            message: message.into(),
        }
    }

    /// Create a system error
    pub fn system(error_type: SystemErrorType, message: impl Into<Arc<str>>) -> Self {
        Self::System {
            error_type,
            message: message.into(),
            source_message: None,
        }
    }

    /// Create a system error with source
    pub fn system_with_source(
        error_type: SystemErrorType,
        message: impl Into<Arc<str>>,
        source: impl std::error::Error,
    ) -> Self {
        Self::System {
            error_type,
            message: message.into(),
            source_message: Some(Arc::from(source.to_string())),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<Arc<str>>) -> Self {
        Self::Internal {
            message: message.into(),
            location: None,
        }
    }

    /// Create an internal error with location
    pub fn internal_at(message: impl Into<Arc<str>>, location: impl Into<Arc<str>>) -> Self {
        Self::Internal {
            message: message.into(),
            location: Some(location.into()),
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Memory { .. } => true,
            Self::Analysis { recoverable, .. } => *recoverable,
            Self::Export {
                partial_success, ..
            } => *partial_success,
            Self::Configuration { .. } => false,
            Self::System { error_type, .. } => matches!(
                error_type,
                SystemErrorType::Io | SystemErrorType::Network | SystemErrorType::Locking
            ),
            Self::Internal { .. } => false,
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Memory { .. } => ErrorSeverity::Medium,
            Self::Analysis {
                recoverable: false, ..
            } => ErrorSeverity::High,
            Self::Analysis { .. } => ErrorSeverity::Low,
            Self::Export {
                partial_success: true,
                ..
            } => ErrorSeverity::Low,
            Self::Export { .. } => ErrorSeverity::Medium,
            Self::Configuration { .. } => ErrorSeverity::High,
            Self::System {
                error_type: SystemErrorType::Threading,
                ..
            } => ErrorSeverity::High,
            Self::System { .. } => ErrorSeverity::Medium,
            Self::Internal { .. } => ErrorSeverity::Critical,
        }
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> &str {
        match self {
            Self::Memory { message, .. } => message,
            Self::Analysis { message, .. } => message,
            Self::Export { message, .. } => message,
            Self::Configuration { message, .. } => message,
            Self::System { message, .. } => message,
            Self::Internal { message, .. } => message,
        }
    }

    /// Get error category for logging/metrics
    pub fn category(&self) -> &'static str {
        match self {
            Self::Memory { .. } => "memory",
            Self::Analysis { .. } => "analysis",
            Self::Export { .. } => "export",
            Self::Configuration { .. } => "config",
            Self::System { .. } => "system",
            Self::Internal { .. } => "internal",
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for MemScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Memory {
                operation,
                message,
                context,
            } => {
                write!(
                    f,
                    "Memory {} error: {}",
                    match operation {
                        MemoryOperation::Allocation => "allocation",
                        MemoryOperation::Deallocation => "deallocation",
                        MemoryOperation::Association => "association",
                        MemoryOperation::Tracking => "tracking",
                        MemoryOperation::Validation => "validation",
                    },
                    message
                )?;
                if let Some(ctx) = context {
                    write!(f, " (context: {ctx})")?;
                }
                Ok(())
            }
            Self::Analysis {
                analyzer, message, ..
            } => {
                write!(f, "Analysis error in {analyzer}: {message}")
            }
            Self::Export {
                format,
                message,
                partial_success,
            } => {
                if *partial_success {
                    write!(f, "Partial export error ({format}): {message}")
                } else {
                    write!(f, "Export error ({format}): {message}")
                }
            }
            Self::Configuration { component, message } => {
                write!(f, "Configuration error in {component}: {message}")
            }
            Self::System {
                error_type,
                message,
                source_message,
            } => {
                write!(
                    f,
                    "{} error: {}",
                    match error_type {
                        SystemErrorType::Io => "I/O",
                        SystemErrorType::Threading => "Threading",
                        SystemErrorType::Locking => "Locking",
                        SystemErrorType::Channel => "Channel",
                        SystemErrorType::Serialization => "Serialization",
                        SystemErrorType::Network => "Network",
                        SystemErrorType::FileSystem => "File system",
                    },
                    message
                )?;
                if let Some(source) = source_message {
                    write!(f, " (source: {source})")?;
                }
                Ok(())
            }
            Self::Internal { message, location } => {
                write!(f, "Internal error: {message}")?;
                if let Some(loc) = location {
                    write!(f, " at {loc}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for MemScopeError {}

// Conversion from standard library errors
impl From<std::io::Error> for MemScopeError {
    fn from(err: std::io::Error) -> Self {
        Self::system_with_source(
            SystemErrorType::Io,
            format!("I/O operation failed: {err}"),
            err,
        )
    }
}

impl From<serde_json::Error> for MemScopeError {
    fn from(err: serde_json::Error) -> Self {
        Self::system_with_source(
            SystemErrorType::Serialization,
            format!("JSON serialization failed: {err}"),
            err,
        )
    }
}

// Backward compatibility: conversion from old TrackingError
impl From<crate::core::types::TrackingError> for MemScopeError {
    fn from(err: crate::core::types::TrackingError) -> Self {
        use crate::core::types::TrackingError as TE;

        match err {
            TE::AllocationFailed(msg) => Self::memory(MemoryOperation::Allocation, msg),
            TE::DeallocationFailed(msg) => Self::memory(MemoryOperation::Deallocation, msg),
            TE::TrackingDisabled => Self::config("tracker", "Memory tracking is disabled"),
            TE::InvalidPointer(msg) => Self::memory(MemoryOperation::Validation, msg),
            TE::SerializationError(msg) => Self::system(SystemErrorType::Serialization, msg),
            TE::VisualizationError(msg) => Self::export("visualization", msg),
            TE::ThreadSafetyError(msg) => Self::system(SystemErrorType::Threading, msg),
            TE::ConfigurationError(msg) => Self::config("general", msg),
            TE::AnalysisError(msg) => Self::analysis("general", msg),
            TE::ExportError(msg) => Self::export("general", msg),
            TE::MemoryCorruption(msg) => Self::memory(MemoryOperation::Validation, msg),
            TE::UnsafeOperationDetected(msg) => Self::analysis("unsafe", msg),
            TE::FFIError(msg) => Self::analysis("ffi", msg),
            TE::ScopeError(msg) => Self::memory(MemoryOperation::Tracking, msg),
            TE::BorrowCheckError(msg) => Self::analysis("borrow", msg),
            TE::LifetimeError(msg) => Self::analysis("lifetime", msg),
            TE::TypeInferenceError(msg) => Self::analysis("type", msg),
            TE::PerformanceError(msg) => Self::system(SystemErrorType::Io, msg),
            TE::ResourceExhausted(msg) => Self::system(SystemErrorType::Io, msg),
            TE::InternalError(msg) => Self::internal(msg),
            TE::IoError(msg) => Self::system(SystemErrorType::Io, msg),
            TE::LockError(msg) => Self::system(SystemErrorType::Locking, msg),
            TE::ChannelError(msg) => Self::system(SystemErrorType::Channel, msg),
            TE::ThreadError(msg) => Self::system(SystemErrorType::Threading, msg),
            TE::InitializationError(msg) => Self::config("initialization", msg),
            TE::NotImplemented(msg) => Self::internal(msg),
            TE::ValidationError(msg) => Self::memory(MemoryOperation::Validation, msg),
            TE::InvalidOperation(msg) => Self::memory(MemoryOperation::Tracking, msg),
            TE::LockContention(msg) => Self::memory(MemoryOperation::Tracking, msg),
            TE::DataError(msg) => Self::memory(MemoryOperation::Tracking, msg),
        }
    }
}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// Retry the operation with the same parameters
    Retry { max_attempts: u32, delay_ms: u64 },
    /// Use a default value and continue
    UseDefault { value: String },
    /// Skip this operation and continue
    Skip,
    /// Abort the current operation
    Abort,
    /// Try an alternative approach
    Fallback { strategy: String },
}

/// Error recovery trait for implementing recovery strategies
pub trait ErrorRecovery {
    /// Determine if an error can be recovered from
    fn can_recover(&self, error: &MemScopeError) -> bool;

    /// Get the recovery action for an error
    fn get_recovery_action(&self, error: &MemScopeError) -> Option<RecoveryAction>;

    /// Execute recovery action
    fn execute_recovery(&self, action: &RecoveryAction) -> Result<()>;
}

/// Default error recovery implementation
pub struct DefaultErrorRecovery {
    max_retries: u32,
    retry_delay_ms: u64,
}

impl DefaultErrorRecovery {
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 100,
        }
    }

    pub fn with_config(max_retries: u32, retry_delay_ms: u64) -> Self {
        Self {
            max_retries,
            retry_delay_ms,
        }
    }
}

impl Default for DefaultErrorRecovery {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorRecovery for DefaultErrorRecovery {
    fn can_recover(&self, error: &MemScopeError) -> bool {
        error.is_recoverable() && error.severity() != ErrorSeverity::Critical
    }

    fn get_recovery_action(&self, error: &MemScopeError) -> Option<RecoveryAction> {
        if !self.can_recover(error) {
            return Some(RecoveryAction::Abort);
        }

        match error {
            MemScopeError::Memory { .. } => Some(RecoveryAction::Retry {
                max_attempts: self.max_retries,
                delay_ms: self.retry_delay_ms,
            }),
            MemScopeError::Analysis { .. } => Some(RecoveryAction::UseDefault {
                value: "{}".to_string(), // Empty JSON object as default
            }),
            MemScopeError::Export {
                partial_success: true,
                ..
            } => Some(RecoveryAction::Skip),
            MemScopeError::Export { .. } => Some(RecoveryAction::Fallback {
                strategy: "minimal_export".to_string(),
            }),
            MemScopeError::System {
                error_type: SystemErrorType::Locking,
                ..
            } => Some(RecoveryAction::Retry {
                max_attempts: 1,
                delay_ms: 50,
            }),
            _ => Some(RecoveryAction::Abort),
        }
    }

    fn execute_recovery(&self, action: &RecoveryAction) -> Result<()> {
        match action {
            RecoveryAction::Retry { delay_ms, .. } => {
                if *delay_ms > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(*delay_ms));
                }
                Ok(())
            }
            RecoveryAction::UseDefault { .. } => Ok(()),
            RecoveryAction::Skip => Ok(()),
            RecoveryAction::Fallback { .. } => Ok(()),
            RecoveryAction::Abort => Err(MemScopeError::internal("Recovery aborted")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    // Test all error creation methods
    #[test]
    fn test_memory_error_creation() {
        let err = MemScopeError::memory(MemoryOperation::Allocation, "allocation failed");
        assert!(matches!(
            err,
            MemScopeError::Memory {
                operation: MemoryOperation::Allocation,
                ..
            }
        ));
        assert_eq!(err.category(), "memory");
        assert_eq!(err.user_message(), "allocation failed");
        assert!(err.is_recoverable());
        assert_eq!(err.severity(), ErrorSeverity::Medium);

        let err_with_context = MemScopeError::memory_with_context(
            MemoryOperation::Deallocation,
            "deallocation failed",
            "in cleanup",
        );
        assert!(matches!(
            err_with_context,
            MemScopeError::Memory {
                operation: MemoryOperation::Deallocation,
                ..
            }
        ));
        assert_eq!(err_with_context.category(), "memory");
    }

    #[test]
    fn test_analysis_error_creation() {
        let err = MemScopeError::analysis("fragmentation", "analysis failed");
        assert!(matches!(
            err,
            MemScopeError::Analysis {
                recoverable: true,
                ..
            }
        ));
        assert_eq!(err.category(), "analysis");
        assert!(err.is_recoverable());
        assert_eq!(err.severity(), ErrorSeverity::Low);

        let fatal_err = MemScopeError::analysis_fatal("lifecycle", "critical analysis failed");
        assert!(matches!(
            fatal_err,
            MemScopeError::Analysis {
                recoverable: false,
                ..
            }
        ));
        assert_eq!(fatal_err.category(), "analysis");
        assert!(!fatal_err.is_recoverable());
        assert_eq!(fatal_err.severity(), ErrorSeverity::High);
    }

    #[test]
    fn test_export_error_creation() {
        let err = MemScopeError::export("json", "export failed");
        assert!(matches!(
            err,
            MemScopeError::Export {
                partial_success: false,
                ..
            }
        ));
        assert_eq!(err.category(), "export");
        assert!(!err.is_recoverable());
        assert_eq!(err.severity(), ErrorSeverity::Medium);

        let partial_err = MemScopeError::export_partial("html", "partial export");
        assert!(matches!(
            partial_err,
            MemScopeError::Export {
                partial_success: true,
                ..
            }
        ));
        assert_eq!(partial_err.category(), "export");
        assert!(partial_err.is_recoverable());
        assert_eq!(partial_err.severity(), ErrorSeverity::Low);
    }

    #[test]
    fn test_config_error_creation() {
        let err = MemScopeError::config("tracker", "invalid configuration");
        assert!(matches!(err, MemScopeError::Configuration { .. }));
        assert_eq!(err.category(), "config");
        assert!(!err.is_recoverable());
        assert_eq!(err.severity(), ErrorSeverity::High);
    }

    #[test]
    fn test_system_error_creation() {
        let err = MemScopeError::system(SystemErrorType::Io, "io error");
        assert!(matches!(
            err,
            MemScopeError::System {
                error_type: SystemErrorType::Io,
                ..
            }
        ));
        assert_eq!(err.category(), "system");
        assert!(err.is_recoverable());
        assert_eq!(err.severity(), ErrorSeverity::Medium);

        let io_err = io::Error::other("test io error");
        let converted_err: MemScopeError = io_err.into();
        assert!(matches!(
            converted_err,
            MemScopeError::System {
                error_type: SystemErrorType::Io,
                ..
            }
        ));
        assert_eq!(converted_err.category(), "system");
    }

    #[test]
    fn test_internal_error_creation() {
        let err = MemScopeError::internal("internal error");
        assert!(matches!(err, MemScopeError::Internal { .. }));
        assert_eq!(err.category(), "internal");
        assert!(!err.is_recoverable());
        assert_eq!(err.severity(), ErrorSeverity::Critical);

        let err_with_location = MemScopeError::internal_at("internal error", "test_function");
        assert!(matches!(err_with_location, MemScopeError::Internal { .. }));
        assert_eq!(err_with_location.category(), "internal");
    }

    #[test]
    fn test_error_display_formatting() {
        // Test memory error display
        let mem_err = MemScopeError::memory_with_context(
            MemoryOperation::Allocation,
            "allocation failed",
            "in test function",
        );
        let mem_display = format!("{mem_err}");
        assert!(mem_display.contains("Memory allocation error"));
        assert!(mem_display.contains("allocation failed"));
        assert!(mem_display.contains("context: in test function"));

        // Test analysis error display
        let analysis_err = MemScopeError::analysis("fragmentation", "analysis failed");
        let analysis_display = format!("{analysis_err}");
        assert!(analysis_display.contains("Analysis error in fragmentation: analysis failed"));

        // Test export error display
        let export_err = MemScopeError::export("json", "export failed");
        let export_display = format!("{export_err}");
        assert!(export_display.contains("Export error (json): export failed"));

        let partial_export_err = MemScopeError::export_partial("html", "partial export");
        let partial_export_display = format!("{partial_export_err}");
        assert!(partial_export_display.contains("Partial export error (html): partial export"));

        // Test configuration error display
        let config_err = MemScopeError::config("tracker", "invalid config");
        let config_display = format!("{config_err}");
        assert!(config_display.contains("Configuration error in tracker: invalid config"));

        // Test system error display
        let system_err = MemScopeError::system(SystemErrorType::Io, "io error");
        let system_display = format!("{system_err}");
        assert!(system_display.contains("I/O error: io error"));

        // Test internal error display
        let internal_err = MemScopeError::internal_at("internal error", "test_function");
        let internal_display = format!("{internal_err}");
        assert!(internal_display.contains("Internal error: internal error at test_function"));
    }

    #[test]
    fn test_error_severity_ordering() {
        assert!(ErrorSeverity::Low < ErrorSeverity::Medium);
        assert!(ErrorSeverity::Medium < ErrorSeverity::High);
        assert!(ErrorSeverity::High < ErrorSeverity::Critical);
    }

    #[test]
    fn test_memory_operation_variants() {
        let operations = [
            MemoryOperation::Allocation,
            MemoryOperation::Deallocation,
            MemoryOperation::Association,
            MemoryOperation::Tracking,
            MemoryOperation::Validation,
        ];

        for op in operations {
            let err = MemScopeError::memory(op, "test");
            assert_eq!(err.category(), "memory");
        }
    }

    #[test]
    fn test_system_error_type_variants() {
        let error_types = [
            SystemErrorType::Io,
            SystemErrorType::Threading,
            SystemErrorType::Locking,
            SystemErrorType::Channel,
            SystemErrorType::Serialization,
            SystemErrorType::Network,
            SystemErrorType::FileSystem,
        ];

        for error_type in error_types {
            let err = MemScopeError::system(error_type, "test");
            assert_eq!(err.category(), "system");
        }
    }

    #[test]
    fn test_backward_compatibility() {
        use crate::core::types::TrackingError;

        // Test conversion of all TrackingError variants
        let test_cases = vec![
            TrackingError::AllocationFailed("alloc".to_string()),
            TrackingError::DeallocationFailed("dealloc".to_string()),
            TrackingError::TrackingDisabled,
            TrackingError::InvalidPointer("invalid ptr".to_string()),
            TrackingError::SerializationError("serialize".to_string()),
            TrackingError::VisualizationError("visualize".to_string()),
            TrackingError::ThreadSafetyError("thread".to_string()),
            TrackingError::ConfigurationError("config".to_string()),
            TrackingError::AnalysisError("analysis".to_string()),
            TrackingError::ExportError("export".to_string()),
            TrackingError::MemoryCorruption("corruption".to_string()),
            TrackingError::UnsafeOperationDetected("unsafe".to_string()),
            TrackingError::FFIError("ffi".to_string()),
            TrackingError::ScopeError("scope".to_string()),
            TrackingError::BorrowCheckError("borrow".to_string()),
            TrackingError::LifetimeError("lifetime".to_string()),
            TrackingError::TypeInferenceError("type".to_string()),
            TrackingError::PerformanceError("perf".to_string()),
            TrackingError::ResourceExhausted("resource".to_string()),
            TrackingError::InternalError("internal".to_string()),
            TrackingError::IoError("io".to_string()),
            TrackingError::LockError("lock".to_string()),
            TrackingError::ChannelError("channel".to_string()),
            TrackingError::ThreadError("thread".to_string()),
            TrackingError::InitializationError("init".to_string()),
            TrackingError::NotImplemented("not_impl".to_string()),
            TrackingError::ValidationError("validation".to_string()),
            TrackingError::InvalidOperation("invalid_op".to_string()),
            TrackingError::LockContention("contention".to_string()),
            TrackingError::DataError("data".to_string()),
        ];

        for old_err in test_cases {
            let new_err: MemScopeError = old_err.into();
            // Just ensure conversion doesn't panic and produces a valid error
            assert!(!new_err.category().is_empty());
        }
    }

    #[test]
    fn test_error_recovery_strategies() {
        let recovery = DefaultErrorRecovery::new();

        // Test memory error recovery
        let mem_err = MemScopeError::memory(MemoryOperation::Allocation, "test error");
        assert!(recovery.can_recover(&mem_err));
        let action = recovery.get_recovery_action(&mem_err);
        assert!(matches!(action, Some(RecoveryAction::Retry { .. })));

        // Test analysis error recovery
        let analysis_err = MemScopeError::analysis("test", "analysis error");
        assert!(recovery.can_recover(&analysis_err));
        let action = recovery.get_recovery_action(&analysis_err);
        assert!(matches!(action, Some(RecoveryAction::UseDefault { .. })));

        // Test partial export recovery
        let partial_export_err = MemScopeError::export_partial("json", "partial export");
        assert!(recovery.can_recover(&partial_export_err));
        let action = recovery.get_recovery_action(&partial_export_err);
        assert!(matches!(action, Some(RecoveryAction::Skip)));

        // Test full export recovery
        let export_err = MemScopeError::export("json", "export error");
        // Export errors are not recoverable according to is_recoverable() implementation
        assert!(!export_err.is_recoverable());
        assert!(!recovery.can_recover(&export_err)); // Not recoverable
        let action = recovery.get_recovery_action(&export_err);
        assert!(matches!(action, Some(RecoveryAction::Abort)));

        // Test internal error recovery
        let internal_err = MemScopeError::internal("internal error");
        assert!(!recovery.can_recover(&internal_err)); // Not recoverable
        let action = recovery.get_recovery_action(&internal_err);
        assert!(matches!(action, Some(RecoveryAction::Abort)));
    }

    #[test]
    fn test_recovery_execution() {
        let recovery = DefaultErrorRecovery::new();

        // Test retry execution
        let retry_action = RecoveryAction::Retry {
            max_attempts: 1,
            delay_ms: 1, // Minimal delay for testing
        };
        assert!(recovery.execute_recovery(&retry_action).is_ok());

        // Test use default execution
        let default_action = RecoveryAction::UseDefault {
            value: "test".to_string(),
        };
        assert!(recovery.execute_recovery(&default_action).is_ok());

        // Test skip execution
        let skip_action = RecoveryAction::Skip;
        assert!(recovery.execute_recovery(&skip_action).is_ok());

        // Test fallback execution
        let fallback_action = RecoveryAction::Fallback {
            strategy: "test".to_string(),
        };
        assert!(recovery.execute_recovery(&fallback_action).is_ok());

        // Test abort execution
        let abort_action = RecoveryAction::Abort;
        assert!(recovery.execute_recovery(&abort_action).is_err());
    }

    #[test]
    fn test_result_type_alias() {
        fn test_function() -> Result<()> {
            Ok(())
        }

        assert!(test_function().is_ok());
    }

    #[test]
    fn test_serde_conversion() {
        let io_err = std::io::Error::other("test io error");
        let json_err = serde_json::Error::io(io_err);
        let memscope_err: MemScopeError = json_err.into();
        assert!(matches!(
            memscope_err,
            MemScopeError::System {
                error_type: SystemErrorType::Serialization,
                ..
            }
        ));
        assert_eq!(memscope_err.category(), "system");
    }
}
