//! Lifecycle summary generation for enhanced memory analysis
//!
//! This module provides functionality to generate high-level lifecycle summaries
//! from detailed ownership history, creating the data needed for lifetime.json export.

use super::ownership_history::{BorrowInfo, CloneInfo, OwnershipHistoryRecorder, OwnershipSummary};
use crate::capture::types::AllocationInfo;
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
                event
                    .var_name
                    .as_ref()
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
    fn format_event_type(
        &self,
        event_type: &super::ownership_history::OwnershipEventType,
    ) -> String {
        match event_type {
            super::ownership_history::OwnershipEventType::Allocated => "Allocation".to_string(),
            super::ownership_history::OwnershipEventType::Cloned { .. } => "Clone".to_string(),
            super::ownership_history::OwnershipEventType::Dropped => "Deallocation".to_string(),
            super::ownership_history::OwnershipEventType::OwnershipTransferred { .. } => {
                "OwnershipTransfer".to_string()
            }
            super::ownership_history::OwnershipEventType::Borrowed { .. } => "Borrow".to_string(),
            super::ownership_history::OwnershipEventType::MutablyBorrowed { .. } => {
                "MutableBorrow".to_string()
            }
            super::ownership_history::OwnershipEventType::BorrowReleased { .. } => {
                "BorrowRelease".to_string()
            }
            super::ownership_history::OwnershipEventType::RefCountChanged { .. } => {
                "RefCountChange".to_string()
            }
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
    fn calculate_efficiency_score(
        &self,
        allocation: &AllocationInfo,
        ownership_summary: &OwnershipSummary,
    ) -> f64 {
        let mut score: f64 = 0.5; // Base score

        // Bonus for having a meaningful variable name
        if allocation
            .var_name
            .as_ref()
            .map(|name| self.is_user_variable(name))
            .unwrap_or(false)
        {
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
        score.clamp(0.0, 1.0)
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
    fn generate_variable_groups(
        &self,
        lifecycle_events: &[LifecycleEventSummary],
    ) -> Vec<VariableGroup> {
        let mut groups: HashMap<String, Vec<&LifecycleEventSummary>> = HashMap::new();

        // Group by type name
        for event in lifecycle_events {
            if let Some(ref type_name) = event.type_name {
                let group_name = self.extract_base_type_name(type_name);
                groups.entry(group_name).or_default().push(event);
            }
        }

        // Convert to VariableGroup structs
        groups
            .into_iter()
            .map(|(name, events)| {
                let variables: Vec<String> =
                    events.iter().filter_map(|e| e.var_name.clone()).collect();

                let total_memory: usize = events.iter().map(|e| e.size).sum();

                let average_lifetime_ms = if !events.is_empty() {
                    let total_lifetime: u64 = events.iter().filter_map(|e| e.lifetime_ms).sum();
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

    /// Objective: Verify LifecycleSummaryGenerator creation with default config
    /// Invariants: Default config should have include_borrow_details=true, include_clone_details=true
    #[test]
    fn test_lifecycle_summary_generator_creation() {
        let generator = LifecycleSummaryGenerator::new();
        assert!(
            generator.config.include_borrow_details,
            "Default include_borrow_details should be true"
        );
        assert!(
            generator.config.include_clone_details,
            "Default include_clone_details should be true"
        );
    }

    /// Objective: Verify Default trait implementation
    /// Invariants: Default should create same as new()
    #[test]
    fn test_lifecycle_summary_generator_default() {
        let generator = LifecycleSummaryGenerator::default();
        assert!(
            generator.config.include_borrow_details,
            "Default should have include_borrow_details=true"
        );
    }

    /// Objective: Verify with_config functionality
    /// Invariants: Custom config should be applied
    #[test]
    fn test_lifecycle_summary_generator_with_config() {
        let config = SummaryConfig {
            include_borrow_details: false,
            include_clone_details: false,
            min_lifetime_threshold_ms: 100,
            max_events_per_allocation: 10,
        };

        let generator = LifecycleSummaryGenerator::with_config(config);

        assert!(
            !generator.config.include_borrow_details,
            "Custom include_borrow_details should be false"
        );
        assert!(
            !generator.config.include_clone_details,
            "Custom include_clone_details should be false"
        );
        assert_eq!(
            generator.config.min_lifetime_threshold_ms, 100,
            "Custom min_lifetime_threshold_ms should be 100"
        );
        assert_eq!(
            generator.config.max_events_per_allocation, 10,
            "Custom max_events_per_allocation should be 10"
        );
    }

    /// Objective: Verify classify_lifecycle_pattern for all patterns
    /// Invariants: Should correctly classify all lifecycle patterns
    #[test]
    fn test_lifecycle_pattern_classification() {
        let generator = LifecycleSummaryGenerator::default();

        assert!(
            matches!(
                generator.classify_lifecycle_pattern(None),
                LifecyclePattern::Leaked
            ),
            "None should be Leaked"
        );
        assert!(
            matches!(
                generator.classify_lifecycle_pattern(Some(0)),
                LifecyclePattern::Ephemeral
            ),
            "0ms should be Ephemeral"
        );
        assert!(
            matches!(
                generator.classify_lifecycle_pattern(Some(50)),
                LifecyclePattern::ShortTerm
            ),
            "50ms should be ShortTerm"
        );
        assert!(
            matches!(
                generator.classify_lifecycle_pattern(Some(5000)),
                LifecyclePattern::MediumTerm
            ),
            "5000ms should be MediumTerm"
        );
        assert!(
            matches!(
                generator.classify_lifecycle_pattern(Some(15000)),
                LifecyclePattern::LongTerm
            ),
            "15000ms should be LongTerm"
        );
    }

    /// Objective: Verify classify_lifecycle_pattern boundary values
    /// Invariants: Should correctly handle boundary values
    #[test]
    fn test_lifecycle_pattern_classification_boundaries() {
        let generator = LifecycleSummaryGenerator::default();

        assert!(
            matches!(
                generator.classify_lifecycle_pattern(Some(1)),
                LifecyclePattern::ShortTerm
            ),
            "1ms should be ShortTerm"
        );
        assert!(
            matches!(
                generator.classify_lifecycle_pattern(Some(99)),
                LifecyclePattern::ShortTerm
            ),
            "99ms should be ShortTerm"
        );
        assert!(
            matches!(
                generator.classify_lifecycle_pattern(Some(100)),
                LifecyclePattern::MediumTerm
            ),
            "100ms should be MediumTerm"
        );
        assert!(
            matches!(
                generator.classify_lifecycle_pattern(Some(9999)),
                LifecyclePattern::MediumTerm
            ),
            "9999ms should be MediumTerm"
        );
        assert!(
            matches!(
                generator.classify_lifecycle_pattern(Some(10000)),
                LifecyclePattern::LongTerm
            ),
            "10000ms should be LongTerm"
        );
    }

    /// Objective: Verify is_user_variable with valid names
    /// Invariants: Should return true for user-defined variable names
    #[test]
    fn test_is_user_variable_valid() {
        let generator = LifecycleSummaryGenerator::default();

        assert!(
            generator.is_user_variable("my_variable"),
            "my_variable should be user variable"
        );
        assert!(
            generator.is_user_variable("data"),
            "data should be user variable"
        );
        assert!(
            generator.is_user_variable("result"),
            "result should be user variable"
        );
    }

    /// Objective: Verify is_user_variable with system names
    /// Invariants: Should return false for system-generated names
    #[test]
    fn test_is_user_variable_system() {
        let generator = LifecycleSummaryGenerator::default();

        assert!(
            !generator.is_user_variable("primitive_int"),
            "primitive_int should not be user variable"
        );
        assert!(
            !generator.is_user_variable("struct_Data"),
            "struct_Data should not be user variable"
        );
        assert!(
            !generator.is_user_variable("collection_vec"),
            "collection_vec should not be user variable"
        );
        assert!(
            !generator.is_user_variable("buffer_data"),
            "buffer_data should not be user variable"
        );
        assert!(
            !generator.is_user_variable("system_temp"),
            "system_temp should not be user variable"
        );
        assert!(
            !generator.is_user_variable("fast_tracked_123"),
            "fast_tracked should not be user variable"
        );
        assert!(
            !generator.is_user_variable("unknown"),
            "unknown should not be user variable"
        );
    }

    /// Objective: Verify extract_base_type_name with generic types
    /// Invariants: Should extract base type from generic types
    #[test]
    fn test_extract_base_type_name_generic() {
        let generator = LifecycleSummaryGenerator::default();

        assert_eq!(
            generator.extract_base_type_name("Vec<i32>"),
            "Vec",
            "Should extract Vec from Vec<i32>"
        );
        assert_eq!(
            generator.extract_base_type_name("HashMap<String, i32>"),
            "HashMap",
            "Should extract HashMap from HashMap<String, i32>"
        );
        assert_eq!(
            generator.extract_base_type_name("Option<String>"),
            "Option",
            "Should extract Option from Option<String>"
        );
    }

    /// Objective: Verify extract_base_type_name with module paths
    /// Invariants: Should extract type name from module path
    #[test]
    fn test_extract_base_type_name_module_path() {
        let generator = LifecycleSummaryGenerator::default();

        assert_eq!(
            generator.extract_base_type_name("std::collections::HashMap"),
            "HashMap",
            "Should extract HashMap from module path"
        );
        assert_eq!(
            generator.extract_base_type_name("my_module::MyType"),
            "MyType",
            "Should extract MyType from module path"
        );
    }

    /// Objective: Verify extract_base_type_name with simple types
    /// Invariants: Should return same type for simple names
    #[test]
    fn test_extract_base_type_name_simple() {
        let generator = LifecycleSummaryGenerator::default();

        assert_eq!(
            generator.extract_base_type_name("String"),
            "String",
            "Should return String for String"
        );
        assert_eq!(
            generator.extract_base_type_name("i32"),
            "i32",
            "Should return i32 for i32"
        );
    }

    /// Objective: Verify export_to_json with empty data
    /// Invariants: Should produce valid JSON with all fields
    #[test]
    fn test_json_export_basic() {
        let generator = LifecycleSummaryGenerator::default();
        let export_data = LifecycleExportData {
            lifecycle_events: vec![],
            variable_groups: vec![],
            user_variables_count: 0,
            visualization_ready: true,
            metadata: ExportMetadata {
                export_timestamp: 0,
                total_allocations: 0,
                total_events: 0,
                analysis_duration_ms: 0,
            },
        };

        let json = generator.export_to_json(&export_data).unwrap();
        assert!(
            json.contains("lifecycle_events"),
            "JSON should contain lifecycle_events"
        );
        assert!(
            json.contains("variable_groups"),
            "JSON should contain variable_groups"
        );
        assert!(json.contains("metadata"), "JSON should contain metadata");
    }

    /// Objective: Verify export_to_json with event data
    /// Invariants: Should include all event details in JSON
    #[test]
    fn test_json_export_with_events() {
        let generator = LifecycleSummaryGenerator::default();

        let event_summary = LifecycleEventSummary {
            allocation_ptr: 0x1000,
            var_name: Some("test_var".to_string()),
            type_name: Some("String".to_string()),
            size: 1024,
            lifetime_ms: Some(1000),
            events: vec![],
            summary: AllocationLifecycleSummary {
                lifetime_ms: Some(1000),
                borrow_info: BorrowInfo {
                    immutable_borrows: 0,
                    mutable_borrows: 0,
                    max_concurrent_borrows: 0,
                    last_borrow_timestamp: None,
                    active_borrows: vec![],
                },
                clone_info: CloneInfo {
                    clone_count: 0,
                    is_clone: false,
                    original_ptr: None,
                    cloned_ptrs: vec![],
                },
                ownership_history_available: false,
                lifecycle_pattern: LifecyclePattern::ShortTerm,
                efficiency_score: 0.8,
            },
        };

        let export_data = LifecycleExportData {
            lifecycle_events: vec![event_summary],
            variable_groups: vec![],
            user_variables_count: 1,
            visualization_ready: true,
            metadata: ExportMetadata {
                export_timestamp: 1000,
                total_allocations: 1,
                total_events: 1,
                analysis_duration_ms: 100,
            },
        };

        let json = generator.export_to_json(&export_data).unwrap();
        assert!(
            json.contains("test_var"),
            "JSON should contain variable name"
        );
        assert!(json.contains("String"), "JSON should contain type name");
        assert!(
            json.contains("ShortTerm"),
            "JSON should contain lifecycle pattern"
        );
    }

    /// Objective: Verify SummaryConfig default values
    /// Invariants: Default should have sensible values
    #[test]
    fn test_summary_config_default() {
        let config = SummaryConfig::default();
        assert!(
            config.include_borrow_details,
            "Default include_borrow_details should be true"
        );
        assert!(
            config.include_clone_details,
            "Default include_clone_details should be true"
        );
        assert_eq!(
            config.min_lifetime_threshold_ms, 0,
            "Default min_lifetime_threshold_ms should be 0"
        );
        assert_eq!(
            config.max_events_per_allocation, 50,
            "Default max_events_per_allocation should be 50"
        );
    }

    /// Objective: Verify ExportMetadata creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_lifecycle_export_metadata() {
        let metadata = ExportMetadata {
            export_timestamp: 1234567890,
            total_allocations: 100,
            total_events: 500,
            analysis_duration_ms: 1000,
        };

        assert_eq!(
            metadata.export_timestamp, 1234567890,
            "export_timestamp should match"
        );
        assert_eq!(
            metadata.total_allocations, 100,
            "total_allocations should match"
        );
        assert_eq!(metadata.total_events, 500, "total_events should match");
        assert_eq!(
            metadata.analysis_duration_ms, 1000,
            "analysis_duration_ms should match"
        );
    }

    /// Objective: Verify LifecycleEvent serialization
    /// Invariants: Should serialize to valid JSON
    #[test]
    fn test_lifecycle_event_serialization() {
        let event = LifecycleEvent {
            id: 1,
            event_type: "Allocation".to_string(),
            timestamp: 1000,
            size: Some(1024),
            details: Some("Memory allocated".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(
            json.contains("Allocation"),
            "JSON should contain event_type"
        );
        assert!(json.contains("1024"), "JSON should contain size");
    }

    /// Objective: Verify LifecycleEvent with None fields
    /// Invariants: Should handle None fields correctly
    #[test]
    fn test_lifecycle_event_with_none_fields() {
        let event = LifecycleEvent {
            id: 2,
            event_type: "Deallocation".to_string(),
            timestamp: 2000,
            size: None,
            details: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(
            json.contains("Deallocation"),
            "JSON should contain event_type"
        );
        assert!(
            json.contains("null"),
            "JSON should contain null for None fields"
        );
    }

    /// Objective: Verify LifecyclePattern serialization
    /// Invariants: All patterns should serialize correctly
    #[test]
    fn test_lifecycle_pattern_serialization() {
        let patterns = vec![
            LifecyclePattern::Ephemeral,
            LifecyclePattern::ShortTerm,
            LifecyclePattern::MediumTerm,
            LifecyclePattern::LongTerm,
            LifecyclePattern::Leaked,
            LifecyclePattern::Unknown,
        ];

        for pattern in patterns {
            let json = serde_json::to_string(&pattern).unwrap();
            assert!(
                !json.is_empty(),
                "Pattern should serialize to non-empty JSON"
            );
            let deserialized: LifecyclePattern = serde_json::from_str(&json).unwrap();
            assert_eq!(
                format!("{:?}", pattern),
                format!("{:?}", deserialized),
                "Pattern should deserialize correctly"
            );
        }
    }

    /// Objective: Verify VariableGroup creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_variable_group() {
        let group = VariableGroup {
            name: "String".to_string(),
            variables: vec!["var1".to_string(), "var2".to_string()],
            total_memory: 2048,
            average_lifetime_ms: 500.0,
        };

        assert_eq!(group.name, "String", "name should match");
        assert_eq!(group.variables.len(), 2, "variables count should match");
        assert_eq!(group.total_memory, 2048, "total_memory should match");
        assert_eq!(
            group.average_lifetime_ms, 500.0,
            "average_lifetime_ms should match"
        );
    }

    /// Objective: Verify VariableGroup serialization
    /// Invariants: Should serialize to valid JSON
    #[test]
    fn test_variable_group_serialization() {
        let group = VariableGroup {
            name: "Vec".to_string(),
            variables: vec!["data".to_string()],
            total_memory: 1024,
            average_lifetime_ms: 100.0,
        };

        let json = serde_json::to_string(&group).unwrap();
        assert!(json.contains("Vec"), "JSON should contain group name");
        assert!(json.contains("data"), "JSON should contain variable name");
    }

    /// Objective: Verify AllocationLifecycleSummary creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_allocation_lifecycle_summary() {
        let summary = AllocationLifecycleSummary {
            lifetime_ms: Some(1000),
            borrow_info: BorrowInfo {
                immutable_borrows: 5,
                mutable_borrows: 2,
                max_concurrent_borrows: 3,
                last_borrow_timestamp: Some(500),
                active_borrows: vec![],
            },
            clone_info: CloneInfo {
                clone_count: 1,
                is_clone: false,
                original_ptr: None,
                cloned_ptrs: vec![0x2000],
            },
            ownership_history_available: true,
            lifecycle_pattern: LifecyclePattern::MediumTerm,
            efficiency_score: 0.75,
        };

        assert_eq!(summary.lifetime_ms, Some(1000), "lifetime_ms should match");
        assert_eq!(
            summary.borrow_info.immutable_borrows, 5,
            "immutable_borrows should match"
        );
        assert_eq!(
            summary.clone_info.clone_count, 1,
            "clone_count should match"
        );
        assert!(
            summary.ownership_history_available,
            "ownership_history_available should be true"
        );
        assert_eq!(
            summary.efficiency_score, 0.75,
            "efficiency_score should match"
        );
    }

    /// Objective: Verify get_current_timestamp returns valid value
    /// Invariants: Timestamp should be positive
    #[test]
    fn test_get_current_timestamp() {
        let generator = LifecycleSummaryGenerator::default();
        let ts = generator.get_current_timestamp();

        assert!(ts > 0, "Timestamp should be positive");
    }

    /// Objective: Verify LifecycleEventSummary creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_lifecycle_event_summary() {
        let summary = LifecycleEventSummary {
            allocation_ptr: 0x1000,
            var_name: Some("test".to_string()),
            type_name: Some("i32".to_string()),
            size: 4,
            lifetime_ms: Some(100),
            events: vec![],
            summary: AllocationLifecycleSummary {
                lifetime_ms: Some(100),
                borrow_info: BorrowInfo {
                    immutable_borrows: 0,
                    mutable_borrows: 0,
                    max_concurrent_borrows: 0,
                    last_borrow_timestamp: None,
                    active_borrows: vec![],
                },
                clone_info: CloneInfo {
                    clone_count: 0,
                    is_clone: false,
                    original_ptr: None,
                    cloned_ptrs: vec![],
                },
                ownership_history_available: false,
                lifecycle_pattern: LifecyclePattern::ShortTerm,
                efficiency_score: 0.5,
            },
        };

        assert_eq!(
            summary.allocation_ptr, 0x1000,
            "allocation_ptr should match"
        );
        assert_eq!(
            summary.var_name,
            Some("test".to_string()),
            "var_name should match"
        );
        assert_eq!(summary.size, 4, "size should match");
    }

    /// Objective: Verify SummaryConfig clone functionality
    /// Invariants: Cloned config should have same values
    #[test]
    fn test_summary_config_clone() {
        let original = SummaryConfig {
            include_borrow_details: false,
            include_clone_details: true,
            min_lifetime_threshold_ms: 50,
            max_events_per_allocation: 25,
        };

        let cloned = original.clone();

        assert_eq!(
            original.include_borrow_details, cloned.include_borrow_details,
            "Cloned include_borrow_details should match"
        );
        assert_eq!(
            original.min_lifetime_threshold_ms, cloned.min_lifetime_threshold_ms,
            "Cloned min_lifetime_threshold_ms should match"
        );
    }

    /// Objective: Verify LifecycleExportData clone functionality
    /// Invariants: Cloned data should have same values
    #[test]
    fn test_lifecycle_export_data_clone() {
        let original = LifecycleExportData {
            lifecycle_events: vec![],
            variable_groups: vec![],
            user_variables_count: 5,
            visualization_ready: true,
            metadata: ExportMetadata {
                export_timestamp: 1000,
                total_allocations: 10,
                total_events: 20,
                analysis_duration_ms: 50,
            },
        };

        let cloned = original.clone();

        assert_eq!(
            original.user_variables_count, cloned.user_variables_count,
            "Cloned user_variables_count should match"
        );
        assert_eq!(
            original.visualization_ready, cloned.visualization_ready,
            "Cloned visualization_ready should match"
        );
    }

    /// Objective: Verify generate_lifecycle_export with empty allocations
    /// Invariants: Should return valid export with empty events
    #[test]
    fn test_generate_lifecycle_export_empty() {
        let generator = LifecycleSummaryGenerator::default();
        let ownership_history = OwnershipHistoryRecorder::new();
        let allocations: Vec<AllocationInfo> = vec![];

        let export_data = generator.generate_lifecycle_export(&ownership_history, &allocations);

        assert_eq!(
            export_data.lifecycle_events.len(),
            0,
            "Should have no lifecycle events"
        );
        assert_eq!(
            export_data.variable_groups.len(),
            0,
            "Should have no variable groups"
        );
        assert_eq!(
            export_data.user_variables_count, 0,
            "Should have 0 user variables"
        );
        assert_eq!(
            export_data.metadata.total_allocations, 0,
            "Should have 0 total allocations"
        );
    }
}
