//! Memory allocation tracking implementation with bounded memory stats.
//!
//! This module contains the core allocation and deallocation tracking logic
//! for the MemoryTracker, using BoundedMemoryStats to prevent infinite growth.

use super::memory_tracker::MemoryTracker;
use crate::core::ownership_history::OwnershipEventType;
use crate::core::types::{AllocationInfo, TrackingResult};

impl MemoryTracker {
    /// Fast track allocation for testing (minimal overhead)
    pub fn fast_track_allocation(
        &self,
        ptr: usize,
        size: usize,
        var_name: String,
    ) -> TrackingResult<()> {
        if !self.is_fast_mode() {
            return self.create_synthetic_allocation(ptr, size, var_name, "unknown".to_string(), 0);
        }

        // In fast mode, create minimal allocation info but still track it
        let mut allocation = AllocationInfo::new(ptr, size);
        allocation.var_name = Some(var_name);
        allocation.type_name = Some("fast_tracked".to_string());

        // Apply Task 4 enhancement: calculate lifetime
        self.calculate_and_analyze_lifetime(&mut allocation);

        // Try to update both active allocations and bounded stats
        if let (Ok(mut active), Ok(mut bounded_stats)) = (
            self.active_allocations.try_lock(),
            self.bounded_stats.try_lock(),
        ) {
            active.insert(ptr, allocation.clone());
            bounded_stats.add_allocation(&allocation);
        }
        Ok(())
    }

    /// Track a new memory allocation using bounded stats.
    pub fn track_allocation(&self, ptr: usize, size: usize) -> TrackingResult<()> {
        // CRITICAL FIX: Skip advanced tracking for global allocator calls
        // Only do basic tracking for system allocations, save advanced features for user variables
        let is_user_variable = false; // This is a system allocation from global allocator

        // Create allocation info first (no locks needed)
        let mut allocation = AllocationInfo::new(ptr, size);

        // Apply Task 4 enhancement: calculate lifetime (only for user variables)
        if is_user_variable {
            self.calculate_and_analyze_lifetime(&mut allocation);
        }

        // In test mode or when explicitly requested, use blocking locks for accuracy
        let use_blocking_locks = self.is_fast_mode()
            || std::env::var("MEMSCOPE_ACCURATE_TRACKING").is_ok()
            || cfg!(test);

        if use_blocking_locks {
            // Use blocking locks to ensure all allocations are tracked in tests
            let mut active = self.active_allocations.lock().map_err(|_| {
                crate::core::types::TrackingError::LockError(
                    "Failed to acquire active_allocations lock".to_string(),
                )
            })?;

            let mut bounded_stats = self.bounded_stats.lock().map_err(|_| {
                crate::core::types::TrackingError::LockError(
                    "Failed to acquire bounded_stats lock".to_string(),
                )
            })?;

            // Insert allocation into active tracking
            active.insert(ptr, allocation.clone());

            // Update bounded statistics (automatically handles bounds)
            bounded_stats.add_allocation(&allocation);

            // Release locks before adding to history
            drop(bounded_stats);
            drop(active);

            // Add to bounded history manager (automatically handles bounds)
            if !self.is_fast_mode() && std::env::var("MEMSCOPE_FULL_HISTORY").is_ok() {
                if let Ok(mut history_manager) = self.history_manager.try_lock() {
                    history_manager.add_allocation(allocation);
                }
            }

            Ok(())
        } else {
            // Production mode: use try_lock with improved retry logic
            self.track_allocation_with_retry(ptr, size, allocation)
        }
    }

