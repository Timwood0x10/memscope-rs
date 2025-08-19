//! JSON file discovery module for MemScope HTML generation
//!
//! This module provides functionality to discover and validate JSON files
//! in the MemoryAnalysis directory structure for HTML report generation.

use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration for a specific type of JSON file
#[derive(Debug, Clone)]
pub struct JsonFileConfig {
    /// File suffix used to identify the file type (e.g., "memory_analysis")
    pub suffix: &'static str,
    /// Human-readable description of the file type
    pub description: &'static str,
    /// Whether this file type is required for HTML generation
    pub required: bool,
    /// Maximum allowed file size in megabytes
    pub max_size_mb: Option<usize>,
}

impl JsonFileConfig {
    /// Create a new JSON file configuration
    pub fn new(suffix: &'static str, description: &'static str) -> Self {
        Self {
            suffix,
            description,
            required: false,
            max_size_mb: Some(100), // Default 100MB limit
        }
    }

    /// Mark this file type as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set maximum file size limit in megabytes
    pub fn max_size_mb(mut self, size: usize) -> Self {
        self.max_size_mb = Some(size);
        self
    }
}

/// Information about a discovered JSON file
#[derive(Debug, Clone)]
pub struct JsonFileInfo {
    /// Full path to the JSON file
    pub path: PathBuf,
    /// File type configuration
    pub config: JsonFileConfig,
    /// File size in bytes
    pub size_bytes: u64,
    /// Whether the file is readable
    pub is_readable: bool,
}

/// Result of JSON file discovery process
#[derive(Debug)]
pub struct JsonDiscoveryResult {
    /// Successfully discovered files
    pub found_files: Vec<JsonFileInfo>,
    /// Missing required files
    pub missing_required: Vec<JsonFileConfig>,
    /// Files that exceed size limits
    pub oversized_files: Vec<JsonFileInfo>,
    /// Files that are not readable
    pub unreadable_files: Vec<JsonFileInfo>,
    /// Total size of all discovered files in bytes
    pub total_size_bytes: u64,
}

/// Errors that can occur during JSON file discovery
#[derive(Debug)]
pub enum JsonDiscoveryError {
    /// Directory does not exist or is not accessible
    DirectoryNotFound(String),
    /// Required JSON files are missing
    MissingRequiredFiles(Vec<String>),
    /// Files exceed maximum size limits
    FilesTooLarge(Vec<String>),
    /// IO error during file discovery
    IoError(std::io::Error),
}

impl fmt::Display for JsonDiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonDiscoveryError::DirectoryNotFound(dir) => {
                write!(f, "Directory not found or not accessible: {}", dir)
            }
            JsonDiscoveryError::MissingRequiredFiles(files) => {
                write!(f, "Missing required JSON files: {}", files.join(", "))
            }
            JsonDiscoveryError::FilesTooLarge(files) => {
                write!(f, "Files exceed size limit: {}", files.join(", "))
            }
            JsonDiscoveryError::IoError(err) => {
                write!(f, "IO error during file discovery: {}", err)
            }
        }
    }
}

impl Error for JsonDiscoveryError {}

/// JSON file discovery service
pub struct JsonFileDiscovery {
    /// Base directory to search for JSON files
    input_dir: String,
    /// Base name pattern for JSON files
    base_name: String,
}

impl JsonFileDiscovery {
    /// Create a new JSON file discovery instance
    pub fn new(input_dir: String, base_name: String) -> Self {
        Self {
            input_dir,
            base_name,
        }
    }

    /// Get the default file configurations for MemScope analysis
    pub fn get_default_file_configs() -> Vec<JsonFileConfig> {
        vec![
            JsonFileConfig::new("memory_analysis", "Memory Analysis").required(),
            JsonFileConfig::new("lifetime", "Lifecycle Analysis"),
            JsonFileConfig::new("unsafe_ffi", "Unsafe/FFI Analysis"),
            JsonFileConfig::new("performance", "Performance Metrics"),
            JsonFileConfig::new("complex_types", "Complex Types Analysis"),
            JsonFileConfig::new("security_violations", "Security Violations"),
            JsonFileConfig::new("variable_relationships", "Variable Relationships"),
        ]
    }

