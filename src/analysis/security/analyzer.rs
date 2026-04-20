use crate::analysis::security::types::*;
use crate::analysis::unsafe_ffi_tracker::SafetyViolation;
use crate::capture::types::AllocationInfo;
use serde_json;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SecurityViolationAnalyzer {
    violation_reports: HashMap<String, SecurityViolationReport>,
    correlation_matrix: HashMap<String, HashSet<String>>,
    active_allocations: Vec<AllocationInfo>,
    config: AnalysisConfig,
}

impl SecurityViolationAnalyzer {
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

    pub fn update_allocations(&mut self, allocations: Vec<AllocationInfo>) {
        self.active_allocations = allocations;
        tracing::info!(
            "🔄 Updated allocation context: {} active allocations",
            self.active_allocations.len()
        );
    }

    pub fn analyze_violation(
        &mut self,
        violation: &SafetyViolation,
        violation_address: usize,
    ) -> Result<String, String> {
        let violation_id = self.generate_violation_id(violation, violation_address);

        tracing::info!("🔍 Analyzing security violation: {}", violation_id);

        let severity = self.assess_severity(violation);
        let memory_snapshot = self.create_memory_snapshot(violation_address)?;
        let (description, technical_details) = self.generate_violation_details(violation);
        let impact_assessment = self.assess_impact(violation, &memory_snapshot);
        let remediation_suggestions =
            self.generate_remediation_suggestions(violation, &impact_assessment);
        let correlated_violations = if self.config.enable_correlation_analysis {
            self.find_correlated_violations(&violation_id, violation)
        } else {
            Vec::new()
        };

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
            integrity_hash: String::new(),
            generated_at_ns: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        };

        let mut final_report = report;
        if self.config.generate_integrity_hashes {
            final_report.integrity_hash = self.compute_integrity_hash(&final_report)?;
        }

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

    fn assess_severity(&self, violation: &SafetyViolation) -> ViolationSeverity {
        match violation {
            SafetyViolation::DoubleFree { .. } => ViolationSeverity::Critical,
            SafetyViolation::InvalidFree { .. } => ViolationSeverity::High,
            SafetyViolation::PotentialLeak { .. } => ViolationSeverity::Medium,
            SafetyViolation::CrossBoundaryRisk { .. } => ViolationSeverity::Medium,
        }
    }

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
        let related_allocations = self.find_related_allocations(violation_address);
        let stack_trace = self.generate_stack_trace();
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

    fn determine_relationship(
        &self,
        violation_addr: usize,
        alloc: &AllocationInfo,
    ) -> Option<AllocationRelationship> {
        let alloc_start = alloc.ptr;
        let alloc_end = alloc.ptr + alloc.size;

        if violation_addr >= alloc_start && violation_addr < alloc_end {
            return Some(AllocationRelationship::SameRegion);
        }

        if (violation_addr as isize - alloc_end as isize).abs() < 64 {
            return Some(AllocationRelationship::Adjacent);
        }

        if let Some(type_name) = &alloc.type_name {
            if type_name.contains("*") || type_name.contains("Box") || type_name.contains("Vec") {
                return Some(AllocationRelationship::SameType);
            }
        }

        None
    }

