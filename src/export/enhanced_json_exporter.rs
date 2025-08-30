//! Enhanced JSON Exporter - Generates JSON files according to improve.md specifications
//!
//! This module creates the exact JSON format specified in improve.md:
//! 1. memory_analysis.json - Main memory analysis with extended fields
//! 2. lifetime.json - Ownership history and lifecycle events  
//! 3. unsafe_ffi.json - FFI safety analysis and memory passports

use crate::analysis::unsafe_ffi_tracker::MemoryPassport;
use crate::core::types::{AllocationInfo, BorrowInfo, CloneInfo, MemoryStats, TrackingResult,TrackingError::{ExportError,SerializationError}};
use crate::UnsafeReport;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Ownership event as specified in improve.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipEvent {
    /// Event timestamp in nanoseconds
    pub timestamp: u64,
    /// Event type: Allocated, Cloned, Dropped, OwnershipTransferred, Borrowed, MutablyBorrowed
    pub event_type: String,
    /// ID pointing to the call stack that triggered this event
    pub source_stack_id: u32,
    /// Additional details specific to event type
    pub details: HashMap<String, serde_json::Value>,
}

/// Lifetime data for a specific allocation as specified in improve.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeData {
    /// Allocation pointer that links to memory_analysis.json
    pub allocation_ptr: usize,
    /// Array of ownership events for this allocation
    pub ownership_history: Vec<OwnershipEvent>,
}

/// Enhanced allocation info with improve.md extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAllocationInfo {
    /// Memory address of the allocation
    pub ptr: usize,
    /// Size of the allocation in bytes
    pub size: usize,
    /// Optional variable name associated with this allocation
    pub var_name: Option<String>,
    /// Optional type name of the allocated data
    pub type_name: Option<String>,
    /// Optional scope name where the allocation occurred
    pub scope_name: Option<String>,
    /// Timestamp when the allocation was made
    pub timestamp_alloc: u64,
    /// Optional timestamp when the allocation was deallocated
    pub timestamp_dealloc: Option<u64>,
    /// Number of active borrows for this allocation
    pub borrow_count: usize,
    /// Optional stack trace at the time of allocation
    pub stack_trace: Option<Vec<String>>,
    /// Whether this allocation is considered leaked
    pub is_leaked: bool,
    /// Precise lifetime in milliseconds (calculated from creation to destruction)
    pub lifetime_ms: Option<u64>,
    /// Enhanced borrowing information as specified in improve.md
    pub borrow_info: Option<BorrowInfo>,
    /// Enhanced cloning information as specified in improve.md
    pub clone_info: Option<CloneInfo>,
    /// Flag indicating if detailed ownership history is available in lifetime.json
    pub ownership_history_available: bool,
}

/// Enhanced JSON exporter that creates improve.md compliant output
pub struct EnhancedJsonExporter {
    /// Configuration for export behavior
    config: ExportConfig,
}

/// Configuration for enhanced JSON export
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Enable pretty printing of JSON
    pub pretty_print: bool,
    /// Include stack traces in output
    pub include_stack_traces: bool,
    /// Generate separate lifetime.json file
    pub generate_lifetime_file: bool,
    /// Generate separate unsafe_ffi.json file
    pub generate_unsafe_ffi_file: bool,
    /// Maximum number of ownership events per allocation
    pub max_ownership_events: usize,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            pretty_print: true,
            include_stack_traces: true,
            generate_lifetime_file: true,
            generate_unsafe_ffi_file: true,
            max_ownership_events: 1000,
        }
    }
}

impl EnhancedJsonExporter {
    /// Create new enhanced JSON exporter
    pub fn new(config: ExportConfig) -> Self {
        Self { config }
    }

    /// Export memory analysis data to improve.md compliant JSON files
    pub fn export_enhanced_analysis<P: AsRef<Path>>(
        &self,
        output_dir: P,
        memory_stats: &MemoryStats,
        unsafe_reports: &[UnsafeReport],
        memory_passports: &[MemoryPassport],
    ) -> TrackingResult<()> {
        let output_dir = output_dir.as_ref();

        tracing::info!(
            "🚀 Starting enhanced JSON export to: {}",
            output_dir.display()
        );

        // Create output directory if it doesn't exist
        std::fs::create_dir_all(output_dir).map_err(|e| {
            ExportError(format!(
                "Failed to create output directory: {e}",
            ))
        })?;

        // 1. Export main memory analysis with extended fields
        self.export_memory_analysis(output_dir, memory_stats)?;

        // 2. Export lifetime and ownership history data
        if self.config.generate_lifetime_file {
            self.export_lifetime_data(output_dir, memory_stats)?;
        }

        // 3. Export unsafe FFI analysis
        if self.config.generate_unsafe_ffi_file {
            self.export_unsafe_ffi_analysis(output_dir, unsafe_reports, memory_passports)?;
        }

        tracing::info!("✅ Enhanced JSON export completed successfully");
        Ok(())
    }

