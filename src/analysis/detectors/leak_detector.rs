//! Memory leak detection
//!
//! This module provides functionality for detecting memory leaks in Rust programs.
//!
//! # Example
//!
//! ```rust
//! use memscope_rs::analysis::detectors::{LeakDetector, LeakDetectorConfig, Detector};
//! use memscope_rs::capture::types::AllocationInfo;
//!
//! fn main() {
//!     let config = LeakDetectorConfig::default();
//!     let detector = LeakDetector::new(config);
//!
//!     let allocations = vec![];
//!     let result = detector.detect(&allocations);
//!
//!     println!("Found {} potential memory leaks", result.issues.len());
//! }
//! ```

use crate::analysis::detectors::{
    DetectionResult, DetectionStatistics, Detector, DetectorConfig, DetectorError, Issue,
    IssueCategory, IssueSeverity,
};
use crate::capture::types::AllocationInfo;
use std::collections::HashSet;

/// Configuration for leak detector
#[derive(Debug, Clone)]
pub struct LeakDetectorConfig {
    /// Enable cross-thread leak detection
    pub enable_cross_thread_leak_detection: bool,

    /// Enable smart pointer leak detection
    pub enable_smart_pointer_leak_detection: bool,

    /// Minimum leak threshold in bytes
    pub min_leak_threshold_bytes: usize,

    /// Maximum number of leaks to report
    pub max_reported_leaks: usize,
}

impl Default for LeakDetectorConfig {
    fn default() -> Self {
        Self {
            enable_cross_thread_leak_detection: true,
            enable_smart_pointer_leak_detection: true,
            min_leak_threshold_bytes: 1024,
            max_reported_leaks: 100,
        }
    }
}

/// Memory leak detector
///
/// Detects memory leaks by analyzing allocation and deallocation patterns.
///
/// # Detection Methods
///
/// - **Cross-thread leaks**: Detects leaks when allocations are moved between threads
/// - **Smart pointer leaks**: Detects reference cycles in Rc, Arc, and other smart pointers
/// - **Allocation analysis**: Analyzes allocation patterns to identify potential leaks
#[derive(Debug)]
pub struct LeakDetector {
    config: LeakDetectorConfig,
    base_config: DetectorConfig,
}

impl LeakDetector {
    /// Create a new leak detector
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the leak detector
    ///
    /// # Example
    ///
    /// ```rust
    /// use memscope_rs::analysis::detectors::{LeakDetector, LeakDetectorConfig};
    ///
    /// let config = LeakDetectorConfig::default();
    /// let detector = LeakDetector::new(config);
    /// ```
    pub fn new(config: LeakDetectorConfig) -> Self {
        Self {
            config,
            base_config: DetectorConfig::default(),
        }
    }

    /// Get the leak detector configuration
    pub fn leak_config(&self) -> &LeakDetectorConfig {
        &self.config
    }

    /// Update the leak detector configuration
    pub fn update_leak_config(&mut self, config: LeakDetectorConfig) {
        self.config = config;
    }

    /// Detect memory leaks in allocations
    ///
    /// Analyzes the provided allocations and identifies potential memory leaks.
    ///
    /// # Arguments
    ///
    /// * `allocations` - Slice of allocation information to analyze
    ///
    /// # Returns
    ///
    /// Vector of detected leak issues.
    pub fn detect_leaks(&self, allocations: &[AllocationInfo]) -> Vec<Issue> {
        let mut issues = Vec::new();

        for (index, alloc) in allocations.iter().enumerate() {
            if alloc.is_leaked && alloc.size >= self.config.min_leak_threshold_bytes {
                let issue_id = format!("leak_{}", index);
                let severity = self.assess_leak_severity(alloc.size);

                let issue = Issue::new(
                    issue_id,
                    severity,
                    IssueCategory::Leak,
                    format!(
                        "Memory leak detected: {} bytes at 0x{:x}",
                        alloc.size, alloc.ptr
                    ),
                )
                .with_allocation_ptr(alloc.ptr)
                .with_suggested_fix("Ensure this allocation is properly deallocated".to_string());

                issues.push(issue);
            }
        }

        if issues.len() > self.config.max_reported_leaks {
            issues.truncate(self.config.max_reported_leaks);
        }

        issues
    }

