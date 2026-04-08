//! Error type conversions to MemScopeError
//!
//! Provides conversions from various module-specific error types
//! to the unified MemScopeError type for centralized error handling.

use crate::core::error::{MemScopeError, MemoryOperation, SystemErrorType};

use super::error_manager::IntoMemScopeError;

// Implement IntoMemScopeError for AsyncError
impl IntoMemScopeError for crate::capture::backends::async_types::AsyncError {
    fn into_memscope_error(self, module: &str) -> MemScopeError {
        use crate::capture::backends::async_types::AsyncError;

        match self {
            AsyncError::Initialization {
                component,
                message,
                recoverable,
            } => {
                if recoverable {
                    MemScopeError::config(component, message.as_ref())
                } else {
                    MemScopeError::config(component, message.as_ref())
                }
            }
            AsyncError::TaskTracking {
                operation,
                message,
                task_id,
            } => {
                let context = task_id.map(|id| format!("task_id: {}", id));
                let operation_str = format!("{:?}", operation);
                MemScopeError::memory_with_context(
                    MemoryOperation::Tracking,
                    message.as_ref(),
                    format!(
                        "{}: {} - {:?}",
                        module,
                        operation_str,
                        context.unwrap_or_default()
                    ),
                )
            }
            AsyncError::AllocationTracking {
                event_type,
                message,
                allocation_size,
            } => {
                let context = allocation_size.map(|size| format!("size: {} bytes", size));
                let event_type_str = format!("{:?}", event_type);
                MemScopeError::memory_with_context(
                    MemoryOperation::Allocation,
                    message.as_ref(),
                    format!(
                        "{}: {} - {:?}",
                        module,
                        event_type_str,
                        context.unwrap_or_default()
                    ),
                )
            }
            AsyncError::System {
                operation: _,
                message,
            } => MemScopeError::system(
                SystemErrorType::Threading,
                format!("{}: {}", module, message),
            ),
        }
    }
}

// Implement IntoMemScopeError for ExportError
impl IntoMemScopeError for crate::render_engine::export::ExportError {
    fn into_memscope_error(self, module: &str) -> MemScopeError {
        use crate::render_engine::export::ExportError;

        match self {
            ExportError::Io(err) => MemScopeError::system_with_source(
                SystemErrorType::Io,
                format!("{}: IO error", module),
                err,
            ),
            ExportError::Json(err) => MemScopeError::system_with_source(
                SystemErrorType::Serialization,
                format!("{}: JSON error", module),
                err,
            ),
            ExportError::ExportFailed(msg) => {
                MemScopeError::export_partial("general", format!("{}: {}", module, msg))
            }
        }
    }
}

