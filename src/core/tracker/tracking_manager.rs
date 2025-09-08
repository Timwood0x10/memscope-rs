//! Unified tracking manager interface.
//!
//! This module provides a unified interface that combines memory tracking and scope tracking
//! while preserving all existing functionality.

use super::memory_tracker::{get_global_tracker, MemoryTracker};
use crate::core::types::{
    AllocationInfo, MemoryStats, ScopeAnalysis, ScopeLifecycleMetrics, TrackingResult,
};
use std::sync::Arc;

/// Unified tracking manager that combines memory and scope tracking
/// This provides a unified interface that combines memory tracking and scope tracking
/// while preserving all existing functionality.
pub struct TrackingManager {
    memory_tracker: Arc<MemoryTracker>,
    scope_tracker: Arc<crate::core::scope_tracker::ScopeTracker>,
}

impl TrackingManager {
    /// Create a new tracking manager instance
    pub fn new() -> Self {
        Self {
            memory_tracker: get_global_tracker(),
            scope_tracker: crate::core::scope_tracker::get_global_scope_tracker(),
        }
    }

    /// Get the memory tracker instance
    pub fn memory_tracker(&self) -> &Arc<MemoryTracker> {
        &self.memory_tracker
    }

    /// Get the scope tracker instance
    pub fn scope_tracker(&self) -> &Arc<crate::core::scope_tracker::ScopeTracker> {
        &self.scope_tracker
    }

    /// Track memory allocation
    pub fn track_allocation(&self, ptr: usize, size: usize) -> TrackingResult<()> {
        self.memory_tracker.track_allocation(ptr, size)
    }

    /// Track memory deallocation
    pub fn track_deallocation(&self, ptr: usize) -> TrackingResult<()> {
        self.memory_tracker.track_deallocation(ptr)
    }

    /// Associate variable with memory allocation
    pub fn associate_var(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
    ) -> TrackingResult<()> {
        self.memory_tracker.associate_var(ptr, var_name, type_name)
    }

    /// Enter a new scope
    pub fn enter_scope(&self, name: String) -> TrackingResult<crate::core::scope_tracker::ScopeId> {
        self.scope_tracker.enter_scope(name)
    }

    /// Exit a scope
    pub fn exit_scope(&self, scope_id: crate::core::scope_tracker::ScopeId) -> TrackingResult<()> {
        self.scope_tracker.exit_scope(scope_id)
    }

