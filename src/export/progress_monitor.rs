//! Progress monitoring for export operations

/// Configuration for progress monitoring
#[derive(Debug, Clone)]
pub struct ProgressConfig {
    /// Whether to enable progress reporting
    pub enabled: bool,
    /// Update interval in milliseconds
    pub update_interval_ms: u64,
    /// Update interval as Duration
    pub update_interval: std::time::Duration,
    /// Whether to show details
    pub show_details: bool,
    /// Whether to show estimated time
    pub show_estimated_time: bool,
    /// Whether to allow cancellation
    pub allow_cancellation: bool,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            update_interval_ms: 1000,
            update_interval: std::time::Duration::from_millis(1000),
            show_details: false,
            show_estimated_time: false,
            allow_cancellation: false,
        }
    }
}