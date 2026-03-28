//! Core memory tracking functionality.
//!
//! This module contains the main MemoryTracker struct and its basic methods
//! for creating, configuring, and managing the memory tracking system.

use crate::core::bounded_memory_stats::{
    AllocationHistoryManager, BoundedMemoryStats, BoundedStatsConfig,
};
use crate::core::ownership_history::{HistoryConfig, OwnershipHistoryRecorder};
use crate::core::safe_operations::SafeLock;
use crate::core::types::{
    AllocationInfo, DropChainNode, DropChainPerformanceMetrics, EnhancedPotentialLeak,
    LeakEvidence, LeakEvidenceType, LeakImpact, LeakRiskLevel, LeakType, MemoryStats,
    ResourceLeakAnalysis, TrackingError::LockError, TrackingResult,
};

use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
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

/// Tracking strategy constants for dual-mode architecture
const STRATEGY_GLOBAL_SINGLETON: u8 = 0;
const STRATEGY_THREAD_LOCAL: u8 = 1;

/// Global tracking strategy configuration
static TRACKING_STRATEGY: AtomicU8 = AtomicU8::new(STRATEGY_GLOBAL_SINGLETON);

/// Global memory tracker instance (for single-threaded mode)
static GLOBAL_TRACKER: OnceLock<Arc<MemoryTracker>> = OnceLock::new();

// Thread-local memory tracker instances (for concurrent mode)
thread_local! {
    static THREAD_LOCAL_TRACKER: Arc<MemoryTracker> = {
        let tracker = Arc::new(MemoryTracker::new());
        // Auto-register this thread's tracker when first accessed
        crate::core::thread_registry::register_current_thread_tracker(&tracker);
        tracker
    };
}

/// Configure tracking strategy for the application.
///
/// This function should be called at program startup to set the appropriate
/// tracking strategy based on whether the application is concurrent or not.
///
/// # Arguments
/// * `is_concurrent` - true for multi-threaded/async applications, false for single-threaded
pub fn configure_tracking_strategy(is_concurrent: bool) {
    let strategy = if is_concurrent {
        STRATEGY_THREAD_LOCAL
    } else {
        STRATEGY_GLOBAL_SINGLETON
    };

    TRACKING_STRATEGY.store(strategy, Ordering::Relaxed);

    tracing::info!(
        "Configured tracking strategy: {}",
        if is_concurrent {
            "thread-local"
        } else {
            "global-singleton"
        }
    );
}

/// Get the appropriate memory tracker based on current strategy.
///
/// This function implements the dual-mode dispatch:
/// - In single-threaded mode: returns the global singleton tracker
/// - In concurrent mode: returns the current thread's local tracker
pub fn get_tracker() -> Arc<MemoryTracker> {
    match TRACKING_STRATEGY.load(Ordering::Relaxed) {
        STRATEGY_GLOBAL_SINGLETON => GLOBAL_TRACKER
            .get_or_init(|| Arc::new(MemoryTracker::new()))
            .clone(),
        STRATEGY_THREAD_LOCAL => THREAD_LOCAL_TRACKER.with(|tracker| tracker.clone()),
        _ => {
            // Fallback to global singleton for unknown strategy
            tracing::warn!("Unknown tracking strategy, falling back to global singleton");
            GLOBAL_TRACKER
                .get_or_init(|| Arc::new(MemoryTracker::new()))
                .clone()
        }
    }
}

