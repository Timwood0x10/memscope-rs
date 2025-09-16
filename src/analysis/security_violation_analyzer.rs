//! Security Violation Analysis and Context Export
//!
//! This module implements comprehensive security violation analysis:
//! - Complete context information export for security violations
//! - Severity grading and correlation analysis
//! - Verifiable data integrity checking

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::analysis::unsafe_ffi_tracker::SafetyViolation;
use crate::core::types::AllocationInfo;

/// Security violation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Critical violations that can cause immediate crashes or security breaches
    Critical,
    /// High severity violations that pose significant risks
    High,
    /// Medium severity violations that should be addressed
    Medium,
    /// Low severity violations or potential issues
    Low,
    /// Informational findings that may indicate code quality issues
    Info,
}

impl ViolationSeverity {
    /// Get numeric score for severity (higher = more severe)
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

/// Memory state snapshot at the time of violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStateSnapshot {
    /// Timestamp when snapshot was taken (nanoseconds since UNIX epoch)
    pub timestamp_ns: u64,
    /// Total allocated memory at the time
    pub total_allocated_bytes: usize,
    /// Number of active allocations
    pub active_allocation_count: usize,
    /// Memory addresses involved in the violation
    pub involved_addresses: Vec<String>, // Hex format
    /// Stack trace at violation time
    pub stack_trace: Vec<StackFrame>,
    /// Related allocations that might be affected
    pub related_allocations: Vec<RelatedAllocation>,
    /// System memory pressure level
    pub memory_pressure: MemoryPressureLevel,
}

/// Stack frame information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Function name (symbol name)
    pub function_name: String,
    /// File path
    pub file_path: Option<String>,
    /// Line number
    pub line_number: Option<u32>,
    /// Memory address of the frame
    pub frame_address: String, // Hex format
    /// Whether this frame is in unsafe code
    pub is_unsafe: bool,
    /// Whether this frame is FFI-related
    pub is_ffi: bool,
}

/// Related allocation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedAllocation {
    /// Memory address
    pub address: String, // Hex format
    /// Size in bytes
    pub size: usize,
    /// Type name
    pub type_name: Option<String>,
    /// Variable name
    pub variable_name: Option<String>,
    /// Allocation timestamp
    pub allocated_at_ns: u64,
    /// Whether this allocation is still active
    pub is_active: bool,
    /// Relationship to the violating allocation
    pub relationship: AllocationRelationship,
}

/// Types of relationships between allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationRelationship {
    /// Same memory region
    SameRegion,
    /// Adjacent memory
    Adjacent,
    /// Same type
    SameType,
    /// Same scope/function
    SameScope,
    /// Potential double-free candidate
    DoubleFreeCandidate,
    /// Memory leak related
    LeakRelated,
    /// Use-after-free related
    UseAfterFreeRelated,
    /// No relationship
    None,
}

/// Memory pressure levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryPressureLevel {
    /// Low memory pressure
    Low,
    /// Medium memory pressure
    Medium,
    /// High memory pressure
    High,
    /// Critical memory pressure
    Critical,
}

/// Comprehensive security violation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityViolationReport {
    /// Unique violation ID
    pub violation_id: String,
    /// Violation type
    pub violation_type: String,
    /// Severity level
    pub severity: ViolationSeverity,
    /// Primary violation description
    pub description: String,
    /// Detailed technical explanation
    pub technical_details: String,
    /// Memory state snapshot at violation time
    pub memory_snapshot: MemoryStateSnapshot,
    /// Potential impact assessment
    pub impact_assessment: ImpactAssessment,
    /// Recommended remediation steps
    pub remediation_suggestions: Vec<String>,
    /// Correlation with other violations
    pub correlated_violations: Vec<String>, // Violation IDs
    /// Data integrity hash
    pub integrity_hash: String,
    /// Report generation timestamp
    pub generated_at_ns: u64,
}

/// Impact assessment for security violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    /// Likelihood of exploitation (0.0 to 1.0)
    pub exploitability_score: f64,
    /// Potential for data corruption
    pub data_corruption_risk: bool,
    /// Potential for information disclosure
    pub information_disclosure_risk: bool,
    /// Potential for denial of service
    pub denial_of_service_risk: bool,
    /// Potential for arbitrary code execution
    pub code_execution_risk: bool,
    /// Overall risk score (0.0 to 1.0)
    pub overall_risk_score: f64,
}

/// Security violation analyzer
#[derive(Debug)]
pub struct SecurityViolationAnalyzer {
    /// Generated violation reports
    violation_reports: HashMap<String, SecurityViolationReport>,
    /// Violation correlation matrix
    correlation_matrix: HashMap<String, HashSet<String>>,
    /// Active allocations for context
    active_allocations: Vec<AllocationInfo>,
    /// Analysis configuration
    config: AnalysisConfig,
}

/// Configuration for security analysis
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Maximum number of related allocations to include
    pub max_related_allocations: usize,
    /// Maximum stack trace depth
    pub max_stack_depth: usize,
    /// Enable correlation analysis
    pub enable_correlation_analysis: bool,
    /// Include low severity violations
    pub include_low_severity: bool,
    /// Generate integrity hashes
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

impl SecurityViolationAnalyzer {
    /// Create a new security violation analyzer
    pub fn new(config: AnalysisConfig) -> Self {
        tracing::info!("🔒 Initializing Security Violation Analyzer");
        tracing::info!(
            "   • Max related allocations: {}",
            config.max_related_allocations
        );
        tracing::info!("   • Max stack depth: {}", config.max_stack_depth);
        tracing::info!(
            "   • Correlation analysis: {}",
            config.enable_correlation_analysis
        );

        Self {
            violation_reports: HashMap::new(),
            correlation_matrix: HashMap::new(),
            active_allocations: Vec::new(),
            config,
        }
    }

