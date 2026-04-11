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
            crate::analysis::unsafe_ffi_tracker::RiskLevel::Medium
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