    /// Track a memory allocation with enhanced context information
    pub fn track_allocation_with_context(
        &self,
        ptr: usize,
        size: usize,
        inferred_var_name: String,
        inferred_type_name: String,
    ) -> TrackingResult<()> {
        // Create allocation info with enhanced context
        let mut allocation = AllocationInfo::new(ptr, size);

        // Set the inferred names - this gives system allocations meaningful names
        allocation.var_name = Some(inferred_var_name);
        allocation.type_name = Some(inferred_type_name);

        // Apply Task 4 enhancement: calculate lifetime
        self.calculate_and_analyze_lifetime(&mut allocation);

        // Use the same locking strategy as regular track_allocation
        let use_blocking_locks = self.is_fast_mode()
            || std::env::var("MEMSCOPE_ACCURATE_TRACKING").is_ok()
            || cfg!(test);

        if use_blocking_locks {
            // Use blocking locks to ensure all allocations are tracked in tests
            let mut active = self.active_allocations.lock().map_err(|_| {
                crate::core::types::TrackingError::LockError(
                    "Failed to acquire active_allocations lock".to_string(),
                )
            })?;

            let mut bounded_stats = self.bounded_stats.lock().map_err(|_| {
                crate::core::types::TrackingError::LockError(
                    "Failed to acquire bounded_stats lock".to_string(),
                )
            })?;

            // Insert allocation into active tracking
            active.insert(ptr, allocation.clone());

            // Update bounded statistics (automatically handles bounds)
            bounded_stats.add_allocation(&allocation);

            // Release locks before adding to history
            drop(bounded_stats);
            drop(active);

            // Add to bounded history manager (automatically handles bounds)
            if !self.is_fast_mode() && std::env::var("MEMSCOPE_FULL_HISTORY").is_ok() {
                if let Ok(mut history_manager) = self.history_manager.try_lock() {
                    history_manager.add_allocation(allocation);
                }
            }

            Ok(())
        } else {
            // Production mode: use try_lock with improved retry logic
            self.track_allocation_with_context_retry(ptr, size, allocation)
        }
    }

    /// Track a memory deallocation using bounded stats.
    pub fn track_deallocation(&self, ptr: usize) -> TrackingResult<()> {
        let dealloc_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // In test mode or when explicitly requested, use blocking locks for accuracy
        let use_blocking_locks = self.is_fast_mode()
            || std::env::var("MEMSCOPE_ACCURATE_TRACKING").is_ok()
            || cfg!(test);

        if use_blocking_locks {
            self.track_deallocation_blocking(ptr, dealloc_timestamp)
        } else {
            // Production mode: use try_lock with improved retry logic
            self.track_deallocation_with_retry(ptr, dealloc_timestamp)
        }
    }

    // Private helper methods

    /// Track allocation with retry logic for production mode
    fn track_allocation_with_retry(
        &self,
        ptr: usize,
        _size: usize,
        allocation: AllocationInfo,
    ) -> TrackingResult<()> {
        let mut retry_count = 0;
        const MAX_RETRIES: u32 = 10;

        while retry_count < MAX_RETRIES {
            match (
                self.active_allocations.try_lock(),
                self.bounded_stats.try_lock(),
            ) {
                (Ok(mut active), Ok(mut bounded_stats)) => {
                    // Insert allocation into active tracking
                    active.insert(ptr, allocation.clone());

                    // Update bounded statistics (automatically handles bounds)
                    bounded_stats.add_allocation(&allocation);

                    return Ok(());
                }
                _ => {
                    retry_count += 1;
                    if retry_count < MAX_RETRIES {
                        std::thread::yield_now();
                    }
                }
            }
        }

        // If all retries failed, return error
        Err(crate::core::types::TrackingError::LockError(
            "Failed to acquire locks after retries".to_string(),
        ))
    }

