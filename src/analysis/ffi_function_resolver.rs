//! FFI Function Name Resolution System
//!
//! This module provides enhanced FFI function name resolution to replace
//! vague "potential_ffi_target" with specific function and library information.

use crate::core::types::{TrackingResult, TrackingError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Resolved FFI function information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedFfiFunction {
    /// Library name (e.g., "libc", "libssl", "user_library")
    pub library_name: String,
    /// Function name (e.g., "malloc", "free", "SSL_new")
    pub function_name: String,
    /// Function signature if available
    pub signature: Option<String>,
    /// Function category
    pub category: FfiFunctionCategory,
    /// Risk level associated with this function
    pub risk_level: FfiRiskLevel,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Categories of FFI functions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FfiFunctionCategory {
    /// Memory management functions (malloc, free, realloc)
    MemoryManagement,
    /// String manipulation functions (strcpy, strcat, sprintf)
    StringManipulation,
    /// File I/O functions (fopen, fread, fwrite)
    FileIO,
    /// Network functions (socket, connect, send)
    Network,
    /// Cryptographic functions (SSL_*, crypto_*)
    Cryptographic,
    /// System calls (fork, exec, signal)
    SystemCall,
    /// Graphics/UI functions (OpenGL, DirectX, etc.)
    Graphics,
    /// Database functions
    Database,
    /// Custom user library functions
    UserLibrary,
    /// Unknown or unclassified functions
    Unknown,
}

/// Risk levels for FFI functions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum FfiRiskLevel {
    /// Very low risk, well-tested standard functions
    VeryLow,
    /// Low risk, standard library functions with good safety record
    Low,
    /// Medium risk, functions that require careful parameter validation
    Medium,
    /// High risk, functions known to be dangerous if misused
    High,
    /// Critical risk, functions that are inherently unsafe
    Critical,
}

/// FFI function resolver with built-in knowledge base
pub struct FfiFunctionResolver {
    /// Known function database
    function_database: Arc<Mutex<HashMap<String, ResolvedFfiFunction>>>,
    /// Library mapping (function -> library)
    library_mapping: Arc<Mutex<HashMap<String, String>>>,
    /// Resolution statistics
    stats: Arc<Mutex<ResolutionStats>>,
    /// Configuration
    config: ResolverConfig,
}

/// Configuration for FFI function resolver
#[derive(Debug, Clone)]
pub struct ResolverConfig {
    /// Enable automatic function discovery
    pub enable_auto_discovery: bool,
    /// Enable risk assessment
    pub enable_risk_assessment: bool,
    /// Cache resolved functions
    pub enable_caching: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            enable_auto_discovery: true,
            enable_risk_assessment: true,
            enable_caching: true,
            max_cache_size: 1000,
        }
    }
}

/// Statistics for function resolution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolutionStats {
    /// Total resolution attempts
    pub total_attempts: usize,
    /// Successful resolutions
    pub successful_resolutions: usize,
    /// Failed resolutions
    pub failed_resolutions: usize,
    /// Cache hits
    pub cache_hits: usize,
    /// Functions by category
    pub functions_by_category: HashMap<String, usize>,
    /// Functions by risk level
    pub functions_by_risk: HashMap<String, usize>,
    /// Most frequently resolved functions
    pub top_functions: Vec<(String, usize)>,
}

impl FfiFunctionResolver {
    /// Create new FFI function resolver
    pub fn new(config: ResolverConfig) -> Self {
        tracing::info!("🔍 Initializing FFI Function Resolver");
        tracing::info!("   • Auto discovery: {}", config.enable_auto_discovery);
        tracing::info!("   • Risk assessment: {}", config.enable_risk_assessment);
        tracing::info!("   • Caching: {}", config.enable_caching);

        let resolver = Self {
            function_database: Arc::new(Mutex::new(HashMap::new())),
            library_mapping: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(ResolutionStats::default())),
            config,
        };

