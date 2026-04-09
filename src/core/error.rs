//! Unified error handling system for memscope-rs
//!
//! This module provides a simplified, efficient error handling system that:
//! - Reduces string cloning overhead using `Arc<str>`
//! - Maintains all existing error information
//! - Provides error recovery mechanisms
//! - Ensures backward compatibility

use std::fmt;
use std::sync::Arc;
use std::time::SystemTime;

/// Error context information for tracking and debugging
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorContext {
    /// Timestamp when the error occurred
    pub timestamp: SystemTime,
    /// File where the error originated
    pub file: Option<String>,
    /// Line number where the error originated
    pub line: Option<u32>,
    /// Additional context information
    pub extra: Option<String>,
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::now(),
            file: None,
            line: None,
            extra: None,
        }
    }
}

impl ErrorContext {
    /// Create new error context
    pub fn new() -> Self {
        Self::default()
    }

    /// Create error context with file and line information
    pub fn with_location(file: impl Into<String>, line: u32) -> Self {
        Self {
            timestamp: SystemTime::now(),
            file: Some(file.into()),
            line: Some(line),
            extra: None,
        }
    }

    /// Add extra context information
    pub fn with_extra(mut self, extra: impl Into<String>) -> Self {
        self.extra = Some(extra.into());
        self
    }
}

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
        error_kind: ErrorKind,
        error_context: ErrorContext,
    },

    /// Analysis operations (fragmentation, lifecycle, etc.)
    Analysis {
        analyzer: Arc<str>,
        message: Arc<str>,
        recoverable: bool,
        error_kind: ErrorKind,
        error_context: ErrorContext,
    },

    /// Export operations (JSON, binary, HTML, etc.)
    Export {
        format: Arc<str>,
        message: Arc<str>,
        partial_success: bool,
        error_kind: ErrorKind,
        error_context: ErrorContext,
    },

    /// Configuration and initialization errors
    Configuration {
        component: Arc<str>,
        message: Arc<str>,
        error_kind: ErrorKind,
        error_context: ErrorContext,
    },

    /// System-level errors (IO, threading, etc.)
    System {
        error_type: SystemErrorType,
        message: Arc<str>,
        source_message: Option<Arc<str>>,
        error_kind: ErrorKind,
        error_context: ErrorContext,
    },

    /// Internal errors that should not normally occur
    Internal {
        message: Arc<str>,
        location: Option<Arc<str>>,
        error_kind: ErrorKind,
        error_context: ErrorContext,
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

/// Result type alias for MemScopeError
pub type Result<T> = std::result::Result<T, MemScopeError>;

/// Result type alias for MemScopeError (alternative name)
pub type MemScopeResult<T> = std::result::Result<T, MemScopeError>;

impl MemScopeError {
    /// Unified error creation - external modules only need to provide:
    /// - module: which module the error occurred in (e.g., "variable_registry")
    /// - method: which method/function (e.g., "register_variable")
    /// - message: what went wrong
    ///
    /// Internal classification by module:
    /// - MemoryError: tracker, allocator, variable_registry, memory_tracker, capture
    /// - ExportError: export, render_engine, snapshot
    /// - AnalysisError: ffi_function_resolver, call_stack_normalizer, analysis, lifecycle,
    ///                  rule_engine, pattern_matcher, detector, unsafe_ffi_tracker
    /// - ConfigurationError: config, initialization
    /// - IoError: system errors (io, threading, locking, network, filesystem)
    /// - InternalError: fallback for unknown modules
    pub fn error(module: &str, method: &str, message: impl Into<Arc<str>>) -> Self {
        let kind = match module {
            // Memory tracking modules
            "tracker" | "allocator" | "variable_registry" | "memory_tracker" | "capture"
            | "global_tracking" | "async_tracker" | "unified_tracker" | "core_tracker"
            | "backends" | "platform" | "stack_walker" | "symbol_resolver" | "memory_info"
            | "async_types" => ErrorKind::MemoryError,

            // Export modules
            "export" | "render_engine" | "snapshot" => ErrorKind::ExportError,

            // Analysis modules
            "ffi_function_resolver"
            | "call_stack_normalizer"
            | "analysis"
            | "lifecycle"
            | "lifecycle_analysis"
            | "rule_engine"
            | "pattern_matcher"
            | "detector"
            | "unsafe_ffi_tracker"
            | "classification"
            | "estimation"
            | "metrics"
            | "quality"
            | "circular_reference"
            | "memory_passport_tracker"
            | "borrow_analysis"
            | "async_analysis" => ErrorKind::AnalysisError,

            // Configuration modules
            "config" | "initialization" => ErrorKind::ConfigurationError,

            // System errors
            "system" | "io" | "threading" | "locking" | "network" | "filesystem"
            | "serialization" => ErrorKind::IoError,

            // Fallback
            _ => ErrorKind::InternalError,
        };
        let full_message = format!("{}::{} - {}", module, method, message.into());
        Self::new(kind, full_message)
    }

    /// Create a new error with the specified kind and message
    pub fn new(kind: ErrorKind, message: impl Into<Arc<str>>) -> Self {
        match kind {
            ErrorKind::MemoryError => Self::Memory {
                operation: MemoryOperation::Tracking,
                message: message.into(),
                context: None,
                error_kind: kind,
                error_context: ErrorContext::new(),
            },
            ErrorKind::ValidationError => Self::Memory {
                operation: MemoryOperation::Validation,
                message: message.into(),
                context: None,
                error_kind: kind,
                error_context: ErrorContext::new(),
            },
            ErrorKind::ConfigurationError => Self::Configuration {
                component: "general".into(),
                message: message.into(),
                error_kind: kind,
                error_context: ErrorContext::new(),
            },
            ErrorKind::IoError | ErrorKind::CacheError => Self::System {
                error_type: SystemErrorType::Io,
                message: message.into(),
                source_message: None,
                error_kind: kind,
                error_context: ErrorContext::new(),
            },
            ErrorKind::SymbolResolutionError
            | ErrorKind::StackTraceError
            | ErrorKind::AnalysisError => Self::Analysis {
                analyzer: match kind {
                    ErrorKind::SymbolResolutionError => "symbol_resolution",
                    ErrorKind::StackTraceError => "stack_trace",
                    _ => "general",
                }
                .into(),
                message: message.into(),
                recoverable: true,
                error_kind: kind,
                error_context: ErrorContext::new(),
            },
            ErrorKind::ExportError => Self::Export {
                format: "general".into(),
                message: message.into(),
                partial_success: false,
                error_kind: kind,
                error_context: ErrorContext::new(),
            },
            ErrorKind::InternalError => Self::Internal {
                message: message.into(),
                location: None,
                error_kind: kind,
                error_context: ErrorContext::new(),
            },
        }
    }

    /// Create a new error with the specified kind, severity, and context
    pub fn with_context(
        kind: ErrorKind,
        _severity: ErrorSeverity,
        message: impl Into<Arc<str>>,
        context: impl Into<Arc<str>>,
    ) -> Self {
        let mut error = Self::new(kind, message);
        // Update error context with the provided context information
        match &mut error {
            Self::Memory { error_context, .. }
            | Self::Analysis { error_context, .. }
            | Self::Export { error_context, .. }
            | Self::Configuration { error_context, .. }
            | Self::System { error_context, .. }
            | Self::Internal { error_context, .. } => {
                error_context.extra = Some(context.into().to_string());
            }
        }
        error
    }

    /// Create a memory operation error
    pub fn memory(operation: MemoryOperation, message: impl Into<Arc<str>>) -> Self {
        Self::Memory {
            operation,
            message: message.into(),
            context: None,
            error_kind: ErrorKind::MemoryError,
            error_context: ErrorContext::new(),
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
            error_kind: ErrorKind::MemoryError,
            error_context: ErrorContext::new(),
        }
    }

    /// Create an analysis error
    pub fn analysis(analyzer: impl Into<Arc<str>>, message: impl Into<Arc<str>>) -> Self {
        Self::Analysis {
            analyzer: analyzer.into(),
            message: message.into(),
            recoverable: true,
            error_kind: ErrorKind::AnalysisError,
            error_context: ErrorContext::new(),
        }
    }

    /// Create a non-recoverable analysis error
    pub fn analysis_fatal(analyzer: impl Into<Arc<str>>, message: impl Into<Arc<str>>) -> Self {
        Self::Analysis {
            analyzer: analyzer.into(),
            message: message.into(),
            recoverable: false,
            error_kind: ErrorKind::AnalysisError,
            error_context: ErrorContext::new(),
        }
    }

    /// Create an export error
    pub fn export(format: impl Into<Arc<str>>, message: impl Into<Arc<str>>) -> Self {
        Self::Export {
            format: format.into(),
            message: message.into(),
            partial_success: false,
            error_kind: ErrorKind::ExportError,
            error_context: ErrorContext::new(),
        }
    }

    /// Create an export error with partial success
    pub fn export_partial(format: impl Into<Arc<str>>, message: impl Into<Arc<str>>) -> Self {
        Self::Export {
            format: format.into(),
            message: message.into(),
            partial_success: true,
            error_kind: ErrorKind::ExportError,
            error_context: ErrorContext::new(),
        }
    }

    /// Create an export error with source information
    pub fn export_with_source(
        format: impl Into<Arc<str>>,
        message: impl Into<Arc<str>>,
        source: impl AsRef<str>,
    ) -> Self {
        Self::Export {
            format: format.into(),
            message: format!("{} (source: {})", message.into(), source.as_ref()).into(),
            partial_success: false,
            error_kind: ErrorKind::ExportError,
            error_context: ErrorContext::new(),
        }
    }

    /// Create a configuration error
    pub fn config(component: impl Into<Arc<str>>, message: impl Into<Arc<str>>) -> Self {
        Self::Configuration {
            component: component.into(),
            message: message.into(),
            error_kind: ErrorKind::ConfigurationError,
            error_context: ErrorContext::new(),
        }
    }

    /// Create a system error
    pub fn system(error_type: SystemErrorType, message: impl Into<Arc<str>>) -> Self {
        Self::System {
            error_type,
            message: message.into(),
            source_message: None,
            error_kind: ErrorKind::IoError,
            error_context: ErrorContext::new(),
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
            error_kind: ErrorKind::IoError,
            error_context: ErrorContext::new(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<Arc<str>>) -> Self {
        Self::Internal {
            message: message.into(),
            location: None,
            error_kind: ErrorKind::InternalError,
            error_context: ErrorContext::new(),
        }
    }

    /// Create an internal error with location
    pub fn internal_at(message: impl Into<Arc<str>>, location: impl Into<Arc<str>>) -> Self {
        Self::Internal {
            message: message.into(),
            location: Some(location.into()),
            error_kind: ErrorKind::InternalError,
            error_context: ErrorContext::new(),
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

    /// Get error kind for categorization and statistics
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Memory { error_kind, .. } => *error_kind,
            Self::Analysis { error_kind, .. } => *error_kind,
            Self::Export { error_kind, .. } => *error_kind,
            Self::Configuration { error_kind, .. } => *error_kind,
            Self::System { error_kind, .. } => *error_kind,
            Self::Internal { error_kind, .. } => *error_kind,
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Memory { .. } => ErrorSeverity::Error,
            Self::Analysis {
                recoverable: false, ..
            } => ErrorSeverity::Critical,
            Self::Analysis { .. } => ErrorSeverity::Warning,
            Self::Export {
                partial_success: true,
                ..
            } => ErrorSeverity::Warning,
            Self::Export { .. } => ErrorSeverity::Error,
            Self::Configuration { .. } => ErrorSeverity::Error,
            Self::System { .. } => ErrorSeverity::Error,
            Self::Internal { .. } => ErrorSeverity::Fatal,
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

    /// Get error context information
    pub fn context(&self) -> &ErrorContext {
        match self {
            Self::Memory { error_context, .. } => error_context,
            Self::Analysis { error_context, .. } => error_context,
            Self::Export { error_context, .. } => error_context,
            Self::Configuration { error_context, .. } => error_context,
            Self::System { error_context, .. } => error_context,
            Self::Internal { error_context, .. } => error_context,
        }
    }

    /// Get a detailed error description
    pub fn description(&self) -> String {
        format!("{}", self)
    }
}

/// Error kind classification for error tracking and statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// Memory-related errors
    MemoryError,
    /// Configuration errors
    ConfigurationError,
    /// I/O errors
    IoError,
    /// Symbol resolution errors
    SymbolResolutionError,
    /// Stack trace errors
    StackTraceError,
    /// Cache errors
    CacheError,
    /// Internal errors
    InternalError,
    /// Validation errors
    ValidationError,
    /// Analysis errors
    AnalysisError,
    /// Export errors
    ExportError,
}

impl ErrorKind {
    /// Get the default severity for this error kind
    pub fn default_severity(self) -> ErrorSeverity {
        match self {
            ErrorKind::MemoryError => ErrorSeverity::Error,
            ErrorKind::ConfigurationError => ErrorSeverity::Error,
            ErrorKind::IoError => ErrorSeverity::Error,
            ErrorKind::SymbolResolutionError => ErrorSeverity::Warning,
            ErrorKind::StackTraceError => ErrorSeverity::Warning,
            ErrorKind::CacheError => ErrorSeverity::Warning,
            ErrorKind::InternalError => ErrorSeverity::Fatal,
            ErrorKind::ValidationError => ErrorSeverity::Error,
            ErrorKind::AnalysisError => ErrorSeverity::Warning,
            ErrorKind::ExportError => ErrorSeverity::Error,
        }
    }
}

/// Error severity levels
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum ErrorSeverity {
    /// Non-critical issues that don't prevent operation
    Warning,
    /// Errors that prevent normal operation but may be recoverable
    Error,
    /// Critical errors that require immediate attention
    Critical,
    /// Fatal errors that cannot be recovered from
    Fatal,
}

impl fmt::Display for MemScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Memory {
                operation,
                message,
                context,
                error_kind: _,
                error_context: _,
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
                analyzer,
                message,
                error_kind: _,
                ..
            } => {
                write!(f, "Analysis error in {analyzer}: {message}")
            }
            Self::Export {
                format,
                message,
                partial_success,
                error_kind: _,
                error_context: _,
            } => {
                if *partial_success {
                    write!(f, "Partial export error ({format}): {message}")
                } else {
                    write!(f, "Export error ({format}): {message}")
                }
            }
            Self::Configuration {
                component,
                message,
                error_kind: _,
                error_context: _,
            } => {
                write!(f, "Configuration error in {component}: {message}")
            }
            Self::System {
                error_type,
                message,
                source_message,
                error_kind: _,
                error_context: _,
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
            Self::Internal {
                message,
                location,
                error_kind: _,
                error_context: _,
            } => {
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

impl From<crate::render_engine::export::ExportError> for MemScopeError {
    fn from(err: crate::render_engine::export::ExportError) -> Self {
        Self::error("export", "unknown", err.to_string())
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
    fn execute_recovery(&self, action: &RecoveryAction) -> MemScopeResult<()>;
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

    fn execute_recovery(&self, action: &RecoveryAction) -> MemScopeResult<()> {
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
        assert_eq!(err.severity(), ErrorSeverity::Error);

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
        assert_eq!(err.severity(), ErrorSeverity::Warning);

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
        assert_eq!(fatal_err.severity(), ErrorSeverity::Critical);
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
        assert_eq!(err.severity(), ErrorSeverity::Error);

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
        assert_eq!(partial_err.severity(), ErrorSeverity::Warning);
    }

    #[test]
    fn test_config_error_creation() {
        let err = MemScopeError::config("tracker", "invalid configuration");
        assert!(matches!(err, MemScopeError::Configuration { .. }));
        assert_eq!(err.category(), "config");
        assert!(!err.is_recoverable());
        assert_eq!(err.severity(), ErrorSeverity::Error);
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
        assert_eq!(err.severity(), ErrorSeverity::Error);

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
        assert!(ErrorSeverity::Warning < ErrorSeverity::Error);
        assert!(ErrorSeverity::Error < ErrorSeverity::Critical);
        assert!(ErrorSeverity::Critical < ErrorSeverity::Fatal);
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
        fn test_function() -> MemScopeResult<()> {
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
