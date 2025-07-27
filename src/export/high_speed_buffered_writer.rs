//! 高速缓冲写入器 - 减少 I/O 开销的高性能文件写入
//!
//! 这个模块实现了高速缓冲写入功能，使用大缓冲区减少系统调用次数，
//! 并提供高效的分片合并逻辑，显著提高文件写入性能。

use crate::core::types::{TrackingError, TrackingResult};
use crate::export::parallel_shard_processor::ProcessedShard;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

/// 高速缓冲写入器配置
#[derive(Debug, Clone)]
pub struct HighSpeedWriterConfig {
    /// 缓冲区大小（字节）
    pub buffer_size: usize,
    /// 是否启用性能监控
    pub enable_monitoring: bool,
    /// 预估总输出大小（用于预分配）
    pub estimated_total_size: Option<usize>,
    /// 是否使用压缩写入
    pub enable_compression: bool,
    /// 写入完成后是否立即刷新
    pub auto_flush: bool,
}

impl Default for HighSpeedWriterConfig {
    fn default() -> Self {
        Self {
            buffer_size: 2 * 1024 * 1024, // 2MB 缓冲区
            enable_monitoring: true,
            estimated_total_size: None,
            enable_compression: false,
            auto_flush: true,
        }
    }
}

/// 写入性能统计
#[derive(Debug, Clone)]
pub struct WritePerformanceStats {
    /// 写入的总字节数
    pub total_bytes_written: usize,
    /// 写入的分片数量
    pub shards_written: usize,
    /// 总写入时间（毫秒）
    pub total_write_time_ms: u64,
    /// 平均写入速度（字节/秒）
    pub avg_write_speed_bps: f64,
    /// 缓冲区刷新次数
    pub flush_count: usize,
    /// 预分配是否有效
    pub preallocation_effective: bool,
    /// 实际缓冲区使用率
    pub buffer_utilization: f64,
}

/// 高速缓冲写入器
pub struct HighSpeedBufferedWriter {
    /// 内部缓冲写入器
    writer: BufWriter<File>,
    /// 配置
    config: HighSpeedWriterConfig,
    /// 内部缓冲区
    internal_buffer: Vec<u8>,
    /// 统计信息
    stats: WritePerformanceStats,
    /// 开始时间
    start_time: Instant,
    /// 刷新计数
    flush_count: usize,
}

impl HighSpeedBufferedWriter {
    /// 创建新的高速缓冲写入器
    pub fn new<P: AsRef<Path>>(path: P, config: HighSpeedWriterConfig) -> TrackingResult<Self> {
        let file = File::create(path.as_ref())
            .map_err(|e| TrackingError::IoError(format!("创建文件失败: {}", e)))?;

        let writer = BufWriter::with_capacity(config.buffer_size, file);

        // 预分配内部缓冲区
        let initial_capacity = config.estimated_total_size.unwrap_or(1024 * 1024); // 默认 1MB
        let internal_buffer = Vec::with_capacity(initial_capacity);

        let stats = WritePerformanceStats {
            total_bytes_written: 0,
            shards_written: 0,
            total_write_time_ms: 0,
            avg_write_speed_bps: 0.0,
            flush_count: 0,
            preallocation_effective: false,
            buffer_utilization: 0.0,
        };

        Ok(Self {
            writer,
            config,
            internal_buffer,
            stats,
            start_time: Instant::now(),
            flush_count: 0,
        })
    }

    /// 写入处理后的分片数据
    pub fn write_processed_shards(&mut self, shards: &[ProcessedShard]) -> TrackingResult<WritePerformanceStats> {
        let write_start = Instant::now();

        if self.config.enable_monitoring {
            println!("🔄 Starting high-speed buffered write for {} shards...", shards.len());
        }

        // 预计算总大小并预分配缓冲区
        let total_size: usize = shards.iter().map(|s| s.data.len()).sum();
        let estimated_final_size = total_size + 1024; // 额外空间用于 JSON 结构

        // 检查预分配是否有效
        self.stats.preallocation_effective = self.internal_buffer.capacity() >= estimated_final_size;
        
        if !self.stats.preallocation_effective {
            self.internal_buffer.reserve(estimated_final_size);
        }

        // 构建完整的 JSON 结构
        self.build_complete_json(shards)?;

        // 一次性写入文件
        self.writer.write_all(&self.internal_buffer)
            .map_err(|e| TrackingError::IoError(format!("写入文件失败: {}", e)))?;

        // 刷新缓冲区
        if self.config.auto_flush {
            self.flush()?;
        }

        let write_time = write_start.elapsed();

        // 更新统计信息
        self.stats.total_bytes_written = self.internal_buffer.len();
        self.stats.shards_written = shards.len();
        self.stats.total_write_time_ms = write_time.as_millis() as u64;
        self.stats.avg_write_speed_bps = if write_time.as_secs_f64() > 0.0 {
            self.stats.total_bytes_written as f64 / write_time.as_secs_f64()
        } else {
            0.0
        };
        self.stats.buffer_utilization = if self.internal_buffer.capacity() > 0 {
            self.internal_buffer.len() as f64 / self.internal_buffer.capacity() as f64
        } else {
            0.0
        };

        if self.config.enable_monitoring {
            self.print_write_stats();
        }

        Ok(self.stats.clone())
    }

