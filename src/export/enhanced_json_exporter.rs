//! Enhanced JSON Exporter - Generates JSON files according to improve.md specifications
//!
//! This module creates the exact JSON format specified in improve.md:
//! 1. memory_analysis.json - Main memory analysis with extended fields
//! 2. lifetime.json - Ownership history and lifecycle events  
//! 3. unsafe_ffi.json - FFI safety analysis and memory passports

use crate::analysis::unsafe_ffi_tracker::MemoryPassport;
use crate::core::types::{
    AllocationInfo, BorrowInfo, CloneInfo, MemoryStats,
    TrackingError::{ExportError, SerializationError},
    TrackingResult,
};
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
        std::fs::create_dir_all(output_dir)
            .map_err(|e| ExportError(format!("Failed to create output directory: {e}",)))?;

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
        .map_err(|e| SerializationError(format!("Failed to serialize JSON: {e}",)))?;

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

    fn create_test_allocation(ptr: usize, size: usize) -> AllocationInfo {
        let mut alloc = AllocationInfo::new(ptr, size);
        alloc.var_name = Some("test_var".to_string());
        alloc.type_name = Some("TestType".to_string());
        alloc.scope_name = Some("test_scope".to_string());
        alloc.timestamp_alloc = 1234567890;
        alloc.timestamp_dealloc = None;
        alloc.borrow_count = 0;
        alloc.is_leaked = false;
        alloc.lifetime_ms = Some(1000);
        alloc
    }

    fn create_test_memory_stats() -> MemoryStats {
        let allocations = vec![
            create_test_allocation(0x1000, 64),
            create_test_allocation(0x2000, 128),
        ];

        MemoryStats {
            total_allocations: 2,
            total_allocated: 192,
            active_allocations: 2,
            active_memory: 192,
            peak_allocations: 2,
            peak_memory: 192,
            total_deallocations: 0,
            total_deallocated: 0,
            leaked_allocations: 0,
            leaked_memory: 0,
            fragmentation_analysis: crate::core::types::FragmentationAnalysis::default(),
            lifecycle_stats: crate::core::types::ScopeLifecycleMetrics::default(),
            allocations,
            system_library_stats: crate::core::types::SystemLibraryStats::default(),
            concurrency_analysis: crate::core::types::ConcurrencyAnalysis::default(),
        }
    }

    #[test]
    fn test_export_config_default() {
        let config = ExportConfig::default();
        
        assert!(config.pretty_print);
        assert!(config.include_stack_traces);
        assert!(config.generate_lifetime_file);
        assert!(config.generate_unsafe_ffi_file);
        assert_eq!(config.max_ownership_events, 1000);
    }

    #[test]
    fn test_export_config_debug_clone() {
        let config = ExportConfig {
            pretty_print: false,
            include_stack_traces: false,
            generate_lifetime_file: false,
            generate_unsafe_ffi_file: false,
            max_ownership_events: 500,
        };
        
        // Test Debug trait
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ExportConfig"));
        assert!(debug_str.contains("pretty_print"));
        assert!(debug_str.contains("false"));
        
        // Test Clone trait
        let cloned_config = config.clone();
        assert_eq!(cloned_config.pretty_print, config.pretty_print);
        assert_eq!(cloned_config.include_stack_traces, config.include_stack_traces);
        assert_eq!(cloned_config.generate_lifetime_file, config.generate_lifetime_file);
        assert_eq!(cloned_config.generate_unsafe_ffi_file, config.generate_unsafe_ffi_file);
        assert_eq!(cloned_config.max_ownership_events, config.max_ownership_events);
    }

    #[test]
    fn test_enhanced_json_exporter_new() {
        let config = ExportConfig {
            pretty_print: false,
            include_stack_traces: true,
            generate_lifetime_file: false,
            generate_unsafe_ffi_file: true,
            max_ownership_events: 2000,
        };
        
        let exporter = EnhancedJsonExporter::new(config.clone());
        assert_eq!(exporter.config.pretty_print, config.pretty_print);
        assert_eq!(exporter.config.include_stack_traces, config.include_stack_traces);
        assert_eq!(exporter.config.generate_lifetime_file, config.generate_lifetime_file);
        assert_eq!(exporter.config.generate_unsafe_ffi_file, config.generate_unsafe_ffi_file);
        assert_eq!(exporter.config.max_ownership_events, config.max_ownership_events);
    }

    #[test]
    fn test_enhanced_json_exporter_default() {
        let exporter1 = EnhancedJsonExporter::default();
        let exporter2 = EnhancedJsonExporter::new(ExportConfig::default());
        
        assert_eq!(exporter1.config.pretty_print, exporter2.config.pretty_print);
        assert_eq!(exporter1.config.include_stack_traces, exporter2.config.include_stack_traces);
        assert_eq!(exporter1.config.generate_lifetime_file, exporter2.config.generate_lifetime_file);
        assert_eq!(exporter1.config.generate_unsafe_ffi_file, exporter2.config.generate_unsafe_ffi_file);
        assert_eq!(exporter1.config.max_ownership_events, exporter2.config.max_ownership_events);
    }

    #[test]
    fn test_convert_to_enhanced_allocation_with_stack_traces() {
        let config = ExportConfig {
            include_stack_traces: true,
            ..Default::default()
        };
        let exporter = EnhancedJsonExporter::new(config);

        let mut alloc = create_test_allocation(0x1000, 64);
        alloc.stack_trace = Some(vec!["main".to_string(), "allocate".to_string()]);
        alloc.borrow_info = Some(BorrowInfo {
            immutable_borrows: 3,
            mutable_borrows: 1,
            max_concurrent_borrows: 2,
            last_borrow_timestamp: Some(1234567890),
        });

        let enhanced = exporter.convert_to_enhanced_allocation(&alloc);

        assert_eq!(enhanced.ptr, 0x1000);
        assert_eq!(enhanced.size, 64);
        assert!(enhanced.stack_trace.is_some());
        assert_eq!(enhanced.stack_trace.as_ref().unwrap().len(), 2);
        assert!(enhanced.borrow_info.is_some());
        assert_eq!(enhanced.borrow_info.as_ref().unwrap().immutable_borrows, 3);
    }

    #[test]
    fn test_convert_to_enhanced_allocation_without_stack_traces() {
        let config = ExportConfig {
            include_stack_traces: false,
            ..Default::default()
        };
        let exporter = EnhancedJsonExporter::new(config);

        let mut alloc = create_test_allocation(0x1000, 64);
        alloc.stack_trace = Some(vec!["main".to_string(), "allocate".to_string()]);

        let enhanced = exporter.convert_to_enhanced_allocation(&alloc);

        assert_eq!(enhanced.ptr, 0x1000);
        assert_eq!(enhanced.size, 64);
        assert!(enhanced.stack_trace.is_none()); // Should be None due to config
    }

    #[test]
    fn test_generate_lifetime_data_with_clone_info() {
        let exporter = EnhancedJsonExporter::default();

        let mut alloc = create_test_allocation(0x1000, 64);
        alloc.clone_info = Some(CloneInfo {
            clone_count: 2,
            is_clone: true,
            original_ptr: Some(0x2000),
        });
        alloc.ownership_history_available = true;

        let lifetime_data = exporter.generate_lifetime_data(&alloc);

        assert_eq!(lifetime_data.allocation_ptr, 0x1000);
        assert!(!lifetime_data.ownership_history.is_empty());

        // Should have Allocated and Cloned events
        let allocated_event = lifetime_data.ownership_history.iter()
            .find(|e| e.event_type == "Allocated");
        assert!(allocated_event.is_some());

        let cloned_event = lifetime_data.ownership_history.iter()
            .find(|e| e.event_type == "Cloned");
        assert!(cloned_event.is_some());

        // Check clone details
        let clone_event = cloned_event.unwrap();
        assert!(clone_event.details.contains_key("clone_source_ptr"));
    }

    #[test]
    fn test_generate_lifetime_data_with_borrow_events() {
        let exporter = EnhancedJsonExporter::default();

        let mut alloc = create_test_allocation(0x1000, 64);
        alloc.borrow_info = Some(BorrowInfo {
            immutable_borrows: 3,
            mutable_borrows: 1,
            max_concurrent_borrows: 2,
            last_borrow_timestamp: Some(1234567890),
        });
        alloc.ownership_history_available = true;

        let lifetime_data = exporter.generate_lifetime_data(&alloc);

        // Should have Allocated and Borrowed events
        let borrowed_events: Vec<_> = lifetime_data.ownership_history.iter()
            .filter(|e| e.event_type == "Borrowed")
            .collect();
        assert_eq!(borrowed_events.len(), 3); // Should match immutable_borrows

        // Check borrow details
        for borrow_event in borrowed_events {
            assert!(borrow_event.details.contains_key("borrower_scope"));
        }
    }

    #[test]
    fn test_generate_lifetime_data_with_deallocation() {
        let exporter = EnhancedJsonExporter::default();

        let mut alloc = create_test_allocation(0x1000, 64);
        alloc.timestamp_dealloc = Some(1234567890 + 5000);
        alloc.ownership_history_available = true;

        let lifetime_data = exporter.generate_lifetime_data(&alloc);

        // Should have Allocated and Dropped events
        let dropped_event = lifetime_data.ownership_history.iter()
            .find(|e| e.event_type == "Dropped");
        assert!(dropped_event.is_some());

        let drop_event = dropped_event.unwrap();
        assert_eq!(drop_event.timestamp, 1234567890 + 5000);
        assert_eq!(drop_event.source_stack_id, 99);
    }

    #[test]
    fn test_generate_lifetime_data_with_max_events_limit() {
        let config = ExportConfig {
            max_ownership_events: 2,
            ..Default::default()
        };
        let exporter = EnhancedJsonExporter::new(config);

        let mut alloc = create_test_allocation(0x1000, 64);
        alloc.borrow_info = Some(BorrowInfo {
            immutable_borrows: 10, // More than max_ownership_events
            mutable_borrows: 1,
            max_concurrent_borrows: 5,
            last_borrow_timestamp: Some(1234567890),
        });
        alloc.timestamp_dealloc = Some(1234567890 + 5000);
        alloc.ownership_history_available = true;

        let lifetime_data = exporter.generate_lifetime_data(&alloc);

        // Should be limited to max_ownership_events
        assert!(lifetime_data.ownership_history.len() <= 2);
    }

    #[test]
    fn test_ownership_event_debug_clone_serialize() {
        let mut details = HashMap::new();
        details.insert("test_key".to_string(), serde_json::Value::String("test_value".to_string()));

        let event = OwnershipEvent {
            timestamp: 1234567890,
            event_type: "TestEvent".to_string(),
            source_stack_id: 42,
            details,
        };

        // Test Debug trait
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("OwnershipEvent"));
        assert!(debug_str.contains("TestEvent"));
        assert!(debug_str.contains("42"));

        // Test Clone trait
        let cloned_event = event.clone();
        assert_eq!(cloned_event.timestamp, event.timestamp);
        assert_eq!(cloned_event.event_type, event.event_type);
        assert_eq!(cloned_event.source_stack_id, event.source_stack_id);
        assert_eq!(cloned_event.details.len(), event.details.len());

        // Test Serialize trait
        let serialized = serde_json::to_string(&event);
        assert!(serialized.is_ok());
        let json_str = serialized.unwrap();
        assert!(json_str.contains("TestEvent"));
        assert!(json_str.contains("1234567890"));

        // Test Deserialize trait
        let deserialized: Result<OwnershipEvent, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
        let deserialized_event = deserialized.unwrap();
        assert_eq!(deserialized_event.timestamp, event.timestamp);
        assert_eq!(deserialized_event.event_type, event.event_type);
    }

    #[test]
    fn test_lifetime_data_debug_clone_serialize() {
        let ownership_history = vec![
            OwnershipEvent {
                timestamp: 1234567890,
                event_type: "Allocated".to_string(),
                source_stack_id: 1,
                details: HashMap::new(),
            },
            OwnershipEvent {
                timestamp: 1234567900,
                event_type: "Dropped".to_string(),
                source_stack_id: 2,
                details: HashMap::new(),
            },
        ];

        let lifetime_data = LifetimeData {
            allocation_ptr: 0x1000,
            ownership_history,
        };

        // Test Debug trait
        let debug_str = format!("{:?}", lifetime_data);
        assert!(debug_str.contains("LifetimeData"));
        assert!(debug_str.contains("4096")); // 0x1000 in decimal
        assert!(debug_str.contains("Allocated"));

        // Test Clone trait
        let cloned_data = lifetime_data.clone();
        assert_eq!(cloned_data.allocation_ptr, lifetime_data.allocation_ptr);
        assert_eq!(cloned_data.ownership_history.len(), lifetime_data.ownership_history.len());

        // Test Serialize trait
        let serialized = serde_json::to_string(&lifetime_data);
        assert!(serialized.is_ok());
        let json_str = serialized.unwrap();
        assert!(json_str.contains("allocation_ptr"));
        assert!(json_str.contains("ownership_history"));

        // Test Deserialize trait
        let deserialized: Result<LifetimeData, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
        let deserialized_data = deserialized.unwrap();
        assert_eq!(deserialized_data.allocation_ptr, lifetime_data.allocation_ptr);
        assert_eq!(deserialized_data.ownership_history.len(), lifetime_data.ownership_history.len());
    }

    #[test]
    fn test_enhanced_allocation_info_debug_clone_serialize() {
        let enhanced_alloc = EnhancedAllocationInfo {
            ptr: 0x1000,
            size: 64,
            var_name: Some("test_var".to_string()),
            type_name: Some("TestType".to_string()),
            scope_name: Some("test_scope".to_string()),
            timestamp_alloc: 1234567890,
            timestamp_dealloc: Some(1234567900),
            borrow_count: 2,
            stack_trace: Some(vec!["main".to_string()]),
            is_leaked: false,
            lifetime_ms: Some(1000),
            borrow_info: Some(BorrowInfo {
                immutable_borrows: 1,
                mutable_borrows: 0,
                max_concurrent_borrows: 1,
                last_borrow_timestamp: Some(1234567890),
            }),
            clone_info: None,
            ownership_history_available: true,
        };

        // Test Debug trait
        let debug_str = format!("{:?}", enhanced_alloc);
        assert!(debug_str.contains("EnhancedAllocationInfo"));
        assert!(debug_str.contains("test_var"));
        assert!(debug_str.contains("TestType"));

        // Test Clone trait
        let cloned_alloc = enhanced_alloc.clone();
        assert_eq!(cloned_alloc.ptr, enhanced_alloc.ptr);
        assert_eq!(cloned_alloc.size, enhanced_alloc.size);
        assert_eq!(cloned_alloc.var_name, enhanced_alloc.var_name);
        assert_eq!(cloned_alloc.ownership_history_available, enhanced_alloc.ownership_history_available);

        // Test Serialize trait
        let serialized = serde_json::to_string(&enhanced_alloc);
        assert!(serialized.is_ok());
        let json_str = serialized.unwrap();
        assert!(json_str.contains("test_var"));
        assert!(json_str.contains("TestType"));
        assert!(json_str.contains("ownership_history_available"));

        // Test Deserialize trait
        let deserialized: Result<EnhancedAllocationInfo, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
        let deserialized_alloc = deserialized.unwrap();
        assert_eq!(deserialized_alloc.ptr, enhanced_alloc.ptr);
        assert_eq!(deserialized_alloc.var_name, enhanced_alloc.var_name);
        assert_eq!(deserialized_alloc.ownership_history_available, enhanced_alloc.ownership_history_available);
    }

    #[test]
    fn test_export_memory_analysis() -> TrackingResult<()> {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().map_err(|e| ExportError(e.to_string()))?;
        let exporter = EnhancedJsonExporter::default();
        let memory_stats = create_test_memory_stats();
        
        exporter.export_memory_analysis(&temp_dir, &memory_stats)?;
        
        let output_path = temp_dir.path().join("memory_analysis.json");
        assert!(output_path.exists());
        
        // Verify file content
        let content = std::fs::read_to_string(&output_path)
            .map_err(|e| ExportError(e.to_string()))?;
        let json_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| SerializationError(e.to_string()))?;
        
        assert!(json_data.get("metadata").is_some());
        assert!(json_data.get("summary").is_some());
        assert!(json_data.get("allocations").is_some());
        
        let metadata = &json_data["metadata"];
        assert_eq!(metadata["export_version"].as_str().unwrap(), "2.0");
        assert_eq!(metadata["specification"].as_str().unwrap(), "improve.md compliant");
        assert_eq!(metadata["total_allocations"].as_u64().unwrap(), 2);
        
        Ok(())
    }

    #[test]
    fn test_export_lifetime_data() -> TrackingResult<()> {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().map_err(|e| ExportError(e.to_string()))?;
        let exporter = EnhancedJsonExporter::default();
        
        let mut memory_stats = create_test_memory_stats();
        // Mark some allocations as having ownership history
        memory_stats.allocations[0].ownership_history_available = true;
        memory_stats.allocations[1].ownership_history_available = true;
        
        exporter.export_lifetime_data(&temp_dir, &memory_stats)?;
        
        let output_path = temp_dir.path().join("lifetime.json");
        assert!(output_path.exists());
        
        // Verify file content
        let content = std::fs::read_to_string(&output_path)
            .map_err(|e| ExportError(e.to_string()))?;
        let json_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| SerializationError(e.to_string()))?;
        
        assert!(json_data.get("metadata").is_some());
        assert!(json_data.get("ownership_histories").is_some());
        
        let metadata = &json_data["metadata"];
        assert_eq!(metadata["export_version"].as_str().unwrap(), "2.0");
        assert_eq!(metadata["specification"].as_str().unwrap(), "improve.md lifetime tracking");
        assert_eq!(metadata["total_tracked_allocations"].as_u64().unwrap(), 2);
        
        Ok(())
    }

    #[test]
    fn test_export_unsafe_ffi_analysis() -> TrackingResult<()> {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().map_err(|e| ExportError(e.to_string()))?;
        let exporter = EnhancedJsonExporter::default();
        
        let unsafe_reports = vec![];
        let memory_passports = vec![];
        
        exporter.export_unsafe_ffi_analysis(&temp_dir, &unsafe_reports, &memory_passports)?;
        
        let output_path = temp_dir.path().join("unsafe_ffi.json");
        assert!(output_path.exists());
        
        // Verify file content
        let content = std::fs::read_to_string(&output_path)
            .map_err(|e| ExportError(e.to_string()))?;
        let json_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| SerializationError(e.to_string()))?;
        
        assert!(json_data.get("metadata").is_some());
        assert!(json_data.get("unsafe_reports").is_some());
        assert!(json_data.get("memory_passports").is_some());
        
        let metadata = &json_data["metadata"];
        assert_eq!(metadata["export_version"].as_str().unwrap(), "2.0");
        assert_eq!(metadata["specification"].as_str().unwrap(), "improve.md unsafe FFI tracking");
        assert_eq!(metadata["total_unsafe_reports"].as_u64().unwrap(), 0);
        assert_eq!(metadata["total_memory_passports"].as_u64().unwrap(), 0);
        
        Ok(())
    }

    #[test]
    fn test_export_enhanced_analysis_all_files() -> TrackingResult<()> {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().map_err(|e| ExportError(e.to_string()))?;
        let exporter = EnhancedJsonExporter::default();
        let memory_stats = create_test_memory_stats();
        let unsafe_reports = vec![];
        let memory_passports = vec![];
        
        exporter.export_enhanced_analysis(&temp_dir, &memory_stats, &unsafe_reports, &memory_passports)?;
        
        // Verify all files were created
        assert!(temp_dir.path().join("memory_analysis.json").exists());
        assert!(temp_dir.path().join("lifetime.json").exists());
        assert!(temp_dir.path().join("unsafe_ffi.json").exists());
        
        Ok(())
    }

    #[test]
    fn test_export_enhanced_analysis_selective_files() -> TrackingResult<()> {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().map_err(|e| ExportError(e.to_string()))?;
        let config = ExportConfig {
            generate_lifetime_file: false,
            generate_unsafe_ffi_file: true,
            ..Default::default()
        };
        let exporter = EnhancedJsonExporter::new(config);
        let memory_stats = create_test_memory_stats();
        let unsafe_reports = vec![];
        let memory_passports = vec![];
        
        exporter.export_enhanced_analysis(&temp_dir, &memory_stats, &unsafe_reports, &memory_passports)?;
        
        // Verify selective file creation
        assert!(temp_dir.path().join("memory_analysis.json").exists());
        assert!(!temp_dir.path().join("lifetime.json").exists()); // Should not exist
        assert!(temp_dir.path().join("unsafe_ffi.json").exists());
        
        Ok(())
    }

    #[test]
    fn test_write_json_file_pretty_print() -> TrackingResult<()> {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().map_err(|e| ExportError(e.to_string()))?;
        let config = ExportConfig {
            pretty_print: true,
            ..Default::default()
        };
        let exporter = EnhancedJsonExporter::new(config);
        
        let test_data = serde_json::json!({
            "test": "value",
            "number": 42
        });
        
        let output_path = temp_dir.path().join("pretty_test.json");
        exporter.write_json_file(&output_path, &test_data)?;
        
        assert!(output_path.exists());
        
        let content = std::fs::read_to_string(&output_path)
            .map_err(|e| ExportError(e.to_string()))?;
        
        // Pretty printed JSON should contain newlines and indentation
        assert!(content.contains('\n'));
        assert!(content.contains("  ")); // Indentation
        
        Ok(())
    }

    #[test]
    fn test_write_json_file_compact() -> TrackingResult<()> {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().map_err(|e| ExportError(e.to_string()))?;
        let config = ExportConfig {
            pretty_print: false,
            ..Default::default()
        };
        let exporter = EnhancedJsonExporter::new(config);
        
        let test_data = serde_json::json!({
            "test": "value",
            "number": 42
        });
        
        let output_path = temp_dir.path().join("compact_test.json");
        exporter.write_json_file(&output_path, &test_data)?;
        
        assert!(output_path.exists());
        
        let content = std::fs::read_to_string(&output_path)
            .map_err(|e| ExportError(e.to_string()))?;
        
        // Compact JSON should not contain extra whitespace
        assert!(!content.contains('\n'));
        assert!(!content.contains("  ")); // No indentation
        
        Ok(())
    }

    #[test]
    fn test_export_enhanced_json_convenience_function() -> TrackingResult<()> {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().map_err(|e| ExportError(e.to_string()))?;
        let memory_stats = create_test_memory_stats();
        let unsafe_reports = vec![];
        let memory_passports = vec![];
        
        export_enhanced_json(&temp_dir, &memory_stats, &unsafe_reports, &memory_passports)?;
        
        // Verify all files were created using default settings
        assert!(temp_dir.path().join("memory_analysis.json").exists());
        assert!(temp_dir.path().join("lifetime.json").exists());
        assert!(temp_dir.path().join("unsafe_ffi.json").exists());
        
        Ok(())
    }

    #[test]
    fn test_export_with_empty_allocations() -> TrackingResult<()> {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().map_err(|e| ExportError(e.to_string()))?;
        let exporter = EnhancedJsonExporter::default();
        
        let empty_memory_stats = MemoryStats {
            total_allocations: 0,
            total_allocated: 0,
            active_allocations: 0,
            active_memory: 0,
            peak_allocations: 0,
            peak_memory: 0,
            total_deallocations: 0,
            total_deallocated: 0,
            leaked_allocations: 0,
            leaked_memory: 0,
            fragmentation_analysis: crate::core::types::FragmentationAnalysis::default(),
            lifecycle_stats: crate::core::types::ScopeLifecycleMetrics::default(),
            allocations: vec![], // Empty allocations
            system_library_stats: crate::core::types::SystemLibraryStats::default(),
            concurrency_analysis: crate::core::types::ConcurrencyAnalysis::default(),
        };
        
        exporter.export_memory_analysis(&temp_dir, &empty_memory_stats)?;
        
        let output_path = temp_dir.path().join("memory_analysis.json");
        assert!(output_path.exists());
        
        // Verify file content with empty allocations
        let content = std::fs::read_to_string(&output_path)
            .map_err(|e| ExportError(e.to_string()))?;
        let json_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| SerializationError(e.to_string()))?;
        
        let metadata = &json_data["metadata"];
        assert_eq!(metadata["total_allocations"].as_u64().unwrap(), 0);
        
        let allocations = json_data["allocations"].as_array().unwrap();
        assert_eq!(allocations.len(), 0);
        
        Ok(())
    }

    #[test]
    fn test_export_with_large_dataset() -> TrackingResult<()> {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().map_err(|e| ExportError(e.to_string()))?;
        let exporter = EnhancedJsonExporter::default();
        
        // Create a large dataset
        let mut large_allocations = Vec::new();
        for i in 0..1000 {
            let mut alloc = create_test_allocation(0x1000 + i * 0x100, 64 + i % 100);
            alloc.ownership_history_available = i % 2 == 0; // Half have ownership history
            large_allocations.push(alloc);
        }
        
        let large_memory_stats = MemoryStats {
            total_allocations: 1000,
            total_allocated: 114000, // Approximate
            active_allocations: 1000,
            active_memory: 114000,
            peak_allocations: 1000,
            peak_memory: 114000,
            total_deallocations: 0,
            total_deallocated: 0,
            leaked_allocations: 0,
            leaked_memory: 0,
            fragmentation_analysis: crate::core::types::FragmentationAnalysis::default(),
            lifecycle_stats: crate::core::types::ScopeLifecycleMetrics::default(),
            allocations: large_allocations,
            system_library_stats: crate::core::types::SystemLibraryStats::default(),
            concurrency_analysis: crate::core::types::ConcurrencyAnalysis::default(),
        };
        
        let unsafe_reports = vec![];
        let memory_passports = vec![];
        
        exporter.export_enhanced_analysis(&temp_dir, &large_memory_stats, &unsafe_reports, &memory_passports)?;
        
        // Verify all files were created
        assert!(temp_dir.path().join("memory_analysis.json").exists());
        assert!(temp_dir.path().join("lifetime.json").exists());
        assert!(temp_dir.path().join("unsafe_ffi.json").exists());
        
        // Verify memory analysis content
        let memory_content = std::fs::read_to_string(temp_dir.path().join("memory_analysis.json"))
            .map_err(|e| ExportError(e.to_string()))?;
        let memory_json: serde_json::Value = serde_json::from_str(&memory_content)
            .map_err(|e| SerializationError(e.to_string()))?;
        
        assert_eq!(memory_json["metadata"]["total_allocations"].as_u64().unwrap(), 1000);
        
        // Verify lifetime data content
        let lifetime_content = std::fs::read_to_string(temp_dir.path().join("lifetime.json"))
            .map_err(|e| ExportError(e.to_string()))?;
        let lifetime_json: serde_json::Value = serde_json::from_str(&lifetime_content)
            .map_err(|e| SerializationError(e.to_string()))?;
        
        assert_eq!(lifetime_json["metadata"]["total_tracked_allocations"].as_u64().unwrap(), 500); // Half have ownership history
        
        Ok(())
    }
}
