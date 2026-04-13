use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Code quality validator for memory analysis operations
pub struct QualityValidator {
    /// Active validation rules
    rules: Vec<ValidationRule>,
    /// Rule execution statistics
    rule_stats: HashMap<String, RuleStats>,
    /// Validation configuration
    config: ValidationConfig,
}

/// Individual validation rule
#[derive(Debug, Clone)]
pub struct ValidationRule {
    /// Unique rule identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Rule category
    pub category: RuleCategory,
    /// Severity level if rule fails
    pub severity: ValidationSeverity,
    /// Whether rule is enabled
    pub enabled: bool,
    /// Validation function
    pub validator: ValidationFunction,
}

/// Categories of validation rules
#[derive(Debug, Clone, PartialEq)]
pub enum RuleCategory {
    /// Memory safety and correctness
    MemorySafety,
    /// Performance and efficiency
    Performance,
    /// Code style and maintainability
    CodeStyle,
    /// Error handling patterns
    ErrorHandling,
    /// Thread safety
    ThreadSafety,
    /// Resource management
    ResourceManagement,
}

/// Validation severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    /// Informational note
    Info,
    /// Style or convention issue
    Style,
    /// Potential problem
    Warning,
    /// Definite problem
    Error,
    /// Critical issue that must be fixed
    Critical,
}

/// Validation function type
pub type ValidationFunction = fn(&ValidationContext) -> Result<(), ValidationError>;

/// Context provided to validation rules
#[derive(Debug)]
pub struct ValidationContext {
    /// Operation being validated
    pub operation_name: String,
    /// Performance metrics for the operation
    pub metrics: OperationMetrics,
    /// Memory usage information
    pub memory_info: MemoryInfo,
    /// Error handling status
    pub error_handling: ErrorHandlingInfo,
    /// Thread safety information
    pub thread_safety: ThreadSafetyInfo,
}

/// Metrics for a specific operation
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    /// Average execution time
    pub avg_duration: Duration,
    /// Peak memory usage during operation
    pub peak_memory: usize,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Number of allocations performed
    pub allocation_count: usize,
    /// CPU usage percentage
    pub cpu_usage: f64,
}

/// Memory usage information
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    /// Current memory usage in bytes
    pub current_usage: usize,
    /// Peak memory usage in bytes
    pub peak_usage: usize,
    /// Number of active allocations
    pub active_allocations: usize,
    /// Memory fragmentation ratio
    pub fragmentation_ratio: f64,
    /// Memory growth rate (bytes per second)
    pub growth_rate: f64,
}

/// Error handling information
#[derive(Debug, Clone)]
pub struct ErrorHandlingInfo {
    /// Whether errors are properly handled
    pub has_error_handling: bool,
    /// Number of potential error points
    pub error_points: usize,
    /// Number of handled error points
    pub handled_error_points: usize,
    /// Whether recovery mechanisms exist
    pub has_recovery: bool,
}

/// Thread safety information
#[derive(Debug, Clone)]
pub struct ThreadSafetyInfo {
    /// Whether operation is thread-safe
    pub is_thread_safe: bool,
    /// Number of shared resources accessed
    pub shared_resources: usize,
    /// Whether proper synchronization is used
    pub has_synchronization: bool,
    /// Lock contention level (0.0 to 1.0)
    pub contention_level: f64,
}

/// Validation error details
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Suggested fix
    pub suggestion: Option<String>,
    /// Code location if applicable
    pub location: Option<String>,
}

/// Result of running validation rules
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Overall validation status
    pub status: ValidationStatus,
    /// Individual rule results
    pub rule_results: Vec<RuleResult>,
    /// Summary statistics
    pub summary: ValidationSummary,
    /// Performance impact of validation
    pub validation_overhead: Duration,
}

/// Overall validation status
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStatus {
    /// All rules passed
    Passed,
    /// Some warnings found
    WarningsFound,
    /// Errors found that should be addressed
    ErrorsFound,
    /// Critical issues found
    CriticalIssuesFound,
}

/// Result of a single validation rule
#[derive(Debug, Clone)]
pub struct RuleResult {
    /// Rule that was executed
    pub rule_id: String,
    /// Whether rule passed
    pub passed: bool,
    /// Error details if rule failed
    pub error: Option<ValidationError>,
    /// Execution time for this rule
    pub execution_time: Duration,
}