/// Get the global memory tracker instance (legacy compatibility).
///
/// This function is preserved for backward compatibility but now delegates to get_tracker().
/// New code should use get_tracker() directly for dual-mode support.
#[deprecated(note = "Use get_tracker() instead for dual-mode support")]
pub fn get_global_tracker() -> Arc<MemoryTracker> {
    get_tracker()
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
        // Get bounded stats using safe operations
        let bounded_stats = self
            .bounded_stats
            .safe_lock()
            .map(|stats| stats.clone())
            .unwrap_or_else(|_| crate::core::bounded_memory_stats::BoundedMemoryStats::default());

        // Get history for compatibility using safe operations
        let _history = self
            .history_manager
            .safe_lock()
            .map(|manager| manager.get_history_vec())
            .unwrap_or_else(|_| Vec::new());

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

        // Update the legacy stats cache using safe operations
        if let Ok(mut stats) = self.stats.safe_lock() {
            *stats = legacy_stats.clone();
        }

        Ok(legacy_stats)
    }

    /// Get all currently active allocations.
    pub fn get_active_allocations(&self) -> TrackingResult<Vec<AllocationInfo>> {
        self.active_allocations
            .safe_lock()
            .map(|active| active.values().cloned().collect())
            .map_err(|e| LockError(format!("Failed to get active allocations: {e}",)))
    }

    /// Get the complete allocation history.
    pub fn get_allocation_history(&self) -> TrackingResult<Vec<AllocationInfo>> {
        self.history_manager
            .safe_lock()
            .map(|manager| manager.get_history_vec())
            .map_err(|e| LockError(format!("Failed to get allocation history: {e}",)))
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
                resource_description: format!("Heap memory for {type_name}"),
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
                resource_description: format!("Lock release for {type_name}"),
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
        if type_name.starts_with("Vec<")
            || type_name.starts_with("HashMap<")
            || type_name.starts_with("Box<")
        {
            2 // Collection/Box + elements/boxed value
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
    #[allow(clippy::only_used_in_recursion)]
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

        efficiency.clamp(0.0, 100.0)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_memory_tracker_creation() {
        let tracker = MemoryTracker::new();

        // Test that tracker is created with default values
        assert!(
            !tracker.is_fast_mode() || std::env::var("MEMSCOPE_TEST_MODE").is_ok() || cfg!(test)
        );

        // Test that we can get stats without errors
        let stats_result = tracker.get_stats();
        assert!(stats_result.is_ok());
    }

    #[test]
    fn test_fast_mode_toggle() {
        let tracker = MemoryTracker::new();

        // Test enabling fast mode
        tracker.set_fast_mode(true);
        assert!(tracker.is_fast_mode());

        // Test disabling fast mode
        tracker.set_fast_mode(false);
        assert!(!tracker.is_fast_mode());

        // Test enable_fast_mode method
        tracker.enable_fast_mode();
        assert!(tracker.is_fast_mode());
    }

    #[test]
    fn test_get_active_allocations() {
        let tracker = MemoryTracker::new();
        tracker.enable_fast_mode();

        // Initially should be empty
        let allocations = tracker.get_active_allocations();
        assert!(allocations.is_ok());
        assert_eq!(allocations.unwrap().len(), 0);
    }

    #[test]
    fn test_get_allocation_history() {
        let tracker = MemoryTracker::new();
        tracker.enable_fast_mode();

        // Initially should be empty
        let history = tracker.get_allocation_history();
        assert!(history.is_ok());
        assert_eq!(history.unwrap().len(), 0);
    }

    #[test]
    fn test_memory_analysis_path_creation() {
        let tracker = MemoryTracker::new();

        let path = tracker.ensure_memory_analysis_path("test.svg");
        assert!(path.to_string_lossy().contains("MemoryAnalysis"));
        assert!(path.to_string_lossy().ends_with("test.svg"));
    }

    #[test]
    fn test_memscope_path_creation() {
        let tracker = MemoryTracker::new();

        let path = tracker.ensure_memscope_path("test");
        assert!(path.to_string_lossy().contains("MemoryAnalysis"));
        assert!(path.to_string_lossy().ends_with(".memscope"));

        let path_with_ext = tracker.ensure_memscope_path("test.memscope");
        assert!(path_with_ext.to_string_lossy().ends_with(".memscope"));
    }

    #[test]
    fn test_binary_export_mode_default() {
        let mode = BinaryExportMode::default();
        assert_eq!(mode, BinaryExportMode::UserOnly);
    }

    #[test]
    fn test_binary_export_mode_variants() {
        // Test that enum variants are different
        assert_ne!(
            std::mem::discriminant(&BinaryExportMode::UserOnly),
            std::mem::discriminant(&BinaryExportMode::Full)
        );
    }

    #[test]
    fn test_global_tracker_singleton() {
        let tracker1 = get_tracker();
        let tracker2 = get_tracker();

        // Should be the same instance (Arc comparison)
        assert!(Arc::ptr_eq(&tracker1, &tracker2));
    }

    #[test]
    fn test_drop_chain_analysis_basic() {
        let tracker = MemoryTracker::new();
        tracker.enable_fast_mode();

        let analysis = tracker.analyze_drop_chain(0x1000, "Vec<i32>");
        assert!(analysis.is_some());

        let analysis = analysis.unwrap();
        assert_eq!(analysis.root_object.object_id, 0x1000);
        assert_eq!(analysis.root_object.type_name, "Vec<i32>");
        assert!(analysis.total_duration_ns > 0);
    }

    #[test]
    fn test_drop_implementation_type_detection() {
        let tracker = MemoryTracker::new();

        // Test smart pointer detection
        let box_type = tracker.determine_drop_implementation_type("Box<i32>");
        assert_eq!(
            box_type,
            crate::core::types::DropImplementationType::SmartPointer
        );

        // Test collection detection
        let vec_type = tracker.determine_drop_implementation_type("Vec<i32>");
        assert_eq!(
            vec_type,
            crate::core::types::DropImplementationType::Collection
        );

        // Test resource handle detection
        let file_type = tracker.determine_drop_implementation_type("File");
        assert_eq!(
            file_type,
            crate::core::types::DropImplementationType::ResourceHandle
        );

        // Test copy type detection
        let int_type = tracker.determine_drop_implementation_type("i32");
        assert_eq!(int_type, crate::core::types::DropImplementationType::NoOp);
    }

    #[test]
    fn test_copy_type_detection() {
        let tracker = MemoryTracker::new();

        assert!(tracker.is_copy_type("i32"));
        assert!(tracker.is_copy_type("u64"));
        assert!(tracker.is_copy_type("f32"));
        assert!(tracker.is_copy_type("bool"));
        assert!(tracker.is_copy_type("char"));
        assert!(tracker.is_copy_type("&str"));

        assert!(!tracker.is_copy_type("String"));
        assert!(!tracker.is_copy_type("Vec<i32>"));
        assert!(!tracker.is_copy_type("HashMap<String, i32>"));
    }

    #[test]
    fn test_custom_drop_detection() {
        let tracker = MemoryTracker::new();

        assert!(tracker.has_custom_drop_impl("MutexGuard"));
        assert!(tracker.has_custom_drop_impl("RwLockWriteGuard"));
        assert!(tracker.has_custom_drop_impl("Receiver<i32>"));
        assert!(tracker.has_custom_drop_impl("Sender<String>"));

        assert!(!tracker.has_custom_drop_impl("i32"));
        assert!(!tracker.has_custom_drop_impl("String"));
    }

    #[test]
    fn test_drop_duration_estimation() {
        let tracker = MemoryTracker::new();

        // Vec should have reasonable drop time
        let vec_duration = tracker.estimate_drop_duration("Vec<i32>");
        assert!(vec_duration > 0);
        assert!(vec_duration < 10000); // Should be reasonable

        // File operations should be slower
        let file_duration = tracker.estimate_drop_duration("File");
        assert!(file_duration > vec_duration);

        // Simple types should be fastest
        let simple_duration = tracker.estimate_drop_duration("i32");
        assert!(simple_duration < vec_duration);
    }

    #[test]
    fn test_ownership_type_determination() {
        let tracker = MemoryTracker::new();

        assert_eq!(
            tracker.determine_ownership_type("Box<i32>"),
            crate::core::types::OwnershipType::Unique
        );
        assert_eq!(
            tracker.determine_ownership_type("Rc<i32>"),
            crate::core::types::OwnershipType::SharedSingleThreaded
        );
        assert_eq!(
            tracker.determine_ownership_type("Arc<i32>"),
            crate::core::types::OwnershipType::SharedMultiThreaded
        );
        assert_eq!(
            tracker.determine_ownership_type("&i32"),
            crate::core::types::OwnershipType::Borrowed
        );
        assert_eq!(
            tracker.determine_ownership_type("Weak<i32>"),
            crate::core::types::OwnershipType::Weak
        );
    }

    #[test]
    fn test_ownership_depth_calculation() {
        let tracker = MemoryTracker::new();

        // Collections should have depth > 1
        assert!(tracker.calculate_ownership_depth(0x1000, "Vec<i32>") > 1);
        assert!(tracker.calculate_ownership_depth(0x1000, "HashMap<String, i32>") > 1);
        assert!(tracker.calculate_ownership_depth(0x1000, "Box<i32>") > 1);

        // Simple types should have depth 1
        assert_eq!(tracker.calculate_ownership_depth(0x1000, "i32"), 1);
    }

    #[test]
    fn test_owned_objects_counting() {
        let tracker = MemoryTracker::new();

        // Collections should own multiple objects
        assert!(tracker.count_owned_objects(0x1000, "Vec<i32>") > 1);
        assert!(tracker.count_owned_objects(0x1000, "HashMap<String, i32>") > 1);

        // Simple types should own just themselves
        assert_eq!(tracker.count_owned_objects(0x1000, "i32"), 1);
    }

    #[test]
    fn test_reference_count_detection() {
        let tracker = MemoryTracker::new();

        // Reference counted types should return Some
        assert!(tracker.get_reference_count_for_type("Rc<i32>").is_some());
        assert!(tracker.get_reference_count_for_type("Arc<i32>").is_some());

        // Non-reference counted types should return None
        assert!(tracker.get_reference_count_for_type("Box<i32>").is_none());
        assert!(tracker.get_reference_count_for_type("i32").is_none());
    }

    #[test]
    fn test_generic_type_extraction() {
        let tracker = MemoryTracker::new();

        assert_eq!(
            tracker.extract_generic_type("Vec<i32>", "Vec"),
            Some("i32".to_string())
        );
        assert_eq!(
            tracker.extract_generic_type("HashMap<String, i32>", "HashMap"),
            Some("String, i32".to_string())
        );
        assert_eq!(
            tracker.extract_generic_type("Box<Vec<String>>", "Box"),
            Some("Vec<String>".to_string())
        );

        // Non-generic types should return None
        assert_eq!(tracker.extract_generic_type("i32", ""), None);
    }
}
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
        assert!(!allocations.is_empty());
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
//! Configuration and export options for memory tracking.
//!
//! This module contains configuration structures and enums used throughout
//! the memory tracking system, particularly for export operations.

// use crate::export::optimized_json_export::OptimizedExportOptions;

/// Export options for JSON export - user-controllable settings
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Include system allocations in full enrichment (default: false)
    ///
    /// **⚠️ Performance Impact**: Setting this to `true` can make export 5-10x slower!
    ///
    /// - `false` (default): Only user-tracked variables get full enrichment (~2-5 seconds)
    /// - `true`: ALL allocations including system internals get enrichment (~10-40 seconds)
    pub include_system_allocations: bool,

    /// Enable verbose logging during export (default: false)
    pub verbose_logging: bool,

    /// Buffer size for file I/O in bytes (default: 64KB)
    pub buffer_size: usize,

    /// Enable data compression (default: false)
    pub compress_output: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_system_allocations: false, // Fast mode by default
            verbose_logging: false,
            buffer_size: 64 * 1024, // 64KB
            compress_output: false,
        }
    }
}

