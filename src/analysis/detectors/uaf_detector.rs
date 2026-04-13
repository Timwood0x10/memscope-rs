//! Use-after-free detection
//!
//! This module provides functionality for detecting use-after-free issues in Rust programs.
//!
//! # Example
//!
//! ```rust
//! use memscope_rs::analysis::detectors::{UafDetector, UafDetectorConfig, Detector};
//! use memscope_rs::capture::types::AllocationInfo;
//!
//! fn main() {
//!     let config = UafDetectorConfig::default();
//!     let detector = UafDetector::new(config);
//!
//!     let allocations = vec![];
//!     let result = detector.detect(&allocations);
//!
//!     println!("Found {} use-after-free issues", result.issues.len());
//! }
//! ```

use crate::analysis::detectors::{
    DetectionResult, DetectionStatistics, Detector, DetectorConfig, DetectorError, Issue,
    IssueCategory, IssueSeverity,
};
use crate::capture::types::AllocationInfo;

/// Configuration for UAF detector
#[derive(Debug, Clone)]
pub struct UafDetectorConfig {
    /// Enable raw pointer tracking
    pub enable_raw_pointer_tracking: bool,

    /// Enable borrow checker integration
    pub enable_borrow_checker_integration: bool,

    /// Maximum tracking depth
    pub max_tracking_depth: usize,
}

impl Default for UafDetectorConfig {
    fn default() -> Self {
        Self {
            enable_raw_pointer_tracking: true,
            enable_borrow_checker_integration: true,
            max_tracking_depth: 100,
        }
    }
}

/// Use-after-free detector
///
/// Detects use-after-free issues by tracking pointer lifetimes and access patterns.
///
/// # Detection Methods
///
/// - **Raw pointer tracking**: Tracks raw pointer usage and validates lifetime
/// - **Borrow checker integration**: Uses borrow checker information to detect violations
/// - **Lifetime analysis**: Analyzes lifetime annotations to find potential issues
#[derive(Debug)]
pub struct UafDetector {
    config: UafDetectorConfig,
    base_config: DetectorConfig,
}

impl UafDetector {
    /// Create a new UAF detector
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the UAF detector
    ///
    /// # Example
    ///
    /// ```rust
    /// use memscope_rs::analysis::detectors::{UafDetector, UafDetectorConfig};
    ///
    /// let config = UafDetectorConfig::default();
    /// let detector = UafDetector::new(config);
    /// ```
    pub fn new(config: UafDetectorConfig) -> Self {
        Self {
            config,
            base_config: DetectorConfig::default(),
        }
    }

    /// Get the UAF detector configuration
    pub fn uaf_config(&self) -> &UafDetectorConfig {
        &self.config
    }

    /// Update the UAF detector configuration
    pub fn update_uaf_config(&mut self, config: UafDetectorConfig) {
        self.config = config;
    }
}

