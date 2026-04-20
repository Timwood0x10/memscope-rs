use crate::analysis::generic::types::*;
use crate::analysis::generic::utils::{extract_constraints, parse_generic_parameters};
use crate::core::safe_operations::SafeLock;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static GLOBAL_GENERIC_ANALYZER: OnceLock<Arc<GenericAnalyzer>> = OnceLock::new();

pub fn get_global_generic_analyzer() -> Arc<GenericAnalyzer> {
    GLOBAL_GENERIC_ANALYZER
        .get_or_init(|| Arc::new(GenericAnalyzer::default()))
        .clone()
}

pub struct GenericAnalyzer {
    pub generic_instances: Mutex<HashMap<String, Vec<GenericInstance>>>,
    pub constraint_violations: Mutex<Vec<ConstraintViolation>>,
    pub instantiation_events: Mutex<Vec<InstantiationEvent>>,
}

impl Default for GenericAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl GenericAnalyzer {
    pub fn new() -> Self {
        Self {
            generic_instances: Mutex::new(HashMap::new()),
            constraint_violations: Mutex::new(Vec::new()),
            instantiation_events: Mutex::new(Vec::new()),
        }
    }

    pub fn track_generic_instantiation(
        &self,
        base_type: &str,
        type_params: Vec<String>,
        ptr: usize,
    ) {
        self.track_generic_instantiation_with_name(base_type, base_type, type_params, ptr);
    }

    pub fn track_generic_instantiation_with_name(
        &self,
        name: &str,
        base_type: &str,
        type_params: Vec<String>,
        ptr: usize,
    ) {
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

        let instance = GenericInstance {
            name: name.to_string(),
            base_type: base_type.to_string(),
            underlying_type: base_type.to_string(),
            type_parameters: type_params.clone(),
            ptr,
            size: 0,
            constraints: extract_constraints(base_type),
            is_type_alias: name != base_type,
        };

        if let Ok(mut instances) = self.generic_instances.lock() {
            instances
                .entry(name.to_string())
                .or_default()
                .push(instance);
        }
    }

    pub fn track_type_alias_instantiation(
        &self,
        alias_name: &str,
        underlying_type: &str,
        type_params: Vec<String>,
        ptr: usize,
    ) {
        let (base_type, resolved_params) = parse_generic_parameters(underlying_type);

        let event = InstantiationEvent {
            base_type: base_type.clone(),
            type_parameters: if resolved_params.is_empty() {
                type_params.clone()
            } else {
                resolved_params.clone()
            },
            ptr,
            timestamp: current_timestamp(),
            thread_id: format!("{:?}", std::thread::current().id()),
        };

        if let Ok(mut events) = self.instantiation_events.lock() {
            events.push(event);
        }

        let instance = GenericInstance {
            name: alias_name.to_string(),
            base_type: base_type.clone(),
            underlying_type: underlying_type.to_string(),
            type_parameters: if resolved_params.is_empty() {
                type_params
            } else {
                resolved_params
            },
            ptr,
            size: 0,
            constraints: extract_constraints(underlying_type),
            is_type_alias: true,
        };

        if let Ok(mut instances) = self.generic_instances.lock() {
            instances
                .entry(alias_name.to_string())
                .or_default()
                .push(instance);
        }
    }

    pub fn analyze_constraints(&self, type_name: &str) -> Vec<GenericConstraint> {
        extract_constraints(type_name)
    }

