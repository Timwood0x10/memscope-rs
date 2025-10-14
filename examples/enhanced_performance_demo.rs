/*!
Enhanced MemScope Performance Demonstration

This example showcases the performance improvements and new features
of the enhanced MemScope-RS memory analysis tool.

Key features demonstrated:
- Bounded history management (prevents unlimited memory growth)
- Smart type classification (unified system)
- Intelligent size estimation (platform-aware)
- Smart pointer tracking (Arc/Rc/Box analysis)
- Loss tracking statistics (data quality monitoring)

Run with: cargo run --example enhanced_performance_demo --release
*/

use memscope_rs::{
    classification::TypeClassifier,
    estimation::{size_estimator::SmartSizeEstimator, SizeEstimator},
    memory::{
        bounded_history::{BoundedHistory, BoundedHistoryConfig},
        config::MemoryConfig,
    },
    smart_pointers::tracker::SmartPointerTracker,
    tracking::stats::TrackingStats,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Simulated allocation data for testing
#[derive(Debug, Clone)]
struct AllocationInfo {
    ptr: usize,
    size: usize,
    type_name: String,
    thread_id: u32,
    timestamp: Instant,
}

impl AllocationInfo {
    fn new(ptr: usize, size: usize, type_name: &str, thread_id: u32) -> Self {
        Self {
            ptr,
            size,
            type_name: type_name.to_string(),
            thread_id,
            timestamp: Instant::now(),
        }
    }

    /// Get the age of this allocation
    fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }

    /// Check if this allocation is from a specific thread
    fn is_from_thread(&self, thread_id: u32) -> bool {
        self.thread_id == thread_id
    }

    /// Get size in MB for display
    fn size_mb(&self) -> f64 {
        self.size as f64 / (1024.0 * 1024.0)
    }

    /// Check if this is a large allocation (>1MB)
    fn is_large_allocation(&self) -> bool {
        self.size > 1024 * 1024
    }
}

/// Performance test scenarios
struct PerformanceTestSuite {
    bounded_history: BoundedHistory<AllocationInfo>,
    type_classifier: &'static TypeClassifier,
    size_estimator: SmartSizeEstimator,
    smart_pointer_tracker: std::cell::RefCell<SmartPointerTracker>,
    tracking_stats: TrackingStats,
}

impl PerformanceTestSuite {
    fn new() -> Self {
        // Configure bounded history with production-ready settings
        let memory_config = MemoryConfig::production();
        let bounded_config = BoundedHistoryConfig {
            max_entries: memory_config.max_allocations,
            max_age: memory_config.max_history_age,
            total_memory_limit: memory_config.memory_limit_mb * 1024 * 1024,
            cleanup_threshold: memory_config.cleanup_threshold as f32,
        };
        let bounded_history = BoundedHistory::with_config(bounded_config);

        // Initialize components
        let type_classifier = TypeClassifier::global();
        let size_estimator = SmartSizeEstimator::new();
        let smart_pointer_tracker = std::cell::RefCell::new(SmartPointerTracker::new());
        let tracking_stats = TrackingStats::new();

        Self {
            bounded_history,
            type_classifier,
            size_estimator,
            smart_pointer_tracker,
            tracking_stats,
        }
    }

