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