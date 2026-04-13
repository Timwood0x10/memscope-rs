//! Scope tracking types.
//!
//! This module contains types for tracking scope lifecycle,
//! hierarchy, and memory usage within scopes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export RiskDistribution and ImpactLevel from allocation module
pub use super::allocation::{ImpactLevel, RiskDistribution};

/// Scope information.
#[derive(Debug, Clone, Serialize)]
pub struct ScopeInfo {
    /// Name.
    pub name: String,
    /// Parent.
    pub parent: Option<String>,
    /// Children.
    pub children: Vec<String>,
    /// Depth.
    pub depth: usize,
    /// Variables.
    pub variables: Vec<String>,
    /// Total Memory.
    pub total_memory: usize,
    /// Peak Memory.
    pub peak_memory: usize,
    /// Number of allocations.
    pub allocation_count: usize,
    /// Lifetime Start.
    pub lifetime_start: Option<u64>,
    /// Lifetime End.
    pub lifetime_end: Option<u64>,
    /// Is Active.
    pub is_active: bool,
    /// Start Time.
    pub start_time: u64,
    /// End Time.
    pub end_time: Option<u64>,
    /// Memory Usage.
    pub memory_usage: usize,
    /// Child Scopes.
    pub child_scopes: Vec<String>,
    /// Parent Scope.
    pub parent_scope: Option<String>,
}

/// Scope hierarchy.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ScopeHierarchy {
    /// Root Scopes.
    pub root_scopes: Vec<String>,
    /// Scope Tree.
    pub scope_tree: HashMap<String, ScopeInfo>,
    /// Max Depth.
    pub max_depth: usize,
    /// Total Scopes.
    pub total_scopes: usize,
    /// Relationships.
    pub relationships: HashMap<String, Vec<String>>,
    /// Depth Map.
    pub depth_map: HashMap<String, usize>,
}

/// Type-specific lifecycle pattern analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeLifecyclePattern {
    /// Type Name.
    pub type_name: String,
    /// Average lifetime in milliseconds.
    pub average_lifetime_ms: f64,
    /// Typical Size.
    pub typical_size: usize,
    /// Growth Pattern.
    pub growth_pattern: String,
    /// Risk Level.
    pub risk_level: String,
    /// Instance Count.
    pub instance_count: usize,
}

/// Growth reason for tracking allocation growth.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GrowthReason {
    /// Initial allocation.
    Initial,
    /// Memory expansion.
    Expansion,
    /// Memory reallocation.
    Reallocation,
    /// Performance optimization.
    Optimization,
    /// User-requested allocation.
    UserRequested,
}

/// Type of allocation event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AllocationEventType {
    /// Memory allocation event.
    Allocate,
    /// Memory deallocation event.
    Deallocate,
    /// Memory reallocation event.
    Reallocate,
    /// Memory move event.
    Move,
    /// Memory borrow event.
    Borrow,
    /// Memory return event.
    Return,
}

/// Type of scope event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScopeEventType {
    /// Enter scope event.
    Enter,
    /// Exit scope event.
    Exit,
    /// Create scope event.
    Create,
    /// Destroy scope event.
    Destroy,
}

/// Growth event tracking allocation growth patterns.
#[derive(Debug, Clone, Serialize)]
pub struct GrowthEvent {
    /// Timestamp.
    pub timestamp: u64,
    /// Old Size.
    pub old_size: usize,
    /// New Size.
    pub new_size: usize,
    /// Growth Factor.
    pub growth_factor: f64,
    /// Reason.
    pub reason: GrowthReason,
    /// Var Name.
    pub var_name: String,
}

/// Borrow event for tracking borrowing patterns.
#[derive(Debug, Clone, Serialize)]
pub struct BorrowEvent {
    /// Timestamp.
    pub timestamp: u64,
    /// Memory pointer address.
    pub ptr: usize,
    /// Borrow Type.
    pub borrow_type: String,
    /// Duration Ms.
    pub duration_ms: u64,
    /// Var Name.
    pub var_name: String,
}

