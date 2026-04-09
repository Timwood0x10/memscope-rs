//! Integration tests for Analysis module
//!
//! Tests cover:
//! - Core detection: leaks, UAF, overflow, safety violations
//! - Boundary conditions: empty data, single allocation
//! - Edge cases: multiple issues, complex patterns

use memscope_rs::analysis::{
    AnalysisManager, Detector, LeakDetector, LeakDetectorConfig, LifecycleDetector,
    LifecycleDetectorConfig, OverflowDetector, OverflowDetectorConfig, SafetyDetector,
    SafetyDetectorConfig, UafDetector, UafDetectorConfig,
};
use memscope_rs::capture::types::{AllocationInfo, MemoryStats};

// ============================================================================
// Test Helpers
// ============================================================================

fn create_test_allocations() -> Vec<AllocationInfo> {
    vec![
        AllocationInfo::new(0x1000, 2048),
        AllocationInfo::new(0x2000, 4096),
        AllocationInfo::new(0x3000, 1024),
    ]
}

fn create_leaked_allocations() -> Vec<AllocationInfo> {
    let mut allocations = create_test_allocations();
    for alloc in &mut allocations {
        alloc.is_leaked = true;
    }
    allocations
}

// ============================================================================
// Leak Detection Tests
// ============================================================================

#[test]
fn test_leak_detector_finds_no_leaks_in_normal_allocations() {
    let detector = LeakDetector::new(LeakDetectorConfig::default());
    let allocations = create_test_allocations();

    let result = detector.detect(&allocations);
    assert!(
        result.issues.is_empty(),
        "Normal allocations should not trigger leak detection"
    );
}

#[test]
fn test_leak_detector_identifies_leaked_memory() {
    let detector = LeakDetector::new(LeakDetectorConfig::default());
    let allocations = create_leaked_allocations();

    let result = detector.detect(&allocations);
    assert_eq!(
        result.issues.len(),
        3,
        "Should detect exactly 3 leaked allocations"
    );
}

#[test]
fn test_leak_detector_handles_empty_input() {
    let detector = LeakDetector::new(LeakDetectorConfig::default());
    let empty: Vec<AllocationInfo> = vec![];

    let result = detector.detect(&empty);
    assert!(
        result.issues.is_empty(),
        "Empty input should produce no issues"
    );
}

#[test]
fn test_leak_detector_with_single_leaked_allocation() {
    let detector = LeakDetector::new(LeakDetectorConfig::default());
    let mut allocations = vec![AllocationInfo::new(0x1000, 1024)];
    allocations[0].is_leaked = true;

    let result = detector.detect(&allocations);
    assert_eq!(result.issues.len(), 1, "Should detect single leak");
}

// ============================================================================
// UAF Detection Tests
// ============================================================================

#[test]
fn test_uaf_detector_no_issues_for_normal_allocations() {
    let detector = UafDetector::new(UafDetectorConfig::default());
    let allocations = create_test_allocations();

    let result = detector.detect(&allocations);
    assert!(
        result.issues.is_empty(),
        "Normal allocations should not trigger UAF detection"
    );
}

#[test]
fn test_uaf_detector_handles_empty_input() {
    let detector = UafDetector::new(UafDetectorConfig::default());
    let empty: Vec<AllocationInfo> = vec![];

    let result = detector.detect(&empty);
    assert!(
        result.issues.is_empty(),
        "Empty input should produce no UAF issues"
    );
}

// ============================================================================
// Overflow Detection Tests
// ============================================================================

#[test]
fn test_overflow_detector_no_issues_for_normal_allocations() {
    let detector = OverflowDetector::new(OverflowDetectorConfig::default());
    let allocations = create_test_allocations();

    let result = detector.detect(&allocations);
    assert!(
        result.issues.is_empty(),
        "Normal allocations should not trigger overflow detection"
    );
}

#[test]
fn test_overflow_detector_handles_empty_input() {
    let detector = OverflowDetector::new(OverflowDetectorConfig::default());
    let empty: Vec<AllocationInfo> = vec![];

    let result = detector.detect(&empty);
    assert!(
        result.issues.is_empty(),
        "Empty input should produce no overflow issues"
    );
}

