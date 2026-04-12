//! Bounded History Implementation
//!
//! This module provides memory-bounded history tracking with automatic cleanup
//! and age-based expiration, addressing the unlimited memory growth issue
//! identified in the improvement plan.

use crate::core::{MemScopeError, MemScopeResult};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Configuration for bounded history behavior
#[derive(Debug, Clone)]
pub struct BoundedHistoryConfig {
    /// Maximum number of entries to keep
    pub max_entries: usize,
    /// Maximum age of entries before expiration
    pub max_age: Duration,
    /// Total memory limit in bytes
    pub total_memory_limit: usize,
    /// Cleanup threshold (percentage of max_entries)
    pub cleanup_threshold: f32,
}

impl Default for BoundedHistoryConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            max_age: Duration::from_secs(3600),   // 1 hour
            total_memory_limit: 50 * 1024 * 1024, // 50MB
            cleanup_threshold: 0.8,               // Cleanup when 80% full
        }
    }
}

/// Thread-safe bounded history with automatic cleanup
pub struct BoundedHistory<T> {
    /// Configuration parameters
    config: BoundedHistoryConfig,
    /// The actual history entries
    entries: Arc<Mutex<VecDeque<TimestampedEntry<T>>>>,
    /// Current estimated memory usage
    current_memory_usage: Arc<Mutex<usize>>,
    /// Operation statistics
    stats: Arc<RwLock<BoundedHistoryStats>>,
    /// Last cleanup timestamp
    last_cleanup: Arc<Mutex<Instant>>,
}

/// Timestamped entry wrapper for history tracking
#[derive(Debug, Clone)]
pub struct TimestampedEntry<T> {
    /// The actual data being stored
    pub data: T,
    /// Timestamp when this entry was created
    pub timestamp: Instant,
    /// Estimated memory size of this entry
    pub estimated_size: usize,
}

/// Statistics about bounded history operation
#[derive(Debug, Clone, Default)]
pub struct BoundedHistoryStats {
    /// Total entries added
    pub total_entries_added: u64,
    /// Total entries removed due to age
    pub entries_expired: u64,
    /// Total entries removed due to capacity
    pub entries_evicted: u64,
    /// Total cleanup operations performed
    pub cleanup_operations: u64,
    /// Current memory usage estimate
    pub current_memory_usage: usize,
    /// Peak memory usage observed
    pub peak_memory_usage: usize,
}

impl<T> TimestampedEntry<T> {
    /// Create a new timestamped entry
    pub fn new(data: T, estimated_size: usize) -> Self {
        Self {
            data,
            timestamp: Instant::now(),
            estimated_size,
        }
    }

    /// Check if this entry has expired based on max age
    pub fn is_expired(&self, max_age: Duration) -> bool {
        self.timestamp.elapsed() > max_age
    }

    /// Get the age of this entry
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }
}

