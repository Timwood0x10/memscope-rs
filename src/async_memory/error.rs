//! Error handling for async memory tracking
//!
//! Provides a unified error type following the project's error handling patterns.
//! All errors are designed to be recoverable and provide meaningful context.

use std::fmt;
use std::sync::Arc;

/// Unified error type for async memory tracking operations
///
/// Follows the project's pattern of using `Arc<str>` for efficient error messages
/// and avoiding string cloning overhead.
#[derive(Debug, Clone)]
pub enum AsyncError {
    /// Task tracking initialization or configuration errors
    Initialization {
        component: Arc<str>,
        message: Arc<str>,
        recoverable: bool,
    },

    /// Task identification and propagation errors
    TaskTracking {
        operation: TaskOperation,
        message: Arc<str>,
        task_id: Option<crate::async_memory::TaskId>,
    },

    /// Memory allocation tracking errors
    AllocationTracking {
        event_type: AllocationEventType,
        message: Arc<str>,
        allocation_size: Option<usize>,
    },

    /// Event buffer management errors
    BufferManagement {
        buffer_type: BufferType,
        message: Arc<str>,
        events_lost: Option<usize>,
    },

    /// Data aggregation and analysis errors
    DataAggregation {
        aggregator: Arc<str>,
        message: Arc<str>,
        partial_data_available: bool,
    },

    /// Integration errors with tokio runtime or tracing
    Integration {
        component: Arc<str>,
        message: Arc<str>,
        fallback_available: bool,
    },

    /// System-level errors (threading, IO, etc.)
    System {
        operation: Arc<str>,
        message: Arc<str>,
        source_error: Option<Arc<str>>,
    },
}

/// Task tracking operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskOperation {
    /// Task ID generation from Context
    IdGeneration,
    /// Task information propagation via thread-local storage
    Propagation,
    /// Task registration in tracking system
    Registration,
    /// Task completion cleanup
    Cleanup,
}

/// Allocation event types for error context
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationEventType {
    /// Memory allocation event
    Allocation,
    /// Memory deallocation event
    Deallocation,
    /// Event buffer write operation
    BufferWrite,
    /// Event processing by aggregator
    Processing,
}

/// Buffer management operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    /// Per-thread allocation event buffer
    AllocationEvents,
    /// Task profile cache
    TaskProfiles,
    /// Quality metrics buffer
    QualityMetrics,
}

impl AsyncError {
    /// Create an initialization error
    pub fn initialization(component: &str, message: &str, recoverable: bool) -> Self {
        Self::Initialization {
            component: Arc::from(component),
            message: Arc::from(message),
            recoverable,
        }
    }

    /// Create a task tracking error
    pub fn task_tracking(
        operation: TaskOperation,
        message: &str,
        task_id: Option<crate::async_memory::TaskId>,
    ) -> Self {
        Self::TaskTracking {
            operation,
            message: Arc::from(message),
            task_id,
        }
    }

    /// Create an allocation tracking error
    pub fn allocation_tracking(
        event_type: AllocationEventType,
        message: &str,
        allocation_size: Option<usize>,
    ) -> Self {
        Self::AllocationTracking {
            event_type,
            message: Arc::from(message),
            allocation_size,
        }
    }

    /// Create a buffer management error
    pub fn buffer_management(
        buffer_type: BufferType,
        message: &str,
        events_lost: Option<usize>,
    ) -> Self {
        Self::BufferManagement {
            buffer_type,
            message: Arc::from(message),
            events_lost,
        }
    }

    /// Create a data aggregation error
    pub fn data_aggregation(aggregator: &str, message: &str, partial_data: bool) -> Self {
        Self::DataAggregation {
            aggregator: Arc::from(aggregator),
            message: Arc::from(message),
            partial_data_available: partial_data,
        }
    }

    /// Create an integration error
    pub fn integration(component: &str, message: &str, fallback_available: bool) -> Self {
        Self::Integration {
            component: Arc::from(component),
            message: Arc::from(message),
            fallback_available,
        }
    }

    /// Create a system error
    pub fn system(operation: &str, message: &str, source: Option<&str>) -> Self {
        Self::System {
            operation: Arc::from(operation),
            message: Arc::from(message),
            source_error: source.map(Arc::from),
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Initialization { recoverable, .. } => *recoverable,
            Self::TaskTracking { .. } => true, // Task errors are usually recoverable
            Self::AllocationTracking { .. } => true, // Allocation errors don't crash system
            Self::BufferManagement { .. } => true, // Buffer overflow is expected
            Self::DataAggregation {
                partial_data_available,
                ..
            } => *partial_data_available,
            Self::Integration {
                fallback_available, ..
            } => *fallback_available,
            Self::System { .. } => false, // System errors are typically fatal
        }
    }

