//! Batch processor for optimized unsafe/FFI data processing
//!
//! This module provides high-performance batch processing capabilities for
//! large datasets of unsafe and FFI memory allocations, with support for
//! parallel processing and performance monitoring.

use crate::analysis::unsafe_ffi_tracker::{
    AllocationSource, BoundaryEvent, EnhancedAllocationInfo, LibCHookInfo, MemoryPassport,
    RiskAssessment, RiskLevel,
};
use crate::core::types::{TrackingError, TrackingResult};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Configuration for batch processing operations
#[derive(Debug, Clone)]
pub struct BatchProcessorConfig {
    /// Size of each processing batch
    pub batch_size: usize,
    /// Threshold for enabling parallel processing
    pub parallel_threshold: usize,
    /// Maximum number of threads to use for parallel processing
    pub max_threads: Option<usize>,
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    /// Memory usage limit per batch (in bytes)
    pub memory_limit_per_batch: Option<usize>,
}

impl Default for BatchProcessorConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            parallel_threshold: 5000,
            max_threads: None, // Use system default
            enable_monitoring: true,
            memory_limit_per_batch: Some(64 * 1024 * 1024), // 64MB per batch
        }
    }
}

/// Performance metrics for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessingMetrics {
    /// Total number of items processed
    pub total_items: usize,
    /// Number of batches processed
    pub batch_count: usize,
    /// Total processing time in milliseconds
    pub total_processing_time_ms: u64,
    /// Average processing time per batch in milliseconds
    pub avg_batch_time_ms: f64,
    /// Peak memory usage during processing
    pub peak_memory_usage_bytes: usize,
    /// Whether parallel processing was used
    pub parallel_processing_used: bool,
    /// Number of threads used
    pub threads_used: usize,
    /// Processing throughput (items per second)
    pub throughput_items_per_sec: f64,
}

/// Processed unsafe allocation data
#[derive(Debug, Clone, Serialize)]
pub struct ProcessedUnsafeData {
    /// Total number of unsafe allocations
    pub total_allocations: usize,
    /// Total memory allocated in unsafe blocks
    pub total_memory: usize,
    /// Risk distribution across all allocations
    pub risk_distribution: RiskDistribution,
    /// Information about unsafe blocks
    pub unsafe_blocks: Vec<UnsafeBlockInfo>,
    /// Processed unsafe allocations
    pub allocations: Vec<ProcessedUnsafeAllocation>,
    /// Performance metrics for processing
    pub performance_metrics: UnsafePerformanceMetrics,
}

/// Processed FFI allocation data
#[derive(Debug, Clone, Serialize)]
pub struct ProcessedFFIData {
    /// Total number of FFI allocations
    pub total_allocations: usize,
    /// Total memory allocated through FFI
    pub total_memory: usize,
    /// Libraries involved in FFI operations
    pub libraries_involved: Vec<LibraryInfo>,
    /// Hook statistics
    pub hook_statistics: HookStatistics,
    /// Processed FFI allocations
    pub allocations: Vec<ProcessedFFIAllocation>,
    /// Performance metrics for processing
    pub performance_metrics: FFIPerformanceMetrics,
}

/// Processed boundary event data
#[derive(Debug, Clone, Serialize)]
pub struct ProcessedBoundaryData {
    /// Total number of boundary crossings
    pub total_crossings: usize,
    /// Transfer patterns analysis
    pub transfer_patterns: TransferPatterns,
    /// Risk analysis for boundary operations
    pub risk_analysis: BoundaryRiskAnalysis,
    /// Processed boundary events
    pub events: Vec<ProcessedBoundaryEvent>,
    /// Performance impact analysis
    pub performance_impact: BoundaryPerformanceImpact,
}

/// Risk distribution statistics
#[derive(Debug, Clone, Serialize)]
pub struct RiskDistribution {
    /// Number of low risk allocations
    pub low_risk: usize,
    /// Number of medium risk allocations
    pub medium_risk: usize,
    /// Number of high risk allocations
    pub high_risk: usize,
    /// Number of critical risk allocations
    pub critical_risk: usize,
    /// Overall risk score (0.0 to 10.0)
    pub overall_risk_score: f64,
}

