// Unified Tracking module - Consolidated interface for tracker.rs and scope_tracker.rs
// This module provides a unified interface while preserving all existing functionality

use crate::types::*;
use std::sync::Arc;

/// Main tracking interface - consolidates all tracking functionality
pub struct TrackingManager {
    memory_tracker: Arc<crate::tracker::MemoryTracker>,
    scope_tracker: Arc<crate::scope_tracker::ScopeTracker>,
}

impl TrackingManager {
    pub fn new() -> Self {
        Self {
            memory_tracker: crate::tracker::get_global_tracker(),
            scope_tracker: crate::scope_tracker::get_global_scope_tracker(),
        }
    }
    
    /// Get the memory tracker instance
    pub fn memory_tracker(&self) -> &Arc<crate::tracker::MemoryTracker> {
        &self.memory_tracker
    }
    
    /// Get the scope tracker instance
    pub fn scope_tracker(&self) -> &Arc<crate::scope_tracker::ScopeTracker> {
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
    pub fn associate_var(&self, ptr: usize, var_name: String, type_name: String) -> TrackingResult<()> {
        self.memory_tracker.associate_var(ptr, var_name, type_name)
    }
    
    /// Enter a new scope
    pub fn enter_scope(&self, name: String) -> TrackingResult<crate::scope_tracker::ScopeId> {
        self.scope_tracker.enter_scope(name)
    }
    
    /// Exit a scope
    pub fn exit_scope(&self, scope_id: crate::scope_tracker::ScopeId) -> TrackingResult<()> {
        self.scope_tracker.exit_scope(scope_id)
    }
    
    /// Associate variable with current scope
    pub fn associate_variable(&self, variable_name: String, memory_size: usize) -> TrackingResult<()> {
        self.scope_tracker.associate_variable(variable_name, memory_size)
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
    
    /// Export to JSON (standard format)
    pub fn export_to_json<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        self.memory_tracker.export_to_json(path)
    }
    
    /// Export enhanced JSON (with complete data including scope information)
    pub fn export_enhanced_json<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        self.memory_tracker.export_enhanced_json(path)
    }
    
    /// Export memory analysis SVG
    pub fn export_memory_analysis<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        self.memory_tracker.export_memory_analysis(path)
    }
    
    /// Export lifecycle timeline SVG
    pub fn export_lifecycle_timeline<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        self.memory_tracker.export_lifecycle_timeline(path)
    }
    
    /// Export interactive dashboard
    pub fn export_interactive_dashboard<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        self.memory_tracker.export_interactive_dashboard(path)
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
    pub memory_stats: MemoryStats,
    pub active_allocations: Vec<AllocationInfo>,
    pub allocation_history: Vec<AllocationInfo>,
    pub scope_analysis: ScopeAnalysis,
    pub scope_metrics: Vec<ScopeLifecycleMetrics>,
    pub analysis_timestamp: u64,
}

// Re-export all the existing tracking functions for backward compatibility
// This ensures that existing code continues to work without changes

/// Get global memory tracker - backward compatibility function
pub fn get_global_tracker() -> Arc<crate::tracker::MemoryTracker> {
    crate::tracker::get_global_tracker()
}

/// Get global scope tracker - backward compatibility function
pub fn get_global_scope_tracker() -> Arc<crate::scope_tracker::ScopeTracker> {
    crate::scope_tracker::get_global_scope_tracker()
}

/// Get unified tracking manager - convenience function
pub fn get_tracking_manager() -> TrackingManager {
    TrackingManager::new()
}

/// Track allocation - convenience function
pub fn track_allocation(ptr: usize, size: usize) -> TrackingResult<()> {
    let manager = TrackingManager::new();
    manager.track_allocation(ptr, size)
}

/// Track deallocation - convenience function
pub fn track_deallocation(ptr: usize) -> TrackingResult<()> {
    let manager = TrackingManager::new();
    manager.track_deallocation(ptr)
}

/// Associate variable - convenience function
pub fn associate_var(ptr: usize, var_name: String, type_name: String) -> TrackingResult<()> {
    let manager = TrackingManager::new();
    manager.associate_var(ptr, var_name, type_name)
}

/// Enter scope - convenience function
pub fn enter_scope(name: String) -> TrackingResult<crate::scope_tracker::ScopeId> {
    let manager = TrackingManager::new();
    manager.enter_scope(name)
}

/// Exit scope - convenience function
pub fn exit_scope(scope_id: crate::scope_tracker::ScopeId) -> TrackingResult<()> {
    let manager = TrackingManager::new();
    manager.exit_scope(scope_id)
}

// TODO: Gradually move the actual implementation from the individual files to this module
// For now, we're just creating the interface and delegating to the existing implementations