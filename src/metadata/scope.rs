//! Scope Tracker - Scope metadata management
//!
//! This module provides scope tracking and metadata management
//! for the MetadataEngine.
//!
//! # Concurrency Safety
//!
//! All scope-related data is protected by a single `Mutex<ScopeData>` to prevent
//! deadlock. The lock order is always: `scope_data` (single lock).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::debug;

/// Unique identifier for scopes
pub type ScopeId = u64;

/// Scope information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeInfo {
    /// Name of the scope
    pub name: String,
    /// Parent scope name (if any)
    pub parent: Option<String>,
    /// Depth of this scope in the hierarchy
    pub depth: usize,
    /// Variables in this scope
    pub variables: Vec<String>,
    /// Total memory usage in this scope
    pub total_memory: usize,
    /// Peak memory usage in this scope
    pub peak_memory: usize,
    /// Number of allocations in this scope
    pub allocation_count: usize,
    /// When this scope was created
    pub lifetime_start: Option<u64>,
    /// When this scope was destroyed
    pub lifetime_end: Option<u64>,
    /// Whether this scope is currently active
    pub is_active: bool,
}

/// Scope hierarchy information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeHierarchy {
    /// Root scopes
    pub root_scopes: Vec<String>,
    /// Scope tree relationships
    pub scope_tree: HashMap<String, Vec<String>>,
    /// Maximum depth observed
    pub max_depth: usize,
    /// Total number of scopes
    pub total_scopes: usize,
}

/// Internal data structure protected by a single lock.
/// This prevents deadlock by ensuring all related data is accessed atomically.
#[derive(Debug)]
struct ScopeData {
    /// Active scopes
    active_scopes: HashMap<ScopeId, ScopeInfo>,
    /// Scope stack per thread
    scope_stack: HashMap<String, Vec<ScopeId>>,
    /// Next available scope ID
    next_scope_id: u64,
    /// Scope hierarchy
    hierarchy: ScopeHierarchy,
}

impl ScopeData {
    fn new() -> Self {
        Self {
            active_scopes: HashMap::new(),
            scope_stack: HashMap::new(),
            next_scope_id: 1,
            hierarchy: ScopeHierarchy {
                root_scopes: Vec::new(),
                scope_tree: HashMap::new(),
                max_depth: 0,
                total_scopes: 0,
            },
        }
    }
}

/// Scope Tracker - manages scope hierarchy and metadata
#[derive(Debug)]
pub struct ScopeTracker {
    /// All scope data protected by a single lock to prevent deadlock
    data: Arc<Mutex<ScopeData>>,
}

