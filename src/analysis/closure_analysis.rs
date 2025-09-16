//! Closure capture analysis for memscope
//!
//! This module provides detailed analysis of closure captures, including:
//! - Variable capture modes (by value, by reference, by mutable reference)
//! - Lifetime tracking of captured variables
//! - Memory impact analysis of closures
//! - Optimization suggestions for closure usage

use crate::core::types::AllocationInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Global closure analyzer instance
static GLOBAL_CLOSURE_ANALYZER: OnceLock<Arc<ClosureAnalyzer>> = OnceLock::new();

/// Get the global closure analyzer instance
pub fn get_global_closure_analyzer() -> Arc<ClosureAnalyzer> {
    GLOBAL_CLOSURE_ANALYZER
        .get_or_init(|| Arc::new(ClosureAnalyzer::new()))
        .clone()
}

/// Advanced closure capture analysis
pub struct ClosureAnalyzer {
    /// Tracked closures and their captures
    closures: Mutex<HashMap<usize, ClosureInfo>>,
    /// Capture events history
    capture_events: Mutex<Vec<CaptureEvent>>,
    /// Lifetime relationships between closures and captured variables
    lifetime_graph: Mutex<LifetimeGraph>,
}

impl ClosureAnalyzer {
    /// Create a new closure analyzer
    pub fn new() -> Self {
        Self {
            closures: Mutex::new(HashMap::new()),
            capture_events: Mutex::new(Vec::new()),
            lifetime_graph: Mutex::new(LifetimeGraph::new()),
        }
    }

    /// Register a new closure with its captures
    pub fn register_closure(&self, closure_ptr: usize, captures: Vec<CaptureInfo>) {
        let closure_info = ClosureInfo {
            ptr: closure_ptr,
            captures: captures.clone(),
            creation_timestamp: current_timestamp(),
            thread_id: format!("{:?}", std::thread::current().id()),
            call_site: capture_call_site(),
            memory_footprint: self.calculate_closure_footprint(&captures),
            optimization_potential: self.analyze_optimization_potential(&captures),
        };

        // Record capture events
        for capture in &captures {
            let event = CaptureEvent {
                closure_ptr,
                captured_var: capture.clone(),
                event_type: CaptureEventType::Captured,
                timestamp: current_timestamp(),
            };

            if let Ok(mut events) = self.capture_events.lock() {
                events.push(event);
            }
        }

        // Update lifetime graph
        if let Ok(mut graph) = self.lifetime_graph.lock() {
            graph.add_closure_relationships(closure_ptr, &captures);
        }

        // Store closure info
        if let Ok(mut closures) = self.closures.lock() {
            closures.insert(closure_ptr, closure_info);
        }
    }

    /// Track when a closure is dropped
    pub fn track_closure_drop(&self, closure_ptr: usize) {
        if let Ok(mut closures) = self.closures.lock() {
            if let Some(closure_info) = closures.get_mut(&closure_ptr) {
                // Record drop events for all captures
                for capture in &closure_info.captures {
                    let event = CaptureEvent {
                        closure_ptr,
                        captured_var: capture.clone(),
                        event_type: CaptureEventType::Released,
                        timestamp: current_timestamp(),
                    };

                    if let Ok(mut events) = self.capture_events.lock() {
                        events.push(event);
                    }
                }
            }
            closures.remove(&closure_ptr);
        }

        // Update lifetime graph
        if let Ok(mut graph) = self.lifetime_graph.lock() {
            graph.remove_closure(closure_ptr);
        }
    }

    /// Analyze closure capture patterns in allocations
    pub fn analyze_closure_patterns(
        &self,
        allocations: &[AllocationInfo],
    ) -> ClosureAnalysisReport {
        let mut detected_closures = Vec::new();
        let mut capture_statistics = CaptureStatistics::default();

        for allocation in allocations {
            if let Some(type_name) = &allocation.type_name {
                if self.is_closure_type(type_name) {
                    if let Some(analysis) = self.analyze_closure_allocation(allocation) {
                        detected_closures.push(analysis);
                    }
                }
            }
        }

        // Calculate statistics
        if let Ok(closures) = self.closures.lock() {
            capture_statistics = self.calculate_capture_statistics(&closures);
        }

        let optimization_suggestions = self.generate_optimization_suggestions(&detected_closures);
        let lifetime_analysis = self.analyze_capture_lifetimes();

        ClosureAnalysisReport {
            detected_closures,
            capture_statistics,
            optimization_suggestions,
            lifetime_analysis,
            analysis_timestamp: current_timestamp(),
        }
    }

    /// Check if a type represents a closure
    fn is_closure_type(&self, type_name: &str) -> bool {
        // Rust closures typically have names like:
        // - "closure" (in debug builds)
        // - Complex mangled names containing "closure"
        // - Function pointer types
        type_name.contains("closure")
            || type_name.contains("{{closure}}")
            || type_name.starts_with("fn(")
            || type_name.contains("dyn Fn")
            || type_name.contains("impl Fn")
    }

