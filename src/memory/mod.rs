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
//! use std::time::Duration;
//!
//! let config = MemoryConfig::default();
//! let mut history = BoundedHistory::new(
//!     config.max_allocations,
//!     config.max_history_age,
//!     config.memory_limit_mb
//! );
//! ```

pub mod bounded_history;
pub mod config;

pub use bounded_history::{
    BoundedHistory, BoundedHistoryConfig, BoundedHistoryStats, TimestampedEntry,
};
pub use config::MemoryConfig;
