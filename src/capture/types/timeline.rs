//! Timeline tracking types.
//!
//! This module contains types for timeline visualization,
//! including memory snapshots, allocation events, and time ranges.

use serde::{Deserialize, Serialize};

use super::scope::{AllocationEventType, ScopeEventType};
use super::stats::TypeMemoryUsage;

/// Timeline data for visualization.
#[derive(Debug, Clone, Serialize)]
pub struct TimelineData {
    /// Time range.
    pub time_range: TimeRange,
    /// Allocation events.
    pub allocation_events: Vec<AllocationEvent>,
    /// Scope events.
    pub scope_events: Vec<ScopeEvent>,
    /// Memory snapshots.
    pub memory_snapshots: Vec<MemorySnapshot>,
}

/// Time range for timeline visualization.
#[derive(Debug, Clone, Serialize)]
pub struct TimeRange {
    /// Start time.
    pub start_time: u64,
    /// End time.
    pub end_time: u64,
    /// Duration in milliseconds.
    pub duration_ms: u64,
}

/// Memory snapshot at a point in time.
#[derive(Debug, Clone, Serialize)]
pub struct MemorySnapshot {
    /// Timestamp.
    pub timestamp: u64,
    /// Total memory.
    pub total_memory: usize,
    /// Active allocations.
    pub active_allocations: usize,
    /// Fragmentation ratio.
    pub fragmentation_ratio: f64,
    /// Top types by memory usage.
    pub top_types: Vec<TypeMemoryUsage>,
}

/// Allocation event for timeline.
#[derive(Debug, Clone, Serialize)]
pub struct AllocationEvent {
    /// Timestamp.
    pub timestamp: u64,
    /// Event type.
    pub event_type: AllocationEventType,
    /// Memory pointer address.
    pub ptr: usize,
    /// Size in bytes.
    pub size: usize,
    /// Variable name.
    pub var_name: Option<String>,
    /// Type name.
    pub type_name: Option<String>,
}

/// Scope event for timeline.
#[derive(Debug, Clone, Serialize)]
pub struct ScopeEvent {
    /// Timestamp.
    pub timestamp: u64,
    /// Event type.
    pub event_type: ScopeEventType,
    /// Scope name.
    pub scope_name: String,
    /// Memory usage.
    pub memory_usage: usize,
    /// Variable count.
    pub variable_count: usize,
}

/// Stack trace data for analysis.
#[derive(Debug, Clone, Serialize)]
pub struct StackTraceData {
    /// Memory allocation hotspots.
    pub hotspots: Vec<StackTraceHotspot>,
    /// Detected allocation patterns.
    pub allocation_patterns: Vec<AllocationPattern>,
    /// Total number of samples.
    pub total_samples: usize,
}

/// Stack trace hotspot.
#[derive(Debug, Clone, Serialize)]
pub struct StackTraceHotspot {
    /// Function name.
    pub function_name: String,
    /// Number of allocations.
    pub allocation_count: usize,
    /// Total bytes allocated.
    pub total_bytes: usize,
    /// Average allocation size.
    pub average_size: f64,
    /// Percentage of total allocations.
    pub percentage: f64,
}

/// Allocation pattern analysis.
#[derive(Debug, Clone, Serialize)]
pub struct AllocationPattern {
    /// Pattern type.
    pub pattern_type: String,
    /// Frequency of occurrence.
    pub frequency: usize,
    /// Total bytes allocated.
    pub total_bytes: usize,
    /// Description.
    pub description: String,
}

/// Stack frame for stack traces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Function name.
    pub function_name: String,
    /// Source file name.
    pub file_name: Option<String>,
    /// Line number in source code.
    pub line_number: Option<u32>,
    /// Module path.
    pub module_path: Option<String>,
}

/// Safety violation types.
#[derive(Debug, Clone, Serialize)]
pub enum SafetyViolation {
    /// Potential memory leak detected.
    PotentialLeak {
        /// Memory pointer.
        ptr: usize,
        /// Size in bytes.
        size: usize,
        /// Age in milliseconds.
        age_ms: u64,
        /// Description.
        description: String,
    },
    /// Use after free violation detected.
    UseAfterFree {
        /// Memory pointer.
        ptr: usize,
        /// Description.
        description: String,
    },
    /// Double free violation detected.
    DoubleFree {
        /// Memory pointer.
        ptr: usize,
        /// Description.
        description: String,
    },
    /// Buffer overflow detected.
    BufferOverflow {
        /// Memory pointer.
        ptr: usize,
        /// Size in bytes.
        size: usize,
        /// Description.
        description: String,
    },
}