/// Summary of validation results
#[derive(Debug, Clone)]
pub struct ValidationSummary {
    /// Total rules executed
    pub total_rules: usize,
    /// Number of rules that passed
    pub passed_rules: usize,
    /// Number of rules that failed
    pub failed_rules: usize,
    /// Number of critical issues
    pub critical_issues: usize,
    /// Number of errors
    pub errors: usize,
    /// Number of warnings
    pub warnings: usize,
    /// Overall quality score (0.0 to 1.0)
    pub quality_score: f64,
}

/// Configuration for validation behavior
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to stop on first critical error
    pub fail_fast: bool,
    /// Maximum time to spend on validation
    pub max_validation_time: Duration,
    /// Whether to enable performance-intensive checks
    pub enable_deep_checks: bool,
    /// Minimum severity level to report
    pub min_severity: ValidationSeverity,
}

/// Statistics for rule execution
#[derive(Debug, Clone)]
pub struct RuleStats {
    /// Number of times rule was executed
    execution_count: usize,
    /// Total execution time
    total_time: Duration,
    /// Number of times rule failed
    failure_count: usize,
    /// Average execution time
    avg_time: Duration,
}

impl QualityValidator {
    /// Create new validator with default rules
    pub fn new() -> Self {
        let mut validator = Self {
            rules: Vec::new(),
            rule_stats: HashMap::new(),
            config: ValidationConfig::default(),
        };

        validator.add_default_rules();
        validator
    }

