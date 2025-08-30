//! Schema validator for unsafe/FFI analysis data
//!
//! This module provides a schema validator for unsafe/FFI analysis data.
//! It validates JSON data against the unsafe/FFI analysis schema.

use crate::core::types::{TrackingError, TrackingResult};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Validation result containing errors, warnings, and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// List of validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// Validation metrics
    pub validation_metrics: ValidationMetrics,
    /// Data integrity hash
    pub integrity_hash: String,
    /// Validation timestamp
    pub validation_timestamp: u128,
}

/// Validation error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// JSON path where error occurred
    pub path: String,
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Suggested fix for the error
    pub suggested_fix: Option<String>,
}

/// Validation warning details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning code
    pub warning_code: String,
    /// Human-readable warning message
    pub message: String,
    /// JSON path where warning occurred
    pub path: String,
    /// Suggested action
    pub suggestion: Option<String>,
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Critical - validation fails
    Critical,
    /// Warning - validation passes with warnings
    Warning,
    /// Info - informational messages
    Info,
}

/// Schema version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// Version string (e.g., "2.0")
    pub version: String,
    /// Version components (major, minor, patch)
    pub components: (u32, u32, u32),
    /// Whether this version is supported
    pub is_supported: bool,
    /// Compatibility level with current version
    pub compatibility: CompatibilityLevel,
    /// Backward compatibility information
    pub backward_compatible_with: Vec<String>,
    /// Forward compatibility information
    pub forward_compatible_with: Vec<String>,
}

/// Schema compatibility levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompatibilityLevel {
    /// Fully compatible
    FullyCompatible,
    /// Backward compatible
    BackwardCompatible,
    /// Forward compatible
    ForwardCompatible,
    /// Incompatible
    Incompatible,
}

/// Configuration for schema validation
#[derive(Debug, Clone)]
pub struct SchemaValidatorConfig {
    /// Strict validation mode (fail on warnings)
    pub strict_mode: bool,
    /// Enable data integrity checking
    pub enable_integrity_check: bool,
    /// Enable backward compatibility checking
    pub enable_backward_compatibility: bool,
    /// Maximum allowed schema version
    pub max_schema_version: String,
    /// Custom validation rules
    pub custom_rules: Vec<CustomValidationRule>,
}

/// Custom validation rule
#[derive(Debug, Clone)]
pub struct CustomValidationRule {
    /// Rule name
    pub name: String,
    /// JSONPath pattern to match
    pub path_pattern: String,
    /// Validation function
    pub validator: fn(&Value) -> Result<(), String>,
}

impl Default for SchemaValidatorConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            enable_integrity_check: true,
            enable_backward_compatibility: true,
            max_schema_version: "2.0".to_string(),
            custom_rules: Vec::new(),
        }
    }
}

/// JSON Schema validator for unsafe/FFI analysis data
pub struct SchemaValidator {
    /// Validation configuration
    config: SchemaValidatorConfig,
    /// Supported schema versions
    supported_versions: HashMap<String, SchemaVersion>,
    /// Schema definitions
    schema_definitions: HashMap<String, Value>,
}

impl SchemaValidator {
    /// Create a new schema validator with default configuration
    pub fn new() -> Self {
        Self::with_config(SchemaValidatorConfig::default())
    }

    /// Create a new schema validator with custom configuration
    pub fn with_config(config: SchemaValidatorConfig) -> Self {
        let mut validator = Self {
            config,
            supported_versions: HashMap::new(),
            schema_definitions: HashMap::new(),
        };

        validator.initialize_schemas();
        validator
    }

    /// Validate JSON data against the unsafe/FFI analysis schema
    pub fn validate_unsafe_ffi_analysis(&self, data: &Value) -> TrackingResult<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut validation_metrics = ValidationMetrics::default();

        // Extract and validate metadata
        let metadata = self.extract_metadata(data, &mut errors)?;

        let schema_version = metadata
            .get("schema_version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0");

        // Validate schema version
        self.validate_schema_version(schema_version, &mut errors, &mut warnings)?;

        // Validate main document structure
        self.validate_main_structure(data, &mut errors, &mut warnings, &mut validation_metrics)?;

