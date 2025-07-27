//! 并行分片处理器 - 高性能并行 JSON 序列化
//!
//! 这个模块实现了并行分片处理功能，将大量分配数据分片后并行处理，
//! 显著提高 JSON 序列化的性能，特别是在多核系统上。

use crate::core::types::{AllocationInfo, TrackingError, TrackingResult};
use crate::export::data_localizer::LocalizedExportData;
use rayon::prelude::*;
use serde_json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// 并行分片处理器配置
#[derive(Debug, Clone)]
pub struct ParallelShardConfig {
    /// 每个分片的大小（分配数量）
    pub shard_size: usize,
    /// 并行处理的阈值（超过此数量才启用并行）
    pub parallel_threshold: usize,
    /// 最大线程数（None 表示使用系统默认）
    pub max_threads: Option<usize>,
    /// 是否启用性能监控
    pub enable_monitoring: bool,
    /// 预估每个分配的 JSON 大小（用于预分配）
    pub estimated_json_size_per_allocation: usize,
}

impl Default for ParallelShardConfig {
    fn default() -> Self {
        Self {
            shard_size: 1000,                    // 每个分片 1000 个分配
            parallel_threshold: 2000,            // 超过 2000 个分配才并行
            max_threads: None,                   // 使用系统默认线程数
            enable_monitoring: true,             // 启用性能监控
            estimated_json_size_per_allocation: 200, // 每个分配约 200 字节 JSON
        }
    }
}

/// 处理后的分片数据
#[derive(Debug, Clone)]
pub struct ProcessedShard {
    /// 序列化后的 JSON 数据
    pub data: Vec<u8>,
    /// 分片中的分配数量
    pub allocation_count: usize,
    /// 分片索引
    pub shard_index: usize,
    /// 处理耗时（毫秒）
    pub processing_time_ms: u64,
}

/// 并行处理统计信息
#[derive(Debug, Clone)]
pub struct ParallelProcessingStats {
    /// 总分配数量
    pub total_allocations: usize,
    /// 分片数量
    pub shard_count: usize,
    /// 使用的线程数
    pub threads_used: usize,
    /// 总处理时间（毫秒）
    pub total_processing_time_ms: u64,
    /// 平均每个分片的处理时间（毫秒）
    pub avg_shard_processing_time_ms: f64,
    /// 并行效率（相对于单线程的加速比）
    pub parallel_efficiency: f64,
    /// 吞吐量（分配/秒）
    pub throughput_allocations_per_sec: f64,
    /// 是否使用了并行处理
    pub used_parallel_processing: bool,
    /// 总输出大小（字节）
    pub total_output_size_bytes: usize,
}

/// 并行分片处理器
pub struct ParallelShardProcessor {
    /// 配置
    config: ParallelShardConfig,
    /// 处理计数器（用于监控）
    processed_count: AtomicUsize,
}

impl ParallelShardProcessor {
    /// 创建新的并行分片处理器
    pub fn new(config: ParallelShardConfig) -> Self {
        // 如果指定了最大线程数，设置 rayon 线程池
        if let Some(max_threads) = config.max_threads {
            rayon::ThreadPoolBuilder::new()
                .num_threads(max_threads)
                .build_global()
                .unwrap_or_else(|e| {
                    eprintln!("⚠️ 无法设置线程池大小为 {}: {}", max_threads, e);
                });
        }

        Self {
            config,
            processed_count: AtomicUsize::new(0),
        }
    }

    /// 并行处理分配数据
    pub fn process_allocations_parallel(
        &self,
        data: &LocalizedExportData,
    ) -> TrackingResult<(Vec<ProcessedShard>, ParallelProcessingStats)> {
        let start_time = Instant::now();
        let allocations = &data.allocations;

        println!(
            "🔄 Starting parallel shard processing for {} allocations...",
            allocations.len()
        );

        // 决定是否使用并行处理
        let use_parallel = allocations.len() >= self.config.parallel_threshold;
        let actual_threads = if use_parallel {
            rayon::current_num_threads()
        } else {
            1
        };

        println!(
            "   Parallel mode: {}, threads: {}, shard size: {}",
            if use_parallel { "enabled" } else { "disabled" },
            actual_threads,
            self.config.shard_size
        );

        // 重置计数器
        self.processed_count.store(0, Ordering::Relaxed);

        // 将数据分片
        let shards: Vec<&[AllocationInfo]> = allocations
            .chunks(self.config.shard_size)
            .collect();

        println!("   Shard count: {}", shards.len());

        // 并行或串行处理分片
        let processed_shards: TrackingResult<Vec<ProcessedShard>> = if use_parallel {
            shards
                .into_par_iter()
                .enumerate()
                .map(|(index, shard)| self.process_shard_optimized(shard, index))
                .collect()
        } else {
            shards
                .into_iter()
                .enumerate()
                .map(|(index, shard)| self.process_shard_optimized(shard, index))
                .collect()
        };

        let processed_shards = processed_shards?;
        let total_time = start_time.elapsed();

        // 计算统计信息
        let stats = self.calculate_processing_stats(
            &processed_shards,
            allocations.len(),
            actual_threads,
            total_time.as_millis() as u64,
            use_parallel,
        );

        // 打印性能统计
        self.print_performance_stats(&stats);

        Ok((processed_shards, stats))
    }

