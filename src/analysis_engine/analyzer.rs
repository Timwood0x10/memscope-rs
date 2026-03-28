//! Analyzer trait for pluggable analyzers
//!
//! This module defines the Analyzer trait that all analyzers must implement.

use crate::snapshot::MemorySnapshot;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Name of the analyzer
    pub analyzer_name: String,
    /// Number of issues found
    pub issue_count: usize,
    /// Severity level
    pub severity: Severity,
    /// Description of the result
    pub description: String,
    /// Detailed findings
    pub findings: Vec<Finding>,
}

/// Individual finding from an analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Type of issue
    pub issue_type: String,
    /// Description of the issue
    pub description: String,
    /// Memory pointer address (if applicable)
    pub ptr: Option<usize>,
    /// Size in bytes (if applicable)
    pub size: Option<usize>,
    /// Additional context
    pub context: String,
}

/// Severity level of an issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Informational only
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "Info"),
            Severity::Warning => write!(f, "Warning"),
            Severity::Error => write!(f, "Error"),
            Severity::Critical => write!(f, "Critical"),
        }
    }
}

/// Analyzer trait for pluggable analysis modules
///
/// All analyzers must implement this trait to be used with the AnalysisEngine.
pub trait Analyzer: Send + Sync {
    /// Get the name of this analyzer
    fn name(&self) -> &str;

    /// Analyze a memory snapshot and return results
    ///
    /// # Arguments
    /// * `snapshot` - The memory snapshot to analyze
    fn analyze(&self, snapshot: &MemorySnapshot) -> AnalysisResult;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyAnalyzer;

    impl Analyzer for DummyAnalyzer {
        fn name(&self) -> &str {
            "dummy"
        }

        fn analyze(&self, snapshot: &MemorySnapshot) -> AnalysisResult {
            AnalysisResult {
                analyzer_name: "dummy".to_string(),
                issue_count: snapshot.active_count(),
                severity: Severity::Info,
                description: "Dummy analysis".to_string(),
                findings: vec![],
            }
        }
    }

    #[test]
    fn test_analyzer_trait() {
        let analyzer = DummyAnalyzer;
        assert_eq!(analyzer.name(), "dummy");

        let snapshot = MemorySnapshot::new();
        let result = analyzer.analyze(&snapshot);
        assert_eq!(result.analyzer_name, "dummy");
    }
}