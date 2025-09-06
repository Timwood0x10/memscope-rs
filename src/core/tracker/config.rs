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
#[allow(dead_code)]
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
        
        assert_eq!(options.include_system_allocations, false);
        assert_eq!(options.verbose_logging, false);
        assert_eq!(options.buffer_size, 64 * 1024);
        assert_eq!(options.compress_output, false);
    }

    #[test]
    fn test_export_options_new() {
        let options = ExportOptions::new();
        
        // new() should be equivalent to default()
        assert_eq!(options.include_system_allocations, false);
        assert_eq!(options.verbose_logging, false);
        assert_eq!(options.buffer_size, 64 * 1024);
        assert_eq!(options.compress_output, false);
    }

    #[test]
    fn test_export_options_builder_pattern() {
        let options = ExportOptions::new()
            .include_system_allocations(true)
            .verbose_logging(true)
            .buffer_size(128 * 1024)
            .compress_output(true);
        
        assert_eq!(options.include_system_allocations, true);
        assert_eq!(options.verbose_logging, true);
        assert_eq!(options.buffer_size, 128 * 1024);
        assert_eq!(options.compress_output, true);
    }

    #[test]
    fn test_export_options_individual_setters() {
        let mut options = ExportOptions::new();
        
        // Test include_system_allocations
        options = options.include_system_allocations(true);
        assert_eq!(options.include_system_allocations, true);
        
        options = options.include_system_allocations(false);
        assert_eq!(options.include_system_allocations, false);
        
        // Test verbose_logging
        options = options.verbose_logging(true);
        assert_eq!(options.verbose_logging, true);
        
        options = options.verbose_logging(false);
        assert_eq!(options.verbose_logging, false);
        
        // Test buffer_size
        options = options.buffer_size(1024);
        assert_eq!(options.buffer_size, 1024);
        
        options = options.buffer_size(256 * 1024);
        assert_eq!(options.buffer_size, 256 * 1024);
        
        // Test compress_output
        options = options.compress_output(true);
        assert_eq!(options.compress_output, true);
        
        options = options.compress_output(false);
        assert_eq!(options.compress_output, false);
    }

    #[test]
    fn test_export_options_chaining() {
        // Test that method chaining works correctly
        let options1 = ExportOptions::new()
            .include_system_allocations(true)
            .verbose_logging(true);
        
        let options2 = options1.clone()
            .buffer_size(32 * 1024)
            .compress_output(true);
        
        // Original options1 should be unchanged (methods consume self)
        assert_eq!(options1.include_system_allocations, true);
        assert_eq!(options1.verbose_logging, true);
        assert_eq!(options1.buffer_size, 64 * 1024); // Still default
        assert_eq!(options1.compress_output, false); // Still default
        
        // options2 should have all changes
        assert_eq!(options2.include_system_allocations, true);
        assert_eq!(options2.verbose_logging, true);
        assert_eq!(options2.buffer_size, 32 * 1024);
        assert_eq!(options2.compress_output, true);
    }

    #[test]
    fn test_export_options_clone() {
        let original = ExportOptions::new()
            .include_system_allocations(true)
            .verbose_logging(true)
            .buffer_size(128 * 1024)
            .compress_output(true);
        
        let cloned = original.clone();
        
        assert_eq!(original.include_system_allocations, cloned.include_system_allocations);
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
        
        let debug_str = format!("{:?}", options);
        
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
        let debug_user = format!("{:?}", user_focused);
        let debug_complete = format!("{:?}", complete);
        
        assert_eq!(debug_user, "UserFocused");
        assert_eq!(debug_complete, "Complete");
    }

    #[test]
    fn test_export_mode_clone_copy() {
        let original = ExportMode::UserFocused;
        let cloned = original.clone();
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
        
        assert_eq!(dev_config.include_system_allocations, false);
        assert_eq!(dev_config.verbose_logging, false);
        assert_eq!(dev_config.buffer_size, 64 * 1024);
        assert_eq!(dev_config.compress_output, false);
        
        // Debug mode with full details
        let debug_config = ExportOptions::new()
            .include_system_allocations(true)
            .verbose_logging(true)
            .buffer_size(128 * 1024)
            .compress_output(false);
        
        assert_eq!(debug_config.include_system_allocations, true);
        assert_eq!(debug_config.verbose_logging, true);
        assert_eq!(debug_config.buffer_size, 128 * 1024);
        assert_eq!(debug_config.compress_output, false);
        
        // Production mode with compression
        let prod_config = ExportOptions::new()
            .include_system_allocations(false)
            .verbose_logging(false)
            .buffer_size(256 * 1024)
            .compress_output(true);
        
        assert_eq!(prod_config.include_system_allocations, false);
        assert_eq!(prod_config.verbose_logging, false);
        assert_eq!(prod_config.buffer_size, 256 * 1024);
        assert_eq!(prod_config.compress_output, true);
    }
}
