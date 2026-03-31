//! Capture types module.
//!
//! This module contains all type definitions for the capture engine,
//! organized into logical submodules for better maintainability.
//!
//! # Organization
//!
//! Types are organized by their functional area:
//! - `allocation`: Memory allocation tracking types
//! - `smart_pointer`: Smart pointer tracking (Rc, Arc, Box)
//! - `error`: Error types for tracking operations
//! - `stats`: Memory statistics and analysis
//! - `scope`: Scope tracking and lifecycle
//! - `memory_layout`: Memory layout analysis
//! - `generic`: Generic type tracking
//! - `dynamic_type`: Dynamic type (trait object) tracking
//! - `runtime_state`: Runtime state information
//! - `fragmentation`: Memory fragmentation analysis
//! - `ownership`: Ownership hierarchy tracking
//! - `drop_chain`: Drop chain analysis
//! - `leak_detection`: Memory leak detection
//! - `timeline`: Timeline visualization types
//! - `access_tracking`: Memory access pattern tracking
//! - `stack`: Stack allocation tracking
//! - `temporary`: Temporary object tracking
//! - `tracking`: Function call tracking

pub mod access_tracking;
pub mod allocation;
pub mod drop_chain;
pub mod dynamic_type;
pub mod error;
pub mod fragmentation;
pub mod generic;
pub mod leak_detection;
pub mod lifecycle;
pub mod memory_layout;
pub mod ownership;
pub mod runtime_state;
pub mod scope;
pub mod smart_pointer;
pub mod stack;
pub mod stats;
pub mod temporary;
pub mod timeline;
pub mod tracking;

