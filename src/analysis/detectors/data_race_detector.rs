//! Data race detection module
//!
//! Detects potential data races by analyzing memory access patterns across threads.
//! A data race occurs when two or more threads access shared memory concurrently,
//! and at least one access is a write.

use crate::analysis::detectors::{
    DetectionResult, DetectionStatistics, Detector, DetectorConfig, DetectorError, Issue,
    IssueCategory,
};
use crate::capture::types::AllocationInfo;
use crate::event_store::event::{MemoryEvent, MemoryEventType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRaceDetector {
    config: DataRaceConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryAccessType {
    Read,
    Write,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRecord {
    pub thread_id: u64,
    pub access_type: MemoryAccessType,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRaceConfig {
    pub time_window_ns: u64,
    pub min_threads: usize,
}

impl Default for DataRaceConfig {
    fn default() -> Self {
        Self {
            time_window_ns: 1000,
            min_threads: 2,
        }
    }
}

impl DataRaceDetector {
    pub fn new(config: DataRaceConfig) -> Self {
        Self { config }
    }

    fn check_race_on_pointer(&self, ptr: usize, accesses: &[AccessRecord]) -> Option<Issue> {
        if accesses.len() < 2 {
            return None;
        }

        let mut sorted_accesses = accesses.to_vec();
        sorted_accesses.sort_by_key(|a| a.timestamp);

        let threads_accessing: HashSet<u64> = sorted_accesses.iter().map(|a| a.thread_id).collect();
        if threads_accessing.len() < self.config.min_threads {
            return None;
        }

        for i in 0..sorted_accesses.len() {
            for j in (i + 1)..sorted_accesses.len() {
                let access1 = &sorted_accesses[i];
                let access2 = &sorted_accesses[j];

                let time_diff = access2.timestamp.saturating_sub(access1.timestamp);
                if time_diff > self.config.time_window_ns {
                    break;
                }

                if access1.thread_id != access2.thread_id {
                    if access1.access_type == MemoryAccessType::Write
                        || access2.access_type == MemoryAccessType::Write
                    {
                        return Some(Issue::critical(
                            format!("DR-{:08x}", ptr),
                            IssueCategory::Concurrency,
                            format!(
                                "Potential data race on pointer {:x}: thread {} and thread {} accessed concurrently within {}ns",
                                ptr, access1.thread_id, access2.thread_id, time_diff
                            ),
                        )
                        .with_allocation_ptr(ptr)
                        .with_suggested_fix(
                            "Use proper synchronization (Mutex, RwLock, atomic operations) or ensure proper memory ordering.".to_string(),
                        ));
                    }
                }
            }
        }

        None
    }
}

impl Default for DataRaceDetector {
    fn default() -> Self {
        Self::new(DataRaceConfig::default())
    }
}

impl Detector for DataRaceDetector {
    fn name(&self) -> &str {
        "DataRaceDetector"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        let start_time = std::time::Instant::now();
        let mut issues = Vec::new();
        let statistics = DetectionStatistics::new();

        let mut pointer_accesses: HashMap<usize, Vec<AccessRecord>> = HashMap::new();

        for alloc in allocations {
            let access = AccessRecord {
                thread_id: alloc.thread_id_u64,
                access_type: if alloc.timestamp_dealloc.is_none() {
                    MemoryAccessType::Write
                } else {
                    MemoryAccessType::Read
                },
                timestamp: alloc.timestamp_alloc,
            };

            pointer_accesses.entry(alloc.ptr).or_default().push(access);
        }

        for (ptr, accesses) in &pointer_accesses {
            if let Some(issue) = self.check_race_on_pointer(*ptr, accesses) {
                issues.push(issue);
            }
        }

        DetectionResult {
            detector_name: self.name().to_string(),
            issues,
            statistics,
            detection_time_ms: start_time.elapsed().as_millis() as u64,
        }
    }

    fn config(&self) -> &DetectorConfig {
        unimplemented!("Use DataRaceConfig directly")
    }

    fn update_config(&mut self, _config: DetectorConfig) -> Result<(), DetectorError> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct DataRaceDetectorWithEvents {
    config: DataRaceConfig,
}

impl DataRaceDetectorWithEvents {
    pub fn new() -> Self {
        Self {
            config: DataRaceConfig::default(),
        }
    }

    pub fn detect_from_events(&self, events: &[MemoryEvent]) -> DetectionResult {
        let start_time = std::time::Instant::now();
        let mut issues = Vec::new();
        let statistics = DetectionStatistics::new();

        let mut pointer_accesses: HashMap<usize, Vec<AccessRecord>> = HashMap::new();

        for event in events {
            if event.event_type == MemoryEventType::Allocate || event.event_type == MemoryEventType::Deallocate {
                let access_type = match event.event_type {
                    MemoryEventType::Allocate => MemoryAccessType::Write,
                    _ => MemoryAccessType::Write,
                };

                let record = AccessRecord {
                    thread_id: event.thread_id,
                    access_type,
                    timestamp: event.timestamp,
                };

                pointer_accesses.entry(event.ptr).or_default().push(record);
            }
        }

        let detector = DataRaceDetector::new(self.config.clone());
        for (ptr, accesses) in &pointer_accesses {
            if let Some(issue) = detector.check_race_on_pointer(*ptr, accesses) {
                issues.push(issue);
            }
        }

        DetectionResult {
            detector_name: "DataRaceDetector".to_string(),
            issues,
            statistics,
            detection_time_ms: start_time.elapsed().as_millis() as u64,
        }
    }
}

impl Default for DataRaceDetectorWithEvents {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for DataRaceDetectorWithEvents {
    fn name(&self) -> &str {
        "DataRaceDetector"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        let start_time = std::time::Instant::now();
        let mut issues = Vec::new();
        let statistics = DetectionStatistics::new();

        let mut pointer_accesses: HashMap<usize, Vec<AccessRecord>> = HashMap::new();

        for alloc in allocations {
            let record = AccessRecord {
                thread_id: alloc.thread_id_u64,
                access_type: MemoryAccessType::Write,
                timestamp: alloc.timestamp_alloc,
            };

            pointer_accesses.entry(alloc.ptr).or_default().push(record);
        }

        let detector = DataRaceDetector::new(self.config.clone());
        for (ptr, accesses) in &pointer_accesses {
            if let Some(issue) = detector.check_race_on_pointer(*ptr, accesses) {
                issues.push(issue);
            }
        }

        DetectionResult {
            detector_name: self.name().to_string(),
            issues,
            statistics,
            detection_time_ms: start_time.elapsed().as_millis() as u64,
        }
    }

    fn config(&self) -> &DetectorConfig {
        unimplemented!("Use DataRaceConfig directly")
    }

    fn update_config(&mut self, _config: DetectorConfig) -> Result<(), DetectorError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_race_single_thread() {
        let detector = DataRaceDetector::new(DataRaceConfig::default());

        let alloc1 = AllocationInfo::new(0x1000, 1024);
        let alloc2 = AllocationInfo::new(0x1000, 1024);

        let result = detector.detect(&[alloc1, alloc2]);

        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_different_pointers_no_race() {
        let detector = DataRaceDetector::new(DataRaceConfig::default());

        let alloc1 = AllocationInfo::new(0x1000, 1024);
        let alloc2 = AllocationInfo::new(0x2000, 1024);

        let result = detector.detect(&[alloc1, alloc2]);

        assert_eq!(result.issues.len(), 0);
    }
}