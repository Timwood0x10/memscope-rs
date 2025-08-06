//! Unified error handling system for memscope-rs
//! 
//! This module provides a simplified, efficient error handling system that:
//! - Reduces string cloning overhead using Arc<str>
//! - Maintains all existing error information
//! - Provides error recovery mechanisms
//! - Ensures backward compatibility

use std::sync::Arc;
use std::fmt;

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
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
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
        context: impl Into<Arc<str>>
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
            source: None,
        }
    }
    
    /// Create a system error with source
    pub fn system_with_source(
        error_type: SystemErrorType, 
        message: impl Into<Arc<str>>,
        source: Box<dyn std::error::Error + Send + Sync>
    ) -> Self {
        Self::System {
            error_type,
            message: message.into(),
            source: Some(source),
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
            Self::Export { partial_success, .. } => *partial_success,
            Self::Configuration { .. } => false,
            Self::System { error_type, .. } => matches!(error_type, 
                SystemErrorType::Io | SystemErrorType::Network | SystemErrorType::Locking
            ),
            Self::Internal { .. } => false,
        }
    }
    
    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Memory { .. } => ErrorSeverity::Medium,
            Self::Analysis { recoverable: false, .. } => ErrorSeverity::High,
            Self::Analysis { .. } => ErrorSeverity::Low,
            Self::Export { partial_success: true, .. } => ErrorSeverity::Low,
            Self::Export { .. } => ErrorSeverity::Medium,
            Self::Configuration { .. } => ErrorSeverity::High,
            Self::System { error_type: SystemErrorType::Threading, .. } => ErrorSeverity::High,
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
            Self::Memory { operation, message, context } => {
                write!(f, "Memory {} error: {}", 
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
                    write!(f, " (context: {})", ctx)?;
                }
                Ok(())
            }
            Self::Analysis { analyzer, message, .. } => {
                write!(f, "Analysis error in {}: {}", analyzer, message)
            }
            Self::Export { format, message, partial_success } => {
                if *partial_success {
                    write!(f, "Partial export error ({}): {}", format, message)
                } else {
                    write!(f, "Export error ({}): {}", format, message)
                }
            }
            Self::Configuration { component, message } => {
                write!(f, "Configuration error in {}: {}", component, message)
            }
            Self::System { error_type, message, .. } => {
                write!(f, "{} error: {}", 
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
                )
            }
            Self::Internal { message, location } => {
                write!(f, "Internal error: {}", message)?;
                if let Some(loc) = location {
                    write!(f, " at {}", loc)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for MemScopeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::System { source: Some(source), .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

// Conversion from standard library errors
impl From<std::io::Error> for MemScopeError {
    fn from(err: std::io::Error) -> Self {
        Self::system_with_source(
            SystemErrorType::Io,
            format!("I/O operation failed: {}", err),
            Box::new(err)
        )
    }
}

impl From<serde_json::Error> for MemScopeError {
    fn from(err: serde_json::Error) -> Self {
        Self::system_with_source(
            SystemErrorType::Serialization,
            format!("JSON serialization failed: {}", err),
            Box::new(err)
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
            MemScopeError::Export { partial_success: true, .. } => Some(RecoveryAction::Skip),
            MemScopeError::Export { .. } => Some(RecoveryAction::Fallback {
                strategy: "minimal_export".to_string(),
            }),
            MemScopeError::System { error_type: SystemErrorType::Locking, .. } => {
                Some(RecoveryAction::Retry {
                    max_attempts: 1,
                    delay_ms: 50,
                })
            }
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
    
    #[test]
    fn test_error_creation() {
        let err = MemScopeError::memory(MemoryOperation::Allocation, "test allocation failed");
        assert_eq!(err.category(), "memory");
        assert_eq!(err.severity(), ErrorSeverity::Medium);
        assert!(err.is_recoverable());
    }
    
    #[test]
    fn test_error_recovery() {
        let recovery = DefaultErrorRecovery::new();
        let err = MemScopeError::memory(MemoryOperation::Allocation, "test error");
        
        assert!(recovery.can_recover(&err));
        
        let action = recovery.get_recovery_action(&err);
        assert!(matches!(action, Some(RecoveryAction::Retry { .. })));
    }
    
    #[test]
    fn test_error_display() {
        let err = MemScopeError::memory_with_context(
            MemoryOperation::Allocation,
            "allocation failed",
            "in test function"
        );
        
        let display = format!("{}", err);
        assert!(display.contains("Memory allocation error"));
        assert!(display.contains("allocation failed"));
        assert!(display.contains("context: in test function"));
    }
    
    #[test]
    fn test_backward_compatibility() {
        use crate::core::types::TrackingError;
        
        let old_err = TrackingError::AllocationFailed("test".to_string());
        let new_err: MemScopeError = old_err.into();
        
        assert_eq!(new_err.category(), "memory");
        assert!(matches!(new_err, MemScopeError::Memory { .. }));
    }
}