    /// Create validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        let mut validator = Self {
            rules: Vec::new(),
            rule_stats: HashMap::new(),
            config,
        };

        validator.add_default_rules();
        validator
    }

    /// Add custom validation rule
    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.rules.push(rule);
    }

    /// Remove validation rule by ID
    pub fn remove_rule(&mut self, rule_id: &str) -> bool {
        if let Some(pos) = self.rules.iter().position(|r| r.id == rule_id) {
            self.rules.remove(pos);
            self.rule_stats.remove(rule_id);
            true
        } else {
            false
        }
    }

    /// Enable or disable a specific rule
    pub fn set_rule_enabled(&mut self, rule_id: &str, enabled: bool) -> bool {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Validate operation with all enabled rules
    pub fn validate(&mut self, context: &ValidationContext) -> ValidationResult {
        let start_time = Instant::now();
        let mut rule_results = Vec::new();
        let mut should_stop = false;

        // Collect rule information to avoid borrowing issues
        let rules_info: Vec<_> = self
            .rules
            .iter()
            .filter(|rule| rule.enabled)
            .map(|rule| (rule.id.clone(), rule.severity.clone(), rule.validator))
            .collect();

        for (rule_id, severity, validator) in rules_info {
            if should_stop && self.config.fail_fast {
                break;
            }

            let rule_start = Instant::now();
            let result = validator(context);
            let rule_duration = rule_start.elapsed();

            // Update statistics
            self.update_rule_stats(&rule_id, rule_duration, result.is_err());

            let rule_result = RuleResult {
                rule_id,
                passed: result.is_ok(),
                error: result.err(),
                execution_time: rule_duration,
            };

            if !rule_result.passed && severity >= ValidationSeverity::Critical {
                should_stop = true;
            }

            rule_results.push(rule_result);

            // Check timeout
            if start_time.elapsed() > self.config.max_validation_time {
                break;
            }
        }

        let validation_overhead = start_time.elapsed();
        let summary = self.calculate_summary(&rule_results);
        let status = self.determine_status(&summary);

        ValidationResult {
            status,
            rule_results,
            summary,
            validation_overhead,
        }
    }

    /// Get statistics for all rules
    pub fn get_rule_statistics(&self) -> &HashMap<String, RuleStats> {
        &self.rule_stats
    }

    /// Reset all statistics
    pub fn reset_statistics(&mut self) {
        self.rule_stats.clear();
    }

    fn add_default_rules(&mut self) {
        // Memory safety rules
        self.add_rule(ValidationRule {
            id: "memory_leak_check".to_string(),
            name: "Memory Leak Detection".to_string(),
            description: "Check for potential memory leaks in tracking operations".to_string(),
            category: RuleCategory::MemorySafety,
            severity: ValidationSeverity::Critical,
            enabled: true,
            validator: validate_memory_leaks,
        });

        self.add_rule(ValidationRule {
            id: "allocation_overhead_check".to_string(),
            name: "Allocation Overhead Check".to_string(),
            description: "Ensure allocation tracking overhead is within acceptable limits"
                .to_string(),
            category: RuleCategory::Performance,
            severity: ValidationSeverity::Warning,
            enabled: true,
            validator: validate_allocation_overhead,
        });

        // Performance rules
        self.add_rule(ValidationRule {
            id: "tracking_latency_check".to_string(),
            name: "Tracking Latency Check".to_string(),
            description: "Verify allocation tracking latency is acceptable".to_string(),
            category: RuleCategory::Performance,
            severity: ValidationSeverity::Error,
            enabled: true,
            validator: validate_tracking_latency,
        });

        self.add_rule(ValidationRule {
            id: "symbol_resolution_performance".to_string(),
            name: "Symbol Resolution Performance".to_string(),
            description: "Check symbol resolution performance metrics".to_string(),
            category: RuleCategory::Performance,
            severity: ValidationSeverity::Warning,
            enabled: true,
            validator: validate_symbol_performance,
        });

        // Error handling rules
        self.add_rule(ValidationRule {
            id: "error_handling_coverage".to_string(),
            name: "Error Handling Coverage".to_string(),
            description: "Ensure proper error handling in critical paths".to_string(),
            category: RuleCategory::ErrorHandling,
            severity: ValidationSeverity::Error,
            enabled: true,
            validator: validate_error_handling,
        });

        // Thread safety rules
        self.add_rule(ValidationRule {
            id: "thread_safety_check".to_string(),
            name: "Thread Safety Check".to_string(),
            description: "Verify thread safety of concurrent operations".to_string(),
            category: RuleCategory::ThreadSafety,
            severity: ValidationSeverity::Critical,
            enabled: true,
            validator: validate_thread_safety,
        });
    }

    fn update_rule_stats(&mut self, rule_id: &str, duration: Duration, failed: bool) {
        let stats = self
            .rule_stats
            .entry(rule_id.to_string())
            .or_insert(RuleStats {
                execution_count: 0,
                total_time: Duration::ZERO,
                failure_count: 0,
                avg_time: Duration::ZERO,
            });

        stats.execution_count += 1;
        stats.total_time += duration;
        if failed {
            stats.failure_count += 1;
        }
        stats.avg_time = stats.total_time / stats.execution_count as u32;
    }

    fn calculate_summary(&self, results: &[RuleResult]) -> ValidationSummary {
        let total_rules = results.len();
        let passed_rules = results.iter().filter(|r| r.passed).count();
        let failed_rules = total_rules - passed_rules;

        let mut critical_issues = 0;
        let mut errors = 0;
        let mut warnings = 0;

        for result in results {
            if !result.passed {
                if let Some(rule) = self.rules.iter().find(|r| r.id == result.rule_id) {
                    match rule.severity {
                        ValidationSeverity::Critical => critical_issues += 1,
                        ValidationSeverity::Error => errors += 1,
                        ValidationSeverity::Warning => warnings += 1,
                        _ => {}
                    }
                }
            }
        }

        let quality_score = if total_rules > 0 {
            passed_rules as f64 / total_rules as f64
        } else {
            1.0
        };

        ValidationSummary {
            total_rules,
            passed_rules,
            failed_rules,
            critical_issues,
            errors,
            warnings,
            quality_score,
        }
    }

    fn determine_status(&self, summary: &ValidationSummary) -> ValidationStatus {
        if summary.critical_issues > 0 {
            ValidationStatus::CriticalIssuesFound
        } else if summary.errors > 0 {
            ValidationStatus::ErrorsFound
        } else if summary.warnings > 0 {
            ValidationStatus::WarningsFound
        } else {
            ValidationStatus::Passed
        }
    }
}

// Validation rule implementations
fn validate_memory_leaks(context: &ValidationContext) -> Result<(), ValidationError> {
    if context.memory_info.growth_rate > 10.0 * 1024.0 * 1024.0 {
        // 10MB/sec
        return Err(ValidationError {
            message: format!(
                "High memory growth rate detected: {:.2}MB/sec",
                context.memory_info.growth_rate / (1024.0 * 1024.0)
            ),
            suggestion: Some("Check for memory leaks in allocation tracking".to_string()),
            location: Some(context.operation_name.clone()),
        });
    }

    if context.memory_info.fragmentation_ratio > 0.5 {
        return Err(ValidationError {
            message: format!(
                "High memory fragmentation: {:.1}%",
                context.memory_info.fragmentation_ratio * 100.0
            ),
            suggestion: Some("Consider implementing memory compaction".to_string()),
            location: Some(context.operation_name.clone()),
        });
    }

    Ok(())
}

