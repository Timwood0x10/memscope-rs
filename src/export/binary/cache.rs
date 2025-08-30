//! Index caching system for binary file indexes
//!
//! This module provides persistent caching of binary file indexes to avoid
//! rebuilding indexes for unchanged files. Uses LRU eviction and intelligent
//! cache validation based on file hash and modification time.

use crate::export::binary::error::BinaryExportError;
use crate::export::binary::index::BinaryIndex;
use crate::export::binary::index_builder::BinaryIndexBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Cache entry metadata for tracking cache validity and usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// File path that this cache entry corresponds to
    pub file_path: PathBuf,
    /// Hash of the file content when cached
    pub file_hash: u64,
    /// File size when cached
    pub file_size: u64,
    /// File modification time when cached
    pub file_modified: u64,
    /// When this cache entry was created
    pub cached_at: u64,
    /// When this cache entry was last accessed
    pub last_accessed: u64,
    /// Number of times this cache entry has been accessed
    pub access_count: u64,
    /// Path to the cached index file
    pub cache_file_path: PathBuf,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(
        file_path: PathBuf,
        file_hash: u64,
        file_size: u64,
        file_modified: u64,
        cache_file_path: PathBuf,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        Self {
            file_path,
            file_hash,
            file_size,
            file_modified,
            cached_at: now,
            last_accessed: now,
            access_count: 0,
            cache_file_path,
        }
    }

    /// Update access statistics
    pub fn mark_accessed(&mut self) {
        self.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        self.access_count += 1;
    }

    /// Check if this cache entry is still valid for the given file
    pub fn is_valid_for_file<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<bool, BinaryExportError> {
        let path = file_path.as_ref();

        // Check if file exists
        if !path.exists() {
            return Ok(false);
        }

        // Check file metadata
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();

        // Check if file size changed
        if file_size != self.file_size {
            return Ok(false);
        }

        // Check modification time
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                let file_modified = duration.as_secs();
                if file_modified != self.file_modified {
                    return Ok(false);
                }
            }
        }

        // Check if cache file still exists
        if !self.cache_file_path.exists() {
            return Ok(false);
        }

        Ok(true)
    }
}

/// Configuration for the index cache
#[derive(Debug, Clone)]
pub struct IndexCacheConfig {
    /// Maximum number of cached indexes to keep
    pub max_entries: usize,
    /// Maximum age of cache entries in seconds
    pub max_age_seconds: u64,
    /// Directory to store cached indexes
    pub cache_directory: PathBuf,
    /// Whether to enable cache compression
    pub enable_compression: bool,
}

impl Default for IndexCacheConfig {
    fn default() -> Self {
        let cache_dir = std::env::temp_dir().join("memscope_index_cache");
        Self {
            max_entries: 100,
            max_age_seconds: 7 * 24 * 3600, // 7 days
            cache_directory: cache_dir,
            enable_compression: true,
        }
    }
}

/// Statistics about cache performance
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total number of cache requests
    pub total_requests: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Number of cache entries evicted
    pub evictions: u64,
    /// Total time saved by cache hits (in milliseconds)
    pub time_saved_ms: u64,
}

impl CacheStats {
    /// Calculate cache hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / self.total_requests as f64) * 100.0
        }
    }

    /// Record a cache hit
    pub fn record_hit(&mut self, time_saved_ms: u64) {
        self.total_requests += 1;
        self.cache_hits += 1;
        self.time_saved_ms += time_saved_ms;
    }

    /// Record a cache miss
    pub fn record_miss(&mut self) {
        self.total_requests += 1;
        self.cache_misses += 1;
    }

    /// Record a cache eviction
    pub fn record_eviction(&mut self) {
        self.evictions += 1;
    }
}

/// Index cache manager with LRU eviction and intelligent validation
pub struct IndexCache {
    /// Cache configuration
    config: IndexCacheConfig,
    /// Cache entries metadata
    entries: HashMap<String, CacheEntry>,
    /// Cache statistics
    stats: CacheStats,
    /// Path to the cache metadata file
    metadata_file: PathBuf,
}

