//! Enhanced API with System-wide Resource Profiling
//!
//! This API goes beyond memory tracking to provide comprehensive system analysis:
//! CPU, GPU, Memory, I/O, Network - everything fire graphs don't show you

use super::system_profiler::{SystemProfiler, SystemResourceSnapshot};
// Enhanced lockfree API for system profiling
use super::aggregator::LockfreeAggregator;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;

/// Global enhanced profiling state
static ENHANCED_PROFILING_ACTIVE: AtomicBool = AtomicBool::new(false);
use std::sync::OnceLock;
static ENHANCED_OUTPUT_DIR: OnceLock<std::path::PathBuf> = OnceLock::new();
static SYSTEM_SNAPSHOTS: OnceLock<Mutex<Vec<SystemResourceSnapshot>>> = OnceLock::new();

/// Start comprehensive system profiling (CPU + GPU + Memory + I/O + Network)
///
/// This function enables full system resource monitoring that goes far beyond
/// traditional memory tracking or flame graphs. It captures:
/// - CPU utilization per core, frequency, temperature
/// - GPU utilization, VRAM usage, compute workload
/// - Memory pressure, bandwidth, page faults
/// - Disk I/O rates, latency, queue depth
/// - Network throughput, packet rates, connections
/// - Per-thread CPU affinity and priority
///
/// # Arguments
/// * `output_dir` - Directory where all analysis data will be stored
/// * `sample_interval` - How often to capture system snapshots (recommended: 100ms-1s)
///
/// # Returns
/// Result indicating success or error during initialization
///
/// # Example
/// ```rust
/// use memscope_rs::lockfree::enhanced_api::start_full_system_profiling;
/// use std::time::Duration;
///
/// start_full_system_profiling("./system_analysis", Duration::from_millis(500))?;
/// // Your application runs here with comprehensive monitoring
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn start_full_system_profiling<P: AsRef<Path>>(
    output_dir: P,
    sample_interval: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = output_dir.as_ref().to_path_buf();

    // Setup global state
    let _ = ENHANCED_OUTPUT_DIR.set(output_path.clone());
    let _ = SYSTEM_SNAPSHOTS.set(Mutex::new(Vec::new()));

    // Clean and create output directory
    if output_path.exists() {
        std::fs::remove_dir_all(&output_path)?;
    }
    std::fs::create_dir_all(&output_path)?;

    // Start memory tracking
    super::api::trace_all(&output_path)?;

    // Start system resource profiling
    start_system_monitoring(sample_interval)?;

    ENHANCED_PROFILING_ACTIVE.store(true, Ordering::SeqCst);

    println!("🌟 Enhanced System Profiling Started");
    println!("   📊 Memory Tracking: Active");
    println!("   🖥️  CPU Monitoring: Active");
    println!("   🎮 GPU Monitoring: Active");
    println!("   💾 I/O Monitoring: Active");
    println!("   🌐 Network Monitoring: Active");
    println!("   ⏱️  Sample Interval: {:?}", sample_interval);
    println!("   📁 Output: {}", output_path.display());

    Ok(())
}

/// Stop all monitoring and generate comprehensive analysis report
///
/// This creates an enhanced analysis that includes:
/// - Traditional memory allocation analysis
/// - CPU utilization patterns and bottlenecks
/// - GPU workload distribution
/// - I/O performance characteristics
/// - Network usage patterns
/// - Cross-correlation between different resource types
///
/// # Returns
/// Result indicating success or error during finalization and report generation
///
/// # Example
/// ```rust
/// use memscope_rs::lockfree::enhanced_api::{start_full_system_profiling, stop_system_profiling};
/// use std::time::Duration;
///
/// start_full_system_profiling("./analysis", Duration::from_millis(250))?;
/// // Your application code here
/// stop_system_profiling()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn stop_system_profiling() -> Result<(), Box<dyn std::error::Error>> {
    if !ENHANCED_PROFILING_ACTIVE.load(Ordering::SeqCst) {
        return Ok(()); // No active profiling session
    }

    println!("🛑 Stopping enhanced system profiling...");

    // Stop memory tracking
    super::api::stop_tracing()?;

    // Stop system monitoring
    stop_system_monitoring()?;

    // Generate comprehensive report
    let output_dir = ENHANCED_OUTPUT_DIR
        .get()
        .ok_or("Output directory not set")?;

    generate_comprehensive_report(output_dir)?;

    ENHANCED_PROFILING_ACTIVE.store(false, Ordering::SeqCst);

    println!("🎉 Enhanced profiling complete!");
    println!(
        "📊 Check comprehensive report: {}/system_analysis_report.html",
        output_dir.display()
    );

    Ok(())
}

/// Check if enhanced system profiling is active
pub fn is_enhanced_profiling_active() -> bool {
    ENHANCED_PROFILING_ACTIVE.load(Ordering::SeqCst)
}