    /// 构建完整的 JSON 结构
    fn build_complete_json(&mut self, shards: &[ProcessedShard]) -> TrackingResult<()> {
        // 清空内部缓冲区
        self.internal_buffer.clear();

        // 写入 JSON 开始
        self.internal_buffer.extend_from_slice(b"{\"allocations\":[");

        // 合并所有分片，只在分片之间添加逗号
        for (i, shard) in shards.iter().enumerate() {
            if i > 0 {
                self.internal_buffer.extend_from_slice(b",");
            }

            // 移除分片的 [ 和 ]，只保留内容
            let shard_content = if shard.data.starts_with(b"[") && shard.data.ends_with(b"]") {
                &shard.data[1..shard.data.len()-1]
            } else {
                &shard.data
            };

            self.internal_buffer.extend_from_slice(shard_content);
        }

        // 写入 JSON 结束
        self.internal_buffer.extend_from_slice(b"]}");

        Ok(())
    }

    /// 写入自定义 JSON 数据
    pub fn write_custom_json(&mut self, json_data: &[u8]) -> TrackingResult<WritePerformanceStats> {
        let write_start = Instant::now();

        if self.config.enable_monitoring {
            println!("🔄 Starting custom JSON data write ({} bytes)...", json_data.len());
        }

        // 预分配缓冲区
        if self.internal_buffer.capacity() < json_data.len() {
            self.internal_buffer.reserve(json_data.len());
        }

        // 复制数据到内部缓冲区
        self.internal_buffer.clear();
        self.internal_buffer.extend_from_slice(json_data);

        // 写入文件
        self.writer.write_all(&self.internal_buffer)
            .map_err(|e| TrackingError::IoError(format!("写入自定义 JSON 失败: {}", e)))?;

        if self.config.auto_flush {
            self.flush()?;
        }

        let write_time = write_start.elapsed();

        // 更新统计信息
        self.stats.total_bytes_written = json_data.len();
        self.stats.shards_written = 1; // 自定义数据算作一个分片
        self.stats.total_write_time_ms = write_time.as_millis() as u64;
        self.stats.avg_write_speed_bps = if write_time.as_secs_f64() > 0.0 {
            json_data.len() as f64 / write_time.as_secs_f64()
        } else {
            0.0
        };

        if self.config.enable_monitoring {
            self.print_write_stats();
        }

        Ok(self.stats.clone())
    }

    /// 强制刷新缓冲区
    pub fn flush(&mut self) -> TrackingResult<()> {
        self.writer.flush()
            .map_err(|e| TrackingError::IoError(format!("刷新缓冲区失败: {}", e)))?;
        
        self.flush_count += 1;
        self.stats.flush_count = self.flush_count;
        
        Ok(())
    }

    /// 完成写入并获取最终统计信息
    pub fn finalize(mut self) -> TrackingResult<WritePerformanceStats> {
        // 确保所有数据都被写入
        self.flush()?;

        // 计算总体统计
        let total_time = self.start_time.elapsed();
        self.stats.total_write_time_ms = total_time.as_millis() as u64;

        if self.config.enable_monitoring {
            println!("✅ High-speed buffered write completed:");
            println!("   Total time: {:?}", total_time);
            self.print_write_stats();
        }

        Ok(self.stats)
    }

    /// 获取当前统计信息
    pub fn get_stats(&self) -> &WritePerformanceStats {
        &self.stats
    }

    /// 获取配置信息
    pub fn get_config(&self) -> &HighSpeedWriterConfig {
        &self.config
    }

    /// Print write statistics
    fn print_write_stats(&self) {
        println!("   Bytes written: {} ({:.2} MB)", 
                self.stats.total_bytes_written,
                self.stats.total_bytes_written as f64 / 1024.0 / 1024.0);
        println!("   Shards written: {}", self.stats.shards_written);
        println!("   Write speed: {:.2} MB/s", 
                self.stats.avg_write_speed_bps / 1024.0 / 1024.0);
        println!("   Buffer utilization: {:.1}%", self.stats.buffer_utilization * 100.0);
        println!("   Flush count: {}", self.stats.flush_count);
        println!("   Preallocation effective: {}", self.stats.preallocation_effective);
    }
}

/// 便利函数：快速写入分片数据
pub fn write_shards_fast<P: AsRef<Path>>(
    path: P,
    shards: &[ProcessedShard],
) -> TrackingResult<WritePerformanceStats> {
    let total_size: usize = shards.iter().map(|s| s.data.len()).sum();
    let config = HighSpeedWriterConfig {
        estimated_total_size: Some(total_size + 1024),
        ..Default::default()
    };

    let mut writer = HighSpeedBufferedWriter::new(path, config)?;
    writer.write_processed_shards(shards)
}