    /// Analyze a specific closure allocation
    fn analyze_closure_allocation(&self, allocation: &AllocationInfo) -> Option<DetectedClosure> {
        let type_name = allocation.type_name.as_ref()?;

        Some(DetectedClosure {
            ptr: allocation.ptr,
            type_name: type_name.clone(),
            size: allocation.size,
            estimated_captures: self.estimate_captures_from_size(allocation.size),
            closure_type: self.classify_closure_type(type_name),
            creation_context: CreationContext {
                scope_name: allocation.scope_name.clone(),
                thread_id: allocation.thread_id.clone(),
                timestamp: allocation.timestamp_alloc,
            },
            memory_impact: self.assess_memory_impact(allocation.size),
        })
    }

    /// Estimate number of captures based on closure size
    fn estimate_captures_from_size(&self, size: usize) -> usize {
        // Rough estimation: each capture is typically 8-24 bytes
        // depending on the type (pointer, value, etc.)
        if size <= 8 {
            0 // Empty closure or single small capture
        } else if size <= 32 {
            2 // Small number of captures
        } else if size <= 128 {
            8 // Medium number of captures
        } else {
            size / 16 // Large number of captures (rough estimate)
        }
    }

    /// Classify the type of closure
    fn classify_closure_type(&self, type_name: &str) -> ClosureType {
        if type_name.contains("FnOnce") {
            ClosureType::FnOnce
        } else if type_name.contains("FnMut") {
            ClosureType::FnMut
        } else if type_name.contains("Fn") {
            ClosureType::Fn
        } else {
            ClosureType::Unknown
        }
    }

    /// Assess memory impact of a closure
    fn assess_memory_impact(&self, size: usize) -> MemoryImpact {
        match size {
            0..=16 => MemoryImpact::Minimal,
            17..=64 => MemoryImpact::Low,
            65..=256 => MemoryImpact::Medium,
            257..=1024 => MemoryImpact::High,
            _ => MemoryImpact::VeryHigh,
        }
    }

    /// Calculate closure memory footprint
    fn calculate_closure_footprint(&self, captures: &[CaptureInfo]) -> ClosureFootprint {
        let total_size = captures.iter().map(|c| c.size).sum();
        let by_value_count = captures
            .iter()
            .filter(|c| c.mode == CaptureMode::ByValue)
            .count();
        let by_ref_count = captures
            .iter()
            .filter(|c| c.mode == CaptureMode::ByReference)
            .count();
        let by_mut_ref_count = captures
            .iter()
            .filter(|c| c.mode == CaptureMode::ByMutableReference)
            .count();

        ClosureFootprint {
            total_size,
            capture_count: captures.len(),
            by_value_count,
            by_ref_count,
            by_mut_ref_count,
            estimated_heap_usage: self.estimate_heap_usage(captures),
        }
    }

    /// Estimate heap usage from captures
    fn estimate_heap_usage(&self, captures: &[CaptureInfo]) -> usize {
        captures
            .iter()
            .filter(|c| c.mode == CaptureMode::ByValue)
            .filter(|c| self.is_heap_allocated_type(&c.var_type))
            .map(|c| c.size)
            .sum()
    }

    /// Check if a type is typically heap-allocated
    fn is_heap_allocated_type(&self, type_name: &str) -> bool {
        type_name.contains("Vec")
            || type_name.contains("String")
            || type_name.contains("HashMap")
            || type_name.contains("Box")
            || type_name.contains("Arc")
            || type_name.contains("Rc")
    }

    /// Analyze optimization potential for captures
    fn analyze_optimization_potential(&self, captures: &[CaptureInfo]) -> OptimizationPotential {
        let mut suggestions = Vec::new();
        let mut potential_savings = 0;

        // Check for large by-value captures
        for capture in captures {
            if capture.mode == CaptureMode::ByValue && capture.size > 64 {
                suggestions.push(format!(
                    "Consider capturing '{}' by reference instead of by value to save {} bytes",
                    capture.var_name, capture.size
                ));
                potential_savings += capture.size;
            }
        }

        // Check for unnecessary mutable captures
        let mut_captures = captures
            .iter()
            .filter(|c| c.mode == CaptureMode::ByMutableReference)
            .count();
        if mut_captures > captures.len() / 2 {
            suggestions.push("Consider if all mutable captures are necessary".to_string());
        }

        // Check for potential move optimizations
        let heap_captures = captures
            .iter()
            .filter(|c| c.mode == CaptureMode::ByValue && self.is_heap_allocated_type(&c.var_type))
            .count();

        if heap_captures > 0 {
            suggestions
                .push("Consider using move semantics for heap-allocated captures".to_string());
        }

        OptimizationPotential {
            level: if potential_savings > 256 {
                OptimizationLevel::High
            } else if potential_savings > 64 {
                OptimizationLevel::Medium
            } else if !suggestions.is_empty() {
                OptimizationLevel::Low
            } else {
                OptimizationLevel::None
            },
            potential_savings,
            suggestions,
        }
    }

    /// Calculate capture statistics
    fn calculate_capture_statistics(
        &self,
        closures: &HashMap<usize, ClosureInfo>,
    ) -> CaptureStatistics {
        let total_closures = closures.len();
        let total_captures = closures.values().map(|c| c.captures.len()).sum();

        let mut by_mode = HashMap::new();
        let mut by_type = HashMap::new();
        let mut total_memory = 0;

        for closure in closures.values() {
            total_memory += closure.memory_footprint.total_size;

            for capture in &closure.captures {
                *by_mode.entry(capture.mode.clone()).or_insert(0) += 1;
                *by_type.entry(capture.var_type.clone()).or_insert(0) += 1;
            }
        }

        let avg_captures_per_closure = if total_closures > 0 {
            total_captures as f64 / total_closures as f64
        } else {
            0.0
        };

        CaptureStatistics {
            total_closures,
            total_captures,
            avg_captures_per_closure,
            total_memory_usage: total_memory,
            captures_by_mode: by_mode,
            captures_by_type: by_type,
        }
    }

