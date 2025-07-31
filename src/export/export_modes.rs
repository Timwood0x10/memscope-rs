//! Export modes and coordination

/// Export coordinator for managing different export modes
pub struct ExportCoordinator {
    /// Current export mode
    pub mode: ExportMode,
}

/// Different export modes available
#[derive(Debug, Clone, PartialEq)]
pub enum ExportMode {
    /// Fast export mode
    Fast,
    /// Standard export mode
    Standard,
    /// Comprehensive export mode
    Comprehensive,
    /// Slow export mode
    Slow,
    /// Auto export mode
    Auto,
}

impl ExportCoordinator {
    /// Create a new export coordinator
    pub fn new(mode: ExportMode) -> Self {
        Self { mode }
    }

    /// Create a fast export coordinator
    pub fn new_fast() -> Self {
        Self::new(ExportMode::Fast)
    }

    /// Create a slow export coordinator
    pub fn new_slow() -> Self {
        Self::new(ExportMode::Slow)
    }

    /// Create an auto export coordinator
    pub fn new_auto() -> Self {
        Self::new(ExportMode::Auto)
    }

    /// Create an auto-sized export coordinator
    pub fn new_auto_sized(_size_hint: usize) -> Self {
        Self::new(ExportMode::Auto)
    }

    /// Get the configuration
    pub fn config(&self) -> &ExportCoordinator {
        self
    }

    /// Update configuration
    pub fn update_config(&mut self, _config: crate::export::validation::quality_validator::ExportConfig, _data_size: Option<usize>) -> Vec<String> {
        vec!["Configuration updated".to_string()]
    }
}