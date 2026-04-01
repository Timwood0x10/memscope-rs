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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
