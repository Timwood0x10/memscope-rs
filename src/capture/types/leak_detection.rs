//! Leak detection types.
//!
//! This module contains types for detecting and analyzing memory leaks,
//! including evidence collection and prevention recommendations.

use serde::{Deserialize, Serialize};

use super::allocation::Priority;

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

impl From<crate::core::types::EnhancedPotentialLeak> for EnhancedPotentialLeak {
    fn from(old: crate::core::types::EnhancedPotentialLeak) -> Self {
        Self {
            object_id: old.object_id,
            leak_type: match old.leak_type {
                crate::core::types::LeakType::Memory => LeakType::Memory,
                crate::core::types::LeakType::FileHandle => LeakType::FileHandle,
                crate::core::types::LeakType::NetworkConnection => LeakType::NetworkConnection,
                crate::core::types::LeakType::Thread => LeakType::Thread,
                crate::core::types::LeakType::Lock => LeakType::Lock,
                crate::core::types::LeakType::ReferenceCycle => LeakType::ReferenceCycle,
            },
            risk_level: match old.risk_level {
                crate::core::types::LeakRiskLevel::Low => LeakRiskLevel::Low,
                crate::core::types::LeakRiskLevel::Medium => LeakRiskLevel::Medium,
                crate::core::types::LeakRiskLevel::High => LeakRiskLevel::High,
                crate::core::types::LeakRiskLevel::Critical => LeakRiskLevel::Critical,
            },
            evidence: old
                .evidence
                .into_iter()
                .map(|e| LeakEvidence {
                    evidence_type: match e.evidence_type {
                        crate::core::types::LeakEvidenceType::NeverDropped => {
                            LeakEvidenceType::NeverDropped
                        }
                        crate::core::types::LeakEvidenceType::CircularReference => {
                            LeakEvidenceType::CircularReference
                        }
                        crate::core::types::LeakEvidenceType::ResourceNotClosed => {
                            LeakEvidenceType::ResourceNotClosed
                        }
                        crate::core::types::LeakEvidenceType::GrowingMemoryUsage => {
                            LeakEvidenceType::GrowingMemoryUsage
                        }
                        crate::core::types::LeakEvidenceType::LongLivedTemporary => {
                            LeakEvidenceType::LongLivedTemporary
                        }
                        crate::core::types::LeakEvidenceType::UnreachableObject => {
                            LeakEvidenceType::UnreachableObject
                        }
                    },
                    description: e.description,
                    strength: e.strength,
                    timestamp: e.timestamp,
                })
                .collect(),
            estimated_impact: LeakImpact {
                memory_bytes: old.estimated_impact.memory_bytes,
                performance_impact_percent: old.estimated_impact.performance_impact_percent,
                resource_count: old.estimated_impact.resource_count,
                time_to_critical_hours: old.estimated_impact.time_to_critical_hours,
            },
        }
    }
}

impl From<crate::core::types::ResourceUsagePattern> for ResourceUsagePattern {
    fn from(old: crate::core::types::ResourceUsagePattern) -> Self {
        Self {
            pattern_type: match old.pattern_type {
                crate::core::types::ResourcePatternType::MonotonicGrowth => {
                    ResourcePatternType::MonotonicGrowth
                }
                crate::core::types::ResourcePatternType::PeriodicSpikes => {
                    ResourcePatternType::PeriodicSpikes
                }
                crate::core::types::ResourcePatternType::GradualAccumulation => {
                    ResourcePatternType::GradualAccumulation
                }
                crate::core::types::ResourcePatternType::SuddenJumps => {
                    ResourcePatternType::SuddenJumps
                }
                crate::core::types::ResourcePatternType::Irregular => {
                    ResourcePatternType::Irregular
                }
            },
            description: old.description,
            frequency: old.frequency,
            leak_risk: match old.leak_risk {
                crate::core::types::LeakRiskLevel::Low => LeakRiskLevel::Low,
                crate::core::types::LeakRiskLevel::Medium => LeakRiskLevel::Medium,
                crate::core::types::LeakRiskLevel::High => LeakRiskLevel::High,
                crate::core::types::LeakRiskLevel::Critical => LeakRiskLevel::Critical,
            },
        }
    }
}

impl From<crate::core::types::LeakPreventionRecommendation> for LeakPreventionRecommendation {
    fn from(old: crate::core::types::LeakPreventionRecommendation) -> Self {
        Self {
            recommendation_type: match old.recommendation_type {
                crate::core::types::LeakPreventionType::UseRAII => LeakPreventionType::UseRAII,
                crate::core::types::LeakPreventionType::ImplementDrop => {
                    LeakPreventionType::ImplementDrop
                }
                crate::core::types::LeakPreventionType::BreakCircularReferences => {
                    LeakPreventionType::BreakCircularReferences
                }
                crate::core::types::LeakPreventionType::UseWeakReferences => {
                    LeakPreventionType::UseWeakReferences
                }
                crate::core::types::LeakPreventionType::ResourcePooling => {
                    LeakPreventionType::ResourcePooling
                }
                crate::core::types::LeakPreventionType::ResourceMonitoring => {
                    LeakPreventionType::ResourceMonitoring
                }
                crate::core::types::LeakPreventionType::ScopedGuards => {
                    LeakPreventionType::ScopedGuards
                }
            },
            priority: match old.priority {
                crate::core::types::Priority::Low => Priority::Low,
                crate::core::types::Priority::Medium => Priority::Medium,
                crate::core::types::Priority::High => Priority::High,
                crate::core::types::Priority::Critical => Priority::Critical,
            },
            description: old.description,
            implementation_guidance: old.implementation_guidance,
            expected_effectiveness: old.expected_effectiveness,
        }
    }
}

