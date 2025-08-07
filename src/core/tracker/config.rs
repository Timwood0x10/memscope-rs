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
    /// ```rust
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
#[allow(dead_code)]
pub enum ExportMode {
    /// Fast mode: Only enrich user-tracked variables
    UserFocused,
    /// Complete mode: Enrich all allocations including system data
    Complete,
}
