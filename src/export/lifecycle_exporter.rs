use crate::core::types::AllocationInfo;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LifecycleExportError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Export error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, LifecycleExportError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnershipEventType {
    Allocated,
    Cloned,
    Dropped,
    OwnershipTransferred,
    Borrowed,
    BorrowReleased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OwnershipEventDetails {
    Allocated { size: usize, type_name: String },
    Cloned { source_ptr: usize },
    Dropped { reason: String },
    OwnershipTransferred { new_owner: String },
    Borrowed { is_mutable: bool, scope: String },
    BorrowReleased { is_mutable: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipEvent {
    pub timestamp: u64,
    pub event_type: OwnershipEventType,
    pub source_stack_id: u64,
    pub details: OwnershipEventDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShutdownStatus {
    Reclaimed,
    FreedByForeign,
    InForeignCustody,
    Leaked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectLifecycle {
    pub allocation_ptr: usize,
    pub size_bytes: usize,
    pub type_name: String,
    pub var_name: Option<String>,
    pub ownership_history: Vec<OwnershipEvent>,
    pub status_at_shutdown: ShutdownStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub timestamp: String,
    pub version: String,
    pub total_objects: usize,
    pub export_duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct LifecycleExportConfig {
    pub include_system_allocations: bool,
    pub pretty_print: bool,
    pub batch_size: usize,
}

impl Default for LifecycleExportConfig {
    fn default() -> Self {
        Self {
            include_system_allocations: false,
            pretty_print: true,
            batch_size: 1000,
        }
    }
}

#[derive(Debug)]
pub struct ExportStats {
    pub objects_exported: usize,
    pub processing_time: Duration,
    pub output_size: u64,
}

pub struct LifecycleExporter {
    config: LifecycleExportConfig,
    next_stack_id: AtomicUsize,
}

impl LifecycleExporter {
    pub fn new(config: LifecycleExportConfig) -> Self {
        Self {
            config,
            next_stack_id: AtomicUsize::new(1),
        }
    }

    pub fn export_lifecycle_data<P: AsRef<Path>>(
        &self,
        allocations: &[AllocationInfo],
        output_path: P,
    ) -> Result<ExportStats> {
        let start_time = Instant::now();
        let output_file = File::create(&output_path)?;
        let mut writer = BufWriter::new(output_file);

        // Write start of JSON array
        writer.write_all(b"{\"objects\":[")?;

        let mut first = true;
        let mut objects_exported = 0;

        for chunk in allocations.chunks(self.config.batch_size) {
            for alloc in chunk {
                if let Some(lifecycle) = self.build_object_lifecycle(alloc)? {
                    if !first {
                        writer.write_all(b",")?;
                    }
                    first = false;

                    let json = if self.config.pretty_print {
                        serde_json::to_vec_pretty(&lifecycle)?
                    } else {
                        serde_json::to_vec(&lifecycle)?
                    };

                    writer.write_all(&json)?;
                    objects_exported += 1;
                }
            }
        }

        // Write end of JSON array and metadata
        let export_duration = start_time.elapsed();
        let metadata = ExportMetadata {
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            total_objects: objects_exported,
            export_duration_ms: export_duration.as_millis() as u64,
        };

        let metadata_json = if self.config.pretty_print {
            serde_json::to_string_pretty(&metadata)?
        } else {
            serde_json::to_string(&metadata)?
        };

        write!(writer, "],\"metadata\":{metadata_json}}}")?;

        writer.flush()?;

        Ok(ExportStats {
            objects_exported,
            processing_time: export_duration,
            output_size: output_path.as_ref().metadata()?.len(),
        })
    }

    fn build_object_lifecycle(&self, alloc: &AllocationInfo) -> Result<Option<ObjectLifecycle>> {
        // Skip system allocations if configured
        if !self.config.include_system_allocations && alloc.var_name.is_none() {
            return Ok(None);
        }

        let ownership_history = self.build_ownership_history(alloc)?;

        Ok(Some(ObjectLifecycle {
            allocation_ptr: alloc.ptr,
            size_bytes: alloc.size,
            type_name: alloc
                .type_name
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            var_name: alloc.var_name.clone(),
            ownership_history,
            status_at_shutdown: self.determine_shutdown_status(alloc),
        }))
    }

    fn build_ownership_history(&self, alloc: &AllocationInfo) -> Result<Vec<OwnershipEvent>> {
        let mut events = Vec::new();

        // Add allocation event
        events.push(OwnershipEvent {
            timestamp: alloc.timestamp_alloc,
            event_type: OwnershipEventType::Allocated,
            source_stack_id: self.next_stack_id(),
            details: OwnershipEventDetails::Allocated {
                size: alloc.size,
                type_name: alloc
                    .type_name
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
            },
        });

        // Additional events from allocation history can be added here
        // This would be populated from the allocation's history if available

        // Add deallocation event if applicable
        if let Some(dealloc_time) = alloc.timestamp_dealloc {
            events.push(OwnershipEvent {
                timestamp: dealloc_time,
                event_type: OwnershipEventType::Dropped,
                source_stack_id: self.next_stack_id(),
                details: OwnershipEventDetails::Dropped {
                    reason: "Deallocation".to_string(),
                },
            });
        }

        Ok(events)
    }

    fn determine_shutdown_status(&self, alloc: &AllocationInfo) -> ShutdownStatus {
        let is_leaked = alloc.timestamp_dealloc.is_none();
        match (alloc.timestamp_dealloc, is_leaked) {
            (Some(_), _) => ShutdownStatus::Reclaimed,
            (None, true) => ShutdownStatus::Leaked,
            (None, false) => ShutdownStatus::InForeignCustody,
        }
    }

    fn next_stack_id(&self) -> u64 {
        self.next_stack_id.fetch_add(1, Ordering::SeqCst) as u64
    }
}

/// Convenience function for one-shot lifecycle data export
///
/// # Arguments
/// * `allocations` - Slice of allocation info to export
/// * `output_path` - Path where to save the exported JSON file
/// * `config` - Optional configuration for the export
///
/// # Example
/// ```no_run
/// use memscope_rs::export::{export_lifecycle_data, LifecycleExportConfig};
/// use memscope_rs::core::types::AllocationInfo;
///
/// let allocations = vec![]; // Your allocations here
/// let config = LifecycleExportConfig {
///     include_system_allocations: false,
///     pretty_print: true,
///     batch_size: 1000,
/// };
///
/// export_lifecycle_data(&allocations, "lifecycle.json", Some(config)).unwrap();
/// ```
pub fn export_lifecycle_data<P: AsRef<Path>>(
    allocations: &[AllocationInfo],
    output_path: P,
    config: Option<LifecycleExportConfig>,
) -> Result<ExportStats> {
    let config = config.unwrap_or_default();
    let exporter = LifecycleExporter::new(config);
    exporter.export_lifecycle_data(allocations, output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AllocationInfo;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_allocation(
        ptr: usize,
        size: usize,
        type_name: Option<String>,
        var_name: Option<String>,
        timestamp_alloc: u64,
        timestamp_dealloc: Option<u64>,
    ) -> AllocationInfo {
        AllocationInfo {
            ptr,
            size,
            var_name,
            type_name,
            scope_name: None,
            timestamp_alloc,
            timestamp_dealloc,
            thread_id: "test_thread".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: timestamp_dealloc.is_none(),
            lifetime_ms: timestamp_dealloc.map(|dealloc| dealloc.saturating_sub(timestamp_alloc)),
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        }
    }

    fn create_temp_file_path(temp_dir: &TempDir, filename: &str) -> PathBuf {
        temp_dir.path().join(filename)
    }

    #[test]
    fn test_lifecycle_exporter_creation() {
        let config = LifecycleExportConfig::default();
        let exporter = LifecycleExporter::new(config);

        // Verify initial state
        assert_eq!(exporter.next_stack_id.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_default_config() {
        let config = LifecycleExportConfig::default();

        assert!(!config.include_system_allocations);
        assert!(config.pretty_print);
        assert_eq!(config.batch_size, 1000);
    }

    #[test]
    fn test_export_empty_allocations() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output_path = create_temp_file_path(&temp_dir, "empty_lifecycle.json");

        let config = LifecycleExportConfig::default();
        let exporter = LifecycleExporter::new(config);

        let allocations = vec![];
        let result = exporter.export_lifecycle_data(&allocations, &output_path);

        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.objects_exported, 0);
        assert!(stats.output_size > 0); // Should still have metadata

        // Verify file content
        let content = fs::read_to_string(&output_path).expect("Failed to read output file");
        assert!(content.contains("\"objects\":[]"));
        assert!(content.contains("\"metadata\""));
    }

    #[test]
    fn test_export_single_allocation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output_path = create_temp_file_path(&temp_dir, "single_lifecycle.json");

        let config = LifecycleExportConfig::default();
        let exporter = LifecycleExporter::new(config);

        let allocations = vec![create_test_allocation(
            0x1000,
            64,
            Some("String".to_string()),
            Some("test_var".to_string()),
            1000,
            Some(2000),
        )];

        let result = exporter.export_lifecycle_data(&allocations, &output_path);

        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.objects_exported, 1);

        // Verify file content
        let content = fs::read_to_string(&output_path).expect("Failed to read output file");
        assert!(content.contains("\"allocation_ptr\": 4096"));
        assert!(content.contains("\"size_bytes\": 64"));
        assert!(content.contains("\"type_name\": \"String\""));
        assert!(content.contains("\"var_name\": \"test_var\""));
    }

    #[test]
    fn test_export_multiple_allocations() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output_path = create_temp_file_path(&temp_dir, "multiple_lifecycle.json");

        let config = LifecycleExportConfig::default();
        let exporter = LifecycleExporter::new(config);

        let allocations = vec![
            create_test_allocation(
                0x1000,
                64,
                Some("String".to_string()),
                Some("var1".to_string()),
                1000,
                Some(2000),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("Vec<i32>".to_string()),
                Some("var2".to_string()),
                1500,
                None,
            ),
        ];

        let result = exporter.export_lifecycle_data(&allocations, &output_path);

        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.objects_exported, 2);

        // Verify file content contains both allocations
        let content = fs::read_to_string(&output_path).expect("Failed to read output file");
        assert!(content.contains("\"allocation_ptr\": 4096"));
        assert!(content.contains("\"allocation_ptr\": 8192"));
        assert!(content.contains("\"var1\""));
        assert!(content.contains("\"var2\""));
    }

    #[test]
    fn test_system_allocations_filtering() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output_path = create_temp_file_path(&temp_dir, "filtered_lifecycle.json");

        let config = LifecycleExportConfig {
            include_system_allocations: false,
            pretty_print: true,
            batch_size: 1000,
        };
        let exporter = LifecycleExporter::new(config);

        let allocations = vec![
            // User allocation (should be included)
            create_test_allocation(
                0x1000,
                64,
                Some("String".to_string()),
                Some("user_var".to_string()),
                1000,
                Some(2000),
            ),
            // System allocation (should be excluded)
            create_test_allocation(
                0x2000,
                128,
                Some("SystemAlloc".to_string()),
                None, // No var_name indicates system allocation
                1500,
                None,
            ),
        ];

        let result = exporter.export_lifecycle_data(&allocations, &output_path);

        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.objects_exported, 1); // Only user allocation

        let content = fs::read_to_string(&output_path).expect("Failed to read output file");
        assert!(content.contains("\"user_var\""));
        assert!(!content.contains("\"SystemAlloc\""));
    }

    #[test]
    fn test_include_system_allocations() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output_path = create_temp_file_path(&temp_dir, "with_system_lifecycle.json");

        let config = LifecycleExportConfig {
            include_system_allocations: true,
            pretty_print: true,
            batch_size: 1000,
        };
        let exporter = LifecycleExporter::new(config);

        let allocations = vec![
            create_test_allocation(
                0x1000,
                64,
                Some("String".to_string()),
                Some("user_var".to_string()),
                1000,
                Some(2000),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("SystemAlloc".to_string()),
                None,
                1500,
                None,
            ),
        ];

        let result = exporter.export_lifecycle_data(&allocations, &output_path);

        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.objects_exported, 2); // Both allocations
    }

    #[test]
    fn test_ownership_event_types() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output_path = create_temp_file_path(&temp_dir, "events_lifecycle.json");

        let config = LifecycleExportConfig::default();
        let exporter = LifecycleExporter::new(config);

        let allocations = vec![create_test_allocation(
            0x1000,
            64,
            Some("String".to_string()),
            Some("test_var".to_string()),
            1000,
            Some(2000),
        )];

        let result = exporter.export_lifecycle_data(&allocations, &output_path);
        assert!(result.is_ok());

        let content = fs::read_to_string(&output_path).expect("Failed to read output file");

        // Should contain allocation and deallocation events
        assert!(content.contains("\"Allocated\""));
        assert!(content.contains("\"Dropped\""));
        assert!(content.contains("\"ownership_history\""));
    }

    #[test]
    fn test_shutdown_status_determination() {
        let config = LifecycleExportConfig::default();
        let exporter = LifecycleExporter::new(config);

        // Test reclaimed status (has deallocation timestamp)
        let reclaimed_alloc = create_test_allocation(
            0x1000,
            64,
            Some("String".to_string()),
            Some("var1".to_string()),
            1000,
            Some(2000),
        );
        let status = exporter.determine_shutdown_status(&reclaimed_alloc);
        assert!(matches!(status, ShutdownStatus::Reclaimed));

        // Test leaked status (no deallocation timestamp)
        let leaked_alloc = create_test_allocation(
            0x2000,
            128,
            Some("Vec<i32>".to_string()),
            Some("var2".to_string()),
            1500,
            None,
        );
        let status = exporter.determine_shutdown_status(&leaked_alloc);
        assert!(matches!(status, ShutdownStatus::Leaked));
    }

    #[test]
    fn test_batch_processing() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output_path = create_temp_file_path(&temp_dir, "batch_lifecycle.json");

        let config = LifecycleExportConfig {
            include_system_allocations: true,
            pretty_print: false,
            batch_size: 2, // Small batch size for testing
        };
        let exporter = LifecycleExporter::new(config);

        let allocations = vec![
            create_test_allocation(
                0x1000,
                64,
                Some("String".to_string()),
                Some("var1".to_string()),
                1000,
                Some(2000),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("Vec<i32>".to_string()),
                Some("var2".to_string()),
                1500,
                None,
            ),
            create_test_allocation(
                0x3000,
                256,
                Some("HashMap".to_string()),
                Some("var3".to_string()),
                2000,
                Some(3000),
            ),
        ];

        let result = exporter.export_lifecycle_data(&allocations, &output_path);

        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.objects_exported, 3);
    }

    #[test]
    fn test_compact_output_format() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output_path = create_temp_file_path(&temp_dir, "compact_lifecycle.json");

        let config = LifecycleExportConfig {
            include_system_allocations: true,
            pretty_print: false, // Compact format
            batch_size: 1000,
        };
        let exporter = LifecycleExporter::new(config);

        let allocations = vec![create_test_allocation(
            0x1000,
            64,
            Some("String".to_string()),
            Some("test_var".to_string()),
            1000,
            Some(2000),
        )];

        let result = exporter.export_lifecycle_data(&allocations, &output_path);
        assert!(result.is_ok());

        let content = fs::read_to_string(&output_path).expect("Failed to read output file");

        // Compact format should not have extra whitespace
        assert!(!content.contains("  ")); // No double spaces
        assert!(!content.contains("\n  ")); // No indented newlines
    }

    #[test]
    fn test_convenience_function() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output_path = create_temp_file_path(&temp_dir, "convenience_lifecycle.json");

        let allocations = vec![create_test_allocation(
            0x1000,
            64,
            Some("String".to_string()),
            Some("test_var".to_string()),
            1000,
            Some(2000),
        )];

        // Test with custom config
        let config = LifecycleExportConfig {
            include_system_allocations: true,
            pretty_print: false,
            batch_size: 500,
        };

        let result = export_lifecycle_data(&allocations, &output_path, Some(config));
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.objects_exported, 1);
        assert!(stats.output_size > 0);
    }

    #[test]
    fn test_convenience_function_default_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output_path = create_temp_file_path(&temp_dir, "convenience_default_lifecycle.json");

        let allocations = vec![create_test_allocation(
            0x1000,
            64,
            Some("String".to_string()),
            Some("test_var".to_string()),
            1000,
            Some(2000),
        )];

        // Test with default config (None)
        let result = export_lifecycle_data(&allocations, &output_path, None);
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.objects_exported, 1);
    }

    #[test]
    fn test_stack_id_generation() {
        let config = LifecycleExportConfig::default();
        let exporter = LifecycleExporter::new(config);

        let id1 = exporter.next_stack_id();
        let id2 = exporter.next_stack_id();
        let id3 = exporter.next_stack_id();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_metadata_generation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output_path = create_temp_file_path(&temp_dir, "metadata_lifecycle.json");

        let config = LifecycleExportConfig::default();
        let exporter = LifecycleExporter::new(config);

        let allocations = vec![create_test_allocation(
            0x1000,
            64,
            Some("String".to_string()),
            Some("test_var".to_string()),
            1000,
            Some(2000),
        )];

        let result = exporter.export_lifecycle_data(&allocations, &output_path);
        assert!(result.is_ok());

        let content = fs::read_to_string(&output_path).expect("Failed to read output file");

        // Verify metadata fields
        assert!(content.contains("\"metadata\""));
        assert!(content.contains("\"timestamp\""));
        assert!(content.contains("\"version\""));
        assert!(content.contains("\"total_objects\": 1"));
        assert!(content.contains("\"export_duration_ms\""));
    }

    #[test]
    fn test_unknown_type_handling() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output_path = create_temp_file_path(&temp_dir, "unknown_type_lifecycle.json");

        let config = LifecycleExportConfig::default();
        let exporter = LifecycleExporter::new(config);

        let allocations = vec![create_test_allocation(
            0x1000,
            64,
            None, // No type name
            Some("test_var".to_string()),
            1000,
            Some(2000),
        )];

        let result = exporter.export_lifecycle_data(&allocations, &output_path);
        assert!(result.is_ok());

        let content = fs::read_to_string(&output_path).expect("Failed to read output file");
        assert!(content.contains("\"type_name\": \"unknown\""));
    }
}
