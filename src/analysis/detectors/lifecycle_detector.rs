//! Lifecycle pattern detection
//!
//! This module provides functionality for detecting lifecycle pattern issues in Rust programs.
//!
//! # Example
//!
//! ```rust
//! use memscope_rs::analysis::detectors::{LifecycleDetector, LifecycleDetectorConfig, Detector};
//! use memscope_rs::capture::types::AllocationInfo;
//!
//! fn main() {
//!     let config = LifecycleDetectorConfig::default();
//!     let detector = LifecycleDetector::new(config);
//!
//!     let allocations = vec![];
//!     let result = detector.detect(&allocations);
//!
//!     println!("Found {} lifecycle issues", result.issues.len());
//! }
//! ```

use crate::analysis::detectors::{
    DetectionResult, DetectionStatistics, Detector, DetectorConfig, DetectorError, Issue,
    IssueCategory, IssueSeverity,
};
use crate::capture::types::AllocationInfo;

/// Configuration for lifecycle detector
#[derive(Debug, Clone)]
pub struct LifecycleDetectorConfig {
    /// Enable drop trait analysis
    pub enable_drop_trait_analysis: bool,

    /// Enable borrow violation detection
    pub enable_borrow_violation_detection: bool,

    /// Enable lifetime violation detection
    pub enable_lifetime_violation_detection: bool,

    /// Enable ownership pattern detection
    pub enable_ownership_pattern_detection: bool,

    /// Maximum depth for lifetime analysis
    pub max_lifetime_analysis_depth: usize,
}

impl Default for LifecycleDetectorConfig {
    fn default() -> Self {
        Self {
            enable_drop_trait_analysis: true,
            enable_borrow_violation_detection: true,
            enable_lifetime_violation_detection: true,
            enable_ownership_pattern_detection: true,
            max_lifetime_analysis_depth: 100,
        }
    }
}

/// Lifecycle pattern detector
///
/// Detects lifecycle pattern issues by analyzing memory lifecycle and borrow patterns.
///
/// # Detection Methods
///
/// - **Drop trait**: Analyzes Drop trait implementations
/// - **Borrow violations**: Detects borrow checker violations
/// - **Lifetime violations**: Detects lifetime annotation issues
/// - **Ownership patterns**: Analyzes ownership transfer and clone patterns
#[derive(Debug)]
pub struct LifecycleDetector {
    config: LifecycleDetectorConfig,
    base_config: DetectorConfig,
}

impl LifecycleDetector {
    /// Create a new lifecycle detector
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the lifecycle detector
    ///
    /// # Example
    ///
    /// ```rust
    /// use memscope_rs::analysis::detectors::{LifecycleDetector, LifecycleDetectorConfig};
    ///
    /// let config = LifecycleDetectorConfig::default();
    /// let detector = LifecycleDetector::new(config);
    /// ```
    pub fn new(config: LifecycleDetectorConfig) -> Self {
        Self {
            config,
            base_config: DetectorConfig::default(),
        }
    }

    /// Get the lifecycle detector configuration
    pub fn lifecycle_config(&self) -> &LifecycleDetectorConfig {
        &self.config
    }

    /// Update the lifecycle detector configuration
    pub fn update_lifecycle_config(&mut self, config: LifecycleDetectorConfig) {
        self.config = config;
    }
}

impl Detector for LifecycleDetector {
    fn name(&self) -> &str {
        "LifecycleDetector"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        let start_time = std::time::Instant::now();

        let mut statistics = DetectionStatistics::new();
        statistics.total_allocations = allocations.len();

        let mut issues = Vec::new();

        // Detect lifecycle issues
        if self.config.enable_lifetime_violation_detection {
            let lifecycle_issues = self.detect_lifetime_issues(allocations, &mut statistics);
            issues.extend(lifecycle_issues);
        }

        // Detect ownership patterns
        if self.config.enable_ownership_pattern_detection {
            let ownership_issues = self.detect_ownership_patterns(allocations, &mut statistics);
            issues.extend(ownership_issues);
        }

        // Detect drop trait issues
        if self.config.enable_drop_trait_analysis {
            let drop_issues = self.detect_drop_trait_issues(allocations, &mut statistics);
            issues.extend(drop_issues);
        }

        // Detect borrow violations
        if self.config.enable_borrow_violation_detection {
            let borrow_issues = self.detect_borrow_violations(allocations, &mut statistics);
            issues.extend(borrow_issues);
        }

        let detection_time_ms = start_time.elapsed().as_millis() as u64;

        DetectionResult {
            detector_name: self.name().to_string(),
            issues,
            statistics,
            detection_time_ms,
        }
    }