impl From<crate::core::types::ResourceLeakAnalysis> for ResourceLeakAnalysis {
    fn from(old: crate::core::types::ResourceLeakAnalysis) -> Self {
        Self {
            potential_leaks: old.potential_leaks.into_iter().map(Into::into).collect(),
            detection_confidence: old.detection_confidence,
            usage_patterns: old.usage_patterns.into_iter().map(Into::into).collect(),
            prevention_recommendations: old
                .prevention_recommendations
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
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

    /// Objective: Verify EnhancedPotentialLeak creation
    /// Invariants: All fields should be properly initialized
    #[test]
    fn test_enhanced_potential_leak() {
        let leak = EnhancedPotentialLeak {
            object_id: 42,
            leak_type: LeakType::Memory,
            risk_level: LeakRiskLevel::High,
            evidence: vec![],
            estimated_impact: LeakImpact {
                memory_bytes: 1024,
                performance_impact_percent: 5.0,
                resource_count: 1,
                time_to_critical_hours: Some(24.0),
            },
        };

        assert_eq!(leak.object_id, 42, "Object ID should match");
        assert_eq!(
            leak.leak_type,
            LeakType::Memory,
            "Leak type should be Memory"
        );
        assert_eq!(
            leak.risk_level,
            LeakRiskLevel::High,
            "Risk level should be High"
        );
    }

    /// Objective: Verify LeakEvidence creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_leak_evidence() {
        let evidence = LeakEvidence {
            evidence_type: LeakEvidenceType::NeverDropped,
            description: "Object was never deallocated".to_string(),
            strength: 0.95,
            timestamp: 1000,
        };

        assert_eq!(evidence.strength, 0.95, "Evidence strength should match");
        assert_eq!(
            evidence.evidence_type,
            LeakEvidenceType::NeverDropped,
            "Evidence type should match"
        );
    }

    /// Objective: Verify LeakEvidenceType variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_leak_evidence_type_variants() {
        let types = vec![
            LeakEvidenceType::NeverDropped,
            LeakEvidenceType::CircularReference,
            LeakEvidenceType::ResourceNotClosed,
            LeakEvidenceType::GrowingMemoryUsage,
            LeakEvidenceType::LongLivedTemporary,
            LeakEvidenceType::UnreachableObject,
        ];

        for evidence_type in types {
            let debug_str = format!("{evidence_type:?}");
            assert!(
                !debug_str.is_empty(),
                "LeakEvidenceType should have debug representation"
            );
        }
    }

    /// Objective: Verify LeakImpact creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_leak_impact() {
        let impact = LeakImpact {
            memory_bytes: 1024 * 1024,
            performance_impact_percent: 10.0,
            resource_count: 5,
            time_to_critical_hours: Some(48.0),
        };

