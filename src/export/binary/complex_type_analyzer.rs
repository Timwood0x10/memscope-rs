//! Complex type analysis for binary to HTML conversion
//!
//! This module provides comprehensive analysis of type complexity based on allocation data,
//! categorizing types and calculating complexity scores for dashboard visualization.

use crate::core::types::AllocationInfo;
use crate::export::binary::error::BinaryExportError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complex type analysis results for dashboard display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexTypeAnalysis {
    /// Summary statistics for complex types
    pub summary: ComplexTypeSummary,
    /// Types categorized by complexity
    pub categorized_types: CategorizedTypes,
    /// Complexity scores for each type (1-10 scale)
    pub complexity_scores: HashMap<String, u32>,
    /// Generic type analysis
    pub generic_analysis: GenericTypeAnalysis,
}

/// Summary statistics for complex type analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexTypeSummary {
    /// Total number of unique types analyzed
    pub total_types: usize,
    /// Number of primitive types
    pub primitive_count: usize,
    /// Number of collection types
    pub collection_count: usize,
    /// Number of smart pointer types
    pub smart_pointer_count: usize,
    /// Number of generic types
    pub generic_count: usize,
    /// Average complexity score across all types
    pub average_complexity: f64,
    /// Highest complexity score found
    pub max_complexity: u32,
    /// Most complex type name
    pub most_complex_type: Option<String>,
}

/// Types categorized by their complexity characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorizedTypes {
    /// Primitive types (i32, f64, bool, etc.)
    pub primitives: Vec<TypeInfo>,
    /// Collection types (Vec, HashMap, etc.)
    pub collections: Vec<TypeInfo>,
    /// Smart pointer types (Box, Rc, Arc, etc.)
    pub smart_pointers: Vec<TypeInfo>,
    /// Generic types with type parameters
    pub generics: Vec<TypeInfo>,
    /// Custom user-defined types
    pub custom_types: Vec<TypeInfo>,
}

/// Information about a specific type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    /// Name of the type
    pub name: String,
    /// Complexity score (1-10)
    pub complexity_score: u32,
    /// Number of allocations of this type
    pub allocation_count: usize,
    /// Total memory used by this type
    pub total_memory: usize,
    /// Average allocation size for this type
    pub average_size: f64,
    /// Type category
    pub category: TypeCategory,
    /// Generic type parameters if applicable
    pub generic_parameters: Vec<String>,
    /// Memory efficiency score
    pub memory_efficiency: f64,
}

/// Generic type analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericTypeAnalysis {
    /// Total number of generic instantiations
    pub total_instantiations: usize,
    /// Most frequently instantiated generic types
    pub frequent_generics: Vec<GenericInstantiation>,
    /// Generic type complexity distribution
    pub complexity_distribution: HashMap<String, u32>,
    /// Memory usage by generic types
    pub memory_usage_by_generic: HashMap<String, usize>,
}

/// Information about a generic type instantiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericInstantiation {
    /// Base generic type name (e.g., "Vec")
    pub base_type: String,
    /// Full instantiated type name (e.g., "Vec<String>")
    pub full_type: String,
    /// Type parameters
    pub type_parameters: Vec<String>,
    /// Number of instances
    pub instance_count: usize,
    /// Total memory usage
    pub total_memory: usize,
    /// Complexity score
    pub complexity_score: u32,
}

/// Type category enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TypeCategory {
    Primitive,
    Collection,
    SmartPointer,
    Generic,
    Custom,
    Unknown,
}

/// Complex type analyzer for processing allocation data
pub struct ComplexTypeAnalyzer {
    /// Type statistics accumulator
    type_stats: HashMap<String, TypeStats>,
    /// Generic type tracker
    generic_tracker: HashMap<String, GenericStats>,
}

/// Internal statistics for a type
#[derive(Debug, Clone)]
struct TypeStats {
    allocation_count: usize,
    total_memory: usize,
    sizes: Vec<usize>,
    category: TypeCategory,
    generic_parameters: Vec<String>,
}

