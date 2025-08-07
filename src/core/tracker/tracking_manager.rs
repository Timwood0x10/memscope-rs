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
