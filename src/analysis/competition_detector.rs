// Competition detector for identifying variable races and resource conflicts
// Analyzes variable access patterns to detect potential race conditions

use crate::core::types::{TrackingResult, TrackingError};
use crate::export::fixed_hybrid_template::HybridAnalysisData;
use std::collections::HashMap;
use tracing::{info, debug, trace};

/// Competition detector for identifying variable race conditions
pub struct CompetitionDetector {
    detection_algorithms: Vec<Box<dyn CompetitionDetectionAlgorithm>>,
    risk_calculator: RiskCalculator,
    threshold_adjuster: ThresholdAdjuster,
}

impl CompetitionDetector {
    /// Create new competition detector with default algorithms
    pub fn new() -> Self {
        info!("Initializing competition detector with default algorithms");
        
        let algorithms: Vec<Box<dyn CompetitionDetectionAlgorithm>> = vec![
            Box::new(DataRaceDetector::new()),
            Box::new(ResourceContentionDetector::new()),
            Box::new(MemoryAliasDetector::new()),
            Box::new(LockFreeViolationDetector::new()),
        ];
        
        Self {
            detection_algorithms: algorithms,
            risk_calculator: RiskCalculator::new(),
            threshold_adjuster: ThresholdAdjuster::new(),
        }
    }
    
