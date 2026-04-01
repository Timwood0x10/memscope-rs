//! Ownership tracking types.
//!
//! This module contains types for tracking ownership hierarchy,
//! ownership transfers, weak references, and circular references.

use serde::{Deserialize, Serialize};

use super::allocation::ImpactLevel;
use super::generic::MemoryImpact;
use super::leak_detection::{LeakRiskLevel, ResourceUsagePattern};

/// Ownership hierarchy analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OwnershipHierarchy {
    /// Root owners in the hierarchy.
    pub root_owners: Vec<OwnershipNode>,
    /// Maximum ownership depth.
    pub max_depth: usize,
    /// Total objects in hierarchy.
    pub total_objects: usize,
    /// Ownership transfer events.
    pub transfer_events: Vec<OwnershipTransferEvent>,
    /// Weak reference analysis.
    pub weak_references: Vec<WeakReferenceInfo>,
    /// Circular reference detection.
    pub circular_references: Vec<CircularReferenceInfo>,
}

/// Node in ownership hierarchy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OwnershipNode {
    /// Object identifier.
    pub object_id: usize,
    /// Object type name.
    pub type_name: String,
    /// Ownership type.
    pub ownership_type: OwnershipType,
    /// Owned objects.
    pub owned_objects: Vec<OwnershipNode>,
    /// Reference count (for Rc/Arc).
    pub reference_count: Option<usize>,
    /// Weak reference count.
    pub weak_reference_count: Option<usize>,
}

/// Types of ownership.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OwnershipType {
    /// Unique ownership (Box, owned values).
    Unique,
    /// Shared ownership (Rc).
    SharedSingleThreaded,
    /// Shared ownership (Arc).
    SharedMultiThreaded,
    /// Borrowed reference.
    Borrowed,
    /// Weak reference.
    Weak,
    /// Raw pointer (unsafe).
    Raw,
}

/// Ownership transfer event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OwnershipTransferEvent {
    /// Source object.
    pub source_object: usize,
    /// Target object.
    pub target_object: usize,
    /// Transfer type.
    pub transfer_type: OwnershipTransferType,
    /// Transfer timestamp.
    pub timestamp: u64,
    /// Transfer mechanism.
    pub mechanism: String,
}

/// Types of ownership transfers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OwnershipTransferType {
    /// Move operation.
    Move,
    /// Clone operation.
    Clone,
    /// Reference creation.
    Borrow,
    /// Reference counting increment.
    ReferenceIncrement,
    /// Reference counting decrement.
    ReferenceDecrement,
}

/// Weak reference information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeakReferenceInfo {
    /// Weak reference object ID.
    pub weak_ref_id: usize,
    /// Target object ID.
    pub target_object_id: usize,
    /// Weak reference type.
    pub weak_ref_type: WeakReferenceType,
    /// Is target still alive.
    pub target_alive: bool,
    /// Upgrade attempts.
    pub upgrade_attempts: u32,
    /// Successful upgrades.
    pub successful_upgrades: u32,
}

/// Types of weak references.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WeakReferenceType {
    /// std::rc::Weak.
    RcWeak,
    /// std::sync::Weak.
    ArcWeak,
    /// Custom weak reference.
    Custom,
}

/// Circular reference information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircularReferenceInfo {
    /// Objects involved in the cycle.
    pub cycle_objects: Vec<usize>,
    /// Cycle detection timestamp.
    pub detection_timestamp: u64,
    /// Cycle type.
    pub cycle_type: CircularReferenceType,
    /// Potential memory leak risk.
    pub leak_risk: LeakRiskLevel,
    /// Suggested resolution.
    pub resolution_suggestion: String,
}

/// Types of circular references.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircularReferenceType {
    /// Direct circular reference (A -> B -> A).
    Direct,
    /// Indirect circular reference (A -> B -> C -> A).
    Indirect,
    /// Self-referential (A -> A).
    SelfReferential,
    /// Complex multi-path cycle.
    Complex,
}

/// Type relationship information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeRelationshipInfo {
    /// Type name.
    pub type_name: String,
    /// Parent types (traits, base structs).
    pub parent_types: Vec<ParentTypeInfo>,
    /// Child types (implementors, derived types).
    pub child_types: Vec<ChildTypeInfo>,
    /// Composed types (fields, associated types).
    pub composed_types: Vec<ComposedTypeInfo>,
    /// Relationship complexity score.
    pub complexity_score: u32,
    /// Inheritance depth.
    pub inheritance_depth: u32,
    /// Composition breadth.
    pub composition_breadth: u32,
}

/// Parent type information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParentTypeInfo {
    /// Parent type name.
    pub type_name: String,
    /// Relationship type.
    pub relationship_type: RelationshipType,
    /// Inheritance level.
    pub inheritance_level: u32,
    /// Memory layout impact.
    pub memory_impact: MemoryImpact,
}

/// Child type information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChildTypeInfo {
    /// Child type name.
    pub type_name: String,
    /// Relationship type.
    pub relationship_type: RelationshipType,
    /// Specialization level.
    pub specialization_level: u32,
    /// Usage frequency.
    pub usage_frequency: u32,
}