// Implement IntoMemScopeError for TrackingError (capture::types)
impl IntoMemScopeError for crate::capture::types::TrackingError {
    fn into_memscope_error(self, module: &str) -> MemScopeError {
        use crate::capture::types::TrackingError;

        match self {
            TrackingError::AllocationFailed(msg) => MemScopeError::memory_with_context(
                MemoryOperation::Allocation,
                msg.as_str(),
                module,
            ),
            TrackingError::DeallocationFailed(msg) => MemScopeError::memory_with_context(
                MemoryOperation::Deallocation,
                msg.as_str(),
                module,
            ),
            TrackingError::TrackingDisabled => {
                MemScopeError::config(module, "Memory tracking is disabled")
            }
            TrackingError::InvalidPointer(msg) => MemScopeError::memory_with_context(
                MemoryOperation::Validation,
                msg.as_str(),
                module,
            ),
            TrackingError::SerializationError(msg) => MemScopeError::system(
                SystemErrorType::Serialization,
                format!("{}: {}", module, msg),
            ),
            TrackingError::VisualizationError(msg) => {
                MemScopeError::export("visualization", format!("{}: {}", module, msg))
            }
            TrackingError::ThreadSafetyError(msg) => {
                MemScopeError::system(SystemErrorType::Threading, format!("{}: {}", module, msg))
            }
            TrackingError::ConfigurationError(msg) => MemScopeError::config(module, msg.as_str()),
            TrackingError::AnalysisError(msg) => MemScopeError::analysis(module, msg.as_str()),
            TrackingError::ExportError(msg) => {
                MemScopeError::export("general", format!("{}: {}", module, msg))
            }
            TrackingError::MemoryCorruption(msg) => MemScopeError::memory_with_context(
                MemoryOperation::Validation,
                msg.as_str(),
                module,
            ),
            TrackingError::UnsafeOperationDetected(msg) => {
                MemScopeError::analysis("unsafe", format!("{}: {}", module, msg))
            }
            TrackingError::FFIError(msg) => {
                MemScopeError::analysis("ffi", format!("{}: {}", module, msg))
            }
            TrackingError::ScopeError(msg) => {
                MemScopeError::memory_with_context(MemoryOperation::Tracking, msg.as_str(), module)
            }
            TrackingError::BorrowCheckError(msg) => {
                MemScopeError::analysis("borrow", format!("{}: {}", module, msg))
            }
            TrackingError::LifetimeError(msg) => {
                MemScopeError::analysis("lifetime", format!("{}: {}", module, msg))
            }
            TrackingError::TypeInferenceError(msg) => {
                MemScopeError::analysis("type", format!("{}: {}", module, msg))
            }
            TrackingError::PerformanceError(msg) => {
                MemScopeError::system(SystemErrorType::Io, format!("{}: {}", module, msg))
            }
            TrackingError::ResourceExhausted(msg) => {
                MemScopeError::system(SystemErrorType::Io, format!("{}: {}", module, msg))
            }
            TrackingError::InternalError(msg) => {
                MemScopeError::internal(format!("{}: {}", module, msg))
            }
            TrackingError::IoError(msg) => {
                MemScopeError::system(SystemErrorType::Io, format!("{}: {}", module, msg))
            }
            TrackingError::LockError(msg) => {
                MemScopeError::system(SystemErrorType::Locking, format!("{}: {}", module, msg))
            }
            TrackingError::ChannelError(msg) => {
                MemScopeError::system(SystemErrorType::Channel, format!("{}: {}", module, msg))
            }
            TrackingError::ThreadError(msg) => {
                MemScopeError::system(SystemErrorType::Threading, format!("{}: {}", module, msg))
            }
            TrackingError::InitializationError(msg) => MemScopeError::config(module, msg.as_str()),
            TrackingError::NotImplemented(msg) => {
                MemScopeError::internal(format!("{}: {}", module, msg))
            }
            TrackingError::ValidationError(msg) => MemScopeError::memory_with_context(
                MemoryOperation::Validation,
                msg.as_str(),
                module,
            ),
            TrackingError::InvalidOperation(msg) => {
                MemScopeError::memory_with_context(MemoryOperation::Tracking, msg.as_str(), module)
            }
            TrackingError::LockContention(msg) => {
                MemScopeError::system(SystemErrorType::Locking, format!("{}: {}", module, msg))
            }
            TrackingError::DataError(msg) => {
                MemScopeError::memory_with_context(MemoryOperation::Tracking, msg.as_str(), module)
            }
        }
    }
}

