//! Single-threaded Memory Tracking Test
//!
//! This example demonstrates the performance and capabilities of track_var! macro
//! for single-threaded memory analysis. It provides real data without any simulation
//! or fake tracking statistics.
//!
//! Tests different load scenarios (light, medium, heavy) to evaluate:
//! - Real data completeness from track_var! macro
//! - Actual throughput performance
//! - True memory usage from process RSS
//! - Real variable tracking capabilities

use memscope_rs::track_var;

use std::time::{Duration, Instant};

/// Test configuration for different load scenarios
#[derive(Debug, Clone)]
struct TestConfig {
    num_allocations: usize,
    allocation_size_range: (usize, usize),
}

impl TestConfig {
    fn light_load() -> Self {
        Self {
            num_allocations: 10_000,
            allocation_size_range: (64, 1024),
        }
    }

    fn medium_load() -> Self {
        Self {
            num_allocations: 50_000,
            allocation_size_range: (128, 2048),
        }
    }

    fn heavy_load() -> Self {
        Self {
            num_allocations: 100_000,
            allocation_size_range: (256, 4096),
        }
    }
}

/// Test results structure
#[derive(Debug)]
struct TestResults {
    mode: String,
    duration: Duration,
    allocations_tracked: usize,
    throughput: f64,
    memory_usage_mb: f64,
    completeness_rate: f64,
    smart_pointers_tracked: usize,
}

impl TestResults {
    fn new(mode: &str) -> Self {
        Self {
            mode: mode.to_string(),
            duration: Duration::default(),
            allocations_tracked: 0,
            throughput: 0.0,
            memory_usage_mb: 0.0,
            completeness_rate: 0.0,
            smart_pointers_tracked: 0,
        }
    }

    fn print_results(&self) {
        println!("📊 {} Results:", self.mode);
        println!("  Duration: {:.2}s", self.duration.as_secs_f64());
        println!("  Allocations Tracked: {}", self.allocations_tracked);
        println!("  Throughput: {:.0} allocs/sec", self.throughput);
        println!("  Memory Usage: {:.1} MB", self.memory_usage_mb);
        println!("  Completeness: {:.1}%", self.completeness_rate * 100.0);
        println!("  Smart Pointers: {}", self.smart_pointers_tracked);
        println!();
    }
}

/// Single-threaded test suite - REAL track_var! macro testing only
///
/// No simulation, no fake tracking. Only real single-threaded variable tracking.
struct SingleThreadedTestSuite {
    config: TestConfig,
}

impl SingleThreadedTestSuite {
    /// Create new test suite - no fake resources needed
    fn new(config: TestConfig) -> Self {
        Self { config }
    }

    /// Test 1: Single-threaded Mode - Using track_var! macro
    ///
    /// Uses ONLY track_var! macro for real variable tracking with actual memory allocations.
    /// No simulation, no fake data - only real heap-allocated variables.
    fn test_single_threaded(&self) -> TestResults {
        println!("🧵 Testing Single-threaded Mode (track_var! macro)");
        let mut results = TestResults::new("Single-threaded");

        let start = Instant::now();

        // Real allocation tracking using track_var! macro ONLY
        for i in 0..self.config.num_allocations {
            let size = self.config.allocation_size_range.0
                + (i % (self.config.allocation_size_range.1 - self.config.allocation_size_range.0));

            // Create REAL data structures and track them with track_var! - no simulation
            match i % 10 {
                0 => {
                    let numbers_vec: Vec<u64> =
                        (0..size / 8).map(|x| x as u64 + i as u64).collect();
                    track_var!(numbers_vec);
                }
                1 => {
                    let text_string = format!("data_{}_", i).repeat(size.min(100));
                    track_var!(text_string);
                }
                2 => {
                    let boxed_data = Box::new(vec![i as u8; size.min(1000)]);
                    track_var!(boxed_data);
                }
                3 => {
                    let rc_data = std::rc::Rc::new(format!("rc_data_{}", i));
                    track_var!(rc_data);
                }
                4 => {
                    let arc_data = std::sync::Arc::new(vec![i; size.min(500)]);
                    track_var!(arc_data);
                }
                5 => {
                    let hash_map: std::collections::HashMap<usize, String> = (0..size.min(100))
                        .map(|x| (x, format!("value_{}", x)))
                        .collect();
                    track_var!(hash_map);
                }
                6 => {
                    let btree_set: std::collections::BTreeSet<usize> = (0..size.min(200)).collect();
                    track_var!(btree_set);
                }
                7 => {
                    let vec_deque: std::collections::VecDeque<String> =
                        (0..size.min(150)).map(|x| format!("item_{}", x)).collect();
                    track_var!(vec_deque);
                }
                8 => {
                    let binary_heap: std::collections::BinaryHeap<usize> =
                        (0..size.min(100)).collect();
                    track_var!(binary_heap);
                }
                _ => {
                    let byte_buffer = vec![0u8; size.min(2000)];
                    track_var!(byte_buffer);
                }
            }
        }

        results.duration = start.elapsed();
        self.finalize_results_st(&mut results);
        results
    }

