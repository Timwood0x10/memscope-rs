//! Enhanced types for comprehensive memory analysis
//!
//! This module contains additional type definitions needed for the enhanced memory analysis system.
//! These types support advanced memory tracking features including stack/heap distinction,
//! temporary object optimization, fragmentation monitoring, generic type analysis,
//! object lifecycle tracking, and memory access pattern analysis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
// Import specific types rather than wildcard to avoid conflicts
use crate::core::types::{
    AccessPattern, AllocationInfo, CreationContext, FragmentationAnalysis,
    LifecycleEfficiencyMetrics, LifecyclePattern, ObjectLifecycleInfo, OptimizationPotential,
    OptimizationRecommendation, PerformanceCharacteristics, PerformanceImpact, ScopeType,
    TemporaryUsagePattern,
};

///  Stack Frame and Boundary Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedStackFrame {
    /// Frame ID
    pub frame_id: u64,
    /// Function name
    pub function_name: String,
    /// Allocations in the frame
    pub allocations: Vec<usize>,
    /// Total allocated memory in the frame
    pub total_allocated: usize,
    /// Frame size
    pub frame_size: usize,
}

/// Stack boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackBoundaries {
    /// Stack base address
    pub stack_base: usize,
    /// Stack top address
    pub stack_top: usize,
    /// Stack size
    pub stack_size: usize,
}

impl StackBoundaries {
    /// Detect stack boundaries
    pub fn detect() -> Self {
        // Detect stack boundaries using platform-specific methods
        // This is a simplified implementation
        let stack_base = 0x7fff_0000_0000; // Typical stack base on x64
        let stack_size = 8 * 1024 * 1024; // 8MB default stack size

        Self {
            stack_base,
            stack_top: stack_base + stack_size,
            stack_size,
        }
    }

    /// Check if a pointer is within the stack boundaries
    pub fn contains(&self, ptr: usize) -> bool {
        ptr >= self.stack_base && ptr < self.stack_top
    }

    /// Get the base address of a stack frame
    pub fn get_frame_base(&self, frame_id: u64) -> usize {
        // Estimate frame base from frame ID
        self.stack_base + (frame_id as usize * 4096)
    }
}

/// Heap Boundary Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapSegment {
    /// Start address of the heap segment
    pub start: usize,
    /// End address of the heap segment
    pub end: usize,
}

impl HeapSegment {
        pub fn contains(&self, ptr: usize) -> bool {
        ptr >= self.start && ptr < self.end
    }
}

/// Allocator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatorInfo {
    /// Allocator name
    pub name: String,
    /// Allocator strategy
    pub strategy: AllocationStrategy,
    /// Allocator heap segments
    pub heap_segments: Vec<HeapSegment>,
}

/// Allocation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationStrategy {
    /// First fit allocation strategy
    FirstFit,
    /// Best fit allocation strategy
    BestFit,
    /// Worst fit allocation strategy
    WorstFit,
    /// Next fit allocation strategy
    NextFit,
    /// Buddy system allocation strategy
    SlabAllocation,
}

/// Enhanced Temporary Object Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTemporaryObjectInfo {
    /// Allocation information
    pub allocation: AllocationInfo,
    /// Temporary pattern classification
    pub pattern_classification: TemporaryPatternClassification,
    /// Temporary usage pattern
    pub usage_pattern: TemporaryUsagePattern,
    /// Hot path involvement
    pub hot_path_involvement: bool,
    /// Elimination feasibility
    pub elimination_feasibility: EliminationFeasibility,
    /// Optimization potential
    pub optimization_potential: OptimizationPotential,
    /// Creation context
    pub creation_context: CreationContext,
    /// Lifetime analysis
    pub lifetime_analysis: TemporaryLifetimeAnalysis,
    /// Performance impact
    pub performance_impact: PerformanceImpact,
}

/// Temporary pattern classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemporaryPatternClassification {
    /// String concatenation pattern classification
    StringConcatenation,
    /// Vector reallocation pattern classification
    VectorReallocation,
    /// Iterator chaining pattern classification
    IteratorChaining,
    /// Closure capture pattern classification
    ClosureCapture,
    /// Async await pattern classification
    AsyncAwait,
    /// Error handling pattern classification
    ErrorHandling,
    /// Serialization/Deserialization pattern classification
    SerializationDeserialization,
    /// Generic instantiation pattern classification
    GenericInstantiation,
    /// Trait object creation pattern classification
    TraitObjectCreation,
    /// Unknown pattern classification
    Unknown,
}

/// Elimination feasibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EliminationFeasibility {
    /// Highly feasible elimination feasibility
    HighlyFeasible {
        /// Suggested approach for elimination
        suggested_approach: String,
    },
    /// Feasible elimination feasibility
    Feasible {
        /// Constraints for feasibility
        constraints: Vec<String>,
    },
    /// Difficult elimination feasibility
    Difficult {
        /// Blockers for difficulty
        blockers: Vec<String>,
    },
    /// Infeasible elimination feasibility
    Infeasible {
        /// Reasons for infeasibility
        reasons: Vec<String>,
    },
}

