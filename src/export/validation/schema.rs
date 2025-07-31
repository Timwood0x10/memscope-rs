//! Schema validation for exports.
//!
//! This module consolidates schema validation functionality from:
//! - schema_validator.rs

// Re-export existing schema validation functionality
pub use super::schema_validator::*;

/// Unified schema validation interface
pub struct SchemaValidator {
    // Will consolidate all schema validation here
}

impl SchemaValidator {
    /// Create a new schema validator
    pub fn new() -> Self {
        Self {}
    }
    
    /// Validate export data against schema
    pub fn validate_export_data(&self, _data: &[crate::core::types::AllocationInfo]) -> ValidationResult {
        // TODO: Consolidate schema validation logic
        ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    /// Validate JSON schema
    pub fn validate_json_schema(&self, _json_data: &str) -> ValidationResult {
        // TODO: Move JSON schema validation here
        ValidationResult::default()
    }
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}