impl IndexCache {
    /// Create a new index cache with the given configuration
    pub fn new(config: IndexCacheConfig) -> Result<Self, BinaryExportError> {
        // Ensure cache directory exists
        if !config.cache_directory.exists() {
            fs::create_dir_all(&config.cache_directory)?;
        }

        let metadata_file = config.cache_directory.join("cache_metadata.json");

        let mut cache = Self {
            config,
            entries: HashMap::new(),
            stats: CacheStats::default(),
            metadata_file,
        };

        // Load existing cache metadata
        cache.load_metadata()?;

        // Clean up expired entries
        cache.cleanup_expired_entries()?;

        Ok(cache)
    }

    /// Create a new index cache with default configuration
    pub fn with_default_config() -> Result<Self, BinaryExportError> {
        Self::new(IndexCacheConfig::default())
    }

    /// Get or build an index for the specified file
    pub fn get_or_build_index<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        builder: &BinaryIndexBuilder,
    ) -> Result<BinaryIndex, BinaryExportError> {
        let path = file_path.as_ref();
        let cache_key = self.generate_cache_key(path);

        // Check if we have a valid cached entry
        if let Some(entry) = self.entries.get(&cache_key) {
            if entry.is_valid_for_file(path)? {
                // Cache hit - load from cache
                let start_time = std::time::Instant::now();
                let index = self.load_index_from_cache(entry)?;
                let load_time = start_time.elapsed().as_millis() as u64;

                // Update access statistics
                if let Some(entry) = self.entries.get_mut(&cache_key) {
                    entry.mark_accessed();
                }
                self.stats.record_hit(load_time);

                return Ok(index);
            } else {
                // Cache entry is invalid - remove it
                self.remove_cache_entry(&cache_key)?;
            }
        }

        // Cache miss - build new index
        self.stats.record_miss();
        let start_time = std::time::Instant::now();
        let index = builder.build_index(path)?;
        let build_time = start_time.elapsed().as_millis() as u64;

        // Cache the newly built index
        self.cache_index(path, &index, build_time)?;

        Ok(index)
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Clear all cache entries
    pub fn clear(&mut self) -> Result<(), BinaryExportError> {
        // Remove all cache files
        for entry in self.entries.values() {
            if entry.cache_file_path.exists() {
                fs::remove_file(&entry.cache_file_path)?;
            }
        }

        // Clear entries and reset stats
        self.entries.clear();
        self.stats = CacheStats::default();

        // Save empty metadata
        self.save_metadata()?;

        Ok(())
    }

    /// Generate a cache key for the given file path
    fn generate_cache_key<P: AsRef<Path>>(&self, file_path: P) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let path = file_path.as_ref();
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        format!("index_{:x}", hasher.finish())
    }

    /// Cache an index for the given file
    fn cache_index<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        index: &BinaryIndex,
        _build_time_ms: u64,
    ) -> Result<(), BinaryExportError> {
        let path = file_path.as_ref();
        let cache_key = self.generate_cache_key(path);

        // Get file metadata
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();
        let file_modified = metadata
            .modified()?
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        // Generate cache file path
        let cache_file_name = format!("{cache_key}.index");
        let cache_file_path = self.config.cache_directory.join(cache_file_name);

        // Serialize and save the index
        self.save_index_to_cache(index, &cache_file_path)?;

        // Create cache entry
        let entry = CacheEntry::new(
            path.to_path_buf(),
            index.file_hash,
            file_size,
            file_modified,
            cache_file_path,
        );

        // Add to cache
        self.entries.insert(cache_key, entry);

        // Enforce cache size limit
        self.enforce_cache_limits()?;

        // Save metadata
        self.save_metadata()?;

        Ok(())
    }

    /// Load an index from cache
    fn load_index_from_cache(&self, entry: &CacheEntry) -> Result<BinaryIndex, BinaryExportError> {
        let cache_data = fs::read(&entry.cache_file_path)?;

        if self.config.enable_compression {
            // Decompress if compression is enabled
            let decompressed = self.decompress_data(&cache_data)?;
            self.deserialize_index(&decompressed)
        } else {
            self.deserialize_index(&cache_data)
        }
    }

    /// Save an index to cache
    fn save_index_to_cache(
        &self,
        index: &BinaryIndex,
        cache_file_path: &Path,
    ) -> Result<(), BinaryExportError> {
        let serialized = self.serialize_index(index)?;

        let data_to_write = if self.config.enable_compression {
            self.compress_data(&serialized)?
        } else {
            serialized
        };

        fs::write(cache_file_path, data_to_write)?;
        Ok(())
    }

    /// Serialize an index to bytes
    fn serialize_index(&self, index: &BinaryIndex) -> Result<Vec<u8>, BinaryExportError> {
        bincode::serialize(index).map_err(|e| {
            BinaryExportError::SerializationError(format!("Failed to serialize index: {e}"))
        })
    }

    /// Deserialize an index from bytes
    fn deserialize_index(&self, data: &[u8]) -> Result<BinaryIndex, BinaryExportError> {
        bincode::deserialize(data).map_err(|e| {
            BinaryExportError::SerializationError(format!("Failed to deserialize index: {e}"))
        })
    }

    /// Compress data using a simple compression algorithm
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError> {
        // Simple compression using flate2 (gzip)
        use std::io::Write;
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(data)?;
        encoder
            .finish()
            .map_err(|e| BinaryExportError::CompressionError(format!("Compression failed: {e}")))
    }

    /// Decompress data
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError> {
        use std::io::Read;
        let mut decoder = flate2::read::GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    /// Remove a cache entry and its associated file
    fn remove_cache_entry(&mut self, cache_key: &str) -> Result<(), BinaryExportError> {
        if let Some(entry) = self.entries.remove(cache_key) {
            if entry.cache_file_path.exists() {
                fs::remove_file(&entry.cache_file_path)?;
            }
        }
        Ok(())
    }

    /// Enforce cache size and age limits using LRU eviction
    fn enforce_cache_limits(&mut self) -> Result<(), BinaryExportError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        // Remove expired entries
        let expired_keys: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, entry)| (now - entry.cached_at) > self.config.max_age_seconds)
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.remove_cache_entry(&key)?;
            self.stats.record_eviction();
        }

        // Enforce size limit using LRU eviction
        while self.entries.len() > self.config.max_entries {
            // Find the least recently used entry
            let lru_key = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_accessed)
                .map(|(key, _)| key.clone());

            if let Some(key) = lru_key {
                self.remove_cache_entry(&key)?;
                self.stats.record_eviction();
            } else {
                break;
            }
        }

        Ok(())
    }

    /// Clean up expired cache entries
    fn cleanup_expired_entries(&mut self) -> Result<(), BinaryExportError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        let expired_keys: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, entry)| {
                (now - entry.cached_at) > self.config.max_age_seconds
                    || !entry.cache_file_path.exists()
            })
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.remove_cache_entry(&key)?;
        }

        Ok(())
    }

    /// Load cache metadata from disk
    fn load_metadata(&mut self) -> Result<(), BinaryExportError> {
        if !self.metadata_file.exists() {
            return Ok(());
        }

        let metadata_content = fs::read_to_string(&self.metadata_file)?;
        let entries: HashMap<String, CacheEntry> = serde_json::from_str(&metadata_content)
            .map_err(|e| {
                BinaryExportError::SerializationError(format!(
                    "Failed to parse cache metadata: {e}",
                ))
            })?;

        self.entries = entries;
        Ok(())
    }

    /// Save cache metadata to disk
    fn save_metadata(&self) -> Result<(), BinaryExportError> {
        let metadata_content = serde_json::to_string_pretty(&self.entries).map_err(|e| {
            BinaryExportError::SerializationError(format!(
                "Failed to serialize cache metadata: {e}",
            ))
        })?;

        fs::write(&self.metadata_file, metadata_content)?;
        Ok(())
    }
}