    /// 优化的分片处理方法
    fn process_shard_optimized(
        &self,
        shard: &[AllocationInfo],
        shard_index: usize,
    ) -> TrackingResult<ProcessedShard> {
        let shard_start = Instant::now();

        // 预估输出大小并预分配缓冲区
        let estimated_size = shard.len() * self.config.estimated_json_size_per_allocation;
        let mut output_buffer = Vec::with_capacity(estimated_size);

        // 使用 serde_json 的高效 API 直接序列化到字节向量
        // 这比手动格式化更可靠，性能也很好
        serde_json::to_writer(&mut output_buffer, shard)
            .map_err(|e| TrackingError::ExportError(format!("分片 {} 序列化失败: {}", shard_index, e)))?;

        let processing_time = shard_start.elapsed();

        // 更新处理计数器
        self.processed_count.fetch_add(shard.len(), Ordering::Relaxed);

        // 如果启用监控，打印进度
        if self.config.enable_monitoring && shard_index % 10 == 0 {
            let _processed = self.processed_count.load(Ordering::Relaxed);
            println!(
                "   Shard {} completed: {} allocations, {} bytes, {:?}",
                shard_index,
                shard.len(),
                output_buffer.len(),
                processing_time
            );
        }

        Ok(ProcessedShard {
            data: output_buffer,
            allocation_count: shard.len(),
            shard_index,
            processing_time_ms: processing_time.as_millis() as u64,
        })
    }

    /// 计算处理统计信息
    fn calculate_processing_stats(
        &self,
        shards: &[ProcessedShard],
        total_allocations: usize,
        threads_used: usize,
        total_time_ms: u64,
        used_parallel: bool,
    ) -> ParallelProcessingStats {
        let total_output_size: usize = shards.iter().map(|s| s.data.len()).sum();
        let avg_shard_time: f64 = if !shards.is_empty() {
            shards.iter().map(|s| s.processing_time_ms as f64).sum::<f64>() / shards.len() as f64
        } else {
            0.0
        };

        let throughput = if total_time_ms > 0 {
            (total_allocations as f64 * 1000.0) / total_time_ms as f64
        } else {
            0.0
        };

        // 估算并行效率（简化计算）
        let parallel_efficiency = if used_parallel && threads_used > 1 {
            // 理想情况下，N 个线程应该提供接近 N 倍的性能
            // 实际效率 = 实际加速比 / 理论加速比
            let theoretical_speedup = threads_used as f64;
            let estimated_sequential_time = avg_shard_time * shards.len() as f64;
            let actual_speedup = if total_time_ms > 0 {
                estimated_sequential_time / total_time_ms as f64
            } else {
                1.0
            };
            (actual_speedup / theoretical_speedup).min(1.0)
        } else {
            1.0 // 单线程效率为 100%
        };

        ParallelProcessingStats {
            total_allocations,
            shard_count: shards.len(),
            threads_used,
            total_processing_time_ms: total_time_ms,
            avg_shard_processing_time_ms: avg_shard_time,
            parallel_efficiency,
            throughput_allocations_per_sec: throughput,
            used_parallel_processing: used_parallel,
            total_output_size_bytes: total_output_size,
        }
    }

    /// 打印性能统计信息
    fn print_performance_stats(&self, stats: &ParallelProcessingStats) {
        println!("✅ Parallel shard processing completed:");
        println!("   Total allocations: {}", stats.total_allocations);
        println!("   Shard count: {}", stats.shard_count);
        println!("   Threads used: {}", stats.threads_used);
        println!("   Total time: {}ms", stats.total_processing_time_ms);
        println!("   Average shard time: {:.2}ms", stats.avg_shard_processing_time_ms);
        println!("   Throughput: {:.0} allocations/sec", stats.throughput_allocations_per_sec);
        println!("   Output size: {:.2} MB", stats.total_output_size_bytes as f64 / 1024.0 / 1024.0);
        
        if stats.used_parallel_processing {
            println!("   Parallel efficiency: {:.1}%", stats.parallel_efficiency * 100.0);
            let speedup = stats.parallel_efficiency * stats.threads_used as f64;
            println!("   Actual speedup: {:.2}x", speedup);
        }
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &ParallelShardConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: ParallelShardConfig) {
        self.config = config;
    }

    /// 获取处理进度
    pub fn get_processed_count(&self) -> usize {
        self.processed_count.load(Ordering::Relaxed)
    }
}

impl Default for ParallelShardProcessor {
    fn default() -> Self {
        Self::new(ParallelShardConfig::default())
    }
}

/// 便利函数：快速并行处理分配数据
pub fn process_allocations_fast(
    data: &LocalizedExportData,
) -> TrackingResult<(Vec<ProcessedShard>, ParallelProcessingStats)> {
    let processor = ParallelShardProcessor::default();
    processor.process_allocations_parallel(data)
}

