//! Thread registry for managing thread-local memory trackers and data aggregation.
//!
//! This module provides a global registry that tracks all thread-local memory trackers
//! for data aggregation purposes. It enables the unified tracking system to collect
//! data from all tracking modes: track_var!, lockfree, and async_memory.

use crate::core::tracker::memory_tracker::MemoryTracker;
use crate::core::types::MemoryStats;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::thread::ThreadId;

/// Global thread registry for tracking all thread-local memory trackers
static THREAD_REGISTRY: std::sync::OnceLock<Arc<Mutex<ThreadRegistry>>> =
    std::sync::OnceLock::new();

/// Thread registry that maintains weak references to all thread-local trackers
struct ThreadRegistry {
    /// Map of thread ID to weak reference of memory tracker
    trackers: HashMap<ThreadId, Weak<MemoryTracker>>,
    /// Cached data from completed threads (persisted after thread exit)
    cached_thread_data: HashMap<ThreadId, CachedThreadData>,
    /// Total number of threads ever registered
    total_threads_registered: usize,
    /// Number of currently active threads
    active_threads: usize,
}

/// Cached tracking data from a completed thread
#[derive(Debug, Clone)]
pub struct CachedThreadData {
    /// Thread ID
    pub thread_id: ThreadId,
    /// Cached memory stats
    pub stats: MemoryStats,
    /// Timestamp when data was cached
    #[allow(dead_code)]
    pub cached_at: std::time::SystemTime,
}

impl ThreadRegistry {
    /// Create a new thread registry
    fn new() -> Self {
        Self {
            trackers: HashMap::new(),
            cached_thread_data: HashMap::new(),
            total_threads_registered: 0,
            active_threads: 0,
        }
    }