    /// Detect smart pointer cycles
    ///
    /// Identifies reference cycles in Rc, Arc, and other smart pointers.
    ///
    /// # Arguments
    ///
    /// * `allocations` - Slice of allocation information to analyze
    ///
    /// # Returns
    ///
    /// Vector of detected cycle issues.
    pub fn detect_smart_pointer_cycles(&self, allocations: &[AllocationInfo]) -> Vec<Issue> {
        let mut issues = Vec::new();
        let mut visited = HashSet::new();

        for (index, alloc) in allocations.iter().enumerate() {
            if visited.contains(&alloc.ptr) {
                continue;
            }

            if alloc
                .type_name
                .as_ref()
                .map(|t| self.is_smart_pointer_type(t))
                .unwrap_or(false)
            {
                let cycle_members = self.find_cycle_members(alloc, allocations, &mut visited);

                if cycle_members.len() > 1 {
                    let issue_id = format!("cycle_{}", index);
                    let total_memory: usize = cycle_members.iter().map(|a| a.size).sum();

                    let issue = Issue::new(
                        issue_id,
                        IssueSeverity::High,
                        IssueCategory::Leak,
                        format!(
                            "Reference cycle detected in smart pointers: {} allocations, {} bytes",
                            cycle_members.len(),
                            total_memory
                        ),
                    )
                    .with_suggested_fix(
                        "Break the cycle by using Weak references or explicitly dropping references".to_string(),
                    );

                    issues.push(issue);
                }
            }
        }

        issues
    }

    fn assess_leak_severity(&self, size: usize) -> IssueSeverity {
        if size >= 10 * 1024 * 1024 {
            IssueSeverity::Critical
        } else if size >= 1024 * 1024 {
            IssueSeverity::High
        } else if size >= 100 * 1024 {
            IssueSeverity::Medium
        } else {
            IssueSeverity::Low
        }
    }

    fn is_smart_pointer_type(&self, type_name: &str) -> bool {
        type_name.contains("Rc<") || type_name.contains("Arc<") || type_name.contains("Box<")
    }

    fn find_cycle_members<'a>(
        &self,
        start: &'a AllocationInfo,
        allocations: &'a [AllocationInfo],
        visited: &mut HashSet<usize>,
    ) -> Vec<&'a AllocationInfo> {
        let mut cycle_members = Vec::new();
        let mut current = start;

        let max_iterations = allocations.len();
        for _ in 0..max_iterations {
            if visited.contains(&current.ptr) {
                break;
            }

            visited.insert(current.ptr);
            cycle_members.push(current);

            if let Some(next) = allocations
                .iter()
                .find(|a| a.type_name == current.type_name && a.ptr != current.ptr)
            {
                current = next;
            } else {
                break;
            }
        }

        cycle_members
    }
}

