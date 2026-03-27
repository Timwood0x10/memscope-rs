//! Strategy implementations
//!
//! This module contains the concrete implementations of the 4 tracking strategies:
//! - CoreTracker: Detailed single-threaded tracking
//! - LockfreeTracker: Event-based lockfree tracking
//! - AsyncTracker: Task-based async tracking
//! - UnifiedTracker: Hybrid multi-strategy tracking

pub mod core_tracker;
pub mod lockfree_tracker;
pub mod async_tracker;
pub mod unified_tracker;

// Re-export for convenience
pub use core_tracker::CoreTracker;
pub use lockfree_tracker::LockfreeTracker;
pub use async_tracker::AsyncTracker;
pub use unified_tracker::UnifiedTracker;