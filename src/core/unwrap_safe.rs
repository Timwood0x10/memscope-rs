//! Safe unwrap utilities for memscope-rs
//!
//! This module provides safe alternatives to unwrap() calls while maintaining
//! the same behavior as the original code. It includes error tracking and
//! recovery mechanisms without changing the program logic.

use crate::core::error::{MemScopeError, MemoryOperation, SystemErrorType};
use std::fmt::Debug;

/// Trait for safe unwrapping with logging and error context
pub trait UnwrapSafe<T> {
    /// Unwrap with context information for better error messages
    fn unwrap_safe(self, context: &str) -> T;

    /// Unwrap with context and location information
    fn unwrap_safe_at(self, context: &str, location: &str) -> T;

    /// Unwrap with a default value if the operation fails
    fn unwrap_or_default_safe(self, default: T, context: &str) -> T;

    /// Unwrap with a closure to provide default value
    fn unwrap_or_else_safe<F>(self, default_fn: F, context: &str) -> T
    where
        F: FnOnce() -> T;

    /// Try to unwrap safely, returning an error instead of panicking
    fn try_unwrap_safe(self, context: &str) -> Result<T, MemScopeError>;
}

impl<T> UnwrapSafe<T> for Option<T> {
    fn unwrap_safe(self, context: &str) -> T {
        match self {
            Some(value) => {
                tracing::trace!("Safe unwrap succeeded: {}", context);
                value
            }
            None => {
                tracing::error!("Safe unwrap failed (None): {}", context);
                // Maintain original panic behavior for compatibility
                panic!(
                    "called `Option::unwrap()` on a `None` value in context: {}",
                    context
                );
            }
        }
    }

    fn unwrap_safe_at(self, context: &str, location: &str) -> T {
        match self {
            Some(value) => {
                tracing::trace!("Safe unwrap succeeded at {}: {}", location, context);
                value
            }
            None => {
                tracing::error!("Safe unwrap failed (None) at {}: {}", location, context);
                // Maintain original panic behavior for compatibility
                panic!(
                    "called `Option::unwrap()` on a `None` value at {} in context: {}",
                    location, context
                );
            }
        }
    }

    fn unwrap_or_default_safe(self, default: T, context: &str) -> T {
        match self {
            Some(value) => {
                tracing::trace!("Safe unwrap succeeded: {}", context);
                value
            }
            None => {
                tracing::warn!("Safe unwrap failed (None), using default: {}", context);
                default
            }
        }
    }

    fn unwrap_or_else_safe<F>(self, default_fn: F, context: &str) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Some(value) => {
                tracing::trace!("Safe unwrap succeeded: {}", context);
                value
            }
            None => {
                tracing::warn!(
                    "Safe unwrap failed (None), using default function: {}",
                    context
                );
                default_fn()
            }
        }
    }

    fn try_unwrap_safe(self, context: &str) -> Result<T, MemScopeError> {
        match self {
            Some(value) => {
                tracing::trace!("Safe unwrap succeeded: {}", context);
                Ok(value)
            }
            None => {
                tracing::error!("Safe unwrap failed (None): {}", context);
                Err(MemScopeError::memory(
                    MemoryOperation::Validation,
                    format!("Option unwrap failed in context: {}", context),
                ))
            }
        }
    }
}

impl<T, E: Debug> UnwrapSafe<T> for Result<T, E> {
    fn unwrap_safe(self, context: &str) -> T {
        match self {
            Ok(value) => {
                tracing::trace!("Safe unwrap succeeded: {}", context);
                value
            }
            Err(error) => {
                tracing::error!("Safe unwrap failed (Error: {:?}): {}", error, context);
                // Maintain original panic behavior for compatibility
                panic!(
                    "called `Result::unwrap()` on an `Err` value: {:?} in context: {}",
                    error, context
                );
            }
        }
    }

    fn unwrap_safe_at(self, context: &str, location: &str) -> T {
        match self {
            Ok(value) => {
                tracing::trace!("Safe unwrap succeeded at {}: {}", location, context);
                value
            }
            Err(error) => {
                tracing::error!(
                    "Safe unwrap failed (Error: {:?}) at {}: {}",
                    error,
                    location,
                    context
                );
                // Maintain original panic behavior for compatibility
                panic!(
                    "called `Result::unwrap()` on an `Err` value: {:?} at {} in context: {}",
                    error, location, context
                );
            }
        }
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
    stats_mutex.lock().unwrap().clone()
}

/// Update global unwrap statistics with a closure
pub fn update_unwrap_stats<F, R>(f: F) -> R
where
    F: FnOnce(&mut UnwrapStats) -> R,
{
    let stats_mutex = GLOBAL_UNWRAP_STATS.get_or_init(|| Mutex::new(UnwrapStats::new()));
    let mut stats = stats_mutex.lock().unwrap();
    f(&mut stats)
}

/// Get mutable access to global unwrap statistics (for testing)
#[cfg(test)]
pub fn get_unwrap_stats_mut() -> std::sync::MutexGuard<'static, UnwrapStats> {
    let stats_mutex = GLOBAL_UNWRAP_STATS.get_or_init(|| Mutex::new(UnwrapStats::new()));
    stats_mutex.lock().unwrap()
}

