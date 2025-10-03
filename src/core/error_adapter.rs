//! Error adapter for backward compatibility
//!
//! This module provides adapters to maintain compatibility with existing code
//! that uses the old TrackingError type while internally using the new
//! unified MemScopeError system.

use crate::core::error::{MemScopeError, MemoryOperation, SystemErrorType};
use crate::core::types::TrackingError;

/// Adapter trait for converting between old and new error types
pub trait ErrorAdapter {
    /// Convert from old TrackingError to new MemScopeError
    fn from_tracking_error(error: TrackingError) -> MemScopeError;

    /// Convert from new MemScopeError to old TrackingError for compatibility
    fn to_tracking_error(error: &MemScopeError) -> TrackingError;
}

/// Default implementation of ErrorAdapter
pub struct DefaultErrorAdapter;

impl ErrorAdapter for DefaultErrorAdapter {
    fn from_tracking_error(error: TrackingError) -> MemScopeError {
        match error {
            TrackingError::DataError(msg) => MemScopeError::memory(MemoryOperation::Tracking, msg),
            TrackingError::AllocationFailed(msg) => {
                MemScopeError::memory(MemoryOperation::Allocation, msg)
            }
            TrackingError::DeallocationFailed(msg) => {
                MemScopeError::memory(MemoryOperation::Deallocation, msg)
            }
            TrackingError::TrackingDisabled => {
                MemScopeError::config("tracker", "Memory tracking is disabled")
            }
            TrackingError::InvalidPointer(msg) => {
                MemScopeError::memory(MemoryOperation::Validation, msg)
            }
            TrackingError::SerializationError(msg) => {
                MemScopeError::system(SystemErrorType::Serialization, msg)
            }
            TrackingError::VisualizationError(msg) => MemScopeError::export("visualization", msg),
            TrackingError::ThreadSafetyError(msg) => MemScopeError::analysis("thread_safety", msg),
            TrackingError::ConfigurationError(msg) => MemScopeError::config("general", msg),
            TrackingError::AnalysisError(msg) => MemScopeError::analysis("general", msg),
            TrackingError::ExportError(msg) => MemScopeError::export("general", msg),
            TrackingError::MemoryCorruption(msg) => {
                MemScopeError::memory(MemoryOperation::Validation, msg)
            }
            TrackingError::UnsafeOperationDetected(msg) => MemScopeError::analysis("unsafe", msg),
            TrackingError::FFIError(msg) => MemScopeError::analysis("ffi", msg),
            TrackingError::ScopeError(msg) => MemScopeError::memory(MemoryOperation::Tracking, msg),
            TrackingError::BorrowCheckError(msg) => MemScopeError::analysis("borrow", msg),
            TrackingError::LifetimeError(msg) => MemScopeError::analysis("lifetime", msg),
            TrackingError::TypeInferenceError(msg) => MemScopeError::analysis("type", msg),
            TrackingError::PerformanceError(msg) => MemScopeError::system(SystemErrorType::Io, msg),
            TrackingError::ResourceExhausted(msg) => {
                MemScopeError::system(SystemErrorType::Io, msg)
            }
            TrackingError::InternalError(msg) => MemScopeError::internal(msg),
            TrackingError::IoError(msg) => MemScopeError::system(SystemErrorType::Io, msg),
            TrackingError::LockError(msg) => MemScopeError::system(SystemErrorType::Locking, msg),
            TrackingError::ChannelError(msg) => {
                MemScopeError::system(SystemErrorType::Channel, msg)
            }
            TrackingError::ThreadError(msg) => {
                MemScopeError::system(SystemErrorType::Threading, msg)
            }
            TrackingError::InitializationError(msg) => MemScopeError::config("initialization", msg),
            TrackingError::NotImplemented(msg) => MemScopeError::internal(msg),
            TrackingError::ValidationError(msg) => {
                MemScopeError::memory(MemoryOperation::Validation, msg)
            }
            TrackingError::InvalidOperation(msg) => {
                MemScopeError::memory(MemoryOperation::Tracking, msg)
            }
            TrackingError::LockContention(msg) => {
                MemScopeError::memory(MemoryOperation::Tracking, msg)
            }
        }
    }

