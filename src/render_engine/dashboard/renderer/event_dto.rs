//! Lightweight event transfer objects for dashboard serialization.
//!
//! These DTOs avoid exposing the full internal `MemoryEvent` struct to
//! the frontend while still providing enough detail for time-travel,
//! filtering, and correlation views.

use serde::{Deserialize, Serialize};

/// A lightweight, serializable view of a `MemoryEvent` for dashboard use.
///
/// Contains only the fields needed by the frontend for timeline,
/// allocation, and correlation views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardEventDTO {
    /// Event timestamp (nanoseconds since epoch)
    pub timestamp: u64,
    /// Event type as a string: "Allocate", "Deallocate", "Reallocate", "Move", "Clone"
    pub event_type: String,
    /// Memory pointer address (formatted as hex string)
    pub ptr: String,
    /// Allocation size in bytes
    pub size: usize,
    /// Thread identifier
    pub thread_id: u64,
    /// Optional task identifier
    pub task_id: Option<u64>,
    /// Optional variable name
    pub var_name: Option<String>,
    /// Optional type name
    pub type_name: Option<String>,
    /// Optional source file path
    pub source_file: Option<String>,
    /// Optional source line number
    pub source_line: Option<u32>,
}

impl From<&crate::event_store::event::MemoryEvent> for DashboardEventDTO {
    fn from(e: &crate::event_store::event::MemoryEvent) -> Self {
        Self {
            timestamp: e.timestamp,
            event_type: e.event_type.to_string(),
            ptr: format!("0x{:x}", e.ptr),
            size: e.size,
            thread_id: e.thread_id,
            task_id: e.task_id,
            var_name: e.var_name.clone(),
            type_name: e.type_name.clone(),
            source_file: e.source_file.clone(),
            source_line: e.source_line,
        }
    }
}

/// Frontend data index for cross-referencing allocations by various keys.
///
/// This index is built once during dashboard initialization and embedded
/// in the JSON payload so the frontend can answer queries without
/// linear scans.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataIndex {
    /// Maps pointer hex-string to list of allocation indices
    pub ptr_to_allocation: std::collections::HashMap<String, Vec<usize>>,
    /// Maps thread ID to list of allocation indices
    pub thread_id_to_allocations: std::collections::HashMap<u64, Vec<usize>>,
    /// Maps thread ID to list of event indices
    pub thread_id_to_events: std::collections::HashMap<u64, Vec<usize>>,
    /// Maps type name to list of allocation indices
    pub type_name_to_allocations: std::collections::HashMap<String, Vec<usize>>,
    /// Maps variable name to list of allocation indices
    pub var_name_to_allocations: std::collections::HashMap<String, Vec<usize>>,
    /// Maps allocation pointer to passport index
    pub allocation_ptr_to_passport: std::collections::HashMap<String, Vec<usize>>,
    /// Maps allocation pointer to unsafe report index
    pub allocation_ptr_to_unsafe_reports: std::collections::HashMap<String, Vec<usize>>,
    /// Maps source location "file:line" to list of allocation indices
    pub source_location_to_allocations: std::collections::HashMap<String, Vec<usize>>,
}

/// Summary of event data included in the dashboard JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    /// Total number of raw events in the event store
    pub total_event_count: usize,
    /// Number of events included in the dashboard payload
    pub exported_event_count: usize,
    /// Whether a subset of events was exported
    pub is_sampled: bool,
    /// Description of the sampling strategy
    pub sampling_strategy: String,
    /// Total number of allocations (from event reconstruction)
    pub total_allocations: usize,
    /// Number of active allocations
    pub active_allocations: usize,
}

impl EventSummary {
    pub fn new(
        total_event_count: usize,
        exported_event_count: usize,
        total_allocations: usize,
        active_allocations: usize,
    ) -> Self {
        let is_sampled = exported_event_count < total_event_count;
        let strategy = if is_sampled {
            format!("truncated to {} events", exported_event_count)
        } else {
            "complete".to_string()
        };
        Self {
            total_event_count,
            exported_event_count,
            is_sampled,
            sampling_strategy: strategy,
            total_allocations,
            active_allocations,
        }
    }
}

