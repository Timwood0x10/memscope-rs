//! Timestamp-based lifecycle analysis
//!
//! This module provides analysis of object lifetimes based on existing
//! timestamp_alloc and timestamp_dealloc data from AllocationInfo.

use crate::capture::types::AllocationInfo;
use std::collections::HashMap;

/// Lifecycle analyzer using timestamp data
pub struct LifecycleAnalyzer {
    allocations: Vec<AllocationInfo>,
}

impl LifecycleAnalyzer {
    /// Create a new lifecycle analyzer from allocation data
    pub fn new(allocations: Vec<AllocationInfo>) -> Self {
        Self { allocations }
    }

    /// Get lifetime distribution by time buckets
    pub fn lifetime_distribution(&self) -> Vec<(String, usize)> {
        let mut distribution = HashMap::new();
        for alloc in &self.allocations {
            if let Some(dealloc_ts) = alloc.timestamp_dealloc {
                let lifetime_ms = dealloc_ts - alloc.timestamp_alloc;
                let bucket = self.lifetime_bucket(lifetime_ms);
                *distribution.entry(bucket).or_insert(0) += 1;
            }
        }
        let mut result: Vec<_> = distribution.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    /// Detect temporary objects (short-lived allocations)
    ///
    /// Returns allocations that lived less than the threshold
    pub fn detect_temporary_objects(&self, threshold_ms: u64) -> Vec<&AllocationInfo> {
        self.allocations
            .iter()
            .filter(|alloc| {
                if let Some(dealloc_ts) = alloc.timestamp_dealloc {
                    (dealloc_ts - alloc.timestamp_alloc) < threshold_ms
                } else {
                    false
                }
            })
            .collect()
    }

    /// Detect long-lived objects (potential memory leaks)
    ///
    /// Returns allocations that have lived longer than threshold and are not deallocated
    pub fn detect_long_lived_objects(&self, threshold_ms: u64) -> Vec<&AllocationInfo> {
        let current = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        self.allocations
            .iter()
            .filter(|alloc| {
                (current - alloc.timestamp_alloc) > threshold_ms
                    && alloc.timestamp_dealloc.is_none()
            })
            .collect()
    }

    /// Find allocation hotspots by call stack
    pub fn allocation_hotspots(&self) -> Vec<(String, usize)> {
        let mut hotspots = HashMap::new();
        for alloc in &self.allocations {
            if let Some(ref stack) = alloc.stack_trace {
                let key = stack
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                *hotspots.entry(key).or_insert(0) += 1;
            }
        }
        let mut result: Vec<_> = hotspots.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    /// Calculate average lifetime of deallocated allocations
    pub fn average_lifetime_ms(&self) -> Option<f64> {
        let mut total_lifetime = 0u64;
        let mut count = 0usize;

        for alloc in &self.allocations {
            if let Some(dealloc_ts) = alloc.timestamp_dealloc {
                total_lifetime += dealloc_ts - alloc.timestamp_alloc;
                count += 1;
            }
        }

        if count > 0 {
            Some(total_lifetime as f64 / count as f64)
        } else {
            None
        }
    }

    /// Get statistics summary
    pub fn statistics(&self) -> LifecycleStatistics {
        let total_allocations = self.allocations.len();
        let deallocated = self
            .allocations
            .iter()
            .filter(|a| a.timestamp_dealloc.is_some())
            .count();
        let active = total_allocations - deallocated;

        LifecycleStatistics {
            total_allocations,
            deallocated,
            active,
            average_lifetime_ms: self.average_lifetime_ms(),
        }
    }

    fn lifetime_bucket(&self, ms: u64) -> String {
        if ms < 1 {
            "<1ms".to_string()
        } else if ms < 10 {
            "1-10ms".to_string()
        } else if ms < 100 {
            "10-100ms".to_string()
        } else if ms < 1000 {
            "100ms-1s".to_string()
        } else if ms < 10000 {
            "1-10s".to_string()
        } else {
            ">10s".to_string()
        }
    }
}

/// Lifecycle statistics summary
#[derive(Debug, Clone)]
pub struct LifecycleStatistics {
    pub total_allocations: usize,
    pub deallocated: usize,
    pub active: usize,
    pub average_lifetime_ms: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifetime_bucket() {
        let analyzer = LifecycleAnalyzer::new(vec![]);
        assert_eq!(analyzer.lifetime_bucket(0), "<1ms");
        assert_eq!(analyzer.lifetime_bucket(5), "1-10ms");
        assert_eq!(analyzer.lifetime_bucket(50), "10-100ms");
        assert_eq!(analyzer.lifetime_bucket(500), "100ms-1s");
        assert_eq!(analyzer.lifetime_bucket(5000), "1-10s");
        assert_eq!(analyzer.lifetime_bucket(20000), ">10s");
    }

    #[test]
    fn test_lifetime_distribution_empty() {
        let analyzer = LifecycleAnalyzer::new(vec![]);
        let distribution = analyzer.lifetime_distribution();
        assert!(distribution.is_empty());
    }

    #[test]
    fn test_detect_temporary_objects_empty() {
        let analyzer = LifecycleAnalyzer::new(vec![]);
        let temp = analyzer.detect_temporary_objects(100);
        assert!(temp.is_empty());
    }

    #[test]
    fn test_detect_long_lived_objects_empty() {
        let analyzer = LifecycleAnalyzer::new(vec![]);
        let long = analyzer.detect_long_lived_objects(1000);
        assert!(long.is_empty());
    }

    #[test]
    fn test_allocation_hotspots_empty() {
        let analyzer = LifecycleAnalyzer::new(vec![]);
        let hotspots = analyzer.allocation_hotspots();
        assert!(hotspots.is_empty());
    }

    #[test]
    fn test_average_lifetime_empty() {
        let analyzer = LifecycleAnalyzer::new(vec![]);
        assert!(analyzer.average_lifetime_ms().is_none());
    }

    #[test]
    fn test_statistics_empty() {
        let analyzer = LifecycleAnalyzer::new(vec![]);
        let stats = analyzer.statistics();
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.deallocated, 0);
        assert_eq!(stats.active, 0);
        assert!(stats.average_lifetime_ms.is_none());
    }
}