/// Move event for tracking ownership transfers.
#[derive(Debug, Clone, Serialize)]
pub struct MoveEvent {
    /// Timestamp.
    pub timestamp: u64,
    /// From Ptr.
    pub from_ptr: usize,
    /// To Ptr.
    pub to_ptr: usize,
    /// Size in bytes.
    pub size: usize,
    /// Var Name.
    pub var_name: String,
}

/// Variable relationship tracking.
#[derive(Debug, Clone, Serialize)]
pub struct VariableRelationship {
    /// Source Var.
    pub source_var: String,
    /// Target Var.
    pub target_var: String,
    /// Relationship Type.
    pub relationship_type: String,
    /// Strength.
    pub strength: f64,
}

/// Potential memory leak detection.
#[derive(Debug, Clone, Serialize)]
pub struct PotentialLeak {
    /// Memory pointer address.
    pub ptr: usize,
    /// Size in bytes.
    pub size: usize,
    /// Age in milliseconds.
    pub age_ms: u64,
    /// Var Name.
    pub var_name: Option<String>,
    /// Type Name.
    pub type_name: Option<String>,
    /// Severity.
    pub severity: String,
}

/// Scope analysis results.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ScopeAnalysis {
    /// Total scopes
    pub total_scopes: usize,
    /// Active scopes
    pub active_scopes: usize,
    /// Max depth
    pub max_depth: usize,
    /// Average lifetime in milliseconds
    pub average_lifetime: f64,
    /// Memory efficiency ratio
    pub memory_efficiency: f64,
    /// Scope information
    pub scopes: Vec<ScopeInfo>,
    /// Scope hierarchy
    pub scope_hierarchy: ScopeHierarchy,
    /// Cross scope references
    pub cross_scope_references: Vec<String>,
}