// Implement IntoMemScopeError for TrackingError (core::types)
impl IntoMemScopeError for crate::core::types::TrackingError {
    fn into_memscope_error(self, module: &str) -> MemScopeError {
        use crate::core::types::TrackingError;

        match self {
            TrackingError::AllocationFailed(msg) => MemScopeError::memory_with_context(
                MemoryOperation::Allocation,
                msg.as_str(),
                module,
            ),
            TrackingError::DeallocationFailed(msg) => MemScopeError::memory_with_context(
                MemoryOperation::Deallocation,
                msg.as_str(),
                module,
            ),
            TrackingError::TrackingDisabled => {
                MemScopeError::config(module, "Memory tracking is disabled")
            }
            TrackingError::InvalidPointer(msg) => MemScopeError::memory_with_context(
                MemoryOperation::Validation,
                msg.as_str(),
                module,
            ),
            TrackingError::SerializationError(msg) => MemScopeError::system(
                SystemErrorType::Serialization,
                format!("{}: {}", module, msg),
            ),
            TrackingError::VisualizationError(msg) => {
                MemScopeError::export("visualization", format!("{}: {}", module, msg))
            }
            TrackingError::ThreadSafetyError(msg) => {
                MemScopeError::system(SystemErrorType::Threading, format!("{}: {}", module, msg))
            }
            TrackingError::ConfigurationError(msg) => MemScopeError::config(module, msg.as_str()),
            TrackingError::AnalysisError(msg) => MemScopeError::analysis(module, msg.as_str()),
            TrackingError::ExportError(msg) => {
                MemScopeError::export("general", format!("{}: {}", module, msg))
            }
            TrackingError::MemoryCorruption(msg) => MemScopeError::memory_with_context(
                MemoryOperation::Validation,
                msg.as_str(),
                module,
            ),
            TrackingError::UnsafeOperationDetected(msg) => {
                MemScopeError::analysis("unsafe", format!("{}: {}", module, msg))
            }
            TrackingError::FFIError(msg) => {
                MemScopeError::analysis("ffi", format!("{}: {}", module, msg))
            }
            TrackingError::ScopeError(msg) => {
                MemScopeError::memory_with_context(MemoryOperation::Tracking, msg.as_str(), module)
            }
            TrackingError::BorrowCheckError(msg) => {
                MemScopeError::analysis("borrow", format!("{}: {}", module, msg))
            }
            TrackingError::LifetimeError(msg) => {
                MemScopeError::analysis("lifetime", format!("{}: {}", module, msg))
            }
            TrackingError::TypeInferenceError(msg) => {
                MemScopeError::analysis("type", format!("{}: {}", module, msg))
            }
            TrackingError::PerformanceError(msg) => {
                MemScopeError::system(SystemErrorType::Io, format!("{}: {}", module, msg))
            }
            TrackingError::ResourceExhausted(msg) => {
                MemScopeError::system(SystemErrorType::Io, format!("{}: {}", module, msg))
            }
            TrackingError::InternalError(msg) => {
                MemScopeError::internal(format!("{}: {}", module, msg))
            }
            TrackingError::IoError(msg) => {
                MemScopeError::system(SystemErrorType::Io, format!("{}: {}", module, msg))
            }
            TrackingError::LockError(msg) => {
                MemScopeError::system(SystemErrorType::Locking, format!("{}: {}", module, msg))
            }
            TrackingError::ChannelError(msg) => {
                MemScopeError::system(SystemErrorType::Channel, format!("{}: {}", module, msg))
            }
            TrackingError::ThreadError(msg) => {
                MemScopeError::system(SystemErrorType::Threading, format!("{}: {}", module, msg))
            }
            TrackingError::InitializationError(msg) => MemScopeError::config(module, msg.as_str()),
            TrackingError::NotImplemented(msg) => {
                MemScopeError::internal(format!("{}: {}", module, msg))
            }
            TrackingError::ValidationError(msg) => MemScopeError::memory_with_context(
                MemoryOperation::Validation,
                msg.as_str(),
                module,
            ),
            TrackingError::InvalidOperation(msg) => {
                MemScopeError::memory_with_context(MemoryOperation::Tracking, msg.as_str(), module)
            }
            TrackingError::LockContention(msg) => {
                MemScopeError::system(SystemErrorType::Locking, format!("{}: {}", module, msg))
            }
            TrackingError::DataError(msg) => {
                MemScopeError::memory_with_context(MemoryOperation::Tracking, msg.as_str(), module)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_error_conversion() {
        use crate::capture::backends::async_types::AsyncError;

        let error = AsyncError::Initialization {
            component: "test".into(),
            message: "failed".into(),
            recoverable: false,
        };
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(
            memscope_error,
            MemScopeError::Configuration { .. }
        ));
    }

    #[test]
    fn test_tracking_error_conversion() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::AllocationFailed("test failed".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }
}