/// Information about an unsafe block
#[derive(Debug, Clone, Serialize)]
pub struct UnsafeBlockInfo {
    /// Location of the unsafe block
    pub location: String,
    /// Number of allocations in this block
    pub allocation_count: usize,
    /// Total memory allocated in this block
    pub total_memory: usize,
    /// Risk level of this block
    pub risk_level: RiskLevel,
    /// Functions called within this block
    pub functions_called: Vec<String>,
}

/// Processed unsafe allocation
#[derive(Debug, Clone, Serialize)]
pub struct ProcessedUnsafeAllocation {
    /// Memory pointer (as hex string)
    pub ptr: String,
    /// Allocation size
    pub size: usize,
    /// Type name if available
    pub type_name: Option<String>,
    /// Unsafe block location
    pub unsafe_block_location: String,
    /// Call stack information
    pub call_stack: Vec<String>,
    /// Risk assessment
    pub risk_assessment: RiskAssessment,
    /// Lifetime information
    pub lifetime_info: LifetimeInfo,
    /// Memory layout information
    pub memory_layout: Option<MemoryLayoutInfo>,
}

/// Library information for FFI operations
#[derive(Debug, Clone, Serialize)]
pub struct LibraryInfo {
    /// Name of the library
    pub name: String,
    /// Number of allocations from this library
    pub allocation_count: usize,
    /// Total memory allocated by this library
    pub total_memory: usize,
    /// Functions used from this library
    pub functions_used: Vec<String>,
    /// Average allocation size
    pub avg_allocation_size: usize,
}

/// Hook statistics
#[derive(Debug, Clone, Serialize)]
pub struct HookStatistics {
    /// Total number of hooks installed
    pub total_hooks: usize,
    /// Hook success rate
    pub success_rate: f64,
    /// Average hook overhead in nanoseconds
    pub avg_overhead_ns: f64,
    /// Hook methods used
    pub methods_used: HashMap<String, usize>,
}

/// Processed FFI allocation
#[derive(Debug, Clone, Serialize)]
pub struct ProcessedFFIAllocation {
    /// Memory pointer (as hex string)
    pub ptr: String,
    /// Allocation size
    pub size: usize,
    /// Library name
    pub library_name: String,
    /// Function name
    pub function_name: String,
    /// Call stack information
    pub call_stack: Vec<String>,
    /// Hook information
    pub hook_info: LibCHookInfo,
    /// Ownership information
    pub ownership_info: OwnershipInfo,
    /// Interop metadata
    pub interop_metadata: InteropMetadata,
}

/// Transfer patterns analysis
#[derive(Debug, Clone, Serialize)]
pub struct TransferPatterns {
    /// Most common transfer direction
    pub dominant_direction: String,
    /// Transfer frequency by type
    pub frequency_by_type: HashMap<String, usize>,
    /// Average transfer size
    pub avg_transfer_size: usize,
    /// Peak transfer activity time
    pub peak_activity_time: Option<u128>,
}

/// Boundary risk analysis
#[derive(Debug, Clone, Serialize)]
pub struct BoundaryRiskAnalysis {
    /// Overall boundary risk score
    pub overall_risk_score: f64,
    /// High risk transfer count
    pub high_risk_transfers: usize,
    /// Common risk patterns
    pub common_risk_patterns: Vec<String>,
    /// Mitigation recommendations
    pub mitigation_recommendations: Vec<String>,
}

/// Processed boundary event
#[derive(Debug, Clone, Serialize)]
pub struct ProcessedBoundaryEvent {
    /// Event identifier
    pub event_id: String,
    /// Event type
    pub event_type: String,
    /// Timestamp
    pub timestamp: u128,
    /// Source context information
    pub from_context: ContextInfo,
    /// Destination context information
    pub to_context: ContextInfo,
    /// Memory passport information
    pub memory_passport: Option<MemoryPassport>,
    /// Risk factors
    pub risk_factors: Vec<String>,
}