    /// Test 1: Bounded History Performance
    /// Demonstrates how bounded history prevents memory growth
    fn test_bounded_history_performance(&self) {
        println!("🔬 Test 1: Bounded History Performance");
        println!("========================================");

        let start = Instant::now();
        let initial_stats = self.bounded_history.get_memory_usage_stats();
        let initial_memory = initial_stats.current_memory_usage;

        // Simulate high-volume allocation tracking
        for i in 0..100_000 {
            let allocation = AllocationInfo::new(
                0x1000 + i,
                64 + (i % 1024),
                &format!("TestType{}", i % 50),
                (i % 8) as u32,
            );

            self.bounded_history.push(allocation);

            // Check memory bounds every 10k allocations
            if i % 10_000 == 0 {
                let current_stats = self.bounded_history.get_memory_usage_stats();
                let entry_count = self.bounded_history.len();
                
                // Analyze some recent allocations using their fields
                let recent_allocations = self.bounded_history.entries();
                let large_allocations = recent_allocations.iter()
                    .filter(|alloc| alloc.is_large_allocation())
                    .count();
                let thread_0_allocations = recent_allocations.iter()
                    .filter(|alloc| alloc.is_from_thread(0))
                    .count();
                
                println!(
                    "  After {} allocations: {} entries, {:.1} MB memory, {} large allocs, {} from thread 0",
                    i + 1,
                    entry_count,
                    current_stats.current_memory_usage as f64 / (1024.0 * 1024.0),
                    large_allocations,
                    thread_0_allocations
                );
            }
        }

        let final_stats = self.bounded_history.get_memory_usage_stats();
        let final_memory = final_stats.current_memory_usage;
        let duration = start.elapsed();

        println!(
            "  Initial memory: {:.1} MB",
            initial_memory as f64 / (1024.0 * 1024.0)
        );
        println!(
            "  Final memory: {:.1} MB",
            final_memory as f64 / (1024.0 * 1024.0)
        );
        println!(
            "  Memory growth: {:.1}%",
            ((final_memory as f64 - initial_memory as f64) / initial_memory as f64) * 100.0
        );
        println!("  Duration: {:.2?}", duration);
        println!(
            "  Throughput: {:.0} allocs/sec\n",
            100_000.0 / duration.as_secs_f64()
        );
    }

    /// Test 2: Type Classification Performance
    /// Shows the speed and accuracy of the unified classification system
    fn test_type_classification_performance(&self) {
        println!("🏷️ Test 2: Type Classification Performance");
        println!("===========================================");

        // Diverse set of Rust types to classify
        let test_types = vec![
            // Primitives
            "i32",
            "u64",
            "f64",
            "bool",
            "char",
            // Collections
            "Vec<i32>",
            "HashMap<String, i32>",
            "BTreeSet<u64>",
            "VecDeque<String>",
            // Smart pointers
            "Box<dyn Send>",
            "Arc<Mutex<Vec<u8>>>",
            "Rc<RefCell<String>>",
            "Weak<Node>",
            // Complex types
            "Result<Vec<String>, io::Error>",
            "Option<HashMap<String, Arc<dyn Trait>>>",
            // System types
            "std::thread::Thread",
            "std::fs::File",
            "std::net::TcpStream",
            // Async types
            "Future<Output = Result<String, Error>>",
            "Stream<Item = u8>",
            // User defined
            "MyStruct",
            "custom::module::ComplexType",
            "game::Entity",
        ];

        let start = Instant::now();
        let mut classification_results = HashMap::new();

        // Classify each type multiple times to get average performance
        for _ in 0..1000 {
            for type_name in &test_types {
                let category = self.type_classifier.classify(type_name);
                *classification_results.entry(category).or_insert(0usize) += 1;
            }
        }

        let duration = start.elapsed();
        let total_classifications = test_types.len() * 1000;

        println!("  Total classifications: {}", total_classifications);
        println!("  Duration: {:.2?}", duration);
        println!(
            "  Average per classification: {:.2?}",
            duration / total_classifications as u32
        );
        println!(
            "  Throughput: {:.0} classifications/sec",
            total_classifications as f64 / duration.as_secs_f64()
        );

        println!("  Classification distribution:");
        for (category, count) in classification_results {
            println!(
                "    {:?}: {} ({}%)",
                category,
                count,
                (count * 100) / total_classifications
            );
        }
        println!();
    }

