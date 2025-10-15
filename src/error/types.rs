use std::error::Error;
use std::fmt;

/// Primary error type for all MemScope operations
#[derive(Debug)]
pub struct MemScopeError {
    /// Error classification
    pub kind: ErrorKind,
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Additional context information
    pub context: ErrorContext,
    /// Underlying cause if any
    pub source: Option<Box<dyn Error + Send + Sync>>,
}

impl Clone for MemScopeError {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind.clone(),
            severity: self.severity.clone(),
            context: self.context.clone(),
            source: None, // Cannot clone trait objects, so we lose the source
        }
    }
}

/// Classification of error types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// Memory allocation or tracking failures
    MemoryError,
    /// Configuration or parameter issues
    ConfigurationError,
    /// I/O related errors
    IoError,
    /// Threading or concurrency issues
    ConcurrencyError,
    /// Symbol resolution failures
    SymbolResolutionError,
    /// Stack trace capture failures
    StackTraceError,
    /// Smart pointer tracking issues
    SmartPointerError,
    /// Cache operation failures
    CacheError,
    /// Invalid input or state
    ValidationError,
    /// Internal logic errors
    InternalError,
}

/// Error severity levels for prioritization
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorSeverity {
    /// Low impact, operation can continue
    Warning,
    /// Moderate impact, degraded functionality
    Error,
    /// High impact, operation must stop
    Critical,
    /// System-level failure
    Fatal,
}

/// Additional context for error diagnosis
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Operation being performed when error occurred
    pub operation: String,
    /// Module or component where error originated
    pub component: String,
    /// Additional metadata for debugging
    pub metadata: std::collections::HashMap<String, String>,
    /// Timestamp when error occurred
    pub timestamp: std::time::Instant,
}

impl MemScopeError {
    /// Create new error with minimal information
    pub fn new(kind: ErrorKind, message: &str) -> Self {
        Self {
            kind,
            severity: ErrorSeverity::Error,
            context: ErrorContext {
                operation: message.to_string(),
                component: "unknown".to_string(),
                metadata: std::collections::HashMap::new(),
                timestamp: std::time::Instant::now(),
            },
            source: None,
        }
    }

    /// Create error with full context
    pub fn with_context(
        kind: ErrorKind,
        severity: ErrorSeverity,
        operation: &str,
        component: &str,
    ) -> Self {
        Self {
            kind,
            severity,
            context: ErrorContext {
                operation: operation.to_string(),
                component: component.to_string(),
                metadata: std::collections::HashMap::new(),
                timestamp: std::time::Instant::now(),
            },
            source: None,
        }
    }

    /// Chain with underlying error
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    /// Add metadata for debugging
    pub fn with_metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.context.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(self.severity, ErrorSeverity::Warning | ErrorSeverity::Error)
    }

    /// Check if error requires immediate attention
    pub fn is_critical(&self) -> bool {
        matches!(
            self.severity,
            ErrorSeverity::Critical | ErrorSeverity::Fatal
        )
    }

    /// Get human-readable error description
    pub fn description(&self) -> String {
        format!(
            "[{}:{}] {} in {} ({})",
            self.severity_str(),
            self.kind_str(),
            self.context.operation,
            self.context.component,
            self.elapsed_time_str()
        )
    }

    /// Get error age since occurrence
    pub fn age(&self) -> std::time::Duration {
        self.context.timestamp.elapsed()
    }

    fn severity_str(&self) -> &'static str {
        match self.severity {
            ErrorSeverity::Warning => "WARN",
            ErrorSeverity::Error => "ERROR",
            ErrorSeverity::Critical => "CRITICAL",
            ErrorSeverity::Fatal => "FATAL",
        }
    }

    fn kind_str(&self) -> &'static str {
        match self.kind {
            ErrorKind::MemoryError => "MEMORY",
            ErrorKind::ConfigurationError => "CONFIG",
            ErrorKind::IoError => "IO",
            ErrorKind::ConcurrencyError => "CONCURRENCY",
            ErrorKind::SymbolResolutionError => "SYMBOL",
            ErrorKind::StackTraceError => "STACKTRACE",
            ErrorKind::SmartPointerError => "SMARTPTR",
            ErrorKind::CacheError => "CACHE",
            ErrorKind::ValidationError => "VALIDATION",
            ErrorKind::InternalError => "INTERNAL",
        }
    }

    fn elapsed_time_str(&self) -> String {
        let elapsed = self.age();
        if elapsed.as_secs() > 0 {
            format!("{}s ago", elapsed.as_secs())
        } else {
            format!("{}ms ago", elapsed.as_millis())
        }
    }
}

