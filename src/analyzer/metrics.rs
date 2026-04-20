//! Metrics analysis module.

use crate::analyzer::report::{MetricsReport, TypeMetric};
use crate::view::MemoryView;
use std::collections::HashMap;
use tracing::debug;

/// Metrics analysis module.
///
/// Provides memory metrics and statistics.
pub struct MetricsAnalysis {
    view: MemoryView,
}

impl MetricsAnalysis {
    /// Create from view.
    pub fn from_view(view: &MemoryView) -> Self {
        debug!("Creating MetricsAnalysis with {} allocations", view.len());
        Self { view: view.clone() }
    }

    /// Get metrics summary.
    pub fn summary(&self) -> MetricsReport {
        debug!("Computing metrics summary");
        let stats = self.view.snapshot();
        let report = MetricsReport {
            allocation_count: stats.stats.active_allocations,
            total_bytes: stats.stats.current_memory,
            peak_bytes: stats.stats.peak_memory,
            thread_count: stats.thread_stats.len(),
            by_type: self.by_type(),
        };
        debug!(
            "Metrics summary: {} allocations, {} bytes, {} types, {} threads",
            report.allocation_count,
            report.total_bytes,
            report.by_type.len(),
            report.thread_count
        );
        report
    }

    /// Get top allocations by size.
    pub fn top_by_size(&self, n: usize) -> Vec<AllocationMetric> {
        debug!("Getting top {} allocations by size", n);
        let mut allocs: Vec<_> = self.view.allocations();
        allocs.sort_by_key(|a| std::cmp::Reverse(a.size));
        let result: Vec<AllocationMetric> = allocs
            .into_iter()
            .take(n)
            .map(AllocationMetric::from_allocation)
            .collect();
        debug!("Returning {} top allocations", result.len());
        result
    }

    /// Get allocations by type.
    pub fn by_type(&self) -> HashMap<String, TypeMetric> {
        let mut types: HashMap<String, TypeMetric> = HashMap::new();
        for a in self.view.allocations() {
            let type_name = a.type_name.clone().unwrap_or_else(|| "unknown".to_string());
            let entry = types.entry(type_name).or_default();
            entry.count += 1;
            entry.total_bytes += a.size;
        }
        debug!("Computed metrics for {} types", types.len());
        types
    }

    /// Get allocations by thread.
    pub fn by_thread(&self) -> HashMap<u64, ThreadMetric> {
        let mut threads: HashMap<u64, ThreadMetric> = HashMap::new();
        for a in self.view.allocations() {
            let entry = threads.entry(a.thread_id).or_default();
            entry.thread_id = a.thread_id;
            entry.allocation_count += 1;
            entry.total_bytes += a.size;
        }
        debug!("Computed metrics for {} threads", threads.len());
        threads
    }

    /// Get size distribution.
    pub fn size_distribution(&self) -> SizeDistribution {
        let allocations = self.view.allocations();
        let sizes: Vec<usize> = allocations.iter().map(|a| a.size).collect();

        if sizes.is_empty() {
            debug!("Size distribution: no allocations");
            return SizeDistribution::default();
        }

        let min = *sizes.iter().min().unwrap_or(&0);
        let max = *sizes.iter().max().unwrap_or(&0);
        let avg = sizes.iter().sum::<usize>() / sizes.len();

        debug!("Size distribution: min={}, max={}, avg={}", min, max, avg);
        SizeDistribution { min, max, avg }
    }
}

/// Metric for a single allocation.
#[derive(Debug, Clone)]
pub struct AllocationMetric {
    /// Memory pointer
    pub ptr: usize,
    /// Allocation size
    pub size: usize,
    /// Type name
    pub type_name: Option<String>,
    /// Variable name
    pub var_name: Option<String>,
    /// Thread ID
    pub thread_id: u64,
}

impl AllocationMetric {
    fn from_allocation(alloc: &crate::snapshot::ActiveAllocation) -> Self {
        Self {
            ptr: alloc.ptr.unwrap_or(0),
            size: alloc.size,
            type_name: alloc.type_name.clone(),
            var_name: alloc.var_name.clone(),
            thread_id: alloc.thread_id,
        }
    }
}

/// Metric for a thread.
#[derive(Debug, Clone, Default)]
pub struct ThreadMetric {
    /// Thread ID
    pub thread_id: u64,
    /// Number of allocations
    pub allocation_count: usize,
    /// Total bytes
    pub total_bytes: usize,
}

/// Size distribution statistics.
#[derive(Debug, Clone, Default)]
pub struct SizeDistribution {
    /// Minimum allocation size
    pub min: usize,
    /// Maximum allocation size
    pub max: usize,
    /// Average allocation size
    pub avg: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::MemoryEvent;

    #[test]
    fn test_metrics_summary() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 2),
        ];
        let view = MemoryView::from_events(events);
        let analysis = MetricsAnalysis::from_view(&view);
        let summary = analysis.summary();
        assert_eq!(summary.allocation_count, 2);
        assert_eq!(summary.total_bytes, 192);
    }

    #[test]
    fn test_top_by_size() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 256, 1),
            MemoryEvent::allocate(0x3000, 128, 1),
        ];
        let view = MemoryView::from_events(events);
        let analysis = MetricsAnalysis::from_view(&view);
        let top = analysis.top_by_size(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].size, 256);
        assert_eq!(top[1].size, 128);
    }
}
