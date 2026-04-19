//! Double-free detection module
//!
//! Detects attempts to free the same memory address twice, which is a critical
//! memory safety issue that can lead to crashes or security vulnerabilities.

use crate::analysis::detectors::{
    DetectionResult, DetectionStatistics, Detector, DetectorConfig, DetectorError, Issue,
    IssueCategory, Location,
};
use crate::capture::types::AllocationInfo;
use crate::event_store::event::{MemoryEvent, MemoryEventType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoubleFreeDetector {
    config: DoubleFreeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoubleFreeConfig {
    pub enable_early_detection: bool,
}

impl Default for DoubleFreeConfig {
    fn default() -> Self {
        Self {
            enable_early_detection: true,
        }
    }
}

impl DoubleFreeDetector {
    pub fn new(config: DoubleFreeConfig) -> Self {
        Self { config }
    }
}

impl Default for DoubleFreeDetector {
    fn default() -> Self {
        Self::new(DoubleFreeConfig::default())
    }
}

impl Detector for DoubleFreeDetector {
    fn name(&self) -> &str {
        "DoubleFreeDetector"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        let start_time = std::time::Instant::now();
        let mut issues = Vec::new();
        let mut freed_pointers: HashSet<usize> = HashSet::new();
        let mut statistics = DetectionStatistics::new();

        for alloc in allocations {
            statistics.total_allocations += 1;
            statistics.total_memory_analyzed += alloc.size;

            if alloc.timestamp_dealloc.is_some() {
                if freed_pointers.contains(&alloc.ptr) {
                    issues.push(
                        Issue::critical(
                            format!("DF-{:08x}", alloc.ptr),
                            IssueCategory::Concurrency,
                            format!(
                                "Double-free detected: pointer {:x} freed twice (size: {})",
                                alloc.ptr, alloc.size
                            ),
                        )
                        .with_allocation_ptr(alloc.ptr)
                        .with_suggested_fix(
                            "Ensure each allocation is freed exactly once. Consider using Arc or Rc for shared ownership.".to_string(),
                        ),
                    );
                    statistics.memory_affected += alloc.size;
                } else {
                    freed_pointers.insert(alloc.ptr);
                }
            }
        }

        statistics.allocations_with_issues = issues.len();
        for issue in &issues {
            statistics.update_severity(issue.severity);
            statistics.update_category(issue.category);
        }

        DetectionResult {
            detector_name: self.name().to_string(),
            issues,
            statistics,
            detection_time_ms: start_time.elapsed().as_millis() as u64,
        }
    }

    fn config(&self) -> &DetectorConfig {
        unimplemented!("Use DoubleFreeConfig directly")
    }

    fn update_config(&mut self, _config: DetectorConfig) -> Result<(), DetectorError> {
        Ok(())
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DoubleFreeDetectorWithEvents {
    config: DoubleFreeConfig,
}

impl DoubleFreeDetectorWithEvents {
    pub fn new() -> Self {
        Self {
            config: DoubleFreeConfig::default(),
        }
    }

    pub fn detect_from_events(&self, events: &[MemoryEvent]) -> DetectionResult {
        let start_time = std::time::Instant::now();
        let mut issues = Vec::new();
        let mut freed_pointers: HashMap<usize, (u64, Option<String>, Option<u32>)> = HashMap::new();
        let mut statistics = DetectionStatistics::new();

        for event in events {
            if event.event_type == MemoryEventType::Deallocate {
                statistics.total_allocations += 1;

                if let Some((first_time, _, _)) = freed_pointers.get(&event.ptr) {
                    issues.push(
                        Issue::critical(
                            format!("DF-{:08x}", event.ptr),
                            IssueCategory::Concurrency,
                            format!(
                                "Double-free detected: pointer {:x} freed twice (first_free_time: {}, second_free_time: {})",
                                event.ptr, first_time, event.timestamp
                            ),
                        )
                        .with_allocation_ptr(event.ptr)
                        .with_location(Location::new(
                            event.source_file.as_deref().unwrap_or("unknown").to_string(),
                        ))
                        .with_suggested_fix(
                            "Ensure each allocation is freed exactly once.".to_string(),
                        ),
                    );
                    statistics.memory_affected += event.size;
                } else {
                    freed_pointers.insert(
                        event.ptr,
                        (event.timestamp, event.source_file.clone(), event.source_line),
                    );
                }
            }
        }

        statistics.allocations_with_issues = issues.len();
        for issue in &issues {
            statistics.update_severity(issue.severity);
            statistics.update_category(issue.category);
        }

        DetectionResult {
            detector_name: "DoubleFreeDetector".to_string(),
            issues,
            statistics,
            detection_time_ms: start_time.elapsed().as_millis() as u64,
        }
    }
}

impl Default for DoubleFreeDetectorWithEvents {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for DoubleFreeDetectorWithEvents {
    fn name(&self) -> &str {
        "DoubleFreeDetector"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        let start_time = std::time::Instant::now();
        let mut issues = Vec::new();
        let mut freed_pointers: HashSet<usize> = HashSet::new();
        let mut statistics = DetectionStatistics::new();

        for alloc in allocations {
            statistics.total_allocations += 1;
            if alloc.timestamp_dealloc.is_some() {
                if freed_pointers.contains(&alloc.ptr) {
                    issues.push(Issue::critical(
                        format!("DF-{:08x}", alloc.ptr),
                        IssueCategory::Concurrency,
                        format!("Double-free detected: pointer {:x}", alloc.ptr),
                    ));
                    statistics.memory_affected += alloc.size;
                } else {
                    freed_pointers.insert(alloc.ptr);
                }
            }
        }

        statistics.allocations_with_issues = issues.len();

        DetectionResult {
            detector_name: self.name().to_string(),
            issues,
            statistics,
            detection_time_ms: start_time.elapsed().as_millis() as u64,
        }
    }

    fn config(&self) -> &DetectorConfig {
        unimplemented!("Use DoubleFreeConfig directly")
    }

    fn update_config(&mut self, _config: DetectorConfig) -> Result<(), DetectorError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_free_detection() {
        let detector = DoubleFreeDetector::new(DoubleFreeConfig::default());

        let mut alloc1 = AllocationInfo::new(0x1000, 1024);
        alloc1.timestamp_dealloc = Some(1000);

        let mut alloc2 = AllocationInfo::new(0x1000, 1024);
        alloc2.timestamp_dealloc = Some(2000);

        let result = detector.detect(&[alloc1, alloc2]);

        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].id, "DF-00001000");
    }

    #[test]
    fn test_no_false_positives() {
        let detector = DoubleFreeDetector::new(DoubleFreeConfig::default());

        let alloc1 = AllocationInfo::new(0x1000, 1024);
        let alloc2 = AllocationInfo::new(0x2000, 1024);

        let result = detector.detect(&[alloc1, alloc2]);

        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_single_free() {
        let detector = DoubleFreeDetector::new(DoubleFreeConfig::default());

        let alloc = AllocationInfo::new(0x1000, 1024);

        let result = detector.detect(&[alloc]);

        assert_eq!(result.issues.len(), 0);
    }
}