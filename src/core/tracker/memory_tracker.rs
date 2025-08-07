//! Core memory tracking functionality.
//!
//! This module contains the main MemoryTracker struct and its basic methods
//! for creating, configuring, and managing the memory tracking system.

use crate::core::types::{AllocationInfo, MemoryStats, TrackingResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

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
    /// Complete allocation history (for analysis)
    pub(crate) allocation_history: Mutex<Vec<AllocationInfo>>,
    /// Memory usage statistics
    pub(crate) stats: Mutex<MemoryStats>,
    /// Fast mode flag for testing (reduces overhead)
    pub(crate) fast_mode: std::sync::atomic::AtomicBool,
}

impl MemoryTracker {
    /// Create a new memory tracker.
    pub fn new() -> Self {
        let fast_mode =
            std::env::var("MEMSCOPE_TEST_MODE").is_ok() || cfg!(test) || cfg!(feature = "test");
        Self {
            active_allocations: Mutex::new(HashMap::new()),
            allocation_history: Mutex::new(Vec::new()),
            stats: Mutex::new(MemoryStats::default()),
            fast_mode: std::sync::atomic::AtomicBool::new(fast_mode),
        }
    }

    /// Get current memory statistics with advanced analysis.
    pub fn get_stats(&self) -> TrackingResult<MemoryStats> {
        let base_stats = match self.stats.lock() {
            Ok(stats) => stats.clone(),
            Err(poisoned) => {
                // Handle poisoned lock by recovering the data
                let stats = poisoned.into_inner();
                stats.clone()
            }
        };

        // For now, return the base stats directly
        // TODO: Add advanced analysis like in the original implementation
        Ok(base_stats)
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
        match self.allocation_history.lock() {
            Ok(history) => Ok(history.clone()),
            Err(poisoned) => {
                // Handle poisoned lock by recovering the data
                let history = poisoned.into_inner();
                Ok(history.clone())
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
    ///
    /// # Arguments
    /// * `path` - Base filename for the binary export (extension .memscope will be added automatically)
    ///
    /// # Example
    /// ```rust
    /// let tracker = get_global_tracker();
    /// tracker.export_to_binary("my_program")?;
    /// // Creates: MemoryAnalysis/my_program.memscope
    /// ```
    pub fn export_to_binary<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        let output_path = self.ensure_memscope_path(path);

        tracing::info!("Starting binary export to: {}", output_path.display());

        let allocations = self.get_active_allocations()?;

        crate::export::binary::export_to_binary(&allocations, output_path)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))?;

        tracing::info!("Binary export completed successfully");
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
    /// ```rust
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
    /// ```rust
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
    /// ```rust
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

    /// Analyze drop chain for an object being deallocated (simplified implementation)
    pub fn analyze_drop_chain(
        &self,
        _ptr: usize,
        _type_name: &str,
    ) -> Option<crate::core::types::DropChainAnalysis> {
        // Simplified implementation - return None for now
        // The full implementation would be much more complex and requires
        // additional type definitions that may not be available
        None
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
