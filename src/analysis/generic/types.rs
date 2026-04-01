use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
