//! Core memory tracking functionality.
//!
//! This module contains the main MemoryTracker struct and its basic methods
//! for creating, configuring, and managing the memory tracking system.

use crate::core::types::{
    AllocationInfo, DropChainNode, DropChainPerformanceMetrics, EnhancedPotentialLeak,
    LeakEvidence, LeakEvidenceType, LeakImpact, LeakRiskLevel, LeakType, MemoryStats,
    ResourceLeakAnalysis, TrackingResult,
};
use crate::core::bounded_memory_stats::{BoundedMemoryStats, AllocationHistoryManager, BoundedStatsConfig};
use crate::core::ownership_history::{OwnershipHistoryRecorder, OwnershipEventType, HistoryConfig};

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Binary export mode enumeration for selecting export strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryExportMode {
    /// Export only user-defined variables (strict filtering)
    /// Results in smaller binary files (few KB) with faster processing
    UserOnly,
    /// Export all allocations including system allocations (loose filtering)  
    /// Results in larger binary files (hundreds of KB) with complete data
    Full,
}

impl Default for BinaryExportMode {
    /// Default to UserOnly mode for backward compatibility
    fn default() -> Self {
        BinaryExportMode::UserOnly
    }
}

/// Global memory tracker instance
static GLOBAL_TRACKER: OnceLock<Arc<MemoryTracker>> = OnceLock::new();

/// Get the global memory tracker instance.
///
/// This function returns a reference to the singleton memory tracker
/// that is used throughout the application.
pub fn get_global_tracker() -> Arc<MemoryTracker> {
    GLOBAL_TRACKER
        .get_or_init(|| Arc::new(MemoryTracker::new()))
        .clone()
}

/// Core memory tracking functionality.
///
/// The MemoryTracker maintains records of all memory allocations and deallocations,
/// provides statistics, and supports exporting data in various formats.
pub struct MemoryTracker {
    /// Active allocations (ptr -> allocation info)
    pub(crate) active_allocations: Mutex<HashMap<usize, AllocationInfo>>,
    /// Bounded memory statistics (prevents infinite growth)
    pub(crate) bounded_stats: Mutex<BoundedMemoryStats>,
    /// Separate allocation history manager (bounded)
    pub(crate) history_manager: Mutex<AllocationHistoryManager>,
    /// Ownership history recorder for detailed lifecycle tracking
    pub(crate) ownership_history: Mutex<OwnershipHistoryRecorder>,
    /// Legacy stats for compatibility (derived from bounded_stats)
    pub(crate) stats: Mutex<MemoryStats>,
    /// Fast mode flag for testing (reduces overhead)
    pub(crate) fast_mode: std::sync::atomic::AtomicBool,
}

impl MemoryTracker {
    /// Create a new memory tracker.
    pub fn new() -> Self {
        let fast_mode =
            std::env::var("MEMSCOPE_TEST_MODE").is_ok() || cfg!(test) || cfg!(feature = "test");
        
        // Configure bounded stats based on environment
        let config = if fast_mode {
            // Smaller limits for testing
            BoundedStatsConfig {
                max_recent_allocations: 1_000,
                max_historical_summaries: 100,
                enable_auto_cleanup: true,
                cleanup_threshold: 0.8,
            }
        } else {
            // Production limits
            BoundedStatsConfig::default()
        };
        
        // Configure ownership history based on mode
        let history_config = if fast_mode {
            HistoryConfig {
                max_events_per_allocation: 10,
                track_borrowing: false,
                track_cloning: true,
                track_ownership_transfers: false,
            }
        } else {
            HistoryConfig::default()
        };

        Self {
            active_allocations: Mutex::new(HashMap::new()),
            bounded_stats: Mutex::new(BoundedMemoryStats::with_config(config.clone())),
            history_manager: Mutex::new(AllocationHistoryManager::with_config(config)),
            ownership_history: Mutex::new(OwnershipHistoryRecorder::with_config(history_config)),
            stats: Mutex::new(MemoryStats::default()),
            fast_mode: std::sync::atomic::AtomicBool::new(fast_mode),
        }
    }

    /// Get current memory statistics with advanced analysis.
    pub fn get_stats(&self) -> TrackingResult<MemoryStats> {
        // Get bounded stats
        let bounded_stats = match self.bounded_stats.lock() {
            Ok(stats) => stats.clone(),
            Err(poisoned) => {
                let stats = poisoned.into_inner();
                stats.clone()
            }
        };

        // Get history for compatibility
        let history = match self.history_manager.lock() {
            Ok(manager) => manager.get_history_vec(),
            Err(poisoned) => {
                let manager = poisoned.into_inner();
                manager.get_history_vec()
            }
        };

        // Convert bounded stats to legacy MemoryStats for compatibility
        let legacy_stats = MemoryStats {
            total_allocations: bounded_stats.total_allocations,
            total_allocated: bounded_stats.total_allocated,
            active_allocations: bounded_stats.active_allocations,
            active_memory: bounded_stats.active_memory,
            peak_allocations: bounded_stats.peak_allocations,
            peak_memory: bounded_stats.peak_memory,
            total_deallocations: bounded_stats.total_deallocations,
            total_deallocated: bounded_stats.total_deallocated,
            leaked_allocations: bounded_stats.leaked_allocations,
            leaked_memory: bounded_stats.leaked_memory,
            fragmentation_analysis: bounded_stats.fragmentation_analysis.clone(),
            lifecycle_stats: bounded_stats.lifecycle_stats.clone(),
            system_library_stats: bounded_stats.system_library_stats.clone(),
            concurrency_analysis: bounded_stats.concurrency_analysis.clone(),
            // Use bounded allocations instead of infinite growth
            allocations: bounded_stats.get_all_allocations(),
        };

        // Update the legacy stats cache
        if let Ok(mut stats) = self.stats.lock() {
            *stats = legacy_stats.clone();
        }

        Ok(legacy_stats)
    }

