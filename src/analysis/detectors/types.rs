//! Common types for memory analysis detectors
//!
//! This module provides shared types used across all detectors,
//! ensuring consistent interfaces and data structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Result of a detection operation
///
/// Contains all detected issues, statistics, and metadata from a detector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    /// Name of the detector that produced this result
    pub detector_name: String,

    /// List of detected issues
    pub issues: Vec<Issue>,

    /// Detection statistics
    pub statistics: DetectionStatistics,

    /// Time taken for detection in milliseconds
    pub detection_time_ms: u64,
}

impl DetectionResult {
    /// Create a new detection result
    pub fn new(
        detector_name: String,
        issues: Vec<Issue>,
        statistics: DetectionStatistics,
        detection_time_ms: u64,
    ) -> Self {
        Self {
            detector_name,
            issues,
            statistics,
            detection_time_ms,
        }
    }

    /// Get number of detected issues
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    /// Get issues by severity
    pub fn issues_by_severity(&self, severity: IssueSeverity) -> Vec<&Issue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity == severity)
            .collect()
    }

    /// Get issues by category
    pub fn issues_by_category(&self, category: IssueCategory) -> Vec<&Issue> {
        self.issues
            .iter()
            .filter(|issue| issue.category == category)
            .collect()
    }

    /// Check if result has any critical issues
    pub fn has_critical_issues(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| issue.severity == IssueSeverity::Critical)
    }
}

impl fmt::Display for DetectionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} issues detected in {}ms",
            self.detector_name,
            self.issue_count(),
            self.detection_time_ms
        )
    }
}

/// A detected issue
///
/// Represents a single memory issue found by a detector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Unique identifier for this issue
    pub id: String,

    /// Severity level of the issue
    pub severity: IssueSeverity,

    /// Category of the issue
    pub category: IssueCategory,

    /// Human-readable description
    pub description: String,

    /// Location where the issue was detected
    pub location: Option<Location>,

    /// Pointer to the allocation involved (if applicable)
    pub allocation_ptr: Option<usize>,

    /// Suggested fix for the issue
    pub suggested_fix: Option<String>,
}

impl Issue {
    /// Create a new issue
    pub fn new(
        id: String,
        severity: IssueSeverity,
        category: IssueCategory,
        description: String,
    ) -> Self {
        Self {
            id,
            severity,
            category,
            description,
            location: None,
            allocation_ptr: None,
            suggested_fix: None,
        }
    }

    /// Create a critical issue
    pub fn critical(id: String, category: IssueCategory, description: String) -> Self {
        Self::new(id, IssueSeverity::Critical, category, description)
    }

    /// Create a high severity issue
    pub fn high(id: String, category: IssueCategory, description: String) -> Self {
        Self::new(id, IssueSeverity::High, category, description)
    }

    /// Create a medium severity issue
    pub fn medium(id: String, category: IssueCategory, description: String) -> Self {
        Self::new(id, IssueSeverity::Medium, category, description)
    }

    /// Create a low severity issue
    pub fn low(id: String, category: IssueCategory, description: String) -> Self {
        Self::new(id, IssueSeverity::Low, category, description)
    }

    /// Set the location of the issue
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }

    /// Set the allocation pointer
    pub fn with_allocation_ptr(mut self, ptr: usize) -> Self {
        self.allocation_ptr = Some(ptr);
        self
    }

    /// Set a suggested fix
    pub fn with_suggested_fix(mut self, fix: String) -> Self {
        self.suggested_fix = Some(fix);
        self
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}: {}", self.severity, self.id, self.description)
    }
}

/// Severity level of an issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Informational
    Info,
    /// Optional
    Low,
    /// Nice to fix
    Medium,
    /// Should fix soon
    High,
    /// Must fix immediately
    Critical,
}

impl fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueSeverity::Critical => write!(f, "CRITICAL"),
            IssueSeverity::High => write!(f, "HIGH"),
            IssueSeverity::Medium => write!(f, "MEDIUM"),
            IssueSeverity::Low => write!(f, "LOW"),
            IssueSeverity::Info => write!(f, "INFO"),
        }
    }
}

impl Default for IssueSeverity {
    fn default() -> Self {
        IssueSeverity::Info
    }
}

/// Category of the issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueCategory {
    /// Memory leak
    Leak,
    /// Use-after-free
    Uaf,
    /// Buffer overflow
    Overflow,
    /// Safety violation
    Safety,
    /// Performance issue
    Performance,
    /// Lifetime issue
    Lifetime,
    /// Concurrency issue
    Concurrency,
    /// Type issue
    Type,
    /// Other
    Other,
}

impl fmt::Display for IssueCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueCategory::Leak => write!(f, "Leak"),
            IssueCategory::Uaf => write!(f, "Use-After-Free"),
            IssueCategory::Overflow => write!(f, "Overflow"),
            IssueCategory::Safety => write!(f, "Safety"),
            IssueCategory::Performance => write!(f, "Performance"),
            IssueCategory::Lifetime => write!(f, "Lifetime"),
            IssueCategory::Concurrency => write!(f, "Concurrency"),
            IssueCategory::Type => write!(f, "Type"),
            IssueCategory::Other => write!(f, "Other"),
        }
    }
}

