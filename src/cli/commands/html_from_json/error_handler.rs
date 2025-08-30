//! Comprehensive error handling module for HTML generation
//!
//! This module provides detailed error handling, recovery mechanisms,
//! and user-friendly error messages for the HTML generation process.

use serde_json::Value;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::time::Instant;

/// Comprehensive error types for HTML generation
#[derive(Debug)]
pub enum HtmlGenerationError {
    /// JSON file discovery failed
    FileDiscoveryError {
        /// Directory that was searched
        directory: String,
        /// Base name pattern used
        base_name: String,
        /// Underlying error
        source: Box<dyn Error + Send + Sync>,
        /// Suggested recovery actions
        recovery_suggestions: Vec<String>,
    },
    /// JSON file loading failed
    FileLoadingError {
        /// Path to the file that failed
        file_path: PathBuf,
        /// File type (e.g., "memory_analysis")
        file_type: String,
        /// File size in bytes
        file_size: usize,
        /// Underlying error
        source: Box<dyn Error + Send + Sync>,
        /// Whether this is a recoverable error
        recoverable: bool,
        /// Recovery suggestions
        recovery_suggestions: Vec<String>,
    },
    /// JSON parsing failed
    JsonParsingError {
        /// Path to the file with parsing error
        file_path: PathBuf,
        /// Line number where error occurred (if available)
        line_number: Option<usize>,
        /// Column number where error occurred (if available)
        column_number: Option<usize>,
        /// Parsing error details
        parsing_error: String,
        /// JSON content snippet around error (if available)
        content_snippet: Option<String>,
        /// Recovery suggestions
        recovery_suggestions: Vec<String>,
    },
    /// Data validation failed
    ValidationError {
        /// File that failed validation
        file_path: PathBuf,
        /// Type of validation that failed
        validation_type: String,
        /// Specific validation error
        validation_error: String,
        /// Expected data structure
        expected_structure: String,
        /// Actual data structure found
        actual_structure: String,
        /// Recovery suggestions
        recovery_suggestions: Vec<String>,
    },
    /// Data normalization failed
    NormalizationError {
        /// Stage where normalization failed
        stage: String,
        /// Number of items processed before failure
        processed_count: usize,
        /// Total items to process
        total_count: usize,
        /// Underlying error
        source: Box<dyn Error + Send + Sync>,
        /// Whether partial data can be used
        partial_data_available: bool,
        /// Recovery suggestions
        recovery_suggestions: Vec<String>,
    },
    /// Template generation failed
    TemplateError {
        /// Template stage that failed
        stage: String,
        /// Template size processed so far
        processed_size: usize,
        /// Underlying error
        source: Box<dyn Error + Send + Sync>,
        /// Recovery suggestions
        recovery_suggestions: Vec<String>,
    },
    /// Memory limit exceeded
    MemoryLimitError {
        /// Current memory usage in bytes
        current_usage: usize,
        /// Memory limit in bytes
        memory_limit: usize,
        /// Operation that caused the limit to be exceeded
        operation: String,
        /// Recovery suggestions
        recovery_suggestions: Vec<String>,
    },
    /// Multiple errors occurred
    MultipleErrors {
        /// List of errors that occurred
        errors: Vec<HtmlGenerationError>,
        /// Whether processing can continue with partial data
        can_continue: bool,
        /// Recovery suggestions
        recovery_suggestions: Vec<String>,
    },
}