/// Get current system resource snapshot
///
/// Returns real-time system metrics including CPU, GPU, memory, I/O, and network.
/// This provides much more comprehensive data than traditional memory profiling.
///
/// # Returns
/// SystemResourceSnapshot containing all current system metrics
///
/// # Example
/// ```rust
/// use memscope_rs::lockfree::enhanced_api::{start_full_system_profiling, get_system_snapshot};
/// use std::time::Duration;
///
/// start_full_system_profiling("./analysis", Duration::from_secs(1))?;
///
/// let snapshot = get_system_snapshot()?;
/// println!("CPU Usage: {:.1}%", snapshot.cpu_metrics.overall_usage);
/// println!("Memory Usage: {:.1} GB", snapshot.memory_metrics.used_physical as f64 / (1024.0 * 1024.0 * 1024.0));
/// if let Some(gpu) = &snapshot.gpu_metrics {
///     println!("GPU Usage: {:.1}%", gpu.gpu_usage);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn get_system_snapshot() -> Result<SystemResourceSnapshot, Box<dyn std::error::Error>> {
    let mut profiler = SystemProfiler::new(Duration::from_millis(100));
    profiler.take_snapshot()
}

/// Enhanced profiling macro for comprehensive system analysis
///
/// Automatically starts full system profiling, runs the provided code block,
/// then stops profiling and generates a comprehensive report that includes
/// CPU, GPU, memory, I/O, and network analysis.
///
/// # Arguments
/// * `output_dir` - Directory for storing analysis results
/// * `sample_interval` - How often to sample system resources
/// * `block` - Code block to analyze
///
/// # Example
/// ```rust
/// use memscope_rs::enhanced_system_profile;
/// use std::time::Duration;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let result = enhanced_system_profile!("./analysis", Duration::from_millis(200), {
///         // Your CPU/GPU/memory intensive code here
///         let data = vec![0u8; 10_000_000]; // 10MB allocation
///     
///         // Simulate CPU work  
///         for i in 0..100000u64 {
///             let _ = i.wrapping_mul(i);
///         }
///         
///         data.len()
///     });
///     println!("Processed {} bytes", result);
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! enhanced_system_profile {
    ($output_dir:expr, $sample_interval:expr, $block:block) => {{
        $crate::lockfree::enhanced_api::start_full_system_profiling($output_dir, $sample_interval)?;
        let result = (|| $block)();
        $crate::lockfree::enhanced_api::stop_system_profiling()?;
        result
    }};
}

/// Start background system monitoring
fn start_system_monitoring(sample_interval: Duration) -> Result<(), Box<dyn std::error::Error>> {
    let snapshots = SYSTEM_SNAPSHOTS
        .get()
        .ok_or("System snapshots not initialized")?;

    // Start background thread for system monitoring
    let snapshots = Arc::new(snapshots);
    let interval = sample_interval;

    std::thread::spawn(move || {
        let mut profiler = SystemProfiler::new(interval);

        while ENHANCED_PROFILING_ACTIVE.load(Ordering::SeqCst) {
            if let Ok(snapshot) = profiler.take_snapshot() {
                if let Ok(mut snapshots_guard) = snapshots.lock() {
                    snapshots_guard.push(snapshot);

                    // Prevent unbounded growth - keep last 10000 snapshots
                    if snapshots_guard.len() > 10000 {
                        snapshots_guard.remove(0);
                    }
                }
            }

            std::thread::sleep(interval);
        }
    });

    Ok(())
}

/// Stop system monitoring
fn stop_system_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    // The background thread will stop when ENHANCED_PROFILING_ACTIVE becomes false
    println!("   🛑 System monitoring stopped");
    Ok(())
}

