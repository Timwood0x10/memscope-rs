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
                recoverable: _,
            } => MemScopeError::config(component, message.as_ref()),
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
    use std::sync::Arc;

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

    #[test]
    fn test_async_error_task_tracking_conversion() {
        use crate::capture::backends::async_types::{AsyncError, TaskOperation};

        let error = AsyncError::TaskTracking {
            operation: TaskOperation::IdGeneration,
            message: Arc::from("task error"),
            task_id: Some(123),
        };
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_async_error_task_tracking_no_id() {
        use crate::capture::backends::async_types::{AsyncError, TaskOperation};

        let error = AsyncError::TaskTracking {
            operation: TaskOperation::Registration,
            message: Arc::from("no task id"),
            task_id: None,
        };
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_async_error_allocation_tracking_conversion() {
        use crate::capture::backends::async_types::{AllocationEventType, AsyncError};

        let error = AsyncError::AllocationTracking {
            event_type: AllocationEventType::Allocation,
            message: Arc::from("alloc error"),
            allocation_size: Some(1024),
        };
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_async_error_allocation_tracking_no_size() {
        use crate::capture::backends::async_types::{AllocationEventType, AsyncError};

        let error = AsyncError::AllocationTracking {
            event_type: AllocationEventType::Deallocation,
            message: Arc::from("dealloc error"),
            allocation_size: None,
        };
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_async_error_system_conversion() {
        use crate::capture::backends::async_types::AsyncError;

        let error = AsyncError::System {
            operation: Arc::from("test_op"),
            message: Arc::from("system error"),
        };
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_export_error_io_conversion() {
        use crate::render_engine::export::ExportError;
        use std::io;

        let error = ExportError::Io(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        let memscope_error = error.into_memscope_error("export_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_export_error_json_conversion() {
        use crate::render_engine::export::ExportError;

        let error = ExportError::Json(serde_json::from_str::<i32>("invalid json").unwrap_err());
        let memscope_error = error.into_memscope_error("export_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_export_error_export_failed_conversion() {
        use crate::render_engine::export::ExportError;

        let error = ExportError::ExportFailed("export failed".to_string());
        let memscope_error = error.into_memscope_error("export_module");

        assert!(matches!(memscope_error, MemScopeError::Export { .. }));
    }

    #[test]
    fn test_tracking_error_deallocation_failed() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::DeallocationFailed("dealloc failed".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_tracking_error_tracking_disabled() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::TrackingDisabled;
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(
            memscope_error,
            MemScopeError::Configuration { .. }
        ));
    }

    #[test]
    fn test_tracking_error_invalid_pointer() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::InvalidPointer("bad pointer".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_tracking_error_serialization_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::SerializationError("serde error".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_tracking_error_visualization_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::VisualizationError("viz error".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Export { .. }));
    }

    #[test]
    fn test_tracking_error_thread_safety_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::ThreadSafetyError("thread error".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_tracking_error_configuration_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::ConfigurationError("config error".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(
            memscope_error,
            MemScopeError::Configuration { .. }
        ));
    }

    #[test]
    fn test_tracking_error_analysis_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::AnalysisError("analysis error".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Analysis { .. }));
    }

    #[test]
    fn test_tracking_error_export_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::ExportError("export error".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Export { .. }));
    }

    #[test]
    fn test_tracking_error_memory_corruption() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::MemoryCorruption("corruption detected".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_tracking_error_unsafe_operation() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::UnsafeOperationDetected("unsafe code".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Analysis { .. }));
    }

    #[test]
    fn test_tracking_error_ffi_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::FFIError("ffi boundary error".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Analysis { .. }));
    }

    #[test]
    fn test_tracking_error_scope_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::ScopeError("scope error".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_tracking_error_borrow_check_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::BorrowCheckError("borrow error".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Analysis { .. }));
    }

    #[test]
    fn test_tracking_error_lifetime_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::LifetimeError("lifetime error".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Analysis { .. }));
    }

    #[test]
    fn test_tracking_error_type_inference_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::TypeInferenceError("type error".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Analysis { .. }));
    }

    #[test]
    fn test_tracking_error_performance_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::PerformanceError("slow".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_tracking_error_resource_exhausted() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::ResourceExhausted("out of memory".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_tracking_error_internal_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::InternalError("internal bug".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Internal { .. }));
    }

    #[test]
    fn test_tracking_error_io_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::IoError("io failed".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_tracking_error_lock_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::LockError("lock poisoned".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_tracking_error_channel_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::ChannelError("channel closed".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_tracking_error_thread_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::ThreadError("thread panic".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_tracking_error_initialization_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::InitializationError("init failed".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(
            memscope_error,
            MemScopeError::Configuration { .. }
        ));
    }

    #[test]
    fn test_tracking_error_not_implemented() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::NotImplemented("todo".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Internal { .. }));
    }

    #[test]
    fn test_tracking_error_validation_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::ValidationError("invalid data".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_tracking_error_invalid_operation() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::InvalidOperation("bad op".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_tracking_error_lock_contention() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::LockContention("high contention".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_tracking_error_data_error() {
        use crate::capture::types::TrackingError;

        let error = TrackingError::DataError("corrupt data".to_string());
        let memscope_error = error.into_memscope_error("test_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_core_tracking_error_allocation_failed() {
        use crate::core::types::TrackingError;

        let error = TrackingError::AllocationFailed("core alloc failed".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_core_tracking_error_deallocation_failed() {
        use crate::core::types::TrackingError;

        let error = TrackingError::DeallocationFailed("core dealloc failed".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_core_tracking_error_tracking_disabled() {
        use crate::core::types::TrackingError;

        let error = TrackingError::TrackingDisabled;
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(
            memscope_error,
            MemScopeError::Configuration { .. }
        ));
    }

    #[test]
    fn test_core_tracking_error_invalid_pointer() {
        use crate::core::types::TrackingError;

        let error = TrackingError::InvalidPointer("core bad pointer".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_core_tracking_error_serialization_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::SerializationError("core serde error".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_core_tracking_error_visualization_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::VisualizationError("core viz error".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Export { .. }));
    }

    #[test]
    fn test_core_tracking_error_thread_safety_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::ThreadSafetyError("core thread error".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_core_tracking_error_configuration_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::ConfigurationError("core config error".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(
            memscope_error,
            MemScopeError::Configuration { .. }
        ));
    }

    #[test]
    fn test_core_tracking_error_analysis_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::AnalysisError("core analysis error".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Analysis { .. }));
    }

    #[test]
    fn test_core_tracking_error_export_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::ExportError("core export error".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Export { .. }));
    }

    #[test]
    fn test_core_tracking_error_memory_corruption() {
        use crate::core::types::TrackingError;

        let error = TrackingError::MemoryCorruption("core corruption".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_core_tracking_error_unsafe_operation() {
        use crate::core::types::TrackingError;

        let error = TrackingError::UnsafeOperationDetected("core unsafe".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Analysis { .. }));
    }

    #[test]
    fn test_core_tracking_error_ffi_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::FFIError("core ffi error".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Analysis { .. }));
    }

    #[test]
    fn test_core_tracking_error_scope_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::ScopeError("core scope error".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_core_tracking_error_borrow_check_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::BorrowCheckError("core borrow error".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Analysis { .. }));
    }

    #[test]
    fn test_core_tracking_error_lifetime_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::LifetimeError("core lifetime error".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Analysis { .. }));
    }

    #[test]
    fn test_core_tracking_error_type_inference_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::TypeInferenceError("core type error".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Analysis { .. }));
    }

    #[test]
    fn test_core_tracking_error_performance_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::PerformanceError("core slow".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_core_tracking_error_resource_exhausted() {
        use crate::core::types::TrackingError;

        let error = TrackingError::ResourceExhausted("core oom".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_core_tracking_error_internal_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::InternalError("core internal".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Internal { .. }));
    }

    #[test]
    fn test_core_tracking_error_io_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::IoError("core io failed".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_core_tracking_error_lock_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::LockError("core lock poisoned".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_core_tracking_error_channel_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::ChannelError("core channel closed".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_core_tracking_error_thread_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::ThreadError("core thread panic".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_core_tracking_error_initialization_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::InitializationError("core init failed".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(
            memscope_error,
            MemScopeError::Configuration { .. }
        ));
    }

    #[test]
    fn test_core_tracking_error_not_implemented() {
        use crate::core::types::TrackingError;

        let error = TrackingError::NotImplemented("core todo".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Internal { .. }));
    }

    #[test]
    fn test_core_tracking_error_validation_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::ValidationError("core invalid".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_core_tracking_error_invalid_operation() {
        use crate::core::types::TrackingError;

        let error = TrackingError::InvalidOperation("core bad op".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }

    #[test]
    fn test_core_tracking_error_lock_contention() {
        use crate::core::types::TrackingError;

        let error = TrackingError::LockContention("core contention".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::System { .. }));
    }

    #[test]
    fn test_core_tracking_error_data_error() {
        use crate::core::types::TrackingError;

        let error = TrackingError::DataError("core corrupt".to_string());
        let memscope_error = error.into_memscope_error("core_module");

        assert!(matches!(memscope_error, MemScopeError::Memory { .. }));
    }
}