/// Context information for boundary events
#[derive(Debug, Clone, Serialize)]
pub struct ContextInfo {
    /// Context name (Rust/FFI)
    pub name: String,
    /// Function or module name
    pub function: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Performance metrics for unsafe operations
#[derive(Debug, Clone, Serialize)]
pub struct UnsafePerformanceMetrics {
    /// Processing time for unsafe analysis
    pub processing_time_ms: u64,
    /// Memory usage during processing
    pub memory_usage_bytes: usize,
    /// Number of risk assessments performed
    pub risk_assessments_performed: usize,
    /// Average risk assessment time
    pub avg_risk_assessment_time_ns: f64,
}

/// Performance metrics for FFI operations
#[derive(Debug, Clone, Serialize)]
pub struct FFIPerformanceMetrics {
    /// Processing time for FFI analysis
    pub processing_time_ms: u64,
    /// Memory usage during processing
    pub memory_usage_bytes: usize,
    /// Number of hook operations processed
    pub hook_operations_processed: usize,
    /// Average hook processing time
    pub avg_hook_processing_time_ns: f64,
}

/// Performance impact of boundary operations
#[derive(Debug, Clone, Serialize)]
pub struct BoundaryPerformanceImpact {
    /// Total boundary processing time
    pub total_processing_time_ms: u64,
    /// Average time per boundary crossing
    pub avg_crossing_time_ns: f64,
    /// Performance overhead percentage
    pub overhead_percentage: f64,
    /// Optimization opportunities
    pub optimization_opportunities: Vec<String>,
}

/// Lifetime information for allocations
#[derive(Debug, Clone, Serialize)]
pub struct LifetimeInfo {
    /// Allocation timestamp
    pub allocated_at: u128,
    /// Deallocation timestamp (if deallocated)
    pub deallocated_at: Option<u128>,
    /// Lifetime duration in nanoseconds
    pub lifetime_ns: Option<u64>,
    /// Scope information
    pub scope: String,
}

/// Memory layout information
#[derive(Debug, Clone, Serialize)]
pub struct MemoryLayoutInfo {
    /// Total size of the allocation
    pub total_size: usize,
    /// Memory alignment
    pub alignment: usize,
    /// Padding information
    pub padding_bytes: usize,
    /// Layout efficiency score
    pub efficiency_score: f64,
}

/// Ownership information for FFI allocations
#[derive(Debug, Clone, Serialize)]
pub struct OwnershipInfo {
    /// Current owner context
    pub owner_context: String,
    /// Owner function
    pub owner_function: String,
    /// Transfer timestamp
    pub transfer_timestamp: u128,
    /// Expected lifetime
    pub expected_lifetime: Option<u128>,
}

/// Interop metadata for FFI operations
#[derive(Debug, Clone, Serialize)]
pub struct InteropMetadata {
    /// Data marshalling information
    pub marshalling_info: String,
    /// Type conversion details
    pub type_conversion: String,
    /// Performance impact
    pub performance_impact: String,
    /// Safety considerations
    pub safety_considerations: Vec<String>,
}

/// High-performance batch processor for unsafe/FFI data
pub struct BatchProcessor {
    /// Configuration for batch processing
    config: BatchProcessorConfig,
    /// Performance monitoring data
    metrics: Arc<Mutex<BatchProcessingMetrics>>,
    /// Thread pool for parallel processing
    thread_pool: Option<rayon::ThreadPool>,
}

impl BatchProcessor {
    /// Create a new batch processor with default configuration
    pub fn new() -> Self {
        Self::with_config(BatchProcessorConfig::default())
    }

    /// Create a new batch processor with custom configuration
    pub fn with_config(config: BatchProcessorConfig) -> Self {
        let thread_pool = config.max_threads.map(|max_threads| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(max_threads)
                .build()
                .expect("Failed to create thread pool")
        });

        let metrics = Arc::new(Mutex::new(BatchProcessingMetrics {
            total_items: 0,
            batch_count: 0,
            total_processing_time_ms: 0,
            avg_batch_time_ms: 0.0,
            peak_memory_usage_bytes: 0,
            parallel_processing_used: false,
            threads_used: 1,
            throughput_items_per_sec: 0.0,
        }));

