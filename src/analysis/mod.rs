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
pub mod generic_analysis;
pub mod lifecycle_analysis;
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
pub use generic_analysis::{get_global_generic_analyzer, GenericAnalyzer, GenericStatistics};
pub use lifecycle_analysis::{
    get_global_lifecycle_analyzer, LifecycleAnalysisReport, LifecycleAnalyzer,
};

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

// TODO: Gradually move the actual implementation from the individual files to this module
// For now, we're just creating the interface and delegating to the existing implementations