/// Internal statistics for generic types
#[derive(Debug, Clone)]
struct GenericStats {
    base_type: String,
    instantiations: HashMap<String, usize>,
    total_memory: usize,
    complexity_scores: Vec<u32>,
}

impl ComplexTypeAnalyzer {
    /// Create a new complex type analyzer
    pub fn new() -> Self {
        Self {
            type_stats: HashMap::new(),
            generic_tracker: HashMap::new(),
        }
    }

    /// Analyze complex types from allocation data
    pub fn analyze_allocations(
        allocations: &[AllocationInfo],
    ) -> Result<ComplexTypeAnalysis, BinaryExportError> {
        let mut analyzer = Self::new();

        // Process each allocation
        for allocation in allocations {
            analyzer.process_allocation(allocation)?;
        }

        // Generate analysis results
        analyzer.generate_analysis()
    }

    /// Process a single allocation for type analysis
    fn process_allocation(&mut self, allocation: &AllocationInfo) -> Result<(), BinaryExportError> {
        let type_name = allocation
            .type_name
            .as_ref()
            .unwrap_or(&"Unknown".to_string())
            .clone();

        // Skip empty or invalid type names
        if type_name.is_empty() || type_name == "Unknown" {
            return Ok(());
        }

        // Analyze type characteristics
        let category = self.categorize_type(&type_name);
        let generic_params = self.extract_generic_parameters(&type_name);

        // Update type statistics
        let stats = self.type_stats.entry(type_name.clone()).or_insert(TypeStats {
            allocation_count: 0,
            total_memory: 0,
            sizes: Vec::new(),
            category: category.clone(),
            generic_parameters: generic_params.clone(),
        });

        stats.allocation_count += 1;
        stats.total_memory += allocation.size;
        stats.sizes.push(allocation.size);

        // Track generic types separately
        if category == TypeCategory::Generic {
            self.track_generic_type(&type_name, allocation.size)?;
        }

        Ok(())
    }

    /// Categorize a type based on its name and characteristics
    fn categorize_type(&self, type_name: &str) -> TypeCategory {
        // Primitive types
        if self.is_primitive_type(type_name) {
            return TypeCategory::Primitive;
        }

        // Smart pointer types
        if self.is_smart_pointer_type(type_name) {
            return TypeCategory::SmartPointer;
        }

        // Collection types
        if self.is_collection_type(type_name) {
            return TypeCategory::Collection;
        }

        // Generic types (contain angle brackets)
        if type_name.contains('<') && type_name.contains('>') {
            return TypeCategory::Generic;
        }

        // Custom types (everything else)
        TypeCategory::Custom
    }