        Self {
            config,
            metrics,
            thread_pool,
        }
    }

    /// Process unsafe allocations in batches
    pub fn process_unsafe_allocations(
        &self,
        allocations: &[EnhancedAllocationInfo],
    ) -> TrackingResult<ProcessedUnsafeData> {
        let start_time = Instant::now();
        let use_parallel = allocations.len() >= self.config.parallel_threshold;

        // Update metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.total_items = allocations.len();
            metrics.parallel_processing_used = use_parallel;
            metrics.threads_used = if use_parallel {
                self.thread_pool
                    .as_ref()
                    .map(|p| p.current_num_threads())
                    .unwrap_or_else(rayon::current_num_threads)
            } else {
                1
            };
        }

        let processed_allocations = if use_parallel {
            self.process_unsafe_parallel(allocations)?
        } else {
            self.process_unsafe_sequential(allocations)?
        };

        let processing_time = start_time.elapsed();

        // Calculate statistics
        let total_memory: usize = processed_allocations.iter().map(|a| a.size).sum();
        let risk_distribution = self.calculate_risk_distribution(&processed_allocations);
        let unsafe_blocks = self.analyze_unsafe_blocks(&processed_allocations);

        let performance_metrics = UnsafePerformanceMetrics {
            processing_time_ms: processing_time.as_millis() as u64,
            memory_usage_bytes: self.estimate_memory_usage(allocations.len()),
            risk_assessments_performed: processed_allocations.len(),
            avg_risk_assessment_time_ns: if processed_allocations.is_empty() {
                0.0
            } else {
                processing_time.as_nanos() as f64 / processed_allocations.len() as f64
            },
        };

