//! Memory Management Module
//!
//! Provides intelligent memory management features, including:
//! - Bounded history recorder
//! - Memory usage monitoring
//! - Configurable memory policies
//!
//! # Examples
//!
//! ```rust
//! use memscope_rs::memory::{BoundedHistory, MemoryConfig};
//!
//! let config = MemoryConfig::default();
//! let mut history: BoundedHistory<String> = BoundedHistory::new();
//! ```

pub mod bounded_history;
pub mod config;

pub use bounded_history::{
    BoundedHistory, BoundedHistoryConfig, BoundedHistoryStats, TimestampedEntry,
};
pub use config::MemoryConfig;
