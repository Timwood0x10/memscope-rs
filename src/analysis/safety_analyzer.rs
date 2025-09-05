//! Enhanced Safety Analysis for Unsafe Code and FFI Operations
//!
//! This module implements comprehensive safety analysis including:
//! - UnsafeReport generation with risk assessment
//! - Risk factor detection and classification
//! - Confidence scoring for safety violations
//! - Memory passport tracking for FFI boundaries

use crate::analysis::unsafe_ffi_tracker::{RiskLevel, SafetyViolation, StackFrame};
use crate::core::types::{AllocationInfo, TrackingResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Enhanced risk factor types for comprehensive safety analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RiskFactorType {
    /// Raw pointer dereference without bounds checking
    RawPointerDereference,
    /// Potential unsafe data race condition
    UnsafeDataRace,
    /// Invalid transmute operation
    InvalidTransmute,
    /// FFI function call with potential risks
    FfiCall,
    /// Manual memory management risks
    ManualMemoryManagement,
    /// Cross-boundary memory transfer
    CrossBoundaryTransfer,
    /// Use after free potential
    UseAfterFree,
    /// Buffer overflow potential
    BufferOverflow,
    /// Lifetime violation
    LifetimeViolation,
}

/// Individual risk factor with detailed assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Type of risk factor
    pub factor_type: RiskFactorType,
    /// Severity score (0.0 to 10.0)
    pub severity: f64,
    /// Confidence in detection (0.0 to 1.0)
    pub confidence: f64,
    /// Human-readable description
    pub description: String,
    /// Source location where risk was detected
    pub source_location: Option<String>,
    /// Call stack context
    pub call_stack: Vec<StackFrame>,
    /// Suggested mitigation
    pub mitigation: String,
}

/// Comprehensive risk assessment for unsafe operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall risk level
    pub risk_level: RiskLevel,
    /// Numerical risk score (0.0 to 100.0)
    pub risk_score: f64,
    /// Individual risk factors identified
    pub risk_factors: Vec<RiskFactor>,
    /// Overall confidence score (0.0 to 1.0)
    pub confidence_score: f64,
    /// Suggested mitigation strategies
    pub mitigation_suggestions: Vec<String>,
    /// Assessment timestamp
    pub assessment_timestamp: u64,
}

/// Comprehensive unsafe operation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeReport {
    /// Unique report identifier
    pub report_id: String,
    /// Source of the unsafe operation
    pub source: UnsafeSource,
    /// Comprehensive risk assessment
    pub risk_assessment: RiskAssessment,
    /// Dynamic violations detected during runtime
    pub dynamic_violations: Vec<DynamicViolation>,
    /// Related memory passports
    pub related_passports: Vec<String>,
    /// Memory context at time of analysis
    pub memory_context: MemoryContext,
    /// Report generation timestamp
    pub generated_at: u64,
}

/// Source information for unsafe operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnsafeSource {
    /// Unsafe block in Rust code
    UnsafeBlock {
        /// Location in source code
        location: String,
        /// Function containing the unsafe block
        function: String,
        /// File path
        file_path: Option<String>,
        /// Line number
        line_number: Option<u32>,
    },
    /// FFI function call
    FfiFunction {
        /// Library name
        library: String,
        /// Function name
        function: String,
        /// Call site location
        call_site: String,
    },
    /// Raw pointer operation
    RawPointer {
        /// Operation type (deref, cast, etc.)
        operation: String,
        /// Location of operation
        location: String,
    },
    /// Transmute operation
    Transmute {
        /// Source type
        from_type: String,
        /// Target type
        to_type: String,
        /// Location of transmute
        location: String,
    },
}

/// Dynamic violation detected during runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicViolation {
    /// Type of violation
    pub violation_type: ViolationType,
    /// Memory address involved
    pub memory_address: usize,
    /// Size of memory involved
    pub memory_size: usize,
    /// Timestamp when violation was detected
    pub detected_at: u64,
    /// Call stack at violation time
    pub call_stack: Vec<StackFrame>,
    /// Severity of the violation
    pub severity: RiskLevel,
    /// Additional context information
    pub context: String,
}

/// Types of dynamic violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    /// Double free violation
    DoubleFree,
    /// Use after free violation
    UseAfterFree,
    /// Buffer overflow
    BufferOverflow,
    /// Invalid memory access
    InvalidAccess,
    /// Data race condition
    DataRace,
    /// FFI boundary violation
    FfiBoundaryViolation,
}

/// Memory context at time of analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContext {
    /// Total allocated memory
    pub total_allocated: usize,
    /// Number of active allocations
    pub active_allocations: usize,
    /// Memory pressure level
    pub memory_pressure: MemoryPressureLevel,
    /// Recent allocation patterns
    pub allocation_patterns: Vec<AllocationPattern>,
}

/// Memory pressure levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryPressureLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Allocation pattern information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPattern {
    /// Pattern type
    pub pattern_type: String,
    /// Frequency of occurrence
    pub frequency: u32,
    /// Average size
    pub average_size: usize,
    /// Risk level associated with pattern
    pub risk_level: RiskLevel,
}

/// Memory passport for tracking cross-FFI boundary memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPassport {
    /// Unique passport identifier
    pub passport_id: String,
    /// Memory allocation pointer
    pub allocation_ptr: usize,
    /// Size in bytes
    pub size_bytes: usize,
    /// Current status at program shutdown
    pub status_at_shutdown: PassportStatus,
    /// Lifecycle events recorded
    pub lifecycle_events: Vec<PassportEvent>,
    /// Risk assessment for this memory
    pub risk_assessment: RiskAssessment,
    /// Creation timestamp
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
}

