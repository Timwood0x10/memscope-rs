//! Core tracking strategy implementation
//!
//! CoreTracker provides detailed single-threaded memory tracking with
//! comprehensive allocation metadata.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::data::{AllocationRecord, TrackingSnapshot, TrackingStats, TrackingStrategy};
use crate::tracker::base::TrackBase;

/// Core tracking state
#[derive(Debug)]
struct CoreState {
    enabled: bool,
    allocations: HashMap<usize, AllocationRecord>,
    total_allocated: usize,
    total_deallocated: usize,
    allocation_count: u64,
    deallocation_count: u64,
    peak_memory: usize,
}

impl CoreState {
    fn new() -> Self {
        CoreState {
            enabled: true,
            allocations: HashMap::new(),
            total_allocated: 0,
            total_deallocated: 0,
            allocation_count: 0,
            deallocation_count: 0,
            peak_memory: 0,
        }
    }

    fn update_peak_memory(&mut self) {
        let current_memory = self.total_allocated - self.total_deallocated;
        if current_memory > self.peak_memory {
            self.peak_memory = current_memory;
        }
    }
}

/// Core tracking strategy
///
/// Provides detailed single-threaded memory tracking with comprehensive
/// allocation metadata. Uses Arc<RwLock<CoreState>> for thread-safe access.
pub struct CoreTracker {
    state: Arc<RwLock<CoreState>>,
}

impl CoreTracker {
    /// Create a new CoreTracker
    pub fn new() -> Self {
        CoreTracker {
            state: Arc::new(RwLock::new(CoreState::new())),
        }
    }

    /// Get current timestamp in microseconds
    fn timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }

    /// Get current thread ID
    fn thread_id() -> u32 {
        use std::hash::{Hash, Hasher};
        let thread_id = std::thread::current().id();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        thread_id.hash(&mut hasher);
        (hasher.finish() & 0xFFFFFFFF) as u32
    }
}

impl Default for CoreTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackBase for CoreTracker {
    fn strategy(&self) -> TrackingStrategy {
        TrackingStrategy::Core
    }

    fn track_alloc(&self, ptr: usize, size: usize) {
        let mut state = self.state.write().unwrap();
        if !state.enabled {
            return;
        }

        let record = AllocationRecord {
            ptr,
            size,
            timestamp: Self::timestamp(),
            thread_id: Self::thread_id(),
            stack_id: None,  // Optional: capture stack trace
            var_name: None,  // Optional: capture variable name
            type_name: None, // Optional: capture type name
            is_active: true,
            dealloc_timestamp: None,
        };

        state.total_allocated += size;
        state.allocation_count += 1;
        state.update_peak_memory();
        state.allocations.insert(ptr, record);
    }

    fn track_dealloc(&self, ptr: usize) {
        let mut state = self.state.write().unwrap();
        if !state.enabled {
            return;
        }

        if let Some(mut record) = state.allocations.remove(&ptr) {
            state.total_deallocated += record.size;
            state.deallocation_count += 1;
            record.is_active = false;
            record.dealloc_timestamp = Some(Self::timestamp());
            // Could store deallocation records separately if needed
        }
    }

    fn snapshot(&self) -> TrackingSnapshot {
        let state = self.state.read().unwrap();
        let allocations: Vec<AllocationRecord> = state.allocations.values().cloned().collect();

        let current_memory = state.total_allocated - state.total_deallocated;
        let fragmentation = if state.total_allocated > 0 {
            ((state.total_allocated - current_memory) as f64 / state.total_allocated as f64) * 100.0
        } else {
            0.0
        };

        let stats = TrackingStats {
            total_allocations: state.allocation_count,
            total_deallocations: state.deallocation_count,
            total_allocated: state.total_allocated as u64,
            total_deallocated: state.total_deallocated as u64,
            peak_memory: state.peak_memory as u64,
            active_allocations: state.allocations.len() as u64,
            active_memory: current_memory as u64,
            leaked_allocations: state.allocations.iter().filter(|a| a.1.is_active).count() as u64,
            leaked_memory: state
                .allocations
                .iter()
                .filter(|a| a.1.is_active)
                .map(|a| a.1.size as u64)
                .sum(),
            fragmentation_ratio: fragmentation,
            allocation_count: state.allocation_count,
            deallocation_count: state.deallocation_count,
            average_allocation_size: if state.allocation_count > 0 {
                state.total_allocated / state.allocation_count as usize
            } else {
                0
            },
            current_allocated: current_memory,
            fragmentation,
        };

        TrackingSnapshot {
            strategy: TrackingStrategy::Core,
            allocations,
            events: vec![], // Core strategy doesn't track events
            tasks: vec![],  // Core strategy doesn't track tasks
            stats,
            timestamp: Self::timestamp(),
        }
    }

    fn clear(&self) {
        let mut state = self.state.write().unwrap();
        state.allocations.clear();
        state.total_allocated = 0;
        state.total_deallocated = 0;
        state.allocation_count = 0;
        state.deallocation_count = 0;
        state.peak_memory = 0;
    }

    fn set_enabled(&self, enabled: bool) {
        let mut state = self.state.write().unwrap();
        state.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        let state = self.state.read().unwrap();
        state.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_tracker_creation() {
        let tracker = CoreTracker::new();
        assert_eq!(tracker.strategy(), TrackingStrategy::Core);
        assert!(tracker.is_enabled());
    }

    #[test]
    fn test_core_tracker_alloc_dealloc() {
        let tracker = CoreTracker::new();
        tracker.track_alloc(0x1000, 1024);
        tracker.track_alloc(0x2000, 2048);
        tracker.track_dealloc(0x1000);

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.allocations.len(), 1);
        assert_eq!(snapshot.stats.allocation_count, 2);
        assert_eq!(snapshot.stats.deallocation_count, 1);
        assert_eq!(snapshot.stats.current_allocated, 2048);
    }

    #[test]
    fn test_core_tracker_clear() {
        let tracker = CoreTracker::new();
        tracker.track_alloc(0x1000, 1024);
        tracker.clear();

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.allocations.len(), 0);
        assert_eq!(snapshot.stats.allocation_count, 0);
    }

    #[test]
    fn test_core_tracker_enable_disable() {
        let tracker = CoreTracker::new();
        tracker.set_enabled(false);
        tracker.track_alloc(0x1000, 1024);

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.allocations.len(), 0);

        tracker.set_enabled(true);
        tracker.track_alloc(0x2000, 2048);

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.allocations.len(), 1);
    }

    #[test]
    fn test_core_tracker_peak_memory() {
        let tracker = CoreTracker::new();
        tracker.track_alloc(0x1000, 1024);
        tracker.track_alloc(0x2000, 2048);
        tracker.track_dealloc(0x1000);

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.stats.peak_memory, 3072);
    }
}