        // Update final metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.total_processing_time_ms = processing_time.as_millis() as u64;
            metrics.avg_batch_time_ms = processing_time.as_millis() as f64
                / ((allocations.len() / self.config.batch_size).max(1)) as f64;
            metrics.throughput_items_per_sec = if processing_time.as_secs_f64() > 0.0 {
                allocations.len() as f64 / processing_time.as_secs_f64()
            } else {
                0.0
            };
        }

        Ok(ProcessedUnsafeData {
            total_allocations: allocations.len(),
            total_memory,
            risk_distribution,
            unsafe_blocks,
            allocations: processed_allocations,
            performance_metrics,
        })
    }

    /// Process FFI allocations in batches
    pub fn process_ffi_allocations(
        &self,
        allocations: &[EnhancedAllocationInfo],
    ) -> TrackingResult<ProcessedFFIData> {
        let start_time = Instant::now();
        let use_parallel = allocations.len() >= self.config.parallel_threshold;

        let processed_allocations = if use_parallel {
            self.process_ffi_parallel(allocations)?
        } else {
            self.process_ffi_sequential(allocations)?
        };

        let processing_time = start_time.elapsed();

        // Calculate statistics
        let total_memory: usize = processed_allocations.iter().map(|a| a.size).sum();
        let libraries_involved = self.analyze_libraries(&processed_allocations);
        let hook_statistics = self.calculate_hook_statistics(&processed_allocations);

        let performance_metrics = FFIPerformanceMetrics {
            processing_time_ms: processing_time.as_millis() as u64,
            memory_usage_bytes: self.estimate_memory_usage(allocations.len()),
            hook_operations_processed: processed_allocations.len(),
            avg_hook_processing_time_ns: if processed_allocations.is_empty() {
                0.0
            } else {
                processing_time.as_nanos() as f64 / processed_allocations.len() as f64
            },
        };

        Ok(ProcessedFFIData {
            total_allocations: allocations.len(),
            total_memory,
            libraries_involved,
            hook_statistics,
            allocations: processed_allocations,
            performance_metrics,
        })
    }

    /// Process boundary events in batches
    pub fn process_boundary_events(
        &self,
        allocations: &[EnhancedAllocationInfo],
    ) -> TrackingResult<ProcessedBoundaryData> {
        let start_time = Instant::now();

        // Extract all boundary events from allocations
        let mut all_events = Vec::new();
        for allocation in allocations {
            for event in &allocation.cross_boundary_events {
                all_events.push((allocation, event));
            }
        }

        let use_parallel = all_events.len() >= self.config.parallel_threshold;

        let processed_events = if use_parallel {
            self.process_boundary_parallel(&all_events)?
        } else {
            self.process_boundary_sequential(&all_events)?
        };

        let processing_time = start_time.elapsed();

        // Calculate statistics
        let transfer_patterns = self.analyze_transfer_patterns(&processed_events);
        let risk_analysis = self.analyze_boundary_risks(&processed_events);

        let performance_impact = BoundaryPerformanceImpact {
            total_processing_time_ms: processing_time.as_millis() as u64,
            avg_crossing_time_ns: if processed_events.is_empty() {
                0.0
            } else {
                processing_time.as_nanos() as f64 / processed_events.len() as f64
            },
            overhead_percentage: 5.0, // Estimated overhead
            optimization_opportunities: vec![
                "Reduce boundary crossings".to_string(),
                "Batch transfer operations".to_string(),
            ],
        };

        Ok(ProcessedBoundaryData {
            total_crossings: processed_events.len(),
            transfer_patterns,
            risk_analysis,
            events: processed_events,
            performance_impact,
        })
    }

    /// Get current processing metrics
    pub fn get_metrics(&self) -> TrackingResult<BatchProcessingMetrics> {
        self.metrics
            .lock()
            .map(|m| m.clone())
            .map_err(|e| TrackingError::LockError(e.to_string()))
    }

    /// Reset processing metrics
    pub fn reset_metrics(&self) -> TrackingResult<()> {
        if let Ok(mut metrics) = self.metrics.lock() {
            *metrics = BatchProcessingMetrics {
                total_items: 0,
                batch_count: 0,
                total_processing_time_ms: 0,
                avg_batch_time_ms: 0.0,
                peak_memory_usage_bytes: 0,
                parallel_processing_used: false,
                threads_used: 1,
                throughput_items_per_sec: 0.0,
            };
        }
        Ok(())
    }
}
impl BatchProcessor {
    /// Process unsafe allocations sequentially
    fn process_unsafe_sequential(
        &self,
        allocations: &[EnhancedAllocationInfo],
    ) -> TrackingResult<Vec<ProcessedUnsafeAllocation>> {
        let mut processed = Vec::with_capacity(allocations.len());

        for allocation in allocations {
            if let AllocationSource::UnsafeRust {
                unsafe_block_location,
                call_stack,
                risk_assessment,
            } = &allocation.source
            {
                processed.push(ProcessedUnsafeAllocation {
                    ptr: format!("0x{:x}", allocation.base.ptr),
                    size: allocation.base.size,
                    type_name: allocation.base.type_name.clone(),
                    unsafe_block_location: unsafe_block_location.clone(),
                    call_stack: call_stack
                        .get_frames()
                        .unwrap_or_default()
                        .iter()
                        .map(|f| f.function_name.clone())
                        .collect(),
                    risk_assessment: risk_assessment.clone(),
                    lifetime_info: LifetimeInfo {
                        allocated_at: allocation.base.timestamp_alloc as u128,
                        deallocated_at: allocation.base.timestamp_dealloc.map(|t| t as u128),
                        lifetime_ns: allocation
                            .base
                            .timestamp_dealloc
                            .map(|dealloc| (dealloc - allocation.base.timestamp_alloc) * 1_000_000),
                        scope: allocation
                            .base
                            .scope_name
                            .clone()
                            .unwrap_or_else(|| "unknown".to_string()),
                    },
                    memory_layout: Some(MemoryLayoutInfo {
                        total_size: allocation.base.size,
                        alignment: 8,          // Default alignment
                        padding_bytes: 0,      // Simplified
                        efficiency_score: 0.9, // Estimated
                    }),
                });
            }
        }

        Ok(processed)
    }

