//! Memory analysis functionality
//!
//! This module provides advanced analysis capabilities:
//! - Enhanced memory analysis
//! - Circular reference detection
//! - Unsafe FFI tracking
//! - Unknown memory region analysis
//! - Type classification
//! - Performance metrics
//! - Quality assurance
//! - Size estimation
//! - Dedicated detectors (LeakDetector, UafDetector, etc.)

// Detector modules
pub mod detectors;

pub mod circular_reference;
pub mod heap_scanner;
pub mod relation_inference;
pub mod unsafe_ffi_tracker;
pub mod unsafe_inference;
pub mod variable_relationships;

// Submodules from refactoring
pub mod closure;
pub mod generic;
pub mod safety;
pub mod security;
pub mod unknown;

// New analysis modules for ComplexTypeForRust.md features
pub mod async_analysis;
pub mod borrow_analysis;

pub mod ffi_function_resolver;
pub mod lifecycle;
pub mod lifecycle_analysis;
pub mod memory_passport_tracker;

// Integrated analysis submodules
pub mod classification;
pub mod estimation;
pub mod metrics;
pub mod quality;

// Re-export key analysis functions
pub use circular_reference::{CircularReference, CircularReferenceAnalysis, CircularReferenceNode};
pub use unsafe_ffi_tracker::UnsafeFFITracker;
pub use variable_relationships::{
    build_variable_relationship_graph, GraphStatistics, RelationshipType as VarRelationshipType,
    SmartPointerInfo as VarSmartPointerInfo, VariableCategory, VariableCluster, VariableNode,
    VariableRelationship, VariableRelationshipGraph,
};

// Relationship cycle detection
pub mod relationship_cycle_detector;
pub use relationship_cycle_detector::{detect_cycles_in_relationships, CycleDetectionResult};

// Ownership graph analysis
pub mod ownership_graph;
pub use ownership_graph::{Edge, EdgeKind, Node, ObjectId, OwnershipGraph, OwnershipOp};

// Re-export new analysis modules
pub use detectors::{
    Detector, LeakDetector, LeakDetectorConfig, LifecycleDetector, LifecycleDetectorConfig,
    OverflowDetector, OverflowDetectorConfig, SafetyDetector, SafetyDetectorConfig, UafDetector,
    UafDetectorConfig,
};

pub use async_analysis::{
    get_global_async_analyzer, AsyncAnalyzer, AsyncPatternAnalysis, AsyncStatistics,
};
pub use borrow_analysis::{get_global_borrow_analyzer, BorrowAnalyzer, BorrowPatternAnalysis};
pub use closure::{get_global_closure_analyzer, ClosureAnalysisReport, ClosureAnalyzer};
pub use ffi_function_resolver::{
    get_global_ffi_resolver, initialize_global_ffi_resolver, FfiFunctionCategory,
    FfiFunctionResolver, FfiRiskLevel, ResolutionStats, ResolvedFfiFunction, ResolverConfig,
};
pub use generic::{get_global_generic_analyzer, GenericAnalyzer, GenericStatistics};
pub use lifecycle::{
    lifecycle_summary::{
        AllocationLifecycleSummary, ExportMetadata, LifecycleEvent, LifecycleEventSummary,
        LifecycleExportData, LifecyclePattern, LifecycleSummaryGenerator, SummaryConfig,
        VariableGroup,
    },
    ownership_history::{
        BorrowInfo, CloneInfo, OwnershipEvent, OwnershipEventType, OwnershipHistoryRecorder,
        OwnershipStatistics, OwnershipSummary, RefCountInfo,
    },
};
pub use lifecycle_analysis::{
    get_global_lifecycle_analyzer, LifecycleAnalysisReport, LifecycleAnalyzer,
};
pub use memory_passport_tracker::{
    get_global_passport_tracker, initialize_global_passport_tracker, LeakDetail,
    LeakDetectionResult, MemoryPassport, MemoryPassportTracker, PassportEvent, PassportEventType,
    PassportStatus, PassportTrackerConfig, PassportTrackerStats,
};
pub use safety::{
    DynamicViolation, RiskAssessment, RiskFactor, RiskFactorType, SafetyAnalysisConfig,
    SafetyAnalysisStats, SafetyAnalyzer, UnsafeReport, UnsafeSource,
};
pub use unsafe_ffi_tracker::ComprehensiveSafetyReport;
pub use unsafe_inference::{
    count_valid_pointers, get_valid_regions, is_valid_ptr, is_valid_ptr_static, InferenceMethod,
    InferenceRecord, MemoryRegion, MemoryView, TypeGuess, TypeKind, UnsafeInferenceEngine,
    ValidRegions,
};