impl<T> Default for BoundedHistory<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> BoundedHistory<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Create a new bounded history with default configuration
    pub fn new() -> Self {
        Self::with_config(BoundedHistoryConfig::default())
    }

    pub fn last_cleanup(&self) -> MemScopeResult<Instant> {
        let last = self.last_cleanup.lock().map_err(|e| {
            MemScopeError::system(
                crate::core::error::SystemErrorType::Locking,
                format!("Failed to acquire last_cleanup lock: {}", e),
            )
        })?;
        Ok(*last)
    }

    /// Create a new bounded history with custom configuration
    pub fn with_config(config: BoundedHistoryConfig) -> Self {
        Self {
            config,
            entries: Arc::new(Mutex::new(VecDeque::new())),
            current_memory_usage: Arc::new(Mutex::new(0)),
            stats: Arc::new(RwLock::new(BoundedHistoryStats::default())),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Add a new entry to the history
    pub fn push(&self, data: T) -> bool {
        // Improved memory estimation: calculate actual struct overhead
        // TimestampedEntry contains: data (T) + timestamp (Instant) + estimated_size (usize)
        let data_size = std::mem::size_of::<T>();
        let timestamp_size = std::mem::size_of::<Instant>();
        let size_field_size = std::mem::size_of::<usize>();
        let estimated_size = data_size + timestamp_size + size_field_size;

        let entry = TimestampedEntry::new(data, estimated_size);

        if let (Ok(mut entries), Ok(mut usage)) =
            (self.entries.lock(), self.current_memory_usage.lock())
        {
            // Check memory limit
            if *usage + estimated_size > self.config.total_memory_limit {
                self.evict_oldest_entries(estimated_size);
            }

            // Check entry count limit
            if entries.len() >= self.config.max_entries {
                if let Some(removed) = entries.pop_front() {
                    *usage = usage.saturating_sub(removed.estimated_size);
                }
            }

            entries.push_back(entry);
            *usage += estimated_size;

            // Update stats
            if let Ok(mut stats) = self.stats.write() {
                stats.total_entries_added += 1;
                stats.current_memory_usage = *usage;
                if *usage > stats.peak_memory_usage {
                    stats.peak_memory_usage = *usage;
                }
            }

            true
        } else {
            false
        }
    }

    pub fn entries(&self) -> Vec<T> {
        if let Ok(entries) = self.entries.lock() {
            entries.iter().map(|entry| entry.data.clone()).collect()
        } else {
            Vec::new()
        }
    }

    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
        if let Ok(mut usage) = self.current_memory_usage.lock() {
            *usage = 0;
        }
    }

    pub fn len(&self) -> usize {
        self.entries.lock().map(|e| e.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.lock().map(|e| e.is_empty()).unwrap_or(true)
    }

    pub fn get_memory_usage_stats(&self) -> BoundedHistoryStats {
        if let (Ok(entries), Ok(usage)) = (self.entries.lock(), self.current_memory_usage.lock()) {
            let _memory_usage_mb = *usage as f64 / (1024.0 * 1024.0);
            let _oldest_entry_age_secs = entries
                .front()
                .map(|entry| entry.timestamp.elapsed().as_secs_f64());
            let _average_entry_size = if entries.is_empty() {
                0.0
            } else {
                *usage as f64 / entries.len() as f64
            };

            if let Ok(stats) = self.stats.read() {
                stats.clone()
            } else {
                BoundedHistoryStats::default()
            }
        } else {
            BoundedHistoryStats::default()
        }
    }

    pub fn cleanup_expired(&self) -> usize {
        let cutoff = Instant::now() - self.config.max_age;
        let mut removed_count = 0;

        if let (Ok(mut entries), Ok(mut usage)) =
            (self.entries.lock(), self.current_memory_usage.lock())
        {
            while let Some(entry) = entries.front() {
                if entry.timestamp < cutoff {
                    if let Some(removed) = entries.pop_front() {
                        *usage = usage.saturating_sub(removed.estimated_size);
                        removed_count += 1;
                    }
                } else {
                    break;
                }
            }
        }
        removed_count
    }

    fn evict_oldest_entries(&self, needed_space: usize) {
        if let (Ok(mut entries), Ok(mut usage)) =
            (self.entries.lock(), self.current_memory_usage.lock())
        {
            let mut freed_space = 0;
            while freed_space < needed_space && !entries.is_empty() {
                if let Some(entry) = entries.pop_front() {
                    freed_space += entry.estimated_size;
                    *usage = usage.saturating_sub(entry.estimated_size);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_basic_functionality() {
        let config = BoundedHistoryConfig {
            max_entries: 3,
            max_age: Duration::from_secs(60),
            total_memory_limit: 1024 * 1024,
            cleanup_threshold: 0.8,
        };
        let history = BoundedHistory::with_config(config);

        assert!(history.push(1));
        assert!(history.push(2));
        assert!(history.push(3));
        assert_eq!(history.len(), 3);

        assert!(history.push(4));
        assert_eq!(history.len(), 3);

        let values = history.entries();
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_memory_stats() {
        let config = BoundedHistoryConfig {
            max_entries: 100,
            max_age: Duration::from_secs(60),
            total_memory_limit: 10 * 1024 * 1024,
            cleanup_threshold: 0.8,
        };
        let history = BoundedHistory::with_config(config);

        for i in 0..50 {
            history.push(i);
        }

        let _stats = history.get_memory_usage_stats();
    }

    #[test]
    fn test_bounded_history_config_default() {
        let config = BoundedHistoryConfig::default();

        assert_eq!(config.max_entries, 10_000);
        assert_eq!(config.max_age, Duration::from_secs(3600));
        assert_eq!(config.total_memory_limit, 50 * 1024 * 1024);
        assert!((config.cleanup_threshold - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_bounded_history_config_clone() {
        let config = BoundedHistoryConfig {
            max_entries: 100,
            max_age: Duration::from_secs(30),
            total_memory_limit: 1024,
            cleanup_threshold: 0.5,
        };

        let cloned = config.clone();
        assert_eq!(cloned.max_entries, 100);
        assert_eq!(cloned.max_age, Duration::from_secs(30));
    }

    #[test]
    fn test_bounded_history_config_debug() {
        let config = BoundedHistoryConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("BoundedHistoryConfig"));
        assert!(debug_str.contains("max_entries"));
    }

    #[test]
    fn test_timestamped_entry_new() {
        let entry = TimestampedEntry::new("test_data", 100);

        assert_eq!(entry.data, "test_data");
        assert_eq!(entry.estimated_size, 100);
    }

    #[test]
    fn test_timestamped_entry_is_expired() {
        let entry = TimestampedEntry::new(42, 8);

        // Should not be expired immediately
        assert!(!entry.is_expired(Duration::from_secs(60)));

        // Should be expired for zero duration
        thread::sleep(Duration::from_millis(1));
        assert!(entry.is_expired(Duration::from_nanos(1)));
    }

    #[test]
    fn test_timestamped_entry_age() {
        let entry = TimestampedEntry::new("test", 10);
        thread::sleep(Duration::from_millis(10));

        let age = entry.age();
        assert!(age >= Duration::from_millis(10));
    }

    #[test]
    fn test_timestamped_entry_clone() {
        let entry = TimestampedEntry::new(vec![1, 2, 3], 24);
        let cloned = entry.clone();

        assert_eq!(cloned.data, entry.data);
        assert_eq!(cloned.estimated_size, entry.estimated_size);
    }

    #[test]
    fn test_bounded_history_default() {
        let history = BoundedHistory::<i32>::default();
        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
    }

    #[test]
    fn test_bounded_history_with_config() {
        let config = BoundedHistoryConfig {
            max_entries: 10,
            max_age: Duration::from_secs(10),
            total_memory_limit: 1024,
            cleanup_threshold: 0.9,
        };

        let history = BoundedHistory::<String>::with_config(config);
        assert!(history.is_empty());
    }

    #[test]
    fn test_bounded_history_push_and_len() {
        let history = BoundedHistory::<i32>::default();

        assert!(history.push(1));
        assert!(history.push(2));
        assert_eq!(history.len(), 2);
        assert!(!history.is_empty());
    }

    #[test]
    fn test_bounded_history_entries() {
        let history = BoundedHistory::<i32>::default();
        history.push(10);
        history.push(20);
        history.push(30);

        let entries = history.entries();
        assert_eq!(entries.len(), 3);

        assert!(entries.contains(&10));
        assert!(entries.contains(&20));
        assert!(entries.contains(&30));
    }

    #[test]
    fn test_bounded_history_clear() {
        let history = BoundedHistory::<i32>::default();
        history.push(1);
        history.push(2);
        history.push(3);

        history.clear();
        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
    }

    #[test]
    fn test_bounded_history_stats_access() {
        let history = BoundedHistory::<i32>::default();
        history.push(1);
        history.push(2);

        let stats = history.get_memory_usage_stats();
        assert_eq!(stats.total_entries_added, 2);
    }

    #[test]
    fn test_bounded_history_stats_default() {
        let stats = BoundedHistoryStats::default();

        assert_eq!(stats.total_entries_added, 0);
        assert_eq!(stats.entries_expired, 0);
        assert_eq!(stats.entries_evicted, 0);
        assert_eq!(stats.cleanup_operations, 0);
        assert_eq!(stats.current_memory_usage, 0);
        assert_eq!(stats.peak_memory_usage, 0);
    }

    #[test]
    fn test_bounded_history_stats_clone() {
        let stats = BoundedHistoryStats {
            total_entries_added: 100,
            entries_expired: 10,
            entries_evicted: 5,
            cleanup_operations: 3,
            current_memory_usage: 1024,
            peak_memory_usage: 2048,
        };

        let cloned = stats.clone();
        assert_eq!(cloned.total_entries_added, 100);
        assert_eq!(cloned.peak_memory_usage, 2048);
    }

    #[test]
    fn test_bounded_history_stats_debug() {
        let stats = BoundedHistoryStats::default();
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("BoundedHistoryStats"));
    }

    #[test]
    fn test_bounded_history_capacity_eviction() {
        let config = BoundedHistoryConfig {
            max_entries: 2,
            max_age: Duration::from_secs(3600),
            total_memory_limit: 1024 * 1024,
            cleanup_threshold: 1.0,
        };
        let history = BoundedHistory::with_config(config);

        history.push(1);
        history.push(2);
        history.push(3);

        // Should have evicted one entry
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_bounded_history_get_memory_usage_stats() {
        let history = BoundedHistory::<String>::default();
        history.push("test1".to_string());
        history.push("test2".to_string());

        let stats = history.get_memory_usage_stats();
        assert_eq!(stats.total_entries_added, 2);
    }

    #[test]
    fn test_bounded_history_with_complex_type() {
        #[derive(Clone)]
        #[allow(dead_code)]
        struct TestData {
            id: u32,
            name: String,
        }

        let history = BoundedHistory::<TestData>::default();
        history.push(TestData {
            id: 1,
            name: "test".to_string(),
        });

        assert_eq!(history.len(), 1);
    }
}