    fn config(&self) -> &DetectorConfig {
        &self.base_config
    }

    fn update_config(&mut self, config: DetectorConfig) -> Result<(), DetectorError> {
        self.base_config = config;
        Ok(())
    }
}

impl LifecycleDetector {
    /// Detect lifetime issues
    fn detect_lifetime_issues(
        &self,
        allocations: &[AllocationInfo],
        statistics: &mut DetectionStatistics,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();

        for (index, alloc) in allocations.iter().enumerate() {
            // Check for dangling references
            if let Some(lifecycle) = &alloc.lifecycle_tracking {
                // Check for lifecycle events after deallocation
                if let Some(dealloc_time) = alloc.timestamp_dealloc {
                    for event in &lifecycle.lifecycle_events {
                        if event.timestamp > dealloc_time {
                            let issue_id = format!("lifetime_post_dealloc_{}", index);
                            let severity = IssueSeverity::Critical;

                            let issue = Issue::new(
                                issue_id,
                                severity,
                                IssueCategory::Safety,
                                format!(
                                    "Lifetime violation: access at 0x{:x} after deallocation at {}",
                                    alloc.ptr, dealloc_time
                                ),
                            )
                            .with_allocation_ptr(alloc.ptr)
                            .with_suggested_fix(
                                "Review lifetime annotations and ensure references are properly scoped".to_string(),
                            );

                            issues.push(issue);
                            statistics.allocations_with_issues += 1;
                        }
                    }
                }

                // Check for excessive lifecycle events
                if lifecycle.lifecycle_events.len() > self.config.max_lifetime_analysis_depth {
                    let issue_id = format!("lifetime_complexity_{}", index);
                    let severity = IssueSeverity::Medium;

                    let issue = Issue::new(
                        issue_id,
                        severity,
                        IssueCategory::Performance,
                        format!(
                            "High lifecycle complexity detected at 0x{:x}: {} events",
                            alloc.ptr, lifecycle.lifecycle_events.len()
                        ),
                    )
                    .with_allocation_ptr(alloc.ptr)
                    .with_suggested_fix(
                        "Consider simplifying lifetime patterns or breaking into smaller components".to_string(),
                    );

                    issues.push(issue);
                    statistics.allocations_with_issues += 1;
                }
            }

            // Check for temporary object misuse
            if let Some(temp_info) = &alloc.temporary_object {
                if let Some(actual_lifetime) = alloc.lifetime_ms {
                    // Estimate expected lifetime
                    let expected_lifetime = match temp_info.lifetime_ns {
                        Some(ns) => ns / 1_000_000, // Convert ns to ms
                        None => 100,                // Default 100ms
                    };

                    if actual_lifetime > expected_lifetime * 10 {
                        let issue_id = format!("temporary_lifetime_{}", index);
                        let severity = IssueSeverity::Low;

                        let issue = Issue::new(
                            issue_id,
                            severity,
                            IssueCategory::Other,
                            format!(
                                "Temporary object at 0x{:x} lives longer than expected: {}ms vs expected < {}ms",
                                alloc.ptr, actual_lifetime, expected_lifetime
                            ),
                        )
                        .with_allocation_ptr(alloc.ptr)
                        .with_suggested_fix(
                            "Store temporary in named variable with explicit lifetime".to_string(),
                        );

                        issues.push(issue);
                        statistics.allocations_with_issues += 1;
                    }
                }
            }

            // Check for lifetime scope violations
            if let Some(scope_name) = &alloc.scope_name {
                if let Some(lifetime_ms) = alloc.lifetime_ms {
                    let expected_lifetime = self.estimate_scope_lifetime(scope_name);
                    if lifetime_ms > expected_lifetime * 5 {
                        let issue_id = format!("scope_lifetime_violation_{}", index);
                        let severity = IssueSeverity::Medium;

                        let issue = Issue::new(
                            issue_id,
                            severity,
                            IssueCategory::Lifetime,
                            format!(
                                "Scope lifetime violation at 0x{:x}: {}ms lifetime in scope '{}', expected < {}ms",
                                alloc.ptr, lifetime_ms, scope_name, expected_lifetime
                            ),
                        )
                        .with_allocation_ptr(alloc.ptr)
                        .with_suggested_fix(
                            "Move allocation to outer scope or reduce lifetime".to_string(),
                        );

                        issues.push(issue);
                        statistics.allocations_with_issues += 1;
                    }
                }
            }
        }

        issues
    }

