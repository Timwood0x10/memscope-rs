//! Lockfree adapter for bridging old lockfree system to new LockfreeTracker
//!
//! This adapter maintains backward compatibility with the old lockfree tracking API
//! while internally using the new unified tracking system.

use crate::manager::TrackingManager;
use crate::tracker::base::TrackBase;
use std::sync::Arc;

/// Lockfree adapter for high-performance event tracking
///
/// This adapter bridges the old lockfree API to the new unified tracking system,
/// providing compatibility for legacy lockfree tracking features.
pub struct LockfreeAdapter {
    /// New unified tracking manager
    manager: Arc<TrackingManager>,
}

impl LockfreeAdapter {
    /// Create a new lockfree adapter
    pub fn new() -> Self {
        Self {
            manager: Arc::new(TrackingManager::new_lockfree()),
        }
    }

    /// Get the underlying tracking manager
    pub fn manager(&self) -> &Arc<TrackingManager> {
        &self.manager
    }

    /// Track an allocation (lockfree mode)
    pub fn track_alloc(&self, ptr: usize, size: usize) {
        self.manager.track_alloc(ptr, size);
    }

    /// Track a deallocation (lockfree mode)
    pub fn track_dealloc(&self, ptr: usize) {
        self.manager.track_dealloc(ptr);
    }

    /// Get current snapshot
    pub fn snapshot(&self) -> crate::data::TrackingSnapshot {
        self.manager.snapshot()
    }
}

impl Default for LockfreeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfree_adapter_creation() {
        let adapter = LockfreeAdapter::new();
        assert!(adapter.manager().is_enabled());
    }

    #[test]
    fn test_lockfree_tracking() {
        let adapter = LockfreeAdapter::new();
        adapter.track_alloc(0x1000, 1024);
        adapter.track_dealloc(0x1000);

        let snapshot = adapter.snapshot();
        assert_eq!(snapshot.stats.allocation_count, 1);
        assert_eq!(snapshot.stats.deallocation_count, 1);
    }
}
