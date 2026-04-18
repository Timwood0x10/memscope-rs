//! MemScope - Unified facade for all engines
//!
//! This module provides the MemScope facade which unifies all engines
//! into a simple, easy-to-use interface.

use crate::analysis::detectors::Detector;
use crate::analysis_engine::{AnalysisEngine, Analyzer};
use crate::capture::{CaptureBackendType, CaptureEngine};
use crate::event_store::EventStore;
use crate::metadata::MetadataEngine;
use crate::query::QueryEngine;
use crate::render_engine::RenderEngine;
use crate::snapshot::SnapshotEngine;
use crate::timeline::TimelineEngine;
use std::sync::{Arc, Mutex};

/// MemScope - Unified facade for all engines
///
/// MemScope provides a simple, unified interface for memory tracking,
/// analysis, and visualization. It integrates all 9 engines into a
/// single, easy-to-use API.
///
/// Key properties:
/// - Simple: One-stop interface for all functionality
/// - Powerful: Access to all engines when needed
/// - Type-safe: Strong typing throughout
/// - Thread-safe: All operations are thread-safe
pub struct MemScope {
    /// Event Store - Centralized event storage
    pub event_store: Arc<EventStore>,
    /// Capture Engine - Event capture backend
    pub capture: Arc<CaptureEngine>,
    /// Metadata Engine - Centralized metadata management
    pub metadata: Arc<MetadataEngine>,
    /// Snapshot Engine - Snapshot construction and aggregation
    pub snapshot: Arc<SnapshotEngine>,
    /// Query Engine - Unified query interface
    pub query: Arc<QueryEngine>,
    /// Analysis Engine - Memory analysis logic (wrapped in Mutex for interior mutability)
    pub analysis: Arc<Mutex<AnalysisEngine>>,
    /// Timeline Engine - Time-based memory analysis
    pub timeline: Arc<TimelineEngine>,
    /// Render Engine - Output rendering
    pub render: Arc<RenderEngine>,
}