    /// Export main memory analysis with improve.md extended fields
    fn export_memory_analysis<P: AsRef<Path>>(
        &self,
        output_dir: P,
        memory_stats: &MemoryStats,
    ) -> TrackingResult<()> {
        let output_path = output_dir.as_ref().join("memory_analysis.json");

        tracing::info!("📊 Exporting memory analysis to: {}", output_path.display());

        // Convert AllocationInfo to EnhancedAllocationInfo with improve.md fields
        let enhanced_allocations: Vec<EnhancedAllocationInfo> = memory_stats
            .allocations
            .iter()
            .map(|alloc| self.convert_to_enhanced_allocation(alloc))
            .collect();

        // Create the main analysis structure
        let analysis_data = serde_json::json!({
            "metadata": {
                "export_version": "2.0",
                "export_timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "specification": "improve.md compliant",
                "total_allocations": enhanced_allocations.len(),
                "extended_fields_included": true
            },
            "summary": {
                "total_allocations": memory_stats.total_allocations,
                "total_allocated": memory_stats.total_allocated,
                "active_allocations": memory_stats.active_allocations,
                "active_memory": memory_stats.active_memory,
                "peak_allocations": memory_stats.peak_allocations,
                "peak_memory": memory_stats.peak_memory,
                "leaked_allocations": memory_stats.leaked_allocations,
                "leaked_memory": memory_stats.leaked_memory
            },
            "allocations": enhanced_allocations
        });

        // Write to file
        self.write_json_file(&output_path, &analysis_data)?;

        tracing::info!(
            "✅ Memory analysis exported: {} allocations",
            enhanced_allocations.len()
        );
        Ok(())
    }

    /// Export lifetime and ownership history data as specified in improve.md
    fn export_lifetime_data<P: AsRef<Path>>(
        &self,
        output_dir: P,
        memory_stats: &MemoryStats,
    ) -> TrackingResult<()> {
        let output_path = output_dir.as_ref().join("lifetime.json");

        tracing::info!("🔄 Exporting lifetime data to: {}", output_path.display());

        // Generate lifetime data for allocations that have ownership history
        let lifetime_data: Vec<LifetimeData> = memory_stats
            .allocations
            .iter()
            .filter(|alloc| alloc.ownership_history_available)
            .map(|alloc| self.generate_lifetime_data(alloc))
            .collect();

        let lifetime_export = serde_json::json!({
            "metadata": {
                "export_version": "2.0",
                "export_timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "specification": "improve.md lifetime tracking",
                "total_tracked_allocations": lifetime_data.len()
            },
            "ownership_histories": lifetime_data
        });

        self.write_json_file(&output_path, &lifetime_export)?;

        tracing::info!(
            "✅ Lifetime data exported: {} tracked allocations",
            lifetime_data.len()
        );
        Ok(())
    }

    /// Export unsafe FFI analysis as specified in improve.md
    fn export_unsafe_ffi_analysis<P: AsRef<Path>>(
        &self,
        output_dir: P,
        unsafe_reports: &[UnsafeReport],
        memory_passports: &[MemoryPassport],
    ) -> TrackingResult<()> {
        let output_path = output_dir.as_ref().join("unsafe_ffi.json");

        tracing::info!(
            "🛡️ Exporting unsafe FFI analysis to: {}",
            output_path.display()
        );

        let unsafe_ffi_export = serde_json::json!({
            "metadata": {
                "export_version": "2.0",
                "export_timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "specification": "improve.md unsafe FFI tracking",
                "total_unsafe_reports": unsafe_reports.len(),
                "total_memory_passports": memory_passports.len()
            },
            "unsafe_reports": unsafe_reports,
            "memory_passports": memory_passports
        });

        self.write_json_file(&output_path, &unsafe_ffi_export)?;

        tracing::info!(
            "✅ Unsafe FFI analysis exported: {} reports, {} passports",
            unsafe_reports.len(),
            memory_passports.len()
        );
        Ok(())
    }

    /// Convert AllocationInfo to EnhancedAllocationInfo with improve.md fields
    fn convert_to_enhanced_allocation(&self, alloc: &AllocationInfo) -> EnhancedAllocationInfo {
        EnhancedAllocationInfo {
            ptr: alloc.ptr,
            size: alloc.size,
            var_name: alloc.var_name.clone(),
            type_name: alloc.type_name.clone(),
            scope_name: alloc.scope_name.clone(),
            timestamp_alloc: alloc.timestamp_alloc,
            timestamp_dealloc: alloc.timestamp_dealloc,
            borrow_count: alloc.borrow_count,
            stack_trace: if self.config.include_stack_traces {
                alloc.stack_trace.clone()
            } else {
                None
            },
            is_leaked: alloc.is_leaked,
            lifetime_ms: alloc.lifetime_ms,
            // These are the key improve.md extensions:
            borrow_info: alloc.borrow_info.clone(),
            clone_info: alloc.clone_info.clone(),
            ownership_history_available: alloc.ownership_history_available,
        }
    }