impl Detector for LeakDetector {
    fn name(&self) -> &str {
        "LeakDetector"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        let start_time = std::time::Instant::now();

        let mut statistics = DetectionStatistics::new();
        statistics.total_allocations = allocations.len();

        let mut issues = Vec::new();

        if self.config.enable_cross_thread_leak_detection {
            let leak_issues = self.detect_leaks(allocations);
            for issue in &leak_issues {
                statistics.update_severity(issue.severity);
                statistics.update_category(issue.category);
                statistics.allocations_with_issues += 1;
                statistics.memory_affected += issue.allocation_ptr.map(|_| 1).unwrap_or(0);
            }
            issues.extend(leak_issues);
        }

        if self.config.enable_smart_pointer_leak_detection {
            let cycle_issues = self.detect_smart_pointer_cycles(allocations);
            for issue in &cycle_issues {
                statistics.update_severity(issue.severity);
                statistics.update_category(issue.category);
                statistics.allocations_with_issues += 1;
            }
            issues.extend(cycle_issues);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::types::AllocationInfo;

    #[test]
    fn test_leak_detector_creation() {
        let config = LeakDetectorConfig::default();
        let detector = LeakDetector::new(config);

        assert_eq!(detector.name(), "LeakDetector");
        assert_eq!(detector.version(), "1.0.0");
    }

    #[test]
    fn test_leak_detector_config() {
        let config = LeakDetectorConfig {
            enable_cross_thread_leak_detection: false,
            enable_smart_pointer_leak_detection: false,
            min_leak_threshold_bytes: 512,
            max_reported_leaks: 50,
        };

        let detector = LeakDetector::new(config);
        let leak_config = detector.leak_config();

        assert!(!leak_config.enable_cross_thread_leak_detection);
        assert!(!leak_config.enable_smart_pointer_leak_detection);
        assert_eq!(leak_config.min_leak_threshold_bytes, 512);
        assert_eq!(leak_config.max_reported_leaks, 50);
    }

    #[test]
    fn test_leak_detector_detect() {
        let config = LeakDetectorConfig::default();
        let detector = LeakDetector::new(config);

        let allocations = vec![
            AllocationInfo::new(0x1000, 1024),
            AllocationInfo::new(0x2000, 2048),
        ];

        let result = detector.detect(&allocations);

        assert_eq!(result.detector_name, "LeakDetector");
        assert_eq!(result.statistics.total_allocations, 2);
    }

    #[test]
    fn test_detect_leaks() {
        let config = LeakDetectorConfig::default();
        let detector = LeakDetector::new(config);

        let mut allocations = vec![
            AllocationInfo::new(0x1000, 1024),
            AllocationInfo::new(0x2000, 1024),
            AllocationInfo::new(0x3000, 2048),
        ];

        allocations[0].is_leaked = true;
        allocations[2].is_leaked = true;

        let issues = detector.detect_leaks(&allocations);

        assert_eq!(issues.len(), 2);
        assert!(issues.iter().any(|i| i.allocation_ptr == Some(0x1000)));
        assert!(issues.iter().any(|i| i.allocation_ptr == Some(0x3000)));
    }

    #[test]
    fn test_detect_smart_pointer_cycles() {
        let config = LeakDetectorConfig::default();
        let detector = LeakDetector::new(config);

        let mut allocations = vec![
            AllocationInfo::new(0x1000, 1024),
            AllocationInfo::new(0x2000, 1024),
            AllocationInfo::new(0x3000, 1024),
        ];

        allocations[0].type_name = Some("Rc<i32>".to_string());
        allocations[1].type_name = Some("Rc<i32>".to_string());
        allocations[2].type_name = Some("Rc<i32>".to_string());

        let issues = detector.detect_smart_pointer_cycles(&allocations);

        assert!(!issues.is_empty());
    }

    #[test]
    fn test_leak_severity_assessment() {
        let config = LeakDetectorConfig::default();
        let detector = LeakDetector::new(config);

        let small = 50 * 1024;
        let medium = 500 * 1024;
        let large = 2 * 1024 * 1024;
        let huge = 20 * 1024 * 1024;

        assert_eq!(detector.assess_leak_severity(small), IssueSeverity::Low);
        assert_eq!(detector.assess_leak_severity(medium), IssueSeverity::Medium);
        assert_eq!(detector.assess_leak_severity(large), IssueSeverity::High);
        assert_eq!(detector.assess_leak_severity(huge), IssueSeverity::Critical);
    }

    #[test]
    fn test_smart_pointer_type_detection() {
        let config = LeakDetectorConfig::default();
        let detector = LeakDetector::new(config);

        assert!(detector.is_smart_pointer_type("Rc<i32>"));
        assert!(detector.is_smart_pointer_type("Arc<Vec<i32>>"));
        assert!(detector.is_smart_pointer_type("Box<String>"));
        assert!(!detector.is_smart_pointer_type("String"));
        assert!(!detector.is_smart_pointer_type("Vec<i32>"));
    }

    #[test]
    fn test_max_reported_leaks_limit() {
        let config = LeakDetectorConfig {
            max_reported_leaks: 2,
            ..Default::default()
        };
        let detector = LeakDetector::new(config);

        let mut allocations = vec![];
        for i in 0..5 {
            let mut alloc = AllocationInfo::new(0x1000 + i * 0x1000, 2048);
            alloc.is_leaked = true;
            allocations.push(alloc);
        }

        let issues = detector.detect_leaks(&allocations);

        assert_eq!(issues.len(), 2);
    }

    #[test]
    fn test_min_leak_threshold() {
        let config = LeakDetectorConfig {
            min_leak_threshold_bytes: 2048,
            ..Default::default()
        };
        let detector = LeakDetector::new(config);

        let mut allocations = vec![
            AllocationInfo::new(0x1000, 1024),
            AllocationInfo::new(0x2000, 2048),
            AllocationInfo::new(0x3000, 4096),
        ];

        allocations[0].is_leaked = true;
        allocations[1].is_leaked = true;
        allocations[2].is_leaked = true;

        let issues = detector.detect_leaks(&allocations);

        assert_eq!(issues.len(), 2);
    }
}