/// Temporary lifetime analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryLifetimeAnalysis {
    /// Temporary creation time
    pub creation_time: u64,
    /// Temporary destruction time
    pub destruction_time: Option<u64>,
    /// Temporary estimated lifetime
    pub estimated_lifetime: Duration,
    /// Temporary usage frequency
    pub usage_frequency: usize,
    /// Temporary scope escape analysis
    pub scope_escape_analysis: EscapeAnalysis,
}

/// Analysis of how a variable escapes its scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscapeAnalysis {
    /// Variable does not escape its local scope
    DoesNotEscape,
    /// Variable escapes to the heap (stored in a heap allocation)
    EscapesToHeap,
    /// Variable escapes to the calling function
    EscapesToCaller,
    /// Variable escapes to a global scope
    EscapesToGlobal,
    /// Escape behavior is unknown or cannot be determined
    Unknown,
}

/// Fragmentation Analysis Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationEvent {
    /// Allocation timestamp
    pub timestamp: u64,
    /// Allocation event type
    pub event_type: EnhancedAllocationEventType,
    /// Allocation pointer
    pub ptr: usize,
    /// Allocation size
    pub size: usize,
    /// Allocation type name
    pub type_name: Option<String>,
}

/// Types of memory allocation events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnhancedAllocationEventType {
    /// Memory allocation event
    Allocate,
    /// Memory deallocation event
    Deallocate,
    /// Memory reallocation event (resize)
    Reallocate,
}

/// Real-time metrics for memory fragmentation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeMetrics {
    /// Current fragmentation level
    pub current_fragmentation: f64,
    /// Allocation rate
    pub allocation_rate: f64,
    /// Deallocation rate
    pub deallocation_rate: f64,
    /// Memory pressure
    pub memory_pressure: f64,
}

impl RealTimeMetrics {
    /// Create a new instance of RealTimeMetrics
    pub fn new() -> Self {
        Self {
            current_fragmentation: 0.0,
            allocation_rate: 0.0,
            deallocation_rate: 0.0,
            memory_pressure: 0.0,
        }
    }

    /// Update metrics based on new allocation
    pub fn update_allocation(&mut self, _allocation: &AllocationInfo) {
        // Update metrics based on new allocation
        self.allocation_rate += 1.0;
        // Additional metric updates would go here
    }
}
/// Fragmentation causes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FragmentationCause {
    /// Frequent small allocations cause fragmentation
    FrequentSmallAllocations,
    /// Mixed size allocations cause fragmentation
    MixedSizeAllocations,
    /// Long-lived allocations cause fragmentation
    LongLivedAllocations,
    /// Poor deallocation patterns cause fragmentation
    PoorDeallocationPatterns,
    /// Allocator limitations cause fragmentation
    AllocatorLimitations,
}

/// Fragmentation mitigation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationMitigationStrategy {
    /// Mitigation strategy type
    pub strategy_type: MitigationStrategyType,
    /// Description of the mitigation strategy
    pub description: String,
    /// Expected improvement from the mitigation strategy
    pub expected_improvement: f64,
    /// Implementation complexity of the mitigation strategy
    pub implementation_complexity: ImplementationComplexity,
}

/// Types of fragmentation mitigation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MitigationStrategyType {
    /// Pool allocation strategy
    PoolAllocation,
    /// Size class segregation strategy
    SizeClassSegregation,
    /// Generational garbage collection strategy
    GenerationalGC,
    /// Compaction garbage collection strategy
    CompactionGC,
    /// Custom allocator strategy
    CustomAllocator,
}

/// Complexity levels for implementing optimization strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationComplexity {
    /// Low complexity - easy to implement with minimal changes
    Low,
    /// Medium complexity - requires moderate changes but straightforward
    Medium,
    /// High complexity - requires significant changes and careful planning
    High,
    /// Very high complexity - requires extensive changes and deep system knowledge
    VeryHigh,
}

/// Stack allocation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackAllocationDetails {
    /// Allocation information
    pub allocation: AllocationInfo,
    /// Frame information
    pub frame_info: crate::core::types::StackFrame,
    /// Stack depth
    pub stack_depth: usize,
    /// Scope analysis
    pub scope_analysis: StackScopeAnalysis,
}
/// Heap allocation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapAllocationDetails {
    /// Allocation information
    pub allocation: AllocationInfo,
    /// Heap region information
    pub heap_info: HeapRegionInfo,
    /// Allocator type
    pub allocator_type: String,
    /// Fragmentation impact on memory performance
    pub fragmentation_impact: FragmentationImpact,
}

/// Ambiguous allocation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguousAllocation {
    /// Allocation information
    pub allocation: AllocationInfo,
    /// Reason for ambiguity
    pub ambiguity_reason: AmbiguityReason,
    /// Confidence score for the allocation
    pub confidence_score: f64,
}

