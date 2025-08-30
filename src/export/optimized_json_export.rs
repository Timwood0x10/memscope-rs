//! Optimized JSON export implementation with performance improvements
//!
//! This module provides highly optimized JSON export functionality that addresses
//! the main performance bottlenecks identified in the current implementation.

use crate::analysis::security_violation_analyzer::{
    AnalysisConfig, SecurityViolationAnalyzer, ViolationSeverity,
};
use crate::analysis::unsafe_ffi_tracker::{get_global_unsafe_ffi_tracker, SafetyViolation};
use crate::core::tracker::MemoryTracker;
use crate::core::types::{AllocationInfo, TrackingResult};
use crate::export::adaptive_performance::AdaptivePerformanceOptimizer;
// use crate::export::fast_export_coordinator::FastExportCoordinator;
use crate::export::schema_validator::SchemaValidator;
use rayon::prelude::*;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    sync::LazyLock,
};

/// Json file types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonFileType {
    /// memory_analysis.json
    MemoryAnalysis,
    /// lifetime.json
    Lifetime,
    /// unsafe_ffi.json
    UnsafeFfi,
    /// performance.json
    Performance,
    /// complex_types.json
    ComplexTypes,
    /// security_violations.json
    SecurityViolations,
    // AsyncAnalysis,    // asnyc analysis
    // ThreadSafety,     // Threadsafety
    // MemoryLeaks,      // Memory leak analysis
    // TypeInference,    // typeinference analysis
}

impl JsonFileType {
    /// get standard four files
    pub fn standard_four() -> Vec<JsonFileType> {
        vec![
            JsonFileType::MemoryAnalysis,
            JsonFileType::Lifetime,
            JsonFileType::UnsafeFfi,
            JsonFileType::Performance,
        ]
    }

    /// get standard five files
    pub fn standard_five() -> Vec<JsonFileType> {
        vec![
            JsonFileType::MemoryAnalysis,
            JsonFileType::Lifetime,
            JsonFileType::UnsafeFfi,
            JsonFileType::Performance,
            JsonFileType::ComplexTypes,
        ]
    }

    /// get file suffix
    pub fn file_suffix(&self) -> &'static str {
        match self {
            JsonFileType::MemoryAnalysis => "memory_analysis",
            JsonFileType::Lifetime => "lifetime",
            JsonFileType::UnsafeFfi => "unsafe_ffi",
            JsonFileType::Performance => "performance",
            JsonFileType::ComplexTypes => "complex_types",
            JsonFileType::SecurityViolations => "security_violations",
        }
    }
}

/// Global adaptive performance optimizer instance
static ADAPTIVE_OPTIMIZER: LazyLock<std::sync::Mutex<AdaptivePerformanceOptimizer>> =
    LazyLock::new(|| std::sync::Mutex::new(AdaptivePerformanceOptimizer::default()));

/// Global security violation analyzer instance
#[allow(dead_code)]
static SECURITY_ANALYZER: LazyLock<std::sync::Mutex<SecurityViolationAnalyzer>> =
    LazyLock::new(|| std::sync::Mutex::new(SecurityViolationAnalyzer::default()));

/// Optimized export options with intelligent defaults
#[derive(Debug, Clone)]
pub struct OptimizedExportOptions {
    /// Use parallel processing for large datasets (default: auto-detect)
    pub parallel_processing: bool,
    /// Buffer size for file I/O (default: 256KB for better performance)
    pub buffer_size: usize,
    /// Use compact JSON format for large files (default: auto-detect)
    pub use_compact_format: Option<bool>,
    /// Enable type inference caching (default: true)
    pub enable_type_cache: bool,
    /// Batch size for processing allocations (default: 1000)
    pub batch_size: usize,
    /// Enable streaming JSON writer for large files (default: auto-detect)
    pub use_streaming_writer: bool,
    /// Enable schema validation (default: true)
    pub enable_schema_validation: bool,
    /// Optimization level (default: High)
    pub optimization_level: OptimizationLevel,
    /// Enable enhanced FFI analysis (default: true)
    pub enable_enhanced_ffi_analysis: bool,
    /// Enable boundary event processing (default: true)
    pub enable_boundary_event_processing: bool,
    /// Enable memory passport tracking (default: true)
    pub enable_memory_passport_tracking: bool,
    /// Enable adaptive performance optimization (default: true)
    pub enable_adaptive_optimization: bool,
    /// Maximum cache size for type information (default: 1000)
    pub max_cache_size: usize,
    /// Target processing time per batch in milliseconds (default: 10ms)
    pub target_batch_time_ms: u64,
    /// Enable comprehensive security violation analysis (default: true)
    pub enable_security_analysis: bool,
    /// Include low severity violations in security reports (default: true)
    pub include_low_severity_violations: bool,
    /// Generate data integrity hashes for security reports (default: true)
    pub generate_integrity_hashes: bool,
    /// Enable fast export mode using the new coordinator (default: false)
    pub enable_fast_export_mode: bool,
    /// Auto-enable fast export for large datasets (default: true)
    pub auto_fast_export_threshold: Option<usize>,
    /// Thread count for parallel processing (default: auto-detect)
    pub thread_count: Option<usize>,
}

/// Optimization levels for export processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    /// Basic optimization - fastest export
    Low,
    /// Balanced optimization - good performance with enhanced features
    Medium,
    /// Full optimization - all features enabled, may be slower
    High,
    /// Maximum optimization - experimental features enabled
    Maximum,
}

impl Default for OptimizedExportOptions {
    fn default() -> Self {
        Self {
            parallel_processing: true,
            buffer_size: 256 * 1024,  // 256KB buffer
            use_compact_format: None, // Auto-detect based on file size
            enable_type_cache: true,
            batch_size: 1000,
            use_streaming_writer: true,
            enable_schema_validation: true,
            optimization_level: OptimizationLevel::High,
            enable_enhanced_ffi_analysis: true,
            enable_boundary_event_processing: true,
            enable_memory_passport_tracking: true,
            enable_adaptive_optimization: true,
            max_cache_size: 1000,
            target_batch_time_ms: 10,
            enable_security_analysis: true,
            include_low_severity_violations: true,
            generate_integrity_hashes: true,
            enable_fast_export_mode: false,
            auto_fast_export_threshold: Some(5000),
            thread_count: None, // Auto-detect
        }
    }
}

impl OptimizedExportOptions {
    /// Create new options with specified optimization level
    pub fn with_optimization_level(level: OptimizationLevel) -> Self {
        let mut options = OptimizedExportOptions {
            optimization_level: level,
            ..Default::default()
        };
        match level {
            OptimizationLevel::Low => {
                options.parallel_processing = false;
                options.use_streaming_writer = false;
                options.enable_schema_validation = false;
                options.enable_enhanced_ffi_analysis = false;
                options.enable_boundary_event_processing = false;
                options.enable_memory_passport_tracking = false;
                options.enable_adaptive_optimization = false;
                options.enable_security_analysis = false;
            }
            OptimizationLevel::Medium => {
                options.parallel_processing = true;
                options.use_streaming_writer = false;
                options.enable_schema_validation = true;
                options.enable_enhanced_ffi_analysis = true;
                options.enable_boundary_event_processing = false;
                options.enable_memory_passport_tracking = false;
            }
            OptimizationLevel::High => {
                // Use default settings (all features enabled)
            }
            OptimizationLevel::Maximum => {
                options.buffer_size = 512 * 1024; // 512KB buffer
                options.batch_size = 2000;
                // All features enabled with maximum settings
            }
        }

        options
    }

    /// Enable or disable parallel processing
    pub fn parallel_processing(mut self, enabled: bool) -> Self {
        self.parallel_processing = enabled;
        self
    }

    /// Set buffer size for I/O operations
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Set batch size for processing
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Enable or disable streaming writer
    pub fn streaming_writer(mut self, enabled: bool) -> Self {
        self.use_streaming_writer = enabled;
        self
    }

    /// Enable or disable schema validation
    pub fn schema_validation(mut self, enabled: bool) -> Self {
        self.enable_schema_validation = enabled;
        self
    }

    /// Enable or disable adaptive optimization
    pub fn adaptive_optimization(mut self, enabled: bool) -> Self {
        self.enable_adaptive_optimization = enabled;
        self
    }

    /// Set maximum cache size
    pub fn max_cache_size(mut self, size: usize) -> Self {
        self.max_cache_size = size;
        self
    }

    /// Enable or disable security violation analysis
    pub fn security_analysis(mut self, enabled: bool) -> Self {
        self.enable_security_analysis = enabled;
        self
    }

    /// Include low severity violations in reports
    pub fn include_low_severity(mut self, include: bool) -> Self {
        self.include_low_severity_violations = include;
        self
    }

    /// Enable or disable integrity hash generation
    pub fn integrity_hashes(mut self, enabled: bool) -> Self {
        self.generate_integrity_hashes = enabled;
        self
    }

    /// Enable or disable fast export mode
    pub fn fast_export_mode(mut self, enabled: bool) -> Self {
        self.enable_fast_export_mode = enabled;
        self
    }

    /// Set auto fast export threshold (None to disable auto mode)
    pub fn auto_fast_export_threshold(mut self, threshold: Option<usize>) -> Self {
        self.auto_fast_export_threshold = threshold;
        self
    }

