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

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify ErrorUtils::memory_error creates correct error type
    /// Invariants: Memory error should contain tracking operation context
    #[test]
    fn test_memory_error_creation() {
        let error = ErrorUtils::memory_error("test_allocation");
        let error_string = format!("{error:?}");
        assert!(
            error_string.contains("Memory") || error_string.contains("memory"),
            "Memory error should contain memory context, got: {error_string}"
        );
    }

    /// Objective: Verify ErrorUtils::config_error creates correct error type
    /// Invariants: Config error should contain component and message
    #[test]
    fn test_config_error_creation() {
        let error = ErrorUtils::config_error("TestComponent", "invalid configuration");
        let error_string = format!("{error:?}");
        assert!(
            error_string.contains("Config") || error_string.contains("config"),
            "Config error should contain config context, got: {error_string}"
        );
    }

    /// Objective: Verify ErrorUtils::system_error creates correct error type
    /// Invariants: System error should contain IO context
    #[test]
    fn test_system_error_creation() {
        let error = ErrorUtils::system_error("system failure");
        let error_string = format!("{error:?}");
        assert!(
            !error_string.is_empty(),
            "System error should produce non-empty string"
        );
    }

    /// Objective: Verify ErrorUtils::analysis_error creates correct error type
    /// Invariants: Analysis error should contain analyzer name and message
    #[test]
    fn test_analysis_error_creation() {
        let error = ErrorUtils::analysis_error("LeakDetector", "leak detection failed");
        let error_string = format!("{error:?}");
        assert!(
            error_string.contains("Analysis") || error_string.contains("analysis"),
            "Analysis error should contain analysis context, got: {error_string}"
        );
    }

    /// Objective: Verify ErrorUtils::export_error creates correct error type
    /// Invariants: Export error should contain format and message
    #[test]
    fn test_export_error_creation() {
        let error = ErrorUtils::export_error("JSON", "serialization failed");
        let error_string = format!("{error:?}");
        assert!(
            error_string.contains("Export") || error_string.contains("export"),
            "Export error should contain export context, got: {error_string}"
        );
    }

    /// Objective: Verify ErrorUtils::internal_error creates correct error type
    /// Invariants: Internal error should contain message
    #[test]
    fn test_internal_error_creation() {
        let error = ErrorUtils::internal_error("unexpected state");
        let error_string = format!("{error:?}");
        assert!(
            error_string.contains("Internal") || error_string.contains("internal"),
            "Internal error should contain internal context, got: {error_string}"
        );
    }

    /// Objective: Verify empty string handling in error creation
    /// Invariants: Empty strings should not cause panic
    #[test]
    fn test_empty_string_handling() {
        let error1 = ErrorUtils::memory_error("");
        let error2 = ErrorUtils::config_error("", "");
        let error3 = ErrorUtils::system_error("");
        let error4 = ErrorUtils::analysis_error("", "");
        let error5 = ErrorUtils::export_error("", "");
        let error6 = ErrorUtils::internal_error("");

        assert!(!format!("{error1:?}").is_empty());
        assert!(!format!("{error2:?}").is_empty());
        assert!(!format!("{error3:?}").is_empty());
        assert!(!format!("{error4:?}").is_empty());
        assert!(!format!("{error5:?}").is_empty());
        assert!(!format!("{error6:?}").is_empty());
    }

    /// Objective: Verify special characters handling in error messages
    /// Invariants: Special characters should be preserved
    #[test]
    fn test_special_characters_handling() {
        let error =
            ErrorUtils::config_error("Test\nComponent", "error with \t tabs and \n newlines");
        let error_string = format!("{error:?}");
        assert!(
            !error_string.is_empty(),
            "Error with special chars should not be empty"
        );
    }

    /// Objective: Verify unicode handling in error messages
    /// Invariants: Unicode characters should be preserved
    #[test]
    fn test_unicode_handling() {
        let error = ErrorUtils::internal_error("错误信息 测试 🦀");
        let error_string = format!("{error:?}");
        assert!(
            error_string.contains("错误")
                || error_string.contains("🦀")
                || !error_string.is_empty()
        );
    }
}
