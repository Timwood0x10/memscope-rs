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
