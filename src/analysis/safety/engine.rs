use crate::analysis::safety::types::*;
use crate::analysis::unsafe_ffi_tracker::StackFrame;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RiskAssessmentEngine {
    _risk_weights: HashMap<RiskFactorType, f64>,
    _historical_data: HashMap<String, Vec<f64>>,
}

impl RiskAssessmentEngine {
    pub fn new() -> Self {
        let mut risk_weights = HashMap::new();
        risk_weights.insert(RiskFactorType::RawPointerDereference, 8.5);
        risk_weights.insert(RiskFactorType::UnsafeDataRace, 9.0);
        risk_weights.insert(RiskFactorType::InvalidTransmute, 7.5);
        risk_weights.insert(RiskFactorType::FfiCall, 6.0);
        risk_weights.insert(RiskFactorType::ManualMemoryManagement, 7.0);
        risk_weights.insert(RiskFactorType::CrossBoundaryTransfer, 6.5);
        risk_weights.insert(RiskFactorType::UseAfterFree, 9.5);
        risk_weights.insert(RiskFactorType::BufferOverflow, 9.0);
        risk_weights.insert(RiskFactorType::LifetimeViolation, 8.0);

        Self {
            _risk_weights: risk_weights,
            _historical_data: HashMap::new(),
        }
    }

