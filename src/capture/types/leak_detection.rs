//! Leak detection types.
//!
//! This module contains types for detecting and analyzing memory leaks,
//! including evidence collection and prevention recommendations.

use serde::{Deserialize, Serialize};

use super::allocation::{ImpactLevel, Priority};

/// Resource leak analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceLeakAnalysis {
    /// Potential leaks detected.
    pub potential_leaks: Vec<EnhancedPotentialLeak>,
    /// Leak detection confidence.
    pub detection_confidence: f64,
    /// Resource usage patterns.
    pub usage_patterns: Vec<ResourceUsagePattern>,
    /// Leak prevention recommendations.
    pub prevention_recommendations: Vec<LeakPreventionRecommendation>,
}

/// Enhanced potential resource leak.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnhancedPotentialLeak {
    /// Object that may be leaking.
    pub object_id: usize,
    /// Leak type.
    pub leak_type: LeakType,
    /// Risk level.
    pub risk_level: LeakRiskLevel,
    /// Evidence for the leak.
    pub evidence: Vec<LeakEvidence>,
    /// Estimated impact.
    pub estimated_impact: LeakImpact,
}

/// Types of resource leaks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LeakType {
    /// Memory leak.
    Memory,
    /// File handle leak.
    FileHandle,
    /// Network connection leak.
    NetworkConnection,
    /// Thread leak.
    Thread,
    /// Lock leak (unreleased mutex).
    Lock,
    /// Reference cycle leak.
    ReferenceCycle,
}

/// Leak risk levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LeakRiskLevel {
    /// Low risk.
    Low,
    /// Medium risk.
    Medium,
    /// High risk.
    High,
    /// Critical risk.
    Critical,
}

/// Evidence for a potential leak.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeakEvidence {
    /// Evidence type.
    pub evidence_type: LeakEvidenceType,
    /// Evidence description.
    pub description: String,
    /// Evidence strength (0-100).
    pub strength: f64,
    /// Timestamp when evidence was collected.
    pub timestamp: u64,
}

/// Types of leak evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LeakEvidenceType {
    /// Object never dropped.
    NeverDropped,
    /// Circular reference detected.
    CircularReference,
    /// Resource handle not closed.
    ResourceNotClosed,
    /// Growing memory usage.
    GrowingMemoryUsage,
    /// Long-lived temporary object.
    LongLivedTemporary,
    /// Unreachable object still allocated.
    UnreachableObject,
}

/// Estimated leak impact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeakImpact {
    /// Memory impact in bytes.
    pub memory_bytes: usize,
    /// Performance impact percentage.
    pub performance_impact_percent: f64,
    /// Resource count impact.
    pub resource_count: u32,
    /// Time to critical impact.
    pub time_to_critical_hours: Option<f64>,
}

/// Resource usage pattern for leak detection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceUsagePattern {
    /// Pattern type.
    pub pattern_type: ResourcePatternType,
    /// Pattern description.
    pub description: String,
    /// Frequency of occurrence.
    pub frequency: f64,
    /// Associated leak risk.
    pub leak_risk: LeakRiskLevel,
}

/// Types of resource usage patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResourcePatternType {
    /// Monotonic growth.
    MonotonicGrowth,
    /// Periodic spikes.
    PeriodicSpikes,
    /// Gradual accumulation.
    GradualAccumulation,
    /// Sudden jumps.
    SuddenJumps,
    /// Irregular patterns.
    Irregular,
}

/// Leak prevention recommendation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeakPreventionRecommendation {
    /// Recommendation type.
    pub recommendation_type: LeakPreventionType,
    /// Priority level.
    pub priority: Priority,
    /// Description.
    pub description: String,
    /// Implementation guidance.
    pub implementation_guidance: String,
    /// Expected effectiveness.
    pub expected_effectiveness: f64,
}

/// Types of leak prevention recommendations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LeakPreventionType {
    /// Use RAII patterns.
    UseRAII,
    /// Implement proper Drop.
    ImplementDrop,
    /// Break circular references.
    BreakCircularReferences,
    /// Use weak references.
    UseWeakReferences,
    /// Implement resource pooling.
    ResourcePooling,
    /// Add resource monitoring.
    ResourceMonitoring,
    /// Use scoped guards.
    ScopedGuards,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_leak_analysis() {
        let analysis = ResourceLeakAnalysis {
            potential_leaks: vec![],
            detection_confidence: 0.95,
            usage_patterns: vec![],
            prevention_recommendations: vec![],
        };

        assert_eq!(analysis.detection_confidence, 0.95);
    }

    #[test]
    fn test_leak_type_variants() {
        let types = vec![
            LeakType::Memory,
            LeakType::FileHandle,
            LeakType::NetworkConnection,
            LeakType::Thread,
            LeakType::Lock,
            LeakType::ReferenceCycle,
        ];

        for leak_type in types {
            assert!(!format!("{leak_type:?}").is_empty());
        }
    }

    #[test]
    fn test_leak_risk_level() {
        assert!(matches!(LeakRiskLevel::Low, LeakRiskLevel::Low));
        assert!(matches!(LeakRiskLevel::Critical, LeakRiskLevel::Critical));
    }
}
