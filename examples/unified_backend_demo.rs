//! Unified Backend Demo
//!
//! This example demonstrates the key features of the MemScope Unified Backend:
//! - Automatic environment detection
//! - Strategy selection and switching
//! - Multi-context memory tracking (sync, async, multi-threaded)
//! - Data collection and export

use memscope_rs::unified::{
    detect_environment, test_unified_system, BackendConfig, TrackingStrategy, UnifiedBackend,
};
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 MemScope Unified Backend Demo");
    println!("================================");

    // Demo 1: Quick Start
    demo_quick_start()?;

    // Demo 2: Environment Detection
    demo_environment_detection()?;

    // Demo 3: Manual Strategy Configuration
    demo_manual_strategy().await?;

    // Demo 4: Multi-threaded Tracking
    demo_multi_threaded_tracking()?;

    // Demo 5: Async Tracking
    demo_async_tracking().await?;

    // Demo 6: System Integration Test
    demo_system_integration()?;

    println!("\n🎉 All demos completed successfully!");
    Ok(())
}

/// Demo 1: Quick Start - Simplest way to use unified backend
fn demo_quick_start() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 1: Quick Start");
    println!("---------------------");

    // Quick start for immediate use
    let mut backend = UnifiedBackend::initialize(BackendConfig::default())?;
    println!("✅ Unified backend initialized with quick start");

    // Start tracking session
    let session = backend.start_tracking()?;
    println!("✅ Tracking session started: {}", session.session_id());

    // Simulate some memory operations
    let data = [1, 2, 3, 4, 5];
    let processed: Vec<i32> = data.iter().map(|x| x * 2).collect();
    let sum: i32 = processed.iter().sum();

    println!("📊 Processed data: {:?}, Sum: {}", processed, sum);

    // Collect results
    let analysis = backend.collect_data()?;
    println!(
        "✅ Analysis completed: {} bytes collected",
        analysis.raw_data.len()
    );

    Ok(())
}

/// Demo 2: Environment Detection - Show automatic environment analysis
fn demo_environment_detection() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🌍 Demo 2: Environment Detection");
    println!("--------------------------------");

    // Detect current environment
    let environment = detect_environment()?;
    println!("🔍 Detected environment: {:?}", environment);

    // Show environment-specific recommendations
    match environment {
        memscope_rs::unified::RuntimeEnvironment::SingleThreaded => {
            println!("💡 Recommendation: Use single-thread strategy for optimal performance");
        }
        memscope_rs::unified::RuntimeEnvironment::MultiThreaded { thread_count: _ } => {
            println!("💡 Recommendation: Use thread-local strategy for better scaling");
        }
        memscope_rs::unified::RuntimeEnvironment::AsyncRuntime { runtime_type: _ } => {
            println!("💡 Recommendation: Use async strategy for task-aware tracking");
        }
        memscope_rs::unified::RuntimeEnvironment::Hybrid {
            thread_count: _,
            async_task_count: _,
        } => {
            println!("💡 Recommendation: Use hybrid strategy for adaptive behavior");
        }
    }

    Ok(())
}

/// Demo 3: Manual Strategy Configuration - Custom backend setup
async fn demo_manual_strategy() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚙️  Demo 3: Manual Strategy Configuration");
    println!("---------------------------------------");

    // Configure backend for async workload
    let config = BackendConfig {
        auto_detect: false,
        force_strategy: Some(TrackingStrategy::GlobalDirect),
        sample_rate: 0.8, // 80% sampling for performance
        max_overhead_percent: 3.0,
    };

    println!("🔧 Configuration: Strategy=GlobalDirect, SampleRate=80%, MaxOverhead=3%");

    let mut backend = UnifiedBackend::initialize(config)?;
    let session = backend.start_tracking()?;

    println!(
        "✅ Custom backend initialized with session: {}",
        session.session_id()
    );

    // Simulate async workload
    let result = async_workload().await;
    println!("📊 Async workload result: {}", result);

    let analysis = backend.collect_data()?;
    println!(
        "✅ Async analysis: {} bytes collected",
        analysis.raw_data.len()
    );

    Ok(())
}

