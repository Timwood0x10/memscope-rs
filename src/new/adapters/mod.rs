//! Adapters for integrating old modules with new architecture
//!
//! This module provides adapter implementations that wrap old modules
//! and integrate them with the new unified architecture while maintaining
//! backward compatibility.

use crate::new::tracker::UnifiedTracker;
use crate::new::types::Snapshot;
use std::sync::Arc;

// Temporary type for compatibility
#[derive(Debug, Clone, Default)]
struct LifecycleStats {
    // Placeholder for lifecycle statistics
}

impl From<LifecycleStats> for crate::core::types::ScopeLifecycleMetrics {
    fn from(_: LifecycleStats) -> Self {
        crate::core::types::ScopeLifecycleMetrics::default()
    }
}

// ============================================================================
// Memory Tracker Adapter
// ============================================================================

/// Adapter for MemoryTracker that integrates with UnifiedTracker
pub struct MemoryTrackerAdapter {
    /// Reference to the old MemoryTracker
    #[allow(dead_code)]
    old_tracker: Arc<crate::core::tracker::memory_tracker::MemoryTracker>,
    /// Reference to the new UnifiedTracker
    unified: Arc<UnifiedTracker>,
}

impl MemoryTrackerAdapter {
    /// Create a new MemoryTracker adapter
    pub fn new(
        old_tracker: Arc<crate::core::tracker::memory_tracker::MemoryTracker>,
        unified: Arc<UnifiedTracker>,
    ) -> Self {
        Self {
            old_tracker,
            unified,
        }
    }

    /// Get statistics from unified tracker
    pub fn get_stats(
        &self,
    ) -> Result<crate::core::types::MemoryStats, crate::core::types::TrackingError> {
        let snapshot = self.unified.snapshot();
        Ok(crate::core::types::MemoryStats {
            total_allocations: snapshot.stats.total_allocations,
            total_allocated: snapshot.stats.total_size,
            active_allocations: snapshot.stats.active_allocations,
            active_memory: snapshot.stats.total_size,
            peak_allocations: snapshot.stats.total_allocations,
            peak_memory: snapshot.stats.total_size,
            total_deallocations: 0,
            total_deallocated: 0,
            leaked_allocations: snapshot.stats.leaked_allocations,
            leaked_memory: 0,
            fragmentation_analysis: crate::core::types::FragmentationAnalysis::default(),
            lifecycle_stats: crate::core::types::ScopeLifecycleMetrics::from(
                LifecycleStats::default(),
            ), // Will be implemented
            system_library_stats: crate::core::types::SystemLibraryStats::default(),
            concurrency_analysis: crate::core::types::ConcurrencyAnalysis::default(),
            allocations: Vec::new(), // Simplified for now
        })
    }
}

// ============================================================================
// Export Adapter
// ============================================================================

/// Adapter for integrating old export modules with new export system
pub struct ExportAdapter {
    /// Old exporters
    #[allow(dead_code)]
    old_exporters: Vec<Box<dyn OldExporter>>,
    /// New unified exporter
    unified: Arc<crate::new::export::CompositeExporter>,
}

impl ExportAdapter {
    /// Create a new export adapter
    pub fn new(unified: Arc<crate::new::export::CompositeExporter>) -> Self {
        Self {
            old_exporters: Vec::new(),
            unified,
        }
    }
}

/// Trait for old exporters
pub trait OldExporter {
    fn export(&self, data: &Snapshot) -> Result<Vec<u8>, String>;
}

// ============================================================================
// Analysis Adapter
// ============================================================================

/// Adapter for integrating old analyzers with new analysis system
pub struct AnalysisAdapter {
    /// Old analyzers
    #[allow(dead_code)]
    old_analyzers: Vec<Box<dyn OldAnalyzer>>,
    /// New unified analyzer
    unified: Arc<crate::new::analysis::CompositeAnalyzer>,
}

impl AnalysisAdapter {
    /// Create a new analysis adapter
    pub fn new(unified: Arc<crate::new::analysis::CompositeAnalyzer>) -> Self {
        Self {
            old_analyzers: Vec::new(),
            unified,
        }
    }
}

/// Trait for old analyzers
pub trait OldAnalyzer {
    fn analyze(&self, data: &Snapshot) -> Result<String, String>;
}