    /// Detect ownership patterns
    fn detect_ownership_patterns(
        &self,
        allocations: &[AllocationInfo],
        statistics: &mut DetectionStatistics,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();

        for (index, alloc) in allocations.iter().enumerate() {
            // Check for excessive cloning
            if let Some(clone_info) = &alloc.clone_info {
                if clone_info.clone_count > 10 {
                    let issue_id = format!("excessive_cloning_{}", index);
                    let severity = self.assess_clone_severity(clone_info.clone_count);

                    let issue = Issue::new(
                        issue_id,
                        severity,
                        IssueCategory::Performance,
                        format!(
                            "Excessive cloning detected at 0x{:x}: {} clones",
                            alloc.ptr, clone_info.clone_count
                        ),
                    )
                    .with_allocation_ptr(alloc.ptr)
                    .with_suggested_fix(
                        "Consider using Arc for shared ownership or redesign to reduce cloning"
                            .to_string(),
                    );

                    issues.push(issue);
                    statistics.allocations_with_issues += 1;
                }

                // Check for expensive clone operations
                if clone_info.clone_count > 0 && alloc.size > 1024 * 1024 {
                    // >1MB
                    let issue_id = format!("expensive_clone_{}", index);
                    let severity = IssueSeverity::High;

                    let issue = Issue::new(
                        issue_id,
                        severity,
                        IssueCategory::Performance,
                        format!(
                            "Expensive clone operation at 0x{:x}: cloning {} bytes",
                            alloc.ptr, alloc.size
                        ),
                    )
                    .with_allocation_ptr(alloc.ptr)
                    .with_suggested_fix(
                        "Use Arc for shared ownership or reference instead of cloning".to_string(),
                    );

                    issues.push(issue);
                    statistics.allocations_with_issues += 1;
                }
            }

            // Check for smart pointer patterns
            if let Some(smart_ptr_info) = &alloc.smart_pointer_info {
                // Check for excessive ref count history
                if smart_ptr_info.ref_count_history.len() > 100 {
                    let issue_id = format!("high_ref_count_history_{}", index);
                    let severity = IssueSeverity::Medium;

                    let issue = Issue::new(
                        issue_id,
                        severity,
                        IssueCategory::Performance,
                        format!(
                            "High reference count history at 0x{:x}: {} snapshots",
                            alloc.ptr,
                            smart_ptr_info.ref_count_history.len()
                        ),
                    )
                    .with_allocation_ptr(alloc.ptr)
                    .with_suggested_fix(
                        "Review reference counting pattern and consider using Weak references"
                            .to_string(),
                    );

                    issues.push(issue);
                    statistics.allocations_with_issues += 1;
                }

                // Check for reference cycles
                if self.has_reference_cycle(smart_ptr_info) {
                    let issue_id = format!("reference_cycle_{}", index);
                    let severity = IssueSeverity::High;

                    let issue = Issue::new(
                        issue_id,
                        severity,
                        IssueCategory::Safety,
                        format!("Potential reference cycle detected at 0x{:x}", alloc.ptr),
                    )
                    .with_allocation_ptr(alloc.ptr)
                    .with_suggested_fix(
                        "Break reference cycles using Weak references or explicit cleanup"
                            .to_string(),
                    );

                    issues.push(issue);
                    statistics.allocations_with_issues += 1;
                }
            }

            // Check for ownership transfer issues
            if self.is_move_semantics_violation(alloc) {
                let issue_id = format!("move_semantics_violation_{}", index);
                let severity = IssueSeverity::High;

                let issue = Issue::new(
                    issue_id,
                    severity,
                    IssueCategory::Safety,
                    format!("Move semantics violation detected at 0x{:x}", alloc.ptr),
                )
                .with_allocation_ptr(alloc.ptr)
                .with_suggested_fix(
                    "Review ownership transfer and ensure proper move semantics".to_string(),
                );

                issues.push(issue);
                statistics.allocations_with_issues += 1;
            }
        }

        issues
    }