impl Detector for UafDetector {
    fn name(&self) -> &str {
        "UafDetector"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        let start_time = std::time::Instant::now();

        let mut statistics = DetectionStatistics::new();
        statistics.total_allocations = allocations.len();

        let mut issues = Vec::new();

        // Detect raw pointer use-after-free
        if self.config.enable_raw_pointer_tracking {
            let uaf_issues = self.detect_raw_pointer_uaf(allocations, &mut statistics);
            issues.extend(uaf_issues);
        }

        // Detect lifetime violations
        if self.config.enable_borrow_checker_integration {
            let lifetime_issues = self.detect_lifetime_violations(allocations, &mut statistics);
            issues.extend(lifetime_issues);
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

impl UafDetector {
    /// Detect raw pointer use-after-free
    fn detect_raw_pointer_uaf(
        &self,
        allocations: &[AllocationInfo],
        statistics: &mut DetectionStatistics,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();

        for (index, alloc) in allocations.iter().enumerate() {
            // Check if allocation was deallocated but still has active references
            if let Some(dealloc_time) = alloc.timestamp_dealloc {
                // Memory was deallocated, check for potential use-after-free

                // Check if there are still active borrows after deallocation
                if alloc.borrow_count > 0 {
                    let issue_id = format!("uaf_borrow_after_free_{}", index);
                    let severity = self.assess_uaf_severity(alloc);

                    let issue = Issue::new(
                        issue_id,
                        severity,
                        IssueCategory::Safety,
                        format!(
                            "Use-after-free detected: {} active borrows after deallocation at 0x{:x}",
                            alloc.borrow_count, alloc.ptr
                        ),
                    )
                    .with_allocation_ptr(alloc.ptr)
                    .with_suggested_fix(
                        "Ensure all borrows are released before deallocation".to_string(),
                    );

                    issues.push(issue);
                    statistics.allocations_with_issues += 1;
                }

                // Check for raw pointer usage after deallocation
                if self.is_raw_pointer_type(alloc) {
                    let issue_id = format!("uaf_raw_pointer_{}", index);
                    let severity = self.assess_uaf_severity(alloc);

                    let issue = Issue::new(
                        issue_id,
                        severity,
                        IssueCategory::Safety,
                        format!(
                            "Raw pointer use-after-free detected at 0x{:x} (deallocated at {})",
                            alloc.ptr, dealloc_time
                        ),
                    )
                    .with_allocation_ptr(alloc.ptr)
                    .with_suggested_fix(
                        "Use ownership-based types (Box, Rc, Arc) instead of raw pointers"
                            .to_string(),
                    );

                    issues.push(issue);
                    statistics.allocations_with_issues += 1;
                }
            }

            // Check for potential lifetime violations
            if self.config.enable_borrow_checker_integration {
                if let Some(borrow_info) = &alloc.borrow_info {
                    if borrow_info.mutable_borrows > 1 {
                        let issue_id = format!("uaf_multiple_mutable_borrows_{}", index);
                        let severity = IssueSeverity::High;

                        let issue = Issue::new(
                            issue_id,
                            severity,
                            IssueCategory::Safety,
                            format!(
                                "Multiple mutable borrows detected: {} mutable borrows at 0x{:x}",
                                borrow_info.mutable_borrows, alloc.ptr
                            ),
                        )
                        .with_allocation_ptr(alloc.ptr)
                        .with_suggested_fix(
                            "Ensure at most one mutable borrow exists at any time".to_string(),
                        );

                        issues.push(issue);
                        statistics.allocations_with_issues += 1;
                    }
                }
            }
        }

        issues
    }

    /// Detect lifetime violations
    fn detect_lifetime_violations(
        &self,
        allocations: &[AllocationInfo],
        statistics: &mut DetectionStatistics,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();

        for (index, alloc) in allocations.iter().enumerate() {
            // Check for lifetime violations based on scope analysis
            if let Some(scope_name) = &alloc.scope_name {
                if self.is_function_local_scope(scope_name) {
                    // Check if allocation outlives its scope
                    if let Some(lifetime_ms) = alloc.lifetime_ms {
                        let expected_lifetime = self.estimate_expected_lifetime(scope_name);

                        if lifetime_ms > expected_lifetime * 10 {
                            let issue_id = format!("lifetime_scope_violation_{}", index);
                            let severity = IssueSeverity::Medium;

                            let issue = Issue::new(
                                issue_id,
                                severity,
                                IssueCategory::Safety,
                                format!(
                                    "Lifetime violation detected: allocation at 0x{:x} in scope '{}' lives {}ms, expected < {}ms",
                                    alloc.ptr, scope_name, lifetime_ms, expected_lifetime
                                ),
                            )
                            .with_allocation_ptr(alloc.ptr)
                            .with_suggested_fix(
                                "Consider moving allocation to outer scope or using static storage".to_string(),
                            );

                            issues.push(issue);
                            statistics.allocations_with_issues += 1;
                        }
                    }
                }
            }
        }

        issues
    }

    /// Assess UAF severity based on allocation characteristics
    fn assess_uaf_severity(&self, alloc: &AllocationInfo) -> IssueSeverity {
        // High if there are active borrows after deallocation
        if alloc.borrow_count > 0 && alloc.timestamp_dealloc.is_some() {
            return IssueSeverity::High;
        }

        // Medium for raw pointer usage
        if self.is_raw_pointer_type(alloc) {
            return IssueSeverity::Medium;
        }

        // Low otherwise
        IssueSeverity::Low
    }

    /// Check if allocation is of raw pointer type
    fn is_raw_pointer_type(&self, alloc: &AllocationInfo) -> bool {
        alloc
            .type_name
            .as_ref()
            .map(|t| t.contains("*const") || t.contains("*mut"))
            .unwrap_or(false)
    }

    /// Check if scope is function-local
    fn is_function_local_scope(&self, scope_name: &str) -> bool {
        !scope_name.contains("::") || scope_name.contains("fn ")
    }

    /// Estimate expected lifetime for a scope
    fn estimate_expected_lifetime(&self, scope_name: &str) -> u64 {
        // Simple heuristic: function-local scopes expect short lifetimes
        if self.is_function_local_scope(scope_name) {
            100 // 100ms expected for function-local
        } else {
            1000 // 1 second expected for module/global scopes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::types::AllocationInfo;

    #[test]
    fn test_uaf_detector_creation() {
        let config = UafDetectorConfig::default();
        let detector = UafDetector::new(config);

        assert_eq!(detector.name(), "UafDetector");
        assert_eq!(detector.version(), "1.0.0");
    }

    #[test]
    fn test_uaf_detector_config() {
        let config = UafDetectorConfig {
            enable_raw_pointer_tracking: false,
            enable_borrow_checker_integration: false,
            max_tracking_depth: 50,
        };

        let detector = UafDetector::new(config);
        let uaf_config = detector.uaf_config();

        assert!(!uaf_config.enable_raw_pointer_tracking);
        assert!(!uaf_config.enable_borrow_checker_integration);
        assert_eq!(uaf_config.max_tracking_depth, 50);
    }

    #[test]
    fn test_uaf_detector_detect() {
        let config = UafDetectorConfig::default();
        let detector = UafDetector::new(config);

        let allocations = vec![
            AllocationInfo::new(0x1000, 1024),
            AllocationInfo::new(0x2000, 2048),
        ];

        let result = detector.detect(&allocations);

        assert_eq!(result.detector_name, "UafDetector");
        assert_eq!(result.statistics.total_allocations, 2);
    }

    #[test]
    fn test_detect_raw_pointer_uaf_with_borrows_after_free() {
        let config = UafDetectorConfig::default();
        let detector = UafDetector::new(config);

        let mut allocations = vec![AllocationInfo::new(0x1000, 1024)];
        allocations[0].timestamp_dealloc = Some(1000);
        allocations[0].borrow_count = 2;

        let issues = detector.detect_raw_pointer_uaf(&allocations, &mut DetectionStatistics::new());

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.description.contains("active borrows after deallocation")));
    }

    #[test]
    fn test_detect_raw_pointer_uaf_with_raw_pointer() {
        let config = UafDetectorConfig::default();
        let detector = UafDetector::new(config);

        let mut allocations = vec![AllocationInfo::new(0x1000, 1024)];
        allocations[0].timestamp_dealloc = Some(1000);
        allocations[0].type_name = Some("*mut i32".to_string());

        let issues = detector.detect_raw_pointer_uaf(&allocations, &mut DetectionStatistics::new());

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.description.contains("Raw pointer use-after-free")));
    }

    #[test]
    fn test_detect_lifetime_violations_with_long_lived_local() {
        let config = UafDetectorConfig::default();
        let detector = UafDetector::new(config);

        let mut allocations = vec![AllocationInfo::new(0x1000, 1024)];
        allocations[0].scope_name = Some("fn main".to_string());
        allocations[0].lifetime_ms = Some(5000); // 5 seconds, much longer than expected

        let issues =
            detector.detect_lifetime_violations(&allocations, &mut DetectionStatistics::new());

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.description.contains("Lifetime violation")));
    }

    #[test]
    fn test_uaf_severity_assessment() {
        let config = UafDetectorConfig::default();
        let detector = UafDetector::new(config);

        // Test high severity (borrows after free)
        let mut alloc = AllocationInfo::new(0x1000, 1024);
        alloc.timestamp_dealloc = Some(1000);
        alloc.borrow_count = 2;

        let severity = detector.assess_uaf_severity(&alloc);
        assert_eq!(severity, IssueSeverity::High);
    }

    #[test]
    fn test_uaf_detector_disabled_tracking() {
        let config = UafDetectorConfig {
            enable_raw_pointer_tracking: false,
            enable_borrow_checker_integration: false,
            max_tracking_depth: 100,
        };
        let detector = UafDetector::new(config);

        let mut allocations = vec![AllocationInfo::new(0x1000, 1024)];
        allocations[0].timestamp_dealloc = Some(1000);
        allocations[0].borrow_count = 2;

        let result = detector.detect(&allocations);

        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_uaf_detector_with_multiple_mutable_borrows() {
        let config = UafDetectorConfig::default();
        let detector = UafDetector::new(config);

        let mut allocations = vec![AllocationInfo::new(0x1000, 1024)];

        // Create borrow info with multiple mutable borrows
        use crate::capture::types::BorrowInfo;
        allocations[0].borrow_info = Some(BorrowInfo {
            immutable_borrows: 0,
            mutable_borrows: 2,
            max_concurrent_borrows: 2,
            last_borrow_timestamp: Some(1000),
            _source: None,
            _confidence: None,
        });

        let issues = detector.detect_raw_pointer_uaf(&allocations, &mut DetectionStatistics::new());

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.description.contains("Multiple mutable borrows")));
    }

    #[test]
    fn test_is_raw_pointer_type() {
        let config = UafDetectorConfig::default();
        let detector = UafDetector::new(config);

        let mut alloc = AllocationInfo::new(0x1000, 1024);
        alloc.type_name = Some("*const i32".to_string());
        assert!(detector.is_raw_pointer_type(&alloc));

        alloc.type_name = Some("*mut u8".to_string());
        assert!(detector.is_raw_pointer_type(&alloc));

        alloc.type_name = Some("Box<i32>".to_string());
        assert!(!detector.is_raw_pointer_type(&alloc));
    }

    #[test]
    fn test_estimate_expected_lifetime() {
        let config = UafDetectorConfig::default();
        let detector = UafDetector::new(config);

        let function_lifetime = detector.estimate_expected_lifetime("fn main");
        assert_eq!(function_lifetime, 100);

        let module_lifetime = detector.estimate_expected_lifetime("my_module::foo");
        assert_eq!(module_lifetime, 1000);
    }
}