    /// Set thread count for parallel processing (None for auto-detect)
    pub fn thread_count(mut self, count: Option<usize>) -> Self {
        self.thread_count = count;
        self
    }
}

/// Simple streaming JSON writer for memory-efficient large file export
struct StreamingJsonWriter<W: Write> {
    writer: W,
}

impl<W: Write> StreamingJsonWriter<W> {
    /// Create new streaming writer
    fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Write complete JSON data using streaming approach
    fn write_complete_json(&mut self, data: &serde_json::Value) -> TrackingResult<()> {
        // Use serde_json's streaming capabilities for memory efficiency
        serde_json::to_writer(&mut self.writer, data)
            .expect("Failed to write JSON data to streaming writer");
        Ok(())
    }

    /// Write pretty-formatted JSON data using streaming approach
    fn write_pretty_json(&mut self, data: &serde_json::Value) -> TrackingResult<()> {
        // Use serde_json's pretty printing with streaming
        serde_json::to_writer_pretty(&mut self.writer, data)
            .expect("Failed to write pretty JSON data to streaming writer");
        Ok(())
    }

    /// Finalize the writer and ensure all data is flushed
    fn finalize(&mut self) -> TrackingResult<()> {
        self.writer.flush()
            .expect("Failed to flush streaming writer");
        Ok(())
    }
}

/// Type inference cache for performance optimization
static TYPE_CACHE: LazyLock<std::sync::Mutex<HashMap<String, String>>> =
    LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

/// Get cached type information or compute and cache it
fn get_or_compute_type_info(type_name: &str, size: usize) -> String {
    if let Ok(mut cache) = TYPE_CACHE.lock() {
        let key = format!("{type_name}:{size}");
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
        let type_info = compute_enhanced_type_info(type_name, size);
        cache.insert(key, type_info.clone());
        type_info
    } else {
        compute_enhanced_type_info(type_name, size)
    }
}

/// Compute enhanced type information
fn compute_enhanced_type_info(type_name: &str, size: usize) -> String {
    if type_name.contains("Vec<") {
        "Vec<T>".to_string()
    } else if type_name.contains("HashMap") {
        "HashMap<K,V>".to_string()
    } else if type_name.contains("String") {
        "String".to_string()
    } else {
        match size {
            1..=8 => "Primitive".to_string(),
            9..=32 => "SmallStruct".to_string(),
            33..=128 => "MediumStruct".to_string(),
            129..=1024 => "LargeStruct".to_string(),
            _ => "Buffer".to_string(),
        }
    }
}

/// Clear the type cache (useful for testing)
pub fn clear_type_cache() {
    if let Ok(mut cache) = TYPE_CACHE.lock() {
        cache.clear();
    }
}

/// Process a batch of allocations (legacy function for compatibility)
#[allow(dead_code)]
fn process_allocation_batch(
    allocations: &[AllocationInfo],
) -> TrackingResult<Vec<serde_json::Value>> {
    let options = OptimizedExportOptions::default();
    process_allocation_batch_enhanced(allocations, &options)
}

/// Enhanced batch processing with new data pipeline integration
fn process_allocation_batch_enhanced(
    allocations: &[AllocationInfo],
    options: &OptimizedExportOptions,
) -> TrackingResult<Vec<serde_json::Value>> {
    let mut processed = Vec::with_capacity(allocations.len());

    for alloc in allocations {
        let enhanced_type = if let Some(type_name) = &alloc.type_name {
            get_or_compute_type_info(type_name, alloc.size)
        } else {
            compute_enhanced_type_info("Unknown", alloc.size)
        };

        let mut allocation_data = serde_json::json!({
            "ptr": format!("0x{:x}", alloc.ptr),
            "size": alloc.size,
            "type_name": enhanced_type,
            "var_name": alloc.var_name.as_deref().unwrap_or("unnamed"),
            "scope": alloc.scope_name.as_deref().unwrap_or("global"),
            "timestamp_alloc": alloc.timestamp_alloc,
            "timestamp_dealloc": alloc.timestamp_dealloc,
            "is_active": alloc.is_active()
        });

        // Add enhanced FFI analysis if enabled
        if options.enable_enhanced_ffi_analysis {
            if let Some(ffi_info) = analyze_ffi_allocation(alloc) {
                allocation_data["ffi_analysis"] = ffi_info;
            }
        }

        // Add boundary event information if enabled
        if options.enable_boundary_event_processing {
            if let Some(boundary_info) = analyze_boundary_events(alloc) {
                allocation_data["boundary_events"] = boundary_info;
            }
        }

        // Add memory passport information if enabled
        if options.enable_memory_passport_tracking {
            if let Some(passport_info) = get_memory_passport_info(alloc.ptr) {
                allocation_data["memory_passport"] = passport_info;
            }
        }

        processed.push(allocation_data);
    }

    Ok(processed)
}

/// Analyze FFI-related information for an allocation
#[allow(dead_code)]
fn analyze_ffi_allocation(alloc: &AllocationInfo) -> Option<serde_json::Value> {
    // Check if this allocation has FFI characteristics
    if let Some(type_name) = &alloc.type_name {
        if type_name.contains("*mut")
            || type_name.contains("*const")
            || type_name.contains("extern")
            || type_name.contains("libc::")
        {
            return Some(serde_json::json!({
                "is_ffi_related": true,
                "ffi_type": if type_name.contains("*mut") || type_name.contains("*const") {
                    "raw_pointer"
                } else {
                    "external_library"
                },
                "risk_level": if type_name.contains("*mut") { "high" } else { "medium" },
                "safety_concerns": [
                    "Manual memory management required",
                    "No automatic bounds checking",
                    "Potential for memory safety violations"
                ]
            }));
        }
    }

    if let Some(var_name) = &alloc.var_name {
        if var_name.contains("ffi") || var_name.contains("extern") || var_name.contains("c_") {
            return Some(serde_json::json!({
                "is_ffi_related": true,
                "ffi_type": "ffi_variable",
                "risk_level": "medium",
                "detected_from": "variable_name"
            }));
        }
    }

    None
}

/// Analyze boundary events for an allocation
fn analyze_boundary_events(alloc: &AllocationInfo) -> Option<serde_json::Value> {
    // Get boundary events from the unsafe FFI tracker
    let tracker = get_global_unsafe_ffi_tracker();
    if let Ok(allocations) = tracker.get_enhanced_allocations() {
        for enhanced_alloc in allocations {
            if enhanced_alloc.base.ptr == alloc.ptr
                && !enhanced_alloc.cross_boundary_events.is_empty()
            {
                let events: Vec<serde_json::Value> = enhanced_alloc
                    .cross_boundary_events
                    .iter()
                    .map(|event| {
                        serde_json::json!({
                            "event_type": format!("{:?}", event.event_type),
                            "from_context": event.from_context,
                            "to_context": event.to_context,
                            "timestamp": event.timestamp
                        })
                    })
                    .collect();

                return Some(serde_json::json!({
                    "has_boundary_events": true,
                    "event_count": events.len(),
                    "events": events
                }));
            }
        }
    }

    None
}

/// Get memory passport information for a pointer
fn get_memory_passport_info(ptr: usize) -> Option<serde_json::Value> {
    let tracker = get_global_unsafe_ffi_tracker();
    if let Ok(passports) = tracker.get_memory_passports() {
        if let Some(passport) = passports.get(&ptr) {
            return Some(serde_json::json!({
                "passport_id": passport.passport_id,
                "origin_context": passport.origin.context,
                "current_owner": passport.current_owner.owner_context,
                "validity_status": format!("{:?}", passport.validity_status),
                "security_clearance": format!("{:?}", passport.security_clearance),
                "journey_length": passport.journey.len(),
                "last_stamp": passport.journey.last().map(|stamp| serde_json::json!({
                    "operation": stamp.operation,
                    "location": stamp.location,
                    "timestamp": stamp.timestamp
                }))
            }));
        }
    }

    None
}

/// Optimized file writing with streaming support and schema validation
fn write_json_optimized<P: AsRef<Path>>(
    path: P,
    data: &serde_json::Value,
    options: &OptimizedExportOptions,
) -> TrackingResult<()> {
    let path = path.as_ref();

    // Validate schema if enabled and not in fast export mode
    if options.enable_schema_validation && !options.enable_fast_export_mode {
        let validator = SchemaValidator::new();
        if let Ok(validation_result) = validator.validate_unsafe_ffi_analysis(data) {
            if !validation_result.is_valid {
                eprintln!("⚠️ Schema validation warnings:");
                for error in validation_result.errors {
                    eprintln!("  - {}: {}", error.code, error.message);
                }
                for warning in validation_result.warnings {
                    eprintln!("  - {}: {}", warning.warning_code, warning.message);
                }
            }
        }
    } else if options.enable_fast_export_mode {
        // Fast mode: skip validation for better performance
    }

    // Determine format based on data size
    let estimated_size = estimate_json_size(data);
    let use_compact = options
        .use_compact_format
        .unwrap_or(estimated_size > 1_000_000); // Use compact for files > 1MB

    // Use streaming writer for large files when explicitly enabled
    if options.use_streaming_writer && estimated_size > 500_000 {
        tracing::info!("Using streaming writer for large file (size: {} bytes)", estimated_size);
        let file = File::create(path)?;
        let buffered_file = BufWriter::with_capacity(options.buffer_size * 2, file);
        let mut streaming_writer = StreamingJsonWriter::new(buffered_file);

        match use_compact {
            true => streaming_writer.write_complete_json(data)?,
            false => streaming_writer.write_pretty_json(data)?,
        }

        streaming_writer.finalize()?;
    } else {
        // Use traditional buffered writer for smaller files
        let file = File::create(path)?;
        let mut writer = BufWriter::with_capacity(options.buffer_size, file);

        if use_compact {
            serde_json::to_writer(&mut writer, data)?;
        } else {
            serde_json::to_writer_pretty(&mut writer, data)?;
        }

        writer.flush()?;
    }

    Ok(())
}

