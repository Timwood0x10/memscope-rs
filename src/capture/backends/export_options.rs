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