/// Composed type information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComposedTypeInfo {
    /// Composed type name.
    pub type_name: String,
    /// Field or association name.
    pub field_name: String,
    /// Composition type.
    pub composition_type: CompositionType,
    /// Memory offset (if applicable).
    pub memory_offset: Option<usize>,
    /// Access frequency.
    pub access_frequency: u32,
}

/// Type relationship types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Trait implementation relationship.
    TraitImplementation,
    /// Trait bound relationship.
    TraitBound,
    /// Inheritance relationship.
    Inheritance,
    /// Association relationship.
    Association,
    /// Aggregation relationship.
    Composition,
    /// Dependency relationship.
    Dependency,
}

/// Composition types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompositionType {
    /// Field composition type.
    Field,
    /// Associated type composition type.
    AssociatedType,
    /// Generic parameter composition type.
    GenericParameter,
    /// Nested type composition type.
    NestedType,
    /// Reference composition type.
    Reference,
    /// Smart pointer composition type.
    SmartPointer,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_hierarchy() {
        let hierarchy = OwnershipHierarchy {
            root_owners: vec![],
            max_depth: 3,
            total_objects: 10,
            transfer_events: vec![],
            weak_references: vec![],
            circular_references: vec![],
        };

        assert_eq!(hierarchy.max_depth, 3);
        assert_eq!(hierarchy.total_objects, 10);
    }

    #[test]
    fn test_ownership_type() {
        let ownership = OwnershipType::SharedMultiThreaded;
        assert!(matches!(ownership, OwnershipType::SharedMultiThreaded));
    }

    #[test]
    fn test_weak_reference_info() {
        let weak_ref = WeakReferenceInfo {
            weak_ref_id: 1,
            target_object_id: 100,
            weak_ref_type: WeakReferenceType::ArcWeak,
            target_alive: true,
            upgrade_attempts: 5,
            successful_upgrades: 4,
        };

        assert_eq!(weak_ref.upgrade_attempts, 5);
        assert!(weak_ref.target_alive);
    }
}

impl From<crate::core::types::TypeRelationshipInfo> for TypeRelationshipInfo {
    fn from(old: crate::core::types::TypeRelationshipInfo) -> Self {
        Self {
            type_name: old.type_name,
            parent_types: old
                .parent_types
                .into_iter()
                .map(|p| ParentTypeInfo {
                    type_name: p.type_name,
                    relationship_type: match p.relationship_type {
                        crate::core::types::RelationshipType::TraitImplementation => {
                            RelationshipType::TraitImplementation
                        }
                        crate::core::types::RelationshipType::TraitBound => {
                            RelationshipType::TraitBound
                        }
                        crate::core::types::RelationshipType::Inheritance => {
                            RelationshipType::Inheritance
                        }
                        crate::core::types::RelationshipType::Association => {
                            RelationshipType::Association
                        }
                        crate::core::types::RelationshipType::Composition => {
                            RelationshipType::Composition
                        }
                        crate::core::types::RelationshipType::Dependency => {
                            RelationshipType::Dependency
                        }
                    },
                    inheritance_level: p.inheritance_level,
                    memory_impact: match p.memory_impact {
                        crate::core::types::MemoryImpact::None => MemoryImpact::None,
                        crate::core::types::MemoryImpact::SizeIncrease(s) => {
                            MemoryImpact::SizeIncrease(s)
                        }
                        crate::core::types::MemoryImpact::AlignmentChange(s) => {
                            MemoryImpact::AlignmentChange(s)
                        }
                        crate::core::types::MemoryImpact::LayoutChange(s) => {
                            MemoryImpact::LayoutChange(s)
                        }
                    },
                })
                .collect(),
            child_types: old
                .child_types
                .into_iter()
                .map(|c| ChildTypeInfo {
                    type_name: c.type_name,
                    relationship_type: match c.relationship_type {
                        crate::core::types::RelationshipType::TraitImplementation => {
                            RelationshipType::TraitImplementation
                        }
                        crate::core::types::RelationshipType::TraitBound => {
                            RelationshipType::TraitBound
                        }
                        crate::core::types::RelationshipType::Inheritance => {
                            RelationshipType::Inheritance
                        }
                        crate::core::types::RelationshipType::Association => {
                            RelationshipType::Association
                        }
                        crate::core::types::RelationshipType::Composition => {
                            RelationshipType::Composition
                        }
                        crate::core::types::RelationshipType::Dependency => {
                            RelationshipType::Dependency
                        }
                    },
                    specialization_level: c.specialization_level,
                    usage_frequency: c.usage_frequency,
                })
                .collect(),
            composed_types: old
                .composed_types
                .into_iter()
                .map(|c| ComposedTypeInfo {
                    type_name: c.type_name,
                    field_name: c.field_name,
                    composition_type: match c.composition_type {
                        crate::core::types::CompositionType::Field => CompositionType::Field,
                        crate::core::types::CompositionType::AssociatedType => {
                            CompositionType::AssociatedType
                        }
                        crate::core::types::CompositionType::GenericParameter => {
                            CompositionType::GenericParameter
                        }
                        crate::core::types::CompositionType::NestedType => {
                            CompositionType::NestedType
                        }
                        crate::core::types::CompositionType::Reference => {
                            CompositionType::Reference
                        }
                        crate::core::types::CompositionType::SmartPointer => {
                            CompositionType::SmartPointer
                        }
                    },
                    memory_offset: c.memory_offset,
                    access_frequency: c.access_frequency,
                })
                .collect(),
            complexity_score: old.complexity_score,
            inheritance_depth: old.inheritance_depth,
            composition_breadth: old.composition_breadth,
        }
    }
}