    /// Discover JSON files in the input directory
    pub fn discover_files(&self) -> Result<JsonDiscoveryResult, JsonDiscoveryError> {
        // Check if input directory exists
        let input_path = Path::new(&self.input_dir);
        if !input_path.exists() || !input_path.is_dir() {
            return Err(JsonDiscoveryError::DirectoryNotFound(
                self.input_dir.clone(),
            ));
        }

        let file_configs = Self::get_default_file_configs();
        let mut found_files = Vec::new();
        let mut missing_required = Vec::new();
        let mut oversized_files = Vec::new();
        let mut unreadable_files = Vec::new();
        let mut total_size_bytes = 0u64;

        tracing::info!("🔍 Discovering JSON files in directory: {}", self.input_dir);
        tracing::info!("🏷️  Using base name pattern: {}", self.base_name);

        for config in file_configs {
            match self.find_file_for_config(&config) {
                Ok(Some(file_info)) => {
                    // Check file size limits
                    if let Some(max_size_mb) = config.max_size_mb {
                        let max_bytes = (max_size_mb * 1024 * 1024) as u64;
                        if file_info.size_bytes > max_bytes {
                            tracing::info!(
                                "⚠️  File {} ({:.1} MB) exceeds size limit ({} MB)",
                                file_info.path.display(),
                                file_info.size_bytes as f64 / 1024.0 / 1024.0,
                                max_size_mb
                            );
                            oversized_files.push(file_info);
                            continue;
                        }
                    }

                    // Check readability
                    if !file_info.is_readable {
                        tracing::info!("⚠️  File {} is not readable", file_info.path.display());
                        unreadable_files.push(file_info);
                        continue;
                    }

                    total_size_bytes += file_info.size_bytes;
                    tracing::info!(
                        "✅ Found {}: {} ({:.1} KB)",
                        config.description,
                        file_info.path.display(),
                        file_info.size_bytes as f64 / 1024.0
                    );
                    found_files.push(file_info);
                }
                Ok(None) => {
                    if config.required {
                        tracing::info!(
                            "❌ Required file not found: {}_{}*.json",
                            self.base_name,
                            config.suffix
                        );
                        missing_required.push(config);
                    } else {
                        tracing::info!(
                            "⚠️  Optional file not found: {}_{}*.json (skipping)",
                            self.base_name,
                            config.suffix
                        );
                    }
                }
                Err(e) => {
                    tracing::info!("❌ Error searching for {}: {}", config.description, e);
                    if config.required {
                        missing_required.push(config);
                    }
                }
            }
        }

        // Print discovery summary
        tracing::info!("📊 Discovery Summary:");
        tracing::info!("   Files found: {}", found_files.len());
        tracing::info!(
            "   Total size: {:.1} MB",
            total_size_bytes as f64 / 1024.0 / 1024.0
        );
        tracing::info!("   Missing required: {}", missing_required.len());
        tracing::info!("   Oversized files: {}", oversized_files.len());
        tracing::info!("   Unreadable files: {}", unreadable_files.len());

        // Check for critical errors
        if !missing_required.is_empty() {
            let missing_names: Vec<String> = missing_required
                .iter()
                .map(|config| format!("{}_{}", self.base_name, config.suffix))
                .collect();
            return Err(JsonDiscoveryError::MissingRequiredFiles(missing_names));
        }

        if !oversized_files.is_empty() {
            let oversized_names: Vec<String> = oversized_files
                .iter()
                .map(|file| file.path.to_string_lossy().to_string())
                .collect();
            return Err(JsonDiscoveryError::FilesTooLarge(oversized_names));
        }

        Ok(JsonDiscoveryResult {
            found_files,
            missing_required,
            oversized_files,
            unreadable_files,
            total_size_bytes,
        })
    }

    /// Find a JSON file for a specific configuration
    fn find_file_for_config(
        &self,
        config: &JsonFileConfig,
    ) -> Result<Option<JsonFileInfo>, std::io::Error> {
        // Try exact match first
        let exact_path = format!(
            "{}/{}_{}.json",
            self.input_dir, self.base_name, config.suffix
        );
        if let Ok(file_info) = self.create_file_info(&exact_path, config) {
            return Ok(Some(file_info));
        }

        // If exact match fails, search for files containing the suffix
        let entries = fs::read_dir(&self.input_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.contains(config.suffix) && file_name.ends_with(".json") {
                    if let Ok(file_info) = self.create_file_info(&path.to_string_lossy(), config) {
                        return Ok(Some(file_info));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Create file info for a discovered file
    fn create_file_info(
        &self,
        file_path: &str,
        config: &JsonFileConfig,
    ) -> Result<JsonFileInfo, std::io::Error> {
        let path = PathBuf::from(file_path);
        let metadata = fs::metadata(&path)?;
        let size_bytes = metadata.len();

        // Check if file is readable by attempting to open it
        let is_readable = fs::File::open(&path).is_ok();

        Ok(JsonFileInfo {
            path,
            config: config.clone(),
            size_bytes,
            is_readable,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_json_file_config_creation() {
        let config = JsonFileConfig::new("test", "Test File");
        assert_eq!(config.suffix, "test");
        assert_eq!(config.description, "Test File");
        assert!(!config.required);
        assert_eq!(config.max_size_mb, Some(100));
    }

    #[test]
    fn test_json_file_config_required() {
        let config = JsonFileConfig::new("test", "Test File").required();
        assert!(config.required);
    }

    #[test]
    fn test_json_file_config_max_size() {
        let config = JsonFileConfig::new("test", "Test File").max_size_mb(50);
        assert_eq!(config.max_size_mb, Some(50));
    }

    #[test]
    fn test_directory_not_found() {
        let discovery = JsonFileDiscovery::new("/nonexistent/path".to_string(), "test".to_string());
        let result = discovery.discover_files();
        assert!(matches!(
            result,
            Err(JsonDiscoveryError::DirectoryNotFound(_))
        ));
    }

    #[test]
    fn test_discover_files_with_temp_dir() {
        let temp_dir = TempDir::new().expect("Failed to get test value");
        let temp_path = temp_dir
            .path()
            .to_str()
            .expect("Failed to convert path to string");

        // Create a test JSON file
        let test_file_path = format!("{}/test_memory_analysis.json", temp_path);
        fs::write(&test_file_path, r#"{"test": "data"}"#).expect("Failed to write test file");

        let discovery = JsonFileDiscovery::new(temp_path.to_string(), "test".to_string());
        let result = discovery
            .discover_files()
            .expect("Failed to discover files");

        assert!(!result.found_files.is_empty());
        assert_eq!(result.found_files[0].config.suffix, "memory_analysis");
    }
}
