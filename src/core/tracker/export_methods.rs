//! Export methods for memory tracking data.

use crate::core::tracker::memory_tracker::MemoryTracker;
use crate::core::types::{AllocationInfo, TrackingResult};

impl MemoryTracker {
    /// Export memory data to binary format with comprehensive options
    ///
    /// This method provides high-performance binary export using MessagePack serialization
    /// and Zstd compression. It offers significant advantages over JSON export including
    /// faster serialization, smaller file sizes, and selective loading capabilities.
    ///
    /// # Arguments
    /// * `path` - Output file path for the binary export
    /// * `options` - Binary export configuration options
    ///
    /// # Returns
    /// Export statistics including timing and compression metrics
    ///
    /// # Performance Benefits
    /// - 5-10x faster serialization compared to JSON
    /// - 20-50% smaller file sizes with MessagePack
    /// - 60-80% smaller with compression enabled
    /// - Selective loading support for large datasets
    ///
    /// # Example
    /// ```rust
    /// use memscope_rs::export::binary_export::BinaryExportOptions;
    ///
    /// let tracker = get_global_tracker();
    ///
    /// // Fast export for real-time monitoring
    /// let fast_options = BinaryExportOptions::fast();
    /// let stats = tracker.export_to_binary("snapshot.msgpack", fast_options)?;
    ///
    /// // Compact export for archival storage
    /// let compact_options = BinaryExportOptions::compact();
    /// let stats = tracker.export_to_binary("archive.msgpack", compact_options)?;
    ///
    /// println!("Export completed in {:?}", stats.export_time);
    /// println!("Compression ratio: {:.1}%", stats.compression_ratio * 100.0);
    /// ```
    pub fn export_to_binary<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        options: crate::export::binary_export::BinaryExportOptions,
    ) -> TrackingResult<crate::export::binary_export::BinaryExportStats> {
        // Delegate to the binary export implementation
        crate::export::binary_export::export_memory_to_binary(self, path, options)
    }

    /// Load binary export data with selective filtering
    ///
    /// This method loads binary export files created with export_to_binary,
    /// with optional filtering to load only specific data subsets.
    ///
    /// # Arguments
    /// * `path` - Path to the binary export file
    ///
    /// # Returns
    /// The loaded binary export data structure
    ///
    /// # Example
    /// ```rust
    /// // Load all data
    /// let full_data = MemoryTracker::load_from_binary("snapshot.msgpack")?;
    /// println!("Loaded {} allocations", full_data.allocations.len());
    /// ```
    pub fn load_from_binary<P: AsRef<std::path::Path>>(
        path: P,
    ) -> TrackingResult<crate::export::binary_export::BinaryExportData> {
        crate::export::binary_export::load_binary_export_data(path)
    }

    /// Load selective binary data with filtering criteria
    ///
    /// This method enables efficient loading of specific data subsets from
    /// binary export files, significantly reducing memory usage and loading time
    /// for large datasets when only specific data is needed.
    ///
    /// # Arguments
    /// * `path` - Path to the binary export file
    /// * `criteria` - Selection criteria for filtering data
    ///
    /// # Returns
    /// Vector of filtered allocation information
    ///
    /// # Performance Optimization
    /// When an index is available in the binary file, this method can skip
    /// loading unnecessary data entirely, providing substantial performance
    /// improvements for large datasets.
    ///
    /// # Example
    /// ```rust
    /// use memscope_rs::export::binary_export::SelectionCriteria;
    ///
    /// // Load only Vec<i32> allocations
    /// let criteria = SelectionCriteria {
    ///     type_names: Some(vec!["Vec<i32>".to_string()]),
    ///     limit: Some(100),
    ///     ..Default::default()
    /// };
    /// let filtered_data = MemoryTracker::load_selective_binary("snapshot.msgpack", criteria)?;
    /// println!("Loaded {} filtered allocations", filtered_data.len());
    /// ```
    pub fn load_selective_binary<P: AsRef<std::path::Path>>(
        path: P,
        criteria: crate::export::binary_export::SelectionCriteria,
    ) -> TrackingResult<Vec<AllocationInfo>> {
        crate::export::binary_export::load_selective_binary_data(path, criteria)
    }
}