/// Allocation hotspot information.
#[derive(Debug, Clone, Serialize)]
pub struct AllocationHotspot {
    /// Location information.
    pub location: HotspotLocation,
    /// Number of allocations.
    pub allocation_count: usize,
    /// Total bytes allocated.
    pub total_bytes: usize,
    /// Average allocation size.
    pub average_size: f64,
    /// Frequency of occurrence.
    pub frequency: f64,
}

/// Hotspot location information.
#[derive(Debug, Clone, Serialize)]
pub struct HotspotLocation {
    /// Function name.
    pub function_name: String,
    /// File path.
    pub file_path: Option<String>,
    /// Line number.
    pub line_number: Option<u32>,
    /// Module path.
    pub module_path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_data() {
        let timeline = TimelineData {
            time_range: TimeRange {
                start_time: 0,
                end_time: 1000,
                duration_ms: 1000,
            },
            allocation_events: vec![],
            scope_events: vec![],
            memory_snapshots: vec![],
        };

        assert_eq!(timeline.time_range.duration_ms, 1000);
    }

    #[test]
    fn test_memory_snapshot() {
        let snapshot = MemorySnapshot {
            timestamp: 100,
            total_memory: 1024 * 1024,
            active_allocations: 10,
            fragmentation_ratio: 0.1,
            top_types: vec![],
        };

        assert_eq!(snapshot.active_allocations, 10);
    }

    #[test]
    fn test_safety_violation() {
        let violation = SafetyViolation::PotentialLeak {
            ptr: 0x1000,
            size: 1024,
            age_ms: 5000,
            description: "test leak".to_string(),
        };

        assert!(matches!(violation, SafetyViolation::PotentialLeak { .. }));
    }

    #[test]
    fn test_time_range_creation() {
        let range = TimeRange {
            start_time: 0,
            end_time: 10000,
            duration_ms: 10000,
        };

        assert_eq!(range.start_time, 0);
        assert_eq!(range.end_time, 10000);
        assert_eq!(range.duration_ms, 10000);
    }

    #[test]
    fn test_allocation_event_creation() {
        let event = AllocationEvent {
            timestamp: 1000,
            event_type: AllocationEventType::Allocate,
            ptr: 0x1000,
            size: 1024,
            var_name: Some("buffer".to_string()),
            type_name: Some("Vec<u8>".to_string()),
        };

        assert_eq!(event.timestamp, 1000);
        assert_eq!(event.event_type, AllocationEventType::Allocate);
        assert_eq!(event.size, 1024);
    }

    #[test]
    fn test_scope_event_creation() {
        let event = ScopeEvent {
            timestamp: 500,
            event_type: ScopeEventType::Enter,
            scope_name: "main".to_string(),
            memory_usage: 2048,
            variable_count: 5,
        };

        assert_eq!(event.event_type, ScopeEventType::Enter);
        assert_eq!(event.scope_name, "main");
        assert_eq!(event.variable_count, 5);
    }

    #[test]
    fn test_stack_trace_data_creation() {
        let data = StackTraceData {
            hotspots: vec![],
            allocation_patterns: vec![],
            total_samples: 100,
        };

        assert_eq!(data.total_samples, 100);
        assert!(data.hotspots.is_empty());
    }

    #[test]
    fn test_stack_trace_hotspot_creation() {
        let hotspot = StackTraceHotspot {
            function_name: "allocate_buffer".to_string(),
            allocation_count: 50,
            total_bytes: 10240,
            average_size: 204.8,
            percentage: 25.0,
        };

        assert_eq!(hotspot.function_name, "allocate_buffer");
        assert_eq!(hotspot.allocation_count, 50);
    }

    #[test]
    fn test_allocation_pattern_creation() {
        let pattern = AllocationPattern {
            pattern_type: "repeated".to_string(),
            frequency: 100,
            total_bytes: 4096,
            description: "Repeated small allocations".to_string(),
        };

        assert_eq!(pattern.pattern_type, "repeated");
        assert_eq!(pattern.frequency, 100);
    }

    #[test]
    fn test_stack_frame_creation() {
        let frame = StackFrame {
            function_name: "main".to_string(),
            file_name: Some("main.rs".to_string()),
            line_number: Some(42),
            module_path: Some("myapp".to_string()),
        };

        assert_eq!(frame.function_name, "main");
        assert_eq!(frame.line_number, Some(42));
    }

