//! Generic type analysis for Rust types
//!
//! This module implements generic type analysis features from ComplexTypeForRust.md:
//! - Generic parameter tracking
//! - Generic constraint analysis  
//! - Generic instantiation tracking


use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Global generic analyzer instance
static GLOBAL_GENERIC_ANALYZER: OnceLock<Arc<GenericAnalyzer>> = OnceLock::new();

/// Get the global generic analyzer instance
pub fn get_global_generic_analyzer() -> Arc<GenericAnalyzer> {
    GLOBAL_GENERIC_ANALYZER
        .get_or_init(|| Arc::new(GenericAnalyzer::new()))
        .clone()
}

/// Generic type analysis system
pub struct GenericAnalyzer {
    /// Generic parameter tracking
    generic_instances: Mutex<HashMap<String, Vec<GenericInstance>>>,
    /// Generic constraint analysis
    constraint_violations: Mutex<Vec<ConstraintViolation>>,
    /// Generic instantiation events
    instantiation_events: Mutex<Vec<InstantiationEvent>>,
}

impl GenericAnalyzer {
    /// Create a new generic analyzer
    pub fn new() -> Self {
        Self {
            generic_instances: Mutex::new(HashMap::new()),
            constraint_violations: Mutex::new(Vec::new()),
            instantiation_events: Mutex::new(Vec::new()),
        }
    }

    /// Track a generic type instantiation
    pub fn track_generic_instantiation(&self, base_type: &str, type_params: Vec<String>, ptr: usize) {
        let event = InstantiationEvent {
            base_type: base_type.to_string(),
            type_parameters: type_params.clone(),
            ptr,
            timestamp: current_timestamp(),
            thread_id: format!("{:?}", std::thread::current().id()),
        };

        if let Ok(mut events) = self.instantiation_events.lock() {
            events.push(event);
        }

        // Track the instance
        let instance = GenericInstance {
            base_type: base_type.to_string(),
            type_parameters: type_params,
            ptr,
            size: 0, // Will be updated when allocation info is available
            constraints: extract_constraints(base_type),
        };

        if let Ok(mut instances) = self.generic_instances.lock() {
            instances.entry(base_type.to_string()).or_insert_with(Vec::new).push(instance);
        }
    }

    /// Analyze generic constraints for a type
    pub fn analyze_constraints(&self, type_name: &str) -> Vec<GenericConstraint> {
        extract_constraints(type_name)
    }

    /// Check for constraint violations
    pub fn check_constraint_violations(&self, type_name: &str, actual_params: &[String]) -> Vec<ConstraintViolation> {
        let constraints = self.analyze_constraints(type_name);
        let mut violations = Vec::new();

        for constraint in constraints {
            if !self.validate_constraint(&constraint, actual_params) {
                violations.push(ConstraintViolation {
                    constraint: constraint.clone(),
                    actual_type: actual_params.join(", "),
                    violation_type: ViolationType::ConstraintNotSatisfied,
                    timestamp: current_timestamp(),
                });
            }
        }

        violations
    }

    /// Get statistics about generic usage
    pub fn get_generic_statistics(&self) -> GenericStatistics {
        let instances = self.generic_instances.lock().unwrap();
        let events = self.instantiation_events.lock().unwrap();
        let violations = self.constraint_violations.lock().unwrap();

        let total_instances: usize = instances.values().map(|v| v.len()).sum();
        let unique_base_types = instances.len();
        let total_instantiations = events.len();
        let constraint_violations = violations.len();

        // Calculate most used generic types
        let mut type_usage: HashMap<String, usize> = HashMap::new();
        for (base_type, instances) in instances.iter() {
            type_usage.insert(base_type.clone(), instances.len());
        }

        let most_used_types: Vec<(String, usize)> = {
            let mut sorted: Vec<_> = type_usage.into_iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(&a.1));
            sorted.into_iter().take(10).collect()
        };

        GenericStatistics {
            total_instances,
            unique_base_types,
            total_instantiations,
            constraint_violations,
            most_used_types,
        }
    }

    /// Validate a constraint against actual type parameters
    fn validate_constraint(&self, constraint: &GenericConstraint, actual_params: &[String]) -> bool {
        match &constraint.constraint_type {
            ConstraintType::Trait(trait_name) => {
                // In a real implementation, this would check if the type implements the trait
                // For now, we'll do basic pattern matching
                self.type_implements_trait(actual_params, trait_name)
            }
            ConstraintType::Lifetime => {
                // Lifetime constraints are typically handled by the compiler
                true
            }
            ConstraintType::Sized => {
                // Most types are Sized by default
                !actual_params.iter().any(|t| t.contains("dyn ") || t.contains("?Sized"))
            }
            ConstraintType::Send => {
                // Check if types are Send
                self.type_is_send(actual_params)
            }
            ConstraintType::Sync => {
                // Check if types are Sync
                self.type_is_sync(actual_params)
            }
        }
    }

    /// Check if type implements a trait (simplified)
    fn type_implements_trait(&self, types: &[String], trait_name: &str) -> bool {
        // Simplified trait checking - in reality this would need compiler integration
        match trait_name {
            "Clone" => types.iter().all(|t| self.is_cloneable_type(t)),
            "Debug" => types.iter().all(|t| self.is_debug_type(t)),
            "Default" => types.iter().all(|t| self.is_default_type(t)),
            "PartialEq" => types.iter().all(|t| self.is_partial_eq_type(t)),
            _ => true, // Assume satisfied for unknown traits
        }
    }

    /// Check if type is Send (simplified)
    fn type_is_send(&self, types: &[String]) -> bool {
        !types.iter().any(|t| t.contains("Rc<") || t.contains("RefCell<"))
    }

    /// Check if type is Sync (simplified)
    fn type_is_sync(&self, types: &[String]) -> bool {
        !types.iter().any(|t| t.contains("Cell<") || t.contains("RefCell<"))
    }

    /// Simplified trait implementation checks
    fn is_cloneable_type(&self, type_name: &str) -> bool {
        !type_name.contains("Mutex<") && !type_name.contains("File")
    }

    fn is_debug_type(&self, type_name: &str) -> bool {
        !type_name.contains("fn(") // Function pointers don't implement Debug by default
    }

    fn is_default_type(&self, type_name: &str) -> bool {
        type_name.contains("Vec<") || type_name.contains("HashMap<") || type_name.contains("String")
    }

    fn is_partial_eq_type(&self, type_name: &str) -> bool {
        !type_name.contains("fn(") && !type_name.contains("Mutex<")
    }
}

