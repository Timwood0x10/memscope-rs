//! Comprehensive error handling for binary data processing
//!
//! This module provides detailed error types, recovery strategies, and
//! structured logging for all binary data operations.

use std::fmt;
use thiserror::Error;

/// Comprehensive error types for binary data processing operations
#[derive(Debug, Error)]
pub enum BinaryProcessingError {
    /// File system related errors
    #[error("File system error: {message}")]
    FileSystem {
        /// Error message
        message: String,
        /// File path that caused the error
        path: Option<String>,
        /// Underlying IO error code
        error_code: Option<i32>,
    },

    /// Compression and decompression errors
    #[error("Compression error: {message}")]
    Compression {
        /// Error message
        message: String,
        /// Compression algorithm that failed
        algorithm: String,
        /// Compression level used
        level: Option<i32>,
        /// Whether this was compression or decompression
        operation: CompressionOperation,
    },

    /// Data serialization and deserialization errors
    #[error("Serialization error: {message}")]
    Serialization {
        /// Error message
        message: String,
        /// Data format (MessagePack, JSON, etc.)
        format: String,
        /// Byte position where error occurred (if known)
        position: Option<usize>,
        /// Whether this was serialization or deserialization
        operation: SerializationOperation,
    },

    /// Data validation and integrity errors
    #[error("Validation error: {message}")]
    Validation {
        /// Error message
        message: String,
        /// Type of validation that failed
        validation_type: ValidationType,
        /// Expected value (if applicable)
        expected: Option<String>,
        /// Actual value found
        actual: Option<String>,
    },

    /// Memory and resource limit errors
    #[error("Resource limit exceeded: {message}")]
    ResourceLimit {
        /// Error message
        message: String,
        /// Type of resource that was exceeded
        resource_type: ResourceType,
        /// Limit that was exceeded
        limit: u64,
        /// Actual usage that exceeded the limit
        usage: u64,
    },

    /// Configuration and parameter errors
    #[error("Configuration error: {message}")]
    Configuration {
        /// Error message
        message: String,
        /// Parameter name that caused the error
        parameter: String,
        /// Invalid value provided
        value: String,
        /// Valid range or options
        valid_options: Vec<String>,
    },

    /// Network or external service errors
    #[error("External service error: {message}")]
    ExternalService {
        /// Error message
        message: String,
        /// Service name
        service: String,
        /// HTTP status code (if applicable)
        status_code: Option<u16>,
    },

    /// Template and rendering errors
    #[error("Template error: {message}")]
    Template {
        /// Error message
        message: String,
        /// Template name or path
        template: String,
        /// Line number where error occurred (if known)
        line: Option<usize>,
    },

    /// Generic processing errors with context
    #[error("Processing error: {message}")]
    Processing {
        /// Error message
        message: String,
        /// Processing stage where error occurred
        stage: ProcessingStage,
        /// Additional context information
        context: std::collections::HashMap<String, String>,
    },
}

/// Compression operation type
#[derive(Debug, Clone)]
pub enum CompressionOperation {
    /// Compression operation
    Compress,
    /// Decompression operation
    Decompress,
}

/// Serialization operation type
#[derive(Debug, Clone)]
pub enum SerializationOperation {
    /// Serialization operation
    Serialize,
    /// Deserialization operation
    Deserialize,
}

/// Validation type
#[derive(Debug, Clone)]
pub enum ValidationType {
    /// File format validation
    Format,
    /// Data integrity validation
    Integrity,
    /// Schema validation
    Schema,
    /// Checksum validation
    Checksum,
    /// Size validation
    Size,
    /// Version compatibility validation
    Version,
}

/// Resource type
#[derive(Debug, Clone)]
pub enum ResourceType {
    /// Memory usage
    Memory,
    /// Disk space
    DiskSpace,
    /// Processing time
    Time,
    /// File handles
    FileHandles,
    /// Network bandwidth
    Bandwidth,
}