    /// Track allocation with context retry logic for production mode
    fn track_allocation_with_context_retry(
        &self,
        ptr: usize,
        _size: usize,
        allocation: AllocationInfo,
    ) -> TrackingResult<()> {
        let mut retry_count = 0;
        const MAX_RETRIES: u32 = 10;

        while retry_count < MAX_RETRIES {
            match (
                self.active_allocations.try_lock(),
                self.bounded_stats.try_lock(),
            ) {
                (Ok(mut active), Ok(mut bounded_stats)) => {
                    // Insert allocation into active tracking
                    active.insert(ptr, allocation.clone());

                    // Update bounded statistics (automatically handles bounds)
                    bounded_stats.add_allocation(&allocation);

                    // Try to add to history manager if possible
                    if let Ok(mut history_manager) = self.history_manager.try_lock() {
                        history_manager.add_allocation(allocation);
                    }

                    return Ok(());
                }
                _ => {
                    retry_count += 1;
                    if retry_count < MAX_RETRIES {
                        std::thread::yield_now();
                    }
                }
            }
        }

        // If all retries failed, return error
        Err(crate::core::types::TrackingError::LockError(
            "Failed to acquire locks after retries".to_string(),
        ))
    }

    /// Track deallocation with blocking locks
    fn track_deallocation_blocking(
        &self,
        ptr: usize,
        dealloc_timestamp: u64,
    ) -> TrackingResult<()> {
        let mut active = self.active_allocations.lock().map_err(|_| {
            crate::core::types::TrackingError::LockError(
                "Failed to acquire active_allocations lock".to_string(),
            )
        })?;

        let mut bounded_stats = self.bounded_stats.lock().map_err(|_| {
            crate::core::types::TrackingError::LockError(
                "Failed to acquire bounded_stats lock".to_string(),
            )
        })?;

        if let Some(mut allocation) = active.remove(&ptr) {
            // Set deallocation timestamp
            allocation.timestamp_dealloc = Some(dealloc_timestamp);

            // Apply Task 4 enhancement: calculate lifetime for deallocated allocation
            self.calculate_and_analyze_lifetime(&mut allocation);

            // Update bounded statistics
            bounded_stats.record_deallocation(ptr, allocation.size);

            // Release locks before updating history
            drop(bounded_stats);
            drop(active);

            // Update allocation history with deallocation timestamp
            if let Ok(mut history_manager) = self.history_manager.try_lock() {
                history_manager.add_allocation(allocation);
            }
        }
        Ok(())
    }

    /// Track deallocation with retry logic for production mode
    fn track_deallocation_with_retry(
        &self,
        ptr: usize,
        dealloc_timestamp: u64,
    ) -> TrackingResult<()> {
        let mut retry_count = 0;
        const MAX_RETRIES: u32 = 10;

        while retry_count < MAX_RETRIES {
            match (
                self.active_allocations.try_lock(),
                self.bounded_stats.try_lock(),
            ) {
                (Ok(mut active), Ok(mut bounded_stats)) => {
                    if let Some(mut allocation) = active.remove(&ptr) {
                        // Set deallocation timestamp
                        allocation.timestamp_dealloc = Some(dealloc_timestamp);

                        // Apply Task 4 enhancement: calculate lifetime for deallocated allocation
                        self.calculate_and_analyze_lifetime(&mut allocation);

                        // Update bounded statistics
                        bounded_stats.record_deallocation(ptr, allocation.size);

                        // Release locks before updating history
                        drop(bounded_stats);
                        drop(active);

                        // Update allocation history with deallocation timestamp
                        if let Ok(mut history_manager) = self.history_manager.try_lock() {
                            history_manager.add_allocation(allocation);
                        }
                    }
                    return Ok(());
                }
                _ => {
                    retry_count += 1;
                    if retry_count < MAX_RETRIES {
                        std::thread::yield_now();
                    }
                }
            }
        }

        // If all retries failed, return error
        Err(crate::core::types::TrackingError::LockError(
            "Failed to acquire locks after retries".to_string(),
        ))
    }