    /// Finalize single-threaded test results - get data from REAL tracking system
    fn finalize_results_st(&self, results: &mut TestResults) {
        // Get REAL statistics from track_var! macro's global tracking system
        let tracker = memscope_rs::core::tracker::get_tracker();
        let allocations = tracker.get_active_allocations();

        // Real data from actual tracking
        results.allocations_tracked = match allocations {
            Ok(allocs) => allocs.len(),
            Err(_) => 0,
        };
        results.memory_usage_mb = self.get_process_memory_usage() as f64 / (1024.0 * 1024.0);
        results.throughput = results.allocations_tracked as f64 / results.duration.as_secs_f64();

        // Calculate REAL completeness based on actual tracked vs expected allocations
        let expected = self.config.num_allocations;
        results.completeness_rate = if expected > 0 {
            (results.allocations_tracked as f64 / expected as f64).min(1.0)
        } else {
            0.0
        };

        // No fake smart pointer counting
        results.smart_pointers_tracked = 0;
    }

    /// Get current process memory usage using system APIs
    fn get_process_memory_usage(&self) -> usize {
        #[cfg(target_os = "linux")]
        {
            // On Linux, read from /proc/self/status
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<usize>() {
                                return kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, try to read memory info
            // Simplified approach without libc dependency
            if let Ok(output) = std::process::Command::new("ps")
                .args(["-o", "rss=", "-p"])
                .arg(std::process::id().to_string())
                .output()
            {
                if let Ok(rss_str) = String::from_utf8(output.stdout) {
                    if let Ok(rss_kb) = rss_str.trim().parse::<usize>() {
                        return rss_kb * 1024; // Convert KB to bytes
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, use a simpler approach or fallback
            // For now, use fallback since we don't have proper Windows API bindings
        }

        // Fallback: use a basic estimation if system APIs are unavailable
        let basic_estimate = self.config.num_allocations
            * ((self.config.allocation_size_range.0 + self.config.allocation_size_range.1) / 2);

        println!("Warning: Could not get real memory usage, using basic estimation");
        basic_estimate
    }

    /// Run single-threaded test suite across different load scenarios
    pub fn run_all_tests() {
        println!("🚀 Starting Single-threaded MemScope-RS Test Suite");
        println!("{}", "=".repeat(60));

        let scenarios = [
            ("Light Load", TestConfig::light_load()),
            ("Medium Load", TestConfig::medium_load()),
            ("Heavy Load", TestConfig::heavy_load()),
        ];

        for (scenario_name, config) in scenarios.iter() {
            println!("\n📋 Scenario: {scenario_name}");
            println!("{}", "-".repeat(40));

            let suite = SingleThreadedTestSuite::new(config.clone());

            // Test single-threaded mode only
            let st_results = suite.test_single_threaded();

            // Print results
            st_results.print_results();

            println!("\n📊 {} Summary:", scenario_name);
            println!(
                "  Real Allocations Tracked: {}",
                st_results.allocations_tracked
            );
            println!(
                "  Real Completeness: {:.1}%",
                st_results.completeness_rate * 100.0
            );
            println!("  Real Memory Usage: {:.1} MB", st_results.memory_usage_mb);
            println!("  Smart Pointers: {}", st_results.smart_pointers_tracked);
        }

        println!("\n✅ Single-threaded test suite completed!");
        println!("💡 For multi-threaded tests, run: cargo run --example lockfree_test");
        println!("💡 For async tests, run: cargo run --example async_memory_test");
    }
}

fn main() {
    // Initialize MemScope-RS (note: init() returns () so no expect needed)
    memscope_rs::init();

    SingleThreadedTestSuite::run_all_tests();
}
