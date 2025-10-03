//! Lock-free multi-threaded memory tracking implementation
//!
//! This module provides a completely separate implementation optimized for
//! high-concurrency scenarios (100+ threads) without any shared state or locks.
//!
//! Key differences from the single-threaded version:
//! - Thread-local tracking with zero shared state
//! - Intelligent sampling for performance
//! - Binary file format for efficiency
//! - Offline aggregation and analysis
//!
//! Use this version when:
//! - High thread concurrency (20+ threads)
//! - Performance is critical
//! - Approximate tracking is acceptable
//!
//! Use the main version when:
//! - Single-threaded or low concurrency
//! - Exact precision is required
//! - Real-time analysis is needed

pub mod aggregator;
pub mod analysis;
pub mod api;
pub mod comprehensive_export;
pub mod enhanced_api;
pub mod platform_resources;
pub mod resource_integration;
pub mod sampling;
pub mod system_profiler;
pub mod tracker;
pub mod visualizer;

pub use tracker::{
    finalize_thread_tracker, init_thread_tracker, track_allocation_lockfree,
    track_deallocation_lockfree, ThreadLocalTracker,
};

pub use aggregator::LockfreeAggregator;
pub use analysis::{LockfreeAnalysis, PerformanceBottleneck, ThreadInteraction, ThreadStats};
pub use api::{
    is_tracking, memory_snapshot, quick_trace, stop_tracing, trace_all, trace_thread,
    MemorySnapshot,
};
pub use sampling::SamplingConfig;
// Platform-specific resource monitoring
pub use comprehensive_export::{
    export_comprehensive_analysis, export_comprehensive_json, export_resource_rankings_json,
};
pub use platform_resources::{
    CpuResourceMetrics, GpuResourceMetrics, IoResourceMetrics, PlatformResourceCollector,
    PlatformResourceMetrics, ThreadResourceMetrics,
};
pub use resource_integration::{
    comprehensive_profile_execution, ComprehensiveAnalysis, IntegratedProfilingSession,
};
pub use visualizer::generate_comprehensive_html_report;