// Heap scanner exports
pub use heap_scanner::{HeapScanner, ScanResult};

// Relation inference exports
pub use relation_inference::{
    detect_clones, detect_owner, detect_slice, CloneConfig, GraphBuilderConfig, RangeMap, Relation,
    RelationEdge, RelationGraph, RelationGraphBuilder,
};

// Re-export integrated submodules
pub use classification::{
    pattern_matcher::PatternMatcher,
    rule_engine::{Rule as ClassificationRule, RuleEngine},
    type_classifier::{TypeCategory, TypeClassifier},
};
pub use estimation::{
    size_estimator::SizeEstimator, type_classifier::TypeClassifier as EstimationTypeClassifier,
};
pub use metrics::{
    analyzer::{Benchmark, PerformanceAnalyzer, PerformanceReport},
    collector::{Metric, MetricType, MetricValue, MetricsCollector},
    reporter::{AlertThreshold, MetricsReporter, ReportFormat},
};
pub use quality::{
    analyzer::{AnalysisReport, CodeAnalyzer, QualityMetric},
    checker::{MemoryLeakChecker, PerformanceChecker, SafetyChecker},
    validator::{QualityValidator, ValidationResult, ValidationRule},
};

use crate::capture::types::stats::FragmentationAnalysis;
use crate::capture::types::*;
use std::sync::Arc;

/// Main analysis interface - consolidates all analysis functionality
pub struct AnalysisManager {
    // This will contain the consolidated analysis functionality
}

impl AnalysisManager {
    /// Create a new analysis manager instance
    pub fn new() -> Self {
        Self {}
    }

    /// Analyze memory fragmentation
    pub fn analyze_fragmentation(&self, allocations: &[AllocationInfo]) -> FragmentationAnalysis {
        if allocations.is_empty() {
            return FragmentationAnalysis::default();
        }

        let active_allocations: Vec<_> = allocations
            .iter()
            .filter(|a| a.timestamp_dealloc.is_none())
            .collect();

        if active_allocations.is_empty() {
            return FragmentationAnalysis::default();
        }

        let mut sorted_ptrs: Vec<usize> = active_allocations.iter().map(|a| a.ptr).collect();
        sorted_ptrs.sort();

        let mut gaps: Vec<usize> = Vec::new();
        for i in 1..sorted_ptrs.len() {
            let prev = sorted_ptrs[i - 1];
            let curr = sorted_ptrs[i];
            if curr > prev {
                let prev_alloc = active_allocations
                    .iter()
                    .find(|a| a.ptr == prev)
                    .map(|a| a.size)
                    .unwrap_or(0);
                let gap = curr.saturating_sub(prev + prev_alloc);
                if gap > 0 {
                    gaps.push(gap);
                }
            }
        }

        let total_memory: usize = active_allocations.iter().map(|a| a.size).sum();
        let total_gap: usize = gaps.iter().sum();

        let fragmentation_ratio = if total_memory > 0 {
            total_gap as f64 / (total_memory + total_gap) as f64
        } else {
            0.0
        };

        let largest_free_block = gaps.iter().max().copied().unwrap_or(0);
        let smallest_free_block = gaps.iter().min().copied().unwrap_or(0);
        let free_block_count = gaps.len();
        let total_free_memory = total_gap;

        let external_fragmentation = if !gaps.is_empty() {
            let avg_gap = total_gap as f64 / gaps.len() as f64;
            let max_gap = largest_free_block as f64;
            if max_gap > 0.0 {
                1.0 - (avg_gap / max_gap)
            } else {
                0.0
            }
        } else {
            0.0
        };

        let total_requested: usize = active_allocations.iter().map(|a| a.size).sum();
        let total_allocated = total_requested + total_gap;
        let internal_fragmentation = if total_allocated > 0 {
            (total_gap as f64 / total_allocated as f64) * 100.0
        } else {
            0.0
        };

        FragmentationAnalysis {
            fragmentation_ratio,
            largest_free_block,
            smallest_free_block,
            free_block_count,
            total_free_memory,
            external_fragmentation,
            internal_fragmentation,
        }
    }

