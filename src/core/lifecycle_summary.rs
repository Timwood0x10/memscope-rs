//! Lifecycle summary generation for enhanced memory analysis
//!
//! This module provides functionality to generate high-level lifecycle summaries
//! from detailed ownership history, creating the data needed for lifetime.json export.

use crate::core::ownership_history::{OwnershipHistoryRecorder, OwnershipSummary, BorrowInfo, CloneInfo};
use crate::core::types::AllocationInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generator for lifecycle summaries and lifetime.json export
pub struct LifecycleSummaryGenerator {
    /// Configuration for summary generation
    config: SummaryConfig,
}

/// Configuration for lifecycle summary generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryConfig {
    /// Include detailed borrow information
    pub include_borrow_details: bool,
    /// Include clone relationship information
    pub include_clone_details: bool,
    /// Minimum lifetime threshold for inclusion (in milliseconds)
    pub min_lifetime_threshold_ms: u64,
    /// Maximum number of lifecycle events to include per allocation
    pub max_events_per_allocation: usize,
}

impl Default for SummaryConfig {
    fn default() -> Self {
        Self {
            include_borrow_details: true,
            include_clone_details: true,
            min_lifetime_threshold_ms: 0,
            max_events_per_allocation: 50,
        }
    }
}

/// Complete lifecycle data for export to lifetime.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleExportData {
    /// Lifecycle events for each allocation
    pub lifecycle_events: Vec<LifecycleEventSummary>,
    /// Variable groups for organization
    pub variable_groups: Vec<VariableGroup>,
    /// Count of user variables (those with meaningful names)
    pub user_variables_count: usize,
    /// Whether visualization data is ready
    pub visualization_ready: bool,
    /// Export metadata
    pub metadata: ExportMetadata,
}

/// Summary of lifecycle events for a single allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEventSummary {
    /// Allocation pointer
    pub allocation_ptr: usize,
    /// Variable name (if available)
    pub var_name: Option<String>,
    /// Type name
    pub type_name: Option<String>,
    /// Size of the allocation
    pub size: usize,
    /// Total lifetime in milliseconds
    pub lifetime_ms: Option<u64>,
    /// Lifecycle events
    pub events: Vec<LifecycleEvent>,
    /// High-level summary
    pub summary: AllocationLifecycleSummary,
}

/// Individual lifecycle event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    /// Event ID
    pub id: u64,
    /// Event type
    pub event_type: String,
    /// Timestamp when the event occurred
    pub timestamp: u64,
    /// Size involved in the event (if applicable)
    pub size: Option<usize>,
    /// Additional event details
    pub details: Option<String>,
}

/// High-level summary of an allocation's lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationLifecycleSummary {
    /// Total lifetime in milliseconds
    pub lifetime_ms: Option<u64>,
    /// Borrowing information
    pub borrow_info: BorrowInfo,
    /// Cloning information
    pub clone_info: CloneInfo,
    /// Whether detailed ownership history is available
    pub ownership_history_available: bool,
    /// Lifecycle pattern classification
    pub lifecycle_pattern: LifecyclePattern,
    /// Memory efficiency score (0.0 to 1.0)
    pub efficiency_score: f64,
}

/// Classification of lifecycle patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecyclePattern {
    /// Short-lived allocation (< 1ms)
    Ephemeral,
    /// Short-term allocation (1ms - 100ms)
    ShortTerm,
    /// Medium-term allocation (100ms - 10s)
    MediumTerm,
    /// Long-term allocation (> 10s)
    LongTerm,
    /// Leaked allocation (never deallocated)
    Leaked,
    /// Unknown pattern
    Unknown,
}

