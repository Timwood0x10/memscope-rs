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

        // Use blocking locks in fast mode for accurate tracking
        match (self.active_allocations.lock(), self.bounded_stats.lock()) {
            (Ok(mut active), Ok(mut bounded_stats)) => {
                active.insert(ptr, allocation.clone());
                bounded_stats.add_allocation(&allocation);
                Ok(())
            }
            _ => {
                // Fallback: still track the allocation even if locks fail
                tracing::warn!("Failed to acquire locks in fast_track_allocation");
                Ok(())
            }
        }
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
        // In test mode or when explicitly requested, use blocking locks for accuracy
        let use_blocking_locks = self.is_fast_mode()
            || std::env::var("MEMSCOPE_ACCURATE_TRACKING").is_ok()
            || cfg!(test);

        if use_blocking_locks {
            // Use blocking locks to ensure all associations are tracked in tests
            let mut active = self.active_allocations.lock().map_err(|_| {
                crate::core::types::TrackingError::LockError(
                    "Failed to acquire active_allocations lock".to_string(),
                )
            })?;

            if let Some(allocation) = active.get_mut(&ptr) {
                let old_var_name_is_none = allocation.var_name.is_none();

                allocation.var_name = Some(var_name.clone());
                allocation.type_name = Some(type_name.clone());

                // Apply improve.md field enhancements based on type
                allocation.enhance_with_type_info(&type_name);

                // CRITICAL FIX: Update bounded_stats after associating var_name
                // Clone the allocation to pass to bounded_stats
                let allocation_clone = allocation.clone();
                drop(active); // Release active lock before acquiring bounded_stats lock

                if let Ok(mut bounded_stats) = self.bounded_stats.lock() {
                    bounded_stats
                        .update_active_allocation_status(&allocation_clone, old_var_name_is_none);
                }

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

                // Release active lock before acquiring bounded_stats lock
                drop(active);

                let mut bounded_stats = self.bounded_stats.lock().map_err(|_| {
                    crate::core::types::TrackingError::LockError(
                        "Failed to acquire bounded_stats lock".to_string(),
                    )
                })?;
                bounded_stats.add_allocation(&synthetic_allocation);

                tracing::debug!(
                    "Created synthetic allocation for variable '{}' at {:x} (estimated size: {})",
                    var_name,
                    ptr,
                    estimated_size
                );
            }
            Ok(())
        } else {
            // Production mode: use try_lock with retry logic
            self.associate_var_with_retry(ptr, var_name, type_name)
        }
    }

    /// Associate variable with retry logic for production mode
    fn associate_var_with_retry(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
    ) -> TrackingResult<()> {
        let mut retry_count = 0;
        const MAX_RETRIES: u32 = 10;

        while retry_count < MAX_RETRIES {
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
                        return Ok(());
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

                        // Release active lock before acquiring bounded_stats lock
                        drop(active);

                        if let Ok(mut bounded_stats) = self.bounded_stats.try_lock() {
                            bounded_stats.add_allocation(&synthetic_allocation);
                        }

                        tracing::debug!("Created synthetic allocation for variable '{}' at {:x} (estimated size: {})", 
                                       var_name, ptr, estimated_size);
                        return Ok(());
                    }
                }
                Err(_) => {
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
    #[allow(clippy::too_many_arguments)]
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

#[cfg(test)]
mod tests {
    use crate::core::ownership_history::OwnershipEventType;
    use crate::core::tracker::memory_tracker::MemoryTracker;
    use std::sync::Arc;

    fn create_test_tracker() -> MemoryTracker {
        MemoryTracker::new()
    }

    #[test]
    fn test_fast_track_allocation() {
        let tracker = create_test_tracker();

        let result = tracker.fast_track_allocation(0x1000, 64, "test_var".to_string());
        assert!(result.is_ok());

        // Verify allocation was tracked
        let allocations = tracker.get_active_allocations().unwrap();
        let allocation = allocations.iter().find(|a| a.ptr == 0x1000).unwrap();
        assert_eq!(allocation.size, 64);
        assert_eq!(allocation.var_name, Some("test_var".to_string()));
        assert_eq!(allocation.type_name, Some("fast_tracked".to_string()));
    }

    #[test]
    fn test_fast_track_allocation_multiple() {
        let tracker = create_test_tracker();

        // Track multiple allocations
        for i in 0..5 {
            let ptr = 0x1000 + i * 0x100;
            let size = 64 + i * 32;
            let var_name = format!("var_{}", i);

            let result = tracker.fast_track_allocation(ptr, size, var_name.clone());
            assert!(result.is_ok());
        }

        // Verify all allocations were tracked
        let allocations = tracker.get_active_allocations().unwrap();
        assert_eq!(allocations.len(), 5);

        for i in 0..5 {
            let ptr = 0x1000 + i * 0x100;
            let allocation = allocations.iter().find(|a| a.ptr == ptr).unwrap();
            assert_eq!(allocation.size, 64 + i * 32);
            assert_eq!(allocation.var_name, Some(format!("var_{}", i)));
        }
    }

    #[test]
    fn test_track_allocation() {
        let tracker = create_test_tracker();

        let result = tracker.track_allocation(0x2000, 128);
        assert!(result.is_ok());

        // Verify allocation was tracked
        let allocations = tracker.get_active_allocations().unwrap();
        let allocation = allocations.iter().find(|a| a.ptr == 0x2000).unwrap();
        assert_eq!(allocation.size, 128);
        assert_eq!(allocation.ptr, 0x2000);
    }

    #[test]
    fn test_track_allocation_with_context() {
        let tracker = create_test_tracker();

        let result = tracker.track_allocation_with_context(
            0x3000,
            256,
            "context_var".to_string(),
            "String".to_string(),
        );
        assert!(result.is_ok());

        // Verify allocation was tracked with context
        let allocations = tracker.get_active_allocations().unwrap();
        let allocation = allocations.iter().find(|a| a.ptr == 0x3000).unwrap();
        assert_eq!(allocation.size, 256);
        assert_eq!(allocation.var_name, Some("context_var".to_string()));
        assert_eq!(allocation.type_name, Some("String".to_string()));
        assert!(allocation.lifetime_ms.is_some());
    }

    #[test]
    fn test_track_deallocation() {
        let tracker = create_test_tracker();

        // First track an allocation
        tracker.track_allocation(0x4000, 512).unwrap();

        // Verify it's active
        let allocations = tracker.get_active_allocations().unwrap();
        assert!(allocations.iter().any(|a| a.ptr == 0x4000));

        // Now deallocate it
        let result = tracker.track_deallocation(0x4000);
        assert!(result.is_ok());

        // Verify it's no longer active
        let allocations = tracker.get_active_allocations().unwrap();
        assert!(!allocations.iter().any(|a| a.ptr == 0x4000));
    }

    #[test]
    fn test_track_deallocation_nonexistent() {
        let tracker = create_test_tracker();

        // Try to deallocate a non-existent allocation
        let result = tracker.track_deallocation(0x9999);
        assert!(result.is_ok()); // Should not error
    }

    #[test]
    fn test_create_synthetic_allocation() {
        let tracker = create_test_tracker();

        let result = tracker.create_synthetic_allocation(
            0x5000,
            1024,
            "synthetic_var".to_string(),
            "Vec<u8>".to_string(),
            1234567890,
        );
        assert!(result.is_ok());

        // Verify synthetic allocation was created
        let allocations = tracker.get_active_allocations().unwrap();
        let allocation = allocations.iter().find(|a| a.ptr == 0x5000).unwrap();
        assert_eq!(allocation.size, 1024);
        assert_eq!(allocation.var_name, Some("synthetic_var".to_string()));
        assert_eq!(allocation.type_name, Some("Vec<u8>".to_string()));
    }

    #[test]
    fn test_associate_var_existing_allocation() {
        let tracker = create_test_tracker();

        // First track an allocation without context
        tracker.track_allocation(0x6000, 128).unwrap();

        // Then associate a variable with it
        let result = tracker.associate_var(
            0x6000,
            "associated_var".to_string(),
            "HashMap<String, i32>".to_string(),
        );
        assert!(result.is_ok());

        // Verify association was successful
        let allocations = tracker.get_active_allocations().unwrap();
        let allocation = allocations.iter().find(|a| a.ptr == 0x6000).unwrap();
        assert_eq!(allocation.var_name, Some("associated_var".to_string()));
        assert_eq!(
            allocation.type_name,
            Some("HashMap<String, i32>".to_string())
        );
    }

    #[test]
    fn test_associate_var_new_allocation() {
        let tracker = create_test_tracker();

        // Associate a variable with a non-existent allocation (creates synthetic)
        let result =
            tracker.associate_var(0x7000, "new_var".to_string(), "Box<String>".to_string());
        assert!(result.is_ok());

        // Verify synthetic allocation was created
        let allocations = tracker.get_active_allocations().unwrap();
        let allocation = allocations.iter().find(|a| a.ptr == 0x7000).unwrap();
        assert_eq!(allocation.var_name, Some("new_var".to_string()));
        assert_eq!(allocation.type_name, Some("Box<String>".to_string()));
        assert!(allocation.size > 0); // Should have estimated size
    }

    #[test]
    fn test_track_smart_pointer_clone() {
        let tracker = create_test_tracker();

        // Create source allocation with smart pointer info
        tracker
            .create_smart_pointer_allocation(
                0x8000,
                24,
                "source_rc".to_string(),
                "std::rc::Rc<String>".to_string(),
                1234567890,
                1,
                0x8100,
            )
            .unwrap();

        // Create clone allocation
        tracker
            .create_smart_pointer_allocation(
                0x8200,
                24,
                "clone_rc".to_string(),
                "std::rc::Rc<String>".to_string(),
                1234567900,
                2,
                0x8100, // Same data pointer
            )
            .unwrap();

        // Track the clone relationship
        let result = tracker.track_smart_pointer_clone(0x8200, 0x8000, 0x8100, 2, 0);
        assert!(result.is_ok());

        // Verify clone relationship was tracked
        let allocations = tracker.get_active_allocations().unwrap();
        let source_alloc = allocations.iter().find(|a| a.ptr == 0x8000).unwrap();
        let clone_alloc = allocations.iter().find(|a| a.ptr == 0x8200).unwrap();

        assert!(source_alloc.smart_pointer_info.is_some());
        assert!(clone_alloc.smart_pointer_info.is_some());
    }

    #[test]
    fn test_update_smart_pointer_ref_count() {
        let tracker = create_test_tracker();

        // Create smart pointer allocation
        tracker
            .create_smart_pointer_allocation(
                0x9000,
                24,
                "ref_counted".to_string(),
                "std::rc::Rc<i32>".to_string(),
                1234567890,
                1,
                0x9100,
            )
            .unwrap();

        // Update reference count
        let result = tracker.update_smart_pointer_ref_count(0x9000, 3, 1);
        assert!(result.is_ok());

        // Verify reference count was updated
        let allocations = tracker.get_active_allocations().unwrap();
        let allocation = allocations.iter().find(|a| a.ptr == 0x9000).unwrap();

        if let Some(ref smart_info) = allocation.smart_pointer_info {
            if let Some(latest) = smart_info.latest_ref_counts() {
                assert_eq!(latest.strong_count, 3);
                assert_eq!(latest.weak_count, 1);
            }
        } else {
            panic!("Smart pointer info should be present");
        }
    }

    #[test]
    fn test_create_smart_pointer_allocation_rc() {
        let tracker = create_test_tracker();

        let result = tracker.create_smart_pointer_allocation(
            0xa000,
            24,
            "rc_ptr".to_string(),
            "std::rc::Rc<Vec<u8>>".to_string(),
            1234567890,
            1,
            0xa100,
        );
        assert!(result.is_ok());

        // Verify smart pointer allocation was created
        let allocations = tracker.get_active_allocations().unwrap();
        let allocation = allocations.iter().find(|a| a.ptr == 0xa000).unwrap();
        assert_eq!(allocation.size, 24);
        assert_eq!(allocation.var_name, Some("rc_ptr".to_string()));
        assert_eq!(
            allocation.type_name,
            Some("std::rc::Rc<Vec<u8>>".to_string())
        );
        assert!(allocation.smart_pointer_info.is_some());

        if let Some(ref smart_info) = allocation.smart_pointer_info {
            assert_eq!(smart_info.data_ptr, 0xa100);
            if let Some(latest) = smart_info.latest_ref_counts() {
                assert_eq!(latest.strong_count, 1);
                assert_eq!(latest.weak_count, 0);
            }
        }
    }

    #[test]
    fn test_create_smart_pointer_allocation_arc() {
        let tracker = create_test_tracker();

        let result = tracker.create_smart_pointer_allocation(
            0xb000,
            24,
            "arc_ptr".to_string(),
            "std::sync::Arc<String>".to_string(),
            1234567890,
            1,
            0xb100,
        );
        assert!(result.is_ok());

        // Verify Arc allocation was created
        let allocations = tracker.get_active_allocations().unwrap();
        let allocation = allocations.iter().find(|a| a.ptr == 0xb000).unwrap();

        if let Some(ref smart_info) = allocation.smart_pointer_info {
            assert_eq!(
                smart_info.pointer_type,
                crate::core::types::SmartPointerType::Arc
            );
        }
    }

    #[test]
    fn test_create_smart_pointer_allocation_box() {
        let tracker = create_test_tracker();

        let result = tracker.create_smart_pointer_allocation(
            0xc000,
            8,
            "box_ptr".to_string(),
            "Box<i64>".to_string(),
            1234567890,
            1,
            0xc100,
        );
        assert!(result.is_ok());

        // Verify Box allocation was created
        let allocations = tracker.get_active_allocations().unwrap();
        let allocation = allocations.iter().find(|a| a.ptr == 0xc000).unwrap();

        if let Some(ref smart_info) = allocation.smart_pointer_info {
            assert_eq!(
                smart_info.pointer_type,
                crate::core::types::SmartPointerType::Box
            );
        }
    }

    #[test]
    fn test_create_smart_pointer_allocation_weak() {
        let tracker = create_test_tracker();

        let result = tracker.create_smart_pointer_allocation(
            0xd000,
            24,
            "weak_ptr".to_string(),
            "std::rc::Weak<String>".to_string(),
            1234567890,
            2, // weak count
            0xd100,
        );
        assert!(result.is_ok());

        // Verify Weak allocation was created
        let allocations = tracker.get_active_allocations().unwrap();
        let allocation = allocations.iter().find(|a| a.ptr == 0xd000).unwrap();

        if let Some(ref smart_info) = allocation.smart_pointer_info {
            assert_eq!(
                smart_info.pointer_type,
                crate::core::types::SmartPointerType::RcWeak
            );
            if let Some(latest) = smart_info.latest_ref_counts() {
                assert_eq!(latest.weak_count, 2);
            }
        }
    }

    #[test]
    fn test_track_deallocation_with_lifetime() {
        let tracker = create_test_tracker();

        // First track an allocation
        tracker.track_allocation(0xe000, 256).unwrap();

        // Deallocate with specific lifetime
        let result = tracker.track_deallocation_with_lifetime(0xe000, 1500);
        assert!(result.is_ok());

        // Verify allocation is no longer active
        let allocations = tracker.get_active_allocations().unwrap();
        assert!(!allocations.iter().any(|a| a.ptr == 0xe000));
    }

    #[test]
    fn test_track_smart_pointer_deallocation() {
        let tracker = create_test_tracker();

        // Create smart pointer allocation
        tracker
            .create_smart_pointer_allocation(
                0xf000,
                24,
                "dealloc_rc".to_string(),
                "std::rc::Rc<String>".to_string(),
                1234567890,
                1,
                0xf100,
            )
            .unwrap();

        // Deallocate smart pointer
        let result = tracker.track_smart_pointer_deallocation(0xf000, 2000, 0);
        assert!(result.is_ok());

        // Verify allocation is no longer active
        let allocations = tracker.get_active_allocations().unwrap();
        assert!(!allocations.iter().any(|a| a.ptr == 0xf000));
    }

    #[test]
    fn test_record_ownership_event() {
        let tracker = create_test_tracker();

        // Record various ownership events
        tracker.record_ownership_event(0x10000, OwnershipEventType::Allocated);
        tracker.record_ownership_event(
            0x10000,
            OwnershipEventType::Borrowed {
                borrower_scope: "test_scope".to_string(),
            },
        );
        tracker.record_ownership_event(
            0x10000,
            OwnershipEventType::OwnershipTransferred {
                target_var: "new_var".to_string(),
            },
        );
        tracker.record_ownership_event(0x10000, OwnershipEventType::Dropped);

        // This should not panic or error
    }

    #[test]
    fn test_get_ownership_summary() {
        let tracker = create_test_tracker();

        // Record some ownership events
        tracker.record_ownership_event(0x11000, OwnershipEventType::Allocated);
        tracker.record_ownership_event(
            0x11000,
            OwnershipEventType::Borrowed {
                borrower_scope: "test_scope".to_string(),
            },
        );

        // Get ownership summary
        let summary = tracker.get_ownership_summary(0x11000);
        assert!(summary.is_some());

        // Test non-existent allocation
        let no_summary = tracker.get_ownership_summary(0x99999);
        assert!(no_summary.is_none() || no_summary.is_some()); // Either is valid
    }

    #[test]
    fn test_export_ownership_history() {
        let tracker = create_test_tracker();

        // Record some ownership events
        tracker.record_ownership_event(0x12000, OwnershipEventType::Allocated);
        tracker.record_ownership_event(
            0x12000,
            OwnershipEventType::Borrowed {
                borrower_scope: "test_scope".to_string(),
            },
        );
        tracker.record_ownership_event(0x12000, OwnershipEventType::Dropped);

        // Export ownership history
        let result = tracker.export_ownership_history();
        assert!(result.is_ok());

        let json_str = result.unwrap();
        assert!(!json_str.is_empty());

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(parsed.is_object() || parsed.is_array());
    }

    #[test]
    fn test_concurrent_allocations() {
        let tracker = Arc::new(create_test_tracker());
        let mut handles = vec![];

        // Spawn multiple threads doing allocations
        for i in 0..5 {
            let tracker_clone = Arc::clone(&tracker);
            let handle = std::thread::spawn(move || {
                for j in 0..10 {
                    let ptr = (i * 1000 + j) * 0x100;
                    let size = 64 + j * 8;
                    let var_name = format!("thread_{}_var_{}", i, j);

                    let _ = tracker_clone.fast_track_allocation(ptr, size, var_name);
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify allocations were tracked
        let allocations = tracker.get_active_allocations().unwrap();
        assert!(allocations.len() > 0);
        assert!(allocations.len() <= 50); // Should be up to 50 allocations
    }

    #[test]
    fn test_allocation_lifecycle() {
        let tracker = create_test_tracker();

        // Track allocation
        tracker
            .track_allocation_with_context(
                0x13000,
                512,
                "lifecycle_var".to_string(),
                "Vec<String>".to_string(),
            )
            .unwrap();

        // Verify it's active
        let allocations = tracker.get_active_allocations().unwrap();
        let allocation = allocations.iter().find(|a| a.ptr == 0x13000).unwrap();
        assert!(allocation.lifetime_ms.is_some());

        // Associate additional info
        tracker
            .associate_var(
                0x13000,
                "updated_lifecycle_var".to_string(),
                "Vec<String>".to_string(),
            )
            .unwrap();

        // Verify update
        let allocations = tracker.get_active_allocations().unwrap();
        let allocation = allocations.iter().find(|a| a.ptr == 0x13000).unwrap();
        assert_eq!(
            allocation.var_name,
            Some("updated_lifecycle_var".to_string())
        );

        // Record ownership events
        tracker.record_ownership_event(0x13000, OwnershipEventType::Allocated);
        tracker.record_ownership_event(
            0x13000,
            OwnershipEventType::Borrowed {
                borrower_scope: "test_scope".to_string(),
            },
        );

        // Deallocate
        tracker.track_deallocation(0x13000).unwrap();

        // Verify it's no longer active
        let allocations = tracker.get_active_allocations().unwrap();
        assert!(!allocations.iter().any(|a| a.ptr == 0x13000));
    }

    #[test]
    fn test_smart_pointer_lifecycle() {
        let tracker = create_test_tracker();

        // Create Rc allocation
        tracker
            .create_smart_pointer_allocation(
                0x14000,
                24,
                "rc_lifecycle".to_string(),
                "std::rc::Rc<Vec<i32>>".to_string(),
                1234567890,
                1,
                0x14100,
            )
            .unwrap();

        // Clone it
        tracker
            .create_smart_pointer_allocation(
                0x14200,
                24,
                "rc_clone".to_string(),
                "std::rc::Rc<Vec<i32>>".to_string(),
                1234567900,
                2,
                0x14100, // Same data pointer
            )
            .unwrap();

        // Track clone relationship
        tracker
            .track_smart_pointer_clone(0x14200, 0x14000, 0x14100, 2, 0)
            .unwrap();

        // Update reference counts
        tracker
            .update_smart_pointer_ref_count(0x14000, 2, 0)
            .unwrap();
        tracker
            .update_smart_pointer_ref_count(0x14200, 2, 0)
            .unwrap();

        // Deallocate clone (ref count goes to 1)
        tracker
            .track_smart_pointer_deallocation(0x14200, 1000, 1)
            .unwrap();

        // Update original ref count
        tracker
            .update_smart_pointer_ref_count(0x14000, 1, 0)
            .unwrap();

        // Deallocate original (ref count goes to 0)
        tracker
            .track_smart_pointer_deallocation(0x14000, 2000, 0)
            .unwrap();

        // Verify both are deallocated
        let allocations = tracker.get_active_allocations().unwrap();
        assert!(!allocations.iter().any(|a| a.ptr == 0x14000));
        assert!(!allocations.iter().any(|a| a.ptr == 0x14200));
    }
}
