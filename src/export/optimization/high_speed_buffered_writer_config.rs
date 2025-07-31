//! Configuration for high speed buffered writer

/// Configuration for high speed buffered writer
#[derive(Debug, Clone)]
pub struct HighSpeedWriterConfig {
    /// Buffer size for writing operations
    pub buffer_size: usize,
    /// Whether to enable compression
    pub compression_enabled: bool,
}

impl Default for HighSpeedWriterConfig {
    fn default() -> Self {
        Self {
            buffer_size: 64 * 1024,
            compression_enabled: false,
        }
    }
}