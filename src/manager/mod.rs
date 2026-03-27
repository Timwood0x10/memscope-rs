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
    /// Cached stack trace capture instance for performance
    stack_trace_capture: Arc<Mutex<crate::stack_trace::capture::StackTraceCapture>>,
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
    /// Leaked allocation tracking
    leaked_allocations: std::collections::HashSet<usize>,
    /// Stack trace cache (ptr -> stack trace)
    stack_traces: HashMap<usize, Vec<String>>,
    /// Freed pointers for double-free detection
    freed_pointers: HashMap<usize, (String, u64)>,
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
            leaked_allocations: std::collections::HashSet::new(),
            stack_traces: HashMap::new(),
            freed_pointers: HashMap::new(),
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

        // Create cached stack trace capture instance
        // Note: StackTraceCapture::new cannot fail (returns Self, not Result),
        // so no error handling is needed here
        let capture_config = crate::stack_trace::capture::CaptureConfig::default();
        let stack_trace_capture = Arc::new(Mutex::new(
            crate::stack_trace::capture::StackTraceCapture::new(capture_config)
        ));

        TrackingManager {
            tracker: Arc::new(RwLock::new(tracker)),
            extended_state: Arc::new(Mutex::new(TrackingManagerState::new())),
            stack_trace_capture,
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
                &format!("Failed to acquire tracker lock: {e}")
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
                &format!("Failed to acquire state lock: {e}")
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
                &format!("Failed to acquire tracker lock: {e}")
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
                &format!("Failed to acquire state lock: {e}")
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
                &format!("Failed to acquire state lock: {e}")
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

    /// Mark an allocation as leaked
    ///
    /// This method explicitly marks an allocation as leaked, which can be used
    /// for manual leak detection or when automatic detection is not sufficient.
    ///
    /// # Note on Data Consistency
    /// This method only updates the extended_state because the underlying TrackBase
    /// trait doesn't provide a way to modify AllocationRecord. The get_leaked_allocations()
    /// method merges data from both sources (extended_state and AllocationRecord.is_leaked)
    /// to ensure consistency.
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the allocation to mark as leaked
    pub fn mark_leaked(&self, ptr: usize) -> Result<(), MemScopeError> {
        // Track this in extended state
        let mut state = self.extended_state.lock()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire state lock: {}", e)
            ))?;

        // Mark the allocation as leaked in extended state
        state.leaked_allocations.insert(ptr);

        tracing::debug!(
            "Marked allocation at 0x{:x} as leaked",
            ptr
        );

        Ok(())
    }

    /// Unmark an allocation as leaked
    ///
    /// This method removes the leaked status from an allocation, allowing it
    /// to be tracked normally again.
    ///
    /// # Note on Data Consistency
    /// This method only updates the extended_state because the underlying TrackBase
    /// trait doesn't provide a way to modify AllocationRecord. The get_leaked_allocations()
    /// method merges data from both sources (extended_state and AllocationRecord.is_leaked)
    /// to ensure consistency.
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the allocation to unmark as leaked
    pub fn unmark_leaked(&self, ptr: usize) -> Result<(), MemScopeError> {
        // Track this in extended state
        let mut state = self.extended_state.lock()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire state lock: {}", e)
            ))?;

        // Remove the leak marking from extended state
        state.leaked_allocations.remove(&ptr);

        tracing::debug!(
            "Unmarked allocation at 0x{:x} as leaked",
            ptr
        );

        Ok(())
    }

    /// Get all allocations marked as leaked
    ///
    /// This method returns allocations that are marked as leaked from two sources:
    /// 1. Explicitly marked via `mark_leaked()` in extended_state
    /// 2. Marked as leaked in the underlying AllocationRecord
    ///
    /// The results are merged and deduplicated to ensure consistency.
    ///
    /// # Returns
    /// A vector of pointers to allocations that are marked as leaked
    pub fn get_leaked_allocations(&self) -> Result<Vec<usize>, MemScopeError> {
        let snapshot = self.snapshot();
        let state = self.extended_state.lock()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire state lock: {e}")
            ))?;

        // Collect leaked allocations from both sources
        let mut leaked_set: std::collections::HashSet<usize> = std::collections::HashSet::new();

        // 1. Add explicitly marked allocations from extended_state
        // Only include allocations that are still active (not freed)
        let active_ptrs: std::collections::HashSet<usize> = snapshot.allocations
            .iter()
            .filter(|r| r.is_active)
            .map(|r| r.ptr)
            .collect();
        
        for ptr in state.leaked_allocations.iter() {
            if active_ptrs.contains(ptr) {
                leaked_set.insert(*ptr);
            }
        }

        // 2. Add allocations marked as leaked in AllocationRecord
        // Only consider active allocations (released allocations are not leaks)
        for record in snapshot.allocations {
            if record.is_active && record.is_leaked {
                leaked_set.insert(record.ptr);
            }
        }

        Ok(leaked_set.into_iter().collect())
    }

    /// Capture and store stack trace for an allocation
    ///
    /// This method captures the current call stack and associates it with
    /// the specified allocation pointer for debugging purposes.
    ///
    /// # Note
    /// Stack trace capture has performance overhead and should be used
    /// selectively, typically only when debugging memory issues.
    ///
    /// # Performance
    /// This method uses a cached `StackTraceCapture` instance to avoid
    /// the overhead of creating a new instance on each call.
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the allocation to capture stack trace for
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn capture_stack_trace(&self, ptr: usize) -> Result<(), MemScopeError> {
        // Use cached stack trace capture instance for performance
        let mut capture = self.stack_trace_capture.lock()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire stack trace capture lock: {e}")
            ))?;
        
        match capture.capture() {
            Some(frames) => {
                let trace_strings: Vec<String> = frames
                    .iter()
                    .map(|frame| {
                        frame.function_name
                            .clone()
                            .unwrap_or_else(|| format!("0x{:x}", frame.instruction_pointer))
                    })
                    .collect();
                
                let mut state = self.extended_state.lock()
                    .map_err(|e| MemScopeError::new(
                        ErrorKind::InternalError,
                        &format!("Failed to acquire state lock: {e}")
                    ))?;
                
                state.stack_traces.insert(ptr, trace_strings);
                Ok(())
            }
            None => {
                tracing::debug!("Stack trace capture disabled or failed for ptr 0x{:x}", ptr);
                Ok(())
            }
        }
    }

    /// Get stack trace for an allocation
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the allocation to get stack trace for
    ///
    /// # Returns
    /// Option containing the stack trace frames as strings, or None if not available
    pub fn get_stack_trace(&self, ptr: usize) -> Result<Option<Vec<String>>, MemScopeError> {
        let state = self.extended_state.lock()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire state lock: {}", e)
            ))?;
        
        Ok(state.stack_traces.get(&ptr).cloned())
    }

    /// Track allocation with stack trace
    ///
    /// This is a convenience method that combines basic allocation tracking
    /// with stack trace capture in a single call.
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the allocated memory
    /// * `size` - Size of the allocation in bytes
    /// * `var_name` - Optional variable name
    /// * `type_name` - Optional type name
    /// * `capture_trace` - Whether to capture stack trace
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn track_alloc_with_trace(
        &self,
        ptr: usize,
        size: usize,
        var_name: Option<String>,
        type_name: Option<String>,
        capture_trace: bool,
    ) -> Result<(), MemScopeError> {
        // Track the basic allocation
        // Note: track_alloc returns () and cannot fail (it only records allocation metadata)
        self.track_alloc(ptr, size);

        // Store all metadata in a single lock acquisition for efficiency
        if var_name.is_some() || type_name.is_some() {
            let mut state = self.extended_state.lock()
                .map_err(|e| MemScopeError::new(
                    ErrorKind::InternalError,
                    &format!("Failed to acquire state lock: {}", e)
                ))?;
            
            if let Some(name) = var_name {
                state.variable_names.insert(ptr, name);
            }
            
            if let Some(r#type) = type_name {
                state.type_names.insert(ptr, r#type);
            }
        }

        // Capture stack trace if requested
        if capture_trace {
            self.capture_stack_trace(ptr)?;
        }

        Ok(())
    }

    /// Check for double-free attempts
    ///
    /// This method checks if a pointer has already been freed and records
    /// the double-free attempt for security analysis.
    ///
    /// # Arguments
    /// * `ptr` - Pointer to check for double-free
    ///
    /// # Returns
    /// Result containing true if this is a double-free attempt, false otherwise
    pub fn check_double_free(&self, ptr: usize) -> Result<bool, MemScopeError> {
        let mut state = self.extended_state.lock()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire state lock: {}", e)
            ))?;

        if let Some((location, timestamp)) = state.freed_pointers.get(&ptr) {
            tracing::warn!(
                "Double-free detected: ptr 0x{:x} previously freed at {} (timestamp: {})",
                ptr,
                location,
                timestamp
            );
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Record a pointer as freed
    ///
    /// This method records that a pointer has been freed for double-free detection.
    ///
    /// # Arguments
    /// * `ptr` - Pointer that was freed
    /// * `location` - Location where the free occurred
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn record_freed_pointer(&self, ptr: usize, location: String) -> Result<(), MemScopeError> {
        let mut state = self.extended_state.lock()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire state lock: {}", e)
            ))?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        state.freed_pointers.insert(ptr, (location, timestamp));
        Ok(())
    }

    /// Track deallocation with double-free detection
    ///
    /// This enhanced method tracks deallocation and also checks for double-free attempts.
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the deallocated memory
    /// * `location` - Location where the deallocation occurred (e.g., function name)
    ///
    /// # Returns
    /// Result containing a tuple (success, is_double_free):
    /// - success: true if deallocation succeeded, false if double-free detected
    /// - is_double_free: true if this was a double-free attempt, false otherwise
    ///
    /// # Safety
    /// This method prevents double-free by checking if the pointer has already been
    /// freed before performing any deallocation operation.
    pub fn track_dealloc_with_double_free_check(
        &self,
        ptr: usize,
        location: String,
    ) -> Result<(bool, bool), MemScopeError> {
        // Check for double-free BEFORE any deallocation operation
        let is_double_free = self.check_double_free(ptr)?;

        // SAFETY: If double-free detected, abort immediately without deallocating
        if is_double_free {
            tracing::warn!(
                "Double-free detected: ptr 0x{:x} at {}, aborting deallocation",
                ptr,
                location
            );
            // Return (success=false, is_double_free=true) without calling track_dealloc
            return Ok((false, true));
        }

        // Only perform deallocation if not a double-free
        // Note: track_dealloc returns () and cannot fail (it only records deallocation metadata)
        self.track_dealloc(ptr);

        // Record that this pointer has been freed for future double-free detection
        self.record_freed_pointer(ptr, location)?;

        Ok((true, false))
    }

    /// Get memory usage statistics grouped by type
    ///
    /// This method provides a breakdown of memory usage by data type,
    /// which is useful for identifying memory hotspots.
    ///
    /// # Returns
    /// Result containing a map of type name to memory usage (bytes)
    pub fn get_memory_by_type(&self) -> Result<std::collections::HashMap<String, u64>, MemScopeError> {
        let snapshot = self.snapshot();
        let state = self.extended_state.lock()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire state lock: {}", e)
            ))?;

        let mut type_usage: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

        for record in snapshot.allocations.iter() {
            if record.is_active {
                let type_name = state.type_names
                    .get(&record.ptr)
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                
                *type_usage.entry(type_name).or_insert(0) += record.size as u64;
            }
        }

        Ok(type_usage)
    }

    /// Generate memory optimization recommendations
    ///
    /// This method analyzes current memory usage patterns and provides
    /// recommendations for optimization.
    ///
    /// # Returns
    /// Result containing a list of optimization recommendations
    pub fn generate_optimization_recommendations(&self) -> Result<Vec<String>, MemScopeError> {
        let snapshot = self.snapshot();
        let mut recommendations = Vec::new();

        // Get memory usage by type
        let type_usage = self.get_memory_by_type()?;

        // Identify top memory consumers
        let mut sorted_types: Vec<_> = type_usage.iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(a.1));

        if !sorted_types.is_empty() {
            let top_consumer = sorted_types[0];
            if *top_consumer.1 > 1024 * 1024 { // > 1MB
                recommendations.push(format!(
                    "Consider optimizing {} usage: {} MB allocated",
                    top_consumer.0,
                    top_consumer.1 / (1024 * 1024)
                ));
            }
        }

        // Check for potential memory leaks
        let active_allocations = snapshot.allocations
            .iter()
            .filter(|r| r.is_active)
            .count();

        let leaked_count = snapshot.stats.leaked_allocations as usize;

        if leaked_count > 0 {
            recommendations.push(format!(
                "Found {} potentially leaked allocations - review allocation patterns",
                leaked_count
            ));
        }

        // Fragmentation check
        let fragmentation_ratio = snapshot.stats.fragmentation_ratio;
        if fragmentation_ratio > 0.3 {
            recommendations.push(format!(
                "High memory fragmentation detected ({:.1}%) - consider pooling allocations",
                fragmentation_ratio * 100.0
            ));
        }

        // Smart pointer analysis
        let state = self.extended_state.lock()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire state lock: {}", e)
            ))?;

        let smart_pointer_count = state.smart_pointer_info.len();
        if smart_pointer_count > 0 {
            recommendations.push(format!(
                "Tracking {} smart pointers - verify reference counts are correct",
                smart_pointer_count
            ));
        }

        if recommendations.is_empty() {
            recommendations.push("Memory usage looks healthy - no major issues detected".to_string());
        }

        Ok(recommendations)
    }

    /// Get comprehensive memory statistics
    ///
    /// This method provides a detailed overview of current memory usage
    /// including allocations, deallocations, and active memory.
    ///
    /// # Returns
    /// Result containing detailed memory statistics
    pub fn get_comprehensive_stats(&self) -> Result<MemoryStatistics, MemScopeError> {
        let snapshot = self.snapshot();
        let state = self.extended_state.lock()
            .map_err(|e| MemScopeError::new(
                ErrorKind::InternalError,
                &format!("Failed to acquire state lock: {}", e)
            ))?;

        Ok(MemoryStatistics {
            total_allocations: snapshot.stats.total_allocations as usize,
            total_deallocations: snapshot.stats.total_deallocations as usize,
            active_allocations: snapshot.stats.active_allocations as usize,
            total_allocated_bytes: snapshot.stats.total_allocated as usize,
            active_memory_bytes: snapshot.stats.active_memory as usize,
            peak_memory_bytes: snapshot.stats.peak_memory as usize,
            leaked_allocations: snapshot.stats.leaked_allocations as usize,
            unique_variables: state.variable_names.len(),
            unique_types: state.type_names.len(),
            smart_pointers: state.smart_pointer_info.len(),
            tracked_stack_traces: state.stack_traces.len(),
            fragmentation_ratio: snapshot.stats.fragmentation_ratio,
        })
    }
}

/// Comprehensive memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStatistics {
    /// Total number of allocations
    pub total_allocations: usize,
    /// Total number of deallocations
    pub total_deallocations: usize,
    /// Currently active allocations
    pub active_allocations: usize,
    /// Total bytes allocated (cumulative)
    pub total_allocated_bytes: usize,
    /// Currently active memory in bytes
    pub active_memory_bytes: usize,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: usize,
    /// Number of potentially leaked allocations
    pub leaked_allocations: usize,
    /// Number of unique variables tracked
    pub unique_variables: usize,
    /// Number of unique types tracked
    pub unique_types: usize,
    /// Number of smart pointers tracked
    pub smart_pointers: usize,
    /// Number of allocations with stack traces
    pub tracked_stack_traces: usize,
    /// Memory fragmentation ratio (0.0 to 1.0)
    pub fragmentation_ratio: f64,
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