    #[test]
    fn test_stack_frame_minimal() {
        let frame = StackFrame {
            function_name: "unknown".to_string(),
            file_name: None,
            line_number: None,
            module_path: None,
        };

        assert_eq!(frame.function_name, "unknown");
        assert!(frame.file_name.is_none());
    }

    #[test]
    fn test_safety_violation_use_after_free() {
        let violation = SafetyViolation::UseAfterFree {
            ptr: 0x2000,
            description: "Access after free".to_string(),
        };

        assert!(matches!(violation, SafetyViolation::UseAfterFree { .. }));
    }

    #[test]
    fn test_safety_violation_double_free() {
        let violation = SafetyViolation::DoubleFree {
            ptr: 0x3000,
            description: "Double free detected".to_string(),
        };

        assert!(matches!(violation, SafetyViolation::DoubleFree { .. }));
    }

    #[test]
    fn test_safety_violation_buffer_overflow() {
        let violation = SafetyViolation::BufferOverflow {
            ptr: 0x4000,
            size: 1024,
            description: "Buffer overflow".to_string(),
        };

        assert!(matches!(violation, SafetyViolation::BufferOverflow { .. }));
    }

    #[test]
    fn test_allocation_hotspot_creation() {
        let hotspot = AllocationHotspot {
            location: HotspotLocation {
                function_name: "process_data".to_string(),
                file_path: Some("processor.rs".to_string()),
                line_number: Some(100),
                module_path: None,
            },
            allocation_count: 200,
            total_bytes: 8192,
            average_size: 40.96,
            frequency: 0.5,
        };

        assert_eq!(hotspot.allocation_count, 200);
        assert_eq!(hotspot.location.function_name, "process_data");
    }

    #[test]
    fn test_hotspot_location_creation() {
        let location = HotspotLocation {
            function_name: "test_fn".to_string(),
            file_path: Some("test.rs".to_string()),
            line_number: Some(10),
            module_path: Some("test_module".to_string()),
        };

        assert_eq!(location.function_name, "test_fn");
        assert_eq!(location.line_number, Some(10));
    }

    #[test]
    fn test_timeline_data_with_events() {
        let alloc_event = AllocationEvent {
            timestamp: 100,
            event_type: AllocationEventType::Allocate,
            ptr: 0x1000,
            size: 512,
            var_name: None,
            type_name: None,
        };

        let scope_event = ScopeEvent {
            timestamp: 50,
            event_type: ScopeEventType::Enter,
            scope_name: "test".to_string(),
            memory_usage: 0,
            variable_count: 0,
        };

        let timeline = TimelineData {
            time_range: TimeRange {
                start_time: 0,
                end_time: 1000,
                duration_ms: 1000,
            },
            allocation_events: vec![alloc_event],
            scope_events: vec![scope_event],
            memory_snapshots: vec![],
        };

        assert_eq!(timeline.allocation_events.len(), 1);
        assert_eq!(timeline.scope_events.len(), 1);
    }

    #[test]
    fn test_memory_snapshot_with_types() {
        let type_usage = TypeMemoryUsage {
            type_name: "String".to_string(),
            total_size: 4096,
            allocation_count: 100,
            average_size: 40.96,
            peak_size: 5000,
            current_size: 4096,
            efficiency_score: 0.9,
        };

        let snapshot = MemorySnapshot {
            timestamp: 500,
            total_memory: 8192,
            active_allocations: 50,
            fragmentation_ratio: 0.15,
            top_types: vec![type_usage],
        };

        assert_eq!(snapshot.top_types.len(), 1);
        assert_eq!(snapshot.top_types[0].type_name, "String");
    }

    #[test]
    fn test_stack_trace_data_with_hotspots() {
        let hotspot = StackTraceHotspot {
            function_name: "hot_function".to_string(),
            allocation_count: 500,
            total_bytes: 20480,
            average_size: 40.96,
            percentage: 50.0,
        };

        let pattern = AllocationPattern {
            pattern_type: "burst".to_string(),
            frequency: 10,
            total_bytes: 1024,
            description: "Burst allocation pattern".to_string(),
        };

        let data = StackTraceData {
            hotspots: vec![hotspot],
            allocation_patterns: vec![pattern],
            total_samples: 1000,
        };

        assert_eq!(data.hotspots.len(), 1);
        assert_eq!(data.allocation_patterns.len(), 1);
    }
}