    /// Process unsafe allocations in parallel
    fn process_unsafe_parallel(
        &self,
        allocations: &[EnhancedAllocationInfo],
    ) -> TrackingResult<Vec<ProcessedUnsafeAllocation>> {
        let processed: Result<Vec<_>, TrackingError> = if let Some(pool) = &self.thread_pool {
            pool.install(|| {
                allocations
                    .par_chunks(self.config.batch_size)
                    .map(|chunk| self.process_unsafe_sequential(chunk))
                    .collect::<Result<Vec<_>, _>>()
                    .map(|batches| batches.into_iter().flatten().collect())
            })
        } else {
            allocations
                .par_chunks(self.config.batch_size)
                .map(|chunk| self.process_unsafe_sequential(chunk))
                .collect::<Result<Vec<_>, _>>()
                .map(|batches| batches.into_iter().flatten().collect())
        };

        processed
    }

    /// Process FFI allocations sequentially
    fn process_ffi_sequential(
        &self,
        allocations: &[EnhancedAllocationInfo],
    ) -> TrackingResult<Vec<ProcessedFFIAllocation>> {
        let mut processed = Vec::with_capacity(allocations.len());

        for allocation in allocations {
            if let AllocationSource::FfiC {
                resolved_function,
                call_stack,
                libc_hook_info,
            } = &allocation.source
            {
                processed.push(ProcessedFFIAllocation {
                    ptr: format!("0x{:x}", allocation.base.ptr),
                    size: allocation.base.size,
                    library_name: resolved_function.library_name.clone(),
                    function_name: resolved_function.function_name.clone(),
                    call_stack: call_stack
                        .get_frames()
                        .unwrap_or_default()
                        .iter()
                        .map(|f| f.function_name.clone())
                        .collect(),
                    hook_info: libc_hook_info.clone(),
                    ownership_info: OwnershipInfo {
                        owner_context: "FFI".to_string(),
                        owner_function: resolved_function.function_name.clone(),
                        transfer_timestamp: allocation.base.timestamp_alloc as u128,
                        expected_lifetime: None,
                    },
                    interop_metadata: InteropMetadata {
                        marshalling_info: "C-compatible".to_string(),
                        type_conversion: "Direct".to_string(),
                        performance_impact: "Low".to_string(),
                        safety_considerations: vec![
                            "Manual memory management required".to_string(),
                            "Potential for memory leaks".to_string(),
                        ],
                    },
                });
            }
        }

        Ok(processed)
    }

    /// Process FFI allocations in parallel
    fn process_ffi_parallel(
        &self,
        allocations: &[EnhancedAllocationInfo],
    ) -> TrackingResult<Vec<ProcessedFFIAllocation>> {
        let processed: Result<Vec<_>, TrackingError> = if let Some(pool) = &self.thread_pool {
            pool.install(|| {
                allocations
                    .par_chunks(self.config.batch_size)
                    .map(|chunk| self.process_ffi_sequential(chunk))
                    .collect::<Result<Vec<_>, _>>()
                    .map(|batches| batches.into_iter().flatten().collect())
            })
        } else {
            allocations
                .par_chunks(self.config.batch_size)
                .map(|chunk| self.process_ffi_sequential(chunk))
                .collect::<Result<Vec<_>, _>>()
                .map(|batches| batches.into_iter().flatten().collect())
        };

        processed
    }

    /// Process boundary events sequentially
    fn process_boundary_sequential(
        &self,
        events: &[(&EnhancedAllocationInfo, &BoundaryEvent)],
    ) -> TrackingResult<Vec<ProcessedBoundaryEvent>> {
        let mut processed = Vec::with_capacity(events.len());

        for (allocation, event) in events {
            processed.push(ProcessedBoundaryEvent {
                event_id: format!("boundary_{:x}_{}", allocation.base.ptr, event.timestamp),
                event_type: format!("{:?}", event.event_type),
                timestamp: event.timestamp,
                from_context: ContextInfo {
                    name: event.from_context.clone(),
                    function: "unknown".to_string(),
                    metadata: HashMap::new(),
                },
                to_context: ContextInfo {
                    name: event.to_context.clone(),
                    function: "unknown".to_string(),
                    metadata: HashMap::new(),
                },
                memory_passport: allocation.memory_passport.clone(),
                risk_factors: vec!["Cross-boundary transfer".to_string()],
            });
        }

        Ok(processed)
    }