/// Processing stage
#[derive(Debug, Clone)]
pub enum ProcessingStage {
    /// Initialization stage
    Initialization,
    /// File reading stage
    FileReading,
    /// Data parsing stage
    DataParsing,
    /// Validation stage
    Validation,
    /// Conversion stage
    Conversion,
    /// Output generation stage
    OutputGeneration,
    /// Cleanup stage
    Cleanup,
}

impl fmt::Display for CompressionOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompressionOperation::Compress => write!(f, "compression"),
            CompressionOperation::Decompress => write!(f, "decompression"),
        }
    }
}

impl fmt::Display for SerializationOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerializationOperation::Serialize => write!(f, "serialization"),
            SerializationOperation::Deserialize => write!(f, "deserialization"),
        }
    }
}

impl fmt::Display for ValidationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationType::Format => write!(f, "format"),
            ValidationType::Integrity => write!(f, "integrity"),
            ValidationType::Schema => write!(f, "schema"),
            ValidationType::Checksum => write!(f, "checksum"),
            ValidationType::Size => write!(f, "size"),
            ValidationType::Version => write!(f, "version"),
        }
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceType::Memory => write!(f, "memory"),
            ResourceType::DiskSpace => write!(f, "disk space"),
            ResourceType::Time => write!(f, "processing time"),
            ResourceType::FileHandles => write!(f, "file handles"),
            ResourceType::Bandwidth => write!(f, "network bandwidth"),
        }
    }
}

impl fmt::Display for ProcessingStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingStage::Initialization => write!(f, "initialization"),
            ProcessingStage::FileReading => write!(f, "file reading"),
            ProcessingStage::DataParsing => write!(f, "data parsing"),
            ProcessingStage::Validation => write!(f, "validation"),
            ProcessingStage::Conversion => write!(f, "conversion"),
            ProcessingStage::OutputGeneration => write!(f, "output generation"),
            ProcessingStage::Cleanup => write!(f, "cleanup"),
        }
    }
}

/// Error recovery strategy
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation with the same parameters
    Retry {
        /// Maximum number of retry attempts
        max_attempts: u32,
        /// Delay between retry attempts
        delay: std::time::Duration,
    },
    /// Retry with modified parameters
    RetryWithFallback {
        /// Alternative parameters to try
        fallback_options: Vec<String>,
        /// Maximum number of retry attempts
        max_attempts: u32,
    },
    /// Skip the failed operation and continue
    Skip {
        /// Whether to log the skip
        log_skip: bool,
    },
    /// Use default values and continue
    UseDefaults {
        /// Default values to use
        defaults: std::collections::HashMap<String, String>,
    },
    /// Abort the entire operation
    Abort {
        /// Whether to clean up partial results
        cleanup: bool,
    },
    /// Request user intervention
    UserIntervention {
        /// Message to display to user
        message: String,
        /// Available options for user
        options: Vec<String>,
    },
}

/// Error context information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Operation being performed
    pub operation: String,
    /// File path involved (if any)
    pub file_path: Option<String>,
    /// Processing stage
    pub stage: ProcessingStage,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
    /// Timestamp when error occurred
    pub timestamp: std::time::SystemTime,
    /// Thread ID where error occurred
    pub thread_id: String,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: &str, stage: ProcessingStage) -> Self {
        Self {
            operation: operation.to_string(),
            file_path: None,
            stage,
            metadata: std::collections::HashMap::new(),
            timestamp: std::time::SystemTime::now(),
            thread_id: format!("{:?}", std::thread::current().id()),
        }
    }

    /// Add file path to context
    pub fn with_file_path(mut self, path: &str) -> Self {
        self.file_path = Some(path.to_string());
        self
    }

    /// Add metadata to context
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Enhanced error with context and recovery information
#[derive(Debug)]
pub struct EnhancedError {
    /// The underlying error
    pub error: BinaryProcessingError,
    /// Error context
    pub context: ErrorContext,
    /// Suggested recovery strategies
    pub recovery_strategies: Vec<RecoveryStrategy>,
    /// Whether this error is recoverable
    pub is_recoverable: bool,
    /// Error severity level
    pub severity: ErrorSeverity,
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational - operation completed with minor issues
    Info,
    /// Warning - operation completed but with concerns
    Warning,
    /// Error - operation failed but recovery might be possible
    Error,
    /// Critical - operation failed and recovery is unlikely
    Critical,
    /// Fatal - system is in an inconsistent state
    Fatal,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "INFO"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
            ErrorSeverity::Fatal => write!(f, "FATAL"),
        }
    }
}