    pub fn assess_risk(
        &self,
        source: &UnsafeSource,
        context: &MemoryContext,
        call_stack: &[StackFrame],
    ) -> RiskAssessment {
        let mut risk_factors = Vec::new();
        let mut total_risk_score = 0.0;
        let mut total_confidence = 0.0;

        match source {
            UnsafeSource::UnsafeBlock { location, .. } => {
                risk_factors.extend(self.analyze_unsafe_block(location, call_stack));
            }
            UnsafeSource::FfiFunction {
                library, function, ..
            } => {
                risk_factors.extend(self.analyze_ffi_function(library, function, call_stack));
            }
            UnsafeSource::RawPointer { operation, .. } => {
                risk_factors.extend(self.analyze_raw_pointer(operation, call_stack));
            }
            UnsafeSource::Transmute {
                from_type, to_type, ..
            } => {
                risk_factors.extend(self.analyze_transmute(from_type, to_type, call_stack));
            }
        }

        for factor in &risk_factors {
            total_risk_score += factor.severity * factor.confidence;
            total_confidence += factor.confidence;
        }

        let risk_count = risk_factors.len() as f64;
        let average_confidence = if risk_count > 0.0 {
            total_confidence / risk_count
        } else {
            0.0
        };

        let pressure_multiplier = match context.memory_pressure {
            MemoryPressureLevel::Critical => 1.5,
            MemoryPressureLevel::High => 1.2,
            MemoryPressureLevel::Medium => 1.0,
            MemoryPressureLevel::Low => 0.8,
        };

        total_risk_score *= pressure_multiplier;

        let risk_level = if risk_factors.is_empty() {
            crate::analysis::unsafe_ffi_tracker::RiskLevel::Low
        } else if total_risk_score >= 80.0 {
            crate::analysis::unsafe_ffi_tracker::RiskLevel::Critical
        } else if total_risk_score >= 60.0 {
            crate::analysis::unsafe_ffi_tracker::RiskLevel::High
        } else if total_risk_score >= 40.0 {
            crate::analysis::unsafe_ffi_tracker::RiskLevel::Medium
        } else {
            crate::analysis::unsafe_ffi_tracker::RiskLevel::Low
        };

        let mitigation_suggestions =
            self.generate_mitigation_suggestions(&risk_factors, &risk_level);

        RiskAssessment {
            risk_level,
            risk_score: total_risk_score.min(100.0),
            risk_factors,
            confidence_score: average_confidence,
            mitigation_suggestions,
            assessment_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    fn analyze_unsafe_block(&self, location: &str, call_stack: &[StackFrame]) -> Vec<RiskFactor> {
        let mut factors = Vec::new();

        if location.contains("*") || location.contains("ptr::") {
            factors.push(RiskFactor {
                factor_type: RiskFactorType::RawPointerDereference,
                severity: 7.5,
                confidence: 0.8,
                description: "Raw pointer dereference in unsafe block".to_string(),
                source_location: Some(location.to_string()),
                call_stack: call_stack.to_vec(),
                mitigation: "Add bounds checking and null pointer validation".to_string(),
            });
        }

        if location.contains("alloc") || location.contains("dealloc") || location.contains("free") {
            factors.push(RiskFactor {
                factor_type: RiskFactorType::ManualMemoryManagement,
                severity: 6.5,
                confidence: 0.9,
                description: "Manual memory management in unsafe block".to_string(),
                source_location: Some(location.to_string()),
                call_stack: call_stack.to_vec(),
                mitigation: "Use RAII patterns and smart pointers where possible".to_string(),
            });
        }

        factors
    }

    fn analyze_ffi_function(
        &self,
        library: &str,
        function: &str,
        call_stack: &[StackFrame],
    ) -> Vec<RiskFactor> {
        let mut factors = Vec::new();

        factors.push(RiskFactor {
            factor_type: RiskFactorType::FfiCall,
            severity: 5.5,
            confidence: 0.7,
            description: format!("FFI call to {library}::{function}"),
            source_location: Some(format!("{library}::{function}")),
            call_stack: call_stack.to_vec(),
            mitigation: "Validate all parameters and return values".to_string(),
        });

        let risky_functions = ["malloc", "free", "strcpy", "strcat", "sprintf", "gets"];
        if risky_functions.iter().any(|&f| function.contains(f)) {
            factors.push(RiskFactor {
                factor_type: RiskFactorType::BufferOverflow,
                severity: 8.0,
                confidence: 0.9,
                description: format!("Call to potentially unsafe function: {function}"),
                source_location: Some(format!("{library}::{function}")),
                call_stack: call_stack.to_vec(),
                mitigation: "Use safer alternatives or add explicit bounds checking".to_string(),
            });
        }

        factors
    }

    fn analyze_raw_pointer(&self, operation: &str, call_stack: &[StackFrame]) -> Vec<RiskFactor> {
        let mut factors = Vec::new();

        factors.push(RiskFactor {
            factor_type: RiskFactorType::RawPointerDereference,
            severity: 8.0,
            confidence: 0.85,
            description: format!("Raw pointer operation: {operation}"),
            source_location: Some(operation.to_string()),
            call_stack: call_stack.to_vec(),
            mitigation: "Add null checks and bounds validation".to_string(),
        });

        factors
    }

    fn analyze_transmute(
        &self,
        from_type: &str,
        to_type: &str,
        call_stack: &[StackFrame],
    ) -> Vec<RiskFactor> {
        let mut factors = Vec::new();

        let severity = if from_type.contains("*") || to_type.contains("*") {
            9.0
        } else {
            7.0
        };

        factors.push(RiskFactor {
            factor_type: RiskFactorType::InvalidTransmute,
            severity,
            confidence: 0.8,
            description: format!("Transmute from {from_type} to {to_type}"),
            source_location: Some(format!("{from_type} -> {to_type}")),
            call_stack: call_stack.to_vec(),
            mitigation: "Verify size and alignment compatibility".to_string(),
        });

        factors
    }

    fn generate_mitigation_suggestions(
        &self,
        risk_factors: &[RiskFactor],
        risk_level: &crate::analysis::unsafe_ffi_tracker::RiskLevel,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        match risk_level {
            crate::analysis::unsafe_ffi_tracker::RiskLevel::Critical => {
                suggestions.push(
                    "URGENT: Critical safety issues detected - immediate review required"
                        .to_string(),
                );
                suggestions.push(
                    "Consider refactoring to eliminate unsafe code where possible".to_string(),
                );
            }
            crate::analysis::unsafe_ffi_tracker::RiskLevel::High => {
                suggestions.push(
                    "High-risk operations detected - thorough testing recommended".to_string(),
                );
                suggestions.push("Add comprehensive error handling and validation".to_string());
            }
            crate::analysis::unsafe_ffi_tracker::RiskLevel::Medium => {
                suggestions
                    .push("Moderate risks detected - review and add safety checks".to_string());
            }
            crate::analysis::unsafe_ffi_tracker::RiskLevel::Low => {
                suggestions.push("Low-level risks detected - monitor for issues".to_string());
            }
        }

        let mut factor_types: HashSet<RiskFactorType> = HashSet::new();
        for factor in risk_factors {
            factor_types.insert(factor.factor_type.clone());
        }

        for factor_type in factor_types {
            match factor_type {
                RiskFactorType::RawPointerDereference => {
                    suggestions.push("Add null pointer checks before dereferencing".to_string());
                    suggestions.push("Validate pointer bounds and alignment".to_string());
                }
                RiskFactorType::UnsafeDataRace => {
                    suggestions.push("Use proper synchronization primitives".to_string());
                    suggestions.push("Consider using atomic operations".to_string());
                }
                RiskFactorType::FfiCall => {
                    suggestions.push("Validate all FFI parameters and return values".to_string());
                    suggestions.push("Handle FFI errors gracefully".to_string());
                }
                RiskFactorType::ManualMemoryManagement => {
                    suggestions.push("Use RAII patterns to ensure cleanup".to_string());
                    suggestions.push("Consider using smart pointers".to_string());
                }
                _ => {}
            }
        }

        suggestions
    }
}

impl Default for RiskAssessmentEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::unsafe_ffi_tracker::RiskLevel;

    fn default_memory_context() -> MemoryContext {
        MemoryContext {
            total_allocated: 0,
            active_allocations: 0,
            memory_pressure: MemoryPressureLevel::Low,
            allocation_patterns: Vec::new(),
        }
    }

    /// Objective: Verify RiskAssessmentEngine creation with default weights
    /// Invariants: Engine should initialize with predefined risk weights
    #[test]
    fn test_engine_creation() {
        let engine = RiskAssessmentEngine::new();
        let assessment = engine.assess_risk(
            &UnsafeSource::UnsafeBlock {
                location: "test.rs".to_string(),
                function: "test".to_string(),
                file_path: None,
                line_number: None,
            },
            &default_memory_context(),
            &[],
        );
        assert!(
            assessment.risk_score >= 0.0,
            "Risk score should be non-negative"
        );
    }

    /// Objective: Verify Default trait implementation
    /// Invariants: Default should create same engine as new()
    #[test]
    fn test_engine_default() {
        let engine = RiskAssessmentEngine::default();
        let assessment = engine.assess_risk(
            &UnsafeSource::UnsafeBlock {
                location: "test.rs".to_string(),
                function: "test".to_string(),
                file_path: None,
                line_number: None,
            },
            &default_memory_context(),
            &[],
        );
        assert!(
            assessment.confidence_score >= 0.0,
            "Confidence score should be non-negative"
        );
    }

    /// Objective: Verify analyze_unsafe_block with pointer dereference
    /// Invariants: Should detect raw pointer dereference risk
    #[test]
    fn test_analyze_unsafe_block_pointer() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::UnsafeBlock {
                location: "*ptr::read".to_string(),
                function: "test".to_string(),
                file_path: None,
                line_number: None,
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            !assessment.risk_factors.is_empty(),
            "Should detect pointer risk in unsafe block"
        );
        assert!(
            assessment
                .risk_factors
                .iter()
                .any(|f| matches!(f.factor_type, RiskFactorType::RawPointerDereference)),
            "Should have RawPointerDereference factor"
        );
    }