    /// Process boundary events in parallel
    fn process_boundary_parallel(
        &self,
        events: &[(&EnhancedAllocationInfo, &BoundaryEvent)],
    ) -> TrackingResult<Vec<ProcessedBoundaryEvent>> {
        let processed: Result<Vec<_>, TrackingError> = if let Some(pool) = &self.thread_pool {
            pool.install(|| {
                events
                    .par_chunks(self.config.batch_size)
                    .map(|chunk| self.process_boundary_sequential(chunk))
                    .collect::<Result<Vec<_>, _>>()
                    .map(|batches| batches.into_iter().flatten().collect())
            })
        } else {
            events
                .par_chunks(self.config.batch_size)
                .map(|chunk| self.process_boundary_sequential(chunk))
                .collect::<Result<Vec<_>, _>>()
                .map(|batches| batches.into_iter().flatten().collect())
        };

        processed
    }

    /// Calculate risk distribution from processed allocations
    fn calculate_risk_distribution(
        &self,
        allocations: &[ProcessedUnsafeAllocation],
    ) -> RiskDistribution {
        let mut low_risk = 0;
        let mut medium_risk = 0;
        let mut high_risk = 0;
        let mut critical_risk = 0;
        let mut total_risk_score = 0.0;

        for allocation in allocations {
            match allocation.risk_assessment.risk_level {
                RiskLevel::Low => low_risk += 1,
                RiskLevel::Medium => medium_risk += 1,
                RiskLevel::High => high_risk += 1,
                RiskLevel::Critical => critical_risk += 1,
            }

            // Calculate risk score based on level
            let risk_score = match allocation.risk_assessment.risk_level {
                RiskLevel::Low => 2.0,
                RiskLevel::Medium => 5.0,
                RiskLevel::High => 8.0,
                RiskLevel::Critical => 10.0,
            };
            total_risk_score += risk_score;
        }

        let overall_risk_score = if allocations.is_empty() {
            0.0
        } else {
            total_risk_score / allocations.len() as f64
        };

        RiskDistribution {
            low_risk,
            medium_risk,
            high_risk,
            critical_risk,
            overall_risk_score,
        }
    }

    /// Analyze unsafe blocks from processed allocations
    fn analyze_unsafe_blocks(
        &self,
        allocations: &[ProcessedUnsafeAllocation],
    ) -> Vec<UnsafeBlockInfo> {
        let mut blocks: HashMap<String, UnsafeBlockInfo> = HashMap::new();

        for allocation in allocations {
            let entry = blocks
                .entry(allocation.unsafe_block_location.clone())
                .or_insert_with(|| UnsafeBlockInfo {
                    location: allocation.unsafe_block_location.clone(),
                    allocation_count: 0,
                    total_memory: 0,
                    risk_level: RiskLevel::Low,
                    functions_called: Vec::new(),
                });

            entry.allocation_count += 1;
            entry.total_memory += allocation.size;

            // Update risk level to highest found
            if matches!(allocation.risk_assessment.risk_level, RiskLevel::Critical) {
                entry.risk_level = RiskLevel::Critical;
            } else if matches!(allocation.risk_assessment.risk_level, RiskLevel::High)
                && !matches!(entry.risk_level, RiskLevel::Critical)
            {
                entry.risk_level = RiskLevel::High;
            } else if matches!(allocation.risk_assessment.risk_level, RiskLevel::Medium)
                && matches!(entry.risk_level, RiskLevel::Low)
            {
                entry.risk_level = RiskLevel::Medium;
            }

            // Add unique functions from call stack
            for func in &allocation.call_stack {
                if !entry.functions_called.contains(func) {
                    entry.functions_called.push(func.clone());
                }
            }
        }

        blocks.into_values().collect()
    }

