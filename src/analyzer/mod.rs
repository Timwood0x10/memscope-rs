//! Analyzer Module - Unified analysis entry point.
//!
//! This module provides the Analyzer which serves as the single entry
//! point for all memory analysis operations. It integrates all analysis
//! modules through a structured interface.

mod classify;
mod core;
mod detect;
mod export;
mod graph;
mod metrics;
mod report;
mod safety;
mod timeline;

pub use classify::{
    ClassificationAnalysis, ClassificationSummary, TypeCategory, TypeClassification,
};
pub use core::Analyzer;
pub use detect::DetectionAnalysis;
pub use export::ExportEngine;
pub use graph::{GraphAnalysis, OwnershipStats, RelationshipEdge, RelationshipStats};
pub use metrics::MetricsAnalysis;
pub use report::{AnalysisReport, CycleInfo, CycleReport, LeakInfo, LeakReport, MetricsReport};
pub use safety::{SafetyAnalysis, SafetySummary};
pub use timeline::TimelineAnalysis;

// Re-export Relation type for convenience
pub use crate::analysis::relation_inference::Relation;
