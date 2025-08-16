//! Ownership history tracking system
//!
//! This module provides detailed tracking of ownership events for memory allocations,
//! including cloning, borrowing, ownership transfers, and lifetime analysis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global event ID generator
static EVENT_ID_GENERATOR: AtomicU64 = AtomicU64::new(1);

/// Ownership history recorder for tracking detailed ownership events
pub struct OwnershipHistoryRecorder {
    /// Map from allocation pointer to its ownership events
    ownership_events: HashMap<usize, Vec<OwnershipEvent>>,
    /// Map from allocation pointer to its current ownership summary
    ownership_summaries: HashMap<usize, OwnershipSummary>,
    /// Configuration for history recording
    config: HistoryConfig,
}

/// Configuration for ownership history recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// Maximum number of events to keep per allocation
    pub max_events_per_allocation: usize,
    /// Enable detailed borrowing tracking
    pub track_borrowing: bool,
    /// Enable clone relationship tracking
    pub track_cloning: bool,
    /// Enable ownership transfer tracking
    pub track_ownership_transfers: bool,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_events_per_allocation: 100,
            track_borrowing: true,
            track_cloning: true,
            track_ownership_transfers: true,
        }
    }
}

/// A single ownership event in the history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipEvent {
    /// Unique event ID
    pub event_id: u64,
    /// Timestamp when the event occurred
    pub timestamp: u64,
    /// Type of ownership event
    pub event_type: OwnershipEventType,
    /// ID of the call stack that triggered this event
    pub source_stack_id: u32,
    /// Additional details specific to the event type
    pub details: OwnershipEventDetails,
}

/// Types of ownership events that can occur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnershipEventType {
    /// Initial allocation of memory
    Allocated,
    /// Memory was cloned from another allocation
    Cloned { source_ptr: usize },
    /// Memory was dropped/deallocated
    Dropped,
    /// Ownership was transferred to another variable
    OwnershipTransferred { target_var: String },
    /// Memory was immutably borrowed
    Borrowed { borrower_scope: String },
    /// Memory was mutably borrowed
    MutablyBorrowed { borrower_scope: String },
    /// Borrow was released
    BorrowReleased { borrower_scope: String },
    /// Reference count changed (for Rc/Arc)
    RefCountChanged { old_count: usize, new_count: usize },
}

/// Additional details for ownership events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipEventDetails {
    /// Optional clone source pointer (for Cloned events)
    pub clone_source_ptr: Option<usize>,
    /// Optional target variable name (for OwnershipTransferred events)
    pub transfer_target_var: Option<String>,
    /// Optional borrower scope (for borrow events)
    pub borrower_scope: Option<String>,
    /// Optional reference count information
    pub ref_count_info: Option<RefCountInfo>,
    /// Optional additional context
    pub context: Option<String>,
}

/// Reference count information for smart pointers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefCountInfo {
    pub strong_count: usize,
    pub weak_count: usize,
    pub data_ptr: usize,
}

/// High-level ownership summary for an allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipSummary {
    /// Pointer to the allocation
    pub allocation_ptr: usize,
    /// Total lifetime in milliseconds (if known)
    pub lifetime_ms: Option<u64>,
    /// Borrowing information
    pub borrow_info: BorrowInfo,
    /// Cloning information
    pub clone_info: CloneInfo,
    /// Whether detailed ownership history is available
    pub ownership_history_available: bool,
    /// Total number of ownership events
    pub total_events: usize,
}

/// Detailed borrowing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowInfo {
    /// Total number of immutable borrows during lifetime
    pub immutable_borrows: u32,
    /// Total number of mutable borrows during lifetime
    pub mutable_borrows: u32,
    /// Maximum number of concurrent borrows observed
    pub max_concurrent_borrows: u32,
    /// Timestamp of the last borrow
    pub last_borrow_timestamp: Option<u64>,
    /// Currently active borrows
    pub active_borrows: Vec<ActiveBorrow>,
}