/// Stack scope analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackScopeAnalysis {
    /// Scope type
    pub scope_type: ScopeType,
    /// Nesting level of the scope
    pub nesting_level: usize,
    /// Estimated lifetime of the scope
    pub estimated_lifetime: Duration,
    /// Escape analysis result
    pub escape_analysis: EscapeAnalysis,
}

/// Fragmentation impact on memory performance  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationImpact {
    /// Severity of fragmentation impact
    pub severity: FragmentationSeverity,
    /// Affected allocations
    pub affected_allocations: Vec<usize>,
    /// Estimated waste due to fragmentation
    pub estimated_waste: usize,
    /// Impact level of fragmentation on memory performance
    pub impact_level: ImpactLevel,
}

/// Impact level of fragmentation on memory performance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImpactLevel {
    /// Low impact - minimal effect on performance
    Low,
    /// Medium impact - noticeable effect on performance
    Medium,
    /// High impact - significant effect on performance
    High,
    /// Critical impact - severe effect on performance, requires immediate attention
    Critical,
}

/// Temporary object analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryObjectAnalysisReport {
    /// Temporary objects detected
    pub temporary_objects: Vec<EnhancedTemporaryObjectInfo>,
    /// Optimization candidates
    pub optimization_candidates: Vec<OptimizationCandidate>,
    /// Hot temporary patterns
    pub hot_temporary_patterns: Vec<HotTemporaryPattern>,
    /// Optimization suggestions
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    /// Pattern statistics
    pub pattern_statistics: PatternStatistics,
    /// Performance impact assessment
    pub performance_impact_assessment: PerformanceImpactAssessment,
}

/// Optimization candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationCandidate {
    /// Allocation information
    pub allocation: AllocationInfo,
    /// Optimization type
    pub optimization_type: OptimizationType,
    /// Expected benefit of the optimization
    pub expected_benefit: f64,
    /// Implementation difficulty of the optimization
    pub implementation_effort: ImplementationDifficulty,
}

/// Types of memory optimization strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    /// Eliminate temporary object creation entirely
    EliminateTemporary,
    /// Reuse existing allocations instead of creating new ones
    ReuseAllocation,
    /// Use memory pools for similar-sized allocations
    PoolAllocation,
    /// Initialize objects only when needed
    LazyInitialization,
    /// Avoid unnecessary copying of objects
    CopyElision,
}
/// Implementation difficulty of memory optimization strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationDifficulty {
    /// Easy implementation difficulty
    Easy,
    /// Medium implementation difficulty
    Medium,
    /// Hard implementation difficulty
    Hard,
    /// Very hard implementation difficulty
    VeryHard,
}

/// Hot temporary pattern classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotTemporaryPattern {
    /// Temporary pattern classification
    pub pattern: TemporaryPatternClassification,
    /// Frequency of the temporary pattern
    pub frequency: usize,
    /// Total memory impact of the temporary pattern
    pub total_memory_impact: usize,
    /// Optimization priority of the temporary pattern
    pub optimization_priority: Priority,
}

/// Priority levels for optimization recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    /// Low priority - minimal impact on performance or memory usage
    Low,
    /// Medium priority - moderate impact on performance or memory usage
    Medium,
    /// High priority - significant impact on performance or memory usage
    High,
    /// Critical priority - severe impact on performance or memory usage, should be addressed immediately
    Critical,
}

/// Optimization suggestion for memory management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// Category of optimization
    pub category: OptimizationCategory,
    /// Description of the optimization
    pub description: String,
    /// Code example for the optimization
    pub code_example: Option<String>,
    /// Expected improvement from the optimization
    pub expected_improvement: f64,
}

/// Categories of memory optimization techniques
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationCategory {
    /// Optimizing memory layout of data structures
    MemoryLayout,
    /// Reducing temporary object creation and lifetime
    TemporaryObjectReduction,
    /// Optimizing for cache efficiency
    CacheOptimization,
    /// Improving allocation strategy
    AllocationStrategy,
    /// Better management of object lifecycles
    LifecycleManagement,
}

/// Statistics about detected temporary object patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStatistics {
    /// Total number of patterns detected
    pub total_patterns_detected: usize,
    /// Frequency distribution of patterns
    pub pattern_frequency_distribution: HashMap<TemporaryPatternClassification, usize>,
    /// Memory impact by pattern
    pub memory_impact_by_pattern: HashMap<TemporaryPatternClassification, usize>,
}

/// Assessment of performance impact of memory management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImpactAssessment {
    /// Overhead of allocation
    pub allocation_overhead: f64,
    /// Overhead of deallocation
    pub deallocation_overhead: f64,
    /// Impact on cache performance
    pub cache_impact: f64,
    /// Overall performance cost
    pub overall_performance_cost: f64,
}

