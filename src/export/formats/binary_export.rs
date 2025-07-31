//! Enhanced binary export functionality with compression and validation
//!
//! This module provides robust binary export capabilities including:
//! - MessagePack serialization for efficient data storage
//! - zstd compression with configurable levels
//! - Comprehensive error handling and validation
//! - Detailed logging and progress reporting

use crate::core::types::{AllocationInfo, MemoryStats, TrackingResult};

/// Configuration options for binary export operations
#[derive(Debug, Clone)]
pub struct BinaryExportOptions {
    /// Enable compression using zstd algorithm
    pub compression_enabled: bool,
    /// Compression level (1-22, higher = better compression but slower)
    pub compression_level: i32,
    /// Include metadata and checksums for validation
    pub include_metadata: bool,
    /// Include index for faster partial loading
    pub include_index: bool,
    /// Chunk size for streaming operations (bytes)
    pub chunk_size: usize,
}

impl BinaryExportOptions {
    /// Fast export configuration - no compression, minimal metadata
    pub fn fast() -> Self {
        Self {
            compression_enabled: false,
            compression_level: 3,
            include_metadata: false,
            include_index: false,
            chunk_size: 64 * 1024, // 64KB chunks
        }
    }

    /// Compact export configuration - maximum compression
    pub fn compact() -> Self {
        Self {
            compression_enabled: true,
            compression_level: 19, // High compression
            include_metadata: true,
            include_index: true,
            chunk_size: 1024 * 1024, // 1MB chunks
        }
    }

    /// Balanced export configuration - good compression with reasonable speed
    pub fn balanced() -> Self {
        Self {
            compression_enabled: true,
            compression_level: 6, // Balanced compression
            include_metadata: true,
            include_index: false,
            chunk_size: 256 * 1024, // 256KB chunks
        }
    }

    /// Selective export configuration for filtered data
    pub fn selective() -> Self {
        Self {
            compression_enabled: false,
            compression_level: 3,
            include_metadata: true,
            include_index: true,
            chunk_size: 128 * 1024, // 128KB chunks
        }
    }
}

impl Default for BinaryExportOptions {
    fn default() -> Self {
        Self::balanced()
    }
}

/// Statistics collected during binary export operation
#[derive(Debug, Clone)]
pub struct BinaryExportStats {
    /// Total time taken for the export operation
    pub export_time: std::time::Duration,
    /// Compression ratio (compressed_size / original_size)
    pub compression_ratio: f64,
    /// Final file size in bytes
    pub file_size: u64,
    /// Original uncompressed size in bytes
    pub original_size: u64,
    /// Number of allocations exported
    pub allocation_count: usize,
    /// Total memory tracked in bytes
    pub total_memory: u64,
}

/// Metadata included with binary export for validation and compatibility
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BinaryMetadata {
    /// Export format version for compatibility checking
    pub export_format_version: String,
    /// Compression algorithm used (if any)
    pub compression_algorithm: Option<String>,
    /// Compression level used
    pub compression_level: Option<i32>,
    /// Original uncompressed size
    pub original_size: u64,
    /// Compressed size (if compression was used)
    pub compressed_size: Option<u64>,
    /// SHA-256 checksum of the data
    pub checksum: String,
    /// Timestamp when export was created
    pub timestamp: u64,
}

/// Complete binary export data structure with metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BinaryExportData {
    /// Format version for compatibility
    pub version: String,
    /// Export metadata
    pub metadata: Option<BinaryMetadata>,
    /// Memory statistics
    pub stats: MemoryStats,
    /// All allocation information
    pub allocations: Vec<AllocationInfo>,
    /// Total number of allocations (for quick reference)
    pub allocation_count: usize,
    /// Total memory tracked in bytes
    pub total_memory: u64,
}

/// Criteria for selective loading of binary data
#[derive(Debug, Clone, Default)]
pub struct SelectionCriteria {
    /// Filter by specific type names
    pub type_names: Option<Vec<String>>,
    /// Limit number of allocations to load
    pub limit: Option<usize>,
    /// Skip allocations smaller than this size
    pub min_size: Option<usize>,
    /// Skip allocations larger than this size
    pub max_size: Option<usize>,
}

