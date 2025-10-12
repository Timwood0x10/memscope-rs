use super::{PointerType, SmartPointerTracker};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum LeakPattern {
    CircularReference,
    WeakReferenceHold,
    LongLivedBox,
    HighRefCount,
    SynchronizationOveruse,
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub total_active_pointers: usize,
    pub memory_usage_by_type: HashMap<PointerType, usize>,
    pub detected_patterns: Vec<(LeakPattern, Vec<u64>)>,
    pub recommendations: Vec<String>,
    pub health_score: f64,
}

pub struct SmartPointerAnalyzer {
    leak_thresholds: LeakThresholds,
}

#[derive(Debug)]
struct LeakThresholds {
    long_lived_secs: u64,
    high_ref_count: usize,
    max_sync_objects: usize,
    circular_ref_timeout: Duration,
}

impl Default for LeakThresholds {
    fn default() -> Self {
        Self {
            long_lived_secs: 3600, // 1 hour
            high_ref_count: 10,
            max_sync_objects: 100,
            circular_ref_timeout: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl SmartPointerAnalyzer {
    pub fn new() -> Self {
        Self {
            leak_thresholds: LeakThresholds::default(),
        }
    }

    pub fn with_thresholds(
        long_lived_secs: u64,
        high_ref_count: usize,
        max_sync_objects: usize,
    ) -> Self {
        Self {
            leak_thresholds: LeakThresholds {
                long_lived_secs,
                high_ref_count,
                max_sync_objects,
                circular_ref_timeout: Duration::from_secs(300),
            },
        }
    }

    pub fn analyze(&self, tracker: &SmartPointerTracker) -> AnalysisResult {
        let total_active_pointers = tracker.get_active_count();
        let memory_usage_by_type = tracker.get_memory_usage_by_type();
        let mut detected_patterns = Vec::new();
        let mut recommendations = Vec::new();

        // Detect long-lived Box allocations
        let long_lived_boxes = self.detect_long_lived_boxes(tracker);
        if !long_lived_boxes.is_empty() {
            detected_patterns.push((LeakPattern::LongLivedBox, long_lived_boxes));
            recommendations.push(
                "Consider using Rc/Arc for shared data instead of long-lived Box".to_string(),
            );
        }

        // Detect high reference counts
        let high_ref_counts = self.detect_high_ref_counts(tracker);
        if !high_ref_counts.is_empty() {
            detected_patterns.push((LeakPattern::HighRefCount, high_ref_counts));
            recommendations.push(
                "Review objects with high reference counts for potential circular references"
                    .to_string(),
            );
        }

        // Detect synchronization overuse
        let sync_overuse = self.detect_sync_overuse(tracker);
        if !sync_overuse.is_empty() {
            detected_patterns.push((LeakPattern::SynchronizationOveruse, sync_overuse));
            recommendations.push(
                "Consider reducing synchronization overhead or using lock-free alternatives"
                    .to_string(),
            );
        }

        // Detect circular references using enhanced heuristics
        let circular_refs = self.detect_circular_references(tracker);
        if !circular_refs.is_empty() {
            detected_patterns.push((LeakPattern::CircularReference, circular_refs));
            recommendations.push(
                "Circular reference detected - consider using Weak references to break cycles"
                    .to_string(),
            );
        }

        // Detect weak reference holds
        let weak_holds = self.detect_weak_reference_holds(tracker);
        if !weak_holds.is_empty() {
            detected_patterns.push((LeakPattern::WeakReferenceHold, weak_holds));
            recommendations
                .push("Check if weak references are being held longer than necessary".to_string());
        }

        let health_score = self.calculate_health_score(&detected_patterns, total_active_pointers);

        AnalysisResult {
            total_active_pointers,
            memory_usage_by_type,
            detected_patterns,
            recommendations,
            health_score,
        }
    }

    fn detect_long_lived_boxes(&self, tracker: &SmartPointerTracker) -> Vec<u64> {
        tracker
            .find_long_lived_pointers(self.leak_thresholds.long_lived_secs)
            .iter()
            .filter(|info| info.ptr_type == PointerType::Box)
            .map(|info| info.allocation_id)
            .collect()
    }

    fn detect_high_ref_counts(&self, tracker: &SmartPointerTracker) -> Vec<u64> {
        let mut high_ref_ids = Vec::new();

        for ptr_type in &[PointerType::Rc, PointerType::Arc] {
            let pointers = tracker.get_active_by_type(ptr_type);
            for info in pointers {
                if let Some(ref_count) = info.ref_count {
                    // Enhanced detection: lower threshold for better recall
                    if ref_count >= self.leak_thresholds.high_ref_count
                        || (ref_count >= 3 && self.is_likely_circular_reference(info, tracker))
                    {
                        high_ref_ids.push(info.allocation_id);
                    }
                }
            }
        }

        high_ref_ids
    }

    fn detect_sync_overuse(&self, tracker: &SmartPointerTracker) -> Vec<u64> {
        let sync_types = [PointerType::Mutex, PointerType::RwLock, PointerType::Arc];
        let mut sync_count = 0;
        let mut overuse_ids = Vec::new();

        for sync_type in &sync_types {
            let pointers = tracker.get_active_by_type(sync_type);
            sync_count += pointers.len();

            if sync_count > self.leak_thresholds.max_sync_objects {
                overuse_ids.extend(pointers.iter().map(|info| info.allocation_id));
            }
        }

        overuse_ids
    }

    fn detect_weak_reference_holds(&self, tracker: &SmartPointerTracker) -> Vec<u64> {
        tracker
            .get_active_by_type(&PointerType::Weak)
            .iter()
            .filter(|info| {
                // Enhanced weak reference leak detection
                info.age() > self.leak_thresholds.circular_ref_timeout ||
                // Also detect weak refs that might be part of circular patterns
                (info.age() > Duration::from_secs(30) && self.is_suspicious_weak_ref(info, tracker))
            })
            .map(|info| info.allocation_id)
            .collect()
    }

    /// Enhanced circular reference detection using heuristics
    fn detect_circular_references(&self, tracker: &SmartPointerTracker) -> Vec<u64> {
        let mut circular_ids = Vec::new();

        // Look for pairs of Rc/Arc with similar lifetimes and ref counts
        let rc_pointers = tracker.get_active_by_type(&PointerType::Rc);
        let arc_pointers = tracker.get_active_by_type(&PointerType::Arc);

        // Check Rc circular patterns
        circular_ids.extend(self.find_circular_patterns(&rc_pointers));
        // Check Arc circular patterns
        circular_ids.extend(self.find_circular_patterns(&arc_pointers));

        circular_ids
    }

    fn find_circular_patterns(&self, pointers: &[&super::PointerInfo]) -> Vec<u64> {
        let mut patterns = Vec::new();

        for (i, info_a) in pointers.iter().enumerate() {
            for info_b in pointers.iter().skip(i + 1) {
                // Heuristic: two pointers with ref_count >= 2, similar age, similar size
                if let (Some(ref_a), Some(ref_b)) = (info_a.ref_count, info_b.ref_count) {
                    if ref_a >= 2 && ref_b >= 2 {
                        let age_diff = info_a.age().as_secs().abs_diff(info_b.age().as_secs());
                        let size_ratio =
                            info_a.allocation_size as f64 / info_b.allocation_size as f64;

                        // Strong indicators of circular reference
                        if age_diff <= 5 && // Created within 5 seconds of each other
                           (0.5..=2.0).contains(&size_ratio) && // Similar sizes
                           ref_a == ref_b
                        {
                            // Same reference count (suspicious)
                            patterns.push(info_a.allocation_id);
                            patterns.push(info_b.allocation_id);
                        }
                    }
                }
            }
        }

        patterns
    }

    fn is_likely_circular_reference(
        &self,
        info: &super::PointerInfo,
        tracker: &SmartPointerTracker,
    ) -> bool {
        // Heuristic: check if there are other similar allocations that might form a cycle
        let same_type_pointers = tracker.get_active_by_type(&info.ptr_type);

        for other in same_type_pointers {
            if other.allocation_id != info.allocation_id {
                if let (Some(ref_a), Some(ref_b)) = (info.ref_count, other.ref_count) {
                    // Both have ref count >= 2 and were created close in time
                    if ref_a >= 2 && ref_b >= 2 {
                        let age_diff = info.age().as_secs().abs_diff(other.age().as_secs());
                        if age_diff <= 10 {
                            // Created within 10 seconds
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    fn is_suspicious_weak_ref(
        &self,
        info: &super::PointerInfo,
        _tracker: &SmartPointerTracker,
    ) -> bool {
        // Weak references that live longer than 30 seconds might indicate
        // they're being held to break circular references but never cleaned up
        info.age() > Duration::from_secs(30)
    }

    fn calculate_health_score(
        &self,
        patterns: &[(LeakPattern, Vec<u64>)],
        total_pointers: usize,
    ) -> f64 {
        if total_pointers == 0 {
            return 1.0;
        }

        let mut penalty = 0.0;

        for (pattern, ids) in patterns {
            let severity = match pattern {
                LeakPattern::CircularReference => 0.3,
                LeakPattern::WeakReferenceHold => 0.1,
                LeakPattern::LongLivedBox => 0.15,
                LeakPattern::HighRefCount => 0.2,
                LeakPattern::SynchronizationOveruse => 0.1,
            };

            let ratio = ids.len() as f64 / total_pointers as f64;
            penalty += severity * ratio;
        }

        (1.0 - penalty).max(0.0)
    }

    pub fn generate_report(&self, result: &AnalysisResult) -> String {
        let mut report = String::new();

        report.push_str(&format!("Smart Pointer Analysis Report\n"));
        report.push_str(&format!("============================\n\n"));

        report.push_str(&format!(
            "Total Active Pointers: {}\n",
            result.total_active_pointers
        ));
        report.push_str(&format!(
            "Health Score: {:.2}/1.00\n\n",
            result.health_score
        ));

        report.push_str("Memory Usage by Type:\n");
        for (ptr_type, usage) in &result.memory_usage_by_type {
            report.push_str(&format!("  {:?}: {} bytes\n", ptr_type, usage));
        }

        if !result.detected_patterns.is_empty() {
            report.push_str("\nDetected Patterns:\n");
            for (pattern, ids) in &result.detected_patterns {
                report.push_str(&format!("  {:?}: {} instances\n", pattern, ids.len()));
            }
        }

        if !result.recommendations.is_empty() {
            report.push_str("\nRecommendations:\n");
            for (i, rec) in result.recommendations.iter().enumerate() {
                report.push_str(&format!("  {}. {}\n", i + 1, rec));
            }
        }

        report
    }
}

impl Default for SmartPointerAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisResult {
    pub fn is_healthy(&self) -> bool {
        self.health_score >= 0.8
    }

    pub fn has_critical_issues(&self) -> bool {
        self.detected_patterns.iter().any(|(pattern, ids)| {
            matches!(pattern, LeakPattern::CircularReference) && !ids.is_empty()
        })
    }

    pub fn total_memory_usage(&self) -> usize {
        self.memory_usage_by_type.values().sum()
    }

    pub fn pattern_count(&self, pattern: &LeakPattern) -> usize {
        self.detected_patterns
            .iter()
            .find(|(p, _)| std::mem::discriminant(p) == std::mem::discriminant(pattern))
            .map(|(_, ids)| ids.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_analysis() {
        let mut tracker = SmartPointerTracker::new();
        let analyzer = SmartPointerAnalyzer::new();

        tracker.track_allocation(0x1000, PointerType::Box, 64, "String".to_string(), None);
        tracker.track_allocation(0x2000, PointerType::Arc, 128, "Data".to_string(), Some(1));

        let result = analyzer.analyze(&tracker);

        assert_eq!(result.total_active_pointers, 2);
        assert!(result.health_score > 0.8);
        assert!(!result.has_critical_issues());
    }

    #[test]
    fn test_high_ref_count_detection() {
        let mut tracker = SmartPointerTracker::new();
        let analyzer = SmartPointerAnalyzer::with_thresholds(3600, 2, 100);

        tracker.track_allocation(0x1000, PointerType::Rc, 64, "String".to_string(), Some(5));

        let result = analyzer.analyze(&tracker);

        assert_eq!(result.pattern_count(&LeakPattern::HighRefCount), 1);
        assert!(result.health_score < 1.0);
    }

    #[test]
    fn test_health_score_calculation() {
        let tracker = SmartPointerTracker::new();
        let analyzer = SmartPointerAnalyzer::new();

        let result = analyzer.analyze(&tracker);

        // Empty tracker should have perfect health score
        assert_eq!(result.health_score, 1.0);
        assert!(result.is_healthy());
    }

    #[test]
    fn test_report_generation() {
        let mut tracker = SmartPointerTracker::new();
        let analyzer = SmartPointerAnalyzer::new();

        tracker.track_allocation(0x1000, PointerType::Box, 64, "String".to_string(), None);

        let result = analyzer.analyze(&tracker);
        let report = analyzer.generate_report(&result);

        assert!(report.contains("Smart Pointer Analysis Report"));
        assert!(report.contains("Total Active Pointers: 1"));
        assert!(report.contains("Health Score:"));
    }
}
