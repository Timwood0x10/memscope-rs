//! Core types for memory tracking.
//!
//! This module contains type definitions used throughout the core tracking system.

use std::sync::atomic::AtomicU8;

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
#[allow(dead_code)]
const STRATEGY_THREAD_LOCAL: u8 = 1;

/// Global tracking strategy configuration
pub static TRACKING_STRATEGY: AtomicU8 = AtomicU8::new(STRATEGY_GLOBAL_SINGLETON);

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
    fn test_binary_export_mode_default() {
        let mode = BinaryExportMode::default();
        assert_eq!(mode, BinaryExportMode::UserOnly);
    }

    #[test]
    fn test_export_options_default() {
        let options = ExportOptions::default();

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
    fn test_export_mode_variants() {
        let user_focused = ExportMode::UserFocused;
        let complete = ExportMode::Complete;

        let debug_user = format!("{user_focused:?}");
        let debug_complete = format!("{complete:?}");

        assert_eq!(debug_user, "UserFocused");
        assert_eq!(debug_complete, "Complete");
    }
}