    /// Detect drop trait issues
    fn detect_drop_trait_issues(
        &self,
        allocations: &[AllocationInfo],
        statistics: &mut DetectionStatistics,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();

        for (index, alloc) in allocations.iter().enumerate() {
            // Check for missing drop implementation on large allocations
            if alloc.size > 10 * 1024 * 1024 {
                // >10MB
                let issue_id = format!("large_allocation_no_drop_{}", index);
                let severity = IssueSeverity::Medium;

                let issue = Issue::new(
                    issue_id,
                    severity,
                    IssueCategory::Other,
                    format!(
                        "Large allocation at 0x{:x} without custom Drop: {} bytes",
                        alloc.ptr, alloc.size
                    ),
                )
                .with_allocation_ptr(alloc.ptr)
                .with_suggested_fix(
                    "Consider implementing custom Drop for proper resource cleanup".to_string(),
                );

                issues.push(issue);
                statistics.allocations_with_issues += 1;
            }

            // Check for panics during drop using lifecycle events
            if let Some(lifecycle) = &alloc.lifecycle_tracking {
                for event in &lifecycle.lifecycle_events {
                    // Check if event indicates a problematic state
                    if let crate::capture::types::LifecycleEventType::Drop = event.event_type {
                        // Check if there are lifecycle events after drop
                        if let Some(dealloc_time) = alloc.timestamp_dealloc {
                            if event.timestamp > dealloc_time + 1000 {
                                // Event more than 1s after deallocation
                                let issue_id = format!("slow_drop_{}", index);
                                let severity = IssueSeverity::Medium;

                                let issue = Issue::new(
                                    issue_id,
                                    severity,
                                    IssueCategory::Performance,
                                    format!(
                                        "Slow drop detected at 0x{:x}: {}ms delay",
                                        alloc.ptr, event.timestamp - dealloc_time
                                    ),
                                )
                                .with_allocation_ptr(alloc.ptr)
                                .with_suggested_fix(
                                    "Optimize Drop implementation or use async cleanup for expensive operations".to_string(),
                                );

                                issues.push(issue);
                                statistics.allocations_with_issues += 1;
                            }
                        }
                    }
                }
            }
        }

        issues
    }

    /// Detect borrow violations
    fn detect_borrow_violations(
        &self,
        allocations: &[AllocationInfo],
        statistics: &mut DetectionStatistics,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();

        for (index, alloc) in allocations.iter().enumerate() {
            // Check for concurrent mutable borrows
            if let Some(borrow_info) = &alloc.borrow_info {
                if borrow_info.mutable_borrows > 1 {
                    let issue_id = format!("concurrent_mutable_borrows_{}", index);
                    let severity = IssueSeverity::High;

                    let issue = Issue::new(
                        issue_id,
                        severity,
                        IssueCategory::Safety,
                        format!(
                            "Concurrent mutable borrows detected at 0x{:x}: {} mutable borrows",
                            alloc.ptr, borrow_info.mutable_borrows
                        ),
                    )
                    .with_allocation_ptr(alloc.ptr)
                    .with_suggested_fix(
                        "Ensure at most one mutable borrow exists at any time".to_string(),
                    );

                    issues.push(issue);
                    statistics.allocations_with_issues += 1;
                }

                // Check for excessive borrow count
                let total_borrows = borrow_info.mutable_borrows + borrow_info.immutable_borrows;
                if total_borrows > 50 {
                    let issue_id = format!("excessive_borrows_{}", index);
                    let severity = IssueSeverity::Low;

                    let issue = Issue::new(
                        issue_id,
                        severity,
                        IssueCategory::Other,
                        format!(
                            "Excessive borrows detected at 0x{:x}: {} total borrows",
                            alloc.ptr, total_borrows
                        ),
                    )
                    .with_allocation_ptr(alloc.ptr)
                    .with_suggested_fix(
                        "Reduce borrow count or consider using references more efficiently"
                            .to_string(),
                    );

                    issues.push(issue);
                    statistics.allocations_with_issues += 1;
                }
            }

            // Check for borrow after move
            if self.is_borrow_after_move(alloc) {
                let issue_id = format!("borrow_after_move_{}", index);
                let severity = IssueSeverity::High;

                let issue = Issue::new(
                    issue_id,
                    severity,
                    IssueCategory::Safety,
                    format!("Borrow after move detected at 0x{:x}", alloc.ptr),
                )
                .with_allocation_ptr(alloc.ptr)
                .with_suggested_fix(
                    "Review move semantics and ensure borrows are before move operations"
                        .to_string(),
                );

                issues.push(issue);
                statistics.allocations_with_issues += 1;
            }
        }

        issues
    }