impl Default for IssueCategory {
    fn default() -> Self {
        IssueCategory::Other
    }
}

/// Location information for an issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// File path
    pub file: String,

    /// Line number
    pub line: Option<usize>,

    /// Column number
    pub column: Option<usize>,

    /// Function name
    pub function: Option<String>,
}

impl Location {
    /// Create a new location
    pub fn new(file: String) -> Self {
        Self {
            file,
            line: None,
            column: None,
            function: None,
        }
    }

    /// Set the line number
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Set the column number
    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// Set the function name
    pub fn with_function(mut self, function: String) -> Self {
        self.function = Some(function);
        self
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.file)?;
        if let Some(line) = self.line {
            write!(f, ":{}", line)?;
            if let Some(col) = self.column {
                write!(f, ":{}", col)?;
            }
        }
        if let Some(func) = &self.function {
            write!(f, " in {}", func)?;
        }
        Ok(())
    }
}

/// Configuration for a detector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorConfig {
    /// Whether the detector is enabled
    pub enabled: bool,

    /// Maximum number of issues to report
    pub max_reported_issues: usize,

    /// Whether to include detailed information
    pub include_details: bool,

    /// Custom configuration options
    pub custom_options: HashMap<String, String>,
}

impl DetectorConfig {
    /// Create a new detector configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a custom option value
    pub fn get_custom_option(&self, key: &str) -> Option<&String> {
        self.custom_options.get(key)
    }

    /// Set a custom option
    pub fn set_custom_option(&mut self, key: String, value: String) {
        self.custom_options.insert(key, value);
    }

    /// Check if a custom option exists
    pub fn has_custom_option(&self, key: &str) -> bool {
        self.custom_options.contains_key(key)
    }
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_reported_issues: 100,
            include_details: true,
            custom_options: HashMap::new(),
        }
    }
}

/// Error from a detector
#[derive(Debug, Clone)]
pub enum DetectorError {
    /// Configuration error
    ConfigError(String),

    /// Detection error
    DetectionError(String),

    /// Invalid input
    InvalidInput(String),

    /// Internal error
    InternalError(String),
}

impl fmt::Display for DetectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectorError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            DetectorError::DetectionError(msg) => write!(f, "Detection error: {}", msg),
            DetectorError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            DetectorError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for DetectorError {}

/// Statistics from a detection operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionStatistics {
    /// Total number of allocations analyzed
    pub total_allocations: usize,

    /// Number of allocations with issues
    pub allocations_with_issues: usize,

    /// Total memory analyzed in bytes
    pub total_memory_analyzed: usize,

    /// Memory affected by issues in bytes
    pub memory_affected: usize,

    /// Breakdown by severity
    pub severity_breakdown: HashMap<IssueSeverity, usize>,

    /// Breakdown by category
    pub category_breakdown: HashMap<IssueCategory, usize>,
}

impl DetectionStatistics {
    /// Create new detection statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Update severity breakdown
    pub fn update_severity(&mut self, severity: IssueSeverity) {
        *self.severity_breakdown.entry(severity).or_insert(0) += 1;
    }

    /// Update category breakdown
    pub fn update_category(&mut self, category: IssueCategory) {
        *self.category_breakdown.entry(category).or_insert(0) += 1;
    }

    /// Get total issues count
    pub fn total_issues(&self) -> usize {
        self.severity_breakdown.values().sum()
    }

    /// Get issues count by severity
    pub fn issues_by_severity(&self, severity: IssueSeverity) -> usize {
        *self.severity_breakdown.get(&severity).unwrap_or(&0)
    }

    /// Get issues count by category
    pub fn issues_by_category(&self, category: IssueCategory) -> usize {
        *self.category_breakdown.get(&category).unwrap_or(&0)
    }
}

