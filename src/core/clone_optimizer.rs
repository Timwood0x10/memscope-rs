//! Clone optimization system for reducing unnecessary clone() calls
//!
//! This module provides tools to analyze and optimize clone() calls throughout
//! the codebase by replacing them with Arc-based sharing where appropriate.

use serde::{Deserialize, Serialize};

/// Statistics about clone operations in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneStats {
    /// Total number of clone operations detected
    pub total_clones: u64,
    /// Number of clones that could be optimized with Arc
    pub optimizable_clones: u64,
    /// Number of clones already optimized
    pub optimized_clones: u64,
    /// Estimated memory saved by optimization (bytes)
    pub memory_saved_bytes: u64,
    /// Performance improvement ratio (0.0 to 1.0)
    pub performance_improvement: f64,
}

/// Information about a specific clone operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneInfo {
    /// Location where clone occurs (file:line)
    pub location: String,
    /// Type being cloned
    pub type_name: String,
    /// Estimated size of the clone
    pub estimated_size: usize,
    /// Whether this clone can be optimized
    pub can_optimize: bool,
    /// Reason why it can/cannot be optimized
    pub optimization_reason: String,
}

/// Clone optimization analyzer
pub struct CloneOptimizer {
    /// Statistics about clone operations
    stats: CloneStats,
    /// Information about individual clone operations
    clone_info: Vec<CloneInfo>,
}

impl CloneOptimizer {
    /// Create a new clone optimizer
    pub fn new() -> Self {
        Self {
            stats: CloneStats {
                total_clones: 0,
                optimizable_clones: 0,
                optimized_clones: 0,
                memory_saved_bytes: 0,
                performance_improvement: 0.0,
            },
            clone_info: Vec::new(),
        }
    }

    /// Record a clone operation
    pub fn record_clone(&mut self, location: &str, type_name: &str, size: usize) {
        self.stats.total_clones += 1;

        let can_optimize = self.can_optimize_type(type_name);
        if can_optimize {
            self.stats.optimizable_clones += 1;
        }

        let optimization_reason = if can_optimize {
            "Can be replaced with Arc sharing".to_string()
        } else {
            self.get_optimization_reason(type_name)
        };

        self.clone_info.push(CloneInfo {
            location: location.to_string(),
            type_name: type_name.to_string(),
            estimated_size: size,
            can_optimize,
            optimization_reason,
        });
    }

    /// Check if a type can be optimized with Arc sharing
    fn can_optimize_type(&self, type_name: &str) -> bool {
        // Types that benefit from Arc sharing
        matches!(type_name,
            "AllocationInfo" |
            "String" |
            "Vec<_>" |
            "HashMap<_,_>" |
            "BTreeMap<_,_>" |
            "ExportConfig" |
            "AnalysisResult" |
            _ if type_name.contains("Config") ||
                 type_name.contains("Result") ||
                 type_name.contains("Info") ||
                 type_name.contains("Data")
        )
    }

    /// Get reason why a type cannot be optimized
    fn get_optimization_reason(&self, type_name: &str) -> String {
        if type_name.contains("Mutex") || type_name.contains("RwLock") {
            "Already uses interior mutability".to_string()
        } else if type_name.contains("Arc") {
            "Already uses Arc sharing".to_string()
        } else if type_name.contains("Rc") {
            "Uses Rc, consider Arc for thread safety".to_string()
        } else if self.is_primitive_type(type_name) {
            "Primitive type, clone is cheap".to_string()
        } else {
            "Type analysis needed".to_string()
        }
    }

    /// Check if a type is primitive
    fn is_primitive_type(&self, type_name: &str) -> bool {
        matches!(
            type_name,
            "i8" | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "f32"
                | "f64"
                | "bool"
                | "char"
        )
    }

    /// Get current statistics
    pub fn get_stats(&self) -> &CloneStats {
        &self.stats
    }

    /// Get clone information
    pub fn get_clone_info(&self) -> &[CloneInfo] {
        &self.clone_info
    }

    /// Mark a clone as optimized
    pub fn mark_optimized(&mut self, location: &str) {
        if let Some(info) = self.clone_info.iter_mut().find(|i| i.location == location) {
            if info.can_optimize {
                self.stats.optimized_clones += 1;
                self.stats.memory_saved_bytes += info.estimated_size as u64;
                info.optimization_reason = "Optimized with Arc sharing".to_string();
            }
        }

        // Update performance improvement
        if self.stats.optimizable_clones > 0 {
            self.stats.performance_improvement =
                self.stats.optimized_clones as f64 / self.stats.optimizable_clones as f64;
        }
    }
}

impl Default for CloneOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a type should use Arc sharing based on common patterns
pub fn should_use_arc(type_name: &str) -> bool {
    type_name.contains("AllocationInfo")
        || type_name.contains("Config")
        || type_name.contains("Result")
        || type_name.contains("Collection")
        || type_name.len() > 50 // Large type names often indicate complex types
}