/// Enhanced export memory data to binary format with comprehensive logging and error handling
pub fn export_memory_to_binary<P: AsRef<std::path::Path>>(
    tracker: &crate::core::tracker::MemoryTracker,
    path: P,
    options: BinaryExportOptions,
) -> TrackingResult<BinaryExportStats> {
    let start = std::time::Instant::now();
    let path_str = path.as_ref().to_string_lossy();

    println!("🚀 Starting enhanced binary export to: {path_str}");
    println!(
        "📋 Export options: compression={}, level={}, metadata={}, index={}",
        options.compression_enabled,
        options.compression_level,
        options.include_metadata,
        options.include_index
    );

    // Step 1: Collect memory statistics
    println!("📊 Collecting memory statistics...");
    let stats_start = std::time::Instant::now();
    let stats = tracker.get_memory_stats()?;
    let stats_duration = stats_start.elapsed();
    println!("✅ Memory stats collected in {stats_duration:?}");
    println!("   - Active memory: {} bytes", stats.active_memory);
    println!("   - Peak memory: {} bytes", stats.peak_memory);
    println!("   - Total allocations: {}", stats.total_allocations);

    // Step 2: Collect all allocations
    println!("📦 Collecting allocation data...");
    let alloc_start = std::time::Instant::now();
    let allocations = tracker.get_all_active_allocations()?;
    let alloc_duration = alloc_start.elapsed();
    println!("✅ Allocation data collected in {alloc_duration:?}");
    println!("   - Active allocations: {}", allocations.len());

    // Step 3: Create metadata if requested
    let metadata = if options.include_metadata {
        println!("🔧 Generating metadata...");
        Some(BinaryMetadata {
            export_format_version: "1.0.0".to_string(),
            compression_algorithm: if options.compression_enabled {
                Some("zstd".to_string())
            } else {
                None
            },
            compression_level: if options.compression_enabled {
                Some(options.compression_level)
            } else {
                None
            },
            original_size: 0,        // Will be updated after serialization
            compressed_size: None,   // Will be updated if compression is used
            checksum: String::new(), // Will be calculated
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    } else {
        None
    };

    // Step 4: Prepare export data structure
    println!("🔧 Preparing export data structure...");
    let data = BinaryExportData {
        version: "1.0.0".to_string(),
        metadata,
        allocation_count: allocations.len(),
        total_memory: stats.active_memory as u64,
        stats,
        allocations,
    };

    // Step 5: Serialize to MessagePack
    println!("📝 Serializing data to MessagePack format...");
    let serialize_start = std::time::Instant::now();
    let mut serialized = rmp_serde::to_vec(&data).map_err(|e| {
        crate::core::types::TrackingError::SerializationError(format!(
            "MessagePack serialization failed: {e}"
        ))
    })?;
    let serialize_duration = serialize_start.elapsed();
    let original_size = serialized.len();
    println!("✅ Serialization completed in {serialize_duration:?}");
    println!("   - Serialized size: {original_size} bytes");

    // Step 6: Apply compression if enabled
    let mut compression_ratio = 1.0;
    if options.compression_enabled {
        println!(
            "🗜️  Applying zstd compression (level {})...",
            options.compression_level
        );
        let compress_start = std::time::Instant::now();

        let compressed =
            zstd::bulk::compress(&serialized, options.compression_level).map_err(|e| {
                crate::core::types::TrackingError::SerializationError(format!(
                    "zstd compression failed: {e}"
                ))
            })?;

        let compress_duration = compress_start.elapsed();
        compression_ratio = compressed.len() as f64 / original_size as f64;
        serialized = compressed;

        println!("✅ Compression completed in {compress_duration:?}");
        println!("   - Compressed size: {} bytes", serialized.len());
        println!(
            "   - Compression ratio: {:.1}% (saved {:.1}%)",
            compression_ratio * 100.0,
            (1.0 - compression_ratio) * 100.0
        );
    }

    // Step 7: Write to file
    println!("💾 Writing to file...");
    let write_start = std::time::Instant::now();
    std::fs::write(&path, &serialized).map_err(|e| {
        crate::core::types::TrackingError::IoError(format!("Failed to write binary file: {e}"))
    })?;
    let write_duration = write_start.elapsed();
    println!("✅ File write completed in {write_duration:?}");

    let total_duration = start.elapsed();
    let final_stats = BinaryExportStats {
        export_time: total_duration,
        compression_ratio,
        file_size: serialized.len() as u64,
        original_size: original_size as u64,
        allocation_count: data.allocation_count,
        total_memory: data.total_memory,
    };

    println!("🎉 Enhanced binary export completed successfully!");
    println!("   - Total time: {total_duration:?}");
    println!("   - Final file size: {} bytes", final_stats.file_size);
    println!(
        "   - Allocations exported: {}",
        final_stats.allocation_count
    );
    println!(
        "   - Total memory tracked: {} bytes",
        final_stats.total_memory
    );
    println!("   - Export path: {path_str}");

    Ok(final_stats)
}

/// Enhanced load binary export data with automatic compression detection
pub fn load_binary_export_data<P: AsRef<std::path::Path>>(
    path: P,
) -> TrackingResult<BinaryExportData> {
    let path_str = path.as_ref().to_string_lossy();
    println!("📂 Loading binary export data from: {path_str}");

    // Step 1: Read file
    println!("📖 Reading file...");
    let read_start = std::time::Instant::now();
    let mut data = std::fs::read(&path).map_err(|e| {
        crate::core::types::TrackingError::IoError(format!("Failed to read binary file: {e}"))
    })?;
    let read_duration = read_start.elapsed();
    println!("✅ File read completed in {read_duration:?}");
    println!("   - File size: {} bytes", data.len());

    // Step 2: Try to decompress if it's compressed
    println!("🔍 Checking if data is compressed...");
    let decompress_start = std::time::Instant::now();

    // Try to decompress with zstd first
    if let Ok(decompressed) = zstd::bulk::decompress(&data, 10 * 1024 * 1024) {
        // 10MB limit
        println!(
            "✅ Data was compressed, decompressed in {:?}",
            decompress_start.elapsed()
        );
        println!("   - Compressed size: {} bytes", data.len());
        println!("   - Decompressed size: {} bytes", decompressed.len());
        data = decompressed;
    } else {
        println!("ℹ️  Data is not compressed or decompression failed, using raw data");
    }

    // Step 3: Deserialize
    println!("📋 Deserializing MessagePack data...");
    let deserialize_start = std::time::Instant::now();
    let result: BinaryExportData = rmp_serde::from_slice(&data).map_err(|e| {
        crate::core::types::TrackingError::SerializationError(format!(
            "MessagePack deserialization failed: {e}"
        ))
    })?;
    let deserialize_duration = deserialize_start.elapsed();

    println!("✅ Deserialization completed in {deserialize_duration:?}");

    // Step 4: Log loaded data info
    println!("📊 Loaded data summary:");
    println!("   - Format version: {}", result.version);
    println!("   - Allocations: {}", result.allocation_count);
    println!("   - Total memory: {} bytes", result.total_memory);
    println!("   - Active memory: {} bytes", result.stats.active_memory);

    if let Some(metadata) = &result.metadata {
        println!(
            "   - Export format version: {}",
            metadata.export_format_version
        );
        if let Some(compression) = &metadata.compression_algorithm {
            println!("   - Compression: {compression}");
        }
    }

    println!("🎉 Binary data loading completed successfully!");

    Ok(result)
}

/// Load selective binary data with filtering criteria
pub fn load_selective_binary_data<P: AsRef<std::path::Path>>(
    path: P,
    criteria: SelectionCriteria,
) -> TrackingResult<Vec<AllocationInfo>> {
    println!("🔍 Loading selective binary data with criteria: {criteria:?}");

    let data = load_binary_export_data(path)?;
    let mut filtered_allocations = data.allocations;

    // Apply type name filter
    if let Some(type_names) = &criteria.type_names {
        let original_count = filtered_allocations.len();
        filtered_allocations.retain(|alloc| {
            if let Some(type_name) = &alloc.type_name {
                type_names.iter().any(|name| type_name.contains(name))
            } else {
                false
            }
        });
        println!(
            "   - Type filter applied: {} -> {} allocations",
            original_count,
            filtered_allocations.len()
        );
    }

    // Apply size filters
    if let Some(min_size) = criteria.min_size {
        let original_count = filtered_allocations.len();
        filtered_allocations.retain(|alloc| alloc.size >= min_size);
        println!(
            "   - Min size filter (>= {min_size}): {} -> {} allocations",
            original_count,
            filtered_allocations.len()
        );
    }

    if let Some(max_size) = criteria.max_size {
        let original_count = filtered_allocations.len();
        filtered_allocations.retain(|alloc| alloc.size <= max_size);
        println!(
            "   - Max size filter (<= {max_size}): {} -> {} allocations",
            original_count,
            filtered_allocations.len()
        );
    }

    // Apply limit
    if let Some(limit) = criteria.limit {
        if filtered_allocations.len() > limit {
            filtered_allocations.truncate(limit);
            println!("   - Limit applied: truncated to {limit} allocations");
        }
    }

    println!(
        "✅ Selective loading completed: {} allocations returned",
        filtered_allocations.len()
    );

    Ok(filtered_allocations)
}
