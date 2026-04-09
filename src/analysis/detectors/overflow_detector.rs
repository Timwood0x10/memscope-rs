//! Buffer overflow detection
//!
//! This module provides functionality for detecting buffer overflow issues in Rust programs.
//!
//! # Example
//!
//! ```rust
//! use memscope_rs::analysis::detectors::{OverflowDetector, OverflowDetectorConfig, Detector};
//! use memscope_rs::capture::types::AllocationInfo;
//!
//! fn main() {
//!     let config = OverflowDetectorConfig::default();
//!     let detector = OverflowDetector::new(config);
//!
//!     let allocations = vec![];
//!     let result = detector.detect(&allocations);
//!
//!     println!("Found {} buffer overflow issues", result.issues.len());
//! }
//! ```

use crate::analysis::detectors::{
    DetectionResult, DetectionStatistics, Detector, DetectorConfig, DetectorError, Issue,
    IssueCategory, IssueSeverity,
};
use crate::capture::types::AllocationInfo;

/// Configuration for overflow detector
#[derive(Debug, Clone)]
pub struct OverflowDetectorConfig {
    /// Enable heap overflow detection
    pub enable_heap_overflow_detection: bool,

    /// Enable stack overflow detection
    pub enable_stack_overflow_detection: bool,

    /// Enable integer overflow detection
    pub enable_integer_overflow_detection: bool,

    /// Minimum buffer size in bytes to analyze
    pub min_buffer_size_bytes: usize,

    /// Maximum array size to analyze
    pub max_array_size: usize,
}

impl Default for OverflowDetectorConfig {
    fn default() -> Self {
        Self {
            enable_heap_overflow_detection: true,
            enable_stack_overflow_detection: true,
            enable_integer_overflow_detection: true,
            min_buffer_size_bytes: 64,
            max_array_size: 1024 * 1024, // 1MB
        }
    }
}

/// Buffer overflow detector
///
/// Detects buffer overflow issues by analyzing memory access patterns.
///
/// # Detection Methods
///
/// - **Heap overflow**: Detects overflows in heap-allocated buffers
/// - **Stack overflow**: Detects overflows in stack-allocated buffers
/// - **Integer overflow**: Detects arithmetic operations that exceed type bounds
#[derive(Debug)]
pub struct OverflowDetector {
    config: OverflowDetectorConfig,
    base_config: DetectorConfig,
}

impl OverflowDetector {
    /// Create a new overflow detector
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the overflow detector
    ///
    /// # Example
    ///
    /// ```rust
    /// use memscope_rs::analysis::detectors::{OverflowDetector, OverflowDetectorConfig};
    ///
    /// let config = OverflowDetectorConfig::default();
    /// let detector = OverflowDetector::new(config);
    /// ```
    pub fn new(config: OverflowDetectorConfig) -> Self {
        Self {
            config,
            base_config: DetectorConfig::default(),
        }
    }

    /// Get the overflow detector configuration
    pub fn overflow_config(&self) -> &OverflowDetectorConfig {
        &self.config
    }

    /// Update the overflow detector configuration
    pub fn update_overflow_config(&mut self, config: OverflowDetectorConfig) {
        self.config = config;
    }
}