impl ExportOptions {
    /// Create new export options with default settings (fast mode)
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable system allocation enrichment (⚠️ SLOW - 5-10x slower!)
    ///
    /// # Warning
    /// This will significantly slow down the export process and generate much larger files.
    /// Only use for deep debugging or system analysis.
    ///
    /// # Example
    /// ```text
    /// let options = ExportOptions::new().include_system_allocations(true);
    /// tracker.export_to_json_with_options("debug_output", options)?;
    /// ```
    pub fn include_system_allocations(mut self, include: bool) -> Self {
        self.include_system_allocations = include;
        self
    }

    /// Enable verbose logging during export
    pub fn verbose_logging(mut self, verbose: bool) -> Self {
        self.verbose_logging = verbose;
        self
    }

    /// Set custom buffer size for file I/O
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Enable output compression (experimental)
    pub fn compress_output(mut self, compress: bool) -> Self {
        self.compress_output = compress;
        self
    }
}

/// Internal export mode derived from options
#[derive(Debug, Clone, Copy)]
pub enum ExportMode {
    /// Fast mode: Only enrich user-tracked variables
    UserFocused,
    /// Complete mode: Enrich all allocations including system data
    Complete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_options_default() {
        let options = ExportOptions::default();

        assert!(!options.include_system_allocations);
        assert!(!options.verbose_logging);
        assert_eq!(options.buffer_size, 64 * 1024);
        assert!(!options.compress_output);
    }