    /// Analyze system library usage
    pub fn analyze_system_libraries(&self, allocations: &[AllocationInfo]) -> SystemLibraryStats {
        let mut stats = SystemLibraryStats::default();

        for alloc in allocations {
            let type_name = alloc.type_name.as_deref().unwrap_or("");
            let stack = alloc.stack_trace.as_deref().unwrap_or(&[]);

            if type_name.contains("Vec<")
                || type_name.contains("HashMap")
                || type_name.contains("HashSet")
                || type_name.contains("BTreeMap")
                || type_name.contains("BTreeSet")
            {
                stats.std_collections.allocation_count += 1;
                stats.std_collections.total_bytes += alloc.size;
            }

            if type_name.contains("Future")
                || type_name.contains("async")
                || type_name.contains("tokio")
                || type_name.contains("async_std")
            {
                stats.async_runtime.allocation_count += 1;
                stats.async_runtime.total_bytes += alloc.size;
            }

            if type_name.contains("TcpStream")
                || type_name.contains("UdpSocket")
                || type_name.contains("Http")
                || stack.iter().any(|s| s.contains("net::"))
            {
                stats.network_io.allocation_count += 1;
                stats.network_io.total_bytes += alloc.size;
            }

            if type_name.contains("File")
                || type_name.contains("Path")
                || stack.iter().any(|s| s.contains("fs::"))
            {
                stats.file_system.allocation_count += 1;
                stats.file_system.total_bytes += alloc.size;
            }

            if type_name.contains("serde")
                || type_name.contains("Json")
                || type_name.contains("Deserialize")
            {
                stats.serialization.allocation_count += 1;
                stats.serialization.total_bytes += alloc.size;
            }

            if type_name.contains("Regex") || type_name.contains("regex") {
                stats.regex_engine.allocation_count += 1;
                stats.regex_engine.total_bytes += alloc.size;
            }

            if type_name.contains("Crypto")
                || type_name.contains("Hash")
                || type_name.contains("Signature")
            {
                stats.crypto_security.allocation_count += 1;
                stats.crypto_security.total_bytes += alloc.size;
            }

            if type_name.contains("Database")
                || type_name.contains("Connection")
                || type_name.contains("Query")
            {
                stats.database.allocation_count += 1;
                stats.database.total_bytes += alloc.size;
            }

            if type_name.contains("Window")
                || type_name.contains("Canvas")
                || type_name.contains("Surface")
                || type_name.contains("wgpu")
            {
                stats.graphics_ui.allocation_count += 1;
                stats.graphics_ui.total_bytes += alloc.size;
            }

            if type_name.contains("Request")
                || type_name.contains("Response")
                || type_name.contains("hyper")
                || type_name.contains("reqwest")
            {
                stats.http_stack.allocation_count += 1;
                stats.http_stack.total_bytes += alloc.size;
            }
        }

        stats
    }

    /// Analyze concurrency safety
    pub fn analyze_concurrency_safety(
        &self,
        allocations: &[AllocationInfo],
    ) -> ConcurrencyAnalysis {
        let mut analysis = ConcurrencyAnalysis::default();

        let mut thread_alloc_counts: std::collections::HashMap<std::thread::ThreadId, usize> =
            std::collections::HashMap::new();

        for alloc in allocations {
            let type_name = alloc.type_name.as_deref().unwrap_or("");

            if type_name.contains("Arc<")
                || type_name.contains("Mutex<")
                || type_name.contains("RwLock<")
            {
                analysis.thread_safety_allocations += 1;
                analysis.shared_memory_bytes += alloc.size;
            }

            if type_name.contains("Arc<") {
                analysis.arc_shared += 1;
            }

            if type_name.contains("mpsc")
                || type_name.contains("channel")
                || type_name.contains("Sender")
                || type_name.contains("Receiver")
            {
                analysis.channel_buffers += 1;
            }

            if type_name.contains("thread_local") || type_name.contains("LocalKey") {
                analysis.thread_local_storage += 1;
            }

            if type_name.contains("Atomic") {
                analysis.atomic_operations += 1;
            }

            *thread_alloc_counts.entry(alloc.thread_id).or_insert(0) += 1;
        }

        let max_thread_allocs = thread_alloc_counts.values().max().copied().unwrap_or(0);
        let min_thread_allocs = thread_alloc_counts.values().min().copied().unwrap_or(0);
        let thread_count = thread_alloc_counts.len();

        if thread_count <= 1 {
            analysis.lock_contention_risk = "None".to_string();
        } else {
            let imbalance_ratio = if min_thread_allocs > 0 {
                max_thread_allocs as f64 / min_thread_allocs as f64
            } else {
                f64::MAX
            };

            if imbalance_ratio > 10.0 {
                analysis.lock_contention_risk = "High".to_string();
            } else if imbalance_ratio > 3.0 {
                analysis.lock_contention_risk = "Medium".to_string();
            } else {
                analysis.lock_contention_risk = "Low".to_string();
            }
        }

        analysis
    }

