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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub enum IssueSeverity {
    /// Informational
    #[default]
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

/// Category of the issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
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
    #[default]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

    #[test]
    fn test_issue_critical_helper() {
        let issue = Issue::critical(
            "CRIT-001".to_string(),
            IssueCategory::Leak,
            "Critical memory leak".to_string(),
        );

        assert_eq!(issue.severity, IssueSeverity::Critical);
        assert_eq!(issue.id, "CRIT-001");
    }

    #[test]
    fn test_issue_high_helper() {
        let issue = Issue::high(
            "HIGH-001".to_string(),
            IssueCategory::Uaf,
            "Use after free detected".to_string(),
        );

        assert_eq!(issue.severity, IssueSeverity::High);
        assert_eq!(issue.category, IssueCategory::Uaf);
    }

    #[test]
    fn test_issue_medium_helper() {
        let issue = Issue::medium(
            "MED-001".to_string(),
            IssueCategory::Performance,
            "Inefficient allocation pattern".to_string(),
        );

        assert_eq!(issue.severity, IssueSeverity::Medium);
        assert_eq!(issue.category, IssueCategory::Performance);
    }

    #[test]
    fn test_issue_low_helper() {
        let issue = Issue::low(
            "LOW-001".to_string(),
            IssueCategory::Type,
            "Type could be optimized".to_string(),
        );

        assert_eq!(issue.severity, IssueSeverity::Low);
        assert_eq!(issue.category, IssueCategory::Type);
    }

    #[test]
    fn test_issue_severity_display() {
        assert_eq!(format!("{}", IssueSeverity::Critical), "CRITICAL");
        assert_eq!(format!("{}", IssueSeverity::High), "HIGH");
        assert_eq!(format!("{}", IssueSeverity::Medium), "MEDIUM");
        assert_eq!(format!("{}", IssueSeverity::Low), "LOW");
        assert_eq!(format!("{}", IssueSeverity::Info), "INFO");
    }

    #[test]
    fn test_issue_category_display() {
        assert_eq!(format!("{}", IssueCategory::Leak), "Leak");
        assert_eq!(format!("{}", IssueCategory::Uaf), "Use-After-Free");
        assert_eq!(format!("{}", IssueCategory::Overflow), "Overflow");
        assert_eq!(format!("{}", IssueCategory::Safety), "Safety");
        assert_eq!(format!("{}", IssueCategory::Performance), "Performance");
        assert_eq!(format!("{}", IssueCategory::Lifetime), "Lifetime");
        assert_eq!(format!("{}", IssueCategory::Concurrency), "Concurrency");
        assert_eq!(format!("{}", IssueCategory::Type), "Type");
        assert_eq!(format!("{}", IssueCategory::Other), "Other");
    }

    #[test]
    fn test_issue_display() {
        let issue = Issue::new(
            "TEST-001".to_string(),
            IssueSeverity::High,
            IssueCategory::Leak,
            "Memory leak detected".to_string(),
        );

        let display = format!("{}", issue);
        assert!(display.contains("High"));
        assert!(display.contains("TEST-001"));
        assert!(display.contains("Memory leak detected"));
    }

    #[test]
    fn test_detection_result_display() {
        let result = DetectionResult::new(
            "LeakDetector".to_string(),
            vec![Issue::new(
                "L1".to_string(),
                IssueSeverity::Medium,
                IssueCategory::Leak,
                "Test".to_string(),
            )],
            DetectionStatistics::default(),
            50,
        );

        let display = format!("{}", result);
        assert!(display.contains("LeakDetector"));
        assert!(display.contains("1 issues"));
        assert!(display.contains("50ms"));
    }

    #[test]
    fn test_detection_result_no_critical_issues() {
        let result = DetectionResult::new(
            "TestDetector".to_string(),
            vec![
                Issue::medium("1".to_string(), IssueCategory::Leak, "Test".to_string()),
                Issue::low(
                    "2".to_string(),
                    IssueCategory::Performance,
                    "Test".to_string(),
                ),
            ],
            DetectionStatistics::default(),
            10,
        );

        assert!(!result.has_critical_issues());
    }

    #[test]
    fn test_detection_result_issues_by_category() {
        let result = DetectionResult::new(
            "TestDetector".to_string(),
            vec![
                Issue::medium("1".to_string(), IssueCategory::Leak, "Test".to_string()),
                Issue::medium("2".to_string(), IssueCategory::Leak, "Test".to_string()),
                Issue::medium("3".to_string(), IssueCategory::Uaf, "Test".to_string()),
                Issue::low(
                    "4".to_string(),
                    IssueCategory::Performance,
                    "Test".to_string(),
                ),
            ],
            DetectionStatistics::default(),
            10,
        );

        assert_eq!(result.issues_by_category(IssueCategory::Leak).len(), 2);
        assert_eq!(result.issues_by_category(IssueCategory::Uaf).len(), 1);
        assert_eq!(
            result.issues_by_category(IssueCategory::Performance).len(),
            1
        );
        assert_eq!(result.issues_by_category(IssueCategory::Safety).len(), 0);
    }

    #[test]
    fn test_location_display_no_line() {
        let location = Location::new("src/main.rs".to_string());
        let display = format!("{}", location);
        assert_eq!(display, "src/main.rs");
    }

    #[test]
    fn test_location_display_with_line_only() {
        let location = Location::new("src/main.rs".to_string()).with_line(42);
        let display = format!("{}", location);
        assert_eq!(display, "src/main.rs:42");
    }

    #[test]
    fn test_location_display_no_function() {
        let location = Location::new("src/main.rs".to_string())
            .with_line(42)
            .with_column(10);
        let display = format!("{}", location);
        assert_eq!(display, "src/main.rs:42:10");
    }

    #[test]
    fn test_detector_error_all_variants() {
        let config_err = DetectorError::ConfigError("config issue".to_string());
        let detection_err = DetectorError::DetectionError("detection failed".to_string());
        let invalid_err = DetectorError::InvalidInput("bad input".to_string());
        let internal_err = DetectorError::InternalError("internal bug".to_string());

        assert!(format!("{}", config_err).contains("Configuration error"));
        assert!(format!("{}", detection_err).contains("Detection error"));
        assert!(format!("{}", invalid_err).contains("Invalid input"));
        assert!(format!("{}", internal_err).contains("Internal error"));
    }

    #[test]
    fn test_detector_config_new() {
        let config = DetectorConfig::new();
        assert!(config.enabled);
        assert_eq!(config.max_reported_issues, 100);
    }

    #[test]
    fn test_detector_config_disabled() {
        let config = DetectorConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(!config.enabled);
    }

    #[test]
    fn test_detector_config_max_issues() {
        let config = DetectorConfig {
            max_reported_issues: 50,
            ..Default::default()
        };
        assert_eq!(config.max_reported_issues, 50);
    }

    #[test]
    fn test_detector_config_no_details() {
        let config = DetectorConfig {
            include_details: false,
            ..Default::default()
        };
        assert!(!config.include_details);
    }

    #[test]
    fn test_detection_statistics_default() {
        let stats = DetectionStatistics::default();
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.allocations_with_issues, 0);
        assert_eq!(stats.total_memory_analyzed, 0);
        assert_eq!(stats.memory_affected, 0);
        assert!(stats.severity_breakdown.is_empty());
        assert!(stats.category_breakdown.is_empty());
    }

    #[test]
    fn test_detection_statistics_with_values() {
        let mut stats = DetectionStatistics::new();
        stats.total_allocations = 1000;
        stats.allocations_with_issues = 50;
        stats.total_memory_analyzed = 1024 * 1024;
        stats.memory_affected = 512 * 1024;

        assert_eq!(stats.total_allocations, 1000);
        assert_eq!(stats.allocations_with_issues, 50);
    }

    #[test]
    fn test_detection_statistics_multiple_updates() {
        let mut stats = DetectionStatistics::new();

        for _ in 0..10 {
            stats.update_severity(IssueSeverity::Low);
        }
        for _ in 0..5 {
            stats.update_severity(IssueSeverity::Medium);
        }
        for _ in 0..3 {
            stats.update_category(IssueCategory::Performance);
        }

        assert_eq!(stats.issues_by_severity(IssueSeverity::Low), 10);
        assert_eq!(stats.issues_by_severity(IssueSeverity::Medium), 5);
        assert_eq!(stats.issues_by_category(IssueCategory::Performance), 3);
    }

    #[test]
    fn test_issue_severity_default() {
        let severity = IssueSeverity::default();
        assert_eq!(severity, IssueSeverity::Info);
    }

    #[test]
    fn test_issue_category_default() {
        let category = IssueCategory::default();
        assert_eq!(category, IssueCategory::Other);
    }

    #[test]
    fn test_issue_severity_equality() {
        assert_eq!(IssueSeverity::Critical, IssueSeverity::Critical);
        assert_ne!(IssueSeverity::Critical, IssueSeverity::High);
    }

    #[test]
    fn test_issue_category_equality() {
        assert_eq!(IssueCategory::Leak, IssueCategory::Leak);
        assert_ne!(IssueCategory::Leak, IssueCategory::Uaf);
    }

    #[test]
    fn test_issue_severity_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(IssueSeverity::Critical);
        set.insert(IssueSeverity::High);
        set.insert(IssueSeverity::Critical);

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_issue_category_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(IssueCategory::Leak);
        set.insert(IssueCategory::Uaf);
        set.insert(IssueCategory::Leak);

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_detection_result_serialization() {
        let result = DetectionResult::new(
            "TestDetector".to_string(),
            vec![],
            DetectionStatistics::default(),
            100,
        );

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("TestDetector"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_issue_serialization() {
        let issue = Issue::new(
            "SER-001".to_string(),
            IssueSeverity::High,
            IssueCategory::Leak,
            "Serialized issue".to_string(),
        );

        let json = serde_json::to_string(&issue).unwrap();
        let deserialized: Issue = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, issue.id);
        assert_eq!(deserialized.severity, issue.severity);
    }

    #[test]
    fn test_location_serialization() {
        let location = Location::new("test.rs".to_string())
            .with_line(10)
            .with_column(5)
            .with_function("test_func".to_string());

        let json = serde_json::to_string(&location).unwrap();
        let deserialized: Location = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.file, location.file);
        assert_eq!(deserialized.line, location.line);
    }

    #[test]
    fn test_detector_config_serialization() {
        let mut config = DetectorConfig::default();
        config.set_custom_option("key".to_string(), "value".to_string());

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: DetectorConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.enabled);
        assert_eq!(
            deserialized.get_custom_option("key"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_issue_severity_serialization() {
        let severities = vec![
            IssueSeverity::Info,
            IssueSeverity::Low,
            IssueSeverity::Medium,
            IssueSeverity::High,
            IssueSeverity::Critical,
        ];

        for severity in severities {
            let json = serde_json::to_string(&severity).unwrap();
            let deserialized: IssueSeverity = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, severity);
        }
    }

    #[test]
    fn test_issue_category_serialization() {
        let categories = vec![
            IssueCategory::Leak,
            IssueCategory::Uaf,
            IssueCategory::Overflow,
            IssueCategory::Safety,
            IssueCategory::Performance,
            IssueCategory::Lifetime,
            IssueCategory::Concurrency,
            IssueCategory::Type,
            IssueCategory::Other,
        ];

        for category in categories {
            let json = serde_json::to_string(&category).unwrap();
            let deserialized: IssueCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, category);
        }
    }

    #[test]
    fn test_detection_statistics_serialization() {
        let mut stats = DetectionStatistics::new();
        stats.total_allocations = 100;
        stats.update_severity(IssueSeverity::High);

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: DetectionStatistics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_allocations, 100);
    }

    #[test]
    fn test_issue_clone() {
        let issue = Issue::new(
            "CLONE-001".to_string(),
            IssueSeverity::Medium,
            IssueCategory::Performance,
            "Clone test".to_string(),
        );

        let cloned = issue.clone();
        assert_eq!(cloned.id, issue.id);
        assert_eq!(cloned.severity, issue.severity);
    }

    #[test]
    fn test_detection_result_clone() {
        let result = DetectionResult::new(
            "CloneDetector".to_string(),
            vec![],
            DetectionStatistics::default(),
            50,
        );

        let cloned = result.clone();
        assert_eq!(cloned.detector_name, result.detector_name);
    }

    #[test]
    fn test_location_clone() {
        let location = Location::new("clone.rs".to_string()).with_line(100);
        let cloned = location.clone();
        assert_eq!(cloned.file, location.file);
        assert_eq!(cloned.line, location.line);
    }

    #[test]
    fn test_detector_config_clone() {
        let mut config = DetectorConfig::default();
        config.set_custom_option("test".to_string(), "value".to_string());
        let cloned = config.clone();
        assert_eq!(cloned.get_custom_option("test"), Some(&"value".to_string()));
    }

    #[test]
    fn test_issue_debug() {
        let issue = Issue::new(
            "DEBUG-001".to_string(),
            IssueSeverity::High,
            IssueCategory::Leak,
            "Debug test".to_string(),
        );

        let debug_str = format!("{:?}", issue);
        assert!(debug_str.contains("Issue"));
        assert!(debug_str.contains("DEBUG-001"));
    }

    #[test]
    fn test_detection_result_debug() {
        let result = DetectionResult::new(
            "DebugDetector".to_string(),
            vec![],
            DetectionStatistics::default(),
            25,
        );

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("DetectionResult"));
        assert!(debug_str.contains("DebugDetector"));
    }

    #[test]
    fn test_location_debug() {
        let location = Location::new("debug.rs".to_string()).with_line(50);
        let debug_str = format!("{:?}", location);
        assert!(debug_str.contains("Location"));
        assert!(debug_str.contains("debug.rs"));
    }

    #[test]
    fn test_detector_config_debug() {
        let config = DetectorConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("DetectorConfig"));
        assert!(debug_str.contains("enabled"));
    }

    #[test]
    fn test_detector_error_debug() {
        let error = DetectorError::ConfigError("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ConfigError"));
    }

    #[test]
    fn test_detection_statistics_debug() {
        let stats = DetectionStatistics::new();
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("DetectionStatistics"));
    }
}
