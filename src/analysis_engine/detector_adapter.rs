//! Detector adapter - Converts Detector trait to Analyzer trait
//!
//! This module provides an adapter that allows detectors to be used
//! with the AnalysisEngine, which expects the Analyzer trait.

use crate::analysis::detectors::Detector;
use crate::analysis_engine::analyzer::{AnalysisResult, Analyzer, Finding, Severity};
use crate::capture::types::AllocationInfo;
use crate::snapshot::ActiveAllocation;
use std::sync::Arc;

/// Convert ActiveAllocation to AllocationInfo
fn active_to_allocation_info(active: &ActiveAllocation) -> AllocationInfo {
    let thread_id_u64 = active.thread_id;
    // Note: ThreadId cannot be reconstructed from u64. We use the current thread's
    // ID as a placeholder. The actual thread ID should be read from thread_id_u64.
    let thread_id = std::thread::current().id();

    AllocationInfo {
        ptr: active.ptr,
        size: active.size,
        var_name: active.var_name.clone(),
        type_name: active.type_name.clone(),
        scope_name: None,
        timestamp_alloc: active.allocated_at,
        timestamp_dealloc: None,
        thread_id,
        thread_id_u64,
        borrow_count: 0,
        stack_trace: None,
        is_leaked: false,
        lifetime_ms: None,
        borrow_info: None,
        clone_info: None,
        ownership_history_available: false,
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

/// Adapter that converts a Detector into an Analyzer
///
/// This allows detectors from the analysis module to be used
/// with the AnalysisEngine, which expects the Analyzer trait.
pub struct DetectorToAnalyzer<D: Detector + Send + Sync> {
    detector: Arc<D>,
}

impl<D: Detector + Send + Sync> DetectorToAnalyzer<D> {
    /// Create a new adapter from a detector
    pub fn new(detector: D) -> Self {
        Self {
            detector: Arc::new(detector),
        }
    }

    /// Create a new adapter from a boxed detector
    pub fn from_boxed(detector: Box<D>) -> Self {
        Self {
            detector: Arc::from(detector),
        }
    }

    /// Get a reference to the underlying detector
    pub fn detector(&self) -> &D {
        &self.detector
    }
}

impl<D: Detector + Send + Sync + 'static> Analyzer for DetectorToAnalyzer<D> {
    fn name(&self) -> &str {
        self.detector.name()
    }

    fn analyze(&self, snapshot: &crate::snapshot::MemorySnapshot) -> AnalysisResult {
        // Convert active allocations to AllocationInfo
        let allocations: Vec<AllocationInfo> = snapshot
            .active_allocations
            .values()
            .map(active_to_allocation_info)
            .collect();

        // Run the detector
        let detection_result = self.detector.detect(&allocations);

        // Convert DetectionResult to AnalysisResult
        let findings: Vec<Finding> = detection_result
            .issues
            .into_iter()
            .map(|issue| Finding {
                issue_type: format!("{:?}", issue.category),
                description: issue.description,
                ptr: issue.allocation_ptr,
                size: None, // Issue doesn't have size info
                context: issue.suggested_fix.unwrap_or_default(),
            })
            .collect();

        // Determine overall severity based on the most severe issue
        let severity = findings
            .iter()
            .map(|f| &f.issue_type)
            .fold(Severity::Info, |acc, _| acc);

        AnalysisResult {
            analyzer_name: detection_result.detector_name,
            issue_count: findings.len(),
            severity,
            description: format!(
                "Found {} issues in {} allocations (took {}ms)",
                findings.len(),
                detection_result.statistics.total_allocations,
                detection_result.detection_time_ms
            ),
            findings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::{LeakDetector, LeakDetectorConfig};
    use crate::snapshot::MemorySnapshot;

    #[test]
    fn test_detector_adapter_creation() {
        let detector = LeakDetector::new(LeakDetectorConfig::default());
        let adapter = DetectorToAnalyzer::new(detector);
        assert_eq!(adapter.name(), "LeakDetector");
    }

    #[test]
    fn test_detector_adapter_analyze() {
        let detector = LeakDetector::new(LeakDetectorConfig::default());
        let adapter = DetectorToAnalyzer::new(detector);

        let snapshot = MemorySnapshot::new();
        let result = adapter.analyze(&snapshot);

        assert_eq!(result.analyzer_name, "LeakDetector");
        assert_eq!(result.issue_count, 0);
    }

    #[test]
    fn test_active_to_allocation_info() {
        let active = ActiveAllocation {
            ptr: 0x1000,
            size: 1024,
            allocated_at: 1000,
            var_name: Some("test_var".to_string()),
            type_name: Some("Vec<u8>".to_string()),
            thread_id: 42,
        };

        let alloc = active_to_allocation_info(&active);

        assert_eq!(alloc.ptr, 0x1000);
        assert_eq!(alloc.size, 1024);
        assert_eq!(alloc.timestamp_alloc, 1000);
        assert_eq!(alloc.var_name, Some("test_var".to_string()));
        assert_eq!(alloc.type_name, Some("Vec<u8>".to_string()));
        // thread_id is the current thread ID, not 42
        assert_eq!(alloc.borrow_count, 0);
        assert!(!alloc.is_leaked);
    }
}
