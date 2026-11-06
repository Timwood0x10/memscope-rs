//! Top N report generation
//!
//! This module provides analysis to identify the top N allocation sites,
//! leaked bytes, and temporary churn for quick performance insights.

use crate::capture::types::AllocationInfo;
use std::collections::HashMap;

/// Top N analyzer
pub struct TopNAnalyzer {
    allocations: Vec<AllocationInfo>,
}

impl TopNAnalyzer {
    /// Create a new Top N analyzer
    pub fn new(allocations: Vec<AllocationInfo>) -> Self {
        Self { allocations }
    }

    /// Get top N allocation sites by total bytes allocated
    pub fn top_allocation_sites(&self, n: usize) -> Vec<AllocationSite> {
        let mut sites = HashMap::new();
        for alloc in &self.allocations {
            let key = alloc
                .stack_trace
                .as_ref()
                .and_then(|s| s.first().cloned())
                .unwrap_or_else(|| "unknown".to_string());
            let entry = sites.entry(key.clone()).or_insert(AllocationSite {
                name: key,
                total_bytes: 0,
                allocation_count: 0,
            });
            entry.total_bytes += alloc.size;
            entry.allocation_count += 1;
        }

        let mut result: Vec<_> = sites.into_values().collect();
        result.sort_by_key(|b| std::cmp::Reverse(b.total_bytes));
        result.truncate(n);
        result
    }

    /// Get top N leaked allocations by bytes.
    ///
    /// Only allocations explicitly flagged as leaked (`AllocationInfo::is_leaked`)
    /// are considered: "no deallocation timestamp" describes an *observed live*
    /// allocation, not a leak, and must never be promoted to one here.
    pub fn top_leaked_bytes(&self, n: usize) -> Vec<LeakedAllocation> {
        let mut leaked: Vec<LeakedAllocation> = self
            .allocations
            .iter()
            .filter(|alloc| alloc.is_leaked)
            .map(|alloc| LeakedAllocation {
                ptr: alloc.ptr,
                size: alloc.size,
                type_name: alloc.type_name.clone(),
                stack_trace: alloc.stack_trace.clone(),
                timestamp_alloc: alloc.timestamp_alloc,
            })
            .collect();

        leaked.sort_by_key(|b| std::cmp::Reverse(b.size));
        leaked.truncate(n);
        leaked
    }

    /// Get top N temporary churn (short-lived allocations) by count
    pub fn top_temporary_churn(&self, n: usize, threshold_ms: u64) -> Vec<TemporaryChurn> {
        let mut churn = HashMap::new();
        for alloc in &self.allocations {
            if let Some(dealloc_ts) = alloc.timestamp_dealloc {
                let lifetime_ms = dealloc_ts - alloc.timestamp_alloc;
                if lifetime_ms < threshold_ms {
                    let key = alloc
                        .stack_trace
                        .as_ref()
                        .and_then(|s| s.first().cloned())
                        .unwrap_or_else(|| "unknown".to_string());
                    let entry = churn.entry(key.clone()).or_insert(TemporaryChurn {
                        name: key,
                        allocation_count: 0,
                        total_bytes: 0,
                        average_lifetime_ms: 0.0,
                    });
                    entry.allocation_count += 1;
                    entry.total_bytes += alloc.size;
                }
            }
        }

        let mut result: Vec<_> = churn.into_values().collect();
        result.sort_by_key(|b| std::cmp::Reverse(b.allocation_count));
        result.truncate(n);
        result
    }

    /// Get summary statistics
    pub fn summary(&self) -> TopNSummary {
        let total_allocations = self.allocations.len();
        let total_bytes: usize = self.allocations.iter().map(|a| a.size).sum();
        // Leak counters follow the explicit leak flag, never the absence of a
        // deallocation timestamp (see `top_leaked_bytes`).
        let leaked_count = self.allocations.iter().filter(|a| a.is_leaked).count();
        let leaked_bytes: usize = self
            .allocations
            .iter()
            .filter(|a| a.is_leaked)
            .map(|a| a.size)
            .sum();

        TopNSummary {
            total_allocations,
            total_bytes,
            leaked_count,
            leaked_bytes,
        }
    }
}

/// Allocation site information
#[derive(Debug, Clone)]
pub struct AllocationSite {
    pub name: String,
    pub total_bytes: usize,
    pub allocation_count: usize,
}

/// Leaked allocation information
#[derive(Debug, Clone)]
pub struct LeakedAllocation {
    pub ptr: usize,
    pub size: usize,
    pub type_name: Option<String>,
    pub stack_trace: Option<Vec<String>>,
    pub timestamp_alloc: u64,
}

/// Temporary churn information
#[derive(Debug, Clone)]
pub struct TemporaryChurn {
    pub name: String,
    pub allocation_count: usize,
    pub total_bytes: usize,
    pub average_lifetime_ms: f64,
}

/// Top N summary statistics
#[derive(Debug, Clone)]
pub struct TopNSummary {
    pub total_allocations: usize,
    pub total_bytes: usize,
    pub leaked_count: usize,
    pub leaked_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_allocation_sites_empty() {
        let analyzer = TopNAnalyzer::new(vec![]);
        let top = analyzer.top_allocation_sites(10);
        assert!(top.is_empty());
    }

    #[test]
    fn test_top_leaked_bytes_empty() {
        let analyzer = TopNAnalyzer::new(vec![]);
        let leaked = analyzer.top_leaked_bytes(10);
        assert!(leaked.is_empty());
    }

    #[test]
    fn test_top_temporary_churn_empty() {
        let analyzer = TopNAnalyzer::new(vec![]);
        let churn = analyzer.top_temporary_churn(10, 100);
        assert!(churn.is_empty());
    }

    #[test]
    fn test_summary_empty() {
        let analyzer = TopNAnalyzer::new(vec![]);
        let summary = analyzer.summary();
        assert_eq!(summary.total_allocations, 0);
        assert_eq!(summary.total_bytes, 0);
        assert_eq!(summary.leaked_count, 0);
        assert_eq!(summary.leaked_bytes, 0);
    }

    /// Objective: Verify that a live allocation (no deallocation timestamp) is
    /// never promoted to a leak by the Top-N analysis.
    /// Invariants: only allocations with the explicit `is_leaked` flag appear in
    /// the leaked reports, and the summary counts match them exactly.
    #[test]
    fn test_live_allocation_is_not_counted_as_leak() {
        let mut live = AllocationInfo::new(0x1000, 512);
        live.timestamp_dealloc = None;
        live.is_leaked = false;

        let mut leaked = AllocationInfo::new(0x2000, 128);
        leaked.timestamp_dealloc = None;
        leaked.is_leaked = true;

        let analyzer = TopNAnalyzer::new(vec![live, leaked]);

        let top = analyzer.top_leaked_bytes(10);
        assert_eq!(
            top.len(),
            1,
            "only the explicitly flagged allocation may appear, got {} entries",
            top.len()
        );
        assert_eq!(top[0].ptr, 0x2000, "the flagged allocation must be kept");

        let summary = analyzer.summary();
        assert_eq!(
            summary.leaked_count, 1,
            "a live allocation must not be counted as leaked"
        );
        assert_eq!(
            summary.leaked_bytes, 128,
            "leaked bytes must only cover flagged allocations"
        );
    }
}