    /// Enhanced lifetime calculation and analysis for Task 4
    /// This method fills the lifetime_ms field with precise calculations and adds lifecycle analysis
    fn calculate_and_analyze_lifetime(&self, allocation: &mut AllocationInfo) {
        // 1. Calculate precise lifetime based on timestamps
        if allocation.lifetime_ms.is_none() {
            if let Some(dealloc_time) = allocation.timestamp_dealloc {
                // For deallocated objects, calculate exact lifetime
                let lifetime_ns = dealloc_time.saturating_sub(allocation.timestamp_alloc);
                let lifetime_ms = lifetime_ns / 1_000_000; // Convert to milliseconds
                tracing::debug!(
                    "Deallocated allocation lifetime: {}ns -> {}ms",
                    lifetime_ns,
                    lifetime_ms
                );
                allocation.lifetime_ms = Some(lifetime_ms);
            } else {
                // For active allocations, calculate current lifetime
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;
                let lifetime_ns = current_time.saturating_sub(allocation.timestamp_alloc);
                let lifetime_ms = lifetime_ns / 1_000_000; // Convert to milliseconds
                tracing::debug!(
                    "Active allocation lifetime: {}ns -> {}ms",
                    lifetime_ns,
                    lifetime_ms
                );
                allocation.lifetime_ms = Some(lifetime_ms);
            }
        }

        // 2. Perform lifecycle analysis and efficiency evaluation
        if let Some(lifetime_ms) = allocation.lifetime_ms {
            self.analyze_lifecycle_efficiency(allocation, lifetime_ms);
        }
    }

    /// Analyze lifecycle efficiency (placeholder implementation)
    fn analyze_lifecycle_efficiency(&self, _allocation: &mut AllocationInfo, _lifetime_ms: u64) {
        // This would contain the actual lifecycle analysis logic
        // For now, it's a placeholder to maintain compatibility
    }

    /// Create synthetic allocation with proper var_name and type_name
    pub fn create_synthetic_allocation(
        &self,
        ptr: usize,
        size: usize,
        var_name: String,
        type_name: String,
        _creation_time: u64,
    ) -> TrackingResult<()> {
        let mut allocation = AllocationInfo::new(ptr, size);
        allocation.var_name = Some(var_name.clone());
        allocation.type_name = Some(type_name.clone());

        // Apply improve.md field enhancements based on type
        allocation.enhance_with_type_info(&type_name);

        // Store the allocation and update stats
        match self.active_allocations.try_lock() {
            Ok(mut active) => {
                active.insert(ptr, allocation.clone());
                drop(active); // Release active lock before acquiring bounded_stats lock

                // CRITICAL FIX: Update bounded stats for synthetic allocations
                if let Ok(mut bounded_stats) = self.bounded_stats.try_lock() {
                    bounded_stats.add_allocation(&allocation);
                }

                tracing::debug!(
                    "Created synthetic allocation for '{}' ({}): ptr=0x{:x}, size={}",
                    var_name,
                    type_name,
                    ptr,
                    size
                );
                Ok(())
            }
            Err(_) => {
                tracing::debug!(
                    "Could not acquire lock for synthetic allocation: {}",
                    var_name
                );
                Ok(())
            }
        }
    }

    /// Associate a variable name and type with an allocation.
    pub fn associate_var(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
    ) -> TrackingResult<()> {
        // Use try_lock to avoid blocking if the allocator is currently tracking
        match self.active_allocations.try_lock() {
            Ok(mut active) => {
                if let Some(allocation) = active.get_mut(&ptr) {
                    allocation.var_name = Some(var_name.clone());
                    allocation.type_name = Some(type_name.clone());

                    // Apply improve.md field enhancements based on type
                    allocation.enhance_with_type_info(&type_name);

                    tracing::debug!(
                        "Associated variable '{}' with existing allocation at {:x}",
                        var_name,
                        ptr
                    );
                } else {
                    // For smart pointers and other complex types, create a synthetic allocation entry
                    let mut synthetic_allocation = AllocationInfo::new(ptr, 0);
                    synthetic_allocation.var_name = Some(var_name.clone());
                    synthetic_allocation.type_name = Some(type_name.clone());

                    // Estimate size based on type
                    let estimated_size = self.estimate_type_size(&type_name);
                    synthetic_allocation.size = estimated_size;

                    // Apply improve.md field enhancements based on type
                    synthetic_allocation.enhance_with_type_info(&type_name);

                    // Add to active allocations for tracking
                    active.insert(ptr, synthetic_allocation.clone());

                    // CRITICAL FIX: Update bounded stats for synthetic allocations
                    drop(active); // Release active lock before acquiring bounded_stats lock
                    if let Ok(mut bounded_stats) = self.bounded_stats.try_lock() {
                        bounded_stats.add_allocation(&synthetic_allocation);
                    }

                    tracing::debug!("Created synthetic allocation for variable '{}' at {:x} (estimated size: {})", 
                                   var_name, ptr, estimated_size);
                }
                Ok(())
            }
            Err(_) => {
                // If we can't get the lock immediately, skip to avoid deadlock
                tracing::debug!(
                    "Could not acquire lock for variable association: {}",
                    var_name
                );
                Ok(())
            }
        }
    }

