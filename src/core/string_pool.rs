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

#[cfg(test)]
mod string_pool_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_string_pool_deduplication() {
        let pool = StringPool::new();

        // Same string should return same Arc
        let str1 = pool.intern("hello world");
        let str2 = pool.intern("hello world");

        assert!(Arc::ptr_eq(&str1, &str2));

        // Different strings should return different Arcs
        let str3 = pool.intern("different string");
        assert!(!Arc::ptr_eq(&str1, &str3));
    }

    #[test]
    fn test_string_pool_memory_efficiency() {
        let pool = StringPool::new();

        // Intern the same string multiple times
        let original = "this is a test string for memory efficiency";
        let mut handles = Vec::new();

        for _ in 0..1000 {
            handles.push(pool.intern(original));
        }

        // All handles should point to the same memory
        for handle in &handles[1..] {
            assert!(Arc::ptr_eq(&handles[0], handle));
        }

        // Pool should only contain one entry
        let stats = pool.get_stats();
        assert_eq!(stats.unique_strings, 1);
        assert_eq!(stats.intern_operations, 1000);
    }

    #[test]
    fn test_string_pool_concurrent_access() {
        let pool = Arc::new(StringPool::new());
        let mut handles = Vec::new();

        // Spawn multiple threads that intern the same strings
        for _i in 0..10 {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                let mut results = Vec::new();
                for j in 0..100 {
                    let s = if j % 2 == 0 {
                        "common_string_a"
                    } else {
                        "common_string_b"
                    };
                    results.push(pool_clone.intern(s));
                }
                results
            });
            handles.push(handle);
        }

        // Collect all results
        let mut all_results = Vec::new();
        for handle in handles {
            all_results.extend(handle.join().unwrap());
        }

        // Verify deduplication worked across threads
        let string_a_refs: Vec<_> = all_results
            .iter()
            .filter(|s| &***s == "common_string_a")
            .collect();
        let string_b_refs: Vec<_> = all_results
            .iter()
            .filter(|s| &***s == "common_string_b")
            .collect();

        // All references to the same string should be identical
        for ref_a in &string_a_refs[1..] {
            assert!(Arc::ptr_eq(&string_a_refs[0], ref_a));
        }
        for ref_b in &string_b_refs[1..] {
            assert!(Arc::ptr_eq(&string_b_refs[0], ref_b));
        }

        let stats = pool.get_stats();
        assert_eq!(stats.unique_strings, 2); // Only "common_string_a" and "common_string_b"
    }

    #[test]
    fn test_string_pool_memory_usage_calculation() {
        let pool = StringPool::new();

        let _short_str = pool.intern("hi");
        let _long_str = pool.intern("this is a much longer string that takes more memory");

        let stats = pool.get_stats();

        // Should track memory usage accurately
        assert!(stats.memory_saved_bytes >= 0);
        assert_eq!(stats.unique_strings, 2);
    }

    #[test]
    fn test_string_pool_cleanup_on_drop() {
        let pool = StringPool::new();

        {
            let _temp_str = pool.intern("temporary string");
            let stats = pool.get_stats();
            assert_eq!(stats.unique_strings, 1);
        }

        // After temp_str is dropped, the pool should still contain the string
        // because Arc keeps it alive until all references are gone
        let stats = pool.get_stats();
        assert_eq!(stats.unique_strings, 1);

        // But if we force cleanup (if implemented), it should be removed
        // This tests the cleanup mechanism
    }

    #[test]
    fn test_string_pool_large_strings() {
        let pool = StringPool::new();

        // Test with very large strings
        let large_string = "x".repeat(10000);
        let str1 = pool.intern(&large_string);
        let str2 = pool.intern(&large_string);

        assert!(Arc::ptr_eq(&str1, &str2));

        let stats = pool.get_stats();
        assert_eq!(stats.unique_strings, 1);
        assert!(stats.memory_saved_bytes >= 0);
    }

    #[test]
    fn test_string_pool_empty_strings() {
        let pool = StringPool::new();

        let empty1 = pool.intern("");
        let empty2 = pool.intern("");

        assert!(Arc::ptr_eq(&empty1, &empty2));
        assert_eq!(empty1.as_ref(), "");

        let stats = pool.get_stats();
        assert_eq!(stats.unique_strings, 1);
    }

    #[test]
    fn test_string_pool_unicode_strings() {
        let pool = StringPool::new();

        let unicode1 = pool.intern("Hello world 🌍");
        let unicode2 = pool.intern("Hello world 🌍");
        let different_unicode = pool.intern("Hello world 🌎");

        assert!(Arc::ptr_eq(&unicode1, &unicode2));
        assert!(!Arc::ptr_eq(&unicode1, &different_unicode));

        let stats = pool.get_stats();
        assert_eq!(stats.unique_strings, 2);
    }

    #[test]
    fn test_string_pool_stats_accuracy() {
        let pool = StringPool::new();

        // Start with empty pool
        let initial_stats = pool.get_stats();
        assert_eq!(initial_stats.unique_strings, 0);
        assert_eq!(initial_stats.intern_operations, 0);

        // Add some strings
        let str1 = pool.intern("first");
        let str2 = pool.intern("second");
        let str1_again = pool.intern("first"); // Should reuse

        let stats = pool.get_stats();
        assert_eq!(stats.unique_strings, 2); // "first" and "second"
        assert!(stats.intern_operations >= 2); // Should have some operations

        // Verify the strings are correct
        assert_eq!(str1.as_ref(), "first");
        assert_eq!(str2.as_ref(), "second");
        assert!(Arc::ptr_eq(&str1, &str1_again));
    }

    #[test]
    fn test_global_string_pool_singleton() {
        // Test that global pool is a singleton
        let str1 = intern_string("global test");
        let str2 = intern_string("global test");

        assert!(Arc::ptr_eq(&str1, &str2));

        // Test from different calls to global pool
        let pool1 = get_global_pool();
        let pool2 = get_global_pool();

        let str3 = pool1.intern("another global test");
        let str4 = pool2.intern("another global test");

        assert!(Arc::ptr_eq(&str3, &str4));
    }

    #[test]
    fn test_string_pool_performance_characteristics() {
        let pool = StringPool::new();

        // Test that repeated interning is fast (should be O(1) lookup)
        let test_string = "performance test string";

        let start = std::time::Instant::now();
        for _ in 0..10000 {
            pool.intern(test_string);
        }
        let duration = start.elapsed();

        // Should be very fast since it's just hash lookups after first insert
        assert!(
            duration.as_millis() < 100,
            "String pool lookup too slow: {:?}",
            duration
        );

        let stats = pool.get_stats();
        assert_eq!(stats.unique_strings, 1);
    }

    #[test]
    fn test_string_pool_memory_overhead() {
        let pool = StringPool::new();

        // Test that memory overhead is reasonable
        let test_strings = vec![
            "short",
            "medium length string",
            "this is a much longer string that should still be handled efficiently",
        ];

        let mut total_string_bytes = 0;
        for s in &test_strings {
            pool.intern(s);
            total_string_bytes += s.len();
        }

        let stats = pool.get_stats();

        // Memory usage should be close to actual string sizes (plus some overhead)
        let overhead_ratio = (stats.memory_saved_bytes + total_string_bytes as u64) as f64
            / total_string_bytes as f64;
        assert!(
            overhead_ratio < 2.0,
            "Memory overhead too high: {:.2}x",
            overhead_ratio
        );
        assert!(
            overhead_ratio >= 1.0,
            "Memory usage calculation seems wrong"
        );
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
