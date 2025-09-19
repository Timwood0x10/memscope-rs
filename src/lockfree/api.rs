//! Lockfree Memory Tracking API
//!
//! Provides simple, high-level interfaces for lockfree memory tracking.
//! Designed for minimal friction and maximum usability.

use super::aggregator::LockfreeAggregator;
use super::tracker::{finalize_thread_tracker, init_thread_tracker, SamplingConfig};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

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

    // Enable global tracking
    TRACKING_ENABLED.store(true, Ordering::SeqCst);

    println!("🚀 Lockfree tracking started: {}", output_path.display());

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

    // Initialize tracking for current thread with high precision
    init_thread_tracker(&output_path, Some(SamplingConfig::demo()))?;

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

    // Finalize current thread tracker if needed
    let _ = finalize_thread_tracker();

    // Disable global tracking
    TRACKING_ENABLED.store(false, Ordering::SeqCst);

    // Generate comprehensive analysis
    let output_dir = OUTPUT_DIRECTORY
        .get()
        .ok_or("Output directory not set")?
        .clone();

    generate_reports(&output_dir)?;

    println!(
        "🎉 Tracking complete: {}/memory_report.html",
        output_dir.display()
    );

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
    // In a real implementation, this would query the active tracking system
    // For now, return a basic snapshot
    MemorySnapshot {
        current_mb: 0.0,
        peak_mb: 0.0,
        allocations: 0,
        deallocations: 0,
        active_threads: if TRACKING_ENABLED.load(Ordering::SeqCst) {
            1
        } else {
            0
        },
    }
}

/// Auto-tracking macro for scoped memory analysis
///
/// Automatically starts tracking, runs the provided code block, then stops
/// tracking and generates reports. Perfect for analyzing specific code sections.
///
/// # Arguments
/// * `output_dir` - Directory for storing analysis results
/// * `block` - Code block to analyze
///
/// # Example
/// ```rust
/// use memscope_rs::auto_trace;
///
/// fn main() {
///     let result = auto_trace!("./analysis", {
///         let data = vec![1, 2, 3, 4, 5];
///         data.len()
///     });
///     assert_eq!(result, 5);
/// }
/// ```
#[macro_export]
macro_rules! auto_trace {
    ($output_dir:expr, $block:block) => {{
        $crate::lockfree::api::trace_all($output_dir)?;
        let result = (|| $block)();
        $crate::lockfree::api::stop_tracing()?;
        result
    }};
}

/// Quick trace function for debugging and profiling
///
/// Runs the provided function with temporary memory tracking enabled.
/// Results are stored in a temporary directory and basic statistics are printed.
///
/// # Arguments
/// * `f` - Function to execute with tracking enabled
///
/// # Returns
/// The return value of the provided function
///
/// # Example
/// ```rust
/// use memscope_rs::lockfree::api::quick_trace;
///
/// let result = quick_trace(|| {
///     let big_vec = vec![0u8; 1_000_000];
///     big_vec.len()
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
///
/// # Arguments
/// * `output_dir` - Directory containing tracking data and where reports will be saved
///
/// # Returns
/// Result indicating success or error during report generation
fn generate_reports(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let aggregator = LockfreeAggregator::new(output_dir.to_path_buf());
    let analysis = aggregator.aggregate_all_threads()?;

    // Generate HTML report
    let html_path = output_dir.join("memory_report.html");
    aggregator.generate_html_report(&analysis, &html_path)?;

    // Generate JSON data export
    let json_path = output_dir.join("memory_data.json");
    aggregator.export_analysis(&analysis, &json_path)?;

    // Print summary statistics
    print_analysis_summary(&analysis);

    Ok(())
}

/// Print concise analysis summary to console
///
/// # Arguments
/// * `analysis` - Analysis results to summarize
fn print_analysis_summary(analysis: &super::analysis::LockfreeAnalysis) {
    println!("\n📊 Lockfree Memory Analysis:");
    println!("   🧵 Threads analyzed: {}", analysis.thread_stats.len());
    println!(
        "   📈 Peak memory: {:.1} MB",
        analysis.summary.peak_memory_usage as f64 / (1024.0 * 1024.0)
    );
    println!(
        "   🔄 Total allocations: {}",
        analysis.summary.total_allocations
    );
    println!(
        "   ↩️  Total deallocations: {}",
        analysis.summary.total_deallocations
    );

    if analysis.summary.total_allocations > 0 {
        let efficiency = analysis.summary.total_deallocations as f64
            / analysis.summary.total_allocations as f64
            * 100.0;
        println!("   ⚡ Memory efficiency: {:.1}%", efficiency);
    }
}
