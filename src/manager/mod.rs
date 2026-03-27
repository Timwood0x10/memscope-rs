//! Unified tracking manager
//!
//! This module provides the TrackingManager which serves as the central
//! coordination point for all tracking operations, supporting strategy
//! switching and data export.

use std::sync::{Arc, RwLock};
use std::path::Path;

use crate::data::{TrackingSnapshot, TrackingStrategy, ExportFormat};
use crate::tracker::base::TrackBase;
use crate::tracker::strategies::{CoreTracker, LockfreeTracker, AsyncTracker, UnifiedTracker};
use crate::render::renderer::Renderer;
use crate::render::{JsonRenderer, BinaryRenderer, HtmlRenderer};
use crate::error::types::{ErrorKind, ErrorSeverity, MemScopeError};

/// Unified tracking manager
///
/// Provides the central coordination point for all tracking operations.
/// Supports dynamic strategy switching and unified data export.
pub struct TrackingManager {
    tracker: Arc<RwLock<dyn TrackBase>>,
}

impl TrackingManager {
    /// Create a new TrackingManager with the specified strategy
    ///
    /// # Arguments
    /// * `strategy` - The tracking strategy to use
    ///
    /// # Returns
    /// A new TrackingManager instance
    pub fn new(strategy: TrackingStrategy) -> Self {
        let tracker: Arc<RwLock<dyn TrackBase>> = match strategy {
            TrackingStrategy::Core => {
                Arc::new(RwLock::new(CoreTracker::new()))
            }
            TrackingStrategy::Lockfree => {
                Arc::new(RwLock::new(LockfreeTracker::new()))
            }
            TrackingStrategy::Async => {
                Arc::new(RwLock::new(AsyncTracker::new()))
            }
            TrackingStrategy::Unified => {
                Arc::new(RwLock::new(UnifiedTracker::new_hybrid()))
            }
        };

        TrackingManager { tracker }
    }

    /// Create a new TrackingManager with Core strategy (default)
    pub fn new_core() -> Self {
        Self::new(TrackingStrategy::Core)
    }

    /// Create a new TrackingManager with Lockfree strategy
    pub fn new_lockfree() -> Self {
        Self::new(TrackingStrategy::Lockfree)
    }

    /// Create a new TrackingManager with Async strategy
    pub fn new_async() -> Self {
        Self::new(TrackingStrategy::Async)
    }

    /// Create a new TrackingManager with Unified strategy
    pub fn new_unified() -> Self {
        Self::new(TrackingStrategy::Unified)
    }

    /// Get current tracking strategy
    pub fn strategy(&self) -> TrackingStrategy {
        let tracker = self.tracker.read().unwrap();
        tracker.strategy()
    }

    /// Track an allocation
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the allocated memory
    /// * `size` - Size of the allocation in bytes
    pub fn track_alloc(&self, ptr: usize, size: usize) {
        let tracker = self.tracker.read().unwrap();
        tracker.track_alloc(ptr, size);
    }

    /// Track a deallocation
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the deallocated memory
    pub fn track_dealloc(&self, ptr: usize) {
        let tracker = self.tracker.read().unwrap();
        tracker.track_dealloc(ptr);
    }

    /// Get current tracking snapshot
    ///
    /// # Returns
    /// A TrackingSnapshot containing all current data
    pub fn snapshot(&self) -> TrackingSnapshot {
        let tracker = self.tracker.read().unwrap();
        tracker.snapshot()
    }

    /// Clear all tracked data
    pub fn clear(&self) {
        let tracker = self.tracker.read().unwrap();
        tracker.clear();
    }

    /// Enable/disable tracking
    pub fn set_enabled(&self, enabled: bool) {
        let tracker = self.tracker.read().unwrap();
        tracker.set_enabled(enabled);
    }

    /// Check if tracking is enabled
    pub fn is_enabled(&self) -> bool {
        let tracker = self.tracker.read().unwrap();
        tracker.is_enabled()
    }

    /// Export tracking data to the specified format
    ///
    /// # Arguments
    /// * `format` - The export format to use
    ///
    /// # Returns
    /// A RenderOutput containing the exported data
    ///
    /// # Errors
    /// Returns an error if the export fails
    pub fn export(&self, format: ExportFormat) -> Result<crate::data::RenderOutput, MemScopeError> {
        let snapshot = self.snapshot();
        let renderer: Box<dyn Renderer> = match format {
            ExportFormat::Json => Box::new(JsonRenderer),
            ExportFormat::Binary => Box::new(BinaryRenderer),
            ExportFormat::Html => Box::new(HtmlRenderer),
        };

        renderer.render(&snapshot).map_err(|e| {
            MemScopeError::new(
                ErrorKind::ExportError,
                ErrorSeverity::Medium,
                format!("Export failed: {}", e)
            )
        })
    }

    /// Export tracking data to a file
    ///
    /// # Arguments
    /// * `path` - Path to the output file
    /// * `format` - The export format to use
    ///
    /// # Errors
    /// Returns an error if the export or file write fails
    pub fn export_to_file<P: AsRef<Path>>(
        &self,
        path: P,
        format: ExportFormat
    ) -> Result<(), MemScopeError> {
        let output = self.export(format)?;

        std::fs::write(path, output.data).map_err(|e| {
            MemScopeError::new(
                ErrorKind::ExportError,
                ErrorSeverity::Medium,
                format!("Failed to write to file: {}", e)
            )
        })?;

        Ok(())
    }