/// Generic type instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericInstance {
    /// Base generic type (e.g., "Vec", "HashMap")
    pub base_type: String,
    /// Actual type parameters (e.g., ["i32"], ["String", "usize"])
    pub type_parameters: Vec<String>,
    /// Memory pointer
    pub ptr: usize,
    /// Size of the instance
    pub size: usize,
    /// Constraints that apply to this generic type
    pub constraints: Vec<GenericConstraint>,
}

/// Generic constraint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericConstraint {
    /// Parameter name (e.g., "T", "K", "V")
    pub parameter_name: String,
    /// Type of constraint
    pub constraint_type: ConstraintType,
    /// Human-readable description
    pub description: String,
}

/// Types of generic constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Trait bound (e.g., T: Clone)
    Trait(String),
    /// Lifetime constraint
    Lifetime,
    /// Sized constraint
    Sized,
    /// Send constraint
    Send,
    /// Sync constraint
    Sync,
}

/// Generic instantiation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantiationEvent {
    /// Base type being instantiated
    pub base_type: String,
    /// Type parameters used
    pub type_parameters: Vec<String>,
    /// Memory pointer
    pub ptr: usize,
    /// Timestamp of instantiation
    pub timestamp: u64,
    /// Thread where instantiation occurred
    pub thread_id: String,
}

/// Constraint violation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintViolation {
    /// The constraint that was violated
    pub constraint: GenericConstraint,
    /// The actual type that violated the constraint
    pub actual_type: String,
    /// Type of violation
    pub violation_type: ViolationType,
    /// When the violation was detected
    pub timestamp: u64,
}

/// Types of constraint violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    /// Constraint not satisfied
    ConstraintNotSatisfied,
    /// Lifetime mismatch
    LifetimeMismatch,
    /// Missing trait implementation
    MissingTraitImpl,
}

/// Statistics about generic type usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericStatistics {
    /// Total number of generic instances
    pub total_instances: usize,
    /// Number of unique base types
    pub unique_base_types: usize,
    /// Total instantiation events
    pub total_instantiations: usize,
    /// Number of constraint violations
    pub constraint_violations: usize,
    /// Most frequently used generic types
    pub most_used_types: Vec<(String, usize)>,
}

/// Extract generic constraints from a type name
fn extract_constraints(type_name: &str) -> Vec<GenericConstraint> {
    let mut constraints = Vec::new();

    // Common patterns for constraint extraction
    if type_name.contains("Vec<") || type_name.contains("HashMap<") {
        constraints.push(GenericConstraint {
            parameter_name: "T".to_string(),
            constraint_type: ConstraintType::Sized,
            description: "Type must be Sized".to_string(),
        });
    }

    if type_name.contains("Arc<") || type_name.contains("Rc<") {
        constraints.push(GenericConstraint {
            parameter_name: "T".to_string(),
            constraint_type: ConstraintType::Sized,
            description: "Type must be Sized for smart pointers".to_string(),
        });
    }

    if type_name.contains("Mutex<") || type_name.contains("RwLock<") {
        constraints.push(GenericConstraint {
            parameter_name: "T".to_string(),
            constraint_type: ConstraintType::Send,
            description: "Type must be Send for thread-safe containers".to_string(),
        });
    }

    constraints
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Parse generic type parameters from a type name
pub fn parse_generic_parameters(type_name: &str) -> (String, Vec<String>) {
    if let Some(start) = type_name.find('<') {
        if let Some(end) = type_name.rfind('>') {
            let base_type = type_name[..start].to_string();
            let params_str = &type_name[start + 1..end];
            
            // Simple parameter parsing - in reality this would need proper parsing
            let params: Vec<String> = params_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            
            return (base_type, params);
        }
    }
    
    (type_name.to_string(), Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_generic_parameters() {
        let (base, params) = parse_generic_parameters("Vec<i32>");
        assert_eq!(base, "Vec");
        assert_eq!(params, vec!["i32"]);

        let (base, params) = parse_generic_parameters("HashMap<String, usize>");
        assert_eq!(base, "HashMap");
        assert_eq!(params, vec!["String", "usize"]);

        let (base, params) = parse_generic_parameters("String");
        assert_eq!(base, "String");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_constraint_extraction() {
        let constraints = extract_constraints("Vec<T>");
        assert!(!constraints.is_empty());
        assert!(constraints.iter().any(|c| matches!(c.constraint_type, ConstraintType::Sized)));

        let constraints = extract_constraints("Mutex<T>");
        assert!(constraints.iter().any(|c| matches!(c.constraint_type, ConstraintType::Send)));
    }
}