/// Information about an active borrow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveBorrow {
    pub borrower_scope: String,
    pub borrow_type: BorrowType,
    pub start_timestamp: u64,
}

/// Type of borrow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BorrowType {
    Immutable,
    Mutable,
}

/// Detailed cloning information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneInfo {
    /// Number of times this allocation was cloned
    pub clone_count: u32,
    /// Whether this allocation is itself a clone
    pub is_clone: bool,
    /// Pointer to the original allocation (if this is a clone)
    pub original_ptr: Option<usize>,
    /// List of pointers that were cloned from this allocation
    pub cloned_ptrs: Vec<usize>,
}

impl OwnershipHistoryRecorder {
    /// Create a new ownership history recorder
    pub fn new() -> Self {
        Self::with_config(HistoryConfig::default())
    }

    /// Create a new ownership history recorder with custom configuration
    pub fn with_config(config: HistoryConfig) -> Self {
        Self {
            ownership_events: HashMap::new(),
            ownership_summaries: HashMap::new(),
            config,
        }
    }

    /// Record a new ownership event
    pub fn record_event(&mut self, ptr: usize, event_type: OwnershipEventType, source_stack_id: u32) {
        let event_id = EVENT_ID_GENERATOR.fetch_add(1, Ordering::Relaxed);
        let timestamp = self.get_current_timestamp();

        let details = self.create_event_details(&event_type);

        let event = OwnershipEvent {
            event_id,
            timestamp,
            event_type: event_type.clone(),
            source_stack_id,
            details,
        };

        // Add event to history
        let events = self.ownership_events.entry(ptr).or_insert_with(Vec::new);
        events.push(event);

        // Limit the number of events per allocation
        if events.len() > self.config.max_events_per_allocation {
            events.remove(0); // Remove oldest event
        }

        // Update ownership summary
        self.update_ownership_summary(ptr, &event_type, timestamp);
    }

    /// Create event details based on event type
    fn create_event_details(&self, event_type: &OwnershipEventType) -> OwnershipEventDetails {
        match event_type {
            OwnershipEventType::Cloned { source_ptr } => OwnershipEventDetails {
                clone_source_ptr: Some(*source_ptr),
                transfer_target_var: None,
                borrower_scope: None,
                ref_count_info: None,
                context: Some("Memory cloned from another allocation".to_string()),
            },
            OwnershipEventType::OwnershipTransferred { target_var } => OwnershipEventDetails {
                clone_source_ptr: None,
                transfer_target_var: Some(target_var.clone()),
                borrower_scope: None,
                ref_count_info: None,
                context: Some("Ownership transferred to another variable".to_string()),
            },
            OwnershipEventType::Borrowed { borrower_scope } => OwnershipEventDetails {
                clone_source_ptr: None,
                transfer_target_var: None,
                borrower_scope: Some(borrower_scope.clone()),
                ref_count_info: None,
                context: Some("Memory immutably borrowed".to_string()),
            },
            OwnershipEventType::MutablyBorrowed { borrower_scope } => OwnershipEventDetails {
                clone_source_ptr: None,
                transfer_target_var: None,
                borrower_scope: Some(borrower_scope.clone()),
                ref_count_info: None,
                context: Some("Memory mutably borrowed".to_string()),
            },
            OwnershipEventType::BorrowReleased { borrower_scope } => OwnershipEventDetails {
                clone_source_ptr: None,
                transfer_target_var: None,
                borrower_scope: Some(borrower_scope.clone()),
                ref_count_info: None,
                context: Some("Borrow released".to_string()),
            },
            OwnershipEventType::RefCountChanged { old_count, new_count } => OwnershipEventDetails {
                clone_source_ptr: None,
                transfer_target_var: None,
                borrower_scope: None,
                ref_count_info: Some(RefCountInfo {
                    strong_count: *new_count,
                    weak_count: 0, // Would need to be provided separately
                    data_ptr: 0,   // Would need to be provided separately
                }),
                context: Some(format!("Reference count changed from {} to {}", old_count, new_count)),
            },
            _ => OwnershipEventDetails {
                clone_source_ptr: None,
                transfer_target_var: None,
                borrower_scope: None,
                ref_count_info: None,
                context: None,
            },
        }
    }

