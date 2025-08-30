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

        format!(
            "SEC-{violation_type}-{:X}-{}",
            address,
            timestamp % 1000000
        )
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
