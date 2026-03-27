//! Memory tracking module
//!
//! # Deprecated
//!
//! This module is deprecated. Please use the new unified tracking system
//! located in `src/new/tracker/mod.rs` with `UnifiedTracker`.
//!
//! The new unified tracking system provides:
//! - Better performance through configurable strategies
//! - Cleaner API with reduced complexity
//! - Unified type system across all tracking modes
//!
//! Migration Guide:
//! - Replace `crate::tracking::TrackingStats` with `crate::new::tracker::UnifiedTracker::stats()`
//! - Use `UnifiedTracker` for tracking operations
//! - All functionality is preserved for backward compatibility
//!
//! Provides high-performance, reliable memory allocation tracking features, including:
//! - Tracking statistics and quality monitoring
//! - Bounded memory management
//! - Intelligent size estimation
//!
//! # Examples
//!
//! ```rust
//! use memscope_rs::tracking::TrackingStats;
//! use memscope_rs::memory::BoundedHistory;
//!
//! let stats = TrackingStats::new();
//! let history: BoundedHistory<String> = BoundedHistory::new();
//! ```

pub mod stats;

pub use stats::{DetailedStats, TrackingStats};
