//! Memory analysis functionality
//!
//! This module provides advanced analysis capabilities:
//! - Enhanced memory analysis
//! - Circular reference detection
//! - Unsafe FFI tracking
//! - Unknown memory region analysis

pub mod circular_reference;
pub mod enhanced_memory_analysis;
pub mod unknown_memory_regions;
pub mod unsafe_ffi_tracker;
pub mod variable_relationships;

// New analysis modules for ComplexTypeForRust.md features
pub mod async_analysis;
pub mod borrow_analysis;
pub mod closure_analysis;
pub mod enhanced_ffi_function_resolver;
pub mod ffi_function_resolver;
pub mod generic_analysis;
pub mod lifecycle_analysis;
pub mod memory_passport_tracker;
pub mod safety_analyzer;
pub mod security_violation_analyzer;

// Re-export key analysis functions
pub use circular_reference::{CircularReference, CircularReferenceAnalysis, CircularReferenceNode};
pub use enhanced_memory_analysis::{analyze_memory_with_enhanced_features, EnhancedMemoryAnalyzer};
pub use unsafe_ffi_tracker::UnsafeFFITracker;
pub use variable_relationships::{
    build_variable_relationship_graph, GraphStatistics, RelationshipType as VarRelationshipType,
    SmartPointerInfo as VarSmartPointerInfo, VariableCategory, VariableCluster, VariableNode,
    VariableRelationship, VariableRelationshipGraph,
};

// Re-export new analysis modules
pub use async_analysis::{
    get_global_async_analyzer, AsyncAnalyzer, AsyncPatternAnalysis, AsyncStatistics,
};
pub use borrow_analysis::{get_global_borrow_analyzer, BorrowAnalyzer, BorrowPatternAnalysis};
pub use closure_analysis::{get_global_closure_analyzer, ClosureAnalysisReport, ClosureAnalyzer};
pub use ffi_function_resolver::{
    get_global_ffi_resolver, initialize_global_ffi_resolver, FfiFunctionCategory,
    FfiFunctionResolver, FfiRiskLevel, ResolutionStats, ResolvedFfiFunction, ResolverConfig,
};
pub use generic_analysis::{get_global_generic_analyzer, GenericAnalyzer, GenericStatistics};
pub use lifecycle_analysis::{
    get_global_lifecycle_analyzer, LifecycleAnalysisReport, LifecycleAnalyzer,
};
pub use memory_passport_tracker::{
    get_global_passport_tracker, initialize_global_passport_tracker, LeakDetail,
    LeakDetectionResult, MemoryPassport, MemoryPassportTracker, PassportEvent, PassportEventType,
    PassportStatus, PassportTrackerConfig, PassportTrackerStats,
};
pub use safety_analyzer::{
    DynamicViolation, RiskAssessment, RiskFactor, RiskFactorType, SafetyAnalysisConfig,
    SafetyAnalysisStats, SafetyAnalyzer, UnsafeReport, UnsafeSource,
};
pub use unsafe_ffi_tracker::ComprehensiveSafetyReport;

