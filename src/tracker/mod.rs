//! Unified tracking layer
//!
//! This module provides the strategy-based tracking system that unifies
//! the 4 existing tracking systems (Core, Lockfree, Async, Unified) into
//! a single configurable architecture.

pub mod base;
pub mod strategies;

// Re-export commonly used types for convenience
pub use base::TrackBase;
pub use crate::data::TrackingStrategy;
pub use strategies::CoreTracker;
pub use strategies::LockfreeTracker;
pub use strategies::AsyncTracker;
pub use strategies::UnifiedTracker;