        // Validate unsafe analysis section
        if let Some(unsafe_analysis) = data.get("unsafe_analysis") {
            self.validate_unsafe_analysis(unsafe_analysis, &mut errors, &mut warnings)?;
        }

        // Validate FFI analysis section
        if let Some(ffi_analysis) = data.get("ffi_analysis") {
            self.validate_ffi_analysis(ffi_analysis, &mut errors, &mut warnings)?;
        }

        // Validate boundary analysis section
        if let Some(boundary_analysis) = data.get("boundary_analysis") {
            self.validate_boundary_analysis(boundary_analysis, &mut errors, &mut warnings)?;
        }

        // Validate safety violations section
        if let Some(safety_violations) = data.get("safety_violations") {
            self.validate_safety_violations(safety_violations, &mut errors, &mut warnings)?;
        }

        // Apply custom validation rules
        self.apply_custom_rules(data, &mut errors, &mut warnings)?;

        // Calculate data integrity hash
        let integrity_hash = if self.config.enable_integrity_check {
            self.calculate_integrity_hash(data)?
        } else {
            "disabled".to_string()
        };

        // Determine if validation passed
        let is_valid = errors.is_empty() && (!self.config.strict_mode || warnings.is_empty());

        Ok(ValidationResult {
            is_valid,
            errors,
            warnings,
            validation_metrics,
            integrity_hash,
            validation_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        })
    }

    /// Extract metadata from JSON data
    fn extract_metadata<'a>(
        &self,
        data: &'a Value,
        errors: &mut Vec<ValidationError>,
    ) -> TrackingResult<&'a Map<String, Value>> {
        match data.get("metadata") {
            Some(Value::Object(metadata)) => Ok(metadata),
            Some(_) => {
                errors.push(ValidationError {
                    code: "INVALID_METADATA_TYPE".to_string(),
                    message: "Metadata must be an object".to_string(),
                    path: "metadata".to_string(),
                    severity: ErrorSeverity::Critical,
                    suggested_fix: Some("Ensure metadata is a JSON object".to_string()),
                });
                Err(TrackingError::ValidationError(
                    "Invalid metadata type".to_string(),
                ))
            }
            None => {
                errors.push(ValidationError {
                    code: "MISSING_METADATA".to_string(),
                    message: "Required metadata section is missing".to_string(),
                    path: "metadata".to_string(),
                    severity: ErrorSeverity::Critical,
                    suggested_fix: Some("Add metadata section with required fields".to_string()),
                });
                Err(TrackingError::ValidationError(
                    "Missing metadata".to_string(),
                ))
            }
        }
    }

    /// Validate schema version
    fn validate_schema_version(
        &self,
        version: &str,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) -> TrackingResult<()> {
        if let Some(schema_version) = self.supported_versions.get(version) {
            if !schema_version.is_supported {
                errors.push(ValidationError {
                    code: "UNSUPPORTED_SCHEMA_VERSION".to_string(),
                    message: format!("Schema version {version} is not supported"),
                    path: "metadata.schema_version".to_string(),
                    severity: ErrorSeverity::Critical,
                    suggested_fix: Some("Use a supported schema version".to_string()),
                });
            }
        } else {
            errors.push(ValidationError {
                code: "UNKNOWN_SCHEMA_VERSION".to_string(),
                message: format!("Unknown schema version: {version}"),
                path: "metadata.schema_version".to_string(),
                severity: ErrorSeverity::Critical,
                suggested_fix: Some("Use a supported schema version".to_string()),
            });
        }

        // Check version compatibility
        if self.config.enable_backward_compatibility {
            self.check_backward_compatibility(version, warnings)?;
        }

        Ok(())
    }

    /// Calculate data integrity hash
    fn calculate_integrity_hash(&self, data: &Value) -> TrackingResult<String> {
        // Convert to canonical JSON string for consistent hashing
        let canonical_json = serde_json::to_string(data)
            .map_err(|e| TrackingError::SerializationError(e.to_string()))?;

        let hash = self.simple_hash(&canonical_json);
        Ok(format!("{hash:x}"))
    }

    /// Verify data integrity using provided hash
    pub fn verify_integrity(&self, data: &Value, expected_hash: &str) -> TrackingResult<bool> {
        let calculated_hash = self.calculate_integrity_hash(data)?;
        Ok(calculated_hash == expected_hash)
    }

    /// Get supported schema versions
    pub fn get_supported_versions(&self) -> &HashMap<String, SchemaVersion> {
        &self.supported_versions
    }

    /// Add custom validation rule
    pub fn add_custom_rule(&mut self, rule: CustomValidationRule) {
        self.config.custom_rules.push(rule);
    }
}