impl fmt::Display for HtmlGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HtmlGenerationError::FileDiscoveryError {
                directory,
                base_name,
                source,
                recovery_suggestions,
            } => {
                writeln!(f, "❌ File Discovery Error")?;
                writeln!(f, "   Directory: {directory}")?;
                writeln!(f, "   Base name: {base_name}")?;
                writeln!(f, "   Error: {source}")?;
                if !recovery_suggestions.is_empty() {
                    writeln!(f, "   💡 Suggestions:")?;
                    for suggestion in recovery_suggestions {
                        writeln!(f, "      • {suggestion}")?;
                    }
                }
                Ok(())
            }
            HtmlGenerationError::FileLoadingError {
                file_path,
                file_type,
                file_size,
                source,
                recoverable,
                recovery_suggestions,
            } => {
                writeln!(f, "❌ File Loading Error")?;
                writeln!(f, "   File: {}", file_path.display())?;
                writeln!(f, "   Type: {file_type}")?;
                writeln!(f, "   Size: {:.1} KB", *file_size as f64 / 1024.0)?;
                writeln!(f, "   Error: {source}")?;
                writeln!(
                    f,
                    "   Recoverable: {}",
                    if *recoverable { "Yes" } else { "No" }
                )?;
                if !recovery_suggestions.is_empty() {
                    writeln!(f, "   💡 Suggestions:")?;
                    for suggestion in recovery_suggestions {
                        writeln!(f, "      • {suggestion}")?;
                    }
                }
                Ok(())
            }
            HtmlGenerationError::JsonParsingError {
                file_path,
                line_number,
                column_number,
                parsing_error,
                content_snippet,
                recovery_suggestions,
            } => {
                writeln!(f, "❌ JSON Parsing Error")?;
                writeln!(f, "   File: {}", file_path.display())?;
                if let Some(line) = line_number {
                    write!(f, "   Location: line {line}")?;
                    if let Some(col) = column_number {
                        write!(f, ", column {col}")?;
                    }
                    writeln!(f)?;
                }
                writeln!(f, "   Error: {parsing_error}")?;
                if let Some(snippet) = content_snippet {
                    writeln!(f, "   Context:")?;
                    writeln!(f, "   {snippet}")?;
                }
                if !recovery_suggestions.is_empty() {
                    writeln!(f, "   💡 Suggestions:")?;
                    for suggestion in recovery_suggestions {
                        writeln!(f, "      • {suggestion}")?;
                    }
                }
                Ok(())
            }
            HtmlGenerationError::ValidationError {
                file_path,
                validation_type,
                validation_error,
                expected_structure,
                actual_structure,
                recovery_suggestions,
            } => {
                writeln!(f, "❌ Data Validation Error")?;
                writeln!(f, "   File: {}", file_path.display())?;
                writeln!(f, "   Validation: {validation_type}")?;
                writeln!(f, "   Error: {validation_error}")?;
                writeln!(f, "   Expected: {expected_structure}")?;
                writeln!(f, "   Found: {actual_structure}")?;
                if !recovery_suggestions.is_empty() {
                    writeln!(f, "   💡 Suggestions:")?;
                    for suggestion in recovery_suggestions {
                        writeln!(f, "      • {suggestion}")?;
                    }
                }
                Ok(())
            }
            HtmlGenerationError::NormalizationError {
                stage,
                processed_count,
                total_count,
                source,
                partial_data_available,
                recovery_suggestions,
            } => {
                writeln!(f, "❌ Data Normalization Error")?;
                writeln!(f, "   Stage: {stage}")?;
                writeln!(
                    f,
                    "   Progress: {processed_count}/{total_count} ({:.1}%)",
                    (*processed_count as f64 / *total_count as f64) * 100.0
                )?;
                writeln!(f, "   Error: {source}")?;
                writeln!(
                    f,
                    "   Partial data available: {}",
                    if *partial_data_available { "Yes" } else { "No" }
                )?;
                if !recovery_suggestions.is_empty() {
                    writeln!(f, "   💡 Suggestions:")?;
                    for suggestion in recovery_suggestions {
                        writeln!(f, "      • {suggestion}")?;
                    }
                }
                Ok(())
            }
            HtmlGenerationError::TemplateError {
                stage,
                processed_size,
                source,
                recovery_suggestions,
            } => {
                writeln!(f, "❌ Template Generation Error")?;
                writeln!(f, "   Stage: {stage}")?;
                writeln!(f, "   Processed: {:.1} KB", *processed_size as f64 / 1024.0)?;
                writeln!(f, "   Error: {source}")?;
                if !recovery_suggestions.is_empty() {
                    writeln!(f, "   💡 Suggestions:")?;
                    for suggestion in recovery_suggestions {
                        writeln!(f, "      • {suggestion}")?;
                    }
                }
                Ok(())
            }
            HtmlGenerationError::MemoryLimitError {
                current_usage,
                memory_limit,
                operation,
                recovery_suggestions,
            } => {
                writeln!(f, "❌ Memory Limit Exceeded")?;
                writeln!(
                    f,
                    "   Current usage: {:.1} MB",
                    *current_usage as f64 / 1024.0 / 1024.0
                )?;
                writeln!(
                    f,
                    "   Memory limit: {:.1} MB",
                    *memory_limit as f64 / 1024.0 / 1024.0
                )?;
                writeln!(f, "   Operation: {operation}")?;
                writeln!(
                    f,
                    "   Usage: {:.1}%",
                    (*current_usage as f64 / *memory_limit as f64) * 100.0
                )?;
                if !recovery_suggestions.is_empty() {
                    writeln!(f, "   💡 Suggestions:")?;
                    for suggestion in recovery_suggestions {
                        writeln!(f, "      • {suggestion}")?;
                    }
                }
                Ok(())
            }
            HtmlGenerationError::MultipleErrors {
                errors,
                can_continue,
                recovery_suggestions,
            } => {
                writeln!(f, "❌ Multiple Errors Occurred ({} errors)", errors.len())?;
                writeln!(
                    f,
                    "   Can continue: {}",
                    if *can_continue { "Yes" } else { "No" }
                )?;
                for (i, error) in errors.iter().enumerate() {
                    writeln!(f, "   Error {}: {}", i + 1, error)?;
                }
                if !recovery_suggestions.is_empty() {
                    writeln!(f, "   💡 Suggestions:")?;
                    for suggestion in recovery_suggestions {
                        writeln!(f, "      • {suggestion}")?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl Error for HtmlGenerationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            HtmlGenerationError::FileDiscoveryError { source, .. } => Some(source.as_ref()),
            HtmlGenerationError::FileLoadingError { source, .. } => Some(source.as_ref()),
            HtmlGenerationError::NormalizationError { source, .. } => Some(source.as_ref()),
            HtmlGenerationError::TemplateError { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

/// Error recovery context
#[derive(Debug, Clone)]
pub struct ErrorRecoveryContext {
    /// Whether to attempt recovery
    pub attempt_recovery: bool,
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Whether to continue with partial data
    pub allow_partial_data: bool,
    /// Whether to use fallback mechanisms
    pub use_fallbacks: bool,
    /// Verbose error reporting
    pub verbose_errors: bool,
}

impl Default for ErrorRecoveryContext {
    fn default() -> Self {
        Self {
            attempt_recovery: true,
            max_retries: 3,
            allow_partial_data: true,
            use_fallbacks: true,
            verbose_errors: false,
        }
    }
}

/// Error recovery statistics
#[derive(Debug, Default)]
pub struct ErrorRecoveryStats {
    /// Total errors encountered
    pub total_errors: usize,
    /// Errors successfully recovered
    pub recovered_errors: usize,
    /// Errors that required fallbacks
    pub fallback_errors: usize,
    /// Errors that were unrecoverable
    pub unrecoverable_errors: usize,
    /// Total retry attempts made
    pub retry_attempts: usize,
    /// Time spent on error recovery
    pub recovery_time_ms: u64,
}

/// Comprehensive error handler for HTML generation
pub struct HtmlErrorHandler {
    /// Recovery context configuration
    recovery_context: ErrorRecoveryContext,
    /// Recovery statistics
    stats: ErrorRecoveryStats,
    /// Start time for timing recovery operations
    start_time: Option<Instant>,
}

impl HtmlErrorHandler {
    /// Create a new error handler with default settings
    pub fn new() -> Self {
        Self {
            recovery_context: ErrorRecoveryContext::default(),
            stats: ErrorRecoveryStats::default(),
            start_time: None,
        }
    }

    /// Create error handler with custom recovery context
    pub fn with_context(context: ErrorRecoveryContext) -> Self {
        Self {
            recovery_context: context,
            stats: ErrorRecoveryStats::default(),
            start_time: None,
        }
    }

    /// Handle file discovery errors with recovery
    pub fn handle_file_discovery_error(
        &mut self,
        directory: &str,
        base_name: &str,
        error: Box<dyn Error + Send + Sync>,
    ) -> Result<Vec<String>, HtmlGenerationError> {
        self.start_recovery_timing();
        self.stats.total_errors += 1;

        let recovery_suggestions = vec![
            format!("Check if directory '{directory}' exists and is readable"),
            "Verify the base name pattern matches your JSON files".to_string(),
            "Ensure JSON files follow the naming convention: {base_name}_{type}.json".to_string(),
            "Check file permissions for the directory".to_string(),
        ];

        if self.recovery_context.attempt_recovery {
            // Attempt to find alternative directories or patterns
            if let Ok(alternatives) = self.find_alternative_directories(directory, base_name) {
                if !alternatives.is_empty() {
                    self.stats.recovered_errors += 1;
                    self.end_recovery_timing();
                    return Ok(alternatives);
                }
            }
        }

        self.stats.unrecoverable_errors += 1;
        self.end_recovery_timing();

        Err(HtmlGenerationError::FileDiscoveryError {
            directory: directory.to_string(),
            base_name: base_name.to_string(),
            source: error,
            recovery_suggestions,
        })
    }

    /// Handle file loading errors with recovery
    pub fn handle_file_loading_error(
        &mut self,
        file_path: PathBuf,
        file_type: &str,
        file_size: usize,
        error: Box<dyn Error + Send + Sync>,
    ) -> Result<Option<Value>, HtmlGenerationError> {
        self.start_recovery_timing();
        self.stats.total_errors += 1;

        let recoverable = self.is_file_error_recoverable(&error);
        let recovery_suggestions = self.get_file_loading_suggestions(file_type, file_size, &error);

        if recoverable && self.recovery_context.attempt_recovery {
            // Attempt recovery strategies
            for attempt in 1..=self.recovery_context.max_retries {
                self.stats.retry_attempts += 1;

                if let Ok(recovered_data) = self.attempt_file_recovery(&file_path, file_type) {
                    tracing::info!(
                        "✅ Recovered file {} after {} attempts",
                        file_path.display(),
                        attempt
                    );
                    self.stats.recovered_errors += 1;
                    self.end_recovery_timing();
                    return Ok(Some(recovered_data));
                }

                // Wait before retry
                std::thread::sleep(std::time::Duration::from_millis(100 * attempt as u64));
            }

            // Try fallback if available
            if self.recovery_context.use_fallbacks {
                if let Ok(fallback_data) = self.get_fallback_data(file_type) {
                    tracing::info!("⚠️  Using fallback data for {}", file_type);
                    self.stats.fallback_errors += 1;
                    self.end_recovery_timing();
                    return Ok(Some(fallback_data));
                }
            }
        }

        self.stats.unrecoverable_errors += 1;
        self.end_recovery_timing();

        Err(HtmlGenerationError::FileLoadingError {
            file_path,
            file_type: file_type.to_string(),
            file_size,
            source: error,
            recoverable,
            recovery_suggestions,
        })
    }

    /// Handle JSON parsing errors with detailed context
    pub fn handle_json_parsing_error(
        &mut self,
        file_path: PathBuf,
        parsing_error: &str,
    ) -> HtmlGenerationError {
        self.start_recovery_timing();
        self.stats.total_errors += 1;

        let (line_number, column_number, content_snippet) =
            self.extract_parsing_context(&file_path, parsing_error);

        let recovery_suggestions = vec![
            "Check JSON syntax for missing commas, brackets, or quotes".to_string(),
            "Validate JSON structure using a JSON validator tool".to_string(),
            "Ensure file encoding is UTF-8".to_string(),
            "Check for trailing commas which are not valid in JSON".to_string(),
            "Verify that all strings are properly quoted".to_string(),
        ];

        self.stats.unrecoverable_errors += 1;
        self.end_recovery_timing();

        HtmlGenerationError::JsonParsingError {
            file_path,
            line_number,
            column_number,
            parsing_error: parsing_error.to_string(),
            content_snippet,
            recovery_suggestions,
        }
    }

    /// Handle validation errors with detailed analysis
    pub fn handle_validation_error(
        &mut self,
        file_path: PathBuf,
        validation_type: &str,
        validation_error: &str,
        json_data: &Value,
    ) -> HtmlGenerationError {
        self.start_recovery_timing();
        self.stats.total_errors += 1;

        let expected_structure = self.get_expected_structure(validation_type);
        let actual_structure = self.analyze_json_structure(json_data);

        let recovery_suggestions =
            self.get_validation_recovery_suggestions(validation_type, json_data);

        self.stats.unrecoverable_errors += 1;
        self.end_recovery_timing();

        HtmlGenerationError::ValidationError {
            file_path,
            validation_type: validation_type.to_string(),
            validation_error: validation_error.to_string(),
            expected_structure,
            actual_structure,
            recovery_suggestions,
        }
    }

    /// Get error recovery statistics
    pub fn get_stats(&self) -> &ErrorRecoveryStats {
        &self.stats
    }

    /// Print error recovery summary
    pub fn print_recovery_summary(&self) {
        if self.stats.total_errors == 0 {
            return;
        }

        tracing::info!("\n📊 Error Recovery Summary:");
        tracing::info!("   Total errors: {}", self.stats.total_errors);
        tracing::info!("   Recovered: {}", self.stats.recovered_errors);
        tracing::info!("   Used fallbacks: {}", self.stats.fallback_errors);
        tracing::info!("   Unrecoverable: {}", self.stats.unrecoverable_errors);
        tracing::info!("   Retry attempts: {}", self.stats.retry_attempts);
        tracing::info!("   Recovery time: {}ms", self.stats.recovery_time_ms);

        let success_rate = if self.stats.total_errors > 0 {
            ((self.stats.recovered_errors + self.stats.fallback_errors) as f64
                / self.stats.total_errors as f64)
                * 100.0
        } else {
            100.0
        };
        tracing::info!("   Success rate: {:.1}%", success_rate);
    }

    // Private helper methods

    fn start_recovery_timing(&mut self) {
        self.start_time = Some(Instant::now());
    }

    fn end_recovery_timing(&mut self) {
        if let Some(start) = self.start_time.take() {
            self.stats.recovery_time_ms += start.elapsed().as_millis() as u64;
        }
    }

    fn find_alternative_directories(
        &self,
        _directory: &str,
        _base_name: &str,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        // Implementation would search for alternative directories
        // For now, return empty to indicate no alternatives found
        Ok(vec![])
    }

    fn is_file_error_recoverable(&self, error: &Box<dyn Error + Send + Sync>) -> bool {
        let error_str = error.to_string().to_lowercase();

        // Recoverable errors
        error_str.contains("permission denied")
            || error_str.contains("temporarily unavailable")
            || error_str.contains("resource busy")
            || error_str.contains("interrupted")
    }

    fn get_file_loading_suggestions(
        &self,
        file_type: &str,
        file_size: usize,
        error: &Box<dyn Error + Send + Sync>,
    ) -> Vec<String> {
        let mut suggestions = vec![];
        let error_str = error.to_string().to_lowercase();

        if error_str.contains("permission") {
            suggestions.push("Check file permissions and ensure read access".to_string());
        }

        if error_str.contains("not found") {
            suggestions.push(format!(
                "Verify the {file_type} file exists in the expected location"
            ));
            suggestions
                .push("Check the file naming convention matches the expected pattern".to_string());
        }

        if file_size > 100 * 1024 * 1024 {
            // > 100MB
            suggestions.push("Consider using streaming mode for large files".to_string());
            suggestions.push("Increase memory limits if processing large datasets".to_string());
        }

        suggestions.push("Retry the operation after a brief delay".to_string());
        suggestions.push("Check available disk space and memory".to_string());

        suggestions
    }

    fn attempt_file_recovery(
        &self,
        file_path: &PathBuf,
        _file_type: &str,
    ) -> Result<Value, Box<dyn Error>> {
        // Attempt to read the file again
        let content = std::fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;
        Ok(json_value)
    }

    fn get_fallback_data(&self, file_type: &str) -> Result<Value, Box<dyn Error>> {
        // Provide minimal fallback data structures
        let fallback = match file_type {
            "memory_analysis" => serde_json::json!({
                "allocations": [],
                "summary": {
                    "total_allocations": 0,
                    "active_allocations": 0,
                    "total_memory": 0
                }
            }),
            "performance" => serde_json::json!({
                "memory_performance": {
                    "active_memory": 0,
                    "peak_memory": 0,
                    "total_allocated": 0
                },
                "allocation_distribution": {}
            }),
            "unsafe_ffi" => serde_json::json!({
                "summary": {
                    "unsafe_count": 0,
                    "ffi_count": 0,
                    "safety_violations": 0
                },
                "enhanced_ffi_data": [],
                "boundary_events": []
            }),
            "lifetime" => serde_json::json!({
                "lifecycle_events": []
            }),
            "complex_types" => serde_json::json!({
                "categorized_types": {},
                "generic_types": []
            }),
            _ => serde_json::json!({}),
        };

        Ok(fallback)
    }

    fn extract_parsing_context(
        &self,
        file_path: &PathBuf,
        parsing_error: &str,
    ) -> (Option<usize>, Option<usize>, Option<String>) {
        // Extract line and column information from parsing error
        let line_regex = regex::Regex::new(r"line (\d+)").ok();
        let col_regex = regex::Regex::new(r"column (\d+)").ok();

        let line_number: Option<usize> = line_regex
            .and_then(|re| re.captures(parsing_error))
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse().ok());

        let column_number: Option<usize> = col_regex
            .and_then(|re| re.captures(parsing_error))
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse().ok());

        // Try to read file content around the error
        let content_snippet =
            if let (Some(line), Ok(content)) = (line_number, std::fs::read_to_string(file_path)) {
                let lines: Vec<&str> = content.lines().collect();
                if line > 0 && line <= lines.len() {
                    let start = (line.saturating_sub(3)).max(1);
                    let end = (line + 2).min(lines.len());
                    let snippet_lines: Vec<String> = (start..=end)
                        .map(|i| {
                            let marker = if i == line { ">>> " } else { "    " };
                            format!(
                                "{marker}{i:3}: {}",
                                lines.get(i.saturating_sub(1)).unwrap_or(&"")
                            )
                        })
                        .collect();
                    Some(snippet_lines.join("\n"))
                } else {
                    None
                }
            } else {
                None
            };

        (line_number, column_number, content_snippet)
    }

    fn get_expected_structure(&self, validation_type: &str) -> String {
        match validation_type {
            "memory_analysis" => "Object with 'allocations' array and 'summary' object".to_string(),
            "performance" => {
                "Object with 'memory_performance' and 'allocation_distribution'".to_string()
            }
            "unsafe_ffi" => {
                "Object with 'summary', 'enhanced_ffi_data', and 'boundary_events'".to_string()
            }
            "lifetime" => "Object with 'lifecycle_events' array".to_string(),
            "complex_types" => "Object with 'categorized_types' and 'generic_types'".to_string(),
            _ => "Valid JSON object or array".to_string(),
        }
    }

    fn analyze_json_structure(&self, json_data: &Value) -> String {
        match json_data {
            Value::Object(obj) => {
                let keys: Vec<String> = obj.keys().cloned().collect();
                format!("Object with keys: [{}]", keys.join(", "))
            }
            Value::Array(arr) => {
                format!("Array with {} elements", arr.len())
            }
            Value::String(_) => "String value".to_string(),
            Value::Number(_) => "Number value".to_string(),
            Value::Bool(_) => "Boolean value".to_string(),
            Value::Null => "Null value".to_string(),
        }
    }

    fn get_validation_recovery_suggestions(
        &self,
        validation_type: &str,
        json_data: &Value,
    ) -> Vec<String> {
        let mut suggestions = vec![];

        match validation_type {
            "memory_analysis" => {
                suggestions.push("Ensure the JSON contains an 'allocations' array".to_string());
                suggestions.push("Add a 'summary' object with allocation statistics".to_string());
                if let Value::Object(obj) = json_data {
                    if !obj.contains_key("allocations") {
                        suggestions.push("Add missing 'allocations' field as an array".to_string());
                    }
                    if !obj.contains_key("summary") {
                        suggestions.push("Add missing 'summary' field as an object".to_string());
                    }
                }
            }
            "unsafe_ffi" => {
                suggestions.push("Ensure the JSON contains 'enhanced_ffi_data' array".to_string());
                suggestions
                    .push("Add 'boundary_events' array for FFI boundary tracking".to_string());
                suggestions.push("Include 'summary' object with FFI statistics".to_string());
            }
            _ => {
                suggestions
                    .push("Check the JSON structure matches the expected format".to_string());
                suggestions
                    .push("Refer to the documentation for the correct data format".to_string());
            }
        }

        suggestions.push("Validate JSON syntax and structure".to_string());
        suggestions.push("Check for required fields and correct data types".to_string());

        suggestions
    }
}

impl Default for HtmlErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_handler_creation() {
        let handler = HtmlErrorHandler::new();
        assert_eq!(handler.stats.total_errors, 0);
        assert!(handler.recovery_context.attempt_recovery);
    }

    #[test]
    fn test_file_loading_error_handling() {
        let mut handler = HtmlErrorHandler::new();
        let error = Box::new(io::Error::new(io::ErrorKind::NotFound, "File not found"));

        let result = handler.handle_file_loading_error(
            PathBuf::from("test.json"),
            "memory_analysis",
            1024,
            error,
        );

        assert!(result.is_err());
        assert_eq!(handler.stats.total_errors, 1);
    }

    #[test]
    fn test_fallback_data_generation() {
        let handler = HtmlErrorHandler::new();

        let fallback = handler
            .get_fallback_data("memory_analysis")
            .expect("Failed to get fallback data");
        assert!(fallback.is_object());
        assert!(fallback.get("allocations").is_some());
        assert!(fallback.get("summary").is_some());
    }

    #[test]
    fn test_error_recovery_context() {
        let context = ErrorRecoveryContext {
            attempt_recovery: false,
            max_retries: 1,
            allow_partial_data: false,
            use_fallbacks: false,
            verbose_errors: true,
        };

        let handler = HtmlErrorHandler::with_context(context);
        assert!(!handler.recovery_context.attempt_recovery);
        assert_eq!(handler.recovery_context.max_retries, 1);
    }
}