impl EnhancedError {
    /// Create a new enhanced error
    pub fn new(
        error: BinaryProcessingError,
        context: ErrorContext,
        severity: ErrorSeverity,
    ) -> Self {
        let is_recoverable = matches!(severity, ErrorSeverity::Info | ErrorSeverity::Warning | ErrorSeverity::Error);
        let recovery_strategies = Self::suggest_recovery_strategies(&error, &context);

        Self {
            error,
            context,
            recovery_strategies,
            is_recoverable,
            severity,
        }
    }

    /// Suggest recovery strategies based on error type and context
    fn suggest_recovery_strategies(
        error: &BinaryProcessingError,
        context: &ErrorContext,
    ) -> Vec<RecoveryStrategy> {
        match error {
            BinaryProcessingError::FileSystem { .. } => vec![
                RecoveryStrategy::Retry {
                    max_attempts: 3,
                    delay: std::time::Duration::from_millis(100),
                },
                RecoveryStrategy::UserIntervention {
                    message: "File system error occurred. Please check file permissions and disk space.".to_string(),
                    options: vec!["Retry".to_string(), "Skip".to_string(), "Abort".to_string()],
                },
            ],
            BinaryProcessingError::Compression { .. } => vec![
                RecoveryStrategy::RetryWithFallback {
                    fallback_options: vec!["none".to_string(), "gzip".to_string()],
                    max_attempts: 2,
                },
                RecoveryStrategy::UseDefaults {
                    defaults: [("compression".to_string(), "none".to_string())].into(),
                },
            ],
            BinaryProcessingError::Serialization { .. } => vec![
                RecoveryStrategy::Skip { log_skip: true },
                RecoveryStrategy::UseDefaults {
                    defaults: [("format".to_string(), "json".to_string())].into(),
                },
            ],
            BinaryProcessingError::Validation { .. } => vec![
                RecoveryStrategy::Skip { log_skip: true },
                RecoveryStrategy::UserIntervention {
                    message: "Data validation failed. Continue with potentially corrupted data?".to_string(),
                    options: vec!["Continue".to_string(), "Abort".to_string()],
                },
            ],
            BinaryProcessingError::ResourceLimit { .. } => vec![
                RecoveryStrategy::RetryWithFallback {
                    fallback_options: vec!["streaming".to_string(), "chunked".to_string()],
                    max_attempts: 1,
                },
                RecoveryStrategy::Abort { cleanup: true },
            ],
            BinaryProcessingError::Configuration { .. } => vec![
                RecoveryStrategy::UseDefaults {
                    defaults: context.metadata.clone(),
                },
                RecoveryStrategy::UserIntervention {
                    message: "Invalid configuration detected. Please provide valid parameters.".to_string(),
                    options: vec!["Use defaults".to_string(), "Reconfigure".to_string()],
                },
            ],
            _ => vec![
                RecoveryStrategy::Retry {
                    max_attempts: 1,
                    delay: std::time::Duration::from_millis(50),
                },
                RecoveryStrategy::Abort { cleanup: true },
            ],
        }
    }