        // Initialize with known functions
        resolver.initialize_known_functions();
        resolver
    }

    /// Resolve FFI function name and library
    pub fn resolve_function(&self, function_name: &str, library_hint: Option<&str>) -> TrackingResult<ResolvedFfiFunction> {
        self.update_stats_attempt();

        // Check cache first
        if self.config.enable_caching {
            if let Ok(db) = self.function_database.lock() {
                if let Some(cached) = db.get(function_name) {
                    self.update_stats_cache_hit();
                    self.update_stats_success();
                    tracing::debug!("🔍 Cache hit for function: {}", function_name);
                    return Ok(cached.clone());
                }
            }
        }

        // Try to resolve from known functions
        let resolved = if let Some(known) = self.get_known_function(function_name) {
            known
        } else if let Some(lib_hint) = library_hint {
            // Use library hint to create resolution
            self.resolve_with_library_hint(function_name, lib_hint)?
        } else if self.config.enable_auto_discovery {
            // Try automatic discovery
            self.auto_discover_function(function_name)?
        } else {
            // Create unknown function entry
            self.create_unknown_function(function_name)
        };

        // Cache the result
        if self.config.enable_caching {
            self.cache_function(function_name, &resolved)?;
        }

        self.update_stats_success();
        tracing::debug!("🔍 Resolved function: {} -> {}::{}", 
            function_name, resolved.library_name, resolved.function_name);

        Ok(resolved)
    }

    /// Resolve multiple functions in batch
    pub fn resolve_functions_batch(&self, function_names: &[String]) -> Vec<TrackingResult<ResolvedFfiFunction>> {
        function_names.iter()
            .map(|name| self.resolve_function(name, None))
            .collect()
    }

    /// Add custom function to database
    pub fn add_custom_function(&self, function_name: String, resolved: ResolvedFfiFunction) -> TrackingResult<()> {
        if let Ok(mut db) = self.function_database.lock() {
            db.insert(function_name.clone(), resolved.clone());
            
            if let Ok(mut mapping) = self.library_mapping.lock() {
                mapping.insert(function_name.clone(), resolved.library_name.clone());
            }
            
            tracing::info!("📚 Added custom function: {} -> {}::{}", 
                function_name, resolved.library_name, resolved.function_name);
            Ok(())
        } else {
            Err(TrackingError::LockContention("Failed to lock function database".to_string()))
        }
    }

    /// Get resolution statistics
    pub fn get_stats(&self) -> ResolutionStats {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            tracing::error!("Failed to lock resolution stats");
            ResolutionStats::default()
        }
    }

    /// Clear function cache
    pub fn clear_cache(&self) {
        if let Ok(mut db) = self.function_database.lock() {
            let initial_size = db.len();
            db.retain(|_, func| self.is_builtin_function(func));
            let cleared = initial_size - db.len();
            tracing::info!("🧹 Cleared {} cached functions", cleared);
        }
    }

    // Private helper methods

    fn initialize_known_functions(&self) {
        let known_functions = vec![
            // Memory management functions
            ("malloc", ResolvedFfiFunction {
                library_name: "libc".to_string(),
                function_name: "malloc".to_string(),
                signature: Some("void* malloc(size_t size)".to_string()),
                category: FfiFunctionCategory::MemoryManagement,
                risk_level: FfiRiskLevel::Medium,
                metadata: [("description".to_string(), "Allocate memory".to_string())].into(),
            }),
            ("free", ResolvedFfiFunction {
                library_name: "libc".to_string(),
                function_name: "free".to_string(),
                signature: Some("void free(void* ptr)".to_string()),
                category: FfiFunctionCategory::MemoryManagement,
                risk_level: FfiRiskLevel::High,
                metadata: [("description".to_string(), "Free allocated memory".to_string())].into(),
            }),
            ("realloc", ResolvedFfiFunction {
                library_name: "libc".to_string(),
                function_name: "realloc".to_string(),
                signature: Some("void* realloc(void* ptr, size_t size)".to_string()),
                category: FfiFunctionCategory::MemoryManagement,
                risk_level: FfiRiskLevel::High,
                metadata: [("description".to_string(), "Reallocate memory".to_string())].into(),
            }),
            ("calloc", ResolvedFfiFunction {
                library_name: "libc".to_string(),
                function_name: "calloc".to_string(),
                signature: Some("void* calloc(size_t num, size_t size)".to_string()),
                category: FfiFunctionCategory::MemoryManagement,
                risk_level: FfiRiskLevel::Medium,
                metadata: [("description".to_string(), "Allocate and zero memory".to_string())].into(),
            }),

            // String manipulation functions
            ("strcpy", ResolvedFfiFunction {
                library_name: "libc".to_string(),
                function_name: "strcpy".to_string(),
                signature: Some("char* strcpy(char* dest, const char* src)".to_string()),
                category: FfiFunctionCategory::StringManipulation,
                risk_level: FfiRiskLevel::Critical,
                metadata: [("description".to_string(), "Copy string (unsafe)".to_string())].into(),
            }),
            ("strncpy", ResolvedFfiFunction {
                library_name: "libc".to_string(),
                function_name: "strncpy".to_string(),
                signature: Some("char* strncpy(char* dest, const char* src, size_t n)".to_string()),
                category: FfiFunctionCategory::StringManipulation,
                risk_level: FfiRiskLevel::Medium,
                metadata: [("description".to_string(), "Copy string with length limit".to_string())].into(),
            }),
            ("sprintf", ResolvedFfiFunction {
                library_name: "libc".to_string(),
                function_name: "sprintf".to_string(),
                signature: Some("int sprintf(char* str, const char* format, ...)".to_string()),
                category: FfiFunctionCategory::StringManipulation,
                risk_level: FfiRiskLevel::Critical,
                metadata: [("description".to_string(), "Format string (unsafe)".to_string())].into(),
            }),
            ("snprintf", ResolvedFfiFunction {
                library_name: "libc".to_string(),
                function_name: "snprintf".to_string(),
                signature: Some("int snprintf(char* str, size_t size, const char* format, ...)".to_string()),
                category: FfiFunctionCategory::StringManipulation,
                risk_level: FfiRiskLevel::Low,
                metadata: [("description".to_string(), "Safe format string".to_string())].into(),
            }),

            // File I/O functions
            ("fopen", ResolvedFfiFunction {
                library_name: "libc".to_string(),
                function_name: "fopen".to_string(),
                signature: Some("FILE* fopen(const char* filename, const char* mode)".to_string()),
                category: FfiFunctionCategory::FileIO,
                risk_level: FfiRiskLevel::Low,
                metadata: [("description".to_string(), "Open file".to_string())].into(),
            }),
            ("fclose", ResolvedFfiFunction {
                library_name: "libc".to_string(),
                function_name: "fclose".to_string(),
                signature: Some("int fclose(FILE* stream)".to_string()),
                category: FfiFunctionCategory::FileIO,
                risk_level: FfiRiskLevel::Low,
                metadata: [("description".to_string(), "Close file".to_string())].into(),
            }),

            // System calls
            ("fork", ResolvedFfiFunction {
                library_name: "libc".to_string(),
                function_name: "fork".to_string(),
                signature: Some("pid_t fork(void)".to_string()),
                category: FfiFunctionCategory::SystemCall,
                risk_level: FfiRiskLevel::Medium,
                metadata: [("description".to_string(), "Create child process".to_string())].into(),
            }),
        ];

        if let Ok(mut db) = self.function_database.lock() {
            if let Ok(mut mapping) = self.library_mapping.lock() {
                for (name, func) in known_functions {
                    db.insert(name.to_string(), func.clone());
                    mapping.insert(name.to_string(), func.library_name);
                }
            }
        }

        tracing::info!("📚 Initialized {} known FFI functions", 11);
    }

    fn get_known_function(&self, function_name: &str) -> Option<ResolvedFfiFunction> {
        if let Ok(db) = self.function_database.lock() {
            db.get(function_name).cloned()
        } else {
            None
        }
    }

    fn resolve_with_library_hint(&self, function_name: &str, library_hint: &str) -> TrackingResult<ResolvedFfiFunction> {
        let category = self.infer_category_from_name(function_name);
        let risk_level = self.assess_risk_from_name(function_name, &category);

        Ok(ResolvedFfiFunction {
            library_name: library_hint.to_string(),
            function_name: function_name.to_string(),
            signature: None,
            category,
            risk_level,
            metadata: HashMap::new(),
        })
    }

    fn auto_discover_function(&self, function_name: &str) -> TrackingResult<ResolvedFfiFunction> {
        // Try to infer library from function name patterns
        let library_name = self.infer_library_from_name(function_name);
        let category = self.infer_category_from_name(function_name);
        let risk_level = self.assess_risk_from_name(function_name, &category);

        Ok(ResolvedFfiFunction {
            library_name,
            function_name: function_name.to_string(),
            signature: None,
            category,
            risk_level,
            metadata: [("auto_discovered".to_string(), "true".to_string())].into(),
        })
    }

    fn create_unknown_function(&self, function_name: &str) -> ResolvedFfiFunction {
        ResolvedFfiFunction {
            library_name: "unknown".to_string(),
            function_name: function_name.to_string(),
            signature: None,
            category: FfiFunctionCategory::Unknown,
            risk_level: FfiRiskLevel::Medium,
            metadata: [("status".to_string(), "unresolved".to_string())].into(),
        }
    }

    fn infer_library_from_name(&self, function_name: &str) -> String {
        // Common library patterns
        if function_name.starts_with("SSL_") || function_name.starts_with("crypto_") {
            "libssl".to_string()
        } else if function_name.starts_with("pthread_") {
            "libpthread".to_string()
        } else if function_name.starts_with("gl") || function_name.starts_with("GL") {
            "libGL".to_string()
        } else if function_name.starts_with("sqlite3_") {
            "libsqlite3".to_string()
        } else if ["malloc", "free", "printf", "scanf", "fopen", "fork"].iter().any(|&f| function_name.contains(f)) {
            "libc".to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn infer_category_from_name(&self, function_name: &str) -> FfiFunctionCategory {
        if ["malloc", "free", "realloc", "calloc"].iter().any(|&f| function_name.contains(f)) {
            FfiFunctionCategory::MemoryManagement
        } else if ["str", "sprintf", "printf"].iter().any(|&f| function_name.contains(f)) {
            FfiFunctionCategory::StringManipulation
        } else if ["fopen", "fread", "fwrite", "fclose"].iter().any(|&f| function_name.contains(f)) {
            FfiFunctionCategory::FileIO
        } else if ["socket", "connect", "send", "recv"].iter().any(|&f| function_name.contains(f)) {
            FfiFunctionCategory::Network
        } else if ["SSL_", "crypto_"].iter().any(|&f| function_name.starts_with(f)) {
            FfiFunctionCategory::Cryptographic
        } else if ["fork", "exec", "signal", "kill"].iter().any(|&f| function_name.contains(f)) {
            FfiFunctionCategory::SystemCall
        } else if ["gl", "GL", "Direct"].iter().any(|&f| function_name.contains(f)) {
            FfiFunctionCategory::Graphics
        } else if ["sqlite", "mysql", "postgres"].iter().any(|&f| function_name.contains(f)) {
            FfiFunctionCategory::Database
        } else {
            FfiFunctionCategory::Unknown
        }
    }

    fn assess_risk_from_name(&self, function_name: &str, category: &FfiFunctionCategory) -> FfiRiskLevel {
        // High-risk functions
        if ["strcpy", "sprintf", "gets", "scanf"].iter().any(|&f| function_name == f) {
            return FfiRiskLevel::Critical;
        }

        // Medium-high risk functions
        if ["free", "realloc"].iter().any(|&f| function_name == f) {
            return FfiRiskLevel::High;
        }

        // Category-based risk assessment
        match category {
            FfiFunctionCategory::MemoryManagement => FfiRiskLevel::Medium,
            FfiFunctionCategory::StringManipulation => FfiRiskLevel::High,
            FfiFunctionCategory::SystemCall => FfiRiskLevel::Medium,
            FfiFunctionCategory::Cryptographic => FfiRiskLevel::Low,
            FfiFunctionCategory::FileIO => FfiRiskLevel::Low,
            FfiFunctionCategory::Network => FfiRiskLevel::Medium,
            FfiFunctionCategory::Graphics => FfiRiskLevel::Low,
            FfiFunctionCategory::Database => FfiRiskLevel::Low,
            FfiFunctionCategory::UserLibrary => FfiRiskLevel::Medium,
            FfiFunctionCategory::Unknown => FfiRiskLevel::Medium,
        }
    }

    fn cache_function(&self, function_name: &str, resolved: &ResolvedFfiFunction) -> TrackingResult<()> {
        if let Ok(mut db) = self.function_database.lock() {
            // Check cache size limit
            if db.len() >= self.config.max_cache_size {
                // Remove some non-builtin entries
                let keys_to_remove: Vec<String> = db.iter()
                    .filter(|(_, func)| !self.is_builtin_function(func))
                    .take(10)
                    .map(|(k, _)| k.clone())
                    .collect();
                
                for key in keys_to_remove {
                    db.remove(&key);
                }
            }

            db.insert(function_name.to_string(), resolved.clone());
            Ok(())
        } else {
            Err(TrackingError::LockContention("Failed to lock function database".to_string()))
        }
    }

    fn is_builtin_function(&self, func: &ResolvedFfiFunction) -> bool {
        !func.metadata.contains_key("auto_discovered") && func.library_name != "unknown"
    }

    fn update_stats_attempt(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_attempts += 1;
        }
    }

    fn update_stats_success(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.successful_resolutions += 1;
        }
    }

    fn update_stats_cache_hit(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.cache_hits += 1;
        }
    }
}