/// Real-time Fragmentation Analysis Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeFragmentationAnalysis {
    /// Current fragmentation metrics
    pub current_fragmentation: FragmentationMetrics,
    /// Trends in fragmentation over time
    pub fragmentation_trends: FragmentationTrends,
    /// Adaptive strategies for fragmentation
    pub adaptive_strategies: Vec<AdaptiveStrategy>,
    /// Real-time metrics for fragmentation
    pub real_time_metrics: RealTimeMetrics,
    /// Fragmentation visualization
    pub fragmentation_visualization: FragmentationVisualization,
    /// Mitigation recommendations for fragmentation
    pub mitigation_recommendations: Vec<FragmentationMitigationStrategy>,
}

/// Trends in fragmentation over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationTrends {
    /// Direction of fragmentation trend over time
    pub trend_direction: TrendDirection,
    /// Rate of change of fragmentation
    pub rate_of_change: f64,
    /// Predicted future state of fragmentation
    pub predicted_future_state: FragmentationPrediction,
}

/// Direction of fragmentation trend over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Fragmentation is decreasing over time
    Improving,
    /// Fragmentation is relatively constant
    Stable,
    /// Fragmentation is increasing over time
    Degrading,
    /// Fragmentation is changing unpredictably
    Volatile,
}

/// Prediction of fragmentation in the future
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationPrediction {
    /// Predicted fragmentation in 1 hour
    pub predicted_fragmentation_in_1h: f64,
    /// Predicted fragmentation in 24 hours
    pub predicted_fragmentation_in_24h: f64,
    /// Confidence level of the prediction
    pub confidence_level: f64,
}

/// Time point of fragmentation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationTimePoint {
    /// Timestamp of the fragmentation analysis
    pub timestamp: u64,
    /// Fragmentation level at the time point
    pub fragmentation_level: f64,
    /// Number of allocations at the time point
    pub allocation_count: usize,
}

/// Fragmentation heatmap data for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationHeatmapData {
    /// Memory regions for fragmentation analysis
    pub memory_regions: Vec<MemoryRegion>,
    /// Fragmentation scores for each memory region
    pub fragmentation_scores: Vec<f64>,
}

/// Memory region information for fragmentation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegion {
    /// Start address of the memory region
    pub start_address: usize,
    /// End address of the memory region
    pub end_address: usize,
    /// Fragmentation score of the memory region
    pub fragmentation_score: f64,
}

/// Types of fragmentation mitigation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MitigationType {
    /// Memory pooling strategy
    MemoryPooling,
    /// Compaction strategy
    Compaction,
    /// Size class segregation strategy
    SizeClassSegregation,
    /// Custom allocator strategy
    CustomAllocator,
}

/// Adaptive strategy for fragmentation mitigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveStrategy {
    /// Name of the adaptive strategy
    pub strategy_name: String,
    /// Trigger conditions for the adaptive strategy
    pub trigger_conditions: Vec<String>,
    /// Actions to be taken by the adaptive strategy
    pub actions: Vec<String>,
    /// Effectiveness score of the adaptive strategy
    pub effectiveness_score: f64,
}

/// Fragmentation visualization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationVisualization {
    /// Memory map of the fragmentation visualization
    pub memory_map: Vec<MemoryBlock>,
    /// Fragmentation heatmap of the fragmentation visualization
    pub fragmentation_heatmap: Vec<f64>,
    /// Allocation timeline of the fragmentation visualization
    pub allocation_timeline: Vec<AllocationEvent>,
}

/// Memory block information for fragmentation visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBlock {
    /// Start address of the memory block
    pub start_address: usize,
    /// Size of the memory block
    pub size: usize,
    /// Type of the memory block
    pub block_type: MemoryBlockType,
    /// Fragmentation score of the memory block
    pub fragmentation_score: f64,
}

/// Types of memory blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryBlockType {
    /// Allocated memory block
    Allocated,
    /// Free memory block
    Free,
    /// Reserved memory block
    Reserved,
    /// Fragmented memory block
    Fragmented,
}

/// Overall optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallOptimizationRecommendation {
    /// Category of the optimization recommendation
    pub category: OptimizationCategory,
    /// Priority of the optimization recommendation
    pub priority: Priority,
    /// Description of the optimization recommendation
    pub description: String,
    /// Expected improvement of the optimization recommendation
    pub expected_improvement: f64,
    /// Implementation effort of the optimization recommendation
    pub implementation_effort: ImplementationDifficulty,
    /// Affected components of the optimization recommendation
    pub affected_components: Vec<String>,
}

/// Additional supporting types for comprehensive analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackHeapInteractionAnalysis {
    /// Reference relationships between stack and heap allocations
    pub reference_relationships: Vec<ReferenceRelationship>,
    /// Lifetime dependencies between allocations
    pub lifetime_dependencies: Vec<LifetimeDependency>,
    /// Performance implications of stack and heap interactions
    pub performance_implications: Vec<PerformanceImplication>,
}

