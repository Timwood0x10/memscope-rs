//! Intelligent size estimation module
//!
//! Provides accurate memory size estimation functionality, supporting:
//! - Precise size calculation for basic types
//! - Pattern matching estimation for complex types
//! - Dynamic learning and adaptive estimation
//! - Platform-specific size adaptation
//!
//! # Examples
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