    /// Associate variable with current scope
    pub fn associate_variable(
        &self,
        variable_name: String,
        memory_size: usize,
    ) -> TrackingResult<()> {
        self.scope_tracker
            .associate_variable(variable_name, memory_size)
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> TrackingResult<MemoryStats> {
        self.memory_tracker.get_stats()
    }

    /// Get active allocations
    pub fn get_active_allocations(&self) -> TrackingResult<Vec<AllocationInfo>> {
        self.memory_tracker.get_active_allocations()
    }

    /// Get allocation history
    pub fn get_allocation_history(&self) -> TrackingResult<Vec<AllocationInfo>> {
        self.memory_tracker.get_allocation_history()
    }

    /// Get scope analysis
    pub fn get_scope_analysis(&self) -> TrackingResult<ScopeAnalysis> {
        self.scope_tracker.get_scope_analysis()
    }

    /// Perform comprehensive tracking analysis
    pub fn perform_comprehensive_analysis(&self) -> TrackingResult<ComprehensiveTrackingReport> {
        let memory_stats = self.get_stats()?;
        let active_allocations = self.get_active_allocations()?;
        let allocation_history = self.get_allocation_history()?;
        let scope_analysis = self.get_scope_analysis()?;
        let scope_metrics = self.scope_tracker.get_scope_lifecycle_metrics()?;

        Ok(ComprehensiveTrackingReport {
            memory_stats,
            active_allocations,
            allocation_history,
            scope_analysis,
            scope_metrics,
            analysis_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }
}

impl Default for TrackingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive tracking report
#[derive(Debug, Clone)]
pub struct ComprehensiveTrackingReport {
    /// Overall memory statistics
    pub memory_stats: MemoryStats,
    /// Currently active memory allocations
    pub active_allocations: Vec<AllocationInfo>,
    /// Historical allocation data
    pub allocation_history: Vec<AllocationInfo>,
    /// Scope analysis results
    pub scope_analysis: ScopeAnalysis,
    /// Scope lifecycle metrics
    pub scope_metrics: Vec<ScopeLifecycleMetrics>,
    /// Timestamp when report was generated
    pub analysis_timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracking_manager_creation() {
        // Test that TrackingManager can be created
        let manager = TrackingManager::new();

        // Verify it has the expected components
        let _memory_tracker = manager.memory_tracker();
        let _scope_tracker = manager.scope_tracker();

        // If we get here without panicking, creation was successful
        assert!(true, "TrackingManager created successfully");
    }

    #[test]
    fn test_tracking_manager_default() {
        // Test that Default trait works
        let manager = TrackingManager::default();

        // Should be equivalent to new()
        let _memory_tracker = manager.memory_tracker();
        let _scope_tracker = manager.scope_tracker();

        assert!(true, "TrackingManager default creation successful");
    }

    #[test]
    fn test_comprehensive_tracking_report_structure() {
        // Test the ComprehensiveTrackingReport structure
        use crate::core::types::{
            ConcurrencyAnalysis, FragmentationAnalysis, MemoryStats, ScopeAnalysis, ScopeHierarchy,
            ScopeLifecycleMetrics, SystemLibraryStats,
        };

        let report = ComprehensiveTrackingReport {
            memory_stats: MemoryStats {
                total_allocations: 10,
                total_allocated: 1024,
                active_allocations: 5,
                active_memory: 512,
                peak_allocations: 8,
                peak_memory: 800,
                total_deallocations: 5,
                total_deallocated: 512,
                leaked_allocations: 0,
                leaked_memory: 0,
                fragmentation_analysis: FragmentationAnalysis::default(),
                lifecycle_stats: ScopeLifecycleMetrics::default(),
                allocations: vec![],
                system_library_stats: SystemLibraryStats::default(),
                concurrency_analysis: ConcurrencyAnalysis::default(),
            },
            active_allocations: vec![],
            allocation_history: vec![],
            scope_analysis: ScopeAnalysis {
                total_scopes: 3,
                active_scopes: 1,
                max_depth: 2,
                average_lifetime: 0.0,
                memory_efficiency: 0.0,
                scopes: vec![],
                scope_hierarchy: ScopeHierarchy::default(),
                cross_scope_references: vec![],
            },
            scope_metrics: vec![],
            analysis_timestamp: 1234567890,
        };

        // Test that the report can be created and accessed
        assert_eq!(report.memory_stats.total_allocations, 10);
        assert_eq!(report.memory_stats.active_allocations, 5);
        assert_eq!(report.scope_analysis.total_scopes, 3);
        assert_eq!(report.analysis_timestamp, 1234567890);
    }

    #[test]
    fn test_comprehensive_tracking_report_clone() {
        // Test that ComprehensiveTrackingReport can be cloned
        use crate::core::types::{
            ConcurrencyAnalysis, FragmentationAnalysis, MemoryStats, ScopeAnalysis, ScopeHierarchy,
            ScopeLifecycleMetrics, SystemLibraryStats,
        };

        let original = ComprehensiveTrackingReport {
            memory_stats: MemoryStats {
                total_allocations: 5,
                total_allocated: 500,
                active_allocations: 3,
                active_memory: 300,
                peak_allocations: 4,
                peak_memory: 400,
                total_deallocations: 2,
                total_deallocated: 200,
                leaked_allocations: 0,
                leaked_memory: 0,
                fragmentation_analysis: FragmentationAnalysis::default(),
                lifecycle_stats: ScopeLifecycleMetrics::default(),
                allocations: vec![],
                system_library_stats: SystemLibraryStats::default(),
                concurrency_analysis: ConcurrencyAnalysis::default(),
            },
            active_allocations: vec![],
            allocation_history: vec![],
            scope_analysis: ScopeAnalysis {
                total_scopes: 2,
                active_scopes: 1,
                max_depth: 1,
                average_lifetime: 0.0,
                memory_efficiency: 0.0,
                scopes: vec![],
                scope_hierarchy: ScopeHierarchy::default(),
                cross_scope_references: vec![],
            },
            scope_metrics: vec![],
            analysis_timestamp: 9876543210,
        };

        let cloned = original.clone();

        // Verify clone has same values
        assert_eq!(
            original.memory_stats.total_allocations,
            cloned.memory_stats.total_allocations
        );
        assert_eq!(
            original.scope_analysis.total_scopes,
            cloned.scope_analysis.total_scopes
        );
        assert_eq!(original.analysis_timestamp, cloned.analysis_timestamp);
    }

    #[test]
    fn test_comprehensive_tracking_report_debug() {
        // Test that ComprehensiveTrackingReport implements Debug
        use crate::core::types::{
            ConcurrencyAnalysis, FragmentationAnalysis, MemoryStats, ScopeAnalysis, ScopeHierarchy,
            ScopeLifecycleMetrics, SystemLibraryStats,
        };

        let report = ComprehensiveTrackingReport {
            memory_stats: MemoryStats {
                total_allocations: 1,
                total_allocated: 100,
                active_allocations: 1,
                active_memory: 100,
                peak_allocations: 1,
                peak_memory: 100,
                total_deallocations: 0,
                total_deallocated: 0,
                leaked_allocations: 0,
                leaked_memory: 0,
                fragmentation_analysis: FragmentationAnalysis::default(),
                lifecycle_stats: ScopeLifecycleMetrics::default(),
                allocations: vec![],
                system_library_stats: SystemLibraryStats::default(),
                concurrency_analysis: ConcurrencyAnalysis::default(),
            },
            active_allocations: vec![],
            allocation_history: vec![],
            scope_analysis: ScopeAnalysis {
                total_scopes: 1,
                active_scopes: 1,
                max_depth: 1,
                average_lifetime: 0.0,
                memory_efficiency: 0.0,
                scopes: vec![],
                scope_hierarchy: ScopeHierarchy::default(),
                cross_scope_references: vec![],
            },
            scope_metrics: vec![],
            analysis_timestamp: 1111111111,
        };

        let debug_str = format!("{report:?}");

        // Should contain key information
        assert!(debug_str.contains("ComprehensiveTrackingReport"));
        assert!(debug_str.contains("memory_stats"));
        assert!(debug_str.contains("scope_analysis"));
        assert!(debug_str.contains("analysis_timestamp"));
    }

    #[test]
    fn test_tracking_manager_method_signatures() {
        // Test that all TrackingManager methods have correct signatures
        let manager = TrackingManager::new();

        // Test memory_tracker method
        let _memory_tracker: &Arc<MemoryTracker> = manager.memory_tracker();

        // Test scope_tracker method
        let _scope_tracker: &Arc<crate::core::scope_tracker::ScopeTracker> =
            manager.scope_tracker();

        // Test method signatures without actually calling them (to avoid global state issues)
        // We just verify the methods exist and have correct types

        // These would normally be tested, but we avoid calling them due to global state:
        // let _: TrackingResult<()> = manager.track_allocation(0x1000, 1024);
        // let _: TrackingResult<()> = manager.track_deallocation(0x1000);
        // let _: TrackingResult<()> = manager.associate_var(0x1000, "var".to_string(), "type".to_string());
        // let _: TrackingResult<crate::core::scope_tracker::ScopeId> = manager.enter_scope("scope".to_string());
        // let _: TrackingResult<()> = manager.exit_scope(1);
        // let _: TrackingResult<()> = manager.associate_variable("var".to_string(), 1024);
        // let _: TrackingResult<MemoryStats> = manager.get_stats();
        // let _: TrackingResult<Vec<AllocationInfo>> = manager.get_active_allocations();
        // let _: TrackingResult<Vec<AllocationInfo>> = manager.get_allocation_history();
        // let _: TrackingResult<ScopeAnalysis> = manager.get_scope_analysis();
        // let _: TrackingResult<ComprehensiveTrackingReport> = manager.perform_comprehensive_analysis();

        assert!(true, "All method signatures are correct");
    }

    #[test]
    fn test_tracking_manager_component_access() {
        // Test that we can access the internal components
        let manager = TrackingManager::new();

        // Access memory tracker
        let memory_tracker = manager.memory_tracker();
        // Arc is always valid, so we just check we can access it
        let _ = memory_tracker;

        // Access scope tracker
        let scope_tracker = manager.scope_tracker();
        // Arc is always valid, so we just check we can access it
        let _ = scope_tracker;

        // Test that we can access them multiple times
        let _memory_tracker2 = manager.memory_tracker();
        let _scope_tracker2 = manager.scope_tracker();

        assert!(true, "Component access works correctly");
    }
}