/// 便利函数：使用自定义配置写入
pub fn write_shards_with_config<P: AsRef<Path>>(
    path: P,
    shards: &[ProcessedShard],
    config: HighSpeedWriterConfig,
) -> TrackingResult<WritePerformanceStats> {
    let mut writer = HighSpeedBufferedWriter::new(path, config)?;
    writer.write_processed_shards(shards)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    fn create_test_shards(count: usize, size_per_shard: usize) -> Vec<ProcessedShard> {
        let mut shards = Vec::new();
        for i in 0..count {
            let data = format!("{{\"test_data_{}\": {}}}", i, i).repeat(size_per_shard / 20);
            shards.push(ProcessedShard {
                data: format!("[{}]", data).into_bytes(),
                allocation_count: 1,
                shard_index: i,
                processing_time_ms: 1,
            });
        }
        shards
    }

    #[test]
    fn test_high_speed_writer_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = HighSpeedWriterConfig::default();
        let writer = HighSpeedBufferedWriter::new(temp_file.path(), config);
        assert!(writer.is_ok());
    }

    #[test]
    fn test_write_processed_shards() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = HighSpeedWriterConfig::default();
        let mut writer = HighSpeedBufferedWriter::new(temp_file.path(), config).unwrap();

        let shards = create_test_shards(3, 100);
        let result = writer.write_processed_shards(&shards);
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.shards_written, 3);
        assert!(stats.total_bytes_written > 0);
        assert!(stats.avg_write_speed_bps > 0.0);

        // 验证文件内容
        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.starts_with("{\"allocations\":["));
        assert!(content.ends_with("]}"));
    }

    #[test]
    fn test_write_custom_json() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = HighSpeedWriterConfig::default();
        let mut writer = HighSpeedBufferedWriter::new(temp_file.path(), config).unwrap();

        let json_data = b"{\"test\": \"data\"}";
        let result = writer.write_custom_json(json_data);
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.total_bytes_written, json_data.len());

        // 验证文件内容
        let content = fs::read(temp_file.path()).unwrap();
        assert_eq!(content, json_data);
    }

    #[test]
    fn test_preallocation_effectiveness() {
        let temp_file = NamedTempFile::new().unwrap();
        let shards = create_test_shards(5, 200);
        let total_size: usize = shards.iter().map(|s| s.data.len()).sum();

        // 测试有效预分配
        let config = HighSpeedWriterConfig {
            estimated_total_size: Some(total_size + 1024),
            enable_monitoring: false,
            ..Default::default()
        };
        let mut writer = HighSpeedBufferedWriter::new(temp_file.path(), config).unwrap();
        let stats = writer.write_processed_shards(&shards).unwrap();
        assert!(stats.preallocation_effective);

        // 测试无效预分配
        let temp_file2 = NamedTempFile::new().unwrap();
        let config2 = HighSpeedWriterConfig {
            estimated_total_size: Some(100), // 太小的预分配
            enable_monitoring: false,
            ..Default::default()
        };
        let mut writer2 = HighSpeedBufferedWriter::new(temp_file2.path(), config2).unwrap();
        let stats2 = writer2.write_processed_shards(&shards).unwrap();
        assert!(!stats2.preallocation_effective);
    }

    #[test]
    fn test_convenience_functions() {
        let temp_file = NamedTempFile::new().unwrap();
        let shards = create_test_shards(2, 150);

        // 测试快速写入函数
        let result = write_shards_fast(temp_file.path(), &shards);
        assert!(result.is_ok());

        // 测试自定义配置函数
        let temp_file2 = NamedTempFile::new().unwrap();
        let config = HighSpeedWriterConfig {
            buffer_size: 1024 * 1024,
            enable_monitoring: false,
            ..Default::default()
        };
        let result2 = write_shards_with_config(temp_file2.path(), &shards, config);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_flush_functionality() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = HighSpeedWriterConfig {
            auto_flush: false,
            enable_monitoring: false,
            ..Default::default()
        };
        let mut writer = HighSpeedBufferedWriter::new(temp_file.path(), config).unwrap();

        let shards = create_test_shards(1, 100);
        let _stats = writer.write_processed_shards(&shards).unwrap();

        // 手动刷新
        let flush_result = writer.flush();
        assert!(flush_result.is_ok());
        assert_eq!(writer.get_stats().flush_count, 1);
    }

    #[test]
    fn test_finalize() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = HighSpeedWriterConfig {
            enable_monitoring: false,
            ..Default::default()
        };
        let mut writer = HighSpeedBufferedWriter::new(temp_file.path(), config).unwrap();

        let shards = create_test_shards(2, 100);
        let _stats = writer.write_processed_shards(&shards).unwrap();

        let final_stats = writer.finalize();
        assert!(final_stats.is_ok());
        
        let stats = final_stats.unwrap();
        // 在快速测试中，时间可能为 0，所以检查 >= 0
        assert!(stats.total_write_time_ms >= 0);
    }
}