    /// Check if a type is a primitive type
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
                | "()"
                | "str"
        )
    }

    /// Check if a type is a smart pointer type
    fn is_smart_pointer_type(&self, type_name: &str) -> bool {
        type_name.starts_with("Box<")
            || type_name.starts_with("Rc<")
            || type_name.starts_with("Arc<")
            || type_name.starts_with("Weak<")
            || type_name.starts_with("RefCell<")
            || type_name.starts_with("Mutex<")
            || type_name.starts_with("RwLock<")
    }

    /// Check if a type is a collection type
    fn is_collection_type(&self, type_name: &str) -> bool {
        type_name.starts_with("Vec<")
            || type_name.starts_with("HashMap<")
            || type_name.starts_with("HashSet<")
            || type_name.starts_with("BTreeMap<")
            || type_name.starts_with("BTreeSet<")
            || type_name.starts_with("VecDeque<")
            || type_name.starts_with("LinkedList<")
            || type_name.starts_with("BinaryHeap<")
            || type_name.starts_with("String")
            || type_name == "&str"
            || type_name.starts_with("&[")
            || type_name.starts_with("[")
    }

    /// Extract generic type parameters from a type name
    fn extract_generic_parameters(&self, type_name: &str) -> Vec<String> {
        if let Some(start) = type_name.find('<') {
            if let Some(end) = type_name.rfind('>') {
                let params_str = &type_name[start + 1..end];
                return self.parse_generic_parameters(params_str);
            }
        }
        Vec::new()
    }

    /// Parse generic parameters from a parameter string
    fn parse_generic_parameters(&self, params_str: &str) -> Vec<String> {
        let mut parameters = Vec::new();
        let mut current_param = String::new();
        let mut bracket_depth = 0;

        for ch in params_str.chars() {
            match ch {
                '<' => {
                    bracket_depth += 1;
                    current_param.push(ch);
                }
                '>' => {
                    bracket_depth -= 1;
                    current_param.push(ch);
                }
                ',' if bracket_depth == 0 => {
                    if !current_param.trim().is_empty() {
                        parameters.push(current_param.trim().to_string());
                    }
                    current_param.clear();
                }
                _ => {
                    current_param.push(ch);
                }
            }
        }

        if !current_param.trim().is_empty() {
            parameters.push(current_param.trim().to_string());
        }

        parameters
    }

    /// Track generic type instantiations
    fn track_generic_type(&mut self, type_name: &str, size: usize) -> Result<(), BinaryExportError> {
        let base_type = self.extract_base_generic_type(type_name);
        let complexity = self.calculate_type_complexity(type_name);

        let stats = self
            .generic_tracker
            .entry(base_type.clone())
            .or_insert(GenericStats {
                base_type: base_type.clone(),
                instantiations: HashMap::new(),
                total_memory: 0,
                complexity_scores: Vec::new(),
            });

        *stats.instantiations.entry(type_name.to_string()).or_insert(0) += 1;
        stats.total_memory += size;
        stats.complexity_scores.push(complexity);

        Ok(())
    }

    /// Extract base type from generic type (e.g., "Vec" from "Vec<String>")
    fn extract_base_generic_type(&self, type_name: &str) -> String {
        if let Some(pos) = type_name.find('<') {
            type_name[..pos].to_string()
        } else {
            type_name.to_string()
        }
    }

    /// Calculate complexity score for a type (1-10 scale)
    fn calculate_type_complexity(&self, type_name: &str) -> u32 {
        // Base complexity by category
        let mut complexity = match self.categorize_type(type_name) {
            TypeCategory::Primitive => 1,
            TypeCategory::Collection => 4,
            TypeCategory::SmartPointer => 6,
            TypeCategory::Generic => 5,
            TypeCategory::Custom => 3,
            TypeCategory::Unknown => 2,
        };

        // Increase complexity for nested generics (only for depth > 1)
        let generic_depth = self.calculate_generic_depth(type_name);
        if generic_depth > 1 {
            complexity += (generic_depth - 1) * 2;
        }

        // Increase complexity for multiple type parameters
        let param_count = self.extract_generic_parameters(type_name).len() as u32;
        complexity += param_count.saturating_sub(1);

        // Special cases for highly complex types
        if type_name.contains("HashMap") || type_name.contains("BTreeMap") {
            complexity += 1;
        }

        if type_name.contains("Mutex") || type_name.contains("RwLock") {
            complexity += 2;
        }

        if type_name.contains("Arc") && type_name.contains("Mutex") {
            complexity += 1; // Arc<Mutex<T>> is particularly complex
        }

        // Cap at maximum complexity of 10
        complexity.min(10)
    }

    /// Calculate the depth of nested generic types
    fn calculate_generic_depth(&self, type_name: &str) -> u32 {
        let mut depth = 0u32;
        let mut max_depth = 0u32;

        for ch in type_name.chars() {
            match ch {
                '<' => {
                    depth += 1;
                    max_depth = max_depth.max(depth);
                }
                '>' => {
                    depth = depth.saturating_sub(1);
                }
                _ => {}
            }
        }

        max_depth
    }

    /// Generate the final analysis results
    fn generate_analysis(&self) -> Result<ComplexTypeAnalysis, BinaryExportError> {
        let mut categorized_types = CategorizedTypes {
            primitives: Vec::new(),
            collections: Vec::new(),
            smart_pointers: Vec::new(),
            generics: Vec::new(),
            custom_types: Vec::new(),
        };

        let mut complexity_scores = HashMap::new();
        let mut total_complexity = 0u32;
        let mut max_complexity = 0u32;
        let mut most_complex_type = None;

        // Process each type
        for (type_name, stats) in &self.type_stats {
            let complexity = self.calculate_type_complexity(type_name);
            complexity_scores.insert(type_name.clone(), complexity);

            total_complexity += complexity;
            if complexity > max_complexity {
                max_complexity = complexity;
                most_complex_type = Some(type_name.clone());
            }

            let average_size = if stats.allocation_count > 0 {
                stats.total_memory as f64 / stats.allocation_count as f64
            } else {
                0.0
            };

            let memory_efficiency = if stats.total_memory > 0 {
                (stats.allocation_count as f64) / (stats.total_memory as f64 / 1024.0)
            } else {
                0.0
            };

            let type_info = TypeInfo {
                name: type_name.clone(),
                complexity_score: complexity,
                allocation_count: stats.allocation_count,
                total_memory: stats.total_memory,
                average_size,
                category: stats.category.clone(),
                generic_parameters: stats.generic_parameters.clone(),
                memory_efficiency,
            };

            // Categorize the type
            match stats.category {
                TypeCategory::Primitive => categorized_types.primitives.push(type_info),
                TypeCategory::Collection => categorized_types.collections.push(type_info),
                TypeCategory::SmartPointer => categorized_types.smart_pointers.push(type_info),
                TypeCategory::Generic => categorized_types.generics.push(type_info),
                TypeCategory::Custom => categorized_types.custom_types.push(type_info),
                TypeCategory::Unknown => categorized_types.custom_types.push(type_info),
            }
        }

        // Sort each category by complexity (descending)
        categorized_types
            .primitives
            .sort_by(|a, b| b.complexity_score.cmp(&a.complexity_score));
        categorized_types
            .collections
            .sort_by(|a, b| b.complexity_score.cmp(&a.complexity_score));
        categorized_types
            .smart_pointers
            .sort_by(|a, b| b.complexity_score.cmp(&a.complexity_score));
        categorized_types
            .generics
            .sort_by(|a, b| b.complexity_score.cmp(&a.complexity_score));
        categorized_types
            .custom_types
            .sort_by(|a, b| b.complexity_score.cmp(&a.complexity_score));

        // Generate summary
        let total_types = self.type_stats.len();
        let average_complexity = if total_types > 0 {
            total_complexity as f64 / total_types as f64
        } else {
            0.0
        };

        let summary = ComplexTypeSummary {
            total_types,
            primitive_count: categorized_types.primitives.len(),
            collection_count: categorized_types.collections.len(),
            smart_pointer_count: categorized_types.smart_pointers.len(),
            generic_count: categorized_types.generics.len(),
            average_complexity,
            max_complexity,
            most_complex_type,
        };

        // Generate generic analysis
        let generic_analysis = self.generate_generic_analysis()?;

        Ok(ComplexTypeAnalysis {
            summary,
            categorized_types,
            complexity_scores,
            generic_analysis,
        })
    }

    /// Generate generic type analysis
    fn generate_generic_analysis(&self) -> Result<GenericTypeAnalysis, BinaryExportError> {
        let mut frequent_generics = Vec::new();
        let mut complexity_distribution = HashMap::new();
        let mut memory_usage_by_generic = HashMap::new();
        let mut total_instantiations = 0;

        for (base_type, stats) in &self.generic_tracker {
            total_instantiations += stats.instantiations.len();

            // Calculate average complexity for this generic type
            let avg_complexity = if !stats.complexity_scores.is_empty() {
                stats.complexity_scores.iter().sum::<u32>() / stats.complexity_scores.len() as u32
            } else {
                0
            };

            complexity_distribution.insert(base_type.clone(), avg_complexity);
            memory_usage_by_generic.insert(base_type.clone(), stats.total_memory);

            // Create instantiation info for frequent generics
            for (full_type, count) in &stats.instantiations {
                let type_params = self.extract_generic_parameters(full_type);
                let complexity = self.calculate_type_complexity(full_type);

                frequent_generics.push(GenericInstantiation {
                    base_type: stats.base_type.clone(), // Use the base_type field from stats
                    full_type: full_type.clone(),
                    type_parameters: type_params,
                    instance_count: *count,
                    total_memory: stats.total_memory / stats.instantiations.len(), // Approximate
                    complexity_score: complexity,
                });
            }
        }

        // Sort by instance count (descending)
        frequent_generics.sort_by(|a, b| b.instance_count.cmp(&a.instance_count));

        // Keep only top 20 most frequent
        frequent_generics.truncate(20);

        Ok(GenericTypeAnalysis {
            total_instantiations,
            frequent_generics,
            complexity_distribution,
            memory_usage_by_generic,
        })
    }
}