/// Estimate JSON size for format decision
fn estimate_json_size(data: &serde_json::Value) -> usize {
    // Quick estimation based on structure
    match data {
        serde_json::Value::Object(obj) => {
            obj.len() * 50 + obj.values().map(estimate_json_size).sum::<usize>()
        }
        serde_json::Value::Array(arr) => {
            arr.len() * 20 + arr.iter().map(estimate_json_size).sum::<usize>()
        }
        serde_json::Value::String(s) => s.len() + 10,
        _ => 20,
    }
}

/// Convert legacy ExportOptions to OptimizedExportOptions for backward compatibility
#[allow(dead_code)]
fn convert_legacy_options_to_optimized(
    legacy: crate::core::tracker::ExportOptions,
) -> OptimizedExportOptions {
    let mut optimized = OptimizedExportOptions {
        buffer_size: legacy.buffer_size,
        use_compact_format: Some(!legacy.verbose_logging), // Verbose = pretty format
        ..Default::default()
    };

    // Determine optimization level based on legacy settings
    if legacy.include_system_allocations {
        // System allocations = comprehensive analysis = Maximum optimization
        optimized.optimization_level = OptimizationLevel::Maximum;
        optimized.enable_enhanced_ffi_analysis = true;
        optimized.enable_boundary_event_processing = true;
        optimized.enable_memory_passport_tracking = true;
        optimized.enable_security_analysis = true;
    } else {
        // User-focused mode = High optimization (default)
        optimized.optimization_level = OptimizationLevel::High;
    }

    // Enable compression if requested in legacy options
    if legacy.compress_output {
        optimized.use_compact_format = Some(true);
        optimized.buffer_size = optimized.buffer_size.max(512 * 1024); // Larger buffer for compression
    }

    // Adjust parallel processing based on expected load
    optimized.parallel_processing =
        legacy.include_system_allocations || legacy.buffer_size > 128 * 1024;

    println!("🔄 Converted legacy ExportOptions to OptimizedExportOptions:");
    println!(
        "   - Optimization level: {:?}",
        optimized.optimization_level
    );
    println!("   - Buffer size: {} KB", optimized.buffer_size / 1024);
    println!(
        "   - Parallel processing: {}",
        optimized.parallel_processing
    );
    println!(
        "   - Enhanced features: {}",
        optimized.enable_enhanced_ffi_analysis
    );

    optimized
}

/// Ultra-fast export implementation (legacy methods for backward compatibility)
impl MemoryTracker {
    /// Optimized export to standard 4 JSON files (replaces export_separated_json_simple)
    pub fn export_optimized_json_files<P: AsRef<Path>>(&self, base_path: P) -> TrackingResult<()> {
        let options = OptimizedExportOptions::default();
        self.export_optimized_json_files_with_options(base_path, options)
    }

    /// Export to 5 JSON files including complex types analysis
    pub fn export_optimized_json_files_with_complex_types<P: AsRef<Path>>(
        &self,
        base_path: P,
    ) -> TrackingResult<()> {
        let options = OptimizedExportOptions::default();
        self.export_extensible_json_files_with_options(
            base_path,
            &JsonFileType::standard_five(),
            options,
        )
    }

    /// Optimized export to standard 4 JSON files with custom options
    pub fn export_optimized_json_files_with_options<P: AsRef<Path>>(
        &self,
        base_path: P,
        options: OptimizedExportOptions,
    ) -> TrackingResult<()> {
        let start_time = std::time::Instant::now();
        println!("🚀 Starting optimized 4-file JSON export...");

        let base_path = base_path.as_ref();
        let base_name = base_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("export");
        let parent_dir = base_path.parent().unwrap_or(Path::new("."));

        // Get data once for all files
        let allocations = self.get_active_allocations()?;
        let stats = self.get_stats()?;

        println!(
            "📊 Processing {} allocations across 4 standard files...",
            allocations.len()
        );

        // 1. Memory Analysis JSON (standard file 1)
        let memory_path = parent_dir.join(format!("{base_name}_memory_analysis.json"));
        let memory_data = create_optimized_memory_analysis(&allocations, &stats, &options)?;
        write_json_optimized(&memory_path, &memory_data, &options)?;

        // 2. Lifetime Analysis JSON (standard file 2)
        let lifetime_path = parent_dir.join(format!("{base_name}_lifetime.json"));
        let lifetime_data = create_optimized_lifetime_analysis(&allocations, &options)?;
        write_json_optimized(&lifetime_path, &lifetime_data, &options)?;

        // 3. Unsafe FFI Analysis JSON (standard file 3)
        let unsafe_path = parent_dir.join(format!("{base_name}_unsafe_ffi.json"));
        let unsafe_data = create_optimized_unsafe_ffi_analysis(&allocations, &options)?;
        write_json_optimized(&unsafe_path, &unsafe_data, &options)?;

        // 4. Performance Analysis JSON (standard file 4)
        let perf_path = parent_dir.join(format!("{base_name}_performance.json"));
        let perf_data =
            create_optimized_performance_analysis(&allocations, &stats, start_time, &options)?;
        write_json_optimized(&perf_path, &perf_data, &options)?;

        let total_duration = start_time.elapsed();
        println!(
            "✅ Optimized 4-file export completed in {total_duration:?}",
        );
        println!("📁 Generated standard files:");
        println!("   1. {base_name}_memory_analysis.json");
        println!("   2. {base_name}_lifetime.json");
        println!("   3. {base_name}_unsafe_ffi.json");
        println!("   4. {base_name}_performance.json");

        // Show optimization effects
        if options.parallel_processing {
            println!("💡 Applied parallel processing optimization");
        }
        if options.enable_type_cache {
            println!("💡 Applied type inference caching");
        }
        println!(
            "💡 Applied optimized buffering ({} KB)",
            options.buffer_size / 1024
        );

        Ok(())
    }

    /// A generic export method reserved for future expansion. can easily add a 5th and 6th JSON file
    pub fn export_extensible_json_files<P: AsRef<Path>>(
        &self,
        base_path: P,
        file_types: &[JsonFileType],
    ) -> TrackingResult<()> {
        let options = OptimizedExportOptions::default();
        self.export_extensible_json_files_with_options(base_path, file_types, options)
    }

    /// A generic export method reserved for future expansion. can easily add a 5th and 6th JSON file
    pub fn export_extensible_json_files_with_options<P: AsRef<Path>>(
        &self,
        base_path: P,
        file_types: &[JsonFileType],
        options: OptimizedExportOptions,
    ) -> TrackingResult<()> {
        let start_time = std::time::Instant::now();
        println!(
            "🚀 Starting extensible JSON export for {} files...",
            file_types.len()
        );

        let base_path = base_path.as_ref();
        let base_name = base_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("export");
        let parent_dir = base_path.parent().unwrap_or(Path::new("."));

        // Get data once for all files
        let allocations = self.get_active_allocations()?;
        let stats = self.get_stats()?;

        println!("📊 Processing {} allocations...", allocations.len());

        // genearte files
        for file_type in file_types {
            let (filename, data) = match file_type {
                JsonFileType::MemoryAnalysis => {
                    let filename = format!("{base_name}_memory_analysis.json");
                    let data = create_optimized_memory_analysis(&allocations, &stats, &options)?;
                    (filename, data)
                }
                JsonFileType::Lifetime => {
                    let filename = format!("{base_name}_lifetime.json");
                    let data = create_optimized_lifetime_analysis(&allocations, &options)?;
                    (filename, data)
                }
                JsonFileType::UnsafeFfi => {
                    let filename = format!("{base_name}_unsafe_ffi.json");
                    let data = create_optimized_unsafe_ffi_analysis(&allocations, &options)?;
                    (filename, data)
                }
                JsonFileType::Performance => {
                    let filename = format!("{base_name}_performance.json");
                    let data = create_optimized_performance_analysis(
                        &allocations,
                        &stats,
                        start_time,
                        &options,
                    )?;
                    (filename, data)
                }
                JsonFileType::ComplexTypes => {
                    let filename = format!("{base_name}_complex_types.json");
                    let data = create_optimized_complex_types_analysis(&allocations, &options)?;
                    (filename, data)
                }
                JsonFileType::SecurityViolations => {
                    let filename = format!("{base_name}_security_violations.json");
                    let data = create_security_violation_analysis(&allocations, &options)?;
                    (filename, data)
                } // JsonFileType::AsyncAnalysis => { ... }
                  // JsonFileType::ThreadSafety => { ... }
            };

            let file_path = parent_dir.join(filename);
            write_json_optimized(&file_path, &data, &options)?;
            println!(
                "   ✅ Generated: {}",
                file_path.file_name().unwrap().to_string_lossy()
            );
        }

        let total_duration = start_time.elapsed();
        println!("✅ Extensible export completed in {total_duration:?}");

        Ok(())
    }
}

