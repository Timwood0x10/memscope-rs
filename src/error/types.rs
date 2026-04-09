//! Error type definitions for MemScope
//!
//! Re-exports the main MemScopeError from core/error module
//! for unified error handling across the project.

pub use crate::core::error::MemScopeError;
pub use crate::core::error::Result as MemScopeResult;

/// Additional error utilities and helpers
pub struct ErrorUtils;

impl ErrorUtils {
    /// Create a memory error with operation context
    pub fn memory_error(operation: &str) -> MemScopeError {
        crate::core::error::MemScopeError::memory(
            crate::core::error::MemoryOperation::Tracking,
            operation,
        )
    }

    /// Create a configuration error
    pub fn config_error(component: &str, message: &str) -> MemScopeError {
        crate::core::error::MemScopeError::config(component, message)
    }

    /// Create a system error
    pub fn system_error(message: &str) -> MemScopeError {
        crate::core::error::MemScopeError::system(crate::core::error::SystemErrorType::Io, message)
    }

    /// Create an analysis error
    pub fn analysis_error(analyzer: &str, message: &str) -> MemScopeError {
        crate::core::error::MemScopeError::analysis(analyzer, message)
    }

    /// Create an export error
    pub fn export_error(format: &str, message: &str) -> MemScopeError {
        crate::core::error::MemScopeError::export(format, message)
    }

    /// Create an internal error
    pub fn internal_error(message: &str) -> MemScopeError {
        crate::core::error::MemScopeError::internal(message)
    }
}