fn validate_allocation_overhead(context: &ValidationContext) -> Result<(), ValidationError> {
    let overhead_ratio =
        context.metrics.peak_memory as f64 / (context.memory_info.current_usage as f64).max(1.0);

    if overhead_ratio > 0.1 {
        // 10% overhead threshold
        return Err(ValidationError {
            message: format!("High tracking overhead: {:.1}%", overhead_ratio * 100.0),
            suggestion: Some(
                "Optimize tracking data structures to reduce memory overhead".to_string(),
            ),
            location: Some(context.operation_name.clone()),
        });
    }

    Ok(())
}

fn validate_tracking_latency(context: &ValidationContext) -> Result<(), ValidationError> {
    if context.metrics.avg_duration > Duration::from_micros(100) {
        return Err(ValidationError {
            message: format!(
                "High tracking latency: {:.2}µs",
                context.metrics.avg_duration.as_micros()
            ),
            suggestion: Some("Optimize allocation tracking path for lower latency".to_string()),
            location: Some(context.operation_name.clone()),
        });
    }

    Ok(())
}

fn validate_symbol_performance(context: &ValidationContext) -> Result<(), ValidationError> {
    if context.operation_name.contains("symbol")
        && context.metrics.avg_duration > Duration::from_millis(10)
    {
        return Err(ValidationError {
            message: format!(
                "Slow symbol resolution: {:.2}ms",
                context.metrics.avg_duration.as_millis()
            ),
            suggestion: Some("Consider symbol caching or preloading".to_string()),
            location: Some(context.operation_name.clone()),
        });
    }

    Ok(())
}

fn validate_error_handling(context: &ValidationContext) -> Result<(), ValidationError> {
    let coverage_ratio = if context.error_handling.error_points > 0 {
        context.error_handling.handled_error_points as f64
            / context.error_handling.error_points as f64
    } else {
        1.0
    };

    if coverage_ratio < 0.9 {
        return Err(ValidationError {
            message: format!(
                "Low error handling coverage: {:.1}%",
                coverage_ratio * 100.0
            ),
            suggestion: Some("Add error handling for unhandled error points".to_string()),
            location: Some(context.operation_name.clone()),
        });
    }

    if !context.error_handling.has_recovery && context.operation_name.contains("critical") {
        return Err(ValidationError {
            message: "Critical operation lacks recovery mechanism".to_string(),
            suggestion: Some("Implement recovery strategies for critical operations".to_string()),
            location: Some(context.operation_name.clone()),
        });
    }

    Ok(())
}

fn validate_thread_safety(context: &ValidationContext) -> Result<(), ValidationError> {
    if context.thread_safety.shared_resources > 0 && !context.thread_safety.is_thread_safe {
        return Err(ValidationError {
            message: "Operation accesses shared resources without thread safety".to_string(),
            suggestion: Some("Add proper synchronization for shared resource access".to_string()),
            location: Some(context.operation_name.clone()),
        });
    }

    if context.thread_safety.contention_level > 0.3 {
        return Err(ValidationError {
            message: format!(
                "High lock contention: {:.1}%",
                context.thread_safety.contention_level * 100.0
            ),
            suggestion: Some(
                "Consider lock-free alternatives or finer-grained locking".to_string(),
            ),
            location: Some(context.operation_name.clone()),
        });
    }

    Ok(())
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            fail_fast: false,
            max_validation_time: Duration::from_secs(10),
            enable_deep_checks: true,
            min_severity: ValidationSeverity::Info,
        }
    }
}