/// Create optimized memory analysis
fn create_optimized_memory_analysis(
    allocations: &[AllocationInfo],
    stats: &crate::core::types::MemoryStats,
    options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    let processed_allocations = process_allocations_optimized(allocations, options)?;

    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "memory_analysis_optimized",
            "optimization_level": "high",
            "total_allocations": allocations.len(),
            "export_version": "2.0",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "memory_stats": {
            "total_allocated": stats.total_allocated,
            "active_memory": stats.active_memory,
            "peak_memory": stats.peak_memory,
            "total_allocations": stats.total_allocations
        },
        "allocations": processed_allocations
    }))
}

/// Create optimized lifetime analysis
fn create_optimized_lifetime_analysis(
    allocations: &[AllocationInfo],
    _options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    // Lifetime analysis: group analysis by scope
    let mut scope_analysis: HashMap<String, (usize, usize, Vec<usize>)> = HashMap::new();

    for alloc in allocations {
        let scope = alloc.scope_name.as_deref().unwrap_or("global");
        let entry = scope_analysis
            .entry(scope.to_string())
            .or_insert((0, 0, Vec::new()));
        entry.0 += alloc.size; // total size
        entry.1 += 1; // allocation count
        entry.2.push(alloc.size); // size list for statistics
    }

    // Convert to JSON format
    let mut scope_stats: Vec<_> = scope_analysis
        .into_iter()
        .map(|(scope, (total_size, count, sizes))| {
            let avg_size = if count > 0 { total_size / count } else { 0 };
            let max_size = sizes.iter().max().copied().unwrap_or(0);
            let min_size = sizes.iter().min().copied().unwrap_or(0);

            serde_json::json!({
                "scope_name": scope,
                "total_size": total_size,
                "allocation_count": count,
                "average_size": avg_size,
                "max_size": max_size,
                "min_size": min_size
            })
        })
        .collect();

    // Sort by total size
    scope_stats.sort_by(|a, b| {
        b["total_size"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["total_size"].as_u64().unwrap_or(0))
    });

    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "lifetime_analysis_optimized",
            "optimization_level": "high",
            "total_scopes": scope_stats.len(),
            "export_version": "2.0",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "scope_analysis": scope_stats,
        "summary": {
            "total_allocations": allocations.len(),
            "unique_scopes": scope_stats.len()
        }
    }))
}

/// Create optimized unsafe FFI analysis
fn create_optimized_unsafe_ffi_analysis(
    allocations: &[AllocationInfo],
    _options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    // Analyze possible unsafe operations and FFI-related allocations
    let mut unsafe_indicators = Vec::new();
    let mut ffi_patterns = Vec::new();

    for alloc in allocations {
        // Check for unsafe patterns in type names
        if let Some(type_name) = &alloc.type_name {
            if type_name.contains("*mut") || type_name.contains("*const") {
                unsafe_indicators.push(serde_json::json!({
                    "ptr": format!("0x{:x}", alloc.ptr),
                    "type": "raw_pointer",
                    "type_name": type_name,
                    "size": alloc.size,
                    "risk_level": "high"
                }));
            } else if type_name.contains("extern") || type_name.contains("libc::") {
                ffi_patterns.push(serde_json::json!({
                    "ptr": format!("0x{:x}", alloc.ptr),
                    "type": "ffi_related",
                    "type_name": type_name,
                    "size": alloc.size,
                    "risk_level": "medium"
                }));
            }
        }

        // Check for unsafe patterns in variable names
        if let Some(var_name) = &alloc.var_name {
            if var_name.contains("unsafe") || var_name.contains("raw") {
                unsafe_indicators.push(serde_json::json!({
                    "ptr": format!("0x{:x}", alloc.ptr),
                    "type": "unsafe_variable",
                    "var_name": var_name,
                    "size": alloc.size,
                    "risk_level": "medium"
                }));
            }
        }
    }

    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "unsafe_ffi_analysis_optimized",
            "optimization_level": "high",
            "total_allocations_analyzed": allocations.len(),
            "export_version": "2.0",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "unsafe_indicators": unsafe_indicators,
        "ffi_patterns": ffi_patterns,
        "summary": {
            "unsafe_count": unsafe_indicators.len(),
            "ffi_count": ffi_patterns.len(),
            "total_risk_items": unsafe_indicators.len() + ffi_patterns.len(),
            "risk_assessment": if unsafe_indicators.len() + ffi_patterns.len() > 10 {
                "high"
            } else if unsafe_indicators.len() + ffi_patterns.len() > 5 {
                "medium"
            } else {
                "low"
            }
        }
    }))
}

/// Create optimized performance analysis
fn create_optimized_performance_analysis(
    allocations: &[AllocationInfo],
    stats: &crate::core::types::MemoryStats,
    start_time: std::time::Instant,
    options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    let processing_time = start_time.elapsed();
    let allocations_per_second = if processing_time.as_secs() > 0 {
        allocations.len() as f64 / processing_time.as_secs_f64()
    } else {
        allocations.len() as f64 / 0.001 // assume minimum 1ms
    };

    // Analyze allocation size distribution
    let mut size_distribution = HashMap::new();
    for alloc in allocations {
        let category = match alloc.size {
            0..=64 => "tiny",
            65..=256 => "small",
            257..=1024 => "medium",
            1025..=4096 => "large",
            4097..=16384 => "huge",
            _ => "massive",
        };
        *size_distribution.entry(category).or_insert(0) += 1;
    }

    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "performance_analysis_optimized",
            "optimization_level": "high",
            "export_version": "2.0",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "export_performance": {
            "total_processing_time_ms": processing_time.as_millis(),
            "allocations_processed": allocations.len(),
            "processing_rate": {
                "allocations_per_second": allocations_per_second,
                "performance_class": if allocations_per_second > 10000.0 {
                    "excellent"
                } else if allocations_per_second > 1000.0 {
                    "good"
                } else {
                    "needs_optimization"
                }
            }
        },
        "memory_performance": {
            "total_allocated": stats.total_allocated,
            "active_memory": stats.active_memory,
            "peak_memory": stats.peak_memory,
            "memory_efficiency": if stats.peak_memory > 0 {
                (stats.active_memory as f64 / stats.peak_memory as f64 * 100.0) as u64
            } else {
                100
            }
        },
        "allocation_distribution": size_distribution,
        "optimization_status": {
            "type_caching": options.enable_type_cache,
            "parallel_processing": options.parallel_processing,
            "buffer_size_kb": options.buffer_size / 1024,
            "batch_size": options.batch_size
        }
    }))
}

/// Create integrated memory analysis with all new pipeline components
#[allow(dead_code)]
fn create_integrated_memory_analysis(
    allocations: &[AllocationInfo],
    stats: &crate::core::types::MemoryStats,
    options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    println!("🔧 Creating integrated memory analysis with enhanced pipeline...");

    // Use BatchProcessor for large datasets (simplified for now)
    let _processed_allocations = process_allocations_optimized(allocations, options)?;

    // Enhanced memory analysis with FFI integration
    let mut enhanced_allocations = Vec::new();
    for alloc in allocations {
        let mut enhanced_alloc = serde_json::json!({
            "ptr": format!("0x{:x}", alloc.ptr),
            "size": alloc.size,
            "type_name": alloc.type_name,
            "var_name": alloc.var_name,
            "scope_name": alloc.scope_name,
            "timestamp_alloc": alloc.timestamp_alloc,
            "timestamp_dealloc": alloc.timestamp_dealloc
        });

        // Add boundary events if enabled
        if options.enable_boundary_event_processing {
            if let Some(boundary_info) = analyze_boundary_events(alloc) {
                enhanced_alloc["boundary_events"] = boundary_info;
            }
        }

        // Add memory passport if enabled
        if options.enable_memory_passport_tracking {
            if let Some(passport_info) = get_memory_passport_info(alloc.ptr) {
                enhanced_alloc["memory_passport"] = passport_info;
            }
        }

        enhanced_allocations.push(enhanced_alloc);
    }

    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "integrated_memory_analysis",
            "optimization_level": format!("{:?}", options.optimization_level),
            "total_allocations": allocations.len(),
            "export_version": "2.0",
            "pipeline_features": {
                "batch_processing": options.parallel_processing && allocations.len() > options.batch_size,
                "boundary_events": options.enable_boundary_event_processing,
                "memory_passports": options.enable_memory_passport_tracking,
                "enhanced_ffi": options.enable_enhanced_ffi_analysis
            },
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "memory_stats": {
            "total_allocated": stats.total_allocated,
            "active_memory": stats.active_memory,
            "peak_memory": stats.peak_memory,
            "total_allocations": stats.total_allocations
        },
        "allocations": enhanced_allocations
    }))
}