    fn to_tracking_error(error: &MemScopeError) -> TrackingError {
        match error {
            MemScopeError::Memory {
                operation,
                message,
                context,
            } => {
                let full_message = if let Some(ctx) = context {
                    format!("{message} (context: {ctx})")
                } else {
                    message.to_string()
                };

                match operation {
                    MemoryOperation::Allocation => TrackingError::AllocationFailed(full_message),
                    MemoryOperation::Deallocation => {
                        TrackingError::DeallocationFailed(full_message)
                    }
                    MemoryOperation::Association => TrackingError::InvalidOperation(full_message),
                    MemoryOperation::Tracking => TrackingError::ScopeError(full_message),
                    MemoryOperation::Validation => TrackingError::InvalidPointer(full_message),
                }
            }
            MemScopeError::Analysis {
                analyzer, message, ..
            } => {
                let full_message = format!("{analyzer}: {message}");
                match analyzer.as_ref() {
                    "unsafe" => TrackingError::UnsafeOperationDetected(full_message),
                    "ffi" => TrackingError::FFIError(full_message),
                    "borrow" => TrackingError::BorrowCheckError(full_message),
                    "lifetime" => TrackingError::LifetimeError(full_message),
                    "type" => TrackingError::TypeInferenceError(full_message),
                    "thread_safety" => TrackingError::ThreadSafetyError(full_message),
                    _ => TrackingError::AnalysisError(full_message),
                }
            }
            MemScopeError::Export {
                format, message, ..
            } => {
                let full_message = format!("{format}: {message}");
                match format.as_ref() {
                    "visualization" => TrackingError::VisualizationError(full_message),
                    _ => TrackingError::ExportError(full_message),
                }
            }
            MemScopeError::Configuration { component, message } => {
                let full_message = format!("{component}: {message}");
                match component.as_ref() {
                    "tracker" => TrackingError::TrackingDisabled,
                    "initialization" => TrackingError::InitializationError(full_message),
                    _ => TrackingError::ConfigurationError(full_message),
                }
            }
            MemScopeError::System {
                error_type,
                message,
                ..
            } => match error_type {
                SystemErrorType::Io => TrackingError::IoError(message.to_string()),
                SystemErrorType::Threading => TrackingError::ThreadSafetyError(message.to_string()),
                SystemErrorType::Locking => TrackingError::LockError(message.to_string()),
                SystemErrorType::Channel => TrackingError::ChannelError(message.to_string()),
                SystemErrorType::Serialization => {
                    TrackingError::SerializationError(message.to_string())
                }
                SystemErrorType::Network => TrackingError::IoError(message.to_string()),
                SystemErrorType::FileSystem => TrackingError::IoError(message.to_string()),
            },
            MemScopeError::Internal { message, .. } => {
                TrackingError::InternalError(message.to_string())
            }
        }
    }
}

/// Convenience functions for error conversion
pub fn from_tracking_error(error: TrackingError) -> MemScopeError {
    DefaultErrorAdapter::from_tracking_error(error)
}

pub fn to_tracking_error(error: &MemScopeError) -> TrackingError {
    DefaultErrorAdapter::to_tracking_error(error)
}

/// Macro for easy error conversion in existing code
#[macro_export]
macro_rules! convert_error {
    ($error:expr) => {
        $crate::core::error_adapter::from_tracking_error($error)
    };
}

/// Result type adapter for backward compatibility
pub type AdaptedResult<T> = Result<T, MemScopeError>;

/// Convert old TrackingResult to new Result type
pub fn adapt_result<T>(result: crate::core::types::TrackingResult<T>) -> AdaptedResult<T> {
    result.map_err(from_tracking_error)
}

