//! Safe unwrap utilities for memscope-rs
//!
//! This module provides safe, panic-free alternatives to unwrap() calls.
//! It includes comprehensive error tracking and recovery mechanisms.

use crate::core::error::{MemScopeError, MemoryOperation, SystemErrorType};
use crate::core::safe_operations::SafeLock;
use std::fmt::{self, Debug, Display};
use std::backtrace::Backtrace;
use std::error::Error as StdError;

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
            Self::NoneValue { context, location, .. } => {
                if let Some(loc) = location {
                    write!(f, "Attempt to unwrap None at {}: {}", loc, context)
                } else {
                    write!(f, "Attempt to unwrap None: {}", context)
                }
            }
            Self::ResultError { source, context, location, .. } => {
                if let Some(loc) = location {
                    write!(f, "Unwrap failed at {} ({}): {}", loc, context, source)
                } else {
                    write!(f, "Unwrap failed ({}): {}", context, source)
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
            Self::NoneValue { backtrace, .. } |
            Self::ResultError { backtrace, .. } => backtrace,
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
                UnwrapError::NoneValue { location: loc, .. } |
                UnwrapError::ResultError { location: loc, .. } => {
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
                format!("Failed to unwrap value: {}", e)
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
                format!("Failed to unwrap value: {}", e)
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
                tracing::error!("Result unwrap failed: {}", error);
                Err(error)
            }
        }
    }

    fn try_unwrap_at(self, context: &'static str, location: &'static str) -> Result<T, UnwrapError> {
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
                tracing::warn!(
                    "Safe unwrap failed (Error: {:?}), using default: {}",
                    error,
                    context
                );
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
                    "Safe unwrap failed (Error: {:?}), using default function: {}",
                    error,
                    context
                );
                default_fn()
            }
        }
    }

    fn try_unwrap_safe(self, context: &str) -> Result<T, MemScopeError> {
        match self {
            Ok(value) => {
                tracing::trace!("Safe unwrap succeeded: {}", context);
                Ok(value)
            }
            Err(error) => {
                tracing::error!("Safe unwrap failed (Error: {:?}): {}", error, context);
                Err(MemScopeError::system(
                    SystemErrorType::Io,
                    format!(
                        "Result unwrap failed in context: {} - error: {:?}",
                        context, error
                    ),
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
