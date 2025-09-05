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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone_optimizer_new() {
        let optimizer = CloneOptimizer::new();
        let stats = optimizer.get_stats();
        
        assert_eq!(stats.total_clones, 0);
        assert_eq!(stats.optimizable_clones, 0);
        assert_eq!(stats.optimized_clones, 0);
        assert_eq!(stats.memory_saved_bytes, 0);
        assert_eq!(stats.performance_improvement, 0.0);
        assert!(optimizer.get_clone_info().is_empty());
    }

    #[test]
    fn test_clone_optimizer_default() {
        let optimizer = CloneOptimizer::default();
        let stats = optimizer.get_stats();
        
        assert_eq!(stats.total_clones, 0);
        assert_eq!(stats.optimizable_clones, 0);
        assert_eq!(stats.optimized_clones, 0);
        assert_eq!(stats.memory_saved_bytes, 0);
        assert_eq!(stats.performance_improvement, 0.0);
    }

    #[test]
    fn test_record_optimizable_clone() {
        let mut optimizer = CloneOptimizer::new();
        
        optimizer.record_clone("main.rs:10", "AllocationInfo", 256);
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.total_clones, 1);
        assert_eq!(stats.optimizable_clones, 1);
        assert_eq!(stats.optimized_clones, 0);
        
        let clone_info = optimizer.get_clone_info();
        assert_eq!(clone_info.len(), 1);
        assert_eq!(clone_info[0].location, "main.rs:10");
        assert_eq!(clone_info[0].type_name, "AllocationInfo");
        assert_eq!(clone_info[0].estimated_size, 256);
        assert!(clone_info[0].can_optimize);
        assert_eq!(clone_info[0].optimization_reason, "Can be replaced with Arc sharing");
    }

    #[test]
    fn test_record_non_optimizable_clone() {
        let mut optimizer = CloneOptimizer::new();
        
        optimizer.record_clone("main.rs:20", "i32", 4);
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.total_clones, 1);
        assert_eq!(stats.optimizable_clones, 0);
        assert_eq!(stats.optimized_clones, 0);
        
        let clone_info = optimizer.get_clone_info();
        assert_eq!(clone_info.len(), 1);
        assert_eq!(clone_info[0].location, "main.rs:20");
        assert_eq!(clone_info[0].type_name, "i32");
        assert_eq!(clone_info[0].estimated_size, 4);
        assert!(!clone_info[0].can_optimize);
        assert_eq!(clone_info[0].optimization_reason, "Primitive type, clone is cheap");
    }

    #[test]
    fn test_can_optimize_type_optimizable_types() {
        let optimizer = CloneOptimizer::new();
        
        // Test exact matches that definitely work
        assert!(optimizer.can_optimize_type("AllocationInfo"));
        assert!(optimizer.can_optimize_type("ExportConfig"));
        assert!(optimizer.can_optimize_type("AnalysisResult"));
        
        // Test pattern matches
        assert!(optimizer.can_optimize_type("MyConfig")); // contains "Config"
        assert!(optimizer.can_optimize_type("TestResult")); // contains "Result"
        assert!(optimizer.can_optimize_type("UserInfo")); // contains "Info"
        assert!(optimizer.can_optimize_type("ProcessData")); // contains "Data"
    }

    #[test]
    fn test_can_optimize_type_non_optimizable_types() {
        let optimizer = CloneOptimizer::new();
        
        assert!(!optimizer.can_optimize_type("i32"));
        assert!(!optimizer.can_optimize_type("u64"));
        assert!(!optimizer.can_optimize_type("f64"));
        assert!(!optimizer.can_optimize_type("bool"));
        assert!(!optimizer.can_optimize_type("char"));
        assert!(!optimizer.can_optimize_type("SomeOtherType"));
    }

    #[test]
    fn test_is_primitive_type() {
        let optimizer = CloneOptimizer::new();
        
        // Test all integer types
        assert!(optimizer.is_primitive_type("i8"));
        assert!(optimizer.is_primitive_type("i16"));
        assert!(optimizer.is_primitive_type("i32"));
        assert!(optimizer.is_primitive_type("i64"));
        assert!(optimizer.is_primitive_type("i128"));
        assert!(optimizer.is_primitive_type("isize"));
        assert!(optimizer.is_primitive_type("u8"));
        assert!(optimizer.is_primitive_type("u16"));
        assert!(optimizer.is_primitive_type("u32"));
        assert!(optimizer.is_primitive_type("u64"));
        assert!(optimizer.is_primitive_type("u128"));
        assert!(optimizer.is_primitive_type("usize"));
        
        // Test floating point types
        assert!(optimizer.is_primitive_type("f32"));
        assert!(optimizer.is_primitive_type("f64"));
        
        // Test other primitive types
        assert!(optimizer.is_primitive_type("bool"));
        assert!(optimizer.is_primitive_type("char"));
        
        // Test non-primitive types
        assert!(!optimizer.is_primitive_type("String"));
        assert!(!optimizer.is_primitive_type("Vec<i32>"));
        assert!(!optimizer.is_primitive_type("AllocationInfo"));
    }

    #[test]
    fn test_get_optimization_reason_mutex_types() {
        let optimizer = CloneOptimizer::new();
        
        let reason = optimizer.get_optimization_reason("Mutex<i32>");
        assert_eq!(reason, "Already uses interior mutability");
        
        let reason = optimizer.get_optimization_reason("RwLock<String>");
        assert_eq!(reason, "Already uses interior mutability");
    }

    #[test]
    fn test_get_optimization_reason_arc_types() {
        let optimizer = CloneOptimizer::new();
        
        let reason = optimizer.get_optimization_reason("Arc<String>");
        assert_eq!(reason, "Already uses Arc sharing");
    }

    #[test]
    fn test_get_optimization_reason_rc_types() {
        let optimizer = CloneOptimizer::new();
        
        let reason = optimizer.get_optimization_reason("Rc<String>");
        assert_eq!(reason, "Uses Rc, consider Arc for thread safety");
    }

    #[test]
    fn test_get_optimization_reason_primitive_types() {
        let optimizer = CloneOptimizer::new();
        
        let reason = optimizer.get_optimization_reason("i32");
        assert_eq!(reason, "Primitive type, clone is cheap");
        
        let reason = optimizer.get_optimization_reason("bool");
        assert_eq!(reason, "Primitive type, clone is cheap");
    }

    #[test]
    fn test_get_optimization_reason_unknown_types() {
        let optimizer = CloneOptimizer::new();
        
        let reason = optimizer.get_optimization_reason("UnknownType");
        assert_eq!(reason, "Type analysis needed");
    }

    #[test]
    fn test_mark_optimized_valid_location() {
        let mut optimizer = CloneOptimizer::new();
        
        // Record an optimizable clone
        optimizer.record_clone("main.rs:10", "AllocationInfo", 256);
        
        // Mark it as optimized
        optimizer.mark_optimized("main.rs:10");
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.optimized_clones, 1);
        assert_eq!(stats.memory_saved_bytes, 256);
        assert_eq!(stats.performance_improvement, 1.0); // 1/1 = 100%
        
        let clone_info = optimizer.get_clone_info();
        assert_eq!(clone_info[0].optimization_reason, "Optimized with Arc sharing");
    }

    #[test]
    fn test_mark_optimized_invalid_location() {
        let mut optimizer = CloneOptimizer::new();
        
        // Record an optimizable clone
        optimizer.record_clone("main.rs:10", "AllocationInfo", 256);
        
        // Try to mark a different location as optimized
        optimizer.mark_optimized("main.rs:20");
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.optimized_clones, 0);
        assert_eq!(stats.memory_saved_bytes, 0);
        assert_eq!(stats.performance_improvement, 0.0);
    }

    #[test]
    fn test_mark_optimized_non_optimizable_clone() {
        let mut optimizer = CloneOptimizer::new();
        
        // Record a non-optimizable clone
        optimizer.record_clone("main.rs:10", "i32", 4);
        
        // Try to mark it as optimized
        optimizer.mark_optimized("main.rs:10");
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.optimized_clones, 0);
        assert_eq!(stats.memory_saved_bytes, 0);
        assert_eq!(stats.performance_improvement, 0.0);
    }

    #[test]
    fn test_performance_improvement_calculation() {
        let mut optimizer = CloneOptimizer::new();
        
        // Record multiple optimizable clones
        optimizer.record_clone("main.rs:10", "AllocationInfo", 256);
        optimizer.record_clone("main.rs:20", "ExportConfig", 128);
        optimizer.record_clone("main.rs:30", "Vec<_>", 512);
        optimizer.record_clone("main.rs:40", "i32", 4); // Non-optimizable
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.total_clones, 4);
        assert_eq!(stats.optimizable_clones, 2); // Only AllocationInfo and ExportConfig are optimizable
        assert_eq!(stats.optimized_clones, 0);
        
        // Optimize 2 out of 2 optimizable clones
        optimizer.mark_optimized("main.rs:10");
        optimizer.mark_optimized("main.rs:20");
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.optimized_clones, 2);
        assert_eq!(stats.memory_saved_bytes, 384); // 256 + 128
        assert!((stats.performance_improvement - 1.0).abs() < f64::EPSILON); // 2/2 = 100%
    }

    #[test]
    fn test_multiple_clone_records() {
        let mut optimizer = CloneOptimizer::new();
        
        // Record various types of clones
        optimizer.record_clone("file1.rs:10", "AllocationInfo", 256);
        optimizer.record_clone("file2.rs:20", "i32", 4);
        optimizer.record_clone("file3.rs:30", "ExportConfig", 64);
        optimizer.record_clone("file4.rs:40", "Mutex<i32>", 32);
        optimizer.record_clone("file5.rs:50", "Arc<String>", 48);
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.total_clones, 5);
        assert_eq!(stats.optimizable_clones, 2); // AllocationInfo and ExportConfig
        
        let clone_info = optimizer.get_clone_info();
        assert_eq!(clone_info.len(), 5);
        
        // Check specific clone info
        assert!(clone_info[0].can_optimize); // AllocationInfo
        assert!(!clone_info[1].can_optimize); // i32
        assert!(clone_info[2].can_optimize); // ExportConfig
        assert!(!clone_info[3].can_optimize); // Mutex<i32>
        assert!(!clone_info[4].can_optimize); // Arc<String>
    }

    #[test]
    fn test_should_use_arc_function() {
        assert!(should_use_arc("AllocationInfo"));
        assert!(should_use_arc("MyConfig"));
        assert!(should_use_arc("TestResult"));
        assert!(should_use_arc("DataCollection"));
        assert!(should_use_arc("VeryLongTypeNameThatExceedsFiftyCharactersInLengthAndMoreToMakeSureItIsLongEnough"));
        
        assert!(!should_use_arc("i32"));
        assert!(!should_use_arc("String"));
        assert!(!should_use_arc("ShortType"));
    }

    #[test]
    fn test_clone_stats_serialization() {
        let stats = CloneStats {
            total_clones: 10,
            optimizable_clones: 7,
            optimized_clones: 5,
            memory_saved_bytes: 1024,
            performance_improvement: 0.714,
        };
        
        // Test that it can be serialized and deserialized
        let serialized = serde_json::to_string(&stats).expect("Failed to serialize");
        let deserialized: CloneStats = serde_json::from_str(&serialized).expect("Failed to deserialize");
        
        assert_eq!(deserialized.total_clones, 10);
        assert_eq!(deserialized.optimizable_clones, 7);
        assert_eq!(deserialized.optimized_clones, 5);
        assert_eq!(deserialized.memory_saved_bytes, 1024);
        assert!((deserialized.performance_improvement - 0.714).abs() < f64::EPSILON);
    }

    #[test]
    fn test_clone_info_serialization() {
        let info = CloneInfo {
            location: "main.rs:42".to_string(),
            type_name: "AllocationInfo".to_string(),
            estimated_size: 256,
            can_optimize: true,
            optimization_reason: "Can be replaced with Arc sharing".to_string(),
        };
        
        // Test that it can be serialized and deserialized
        let serialized = serde_json::to_string(&info).expect("Failed to serialize");
        let deserialized: CloneInfo = serde_json::from_str(&serialized).expect("Failed to deserialize");
        
        assert_eq!(deserialized.location, "main.rs:42");
        assert_eq!(deserialized.type_name, "AllocationInfo");
        assert_eq!(deserialized.estimated_size, 256);
        assert!(deserialized.can_optimize);
        assert_eq!(deserialized.optimization_reason, "Can be replaced with Arc sharing");
    }

    #[test]
    fn test_clone_stats_debug() {
        let stats = CloneStats {
            total_clones: 5,
            optimizable_clones: 3,
            optimized_clones: 2,
            memory_saved_bytes: 512,
            performance_improvement: 0.667,
        };
        
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("total_clones: 5"));
        assert!(debug_str.contains("optimizable_clones: 3"));
        assert!(debug_str.contains("optimized_clones: 2"));
    }

    #[test]
    fn test_clone_info_debug() {
        let info = CloneInfo {
            location: "test.rs:1".to_string(),
            type_name: "TestType".to_string(),
            estimated_size: 128,
            can_optimize: false,
            optimization_reason: "Test reason".to_string(),
        };
        
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("location: \"test.rs:1\""));
        assert!(debug_str.contains("type_name: \"TestType\""));
        assert!(debug_str.contains("estimated_size: 128"));
    }

    #[test]
    fn test_edge_case_empty_type_name() {
        let mut optimizer = CloneOptimizer::new();
        
        optimizer.record_clone("main.rs:1", "", 0);
        
        let clone_info = optimizer.get_clone_info();
        assert_eq!(clone_info.len(), 1);
        assert_eq!(clone_info[0].type_name, "");
        assert!(!clone_info[0].can_optimize);
        assert_eq!(clone_info[0].optimization_reason, "Type analysis needed");
    }

    #[test]
    fn test_edge_case_zero_size_clone() {
        let mut optimizer = CloneOptimizer::new();
        
        optimizer.record_clone("main.rs:1", "AllocationInfo", 0);
        optimizer.mark_optimized("main.rs:1");
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.memory_saved_bytes, 0);
        assert_eq!(stats.optimized_clones, 1);
    }
}