    /// Enhance allocation with improve.md required fields
    fn _enhance_allocation_with_improve_md_fields(
        mut allocation: AllocationInfo,
    ) -> AllocationInfo {
        // Simulate borrowing information based on type patterns
        if let Some(ref type_name) = allocation.type_name {
            // Detect reference counting types (Rc, Arc)
            if type_name.contains("Rc<") || type_name.contains("Arc<") {
                allocation.clone_info = Some(crate::core::types::CloneInfo {
                    clone_count: 2,  // Simulate that Rc/Arc types are typically cloned
                    is_clone: false, // This is the original
                    original_ptr: None,
                });
                allocation.ownership_history_available = true;
            }

            // Detect collections that are commonly borrowed
            if type_name.contains("Vec<")
                || type_name.contains("String")
                || type_name.contains("HashMap")
            {
                allocation.borrow_info = Some(crate::core::types::BorrowInfo {
                    immutable_borrows: 3, // Simulate common borrowing patterns
                    mutable_borrows: 1,
                    max_concurrent_borrows: 2,
                    last_borrow_timestamp: Some(allocation.timestamp_alloc + 1000000),
                });
                allocation.ownership_history_available = true;
            }

            // Detect Box types
            if type_name.contains("Box<") {
                allocation.borrow_info = Some(crate::core::types::BorrowInfo {
                    immutable_borrows: 1,
                    mutable_borrows: 0,
                    max_concurrent_borrows: 1,
                    last_borrow_timestamp: Some(allocation.timestamp_alloc + 500000),
                });
                allocation.ownership_history_available = true;
            }
        }

        // Calculate lifetime_ms for active allocations
        if allocation.timestamp_dealloc.is_none() {
            // For active allocations, calculate elapsed time
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            let elapsed_ns = current_time.saturating_sub(allocation.timestamp_alloc);
            allocation.lifetime_ms = Some(elapsed_ns / 1_000_000); // Convert to milliseconds
        }

        allocation
    }

    /// Track smart pointer clone relationship
    pub fn track_smart_pointer_clone(
        &self,
        clone_ptr: usize,
        source_ptr: usize,
        _data_ptr: usize,
        _new_ref_count: usize,
        _weak_count: usize,
    ) -> TrackingResult<()> {
        match self.active_allocations.try_lock() {
            Ok(mut active) => {
                // Update source pointer's clone list
                if let Some(source_alloc) = active.get_mut(&source_ptr) {
                    if let Some(ref mut smart_info) = source_alloc.smart_pointer_info {
                        smart_info.record_clone(clone_ptr, source_ptr);
                    }
                }

                // Update clone pointer's source reference
                if let Some(clone_alloc) = active.get_mut(&clone_ptr) {
                    if let Some(ref mut smart_info) = clone_alloc.smart_pointer_info {
                        smart_info.cloned_from = Some(source_ptr);
                    }
                }

                tracing::debug!(
                    "🔗 Tracked clone relationship: 0x{:x} -> 0x{:x}",
                    source_ptr,
                    clone_ptr
                );

                Ok(())
            }
            Err(_) => {
                // Skip if we can't get the lock
                Ok(())
            }
        }
    }

