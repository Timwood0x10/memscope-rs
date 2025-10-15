use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Statistics for stack trace cache performance
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total cache lookups performed
    pub total_lookups: usize,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Number of cache misses
    pub cache_misses: usize,
    /// Current cache size (number of entries)
    pub cache_size: usize,
    /// Cache hit ratio (0.0 to 1.0)
    pub hit_ratio: f64,
}

/// High-performance cache for stack trace data
pub struct StackTraceCache<T> {
    /// Internal cache storage
    cache: HashMap<u64, T>,
    /// Total number of lookups
    lookup_count: AtomicUsize,
    /// Number of successful cache hits
    hit_count: AtomicUsize,
    /// Maximum cache size before eviction
    max_size: usize,
}

impl<T> StackTraceCache<T> {
    /// Create new cache with specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            lookup_count: AtomicUsize::new(0),
            hit_count: AtomicUsize::new(0),
            max_size,
        }
    }

    /// Get value from cache by key
    pub fn get(&mut self, key: u64) -> Option<&T> {
        self.lookup_count.fetch_add(1, Ordering::Relaxed);

        if let Some(value) = self.cache.get(&key) {
            self.hit_count.fetch_add(1, Ordering::Relaxed);
            Some(value)
        } else {
            None
        }
    }

    /// Insert value into cache
    pub fn insert(&mut self, key: u64, value: T) {
        // Evict oldest entries if cache is full
        if self.cache.len() >= self.max_size {
            self.evict_oldest();
        }

        self.cache.insert(key, value);
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.cache.clear();
        self.lookup_count.store(0, Ordering::Relaxed);
        self.hit_count.store(0, Ordering::Relaxed);
    }

    /// Get current cache statistics
    pub fn stats(&self) -> CacheStats {
        let lookups = self.lookup_count.load(Ordering::Relaxed);
        let hits = self.hit_count.load(Ordering::Relaxed);
        let misses = lookups.saturating_sub(hits);
        let hit_ratio = if lookups > 0 {
            hits as f64 / lookups as f64
        } else {
            0.0
        };

        CacheStats {
            total_lookups: lookups,
            cache_hits: hits,
            cache_misses: misses,
            cache_size: self.cache.len(),
            hit_ratio,
        }
    }

    /// Get current cache size
    pub fn size(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Simple eviction strategy - remove first entry
    fn evict_oldest(&mut self) {
        if let Some(key) = self.cache.keys().next().copied() {
            self.cache.remove(&key);
        }
    }
}

impl<T> Default for StackTraceCache<T> {
    fn default() -> Self {
        Self::new(1000) // Default cache size
    }
}

impl CacheStats {
    /// Check if cache performance is good
    pub fn is_performing_well(&self) -> bool {
        self.hit_ratio >= 0.8 && self.total_lookups > 10
    }

    /// Get cache efficiency description
    pub fn efficiency_description(&self) -> &'static str {
        match self.hit_ratio {
            x if x >= 0.9 => "Excellent",
            x if x >= 0.8 => "Good",
            x if x >= 0.6 => "Fair",
            x if x >= 0.4 => "Poor",
            _ => "Very Poor",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cache_operations() {
        let mut cache = StackTraceCache::new(3);

        assert!(cache.is_empty());
        assert_eq!(cache.size(), 0);

        cache.insert(1, "first");
        cache.insert(2, "second");

        assert_eq!(cache.size(), 2);
        assert!(!cache.is_empty());

        assert_eq!(cache.get(1), Some(&"first"));
        assert_eq!(cache.get(2), Some(&"second"));
        assert_eq!(cache.get(3), None);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = StackTraceCache::new(2);

        cache.insert(1, "first");
        cache.insert(2, "second");
        assert_eq!(cache.size(), 2);

        // Adding third item should evict one
        cache.insert(3, "third");
        assert_eq!(cache.size(), 2);

        // One item should be evicted (could be any due to HashMap ordering)
        let get1 = cache.get(1).is_some();
        let get2 = cache.get(2).is_some();
        let get3 = cache.get(3).is_some();
        let remaining_count = [get1, get2, get3].iter().filter(|&&x| x).count();
        assert_eq!(remaining_count, 2); // Only 2 items should remain
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = StackTraceCache::new(10);

        cache.insert(1, "value1");
        cache.insert(2, "value2");

        // Perform some lookups
        cache.get(1); // hit
        cache.get(2); // hit
        cache.get(3); // miss
        cache.get(1); // hit

        let stats = cache.stats();
        assert_eq!(stats.total_lookups, 4);
        assert_eq!(stats.cache_hits, 3);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.cache_size, 2);
        assert!((stats.hit_ratio - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = StackTraceCache::new(10);

        cache.insert(1, "value");
        cache.get(1);

        assert!(!cache.is_empty());
        assert!(cache.stats().total_lookups > 0);

        cache.clear();

        assert!(cache.is_empty());
        assert_eq!(cache.stats().total_lookups, 0);
        assert_eq!(cache.stats().cache_hits, 0);
    }

    #[test]
    fn test_efficiency_description() {
        let excellent = CacheStats {
            hit_ratio: 0.95,
            total_lookups: 100,
            cache_hits: 95,
            cache_misses: 5,
            cache_size: 50,
        };
        assert_eq!(excellent.efficiency_description(), "Excellent");

        let poor = CacheStats {
            hit_ratio: 0.45, // Changed to fall in "Poor" range (0.4-0.6)
            total_lookups: 100,
            cache_hits: 45,
            cache_misses: 55,
            cache_size: 20,
        };
        assert_eq!(poor.efficiency_description(), "Poor");
    }
}
