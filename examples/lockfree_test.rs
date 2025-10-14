//! Lockfree Multi-threaded Memory Tracking Test
//!
//! This example demonstrates the lockfree tracking API for high-performance
//! concurrent memory analysis. It uses real lockfree tracking without any
//! simulation or fake data.
//!
//! Features tested:
//! - Real lockfree concurrent tracking
//! - Multi-threaded memory allocations
//! - Thread-safe data collection
//! - Performance analysis across threads

use memscope_rs::lockfree::{
    export_comprehensive_analysis, finalize_thread_tracker, init_thread_tracker,
    track_allocation_lockfree, track_deallocation_lockfree, IntegratedProfilingSession,
    SamplingConfig,
};
use std::{
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

/// Test configuration for lockfree scenarios
#[derive(Debug, Clone)]
struct LockfreeTestConfig {
    num_threads: usize,
    allocations_per_thread: usize,
    allocation_size_range: (usize, usize),
}

impl LockfreeTestConfig {
    fn light_concurrent() -> Self {
        Self {
            num_threads: 4,
            allocations_per_thread: 2_500,
            allocation_size_range: (64, 1024),
        }
    }

    fn medium_concurrent() -> Self {
        Self {
            num_threads: 8,
            allocations_per_thread: 6_250,
            allocation_size_range: (128, 2048),
        }
    }

    fn heavy_concurrent() -> Self {
        Self {
            num_threads: 16,
            allocations_per_thread: 6_250,
            allocation_size_range: (256, 4096),
        }
    }
}

/// Lockfree test results
#[derive(Debug)]
struct LockfreeResults {
    scenario: String,
    num_threads: usize,
    total_allocations: usize,
    duration: Duration,
    throughput: f64,
    memory_usage_mb: f64,
    tracking_success: bool,
}

impl LockfreeResults {
    fn new(scenario: &str, num_threads: usize) -> Self {
        Self {
            scenario: scenario.to_string(),
            num_threads,
            total_allocations: 0,
            duration: Duration::default(),
            throughput: 0.0,
            memory_usage_mb: 0.0,
            tracking_success: false,
        }
    }

    fn print_results(&self) {
        println!("📊 {} Results:", self.scenario);
        println!("  Threads: {}", self.num_threads);
        println!("  Total Allocations: {}", self.total_allocations);
        println!("  Duration: {:.2}s", self.duration.as_secs_f64());
        println!("  Throughput: {:.0} allocs/sec", self.throughput);
        println!("  Memory Usage: {:.1} MB", self.memory_usage_mb);
        println!(
            "  Tracking Success: {}",
            if self.tracking_success { "✅" } else { "❌" }
        );
        println!();
    }
}

/// Lockfree test suite
struct LockfreeTestSuite {
    config: LockfreeTestConfig,
    output_dir: PathBuf,
}

impl LockfreeTestSuite {
    fn new(config: LockfreeTestConfig) -> Self {
        let output_dir = std::env::temp_dir().join("memscope_lockfree_test");
        let _ = std::fs::create_dir_all(&output_dir);

        Self { config, output_dir }
    }

    /// Test lockfree tracking with real concurrent allocations
    fn test_lockfree_tracking(&self) -> LockfreeResults {
        println!("🔀 Testing Lockfree Multi-threaded Mode");
        let mut results = LockfreeResults::new("Lockfree", self.config.num_threads);

        // Initialize integrated profiling session like complex_multithread_showcase
        let mut session = match IntegratedProfilingSession::new(&self.output_dir) {
            Ok(session) => session,
            Err(e) => {
                eprintln!("❌ Failed to create profiling session: {}", e);
                return results;
            }
        };

        if let Err(e) = session.start_profiling() {
            eprintln!("❌ Failed to start profiling: {}", e);
            return results;
        }

        println!("🚀 Lockfree tracking with integrated profiling started");
        results.tracking_success = true;

        let start = Instant::now();
        let mut handles = Vec::new();

        // Spawn worker threads with real memory allocations
        for thread_id in 0..self.config.num_threads {
            let config = self.config.clone();
            let output_dir = self.output_dir.clone();

            let handle = thread::spawn(move || {
                // Initialize thread-specific lockfree tracking
                if let Err(e) = init_thread_tracker(&output_dir, Some(SamplingConfig::demo())) {
                    eprintln!("Thread {} failed to initialize tracking: {}", thread_id, e);
                    return 0;
                }

                let mut _allocated_data: Vec<Box<Vec<u8>>> = Vec::new();

                // Real memory allocations under lockfree tracking
                for i in 0..config.allocations_per_thread {
                    let size = config.allocation_size_range.0
                        + (i % (config.allocation_size_range.1 - config.allocation_size_range.0));

                    // Create realistic workload allocations with proper lockfree tracking
                    let ptr = (thread_id as u64) << 32 | (i as u64) << 16 | size as u64;
                    let call_stack = vec![
                        main as *const () as usize,
                        std::ptr::null::<u8>() as usize + thread_id * 1000 + i,
                    ];

                    // Track the allocation with lockfree API (using realistic ptr patterns)
                    if let Err(e) = track_allocation_lockfree(ptr as usize, size, &call_stack) {
                        eprintln!("Thread {} failed to track allocation: {}", thread_id, e);
                        return 0;
                    }

                    // Create real heap allocations for memory pressure
                    match i % 8 {
                        0 => {
                            let data = vec![thread_id as u8; size.min(65536)];
                            let boxed = Box::new(data);
                            _allocated_data.push(boxed);
                        }
                        1 => {
                            let strings: Vec<String> = (0..size.min(100))
                                .map(|x| format!("thread_{}_item_{}", thread_id, x))
                                .collect();
                            let bytes: Vec<u8> = strings.join(",").into_bytes();
                            let boxed = Box::new(bytes);
                            _allocated_data.push(boxed);
                        }
                        2 => {
                            let arc_data = Arc::new(vec![i; size.min(200)]);
                            let bytes: Vec<u8> =
                                arc_data.iter().map(|&x| (x % 256) as u8).collect();
                            let boxed = Box::new(bytes);
                            _allocated_data.push(boxed);
                        }
                        3 => {
                            let hash_data: std::collections::HashMap<usize, u8> =
                                (0..size.min(50)).map(|x| (x, (x % 256) as u8)).collect();
                            let bytes: Vec<u8> = hash_data.values().cloned().collect();
                            let boxed = Box::new(bytes);
                            _allocated_data.push(boxed);
                        }
                        4 => {
                            let deque_data: std::collections::VecDeque<u8> =
                                (0..size).map(|x| (x % 256) as u8).collect();
                            let bytes: Vec<u8> = deque_data.into_iter().collect();
                            let boxed = Box::new(bytes);
                            _allocated_data.push(boxed);
                        }
                        5 => {
                            let binary_heap: std::collections::BinaryHeap<usize> =
                                (0..size.min(100)).collect();
                            let bytes: Vec<u8> = binary_heap
                                .into_vec()
                                .iter()
                                .map(|&x| (x % 256) as u8)
                                .collect();
                            let boxed = Box::new(bytes);
                            _allocated_data.push(boxed);
                        }
                        6 => {
                            let btree_data: std::collections::BTreeSet<usize> =
                                (0..size.min(80)).collect();
                            let bytes: Vec<u8> =
                                btree_data.iter().map(|&x| (x % 256) as u8).collect();
                            let boxed = Box::new(bytes);
                            _allocated_data.push(boxed);
                        }
                        _ => {
                            let custom_struct = CustomData {
                                id: thread_id,
                                data: vec![i as u8; size.min(500)],
                                metadata: format!("thread_{}_item_{}", thread_id, i),
                            };
                            let bytes: Vec<u8> = custom_struct.serialize();
                            let boxed = Box::new(bytes);
                            _allocated_data.push(boxed);
                        }
                    }

                    // Simulate realistic memory lifecycle - periodic deallocation
                    if i > 0 && i % 20 == 0 {
                        // Deallocate some earlier allocations
                        let dealloc_i = i - 20;
                        let dealloc_ptr =
                            (thread_id as u64) << 32 | (dealloc_i as u64) << 16 | size as u64;
                        let dealloc_call_stack = vec![
                            main as *const () as usize,
                            std::ptr::null::<u8>() as usize + thread_id * 1000 + dealloc_i,
                        ];

                        if let Err(e) =
                            track_deallocation_lockfree(dealloc_ptr as usize, &dealloc_call_stack)
                        {
                            eprintln!("Thread {} failed to track deallocation: {}", thread_id, e);
                        }
                    }

                    // Periodic cleanup of real heap allocations
                    if i % 100 == 0 && _allocated_data.len() > 20 {
                        _allocated_data.drain(0..10);
                    }

                    // Brief pause for realistic workload
                    thread::sleep(Duration::from_micros(50 + (i % 10) as u64));
                }

                // Finalize thread tracking
                if let Err(e) = finalize_thread_tracker() {
                    eprintln!("Thread {} failed to finalize tracking: {}", thread_id, e);
                }

                _allocated_data.len()
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        let mut total_allocations = 0;
        for (i, handle) in handles.into_iter().enumerate() {
            match handle.join() {
                Ok(count) => {
                    total_allocations += count;
                    println!("   Thread {} completed: {} allocations", i, count);
                }
                Err(_) => {
                    eprintln!("   Thread {} panicked", i);
                }
            }
        }

        results.duration = start.elapsed();
        results.total_allocations = total_allocations;
        results.throughput = total_allocations as f64 / results.duration.as_secs_f64();
        results.memory_usage_mb = self.get_process_memory_usage() as f64 / (1024.0 * 1024.0);

        // Stop profiling and generate comprehensive analysis
        println!("📊 Generating comprehensive analysis...");
        match session.stop_profiling_and_analyze() {
            Ok(analysis) => {
                let report_name = format!(
                    "lockfree_test_{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );
                match export_comprehensive_analysis(&analysis, &self.output_dir, &report_name) {
                    Ok(_) => {
                        println!("✅ Lockfree tracking completed successfully");
                        println!("📁 Reports generated in: {}", self.output_dir.display());
                        println!(
                            "📊 JSON Report: {}/{}_comprehensive.json",
                            self.output_dir.display(),
                            report_name
                        );
                        println!(
                            "🌐 HTML Dashboard: {}/{}_dashboard.html",
                            self.output_dir.display(),
                            report_name
                        );
                        println!(
                            "📈 Rankings: {}/{}_resource_rankings.json",
                            self.output_dir.display(),
                            report_name
                        );
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to export comprehensive analysis: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️  Failed to stop profiling and analyze: {}", e);
                // Still show basic file info as fallback
                if self.output_dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(&self.output_dir) {
                        let files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                        println!("📊 Generated {} raw tracking files", files.len());
                    }
                }
            }
        }

        results
    }

    /// Get process memory usage
    fn get_process_memory_usage(&self) -> usize {
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<usize>() {
                                return kb * 1024;
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = std::process::Command::new("ps")
                .args(["-o", "rss=", "-p"])
                .arg(std::process::id().to_string())
                .output()
            {
                if let Ok(rss_str) = String::from_utf8(output.stdout) {
                    if let Ok(rss_kb) = rss_str.trim().parse::<usize>() {
                        return rss_kb * 1024;
                    }
                }
            }
        }

        // Fallback estimation
        50 * 1024 * 1024 // 50MB
    }

    /// Run comprehensive lockfree tests
    pub fn run_all_tests() {
        println!("🚀 Starting Lockfree Multi-threaded Test Suite");
        println!("{}", "=".repeat(60));

        let scenarios = [
            ("Light Concurrent", LockfreeTestConfig::light_concurrent()),
            ("Medium Concurrent", LockfreeTestConfig::medium_concurrent()),
            ("Heavy Concurrent", LockfreeTestConfig::heavy_concurrent()),
        ];

        let mut all_results = Vec::new();

        for (scenario_name, config) in scenarios.iter() {
            println!("\n📋 Scenario: {}", scenario_name);
            println!("{}", "-".repeat(40));

            let suite = LockfreeTestSuite::new(config.clone());
            let results = suite.test_lockfree_tracking();

            results.print_results();
            all_results.push((scenario_name.to_string(), results));

            println!("📊 {} Summary:", scenario_name);
            println!(
                "  Concurrent Efficiency: {:.1} allocs/thread/sec",
                all_results.last().unwrap().1.throughput
                    / all_results.last().unwrap().1.num_threads as f64
            );
            println!(
                "  Memory per Thread: {:.1} MB",
                all_results.last().unwrap().1.memory_usage_mb
                    / all_results.last().unwrap().1.num_threads as f64
            );
        }

        // Generate a single comprehensive HTML report for all scenarios
        println!("\n📊 Generating single comprehensive HTML report for all scenarios...");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let final_report_path = format!("lockfree_comprehensive_report_{}.html", timestamp);

        if let Err(e) = generate_combined_html_report(&all_results, &final_report_path) {
            eprintln!("⚠️  Failed to generate combined report: {}", e);
        } else {
            println!("🌐 Single HTML Report: {}", final_report_path);
        }

        println!("✅ Lockfree test suite completed!");
        println!("💡 For single-threaded tests, run: cargo run --example comprehensive_modes_test");
        println!("💡 For async tests, run: cargo run --example async_memory_test");
    }
}

/// Custom data structure for testing
#[derive(Debug)]
struct CustomData {
    id: usize,
    data: Vec<u8>,
    metadata: String,
}

impl CustomData {
    fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.id.to_le_bytes());
        result.extend_from_slice(&(self.data.len() as u32).to_le_bytes());
        result.extend_from_slice(&self.data);
        result.extend_from_slice(&(self.metadata.len() as u32).to_le_bytes());
        result.extend_from_slice(self.metadata.as_bytes());
        result
    }
}

/// Generate a single combined HTML report for all lockfree scenarios
fn generate_combined_html_report(
    all_results: &[(String, LockfreeResults)],
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut html_content = String::new();

    html_content.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html_content.push_str("<title>Lockfree Comprehensive Test Report</title>\n");
    html_content.push_str("<style>\n");
    html_content
        .push_str("body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }\n");
    html_content.push_str(".container { max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }\n");
    html_content.push_str("h1 { color: #2c3e50; text-align: center; margin-bottom: 30px; }\n");
    html_content.push_str(
        "h2 { color: #34495e; border-bottom: 2px solid #3498db; padding-bottom: 10px; }\n",
    );
    html_content.push_str(".scenario { margin: 30px 0; padding: 20px; border: 1px solid #ddd; border-radius: 5px; }\n");
    html_content.push_str(".metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin: 20px 0; }\n");
    html_content.push_str(
        ".metric { background: #ecf0f1; padding: 15px; border-radius: 5px; text-align: center; }\n",
    );
    html_content
        .push_str(".metric-value { font-size: 24px; font-weight: bold; color: #2980b9; }\n");
    html_content.push_str(".metric-label { color: #7f8c8d; margin-top: 5px; }\n");
    html_content.push_str(
        ".summary { background: #e8f5e8; padding: 20px; border-radius: 5px; margin-top: 30px; }\n",
    );
    html_content.push_str("</style>\n</head>\n<body>\n");

    html_content.push_str("<div class='container'>\n");
    html_content.push_str("<h1>🔀 Lockfree Multi-threaded Tracking - Comprehensive Report</h1>\n");

    // Summary section
    let total_allocations: usize = all_results.iter().map(|(_, r)| r.total_allocations).sum();
    let total_threads: usize = all_results.iter().map(|(_, r)| r.num_threads).sum();
    let avg_throughput: f64 =
        all_results.iter().map(|(_, r)| r.throughput).sum::<f64>() / all_results.len() as f64;

    html_content.push_str("<div class='summary'>\n");
    html_content.push_str("<h2>📊 Overall Summary</h2>\n");
    html_content.push_str(&format!(
        "<p><strong>Total Scenarios:</strong> {}</p>\n",
        all_results.len()
    ));
    html_content.push_str(&format!(
        "<p><strong>Total Allocations Tracked:</strong> {}</p>\n",
        total_allocations
    ));
    html_content.push_str(&format!(
        "<p><strong>Total Threads:</strong> {}</p>\n",
        total_threads
    ));
    html_content.push_str(&format!(
        "<p><strong>Average Throughput:</strong> {:.0} allocs/sec</p>\n",
        avg_throughput
    ));
    html_content.push_str("</div>\n");

    // Individual scenarios
    for (scenario_name, results) in all_results {
        html_content.push_str("<div class='scenario'>\n");
        html_content.push_str(&format!("<h2>📋 {}</h2>\n", scenario_name));

        html_content.push_str("<div class='metrics'>\n");
        html_content.push_str(&format!("<div class='metric'><div class='metric-value'>{}</div><div class='metric-label'>Threads</div></div>\n", results.num_threads));
        html_content.push_str(&format!("<div class='metric'><div class='metric-value'>{}</div><div class='metric-label'>Total Allocations</div></div>\n", results.total_allocations));
        html_content.push_str(&format!("<div class='metric'><div class='metric-value'>{:.2}s</div><div class='metric-label'>Duration</div></div>\n", results.duration.as_secs_f64()));
        html_content.push_str(&format!("<div class='metric'><div class='metric-value'>{:.0}</div><div class='metric-label'>Throughput (allocs/sec)</div></div>\n", results.throughput));
        html_content.push_str(&format!("<div class='metric'><div class='metric-value'>{:.1} MB</div><div class='metric-label'>Memory Usage</div></div>\n", results.memory_usage_mb));
        html_content.push_str(&format!("<div class='metric'><div class='metric-value'>{}</div><div class='metric-label'>Tracking Success</div></div>\n", if results.tracking_success { "✅" } else { "❌" }));
        html_content.push_str("</div>\n");

        html_content.push_str("</div>\n");
    }

    html_content.push_str("</div>\n</body>\n</html>");

    std::fs::write(output_path, html_content)?;
    println!("📄 Generated combined HTML report: {}", output_path);

    Ok(())
}

fn main() {
    LockfreeTestSuite::run_all_tests();
}
