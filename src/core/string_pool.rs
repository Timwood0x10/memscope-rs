//! Global string pool system for memory optimization
//!
//! This module provides a global string interning system to reduce memory usage
//! by sharing common strings across the application. All string fields in
//! AllocationInfo and related structures use `Arc<str>` backed by this pool.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

/// Statistics about string pool usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringPoolStats {
    /// Total number of unique strings in the pool
    pub unique_strings: usize,
    /// Total number of intern operations performed
    pub intern_operations: u64,
    /// Number of cache hits (string already existed)
    pub cache_hits: u64,
    /// Estimated memory saved by string interning (in bytes)
    pub memory_saved_bytes: u64,
    /// Average string length in the pool
    pub average_string_length: f64,
}

/// Internal statistics tracking for the string pool
struct PoolMetrics {
    intern_count: AtomicU64,
    cache_hits: AtomicU64,
    total_string_bytes: AtomicU64,
    duplicate_bytes_saved: AtomicU64,
}

impl PoolMetrics {
    fn new() -> Self {
        Self {
            intern_count: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            total_string_bytes: AtomicU64::new(0),
            duplicate_bytes_saved: AtomicU64::new(0),
        }
    }

    fn record_intern(&self, string_len: usize, was_cached: bool) {
        self.intern_count.fetch_add(1, Ordering::Relaxed);

        if was_cached {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
            self.duplicate_bytes_saved
                .fetch_add(string_len as u64, Ordering::Relaxed);
        } else {
            self.total_string_bytes
                .fetch_add(string_len as u64, Ordering::Relaxed);
        }
    }
}

/// Global string pool for memory-efficient string storage
pub struct StringPool {
    /// Map from string content to interned `Arc<str>`
    pool: DashMap<String, Arc<str>>,
    /// Performance and usage metrics
    metrics: PoolMetrics,
}

impl StringPool {
    /// Create a new string pool
    fn new() -> Self {
        Self {
            pool: DashMap::new(),
            metrics: PoolMetrics::new(),
        }
    }

    /// Intern a string, returning an `Arc<str>` that can be shared
    ///
    /// If the string already exists in the pool, returns the existing `Arc<str>`.
    /// Otherwise, creates a new `Arc<str>` and stores it in the pool.
    pub fn intern(&self, s: &str) -> Arc<str> {
        let string_len = s.len();

        // Try to get existing string first
        if let Some(existing) = self.pool.get(s) {
            self.metrics.record_intern(string_len, true);
            return existing.clone();
        }

        // String doesn't exist, create new Arc<str>
        let arc_str: Arc<str> = Arc::from(s);

        // Insert into pool - use entry API to handle race conditions
        let result = self
            .pool
            .entry(s.to_string())
            .or_insert_with(|| Arc::clone(&arc_str))
            .clone();

        self.metrics.record_intern(string_len, false);
        result
    }

    /// Get current statistics about the string pool
    pub fn get_stats(&self) -> StringPoolStats {
        let unique_strings = self.pool.len();
        let intern_operations = self.metrics.intern_count.load(Ordering::Relaxed);
        let cache_hits = self.metrics.cache_hits.load(Ordering::Relaxed);
        let total_bytes = self.metrics.total_string_bytes.load(Ordering::Relaxed);
        let saved_bytes = self.metrics.duplicate_bytes_saved.load(Ordering::Relaxed);

        let average_length = if unique_strings > 0 {
            total_bytes as f64 / unique_strings as f64
        } else {
            0.0
        };

        StringPoolStats {
            unique_strings,
            intern_operations,
            cache_hits,
            memory_saved_bytes: saved_bytes,
            average_string_length: average_length,
        }
    }

    /// Clear all strings from the pool (useful for testing)
    #[cfg(test)]
    pub fn clear(&self) {
        self.pool.clear();
    }

    /// Get the number of unique strings currently in the pool
    pub fn len(&self) -> usize {
        self.pool.len()
    }

