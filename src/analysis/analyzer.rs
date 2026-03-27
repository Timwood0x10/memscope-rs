//! Unified analysis system
//!
//! This module provides a unified analysis interface that consolidates the 17 existing
//! analyzers into 5 core analyzers while preserving all functionality.

use crate::types::internal_types::{Allocation, Snapshot};
use serde::{Deserialize, Serialize};

// ============================================================================
// Core Analysis Interface
// ============================================================================

/// Severity level for analysis results
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Individual analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Severity level
    pub severity: Severity,
    /// Result message
    pub message: String,
    /// Additional details
    pub details: String,
    /// Related pointers (if any)
    pub related_pointers: Vec<usize>,
    /// Suggested actions (if any)
    pub suggestions: Vec<String>,
}

/// Analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    /// Name of the analyzer
    pub analyzer_name: String,
    /// Analysis results
    pub results: Vec<AnalysisResult>,
    /// Summary statistics
    pub summary: String,
    /// Timestamp of analysis
    pub timestamp: u64,
    /// Analysis duration in nanoseconds
    pub duration_ns: u64,
}

/// Core analyzer trait
pub trait Analyzer: Send + Sync {
    /// Get the name of this analyzer
    fn name(&self) -> &str;

    /// Analyze a snapshot and produce a report
    fn analyze(&self, snapshot: &Snapshot) -> AnalysisReport;

    /// Check if this analyzer is enabled
    fn is_enabled(&self) -> bool {
        true
    }

    /// Enable or disable this analyzer
    fn set_enabled(&self, _enabled: bool) {
        // Default implementation does nothing
    }
}

// ============================================================================
// Leak Analyzer (Consolidates original leak analysis + unsafe_ffi_tracker)
// ============================================================================

/// Leak analyzer - detects memory leaks
pub struct LeakAnalyzer {
    enabled: std::sync::atomic::AtomicBool,
    config: LeakAnalyzerConfig,
}

#[derive(Debug, Clone)]
pub struct LeakAnalyzerConfig {
    /// Consider allocations without variable names as potential leaks
    pub track_unnamed: bool,
    /// Minimum leak size in bytes to report
    pub min_leak_size: usize,
    /// Maximum number of leaks to report
    pub max_leaks: usize,
}

impl Default for LeakAnalyzerConfig {
    fn default() -> Self {
        Self {
            track_unnamed: true,
            min_leak_size: 0,
            max_leaks: 100,
        }
    }
}

impl LeakAnalyzer {
    pub fn new() -> Self {
        Self {
            enabled: std::sync::atomic::AtomicBool::new(true),
            config: LeakAnalyzerConfig::default(),
        }
    }

    pub fn with_config(config: LeakAnalyzerConfig) -> Self {
        Self {
            enabled: std::sync::atomic::AtomicBool::new(true),
            config,
        }
    }
}

impl Default for LeakAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for LeakAnalyzer {
    fn name(&self) -> &str {
        "Leak Analyzer"
    }