impl Detector for OverflowDetector {
    fn name(&self) -> &str {
        "OverflowDetector"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        let start_time = std::time::Instant::now();

        let mut statistics = DetectionStatistics::new();
        statistics.total_allocations = allocations.len();

        let mut issues = Vec::new();

        // Detect heap buffer overflows
        if self.config.enable_heap_overflow_detection {
            let heap_issues = self.detect_buffer_overflow(allocations, &mut statistics);
            issues.extend(heap_issues);
        }

        // Detect stack buffer overflows
        if self.config.enable_stack_overflow_detection {
            let stack_issues = self.detect_stack_overflow(allocations, &mut statistics);
            issues.extend(stack_issues);
        }

        // Detect integer overflows
        if self.config.enable_integer_overflow_detection {
            let int_issues = self.detect_integer_overflow(allocations, &mut statistics);
            issues.extend(int_issues);
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

impl OverflowDetector {
    /// Detect buffer overflows (heap and stack)
    fn detect_buffer_overflow(
        &self,
        allocations: &[AllocationInfo],
        statistics: &mut DetectionStatistics,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();

        for (index, alloc) in allocations.iter().enumerate() {
            // Skip allocations smaller than minimum buffer size
            if alloc.size < self.config.min_buffer_size_bytes {
                continue;
            }

            // Check if this is a buffer type (array, slice, or vector)
            if self.is_buffer_type(alloc) {
                // Check for potential buffer overflow based on access patterns
                if let Some(access_tracking) = &alloc.access_tracking {
                    for event in &access_tracking.access_events {
                        // Check if access is outside buffer bounds
                        if event.address < alloc.ptr || event.address >= alloc.ptr + alloc.size {
                            let issue_id = format!("buffer_overflow_{}_{}", index, event.timestamp);
                            let severity = self.assess_overflow_severity(alloc, event);

                            let issue = Issue::new(
                                issue_id,
                                severity,
                                IssueCategory::Safety,
                                format!(
                                    "Buffer overflow detected at 0x{:x}: access at 0x{:x} exceeds buffer bounds [0x{:x}-0x{:x}]",
                                    alloc.ptr, event.address, alloc.ptr, alloc.ptr + alloc.size
                                ),
                            )
                            .with_allocation_ptr(alloc.ptr)
                            .with_suggested_fix(
                                "Use bounds-checked access methods (get, get_mut) or validate indices before access".to_string(),
                            );

                            issues.push(issue);
                            statistics.allocations_with_issues += 1;
                        }
                    }
                }

                // Check for suspicious write patterns
                if let Some(access_tracking) = &alloc.access_tracking {
                    let write_count = access_tracking
                        .access_events
                        .iter()
                        .filter(|e| {
                            matches!(
                                e.access_type,
                                crate::capture::types::MemoryAccessType::Write
                            )
                        })
                        .count();

                    // High frequency of writes might indicate overflow attempts
                    if write_count > 100 && write_count > access_tracking.access_events.len() / 2 {
                        let issue_id = format!("suspicious_write_pattern_{}", index);
                        let severity = IssueSeverity::Medium;

                        let issue = Issue::new(
                            issue_id,
                            severity,
                            IssueCategory::Safety,
                            format!(
                                "Suspicious write pattern detected at 0x{:x}: {}/{} writes",
                                alloc.ptr,
                                write_count,
                                access_tracking.access_events.len()
                            ),
                        )
                        .with_allocation_ptr(alloc.ptr)
                        .with_suggested_fix(
                            "Review write operations for potential buffer overflow".to_string(),
                        );

                        issues.push(issue);
                        statistics.allocations_with_issues += 1;
                    }
                }

                // Check for very large arrays that might cause stack overflow
                if alloc.size > self.config.max_array_size {
                    let issue_id = format!("large_array_{}", index);
                    let severity = IssueSeverity::High;

                    let issue = Issue::new(
                        issue_id,
                        severity,
                        IssueCategory::Safety,
                        format!(
                            "Large array detected at 0x{:x}: {} bytes (max: {} bytes)",
                            alloc.ptr, alloc.size, self.config.max_array_size
                        ),
                    )
                    .with_allocation_ptr(alloc.ptr)
                    .with_suggested_fix(
                        "Consider heap allocation (Vec, Box) instead of stack allocation"
                            .to_string(),
                    );

                    issues.push(issue);
                    statistics.allocations_with_issues += 1;
                }
            }
        }

        issues
    }

    /// Detect stack overflow
    fn detect_stack_overflow(
        &self,
        allocations: &[AllocationInfo],
        statistics: &mut DetectionStatistics,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();

        for (index, alloc) in allocations.iter().enumerate() {
            // Check if allocation is on stack
            if self.is_stack_allocation(alloc) {
                // Check for large stack allocations
                if alloc.size > 8 * 1024 {
                    // 8KB threshold
                    let issue_id = format!("large_stack_allocation_{}", index);
                    let severity = self.assess_stack_overflow_severity(alloc);

                    let issue = Issue::new(
                        issue_id,
                        severity,
                        IssueCategory::Safety,
                        format!(
                            "Large stack allocation detected at 0x{:x}: {} bytes",
                            alloc.ptr, alloc.size
                        ),
                    )
                    .with_allocation_ptr(alloc.ptr)
                    .with_suggested_fix(
                        "Consider heap allocation (Box, Vec) to avoid stack overflow".to_string(),
                    );

                    issues.push(issue);
                    statistics.allocations_with_issues += 1;
                }

                // Check for deep recursion pattern
                if let Some(lifecycle) = &alloc.lifecycle_tracking {
                    if lifecycle.lifecycle_events.len() > 100 {
                        let issue_id = format!("deep_recursion_{}", index);
                        let severity = IssueSeverity::High;

                        let issue = Issue::new(
                            issue_id,
                            severity,
                            IssueCategory::Safety,
                            format!(
                                "Deep recursion pattern detected: {} lifecycle events",
                                lifecycle.lifecycle_events.len()
                            ),
                        )
                        .with_allocation_ptr(alloc.ptr)
                        .with_suggested_fix(
                            "Consider iterative approach or increase stack size".to_string(),
                        );

                        issues.push(issue);
                        statistics.allocations_with_issues += 1;
                    }
                }
            }
        }

        issues
    }

    /// Detect integer overflow
    fn detect_integer_overflow(
        &self,
        allocations: &[AllocationInfo],
        statistics: &mut DetectionStatistics,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();

        for (index, alloc) in allocations.iter().enumerate() {
            // Check for potential integer overflow in type usage
            if let Some(type_name) = &alloc.type_name {
                if self.is_integer_type(type_name) {
                    // Check for suspicious large allocations that might overflow size calculations
                    if alloc.size > usize::MAX / 2 {
                        let issue_id = format!("integer_overflow_risk_{}", index);
                        let severity = IssueSeverity::Critical;

                        let issue = Issue::new(
                            issue_id,
                            severity,
                            IssueCategory::Safety,
                            format!(
                                "Integer overflow risk detected: allocation size {} bytes exceeds safe range",
                                alloc.size
                            ),
                        )
                        .with_allocation_ptr(alloc.ptr)
                            .with_suggested_fix(
                                "Use checked arithmetic methods (checked_add, checked_mul) or saturating operations".to_string(),
                            );

                        issues.push(issue);
                        statistics.allocations_with_issues += 1;
                    }

                    // Check for allocation near usize::MAX
                    if alloc.size > usize::MAX - 1024 {
                        let issue_id = format!("near_max_allocation_{}", index);
                        let severity = IssueSeverity::Critical;

                        let issue = Issue::new(
                            issue_id,
                            severity,
                            IssueCategory::Safety,
                            format!("Allocation near usize::MAX detected: {} bytes", alloc.size),
                        )
                        .with_allocation_ptr(alloc.ptr)
                        .with_suggested_fix(
                            "Reduce allocation size or use checked arithmetic".to_string(),
                        );

                        issues.push(issue);
                        statistics.allocations_with_issues += 1;
                    }
                }
            }

            // Check for suspicious size patterns in clone operations
            if let Some(clone_info) = &alloc.clone_info {
                if clone_info.clone_count > 0
                    && alloc.size * clone_info.clone_count > self.config.max_array_size
                {
                    let issue_id = format!("clone_overflow_risk_{}", index);
                    let severity = IssueSeverity::High;

                    let issue = Issue::new(
                        issue_id,
                        severity,
                        IssueCategory::Safety,
                        format!(
                            "Clone operation overflow risk: {} clones of {} bytes = {} bytes",
                            clone_info.clone_count,
                            alloc.size,
                            alloc.size * clone_info.clone_count
                        ),
                    )
                    .with_allocation_ptr(alloc.ptr)
                    .with_suggested_fix(
                        "Validate clone counts or use Arc for shared ownership".to_string(),
                    );

                    issues.push(issue);
                    statistics.allocations_with_issues += 1;
                }
            }
        }

        issues
    }

    /// Assess buffer overflow severity
    fn assess_overflow_severity(
        &self,
        alloc: &AllocationInfo,
        event: &crate::capture::types::MemoryAccessEvent,
    ) -> IssueSeverity {
        // Calculate distance from buffer bounds
        let start_distance = alloc.ptr.saturating_sub(event.address);
        let end_distance = event.address.saturating_sub(alloc.ptr + alloc.size);

        // Critical if far outside bounds (>1KB)
        if start_distance > 1024 || end_distance > 1024 {
            return IssueSeverity::Critical;
        }

        // High if write access outside bounds
        if matches!(
            event.access_type,
            crate::capture::types::MemoryAccessType::Write
        ) {
            return IssueSeverity::High;
        }

        // Medium if read access outside bounds
        IssueSeverity::Medium
    }

    /// Assess stack overflow severity
    fn assess_stack_overflow_severity(&self, alloc: &AllocationInfo) -> IssueSeverity {
        // Critical if >1MB stack allocation
        if alloc.size > 1024 * 1024 {
            return IssueSeverity::Critical;
        }

        // High if >256KB stack allocation
        if alloc.size > 256 * 1024 {
            return IssueSeverity::High;
        }

        // Medium if >64KB stack allocation
        IssueSeverity::Medium
    }

    /// Check if allocation is a buffer type
    fn is_buffer_type(&self, alloc: &AllocationInfo) -> bool {
        alloc
            .type_name
            .as_ref()
            .map(|t| {
                t.contains('[') || t.contains("Vec") || t.contains("slice") || t.contains("array")
            })
            .unwrap_or(false)
    }

    /// Check if allocation is on stack
    fn is_stack_allocation(&self, alloc: &AllocationInfo) -> bool {
        alloc.stack_allocation.is_some()
    }

    /// Check if type is an integer type
    fn is_integer_type(&self, type_name: &str) -> bool {
        type_name.contains("i8")
            || type_name.contains("i16")
            || type_name.contains("i32")
            || type_name.contains("i64")
            || type_name.contains("i128")
            || type_name.contains("isize")
            || type_name.contains("u8")
            || type_name.contains("u16")
            || type_name.contains("u32")
            || type_name.contains("u64")
            || type_name.contains("u128")
            || type_name.contains("usize")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::types::AllocationInfo;
    use crate::capture::types::LocalityMetrics;

    #[test]
    fn test_overflow_detector_creation() {
        let config = OverflowDetectorConfig::default();
        let detector = OverflowDetector::new(config);

        assert_eq!(detector.name(), "OverflowDetector");
        assert_eq!(detector.version(), "1.0.0");
    }

    #[test]
    fn test_overflow_detector_config() {
        let config = OverflowDetectorConfig {
            enable_heap_overflow_detection: false,
            enable_stack_overflow_detection: false,
            enable_integer_overflow_detection: false,
            min_buffer_size_bytes: 128,
            max_array_size: 2048,
        };

        let detector = OverflowDetector::new(config);
        let overflow_config = detector.overflow_config();

        assert!(!overflow_config.enable_heap_overflow_detection);
        assert!(!overflow_config.enable_stack_overflow_detection);
        assert!(!overflow_config.enable_integer_overflow_detection);
        assert_eq!(overflow_config.min_buffer_size_bytes, 128);
        assert_eq!(overflow_config.max_array_size, 2048);
    }

    #[test]
    fn test_overflow_detector_detect() {
        let config = OverflowDetectorConfig::default();
        let detector = OverflowDetector::new(config);

        let allocations = vec![
            AllocationInfo::new(0x1000, 1024),
            AllocationInfo::new(0x2000, 2048),
        ];

        let result = detector.detect(&allocations);

        assert_eq!(result.detector_name, "OverflowDetector");
        assert_eq!(result.statistics.total_allocations, 2);
    }

    #[test]
    fn test_detect_buffer_overflow() {
        let config = OverflowDetectorConfig::default();
        let detector = OverflowDetector::new(config);

        let mut allocations = vec![AllocationInfo::new(0x1000, 1024)];
        allocations[0].type_name = Some("Vec<i32>".to_string());

        // Create access tracking with out-of-bounds access
        use crate::capture::types::{
            AddressRange, CacheAccessInfo, CacheLatencyBreakdown, MemoryAccessEvent,
            MemoryAccessPerformanceImpact, MemoryAccessTrackingInfo, MemoryAccessType,
        };

        allocations[0].access_tracking = Some(MemoryAccessTrackingInfo {
            region_id: 0,
            address_range: AddressRange {
                start_address: 0x1000,
                end_address: 0x1000 + 1024,
                size: 1024,
            },
            access_events: vec![
                MemoryAccessEvent {
                    access_type: MemoryAccessType::Read,
                    timestamp: 100,
                    address: 0x1000, // Valid access
                    size: 4,
                    function_name: "test".to_string(),
                    latency_ns: 100,
                    cache_info: CacheAccessInfo {
                        l1_hit: true,
                        l2_hit: false,
                        l3_hit: false,
                        memory_access: false,
                        latency_breakdown: CacheLatencyBreakdown {
                            l1_latency_ns: 1.0,
                            l2_latency_ns: 5.0,
                            l3_latency_ns: 20.0,
                            memory_latency_ns: 100.0,
                        },
                    },
                },
                MemoryAccessEvent {
                    access_type: MemoryAccessType::Write,
                    timestamp: 200,
                    address: 0x2000, // Out of bounds
                    size: 4,
                    function_name: "test".to_string(),
                    latency_ns: 100,
                    cache_info: CacheAccessInfo {
                        l1_hit: true,
                        l2_hit: false,
                        l3_hit: false,
                        memory_access: false,
                        latency_breakdown: CacheLatencyBreakdown {
                            l1_latency_ns: 1.0,
                            l2_latency_ns: 5.0,
                            l3_latency_ns: 20.0,
                            memory_latency_ns: 100.0,
                        },
                    },
                },
            ],
            access_statistics: crate::capture::types::MemoryAccessStatistics {
                total_reads: 1,
                total_writes: 1,
                read_write_ratio: 1.0,
                avg_access_frequency: 10.0,
                peak_access_frequency: 20.0,
                locality_metrics: LocalityMetrics {
                    temporal_locality: 0.8,
                    spatial_locality: 0.9,
                    sequential_access_percent: 95.0,
                    random_access_percent: 5.0,
                    stride_patterns: vec![],
                },
                bandwidth_utilization: crate::capture::types::BandwidthUtilization {
                    peak_bandwidth: 100.0,
                    avg_bandwidth: 50.0,
                    efficiency_percent: 80.0,
                    bottlenecks: vec![],
                },
            },
            access_patterns: vec![],
            performance_impact: MemoryAccessPerformanceImpact {
                performance_score: 0.9,
                cache_efficiency_impact: 0.1,
                bandwidth_impact: 0.05,
                pipeline_impact: 0.03,
                optimization_recommendations: vec![],
            },
        });

        let issues = detector.detect_buffer_overflow(&allocations, &mut DetectionStatistics::new());

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.description.contains("Buffer overflow detected")));
    }

    #[test]
    fn test_detect_large_stack_allocation() {
        let config = OverflowDetectorConfig::default();
        let detector = OverflowDetector::new(config);

        let mut allocations = vec![AllocationInfo::new(0x1000, 16 * 1024)]; // 16KB stack allocation
        allocations[0].type_name = Some("[u8; 16384]".to_string());

        use crate::capture::types::{ScopeType, StackAllocationInfo, StackScopeInfo};
        allocations[0].stack_allocation = Some(StackAllocationInfo {
            frame_id: 0,
            var_name: "buffer".to_string(),
            stack_offset: 1024,
            size: 16 * 1024,
            function_name: "test_function".to_string(),
            stack_depth: 1,
            scope_info: StackScopeInfo {
                scope_type: ScopeType::Function,
                start_line: Some(10),
                end_line: Some(50),
                parent_scope: None,
                nesting_level: 0,
            },
        });

        let issues = detector.detect_stack_overflow(&allocations, &mut DetectionStatistics::new());

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.description.contains("Large stack allocation")));
    }