/// Reference relationship between stack and heap allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceRelationship {
    /// Stack allocation address
    pub stack_allocation: usize,
    /// Heap allocation address
    pub heap_allocation: usize,
    /// Type of reference relationship
    pub relationship_type: ReferenceType,
}

/// Types of reference relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceType {
    /// Direct reference relationship
    DirectReference,
    /// Indirect reference relationship
    IndirectReference,
    /// Weak reference relationship
    WeakReference,
    /// Ownership transfer relationship
    OwnershipTransfer,
}

/// Lifetime dependency between allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeDependency {
    /// Dependent allocation address
    pub dependent_allocation: usize,
    /// Dependency allocation address
    pub dependency_strength: DependencyStrength,
}

/// Dependency strength between allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyStrength {
    /// Strong dependency strength
    Strong,
    /// Weak dependency strength
    Weak,
    /// Optional dependency strength
    Optional,
}

/// Performance implication of stack and heap interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImplication {
    /// Type of performance implication
    pub implication_type: PerformanceImplicationType,
    /// Severity of the performance implication
    pub severity: Severity,
    /// Description of the performance implication
    pub description: String,
    /// Mitigation suggestion for the performance implication
    pub mitigation_suggestion: String,
}

/// Types of performance implications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceImplicationType {
    /// Cache miss performance implication
    CacheMiss,
    /// Memory latency performance implication
    MemoryLatency,
    /// Allocation overhead performance implication
    AllocationOverhead,
    /// Fragmentation impact performance implication
    MemoryOptimization,
    /// Positive performance implication
    Positive,
    /// Negative performance implication
    Negative,
    /// Neutral performance implication
    Neutral,
}

/// Severity of performance implications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Memory space coverage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySpaceCoverage {
    /// Total tracked bytes
    pub total_tracked_bytes: usize,
    /// Stack coverage percentage
    pub stack_coverage_percent: f64,
    /// Heap coverage percentage
    pub heap_coverage_percent: f64,
    /// Unknown region percentage
    pub unknown_region_percent: f64,
}

/// Boundary detection accuracy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryDetectionAccuracy {
    /// Stack detection accuracy
    pub stack_detection_accuracy: f64,
    /// Heap detection accuracy
    pub heap_detection_accuracy: f64,
    /// False positive rate
    pub false_positive_rate: f64,
    /// False negative rate
    pub false_negative_rate: f64,
}

/// Stack and heap optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackHeapOptimization {
    /// Type of stack and heap optimization
    pub optimization_type: StackHeapOptimizationType,
    /// Description of the stack and heap optimization
    pub description: String,
    /// Affected allocations
    pub affected_allocations: Vec<usize>,
    /// Expected benefit of the stack and heap optimization
    pub expected_benefit: String,
}

/// Types of stack and heap optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StackHeapOptimizationType {
    /// Stack to heap promotion
    StackToHeapPromotion,
    /// Heap to stack demotion
    HeapToStackDemotion,
    /// Allocation elimination
    AllocationElimination,
    /// Lifetime optimization
    LifetimeOptimization,
}

/// Generic type analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericTypeAnalysisReport {
    /// Instantiation analysis of generic types
    pub instantiation_analysis: Vec<crate::core::types::GenericInstantiationInfo>,
    /// Code bloat assessment of generic types
    pub code_bloat_assessment: CodeBloatAssessment,
    /// Optimization recommendations for generic types
    pub optimization_recommendations: Vec<crate::core::types::MemoryOptimizationRecommendation>,
    /// Monomorphization statistics of generic types
    pub monomorphization_statistics: crate::enhanced_memory_analysis::MonomorphizationStatistics,
    /// Performance characteristics of generic types
    pub performance_characteristics: PerformanceCharacteristics,
}

/// Object lifecycle analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectLifecycleAnalysisReport {
    /// Lifecycle reports of objects
    pub lifecycle_reports: Vec<ObjectLifecycleInfo>,
    /// Lifecycle patterns of objects
    pub lifecycle_patterns: Vec<LifecyclePattern>,
    /// Resource waste analysis of objects
    pub resource_waste_analysis: ResourceWasteAnalysis,
    /// Lifecycle optimizations of objects
    pub lifecycle_optimizations: Vec<crate::enhanced_memory_analysis::LifecycleOptimization>,
    /// Efficiency metrics of objects
    pub efficiency_metrics: crate::enhanced_memory_analysis::EfficiencyMetrics,
    /// Object relationship graph of objects
    pub object_relationship_graph: crate::enhanced_memory_analysis::ObjectRelationshipGraph,
}

