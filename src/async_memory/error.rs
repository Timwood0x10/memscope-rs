//! Error handling for async memory tracking
//!
//! Provides a unified error type following the project's error handling patterns.
//! All errors are designed to be recoverable and provide meaningful context.

use std::fmt;
use std::sync::Arc;

/// Unified error type for async memory tracking operations
///
/// Follows the project's pattern of using Arc<str> for efficient error messages
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
            Self::DataAggregation { partial_data_available, .. } => *partial_data_available,
            Self::Integration { fallback_available, .. } => *fallback_available,
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
            Self::Initialization { component, message, recoverable } => {
                write!(
                    f,
                    "Async memory tracking initialization failed in {}: {} ({})",
                    component,
                    message,
                    if *recoverable { "recoverable" } else { "fatal" }
                )
            }
            Self::TaskTracking { operation, message, task_id } => {
                if let Some(id) = task_id {
                    write!(f, "Task tracking error during {:?} for task {}: {}", operation, id, message)
                } else {
                    write!(f, "Task tracking error during {:?}: {}", operation, message)
                }
            }
            Self::AllocationTracking { event_type, message, allocation_size } => {
                if let Some(size) = allocation_size {
                    write!(f, "Allocation tracking error during {:?} ({}B): {}", event_type, size, message)
                } else {
                    write!(f, "Allocation tracking error during {:?}: {}", event_type, message)
                }
            }
            Self::BufferManagement { buffer_type, message, events_lost } => {
                if let Some(lost) = events_lost {
                    write!(f, "Buffer management error in {:?} ({} events lost): {}", buffer_type, lost, message)
                } else {
                    write!(f, "Buffer management error in {:?}: {}", buffer_type, message)
                }
            }
            Self::DataAggregation { aggregator, message, partial_data_available } => {
                write!(
                    f,
                    "Data aggregation error in {}: {} (partial data: {})",
                    aggregator,
                    message,
                    partial_data_available
                )
            }
            Self::Integration { component, message, fallback_available } => {
                write!(
                    f,
                    "Integration error with {}: {} (fallback: {})",
                    component,
                    message,
                    if *fallback_available { "available" } else { "unavailable" }
                )
            }
            Self::System { operation, message, source_error } => {
                if let Some(source) = source_error {
                    write!(f, "System error during {}: {} (source: {})", operation, message, source)
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

    #[test]
    fn test_error_creation() {
        let error = AsyncError::initialization("tracker", "Failed to start", true);
        assert!(error.is_recoverable());
        assert_eq!(error.component(), "tracker");
        assert_eq!(error.message(), "Failed to start");
    }

    #[test]
    fn test_error_display() {
        let error = AsyncError::task_tracking(
            TaskOperation::IdGeneration,
            "Invalid context",
            Some(12345),
        );
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
        assert!(AsyncError::task_tracking(TaskOperation::Propagation, "test", None).is_recoverable());
        assert!(AsyncError::allocation_tracking(AllocationEventType::Allocation, "test", None).is_recoverable());
        
        // Non-recoverable errors
        assert!(!AsyncError::system("io", "disk failure", None).is_recoverable());
    }
}