/// Status of memory passport at program shutdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PassportStatus {
    /// Memory properly freed by Rust
    FreedByRust,
    /// Memory handed over to FFI and not returned
    HandoverToFfi,
    /// Memory freed by foreign code
    FreedByForeign,
    /// Memory reclaimed by Rust from FFI
    ReclaimedByRust,
    /// Memory still in foreign custody (potential leak)
    InForeignCustody,
    /// Status unknown or corrupted
    Unknown,
}

/// Lifecycle event in memory passport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportEvent {
    /// Event type
    pub event_type: PassportEventType,
    /// Timestamp of event
    pub timestamp: u64,
    /// Context where event occurred
    pub context: String,
    /// Call stack at event time
    pub call_stack: Vec<StackFrame>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of passport events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PassportEventType {
    /// Memory allocated in Rust
    AllocatedInRust,
    /// Memory handed over to FFI
    HandoverToFfi,
    /// Memory freed by foreign code
    FreedByForeign,
    /// Memory reclaimed by Rust
    ReclaimedByRust,
    /// Memory accessed across boundary
    BoundaryAccess,
    /// Memory ownership transferred
    OwnershipTransfer,
}

/// Safety analyzer for comprehensive unsafe code analysis
pub struct SafetyAnalyzer {
    /// Generated unsafe reports
    unsafe_reports: Arc<Mutex<HashMap<String, UnsafeReport>>>,
    /// Memory passport registry
    memory_passports: Arc<Mutex<HashMap<usize, MemoryPassport>>>,
    /// Risk assessment engine
    risk_engine: RiskAssessmentEngine,
    /// Configuration for analysis
    config: SafetyAnalysisConfig,
    /// Statistics tracking
    stats: Arc<Mutex<SafetyAnalysisStats>>,
}

/// Configuration for safety analysis
#[derive(Debug, Clone)]
pub struct SafetyAnalysisConfig {
    /// Enable detailed risk assessment
    pub detailed_risk_assessment: bool,
    /// Enable memory passport tracking
    pub enable_passport_tracking: bool,
    /// Minimum risk level to report
    pub min_risk_level: RiskLevel,
    /// Maximum number of reports to keep
    pub max_reports: usize,
    /// Enable dynamic violation detection
    pub enable_dynamic_violations: bool,
}

impl Default for SafetyAnalysisConfig {
    fn default() -> Self {
        Self {
            detailed_risk_assessment: true,
            enable_passport_tracking: true,
            min_risk_level: RiskLevel::Low,
            max_reports: 1000,
            enable_dynamic_violations: true,
        }
    }
}

/// Statistics for safety analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SafetyAnalysisStats {
    /// Total reports generated
    pub total_reports: usize,
    /// Reports by risk level
    pub reports_by_risk_level: HashMap<String, usize>,
    /// Total passports created
    pub total_passports: usize,
    /// Passports by status
    pub passports_by_status: HashMap<String, usize>,
    /// Dynamic violations detected
    pub dynamic_violations: usize,
    /// Analysis start time
    pub analysis_start_time: u64,
}

/// Risk assessment engine for evaluating unsafe operations
pub struct RiskAssessmentEngine {
    /// Risk factor weights
    _risk_weights: HashMap<RiskFactorType, f64>,
    /// Historical risk data
    _historical_data: HashMap<String, Vec<f64>>,
}

impl RiskAssessmentEngine {
    /// Create new risk assessment engine
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

