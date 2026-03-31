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
}