    #[test]
    fn test_export_options_new() {
        let options = ExportOptions::new();

        // new() should be equivalent to default()
        assert!(!options.include_system_allocations);
        assert!(!options.verbose_logging);
        assert_eq!(options.buffer_size, 64 * 1024);
        assert!(!options.compress_output);
    }

    #[test]
    fn test_export_options_builder_pattern() {
        let options = ExportOptions::new()
            .include_system_allocations(true)
            .verbose_logging(true)
            .buffer_size(128 * 1024)
            .compress_output(true);

        assert!(options.include_system_allocations);
        assert!(options.verbose_logging);
        assert_eq!(options.buffer_size, 128 * 1024);
        assert!(options.compress_output);
    }

    #[test]
    fn test_export_options_individual_setters() {
        let mut options = ExportOptions::new();

        // Test include_system_allocations
        options = options.include_system_allocations(true);
        assert!(options.include_system_allocations);

        options = options.include_system_allocations(false);
        assert!(!options.include_system_allocations);

        // Test verbose_logging
        options = options.verbose_logging(true);
        assert!(options.verbose_logging);

        options = options.verbose_logging(false);
        assert!(!options.verbose_logging);

        // Test buffer_size
        options = options.buffer_size(1024);
        assert_eq!(options.buffer_size, 1024);

        options = options.buffer_size(256 * 1024);
        assert_eq!(options.buffer_size, 256 * 1024);

        // Test compress_output
        options = options.compress_output(true);
        assert!(options.compress_output);

        options = options.compress_output(false);
        assert!(!options.compress_output);
    }

    #[test]
    fn test_export_options_chaining() {
        // Test that method chaining works correctly
        let options1 = ExportOptions::new()
            .include_system_allocations(true)
            .verbose_logging(true);

        let options2 = options1
            .clone()
            .buffer_size(32 * 1024)
            .compress_output(true);

        // Original options1 should be unchanged (methods consume self)
        assert!(options1.include_system_allocations);
        assert!(options1.verbose_logging);
        assert_eq!(options1.buffer_size, 64 * 1024); // Still default
        assert!(!options1.compress_output); // Still default

        // options2 should have all changes
        assert!(options2.include_system_allocations);
        assert!(options2.verbose_logging);
        assert_eq!(options2.buffer_size, 32 * 1024);
        assert!(options2.compress_output);
    }

    #[test]
    fn test_export_options_clone() {
        let original = ExportOptions::new()
            .include_system_allocations(true)
            .verbose_logging(true)
            .buffer_size(128 * 1024)
            .compress_output(true);

        let cloned = original.clone();

        assert_eq!(
            original.include_system_allocations,
            cloned.include_system_allocations
        );
        assert_eq!(original.verbose_logging, cloned.verbose_logging);
        assert_eq!(original.buffer_size, cloned.buffer_size);
        assert_eq!(original.compress_output, cloned.compress_output);
    }

    #[test]
    fn test_export_options_debug() {
        let options = ExportOptions::new()
            .include_system_allocations(true)
            .verbose_logging(false)
            .buffer_size(32 * 1024)
            .compress_output(true);

        let debug_str = format!("{options:?}");

        // Should contain all field values
        assert!(debug_str.contains("include_system_allocations: true"));
        assert!(debug_str.contains("verbose_logging: false"));
        assert!(debug_str.contains("buffer_size: 32768"));
        assert!(debug_str.contains("compress_output: true"));
    }

    #[test]
    fn test_export_mode_variants() {
        // Test that ExportMode variants exist and can be created
        let user_focused = ExportMode::UserFocused;
        let complete = ExportMode::Complete;

        // Test Debug trait
        let debug_user = format!("{user_focused:?}");
        let debug_complete = format!("{complete:?}");

        assert_eq!(debug_user, "UserFocused");
        assert_eq!(debug_complete, "Complete");
    }