/// Memory access analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAccessAnalysisReport {
    /// Access patterns of memory
    pub access_patterns: Vec<AccessPattern>,
    /// Layout recommendations of memory
    pub layout_recommendations: Vec<crate::enhanced_memory_analysis::LayoutRecommendation>,
    /// Actual access tracking of memory
    pub actual_access_tracking: crate::enhanced_memory_analysis::ActualAccessTracking,
    /// Bandwidth utilization of memory
    pub bandwidth_utilization: crate::enhanced_memory_analysis::BandwidthUtilization,
    /// Locality analysis of memory
    pub locality_analysis: crate::enhanced_memory_analysis::LocalityAnalysis,
}

/// Cache optimization report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheOptimizationReport {
    /// Cache line analysis of memory
    pub cache_line_analysis: crate::enhanced_memory_analysis::CacheLineAnalysis,
    /// Data structure optimizations of memory
    pub data_structure_optimizations:
        Vec<crate::enhanced_memory_analysis::DataStructureOptimization>,
    /// Access pattern optimizations of memory
    pub access_pattern_optimizations:
        Vec<crate::enhanced_memory_analysis::AccessPatternOptimization>,
    /// Cache efficiency metrics of memory
    pub cache_efficiency_metrics: LifecycleEfficiencyMetrics,
    /// Optimization recommendations of memory
    pub optimization_recommendations: Vec<OptimizationRecommendation>,
    /// Performance projections of memory
    pub performance_projections: PerformanceImplication,
}

/// Enhanced memory analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMemoryAnalysisReport {
    /// Timestamp of the analysis
    pub timestamp: u64,
    /// Analysis duration in milliseconds
    pub analysis_duration_ms: u64,
    /// Stack and heap boundary analysis
    pub stack_heap_analysis: StackHeapBoundaryAnalysis,
    /// Temporary object analysis report
    pub temp_object_analysis: TemporaryObjectAnalysisReport,
    /// Fragmentation analysis report
    pub fragmentation_analysis: RealTimeFragmentationAnalysis,
    /// Generic type analysis report
    pub generic_analysis: GenericTypeAnalysisReport,
    /// Object lifecycle analysis report
    pub lifecycle_analysis: ObjectLifecycleAnalysisReport,
    /// Memory access analysis report
    pub access_pattern_analysis: MemoryAccessAnalysisReport,
    /// Cache optimization report
    pub cache_optimization: CacheOptimizationReport,
    /// Overall optimization recommendations
    pub overall_recommendations: Vec<OverallOptimizationRecommendation>,
}

/// Stack heap boundary analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackHeapBoundaryAnalysis {
    /// Stack allocations of memory
    pub stack_allocations: Vec<StackAllocationDetails>,
    /// Heap allocations of memory
    pub heap_allocations: Vec<HeapAllocationDetails>,
    /// Ambiguous allocations of memory
    pub ambiguous_allocations: Vec<AmbiguousAllocation>,
    /// Stack and heap interactions of memory
    pub stack_heap_interactions: StackHeapInteractionAnalysis,
    /// Memory space coverage of memory
    pub memory_space_coverage: MemorySpaceCoverage,
    /// Boundary detection accuracy of memory
    pub boundary_detection_accuracy: BoundaryDetectionAccuracy,
    /// Optimization opportunities of memory
    pub optimization_opportunities: Vec<StackHeapOptimization>,
}

/// Code bloat assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBloatAssessment {
    /// Bloat level of the code
    pub bloat_level: BloatLevel,
    /// Estimated code size increase
    pub estimated_code_size_increase: f64,
    /// Compilation time impact
    pub compilation_time_impact: f64,
    /// Binary size impact
    pub binary_size_impact: f64,
}

/// Bloat level of the code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BloatLevel {
    /// Minimal bloat level
    Minimal,
    /// Low bloat level
    Low,
    /// Moderate bloat level
    Moderate,
    /// High bloat level
    High,
    /// Severe bloat level
    Severe,
}

/// Resource waste analysis of memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceWasteAnalysis {
    /// Wasted allocations of memory
    pub wasted_allocations: usize,
    /// Total wasted memory of memory
    pub total_wasted_memory: usize,
    /// Waste percentage of memory
    pub waste_percentage: f64,
    /// Waste categories of memory
    pub waste_categories: Vec<WasteCategory>,
}

/// Waste category of memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasteCategory {
    /// Waste category type
    pub category_type: WasteCategoryType,
    /// Wasted bytes of memory
    pub wasted_bytes: usize,
    /// Frequency of memory
    pub frequency: usize,
}

/// Waste category type of memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasteCategoryType {
    /// Unused allocations of memory
    UnusedAllocations,
    /// Over allocations of memory
    OverAllocations,
    /// Leaked memory of memory
    LeakedMemory,
    /// Fragmentation waste of memory
    FragmentationWaste,
    /// Temporary object waste of memory
    TemporaryObjectWaste,
}

/// Additional supporting types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryLocation {
    /// Stack allocation of memory
    Stack(crate::enhanced_memory_analysis::StackFrameInfo),
    /// Heap allocation of memory
    Heap(HeapRegionInfo),
    /// Ambiguous allocation of memory
    Ambiguous(AmbiguityReason),
}