/// Group of related variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableGroup {
    /// Group name
    pub name: String,
    /// Variables in this group
    pub variables: Vec<String>,
    /// Total memory used by this group
    pub total_memory: usize,
    /// Average lifetime of variables in this group
    pub average_lifetime_ms: f64,
}

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// Export timestamp
    pub export_timestamp: u64,
    /// Total allocations analyzed
    pub total_allocations: usize,
    /// Total events processed
    pub total_events: usize,
    /// Analysis duration in milliseconds
    pub analysis_duration_ms: u64,
}

impl LifecycleSummaryGenerator {
    /// Create a new lifecycle summary generator
    pub fn new() -> Self {
        Self::with_config(SummaryConfig::default())
    }

    /// Create a new lifecycle summary generator with custom configuration
    pub fn with_config(config: SummaryConfig) -> Self {
        Self { config }
    }

    /// Generate complete lifecycle export data
    pub fn generate_lifecycle_export(
        &self,
        ownership_history: &OwnershipHistoryRecorder,
        allocations: &[AllocationInfo],
    ) -> LifecycleExportData {
        let start_time = std::time::Instant::now();

        // Generate lifecycle event summaries
        let lifecycle_events = self.generate_lifecycle_events(ownership_history, allocations);

        // Generate variable groups
        let variable_groups = self.generate_variable_groups(&lifecycle_events);

        // Count user variables (those with meaningful names)
        let user_variables_count = lifecycle_events
            .iter()
            .filter(|event| {
                event.var_name.as_ref()
                    .map(|name| self.is_user_variable(name))
                    .unwrap_or(false)
            })
            .count();

        let analysis_duration = start_time.elapsed().as_millis() as u64;

        LifecycleExportData {
            lifecycle_events,
            variable_groups,
            user_variables_count,
            visualization_ready: true,
            metadata: ExportMetadata {
                export_timestamp: self.get_current_timestamp(),
                total_allocations: allocations.len(),
                total_events: ownership_history.get_statistics().total_events,
                analysis_duration_ms: analysis_duration,
            },
        }
    }

    /// Generate lifecycle event summaries for all allocations
    fn generate_lifecycle_events(
        &self,
        ownership_history: &OwnershipHistoryRecorder,
        allocations: &[AllocationInfo],
    ) -> Vec<LifecycleEventSummary> {
        let mut summaries = Vec::new();

        for allocation in allocations {
            // Skip allocations below the minimum lifetime threshold
            if let Some(lifetime_ms) = allocation.lifetime_ms {
                if lifetime_ms < self.config.min_lifetime_threshold_ms {
                    continue;
                }
            }

            let summary = self.generate_single_lifecycle_summary(ownership_history, allocation);
            summaries.push(summary);
        }

        summaries
    }

    /// Generate lifecycle summary for a single allocation
    fn generate_single_lifecycle_summary(
        &self,
        ownership_history: &OwnershipHistoryRecorder,
        allocation: &AllocationInfo,
    ) -> LifecycleEventSummary {
        let ptr = allocation.ptr;

        // Get ownership summary if available
        let ownership_summary = ownership_history.get_summary(ptr);

        // Generate lifecycle events
        let events = if let Some(ownership_events) = ownership_history.get_events(ptr) {
            ownership_events
                .iter()
                .take(self.config.max_events_per_allocation)
                .map(|event| LifecycleEvent {
                    id: event.event_id,
                    event_type: self.format_event_type(&event.event_type),
                    timestamp: event.timestamp,
                    size: Some(allocation.size),
                    details: event.details.context.clone(),
                })
                .collect()
        } else {
            // Create basic events from allocation info
            let mut basic_events = vec![LifecycleEvent {
                id: 1,
                event_type: "Allocation".to_string(),
                timestamp: allocation.timestamp_alloc,
                size: Some(allocation.size),
                details: Some("Memory allocated".to_string()),
            }];

            if let Some(dealloc_time) = allocation.timestamp_dealloc {
                basic_events.push(LifecycleEvent {
                    id: 2,
                    event_type: "Deallocation".to_string(),
                    timestamp: dealloc_time,
                    size: Some(allocation.size),
                    details: Some("Memory deallocated".to_string()),
                });
            }

            basic_events
        };

        // Create allocation lifecycle summary
        let summary = if let Some(ownership_summary) = ownership_summary {
            AllocationLifecycleSummary {
                lifetime_ms: allocation.lifetime_ms,
                borrow_info: ownership_summary.borrow_info.clone(),
                clone_info: ownership_summary.clone_info.clone(),
                ownership_history_available: true,
                lifecycle_pattern: self.classify_lifecycle_pattern(allocation.lifetime_ms),
                efficiency_score: self.calculate_efficiency_score(allocation, ownership_summary),
            }
        } else {
            AllocationLifecycleSummary {
                lifetime_ms: allocation.lifetime_ms,
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
                ownership_history_available: false,
                lifecycle_pattern: self.classify_lifecycle_pattern(allocation.lifetime_ms),
                efficiency_score: 0.5, // Default score
            }
        };

        LifecycleEventSummary {
            allocation_ptr: ptr,
            var_name: allocation.var_name.clone(),
            type_name: allocation.type_name.clone(),
            size: allocation.size,
            lifetime_ms: allocation.lifetime_ms,
            events,
            summary,
        }
    }

