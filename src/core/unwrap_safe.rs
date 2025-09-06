//! Safe unwrap utilities for memscope-rs
//!
//! This module provides safe, panic-free alternatives to unwrap() calls.
//! It includes comprehensive error tracking and recovery mechanisms.

use crate::core::error::{MemScopeError, MemoryOperation, SystemErrorType};
use crate::core::safe_operations::SafeLock;
use std::backtrace::Backtrace;
use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};

/// Custom error type for unwrap operations
#[derive(Debug)]
pub enum UnwrapError {
    NoneValue {
        context: &'static str,
        location: Option<&'static str>,
        backtrace: Backtrace,
    },
    ResultError {
        source: Box<dyn StdError + Send + Sync>,
        context: &'static str,
        location: Option<&'static str>,
        backtrace: Backtrace,
    },
}

impl Display for UnwrapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoneValue {
                context, location, ..
            } => {
                if let Some(loc) = location {
                    write!(f, "Attempt to unwrap None at {loc} ({context})")
                } else {
                    write!(f, "Attempt to unwrap None ({context})")
                }
            }
            Self::ResultError {
                source,
                context,
                location,
                ..
            } => {
                if let Some(loc) = location {
                    write!(f, "Unwrap failed at {loc} ({context}): {source}")
                } else {
                    write!(f, "Unwrap failed ({context}): {source}")
                }
            }
        }
    }
}

impl StdError for UnwrapError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::NoneValue { .. } => None,
            Self::ResultError { source, .. } => Some(&**source),
        }
    }
}

impl UnwrapError {
    /// Get the backtrace for this error
    pub fn backtrace(&self) -> &Backtrace {
        match self {
            Self::NoneValue { backtrace, .. } | Self::ResultError { backtrace, .. } => backtrace,
        }
    }
}