/// Demo 4: Multi-threaded Tracking - Show thread-local strategy
fn demo_multi_threaded_tracking() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧵 Demo 4: Multi-threaded Tracking");
    println!("---------------------------------");

    let config = BackendConfig {
        auto_detect: false,
        force_strategy: Some(TrackingStrategy::ThreadLocal),
        sample_rate: 1.0,
        max_overhead_percent: 5.0,
    };

    let mut backend = UnifiedBackend::initialize(config)?;
    let session = backend.start_tracking()?;

    println!("✅ Thread-local backend started: {}", session.session_id());

    // Launch multiple threads
    let handles: Vec<_> = (0..4)
        .map(|thread_id| {
            thread::spawn(move || {
                // Simulate thread-specific work
                let data = vec![thread_id; 100];
                thread::sleep(Duration::from_millis(50));
                data.into_iter().sum::<usize>()
            })
        })
        .collect();

    // Collect results from all threads
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    println!("📊 Thread results: {:?}", results);

    let analysis = backend.collect_data()?;
    println!(
        "✅ Multi-threaded analysis: {} bytes collected",
        analysis.raw_data.len()
    );

    Ok(())
}

/// Demo 5: Async Tracking - Demonstrate async-aware memory tracking
async fn demo_async_tracking() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚡ Demo 5: Async Tracking");
    println!("-----------------------");

    let config = BackendConfig {
        auto_detect: false,
        force_strategy: Some(TrackingStrategy::GlobalDirect),
        sample_rate: 1.0,
        max_overhead_percent: 5.0,
    };

    let mut backend = UnifiedBackend::initialize(config)?;
    let session = backend.start_tracking()?;

    println!("✅ Async backend started: {}", session.session_id());

    // Create multiple async tasks
    let tasks = (0..5).map(|task_id| {
        tokio::spawn(async move {
            // Simulate async work with memory allocation
            let data = vec![task_id; 200];
            tokio::time::sleep(Duration::from_millis(20)).await;

            // Nested async operation
            let processed = process_data_async(data).await;
            processed.len()
        })
    });

    // Wait for all tasks to complete
    let results = futures::future::try_join_all(tasks).await?;
    println!("📊 Async task results: {:?}", results);

    let analysis = backend.collect_data()?;
    println!(
        "✅ Async analysis: {} bytes collected",
        analysis.raw_data.len()
    );

    Ok(())
}

/// Demo 6: System Integration Test - Use built-in test system
fn demo_system_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔧 Demo 6: System Integration Test");
    println!("----------------------------------");

    // Run the built-in unified system test
    println!("🧪 Running unified system integration test...");
    test_unified_system()?;
    println!("✅ System integration test passed!");

    Ok(())
}

/// Helper function: Simulate async workload
async fn async_workload() -> usize {
    let mut total = 0;

    for i in 0..3 {
        let data = [i; 50];
        tokio::time::sleep(Duration::from_millis(10)).await;
        total += data.len();
    }

    total
}

/// Helper function: Process data asynchronously
async fn process_data_async(data: Vec<usize>) -> Vec<usize> {
    tokio::time::sleep(Duration::from_millis(5)).await;
    data.into_iter().map(|x| x * 2).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_backend_demo() {
        // Test that all demo functions work
        assert!(demo_quick_start().is_ok());
        assert!(demo_environment_detection().is_ok());
        assert!(demo_manual_strategy().await.is_ok());
        assert!(demo_multi_threaded_tracking().is_ok());
        assert!(demo_async_tracking().await.is_ok());
        assert!(demo_system_integration().is_ok());
    }

    #[test]
    fn test_backend_configuration() {
        let config = BackendConfig {
            auto_detect: true,
            force_strategy: None,
            sample_rate: 0.5,
            max_overhead_percent: 2.0,
        };

        let backend = UnifiedBackend::initialize(config);
        assert!(backend.is_ok());
    }

    #[tokio::test]
    async fn test_async_workload() {
        let result = async_workload().await;
        assert!(result > 0);
    }
}
