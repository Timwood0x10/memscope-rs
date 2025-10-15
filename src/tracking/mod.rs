//! Memory tracking module
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