    /// Assess clone severity
    fn assess_clone_severity(&self, clone_count: usize) -> IssueSeverity {
        // Critical for very high clone counts
        if clone_count > 1000 {
            return IssueSeverity::Critical;
        }

        // High for high clone counts
        if clone_count > 100 {
            return IssueSeverity::High;
        }

        // Medium for moderate clone counts
        IssueSeverity::Medium
    }

    /// Estimate expected lifetime for a scope
    fn estimate_scope_lifetime(&self, scope_name: &str) -> u64 {
        if scope_name.contains("fn ") {
            100 // 100ms for function-local
        } else if scope_name.contains("::") {
            1000 // 1 second for module-level
        } else {
            500 // 500ms for block-level
        }
    }

    /// Check if smart pointer has reference cycle
    fn has_reference_cycle(
        &self,
        smart_ptr_info: &crate::capture::types::SmartPointerInfo,
    ) -> bool {
        // Check if reference count history shows cycles
        let mut counts: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for snapshot in &smart_ptr_info.ref_count_history {
            if counts.contains(&snapshot.strong_count) {
                // Strong count seen before - potential cycle
                return true;
            }
            counts.insert(snapshot.strong_count);
        }
        false
    }

    /// Check if move semantics are violated
    fn is_move_semantics_violation(&self, alloc: &AllocationInfo) -> bool {
        // Check for use after move (indicated by access after ownership transfer)
        if let Some(clone_info) = &alloc.clone_info {
            if clone_info.is_clone && clone_info.original_ptr.is_some() {
                // Clone after potential move
                return true;
            }
        }
        false
    }