/// 便利函数：使用自定义配置并行处理
pub fn process_allocations_with_config(
    data: &LocalizedExportData,
    config: ParallelShardConfig,
) -> TrackingResult<(Vec<ProcessedShard>, ParallelProcessingStats)> {
    let processor = ParallelShardProcessor::new(config);
    processor.process_allocations_parallel(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{MemoryStats, ScopeInfo};
    use crate::analysis::unsafe_ffi_tracker::UnsafeFFIStats;
    use std::time::Instant;

    fn create_test_data(allocation_count: usize) -> LocalizedExportData {
        let mut allocations = Vec::new();
        for i in 0..allocation_count {
            allocations.push(AllocationInfo {
                ptr: 0x1000 + i,
                size: 64 + (i % 100),
                type_name: Some(format!("TestType{}", i % 10)),
                var_name: Some(format!("var_{}", i)),
                scope_name: Some(format!("scope_{}", i % 5)),
                timestamp_alloc: 1000000 + i as u64,
                timestamp_dealloc: None,
                thread_id: format!("test_thread_{}", i % 3),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: None,
                smart_pointer_info: None,
                memory_layout: None,
                generic_info: None,
                dynamic_type_info: None,
                runtime_state: None,
                stack_allocation: None,
                temporary_object: None,
                fragmentation_analysis: None,
                generic_instantiation: None,
                type_relationships: None,
                type_usage: None,
                function_call_tracking: None,
                lifecycle_tracking: None,
                access_tracking: None,
            });
        }

        LocalizedExportData {
            allocations,
            enhanced_allocations: Vec::new(),
            stats: MemoryStats::default(),
            ffi_stats: UnsafeFFIStats::default(),
            scope_info: Vec::<ScopeInfo>::new(),
            timestamp: Instant::now(),
        }
    }

    #[test]
    fn test_parallel_shard_processor_creation() {
        let config = ParallelShardConfig::default();
        let processor = ParallelShardProcessor::new(config);
        assert_eq!(processor.get_config().shard_size, 1000);
    }

    #[test]
    fn test_small_dataset_sequential_processing() {
        let data = create_test_data(100); // 小数据集，应该使用串行处理
        let processor = ParallelShardProcessor::default();
        
        let result = processor.process_allocations_parallel(&data);
        assert!(result.is_ok());
        
        let (shards, stats) = result.unwrap();
        assert_eq!(stats.total_allocations, 100);
        assert!(!stats.used_parallel_processing); // 应该使用串行处理
        assert_eq!(shards.len(), 1); // 只有一个分片
    }

    #[test]
    fn test_large_dataset_parallel_processing() {
        let data = create_test_data(5000); // 大数据集，应该使用并行处理
        let processor = ParallelShardProcessor::default();
        
        let result = processor.process_allocations_parallel(&data);
        assert!(result.is_ok());
        
        let (shards, stats) = result.unwrap();
        assert_eq!(stats.total_allocations, 5000);
        assert!(stats.used_parallel_processing); // 应该使用并行处理
        assert!(shards.len() > 1); // 应该有多个分片
        
        // 验证所有分片的分配总数等于原始数据
        let total_processed: usize = shards.iter().map(|s| s.allocation_count).sum();
        assert_eq!(total_processed, 5000);
    }

    #[test]
    fn test_custom_config() {
        let config = ParallelShardConfig {
            shard_size: 500,
            parallel_threshold: 1000,
            max_threads: Some(2),
            enable_monitoring: false,
            estimated_json_size_per_allocation: 150,
        };
        
        let data = create_test_data(2000);
        let processor = ParallelShardProcessor::new(config);
        
        let result = processor.process_allocations_parallel(&data);
        assert!(result.is_ok());
        
        let (shards, stats) = result.unwrap();
        assert_eq!(stats.total_allocations, 2000);
        assert_eq!(shards.len(), 4); // 2000 / 500 = 4 个分片
    }

    #[test]
    fn test_convenience_functions() {
        let data = create_test_data(1500);
        
        // 测试快速处理函数
        let result = process_allocations_fast(&data);
        assert!(result.is_ok());
        
        // 测试自定义配置函数
        let config = ParallelShardConfig {
            shard_size: 300,
            ..Default::default()
        };
        let result = process_allocations_with_config(&data, config);
        assert!(result.is_ok());
        
        let (shards, _) = result.unwrap();
        assert_eq!(shards.len(), 5); // 1500 / 300 = 5 个分片
    }

    #[test]
    fn test_processed_shard_structure() {
        let data = create_test_data(100);
        let processor = ParallelShardProcessor::default();
        
        let result = processor.process_allocations_parallel(&data);
        assert!(result.is_ok());
        
        let (shards, _) = result.unwrap();
        assert_eq!(shards.len(), 1);
        
        let shard = &shards[0];
        assert_eq!(shard.allocation_count, 100);
        assert_eq!(shard.shard_index, 0);
        assert!(!shard.data.is_empty());
        // processing_time_ms is u64, always >= 0, so just check it exists
        assert!(shard.processing_time_ms < u64::MAX);
        
        // 验证 JSON 数据是有效的
        let parsed: Result<Vec<AllocationInfo>, _> = serde_json::from_slice(&shard.data);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().len(), 100);
    }
}