    #[test]
    fn test_detect_integer_overflow_risk() {
        let config = OverflowDetectorConfig::default();
        let detector = OverflowDetector::new(config);

        let mut allocations = vec![AllocationInfo::new(0x1000, usize::MAX / 2 + 1)];
        allocations[0].type_name = Some("Vec<i32>".to_string());

        let issues =
            detector.detect_integer_overflow(&allocations, &mut DetectionStatistics::new());

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.description.contains("Integer overflow risk")));
    }

    #[test]
    fn test_is_buffer_type() {
        let config = OverflowDetectorConfig::default();
        let detector = OverflowDetector::new(config);

        let mut alloc = AllocationInfo::new(0x1000, 1024);
        alloc.type_name = Some("Vec<i32>".to_string());
        assert!(detector.is_buffer_type(&alloc));

        alloc.type_name = Some("[u8; 1024]".to_string());
        assert!(detector.is_buffer_type(&alloc));

        alloc.type_name = Some("Box<i32>".to_string());
        assert!(!detector.is_buffer_type(&alloc));
    }

    #[test]
    fn test_is_integer_type() {
        let config = OverflowDetectorConfig::default();
        let detector = OverflowDetector::new(config);

        assert!(detector.is_integer_type("i32"));
        assert!(detector.is_integer_type("u64"));
        assert!(detector.is_integer_type("usize"));
        assert!(!detector.is_integer_type("f32"));
        assert!(!detector.is_integer_type("String"));
    }

    #[test]
    fn test_overflow_assessment_severity() {
        let config = OverflowDetectorConfig::default();
        let detector = OverflowDetector::new(config);

        let alloc = AllocationInfo::new(0x1000, 1024);

        use crate::capture::types::{
            CacheAccessInfo, CacheLatencyBreakdown, MemoryAccessEvent, MemoryAccessType,
        };
        let event = MemoryAccessEvent {
            access_type: MemoryAccessType::Write,
            timestamp: 100,
            address: 0x3000, // Far outside bounds
            size: 4,
            function_name: "test".to_string(),
            latency_ns: 100,
            cache_info: CacheAccessInfo {
                l1_hit: true,
                l2_hit: false,
                l3_hit: false,
                memory_access: false,
                latency_breakdown: CacheLatencyBreakdown {
                    l1_latency_ns: 1.0,
                    l2_latency_ns: 5.0,
                    l3_latency_ns: 20.0,
                    memory_latency_ns: 100.0,
                },
            },
        };

        let severity = detector.assess_overflow_severity(&alloc, &event);
        assert_eq!(severity, IssueSeverity::Critical);
    }

    #[test]
    fn test_overflow_detector_disabled() {
        let config = OverflowDetectorConfig {
            enable_heap_overflow_detection: false,
            enable_stack_overflow_detection: false,
            enable_integer_overflow_detection: false,
            min_buffer_size_bytes: 64,
            max_array_size: 1024 * 1024,
        };
        let detector = OverflowDetector::new(config);

        let allocations = vec![AllocationInfo::new(0x1000, 1024)];
        let result = detector.detect(&allocations);

        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_detect_large_array() {
        let config = OverflowDetectorConfig::default();
        let detector = OverflowDetector::new(config);

        let mut allocations = vec![AllocationInfo::new(0x1000, 2 * 1024 * 1024)]; // 2MB array
        allocations[0].type_name = Some("Vec<i32>".to_string());

        let issues = detector.detect_buffer_overflow(&allocations, &mut DetectionStatistics::new());

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.description.contains("Large array detected")));
    }

    #[test]
    fn test_clone_overflow_risk() {
        let config = OverflowDetectorConfig::default();
        let detector = OverflowDetector::new(config);

        let mut allocations = vec![AllocationInfo::new(0x1000, 512 * 1024)]; // 512KB
        allocations[0].type_name = Some("Vec<i32>".to_string());

        use crate::capture::types::CloneInfo;
        allocations[0].clone_info = Some(CloneInfo {
            clone_count: 3, // 3 * 512KB = 1.5MB > max_array_size
            is_clone: true,
            original_ptr: Some(0x1000),
        });

        let issues =
            detector.detect_integer_overflow(&allocations, &mut DetectionStatistics::new());

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.description.contains("Clone operation overflow risk")));
    }
}
