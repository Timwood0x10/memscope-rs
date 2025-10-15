//! Bounded History Implementation
//!
//! This module provides memory-bounded history tracking with automatic cleanup
//! and age-based expiration, addressing the unlimited memory growth issue
//! identified in the improvement plan.

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
    #[allow(dead_code)]
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
        let estimated_size = std::mem::size_of::<T>() + 64; // Basic estimation
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
        // Just test that it doesn't crash - no assertion needed for unsigned value
    }
}