    /// Format ownership event type for display
    fn format_event_type(&self, event_type: &crate::core::ownership_history::OwnershipEventType) -> String {
        match event_type {
            crate::core::ownership_history::OwnershipEventType::Allocated => "Allocation".to_string(),
            crate::core::ownership_history::OwnershipEventType::Cloned { .. } => "Clone".to_string(),
            crate::core::ownership_history::OwnershipEventType::Dropped => "Deallocation".to_string(),
            crate::core::ownership_history::OwnershipEventType::OwnershipTransferred { .. } => "OwnershipTransfer".to_string(),
            crate::core::ownership_history::OwnershipEventType::Borrowed { .. } => "Borrow".to_string(),
            crate::core::ownership_history::OwnershipEventType::MutablyBorrowed { .. } => "MutableBorrow".to_string(),
            crate::core::ownership_history::OwnershipEventType::BorrowReleased { .. } => "BorrowRelease".to_string(),
            crate::core::ownership_history::OwnershipEventType::RefCountChanged { .. } => "RefCountChange".to_string(),
        }
    }

    /// Classify the lifecycle pattern based on lifetime
    fn classify_lifecycle_pattern(&self, lifetime_ms: Option<u64>) -> LifecyclePattern {
        match lifetime_ms {
            None => LifecyclePattern::Leaked,
            Some(0) => LifecyclePattern::Ephemeral,
            Some(ms) if ms < 1 => LifecyclePattern::Ephemeral,
            Some(ms) if ms < 100 => LifecyclePattern::ShortTerm,
            Some(ms) if ms < 10_000 => LifecyclePattern::MediumTerm,
            Some(_) => LifecyclePattern::LongTerm,
        }
    }

    /// Calculate efficiency score for an allocation
    fn calculate_efficiency_score(&self, allocation: &AllocationInfo, ownership_summary: &OwnershipSummary) -> f64 {
        let mut score: f64 = 0.5; // Base score

        // Bonus for having a meaningful variable name
        if allocation.var_name.as_ref().map(|name| self.is_user_variable(name)).unwrap_or(false) {
            score += 0.1;
        }

        // Bonus for appropriate lifetime (not too short, not leaked)
        match self.classify_lifecycle_pattern(allocation.lifetime_ms) {
            LifecyclePattern::ShortTerm | LifecyclePattern::MediumTerm => score += 0.2,
            LifecyclePattern::Ephemeral => score -= 0.1,
            LifecyclePattern::Leaked => score -= 0.3,
            _ => {}
        }

        // Penalty for excessive borrowing
        if ownership_summary.borrow_info.max_concurrent_borrows > 5 {
            score -= 0.1;
        }

        // Bonus for being part of a clone chain (indicates reuse)
        if ownership_summary.clone_info.clone_count > 0 || ownership_summary.clone_info.is_clone {
            score += 0.1;
        }

        // Clamp score between 0.0 and 1.0
        score.max(0.0).min(1.0)
    }