/// Scope lifecycle metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeLifecycleMetrics {
    /// Name of the scope
    pub scope_name: String,
    /// Number of variables in scope
    pub variable_count: usize,
    /// Average lifetime in milliseconds
    pub average_lifetime_ms: f64,
    /// Total memory used by scope
    pub total_memory_usage: usize,
    /// Peak memory usage in scope
    pub peak_memory_usage: usize,
    /// Frequency of allocations
    pub allocation_frequency: f64,
    /// Efficiency of deallocations
    pub deallocation_efficiency: f64,
    /// Number of completed allocations
    pub completed_allocations: usize,
    /// Number of memory growth events
    pub memory_growth_events: usize,
    /// Peak number of concurrent variables
    pub peak_concurrent_variables: usize,
    /// Memory efficiency ratio
    pub memory_efficiency_ratio: f64,
    /// Number of ownership transfers
    pub ownership_transfer_events: usize,
    /// Fragmentation score
    pub fragmentation_score: f64,
    /// Number of instant allocations
    pub instant_allocations: usize,
    /// Number of short-term allocations
    pub short_term_allocations: usize,
    /// Number of medium-term allocations
    pub medium_term_allocations: usize,
    /// Number of long-term allocations
    pub long_term_allocations: usize,
    /// Number of suspected memory leaks
    pub suspected_leaks: usize,
    /// Risk distribution analysis
    pub risk_distribution: RiskDistribution,
    /// Metrics for individual scopes
    pub scope_metrics: Vec<ScopeLifecycleMetrics>,
    /// Lifecycle patterns for types
    pub type_lifecycle_patterns: Vec<TypeLifecyclePattern>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_distribution_default() {
        let risk = RiskDistribution::default();

        assert_eq!(risk.low_risk, 0);
        assert_eq!(risk.medium_risk, 0);
        assert_eq!(risk.high_risk, 0);
        assert_eq!(risk.critical_risk, 0);
    }

    #[test]
    fn test_allocation_event_type_variants() {
        let events = vec![
            AllocationEventType::Allocate,
            AllocationEventType::Deallocate,
            AllocationEventType::Reallocate,
            AllocationEventType::Move,
            AllocationEventType::Borrow,
            AllocationEventType::Return,
        ];

        for event in events {
            assert!(!format!("{event:?}").is_empty());
        }
    }

    #[test]
    fn test_scope_event_type_variants() {
        let events = vec![
            ScopeEventType::Enter,
            ScopeEventType::Exit,
            ScopeEventType::Create,
            ScopeEventType::Destroy,
        ];

        for event in events {
            assert!(!format!("{event:?}").is_empty());
        }
    }

    #[test]
    fn test_growth_reason_variants() {
        let reasons = vec![
            GrowthReason::Initial,
            GrowthReason::Expansion,
            GrowthReason::Reallocation,
            GrowthReason::Optimization,
            GrowthReason::UserRequested,
        ];

        for reason in reasons {
            assert!(!format!("{reason:?}").is_empty());
        }
    }

    #[test]
    fn test_scope_info_creation() {
        let scope = ScopeInfo {
            name: "main".to_string(),
            parent: None,
            children: vec![],
            depth: 0,
            variables: vec!["x".to_string(), "y".to_string()],
            total_memory: 1024,
            peak_memory: 2048,
            allocation_count: 5,
            lifetime_start: Some(0),
            lifetime_end: Some(1000),
            is_active: false,
            start_time: 0,
            end_time: Some(1000),
            memory_usage: 1024,
            child_scopes: vec![],
            parent_scope: None,
        };

        assert_eq!(scope.name, "main");
        assert_eq!(scope.variables.len(), 2);
        assert_eq!(scope.total_memory, 1024);
    }

    #[test]
    fn test_scope_info_with_parent() {
        let scope = ScopeInfo {
            name: "inner".to_string(),
            parent: Some("main".to_string()),
            children: vec![],
            depth: 1,
            variables: vec![],
            total_memory: 0,
            peak_memory: 0,
            allocation_count: 0,
            lifetime_start: None,
            lifetime_end: None,
            is_active: true,
            start_time: 100,
            end_time: None,
            memory_usage: 0,
            child_scopes: vec![],
            parent_scope: Some("main".to_string()),
        };

        assert_eq!(scope.depth, 1);
        assert!(scope.is_active);
        assert_eq!(scope.parent, Some("main".to_string()));
    }

    #[test]
    fn test_scope_hierarchy_default() {
        let hierarchy = ScopeHierarchy::default();

        assert!(hierarchy.root_scopes.is_empty());
        assert!(hierarchy.scope_tree.is_empty());
        assert_eq!(hierarchy.max_depth, 0);
        assert_eq!(hierarchy.total_scopes, 0);
    }

    #[test]
    fn test_scope_hierarchy_with_scopes() {
        let mut scope_tree = HashMap::new();
        scope_tree.insert(
            "main".to_string(),
            ScopeInfo {
                name: "main".to_string(),
                parent: None,
                children: vec!["inner".to_string()],
                depth: 0,
                variables: vec![],
                total_memory: 1024,
                peak_memory: 1024,
                allocation_count: 1,
                lifetime_start: None,
                lifetime_end: None,
                is_active: true,
                start_time: 0,
                end_time: None,
                memory_usage: 1024,
                child_scopes: vec!["inner".to_string()],
                parent_scope: None,
            },
        );

        let hierarchy = ScopeHierarchy {
            root_scopes: vec!["main".to_string()],
            scope_tree,
            max_depth: 1,
            total_scopes: 2,
            relationships: HashMap::new(),
            depth_map: HashMap::new(),
        };

        assert_eq!(hierarchy.root_scopes.len(), 1);
        assert_eq!(hierarchy.total_scopes, 2);
    }

    #[test]
    fn test_type_lifecycle_pattern_creation() {
        let pattern = TypeLifecyclePattern {
            type_name: "Vec<u8>".to_string(),
            average_lifetime_ms: 500.0,
            typical_size: 1024,
            growth_pattern: "exponential".to_string(),
            risk_level: "low".to_string(),
            instance_count: 100,
        };

        assert_eq!(pattern.type_name, "Vec<u8>");
        assert_eq!(pattern.average_lifetime_ms, 500.0);
    }

    #[test]
    fn test_growth_event_creation() {
        let event = GrowthEvent {
            timestamp: 1000,
            old_size: 1024,
            new_size: 2048,
            growth_factor: 2.0,
            reason: GrowthReason::Expansion,
            var_name: "buffer".to_string(),
        };

        assert_eq!(event.growth_factor, 2.0);
        assert_eq!(event.reason, GrowthReason::Expansion);
    }

    #[test]
    fn test_borrow_event_creation() {
        let event = BorrowEvent {
            timestamp: 100,
            ptr: 0x1000,
            borrow_type: "immutable".to_string(),
            duration_ms: 50,
            var_name: "data".to_string(),
        };

        assert_eq!(event.borrow_type, "immutable");
        assert_eq!(event.duration_ms, 50);
    }

    #[test]
    fn test_move_event_creation() {
        let event = MoveEvent {
            timestamp: 200,
            from_ptr: 0x1000,
            to_ptr: 0x2000,
            size: 1024,
            var_name: "value".to_string(),
        };

        assert_eq!(event.from_ptr, 0x1000);
        assert_eq!(event.to_ptr, 0x2000);
    }

    #[test]
    fn test_variable_relationship_creation() {
        let rel = VariableRelationship {
            source_var: "parent".to_string(),
            target_var: "child".to_string(),
            relationship_type: "ownership".to_string(),
            strength: 0.9,
        };

        assert_eq!(rel.relationship_type, "ownership");
        assert_eq!(rel.strength, 0.9);
    }

    #[test]
    fn test_potential_leak_creation() {
        let leak = PotentialLeak {
            ptr: 0x1000,
            size: 1024,
            age_ms: 5000,
            var_name: Some("leaked".to_string()),
            type_name: Some("String".to_string()),
            severity: "high".to_string(),
        };

        assert_eq!(leak.ptr, 0x1000);
        assert_eq!(leak.severity, "high");
    }

    #[test]
    fn test_scope_analysis_default() {
        let analysis = ScopeAnalysis::default();

        assert_eq!(analysis.total_scopes, 0);
        assert_eq!(analysis.active_scopes, 0);
        assert_eq!(analysis.max_depth, 0);
    }

    #[test]
    fn test_scope_lifecycle_metrics_default() {
        let metrics = ScopeLifecycleMetrics::default();

        assert!(metrics.scope_name.is_empty());
        assert_eq!(metrics.variable_count, 0);
        assert_eq!(metrics.average_lifetime_ms, 0.0);
    }

    #[test]
    fn test_scope_lifecycle_metrics_with_values() {
        let metrics = ScopeLifecycleMetrics {
            scope_name: "test_scope".to_string(),
            variable_count: 10,
            average_lifetime_ms: 100.0,
            total_memory_usage: 4096,
            peak_memory_usage: 8192,
            allocation_frequency: 50.0,
            deallocation_efficiency: 0.95,
            completed_allocations: 100,
            memory_growth_events: 5,
            peak_concurrent_variables: 20,
            memory_efficiency_ratio: 0.85,
            ownership_transfer_events: 10,
            fragmentation_score: 0.1,
            instant_allocations: 30,
            short_term_allocations: 40,
            medium_term_allocations: 20,
            long_term_allocations: 10,
            suspected_leaks: 1,
            risk_distribution: RiskDistribution::default(),
            scope_metrics: vec![],
            type_lifecycle_patterns: vec![],
        };

        assert_eq!(metrics.scope_name, "test_scope");
        assert_eq!(metrics.variable_count, 10);
        assert_eq!(metrics.total_memory_usage, 4096);
    }

    #[test]
    fn test_allocation_event_type_equality() {
        assert_eq!(AllocationEventType::Allocate, AllocationEventType::Allocate);
        assert_ne!(
            AllocationEventType::Allocate,
            AllocationEventType::Deallocate
        );
    }

    #[test]
    fn test_scope_event_type_equality() {
        assert_eq!(ScopeEventType::Enter, ScopeEventType::Enter);
        assert_ne!(ScopeEventType::Enter, ScopeEventType::Exit);
    }

    #[test]
    fn test_growth_reason_equality() {
        assert_eq!(GrowthReason::Initial, GrowthReason::Initial);
        assert_ne!(GrowthReason::Initial, GrowthReason::Expansion);
    }
}