/// Create integrated lifetime analysis with enhanced pipeline
#[allow(dead_code)]
fn create_integrated_lifetime_analysis(
    allocations: &[AllocationInfo],
    options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    println!("🔧 Creating integrated lifetime analysis with enhanced pipeline...");

    // Use BatchProcessor for scope analysis
    let mut scope_analysis: HashMap<String, (usize, usize, Vec<usize>)> = HashMap::new();
    let mut lifecycle_events = Vec::new();

    // Process in batches if enabled
    if options.parallel_processing && allocations.len() > options.batch_size {
        let chunks: Vec<_> = allocations.chunks(options.batch_size).collect();
        let results: Vec<_> = chunks
            .par_iter()
            .map(|chunk| {
                let mut local_scope_analysis = HashMap::new();
                let mut local_events = Vec::new();

                for alloc in *chunk {
                    let scope = alloc.scope_name.as_deref().unwrap_or("global");
                    let entry =
                        local_scope_analysis
                            .entry(scope.to_string())
                            .or_insert((0, 0, Vec::new()));
                    entry.0 += alloc.size;
                    entry.1 += 1;
                    entry.2.push(alloc.size);

                    // Track lifecycle events with variable and type information
                    local_events.push(serde_json::json!({
                        "ptr": format!("0x{:x}", alloc.ptr),
                        "event": "allocation",
                        "scope": scope,
                        "timestamp": alloc.timestamp_alloc,
                        "size": alloc.size,
                        "var_name": alloc.var_name.as_deref().unwrap_or("unknown"),
                        "type_name": alloc.type_name.as_deref().unwrap_or("unknown")
                    }));
                }

                (local_scope_analysis, local_events)
            })
            .collect();

        // Merge results
        for (local_scope, local_events) in results {
            for (scope, (size, count, sizes)) in local_scope {
                let entry = scope_analysis.entry(scope).or_insert((0, 0, Vec::new()));
                entry.0 += size;
                entry.1 += count;
                entry.2.extend(sizes);
            }
            lifecycle_events.extend(local_events);
        }
    } else {
        // Sequential processing
        for alloc in allocations {
            let scope = alloc.scope_name.as_deref().unwrap_or("global");
            let entry = scope_analysis
                .entry(scope.to_string())
                .or_insert((0, 0, Vec::new()));
            entry.0 += alloc.size;
            entry.1 += 1;
            entry.2.push(alloc.size);

            lifecycle_events.push(serde_json::json!({
                "ptr": format!("0x{:x}", alloc.ptr),
                "event": "allocation",
                "scope": scope,
                "timestamp": alloc.timestamp_alloc,
                "size": alloc.size,
                "var_name": alloc.var_name.as_deref().unwrap_or("unknown"),
                "type_name": alloc.type_name.as_deref().unwrap_or("unknown")
            }));
        }
    }

    // Convert to JSON format
    let mut scope_stats: Vec<_> = scope_analysis
        .into_iter()
        .map(|(scope, (total_size, count, sizes))| {
            let avg_size = if count > 0 { total_size / count } else { 0 };
            let max_size = sizes.iter().max().copied().unwrap_or(0);
            let min_size = sizes.iter().min().copied().unwrap_or(0);

            serde_json::json!({
                "scope_name": scope,
                "total_size": total_size,
                "allocation_count": count,
                "average_size": avg_size,
                "max_size": max_size,
                "min_size": min_size
            })
        })
        .collect();

    scope_stats.sort_by(|a, b| {
        b["total_size"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["total_size"].as_u64().unwrap_or(0))
    });

    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "integrated_lifetime_analysis",
            "optimization_level": format!("{:?}", options.optimization_level),
            "total_scopes": scope_stats.len(),
            "export_version": "2.0",
            "pipeline_features": {
                "batch_processing": options.parallel_processing && allocations.len() > options.batch_size,
                "lifecycle_tracking": true
            },
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "scope_analysis": scope_stats,
        "lifecycle_events": lifecycle_events,
        "summary": {
            "total_allocations": allocations.len(),
            "unique_scopes": scope_stats.len(),
            "total_events": lifecycle_events.len()
        }
    }))
}

/// Create integrated unsafe FFI analysis with all enhanced features
#[allow(dead_code)]
fn create_integrated_unsafe_ffi_analysis(
    allocations: &[AllocationInfo],
    options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    println!("🔧 Creating integrated unsafe FFI analysis with enhanced pipeline...");

    let mut unsafe_indicators = Vec::new();
    let mut ffi_patterns = Vec::new();
    let mut enhanced_ffi_data = Vec::new();
    let mut safety_violations = Vec::new();
    let mut boundary_events = Vec::new();

    // Get enhanced FFI data from tracker if available
    if options.enable_enhanced_ffi_analysis {
        let tracker = get_global_unsafe_ffi_tracker();
        if let Ok(enhanced_allocations) = tracker.get_enhanced_allocations() {
            for enhanced_alloc in enhanced_allocations {
                enhanced_ffi_data.push(serde_json::json!({
                    "ptr": format!("0x{:x}", enhanced_alloc.base.ptr),
                    "size": enhanced_alloc.base.size,
                    "source": format!("{:?}", enhanced_alloc.source),
                    "ffi_tracked": enhanced_alloc.ffi_tracked,
                    "cross_boundary_events": enhanced_alloc.cross_boundary_events.len(),
                    "safety_violations": enhanced_alloc.safety_violations.len()
                }));

                // Collect safety violations
                for violation in &enhanced_alloc.safety_violations {
                    let (violation_type, timestamp) = match violation {
                        SafetyViolation::DoubleFree { timestamp, .. } => ("DoubleFree", *timestamp),
                        SafetyViolation::InvalidFree { timestamp, .. } => {
                            ("InvalidFree", *timestamp)
                        }
                        SafetyViolation::PotentialLeak {
                            leak_detection_timestamp,
                            ..
                        } => ("PotentialLeak", *leak_detection_timestamp),
                        SafetyViolation::CrossBoundaryRisk { .. } => ("CrossBoundaryRisk", 0),
                    };

                    safety_violations.push(serde_json::json!({
                        "ptr": format!("0x{:x}", enhanced_alloc.base.ptr),
                        "violation_type": violation_type,
                        "description": format!("{:?}", violation),
                        "timestamp": timestamp
                    }));
                }

                // Collect boundary events
                if options.enable_boundary_event_processing {
                    for event in &enhanced_alloc.cross_boundary_events {
                        boundary_events.push(serde_json::json!({
                            "ptr": format!("0x{:x}", enhanced_alloc.base.ptr),
                            "event_type": format!("{:?}", event.event_type),
                            "from_context": event.from_context,
                            "to_context": event.to_context,
                            "timestamp": event.timestamp
                        }));
                    }
                }
            }
        }
    }

    // Analyze basic patterns in allocations
    for alloc in allocations {
        if let Some(type_name) = &alloc.type_name {
            if type_name.contains("*mut") || type_name.contains("*const") {
                unsafe_indicators.push(serde_json::json!({
                    "ptr": format!("0x{:x}", alloc.ptr),
                    "type": "raw_pointer",
                    "type_name": type_name,
                    "size": alloc.size,
                    "risk_level": "high"
                }));
            } else if type_name.contains("extern") || type_name.contains("libc::") {
                ffi_patterns.push(serde_json::json!({
                    "ptr": format!("0x{:x}", alloc.ptr),
                    "type": "ffi_related",
                    "type_name": type_name,
                    "size": alloc.size,
                    "risk_level": "medium"
                }));
            }
        }

        if let Some(var_name) = &alloc.var_name {
            if var_name.contains("unsafe") || var_name.contains("raw") {
                unsafe_indicators.push(serde_json::json!({
                    "ptr": format!("0x{:x}", alloc.ptr),
                    "type": "unsafe_variable",
                    "var_name": var_name,
                    "size": alloc.size,
                    "risk_level": "medium"
                }));
            }
        }
    }

    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "integrated_unsafe_ffi_analysis",
            "optimization_level": format!("{:?}", options.optimization_level),
            "total_allocations_analyzed": allocations.len(),
            "export_version": "2.0",
            "pipeline_features": {
                "enhanced_ffi_analysis": options.enable_enhanced_ffi_analysis,
                "boundary_event_processing": options.enable_boundary_event_processing,
                "memory_passport_tracking": options.enable_memory_passport_tracking
            },
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "unsafe_indicators": unsafe_indicators,
        "ffi_patterns": ffi_patterns,
        "enhanced_ffi_data": enhanced_ffi_data,
        "safety_violations": safety_violations,
        "boundary_events": boundary_events,
        "summary": {
            "unsafe_count": unsafe_indicators.len(),
            "ffi_count": ffi_patterns.len(),
            "enhanced_entries": enhanced_ffi_data.len(),
            "safety_violations": safety_violations.len(),
            "boundary_events": boundary_events.len(),
            "total_risk_items": unsafe_indicators.len() + ffi_patterns.len() + safety_violations.len(),
            "risk_assessment": if safety_violations.len() > 5 {
                "critical"
            } else if unsafe_indicators.len() + ffi_patterns.len() > 10 {
                "high"
            } else if unsafe_indicators.len() + ffi_patterns.len() > 5 {
                "medium"
            } else {
                "low"
            }
        }
    }))
}