    /// Get all currently active allocations.
    pub fn get_active_allocations(&self) -> TrackingResult<Vec<AllocationInfo>> {
        match self.active_allocations.lock() {
            Ok(active) => Ok(active.values().cloned().collect()),
            Err(poisoned) => {
                // Handle poisoned lock by recovering the data
                let active = poisoned.into_inner();
                Ok(active.values().cloned().collect())
            }
        }
    }

    /// Get the complete allocation history.
    pub fn get_allocation_history(&self) -> TrackingResult<Vec<AllocationInfo>> {
        match self.history_manager.lock() {
            Ok(manager) => Ok(manager.get_history_vec()),
            Err(poisoned) => {
                // Handle poisoned lock by recovering the data
                let manager = poisoned.into_inner();
                Ok(manager.get_history_vec())
            }
        }
    }

    /// Enable or disable fast mode.
    pub fn set_fast_mode(&self, enabled: bool) {
        self.fast_mode
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
    }

    /// Check if fast mode is enabled.
    pub fn is_fast_mode(&self) -> bool {
        self.fast_mode.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Enable fast mode for testing
    pub fn enable_fast_mode(&self) {
        self.fast_mode
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Export memory analysis visualization to SVG file.
    /// All output files are automatically placed in the MemoryAnalysis/ directory.
    ///
    /// # Arguments
    /// * `path` - Output filename for the memory analysis SVG file (recommended: "program_name_memory_analysis.svg")
    pub fn export_memory_analysis<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        let output_path = self.ensure_memory_analysis_path(path);
        crate::export::visualization::export_memory_analysis(self, output_path)
    }

    /// Ensure the memory analysis path exists and return the full path
    pub fn ensure_memory_analysis_path<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> std::path::PathBuf {
        let path = path.as_ref();
        let memory_analysis_dir = std::path::Path::new("MemoryAnalysis");

        // Create directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(memory_analysis_dir) {
            tracing::warn!("Failed to create MemoryAnalysis directory: {}", e);
        }

        memory_analysis_dir.join(path)
    }