impl fmt::Display for MemScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())?;

        if !self.context.metadata.is_empty() {
            write!(f, " [")?;
            for (i, (key, value)) in self.context.metadata.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}={}", key, value)?;
            }
            write!(f, "]")?;
        }

        Ok(())
    }
}

impl Error for MemScopeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn Error + 'static))
    }
}

/// Convenience type alias for Results
pub type MemScopeResult<T> = Result<T, MemScopeError>;

/// Macro for creating errors with automatic context
#[macro_export]
macro_rules! memscope_error {
    ($kind:expr, $operation:expr) => {
        $crate::error::MemScopeError::new($kind, $operation)
    };

    ($kind:expr, $severity:expr, $operation:expr, $component:expr) => {
        $crate::error::MemScopeError::with_context($kind, $severity, $operation, $component)
    };
}

/// Macro for creating results with error context
#[macro_export]
macro_rules! memscope_bail {
    ($kind:expr, $operation:expr) => {
        return Err($crate::memscope_error!($kind, $operation))
    };

    ($kind:expr, $severity:expr, $operation:expr, $component:expr) => {
        return Err($crate::memscope_error!(
            $kind, $severity, $operation, $component
        ))
    };
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            operation: "unknown_operation".to_string(),
            component: "unknown_component".to_string(),
            metadata: std::collections::HashMap::new(),
            timestamp: std::time::Instant::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = MemScopeError::new(ErrorKind::MemoryError, "allocation failed");

        assert_eq!(error.kind, ErrorKind::MemoryError);
        assert_eq!(error.severity, ErrorSeverity::Error);
        assert!(error.is_recoverable());
        assert!(!error.is_critical());
    }

    #[test]
    fn test_error_with_context() {
        let error = MemScopeError::with_context(
            ErrorKind::ConfigurationError,
            ErrorSeverity::Critical,
            "invalid_config",
            "memory_tracker",
        );

        assert_eq!(error.severity, ErrorSeverity::Critical);
        assert!(!error.is_recoverable());
        assert!(error.is_critical());
        assert!(error.description().contains("invalid_config"));
    }

    #[test]
    fn test_error_chaining() {
        use std::io;

        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error =
            MemScopeError::new(ErrorKind::IoError, "config read failed").with_source(io_error);

        assert!(error.source().is_some());
    }

    #[test]
    fn test_error_metadata() {
        let error = MemScopeError::new(ErrorKind::CacheError, "cache miss")
            .with_metadata("cache_size", "1024")
            .with_metadata("hit_ratio", "0.85");

        assert_eq!(error.context.metadata.len(), 2);
        assert_eq!(
            error.context.metadata.get("cache_size"),
            Some(&"1024".to_string())
        );
    }

    #[test]
    fn test_error_display() {
        let error = MemScopeError::with_context(
            ErrorKind::SymbolResolutionError,
            ErrorSeverity::Warning,
            "symbol_lookup",
            "stack_tracer",
        )
        .with_metadata("address", "0x12345678");

        let display_str = format!("{}", error);
        assert!(display_str.contains("WARN"));
        assert!(display_str.contains("SYMBOL"));
        assert!(display_str.contains("symbol_lookup"));
        assert!(display_str.contains("stack_tracer"));
        assert!(display_str.contains("address=0x12345678"));
    }

    #[test]
    fn test_macro_usage() {
        let error = memscope_error!(ErrorKind::ValidationError, "invalid input");
        assert_eq!(error.kind, ErrorKind::ValidationError);

        let error = memscope_error!(
            ErrorKind::InternalError,
            ErrorSeverity::Fatal,
            "system_failure",
            "core"
        );
        assert_eq!(error.severity, ErrorSeverity::Fatal);
    }
}