    /// Register a thread-local tracker
    fn register_tracker(&mut self, thread_id: ThreadId, tracker: &Arc<MemoryTracker>) {
        // Clean up any dead weak references first
        self.cleanup_dead_references();

        // Register the new tracker
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

    /// Collect all currently active trackers for data aggregation
    fn collect_active_trackers(&mut self) -> Vec<Arc<MemoryTracker>> {
        // First, try to cache data from all trackers that are still upgradeable
        self.cache_all_available_data();

        // Collect all strong references that are still alive
        let mut active_trackers = Vec::new();
        let mut dead_thread_ids = Vec::new();

        for (thread_id, weak_tracker) in &self.trackers {
            if let Some(strong_tracker) = weak_tracker.upgrade() {
                active_trackers.push(strong_tracker);
                tracing::debug!("Successfully collected tracker for thread {:?}", thread_id);
            } else {
                // Tracker is dead but we might have cached data
                if self.cached_thread_data.contains_key(thread_id) {
                    tracing::debug!("Found cached data for dead thread {:?}", thread_id);
                } else {
                    tracing::debug!(
                        "Found dead tracker reference with no cached data for thread {:?}",
                        thread_id
                    );
                }
                dead_thread_ids.push(*thread_id);
            }
        }

        tracing::debug!(
            "Collected {} active trackers, {} dead with cached data, {} total cached entries",
            active_trackers.len(),
            dead_thread_ids.len(),
            self.cached_thread_data.len()
        );

        active_trackers
    }

    /// Cache data from all currently available trackers
    fn cache_all_available_data(&mut self) {
        for (thread_id, weak_tracker) in &self.trackers {
            if let Some(strong_tracker) = weak_tracker.upgrade() {
                if let Ok(stats) = strong_tracker.get_stats() {
                    // Only cache if we have meaningful data
                    if stats.total_allocations > 0 {
                        let allocations = stats.total_allocations;
                        let allocated = stats.total_allocated;

                        self.cached_thread_data.insert(
                            *thread_id,
                            CachedThreadData {
                                thread_id: *thread_id,
                                stats,
                                cached_at: std::time::SystemTime::now(),
                            },
                        );
                        tracing::debug!(
                            "Cached data for thread {:?}: {} allocations, {} bytes",
                            thread_id,
                            allocations,
                            allocated
                        );
                    }
                }
            }
        }
    }

    /// Remove dead weak references from the registry
    fn cleanup_dead_references(&mut self) {
        let initial_count = self.trackers.len();
        self.trackers
            .retain(|_thread_id, weak_tracker| weak_tracker.strong_count() > 0);

        let removed_count = initial_count - self.trackers.len();
        if removed_count > 0 {
            tracing::debug!("Cleaned up {} dead tracker references", removed_count);
        }

        self.active_threads = self.trackers.len();
    }

    /// Get registry statistics for monitoring
    fn get_stats(&self) -> ThreadRegistryStats {
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
}

/// Statistics about the thread registry for monitoring and debugging
#[derive(Debug, Clone)]
pub struct ThreadRegistryStats {
    /// Total number of threads that have ever been registered
    pub total_threads_registered: usize,
    /// Number of currently active threads
    pub active_threads: usize,
    /// Number of dead weak references (cleanup candidates)
    pub dead_references: usize,
}

/// Data aggregation result from all tracking modes
#[derive(Debug, Clone)]
pub struct AggregatedTrackingData {
    /// Number of trackers included in this aggregation
    pub tracker_count: usize,
    /// Total allocations across all trackers
    pub total_allocations: u64,
    /// Total bytes allocated across all trackers
    pub total_bytes_allocated: u64,
    /// Peak memory usage across all trackers
    pub peak_memory_usage: u64,
    /// Number of active threads that contributed data
    pub active_threads: usize,
    /// Combined statistics from all tracking modes
    pub combined_stats: Vec<CombinedTrackerStats>,
}

/// Combined statistics from a single tracker (can be track_var!, lockfree, or async)
#[derive(Debug, Clone)]
pub struct CombinedTrackerStats {
    /// Thread ID where this tracker operates
    pub thread_id: ThreadId,
    /// Type of tracking mode
    pub tracking_mode: String,
    /// Number of allocations in this tracker
    pub allocations: u64,
    /// Bytes allocated in this tracker
    pub bytes_allocated: u64,
    /// Peak memory for this tracker
    pub peak_memory: u64,
}

/// Get the global thread registry instance
fn get_registry() -> Arc<Mutex<ThreadRegistry>> {
    THREAD_REGISTRY
        .get_or_init(|| Arc::new(Mutex::new(ThreadRegistry::new())))
        .clone()
}

/// Register the current thread's tracker with the global registry.
///
/// This function should be called automatically when a thread-local tracker
/// is first accessed. It stores a weak reference to avoid preventing
/// tracker cleanup when threads exit.
pub fn register_current_thread_tracker(tracker: &Arc<MemoryTracker>) {
    let thread_id = std::thread::current().id();

    if let Ok(mut registry) = get_registry().lock() {
        registry.register_tracker(thread_id, tracker);
    } else {
        tracing::error!("Failed to acquire registry lock for thread registration");
    }
}

/// Collect and aggregate data from all tracking modes.
///
/// This is the main function for unified data collection that combines:
/// - track_var! data from all threads
/// - lockfree module data
/// - async_memory module data
pub fn collect_unified_tracking_data() -> Result<AggregatedTrackingData, String> {
    let mut combined_stats = Vec::new();
    let mut total_allocations = 0u64;
    let mut total_bytes_allocated = 0u64;
    let mut peak_memory_usage = 0u64;

    // Collect track_var! data from all thread-local trackers (active + cached)
    let active_trackers = collect_all_trackers();
    let cached_data = get_cached_thread_data();

    // Process active trackers
    for tracker in &active_trackers {
        if let Ok(stats) = tracker.get_stats() {
            let thread_stats = CombinedTrackerStats {
                thread_id: std::thread::current().id(), // Will be improved with actual thread IDs
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

    // Process cached data from completed threads
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

    // TODO: Integrate with lockfree module data
    // This will be implemented to collect data from lockfree aggregators

    // TODO: Integrate with async_memory module data
    // This will be implemented to collect data from async trackers

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

/// Collect all currently active thread-local memory trackers.
///
/// This function is used by the aggregation system to gather data from
/// all active threads when running in thread-local mode.
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

/// Get cached thread data from completed threads.
///
/// This function returns data that was cached from threads that have already
/// completed but whose tracking data is still valuable for aggregation.
pub fn get_cached_thread_data() -> Vec<CachedThreadData> {
    match get_registry().lock() {
        Ok(registry) => registry.cached_thread_data.values().cloned().collect(),
        Err(e) => {
            tracing::error!("Failed to acquire registry lock for cached data: {}", e);
            Vec::new()
        }
    }
}

/// Get statistics about the thread registry.
///
/// This function provides information about how many threads have been
/// registered and how many are currently active.
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

/// Clean up dead references from the registry.
///
/// This function can be called periodically to remove weak references
/// to trackers whose threads have exited.
pub fn cleanup_registry() {
    if let Ok(mut registry) = get_registry().lock() {
        registry.cleanup_dead_references();
    } else {
        tracing::error!("Failed to acquire registry lock for cleanup");
    }
}

/// Check if the registry has any active trackers.
pub fn has_active_trackers() -> bool {
    match get_registry().lock() {
        Ok(registry) => !registry.trackers.is_empty(),
        Err(_) => false,
    }
}

/// Enable precise tracking mode for maximum accuracy.
///
/// This configures all trackers to use thread-local mode and enables
/// detailed tracking for precise allocation tracking.
pub fn enable_precise_tracking() {
    crate::core::tracker::configure_tracking_strategy(true);
    tracing::info!("Enabled precise tracking mode with thread-local trackers");
}

/// Enable performance tracking mode for production use.
///
/// This configures trackers for minimal overhead while still providing
/// useful tracking data.
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
