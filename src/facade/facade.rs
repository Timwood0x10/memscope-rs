//! MemScope - Unified facade for all engines
//!
//! This module provides the MemScope facade which unifies all engines
//! into a simple, easy-to-use interface.

use crate::analysis_engine::{AnalysisEngine, Analyzer};
use crate::capture::{CaptureBackendType, CaptureEngine};
use crate::event_store::EventStore;
use crate::metadata::MetadataEngine;
use crate::query::QueryEngine;
use crate::render_engine::RenderEngine;
use crate::snapshot::SnapshotEngine;
use crate::timeline::TimelineEngine;
use std::sync::{Arc, Mutex};

/// MemScope - Unified facade for all engines
///
/// MemScope provides a simple, unified interface for memory tracking,
/// analysis, and visualization. It integrates all 9 engines into a
/// single, easy-to-use API.
///
/// Key properties:
/// - Simple: One-stop interface for all functionality
/// - Powerful: Access to all engines when needed
/// - Type-safe: Strong typing throughout
/// - Thread-safe: All operations are thread-safe
pub struct MemScope {
    /// Event Store - Centralized event storage
    pub event_store: Arc<EventStore>,
    /// Capture Engine - Event capture backend
    pub capture: Arc<CaptureEngine>,
    /// Metadata Engine - Centralized metadata management
    pub metadata: Arc<MetadataEngine>,
    /// Snapshot Engine - Snapshot construction and aggregation
    pub snapshot: Arc<SnapshotEngine>,
    /// Query Engine - Unified query interface
    pub query: Arc<QueryEngine>,
    /// Analysis Engine - Memory analysis logic (wrapped in Mutex for interior mutability)
    pub analysis: Arc<Mutex<AnalysisEngine>>,
    /// Timeline Engine - Time-based memory analysis
    pub timeline: Arc<TimelineEngine>,
    /// Render Engine - Output rendering
    pub render: Arc<RenderEngine>,
}

impl MemScope {
    /// Create a new MemScope instance
    ///
    /// This creates all engines with default settings and connects them
    /// together in the correct configuration.
    pub fn new() -> Self {
        // Create EventStore (the foundation)
        let event_store = Arc::new(EventStore::new());

        // Create Capture Engine
        let capture = Arc::new(CaptureEngine::new(
            CaptureBackendType::Unified,
            event_store.clone(),
        ));

        // Create Metadata Engine
        let metadata = Arc::new(MetadataEngine::new());

        // Create Snapshot Engine
        let snapshot = Arc::new(SnapshotEngine::new(event_store.clone()));

        // Create Query Engine
        let query = Arc::new(QueryEngine::new(snapshot.clone()));

        // Create Analysis Engine
        let analysis = Arc::new(Mutex::new(AnalysisEngine::new(snapshot.clone())));

        // Create Timeline Engine
        let timeline = Arc::new(TimelineEngine::new(event_store.clone()));

        // Create Render Engine
        let render = Arc::new(RenderEngine::new(snapshot.clone()));

        Self {
            event_store,
            capture,
            metadata,
            snapshot,
            query,
            analysis,
            timeline,
            render,
        }
    }

    /// Create a new MemScope instance with a specific capture backend
    ///
    /// # Arguments
    /// * `backend_type` - The type of capture backend to use
    pub fn with_backend(backend_type: CaptureBackendType) -> Self {
        // Create EventStore (the foundation)
        let event_store = Arc::new(EventStore::new());

        // Create Capture Engine with specified backend
        let capture = Arc::new(CaptureEngine::new(backend_type, event_store.clone()));

        // Create Metadata Engine
        let metadata = Arc::new(MetadataEngine::new());

        // Create Snapshot Engine
        let snapshot = Arc::new(SnapshotEngine::new(event_store.clone()));

        // Create Query Engine
        let query = Arc::new(QueryEngine::new(snapshot.clone()));

        // Create Analysis Engine
        let analysis = Arc::new(Mutex::new(AnalysisEngine::new(snapshot.clone())));

        // Create Timeline Engine
        let timeline = Arc::new(TimelineEngine::new(event_store.clone()));

        // Create Render Engine
        let render = Arc::new(RenderEngine::new(snapshot.clone()));

        Self {
            event_store,
            capture,
            metadata,
            snapshot,
            query,
            analysis,
            timeline,
            render,
        }
    }

    /// Register an analyzer with the analysis engine
    ///
    /// # Arguments
    /// * `analyzer` - The analyzer to register
    pub fn register_analyzer(&self, analyzer: Box<dyn Analyzer>) {
        if let Ok(mut analysis) = self.analysis.lock() {
            analysis.register_analyzer(analyzer);
            tracing::info!("Analyzer registered successfully");
        } else {
            tracing::error!("Failed to acquire analysis engine lock for registration");
        }
    }

    /// Get a summary of current memory usage
    pub fn summary(&self) -> crate::query::QueryResult {
        self.query.summary()
    }

    /// Get top allocations by size
    ///
    /// # Arguments
    /// * `limit` - Maximum number of allocations to return
    pub fn top_allocations(&self, limit: usize) -> crate::query::QueryResult {
        self.query.top_allocations(limit)
    }

    /// Render current snapshot to JSON
    ///
    /// # Arguments
    /// * `verbose` - Whether to include verbose output
    pub fn render_json(&self, verbose: bool) -> Result<crate::render_engine::RenderResult, String> {
        let snapshot = self.snapshot.build_snapshot();
        self.render.render_json(&snapshot, verbose)
    }

    /// Clear all events and reset state
    pub fn clear(&self) {
        self.event_store.clear();
    }

    /// Get the total number of events
    pub fn event_count(&self) -> usize {
        self.event_store.len()
    }
}

impl Default for MemScope {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memscope_creation() {
        let memscope = MemScope::new();
        assert_eq!(memscope.event_count(), 0);
    }

    #[test]
    fn test_memscope_default() {
        let memscope = MemScope::default();
        assert_eq!(memscope.event_count(), 0);
    }

    #[test]
    fn test_memscope_with_backend() {
        let memscope = MemScope::with_backend(CaptureBackendType::Core);
        assert_eq!(memscope.event_count(), 0);
    }

    #[test]
    fn test_summary() {
        let memscope = MemScope::new();
        let result = memscope.summary();
        match result {
            crate::query::QueryResult::Summary(_) => (),
            _ => panic!("Expected summary result"),
        }
    }

    #[test]
    fn test_top_allocations() {
        let memscope = MemScope::new();
        let result = memscope.top_allocations(10);
        match result {
            crate::query::QueryResult::Allocations(_) => (),
            _ => panic!("Expected allocations result"),
        }
    }

    #[test]
    fn test_render_json() {
        let memscope = MemScope::new();
        let result = memscope.render_json(false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear() {
        let memscope = MemScope::new();
        // Simulate some events
        memscope.capture.capture_alloc(0x1000, 1024, 1);
        assert_eq!(memscope.event_count(), 1);

        memscope.clear();
        assert_eq!(memscope.event_count(), 0);
    }
}