    pub fn check_constraint_violations(
        &self,
        type_name: &str,
        actual_params: &[String],
    ) -> Vec<ConstraintViolation> {
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

    pub fn get_generic_statistics(&self) -> GenericStatistics {
        let instances = self
            .generic_instances
            .safe_lock()
            .expect("Failed to acquire lock on generic_instances");
        let events = self
            .instantiation_events
            .safe_lock()
            .expect("Failed to acquire lock on instantiation_events");
        let violations = self
            .constraint_violations
            .safe_lock()
            .expect("Failed to acquire lock on constraint_violations");
        let total_instances: usize = instances.values().map(|v| v.len()).sum();
        let unique_base_types = instances.len();
        let total_instantiations = events.len();
        let constraint_violations = violations.len();

        let mut type_usage: HashMap<String, usize> = HashMap::new();
        let mut alias_count = 0;

        for (_name, instance_list) in instances.iter() {
            for instance in instance_list {
                if instance.is_type_alias {
                    alias_count += 1;
                    *type_usage
                        .entry(instance.underlying_type.clone())
                        .or_insert(0) += 1;
                } else {
                    *type_usage.entry(instance.name.clone()).or_insert(0) += 1;
                }
            }
        }

        let most_used_types: Vec<(String, usize)> = {
            let mut sorted: Vec<_> = type_usage.into_iter().collect();
            sorted.sort_by_key(|b| std::cmp::Reverse(b.1));
            sorted.into_iter().take(10).collect()
        };

        GenericStatistics {
            total_instances,
            unique_base_types,
            total_instantiations,
            constraint_violations,
            most_used_types,
            type_aliases_count: alias_count,
        }
    }

    pub fn get_type_aliases(&self) -> Vec<TypeAliasInfo> {
        let instances = self
            .generic_instances
            .safe_lock()
            .expect("Failed to acquire lock on generic_instances");
        let mut alias_map: HashMap<String, TypeAliasInfo> = HashMap::new();

        for (_name, instance_list) in instances.iter() {
            for instance in instance_list {
                if instance.is_type_alias {
                    let alias_name = instance.name.clone();

                    if let Some(existing) = alias_map.get_mut(&alias_name) {
                        existing.usage_count += 1;
                    } else {
                        alias_map.insert(
                            alias_name.clone(),
                            TypeAliasInfo {
                                alias_name,
                                underlying_type: instance.underlying_type.clone(),
                                base_type: instance.base_type.clone(),
                                type_parameters: instance.type_parameters.clone(),
                                usage_count: 1,
                            },
                        );
                    }
                }
            }
        }

        alias_map.into_values().collect()
    }

    pub fn resolve_type_alias(&self, alias_name: &str) -> Option<String> {
        let instances = self
            .generic_instances
            .safe_lock()
            .expect("Failed to acquire lock on generic_instances");

        if let Some(instance_list) = instances.get(alias_name) {
            for instance in instance_list {
                if instance.is_type_alias {
                    return Some(instance.underlying_type.clone());
                }
            }
        }
        None
    }

    fn validate_constraint(
        &self,
        constraint: &GenericConstraint,
        actual_params: &[String],
    ) -> bool {
        match &constraint.constraint_type {
            ConstraintType::Trait(trait_name) => {
                self.type_implements_trait(actual_params, trait_name)
            }
            ConstraintType::Lifetime => true,
            ConstraintType::Sized => !actual_params
                .iter()
                .any(|t| t.contains("dyn ") || t.contains("?Sized")),
            ConstraintType::Send => self.type_is_send(actual_params),
            ConstraintType::Sync => self.type_is_sync(actual_params),
        }
    }

    fn type_implements_trait(&self, types: &[String], trait_name: &str) -> bool {
        match trait_name {
            "Clone" => types.iter().all(|t| self.is_cloneable_type(t)),
            "Debug" => types.iter().all(|t| self.is_debug_type(t)),
            "Default" => types.iter().all(|t| self.is_default_type(t)),
            "PartialEq" => types.iter().all(|t| self.is_partial_eq_type(t)),
            _ => true,
        }
    }

    fn type_is_send(&self, types: &[String]) -> bool {
        !types
            .iter()
            .any(|t| t.contains("Rc<") || t.contains("RefCell<"))
    }

    fn type_is_sync(&self, types: &[String]) -> bool {
        !types
            .iter()
            .any(|t| t.contains("Cell<") || t.contains("RefCell<"))
    }

    fn is_cloneable_type(&self, type_name: &str) -> bool {
        !type_name.contains("Mutex<") && !type_name.contains("File")
    }

    fn is_debug_type(&self, type_name: &str) -> bool {
        !type_name.contains("fn(")
    }

    fn is_default_type(&self, type_name: &str) -> bool {
        type_name.contains("Vec<") || type_name.contains("HashMap<") || type_name.contains("String")
    }

    fn is_partial_eq_type(&self, type_name: &str) -> bool {
        !type_name.contains("fn(") && !type_name.contains("Mutex<")
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify GenericAnalyzer creation with default values
    /// Invariants: New analyzer should have empty collections
    #[test]
    fn test_generic_analyzer_creation() {
        let analyzer = GenericAnalyzer::new();

        let instances = analyzer.generic_instances.lock().unwrap();
        let violations = analyzer.constraint_violations.lock().unwrap();
        let events = analyzer.instantiation_events.lock().unwrap();

        assert!(
            instances.is_empty(),
            "New analyzer should have no instances"
        );
        assert!(
            violations.is_empty(),
            "New analyzer should have no violations"
        );
        assert!(events.is_empty(), "New analyzer should have no events");
    }

    /// Objective: Verify Default trait implementation
    /// Invariants: Default should create same as new()
    #[test]
    fn test_generic_analyzer_default() {
        let analyzer = GenericAnalyzer::default();

        let instances = analyzer.generic_instances.lock().unwrap();
        assert!(
            instances.is_empty(),
            "Default analyzer should have no instances"
        );
    }

    /// Objective: Verify track_generic_instantiation functionality
    /// Invariants: Should add instance and event correctly
    #[test]
    fn test_track_generic_instantiation() {
        let analyzer = GenericAnalyzer::new();

        analyzer.track_generic_instantiation("Vec", vec!["i32".to_string()], 0x1000);

        let instances = analyzer.generic_instances.lock().unwrap();
        assert!(instances.contains_key("Vec"), "Should contain Vec instance");
        assert_eq!(
            instances.get("Vec").unwrap().len(),
            1,
            "Should have one Vec instance"
        );

        let events = analyzer.instantiation_events.lock().unwrap();
        assert_eq!(events.len(), 1, "Should have one event");
    }

    /// Objective: Verify track_generic_instantiation_with_name functionality
    /// Invariants: Should track instance with custom name
    #[test]
    fn test_track_generic_instantiation_with_name() {
        let analyzer = GenericAnalyzer::new();

        analyzer.track_generic_instantiation_with_name(
            "MyVec",
            "Vec",
            vec!["String".to_string()],
            0x2000,
        );

        let instances = analyzer.generic_instances.lock().unwrap();
        assert!(
            instances.contains_key("MyVec"),
            "Should contain MyVec instance"
        );

        let instance = &instances.get("MyVec").unwrap()[0];
        assert_eq!(instance.name, "MyVec", "Name should be MyVec");
        assert_eq!(instance.base_type, "Vec", "Base type should be Vec");
        assert!(instance.is_type_alias, "Should be marked as type alias");
    }

    /// Objective: Verify track_type_alias_instantiation functionality
    /// Invariants: Should track type alias correctly
    #[test]
    fn test_track_type_alias_instantiation() {
        let analyzer = GenericAnalyzer::new();

        analyzer.track_type_alias_instantiation("StringList", "Vec<String>", vec![], 0x3000);

        let instances = analyzer.generic_instances.lock().unwrap();
        assert!(
            instances.contains_key("StringList"),
            "Should contain StringList instance"
        );

        let instance = &instances.get("StringList").unwrap()[0];
        assert_eq!(instance.name, "StringList", "Name should be StringList");
        assert_eq!(
            instance.underlying_type, "Vec<String>",
            "Underlying type should be Vec<String>"
        );
        assert!(instance.is_type_alias, "Should be marked as type alias");
    }

    /// Objective: Verify analyze_constraints functionality
    /// Invariants: Should extract constraints from type name
    #[test]
    fn test_analyze_constraints() {
        let analyzer = GenericAnalyzer::new();

        let constraints = analyzer.analyze_constraints("Vec<T: Clone>");

        assert!(
            !constraints.is_empty(),
            "Should have constraints for Vec<T: Clone>"
        );
    }

    /// Objective: Verify check_constraint_violations with valid params
    /// Invariants: Should return empty violations for valid params
    #[test]
    fn test_check_constraint_violations_valid() {
        let analyzer = GenericAnalyzer::new();

        let violations = analyzer.check_constraint_violations("Vec<T>", &["i32".to_string()]);

        assert!(
            violations.is_empty(),
            "Should have no violations for valid params"
        );
    }

    /// Objective: Verify get_generic_statistics with no data
    /// Invariants: Should return zero statistics
    #[test]
    fn test_get_generic_statistics_empty() {
        let analyzer = GenericAnalyzer::new();

        let stats = analyzer.get_generic_statistics();

        assert_eq!(stats.total_instances, 0, "Should have 0 total instances");
        assert_eq!(
            stats.unique_base_types, 0,
            "Should have 0 unique base types"
        );
        assert_eq!(
            stats.total_instantiations, 0,
            "Should have 0 instantiations"
        );
        assert_eq!(stats.constraint_violations, 0, "Should have 0 violations");
    }

    /// Objective: Verify get_generic_statistics with data
    /// Invariants: Should return correct statistics
    #[test]
    fn test_get_generic_statistics_with_data() {
        let analyzer = GenericAnalyzer::new();

        analyzer.track_generic_instantiation("Vec", vec!["i32".to_string()], 0x1000);
        analyzer.track_generic_instantiation("Vec", vec!["String".to_string()], 0x2000);
        analyzer.track_generic_instantiation(
            "HashMap",
            vec!["String".to_string(), "i32".to_string()],
            0x3000,
        );

        let stats = analyzer.get_generic_statistics();

        assert_eq!(stats.total_instances, 3, "Should have 3 total instances");
        assert_eq!(
            stats.unique_base_types, 2,
            "Should have 2 unique base types (Vec, HashMap)"
        );
        assert_eq!(
            stats.total_instantiations, 3,
            "Should have 3 instantiations"
        );
    }

    /// Objective: Verify get_type_aliases with no aliases
    /// Invariants: Should return empty list
    #[test]
    fn test_get_type_aliases_empty() {
        let analyzer = GenericAnalyzer::new();

        analyzer.track_generic_instantiation("Vec", vec!["i32".to_string()], 0x1000);

        let aliases = analyzer.get_type_aliases();

        assert!(aliases.is_empty(), "Should have no type aliases");
    }

    /// Objective: Verify get_type_aliases with aliases
    /// Invariants: Should return correct alias info
    #[test]
    fn test_get_type_aliases_with_data() {
        let analyzer = GenericAnalyzer::new();

        analyzer.track_type_alias_instantiation("StringList", "Vec<String>", vec![], 0x1000);
        analyzer.track_type_alias_instantiation("StringList", "Vec<String>", vec![], 0x2000);

        let aliases = analyzer.get_type_aliases();

        assert_eq!(aliases.len(), 1, "Should have 1 type alias");
        assert_eq!(
            aliases[0].alias_name, "StringList",
            "Alias name should be StringList"
        );
        assert_eq!(aliases[0].usage_count, 2, "Usage count should be 2");
    }

    /// Objective: Verify resolve_type_alias functionality
    /// Invariants: Should resolve alias to underlying type
    #[test]
    fn test_resolve_type_alias() {
        let analyzer = GenericAnalyzer::new();

        analyzer.track_type_alias_instantiation("IntList", "Vec<i32>", vec![], 0x1000);

        let resolved = analyzer.resolve_type_alias("IntList");

        assert!(resolved.is_some(), "Should resolve IntList");
        assert_eq!(resolved.unwrap(), "Vec<i32>", "Should resolve to Vec<i32>");
    }

    /// Objective: Verify resolve_type_alias for non-existent alias
    /// Invariants: Should return None
    #[test]
    fn test_resolve_type_alias_nonexistent() {
        let analyzer = GenericAnalyzer::new();

        let resolved = analyzer.resolve_type_alias("NonExistent");

        assert!(
            resolved.is_none(),
            "Should return None for non-existent alias"
        );
    }

    /// Objective: Verify type_implements_trait for Clone
    /// Invariants: Should correctly identify Clone types
    #[test]
    fn test_type_implements_trait_clone() {
        let analyzer = GenericAnalyzer::new();

        assert!(
            analyzer.type_implements_trait(&["i32".to_string()], "Clone"),
            "i32 should implement Clone"
        );
        assert!(
            analyzer.type_implements_trait(&["String".to_string()], "Clone"),
            "String should implement Clone"
        );
        assert!(
            !analyzer.type_implements_trait(&["Mutex<i32>".to_string()], "Clone"),
            "Mutex should not implement Clone"
        );
    }

    /// Objective: Verify type_implements_trait for Debug
    /// Invariants: Should correctly identify Debug types
    #[test]
    fn test_type_implements_trait_debug() {
        let analyzer = GenericAnalyzer::new();

        assert!(
            analyzer.type_implements_trait(&["i32".to_string()], "Debug"),
            "i32 should implement Debug"
        );
        assert!(
            !analyzer.type_implements_trait(&["fn()".to_string()], "Debug"),
            "fn() should not implement Debug"
        );
    }

    /// Objective: Verify type_implements_trait for Default
    /// Invariants: Should correctly identify Default types
    #[test]
    fn test_type_implements_trait_default() {
        let analyzer = GenericAnalyzer::new();

        assert!(
            analyzer.type_implements_trait(&["Vec<i32>".to_string()], "Default"),
            "Vec should implement Default"
        );
        assert!(
            analyzer.type_implements_trait(&["String".to_string()], "Default"),
            "String should implement Default"
        );
        assert!(
            !analyzer.type_implements_trait(&["i32".to_string()], "Default"),
            "i32 should not implement Default"
        );
    }

    /// Objective: Verify type_implements_trait for PartialEq
    /// Invariants: Should correctly identify PartialEq types
    #[test]
    fn test_type_implements_trait_partial_eq() {
        let analyzer = GenericAnalyzer::new();

        assert!(
            analyzer.type_implements_trait(&["i32".to_string()], "PartialEq"),
            "i32 should implement PartialEq"
        );
        assert!(
            !analyzer.type_implements_trait(&["fn()".to_string()], "PartialEq"),
            "fn() should not implement PartialEq"
        );
    }

    /// Objective: Verify type_is_send functionality
    /// Invariants: Should correctly identify Send types
    #[test]
    fn test_type_is_send() {
        let analyzer = GenericAnalyzer::new();

        assert!(
            analyzer.type_is_send(&["i32".to_string()]),
            "i32 should be Send"
        );
        assert!(
            analyzer.type_is_send(&["Arc<i32>".to_string()]),
            "Arc should be Send"
        );
        assert!(
            !analyzer.type_is_send(&["Rc<i32>".to_string()]),
            "Rc should not be Send"
        );
        assert!(
            !analyzer.type_is_send(&["RefCell<i32>".to_string()]),
            "RefCell should not be Send"
        );
    }

    /// Objective: Verify type_is_sync functionality
    /// Invariants: Should correctly identify Sync types
    #[test]
    fn test_type_is_sync() {
        let analyzer = GenericAnalyzer::new();

        assert!(
            analyzer.type_is_sync(&["i32".to_string()]),
            "i32 should be Sync"
        );
        assert!(
            analyzer.type_is_sync(&["Arc<i32>".to_string()]),
            "Arc should be Sync"
        );
        assert!(
            !analyzer.type_is_sync(&["Cell<i32>".to_string()]),
            "Cell should not be Sync"
        );
        assert!(
            !analyzer.type_is_sync(&["RefCell<i32>".to_string()]),
            "RefCell should not be Sync"
        );
    }

    /// Objective: Verify is_cloneable_type functionality
    /// Invariants: Should correctly identify cloneable types
    #[test]
    fn test_is_cloneable_type() {
        let analyzer = GenericAnalyzer::new();

        assert!(analyzer.is_cloneable_type("i32"), "i32 should be cloneable");
        assert!(
            analyzer.is_cloneable_type("String"),
            "String should be cloneable"
        );
        assert!(
            !analyzer.is_cloneable_type("Mutex<i32>"),
            "Mutex should not be cloneable"
        );
        assert!(
            !analyzer.is_cloneable_type("File"),
            "File should not be cloneable"
        );
    }

    /// Objective: Verify is_debug_type functionality
    /// Invariants: Should correctly identify Debug types
    #[test]
    fn test_is_debug_type() {
        let analyzer = GenericAnalyzer::new();

        assert!(analyzer.is_debug_type("i32"), "i32 should be Debug");
        assert!(analyzer.is_debug_type("String"), "String should be Debug");
        assert!(
            !analyzer.is_debug_type("fn() -> i32"),
            "fn() should not be Debug"
        );
    }

    /// Objective: Verify is_default_type functionality
    /// Invariants: Should correctly identify Default types
    #[test]
    fn test_is_default_type() {
        let analyzer = GenericAnalyzer::new();

        assert!(
            analyzer.is_default_type("Vec<i32>"),
            "Vec should be Default"
        );
        assert!(
            analyzer.is_default_type("HashMap<String, i32>"),
            "HashMap should be Default"
        );
        assert!(
            analyzer.is_default_type("String"),
            "String should be Default"
        );
        assert!(
            !analyzer.is_default_type("i32"),
            "i32 should not be Default"
        );
    }

    /// Objective: Verify is_partial_eq_type functionality
    /// Invariants: Should correctly identify PartialEq types
    #[test]
    fn test_is_partial_eq_type() {
        let analyzer = GenericAnalyzer::new();

        assert!(
            analyzer.is_partial_eq_type("i32"),
            "i32 should be PartialEq"
        );
        assert!(
            analyzer.is_partial_eq_type("String"),
            "String should be PartialEq"
        );
        assert!(
            !analyzer.is_partial_eq_type("fn()"),
            "fn() should not be PartialEq"
        );
        assert!(
            !analyzer.is_partial_eq_type("Mutex<i32>"),
            "Mutex should not be PartialEq"
        );
    }

    /// Objective: Verify get_global_generic_analyzer singleton
    /// Invariants: Should return same instance
    #[test]
    fn test_global_generic_analyzer_singleton() {
        let analyzer1 = get_global_generic_analyzer();
        let analyzer2 = get_global_generic_analyzer();

        assert!(
            Arc::ptr_eq(&analyzer1, &analyzer2),
            "Should return same instance"
        );
    }

    /// Objective: Verify current_timestamp returns valid value
    /// Invariants: Timestamp should be positive
    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 0, "Timestamp should be positive");
    }

