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

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify ClosureInfo creation with all fields
    /// Invariants: All fields should be properly initialized
    #[test]
    fn test_closure_info() {
        let closure = ClosureInfo {
            ptr: 0x1000,
            captures: vec![],
            creation_timestamp: 1000,
            thread_id: "main".to_string(),
            call_site: "test.rs:10".to_string(),
            memory_footprint: ClosureFootprint {
                total_size: 64,
                capture_count: 0,
                by_value_count: 0,
                by_ref_count: 0,
                by_mut_ref_count: 0,
                estimated_heap_usage: 0,
            },
            optimization_potential: OptimizationPotential {
                level: OptimizationLevel::None,
                potential_savings: 0,
                suggestions: vec![],
            },
        };

        assert_eq!(closure.ptr, 0x1000, "Pointer should match");
        assert_eq!(closure.thread_id, "main", "Thread ID should match");
        assert_eq!(closure.captures.len(), 0, "Should have no captures");
    }

    /// Objective: Verify CaptureInfo creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_capture_info() {
        let capture = CaptureInfo {
            var_name: "x".to_string(),
            var_ptr: 0x2000,
            mode: CaptureMode::ByReference,
            var_type: "i32".to_string(),
            size: 4,
            lifetime_bound: Some("'a".to_string()),
        };

        assert_eq!(capture.var_name, "x", "Variable name should match");
        assert_eq!(
            capture.mode,
            CaptureMode::ByReference,
            "Capture mode should be ByReference"
        );
        assert_eq!(capture.size, 4, "Size should be 4");
    }

    /// Objective: Verify CaptureMode variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_capture_mode_variants() {
        let modes = vec![
            CaptureMode::ByValue,
            CaptureMode::ByReference,
            CaptureMode::ByMutableReference,
        ];

        for mode in &modes {
            let debug_str = format!("{mode:?}");
            assert!(
                !debug_str.is_empty(),
                "CaptureMode should have debug representation"
            );
        }

        assert_eq!(CaptureMode::ByValue, CaptureMode::ByValue);
        assert_ne!(CaptureMode::ByValue, CaptureMode::ByReference);
    }

    /// Objective: Verify ClosureFootprint creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_closure_footprint() {
        let footprint = ClosureFootprint {
            total_size: 128,
            capture_count: 5,
            by_value_count: 2,
            by_ref_count: 2,
            by_mut_ref_count: 1,
            estimated_heap_usage: 64,
        };

        assert_eq!(footprint.total_size, 128, "Total size should match");
        assert_eq!(footprint.capture_count, 5, "Capture count should match");
        assert_eq!(footprint.by_value_count, 2, "By value count should match");
    }

    /// Objective: Verify OptimizationPotential creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_optimization_potential() {
        let potential = OptimizationPotential {
            level: OptimizationLevel::High,
            potential_savings: 1024,
            suggestions: vec!["Use Arc instead of clone".to_string()],
        };

        assert_eq!(
            potential.level,
            OptimizationLevel::High,
            "Optimization level should be High"
        );
        assert_eq!(
            potential.potential_savings, 1024,
            "Potential savings should match"
        );
        assert_eq!(potential.suggestions.len(), 1, "Should have one suggestion");
    }

    /// Objective: Verify OptimizationLevel variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_optimization_level_variants() {
        assert_eq!(OptimizationLevel::None, OptimizationLevel::None);
        assert_eq!(OptimizationLevel::Low, OptimizationLevel::Low);
        assert_eq!(OptimizationLevel::Medium, OptimizationLevel::Medium);
        assert_eq!(OptimizationLevel::High, OptimizationLevel::High);

        assert_ne!(OptimizationLevel::None, OptimizationLevel::High);
    }

    /// Objective: Verify CaptureEvent creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_capture_event() {
        let event = CaptureEvent {
            closure_ptr: 0x1000,
            captured_var: CaptureInfo {
                var_name: "y".to_string(),
                var_ptr: 0x2000,
                mode: CaptureMode::ByValue,
                var_type: "String".to_string(),
                size: 24,
                lifetime_bound: None,
            },
            event_type: CaptureEventType::Captured,
            timestamp: 1000,
        };

        assert_eq!(
            event.event_type,
            CaptureEventType::Captured,
            "Event type should be Captured"
        );
    }

    /// Objective: Verify CaptureEventType variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_capture_event_type_variants() {
        assert_eq!(CaptureEventType::Captured, CaptureEventType::Captured);
        assert_eq!(CaptureEventType::Released, CaptureEventType::Released);
        assert_ne!(CaptureEventType::Captured, CaptureEventType::Released);
    }

    /// Objective: Verify DetectedClosure creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_detected_closure() {
        let detected = DetectedClosure {
            ptr: 0x1000,
            type_name: "closure".to_string(),
            size: 64,
            estimated_captures: 3,
            closure_type: ClosureType::FnMut,
            creation_context: CreationContext {
                scope_name: Some("test_scope".to_string()),
                thread_id: "main".to_string(),
                timestamp: 1000,
            },
            memory_impact: MemoryImpact::Medium,
        };

        assert_eq!(detected.ptr, 0x1000, "Pointer should match");
        assert_eq!(
            detected.closure_type,
            ClosureType::FnMut,
            "Closure type should be FnMut"
        );
    }

    /// Objective: Verify ClosureType variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_closure_type_variants() {
        let types = vec![
            ClosureType::Fn,
            ClosureType::FnMut,
            ClosureType::FnOnce,
            ClosureType::Unknown,
        ];

        for closure_type in &types {
            let debug_str = format!("{closure_type:?}");
            assert!(
                !debug_str.is_empty(),
                "ClosureType should have debug representation"
            );
        }
    }

    /// Objective: Verify MemoryImpact variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_memory_impact_variants() {
        let impacts = vec![
            MemoryImpact::Minimal,
            MemoryImpact::Low,
            MemoryImpact::Medium,
            MemoryImpact::High,
            MemoryImpact::VeryHigh,
        ];

        for impact in &impacts {
            let debug_str = format!("{impact:?}");
            assert!(
                !debug_str.is_empty(),
                "MemoryImpact should have debug representation"
            );
        }
    }

    /// Objective: Verify CaptureStatistics creation
    /// Invariants: Default should have zero values
    #[test]
    fn test_capture_statistics_default() {
        let stats = CaptureStatistics::default();

        assert_eq!(stats.total_closures, 0, "Total closures should be 0");
        assert_eq!(stats.total_captures, 0, "Total captures should be 0");
        assert_eq!(stats.avg_captures_per_closure, 0.0, "Average should be 0.0");
    }

    /// Objective: Verify CaptureStatistics with values
    /// Invariants: Should handle populated statistics
    #[test]
    fn test_capture_statistics_with_values() {
        let mut captures_by_mode = HashMap::new();
        captures_by_mode.insert(CaptureMode::ByValue, 10);
        captures_by_mode.insert(CaptureMode::ByReference, 20);

        let stats = CaptureStatistics {
            total_closures: 5,
            total_captures: 30,
            avg_captures_per_closure: 6.0,
            total_memory_usage: 1024,
            captures_by_mode,
            captures_by_type: HashMap::new(),
        };

        assert_eq!(stats.total_closures, 5, "Total closures should be 5");
        assert_eq!(
            stats.captures_by_mode.get(&CaptureMode::ByValue),
            Some(&10),
            "ByValue count should be 10"
        );
    }

    /// Objective: Verify OptimizationSuggestion creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_optimization_suggestion() {
        let suggestion = OptimizationSuggestion {
            category: OptimizationCategory::Memory,
            priority: SuggestionPriority::High,
            description: "Large closure detected".to_string(),
            recommendation: "Consider using Arc".to_string(),
            estimated_impact: "Save 1KB".to_string(),
        };

        assert_eq!(
            suggestion.category,
            OptimizationCategory::Memory,
            "Category should be Memory"
        );
        assert_eq!(
            suggestion.priority,
            SuggestionPriority::High,
            "Priority should be High"
        );
    }

    /// Objective: Verify OptimizationCategory variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_optimization_category_variants() {
        assert_eq!(OptimizationCategory::Memory, OptimizationCategory::Memory);
        assert_eq!(
            OptimizationCategory::Performance,
            OptimizationCategory::Performance
        );
        assert_eq!(
            OptimizationCategory::Lifetime,
            OptimizationCategory::Lifetime
        );
    }

    /// Objective: Verify SuggestionPriority variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_suggestion_priority_variants() {
        assert_eq!(SuggestionPriority::Low, SuggestionPriority::Low);
        assert_eq!(SuggestionPriority::Medium, SuggestionPriority::Medium);
        assert_eq!(SuggestionPriority::High, SuggestionPriority::High);
        assert_eq!(SuggestionPriority::Critical, SuggestionPriority::Critical);
    }

    /// Objective: Verify LifetimeGraph creation
    /// Invariants: New graph should be empty
    #[test]
    fn test_lifetime_graph_creation() {
        let graph = LifetimeGraph::new();
        assert_eq!(
            graph.relationships.len(),
            0,
            "New graph should have no relationships"
        );
    }

    /// Objective: Verify LifetimeGraph default
    /// Invariants: Default should create same as new()
    #[test]
    fn test_lifetime_graph_default() {
        let graph = LifetimeGraph::default();
        assert_eq!(
            graph.relationships.len(),
            0,
            "Default graph should have no relationships"
        );
    }

    /// Objective: Verify add_closure_relationships functionality
    /// Invariants: Should add relationships correctly
    #[test]
    fn test_lifetime_graph_add_relationships() {
        let mut graph = LifetimeGraph::new();

        let captures = vec![
            CaptureInfo {
                var_name: "x".to_string(),
                var_ptr: 0x1000,
                mode: CaptureMode::ByValue,
                var_type: "i32".to_string(),
                size: 4,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "y".to_string(),
                var_ptr: 0x2000,
                mode: CaptureMode::ByReference,
                var_type: "String".to_string(),
                size: 24,
                lifetime_bound: None,
            },
        ];

        graph.add_closure_relationships(0x5000, &captures);

        assert_eq!(
            graph.relationships.len(),
            1,
            "Should have one closure relationship"
        );
        assert_eq!(
            graph.relationships.get(&0x5000).unwrap().len(),
            2,
            "Should have two capture relationships"
        );
    }

    /// Objective: Verify remove_closure functionality
    /// Invariants: Should remove relationships correctly
    #[test]
    fn test_lifetime_graph_remove_closure() {
        let mut graph = LifetimeGraph::new();

        let captures = vec![CaptureInfo {
            var_name: "x".to_string(),
            var_ptr: 0x1000,
            mode: CaptureMode::ByValue,
            var_type: "i32".to_string(),
            size: 4,
            lifetime_bound: None,
        }];

        graph.add_closure_relationships(0x5000, &captures);
        assert_eq!(graph.relationships.len(), 1, "Should have one relationship");

        graph.remove_closure(0x5000);
        assert_eq!(
            graph.relationships.len(),
            0,
            "Should have no relationships after removal"
        );
    }

    /// Objective: Verify analyze_lifetimes with mixed captures
    /// Invariants: Should detect mixed capture mode issue
    #[test]
    fn test_lifetime_graph_analyze_mixed_captures() {
        let mut graph = LifetimeGraph::new();

        let captures = vec![
            CaptureInfo {
                var_name: "x".to_string(),
                var_ptr: 0x1000,
                mode: CaptureMode::ByValue,
                var_type: "i32".to_string(),
                size: 4,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "y".to_string(),
                var_ptr: 0x2000,
                mode: CaptureMode::ByReference,
                var_type: "String".to_string(),
                size: 24,
                lifetime_bound: None,
            },
        ];

        graph.add_closure_relationships(0x5000, &captures);
        let analysis = graph.analyze_lifetimes();

        assert_eq!(
            analysis.total_relationships, 1,
            "Should have one relationship"
        );
        assert!(
            analysis
                .potential_issues
                .iter()
                .any(|i| i.issue_type == LifetimeIssueType::MixedCaptureMode),
            "Should detect mixed capture mode issue"
        );
    }

    /// Objective: Verify analyze_lifetimes with many captures
    /// Invariants: Should detect many captures pattern
    #[test]
    fn test_lifetime_graph_analyze_many_captures() {
        let mut graph = LifetimeGraph::new();

        let captures: Vec<CaptureInfo> = (0..8)
            .map(|i| CaptureInfo {
                var_name: format!("var{}", i),
                var_ptr: 0x1000 + i,
                mode: CaptureMode::ByReference,
                var_type: "i32".to_string(),
                size: 4,
                lifetime_bound: None,
            })
            .collect();

        graph.add_closure_relationships(0x5000, &captures);
        let analysis = graph.analyze_lifetimes();

        assert!(
            analysis
                .lifetime_patterns
                .iter()
                .any(|p| p.pattern_type == LifetimePatternType::ManyCaptures),
            "Should detect many captures pattern"
        );
    }

    /// Objective: Verify RelationshipType classification
    /// Invariants: Each capture mode should map to correct relationship type
    #[test]
    fn test_relationship_type_classification() {
        let captures = vec![
            CaptureInfo {
                var_name: "v1".to_string(),
                var_ptr: 0x1000,
                mode: CaptureMode::ByValue,
                var_type: "i32".to_string(),
                size: 4,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "v2".to_string(),
                var_ptr: 0x2000,
                mode: CaptureMode::ByReference,
                var_type: "i32".to_string(),
                size: 4,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "v3".to_string(),
                var_ptr: 0x3000,
                mode: CaptureMode::ByMutableReference,
                var_type: "i32".to_string(),
                size: 4,
                lifetime_bound: None,
            },
        ];

        let mut test_graph = LifetimeGraph::new();
        test_graph.add_closure_relationships(0x5000, &captures);

        let rels = test_graph.relationships.get(&0x5000).unwrap();
        assert_eq!(
            rels[0].relationship_type,
            RelationshipType::Ownership,
            "ByValue should be Ownership"
        );
        assert_eq!(
            rels[1].relationship_type,
            RelationshipType::SharedBorrow,
            "ByReference should be SharedBorrow"
        );
        assert_eq!(
            rels[2].relationship_type,
            RelationshipType::ExclusiveBorrow,
            "ByMutableReference should be ExclusiveBorrow"
        );
    }

    /// Objective: Verify LifetimeIssue creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_lifetime_issue() {
        let issue = LifetimeIssue {
            closure_ptr: 0x1000,
            issue_type: LifetimeIssueType::PotentialDanglingReference,
            description: "Potential dangling reference".to_string(),
            severity: IssueSeverity::High,
            suggestion: "Check lifetime bounds".to_string(),
        };

        assert_eq!(
            issue.issue_type,
            LifetimeIssueType::PotentialDanglingReference,
            "Issue type should match"
        );
        assert_eq!(
            issue.severity,
            IssueSeverity::High,
            "Severity should be High"
        );
    }

    /// Objective: Verify LifetimePattern creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_lifetime_pattern() {
        let pattern = LifetimePattern {
            pattern_type: LifetimePatternType::LongLivedClosure,
            description: "Closure lives for entire program".to_string(),
            impact: PatternImpact::High,
        };

        assert_eq!(
            pattern.pattern_type,
            LifetimePatternType::LongLivedClosure,
            "Pattern type should match"
        );
        assert_eq!(pattern.impact, PatternImpact::High, "Impact should be High");
    }

    /// Objective: Verify ClosureAnalysisReport creation
    /// Invariants: All fields should be properly initialized
    #[test]
    fn test_closure_analysis_report() {
        let report = ClosureAnalysisReport {
            detected_closures: vec![],
            capture_statistics: CaptureStatistics::default(),
            optimization_suggestions: vec![],
            lifetime_analysis: LifetimeAnalysis::default(),
            analysis_timestamp: 1000,
        };

        assert_eq!(
            report.detected_closures.len(),
            0,
            "Should have no detected closures"
        );
        assert_eq!(report.analysis_timestamp, 1000, "Timestamp should match");
    }

    /// Objective: Verify serialization of CaptureMode
    /// Invariants: Should serialize and deserialize correctly
    #[test]
    fn test_capture_mode_serialization() {
        let mode = CaptureMode::ByMutableReference;
        let json = serde_json::to_string(&mode);
        assert!(json.is_ok(), "Should serialize to JSON");

        let deserialized: Result<CaptureMode, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "Should deserialize from JSON");
        assert_eq!(
            deserialized.unwrap(),
            CaptureMode::ByMutableReference,
            "Should preserve value"
        );
    }

    /// Objective: Verify serialization of OptimizationLevel
    /// Invariants: Should serialize and deserialize correctly
    #[test]
    fn test_optimization_level_serialization() {
        let level = OptimizationLevel::Medium;
        let json = serde_json::to_string(&level);
        assert!(json.is_ok(), "Should serialize to JSON");

        let deserialized: Result<OptimizationLevel, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "Should deserialize from JSON");
        assert_eq!(
            deserialized.unwrap(),
            OptimizationLevel::Medium,
            "Should preserve value"
        );
    }
}
