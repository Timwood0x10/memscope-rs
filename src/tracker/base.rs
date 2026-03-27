//! Unified tracking base trait
//!
//! This module defines the core abstractions for the strategy-based
//! tracking system. All tracking strategies must implement TrackBase.

use crate::data::{TrackingSnapshot, TrackingStrategy};
use crate::error::types::{ErrorKind, ErrorSeverity, MemScopeError};

/// Unified tracking base trait
///
/// All tracking strategies must implement this trait to provide
/// a consistent interface while maintaining different internal implementations.
pub trait TrackBase: Send + Sync {
    /// Get the strategy type
    fn strategy(&self) -> TrackingStrategy;

    /// Track an allocation
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the allocated memory
    /// * `size` - Size of the allocation in bytes
    fn track_alloc(&self, ptr: usize, size: usize);

    /// Track a deallocation
    ///
    /// # Arguments
    /// * `ptr` - Pointer to the deallocated memory
    fn track_dealloc(&self, ptr: usize);

    /// Get current tracking snapshot
    ///
    /// # Returns
    /// A TrackingSnapshot containing all current data
    fn snapshot(&self) -> TrackingSnapshot;

    /// Clear all tracked data
    fn clear(&self);

    /// Enable/disable tracking
    fn set_enabled(&self, enabled: bool);

    /// Check if tracking is enabled
    fn is_enabled(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracking_strategy_display() {
        assert_eq!(TrackingStrategy::Core.to_string(), "Core");
        assert_eq!(TrackingStrategy::Lockfree.to_string(), "Lockfree");
        assert_eq!(TrackingStrategy::Async.to_string(), "Async");
        assert_eq!(TrackingStrategy::Unified.to_string(), "Unified");
    }
}