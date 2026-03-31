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
}