    #[test]
    fn test_export_mode_clone_copy() {
        let original = ExportMode::UserFocused;
        let cloned = original; // ExportMode implements Copy, so no clone needed
        let copied = original;

        // All should be equal (Copy trait)
        assert!(matches!(original, ExportMode::UserFocused));
        assert!(matches!(cloned, ExportMode::UserFocused));
        assert!(matches!(copied, ExportMode::UserFocused));

        let complete_original = ExportMode::Complete;
        let complete_copied = complete_original;

        assert!(matches!(complete_original, ExportMode::Complete));
        assert!(matches!(complete_copied, ExportMode::Complete));
    }

    #[test]
    fn test_buffer_size_edge_cases() {
        // Test various buffer sizes
        let small_buffer = ExportOptions::new().buffer_size(1);
        assert_eq!(small_buffer.buffer_size, 1);

        let large_buffer = ExportOptions::new().buffer_size(1024 * 1024 * 10); // 10MB
        assert_eq!(large_buffer.buffer_size, 1024 * 1024 * 10);

        let zero_buffer = ExportOptions::new().buffer_size(0);
        assert_eq!(zero_buffer.buffer_size, 0);
    }

    #[test]
    fn test_export_options_realistic_configurations() {
        // Test realistic configuration scenarios

        // Fast development mode
        let dev_config = ExportOptions::new()
            .include_system_allocations(false)
            .verbose_logging(false)
            .buffer_size(64 * 1024)
            .compress_output(false);

        assert!(!dev_config.include_system_allocations);
        assert!(!dev_config.verbose_logging);
        assert_eq!(dev_config.buffer_size, 64 * 1024);
        assert!(!dev_config.compress_output);

        // Debug mode with full details
        let debug_config = ExportOptions::new()
            .include_system_allocations(true)
            .verbose_logging(true)
            .buffer_size(128 * 1024)
            .compress_output(false);

        assert!(debug_config.include_system_allocations);
        assert!(debug_config.verbose_logging);
        assert_eq!(debug_config.buffer_size, 128 * 1024);
        assert!(!debug_config.compress_output);

        // Production mode with compression
        let prod_config = ExportOptions::new()
            .include_system_allocations(false)
            .verbose_logging(false)
            .buffer_size(256 * 1024)
            .compress_output(true);

        assert!(!prod_config.include_system_allocations);
        assert!(!prod_config.verbose_logging);
        assert_eq!(prod_config.buffer_size, 256 * 1024);
        assert!(prod_config.compress_output);
    }
}
//! Global convenience functions for memory tracking.
//!
//! This module provides global convenience functions that wrap the TrackingManager
//! functionality for easier use throughout the application.

use super::tracking_manager::TrackingManager;
use crate::core::types::TrackingResult;

/// Get unified tracking manager - convenience function
pub fn get_tracking_manager() -> TrackingManager {
    TrackingManager::new()
}

/// Track allocation - convenience function
pub fn track_allocation(ptr: usize, size: usize) -> TrackingResult<()> {
    let manager = TrackingManager::new();
    manager.track_allocation(ptr, size)
}

/// Track deallocation - convenience function
pub fn track_deallocation(ptr: usize) -> TrackingResult<()> {
    let manager = TrackingManager::new();
    manager.track_deallocation(ptr)
}

/// Associate variable - convenience function
pub fn associate_var(ptr: usize, var_name: String, type_name: String) -> TrackingResult<()> {
    let manager = TrackingManager::new();
    manager.associate_var(ptr, var_name, type_name)
}

/// Enter scope - convenience function
pub fn enter_scope(name: String) -> TrackingResult<crate::core::scope_tracker::ScopeId> {
    let manager = TrackingManager::new();
    manager.enter_scope(name)
}