    /// Update the ownership summary for an allocation
    fn update_ownership_summary(&mut self, ptr: usize, event_type: &OwnershipEventType, timestamp: u64) {
        let summary = self.ownership_summaries.entry(ptr).or_insert_with(|| OwnershipSummary {
            allocation_ptr: ptr,
            lifetime_ms: None,
            borrow_info: BorrowInfo {
                immutable_borrows: 0,
                mutable_borrows: 0,
                max_concurrent_borrows: 0,
                last_borrow_timestamp: None,
                active_borrows: Vec::new(),
            },
            clone_info: CloneInfo {
                clone_count: 0,
                is_clone: false,
                original_ptr: None,
                cloned_ptrs: Vec::new(),
            },
            ownership_history_available: true,
            total_events: 0,
        });

        summary.total_events += 1;

        match event_type {
            OwnershipEventType::Borrowed { borrower_scope } => {
                summary.borrow_info.immutable_borrows += 1;
                summary.borrow_info.last_borrow_timestamp = Some(timestamp);
                summary.borrow_info.active_borrows.push(ActiveBorrow {
                    borrower_scope: borrower_scope.clone(),
                    borrow_type: BorrowType::Immutable,
                    start_timestamp: timestamp,
                });
                summary.borrow_info.max_concurrent_borrows = 
                    summary.borrow_info.max_concurrent_borrows.max(summary.borrow_info.active_borrows.len() as u32);
            },
            OwnershipEventType::MutablyBorrowed { borrower_scope } => {
                summary.borrow_info.mutable_borrows += 1;
                summary.borrow_info.last_borrow_timestamp = Some(timestamp);
                summary.borrow_info.active_borrows.push(ActiveBorrow {
                    borrower_scope: borrower_scope.clone(),
                    borrow_type: BorrowType::Mutable,
                    start_timestamp: timestamp,
                });
                summary.borrow_info.max_concurrent_borrows = 
                    summary.borrow_info.max_concurrent_borrows.max(summary.borrow_info.active_borrows.len() as u32);
            },
            OwnershipEventType::BorrowReleased { borrower_scope } => {
                // Remove the corresponding active borrow
                summary.borrow_info.active_borrows.retain(|borrow| borrow.borrower_scope != *borrower_scope);
            },
            OwnershipEventType::Cloned { source_ptr } => {
                summary.clone_info.is_clone = true;
                summary.clone_info.original_ptr = Some(*source_ptr);
                
                // Update the source allocation's clone info
                if let Some(source_summary) = self.ownership_summaries.get_mut(source_ptr) {
                    source_summary.clone_info.clone_count += 1;
                    source_summary.clone_info.cloned_ptrs.push(ptr);
                }
            },
            _ => {
                // Other events don't need special summary updates
            }
        }
    }

    /// Get ownership events for a specific allocation
    pub fn get_events(&self, ptr: usize) -> Option<&Vec<OwnershipEvent>> {
        self.ownership_events.get(&ptr)
    }

    /// Get ownership summary for a specific allocation
    pub fn get_summary(&self, ptr: usize) -> Option<&OwnershipSummary> {
        self.ownership_summaries.get(&ptr)
    }

    /// Get all ownership summaries
    pub fn get_all_summaries(&self) -> &HashMap<usize, OwnershipSummary> {
        &self.ownership_summaries
    }

