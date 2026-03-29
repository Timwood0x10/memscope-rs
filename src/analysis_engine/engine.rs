//! Analysis Engine - Memory analysis logic
//!
//! This module provides the AnalysisEngine which coordinates multiple
//! analyzers to detect memory issues.

use crate::analysis_engine::analyzer::{AnalysisResult, Analyzer};
use crate::snapshot::{MemorySnapshot, SharedSnapshotEngine};

/// Analysis Engine - Coordinates memory analysis
///
/// The AnalysisEngine manages multiple analyzers and runs them on
/// memory snapshots to detect issues like leaks, fragmentation, and
/// safety violations.
///
/// Key properties:
/// - Pluggable: Supports adding custom analyzers
/// - Concurrent: Can run analyzers in parallel
/// - Comprehensive: Covers multiple analysis dimensions
pub struct AnalysisEngine {
    /// Reference to the snapshot engine
    snapshot_engine: SharedSnapshotEngine,
    /// Registered analyzers
    analyzers: Vec<Box<dyn Analyzer>>,
}

impl AnalysisEngine {
    /// Create a new AnalysisEngine
    pub fn new(snapshot_engine: SharedSnapshotEngine) -> Self {
        Self {
            snapshot_engine,
            analyzers: Vec::new(),
        }
    }

    /// Register an analyzer
    ///
    /// # Arguments
    /// * `analyzer` - The analyzer to register
    pub fn register_analyzer(&mut self, analyzer: Box<dyn Analyzer>) {
        self.analyzers.push(analyzer);
    }

    /// Run all registered analyzers on the current snapshot
    ///
    /// # Returns
    /// A vector of analysis results from all analyzers
    pub fn analyze(&self) -> Vec<AnalysisResult> {
        let snapshot = self.snapshot_engine.build_snapshot();
        self.analyze_snapshot(&snapshot)
    }

    /// Run all registered analyzers on a specific snapshot
    ///
    /// # Arguments
    /// * `snapshot` - The snapshot to analyze
    ///
    /// # Returns
    /// A vector of analysis results from all analyzers
    pub fn analyze_snapshot(&self, snapshot: &MemorySnapshot) -> Vec<AnalysisResult> {
        self.analyzers
            .iter()
            .map(|analyzer| analyzer.analyze(snapshot))
            .collect()
    }

    /// Get the number of registered analyzers
    pub fn analyzer_count(&self) -> usize {
        self.analyzers.len()
    }

    /// Check if any analyzers are registered
    pub fn has_analyzers(&self) -> bool {
        !self.analyzers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis_engine::analyzer::{AnalysisResult, Analyzer, Severity};
    use crate::event_store::EventStore;
    use crate::snapshot::SnapshotEngine;

    use std::sync::Arc;

    struct TestAnalyzer {
        name: String,
    }

    impl Analyzer for TestAnalyzer {
        fn name(&self) -> &str {
            &self.name
        }

        fn analyze(&self, snapshot: &MemorySnapshot) -> AnalysisResult {
            AnalysisResult {
                analyzer_name: self.name.clone(),
                issue_count: snapshot.active_count(),
                severity: Severity::Warning,
                description: format!("Analysis by {}", self.name),
                findings: vec![],
            }
        }
    }

    #[test]
    fn test_analysis_engine_creation() {
        let event_store = Arc::new(EventStore::new());
        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let engine = AnalysisEngine::new(snapshot_engine);

        assert_eq!(engine.analyzer_count(), 0);
        assert!(!engine.has_analyzers());
    }

    #[test]
    fn test_register_analyzer() {
        let event_store = Arc::new(EventStore::new());
        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let mut engine = AnalysisEngine::new(snapshot_engine);

        engine.register_analyzer(Box::new(TestAnalyzer {
            name: "test".to_string(),
        }));

        assert_eq!(engine.analyzer_count(), 1);
        assert!(engine.has_analyzers());
    }

    #[test]
    fn test_analyze() {
        let event_store = Arc::new(EventStore::new());
        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let mut engine = AnalysisEngine::new(snapshot_engine);

        engine.register_analyzer(Box::new(TestAnalyzer {
            name: "test".to_string(),
        }));

        let results = engine.analyze();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].analyzer_name, "test");
    }

    #[test]
    fn test_multiple_analyzers() {
        let event_store = Arc::new(EventStore::new());
        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let mut engine = AnalysisEngine::new(snapshot_engine);

        engine.register_analyzer(Box::new(TestAnalyzer {
            name: "analyzer1".to_string(),
        }));
        engine.register_analyzer(Box::new(TestAnalyzer {
            name: "analyzer2".to_string(),
        }));

        assert_eq!(engine.analyzer_count(), 2);

        let results = engine.analyze();
        assert_eq!(results.len(), 2);
    }
}