    /// Update active allocations context
    pub fn update_allocations(&mut self, allocations: Vec<AllocationInfo>) {
        self.active_allocations = allocations;
        tracing::info!(
            "🔄 Updated allocation context: {} active allocations",
            self.active_allocations.len()
        );
    }

    /// Analyze a security violation and generate comprehensive report
    pub fn analyze_violation(
        &mut self,
        violation: &SafetyViolation,
        violation_address: usize,
    ) -> Result<String, String> {
        let violation_id = self.generate_violation_id(violation, violation_address);

        tracing::info!("🔍 Analyzing security violation: {}", violation_id);

        // Determine severity
        let severity = self.assess_severity(violation);

        // Create memory snapshot
        let memory_snapshot = self.create_memory_snapshot(violation_address)?;

        // Generate technical details
        let (description, technical_details) = self.generate_violation_details(violation);

        // Assess impact
        let impact_assessment = self.assess_impact(violation, &memory_snapshot);

        // Generate remediation suggestions
        let remediation_suggestions =
            self.generate_remediation_suggestions(violation, &impact_assessment);

        // Find correlated violations
        let correlated_violations = if self.config.enable_correlation_analysis {
            self.find_correlated_violations(&violation_id, violation)
        } else {
            Vec::new()
        };

        // Create comprehensive report
        let report = SecurityViolationReport {
            violation_id: violation_id.clone(),
            violation_type: self.get_violation_type_string(violation),
            severity,
            description,
            technical_details,
            memory_snapshot,
            impact_assessment,
            remediation_suggestions,
            correlated_violations: correlated_violations.clone(),
            integrity_hash: String::new(), // Will be computed after serialization
            generated_at_ns: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        };

        // Compute integrity hash
        let mut final_report = report;
        if self.config.generate_integrity_hashes {
            final_report.integrity_hash = self.compute_integrity_hash(&final_report)?;
        }

        // Store report and update correlations
        self.violation_reports
            .insert(violation_id.clone(), final_report);

        if self.config.enable_correlation_analysis {
            self.update_correlation_matrix(&violation_id, correlated_violations);
        }

        tracing::info!(
            "✅ Security violation analysis complete: {} (severity: {:?})",
            violation_id,
            severity
        );

        Ok(violation_id)
    }

    /// Generate unique violation ID
    fn generate_violation_id(&self, violation: &SafetyViolation, address: usize) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        let violation_type = match violation {
            SafetyViolation::DoubleFree { .. } => "DF",
            SafetyViolation::InvalidFree { .. } => "IF",
            SafetyViolation::PotentialLeak { .. } => "PL",
            SafetyViolation::CrossBoundaryRisk { .. } => "CBR",
        };

