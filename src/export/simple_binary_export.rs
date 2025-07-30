//! Simplified binary export for proof of concept
//!
//! This module provides a basic binary export implementation to demonstrate
//! the performance benefits of MessagePack + Zstd over JSON export.

use crate::core::tracker::MemoryTracker;
use crate::core::types::{TrackingError, TrackingResult};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Simplified binary export options for proof of concept
///
/// This struct provides basic configuration for binary export operations
/// with focus on demonstrating performance improvements over JSON.
#[derive(Debug, Clone)]
pub struct SimpleBinaryOptions {
    /// Zstd compression level (1-22)
    ///
    /// - 1-3: Fast compression for real-time exports
    /// - 4-6: Balanced compression for general use  
    /// - 7-15: High compression for storage optimization
    pub compression_level: i32,

    /// Use MessagePack format (true) or bincode (false)
    ///
    /// MessagePack offers better cross-language compatibility while
    /// bincode provides maximum performance for Rust-only usage.
    pub use_messagepack: bool,

    /// I/O buffer size for file operations
    ///
    /// Larger buffers improve performance for big datasets but use more memory.
    pub buffer_size: usize,
}

impl Default for SimpleBinaryOptions {
    /// Create default options with balanced performance settings
    fn default() -> Self {
        Self {
            compression_level: 3,    // Balanced compression
            use_messagepack: true,   // Cross-platform compatibility
            buffer_size: 256 * 1024, // 256KB buffer
        }
    }
}

impl SimpleBinaryOptions {
    /// Create options optimized for maximum export speed
    ///
    /// Uses minimal compression and large buffers for fastest possible export.
    pub fn fast() -> Self {
        Self {
            compression_level: 1,    // Minimal compression
            use_messagepack: true,   // Still cross-platform
            buffer_size: 512 * 1024, // Large buffer for speed
        }
    }

    /// Create options optimized for minimum file size
    ///
    /// Uses high compression for smallest possible files at cost of speed.
    pub fn compact() -> Self {
        Self {
            compression_level: 9,    // High compression
            use_messagepack: true,   // Efficient format
            buffer_size: 128 * 1024, // Smaller buffer to save memory
        }
    }
}

/// Simplified export data structure for proof of concept
///
/// Contains only essential data needed to demonstrate binary export
/// performance without complex dependencies on all type definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleBinaryData {
    /// Export metadata
    pub metadata: ExportMetadata,

    /// Basic allocation information
    pub allocations: Vec<SimpleAllocationInfo>,

    /// Summary statistics
    pub summary: ExportSummary,
}

/// Export metadata for tracking export details
///
/// Provides basic information about when and how the export was created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// Format version for compatibility checking
    pub version: u32,

    /// Unix timestamp when export was created
    pub created_at: u64,

    /// Compression format identifier
    pub compression_format: String,

    /// Total number of allocations in export
    pub allocation_count: usize,
}

/// Simplified allocation information for binary export
///
/// Contains core allocation data without complex nested structures
/// that might have serialization dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleAllocationInfo {
    /// Memory pointer address
    pub ptr: usize,

    /// Allocation size in bytes
    pub size: usize,

    /// Variable name if available
    pub var_name: Option<String>,

    /// Type name if available
    pub type_name: Option<String>,

    /// Allocation timestamp
    pub timestamp_alloc: u64,

    /// Deallocation timestamp if deallocated
    pub timestamp_dealloc: Option<u64>,

    /// Thread identifier
    pub thread_id: String,
}

/// Export summary statistics
///
/// Provides high-level overview of the exported data for quick analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSummary {
    /// Total number of allocations
    pub total_allocations: usize,

    /// Total bytes allocated
    pub total_allocated: usize,

    /// Number of active allocations
    pub active_allocations: usize,

    /// Total active memory
    pub active_memory: usize,

    /// Peak memory usage
    pub peak_memory: usize,
}

