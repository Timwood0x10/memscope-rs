//! Detection analysis module.

use crate::analyzer::report::{LeakReport, SafetyReport, UafReport};
use crate::snapshot::ActiveAllocation;
use crate::view::MemoryView;
use tracing::{debug, warn};

/// Detection analysis module.
///
/// Provides leak, UAF, and safety detection.
pub struct DetectionAnalysis {
    view: MemoryView,
}

impl DetectionAnalysis {
    /// Create from view.
    pub fn from_view(view: &MemoryView) -> Self {
        debug!("Creating DetectionAnalysis with {} allocations", view.len());
        Self { view: view.clone() }
    }

    /// Allocations that are still live at the cut.
    ///
    /// This is a **fact**, not a leak verdict: an allocation that was not released
    /// inside the capture window is simply still live there.
    pub fn unreleased(&self) -> Vec<&ActiveAllocation> {
        self.view
            .allocations()
            .into_iter()
            .filter(|a| a.ptr.is_some())
            .collect()
    }

    /// Detect memory leaks.
    ///
    /// A leak is a **conclusion** that needs an assessment rule plus evidence about
    /// capture completeness (plan §4.5). The view only carries the active set, so no
    /// allocation can be assessed here and this returns an empty report.
    ///
    /// The historical implementation reported *every* live allocation as a leak
    /// (`filter(|a| a.ptr.is_some())`), which made any normal program look like it was
    /// leaking everything it held (plan A02). Use [`Self::unreleased`] when the caller
    /// needs the "still live, no release observed" list.
    pub fn leaks(&self) -> LeakReport {
        let unreleased = self.unreleased();
        if !unreleased.is_empty() {
            debug!(
                "Leak assessment not performed: {} allocations are still live at the cut",
                unreleased.len()
            );
        }

        LeakReport {
            leak_count: 0,
            total_leaked_bytes: 0,
            leaked_allocations: Vec::new(),
        }
    }

    /// Detect use-after-free.
    ///
    /// Analyzes event stream for potential UAF patterns by tracking
    /// deallocations and checking for subsequent accesses to the same address.
    ///
    /// Note:
    /// - Reallocations are NOT flagged as UAF because realloc on a
    ///   deallocated pointer is valid behavior (the allocator may reuse the address).
    /// - Metadata events are NOT flagged as UAF because they represent
    ///   type/container information, not actual memory access.
    /// - Only actual memory access events (read/write operations) on freed
    ///   memory would indicate UAF, but these are not currently tracked.
    ///
    /// Returns an empty report as UAF detection requires runtime memory
    /// access tracking which is not implemented in the current architecture.
    pub fn uaf(&self) -> UafReport {
        // UAF detection requires tracking actual memory accesses (read/write)
        // on freed pointers. The current event system does not track these
        // operations. Realloc and Metadata events are not UAF indicators:
        // - Realloc: valid allocator behavior, may reuse addresses
        // - Metadata: type information only, no memory access
        //
        // For proper UAF detection, consider using sanitizers like ASAN
        // or instrumenting memory access at runtime.
        debug!("UAF detection: not implemented, returning empty report");
        UafReport::empty()
    }

    /// Analyze memory safety.
    ///
    /// Checks for common safety issues.
    pub fn safety(&self) -> SafetyReport {
        let allocations = self.view.allocations();
        let report = SafetyReport::from_allocations(&allocations);
        if report.issue_count > 0 {
            warn!(
                "Safety analysis: found {} potential safety issues",
                report.issue_count
            );
        }
        report
    }

    /// Get detection summary.
    pub fn summary(&self) -> DetectionSummary {
        let leaks = self.leaks();
        let safety = self.safety();
        DetectionSummary {
            leak_count: leaks.leak_count,
            leaked_bytes: leaks.total_leaked_bytes,
            uaf_count: 0, // UAF detection not implemented
            safety_issues: safety.issue_count,
        }
    }
}

/// Detection summary.
#[derive(Debug, Clone)]
pub struct DetectionSummary {
    /// Number of leaks
    pub leak_count: usize,
    /// Total leaked bytes
    pub leaked_bytes: usize,
    /// Number of UAF issues
    pub uaf_count: usize,
    /// Number of safety issues
    pub safety_issues: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::MemoryEvent;

    /// Objective: Verify that a still-live allocation is reported as "not released"
    /// and *not* as a leak (plan A02).
    /// Invariants: one unreleased allocation of 128 bytes, zero leak verdicts.
    #[test]
    fn test_live_allocation_is_not_a_leak() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 2),
            MemoryEvent::deallocate(0x1000, 64, 1),
        ];
        let view = MemoryView::from_events(events);
        let analysis = DetectionAnalysis::from_view(&view);

        let unreleased = analysis.unreleased();
        assert_eq!(
            unreleased.len(),
            1,
            "one allocation is still live at the cut"
        );
        assert_eq!(
            unreleased[0].size, 128,
            "the unreleased allocation keeps its recorded size"
        );

        let leaks = analysis.leaks();
        assert_eq!(
            leaks.leak_count, 0,
            "being alive at the cut is not evidence of a leak"
        );
        assert_eq!(
            leaks.total_leaked_bytes, 0,
            "no leaked bytes may be reported without an assessment"
        );
    }

    #[test]
    fn test_no_leaks() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::deallocate(0x1000, 64, 1),
        ];
        let view = MemoryView::from_events(events);
        let analysis = DetectionAnalysis::from_view(&view);
        let leaks = analysis.leaks();
        assert_eq!(leaks.leak_count, 0);
    }

    #[test]
    fn test_uaf_returns_empty() {
        // UAF detection is not implemented, should return empty report
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::deallocate(0x1000, 64, 1),
        ];
        let view = MemoryView::from_events(events);
        let analysis = DetectionAnalysis::from_view(&view);
        let uaf = analysis.uaf();
        assert_eq!(uaf.uaf_count, 0);
    }
}