impl From<crate::core::types::OwnershipNode> for OwnershipNode {
    fn from(old: crate::core::types::OwnershipNode) -> Self {
        Self {
            object_id: old.object_id,
            type_name: old.type_name,
            ownership_type: match old.ownership_type {
                crate::core::types::OwnershipType::Unique => OwnershipType::Unique,
                crate::core::types::OwnershipType::SharedSingleThreaded => {
                    OwnershipType::SharedSingleThreaded
                }
                crate::core::types::OwnershipType::SharedMultiThreaded => {
                    OwnershipType::SharedMultiThreaded
                }
                crate::core::types::OwnershipType::Borrowed => OwnershipType::Borrowed,
                crate::core::types::OwnershipType::Weak => OwnershipType::Weak,
                crate::core::types::OwnershipType::Raw => OwnershipType::Raw,
            },
            owned_objects: old.owned_objects.into_iter().map(Into::into).collect(),
            reference_count: old.reference_count,
            weak_reference_count: old.weak_reference_count,
        }
    }
}

impl From<crate::core::types::OwnershipTransferEvent> for OwnershipTransferEvent {
    fn from(old: crate::core::types::OwnershipTransferEvent) -> Self {
        Self {
            source_object: old.source_object,
            target_object: old.target_object,
            transfer_type: match old.transfer_type {
                crate::core::types::OwnershipTransferType::Move => OwnershipTransferType::Move,
                crate::core::types::OwnershipTransferType::Clone => OwnershipTransferType::Clone,
                crate::core::types::OwnershipTransferType::Borrow => OwnershipTransferType::Borrow,
                crate::core::types::OwnershipTransferType::ReferenceIncrement => {
                    OwnershipTransferType::ReferenceIncrement
                }
                crate::core::types::OwnershipTransferType::ReferenceDecrement => {
                    OwnershipTransferType::ReferenceDecrement
                }
            },
            timestamp: old.timestamp,
            mechanism: old.mechanism,
        }
    }
}

impl From<crate::core::types::WeakReferenceInfo> for WeakReferenceInfo {
    fn from(old: crate::core::types::WeakReferenceInfo) -> Self {
        Self {
            weak_ref_id: old.weak_ref_id,
            target_object_id: old.target_object_id,
            weak_ref_type: match old.weak_ref_type {
                crate::core::types::WeakReferenceType::RcWeak => WeakReferenceType::RcWeak,
                crate::core::types::WeakReferenceType::ArcWeak => WeakReferenceType::ArcWeak,
                crate::core::types::WeakReferenceType::Custom => WeakReferenceType::Custom,
            },
            target_alive: old.target_alive,
            upgrade_attempts: old.upgrade_attempts,
            successful_upgrades: old.successful_upgrades,
        }
    }
}

impl From<crate::core::types::CircularReferenceInfo> for CircularReferenceInfo {
    fn from(old: crate::core::types::CircularReferenceInfo) -> Self {
        Self {
            cycle_objects: old.cycle_objects,
            detection_timestamp: old.detection_timestamp,
            cycle_type: match old.cycle_type {
                crate::core::types::CircularReferenceType::Direct => CircularReferenceType::Direct,
                crate::core::types::CircularReferenceType::Indirect => {
                    CircularReferenceType::Indirect
                }
                crate::core::types::CircularReferenceType::SelfReferential => {
                    CircularReferenceType::SelfReferential
                }
                crate::core::types::CircularReferenceType::Complex => {
                    CircularReferenceType::Complex
                }
            },
            leak_risk: match old.leak_risk {
                crate::core::types::LeakRiskLevel::Low => LeakRiskLevel::Low,
                crate::core::types::LeakRiskLevel::Medium => LeakRiskLevel::Medium,
                crate::core::types::LeakRiskLevel::High => LeakRiskLevel::High,
                crate::core::types::LeakRiskLevel::Critical => LeakRiskLevel::Critical,
            },
            resolution_suggestion: old.resolution_suggestion,
        }
    }
}

impl From<crate::core::types::OwnershipHierarchy> for OwnershipHierarchy {
    fn from(old: crate::core::types::OwnershipHierarchy) -> Self {
        Self {
            root_owners: old.root_owners.into_iter().map(Into::into).collect(),
            max_depth: old.max_depth,
            total_objects: old.total_objects,
            transfer_events: old.transfer_events.into_iter().map(Into::into).collect(),
            weak_references: old.weak_references.into_iter().map(Into::into).collect(),
            circular_references: old
                .circular_references
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}
