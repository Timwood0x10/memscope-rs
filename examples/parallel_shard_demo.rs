//! 并行分片处理器演示
//! 
//! 这个示例展示了如何使用并行分片处理器来加速大量分配数据的 JSON 序列化。

use memscope_rs::export::data_localizer::DataLocalizer;
use memscope_rs::export::parallel_shard_processor::{ParallelShardConfig, ParallelShardProcessor};
use memscope_rs::{init, track_var};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化内存跟踪
    init();
    
    println!("🚀 并行分片处理器演示");
    println!("====================");
    
    // 创建大量测试数据
    create_large_test_dataset();
    
    // 演示并行分片处理的性能优势
    demonstrate_parallel_processing()?;
    
    Ok(())
}

fn create_large_test_dataset() {
    println!("\n📦 创建大型测试数据集...");
    
    // 创建大量不同类型的分配
    let mut large_vectors = Vec::new();
    for i in 0..500 {
        let vec = vec![i; 100 + (i % 50)];
        track_var!(vec);
        large_vectors.push(vec);
    }
    
    let mut strings = Vec::new();
    for i in 0..300 {
        let string = format!("Large string data item {} with some content", i);
        track_var!(string);
        strings.push(string);
    }
    
    let mut hash_maps = Vec::new();
    for i in 0..200 {
        let mut map = std::collections::HashMap::new();
        for j in 0..20 {
            map.insert(format!("key_{}_{}", i, j), j * i);
        }
        track_var!(map);
        hash_maps.push(map);
    }
    
    println!("   ✅ 创建了大量测试分配（预计 >2000 个）");
    
    // 保持数据存活
    std::mem::forget(large_vectors);
    std::mem::forget(strings);
    std::mem::forget(hash_maps);
}

fn demonstrate_parallel_processing() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 演示并行分片处理性能...");
    
    // 获取本地化数据
    let mut localizer = DataLocalizer::new();
    let (data, _) = localizer.gather_all_export_data()?;
    
    println!("获取到 {} 个分配进行处理", data.allocations.len());
    
    // 测试不同的配置
    test_sequential_processing(&data)?;
    test_parallel_processing(&data)?;
    test_custom_configurations(&data)?;
    
    Ok(())
}

fn test_sequential_processing(data: &memscope_rs::export::data_localizer::LocalizedExportData) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 串行处理测试 ---");
    
    let config = ParallelShardConfig {
        shard_size: 1000,
        parallel_threshold: 999999, // 设置很高的阈值，强制串行处理
        max_threads: Some(1),
        enable_monitoring: true,
        estimated_json_size_per_allocation: 200,
    };
    
    let processor = ParallelShardProcessor::new(config);
    let start_time = Instant::now();
    let (shards, stats) = processor.process_allocations_parallel(data)?;
    let total_time = start_time.elapsed();
    
    println!("串行处理结果:");
    println!("   分片数量: {}", shards.len());
    println!("   总耗时: {:?}", total_time);
    println!("   吞吐量: {:.0} 分配/秒", stats.throughput_allocations_per_sec);
    println!("   输出大小: {:.2} MB", stats.total_output_size_bytes as f64 / 1024.0 / 1024.0);
    
    Ok(())
}

fn test_parallel_processing(data: &memscope_rs::export::data_localizer::LocalizedExportData) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 并行处理测试 ---");
    
    let config = ParallelShardConfig {
        shard_size: 500,
        parallel_threshold: 1000, // 低阈值，启用并行处理
        max_threads: None, // 使用系统默认线程数
        enable_monitoring: true,
        estimated_json_size_per_allocation: 200,
    };
    
    let processor = ParallelShardProcessor::new(config);
    let start_time = Instant::now();
    let (shards, stats) = processor.process_allocations_parallel(data)?;
    let total_time = start_time.elapsed();
    
    println!("并行处理结果:");
    println!("   分片数量: {}", shards.len());
    println!("   使用线程: {}", stats.threads_used);
    println!("   总耗时: {:?}", total_time);
    println!("   吞吐量: {:.0} 分配/秒", stats.throughput_allocations_per_sec);
    println!("   并行效率: {:.1}%", stats.parallel_efficiency * 100.0);
    println!("   输出大小: {:.2} MB", stats.total_output_size_bytes as f64 / 1024.0 / 1024.0);
    
    Ok(())
}

fn test_custom_configurations(data: &memscope_rs::export::data_localizer::LocalizedExportData) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 自定义配置测试 ---");
    
    // 测试小分片配置
    println!("\n小分片配置 (分片大小: 200):");
    let small_shard_config = ParallelShardConfig {
        shard_size: 200,
        parallel_threshold: 500,
        max_threads: Some(4),
        enable_monitoring: false, // 关闭监控以减少输出
        estimated_json_size_per_allocation: 200,
    };
    
    let processor = ParallelShardProcessor::new(small_shard_config);
    let start_time = Instant::now();
    let (shards, stats) = processor.process_allocations_parallel(data)?;
    let small_shard_time = start_time.elapsed();
    
    println!("   分片数量: {}", shards.len());
    println!("   总耗时: {:?}", small_shard_time);
    println!("   平均分片耗时: {:.2}ms", stats.avg_shard_processing_time_ms);
    
    // 测试大分片配置
    println!("\n大分片配置 (分片大小: 2000):");
    let large_shard_config = ParallelShardConfig {
        shard_size: 2000,
        parallel_threshold: 1000,
        max_threads: Some(2),
        enable_monitoring: false,
        estimated_json_size_per_allocation: 200,
    };
    
    let processor = ParallelShardProcessor::new(large_shard_config);
    let start_time = Instant::now();
    let (shards, stats) = processor.process_allocations_parallel(data)?;
    let large_shard_time = start_time.elapsed();
    
    println!("   分片数量: {}", shards.len());
    println!("   总耗时: {:?}", large_shard_time);
    println!("   平均分片耗时: {:.2}ms", stats.avg_shard_processing_time_ms);
    
    // 性能对比
    println!("\n📊 配置对比:");
    println!("   小分片 vs 大分片: {:?} vs {:?}", small_shard_time, large_shard_time);
    if small_shard_time < large_shard_time {
        let speedup = large_shard_time.as_millis() as f64 / small_shard_time.as_millis() as f64;
        println!("   小分片配置快 {:.2}x", speedup);
    } else {
        let speedup = small_shard_time.as_millis() as f64 / large_shard_time.as_millis() as f64;
        println!("   大分片配置快 {:.2}x", speedup);
    }
    
    Ok(())
}