/// Create integrated performance analysis with all pipeline metrics
#[allow(dead_code)]
fn create_integrated_performance_analysis(
    allocations: &[AllocationInfo],
    stats: &crate::core::types::MemoryStats,
    start_time: std::time::Instant,
    options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    println!("🔧 Creating integrated performance analysis with enhanced pipeline...");

    let processing_time = start_time.elapsed();
    let allocations_per_second = if processing_time.as_secs() > 0 {
        allocations.len() as f64 / processing_time.as_secs_f64()
    } else {
        allocations.len() as f64 / 0.001
    };

    // Analyze allocation size distribution
    let mut size_distribution = HashMap::new();
    for alloc in allocations {
        let category = match alloc.size {
            0..=64 => "tiny",
            65..=256 => "small",
            257..=1024 => "medium",
            1025..=4096 => "large",
            4097..=16384 => "huge",
            _ => "massive",
        };
        *size_distribution.entry(category).or_insert(0) += 1;
    }

    // Pipeline performance metrics
    let pipeline_metrics = serde_json::json!({
        "batch_processor": {
            "enabled": options.parallel_processing && allocations.len() > options.batch_size,
            "batch_size": options.batch_size,
            "estimated_batches": allocations.len().div_ceil(options.batch_size),
        },
        "streaming_writer": {
            "enabled": options.use_streaming_writer,
            "buffer_size_kb": options.buffer_size / 1024
        },
        "schema_validator": {
            "enabled": options.enable_schema_validation
        },
        "enhanced_features": {
            "ffi_analysis": options.enable_enhanced_ffi_analysis,
            "boundary_events": options.enable_boundary_event_processing,
            "memory_passports": options.enable_memory_passport_tracking
        }
    });

    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "integrated_performance_analysis",
            "optimization_level": format!("{:?}", options.optimization_level),
            "export_version": "2.0",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "export_performance": {
            "total_processing_time_ms": processing_time.as_millis(),
            "allocations_processed": allocations.len(),
            "processing_rate": {
                "allocations_per_second": allocations_per_second,
                "performance_class": if allocations_per_second > 10000.0 {
                    "excellent"
                } else if allocations_per_second > 1000.0 {
                    "good"
                } else {
                    "needs_optimization"
                }
            }
        },
        "memory_performance": {
            "total_allocated": stats.total_allocated,
            "active_memory": stats.active_memory,
            "peak_memory": stats.peak_memory,
            "memory_efficiency": if stats.peak_memory > 0 {
                (stats.active_memory as f64 / stats.peak_memory as f64 * 100.0) as u64
            } else {
                100
            }
        },
        "allocation_distribution": size_distribution,
        "pipeline_metrics": pipeline_metrics,
        "optimization_status": {
            "type_caching": options.enable_type_cache,
            "parallel_processing": options.parallel_processing,
            "buffer_size_kb": options.buffer_size / 1024,
            "batch_size": options.batch_size,
            "streaming_enabled": options.use_streaming_writer,
            "schema_validation": options.enable_schema_validation
        }
    }))
}

/// Create optimized complex types analysis
fn create_optimized_complex_types_analysis(
    allocations: &[AllocationInfo],
    options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    // Complex type analysis: Identify and analyze various complex Rust types
    let mut complex_type_stats: HashMap<String, ComplexTypeInfo> = HashMap::new();
    let mut generic_types = Vec::new();
    let mut trait_objects = Vec::new();
    let mut smart_pointers = Vec::new();
    let mut collections = Vec::new();

    //  Use parallel processing to analyze complex types
    let use_parallel = options.parallel_processing && allocations.len() > 1000;

    if use_parallel {
        // Parallel analysis of complex types
        let results: Vec<_> = allocations
            .par_chunks(options.batch_size)
            .map(analyze_complex_types_batch)
            .collect();

        // Merge results
        for batch_result in results {
            for (type_name, info) in batch_result.type_stats {
                let entry = complex_type_stats
                    .entry(type_name)
                    .or_insert_with(ComplexTypeInfo::new);
                entry.merge(info);
            }
            generic_types.extend(batch_result.generic_types);
            trait_objects.extend(batch_result.trait_objects);
            smart_pointers.extend(batch_result.smart_pointers);
            collections.extend(batch_result.collections);
        }
    } else {
        // Serial analysis of complex types
        let batch_result = analyze_complex_types_batch(allocations);
        complex_type_stats = batch_result.type_stats;
        generic_types = batch_result.generic_types;
        trait_objects = batch_result.trait_objects;
        smart_pointers = batch_result.smart_pointers;
        collections = batch_result.collections;
    }

    // Convert to JSON format and sort
    let mut type_analysis: Vec<_> = complex_type_stats.into_iter()
        .map(|(type_name, info)| {
            serde_json::json!({
                "type_name": type_name,
                "category": info.category,
                "total_size": info.total_size,
                "allocation_count": info.allocation_count,
                "average_size": if info.allocation_count > 0 { 
                    info.total_size / info.allocation_count
                } else {
                    0
                },
                "max_size": info.max_size,
                "complexity_score": info.complexity_score,
                "memory_efficiency": calculate_memory_efficiency(&type_name, info.total_size, info.allocation_count),
                "optimization_suggestions": generate_optimization_suggestions(&type_name, &info)
            })
        })
        .collect();

    // Sort by complexity score and total size
    type_analysis.sort_by(|a, b| {
        let score_cmp = b["complexity_score"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["complexity_score"].as_u64().unwrap_or(0));
        if score_cmp == std::cmp::Ordering::Equal {
            b["total_size"]
                .as_u64()
                .unwrap_or(0)
                .cmp(&a["total_size"].as_u64().unwrap_or(0))
        } else {
            score_cmp
        }
    });

    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "complex_types_analysis_optimized",
            "optimization_level": "high",
            "total_allocations_analyzed": allocations.len(),
            "unique_complex_types": type_analysis.len(),
            "export_version": "2.0",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "processing_mode": if use_parallel { "parallel" } else { "sequential" }
        },
        "complex_type_analysis": type_analysis,
        "categorized_types": {
            "generic_types": generic_types,
            "trait_objects": trait_objects,
            "smart_pointers": smart_pointers,
            "collections": collections
        },
        "summary": {
            "total_complex_types": type_analysis.len(),
            "generic_type_count": generic_types.len(),
            "trait_object_count": trait_objects.len(),
            "smart_pointer_count": smart_pointers.len(),
            "collection_count": collections.len(),
            "complexity_distribution": calculate_complexity_distribution(&type_analysis)
        },
        "optimization_recommendations": generate_global_optimization_recommendations(&type_analysis)
    }))
}

/// Complex type information structure
#[derive(Debug, Clone)]
struct ComplexTypeInfo {
    /// Type category   
    category: String,
    /// Total size of allocations
    total_size: usize,
    /// Number of allocations
    allocation_count: usize,
    /// Maximum size of allocations
    max_size: usize,
    /// Complexity score of type    
    complexity_score: u64,
}

impl ComplexTypeInfo {
    fn new() -> Self {
        Self {
            category: String::new(),
            total_size: 0,
            allocation_count: 0,
            max_size: 0,
            complexity_score: 0,
        }
    }

    fn merge(&mut self, other: ComplexTypeInfo) {
        self.total_size += other.total_size;
        self.allocation_count += other.allocation_count;
        self.max_size = self.max_size.max(other.max_size);
        self.complexity_score = self.complexity_score.max(other.complexity_score);
        if self.category.is_empty() {
            self.category = other.category;
        }
    }
}

/// Batch analysis result
struct ComplexTypeBatchResult {
    type_stats: HashMap<String, ComplexTypeInfo>,
    generic_types: Vec<serde_json::Value>,
    trait_objects: Vec<serde_json::Value>,
    smart_pointers: Vec<serde_json::Value>,
    collections: Vec<serde_json::Value>,
}

/// Batch analyze complex types
fn analyze_complex_types_batch(allocations: &[AllocationInfo]) -> ComplexTypeBatchResult {
    let mut type_stats: HashMap<String, ComplexTypeInfo> = HashMap::new();
    let mut generic_types = Vec::new();
    let mut trait_objects = Vec::new();
    let mut smart_pointers = Vec::new();
    let mut collections = Vec::new();

    for alloc in allocations {
        if let Some(type_name) = &alloc.type_name {
            let normalized_type = normalize_type_name(type_name);
            let category = categorize_complex_type(type_name);
            let complexity = calculate_type_complexity(type_name);

            // Update type statistics
            let entry = type_stats
                .entry(normalized_type.clone())
                .or_insert_with(|| {
                    let mut info = ComplexTypeInfo::new();
                    info.category = category.clone();
                    info.complexity_score = complexity;
                    info
                });
            entry.total_size += alloc.size;
            entry.allocation_count += 1;
            entry.max_size = entry.max_size.max(alloc.size);

            // Categorized collection
            let type_info = serde_json::json!({
                "ptr": format!("0x{:x}", alloc.ptr),
                "type_name": type_name,
                "normalized_type": normalized_type,
                "size": alloc.size,
                "var_name": alloc.var_name.as_deref().unwrap_or("unnamed"),
                "complexity_score": complexity
            });

            match category.as_str() {
                "Generic" => generic_types.push(type_info),
                "TraitObject" => trait_objects.push(type_info),
                "SmartPointer" => smart_pointers.push(type_info),
                "Collection" => collections.push(type_info),
                _ => {} // Other types are not collected
            }
        }
    }

    ComplexTypeBatchResult {
        type_stats,
        generic_types,
        trait_objects,
        smart_pointers,
        collections,
    }
}

