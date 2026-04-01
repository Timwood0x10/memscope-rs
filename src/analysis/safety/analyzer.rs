use crate::analysis::safety::engine::RiskAssessmentEngine;
use crate::analysis::safety::types::*;
use crate::analysis::unsafe_ffi_tracker::{RiskLevel, SafetyViolation, StackFrame};
use crate::capture::types::{AllocationInfo, TrackingResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct SafetyAnalysisConfig {
    pub detailed_risk_assessment: bool,
    pub enable_passport_tracking: bool,
    pub min_risk_level: RiskLevel,
    pub max_reports: usize,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SafetyAnalysisStats {
    pub total_reports: usize,
    pub reports_by_risk_level: HashMap<String, usize>,
    pub total_passports: usize,
    pub passports_by_status: HashMap<String, usize>,
    pub dynamic_violations: usize,
    pub analysis_start_time: u64,
}

pub struct SafetyAnalyzer {
    unsafe_reports: Arc<Mutex<HashMap<String, UnsafeReport>>>,
    memory_passports: Arc<Mutex<HashMap<usize, MemoryPassport>>>,
    risk_engine: RiskAssessmentEngine,
    config: SafetyAnalysisConfig,
    stats: Arc<Mutex<SafetyAnalysisStats>>,
}

impl SafetyAnalyzer {
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

    pub fn generate_unsafe_report(
        &self,
        source: UnsafeSource,
        allocations: &[AllocationInfo],
        violations: &[SafetyViolation],
    ) -> TrackingResult<String> {
        let report_id = self.generate_report_id(&source);

        tracing::info!("🔍 Generating unsafe report: {}", report_id);

        let memory_context = self.create_memory_context(allocations);
        let call_stack = self.capture_call_stack()?;

        let risk_assessment = if self.config.detailed_risk_assessment {
            self.risk_engine
                .assess_risk(&source, &memory_context, &call_stack)
        } else {
            self.create_basic_risk_assessment(&source)
        };

        if !self.should_generate_report(&risk_assessment.risk_level) {
            return Ok(report_id);
        }

        let dynamic_violations = self.convert_safety_violations(violations);

        let related_passports = if self.config.enable_passport_tracking {
            self.find_related_passports(&source, allocations)
        } else {
            Vec::new()
        };

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

        if let Ok(mut reports) = self.unsafe_reports.lock() {
            if reports.len() >= self.config.max_reports {
                if let Some(oldest_id) = reports.keys().next().cloned() {
                    reports.remove(&oldest_id);
                }
            }
            reports.insert(report_id.clone(), report);
        }

        self.update_stats(&report_id, &risk_assessment.risk_level);

        tracing::info!(
            "✅ Generated unsafe report: {} (risk: {:?})",
            report_id,
            risk_assessment.risk_level
        );

        Ok(report_id)
    }

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

        let initial_passport_event = PassportEvent {
            event_type: initial_event,
            timestamp: current_time,
            context: "SafetyAnalyzer".to_string(),
            call_stack,
            metadata: HashMap::new(),
        };

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

    pub fn finalize_passports_at_shutdown(&self) -> Vec<String> {
        let mut leaked_passports = Vec::new();

        if let Ok(mut passports) = self.memory_passports.lock() {
            for (ptr, passport) in passports.iter_mut() {
                let final_status = self.determine_final_passport_status(&passport.lifecycle_events);
                passport.status_at_shutdown = final_status.clone();

                if matches!(final_status, PassportStatus::InForeignCustody) {
                    leaked_passports.push(passport.passport_id.clone());
                    tracing::warn!(
                        "🚨 Memory leak detected: passport {} (0x{:x}) in foreign custody",
                        passport.passport_id,
                        ptr
                    );
                }
            }

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

    pub fn get_unsafe_reports(&self) -> HashMap<String, UnsafeReport> {
        self.unsafe_reports
            .lock()
            .unwrap_or_else(|_| {
                tracing::error!("Failed to lock unsafe reports");
                std::process::exit(1);
            })
            .clone()
    }

    pub fn get_memory_passports(&self) -> HashMap<usize, MemoryPassport> {
        self.memory_passports
            .lock()
            .unwrap_or_else(|_| {
                tracing::error!("Failed to lock memory passports");
                std::process::exit(1);
            })
            .clone()
    }

    pub fn get_stats(&self) -> SafetyAnalysisStats {
        self.stats
            .lock()
            .unwrap_or_else(|_| {
                tracing::error!("Failed to lock stats");
                std::process::exit(1);
            })
            .clone()
    }

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
            allocation_patterns: Vec::new(),
        }
    }

    fn capture_call_stack(&self) -> TrackingResult<Vec<StackFrame>> {
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
            .map(|v| match v {
                SafetyViolation::DoubleFree { timestamp, .. } => DynamicViolation {
                    violation_type: ViolationType::DoubleFree,
                    memory_address: 0,
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
            })
            .collect()
    }

    fn find_related_passports(
        &self,
        _source: &UnsafeSource,
        _allocations: &[AllocationInfo],
    ) -> Vec<String> {
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