// Private implementation methods
impl SchemaValidator {
    /// Initialize schema definitions and supported versions
    fn initialize_schemas(&mut self) {
        // Define supported schema versions
        self.supported_versions.insert(
            "1.0".to_string(),
            SchemaVersion {
                version: "1.0".to_string(),
                components: (1, 0, 0),
                is_supported: true,
                compatibility: CompatibilityLevel::BackwardCompatible,
                backward_compatible_with: vec![],
                forward_compatible_with: vec!["2.0".to_string()],
            },
        );

        self.supported_versions.insert(
            "2.0".to_string(),
            SchemaVersion {
                version: "2.0".to_string(),
                components: (2, 0, 0),
                is_supported: true,
                compatibility: CompatibilityLevel::FullyCompatible,
                backward_compatible_with: vec!["1.0".to_string()],
                forward_compatible_with: vec![],
            },
        );

        // Initialize schema definitions
        self.initialize_v2_schema();
    }

    /// Initialize v2.0 schema definition
    fn initialize_v2_schema(&mut self) {
        let schema = serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Unsafe/FFI Memory Analysis Schema v2.0",
            "type": "object",
            "required": ["metadata", "unsafe_analysis", "ffi_analysis"],
            "properties": {
                "metadata": {
                    "type": "object",
                    "required": ["analysis_type", "schema_version", "export_timestamp"],
                    "properties": {
                        "analysis_type": {"const": "unsafe_ffi_analysis_optimized"},
                        "schema_version": {"const": "2.0"},
                        "export_timestamp": {"type": "integer", "minimum": 0},
                        "optimization_level": {"enum": ["low", "medium", "high"]},
                        "processing_mode": {"enum": ["sequential", "parallel", "streaming"]},
                        "data_integrity_hash": {"type": "string"}
                    }
                },
                "unsafe_analysis": {
                    "type": "object",
                    "required": ["summary", "allocations"],
                    "properties": {
                        "summary": {"$ref": "#/definitions/RiskDistribution"},
                        "allocations": {"type": "array"},
                        "performance_metrics": {"type": "object"}
                    }
                },
                "ffi_analysis": {
                    "type": "object",
                    "required": ["summary", "allocations"],
                    "properties": {
                        "summary": {"$ref": "#/definitions/RiskDistribution"},
                        "allocations": {"type": "array"},
                        "performance_metrics": {"type": "object"}
                    }
                },
                "boundary_analysis": {
                    "type": "object",
                    "properties": {
                        "cross_boundary_transfers": {"type": "integer", "minimum": 0},
                        "events": {"type": "array"},
                        "performance_impact": {"type": "object"}
                    }
                },
                "safety_violations": {
                    "type": "object",
                    "properties": {
                        "severity_breakdown": {"type": "object"},
                        "violations": {"type": "array"}
                    }
                }
            },
            "definitions": {
                "RiskDistribution": {
                    "type": "object",
                    "properties": {
                        "low_risk": {"type": "integer", "minimum": 0},
                        "medium_risk": {"type": "integer", "minimum": 0},
                        "high_risk": {"type": "integer", "minimum": 0},
                        "critical_risk": {"type": "integer", "minimum": 0},
                        "overall_risk_score": {"type": "number", "minimum": 0.0, "maximum": 10.0}
                    }
                }
            }
        });

        self.schema_definitions.insert("2.0".to_string(), schema);
    }

    /// Validate main document structure
    fn validate_main_structure(
        &self,
        data: &Value,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
        metrics: &mut ValidationMetrics,
    ) -> TrackingResult<()> {
        // Validate that root is an object
        if !data.is_object() {
            errors.push(ValidationError {
                code: "INVALID_ROOT_TYPE".to_string(),
                message: "Root document must be a JSON object".to_string(),
                path: "$".to_string(),
                severity: ErrorSeverity::Critical,
                suggested_fix: Some("Ensure root document is a JSON object".to_string()),
            });
            return Ok(());
        }

        metrics.sections_validated += 1;

        // Check for unknown sections
        let known_sections = [
            "metadata",
            "unsafe_analysis",
            "ffi_analysis",
            "boundary_analysis",
            "safety_violations",
        ];

        if let Some(obj) = data.as_object() {
            for key in obj.keys() {
                if !known_sections.contains(&key.as_str()) {
                    warnings.push(ValidationWarning {
                        warning_code: "UNKNOWN_SECTION".to_string(),
                        message: format!("Unknown section: {key}"),
                        path: key.clone(),
                        suggestion: Some("Remove unknown sections or update schema".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Validate unsafe analysis section
    fn validate_unsafe_analysis(
        &self,
        unsafe_analysis: &Value,
        errors: &mut Vec<ValidationError>,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> TrackingResult<()> {
        if !unsafe_analysis.is_object() {
            errors.push(ValidationError {
                code: "INVALID_UNSAFE_ANALYSIS_TYPE".to_string(),
                message: "unsafe_analysis must be an object".to_string(),
                path: "unsafe_analysis".to_string(),
                severity: ErrorSeverity::Critical,
                suggested_fix: Some("Ensure unsafe_analysis is a JSON object".to_string()),
            });
            return Ok(());
        }

        // Validate required fields
        let required_fields = vec!["summary", "allocations"];
        for field in required_fields {
            if unsafe_analysis.get(field).is_none() {
                errors.push(ValidationError {
                    code: "MISSING_REQUIRED_FIELD".to_string(),
                    message: format!("Required field '{field}' is missing in unsafe_analysis"),
                    path: format!("unsafe_analysis.{field}"),
                    severity: ErrorSeverity::Critical,
                    suggested_fix: Some(format!("Add required field '{field}'")),
                });
            }
        }

        Ok(())
    }

    /// Validate FFI analysis section
    fn validate_ffi_analysis(
        &self,
        ffi_analysis: &Value,
        errors: &mut Vec<ValidationError>,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> TrackingResult<()> {
        if !ffi_analysis.is_object() {
            errors.push(ValidationError {
                code: "INVALID_FFI_ANALYSIS_TYPE".to_string(),
                message: "ffi_analysis must be an object".to_string(),
                path: "ffi_analysis".to_string(),
                severity: ErrorSeverity::Critical,
                suggested_fix: Some("Ensure ffi_analysis is a JSON object".to_string()),
            });
            return Ok(());
        }

        // Validate required fields
        let required_fields = vec!["summary", "allocations"];
        for field in required_fields {
            if ffi_analysis.get(field).is_none() {
                errors.push(ValidationError {
                    code: "MISSING_REQUIRED_FIELD".to_string(),
                    message: format!("Required field '{field}' is missing in ffi_analysis"),
                    path: format!("ffi_analysis.{field}"),
                    severity: ErrorSeverity::Critical,
                    suggested_fix: Some(format!("Add required field '{field}'")),
                });
            }
        }

        Ok(())
    }

    /// Validate boundary analysis section
    fn validate_boundary_analysis(
        &self,
        boundary_analysis: &Value,
        errors: &mut Vec<ValidationError>,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> TrackingResult<()> {
        if !boundary_analysis.is_object() {
            errors.push(ValidationError {
                code: "INVALID_BOUNDARY_ANALYSIS_TYPE".to_string(),
                message: "boundary_analysis must be an object".to_string(),
                path: "boundary_analysis".to_string(),
                severity: ErrorSeverity::Critical,
                suggested_fix: Some("Ensure boundary_analysis is a JSON object".to_string()),
            });
            return Ok(());
        }

        // Validate cross_boundary_transfers field
        if let Some(transfers) = boundary_analysis.get("cross_boundary_transfers") {
            if !transfers.is_number() {
                errors.push(ValidationError {
                    code: "INVALID_FIELD_TYPE".to_string(),
                    message: "cross_boundary_transfers must be a number".to_string(),
                    path: "boundary_analysis.cross_boundary_transfers".to_string(),
                    severity: ErrorSeverity::Warning,
                    suggested_fix: Some("Ensure cross_boundary_transfers is a number".to_string()),
                });
            }
        }

        Ok(())
    }

    /// Validate safety violations section
    fn validate_safety_violations(
        &self,
        safety_violations: &Value,
        errors: &mut Vec<ValidationError>,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> TrackingResult<()> {
        if !safety_violations.is_object() {
            errors.push(ValidationError {
                code: "INVALID_SAFETY_VIOLATIONS_TYPE".to_string(),
                message: "safety_violations must be an object".to_string(),
                path: "safety_violations".to_string(),
                severity: ErrorSeverity::Critical,
                suggested_fix: Some("Ensure safety_violations is a JSON object".to_string()),
            });
            return Ok(());
        }

        // Validate violations array
        if let Some(violations) = safety_violations.get("violations") {
            if !violations.is_array() {
                errors.push(ValidationError {
                    code: "INVALID_FIELD_TYPE".to_string(),
                    message: "violations must be an array".to_string(),
                    path: "safety_violations.violations".to_string(),
                    severity: ErrorSeverity::Critical,
                    suggested_fix: Some("Ensure violations is an array".to_string()),
                });
            }
        }

        Ok(())
    }

    /// Apply custom validation rules
    fn apply_custom_rules(
        &self,
        data: &Value,
        errors: &mut Vec<ValidationError>,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> TrackingResult<()> {
        for rule in &self.config.custom_rules {
            // Simplified path matching - in production, use proper JSONPath
            if let Some(target_value) = self.get_value_by_path(data, &rule.path_pattern) {
                if let Err(error_msg) = (rule.validator)(target_value) {
                    errors.push(ValidationError {
                        code: "CUSTOM_RULE_VIOLATION".to_string(),
                        message: format!("Custom rule '{}' failed: {}", rule.name, error_msg),
                        path: rule.path_pattern.clone(),
                        severity: ErrorSeverity::Warning,
                        suggested_fix: None,
                    });
                }
            }
        }
        Ok(())
    }

    /// Check backward compatibility
    fn check_backward_compatibility(
        &self,
        version: &str,
        warnings: &mut Vec<ValidationWarning>,
    ) -> TrackingResult<()> {
        if let Some(schema_version) = self.supported_versions.get(version) {
            if schema_version.backward_compatible_with.is_empty() {
                warnings.push(ValidationWarning {
                    warning_code: "NO_BACKWARD_COMPATIBILITY".to_string(),
                    message: format!("Schema version {version} has no backward compatibility"),
                    path: "metadata.schema_version".to_string(),
                    suggestion: Some("Consider using a more recent schema version".to_string()),
                });
            }
        }
        Ok(())
    }

    /// Get value by simple path (simplified JSONPath)
    fn get_value_by_path<'a>(&self, data: &'a Value, path: &str) -> Option<&'a Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = data;

        for part in parts {
            current = current.get(part)?;
        }

        Some(current)
    }

    /// Simple hash function (use proper cryptographic hash in production)
    fn simple_hash(&self, data: &str) -> u64 {
        let mut hash = 0u64;
        for byte in data.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationMetrics {
    /// Number of sections validated
    pub sections_validated: u32,
    /// Number of fields validated
    pub fields_validated: u32,
    /// Validation duration in nanoseconds
    pub validation_duration_ns: u128,
}

/// Convenience functions for common validation tasks
impl SchemaValidator {
    /// Create a validator with strict mode enabled
    pub fn strict() -> Self {
        Self::with_config(SchemaValidatorConfig {
            strict_mode: true,
            ..Default::default()
        })
    }

    /// Create a validator with integrity checking disabled
    pub fn without_integrity_check() -> Self {
        Self::with_config(SchemaValidatorConfig {
            enable_integrity_check: false,
            ..Default::default()
        })
    }
}

/// Builder for schema validator configuration
pub struct SchemaValidatorConfigBuilder {
    config: SchemaValidatorConfig,
}

impl SchemaValidatorConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: SchemaValidatorConfig::default(),
        }
    }

    /// Enable strict mode
    pub fn strict_mode(mut self, enabled: bool) -> Self {
        self.config.strict_mode = enabled;
        self
    }

    /// Enable integrity checking
    pub fn integrity_check(mut self, enabled: bool) -> Self {
        self.config.enable_integrity_check = enabled;
        self
    }

    /// Enable backward compatibility checking
    pub fn backward_compatibility(mut self, enabled: bool) -> Self {
        self.config.enable_backward_compatibility = enabled;
        self
    }

    /// Set maximum allowed schema version
    pub fn max_schema_version(mut self, version: String) -> Self {
        self.config.max_schema_version = version;
        self
    }

    /// Add a custom validation rule
    pub fn add_custom_rule(mut self, rule: CustomValidationRule) -> Self {
        self.config.custom_rules.push(rule);
        self
    }

    /// Build the configuration
    pub fn build(self) -> SchemaValidatorConfig {
        self.config
    }
}

impl Default for SchemaValidatorConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validator_creation() {
        let validator = SchemaValidator::new();
        assert!(!validator.supported_versions.is_empty());
    }

    #[test]
    fn test_valid_minimal_document() {
        let validator = SchemaValidator::new();
        let data = json!({
            "metadata": {
                "analysis_type": "unsafe_ffi_analysis_optimized",
                "schema_version": "2.0",
                "export_timestamp": 1234567890
            },
            "unsafe_analysis": {
                "summary": {"low_risk": 0, "medium_risk": 0, "high_risk": 0, "critical_risk": 0},
                "allocations": []
            },
            "ffi_analysis": {
                "summary": {"low_risk": 0, "medium_risk": 0, "high_risk": 0, "critical_risk": 0},
                "allocations": []
            }
        });

        let result = validator
            .validate_unsafe_ffi_analysis(&data)
            .expect("Failed to validate FFI analysis");
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_missing_metadata() {
        let validator = SchemaValidator::new();
        let data = json!({
            "unsafe_analysis": {},
            "ffi_analysis": {}
        });

        let result = validator.validate_unsafe_ffi_analysis(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_schema_version() {
        let validator = SchemaValidator::new();
        let data = json!({
            "metadata": {
                "analysis_type": "unsafe_ffi_analysis_optimized",
                "schema_version": "999.0",
                "export_timestamp": 1234567890
            },
            "unsafe_analysis": {"summary": {}, "allocations": []},
            "ffi_analysis": {"summary": {}, "allocations": []}
        });

        let result = validator
            .validate_unsafe_ffi_analysis(&data)
            .expect("Test operation failed");
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_integrity_hash_calculation() {
        let validator = SchemaValidator::new();
        let data = json!({"test": "data"});

        let hash1 = validator
            .calculate_integrity_hash(&data)
            .expect("Failed to calculate hash 1");
        let hash2 = validator
            .calculate_integrity_hash(&data)
            .expect("Failed to calculate hash 2");

        assert_eq!(hash1, hash2);
        assert!(validator
            .verify_integrity(&data, &hash1)
            .expect("Failed to verify integrity"));
    }

    #[test]
    fn test_config_builder() {
        let config = SchemaValidatorConfigBuilder::new()
            .strict_mode(true)
            .integrity_check(false)
            .max_schema_version("2.0".to_string())
            .build();

        assert!(config.strict_mode);
        assert!(!config.enable_integrity_check);
        assert_eq!(config.max_schema_version, "2.0");
    }

    #[test]
    fn test_convenience_methods() {
        let validator = SchemaValidator::new();
        let valid_data = json!({
            "metadata": {
                "analysis_type": "unsafe_ffi_analysis_optimized",
                "schema_version": "2.0",
                "export_timestamp": 1234567890
            },
            "unsafe_analysis": {"summary": {}, "allocations": []},
            "ffi_analysis": {"summary": {}, "allocations": []}
        });

        let result = validator
            .validate_unsafe_ffi_analysis(&valid_data)
            .expect("Failed to validate valid data");
        assert!(result.is_valid);

        let strict_validator = SchemaValidator::strict();
        let result = strict_validator
            .validate_unsafe_ffi_analysis(&valid_data)
            .expect("Failed to validate with strict mode");
        // In strict mode, the minimal valid data should still be valid
        assert!(result.is_valid);

        let no_integrity_validator = SchemaValidator::without_integrity_check();
        let result = no_integrity_validator
            .validate_unsafe_ffi_analysis(&valid_data)
            .expect("Failed to validate without integrity check");
        assert_eq!(result.integrity_hash, "disabled");
    }
}