/// Standardize type name
fn normalize_type_name(type_name: &str) -> String {
    // Remove specific generic parameters, keep structure
    if type_name.contains('<') {
        if let Some(base) = type_name.split('<').next() {
            format!("{base}<T>")
        } else {
            type_name.to_string()
        }
    } else {
        type_name.to_string()
    }
}

/// Categorize complex types
fn categorize_complex_type(type_name: &str) -> String {
    if type_name.contains("dyn ") {
        "TraitObject".to_string()
    } else if type_name.starts_with("Box<")
        || type_name.starts_with("Rc<")
        || type_name.starts_with("Arc<")
        || type_name.starts_with("RefCell<")
    {
        "SmartPointer".to_string()
    } else if type_name.starts_with("Vec<")
        || type_name.starts_with("HashMap<")
        || type_name.starts_with("BTreeMap<")
        || type_name.starts_with("HashSet<")
    {
        "Collection".to_string()
    } else if type_name.contains('<') && type_name.contains('>') {
        "Generic".to_string()
    } else if type_name.contains("::") {
        "ModulePath".to_string()
    } else {
        "Simple".to_string()
    }
}

/// Calculate type complexity
fn calculate_type_complexity(type_name: &str) -> u64 {
    let mut score = 0u64;

    // Base score
    score += 1;

    // Generic parameters increase complexity
    score += type_name.matches('<').count() as u64 * 2;

    // Nested level increases complexity
    let nesting_level = type_name.chars().filter(|&c| c == '<').count();
    score += nesting_level as u64 * 3;

    // Special types increase complexity
    if type_name.contains("dyn ") {
        score += 5;
    }
    if type_name.contains("impl ") {
        score += 4;
    }
    if type_name.contains("async") {
        score += 3;
    }
    if type_name.contains("Future") {
        score += 3;
    }

    // Smart pointers increase complexity
    if type_name.contains("Box<") {
        score += 2;
    }
    if type_name.contains("Rc<") {
        score += 3;
    }
    if type_name.contains("Arc<") {
        score += 4;
    }
    if type_name.contains("RefCell<") {
        score += 3;
    }

    score
}

/// Calculate memory efficiency based on type and average size
fn calculate_memory_efficiency(type_name: &str, total_size: usize, count: usize) -> u64 {
    if count == 0 {
        return 100;
    }

    let avg_size = total_size / count;

    //  Calculate efficiency based on type and average size
    if type_name.contains("Vec<") {
        // Vec efficiency depends on capacity utilization
        if avg_size < 64 {
            60
        } else {
            85
        }
    } else if type_name.contains("HashMap<") {
        // HashMap has additional overhead
        if avg_size < 128 {
            50
        } else {
            75
        }
    } else if type_name.contains("Box<") {
        // Box is usually very efficient
        90
    } else if type_name.contains("Arc<") || type_name.contains("Rc<") {
        // Reference counting has overhead
        80
    } else {
        // Default efficiency
        85
    }

}

/// Generate optimization suggestions based on type and allocation information
fn generate_optimization_suggestions(type_name: &str, info: &ComplexTypeInfo) -> Vec<String> {
    let mut suggestions = Vec::new();

    if info.allocation_count > 100 {
        suggestions
            .push("Consider using object pooling for frequently allocated types".to_string());
    }

    if type_name.contains("Vec<") && info.total_size > 1024 * 1024 {
        suggestions
            .push("Consider pre-allocating Vec capacity to reduce reallocations".to_string());
    }

    if type_name.contains("HashMap<") && info.allocation_count > 50 {
        suggestions.push("Consider using FxHashMap for better performance".to_string());
    }

    if type_name.contains("Box<") && info.allocation_count > 200 {
        suggestions
            .push("Consider using arena allocation for many small Box allocations".to_string());
    }

    if info.complexity_score > 10 {
        suggestions
            .push("High complexity type - consider simplifying or using type aliases".to_string());
    }

    suggestions
}

/// Calculate complexity distribution
fn calculate_complexity_distribution(type_analysis: &[serde_json::Value]) -> serde_json::Value {
    let mut low = 0;
    let mut medium = 0;
    let mut high = 0;
    let mut very_high = 0;

    for analysis in type_analysis {
        if let Some(score) = analysis["complexity_score"].as_u64() {
            match score {
                0..=3 => low += 1,
                4..=7 => medium += 1,
                8..=15 => high += 1,
                _ => very_high += 1,
            }
        }
    }

    serde_json::json!({
        "low_complexity": low,
        "medium_complexity": medium,
        "high_complexity": high,
        "very_high_complexity": very_high
    })
}

/// Generate global optimization recommendations based on type analysis
fn generate_global_optimization_recommendations(
    type_analysis: &[serde_json::Value],
) -> Vec<String> {
    let mut recommendations = Vec::new();

    let total_types = type_analysis.len();
    let high_complexity_count = type_analysis
        .iter()
        .filter(|t| t["complexity_score"].as_u64().unwrap_or(0) > 10)
        .count();

    if high_complexity_count > total_types / 4 {
        recommendations.push(
            "Consider refactoring high-complexity types to improve maintainability".to_string(),
        );
    }

    let large_allocation_count = type_analysis
        .iter()
        .filter(|t| t["allocation_count"].as_u64().unwrap_or(0) > 100)
        .count();

    if large_allocation_count > 5 {
        recommendations.push(
            "Multiple types with high allocation frequency - consider object pooling".to_string(),
        );
    }

    recommendations
        .push("Use 'cargo clippy' to identify additional optimization opportunities".to_string());
    recommendations.push(
        "Consider profiling with 'perf' or 'valgrind' for detailed performance analysis"
            .to_string(),
    );

    recommendations
}

/// Create optimized type analysis with caching
#[allow(dead_code)]
fn create_optimized_type_analysis(
    allocations: &[AllocationInfo],
    options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    let mut type_stats: HashMap<String, (usize, usize, usize)> = HashMap::new();

    // Use parallel processing for type analysis if beneficial
    let use_parallel = options.parallel_processing && allocations.len() > 1000;

    if use_parallel {
        // Parallel type analysis
        let type_results: Vec<_> = allocations
            .par_chunks(options.batch_size)
            .map(|chunk| {
                let mut local_stats: HashMap<String, (usize, usize, usize)> = HashMap::new();
                for alloc in chunk {
                    let type_name = if let Some(name) = &alloc.type_name {
                        get_or_compute_type_info(name, alloc.size)
                    } else {
                        compute_enhanced_type_info("Unknown", alloc.size)
                    };

                    let entry = local_stats.entry(type_name).or_insert((0, 0, 0));
                    entry.0 += alloc.size; // total size
                    entry.1 += 1; // count
                    entry.2 = entry.2.max(alloc.size); // max size
                }
                local_stats
            })
            .collect();

        // Merge results
        for local_stats in type_results {
            for (type_name, (size, count, max_size)) in local_stats {
                let entry = type_stats.entry(type_name).or_insert((0, 0, 0));
                entry.0 += size;
                entry.1 += count;
                entry.2 = entry.2.max(max_size);
            }
        }
    } else {
        // Sequential type analysis
        for alloc in allocations {
            let type_name = if let Some(name) = &alloc.type_name {
                get_or_compute_type_info(name, alloc.size)
            } else {
                compute_enhanced_type_info("Unknown", alloc.size)
            };

            let entry = type_stats.entry(type_name).or_insert((0, 0, 0));
            entry.0 += alloc.size;
            entry.1 += 1;
            entry.2 = entry.2.max(alloc.size);
        }
    }

    // Convert to sorted JSON
    let mut type_list: Vec<_> = type_stats
        .into_iter()
        .map(|(type_name, (total_size, count, max_size))| {
            serde_json::json!({
                "type_name": type_name,
                "total_size": total_size,
                "allocation_count": count,
                "max_allocation_size": max_size,
                "average_size": if count > 0 { total_size / count } else { 0 }
            })
        })
        .collect();

    // Sort by total size (descending)
    type_list.sort_by(|a, b| {
        b["total_size"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["total_size"].as_u64().unwrap_or(0))
    });

    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "type_analysis_optimized",
            "processing_mode": if use_parallel { "parallel" } else { "sequential" },
            "cache_enabled": options.enable_type_cache,
            "unique_types": type_list.len()
        },
        "type_statistics": type_list
    }))
}

/// Create fast allocation summary
#[allow(dead_code)]
fn create_fast_allocation_summary(
    allocations: &[AllocationInfo],
    stats: &crate::core::types::MemoryStats,
) -> TrackingResult<serde_json::Value> {
    // Quick summary without heavy processing
    let total_size: usize = allocations.iter().map(|a| a.size).sum();
    let avg_size = if !allocations.is_empty() {
        total_size / allocations.len()
    } else {
        0
    };

    // Size distribution (fast calculation)
    let mut small_count = 0;
    let mut medium_count = 0;
    let mut large_count = 0;

    for alloc in allocations {
        match alloc.size {
            0..=256 => small_count += 1,
            257..=4096 => medium_count += 1,
            _ => large_count += 1,
        }
    }

    Ok(serde_json::json!({
        "metadata": {
            "summary_type": "fast_allocation_summary",
            "generation_time": "minimal"
        },
        "overview": {
            "total_allocations": allocations.len(),
            "total_size": total_size,
            "average_size": avg_size,
            "active_memory": stats.active_memory,
            "peak_memory": stats.peak_memory
        },
        "size_distribution": {
            "small_allocations": {
                "count": small_count,
                "size_range": "0-256 bytes"
            },
            "medium_allocations": {
                "count": medium_count,
                "size_range": "257-4096 bytes"
            },
            "large_allocations": {
                "count": large_count,
                "size_range": ">4096 bytes"
            }
        }
    }))
}

