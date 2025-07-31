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

    /// Get memory usage by type (complete implementation)
    pub fn get_memory_by_type(&self) -> TrackingResult<Vec<crate::core::types::TypeMemoryUsage>> {
        let allocations = self.get_all_active_allocations()?;
        let mut type_usage = std::collections::HashMap::new();
        
        for allocation in &allocations {
            let type_name = allocation.type_name.as_ref()
                .unwrap_or(&"Unknown".to_string())
                .clone();
            
            let entry = type_usage.entry(type_name.clone()).or_insert(crate::core::types::TypeMemoryUsage {
                type_name,
                total_size: 0,
                allocation_count: 0,
            });
            
            entry.total_size += allocation.size;
            entry.allocation_count += 1;
        }
        
        let mut result: Vec<_> = type_usage.into_values().collect();
        result.sort_by(|a, b| b.total_size.cmp(&a.total_size));
        Ok(result)
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
    pub fn export_interactive_dashboard<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        // Basic HTML dashboard implementation
        let allocations = self.get_all_active_allocations()?;
        let stats = self.get_memory_stats()?;
        
        let html_content = format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Memory Dashboard</title></head>
<body>
    <h1>Memory Tracking Dashboard</h1>
    <p>Active Allocations: {}</p>
    <p>Total Memory: {} bytes</p>
    <p>Peak Memory: {} bytes</p>
</body>
</html>"#,
            allocations.len(),
            stats.active_memory,
            stats.peak_memory
        );
        
        std::fs::write(path, html_content)
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Export memory analysis visualization
    pub fn export_memory_analysis<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        crate::export::formats::svg::export_memory_analysis(self, path)
    }

    /// Export lifecycle timeline visualization
    pub fn export_lifecycle_timeline<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        _start_time: Option<u64>,
        _end_time: Option<u64>,
    ) -> TrackingResult<()> {
        crate::export::formats::svg::export_lifecycle_timeline(self, path)
    }

    /// Export to SVG format
    pub fn export_to_svg<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        // Basic SVG export implementation
        let allocations = self.get_all_active_allocations()?;
        let svg_content = format!(
            r#"<svg width="800" height="600" xmlns="http://www.w3.org/2000/svg">
                <text x="10" y="30" font-family="Arial" font-size="16">Memory Allocations: {}</text>
                <text x="10" y="60" font-family="Arial" font-size="14">Total Active Memory: {} bytes</text>
            </svg>"#,
            allocations.len(),
            allocations.iter().map(|a| a.size).sum::<usize>()
        );
        
        std::fs::write(path, svg_content)
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Export enhanced JSON with detailed analysis
    pub fn export_enhanced_json<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        // Enhanced JSON export with analysis
        let allocations = self.get_all_active_allocations()?;
        let stats = self.get_memory_stats()?;
        let type_usage = self.get_memory_by_type()?;
        
        let enhanced_data = serde_json::json!({
            "allocations": allocations,
            "statistics": stats,
            "type_usage": type_usage,
            "export_timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });
        
        let json = serde_json::to_string_pretty(&enhanced_data)
            .map_err(|e| crate::core::types::TrackingError::SerializationError(e.to_string()))?;
        
        std::fs::write(path, json)
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Generate timeline data for visualization
    pub fn generate_timeline_data(&self, allocation_history: &[AllocationInfo], _active_allocations: &[AllocationInfo]) -> crate::core::types::SimpleTimelineData {
        let mut events = Vec::new();
        
        for allocation in allocation_history {
            events.push(crate::core::types::TimelineEvent {
                timestamp: allocation.timestamp_alloc,
                event_type: "allocation".to_string(),
                ptr: allocation.ptr,
                size: allocation.size,
                var_name: allocation.var_name.clone(),
                type_name: allocation.type_name.clone(),
            });
            
            if let Some(dealloc_time) = allocation.timestamp_dealloc {
                events.push(crate::core::types::TimelineEvent {
                    timestamp: dealloc_time,
                    event_type: "deallocation".to_string(),
                    ptr: allocation.ptr,
                    size: allocation.size,
                    var_name: allocation.var_name.clone(),
                    type_name: allocation.type_name.clone(),
                });
            }
        }
        
        events.sort_by_key(|e| e.timestamp);
        
        crate::core::types::SimpleTimelineData {
            events,
            start_time: events.first().map(|e| e.timestamp).unwrap_or(0),
            end_time: events.last().map(|e| e.timestamp).unwrap_or(0),
        }
    }
}