/// Extension trait for safe, panic-free unwrapping
pub trait UnwrapSafe<T> {
    /// Try to unwrap a value, returning a Result
    fn try_unwrap(self, context: &'static str) -> Result<T, UnwrapError>;

    /// Try to unwrap with location information
    fn try_unwrap_at(self, context: &'static str, location: &'static str) -> Result<T, UnwrapError>
    where
        Self: Sized,
    {
        self.try_unwrap(context).map_err(|mut e| {
            match &mut e {
                UnwrapError::NoneValue { location: loc, .. }
                | UnwrapError::ResultError { location: loc, .. } => {
                    *loc = Some(location);
                }
            }
            e
        })
    }

    /// Unwrap or return a default value
    fn unwrap_or_default_safe(self, default: T, context: &'static str) -> T
    where
        Self: Sized,
    {
        self.try_unwrap(context).unwrap_or_else(|e| {
            tracing::warn!("{}", e);
            default
        })
    }

    /// Unwrap or compute a default value
    fn unwrap_or_else_safe<F>(self, default_fn: F, context: &'static str) -> T
    where
        Self: Sized,
        F: FnOnce() -> T,
    {
        self.try_unwrap(context).unwrap_or_else(|e| {
            tracing::warn!("{}", e);
            default_fn()
        })
    }

    /// Backwards compatibility method (deprecated)
    #[deprecated(note = "Use try_unwrap() instead")]
    fn try_unwrap_safe(self, context: &'static str) -> Result<T, MemScopeError>
    where
        Self: Sized,
    {
        self.try_unwrap(context).map_err(|e| {
            MemScopeError::memory(
                MemoryOperation::Allocation,
                format!("Failed to unwrap value: {e}"),
            )
        })
    }

    /// Unwrap with context information (deprecated)
    /// Unwrap or abort the process
    ///
    /// # Safety
    /// This method will abort the process on error. Only use when the program
    /// cannot continue without this value.
    #[deprecated(note = "Use try_unwrap() instead")]
    fn unwrap_safe(self, context: &'static str) -> T
    where
        Self: Sized,
    {
        self.try_unwrap(context).unwrap_or_else(|e| {
            tracing::error!("Fatal error: {}\nBacktrace:\n{:?}", e, e.backtrace());
            std::process::abort();
        })
    }

    /// Unwrap with context and location information (deprecated)
    /// Unwrap or abort the process with location information
    ///
    /// # Safety
    /// This method will abort the process on error. Only use when the program
    /// cannot continue without this value.
    #[deprecated(note = "Use try_unwrap_at() instead")]
    fn unwrap_safe_at(self, context: &'static str, location: &'static str) -> T
    where
        Self: Sized,
    {
        self.try_unwrap_at(context, location).unwrap_or_else(|e| {
            let backtrace = e.backtrace();
            tracing::error!(
                "Fatal error at {}: {}\nBacktrace:\n{:?}",
                location,
                e,
                backtrace
            );
            std::process::abort();
        })
    }
}

impl<T> UnwrapSafe<T> for Option<T> {
    fn try_unwrap(self, context: &'static str) -> Result<T, UnwrapError> {
        match self {
            Some(value) => {
                tracing::trace!("Unwrap succeeded: {}", context);
                Ok(value)
            }
            None => {
                let error = UnwrapError::NoneValue {
                    context,
                    location: None,
                    backtrace: Backtrace::capture(),
                };
                tracing::error!("Unwrap failed: {}", error);
                Err(error)
            }
        }
    }

    fn unwrap_or_else_safe<F>(self, default_fn: F, context: &'static str) -> T
    where
        F: FnOnce() -> T,
    {
        self.try_unwrap(context).unwrap_or_else(|e| {
            tracing::warn!("Using default value: {}", e);
            default_fn()
        })
    }

    fn try_unwrap_safe(self, context: &'static str) -> Result<T, MemScopeError> {
        self.try_unwrap(context).map_err(|e| {
            MemScopeError::memory(
                MemoryOperation::Allocation,
                format!("Failed to unwrap value: {e:?}"),
            )
        })
    }
}

impl<T, E: StdError + Send + Sync + 'static> UnwrapSafe<T> for Result<T, E> {
    fn try_unwrap(self, context: &'static str) -> Result<T, UnwrapError> {
        match self {
            Ok(value) => {
                tracing::trace!("Result unwrap succeeded: {}", context);
                Ok(value)
            }
            Err(error) => {
                let error = UnwrapError::ResultError {
                    source: Box::new(error),
                    context,
                    location: None,
                    backtrace: Backtrace::capture(),
                };
                tracing::error!("Result unwrap failed: {error:?}");
                Err(error)
            }
        }
    }

    fn try_unwrap_at(
        self,
        context: &'static str,
        location: &'static str,
    ) -> Result<T, UnwrapError> {
        self.try_unwrap(context).map_err(|mut e| {
            if let UnwrapError::ResultError { location: loc, .. } = &mut e {
                *loc = Some(location);
            }
            e
        })
    }

    fn unwrap_or_default_safe(self, default: T, context: &str) -> T {
        match self {
            Ok(value) => {
                tracing::trace!("Safe unwrap succeeded: {}", context);
                value
            }
            Err(error) => {
                tracing::warn!("Safe unwrap failed (Error: {error:?}), using default: {context}",);
                default
            }
        }
    }

    fn unwrap_or_else_safe<F>(self, default_fn: F, context: &str) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Ok(value) => {
                tracing::trace!("Safe unwrap succeeded: {}", context);
                value
            }
            Err(error) => {
                tracing::warn!(
                    "Safe unwrap failed (Error: {error:?}), using default function: {context}",
                );
                default_fn()
            }
        }
    }

    fn try_unwrap_safe(self, context: &str) -> Result<T, MemScopeError> {
        match self {
            Ok(value) => {
                tracing::trace!("Safe unwrap succeeded: {context}");
                Ok(value)
            }
            Err(error) => {
                tracing::error!("Safe unwrap failed (Error: {error:?}): {context}");
                Err(MemScopeError::system(
                    SystemErrorType::Io,
                    format!("Result unwrap failed in context: {context} - error: {error:?}",),
                ))
            }
        }
    }
}

/// Convenience macro for safe unwrapping with automatic context
#[macro_export]
macro_rules! unwrap_safe {
    ($expr:expr) => {
        $expr.unwrap_safe(&format!("{}:{}", file!(), line!()))
    };
    ($expr:expr, $context:expr) => {
        $expr.unwrap_safe($context)
    };
}

/// Convenience macro for safe unwrapping with location
#[macro_export]
macro_rules! unwrap_safe_at {
    ($expr:expr, $context:expr) => {
        $expr.unwrap_safe_at($context, &format!("{}:{}", file!(), line!()))
    };
}

/// Convenience macro for safe unwrapping with default value
#[macro_export]
macro_rules! unwrap_or_default_safe {
    ($expr:expr, $default:expr) => {
        $expr.unwrap_or_default_safe($default, &format!("{}:{}", file!(), line!()))
    };
    ($expr:expr, $default:expr, $context:expr) => {
        $expr.unwrap_or_default_safe($default, $context)
    };
}

/// Convenience macro for safe unwrapping with default function
#[macro_export]
macro_rules! unwrap_or_else_safe {
    ($expr:expr, $default_fn:expr) => {
        $expr.unwrap_or_else_safe($default_fn, &format!("{}:{}", file!(), line!()))
    };
    ($expr:expr, $default_fn:expr, $context:expr) => {
        $expr.unwrap_or_else_safe($default_fn, $context)
    };
}

/// Convenience macro for trying to unwrap safely
#[macro_export]
macro_rules! try_unwrap_safe {
    ($expr:expr) => {
        $expr.try_unwrap_safe(&format!("{}:{}", file!(), line!()))
    };
    ($expr:expr, $context:expr) => {
        $expr.try_unwrap_safe($context)
    };
}

/// Statistics for unwrap operations
#[derive(Debug, Clone, Default)]
pub struct UnwrapStats {
    pub successful_unwraps: u64,
    pub failed_unwraps: u64,
    pub default_value_uses: u64,
    pub panic_preservations: u64,
}

impl UnwrapStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_success(&mut self) {
        self.successful_unwraps += 1;
    }

    pub fn record_failure(&mut self) {
        self.failed_unwraps += 1;
    }

    pub fn record_default_use(&mut self) {
        self.default_value_uses += 1;
    }

    pub fn record_panic_preservation(&mut self) {
        self.panic_preservations += 1;
    }

    pub fn total_operations(&self) -> u64 {
        self.successful_unwraps
            + self.failed_unwraps
            + self.default_value_uses
            + self.panic_preservations
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.total_operations();
        if total == 0 {
            0.0
        } else {
            self.successful_unwraps as f64 / total as f64
        }
    }
}

use std::sync::{Mutex, OnceLock};

/// Global unwrap statistics
static GLOBAL_UNWRAP_STATS: OnceLock<Mutex<UnwrapStats>> = OnceLock::new();

/// Get global unwrap statistics (read-only snapshot)
pub fn get_unwrap_stats() -> UnwrapStats {
    let stats_mutex = GLOBAL_UNWRAP_STATS.get_or_init(|| Mutex::new(UnwrapStats::new()));
    match stats_mutex.safe_lock() {
        Ok(stats) => stats.clone(),
        Err(_) => UnwrapStats::new(),
    }
}

/// Update global unwrap statistics with a closure
pub fn update_unwrap_stats<F, R>(f: F) -> R
where
    F: FnOnce(&mut UnwrapStats) -> R,
    R: Default,
{
    let stats_mutex = GLOBAL_UNWRAP_STATS.get_or_init(|| Mutex::new(UnwrapStats::new()));
    match stats_mutex.try_lock() {
        Ok(mut stats) => f(&mut stats),
        Err(_) => R::default(),
    }
}

/// Get mutable access to global unwrap statistics (for testing)
#[cfg(test)]
pub fn get_unwrap_stats_mut() -> Option<std::sync::MutexGuard<'static, UnwrapStats>> {
    let stats_mutex = GLOBAL_UNWRAP_STATS.get_or_init(|| Mutex::new(UnwrapStats::new()));
    stats_mutex.try_lock().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unwrap_error_display() {
        let none_error = UnwrapError::NoneValue {
            context: "test context",
            location: Some("test.rs:42"),
            backtrace: Backtrace::capture(),
        };

        let display_str = format!("{none_error:?}");
        assert!(display_str.contains("test context"));
        assert!(display_str.contains("test.rs:42"));

        let none_error_no_location = UnwrapError::NoneValue {
            context: "test context",
            location: None,
            backtrace: Backtrace::capture(),
        };

        let display_str = format!("{none_error_no_location:?}");
        assert!(display_str.contains("test context"));
        assert!(!display_str.contains("test.rs:42"));
    }

    #[test]
    fn test_unwrap_error_result_display() {
        let source_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let result_error = UnwrapError::ResultError {
            source: Box::new(source_error),
            context: "file operation",
            location: Some("main.rs:10"),
            backtrace: Backtrace::capture(),
        };

        let display_str = format!("{result_error}");
        assert!(display_str.contains("file operation"));
        assert!(display_str.contains("main.rs:10"));
        assert!(display_str.contains("file not found"));
    }

    #[test]
    fn test_option_try_unwrap_success() {
        let option = Some(42);
        let result = option.try_unwrap("test success");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_option_try_unwrap_failure() {
        let option: Option<i32> = None;
        let result = option.try_unwrap("test failure");
        assert!(result.is_err());

        if let Err(UnwrapError::NoneValue { context, .. }) = result {
            assert_eq!(context, "test failure");
        } else {
            panic!("Expected NoneValue error");
        }
    }

    #[test]
    fn test_option_try_unwrap_at() {
        let option: Option<i32> = None;
        let result = option.try_unwrap_at("test context", "test.rs:100");
        assert!(result.is_err());

        if let Err(UnwrapError::NoneValue {
            context, location, ..
        }) = result
        {
            assert_eq!(context, "test context");
            assert_eq!(location, Some("test.rs:100"));
        } else {
            panic!("Expected NoneValue error with location");
        }
    }

    #[test]
    fn test_option_unwrap_or_default_safe() {
        let some_option = Some(42);
        let result = some_option.unwrap_or_default_safe(99, "test default");
        assert_eq!(result, 42);

        let none_option: Option<i32> = None;
        let result = none_option.unwrap_or_default_safe(99, "test default");
        assert_eq!(result, 99);
    }

    #[test]
    fn test_option_unwrap_or_else_safe() {
        let some_option = Some(42);
        let result = some_option.unwrap_or_else_safe(|| 99, "test else");
        assert_eq!(result, 42);

        let none_option: Option<i32> = None;
        let result = none_option.unwrap_or_else_safe(|| 99, "test else");
        assert_eq!(result, 99);
    }

    #[test]
    fn test_result_try_unwrap_success() {
        let result: Result<i32, std::io::Error> = Ok(42);
        let unwrap_result = result.try_unwrap("test success");
        assert!(unwrap_result.is_ok());
        assert_eq!(unwrap_result.unwrap(), 42);
    }

    #[test]
    fn test_result_try_unwrap_failure() {
        let result: Result<i32, std::io::Error> = Err(std::io::Error::other("test error"));
        let unwrap_result = result.try_unwrap("test failure");
        assert!(unwrap_result.is_err());

        if let Err(UnwrapError::ResultError { context, .. }) = unwrap_result {
            assert_eq!(context, "test failure");
        } else {
            panic!("Expected ResultError");
        }
    }

    #[test]
    fn test_result_try_unwrap_at() {
        let result: Result<i32, std::io::Error> = Err(std::io::Error::other("test error"));
        let unwrap_result = result.try_unwrap_at("test context", "main.rs:50");
        assert!(unwrap_result.is_err());

        if let Err(UnwrapError::ResultError {
            context, location, ..
        }) = unwrap_result
        {
            assert_eq!(context, "test context");
            assert_eq!(location, Some("main.rs:50"));
        } else {
            panic!("Expected ResultError with location");
        }
    }

    #[test]
    fn test_result_unwrap_or_default_safe() {
        let ok_result: Result<i32, std::io::Error> = Ok(42);
        let result = ok_result.unwrap_or_default_safe(99, "test default");
        assert_eq!(result, 42);

        let err_result: Result<i32, std::io::Error> =
            Err(std::io::Error::other("error"));
        let result = err_result.unwrap_or_default_safe(99, "test default");
        assert_eq!(result, 99);
    }

    #[test]
    fn test_result_unwrap_or_else_safe() {
        let ok_result: Result<i32, std::io::Error> = Ok(42);
        let result = ok_result.unwrap_or_else_safe(|| 99, "test else");
        assert_eq!(result, 42);

        let err_result: Result<i32, std::io::Error> =
            Err(std::io::Error::other("error"));
        let result = err_result.unwrap_or_else_safe(|| 99, "test else");
        assert_eq!(result, 99);
    }

    #[test]
    fn test_unwrap_stats_creation() {
        let stats = UnwrapStats::new();
        assert_eq!(stats.successful_unwraps, 0);
        assert_eq!(stats.failed_unwraps, 0);
        assert_eq!(stats.default_value_uses, 0);
        assert_eq!(stats.panic_preservations, 0);
    }

    #[test]
    fn test_unwrap_stats_record_operations() {
        let mut stats = UnwrapStats::new();

        stats.record_success();
        stats.record_failure();
        stats.record_default_use();
        stats.record_panic_preservation();

        assert_eq!(stats.successful_unwraps, 1);
        assert_eq!(stats.failed_unwraps, 1);
        assert_eq!(stats.default_value_uses, 1);
        assert_eq!(stats.panic_preservations, 1);
    }

    #[test]
    fn test_unwrap_stats_total_operations() {
        let mut stats = UnwrapStats::new();

        stats.record_success();
        stats.record_success();
        stats.record_failure();
        stats.record_default_use();

        assert_eq!(stats.total_operations(), 4);
    }

    #[test]
    fn test_unwrap_stats_success_rate() {
        let mut stats = UnwrapStats::new();

        // Test with no operations
        assert_eq!(stats.success_rate(), 0.0);

        // Test with some operations
        stats.record_success();
        stats.record_success();
        stats.record_success();
        stats.record_failure();

        let expected_rate = 3.0 / 4.0;
        assert!((stats.success_rate() - expected_rate).abs() < f64::EPSILON);
    }

    #[test]
    fn test_global_unwrap_stats() {
        let stats = get_unwrap_stats();
        assert!(stats.total_operations() >= 0);
    }

    #[test]
    fn test_update_unwrap_stats() {
        let result = update_unwrap_stats(|stats| {
            stats.record_success();
            42
        });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_unwrap_error_backtrace() {
        let error = UnwrapError::NoneValue {
            context: "test",
            location: None,
            backtrace: Backtrace::capture(),
        };

        let _backtrace = error.backtrace();
        // Just ensure we can access the backtrace without panic
    }

    #[test]
    fn test_unwrap_error_source() {
        let none_error = UnwrapError::NoneValue {
            context: "test",
            location: None,
            backtrace: Backtrace::capture(),
        };
        assert!(none_error.source().is_none());

        let source_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let result_error = UnwrapError::ResultError {
            source: Box::new(source_error),
            context: "test",
            location: None,
            backtrace: Backtrace::capture(),
        };
        assert!(result_error.source().is_some());
    }

    #[test]
    fn test_option_try_unwrap_safe_deprecated() {
        #[allow(deprecated)]
        {
            let some_option = Some(42);
            let result = some_option.try_unwrap_safe("test");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 42);

            let none_option: Option<i32> = None;
            let result = none_option.try_unwrap_safe("test");
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_result_try_unwrap_safe_deprecated() {
        #[allow(deprecated)]
        {
            let ok_result: Result<i32, std::io::Error> = Ok(42);
            let result = ok_result.try_unwrap_safe("test");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 42);

            let err_result: Result<i32, std::io::Error> = Err(std::io::Error::other("error"));
            let result = err_result.try_unwrap_safe("test");
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_unwrap_stats_default() {
        let stats = UnwrapStats::default();
        assert_eq!(stats.successful_unwraps, 0);
        assert_eq!(stats.failed_unwraps, 0);
        assert_eq!(stats.default_value_uses, 0);
        assert_eq!(stats.panic_preservations, 0);
    }
}