    /// Check if a variable name indicates a user-defined variable
    fn is_user_variable(&self, name: &str) -> bool {
        // Filter out system-generated names
        !name.starts_with("primitive_")
            && !name.starts_with("struct_")
            && !name.starts_with("collection_")
            && !name.starts_with("buffer_")
            && !name.starts_with("system_")
            && !name.starts_with("fast_tracked")
            && name != "unknown"
    }

    /// Generate variable groups for organization
    fn generate_variable_groups(&self, lifecycle_events: &[LifecycleEventSummary]) -> Vec<VariableGroup> {
        let mut groups: HashMap<String, Vec<&LifecycleEventSummary>> = HashMap::new();

        // Group by type name
        for event in lifecycle_events {
            if let Some(ref type_name) = event.type_name {
                let group_name = self.extract_base_type_name(type_name);
                groups.entry(group_name).or_insert_with(Vec::new).push(event);
            }
        }

        // Convert to VariableGroup structs
        groups
            .into_iter()
            .map(|(name, events)| {
                let variables: Vec<String> = events
                    .iter()
                    .filter_map(|e| e.var_name.clone())
                    .collect();

                let total_memory: usize = events.iter().map(|e| e.size).sum();

                let average_lifetime_ms = if !events.is_empty() {
                    let total_lifetime: u64 = events
                        .iter()
                        .filter_map(|e| e.lifetime_ms)
                        .sum();
                    let count = events.iter().filter(|e| e.lifetime_ms.is_some()).count();
                    if count > 0 {
                        total_lifetime as f64 / count as f64
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                VariableGroup {
                    name,
                    variables,
                    total_memory,
                    average_lifetime_ms,
                }
            })
            .collect()
    }

    /// Extract base type name for grouping
    fn extract_base_type_name(&self, type_name: &str) -> String {
        // Extract the base type from complex type names
        if let Some(pos) = type_name.find('<') {
            type_name[..pos].to_string()
        } else if let Some(pos) = type_name.rfind("::") {
            type_name[pos + 2..].to_string()
        } else {
            type_name.to_string()
        }
    }

    /// Get current timestamp in nanoseconds
    fn get_current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    /// Export lifecycle data to JSON string
    pub fn export_to_json(&self, export_data: &LifecycleExportData) -> serde_json::Result<String> {
        serde_json::to_string_pretty(export_data)
    }
}

impl Default for LifecycleSummaryGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ownership_history::{OwnershipHistoryRecorder, OwnershipEventType};

    fn create_test_allocation(ptr: usize, size: usize, var_name: Option<String>) -> AllocationInfo {
        AllocationInfo {
            ptr,
            size,
            var_name,
            type_name: Some("Vec<i32>".to_string()),
            scope_name: Some("test".to_string()),
            timestamp_alloc: 1000,
            timestamp_dealloc: Some(2000),
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: Some(1000),
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        }
    }

    #[test]
    fn test_lifecycle_summary_generator_creation() {
        let generator = LifecycleSummaryGenerator::new();
        assert!(generator.config.include_borrow_details);
        assert!(generator.config.include_clone_details);
    }

    #[test]
    fn test_lifecycle_pattern_classification() {
        let generator = LifecycleSummaryGenerator::new();

        assert!(matches!(generator.classify_lifecycle_pattern(None), LifecyclePattern::Leaked));
        assert!(matches!(generator.classify_lifecycle_pattern(Some(0)), LifecyclePattern::Ephemeral));
        assert!(matches!(generator.classify_lifecycle_pattern(Some(50)), LifecyclePattern::ShortTerm));
        assert!(matches!(generator.classify_lifecycle_pattern(Some(500)), LifecyclePattern::MediumTerm));
        assert!(matches!(generator.classify_lifecycle_pattern(Some(15000)), LifecyclePattern::LongTerm));
    }

    #[test]
    fn test_user_variable_detection() {
        let generator = LifecycleSummaryGenerator::new();

        assert!(generator.is_user_variable("my_vec"));
        assert!(generator.is_user_variable("user_data"));
        assert!(!generator.is_user_variable("primitive_data"));
        assert!(!generator.is_user_variable("system_type_64bytes"));
        assert!(!generator.is_user_variable("fast_tracked"));
    }

    #[test]
    fn test_base_type_extraction() {
        let generator = LifecycleSummaryGenerator::new();

        assert_eq!(generator.extract_base_type_name("Vec<i32>"), "Vec");
        assert_eq!(generator.extract_base_type_name("std::collections::HashMap<K,V>"), "HashMap");
        assert_eq!(generator.extract_base_type_name("String"), "String");
    }

    #[test]
    fn test_lifecycle_export_generation() {
        let generator = LifecycleSummaryGenerator::new();
        let mut ownership_history = OwnershipHistoryRecorder::new();
        
        // Create test allocation
        let allocation = create_test_allocation(0x1000, 64, Some("test_var".to_string()));
        
        // Record some ownership events
        ownership_history.record_event(0x1000, OwnershipEventType::Allocated, 1);
        ownership_history.record_event(0x1000, OwnershipEventType::Dropped, 2);
        
        let allocations = vec![allocation];
        let export_data = generator.generate_lifecycle_export(&ownership_history, &allocations);
        
        assert_eq!(export_data.lifecycle_events.len(), 1);
        assert_eq!(export_data.user_variables_count, 1);
        assert!(export_data.visualization_ready);
        assert!(export_data.metadata.total_allocations > 0);
    }

    #[test]
    fn test_variable_grouping() {
        let generator = LifecycleSummaryGenerator::new();
        
        let events = vec![
            LifecycleEventSummary {
                allocation_ptr: 0x1000,
                var_name: Some("vec1".to_string()),
                type_name: Some("Vec<i32>".to_string()),
                size: 64,
                lifetime_ms: Some(1000),
                events: Vec::new(),
                summary: AllocationLifecycleSummary {
                    lifetime_ms: Some(1000),
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
                    ownership_history_available: false,
                    lifecycle_pattern: LifecyclePattern::MediumTerm,
                    efficiency_score: 0.5,
                },
            },
            LifecycleEventSummary {
                allocation_ptr: 0x2000,
                var_name: Some("vec2".to_string()),
                type_name: Some("Vec<f64>".to_string()),
                size: 128,
                lifetime_ms: Some(2000),
                events: Vec::new(),
                summary: AllocationLifecycleSummary {
                    lifetime_ms: Some(2000),
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
                    ownership_history_available: false,
                    lifecycle_pattern: LifecyclePattern::MediumTerm,
                    efficiency_score: 0.5,
                },
            },
        ];
        
        let groups = generator.generate_variable_groups(&events);
        
        assert_eq!(groups.len(), 1); // Both should be grouped under "Vec"
        assert_eq!(groups[0].name, "Vec");
        assert_eq!(groups[0].variables.len(), 2);
        assert_eq!(groups[0].total_memory, 192);
    }

    #[test]
    fn test_json_export() {
        let generator = LifecycleSummaryGenerator::new();
        let ownership_history = OwnershipHistoryRecorder::new();
        let allocations = vec![create_test_allocation(0x1000, 64, Some("test".to_string()))];
        
        let export_data = generator.generate_lifecycle_export(&ownership_history, &allocations);
        let json = generator.export_to_json(&export_data).unwrap();
        
        assert!(json.contains("lifecycle_events"));
        assert!(json.contains("variable_groups"));
        assert!(json.contains("user_variables_count"));
        assert!(json.contains("visualization_ready"));
    }
}