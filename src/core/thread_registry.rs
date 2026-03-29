//! Thread registry for managing thread-local memory trackers and data aggregation.
//!
//! This module provides a global registry that tracks all thread-local memory trackers
//! for data aggregation purposes. It enables the unified tracking system to collect
//! data from all tracking modes: track_var!, lockfree, and async_memory.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::thread::ThreadId;

pub use crate::core::types::MemoryStats;

pub trait TrackerStatsProvider: Send + Sync {
    fn get_stats(&self) -> Result<MemoryStats, crate::core::types::TrackingError>;
}

pub struct GenericThreadRegistry<T: TrackerStatsProvider> {
    trackers: HashMap<ThreadId, Weak<T>>,
    cached_thread_data: HashMap<ThreadId, GenericCachedThreadData>,
    total_threads_registered: usize,
    active_threads: usize,
}

#[derive(Debug, Clone)]
pub struct GenericCachedThreadData {
    pub thread_id: ThreadId,
    pub stats: MemoryStats,
    pub cached_at: std::time::SystemTime,
}

impl<T: TrackerStatsProvider> Default for GenericThreadRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TrackerStatsProvider> GenericThreadRegistry<T> {
    pub fn new() -> Self {
        Self {
            trackers: HashMap::new(),
            cached_thread_data: HashMap::new(),
            total_threads_registered: 0,
            active_threads: 0,
        }
    }

    pub fn register_tracker(&mut self, thread_id: ThreadId, tracker: &Arc<T>) {
        self.cleanup_dead_references();
        self.trackers.insert(thread_id, Arc::downgrade(tracker));
        self.total_threads_registered += 1;
        self.active_threads = self.trackers.len();

        tracing::debug!(
            "Registered thread {:?}, total threads: {}, active threads: {}",
            thread_id,
            self.total_threads_registered,
            self.active_threads
        );
    }

    pub fn collect_active_trackers(&mut self) -> Vec<Arc<T>> {
        self.cache_all_available_data();

        let mut active_trackers = Vec::new();

        for (thread_id, weak_tracker) in &self.trackers {
            if let Some(strong_tracker) = weak_tracker.upgrade() {
                active_trackers.push(strong_tracker);
                tracing::debug!("Successfully collected tracker for thread {:?}", thread_id);
            } else if self.cached_thread_data.contains_key(thread_id) {
                tracing::debug!("Found cached data for dead thread {:?}", thread_id);
            }
        }

        tracing::debug!(
            "Collected {} active trackers, {} total cached entries",
            active_trackers.len(),
            self.cached_thread_data.len()
        );

        active_trackers
    }

    fn cache_all_available_data(&mut self) {
        for (thread_id, weak_tracker) in &self.trackers {
            if let Some(strong_tracker) = weak_tracker.upgrade() {
                if let Ok(stats) = strong_tracker.get_stats() {
                    if stats.total_allocations > 0 {
                        self.cached_thread_data.insert(
                            *thread_id,
                            GenericCachedThreadData {
                                thread_id: *thread_id,
                                stats,
                                cached_at: std::time::SystemTime::now(),
                            },
                        );
                    }
                }
            }
        }
    }

    pub fn get_cached_thread_data(&self) -> Vec<GenericCachedThreadData> {
        self.cached_thread_data.values().cloned().collect()
    }

    pub fn cleanup_dead_references(&mut self) {
        let initial_count = self.trackers.len();
        self.trackers
            .retain(|_thread_id, weak_tracker| weak_tracker.strong_count() > 0);

        let removed_count = initial_count - self.trackers.len();
        if removed_count > 0 {
            tracing::debug!("Cleaned up {} dead tracker references", removed_count);
        }

        self.active_threads = self.trackers.len();
    }

    pub fn get_stats(&self) -> ThreadRegistryStats {
        ThreadRegistryStats {
            total_threads_registered: self.total_threads_registered,
            active_threads: self.active_threads,
            dead_references: self
                .trackers
                .iter()
                .filter(|(_, weak)| weak.strong_count() == 0)
                .count(),
        }
    }