    /// Analyze libraries from processed FFI allocations
    fn analyze_libraries(&self, allocations: &[ProcessedFFIAllocation]) -> Vec<LibraryInfo> {
        let mut libraries: HashMap<String, LibraryInfo> = HashMap::new();

        for allocation in allocations {
            let entry = libraries
                .entry(allocation.library_name.clone())
                .or_insert_with(|| LibraryInfo {
                    name: allocation.library_name.clone(),
                    allocation_count: 0,
                    total_memory: 0,
                    functions_used: Vec::new(),
                    avg_allocation_size: 0,
                });

            entry.allocation_count += 1;
            entry.total_memory += allocation.size;

            if !entry.functions_used.contains(&allocation.function_name) {
                entry.functions_used.push(allocation.function_name.clone());
            }
        }

        // Calculate average allocation sizes
        for library in libraries.values_mut() {
            library.avg_allocation_size = if library.allocation_count > 0 {
                library.total_memory / library.allocation_count
            } else {
                0
            };
        }

        libraries.into_values().collect()
    }

    /// Calculate hook statistics from processed FFI allocations
    fn calculate_hook_statistics(&self, allocations: &[ProcessedFFIAllocation]) -> HookStatistics {
        let mut methods_used = HashMap::new();
        let mut total_overhead = 0.0;
        let mut overhead_count = 0;

        for allocation in allocations {
            let method_name = format!("{:?}", allocation.hook_info.hook_method);
            *methods_used.entry(method_name).or_insert(0) += 1;

            if let Some(overhead) = allocation.hook_info.hook_overhead_ns {
                total_overhead += overhead as f64;
                overhead_count += 1;
            }
        }

        let avg_overhead_ns = if overhead_count > 0 {
            total_overhead / overhead_count as f64
        } else {
            0.0
        };

        HookStatistics {
            total_hooks: allocations.len(),
            success_rate: 0.95, // Estimated success rate
            avg_overhead_ns,
            methods_used,
        }
    }

    /// Analyze transfer patterns from boundary events
    fn analyze_transfer_patterns(&self, events: &[ProcessedBoundaryEvent]) -> TransferPatterns {
        let mut frequency_by_type = HashMap::new();
        let mut total_size = 0;
        let mut rust_to_ffi = 0;
        let mut ffi_to_rust = 0;

        for event in events {
            *frequency_by_type
                .entry(event.event_type.clone())
                .or_insert(0) += 1;

            if event.from_context.name.contains("Rust") && event.to_context.name.contains("FFI") {
                rust_to_ffi += 1;
            } else if event.from_context.name.contains("FFI")
                && event.to_context.name.contains("Rust")
            {
                ffi_to_rust += 1;
            }

            // Estimate transfer size (simplified)
            total_size += 64; // Average estimated size
        }

        let dominant_direction = if rust_to_ffi > ffi_to_rust {
            "Rust -> FFI".to_string()
        } else if ffi_to_rust > rust_to_ffi {
            "FFI -> Rust".to_string()
        } else {
            "Balanced".to_string()
        };

        let avg_transfer_size = if events.is_empty() {
            0
        } else {
            total_size / events.len()
        };

        TransferPatterns {
            dominant_direction,
            frequency_by_type,
            avg_transfer_size,
            peak_activity_time: None, // Could be calculated from timestamps
        }
    }

    /// Analyze boundary risks from processed events
    fn analyze_boundary_risks(&self, events: &[ProcessedBoundaryEvent]) -> BoundaryRiskAnalysis {
        let high_risk_transfers = events.iter().filter(|e| e.risk_factors.len() > 1).count();

        let overall_risk_score = if events.is_empty() {
            0.0
        } else {
            (high_risk_transfers as f64 / events.len() as f64) * 10.0
        };

        BoundaryRiskAnalysis {
            overall_risk_score,
            high_risk_transfers,
            common_risk_patterns: vec![
                "Unvalidated pointer transfer".to_string(),
                "Size mismatch potential".to_string(),
                "Ownership ambiguity".to_string(),
            ],
            mitigation_recommendations: vec![
                "Implement pointer validation".to_string(),
                "Add size checks at boundaries".to_string(),
                "Clarify ownership semantics".to_string(),
            ],
        }
    }

    /// Estimate memory usage for processing
    fn estimate_memory_usage(&self, item_count: usize) -> usize {
        // Rough estimate: 1KB per item for processing overhead
        item_count * 1024
    }
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new()
    }
}