    /// Generate optimization suggestions
    fn generate_optimization_suggestions(
        &self,
        closures: &[DetectedClosure],
    ) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // Analyze memory usage patterns
        let high_memory_closures = closures
            .iter()
            .filter(|c| matches!(c.memory_impact, MemoryImpact::High | MemoryImpact::VeryHigh))
            .count();

        if high_memory_closures > 0 {
            suggestions.push(OptimizationSuggestion {
                category: OptimizationCategory::Memory,
                priority: SuggestionPriority::High,
                description: format!(
                    "Found {high_memory_closures} closures with high memory usage",
                ),
                recommendation: "Consider reducing capture size or using references".to_string(),
                estimated_impact: "20-50% memory reduction".to_string(),
            });
        }

        // Analyze closure types
        let fnonce_count = closures
            .iter()
            .filter(|c| c.closure_type == ClosureType::FnOnce)
            .count();

        if fnonce_count > closures.len() / 2 {
            suggestions.push(OptimizationSuggestion {
                category: OptimizationCategory::Performance,
                priority: SuggestionPriority::Medium,
                description: "Many FnOnce closures detected".to_string(),
                recommendation: "Consider if Fn or FnMut traits would be more appropriate"
                    .to_string(),
                estimated_impact: "Improved reusability".to_string(),
            });
        }

        suggestions
    }

    /// Analyze capture lifetimes
    fn analyze_capture_lifetimes(&self) -> LifetimeAnalysis {
        if let Ok(graph) = self.lifetime_graph.lock() {
            graph.analyze_lifetimes()
        } else {
            LifetimeAnalysis::default()
        }
    }
}

/// Information about a closure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureInfo {
    /// Closure pointer
    pub ptr: usize,
    /// Captured variables
    pub captures: Vec<CaptureInfo>,
    /// Creation timestamp
    pub creation_timestamp: u64,
    /// Thread ID
    pub thread_id: String,
    /// Call site
    pub call_site: String,
    /// Closure memory footprint
    pub memory_footprint: ClosureFootprint,
    /// Optimization potential
    pub optimization_potential: OptimizationPotential,
}

/// Information about a captured variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureInfo {
    /// Variable name
    pub var_name: String,
    /// Variable pointer
    pub var_ptr: usize,
    /// Capture mode
    pub mode: CaptureMode,
    /// Variable type
    pub var_type: String,
    /// Variable size
    pub size: usize,
    /// Lifetime bound
    pub lifetime_bound: Option<String>,
}

/// Modes of variable capture
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CaptureMode {
    /// Capture by value    
    ByValue,
    /// Capture by reference
    ByReference,
    /// Capture by mutable reference
    ByMutableReference,
}

/// Closure memory footprint analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureFootprint {
    /// Total memory footprint
    pub total_size: usize,
    /// Number of captured variables
    pub capture_count: usize,
    /// Number of captured variables by value
    pub by_value_count: usize,
    /// Number of captured variables by reference
    pub by_ref_count: usize,
    /// Number of captured variables by mutable reference
    pub by_mut_ref_count: usize,
    /// Estimated heap usage
    pub estimated_heap_usage: usize,
}

/// Optimization potential analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationPotential {
    /// Optimization level
    pub level: OptimizationLevel,
    /// Potential savings in bytes
    pub potential_savings: usize,
    /// Optimization suggestions
    pub suggestions: Vec<String>,
}

/// Levels of optimization potential
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationLevel {
    /// No optimization potential
    None,
    /// Low optimization potential
    Low,
    /// Medium optimization potential
    Medium,
    /// High optimization potential
    High,
}

/// Capture event tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureEvent {
    /// Closure pointer
    pub closure_ptr: usize,
    /// Captured variable
    pub captured_var: CaptureInfo,
    /// Capture event type
    pub event_type: CaptureEventType,
    /// Timestamp
    pub timestamp: u64,
}

/// Types of capture    events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CaptureEventType {
    /// Capture event
    Captured,
    /// Release event
    Released,
}

/// Detected closure in allocation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedClosure {
    /// Closure pointer
    pub ptr: usize,
    /// Closure type name
    pub type_name: String,
    /// Closure size
    pub size: usize,
    /// Estimated number of captures
    pub estimated_captures: usize,
    /// Closure type
    pub closure_type: ClosureType,
    /// Closure creation context
    pub creation_context: CreationContext,
    /// Closure memory impact
    pub memory_impact: MemoryImpact,
}

/// Types of closures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClosureType {
    /// Fn closure
    Fn,
    /// FnMut closure
    FnMut,
    /// FnOnce closure
    FnOnce,
    /// Unknown closure type
    Unknown,
}

/// Creation context for closures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationContext {
    /// Scope name
    pub scope_name: Option<String>,
    /// Thread ID
    pub thread_id: String,
    /// Timestamp
    pub timestamp: u64,
}