    /// Detect competitions with specified confidence threshold
    pub fn detect_competitions_with_threshold(
        &mut self,
        data: &HashMap<u32, HybridAnalysisData>,
        threshold: f64,
    ) -> TrackingResult<Vec<CompetitionReport>> {
        info!("Starting competition detection with threshold {:.2}", threshold);
        
        let mut all_competitions = Vec::new();
        
        // Run each detection algorithm
        for algorithm in &mut self.detection_algorithms {
            let competitions = algorithm.detect_competitions(data)?;
            debug!("Algorithm '{}' detected {} competitions", 
                   algorithm.name(), competitions.len());
            all_competitions.extend(competitions);
        }
        
        // Filter by confidence threshold
        let filtered_competitions: Vec<CompetitionReport> = all_competitions
            .into_iter()
            .filter(|comp| comp.confidence_score >= threshold)
            .collect();
        
        // Calculate risk scores
        let mut final_competitions = Vec::new();
        for mut competition in filtered_competitions {
            competition.risk_assessment = self.risk_calculator.calculate_risk(&competition)?;
            final_competitions.push(competition);
        }
        
        // Sort by risk level (highest first)
        final_competitions.sort_by(|a, b| {
            b.risk_assessment.overall_risk_score
                .partial_cmp(&a.risk_assessment.overall_risk_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        info!("Competition detection complete: {} high-confidence competitions found", 
              final_competitions.len());
        
        Ok(final_competitions)
    }
}

/// Risk assessment calculator
struct RiskCalculator {
    impact_weights: ImpactWeights,
}

impl RiskCalculator {
    fn new() -> Self {
        Self {
            impact_weights: ImpactWeights::default(),
        }
    }
    
    fn calculate_risk(&self, competition: &CompetitionReport) -> TrackingResult<RiskAssessment> {
        // Calculate individual risk factors
        let frequency_risk = self.calculate_frequency_risk(competition);
        let memory_risk = self.calculate_memory_risk(competition);
        let concurrency_risk = self.calculate_concurrency_risk(competition);
        let timing_risk = self.calculate_timing_risk(competition);
        
        // Weighted overall risk score
        let overall_risk_score = 
            (frequency_risk * self.impact_weights.frequency_weight) +
            (memory_risk * self.impact_weights.memory_weight) +
            (concurrency_risk * self.impact_weights.concurrency_weight) +
            (timing_risk * self.impact_weights.timing_weight);
        
        let risk_level = if overall_risk_score > 0.8 {
            RiskLevel::Critical
        } else if overall_risk_score > 0.6 {
            RiskLevel::High
        } else if overall_risk_score > 0.4 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };
        
        Ok(RiskAssessment {
            overall_risk_score,
            risk_level,
            frequency_risk_factor: frequency_risk,
            memory_risk_factor: memory_risk,
            concurrency_risk_factor: concurrency_risk,
            timing_risk_factor: timing_risk,
            mitigation_strategies: self.generate_mitigation_strategies(overall_risk_score),
        })
    }
    
    fn calculate_frequency_risk(&self, competition: &CompetitionReport) -> f64 {
        // Higher access frequency = higher risk
        let max_frequency = competition.involved_variables.iter()
            .map(|v| v.access_frequency)
            .max()
            .unwrap_or(0);
        
        (max_frequency as f64 / 1000.0).min(1.0) // Normalize to 0-1
    }
    
    fn calculate_memory_risk(&self, competition: &CompetitionReport) -> f64 {
        // Larger memory regions = higher risk
        let total_memory = competition.involved_variables.iter()
            .map(|v| v.memory_size)
            .sum::<u64>();
        
        (total_memory as f64 / (10.0 * 1024.0 * 1024.0)).min(1.0) // Normalize to 10MB max
    }
    
    fn calculate_concurrency_risk(&self, competition: &CompetitionReport) -> f64 {
        // More threads/processes = higher risk
        let thread_count = competition.involved_variables.iter()
            .map(|v| v.thread_id)
            .collect::<std::collections::HashSet<_>>()
            .len();
        
        (thread_count as f64 / 10.0).min(1.0) // Normalize to 10 threads max
    }
    
    fn calculate_timing_risk(&self, competition: &CompetitionReport) -> f64 {
        // Closer timing = higher risk
        match &competition.timing_information {
            Some(timing) => {
                if timing.overlapping_windows > 0 {
                    (timing.overlapping_windows as f64 / 100.0).min(1.0)
                } else {
                    0.0
                }
            }
            None => 0.5, // Default moderate risk when timing unknown
        }
    }
    
    fn generate_mitigation_strategies(&self, risk_score: f64) -> Vec<String> {
        let mut strategies = Vec::new();
        
        if risk_score > 0.8 {
            strategies.push("Critical: Implement immediate synchronization".to_string());
            strategies.push("Use atomic operations or mutex locks".to_string());
            strategies.push("Consider lock-free data structures".to_string());
        } else if risk_score > 0.6 {
            strategies.push("High: Review concurrent access patterns".to_string());
            strategies.push("Consider using Arc<RwLock<T>> for shared data".to_string());
        } else if risk_score > 0.4 {
            strategies.push("Medium: Monitor access patterns".to_string());
            strategies.push("Consider thread-local storage".to_string());
        } else {
            strategies.push("Low: Continue monitoring".to_string());
        }
        
        strategies
    }
}

/// Threshold adjustment for different scenarios
struct ThresholdAdjuster {
    base_threshold: f64,
    scenario_adjustments: HashMap<String, f64>,
}

impl ThresholdAdjuster {
    fn new() -> Self {
        let mut adjustments = HashMap::new();
        adjustments.insert("high_frequency".to_string(), -0.1); // Lower threshold for high freq
        adjustments.insert("large_memory".to_string(), -0.15); // Lower threshold for large memory
        adjustments.insert("many_threads".to_string(), -0.2); // Lower threshold for many threads
        
        Self {
            base_threshold: 0.7,
            scenario_adjustments: adjustments,
        }
    }
}

// Impact weights for risk calculation
#[derive(Debug)]
struct ImpactWeights {
    frequency_weight: f64,
    memory_weight: f64,
    concurrency_weight: f64,
    timing_weight: f64,
}

impl Default for ImpactWeights {
    fn default() -> Self {
        Self {
            frequency_weight: 0.3,
            memory_weight: 0.2,
            concurrency_weight: 0.3,
            timing_weight: 0.2,
        }
    }
}

// Main data structures

/// Competition report containing detected race condition information
#[derive(Debug)]
pub struct CompetitionReport {
    pub competition_id: String,
    pub competition_type: CompetitionType,
    pub involved_variables: Vec<CompetingVariable>,
    pub confidence_score: f64,
    pub risk_assessment: RiskAssessment,
    pub timing_information: Option<TimingAnalysis>,
    pub suggested_fixes: Vec<String>,
    pub detection_algorithm: String,
}

/// Types of detected competitions
#[derive(Debug)]
pub enum CompetitionType {
    DataRace,
    ResourceContention,
    MemoryAliasing,
    LockFreeViolation,
    CacheCoherence,
}

/// Risk assessment for a competition
#[derive(Debug)]
pub struct RiskAssessment {
    pub overall_risk_score: f64,
    pub risk_level: RiskLevel,
    pub frequency_risk_factor: f64,
    pub memory_risk_factor: f64,
    pub concurrency_risk_factor: f64,
    pub timing_risk_factor: f64,
    pub mitigation_strategies: Vec<String>,
}

/// Risk level categories
#[derive(Debug)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Variable involved in competition
#[derive(Debug)]
pub struct CompetingVariable {
    pub variable_name: String,
    pub variable_type: String,
    pub process_id: u32,
    pub thread_id: usize,
    pub memory_size: u64,
    pub access_frequency: u64,
    pub access_pattern: AccessPattern,
}

/// Access patterns for variables
#[derive(Debug)]
pub enum AccessPattern {
    ReadOnly,
    WriteOnly,
    ReadWrite,
    WriteHeavy,
    ReadHeavy,
}

/// Timing analysis for overlapping access
#[derive(Debug)]
pub struct TimingAnalysis {
    pub overlapping_windows: u32,
    pub average_overlap_duration_ms: f64,
    pub peak_concurrency_level: u32,
    pub timing_correlation_score: f64,
}

// Competition detection algorithms trait

/// Trait for competition detection algorithms
pub trait CompetitionDetectionAlgorithm {
    fn name(&self) -> &str;
    fn detect_competitions(
        &mut self,
        data: &HashMap<u32, HybridAnalysisData>,
    ) -> TrackingResult<Vec<CompetitionReport>>;
}

/// Data race detector implementation
struct DataRaceDetector {
    name: String,
}

impl DataRaceDetector {
    fn new() -> Self {
        Self {
            name: "DataRaceDetector".to_string(),
        }
    }
}

impl CompetitionDetectionAlgorithm for DataRaceDetector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn detect_competitions(
        &mut self,
        data: &HashMap<u32, HybridAnalysisData>,
    ) -> TrackingResult<Vec<CompetitionReport>> {
        debug!("Running data race detection");
        
        let mut competitions = Vec::new();
        
        // Simple data race detection based on shared variables without synchronization
        for (process_id, process_data) in data {
            for variable in process_data.variable_registry.values() {
                if variable.allocation_count > 10 && variable.thread_id != 0 {
                    // High-frequency access in non-main thread = potential race
                    let competition = CompetitionReport {
                        competition_id: format!("DR_{}_{}_{}", process_id, variable.thread_id, variable.name),
                        competition_type: CompetitionType::DataRace,
                        involved_variables: vec![CompetingVariable {
                            variable_name: variable.name.clone(),
                            variable_type: variable.type_info.clone(),
                            process_id: *process_id,
                            thread_id: variable.thread_id,
                            memory_size: variable.memory_usage,
                            access_frequency: variable.allocation_count,
                            access_pattern: AccessPattern::ReadWrite, // Simplified
                        }],
                        confidence_score: 0.7, // Moderate confidence
                        risk_assessment: RiskAssessment {
                            overall_risk_score: 0.0, // Will be calculated later
                            risk_level: RiskLevel::Medium,
                            frequency_risk_factor: 0.0,
                            memory_risk_factor: 0.0,
                            concurrency_risk_factor: 0.0,
                            timing_risk_factor: 0.0,
                            mitigation_strategies: vec![],
                        },
                        timing_information: None,
                        suggested_fixes: vec![
                            format!("Consider using Arc<Mutex<{}>> for thread-safe access", variable.type_info),
                            "Review concurrent access patterns".to_string(),
                        ],
                        detection_algorithm: self.name.clone(),
                    };
                    
                    competitions.push(competition);
                }
            }
        }
        
        trace!("Data race detector found {} potential races", competitions.len());
        Ok(competitions)
    }
}

/// Resource contention detector implementation
struct ResourceContentionDetector {
    name: String,
}

impl ResourceContentionDetector {
    fn new() -> Self {
        Self {
            name: "ResourceContentionDetector".to_string(),
        }
    }
}

impl CompetitionDetectionAlgorithm for ResourceContentionDetector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn detect_competitions(
        &mut self,
        data: &HashMap<u32, HybridAnalysisData>,
    ) -> TrackingResult<Vec<CompetitionReport>> {
        debug!("Running resource contention detection");
        
        let mut competitions = Vec::new();
        let mut shared_resources: HashMap<String, Vec<(u32, &crate::export::fixed_hybrid_template::VariableInfo)>> = HashMap::new();
        
        // Group variables by type and name to find shared resources
        for (process_id, process_data) in data {
            for variable in process_data.variable_registry.values() {
                let resource_key = format!("{}:{}", variable.type_info, variable.name);
                shared_resources.entry(resource_key).or_default().push((*process_id, variable));
            }
        }
        
        // Find resources accessed by multiple processes
        for (resource_key, accesses) in shared_resources {
            if accesses.len() > 1 {
                let total_frequency: u64 = accesses.iter().map(|(_, v)| v.allocation_count).sum();
                
                if total_frequency > 20 { // Contention threshold
                    let involved_variables = accesses.into_iter().map(|(process_id, var)| {
                        CompetingVariable {
                            variable_name: var.name.clone(),
                            variable_type: var.type_info.clone(),
                            process_id,
                            thread_id: var.thread_id,
                            memory_size: var.memory_usage,
                            access_frequency: var.allocation_count,
                            access_pattern: AccessPattern::ReadWrite,
                        }
                    }).collect();
                    
                    let competition = CompetitionReport {
                        competition_id: format!("RC_{}", resource_key),
                        competition_type: CompetitionType::ResourceContention,
                        involved_variables,
                        confidence_score: 0.8, // High confidence for multi-process access
                        risk_assessment: RiskAssessment {
                            overall_risk_score: 0.0,
                            risk_level: RiskLevel::High,
                            frequency_risk_factor: 0.0,
                            memory_risk_factor: 0.0,
                            concurrency_risk_factor: 0.0,
                            timing_risk_factor: 0.0,
                            mitigation_strategies: vec![],
                        },
                        timing_information: Some(TimingAnalysis {
                            overlapping_windows: (total_frequency / 10) as u32,
                            average_overlap_duration_ms: 5.0,
                            peak_concurrency_level: accesses.len() as u32,
                            timing_correlation_score: 0.7,
                        }),
                        suggested_fixes: vec![
                            "Implement process-level synchronization".to_string(),
                            "Consider using shared memory with proper locking".to_string(),
                            "Evaluate need for resource pooling".to_string(),
                        ],
                        detection_algorithm: self.name.clone(),
                    };
                    
                    competitions.push(competition);
                }
            }
        }
        
        trace!("Resource contention detector found {} contentions", competitions.len());
        Ok(competitions)
    }
}

