//! Enhanced types for comprehensive memory analysis
//! 
//! This module contains additional type definitions needed for the enhanced memory analysis system

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::Duration;
use crate::types::*;

// Stack Frame and Boundary Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub frame_id: u64,
    pub function_name: String,
    pub allocations: Vec<usize>,
    pub total_allocated: usize,
    pub frame_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackBoundaries {
    pub stack_base: usize,
    pub stack_top: usize,
    pub stack_size: usize,
}

impl StackBoundaries {
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
    
    pub fn contains(&self, ptr: usize) -> bool {
        ptr >= self.stack_base && ptr < self.stack_top
    }
    
    pub fn get_frame_base(&self, frame_id: u64) -> usize {
        // Estimate frame base from frame ID
        self.stack_base + (frame_id as usize * 4096)
    }
}

// Heap Boundary Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapSegment {
    pub start: usize,
    pub end: usize,
}

impl HeapSegment {
    pub fn contains(&self, ptr: usize) -> bool {
        ptr >= self.start && ptr < self.end
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatorInfo {
    pub name: String,
    pub strategy: AllocationStrategy,
    pub heap_segments: Vec<HeapSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationStrategy {
    FirstFit,
    BestFit,
    WorstFit,
    NextFit,
    BuddySystem,
    SlabAllocation,
}

// Enhanced Temporary Object Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTemporaryObjectInfo {
    pub allocation: AllocationInfo,
    pub pattern_classification: TemporaryPatternClassification,
    pub usage_pattern: TemporaryUsagePattern,
    pub hot_path_involvement: bool,
    pub elimination_feasibility: EliminationFeasibility,
    pub optimization_potential: OptimizationPotential,
    pub creation_context: CreationContext,
    pub lifetime_analysis: TemporaryLifetimeAnalysis,
    pub performance_impact: PerformanceImpact,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemporaryPatternClassification {
    StringConcatenation,
    VectorReallocation,
    IteratorChaining,
    ClosureCapture,
    AsyncAwait,
    ErrorHandling,
    SerializationDeserialization,
    GenericInstantiation,
    TraitObjectCreation,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EliminationFeasibility {
    HighlyFeasible { suggested_approach: String },
    Feasible { constraints: Vec<String> },
    Difficult { blockers: Vec<String> },
    Infeasible { reasons: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryLifetimeAnalysis {
    pub creation_time: u64,
    pub destruction_time: Option<u64>,
    pub estimated_lifetime: Duration,
    pub usage_frequency: usize,
    pub scope_escape_analysis: EscapeAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscapeAnalysis {
    DoesNotEscape,
    EscapesToHeap,
    EscapesToCaller,
    EscapesToGlobal,
    Unknown,
}

// Fragmentation Analysis Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationEvent {
    pub timestamp: u64,
    pub event_type: AllocationEventType,
    pub ptr: usize,
    pub size: usize,
    pub type_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AllocationEventType {
    Allocate,
    Deallocate,
    Reallocate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeMetrics {
    pub current_fragmentation: f64,
    pub allocation_rate: f64,
    pub deallocation_rate: f64,
    pub memory_pressure: f64,
}

impl RealTimeMetrics {
    pub fn new() -> Self {
        Self {
            current_fragmentation: 0.0,
            allocation_rate: 0.0,
            deallocation_rate: 0.0,
            memory_pressure: 0.0,
        }
    }
    
    pub fn update_allocation(&mut self, allocation: &AllocationInfo) {
        // Update metrics based on new allocation
        self.allocation_rate += 1.0;
        // Additional metric updates would go here
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FragmentationCause {
    FrequentSmallAllocations,
    MixedSizeAllocations,
    LongLivedAllocations,
    PoorDeallocationPatterns,
    AllocatorLimitations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationMitigationStrategy {
    pub strategy_type: MitigationStrategyType,
    pub description: String,
    pub expected_improvement: f64,
    pub implementation_complexity: ImplementationComplexity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MitigationStrategyType {
    PoolAllocation,
    SizeClassSegregation,
    GenerationalGC,
    CompactionGC,
    CustomAllocator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationComplexity {
    Low,
    Medium,
    High,
    VeryHigh,
}

// Stack/Heap Analysis Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackAllocationDetails {
    pub allocation: AllocationInfo,
    pub frame_info: crate::types::StackFrame,
    pub stack_depth: usize,
    pub scope_analysis: StackScopeAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapAllocationDetails {
    pub allocation: AllocationInfo,
    pub heap_info: HeapRegionInfo,
    pub allocator_type: String,
    pub fragmentation_impact: FragmentationImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguousAllocation {
    pub allocation: AllocationInfo,
    pub ambiguity_reason: AmbiguityReason,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackScopeAnalysis {
    pub scope_type: ScopeType,
    pub nesting_level: usize,
    pub estimated_lifetime: Duration,
    pub escape_analysis: EscapeAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationImpact {
    pub severity: FragmentationSeverity,
    pub affected_allocations: Vec<usize>,
    pub estimated_waste: usize,
}

// Additional Analysis Report Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryObjectAnalysisReport {
    pub temporary_objects: Vec<EnhancedTemporaryObjectInfo>,
    pub optimization_candidates: Vec<OptimizationCandidate>,
    pub hot_temporary_patterns: Vec<HotTemporaryPattern>,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    pub pattern_statistics: PatternStatistics,
    pub performance_impact_assessment: PerformanceImpactAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationCandidate {
    pub allocation: AllocationInfo,
    pub optimization_type: OptimizationType,
    pub expected_benefit: f64,
    pub implementation_effort: ImplementationDifficulty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    EliminateTemporary,
    ReuseAllocation,
    PoolAllocation,
    LazyInitialization,
    CopyElision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationDifficulty {
    Easy,
    Medium,
    Hard,
    VeryHard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotTemporaryPattern {
    pub pattern: TemporaryPatternClassification,
    pub frequency: usize,
    pub total_memory_impact: usize,
    pub optimization_priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub category: OptimizationCategory,
    pub description: String,
    pub code_example: Option<String>,
    pub expected_improvement: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationCategory {
    MemoryLayout,
    TemporaryObjectReduction,
    CacheOptimization,
    AllocationStrategy,
    LifecycleManagement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStatistics {
    pub total_patterns_detected: usize,
    pub pattern_frequency_distribution: HashMap<TemporaryPatternClassification, usize>,
    pub memory_impact_by_pattern: HashMap<TemporaryPatternClassification, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImpactAssessment {
    pub allocation_overhead: f64,
    pub deallocation_overhead: f64,
    pub cache_impact: f64,
    pub overall_performance_cost: f64,
}

// Real-time Fragmentation Analysis Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeFragmentationAnalysis {
    pub current_fragmentation: FragmentationMetrics,
    pub fragmentation_trends: FragmentationTrends,
    pub adaptive_strategies: Vec<AdaptiveStrategy>,
    pub real_time_metrics: RealTimeMetrics,
    pub fragmentation_visualization: FragmentationVisualization,
    pub mitigation_recommendations: Vec<FragmentationMitigationStrategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationTrends {
    pub trend_direction: TrendDirection,
    pub rate_of_change: f64,
    pub predicted_future_state: FragmentationPrediction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationPrediction {
    pub predicted_fragmentation_in_1h: f64,
    pub predicted_fragmentation_in_24h: f64,
    pub confidence_level: f64,
}

// Additional missing types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationTimePoint {
    pub timestamp: u64,
    pub fragmentation_level: f64,
    pub allocation_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationHeatmapData {
    pub memory_regions: Vec<MemoryRegion>,
    pub fragmentation_scores: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegion {
    pub start_address: usize,
    pub end_address: usize,
    pub fragmentation_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MitigationType {
    MemoryPooling,
    Compaction,
    SizeClassSegregation,
    CustomAllocator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveStrategy {
    pub strategy_name: String,
    pub trigger_conditions: Vec<String>,
    pub actions: Vec<String>,
    pub effectiveness_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationVisualization {
    pub memory_map: Vec<MemoryBlock>,
    pub fragmentation_heatmap: Vec<f64>,
    pub allocation_timeline: Vec<AllocationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBlock {
    pub start_address: usize,
    pub size: usize,
    pub block_type: MemoryBlockType,
    pub fragmentation_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryBlockType {
    Allocated,
    Free,
    Reserved,
    Fragmented,
}

// Overall Optimization Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallOptimizationRecommendation {
    pub category: OptimizationCategory,
    pub priority: Priority,
    pub description: String,
    pub expected_improvement: f64,
    pub implementation_effort: ImplementationDifficulty,
    pub affected_components: Vec<String>,
}

// Additional supporting types for comprehensive analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackHeapInteractionAnalysis {
    pub reference_relationships: Vec<ReferenceRelationship>,
    pub lifetime_dependencies: Vec<LifetimeDependency>,
    pub performance_implications: Vec<PerformanceImplication>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceRelationship {
    pub stack_allocation: usize,
    pub heap_allocation: usize,
    pub relationship_type: ReferenceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceType {
    DirectReference,
    IndirectReference,
    WeakReference,
    OwnershipTransfer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeDependency {
    pub dependent_allocation: usize,
    pub dependency_allocation: usize,
    pub dependency_strength: DependencyStrength,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyStrength {
    Strong,
    Weak,
    Optional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImplication {
    pub implication_type: PerformanceImplicationType,
    pub severity: Severity,
    pub description: String,
    pub mitigation_suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceImplicationType {
    CacheMiss,
    MemoryLatency,
    AllocationOverhead,
    FragmentationImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySpaceCoverage {
    pub total_tracked_bytes: usize,
    pub stack_coverage_percent: f64,
    pub heap_coverage_percent: f64,
    pub unknown_region_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryDetectionAccuracy {
    pub stack_detection_accuracy: f64,
    pub heap_detection_accuracy: f64,
    pub false_positive_rate: f64,
    pub false_negative_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackHeapOptimization {
    pub optimization_type: StackHeapOptimizationType,
    pub description: String,
    pub affected_allocations: Vec<usize>,
    pub expected_benefit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StackHeapOptimizationType {
    StackToHeapPromotion,
    HeapToStackDemotion,
    AllocationElimination,
    LifetimeOptimization,
}

// Missing report types for enhanced memory analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericTypeAnalysisReport {
    pub instantiation_analysis: Vec<crate::types::GenericInstantiationInfo>,
    pub code_bloat_assessment: CodeBloatAssessment,
    pub optimization_recommendations: Vec<crate::types::MemoryOptimizationRecommendation>,
    pub monomorphization_statistics: MonomorphizationStatistics,
    pub performance_characteristics: PerformanceCharacteristics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectLifecycleAnalysisReport {
    pub lifecycle_reports: Vec<ObjectLifecycle>,
    pub lifecycle_patterns: Vec<LifecyclePattern>,
    pub resource_waste_analysis: ResourceWasteAnalysis,
    pub lifecycle_optimizations: Vec<LifecycleOptimization>,
    pub efficiency_metrics: EfficiencyMetrics,
    pub object_relationship_graph: ObjectRelationshipGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAccessAnalysisReport {
    pub access_patterns: Vec<AccessPattern>,
    pub layout_recommendations: Vec<LayoutRecommendation>,
    pub actual_access_tracking: ActualAccessTracking,
    pub bandwidth_utilization: BandwidthUtilization,
    pub locality_analysis: LocalityAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheOptimizationReport {
    pub cache_line_analysis: CacheLineAnalysis,
    pub data_structure_optimizations: Vec<DataStructureOptimization>,
    pub access_pattern_optimizations: Vec<AccessPatternOptimization>,
    pub cache_efficiency_metrics: CacheEfficiencyMetrics,
    pub optimization_recommendations: Vec<CacheOptimizationRecommendation>,
    pub performance_projections: PerformanceProjections,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMemoryAnalysisReport {
    pub timestamp: u64,
    pub analysis_duration_ms: u64,
    pub stack_heap_analysis: StackHeapBoundaryAnalysis,
    pub temp_object_analysis: TemporaryObjectAnalysisReport,
    pub fragmentation_analysis: RealTimeFragmentationAnalysis,
    pub generic_analysis: GenericTypeAnalysisReport,
    pub lifecycle_analysis: ObjectLifecycleAnalysisReport,
    pub access_pattern_analysis: MemoryAccessAnalysisReport,
    pub cache_optimization: CacheOptimizationReport,
    pub overall_recommendations: Vec<OverallOptimizationRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackHeapBoundaryAnalysis {
    pub stack_allocations: Vec<StackAllocationDetails>,
    pub heap_allocations: Vec<HeapAllocationDetails>,
    pub ambiguous_allocations: Vec<AmbiguousAllocation>,
    pub stack_heap_interactions: StackHeapInteractionAnalysis,
    pub memory_space_coverage: MemorySpaceCoverage,
    pub boundary_detection_accuracy: BoundaryDetectionAccuracy,
    pub optimization_opportunities: Vec<StackHeapOptimization>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBloatAssessment {
    pub bloat_level: BloatLevel,
    pub estimated_code_size_increase: f64,
    pub compilation_time_impact: f64,
    pub binary_size_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BloatLevel {
    Minimal,
    Low,
    Moderate,
    High,
    Severe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceWasteAnalysis {
    pub wasted_allocations: usize,
    pub total_wasted_memory: usize,
    pub waste_percentage: f64,
    pub waste_categories: Vec<WasteCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasteCategory {
    pub category_type: WasteCategoryType,
    pub wasted_bytes: usize,
    pub frequency: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasteCategoryType {
    UnusedAllocations,
    OverAllocations,
    LeakedMemory,
    FragmentationWaste,
    TemporaryObjectWaste,
}

// Additional supporting types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryLocation {
    Stack(StackFrameInfo),
    Heap(HeapRegionInfo),
    Ambiguous(AmbiguityReason),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AmbiguityReason {
    InsufficientMetadata,
    BorderlineAddress,
    CorruptedTracking,
    ExternalAllocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapRegionInfo {
    pub region_start: usize,
    pub region_end: usize,
    pub allocator_name: String,
    pub region_type: HeapRegionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HeapRegionType {
    MainHeap,
    LargeObjectHeap,
    SmallObjectHeap,
    ThreadLocalHeap,
}

// Fragmentation analysis types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedFragmentationAnalysis {
    pub fragmentation_metrics: FragmentationMetrics,
    pub fragmentation_severity: FragmentationSeverity,
    pub fragmentation_causes: Vec<FragmentationCause>,
    pub mitigation_strategies: Vec<FragmentationMitigationStrategy>,
    pub fragmentation_trends: FragmentationTrends,
    pub real_time_monitoring: RealTimeMonitoringData,
    pub adaptive_recommendations: Vec<AdaptiveRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationMetrics {
    pub external_fragmentation_ratio: f64,
    pub internal_fragmentation_ratio: f64,
    pub total_fragmentation_ratio: f64,
    pub largest_free_block: usize,
    pub free_block_count: usize,
    pub average_free_block_size: f64,
    pub memory_utilization_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FragmentationSeverity {
    Low,
    Moderate,
    High,
    Critical,
}

// Additional missing types for enhanced memory analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryOptimization {
    pub optimization_type: OptimizationType,
    pub expected_benefit: f64,
    pub implementation_effort: ImplementationDifficulty,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryPatternStatistics {
    pub total_patterns: usize,
    pub pattern_distribution: HashMap<TemporaryPatternClassification, usize>,
    pub memory_impact: HashMap<TemporaryPatternClassification, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryObjectPerformanceImpact {
    pub allocation_overhead: f64,
    pub deallocation_overhead: f64,
    pub cache_impact: f64,
    pub overall_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationStrategy {
    Eliminate,
    Reuse,
    Pool,
    Defer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationTrendAnalysis {
    pub historical_data: Vec<FragmentationSnapshot>,
    pub trend_direction: TrendDirection,
    pub projected_levels: Vec<FragmentationProjection>,
    pub fragmentation_levels: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveFragmentationStrategy {
    pub strategy_name: String,
    pub trigger_conditions: Vec<String>,
    pub actions: Vec<String>,
    pub effectiveness_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeFragmentationMetrics {
    pub current_fragmentation: f64,
    pub allocation_rate: f64,
    pub deallocation_rate: f64,
    pub memory_pressure: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationVisualizationData {
    pub memory_map: Vec<MemoryBlock>,
    pub fragmentation_heatmap: Vec<f64>,
    pub allocation_timeline: Vec<AllocationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationMitigationRecommendation {
    pub strategy_type: MitigationStrategyType,
    pub description: String,
    pub expected_improvement: f64,
    pub implementation_complexity: ImplementationComplexity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub direction: TrendDirection,
    pub rate_of_change: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyclicalPattern {
    pub pattern_name: String,
    pub cycle_duration: Duration,
    pub amplitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationAnomaly {
    pub anomaly_type: String,
    pub severity: f64,
    pub timestamp: u64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationProjection {
    pub time_horizon_hours: u32,
    pub projected_fragmentation: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveMitigationStrategy {
    pub strategy_name: String,
    pub trigger_threshold: f64,
    pub actions: Vec<String>,
    pub expected_effectiveness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentFragmentationState {
    pub external_fragmentation: f64,
    pub internal_fragmentation: f64,
    pub severity_level: FragmentationSeverity,
    pub timestamp: u64,
}

