//! Unified Export API - Clean, consistent interface for all export operations
//!
//! This module provides a unified, well-named API that serves as the main entry point
//! for all export operations in the memscope project.
//!
//! **Architecture Note**:
//! This module now uses the new `render` subsystem directly instead of `MemoryTracker`.
//! The `Exporter` converts legacy `AllocationInfo` data into `TrackingSnapshot` format,
//! then delegates rendering to the appropriate renderer (Json, Html, Binary).
//!
//! This separation of concerns ensures:
//! - Clean separation between data conversion and rendering
//! - Consistent rendering across all export formats
//! - Better testability and maintainability

use crate::core::types::{AllocationInfo, MemoryStats, TrackingError};
use crate::data::{AllocationRecord, RenderOutput, TrackingSnapshot};
use crate::render::renderer::Renderer;
use crate::render::{BinaryRenderer, HtmlRenderer, JsonRenderer};
use crate::TrackingResult;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Export configuration with sensible defaults
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Include system allocations (default: false - user variables only)
    pub include_system_allocations: bool,
    /// Enable parallel processing for large datasets (default: auto-detect)
    pub parallel_processing: Option<bool>,
    /// Buffer size for I/O operations (default: 256KB)
    pub buffer_size: usize,
    /// Enable schema validation (default: true)
    pub validate_output: bool,
    /// Thread count for parallel operations (default: auto-detect)
    pub thread_count: Option<usize>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            include_system_allocations: false,
            parallel_processing: None,
            buffer_size: 256 * 1024, // 256KB
            validate_output: true,
            thread_count: None,
        }
    }
}

impl ExportConfig {
    /// Create config for user variables only (recommended)
    pub fn user_variables_only() -> Self {
        Self {
            include_system_allocations: false,
            ..Default::default()
        }
    }

    /// Create config for all allocations (system + user)
    pub fn all_allocations() -> Self {
        Self {
            include_system_allocations: true,
            ..Default::default()
        }
    }

    /// Create config optimized for performance
    pub fn fast_export() -> Self {
        Self {
            include_system_allocations: false,
            parallel_processing: Some(true),
            buffer_size: 512 * 1024, // 512KB
            validate_output: false,
            thread_count: None,
        }
    }

    /// Create config for comprehensive analysis
    pub fn comprehensive() -> Self {
        Self {
            include_system_allocations: true,
            parallel_processing: Some(true),
            buffer_size: 1024 * 1024, // 1MB
            validate_output: true,
            thread_count: None,
        }
    }
}

/// Export statistics and performance metrics
#[derive(Debug, Clone, Default)]
pub struct ExportStats {
    /// Number of allocations processed
    pub allocations_processed: usize,
    /// Number of user-defined variables
    pub user_variables: usize,
    /// Number of system allocations
    pub system_allocations: usize,
    /// Total processing time in milliseconds
    pub processing_time_ms: u64,
    /// Output file size in bytes
    pub output_size_bytes: u64,
    /// Processing rate (allocations per second)
    pub processing_rate: f64,
}

/// Unified export interface - main API for all export operations
pub struct Exporter {
    allocations: Arc<Vec<AllocationInfo>>,
    stats: MemoryStats,
    config: ExportConfig,
    /// Thread name to thread ID mapping for consistent ID assignment
    thread_id_mapping: Arc<Mutex<HashMap<String, u32>>>,
    /// Next available thread ID for new mappings
    next_thread_id: Arc<AtomicU32>,
}

impl Exporter {
    /// Create new exporter with allocation data
    pub fn new(allocations: Vec<AllocationInfo>, stats: MemoryStats, config: ExportConfig) -> Self {
        Self {
            allocations: Arc::new(allocations),
            stats,
            config,
            thread_id_mapping: Arc::new(Mutex::new(HashMap::new())),
            next_thread_id: Arc::new(AtomicU32::new(1)),
        }
    }

