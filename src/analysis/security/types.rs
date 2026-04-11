use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl ViolationSeverity {
    pub fn score(&self) -> u32 {
        match self {
            ViolationSeverity::Critical => 100,
            ViolationSeverity::High => 75,
            ViolationSeverity::Medium => 50,
            ViolationSeverity::Low => 25,
            ViolationSeverity::Info => 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStateSnapshot {
    pub timestamp_ns: u64,
    pub total_allocated_bytes: usize,
    pub active_allocation_count: usize,
    pub involved_addresses: Vec<String>,
    pub stack_trace: Vec<StackFrame>,
    pub related_allocations: Vec<RelatedAllocation>,
    pub memory_pressure: MemoryPressureLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub function_name: String,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub frame_address: String,
    pub is_unsafe: bool,
    pub is_ffi: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedAllocation {
    pub address: String,
    pub size: usize,
    pub type_name: Option<String>,
    pub variable_name: Option<String>,
    pub allocated_at_ns: u64,
    pub is_active: bool,
    pub relationship: AllocationRelationship,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AllocationRelationship {
    SameRegion,
    Adjacent,
    SameType,
    SameScope,
    DoubleFreeCandidate,
    LeakRelated,
    UseAfterFreeRelated,
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryPressureLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityViolationReport {
    pub violation_id: String,
    pub violation_type: String,
    pub severity: ViolationSeverity,
    pub description: String,
    pub technical_details: String,
    pub memory_snapshot: MemoryStateSnapshot,
    pub impact_assessment: ImpactAssessment,
    pub remediation_suggestions: Vec<String>,
    pub correlated_violations: Vec<String>,
    pub integrity_hash: String,
    pub generated_at_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub exploitability_score: f64,
    pub data_corruption_risk: bool,
    pub information_disclosure_risk: bool,
    pub denial_of_service_risk: bool,
    pub code_execution_risk: bool,
    pub overall_risk_score: f64,
}

#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub max_related_allocations: usize,
    pub max_stack_depth: usize,
    pub enable_correlation_analysis: bool,
    pub include_low_severity: bool,
    pub generate_integrity_hashes: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            max_related_allocations: 10,
            max_stack_depth: 20,
            enable_correlation_analysis: true,
            include_low_severity: true,
            generate_integrity_hashes: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify ViolationSeverity score values
    /// Invariants: Critical should be highest, Info should be lowest
    #[test]
    fn test_violation_severity_scores() {
        assert_eq!(
            ViolationSeverity::Critical.score(),
            100,
            "Critical score should be 100"
        );
        assert_eq!(
            ViolationSeverity::High.score(),
            75,
            "High score should be 75"
        );
        assert_eq!(
            ViolationSeverity::Medium.score(),
            50,
            "Medium score should be 50"
        );
        assert_eq!(ViolationSeverity::Low.score(), 25, "Low score should be 25");
        assert_eq!(
            ViolationSeverity::Info.score(),
            10,
            "Info score should be 10"
        );
    }

    /// Objective: Verify ViolationSeverity ordering
    /// Invariants: Scores should reflect severity ordering
    #[test]
    fn test_violation_severity_ordering() {
        assert!(ViolationSeverity::Critical.score() > ViolationSeverity::High.score());
        assert!(ViolationSeverity::High.score() > ViolationSeverity::Medium.score());
        assert!(ViolationSeverity::Medium.score() > ViolationSeverity::Low.score());
        assert!(ViolationSeverity::Low.score() > ViolationSeverity::Info.score());
    }

    /// Objective: Verify ViolationSeverity equality
    /// Invariants: Same severities should be equal
    #[test]
    fn test_violation_severity_equality() {
        assert_eq!(ViolationSeverity::Critical, ViolationSeverity::Critical);
        assert_eq!(ViolationSeverity::High, ViolationSeverity::High);
        assert_ne!(ViolationSeverity::Critical, ViolationSeverity::High);
    }

    /// Objective: Verify MemoryStateSnapshot creation
    /// Invariants: All fields should be properly initialized
    #[test]
    fn test_memory_state_snapshot_creation() {
        let snapshot = MemoryStateSnapshot {
            timestamp_ns: 1000,
            total_allocated_bytes: 1024 * 1024,
            active_allocation_count: 10,
            involved_addresses: vec!["0x1000".to_string()],
            stack_trace: vec![],
            related_allocations: vec![],
            memory_pressure: MemoryPressureLevel::Medium,
        };

        assert_eq!(snapshot.timestamp_ns, 1000, "Timestamp should match");
        assert_eq!(
            snapshot.total_allocated_bytes,
            1024 * 1024,
            "Total bytes should match"
        );
        assert_eq!(
            snapshot.active_allocation_count, 10,
            "Allocation count should match"
        );
    }

    /// Objective: Verify StackFrame creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_stack_frame_creation() {
        let frame = StackFrame {
            function_name: "test_function".to_string(),
            file_path: Some("test.rs".to_string()),
            line_number: Some(42),
            frame_address: "0x7FFF12345678".to_string(),
            is_unsafe: true,
            is_ffi: false,
        };

        assert_eq!(
            frame.function_name, "test_function",
            "Function name should match"
        );
        assert_eq!(frame.line_number, Some(42), "Line number should match");
        assert!(frame.is_unsafe, "Should be marked unsafe");
        assert!(!frame.is_ffi, "Should not be FFI");
    }

    /// Objective: Verify RelatedAllocation creation
    /// Invariants: All fields should be properly initialized
    #[test]
    fn test_related_allocation_creation() {
        let related = RelatedAllocation {
            address: "0x1000".to_string(),
            size: 1024,
            type_name: Some("Vec<u8>".to_string()),
            variable_name: Some("data".to_string()),
            allocated_at_ns: 1000,
            is_active: true,
            relationship: AllocationRelationship::SameRegion,
        };

        assert_eq!(related.size, 1024, "Size should match");
        assert!(related.is_active, "Should be active");
        assert_eq!(
            related.relationship,
            AllocationRelationship::SameRegion,
            "Relationship should match"
        );
    }

    /// Objective: Verify AllocationRelationship variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_allocation_relationship_variants() {
        let relationships = vec![
            AllocationRelationship::SameRegion,
            AllocationRelationship::Adjacent,
            AllocationRelationship::SameType,
            AllocationRelationship::SameScope,
            AllocationRelationship::DoubleFreeCandidate,
            AllocationRelationship::LeakRelated,
            AllocationRelationship::UseAfterFreeRelated,
            AllocationRelationship::None,
        ];

        for rel in &relationships {
            let debug_str = format!("{rel:?}");
            assert!(
                !debug_str.is_empty(),
                "Relationship should have debug representation"
            );
        }
    }

    /// Objective: Verify MemoryPressureLevel variants
    /// Invariants: All levels should be distinct
    #[test]
    fn test_memory_pressure_level_variants() {
        assert_eq!(MemoryPressureLevel::Low, MemoryPressureLevel::Low);
        assert_eq!(MemoryPressureLevel::Medium, MemoryPressureLevel::Medium);
        assert_eq!(MemoryPressureLevel::High, MemoryPressureLevel::High);
        assert_eq!(MemoryPressureLevel::Critical, MemoryPressureLevel::Critical);

        assert_ne!(MemoryPressureLevel::Low, MemoryPressureLevel::Critical);
    }

    /// Objective: Verify SecurityViolationReport creation
    /// Invariants: All fields should be properly initialized
    #[test]
    fn test_security_violation_report_creation() {
        let report = SecurityViolationReport {
            violation_id: "SEC-DF-123".to_string(),
            violation_type: "DoubleFree".to_string(),
            severity: ViolationSeverity::Critical,
            description: "Double free detected".to_string(),
            technical_details: "Technical info".to_string(),
            memory_snapshot: MemoryStateSnapshot {
                timestamp_ns: 0,
                total_allocated_bytes: 0,
                active_allocation_count: 0,
                involved_addresses: vec![],
                stack_trace: vec![],
                related_allocations: vec![],
                memory_pressure: MemoryPressureLevel::Low,
            },
            impact_assessment: ImpactAssessment {
                exploitability_score: 0.9,
                data_corruption_risk: true,
                information_disclosure_risk: false,
                denial_of_service_risk: true,
                code_execution_risk: true,
                overall_risk_score: 0.9,
            },
            remediation_suggestions: vec!["Fix the bug".to_string()],
            correlated_violations: vec![],
            integrity_hash: "abc123".to_string(),
            generated_at_ns: 1000,
        };

        assert_eq!(
            report.violation_id, "SEC-DF-123",
            "Violation ID should match"
        );
        assert_eq!(
            report.severity,
            ViolationSeverity::Critical,
            "Severity should be Critical"
        );
        assert!(
            report.impact_assessment.code_execution_risk,
            "Should have code execution risk"
        );
    }

    /// Objective: Verify ImpactAssessment creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_impact_assessment_creation() {
        let impact = ImpactAssessment {
            exploitability_score: 0.75,
            data_corruption_risk: true,
            information_disclosure_risk: true,
            denial_of_service_risk: false,
            code_execution_risk: false,
            overall_risk_score: 0.6,
        };

        assert_eq!(
            impact.exploitability_score, 0.75,
            "Exploitability should match"
        );
        assert!(
            impact.data_corruption_risk,
            "Should have data corruption risk"
        );
        assert!(!impact.denial_of_service_risk, "Should not have DoS risk");
    }

    /// Objective: Verify ImpactAssessment edge values
    /// Invariants: Should handle zero and max values
    #[test]
    fn test_impact_assessment_edge_values() {
        let zero_impact = ImpactAssessment {
            exploitability_score: 0.0,
            data_corruption_risk: false,
            information_disclosure_risk: false,
            denial_of_service_risk: false,
            code_execution_risk: false,
            overall_risk_score: 0.0,
        };

        let max_impact = ImpactAssessment {
            exploitability_score: 1.0,
            data_corruption_risk: true,
            information_disclosure_risk: true,
            denial_of_service_risk: true,
            code_execution_risk: true,
            overall_risk_score: 1.0,
        };

        assert_eq!(
            zero_impact.exploitability_score, 0.0,
            "Zero exploitability should be valid"
        );
        assert_eq!(
            max_impact.exploitability_score, 1.0,
            "Max exploitability should be valid"
        );
    }

    /// Objective: Verify AnalysisConfig default values
    /// Invariants: Default should have sensible values
    #[test]
    fn test_analysis_config_default() {
        let config = AnalysisConfig::default();

        assert_eq!(
            config.max_related_allocations, 10,
            "Default max related should be 10"
        );
        assert_eq!(
            config.max_stack_depth, 20,
            "Default max stack depth should be 20"
        );
        assert!(
            config.enable_correlation_analysis,
            "Correlation should be enabled by default"
        );
        assert!(
            config.include_low_severity,
            "Low severity should be included by default"
        );
        assert!(
            config.generate_integrity_hashes,
            "Integrity hashes should be enabled by default"
        );
    }

    /// Objective: Verify AnalysisConfig custom values
    /// Invariants: Custom values should be respected
    #[test]
    fn test_analysis_config_custom() {
        let config = AnalysisConfig {
            max_related_allocations: 5,
            max_stack_depth: 10,
            enable_correlation_analysis: false,
            include_low_severity: false,
            generate_integrity_hashes: false,
        };

        assert_eq!(
            config.max_related_allocations, 5,
            "Custom max related should be 5"
        );
        assert_eq!(
            config.max_stack_depth, 10,
            "Custom max stack depth should be 10"
        );
        assert!(
            !config.enable_correlation_analysis,
            "Correlation should be disabled"
        );
    }

    /// Objective: Verify serialization of types
    /// Invariants: Types should serialize and deserialize correctly
    #[test]
    fn test_serialization() {
        let severity = ViolationSeverity::Critical;
        let json = serde_json::to_string(&severity);
        assert!(json.is_ok(), "Should serialize to JSON");

        let deserialized: Result<ViolationSeverity, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "Should deserialize from JSON");
        assert_eq!(
            deserialized.unwrap(),
            ViolationSeverity::Critical,
            "Should preserve value"
        );
    }

    /// Objective: Verify SecurityViolationReport serialization
    /// Invariants: Report should serialize correctly
    #[test]
    fn test_report_serialization() {
        let report = SecurityViolationReport {
            violation_id: "SEC-TEST-123".to_string(),
            violation_type: "Test".to_string(),
            severity: ViolationSeverity::High,
            description: "Test description".to_string(),
            technical_details: "Test details".to_string(),
            memory_snapshot: MemoryStateSnapshot {
                timestamp_ns: 1000,
                total_allocated_bytes: 1024,
                active_allocation_count: 1,
                involved_addresses: vec!["0x1000".to_string()],
                stack_trace: vec![],
                related_allocations: vec![],
                memory_pressure: MemoryPressureLevel::Low,
            },
            impact_assessment: ImpactAssessment {
                exploitability_score: 0.5,
                data_corruption_risk: false,
                information_disclosure_risk: false,
                denial_of_service_risk: true,
                code_execution_risk: false,
                overall_risk_score: 0.5,
            },
            remediation_suggestions: vec!["Fix it".to_string()],
            correlated_violations: vec![],
            integrity_hash: "".to_string(),
            generated_at_ns: 2000,
        };

        let json = serde_json::to_string(&report);
        assert!(json.is_ok(), "Report should serialize to JSON");

        let deserialized: Result<SecurityViolationReport, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "Report should deserialize from JSON");
    }

    /// Objective: Verify StackFrame with None values
    /// Invariants: Should handle missing file path and line number
    #[test]
    fn test_stack_frame_with_none_values() {
        let frame = StackFrame {
            function_name: "unknown".to_string(),
            file_path: None,
            line_number: None,
            frame_address: "0x0".to_string(),
            is_unsafe: false,
            is_ffi: false,
        };

        assert!(frame.file_path.is_none(), "File path should be None");
        assert!(frame.line_number.is_none(), "Line number should be None");
    }

    /// Objective: Verify RelatedAllocation with None type name
    /// Invariants: Should handle missing type and variable names
    #[test]
    fn test_related_allocation_with_none_values() {
        let related = RelatedAllocation {
            address: "0x1000".to_string(),
            size: 0,
            type_name: None,
            variable_name: None,
            allocated_at_ns: 0,
            is_active: false,
            relationship: AllocationRelationship::None,
        };

        assert!(related.type_name.is_none(), "Type name should be None");
        assert!(
            related.variable_name.is_none(),
            "Variable name should be None"
        );
        assert!(!related.is_active, "Should be inactive");
    }
}