    /// Objective: Verify multiple instantiations of same type
    /// Invariants: Should track all instances
    #[test]
    fn test_multiple_instantiations_same_type() {
        let analyzer = GenericAnalyzer::new();

        for i in 0..5 {
            analyzer.track_generic_instantiation("Vec", vec!["i32".to_string()], 0x1000 + i);
        }

        let instances = analyzer.generic_instances.lock().unwrap();
        assert_eq!(
            instances.get("Vec").unwrap().len(),
            5,
            "Should have 5 Vec instances"
        );

        let events = analyzer.instantiation_events.lock().unwrap();
        assert_eq!(events.len(), 5, "Should have 5 events");
    }

    /// Objective: Verify validate_constraint for Sized
    /// Invariants: Should correctly validate Sized constraint
    #[test]
    fn test_validate_constraint_sized() {
        let analyzer = GenericAnalyzer::new();

        let sized_constraint = GenericConstraint {
            constraint_type: ConstraintType::Sized,
            parameter_name: "T".to_string(),
            description: String::new(),
        };

        assert!(
            analyzer.validate_constraint(&sized_constraint, &["i32".to_string()]),
            "i32 should be Sized"
        );
        assert!(
            !analyzer.validate_constraint(&sized_constraint, &["dyn Any".to_string()]),
            "dyn Any should not be Sized"
        );
        assert!(
            !analyzer.validate_constraint(&sized_constraint, &["?Sized".to_string()]),
            "?Sized should not satisfy Sized constraint"
        );
    }