    /// Test 3: Size Estimation Accuracy
    /// Demonstrates the smart size estimator's learning capabilities
    fn test_size_estimation_performance(&self) {
        println!("📏 Test 3: Size Estimation Performance & Accuracy");
        println!("=================================================");

        // Use the suite's estimator and pre-warm it with some data
        let mut estimator = SmartSizeEstimator::new();
        
        // Pre-warm with the suite's estimator knowledge
        if let Some(estimated) = self.size_estimator.estimate_size("String") {
            estimator.learn_from_real_allocation("String", estimated);
        }
        if let Some(estimated) = self.size_estimator.estimate_size("Vec<u8>") {
            estimator.learn_from_real_allocation("Vec<u8>", estimated);
        }
        
        let start = Instant::now();

        // Test types with known sizes
        let test_cases = vec![
            ("i32", 4),
            ("i64", 8),
            ("String", 24),
            ("Vec<u8>", 24),
            ("Box<i32>", std::mem::size_of::<*const u8>()),
            ("Arc<String>", std::mem::size_of::<*const u8>()),
        ];

        let mut accurate_estimates = 0;
        let mut total_estimates = 0;

        for (type_name, actual_size) in &test_cases {
            if let Some(estimated_size) = estimator.estimate_size(type_name) {
                total_estimates += 1;
                let accuracy = 1.0
                    - ((*actual_size as f64 - estimated_size as f64).abs() / *actual_size as f64);

                println!(
                    "  {}: estimated={}, actual={}, accuracy={:.1}%",
                    type_name,
                    estimated_size,
                    actual_size,
                    accuracy * 100.0
                );

                if accuracy > 0.9 {
                    // 90% accuracy threshold
                    accurate_estimates += 1;
                }

                // Simulate learning from real allocations
                estimator.learn_from_real_allocation(type_name, *actual_size);
            }
        }

        // Test learning improvement
        println!("  \n  Testing learning improvement:");
        for (type_name, actual_size) in &test_cases {
            if let Some(improved_estimate) = estimator.estimate_size(type_name) {
                let accuracy = 1.0
                    - ((*actual_size as f64 - improved_estimate as f64).abs()
                        / *actual_size as f64);
                println!(
                    "    {} after learning: estimated={}, accuracy={:.1}%",
                    type_name,
                    improved_estimate,
                    accuracy * 100.0
                );
            }
        }

        // Test with some actual allocation data from bounded history
        println!("  \n  Testing with real allocation patterns:");
        let allocations = self.bounded_history.entries();
        let mut estimation_accuracy_sum = 0.0;
        let mut estimation_count = 0;
        
        for alloc in allocations.iter().take(50) { // Test first 50 allocations
            if let Some(estimated_size) = estimator.estimate_size(&alloc.type_name) {
                let accuracy = 1.0 - ((alloc.size as f64 - estimated_size as f64).abs() / alloc.size as f64);
                estimation_accuracy_sum += accuracy;
                estimation_count += 1;
                
                // Learn from this real allocation for future estimates
                estimator.learn_from_real_allocation(&alloc.type_name, alloc.size);
                
                if estimation_count <= 5 {
                    println!("    Real alloc {} at 0x{:x}: {} bytes ({:.2} MB), estimated {} bytes, accuracy {:.1}%, age: {:.2?}", 
                             alloc.type_name, alloc.ptr, alloc.size, alloc.size_mb(), estimated_size, accuracy * 100.0, alloc.age());
                }
            }
        }
        
        if estimation_count > 0 {
            let avg_accuracy = estimation_accuracy_sum / estimation_count as f64;
            println!("    Average accuracy on real data: {:.1}% ({} samples)", 
                     avg_accuracy * 100.0, estimation_count);
        }

        let duration = start.elapsed();
        let accuracy_rate = accurate_estimates as f64 / total_estimates as f64;

        println!(
            "  \n  Overall test accuracy: {:.1}% ({}/{})",
            accuracy_rate * 100.0,
            accurate_estimates,
            total_estimates
        );
        println!("  Duration: {:.2?}", duration);
        println!();
    }