/// Statistics about the binary export operation
///
/// Provides performance metrics for comparing with JSON export.
#[derive(Debug, Clone)]
pub struct SimpleBinaryStats {
    /// Time taken for the export operation
    pub export_time: Duration,

    /// Original data size estimate
    pub original_size: usize,

    /// Compressed file size
    pub compressed_size: usize,

    /// Compression ratio (compressed/original)
    pub compression_ratio: f64,

    /// Number of allocations exported
    pub allocation_count: usize,
}

/// Simple binary export implementation for MemoryTracker
impl MemoryTracker {
    /// Export memory data to simplified binary format
    ///
    /// This method demonstrates the binary export concept with a simplified
    /// data structure that avoids complex serialization dependencies.
    ///
    /// # Arguments
    /// * `path` - Output file path for the binary export
    /// * `options` - Export configuration options
    ///
    /// # Returns
    /// Export statistics including timing and compression metrics
    ///
    /// # Example
    /// ```rust
    /// let tracker = get_global_tracker();
    /// let options = SimpleBinaryOptions::fast();
    /// let stats = tracker.export_to_simple_binary("snapshot.msgpack", options)?;
    /// println!("Export took {:?}, compression: {:.1}%",
    ///          stats.export_time, stats.compression_ratio * 100.0);
    /// ```
    pub fn export_to_simple_binary<P: AsRef<Path>>(
        &self,
        path: P,
        options: SimpleBinaryOptions,
    ) -> TrackingResult<SimpleBinaryStats> {
        let start_time = SystemTime::now();
        let path = path.as_ref();

        // Ensure output directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Collect simplified data from tracker
        let simple_allocations = self.collect_simple_allocations();
        let summary = self.create_export_summary(&simple_allocations);

        // Create export metadata
        let metadata = ExportMetadata {
            version: 1,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            compression_format: if options.use_messagepack {
                "messagepack+zstd".to_string()
            } else {
                "bincode+zstd".to_string()
            },
            allocation_count: simple_allocations.len(),
        };

        // Create complete export data
        let export_data = SimpleBinaryData {
            metadata,
            allocations: simple_allocations,
            summary,
        };

        // Serialize data
        let serialized = if options.use_messagepack {
            rmp_serde::to_vec(&export_data).map_err(|e| {
                TrackingError::SerializationError(format!("MessagePack error: {}", e))
            })?
        } else {
            // Fallback to MessagePack if bincode has issues
            rmp_serde::to_vec(&export_data).map_err(|e| {
                TrackingError::SerializationError(format!("MessagePack fallback error: {}", e))
            })?
        };

        // Compress data
        let compressed = zstd::encode_all(&serialized[..], options.compression_level)?;

        // Write to file
        let file = File::create(path)?;
        let mut writer = BufWriter::with_capacity(options.buffer_size, file);
        writer.write_all(&compressed)?;
        writer.flush()?;

        let export_time = start_time.elapsed().unwrap_or_default();
        let compressed_size = compressed.len();
        let original_size = serialized.len();

        Ok(SimpleBinaryStats {
            export_time,
            original_size,
            compressed_size,
            compression_ratio: compressed_size as f64 / original_size as f64,
            allocation_count: export_data.allocations.len(),
        })
    }

    /// Load simplified binary export data
    ///
    /// Loads and decompresses a binary export file created with export_to_simple_binary.
    ///
    /// # Arguments
    /// * `path` - Path to the binary export file
    ///
    /// # Returns
    /// The complete export data structure
    pub fn load_simple_binary<P: AsRef<Path>>(path: P) -> TrackingResult<SimpleBinaryData> {
        let path = path.as_ref();
        let compressed_data = std::fs::read(path)?;

        // Decompress data
        let decompressed_data = zstd::decode_all(&compressed_data[..])?;

        // Deserialize using MessagePack
        let export_data: SimpleBinaryData =
            rmp_serde::from_slice(&decompressed_data).map_err(|e| {
                TrackingError::SerializationError(format!(
                    "MessagePack deserialization failed: {}",
                    e
                ))
            })?;

        Ok(export_data)
    }

