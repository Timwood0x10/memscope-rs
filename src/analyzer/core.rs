//! Analyzer - Unified analysis entry point.

use crate::analyzer::report::{
    AnalysisReport, CycleReport, LeakReport, MemoryStatsReport, MetricsReport,
};
use crate::view::MemoryView;

use super::{DetectionAnalysis, ExportEngine, GraphAnalysis, MetricsAnalysis, TimelineAnalysis};

/// Unified analysis entry point.
///
/// Provides access to all analysis modules through a single interface.
/// Uses lazy initialization for expensive operations.
pub struct Analyzer {
    view: MemoryView,
    graph: Option<GraphAnalysis>,
    detect: Option<DetectionAnalysis>,
    metrics: Option<MetricsAnalysis>,
    timeline: Option<TimelineAnalysis>,
}

impl Analyzer {
    /// Create analyzer from GlobalTracker.
    pub fn from_tracker(
        tracker: &crate::capture::backends::global_tracking::GlobalTracker,
    ) -> Self {
        Self::from_view(MemoryView::from_tracker(tracker))
    }

    /// Create analyzer from view.
    pub fn from_view(view: MemoryView) -> Self {
        Self {
            view,
            graph: None,
            detect: None,
            metrics: None,
            timeline: None,
        }
    }

    /// Get graph analysis (lazy).
    pub fn graph(&mut self) -> &mut GraphAnalysis {
        if self.graph.is_none() {
            self.graph = Some(GraphAnalysis::from_view(&self.view));
        }
        self.graph
            .as_mut()
            .expect("GraphAnalysis should be initialized after lazy initialization")
    }

    /// Get detection analysis (lazy).
    pub fn detect(&mut self) -> &DetectionAnalysis {
        if self.detect.is_none() {
            self.detect = Some(DetectionAnalysis::from_view(&self.view));
        }
        self.detect
            .as_ref()
            .expect("DetectionAnalysis should be initialized after lazy initialization")
    }

    /// Get metrics analysis (lazy).
    pub fn metrics(&mut self) -> &MetricsAnalysis {
        if self.metrics.is_none() {
            self.metrics = Some(MetricsAnalysis::from_view(&self.view));
        }
        self.metrics
            .as_ref()
            .expect("MetricsAnalysis should be initialized after lazy initialization")
    }

    /// Get timeline analysis (lazy).
    pub fn timeline(&mut self) -> &TimelineAnalysis {
        if self.timeline.is_none() {
            self.timeline = Some(TimelineAnalysis::from_view(&self.view));
        }
        self.timeline
            .as_ref()
            .expect("TimelineAnalysis should be initialized after lazy initialization")
    }

    /// Get export engine.
    pub fn export(&self) -> ExportEngine<'_> {
        ExportEngine::new(&self.view)
    }

    /// Get underlying view.
    pub fn view(&self) -> &MemoryView {
        &self.view
    }

    /// Run full analysis on the memory data.
    ///
    /// This method performs a comprehensive analysis of all tracked memory
    /// allocations and returns a complete report.
    ///
    /// # Analysis Pipeline
    ///
    /// The analysis is performed in the following order:
    ///
    /// 1. **Statistics Collection** (O(1))
    ///    - Total allocation count
    ///    - Total bytes allocated
    ///    - Peak memory usage
    ///    - Thread count
    ///
    /// 2. **Leak Detection** (O(n))
    ///    - Identifies allocations that were never deallocated
    ///    - Reports leaked bytes and allocation details
    ///
    /// 3. **Cycle Detection** (O(V + E))
    ///    - Uses DFS-based algorithm to detect reference cycles
    ///    - Groups allocations by type for potential cycle identification
    ///
    /// 4. **Metrics Summary** (O(n))
    ///    - Aggregates allocation statistics by type
    ///    - Reports top allocations by size
    ///
    /// # Performance Characteristics
    ///
    /// - **Time Complexity**: O(n + V + E) where n is the number of allocations,
    ///   V is the number of unique pointers, and E is the number of edges
    /// - **Space Complexity**: O(n) for storing analysis results
    /// - **Memory Overhead**: Minimal, results are computed on-demand
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut analyzer = analyzer(&tracker)?;
    /// let report = analyzer.analyze();
    ///
    /// println!("Allocations: {}", report.stats.allocation_count);
    /// println!("Leaks: {}", report.leaks.leak_count);
    /// println!("Cycles: {}", report.cycles.cycle_count);
    /// ```
    ///
    /// # Returns
    ///
    /// An `AnalysisReport` containing:
    /// - `stats`: Basic memory statistics
    /// - `leaks`: Memory leak detection results
    /// - `cycles`: Reference cycle detection results
    /// - `metrics`: Aggregated metrics summary
    pub fn analyze(&mut self) -> AnalysisReport {
        AnalysisReport {
            stats: MemoryStatsReport {
                allocation_count: self.view.len(),
                total_bytes: self.view.total_memory(),
                peak_bytes: self.view.snapshot().stats.peak_memory,
                thread_count: self.view.snapshot().thread_stats.len(),
            },
            leaks: self.detect().leaks(),
            cycles: self.graph().cycles(),
            metrics: self.metrics().summary(),
        }
    }

    /// Quick leak check.
    pub fn quick_leak_check(&mut self) -> LeakReport {
        self.detect().leaks()
    }

    /// Quick cycle check.
    pub fn quick_cycle_check(&mut self) -> CycleReport {
        self.graph().cycles()
    }

    /// Quick metrics.
    pub fn quick_metrics(&mut self) -> MetricsReport {
        self.metrics().summary()
    }
}

impl Clone for Analyzer {
    fn clone(&self) -> Self {
        Self {
            view: self.view.clone(),
            graph: None,
            detect: None,
            metrics: None,
            timeline: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::MemoryEvent;

    #[test]
    fn test_analyzer_from_events() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 2),
        ];
        let view = MemoryView::from_events(events);
        let mut analyzer = Analyzer::from_view(view);
        // Only test detection (no heap scanning required)
        let leaks = analyzer.quick_leak_check();
        assert_eq!(leaks.leak_count, 2);
        // Test metrics (no heap scanning required)
        let metrics = analyzer.quick_metrics();
        assert_eq!(metrics.allocation_count, 2);
    }

    #[test]
    fn test_quick_leak_check() {
        let events = vec![MemoryEvent::allocate(0x1000, 64, 1)];
        let view = MemoryView::from_events(events);
        let mut analyzer = Analyzer::from_view(view);
        let leaks = analyzer.quick_leak_check();
        assert_eq!(leaks.leak_count, 1);
    }
}
