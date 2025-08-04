//! Example demonstrating enhanced type usage data collection

use memscope_rs::{get_global_tracker, init};
use std::collections::HashMap;
use std::f32::consts::PI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize memory tracking
    init();

    println!("🔍 Type Usage Analysis Demo");
    println!("===========================");

    // Create various types of allocations to test type usage tracking
    create_test_allocations();

    // Get the tracker and analyze type usage
    let tracker = get_global_tracker();

    // Test type usage tracking for different types
    test_type_usage_analysis(&tracker, "Vec<i32>")?;
    test_type_usage_analysis(&tracker, "String")?;
    test_type_usage_analysis(&tracker, "HashMap<String, i32>")?;
    test_type_usage_analysis(&tracker, "Box<f64>")?;

    println!("\n✅ Type usage analysis demo completed!");
    Ok(())
}

fn create_test_allocations() {
    println!("\n📊 Creating test allocations for type analysis...");

    // Create multiple Vec allocations
    let _vec1: Vec<i32> = vec![1, 2, 3];
    let _vec2: Vec<i32> = Vec::with_capacity(100);
    let _vec3: Vec<i32> = (0..50).collect();

    // Create multiple String allocations
    let _str1 = String::from("Hello");
    let _str2 = String::with_capacity(200);
    let _str3 = "World".to_string();
    let _str4 = format!("Test string {}", 42);

    // Create HashMap allocations
    let mut _map1: HashMap<String, i32> = HashMap::new();
    _map1.insert("key1".to_string(), 1);
    _map1.insert("key2".to_string(), 2);

    let _map2: HashMap<String, i32> = HashMap::with_capacity(50);

    // Create Box allocations
    let _box1 = Box::new(PI);
    let _box2 = Box::new(2.71);

    println!("   Created various allocations for type analysis");
}

fn test_type_usage_analysis(
    tracker: &memscope_rs::core::tracker::MemoryTracker,
    type_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔬 Analyzing type usage for: {}", type_name);
    println!("{}", "-".repeat(40 + type_name.len()));

    if let Some(type_usage) = tracker.track_type_usage(type_name) {
        println!("   Type: {}", type_usage.type_name);
        println!("   Total usage count: {}", type_usage.total_usage_count);

        // Display usage contexts
        if !type_usage.usage_contexts.is_empty() {
            println!("   Usage contexts:");
            for context in &type_usage.usage_contexts {
                println!("     - Context: {:?}", context.context_type);
                println!("       Frequency: {}", context.frequency);
                println!(
                    "       Avg execution time: {:.2} ns",
                    context.performance_metrics.avg_execution_time_ns
                );
                println!(
                    "       Allocation frequency: {:.2}",
                    context.performance_metrics.allocation_frequency
                );
                println!(
                    "       Cache miss rate: {:.1}%",
                    context.performance_metrics.cache_miss_rate * 100.0
                );
            }
        }

        // Display usage timeline
        if !type_usage.usage_timeline.is_empty() {
            println!(
                "   Usage timeline ({} data points):",
                type_usage.usage_timeline.len()
            );
            for (i, point) in type_usage.usage_timeline.iter().take(3).enumerate() {
                println!(
                    "     {}. Time: {}, Usage: {}, Memory: {} bytes",
                    i + 1,
                    point.timestamp,
                    point.usage_count,
                    point.memory_usage
                );
            }
            if type_usage.usage_timeline.len() > 3 {
                println!(
                    "     ... and {} more data points",
                    type_usage.usage_timeline.len() - 3
                );
            }
        }

        // Display performance impact
        let perf = &type_usage.performance_impact;
        println!("   Performance Impact:");
        println!("     Overall score: {:.1}/100", perf.performance_score);
        println!(
            "     Memory efficiency: {:.1}/100",
            perf.memory_efficiency_score
        );
        println!("     CPU efficiency: {:.1}/100", perf.cpu_efficiency_score);
        println!(
            "     Cache efficiency: {:.1}/100",
            perf.cache_efficiency_score
        );

        // Display optimization recommendations
        if !perf.optimization_recommendations.is_empty() {
            println!("   Optimization Recommendations:");
            for (i, rec) in perf.optimization_recommendations.iter().enumerate() {
                println!(
                    "     {}. [{}] {:?}",
                    i + 1,
                    format_priority(&rec.priority),
                    rec.recommendation_type
                );
                println!("        {}", rec.description);
                println!(
                    "        Expected improvement: {:.1}%",
                    rec.expected_improvement
                );
                println!("        Difficulty: {:?}", rec.implementation_difficulty);
            }
        }

        println!("   ✅ Analysis completed for {}", type_name);
    } else {
        println!("   ❌ No type usage data available for {}", type_name);
    }

    Ok(())
}

fn format_priority(priority: &memscope_rs::core::types::Priority) -> &'static str {
    match priority {
        memscope_rs::core::types::Priority::Low => "LOW",
        memscope_rs::core::types::Priority::Medium => "MED",
        memscope_rs::core::types::Priority::High => "HIGH",
        memscope_rs::core::types::Priority::Critical => "CRIT",
    }
}
