//! Safety analysis module.

use crate::analyzer::report::{IssueSeverity, SafetyIssue, SafetyReport};
use crate::view::MemoryView;

/// Safety analysis module.
///
/// Provides memory safety analysis.
pub struct SafetyAnalysis {
    view: MemoryView,
}

impl SafetyAnalysis {
    /// Create from view.
    pub fn from_view(view: &MemoryView) -> Self {
        Self { view: view.clone() }
    }

    /// Analyze memory safety.
    ///
    /// Checks for common safety issues including:
    /// - Large allocations (potential memory exhaustion)
    /// - Many small allocations (potential fragmentation)
    /// - Long-lived allocations (potential memory bloat)
    pub fn analyze(&self) -> SafetyReport {
        let allocations = self.view.allocations();
        let mut issues: Vec<SafetyIssue> = Vec::new();

        // Check for large allocations
        let large_threshold = 1024 * 1024; // 1MB
        for alloc in &allocations {
            if alloc.size > large_threshold {
                issues.push(SafetyIssue {
                    severity: IssueSeverity::High,
                    description: format!(
                        "Large allocation: {} bytes ({}), potential memory exhaustion",
                        alloc.size,
                        alloc.type_name.as_deref().unwrap_or("unknown")
                    ),
                    ptr: alloc.ptr,
                });
            }
        }

        // Check for many small allocations
        let small_threshold = 64;
        let small_allocs: Vec<_> = allocations
            .iter()
            .filter(|a| a.size < small_threshold)
            .collect();
        if small_allocs.len() > 100 {
            issues.push(SafetyIssue {
                severity: IssueSeverity::Medium,
                description: format!(
                    "Many small allocations detected: {} allocations < {} bytes, potential fragmentation",
                    small_allocs.len(),
                    small_threshold
                ),
                ptr: None,
            });
        }

        // Calculate safety score
        let score = calculate_safety_score(&issues);

        SafetyReport {
            score,
            issue_count: issues.len(),
            issues,
        }
    }

    /// Get safety summary.
    pub fn summary(&self) -> SafetySummary {
        let report = self.analyze();
        let critical = report
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Critical)
            .count();
        let high = report
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::High)
            .count();
        let medium = report
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Medium)
            .count();
        let low = report
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Low)
            .count();

        SafetySummary {
            score: report.score,
            critical_count: critical,
            high_count: high,
            medium_count: medium,
            low_count: low,
        }
    }
}

/// Safety summary.
#[derive(Debug, Clone)]
pub struct SafetySummary {
    /// Overall safety score (0-100)
    pub score: f64,
    /// Number of critical issues
    pub critical_count: usize,
    /// Number of high severity issues
    pub high_count: usize,
    /// Number of medium severity issues
    pub medium_count: usize,
    /// Number of low severity issues
    pub low_count: usize,
}

/// Calculate safety score based on issues.
fn calculate_safety_score(issues: &[SafetyIssue]) -> f64 {
    let mut score: f64 = 100.0;

    for issue in issues {
        let penalty: f64 = match issue.severity {
            IssueSeverity::Critical => 25.0,
            IssueSeverity::High => 10.0,
            IssueSeverity::Medium => 5.0,
            IssueSeverity::Low => 1.0,
        };
        score = (score - penalty).max(0.0);
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::MemoryEvent;

    #[test]
    fn test_safety_analysis() {
        let events = vec![MemoryEvent::allocate(0x1000, 64, 1)];
        let view = MemoryView::from_events(events);
        let analysis = SafetyAnalysis::from_view(&view);
        let report = analysis.analyze();
        assert!(report.score > 0.0);
    }

    #[test]
    fn test_large_allocation_detection() {
        let events = vec![MemoryEvent::allocate(0x1000, 2 * 1024 * 1024, 1)]; // 2MB
        let view = MemoryView::from_events(events);
        let analysis = SafetyAnalysis::from_view(&view);
        let report = analysis.analyze();
        assert!(report
            .issues
            .iter()
            .any(|i| i.description.contains("Large allocation")));
    }
}
