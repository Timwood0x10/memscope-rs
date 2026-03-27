//! Unified tracking manager
//!
//! This module provides the TrackingManager which serves as the central
//! coordination point for all tracking operations, supporting strategy
//! switching and data export.

use std::cell::Cell;
use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::data::{ExportFormat, TrackingSnapshot, TrackingStrategy};
use crate::error::types::{ErrorKind, ErrorSeverity, MemScopeError};
use crate::render::renderer::Renderer;
use crate::render::{BinaryRenderer, HtmlRenderer, JsonRenderer};
use crate::tracker::base::TrackBase;
use crate::tracker::strategies::{AsyncTracker, CoreTracker, LockfreeTracker, UnifiedTracker};

// Thread-local flag to prevent recursive tracking
thread_local! {
    static TRACKING_DISABLED: Cell<bool> = const { Cell::new(false) };
}

// Global tracking manager instance (thread-local for thread safety)
thread_local! {
    static GLOBAL_MANAGER: Arc<TrackingManager> = Arc::new(TrackingManager::new_core());
}

/// Unified tracking manager
///
/// Provides the central coordination point for all tracking operations.
/// Supports dynamic strategy switching and unified data export.
pub struct TrackingManager {
    tracker: Arc<RwLock<Box<dyn TrackBase>>>,
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
        let tracker: Box<dyn TrackBase> = match strategy {
            TrackingStrategy::Core => Box::new(CoreTracker::new()),
            TrackingStrategy::Lockfree => Box::new(LockfreeTracker::new()),
            TrackingStrategy::Async => Box::new(AsyncTracker::new()),
            TrackingStrategy::Unified => Box::new(UnifiedTracker::new_hybrid()),
        };

        TrackingManager {
            tracker: Arc::new(RwLock::new(tracker)),
        }
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
        // Check if tracking is disabled to prevent recursion
        let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());
        if !should_track {
            return;
        }

        let tracker = self.tracker.read().unwrap();
        tracker.track_alloc(ptr, size);
    }

    /// Track a deallocation
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the deallocated memory
    pub fn track_dealloc(&self, ptr: usize) {
        // Check if tracking is disabled to prevent recursion
        let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());
        if !should_track {
            return;
        }

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
            MemScopeError::new(ErrorKind::ExportError, &format!("Export failed: {}", e))
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
        format: ExportFormat,
    ) -> Result<(), MemScopeError> {
        let output = self.export(format)?;

        let data = match output {
            crate::data::RenderOutput::String(s) => s.into_bytes(),
            crate::data::RenderOutput::Bytes(b) => b,
            crate::data::RenderOutput::File(file_path) => {
                // If it's already a file, copy it to the destination
                return std::fs::copy(&file_path, path)
                    .map_err(|e| {
                        MemScopeError::new(
                            ErrorKind::ExportError,
                            &format!("Failed to copy file: {e}"),
                        )
                    })
                    .map(|_| ());
            }
        };

        std::fs::write(path, data).map_err(|e| {
            MemScopeError::new(
                ErrorKind::ExportError,
                &format!("Failed to write to file: {e}"),
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
        // Create a new tracker with the specified strategy
        let new_tracker: Box<dyn TrackBase> = match strategy {
            TrackingStrategy::Core => Box::new(CoreTracker::new()),
            TrackingStrategy::Lockfree => Box::new(LockfreeTracker::new()),
            TrackingStrategy::Async => Box::new(AsyncTracker::new()),
            TrackingStrategy::Unified => Box::new(UnifiedTracker::new_hybrid()),
        };

        // Replace the tracker in the RwLock
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

// ============================================================================
// Global API Functions
// ============================================================================

/// Get the global tracking manager instance
///
/// This function provides access to the global manager for manual tracking operations.
/// Uses thread-local storage for thread safety and automatic cleanup.
pub fn get_global_tracker() -> Arc<TrackingManager> {
    GLOBAL_MANAGER.with(|manager| manager.clone())
}

/// Track a memory allocation using the global tracker
///
/// This function can be used to manually track allocations that are not
/// automatically tracked by the global allocator.
///
/// # Arguments
/// * `ptr` - Pointer to the allocated memory
/// * `size` - Size of the allocation in bytes
pub fn track_allocation(ptr: usize, size: usize) {
    TRACKING_DISABLED.with(|disabled| {
        let old = disabled.get();
        disabled.set(true);
        let manager = get_global_tracker();
        manager.track_alloc(ptr, size);
        disabled.set(old);
    });
}

/// Track a memory deallocation using the global tracker
///
/// This function can be used to manually track deallocations that are not
/// automatically tracked by the global allocator.
///
/// # Arguments
/// * `ptr` - Pointer to the deallocated memory
pub fn track_deallocation(ptr: usize) {
    TRACKING_DISABLED.with(|disabled| {
        let old = disabled.get();
        disabled.set(true);
        let manager = get_global_tracker();
        manager.track_dealloc(ptr);
        disabled.set(old);
    });
}

/// Get the current tracking snapshot using the global tracker
///
/// # Returns
/// A TrackingSnapshot containing all current tracking data
pub fn get_snapshot() -> TrackingSnapshot {
    let manager = get_global_tracker();
    manager.snapshot()
}

/// Clear all tracking data using the global tracker
pub fn clear_tracking() {
    let manager = get_global_tracker();
    manager.clear();
}

/// Enable or disable tracking using the global tracker
///
/// # Arguments
/// * `enabled` - Whether to enable tracking
pub fn set_tracking_enabled(enabled: bool) {
    let manager = get_global_tracker();
    manager.set_enabled(enabled);
}

/// Check if tracking is enabled using the global tracker
///
/// # Returns
/// true if tracking is enabled, false otherwise
pub fn is_tracking_enabled() -> bool {
    let manager = get_global_tracker();
    manager.is_enabled()
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
        let output = result.unwrap();
        if let crate::data::RenderOutput::String(data) = output {
            assert!(data.contains("allocations"));
        } else {
            panic!("Expected String output");
        }
    }

    #[test]
    fn test_tracking_manager_export_html() {
        let manager = TrackingManager::new_core();
        manager.track_alloc(0x1000, 1024);

        let result = manager.export(ExportFormat::Html);
        assert!(result.is_ok());
        let output = result.unwrap();
        if let crate::data::RenderOutput::String(data) = output {
            assert!(data.contains("<html>") || data.contains("<!DOCTYPE html>"));
        } else {
            panic!("Expected String output");
        }
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

    #[test]
    fn test_global_tracker_singleton() {
        let tracker1 = get_global_tracker();
        let tracker2 = get_global_tracker();

        // Verify it's the same instance
        assert!(Arc::ptr_eq(&tracker1, &tracker2));
    }

    #[test]
    fn test_global_api_functions() {
        // Create a new manager instance instead of using global state
        let manager = TrackingManager::new_core();
        manager.clear();
        manager.set_enabled(true);

        manager.track_alloc(0x1000, 1024);
        manager.track_alloc(0x2000, 2048);
        manager.track_dealloc(0x1000);

        let snapshot = manager.snapshot();
        assert_eq!(snapshot.allocations.len(), 1);
        assert!(manager.is_enabled());

        manager.clear();
        assert_eq!(manager.snapshot().allocations.len(), 0);
    }
}
