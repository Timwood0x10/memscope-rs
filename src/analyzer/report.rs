//! Analysis report types.
//!
//! This module defines all report types returned by analysis operations.

use crate::core::error::MemScopeError;
use crate::snapshot::ActiveAllocation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Full analysis report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    /// Memory statistics
    pub stats: MemoryStatsReport,
    /// Leak detection results
    pub leaks: LeakReport,
    /// Cycle detection results
    pub cycles: CycleReport,
    /// Metrics summary
    pub metrics: MetricsReport,
}

impl AnalysisReport {
    /// Check if any issues were found.
    pub fn has_issues(&self) -> bool {
        self.leaks.has_leaks() || self.cycles.has_cycles()
    }

    /// Get summary string.
    pub fn summary(&self) -> String {
        format!(
            "Analysis Report:\n  Allocations: {}\n  Total Bytes: {}\n  Leaks: {}\n  Cycles: {}",
            self.stats.allocation_count,
            self.stats.total_bytes,
            self.leaks.leak_count,
            self.cycles.cycle_count
        )
    }
}

/// Memory statistics report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatsReport {
    /// Total number of allocations
    pub allocation_count: usize,
    /// Total bytes allocated
    pub total_bytes: usize,
    /// Peak memory usage
    pub peak_bytes: usize,
    /// Number of threads
    pub thread_count: usize,
}

/// Memory leak report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakReport {
    /// Number of leaked allocations
    pub leak_count: usize,
    /// Total bytes leaked
    pub total_leaked_bytes: usize,
    /// Details of leaked allocations
    pub leaked_allocations: Vec<LeakInfo>,
}

impl LeakReport {
    /// Check if any leaks were detected.
    pub fn has_leaks(&self) -> bool {
        self.leak_count > 0
    }

    /// Create empty report.
    pub fn empty() -> Self {
        Self {
            leak_count: 0,
            total_leaked_bytes: 0,
            leaked_allocations: vec![],
        }
    }
}

/// Information about a leaked allocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakInfo {
    /// Memory pointer
    pub ptr: usize,
    /// Allocation size
    pub size: usize,
    /// Variable name if available
    pub var_name: Option<String>,
    /// Type name if available
    pub type_name: Option<String>,
    /// Thread ID
    pub thread_id: u64,
}

impl From<&ActiveAllocation> for LeakInfo {
    fn from(alloc: &ActiveAllocation) -> Self {
        Self {
            ptr: alloc.ptr.unwrap_or(0),
            size: alloc.size,
            var_name: alloc.var_name.clone(),
            type_name: alloc.type_name.clone(),
            thread_id: alloc.thread_id,
        }
    }
}

/// Cycle detection report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleReport {
    /// Number of cycles detected
    pub cycle_count: usize,
    /// Details of detected cycles
    pub cycles: Vec<CycleInfo>,
}

impl CycleReport {
    /// Check if any cycles were detected.
    pub fn has_cycles(&self) -> bool {
        self.cycle_count > 0
    }

    /// Create empty report.
    pub fn empty() -> Self {
        Self {
            cycle_count: 0,
            cycles: vec![],
        }
    }

    /// Create from graph (placeholder for now).
    pub fn from_graph<T>(_graph: &T) -> Self {
        Self::empty()
    }
}

/// Information about a reference cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleInfo {
    /// Node IDs in the cycle
    pub nodes: Vec<u64>,
}

/// Metrics report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricsReport {
    /// Total allocation count
    pub allocation_count: usize,
    /// Total bytes
    pub total_bytes: usize,
    /// Peak bytes
    pub peak_bytes: usize,
    /// Thread count
    pub thread_count: usize,
    /// Allocations by type
    pub by_type: HashMap<String, TypeMetric>,
}

/// Metrics for a specific type.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TypeMetric {
    /// Number of allocations
    pub count: usize,
    /// Total bytes
    pub total_bytes: usize,
}

/// UAF (Use-After-Free) detection report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UafReport {
    /// Number of UAF issues detected
    pub uaf_count: usize,
    /// Details of UAF issues
    pub issues: Vec<UafInfo>,
}

impl UafReport {
    /// Create empty report.
    pub fn empty() -> Self {
        Self {
            uaf_count: 0,
            issues: vec![],
        }
    }

    /// Create from events (placeholder for now).
    pub fn from_events(_events: &[crate::event_store::MemoryEvent]) -> Self {
        Self::empty()
    }
}

/// Information about a UAF issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UafInfo {
    /// Memory pointer
    pub ptr: usize,
    /// Deallocation timestamp
    pub deallocated_at: u64,
    /// Access timestamp
    pub accessed_at: u64,
}

/// Safety analysis report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyReport {
    /// Overall safety score (0-100)
    pub score: f64,
    /// Number of issues
    pub issue_count: usize,
    /// Issues found
    pub issues: Vec<SafetyIssue>,
}

impl SafetyReport {
    /// Create from allocations (placeholder for now).
    pub fn from_allocations(_allocations: &[&ActiveAllocation]) -> Self {
        Self {
            score: 100.0,
            issue_count: 0,
            issues: vec![],
        }
    }
}

/// A safety issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyIssue {
    /// Issue severity
    pub severity: IssueSeverity,
    /// Issue description
    pub description: String,
    /// Related pointer
    pub ptr: Option<usize>,
}

/// Issue severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Export error type alias using project's MemScopeError.
#[allow(dead_code)]
pub type ExportError = MemScopeError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_leak_report() {
        let report = LeakReport::empty();
        assert!(!report.has_leaks());
        assert_eq!(report.leak_count, 0);
    }

    #[test]
    fn test_empty_cycle_report() {
        let report = CycleReport::empty();
        assert!(!report.has_cycles());
        assert_eq!(report.cycle_count, 0);
    }

    #[test]
    fn test_analysis_report_summary() {
        let report = AnalysisReport {
            stats: MemoryStatsReport {
                allocation_count: 10,
                total_bytes: 1000,
                peak_bytes: 500,
                thread_count: 2,
            },
            leaks: LeakReport::empty(),
            cycles: CycleReport::empty(),
            metrics: MetricsReport::default(),
        };
        let summary = report.summary();
        assert!(summary.contains("Allocations: 10"));
        assert!(summary.contains("Total Bytes: 1000"));
    }
}