    /// Get the primary component affected by this error
    pub fn component(&self) -> &str {
        match self {
            Self::Initialization { component, .. } => component,
            Self::TaskTracking { .. } => "task_tracking",
            Self::AllocationTracking { .. } => "allocation_tracking",
            Self::BufferManagement { .. } => "buffer_management",
            Self::DataAggregation { aggregator, .. } => aggregator,
            Self::Integration { component, .. } => component,
            Self::System { operation, .. } => operation,
        }
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        match self {
            Self::Initialization { message, .. }
            | Self::TaskTracking { message, .. }
            | Self::AllocationTracking { message, .. }
            | Self::BufferManagement { message, .. }
            | Self::DataAggregation { message, .. }
            | Self::Integration { message, .. }
            | Self::System { message, .. } => message,
        }
    }
}

impl fmt::Display for AsyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initialization {
                component,
                message,
                recoverable,
            } => {
                write!(
                    f,
                    "Async memory tracking initialization failed in {}: {} ({})",
                    component,
                    message,
                    if *recoverable { "recoverable" } else { "fatal" }
                )
            }
            Self::TaskTracking {
                operation,
                message,
                task_id,
            } => {
                if let Some(id) = task_id {
                    write!(
                        f,
                        "Task tracking error during {:?} for task {}: {}",
                        operation, id, message
                    )
                } else {
                    write!(f, "Task tracking error during {:?}: {}", operation, message)
                }
            }
            Self::AllocationTracking {
                event_type,
                message,
                allocation_size,
            } => {
                if let Some(size) = allocation_size {
                    write!(
                        f,
                        "Allocation tracking error during {:?} ({}B): {}",
                        event_type, size, message
                    )
                } else {
                    write!(
                        f,
                        "Allocation tracking error during {:?}: {}",
                        event_type, message
                    )
                }
            }
            Self::BufferManagement {
                buffer_type,
                message,
                events_lost,
            } => {
                if let Some(lost) = events_lost {
                    write!(
                        f,
                        "Buffer management error in {:?} ({} events lost): {}",
                        buffer_type, lost, message
                    )
                } else {
                    write!(
                        f,
                        "Buffer management error in {:?}: {}",
                        buffer_type, message
                    )
                }
            }
            Self::DataAggregation {
                aggregator,
                message,
                partial_data_available,
            } => {
                write!(
                    f,
                    "Data aggregation error in {}: {} (partial data: {})",
                    aggregator, message, partial_data_available
                )
            }
            Self::Integration {
                component,
                message,
                fallback_available,
            } => {
                write!(
                    f,
                    "Integration error with {}: {} (fallback: {})",
                    component,
                    message,
                    if *fallback_available {
                        "available"
                    } else {
                        "unavailable"
                    }
                )
            }
            Self::System {
                operation,
                message,
                source_error,
            } => {
                if let Some(source) = source_error {
                    write!(
                        f,
                        "System error during {}: {} (source: {})",
                        operation, message, source
                    )
                } else {
                    write!(f, "System error during {}: {}", operation, message)
                }
            }
        }
    }
}

impl std::error::Error for AsyncError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // No source errors for now, but could be extended
        None
    }
}