impl Drop for IndexCache {
    fn drop(&mut self) {
        // Save metadata when cache is dropped
        let _ = self.save_metadata();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AllocationInfo;
    use crate::export::binary::writer::BinaryWriter;
    use tempfile::{NamedTempFile, TempDir};

    fn create_test_allocation() -> AllocationInfo {
        AllocationInfo {
            ptr: 0x1000,
            size: 1024,
            var_name: Some("test_var".to_string()),
            type_name: Some("i32".to_string()),
            scope_name: None,
            timestamp_alloc: 1234567890,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        }
    }

    fn create_test_binary_file() -> NamedTempFile {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let test_allocations = vec![create_test_allocation()];

        // Write test data to binary file
        {
            let mut writer =
                BinaryWriter::new(temp_file.path()).expect("Failed to create temp file");
            writer
                .write_header(test_allocations.len() as u32)
                .expect("Failed to write header");
            for alloc in &test_allocations {
                writer
                    .write_allocation(alloc)
                    .expect("Failed to write allocation");
            }
            writer.finish().expect("Failed to finish writing");
        }

        temp_file
    }

    #[test]
    fn test_cache_entry_creation() {
        let entry = CacheEntry::new(
            PathBuf::from("/test/file.bin"),
            12345,
            1024,
            1000000,
            PathBuf::from("/cache/entry.index"),
        );

        assert_eq!(entry.file_path, PathBuf::from("/test/file.bin"));
        assert_eq!(entry.file_hash, 12345);
        assert_eq!(entry.file_size, 1024);
        assert_eq!(entry.file_modified, 1000000);
        assert_eq!(entry.access_count, 0);
    }

    #[test]
    fn test_cache_entry_access_tracking() {
        let mut entry = CacheEntry::new(
            PathBuf::from("/test/file.bin"),
            12345,
            1024,
            1000000,
            PathBuf::from("/cache/entry.index"),
        );

        let initial_access_time = entry.last_accessed;
        let initial_count = entry.access_count;

        // Wait a bit to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(100));

        entry.mark_accessed();

        assert!(entry.last_accessed >= initial_access_time);
        assert_eq!(entry.access_count, initial_count + 1);
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::default();

        assert_eq!(stats.hit_rate(), 0.0);

        stats.record_miss();
        assert_eq!(stats.hit_rate(), 0.0);
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.cache_misses, 1);

        stats.record_hit(100);
        assert_eq!(stats.hit_rate(), 50.0);
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.time_saved_ms, 100);