/// Memory alias detector implementation
struct MemoryAliasDetector {
    name: String,
}

impl MemoryAliasDetector {
    fn new() -> Self {
        Self {
            name: "MemoryAliasDetector".to_string(),
        }
    }
}

impl CompetitionDetectionAlgorithm for MemoryAliasDetector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn detect_competitions(
        &mut self,
        data: &HashMap<u32, HybridAnalysisData>,
    ) -> TrackingResult<Vec<CompetitionReport>> {
        debug!("Running memory alias detection");
        
        let mut competitions = Vec::new();
        
        // Detect potential memory aliasing issues
        for (process_id, process_data) in data {
            let variables: Vec<_> = process_data.variable_registry.values().collect();
            
            for (i, var1) in variables.iter().enumerate() {
                for var2 in variables.iter().skip(i + 1) {
                    // Check for potential aliasing (similar size, same thread, high frequency)
                    if var1.thread_id == var2.thread_id &&
                       var1.memory_usage == var2.memory_usage &&
                       var1.allocation_count > 5 &&
                       var2.allocation_count > 5 &&
                       var1.type_info == var2.type_info {
                        
                        let competition = CompetitionReport {
                            competition_id: format!("MA_{}_{}_{}_{}", process_id, var1.thread_id, var1.name, var2.name),
                            competition_type: CompetitionType::MemoryAliasing,
                            involved_variables: vec![
                                CompetingVariable {
                                    variable_name: var1.name.clone(),
                                    variable_type: var1.type_info.clone(),
                                    process_id: *process_id,
                                    thread_id: var1.thread_id,
                                    memory_size: var1.memory_usage,
                                    access_frequency: var1.allocation_count,
                                    access_pattern: AccessPattern::ReadWrite,
                                },
                                CompetingVariable {
                                    variable_name: var2.name.clone(),
                                    variable_type: var2.type_info.clone(),
                                    process_id: *process_id,
                                    thread_id: var2.thread_id,
                                    memory_size: var2.memory_usage,
                                    access_frequency: var2.allocation_count,
                                    access_pattern: AccessPattern::ReadWrite,
                                },
                            ],
                            confidence_score: 0.6, // Moderate confidence
                            risk_assessment: RiskAssessment {
                                overall_risk_score: 0.0,
                                risk_level: RiskLevel::Medium,
                                frequency_risk_factor: 0.0,
                                memory_risk_factor: 0.0,
                                concurrency_risk_factor: 0.0,
                                timing_risk_factor: 0.0,
                                mitigation_strategies: vec![],
                            },
                            timing_information: None,
                            suggested_fixes: vec![
                                "Review variable lifetimes for potential aliasing".to_string(),
                                "Consider using different variable names or scopes".to_string(),
                                "Verify memory layout and alignment".to_string(),
                            ],
                            detection_algorithm: self.name.clone(),
                        };
                        
                        competitions.push(competition);
                    }
                }
            }
        }
        
        trace!("Memory alias detector found {} potential aliases", competitions.len());
        Ok(competitions)
    }
}