use crate::core::types::*;
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
    pub fn analyze_fragmentation(&self, _allocations: &[AllocationInfo]) -> FragmentationAnalysis {
        // Simple implementation for now
        FragmentationAnalysis::default()
    }

    /// Analyze system library usage
    pub fn analyze_system_libraries(&self, _allocations: &[AllocationInfo]) -> SystemLibraryStats {
        // Simple implementation for now
        SystemLibraryStats::default()
    }

    /// Analyze concurrency safety
    pub fn analyze_concurrency_safety(
        &self,
        _allocations: &[AllocationInfo],
    ) -> ConcurrencyAnalysis {
        // Simple implementation for now
        ConcurrencyAnalysis::default()
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

    /// Analyze advanced types (Cell, RefCell, Mutex, etc.)
    pub fn analyze_advanced_types(
        &self,
        allocations: &[AllocationInfo],
    ) -> crate::advanced_types::AdvancedTypeAnalysisReport {
        crate::advanced_types::analyze_advanced_types(allocations)
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
        let analyzer = get_global_generic_analyzer();
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
        let advanced_types = self.analyze_advanced_types(allocations);

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
            advanced_type_analysis: advanced_types,
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
    /// Advanced type analysis (Cell, RefCell, Mutex, etc.)
    pub advanced_type_analysis: crate::advanced_types::AdvancedTypeAnalysisReport,
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
    use crate::core::types::AllocationInfo;

    #[test]
    fn test_analyze_fragmentation() {
        let manager = AnalysisManager::new();
        let allocations = vec![
            AllocationInfo::new(0x1000, 1024),
            AllocationInfo::new(0x2000, 512),
        ];

        let result = manager.analyze_fragmentation(&allocations);

        // Test that fragmentation analysis returns valid default values
        assert_eq!(result.fragmentation_ratio, 0.0);
        assert_eq!(result.largest_free_block, 0);
        assert_eq!(result.free_block_count, 0);
        assert_eq!(result.total_free_memory, 0);
        assert_eq!(result.external_fragmentation, 0.0);
        assert_eq!(result.internal_fragmentation, 0.0);
    }

    #[test]
    fn test_analyze_system_libraries() {
        let manager = AnalysisManager::new();
        let allocations = vec![AllocationInfo::new(0x1000, 256)];

        let result = manager.analyze_system_libraries(&allocations);

        // Test that system library analysis returns valid default values
        assert_eq!(result.std_collections.allocation_count, 0);
        assert_eq!(result.async_runtime.total_bytes, 0);
        assert_eq!(result.network_io.peak_bytes, 0);
        assert_eq!(result.file_system.average_size, 0.0);
        assert!(result.serialization.categories.is_empty());
        assert!(result.regex_engine.hotspot_functions.is_empty());
    }

    #[test]
    fn test_analyze_concurrency_safety() {
        let manager = AnalysisManager::new();
        let allocations = vec![AllocationInfo::new(0x3000, 2048)];

        let result = manager.analyze_concurrency_safety(&allocations);

        // Test that concurrency analysis returns valid default values
        assert_eq!(result.thread_safety_allocations, 0);
        assert_eq!(result.shared_memory_bytes, 0);
        assert_eq!(result.mutex_protected, 0);
        assert_eq!(result.arc_shared, 0);
        assert_eq!(result.rc_shared, 0);
        assert_eq!(result.channel_buffers, 0);
        assert_eq!(result.thread_local_storage, 0);
        assert_eq!(result.atomic_operations, 0);
        assert_eq!(result.lock_contention_risk, "");
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
        let allocations = vec![
            AllocationInfo::new(0x4000, 128),
            AllocationInfo::new(0x5000, 256),
        ];

        let result = manager.analyze_circular_references(&allocations);

        // Test that circular reference analysis returns valid results
        assert_eq!(result.total_smart_pointers, 0);
        assert_eq!(result.circular_references.len(), 0);
        assert_eq!(result.pointers_in_cycles, 0);
        assert_eq!(result.total_leaked_memory, 0);
    }

    #[test]
    fn test_analyze_advanced_types() {
        let manager = AnalysisManager::new();
        let allocations = vec![AllocationInfo::new(0x6000, 512)];

        let result = manager.analyze_advanced_types(&allocations);

        // Test that advanced type analysis returns valid results
        assert!(result.by_category.is_empty());
        assert!(result.all_issues.is_empty());
        // Performance summary may have default values, so just check it's not negative
        assert!(result.performance_summary.total_overhead_factor >= 0.0);
        assert_eq!(result.statistics.total_advanced_types, 0);
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

        // Test that comprehensive analysis includes all components
        assert_eq!(result.fragmentation_analysis.fragmentation_ratio, 0.0);
        assert_eq!(
            result.system_library_stats.std_collections.allocation_count,
            0
        );
        assert_eq!(result.concurrency_analysis.thread_safety_allocations, 0);
        assert_eq!(result.unsafe_ffi_stats.total_operations, 0);
        assert_eq!(result.circular_reference_analysis.total_smart_pointers, 0);
        assert!(result.advanced_type_analysis.by_category.is_empty());
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

        let adv_result = manager.analyze_advanced_types(&empty_allocations);
        assert!(adv_result.by_category.is_empty());
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