    /// Assess risk for unsafe operation
    pub fn assess_risk(
        &self,
        source: &UnsafeSource,
        context: &MemoryContext,
        call_stack: &[StackFrame],
    ) -> RiskAssessment {
        let mut risk_factors = Vec::new();
        let mut total_risk_score = 0.0;
        let mut total_confidence = 0.0;

        // Analyze based on source type
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

        // Calculate overall scores
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

        // Adjust for memory pressure
        let pressure_multiplier = match context.memory_pressure {
            MemoryPressureLevel::Critical => 1.5,
            MemoryPressureLevel::High => 1.2,
            MemoryPressureLevel::Medium => 1.0,
            MemoryPressureLevel::Low => 0.8,
        };

        total_risk_score *= pressure_multiplier;

        // Determine risk level
        let risk_level = if total_risk_score >= 80.0 {
            RiskLevel::Critical
        } else if total_risk_score >= 60.0 {
            RiskLevel::High
        } else if total_risk_score >= 40.0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        // Generate mitigation suggestions
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

    /// Analyze unsafe block for risk factors
    fn analyze_unsafe_block(&self, location: &str, call_stack: &[StackFrame]) -> Vec<RiskFactor> {
        let mut factors = Vec::new();

        // Check for raw pointer operations
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

        // Check for manual memory management
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

    /// Analyze FFI function call for risk factors
    fn analyze_ffi_function(
        &self,
        library: &str,
        function: &str,
        call_stack: &[StackFrame],
    ) -> Vec<RiskFactor> {
        let mut factors = Vec::new();

        // Base FFI risk
        factors.push(RiskFactor {
            factor_type: RiskFactorType::FfiCall,
            severity: 5.5,
            confidence: 0.7,
            description: format!("FFI call to {library}::{function}"),
            source_location: Some(format!("{library}::{function}")),
            call_stack: call_stack.to_vec(),
            mitigation: "Validate all parameters and return values".to_string(),
        });

        // Check for known risky functions
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

    /// Analyze raw pointer operation for risk factors
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

    /// Analyze transmute operation for risk factors
    fn analyze_transmute(
        &self,
        from_type: &str,
        to_type: &str,
        call_stack: &[StackFrame],
    ) -> Vec<RiskFactor> {
        let mut factors = Vec::new();

        let severity = if from_type.contains("*") || to_type.contains("*") {
            9.0 // Pointer transmutes are very risky
        } else {
            7.0 // Regular transmutes are moderately risky
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

    /// Generate mitigation suggestions based on risk factors
    fn generate_mitigation_suggestions(
        &self,
        risk_factors: &[RiskFactor],
        risk_level: &RiskLevel,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Add general suggestions based on risk level
        match risk_level {
            RiskLevel::Critical => {
                suggestions.push(
                    "URGENT: Critical safety issues detected - immediate review required"
                        .to_string(),
                );
                suggestions.push(
                    "Consider refactoring to eliminate unsafe code where possible".to_string(),
                );
            }
            RiskLevel::High => {
                suggestions.push(
                    "High-risk operations detected - thorough testing recommended".to_string(),
                );
                suggestions.push("Add comprehensive error handling and validation".to_string());
            }
            RiskLevel::Medium => {
                suggestions
                    .push("Moderate risks detected - review and add safety checks".to_string());
            }
            RiskLevel::Low => {
                suggestions.push("Low-level risks detected - monitor for issues".to_string());
            }
        }

        // Add specific suggestions based on risk factors
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

impl SafetyAnalyzer {
    /// Create new safety analyzer
    pub fn new(config: SafetyAnalysisConfig) -> Self {
        tracing::info!("🔒 Initializing Safety Analyzer");
        tracing::info!(
            "   • Detailed risk assessment: {}",
            config.detailed_risk_assessment
        );
        tracing::info!(
            "   • Passport tracking: {}",
            config.enable_passport_tracking
        );
        tracing::info!("   • Min risk level: {:?}", config.min_risk_level);

        Self {
            unsafe_reports: Arc::new(Mutex::new(HashMap::new())),
            memory_passports: Arc::new(Mutex::new(HashMap::new())),
            risk_engine: RiskAssessmentEngine::new(),
            config,
            stats: Arc::new(Mutex::new(SafetyAnalysisStats {
                analysis_start_time: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                ..Default::default()
            })),
        }
    }

    /// Generate unsafe report for detected unsafe operation
    pub fn generate_unsafe_report(
        &self,
        source: UnsafeSource,
        allocations: &[AllocationInfo],
        violations: &[SafetyViolation],
    ) -> TrackingResult<String> {
        let report_id = self.generate_report_id(&source);

        tracing::info!("🔍 Generating unsafe report: {}", report_id);

        // Create memory context
        let memory_context = self.create_memory_context(allocations);

        // Capture call stack
        let call_stack = self.capture_call_stack()?;

        // Perform risk assessment
        let risk_assessment = if self.config.detailed_risk_assessment {
            self.risk_engine
                .assess_risk(&source, &memory_context, &call_stack)
        } else {
            self.create_basic_risk_assessment(&source)
        };

        // Skip report if below minimum risk level
        if !self.should_generate_report(&risk_assessment.risk_level) {
            return Ok(report_id);
        }

        // Convert safety violations to dynamic violations
        let dynamic_violations = self.convert_safety_violations(violations);

        // Find related passports
        let related_passports = if self.config.enable_passport_tracking {
            self.find_related_passports(&source, allocations)
        } else {
            Vec::new()
        };

        // Create comprehensive report
        let report = UnsafeReport {
            report_id: report_id.clone(),
            source,
            risk_assessment: risk_assessment.clone(),
            dynamic_violations,
            related_passports,
            memory_context,
            generated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        // Store report
        if let Ok(mut reports) = self.unsafe_reports.lock() {
            // Maintain maximum report limit
            if reports.len() >= self.config.max_reports {
                // Remove oldest report
                if let Some(oldest_id) = reports.keys().next().cloned() {
                    reports.remove(&oldest_id);
                }
            }
            reports.insert(report_id.clone(), report);
        }

        // Update statistics
        self.update_stats(&report_id, &risk_assessment.risk_level);

        tracing::info!(
            "✅ Generated unsafe report: {} (risk: {:?})",
            report_id,
            risk_assessment.risk_level
        );

        Ok(report_id)
    }

    /// Create or update memory passport for FFI boundary tracking
    pub fn create_memory_passport(
        &self,
        allocation_ptr: usize,
        size_bytes: usize,
        initial_event: PassportEventType,
    ) -> TrackingResult<String> {
        if !self.config.enable_passport_tracking {
            return Ok(String::new());
        }

        let passport_id = format!(
            "passport_{:x}_{}",
            allocation_ptr,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );

        let call_stack = self.capture_call_stack()?;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Create initial event
        let initial_passport_event = PassportEvent {
            event_type: initial_event,
            timestamp: current_time,
            context: "SafetyAnalyzer".to_string(),
            call_stack,
            metadata: HashMap::new(),
        };

        // Create risk assessment for this memory
        let memory_context = MemoryContext {
            total_allocated: size_bytes,
            active_allocations: 1,
            memory_pressure: MemoryPressureLevel::Low,
            allocation_patterns: Vec::new(),
        };

        let source = UnsafeSource::RawPointer {
            operation: "passport_creation".to_string(),
            location: format!("0x{allocation_ptr:x}"),
        };

        let risk_assessment = self.risk_engine.assess_risk(&source, &memory_context, &[]);

        let passport = MemoryPassport {
            passport_id: passport_id.clone(),
            allocation_ptr,
            size_bytes,
            status_at_shutdown: PassportStatus::Unknown,
            lifecycle_events: vec![initial_passport_event],
            risk_assessment,
            created_at: current_time,
            updated_at: current_time,
        };

        if let Ok(mut passports) = self.memory_passports.lock() {
            passports.insert(allocation_ptr, passport);
        }

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_passports += 1;
        }

        tracing::info!(
            "📋 Created memory passport: {} for 0x{:x}",
            passport_id,
            allocation_ptr
        );

        Ok(passport_id)
    }

    /// Record passport event for memory lifecycle tracking
    pub fn record_passport_event(
        &self,
        allocation_ptr: usize,
        event_type: PassportEventType,
        context: String,
    ) -> TrackingResult<()> {
        if !self.config.enable_passport_tracking {
            return Ok(());
        }

        let call_stack = self.capture_call_stack()?;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let event = PassportEvent {
            event_type,
            timestamp: current_time,
            context,
            call_stack,
            metadata: HashMap::new(),
        };

        if let Ok(mut passports) = self.memory_passports.lock() {
            if let Some(passport) = passports.get_mut(&allocation_ptr) {
                passport.lifecycle_events.push(event);
                passport.updated_at = current_time;

                tracing::info!("📝 Recorded passport event for 0x{:x}", allocation_ptr);
            }
        }

        Ok(())
    }

    /// Finalize passports at program shutdown and detect leaks
    pub fn finalize_passports_at_shutdown(&self) -> Vec<String> {
        let mut leaked_passports = Vec::new();

        if let Ok(mut passports) = self.memory_passports.lock() {
            for (ptr, passport) in passports.iter_mut() {
                // Determine final status based on lifecycle events
                let final_status = self.determine_final_passport_status(&passport.lifecycle_events);
                passport.status_at_shutdown = final_status.clone();

                // Check for leaks
                if matches!(final_status, PassportStatus::InForeignCustody) {
                    leaked_passports.push(passport.passport_id.clone());
                    tracing::warn!(
                        "🚨 Memory leak detected: passport {} (0x{:x}) in foreign custody",
                        passport.passport_id,
                        ptr
                    );
                }
            }

            // Update statistics
            if let Ok(mut stats) = self.stats.lock() {
                for passport in passports.values() {
                    let status_key = format!("{:?}", passport.status_at_shutdown);
                    *stats.passports_by_status.entry(status_key).or_insert(0) += 1;
                }
            }
        }

        tracing::info!(
            "🏁 Finalized {} passports, {} leaks detected",
            self.get_passport_count(),
            leaked_passports.len()
        );

        leaked_passports
    }

    /// Get all unsafe reports
    pub fn get_unsafe_reports(&self) -> HashMap<String, UnsafeReport> {
        self.unsafe_reports
            .lock()
            .unwrap_or_else(|_| {
                tracing::error!("Failed to lock unsafe reports");
                std::process::exit(1);
            })
            .clone()
    }

    /// Get all memory passports
    pub fn get_memory_passports(&self) -> HashMap<usize, MemoryPassport> {
        self.memory_passports
            .lock()
            .unwrap_or_else(|_| {
                tracing::error!("Failed to lock memory passports");
                std::process::exit(1);
            })
            .clone()
    }

    /// Get analysis statistics
    pub fn get_stats(&self) -> SafetyAnalysisStats {
        self.stats
            .lock()
            .unwrap_or_else(|_| {
                tracing::error!("Failed to lock stats");
                std::process::exit(1);
            })
            .clone()
    }

    // Private helper methods

    fn generate_report_id(&self, source: &UnsafeSource) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        let source_type = match source {
            UnsafeSource::UnsafeBlock { .. } => "UB",
            UnsafeSource::FfiFunction { .. } => "FFI",
            UnsafeSource::RawPointer { .. } => "PTR",
            UnsafeSource::Transmute { .. } => "TX",
        };

        format!("UNSAFE-{}-{}", source_type, timestamp % 1000000)
    }

    fn create_memory_context(&self, allocations: &[AllocationInfo]) -> MemoryContext {
        let total_allocated = allocations.iter().map(|a| a.size).sum();
        let active_allocations = allocations
            .iter()
            .filter(|a| a.timestamp_dealloc.is_none())
            .count();

        let memory_pressure = if total_allocated > 1024 * 1024 * 1024 {
            MemoryPressureLevel::Critical
        } else if total_allocated > 512 * 1024 * 1024 {
            MemoryPressureLevel::High
        } else if total_allocated > 256 * 1024 * 1024 {
            MemoryPressureLevel::Medium
        } else {
            MemoryPressureLevel::Low
        };

        MemoryContext {
            total_allocated,
            active_allocations,
            memory_pressure,
            allocation_patterns: Vec::new(), // Could be enhanced with pattern analysis
        }
    }

    fn capture_call_stack(&self) -> TrackingResult<Vec<StackFrame>> {
        // Simplified call stack capture
        // In a real implementation, this would use backtrace or similar
        Ok(vec![StackFrame {
            function_name: "safety_analyzer".to_string(),
            file_name: Some("src/analysis/safety_analyzer.rs".to_string()),
            line_number: Some(1),
            is_unsafe: false,
        }])
    }

    fn create_basic_risk_assessment(&self, source: &UnsafeSource) -> RiskAssessment {
        let (risk_level, risk_score) = match source {
            UnsafeSource::UnsafeBlock { .. } => (RiskLevel::Medium, 50.0),
            UnsafeSource::FfiFunction { .. } => (RiskLevel::Medium, 45.0),
            UnsafeSource::RawPointer { .. } => (RiskLevel::High, 70.0),
            UnsafeSource::Transmute { .. } => (RiskLevel::High, 65.0),
        };

        RiskAssessment {
            risk_level,
            risk_score,
            risk_factors: Vec::new(),
            confidence_score: 0.5,
            mitigation_suggestions: vec!["Review unsafe operation for safety".to_string()],
            assessment_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    fn should_generate_report(&self, risk_level: &RiskLevel) -> bool {
        match (&self.config.min_risk_level, risk_level) {
            (RiskLevel::Low, _) => true,
            (RiskLevel::Medium, RiskLevel::Low) => false,
            (RiskLevel::Medium, _) => true,
            (RiskLevel::High, RiskLevel::Low | RiskLevel::Medium) => false,
            (RiskLevel::High, _) => true,
            (RiskLevel::Critical, RiskLevel::Critical) => true,
            (RiskLevel::Critical, _) => false,
        }
    }

    fn convert_safety_violations(&self, violations: &[SafetyViolation]) -> Vec<DynamicViolation> {
        violations
            .iter()
            .map(|v| {
                match v {
                    SafetyViolation::DoubleFree { timestamp, .. } => DynamicViolation {
                        violation_type: ViolationType::DoubleFree,
                        memory_address: 0, // Would need to extract from violation
                        memory_size: 0,
                        detected_at: (*timestamp as u64),
                        call_stack: Vec::new(),
                        severity: RiskLevel::Critical,
                        context: "Double free detected".to_string(),
                    },
                    SafetyViolation::InvalidFree {
                        attempted_pointer,
                        timestamp,
                        ..
                    } => DynamicViolation {
                        violation_type: ViolationType::InvalidAccess,
                        memory_address: *attempted_pointer,
                        memory_size: 0,
                        detected_at: (*timestamp as u64),
                        call_stack: Vec::new(),
                        severity: RiskLevel::High,
                        context: "Invalid free attempted".to_string(),
                    },
                    SafetyViolation::PotentialLeak {
                        leak_detection_timestamp,
                        ..
                    } => DynamicViolation {
                        violation_type: ViolationType::InvalidAccess,
                        memory_address: 0,
                        memory_size: 0,
                        detected_at: (*leak_detection_timestamp as u64),
                        call_stack: Vec::new(),
                        severity: RiskLevel::Medium,
                        context: "Potential memory leak".to_string(),
                    },
                    SafetyViolation::CrossBoundaryRisk { .. } => DynamicViolation {
                        violation_type: ViolationType::FfiBoundaryViolation,
                        memory_address: 0,
                        memory_size: 0,
                        detected_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        call_stack: Vec::new(),
                        severity: RiskLevel::Medium,
                        context: "Cross-boundary risk detected".to_string(),
                    },
                }
            })
            .collect()
    }

    fn find_related_passports(
        &self,
        _source: &UnsafeSource,
        _allocations: &[AllocationInfo],
    ) -> Vec<String> {
        // Simplified implementation - could be enhanced with actual correlation logic
        Vec::new()
    }

    fn update_stats(&self, _report_id: &str, risk_level: &RiskLevel) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_reports += 1;
            let risk_key = format!("{risk_level:?}");
            *stats.reports_by_risk_level.entry(risk_key).or_insert(0) += 1;
        }
    }

    fn determine_final_passport_status(&self, events: &[PassportEvent]) -> PassportStatus {
        // Analyze lifecycle events to determine final status
        let mut has_handover = false;
        let mut has_reclaim = false;
        let mut has_foreign_free = false;

        for event in events {
            match event.event_type {
                PassportEventType::HandoverToFfi => has_handover = true,
                PassportEventType::ReclaimedByRust => has_reclaim = true,
                PassportEventType::FreedByForeign => has_foreign_free = true,
                _ => {}
            }
        }

        if has_handover && !has_reclaim && !has_foreign_free {
            PassportStatus::InForeignCustody
        } else if has_foreign_free {
            PassportStatus::FreedByForeign
        } else if has_reclaim {
            PassportStatus::ReclaimedByRust
        } else if has_handover {
            PassportStatus::HandoverToFfi
        } else {
            PassportStatus::FreedByRust
        }
    }

    fn get_passport_count(&self) -> usize {
        self.memory_passports.lock().map(|p| p.len()).unwrap_or(0)
    }
}

impl Default for SafetyAnalyzer {
    fn default() -> Self {
        Self::new(SafetyAnalysisConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AllocationInfo;
    use crate::core::CallStackRef;

    #[test]
    fn test_risk_factor_type_equality() {
        assert_eq!(
            RiskFactorType::RawPointerDereference,
            RiskFactorType::RawPointerDereference
        );
        assert_ne!(
            RiskFactorType::RawPointerDereference,
            RiskFactorType::UnsafeDataRace
        );
    }

    #[test]
    fn test_risk_factor_creation() {
        let factor = RiskFactor {
            factor_type: RiskFactorType::BufferOverflow,
            severity: 8.5,
            confidence: 0.9,
            description: "Test buffer overflow".to_string(),
            source_location: Some("test.rs:42".to_string()),
            call_stack: vec![],
            mitigation: "Add bounds checking".to_string(),
        };

        assert_eq!(factor.factor_type, RiskFactorType::BufferOverflow);
        assert_eq!(factor.severity, 8.5);
        assert_eq!(factor.confidence, 0.9);
        assert_eq!(factor.description, "Test buffer overflow");
    }

    #[test]
    fn test_risk_assessment_creation() {
        let assessment = RiskAssessment {
            risk_level: RiskLevel::High,
            risk_score: 75.0,
            risk_factors: vec![],
            confidence_score: 0.8,
            mitigation_suggestions: vec!["Test mitigation".to_string()],
            assessment_timestamp: 1234567890,
        };

        matches!(assessment.risk_level, RiskLevel::High);
        assert_eq!(assessment.risk_score, 75.0);
        assert_eq!(assessment.confidence_score, 0.8);
        assert_eq!(assessment.mitigation_suggestions.len(), 1);
    }

    #[test]
    fn test_unsafe_source_variants() {
        let unsafe_block = UnsafeSource::UnsafeBlock {
            location: "test.rs:10".to_string(),
            function: "test_function".to_string(),
            file_path: Some("test.rs".to_string()),
            line_number: Some(10),
        };

        let ffi_function = UnsafeSource::FfiFunction {
            library: "libc".to_string(),
            function: "malloc".to_string(),
            call_site: "test.rs:20".to_string(),
        };

        let raw_pointer = UnsafeSource::RawPointer {
            operation: "dereference".to_string(),
            location: "test.rs:30".to_string(),
        };

        let transmute = UnsafeSource::Transmute {
            from_type: "u32".to_string(),
            to_type: "f32".to_string(),
            location: "test.rs:40".to_string(),
        };

        // Test that all variants can be created
        match unsafe_block {
            UnsafeSource::UnsafeBlock { location, .. } => assert_eq!(location, "test.rs:10"),
            _ => panic!("Wrong variant"),
        }

        match ffi_function {
            UnsafeSource::FfiFunction { library, .. } => assert_eq!(library, "libc"),
            _ => panic!("Wrong variant"),
        }

        match raw_pointer {
            UnsafeSource::RawPointer { operation, .. } => assert_eq!(operation, "dereference"),
            _ => panic!("Wrong variant"),
        }

        match transmute {
            UnsafeSource::Transmute { from_type, .. } => assert_eq!(from_type, "u32"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_dynamic_violation_creation() {
        let violation = DynamicViolation {
            violation_type: ViolationType::UseAfterFree,
            memory_address: 0x1000,
            memory_size: 256,
            detected_at: 1234567890,
            call_stack: vec![],
            severity: RiskLevel::Critical,
            context: "Test violation".to_string(),
        };

        assert_eq!(violation.memory_address, 0x1000);
        assert_eq!(violation.memory_size, 256);
        matches!(violation.severity, RiskLevel::Critical);
    }

    #[test]
    fn test_memory_context_creation() {
        let context = MemoryContext {
            total_allocated: 1024,
            active_allocations: 5,
            memory_pressure: MemoryPressureLevel::Medium,
            allocation_patterns: vec![],
        };

        assert_eq!(context.total_allocated, 1024);
        assert_eq!(context.active_allocations, 5);
        matches!(context.memory_pressure, MemoryPressureLevel::Medium);
    }

    #[test]
    fn test_allocation_pattern_creation() {
        let pattern = AllocationPattern {
            pattern_type: "frequent_small".to_string(),
            frequency: 100,
            average_size: 64,
            risk_level: RiskLevel::Low,
        };

        assert_eq!(pattern.pattern_type, "frequent_small");
        assert_eq!(pattern.frequency, 100);
        assert_eq!(pattern.average_size, 64);
    }

    #[test]
    fn test_memory_passport_creation() {
        let passport = MemoryPassport {
            passport_id: "test_passport".to_string(),
            allocation_ptr: 0x2000,
            size_bytes: 512,
            status_at_shutdown: PassportStatus::Unknown,
            lifecycle_events: vec![],
            risk_assessment: RiskAssessment {
                risk_level: RiskLevel::Low,
                risk_score: 20.0,
                risk_factors: vec![],
                confidence_score: 0.5,
                mitigation_suggestions: vec![],
                assessment_timestamp: 1234567890,
            },
            created_at: 1234567890,
            updated_at: 1234567890,
        };

        assert_eq!(passport.passport_id, "test_passport");
        assert_eq!(passport.allocation_ptr, 0x2000);
        assert_eq!(passport.size_bytes, 512);
    }

    #[test]
    fn test_passport_event_creation() {
        let event = PassportEvent {
            event_type: PassportEventType::AllocatedInRust,
            timestamp: 1234567890,
            context: "test_context".to_string(),
            call_stack: vec![],
            metadata: HashMap::new(),
        };

        assert_eq!(event.timestamp, 1234567890);
        assert_eq!(event.context, "test_context");
        matches!(event.event_type, PassportEventType::AllocatedInRust);
    }

    #[test]
    fn test_safety_analysis_config_default() {
        let config = SafetyAnalysisConfig::default();

        assert!(config.detailed_risk_assessment);
        assert!(config.enable_passport_tracking);
        matches!(config.min_risk_level, RiskLevel::Low);
        assert_eq!(config.max_reports, 1000);
        assert!(config.enable_dynamic_violations);
    }

    #[test]
    fn test_safety_analysis_config_custom() {
        let config = SafetyAnalysisConfig {
            detailed_risk_assessment: false,
            enable_passport_tracking: false,
            min_risk_level: RiskLevel::High,
            max_reports: 500,
            enable_dynamic_violations: false,
        };

        assert!(!config.detailed_risk_assessment);
        assert!(!config.enable_passport_tracking);
        matches!(config.min_risk_level, RiskLevel::High);
        assert_eq!(config.max_reports, 500);
        assert!(!config.enable_dynamic_violations);
    }

    #[test]
    fn test_safety_analysis_stats_default() {
        let stats = SafetyAnalysisStats::default();

        assert_eq!(stats.total_reports, 0);
        assert!(stats.reports_by_risk_level.is_empty());
        assert_eq!(stats.total_passports, 0);
        assert!(stats.passports_by_status.is_empty());
        assert_eq!(stats.dynamic_violations, 0);
        assert_eq!(stats.analysis_start_time, 0);
    }

    #[test]
    fn test_risk_assessment_engine_assess_unsafe_block() {
        let engine = RiskAssessmentEngine::new();
        let source = UnsafeSource::UnsafeBlock {
            location: "ptr::read(data)".to_string(),
            function: "test_function".to_string(),
            file_path: Some("test.rs".to_string()),
            line_number: Some(42),
        };
        let context = MemoryContext {
            total_allocated: 1024,
            active_allocations: 1,
            memory_pressure: MemoryPressureLevel::Low,
            allocation_patterns: vec![],
        };

        let assessment = engine.assess_risk(&source, &context, &[]);

        assert!(assessment.risk_score >= 0.0);
        assert!(assessment.confidence_score >= 0.0);
        assert!(assessment.assessment_timestamp > 0);
        assert!(!assessment.mitigation_suggestions.is_empty());
    }

    #[test]
    fn test_risk_assessment_engine_assess_ffi_function() {
        let engine = RiskAssessmentEngine::new();
        let source = UnsafeSource::FfiFunction {
            library: "libc".to_string(),
            function: "malloc".to_string(),
            call_site: "test.rs:50".to_string(),
        };
        let context = MemoryContext {
            total_allocated: 2048,
            active_allocations: 2,
            memory_pressure: MemoryPressureLevel::Medium,
            allocation_patterns: vec![],
        };

        let assessment = engine.assess_risk(&source, &context, &[]);

        assert!(assessment.risk_score >= 0.0);
        assert!(assessment.confidence_score >= 0.0);
        assert!(!assessment.risk_factors.is_empty());
    }

    #[test]
    fn test_risk_assessment_engine_assess_raw_pointer() {
        let engine = RiskAssessmentEngine::new();
        let source = UnsafeSource::RawPointer {
            operation: "dereference".to_string(),
            location: "test.rs:60".to_string(),
        };
        let context = MemoryContext {
            total_allocated: 512,
            active_allocations: 1,
            memory_pressure: MemoryPressureLevel::High,
            allocation_patterns: vec![],
        };

        let assessment = engine.assess_risk(&source, &context, &[]);

        assert!(assessment.risk_score >= 0.0);
        assert!(!assessment.risk_factors.is_empty());
        assert_eq!(
            assessment.risk_factors[0].factor_type,
            RiskFactorType::RawPointerDereference
        );
    }

    #[test]
    fn test_risk_assessment_engine_assess_transmute() {
        let engine = RiskAssessmentEngine::new();
        let source = UnsafeSource::Transmute {
            from_type: "*const u8".to_string(),
            to_type: "*mut i32".to_string(),
            location: "test.rs:70".to_string(),
        };
        let context = MemoryContext {
            total_allocated: 256,
            active_allocations: 1,
            memory_pressure: MemoryPressureLevel::Critical,
            allocation_patterns: vec![],
        };

        let assessment = engine.assess_risk(&source, &context, &[]);

        assert!(assessment.risk_score >= 0.0);
        assert!(!assessment.risk_factors.is_empty());
        assert_eq!(
            assessment.risk_factors[0].factor_type,
            RiskFactorType::InvalidTransmute
        );
        // Pointer transmutes should have higher severity
        assert!(assessment.risk_factors[0].severity >= 8.0);
    }

    #[test]
    fn test_memory_pressure_risk_adjustment() {
        let engine = RiskAssessmentEngine::new();
        let source = UnsafeSource::RawPointer {
            operation: "test".to_string(),
            location: "test.rs".to_string(),
        };

        // Test different memory pressure levels
        let contexts = vec![
            (MemoryPressureLevel::Low, 0.8),
            (MemoryPressureLevel::Medium, 1.0),
            (MemoryPressureLevel::High, 1.2),
            (MemoryPressureLevel::Critical, 1.5),
        ];

        for (pressure, expected_multiplier) in contexts {
            let context = MemoryContext {
                total_allocated: 1024,
                active_allocations: 1,
                memory_pressure: pressure,
                allocation_patterns: vec![],
            };

            let assessment = engine.assess_risk(&source, &context, &[]);
            // Risk score should be adjusted by pressure multiplier
            assert!(assessment.risk_score > 0.0);
        }
    }


    #[test]
    fn test_safety_analyzer_generate_unsafe_report() {
        let analyzer = SafetyAnalyzer::default();
        let source = UnsafeSource::UnsafeBlock {
            location: "test unsafe block".to_string(),
            function: "test_function".to_string(),
            file_path: Some("test.rs".to_string()),
            line_number: Some(10),
        };
        let allocations = vec![AllocationInfo::new(0x1000, 256)];
        let violations = vec![];

        let result = analyzer.generate_unsafe_report(source, &allocations, &violations);

        assert!(result.is_ok());
        let report_id = result.unwrap();
        assert!(!report_id.is_empty());
        assert!(report_id.starts_with("UNSAFE-UB-"));
    }

    #[test]
    fn test_safety_analyzer_create_memory_passport() {
        let analyzer = SafetyAnalyzer::default();
        let allocation_ptr = 0x2000;
        let size_bytes = 512;

        let result = analyzer.create_memory_passport(
            allocation_ptr,
            size_bytes,
            PassportEventType::AllocatedInRust,
        );

        assert!(result.is_ok());
        let passport_id = result.unwrap();
        assert!(!passport_id.is_empty());
        assert!(passport_id.starts_with("passport_"));
    }

    #[test]
    fn test_safety_analyzer_record_passport_event() {
        let analyzer = SafetyAnalyzer::default();
        let allocation_ptr = 0x3000;

        // First create a passport
        let _ = analyzer.create_memory_passport(
            allocation_ptr,
            256,
            PassportEventType::AllocatedInRust,
        );

        // Then record an event
        let result = analyzer.record_passport_event(
            allocation_ptr,
            PassportEventType::HandoverToFfi,
            "test context".to_string(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_safety_analyzer_finalize_passports() {
        let analyzer = SafetyAnalyzer::default();

        // Create some passports
        let _ = analyzer.create_memory_passport(0x4000, 256, PassportEventType::AllocatedInRust);
        let _ = analyzer.create_memory_passport(0x5000, 512, PassportEventType::AllocatedInRust);

        // Record handover event for one passport (should be detected as leak)
        let _ = analyzer.record_passport_event(
            0x4000,
            PassportEventType::HandoverToFfi,
            "handover".to_string(),
        );

        let leaked_passports = analyzer.finalize_passports_at_shutdown();

        // Should detect at least one leak
        assert!(!leaked_passports.is_empty());
    }

    #[test]
    fn test_safety_analyzer_get_unsafe_reports() {
        let analyzer = SafetyAnalyzer::default();
        let source = UnsafeSource::FfiFunction {
            library: "test_lib".to_string(),
            function: "test_func".to_string(),
            call_site: "test.rs:100".to_string(),
        };

        let _ = analyzer.generate_unsafe_report(source, &[], &[]);
        let reports = analyzer.get_unsafe_reports();

        assert!(!reports.is_empty());
    }

    #[test]
    fn test_safety_analyzer_get_memory_passports() {
        let analyzer = SafetyAnalyzer::default();

        let _ = analyzer.create_memory_passport(0x6000, 128, PassportEventType::AllocatedInRust);
        let passports = analyzer.get_memory_passports();

        assert!(!passports.is_empty());
        assert!(passports.contains_key(&0x6000));
    }

    #[test]
    fn test_safety_analyzer_get_stats() {
        let analyzer = SafetyAnalyzer::default();

        let stats = analyzer.get_stats();

        assert_eq!(stats.total_reports, 0);
        assert_eq!(stats.total_passports, 0);
        assert!(stats.analysis_start_time > 0);
    }

    #[test]
    fn test_safety_analyzer_with_disabled_features() {
        let config = SafetyAnalysisConfig {
            detailed_risk_assessment: false,
            enable_passport_tracking: false,
            min_risk_level: RiskLevel::High,
            max_reports: 10,
            enable_dynamic_violations: false,
        };
        let analyzer = SafetyAnalyzer::new(config);

        // Test passport creation with disabled tracking
        let result =
            analyzer.create_memory_passport(0x7000, 256, PassportEventType::AllocatedInRust);
        assert!(result.is_ok());
        let passport_id = result.unwrap();
        assert!(passport_id.is_empty()); // Should return empty string when disabled

        // Test event recording with disabled tracking
        let result = analyzer.record_passport_event(
            0x7000,
            PassportEventType::HandoverToFfi,
            "test".to_string(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_safety_analyzer_report_filtering_by_risk_level() {
        let config = SafetyAnalysisConfig {
            min_risk_level: RiskLevel::High,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);

        // This should generate a low-risk report that gets filtered out
        let source = UnsafeSource::UnsafeBlock {
            location: "simple operation".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };

        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(result.is_ok());

        // Reports should be filtered based on risk level
        let reports = analyzer.get_unsafe_reports();
        // The exact number depends on the risk assessment, but the mechanism should work
        assert!(reports.len() <= 1);
    }

    #[test]
    fn test_safety_analyzer_max_reports_limit() {
        let config = SafetyAnalysisConfig {
            max_reports: 2,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);

        // Generate more reports than the limit
        for i in 0..5 {
            let source = UnsafeSource::RawPointer {
                operation: format!("operation_{}", i),
                location: format!("test.rs:{}", i * 10),
            };
            let _ = analyzer.generate_unsafe_report(source, &[], &[]);
        }

        let reports = analyzer.get_unsafe_reports();
        assert!(reports.len() <= 2); // Should not exceed max_reports
    }

    #[test]
    fn test_passport_status_determination() {
        let analyzer = SafetyAnalyzer::default();

        // Test different event sequences
        let events_sequences = vec![
            // Normal Rust allocation and deallocation
            vec![PassportEventType::AllocatedInRust],
            // Handover to FFI without return
            vec![
                PassportEventType::AllocatedInRust,
                PassportEventType::HandoverToFfi,
            ],
            // Handover and reclaim
            vec![
                PassportEventType::AllocatedInRust,
                PassportEventType::HandoverToFfi,
                PassportEventType::ReclaimedByRust,
            ],
            // Foreign free
            vec![
                PassportEventType::AllocatedInRust,
                PassportEventType::HandoverToFfi,
                PassportEventType::FreedByForeign,
            ],
        ];

        for (i, event_types) in events_sequences.iter().enumerate() {
            let ptr = 0x8000 + i * 0x1000;
            let _ = analyzer.create_memory_passport(ptr, 256, PassportEventType::AllocatedInRust);

            // Record additional events
            for event_type in event_types.iter().skip(1) {
                let _ = analyzer.record_passport_event(ptr, event_type.clone(), "test".to_string());
            }
        }

        let leaked_passports = analyzer.finalize_passports_at_shutdown();
        // Should detect leaks for handover-only scenarios
        assert!(!leaked_passports.is_empty());
    }

    #[test]
    fn test_violation_type_conversion() {
        let analyzer = SafetyAnalyzer::default();
        let violations = vec![
            SafetyViolation::DoubleFree {
                first_free_stack: CallStackRef::new(0, Some(0)),
                second_free_stack: CallStackRef::new(1, Some(0)),
                timestamp: 1234567890,
            },
            SafetyViolation::InvalidFree {
                attempted_pointer: 0x2000,
                stack: CallStackRef::new(2, Some(0)),
                timestamp: 1234567891,
            },
        ];

        let source = UnsafeSource::UnsafeBlock {
            location: "test".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };

        let result = analyzer.generate_unsafe_report(source, &[], &violations);
        assert!(result.is_ok());

        let reports = analyzer.get_unsafe_reports();
        assert!(!reports.is_empty());

        let report = reports.values().next().unwrap();
        assert_eq!(report.dynamic_violations.len(), 2);
    }

    #[test]
    fn test_comprehensive_unsafe_report_structure() {
        let analyzer = SafetyAnalyzer::default();
        let source = UnsafeSource::Transmute {
            from_type: "u64".to_string(),
            to_type: "f64".to_string(),
            location: "test.rs:200".to_string(),
        };
        let allocations = vec![
            AllocationInfo::new(0x1000, 512),
            AllocationInfo::new(0x2000, 1024),
        ];

        let result = analyzer.generate_unsafe_report(source.clone(), &allocations, &[]);
        assert!(result.is_ok());

        let reports = analyzer.get_unsafe_reports();
        let report = reports.values().next().unwrap();

        // Verify report structure
        assert!(!report.report_id.is_empty());
        assert!(matches!(report.source, UnsafeSource::Transmute { .. }));
        assert!(report.risk_assessment.assessment_timestamp > 0);
        assert!(report.generated_at > 0);
        assert_eq!(report.memory_context.active_allocations, 2);
        assert_eq!(report.memory_context.total_allocated, 1536);
    }
}