/// Lock-free violation detector implementation
struct LockFreeViolationDetector {
    name: String,
}

impl LockFreeViolationDetector {
    fn new() -> Self {
        Self {
            name: "LockFreeViolationDetector".to_string(),
        }
    }
}

impl CompetitionDetectionAlgorithm for LockFreeViolationDetector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn detect_competitions(
        &mut self,
        data: &HashMap<u32, HybridAnalysisData>,
    ) -> TrackingResult<Vec<CompetitionReport>> {
        debug!("Running lock-free violation detection");
        
        let mut competitions = Vec::new();
        
        // Detect violations of lock-free programming patterns
        for (process_id, process_data) in data {
            for variable in process_data.variable_registry.values() {
                // Look for patterns that suggest lock-free violations
                let is_shared = matches!(variable.lifecycle_stage, crate::export::fixed_hybrid_template::LifecycleStage::Shared);
                let high_frequency = variable.allocation_count > 15;
                let multi_thread_env = process_data.thread_task_mapping.len() > 1;
                
                if is_shared && high_frequency && multi_thread_env {
                    let competition = CompetitionReport {
                        competition_id: format!("LFV_{}_{}_{}", process_id, variable.thread_id, variable.name),
                        competition_type: CompetitionType::LockFreeViolation,
                        involved_variables: vec![CompetingVariable {
                            variable_name: variable.name.clone(),
                            variable_type: variable.type_info.clone(),
                            process_id: *process_id,
                            thread_id: variable.thread_id,
                            memory_size: variable.memory_usage,
                            access_frequency: variable.allocation_count,
                            access_pattern: AccessPattern::WriteHeavy,
                        }],
                        confidence_score: 0.75,
                        risk_assessment: RiskAssessment {
                            overall_risk_score: 0.0,
                            risk_level: RiskLevel::High,
                            frequency_risk_factor: 0.0,
                            memory_risk_factor: 0.0,
                            concurrency_risk_factor: 0.0,
                            timing_risk_factor: 0.0,
                            mitigation_strategies: vec![],
                        },
                        timing_information: None,
                        suggested_fixes: vec![
                            "Consider using atomic operations for lock-free access".to_string(),
                            "Implement compare-and-swap (CAS) operations".to_string(),
                            "Review memory ordering requirements".to_string(),
                            "Consider using lock-free data structures from crossbeam".to_string(),
                        ],
                        detection_algorithm: self.name.clone(),
                    };
                    
                    competitions.push(competition);
                }
            }
        }
        
        trace!("Lock-free violation detector found {} violations", competitions.len());
        Ok(competitions)
    }
}

/// Competition risk enumeration for public API
#[derive(Debug)]
pub enum CompetitionRisk {
    DataRace(CompetitionReport),
    ResourceContention(CompetitionReport),
    MemoryAliasing(CompetitionReport),
    LockFreeViolation(CompetitionReport),
}