    /// Objective: Verify validate_constraint for Lifetime
    /// Invariants: Should always return true for Lifetime constraint
    #[test]
    fn test_validate_constraint_lifetime() {
        let analyzer = GenericAnalyzer::new();

        let lifetime_constraint = GenericConstraint {
            constraint_type: ConstraintType::Lifetime,
            parameter_name: "'a".to_string(),
            description: String::new(),
        };

        assert!(
            analyzer.validate_constraint(&lifetime_constraint, &["i32".to_string()]),
            "Lifetime constraint should always be valid"
        );
    }

    /// Objective: Verify validate_constraint for Send
    /// Invariants: Should correctly validate Send constraint
    #[test]
    fn test_validate_constraint_send() {
        let analyzer = GenericAnalyzer::new();

        let send_constraint = GenericConstraint {
            constraint_type: ConstraintType::Send,
            parameter_name: "T".to_string(),
            description: String::new(),
        };

        assert!(
            analyzer.validate_constraint(&send_constraint, &["i32".to_string()]),
            "i32 should be Send"
        );
        assert!(
            !analyzer.validate_constraint(&send_constraint, &["Rc<i32>".to_string()]),
            "Rc should not be Send"
        );
    }

    /// Objective: Verify validate_constraint for Sync
    /// Invariants: Should correctly validate Sync constraint
    #[test]
    fn test_validate_constraint_sync() {
        let analyzer = GenericAnalyzer::new();

        let sync_constraint = GenericConstraint {
            constraint_type: ConstraintType::Sync,
            parameter_name: "T".to_string(),
            description: String::new(),
        };

        assert!(
            analyzer.validate_constraint(&sync_constraint, &["i32".to_string()]),
            "i32 should be Sync"
        );
        assert!(
            !analyzer.validate_constraint(&sync_constraint, &["Cell<i32>".to_string()]),
            "Cell should not be Sync"
        );
    }

