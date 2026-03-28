//! Scope Tracker - Scope metadata management
//!
//! This module provides scope tracking and metadata management
//! for the MetadataEngine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

/// Scope Tracker - manages scope hierarchy and metadata
#[derive(Debug)]
pub struct ScopeTracker {
    /// Active scopes
    active_scopes: Arc<Mutex<HashMap<ScopeId, ScopeInfo>>>,
    /// Scope stack per thread
    scope_stack: Arc<Mutex<HashMap<String, Vec<ScopeId>>>>,
    /// Next available scope ID
    next_scope_id: Arc<Mutex<u64>>,
    /// Scope hierarchy
    hierarchy: Arc<Mutex<ScopeHierarchy>>,
}

impl ScopeTracker {
    /// Create a new ScopeTracker
    pub fn new() -> Self {
        Self {
            active_scopes: Arc::new(Mutex::new(HashMap::new())),
            scope_stack: Arc::new(Mutex::new(HashMap::new())),
            next_scope_id: Arc::new(Mutex::new(1)),
            hierarchy: Arc::new(Mutex::new(ScopeHierarchy {
                root_scopes: Vec::new(),
                scope_tree: HashMap::new(),
                max_depth: 0,
                total_scopes: 0,
            })),
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&self, name: String) -> ScopeId {
        let scope_id = {
            let mut id = self.next_scope_id.lock().unwrap();
            let id_val = *id;
            *id += 1;
            id_val
        };

        let thread_id = format!("{:?}", std::thread::current().id());
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Determine parent scope and depth
        let (parent, depth) = {
            let stack = self.scope_stack.lock().unwrap();
            if let Some(thread_stack) = stack.get(&thread_id) {
                if let Some(&parent_id) = thread_stack.last() {
                    if let Some(active) = self.active_scopes.lock().unwrap().get(&parent_id) {
                        (Some(active.name.clone()), active.depth + 1)
                    } else {
                        (None, 0)
                    }
                } else {
                    (None, 0)
                }
            } else {
                (None, 0)
            }
        };

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
        self.active_scopes.lock().unwrap().insert(scope_id, scope_info.clone());

        // Push to scope stack
        self.scope_stack
            .lock()
            .unwrap()
            .entry(thread_id)
            .or_insert_with(Vec::new)
            .push(scope_id);

        // Update hierarchy
        {
            let mut hierarchy = self.hierarchy.lock().unwrap();
            if let Some(parent_name) = &parent {
                hierarchy
                    .scope_tree
                    .entry(parent_name.clone())
                    .or_insert_with(Vec::new)
                    .push(name.clone());
            } else {
                if !hierarchy.root_scopes.contains(&name) {
                    hierarchy.root_scopes.push(name);
                }
            }
            hierarchy.total_scopes += 1;
            if depth > hierarchy.max_depth {
                hierarchy.max_depth = depth;
            }
        }

        scope_id
    }

    /// Exit the current scope
    pub fn exit_scope(&self) -> Option<ScopeId> {
        let thread_id = format!("{:?}", std::thread::current().id());
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let scope_id = {
            let mut stack = self.scope_stack.lock().unwrap();
            if let Some(thread_stack) = stack.get_mut(&thread_id) {
                thread_stack.pop()
            } else {
                None
            }
        };

        if let Some(scope_id) = scope_id {
            if let Some(mut scope) = self.active_scopes.lock().unwrap().get_mut(&scope_id) {
                scope.lifetime_end = Some(timestamp);
                scope.is_active = false;
            }
            Some(scope_id)
        } else {
            None
        }
    }

    /// Get scope information by ID
    pub fn get_scope(&self, scope_id: ScopeId) -> Option<ScopeInfo> {
        self.active_scopes
            .lock()
            .unwrap()
            .get(&scope_id)
            .cloned()
    }

    /// Get all scopes
    pub fn get_all_scopes(&self) -> Vec<(ScopeId, ScopeInfo)> {
        self.active_scopes
            .lock()
            .unwrap()
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }

    /// Get scope hierarchy
    pub fn get_hierarchy(&self) -> ScopeHierarchy {
        self.hierarchy.lock().unwrap().clone()
    }

    /// Clear all scopes
    pub fn clear(&self) {
        self.active_scopes.lock().unwrap().clear();
        self.scope_stack.lock().unwrap().clear();
        *self.next_scope_id.lock().unwrap() = 1;
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