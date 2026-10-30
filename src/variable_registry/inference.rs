//! Size-based type inference for system allocations.
//!
//! Split out of `variable_registry.rs` to keep that file within the project line
//! budget. The results are heuristics: they are labelled as inferred data and must
//! never be presented as observed types.

use super::*;

impl VariableRegistry {
    /// Smart inference with caching for better performance
    pub fn infer_allocation_info_cached(
        alloc: &crate::capture::types::allocation::AllocationInfo,
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
                    format!("{var_prefix}_{:x}", alloc.ptr),
                    type_name.to_string(),
                );
            }
        }

        // Fallback to original logic for uncommon sizes
        Self::infer_allocation_info(alloc)
    }

    /// Smart inference for system allocations based on size patterns and common allocations
    pub fn infer_allocation_info(
        alloc: &crate::capture::types::allocation::AllocationInfo,
    ) -> (String, String) {
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
                    format!("vec_i64_{elements}elem_{:x}", alloc.ptr),
                    "Vec<i64>".to_string(),
                )
            }
            s if s % 4 == 0 && s >= 8 => {
                let elements = s / 4;
                (
                    format!("vec_i32_{elements}elem_{:x}", alloc.ptr),
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
                format!("small_alloc_{s}b_{:x}", alloc.ptr),
                "SmallAlloc".to_string(),
            ),
            // Default case with size hint
            _ => (
                format!("system_alloc_{size}b_{:x}", alloc.ptr),
                "SystemAlloc".to_string(),
            ),
        };

        (var_name, type_name)
    }
}
