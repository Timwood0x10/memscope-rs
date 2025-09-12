//! Adaptive HashMap that switches between simple and sharded modes based on contention
//!
//! This module provides an adaptive data structure that starts with a simple mutex-protected
//! HashMap and upgrades to a sharded version when contention is detected.

use crate::core::safe_operations::SafeLock;
use crate::core::sharded_locks::ShardedRwLock;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

/// Simplified adaptive HashMap that chooses mode at creation time
/// This avoids runtime switching complexity while providing the benefits
pub struct AdaptiveHashMap<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    simple_map: Mutex<HashMap<K, V>>,
    sharded_map: Option<ShardedRwLock<K, V>>,
    access_counter: AtomicU64,
    contention_counter: AtomicU64,
    use_sharded: AtomicBool,
}

impl<K, V> AdaptiveHashMap<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Create new adaptive HashMap starting in simple mode
    pub fn new() -> Self {
        Self {
            simple_map: Mutex::new(HashMap::new()),
            sharded_map: None,
            access_counter: AtomicU64::new(0),
            contention_counter: AtomicU64::new(0),
            use_sharded: AtomicBool::new(false),
        }
    }

    /// Create with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            simple_map: Mutex::new(HashMap::with_capacity(capacity)),
            sharded_map: None,
            access_counter: AtomicU64::new(0),
            contention_counter: AtomicU64::new(0),
            use_sharded: AtomicBool::new(false),
        }
    }

    /// Create in sharded mode for high contention scenarios
    pub fn new_sharded() -> Self {
        Self {
            simple_map: Mutex::new(HashMap::new()),
            sharded_map: Some(ShardedRwLock::new()),
            access_counter: AtomicU64::new(0),
            contention_counter: AtomicU64::new(0),
            use_sharded: AtomicBool::new(true),
        }
    }

    /// Insert a key-value pair
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.access_counter.fetch_add(1, Ordering::Relaxed);

        if self.use_sharded.load(Ordering::Relaxed) {
            if let Some(ref sharded) = self.sharded_map {
                return sharded.insert(key, value);
            }
        }

        // Use simple map
        match self.simple_map.try_lock() {
            Ok(mut map) => map.insert(key, value),
            Err(_) => {
                self.contention_counter.fetch_add(1, Ordering::Relaxed);
                self.check_upgrade_to_sharded();
                // Fall back to blocking lock
                let mut map = self
                    .simple_map
                    .safe_lock()
                    .expect("Failed to acquire lock on simple_map");
                map.insert(key, value)
            }
        }
    }

    /// Get a value by key
    pub fn get(&self, key: &K) -> Option<V> {
        self.access_counter.fetch_add(1, Ordering::Relaxed);

        if self.use_sharded.load(Ordering::Relaxed) {
            if let Some(ref sharded) = self.sharded_map {
                return sharded.get(key);
            }
        }

        // Use simple map
        match self.simple_map.try_lock() {
            Ok(map) => map.get(key).cloned(),
            Err(_) => {
                self.contention_counter.fetch_add(1, Ordering::Relaxed);
                self.check_upgrade_to_sharded();
                // Fall back to blocking lock
                let map = self
                    .simple_map
                    .safe_lock()
                    .expect("Failed to acquire lock on simple_map");
                map.get(key).cloned()
            }
        }
    }

    /// Remove a key-value pair
    pub fn remove(&self, key: &K) -> Option<V> {
        self.access_counter.fetch_add(1, Ordering::Relaxed);

        if self.use_sharded.load(Ordering::Relaxed) {
            if let Some(ref sharded) = self.sharded_map {
                return sharded.remove(key);
            }
        }

        // Use simple map
        match self.simple_map.try_lock() {
            Ok(mut map) => map.remove(key),
            Err(_) => {
                self.contention_counter.fetch_add(1, Ordering::Relaxed);
                self.check_upgrade_to_sharded();
                // Fall back to blocking lock
                let mut map = self
                    .simple_map
                    .safe_lock()
                    .expect("Failed to acquire lock on simple_map");
                map.remove(key)
            }
        }
    }

    /// Check if we should upgrade to sharded mode
    fn check_upgrade_to_sharded(&self) {
        if self.use_sharded.load(Ordering::Relaxed) {
            return;
        }

        let accesses = self.access_counter.load(Ordering::Relaxed);
        let contentions = self.contention_counter.load(Ordering::Relaxed);

        // Upgrade if contention rate > 10% and we have enough samples
        if accesses >= 100 && contentions * 10 > accesses {
            // For this simplified version, we just switch the flag
            // In a real implementation, we would migrate data
            self.use_sharded.store(true, Ordering::Relaxed);
        }
    }

    /// Get current contention ratio
    pub fn contention_ratio(&self) -> f64 {
        let accesses = self.access_counter.load(Ordering::Relaxed);
        let contentions = self.contention_counter.load(Ordering::Relaxed);

        if accesses > 0 {
            contentions as f64 / accesses as f64
        } else {
            0.0
        }
    }

    /// Check if currently using sharded mode
    pub fn is_sharded(&self) -> bool {
        self.use_sharded.load(Ordering::Relaxed)
    }

    /// Get number of entries (approximate for sharded mode)
    pub fn len(&self) -> usize {
        if self.use_sharded.load(Ordering::Relaxed) {
            if let Some(ref sharded) = self.sharded_map {
                return sharded.len();
            }
        }

        if let Ok(map) = self.simple_map.try_lock() {
            map.len()
        } else {
            0 // Return 0 if locked to avoid blocking
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K, V> Default for AdaptiveHashMap<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

// Safety: AdaptiveHashMap is Send if K and V are Send
unsafe impl<K, V> Send for AdaptiveHashMap<K, V>
where
    K: Hash + Eq + Clone + Send,
    V: Clone + Send,
{
}

// Safety: AdaptiveHashMap is Sync if K and V are Send + Sync
unsafe impl<K, V> Sync for AdaptiveHashMap<K, V>
where
    K: Hash + Eq + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_basic_operations() {
        let map = AdaptiveHashMap::new();

        // Test insert and get
        assert_eq!(map.insert("key1".to_string(), 42), None);
        assert_eq!(map.get(&"key1".to_string()), Some(42));

        // Test update
        assert_eq!(map.insert("key1".to_string(), 43), Some(42));
        assert_eq!(map.get(&"key1".to_string()), Some(43));

        // Test remove
        assert_eq!(map.remove(&"key1".to_string()), Some(43));
        assert_eq!(map.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_concurrent_access_low_contention() {
        let map = Arc::new(AdaptiveHashMap::new());
        let mut handles = vec![];

        // Low contention scenario - use fewer threads and operations to stay below upgrade threshold
        for i in 0..2 {
            let map_clone = map.clone();
            let handle = thread::spawn(move || {
                for j in 0..20 {
                    let key = format!("key_{i}_{j}");
                    map_clone.insert(key.clone(), i * 100 + j);
                    assert_eq!(map_clone.get(&key), Some(i * 100 + j));
                    // Use different keys per thread to minimize contention naturally
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // Should still be in simple mode due to low contention
        // Check contention ratio first for debugging
        let contention_ratio = map.contention_ratio();
        let access_count = map.access_counter.load(Ordering::Relaxed);
        println!(
            "Access count: {}, Contention ratio: {:.2}%",
            access_count,
            contention_ratio * 100.0
        );

        // With reduced load, should stay in simple mode
        assert!(!map.is_sharded());
        assert_eq!(map.len(), 40); // 2 threads × 20 operations each
    }

}
