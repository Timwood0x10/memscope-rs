//! Scope tracking functionality for memory analysis

use crate::core::types::*;
use crate::core::{MemScopeError, MemScopeResult, SystemErrorType};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Global scope tracker instance
static GLOBAL_SCOPE_TRACKER: OnceLock<Arc<ScopeTracker>> = OnceLock::new();

/// Get the global scope tracker instance
pub fn get_global_scope_tracker() -> Arc<ScopeTracker> {
    GLOBAL_SCOPE_TRACKER
        .get_or_init(|| Arc::new(ScopeTracker::new()))
        .clone()
}

/// Unique identifier for scopes
pub type ScopeId = u64;

/// Core scope tracking functionality
pub struct ScopeTracker {
    /// Active scopes
    pub active_scopes: RwLock<HashMap<ScopeId, ScopeInfo>>,
    /// Completed scopes for analysis
    pub completed_scopes: Mutex<Vec<ScopeInfo>>,
    /// Scope hierarchy relationships
    pub scope_hierarchy: Mutex<ScopeHierarchy>,
    /// Next available scope ID using atomic counter
    next_scope_id: AtomicU64,
    /// Current scope stack per thread
    pub scope_stack: RwLock<HashMap<String, Vec<ScopeId>>>,
}

impl ScopeTracker {
    /// Create a new scope tracker
    pub fn new() -> Self {
        Self {
            active_scopes: RwLock::new(HashMap::new()),
            completed_scopes: Mutex::new(Vec::new()),
            scope_hierarchy: Mutex::new(ScopeHierarchy {
                root_scopes: Vec::new(),
                scope_tree: HashMap::new(),
                max_depth: 0,
                total_scopes: 0,
                relationships: HashMap::new(),
                depth_map: HashMap::new(),
            }),
            next_scope_id: AtomicU64::new(1),
            scope_stack: RwLock::new(HashMap::new()),
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&self, name: String) -> MemScopeResult<ScopeId> {
        let scope_id = self.allocate_scope_id();
        let thread_id = format!("{:?}", std::thread::current().id());
        let timestamp = current_timestamp();

        let (parent_scope, depth) = {
            let stack = self.scope_stack.read().map_err(|e| {
                MemScopeError::system(
                    SystemErrorType::Locking,
                    format!("Failed to acquire scope_stack read lock: {}", e),
                )
            })?;
            if let Some(thread_stack) = stack.get(&thread_id) {
                if let Some(&parent_id) = thread_stack.last() {
                    let active = self.active_scopes.read().map_err(|e| {
                        MemScopeError::system(
                            SystemErrorType::Locking,
                            format!("Failed to acquire active_scopes read lock: {}", e),
                        )
                    })?;
                    if let Some(parent) = active.get(&parent_id) {
                        (Some(parent.name.clone()), parent.depth + 1)
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
            parent: parent_scope.clone(),
            children: Vec::new(),
            depth,
            variables: Vec::new(),
            total_memory: 0,
            peak_memory: 0,
            allocation_count: 0,
            lifetime_start: Some(timestamp as u64),
            lifetime_end: None,
            is_active: true,
            start_time: timestamp as u64,
            end_time: None,
            memory_usage: 0,
            child_scopes: Vec::new(),
            parent_scope: parent_scope.clone(),
        };

        self.active_scopes
            .write()
            .map_err(|e| {
                MemScopeError::system(
                    SystemErrorType::Locking,
                    format!("Failed to acquire active_scopes write lock: {}", e),
                )
            })?
            .insert(scope_id, scope_info);

        self.scope_stack
            .write()
            .map_err(|e| {
                MemScopeError::system(
                    SystemErrorType::Locking,
                    format!("Failed to acquire scope_stack write lock: {}", e),
                )
            })?
            .entry(thread_id.clone())
            .or_default()
            .push(scope_id);

        if let Ok(mut hierarchy) = self.scope_hierarchy.lock() {
            hierarchy.depth_map.insert(name.clone(), depth);

            if let Some(parent) = parent_scope.clone() {
                hierarchy
                    .relationships
                    .entry(parent)
                    .or_default()
                    .push(name.clone());
            } else {
                hierarchy.root_scopes.push(name);
            }
        }

        Ok(scope_id)
    }

    /// Exit a scope
    pub fn exit_scope(&self, scope_id: ScopeId) -> MemScopeResult<()> {
        let thread_id = format!("{:?}", std::thread::current().id());
        let timestamp = current_timestamp();

        let mut scope_info = self
            .active_scopes
            .write()
            .map_err(|e| {
                MemScopeError::system(
                    SystemErrorType::Locking,
                    format!("Failed to acquire active_scopes write lock: {}", e),
                )
            })?
            .remove(&scope_id)
            .ok_or_else(|| MemScopeError::internal(format!("Invalid scope ID: {scope_id}")))?;

        scope_info.end_time = Some(timestamp as u64);
        scope_info.lifetime_end = Some(timestamp as u64);

        if let Ok(mut stack) = self.scope_stack.write() {
            if let Some(thread_stack) = stack.get_mut(&thread_id) {
                if let Some(pos) = thread_stack.iter().position(|&id| id == scope_id) {
                    thread_stack.remove(pos);
                }
            }
        }

        if let Ok(mut completed_scopes) = self.completed_scopes.lock() {
            completed_scopes.push(scope_info);
        }

        Ok(())
    }

    pub fn associate_variable(
        &self,
        variable_name: String,
        memory_size: usize,
    ) -> MemScopeResult<()> {
        let thread_id = format!("{:?}", std::thread::current().id());

        let current_scope_id = self
            .scope_stack
            .read()
            .map_err(|e| {
                MemScopeError::system(
                    SystemErrorType::Locking,
                    format!("Failed to acquire scope_stack read lock: {}", e),
                )
            })?
            .get(&thread_id)
            .and_then(|stack| stack.last().copied());

        if let Some(scope_id) = current_scope_id {
            if let Ok(mut active) = self.active_scopes.write() {
                if let Some(scope) = active.get_mut(&scope_id) {
                    scope.variables.push(variable_name);
                    scope.memory_usage += memory_size;
                    scope.peak_memory = scope.peak_memory.max(scope.memory_usage);
                    scope.allocation_count += 1;
                }
            }
        }

        Ok(())
    }

    pub fn get_scope_analysis(&self) -> MemScopeResult<ScopeAnalysis> {
        let mut all_scopes: Vec<ScopeInfo> = self
            .active_scopes
            .read()
            .map_err(|e| {
                MemScopeError::system(
                    SystemErrorType::Locking,
                    format!("Failed to acquire active_scopes read lock: {}", e),
                )
            })?
            .values()
            .cloned()
            .collect();

        if let Ok(completed_scopes) = self.completed_scopes.lock() {
            all_scopes.extend(completed_scopes.iter().cloned());
        }

        let hierarchy = if let Ok(hierarchy) = self.scope_hierarchy.lock() {
            hierarchy.clone()
        } else {
            ScopeHierarchy {
                root_scopes: Vec::new(),
                scope_tree: HashMap::new(),
                max_depth: 0,
                total_scopes: 0,
                relationships: HashMap::new(),
                depth_map: HashMap::new(),
            }
        };

        Ok(ScopeAnalysis {
            total_scopes: all_scopes.len(),
            active_scopes: all_scopes.iter().filter(|s| s.is_active).count(),
            max_depth: hierarchy.max_depth,
            average_lifetime: 1000.0,
            memory_efficiency: 0.8,
            scopes: all_scopes,
            scope_hierarchy: hierarchy,
            cross_scope_references: Vec::new(),
        })
    }

    pub fn get_scope_lifecycle_metrics(&self) -> MemScopeResult<Vec<ScopeLifecycleMetrics>> {
        let metrics = if let Ok(completed_scopes) = self.completed_scopes.lock() {
            completed_scopes
                .iter()
                .map(|scope| {
                    let lifetime =
                        scope.end_time.unwrap_or(current_timestamp() as u64) - scope.start_time;
                    let efficiency_score = if scope.peak_memory > 0 {
                        scope.memory_usage as f64 / scope.peak_memory as f64
                    } else {
                        1.0
                    };

                    ScopeLifecycleMetrics {
                        scope_name: scope.name.clone(),
                        variable_count: scope.variables.len(),
                        average_lifetime_ms: lifetime as f64,
                        total_memory_usage: scope.memory_usage,
                        peak_memory_usage: scope.peak_memory,
                        allocation_frequency: 1.0,
                        deallocation_efficiency: efficiency_score,
                        completed_allocations: scope.allocation_count,
                        memory_growth_events: 0,
                        peak_concurrent_variables: scope.variables.len(),
                        memory_efficiency_ratio: if scope.peak_memory > 0 {
                            scope.memory_usage as f64 / scope.peak_memory as f64
                        } else {
                            1.0
                        },
                        ownership_transfer_events: 0,
                        fragmentation_score: 0.0,
                        instant_allocations: 0,
                        short_term_allocations: 0,
                        medium_term_allocations: 0,
                        long_term_allocations: 0,
                        suspected_leaks: 0,
                        risk_distribution: crate::core::types::RiskDistribution::default(),
                        scope_metrics: Vec::new(),
                        type_lifecycle_patterns: Vec::new(),
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(metrics)
    }

    pub fn get_all_scopes(&self) -> Vec<ScopeInfo> {
        let mut all_scopes: Vec<ScopeInfo> = match self.active_scopes.read() {
            Ok(active) => active.values().cloned().collect(),
            Err(_) => Vec::new(),
        };

        if let Ok(completed_scopes) = self.completed_scopes.lock() {
            all_scopes.extend(completed_scopes.iter().cloned());
        }

        all_scopes
    }

    /// Allocate a new unique scope ID using atomic operations
    fn allocate_scope_id(&self) -> ScopeId {
        self.next_scope_id.fetch_add(1, Ordering::Relaxed)
    }
}

impl Default for ScopeTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII scope guard for automatic scope management
pub struct ScopeGuard {
    scope_id: ScopeId,
    tracker: Arc<ScopeTracker>,
}

impl ScopeGuard {
    pub fn enter(name: &str) -> MemScopeResult<Self> {
        let tracker = get_global_scope_tracker();
        let scope_id = tracker.enter_scope(name.to_string())?;

        Ok(Self { scope_id, tracker })
    }

    pub fn scope_id(&self) -> ScopeId {
        self.scope_id
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        let _ = self.tracker.exit_scope(self.scope_id);
    }
}

fn current_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Macro for tracking scopes with automatic cleanup
#[macro_export]
macro_rules! track_scope {
    ($scope_name:expr) => {
        let _scope_guard = $crate::scope_tracker::ScopeGuard::enter($scope_name)?;
    };
    ($scope_name:expr, $block:block) => {{
        let _scope_guard = $crate::scope_tracker::ScopeGuard::enter($scope_name)?;
        $block
    }};
}

/// Enhanced track_var macro that also associates with current scope
#[macro_export]
macro_rules! track_var_with_scope {
    ($var:ident) => {{
        // Track the variable normally
        let result = $crate::_track_var_impl(&$var, stringify!($var));

        // Also associate with current scope
        if result.is_ok() {
            if let $crate::TrackKind::HeapOwner { ptr: _, size } =
                $crate::Trackable::track_kind(&$var)
            {
                let scope_tracker = $crate::scope_tracker::get_global_scope_tracker();
                let _ = scope_tracker.associate_variable(stringify!($var).to_string(), size);
            }
        }

        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify ScopeTracker creation with default values
    /// Invariants: New tracker should have empty collections and next_scope_id = 1
    #[test]
    fn test_scope_tracker_creation() {
        let tracker = ScopeTracker::new();
        assert_eq!(
            tracker.next_scope_id.load(Ordering::SeqCst),
            1,
            "Next scope ID should start at 1"
        );
        assert!(
            tracker.active_scopes.read().unwrap().is_empty(),
            "Active scopes should be empty"
        );
        assert!(
            tracker.completed_scopes.lock().unwrap().is_empty(),
            "Completed scopes should be empty"
        );
    }

    /// Objective: Verify Default trait implementation
    /// Invariants: Default should create same as new()
    #[test]
    fn test_scope_tracker_default() {
        let tracker = ScopeTracker::default();
        assert_eq!(
            tracker.next_scope_id.load(Ordering::SeqCst),
            1,
            "Default should start with ID 1"
        );
    }

    /// Objective: Verify enter and exit scope functionality
    /// Invariants: Scope should be active after enter, moved to completed after exit
    #[test]
    fn test_enter_and_exit_scope() {
        let tracker = ScopeTracker::new();
        let scope_id = tracker
            .enter_scope("test_scope".to_string())
            .expect("Should enter scope");

        assert!(
            tracker
                .active_scopes
                .read()
                .unwrap()
                .get(&scope_id)
                .is_some(),
            "Scope should be active"
        );

        tracker.exit_scope(scope_id).expect("Should exit scope");

        assert!(
            tracker
                .active_scopes
                .read()
                .unwrap()
                .get(&scope_id)
                .is_none(),
            "Scope should no longer be active"
        );
        assert_eq!(
            tracker.completed_scopes.lock().unwrap().len(),
            1,
            "Should have one completed scope"
        );
    }

    /// Objective: Verify scope ID allocation is sequential
    /// Invariants: Each enter_scope should return incrementing IDs
    #[test]
    fn test_sequential_scope_ids() {
        let tracker = ScopeTracker::new();

        let id1 = tracker.enter_scope("scope1".to_string()).unwrap();
        let id2 = tracker.enter_scope("scope2".to_string()).unwrap();
        let id3 = tracker.enter_scope("scope3".to_string()).unwrap();

        assert!(id1 < id2, "IDs should be sequential");
        assert!(id2 < id3, "IDs should be sequential");
    }

    /// Objective: Verify nested scope tracking
    /// Invariants: Child scope should have correct parent and depth
    #[test]
    fn test_nested_scopes() {
        let tracker = ScopeTracker::new();

        let parent_id = tracker.enter_scope("parent".to_string()).unwrap();
        let child_id = tracker.enter_scope("child".to_string()).unwrap();

        let active = tracker.active_scopes.read().unwrap();
        let parent_scope = active.get(&parent_id).expect("Parent should exist");
        let child_scope = active.get(&child_id).expect("Child should exist");

        assert_eq!(parent_scope.depth, 0, "Parent should have depth 0");
        assert_eq!(child_scope.depth, 1, "Child should have depth 1");
        assert_eq!(
            child_scope.parent_scope,
            Some("parent".to_string()),
            "Child should have parent"
        );
    }

    /// Objective: Verify associate_variable functionality
    /// Invariants: Variable should be associated with current scope
    #[test]
    fn test_associate_variable() {
        let tracker = ScopeTracker::new();
        let _scope_id = tracker.enter_scope("test".to_string()).unwrap();

        tracker
            .associate_variable("my_var".to_string(), 1024)
            .expect("Should associate variable");

        let active = tracker.active_scopes.read().unwrap();
        let scope = active.values().next().expect("Should have scope");

        assert!(
            scope.variables.contains(&"my_var".to_string()),
            "Variable should be in scope"
        );
        assert_eq!(scope.memory_usage, 1024, "Memory usage should be updated");
        assert_eq!(scope.allocation_count, 1, "Allocation count should be 1");
    }

    /// Objective: Verify peak memory tracking
    /// Invariants: Peak memory should track maximum usage
    #[test]
    fn test_peak_memory_tracking() {
        let tracker = ScopeTracker::new();
        let _scope_id = tracker.enter_scope("test".to_string()).unwrap();

        tracker.associate_variable("var1".to_string(), 100).unwrap();
        tracker.associate_variable("var2".to_string(), 200).unwrap();

        let active = tracker.active_scopes.read().unwrap();
        let scope = active.values().next().expect("Should have scope");

        assert_eq!(scope.memory_usage, 300, "Total memory should be 300");
        assert_eq!(scope.peak_memory, 300, "Peak memory should be 300");
    }

    /// Objective: Verify get_scope_analysis functionality
    /// Invariants: Should return analysis with all scopes
    #[test]
    fn test_get_scope_analysis() {
        let tracker = ScopeTracker::new();

        let id1 = tracker.enter_scope("scope1".to_string()).unwrap();
        let _id2 = tracker.enter_scope("scope2".to_string()).unwrap();
        tracker.exit_scope(id1).unwrap();

        let analysis = tracker.get_scope_analysis().expect("Should get analysis");

        assert_eq!(analysis.total_scopes, 2, "Should have 2 scopes total");
        assert!(
            analysis.active_scopes >= 1,
            "Should have at least 1 active scope"
        );
    }

    /// Objective: Verify get_all_scopes returns all scopes
    /// Invariants: Should include both active and completed scopes
    #[test]
    fn test_get_all_scopes() {
        let tracker = ScopeTracker::new();

        let id1 = tracker.enter_scope("scope1".to_string()).unwrap();
        let _id2 = tracker.enter_scope("scope2".to_string()).unwrap();
        tracker.exit_scope(id1).unwrap();

        let all_scopes = tracker.get_all_scopes();

        assert_eq!(all_scopes.len(), 2, "Should have 2 scopes");
    }

    /// Objective: Verify get_scope_lifecycle_metrics functionality
    /// Invariants: Should return metrics for completed scopes
    #[test]
    fn test_get_scope_lifecycle_metrics() {
        let tracker = ScopeTracker::new();

        let id = tracker.enter_scope("test".to_string()).unwrap();
        tracker.associate_variable("var".to_string(), 100).unwrap();
        tracker.exit_scope(id).unwrap();

        let metrics = tracker
            .get_scope_lifecycle_metrics()
            .expect("Should get metrics");

        assert_eq!(metrics.len(), 1, "Should have 1 metric");
        assert_eq!(metrics[0].scope_name, "test", "Scope name should match");
        assert_eq!(metrics[0].variable_count, 1, "Should have 1 variable");
    }

    /// Objective: Verify ScopeGuard RAII behavior
    /// Invariants: Scope should be exited when guard is dropped
    #[test]
    fn test_scope_guard() {
        let tracker = get_global_scope_tracker();

        let guard = ScopeGuard::enter("test_guard").expect("Should enter");
        let scope_id = guard.scope_id();

        let active = tracker.active_scopes.read().unwrap();
        assert!(active.contains_key(&scope_id), "Scope should be active");
        drop(active);

        drop(guard);

        let active = tracker.active_scopes.read().unwrap();
        assert!(
            !active.contains_key(&scope_id),
            "Scope should be exited after guard drop"
        );
    }

    /// Objective: Verify ScopeGuard scope_id method
    /// Invariants: Should return the scope ID
    #[test]
    fn test_scope_guard_id() {
        let guard = ScopeGuard::enter("test").expect("Should enter");
        let id = guard.scope_id();
        assert!(id > 0, "Scope ID should be positive");
    }

    /// Objective: Verify exiting invalid scope returns error
    /// Invariants: Should return error for non-existent scope
    #[test]
    fn test_exit_invalid_scope() {
        let tracker = ScopeTracker::new();
        let result = tracker.exit_scope(999);

        assert!(result.is_err(), "Should return error for invalid scope");
    }

    /// Objective: Verify scope hierarchy tracking
    /// Invariants: Hierarchy should track parent-child relationships
    #[test]
    fn test_scope_hierarchy() {
        let tracker = ScopeTracker::new();

        let _parent = tracker.enter_scope("parent".to_string()).unwrap();
        let _child = tracker.enter_scope("child".to_string()).unwrap();

        let hierarchy = tracker.scope_hierarchy.lock().unwrap();
        assert!(
            hierarchy.relationships.contains_key("parent"),
            "Should have parent relationship"
        );
        assert!(
            hierarchy.depth_map.contains_key("parent"),
            "Should have parent depth"
        );
        assert!(
            hierarchy.depth_map.contains_key("child"),
            "Should have child depth"
        );
    }

    /// Objective: Verify multiple variables in scope
    /// Invariants: All variables should be tracked
    #[test]
    fn test_multiple_variables() {
        let tracker = ScopeTracker::new();
        let _scope_id = tracker.enter_scope("test".to_string()).unwrap();

        tracker.associate_variable("var1".to_string(), 100).unwrap();
        tracker.associate_variable("var2".to_string(), 200).unwrap();
        tracker.associate_variable("var3".to_string(), 300).unwrap();

        let active = tracker.active_scopes.read().unwrap();
        let scope = active.values().next().expect("Should have scope");

        assert_eq!(scope.variables.len(), 3, "Should have 3 variables");
        assert_eq!(scope.allocation_count, 3, "Should have 3 allocations");
    }

    /// Objective: Verify current_timestamp returns valid value
    /// Invariants: Timestamp should be positive
    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 0, "Timestamp should be positive");
    }

    /// Objective: Verify global scope tracker singleton
    /// Invariants: Multiple calls should return same instance
    #[test]
    fn test_global_scope_tracker_singleton() {
        let tracker1 = get_global_scope_tracker();
        let tracker2 = get_global_scope_tracker();

        assert!(Arc::ptr_eq(&tracker1, &tracker2));
    }
}
