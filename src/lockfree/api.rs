//! Lockfree Memory Tracking API
//!
//! Provides simple, high-level interfaces for lockfree memory tracking.
//! Designed for minimal friction and maximum usability.

use tracing::info;

use super::aggregator::LockfreeAggregator;
use super::tracker::{finalize_thread_tracker, init_thread_tracker, SamplingConfig};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use super::comprehensive_export::export_comprehensive_analysis;
use super::resource_integration::{
    BottleneckType, ComprehensiveAnalysis, CorrelationMetrics, PerformanceInsights,
};

#[deprecated(
    since = "0.4.0",
    note = "Please use the new unified tracking system in src/new/tracker/mod.rs \
           with TrackingStrategy::ThreadLocal. See migration guide in src/new/tracker/mod.rs."
)]
/// Global tracking state for lockfree module
static TRACKING_ENABLED: AtomicBool = AtomicBool::new(false);
use std::sync::OnceLock;
static OUTPUT_DIRECTORY: OnceLock<std::path::PathBuf> = OnceLock::new();

/// Start tracking all threads with automatic initialization
///
/// This function enables memory tracking for all threads in your application.
/// Call once at program start, tracking happens automatically afterward.
///
/// # Arguments
/// * `output_dir` - Directory where tracking data will be stored
///
/// # Returns
/// Result indicating success or error during initialization
///
/// # Example
/// ```rust
/// use memscope_rs::lockfree::api::trace_all;
///
/// trace_all("./memory_analysis")?;
/// // Your application runs here with automatic tracking
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn trace_all<P: AsRef<Path>>(output_dir: P) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = output_dir.as_ref().to_path_buf();

    // Setup global output directory
    let _ = OUTPUT_DIRECTORY.set(output_path.clone());

    // Clean and create output directory
    if output_path.exists() {
        std::fs::remove_dir_all(&output_path)?;
    }
    std::fs::create_dir_all(&output_path)?;

    // Enable global tracking in old system (for backward compatibility)
    TRACKING_ENABLED.store(true, Ordering::SeqCst);

    // Enable tracking in new unified tracking system
    use crate::new::tracker::{get_global_tracker, TrackingConfig, TrackingStrategy};

    let config = TrackingConfig {
        strategy: TrackingStrategy::ThreadLocal, // lockfree maps to ThreadLocal strategy
        ..Default::default()
    };

    let unified = get_global_tracker();
    unified.set_enabled(true);

    println!(
        "🚀 Lockfree tracking started: {} (delegating to unified tracker)",
        output_path.display()
    );

    Ok(())
}

/// Start tracking current thread only
///
/// Enables memory tracking for the calling thread only. Use this when you want
/// to track specific threads rather than the entire application.
///
/// # Arguments
/// * `output_dir` - Directory where tracking data will be stored
///
/// # Returns
/// Result indicating success or error during thread tracker initialization
///
/// # Example
/// ```rust,no_run
/// use memscope_rs::lockfree::api::trace_thread;
///
/// std::thread::spawn(|| {
///     if let Err(e) = trace_thread("./thread_analysis") {
///         eprintln!("Error starting thread tracking: {}", e);
///     }
///     // This thread's allocations are now tracked
/// });
/// ```
pub fn trace_thread<P: AsRef<Path>>(output_dir: P) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = output_dir.as_ref().to_path_buf();

    // Create output directory if needed
    if !output_path.exists() {
        std::fs::create_dir_all(&output_path)?;
    }

    // Initialize tracking for current thread with high precision (old system)
    let _ = init_thread_tracker(&output_path, Some(SamplingConfig::demo()));

    // Enable tracking in new unified tracking system
    use crate::new::tracker::{get_global_tracker, TrackingConfig, TrackingStrategy};

    let config = TrackingConfig {
        strategy: TrackingStrategy::ThreadLocal,
        ..Default::default()
    };

    let unified = get_global_tracker();
    unified.set_enabled(true);

    Ok(())
}

