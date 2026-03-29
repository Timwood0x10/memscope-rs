//! Analysis Engine - Memory analysis logic
//!
//! This module provides the AnalysisEngine which is responsible for
//! analyzing memory data and detecting issues like leaks, fragmentation,
// and safety violations.

pub mod analyzer;
pub mod engine;

pub use analyzer::Analyzer;
pub use engine::AnalysisEngine;
