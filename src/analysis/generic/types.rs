use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericInstance {
    pub name: String,
    pub base_type: String,
    pub underlying_type: String,
    pub type_parameters: Vec<String>,
    pub ptr: usize,
    pub size: usize,
    pub constraints: Vec<GenericConstraint>,
    pub is_type_alias: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericConstraint {
    pub parameter_name: String,
    pub constraint_type: ConstraintType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Trait(String),
    Lifetime,
    Sized,
    Send,
    Sync,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantiationEvent {
    pub base_type: String,
    pub type_parameters: Vec<String>,
    pub ptr: usize,
    pub timestamp: u64,
    pub thread_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintViolation {
    pub constraint: GenericConstraint,
    pub actual_type: String,
    pub violation_type: ViolationType,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    ConstraintNotSatisfied,
    LifetimeMismatch,
    MissingTraitImpl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericStatistics {
    pub total_instances: usize,
    pub unique_base_types: usize,
    pub total_instantiations: usize,
    pub constraint_violations: usize,
    pub most_used_types: Vec<(String, usize)>,
    pub type_aliases_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeAliasInfo {
    pub alias_name: String,
    pub underlying_type: String,
    pub base_type: String,
    pub type_parameters: Vec<String>,
    pub usage_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_instance_creation() {
        let instance = GenericInstance {
            name: "Vec<i32>".to_string(),
            base_type: "Vec".to_string(),
            underlying_type: "Vec<i32>".to_string(),
            type_parameters: vec!["i32".to_string()],
            ptr: 0x1000,
            size: 24,
            constraints: vec![],
            is_type_alias: false,
        };

        assert_eq!(instance.name, "Vec<i32>");
        assert_eq!(instance.base_type, "Vec");
        assert_eq!(instance.type_parameters.len(), 1);
        assert!(!instance.is_type_alias);
    }

    #[test]
    fn test_generic_instance_with_constraints() {
        let constraint = GenericConstraint {
            parameter_name: "T".to_string(),
            constraint_type: ConstraintType::Trait("Clone".to_string()),
            description: "T must be Clone".to_string(),
        };

        let instance = GenericInstance {
            name: "HashMap<String, i32>".to_string(),
            base_type: "HashMap".to_string(),
            underlying_type: "HashMap<String, i32>".to_string(),
            type_parameters: vec!["String".to_string(), "i32".to_string()],
            ptr: 0x2000,
            size: 48,
            constraints: vec![constraint],
            is_type_alias: false,
        };

        assert_eq!(instance.constraints.len(), 1);
        assert_eq!(instance.constraints[0].parameter_name, "T");
    }

    #[test]
    fn test_generic_constraint_creation() {
        let constraint = GenericConstraint {
            parameter_name: "T".to_string(),
            constraint_type: ConstraintType::Send,
            description: "T must be Send".to_string(),
        };

        assert_eq!(constraint.parameter_name, "T");
        assert!(matches!(constraint.constraint_type, ConstraintType::Send));
    }

    #[test]
    fn test_constraint_type_variants() {
        let trait_constraint = ConstraintType::Trait("Debug".to_string());
        let lifetime_constraint = ConstraintType::Lifetime;
        let sized_constraint = ConstraintType::Sized;
        let send_constraint = ConstraintType::Send;
        let sync_constraint = ConstraintType::Sync;

        assert!(matches!(trait_constraint, ConstraintType::Trait(_)));
        assert!(matches!(lifetime_constraint, ConstraintType::Lifetime));
        assert!(matches!(sized_constraint, ConstraintType::Sized));
        assert!(matches!(send_constraint, ConstraintType::Send));
        assert!(matches!(sync_constraint, ConstraintType::Sync));
    }

    #[test]
    fn test_instantiation_event_creation() {
        let event = InstantiationEvent {
            base_type: "Vec".to_string(),
            type_parameters: vec!["u8".to_string()],
            ptr: 0x3000,
            timestamp: 1000,
            thread_id: "main".to_string(),
        };

        assert_eq!(event.base_type, "Vec");
        assert_eq!(event.timestamp, 1000);
        assert_eq!(event.thread_id, "main");
    }

    #[test]
    fn test_constraint_violation_creation() {
        let constraint = GenericConstraint {
            parameter_name: "T".to_string(),
            constraint_type: ConstraintType::Trait("Clone".to_string()),
            description: "T must be Clone".to_string(),
        };

        let violation = ConstraintViolation {
            constraint,
            actual_type: "MyType".to_string(),
            violation_type: ViolationType::MissingTraitImpl,
            timestamp: 2000,
        };

        assert_eq!(violation.actual_type, "MyType");
        assert!(matches!(
            violation.violation_type,
            ViolationType::MissingTraitImpl
        ));
    }

    #[test]
    fn test_violation_type_variants() {
        let not_satisfied = ViolationType::ConstraintNotSatisfied;
        let lifetime_mismatch = ViolationType::LifetimeMismatch;
        let missing_trait = ViolationType::MissingTraitImpl;

        assert!(matches!(
            not_satisfied,
            ViolationType::ConstraintNotSatisfied
        ));
        assert!(matches!(lifetime_mismatch, ViolationType::LifetimeMismatch));
        assert!(matches!(missing_trait, ViolationType::MissingTraitImpl));
    }

    #[test]
    fn test_generic_statistics_creation() {
        let stats = GenericStatistics {
            total_instances: 100,
            unique_base_types: 10,
            total_instantiations: 500,
            constraint_violations: 5,
            most_used_types: vec![("Vec<u8>".to_string(), 50)],
            type_aliases_count: 3,
        };

        assert_eq!(stats.total_instances, 100);
        assert_eq!(stats.unique_base_types, 10);
        assert_eq!(stats.most_used_types.len(), 1);
    }

    #[test]
    fn test_generic_statistics_default_values() {
        let stats = GenericStatistics {
            total_instances: 0,
            unique_base_types: 0,
            total_instantiations: 0,
            constraint_violations: 0,
            most_used_types: vec![],
            type_aliases_count: 0,
        };

        assert_eq!(stats.total_instances, 0);
        assert!(stats.most_used_types.is_empty());
    }

    #[test]
    fn test_type_alias_info_creation() {
        let alias = TypeAliasInfo {
            alias_name: "MyResult".to_string(),
            underlying_type: "Result<T, MyError>".to_string(),
            base_type: "Result".to_string(),
            type_parameters: vec!["T".to_string()],
            usage_count: 25,
        };

        assert_eq!(alias.alias_name, "MyResult");
        assert_eq!(alias.usage_count, 25);
    }

    #[test]
    fn test_serialization() {
        let instance = GenericInstance {
            name: "Option<i32>".to_string(),
            base_type: "Option".to_string(),
            underlying_type: "Option<i32>".to_string(),
            type_parameters: vec!["i32".to_string()],
            ptr: 0x1000,
            size: 4,
            constraints: vec![],
            is_type_alias: false,
        };

        let json = serde_json::to_string(&instance);
        assert!(json.is_ok());

        let deserialized: Result<GenericInstance, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_constraint_serialization() {
        let constraint = GenericConstraint {
            parameter_name: "T".to_string(),
            constraint_type: ConstraintType::Trait("Debug".to_string()),
            description: "Test constraint".to_string(),
        };

        let json = serde_json::to_string(&constraint);
        assert!(json.is_ok());

        let deserialized: Result<GenericConstraint, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_statistics_serialization() {
        let stats = GenericStatistics {
            total_instances: 10,
            unique_base_types: 5,
            total_instantiations: 20,
            constraint_violations: 1,
            most_used_types: vec![("String".to_string(), 100)],
            type_aliases_count: 2,
        };

        let json = serde_json::to_string(&stats);
        assert!(json.is_ok());

        let deserialized: Result<GenericStatistics, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_instantiation_event_serialization() {
        let event = InstantiationEvent {
            base_type: "Box".to_string(),
            type_parameters: vec!["dyn Any".to_string()],
            ptr: 0x5000,
            timestamp: 9999,
            thread_id: "worker-1".to_string(),
        };

        let json = serde_json::to_string(&event);
        assert!(json.is_ok());

        let deserialized: Result<InstantiationEvent, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_violation_serialization() {
        let violation = ConstraintViolation {
            constraint: GenericConstraint {
                parameter_name: "T".to_string(),
                constraint_type: ConstraintType::Sync,
                description: "Must be Sync".to_string(),
            },
            actual_type: "Rc<i32>".to_string(),
            violation_type: ViolationType::ConstraintNotSatisfied,
            timestamp: 5000,
        };

        let json = serde_json::to_string(&violation);
        assert!(json.is_ok());

        let deserialized: Result<ConstraintViolation, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_type_alias_serialization() {
        let alias = TypeAliasInfo {
            alias_name: "IntList".to_string(),
            underlying_type: "Vec<i32>".to_string(),
            base_type: "Vec".to_string(),
            type_parameters: vec!["i32".to_string()],
            usage_count: 10,
        };

        let json = serde_json::to_string(&alias);
        assert!(json.is_ok());

        let deserialized: Result<TypeAliasInfo, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_generic_instance_debug() {
        let instance = GenericInstance {
            name: "Test".to_string(),
            base_type: "Base".to_string(),
            underlying_type: "Underlying".to_string(),
            type_parameters: vec![],
            ptr: 0,
            size: 0,
            constraints: vec![],
            is_type_alias: false,
        };

        let debug_str = format!("{:?}", instance);
        assert!(debug_str.contains("GenericInstance"));
    }

    #[test]
    fn test_generic_instance_clone() {
        let instance = GenericInstance {
            name: "Original".to_string(),
            base_type: "Base".to_string(),
            underlying_type: "Underlying".to_string(),
            type_parameters: vec!["T".to_string()],
            ptr: 0x1000,
            size: 8,
            constraints: vec![],
            is_type_alias: true,
        };

        let cloned = instance.clone();
        assert_eq!(cloned.name, instance.name);
        assert_eq!(cloned.ptr, instance.ptr);
    }

    #[test]
    fn test_empty_type_parameters() {
        let instance = GenericInstance {
            name: "String".to_string(),
            base_type: "String".to_string(),
            underlying_type: "String".to_string(),
            type_parameters: vec![],
            ptr: 0x1000,
            size: 24,
            constraints: vec![],
            is_type_alias: false,
        };

        assert!(instance.type_parameters.is_empty());
    }

    #[test]
    fn test_multiple_type_parameters() {
        let instance = GenericInstance {
            name: "HashMap<K, V>".to_string(),
            base_type: "HashMap".to_string(),
            underlying_type: "HashMap<String, i32>".to_string(),
            type_parameters: vec!["String".to_string(), "i32".to_string()],
            ptr: 0x2000,
            size: 48,
            constraints: vec![],
            is_type_alias: false,
        };

        assert_eq!(instance.type_parameters.len(), 2);
    }
}