// Re-export common types for convenience
pub use access_tracking::{
    AccessPattern, AccessPatternType, AddressRange, BandwidthBottleneck,
    BandwidthBottleneckLocation, BandwidthUtilization, CacheAccessInfo, CacheLatencyBreakdown,
    ImplementationDifficulty, LocalityMetrics, MemoryAccessEvent, MemoryAccessPerformanceImpact,
    MemoryAccessStatistics, MemoryAccessTrackingInfo, MemoryAccessType,
    MemoryOptimizationRecommendation, MemoryOptimizationType, StridePattern,
};
pub use allocation::{
    AllocationInfo, BorrowInfo, BottleneckType, CloneInfo, ContextPerformanceMetrics, ContextType,
    HotPath, ImpactLevel, ImplementationDifficulty as AllocImplDifficulty,
    OptimizationRecommendation, PerformanceBottleneck, PerformanceSnapshot, Priority,
    RecommendationType, TypePerformanceImpact, TypeUsageInfo, UsageContext, UsageTimePoint,
};
pub use drop_chain::{
    CleanupAction, CleanupActionType, DropBottleneckType, DropChainAnalysis, DropChainNode,
    DropChainPerformanceMetrics, DropImplementationType, DropPerformanceBottleneck,
    DropPerformanceCharacteristics,
};
pub use dynamic_type::{
    DispatchOverhead, DynamicTypeInfo, PerformanceImpact, TypeErasureInfo, VTableInfo, VTableMethod,
};
pub use error::{TrackingError, TrackingResult};
pub use fragmentation::{
    BlockSizeRange, EnhancedFragmentationAnalysis, FragmentationCause, FragmentationCauseType,
    FragmentationMetrics, FragmentationSeverity,
};
pub use generic::{
    BranchPredictionImpact, CacheImpact, CodeBloatLevel, CompilationImpact, ConcreteTypeParameter,
    ConstraintType, GenericConstraint, GenericInstantiationInfo, GenericTypeInfo, MemoryImpact,
    MonomorphizationInfo, OptimizationDifficulty, PerformanceCharacteristics, SourceLocation,
    TypeCategory, TypeParameter,
};
pub use leak_detection::{
    EnhancedPotentialLeak, LeakEvidence, LeakEvidenceType, LeakImpact,
    LeakPreventionRecommendation, LeakPreventionType, LeakRiskLevel, LeakType,
    ResourceLeakAnalysis, ResourcePatternType, ResourceUsagePattern,
};
pub use lifecycle::{
    BorrowState, EventPerformanceMetrics, LifecycleEfficiencyMetrics, LifecycleEvent,
    LifecycleEventType, LifecyclePattern, LifecyclePatternType, LifecycleStageDurations,
    MemoryLocationType, MemoryState, ObjectLifecycleInfo, ResourceWasteAssessment,
    SimpleLifecyclePattern,
};
pub use memory_layout::{
    AccessEfficiency, CapacityUtilization, ContainerAnalysis, ContainerEfficiencyMetrics,
    ContainerType, FieldLayoutInfo, GrowthPattern, LayoutEfficiency, MemoryLayoutInfo,
    OptimizationPotential, PaddingAnalysis, PaddingLocation, PaddingReason, ReallocationFrequency,
    ReallocationPatterns, UtilizationEfficiency,
};
pub use ownership::{
    ChildTypeInfo, CircularReferenceInfo, CircularReferenceType, ComposedTypeInfo, CompositionType,
    OwnershipHierarchy, OwnershipNode, OwnershipTransferEvent, OwnershipTransferType,
    OwnershipType, ParentTypeInfo, RelationshipType, TypeRelationshipInfo, WeakReferenceInfo,
    WeakReferenceType,
};
pub use runtime_state::{
    AllocatorStateInfo, CachePerformanceInfo, CpuUsageInfo, GcInfo, MemoryAccessPattern,
    MemoryPressureInfo, MemoryPressureLevel, RuntimeStateInfo,
};
pub use scope::{
    AllocationEventType, BorrowEvent, GrowthEvent, GrowthReason, MoveEvent, PotentialLeak,
    RiskDistribution, ScopeEventType, ScopeHierarchy, ScopeInfo, TypeLifecyclePattern,
    VariableRelationship,
};
// Re-export from core::types
pub use crate::core::types::{ScopeAnalysis, ScopeLifecycleMetrics};
pub use smart_pointer::{RefCountSnapshot, SmartPointerInfo, SmartPointerType};
pub use stack::{ScopeType, StackAllocationInfo, StackScopeInfo};
pub use stats::{
    ConcurrencyAnalysis, FragmentationAnalysis as StatsFragmentationAnalysis, LibraryUsage,
    MemoryStats, MemoryTypeInfo, SystemLibraryStats, TypeMemoryUsage,
};
pub use temporary::{CreationContext, ExpressionType, TemporaryObjectInfo, TemporaryUsagePattern};
pub use timeline::{
    AllocationEvent, AllocationHotspot, AllocationPattern, HotspotLocation, MemorySnapshot,
    SafetyViolation, StackFrame, StackTraceData, StackTraceHotspot, TimeRange, TimelineData,
};
pub use tracking::{
    CallPattern, CallPatternType, CallSequence, CallStackInfo, ConcurrencyCharacteristics,
    DeadlockRisk, FunctionCallTrackingInfo, FunctionMemoryCharacteristics,
    FunctionPerformanceCharacteristics, IOCharacteristics, LeakPotential, MemoryUsagePattern,
    RecursionPerformanceImpact, RecursiveCallInfo, StackOverflowRisk, ThreadSafetyLevel,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_reexports() {
        let error = TrackingError::TrackingDisabled;
        assert!(matches!(error, TrackingError::TrackingDisabled));
    }

    #[test]
    fn test_allocation_info_creation() {
        let info = AllocationInfo::new(0x1000, 1024);
        assert_eq!(info.ptr, 0x1000);
        assert_eq!(info.size, 1024);
    }

    #[test]
    fn test_smart_pointer_info_creation() {
        let info = SmartPointerInfo::new_rc_arc(0x1000, SmartPointerType::Rc, 1, 0);
        assert_eq!(info.data_ptr, 0x1000);
        assert!(info.is_data_owner);
    }

    #[test]
    fn test_memory_stats_creation() {
        let stats = MemoryStats::new();
        assert_eq!(stats.total_allocations, 0);
    }
}
