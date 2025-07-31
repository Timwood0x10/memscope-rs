//! Analysis module adapters for data collection
//!
//! This module provides adapter functions to integrate with existing analysis
//! modules and convert their results to the unified data format.

use crate::tracker::MemoryTracker;
use crate::core::types::TrackingResult;
use std::collections::HashMap;
use std::time::Duration;

/// Adapter for lifecycle analysis
pub fn analyze_memory_lifecycle(tracker: &MemoryTracker) -> TrackingResult<LifecycleAnalysisResult> {
    use crate::analysis::lifecycle_analysis;
    
    // Get the global analyzer and extract data
    let analyzer = lifecycle_analysis::get_global_lifecycle_analyzer();
    
    // Try to get allocation data from tracker
    let allocations = match tracker.get_all_allocations() {
        Ok(allocs) => allocs,
        Err(_) => Vec::new(),
    };
    
    // Detect RAII patterns
    let raii_patterns = analyzer.detect_raii_patterns(&allocations);
    
    // Extract patterns and statistics
    let mut patterns = Vec::new();
    let mut lifetime_stats = HashMap::new();
    
    for pattern in raii_patterns {
        patterns.push(pattern.pattern_type);
        lifetime_stats.insert(pattern.resource_type, pattern.average_lifetime_ms);
    }
    
    Ok(LifecycleAnalysisResult {
        patterns: Some(patterns),
        lifetime_stats: Some(lifetime_stats),
    })
}

/// Adapter for leak detection
pub fn detect_potential_leaks(tracker: &MemoryTracker) -> TrackingResult<Vec<LeakCandidateData>> {
    use crate::analysis::enhanced_memory_analysis;
    
    // Get memory stats for analysis
    let stats = tracker.get_memory_stats()?;
    let allocations = tracker.get_all_allocations().unwrap_or_default();
    
    let mut leak_candidates = Vec::new();
    
    // Simple heuristic: allocations that are very old and large
    for (i, alloc) in allocations.iter().enumerate() {
        let age = Duration::from_millis(100); // Placeholder age calculation
        let confidence = if alloc.size > 1024 * 1024 { 0.8 } else { 0.3 }; // Large allocations more suspicious
        
        if age.as_millis() > 10000 && confidence > 0.5 { // Older than 10 seconds
            leak_candidates.push(LeakCandidateData {
                allocation_id: i as u64,
                size: alloc.size,
                age,
                confidence,
            });
        }
    }
    
    Ok(leak_candidates)
}

/// Adapter for unsafe operations analysis
pub fn analyze_unsafe_operations(tracker: &MemoryTracker) -> TrackingResult<Vec<UnsafeOperationData>> {
    use crate::analysis::unsafe_ffi_tracker;
    
    // Get the global tracker and extract unsafe operations
    let unsafe_tracker = unsafe_ffi_tracker::get_global_unsafe_tracker();
    let operations = unsafe_tracker.get_all_operations();
    
    let mut unsafe_ops = Vec::new();
    
    for op in operations {
        unsafe_ops.push(UnsafeOperationData {
            operation_type: op.operation_type,
            location: op.location,
            risk_level: op.risk_level,
        });
    }
    
    Ok(unsafe_ops)
}

/// Adapter for circular reference detection
pub fn detect_circular_references(tracker: &MemoryTracker) -> TrackingResult<CircularReferenceData> {
    use crate::analysis::circular_reference;
    
    // Run circular reference detection
    let analysis_result = circular_reference::analyze_references(tracker)?;
    
    Ok(CircularReferenceData {
        cycles: analysis_result.cycles,
        reference_graph: analysis_result.reference_graph,
    })
}

/// Adapter for generic analysis
pub fn analyze_generic_usage(tracker: &MemoryTracker) -> TrackingResult<GenericAnalysisData> {
    use crate::analysis::generic_analysis;
    
    // Get the global analyzer
    let analyzer = generic_analysis::get_global_generic_analyzer();
    let analysis = analyzer.analyze_generic_instantiations();
    
    Ok(GenericAnalysisData {
        instantiations: analysis.instantiation_counts,
        monomorphization_impact: analysis.monomorphization_impact,
    })
}

/// Adapter for async analysis
pub fn analyze_async_patterns(tracker: &MemoryTracker) -> TrackingResult<AsyncAnalysisData> {
    use crate::analysis::async_analysis;
    
    // Get the global async analyzer
    let analyzer = async_analysis::get_global_async_analyzer();
    let patterns = analyzer.get_async_patterns();
    
    let mut async_allocations = Vec::new();
    let mut future_memory_usage = HashMap::new();
    
    for pattern in patterns {
        if let Some(alloc) = pattern.allocation_info {
            async_allocations.push(AsyncAllocationData {
                future_id: pattern.future_id,
                size: alloc.size,
                state: pattern.state,
            });
            
            future_memory_usage.insert(pattern.future_type, alloc.size as u64);
        }
    }
    
    Ok(AsyncAnalysisData {
        async_allocations,
        future_memory_usage,
    })
}

/// Adapter for borrow analysis
pub fn analyze_borrow_patterns(tracker: &MemoryTracker) -> TrackingResult<BorrowAnalysisData> {
    use crate::analysis::borrow_analysis;
    
    // Get the global borrow analyzer
    let analyzer = borrow_analysis::get_global_borrow_analyzer();
    let conflicts = analyzer.get_borrow_conflicts();
    
    let mut violations = Vec::new();
    let mut lifetime_conflicts = HashMap::new();
    
    for conflict in conflicts {
        violations.push(BorrowViolationData {
            violation_type: conflict.conflict_type,
            location: conflict.location,
            severity: conflict.severity,
        });
        
        lifetime_conflicts.insert(conflict.variable_name, vec![conflict.conflicting_borrow]);
    }
    
    Ok(BorrowAnalysisData {
        violations,
        lifetime_conflicts,
    })
}

// Data structures for analysis results

#[derive(Debug)]
pub struct LifecycleAnalysisResult {
    pub patterns: Option<Vec<String>>,
    pub lifetime_stats: Option<HashMap<String, u64>>,
}

#[derive(Debug)]
pub struct LeakCandidateData {
    pub allocation_id: u64,
    pub size: usize,
    pub age: Duration,
    pub confidence: f64,
}

#[derive(Debug)]
pub struct UnsafeOperationData {
    pub operation_type: String,
    pub location: String,
    pub risk_level: String,
}

#[derive(Debug)]
pub struct CircularReferenceData {
    pub cycles: Vec<CycleData>,
    pub reference_graph: HashMap<u64, Vec<u64>>,
}

#[derive(Debug)]
pub struct CycleData {
    pub nodes: Vec<u64>,
    pub length: usize,
}

#[derive(Debug)]
pub struct GenericAnalysisData {
    pub instantiations: HashMap<String, u32>,
    pub monomorphization_impact: f64,
}

#[derive(Debug)]
pub struct AsyncAnalysisData {
    pub async_allocations: Vec<AsyncAllocationData>,
    pub future_memory_usage: HashMap<String, u64>,
}

#[derive(Debug)]
pub struct AsyncAllocationData {
    pub future_id: u64,
    pub size: usize,
    pub state: String,
}

#[derive(Debug)]
pub struct BorrowAnalysisData {
    pub violations: Vec<BorrowViolationData>,
    pub lifetime_conflicts: HashMap<String, Vec<String>>,
}

#[derive(Debug)]
pub struct BorrowViolationData {
    pub violation_type: String,
    pub location: String,
    pub severity: String,
}