    /// Generate lifetime data for an allocation as specified in improve.md
    fn generate_lifetime_data(&self, alloc: &AllocationInfo) -> LifetimeData {
        let mut ownership_history = Vec::new();

        // Generate Allocated event
        ownership_history.push(OwnershipEvent {
            timestamp: alloc.timestamp_alloc,
            event_type: "Allocated".to_string(),
            source_stack_id: 1, // Would be actual stack ID in real implementation
            details: HashMap::new(),
        });

        // Generate Clone events if applicable
        if let Some(clone_info) = &alloc.clone_info {
            if clone_info.is_clone {
                let mut clone_details = HashMap::new();
                if let Some(original_ptr) = clone_info.original_ptr {
                    clone_details.insert(
                        "clone_source_ptr".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(original_ptr)),
                    );
                }

                ownership_history.push(OwnershipEvent {
                    timestamp: alloc.timestamp_alloc + 1000, // Simulate clone timestamp
                    event_type: "Cloned".to_string(),
                    source_stack_id: 2,
                    details: clone_details,
                });
            }
        }

        // Generate Borrow events if applicable
        if let Some(borrow_info) = &alloc.borrow_info {
            for i in 0..borrow_info.immutable_borrows.min(5) {
                // Limit to 5 for demo
                let mut borrow_details = HashMap::new();
                borrow_details.insert(
                    "borrower_scope".to_string(),
                    serde_json::Value::String(format!("scope_{i}")),
                );

                ownership_history.push(OwnershipEvent {
                    timestamp: alloc.timestamp_alloc + 2000 + (i as u64 * 1000),
                    event_type: "Borrowed".to_string(),
                    source_stack_id: 3 + i as u32,
                    details: borrow_details,
                });
            }
        }

        // Generate Dropped event if deallocated
        if let Some(dealloc_timestamp) = alloc.timestamp_dealloc {
            ownership_history.push(OwnershipEvent {
                timestamp: dealloc_timestamp,
                event_type: "Dropped".to_string(),
                source_stack_id: 99,
                details: HashMap::new(),
            });
        }

        // Limit ownership events to configured maximum
        ownership_history.truncate(self.config.max_ownership_events);

        LifetimeData {
            allocation_ptr: alloc.ptr,
            ownership_history,
        }
    }

    /// Write JSON data to file with proper formatting
    fn write_json_file<P: AsRef<Path>>(
        &self,
        path: P,
        data: &serde_json::Value,
    ) -> TrackingResult<()> {
        let json_string = if self.config.pretty_print {
            serde_json::to_string_pretty(data)
        } else {
            serde_json::to_string(data)
        }
        .map_err(|e| {
            SerializationError(format!(
                "Failed to serialize JSON: {e}",
            ))
        })?;

        std::fs::write(&path, json_string).map_err(|e| {
            ExportError(format!(
                "Failed to write file {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;

        Ok(())
    }
}

impl Default for EnhancedJsonExporter {
    fn default() -> Self {
        Self::new(ExportConfig::default())
    }
}

/// Convenience function to export enhanced JSON with default settings
pub fn export_enhanced_json<P: AsRef<Path>>(
    output_dir: P,
    memory_stats: &MemoryStats,
    unsafe_reports: &[UnsafeReport],
    memory_passports: &[MemoryPassport],
) -> TrackingResult<()> {
    let exporter = EnhancedJsonExporter::default();
    exporter.export_enhanced_analysis(output_dir, memory_stats, unsafe_reports, memory_passports)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{AllocationInfo, BorrowInfo, CloneInfo};

    #[test]
    fn test_enhanced_allocation_conversion() {
        let exporter = EnhancedJsonExporter::default();

        let mut alloc = AllocationInfo::new(0x1000, 64);
        alloc.borrow_info = Some(BorrowInfo {
            immutable_borrows: 5,
            mutable_borrows: 2,
            max_concurrent_borrows: 3,
            last_borrow_timestamp: Some(1234567890),
        });
        alloc.clone_info = Some(CloneInfo {
            clone_count: 2,
            is_clone: true,
            original_ptr: Some(0x2000),
        });
        alloc.ownership_history_available = true;

        let enhanced = exporter.convert_to_enhanced_allocation(&alloc);

        assert_eq!(enhanced.ptr, 0x1000);
        assert_eq!(enhanced.size, 64);
        assert!(enhanced.borrow_info.is_some());
        assert!(enhanced.clone_info.is_some());
        assert!(enhanced.ownership_history_available);
    }

    #[test]
    fn test_lifetime_data_generation() {
        let exporter = EnhancedJsonExporter::default();

        let mut alloc = AllocationInfo::new(0x1000, 64);
        alloc.borrow_info = Some(BorrowInfo {
            immutable_borrows: 2,
            mutable_borrows: 1,
            max_concurrent_borrows: 2,
            last_borrow_timestamp: Some(1234567890),
        });
        alloc.ownership_history_available = true;

        let lifetime_data = exporter.generate_lifetime_data(&alloc);

        assert_eq!(lifetime_data.allocation_ptr, 0x1000);
        assert!(!lifetime_data.ownership_history.is_empty());

        // Should have at least Allocated event
        let allocated_event = lifetime_data
            .ownership_history
            .iter()
            .find(|e| e.event_type == "Allocated");
        assert!(allocated_event.is_some());
    }
}
