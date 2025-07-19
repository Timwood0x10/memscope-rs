//! Variable Registry - Simple HashMap-based variable name tracking
//! 
//! This module provides a lightweight alternative to log-based tracking,
//! using a global HashMap to store variable address -> variable info mappings.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use crate::types::TrackingResult;
use rayon::prelude::*;

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

    /// Enhance tracker allocations with variable names from registry (optimized with parallel processing)
    pub fn enhance_allocations_with_registry(
        allocations: &[crate::types::AllocationInfo],
    ) -> Vec<serde_json::Value> {
        // Early return for small datasets
        if allocations.len() < 100 {
            return Self::enhance_allocations_sequential(allocations);
        }
        
        println!("🚀 Processing {} allocations with parallel optimization...", allocations.len());
        
        let registry = Self::get_all_variables();
        let start_time = std::time::Instant::now();
        
        // Use parallel processing for large datasets
        let enhanced: Vec<serde_json::Value> = allocations
            .par_iter()
            .map(|alloc| {
                Self::enhance_single_allocation(alloc, &registry)
            })
            .collect();
        
        let duration = start_time.elapsed();
        println!("✅ Parallel processing completed in {:?} ({:.2} allocs/ms)", 
                 duration, allocations.len() as f64 / duration.as_millis() as f64);
        
        enhanced
    }
    
    /// Sequential processing for small datasets
    fn enhance_allocations_sequential(
        allocations: &[crate::types::AllocationInfo],
    ) -> Vec<serde_json::Value> {
        let registry = Self::get_all_variables();
        
        allocations.iter().map(|alloc| {
            Self::enhance_single_allocation(alloc, &registry)
        }).collect()
    }
    
    /// Enhance a single allocation (optimized for performance)
    fn enhance_single_allocation(
        alloc: &crate::types::AllocationInfo,
        registry: &HashMap<usize, VariableInfo>,
    ) -> serde_json::Value {
        // Check registry first (fastest path)
        if let Some(var_info) = registry.get(&alloc.ptr) {
            return serde_json::json!({
                "ptr": alloc.ptr,
                "size": alloc.size,
                "timestamp_alloc": alloc.timestamp_alloc,
                "timestamp_dealloc": alloc.timestamp_dealloc,
                "variable_name": var_info.var_name,
                "type_name": var_info.type_name,
                "registry_timestamp": var_info.timestamp,
                "registry_size": var_info.size,
                "source": "registry"
            });
        }
        
        // Use existing data if available
        if let (Some(var_name), Some(type_name)) = (&alloc.var_name, &alloc.type_name) {
            return serde_json::json!({
                "ptr": alloc.ptr,
                "size": alloc.size,
                "timestamp_alloc": alloc.timestamp_alloc,
                "timestamp_dealloc": alloc.timestamp_dealloc,
                "variable_name": var_name,
                "type_name": type_name,
                "source": "existing"
            });
        }
        
        // Smart inference as last resort
        let (inferred_var_name, inferred_type_name) = Self::infer_allocation_info_cached(alloc);
        
        serde_json::json!({
            "ptr": alloc.ptr,
            "size": alloc.size,
            "timestamp_alloc": alloc.timestamp_alloc,
            "timestamp_dealloc": alloc.timestamp_dealloc,
            "variable_name": alloc.var_name.as_ref().unwrap_or(&inferred_var_name),
            "type_name": alloc.type_name.as_ref().unwrap_or(&inferred_type_name),
            "source": "inferred"
        })
    }

    /// Smart inference with caching for better performance
    pub fn infer_allocation_info_cached(alloc: &crate::types::AllocationInfo) -> (String, String) {
        // Use a simple cache for common sizes to avoid repeated string formatting
        static COMMON_TYPES: &[(usize, &str, &str)] = &[
            (1, "box_u8", "Box<u8>"),
            (2, "box_u16", "Box<u16>"),
            (4, "box_u32", "Box<u32>"),
            (8, "box_u64", "Box<u64>"),
            (16, "small_alloc_16b", "SmallAlloc"),
            (24, "string_alloc", "String"),
            (32, "string_alloc", "String"),
        ];
        
        // Fast lookup for common sizes
        for &(size, var_prefix, type_name) in COMMON_TYPES {
            if alloc.size == size {
                return (
                    format!("{}_{:x}", var_prefix, alloc.ptr),
                    type_name.to_string()
                );
            }
        }
        
        // Fallback to original logic for uncommon sizes
        Self::infer_allocation_info(alloc)
    }
    
    /// Smart inference for system allocations based on size patterns and common allocations
    pub fn infer_allocation_info(alloc: &crate::types::AllocationInfo) -> (String, String) {
        let size = alloc.size;
        
        // Common allocation size patterns for type inference
        let (var_name, type_name) = match size {
            // String allocations (common sizes)
            8..=32 if size.is_power_of_two() => {
                (format!("string_alloc_{:x}", alloc.ptr), "String".to_string())
            },
            // Vec allocations (multiples of common element sizes)
            s if s % 8 == 0 && s >= 16 => {
                let elements = s / 8;
                (format!("vec_i64_{}elem_{:x}", elements, alloc.ptr), "Vec<i64>".to_string())
            },
            s if s % 4 == 0 && s >= 8 => {
                let elements = s / 4;
                (format!("vec_i32_{}elem_{:x}", elements, alloc.ptr), "Vec<i32>".to_string())
            },
            // Box allocations (single element sizes)
            1 => (format!("box_u8_{:x}", alloc.ptr), "Box<u8>".to_string()),
            2 => (format!("box_u16_{:x}", alloc.ptr), "Box<u16>".to_string()),
            4 => (format!("box_u32_{:x}", alloc.ptr), "Box<u32>".to_string()),
            8 => (format!("box_u64_{:x}", alloc.ptr), "Box<u64>".to_string()),
            // HashMap/BTreeMap allocations (typically larger, irregular sizes)
            s if s >= 64 && s % 16 == 0 => {
                (format!("hashmap_alloc_{:x}", alloc.ptr), "HashMap<K,V>".to_string())
            },
            // Large allocations (likely buffers or large collections)
            s if s >= 1024 => {
                let kb = s / 1024;
                (format!("large_buffer_{}kb_{:x}", kb, alloc.ptr), "LargeBuffer".to_string())
            },
            // Small system allocations
            s if s <= 16 => {
                (format!("small_alloc_{}b_{:x}", s, alloc.ptr), "SmallAlloc".to_string())
            },
            // Default case with size hint
            _ => {
                (format!("system_alloc_{}b_{:x}", size, alloc.ptr), "SystemAlloc".to_string())
            }
        };
        
        (var_name, type_name)
    }

    /// Generate comprehensive export data combining tracker + registry (optimized)
    pub fn generate_comprehensive_export(
        tracker: &crate::tracker::MemoryTracker,
    ) -> TrackingResult<serde_json::Value> {
        let start_time = std::time::Instant::now();
        println!("🔄 Starting comprehensive export generation...");
        
        // Get tracker data in parallel where possible
        let (active_allocations, other_data) = rayon::join(
            || tracker.get_active_allocations(),
            || {
                let history = tracker.get_allocation_history();
                let memory_types = tracker.get_memory_by_type();
                let stats = tracker.get_stats();
                let registry = Self::get_all_variables();
                (history, memory_types, stats, registry)
            }
        );
        
        let active_allocations = active_allocations?;
        let (allocation_history, memory_by_type, stats, registry) = {
            let allocation_history = other_data.0?;
            let memory_by_type = other_data.1?;
            let stats = other_data.2?;
            let registry = other_data.3;
            (allocation_history, memory_by_type, stats, registry)
        };
        
        println!("📊 Data loaded: {} active, {} history, {} registry entries", 
                 active_allocations.len(), allocation_history.len(), registry.len());
        
        // Filter out very small allocations to reduce processing overhead (optional optimization)
        let filtered_active: Vec<_> = if active_allocations.len() > 10000 {
            active_allocations.into_iter()
                .filter(|alloc| alloc.size >= 8) // Skip tiny allocations
                .collect()
        } else {
            active_allocations
        };
        
        let filtered_history: Vec<_> = if allocation_history.len() > 50000 {
            allocation_history.into_iter()
                .filter(|alloc| alloc.size >= 8) // Skip tiny allocations
                .collect()
        } else {
            allocation_history
        };
        
        // Process allocations in parallel
        let (enhanced_active, enhanced_history) = rayon::join(
            || Self::enhance_allocations_with_registry(&filtered_active),
            || Self::enhance_allocations_with_registry(&filtered_history)
        );
        
        // Build result efficiently
        let registry_variables: Vec<_> = registry.values().collect();
        let enhanced_count = enhanced_active.len() + enhanced_history.len();
        
        let comprehensive_data = serde_json::json!({
            "memory_analysis": {
                "active_allocations": enhanced_active,
                "allocation_history": enhanced_history,
                "memory_by_type": memory_by_type,
                "statistics": stats
            },
            "variable_registry": {
                "total_variables": registry.len(),
                "variables": registry_variables
            },
            "export_metadata": {
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "enhanced_count": enhanced_count,
                "processing_time_ms": start_time.elapsed().as_millis(),
                "optimizations_applied": [
                    "parallel_processing",
                    "allocation_filtering",
                    "cached_inference"
                ]
            }
        });
        
        let total_time = start_time.elapsed();
        println!("✅ Export generation completed in {:?} ({} total allocations)", 
                 total_time, enhanced_count);
        
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