impl Default for DetectionStatistics {
    fn default() -> Self {
        Self {
            total_allocations: 0,
            allocations_with_issues: 0,
            total_memory_analyzed: 0,
            memory_affected: 0,
            severity_breakdown: HashMap::new(),
            category_breakdown: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_result_creation() {
        let result = DetectionResult::new(
            "TestDetector".to_string(),
            vec![],
            DetectionStatistics::default(),
            100,
        );

        assert_eq!(result.detector_name, "TestDetector");
        assert_eq!(result.issue_count(), 0);
        assert_eq!(result.detection_time_ms, 100);
    }

    #[test]
    fn test_detection_result_with_issues() {
        let issues = vec![
            Issue::new(
                "issue1".to_string(),
                IssueSeverity::Critical,
                IssueCategory::Leak,
                "Test issue".to_string(),
            ),
            Issue::new(
                "issue2".to_string(),
                IssueSeverity::High,
                IssueCategory::Uaf,
                "Another issue".to_string(),
            ),
        ];

        let result = DetectionResult::new(
            "TestDetector".to_string(),
            issues,
            DetectionStatistics::default(),
            100,
        );

        assert_eq!(result.issue_count(), 2);
        assert_eq!(result.issues_by_severity(IssueSeverity::Critical).len(), 1);
        assert_eq!(result.issues_by_severity(IssueSeverity::High).len(), 1);
        assert!(result.has_critical_issues());
    }

    #[test]
    fn test_issue_creation() {
        let issue = Issue::new(
            "test_issue".to_string(),
            IssueSeverity::Medium,
            IssueCategory::Performance,
            "Test description".to_string(),
        );

        assert_eq!(issue.id, "test_issue");
        assert_eq!(issue.severity, IssueSeverity::Medium);
        assert_eq!(issue.category, IssueCategory::Performance);
        assert_eq!(issue.description, "Test description");
        assert!(issue.location.is_none());
        assert!(issue.allocation_ptr.is_none());
        assert!(issue.suggested_fix.is_none());
    }

    #[test]
    fn test_issue_builder() {
        let location = Location::new("test.rs".to_string())
            .with_line(10)
            .with_column(5)
            .with_function("test_func".to_string());

        let issue = Issue::new(
            "test_issue".to_string(),
            IssueSeverity::High,
            IssueCategory::Safety,
            "Test description".to_string(),
        )
        .with_location(location)
        .with_allocation_ptr(0x1000)
        .with_suggested_fix("Fix the issue".to_string());

        assert!(issue.location.is_some());
        assert_eq!(issue.allocation_ptr, Some(0x1000));
        assert_eq!(issue.suggested_fix, Some("Fix the issue".to_string()));
    }

    #[test]
    fn test_issue_severity_ordering() {
        assert!(IssueSeverity::Critical > IssueSeverity::High);
        assert!(IssueSeverity::High > IssueSeverity::Medium);
        assert!(IssueSeverity::Medium > IssueSeverity::Low);
        assert!(IssueSeverity::Low > IssueSeverity::Info);
    }

    #[test]
    fn test_location_creation() {
        let location = Location::new("test.rs".to_string());
        assert_eq!(location.file, "test.rs");
        assert!(location.line.is_none());
        assert!(location.column.is_none());
        assert!(location.function.is_none());
    }

    #[test]
    fn test_location_builder() {
        let location = Location::new("test.rs".to_string())
            .with_line(10)
            .with_column(5)
            .with_function("test_func".to_string());

        assert_eq!(location.line, Some(10));
        assert_eq!(location.column, Some(5));
        assert_eq!(location.function, Some("test_func".to_string()));
    }

    #[test]
    fn test_location_display() {
        let location = Location::new("test.rs".to_string())
            .with_line(10)
            .with_column(5)
            .with_function("test_func".to_string());

        let display = format!("{}", location);
        assert!(display.contains("test.rs"));
        assert!(display.contains("10"));
        assert!(display.contains("5"));
        assert!(display.contains("test_func"));
    }

    #[test]
    fn test_detector_config_default() {
        let config = DetectorConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_reported_issues, 100);
        assert!(config.include_details);
        assert!(config.custom_options.is_empty());
    }

    #[test]
    fn test_detector_config_custom_options() {
        let mut config = DetectorConfig::default();
        config.set_custom_option("key1".to_string(), "value1".to_string());
        config.set_custom_option("key2".to_string(), "value2".to_string());

        assert_eq!(
            config.get_custom_option("key1"),
            Some(&"value1".to_string())
        );
        assert_eq!(
            config.get_custom_option("key2"),
            Some(&"value2".to_string())
        );
        assert!(config.has_custom_option("key1"));
        assert!(!config.has_custom_option("key3"));
    }

    #[test]
    fn test_detector_error_display() {
        let error = DetectorError::ConfigError("Invalid configuration".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Configuration error"));
        assert!(display.contains("Invalid configuration"));
    }

    #[test]
    fn test_detection_statistics_update() {
        let mut stats = DetectionStatistics::new();
        stats.total_allocations = 100;
        stats.total_memory_analyzed = 1024 * 1024;

        stats.update_severity(IssueSeverity::Critical);
        stats.update_severity(IssueSeverity::Critical);
        stats.update_severity(IssueSeverity::High);
        stats.update_category(IssueCategory::Leak);
        stats.update_category(IssueCategory::Leak);
        stats.update_category(IssueCategory::Uaf);

        assert_eq!(stats.total_issues(), 3);
        assert_eq!(stats.issues_by_severity(IssueSeverity::Critical), 2);
        assert_eq!(stats.issues_by_severity(IssueSeverity::High), 1);
        assert_eq!(stats.issues_by_category(IssueCategory::Leak), 2);
        assert_eq!(stats.issues_by_category(IssueCategory::Uaf), 1);
    }
}
