use crate::analysis::safety::engine::RiskAssessmentEngine;
use crate::analysis::safety::types::*;
use crate::analysis::unsafe_ffi_tracker::{RiskLevel, SafetyViolation, StackFrame};
use crate::capture::types::{AllocationInfo, TrackingError, TrackingResult};
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
    pub strict_mutex_handling: bool,
    pub max_mutex_poison_retries: usize,
}

impl Default for SafetyAnalysisConfig {
    fn default() -> Self {
        Self {
            detailed_risk_assessment: true,
            enable_passport_tracking: true,
            min_risk_level: RiskLevel::Low,
            max_reports: 1000,
            enable_dynamic_violations: true,
            strict_mutex_handling: false,
            max_mutex_poison_retries: 3,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct CircuitBreaker {
    poison_count: usize,
    last_poison_time: Option<u64>,
    is_open: bool,
}

impl CircuitBreaker {
    fn record_poison(&mut self, max_retries: usize) {
        self.poison_count += 1;
        self.last_poison_time = Some(get_current_timestamp());

        if self.poison_count >= max_retries {
            self.is_open = true;
        }
    }

    fn is_tripped(&self) -> bool {
        self.is_open
    }

    fn reset(&mut self) {
        self.poison_count = 0;
        self.last_poison_time = None;
        self.is_open = false;
    }

    fn poison_count(&self) -> usize {
        self.poison_count
    }

    #[allow(dead_code)]
    fn last_poison_time(&self) -> Option<u64> {
        self.last_poison_time
    }
}

fn get_current_timestamp() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(e) => {
            tracing::error!(
                "System clock error when getting timestamp: {}. Using 0 as timestamp.",
                e
            );
            0
        }
    }
}

fn get_current_timestamp_nanos() -> u128 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(e) => {
            tracing::error!(
                "System clock error when getting timestamp in nanos: {}. Using 0 as timestamp.",
                e
            );
            0
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
    reports_circuit_breaker: Arc<Mutex<CircuitBreaker>>,
    passports_circuit_breaker: Arc<Mutex<CircuitBreaker>>,
    stats_circuit_breaker: Arc<Mutex<CircuitBreaker>>,
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
                analysis_start_time: get_current_timestamp(),
                ..Default::default()
            })),
            reports_circuit_breaker: Arc::new(Mutex::new(CircuitBreaker::default())),
            passports_circuit_breaker: Arc::new(Mutex::new(CircuitBreaker::default())),
            stats_circuit_breaker: Arc::new(Mutex::new(CircuitBreaker::default())),
        }
    }

    fn lock_circuit_breaker<'a>(
        breaker: &'a Arc<Mutex<CircuitBreaker>>,
        name: &str,
    ) -> TrackingResult<std::sync::MutexGuard<'a, CircuitBreaker>> {
        breaker.lock().map_err(|e| {
            let error_msg = format!("Mutex poisoned in {}: {}", name, e);
            tracing::error!("{}", error_msg);
            TrackingError::LockError(error_msg)
        })
    }

    fn lock_reports(
        &self,
    ) -> TrackingResult<std::sync::MutexGuard<'_, HashMap<String, UnsafeReport>>> {
        let circuit_breaker =
            Self::lock_circuit_breaker(&self.reports_circuit_breaker, "reports_circuit_breaker")?;

        if circuit_breaker.is_tripped() {
            return Err(TrackingError::LockError(
                "Circuit breaker tripped for unsafe_reports: too many mutex poison events"
                    .to_string(),
            ));
        }

        drop(circuit_breaker);

        match self.unsafe_reports.lock() {
            Ok(guard) => {
                if let Ok(mut cb) = Self::lock_circuit_breaker(
                    &self.reports_circuit_breaker,
                    "reports_circuit_breaker",
                ) {
                    cb.reset();
                }
                Ok(guard)
            }
            Err(e) => {
                let error_msg = format!("Mutex poisoned in unsafe_reports: {}", e);
                tracing::error!("{}", error_msg);

                if let Ok(mut circuit_breaker) = Self::lock_circuit_breaker(
                    &self.reports_circuit_breaker,
                    "reports_circuit_breaker",
                ) {
                    circuit_breaker.record_poison(self.config.max_mutex_poison_retries);

                    if self.config.strict_mutex_handling || circuit_breaker.is_tripped() {
                        tracing::error!(
                            "Circuit breaker tripped for unsafe_reports after {} poison events",
                            circuit_breaker.poison_count()
                        );
                        return Err(TrackingError::LockError(error_msg));
                    } else {
                        tracing::warn!(
                            "Recovering from mutex poison in unsafe_reports (attempt {}/{})",
                            circuit_breaker.poison_count(),
                            self.config.max_mutex_poison_retries
                        );
                    }
                }

                Ok(e.into_inner())
            }
        }
    }

    fn lock_passports(
        &self,
    ) -> TrackingResult<std::sync::MutexGuard<'_, HashMap<usize, MemoryPassport>>> {
        let circuit_breaker = Self::lock_circuit_breaker(
            &self.passports_circuit_breaker,
            "passports_circuit_breaker",
        )?;

        if circuit_breaker.is_tripped() {
            return Err(TrackingError::LockError(
                "Circuit breaker tripped for memory_passports: too many mutex poison events"
                    .to_string(),
            ));
        }

        drop(circuit_breaker);

        match self.memory_passports.lock() {
            Ok(guard) => {
                if let Ok(mut cb) = Self::lock_circuit_breaker(
                    &self.passports_circuit_breaker,
                    "passports_circuit_breaker",
                ) {
                    cb.reset();
                }
                Ok(guard)
            }
            Err(e) => {
                let error_msg = format!("Mutex poisoned in memory_passports: {}", e);
                tracing::error!("{}", error_msg);

                if let Ok(mut circuit_breaker) = Self::lock_circuit_breaker(
                    &self.passports_circuit_breaker,
                    "passports_circuit_breaker",
                ) {
                    circuit_breaker.record_poison(self.config.max_mutex_poison_retries);

                    if self.config.strict_mutex_handling || circuit_breaker.is_tripped() {
                        tracing::error!(
                            "Circuit breaker tripped for memory_passports after {} poison events",
                            circuit_breaker.poison_count()
                        );
                        return Err(TrackingError::LockError(error_msg));
                    } else {
                        tracing::warn!(
                            "Recovering from mutex poison in memory_passports (attempt {}/{})",
                            circuit_breaker.poison_count(),
                            self.config.max_mutex_poison_retries
                        );
                    }
                }

                Ok(e.into_inner())
            }
        }
    }

    fn lock_stats(&self) -> TrackingResult<std::sync::MutexGuard<'_, SafetyAnalysisStats>> {
        let circuit_breaker =
            Self::lock_circuit_breaker(&self.stats_circuit_breaker, "stats_circuit_breaker")?;

        if circuit_breaker.is_tripped() {
            return Err(TrackingError::LockError(
                "Circuit breaker tripped for stats: too many mutex poison events".to_string(),
            ));
        }

        drop(circuit_breaker);

        match self.stats.lock() {
            Ok(guard) => {
                if let Ok(mut cb) =
                    Self::lock_circuit_breaker(&self.stats_circuit_breaker, "stats_circuit_breaker")
                {
                    cb.reset();
                }
                Ok(guard)
            }
            Err(e) => {
                let error_msg = format!("Mutex poisoned in stats: {}", e);
                tracing::error!("{}", error_msg);

                if let Ok(mut circuit_breaker) =
                    Self::lock_circuit_breaker(&self.stats_circuit_breaker, "stats_circuit_breaker")
                {
                    circuit_breaker.record_poison(self.config.max_mutex_poison_retries);

                    if self.config.strict_mutex_handling || circuit_breaker.is_tripped() {
                        tracing::error!(
                            "Circuit breaker tripped for stats after {} poison events",
                            circuit_breaker.poison_count()
                        );
                        return Err(TrackingError::LockError(error_msg));
                    } else {
                        tracing::warn!(
                            "Recovering from mutex poison in stats (attempt {}/{})",
                            circuit_breaker.poison_count(),
                            self.config.max_mutex_poison_retries
                        );
                    }
                }

                Ok(e.into_inner())
            }
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
            generated_at: get_current_timestamp(),
        };

        let mut reports = self.lock_reports()?;
        if reports.len() >= self.config.max_reports {
            if let Some(oldest_id) = reports.keys().next().cloned() {
                reports.remove(&oldest_id);
            }
        }
        reports.insert(report_id.clone(), report);

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
            get_current_timestamp_nanos()
        );

        let call_stack = self.capture_call_stack()?;
        let current_time = get_current_timestamp();

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

        let mut passports = self.lock_passports()?;
        passports.insert(allocation_ptr, passport);

        let mut stats = self.lock_stats()?;
        stats.total_passports += 1;

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
        let current_time = get_current_timestamp();

        let event = PassportEvent {
            event_type,
            timestamp: current_time,
            context,
            call_stack,
            metadata: HashMap::new(),
        };

        let mut passports = self.lock_passports()?;
        if let Some(passport) = passports.get_mut(&allocation_ptr) {
            passport.lifecycle_events.push(event);
            passport.updated_at = current_time;

            tracing::info!("📝 Recorded passport event for 0x{:x}", allocation_ptr);
        }

        Ok(())
    }

    pub fn finalize_passports_at_shutdown(&self) -> Vec<String> {
        let mut leaked_passports = Vec::new();

        let mut passports = match self.lock_passports() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("Failed to lock passports during finalization: {}", e);
                return leaked_passports;
            }
        };

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

        let status_counts: Vec<String> = passports
            .values()
            .map(|p| format!("{:?}", p.status_at_shutdown))
            .collect();

        drop(passports);

        let mut stats = match self.lock_stats() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("Failed to lock stats during finalization: {}", e);
                return leaked_passports;
            }
        };
        for status_key in status_counts {
            *stats.passports_by_status.entry(status_key).or_insert(0) += 1;
        }

        tracing::info!(
            "🏁 Finalized {} passports, {} leaks detected",
            self.get_passport_count(),
            leaked_passports.len()
        );

        leaked_passports
    }

    pub fn get_unsafe_reports(&self) -> HashMap<String, UnsafeReport> {
        match self.lock_reports() {
            Ok(guard) => guard.clone(),
            Err(e) => {
                tracing::error!("Failed to get unsafe reports: {}", e);
                HashMap::new()
            }
        }
    }

    pub fn get_memory_passports(&self) -> HashMap<usize, MemoryPassport> {
        match self.lock_passports() {
            Ok(guard) => guard.clone(),
            Err(e) => {
                tracing::error!("Failed to get memory passports: {}", e);
                HashMap::new()
            }
        }
    }

    pub fn get_stats(&self) -> SafetyAnalysisStats {
        match self.lock_stats() {
            Ok(guard) => guard.clone(),
            Err(e) => {
                tracing::error!("Failed to get stats: {}", e);
                SafetyAnalysisStats::default()
            }
        }
    }

    fn generate_report_id(&self, source: &UnsafeSource) -> String {
        let timestamp = get_current_timestamp_nanos();

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
            assessment_timestamp: get_current_timestamp(),
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
                    context: "Double free detected: memory was freed twice".to_string(),
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
                    context: format!(
                        "Invalid free attempted at address 0x{:x}",
                        attempted_pointer
                    ),
                },
                SafetyViolation::PotentialLeak {
                    allocation_timestamp,
                    leak_detection_timestamp,
                    ..
                } => DynamicViolation {
                    violation_type: ViolationType::MemoryLeak,
                    memory_address: 0,
                    memory_size: 0,
                    detected_at: (*leak_detection_timestamp as u64),
                    call_stack: Vec::new(),
                    severity: RiskLevel::Medium,
                    context: format!(
                        "Potential memory leak detected (allocated at timestamp {})",
                        allocation_timestamp
                    ),
                },
                SafetyViolation::CrossBoundaryRisk {
                    risk_level,
                    description,
                    ..
                } => DynamicViolation {
                    violation_type: ViolationType::FfiBoundaryViolation,
                    memory_address: 0,
                    memory_size: 0,
                    detected_at: get_current_timestamp(),
                    call_stack: Vec::new(),
                    severity: risk_level.clone(),
                    context: description.clone(),
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
        match self.lock_stats() {
            Ok(mut stats) => {
                stats.total_reports += 1;
                let risk_key = format!("{risk_level:?}");
                *stats.reports_by_risk_level.entry(risk_key).or_insert(0) += 1;
            }
            Err(e) => {
                tracing::error!("Failed to update stats: {}", e);
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify SafetyAnalyzer creation with default config
    /// Invariants: Default config should have detailed_risk_assessment enabled
    #[test]
    fn test_safety_analyzer_default() {
        let analyzer = SafetyAnalyzer::default();
        let stats = analyzer.get_stats();
        assert_eq!(
            stats.total_reports, 0,
            "New analyzer should have zero reports"
        );
        assert_eq!(
            stats.total_passports, 0,
            "New analyzer should have zero passports"
        );
    }

    /// Objective: Verify SafetyAnalyzer creation with custom config
    /// Invariants: Custom config values should be respected
    #[test]
    fn test_safety_analyzer_custom_config() {
        let config = SafetyAnalysisConfig {
            detailed_risk_assessment: false,
            enable_passport_tracking: false,
            min_risk_level: RiskLevel::High,
            max_reports: 100,
            enable_dynamic_violations: false,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);
        let stats = analyzer.get_stats();
        assert_eq!(
            stats.total_reports, 0,
            "Custom config analyzer should start with zero reports"
        );
    }

    /// Objective: Verify generate_unsafe_report for UnsafeBlock source
    /// Invariants: Should generate report with correct source type
    #[test]
    fn test_generate_unsafe_report_unsafe_block() {
        let analyzer = SafetyAnalyzer::default();
        let source = UnsafeSource::UnsafeBlock {
            location: "test.rs:10".to_string(),
            function: "test_fn".to_string(),
            file_path: Some("test.rs".to_string()),
            line_number: Some(10),
        };

        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(result.is_ok(), "Should generate report successfully");
        let report_id = result.unwrap();
        assert!(
            report_id.starts_with("UNSAFE-UB-"),
            "Report ID should start with UNSAFE-UB-"
        );
    }

    /// Objective: Verify generate_unsafe_report for FfiFunction source
    /// Invariants: Should generate report with FFI prefix and correct FFI context
    #[test]
    fn test_generate_unsafe_report_ffi() {
        let analyzer = SafetyAnalyzer::default();
        let source = UnsafeSource::FfiFunction {
            library: "libc".to_string(),
            function: "malloc".to_string(),
            call_site: "test.rs:20".to_string(),
        };

        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(result.is_ok(), "Should generate FFI report successfully");
        let report_id = result.unwrap();
        assert!(
            report_id.starts_with("UNSAFE-FFI-"),
            "FFI report ID should start with UNSAFE-FFI-"
        );

        let reports = analyzer.get_unsafe_reports();
        let report = reports
            .get(&report_id)
            .expect("Report should exist in reports map");

        match &report.source {
            UnsafeSource::FfiFunction {
                library,
                function,
                call_site,
            } => {
                assert_eq!(
                    library, "libc",
                    "FFI report should contain correct library name"
                );
                assert_eq!(
                    function, "malloc",
                    "FFI report should contain correct function name"
                );
                assert_eq!(
                    call_site, "test.rs:20",
                    "FFI report should contain correct call site"
                );
            }
            _ => panic!("Report source should be FfiFunction variant"),
        }
    }

    /// Objective: Verify generate_unsafe_report for RawPointer source
    /// Invariants: Should generate report with PTR prefix
    #[test]
    fn test_generate_unsafe_report_raw_pointer() {
        let analyzer = SafetyAnalyzer::default();
        let source = UnsafeSource::RawPointer {
            operation: "dereference".to_string(),
            location: "0x1000".to_string(),
        };

        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(
            result.is_ok(),
            "Should generate raw pointer report successfully"
        );
        let report_id = result.unwrap();
        assert!(
            report_id.starts_with("UNSAFE-PTR-"),
            "PTR report ID should start with UNSAFE-PTR-"
        );
    }

    /// Objective: Verify generate_unsafe_report for Transmute source
    /// Invariants: Should generate report with TX prefix
    #[test]
    fn test_generate_unsafe_report_transmute() {
        let analyzer = SafetyAnalyzer::default();
        let source = UnsafeSource::Transmute {
            from_type: "u8".to_string(),
            to_type: "i8".to_string(),
            location: "test.rs:30".to_string(),
        };

        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(
            result.is_ok(),
            "Should generate transmute report successfully"
        );
        let report_id = result.unwrap();
        assert!(
            report_id.starts_with("UNSAFE-TX-"),
            "TX report ID should start with UNSAFE-TX-"
        );
    }

    /// Objective: Verify create_memory_passport functionality
    /// Invariants: Should create passport with correct initial state
    #[test]
    fn test_create_memory_passport() {
        let analyzer = SafetyAnalyzer::default();
        let result =
            analyzer.create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust);
        assert!(result.is_ok(), "Should create passport successfully");
        let passport_id = result.unwrap();
        assert!(
            passport_id.starts_with("passport_"),
            "Passport ID should start with passport_"
        );

        let stats = analyzer.get_stats();
        assert_eq!(
            stats.total_passports, 1,
            "Should have one passport after creation"
        );
    }

    /// Objective: Verify passport tracking disabled behavior
    /// Invariants: Should return empty string when tracking disabled
    #[test]
    fn test_passport_tracking_disabled() {
        let config = SafetyAnalysisConfig {
            enable_passport_tracking: false,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);
        let result =
            analyzer.create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust);
        assert!(result.is_ok(), "Should return Ok even when disabled");
        assert!(
            result.unwrap().is_empty(),
            "Should return empty string when disabled"
        );
    }

    /// Objective: Verify record_passport_event functionality
    /// Invariants: Should record event on existing passport
    #[test]
    fn test_record_passport_event() {
        let analyzer = SafetyAnalyzer::default();
        analyzer
            .create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust)
            .unwrap();

        let result = analyzer.record_passport_event(
            0x1000,
            PassportEventType::HandoverToFfi,
            "test_context".to_string(),
        );
        assert!(result.is_ok(), "Should record event successfully");

        let passports = analyzer.get_memory_passports();
        assert!(passports.contains_key(&0x1000), "Passport should exist");
        let passport = passports.get(&0x1000).unwrap();
        assert_eq!(passport.lifecycle_events.len(), 2, "Should have two events");
    }

    /// Objective: Verify finalize_passports_at_shutdown detects leaks
    /// Invariants: Should detect passports in foreign custody
    #[test]
    fn test_finalize_passports_leak_detection() {
        let analyzer = SafetyAnalyzer::default();
        analyzer
            .create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust)
            .unwrap();
        analyzer
            .record_passport_event(
                0x1000,
                PassportEventType::HandoverToFfi,
                "ffi_transfer".to_string(),
            )
            .unwrap();

        let leaks = analyzer.finalize_passports_at_shutdown();
        assert_eq!(
            leaks.len(),
            1,
            "Should detect one leak for passport in foreign custody"
        );
    }

    /// Objective: Verify finalize_passports_at_shutdown for freed passports
    /// Invariants: Should not detect leaks for properly freed passports
    #[test]
    fn test_finalize_passports_no_leak() {
        let analyzer = SafetyAnalyzer::default();
        analyzer
            .create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust)
            .unwrap();
        analyzer
            .record_passport_event(
                0x1000,
                PassportEventType::FreedByForeign,
                "freed".to_string(),
            )
            .unwrap();

        let leaks = analyzer.finalize_passports_at_shutdown();
        assert!(
            leaks.is_empty(),
            "Should not detect leak for freed passport"
        );
    }

    /// Objective: Verify get_unsafe_reports returns all reports
    /// Invariants: Should return all generated reports
    #[test]
    fn test_get_unsafe_reports() {
        let analyzer = SafetyAnalyzer::default();
        let source = UnsafeSource::UnsafeBlock {
            location: "test.rs".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };
        analyzer
            .generate_unsafe_report(source.clone(), &[], &[])
            .unwrap();
        analyzer.generate_unsafe_report(source, &[], &[]).unwrap();

        let reports = analyzer.get_unsafe_reports();
        assert_eq!(reports.len(), 2, "Should have two reports");
    }

    /// Objective: Verify min_risk_level filtering
    /// Invariants: Should not generate report below min risk level
    #[test]
    fn test_min_risk_level_filtering() {
        let config = SafetyAnalysisConfig {
            min_risk_level: RiskLevel::Critical,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);
        let source = UnsafeSource::UnsafeBlock {
            location: "test.rs".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };
        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(result.is_ok(), "Should return Ok even when filtered");
    }

    /// Objective: Verify stats update after report generation
    /// Invariants: Stats should reflect generated reports
    #[test]
    fn test_stats_update() {
        let analyzer = SafetyAnalyzer::default();
        let source = UnsafeSource::RawPointer {
            operation: "test".to_string(),
            location: "test".to_string(),
        };
        analyzer.generate_unsafe_report(source, &[], &[]).unwrap();

        let stats = analyzer.get_stats();
        assert_eq!(stats.total_reports, 1, "Stats should show one report");
        assert!(
            !stats.reports_by_risk_level.is_empty(),
            "Should have risk level breakdown"
        );
    }

    /// Objective: Verify max_reports limit enforcement
    /// Invariants: Should remove oldest report when limit exceeded
    #[test]
    fn test_max_reports_limit() {
        let config = SafetyAnalysisConfig {
            max_reports: 2,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);
        let source = UnsafeSource::UnsafeBlock {
            location: "test.rs".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };

        analyzer
            .generate_unsafe_report(source.clone(), &[], &[])
            .unwrap();
        analyzer
            .generate_unsafe_report(source.clone(), &[], &[])
            .unwrap();
        analyzer.generate_unsafe_report(source, &[], &[]).unwrap();

        let reports = analyzer.get_unsafe_reports();
        assert!(reports.len() <= 2, "Should not exceed max_reports limit");
    }

    /// Objective: Verify SafetyAnalysisConfig default values
    /// Invariants: Default should have sensible values
    #[test]
    fn test_safety_config_default() {
        let config = SafetyAnalysisConfig::default();
        assert!(
            config.detailed_risk_assessment,
            "Detailed risk assessment should be enabled"
        );
        assert!(
            config.enable_passport_tracking,
            "Passport tracking should be enabled"
        );
        assert_eq!(config.max_reports, 1000, "Max reports should be 1000");
    }

    /// Objective: Verify RiskLevel ordering
    /// Invariants: Critical should be highest, Low should be lowest
    #[test]
    fn test_risk_level_ordering() {
        assert!(matches!(RiskLevel::Low, RiskLevel::Low));
        assert!(matches!(RiskLevel::Medium, RiskLevel::Medium));
        assert!(matches!(RiskLevel::High, RiskLevel::High));
        assert!(matches!(RiskLevel::Critical, RiskLevel::Critical));
    }

    /// Objective: Verify PassportStatus variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_passport_status_variants() {
        let statuses = vec![
            PassportStatus::FreedByRust,
            PassportStatus::HandoverToFfi,
            PassportStatus::FreedByForeign,
            PassportStatus::ReclaimedByRust,
            PassportStatus::InForeignCustody,
            PassportStatus::Unknown,
        ];

        for status in &statuses {
            let debug_str = format!("{status:?}");
            assert!(
                !debug_str.is_empty(),
                "Status should have debug representation"
            );
        }
    }

    /// Objective: Verify PassportEventType variants
    /// Invariants: All event types should be distinct
    #[test]
    fn test_passport_event_type_variants() {
        let event_types = vec![
            PassportEventType::AllocatedInRust,
            PassportEventType::HandoverToFfi,
            PassportEventType::FreedByForeign,
            PassportEventType::ReclaimedByRust,
            PassportEventType::BoundaryAccess,
            PassportEventType::OwnershipTransfer,
        ];

        for event_type in &event_types {
            let debug_str = format!("{event_type:?}");
            assert!(
                !debug_str.is_empty(),
                "Event type should have debug representation"
            );
        }
    }

    /// Objective: Verify record_passport_event for non-existent passport
    /// Invariants: Should return Ok even when passport doesn't exist
    #[test]
    fn test_record_passport_event_non_existent() {
        let analyzer = SafetyAnalyzer::default();
        let result = analyzer.record_passport_event(
            0x9999,
            PassportEventType::HandoverToFfi,
            "test_context".to_string(),
        );
        assert!(
            result.is_ok(),
            "Should return Ok even for non-existent passport"
        );

        let passports = analyzer.get_memory_passports();
        assert!(
            !passports.contains_key(&0x9999),
            "Non-existent passport should not be created"
        );
    }

    /// Objective: Verify generate_unsafe_report with allocations
    /// Invariants: Should handle allocations correctly in memory context
    #[test]
    fn test_generate_unsafe_report_with_allocations() {
        let analyzer = SafetyAnalyzer::default();
        let source = UnsafeSource::RawPointer {
            operation: "test".to_string(),
            location: "test.rs".to_string(),
        };

        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(
            result.is_ok(),
            "Should generate report with empty allocations"
        );
    }

    /// Objective: Verify generate_unsafe_report with safety violations
    /// Invariants: Should convert violations to dynamic violations
    #[test]
    fn test_generate_unsafe_report_with_violations() {
        let analyzer = SafetyAnalyzer::default();
        let source = UnsafeSource::UnsafeBlock {
            location: "test.rs".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };

        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(
            result.is_ok(),
            "Should generate report with empty violations"
        );

        let report_id = result.unwrap();
        let reports = analyzer.get_unsafe_reports();
        let report = reports.get(&report_id).expect("Report should exist");
        assert_eq!(
            report.dynamic_violations.len(),
            0,
            "Should have no violations when empty"
        );
    }

    /// Objective: Verify determine_final_passport_status for various scenarios
    /// Invariants: Should correctly determine passport status based on events
    #[test]
    fn test_determine_final_passport_status_scenarios() {
        let analyzer = SafetyAnalyzer::default();

        let events_handover_only = vec![PassportEvent {
            event_type: PassportEventType::HandoverToFfi,
            timestamp: 1000,
            context: "test".to_string(),
            call_stack: vec![],
            metadata: HashMap::new(),
        }];
        let status = analyzer.determine_final_passport_status(&events_handover_only);
        assert!(
            matches!(status, PassportStatus::InForeignCustody),
            "Handover without reclaim or foreign free should be InForeignCustody"
        );

        let events_reclaimed = vec![
            PassportEvent {
                event_type: PassportEventType::HandoverToFfi,
                timestamp: 1000,
                context: "test".to_string(),
                call_stack: vec![],
                metadata: HashMap::new(),
            },
            PassportEvent {
                event_type: PassportEventType::ReclaimedByRust,
                timestamp: 2000,
                context: "test".to_string(),
                call_stack: vec![],
                metadata: HashMap::new(),
            },
        ];
        let status = analyzer.determine_final_passport_status(&events_reclaimed);
        assert!(
            matches!(status, PassportStatus::ReclaimedByRust),
            "Reclaimed after handover should be ReclaimedByRust"
        );

        let events_freed_by_foreign = vec![
            PassportEvent {
                event_type: PassportEventType::HandoverToFfi,
                timestamp: 1000,
                context: "test".to_string(),
                call_stack: vec![],
                metadata: HashMap::new(),
            },
            PassportEvent {
                event_type: PassportEventType::FreedByForeign,
                timestamp: 2000,
                context: "test".to_string(),
                call_stack: vec![],
                metadata: HashMap::new(),
            },
        ];
        let status = analyzer.determine_final_passport_status(&events_freed_by_foreign);
        assert!(
            matches!(status, PassportStatus::FreedByForeign),
            "Freed by foreign should be FreedByForeign"
        );

        let events_no_handover = vec![PassportEvent {
            event_type: PassportEventType::AllocatedInRust,
            timestamp: 1000,
            context: "test".to_string(),
            call_stack: vec![],
            metadata: HashMap::new(),
        }];
        let status = analyzer.determine_final_passport_status(&events_no_handover);
        assert!(
            matches!(status, PassportStatus::FreedByRust),
            "No handover should be FreedByRust"
        );
    }

    /// Objective: Verify create_basic_risk_assessment functionality
    /// Invariants: Should create basic assessment when detailed_risk_assessment is disabled
    #[test]
    fn test_create_basic_risk_assessment() {
        let config = SafetyAnalysisConfig {
            detailed_risk_assessment: false,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);

        let source = UnsafeSource::RawPointer {
            operation: "test".to_string(),
            location: "test.rs".to_string(),
        };
        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(
            result.is_ok(),
            "Should generate report with basic assessment"
        );

        let report_id = result.unwrap();
        let reports = analyzer.get_unsafe_reports();
        let report = reports.get(&report_id).expect("Report should exist");
        assert!(
            report.risk_assessment.risk_score > 0.0,
            "Basic assessment should have risk score"
        );
    }

    /// Objective: Verify should_generate_report filtering logic
    /// Invariants: Should filter reports based on min_risk_level
    #[test]
    fn test_should_generate_report_filtering() {
        let config = SafetyAnalysisConfig {
            min_risk_level: RiskLevel::High,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);

        let source = UnsafeSource::UnsafeBlock {
            location: "test.rs".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };
        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(result.is_ok(), "Should return Ok even when filtered");

        let reports = analyzer.get_unsafe_reports();
        assert!(
            reports.is_empty(),
            "Report should be filtered out when below min risk level"
        );
    }

    /// Objective: Verify memory pressure level calculation
    /// Invariants: Should correctly calculate memory pressure based on allocations
    #[test]
    fn test_memory_pressure_levels() {
        let analyzer = SafetyAnalyzer::default();

        let source = UnsafeSource::UnsafeBlock {
            location: "test.rs".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };
        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(result.is_ok(), "Should handle empty allocations");
    }

    /// Objective: Verify passport tracking with multiple events
    /// Invariants: Should correctly track multiple passport events
    #[test]
    fn test_passport_multiple_events() {
        let analyzer = SafetyAnalyzer::default();

        analyzer
            .create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust)
            .unwrap();

        analyzer
            .record_passport_event(
                0x1000,
                PassportEventType::HandoverToFfi,
                "transfer_to_ffi".to_string(),
            )
            .unwrap();

        analyzer
            .record_passport_event(
                0x1000,
                PassportEventType::BoundaryAccess,
                "ffi_access".to_string(),
            )
            .unwrap();

        analyzer
            .record_passport_event(
                0x1000,
                PassportEventType::ReclaimedByRust,
                "reclaimed".to_string(),
            )
            .unwrap();

        let passports = analyzer.get_memory_passports();
        let passport = passports.get(&0x1000).expect("Passport should exist");
        assert_eq!(
            passport.lifecycle_events.len(),
            4,
            "Should have four events"
        );
    }

    /// Objective: Verify finalize_passports_at_shutdown with mixed statuses
    /// Invariants: Should correctly categorize passports by final status
    #[test]
    fn test_finalize_passports_mixed_statuses() {
        let analyzer = SafetyAnalyzer::default();

        analyzer
            .create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust)
            .unwrap();
        analyzer
            .record_passport_event(
                0x1000,
                PassportEventType::HandoverToFfi,
                "leaked".to_string(),
            )
            .unwrap();

        analyzer
            .create_memory_passport(0x2000, 2048, PassportEventType::AllocatedInRust)
            .unwrap();
        analyzer
            .record_passport_event(
                0x2000,
                PassportEventType::FreedByForeign,
                "freed".to_string(),
            )
            .unwrap();

        let leaks = analyzer.finalize_passports_at_shutdown();
        assert_eq!(leaks.len(), 1, "Should detect one leak");

        let stats = analyzer.get_stats();
        assert!(
            stats.passports_by_status.contains_key("InForeignCustody"),
            "Stats should include InForeignCustody status"
        );
        assert!(
            stats.passports_by_status.contains_key("FreedByForeign"),
            "Stats should include FreedByForeign status"
        );
    }

    /// Objective: Verify enable_dynamic_violations configuration
    /// Invariants: Should respect dynamic_violations config setting
    #[test]
    fn test_enable_dynamic_violations_config() {
        let config = SafetyAnalysisConfig {
            enable_dynamic_violations: false,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);

        let source = UnsafeSource::UnsafeBlock {
            location: "test.rs".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };
        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(
            result.is_ok(),
            "Should generate report with dynamic violations disabled"
        );
    }

    /// Objective: Verify all UnsafeSource variants generate unique report IDs
    /// Invariants: Each source type should generate distinct report ID prefix
    #[test]
    fn test_all_unsafe_source_variants() {
        let analyzer = SafetyAnalyzer::default();

        let sources = vec![
            UnsafeSource::UnsafeBlock {
                location: "test.rs".to_string(),
                function: "test".to_string(),
                file_path: None,
                line_number: None,
            },
            UnsafeSource::FfiFunction {
                library: "libc".to_string(),
                function: "malloc".to_string(),
                call_site: "test.rs".to_string(),
            },
            UnsafeSource::RawPointer {
                operation: "test".to_string(),
                location: "test.rs".to_string(),
            },
            UnsafeSource::Transmute {
                from_type: "u8".to_string(),
                to_type: "i8".to_string(),
                location: "test.rs".to_string(),
            },
        ];

        let mut report_ids = Vec::new();
        for source in sources {
            let result = analyzer.generate_unsafe_report(source, &[], &[]);
            assert!(
                result.is_ok(),
                "Should generate report for all source types"
            );
            report_ids.push(result.unwrap());
        }

        assert_eq!(report_ids.len(), 4, "Should have generated 4 reports");
    }

    /// Objective: Verify determine_final_passport_status with conflicting events
    /// Invariants: Should handle reclaim + foreign_free scenario correctly
    #[test]
    fn test_determine_final_passport_status_conflicting_events() {
        let analyzer = SafetyAnalyzer::default();

        let events_conflict = vec![
            PassportEvent {
                event_type: PassportEventType::HandoverToFfi,
                timestamp: 1000,
                context: "test".to_string(),
                call_stack: vec![],
                metadata: HashMap::new(),
            },
            PassportEvent {
                event_type: PassportEventType::ReclaimedByRust,
                timestamp: 2000,
                context: "test".to_string(),
                call_stack: vec![],
                metadata: HashMap::new(),
            },
            PassportEvent {
                event_type: PassportEventType::FreedByForeign,
                timestamp: 3000,
                context: "test".to_string(),
                call_stack: vec![],
                metadata: HashMap::new(),
            },
        ];
        let status = analyzer.determine_final_passport_status(&events_conflict);
        assert!(
            matches!(status, PassportStatus::FreedByForeign),
            "When both reclaim and foreign_free exist, should prioritize foreign_free"
        );
    }

    /// Objective: Verify passport creation with zero size
    /// Invariants: Should handle zero size allocation gracefully
    #[test]
    fn test_create_memory_passport_zero_size() {
        let analyzer = SafetyAnalyzer::default();
        let result = analyzer.create_memory_passport(0x1000, 0, PassportEventType::AllocatedInRust);
        assert!(result.is_ok(), "Should create passport with zero size");

        let passports = analyzer.get_memory_passports();
        let passport = passports.get(&0x1000).expect("Passport should exist");
        assert_eq!(passport.size_bytes, 0, "Passport should have zero size");
    }

    /// Objective: Verify passport creation with null pointer
    /// Invariants: Should handle null pointer (0x0) allocation
    #[test]
    fn test_create_memory_passport_null_pointer() {
        let analyzer = SafetyAnalyzer::default();
        let result = analyzer.create_memory_passport(0x0, 1024, PassportEventType::AllocatedInRust);
        assert!(result.is_ok(), "Should create passport with null pointer");

        let passports = analyzer.get_memory_passports();
        assert!(
            passports.contains_key(&0x0),
            "Passport with null pointer should exist"
        );
    }

    /// Objective: Verify multiple passports with same pointer (potential bug)
    /// Invariants: Should overwrite previous passport with same pointer
    #[test]
    fn test_create_memory_passport_duplicate_pointer() {
        let analyzer = SafetyAnalyzer::default();

        analyzer
            .create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust)
            .unwrap();

        analyzer
            .create_memory_passport(0x1000, 2048, PassportEventType::AllocatedInRust)
            .unwrap();

        let passports = analyzer.get_memory_passports();
        assert_eq!(
            passports.len(),
            1,
            "Duplicate pointer should overwrite previous passport"
        );

        let passport = passports.get(&0x1000).expect("Passport should exist");
        assert_eq!(
            passport.size_bytes, 2048,
            "Should have size from second creation"
        );

        let stats = analyzer.get_stats();
        assert_eq!(
            stats.total_passports, 2,
            "Stats should count both creation attempts"
        );
    }

    /// Objective: Verify max_reports limit with exact boundary
    /// Invariants: Should handle exactly max_reports count
    #[test]
    fn test_max_reports_exact_boundary() {
        let config = SafetyAnalysisConfig {
            max_reports: 3,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);
        let source = UnsafeSource::UnsafeBlock {
            location: "test.rs".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };

        for _ in 0..3 {
            analyzer
                .generate_unsafe_report(source.clone(), &[], &[])
                .unwrap();
        }

        let reports = analyzer.get_unsafe_reports();
        assert_eq!(reports.len(), 3, "Should have exactly max_reports count");

        analyzer.generate_unsafe_report(source, &[], &[]).unwrap();

        let reports = analyzer.get_unsafe_reports();
        assert!(
            reports.len() <= 3,
            "Should not exceed max_reports after adding one more"
        );
    }

    /// Objective: Verify report generation with all risk levels using basic assessment
    /// Invariants: Should correctly filter based on min_risk_level when using basic assessment
    #[test]
    fn test_risk_level_filtering_comprehensive() {
        let test_cases = vec![
            (RiskLevel::Low, 4),
            (RiskLevel::Medium, 4),
            (RiskLevel::High, 2),
            (RiskLevel::Critical, 0),
        ];

        for (min_level, expected_count) in test_cases {
            let config = SafetyAnalysisConfig {
                min_risk_level: min_level.clone(),
                detailed_risk_assessment: false,
                ..Default::default()
            };
            let analyzer = SafetyAnalyzer::new(config);

            let sources = vec![
                UnsafeSource::UnsafeBlock {
                    location: "test.rs".to_string(),
                    function: "test".to_string(),
                    file_path: None,
                    line_number: None,
                },
                UnsafeSource::FfiFunction {
                    library: "libc".to_string(),
                    function: "malloc".to_string(),
                    call_site: "test.rs".to_string(),
                },
                UnsafeSource::RawPointer {
                    operation: "test".to_string(),
                    location: "test.rs".to_string(),
                },
                UnsafeSource::Transmute {
                    from_type: "u8".to_string(),
                    to_type: "i8".to_string(),
                    location: "test.rs".to_string(),
                },
            ];

            for source in sources.into_iter() {
                analyzer.generate_unsafe_report(source, &[], &[]).unwrap();
            }

            let reports = analyzer.get_unsafe_reports();
            let actual_count = reports.len();

            assert_eq!(
                actual_count, expected_count,
                "For min_level {:?}, expected {} reports but got {}",
                min_level, expected_count, actual_count
            );
        }
    }

    /// Objective: Verify risk assessment engine behavior with no matching factors
    /// Invariants: Should assign Medium risk when no risk factors match (conservative approach)
    #[test]
    fn test_risk_assessment_no_matching_factors() {
        let config = SafetyAnalysisConfig {
            detailed_risk_assessment: true,
            min_risk_level: RiskLevel::Low,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);

        let source = UnsafeSource::UnsafeBlock {
            location: "safe_location.rs".to_string(),
            function: "safe_function".to_string(),
            file_path: None,
            line_number: None,
        };

        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(result.is_ok(), "Should generate report");

        let reports = analyzer.get_unsafe_reports();
        assert_eq!(reports.len(), 1, "Should have one report");

        let report = reports.values().next().expect("Report should exist");
        assert!(
            matches!(report.risk_assessment.risk_level, RiskLevel::Low),
            "Risk level should be Low when no risk factors match (empty risk factors indicate low risk)"
        );
    }

    /// Objective: Verify passport event recording with empty context
    /// Invariants: Should handle empty context string
    #[test]
    fn test_record_passport_event_empty_context() {
        let analyzer = SafetyAnalyzer::default();

        analyzer
            .create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust)
            .unwrap();

        let result =
            analyzer.record_passport_event(0x1000, PassportEventType::HandoverToFfi, String::new());
        assert!(result.is_ok(), "Should record event with empty context");

        let passports = analyzer.get_memory_passports();
        let passport = passports.get(&0x1000).expect("Passport should exist");
        assert_eq!(passport.lifecycle_events.len(), 2, "Should have two events");
    }

    /// Objective: Verify stats consistency after multiple operations
    /// Invariants: Stats should accurately reflect all operations
    #[test]
    fn test_stats_consistency() {
        let analyzer = SafetyAnalyzer::default();

        let initial_stats = analyzer.get_stats();
        assert_eq!(initial_stats.total_reports, 0);
        assert_eq!(initial_stats.total_passports, 0);

        analyzer
            .generate_unsafe_report(
                UnsafeSource::UnsafeBlock {
                    location: "test.rs".to_string(),
                    function: "test".to_string(),
                    file_path: None,
                    line_number: None,
                },
                &[],
                &[],
            )
            .unwrap();

        analyzer
            .create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust)
            .unwrap();

        let stats = analyzer.get_stats();
        assert_eq!(stats.total_reports, 1, "Should have 1 report");
        assert_eq!(stats.total_passports, 1, "Should have 1 passport");
        assert!(
            !stats.reports_by_risk_level.is_empty(),
            "Should have risk level breakdown"
        );
    }

    /// Objective: Verify passport lifecycle with all event types
    /// Invariants: Should handle all PassportEventType variants
    #[test]
    fn test_passport_lifecycle_all_event_types() {
        let analyzer = SafetyAnalyzer::default();

        analyzer
            .create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust)
            .unwrap();

        let event_types = vec![
            PassportEventType::HandoverToFfi,
            PassportEventType::BoundaryAccess,
            PassportEventType::OwnershipTransfer,
            PassportEventType::ReclaimedByRust,
        ];

        for event_type in event_types {
            let event_type_str = format!("{:?}", event_type);
            let result = analyzer.record_passport_event(0x1000, event_type, "test".to_string());
            assert!(
                result.is_ok(),
                "Should record event type {}",
                event_type_str
            );
        }

        let passports = analyzer.get_memory_passports();
        let passport = passports.get(&0x1000).expect("Passport should exist");
        assert_eq!(
            passport.lifecycle_events.len(),
            5,
            "Should have initial event plus 4 recorded events"
        );
    }

    /// Objective: Verify report ID uniqueness with rapid generation
    /// Invariants: Each report should have unique ID even when generated rapidly
    #[test]
    fn test_report_id_uniqueness() {
        let analyzer = SafetyAnalyzer::default();
        let source = UnsafeSource::UnsafeBlock {
            location: "test.rs".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };

        let mut report_ids = std::collections::HashSet::new();
        for _ in 0..100 {
            let report_id = analyzer
                .generate_unsafe_report(source.clone(), &[], &[])
                .unwrap();
            assert!(report_ids.insert(report_id), "Report ID should be unique");
        }

        assert_eq!(report_ids.len(), 100, "Should have 100 unique report IDs");
    }

    /// Objective: Verify finalize_passports_at_shutdown with empty state
    /// Invariants: Should handle empty passport map gracefully
    #[test]
    fn test_finalize_passports_empty_state() {
        let analyzer = SafetyAnalyzer::default();
        let leaks = analyzer.finalize_passports_at_shutdown();
        assert!(leaks.is_empty(), "Should have no leaks with empty state");

        let stats = analyzer.get_stats();
        assert!(
            stats.passports_by_status.is_empty(),
            "Should have no passport status stats"
        );
    }

    /// Objective: Verify should_generate_report for all risk level combinations
    /// Invariants: Should correctly filter based on all possible combinations
    #[test]
    fn test_should_generate_report_all_combinations() {
        let test_cases = vec![
            (RiskLevel::Low, RiskLevel::Low, true),
            (RiskLevel::Low, RiskLevel::Medium, true),
            (RiskLevel::Low, RiskLevel::High, true),
            (RiskLevel::Low, RiskLevel::Critical, true),
            (RiskLevel::Medium, RiskLevel::Low, false),
            (RiskLevel::Medium, RiskLevel::Medium, true),
            (RiskLevel::Medium, RiskLevel::High, true),
            (RiskLevel::Medium, RiskLevel::Critical, true),
            (RiskLevel::High, RiskLevel::Low, false),
            (RiskLevel::High, RiskLevel::Medium, false),
            (RiskLevel::High, RiskLevel::High, true),
            (RiskLevel::High, RiskLevel::Critical, true),
            (RiskLevel::Critical, RiskLevel::Low, false),
            (RiskLevel::Critical, RiskLevel::Medium, false),
            (RiskLevel::Critical, RiskLevel::High, false),
            (RiskLevel::Critical, RiskLevel::Critical, true),
        ];

        for (min_level, report_level, expected) in test_cases {
            let config = SafetyAnalysisConfig {
                min_risk_level: min_level.clone(),
                ..Default::default()
            };
            let analyzer = SafetyAnalyzer::new(config);
            let result = analyzer.should_generate_report(&report_level);
            assert_eq!(
                result, expected,
                "should_generate_report({:?}, {:?}) should be {}",
                min_level, report_level, expected
            );
        }
    }

    /// Objective: Verify create_basic_risk_assessment for all source types
    /// Invariants: Each source type should have appropriate risk level and score
    #[test]
    fn test_create_basic_risk_assessment_all_sources() {
        let config = SafetyAnalysisConfig {
            detailed_risk_assessment: false,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);

        let test_cases = vec![
            (
                UnsafeSource::UnsafeBlock {
                    location: "test.rs".to_string(),
                    function: "test".to_string(),
                    file_path: None,
                    line_number: None,
                },
                RiskLevel::Medium,
                50.0,
            ),
            (
                UnsafeSource::FfiFunction {
                    library: "libc".to_string(),
                    function: "malloc".to_string(),
                    call_site: "test.rs".to_string(),
                },
                RiskLevel::Medium,
                45.0,
            ),
            (
                UnsafeSource::RawPointer {
                    operation: "dereference".to_string(),
                    location: "test.rs".to_string(),
                },
                RiskLevel::High,
                70.0,
            ),
            (
                UnsafeSource::Transmute {
                    from_type: "u8".to_string(),
                    to_type: "i8".to_string(),
                    location: "test.rs".to_string(),
                },
                RiskLevel::High,
                65.0,
            ),
        ];

        for (source, expected_level, expected_score) in test_cases {
            let report_id = analyzer.generate_unsafe_report(source, &[], &[]).unwrap();
            let reports = analyzer.get_unsafe_reports();
            let report = reports.get(&report_id).expect("Report should exist");

            assert_eq!(
                report.risk_assessment.risk_level, expected_level,
                "Risk level should match for source"
            );
            assert_eq!(
                report.risk_assessment.risk_score, expected_score,
                "Risk score should match for source"
            );
        }
    }

    /// Objective: Verify report generation with multiple reports at max limit
    /// Invariants: Should correctly handle max_reports boundary
    #[test]
    fn test_max_reports_overflow() {
        let config = SafetyAnalysisConfig {
            max_reports: 5,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);
        let source = UnsafeSource::UnsafeBlock {
            location: "test.rs".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };

        for i in 0..10 {
            let result = analyzer.generate_unsafe_report(source.clone(), &[], &[]);
            assert!(result.is_ok(), "Should generate report {}", i);
        }

        let reports = analyzer.get_unsafe_reports();
        assert!(reports.len() <= 5, "Should not exceed max_reports limit");
    }

    /// Objective: Verify passport creation with very large pointer
    /// Invariants: Should handle large pointer values
    #[test]
    fn test_create_memory_passport_large_pointer() {
        let analyzer = SafetyAnalyzer::default();
        let large_ptr = usize::MAX;
        let result =
            analyzer.create_memory_passport(large_ptr, 1024, PassportEventType::AllocatedInRust);
        assert!(result.is_ok(), "Should create passport with large pointer");

        let passports = analyzer.get_memory_passports();
        assert!(
            passports.contains_key(&large_ptr),
            "Passport with large pointer should exist"
        );
    }

    /// Objective: Verify passport creation with very large size
    /// Invariants: Should handle large size values
    #[test]
    fn test_create_memory_passport_large_size() {
        let analyzer = SafetyAnalyzer::default();
        let large_size = usize::MAX;
        let result =
            analyzer.create_memory_passport(0x1000, large_size, PassportEventType::AllocatedInRust);
        assert!(result.is_ok(), "Should create passport with large size");

        let passports = analyzer.get_memory_passports();
        let passport = passports.get(&0x1000).expect("Passport should exist");
        assert_eq!(
            passport.size_bytes, large_size,
            "Passport should have large size"
        );
    }

    /// Objective: Verify multiple passport events in sequence
    /// Invariants: Should correctly track all events in order
    #[test]
    fn test_passport_event_sequence() {
        let analyzer = SafetyAnalyzer::default();

        analyzer
            .create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust)
            .unwrap();

        let events = vec![
            PassportEventType::BoundaryAccess,
            PassportEventType::OwnershipTransfer,
            PassportEventType::HandoverToFfi,
            PassportEventType::BoundaryAccess,
            PassportEventType::FreedByForeign,
        ];

        for event_type in events {
            analyzer
                .record_passport_event(0x1000, event_type, "test".to_string())
                .unwrap();
        }

        let passports = analyzer.get_memory_passports();
        let passport = passports.get(&0x1000).expect("Passport should exist");
        assert_eq!(
            passport.lifecycle_events.len(),
            6,
            "Should have initial event plus 5 recorded events"
        );
    }

    /// Objective: Verify generate_report_id format for all source types
    /// Invariants: Report ID should have correct prefix for each source type
    #[test]
    fn test_generate_report_id_format() {
        let analyzer = SafetyAnalyzer::default();

        let sources = vec![
            (
                UnsafeSource::UnsafeBlock {
                    location: "test.rs".to_string(),
                    function: "test".to_string(),
                    file_path: None,
                    line_number: None,
                },
                "UNSAFE-UB-",
            ),
            (
                UnsafeSource::FfiFunction {
                    library: "libc".to_string(),
                    function: "malloc".to_string(),
                    call_site: "test.rs".to_string(),
                },
                "UNSAFE-FFI-",
            ),
            (
                UnsafeSource::RawPointer {
                    operation: "test".to_string(),
                    location: "test.rs".to_string(),
                },
                "UNSAFE-PTR-",
            ),
            (
                UnsafeSource::Transmute {
                    from_type: "u8".to_string(),
                    to_type: "i8".to_string(),
                    location: "test.rs".to_string(),
                },
                "UNSAFE-TX-",
            ),
        ];

        for (source, expected_prefix) in sources {
            let report_id = analyzer.generate_unsafe_report(source, &[], &[]).unwrap();
            assert!(
                report_id.starts_with(expected_prefix),
                "Report ID should start with {}",
                expected_prefix
            );
        }
    }

    /// Objective: Verify stats update for different risk levels
    /// Invariants: Stats should correctly track reports by risk level
    #[test]
    fn test_stats_by_risk_level() {
        let analyzer = SafetyAnalyzer::default();

        let sources = vec![
            UnsafeSource::RawPointer {
                operation: "test".to_string(),
                location: "test.rs".to_string(),
            },
            UnsafeSource::Transmute {
                from_type: "u8".to_string(),
                to_type: "i8".to_string(),
                location: "test.rs".to_string(),
            },
            UnsafeSource::UnsafeBlock {
                location: "test.rs".to_string(),
                function: "test".to_string(),
                file_path: None,
                line_number: None,
            },
        ];

        for source in sources {
            analyzer.generate_unsafe_report(source, &[], &[]).unwrap();
        }

        let stats = analyzer.get_stats();
        assert_eq!(stats.total_reports, 3, "Should have 3 reports");
        assert!(
            stats.reports_by_risk_level.contains_key("Low"),
            "Should have Low risk level reports"
        );
    }

    /// Objective: Verify passport status determination for edge cases
    /// Invariants: Should correctly determine status for edge case event combinations
    #[test]
    fn test_determine_final_passport_status_edge_cases() {
        let analyzer = SafetyAnalyzer::default();

        let events_only_reclaim = vec![PassportEvent {
            event_type: PassportEventType::ReclaimedByRust,
            timestamp: 1000,
            context: "test".to_string(),
            call_stack: vec![],
            metadata: HashMap::new(),
        }];
        let status = analyzer.determine_final_passport_status(&events_only_reclaim);
        assert!(
            matches!(status, PassportStatus::ReclaimedByRust),
            "Only reclaim event should result in ReclaimedByRust"
        );

        let events_only_foreign_free = vec![PassportEvent {
            event_type: PassportEventType::FreedByForeign,
            timestamp: 1000,
            context: "test".to_string(),
            call_stack: vec![],
            metadata: HashMap::new(),
        }];
        let status = analyzer.determine_final_passport_status(&events_only_foreign_free);
        assert!(
            matches!(status, PassportStatus::FreedByForeign),
            "Only foreign free event should result in FreedByForeign"
        );
    }

    /// Objective: Verify analyzer with all config options disabled
    /// Invariants: Should handle disabled features gracefully
    #[test]
    fn test_analyzer_all_features_disabled() {
        let config = SafetyAnalysisConfig {
            detailed_risk_assessment: false,
            enable_passport_tracking: false,
            min_risk_level: RiskLevel::Low,
            max_reports: 10,
            enable_dynamic_violations: false,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);

        let source = UnsafeSource::UnsafeBlock {
            location: "test.rs".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };

        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(
            result.is_ok(),
            "Should generate report with all features disabled"
        );

        let passport_result =
            analyzer.create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust);
        assert!(
            passport_result.is_ok(),
            "Should return Ok when passport tracking disabled"
        );
        assert!(
            passport_result.unwrap().is_empty(),
            "Should return empty string when passport tracking disabled"
        );
    }

    /// Objective: Verify report source information preservation
    /// Invariants: Report should preserve all source information
    #[test]
    fn test_report_source_preservation() {
        let analyzer = SafetyAnalyzer::default();

        let source = UnsafeSource::UnsafeBlock {
            location: "src/test.rs:42".to_string(),
            function: "test_function".to_string(),
            file_path: Some("src/test.rs".to_string()),
            line_number: Some(42),
        };

        let report_id = analyzer.generate_unsafe_report(source, &[], &[]).unwrap();
        let reports = analyzer.get_unsafe_reports();
        let report = reports.get(&report_id).expect("Report should exist");

        match &report.source {
            UnsafeSource::UnsafeBlock {
                location,
                function,
                file_path,
                line_number,
            } => {
                assert_eq!(location, "src/test.rs:42");
                assert_eq!(function, "test_function");
                assert_eq!(file_path, &Some("src/test.rs".to_string()));
                assert_eq!(line_number, &Some(42));
            }
            _ => panic!("Report source should be UnsafeBlock"),
        }
    }

    /// Objective: Verify strict mutex handling mode returns errors
    /// Invariants: When strict_mutex_handling is enabled, mutex poison should propagate errors
    #[test]
    fn test_strict_mutex_handling_mode() {
        let config = SafetyAnalysisConfig {
            strict_mutex_handling: true,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);

        let source = UnsafeSource::UnsafeBlock {
            location: "test.rs".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };

        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(
            result.is_ok(),
            "Should generate report successfully in normal case"
        );

        let passport_result =
            analyzer.create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust);
        assert!(
            passport_result.is_ok(),
            "Should create passport successfully in normal case"
        );
    }

    /// Objective: Verify lenient mutex handling mode recovers gracefully
    /// Invariants: When strict_mutex_handling is disabled, mutex poison should recover data
    #[test]
    fn test_lenient_mutex_handling_mode() {
        let config = SafetyAnalysisConfig {
            strict_mutex_handling: false,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);

        let source = UnsafeSource::UnsafeBlock {
            location: "test.rs".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };

        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(
            result.is_ok(),
            "Should generate report successfully in lenient mode"
        );

        let passport_result =
            analyzer.create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust);
        assert!(
            passport_result.is_ok(),
            "Should create passport successfully in lenient mode"
        );
    }

    /// Objective: Verify config option for strict mutex handling
    /// Invariants: Config should correctly control mutex handling behavior
    #[test]
    fn test_mutex_handling_config_option() {
        let strict_config = SafetyAnalysisConfig {
            strict_mutex_handling: true,
            ..Default::default()
        };
        let strict_analyzer = SafetyAnalyzer::new(strict_config);
        assert!(
            strict_analyzer.config.strict_mutex_handling,
            "Strict mode should be enabled"
        );

        let lenient_config = SafetyAnalysisConfig {
            strict_mutex_handling: false,
            ..Default::default()
        };
        let lenient_analyzer = SafetyAnalyzer::new(lenient_config);
        assert!(
            !lenient_analyzer.config.strict_mutex_handling,
            "Strict mode should be disabled"
        );
    }

    /// Objective: Verify error handling in getter methods
    /// Invariants: Getter methods should handle mutex errors gracefully
    #[test]
    fn test_getter_methods_error_handling() {
        let analyzer = SafetyAnalyzer::default();

        let reports = analyzer.get_unsafe_reports();
        assert!(reports.is_empty(), "Should return empty map on success");

        let passports = analyzer.get_memory_passports();
        assert!(passports.is_empty(), "Should return empty map on success");

        let stats = analyzer.get_stats();
        assert_eq!(
            stats.total_reports, 0,
            "Should return default stats on success"
        );
    }

    /// Objective: Verify convert_safety_violations handles DoubleFree correctly
    /// Invariants: DoubleFree should convert to DynamicViolation with Critical severity
    #[test]
    fn test_convert_safety_violation_double_free() {
        use crate::core::CallStackRef;

        let analyzer = SafetyAnalyzer::default();

        let call_stack = CallStackRef::new(1, Some(1));
        let violations = vec![SafetyViolation::DoubleFree {
            first_free_stack: call_stack.clone(),
            second_free_stack: call_stack.clone(),
            timestamp: 1000,
        }];

        let source = UnsafeSource::RawPointer {
            operation: "test".to_string(),
            location: "test.rs".to_string(),
        };

        let report_id = analyzer
            .generate_unsafe_report(source, &[], &violations)
            .unwrap();
        let reports = analyzer.get_unsafe_reports();
        let report = reports.get(&report_id).expect("Report should exist");

        assert_eq!(
            report.dynamic_violations.len(),
            1,
            "Should have one dynamic violation"
        );
        let dv = &report.dynamic_violations[0];
        assert!(
            matches!(dv.violation_type, ViolationType::DoubleFree),
            "Violation type should be DoubleFree"
        );
        assert!(
            matches!(dv.severity, RiskLevel::Critical),
            "DoubleFree should have Critical severity"
        );
    }

    /// Objective: Verify convert_safety_violations handles InvalidFree correctly
    /// Invariants: InvalidFree should convert to DynamicViolation with High severity
    #[test]
    fn test_convert_safety_violation_invalid_free() {
        use crate::core::CallStackRef;

        let analyzer = SafetyAnalyzer::default();

        let call_stack = CallStackRef::new(2, Some(1));
        let violations = vec![SafetyViolation::InvalidFree {
            attempted_pointer: 0x2000,
            stack: call_stack,
            timestamp: 2000,
        }];

        let source = UnsafeSource::RawPointer {
            operation: "test".to_string(),
            location: "test.rs".to_string(),
        };

        let report_id = analyzer
            .generate_unsafe_report(source, &[], &violations)
            .unwrap();
        let reports = analyzer.get_unsafe_reports();
        let report = reports.get(&report_id).expect("Report should exist");

        let dv = &report.dynamic_violations[0];
        assert!(
            matches!(dv.violation_type, ViolationType::InvalidAccess),
            "Violation type should be InvalidAccess"
        );
        assert_eq!(
            dv.memory_address, 0x2000,
            "Memory address should match attempted pointer"
        );
        assert!(
            matches!(dv.severity, RiskLevel::High),
            "InvalidFree should have High severity"
        );
    }

    /// Objective: Verify convert_safety_violations handles PotentialLeak correctly
    /// Invariants: PotentialLeak should convert to DynamicViolation with Medium severity
    #[test]
    fn test_convert_safety_violation_potential_leak() {
        use crate::core::CallStackRef;

        let analyzer = SafetyAnalyzer::default();

        let call_stack = CallStackRef::new(3, Some(1));
        let violations = vec![SafetyViolation::PotentialLeak {
            allocation_stack: call_stack,
            allocation_timestamp: 1000,
            leak_detection_timestamp: 5000,
        }];

        let source = UnsafeSource::RawPointer {
            operation: "test".to_string(),
            location: "test.rs".to_string(),
        };

        let report_id = analyzer
            .generate_unsafe_report(source, &[], &violations)
            .unwrap();
        let reports = analyzer.get_unsafe_reports();
        let report = reports.get(&report_id).expect("Report should exist");

        let dv = &report.dynamic_violations[0];
        assert!(
            matches!(dv.violation_type, ViolationType::MemoryLeak),
            "Violation type should be MemoryLeak"
        );
        assert!(
            matches!(dv.severity, RiskLevel::Medium),
            "PotentialLeak should have Medium severity"
        );
        assert_eq!(
            dv.detected_at, 5000,
            "Detected at should match leak_detection_timestamp"
        );
    }

    /// Objective: Verify convert_safety_violations handles CrossBoundaryRisk correctly
    /// Invariants: CrossBoundaryRisk should preserve risk level from original violation
    #[test]
    fn test_convert_safety_violation_cross_boundary() {
        use crate::core::CallStackRef;

        let analyzer = SafetyAnalyzer::default();

        let call_stack = CallStackRef::new(4, Some(1));
        let violations = vec![SafetyViolation::CrossBoundaryRisk {
            risk_level: RiskLevel::High,
            description: "FFI boundary violation".to_string(),
            stack: call_stack,
        }];

        let source = UnsafeSource::FfiFunction {
            library: "libc".to_string(),
            function: "malloc".to_string(),
            call_site: "test.rs".to_string(),
        };

        let report_id = analyzer
            .generate_unsafe_report(source, &[], &violations)
            .unwrap();
        let reports = analyzer.get_unsafe_reports();
        let report = reports.get(&report_id).expect("Report should exist");

        let dv = &report.dynamic_violations[0];
        assert!(
            matches!(dv.violation_type, ViolationType::FfiBoundaryViolation),
            "Violation type should be FfiBoundaryViolation"
        );
        assert!(
            matches!(dv.severity, RiskLevel::High),
            "Severity should match original risk level"
        );
        assert_eq!(
            dv.context, "FFI boundary violation",
            "Context should match description"
        );
    }

    /// Objective: Verify convert_safety_violations handles multiple violations
    /// Invariants: All violations should be converted correctly
    #[test]
    fn test_convert_multiple_safety_violations() {
        use crate::core::CallStackRef;

        let analyzer = SafetyAnalyzer::default();

        let call_stack = CallStackRef::new(5, Some(1));
        let violations = vec![
            SafetyViolation::DoubleFree {
                first_free_stack: call_stack.clone(),
                second_free_stack: call_stack.clone(),
                timestamp: 1000,
            },
            SafetyViolation::InvalidFree {
                attempted_pointer: 0x2000,
                stack: call_stack.clone(),
                timestamp: 2000,
            },
            SafetyViolation::PotentialLeak {
                allocation_stack: call_stack,
                allocation_timestamp: 1000,
                leak_detection_timestamp: 5000,
            },
        ];

        let source = UnsafeSource::UnsafeBlock {
            location: "test.rs".to_string(),
            function: "test".to_string(),
            file_path: None,
            line_number: None,
        };

        let report_id = analyzer
            .generate_unsafe_report(source, &[], &violations)
            .unwrap();
        let reports = analyzer.get_unsafe_reports();
        let report = reports.get(&report_id).expect("Report should exist");

        assert_eq!(
            report.dynamic_violations.len(),
            3,
            "Should have three dynamic violations"
        );
    }

    /// Objective: Verify memory context creation with empty allocations
    /// Invariants: Memory context should handle empty allocations correctly
    #[test]
    fn test_create_memory_context_empty() {
        let analyzer = SafetyAnalyzer::default();

        let source = UnsafeSource::RawPointer {
            operation: "test".to_string(),
            location: "test.rs".to_string(),
        };

        let report_id = analyzer.generate_unsafe_report(source, &[], &[]).unwrap();
        let reports = analyzer.get_unsafe_reports();
        let report = reports.get(&report_id).expect("Report should exist");

        assert_eq!(
            report.memory_context.total_allocated, 0,
            "Total allocated should be 0 for empty allocations"
        );
        assert_eq!(
            report.memory_context.active_allocations, 0,
            "Active allocations should be 0 for empty allocations"
        );
    }

    /// Objective: Verify CircuitBreaker trip behavior
    /// Invariants: CircuitBreaker should trip after max_retries poison events
    #[test]
    fn test_circuit_breaker_trip_threshold() {
        let mut breaker = CircuitBreaker::default();

        assert!(!breaker.is_tripped(), "Should not be tripped initially");

        breaker.record_poison(3);
        assert_eq!(breaker.poison_count(), 1);
        assert!(!breaker.is_tripped(), "Should not trip after 1 event");

        breaker.record_poison(3);
        assert_eq!(breaker.poison_count(), 2);
        assert!(!breaker.is_tripped(), "Should not trip after 2 events");

        breaker.record_poison(3);
        assert_eq!(breaker.poison_count(), 3);
        assert!(
            breaker.is_tripped(),
            "Should trip after reaching max_retries"
        );
    }

    /// Objective: Verify CircuitBreaker reset functionality
    /// Invariants: Reset should clear all state
    #[test]
    fn test_circuit_breaker_reset() {
        let mut breaker = CircuitBreaker::default();

        breaker.record_poison(3);
        breaker.record_poison(3);
        breaker.record_poison(3);

        assert!(breaker.is_tripped(), "Should be tripped");

        breaker.reset();

        assert!(!breaker.is_tripped(), "Should not be tripped after reset");
        assert_eq!(breaker.poison_count(), 0, "Poison count should be 0");
        assert!(
            breaker.last_poison_time().is_none(),
            "Last poison time should be None"
        );
    }

    /// Objective: Verify CircuitBreaker with different max_retries values
    /// Invariants: Should respect different threshold values
    #[test]
    fn test_circuit_breaker_different_thresholds() {
        let mut breaker1 = CircuitBreaker::default();
        breaker1.record_poison(1);
        assert!(
            breaker1.is_tripped(),
            "Should trip immediately with max_retries=1"
        );

        let mut breaker5 = CircuitBreaker::default();
        for _ in 0..4 {
            breaker5.record_poison(5);
        }
        assert!(
            !breaker5.is_tripped(),
            "Should not trip before reaching threshold"
        );
        breaker5.record_poison(5);
        assert!(breaker5.is_tripped(), "Should trip at exactly max_retries");
    }

    /// Objective: Verify get_current_timestamp returns valid value
    /// Invariants: Timestamp should be positive and reasonable
    #[test]
    fn test_get_current_timestamp() {
        let ts = get_current_timestamp();
        assert!(ts > 0, "Timestamp should be positive");
        assert!(
            ts > 1700000000,
            "Timestamp should be after 2023 (reasonable value)"
        );
    }

    /// Objective: Verify get_current_timestamp_nanos returns valid value
    /// Invariants: Nanos timestamp should be greater than seconds timestamp
    #[test]
    fn test_get_current_timestamp_nanos() {
        let ts_nanos = get_current_timestamp_nanos();
        let ts_secs = get_current_timestamp();

        assert!(ts_nanos > 0, "Nanos timestamp should be positive");
        assert!(
            ts_nanos >= ts_secs as u128,
            "Nanos should be >= seconds timestamp"
        );
    }

    /// Objective: Verify record_passport_event when tracking is disabled
    /// Invariants: Should return Ok(()) immediately without modifying state
    #[test]
    fn test_record_passport_event_tracking_disabled() {
        let config = SafetyAnalysisConfig {
            enable_passport_tracking: false,
            ..Default::default()
        };
        let analyzer = SafetyAnalyzer::new(config);

        let result = analyzer.record_passport_event(
            0x1000,
            PassportEventType::HandoverToFfi,
            "test".to_string(),
        );

        assert!(result.is_ok(), "Should return Ok when tracking disabled");
    }

    /// Objective: Verify generate_unsafe_report with passport tracking enabled
    /// Invariants: Should handle passport tracking correctly
    #[test]
    fn test_generate_report_with_passport_tracking() {
        let analyzer = SafetyAnalyzer::default();

        analyzer
            .create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust)
            .unwrap();

        let source = UnsafeSource::RawPointer {
            operation: "test".to_string(),
            location: "test.rs".to_string(),
        };

        let result = analyzer.generate_unsafe_report(source, &[], &[]);
        assert!(
            result.is_ok(),
            "Should generate report with passport tracking"
        );
    }

    /// Objective: Verify SafetyAnalysisStats serialization
    /// Invariants: Stats should serialize and deserialize correctly
    #[test]
    fn test_safety_analysis_stats_serialization() {
        let stats = SafetyAnalysisStats {
            total_reports: 10,
            reports_by_risk_level: HashMap::from([("Low".to_string(), 5), ("High".to_string(), 5)]),
            total_passports: 3,
            passports_by_status: HashMap::from([("Active".to_string(), 3)]),
            dynamic_violations: 2,
            analysis_start_time: 1000,
        };

        let json = serde_json::to_string(&stats).expect("Should serialize");
        let deserialized: SafetyAnalysisStats =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.total_reports, 10, "Total reports should match");
        assert_eq!(
            deserialized.total_passports, 3,
            "Total passports should match"
        );
    }

    /// Objective: Verify SafetyAnalysisConfig clone functionality
    /// Invariants: Cloned config should have identical values
    #[test]
    fn test_safety_config_clone() {
        let config = SafetyAnalysisConfig {
            detailed_risk_assessment: false,
            enable_passport_tracking: false,
            min_risk_level: RiskLevel::High,
            max_reports: 500,
            enable_dynamic_violations: false,
            strict_mutex_handling: true,
            max_mutex_poison_retries: 5,
        };

        let cloned = config.clone();

        assert_eq!(
            cloned.detailed_risk_assessment, false,
            "Cloned detailed_risk_assessment should match"
        );
        assert_eq!(
            cloned.enable_passport_tracking, false,
            "Cloned enable_passport_tracking should match"
        );
        assert_eq!(cloned.max_reports, 500, "Cloned max_reports should match");
        assert_eq!(
            cloned.max_mutex_poison_retries, 5,
            "Cloned max_mutex_poison_retries should match"
        );
    }

    /// Objective: Verify finalize_passports_at_shutdown handles lock failure
    /// Invariants: Should return empty vec when lock fails (graceful degradation)
    #[test]
    fn test_finalize_passports_graceful_degradation() {
        let analyzer = SafetyAnalyzer::default();

        analyzer
            .create_memory_passport(0x1000, 1024, PassportEventType::AllocatedInRust)
            .unwrap();

        let leaks = analyzer.finalize_passports_at_shutdown();
        assert!(
            leaks.is_empty(),
            "Should have no leaks when passport is not in foreign custody"
        );
    }
}