/// Result type alias for async memory tracking operations
pub type AsyncResult<T> = Result<T, AsyncError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_error_creation() {
        let error = AsyncError::initialization("tracker", "Failed to start", true);
        assert!(error.is_recoverable());
        assert_eq!(error.component(), "tracker");
        assert_eq!(error.message(), "Failed to start");
    }

    #[test]
    fn test_error_display() {
        let error =
            AsyncError::task_tracking(TaskOperation::IdGeneration, "Invalid context", Some(12345));
        let display = format!("{}", error);
        assert!(display.contains("IdGeneration"));
        assert!(display.contains("12345"));
        assert!(display.contains("Invalid context"));
    }

    #[test]
    fn test_buffer_error_with_events_lost() {
        let error = AsyncError::buffer_management(
            BufferType::AllocationEvents,
            "Buffer overflow",
            Some(42),
        );
        let display = format!("{}", error);
        assert!(display.contains("42 events lost"));
    }

    #[test]
    fn test_error_recoverability() {
        // Recoverable errors
        assert!(
            AsyncError::task_tracking(TaskOperation::Propagation, "test", None).is_recoverable()
        );
        assert!(
            AsyncError::allocation_tracking(AllocationEventType::Allocation, "test", None)
                .is_recoverable()
        );

        // Non-recoverable errors
        assert!(!AsyncError::system("io", "disk failure", None).is_recoverable());
    }

    #[test]
    fn test_initialization_error_non_recoverable() {
        let error = AsyncError::initialization("runtime", "Critical failure", false);
        assert!(!error.is_recoverable());
        assert_eq!(error.component(), "runtime");
        assert_eq!(error.message(), "Critical failure");
        
        let display = format!("{}", error);
        assert!(display.contains("fatal"));
        assert!(display.contains("runtime"));
        assert!(display.contains("Critical failure"));
    }

    #[test]
    fn test_task_tracking_all_operations() {
        let operations = [
            TaskOperation::IdGeneration,
            TaskOperation::Propagation,
            TaskOperation::Registration,
            TaskOperation::Cleanup,
        ];
        
        for op in &operations {
            let error = AsyncError::task_tracking(*op, "test message", None);
            assert!(error.is_recoverable());
            assert_eq!(error.component(), "task_tracking");
            
            let display = format!("{}", error);
            assert!(display.contains(&format!("{:?}", op)));
        }
    }

    #[test]
    fn test_task_tracking_without_task_id() {
        let error = AsyncError::task_tracking(TaskOperation::Registration, "No task ID", None);
        let display = format!("{}", error);
        assert!(display.contains("Registration"));
        assert!(display.contains("No task ID"));
        // When no task_id is provided, the format should not include "for task [id]"
        assert!(!display.contains("for task"));
    }

    #[test]
    fn test_allocation_tracking_all_event_types() {
        let event_types = [
            AllocationEventType::Allocation,
            AllocationEventType::Deallocation,
            AllocationEventType::BufferWrite,
            AllocationEventType::Processing,
        ];
        
        for event_type in &event_types {
            let error = AsyncError::allocation_tracking(*event_type, "test", Some(1024));
            assert!(error.is_recoverable());
            assert_eq!(error.component(), "allocation_tracking");
            
            let display = format!("{}", error);
            assert!(display.contains(&format!("{:?}", event_type)));
            assert!(display.contains("1024B"));
        }
    }

    #[test]
    fn test_allocation_tracking_without_size() {
        let error = AsyncError::allocation_tracking(
            AllocationEventType::Processing,
            "Unknown size",
            None,
        );
        let display = format!("{}", error);
        assert!(display.contains("Processing"));
        assert!(display.contains("Unknown size"));
        assert!(!display.contains("B):"));
    }

    #[test]
    fn test_buffer_management_all_types() {
        let buffer_types = [
            BufferType::AllocationEvents,
            BufferType::TaskProfiles,
            BufferType::QualityMetrics,
        ];
        
        for buffer_type in &buffer_types {
            let error = AsyncError::buffer_management(*buffer_type, "test error", None);
            assert!(error.is_recoverable());
            assert_eq!(error.component(), "buffer_management");
            
            let display = format!("{}", error);
            assert!(display.contains(&format!("{:?}", buffer_type)));
        }
    }

    #[test]
    fn test_buffer_management_without_events_lost() {
        let error = AsyncError::buffer_management(
            BufferType::TaskProfiles,
            "Generic buffer error",
            None,
        );
        let display = format!("{}", error);
        assert!(display.contains("TaskProfiles"));
        assert!(display.contains("Generic buffer error"));
        assert!(!display.contains("events lost"));
    }

    #[test]
    fn test_data_aggregation_with_partial_data() {
        let error = AsyncError::data_aggregation("metrics_collector", "Incomplete data", true);
        assert!(error.is_recoverable());
        assert_eq!(error.component(), "metrics_collector");
        assert_eq!(error.message(), "Incomplete data");
        
        let display = format!("{}", error);
        assert!(display.contains("metrics_collector"));
        assert!(display.contains("partial data: true"));
    }

    #[test]
    fn test_data_aggregation_without_partial_data() {
        let error = AsyncError::data_aggregation("failed_aggregator", "Total failure", false);
        assert!(!error.is_recoverable());
        assert_eq!(error.component(), "failed_aggregator");
        
        let display = format!("{}", error);
        assert!(display.contains("partial data: false"));
    }

    #[test]
    fn test_integration_with_fallback() {
        let error = AsyncError::integration("tokio", "Runtime unavailable", true);
        assert!(error.is_recoverable());
        assert_eq!(error.component(), "tokio");
        
        let display = format!("{}", error);
        assert!(display.contains("tokio"));
        assert!(display.contains("fallback: available"));
    }

    #[test]
    fn test_integration_without_fallback() {
        let error = AsyncError::integration("tracing", "Critical dependency missing", false);
        assert!(!error.is_recoverable());
        assert_eq!(error.component(), "tracing");
        
        let display = format!("{}", error);
        assert!(display.contains("fallback: unavailable"));
    }

    #[test]
    fn test_system_error_with_source() {
        let error = AsyncError::system("file_io", "Write failed", Some("Permission denied"));
        assert!(!error.is_recoverable());
        assert_eq!(error.component(), "file_io");
        assert_eq!(error.message(), "Write failed");
        
        let display = format!("{}", error);
        assert!(display.contains("file_io"));
        assert!(display.contains("Write failed"));
        assert!(display.contains("source: Permission denied"));
    }

    #[test]
    fn test_system_error_without_source() {
        let error = AsyncError::system("network", "Connection timeout", None);
        assert!(!error.is_recoverable());
        
        let display = format!("{}", error);
        assert!(display.contains("network"));
        assert!(display.contains("Connection timeout"));
        assert!(!display.contains("source:"));
    }

    #[test]
    fn test_enum_equality() {
        // Test TaskOperation equality
        assert_eq!(TaskOperation::IdGeneration, TaskOperation::IdGeneration);
        assert_ne!(TaskOperation::IdGeneration, TaskOperation::Propagation);
        
        // Test AllocationEventType equality
        assert_eq!(AllocationEventType::Allocation, AllocationEventType::Allocation);
        assert_ne!(AllocationEventType::Allocation, AllocationEventType::Deallocation);
        
        // Test BufferType equality
        assert_eq!(BufferType::AllocationEvents, BufferType::AllocationEvents);
        assert_ne!(BufferType::AllocationEvents, BufferType::TaskProfiles);
    }

    #[test]
    fn test_enum_debug_formatting() {
        // Test TaskOperation debug format
        let op = TaskOperation::Registration;
        assert_eq!(format!("{:?}", op), "Registration");
        
        // Test AllocationEventType debug format
        let event = AllocationEventType::BufferWrite;
        assert_eq!(format!("{:?}", event), "BufferWrite");
        
        // Test BufferType debug format
        let buffer = BufferType::QualityMetrics;
        assert_eq!(format!("{:?}", buffer), "QualityMetrics");
    }

    #[test]
    fn test_error_clone() {
        let original = AsyncError::initialization("test", "clone test", true);
        let cloned = original.clone();
        
        assert_eq!(original.component(), cloned.component());
        assert_eq!(original.message(), cloned.message());
        assert_eq!(original.is_recoverable(), cloned.is_recoverable());
    }

    #[test]
    fn test_error_debug_formatting() {
        let error = AsyncError::task_tracking(TaskOperation::Cleanup, "debug test", Some(999));
        let debug_str = format!("{:?}", error);
        
        assert!(debug_str.contains("TaskTracking"));
        assert!(debug_str.contains("Cleanup"));
        assert!(debug_str.contains("debug test"));
        assert!(debug_str.contains("999"));
    }

    #[test]
    fn test_error_source_method() {
        let error = AsyncError::system("test", "source test", None);
        assert!(error.source().is_none());
    }

    #[test]
    fn test_async_result_type_alias() {
        let success: AsyncResult<i32> = Ok(42);
        assert_eq!(success.unwrap(), 42);
        
        let failure: AsyncResult<i32> = Err(AsyncError::system("test", "fail", None));
        assert!(failure.is_err());
    }

    #[test]
    fn test_comprehensive_error_scenarios() {
        // Test edge case combinations that might occur in real usage
        
        // Task tracking with large task ID
        let large_id_error = AsyncError::task_tracking(
            TaskOperation::IdGeneration,
            "Large ID test",
            Some(u128::MAX),
        );
        let display = format!("{}", large_id_error);
        assert!(display.contains(&u128::MAX.to_string()));
        
        // Allocation tracking with zero size
        let zero_size_error = AsyncError::allocation_tracking(
            AllocationEventType::Allocation,
            "Zero size allocation",
            Some(0),
        );
        let display = format!("{}", zero_size_error);
        assert!(display.contains("0B"));
        
        // Buffer management with large events lost
        let large_lost_error = AsyncError::buffer_management(
            BufferType::AllocationEvents,
            "Massive overflow",
            Some(1_000_000),
        );
        let display = format!("{}", large_lost_error);
        assert!(display.contains("1000000 events lost"));
    }
}