/// Build a `DataIndex` from allocation info and events.
///
/// This is called once during dashboard initialization and the result
/// is cached in the JSON payload so the frontend can use it immediately.
pub fn build_data_index(
    allocations: &[super::types::AllocationInfo],
    events: &[DashboardEventDTO],
    passport_details: &[super::types::PassportDetail],
    unsafe_reports: &[super::types::UnsafeReport],
) -> DataIndex {
    use std::collections::HashMap;

    let mut ptr_to_allocation: HashMap<String, Vec<usize>> = HashMap::new();
    let mut thread_id_to_allocations: HashMap<u64, Vec<usize>> = HashMap::new();
    let mut thread_id_to_events: HashMap<u64, Vec<usize>> = HashMap::new();
    let mut type_name_to_allocations: HashMap<String, Vec<usize>> = HashMap::new();
    let mut var_name_to_allocations: HashMap<String, Vec<usize>> = HashMap::new();
    let mut allocation_ptr_to_passport: HashMap<String, Vec<usize>> = HashMap::new();
    let mut allocation_ptr_to_unsafe_reports: HashMap<String, Vec<usize>> = HashMap::new();
    let mut source_location_to_allocations: HashMap<String, Vec<usize>> = HashMap::new();

    for (idx, alloc) in allocations.iter().enumerate() {
        ptr_to_allocation
            .entry(alloc.address.clone())
            .or_default()
            .push(idx);

        // Parse thread_id from "Thread-N" format back to numeric
        if let Some(tid) = parse_thread_id(&alloc.thread_id) {
            thread_id_to_allocations.entry(tid).or_default().push(idx);
        }

        let tn = alloc.type_name.to_lowercase();
        if !tn.is_empty() && tn != "unknown" {
            type_name_to_allocations
                .entry(alloc.type_name.clone())
                .or_default()
                .push(idx);
        }

        let vn = alloc.var_name.to_lowercase();
        if !vn.is_empty() && vn != "unknown" {
            var_name_to_allocations
                .entry(alloc.var_name.clone())
                .or_default()
                .push(idx);
        }

        if let (Some(ref file), Some(line)) = (&alloc.source_file, alloc.source_line) {
            let loc = format!("{}:{}", file, line);
            source_location_to_allocations
                .entry(loc)
                .or_default()
                .push(idx);
        } else if let Some(ref file) = &alloc.source_file {
            let loc = format!("{}:0", file);
            source_location_to_allocations
                .entry(loc)
                .or_default()
                .push(idx);
        }
    }

    for (idx, event) in events.iter().enumerate() {
        thread_id_to_events
            .entry(event.thread_id)
            .or_default()
            .push(idx);
    }

    for (idx, passport) in passport_details.iter().enumerate() {
        allocation_ptr_to_passport
            .entry(passport.allocation_ptr.clone())
            .or_default()
            .push(idx);
    }

    for (idx, report) in unsafe_reports.iter().enumerate() {
        allocation_ptr_to_unsafe_reports
            .entry(report.allocation_ptr.clone())
            .or_default()
            .push(idx);
    }

    DataIndex {
        ptr_to_allocation,
        thread_id_to_allocations,
        thread_id_to_events,
        type_name_to_allocations,
        var_name_to_allocations,
        allocation_ptr_to_passport,
        allocation_ptr_to_unsafe_reports,
        source_location_to_allocations,
    }
}

