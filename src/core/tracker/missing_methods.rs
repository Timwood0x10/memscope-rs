//! Missing methods that need to be added back to MemoryTracker

use crate::core::tracker::memory_tracker::MemoryTracker;
use crate::core::types::{AllocationInfo, TrackingResult};

impl MemoryTracker {
    /// Export memory tracking data to JSON format (legacy method)
    pub fn export_to_json<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        // Simple JSON export implementation
        let allocations = self.get_all_active_allocations()?;
        let json = serde_json::to_string_pretty(&allocations)
            .map_err(|e| crate::core::types::TrackingError::SerializationError(e.to_string()))?;
        
        std::fs::write(path, json)
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;
        
        Ok(())
    }

    /// Export memory tracking data to JSON with options (legacy method)
    pub fn export_to_json_with_options<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        _options: crate::core::tracker::ExportOptions,
    ) -> TrackingResult<()> {
        // For now, just delegate to the basic export
        // TODO: Implement options handling
        self.export_to_json(path)
    }

    /// Get active allocations (legacy method name)
    pub fn get_active_allocations(&self) -> TrackingResult<Vec<AllocationInfo>> {
        self.get_all_active_allocations()
    }

    /// Create synthetic allocation for testing purposes
    pub fn create_synthetic_allocation(
        &self,
        ptr: usize,
        size: usize,
        var_name: String,
        type_name: String,
        _line_number: u32,
    ) -> TrackingResult<()> {
        let mut allocation = AllocationInfo::new(ptr, size);
        allocation.var_name = Some(var_name);
        allocation.type_name = Some(type_name);

        // Update active allocations
        if let Ok(mut active) = self.active_allocations.try_lock() {
            active.insert(ptr, allocation.clone());
        }

        // Update statistics
        if let Ok(mut stats) = self.stats.try_lock() {
            stats.total_allocations = stats.total_allocations.saturating_add(1);
            stats.active_allocations = stats.active_allocations.saturating_add(1);
            stats.active_memory = stats.active_memory.saturating_add(size);
            if stats.active_memory > stats.peak_memory {
                stats.peak_memory = stats.active_memory;
            }
        }

        // Add to history
        if let Ok(mut history) = self.allocation_history.try_lock() {
            history.push(allocation);
        }

        Ok(())
    }

    /// Get memory statistics (legacy method name)
    pub fn get_stats(&self) -> TrackingResult<crate::core::types::MemoryStats> {
        self.get_memory_stats()
    }

    /// Get memory usage by type (placeholder implementation)
    pub fn get_memory_by_type(&self) -> std::collections::HashMap<String, usize> {
        let allocations = self.get_all_active_allocations().unwrap_or_default();
        let mut by_type = std::collections::HashMap::new();
        
        for allocation in allocations {
            if let Some(type_name) = &allocation.type_name {
                *by_type.entry(type_name.clone()).or_insert(0) += allocation.size;
            }
        }
        
        by_type
    }

    /// Get allocation history (legacy method name)
    pub fn get_allocation_history(&self) -> TrackingResult<Vec<AllocationInfo>> {
        self.get_complete_allocation_history()
    }

    /// Track smart pointer clone (placeholder)
    pub fn track_smart_pointer_clone(&self, _ptr: usize, _size: usize, _type_name: String, _strong_count: usize, _weak_count: usize) -> TrackingResult<()> {
        // TODO: Implement smart pointer tracking
        Ok(())
    }

    /// Update smart pointer ref count (placeholder)
    pub fn update_smart_pointer_ref_count(&self, _ptr: usize, _strong_count: usize, _weak_count: usize) -> TrackingResult<()> {
        // TODO: Implement ref count tracking
        Ok(())
    }

    /// Create smart pointer allocation (placeholder)
    pub fn create_smart_pointer_allocation(&self, _ptr: usize, _size: usize, _type_name: String, _strong_count: usize, _weak_count: usize, _creation_time: u64) -> TrackingResult<()> {
        // TODO: Implement smart pointer allocation
        Ok(())
    }

    /// Associate variable (legacy method name)
    pub fn associate_var(&self, ptr: usize, var_name: String, type_name: String) -> TrackingResult<()> {
        self.associate_variable(ptr, var_name, type_name)
    }

    /// Track deallocation with lifetime (placeholder)
    pub fn track_deallocation_with_lifetime(&self, ptr: usize, _lifetime_ms: u64) -> TrackingResult<()> {
        self.track_deallocation(ptr)
    }

    /// Track smart pointer deallocation (placeholder)
    pub fn track_smart_pointer_deallocation(&self, ptr: usize, _lifetime_ms: u64, _final_ref_count: usize) -> TrackingResult<()> {
        self.track_deallocation(ptr)
    }

    /// Export interactive dashboard (placeholder)
    pub fn export_interactive_dashboard(&self, _path: &std::path::Path) -> TrackingResult<()> {
        // TODO: Implement interactive dashboard export
        Ok(())
    }
}