/// Reasons why memory allocation tracking might be ambiguous or uncertain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AmbiguityReason {
    /// Insufficient metadata available to accurately track the allocation
    InsufficientMetadata,
    /// Address is at the border of tracked memory regions
    BorderlineAddress,
    /// Memory tracking data has been corrupted
    CorruptedTracking,
    /// Allocation was made by an external system not fully tracked by this tool
    ExternalAllocation,
}

/// Information about a heap memory region managed by an allocator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapRegionInfo {
    /// Starting address of the heap region
    pub region_start: usize,
    /// Ending address of the heap region
    pub region_end: usize,
    /// Name of the allocator managing this heap region
    pub allocator_name: String,
    /// Type of heap region (main heap, large object heap, etc.)
    pub region_type: HeapRegionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Types of heap regions managed by memory allocators
pub enum HeapRegionType {
    /// Main heap area for general allocations
    MainHeap,
    /// Specialized heap area for large object allocations
    LargeObjectHeap,
    /// Specialized heap area for small object allocations
    SmallObjectHeap,
    /// Thread-local heap areas
    ThreadLocalHeap,
}

// Fragmentation analysis types
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Comprehensive analysis of memory fragmentation with metrics, causes, and recommendations
pub struct EnhancedFragmentationAnalysis {
    /// Metrics quantifying different aspects of memory fragmentation
    pub fragmentation_metrics: FragmentationMetrics,
    /// Assessment of fragmentation severity level
    pub fragmentation_severity: FragmentationSeverity,
    /// Identified causes contributing to memory fragmentation
    pub fragmentation_causes: Vec<FragmentationCause>,
    /// Strategies to mitigate the identified fragmentation issues
    pub mitigation_strategies: Vec<FragmentationMitigationStrategy>,
    /// Analysis of fragmentation trends over time
    pub fragmentation_trends: FragmentationTrends,
    /// Real-time monitoring data for ongoing fragmentation analysis
    pub real_time_monitoring: crate::enhanced_memory_analysis::RealTimeMonitoringData,
    /// Adaptive recommendations based on current fragmentation patterns
    pub adaptive_recommendations: Vec<crate::enhanced_memory_analysis::AdaptiveRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Quantitative metrics measuring different aspects of memory fragmentation
pub struct FragmentationMetrics {
    /// Ratio of external fragmentation (unusable gaps between allocations)
    pub external_fragmentation_ratio: f64,
    /// Ratio of internal fragmentation (unused space within allocations)
    pub internal_fragmentation_ratio: f64,
    /// Combined ratio of all fragmentation types
    pub total_fragmentation_ratio: f64,
    /// Size of the largest contiguous free memory block
    pub largest_free_block: usize,
    /// Total number of free memory blocks
    pub free_block_count: usize,
    /// Average size of free memory blocks
    pub average_free_block_size: f64,
    /// Ratio of memory actually used vs. total allocated
    pub memory_utilization_ratio: f64,
}

/// Severity levels of memory fragmentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FragmentationSeverity {
    /// Low severity - minimal fragmentation, little impact on performance
    Low,
    /// Moderate severity - noticeable fragmentation, some impact on performance
    Moderate,
    /// High severity - significant fragmentation, considerable impact on performance
    High,
    /// Critical severity - severe fragmentation, major impact on performance
    Critical,
}

