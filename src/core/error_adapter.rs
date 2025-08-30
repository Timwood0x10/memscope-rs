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
            TrackingError::ThreadSafetyError(msg) => {
                MemScopeError::system(SystemErrorType::Threading, msg)
            }
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
                    MemoryOperation::Validation => TrackingError::ValidationError(full_message),
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
}