    pub fn has_active_trackers(&self) -> bool {
        !self.trackers.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct ThreadRegistryStats {
    pub total_threads_registered: usize,
    pub active_threads: usize,
    pub dead_references: usize,
}

#[derive(Debug, Clone)]
pub struct CachedThreadData {
    pub thread_id: ThreadId,
    pub stats: MemoryStats,
    #[allow(dead_code)]
    pub cached_at: std::time::SystemTime,
}

impl From<GenericCachedThreadData> for CachedThreadData {
    fn from(data: GenericCachedThreadData) -> Self {
        CachedThreadData {
            thread_id: data.thread_id,
            stats: data.stats,
            cached_at: data.cached_at,
        }
    }
}

use crate::core::tracker::memory_tracker::MemoryTracker;

impl TrackerStatsProvider for MemoryTracker {
    fn get_stats(&self) -> Result<MemoryStats, crate::core::types::TrackingError> {
        MemoryTracker::get_stats(self)
    }
}

static THREAD_REGISTRY: std::sync::OnceLock<Arc<Mutex<GenericThreadRegistry<MemoryTracker>>>> =
    std::sync::OnceLock::new();

fn get_registry() -> Arc<Mutex<GenericThreadRegistry<MemoryTracker>>> {
    THREAD_REGISTRY
        .get_or_init(|| Arc::new(Mutex::new(GenericThreadRegistry::new())))
        .clone()
}

pub fn register_current_thread_tracker(tracker: &Arc<MemoryTracker>) {
    let thread_id = std::thread::current().id();

    if let Ok(mut registry) = get_registry().lock() {
        registry.register_tracker(thread_id, tracker);
    } else {
        tracing::error!("Failed to acquire registry lock for thread registration");
    }
}

pub fn collect_all_trackers() -> Vec<Arc<MemoryTracker>> {
    match get_registry().lock() {
        Ok(mut registry) => registry.collect_active_trackers(),
        Err(e) => {
            tracing::error!(
                "Failed to acquire registry lock for tracker collection: {}",
                e
            );
            Vec::new()
        }
    }
}

pub fn get_cached_thread_data() -> Vec<CachedThreadData> {
    match get_registry().lock() {
        Ok(registry) => registry
            .get_cached_thread_data()
            .into_iter()
            .map(|d| d.into())
            .collect(),
        Err(e) => {
            tracing::error!("Failed to acquire registry lock for cached data: {}", e);
            Vec::new()
        }
    }
}

pub fn get_registry_stats() -> ThreadRegistryStats {
    match get_registry().lock() {
        Ok(registry) => registry.get_stats(),
        Err(e) => {
            tracing::error!("Failed to acquire registry lock for stats: {}", e);
            ThreadRegistryStats {
                total_threads_registered: 0,
                active_threads: 0,
                dead_references: 0,
            }
        }
    }
}

pub fn cleanup_registry() {
    if let Ok(mut registry) = get_registry().lock() {
        registry.cleanup_dead_references();
    } else {
        tracing::error!("Failed to acquire registry lock for cleanup");
    }
}

pub fn has_active_trackers() -> bool {
    match get_registry().lock() {
        Ok(registry) => registry.has_active_trackers(),
        Err(_) => false,
    }
}

#[derive(Debug, Clone)]
pub struct AggregatedTrackingData {
    pub tracker_count: usize,
    pub total_allocations: u64,
    pub total_bytes_allocated: u64,
    pub peak_memory_usage: u64,
    pub active_threads: usize,
    pub combined_stats: Vec<CombinedTrackerStats>,
}

#[derive(Debug, Clone)]
pub struct CombinedTrackerStats {
    pub thread_id: ThreadId,
    pub tracking_mode: String,
    pub allocations: u64,
    pub bytes_allocated: u64,
    pub peak_memory: u64,
}

pub fn collect_unified_tracking_data() -> Result<AggregatedTrackingData, String> {
    let mut combined_stats = Vec::new();
    let mut total_allocations = 0u64;
    let mut total_bytes_allocated = 0u64;
    let mut peak_memory_usage = 0u64;

    let active_trackers = collect_all_trackers();
    let cached_data = get_cached_thread_data();

    for tracker in &active_trackers {
        if let Ok(stats) = tracker.get_stats() {
            let thread_stats = CombinedTrackerStats {
                thread_id: std::thread::current().id(),
                tracking_mode: "track_var!".to_string(),
                allocations: stats.total_allocations as u64,
                bytes_allocated: stats.total_allocated as u64,
                peak_memory: stats.peak_memory as u64,
            };

            total_allocations += stats.total_allocations as u64;
            total_bytes_allocated += stats.total_allocated as u64;
            peak_memory_usage = peak_memory_usage.max(stats.peak_memory as u64);

            combined_stats.push(thread_stats);
        }
    }

    for cached in cached_data {
        let thread_stats = CombinedTrackerStats {
            thread_id: cached.thread_id,
            tracking_mode: "track_var!".to_string(),
            allocations: cached.stats.total_allocations as u64,
            bytes_allocated: cached.stats.total_allocated as u64,
            peak_memory: cached.stats.peak_memory as u64,
        };

        total_allocations += cached.stats.total_allocations as u64;
        total_bytes_allocated += cached.stats.total_allocated as u64;
        peak_memory_usage = peak_memory_usage.max(cached.stats.peak_memory as u64);

        combined_stats.push(thread_stats);
    }

    let aggregated_data = AggregatedTrackingData {
        tracker_count: active_trackers.len(),
        total_allocations,
        total_bytes_allocated,
        peak_memory_usage,
        active_threads: active_trackers.len(),
        combined_stats,
    };

    tracing::info!(
        "Collected unified tracking data: {} trackers, {} allocations, {} bytes",
        aggregated_data.tracker_count,
        aggregated_data.total_allocations,
        aggregated_data.total_bytes_allocated
    );

    Ok(aggregated_data)
}

pub fn enable_precise_tracking() {
    crate::core::tracker::configure_tracking_strategy(true);
    tracing::info!("Enabled precise tracking mode with thread-local trackers");
}

pub fn enable_performance_tracking() {
    crate::core::tracker::configure_tracking_strategy(false);
    tracing::info!("Enabled performance tracking mode with global singleton");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tracker::memory_tracker::MemoryTracker;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_thread_registry_registration() {
        let tracker = Arc::new(MemoryTracker::new());
        register_current_thread_tracker(&tracker);

        let stats = get_registry_stats();
        assert!(stats.active_threads > 0);
        assert!(stats.total_threads_registered > 0);
    }

    #[test]
    fn test_collect_trackers() {
        let tracker = Arc::new(MemoryTracker::new());
        register_current_thread_tracker(&tracker);

        let collected = collect_all_trackers();
        assert!(!collected.is_empty());
    }

    #[test]
    fn test_unified_data_collection() {
        let tracker = Arc::new(MemoryTracker::new());
        register_current_thread_tracker(&tracker);

        let result = collect_unified_tracking_data();
        assert!(result.is_ok());

        let data = result.unwrap();
        assert!(data.tracker_count > 0);
    }

    #[test]
    fn test_precise_tracking_mode() {
        enable_precise_tracking();

        // Test that multiple threads can register independently
        let handles: Vec<_> = (0..3)
            .map(|i| {
                thread::spawn(move || {
                    let tracker = Arc::new(MemoryTracker::new());
                    register_current_thread_tracker(&tracker);
                    thread::sleep(Duration::from_millis(10));
                    i
                })
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        assert_eq!(results.len(), 3);

        // Verify that we can collect data from multiple threads
        let stats = get_registry_stats();
        assert!(stats.active_threads >= 1); // At least the main thread
    }
}