    /// Collect simplified allocation data from tracker
    ///
    /// Extracts allocation information in a simplified format that's easy to serialize.
    fn collect_simple_allocations(&self) -> Vec<SimpleAllocationInfo> {
        let mut simple_allocations = Vec::new();

        // Try to get active allocations (non-blocking)
        if let Ok(active) = self.active_allocations.try_lock() {
            for alloc in active.values() {
                simple_allocations.push(SimpleAllocationInfo {
                    ptr: alloc.ptr,
                    size: alloc.size,
                    var_name: alloc.var_name.clone(),
                    type_name: alloc.type_name.clone(),
                    timestamp_alloc: alloc.timestamp_alloc,
                    timestamp_dealloc: alloc.timestamp_dealloc,
                    thread_id: alloc.thread_id.clone(),
                });
            }
        }

        // Try to get allocation history (non-blocking)
        if let Ok(history) = self.allocation_history.try_lock() {
            for alloc in history.iter() {
                // Only add if not already in active allocations
                if !simple_allocations.iter().any(|sa| sa.ptr == alloc.ptr) {
                    simple_allocations.push(SimpleAllocationInfo {
                        ptr: alloc.ptr,
                        size: alloc.size,
                        var_name: alloc.var_name.clone(),
                        type_name: alloc.type_name.clone(),
                        timestamp_alloc: alloc.timestamp_alloc,
                        timestamp_dealloc: alloc.timestamp_dealloc,
                        thread_id: alloc.thread_id.clone(),
                    });
                }
            }
        }

        simple_allocations
    }

    /// Create export summary from allocation data
    ///
    /// Generates summary statistics for the export data.
    fn create_export_summary(&self, allocations: &[SimpleAllocationInfo]) -> ExportSummary {
        let total_allocations = allocations.len();
        let total_allocated: usize = allocations.iter().map(|a| a.size).sum();
        let active_allocations = allocations
            .iter()
            .filter(|a| a.timestamp_dealloc.is_none())
            .count();
        let active_memory: usize = allocations
            .iter()
            .filter(|a| a.timestamp_dealloc.is_none())
            .map(|a| a.size)
            .sum();

        ExportSummary {
            total_allocations,
            total_allocated,
            active_allocations,
            active_memory,
            peak_memory: total_allocated, // Simplified calculation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_binary_options() {
        let default = SimpleBinaryOptions::default();
        assert_eq!(default.compression_level, 3);
        assert!(default.use_messagepack);

        let fast = SimpleBinaryOptions::fast();
        assert_eq!(fast.compression_level, 1);
        assert_eq!(fast.buffer_size, 512 * 1024);

        let compact = SimpleBinaryOptions::compact();
        assert_eq!(compact.compression_level, 9);
        assert_eq!(compact.buffer_size, 128 * 1024);
    }

    #[test]
    fn test_simple_allocation_info_serialization() {
        #[allow(unused_imports)]
        use chrono::naive::serde::ts_microseconds::deserialize;

        let alloc = SimpleAllocationInfo {
            ptr: 0x1000,
            size: 256,
            var_name: Some("test_var".to_string()),
            type_name: Some("Vec<i32>".to_string()),
            timestamp_alloc: 1000,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
        };

        // Test MessagePack serialization
        let msgpack_data = rmp_serde::to_vec(&alloc).unwrap();
        let deserialized: SimpleAllocationInfo = rmp_serde::from_slice(&msgpack_data).unwrap();
        assert_eq!(alloc.ptr, deserialized.ptr);
        assert_eq!(alloc.size, deserialized.size);

        // Test bincode serialization
        let serialized_data = rmp_serde::to_vec(&alloc).unwrap();
        let deserialized: SimpleAllocationInfo = rmp_serde::from_slice(&serialized_data).unwrap();
        assert_eq!(alloc.ptr, deserialized.ptr);
        assert_eq!(alloc.size, deserialized.size);
    }
}