    /// Export ownership history to JSON format
    pub fn export_to_json(&self) -> serde_json::Result<String> {
        let export_data = OwnershipHistoryExport {
            summaries: self.ownership_summaries.clone(),
            detailed_events: self.ownership_events.clone(),
            export_timestamp: self.get_current_timestamp(),
            config: self.config.clone(),
        };
        
        serde_json::to_string_pretty(&export_data)
    }

    /// Clear all ownership history
    pub fn clear(&mut self) {
        self.ownership_events.clear();
        self.ownership_summaries.clear();
    }

    /// Get statistics about the ownership history
    pub fn get_statistics(&self) -> OwnershipStatistics {
        let total_allocations = self.ownership_summaries.len();
        let total_events = self.ownership_events.values().map(|events| events.len()).sum();
        
        let mut event_type_counts = HashMap::new();
        for events in self.ownership_events.values() {
            for event in events {
                let event_type_name = match &event.event_type {
                    OwnershipEventType::Allocated => "Allocated",
                    OwnershipEventType::Cloned { .. } => "Cloned",
                    OwnershipEventType::Dropped => "Dropped",
                    OwnershipEventType::OwnershipTransferred { .. } => "OwnershipTransferred",
                    OwnershipEventType::Borrowed { .. } => "Borrowed",
                    OwnershipEventType::MutablyBorrowed { .. } => "MutablyBorrowed",
                    OwnershipEventType::BorrowReleased { .. } => "BorrowReleased",
                    OwnershipEventType::RefCountChanged { .. } => "RefCountChanged",
                };
                *event_type_counts.entry(event_type_name.to_string()).or_insert(0) += 1;
            }
        }

        let cloned_allocations = self.ownership_summaries.values()
            .filter(|summary| summary.clone_info.is_clone)
            .count();

        let allocations_with_borrows = self.ownership_summaries.values()
            .filter(|summary| summary.borrow_info.immutable_borrows > 0 || summary.borrow_info.mutable_borrows > 0)
            .count();

        OwnershipStatistics {
            total_allocations,
            total_events,
            event_type_counts,
            cloned_allocations,
            allocations_with_borrows,
            average_events_per_allocation: if total_allocations > 0 { 
                total_events as f64 / total_allocations as f64 
            } else { 
                0.0 
            },
        }
    }

    /// Get current timestamp in nanoseconds
    fn get_current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}

impl Default for OwnershipHistoryRecorder {
    fn default() -> Self {
        Self::new()
    }
}

/// Export format for ownership history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipHistoryExport {
    pub summaries: HashMap<usize, OwnershipSummary>,
    pub detailed_events: HashMap<usize, Vec<OwnershipEvent>>,
    pub export_timestamp: u64,
    pub config: HistoryConfig,
}

/// Statistics about ownership history
#[derive(Debug, Clone, Serialize)]
pub struct OwnershipStatistics {
    pub total_allocations: usize,
    pub total_events: usize,
    pub event_type_counts: HashMap<String, usize>,
    pub cloned_allocations: usize,
    pub allocations_with_borrows: usize,
    pub average_events_per_allocation: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_history_recorder_creation() {
        let recorder = OwnershipHistoryRecorder::new();
        assert_eq!(recorder.ownership_events.len(), 0);
        assert_eq!(recorder.ownership_summaries.len(), 0);
    }