impl Default for ComplexTypeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_allocation(type_name: &str, size: usize) -> AllocationInfo {
        AllocationInfo {
            ptr: 0x1000,
            size,
            type_name: Some(type_name.to_string()),
            var_name: Some("test_var".to_string()),
            scope_name: Some("test_scope".to_string()),
            timestamp_alloc: 1000,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        }
    }

    #[test]
    fn test_type_categorization() {
        let analyzer = ComplexTypeAnalyzer::new();

        assert_eq!(analyzer.categorize_type("i32"), TypeCategory::Primitive);
        assert_eq!(analyzer.categorize_type("Vec<String>"), TypeCategory::Collection);
        assert_eq!(analyzer.categorize_type("Box<i32>"), TypeCategory::SmartPointer);
        assert_eq!(analyzer.categorize_type("Option<T>"), TypeCategory::Generic);
        assert_eq!(analyzer.categorize_type("MyStruct"), TypeCategory::Custom);
    }

    #[test]
    fn test_complexity_calculation() {
        let analyzer = ComplexTypeAnalyzer::new();

        assert_eq!(analyzer.calculate_type_complexity("i32"), 1);
        assert_eq!(analyzer.calculate_type_complexity("Vec<String>"), 4);
        assert_eq!(analyzer.calculate_type_complexity("Arc<Mutex<HashMap<String, Vec<i32>>>>"), 10);
    }

    #[test]
    fn test_generic_parameter_extraction() {
        let analyzer = ComplexTypeAnalyzer::new();

        let params = analyzer.extract_generic_parameters("Vec<String>");
        assert_eq!(params, vec!["String"]);

        let params = analyzer.extract_generic_parameters("HashMap<String, Vec<i32>>");
        assert_eq!(params, vec!["String", "Vec<i32>"]);

        let params = analyzer.extract_generic_parameters("i32");
        assert!(params.is_empty());
    }

    #[test]
    fn test_analysis_generation() {
        let allocations = vec![
            create_test_allocation("i32", 4),
            create_test_allocation("Vec<String>", 24),
            create_test_allocation("Box<i32>", 8),
            create_test_allocation("HashMap<String, i32>", 48),
        ];

        let analysis = ComplexTypeAnalyzer::analyze_allocations(&allocations).expect("Failed to get test value");

        assert_eq!(analysis.summary.total_types, 4);
        assert_eq!(analysis.summary.primitive_count, 1);
        assert_eq!(analysis.summary.collection_count, 2);
        assert_eq!(analysis.summary.smart_pointer_count, 1);
        assert!(analysis.summary.average_complexity > 0.0);
    }

    #[test]
    fn test_generic_depth_calculation() {
        let analyzer = ComplexTypeAnalyzer::new();

        assert_eq!(analyzer.calculate_generic_depth("i32"), 0);
        assert_eq!(analyzer.calculate_generic_depth("Vec<String>"), 1);
        assert_eq!(analyzer.calculate_generic_depth("Vec<Vec<String>>"), 2);
        assert_eq!(analyzer.calculate_generic_depth("Arc<Mutex<Vec<String>>>"), 3);
    }
}