// Additional missing types for enhanced memory analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a potential optimization for temporary memory usage patterns
pub struct TemporaryOptimization {
    /// Type of optimization strategy to apply
    pub optimization_type: OptimizationType,
    /// Expected performance or memory benefit as a ratio (higher is better)
    pub expected_benefit: f64,
    /// Estimated difficulty of implementing this optimization
    pub implementation_effort: ImplementationDifficulty,
    /// Detailed description of the optimization approach
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Statistical analysis of temporary memory allocation patterns
pub struct TemporaryPatternStatistics {
    /// Total number of temporary allocation patterns identified
    pub total_patterns: usize,
    /// Distribution of different pattern classifications and their frequencies
    pub pattern_distribution: HashMap<TemporaryPatternClassification, usize>,
    /// Memory impact (in bytes) of each pattern classification
    pub memory_impact: HashMap<TemporaryPatternClassification, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Analysis of performance impacts caused by temporary object allocations
pub struct TemporaryObjectPerformanceImpact {
    /// Computational overhead of allocation operations (normalized ratio)
    pub allocation_overhead: f64,
    /// Computational overhead of deallocation operations (normalized ratio)
    pub deallocation_overhead: f64,
    /// Impact on CPU cache efficiency (normalized ratio, lower is better)
    pub cache_impact: f64,
    /// Combined performance cost metric (normalized ratio, lower is better)
    pub overall_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Strategies for optimizing memory usage patterns
pub enum OptimizationStrategy {
    /// Completely eliminate the temporary allocation
    Eliminate,
    /// Reuse existing objects instead of creating new ones
    Reuse,
    /// Use object pooling to reduce allocation/deallocation overhead
    Pool,
    /// Defer allocation until absolutely necessary
    Defer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Analysis of memory fragmentation trends over time
pub struct FragmentationTrendAnalysis {
    /// Historical fragmentation analysis data points
    pub historical_data: Vec<FragmentationAnalysis>,
    /// Direction of the fragmentation trend (increasing, decreasing, stable)
    pub trend_direction: TrendDirection,
    /// Projected future fragmentation levels
    pub projected_levels: Vec<FragmentationProjection>,
    /// Historical fragmentation level measurements as raw values
    pub fragmentation_levels: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Strategy that adapts to changing fragmentation conditions
pub struct AdaptiveFragmentationStrategy {
    /// Name of the adaptive strategy
    pub strategy_name: String,
    /// Conditions that trigger this strategy to be applied
    pub trigger_conditions: Vec<String>,
    /// Actions to take when the strategy is triggered
    pub actions: Vec<String>,
    /// Estimated effectiveness score of this strategy (0.0-1.0)
    pub effectiveness_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Real-time metrics for monitoring memory fragmentation
pub struct RealTimeFragmentationMetrics {
    /// Current fragmentation ratio (0.0-1.0, higher means more fragmented)
    pub current_fragmentation: f64,
    /// Rate of memory allocations per second
    pub allocation_rate: f64,
    /// Rate of memory deallocations per second
    pub deallocation_rate: f64,
    /// Current memory pressure (0.0-1.0, higher means more pressure)
    pub memory_pressure: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Data for visualizing memory fragmentation patterns
pub struct FragmentationVisualizationData {
    /// Map of memory blocks showing allocated and free regions
    pub memory_map: Vec<MemoryBlock>,
    /// Heatmap data showing fragmentation intensity across memory regions
    pub fragmentation_heatmap: Vec<f64>,
    /// Timeline of allocation events for temporal visualization
    pub allocation_timeline: Vec<AllocationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Recommendation for mitigating memory fragmentation issues
pub struct FragmentationMitigationRecommendation {
    /// Type of mitigation strategy being recommended
    pub strategy_type: MitigationStrategyType,
    /// Detailed description of the recommendation
    pub description: String,
    /// Expected improvement in fragmentation metrics (0.0-1.0)
    pub expected_improvement: f64,
    /// Estimated complexity of implementing this recommendation
    pub implementation_complexity: ImplementationComplexity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Analysis of trends in memory usage or fragmentation metrics
pub struct TrendAnalysis {
    /// Direction of the trend (increasing, decreasing, stable)
    pub direction: TrendDirection,
    /// Rate of change per time unit
    pub rate_of_change: f64,
    /// Confidence level in the trend analysis (0.0-1.0)
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a cyclical pattern in memory usage or fragmentation
pub struct CyclicalPattern {
    /// Name or identifier for this pattern
    pub pattern_name: String,
    /// Duration of one complete cycle
    pub cycle_duration: Duration,
    /// Amplitude of the cycle (magnitude of change)
    pub amplitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents an anomaly detected in memory fragmentation patterns
pub struct FragmentationAnomaly {
    /// Type or category of the anomaly
    pub anomaly_type: String,
    /// Severity of the anomaly (0.0-1.0, higher is more severe)
    pub severity: f64,
    /// Timestamp when the anomaly was detected
    pub timestamp: u64,
    /// Detailed description of the anomaly
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Projection of future memory fragmentation levels
pub struct FragmentationProjection {
    /// Time horizon for the projection in hours
    pub time_horizon_hours: u32,
    /// Projected fragmentation ratio at the specified time horizon
    pub projected_fragmentation: f64,
    /// Confidence level in the projection (0.0-1.0)
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Impact of memory usage patterns on system scalability
pub enum ScalabilityImpact {
    /// Positive impact on scalability (improves as scale increases)
    Positive,
    /// Neutral impact on scalability (neither improves nor worsens with scale)
    Neutral,
    /// Negative impact on scalability (worsens as scale increases)
    Negative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Strategy that adapts to mitigate memory fragmentation based on thresholds
pub struct AdaptiveMitigationStrategy {
    /// Name of the adaptive mitigation strategy
    pub strategy_name: String,
    /// Threshold value that triggers this strategy (typically a fragmentation ratio)
    pub trigger_threshold: f64,
    /// Actions to take when the strategy is triggered
    pub actions: Vec<String>,
    /// Expected effectiveness of this strategy (0.0-1.0)
    pub expected_effectiveness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Current state of memory fragmentation
pub struct CurrentFragmentationState {
    /// Current external fragmentation ratio (0.0-1.0)
    pub external_fragmentation: f64,
    /// Current internal fragmentation ratio (0.0-1.0)
    pub internal_fragmentation: f64,
    /// Assessment of the current fragmentation severity
    pub severity_level: FragmentationSeverity,
    /// Timestamp when this state was captured
    pub timestamp: u64,
}