// ============================================================================
// Safety Detection Tests
// ============================================================================

#[test]
fn test_safety_detector_no_issues_for_normal_allocations() {
    let detector = SafetyDetector::new(SafetyDetectorConfig::default());
    let allocations = create_test_allocations();

    let result = detector.detect(&allocations);
    assert!(
        result.issues.is_empty(),
        "Normal allocations should not trigger safety detection"
    );
}

#[test]
fn test_safety_detector_handles_empty_input() {
    let detector = SafetyDetector::new(SafetyDetectorConfig::default());
    let empty: Vec<AllocationInfo> = vec![];

    let result = detector.detect(&empty);
    assert!(
        result.issues.is_empty(),
        "Empty input should produce no safety issues"
    );
}

// ============================================================================
// Lifecycle Detection Tests
// ============================================================================

#[test]
fn test_lifecycle_detector_no_issues_for_normal_allocations() {
    let detector = LifecycleDetector::new(LifecycleDetectorConfig::default());
    let allocations = create_test_allocations();

    let result = detector.detect(&allocations);
    assert!(
        result.issues.is_empty(),
        "Normal allocations should not trigger lifecycle detection"
    );
}

#[test]
fn test_lifecycle_detector_handles_empty_input() {
    let detector = LifecycleDetector::new(LifecycleDetectorConfig::default());
    let empty: Vec<AllocationInfo> = vec![];

    let result = detector.detect(&empty);
    assert!(
        result.issues.is_empty(),
        "Empty input should produce no lifecycle issues"
    );
}

// ============================================================================
// Analysis Manager Tests
// ============================================================================

#[test]
fn test_analysis_manager_fragmentation_calculation() {
    let manager = AnalysisManager::new();
    let allocations = create_test_allocations();

    let result = manager.analyze_fragmentation(&allocations);
    assert!(
        result.fragmentation_ratio >= 0.0 && result.fragmentation_ratio <= 1.0,
        "Fragmentation ratio should be between 0 and 1"
    );
}

#[test]
fn test_analysis_manager_empty_allocations() {
    let manager = AnalysisManager::new();
    let empty: Vec<AllocationInfo> = vec![];

    let frag = manager.analyze_fragmentation(&empty);
    assert_eq!(
        frag.fragmentation_ratio, 0.0,
        "Empty allocations should have 0 fragmentation"
    );

    let libs = manager.analyze_system_libraries(&empty);
    assert_eq!(
        libs.std_collections.allocation_count, 0,
        "Empty allocations should have no std collections"
    );
}

#[test]
fn test_analysis_manager_large_dataset() {
    let manager = AnalysisManager::new();

    let allocations: Vec<AllocationInfo> = (0..10000)
        .map(|i| AllocationInfo::new(0x1000 + i * 0x100, (i % 100 + 1) * 64))
        .collect();

    let frag = manager.analyze_fragmentation(&allocations);
    assert!(
        frag.fragmentation_ratio >= 0.0,
        "Large dataset should produce valid fragmentation ratio"
    );
}

#[test]
fn test_analysis_manager_comprehensive_analysis() {
    let manager = AnalysisManager::new();
    let allocations = create_test_allocations();
    let stats = MemoryStats::new();

    let report = manager.perform_comprehensive_analysis(&allocations, &stats);
    assert!(
        report.analysis_timestamp > 0,
        "Report should have valid timestamp"
    );
}

#[test]
fn test_analysis_manager_circular_reference_analysis() {
    let manager = AnalysisManager::new();
    let allocations = create_test_allocations();

    let result = manager.analyze_circular_references(&allocations);
    assert_eq!(
        result.total_smart_pointers, 0,
        "Basic allocations should have no smart pointers"
    );
}

#[test]
fn test_analysis_manager_borrow_pattern_analysis() {
    let manager = AnalysisManager::new();
    let allocations = create_test_allocations();

    let result = manager.analyze_borrow_patterns(&allocations);
    assert!(
        result.patterns.is_empty(),
        "Basic allocations should have no borrow patterns"
    );
}