    /// Check if there's a borrow after move
    fn is_borrow_after_move(&self, alloc: &AllocationInfo) -> bool {
        // Check if there are borrows after the last ownership transfer
        if let Some(clone_info) = &alloc.clone_info {
            if clone_info.is_clone && alloc.borrow_count > 0 {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::types::AllocationInfo;

    #[test]
    fn test_lifecycle_detector_creation() {
        let config = LifecycleDetectorConfig::default();
        let detector = LifecycleDetector::new(config);

        assert_eq!(detector.name(), "LifecycleDetector");
        assert_eq!(detector.version(), "1.0.0");
    }

    #[test]
    fn test_lifecycle_detector_config() {
        let config = LifecycleDetectorConfig {
            enable_drop_trait_analysis: false,
            enable_borrow_violation_detection: false,
            enable_lifetime_violation_detection: false,
            enable_ownership_pattern_detection: false,
            max_lifetime_analysis_depth: 50,
        };

        let detector = LifecycleDetector::new(config);
        let lifecycle_config = detector.lifecycle_config();

        assert!(!lifecycle_config.enable_drop_trait_analysis);
        assert!(!lifecycle_config.enable_borrow_violation_detection);
        assert!(!lifecycle_config.enable_lifetime_violation_detection);
        assert!(!lifecycle_config.enable_ownership_pattern_detection);
        assert_eq!(lifecycle_config.max_lifetime_analysis_depth, 50);
    }

    #[test]
    fn test_lifecycle_detector_detect() {
        let config = LifecycleDetectorConfig::default();
        let detector = LifecycleDetector::new(config);

        let allocations = vec![
            AllocationInfo::new(0x1000, 1024),
            AllocationInfo::new(0x2000, 2048),
        ];

        let result = detector.detect(&allocations);

        assert_eq!(result.detector_name, "LifecycleDetector");
        assert_eq!(result.statistics.total_allocations, 2);
    }

    #[test]
    fn test_detect_excessive_cloning() {
        let config = LifecycleDetectorConfig::default();
        let detector = LifecycleDetector::new(config);

        let mut allocations = vec![AllocationInfo::new(0x1000, 1024)];

        use crate::capture::types::CloneInfo;
        allocations[0].clone_info = Some(CloneInfo {
            clone_count: 50, // Excessive cloning
            is_clone: true,
            original_ptr: Some(0x1000),
        });

        let issues =
            detector.detect_ownership_patterns(&allocations, &mut DetectionStatistics::new());

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.description.contains("Excessive cloning")));
    }

    #[test]
    fn test_detect_concurrent_mutable_borrows() {
        let config = LifecycleDetectorConfig::default();
        let detector = LifecycleDetector::new(config);

        let mut allocations = vec![AllocationInfo::new(0x1000, 1024)];
        allocations[0].borrow_count = 2;

        use crate::capture::types::BorrowInfo;
        allocations[0].borrow_info = Some(BorrowInfo {
            immutable_borrows: 0,
            mutable_borrows: 2, // Concurrent mutable borrows
            max_concurrent_borrows: 2,
            last_borrow_timestamp: Some(1000),
        });

        let issues =
            detector.detect_borrow_violations(&allocations, &mut DetectionStatistics::new());

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.description.contains("Concurrent mutable borrows")));
    }

    #[test]
    fn test_detect_scope_lifetime_violation() {
        let config = LifecycleDetectorConfig::default();
        let detector = LifecycleDetector::new(config);

        let mut allocations = vec![AllocationInfo::new(0x1000, 1024)];
        allocations[0].scope_name = Some("fn main".to_string());
        allocations[0].lifetime_ms = Some(5000); // 5 seconds, much longer than expected

        let issues = detector.detect_lifetime_issues(&allocations, &mut DetectionStatistics::new());

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.description.contains("Scope lifetime violation")));
    }

    #[test]
    fn test_assess_clone_severity() {
        let config = LifecycleDetectorConfig::default();
        let detector = LifecycleDetector::new(config);

        assert_eq!(
            detector.assess_clone_severity(1500),
            IssueSeverity::Critical
        );
        assert_eq!(detector.assess_clone_severity(150), IssueSeverity::High);
        assert_eq!(detector.assess_clone_severity(50), IssueSeverity::Medium);
    }

    #[test]
    fn test_estimate_scope_lifetime() {
        let config = LifecycleDetectorConfig::default();
        let detector = LifecycleDetector::new(config);

        assert_eq!(detector.estimate_scope_lifetime("fn main"), 100);
        assert_eq!(detector.estimate_scope_lifetime("module::function"), 1000);
        assert_eq!(detector.estimate_scope_lifetime("block"), 500);
    }

    #[test]
    fn test_lifecycle_detector_disabled() {
        let config = LifecycleDetectorConfig {
            enable_drop_trait_analysis: false,
            enable_borrow_violation_detection: false,
            enable_lifetime_violation_detection: false,
            enable_ownership_pattern_detection: false,
            max_lifetime_analysis_depth: 100,
        };
        let detector = LifecycleDetector::new(config);

        let allocations = vec![AllocationInfo::new(0x1000, 1024)];
        let result = detector.detect(&allocations);

        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_detect_large_allocation_no_drop() {
        let config = LifecycleDetectorConfig::default();
        let detector = LifecycleDetector::new(config);

        let allocations = vec![AllocationInfo::new(0x1000, 20 * 1024 * 1024)]; // 20MB

        let issues =
            detector.detect_drop_trait_issues(&allocations, &mut DetectionStatistics::new());

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.description.contains("Large allocation")));
    }
}
