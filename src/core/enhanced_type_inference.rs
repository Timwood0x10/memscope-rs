//! Enhanced type inference engine
//!
//! This module provides improved type inference that goes beyond simple size-based
//! guessing to use context information, call stacks, and other heuristics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enhanced type inference engine
pub struct TypeInferenceEngine {
    /// Cache of type signatures to avoid repeated inference
    type_cache: HashMap<TypeSignature, InferredType>,
    /// Size-based hints as fallback
    size_hints: HashMap<usize, Vec<String>>,
    /// Call stack patterns for context-based inference
    call_stack_patterns: HashMap<String, Vec<String>>,
}

/// Signature used for caching type inference results
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TypeSignature {
    pub size: usize,
    pub call_stack_hash: u64,
    pub context_hints: Vec<String>,
}

/// Result of type inference with confidence level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredType {
    pub type_name: String,
    pub confidence: TypeConfidence,
    pub inference_method: InferenceMethod,
    pub alternative_types: Vec<String>,
}

/// Confidence level in type inference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TypeConfidence {
    /// Exact type known from compile-time information
    Exact,
    /// High confidence from context analysis
    High,
    /// Medium confidence from heuristics
    Medium,
    /// Low confidence, mostly guesswork
    Low,
    /// Unknown type, using fallback
    Unknown,
}

/// Method used for type inference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InferenceMethod {
    /// Compile-time type information available
    CompileTime,
    /// Inferred from call stack context
    CallStackContext,
    /// Inferred from allocation patterns
    AllocationPattern,
    /// Inferred from size and common patterns
    SizeHeuristic,
    /// Fallback to generic type
    Fallback,
}

/// Context information for type inference
#[derive(Debug, Clone)]
pub struct AllocationContext {
    pub size: usize,
    pub call_stack: Vec<String>,
    pub compile_time_type: Option<String>,
    pub variable_name: Option<String>,
    pub allocation_site: Option<String>,
    pub thread_context: Option<String>,
}

impl TypeInferenceEngine {
    /// Create a new type inference engine
    pub fn new() -> Self {
        let mut engine = Self {
            type_cache: HashMap::new(),
            size_hints: HashMap::new(),
            call_stack_patterns: HashMap::new(),
        };

        engine.initialize_default_patterns();
        engine
    }

    /// Initialize default patterns and hints
    fn initialize_default_patterns(&mut self) {
        // Initialize size-based hints
        self.size_hints.insert(
            1,
            vec!["u8".to_string(), "i8".to_string(), "bool".to_string()],
        );
        self.size_hints
            .insert(2, vec!["u16".to_string(), "i16".to_string()]);
        self.size_hints.insert(
            4,
            vec!["u32".to_string(), "i32".to_string(), "f32".to_string()],
        );
        self.size_hints.insert(
            8,
            vec![
                "u64".to_string(),
                "i64".to_string(),
                "f64".to_string(),
                "usize".to_string(),
                "isize".to_string(),
            ],
        );
        self.size_hints
            .insert(16, vec!["u128".to_string(), "i128".to_string()]);
        self.size_hints
            .insert(24, vec!["String".to_string(), "Vec<T>".to_string()]);
        self.size_hints.insert(
            48,
            vec!["HashMap<K,V>".to_string(), "BTreeMap<K,V>".to_string()],
        );

        // Initialize call stack patterns
        self.call_stack_patterns.insert(
            "vec".to_string(),
            vec!["Vec<T>".to_string(), "VecDeque<T>".to_string()],
        );
        self.call_stack_patterns.insert(
            "string".to_string(),
            vec!["String".to_string(), "CString".to_string()],
        );
        self.call_stack_patterns.insert(
            "hash".to_string(),
            vec!["HashMap<K,V>".to_string(), "HashSet<T>".to_string()],
        );
        self.call_stack_patterns.insert(
            "btree".to_string(),
            vec!["BTreeMap<K,V>".to_string(), "BTreeSet<T>".to_string()],
        );
        self.call_stack_patterns
            .insert("box".to_string(), vec!["Box<T>".to_string()]);
        self.call_stack_patterns.insert(
            "rc".to_string(),
            vec!["Rc<T>".to_string(), "Weak<T>".to_string()],
        );
        self.call_stack_patterns.insert(
            "arc".to_string(),
            vec!["Arc<T>".to_string(), "Weak<T>".to_string()],
        );
        self.call_stack_patterns
            .insert("mutex".to_string(), vec!["Mutex<T>".to_string()]);
        self.call_stack_patterns
            .insert("rwlock".to_string(), vec!["RwLock<T>".to_string()]);
        self.call_stack_patterns.insert(
            "channel".to_string(),
            vec!["Sender<T>".to_string(), "Receiver<T>".to_string()],
        );
    }