    /// Switch to a different tracking strategy
    ///
    /// # Arguments
    /// * `strategy` - The new tracking strategy to use
    ///
    /// # Note
    /// This will clear all existing tracking data
    pub fn switch_strategy(&self, strategy: TrackingStrategy) {
        let new_tracker: Arc<RwLock<dyn TrackBase>> = match strategy {
            TrackingStrategy::Core => {
                Arc::new(RwLock::new(CoreTracker::new()))
            }
            TrackingStrategy::Lockfree => {
                Arc::new(RwLock::new(LockfreeTracker::new()))
            }
            TrackingStrategy::Async => {
                Arc::new(RwLock::new(AsyncTracker::new()))
            }
            TrackingStrategy::Unified => {
                Arc::new(RwLock::new(UnifiedTracker::new_hybrid()))
            }
        };

        let mut tracker = self.tracker.write().unwrap();
        *tracker = new_tracker;
    }

    /// Get access to the underlying tracker for advanced operations
    ///
    /// # Type Parameters
    /// * `T` - The type of the underlying tracker
    ///
    /// # Returns
    /// Some reference to the underlying tracker if type matches, None otherwise
    ///
    /// # Safety
    /// This function is unsafe because it performs type casting
    #[allow(dead_code)]
    pub(crate) unsafe fn downcast_tracker<T: TrackBase + 'static>(&self) -> Option<&T> {
        // This is a simplified implementation
        // In a real implementation, we'd use Any or similar mechanism
        None
    }
}

impl Default for TrackingManager {
    fn default() -> Self {
        Self::new_core()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_tracking_manager_creation() {
        let manager = TrackingManager::new_core();
        assert_eq!(manager.strategy(), TrackingStrategy::Core);
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_tracking_manager_strategies() {
        let manager = TrackingManager::new(TrackingStrategy::Lockfree);
        assert_eq!(manager.strategy(), TrackingStrategy::Lockfree);

        manager.switch_strategy(TrackingStrategy::Async);
        assert_eq!(manager.strategy(), TrackingStrategy::Async);
    }

    #[test]
    fn test_tracking_manager_alloc_dealloc() {
        let manager = TrackingManager::new_core();
        manager.track_alloc(0x1000, 1024);
        manager.track_alloc(0x2000, 2048);
        manager.track_dealloc(0x1000);

        let snapshot = manager.snapshot();
        assert_eq!(snapshot.allocations.len(), 1);
        assert_eq!(snapshot.stats.allocation_count, 2);
    }

    #[test]
    fn test_tracking_manager_clear() {
        let manager = TrackingManager::new_core();
        manager.track_alloc(0x1000, 1024);
        manager.clear();

        let snapshot = manager.snapshot();
        assert_eq!(snapshot.allocations.len(), 0);
    }

    #[test]
    fn test_tracking_manager_export_json() {
        let manager = TrackingManager::new_core();
        manager.track_alloc(0x1000, 1024);

        let result = manager.export(ExportFormat::Json);
        assert!(result.is_ok());
        assert!(result.unwrap().data.contains("allocations"));
    }

    #[test]
    fn test_tracking_manager_export_html() {
        let manager = TrackingManager::new_core();
        manager.track_alloc(0x1000, 1024);

        let result = manager.export(ExportFormat::Html);
        assert!(result.is_ok());
        let data = result.unwrap().data;
        assert!(data.contains("<html>") || data.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_tracking_manager_export_to_file() {
        let manager = TrackingManager::new_core();
        manager.track_alloc(0x1000, 1024);

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.json");

        let result = manager.export_to_file(&file_path, ExportFormat::Json);
        assert!(result.is_ok());
        assert!(file_path.exists());
    }

    #[test]
    fn test_tracking_manager_enable_disable() {
        let manager = TrackingManager::new_core();
        manager.set_enabled(false);
        manager.track_alloc(0x1000, 1024);

        let snapshot = manager.snapshot();
        assert_eq!(snapshot.allocations.len(), 0);

        manager.set_enabled(true);
        manager.track_alloc(0x2000, 2048);

        let snapshot = manager.snapshot();
        assert_eq!(snapshot.allocations.len(), 1);
    }

    #[test]
    fn test_tracking_manager_all_strategies() {
        // Test Core strategy
        let manager = TrackingManager::new_core();
        manager.track_alloc(0x1000, 1024);
        assert!(manager.snapshot().allocations.len() > 0);

        // Test Lockfree strategy
        let manager = TrackingManager::new_lockfree();
        manager.track_alloc(0x2000, 2048);
        assert!(manager.snapshot().stats.allocation_count > 0);

        // Test Async strategy
        let manager = TrackingManager::new_async();
        manager.track_alloc(0x3000, 4096);
        assert!(manager.snapshot().allocations.len() > 0);

        // Test Unified strategy
        let manager = TrackingManager::new_unified();
        manager.track_alloc(0x4000, 8192);
        assert!(manager.snapshot().allocations.len() > 0);
    }
}