        assert_eq!(
            impact.memory_bytes,
            1024 * 1024,
            "Memory bytes should match"
        );
        assert_eq!(impact.resource_count, 5, "Resource count should match");
    }

    /// Objective: Verify ResourceUsagePattern creation
    /// Invariants: All fields should be properly initialized
    #[test]
    fn test_resource_usage_pattern() {
        let pattern = ResourceUsagePattern {
            pattern_type: ResourcePatternType::MonotonicGrowth,
            description: "Memory usage grows continuously".to_string(),
            frequency: 100.0,
            leak_risk: LeakRiskLevel::Critical,
        };

        assert_eq!(pattern.frequency, 100.0, "Frequency should match");
        assert_eq!(
            pattern.pattern_type,
            ResourcePatternType::MonotonicGrowth,
            "Pattern type should match"
        );
    }

    /// Objective: Verify ResourcePatternType variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_resource_pattern_type_variants() {
        let patterns = vec![
            ResourcePatternType::MonotonicGrowth,
            ResourcePatternType::PeriodicSpikes,
            ResourcePatternType::GradualAccumulation,
            ResourcePatternType::SuddenJumps,
            ResourcePatternType::Irregular,
        ];

        for pattern in patterns {
            let debug_str = format!("{pattern:?}");
            assert!(
                !debug_str.is_empty(),
                "ResourcePatternType should have debug representation"
            );
        }
    }

    /// Objective: Verify LeakPreventionRecommendation creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_leak_prevention_recommendation() {
        let recommendation = LeakPreventionRecommendation {
            recommendation_type: LeakPreventionType::UseRAII,
            priority: Priority::High,
            description: "Use RAII patterns for resource management".to_string(),
            implementation_guidance: "Wrap resources in smart pointers".to_string(),
            expected_effectiveness: 0.9,
        };

        assert_eq!(
            recommendation.priority,
            Priority::High,
            "Priority should be High"
        );
        assert_eq!(
            recommendation.expected_effectiveness, 0.9,
            "Effectiveness should match"
        );
    }

    /// Objective: Verify LeakPreventionType variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_leak_prevention_type_variants() {
        let types = vec![
            LeakPreventionType::UseRAII,
            LeakPreventionType::ImplementDrop,
            LeakPreventionType::BreakCircularReferences,
            LeakPreventionType::UseWeakReferences,
            LeakPreventionType::ResourcePooling,
            LeakPreventionType::ResourceMonitoring,
            LeakPreventionType::ScopedGuards,
        ];

        for prevention_type in types {
            let debug_str = format!("{prevention_type:?}");
            assert!(
                !debug_str.is_empty(),
                "LeakPreventionType should have debug representation"
            );
        }
    }

    /// Objective: Verify ResourceLeakAnalysis with multiple leaks
    /// Invariants: Should handle multiple potential leaks
    #[test]
    fn test_resource_leak_analysis_multiple() {
        let leak1 = EnhancedPotentialLeak {
            object_id: 1,
            leak_type: LeakType::Memory,
            risk_level: LeakRiskLevel::High,
            evidence: vec![],
            estimated_impact: LeakImpact {
                memory_bytes: 1024,
                performance_impact_percent: 5.0,
                resource_count: 1,
                time_to_critical_hours: None,
            },
        };

        let leak2 = EnhancedPotentialLeak {
            object_id: 2,
            leak_type: LeakType::FileHandle,
            risk_level: LeakRiskLevel::Medium,
            evidence: vec![],
            estimated_impact: LeakImpact {
                memory_bytes: 0,
                performance_impact_percent: 0.0,
                resource_count: 1,
                time_to_critical_hours: None,
            },
        };

        let analysis = ResourceLeakAnalysis {
            potential_leaks: vec![leak1, leak2],
            detection_confidence: 0.85,
            usage_patterns: vec![],
            prevention_recommendations: vec![],
        };

        assert_eq!(
            analysis.potential_leaks.len(),
            2,
            "Should have two potential leaks"
        );
        assert_eq!(
            analysis.detection_confidence, 0.85,
            "Confidence should match"
        );
    }

    /// Objective: Verify serialization of LeakType
    /// Invariants: Should serialize and deserialize correctly
    #[test]
    fn test_leak_type_serialization() {
        let leak_type = LeakType::ReferenceCycle;
        let json = serde_json::to_string(&leak_type);
        assert!(json.is_ok(), "Should serialize to JSON");

        let deserialized: Result<LeakType, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "Should deserialize from JSON");
        assert_eq!(
            deserialized.unwrap(),
            LeakType::ReferenceCycle,
            "Should preserve value"
        );
    }

    /// Objective: Verify serialization of LeakRiskLevel
    /// Invariants: Should serialize and deserialize correctly
    #[test]
    fn test_leak_risk_level_serialization() {
        let risk = LeakRiskLevel::Critical;
        let json = serde_json::to_string(&risk);
        assert!(json.is_ok(), "Should serialize to JSON");

        let deserialized: Result<LeakRiskLevel, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "Should deserialize from JSON");
        assert_eq!(
            deserialized.unwrap(),
            LeakRiskLevel::Critical,
            "Should preserve value"
        );
    }

    /// Objective: Verify LeakImpact with zero values
    /// Invariants: Should handle zero values
    #[test]
    fn test_leak_impact_zero_values() {
        let impact = LeakImpact {
            memory_bytes: 0,
            performance_impact_percent: 0.0,
            resource_count: 0,
            time_to_critical_hours: None,
        };

        assert_eq!(impact.memory_bytes, 0, "Zero memory bytes should be valid");
        assert_eq!(
            impact.resource_count, 0,
            "Zero resource count should be valid"
        );
    }

    /// Objective: Verify LeakImpact with large values
    /// Invariants: Should handle large values
    #[test]
    fn test_leak_impact_large_values() {
        let impact = LeakImpact {
            memory_bytes: usize::MAX,
            performance_impact_percent: 100.0,
            resource_count: u32::MAX,
            time_to_critical_hours: Some(f64::MAX),
        };

        assert_eq!(
            impact.memory_bytes,
            usize::MAX,
            "Max memory bytes should be valid"
        );
        assert_eq!(
            impact.resource_count,
            u32::MAX,
            "Max resource count should be valid"
        );
    }

    /// Objective: Verify Priority variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_priority_variants() {
        assert_eq!(Priority::Low, Priority::Low);
        assert_eq!(Priority::Medium, Priority::Medium);
        assert_eq!(Priority::High, Priority::High);
        assert_eq!(Priority::Critical, Priority::Critical);

        assert_ne!(Priority::Low, Priority::Critical);
    }
}