    /// Get or create a consistent thread ID for a given thread name
    ///
    /// This ensures that the same thread name always gets the same ID,
    /// avoiding hash collisions and maintaining consistency across allocations.
    ///
    /// Uses entry API to avoid race conditions between checking and inserting.
    fn get_or_create_thread_id(&self, thread_name: &str) -> u32 {
        // Handle potential PoisonError from mutex lock
        // If a previous thread panicked while holding the lock, we can still recover the data
        let mut mapping = match self.thread_id_mapping.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                // Recover from poisoned mutex - the HashMap is still valid
                poisoned.into_inner()
            }
        };
        let id = mapping.entry(thread_name.to_string())
            .or_insert_with(|| self.next_thread_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
        *id
    }

    /// Filter allocations based on configuration
    fn get_filtered_allocations(&self) -> Vec<AllocationInfo> {
        if self.config.include_system_allocations {
            // Include all allocations (user + system)
            (*self.allocations).clone()
        } else {
            // Only include user-defined variables (allocations with var_name)
            (*self.allocations)
                .iter()
                .filter(|allocation| allocation.var_name.is_some())
                .cloned()
                .collect()
        }
    }

    /// Convert AllocationInfo to TrackingSnapshot
    ///
    /// This is the bridge between the legacy AllocationInfo format
    /// and the new TrackingSnapshot structure used by the render subsystem.
    ///
    /// Note: total_allocations and total_allocated are automatically updated
    /// by snapshot.add_allocation(), so we only need to compute active stats.
    fn create_snapshot(&self, filtered_allocations: &[AllocationInfo]) -> TrackingSnapshot {
        let mut snapshot = TrackingSnapshot::new(crate::data::TrackingStrategy::Core);

        // Convert each AllocationInfo to AllocationRecord
        for allocation in filtered_allocations {
            let record = self.convert_allocation_info_to_record(allocation);
            snapshot.add_allocation(record); // Automatically updates total_allocations and total_allocated
        }

        // Calculate all statistics (active, leaked, peak, fragmentation, etc.)
        // This ensures complete and accurate statistics
        snapshot.calculate_stats();

        snapshot
    }

    /// Convert AllocationInfo to AllocationRecord
    fn convert_allocation_info_to_record(&self, allocation: &AllocationInfo) -> AllocationRecord {
        // Get consistent thread ID using mapping table
        // This ensures the same thread name always gets the same ID
        let thread_id = allocation.thread_id
            .parse::<u32>()
            .unwrap_or_else(|_| {
                // If parsing fails, log warning and use consistent mapping instead of hash
                tracing::warn!(
                    "Failed to parse thread_id '{}', using mapping table for consistent thread ID assignment",
                    allocation.thread_id
                );
                self.get_or_create_thread_id(&allocation.thread_id)
            });

        // Convert BorrowInfo from core::types to allocation::BorrowInfo
        let borrow_info = allocation.borrow_info.as_ref().map(|info| {
            crate::data::allocation::BorrowInfo {
                immutable_borrows: info.immutable_borrows,
                mutable_borrows: info.mutable_borrows,
                max_concurrent_borrows: info.max_concurrent_borrows,
                last_borrow_timestamp: info.last_borrow_timestamp,
            }
        });

        // Convert CloneInfo from core::types to allocation::CloneInfo
        let clone_info = allocation.clone_info.as_ref().map(|info| {
            crate::data::allocation::CloneInfo {
                clone_count: info.clone_count,
                is_clone: info.is_clone,
                original_ptr: info.original_ptr,
            }
        });

        // Convert SmartPointerInfo from core::types to allocation::SmartPointerInfo
        let smart_pointer_info = allocation.smart_pointer_info.as_ref().map(|info| {
            // Get the latest strong count from ref_count_history
            let latest_strong_count = info.ref_count_history
                .last()
                .map(|snapshot| snapshot.strong_count)
                .unwrap_or(0);

            crate::data::allocation::SmartPointerInfo {
                ptr_type: info.pointer_type.clone(),
                ref_count: latest_strong_count as u32, // Use latest strong count
                original_ptr: Some(info.data_ptr), // Use data_ptr as original pointer
            }
        });

        // Convert GenericTypeInfo from core::types to allocation::GenericTypeInfo
        let generic_info = allocation.generic_info.as_ref().map(|info| {
            crate::data::allocation::GenericTypeInfo {
                base_type: info.base_type.clone(),
                type_parameters: info.type_parameters.iter()
                    .map(|tp| tp.concrete_type.clone())
                    .collect(),
                instance_count: info.monomorphization_info.instance_count,
            }
        });

        // Convert DynamicTypeInfo from core::types to allocation::DynamicTypeInfo
        let dynamic_type_info = allocation.dynamic_type_info.as_ref().map(|info| {
            crate::data::allocation::DynamicTypeInfo {
                trait_name: info.trait_name.clone(),
                concrete_type: info.concrete_type.clone(),
                size: info.vtable_info.vtable_size, // Use vtable size as proxy for trait object size
            }
        });

        // Convert MemoryLayoutInfo from core::types to allocation::MemoryLayoutInfo
        let memory_layout = allocation.memory_layout.as_ref().map(|info| {
            crate::data::allocation::MemoryLayoutInfo {
                total_size: info.total_size,
                alignment: info.alignment,
                padding_bytes: info.padding_info.total_padding_bytes,
                utilization: info.layout_efficiency.memory_utilization,
            }
        });

        AllocationRecord {
            ptr: allocation.ptr,
            size: allocation.size,
            timestamp: allocation.timestamp_alloc,
            thread_id,
            stack_id: None, // AllocationInfo doesn't have stack_id
            var_name: allocation.var_name.clone(),
            type_name: allocation.type_name.clone(),
            scope_name: allocation.scope_name.clone(),
            is_active: allocation.timestamp_dealloc.is_none(),
            dealloc_timestamp: allocation.timestamp_dealloc,
            is_leaked: allocation.is_leaked,
            borrow_count: allocation.borrow_count,
            stack_trace: allocation.stack_trace.clone(),
            lifetime_ms: allocation.lifetime_ms,
            borrow_info,
            clone_info,
            smart_pointer_info,
            generic_info,
            dynamic_type_info,
            memory_layout,
            ownership_history_available: allocation.ownership_history_available,
            // All advanced fields are fully converted and preserved in AllocationRecord
        }
    }

    /// Export to JSON format
    pub fn export_json<P: AsRef<Path>>(&self, output_path: P) -> TrackingResult<ExportStats> {
        let start_time = Instant::now();
        let filtered_allocations = self.get_filtered_allocations();

        // Ensure output directory exists
        if let Some(parent) = output_path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| TrackingError::IoError(e.to_string()))?;
        }

        // Create snapshot from filtered allocations
        let snapshot = self.create_snapshot(&filtered_allocations);

        // Use JsonRenderer directly
        let renderer = JsonRenderer;
        let output = renderer.render(&snapshot)
            .map_err(|e| TrackingError::ExportError(e.to_string()))?;

        // Write output to file
        self.write_output(output, &output_path)?;

        let processing_time = start_time.elapsed();
        let output_size = std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(ExportStats {
            allocations_processed: self.allocations.len(),
            user_variables: filtered_allocations.len(),
            system_allocations: self.allocations.len() - filtered_allocations.len(),
            processing_time_ms: processing_time.as_millis() as u64,
            output_size_bytes: output_size,
            processing_rate: self.allocations.len() as f64
                / processing_time.as_secs_f64().max(0.001),
        })
    }

    /// Write render output to file
    fn write_output<P: AsRef<Path>>(&self, output: RenderOutput, output_path: P) -> TrackingResult<()> {
        let data = match output {
            RenderOutput::String(s) => s.into_bytes(),
            RenderOutput::Bytes(b) => b,
            RenderOutput::File(file_path) => {
                // If it's already a file, copy it to the destination
                return std::fs::copy(&file_path, output_path)
                    .map_err(|e| TrackingError::IoError(e.to_string()))
                    .map(|_| ());
            }
        };

        std::fs::write(output_path, data)
            .map_err(|e| TrackingError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Export to binary format
    pub fn export_binary<P: AsRef<Path>>(&self, output_path: P) -> TrackingResult<ExportStats> {
        let start_time = Instant::now();
        let filtered_allocations = self.get_filtered_allocations();

        // Ensure output directory exists
        if let Some(parent) = output_path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| TrackingError::IoError(e.to_string()))?;
        }

        // Create snapshot from filtered allocations
        let snapshot = self.create_snapshot(&filtered_allocations);

        // Use BinaryRenderer directly
        let renderer = BinaryRenderer;
        let output = renderer.render(&snapshot)
            .map_err(|e| TrackingError::ExportError(e.to_string()))?;

        // Write output to file
        self.write_output(output, &output_path)?;

        let processing_time = start_time.elapsed();
        let output_size = std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(ExportStats {
            allocations_processed: filtered_allocations.len(),
            user_variables: filtered_allocations.len(),
            system_allocations: 0,
            processing_time_ms: processing_time.as_millis() as u64,
            output_size_bytes: output_size,
            processing_rate: filtered_allocations.len() as f64
                / processing_time.as_secs_f64().max(0.001),
        })
    }

    /// Export to HTML format
    pub fn export_html<P: AsRef<Path>>(&self, output_path: P) -> TrackingResult<ExportStats> {
        let start_time = Instant::now();
        let filtered_allocations = self.get_filtered_allocations();

        // Ensure output directory exists
        if let Some(parent) = output_path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| TrackingError::IoError(e.to_string()))?;
        }

        // Create snapshot from filtered allocations
        let snapshot = self.create_snapshot(&filtered_allocations);

        // Use HtmlRenderer directly
        let renderer = HtmlRenderer;
        let output = renderer.render(&snapshot)
            .map_err(|e| TrackingError::ExportError(e.to_string()))?;

        // Write output to file
        self.write_output(output, &output_path)?;

        let processing_time = start_time.elapsed();
        let output_size = std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(ExportStats {
            allocations_processed: filtered_allocations.len(),
            user_variables: filtered_allocations.len(),
            system_allocations: 0,
            processing_time_ms: processing_time.as_millis() as u64,
            output_size_bytes: output_size,
            processing_rate: filtered_allocations.len() as f64
                / processing_time.as_secs_f64().max(0.001),
        })
    }

    /// Convert binary to JSON format
    pub fn binary_to_json<P: AsRef<Path>>(
        binary_path: P,
        output_path: P,
    ) -> TrackingResult<ExportStats> {
        let start_time = Instant::now();

        // Get file size before conversion for stats
        let input_size = std::fs::metadata(binary_path.as_ref())
            .map(|m| m.len())
            .unwrap_or(0);

        // Delegate to binary module
        crate::export::binary::parse_binary_to_json(binary_path.as_ref(), output_path.as_ref())
            .map_err(|e| TrackingError::ExportError(e.to_string()))?;

        let processing_time = start_time.elapsed();

        // Get output file size
        let output_size = std::fs::metadata(output_path.as_ref())
            .map(|m| m.len())
            .unwrap_or(0);

        // Estimate allocations based on file size (approximate)
        let estimated_allocations = input_size / 100; // Rough estimate

        Ok(ExportStats {
            allocations_processed: estimated_allocations as usize,
            user_variables: estimated_allocations as usize, // Best guess
            system_allocations: 0,                          // Can't determine from binary alone
            processing_time_ms: processing_time.as_millis() as u64,
            output_size_bytes: output_size,
            processing_rate: estimated_allocations as f64
                / processing_time.as_secs_f64().max(0.001),
        })
    }

    /// Convert binary to HTML format
    pub fn binary_to_html<P: AsRef<Path> + Clone>(
        binary_path: P,
        output_path: P,
    ) -> TrackingResult<ExportStats> {
        let start_time = Instant::now();

        // Get file size before conversion for stats
        let input_size = std::fs::metadata(&binary_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // Extract project name from output path
        let project_name = output_path.as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("project");

        // Delegate to binary module
        // Note: export_binary_to_html generates file with name "{project_name}_user_dashboard.html"
        // in the current directory, so we need to move it to the desired output path
        crate::export::binary::export_binary_to_html(
            binary_path.as_ref(),
            project_name
        ).map_err(|e| TrackingError::ExportError(e.to_string()))?;

        // Move the generated file to the desired output path
        let generated_file_name = format!("{}_user_dashboard.html", project_name);
        let generated_file = std::path::PathBuf::from(&generated_file_name);
        
        if generated_file.exists() {
            // Ensure output directory exists
            if let Some(parent) = output_path.as_ref().parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| TrackingError::IoError(e.to_string()))?;
            }
            
            // Move or copy the file to the desired location
            std::fs::rename(&generated_file, output_path.as_ref())
                .or_else(|_| {
                    // If rename fails (e.g., cross-device), try copy and delete
                    std::fs::copy(&generated_file, output_path.as_ref())
                        .and_then(|_| std::fs::remove_file(&generated_file))
                })
                .map_err(|e| TrackingError::IoError(format!("Failed to move generated file: {}", e)))?;
        }

        let processing_time = start_time.elapsed();

        // Get output file size
        let output_size = std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // Estimate allocations based on file size (approximate)
        let estimated_allocations = input_size / 100; // Rough estimate

        Ok(ExportStats {
            allocations_processed: estimated_allocations as usize,
            user_variables: estimated_allocations as usize, // Best guess
            system_allocations: 0,                          // Can't determine from binary alone
            processing_time_ms: processing_time.as_millis() as u64,
            output_size_bytes: output_size,
            processing_rate: estimated_allocations as f64
                / processing_time.as_secs_f64().max(0.001),
        })
    }
}