    /// Get unsafe/FFI tracker instance
    pub fn get_unsafe_ffi_tracker(&self) -> Arc<crate::unsafe_ffi_tracker::UnsafeFFITracker> {
        // Delegate to existing global tracker
        crate::unsafe_ffi_tracker::get_global_unsafe_ffi_tracker()
    }

    /// Get unsafe/FFI statistics
    pub fn get_unsafe_ffi_stats(&self) -> crate::unsafe_ffi_tracker::UnsafeFFIStats {
        // Get stats from the global tracker
        self.get_unsafe_ffi_tracker().get_stats()
    }

    /// Analyze circular references in smart pointers
    pub fn analyze_circular_references(
        &self,
        allocations: &[AllocationInfo],
    ) -> crate::circular_reference::CircularReferenceAnalysis {
        crate::circular_reference::detect_circular_references(allocations)
    }

    /// Analyze borrow checker integration and lifetime tracking
    pub fn analyze_borrow_patterns(
        &self,
        _allocations: &[AllocationInfo],
    ) -> BorrowPatternAnalysis {
        let analyzer = get_global_borrow_analyzer();
        analyzer.analyze_borrow_patterns()
    }

    /// Analyze generic type usage and constraints
    pub fn analyze_generic_types(&self, _allocations: &[AllocationInfo]) -> GenericStatistics {
        // Create a new analyzer instance to avoid global state pollution in tests
        let analyzer = GenericAnalyzer::new();
        analyzer.get_generic_statistics()
    }

    /// Analyze async types and Future state machines
    pub fn analyze_async_patterns(&self, _allocations: &[AllocationInfo]) -> AsyncPatternAnalysis {
        let analyzer = get_global_async_analyzer();
        analyzer.analyze_async_patterns()
    }

    /// Analyze closure captures and lifetime relationships
    pub fn analyze_closure_patterns(
        &self,
        allocations: &[AllocationInfo],
    ) -> ClosureAnalysisReport {
        let analyzer = get_global_closure_analyzer();
        analyzer.analyze_closure_patterns(allocations)
    }

    /// Analyze lifecycle patterns including Drop trait and RAII
    pub fn analyze_lifecycle_patterns(
        &self,
        _allocations: &[AllocationInfo],
    ) -> LifecycleAnalysisReport {
        let analyzer = get_global_lifecycle_analyzer();
        analyzer.get_lifecycle_report()
    }

    /// Analyze memory leaks
    pub fn analyze_memory_leaks(
        &self,
        allocations: &[AllocationInfo],
    ) -> crate::analysis::detectors::DetectionResult {
        let detector = crate::analysis::detectors::LeakDetector::new(
            crate::analysis::detectors::LeakDetectorConfig::default(),
        );
        detector.detect(allocations)
    }

    /// Analyze use-after-free issues
    pub fn analyze_use_after_free(
        &self,
        allocations: &[AllocationInfo],
    ) -> crate::analysis::detectors::DetectionResult {
        let detector = crate::analysis::detectors::UafDetector::new(
            crate::analysis::detectors::UafDetectorConfig::default(),
        );
        detector.detect(allocations)
    }

    /// Analyze buffer overflow issues
    pub fn analyze_buffer_overflow(
        &self,
        allocations: &[AllocationInfo],
    ) -> crate::analysis::detectors::DetectionResult {
        let detector = crate::analysis::detectors::OverflowDetector::new(
            crate::analysis::detectors::OverflowDetectorConfig::default(),
        );
        detector.detect(allocations)
    }

    /// Analyze safety violations
    pub fn analyze_safety_violations(
        &self,
        allocations: &[AllocationInfo],
    ) -> crate::analysis::detectors::DetectionResult {
        let detector = crate::analysis::detectors::SafetyDetector::new(
            crate::analysis::detectors::SafetyDetectorConfig::default(),
        );
        detector.detect(allocations)
    }