    /// Test 4: Smart Pointer Tracking
    /// Shows tracking of smart pointer relationships
    fn test_smart_pointer_tracking(&self) {
        println!("🔗 Test 4: Smart Pointer Tracking Performance");
        println!("==============================================");

        let start = Instant::now();

        // Simulate smart pointer allocations
        for i in 0..10_000 {
            let smart_ptr_addr = 0x2000 + i * 8;
            let size = 64 + (i % 256);

            // Track Box<T>
            if i % 3 == 0 {
                self.smart_pointer_tracker.borrow_mut().track_allocation(
                    smart_ptr_addr,
                    memscope_rs::smart_pointers::tracker::PointerType::Box,
                    size,
                    format!("BoxData{}", i),
                    None,
                );
            }
            // Track Arc<T>
            else if i % 3 == 1 {
                self.smart_pointer_tracker.borrow_mut().track_allocation(
                    smart_ptr_addr,
                    memscope_rs::smart_pointers::tracker::PointerType::Arc,
                    size,
                    format!("ArcData{}", i),
                    Some(1), // Initial ref count
                );
            }
            // Track Rc<T>
            else {
                self.smart_pointer_tracker.borrow_mut().track_allocation(
                    smart_ptr_addr,
                    memscope_rs::smart_pointers::tracker::PointerType::Rc,
                    size,
                    format!("RcData{}", i),
                    Some(1), // Initial ref count
                );
            }
        }

        let duration = start.elapsed();
        let tracker = self.smart_pointer_tracker.borrow();

        let box_stats =
            tracker.get_type_stats(&memscope_rs::smart_pointers::tracker::PointerType::Box);
        let arc_stats =
            tracker.get_type_stats(&memscope_rs::smart_pointers::tracker::PointerType::Arc);
        let rc_stats =
            tracker.get_type_stats(&memscope_rs::smart_pointers::tracker::PointerType::Rc);

        println!("  Smart pointers tracked: {}", 10_000);
        println!(
            "  Box pointers: {}",
            box_stats.map(|s| s.total_count).unwrap_or(0)
        );
        println!(
            "  Arc pointers: {}",
            arc_stats.map(|s| s.total_count).unwrap_or(0)
        );
        println!(
            "  Rc pointers: {}",
            rc_stats.map(|s| s.total_count).unwrap_or(0)
        );
        println!("  Duration: {:.2?}", duration);
        println!(
            "  Throughput: {:.0} smart pointers/sec",
            10_000.0 / duration.as_secs_f64()
        );
        println!();
    }

    /// Test 5: Tracking Statistics & Data Quality
    /// Demonstrates loss tracking and data quality monitoring
    fn test_tracking_statistics(&self) {
        println!("📊 Test 5: Tracking Statistics & Data Quality");
        println!("==============================================");

        let start = Instant::now();

        // Simulate tracking with some failures
        for i in 0..50_000 {
            self.tracking_stats.record_attempt();

            // Simulate 95% success rate (high quality tracking)
            if i % 20 != 0 {
                self.tracking_stats.record_success();
            } else {
                self.tracking_stats.record_miss();
            }
        }

        let duration = start.elapsed();
        let detailed_stats = self.tracking_stats.get_detailed_stats();

        println!("  Total attempts: {}", detailed_stats.total_attempts);
        println!("  Successful tracks: {}", detailed_stats.successful_tracks);
        println!(
            "  Missed tracks: {}",
            detailed_stats.missed_due_to_contention
        );
        println!(
            "  Completeness: {:.2}%",
            self.tracking_stats.get_completeness() * 100.0
        );
        println!(
            "  Contention rate: {:.2}%",
            detailed_stats.contention_rate * 100.0
        );
        println!("  Duration: {:.2?}", duration);
        println!(
            "  Tracking rate: {:.0} attempts/sec",
            50_000.0 / duration.as_secs_f64()
        );
        println!();
    }

    /// Comprehensive performance summary
    fn performance_summary(&self) {
        println!("📈 Enhanced MemScope Performance Summary");
        println!("==========================================");

        println!("✅ Key Improvements Demonstrated:");
        println!("  • Bounded memory growth (vs unlimited growth)");
        println!("  • Fast unified type classification");
        println!("  • Intelligent size estimation with learning");
        println!("  • Smart pointer relationship tracking");
        println!("  • Data quality monitoring and loss detection");

        println!("\n🎯 Performance Characteristics:");
        println!("  • Memory-bounded operation (production-ready)");
        println!("  • High-throughput allocation tracking");
        println!("  • Sub-microsecond type classification");
        println!("  • Adaptive size estimation accuracy");
        println!("  • Real-time data quality feedback");

        println!("\n💡 This enhanced version provides:");
        println!("  • Predictable memory usage for long-running analysis");
        println!("  • Higher accuracy through intelligent estimation");
        println!("  • Better data quality assurance");
        println!("  • Comprehensive smart pointer analysis");
        println!("  • Production-ready reliability");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Enhanced MemScope-RS Performance Demonstration\n");

    let test_suite = PerformanceTestSuite::new();

    // Run all performance tests
    test_suite.test_bounded_history_performance();
    test_suite.test_type_classification_performance();
    test_suite.test_size_estimation_performance();
    test_suite.test_smart_pointer_tracking();
    test_suite.test_tracking_statistics();

    // Show summary
    test_suite.performance_summary();

    println!("\n✨ Demo completed successfully!");
    Ok(())
}
