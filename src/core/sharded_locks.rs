//! Sharded lock system for reducing lock contention
//!
//! This module provides a sharded locking mechanism that distributes
//! lock contention across multiple shards, improving concurrent performance.

use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

/// Number of shards to use by default
const DEFAULT_SHARD_COUNT: usize = 16;

/// Sharded read-write lock for concurrent access
#[derive(Debug)]
pub struct ShardedRwLock<K, V>
where
    K: Hash + Eq,
{
    shards: Vec<RwLock<HashMap<K, V>>>,
    shard_count: usize,
}

impl<K, V> ShardedRwLock<K, V>
where
    K: Hash + Eq,
{
    /// Create a new sharded RwLock with default shard count
    pub fn new() -> Self {
        Self::with_shard_count(DEFAULT_SHARD_COUNT)
    }

    /// Create a new sharded RwLock with specified shard count
    pub fn with_shard_count(shard_count: usize) -> Self {
        let mut shards = Vec::with_capacity(shard_count);
        for _ in 0..shard_count {
            shards.push(RwLock::new(HashMap::new()));
        }

        Self {
            shards,
            shard_count,
        }
    }

    /// Get the shard index for a given key
    fn get_shard_index<Q>(&self, key: &Q) -> usize
    where
        Q: Hash + ?Sized,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.shard_count
    }

    /// Insert a key-value pair
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let shard_index = self.get_shard_index(&key);
        let mut shard = self.shards[shard_index].write();
        shard.insert(key, value)
    }

    /// Get a value by key
    pub fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
        V: Clone,
    {
        let shard_index = self.get_shard_index(key);
        let shard = self.shards[shard_index].read();
        shard.get(key).cloned()
    }

    /// Remove a key-value pair
    pub fn remove<Q>(&self, key: &Q) -> Option<V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let shard_index = self.get_shard_index(key);
        let mut shard = self.shards[shard_index].write();
        shard.remove(key)
    }

    /// Check if a key exists
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let shard_index = self.get_shard_index(key);
        let shard = self.shards[shard_index].read();
        shard.contains_key(key)
    }

    /// Get the total number of entries across all shards
    pub fn len(&self) -> usize {
        self.shards.iter().map(|shard| shard.read().len()).sum()
    }

    /// Check if the sharded map is empty
    pub fn is_empty(&self) -> bool {
        self.shards.iter().all(|shard| shard.read().is_empty())
    }

    /// Clear all entries from all shards
    pub fn clear(&self) {
        for shard in &self.shards {
            shard.write().clear();
        }
    }

    /// Execute a function with read access to a specific shard
    pub fn with_shard_read<Q, F, R>(&self, key: &Q, f: F) -> R
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
        F: FnOnce(&HashMap<K, V>) -> R,
    {
        let shard_index = self.get_shard_index(key);
        let shard = self.shards[shard_index].read();
        f(&*shard)
    }

    /// Execute a function with write access to a specific shard
    pub fn with_shard_write<Q, F, R>(&self, key: &Q, f: F) -> R
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
        F: FnOnce(&mut HashMap<K, V>) -> R,
    {
        let shard_index = self.get_shard_index(key);
        let mut shard = self.shards[shard_index].write();
        f(&mut *shard)
    }

    /// Get statistics about shard distribution
    pub fn shard_stats(&self) -> ShardStats {
        let shard_sizes: Vec<usize> = self.shards.iter().map(|shard| shard.read().len()).collect();

        let total_entries: usize = shard_sizes.iter().sum();
        let max_shard_size = shard_sizes.iter().max().copied().unwrap_or(0);
        let min_shard_size = shard_sizes.iter().min().copied().unwrap_or(0);
        let avg_shard_size = if self.shard_count > 0 {
            total_entries as f64 / self.shard_count as f64
        } else {
            0.0
        };

        ShardStats {
            shard_count: self.shard_count,
            total_entries,
            max_shard_size,
            min_shard_size,
            avg_shard_size,
            shard_sizes,
        }
    }
}

impl<K, V> Default for ShardedRwLock<K, V>
where
    K: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about shard distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardStats {
    pub shard_count: usize,
    pub total_entries: usize,
    pub max_shard_size: usize,
    pub min_shard_size: usize,
    pub avg_shard_size: f64,
    pub shard_sizes: Vec<usize>,
}

