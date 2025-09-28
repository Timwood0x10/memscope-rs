//! Async-Focused Dispatch Strategy Benchmark
//!
//! Dedicated async environment testing for track_var! intelligent dispatch.
//! Tests various async patterns and concurrency levels.
//!
//! Run with: cargo run --example async_dispatch_benchmark

use futures::future::join_all;
use memscope_rs::{init, track_var};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Async Dispatch Strategy Benchmark");
    println!("Testing track_var! performance in pure async environments");
    println!();

    init();

    // Test different async concurrency levels
    let concurrency_levels = vec![10, 25, 50, 100, 200];

    for &level in &concurrency_levels {
        println!("📊 Testing {} concurrent async tasks", level);

        let start = Instant::now();
        let operations = run_async_benchmark(level).await?;
        let duration = start.elapsed();

        println!(
            "   ✅ Completed {} operations in {:?}",
            operations, duration
        );
        println!(
            "   🚀 Rate: {:.0} ops/sec",
            operations as f64 / duration.as_secs_f64()
        );
        println!();
    }

    // Test async patterns
    println!("🔄 Testing different async patterns:");
    test_async_patterns().await?;

    // Test completed
    println!("🎉 Async benchmark completed!");
    Ok(())
}

async fn run_async_benchmark(concurrency: usize) -> Result<usize, String> {
    let operations_counter = Arc::new(AtomicUsize::new(0));
    let semaphore = Arc::new(Semaphore::new(concurrency));

    let mut tasks = Vec::new();

    // Spawn many concurrent tasks
    for task_id in 0..concurrency * 2 {
        let sem = Arc::clone(&semaphore);
        let counter = Arc::clone(&operations_counter);

        let task = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            async_worker_task(task_id, 100, counter).await
        });

        tasks.push(task);
    }

    // Wait for all tasks
    for task in tasks {
        match task.await {
            Ok(result) => result?,
            Err(_) => return Err("Task failed".to_string()),
        }
    }

    Ok(operations_counter.load(Ordering::Relaxed))
}

async fn async_worker_task(
    task_id: usize,
    operations: usize,
    counter: Arc<AtomicUsize>,
) -> Result<(), String> {
    for i in 0..operations {
        // Different async allocation patterns
        match i % 6 {
            0 => {
                let data = create_async_vec(task_id, i).await;
                track_var!(data);
            }
            1 => {
                let result = async_computation_heavy(task_id, i).await;
                track_var!(result);
            }
            2 => {
                let stream_data = simulate_stream_processing(task_id, i).await;
                track_var!(stream_data.0);
                track_var!(stream_data.1);
            }
            3 => {
                let channel_result = async_channel_operation(task_id, i).await;
                track_var!(channel_result);
            }
            4 => {
                let shared_data = create_shared_async_data(task_id, i).await;
                track_var!(shared_data);
            }
            5 => {
                let nested = deeply_nested_async(task_id, i).await;
                track_var!(nested.strings);
                track_var!(nested.numbers);
            }
            _ => unreachable!(),
        }

        counter.fetch_add(1, Ordering::Relaxed);

        // Occasional yields for realistic async behavior
        if i % 20 == 0 {
            tokio::task::yield_now().await;
        }
    }

    Ok(())
}

// Async helper functions for different patterns
async fn create_async_vec(task_id: usize, iteration: usize) -> Vec<String> {
    tokio::time::sleep(Duration::from_micros(1)).await;
    (0..20)
        .map(|i| format!("item_{}_{}_{})", task_id, iteration, i))
        .collect()
}

async fn async_computation_heavy(task_id: usize, iteration: usize) -> Vec<f64> {
    tokio::time::sleep(Duration::from_micros(5)).await;
    (0..50)
        .map(|i| (task_id * iteration + i) as f64 * 1.414)
        .collect()
}

async fn simulate_stream_processing(task_id: usize, iteration: usize) -> (Vec<u8>, String) {
    tokio::time::sleep(Duration::from_micros(2)).await;
    let data = format!("stream_data_{}_{}", task_id, iteration).into_bytes();
    let metadata = format!("metadata_{}_{}", task_id, iteration);
    (data, metadata)
}

async fn async_channel_operation(task_id: usize, iteration: usize) -> Arc<Vec<usize>> {
    tokio::time::sleep(Duration::from_micros(1)).await;
    Arc::new(
        (0..30)
            .map(|i| task_id * 1000 + iteration * 10 + i)
            .collect(),
    )
}

async fn create_shared_async_data(task_id: usize, iteration: usize) -> Arc<RwLock<String>> {
    tokio::time::sleep(Duration::from_micros(1)).await;
    Arc::new(RwLock::new(format!("shared_{}_{}", task_id, iteration)))
}

async fn deeply_nested_async(task_id: usize, iteration: usize) -> NestedAsyncData {
    let level1 = create_async_vec(task_id, iteration).await;
    let level2 = async_computation_heavy(task_id, iteration).await;
    let level3 = simulate_stream_processing(task_id, iteration).await;

    NestedAsyncData {
        id: task_id * 10000 + iteration,
        strings: level1,
        numbers: level2,
        binary: level3.0,
        metadata: level3.1,
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct NestedAsyncData {
    id: usize,
    strings: Vec<String>,
    numbers: Vec<f64>,
    binary: Vec<u8>,
    metadata: String,
}

async fn test_async_patterns() -> Result<(), String> {
    println!("   🔄 Producer-Consumer Pattern");
    test_producer_consumer().await.map_err(|e| e.to_string())?;

    println!("   🔄 Fan-out Pattern");
    test_fan_out().await.map_err(|e| e.to_string())?;

    println!("   🔄 Pipeline Pattern");
    test_pipeline().await.map_err(|e| e.to_string())?;

    Ok(())
}

async fn test_producer_consumer() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    // Producer
    let producer = tokio::spawn(async move {
        for i in 0..50 {
            let data = format!("produced_item_{}", i);
            track_var!(data);
            tx.send(i).await.unwrap();
            tokio::time::sleep(Duration::from_micros(10)).await;
        }
    });

    // Consumer
    let consumer = tokio::spawn(async move {
        let mut received = Vec::new();
        while let Some(item) = rx.recv().await {
            received.push(item);
            track_var!(received);
            if received.len() >= 50 {
                break;
            }
        }
    });

    tokio::try_join!(producer, consumer).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(())
}

async fn test_fan_out() -> Result<(), Box<dyn std::error::Error>> {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    let mut tasks = JoinSet::new();

    for i in 0..10 {
        let data_clone = Arc::clone(&data);
        tasks.spawn(async move {
            let processed = data_clone.iter().map(|x| x * i).collect::<Vec<_>>();
            track_var!(processed);
        });
    }

    while let Some(_) = tasks.join_next().await {}
    Ok(())
}

async fn test_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let input_data: Vec<i32> = (0..20).collect();
    track_var!(input_data);

    // Stage 1: Transform
    let stage1_results = join_all(input_data.into_iter().map(|x| async move {
        tokio::time::sleep(Duration::from_micros(1)).await;
        x * 2
    }))
    .await;
    track_var!(stage1_results);

    // Stage 2: Filter and collect
    let stage2_results = join_all(stage1_results.into_iter().map(|x| async move {
        tokio::time::sleep(Duration::from_micros(1)).await;
        if x % 4 == 0 {
            Some(x)
        } else {
            None
        }
    }))
    .await;

    let final_results: Vec<i32> = stage2_results.into_iter().flatten().collect();
    track_var!(final_results);

    Ok(())
}