#[test]
fn test_analysis_manager_generic_type_analysis() {
    let manager = AnalysisManager::new();
    let allocations = create_test_allocations();

    let result = manager.analyze_generic_types(&allocations);
    assert_eq!(
        result.total_instances, 0,
        "Basic allocations should have no generic instances"
    );
}

#[test]
fn test_analysis_manager_async_pattern_analysis() {
    let manager = AnalysisManager::new();
    let allocations = create_test_allocations();

    let result = manager.analyze_async_patterns(&allocations);
    assert!(
        result.patterns.is_empty(),
        "Basic allocations should have no async patterns"
    );
}

#[test]
fn test_analysis_manager_closure_pattern_analysis() {
    let manager = AnalysisManager::new();
    let allocations = create_test_allocations();

    let result = manager.analyze_closure_patterns(&allocations);
    assert!(
        result.detected_closures.is_empty(),
        "Basic allocations should have no closures"
    );
}

#[test]
fn test_analysis_manager_lifecycle_pattern_analysis() {
    let manager = AnalysisManager::new();
    let allocations = create_test_allocations();

    let result = manager.analyze_lifecycle_patterns(&allocations);
    assert!(
        result.drop_events.is_empty(),
        "Basic allocations should have no drop events"
    );
}

// ============================================================================
// Boundary Condition Tests
// ============================================================================

#[test]
fn test_single_allocation_analysis() {
    let manager = AnalysisManager::new();
    let allocations = vec![AllocationInfo::new(0x1000, 1024)];

    let frag = manager.analyze_fragmentation(&allocations);
    assert_eq!(
        frag.fragmentation_ratio, 0.0,
        "Single allocation should have 0 fragmentation"
    );
}

#[test]
fn test_very_large_allocation() {
    let manager = AnalysisManager::new();
    let allocations = vec![AllocationInfo::new(0x1000, 1024 * 1024 * 1024)]; // 1GB

    let frag = manager.analyze_fragmentation(&allocations);
    assert!(
        frag.fragmentation_ratio >= 0.0,
        "Large allocation should produce valid fragmentation"
    );
}

#[test]
fn test_many_small_allocations() {
    let manager = AnalysisManager::new();

    let allocations: Vec<AllocationInfo> = (0..10000)
        .map(|i| AllocationInfo::new(0x1000 + i, 16))
        .collect();

    let result = manager.analyze_fragmentation(&allocations);
    assert!(
        result.fragmentation_ratio >= 0.0,
        "Many small allocations should produce valid fragmentation"
    );
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_memory_leak_analysis_no_leaks() {
    let manager = AnalysisManager::new();
    let allocations = create_test_allocations();

    let result = manager.analyze_memory_leaks(&allocations);
    assert!(
        result.issues.is_empty(),
        "Normal allocations should have no leaks"
    );
}

#[test]
fn test_use_after_free_analysis_no_issues() {
    let manager = AnalysisManager::new();
    let allocations = create_test_allocations();

    let result = manager.analyze_use_after_free(&allocations);
    assert!(
        result.issues.is_empty(),
        "Normal allocations should have no UAF issues"
    );
}

#[test]
fn test_buffer_overflow_analysis_no_issues() {
    let manager = AnalysisManager::new();
    let allocations = create_test_allocations();

    let result = manager.analyze_buffer_overflow(&allocations);
    assert!(
        result.issues.is_empty(),
        "Normal allocations should have no overflow issues"
    );
}

#[test]
fn test_safety_violation_analysis_no_issues() {
    let manager = AnalysisManager::new();
    let allocations = create_test_allocations();

    let result = manager.analyze_safety_violations(&allocations);
    assert!(
        result.issues.is_empty(),
        "Normal allocations should have no safety violations"
    );
}

#[test]
fn test_lifecycle_issue_analysis_no_issues() {
    let manager = AnalysisManager::new();
    let allocations = create_test_allocations();

    let result = manager.analyze_lifecycle_issues(&allocations);
    assert!(
        result.issues.is_empty(),
        "Normal allocations should have no lifecycle issues"
    );
}