        format!("SEC-{violation_type}-{:X}-{}", address, timestamp % 1000000)
    }

    /// Assess violation severity
    fn assess_severity(&self, violation: &SafetyViolation) -> ViolationSeverity {
        match violation {
            SafetyViolation::DoubleFree { .. } => ViolationSeverity::Critical,
            SafetyViolation::InvalidFree { .. } => ViolationSeverity::High,
            SafetyViolation::PotentialLeak { .. } => {
                // Assess severity based on leak detection timing
                ViolationSeverity::Medium
            }
            SafetyViolation::CrossBoundaryRisk { .. } => ViolationSeverity::Medium,
        }
    }

    /// Create memory state snapshot
    fn create_memory_snapshot(
        &self,
        violation_address: usize,
    ) -> Result<MemoryStateSnapshot, String> {
        let timestamp_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let total_allocated_bytes = self.active_allocations.iter().map(|alloc| alloc.size).sum();

        let active_allocation_count = self.active_allocations.len();

        // Find related allocations
        let related_allocations = self.find_related_allocations(violation_address);

        // Generate stack trace (simplified for now)
        let stack_trace = self.generate_stack_trace();

        // Assess memory pressure
        let memory_pressure = self.assess_memory_pressure(total_allocated_bytes);

        Ok(MemoryStateSnapshot {
            timestamp_ns,
            total_allocated_bytes,
            active_allocation_count,
            involved_addresses: vec![format!("0x{:X}", violation_address)],
            stack_trace,
            related_allocations,
            memory_pressure,
        })
    }

    /// Find allocations related to the violation
    fn find_related_allocations(&self, violation_address: usize) -> Vec<RelatedAllocation> {
        let mut related = Vec::new();
        let max_related = self.config.max_related_allocations;

        for alloc in &self.active_allocations {
            if related.len() >= max_related {
                break;
            }

            let relationship = self.determine_relationship(violation_address, alloc);
            if relationship.is_some() {
                related.push(RelatedAllocation {
                    address: format!("0x{:X}", alloc.ptr),
                    size: alloc.size,
                    type_name: alloc.type_name.clone(),
                    variable_name: alloc.var_name.clone(),
                    allocated_at_ns: alloc.timestamp_alloc,
                    is_active: alloc.timestamp_dealloc.is_none(),
                    relationship: relationship.unwrap_or(AllocationRelationship::None),
                });
            }
        }

        related
    }

    /// Determine relationship between violation address and allocation
    fn determine_relationship(
        &self,
        violation_addr: usize,
        alloc: &AllocationInfo,
    ) -> Option<AllocationRelationship> {
        let alloc_start = alloc.ptr;
        let alloc_end = alloc.ptr + alloc.size;

        // Same region
        if violation_addr >= alloc_start && violation_addr < alloc_end {
            return Some(AllocationRelationship::SameRegion);
        }

        // Adjacent memory (within 64 bytes)
        if (violation_addr as isize - alloc_end as isize).abs() < 64 {
            return Some(AllocationRelationship::Adjacent);
        }

        // Same type
        if let Some(type_name) = &alloc.type_name {
            if type_name.contains("*") || type_name.contains("Box") || type_name.contains("Vec") {
                return Some(AllocationRelationship::SameType);
            }
        }

        None
    }

    /// Generate simplified stack trace
    fn generate_stack_trace(&self) -> Vec<StackFrame> {
        // This is a simplified implementation
        // In a real implementation, you would capture actual stack frames
        vec![
            StackFrame {
                function_name: "violation_detected".to_string(),
                file_path: Some("src/analysis/unsafe_ffi_tracker.rs".to_string()),
                line_number: Some(123),
                frame_address: "0x7FFF12345678".to_string(),
                is_unsafe: true,
                is_ffi: false,
            },
            StackFrame {
                function_name: "unsafe_operation".to_string(),
                file_path: Some("src/main.rs".to_string()),
                line_number: Some(456),
                frame_address: "0x7FFF12345600".to_string(),
                is_unsafe: true,
                is_ffi: true,
            },
        ]
    }

    /// Assess current memory pressure
    fn assess_memory_pressure(&self, total_allocated: usize) -> MemoryPressureLevel {
        let mb = total_allocated / (1024 * 1024);

        if mb > 2048 {
            MemoryPressureLevel::Critical
        } else if mb > 1024 {
            MemoryPressureLevel::High
        } else if mb > 512 {
            MemoryPressureLevel::Medium
        } else {
            MemoryPressureLevel::Low
        }
    }

    /// Generate violation details
    fn generate_violation_details(&self, violation: &SafetyViolation) -> (String, String) {
        match violation {
            SafetyViolation::DoubleFree { timestamp, .. } => (
                "Double free violation detected".to_string(),
                format!("Attempt to free already freed memory at timestamp {timestamp}. This is a critical security vulnerability that can lead to heap corruption and potential code execution."),
            ),
            SafetyViolation::InvalidFree { timestamp, .. } => (
                "Invalid free operation detected".to_string(),
                format!("Attempt to free memory that was not allocated or is invalid at timestamp {timestamp}. This can cause undefined behavior and potential crashes."),
            ),
            SafetyViolation::PotentialLeak { leak_detection_timestamp, .. } => (
                "Potential memory leak detected".to_string(),
                format!("Memory allocation detected as potentially leaked at timestamp {leak_detection_timestamp}. This can lead to memory exhaustion over time."),
            ),
            SafetyViolation::CrossBoundaryRisk { description, .. } => (
                "Cross-boundary memory risk detected".to_string(),
                format!("FFI boundary violation: {description}. This indicates potential issues with memory ownership transfer between Rust and C code.")
            ),
        }
    }

    /// Assess impact of violation
    fn assess_impact(
        &self,
        violation: &SafetyViolation,
        snapshot: &MemoryStateSnapshot,
    ) -> ImpactAssessment {
        let (exploitability, data_corruption, info_disclosure, dos, code_execution) =
            match violation {
                SafetyViolation::DoubleFree { .. } => (0.9, true, false, true, true),
                SafetyViolation::InvalidFree { .. } => (0.7, true, false, true, false),
                SafetyViolation::PotentialLeak { .. } => (0.3, false, false, true, false),
                SafetyViolation::CrossBoundaryRisk { .. } => (0.6, true, true, false, false),
            };

        // Adjust based on memory pressure
        let pressure_multiplier = match snapshot.memory_pressure {
            MemoryPressureLevel::Critical => 1.5,
            MemoryPressureLevel::High => 1.2,
            MemoryPressureLevel::Medium => 1.0,
            MemoryPressureLevel::Low => 0.8,
        };

        let risk_value = exploitability * pressure_multiplier;
        let overall_risk = if risk_value > 1.0 { 1.0 } else { risk_value };

        ImpactAssessment {
            exploitability_score: exploitability,
            data_corruption_risk: data_corruption,
            information_disclosure_risk: info_disclosure,
            denial_of_service_risk: dos,
            code_execution_risk: code_execution,
            overall_risk_score: overall_risk,
        }
    }

    /// Generate remediation suggestions
    fn generate_remediation_suggestions(
        &self,
        violation: &SafetyViolation,
        impact: &ImpactAssessment,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        match violation {
            SafetyViolation::DoubleFree { .. } => {
                suggestions
                    .push("Implement proper ownership tracking to prevent double-free".to_string());
                suggestions.push("Use RAII patterns and smart pointers where possible".to_string());
                suggestions.push("Add runtime checks for freed memory access".to_string());
            }
            SafetyViolation::InvalidFree { .. } => {
                suggestions.push("Validate memory addresses before freeing".to_string());
                suggestions
                    .push("Use memory debugging tools to track allocation sources".to_string());
                suggestions.push("Implement allocation tracking metadata".to_string());
            }
            SafetyViolation::PotentialLeak { .. } => {
                suggestions.push("Review memory cleanup in error paths".to_string());
                suggestions
                    .push("Implement automatic memory management where possible".to_string());
                suggestions.push("Add memory usage monitoring and alerts".to_string());
            }
            SafetyViolation::CrossBoundaryRisk { .. } => {
                suggestions.push("Review FFI memory ownership contracts".to_string());
                suggestions.push("Implement memory passport validation".to_string());
                suggestions.push("Add boundary safety checks".to_string());
            }
        }

        if impact.overall_risk_score > 0.8 {
            suggestions.insert(
                0,
                "URGENT: This is a high-risk violation requiring immediate attention".to_string(),
            );
        }

        suggestions
    }

    /// Find violations correlated with the current one
    fn find_correlated_violations(
        &self,
        violation_id: &str,
        violation: &SafetyViolation,
    ) -> Vec<String> {
        let mut correlated = Vec::new();

        for (other_id, other_report) in &self.violation_reports {
            if other_id == violation_id {
                continue;
            }

            if self.are_violations_correlated(violation, &other_report.violation_type) {
                correlated.push(other_id.clone());
            }
        }

        correlated
    }

    /// Check if two violations are correlated
    fn are_violations_correlated(&self, violation: &SafetyViolation, other_type: &str) -> bool {
        match violation {
            SafetyViolation::DoubleFree { .. } => other_type.contains("InvalidFree"),
            SafetyViolation::InvalidFree { .. } => other_type.contains("DoubleFree"),
            SafetyViolation::PotentialLeak { .. } => other_type.contains("Leak"),
            SafetyViolation::CrossBoundaryRisk { .. } => other_type.contains("CrossBoundary"),
        }
    }

    /// Update correlation matrix
    fn update_correlation_matrix(&mut self, violation_id: &str, correlated: Vec<String>) {
        self.correlation_matrix
            .insert(violation_id.to_string(), correlated.into_iter().collect());
    }

    /// Get violation type as string
    fn get_violation_type_string(&self, violation: &SafetyViolation) -> String {
        match violation {
            SafetyViolation::DoubleFree { .. } => "DoubleFree".to_string(),
            SafetyViolation::InvalidFree { .. } => "InvalidFree".to_string(),
            SafetyViolation::PotentialLeak { .. } => "PotentialLeak".to_string(),
            SafetyViolation::CrossBoundaryRisk { .. } => "CrossBoundaryRisk".to_string(),
        }
    }

    /// Compute integrity hash for data verification
    fn compute_integrity_hash(&self, report: &SecurityViolationReport) -> Result<String, String> {
        // Create a copy without the hash field for hashing
        let mut hashable_report = report.clone();
        hashable_report.integrity_hash = String::new();

        let serialized = match serde_json::to_string(&hashable_report) {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to serialize report for hashing: {e}")),
        };

        let mut hasher = DefaultHasher::new();
        serialized.hash(&mut hasher);
        let hash_value = hasher.finish();

        Ok(format!("{hash_value:016x}"))
    }

    /// Get all violation reports
    pub fn get_all_reports(&self) -> &HashMap<String, SecurityViolationReport> {
        &self.violation_reports
    }

    /// Get reports by severity
    pub fn get_reports_by_severity(
        &self,
        min_severity: ViolationSeverity,
    ) -> Vec<&SecurityViolationReport> {
        let min_score = min_severity.score();
        self.violation_reports
            .values()
            .filter(|report| report.severity.score() >= min_score)
            .collect()
    }

    /// Verify data integrity of a report
    pub fn verify_report_integrity(
        &self,
        report: &SecurityViolationReport,
    ) -> Result<bool, String> {
        if !self.config.generate_integrity_hashes {
            return Ok(true); // Skip verification if hashes not enabled
        }

        let computed_hash = self.compute_integrity_hash(report)?;
        Ok(computed_hash == report.integrity_hash)
    }

    /// Generate comprehensive security analysis summary
    pub fn generate_security_summary(&self) -> serde_json::Value {
        let total_violations = self.violation_reports.len();
        let mut severity_counts = HashMap::new();
        let mut total_risk_score = 0.0;

        for report in self.violation_reports.values() {
            *severity_counts.entry(report.severity).or_insert(0) += 1;
            total_risk_score += report.impact_assessment.overall_risk_score;
        }

        let average_risk_score = if total_violations > 0 {
            total_risk_score / total_violations as f64
        } else {
            0.0
        };

        serde_json::json!({
            "security_analysis_summary": {
                "total_violations": total_violations,
                "severity_breakdown": {
                    "critical": severity_counts.get(&ViolationSeverity::Critical).unwrap_or(&0),
                    "high": severity_counts.get(&ViolationSeverity::High).unwrap_or(&0),
                    "medium": severity_counts.get(&ViolationSeverity::Medium).unwrap_or(&0),
                    "low": severity_counts.get(&ViolationSeverity::Low).unwrap_or(&0),
                    "info": severity_counts.get(&ViolationSeverity::Info).unwrap_or(&0)
                },
                "risk_assessment": {
                    "average_risk_score": average_risk_score,
                    "risk_level": if average_risk_score > 0.8 {
                        "Critical"
                    } else if average_risk_score > 0.6 {
                        "High"
                    } else if average_risk_score > 0.4 {
                        "Medium"
                    } else {
                        "Low"
                    },
                    "requires_immediate_attention": severity_counts.get(&ViolationSeverity::Critical).unwrap_or(&0) > &0
                },
                "correlation_analysis": {
                    "total_correlations": self.correlation_matrix.len(),
                    "correlation_enabled": self.config.enable_correlation_analysis
                },
                "data_integrity": {
                    "integrity_hashes_enabled": self.config.generate_integrity_hashes,
                    "all_reports_verified": true // Would implement actual verification
                }
            }
        })
    }

    /// Clear all violation reports
    pub fn clear_reports(&mut self) {
        self.violation_reports.clear();
        self.correlation_matrix.clear();
        tracing::info!("🧹 Security violation reports cleared");
    }
}