    /// Get a user-friendly error message with recovery suggestions
    pub fn user_friendly_message(&self) -> String {
        let mut message = format!("[{}] {}", self.severity, self.error);
        
        if let Some(file_path) = &self.context.file_path {
            message.push_str(&format!("\n  File: {file_path}"));
        }
        
        message.push_str(&format!("\n  Stage: {}", self.context.stage));
        message.push_str(&format!("\n  Thread: {}", self.context.thread_id));
        
        if !self.context.metadata.is_empty() {
            message.push_str("\n  Context:");
            for (key, value) in &self.context.metadata {
                message.push_str(&format!("\n    {key}: {value}"));
            }
        }
        
        if self.is_recoverable && !self.recovery_strategies.is_empty() {
            message.push_str("\n  Suggested actions:");
            for (i, strategy) in self.recovery_strategies.iter().enumerate() {
                message.push_str(&format!("\n    {}. {}", i + 1, self.strategy_description(strategy)));
            }
        }
        
        message
    }

    /// Get a description of a recovery strategy
    fn strategy_description(&self, strategy: &RecoveryStrategy) -> String {
        match strategy {
            RecoveryStrategy::Retry { max_attempts, .. } => {
                format!("Retry operation (up to {max_attempts} attempts)")
            }
            RecoveryStrategy::RetryWithFallback { fallback_options, .. } => {
                format!("Try alternative options: {}", fallback_options.join(", "))
            }
            RecoveryStrategy::Skip { .. } => {
                "Skip this operation and continue".to_string()
            }
            RecoveryStrategy::UseDefaults { .. } => {
                "Use default values and continue".to_string()
            }
            RecoveryStrategy::Abort { .. } => {
                "Abort the operation".to_string()
            }
            RecoveryStrategy::UserIntervention { message, .. } => {
                format!("User intervention required: {message}")
            }
        }
    }
}

/// Structured logger for binary processing operations
pub struct BinaryLogger {
    /// Log level filter
    level: LogLevel,
    /// Whether to include timestamps
    include_timestamps: bool,
    /// Whether to include thread IDs
    include_thread_ids: bool,
    /// Output destination
    output: LogOutput,
}

/// Log levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Trace level - very detailed debugging information
    Trace,
    /// Debug level - debugging information
    Debug,
    /// Info level - general information
    Info,
    /// Warning level - warning messages
    Warning,
    /// Error level - error messages
    Error,
    /// Critical level - critical error messages
    Critical,
}

/// Log output destination
#[derive(Debug, Clone)]
pub enum LogOutput {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
    /// File output
    File(String),
    /// Multiple outputs
    Multiple(Vec<LogOutput>),
}

impl PartialEq for LogOutput {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LogOutput::Stdout, LogOutput::Stdout) => true,
            (LogOutput::Stderr, LogOutput::Stderr) => true,
            (LogOutput::File(a), LogOutput::File(b)) => a == b,
            (LogOutput::Multiple(a), LogOutput::Multiple(b)) => a == b,
            _ => false,
        }
    }
}

impl BinaryLogger {
    /// Create a new logger with default settings
    pub fn new() -> Self {
        Self {
            level: LogLevel::Info,
            include_timestamps: true,
            include_thread_ids: false,
            output: LogOutput::Stdout,
        }
    }

    /// Set log level
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// Enable/disable timestamps
    pub fn with_timestamps(mut self, include: bool) -> Self {
        self.include_timestamps = include;
        self
    }

    /// Enable/disable thread IDs
    pub fn with_thread_ids(mut self, include: bool) -> Self {
        self.include_thread_ids = include;
        self
    }

    /// Set output destination
    pub fn with_output(mut self, output: LogOutput) -> Self {
        self.output = output;
        self
    }

    /// Log a message at the specified level
    pub fn log(&self, level: LogLevel, message: &str) {
        if level >= self.level {
            let formatted_message = self.format_message(level, message);
            self.write_message(&formatted_message);
        }
    }

