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
            sorted.sort_by(|a, b| b.1.cmp(&a.1));
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