    /// Analyze lifecycle issues
    pub fn analyze_lifecycle_issues(
        &self,
        allocations: &[AllocationInfo],
    ) -> crate::analysis::detectors::DetectionResult {
        let detector = crate::analysis::detectors::LifecycleDetector::new(
            crate::analysis::detectors::LifecycleDetectorConfig::default(),
        );
        detector.detect(allocations)
    }

    /// Perform comprehensive analysis
    pub fn perform_comprehensive_analysis(
        &self,
        allocations: &[AllocationInfo],
        stats: &MemoryStats,
    ) -> ComprehensiveAnalysisReport {
        let fragmentation = self.analyze_fragmentation(allocations);
        let system_libs = self.analyze_system_libraries(allocations);
        let concurrency = self.analyze_concurrency_safety(allocations);
        let unsafe_stats = self.get_unsafe_ffi_stats();
        let circular_refs = self.analyze_circular_references(allocations);

        // New comprehensive analysis features
        let borrow_analysis = self.analyze_borrow_patterns(allocations);
        let generic_analysis = self.analyze_generic_types(allocations);
        let async_analysis = self.analyze_async_patterns(allocations);
        let closure_analysis = self.analyze_closure_patterns(allocations);
        let lifecycle_analysis = self.analyze_lifecycle_patterns(allocations);

        ComprehensiveAnalysisReport {
            fragmentation_analysis: fragmentation,
            system_library_stats: system_libs,
            concurrency_analysis: concurrency,
            unsafe_ffi_stats: unsafe_stats,
            circular_reference_analysis: circular_refs,
            borrow_analysis,
            generic_analysis,
            async_analysis,
            closure_analysis,
            lifecycle_analysis,
            memory_stats: stats.clone(),
            analysis_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

impl Default for AnalysisManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive analysis report
#[derive(Debug, Clone)]
pub struct ComprehensiveAnalysisReport {
    /// Memory fragmentation analysis results
    pub fragmentation_analysis: FragmentationAnalysis,
    /// System library usage statistics
    pub system_library_stats: SystemLibraryStats,
    /// Concurrency safety analysis
    pub concurrency_analysis: ConcurrencyAnalysis,
    /// Unsafe and FFI operation statistics
    pub unsafe_ffi_stats: crate::unsafe_ffi_tracker::UnsafeFFIStats,
    /// Circular reference analysis for smart pointers
    pub circular_reference_analysis: crate::circular_reference::CircularReferenceAnalysis,
    /// Borrow checker integration and lifetime tracking
    pub borrow_analysis: BorrowPatternAnalysis,
    /// Generic type usage and constraint analysis
    pub generic_analysis: GenericStatistics,
    /// Async type and Future state machine analysis
    pub async_analysis: AsyncPatternAnalysis,
    /// Closure capture and lifetime analysis
    pub closure_analysis: ClosureAnalysisReport,
    /// Lifecycle patterns including Drop trait and RAII
    pub lifecycle_analysis: LifecycleAnalysisReport,
    /// Overall memory statistics
    pub memory_stats: MemoryStats,
    /// Timestamp when analysis was performed
    pub analysis_timestamp: u64,
}

// Re-export all the existing analysis functions for backward compatibility
// This ensures that existing code continues to work without changes

/// Analyze memory fragmentation - backward compatibility function
pub fn analyze_fragmentation(allocations: &[AllocationInfo]) -> FragmentationAnalysis {
    let manager = AnalysisManager::new();
    manager.analyze_fragmentation(allocations)
}

/// Analyze system library usage - backward compatibility function
pub fn analyze_system_libraries(allocations: &[AllocationInfo]) -> SystemLibraryStats {
    let manager = AnalysisManager::new();
    manager.analyze_system_libraries(allocations)
}

/// Analyze concurrency safety - backward compatibility function
pub fn analyze_concurrency_safety(allocations: &[AllocationInfo]) -> ConcurrencyAnalysis {
    let manager = AnalysisManager::new();
    manager.analyze_concurrency_safety(allocations)
}

/// Get global unsafe/FFI tracker - backward compatibility function
pub fn get_global_unsafe_ffi_tracker() -> Arc<crate::unsafe_ffi_tracker::UnsafeFFITracker> {
    crate::unsafe_ffi_tracker::get_global_unsafe_ffi_tracker()
}

/// Get unsafe/FFI statistics - convenience function
pub fn get_unsafe_ffi_stats() -> crate::unsafe_ffi_tracker::UnsafeFFIStats {
    let manager = AnalysisManager::new();
    manager.get_unsafe_ffi_stats()
}

/// Perform comprehensive analysis - convenience function
pub fn perform_comprehensive_analysis(
    allocations: &[AllocationInfo],
    stats: &MemoryStats,
) -> ComprehensiveAnalysisReport {
    let manager = AnalysisManager::new();
    manager.perform_comprehensive_analysis(allocations, stats)
}

// Analysis module - consolidating implementations for better organization
// For now, we're just creating the interface and delegating to the existing implementations

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::types::AllocationInfo;

    #[test]
    fn test_analyze_fragmentation() {
        let manager = AnalysisManager::new();
        // Create allocations with known gaps to verify precise calculation.
        // alloc1 at 0x1000 (size 1024) → ends at 0x1400
        // alloc2 at 0x2000 (size 512) → ends at 0x2200
        // Gap between them: 0x2000 - 0x1400 = 3072 bytes
        let allocations = vec![
            AllocationInfo::new(0x1000, 1024),
            AllocationInfo::new(0x2000, 512),
        ];

        let result = manager.analyze_fragmentation(&allocations);

        // With known layout: total_memory = 1536, total_gap = 3072
        // fragmentation_ratio = 3072 / (1536 + 3072) = 3072/4608 ≈ 0.667
        assert!(
            result.fragmentation_ratio > 0.5,
            "fragmentation_ratio should be > 0.5, got {}",
            result.fragmentation_ratio
        );
        assert_eq!(result.free_block_count, 1, "should have exactly 1 gap");
        assert_eq!(
            result.largest_free_block, 3072,
            "largest gap should be 3072"
        );
        assert_eq!(
            result.smallest_free_block, 3072,
            "smallest gap should be 3072"
        );
    }

    #[test]
    fn test_analyze_fragmentation_no_gaps() {
        let manager = AnalysisManager::new();
        // Contiguous allocations: no gaps
        let allocations = vec![
            AllocationInfo::new(0x1000, 256),
            AllocationInfo::new(0x1100, 256),
        ];

        let result = manager.analyze_fragmentation(&allocations);
        assert_eq!(
            result.fragmentation_ratio, 0.0,
            "no gaps means zero fragmentation"
        );
        assert_eq!(result.free_block_count, 0);
    }

    #[test]
    fn test_analyze_system_libraries() {
        let manager = AnalysisManager::new();

        // Create allocations with specific type names to trigger detection
        let mut alloc_vec = AllocationInfo::new(0x1000, 256);
        alloc_vec.type_name = Some("Vec<String>".to_string());

        let mut alloc_hashmap = AllocationInfo::new(0x2000, 512);
        alloc_hashmap.type_name = Some("HashMap<K, V>".to_string());

        let mut alloc_file = AllocationInfo::new(0x3000, 128);
        alloc_file.type_name = Some("File".to_string());

        let mut alloc_regex = AllocationInfo::new(0x4000, 64);
        alloc_regex.type_name = Some("Regex".to_string());

        let allocations = vec![alloc_vec, alloc_hashmap, alloc_file, alloc_regex];

        let result = manager.analyze_system_libraries(&allocations);

        assert_eq!(
            result.std_collections.allocation_count, 2,
            "Vec and HashMap"
        );
        assert_eq!(result.std_collections.total_bytes, 256 + 512);
        assert_eq!(result.file_system.allocation_count, 1, "File");
        assert_eq!(result.file_system.total_bytes, 128);
        assert_eq!(result.regex_engine.allocation_count, 1, "Regex");
        assert_eq!(result.regex_engine.total_bytes, 64);
        assert_eq!(result.async_runtime.allocation_count, 0, "no async types");
    }

    #[test]
    fn test_analyze_concurrency_safety() {
        let manager = AnalysisManager::new();

        let mut alloc_arc = AllocationInfo::new(0x1000, 64);
        alloc_arc.type_name = Some("Arc<Mutex<i32>>".to_string());

        let mut alloc_mutex = AllocationInfo::new(0x2000, 128);
        alloc_mutex.type_name = Some("Mutex<String>".to_string());

        let mut alloc_atomic = AllocationInfo::new(0x3000, 8);
        alloc_atomic.type_name = Some("AtomicUsize".to_string());

        let allocations = vec![alloc_arc, alloc_mutex, alloc_atomic];

        let result = manager.analyze_concurrency_safety(&allocations);

        assert_eq!(result.thread_safety_allocations, 2, "Arc and Mutex");
        assert_eq!(result.arc_shared, 1, "Arc");
        assert_eq!(result.atomic_operations, 1, "AtomicUsize");
        assert_eq!(result.shared_memory_bytes, 64 + 128);
        assert_eq!(result.lock_contention_risk, "None", "single thread");
    }

    #[test]
    fn test_analyze_concurrency_safety_multi_thread() {
        let manager = AnalysisManager::new();

        // Note: We can't easily create different ThreadId values,
        // so this tests the single-thread case which is "None" risk.
        let mut alloc_arc = AllocationInfo::new(0x1000, 64);
        alloc_arc.type_name = Some("Arc<i32>".to_string());

        let allocations = vec![alloc_arc];

        let result = manager.analyze_concurrency_safety(&allocations);
        assert_eq!(result.arc_shared, 1);
    }

    #[test]
    fn test_get_unsafe_ffi_tracker() {
        let manager = AnalysisManager::new();

        let _tracker = manager.get_unsafe_ffi_tracker();

        // Test that tracker is returned successfully
    }

    #[test]
    fn test_get_unsafe_ffi_stats() {
        let manager = AnalysisManager::new();

        let stats = manager.get_unsafe_ffi_stats();

        // Test that stats are returned with valid default values
        assert_eq!(stats.total_operations, 0);
        assert_eq!(stats.ffi_calls, 0);
        assert_eq!(stats.raw_pointer_operations, 0);
        assert_eq!(stats.memory_violations, 0);
        assert!(stats.operations.is_empty());
    }

    #[test]
    fn test_analyze_circular_references() {
        let manager = AnalysisManager::new();
        // Without smart pointer info, no circular references can be detected.
        // This tests the zero case.
        let allocations = vec![
            AllocationInfo::new(0x4000, 128),
            AllocationInfo::new(0x5000, 256),
        ];

        let result = manager.analyze_circular_references(&allocations);
        assert_eq!(result.total_smart_pointers, 0, "no smart pointer info");
        assert_eq!(result.circular_references.len(), 0);
        assert_eq!(result.pointers_in_cycles, 0);
        assert_eq!(result.total_leaked_memory, 0);
    }

    #[test]
    fn test_analyze_borrow_patterns() {
        let manager = AnalysisManager::new();
        let allocations = vec![AllocationInfo::new(0x7000, 1024)];

        let result = manager.analyze_borrow_patterns(&allocations);

        // Test that borrow pattern analysis returns valid results
        assert!(result.patterns.is_empty());
        assert_eq!(result.total_events, 0);
        assert!(result.analysis_timestamp > 0);
    }

    #[test]
    fn test_analyze_generic_types() {
        let manager = AnalysisManager::new();
        let allocations = vec![AllocationInfo::new(0x8000, 256)];

        let result = manager.analyze_generic_types(&allocations);

        // Test that generic type analysis returns valid results
        assert_eq!(result.total_instances, 0);
        assert_eq!(result.unique_base_types, 0);
        assert_eq!(result.total_instantiations, 0);
        assert_eq!(result.constraint_violations, 0);
        assert!(result.most_used_types.is_empty());
    }

    #[test]
    fn test_analyze_async_patterns() {
        let manager = AnalysisManager::new();
        let allocations = vec![AllocationInfo::new(0x9000, 2048)];

        let result = manager.analyze_async_patterns(&allocations);

        // Test that async pattern analysis returns valid results
        assert!(result.patterns.is_empty());
        assert_eq!(result.total_futures_analyzed, 0);
        assert!(result.analysis_timestamp > 0);
    }

    #[test]
    fn test_analyze_closure_patterns() {
        let manager = AnalysisManager::new();
        let allocations = vec![AllocationInfo::new(0xa000, 128)];

        let result = manager.analyze_closure_patterns(&allocations);

        // Test that closure pattern analysis returns valid results
        assert!(result.detected_closures.is_empty());
        assert_eq!(result.capture_statistics.total_closures, 0);
        assert!(result.optimization_suggestions.is_empty());
        assert!(result.lifetime_analysis.lifetime_patterns.is_empty());
        assert!(result.analysis_timestamp > 0);
    }

    #[test]
    fn test_analyze_lifecycle_patterns() {
        let manager = AnalysisManager::new();
        let allocations = vec![AllocationInfo::new(0xb000, 512)];

        let result = manager.analyze_lifecycle_patterns(&allocations);

        // Test that lifecycle pattern analysis returns valid results
        assert!(result.drop_events.is_empty());
        assert!(result.raii_patterns.is_empty());
        assert!(result.borrow_analysis.borrow_patterns.is_empty());
        assert!(result.closure_captures.is_empty());
        assert!(result.analysis_timestamp > 0);
    }

    #[test]
    fn test_perform_comprehensive_analysis() {
        let manager = AnalysisManager::new();
        let allocations = vec![
            AllocationInfo::new(0x1000, 1024),
            AllocationInfo::new(0x2000, 512),
            AllocationInfo::new(0x3000, 256),
        ];
        let stats = MemoryStats::new();

        let result = manager.perform_comprehensive_analysis(&allocations, &stats);

        assert!(
            result.fragmentation_analysis.fragmentation_ratio >= 0.0
                && result.fragmentation_analysis.fragmentation_ratio <= 1.0
        );
        assert_eq!(
            result.system_library_stats.std_collections.allocation_count,
            0
        );
        assert_eq!(result.concurrency_analysis.thread_safety_allocations, 0);
        assert_eq!(result.unsafe_ffi_stats.total_operations, 0);
        assert_eq!(result.circular_reference_analysis.total_smart_pointers, 0);
        assert!(result.borrow_analysis.patterns.is_empty());
        assert_eq!(result.generic_analysis.total_instances, 0);
        assert!(result.async_analysis.patterns.is_empty());
        assert!(result.closure_analysis.detected_closures.is_empty());
        assert!(result.lifecycle_analysis.drop_events.is_empty());
        assert_eq!(result.memory_stats.total_allocations, 0);
        assert!(result.analysis_timestamp > 0);
    }

    #[test]
    fn test_backward_compatibility_functions() {
        let allocations = vec![AllocationInfo::new(0x1000, 1024)];

        // Test backward compatibility functions
        let frag_result = analyze_fragmentation(&allocations);
        assert_eq!(frag_result.fragmentation_ratio, 0.0);

        let lib_result = analyze_system_libraries(&allocations);
        assert_eq!(lib_result.std_collections.allocation_count, 0);

        let conc_result = analyze_concurrency_safety(&allocations);
        assert_eq!(conc_result.thread_safety_allocations, 0);

        let _tracker = get_global_unsafe_ffi_tracker();

        let stats = get_unsafe_ffi_stats();
        assert_eq!(stats.total_operations, 0);

        let memory_stats = MemoryStats::new();
        let comp_result = perform_comprehensive_analysis(&allocations, &memory_stats);
        assert!(comp_result.analysis_timestamp > 0);
    }

    #[test]
    fn test_empty_allocations_analysis() {
        let manager = AnalysisManager::new();
        let empty_allocations: Vec<AllocationInfo> = vec![];

        // Test that analysis works with empty allocation list
        let frag_result = manager.analyze_fragmentation(&empty_allocations);
        assert_eq!(frag_result.fragmentation_ratio, 0.0);

        let lib_result = manager.analyze_system_libraries(&empty_allocations);
        assert_eq!(lib_result.std_collections.allocation_count, 0);

        let conc_result = manager.analyze_concurrency_safety(&empty_allocations);
        assert_eq!(conc_result.thread_safety_allocations, 0);

        let circ_result = manager.analyze_circular_references(&empty_allocations);
        assert_eq!(circ_result.total_smart_pointers, 0);
    }

    #[test]
    fn test_large_allocation_list_analysis() {
        let manager = AnalysisManager::new();
        let mut allocations = Vec::new();

        // Create a larger list of allocations to test performance
        for i in 0..100 {
            allocations.push(AllocationInfo::new(0x1000 + i * 0x1000, 1024 + i));
        }

        let stats = MemoryStats::new();
        let result = manager.perform_comprehensive_analysis(&allocations, &stats);

        // Test that analysis completes successfully with larger datasets
        assert!(result.analysis_timestamp > 0);
        assert_eq!(result.memory_stats.total_allocations, 0); // Default stats
    }
}