impl MemScope {
    /// Convert ActiveAllocation to AllocationInfo (helper function)
    fn to_allocation_info(
        active: &crate::snapshot::ActiveAllocation,
    ) -> crate::capture::types::AllocationInfo {
        let thread_id_u64 = active.thread_id;
        // Note: Rust's ThreadId type cannot be reconstructed from u64.
        // We use the current thread's ID as a placeholder.
        // The actual thread ID value is available in thread_id_u64 field.
        // This is a known limitation that requires tracking ThreadId at allocation time.
        let thread_id = std::thread::current().id();

        // For Container/Value types, ptr is None (no real heap allocation).
        // We use 0 as a sentinel value, which indicates "not a real pointer".
        // Callers should check TrackKind to determine if this is a real allocation.
        let ptr = match active.kind {
            crate::core::types::TrackKind::HeapOwner { ptr, .. } => ptr,
            crate::core::types::TrackKind::Container | crate::core::types::TrackKind::Value => 0,
        };

        crate::capture::types::AllocationInfo {
            ptr,
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
            module_path: None,
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

    /// Create a new MemScope instance
    ///
    /// This creates all engines with default settings and connects them
    /// together in the correct configuration.
    pub fn new() -> Self {
        // Create EventStore (the foundation)
        let event_store = Arc::new(EventStore::new());

        // Create Capture Engine
        let capture = Arc::new(CaptureEngine::new(
            CaptureBackendType::Unified,
            event_store.clone(),
        ));

        // Create Metadata Engine
        let metadata = Arc::new(MetadataEngine::new());

        // Create Snapshot Engine
        let snapshot = Arc::new(SnapshotEngine::new(event_store.clone()));

        // Create Query Engine
        let query = Arc::new(QueryEngine::new(snapshot.clone()));

        // Create Analysis Engine
        let analysis = Arc::new(Mutex::new(AnalysisEngine::new(snapshot.clone())));

        // Create Timeline Engine
        let timeline = Arc::new(TimelineEngine::new(event_store.clone()));

        // Create Render Engine
        let render = Arc::new(RenderEngine::new(snapshot.clone()));

        Self {
            event_store,
            capture,
            metadata,
            snapshot,
            query,
            analysis,
            timeline,
            render,
        }
    }

    /// Create a new MemScope instance with a specific capture backend
    ///
    /// # Arguments
    /// * `backend_type` - The type of capture backend to use
    pub fn with_backend(backend_type: CaptureBackendType) -> Self {
        // Create EventStore (the foundation)
        let event_store = Arc::new(EventStore::new());

        // Create Capture Engine with specified backend
        let capture = Arc::new(CaptureEngine::new(backend_type, event_store.clone()));

        // Create Metadata Engine
        let metadata = Arc::new(MetadataEngine::new());

        // Create Snapshot Engine
        let snapshot = Arc::new(SnapshotEngine::new(event_store.clone()));

        // Create Query Engine
        let query = Arc::new(QueryEngine::new(snapshot.clone()));

        // Create Analysis Engine
        let analysis = Arc::new(Mutex::new(AnalysisEngine::new(snapshot.clone())));

        // Create Timeline Engine
        let timeline = Arc::new(TimelineEngine::new(event_store.clone()));

        // Create Render Engine
        let render = Arc::new(RenderEngine::new(snapshot.clone()));

        Self {
            event_store,
            capture,
            metadata,
            snapshot,
            query,
            analysis,
            timeline,
            render,
        }
    }

    /// Register an analyzer with the analysis engine
    ///
    /// # Arguments
    /// * `analyzer` - The analyzer to register
    pub fn register_analyzer(&self, analyzer: Box<dyn Analyzer>) {
        if let Ok(mut analysis) = self.analysis.lock() {
            analysis.register_analyzer(analyzer);
            tracing::info!("Analyzer registered successfully");
        } else {
            tracing::error!("Failed to acquire analysis engine lock for registration");
        }
    }

    /// Get a summary of current memory usage
    pub fn summary(&self) -> crate::query::QueryResult {
        self.query.summary()
    }

    /// Get top allocations by size
    ///
    /// # Arguments
    /// * `limit` - Maximum number of allocations to return
    pub fn top_allocations(&self, limit: usize) -> crate::query::QueryResult {
        self.query.top_allocations(limit)
    }

    /// Render current snapshot to JSON
    ///
    /// # Arguments
    /// * `verbose` - Whether to include verbose output
    pub fn render_json(&self, verbose: bool) -> Result<crate::render_engine::RenderResult, String> {
        let snapshot = self.snapshot.build_snapshot();
        self.render.render_json(&snapshot, verbose)
    }

    /// Clear all events and reset state
    pub fn clear(&self) {
        self.event_store.clear();
    }

    /// Get the total number of events
    pub fn event_count(&self) -> usize {
        self.event_store.len()
    }

    // ===== Detector Methods =====

    /// Register a detector with the analysis engine
    ///
    /// This method automatically wraps the detector in an adapter
    /// and registers it as an analyzer.
    ///
    /// # Arguments
    /// * `detector` - The detector to register
    pub fn register_detector<D>(&self, detector: D)
    where
        D: crate::analysis::detectors::Detector + Send + Sync + 'static,
    {
        use crate::analysis_engine::DetectorToAnalyzer;
        let adapter = Box::new(DetectorToAnalyzer::new(detector));
        self.register_analyzer(adapter);
    }

    /// Run all registered detectors and return the results
    ///
    /// # Returns
    /// A vector of analysis results from all detectors
    pub fn run_detectors(&self) -> Vec<crate::analysis_engine::analyzer::AnalysisResult> {
        if let Ok(analysis) = self.analysis.lock() {
            analysis.analyze()
        } else {
            tracing::error!("Failed to acquire analysis engine lock");
            vec![]
        }
    }

    /// Run the leak detector on the current snapshot
    ///
    /// # Returns
    /// Detection results from the leak detector
    pub fn run_leak_detector(&self) -> crate::analysis::detectors::DetectionResult {
        use crate::analysis::detectors::{LeakDetector, LeakDetectorConfig};
        let detector = LeakDetector::new(LeakDetectorConfig::default());
        let snapshot = self.snapshot.build_snapshot();
        let allocations: Vec<crate::capture::types::AllocationInfo> = snapshot
            .active_allocations
            .values()
            .map(Self::to_allocation_info)
            .collect();
        detector.detect(&allocations)
    }

    /// Run the UAF (Use-After-Free) detector on the current snapshot
    ///
    /// # Returns
    /// Detection results from the UAF detector
    pub fn run_uaf_detector(&self) -> crate::analysis::detectors::DetectionResult {
        use crate::analysis::detectors::{UafDetector, UafDetectorConfig};
        let detector = UafDetector::new(UafDetectorConfig::default());
        let snapshot = self.snapshot.build_snapshot();
        let allocations: Vec<crate::capture::types::AllocationInfo> = snapshot
            .active_allocations
            .values()
            .map(Self::to_allocation_info)
            .collect();
        detector.detect(&allocations)
    }

    /// Run the overflow detector on the current snapshot
    ///
    /// # Returns
    /// Detection results from the overflow detector
    pub fn run_overflow_detector(&self) -> crate::analysis::detectors::DetectionResult {
        use crate::analysis::detectors::{OverflowDetector, OverflowDetectorConfig};
        let detector = OverflowDetector::new(OverflowDetectorConfig::default());
        let snapshot = self.snapshot.build_snapshot();
        let allocations: Vec<crate::capture::types::AllocationInfo> = snapshot
            .active_allocations
            .values()
            .map(Self::to_allocation_info)
            .collect();
        detector.detect(&allocations)
    }

    /// Run the safety detector on the current snapshot
    ///
    /// # Returns
    /// Detection results from the safety detector
    pub fn run_safety_detector(&self) -> crate::analysis::detectors::DetectionResult {
        use crate::analysis::detectors::{SafetyDetector, SafetyDetectorConfig};
        let detector = SafetyDetector::new(SafetyDetectorConfig::default());
        let snapshot = self.snapshot.build_snapshot();
        let allocations: Vec<crate::capture::types::AllocationInfo> = snapshot
            .active_allocations
            .values()
            .map(Self::to_allocation_info)
            .collect();
        detector.detect(&allocations)
    }

    /// Run the lifecycle detector on the current snapshot
    ///
    /// # Returns
    /// Detection results from the lifecycle detector
    pub fn run_lifecycle_detector(&self) -> crate::analysis::detectors::DetectionResult {
        use crate::analysis::detectors::{LifecycleDetector, LifecycleDetectorConfig};
        let detector = LifecycleDetector::new(LifecycleDetectorConfig::default());
        let snapshot = self.snapshot.build_snapshot();
        let allocations: Vec<crate::capture::types::AllocationInfo> = snapshot
            .active_allocations
            .values()
            .map(Self::to_allocation_info)
            .collect();
        detector.detect(&allocations)
    }

    // ===== Export Methods =====

    /// Export the current memory snapshot as an HTML dashboard with a specific template
    ///
    /// # Arguments
    /// * `path` - Directory path where the HTML file will be saved
    /// * `template` - The dashboard template to use
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn export_html_with_template<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        template: crate::render_engine::export::DashboardTemplate,
    ) -> Result<(), String> {
        use crate::render_engine::export::export_dashboard_html_with_template;
        use crate::tracker::Tracker;
        use std::sync::Arc;

        // Create a new tracker instance for export
        let tracker = Tracker::new();
        let passport_tracker =
            Arc::new(crate::analysis::memory_passport_tracker::get_global_passport_tracker());

        export_dashboard_html_with_template(path, &tracker, &passport_tracker, template, None)
            .map_err(|e| format!("Failed to export HTML: {}", e))
    }

    /// Export the current memory snapshot as an HTML dashboard
    ///
    /// This method automatically detects program characteristics and selects
    /// the most appropriate template:
    /// - Multithread: If the program uses multiple threads
    /// - Binary: Default template for single-threaded programs
    ///
    /// # Arguments
    /// * `path` - Directory path where the HTML file will be saved
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn export_html<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), String> {
        use crate::render_engine::export::DashboardTemplate;

        let _snapshot = self.snapshot.build_snapshot();
        self.export_html_with_template(path, DashboardTemplate::Unified)
    }

