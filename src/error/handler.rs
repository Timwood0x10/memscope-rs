use super::{ErrorKind, ErrorSeverity, MemScopeError};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Central error handling and reporting system
pub struct ErrorHandler {
    /// Error statistics by kind
    error_counts: HashMap<ErrorKind, AtomicUsize>,
    /// Recent errors for analysis
    recent_errors: Arc<Mutex<Vec<MemScopeError>>>,
    /// Maximum number of recent errors to keep
    max_recent_errors: usize,
    /// Error reporting configuration
    reporter: ErrorReporter,
}

/// Configurable error reporting system
pub struct ErrorReporter {
    /// Whether to log errors to stderr
    log_to_stderr: bool,
    /// Whether to store errors for later analysis
    store_errors: bool,
    /// Minimum severity level to report
    min_severity: ErrorSeverity,
    /// Custom error handlers by severity
    custom_handlers: HashMap<ErrorSeverity, Box<dyn Fn(&MemScopeError) + Send + Sync>>,
}

impl ErrorHandler {
    /// Create new error handler with default configuration
    pub fn new() -> Self {
        Self {
            error_counts: HashMap::new(),
            recent_errors: Arc::new(Mutex::new(Vec::new())),
            max_recent_errors: 100,
            reporter: ErrorReporter::new(),
        }
    }

    /// Create error handler with custom configuration
    pub fn with_config(max_recent_errors: usize, reporter: ErrorReporter) -> Self {
        Self {
            error_counts: HashMap::new(),
            recent_errors: Arc::new(Mutex::new(Vec::new())),
            max_recent_errors,
            reporter,
        }
    }

    /// Handle an error with appropriate response
    pub fn handle_error(&mut self, error: MemScopeError) -> ErrorResponse {
        // Update statistics
        self.update_statistics(&error);

        // Store recent error if configured
        if self.reporter.store_errors {
            self.store_recent_error(error.clone());
        }

        // Report error based on configuration
        let reported = self.reporter.report_error(&error);

        // Determine response based on error severity
        let response = self.determine_response(&error);

        if reported && response.should_log() {
            self.log_response(&error, &response);
        }

        response
    }

    /// Get error statistics by kind
    pub fn get_error_counts(&self) -> HashMap<ErrorKind, usize> {
        self.error_counts
            .iter()
            .map(|(kind, count)| (kind.clone(), count.load(Ordering::Relaxed)))
            .collect()
    }

    /// Get recent errors for analysis
    pub fn get_recent_errors(&self) -> Vec<MemScopeError> {
        self.recent_errors
            .lock()
            .map(|errors| errors.clone())
            .unwrap_or_default()
    }

    /// Clear error statistics and history
    pub fn clear_statistics(&mut self) {
        for count in self.error_counts.values() {
            count.store(0, Ordering::Relaxed);
        }

        if let Ok(mut recent) = self.recent_errors.lock() {
            recent.clear();
        }
    }

    /// Get error frequency analysis
    pub fn get_error_frequency(&self) -> ErrorFrequencyAnalysis {
        let counts = self.get_error_counts();
        let total_errors: usize = counts.values().sum();

        let most_common = counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(kind, count)| (kind.clone(), *count));

        let error_rate = if let Ok(recent) = self.recent_errors.lock() {
            if recent.is_empty() {
                0.0
            } else {
                let oldest_time = recent.first().map(|e| e.context.timestamp);
                let newest_time = recent.last().map(|e| e.context.timestamp);

                if let (Some(oldest), Some(newest)) = (oldest_time, newest_time) {
                    let duration = newest.duration_since(oldest).as_secs_f64();
                    if duration > 0.0 {
                        recent.len() as f64 / duration
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            }
        } else {
            0.0
        };

        ErrorFrequencyAnalysis {
            total_errors,
            most_common_error: most_common,
            errors_per_second: error_rate,
            error_distribution: counts,
        }
    }

    fn update_statistics(&mut self, error: &MemScopeError) {
        let counter = self
            .error_counts
            .entry(error.kind.clone())
            .or_insert_with(|| AtomicUsize::new(0));
        counter.fetch_add(1, Ordering::Relaxed);
    }

    fn store_recent_error(&self, error: MemScopeError) {
        if let Ok(mut recent) = self.recent_errors.lock() {
            recent.push(error);

            // Keep only the most recent errors
            if recent.len() > self.max_recent_errors {
                let excess = recent.len() - self.max_recent_errors;
                recent.drain(0..excess);
            }
        }
    }

    fn determine_response(&self, error: &MemScopeError) -> ErrorResponse {
        match error.severity {
            ErrorSeverity::Warning => ErrorResponse::Continue,
            ErrorSeverity::Error => {
                if self.is_frequent_error(&error.kind) {
                    ErrorResponse::Throttle
                } else {
                    ErrorResponse::Retry
                }
            }
            ErrorSeverity::Critical => ErrorResponse::Fallback,
            ErrorSeverity::Fatal => ErrorResponse::Abort,
        }
    }

    fn is_frequent_error(&self, kind: &ErrorKind) -> bool {
        self.error_counts
            .get(kind)
            .map(|count| count.load(Ordering::Relaxed) > 10)
            .unwrap_or(false)
    }

    fn log_response(&self, error: &MemScopeError, response: &ErrorResponse) {
        if self.reporter.log_to_stderr {
            eprintln!(
                "ErrorHandler: {} -> Response: {:?}",
                error.description(),
                response
            );
        }
    }
}

impl ErrorReporter {
    /// Create new reporter with default settings
    pub fn new() -> Self {
        Self {
            log_to_stderr: true,
            store_errors: true,
            min_severity: ErrorSeverity::Warning,
            custom_handlers: HashMap::new(),
        }
    }