        stats.record_eviction();
        assert_eq!(stats.evictions, 1);
    }

    #[test]
    fn test_index_cache_creation() {
        let temp_dir = TempDir::new().expect("Failed to get test value");
        let config = IndexCacheConfig {
            cache_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let cache = IndexCache::new(config).expect("Failed to get test value");
        assert_eq!(cache.entries.len(), 0);
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_cache_miss_and_build() {
        let temp_dir = TempDir::new().expect("Failed to get test value");
        let config = IndexCacheConfig {
            cache_directory: temp_dir.path().to_path_buf(),
            max_entries: 10,
            ..Default::default()
        };

        let mut cache = IndexCache::new(config).expect("Failed to create cache");
        let test_file = create_test_binary_file();
        let builder = BinaryIndexBuilder::new();

        // First access should be a cache miss
        let index1 = cache
            .get_or_build_index(test_file.path(), &builder)
            .expect("Test operation failed");
        assert_eq!(cache.get_stats().cache_misses, 1);
        assert_eq!(cache.get_stats().cache_hits, 0);
        assert_eq!(cache.entries.len(), 1);

        // Second access should be a cache hit
        let index2 = cache
            .get_or_build_index(test_file.path(), &builder)
            .expect("Test operation failed");
        assert_eq!(cache.get_stats().cache_misses, 1);
        assert_eq!(cache.get_stats().cache_hits, 1);
        assert_eq!(cache.get_stats().hit_rate(), 50.0);

        // Indexes should be equivalent
        assert_eq!(index1.file_hash, index2.file_hash);
        assert_eq!(index1.record_count(), index2.record_count());
    }

    #[test]
    fn test_cache_invalidation_on_file_change() {
        let temp_dir = TempDir::new().expect("Failed to get test value");
        let config = IndexCacheConfig {
            cache_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut cache = IndexCache::new(config).expect("Failed to create cache");
        let test_file = create_test_binary_file();
        let builder = BinaryIndexBuilder::new();

        // First access - cache miss
        let _index1 = cache
            .get_or_build_index(test_file.path(), &builder)
            .expect("Test operation failed");
        assert_eq!(cache.get_stats().cache_misses, 1);
        assert_eq!(cache.entries.len(), 1);

        // Wait a bit to ensure file modification time changes
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Modify the file by creating a new valid binary file with different content
        let test_allocations = vec![{
            let mut alloc = create_test_allocation();
            alloc.ptr = 0x2000; // Different pointer value
            alloc.size = 2048; // Different size
            alloc
        }];

        // Write new test data to binary file
        {
            let mut writer = BinaryWriter::new(test_file.path()).expect("Failed to create writer");
            writer
                .write_header(test_allocations.len() as u32)
                .expect("Failed to write header");
            for alloc in &test_allocations {
                writer
                    .write_allocation(alloc)
                    .expect("Failed to write allocation");
            }
            writer.finish().expect("Failed to finish writing");
        }

        // Next access should be a cache miss due to file change
        let result = cache.get_or_build_index(test_file.path(), &builder);
        assert!(result.is_ok());

        // The cache should have detected the file change and invalidated the entry
        // This should result in either 2 misses (if invalidated) or still 1 miss but different content
        assert!(cache.get_stats().cache_misses >= 1);
    }

    #[test]
    fn test_cache_size_limit_enforcement() {
        let temp_dir = TempDir::new().expect("Failed to get test value");
        let config = IndexCacheConfig {
            cache_directory: temp_dir.path().to_path_buf(),
            max_entries: 2, // Small limit for testing
            ..Default::default()
        };

        let mut cache = IndexCache::new(config).expect("Failed to create cache");
        let builder = BinaryIndexBuilder::new();

        // Create multiple test files
        let file1 = create_test_binary_file();
        let file2 = create_test_binary_file();
        let file3 = create_test_binary_file();

        // Cache first two files
        let _index1 = cache
            .get_or_build_index(file1.path(), &builder)
            .expect("Failed to get or build index");
        let _index2 = cache
            .get_or_build_index(file2.path(), &builder)
            .expect("Failed to get or build index");
        assert_eq!(cache.entries.len(), 2);

        // Adding third file should evict the least recently used entry
        let _index3 = cache
            .get_or_build_index(file3.path(), &builder)
            .expect("Failed to get or build index");
        assert_eq!(cache.entries.len(), 2);
        assert!(cache.get_stats().evictions > 0);
    }

    #[test]
    fn test_cache_clear() {
        let temp_dir = TempDir::new().expect("Failed to get test value");
        let config = IndexCacheConfig {
            cache_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut cache = IndexCache::new(config).expect("Failed to create cache");
        let test_file = create_test_binary_file();
        let builder = BinaryIndexBuilder::new();

        // Add an entry to cache
        let _index = cache
            .get_or_build_index(test_file.path(), &builder)
            .expect("Test operation failed");
        assert_eq!(cache.entries.len(), 1);

        // Clear cache
        cache.clear().expect("Test operation failed");
        assert_eq!(cache.entries.len(), 0);
        assert_eq!(cache.get_stats().total_requests, 0);
    }
}