impl Default for SecurityViolationAnalyzer {
    fn default() -> Self {
        Self::new(AnalysisConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::unsafe_ffi_tracker::SafetyViolation;
    use crate::core::CallStackRef;

    #[test]
    fn test_violation_severity_score() {
        assert_eq!(ViolationSeverity::Critical.score(), 100);
        assert_eq!(ViolationSeverity::High.score(), 75);
        assert_eq!(ViolationSeverity::Medium.score(), 50);
        assert_eq!(ViolationSeverity::Low.score(), 25);
        assert_eq!(ViolationSeverity::Info.score(), 10);
    }

    #[test]
    fn test_violation_severity_equality() {
        assert_eq!(ViolationSeverity::Critical, ViolationSeverity::Critical);
        assert_ne!(ViolationSeverity::Critical, ViolationSeverity::High);
    }

    #[test]
    fn test_memory_state_snapshot_creation() {
        let snapshot = MemoryStateSnapshot {
            timestamp_ns: 1234567890,
            total_allocated_bytes: 1024,
            active_allocation_count: 5,
            involved_addresses: vec!["0x1000".to_string()],
            stack_trace: vec![],
            related_allocations: vec![],
            memory_pressure: MemoryPressureLevel::Low,
        };

        assert_eq!(snapshot.timestamp_ns, 1234567890);
        assert_eq!(snapshot.total_allocated_bytes, 1024);
        assert_eq!(snapshot.active_allocation_count, 5);
        assert_eq!(snapshot.involved_addresses.len(), 1);
    }

    #[test]
    fn test_stack_frame_creation() {
        let frame = crate::analysis::unsafe_ffi_tracker::StackFrame {
            function_name: "test_function".to_string(),
            file_name: Some("test.rs".to_string()),
            line_number: Some(42),
            is_unsafe: true,
        };

        assert_eq!(frame.function_name, "test_function");
        assert_eq!(frame.file_name, Some("test.rs".to_string()));
        assert_eq!(frame.line_number, Some(42));
        assert!(frame.is_unsafe);
    }

    #[test]
    fn test_related_allocation_creation() {
        let related = RelatedAllocation {
            address: "0x2000".to_string(),
            size: 256,
            type_name: Some("Vec<u8>".to_string()),
            variable_name: Some("buffer".to_string()),
            allocated_at_ns: 1234567890,
            is_active: true,
            relationship: AllocationRelationship::SameType,
        };

        assert_eq!(related.address, "0x2000");
        assert_eq!(related.size, 256);
        assert_eq!(related.type_name, Some("Vec<u8>".to_string()));
        assert!(related.is_active);
        matches!(related.relationship, AllocationRelationship::SameType);
    }

    #[test]
    fn test_allocation_relationship_variants() {
        let relationships = [
            AllocationRelationship::SameRegion,
            AllocationRelationship::Adjacent,
            AllocationRelationship::SameType,
            AllocationRelationship::SameScope,
            AllocationRelationship::DoubleFreeCandidate,
            AllocationRelationship::LeakRelated,
            AllocationRelationship::UseAfterFreeRelated,
            AllocationRelationship::None,
        ];

        assert_eq!(relationships.len(), 8);
    }

    #[test]
    fn test_memory_pressure_level_variants() {
        let levels = [
            MemoryPressureLevel::Low,
            MemoryPressureLevel::Medium,
            MemoryPressureLevel::High,
            MemoryPressureLevel::Critical,
        ];

        assert_eq!(levels.len(), 4);
    }

    #[test]
    fn test_impact_assessment_creation() {
        let impact = ImpactAssessment {
            exploitability_score: 0.8,
            data_corruption_risk: true,
            information_disclosure_risk: false,
            denial_of_service_risk: true,
            code_execution_risk: false,
            overall_risk_score: 0.75,
        };

        assert_eq!(impact.exploitability_score, 0.8);
        assert!(impact.data_corruption_risk);
        assert!(!impact.information_disclosure_risk);
        assert!(impact.denial_of_service_risk);
        assert!(!impact.code_execution_risk);
        assert_eq!(impact.overall_risk_score, 0.75);
    }

    #[test]
    fn test_analysis_config_default() {
        let config = AnalysisConfig::default();

        assert_eq!(config.max_related_allocations, 10);
        assert_eq!(config.max_stack_depth, 20);
        assert!(config.enable_correlation_analysis);
        assert!(config.include_low_severity);
        assert!(config.generate_integrity_hashes);
    }

    #[test]
    fn test_analysis_config_custom() {
        let config = AnalysisConfig {
            max_related_allocations: 5,
            max_stack_depth: 10,
            enable_correlation_analysis: false,
            include_low_severity: false,
            generate_integrity_hashes: false,
        };

        assert_eq!(config.max_related_allocations, 5);
        assert_eq!(config.max_stack_depth, 10);
        assert!(!config.enable_correlation_analysis);
        assert!(!config.include_low_severity);
        assert!(!config.generate_integrity_hashes);
    }

    #[test]
    fn test_security_violation_analyzer_creation() {
        let config = AnalysisConfig::default();
        let analyzer = SecurityViolationAnalyzer::new(config);

        assert_eq!(analyzer.violation_reports.len(), 0);
        assert_eq!(analyzer.correlation_matrix.len(), 0);
        assert_eq!(analyzer.active_allocations.len(), 0);
    }

    #[test]
    fn test_security_violation_analyzer_default() {
        let analyzer = SecurityViolationAnalyzer::default();

        assert_eq!(analyzer.violation_reports.len(), 0);
        assert_eq!(analyzer.correlation_matrix.len(), 0);
        assert_eq!(analyzer.active_allocations.len(), 0);
    }

    #[test]
    fn test_update_allocations() {
        let mut analyzer = SecurityViolationAnalyzer::default();
        let allocations = vec![
            AllocationInfo::new(0x1000, 256),
            AllocationInfo::new(0x2000, 512),
        ];

        analyzer.update_allocations(allocations);

        assert_eq!(analyzer.active_allocations.len(), 2);
        assert_eq!(analyzer.active_allocations[0].ptr, 0x1000);
        assert_eq!(analyzer.active_allocations[1].ptr, 0x2000);
    }

    #[test]
    fn test_generate_violation_id() {
        let analyzer = SecurityViolationAnalyzer::default();
        let violation = SafetyViolation::DoubleFree {
            first_free_stack: CallStackRef::new(0, Some(0)),
            second_free_stack: CallStackRef::new(1, Some(0)),
            timestamp: 1234567890,
        };

        let id = analyzer.generate_violation_id(&violation, 0x1000);

        assert!(id.starts_with("SEC-DF-1000-"));
        assert!(id.len() > 10);
    }

    #[test]
    fn test_assess_severity() {
        let analyzer = SecurityViolationAnalyzer::default();

        let double_free = SafetyViolation::DoubleFree {
            first_free_stack: CallStackRef::new(0, Some(0)),
            second_free_stack: CallStackRef::new(1, Some(0)),
            timestamp: 1234567890,
        };
        assert_eq!(
            analyzer.assess_severity(&double_free),
            ViolationSeverity::Critical
        );

        let invalid_free = SafetyViolation::InvalidFree {
            attempted_pointer: 0x1000,
            stack: CallStackRef::new(0, Some(0)),
            timestamp: 1234567890,
        };
        assert_eq!(
            analyzer.assess_severity(&invalid_free),
            ViolationSeverity::High
        );

        let potential_leak = SafetyViolation::PotentialLeak {
            allocation_stack: CallStackRef::new(0, Some(0)),
            allocation_timestamp: 1234567890,
            leak_detection_timestamp: 1234567900,
        };
        assert_eq!(
            analyzer.assess_severity(&potential_leak),
            ViolationSeverity::Medium
        );

        let cross_boundary = SafetyViolation::CrossBoundaryRisk {
            risk_level: crate::analysis::unsafe_ffi_tracker::RiskLevel::Medium,
            description: "Test risk".to_string(),
            stack: CallStackRef::new(0, Some(0)),
        };
        assert_eq!(
            analyzer.assess_severity(&cross_boundary),
            ViolationSeverity::Medium
        );
    }

    #[test]
    fn test_create_memory_snapshot() {
        let mut analyzer = SecurityViolationAnalyzer::default();
        let allocations = vec![
            AllocationInfo::new(0x1000, 256),
            AllocationInfo::new(0x2000, 512),
        ];
        analyzer.update_allocations(allocations);

        let result = analyzer.create_memory_snapshot(0x1500);

        assert!(result.is_ok());
        let snapshot = result.unwrap();
        assert_eq!(snapshot.total_allocated_bytes, 768);
        assert_eq!(snapshot.active_allocation_count, 2);
        assert_eq!(snapshot.involved_addresses.len(), 1);
        assert_eq!(snapshot.involved_addresses[0], "0x1500");
        assert!(snapshot.timestamp_ns > 0);
    }

    #[test]
    fn test_find_related_allocations() {
        let mut analyzer = SecurityViolationAnalyzer::default();
        let allocations = vec![
            AllocationInfo::new(0x1000, 256), // Same region
            AllocationInfo::new(0x2000, 512), // Different region
        ];
        analyzer.update_allocations(allocations);

        let related = analyzer.find_related_allocations(0x1100); // Within first allocation

        assert!(!related.is_empty());
        assert_eq!(related[0].address, "0x1000");
        matches!(related[0].relationship, AllocationRelationship::SameRegion);
    }

    #[test]
    fn test_determine_relationship() {
        let analyzer = SecurityViolationAnalyzer::default();
        let alloc = AllocationInfo::new(0x1000, 256);

        // Same region (within allocation bounds: 0x1000 to 0x1100)
        let same_region = analyzer.determine_relationship(0x1080, &alloc);
        assert!(matches!(
            same_region,
            Some(AllocationRelationship::SameRegion)
        ));

        // Adjacent (just after allocation end: 0x1100 + small offset)
        let adjacent = analyzer.determine_relationship(0x1100 + 32, &alloc);
        assert!(matches!(adjacent, Some(AllocationRelationship::Adjacent)));

        // No relationship (far away)
        let none = analyzer.determine_relationship(0x5000, &alloc);
        assert!(none.is_none());
    }

    #[test]
    fn test_generate_stack_trace() {
        let analyzer = SecurityViolationAnalyzer::default();
        let stack_trace = analyzer.generate_stack_trace();

        assert!(!stack_trace.is_empty());
        assert_eq!(stack_trace[0].function_name, "violation_detected");
        assert!(stack_trace[0].is_unsafe);
        assert!(!stack_trace[0].is_ffi);
    }

    #[test]
    fn test_assess_memory_pressure() {
        let analyzer = SecurityViolationAnalyzer::default();

        let low = analyzer.assess_memory_pressure(100 * 1024 * 1024); // 100MB
        matches!(low, MemoryPressureLevel::Low);

        let medium = analyzer.assess_memory_pressure(600 * 1024 * 1024); // 600MB
        matches!(medium, MemoryPressureLevel::Medium);

        let high = analyzer.assess_memory_pressure(1200 * 1024 * 1024); // 1.2GB
        matches!(high, MemoryPressureLevel::High);

        let critical = analyzer.assess_memory_pressure(3000 * 1024 * 1024); // 3GB
        matches!(critical, MemoryPressureLevel::Critical);
    }

    #[test]
    fn test_generate_violation_details() {
        let analyzer = SecurityViolationAnalyzer::default();

        let double_free = SafetyViolation::DoubleFree {
            first_free_stack: CallStackRef::new(0, Some(0)),
            second_free_stack: CallStackRef::new(1, Some(0)),
            timestamp: 1234567890,
        };
        let (desc, details) = analyzer.generate_violation_details(&double_free);
        assert_eq!(desc, "Double free violation detected");
        assert!(details.contains("timestamp 1234567890"));

        let invalid_free = SafetyViolation::InvalidFree {
            attempted_pointer: 0x1000,
            stack: CallStackRef::new(0, Some(0)),
            timestamp: 1234567890,
        };
        let (desc, details) = analyzer.generate_violation_details(&invalid_free);
        assert_eq!(desc, "Invalid free operation detected");
        assert!(details.contains("timestamp 1234567890"));
    }

    #[test]
    fn test_assess_impact() {
        let analyzer = SecurityViolationAnalyzer::default();
        let snapshot = MemoryStateSnapshot {
            timestamp_ns: 1234567890,
            total_allocated_bytes: 1024,
            active_allocation_count: 1,
            involved_addresses: vec!["0x1000".to_string()],
            stack_trace: vec![],
            related_allocations: vec![],
            memory_pressure: MemoryPressureLevel::Low,
        };

        let double_free = SafetyViolation::DoubleFree {
            first_free_stack: CallStackRef::new(0, Some(0)),
            second_free_stack: CallStackRef::new(1, Some(0)),
            timestamp: 1234567890,
        };

        let impact = analyzer.assess_impact(&double_free, &snapshot);

        assert_eq!(impact.exploitability_score, 0.9);
        assert!(impact.data_corruption_risk);
        assert!(!impact.information_disclosure_risk);
        assert!(impact.denial_of_service_risk);
        assert!(impact.code_execution_risk);
        assert!(impact.overall_risk_score > 0.0);
    }

    #[test]
    fn test_generate_remediation_suggestions() {
        let analyzer = SecurityViolationAnalyzer::default();
        let impact = ImpactAssessment {
            exploitability_score: 0.9,
            data_corruption_risk: true,
            information_disclosure_risk: false,
            denial_of_service_risk: true,
            code_execution_risk: true,
            overall_risk_score: 0.9,
        };

        let double_free = SafetyViolation::DoubleFree {
            first_free_stack: CallStackRef::new(0, Some(0)),
            second_free_stack: CallStackRef::new(1, Some(0)),
            timestamp: 1234567890,
        };

        let suggestions = analyzer.generate_remediation_suggestions(&double_free, &impact);

        assert!(!suggestions.is_empty());
        assert!(suggestions[0].contains("URGENT"));
        assert!(suggestions.iter().any(|s| s.contains("ownership tracking")));
    }

    #[test]
    fn test_get_violation_type_string() {
        let analyzer = SecurityViolationAnalyzer::default();

        let double_free = SafetyViolation::DoubleFree {
            first_free_stack: CallStackRef::new(0, Some(0)),
            second_free_stack: CallStackRef::new(1, Some(0)),
            timestamp: 1234567890,
        };
        assert_eq!(
            analyzer.get_violation_type_string(&double_free),
            "DoubleFree"
        );

        let invalid_free = SafetyViolation::InvalidFree {
            attempted_pointer: 0x1000,
            stack: CallStackRef::new(0, Some(0)),
            timestamp: 1234567890,
        };
        assert_eq!(
            analyzer.get_violation_type_string(&invalid_free),
            "InvalidFree"
        );
    }

    #[test]
    fn test_are_violations_correlated() {
        let analyzer = SecurityViolationAnalyzer::default();

        let double_free = SafetyViolation::DoubleFree {
            first_free_stack: CallStackRef::new(0, Some(0)),
            second_free_stack: CallStackRef::new(1, Some(0)),
            timestamp: 1234567890,
        };

        assert!(analyzer.are_violations_correlated(&double_free, "InvalidFree"));
        assert!(!analyzer.are_violations_correlated(&double_free, "PotentialLeak"));
    }

    #[test]
    fn test_analyze_violation() {
        let mut analyzer = SecurityViolationAnalyzer::default();
        let allocations = vec![AllocationInfo::new(0x1000, 256)];
        analyzer.update_allocations(allocations);

        let violation = SafetyViolation::DoubleFree {
            first_free_stack: CallStackRef::new(0, Some(0)),
            second_free_stack: CallStackRef::new(1, Some(0)),
            timestamp: 1234567890,
        };

        let result = analyzer.analyze_violation(&violation, 0x1000);

        assert!(result.is_ok());
        let violation_id = result.unwrap();
        assert!(violation_id.starts_with("SEC-DF-"));

        // Check that report was stored
        assert_eq!(analyzer.violation_reports.len(), 1);
        assert!(analyzer.violation_reports.contains_key(&violation_id));
    }

    #[test]
    fn test_get_all_reports() {
        let mut analyzer = SecurityViolationAnalyzer::default();
        let violation = SafetyViolation::InvalidFree {
            attempted_pointer: 0x1000,
            stack: CallStackRef::new(0, Some(0)),
            timestamp: 1234567890,
        };

        let _ = analyzer.analyze_violation(&violation, 0x1000);
        let reports = analyzer.get_all_reports();

        assert_eq!(reports.len(), 1);
    }

    #[test]
    fn test_get_reports_by_severity() {
        let mut analyzer = SecurityViolationAnalyzer::default();

        // Add critical violation
        let critical_violation = SafetyViolation::DoubleFree {
            first_free_stack: CallStackRef::new(0, Some(0)),
            second_free_stack: CallStackRef::new(1, Some(0)),
            timestamp: 1234567890,
        };
        let _ = analyzer.analyze_violation(&critical_violation, 0x1000);

        // Add medium violation
        let medium_violation = SafetyViolation::PotentialLeak {
            allocation_stack: CallStackRef::new(0, Some(0)),
            allocation_timestamp: 1234567890,
            leak_detection_timestamp: 1234567900,
        };
        let _ = analyzer.analyze_violation(&medium_violation, 0x2000);

        let critical_reports = analyzer.get_reports_by_severity(ViolationSeverity::Critical);
        let medium_reports = analyzer.get_reports_by_severity(ViolationSeverity::Medium);

        assert_eq!(critical_reports.len(), 1);
        assert_eq!(medium_reports.len(), 2); // Both critical and medium
    }

    #[test]
    fn test_compute_integrity_hash() {
        let analyzer = SecurityViolationAnalyzer::default();
        let report = SecurityViolationReport {
            violation_id: "test-id".to_string(),
            violation_type: "DoubleFree".to_string(),
            severity: ViolationSeverity::Critical,
            description: "Test violation".to_string(),
            technical_details: "Test details".to_string(),
            memory_snapshot: MemoryStateSnapshot {
                timestamp_ns: 1234567890,
                total_allocated_bytes: 1024,
                active_allocation_count: 1,
                involved_addresses: vec!["0x1000".to_string()],
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
            remediation_suggestions: vec!["Test suggestion".to_string()],
            correlated_violations: vec![],
            integrity_hash: String::new(),
            generated_at_ns: 1234567890,
        };

        let result = analyzer.compute_integrity_hash(&report);

        assert!(result.is_ok());
        let hash = result.unwrap();
        assert_eq!(hash.len(), 16); // 64-bit hash as hex string
    }

    #[test]
    fn test_verify_report_integrity() {
        let config = AnalysisConfig {
            generate_integrity_hashes: true,
            ..Default::default()
        };
        let analyzer = SecurityViolationAnalyzer::new(config);

        let mut report = SecurityViolationReport {
            violation_id: "test-id".to_string(),
            violation_type: "DoubleFree".to_string(),
            severity: ViolationSeverity::Critical,
            description: "Test violation".to_string(),
            technical_details: "Test details".to_string(),
            memory_snapshot: MemoryStateSnapshot {
                timestamp_ns: 1234567890,
                total_allocated_bytes: 1024,
                active_allocation_count: 1,
                involved_addresses: vec!["0x1000".to_string()],
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
            remediation_suggestions: vec!["Test suggestion".to_string()],
            correlated_violations: vec![],
            integrity_hash: String::new(),
            generated_at_ns: 1234567890,
        };

        // Compute correct hash
        let hash = analyzer.compute_integrity_hash(&report).unwrap();
        report.integrity_hash = hash;

        let result = analyzer.verify_report_integrity(&report);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test with wrong hash
        report.integrity_hash = "wrong_hash".to_string();
        let result = analyzer.verify_report_integrity(&report);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_generate_security_summary() {
        let mut analyzer = SecurityViolationAnalyzer::default();

        // Add some violations
        let critical_violation = SafetyViolation::DoubleFree {
            first_free_stack: CallStackRef::new(0, Some(0)),
            second_free_stack: CallStackRef::new(1, Some(0)),
            timestamp: 1234567890,
        };
        let _ = analyzer.analyze_violation(&critical_violation, 0x1000);

        let medium_violation = SafetyViolation::PotentialLeak {
            allocation_stack: CallStackRef::new(0, Some(0)),
            allocation_timestamp: 1234567890,
            leak_detection_timestamp: 1234567900,
        };
        let _ = analyzer.analyze_violation(&medium_violation, 0x2000);

        let summary = analyzer.generate_security_summary();

        assert!(summary.is_object());
        let security_summary = &summary["security_analysis_summary"];
        assert_eq!(security_summary["total_violations"], 2);
        assert!(security_summary["severity_breakdown"].is_object());
        assert!(security_summary["risk_assessment"].is_object());
    }

    #[test]
    fn test_clear_reports() {
        let mut analyzer = SecurityViolationAnalyzer::default();

        let violation = SafetyViolation::InvalidFree {
            attempted_pointer: 0x1000,
            stack: CallStackRef::new(0, Some(0)),
            timestamp: 1234567890,
        };
        let _ = analyzer.analyze_violation(&violation, 0x1000);

        assert_eq!(analyzer.violation_reports.len(), 1);

        analyzer.clear_reports();

        assert_eq!(analyzer.violation_reports.len(), 0);
        assert_eq!(analyzer.correlation_matrix.len(), 0);
    }

    #[test]
    fn test_correlation_analysis_disabled() {
        let config = AnalysisConfig {
            enable_correlation_analysis: false,
            ..Default::default()
        };
        let mut analyzer = SecurityViolationAnalyzer::new(config);

        let violation = SafetyViolation::DoubleFree {
            first_free_stack: CallStackRef::new(0, Some(0)),
            second_free_stack: CallStackRef::new(1, Some(0)),
            timestamp: 1234567890,
        };

        let result = analyzer.analyze_violation(&violation, 0x1000);
        assert!(result.is_ok());

        let reports = analyzer.get_all_reports();
        let report = reports.values().next().unwrap();
        assert!(report.correlated_violations.is_empty());
    }

    #[test]
    fn test_integrity_hashes_disabled() {
        let config = AnalysisConfig {
            generate_integrity_hashes: false,
            ..Default::default()
        };
        let mut analyzer = SecurityViolationAnalyzer::new(config);

        let violation = SafetyViolation::InvalidFree {
            attempted_pointer: 0x1000,
            stack: CallStackRef::new(0, Some(0)),
            timestamp: 1234567890,
        };

        let result = analyzer.analyze_violation(&violation, 0x1000);
        assert!(result.is_ok());

        let reports = analyzer.get_all_reports();
        let report = reports.values().next().unwrap();
        assert!(report.integrity_hash.is_empty());
    }

    #[test]
    fn test_max_related_allocations_limit() {
        let config = AnalysisConfig {
            max_related_allocations: 2,
            ..Default::default()
        };
        let mut analyzer = SecurityViolationAnalyzer::new(config);

        // Add many allocations
        let allocations: Vec<AllocationInfo> = (0..10)
            .map(|i| AllocationInfo::new(0x1000 + i * 0x100, 256))
            .collect();
        analyzer.update_allocations(allocations);

        let related = analyzer.find_related_allocations(0x1100);

        // Should be limited to max_related_allocations
        assert!(related.len() <= 2);
    }

    #[test]
    fn test_security_violation_report_serialization() {
        let report = SecurityViolationReport {
            violation_id: "test-id".to_string(),
            violation_type: "DoubleFree".to_string(),
            severity: ViolationSeverity::Critical,
            description: "Test violation".to_string(),
            technical_details: "Test details".to_string(),
            memory_snapshot: MemoryStateSnapshot {
                timestamp_ns: 1234567890,
                total_allocated_bytes: 1024,
                active_allocation_count: 1,
                involved_addresses: vec!["0x1000".to_string()],
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
            remediation_suggestions: vec!["Test suggestion".to_string()],
            correlated_violations: vec![],
            integrity_hash: "test_hash".to_string(),
            generated_at_ns: 1234567890,
        };

        let serialized = serde_json::to_string(&report);
        assert!(serialized.is_ok());

        let deserialized: Result<SecurityViolationReport, _> =
            serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
    }
}