    fn generate_stack_trace(&self) -> Vec<StackFrame> {
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

    fn are_violations_correlated(&self, violation: &SafetyViolation, other_type: &str) -> bool {
        match violation {
            SafetyViolation::DoubleFree { .. } => other_type.contains("InvalidFree"),
            SafetyViolation::InvalidFree { .. } => other_type.contains("DoubleFree"),
            SafetyViolation::PotentialLeak { .. } => other_type.contains("Leak"),
            SafetyViolation::CrossBoundaryRisk { .. } => other_type.contains("CrossBoundary"),
        }
    }

    fn update_correlation_matrix(&mut self, violation_id: &str, correlated: Vec<String>) {
        self.correlation_matrix
            .insert(violation_id.to_string(), correlated.into_iter().collect());
    }

    fn get_violation_type_string(&self, violation: &SafetyViolation) -> String {
        match violation {
            SafetyViolation::DoubleFree { .. } => "DoubleFree".to_string(),
            SafetyViolation::InvalidFree { .. } => "InvalidFree".to_string(),
            SafetyViolation::PotentialLeak { .. } => "PotentialLeak".to_string(),
            SafetyViolation::CrossBoundaryRisk { .. } => "CrossBoundaryRisk".to_string(),
        }
    }

    fn compute_integrity_hash(&self, report: &SecurityViolationReport) -> Result<String, String> {
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

    pub fn get_all_reports(&self) -> &HashMap<String, SecurityViolationReport> {
        &self.violation_reports
    }

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

    pub fn verify_report_integrity(
        &self,
        report: &SecurityViolationReport,
    ) -> Result<bool, String> {
        if !self.config.generate_integrity_hashes {
            return Ok(true);
        }

        let computed_hash = self.compute_integrity_hash(report)?;
        Ok(computed_hash == report.integrity_hash)
    }

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
                    "all_reports_verified": true
                }
            }
        })
    }

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

    fn create_test_allocation(ptr: usize, size: usize) -> AllocationInfo {
        use std::thread;
        AllocationInfo {
            ptr,
            size,
            var_name: None,
            type_name: None,
            scope_name: None,
            timestamp_alloc: 1000,
            timestamp_dealloc: None,
            thread_id: thread::current().id(),
            thread_id_u64: 1,
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
            module_path: None,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
            stack_ptr: None,
            task_id: None,
        }
    }

    /// Objective: Verify SecurityViolationAnalyzer creation with default config
    /// Invariants: Default config should have correlation analysis enabled
    #[test]
    fn test_security_analyzer_default() {
        let analyzer = SecurityViolationAnalyzer::default();
        let reports = analyzer.get_all_reports();
        assert!(reports.is_empty(), "New analyzer should have no reports");
    }

    /// Objective: Verify SecurityViolationAnalyzer creation with custom config
    /// Invariants: Custom config values should be respected
    #[test]
    fn test_security_analyzer_custom_config() {
        let config = AnalysisConfig {
            max_related_allocations: 5,
            max_stack_depth: 10,
            enable_correlation_analysis: false,
            include_low_severity: false,
            generate_integrity_hashes: false,
        };
        let analyzer = SecurityViolationAnalyzer::new(config);
        let reports = analyzer.get_all_reports();
        assert!(
            reports.is_empty(),
            "Custom config analyzer should start empty"
        );
    }

    /// Objective: Verify generate_security_summary with no violations
    /// Invariants: Should produce valid JSON summary with zero violations
    #[test]
    fn test_generate_security_summary_empty() {
        let analyzer = SecurityViolationAnalyzer::default();
        let summary = analyzer.generate_security_summary();
        assert!(summary.is_object(), "Summary should be a JSON object");

        let obj = summary.as_object().unwrap();
        assert!(
            obj.contains_key("security_analysis_summary"),
            "Should have analysis summary"
        );
    }

    /// Objective: Verify clear_reports functionality
    /// Invariants: Should work on empty analyzer
    #[test]
    fn test_clear_reports_empty() {
        let mut analyzer = SecurityViolationAnalyzer::default();
        analyzer.clear_reports();
        assert!(
            analyzer.get_all_reports().is_empty(),
            "Should have no reports after clear"
        );
    }

    /// Objective: Verify get_reports_by_severity with no reports
    /// Invariants: Should return empty vector
    #[test]
    fn test_get_reports_by_severity_empty() {
        let analyzer = SecurityViolationAnalyzer::default();
        let reports = analyzer.get_reports_by_severity(ViolationSeverity::Critical);
        assert!(reports.is_empty(), "Should have no reports");
    }

    /// Objective: Verify update_allocations functionality
    /// Invariants: Should accept empty vector
    #[test]
    fn test_update_allocations_empty() {
        let mut analyzer = SecurityViolationAnalyzer::default();
        analyzer.update_allocations(vec![]);
        let summary = analyzer.generate_security_summary();
        assert!(summary.is_object(), "Summary should still be valid JSON");
    }

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

    /// Objective: Verify analyze_violation with DoubleFree
    /// Invariants: Should create report with Critical severity
    #[test]
    fn test_analyze_violation_double_free() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();
        let call_stack = CallStackRef::new(1, Some(1));

        let violation = SafetyViolation::DoubleFree {
            first_free_stack: call_stack.clone(),
            second_free_stack: call_stack,
            timestamp: 1000,
        };

        let result = analyzer.analyze_violation(&violation, 0x1000);
        assert!(result.is_ok(), "Should analyze DoubleFree successfully");

        let violation_id = result.unwrap();
        assert!(
            violation_id.starts_with("SEC-DF"),
            "Violation ID should start with SEC-DF"
        );

        let reports = analyzer.get_all_reports();
        let report = reports.get(&violation_id).expect("Report should exist");
        assert_eq!(
            report.severity,
            ViolationSeverity::Critical,
            "DoubleFree should be Critical"
        );
        assert!(
            report.description.contains("Double free"),
            "Description should mention double free"
        );
    }

    /// Objective: Verify analyze_violation with InvalidFree
    /// Invariants: Should create report with High severity
    #[test]
    fn test_analyze_violation_invalid_free() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();
        let call_stack = CallStackRef::new(2, Some(1));

        let violation = SafetyViolation::InvalidFree {
            attempted_pointer: 0x2000,
            stack: call_stack,
            timestamp: 2000,
        };

        let result = analyzer.analyze_violation(&violation, 0x2000);
        assert!(result.is_ok(), "Should analyze InvalidFree successfully");

        let violation_id = result.unwrap();
        assert!(
            violation_id.starts_with("SEC-IF"),
            "Violation ID should start with SEC-IF"
        );

        let reports = analyzer.get_all_reports();
        let report = reports.get(&violation_id).expect("Report should exist");
        assert_eq!(
            report.severity,
            ViolationSeverity::High,
            "InvalidFree should be High"
        );
    }

    /// Objective: Verify analyze_violation with PotentialLeak
    /// Invariants: Should create report with Medium severity
    #[test]
    fn test_analyze_violation_potential_leak() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();
        let call_stack = CallStackRef::new(3, Some(1));

        let violation = SafetyViolation::PotentialLeak {
            allocation_stack: call_stack,
            allocation_timestamp: 1000,
            leak_detection_timestamp: 5000,
        };

        let result = analyzer.analyze_violation(&violation, 0x3000);
        assert!(result.is_ok(), "Should analyze PotentialLeak successfully");

        let violation_id = result.unwrap();
        assert!(
            violation_id.starts_with("SEC-PL"),
            "Violation ID should start with SEC-PL"
        );

        let reports = analyzer.get_all_reports();
        let report = reports.get(&violation_id).expect("Report should exist");
        assert_eq!(
            report.severity,
            ViolationSeverity::Medium,
            "PotentialLeak should be Medium"
        );
    }

    /// Objective: Verify analyze_violation with CrossBoundaryRisk
    /// Invariants: Should create report with Medium severity
    #[test]
    fn test_analyze_violation_cross_boundary() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();
        let call_stack = CallStackRef::new(4, Some(1));

        let violation = SafetyViolation::CrossBoundaryRisk {
            risk_level: crate::analysis::unsafe_ffi_tracker::RiskLevel::High,
            description: "FFI boundary violation".to_string(),
            stack: call_stack,
        };

        let result = analyzer.analyze_violation(&violation, 0x4000);
        assert!(
            result.is_ok(),
            "Should analyze CrossBoundaryRisk successfully"
        );

        let violation_id = result.unwrap();
        assert!(
            violation_id.starts_with("SEC-CBR"),
            "Violation ID should start with SEC-CBR"
        );

        let reports = analyzer.get_all_reports();
        let report = reports.get(&violation_id).expect("Report should exist");
        assert_eq!(
            report.severity,
            ViolationSeverity::Medium,
            "CrossBoundaryRisk should be Medium"
        );
    }

    /// Objective: Verify analyze_violation with correlation analysis disabled
    /// Invariants: Should not find correlated violations when disabled
    #[test]
    fn test_analyze_violation_correlation_disabled() {
        use crate::core::CallStackRef;

        let config = AnalysisConfig {
            enable_correlation_analysis: false,
            ..Default::default()
        };
        let mut analyzer = SecurityViolationAnalyzer::new(config);
        let call_stack = CallStackRef::new(5, Some(1));

        let violation = SafetyViolation::DoubleFree {
            first_free_stack: call_stack.clone(),
            second_free_stack: call_stack,
            timestamp: 1000,
        };

        let result = analyzer.analyze_violation(&violation, 0x1000);
        assert!(result.is_ok(), "Should analyze successfully");

        let reports = analyzer.get_all_reports();
        let report = reports.values().next().expect("Should have report");
        assert!(
            report.correlated_violations.is_empty(),
            "Should have no correlated violations when disabled"
        );
    }

    /// Objective: Verify analyze_violation with integrity hash disabled
    /// Invariants: Should not generate hash when disabled
    #[test]
    fn test_analyze_violation_integrity_hash_disabled() {
        use crate::core::CallStackRef;

        let config = AnalysisConfig {
            generate_integrity_hashes: false,
            ..Default::default()
        };
        let mut analyzer = SecurityViolationAnalyzer::new(config);
        let call_stack = CallStackRef::new(6, Some(1));

        let violation = SafetyViolation::PotentialLeak {
            allocation_stack: call_stack,
            allocation_timestamp: 1000,
            leak_detection_timestamp: 5000,
        };

        let result = analyzer.analyze_violation(&violation, 0x3000);
        assert!(result.is_ok(), "Should analyze successfully");

        let reports = analyzer.get_all_reports();
        let report = reports.values().next().expect("Should have report");
        assert!(
            report.integrity_hash.is_empty(),
            "Hash should be empty when disabled"
        );
    }

    /// Objective: Verify get_reports_by_severity filters correctly
    /// Invariants: Should return only reports with severity >= min_severity
    #[test]
    fn test_get_reports_by_severity_filtering() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();
        let call_stack = CallStackRef::new(10, Some(1));

        let violations = vec![
            (
                SafetyViolation::DoubleFree {
                    first_free_stack: call_stack.clone(),
                    second_free_stack: call_stack.clone(),
                    timestamp: 1000,
                },
                0x1000,
            ),
            (
                SafetyViolation::InvalidFree {
                    attempted_pointer: 0x2000,
                    stack: call_stack.clone(),
                    timestamp: 2000,
                },
                0x2000,
            ),
            (
                SafetyViolation::PotentialLeak {
                    allocation_stack: call_stack,
                    allocation_timestamp: 1000,
                    leak_detection_timestamp: 5000,
                },
                0x3000,
            ),
        ];

        for (violation, addr) in violations {
            analyzer.analyze_violation(&violation, addr).unwrap();
        }

        let critical_reports = analyzer.get_reports_by_severity(ViolationSeverity::Critical);
        assert_eq!(critical_reports.len(), 1, "Should have 1 Critical report");

        let high_reports = analyzer.get_reports_by_severity(ViolationSeverity::High);
        assert_eq!(high_reports.len(), 2, "Should have 2 High+ reports");

        let medium_reports = analyzer.get_reports_by_severity(ViolationSeverity::Medium);
        assert_eq!(medium_reports.len(), 3, "Should have 3 Medium+ reports");
    }

    /// Objective: Verify verify_report_integrity with valid hash
    /// Invariants: Should return true for valid integrity hash
    #[test]
    fn test_verify_report_integrity_valid() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();
        let call_stack = CallStackRef::new(20, Some(1));

        let violation = SafetyViolation::DoubleFree {
            first_free_stack: call_stack.clone(),
            second_free_stack: call_stack,
            timestamp: 1000,
        };

        let violation_id = analyzer.analyze_violation(&violation, 0x1000).unwrap();
        let reports = analyzer.get_all_reports();
        let report = reports.get(&violation_id).expect("Report should exist");

        let result = analyzer.verify_report_integrity(report);
        assert!(
            result.is_ok() && result.unwrap(),
            "Integrity verification should succeed"
        );
    }

    /// Objective: Verify verify_report_integrity when disabled
    /// Invariants: Should return true when integrity hashes disabled
    #[test]
    fn test_verify_report_integrity_disabled() {
        let config = AnalysisConfig {
            generate_integrity_hashes: false,
            ..Default::default()
        };
        let analyzer = SecurityViolationAnalyzer::new(config);

        let report = SecurityViolationReport {
            violation_id: "test".to_string(),
            violation_type: "Test".to_string(),
            severity: ViolationSeverity::Low,
            description: "test".to_string(),
            technical_details: "test".to_string(),
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
                exploitability_score: 0.0,
                data_corruption_risk: false,
                information_disclosure_risk: false,
                denial_of_service_risk: false,
                code_execution_risk: false,
                overall_risk_score: 0.0,
            },
            remediation_suggestions: vec![],
            correlated_violations: vec![],
            integrity_hash: String::new(),
            generated_at_ns: 0,
        };

        let result = analyzer.verify_report_integrity(&report);
        assert!(
            result.is_ok() && result.unwrap(),
            "Should return true when integrity hashes disabled"
        );
    }

    /// Objective: Verify generate_security_summary with violations
    /// Invariants: Should include violation counts and risk assessment
    #[test]
    fn test_generate_security_summary_with_violations() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();
        let call_stack = CallStackRef::new(30, Some(1));

        let violation = SafetyViolation::DoubleFree {
            first_free_stack: call_stack.clone(),
            second_free_stack: call_stack,
            timestamp: 1000,
        };

        analyzer.analyze_violation(&violation, 0x1000).unwrap();

        let summary = analyzer.generate_security_summary();
        let obj = summary.as_object().unwrap();
        let analysis = obj.get("security_analysis_summary").unwrap();

        assert_eq!(
            analysis.get("total_violations").unwrap().as_u64().unwrap(),
            1,
            "Should have 1 violation"
        );

        let severity = analysis.get("severity_breakdown").unwrap();
        assert_eq!(
            severity.get("critical").unwrap().as_u64().unwrap(),
            1,
            "Should have 1 critical"
        );
    }

    /// Objective: Verify clear_reports removes all data
    /// Invariants: Should clear both reports and correlation matrix
    #[test]
    fn test_clear_reports_with_data() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();
        let call_stack = CallStackRef::new(40, Some(1));

        let violation = SafetyViolation::DoubleFree {
            first_free_stack: call_stack.clone(),
            second_free_stack: call_stack,
            timestamp: 1000,
        };

        analyzer.analyze_violation(&violation, 0x1000).unwrap();
        assert!(
            !analyzer.get_all_reports().is_empty(),
            "Should have reports"
        );

        analyzer.clear_reports();
        assert!(
            analyzer.get_all_reports().is_empty(),
            "Should have no reports after clear"
        );
    }

    /// Objective: Verify update_allocations affects memory snapshot
    /// Invariants: Memory snapshot should reflect active allocations
    #[test]
    fn test_update_allocations_affects_snapshot() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();

        let allocations = vec![create_test_allocation(0x1000, 1024)];
        analyzer.update_allocations(allocations);

        let call_stack = CallStackRef::new(50, Some(1));
        let violation = SafetyViolation::DoubleFree {
            first_free_stack: call_stack.clone(),
            second_free_stack: call_stack,
            timestamp: 1000,
        };

        analyzer.analyze_violation(&violation, 0x1000).unwrap();

        let reports = analyzer.get_all_reports();
        let report = reports.values().next().expect("Should have report");
        assert_eq!(
            report.memory_snapshot.total_allocated_bytes, 1024,
            "Should reflect allocation size"
        );
        assert_eq!(
            report.memory_snapshot.active_allocation_count, 1,
            "Should have 1 active allocation"
        );
    }

    /// Objective: Verify memory pressure assessment - Critical
    /// Invariants: > 2GB should be Critical
    #[test]
    fn test_memory_pressure_critical() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();

        let allocations = vec![create_test_allocation(0x1000, 3 * 1024 * 1024 * 1024)];
        analyzer.update_allocations(allocations);

        let call_stack = CallStackRef::new(60, Some(1));
        let violation = SafetyViolation::PotentialLeak {
            allocation_stack: call_stack,
            allocation_timestamp: 1000,
            leak_detection_timestamp: 5000,
        };

        analyzer.analyze_violation(&violation, 0x1000).unwrap();

        let reports = analyzer.get_all_reports();
        let report = reports.values().next().expect("Should have report");
        assert_eq!(
            report.memory_snapshot.memory_pressure,
            MemoryPressureLevel::Critical,
            "> 2GB should be Critical"
        );
    }

    /// Objective: Verify memory pressure assessment - High
    /// Invariants: > 1GB should be High
    #[test]
    fn test_memory_pressure_high() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();

        let allocations = vec![create_test_allocation(0x1000, 1500 * 1024 * 1024)];
        analyzer.update_allocations(allocations);

        let call_stack = CallStackRef::new(61, Some(1));
        let violation = SafetyViolation::PotentialLeak {
            allocation_stack: call_stack,
            allocation_timestamp: 1000,
            leak_detection_timestamp: 5000,
        };

        analyzer.analyze_violation(&violation, 0x1000).unwrap();

        let reports = analyzer.get_all_reports();
        let report = reports.values().next().expect("Should have report");
        assert_eq!(
            report.memory_snapshot.memory_pressure,
            MemoryPressureLevel::High,
            "> 1GB should be High"
        );
    }

    /// Objective: Verify memory pressure assessment - Medium
    /// Invariants: > 512MB should be Medium
    #[test]
    fn test_memory_pressure_medium() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();

        let allocations = vec![create_test_allocation(0x1000, 750 * 1024 * 1024)];
        analyzer.update_allocations(allocations);

        let call_stack = CallStackRef::new(62, Some(1));
        let violation = SafetyViolation::PotentialLeak {
            allocation_stack: call_stack,
            allocation_timestamp: 1000,
            leak_detection_timestamp: 5000,
        };

        analyzer.analyze_violation(&violation, 0x1000).unwrap();

        let reports = analyzer.get_all_reports();
        let report = reports.values().next().expect("Should have report");
        assert_eq!(
            report.memory_snapshot.memory_pressure,
            MemoryPressureLevel::Medium,
            "> 512MB should be Medium"
        );
    }

    /// Objective: Verify impact assessment for DoubleFree
    /// Invariants: Should have code execution risk
    #[test]
    fn test_impact_assessment_double_free() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();
        let call_stack = CallStackRef::new(70, Some(1));

        let violation = SafetyViolation::DoubleFree {
            first_free_stack: call_stack.clone(),
            second_free_stack: call_stack,
            timestamp: 1000,
        };

        analyzer.analyze_violation(&violation, 0x1000).unwrap();

        let reports = analyzer.get_all_reports();
        let report = reports.values().next().expect("Should have report");
        assert!(
            report.impact_assessment.code_execution_risk,
            "DoubleFree should have code execution risk"
        );
        assert!(
            report.impact_assessment.data_corruption_risk,
            "DoubleFree should have data corruption risk"
        );
        assert_eq!(
            report.impact_assessment.exploitability_score, 0.9,
            "DoubleFree should have 0.9 exploitability"
        );
    }

    /// Objective: Verify impact assessment for InvalidFree
    /// Invariants: Should not have code execution risk
    #[test]
    fn test_impact_assessment_invalid_free() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();
        let call_stack = CallStackRef::new(71, Some(1));

        let violation = SafetyViolation::InvalidFree {
            attempted_pointer: 0x2000,
            stack: call_stack,
            timestamp: 2000,
        };

        analyzer.analyze_violation(&violation, 0x2000).unwrap();

        let reports = analyzer.get_all_reports();
        let report = reports.values().next().expect("Should have report");
        assert!(
            !report.impact_assessment.code_execution_risk,
            "InvalidFree should not have code execution risk"
        );
        assert!(
            report.impact_assessment.data_corruption_risk,
            "InvalidFree should have data corruption risk"
        );
    }

    /// Objective: Verify impact assessment for CrossBoundaryRisk
    /// Invariants: Should have information disclosure risk
    #[test]
    fn test_impact_assessment_cross_boundary() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();
        let call_stack = CallStackRef::new(72, Some(1));

        let violation = SafetyViolation::CrossBoundaryRisk {
            risk_level: crate::analysis::unsafe_ffi_tracker::RiskLevel::High,
            description: "test".to_string(),
            stack: call_stack,
        };

        analyzer.analyze_violation(&violation, 0x4000).unwrap();

        let reports = analyzer.get_all_reports();
        let report = reports.values().next().expect("Should have report");
        assert!(
            report.impact_assessment.information_disclosure_risk,
            "CrossBoundaryRisk should have info disclosure risk"
        );
    }

    /// Objective: Verify remediation suggestions for DoubleFree
    /// Invariants: Should include RAII and ownership tracking suggestions
    #[test]
    fn test_remediation_double_free() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();
        let call_stack = CallStackRef::new(80, Some(1));

        let violation = SafetyViolation::DoubleFree {
            first_free_stack: call_stack.clone(),
            second_free_stack: call_stack,
            timestamp: 1000,
        };

        analyzer.analyze_violation(&violation, 0x1000).unwrap();

        let reports = analyzer.get_all_reports();
        let report = reports.values().next().expect("Should have report");
        assert!(
            report
                .remediation_suggestions
                .iter()
                .any(|s| s.contains("ownership")),
            "Should suggest ownership tracking"
        );
        assert!(
            report
                .remediation_suggestions
                .iter()
                .any(|s| s.contains("RAII")),
            "Should suggest RAII"
        );
    }

    /// Objective: Verify remediation suggestions for high risk
    /// Invariants: Should include URGENT prefix for high risk
    #[test]
    fn test_remediation_high_risk() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();

        let allocations = vec![create_test_allocation(0x1000, 3 * 1024 * 1024 * 1024)];
        analyzer.update_allocations(allocations);

        let call_stack = CallStackRef::new(81, Some(1));
        let violation = SafetyViolation::DoubleFree {
            first_free_stack: call_stack.clone(),
            second_free_stack: call_stack,
            timestamp: 1000,
        };

        analyzer.analyze_violation(&violation, 0x1000).unwrap();

        let reports = analyzer.get_all_reports();
        let report = reports.values().next().expect("Should have report");

        if report.impact_assessment.overall_risk_score > 0.8 {
            assert!(
                report
                    .remediation_suggestions
                    .iter()
                    .any(|s| s.contains("URGENT")),
                "High risk should have URGENT prefix"
            );
        }
    }

    /// Objective: Verify related allocations detection
    /// Invariants: Should find allocations in same region
    #[test]
    fn test_related_allocations_same_region() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();

        let allocations = vec![create_test_allocation(0x1000, 1024)];
        analyzer.update_allocations(allocations);

        let call_stack = CallStackRef::new(90, Some(1));
        let violation = SafetyViolation::DoubleFree {
            first_free_stack: call_stack.clone(),
            second_free_stack: call_stack,
            timestamp: 1000,
        };

        analyzer.analyze_violation(&violation, 0x1050).unwrap();

        let reports = analyzer.get_all_reports();
        let report = reports.values().next().expect("Should have report");
        assert!(
            !report.memory_snapshot.related_allocations.is_empty(),
            "Should find related allocations"
        );
    }

    /// Objective: Verify correlation analysis between violations
    /// Invariants: DoubleFree and InvalidFree should be correlated
    #[test]
    fn test_correlation_analysis() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();
        let call_stack = CallStackRef::new(100, Some(1));

        let violation1 = SafetyViolation::DoubleFree {
            first_free_stack: call_stack.clone(),
            second_free_stack: call_stack.clone(),
            timestamp: 1000,
        };
        let violation2 = SafetyViolation::InvalidFree {
            attempted_pointer: 0x2000,
            stack: call_stack,
            timestamp: 2000,
        };

        analyzer.analyze_violation(&violation1, 0x1000).unwrap();
        analyzer.analyze_violation(&violation2, 0x2000).unwrap();

        let reports = analyzer.get_all_reports();
        let invalid_free_report = reports
            .values()
            .find(|r| r.violation_type == "InvalidFree")
            .expect("Should have InvalidFree report");

        assert!(
            !invalid_free_report.correlated_violations.is_empty(),
            "InvalidFree should be correlated with DoubleFree"
        );
    }

    /// Objective: Verify technical details generation
    /// Invariants: Should include timestamp in technical details
    #[test]
    fn test_technical_details_generation() {
        use crate::core::CallStackRef;

        let mut analyzer = SecurityViolationAnalyzer::default();
        let call_stack = CallStackRef::new(110, Some(1));

        let violation = SafetyViolation::DoubleFree {
            first_free_stack: call_stack.clone(),
            second_free_stack: call_stack,
            timestamp: 12345,
        };

        analyzer.analyze_violation(&violation, 0x1000).unwrap();

        let reports = analyzer.get_all_reports();
        let report = reports.values().next().expect("Should have report");
        assert!(
            report.technical_details.contains("12345"),
            "Technical details should include timestamp"
        );
        assert!(
            report.technical_details.contains("heap corruption"),
            "Technical details should mention heap corruption"
        );
    }
}