    /// Configure error logging
    pub fn with_logging(mut self, enabled: bool) -> Self {
        self.log_to_stderr = enabled;
        self
    }

    /// Configure error storage
    pub fn with_storage(mut self, enabled: bool) -> Self {
        self.store_errors = enabled;
        self
    }

    /// Set minimum severity level for reporting
    pub fn with_min_severity(mut self, severity: ErrorSeverity) -> Self {
        self.min_severity = severity;
        self
    }

    /// Add custom handler for specific severity level
    pub fn with_custom_handler<F>(mut self, severity: ErrorSeverity, handler: F) -> Self
    where
        F: Fn(&MemScopeError) + Send + Sync + 'static,
    {
        self.custom_handlers.insert(severity, Box::new(handler));
        self
    }

    /// Report error if it meets severity threshold
    pub fn report_error(&self, error: &MemScopeError) -> bool {
        if error.severity < self.min_severity {
            return false;
        }

        // Call custom handler if registered
        if let Some(handler) = self.custom_handlers.get(&error.severity) {
            handler(error);
        }

        // Log to stderr if configured
        if self.log_to_stderr {
            eprintln!("MemScope Error: {}", error);
        }

        true
    }
}

/// Response strategy for handling errors
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorResponse {
    /// Continue operation, error is non-critical
    Continue,
    /// Retry operation with backoff
    Retry,
    /// Throttle operations to reduce error rate
    Throttle,
    /// Fall back to alternative approach
    Fallback,
    /// Abort current operation
    Abort,
}

impl ErrorResponse {
    /// Check if response should be logged
    pub fn should_log(&self) -> bool {
        !matches!(self, ErrorResponse::Continue)
    }

    /// Check if operation should be retried
    pub fn should_retry(&self) -> bool {
        matches!(self, ErrorResponse::Retry)
    }

    /// Check if operation should be aborted
    pub fn should_abort(&self) -> bool {
        matches!(self, ErrorResponse::Abort)
    }
}

/// Analysis of error frequency patterns
#[derive(Debug, Clone)]
pub struct ErrorFrequencyAnalysis {
    /// Total number of errors recorded
    pub total_errors: usize,
    /// Most frequently occurring error type
    pub most_common_error: Option<(ErrorKind, usize)>,
    /// Average errors per second
    pub errors_per_second: f64,
    /// Distribution of errors by kind
    pub error_distribution: HashMap<ErrorKind, usize>,
}

impl ErrorFrequencyAnalysis {
    /// Check if error rate is concerning
    pub fn is_error_rate_high(&self) -> bool {
        self.errors_per_second > 1.0 || self.total_errors > 100
    }

    /// Get the most problematic error type
    pub fn get_primary_concern(&self) -> Option<ErrorKind> {
        self.most_common_error
            .as_ref()
            .map(|(kind, _)| kind.clone())
    }

    /// Generate summary report
    pub fn summary(&self) -> String {
        format!(
            "Error Analysis: {} total errors, {:.2} errors/sec, primary concern: {:?}",
            self.total_errors,
            self.errors_per_second,
            self.get_primary_concern()
        )
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::types::ErrorContext;

    #[test]
    fn test_error_handler_basic() {
        let mut handler = ErrorHandler::new();
        let error = MemScopeError::new(ErrorKind::MemoryError, "test error");

        let response = handler.handle_error(error);
        assert_eq!(response, ErrorResponse::Retry);

        let counts = handler.get_error_counts();
        assert_eq!(counts.get(&ErrorKind::MemoryError), Some(&1));
    }

    #[test]
    fn test_error_frequency_analysis() {
        let mut handler = ErrorHandler::new();

        // Generate multiple errors
        for _ in 0..5 {
            let error = MemScopeError::new(ErrorKind::CacheError, "cache miss");
            handler.handle_error(error);
        }

        for _ in 0..3 {
            let error = MemScopeError::new(ErrorKind::MemoryError, "allocation failed");
            handler.handle_error(error);
        }

        let analysis = handler.get_error_frequency();
        assert_eq!(analysis.total_errors, 8);
        assert_eq!(analysis.get_primary_concern(), Some(ErrorKind::CacheError));
    }

    #[test]
    fn test_error_reporter_configuration() {
        let reporter = ErrorReporter::new()
            .with_logging(false)
            .with_storage(true)
            .with_min_severity(ErrorSeverity::Error);

        // Warning should not be reported
        let warning = MemScopeError::with_context(
            ErrorKind::ValidationError,
            ErrorSeverity::Warning,
            "test_warning",
            "test",
        );
        assert!(!reporter.report_error(&warning));

        // Error should be reported
        let error = MemScopeError::with_context(
            ErrorKind::ValidationError,
            ErrorSeverity::Error,
            "test_error",
            "test",
        );
        assert!(reporter.report_error(&error));
    }

    #[test]
    fn test_error_response_strategies() {
        let handler = ErrorHandler::new();

        let warning = MemScopeError::with_context(
            ErrorKind::ValidationError,
            ErrorSeverity::Warning,
            "test",
            "test",
        );
        let response = handler.determine_response(&warning);
        assert_eq!(response, ErrorResponse::Continue);
        assert!(!response.should_abort());

        let fatal = MemScopeError::with_context(
            ErrorKind::InternalError,
            ErrorSeverity::Fatal,
            "test",
            "test",
        );
        let response = handler.determine_response(&fatal);
        assert_eq!(response, ErrorResponse::Abort);
        assert!(response.should_abort());
    }
}