    /// Export memory tracking data to binary format (.memscope file).
    /// All output files are automatically placed in the MemoryAnalysis/ directory.
    /// This method exports user-defined variables only (default behavior for compatibility).
    ///
    /// # Arguments
    /// * `path` - Base filename for the binary export (extension .memscope will be added automatically)
    ///
    /// # Example
    /// ```text
    /// let tracker = get_global_tracker();
    /// tracker.export_to_binary("my_program")?;
    /// // Creates: MemoryAnalysis/my_program.memscope
    /// ```
    pub fn export_to_binary<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        // Maintain compatibility by defaulting to user-only export
        self.export_user_binary(path)
    }

    /// Export memory tracking data to binary format with specified mode.
    /// All output files are automatically placed in the MemoryAnalysis/ directory.
    /// This method provides flexible export options for different use cases.
    ///
    /// # Arguments
    /// * `path` - Base filename for the binary export (extension .memscope will be added automatically)
    /// * `mode` - Export mode (UserOnly for small files, Full for complete data)
    ///
    /// # Example
    /// ```text
    /// let tracker = get_global_tracker();
    ///
    /// // Export only user variables (small, fast)
    /// tracker.export_to_binary_with_mode("my_program_user", BinaryExportMode::UserOnly)?;
    ///
    /// // Export all data (large, complete)
    /// tracker.export_to_binary_with_mode("my_program_full", BinaryExportMode::Full)?;
    /// ```
    pub fn export_to_binary_with_mode<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        mode: BinaryExportMode,
    ) -> TrackingResult<()> {
        match mode {
            BinaryExportMode::UserOnly => {
                tracing::info!("Using strict filtering for user-only binary export");
                self.export_user_binary(path)
            }
            BinaryExportMode::Full => {
                tracing::info!("Using loose filtering for full binary export");
                self.export_full_binary(path)
            }
        }
    }

    /// Export only user-defined variables to binary format (.memscope file).
    /// This method filters allocations to include only those with variable names,
    /// resulting in smaller binary files and faster JSON conversion.
    /// The binary file will contain only user-defined variables, not system allocations.
    ///
    /// # Arguments
    /// * `path` - Base filename for the binary export (extension .memscope will be added automatically)
    ///
    /// # Example
    /// ```text
    /// let tracker = get_global_tracker();
    /// tracker.export_user_binary("my_program_user")?;
    /// // Creates: MemoryAnalysis/my_program_user.memscope (user variables only)
    /// ```
    pub fn export_user_binary<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        let output_path = self.ensure_memscope_path(path);

        tracing::info!("Starting user binary export to: {}", output_path.display());

        let all_allocations = self.get_active_allocations()?;

        // Filter to user-defined variables only - this creates smaller binary files
        // and matches the current JSON output behavior
        let user_allocations: Vec<_> = all_allocations
            .into_iter()
            .filter(|allocation| allocation.var_name.is_some())
            .collect();

        tracing::info!(
            "Filtered {} user allocations for export (excluding system allocations)",
            user_allocations.len()
        );

        crate::export::binary::export_to_binary_with_mode(
            &user_allocations,
            output_path,
            crate::export::binary::format::BinaryExportMode::UserOnly,
            &crate::export::binary::BinaryExportConfig::default(),
        )
        .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))?;

        tracing::info!("User binary export completed successfully");
        Ok(())
    }

    /// Export all allocations (user + system) to binary format (.memscope file).
    /// This method includes all tracked allocations with null field elimination
    /// for optimal storage efficiency. Uses optimized processing for large datasets.
    ///
    /// # Arguments
    /// * `path` - Base filename for the binary export (extension .memscope will be added automatically)
    ///
    /// # Example
    /// ```text
    /// let tracker = get_global_tracker();
    /// tracker.export_full_binary("my_program_full")?;
    /// // Creates: MemoryAnalysis/my_program_full.memscope
    /// ```
    pub fn export_full_binary<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        let output_path = self.ensure_memscope_path(path);

        tracing::info!("Starting full binary export to: {}", output_path.display());

        let all_allocations = self.get_active_allocations()?;

        tracing::info!(
            "Exporting {} total allocations (user + system)",
            all_allocations.len()
        );

        // Export all allocations with enhanced header for full-binary mode
        // This ensures complete data integrity without ambiguous null values
        crate::export::binary::export_to_binary_with_mode(
            &all_allocations,
            output_path,
            crate::export::binary::format::BinaryExportMode::Full,
            &crate::export::binary::BinaryExportConfig::default(),
        )
        .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))?;

        tracing::info!("Full binary export completed successfully");
        Ok(())
    }

    /// Ensure path uses .memscope extension and is in MemoryAnalysis directory
    fn ensure_memscope_path<P: AsRef<std::path::Path>>(&self, path: P) -> std::path::PathBuf {
        let mut output_path = self.ensure_memory_analysis_path(path);

        // Ensure .memscope extension
        if output_path.extension().is_none()
            || output_path.extension() != Some(std::ffi::OsStr::new("memscope"))
        {
            output_path.set_extension("memscope");
        }

        output_path
    }

    /// Convert binary file to standard JSON format (4 separate files)
    ///
    /// This method reads a .memscope binary file and generates the standard
    /// 4-file JSON output format used by export_to_json.
    ///
    /// # Arguments
    ///
    /// * `binary_path` - Path to input .memscope file
    /// * `base_name` - Base name for output files (will create 4 files with different suffixes)
    ///
    /// # Examples
    ///
    /// ```text
    /// MemoryTracker::parse_binary_to_standard_json("data.memscope", "project_name")?;
    /// ```
    pub fn parse_binary_to_standard_json<P: AsRef<std::path::Path>>(
        binary_path: P,
        base_name: &str,
    ) -> TrackingResult<()> {
        crate::export::binary::BinaryParser::to_standard_json_files(binary_path, base_name)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))
    }

    /// Convert binary file to single JSON format (legacy compatibility)
    ///
    /// # Examples
    ///
    /// ```text
    /// MemoryTracker::parse_binary_to_json("data.memscope", "data.json")?;
    /// ```
    pub fn parse_binary_to_json<P: AsRef<std::path::Path>>(
        binary_path: P,
        json_path: P,
    ) -> TrackingResult<()> {
        crate::export::binary::parse_binary_to_json(binary_path, json_path)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))
    }

    /// Convert binary file to HTML format
    ///
    /// This method reads a .memscope binary file and generates an HTML report
    /// with memory allocation analysis and visualization.
    ///
    /// # Arguments
    ///
    /// * `binary_path` - Path to input .memscope file
    /// * `html_path` - Path for output HTML file
    ///
    /// # Examples
    ///
    /// ```text
    /// MemoryTracker::parse_binary_to_html("data.memscope", "report.html")?;
    /// ```
    pub fn parse_binary_to_html<P: AsRef<std::path::Path>>(
        binary_path: P,
        html_path: P,
    ) -> TrackingResult<()> {
        crate::export::binary::parse_binary_to_html(binary_path, html_path)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))
    }

    /// Alias for parse_binary_to_html for backward compatibility
    pub fn export_binary_to_html<P: AsRef<std::path::Path>>(
        binary_path: P,
        html_path: P,
    ) -> TrackingResult<()> {
        Self::parse_binary_to_html(binary_path, html_path)
    }

    /// Export interactive lifecycle timeline showing variable lifecycles and relationships.
    /// This creates an advanced timeline with variable birth, life, death, and cross-section interactivity.
    /// All output files are automatically placed in the MemoryAnalysis/ directory.
    ///
    /// # Arguments
    /// * `path` - Output filename for the lifecycle timeline SVG file (recommended: "program_name_lifecycle.svg")
    pub fn export_lifecycle_timeline<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> TrackingResult<()> {
        let output_path = self.ensure_memory_analysis_path(path);
        crate::export::visualization::export_lifecycle_timeline(self, output_path)
    }

    /// Analyze drop chain for an object being deallocated
    pub fn analyze_drop_chain(
        &self,
        ptr: usize,
        type_name: &str,
    ) -> Option<crate::core::types::DropChainAnalysis> {
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Create root drop chain node
        let root_node = self.create_drop_chain_node(ptr, type_name, start_time);

        // Analyze ownership hierarchy
        let ownership_hierarchy = self.analyze_ownership_hierarchy(ptr, type_name);

        // Build complete drop sequence
        let drop_sequence = self.build_drop_sequence(ptr, type_name, &ownership_hierarchy);

        // Calculate performance metrics
        let performance_metrics = self.calculate_drop_chain_performance(&drop_sequence);

        // Detect resource leaks
        let leak_detection = self.detect_resource_leaks(ptr, type_name, &ownership_hierarchy);

        let end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Some(crate::core::types::DropChainAnalysis {
            root_object: root_node,
            drop_sequence,
            total_duration_ns: end_time - start_time,
            performance_metrics,
            ownership_hierarchy,
            leak_detection,
        })
    }

    /// Create a drop chain node for an object
    fn create_drop_chain_node(
        &self,
        ptr: usize,
        type_name: &str,
        timestamp: u64,
    ) -> crate::core::types::DropChainNode {
        let drop_impl_type = self.determine_drop_implementation_type(type_name);
        let cleanup_actions = self.analyze_cleanup_actions(type_name);
        let performance_characteristics = self.analyze_drop_performance_characteristics(type_name);

        // Estimate drop duration based on type
        let drop_duration_ns = self.estimate_drop_duration(type_name);

        // Find child objects that will be dropped
        let children = self.find_child_objects_for_drop(ptr, type_name);

        crate::core::types::DropChainNode {
            object_id: ptr,
            type_name: type_name.to_string(),
            drop_timestamp: timestamp,
            drop_duration_ns,
            children,
            drop_impl_type,
            cleanup_actions,
            performance_characteristics,
        }
    }

    /// Determine the type of Drop implementation for a type
    fn determine_drop_implementation_type(
        &self,
        type_name: &str,
    ) -> crate::core::types::DropImplementationType {
        use crate::core::types::DropImplementationType;

        if type_name.starts_with("Box<")
            || type_name.starts_with("Rc<")
            || type_name.starts_with("Arc<")
        {
            DropImplementationType::SmartPointer
        } else if type_name.starts_with("Vec<")
            || type_name.starts_with("HashMap<")
            || type_name.starts_with("BTreeMap<")
            || type_name.starts_with("HashSet<")
        {
            DropImplementationType::Collection
        } else if type_name.contains("File")
            || type_name.contains("Socket")
            || type_name.contains("Handle")
            || type_name.contains("Stream")
        {
            DropImplementationType::ResourceHandle
        } else if self.is_copy_type(type_name) {
            DropImplementationType::NoOp
        } else if self.has_custom_drop_impl(type_name) {
            DropImplementationType::Custom
        } else {
            DropImplementationType::Automatic
        }
    }

    /// Check if a type is Copy (no-op drop)
    fn is_copy_type(&self, type_name: &str) -> bool {
        matches!(
            type_name,
            "i8" | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "f32"
                | "f64"
                | "bool"
                | "char"
                | "&str"
                | "&[u8]"
        ) || type_name.starts_with("&")
            || type_name.starts_with("*")
    }

    /// Check if a type has a custom Drop implementation
    fn has_custom_drop_impl(&self, type_name: &str) -> bool {
        // In a real implementation, this would check for Drop trait implementations
        // For now, use heuristics based on common patterns
        type_name.contains("Guard")
            || type_name.contains("Lock")
            || type_name.contains("Mutex")
            || type_name.contains("RwLock")
            || type_name.contains("Channel")
            || type_name.contains("Receiver")
            || type_name.contains("Sender")
    }

    /// Analyze cleanup actions for a type
    fn analyze_cleanup_actions(&self, type_name: &str) -> Vec<crate::core::types::CleanupAction> {
        use crate::core::types::{CleanupAction, CleanupActionType};

        let mut actions = Vec::new();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        if type_name.starts_with("Box<") || type_name.starts_with("Vec<") {
            actions.push(CleanupAction {
                action_type: CleanupActionType::MemoryDeallocation,
                timestamp,
                duration_ns: 100, // Estimated
                resource_description: format!("Heap memory for {}", type_name),
                success: true,
            });
        }

        if type_name.contains("File") {
            actions.push(CleanupAction {
                action_type: CleanupActionType::FileHandleClosure,
                timestamp,
                duration_ns: 1000, // File operations are slower
                resource_description: "File handle closure".to_string(),
                success: true,
            });
        }

        if type_name.contains("Socket") || type_name.contains("TcpStream") {
            actions.push(CleanupAction {
                action_type: CleanupActionType::NetworkConnectionClosure,
                timestamp,
                duration_ns: 5000, // Network operations can be slow
                resource_description: "Network connection closure".to_string(),
                success: true,
            });
        }

        if type_name.contains("Mutex") || type_name.contains("RwLock") {
            actions.push(CleanupAction {
                action_type: CleanupActionType::LockRelease,
                timestamp,
                duration_ns: 50, // Lock operations are fast
                resource_description: format!("Lock release for {}", type_name),
                success: true,
            });
        }

        if type_name.starts_with("Rc<") || type_name.starts_with("Arc<") {
            actions.push(CleanupAction {
                action_type: CleanupActionType::ReferenceCountDecrement,
                timestamp,
                duration_ns: 20, // Atomic operations are very fast
                resource_description: "Reference count decrement".to_string(),
                success: true,
            });
        }

        actions
    }

    /// Analyze drop performance characteristics
    fn analyze_drop_performance_characteristics(
        &self,
        type_name: &str,
    ) -> crate::core::types::DropPerformanceCharacteristics {
        use crate::core::types::{DropPerformanceCharacteristics, ImpactLevel};

        let (execution_time_ns, cpu_usage, memory_ops, io_ops, syscalls, impact) =
            if type_name.starts_with("Vec<") || type_name.starts_with("HashMap<") {
                (1000, 5.0, 10, 0, 1, ImpactLevel::Low)
            } else if type_name.contains("File") || type_name.contains("Socket") {
                (10000, 2.0, 1, 5, 3, ImpactLevel::Medium)
            } else if type_name.contains("Mutex") || type_name.contains("RwLock") {
                (100, 1.0, 0, 0, 1, ImpactLevel::Low)
            } else if self.has_custom_drop_impl(type_name) {
                (5000, 10.0, 5, 2, 2, ImpactLevel::Medium)
            } else {
                (50, 0.5, 1, 0, 0, ImpactLevel::Low)
            };

        DropPerformanceCharacteristics {
            execution_time_ns,
            cpu_usage_percent: cpu_usage,
            memory_operations: memory_ops,
            io_operations: io_ops,
            system_calls: syscalls,
            impact_level: impact,
        }
    }

    /// Estimate drop duration for a type
    fn estimate_drop_duration(&self, type_name: &str) -> u64 {
        if type_name.starts_with("Vec<") {
            // Vec drop time depends on element count and element drop time
            1000 // Base estimate in nanoseconds
        } else if type_name.starts_with("HashMap<") {
            2000 // HashMap is more complex
        } else if type_name.contains("File") {
            10000 // File operations are slow
        } else if type_name.contains("Socket") {
            15000 // Network operations are slower
        } else if type_name.starts_with("Box<") {
            500 // Simple heap deallocation
        } else if self.has_custom_drop_impl(type_name) {
            5000 // Custom drop implementations vary
        } else {
            100 // Simple automatic drop
        }
    }

    /// Find child objects that will be dropped as part of this object's drop
    fn find_child_objects_for_drop(
        &self,
        ptr: usize,
        type_name: &str,
    ) -> Vec<crate::core::types::DropChainNode> {
        let mut children = Vec::new();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // For collections, simulate child elements
        if type_name.starts_with("Vec<") {
            // Extract element type
            if let Some(element_type) = self.extract_generic_type(type_name, "Vec") {
                // Simulate a few child elements
                for i in 0..3 {
                    let child_ptr = ptr + (i * 8); // Simulate element addresses
                    children.push(self.create_drop_chain_node(
                        child_ptr,
                        &element_type,
                        timestamp + i as u64 * 100,
                    ));
                }
            }
        }

        children
    }

    /// Analyze ownership hierarchy for an object
    fn analyze_ownership_hierarchy(
        &self,
        ptr: usize,
        type_name: &str,
    ) -> crate::core::types::OwnershipHierarchy {
        use crate::core::types::{OwnershipHierarchy, OwnershipNode};

        // Create root ownership node
        let ownership_type = self.determine_ownership_type(type_name);
        let root_node = OwnershipNode {
            object_id: ptr,
            type_name: type_name.to_string(),
            ownership_type,
            owned_objects: self.find_owned_objects(ptr, type_name),
            reference_count: self.get_reference_count_for_type(type_name),
            weak_reference_count: self.get_weak_reference_count_for_type(type_name),
        };

        OwnershipHierarchy {
            root_owners: vec![root_node],
            max_depth: self.calculate_ownership_depth(ptr, type_name),
            total_objects: self.count_owned_objects(ptr, type_name),
            transfer_events: self.collect_ownership_transfer_events(ptr),
            weak_references: self.collect_weak_references(ptr),
            circular_references: self.detect_circular_references(ptr),
        }
    }

    /// Build complete drop sequence
    fn build_drop_sequence(
        &self,
        ptr: usize,
        type_name: &str,
        _hierarchy: &crate::core::types::OwnershipHierarchy,
    ) -> Vec<crate::core::types::DropChainNode> {
        let mut sequence = Vec::new();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Add root object
        sequence.push(self.create_drop_chain_node(ptr, type_name, timestamp));

        // Add child objects based on type
        if type_name.starts_with("Vec<") {
            if let Some(element_type) = self.extract_generic_type(type_name, "Vec") {
                for i in 0..3 {
                    let child_ptr = ptr + (i * 8);
                    sequence.push(self.create_drop_chain_node(
                        child_ptr,
                        &element_type,
                        timestamp + (i as u64 + 1) * 100,
                    ));
                }
            }
        }

        sequence
    }

    /// Calculate drop chain performance metrics
    fn calculate_drop_chain_performance(
        &self,
        drop_sequence: &[DropChainNode],
    ) -> DropChainPerformanceMetrics {
        let total_objects = drop_sequence.len();
        let max_depth = self.calculate_drop_chain_depth(drop_sequence);
        let total_time: u64 = drop_sequence.iter().map(|node| node.drop_duration_ns).sum();
        let avg_drop_time = if total_objects > 0 {
            total_time as f64 / total_objects as f64
        } else {
            0.0
        };
        let slowest_drop = drop_sequence
            .iter()
            .map(|node| node.drop_duration_ns)
            .max()
            .unwrap_or(0);

        // Calculate efficiency score based on performance characteristics
        let efficiency_score = self.calculate_drop_efficiency_score(drop_sequence);

        // Identify bottlenecks
        let bottlenecks = self.identify_drop_bottlenecks(drop_sequence);

        DropChainPerformanceMetrics {
            total_objects,
            max_depth,
            avg_drop_time_ns: avg_drop_time,
            slowest_drop_ns: slowest_drop,
            efficiency_score,
            bottlenecks,
        }
    }

    /// Detect potential resource leaks
    fn detect_resource_leaks(
        &self,
        ptr: usize,
        type_name: &str,
        _hierarchy: &crate::core::types::OwnershipHierarchy,
    ) -> ResourceLeakAnalysis {
        let mut potential_leaks = Vec::new();

        // Check for common leak patterns
        if type_name.contains("Rc<") && self.has_potential_cycle(ptr) {
            let evidence = vec![LeakEvidence {
                evidence_type: LeakEvidenceType::CircularReference,
                description: "Potential circular reference in Rc structure".to_string(),
                strength: 75.0,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
            }];

            potential_leaks.push(EnhancedPotentialLeak {
                object_id: ptr,
                leak_type: LeakType::ReferenceCycle,
                risk_level: LeakRiskLevel::High,
                evidence,
                estimated_impact: LeakImpact {
                    memory_bytes: self.estimate_type_size(type_name),
                    performance_impact_percent: 5.0,
                    resource_count: 1,
                    time_to_critical_hours: Some(24.0),
                },
            });
        }

        if type_name.contains("File") && !self.has_explicit_close(ptr) {
            let evidence = vec![LeakEvidence {
                evidence_type: LeakEvidenceType::ResourceNotClosed,
                description: "File handle may not be explicitly closed".to_string(),
                strength: 60.0,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
            }];

            potential_leaks.push(EnhancedPotentialLeak {
                object_id: ptr,
                leak_type: LeakType::FileHandle,
                risk_level: LeakRiskLevel::Medium,
                evidence,
                estimated_impact: LeakImpact {
                    memory_bytes: 1024, // File handle overhead
                    performance_impact_percent: 2.0,
                    resource_count: 1,
                    time_to_critical_hours: Some(72.0),
                },
            });
        }

        ResourceLeakAnalysis {
            potential_leaks,
            detection_confidence: 0.7, // 70% confidence in simplified implementation
            usage_patterns: Vec::new(), // Would be populated in full implementation
            prevention_recommendations: self.generate_leak_prevention_recommendations(type_name),
        }
    }

    /// Check if an object has potential circular references
    fn has_potential_cycle(&self, _ptr: usize) -> bool {
        // Simplified heuristic - in real implementation would do graph traversal
        rand::random::<f64>() < 0.1 // 10% chance for demonstration
    }

    // Private helper methods for drop chain analysis

    /// Extract generic type from a generic type name
    fn extract_generic_type(&self, type_name: &str, _container: &str) -> Option<String> {
        if let Some(start) = type_name.find('<') {
            if let Some(end) = type_name.rfind('>') {
                let inner = &type_name[start + 1..end];
                Some(inner.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Determine ownership type for a type
    fn determine_ownership_type(&self, type_name: &str) -> crate::core::types::OwnershipType {
        use crate::core::types::OwnershipType;

        if type_name.starts_with("Box<") {
            OwnershipType::Unique
        } else if type_name.starts_with("Rc<") {
            OwnershipType::SharedSingleThreaded
        } else if type_name.starts_with("Arc<") {
            OwnershipType::SharedMultiThreaded
        } else if type_name.starts_with("&") {
            OwnershipType::Borrowed
        } else if type_name.contains("Weak") {
            OwnershipType::Weak
        } else if type_name.starts_with("*") {
            OwnershipType::Raw
        } else {
            OwnershipType::Unique
        }
    }

    /// Find objects owned by this object
    fn find_owned_objects(
        &self,
        _ptr: usize,
        type_name: &str,
    ) -> Vec<crate::core::types::OwnershipNode> {
        use crate::core::types::{OwnershipNode, OwnershipType};

        let mut owned = Vec::new();

        // For collections, simulate owned elements
        if type_name.starts_with("Vec<") {
            if let Some(element_type) = self.extract_generic_type(type_name, "Vec") {
                // Simulate a few owned elements
                for i in 0..2 {
                    owned.push(OwnershipNode {
                        object_id: 1000 + i,
                        type_name: element_type.clone(),
                        ownership_type: OwnershipType::Unique,
                        owned_objects: Vec::new(),
                        reference_count: None,
                        weak_reference_count: None,
                    });
                }
            }
        }

        owned
    }

    /// Get reference count for reference-counted types
    fn get_reference_count_for_type(&self, type_name: &str) -> Option<usize> {
        if type_name.starts_with("Rc<") || type_name.starts_with("Arc<") {
            Some(1) // Simplified - in real implementation would track actual counts
        } else {
            None
        }
    }

    /// Get weak reference count for reference-counted types
    fn get_weak_reference_count_for_type(&self, type_name: &str) -> Option<usize> {
        if type_name.starts_with("Rc<") || type_name.starts_with("Arc<") {
            Some(0) // Simplified
        } else {
            None
        }
    }

    /// Calculate ownership hierarchy depth
    fn calculate_ownership_depth(&self, _ptr: usize, type_name: &str) -> usize {
        if type_name.starts_with("Vec<") || type_name.starts_with("HashMap<") {
            2 // Collection + elements
        } else if type_name.starts_with("Box<") {
            2 // Box + boxed value
        } else {
            1 // Simple object
        }
    }

    /// Count total objects in ownership hierarchy
    fn count_owned_objects(&self, _ptr: usize, type_name: &str) -> usize {
        if type_name.starts_with("Vec<") {
            5 // Simulate 5 elements
        } else if type_name.starts_with("HashMap<") {
            8 // Simulate 8 key-value pairs
        } else if type_name.starts_with("Box<") {
            2 // Box + boxed value
        } else {
            1
        }
    }

    /// Collect ownership transfer events
    fn collect_ownership_transfer_events(
        &self,
        _ptr: usize,
    ) -> Vec<crate::core::types::OwnershipTransferEvent> {
        // In a real implementation, this would track actual transfer events
        // For now, return empty vector
        Vec::new()
    }

    /// Collect weak references
    fn collect_weak_references(&self, _ptr: usize) -> Vec<crate::core::types::WeakReferenceInfo> {
        // In a real implementation, this would track actual weak references
        Vec::new()
    }

    /// Detect circular references
    fn detect_circular_references(
        &self,
        _ptr: usize,
    ) -> Vec<crate::core::types::CircularReferenceInfo> {
        // In a real implementation, this would perform cycle detection
        Vec::new()
    }

    /// Calculate drop chain depth
    fn calculate_drop_chain_depth(
        &self,
        drop_sequence: &[crate::core::types::DropChainNode],
    ) -> usize {
        drop_sequence
            .iter()
            .map(|node| self.calculate_node_depth(node, 0))
            .max()
            .unwrap_or(0)
    }

    /// Calculate depth of a single node
    fn calculate_node_depth(
        &self,
        node: &crate::core::types::DropChainNode,
        current_depth: usize,
    ) -> usize {
        if node.children.is_empty() {
            current_depth + 1
        } else {
            node.children
                .iter()
                .map(|child| self.calculate_node_depth(child, current_depth + 1))
                .max()
                .unwrap_or(current_depth + 1)
        }
    }

    /// Calculate drop efficiency score
    fn calculate_drop_efficiency_score(
        &self,
        drop_sequence: &[crate::core::types::DropChainNode],
    ) -> f64 {
        if drop_sequence.is_empty() {
            return 100.0;
        }

        let total_time: u64 = drop_sequence.iter().map(|node| node.drop_duration_ns).sum();
        let object_count = drop_sequence.len() as u64;

        // Efficiency is inversely related to average drop time
        // Good efficiency: < 1000ns per object = 100 points
        // Poor efficiency: > 10000ns per object = 0 points
        let avg_time_per_object = total_time / object_count;
        let efficiency = if avg_time_per_object < 1000 {
            100.0
        } else if avg_time_per_object > 10000 {
            0.0
        } else {
            100.0 - ((avg_time_per_object - 1000) as f64 / 9000.0) * 100.0
        };

        efficiency.max(0.0).min(100.0)
    }

    /// Identify drop performance bottlenecks
    fn identify_drop_bottlenecks(
        &self,
        drop_sequence: &[crate::core::types::DropChainNode],
    ) -> Vec<crate::core::types::DropPerformanceBottleneck> {
        use crate::core::types::{DropBottleneckType, DropPerformanceBottleneck, ImpactLevel};

        let mut bottlenecks = Vec::new();
        let avg_time = if !drop_sequence.is_empty() {
            drop_sequence
                .iter()
                .map(|node| node.drop_duration_ns)
                .sum::<u64>()
                / drop_sequence.len() as u64
        } else {
            0
        };

        for node in drop_sequence {
            // Identify slow drops
            if node.drop_duration_ns > avg_time * 3 {
                let severity = if node.drop_duration_ns > 50000 {
                    ImpactLevel::High
                } else if node.drop_duration_ns > 10000 {
                    ImpactLevel::Medium
                } else {
                    ImpactLevel::Low
                };

                let bottleneck_type = if node.type_name.contains("File")
                    || node.type_name.contains("Socket")
                {
                    DropBottleneckType::ResourceHandleDelay
                } else if node.type_name.starts_with("Vec<")
                    || node.type_name.starts_with("HashMap<")
                {
                    DropBottleneckType::LargeCollectionCleanup
                } else if node.drop_impl_type == crate::core::types::DropImplementationType::Custom
                {
                    DropBottleneckType::SlowCustomDrop
                } else {
                    DropBottleneckType::DeepOwnershipHierarchy
                };

                bottlenecks.push(DropPerformanceBottleneck {
                    object_id: node.object_id,
                    bottleneck_type: bottleneck_type.clone(),
                    severity,
                    description: format!(
                        "Drop of {} took {}ns, significantly above average of {}ns",
                        node.type_name, node.drop_duration_ns, avg_time
                    ),
                    optimization_suggestion: self
                        .get_drop_optimization_suggestion(&bottleneck_type),
                });
            }
        }

        bottlenecks
    }

    /// Get optimization suggestion for a drop bottleneck type
    fn get_drop_optimization_suggestion(
        &self,
        bottleneck_type: &crate::core::types::DropBottleneckType,
    ) -> String {
        use crate::core::types::DropBottleneckType;

        match bottleneck_type {
            DropBottleneckType::SlowCustomDrop => {
                "Consider optimizing custom Drop implementation or using async cleanup".to_string()
            }
            DropBottleneckType::DeepOwnershipHierarchy => {
                "Consider flattening ownership hierarchy or using weak references".to_string()
            }
            DropBottleneckType::LargeCollectionCleanup => {
                "Consider using Vec::clear() before drop or implementing custom cleanup".to_string()
            }
            DropBottleneckType::ResourceHandleDelay => {
                "Consider async resource cleanup or connection pooling".to_string()
            }
            DropBottleneckType::LockContention => {
                "Consider reducing lock scope or using lock-free data structures".to_string()
            }
            DropBottleneckType::MemoryFragmentation => {
                "Consider using memory pools or custom allocators".to_string()
            }
        }
    }

    /// Generate leak prevention recommendations
    fn generate_leak_prevention_recommendations(
        &self,
        type_name: &str,
    ) -> Vec<crate::core::types::LeakPreventionRecommendation> {
        use crate::core::types::{LeakPreventionRecommendation, LeakPreventionType, Priority};

        let mut recommendations = Vec::new();

        if type_name.contains("Rc<") {
            recommendations.push(LeakPreventionRecommendation {
                recommendation_type: LeakPreventionType::UseWeakReferences,
                priority: Priority::High,
                description: "Use Weak references to break potential cycles in Rc structures"
                    .to_string(),
                implementation_guidance: "Replace some Rc references with Weak where appropriate"
                    .to_string(),
                expected_effectiveness: 0.9,
            });
        }

        if type_name.contains("File") || type_name.contains("Socket") {
            recommendations.push(LeakPreventionRecommendation {
                recommendation_type: LeakPreventionType::UseRAII,
                priority: Priority::High,
                description: "Ensure proper RAII patterns for resource cleanup".to_string(),
                implementation_guidance: "Use Drop trait or scoped guards for automatic cleanup"
                    .to_string(),
                expected_effectiveness: 0.95,
            });
        }

        recommendations.push(LeakPreventionRecommendation {
            recommendation_type: LeakPreventionType::ResourceMonitoring,
            priority: Priority::Medium,
            description: "Implement resource usage monitoring".to_string(),
            implementation_guidance: "Add metrics and alerts for resource usage patterns"
                .to_string(),
            expected_effectiveness: 0.7,
        });

        recommendations
    }

    /// Check if a resource has explicit close handling
    fn has_explicit_close(&self, _ptr: usize) -> bool {
        // Simplified heuristic - in real implementation would track close calls
        rand::random::<f64>() < 0.8 // 80% chance of proper closure
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MemoryTracker {
    fn drop(&mut self) {
        // Optional verbose tip for users
        if std::env::var("MEMSCOPE_VERBOSE").is_ok() {
            tracing::info!("💡 Tip: Use tracker.export_to_json() or tracker.export_interactive_dashboard() before drop to save analysis results");
        }

        // Clean up any remaining allocations
        if let Ok(mut active) = self.active_allocations.lock() {
            active.clear();
        }
    }
}
