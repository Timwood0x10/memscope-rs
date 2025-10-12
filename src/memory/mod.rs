//! 内存管理模块
//!
//! 提供智能的内存管理功能，包括：
//! - 有界历史记录器
//! - 内存使用监控
//! - 配置化的内存策略
//!
//! # 示例
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

pub use bounded_history::{BoundedHistory, MemoryUsageStats};
pub use config::MemoryConfig;