    /// Objective: Verify type_implements_trait for unknown trait
    /// Invariants: Should return true for unknown traits
    #[test]
    fn test_type_implements_trait_unknown() {
        let analyzer = GenericAnalyzer::new();

        assert!(
            analyzer.type_implements_trait(&["i32".to_string()], "UnknownTrait"),
            "Unknown traits should return true by default"
        );
    }

    /// Objective: Verify concurrent access to GenericAnalyzer
    /// Invariants: Should handle concurrent operations safely
    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let analyzer = Arc::new(GenericAnalyzer::new());
        let mut handles = vec![];

        for i in 0..5 {
            let analyzer_clone = analyzer.clone();
            let handle = thread::spawn(move || {
                analyzer_clone.track_generic_instantiation(
                    "Vec",
                    vec![format!("Type{}", i)],
                    0x1000 + i,
                );
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let instances = analyzer.generic_instances.lock().unwrap();
        assert_eq!(
            instances.get("Vec").unwrap().len(),
            5,
            "Should have 5 instances from 5 threads"
        );
    }

    /// Objective: Verify statistics with type aliases
    /// Invariants: Should correctly count type aliases
    #[test]
    fn test_statistics_with_type_aliases() {
        let analyzer = GenericAnalyzer::new();

        analyzer.track_generic_instantiation("Vec", vec!["i32".to_string()], 0x1000);
        analyzer.track_type_alias_instantiation("IntVec", "Vec<i32>", vec![], 0x2000);
        analyzer.track_type_alias_instantiation("IntVec", "Vec<i32>", vec![], 0x3000);

        let stats = analyzer.get_generic_statistics();

        assert_eq!(stats.total_instances, 3, "Should have 3 total instances");
        assert_eq!(
            stats.type_aliases_count, 2,
            "Should have 2 type alias instances"
        );
    }
}