/// Memory impact assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryImpact {
    /// Minimal memory impact
    Minimal,
    /// Low memory impact
    Low,
    /// Medium memory impact
    Medium,
    /// High memory impact
    High,
    /// Very high memory impact
    VeryHigh,
}

/// Capture statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CaptureStatistics {
    /// Total number of closures
    pub total_closures: usize,
    /// Total number of captures
    pub total_captures: usize,
    /// Average number of captures per closure
    pub avg_captures_per_closure: f64,
    /// Total memory usage
    pub total_memory_usage: usize,
    /// Captures by mode
    pub captures_by_mode: HashMap<CaptureMode, usize>,
    /// Captures by type
    pub captures_by_type: HashMap<String, usize>,
}

/// Optimization suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// Optimization category
    pub category: OptimizationCategory,
    /// Suggestion priority
    pub priority: SuggestionPriority,
    /// Suggestion description
    pub description: String,
    /// Suggestion recommendation
    pub recommendation: String,
    /// Estimated impact
    pub estimated_impact: String,
}

/// Categories of optimizations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationCategory {
    /// Memory optimization category
    Memory,
    /// Performance optimization category
    Performance,
    /// Lifetime optimization category
    Lifetime,
}

/// Priority levels for suggestions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SuggestionPriority {
    /// Low priority suggestion
    Low,
    /// Medium priority suggestion
    Medium,
    /// High priority suggestion
    High,
    /// Critical priority suggestion
    Critical,
}

/// Lifetime relationship graph
#[derive(Debug)]
pub struct LifetimeGraph {
    /// Relationships between closures and captured variables
    relationships: HashMap<usize, Vec<LifetimeRelationship>>,
}

impl Default for LifetimeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl LifetimeGraph {
    /// Create a new lifetime graph
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
        }
    }

    /// Add relationships for a closure
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

    /// Remove a closure and its relationships
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

    /// Analyze lifetime relationships
    pub fn analyze_lifetimes(&self) -> LifetimeAnalysis {
        let mut potential_issues = Vec::new();
        let mut lifetime_patterns = Vec::new();

        // Analyze for potential lifetime issues
        for (closure_ptr, relationships) in &self.relationships {
            // Check for mixed capture modes
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

            // Analyze patterns
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

/// Lifetime relationship between closure and captured variable
#[derive(Debug, Clone)]
pub struct LifetimeRelationship {
    /// Captured variable pointer
    pub captured_var_ptr: usize,
    /// Capture mode
    pub capture_mode: CaptureMode,
    /// Relationship type
    pub relationship_type: RelationshipType,
}

/// Types of lifetime relationships
#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipType {
    /// Ownership relationship
    Ownership,
    /// Shared borrow relationship
    SharedBorrow,
    /// Exclusive borrow relationship
    ExclusiveBorrow,
}

/// Lifetime analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LifetimeAnalysis {
    /// Total number of relationships
    pub total_relationships: usize,
    /// Potential issues
    pub potential_issues: Vec<LifetimeIssue>,
    /// Lifetime patterns
    pub lifetime_patterns: Vec<LifetimePattern>,
}

/// Lifetime-related issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeIssue {
    /// Closure pointer
    pub closure_ptr: usize,
    /// Issue type
    pub issue_type: LifetimeIssueType,
    /// Issue description
    pub description: String,
    /// Issue severity
    pub severity: IssueSeverity,
    /// Suggestion for resolution
    pub suggestion: String,
}

/// Types of lifetime issues
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LifetimeIssueType {
    /// Mixed capture mode
    MixedCaptureMode,
    /// Potential dangling reference
    PotentialDanglingReference,
    /// Unnecessary capture
    UnnecessaryCapture,
}

/// Issue severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Lifetime patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimePattern {
    /// Pattern type
    pub pattern_type: LifetimePatternType,
    /// Pattern description
    pub description: String,
    /// Pattern impact
    pub impact: PatternImpact,
}

/// Types of lifetime patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LifetimePatternType {
    /// Many captures
    ManyCaptures,
    /// Long-lived closure
    LongLivedClosure,
    /// Frequent creation
    FrequentCreation,
}

/// Impact of patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternImpact {
    /// Low impact
    Low,
    /// Medium impact
    Medium,
    /// High impact
    High,
}

/// Comprehensive closure analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureAnalysisReport {
    /// Detected closures
    pub detected_closures: Vec<DetectedClosure>,
    /// Capture statistics
    pub capture_statistics: CaptureStatistics,
    /// Optimization suggestions
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    /// Lifetime analysis
    pub lifetime_analysis: LifetimeAnalysis,
    /// Analysis timestamp
    pub analysis_timestamp: u64,
}

// Utility functions

/// Get current timestamp in nanoseconds
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Capture call site information (simplified)
fn capture_call_site() -> String {
    // In a real implementation, this would capture more detailed call site info
    "<call_site_placeholder>".to_string()
}