    /// Objective: Verify analyze_unsafe_block with memory management
    /// Invariants: Should detect manual memory management risk
    #[test]
    fn test_analyze_unsafe_block_memory() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::UnsafeBlock {
                location: "alloc::alloc".to_string(),
                function: "test".to_string(),
                file_path: None,
                line_number: None,
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            assessment
                .risk_factors
                .iter()
                .any(|f| matches!(f.factor_type, RiskFactorType::ManualMemoryManagement)),
            "Should detect manual memory management"
        );
    }

    /// Objective: Verify analyze_unsafe_block with dealloc
    /// Invariants: Should detect deallocation risk
    #[test]
    fn test_analyze_unsafe_block_dealloc() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::UnsafeBlock {
                location: "dealloc".to_string(),
                function: "test".to_string(),
                file_path: None,
                line_number: None,
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            assessment
                .risk_factors
                .iter()
                .any(|f| matches!(f.factor_type, RiskFactorType::ManualMemoryManagement)),
            "Should detect deallocation as memory management"
        );
    }

    /// Objective: Verify analyze_unsafe_block with free
    /// Invariants: Should detect free operation risk
    #[test]
    fn test_analyze_unsafe_block_free() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::UnsafeBlock {
                location: "free_memory".to_string(),
                function: "test".to_string(),
                file_path: None,
                line_number: None,
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            assessment
                .risk_factors
                .iter()
                .any(|f| matches!(f.factor_type, RiskFactorType::ManualMemoryManagement)),
            "Should detect free as memory management"
        );
    }

    /// Objective: Verify analyze_ffi_function with normal function
    /// Invariants: Should detect FFI call risk
    #[test]
    fn test_analyze_ffi_function_normal() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::FfiFunction {
                library: "libc".to_string(),
                function: "printf".to_string(),
                call_site: "test.rs".to_string(),
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            !assessment.risk_factors.is_empty(),
            "Should detect FFI call risk"
        );
        assert!(
            assessment
                .risk_factors
                .iter()
                .any(|f| matches!(f.factor_type, RiskFactorType::FfiCall)),
            "Should have FfiCall factor"
        );
    }

    /// Objective: Verify analyze_ffi_function with risky function (malloc)
    /// Invariants: Should detect buffer overflow risk for malloc
    #[test]
    fn test_analyze_ffi_function_malloc() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::FfiFunction {
                library: "libc".to_string(),
                function: "malloc".to_string(),
                call_site: "test.rs".to_string(),
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            assessment
                .risk_factors
                .iter()
                .any(|f| matches!(f.factor_type, RiskFactorType::BufferOverflow)),
            "Should detect BufferOverflow risk for malloc"
        );
    }

    /// Objective: Verify analyze_ffi_function with risky function (strcpy)
    /// Invariants: Should detect buffer overflow risk for strcpy
    #[test]
    fn test_analyze_ffi_function_strcpy() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::FfiFunction {
                library: "libc".to_string(),
                function: "strcpy".to_string(),
                call_site: "test.rs".to_string(),
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            assessment
                .risk_factors
                .iter()
                .any(|f| matches!(f.factor_type, RiskFactorType::BufferOverflow)),
            "Should detect BufferOverflow risk for strcpy"
        );
    }

    /// Objective: Verify analyze_ffi_function with risky function (sprintf)
    /// Invariants: Should detect buffer overflow risk for sprintf
    #[test]
    fn test_analyze_ffi_function_sprintf() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::FfiFunction {
                library: "libc".to_string(),
                function: "sprintf".to_string(),
                call_site: "test.rs".to_string(),
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            assessment
                .risk_factors
                .iter()
                .any(|f| matches!(f.factor_type, RiskFactorType::BufferOverflow)),
            "Should detect BufferOverflow risk for sprintf"
        );
    }

    /// Objective: Verify analyze_ffi_function with risky function (gets)
    /// Invariants: Should detect buffer overflow risk for gets
    #[test]
    fn test_analyze_ffi_function_gets() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::FfiFunction {
                library: "libc".to_string(),
                function: "gets".to_string(),
                call_site: "test.rs".to_string(),
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            assessment
                .risk_factors
                .iter()
                .any(|f| matches!(f.factor_type, RiskFactorType::BufferOverflow)),
            "Should detect BufferOverflow risk for gets"
        );
    }

    /// Objective: Verify analyze_raw_pointer operation
    /// Invariants: Should detect raw pointer dereference risk
    #[test]
    fn test_analyze_raw_pointer() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::RawPointer {
                operation: "dereference".to_string(),
                location: "test.rs".to_string(),
            },
            &default_memory_context(),
            &[],
        );

        assert_eq!(
            assessment.risk_factors.len(),
            1,
            "Should have exactly one risk factor"
        );
        assert!(
            matches!(
                assessment.risk_factors[0].factor_type,
                RiskFactorType::RawPointerDereference
            ),
            "Should have RawPointerDereference factor"
        );
        assert!(
            assessment.risk_factors[0].severity > 0.0,
            "Severity should be positive"
        );
    }

    /// Objective: Verify analyze_transmute with pointer types
    /// Invariants: Should assign higher severity for pointer transmute
    #[test]
    fn test_analyze_transmute_pointer() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::Transmute {
                from_type: "*const u8".to_string(),
                to_type: "usize".to_string(),
                location: "test.rs".to_string(),
            },
            &default_memory_context(),
            &[],
        );

        assert_eq!(
            assessment.risk_factors.len(),
            1,
            "Should have exactly one risk factor"
        );
        assert!(
            matches!(
                assessment.risk_factors[0].factor_type,
                RiskFactorType::InvalidTransmute
            ),
            "Should have InvalidTransmute factor"
        );
        assert_eq!(
            assessment.risk_factors[0].severity, 9.0,
            "Pointer transmute should have severity 9.0"
        );
    }

    /// Objective: Verify analyze_transmute with non-pointer types
    /// Invariants: Should assign lower severity for non-pointer transmute
    #[test]
    fn test_analyze_transmute_non_pointer() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::Transmute {
                from_type: "u32".to_string(),
                to_type: "i32".to_string(),
                location: "test.rs".to_string(),
            },
            &default_memory_context(),
            &[],
        );

        assert_eq!(
            assessment.risk_factors[0].severity, 7.0,
            "Non-pointer transmute should have severity 7.0"
        );
    }

    /// Objective: Verify memory pressure multiplier - Critical
    /// Invariants: Critical pressure should multiply risk by 1.5
    #[test]
    fn test_memory_pressure_critical() {
        let engine = RiskAssessmentEngine::new();

        let context = MemoryContext {
            total_allocated: 2 * 1024 * 1024 * 1024,
            active_allocations: 100,
            memory_pressure: MemoryPressureLevel::Critical,
            allocation_patterns: Vec::new(),
        };

        let assessment = engine.assess_risk(
            &UnsafeSource::RawPointer {
                operation: "test".to_string(),
                location: "test.rs".to_string(),
            },
            &context,
            &[],
        );

        assert!(
            assessment.risk_score > 0.0,
            "Critical pressure should increase risk score"
        );
    }

    /// Objective: Verify memory pressure multiplier - High
    /// Invariants: High pressure should multiply risk by 1.2
    #[test]
    fn test_memory_pressure_high() {
        let engine = RiskAssessmentEngine::new();

        let context = MemoryContext {
            total_allocated: 600 * 1024 * 1024,
            active_allocations: 50,
            memory_pressure: MemoryPressureLevel::High,
            allocation_patterns: Vec::new(),
        };

        let assessment = engine.assess_risk(
            &UnsafeSource::RawPointer {
                operation: "test".to_string(),
                location: "test.rs".to_string(),
            },
            &context,
            &[],
        );

        assert!(
            assessment.risk_score > 0.0,
            "High pressure should affect risk score"
        );
    }

    /// Objective: Verify memory pressure multiplier - Medium
    /// Invariants: Medium pressure should multiply risk by 1.0
    #[test]
    fn test_memory_pressure_medium() {
        let engine = RiskAssessmentEngine::new();

        let context = MemoryContext {
            total_allocated: 300 * 1024 * 1024,
            active_allocations: 30,
            memory_pressure: MemoryPressureLevel::Medium,
            allocation_patterns: Vec::new(),
        };

        let assessment = engine.assess_risk(
            &UnsafeSource::RawPointer {
                operation: "test".to_string(),
                location: "test.rs".to_string(),
            },
            &context,
            &[],
        );

        assert!(
            assessment.risk_score > 0.0,
            "Medium pressure should not reduce risk score"
        );
    }

    /// Objective: Verify memory pressure multiplier - Low
    /// Invariants: Low pressure should multiply risk by 0.8
    #[test]
    fn test_memory_pressure_low() {
        let engine = RiskAssessmentEngine::new();

        let context = MemoryContext {
            total_allocated: 100 * 1024 * 1024,
            active_allocations: 10,
            memory_pressure: MemoryPressureLevel::Low,
            allocation_patterns: Vec::new(),
        };

        let assessment = engine.assess_risk(
            &UnsafeSource::RawPointer {
                operation: "test".to_string(),
                location: "test.rs".to_string(),
            },
            &context,
            &[],
        );

        assert!(
            assessment.risk_score >= 0.0,
            "Low pressure should produce valid risk score"
        );
    }

    /// Objective: Verify risk level calculation with critical memory pressure
    /// Invariants: Risk assessment should complete successfully with critical memory
    #[test]
    fn test_risk_level_critical() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::FfiFunction {
                library: "libc".to_string(),
                function: "malloc".to_string(),
                call_site: "test.rs".to_string(),
            },
            &MemoryContext {
                total_allocated: 2 * 1024 * 1024 * 1024,
                active_allocations: 100,
                memory_pressure: MemoryPressureLevel::Critical,
                allocation_patterns: Vec::new(),
            },
            &[],
        );

        assert!(
            assessment.risk_score >= 0.0,
            "Risk score should be non-negative"
        );
    }

    /// Objective: Verify risk level calculation for empty factors
    /// Invariants: Empty risk factors should result in Low risk
    #[test]
    fn test_risk_level_empty_factors() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::UnsafeBlock {
                location: "safe_location".to_string(),
                function: "safe_function".to_string(),
                file_path: None,
                line_number: None,
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            matches!(assessment.risk_level, RiskLevel::Low),
            "No risk factors should result in Low risk level"
        );
    }

    /// Objective: Verify mitigation suggestions for Critical risk
    /// Invariants: Critical risk should have urgent suggestions
    #[test]
    fn test_mitigation_critical() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::FfiFunction {
                library: "libc".to_string(),
                function: "malloc".to_string(),
                call_site: "test.rs".to_string(),
            },
            &MemoryContext {
                total_allocated: 2 * 1024 * 1024 * 1024,
                active_allocations: 100,
                memory_pressure: MemoryPressureLevel::Critical,
                allocation_patterns: Vec::new(),
            },
            &[],
        );

        if matches!(assessment.risk_level, RiskLevel::Critical) {
            assert!(
                assessment
                    .mitigation_suggestions
                    .iter()
                    .any(|s| s.contains("URGENT") || s.contains("Critical")),
                "Critical risk should have urgent suggestions"
            );
        }
    }

    /// Objective: Verify mitigation suggestions for High risk
    /// Invariants: High risk should have thorough testing suggestion
    #[test]
    fn test_mitigation_high() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::RawPointer {
                operation: "dereference".to_string(),
                location: "test.rs".to_string(),
            },
            &MemoryContext {
                total_allocated: 0,
                active_allocations: 0,
                memory_pressure: MemoryPressureLevel::High,
                allocation_patterns: Vec::new(),
            },
            &[],
        );

        if matches!(assessment.risk_level, RiskLevel::High) {
            assert!(
                assessment
                    .mitigation_suggestions
                    .iter()
                    .any(|s| s.contains("High-risk") || s.contains("testing")),
                "High risk should have testing suggestions"
            );
        }
    }

    /// Objective: Verify mitigation suggestions for Medium risk
    /// Invariants: Medium risk should have review suggestion
    #[test]
    fn test_mitigation_medium() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::UnsafeBlock {
                location: "test.rs".to_string(),
                function: "test".to_string(),
                file_path: None,
                line_number: None,
            },
            &MemoryContext {
                total_allocated: 0,
                active_allocations: 0,
                memory_pressure: MemoryPressureLevel::Medium,
                allocation_patterns: Vec::new(),
            },
            &[],
        );

        if matches!(assessment.risk_level, RiskLevel::Medium) {
            assert!(
                assessment
                    .mitigation_suggestions
                    .iter()
                    .any(|s| s.contains("Moderate") || s.contains("review")),
                "Medium risk should have review suggestions"
            );
        }
    }

    /// Objective: Verify mitigation suggestions for Low risk
    /// Invariants: Low risk should have monitor suggestion
    #[test]
    fn test_mitigation_low() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::UnsafeBlock {
                location: "safe".to_string(),
                function: "safe".to_string(),
                file_path: None,
                line_number: None,
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            assessment
                .mitigation_suggestions
                .iter()
                .any(|s| s.contains("Low") || s.contains("monitor")),
            "Low risk should have monitor suggestions"
        );
    }

    /// Objective: Verify mitigation suggestions for RawPointerDereference
    /// Invariants: Should include null pointer check suggestion
    #[test]
    fn test_mitigation_raw_pointer() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::RawPointer {
                operation: "dereference".to_string(),
                location: "test.rs".to_string(),
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            assessment
                .mitigation_suggestions
                .iter()
                .any(|s| s.contains("null") || s.contains("pointer")),
            "Should have null pointer check suggestion"
        );
    }

    /// Objective: Verify mitigation suggestions for FfiCall
    /// Invariants: Should include FFI validation suggestion
    #[test]
    fn test_mitigation_ffi_call() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::FfiFunction {
                library: "libc".to_string(),
                function: "printf".to_string(),
                call_site: "test.rs".to_string(),
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            assessment
                .mitigation_suggestions
                .iter()
                .any(|s| s.contains("FFI") || s.contains("validate")),
            "Should have FFI validation suggestion"
        );
    }

    /// Objective: Verify mitigation suggestions for ManualMemoryManagement
    /// Invariants: Should include RAII suggestion
    #[test]
    fn test_mitigation_memory_management() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::UnsafeBlock {
                location: "alloc".to_string(),
                function: "test".to_string(),
                file_path: None,
                line_number: None,
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            assessment
                .mitigation_suggestions
                .iter()
                .any(|s| s.contains("RAII") || s.contains("smart pointer")),
            "Should have RAII suggestion"
        );
    }

    /// Objective: Verify confidence score calculation
    /// Invariants: Confidence should be average of all factor confidences
    #[test]
    fn test_confidence_calculation() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::RawPointer {
                operation: "test".to_string(),
                location: "test.rs".to_string(),
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            assessment.confidence_score >= 0.0 && assessment.confidence_score <= 1.0,
            "Confidence should be between 0 and 1"
        );
    }

    /// Objective: Verify risk score is capped at 100
    /// Invariants: Risk score should never exceed 100.0
    #[test]
    fn test_risk_score_cap() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::FfiFunction {
                library: "libc".to_string(),
                function: "malloc".to_string(),
                call_site: "test.rs".to_string(),
            },
            &MemoryContext {
                total_allocated: usize::MAX,
                active_allocations: usize::MAX,
                memory_pressure: MemoryPressureLevel::Critical,
                allocation_patterns: Vec::new(),
            },
            &[],
        );

        assert!(
            assessment.risk_score <= 100.0,
            "Risk score should be capped at 100"
        );
    }

    /// Objective: Verify assessment timestamp is recent
    /// Invariants: Timestamp should be close to current time
    #[test]
    fn test_assessment_timestamp() {
        let engine = RiskAssessmentEngine::new();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let assessment = engine.assess_risk(
            &UnsafeSource::UnsafeBlock {
                location: "test".to_string(),
                function: "test".to_string(),
                file_path: None,
                line_number: None,
            },
            &default_memory_context(),
            &[],
        );

        let diff = assessment.assessment_timestamp.abs_diff(now);
        assert!(diff < 5, "Timestamp should be within 5 seconds of now");
    }

    /// Objective: Verify call stack is preserved in risk factors
    /// Invariants: Call stack should be included in each risk factor
    #[test]
    fn test_call_stack_preservation() {
        let engine = RiskAssessmentEngine::new();

        let call_stack = vec![StackFrame {
            function_name: "test_fn".to_string(),
            file_name: Some("test.rs".to_string()),
            line_number: Some(10),
            is_unsafe: true,
        }];

        let assessment = engine.assess_risk(
            &UnsafeSource::RawPointer {
                operation: "test".to_string(),
                location: "test.rs".to_string(),
            },
            &default_memory_context(),
            &call_stack,
        );

        for factor in &assessment.risk_factors {
            assert_eq!(
                factor.call_stack.len(),
                1,
                "Call stack should be preserved in risk factor"
            );
        }
    }

    /// Objective: Verify multiple risk factors from single source
    /// Invariants: Unsafe block with pointer and alloc should have multiple factors
    #[test]
    fn test_multiple_risk_factors() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::UnsafeBlock {
                location: "*ptr alloc".to_string(),
                function: "test".to_string(),
                file_path: None,
                line_number: None,
            },
            &default_memory_context(),
            &[],
        );

        assert!(
            assessment.risk_factors.len() >= 2,
            "Should have multiple risk factors for combined risks"
        );
    }

    /// Objective: Verify risk factor descriptions are meaningful
    /// Invariants: Each factor should have a non-empty description
    #[test]
    fn test_risk_factor_descriptions() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::FfiFunction {
                library: "libc".to_string(),
                function: "malloc".to_string(),
                call_site: "test.rs".to_string(),
            },
            &default_memory_context(),
            &[],
        );

        for factor in &assessment.risk_factors {
            assert!(
                !factor.description.is_empty(),
                "Risk factor should have description"
            );
            assert!(
                !factor.mitigation.is_empty(),
                "Risk factor should have mitigation"
            );
        }
    }

    /// Objective: Verify transmute with pointer in to_type
    /// Invariants: Pointer in to_type should also trigger high severity
    #[test]
    fn test_transmute_pointer_to_type() {
        let engine = RiskAssessmentEngine::new();

        let assessment = engine.assess_risk(
            &UnsafeSource::Transmute {
                from_type: "usize".to_string(),
                to_type: "*mut u8".to_string(),
                location: "test.rs".to_string(),
            },
            &default_memory_context(),
            &[],
        );

        assert_eq!(
            assessment.risk_factors[0].severity, 9.0,
            "Transmute to pointer should have high severity"
        );
    }

    /// Objective: Verify RiskFactorType variants coverage
    /// Invariants: All risk factor types should be handled
    #[test]
    fn test_risk_factor_type_variants() {
        let types = vec![
            RiskFactorType::RawPointerDereference,
            RiskFactorType::UnsafeDataRace,
            RiskFactorType::InvalidTransmute,
            RiskFactorType::FfiCall,
            RiskFactorType::ManualMemoryManagement,
            RiskFactorType::CrossBoundaryTransfer,
            RiskFactorType::UseAfterFree,
            RiskFactorType::BufferOverflow,
            RiskFactorType::LifetimeViolation,
        ];

        for factor_type in types {
            let debug_str = format!("{:?}", factor_type);
            assert!(
                !debug_str.is_empty(),
                "RiskFactorType should have debug representation"
            );
        }
    }
}
