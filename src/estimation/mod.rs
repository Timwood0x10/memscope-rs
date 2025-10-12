//! 智能大小估算模块
//!
//! 提供准确的内存大小估算功能，支持：
//! - 基础类型的精确大小计算
//! - 复杂类型的模式匹配估算
//! - 动态学习和自适应估算
//! - 平台特定的大小适配
//!
//! # 示例
//!
//! ```rust
//! use memscope_rs::estimation::{SmartSizeEstimator, SizeEstimator};
//!
//! let estimator = SmartSizeEstimator::new();
//! let size = estimator.estimate_size("Vec<i32>").unwrap_or(24);
//! ```

pub mod size_estimator;
pub mod type_classifier;

pub use size_estimator::{LearnedSize, SizeEstimator, SmartSizeEstimator};
pub use type_classifier::{TypeCategory, TypeClassifier};