    /// Log an enhanced error
    pub fn log_error(&self, error: &EnhancedError) {
        let level = match error.severity {
            ErrorSeverity::Info => LogLevel::Info,
            ErrorSeverity::Warning => LogLevel::Warning,
            ErrorSeverity::Error => LogLevel::Error,
            ErrorSeverity::Critical => LogLevel::Critical,
            ErrorSeverity::Fatal => LogLevel::Critical,
        };
        
        self.log(level, &error.user_friendly_message());
    }

    /// Log trace message
    pub fn trace(&self, message: &str) {
        self.log(LogLevel::Trace, message);
    }

    /// Log debug message
    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    /// Log info message
    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    /// Log warning message
    pub fn warning(&self, message: &str) {
        self.log(LogLevel::Warning, message);
    }

    /// Log error message
    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }

    /// Log critical message
    pub fn critical(&self, message: &str) {
        self.log(LogLevel::Critical, message);
    }

    /// Format a log message
    fn format_message(&self, level: LogLevel, message: &str) -> String {
        let mut formatted = String::new();

        if self.include_timestamps {
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
            formatted.push_str(&format!("[{timestamp}] "));
        }

        formatted.push_str(&format!("[{level:?}] "));

        if self.include_thread_ids {
            let thread_id = format!("{:?}", std::thread::current().id());
            formatted.push_str(&format!("[{thread_id}] "));
        }

        formatted.push_str(message);
        formatted
    }

    /// Write message to output destination
    fn write_message(&self, message: &str) {
        match &self.output {
            LogOutput::Stdout => println!("{message}"),
            LogOutput::Stderr => eprintln!("{message}"),
            LogOutput::File(path) => {
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                {
                    use std::io::Write;
                    let _ = writeln!(file, "{message}");
                }
            }
            LogOutput::Multiple(outputs) => {
                for output in outputs {
                    let temp_logger = BinaryLogger {
                        level: self.level.clone(),
                        include_timestamps: self.include_timestamps,
                        include_thread_ids: self.include_thread_ids,
                        output: output.clone(),
                    };
                    temp_logger.write_message(message);
                }
            }
        }
    }
}

impl Default for BinaryLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Global logger instance
static mut GLOBAL_LOGGER: Option<BinaryLogger> = None;
static LOGGER_INIT: std::sync::Once = std::sync::Once::new();
static DEFAULT_LOGGER: BinaryLogger = BinaryLogger {
    level: LogLevel::Info,
    include_timestamps: true,
    include_thread_ids: false,
    output: LogOutput::Stdout,
};

/// Initialize the global logger
pub fn init_logger(logger: BinaryLogger) {
    LOGGER_INIT.call_once(|| {
        unsafe {
            GLOBAL_LOGGER = Some(logger);
        }
    });
}

/// Get the global logger instance
pub fn get_logger() -> &'static BinaryLogger {
    unsafe {
        GLOBAL_LOGGER.as_ref().unwrap_or(&DEFAULT_LOGGER)
    }
}

/// Convenience macro for logging
#[macro_export]
macro_rules! log_binary {
    ($level:expr, $($arg:tt)*) => {
        $crate::export::formats::binary_errors::get_logger().log($level, &format!($($arg)*))
    };
}

/// Convenience macros for different log levels
#[macro_export]
macro_rules! trace_binary {
    ($($arg:tt)*) => {
        $crate::export::formats::binary_errors::get_logger().trace(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! debug_binary {
    ($($arg:tt)*) => {
        $crate::export::formats::binary_errors::get_logger().debug(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! info_binary {
    ($($arg:tt)*) => {
        $crate::export::formats::binary_errors::get_logger().info(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn_binary {
    ($($arg:tt)*) => {
        $crate::export::formats::binary_errors::get_logger().warning(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! error_binary {
    ($($arg:tt)*) => {
        $crate::export::formats::binary_errors::get_logger().error(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! critical_binary {
    ($($arg:tt)*) => {
        $crate::export::formats::binary_errors::get_logger().critical(&format!($($arg)*))
    };
}