    /// Perform enhanced type inference
    pub fn infer_type(&mut self, context: &AllocationContext) -> InferredType {
        // Create signature for caching
        let signature = self.create_type_signature(context);

        // Check cache first
        if let Some(cached_type) = self.type_cache.get(&signature) {
            return cached_type.clone();
        }

        // Perform inference
        let inferred_type = self.perform_inference(context);

        // Cache the result
        self.type_cache.insert(signature, inferred_type.clone());

        inferred_type
    }

    /// Create a type signature for caching
    fn create_type_signature(&self, context: &AllocationContext) -> TypeSignature {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        context.call_stack.hash(&mut hasher);
        let call_stack_hash = hasher.finish();

        let mut context_hints = Vec::new();
        if let Some(ref var_name) = context.variable_name {
            context_hints.push(var_name.clone());
        }
        if let Some(ref alloc_site) = context.allocation_site {
            context_hints.push(alloc_site.clone());
        }

        TypeSignature {
            size: context.size,
            call_stack_hash,
            context_hints,
        }
    }

    /// Perform the actual type inference
    fn perform_inference(&self, context: &AllocationContext) -> InferredType {
        // 1. Highest priority: Compile-time type information
        if let Some(ref compile_time_type) = context.compile_time_type {
            return InferredType {
                type_name: compile_time_type.clone(),
                confidence: TypeConfidence::Exact,
                inference_method: InferenceMethod::CompileTime,
                alternative_types: Vec::new(),
            };
        }

        // 2. High priority: Call stack context analysis
        if let Some(inferred) = self.infer_from_call_stack(&context.call_stack) {
            return InferredType {
                type_name: inferred.type_name,
                confidence: TypeConfidence::High,
                inference_method: InferenceMethod::CallStackContext,
                alternative_types: inferred.alternative_types,
            };
        }

        // 3. Medium priority: Variable name analysis
        if let Some(inferred) = self.infer_from_variable_name(&context.variable_name) {
            return InferredType {
                type_name: inferred.type_name,
                confidence: TypeConfidence::Medium,
                inference_method: InferenceMethod::AllocationPattern,
                alternative_types: inferred.alternative_types,
            };
        }

        // 4. Low priority: Size-based heuristics
        if let Some(inferred) = self.infer_from_size(context.size) {
            return InferredType {
                type_name: inferred.type_name,
                confidence: TypeConfidence::Low,
                inference_method: InferenceMethod::SizeHeuristic,
                alternative_types: inferred.alternative_types,
            };
        }

        // 5. Fallback: Generic type based on size
        InferredType {
            type_name: format!("unknown_type_{}bytes", context.size),
            confidence: TypeConfidence::Unknown,
            inference_method: InferenceMethod::Fallback,
            alternative_types: Vec::new(),
        }
    }

    /// Infer type from call stack context
    fn infer_from_call_stack(&self, call_stack: &[String]) -> Option<InferredType> {
        for frame in call_stack {
            let frame_lower = frame.to_lowercase();

            // Look for patterns in the call stack
            for (pattern, types) in &self.call_stack_patterns {
                if frame_lower.contains(pattern) {
                    return Some(InferredType {
                        type_name: types[0].clone(),
                        confidence: TypeConfidence::High,
                        inference_method: InferenceMethod::CallStackContext,
                        alternative_types: types[1..].to_vec(),
                    });
                }
            }

            // Look for specific function patterns
            if frame_lower.contains("alloc") && frame_lower.contains("vec") {
                return Some(InferredType {
                    type_name: "Vec<T>".to_string(),
                    confidence: TypeConfidence::High,
                    inference_method: InferenceMethod::CallStackContext,
                    alternative_types: vec!["VecDeque<T>".to_string()],
                });
            }

            if frame_lower.contains("string")
                && (frame_lower.contains("new") || frame_lower.contains("from"))
            {
                return Some(InferredType {
                    type_name: "String".to_string(),
                    confidence: TypeConfidence::High,
                    inference_method: InferenceMethod::CallStackContext,
                    alternative_types: vec!["CString".to_string()],
                });
            }

            if frame_lower.contains("hashmap")
                || (frame_lower.contains("hash") && frame_lower.contains("insert"))
            {
                return Some(InferredType {
                    type_name: "HashMap<K,V>".to_string(),
                    confidence: TypeConfidence::High,
                    inference_method: InferenceMethod::CallStackContext,
                    alternative_types: vec!["HashSet<T>".to_string()],
                });
            }
        }

        None
    }