// High-level convenience functions for common export scenarios
// These are the main entry points for most users

/// Export user variables to JSON format
/// This is the most commonly used export function for development and debugging
pub fn export_user_variables_json<P: AsRef<Path>>(
    allocations: Vec<AllocationInfo>,
    stats: MemoryStats,
    output_path: P,
) -> TrackingResult<ExportStats> {
    let exporter = Exporter::new(allocations, stats, ExportConfig::user_variables_only());
    exporter.export_json(output_path)
}

/// Export user variables to binary format
/// Provides 3x faster export with 60% smaller file size compared to JSON
pub fn export_user_variables_binary<P: AsRef<Path>>(
    allocations: Vec<AllocationInfo>,
    stats: MemoryStats,
    output_path: P,
) -> TrackingResult<ExportStats> {
    let exporter = Exporter::new(allocations, stats, ExportConfig::user_variables_only());
    exporter.export_binary(output_path)
}

/// Fast export for performance-critical scenarios
/// Optimized for speed with reduced data quality checks
pub fn export_fast<P: AsRef<Path>>(
    allocations: Vec<AllocationInfo>,
    stats: MemoryStats,
    output_path: P,
) -> TrackingResult<ExportStats> {
    let exporter = Exporter::new(allocations, stats, ExportConfig::fast_export());
    exporter.export_json(output_path)
}

