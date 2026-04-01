use crate::analysis::unsafe_ffi_tracker::{RiskLevel, StackFrame};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RiskFactorType {
    RawPointerDereference,
    UnsafeDataRace,
    InvalidTransmute,
    FfiCall,
    ManualMemoryManagement,
    CrossBoundaryTransfer,
    UseAfterFree,
    BufferOverflow,
    LifetimeViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: RiskFactorType,
    pub severity: f64,
    pub confidence: f64,
    pub description: String,
    pub source_location: Option<String>,
    pub call_stack: Vec<StackFrame>,
    pub mitigation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub risk_level: RiskLevel,
    pub risk_score: f64,
    pub risk_factors: Vec<RiskFactor>,
    pub confidence_score: f64,
    pub mitigation_suggestions: Vec<String>,
    pub assessment_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeReport {
    pub report_id: String,
    pub source: UnsafeSource,
    pub risk_assessment: RiskAssessment,
    pub dynamic_violations: Vec<DynamicViolation>,
    pub related_passports: Vec<String>,
    pub memory_context: MemoryContext,
    pub generated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnsafeSource {
    UnsafeBlock {
        location: String,
        function: String,
        file_path: Option<String>,
        line_number: Option<u32>,
    },
    FfiFunction {
        library: String,
        function: String,
        call_site: String,
    },
    RawPointer {
        operation: String,
        location: String,
    },
    Transmute {
        from_type: String,
        to_type: String,
        location: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicViolation {
    pub violation_type: ViolationType,
    pub memory_address: usize,
    pub memory_size: usize,
    pub detected_at: u64,
    pub call_stack: Vec<StackFrame>,
    pub severity: RiskLevel,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    DoubleFree,
    UseAfterFree,
    BufferOverflow,
    InvalidAccess,
    DataRace,
    FfiBoundaryViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContext {
    pub total_allocated: usize,
    pub active_allocations: usize,
    pub memory_pressure: MemoryPressureLevel,
    pub allocation_patterns: Vec<AllocationPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryPressureLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPattern {
    pub pattern_type: String,
    pub frequency: u32,
    pub average_size: usize,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPassport {
    pub passport_id: String,
    pub allocation_ptr: usize,
    pub size_bytes: usize,
    pub status_at_shutdown: PassportStatus,
    pub lifecycle_events: Vec<PassportEvent>,
    pub risk_assessment: RiskAssessment,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PassportStatus {
    FreedByRust,
    HandoverToFfi,
    FreedByForeign,
    ReclaimedByRust,
    InForeignCustody,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportEvent {
    pub event_type: PassportEventType,
    pub timestamp: u64,
    pub context: String,
    pub call_stack: Vec<StackFrame>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PassportEventType {
    AllocatedInRust,
    HandoverToFfi,
    FreedByForeign,
    ReclaimedByRust,
    BoundaryAccess,
    OwnershipTransfer,
}
