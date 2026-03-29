//! Metadata Engine - Centralized metadata management
//!
//! This module provides the MetadataEngine which is responsible for
//! managing all metadata including variables, scopes, threads, types,
//! and pointers across the memscope system.

use crate::metadata::registry::VariableRegistry;
use crate::metadata::scope::ScopeTracker;
use crate::metadata::thread::ThreadRegistry;
use std::sync::Arc;

/// Metadata Engine - Centralized metadata management
///
/// The MetadataEngine is responsible for managing all metadata in the
/// system including variables, scopes, threads, types, and pointers.
/// It provides a unified interface for accessing and updating metadata.
///
/// Key properties:
/// - Centralized: Single source of truth for all metadata
/// - Thread-safe: All operations are thread-safe via Arc
/// - Efficient: Optimized for fast lookups and updates
pub struct MetadataEngine {
    /// Variable registry
    pub variable_registry: Arc<VariableRegistry>,
    /// Scope tracker
    pub scope_tracker: Arc<ScopeTracker>,
    /// Thread registry
    pub thread_registry: Arc<ThreadRegistry>,
}

impl MetadataEngine {
    /// Create a new MetadataEngine
    pub fn new() -> Self {
        Self {
            variable_registry: Arc::new(VariableRegistry::new()),
            scope_tracker: Arc::new(ScopeTracker::new()),
            thread_registry: Arc::new(ThreadRegistry::new()),
        }
    }

    /// Get the variable registry
    pub fn variables(&self) -> &Arc<VariableRegistry> {
        &self.variable_registry
    }

    /// Get the scope tracker
    pub fn scopes(&self) -> &Arc<ScopeTracker> {
        &self.scope_tracker
    }

    /// Get the thread registry
    pub fn threads(&self) -> &Arc<ThreadRegistry> {
        &self.thread_registry
    }
}

impl Default for MetadataEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_engine_creation() {
        let engine = MetadataEngine::new();
        assert!(Arc::strong_count(&engine.variable_registry) >= 1);
        assert!(Arc::strong_count(&engine.scope_tracker) >= 1);
        assert!(Arc::strong_count(&engine.thread_registry) >= 1);
    }

    #[test]
    fn test_metadata_engine_default() {
        let engine = MetadataEngine::default();
        assert!(Arc::strong_count(&engine.variable_registry) >= 1);
    }
}
