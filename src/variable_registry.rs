//! Variable Registry - Simple HashMap-based variable name tracking
//! 
//! This module provides a lightweight alternative to log-based tracking,
//! using a global HashMap to store variable address -> variable info mappings.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use crate::types::TrackingResult;

/// Variable information stored in registry
#[derive(Debug, Clone, serde::Serialize)]
pub struct VariableInfo {
    /// User-defined variable name
    pub var_name: String,
    /// Type name of the variable
    pub type_name: String,
    /// Timestamp when variable was registered
    pub timestamp: u64,
    /// Estimated size of the variable
    pub size: usize,
}

/// Global variable registry using HashMap for fast lookups
static GLOBAL_VARIABLE_REGISTRY: OnceLock<Arc<Mutex<HashMap<usize, VariableInfo>>>> = OnceLock::new();

/// Get or initialize the global variable registry
fn get_global_registry() -> Arc<Mutex<HashMap<usize, VariableInfo>>> {
    GLOBAL_VARIABLE_REGISTRY
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .clone()
}

/// Variable Registry - manages variable address to name mappings
pub struct VariableRegistry;

impl VariableRegistry {
    /// Register a variable with its address and information
    pub fn register_variable(
        address: usize,
        var_name: String,
        type_name: String,
        size: usize,
    ) -> TrackingResult<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let var_info = VariableInfo {
            var_name,
            type_name,
            timestamp,
            size,
        };

        if let Ok(mut registry) = get_global_registry().try_lock() {
            registry.insert(address, var_info);
        }

        Ok(())
    }

    /// Get variable information by address
    pub fn get_variable_info(address: usize) -> Option<VariableInfo> {
        if let Ok(registry) = get_global_registry().try_lock() {
            registry.get(&address).cloned()
        } else {
            None
        }
    }

    /// Get all variable mappings
    pub fn get_all_variables() -> HashMap<usize, VariableInfo> {
        if let Ok(registry) = get_global_registry().try_lock() {
            registry.clone()
        } else {
            HashMap::new()
        }
    }

    /// Enhance tracker allocations with variable names from registry
    pub fn enhance_allocations_with_registry(
        allocations: &[crate::types::AllocationInfo],
    ) -> Vec<serde_json::Value> {
        let registry = Self::get_all_variables();
        
        allocations.iter().map(|alloc| {
            let mut enhanced = serde_json::json!({
                "ptr": alloc.ptr,
                "size": alloc.size,
                "timestamp_alloc": alloc.timestamp_alloc,
                "timestamp_dealloc": alloc.timestamp_dealloc,
                "variable_name": alloc.var_name.clone().unwrap_or_else(|| 
                    format!("unknown_object_{:x}", alloc.ptr)
                ),
                "type_name": alloc.type_name.clone().unwrap_or_else(|| "Unknown".to_string())
            });
            
            // Enhance with registry data if available
            if let Some(var_info) = registry.get(&alloc.ptr) {
                enhanced["variable_name"] = serde_json::Value::String(var_info.var_name.clone());
                enhanced["type_name"] = serde_json::Value::String(var_info.type_name.clone());
                enhanced["registry_timestamp"] = serde_json::Value::Number(var_info.timestamp.into());
                enhanced["registry_size"] = serde_json::Value::Number(var_info.size.into());
            }
            
            enhanced
        }).collect()
    }

    /// Generate comprehensive export data combining tracker + registry
    pub fn generate_comprehensive_export(
        tracker: &crate::tracker::MemoryTracker,
    ) -> TrackingResult<serde_json::Value> {
        // Get tracker data
        let active_allocations = tracker.get_active_allocations()?;
        let allocation_history = tracker.get_allocation_history()?;
        let memory_by_type = tracker.get_memory_by_type()?;
        let stats = tracker.get_stats()?;
        
        // Get registry data
        let registry = Self::get_all_variables();
        
        // Enhance allocations with registry information
        let enhanced_active = Self::enhance_allocations_with_registry(&active_allocations);
        let enhanced_history = Self::enhance_allocations_with_registry(&allocation_history);
        
        // Create comprehensive export data
        let comprehensive_data = serde_json::json!({
            "memory_analysis": {
                "active_allocations": enhanced_active,
                "allocation_history": enhanced_history,
                "memory_by_type": memory_by_type,
                "statistics": stats
            },
            "variable_registry": {
                "total_variables": registry.len(),
                "variables": registry.values().collect::<Vec<_>>()
            },
            "export_metadata": {
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "enhanced_count": enhanced_active.len() + enhanced_history.len()
            }
        });
        
        Ok(comprehensive_data)
    }

    /// Clear all variable registrations
    pub fn clear_registry() -> TrackingResult<()> {
        if let Ok(mut registry) = get_global_registry().try_lock() {
            registry.clear();
        }
        Ok(())
    }

    /// Get registry statistics
    pub fn get_stats() -> (usize, usize) {
        if let Ok(registry) = get_global_registry().try_lock() {
            let total = registry.len();
            let recent = registry.values()
                .filter(|v| {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64;
                    now - v.timestamp < 1_000_000_000 // Last 1 second
                })
                .count();
            (total, recent)
        } else {
            (0, 0)
        }
    }
}