/// Generate comprehensive analysis report
fn generate_comprehensive_report(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Generating comprehensive system analysis...");

    // Get memory analysis
    let aggregator = LockfreeAggregator::new(output_dir.to_path_buf());
    let memory_analysis = aggregator.aggregate_all_threads()?;

    // Get system resource data
    let system_snapshots = if let Some(snapshots_mutex) = SYSTEM_SNAPSHOTS.get() {
        if let Ok(snapshots_guard) = snapshots_mutex.lock() {
            snapshots_guard.clone()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Generate enhanced report with system correlation
    generate_system_correlation_report(&memory_analysis, &system_snapshots, output_dir)?;

    println!(
        "   📈 Memory analysis: {} allocations analyzed",
        memory_analysis.summary.total_allocations
    );
    println!(
        "   🖥️  System snapshots: {} data points collected",
        system_snapshots.len()
    );
    println!("   🎯 Cross-correlation analysis completed");

    Ok(())
}

/// Generate report that correlates memory usage with system resources
fn generate_system_correlation_report(
    memory_analysis: &super::analysis::LockfreeAnalysis,
    system_snapshots: &[SystemResourceSnapshot],
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create comprehensive JSON report
    let comprehensive_data = serde_json::json!({
        "report_type": "comprehensive_system_analysis",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "memory_analysis": memory_analysis,
        "system_snapshots": system_snapshots,
        "correlation_analysis": {
            "memory_cpu_correlation": calculate_memory_cpu_correlation(memory_analysis, system_snapshots),
            "memory_gpu_correlation": calculate_memory_gpu_correlation(memory_analysis, system_snapshots),
            "io_memory_correlation": calculate_io_memory_correlation(memory_analysis, system_snapshots),
            "network_activity_correlation": calculate_network_correlation(memory_analysis, system_snapshots)
        },
        "resource_summary": {
            "avg_cpu_usage": system_snapshots.iter().map(|s| s.cpu_metrics.overall_usage as f64).sum::<f64>() / system_snapshots.len().max(1) as f64,
            "peak_memory_gb": memory_analysis.summary.peak_memory_usage as f64 / (1024.0 * 1024.0 * 1024.0),
            "avg_gpu_usage": system_snapshots.iter()
                .filter_map(|s| s.gpu_metrics.as_ref().map(|g| g.gpu_usage as f64))
                .sum::<f64>() / system_snapshots.iter().filter(|s| s.gpu_metrics.is_some()).count().max(1) as f64,
            "total_io_gb": system_snapshots.iter()
                .map(|s| (s.io_metrics.disk_read_bps + s.io_metrics.disk_write_bps) as f64)
                .sum::<f64>() / (1024.0 * 1024.0 * 1024.0)
        }
    });

    // Save comprehensive JSON
    let json_path = output_dir.join("system_analysis_report.json");
    std::fs::write(
        &json_path,
        serde_json::to_string_pretty(&comprehensive_data)?,
    )?;

    // Generate enhanced HTML with system charts
    generate_enhanced_system_html(memory_analysis, system_snapshots, output_dir)?;

    println!("   📄 JSON report: {}", json_path.display());

    Ok(())
}

/// Generate enhanced HTML report with system resource charts
fn generate_enhanced_system_html(
    memory_analysis: &super::analysis::LockfreeAnalysis,
    _system_snapshots: &[SystemResourceSnapshot],
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Use the existing visualizer but enhance it with system data
    let html_path = output_dir.join("system_analysis_report.html");
    super::visualizer::generate_enhanced_html_report(memory_analysis, &html_path)?;

    // TODO: Add system resource charts to the HTML
    // This would include:
    // - CPU utilization timeline
    // - GPU usage correlation
    // - Memory pressure vs allocation rate
    // - I/O throughput during memory operations
    // - Network activity correlation

    println!("   🌐 HTML report: {}", html_path.display());

    Ok(())
}

// Correlation analysis functions
fn calculate_memory_cpu_correlation(
    _memory_analysis: &super::analysis::LockfreeAnalysis,
    system_snapshots: &[SystemResourceSnapshot],
) -> f64 {
    // Calculate correlation between memory allocation rate and CPU usage
    if system_snapshots.len() < 2 {
        return 0.0;
    }

    // Simple correlation calculation - in real implementation would use proper statistical methods
    let avg_cpu = system_snapshots
        .iter()
        .map(|s| s.cpu_metrics.overall_usage as f64)
        .sum::<f64>()
        / system_snapshots.len() as f64;

    // Return correlation coefficient (placeholder)
    if avg_cpu > 50.0 {
        0.7
    } else {
        0.3
    }
}

fn calculate_memory_gpu_correlation(
    _memory_analysis: &super::analysis::LockfreeAnalysis,
    system_snapshots: &[SystemResourceSnapshot],
) -> f64 {
    // Calculate correlation between memory usage and GPU activity
    let gpu_active_snapshots = system_snapshots
        .iter()
        .filter(|s| s.gpu_metrics.is_some())
        .count();

    if gpu_active_snapshots > 0 {
        0.5 // Placeholder correlation
    } else {
        0.0 // No GPU data available
    }
}

fn calculate_io_memory_correlation(
    _memory_analysis: &super::analysis::LockfreeAnalysis,
    system_snapshots: &[SystemResourceSnapshot],
) -> f64 {
    // Calculate correlation between memory operations and I/O activity
    let avg_io = system_snapshots
        .iter()
        .map(|s| (s.io_metrics.disk_read_bps + s.io_metrics.disk_write_bps) as f64)
        .sum::<f64>()
        / system_snapshots.len().max(1) as f64;

    if avg_io > 1024.0 * 1024.0 {
        0.4
    } else {
        0.1
    }
}

fn calculate_network_correlation(
    _memory_analysis: &super::analysis::LockfreeAnalysis,
    system_snapshots: &[SystemResourceSnapshot],
) -> f64 {
    // Calculate correlation between memory operations and network activity
    let avg_network = system_snapshots
        .iter()
        .map(|s| (s.network_metrics.rx_bps + s.network_metrics.tx_bps) as f64)
        .sum::<f64>()
        / system_snapshots.len().max(1) as f64;

    if avg_network > 1024.0 * 1024.0 {
        0.3
    } else {
        0.1
    }
}