impl ScopeTracker {
    /// Create a new ScopeTracker
    pub fn new() -> Self {
        debug!("Creating new ScopeTracker");
        Self {
            data: Arc::new(Mutex::new(ScopeData::new())),
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&self, name: String) -> ScopeId {
        let thread_id = format!("{:?}", std::thread::current().id());
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Acquire lock once and do all operations atomically
        let mut data = self.data.lock().expect("Failed to acquire scope_data lock");

        // Generate scope ID
        let scope_id = data.next_scope_id;
        data.next_scope_id += 1;

        // Determine parent scope and depth
        let (parent, depth) = data
            .scope_stack
            .get(&thread_id)
            .and_then(|thread_stack| thread_stack.last())
            .and_then(|&parent_id| data.active_scopes.get(&parent_id))
            .map(|active| (Some(active.name.clone()), active.depth + 1))
            .unwrap_or((None, 0));

        let scope_info = ScopeInfo {
            name: name.clone(),
            parent: parent.clone(),
            depth,
            variables: Vec::new(),
            total_memory: 0,
            peak_memory: 0,
            allocation_count: 0,
            lifetime_start: Some(timestamp),
            lifetime_end: None,
            is_active: true,
        };

        // Add to active scopes
        data.active_scopes.insert(scope_id, scope_info.clone());

        // Push to scope stack
        data.scope_stack
            .entry(thread_id)
            .or_default()
            .push(scope_id);

        // Update hierarchy
        if let Some(parent_name) = &parent {
            data.hierarchy
                .scope_tree
                .entry(parent_name.clone())
                .or_default()
                .push(name.clone());
        } else {
            if !data.hierarchy.root_scopes.contains(&name) {
                data.hierarchy.root_scopes.push(name.clone());
            }
        }
        data.hierarchy.total_scopes += 1;
        if depth > data.hierarchy.max_depth {
            data.hierarchy.max_depth = depth;
        }

        debug!("Entered scope '{}' with id {}", name, scope_id);
        scope_id
    }

    /// Exit the current scope
    pub fn exit_scope(&self) -> Option<ScopeId> {
        let thread_id = format!("{:?}", std::thread::current().id());
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let mut data = self.data.lock().expect("Failed to acquire scope_data lock");

        let scope_id = data
            .scope_stack
            .get_mut(&thread_id)
            .and_then(|thread_stack| thread_stack.pop());

        if let Some(scope_id) = scope_id {
            if let Some(scope) = data.active_scopes.get_mut(&scope_id) {
                scope.lifetime_end = Some(timestamp);
                scope.is_active = false;
            }
            debug!("Exited scope with id {}", scope_id);
            Some(scope_id)
        } else {
            debug!("No scope to exit for thread {}", thread_id);
            None
        }
    }

    /// Get scope information by ID
    pub fn get_scope(&self, scope_id: ScopeId) -> Option<ScopeInfo> {
        let data = self.data.lock().expect("Failed to acquire scope_data lock");
        data.active_scopes.get(&scope_id).cloned()
    }

    /// Get all scopes
    pub fn get_all_scopes(&self) -> Vec<(ScopeId, ScopeInfo)> {
        let data = self.data.lock().expect("Failed to acquire scope_data lock");
        data.active_scopes
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }

    /// Get scope hierarchy
    pub fn get_hierarchy(&self) -> ScopeHierarchy {
        let data = self.data.lock().expect("Failed to acquire scope_data lock");
        data.hierarchy.clone()
    }

    /// Clear all scopes
    pub fn clear(&self) {
        let mut data = self.data.lock().expect("Failed to acquire scope_data lock");
        data.active_scopes.clear();
        data.scope_stack.clear();
        data.next_scope_id = 1;
        data.hierarchy = ScopeHierarchy {
            root_scopes: Vec::new(),
            scope_tree: HashMap::new(),
            max_depth: 0,
            total_scopes: 0,
        };
        debug!("Cleared all scopes");
    }
}

impl Default for ScopeTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_tracker_creation() {
        let tracker = ScopeTracker::new();
        assert_eq!(tracker.get_all_scopes().len(), 0);
    }

    #[test]
    fn test_enter_exit_scope() {
        let tracker = ScopeTracker::new();
        let scope_id = tracker.enter_scope("test_scope".to_string());
        assert!(scope_id > 0);

        let scope = tracker.get_scope(scope_id);
        assert!(scope.is_some());
        assert!(scope.unwrap().is_active);

        let exited = tracker.exit_scope();
        assert!(exited.is_some());

        let scope = tracker.get_scope(scope_id);
        assert!(scope.is_some());
        assert!(!scope.unwrap().is_active);
    }

    #[test]
    fn test_nested_scopes() {
        let tracker = ScopeTracker::new();
        let outer_id = tracker.enter_scope("outer".to_string());
        let inner_id = tracker.enter_scope("inner".to_string());

        let outer = tracker.get_scope(outer_id).unwrap();
        let inner = tracker.get_scope(inner_id).unwrap();

        assert_eq!(inner.depth, outer.depth + 1);
        assert_eq!(inner.parent, Some("outer".to_string()));

        tracker.exit_scope();
        tracker.exit_scope();
    }

    #[test]
    fn test_hierarchy() {
        let tracker = ScopeTracker::new();
        tracker.enter_scope("root".to_string());
        tracker.enter_scope("child1".to_string());
        tracker.exit_scope();
        tracker.enter_scope("child2".to_string());
        tracker.exit_scope();
        tracker.exit_scope();

        let hierarchy = tracker.get_hierarchy();
        assert_eq!(hierarchy.root_scopes.len(), 1);
        assert_eq!(hierarchy.total_scopes, 3);
    }
}