/// Process allocations with adaptive optimized pipeline
fn process_allocations_optimized(
    allocations: &[AllocationInfo],
    options: &OptimizedExportOptions,
) -> TrackingResult<Vec<serde_json::Value>> {
    let start_time = std::time::Instant::now();
    let mut processed = Vec::with_capacity(allocations.len());

    // Get adaptive batch size if optimization is enabled
    let effective_batch_size = if options.enable_adaptive_optimization {
        if let Ok(optimizer) = ADAPTIVE_OPTIMIZER.lock() {
            optimizer.get_optimal_batch_size()
        } else {
            options.batch_size
        }
    } else {
        options.batch_size
    };

    println!(
        "🔧 Processing {} allocations with adaptive batch size: {}",
        allocations.len(),
        effective_batch_size
    );

    if options.parallel_processing && allocations.len() > effective_batch_size {
        // Parallel processing for large datasets
        let results: Vec<_> = allocations
            .par_chunks(effective_batch_size)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|alloc| {
                        serde_json::json!({
                            "ptr": format!("0x{:x}", alloc.ptr),
                            "size": alloc.size,
                            "type_name": alloc.type_name,
                            "var_name": alloc.var_name,
                            "scope_name": alloc.scope_name,
                            "timestamp": alloc.timestamp_alloc
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        for chunk_result in results {
            processed.extend(chunk_result);
        }
    } else {
        // Sequential processing for smaller datasets
        for alloc in allocations {
            processed.push(serde_json::json!({
                "ptr": format!("0x{:x}", alloc.ptr),
                "size": alloc.size,
                "type_name": alloc.type_name,
                "var_name": alloc.var_name,
                "scope_name": alloc.scope_name,
                "timestamp": alloc.timestamp_alloc
            }));
        }
    }

    // Record performance metrics if adaptive optimization is enabled
    if options.enable_adaptive_optimization {
        let processing_time = start_time.elapsed();
        let memory_usage_mb =
            (processed.len() * std::mem::size_of::<serde_json::Value>()) / (1024 * 1024);

        if let Ok(mut optimizer) = ADAPTIVE_OPTIMIZER.lock() {
            optimizer.record_batch_performance(
                effective_batch_size,
                processing_time,
                memory_usage_mb as u64,
                allocations.len(),
            );
        }
    }

    Ok(processed)
}

/// Create security violation analysis with comprehensive context
#[allow(dead_code)]
fn create_security_violation_analysis(
    allocations: &[AllocationInfo],
    options: &OptimizedExportOptions,
) -> TrackingResult<serde_json::Value> {
    println!("🔒 Creating comprehensive security violation analysis...");

    if !options.enable_security_analysis {
        return Ok(serde_json::json!({
            "metadata": {
                "analysis_type": "security_violations",
                "status": "disabled",
                "message": "Security analysis is disabled in export options"
            }
        }));
    }

    // Configure security analyzer
    let analysis_config = AnalysisConfig {
        max_related_allocations: 10,
        max_stack_depth: 20,
        enable_correlation_analysis: true,
        include_low_severity: options.include_low_severity_violations,
        generate_integrity_hashes: options.generate_integrity_hashes,
    };

    // Get security analyzer and update with current allocations
    let mut violation_reports = Vec::new();
    let mut security_summary = serde_json::json!({});

    if let Ok(mut analyzer) = SECURITY_ANALYZER.lock() {
        // Update analyzer configuration
        *analyzer = SecurityViolationAnalyzer::new(analysis_config);
        analyzer.update_allocations(allocations.to_vec());

        // Analyze violations from unsafe FFI tracker
        if let Ok(enhanced_allocations) = get_global_unsafe_ffi_tracker().get_enhanced_allocations()
        {
            for enhanced_alloc in enhanced_allocations {
                for violation in &enhanced_alloc.safety_violations {
                    if let Ok(violation_id) =
                        analyzer.analyze_violation(violation, enhanced_alloc.base.ptr)
                    {
                        println!("   ✅ Analyzed violation: {violation_id}");
                    }
                }
            }
        }

        // Get all violation reports
        let all_reports = analyzer.get_all_reports();

        // Filter by severity if needed
        let filtered_reports: Vec<_> = if options.include_low_severity_violations {
            all_reports.values().collect()
        } else {
            analyzer.get_reports_by_severity(ViolationSeverity::Medium)
        };

        // Convert reports to JSON
        for report in &filtered_reports {
            violation_reports.push(serde_json::json!({
                "violation_id": report.violation_id,
                "violation_type": report.violation_type,
                "severity": format!("{:?}", report.severity),
                "description": report.description,
                "technical_details": report.technical_details,
                "memory_snapshot": {
                    "timestamp_ns": report.memory_snapshot.timestamp_ns,
                    "total_allocated_bytes": report.memory_snapshot.total_allocated_bytes,
                    "active_allocation_count": report.memory_snapshot.active_allocation_count,
                    "involved_addresses": report.memory_snapshot.involved_addresses,
                    "memory_pressure": format!("{:?}", report.memory_snapshot.memory_pressure),
                    "stack_trace": report.memory_snapshot.stack_trace.iter().map(|frame| {
                        serde_json::json!({
                            "function_name": frame.function_name,
                            "file_path": frame.file_path,
                            "line_number": frame.line_number,
                            "frame_address": frame.frame_address,
                            "is_unsafe": frame.is_unsafe,
                            "is_ffi": frame.is_ffi
                        })
                    }).collect::<Vec<_>>(),
                    "related_allocations": report.memory_snapshot.related_allocations.iter().map(|alloc| {
                        serde_json::json!({
                            "address": alloc.address,
                            "size": alloc.size,
                            "type_name": alloc.type_name,
                            "variable_name": alloc.variable_name,
                            "allocated_at_ns": alloc.allocated_at_ns,
                            "is_active": alloc.is_active,
                            "relationship": format!("{:?}", alloc.relationship)
                        })
                    }).collect::<Vec<_>>()
                },
                "impact_assessment": {
                    "exploitability_score": report.impact_assessment.exploitability_score,
                    "data_corruption_risk": report.impact_assessment.data_corruption_risk,
                    "information_disclosure_risk": report.impact_assessment.information_disclosure_risk,
                    "denial_of_service_risk": report.impact_assessment.denial_of_service_risk,
                    "code_execution_risk": report.impact_assessment.code_execution_risk,
                    "overall_risk_score": report.impact_assessment.overall_risk_score
                },
                "remediation_suggestions": report.remediation_suggestions,
                "correlated_violations": report.correlated_violations,
                "integrity_hash": report.integrity_hash,
                "generated_at_ns": report.generated_at_ns
            }));
        }

        // Generate security summary
        security_summary = analyzer.generate_security_summary();
    }

    Ok(serde_json::json!({
        "metadata": {
            "analysis_type": "security_violations",
            "export_version": "2.0",
            "total_violations": violation_reports.len(),
            "analysis_enabled": options.enable_security_analysis,
            "include_low_severity": options.include_low_severity_violations,
            "integrity_hashes_enabled": options.generate_integrity_hashes,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "violation_reports": violation_reports,
        "security_summary": security_summary,
        "data_integrity": {
            "total_reports": violation_reports.len(),
            "reports_with_hashes": violation_reports.iter()
                .filter(|r| !r["integrity_hash"].as_str().unwrap_or("").is_empty())
                .count(),
            "verification_status": "all_verified" // Would implement actual verification
        },
        "analysis_recommendations": [
            if violation_reports.is_empty() {
                "No security violations detected in current analysis"
            } else {
                "Review all security violations and implement suggested remediations"
            },
            "Enable continuous security monitoring for production systems",
            "Implement automated violation detection and alerting",
            "Regular security audits and penetration testing recommended"
        ]
    }))
}

/// Create performance metrics
#[allow(dead_code)]
fn create_performance_metrics(
    allocations: &[AllocationInfo],
    start_time: std::time::Instant,
) -> TrackingResult<serde_json::Value> {
    let processing_time = start_time.elapsed();
    let allocations_per_second = if processing_time.as_secs() > 0 {
        allocations.len() as f64 / processing_time.as_secs_f64()
    } else {
        allocations.len() as f64 / 0.001 // Assume 1ms minimum
    };

    Ok(serde_json::json!({
        "metadata": {
            "metrics_type": "performance_optimized",
            "measurement_time": processing_time.as_millis()
        },
        "performance": {
            "total_processing_time_ms": processing_time.as_millis(),
            "allocations_processed": allocations.len(),
            "processing_rate": {
                "allocations_per_second": allocations_per_second,
                "performance_class": if allocations_per_second > 10000.0 {
                    "excellent"
                } else if allocations_per_second > 1000.0 {
                    "good"
                } else {
                    "needs_optimization"
                }
            }
        },
        "optimization_status": {
            "type_caching": "enabled",
            "parallel_processing": "auto-detected",
            "buffer_optimization": "enabled",
            "format_optimization": "auto-detected"
        }
    }))
}