    /// Infer type from variable name patterns
    fn infer_from_variable_name(&self, var_name: &Option<String>) -> Option<InferredType> {
        let var_name = var_name.as_ref()?;
        let name_lower = var_name.to_lowercase();

        // Common variable naming patterns
        if name_lower.contains("vec") || name_lower.contains("list") || name_lower.contains("array")
        {
            return Some(InferredType {
                type_name: "Vec<T>".to_string(),
                confidence: TypeConfidence::Medium,
                inference_method: InferenceMethod::AllocationPattern,
                alternative_types: vec!["VecDeque<T>".to_string(), "LinkedList<T>".to_string()],
            });
        }

        if name_lower.contains("str")
            || name_lower.contains("text")
            || name_lower.contains("msg")
            || name_lower.contains("message")
        {
            return Some(InferredType {
                type_name: "String".to_string(),
                confidence: TypeConfidence::Medium,
                inference_method: InferenceMethod::AllocationPattern,
                alternative_types: vec!["&str".to_string(), "CString".to_string()],
            });
        }

        if name_lower.contains("map") || name_lower.contains("dict") || name_lower.contains("table")
        {
            return Some(InferredType {
                type_name: "HashMap<K,V>".to_string(),
                confidence: TypeConfidence::Medium,
                inference_method: InferenceMethod::AllocationPattern,
                alternative_types: vec!["BTreeMap<K,V>".to_string()],
            });
        }

        if name_lower.contains("set") || name_lower.contains("collection") {
            return Some(InferredType {
                type_name: "HashSet<T>".to_string(),
                confidence: TypeConfidence::Medium,
                inference_method: InferenceMethod::AllocationPattern,
                alternative_types: vec!["BTreeSet<T>".to_string()],
            });
        }

        if name_lower.contains("box") || name_lower.contains("ptr") {
            return Some(InferredType {
                type_name: "Box<T>".to_string(),
                confidence: TypeConfidence::Medium,
                inference_method: InferenceMethod::AllocationPattern,
                alternative_types: vec!["*mut T".to_string(), "*const T".to_string()],
            });
        }

        None
    }

    /// Infer type from size with improved heuristics
    fn infer_from_size(&self, size: usize) -> Option<InferredType> {
        if let Some(types) = self.size_hints.get(&size) {
            return Some(InferredType {
                type_name: types[0].clone(),
                confidence: TypeConfidence::Low,
                inference_method: InferenceMethod::SizeHeuristic,
                alternative_types: types[1..].to_vec(),
            });
        }

        // Handle common size ranges
        match size {
            0 => Some(InferredType {
                type_name: "()".to_string(),
                confidence: TypeConfidence::Medium,
                inference_method: InferenceMethod::SizeHeuristic,
                alternative_types: vec!["ZST".to_string()],
            }),
            1..=16 => Some(InferredType {
                type_name: "primitive_type".to_string(),
                confidence: TypeConfidence::Low,
                inference_method: InferenceMethod::SizeHeuristic,
                alternative_types: vec!["small_struct".to_string()],
            }),
            17..=64 => Some(InferredType {
                type_name: "small_struct".to_string(),
                confidence: TypeConfidence::Low,
                inference_method: InferenceMethod::SizeHeuristic,
                alternative_types: vec!["array".to_string(), "tuple".to_string()],
            }),
            65..=256 => Some(InferredType {
                type_name: "medium_struct".to_string(),
                confidence: TypeConfidence::Low,
                inference_method: InferenceMethod::SizeHeuristic,
                alternative_types: vec!["large_array".to_string()],
            }),
            _ => Some(InferredType {
                type_name: format!("large_type_{size}bytes"),
                confidence: TypeConfidence::Low,
                inference_method: InferenceMethod::SizeHeuristic,
                alternative_types: vec!["buffer".to_string(), "large_struct".to_string()],
            }),
        }
    }

    /// Get inference statistics
    pub fn get_statistics(&self) -> InferenceStatistics {
        let mut method_counts = HashMap::new();
        let mut confidence_counts = HashMap::new();

        for inferred_type in self.type_cache.values() {
            *method_counts
                .entry(inferred_type.inference_method.clone())
                .or_insert(0) += 1;
            *confidence_counts
                .entry(inferred_type.confidence.clone())
                .or_insert(0) += 1;
        }

        InferenceStatistics {
            total_inferences: self.type_cache.len(),
            cache_size: self.type_cache.len(),
            method_distribution: method_counts,
            confidence_distribution: confidence_counts,
        }
    }