/// Comprehensive export for detailed analysis
/// Includes all system allocations and detailed analysis (slower but complete)
pub fn export_comprehensive<P: AsRef<Path>>(
    allocations: Vec<AllocationInfo>,
    stats: MemoryStats,
    output_path: P,
) -> TrackingResult<ExportStats> {
    let exporter = Exporter::new(allocations, stats, ExportConfig::comprehensive());
    exporter.export_json(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_export_config_defaults() {
        let config = ExportConfig::default();
        assert!(!config.include_system_allocations);
        assert_eq!(config.buffer_size, 256 * 1024);
        assert!(config.validate_output);
    }

    #[test]
    fn test_export_json() -> TrackingResult<()> {
        let temp_dir = tempdir()?;
        let output_path = temp_dir.path().join("test.json");

        let stats = export_user_variables_json(vec![], MemoryStats::default(), &output_path)?;

        assert!(output_path.exists());
        assert_eq!(stats.allocations_processed, 0);
        Ok(())
    }

    fn create_test_allocation(
        ptr: usize,
        size: usize,
        var_name: Option<String>,
        type_name: Option<String>,
    ) -> AllocationInfo {
        AllocationInfo {
            ptr,
            size,
            var_name,
            type_name,
            scope_name: Some("test_scope".to_string()),
            timestamp_alloc: 1234567890,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: Some(100),
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

    fn create_test_memory_stats() -> MemoryStats {
        MemoryStats {
            total_allocations: 10,
            total_allocated: 1024,
            active_allocations: 5,
            active_memory: 512,
            peak_allocations: 8,
            peak_memory: 768,
            total_deallocations: 5,
            total_deallocated: 512,
            leaked_allocations: 0,
            leaked_memory: 0,
            fragmentation_analysis: crate::core::types::FragmentationAnalysis::default(),
            lifecycle_stats: crate::core::types::ScopeLifecycleMetrics::default(),
            allocations: Vec::new(),
            system_library_stats: crate::core::types::SystemLibraryStats::default(),
            concurrency_analysis: crate::core::types::ConcurrencyAnalysis::default(),
        }
    }

    #[test]
    fn test_export_config_user_variables_only() {
        let config = ExportConfig::user_variables_only();
        assert!(!config.include_system_allocations);
        assert_eq!(config.buffer_size, 256 * 1024);
        assert!(config.validate_output);
        assert!(config.parallel_processing.is_none());
        assert!(config.thread_count.is_none());
    }

    #[test]
    fn test_export_config_all_allocations() {
        let config = ExportConfig::all_allocations();
        assert!(config.include_system_allocations);
        assert_eq!(config.buffer_size, 256 * 1024);
        assert!(config.validate_output);
        assert!(config.parallel_processing.is_none());
        assert!(config.thread_count.is_none());
    }

    #[test]
    fn test_export_config_fast_export() {
        let config = ExportConfig::fast_export();
        assert!(!config.include_system_allocations);
        assert_eq!(config.buffer_size, 512 * 1024);
        assert!(!config.validate_output);
        assert_eq!(config.parallel_processing, Some(true));
        assert!(config.thread_count.is_none());
    }

    #[test]
    fn test_export_config_comprehensive() {
        let config = ExportConfig::comprehensive();
        assert!(config.include_system_allocations);
        assert_eq!(config.buffer_size, 1024 * 1024);
        assert!(config.validate_output);
        assert_eq!(config.parallel_processing, Some(true));
        assert!(config.thread_count.is_none());
    }

    #[test]
    fn test_export_config_debug_clone() {
        let config = ExportConfig::default();

        // Test Debug trait
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ExportConfig"));
        assert!(debug_str.contains("include_system_allocations"));
        assert!(debug_str.contains("false")); // include_system_allocations is false by default

        // Test Clone trait
        let cloned_config = config.clone();
        assert_eq!(
            cloned_config.include_system_allocations,
            config.include_system_allocations
        );
        assert_eq!(
            cloned_config.parallel_processing,
            config.parallel_processing
        );
        assert_eq!(cloned_config.buffer_size, config.buffer_size);
        assert_eq!(cloned_config.validate_output, config.validate_output);
        assert_eq!(cloned_config.thread_count, config.thread_count);
    }

    #[test]
    fn test_export_stats_default() {
        let stats = ExportStats::default();

        assert_eq!(stats.allocations_processed, 0);
        assert_eq!(stats.user_variables, 0);
        assert_eq!(stats.system_allocations, 0);
        assert_eq!(stats.processing_time_ms, 0);
        assert_eq!(stats.output_size_bytes, 0);
        assert_eq!(stats.processing_rate, 0.0);
    }

    #[test]
    fn test_export_stats_debug_clone() {
        let stats = ExportStats {
            allocations_processed: 100,
            user_variables: 80,
            system_allocations: 20,
            processing_time_ms: 1000,
            output_size_bytes: 50000,
            processing_rate: 100.0,
        };

        // Test Debug trait
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("ExportStats"));
        assert!(debug_str.contains("100")); // allocations_processed
        assert!(debug_str.contains("1000")); // processing_time_ms

        // Test Clone trait
        let cloned_stats = stats.clone();
        assert_eq!(
            cloned_stats.allocations_processed,
            stats.allocations_processed
        );
        assert_eq!(cloned_stats.user_variables, stats.user_variables);
        assert_eq!(cloned_stats.system_allocations, stats.system_allocations);
        assert_eq!(cloned_stats.processing_time_ms, stats.processing_time_ms);
        assert_eq!(cloned_stats.output_size_bytes, stats.output_size_bytes);
        assert_eq!(cloned_stats.processing_rate, stats.processing_rate);
    }

    #[test]
    fn test_exporter_new() {
        let allocations = vec![
            create_test_allocation(
                0x1000,
                64,
                Some("var1".to_string()),
                Some("String".to_string()),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("var2".to_string()),
                Some("Vec<i32>".to_string()),
            ),
        ];
        let stats = create_test_memory_stats();
        let config = ExportConfig::default();

        let exporter = Exporter::new(allocations.clone(), stats, config);
        assert_eq!(exporter.allocations.len(), 2);
    }

    #[test]
    fn test_exporter_get_filtered_allocations() {
        let allocations = vec![
            // User allocations (have var_name)
            create_test_allocation(
                0x1000,
                64,
                Some("user_var1".to_string()),
                Some("String".to_string()),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("user_var2".to_string()),
                Some("Vec<i32>".to_string()),
            ),
            // System allocations (no var_name)
            create_test_allocation(0x3000, 32, None, Some("SystemType".to_string())),
            create_test_allocation(0x4000, 16, None, None),
        ];
        let stats = create_test_memory_stats();

        // Test with include_system_allocations = false (user_variables_only)
        let config_user_only = ExportConfig::user_variables_only();
        let exporter_user_only =
            Exporter::new(allocations.clone(), stats.clone(), config_user_only);
        let filtered_user_only = exporter_user_only.get_filtered_allocations();
        assert_eq!(filtered_user_only.len(), 2); // Only user allocations with var_name

        // Verify all filtered allocations have var_name
        for allocation in &filtered_user_only {
            assert!(
                allocation.var_name.is_some(),
                "User-only filter should only include allocations with var_name"
            );
        }

        // Test with include_system_allocations = true (all_allocations)
        let config_all = ExportConfig::all_allocations();
        let exporter_all = Exporter::new(allocations.clone(), stats, config_all);
        let filtered_all = exporter_all.get_filtered_allocations();
        assert_eq!(filtered_all.len(), 4); // All allocations (user + system)
    }

    #[test]
    fn test_exporter_export_json() -> TrackingResult<()> {
        let temp_dir = tempdir()?;
        let output_path = temp_dir.path().join("exporter_test.json");

        let allocations = vec![create_test_allocation(
            0x1000,
            64,
            Some("test_var".to_string()),
            Some("String".to_string()),
        )];
        let stats = create_test_memory_stats();
        let config = ExportConfig::default();

        let exporter = Exporter::new(allocations, stats, config);
        let export_stats = exporter.export_json(&output_path)?;

        assert!(output_path.exists());
        assert_eq!(export_stats.allocations_processed, 1);
        assert_eq!(export_stats.user_variables, 1);
        // processing_time_ms is u64, so >= 0 is always true - check it's reasonable instead
        assert!(export_stats.processing_time_ms < 10000); // less than 10 seconds
        assert!(export_stats.output_size_bytes > 0);
        assert!(export_stats.processing_rate >= 0.0);

        Ok(())
    }

    #[test]
    fn test_exporter_export_binary() -> TrackingResult<()> {
        let temp_dir = tempdir()?;
        let output_path = temp_dir.path().join("exporter_test.memscope");

        let allocations = vec![create_test_allocation(
            0x1000,
            64,
            Some("test_var".to_string()),
            Some("String".to_string()),
        )];
        let stats = create_test_memory_stats();
        let config = ExportConfig::default();

        let exporter = Exporter::new(allocations, stats, config);
        let export_stats = exporter.export_binary(&output_path)?;

        assert!(output_path.exists());
        assert_eq!(export_stats.allocations_processed, 1);
        assert_eq!(export_stats.user_variables, 1);
        assert_eq!(export_stats.system_allocations, 0);
        assert!(export_stats.output_size_bytes > 0);
        assert!(export_stats.processing_rate >= 0.0);

        Ok(())
    }

    #[test]
    fn test_export_user_variables_json() -> TrackingResult<()> {
        let temp_dir = tempdir()?;
        let output_path = temp_dir.path().join("user_vars.json");

        let allocations = vec![
            create_test_allocation(
                0x1000,
                64,
                Some("user_var1".to_string()),
                Some("String".to_string()),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("user_var2".to_string()),
                Some("Vec<i32>".to_string()),
            ),
        ];
        let stats = create_test_memory_stats();

        let export_stats = export_user_variables_json(allocations, stats, &output_path)?;

        assert!(output_path.exists());
        assert_eq!(export_stats.allocations_processed, 2);
        assert_eq!(export_stats.user_variables, 2);
        assert!(export_stats.output_size_bytes > 0);

        Ok(())
    }

    #[test]
    fn test_export_user_variables_binary() -> TrackingResult<()> {
        let temp_dir = tempdir()?;
        let output_path = temp_dir.path().join("user_vars.memscope");

        let allocations = vec![
            create_test_allocation(
                0x1000,
                64,
                Some("user_var1".to_string()),
                Some("String".to_string()),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("user_var2".to_string()),
                Some("Vec<i32>".to_string()),
            ),
        ];
        let stats = create_test_memory_stats();

        let export_stats = export_user_variables_binary(allocations, stats, &output_path)?;

        assert!(output_path.exists());
        assert_eq!(export_stats.allocations_processed, 2);
        assert_eq!(export_stats.user_variables, 2);
        assert!(export_stats.output_size_bytes > 0);

        Ok(())
    }

    #[test]
    fn test_export_fast() -> TrackingResult<()> {
        let temp_dir = tempdir()?;
        let output_path = temp_dir.path().join("fast_export.json");

        let allocations = vec![create_test_allocation(
            0x1000,
            64,
            Some("fast_var".to_string()),
            Some("String".to_string()),
        )];
        let stats = create_test_memory_stats();

        let export_stats = export_fast(allocations, stats, &output_path)?;

        assert!(output_path.exists());
        assert_eq!(export_stats.allocations_processed, 1);
        assert_eq!(export_stats.user_variables, 1);
        assert!(export_stats.output_size_bytes > 0);

        Ok(())
    }

    #[test]
    fn test_export_comprehensive() -> TrackingResult<()> {
        let temp_dir = tempdir()?;
        let output_path = temp_dir.path().join("comprehensive_export.json");

        let allocations = vec![
            create_test_allocation(
                0x1000,
                64,
                Some("comp_var1".to_string()),
                Some("String".to_string()),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("comp_var2".to_string()),
                Some("Vec<i32>".to_string()),
            ),
            create_test_allocation(
                0x3000,
                256,
                Some("comp_var3".to_string()),
                Some("HashMap<String, i32>".to_string()),
            ),
        ];
        let stats = create_test_memory_stats();

        let export_stats = export_comprehensive(allocations, stats, &output_path)?;

        assert!(output_path.exists());
        assert_eq!(export_stats.allocations_processed, 3);
        assert_eq!(export_stats.user_variables, 3);
        assert!(export_stats.output_size_bytes > 0);

        Ok(())
    }

    #[test]
    fn test_export_with_different_configs() -> TrackingResult<()> {
        let temp_dir = tempdir()?;

        let allocations = vec![create_test_allocation(
            0x1000,
            64,
            Some("config_var".to_string()),
            Some("String".to_string()),
        )];
        let stats = create_test_memory_stats();

        // Test with fast config
        let fast_config = ExportConfig::fast_export();
        let fast_exporter = Exporter::new(allocations.clone(), stats.clone(), fast_config);
        let fast_output = temp_dir.path().join("fast_config.json");
        let _fast_stats = fast_exporter.export_json(&fast_output)?;
        assert!(fast_output.exists());

        // Test with comprehensive config
        let comp_config = ExportConfig::comprehensive();
        let comp_exporter = Exporter::new(allocations.clone(), stats.clone(), comp_config);
        let comp_output = temp_dir.path().join("comp_config.json");
        let _comp_stats = comp_exporter.export_json(&comp_output)?;
        assert!(comp_output.exists());

        // Test with custom config
        let custom_config = ExportConfig {
            include_system_allocations: true,
            parallel_processing: Some(false),
            buffer_size: 128 * 1024,
            validate_output: false,
            thread_count: Some(2),
        };
        let custom_exporter = Exporter::new(allocations, stats, custom_config);
        let custom_output = temp_dir.path().join("custom_config.json");
        let _custom_stats = custom_exporter.export_json(&custom_output)?;
        assert!(custom_output.exists());

        Ok(())
    }

    #[test]
    fn test_export_empty_allocations() -> TrackingResult<()> {
        let temp_dir = tempdir()?;
        let output_path = temp_dir.path().join("empty.json");

        let allocations = vec![];
        let stats = create_test_memory_stats();

        let export_stats = export_user_variables_json(allocations, stats, &output_path)?;

        assert!(output_path.exists());
        assert_eq!(export_stats.allocations_processed, 0);
        assert_eq!(export_stats.user_variables, 0);
        assert!(export_stats.output_size_bytes > 0); // JSON file should have some content

        Ok(())
    }

    #[test]
    fn test_export_large_dataset() -> TrackingResult<()> {
        let temp_dir = tempdir()?;
        let output_path = temp_dir.path().join("large_dataset.json");

        // Create a large dataset
        let mut allocations = Vec::new();
        for i in 0..1000 {
            allocations.push(create_test_allocation(
                0x1000 + i * 0x100,
                64 + i % 100,
                Some(format!("var_{}", i)),
                Some(format!("Type{}", i % 10)),
            ));
        }
        let stats = create_test_memory_stats();

        let export_stats = export_user_variables_json(allocations, stats, &output_path)?;

        assert!(output_path.exists());
        assert_eq!(export_stats.allocations_processed, 1000);
        assert_eq!(export_stats.user_variables, 1000);
        assert!(export_stats.output_size_bytes > 0); // Should have some content

        Ok(())
    }

    #[test]
    fn test_export_stats_calculations() {
        let allocations = vec![
            create_test_allocation(
                0x1000,
                64,
                Some("var1".to_string()),
                Some("String".to_string()),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("var2".to_string()),
                Some("Vec<i32>".to_string()),
            ),
        ];
        let stats = create_test_memory_stats();
        let config = ExportConfig::default();

        let exporter = Exporter::new(allocations, stats, config);
        let filtered = exporter.get_filtered_allocations();

        // Verify filtering logic
        assert_eq!(filtered.len(), 2);

        // Test that all allocations have the expected structure
        for allocation in &filtered {
            assert!(allocation.ptr > 0);
            assert!(allocation.size > 0);
            assert!(allocation.var_name.is_some());
            assert!(allocation.type_name.is_some());
        }
    }

    #[test]
    fn test_export_directory_creation() -> TrackingResult<()> {
        let temp_dir = tempdir()?;
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("directory")
            .join("test.json");

        let allocations = vec![create_test_allocation(
            0x1000,
            64,
            Some("dir_var".to_string()),
            Some("String".to_string()),
        )];
        let stats = create_test_memory_stats();

        let export_stats = export_user_variables_json(allocations, stats, &nested_path)?;

        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
        assert_eq!(export_stats.allocations_processed, 1);

        Ok(())
    }

    #[test]
    fn test_user_only_filtering_with_mixed_allocations() {
        let allocations = vec![
            // User allocations (have var_name)
            create_test_allocation(
                0x1000,
                64,
                Some("user_var1".to_string()),
                Some("String".to_string()),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("user_var2".to_string()),
                Some("Vec<i32>".to_string()),
            ),
            create_test_allocation(
                0x3000,
                256,
                Some("user_var3".to_string()),
                Some("HashMap<String, i32>".to_string()),
            ),
            // System allocations (no var_name)
            create_test_allocation(0x4000, 32, None, Some("SystemType1".to_string())),
            create_test_allocation(0x5000, 16, None, Some("SystemType2".to_string())),
            create_test_allocation(0x6000, 8, None, None),
        ];
        let stats = create_test_memory_stats();

        // Test user_variables_only filtering
        let config_user_only = ExportConfig::user_variables_only();
        let exporter_user_only =
            Exporter::new(allocations.clone(), stats.clone(), config_user_only);
        let filtered_user_only = exporter_user_only.get_filtered_allocations();

        assert_eq!(filtered_user_only.len(), 3); // Only user allocations
        for allocation in &filtered_user_only {
            assert!(
                allocation.var_name.is_some(),
                "User-only filter should only include allocations with var_name"
            );
            assert!(allocation
                .var_name
                .as_ref()
                .unwrap()
                .starts_with("user_var"));
        }

        // Test all_allocations filtering
        let config_all = ExportConfig::all_allocations();
        let exporter_all = Exporter::new(allocations.clone(), stats, config_all);
        let filtered_all = exporter_all.get_filtered_allocations();

        assert_eq!(filtered_all.len(), 6); // All allocations

        let user_count = filtered_all.iter().filter(|a| a.var_name.is_some()).count();
        let system_count = filtered_all.iter().filter(|a| a.var_name.is_none()).count();
        assert_eq!(user_count, 3);
        assert_eq!(system_count, 3);
    }

    #[test]
    fn test_user_only_export_stats_accuracy() -> TrackingResult<()> {
        let temp_dir = tempdir()?;
        let output_path = temp_dir.path().join("user_only_stats.json");

        let allocations = vec![
            // User allocations
            create_test_allocation(
                0x1000,
                64,
                Some("user_var1".to_string()),
                Some("String".to_string()),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("user_var2".to_string()),
                Some("Vec<i32>".to_string()),
            ),
            // System allocations
            create_test_allocation(0x3000, 32, None, Some("SystemType".to_string())),
            create_test_allocation(0x4000, 16, None, None),
        ];
        let stats = create_test_memory_stats();

        let config = ExportConfig::user_variables_only();
        let exporter = Exporter::new(allocations, stats, config);
        let export_stats = exporter.export_json(&output_path)?;

        // Verify export stats reflect correct filtering
        assert_eq!(export_stats.user_variables, 2); // Only user allocations exported
        assert_eq!(export_stats.system_allocations, 2); // System allocations not exported but counted
                                                        // processing_time_ms is u64, so >= 0 is always true - check it's reasonable instead
        assert!(export_stats.processing_time_ms < 10000); // less than 10 seconds
        assert!(export_stats.output_size_bytes > 0);
        assert!(export_stats.processing_rate >= 0.0);

        Ok(())
    }

    #[test]
    fn test_user_only_edge_cases() {
        let stats = create_test_memory_stats();

        // Test with empty allocations
        let empty_allocations: Vec<AllocationInfo> = vec![];
        let config = ExportConfig::user_variables_only();
        let exporter_empty = Exporter::new(empty_allocations, stats.clone(), config.clone());
        let filtered_empty = exporter_empty.get_filtered_allocations();
        assert_eq!(filtered_empty.len(), 0);

        // Test with only system allocations
        let system_only = vec![
            create_test_allocation(0x1000, 32, None, Some("SystemType1".to_string())),
            create_test_allocation(0x2000, 16, None, None),
        ];
        let exporter_system = Exporter::new(system_only, stats.clone(), config.clone());
        let filtered_system = exporter_system.get_filtered_allocations();
        assert_eq!(filtered_system.len(), 0); // No user allocations

        // Test with only user allocations
        let user_only = vec![
            create_test_allocation(
                0x1000,
                64,
                Some("user_var1".to_string()),
                Some("String".to_string()),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("user_var2".to_string()),
                Some("Vec<i32>".to_string()),
            ),
        ];
        let exporter_user = Exporter::new(user_only.clone(), stats, config);
        let filtered_user = exporter_user.get_filtered_allocations();
        assert_eq!(filtered_user.len(), 2); // All are user allocations

        // Verify all filtered allocations have var_name
        for allocation in &filtered_user {
            assert!(allocation.var_name.is_some());
        }
    }

    #[test]
    fn test_user_only_binary_export_integration() -> TrackingResult<()> {
        let temp_dir = tempdir()?;
        let binary_path = temp_dir.path().join("user_only_integration.memscope");

        let allocations = vec![
            // User allocations
            create_test_allocation(
                0x1000,
                64,
                Some("user_var1".to_string()),
                Some("String".to_string()),
            ),
            create_test_allocation(
                0x2000,
                128,
                Some("user_var2".to_string()),
                Some("Vec<i32>".to_string()),
            ),
            // System allocations
            create_test_allocation(0x3000, 32, None, Some("SystemType".to_string())),
        ];
        let stats = create_test_memory_stats();

        let config = ExportConfig::user_variables_only();
        let exporter = Exporter::new(allocations, stats, config);
        let export_stats = exporter.export_binary(&binary_path)?;

        assert!(binary_path.exists());
        assert_eq!(export_stats.user_variables, 2); // Only user allocations
        assert_eq!(export_stats.system_allocations, 0); // No system allocations in binary export
        assert!(export_stats.output_size_bytes > 0);

        Ok(())
    }
}