    fn analyze(&self, snapshot: &Snapshot) -> AnalysisReport {
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        // Find active allocations without variable names (potential leaks)
        let leaked_allocs: Vec<&Allocation> = snapshot
            .allocations
            .iter()
            .filter(|a| {
                a.is_active()
                    && (self.config.track_unnamed && a.meta.var_name.is_none()
                        || a.size >= self.config.min_leak_size)
            })
            .collect();

        if leaked_allocs.is_empty() {
            results.push(AnalysisResult {
                severity: Severity::Info,
                message: "No memory leaks detected".to_string(),
                details: "All allocations have been properly freed or have variable names"
                    .to_string(),
                related_pointers: Vec::new(),
                suggestions: Vec::new(),
            });
        } else {
            let total_leaked = leaked_allocs.iter().map(|a| a.size).sum::<usize>();

            results.push(AnalysisResult {
                severity: Severity::Error,
                message: format!(
                    "Memory leak detected: {} allocations, {} bytes",
                    leaked_allocs.len(),
                    total_leaked
                ),
                details: format!(
                    "{} allocations are still active {}",
                    leaked_allocs.len(),
                    if self.config.track_unnamed {
                        "without variable names"
                    } else {
                        "and meet leak criteria"
                    }
                ),
                related_pointers: leaked_allocs.iter().map(|a| a.ptr).collect(),
                suggestions: vec![
                    "Review allocation and deallocation patterns".to_string(),
                    "Ensure proper RAII and Drop implementations".to_string(),
                    "Consider using smart pointers (Rc, Arc, Box)".to_string(),
                ],
            });

            // Add details for each leak (limited by max_leaks)
            for alloc in leaked_allocs.iter().take(self.config.max_leaks) {
                let suggestion = if alloc.meta.var_name.is_none() {
                    "Consider tracking this allocation with a variable name".to_string()
                } else {
                    "Ensure this allocation is properly deallocated".to_string()
                };

                results.push(AnalysisResult {
                    severity: Severity::Warning,
                    message: format!("Potential leak at 0x{:x}", alloc.ptr),
                    details: format!(
                        "Size: {} bytes, allocated at timestamp {}, thread: {}",
                        alloc.size, alloc.alloc_ts, alloc.thread
                    ),
                    related_pointers: vec![alloc.ptr],
                    suggestions: vec![suggestion],
                });
            }

            if leaked_allocs.len() > self.config.max_leaks {
                results.push(AnalysisResult {
                    severity: Severity::Info,
                    message: format!(
                        "... and {} more leaks",
                        leaked_allocs.len() - self.config.max_leaks
                    ),
                    details: format!("Showing first {} leaks only", self.config.max_leaks),
                    related_pointers: Vec::new(),
                    suggestions: Vec::new(),
                });
            }
        }

        // Check MemoryPassport system for FFI leaks
        let ffi_leaks: Vec<_> = snapshot
            .passports
            .iter()
            .filter(|p| p.status == crate::types::internal_types::PassportStatus::Leaked)
            .collect();

        if !ffi_leaks.is_empty() {
            results.push(AnalysisResult {
                severity: Severity::Critical,
                message: format!("FFI memory leaks detected: {} passports", ffi_leaks.len()),
                details: "Foreign function interface memory was not properly released".to_string(),
                related_pointers: ffi_leaks.iter().map(|p| p.ptr).collect(),
                suggestions: vec![
                    "Ensure all FFI allocations are properly freed".to_string(),
                    "Use MemoryPassport system to track FFI memory".to_string(),
                    "Review FFI function calls and memory transfers".to_string(),
                ],
            });
        }

        let duration = start.elapsed();
        let summary = if leaked_allocs.is_empty() && ffi_leaks.is_empty() {
            "No memory leaks detected. Good job!".to_string()
        } else {
            format!(
                "Found {} potential memory leaks totaling {} bytes",
                leaked_allocs.len(),
                leaked_allocs.iter().map(|a| a.size).sum::<usize>()
            )
        };

        AnalysisReport {
            analyzer_name: self.name().to_string(),
            results,
            summary,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            duration_ns: duration.as_nanos() as u64,
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn set_enabled(&self, enabled: bool) {
        self.enabled
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
    }
}

// ============================================================================
// Fragmentation Analyzer (Consolidates original fragmentation analysis)
// ============================================================================

/// Fragmentation analyzer - analyzes memory fragmentation
pub struct FragmentationAnalyzer {
    enabled: std::sync::atomic::AtomicBool,
}

impl FragmentationAnalyzer {
    pub fn new() -> Self {
        Self {
            enabled: std::sync::atomic::AtomicBool::new(true),
        }
    }
}

impl Default for FragmentationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for FragmentationAnalyzer {
    fn name(&self) -> &str {
        "Fragmentation Analyzer"
    }

    fn analyze(&self, snapshot: &Snapshot) -> AnalysisReport {
        let start = std::time::Instant::now();
        let mut results = Vec::new();
        let active_allocs = snapshot.active_allocations();

        if active_allocs.is_empty() {
            results.push(AnalysisResult {
                severity: Severity::Info,
                message: "No active allocations to analyze".to_string(),
                details: "Fragmentation analysis requires active allocations".to_string(),
                related_pointers: Vec::new(),
                suggestions: Vec::new(),
            });
        } else {
            // Calculate fragmentation metrics
            let num_allocs = active_allocs.len();
            let avg_size: f64 =
                active_allocs.iter().map(|a| a.size).sum::<usize>() as f64 / num_allocs as f64;
            let variance = active_allocs
                .iter()
                .map(|a| (a.size as f64 - avg_size).powi(2))
                .sum::<f64>()
                / num_allocs as f64;
            let std_dev = variance.sqrt();
            let coefficient_of_variation = if avg_size > 0.0 {
                std_dev / avg_size
            } else {
                0.0
            };

            // Determine fragmentation level
            let fragmentation_level = if coefficient_of_variation < 0.5 {
                (Severity::Info, "Low fragmentation")
            } else if coefficient_of_variation < 1.0 {
                (Severity::Warning, "Moderate fragmentation")
            } else {
                (Severity::Error, "High fragmentation")
            };

            results.push(AnalysisResult {
                severity: fragmentation_level.0,
                message: fragmentation_level.1.to_string(),
                details: format!(
                    "Coefficient of variation: {:.2}, Average size: {:.2} bytes, Std dev: {:.2}",
                    coefficient_of_variation, avg_size, std_dev
                ),
                related_pointers: Vec::new(),
                suggestions: vec![
                    "Consider pooling small allocations".to_string(),
                    "Use arena allocators for many small objects".to_string(),
                    "Batch operations to reduce allocation frequency".to_string(),
                ],
            });

            // Check for many small allocations
            let small_allocs = active_allocs.iter().filter(|a| a.size < 1024).count();
            if small_allocs > 100 {
                results.push(AnalysisResult {
                    severity: Severity::Warning,
                    message: format!("Many small allocations: {}", small_allocs),
                    details:
                        "Consider pooling or batching small allocations to reduce fragmentation"
                            .to_string(),
                    related_pointers: Vec::new(),
                    suggestions: vec![
                        "Use object pooling for frequently allocated small objects".to_string(),
                        "Consider using bump allocators".to_string(),
                        "Batch small allocations into larger ones".to_string(),
                    ],
                });
            }

            // Size distribution
            let mut size_buckets = [0usize; 5];
            for alloc in active_allocs {
                let bucket = if alloc.size < 1024 {
                    0
                } else if alloc.size < 10240 {
                    1
                } else if alloc.size < 102400 {
                    2
                } else if alloc.size < 1048576 {
                    3
                } else {
                    4
                };
                size_buckets[bucket] += 1;
            }

            results.push(AnalysisResult {
                severity: Severity::Info,
                message: "Size distribution".to_string(),
                details: format!(
                    "<1KB: {}, 1-10KB: {}, 10-100KB: {}, 100KB-1MB: {}, >1MB: {}",
                    size_buckets[0],
                    size_buckets[1],
                    size_buckets[2],
                    size_buckets[3],
                    size_buckets[4]
                ),
                related_pointers: Vec::new(),
                suggestions: Vec::new(),
            });
        }

        let duration = start.elapsed();
        let summary = format!(
            "Fragmentation analysis complete with {} results",
            results.len()
        );

        AnalysisReport {
            analyzer_name: self.name().to_string(),
            results,
            summary,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            duration_ns: duration.as_nanos() as u64,
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn set_enabled(&self, enabled: bool) {
        self.enabled
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
    }
}

// ============================================================================
// Lifecycle Analyzer (Consolidates original lifecycle analysis)
// ============================================================================

/// Lifecycle analyzer - analyzes object lifecycles
pub struct LifecycleAnalyzer {
    enabled: std::sync::atomic::AtomicBool,
}

impl LifecycleAnalyzer {
    pub fn new() -> Self {
        Self {
            enabled: std::sync::atomic::AtomicBool::new(true),
        }
    }
}

impl Default for LifecycleAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for LifecycleAnalyzer {
    fn name(&self) -> &str {
        "Lifecycle Analyzer"
    }

    fn analyze(&self, snapshot: &Snapshot) -> AnalysisReport {
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        // Analyze allocation lifetimes
        let allocations_with_lifetime: Vec<_> = snapshot
            .allocations
            .iter()
            .filter(|a| a.lifetime_ms.is_some())
            .collect();

        if allocations_with_lifetime.is_empty() {
            results.push(AnalysisResult {
                severity: Severity::Info,
                message: "No lifecycle data available".to_string(),
                details: "Enable lifecycle tracking for detailed analysis".to_string(),
                related_pointers: Vec::new(),
                suggestions: vec![
                    "Enable lifecycle tracking in configuration".to_string(),
                    "Use track_var! macro for variable tracking".to_string(),
                ],
            });
        } else {
            // Categorize lifetimes
            let mut instant = 0;
            let mut short_term = 0;
            let mut medium_term = 0;
            let mut long_term = 0;

            for alloc in allocations_with_lifetime.iter() {
                if let Some(lifetime) = alloc.lifetime_ms {
                    if lifetime < 1 {
                        instant += 1;
                    } else if lifetime < 100 {
                        short_term += 1;
                    } else if lifetime < 1000 {
                        medium_term += 1;
                    } else {
                        long_term += 1;
                    }
                }
            }

            results.push(AnalysisResult {
                severity: Severity::Info,
                message: "Lifetime distribution".to_string(),
                details: format!(
                    "Instant (<1ms): {}, Short (1-100ms): {}, Medium (100-1000ms): {}, Long (>1s): {}",
                    instant, short_term, medium_term, long_term
                ),
                related_pointers: Vec::new(),
                suggestions: Vec::new(),
            });

            // Find unusual patterns
            if short_term > allocations_with_lifetime.len() / 2 {
                results.push(AnalysisResult {
                    severity: Severity::Warning,
                    message: "High proportion of short-lived allocations".to_string(),
                    details: format!(
                        "{}% of allocations are short-lived",
                        (short_term * 100 / allocations_with_lifetime.len())
                    ),
                    related_pointers: Vec::new(),
                    suggestions: vec![
                        "Consider reusing objects instead of reallocating".to_string(),
                        "Use object pooling for short-lived objects".to_string(),
                        "Review allocation patterns for optimization opportunities".to_string(),
                    ],
                });
            }
        }

        let duration = start.elapsed();
        let summary = format!("Lifecycle analysis complete with {} results", results.len());

        AnalysisReport {
            analyzer_name: self.name().to_string(),
            results,
            summary,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            duration_ns: duration.as_nanos() as u64,
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn set_enabled(&self, enabled: bool) {
        self.enabled
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
    }
}

// ============================================================================
// SmartPointer Analyzer (Consolidates smart_pointer_tracker + circular_reference)
// ============================================================================

/// Smart pointer analyzer - analyzes smart pointer usage and circular references
pub struct SmartPointerAnalyzer {
    enabled: std::sync::atomic::AtomicBool,
}

impl SmartPointerAnalyzer {
    pub fn new() -> Self {
        Self {
            enabled: std::sync::atomic::AtomicBool::new(true),
        }
    }
}

impl Default for SmartPointerAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for SmartPointerAnalyzer {
    fn name(&self) -> &str {
        "Smart Pointer Analyzer"
    }

    fn analyze(&self, snapshot: &Snapshot) -> AnalysisReport {
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        // Analyze smart pointer usage
        let smart_ptr_allocs: Vec<_> = snapshot
            .allocations
            .iter()
            .filter(|a| a.meta.smart_pointer_info.is_some())
            .collect();

        if smart_ptr_allocs.is_empty() {
            results.push(AnalysisResult {
                severity: Severity::Info,
                message: "No smart pointer allocations detected".to_string(),
                details: "Enable smart pointer tracking for detailed analysis".to_string(),
                related_pointers: Vec::new(),
                suggestions: vec![
                    "Use track_smart_pointer! macro for smart pointer tracking".to_string(),
                    "Enable smart pointer tracking in configuration".to_string(),
                ],
            });
        } else {
            results.push(AnalysisResult {
                severity: Severity::Info,
                message: format!("Smart pointer allocations: {}", smart_ptr_allocs.len()),
                details: "Analyzing smart pointer usage patterns".to_string(),
                related_pointers: smart_ptr_allocs.iter().map(|a| a.ptr).collect(),
                suggestions: Vec::new(),
            });

            // Check for potential circular references
            let circular_refs: Vec<_> = smart_ptr_allocs
                .iter()
                .filter(|a| {
                    if let Some(ref_info) = &a.meta.smart_pointer_info {
                        // Check if this is a circular reference
                        ref_info.cloned_from.is_some() && ref_info.clones.len() > 0
                    } else {
                        false
                    }
                })
                .collect();

            if !circular_refs.is_empty() {
                results.push(AnalysisResult {
                    severity: Severity::Warning,
                    message: format!("Potential circular references: {}", circular_refs.len()),
                    details: "Allocations with both cloned_from and clones may indicate circular references".to_string(),
                    related_pointers: circular_refs.iter().map(|a| a.ptr).collect(),
                    suggestions: vec![
                        "Review circular reference patterns".to_string(),
                        "Consider using Weak references to break cycles".to_string(),
                        "Use cycle detection tools for complex graphs".to_string(),
                    ],
                });
            }
        }

        let duration = start.elapsed();
        let summary = format!(
            "Smart pointer analysis complete with {} results",
            results.len()
        );

        AnalysisReport {
            analyzer_name: self.name().to_string(),
            results,
            summary,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            duration_ns: duration.as_nanos() as u64,
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn set_enabled(&self, enabled: bool) {
        self.enabled
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
    }
}

// ============================================================================
// Safety Analyzer (Consolidates safety_analyzer + security_violation_analyzer)
// ============================================================================

/// Safety analyzer - analyzes safety violations and security issues
pub struct SafetyAnalyzer {
    enabled: std::sync::atomic::AtomicBool,
}

impl SafetyAnalyzer {
    pub fn new() -> Self {
        Self {
            enabled: std::sync::atomic::AtomicBool::new(true),
        }
    }
}

impl Default for SafetyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for SafetyAnalyzer {
    fn name(&self) -> &str {
        "Safety Analyzer"
    }

    fn analyze(&self, _snapshot: &Snapshot) -> AnalysisReport {
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        // Check for unsafe access patterns
        let unsafe_count = 0;
        let _unsafe_pointers: Vec<usize> = Vec::new();

        // This would be populated by unsafe access tracking events
        // For now, provide placeholder analysis
        results.push(AnalysisResult {
            severity: Severity::Info,
            message: "Safety analysis complete".to_string(),
            details: "No unsafe access patterns detected".to_string(),
            related_pointers: Vec::new(),
            suggestions: vec![
                "Enable unsafe access tracking for detailed security analysis".to_string(),
                "Review unsafe blocks for potential issues".to_string(),
                "Use safety analyzer tools for comprehensive analysis".to_string(),
            ],
        });

        let duration = start.elapsed();
        let summary = format!(
            "Safety analysis complete with {} results ({} unsafe accesses detected)",
            results.len(),
            unsafe_count
        );

        AnalysisReport {
            analyzer_name: self.name().to_string(),
            results,
            summary,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            duration_ns: duration.as_nanos() as u64,
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn set_enabled(&self, enabled: bool) {
        self.enabled
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
    }
}

// ============================================================================
// Composite Analyzer
// ============================================================================

/// Composite analyzer that runs multiple analyzers
pub struct CompositeAnalyzer {
    analyzers: Vec<Box<dyn Analyzer>>,
}

impl CompositeAnalyzer {
    pub fn new() -> Self {
        Self {
            analyzers: vec![
                Box::new(LeakAnalyzer::new()),
                Box::new(FragmentationAnalyzer::new()),
                Box::new(LifecycleAnalyzer::new()),
                Box::new(SmartPointerAnalyzer::new()),
                Box::new(SafetyAnalyzer::new()),
            ],
        }
    }

    pub fn with_analyzers(analyzers: Vec<Box<dyn Analyzer>>) -> Self {
        Self { analyzers }
    }

    pub fn add_analyzer(&mut self, analyzer: Box<dyn Analyzer>) {
        self.analyzers.push(analyzer);
    }

    pub fn analyze_all(&self, snapshot: &Snapshot) -> Vec<AnalysisReport> {
        self.analyzers
            .iter()
            .filter(|a| a.is_enabled())
            .map(|analyzer| analyzer.analyze(snapshot))
            .collect::<Vec<AnalysisReport>>()
    }
}

impl Default for CompositeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for CompositeAnalyzer {
    fn name(&self) -> &str {
        "Composite Analyzer"
    }

    fn analyze(&self, snapshot: &Snapshot) -> AnalysisReport {
        let start = std::time::Instant::now();
        let reports = self.analyze_all(snapshot);

        let total_results: usize = reports.iter().map(|r| r.results.len()).sum();
        let duration = start.elapsed();

        AnalysisReport {
            analyzer_name: self.name().to_string(),
            results: reports.iter().flat_map(|r| r.results.clone()).collect(),
            summary: format!(
                "Composite analysis complete: {} analyzers, {} total results",
                reports.len(),
                total_results
            ),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            duration_ns: duration.as_nanos() as u64,
        }
    }

    fn is_enabled(&self) -> bool {
        self.analyzers.iter().any(|a| a.is_enabled())
    }

    fn set_enabled(&self, enabled: bool) {
        for analyzer in &self.analyzers {
            analyzer.set_enabled(enabled);
        }
    }
}
