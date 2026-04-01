use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureInfo {
    pub ptr: usize,
    pub captures: Vec<CaptureInfo>,
    pub creation_timestamp: u64,
    pub thread_id: String,
    pub call_site: String,
    pub memory_footprint: ClosureFootprint,
    pub optimization_potential: OptimizationPotential,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureInfo {
    pub var_name: String,
    pub var_ptr: usize,
    pub mode: CaptureMode,
    pub var_type: String,
    pub size: usize,
    pub lifetime_bound: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CaptureMode {
    ByValue,
    ByReference,
    ByMutableReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureFootprint {
    pub total_size: usize,
    pub capture_count: usize,
    pub by_value_count: usize,
    pub by_ref_count: usize,
    pub by_mut_ref_count: usize,
    pub estimated_heap_usage: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationPotential {
    pub level: OptimizationLevel,
    pub potential_savings: usize,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationLevel {
    None,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureEvent {
    pub closure_ptr: usize,
    pub captured_var: CaptureInfo,
    pub event_type: CaptureEventType,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CaptureEventType {
    Captured,
    Released,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedClosure {
    pub ptr: usize,
    pub type_name: String,
    pub size: usize,
    pub estimated_captures: usize,
    pub closure_type: ClosureType,
    pub creation_context: CreationContext,
    pub memory_impact: MemoryImpact,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClosureType {
    Fn,
    FnMut,
    FnOnce,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationContext {
    pub scope_name: Option<String>,
    pub thread_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryImpact {
    Minimal,
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CaptureStatistics {
    pub total_closures: usize,
    pub total_captures: usize,
    pub avg_captures_per_closure: f64,
    pub total_memory_usage: usize,
    pub captures_by_mode: HashMap<CaptureMode, usize>,
    pub captures_by_type: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub category: OptimizationCategory,
    pub priority: SuggestionPriority,
    pub description: String,
    pub recommendation: String,
    pub estimated_impact: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationCategory {
    Memory,
    Performance,
    Lifetime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SuggestionPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug)]
pub struct LifetimeGraph {
    relationships: HashMap<usize, Vec<LifetimeRelationship>>,
}

impl Default for LifetimeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl LifetimeGraph {
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
        }
    }

    pub fn add_closure_relationships(&mut self, closure_ptr: usize, captures: &[CaptureInfo]) {
        let relationships: Vec<LifetimeRelationship> = captures
            .iter()
            .map(|capture| LifetimeRelationship {
                captured_var_ptr: capture.var_ptr,
                capture_mode: capture.mode.clone(),
                relationship_type: self.classify_relationship(&capture.mode),
            })
            .collect();

        self.relationships.insert(closure_ptr, relationships);
    }

    pub fn remove_closure(&mut self, closure_ptr: usize) {
        self.relationships.remove(&closure_ptr);
    }

    fn classify_relationship(&self, mode: &CaptureMode) -> RelationshipType {
        match mode {
            CaptureMode::ByValue => RelationshipType::Ownership,
            CaptureMode::ByReference => RelationshipType::SharedBorrow,
            CaptureMode::ByMutableReference => RelationshipType::ExclusiveBorrow,
        }
    }

    pub fn analyze_lifetimes(&self) -> LifetimeAnalysis {
        let mut potential_issues = Vec::new();
        let mut lifetime_patterns = Vec::new();

        for (closure_ptr, relationships) in &self.relationships {
            let has_value_captures = relationships
                .iter()
                .any(|r| r.capture_mode == CaptureMode::ByValue);
            let has_ref_captures = relationships.iter().any(|r| {
                matches!(
                    r.capture_mode,
                    CaptureMode::ByReference | CaptureMode::ByMutableReference
                )
            });

            if has_value_captures && has_ref_captures {
                potential_issues.push(LifetimeIssue {
                    closure_ptr: *closure_ptr,
                    issue_type: LifetimeIssueType::MixedCaptureMode,
                    description: "Closure mixes value and reference captures".to_string(),
                    severity: IssueSeverity::Medium,
                    suggestion: "Consider consistent capture strategy".to_string(),
                });
            }

            if relationships.len() > 5 {
                lifetime_patterns.push(LifetimePattern {
                    pattern_type: LifetimePatternType::ManyCaptures,
                    description: format!("Closure captures {} variables", relationships.len()),
                    impact: if relationships.len() > 10 {
                        PatternImpact::High
                    } else {
                        PatternImpact::Medium
                    },
                });
            }
        }

        LifetimeAnalysis {
            total_relationships: self.relationships.len(),
            potential_issues,
            lifetime_patterns,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LifetimeRelationship {
    pub captured_var_ptr: usize,
    pub capture_mode: CaptureMode,
    pub relationship_type: RelationshipType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipType {
    Ownership,
    SharedBorrow,
    ExclusiveBorrow,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LifetimeAnalysis {
    pub total_relationships: usize,
    pub potential_issues: Vec<LifetimeIssue>,
    pub lifetime_patterns: Vec<LifetimePattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeIssue {
    pub closure_ptr: usize,
    pub issue_type: LifetimeIssueType,
    pub description: String,
    pub severity: IssueSeverity,
    pub suggestion: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LifetimeIssueType {
    MixedCaptureMode,
    PotentialDanglingReference,
    UnnecessaryCapture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimePattern {
    pub pattern_type: LifetimePatternType,
    pub description: String,
    pub impact: PatternImpact,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LifetimePatternType {
    ManyCaptures,
    LongLivedClosure,
    FrequentCreation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternImpact {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureAnalysisReport {
    pub detected_closures: Vec<DetectedClosure>,
    pub capture_statistics: CaptureStatistics,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    pub lifetime_analysis: LifetimeAnalysis,
    pub analysis_timestamp: u64,
}