/// Convert new Result to old TrackingResult for compatibility
pub fn to_tracking_result<T>(result: AdaptedResult<T>) -> crate::core::types::TrackingResult<T> {
    result.map_err(|e| to_tracking_error(&e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ErrorSeverity;

    #[test]
    fn test_allocation_error_conversion() {
        let old_error = TrackingError::AllocationFailed("test allocation failed".to_string());
        let new_error = DefaultErrorAdapter::from_tracking_error(old_error);

        assert_eq!(new_error.category(), "memory");
        assert!(new_error.user_message().contains("test allocation failed"));

        let converted_back = DefaultErrorAdapter::to_tracking_error(&new_error);
        assert!(matches!(converted_back, TrackingError::AllocationFailed(_)));
    }

    #[test]
    fn test_analysis_error_conversion() {
        let old_error = TrackingError::BorrowCheckError("borrow check failed".to_string());
        let new_error = DefaultErrorAdapter::from_tracking_error(old_error);

        assert_eq!(new_error.category(), "analysis");

        let converted_back = DefaultErrorAdapter::to_tracking_error(&new_error);
        assert!(matches!(converted_back, TrackingError::BorrowCheckError(_)));
    }

    #[test]
    fn test_system_error_conversion() {
        let old_error = TrackingError::IoError("IO operation failed".to_string());
        let new_error = DefaultErrorAdapter::from_tracking_error(old_error);

        assert_eq!(new_error.category(), "system");

        let converted_back = DefaultErrorAdapter::to_tracking_error(&new_error);
        assert!(matches!(converted_back, TrackingError::IoError(_)));
    }

    #[test]
    fn test_result_adaptation() {
        let old_result: crate::core::types::TrackingResult<i32> =
            Err(TrackingError::AllocationFailed("test".to_string()));

        let adapted_result = adapt_result(old_result);
        assert!(adapted_result.is_err());

        let converted_back = to_tracking_result(adapted_result);
        assert!(matches!(
            converted_back,
            Err(TrackingError::AllocationFailed(_))
        ));
    }

    #[test]
    fn test_all_tracking_error_variants() {
        // Test all TrackingError variants for comprehensive coverage
        let test_errors = vec![
            TrackingError::AllocationFailed("allocation failed".to_string()),
            TrackingError::DeallocationFailed("deallocation failed".to_string()),
            TrackingError::BorrowCheckError("borrow check error".to_string()),
            TrackingError::LifetimeError("lifetime error".to_string()),
            TrackingError::ThreadSafetyError("thread safety error".to_string()),
            TrackingError::InvalidPointer("invalid pointer".to_string()),
            TrackingError::IoError("IO error".to_string()),
            TrackingError::SerializationError("serialization error".to_string()),
            TrackingError::ConfigurationError("configuration error".to_string()),
            TrackingError::InternalError("internal error".to_string()),
        ];

        for original_error in test_errors {
            // Convert to new error format
            let adapted_error = DefaultErrorAdapter::from_tracking_error(original_error.clone());

            // Verify the adapted error has appropriate properties
            assert!(!adapted_error.category().is_empty());
            assert!(!adapted_error.user_message().is_empty());
            // Skip technical_details check as method doesn't exist

            // Convert back to tracking error
            let converted_back = DefaultErrorAdapter::to_tracking_error(&adapted_error);

            // Verify the conversion preserves the error type
            match (&original_error, &converted_back) {
                (TrackingError::AllocationFailed(_), TrackingError::AllocationFailed(_)) => {}
                (TrackingError::DeallocationFailed(_), TrackingError::DeallocationFailed(_)) => {}
                (TrackingError::BorrowCheckError(_), TrackingError::BorrowCheckError(_)) => {}
                (TrackingError::LifetimeError(_), TrackingError::LifetimeError(_)) => {}
                (TrackingError::ThreadSafetyError(_), TrackingError::ThreadSafetyError(_)) => {}
                (TrackingError::InvalidPointer(_), TrackingError::InvalidPointer(_)) => {}
                (TrackingError::IoError(_), TrackingError::IoError(_)) => {}
                (TrackingError::SerializationError(_), TrackingError::SerializationError(_)) => {}
                (TrackingError::ConfigurationError(_), TrackingError::ConfigurationError(_)) => {}
                (TrackingError::InternalError(_), TrackingError::InternalError(_)) => {}
                _ => panic!(
                    "Error type mismatch: {:?} -> {:?}",
                    original_error, converted_back
                ),
            }
        }
    }

    #[test]
    fn test_error_categories() {
        // Test that errors are categorized correctly
        let memory_errors = vec![
            TrackingError::AllocationFailed("test".to_string()),
            TrackingError::DeallocationFailed("test".to_string()),
            TrackingError::InvalidPointer("test".to_string()),
        ];

        for error in memory_errors {
            let adapted = DefaultErrorAdapter::from_tracking_error(error);
            assert_eq!(adapted.category(), "memory");
        }

        let analysis_errors = vec![
            TrackingError::BorrowCheckError("test".to_string()),
            TrackingError::LifetimeError("test".to_string()),
            TrackingError::ThreadSafetyError("test".to_string()),
        ];

        for error in analysis_errors {
            let adapted = DefaultErrorAdapter::from_tracking_error(error);
            assert_eq!(adapted.category(), "analysis");
        }

        let system_errors = vec![
            TrackingError::IoError("test".to_string()),
            TrackingError::SerializationError("test".to_string()),
        ];

        let config_errors = vec![TrackingError::ConfigurationError("test".to_string())];

        let internal_errors = vec![TrackingError::InternalError("test".to_string())];

        for error in system_errors {
            let adapted = DefaultErrorAdapter::from_tracking_error(error);
            assert_eq!(adapted.category(), "system");
        }

        for error in config_errors {
            let adapted = DefaultErrorAdapter::from_tracking_error(error);
            assert_eq!(adapted.category(), "config");
        }

        for error in internal_errors {
            let adapted = DefaultErrorAdapter::from_tracking_error(error);
            assert_eq!(adapted.category(), "internal");
        }
    }

    #[test]
    fn test_successful_result_adaptation() {
        // Test successful results pass through unchanged
        let success_result: crate::core::types::TrackingResult<String> = Ok("success".to_string());
        let adapted = adapt_result(success_result);
        assert!(adapted.is_ok());
        assert_eq!(adapted.unwrap(), "success");

        // Test conversion back
        let success_adapted: AdaptedResult<i32> = Ok(42);
        let converted_back = to_tracking_result(success_adapted);
        assert!(converted_back.is_ok());
        assert_eq!(converted_back.unwrap(), 42);
    }

    #[test]
    fn test_error_message_preservation() {
        let test_cases = vec![
            ("Simple error message", TrackingError::AllocationFailed("Simple error message".to_string())),
            ("Error with special chars: !@#$%^&*()", TrackingError::IoError("Error with special chars: !@#$%^&*()".to_string())),
            ("Unicode error: error message 🦀", TrackingError::BorrowCheckError("Unicode error: error message 🦀".to_string())),
            ("", TrackingError::InternalError("".to_string())), // Empty message
            ("Very long error message that spans multiple lines and contains lots of details about what went wrong in the system", 
             TrackingError::ConfigurationError("Very long error message that spans multiple lines and contains lots of details about what went wrong in the system".to_string())),
        ];

        for (expected_content, error) in test_cases {
            let adapted = DefaultErrorAdapter::from_tracking_error(error);
            assert!(adapted.user_message().contains(expected_content));
            // Skip technical_details check as method doesn't exist
        }
    }

    #[test]
    fn test_error_adapter_trait_methods() {
        let _adapter = DefaultErrorAdapter;

        // Test with various error types
        let allocation_error = TrackingError::AllocationFailed("allocation failed".to_string());
        let adapted = DefaultErrorAdapter::from_tracking_error(allocation_error.clone());

        // Test trait methods
        assert!(!adapted.category().is_empty());
        assert!(!adapted.user_message().is_empty());
        assert!(!adapted.user_message().is_empty());
        assert!(adapted.severity() <= ErrorSeverity::Critical); // Check severity is valid

        // Test conversion back
        let converted = DefaultErrorAdapter::to_tracking_error(&adapted);
        assert!(matches!(converted, TrackingError::AllocationFailed(_)));
    }

    #[test]
    fn test_error_severity_levels() {
        // Test that different error types have appropriate severity levels
        let critical_errors = vec![
            TrackingError::InvalidPointer("test".to_string()),
            TrackingError::AllocationFailed("test".to_string()),
        ];

        let warning_errors = vec![
            TrackingError::BorrowCheckError("test".to_string()),
            TrackingError::LifetimeError("test".to_string()),
        ];

        let info_errors = vec![TrackingError::ConfigurationError("test".to_string())];

        // Memory errors should have medium severity
        for error in critical_errors {
            let adapted = DefaultErrorAdapter::from_tracking_error(error);
            assert_eq!(
                adapted.severity(),
                ErrorSeverity::Medium,
                "Memory error should have medium severity"
            );
        }

        // Analysis errors should have low severity (unless non-recoverable)
        for error in warning_errors {
            let adapted = DefaultErrorAdapter::from_tracking_error(error);
            assert_eq!(
                adapted.severity(),
                ErrorSeverity::Low,
                "Analysis error should have low severity"
            );
        }

        // Info errors should have lower severity
        for error in info_errors {
            let adapted = DefaultErrorAdapter::from_tracking_error(error);
            assert!(
                adapted.severity() <= ErrorSeverity::High,
                "Info error should have reasonable severity"
            );
        }
    }

    #[test]
    fn test_error_chain_preservation() {
        // Test that error chains are preserved through adaptation
        let original_error = TrackingError::AllocationFailed("root cause".to_string());
        let adapted = DefaultErrorAdapter::from_tracking_error(original_error);

        // The user message should contain the original error information
        assert!(adapted.user_message().contains("root cause"));
    }

    #[test]
    fn test_concurrent_error_adaptation() {
        use std::sync::Arc;
        use std::thread;

        let _adapter = Arc::new(DefaultErrorAdapter);
        let mut handles = vec![];

        // Test concurrent error adaptation
        for i in 0..10 {
            let handle = thread::spawn(move || {
                let error = TrackingError::AllocationFailed(format!("error_{}", i));
                let adapted = DefaultErrorAdapter::from_tracking_error(error);
                let converted_back = DefaultErrorAdapter::to_tracking_error(&adapted);

                match converted_back {
                    TrackingError::AllocationFailed(msg) => {
                        assert!(msg.contains(&format!("error_{}", i)))
                    }
                    _ => panic!("Unexpected error type"),
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread should complete successfully");
        }
    }

    #[test]
    fn test_error_formatting() {
        let error = TrackingError::AllocationFailed("test allocation".to_string());
        let adapted = DefaultErrorAdapter::from_tracking_error(error);

        // Test that the adapted error can be formatted
        let user_msg = adapted.user_message();
        let category = adapted.category();

        assert!(!user_msg.is_empty());
        assert!(!category.is_empty());

        // Test that formatting doesn't panic
        let _formatted = format!("Error: {} ({})", user_msg, category);
    }

    #[test]
    fn test_result_chain_operations() {
        // Test chaining operations on adapted results
        let success: crate::core::types::TrackingResult<i32> = Ok(10);
        let adapted_success = adapt_result(success);

        // Test mapping operations
        let mapped = adapted_success.map(|x| x * 2);
        assert_eq!(mapped.unwrap(), 20);

        // Test error case
        let error_result: crate::core::types::TrackingResult<i32> =
            Err(TrackingError::AllocationFailed("test".to_string()));
        let adapted_error = adapt_result(error_result);

        let mapped_error = adapted_error.map(|x| x * 2);
        assert!(mapped_error.is_err());

        // Convert back and verify
        let converted_back = to_tracking_result(mapped_error);
        assert!(matches!(
            converted_back,
            Err(TrackingError::AllocationFailed(_))
        ));
    }
}
