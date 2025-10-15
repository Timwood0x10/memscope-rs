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
//! use memscope_rs::tracking::{TrackingStats, BoundedHistory};
//!
//! let stats = TrackingStats::new();
//! let history = BoundedHistory::new(100_000, Duration::from_secs(3600), 512);
//! ```

pub mod stats;

pub use stats::{DetailedStats, TrackingStats};