    /// Clear the inference cache
    pub fn clear_cache(&mut self) {
        self.type_cache.clear();
    }

    /// Add a custom pattern for call stack inference
    pub fn add_call_stack_pattern(&mut self, pattern: String, types: Vec<String>) {
        self.call_stack_patterns.insert(pattern, types);
    }

    /// Add a custom size hint
    pub fn add_size_hint(&mut self, size: usize, types: Vec<String>) {
        self.size_hints.insert(size, types);
    }
}

impl Default for TypeInferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about type inference performance
#[derive(Debug, Clone, Serialize)]
pub struct InferenceStatistics {
    pub total_inferences: usize,
    pub cache_size: usize,
    pub method_distribution: HashMap<InferenceMethod, usize>,
    pub confidence_distribution: HashMap<TypeConfidence, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_time_type_inference() {
        let mut engine = TypeInferenceEngine::new();
        let context = AllocationContext {
            size: 24,
            call_stack: vec!["main".to_string()],
            compile_time_type: Some("Vec<i32>".to_string()),
            variable_name: None,
            allocation_site: None,
            thread_context: None,
        };

        let result = engine.infer_type(&context);
        assert_eq!(result.type_name, "Vec<i32>");
        assert_eq!(result.confidence, TypeConfidence::Exact);
        assert_eq!(result.inference_method, InferenceMethod::CompileTime);
    }

    #[test]
    fn test_call_stack_inference() {
        let mut engine = TypeInferenceEngine::new();
        let context = AllocationContext {
            size: 24,
            call_stack: vec!["vec::Vec::new".to_string(), "main".to_string()],
            compile_time_type: None,
            variable_name: None,
            allocation_site: None,
            thread_context: None,
        };

        let result = engine.infer_type(&context);
        assert_eq!(result.type_name, "Vec<T>");
        assert_eq!(result.confidence, TypeConfidence::High);
        assert_eq!(result.inference_method, InferenceMethod::CallStackContext);
    }

    #[test]
    fn test_variable_name_inference() {
        let mut engine = TypeInferenceEngine::new();
        let context = AllocationContext {
            size: 24,
            call_stack: vec!["main".to_string()],
            compile_time_type: None,
            variable_name: Some("my_vec".to_string()),
            allocation_site: None,
            thread_context: None,
        };

        let result = engine.infer_type(&context);
        assert_eq!(result.type_name, "Vec<T>");
        assert_eq!(result.confidence, TypeConfidence::Medium);
        assert_eq!(result.inference_method, InferenceMethod::AllocationPattern);
    }

    #[test]
    fn test_size_based_inference() {
        let mut engine = TypeInferenceEngine::new();
        let context = AllocationContext {
            size: 8,
            call_stack: vec!["main".to_string()],
            compile_time_type: None,
            variable_name: None,
            allocation_site: None,
            thread_context: None,
        };

        let result = engine.infer_type(&context);
        assert_eq!(result.type_name, "u64");
        assert_eq!(result.confidence, TypeConfidence::Low);
        assert_eq!(result.inference_method, InferenceMethod::SizeHeuristic);
    }

    #[test]
    fn test_caching() {
        let mut engine = TypeInferenceEngine::new();
        let context = AllocationContext {
            size: 24,
            call_stack: vec!["main".to_string()],
            compile_time_type: Some("Vec<i32>".to_string()),
            variable_name: None,
            allocation_site: None,
            thread_context: None,
        };

        // First inference
        let result1 = engine.infer_type(&context);

        // Second inference should use cache
        let result2 = engine.infer_type(&context);

        assert_eq!(result1.type_name, result2.type_name);
        assert_eq!(engine.type_cache.len(), 1);
    }

    #[test]
    fn test_custom_patterns() {
        let mut engine = TypeInferenceEngine::new();

        // Add custom pattern
        engine.add_call_stack_pattern("custom".to_string(), vec!["CustomType".to_string()]);

        let context = AllocationContext {
            size: 100,
            call_stack: vec!["custom::function".to_string()],
            compile_time_type: None,
            variable_name: None,
            allocation_site: None,
            thread_context: None,
        };

        let result = engine.infer_type(&context);
        assert_eq!(result.type_name, "CustomType");
        assert_eq!(result.confidence, TypeConfidence::High);
    }
}
