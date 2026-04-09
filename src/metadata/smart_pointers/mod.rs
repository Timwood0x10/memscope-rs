//! Smart pointer tracking and analysis
//!
//! Provides enhanced tracking for Rust smart pointers including:
//! - Reference counting analysis for Rc/Arc
//! - Weak reference leak detection
//! - Box allocation patterns
//! - Smart pointer overhead calculation

pub mod analyzer;
pub mod tracker;

pub use analyzer::{AnalysisResult, LeakPattern, SmartPointerAnalyzer};
pub use tracker::{PointerInfo, PointerType, SmartPointerTracker};
