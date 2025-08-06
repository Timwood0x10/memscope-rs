//! Scope tracking functionality for memory analysis

use crate::core::types::*;
use crate::core::optimized_locks::OptimizedMutex;
use crate::core::sharded_locks::ShardedRwLock;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::sync::atomic::{AtomicU64, Ordering};
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
    /// Active scopes using sharded storage for better concurrency
    pub active_scopes: ShardedRwLock<ScopeId, ScopeInfo>,
    /// Completed scopes for analysis
    pub completed_scopes: OptimizedMutex<Vec<ScopeInfo>>,
    /// Scope hierarchy relationships
    pub scope_hierarchy: OptimizedMutex<ScopeHierarchy>,
    /// Next available scope ID using atomic counter
    next_scope_id: AtomicU64,
    /// Current scope stack per thread using sharded storage
    pub scope_stack: ShardedRwLock<String, Vec<ScopeId>>,
}

impl ScopeTracker {
    /// Create a new scope tracker
    pub fn new() -> Self {
        Self {
            active_scopes: ShardedRwLock::new(),
            completed_scopes: OptimizedMutex::new(Vec::new()),
            scope_hierarchy: OptimizedMutex::new(ScopeHierarchy {
                root_scopes: Vec::new(),
                scope_tree: HashMap::new(),
                max_depth: 0,
                total_scopes: 0,
                relationships: HashMap::new(),
                depth_map: HashMap::new(),
            }),
            next_scope_id: AtomicU64::new(1),
            scope_stack: ShardedRwLock::new(),
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&self, name: String) -> TrackingResult<ScopeId> {
        let scope_id = self.allocate_scope_id();
        let thread_id = format!("{:?}", std::thread::current().id());
        let timestamp = current_timestamp();

        // Determine parent scope and depth
        let (parent_scope, depth) = {
            if let Some(thread_stack) = self.scope_stack.get(&thread_id) {
                if let Some(&parent_id) = thread_stack.last() {
                    if let Some(parent) = self.active_scopes.get(&parent_id) {
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

        // Create scope info
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

        // Add to active scopes
        self.active_scopes.insert(scope_id, scope_info);

        // Update scope stack
        self.scope_stack.with_shard_write(&thread_id, |shard| {
            shard.entry(thread_id.clone()).or_default().push(scope_id);
        });

        // Update hierarchy
        if let Some(mut hierarchy) = self.scope_hierarchy.try_lock() {
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
    pub fn exit_scope(&self, scope_id: ScopeId) -> TrackingResult<()> {
        let thread_id = format!("{:?}", std::thread::current().id());
        let timestamp = current_timestamp();

        // Remove from active scopes and get scope info
        let mut scope_info = self.active_scopes.remove(&scope_id).ok_or_else(|| {
            TrackingError::InvalidPointer(format!("Invalid scope ID: {scope_id}"))
        })?;

        // Update end time
        scope_info.end_time = Some(timestamp as u64);
        scope_info.lifetime_end = Some(timestamp as u64);

        // Update scope stack
        self.scope_stack.with_shard_write(&thread_id, |shard| {
            if let Some(thread_stack) = shard.get_mut(&thread_id) {
                if let Some(pos) = thread_stack.iter().position(|&id| id == scope_id) {
                    thread_stack.remove(pos);
                }
            }
        });

        // Add to completed scopes
        if let Some(mut completed_scopes) = self.completed_scopes.try_lock() {
            completed_scopes.push(scope_info);
        }

        Ok(())
    }

    /// Associate a variable with the current scope
    pub fn associate_variable(
        &self,
        variable_name: String,
        memory_size: usize,
    ) -> TrackingResult<()> {
        let thread_id = format!("{:?}", std::thread::current().id());

        // Find current scope for this thread
        let current_scope_id = self.scope_stack.get(&thread_id)
            .and_then(|stack| stack.last().copied());

        if let Some(scope_id) = current_scope_id {
            self.active_scopes.with_shard_write(&scope_id, |shard| {
                if let Some(scope) = shard.get_mut(&scope_id) {
                    scope.variables.push(variable_name);
                    scope.memory_usage += memory_size;
                    scope.peak_memory = scope.peak_memory.max(scope.memory_usage);
                    scope.allocation_count += 1;
                }
            });
        }

        Ok(())
    }

    /// Get current scope analysis
    pub fn get_scope_analysis(&self) -> TrackingResult<ScopeAnalysis> {
        let mut all_scopes = Vec::new();
        
        // Collect active scopes from all shards
        for i in 0..16 { // Default shard count
            self.active_scopes.with_shard_read(&i, |shard| {
                all_scopes.extend(shard.values().cloned());
            });
        }
        
        // Add completed scopes
        if let Some(completed_scopes) = self.completed_scopes.try_lock() {
            all_scopes.extend(completed_scopes.iter().cloned());
        }

        let hierarchy = if let Some(hierarchy) = self.scope_hierarchy.try_lock() {
            hierarchy.clone()
        } else {
            // Return default hierarchy if lock fails
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
            average_lifetime: 1000.0, // Placeholder
            memory_efficiency: 0.8,   // Placeholder
            scopes: all_scopes,
            scope_hierarchy: hierarchy,
            cross_scope_references: Vec::new(), // TODO: Implement cross-scope reference tracking
        })
    }

    /// Get scope lifecycle metrics
    pub fn get_scope_lifecycle_metrics(&self) -> TrackingResult<Vec<ScopeLifecycleMetrics>> {
        let metrics = if let Some(completed_scopes) = self.completed_scopes.try_lock() {
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
                        allocation_frequency: 1.0, // Simplified
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

    /// Get all scopes (active and completed) for data localization
    pub fn get_all_scopes(&self) -> Vec<ScopeInfo> {
        let mut all_scopes = Vec::new();

        // Add active scopes from all shards
        for i in 0..16 { // Default shard count
            self.active_scopes.with_shard_read(&i, |shard| {
                all_scopes.extend(shard.values().cloned());
            });
        }

        // Add completed scopes
        if let Some(completed_scopes) = self.completed_scopes.try_lock() {
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
    /// Enter a new scope with automatic cleanup
    pub fn enter(name: &str) -> TrackingResult<Self> {
        let tracker = get_global_scope_tracker();
        let scope_id = tracker.enter_scope(name.to_string())?;

        Ok(Self { scope_id, tracker })
    }

    /// Get the scope ID
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        let _ = self.tracker.exit_scope(self.scope_id);
    }
}

/// Get current timestamp in milliseconds
fn current_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
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
            if let Some(size) = $crate::Trackable::get_heap_ptr(&$var) {
                let scope_tracker = $crate::scope_tracker::get_global_scope_tracker();
                let _ = scope_tracker
                    .associate_variable(stringify!($var).to_string(), std::mem::size_of_val(&$var));
            }
        }

        result
    }};
}
