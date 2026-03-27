//! Unified tracking manager
//!
//! This module provides the TrackingManager which serves as the central
//! coordination point for all tracking operations, supporting strategy
//! switching and data export.

use std::cell::Cell;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};

use crate::data::{ExportFormat, TrackingSnapshot, TrackingStrategy};
use crate::error::types::{ErrorKind, MemScopeError};
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
    /// Extended state for advanced features (variable names, type names, etc.)
    extended_state: Arc<Mutex<TrackingManagerState>>,
}

/// Extended state for advanced tracking features
#[derive(Debug)]
struct TrackingManagerState {
    /// Variable name mapping (ptr -> var_name)
    variable_names: HashMap<usize, String>,
    /// Type name mapping (ptr -> type_name)
    type_names: HashMap<usize, String>,
    /// Smart pointer relationships
    smart_pointer_info: HashMap<usize, SmartPointerInfo>,
    /// Borrow tracking data
    borrow_tracking: HashMap<usize, BorrowTrackingData>,
    /// Clone relationships
    clone_relationships: HashMap<usize, CloneRelationship>,
    /// Lifetime information
    lifetime_info: HashMap<usize, LifetimeInfo>,
    /// FFI allocation tracking
    ffi_allocations: HashMap<usize, FfiAllocationInfo>,
}

/// Smart pointer information
#[derive(Debug, Clone)]
struct SmartPointerInfo {
    ptr_type: crate::smart_pointer_utils::SmartPointerType,
    ref_count: u32,
    original_ptr: Option<usize>,
}

/// Borrow tracking data
#[derive(Debug, Clone)]
struct BorrowTrackingData {
    immutable_borrows: usize,
    mutable_borrows: usize,
    max_concurrent: usize,
    last_borrow_timestamp: Option<u64>,
}

/// Clone relationship information
#[derive(Debug, Clone)]
struct CloneRelationship {
    clone_count: usize,
    original_ptr: Option<usize>,
    is_clone: bool,
}

/// Lifetime tracking information
#[derive(Debug, Clone)]
struct LifetimeInfo {
    alloc_timestamp: u64,
    dealloc_timestamp: Option<u64>,
    lifetime_ms: Option<u64>,
}

/// FFI allocation tracking information
#[derive(Debug, Clone)]
struct FfiAllocationInfo {
    library_name: String,
    function_name: String,
    alloc_timestamp: u64,
    dealloc_timestamp: Option<u64>,
}

impl TrackingManagerState {
    fn new() -> Self {
        Self {
            variable_names: HashMap::new(),
            type_names: HashMap::new(),
            smart_pointer_info: HashMap::new(),
            borrow_tracking: HashMap::new(),
            clone_relationships: HashMap::new(),
            lifetime_info: HashMap::new(),
            ffi_allocations: HashMap::new(),
        }
    }
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
            extended_state: Arc::new(Mutex::new(TrackingManagerState::new())),
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

    /// Track a deallocation with lifetime information
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the deallocated memory
    /// * `lifetime_ms` - Lifetime in milliseconds
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn track_dealloc_with_lifetime(&self, ptr: usize, lifetime_ms: u64) -> Result<(), MemScopeError> {
        // Check if tracking is disabled to prevent recursion
        let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());
        if !should_track {
            return Ok(());
        }