/// Parse a "Thread-N" format string back to a numeric thread ID.
fn parse_thread_id(raw: &str) -> Option<u64> {
    if let Some(stripped) = raw.strip_prefix("Thread-") {
        stripped.parse::<u64>().ok()
    } else {
        raw.parse::<u64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify that DashboardEventDTO is correctly constructed
    /// from a MemoryEvent.
    /// Invariants: All relevant fields are preserved in the DTO.
    #[test]
    fn test_dto_from_memory_event() {
        use crate::event_store::event::MemoryEvent;
        let mut event = MemoryEvent::allocate(0x1000, 64, 1);
        event.var_name = Some("buf".to_string());
        event.type_name = Some("[u8; 64]".to_string());
        event.source_file = Some("src/main.rs".to_string());
        event.source_line = Some(42);
        event.task_id = Some(7);

        let dto = DashboardEventDTO::from(&event);

        assert_eq!(dto.ptr, "0x1000");
        assert_eq!(dto.size, 64);
        assert_eq!(dto.thread_id, 1);
        assert_eq!(dto.var_name.as_deref(), Some("buf"));
        assert_eq!(dto.event_type, "Allocate");
        assert_eq!(dto.task_id, Some(7));
        assert_eq!(dto.source_file.as_deref(), Some("src/main.rs"));
        assert_eq!(dto.source_line, Some(42));
    }

    /// Objective: Verify that EventSummary correctly reports complete
    /// vs sampled data.
    /// Invariants: When all events are exported, is_sampled is false.
    #[test]
    fn test_event_summary_complete() {
        let s = EventSummary::new(100, 100, 50, 25);
        assert!(!s.is_sampled);
        assert_eq!(s.total_event_count, 100);
        assert_eq!(s.exported_event_count, 100);
    }

    /// Objective: Verify that EventSummary correctly reports truncated data.
    /// Invariants: When fewer events are exported, is_sampled is true.
    #[test]
    fn test_event_summary_sampled() {
        let s = EventSummary::new(1000, 100, 500, 200);
        assert!(s.is_sampled);
        assert_eq!(s.total_event_count, 1000);
        assert_eq!(s.exported_event_count, 100);
    }

    /// Objective: Verify that build_data_index creates indexes correctly
    /// from allocation info.
    /// Invariants: Each allocation is indexed by its address, type, var name.
    #[test]
    fn test_build_data_index_with_allocations() {
        use crate::render_engine::dashboard::renderer::types::AllocationInfo;

        let alloc = AllocationInfo {
            address: "0x1000".to_string(),
            type_name: "Vec<u8>".to_string(),
            size: 64,
            var_name: "buffer".to_string(),
            timestamp: "0".to_string(),
            thread_id: "Thread-1".to_string(),
            immutable_borrows: 0,
            mutable_borrows: 0,
            is_clone: false,
            clone_count: 0,
            timestamp_alloc: 1000,
            timestamp_dealloc: 0,
            lifetime_ms: 0.0,
            is_leaked: false,
            allocation_type: "heap".to_string(),
            is_smart_pointer: false,
            smart_pointer_type: String::new(),
            source_file: Some("src/main.rs".to_string()),
            source_line: Some(42),
            module_path: None,
            generation_id: 0,
            provenance: String::new(),
            evidence: Default::default(),
            confidence: Default::default(),
            layout_snapshot: None,
        };

        let index = build_data_index(&[alloc], &[], &[], &[]);

        assert!(index.ptr_to_allocation.contains_key("0x1000"));
        assert!(index.type_name_to_allocations.contains_key("Vec<u8>"));
        assert!(index.var_name_to_allocations.contains_key("buffer"));
        assert!(index
            .source_location_to_allocations
            .contains_key("src/main.rs:42"));
    }

    /// Objective: Verify that build_data_index handles empty inputs.
    /// Invariants: No panic, all index maps are empty.
    #[test]
    fn test_build_data_index_empty() {
        let index = build_data_index(&[], &[], &[], &[]);
        assert!(index.ptr_to_allocation.is_empty());
        assert!(index.thread_id_to_allocations.is_empty());
        assert!(index.type_name_to_allocations.is_empty());
    }

    /// Objective: Verify that parse_thread_id handles various formats.
    /// Invariants: "Thread-N" returns Some(N), plain numbers return Some(n),
    /// arbitrary strings return None.
    #[test]
    fn test_parse_thread_id() {
        assert_eq!(parse_thread_id("Thread-1"), Some(1));
        assert_eq!(parse_thread_id("Thread-42"), Some(42));
        assert_eq!(parse_thread_id("42"), Some(42));
        assert_eq!(parse_thread_id("main"), None);
    }

    /// Objective: Verify that DTO serialization round-trips correctly.
    /// Invariants: Serialize + deserialize preserves all fields.
    #[test]
    fn test_dto_serialization_roundtrip() {
        let dto = DashboardEventDTO {
            timestamp: 1234,
            event_type: "Allocate".to_string(),
            ptr: "0x1000".to_string(),
            size: 64,
            thread_id: 1,
            task_id: Some(7),
            var_name: Some("buf".to_string()),
            type_name: Some("[u8]".to_string()),
            source_file: Some("src/lib.rs".to_string()),
            source_line: Some(10),
        };

        let json = serde_json::to_string(&dto).unwrap();
        let deserialized: DashboardEventDTO = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.ptr, "0x1000");
        assert_eq!(deserialized.event_type, "Allocate");
        assert_eq!(deserialized.task_id, Some(7));
        assert_eq!(deserialized.source_line, Some(10));
    }
}