    #[test]
    fn test_record_allocation_event() {
        let mut recorder = OwnershipHistoryRecorder::new();
        let ptr = 0x1000;
        
        recorder.record_event(ptr, OwnershipEventType::Allocated, 1);
        
        let events = recorder.get_events(ptr).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, OwnershipEventType::Allocated));
        
        let summary = recorder.get_summary(ptr).unwrap();
        assert_eq!(summary.allocation_ptr, ptr);
        assert_eq!(summary.total_events, 1);
    }

    #[test]
    fn test_record_clone_event() {
        let mut recorder = OwnershipHistoryRecorder::new();
        let source_ptr = 0x1000;
        let clone_ptr = 0x2000;
        
        // Record allocation for source
        recorder.record_event(source_ptr, OwnershipEventType::Allocated, 1);
        
        // Record clone event
        recorder.record_event(clone_ptr, OwnershipEventType::Cloned { source_ptr }, 2);
        
        let clone_summary = recorder.get_summary(clone_ptr).unwrap();
        assert!(clone_summary.clone_info.is_clone);
        assert_eq!(clone_summary.clone_info.original_ptr, Some(source_ptr));
        
        let source_summary = recorder.get_summary(source_ptr).unwrap();
        assert_eq!(source_summary.clone_info.clone_count, 1);
        assert!(source_summary.clone_info.cloned_ptrs.contains(&clone_ptr));
    }

    #[test]
    fn test_record_borrow_events() {
        let mut recorder = OwnershipHistoryRecorder::new();
        let ptr = 0x1000;
        
        recorder.record_event(ptr, OwnershipEventType::Allocated, 1);
        recorder.record_event(ptr, OwnershipEventType::Borrowed { borrower_scope: "scope1".to_string() }, 2);
        recorder.record_event(ptr, OwnershipEventType::MutablyBorrowed { borrower_scope: "scope2".to_string() }, 3);
        
        let summary = recorder.get_summary(ptr).unwrap();
        assert_eq!(summary.borrow_info.immutable_borrows, 1);
        assert_eq!(summary.borrow_info.mutable_borrows, 1);
        assert_eq!(summary.borrow_info.max_concurrent_borrows, 2);
        assert_eq!(summary.borrow_info.active_borrows.len(), 2);
    }

    #[test]
    fn test_borrow_release() {
        let mut recorder = OwnershipHistoryRecorder::new();
        let ptr = 0x1000;
        
        recorder.record_event(ptr, OwnershipEventType::Allocated, 1);
        recorder.record_event(ptr, OwnershipEventType::Borrowed { borrower_scope: "scope1".to_string() }, 2);
        recorder.record_event(ptr, OwnershipEventType::BorrowReleased { borrower_scope: "scope1".to_string() }, 3);
        
        let summary = recorder.get_summary(ptr).unwrap();
        assert_eq!(summary.borrow_info.immutable_borrows, 1);
        assert_eq!(summary.borrow_info.active_borrows.len(), 0);
    }

    #[test]
    fn test_event_limit() {
        let config = HistoryConfig {
            max_events_per_allocation: 3,
            ..Default::default()
        };
        let mut recorder = OwnershipHistoryRecorder::with_config(config);
        let ptr = 0x1000;
        
        // Record more events than the limit
        for i in 0..5 {
            recorder.record_event(ptr, OwnershipEventType::Allocated, i as u32);
        }
        
        let events = recorder.get_events(ptr).unwrap();
        assert_eq!(events.len(), 3); // Should be limited to 3
    }

    #[test]
    fn test_statistics() {
        let mut recorder = OwnershipHistoryRecorder::new();
        
        recorder.record_event(0x1000, OwnershipEventType::Allocated, 1);
        recorder.record_event(0x2000, OwnershipEventType::Cloned { source_ptr: 0x1000 }, 2);
        recorder.record_event(0x1000, OwnershipEventType::Borrowed { borrower_scope: "scope1".to_string() }, 3);
        
        let stats = recorder.get_statistics();
        assert_eq!(stats.total_allocations, 2);
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.cloned_allocations, 1);
        assert_eq!(stats.allocations_with_borrows, 1);
    }

    #[test]
    fn test_json_export() {
        let mut recorder = OwnershipHistoryRecorder::new();
        recorder.record_event(0x1000, OwnershipEventType::Allocated, 1);
        
        let json = recorder.export_to_json().unwrap();
        assert!(json.contains("summaries"));
        assert!(json.contains("detailed_events"));
        assert!(json.contains("export_timestamp"));
    }
}