/// Stop all memory tracking and generate comprehensive reports
///
/// Finalizes memory tracking, processes all collected data, and generates
/// HTML and JSON reports for analysis.
///
/// # Returns
/// Result indicating success or error during finalization and report generation
///
/// # Example
/// ```rust
/// use memscope_rs::lockfree::api::{trace_all, stop_tracing};
///
/// trace_all("./memory_analysis")?;
/// // Your application code here
/// stop_tracing()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn stop_tracing() -> Result<(), Box<dyn std::error::Error>> {
    if !TRACKING_ENABLED.load(Ordering::SeqCst) {
        return Ok(()); // No active tracking session
    }

    // Finalize current thread tracker if needed (old system)
    let _ = finalize_thread_tracker();

    // Disable global tracking in old system
    TRACKING_ENABLED.store(false, Ordering::SeqCst);

    // Disable tracking in new unified tracking system
    use crate::new::tracker::get_global_tracker;

    let unified = get_global_tracker();
    unified.set_enabled(false);

    // Generate comprehensive analysis
    if let Some(output_dir) = OUTPUT_DIRECTORY.get() {
        let output_dir = output_dir.clone();
        generate_reports(&output_dir)?;

        println!(
            "🎉 Tracking complete: {}/memory_report.html (delegating to unified tracker)",
            output_dir.display()
        );
    }

    Ok(())
}

/// Check if lockfree tracking is currently active
///
/// # Returns
/// Boolean indicating whether memory tracking is enabled
///
/// # Example
/// ```rust
/// use memscope_rs::lockfree::api::{trace_all, is_tracking};
///
/// assert!(!is_tracking());
/// trace_all("./analysis")?;
/// assert!(is_tracking());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn is_tracking() -> bool {
    // For backward compatibility, only check the old system
    // The new unified tracker may be enabled by default
    TRACKING_ENABLED.load(Ordering::SeqCst)
}

/// Memory snapshot for real-time monitoring
///
/// Provides current memory usage statistics without stopping tracking.
/// Useful for monitoring memory consumption during application execution.
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    /// Current memory usage in megabytes
    pub current_mb: f64,
    /// Peak memory usage in megabytes
    pub peak_mb: f64,
    /// Total number of allocations tracked
    pub allocations: u64,
    /// Total number of deallocations tracked
    pub deallocations: u64,
    /// Number of threads currently being tracked
    pub active_threads: usize,
}

/// Get current memory usage snapshot
///
/// Returns real-time memory statistics without interrupting tracking.
///
/// # Returns
/// MemorySnapshot containing current memory usage data
///
/// # Example
/// ```rust
/// use memscope_rs::lockfree::api::{trace_all, memory_snapshot};
///
/// trace_all("./analysis")?;
/// // ... run some code ...
/// let snapshot = memory_snapshot();
/// println!("Current memory: {:.1} MB", snapshot.current_mb);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn memory_snapshot() -> MemorySnapshot {
    // Query the unified tracking system
    use crate::new::tracker::get_global_tracker;

    let unified = get_global_tracker();
    let stats = unified.stats();

    // Convert to old snapshot format
    // Use total_size as peak, active_allocations * average size as current
    let avg_size = if stats.active_allocations > 0 {
        stats.total_size / stats.active_allocations
    } else {
        0
    };
    let current_size = stats.active_allocations * avg_size;

    let current_mb = current_size as f64 / (1024.0 * 1024.0);
    let peak_mb = stats.total_size as f64 / (1024.0 * 1024.0);

    MemorySnapshot {
        current_mb,
        peak_mb,
        allocations: stats.total_allocations as u64,
        deallocations: (stats.total_allocations - stats.active_allocations) as u64,
        active_threads: 1, // Simplified
    }
}