impl Default for FfiFunctionResolver {
    fn default() -> Self {
        Self::new(ResolverConfig::default())
    }
}

/// Global FFI function resolver instance
static GLOBAL_FFI_RESOLVER: std::sync::OnceLock<Arc<FfiFunctionResolver>> = std::sync::OnceLock::new();

/// Get global FFI function resolver instance
pub fn get_global_ffi_resolver() -> Arc<FfiFunctionResolver> {
    GLOBAL_FFI_RESOLVER.get_or_init(|| {
        Arc::new(FfiFunctionResolver::new(ResolverConfig::default()))
    }).clone()
}

/// Initialize global FFI function resolver with custom config
pub fn initialize_global_ffi_resolver(config: ResolverConfig) -> Arc<FfiFunctionResolver> {
    let resolver = Arc::new(FfiFunctionResolver::new(config));
    if GLOBAL_FFI_RESOLVER.set(resolver.clone()).is_err() {
        tracing::warn!("Global FFI resolver already initialized");
    }
    resolver
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_function_resolution() {
        let resolver = FfiFunctionResolver::new(ResolverConfig::default());
        
        let malloc_result = resolver.resolve_function("malloc", None).unwrap();
        assert_eq!(malloc_result.library_name, "libc");
        assert_eq!(malloc_result.function_name, "malloc");
        assert_eq!(malloc_result.category, FfiFunctionCategory::MemoryManagement);
        assert_eq!(malloc_result.risk_level, FfiRiskLevel::Medium);
    }

    #[test]
    fn test_auto_discovery() {
        let resolver = FfiFunctionResolver::new(ResolverConfig::default());
        
        let ssl_result = resolver.resolve_function("SSL_new", None).unwrap();
        assert_eq!(ssl_result.library_name, "libssl");
        assert_eq!(ssl_result.category, FfiFunctionCategory::Cryptographic);
    }

    #[test]
    fn test_library_hint() {
        let resolver = FfiFunctionResolver::new(ResolverConfig::default());
        
        let custom_result = resolver.resolve_function("custom_func", Some("mylib")).unwrap();
        assert_eq!(custom_result.library_name, "mylib");
        assert_eq!(custom_result.function_name, "custom_func");
    }

    #[test]
    fn test_risk_assessment() {
        let resolver = FfiFunctionResolver::new(ResolverConfig::default());
        
        let strcpy_result = resolver.resolve_function("strcpy", None).unwrap();
        assert_eq!(strcpy_result.risk_level, FfiRiskLevel::Critical);
        
        let snprintf_result = resolver.resolve_function("snprintf", None).unwrap();
        assert_eq!(snprintf_result.risk_level, FfiRiskLevel::Low);
    }

    #[test]
    fn test_batch_resolution() {
        let resolver = FfiFunctionResolver::new(ResolverConfig::default());
        
        let functions = vec!["malloc".to_string(), "free".to_string(), "strcpy".to_string()];
        let results = resolver.resolve_functions_batch(&functions);
        
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_custom_function() {
        let resolver = FfiFunctionResolver::new(ResolverConfig::default());
        
        let custom_func = ResolvedFfiFunction {
            library_name: "mylib".to_string(),
            function_name: "my_func".to_string(),
            signature: Some("int my_func(void)".to_string()),
            category: FfiFunctionCategory::UserLibrary,
            risk_level: FfiRiskLevel::Low,
            metadata: HashMap::new(),
        };
        
        resolver.add_custom_function("my_func".to_string(), custom_func).unwrap();
        
        let resolved = resolver.resolve_function("my_func", None).unwrap();
        assert_eq!(resolved.library_name, "mylib");
        assert_eq!(resolved.category, FfiFunctionCategory::UserLibrary);
    }
}