//! Common types and utilities for the data model layer

use serde::{Deserialize, Serialize};

/// Tracking strategy enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrackingStrategy {
    /// Core strategy - detailed allocation tracking
    Core,
    /// Lockfree strategy - high-performance event tracking
    Lockfree,
    /// Async strategy - task-level memory tracking
    Async,
    /// Unified strategy - combines all strategies
    Unified,
}

impl std::fmt::Display for TrackingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackingStrategy::Core => write!(f, "Core"),
            TrackingStrategy::Lockfree => write!(f, "Lockfree"),
            TrackingStrategy::Async => write!(f, "Async"),
            TrackingStrategy::Unified => write!(f, "Unified"),
        }
    }
}

/// Memory event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// Memory allocation
    Alloc,
    /// Memory deallocation
    Dealloc,
    /// Memory reallocation
    Realloc,
    /// Task spawn (async)
    TaskSpawn,
    /// Task end (async)
    TaskEnd,
    /// FFI allocation
    FfiAlloc,
    /// FFI free
    FfiFree,
}

/// Export format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// Binary format
    Binary,
    /// HTML format
    Html,
    /// CSV format
    Csv,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Json => write!(f, "JSON"),
            ExportFormat::Binary => write!(f, "Binary"),
            ExportFormat::Html => write!(f, "HTML"),
            ExportFormat::Csv => write!(f, "CSV"),
        }
    }
}

/// Render output
#[derive(Debug, Clone)]
pub enum RenderOutput {
    /// String output (JSON, HTML, CSV)
    String(String),
    /// Binary output
    Bytes(Vec<u8>),
    /// File output
    File(std::path::PathBuf),
}

/// Result type for render operations
pub type RenderResult<T> = Result<T, crate::error::types::MemScopeError>;

/// Get current timestamp in nanoseconds since Unix epoch
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Get current thread ID
pub fn current_thread_id() -> u32 {
    // Use a hash of the thread ID to create a stable numeric ID
    use std::hash::{Hash, Hasher};
    let thread_id = std::thread::current().id();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    thread_id.hash(&mut hasher);
    (hasher.finish() & 0xFFFFFFFF) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracking_strategy_display() {
        assert_eq!(TrackingStrategy::Core.to_string(), "Core");
        assert_eq!(TrackingStrategy::Lockfree.to_string(), "Lockfree");
        assert_eq!(TrackingStrategy::Async.to_string(), "Async");
        assert_eq!(TrackingStrategy::Unified.to_string(), "Unified");
    }

    #[test]
    fn test_export_format_display() {
        assert_eq!(ExportFormat::Json.to_string(), "JSON");
        assert_eq!(ExportFormat::Binary.to_string(), "Binary");
        assert_eq!(ExportFormat::Html.to_string(), "HTML");
        assert_eq!(ExportFormat::Csv.to_string(), "CSV");
    }

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 0);
    }

    #[test]
    fn test_current_thread_id() {
        let id = current_thread_id();
        assert!(id > 0);
    }
}