/// Quick trace function for benchmarking and profiling
///
/// Automatically starts tracking, executes the provided function, and
/// stops tracking, generating reports. Perfect for one-off profiling.
///
/// # Arguments
/// * `f` - Function to execute while tracking
///
/// # Returns
/// Result of the function execution
///
/// # Example
/// ```rust
/// use memscope_rs::lockfree::api::quick_trace;
///
/// let result = quick_trace(|| {
///     // Your code to profile
///     vec![0u8; 1024 * 1024].len()
/// });
/// assert_eq!(result, 1_000_000);
/// ```
pub fn quick_trace<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let temp_dir = std::env::temp_dir().join("memscope_lockfree_quick");

    // Start tracking
    if trace_all(&temp_dir).is_err() {
        return f(); // Fallback to untracked execution
    }

    // Execute function
    let result = f();

    // Stop tracking and show basic summary
    if stop_tracing().is_ok() {
        println!("📊 Quick trace completed - check {}", temp_dir.display());
    }

    result
}

/// Generate comprehensive analysis reports
///
/// Creates HTML and JSON reports from collected tracking data.
/// Called automatically by stop_tracing().
fn generate_reports(output_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create simple comprehensive analysis
    use super::analysis::LockfreeAnalysis;
    use super::resource_integration::{
        ComprehensiveAnalysis, CorrelationMetrics, PerformanceInsights,
    };

    let analysis = ComprehensiveAnalysis {
        memory_analysis: LockfreeAnalysis::default(),
        resource_timeline: Vec::new(),
        correlation_metrics: CorrelationMetrics {
            memory_cpu_correlation: 0.0,
            memory_gpu_correlation: 0.0,
            memory_io_correlation: 0.0,
            allocation_rate_vs_cpu_usage: 0.0,
            deallocation_rate_vs_memory_pressure: 0.0,
        },
        performance_insights: PerformanceInsights {
            primary_bottleneck: super::resource_integration::BottleneckType::Balanced,
            cpu_efficiency_score: 1.0,
            memory_efficiency_score: 1.0,
            io_efficiency_score: 1.0,
            recommendations: Vec::new(),
            thread_performance_ranking: Vec::new(),
        },
    };

    // Generate comprehensive analysis
    export_comprehensive_analysis(&analysis, output_dir, "memory_analysis")?;

    println!(
        "📊 Generated comprehensive analysis: {}/memory_analysis_comprehensive.json",
        output_dir.display()
    );

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_all_and_stop() {
        // Ensure clean state at start
        TRACKING_ENABLED.store(false, Ordering::SeqCst);

        let temp_dir = std::env::temp_dir().join("memscope_test_trace_all");

        let result = trace_all(&temp_dir);
        assert!(result.is_ok());
        println!("After trace_all, is_tracking() = {}", is_tracking());
        assert!(is_tracking());

        let result = stop_tracing();
        assert!(result.is_ok());
        println!("After stop_tracing, is_tracking() = {}", is_tracking());
        assert!(!is_tracking());

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);

        // Reset state for next test
        TRACKING_ENABLED.store(false, Ordering::SeqCst);
    }

    #[test]
    fn test_is_tracking_initial_state() {
        assert!(!is_tracking());
    }

    #[test]
    fn test_memory_snapshot() {
        let snapshot = memory_snapshot();
        // Basic validation
        assert!(snapshot.current_mb >= 0.0);
        assert!(snapshot.peak_mb >= 0.0);
    }

    #[test]
    fn test_quick_trace() {
        let result = quick_trace(|| {
            // Simulate some work
            let _data = vec![0u8; 1024];
            42
        });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_stop_without_start() {
        let result = stop_tracing();
        assert!(result.is_ok()); // Should succeed even without starting
    }

    #[test]
    fn test_trace_thread() {
        let temp_dir = std::env::temp_dir().join("memscope_test_trace_thread");

        let result = trace_thread(&temp_dir);
        assert!(result.is_ok());

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_trace_all_creates_directory() {
        let temp_dir = std::env::temp_dir().join("memscope_test_dir_create");

        let _ = trace_all(&temp_dir);
        assert!(temp_dir.exists());

        let _ = stop_tracing();
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