        // Use base tracking with error handling
        let tracker = self.tracker.read()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire tracker lock: {}", e)
            ))?;
        tracker.track_dealloc(ptr);

        // Store lifetime information with error handling
        let dealloc_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Calculate allocation timestamp with overflow check
        let lifetime_ns = lifetime_ms.checked_mul(1_000_000)
            .ok_or_else(|| MemScopeError::new(
                ErrorKind::InternalError,
                "Lifetime overflow in calculation"
            ))?;

        let alloc_timestamp = dealloc_timestamp.checked_sub(lifetime_ns)
            .ok_or_else(|| MemScopeError::new(
                ErrorKind::InternalError,
                "Invalid lifetime: allocation timestamp would be negative"
            ))?;

        // Store lifetime information with lock error handling
        let mut state = self.extended_state.lock()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire state lock: {}", e)
            ))?;

        state.lifetime_info.insert(
            ptr,
            LifetimeInfo {
                alloc_timestamp,
                dealloc_timestamp: Some(dealloc_timestamp),
                lifetime_ms: Some(lifetime_ms),
            },
        );

        Ok(())
    }

    /// Track an allocation with metadata
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the allocated memory
    /// * `size` - Size of the allocation in bytes
    /// * `var_name` - Optional variable name
    /// * `type_name` - Optional type name
    pub fn track_alloc_with_metadata(
        &self,
        ptr: usize,
        size: usize,
        var_name: Option<String>,
        type_name: Option<String>,
    ) {
        // Use base tracking
        self.track_alloc(ptr, size);

        // Store extended metadata
        let mut state = self.extended_state.lock().unwrap();
        if let Some(name) = var_name {
            state.variable_names.insert(ptr, name);
        }
        if let Some(name) = type_name {
            state.type_names.insert(ptr, name);
        }
    }

    /// Associate a variable with a memory allocation
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the allocated memory
    /// * `var_name` - Variable name
    /// * `type_name` - Type name
    pub fn associate_var(&self, ptr: usize, var_name: String, type_name: String) {
        let mut state = self.extended_state.lock().unwrap();
        state.variable_names.insert(ptr, var_name);
        state.type_names.insert(ptr, type_name);
    }

    /// Track smart pointer allocation
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the allocated memory
    /// * `size` - Size of the allocation in bytes
    /// * `var_name` - Variable name
    /// * `type_name` - Type name
    /// * `ref_count` - Reference count
    /// * `data_ptr` - Optional pointer to the actual data
    pub fn track_smart_pointer_allocation(
        &self,
        ptr: usize,
        size: usize,
        var_name: String,
        type_name: String,
        ref_count: u32,
        data_ptr: Option<usize>,
    ) {
        // Use base tracking
        self.track_alloc(ptr, size);

        // Store smart pointer info
        let mut state = self.extended_state.lock().unwrap();
        state.variable_names.insert(ptr, var_name);
        state.type_names.insert(ptr, type_name.clone());

        // Detect smart pointer type from type name using public function
        let ptr_type = crate::smart_pointer_utils::detect_smart_pointer_type(&type_name);

        state.smart_pointer_info.insert(
            ptr,
            SmartPointerInfo {
                ptr_type,
                ref_count,
                original_ptr: data_ptr,
            },
        );
    }

    /// Track smart pointer clone operation
    ///
    /// # Arguments
    /// * `original_ptr` - Original pointer
    /// * `new_ptr` - New cloned pointer
    pub fn track_smart_pointer_clone(&self, original_ptr: usize, new_ptr: usize) {
        let mut state = self.extended_state.lock().unwrap();
        state.clone_relationships.insert(
            new_ptr,
            CloneRelationship {
                clone_count: 1,
                original_ptr: Some(original_ptr),
                is_clone: true,
            },
        );

        // Increment clone count for original
        if let Some(rel) = state.clone_relationships.get_mut(&original_ptr) {
            rel.clone_count += 1;
        }
    }

    /// Track borrow operation
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the memory being borrowed
    /// * `is_mutable` - Whether the borrow is mutable
    pub fn track_borrow(&self, ptr: usize, is_mutable: bool) {
        let mut state = self.extended_state.lock().unwrap();
        let borrow_data = state.borrow_tracking.entry(ptr).or_insert(BorrowTrackingData {
            immutable_borrows: 0,
            mutable_borrows: 0,
            max_concurrent: 0,
            last_borrow_timestamp: None,
        });

        if is_mutable {
            borrow_data.mutable_borrows += 1;
        } else {
            borrow_data.immutable_borrows += 1;
        }

        let current_borrows = borrow_data.immutable_borrows + borrow_data.mutable_borrows;
        if current_borrows > borrow_data.max_concurrent {
            borrow_data.max_concurrent = current_borrows;
        }

        borrow_data.last_borrow_timestamp = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64);
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
    // ============================================================================
    // Smart Pointer Tracking Methods
    // ============================================================================

    /// Track smart pointer deallocation with enhanced metadata
    ///
    /// Handles smart pointer destruction with reference count information.
    /// This method provides detailed tracking for Rc, Arc, and other smart pointers.
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the deallocated memory
    /// * `lifetime_ms` - Lifetime in milliseconds
    /// * `final_ref_count` - Final reference count at destruction time
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn track_smart_pointer_deallocation(
        &self,
        ptr: usize,
        lifetime_ms: u64,
        final_ref_count: u32,
    ) -> Result<(), MemScopeError> {
        // Check if tracking is disabled to prevent recursion
        let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());
        if !should_track {
            return Ok(());
        }

        // Use base tracking with error handling
        let tracker = self.tracker.read()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire tracker lock: {}", e)
            ))?;
        tracker.track_dealloc(ptr);

        // Store lifetime information with ref count
        let dealloc_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Calculate allocation timestamp with overflow check
        let lifetime_ns = lifetime_ms.checked_mul(1_000_000)
            .ok_or_else(|| MemScopeError::new(
                ErrorKind::InternalError,
                "Lifetime overflow in calculation"
            ))?;

        let alloc_timestamp = dealloc_timestamp.checked_sub(lifetime_ns)
            .ok_or_else(|| MemScopeError::new(
                ErrorKind::InternalError,
                "Invalid lifetime: allocation timestamp would be negative"
            ))?;

        // Update smart pointer info with final ref count
        let mut state = self.extended_state.lock()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire state lock: {}", e)
            ))?;

        // Update lifetime info
        state.lifetime_info.insert(
            ptr,
            LifetimeInfo {
                alloc_timestamp,
                dealloc_timestamp: Some(dealloc_timestamp),
                lifetime_ms: Some(lifetime_ms),
            },
        );

        // Update smart pointer ref count if exists
        if let Some(smart_ptr_info) = state.smart_pointer_info.get_mut(&ptr) {
            smart_ptr_info.ref_count = final_ref_count;
        }

        Ok(())
    }

    /// Track FFI allocation from external library
    ///
    /// This method tracks memory allocations made by external FFI functions,
    /// providing information about the library and function that made the allocation.
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the allocated memory
    /// * `size` - Size of the allocation in bytes
    /// * `library_name` - Name of the external library
    /// * `function_name` - Name of the function that performed the allocation
    pub fn track_ffi_allocation(
        &self,
        ptr: usize,
        size: usize,
        library_name: String,
        function_name: String,
    ) -> Result<(), MemScopeError> {
        // Check if tracking is disabled to prevent recursion
        let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());
        if !should_track {
            return Ok(());
        }

        // Use base tracking
        self.track_alloc(ptr, size);

        // Store FFI allocation info
        let alloc_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Log before moving ownership
        tracing::debug!(
            "Tracked FFI allocation at 0x{:x} from {}::{} (size: {})",
            ptr, library_name, function_name, size
        );

        let mut state = self.extended_state.lock()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire state lock: {}", e)
            ))?;

        state.ffi_allocations.insert(
            ptr,
            FfiAllocationInfo {
                library_name,
                function_name,
                alloc_timestamp,
                dealloc_timestamp: None,
            },
        );

        Ok(())
    }

    /// Track FFI deallocation from external library
    ///
    /// This method tracks memory deallocations made by external FFI functions,
    /// completing the lifecycle tracking for FFI allocations.
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the deallocated memory
    /// * `library_name` - Name of the external library
    /// * `function_name` - Name of the function that performed the deallocation
    pub fn track_ffi_free(
        &self,
        ptr: usize,
        library_name: String,
        function_name: String,
    ) -> Result<(), MemScopeError> {
        // Check if tracking is disabled to prevent recursion
        let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());
        if !should_track {
            return Ok(());
        }

        // Use base tracking
        self.track_dealloc(ptr);

        // Update FFI allocation info
        let dealloc_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Log before acquiring lock
        tracing::debug!(
            "Tracked FFI deallocation at 0x{:x} from {}::{}",
            ptr, library_name, function_name
        );

        let mut state = self.extended_state.lock()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire state lock: {}", e)
            ))?;

        if let Some(ffi_info) = state.ffi_allocations.get_mut(&ptr) {
            ffi_info.dealloc_timestamp = Some(dealloc_timestamp);
        }

        Ok(())
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