impl ShardStats {
    /// Calculate load balance ratio (0.0 = perfectly balanced, 1.0 = completely unbalanced)
    pub fn load_balance_ratio(&self) -> f64 {
        if self.total_entries == 0 || self.avg_shard_size == 0.0 {
            return 0.0;
        }

        let variance: f64 = self
            .shard_sizes
            .iter()
            .map(|&size| {
                let diff = size as f64 - self.avg_shard_size;
                diff * diff
            })
            .sum::<f64>()
            / self.shard_count as f64;

        let std_dev = variance.sqrt();
        std_dev / self.avg_shard_size
    }
}

/// Sharded mutex for exclusive access
#[derive(Debug)]
pub struct ShardedMutex<K, V>
where
    K: Hash + Eq,
{
    shards: Vec<Mutex<HashMap<K, V>>>,
    shard_count: usize,
}

impl<K, V> ShardedMutex<K, V>
where
    K: Hash + Eq,
{
    /// Create a new sharded Mutex with default shard count
    pub fn new() -> Self {
        Self::with_shard_count(DEFAULT_SHARD_COUNT)
    }

    /// Create a new sharded Mutex with specified shard count
    pub fn with_shard_count(shard_count: usize) -> Self {
        let mut shards = Vec::with_capacity(shard_count);
        for _ in 0..shard_count {
            shards.push(Mutex::new(HashMap::new()));
        }

        Self {
            shards,
            shard_count,
        }
    }

    /// Get the shard index for a given key
    fn get_shard_index<Q>(&self, key: &Q) -> usize
    where
        Q: Hash + ?Sized,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.shard_count
    }

    /// Insert a key-value pair
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let shard_index = self.get_shard_index(&key);
        let mut shard = self.shards[shard_index].lock();
        shard.insert(key, value)
    }

    /// Get a value by key
    pub fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
        V: Clone,
    {
        let shard_index = self.get_shard_index(key);
        let shard = self.shards[shard_index].lock();
        shard.get(key).cloned()
    }

    /// Remove a key-value pair
    pub fn remove<Q>(&self, key: &Q) -> Option<V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let shard_index = self.get_shard_index(key);
        let mut shard = self.shards[shard_index].lock();
        shard.remove(key)
    }

    /// Execute a function with exclusive access to a specific shard
    pub fn with_shard<Q, F, R>(&self, key: &Q, f: F) -> R
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
        F: FnOnce(&mut HashMap<K, V>) -> R,
    {
        let shard_index = self.get_shard_index(key);
        let mut shard = self.shards[shard_index].lock();
        f(&mut *shard)
    }

    /// Get the total number of entries across all shards
    pub fn len(&self) -> usize {
        self.shards.iter().map(|shard| shard.lock().len()).sum()
    }

    /// Check if the sharded map is empty
    pub fn is_empty(&self) -> bool {
        self.shards.iter().all(|shard| shard.lock().is_empty())
    }
}

impl<K, V> Default for ShardedMutex<K, V>
where
    K: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    

    #[test]
    fn test_sharded_rwlock_basic_operations() {
        let sharded = ShardedRwLock::new();

        // Test insert and get
        assert_eq!(sharded.insert("key1", "value1"), None);
        assert_eq!(sharded.get("key1"), Some("value1"));

        // Test update
        assert_eq!(sharded.insert("key1", "value2"), Some("value1"));
        assert_eq!(sharded.get("key1"), Some("value2"));

        // Test remove
        assert_eq!(sharded.remove("key1"), Some("value2"));
        assert_eq!(sharded.get("key1"), None);
    }

    #[test]
    fn test_shard_stats() {
        let sharded = ShardedRwLock::with_shard_count(4);

        // Insert some data
        for i in 0..100 {
            sharded.insert(i, format!("value_{i}"));
        }

        let stats = sharded.shard_stats();
        assert_eq!(stats.shard_count, 4);
        assert_eq!(stats.total_entries, 100);
        assert!(stats.avg_shard_size > 0.0);
        assert!(stats.load_balance_ratio() >= 0.0);
    }
}