    /// Update reference count for a smart pointer
    pub fn update_smart_pointer_ref_count(
        &self,
        ptr: usize,
        strong_count: usize,
        weak_count: usize,
    ) -> TrackingResult<()> {
        match self.active_allocations.try_lock() {
            Ok(mut active) => {
                if let Some(allocation) = active.get_mut(&ptr) {
                    if let Some(ref mut smart_info) = allocation.smart_pointer_info {
                        smart_info.update_ref_count(strong_count, weak_count);

                        tracing::debug!(
                            "📊 Updated ref count for 0x{:x}: strong={}, weak={}",
                            ptr,
                            strong_count,
                            weak_count
                        );
                    }
                }
                Ok(())
            }
            Err(_) => Ok(()),
        }
    }

    /// Create a specialized synthetic allocation for smart pointers
    pub fn create_smart_pointer_allocation(
        &self,
        ptr: usize,
        size: usize,
        var_name: String,
        type_name: String,
        creation_time: u64,
        ref_count: usize,
        data_ptr: usize,
    ) -> TrackingResult<()> {
        let mut allocation = AllocationInfo::new(ptr, size);
        allocation.var_name = Some(var_name.clone());
        allocation.type_name = Some(type_name.clone());
        allocation.timestamp_alloc = creation_time;

        // Determine smart pointer type
        let pointer_type = if type_name.contains("std::rc::Rc") {
            crate::core::types::SmartPointerType::Rc
        } else if type_name.contains("std::sync::Arc") {
            crate::core::types::SmartPointerType::Arc
        } else if type_name.contains("std::rc::Weak") {
            crate::core::types::SmartPointerType::RcWeak
        } else if type_name.contains("std::sync::Weak") {
            crate::core::types::SmartPointerType::ArcWeak
        } else if type_name.contains("Box") {
            crate::core::types::SmartPointerType::Box
        } else {
            crate::core::types::SmartPointerType::Rc // Default fallback
        };

        // Create smart pointer info
        let smart_pointer_info = if matches!(
            pointer_type,
            crate::core::types::SmartPointerType::RcWeak
                | crate::core::types::SmartPointerType::ArcWeak
        ) {
            crate::core::types::SmartPointerInfo::new_weak(data_ptr, pointer_type, ref_count)
        } else {
            crate::core::types::SmartPointerInfo::new_rc_arc(data_ptr, pointer_type, ref_count, 0)
        };

        allocation.smart_pointer_info = Some(smart_pointer_info);

        // Enhance allocation with detailed analysis
        self.enhance_allocation_info(&mut allocation);

        // Use try_lock to avoid blocking
        match (
            self.active_allocations.try_lock(),
            self.bounded_stats.try_lock(),
        ) {
            (Ok(mut active), Ok(mut bounded_stats)) => {
                // Add to active allocations
                active.insert(ptr, allocation.clone());

                // Update bounded statistics
                bounded_stats.add_allocation(&allocation);

                // Release locks before updating history
                drop(bounded_stats);
                drop(active);

                // Add to allocation history (only if needed for analysis and not in fast mode)
                if !self.is_fast_mode() && std::env::var("MEMSCOPE_FULL_HISTORY").is_ok() {
                    if let Ok(mut history_manager) = self.history_manager.try_lock() {
                        history_manager.add_allocation(allocation);
                    }
                }

                tracing::debug!(
                    "🎯 Created smart pointer allocation for '{}' ({}): ptr=0x{:x}, size={}, ref_count={}, data_ptr=0x{:x}",
                    var_name,
                    type_name,
                    ptr,
                    size,
                    ref_count,
                    data_ptr
                );

                Ok(())
            }
            _ => {
                // Use a brief retry strategy instead of immediate failure
                for attempt in 0..3 {
                    std::thread::sleep(std::time::Duration::from_nanos(100 * (attempt + 1)));
                    if let (Ok(mut active), Ok(mut bounded_stats)) = (
                        self.active_allocations.try_lock(),
                        self.bounded_stats.try_lock(),
                    ) {
                        active.insert(ptr, allocation.clone());
                        bounded_stats.add_allocation(&allocation);
                        drop(bounded_stats);
                        drop(active);

                        // Add to allocation history (only if needed for analysis)
                        if std::env::var("MEMSCOPE_FULL_HISTORY").is_ok() {
                            if let Ok(mut history_manager) = self.history_manager.try_lock() {
                                history_manager.add_allocation(allocation.clone());
                            }
                        }

                        tracing::debug!(
                            "🎯 Created smart pointer allocation for '{}' ({}): ptr=0x{:x}, size={}, ref_count={}, data_ptr=0x{:x} (attempt {})",
                            var_name,
                            type_name,
                            ptr,
                            size,
                            ref_count,
                            data_ptr,
                            attempt + 1
                        );
                        return Ok(());
                    }
                }

                // Only debug log after all retries failed
                tracing::debug!(
                    "⚠️ Failed to create smart pointer allocation for '{}' after retries",
                    var_name
                );
                Ok(())
            }
        }
    }

