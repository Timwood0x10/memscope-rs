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

pub mod tracker;
pub mod aggregator;
pub mod analysis;
pub mod sampling;
pub mod api;
pub mod visualizer;
pub mod system_profiler;
pub mod enhanced_api;
pub mod platform_resources;
pub mod resource_integration;
pub mod comprehensive_export;

pub use tracker::{
    ThreadLocalTracker, 
    init_thread_tracker,
    track_allocation_lockfree,
    track_deallocation_lockfree,
    finalize_thread_tracker,
};

pub use aggregator::LockfreeAggregator;
pub use analysis::{
    LockfreeAnalysis,
    ThreadStats,
    PerformanceBottleneck,
    ThreadInteraction,
};
pub use sampling::SamplingConfig;
pub use api::{
    trace_all,
    trace_thread, 
    stop_tracing,
    is_tracking,
    memory_snapshot,
    quick_trace,
    MemorySnapshot,
};
// Platform-specific resource monitoring
pub use platform_resources::{
    PlatformResourceCollector,
    PlatformResourceMetrics,
    CpuResourceMetrics,
    GpuResourceMetrics,
    IoResourceMetrics,
    ThreadResourceMetrics,
};
pub use resource_integration::{
    IntegratedProfilingSession,
    ComprehensiveAnalysis,
    comprehensive_profile_execution,
};
pub use visualizer::{
    generate_comprehensive_html_report,
    generate_enhanced_html_report,
};
pub use comprehensive_export::{
    export_comprehensive_analysis,
    export_comprehensive_json,
    export_resource_rankings_json,
};