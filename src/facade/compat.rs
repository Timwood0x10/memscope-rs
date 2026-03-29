//! Compatibility layer for legacy API
//!
//! This module provides compatibility functions to bridge the old API
//! with the new facade-based API, ensuring smooth migration for
//! existing code.

use crate::facade::MemScope;
use crate::metadata::registry::VariableInfo;
use std::sync::{Arc, OnceLock};

/// Global MemScope instance for compatibility
static GLOBAL_MEMSCOPE: OnceLock<Arc<MemScope>> = OnceLock::new();

/// Get or create the global MemScope instance
///
/// This function provides compatibility with the old global tracker pattern.
/// New code should use `MemScope::new()` directly.
pub fn get_global_memscope() -> Arc<MemScope> {
    GLOBAL_MEMSCOPE
        .get_or_init(|| Arc::new(MemScope::new()))
        .clone()
}

/// Compatibility wrapper for variable registration
///
/// This function is used to register variable metadata with the
/// global MemScope instance.
///
/// # Arguments
/// * `address` - The memory address of the variable
/// * `var_name` - The name of the variable
/// * `type_name` - The type name of the variable
/// * `size` - The size of the variable in bytes
pub fn register_variable(address: usize, var_name: String, type_name: String, size: usize) {
    let memscope = get_global_memscope();
    memscope
        .metadata
        .variable_registry
        .register_variable(address, var_name, type_name, size);
}

/// Get variable information by address
///
/// # Arguments
/// * `address` - The memory address of the variable
///
/// # Returns
/// The variable information if found, None otherwise
pub fn get_variable_info(address: usize) -> Option<VariableInfo> {
    let memscope = get_global_memscope();
    memscope
        .metadata
        .variable_registry
        .get_variable_info(address)
}

/// Get a summary of current memory usage
///
/// This function provides compatibility with the old API.
pub fn get_memory_summary() -> crate::query::SummaryQueryResult {
    let memscope = get_global_memscope();
    match memscope.summary() {
        crate::query::QueryResult::Summary(summary) => summary,
        _ => crate::query::SummaryQueryResult {
            total_allocations: 0,
            total_deallocations: 0,
            active_allocations: 0,
            total_allocated: 0,
            total_deallocated: 0,
            current_memory: 0,
            peak_memory: 0,
            thread_count: 0,
        },
    }
}

/// Get top allocations by size
///
/// # Arguments
/// * `limit` - Maximum number of allocations to return
pub fn get_top_allocations(limit: usize) -> Vec<crate::snapshot::ActiveAllocation> {
    let memscope = get_global_memscope();
    match memscope.top_allocations(limit) {
        crate::query::QueryResult::Allocations(result) => result.allocations,
        _ => Vec::new(),
    }
}

/// Render current snapshot to JSON
///
/// # Arguments
/// * `verbose` - Whether to include verbose output
///
/// # Returns
/// JSON string of the current memory snapshot
pub fn export_json(verbose: bool) -> Result<String, String> {
    let memscope = get_global_memscope();
    let result = memscope.render_json(verbose)?;
    String::from_utf8(result.data).map_err(|e| e.to_string())
}

/// Clear all events and reset state
pub fn clear_all() {
    let memscope = get_global_memscope();
    memscope.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_memscope_creation() {
        let memscope = get_global_memscope();
        assert_eq!(memscope.event_count(), 0);
    }

    #[test]
    fn test_register_variable() {
        let address = 0x1000;
        register_variable(address, "test_var".to_string(), "i32".to_string(), 4);

        let info = get_variable_info(address);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.var_name, "test_var");
        assert_eq!(info.type_name, "i32");
        assert_eq!(info.size, 4);
    }

    #[test]
    fn test_get_memory_summary() {
        let summary = get_memory_summary();
        assert_eq!(summary.total_allocations, 0);
    }

    #[test]
    fn test_get_top_allocations() {
        let allocations = get_top_allocations(10);
        assert!(allocations.is_empty());
    }

    #[test]
    fn test_clear_all() {
        let memscope = get_global_memscope();
        memscope.clear();
        assert_eq!(memscope.event_count(), 0);
    }
}