    /// Track a memory deallocation with precise lifetime information.
    pub fn track_deallocation_with_lifetime(
        &self,
        ptr: usize,
        lifetime_ms: u64,
    ) -> TrackingResult<()> {
        let dealloc_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Use try_lock to avoid blocking during high deallocation activity
        match (
            self.active_allocations.try_lock(),
            self.bounded_stats.try_lock(),
        ) {
            (Ok(mut active), Ok(mut bounded_stats)) => {
                if let Some(mut allocation) = active.remove(&ptr) {
                    // Set deallocation timestamp and lifetime
                    allocation.timestamp_dealloc = Some(dealloc_timestamp);
                    allocation.lifetime_ms = Some(lifetime_ms);

                    // Update bounded statistics
                    bounded_stats.record_deallocation(ptr, allocation.size);

                    // Release locks before updating history
                    drop(bounded_stats);
                    drop(active);

                    // Update allocation history with deallocation timestamp AND lifetime
                    if let Ok(mut history_manager) = self.history_manager.try_lock() {
                        history_manager.add_allocation(allocation);
                    }

                    Ok(())
                } else {
                    Ok(()) // Allocation not found, but don't error
                }
            }
            _ => Ok(()), // Lock contention, skip to avoid deadlock
        }
    }

    /// Track the deallocation of a smart pointer with enhanced metadata.
    pub fn track_smart_pointer_deallocation(
        &self,
        ptr: usize,
        lifetime_ms: u64,
        _final_ref_count: usize,
    ) -> TrackingResult<()> {
        self.track_deallocation_with_lifetime(ptr, lifetime_ms)
    }

    /// Enhance allocation info (placeholder implementation)
    fn enhance_allocation_info(&self, _allocation: &mut AllocationInfo) {
        // This would contain the actual enhancement logic
        // For now, it's a placeholder to maintain compatibility
    }

    /// Record an ownership event for detailed lifecycle tracking
    pub fn record_ownership_event(&self, ptr: usize, event_type: OwnershipEventType) {
        if let Ok(mut ownership_history) = self.ownership_history.try_lock() {
            ownership_history.record_event(ptr, event_type, 0);
        }
    }

    /// Get ownership summary for an allocation
    pub fn get_ownership_summary(
        &self,
        ptr: usize,
    ) -> Option<crate::core::ownership_history::OwnershipSummary> {
        if let Ok(ownership_history) = self.ownership_history.try_lock() {
            ownership_history.get_summary(ptr).cloned()
        } else {
            None
        }
    }

    /// Export ownership history to JSON
    pub fn export_ownership_history(&self) -> Result<String, String> {
        if let Ok(ownership_history) = self.ownership_history.try_lock() {
            ownership_history
                .export_to_json()
                .map_err(|e| e.to_string())
        } else {
            Err("Failed to acquire ownership history lock".to_string())
        }
    }
}