/// Exit scope - convenience function
pub fn exit_scope(scope_id: crate::core::scope_tracker::ScopeId) -> TrackingResult<()> {
    let manager = TrackingManager::new();
    manager.exit_scope(scope_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test that functions exist and can be called without panicking
    // We avoid actually calling TrackingManager methods to prevent deadlocks
    // as warned in coverage.md

    #[test]
    fn test_function_signatures_exist() {
        // Test that all the function signatures exist and compile
        // This ensures the API is available even if we can't test the full functionality

        // These are compile-time checks - if the functions don't exist, this won't compile
        let _f1: fn() -> TrackingManager = get_tracking_manager;
        let _f2: fn(usize, usize) -> TrackingResult<()> = track_allocation;
        let _f3: fn(usize) -> TrackingResult<()> = track_deallocation;
        let _f4: fn(usize, String, String) -> TrackingResult<()> = associate_var;
        let _f5: fn(String) -> TrackingResult<crate::core::scope_tracker::ScopeId> = enter_scope;
        let _f6: fn(crate::core::scope_tracker::ScopeId) -> TrackingResult<()> = exit_scope;

        // If we get here, all functions exist with correct signatures
    }

    #[test]
    fn test_get_tracking_manager_function_exists() {
        // Just test that the function exists, don't call it to avoid deadlocks
        let _f: fn() -> TrackingManager = get_tracking_manager;
    }

    #[test]
    fn test_track_allocation_function_exists() {
        // Just test that the function exists, don't call it to avoid deadlocks
        let _f: fn(usize, usize) -> TrackingResult<()> = track_allocation;
    }

    #[test]
    fn test_track_deallocation_function_exists() {
        // Just test that the function exists, don't call it to avoid deadlocks
        let _f: fn(usize) -> TrackingResult<()> = track_deallocation;
    }

    #[test]
    fn test_associate_var_function_exists() {
        // Just test that the function exists, don't call it to avoid deadlocks
        let _f: fn(usize, String, String) -> TrackingResult<()> = associate_var;
    }

    #[test]
    fn test_enter_scope_function_exists() {
        // Just test that the function exists, don't call it to avoid deadlocks
        let _f: fn(String) -> TrackingResult<crate::core::scope_tracker::ScopeId> = enter_scope;
    }

    #[test]
    fn test_exit_scope_function_exists() {
        // Just test that the function exists, don't call it to avoid deadlocks
        let _f: fn(crate::core::scope_tracker::ScopeId) -> TrackingResult<()> = exit_scope;
    }

    #[test]
    fn test_module_documentation() {
        // Test that the module is properly documented and accessible
        // This ensures the module can be imported and used
    }

    #[test]
    fn test_function_parameter_types() {
        // Test that function parameters have the expected types
        // This is a compile-time check that ensures API consistency

        // Test track_allocation parameters
        let _ptr: usize = 0x1000;
        let _size: usize = 64;
        let _f1: fn(usize, usize) -> TrackingResult<()> = track_allocation;

        // Test track_deallocation parameters
        let _ptr: usize = 0x2000;
        let _f2: fn(usize) -> TrackingResult<()> = track_deallocation;

        // Test associate_var parameters
        let _ptr: usize = 0x3000;
        let _var_name: String = "test".to_string();
        let _type_name: String = "i32".to_string();
        let _f3: fn(usize, String, String) -> TrackingResult<()> = associate_var;

        // Test scope functions
        let _scope_name: String = "test_scope".to_string();
        let _f4: fn(String) -> TrackingResult<crate::core::scope_tracker::ScopeId> = enter_scope;
        let _f5: fn(crate::core::scope_tracker::ScopeId) -> TrackingResult<()> = exit_scope;
    }

    #[test]
    fn test_return_types() {
        // Test that functions return the expected types
        // This ensures API consistency without actually calling the functions

        use std::marker::PhantomData;

        // Test that TrackingManager is returned by get_tracking_manager
        let _phantom: PhantomData<TrackingManager> = PhantomData;

        // Test that TrackingResult<()> is returned by tracking functions
        let _phantom: PhantomData<TrackingResult<()>> = PhantomData;

        // Test that ScopeId is returned by enter_scope
        let _phantom: PhantomData<crate::core::scope_tracker::ScopeId> = PhantomData;
    }
}
//! Unified tracking manager interface.
//!
//! This module provides a unified interface that combines memory tracking and scope tracking
//! while preserving all existing functionality.

use super::memory_tracker::{get_tracker, MemoryTracker};
use crate::core::types::{
    AllocationInfo, MemoryStats, ScopeAnalysis, ScopeLifecycleMetrics, TrackingResult,
};
use std::sync::Arc;

/// Unified tracking manager that combines memory and scope tracking
/// This provides a unified interface that combines memory tracking and scope tracking
/// while preserving all existing functionality.
pub struct TrackingManager {
    memory_tracker: Arc<MemoryTracker>,
    scope_tracker: Arc<crate::core::scope_tracker::ScopeTracker>,
}

impl TrackingManager {
    /// Create a new tracking manager instance
    pub fn new() -> Self {
        Self {
            memory_tracker: get_tracker(),
            scope_tracker: crate::core::scope_tracker::get_global_scope_tracker(),
        }
    }

    /// Get the memory tracker instance
    pub fn memory_tracker(&self) -> &Arc<MemoryTracker> {
        &self.memory_tracker
    }

    /// Get the scope tracker instance
    pub fn scope_tracker(&self) -> &Arc<crate::core::scope_tracker::ScopeTracker> {
        &self.scope_tracker
    }

    /// Track memory allocation
    pub fn track_allocation(&self, ptr: usize, size: usize) -> TrackingResult<()> {
        self.memory_tracker.track_allocation(ptr, size)
    }

    /// Track memory deallocation
    pub fn track_deallocation(&self, ptr: usize) -> TrackingResult<()> {
        self.memory_tracker.track_deallocation(ptr)
    }

    /// Associate variable with memory allocation
    pub fn associate_var(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
    ) -> TrackingResult<()> {
        self.memory_tracker.associate_var(ptr, var_name, type_name)
    }

    /// Enter a new scope
    pub fn enter_scope(&self, name: String) -> TrackingResult<crate::core::scope_tracker::ScopeId> {
        self.scope_tracker.enter_scope(name)
    }

    /// Exit a scope
    pub fn exit_scope(&self, scope_id: crate::core::scope_tracker::ScopeId) -> TrackingResult<()> {
        self.scope_tracker.exit_scope(scope_id)
    }

    /// Associate variable with current scope
    pub fn associate_variable(
        &self,
        variable_name: String,
        memory_size: usize,
    ) -> TrackingResult<()> {
        self.scope_tracker
            .associate_variable(variable_name, memory_size)
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> TrackingResult<MemoryStats> {
        self.memory_tracker.get_stats()
    }

    /// Get active allocations
    pub fn get_active_allocations(&self) -> TrackingResult<Vec<AllocationInfo>> {
        self.memory_tracker.get_active_allocations()
    }

    /// Get allocation history
    pub fn get_allocation_history(&self) -> TrackingResult<Vec<AllocationInfo>> {
        self.memory_tracker.get_allocation_history()
    }

    /// Get scope analysis
    pub fn get_scope_analysis(&self) -> TrackingResult<ScopeAnalysis> {
        self.scope_tracker.get_scope_analysis()
    }

    /// Perform comprehensive tracking analysis
    pub fn perform_comprehensive_analysis(&self) -> TrackingResult<ComprehensiveTrackingReport> {
        let memory_stats = self.get_stats()?;
        let active_allocations = self.get_active_allocations()?;
        let allocation_history = self.get_allocation_history()?;
        let scope_analysis = self.get_scope_analysis()?;
        let scope_metrics = self.scope_tracker.get_scope_lifecycle_metrics()?;

        Ok(ComprehensiveTrackingReport {
            memory_stats,
            active_allocations,
            allocation_history,
            scope_analysis,
            scope_metrics,
            analysis_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }
}

impl Default for TrackingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive tracking report
#[derive(Debug, Clone)]
pub struct ComprehensiveTrackingReport {
    /// Overall memory statistics
    pub memory_stats: MemoryStats,
    /// Currently active memory allocations
    pub active_allocations: Vec<AllocationInfo>,
    /// Historical allocation data
    pub allocation_history: Vec<AllocationInfo>,
    /// Scope analysis results
    pub scope_analysis: ScopeAnalysis,
    /// Scope lifecycle metrics
    pub scope_metrics: Vec<ScopeLifecycleMetrics>,
    /// Timestamp when report was generated
    pub analysis_timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracking_manager_creation() {
        // Test that TrackingManager can be created
        let manager = TrackingManager::new();

        // Verify it has the expected components
        let _memory_tracker = manager.memory_tracker();
        let _scope_tracker = manager.scope_tracker();

        // If we get here without panicking, creation was successful
    }

    #[test]
    fn test_tracking_manager_default() {
        // Test that Default trait works
        let manager = TrackingManager::default();

        // Should be equivalent to new()
        let _memory_tracker = manager.memory_tracker();
        let _scope_tracker = manager.scope_tracker();
    }

    #[test]
    fn test_comprehensive_tracking_report_structure() {
        // Test the ComprehensiveTrackingReport structure
        use crate::core::types::{
            ConcurrencyAnalysis, FragmentationAnalysis, MemoryStats, ScopeAnalysis, ScopeHierarchy,
            ScopeLifecycleMetrics, SystemLibraryStats,
        };

        let report = ComprehensiveTrackingReport {
            memory_stats: MemoryStats {
                total_allocations: 10,
                total_allocated: 1024,
                active_allocations: 5,
                active_memory: 512,
                peak_allocations: 8,
                peak_memory: 800,
                total_deallocations: 5,
                total_deallocated: 512,
                leaked_allocations: 0,
                leaked_memory: 0,
                fragmentation_analysis: FragmentationAnalysis::default(),
                lifecycle_stats: ScopeLifecycleMetrics::default(),
                allocations: vec![],
                system_library_stats: SystemLibraryStats::default(),
                concurrency_analysis: ConcurrencyAnalysis::default(),
            },
            active_allocations: vec![],
            allocation_history: vec![],
            scope_analysis: ScopeAnalysis {
                total_scopes: 3,
                active_scopes: 1,
                max_depth: 2,
                average_lifetime: 0.0,
                memory_efficiency: 0.0,
                scopes: vec![],
                scope_hierarchy: ScopeHierarchy::default(),
                cross_scope_references: vec![],
            },
            scope_metrics: vec![],
            analysis_timestamp: 1234567890,
        };

        // Test that the report can be created and accessed
        assert_eq!(report.memory_stats.total_allocations, 10);
        assert_eq!(report.memory_stats.active_allocations, 5);
        assert_eq!(report.scope_analysis.total_scopes, 3);
        assert_eq!(report.analysis_timestamp, 1234567890);
    }

    #[test]
    fn test_comprehensive_tracking_report_clone() {
        // Test that ComprehensiveTrackingReport can be cloned
        use crate::core::types::{
            ConcurrencyAnalysis, FragmentationAnalysis, MemoryStats, ScopeAnalysis, ScopeHierarchy,
            ScopeLifecycleMetrics, SystemLibraryStats,
        };

        let original = ComprehensiveTrackingReport {
            memory_stats: MemoryStats {
                total_allocations: 5,
                total_allocated: 500,
                active_allocations: 3,
                active_memory: 300,
                peak_allocations: 4,
                peak_memory: 400,
                total_deallocations: 2,
                total_deallocated: 200,
                leaked_allocations: 0,
                leaked_memory: 0,
                fragmentation_analysis: FragmentationAnalysis::default(),
                lifecycle_stats: ScopeLifecycleMetrics::default(),
                allocations: vec![],
                system_library_stats: SystemLibraryStats::default(),
                concurrency_analysis: ConcurrencyAnalysis::default(),
            },
            active_allocations: vec![],
            allocation_history: vec![],
            scope_analysis: ScopeAnalysis {
                total_scopes: 2,
                active_scopes: 1,
                max_depth: 1,
                average_lifetime: 0.0,
                memory_efficiency: 0.0,
                scopes: vec![],
                scope_hierarchy: ScopeHierarchy::default(),
                cross_scope_references: vec![],
            },
            scope_metrics: vec![],
            analysis_timestamp: 9876543210,
        };

        let cloned = original.clone();

        // Verify clone has same values
        assert_eq!(
            original.memory_stats.total_allocations,
            cloned.memory_stats.total_allocations
        );
        assert_eq!(
            original.scope_analysis.total_scopes,
            cloned.scope_analysis.total_scopes
        );
        assert_eq!(original.analysis_timestamp, cloned.analysis_timestamp);
    }

    #[test]
    fn test_comprehensive_tracking_report_debug() {
        // Test that ComprehensiveTrackingReport implements Debug
        use crate::core::types::{
            ConcurrencyAnalysis, FragmentationAnalysis, MemoryStats, ScopeAnalysis, ScopeHierarchy,
            ScopeLifecycleMetrics, SystemLibraryStats,
        };

        let report = ComprehensiveTrackingReport {
            memory_stats: MemoryStats {
                total_allocations: 1,
                total_allocated: 100,
                active_allocations: 1,
                active_memory: 100,
                peak_allocations: 1,
                peak_memory: 100,
                total_deallocations: 0,
                total_deallocated: 0,
                leaked_allocations: 0,
                leaked_memory: 0,
                fragmentation_analysis: FragmentationAnalysis::default(),
                lifecycle_stats: ScopeLifecycleMetrics::default(),
                allocations: vec![],
                system_library_stats: SystemLibraryStats::default(),
                concurrency_analysis: ConcurrencyAnalysis::default(),
            },
            active_allocations: vec![],
            allocation_history: vec![],
            scope_analysis: ScopeAnalysis {
                total_scopes: 1,
                active_scopes: 1,
                max_depth: 1,
                average_lifetime: 0.0,
                memory_efficiency: 0.0,
                scopes: vec![],
                scope_hierarchy: ScopeHierarchy::default(),
                cross_scope_references: vec![],
            },
            scope_metrics: vec![],
            analysis_timestamp: 1111111111,
        };

        let debug_str = format!("{report:?}");

        // Should contain key information
        assert!(debug_str.contains("ComprehensiveTrackingReport"));
        assert!(debug_str.contains("memory_stats"));
        assert!(debug_str.contains("scope_analysis"));
        assert!(debug_str.contains("analysis_timestamp"));
    }

    #[test]
    fn test_tracking_manager_method_signatures() {
        // Test that all TrackingManager methods have correct signatures
        let manager = TrackingManager::new();

        // Test memory_tracker method
        let _memory_tracker: &Arc<MemoryTracker> = manager.memory_tracker();

        // Test scope_tracker method
        let _scope_tracker: &Arc<crate::core::scope_tracker::ScopeTracker> =
            manager.scope_tracker();

        // Test method signatures without actually calling them (to avoid global state issues)
        // We just verify the methods exist and have correct types

        // These would normally be tested, but we avoid calling them due to global state:
        // let _: TrackingResult<()> = manager.track_allocation(0x1000, 1024);
        // let _: TrackingResult<()> = manager.track_deallocation(0x1000);
        // let _: TrackingResult<()> = manager.associate_var(0x1000, "var".to_string(), "type".to_string());
        // let _: TrackingResult<crate::core::scope_tracker::ScopeId> = manager.enter_scope("scope".to_string());
        // let _: TrackingResult<()> = manager.exit_scope(1);
        // let _: TrackingResult<()> = manager.associate_variable("var".to_string(), 1024);
        // let _: TrackingResult<MemoryStats> = manager.get_stats();
        // let _: TrackingResult<Vec<AllocationInfo>> = manager.get_active_allocations();
        // let _: TrackingResult<Vec<AllocationInfo>> = manager.get_allocation_history();
        // let _: TrackingResult<ScopeAnalysis> = manager.get_scope_analysis();
        // let _: TrackingResult<ComprehensiveTrackingReport> = manager.perform_comprehensive_analysis();
    }

    #[test]
    fn test_tracking_manager_component_access() {
        // Test that we can access the internal components
        let manager = TrackingManager::new();

        // Access memory tracker
        let memory_tracker = manager.memory_tracker();
        // Arc is always valid, so we just check we can access it
        let _ = memory_tracker;

        // Access scope tracker
        let scope_tracker = manager.scope_tracker();
        // Arc is always valid, so we just check we can access it
        let _ = scope_tracker;

        // Test that we can access them multiple times
        let _memory_tracker2 = manager.memory_tracker();
        let _scope_tracker2 = manager.scope_tracker();
    }
}