impl Default for ClosureAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AllocationInfo;

    #[test]
    fn test_closure_analyzer_creation() {
        let analyzer = ClosureAnalyzer::new();

        // Test that analyzer is properly initialized
        assert!(analyzer.closures.lock().unwrap().is_empty());
        assert!(analyzer.capture_events.lock().unwrap().is_empty());
    }

    #[test]
    fn test_closure_analyzer_singleton_behavior() {
        // Test that we can create multiple analyzers without issues
        let analyzer1 = ClosureAnalyzer::new();
        let analyzer2 = ClosureAnalyzer::new();

        // Each should be independent instances
        assert!(analyzer1.closures.lock().unwrap().is_empty());
        assert!(analyzer2.closures.lock().unwrap().is_empty());
    }

    #[test]
    fn test_register_closure() {
        let analyzer = ClosureAnalyzer::new();
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
                var_type: "&str".to_string(),
                size: 8,
                lifetime_bound: Some("'a".to_string()),
            },
        ];

        analyzer.register_closure(0x5000, captures.clone());

        // Verify closure was registered
        let closures = analyzer.closures.lock().unwrap();
        assert!(closures.contains_key(&0x5000));

        let closure_info = &closures[&0x5000];
        assert_eq!(closure_info.ptr, 0x5000);
        assert_eq!(closure_info.captures.len(), 2);
        assert_eq!(closure_info.memory_footprint.capture_count, 2);
        assert_eq!(closure_info.memory_footprint.by_value_count, 1);
        assert_eq!(closure_info.memory_footprint.by_ref_count, 1);

        // Verify capture events were recorded
        let events = analyzer.capture_events.lock().unwrap();
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|e| e.closure_ptr == 0x5000));
        assert!(events
            .iter()
            .all(|e| e.event_type == CaptureEventType::Captured));
    }

    #[test]
    fn test_track_closure_drop() {
        let analyzer = ClosureAnalyzer::new();
        let captures = vec![CaptureInfo {
            var_name: "data".to_string(),
            var_ptr: 0x3000,
            mode: CaptureMode::ByValue,
            var_type: "Vec<i32>".to_string(),
            size: 24,
            lifetime_bound: None,
        }];

        analyzer.register_closure(0x6000, captures);

        // Verify closure exists
        assert!(analyzer.closures.lock().unwrap().contains_key(&0x6000));

        // Track drop
        analyzer.track_closure_drop(0x6000);

        // Verify closure was removed
        assert!(!analyzer.closures.lock().unwrap().contains_key(&0x6000));

        // Verify release events were recorded
        let events = analyzer.capture_events.lock().unwrap();
        let release_events: Vec<_> = events
            .iter()
            .filter(|e| e.event_type == CaptureEventType::Released)
            .collect();
        assert_eq!(release_events.len(), 1);
        assert_eq!(release_events[0].closure_ptr, 0x6000);
    }

    #[test]
    fn test_is_closure_type() {
        let analyzer = ClosureAnalyzer::new();

        // Test various closure type names
        assert!(analyzer.is_closure_type("closure"));
        assert!(analyzer.is_closure_type("{{closure}}"));
        assert!(analyzer.is_closure_type("fn()"));
        assert!(analyzer.is_closure_type("fn(i32) -> bool"));
        assert!(analyzer.is_closure_type("dyn Fn()"));
        assert!(analyzer.is_closure_type("impl Fn(i32)"));
        assert!(analyzer.is_closure_type("some_module::{{closure}}"));

        // Test non-closure types
        assert!(!analyzer.is_closure_type("Vec<i32>"));
        assert!(!analyzer.is_closure_type("String"));
        assert!(!analyzer.is_closure_type("HashMap<K, V>"));
    }

    #[test]
    fn test_estimate_captures_from_size() {
        let analyzer = ClosureAnalyzer::new();

        assert_eq!(analyzer.estimate_captures_from_size(0), 0);
        assert_eq!(analyzer.estimate_captures_from_size(8), 0);
        assert_eq!(analyzer.estimate_captures_from_size(16), 2);
        assert_eq!(analyzer.estimate_captures_from_size(32), 2);
        assert_eq!(analyzer.estimate_captures_from_size(64), 8);
        assert_eq!(analyzer.estimate_captures_from_size(128), 8);
        assert_eq!(analyzer.estimate_captures_from_size(256), 16);
    }

    #[test]
    fn test_classify_closure_type() {
        let analyzer = ClosureAnalyzer::new();

        assert_eq!(
            analyzer.classify_closure_type("dyn FnOnce()"),
            ClosureType::FnOnce
        );
        assert_eq!(
            analyzer.classify_closure_type("impl FnMut(i32)"),
            ClosureType::FnMut
        );
        assert_eq!(
            analyzer.classify_closure_type("dyn Fn() -> bool"),
            ClosureType::Fn
        );
        assert_eq!(
            analyzer.classify_closure_type("{{closure}}"),
            ClosureType::Unknown
        );
        assert_eq!(
            analyzer.classify_closure_type("some_closure"),
            ClosureType::Unknown
        );
    }

    #[test]
    fn test_assess_memory_impact() {
        let analyzer = ClosureAnalyzer::new();

        assert_eq!(analyzer.assess_memory_impact(0), MemoryImpact::Minimal);
        assert_eq!(analyzer.assess_memory_impact(16), MemoryImpact::Minimal);
        assert_eq!(analyzer.assess_memory_impact(32), MemoryImpact::Low);
        assert_eq!(analyzer.assess_memory_impact(64), MemoryImpact::Low);
        assert_eq!(analyzer.assess_memory_impact(128), MemoryImpact::Medium);
        assert_eq!(analyzer.assess_memory_impact(256), MemoryImpact::Medium);
        assert_eq!(analyzer.assess_memory_impact(512), MemoryImpact::High);
        assert_eq!(analyzer.assess_memory_impact(1024), MemoryImpact::High);
        assert_eq!(analyzer.assess_memory_impact(2048), MemoryImpact::VeryHigh);
    }

    #[test]
    fn test_is_heap_allocated_type() {
        let analyzer = ClosureAnalyzer::new();

        // Test heap-allocated types
        assert!(analyzer.is_heap_allocated_type("Vec<i32>"));
        assert!(analyzer.is_heap_allocated_type("String"));
        assert!(analyzer.is_heap_allocated_type("HashMap<K, V>"));
        assert!(analyzer.is_heap_allocated_type("Box<dyn Trait>"));
        assert!(analyzer.is_heap_allocated_type("Arc<Mutex<T>>"));
        assert!(analyzer.is_heap_allocated_type("Rc<RefCell<T>>"));

        // Test stack-allocated types
        assert!(!analyzer.is_heap_allocated_type("i32"));
        assert!(!analyzer.is_heap_allocated_type("&str"));
        assert!(!analyzer.is_heap_allocated_type("bool"));
        assert!(!analyzer.is_heap_allocated_type("[i32; 10]"));
    }

    #[test]
    fn test_calculate_closure_footprint() {
        let analyzer = ClosureAnalyzer::new();
        let captures = vec![
            CaptureInfo {
                var_name: "a".to_string(),
                var_ptr: 0x1000,
                mode: CaptureMode::ByValue,
                var_type: "i32".to_string(),
                size: 4,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "b".to_string(),
                var_ptr: 0x2000,
                mode: CaptureMode::ByReference,
                var_type: "&str".to_string(),
                size: 8,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "c".to_string(),
                var_ptr: 0x3000,
                mode: CaptureMode::ByMutableReference,
                var_type: "&mut Vec<i32>".to_string(),
                size: 8,
                lifetime_bound: None,
            },
        ];

        let footprint = analyzer.calculate_closure_footprint(&captures);

        assert_eq!(footprint.total_size, 20);
        assert_eq!(footprint.capture_count, 3);
        assert_eq!(footprint.by_value_count, 1);
        assert_eq!(footprint.by_ref_count, 1);
        assert_eq!(footprint.by_mut_ref_count, 1);
        assert_eq!(footprint.estimated_heap_usage, 0); // No heap-allocated types by value
    }

    #[test]
    fn test_analyze_optimization_potential() {
        let analyzer = ClosureAnalyzer::new();

        // Test with large by-value capture
        let large_captures = vec![CaptureInfo {
            var_name: "large_data".to_string(),
            var_ptr: 0x1000,
            mode: CaptureMode::ByValue,
            var_type: "[u8; 1024]".to_string(),
            size: 1024,
            lifetime_bound: None,
        }];

        let potential = analyzer.analyze_optimization_potential(&large_captures);
        assert_eq!(potential.level, OptimizationLevel::High);
        assert_eq!(potential.potential_savings, 1024);
        assert!(!potential.suggestions.is_empty());

        // Test with small captures
        let small_captures = vec![CaptureInfo {
            var_name: "x".to_string(),
            var_ptr: 0x1000,
            mode: CaptureMode::ByValue,
            var_type: "i32".to_string(),
            size: 4,
            lifetime_bound: None,
        }];

        let potential = analyzer.analyze_optimization_potential(&small_captures);
        assert_eq!(potential.level, OptimizationLevel::None);
        assert_eq!(potential.potential_savings, 0);
    }

    #[test]
    fn test_analyze_closure_patterns() {
        let analyzer = ClosureAnalyzer::new();

        // Create test allocations with closure types
        let mut alloc1 = AllocationInfo::new(0x1000, 32);
        alloc1.type_name = Some("{{closure}}".to_string());

        let mut alloc2 = AllocationInfo::new(0x2000, 64);
        alloc2.type_name = Some("dyn FnOnce()".to_string());

        let mut alloc3 = AllocationInfo::new(0x3000, 16);
        alloc3.type_name = Some("Vec<i32>".to_string()); // Not a closure

        let allocations = vec![alloc1, alloc2, alloc3];

        let report = analyzer.analyze_closure_patterns(&allocations);

        // Should detect 2 closures
        assert_eq!(report.detected_closures.len(), 2);

        // Check first detected closure
        let first_closure = &report.detected_closures[0];
        assert_eq!(first_closure.ptr, 0x1000);
        assert_eq!(first_closure.size, 32);
        assert_eq!(first_closure.estimated_captures, 2); // 32 bytes -> 2 captures
        assert_eq!(first_closure.closure_type, ClosureType::Unknown);
        assert_eq!(first_closure.memory_impact, MemoryImpact::Low);

        // Check second detected closure
        let second_closure = &report.detected_closures[1];
        assert_eq!(second_closure.ptr, 0x2000);
        assert_eq!(second_closure.size, 64);
        assert_eq!(second_closure.estimated_captures, 8); // 64 bytes -> 8 captures
        assert_eq!(second_closure.closure_type, ClosureType::FnOnce);
        assert_eq!(second_closure.memory_impact, MemoryImpact::Low);

        // Verify timestamp
        assert!(report.analysis_timestamp > 0);
    }

    #[test]
    fn test_lifetime_graph() {
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
                var_type: "&str".to_string(),
                size: 8,
                lifetime_bound: None,
            },
        ];

        graph.add_closure_relationships(0x5000, &captures);

        // Verify relationships were added
        assert!(graph.relationships.contains_key(&0x5000));
        let relationships = &graph.relationships[&0x5000];
        assert_eq!(relationships.len(), 2);

        // Check relationship types
        assert_eq!(
            relationships[0].relationship_type,
            RelationshipType::Ownership
        );
        assert_eq!(
            relationships[1].relationship_type,
            RelationshipType::SharedBorrow
        );

        // Test removal
        graph.remove_closure(0x5000);
        assert!(!graph.relationships.contains_key(&0x5000));
    }

    #[test]
    fn test_lifetime_analysis() {
        let mut graph = LifetimeGraph::new();

        // Add closure with mixed capture modes (should trigger issue)
        let mixed_captures = vec![
            CaptureInfo {
                var_name: "owned".to_string(),
                var_ptr: 0x1000,
                mode: CaptureMode::ByValue,
                var_type: "String".to_string(),
                size: 24,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "borrowed".to_string(),
                var_ptr: 0x2000,
                mode: CaptureMode::ByReference,
                var_type: "&i32".to_string(),
                size: 8,
                lifetime_bound: None,
            },
        ];

        graph.add_closure_relationships(0x5000, &mixed_captures);

        let analysis = graph.analyze_lifetimes();

        assert_eq!(analysis.total_relationships, 1);
        assert_eq!(analysis.potential_issues.len(), 1);

        let issue = &analysis.potential_issues[0];
        assert_eq!(issue.closure_ptr, 0x5000);
        assert_eq!(issue.issue_type, LifetimeIssueType::MixedCaptureMode);
        assert_eq!(issue.severity, IssueSeverity::Medium);
    }

    #[test]
    fn test_lifetime_analysis_many_captures() {
        let mut graph = LifetimeGraph::new();

        // Create closure with many captures
        let many_captures: Vec<CaptureInfo> = (0..8)
            .map(|i| CaptureInfo {
                var_name: format!("var_{i}"),
                var_ptr: 0x1000 + i * 8,
                mode: CaptureMode::ByValue,
                var_type: "i32".to_string(),
                size: 4,
                lifetime_bound: None,
            })
            .collect();

        graph.add_closure_relationships(0x6000, &many_captures);

        let analysis = graph.analyze_lifetimes();

        assert_eq!(analysis.total_relationships, 1);
        assert_eq!(analysis.lifetime_patterns.len(), 1);

        let pattern = &analysis.lifetime_patterns[0];
        assert_eq!(pattern.pattern_type, LifetimePatternType::ManyCaptures);
        assert_eq!(pattern.impact, PatternImpact::Medium);
    }

    #[test]
    fn test_capture_mode_variants() {
        let modes = vec![
            CaptureMode::ByValue,
            CaptureMode::ByReference,
            CaptureMode::ByMutableReference,
        ];

        for mode in modes {
            assert!(!format!("{mode:?}").is_empty());
        }
    }

    #[test]
    fn test_optimization_level_variants() {
        let levels = vec![
            OptimizationLevel::None,
            OptimizationLevel::Low,
            OptimizationLevel::Medium,
            OptimizationLevel::High,
        ];

        for level in levels {
            assert!(!format!("{level:?}").is_empty());
        }
    }

    #[test]
    fn test_closure_type_variants() {
        let types = vec![
            ClosureType::Fn,
            ClosureType::FnMut,
            ClosureType::FnOnce,
            ClosureType::Unknown,
        ];

        for closure_type in types {
            assert!(!format!("{closure_type:?}").is_empty());
        }
    }

    #[test]
    fn test_memory_impact_variants() {
        let impacts = vec![
            MemoryImpact::Minimal,
            MemoryImpact::Low,
            MemoryImpact::Medium,
            MemoryImpact::High,
            MemoryImpact::VeryHigh,
        ];

        for impact in impacts {
            assert!(!format!("{impact:?}").is_empty());
        }
    }

    #[test]
    fn test_capture_statistics_default() {
        let stats = CaptureStatistics::default();

        assert_eq!(stats.total_closures, 0);
        assert_eq!(stats.total_captures, 0);
        assert_eq!(stats.avg_captures_per_closure, 0.0);
        assert_eq!(stats.total_memory_usage, 0);
        assert!(stats.captures_by_mode.is_empty());
        assert!(stats.captures_by_type.is_empty());
    }

    #[test]
    fn test_optimization_category_variants() {
        let categories = vec![
            OptimizationCategory::Memory,
            OptimizationCategory::Performance,
            OptimizationCategory::Lifetime,
        ];

        for category in categories {
            assert!(!format!("{category:?}").is_empty());
        }
    }

    #[test]
    fn test_suggestion_priority_variants() {
        let priorities = vec![
            SuggestionPriority::Low,
            SuggestionPriority::Medium,
            SuggestionPriority::High,
            SuggestionPriority::Critical,
        ];

        for priority in priorities {
            assert!(!format!("{priority:?}").is_empty());
        }
    }

    #[test]
    fn test_relationship_type_variants() {
        let types = vec![
            RelationshipType::Ownership,
            RelationshipType::SharedBorrow,
            RelationshipType::ExclusiveBorrow,
        ];

        for rel_type in types {
            assert!(!format!("{rel_type:?}").is_empty());
        }
    }

    #[test]
    fn test_lifetime_analysis_default() {
        let analysis = LifetimeAnalysis::default();

        assert_eq!(analysis.total_relationships, 0);
        assert!(analysis.potential_issues.is_empty());
        assert!(analysis.lifetime_patterns.is_empty());
    }

    #[test]
    fn test_lifetime_issue_type_variants() {
        let types = vec![
            LifetimeIssueType::MixedCaptureMode,
            LifetimeIssueType::PotentialDanglingReference,
            LifetimeIssueType::UnnecessaryCapture,
        ];

        for issue_type in types {
            assert!(!format!("{issue_type:?}").is_empty());
        }
    }

    #[test]
    fn test_issue_severity_variants() {
        let severities = vec![
            IssueSeverity::Low,
            IssueSeverity::Medium,
            IssueSeverity::High,
            IssueSeverity::Critical,
        ];

        for severity in severities {
            assert!(!format!("{severity:?}").is_empty());
        }
    }

    #[test]
    fn test_lifetime_pattern_type_variants() {
        let types = vec![
            LifetimePatternType::ManyCaptures,
            LifetimePatternType::LongLivedClosure,
            LifetimePatternType::FrequentCreation,
        ];

        for pattern_type in types {
            assert!(!format!("{pattern_type:?}").is_empty());
        }
    }

    #[test]
    fn test_pattern_impact_variants() {
        let impacts = vec![
            PatternImpact::Low,
            PatternImpact::Medium,
            PatternImpact::High,
        ];

        for impact in impacts {
            assert!(!format!("{impact:?}").is_empty());
        }
    }

    #[test]
    fn test_capture_event_type_variants() {
        let types = vec![CaptureEventType::Captured, CaptureEventType::Released];

        for event_type in types {
            assert!(!format!("{event_type:?}").is_empty());
        }
    }

    #[test]
    fn test_current_timestamp() {
        let timestamp1 = current_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let timestamp2 = current_timestamp();

        assert!(timestamp2 > timestamp1);
        assert!(timestamp1 > 0);
    }

    #[test]
    fn test_capture_call_site() {
        let call_site = capture_call_site();
        assert!(!call_site.is_empty());
        assert_eq!(call_site, "<call_site_placeholder>");
    }

    #[test]
    fn test_complex_closure_analysis_scenario() {
        let analyzer = ClosureAnalyzer::new();

        // Register multiple closures with different characteristics
        let small_captures = vec![CaptureInfo {
            var_name: "x".to_string(),
            var_ptr: 0x1000,
            mode: CaptureMode::ByValue,
            var_type: "i32".to_string(),
            size: 4,
            lifetime_bound: None,
        }];

        let large_captures = vec![CaptureInfo {
            var_name: "data".to_string(),
            var_ptr: 0x2000,
            mode: CaptureMode::ByValue,
            var_type: "Vec<u8>".to_string(),
            size: 1024,
            lifetime_bound: None,
        }];

        analyzer.register_closure(0x5000, small_captures);
        analyzer.register_closure(0x6000, large_captures);

        // Create allocations for analysis
        let mut alloc1 = AllocationInfo::new(0x5000, 8);
        alloc1.type_name = Some("{{closure}}".to_string());

        let mut alloc2 = AllocationInfo::new(0x6000, 1024);
        alloc2.type_name = Some("dyn FnOnce()".to_string());

        let allocations = vec![alloc1, alloc2];

        let report = analyzer.analyze_closure_patterns(&allocations);

        // Verify comprehensive analysis
        assert_eq!(report.detected_closures.len(), 2);
        assert!(report.capture_statistics.total_closures > 0);
        assert!(!report.optimization_suggestions.is_empty());
        assert!(report.analysis_timestamp > 0);

        // Check that high memory usage triggered optimization suggestions
        let memory_suggestions: Vec<_> = report
            .optimization_suggestions
            .iter()
            .filter(|s| s.category == OptimizationCategory::Memory)
            .collect();
        assert!(!memory_suggestions.is_empty());
    }

    #[test]
    fn test_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let analyzer = Arc::new(ClosureAnalyzer::new());
        let mut handles = vec![];

        // Test concurrent access
        for i in 0..4 {
            let analyzer_clone = analyzer.clone();
            let handle = thread::spawn(move || {
                let captures = vec![CaptureInfo {
                    var_name: format!("var_{i}"),
                    var_ptr: 0x1000 + i * 8,
                    mode: CaptureMode::ByValue,
                    var_type: "i32".to_string(),
                    size: 4,
                    lifetime_bound: None,
                }];

                analyzer_clone.register_closure(0x5000 + i, captures);
                analyzer_clone.track_closure_drop(0x5000 + i);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // All closures should be dropped
        assert!(analyzer.closures.lock().unwrap().is_empty());

        // Should have capture and release events
        let events = analyzer.capture_events.lock().unwrap();
        assert_eq!(events.len(), 8); // 4 captures + 4 releases
    }
}