impl Default for QualityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = QualityValidator::new();
        assert!(!validator.rules.is_empty());
        assert!(validator.rule_stats.is_empty());
    }

    #[test]
    fn test_rule_management() {
        let mut validator = QualityValidator::new();
        let initial_count = validator.rules.len();

        let custom_rule = ValidationRule {
            id: "test_rule".to_string(),
            name: "Test Rule".to_string(),
            description: "Test rule description".to_string(),
            category: RuleCategory::CodeStyle,
            severity: ValidationSeverity::Info,
            enabled: true,
            validator: |_| Ok(()),
        };

        validator.add_rule(custom_rule);
        assert_eq!(validator.rules.len(), initial_count + 1);

        assert!(validator.remove_rule("test_rule"));
        assert_eq!(validator.rules.len(), initial_count);
        assert!(!validator.remove_rule("nonexistent_rule"));
    }

    #[test]
    fn test_validation_context() {
        let context = ValidationContext {
            operation_name: "test_operation".to_string(),
            metrics: OperationMetrics {
                avg_duration: Duration::from_micros(50),
                peak_memory: 1024,
                success_rate: 0.95,
                allocation_count: 100,
                cpu_usage: 5.0,
            },
            memory_info: MemoryInfo {
                current_usage: 1024 * 1024,
                peak_usage: 2 * 1024 * 1024,
                active_allocations: 100,
                fragmentation_ratio: 0.1,
                growth_rate: 0.0,
            },
            error_handling: ErrorHandlingInfo {
                has_error_handling: true,
                error_points: 10,
                handled_error_points: 9,
                has_recovery: true,
            },
            thread_safety: ThreadSafetyInfo {
                is_thread_safe: true,
                shared_resources: 2,
                has_synchronization: true,
                contention_level: 0.1,
            },
        };

        let mut validator = QualityValidator::new();
        let result = validator.validate(&context);

        assert!(matches!(
            result.status,
            ValidationStatus::Passed | ValidationStatus::WarningsFound
        ));
        assert!(result.summary.quality_score >= 0.0);
        assert!(result.summary.quality_score <= 1.0);
    }

    #[test]
    fn test_validator_default() {
        let validator = QualityValidator::default();
        assert!(!validator.rules.is_empty());
    }

    #[test]
    fn test_validator_with_config() {
        let config = ValidationConfig {
            fail_fast: true,
            max_validation_time: Duration::from_secs(5),
            enable_deep_checks: false,
            min_severity: ValidationSeverity::Warning,
        };
        let validator = QualityValidator::with_config(config);
        assert!(!validator.rules.is_empty());
    }

    #[test]
    fn test_validation_config_default() {
        let config = ValidationConfig::default();
        assert!(!config.fail_fast);
        assert_eq!(config.max_validation_time, Duration::from_secs(10));
        assert!(config.enable_deep_checks);
        assert_eq!(config.min_severity, ValidationSeverity::Info);
    }

    #[test]
    fn test_set_rule_enabled() {
        let mut validator = QualityValidator::new();

        assert!(validator.set_rule_enabled("memory_leak_check", false));
        let rule = validator
            .rules
            .iter()
            .find(|r| r.id == "memory_leak_check")
            .unwrap();
        assert!(!rule.enabled);

        assert!(validator.set_rule_enabled("memory_leak_check", true));
        let rule = validator
            .rules
            .iter()
            .find(|r| r.id == "memory_leak_check")
            .unwrap();
        assert!(rule.enabled);

        assert!(!validator.set_rule_enabled("nonexistent_rule", true));
    }

    #[test]
    fn test_get_rule_statistics() {
        let mut validator = QualityValidator::new();
        let context = create_passing_context();

        validator.validate(&context);

        let stats = validator.get_rule_statistics();
        assert!(!stats.is_empty());
    }

    #[test]
    fn test_reset_statistics() {
        let mut validator = QualityValidator::new();
        let context = create_passing_context();

        validator.validate(&context);
        assert!(!validator.rule_stats.is_empty());

        validator.reset_statistics();
        assert!(validator.rule_stats.is_empty());
    }

    #[test]
    fn test_validate_memory_leaks_high_growth_rate() {
        let mut context = create_passing_context();
        context.memory_info.growth_rate = 20.0 * 1024.0 * 1024.0; // 20MB/sec

        let result = validate_memory_leaks(&context);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("High memory growth rate"));
    }

    #[test]
    fn test_validate_memory_leaks_high_fragmentation() {
        let mut context = create_passing_context();
        context.memory_info.fragmentation_ratio = 0.6;

        let result = validate_memory_leaks(&context);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("High memory fragmentation"));
    }

    #[test]
    fn test_validate_memory_leaks_passing() {
        let context = create_passing_context();
        let result = validate_memory_leaks(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_allocation_overhead_high() {
        let mut context = create_passing_context();
        context.metrics.peak_memory = 500 * 1024; // High overhead
        context.memory_info.current_usage = 1024;

        let result = validate_allocation_overhead(&context);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("High tracking overhead"));
    }

    #[test]
    fn test_validate_allocation_overhead_passing() {
        let context = create_passing_context();
        let result = validate_allocation_overhead(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tracking_latency_high() {
        let mut context = create_passing_context();
        context.metrics.avg_duration = Duration::from_micros(200);

        let result = validate_tracking_latency(&context);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("High tracking latency"));
    }

    #[test]
    fn test_validate_tracking_latency_passing() {
        let context = create_passing_context();
        let result = validate_tracking_latency(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_symbol_performance_slow() {
        let mut context = create_passing_context();
        context.operation_name = "symbol_resolution".to_string();
        context.metrics.avg_duration = Duration::from_millis(20);

        let result = validate_symbol_performance(&context);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("Slow symbol resolution"));
    }

    #[test]
    fn test_validate_symbol_performance_passing() {
        let mut context = create_passing_context();
        context.operation_name = "symbol_resolution".to_string();
        context.metrics.avg_duration = Duration::from_millis(5);

        let result = validate_symbol_performance(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_error_handling_low_coverage() {
        let mut context = create_passing_context();
        context.error_handling.error_points = 10;
        context.error_handling.handled_error_points = 5; // 50% coverage

        let result = validate_error_handling(&context);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("Low error handling coverage"));
    }

    #[test]
    fn test_validate_error_handling_critical_no_recovery() {
        let mut context = create_passing_context();
        context.operation_name = "critical_operation".to_string();
        context.error_handling.has_recovery = false;

        let result = validate_error_handling(&context);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("Critical operation lacks recovery"));
    }

    #[test]
    fn test_validate_error_handling_passing() {
        let context = create_passing_context();
        let result = validate_error_handling(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_thread_safety_not_safe() {
        let mut context = create_passing_context();
        context.thread_safety.is_thread_safe = false;
        context.thread_safety.shared_resources = 2;

        let result = validate_thread_safety(&context);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("shared resources without thread safety"));
    }

    #[test]
    fn test_validate_thread_safety_high_contention() {
        let mut context = create_passing_context();
        context.thread_safety.contention_level = 0.5;

        let result = validate_thread_safety(&context);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("High lock contention"));
    }

    #[test]
    fn test_validate_thread_safety_passing() {
        let context = create_passing_context();
        let result = validate_thread_safety(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_status_critical() {
        let mut validator = QualityValidator::new();
        let mut context = create_passing_context();
        context.memory_info.growth_rate = 20.0 * 1024.0 * 1024.0;

        let result = validator.validate(&context);
        assert_eq!(result.status, ValidationStatus::CriticalIssuesFound);
        assert!(result.summary.critical_issues > 0);
    }

    #[test]
    fn test_validation_status_errors_found() {
        let mut validator = QualityValidator::new();
        let mut context = create_passing_context();
        context.metrics.avg_duration = Duration::from_micros(200);

        let result = validator.validate(&context);
        assert!(matches!(
            result.status,
            ValidationStatus::ErrorsFound | ValidationStatus::CriticalIssuesFound
        ));
    }

    #[test]
    fn test_validation_status_warnings_found() {
        let mut validator = QualityValidator::new();
        let mut context = create_passing_context();
        context.metrics.peak_memory = 500 * 1024;
        context.memory_info.current_usage = 1024;

        let result = validator.validate(&context);
        assert!(matches!(
            result.status,
            ValidationStatus::WarningsFound | ValidationStatus::Passed
        ));
    }

    #[test]
    fn test_validation_status_passed() {
        let mut validator = QualityValidator::new();
        let context = create_passing_context();

        let result = validator.validate(&context);
        assert!(matches!(
            result.status,
            ValidationStatus::Passed | ValidationStatus::WarningsFound
        ));
    }

    #[test]
    fn test_fail_fast_config() {
        let config = ValidationConfig {
            fail_fast: true,
            ..Default::default()
        };
        let mut validator = QualityValidator::with_config(config);
        let mut context = create_passing_context();
        context.memory_info.growth_rate = 20.0 * 1024.0 * 1024.0;

        let result = validator.validate(&context);
        assert_eq!(result.status, ValidationStatus::CriticalIssuesFound);
    }

    #[test]
    fn test_rule_category_variants() {
        let categories = vec![
            RuleCategory::MemorySafety,
            RuleCategory::Performance,
            RuleCategory::CodeStyle,
            RuleCategory::ErrorHandling,
            RuleCategory::ThreadSafety,
            RuleCategory::ResourceManagement,
        ];

        for category in categories {
            let rule = ValidationRule {
                id: "test".to_string(),
                name: "Test".to_string(),
                description: String::new(),
                category: category.clone(),
                severity: ValidationSeverity::Info,
                enabled: true,
                validator: |_| Ok(()),
            };
            assert_eq!(rule.category, category);
        }
    }

    #[test]
    fn test_validation_severity_ordering() {
        assert!(ValidationSeverity::Critical > ValidationSeverity::Error);
        assert!(ValidationSeverity::Error > ValidationSeverity::Warning);
        assert!(ValidationSeverity::Warning > ValidationSeverity::Style);
        assert!(ValidationSeverity::Style > ValidationSeverity::Info);
    }

    #[test]
    fn test_validation_severity_equality() {
        assert_eq!(ValidationSeverity::Critical, ValidationSeverity::Critical);
        assert_ne!(ValidationSeverity::Critical, ValidationSeverity::Error);
    }

    #[test]
    fn test_validation_error_creation() {
        let error = ValidationError {
            message: "Test error".to_string(),
            suggestion: Some("Fix it".to_string()),
            location: Some("test.rs:10".to_string()),
        };

        assert_eq!(error.message, "Test error");
        assert!(error.suggestion.is_some());
        assert!(error.location.is_some());
    }

    #[test]
    fn test_validation_error_no_suggestion() {
        let error = ValidationError {
            message: "Test error".to_string(),
            suggestion: None,
            location: None,
        };

        assert!(error.suggestion.is_none());
        assert!(error.location.is_none());
    }

    #[test]
    fn test_rule_result_passed() {
        let result = RuleResult {
            rule_id: "test_rule".to_string(),
            passed: true,
            error: None,
            execution_time: Duration::from_micros(10),
        };

        assert!(result.passed);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_rule_result_failed() {
        let result = RuleResult {
            rule_id: "test_rule".to_string(),
            passed: false,
            error: Some(ValidationError {
                message: "Failed".to_string(),
                suggestion: None,
                location: None,
            }),
            execution_time: Duration::from_micros(10),
        };

        assert!(!result.passed);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_validation_summary_creation() {
        let summary = ValidationSummary {
            total_rules: 10,
            passed_rules: 8,
            failed_rules: 2,
            critical_issues: 1,
            errors: 1,
            warnings: 0,
            quality_score: 0.8,
        };

        assert_eq!(summary.total_rules, 10);
        assert_eq!(summary.passed_rules, 8);
        assert!((summary.quality_score - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_operation_metrics_creation() {
        let metrics = OperationMetrics {
            avg_duration: Duration::from_micros(50),
            peak_memory: 1024,
            success_rate: 0.95,
            allocation_count: 100,
            cpu_usage: 5.0,
        };

        assert_eq!(metrics.avg_duration, Duration::from_micros(50));
        assert_eq!(metrics.allocation_count, 100);
    }

    #[test]
    fn test_memory_info_creation() {
        let info = MemoryInfo {
            current_usage: 1024,
            peak_usage: 2048,
            active_allocations: 10,
            fragmentation_ratio: 0.1,
            growth_rate: 100.0,
        };

        assert_eq!(info.current_usage, 1024);
        assert_eq!(info.peak_usage, 2048);
    }

    #[test]
    fn test_error_handling_info_creation() {
        let info = ErrorHandlingInfo {
            has_error_handling: true,
            error_points: 5,
            handled_error_points: 4,
            has_recovery: true,
        };

        assert!(info.has_error_handling);
        assert_eq!(info.error_points, 5);
    }

    #[test]
    fn test_thread_safety_info_creation() {
        let info = ThreadSafetyInfo {
            is_thread_safe: true,
            shared_resources: 3,
            has_synchronization: true,
            contention_level: 0.1,
        };

        assert!(info.is_thread_safe);
        assert_eq!(info.shared_resources, 3);
    }

    #[test]
    fn test_validation_result_creation() {
        let result = ValidationResult {
            status: ValidationStatus::Passed,
            rule_results: vec![],
            summary: ValidationSummary {
                total_rules: 0,
                passed_rules: 0,
                failed_rules: 0,
                critical_issues: 0,
                errors: 0,
                warnings: 0,
                quality_score: 1.0,
            },
            validation_overhead: Duration::from_micros(100),
        };

        assert_eq!(result.status, ValidationStatus::Passed);
        assert!(result.rule_results.is_empty());
    }

    #[test]
    fn test_validation_status_equality() {
        assert_eq!(ValidationStatus::Passed, ValidationStatus::Passed);
        assert_ne!(ValidationStatus::Passed, ValidationStatus::ErrorsFound);
    }

    #[test]
    fn test_rule_stats_creation() {
        let stats = RuleStats {
            execution_count: 10,
            total_time: Duration::from_millis(100),
            failure_count: 2,
            avg_time: Duration::from_millis(10),
        };

        assert_eq!(stats.execution_count, 10);
        assert_eq!(stats.failure_count, 2);
    }

    #[test]
    fn test_disabled_rule_not_executed() {
        let mut validator = QualityValidator::new();
        validator.set_rule_enabled("memory_leak_check", false);

        let mut context = create_passing_context();
        context.memory_info.growth_rate = 20.0 * 1024.0 * 1024.0;

        let result = validator.validate(&context);
        assert_ne!(result.status, ValidationStatus::CriticalIssuesFound);
    }

    #[test]
    fn test_multiple_validations_accumulate_stats() {
        let mut validator = QualityValidator::new();
        let context = create_passing_context();

        validator.validate(&context);
        validator.validate(&context);
        validator.validate(&context);

        let stats = validator.get_rule_statistics();
        for stat in stats.values() {
            assert_eq!(stat.execution_count, 3);
        }
    }

    #[test]
    fn test_validation_rule_debug() {
        let rule = ValidationRule {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            category: RuleCategory::MemorySafety,
            severity: ValidationSeverity::Info,
            enabled: true,
            validator: |_| Ok(()),
        };

        let debug_str = format!("{:?}", rule);
        assert!(debug_str.contains("ValidationRule"));
    }

    #[test]
    fn test_validation_rule_clone() {
        let rule = ValidationRule {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            category: RuleCategory::MemorySafety,
            severity: ValidationSeverity::Info,
            enabled: true,
            validator: |_| Ok(()),
        };

        let cloned = rule.clone();
        assert_eq!(cloned.id, rule.id);
        assert_eq!(cloned.severity, rule.severity);
    }

    #[test]
    fn test_validation_context_debug() {
        let context = create_passing_context();
        let debug_str = format!("{:?}", context);
        assert!(debug_str.contains("ValidationContext"));
    }

    #[test]
    fn test_zero_error_points_coverage() {
        let mut context = create_passing_context();
        context.error_handling.error_points = 0;
        context.error_handling.handled_error_points = 0;

        let result = validate_error_handling(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_quality_score_calculation() {
        let mut validator = QualityValidator::new();
        let context = create_passing_context();

        let result = validator.validate(&context);
        let expected_score = result.summary.passed_rules as f64 / result.summary.total_rules as f64;
        assert!((result.summary.quality_score - expected_score).abs() < f64::EPSILON);
    }

    fn create_passing_context() -> ValidationContext {
        ValidationContext {
            operation_name: "test_operation".to_string(),
            metrics: OperationMetrics {
                avg_duration: Duration::from_micros(10),
                peak_memory: 100,
                success_rate: 0.99,
                allocation_count: 10,
                cpu_usage: 1.0,
            },
            memory_info: MemoryInfo {
                current_usage: 1024 * 1024,
                peak_usage: 2 * 1024 * 1024,
                active_allocations: 10,
                fragmentation_ratio: 0.1,
                growth_rate: 0.0,
            },
            error_handling: ErrorHandlingInfo {
                has_error_handling: true,
                error_points: 10,
                handled_error_points: 10,
                has_recovery: true,
            },
            thread_safety: ThreadSafetyInfo {
                is_thread_safe: true,
                shared_resources: 0,
                has_synchronization: true,
                contention_level: 0.0,
            },
        }
    }
}
