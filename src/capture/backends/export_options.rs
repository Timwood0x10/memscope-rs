//! Export options for memory tracking

/// Export options for JSON export - user-controllable settings
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Include system allocations in full enrichment (default: false)
    pub include_system_allocations: bool,
    /// Enable verbose logging during export (default: false)
    pub verbose_logging: bool,
    /// Buffer size for file I/O in bytes (default: 64KB)
    pub buffer_size: usize,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_system_allocations: false,
            verbose_logging: false,
            buffer_size: 64 * 1024,
        }
    }
}

impl ExportOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn include_system_allocations(mut self, include: bool) -> Self {
        self.include_system_allocations = include;
        self
    }

    pub fn verbose_logging(mut self, verbose: bool) -> Self {
        self.verbose_logging = verbose;
        self
    }

    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
}

/// Internal export mode derived from options
#[derive(Debug, Clone, Copy)]
pub enum ExportMode {
    UserFocused,
    Complete,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify ExportOptions creation with default values
    /// Invariants: Default should have include_system_allocations=false, verbose_logging=false, buffer_size=64KB
    #[test]
    fn test_export_options_default() {
        let options = ExportOptions::default();

        assert!(
            !options.include_system_allocations,
            "Default include_system_allocations should be false"
        );
        assert!(
            !options.verbose_logging,
            "Default verbose_logging should be false"
        );
        assert_eq!(
            options.buffer_size,
            64 * 1024,
            "Default buffer_size should be 64KB"
        );
    }

    /// Objective: Verify ExportOptions::new() creates default options
    /// Invariants: new() should return same as default()
    #[test]
    fn test_export_options_new() {
        let options = ExportOptions::new();
        let default_options = ExportOptions::default();

        assert_eq!(
            options.include_system_allocations, default_options.include_system_allocations,
            "new() should match default()"
        );
        assert_eq!(
            options.verbose_logging, default_options.verbose_logging,
            "new() should match default()"
        );
        assert_eq!(
            options.buffer_size, default_options.buffer_size,
            "new() should match default()"
        );
    }

    /// Objective: Verify include_system_allocations builder method
    /// Invariants: Should set include_system_allocations to true
    #[test]
    fn test_export_options_include_system_allocations() {
        let options = ExportOptions::new().include_system_allocations(true);

        assert!(
            options.include_system_allocations,
            "include_system_allocations should be true"
        );
    }

    /// Objective: Verify include_system_allocations can be set to false
    /// Invariants: Should set include_system_allocations to false
    #[test]
    fn test_export_options_exclude_system_allocations() {
        let options = ExportOptions::new().include_system_allocations(false);

        assert!(
            !options.include_system_allocations,
            "include_system_allocations should be false"
        );
    }

    /// Objective: Verify verbose_logging builder method
    /// Invariants: Should set verbose_logging to true
    #[test]
    fn test_export_options_verbose_logging() {
        let options = ExportOptions::new().verbose_logging(true);

        assert!(options.verbose_logging, "verbose_logging should be true");
    }

    /// Objective: Verify buffer_size builder method
    /// Invariants: Should set custom buffer size
    #[test]
    fn test_export_options_buffer_size() {
        let options = ExportOptions::new().buffer_size(128 * 1024);

        assert_eq!(
            options.buffer_size,
            128 * 1024,
            "buffer_size should be 128KB"
        );
    }

    /// Objective: Verify chained builder methods
    /// Invariants: All builder methods should work together
    #[test]
    fn test_export_options_chained_builders() {
        let options = ExportOptions::new()
            .include_system_allocations(true)
            .verbose_logging(true)
            .buffer_size(256 * 1024);

        assert!(
            options.include_system_allocations,
            "include_system_allocations should be true"
        );
        assert!(options.verbose_logging, "verbose_logging should be true");
        assert_eq!(
            options.buffer_size,
            256 * 1024,
            "buffer_size should be 256KB"
        );
    }

    /// Objective: Verify Clone trait implementation
    /// Invariants: Cloned options should have same values
    #[test]
    fn test_export_options_clone() {
        let original = ExportOptions::new()
            .include_system_allocations(true)
            .verbose_logging(true)
            .buffer_size(128 * 1024);

        let cloned = original.clone();

        assert_eq!(
            original.include_system_allocations, cloned.include_system_allocations,
            "Cloned include_system_allocations should match"
        );
        assert_eq!(
            original.verbose_logging, cloned.verbose_logging,
            "Cloned verbose_logging should match"
        );
        assert_eq!(
            original.buffer_size, cloned.buffer_size,
            "Cloned buffer_size should match"
        );
    }

    /// Objective: Verify Debug trait implementation
    /// Invariants: Debug output should contain field names
    #[test]
    fn test_export_options_debug() {
        let options = ExportOptions::new();
        let debug_str = format!("{:?}", options);

        assert!(
            debug_str.contains("ExportOptions"),
            "Debug should contain struct name"
        );
        assert!(
            debug_str.contains("include_system_allocations"),
            "Debug should contain field name"
        );
        assert!(
            debug_str.contains("verbose_logging"),
            "Debug should contain field name"
        );
        assert!(
            debug_str.contains("buffer_size"),
            "Debug should contain field name"
        );
    }

    /// Objective: Verify ExportMode variants
    /// Invariants: All variants should have debug representation
    #[test]
    fn test_export_mode_variants() {
        let user_focused = ExportMode::UserFocused;
        let complete = ExportMode::Complete;

        let debug_focused = format!("{:?}", user_focused);
        let debug_complete = format!("{:?}", complete);

        assert!(
            debug_focused.contains("UserFocused"),
            "Debug should contain UserFocused"
        );
        assert!(
            debug_complete.contains("Complete"),
            "Debug should contain Complete"
        );
    }

    /// Objective: Verify ExportMode Clone trait
    /// Invariants: Cloned mode should be equal
    #[test]
    fn test_export_mode_clone() {
        let original = ExportMode::Complete;
        let cloned = original;

        assert!(
            matches!(cloned, ExportMode::Complete),
            "Cloned mode should be Complete"
        );
    }

    /// Objective: Verify ExportMode Copy trait
    /// Invariants: Copy should work implicitly
    #[test]
    fn test_export_mode_copy() {
        let original = ExportMode::UserFocused;
        let copied = original;

        assert!(
            matches!(original, ExportMode::UserFocused),
            "Original should still be valid after copy"
        );
        assert!(
            matches!(copied, ExportMode::UserFocused),
            "Copied should be UserFocused"
        );
    }

    /// Objective: Verify zero buffer_size
    /// Invariants: Should accept zero buffer size
    #[test]
    fn test_export_options_zero_buffer_size() {
        let options = ExportOptions::new().buffer_size(0);

        assert_eq!(options.buffer_size, 0, "buffer_size should be 0");
    }

    /// Objective: Verify large buffer_size
    /// Invariants: Should accept large buffer size
    #[test]
    fn test_export_options_large_buffer_size() {
        let large_size = 1024 * 1024 * 1024;
        let options = ExportOptions::new().buffer_size(large_size);

        assert_eq!(options.buffer_size, large_size, "buffer_size should be 1GB");
    }

    /// Objective: Verify builder overwrites previous value
    /// Invariants: Later builder call should override earlier
    #[test]
    fn test_export_options_builder_override() {
        let options = ExportOptions::new().buffer_size(1024).buffer_size(2048);

        assert_eq!(
            options.buffer_size, 2048,
            "Later buffer_size should override earlier"
        );
    }
}