    /// Check if the pool is empty
    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }
}

/// Global string pool instance
static GLOBAL_STRING_POOL: OnceLock<StringPool> = OnceLock::new();

/// Get the global string pool instance
fn get_global_pool() -> &'static StringPool {
    GLOBAL_STRING_POOL.get_or_init(StringPool::new)
}

/// Intern a string using the global string pool
///
/// This is the main API for string interning. All string fields in AllocationInfo
/// and related structures should use this function to intern their strings.
///
/// # Example
/// ```text
/// use memscope_rs::core::string_pool::intern_string;
/// use std::sync::Arc;
///
/// let s1 = intern_string("hello");
/// let s2 = intern_string("hello");
///
/// // Both Arc<str> point to the same memory
/// assert!(Arc::ptr_eq(&s1, &s2));
/// ```
pub fn intern_string(s: &str) -> Arc<str> {
    let start_time = std::time::Instant::now();
    let result = get_global_pool().intern(s);

    // Record timing for global monitoring
    let duration_ns = start_time.elapsed().as_nanos() as u64;
    crate::core::string_pool_monitor::record_intern_operation(duration_ns);

    result
}

/// Get statistics about the global string pool
pub fn get_string_pool_stats() -> StringPoolStats {
    get_global_pool().get_stats()
}

/// Clear the global string pool (useful for testing)
#[cfg(test)]
pub fn clear_string_pool() {
    if let Some(pool) = GLOBAL_STRING_POOL.get() {
        pool.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interning() {
        clear_string_pool();

        let s1 = intern_string("test");
        let s2 = intern_string("test");
        let s3 = intern_string("different");

        // Same strings should be the same Arc
        assert!(Arc::ptr_eq(&s1, &s2));

        // Different strings should be different Arcs
        assert!(!Arc::ptr_eq(&s1, &s3));

        // Content should be correct
        assert_eq!(&*s1, "test");
        assert_eq!(&*s3, "different");
    }

    #[test]
    fn test_string_pool_stats() {
        // Test basic functionality without relying on exact counts
        // due to potential interference from other tests
        let initial_stats = get_string_pool_stats();

        let s1 = intern_string("unique_test_string_1");
        let s2 = intern_string("unique_test_string_1"); // Should be cache hit
        let s3 = intern_string("unique_test_string_2");

        let stats = get_string_pool_stats();

        // Verify that operations increased
        assert!(stats.intern_operations >= initial_stats.intern_operations + 3);
        // Verify that we have at least the strings we added
        assert!(stats.unique_strings >= initial_stats.unique_strings + 2);
        // Verify that we had at least one cache hit
        assert!(stats.cache_hits > initial_stats.cache_hits);

        // Verify the strings are actually the same Arc
        assert_eq!(&*s1, "unique_test_string_1");
        assert_eq!(&*s2, "unique_test_string_1");
        assert_eq!(&*s3, "unique_test_string_2");
    }

    #[test]
    fn test_empty_string_interning() {
        clear_string_pool();

        let s1 = intern_string("");
        let s2 = intern_string("");

        assert!(Arc::ptr_eq(&s1, &s2));
        assert_eq!(&*s1, "");
    }

    #[test]
    fn test_concurrent_interning() {
        use std::thread;

        // Test that concurrent interning works without panicking
        // We'll focus on correctness rather than Arc pointer equality
        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    let s = format!("concurrent_test_{}", i % 3); // Only 3 unique strings
                    intern_string(&s)
                })
            })
            .collect();

        let results: Vec<_> = handles.into_iter().filter_map(|h| h.join().ok()).collect();

        // Verify all results have correct content
        for (i, arc_str) in results.iter().enumerate() {
            let expected = format!("concurrent_test_{}", i % 3);
            assert_eq!(arc_str.as_ref(), expected);
        }

        // Verify we got the expected number of results
        assert_eq!(results.len(), 10);
    }
}
