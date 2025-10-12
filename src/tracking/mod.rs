//! 内存跟踪模块
//!
//! 提供高性能、可靠的内存分配跟踪功能，包括：
//! - 跟踪统计和质量监控
//! - 有界内存管理
//! - 智能大小估算
//!
//! # 示例
//!
//! ```rust
//! use memscope_rs::tracking::{TrackingStats, BoundedHistory};
//!
//! let stats = TrackingStats::new();
//! let history = BoundedHistory::new(100_000, Duration::from_secs(3600), 512);
//! ```

pub mod stats;

pub use stats::{DetailedStats, TrackingStats};
