//! Variable Registry - Simple HashMap-based variable name tracking
//!
//! This module provides a lightweight alternative to log-based tracking,
//! using a global HashMap to store variable address -> variable info mappings.

use crate::core::types::TrackingResult;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

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
static GLOBAL_VARIABLE_REGISTRY: OnceLock<Arc<Mutex<HashMap<usize, VariableInfo>>>> =
    OnceLock::new();

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

    /// Mark a variable as destroyed with destruction timestamp
    pub fn mark_variable_destroyed(address: usize, destruction_time: u64) -> TrackingResult<()> {
        // For now, we keep the variable in registry but could add destruction_time field
        // This method ensures the variable registry is aware of destruction events
        tracing::debug!(
            "Variable at address 0x{:x} destroyed at {}",
            address,
            destruction_time
        );
        Ok(())
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
        allocations: &[crate::core::types::AllocationInfo],
    ) -> Vec<serde_json::Value> {
        // Early return for small datasets
        if allocations.len() < 100 {
            return Self::enhance_allocations_sequential(allocations);
        }

        tracing::info!(
            "🚀 Processing {} allocations with parallel optimization...",
            allocations.len()
        );

        let registry = Self::get_all_variables();
        let start_time = std::time::Instant::now();

        // Use parallel processing for large datasets
        let enhanced: Vec<serde_json::Value> = allocations
            .par_iter()
            .map(|alloc| Self::classify_single_allocation(alloc, &registry))
            .collect();

        let duration = start_time.elapsed();
        tracing::info!(
            "✅ Parallel processing completed in {:?} ({:.2} allocs/ms)",
            duration,
            allocations.len() as f64 / duration.as_millis() as f64
        );

        enhanced
    }

    /// Sequential processing for small datasets
    fn enhance_allocations_sequential(
        allocations: &[crate::core::types::AllocationInfo],
    ) -> Vec<serde_json::Value> {
        let registry = Self::get_all_variables();

        allocations
            .iter()
            .map(|alloc| Self::classify_single_allocation(alloc, &registry))
            .collect()
    }

    /// Classify and enhance allocations with user/system distinction and scope information
    fn classify_and_enhance_allocations(
        allocations: &[crate::core::types::AllocationInfo],
        registry: &HashMap<usize, VariableInfo>,
    ) -> Vec<serde_json::Value> {
        allocations
            .par_iter()
            .map(|alloc| Self::classify_single_allocation(alloc, registry))
            .collect()
    }

    /// Classify a single allocation as user or system with full context
    fn classify_single_allocation(
        alloc: &crate::core::types::AllocationInfo,
        registry: &HashMap<usize, VariableInfo>,
    ) -> serde_json::Value {
        // Check if this is a user-tracked variable (highest priority)
        if let Some(var_info) = registry.get(&alloc.ptr) {
            return serde_json::json!({
                "ptr": alloc.ptr,
                "size": alloc.size,
                "timestamp_alloc": alloc.timestamp_alloc,
                "timestamp_dealloc": alloc.timestamp_dealloc,
                "variable_name": var_info.var_name,
                "type_name": var_info.type_name,
                "scope_name": Self::extract_scope_from_var_name(&var_info.var_name),
                "allocation_source": "user",
                "tracking_method": "track_var_macro",
                "registry_timestamp": var_info.timestamp,
                "registry_size": var_info.size,
                "lifetime_ms": alloc.timestamp_dealloc.map(|dealloc|
                    (dealloc.saturating_sub(alloc.timestamp_alloc)) / 1_000_000
                ),
                "current_age_ms": if alloc.timestamp_dealloc.is_none() {
                    // For active allocations, calculate how long they've been alive
                    let current_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64;
                    Some((current_time.saturating_sub(alloc.timestamp_alloc)) / 1_000_000)
                } else {
                    None
                },
                "is_active": alloc.timestamp_dealloc.is_none()
            });
        }

        // Check if allocation has explicit variable information (user allocation)
        if let (Some(var_name), Some(type_name)) = (&alloc.var_name, &alloc.type_name) {
            return serde_json::json!({
                "ptr": alloc.ptr,
                "size": alloc.size,
                "timestamp_alloc": alloc.timestamp_alloc,
                "timestamp_dealloc": alloc.timestamp_dealloc,
                "variable_name": var_name,
                "type_name": type_name,
                "scope_name": alloc.scope_name.as_deref().unwrap_or("user_scope"),
                "allocation_source": "user",
                "tracking_method": "explicit_tracking",
                "lifetime_ms": alloc.timestamp_dealloc.map(|dealloc|
                    (dealloc.saturating_sub(alloc.timestamp_alloc)) / 1_000_000
                ),
                "current_age_ms": if alloc.timestamp_dealloc.is_none() {
                    let current_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64;
                    Some((current_time.saturating_sub(alloc.timestamp_alloc)) / 1_000_000)
                } else {
                    None
                },
                "is_active": alloc.timestamp_dealloc.is_none()
            });
        }

        // This is a system allocation - apply smart inference
        let (inferred_var_name, inferred_type_name) = Self::infer_allocation_info_cached(alloc);
        let system_category = Self::categorize_system_allocation(alloc);

        serde_json::json!({
            "ptr": alloc.ptr,
            "size": alloc.size,
            "timestamp_alloc": alloc.timestamp_alloc,
            "timestamp_dealloc": alloc.timestamp_dealloc,
            "variable_name": inferred_var_name,
            "type_name": inferred_type_name,
            "scope_name": "system",
            "allocation_source": "system",
            "tracking_method": "automatic_inference",
            "system_category": system_category,
            "lifetime_ms": alloc.timestamp_dealloc.map(|dealloc|
                (dealloc.saturating_sub(alloc.timestamp_alloc)) / 1_000_000
            ),
            "current_age_ms": if alloc.timestamp_dealloc.is_none() {
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;
                Some((current_time.saturating_sub(alloc.timestamp_alloc)) / 1_000_000)
            } else {
                None
            },
            "is_active": alloc.timestamp_dealloc.is_none()
        })
    }

    /// Extract scope information from variable name and current scope context
    fn extract_scope_from_var_name(var_name: &str) -> String {
        // First, try to get the current scope from the scope tracker
        if let Some(current_scope) = Self::get_current_scope_name() {
            return current_scope;
        }

        // Fallback: Try to extract scope from variable name patterns
        if var_name.contains("::") {
            if let Some(scope_part) = var_name.split("::").next() {
                return scope_part.to_string();
            }
        }

        // Check for common scope patterns
        if var_name.starts_with("main_") {
            "main_function".to_string()
        } else if var_name.starts_with("test_") {
            "test_function".to_string()
        } else if var_name.starts_with("fn_") {
            "user_function".to_string()
        } else if var_name.contains("_vec")
            || var_name.contains("_string")
            || var_name.contains("_data")
        {
            "user_scope".to_string()
        } else if var_name.starts_with("boxed_")
            || var_name.starts_with("rc_")
            || var_name.starts_with("arc_")
        {
            "user_scope".to_string()
        } else {
            // For user variables, default to user_scope instead of global
            "user_scope".to_string()
        }
    }

    /// Get the current scope name from the scope tracker or infer from call stack
    fn get_current_scope_name() -> Option<String> {
        // First try to get from scope tracker
        if let Some(scope_name) = Self::get_scope_from_tracker() {
            return Some(scope_name);
        }

        // Fallback: Try to infer scope from call stack
        Self::infer_scope_from_call_stack()
    }

    /// Get scope from the scope tracker
    fn get_scope_from_tracker() -> Option<String> {
        use crate::core::scope_tracker::get_global_scope_tracker;

        let scope_tracker = get_global_scope_tracker();
        let thread_id = format!("{:?}", std::thread::current().id());

        // Try to get the current scope from the scope stack
        if let Ok(scope_stack) = scope_tracker.scope_stack.try_lock() {
            if let Some(thread_stack) = scope_stack.get(&thread_id) {
                if let Some(&current_scope_id) = thread_stack.last() {
                    // Get the scope name from active scopes
                    if let Ok(active_scopes) = scope_tracker.active_scopes.try_lock() {
                        if let Some(scope_info) = active_scopes.get(&current_scope_id) {
                            return Some(scope_info.name.clone());
                        }
                    }
                }
            }
        }

        None
    }

    /// Infer scope from call stack information
    fn infer_scope_from_call_stack() -> Option<String> {
        // Try to get function name from backtrace
        let backtrace = std::backtrace::Backtrace::capture();
        let backtrace_str = format!("{}", backtrace);

        // Look for function names in the backtrace
        for line in backtrace_str.lines() {
            if line.contains("::main") {
                return Some("main_function".to_string());
            }
            if line.contains("test_") || line.contains("tests::") {
                return Some("test_function".to_string());
            }
            // Look for user-defined function patterns
            if let Some(func_name) = Self::extract_function_name_from_backtrace(line) {
                return Some(format!("function_{}", func_name));
            }
        }

        // If we can't determine the scope, use a more descriptive default
        Some("user_code_scope".to_string())
    }

    /// Extract function name from backtrace line
    fn extract_function_name_from_backtrace(line: &str) -> Option<String> {
        // Try to extract function names from common patterns
        if let Some(start) = line.find("::") {
            if let Some(end) = line[start + 2..].find("::") {
                let func_name = &line[start + 2..start + 2 + end];
                // Filter out common system functions
                if !func_name.starts_with("_")
                    && !func_name.contains("alloc")
                    && !func_name.contains("std")
                    && !func_name.contains("core")
                    && func_name.len() > 2
                {
                    return Some(func_name.to_string());
                }
            }
        }
        None
    }

    /// Categorize system allocations for better understanding
    fn categorize_system_allocation(alloc: &crate::core::types::AllocationInfo) -> String {
        match alloc.size {
            1..=16 => "small_system_alloc",
            17..=64 => "medium_system_alloc",
            65..=1024 => "large_system_alloc",
            1025..=65536 => "buffer_allocation",
            _ => "huge_allocation",
        }
        .to_string()
    }

    /// Group allocations by scope for better organization
    fn group_by_scope(
        active: &[serde_json::Value],
        history: &[serde_json::Value],
    ) -> serde_json::Value {
        let mut scopes: HashMap<String, Vec<&serde_json::Value>> = HashMap::new();

        // Group active allocations
        for alloc in active {
            if let Some(scope) = alloc["scope_name"].as_str() {
                scopes.entry(scope.to_string()).or_default().push(alloc);
            }
        }

        // Group history allocations
        for alloc in history {
            if let Some(scope) = alloc["scope_name"].as_str() {
                scopes.entry(scope.to_string()).or_default().push(alloc);
            }
        }

        let scope_summary: HashMap<String, serde_json::Value> = scopes
            .into_iter()
            .map(|(scope_name, allocations)| {
                let total_size: u64 = allocations
                    .iter()
                    .map(|a| a["size"].as_u64().unwrap_or(0))
                    .sum();

                (
                    scope_name.clone(),
                    serde_json::json!({
                        "scope_name": scope_name,
                        "allocation_count": allocations.len(),
                        "total_size_bytes": total_size,
                        "allocations": allocations
                    }),
                )
            })
            .collect();

        serde_json::json!(scope_summary)
    }

    /// Get scope summary from registry
    fn get_scope_summary(registry: &HashMap<usize, VariableInfo>) -> serde_json::Value {
        let mut scope_counts: HashMap<String, usize> = HashMap::new();

        for var_info in registry.values() {
            let scope = Self::extract_scope_from_var_name(&var_info.var_name);
            *scope_counts.entry(scope).or_insert(0) += 1;
        }

        serde_json::json!(scope_counts)
    }

    /// Analyze lifecycle statistics for lifetime_ms patterns
    fn analyze_lifecycle_statistics(
        user_active: &[serde_json::Value],
        user_history: &[serde_json::Value],
        system_active: &[serde_json::Value],
        system_history: &[serde_json::Value],
    ) -> serde_json::Value {
        // Combine all allocations for analysis
        let all_user: Vec<&serde_json::Value> =
            user_active.iter().chain(user_history.iter()).collect();
        let all_system: Vec<&serde_json::Value> =
            system_active.iter().chain(system_history.iter()).collect();

        // Analyze user allocations - now all should have lifetime_ms values
        let user_lifetimes: Vec<u64> = all_user
            .iter()
            .filter_map(|a| a["lifetime_ms"].as_u64())
            .collect();

        let user_active_count = all_user
            .iter()
            .filter(|a| a["is_active"].as_bool().unwrap_or(false))
            .count();

        let user_deallocated_count = all_user
            .iter()
            .filter(|a| a["timestamp_dealloc"].is_null() == false)
            .count();

        // Analyze system allocations - now all should have lifetime_ms values
        let system_lifetimes: Vec<u64> = all_system
            .iter()
            .filter_map(|a| a["lifetime_ms"].as_u64())
            .collect();

        let system_active_count = all_system
            .iter()
            .filter(|a| a["is_active"].as_bool().unwrap_or(false))
            .count();

        let system_deallocated_count = all_system
            .iter()
            .filter(|a| a["timestamp_dealloc"].is_null() == false)
            .count();

        serde_json::json!({
            "user_allocations": {
                "total_count": all_user.len(),
                "active_count": user_active_count,
                "deallocated_count": user_deallocated_count,
                "leaked_count": user_active_count, // active = potentially leaked
                "lifetime_stats": Self::calculate_lifetime_stats(&user_lifetimes),
                "average_lifetime_ms": if !user_lifetimes.is_empty() {
                    user_lifetimes.iter().sum::<u64>() / user_lifetimes.len() as u64
                } else { 0 },
                "max_lifetime_ms": user_lifetimes.iter().max().copied().unwrap_or(0),
                "min_lifetime_ms": user_lifetimes.iter().min().copied().unwrap_or(0)
            },
            "system_allocations": {
                "total_count": all_system.len(),
                "active_count": system_active_count,
                "deallocated_count": system_deallocated_count,
                "leaked_count": system_active_count,
                "lifetime_stats": Self::calculate_lifetime_stats(&system_lifetimes),
                "average_lifetime_ms": if !system_lifetimes.is_empty() {
                    system_lifetimes.iter().sum::<u64>() / system_lifetimes.len() as u64
                } else { 0 },
                "max_lifetime_ms": system_lifetimes.iter().max().copied().unwrap_or(0),
                "min_lifetime_ms": system_lifetimes.iter().min().copied().unwrap_or(0)
            },
            "comparison": {
                "user_vs_system_active_ratio": if system_active_count > 0 {
                    user_active_count as f64 / system_active_count as f64
                } else { 0.0 },
                "user_vs_system_lifetime_ratio": if !system_lifetimes.is_empty() && !user_lifetimes.is_empty() {
                    (user_lifetimes.iter().sum::<u64>() / user_lifetimes.len() as u64) as f64 /
                    (system_lifetimes.iter().sum::<u64>() / system_lifetimes.len() as u64) as f64
                } else { 0.0 }
            }
        })
    }

    /// Analyze deallocation patterns for timestamp_dealloc
    fn analyze_deallocation_patterns(
        user_active: &[serde_json::Value],
        user_history: &[serde_json::Value],
        system_active: &[serde_json::Value],
        system_history: &[serde_json::Value],
    ) -> serde_json::Value {
        let all_user: Vec<&serde_json::Value> =
            user_active.iter().chain(user_history.iter()).collect();
        let all_system: Vec<&serde_json::Value> =
            system_active.iter().chain(system_history.iter()).collect();

        // Analyze deallocation timestamps
        let user_dealloc_times: Vec<u64> = all_user
            .iter()
            .filter_map(|a| a["timestamp_dealloc"].as_u64())
            .collect();

        let system_dealloc_times: Vec<u64> = all_system
            .iter()
            .filter_map(|a| a["timestamp_dealloc"].as_u64())
            .collect();

        // Count null deallocations (active/leaked allocations)
        let user_null_dealloc = all_user
            .iter()
            .filter(|a| a["timestamp_dealloc"].is_null())
            .count();

        let system_null_dealloc = all_system
            .iter()
            .filter(|a| a["timestamp_dealloc"].is_null())
            .count();

        serde_json::json!({
            "user_deallocations": {
                "total_deallocated": user_dealloc_times.len(),
                "still_active": user_null_dealloc,
                "deallocation_rate": if all_user.len() > 0 {
                    user_dealloc_times.len() as f64 / all_user.len() as f64 * 100.0
                } else { 0.0 },
                "earliest_dealloc": user_dealloc_times.iter().min().copied(),
                "latest_dealloc": user_dealloc_times.iter().max().copied(),
                "deallocation_timespan_ms": if user_dealloc_times.len() > 1 {
                    user_dealloc_times.iter().max().unwrap_or(&0) -
                    user_dealloc_times.iter().min().unwrap_or(&0)
                } else { 0 }
            },
            "system_deallocations": {
                "total_deallocated": system_dealloc_times.len(),
                "still_active": system_null_dealloc,
                "deallocation_rate": if all_system.len() > 0 {
                    system_dealloc_times.len() as f64 / all_system.len() as f64 * 100.0
                } else { 0.0 },
                "earliest_dealloc": system_dealloc_times.iter().min().copied(),
                "latest_dealloc": system_dealloc_times.iter().max().copied(),
                "deallocation_timespan_ms": if system_dealloc_times.len() > 1 {
                    system_dealloc_times.iter().max().unwrap_or(&0) -
                    system_dealloc_times.iter().min().unwrap_or(&0)
                } else { 0 }
            },
            "memory_leak_analysis": {
                "user_potential_leaks": user_null_dealloc,
                "system_potential_leaks": system_null_dealloc,
                "total_potential_leaks": user_null_dealloc + system_null_dealloc,
                "user_leak_percentage": if all_user.len() > 0 {
                    user_null_dealloc as f64 / all_user.len() as f64 * 100.0
                } else { 0.0 },
                "system_leak_percentage": if all_system.len() > 0 {
                    system_null_dealloc as f64 / all_system.len() as f64 * 100.0
                } else { 0.0 }
            }
        })
    }

    /// Calculate detailed lifetime statistics
    fn calculate_lifetime_stats(lifetimes: &[u64]) -> serde_json::Value {
        if lifetimes.is_empty() {
            return serde_json::json!({
                "count": 0,
                "categories": {
                    "very_short": 0,    // < 1ms
                    "short": 0,         // 1-10ms
                    "medium": 0,        // 10-100ms
                    "long": 0,          // 100-1000ms
                    "very_long": 0      // > 1000ms
                }
            });
        }

        let mut very_short = 0;
        let mut short = 0;
        let mut medium = 0;
        let mut long = 0;
        let mut very_long = 0;

        for &lifetime in lifetimes {
            match lifetime {
                0..=1 => very_short += 1,
                2..=10 => short += 1,
                11..=100 => medium += 1,
                101..=1000 => long += 1,
                _ => very_long += 1,
            }
        }

        serde_json::json!({
            "count": lifetimes.len(),
            "categories": {
                "very_short": very_short,
                "short": short,
                "medium": medium,
                "long": long,
                "very_long": very_long
            },
            "percentiles": {
                "p50": Self::calculate_percentile(lifetimes, 50.0),
                "p90": Self::calculate_percentile(lifetimes, 90.0),
                "p95": Self::calculate_percentile(lifetimes, 95.0),
                "p99": Self::calculate_percentile(lifetimes, 99.0)
            }
        })
    }

    /// Calculate percentile for lifetime analysis
    fn calculate_percentile(sorted_values: &[u64], percentile: f64) -> u64 {
        if sorted_values.is_empty() {
            return 0;
        }

        let mut values = sorted_values.to_vec();
        values.sort_unstable();

        let index = (percentile / 100.0 * (values.len() - 1) as f64) as usize;
        values[index.min(values.len() - 1)]
    }

    /// Smart inference with caching for better performance
    pub fn infer_allocation_info_cached(
        alloc: &crate::core::types::AllocationInfo,
    ) -> (String, String) {
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
                    type_name.to_string(),
                );
            }
        }

        // Fallback to original logic for uncommon sizes
        Self::infer_allocation_info(alloc)
    }

    /// Smart inference for system allocations based on size patterns and common allocations
    pub fn infer_allocation_info(alloc: &crate::core::types::AllocationInfo) -> (String, String) {
        let size = alloc.size;

        // Common allocation size patterns for type inference
        let (var_name, type_name) = match size {
            // String allocations (common sizes)
            8..=32 if size.is_power_of_two() => (
                format!("string_alloc_{:x}", alloc.ptr),
                "String".to_string(),
            ),
            // Vec allocations (multiples of common element sizes)
            s if s % 8 == 0 && s >= 16 => {
                let elements = s / 8;
                (
                    format!("vec_i64_{}elem_{:x}", elements, alloc.ptr),
                    "Vec<i64>".to_string(),
                )
            }
            s if s % 4 == 0 && s >= 8 => {
                let elements = s / 4;
                (
                    format!("vec_i32_{}elem_{:x}", elements, alloc.ptr),
                    "Vec<i32>".to_string(),
                )
            }
            // Box allocations (single element sizes)
            1 => (format!("box_u8_{:x}", alloc.ptr), "Box<u8>".to_string()),
            2 => (format!("box_u16_{:x}", alloc.ptr), "Box<u16>".to_string()),
            4 => (format!("box_u32_{:x}", alloc.ptr), "Box<u32>".to_string()),
            8 => (format!("box_u64_{:x}", alloc.ptr), "Box<u64>".to_string()),
            // HashMap/BTreeMap allocations (typically larger, irregular sizes)
            s if s >= 64 && s % 16 == 0 => (
                format!("hashmap_alloc_{:x}", alloc.ptr),
                "HashMap<K,V>".to_string(),
            ),
            // Large allocations (likely buffers or large collections)
            s if s >= 1024 => {
                let kb = s / 1024;
                (
                    format!("large_buffer_{}kb_{:x}", kb, alloc.ptr),
                    "LargeBuffer".to_string(),
                )
            }
            // Small system allocations
            s if s <= 16 => (
                format!("small_alloc_{}b_{:x}", s, alloc.ptr),
                "SmallAlloc".to_string(),
            ),
            // Default case with size hint
            _ => (
                format!("system_alloc_{}b_{:x}", size, alloc.ptr),
                "SystemAlloc".to_string(),
            ),
        };

        (var_name, type_name)
    }

    /// Generate comprehensive export data with clear separation of system vs user allocations
    pub fn generate_comprehensive_export(
        tracker: &crate::core::tracker::MemoryTracker,
    ) -> TrackingResult<serde_json::Value> {
        let start_time = std::time::Instant::now();
        tracing::info!(
            "🔄 Starting comprehensive export generation with allocation classification..."
        );

        // Get tracker data in parallel where possible
        let (active_allocations, other_data) = rayon::join(
            || tracker.get_active_allocations(),
            || {
                let history = tracker.get_allocation_history();
                let memory_types = tracker.get_memory_by_type();
                let stats = tracker.get_stats();
                let registry = Self::get_all_variables();
                (history, memory_types, stats, registry)
            },
        );

        let active_allocations = active_allocations?;
        let (allocation_history, memory_by_type, stats, registry) = {
            let allocation_history = other_data.0?;
            let memory_by_type = other_data.1?;
            let stats = other_data.2?;
            let registry = other_data.3;
            (allocation_history, memory_by_type, stats, registry)
        };

        tracing::info!(
            "📊 Data loaded: {} active, {} history, {} registry entries",
            active_allocations.len(),
            allocation_history.len(),
            registry.len()
        );

        // Filter out very small allocations to reduce processing overhead
        let filtered_active: Vec<_> = if active_allocations.len() > 10000 {
            active_allocations
                .into_iter()
                .filter(|alloc| alloc.size >= 8)
                .collect()
        } else {
            active_allocations
        };

        let filtered_history: Vec<_> = if allocation_history.len() > 50000 {
            allocation_history
                .into_iter()
                .filter(|alloc| alloc.size >= 8)
                .collect()
        } else {
            allocation_history
        };

        // Classify and enhance allocations in parallel
        let (classified_active, classified_history) = rayon::join(
            || Self::classify_and_enhance_allocations(&filtered_active, &registry),
            || Self::classify_and_enhance_allocations(&filtered_history, &registry),
        );

        // Separate user and system allocations
        let (user_active, system_active): (Vec<_>, Vec<_>) = classified_active
            .into_iter()
            .partition(|alloc| alloc["allocation_source"] == "user");

        let (user_history, system_history): (Vec<_>, Vec<_>) = classified_history
            .into_iter()
            .partition(|alloc| alloc["allocation_source"] == "user");

        // Group user variables by scope
        let user_scopes = Self::group_by_scope(&user_active, &user_history);

        // Build comprehensive result with clear separation
        let comprehensive_data = serde_json::json!({
            "memory_analysis": {
                "user_allocations": {
                    "active": user_active,
                    "history": user_history,
                    "by_scope": user_scopes,
                    "total_count": user_active.len() + user_history.len()
                },
                "system_allocations": {
                    "active": system_active,
                    "history": system_history,
                    "total_count": system_active.len() + system_history.len()
                },
                "memory_by_type": memory_by_type,
                "statistics": {
                    "overall": stats,
                    "user_vs_system": {
                        "user_active_count": user_active.len(),
                        "system_active_count": system_active.len(),
                        "user_total_size": user_active.iter()
                            .map(|a| a["size"].as_u64().unwrap_or(0))
                            .sum::<u64>(),
                        "system_total_size": system_active.iter()
                            .map(|a| a["size"].as_u64().unwrap_or(0))
                            .sum::<u64>()
                    },
                    "lifecycle_analysis": Self::analyze_lifecycle_statistics(&user_active, &user_history, &system_active, &system_history),
                    "deallocation_analysis": Self::analyze_deallocation_patterns(&user_active, &user_history, &system_active, &system_history)
                }
            },
            "variable_registry": {
                "total_variables": registry.len(),
                "user_variables": registry.values().collect::<Vec<_>>(),
                "scope_summary": Self::get_scope_summary(&registry)
            },
            "export_metadata": {
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "total_allocations": user_active.len() + user_history.len() + system_active.len() + system_history.len(),
                "processing_time_ms": start_time.elapsed().as_millis(),
                "classification_features": [
                    "user_vs_system_separation",
                    "scope_based_grouping",
                    "allocation_source_tracking",
                    "enhanced_type_inference"
                ]
            }
        });

        let total_time = start_time.elapsed();
        tracing::info!(
            "✅ Export completed in {:?} - User: {}, System: {}",
            total_time,
            user_active.len() + user_history.len(),
            system_active.len() + system_history.len()
        );

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
            let recent = registry
                .values()
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