    /// Export all JSON files
    ///
    /// This method exports 9 JSON files containing comprehensive memory analysis:
    /// - memory_analysis.json: Complete memory allocation analysis
    /// - lifetime.json: Ownership and lifetime tracking
    /// - ownership_graph.json: Ownership graph analysis with cycle detection
    /// - system_resources.json: System resource monitoring
    /// - thread_analysis.json: Thread-specific memory stats
    /// - unsafe_ffi.json: Unsafe FFI boundary tracking
    /// - memory_passports.json: Memory lifecycle passports
    /// - leak_detection.json: Potential memory leaks
    /// - async_analysis.json: Async task analysis
    ///
    /// # Arguments
    /// * `path` - Directory path where JSON files will be saved
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn export_json<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), String> {
        use crate::render_engine::export::export_all_json;
        use crate::tracker::Tracker;
        use std::sync::Arc;

        let tracker = Tracker::new();
        let passport_tracker =
            Arc::new(crate::analysis::memory_passport_tracker::get_global_passport_tracker());
        let async_tracker = Arc::new(crate::capture::backends::async_tracker::AsyncTracker::new());

        export_all_json(path, &tracker, &passport_tracker, &async_tracker)
            .map_err(|e| format!("Failed to export JSON files: {}", e))
    }
}

impl Default for MemScope {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memscope_creation() {
        let memscope = MemScope::new();
        assert_eq!(memscope.event_count(), 0);
    }

    #[test]
    fn test_memscope_default() {
        let memscope = MemScope::default();
        assert_eq!(memscope.event_count(), 0);
    }

    #[test]
    fn test_memscope_with_backend() {
        let memscope = MemScope::with_backend(CaptureBackendType::Core);
        assert_eq!(memscope.event_count(), 0);
    }

    #[test]
    fn test_summary() {
        let memscope = MemScope::new();
        let result = memscope.summary();
        match result {
            crate::query::QueryResult::Summary(_) => (),
            _ => panic!("Expected summary result"),
        }
    }

    #[test]
    fn test_top_allocations() {
        let memscope = MemScope::new();
        let result = memscope.top_allocations(10);
        match result {
            crate::query::QueryResult::Allocations(_) => (),
            _ => panic!("Expected allocations result"),
        }
    }

    #[test]
    fn test_render_json() {
        let memscope = MemScope::new();
        let result = memscope.render_json(false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear() {
        let memscope = MemScope::new();
        // Simulate some events
        memscope.capture.capture_alloc(0x1000, 1024, 1);
        assert_eq!(memscope.event_count(), 1);

        memscope.clear();
        assert_eq!(memscope.event_count(), 0);
    }
}
