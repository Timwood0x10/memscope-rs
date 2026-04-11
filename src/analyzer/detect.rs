//! Detection analysis module.

use crate::analyzer::report::{LeakInfo, LeakReport, SafetyReport, UafReport};
use crate::snapshot::ActiveAllocation;
use crate::view::MemoryView;
use tracing::{debug, info, warn};

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

    /// Detect memory leaks.
    ///
    /// A leak is an allocation that was never deallocated.
    ///
    /// # Filter Logic
    ///
    /// Uses `ptr.is_some()` to filter allocations because:
    /// - `ptr.is_some()`: Heap allocation with a valid pointer → potential leak
    /// - `ptr.is_none()`: Container/Value type (metadata, not heap memory) → not a leak
    ///
    /// Container/Value types represent stack-allocated container metadata
    /// (e.g., the Vec struct itself), not heap allocations. Only heap
    /// allocations can be memory leaks.
    pub fn leaks(&self) -> LeakReport {
        let allocations = self.view.allocations();
        let leaked: Vec<&ActiveAllocation> = allocations
            .into_iter()
            .filter(|a| a.ptr.is_some())
            .collect();

        let leak_count = leaked.len();
        let total_bytes: usize = leaked.iter().map(|a| a.size).sum();

        if leak_count > 0 {
            info!(
                "Leak detection: found {} leaks totaling {} bytes",
                leak_count, total_bytes
            );
        } else {
            debug!("Leak detection: no leaks found");
        }

        LeakReport {
            leak_count,
            total_leaked_bytes: total_bytes,
            leaked_allocations: leaked.into_iter().map(LeakInfo::from).collect(),
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

    #[test]
    fn test_leak_detection() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 64, 1),
            MemoryEvent::allocate(0x2000, 128, 2),
            MemoryEvent::deallocate(0x1000, 64, 1),
        ];
        let view = MemoryView::from_events(events);
        let analysis = DetectionAnalysis::from_view(&view);
        let leaks = analysis.leaks();
        assert_eq!(leaks.leak_count, 1);
        assert_eq!(leaks.total_leaked_bytes, 128);
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
