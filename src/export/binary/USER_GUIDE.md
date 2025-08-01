# Binary Export System - 用户指南

## 🚀 快速开始

### 最简单的使用方式

```rust
use memscope_rs::export::binary::BinaryExport;

// 一行代码完成导出
let result = BinaryExport::export_default(&tracker, "output.bin")?;
println!("导出完成: {} bytes", result.bytes_written);
```

### 基本概念

Binary Export System 是一个高性能的内存分析数据导出系统，提供：

- **3-8倍性能提升** - 相比JSON导出
- **恒定内存使用** - 支持任意大小数据集
- **多格式支持** - MessagePack、自定义二进制、分块格式
- **智能压缩** - 自动算法选择和优化
- **错误恢复** - 智能重试和降级策略

## 📊 配置选择指南

### 预设配置

```rust
// 高性能配置 - 最大化速度
let config = IntegratedConfig::high_performance();

// 内存高效配置 - 最小化内存使用
let config = IntegratedConfig::memory_efficient();

// 平衡配置 - 速度和内存的平衡
let config = IntegratedConfig::balanced();
```

### 配置选择建议

| 场景 | 推荐配置 | 特点 |
|------|----------|------|
| 小数据集 (<10MB) | `balanced()` | 平衡性能和资源使用 |
| 大数据集 (>100MB) | `memory_efficient()` | 恒定内存使用 |
| 性能优先 | `high_performance()` | 最大化导出速度 |
| 存储优先 | 自定义高压缩 | 最小化文件大小 |

## 🔧 高级配置

### 自定义配置示例

```rust
let mut config = IntegratedConfig::balanced();

// 自定义压缩设置
config.compression.algorithm = CompressionAlgorithm::Zstd;
config.compression.level = 9; // 高压缩比

// 自定义输出格式
config.output_format = OutputFormat::CustomBinary;

// 自定义处理设置
config.processing.chunk_size = 128 * 1024; // 128KB块
config.processing.validate_data = true;

// 启用并行处理
if let Some(ref mut parallel) = config.parallel {
    parallel.worker_threads = num_cpus::get();
    parallel.enable_work_stealing = true;
}

let mut exporter = IntegratedBinaryExporter::new(config);
```

## 📁 支持的格式

### MessagePack格式
```rust
config.output_format = OutputFormat::MessagePack;
```
- **优点**: 最大兼容性，跨语言支持
- **缺点**: 性能略低于自定义格式
- **适用**: 需要与其他工具集成

### 自定义二进制格式
```rust
config.output_format = OutputFormat::CustomBinary;
```
- **优点**: 最高性能，最小文件大小
- **缺点**: 仅限memscope-rs使用
- **适用**: 纯Rust环境，性能优先

### 分块格式
```rust
config.output_format = OutputFormat::Chunked { chunk_size: 1024 * 1024 };
```
- **优点**: 支持流式处理，内存友好
- **缺点**: 略微增加文件大小
- **适用**: 超大数据集，内存受限环境

## 🗜️ 压缩选项

### 压缩算法对比

| 算法 | 速度 | 压缩比 | 内存使用 | 推荐场景 |
|------|------|--------|----------|----------|
| None | 最快 | 1.0 | 最低 | 网络传输，临时文件 |
| LZ4 | 很快 | 2-3x | 低 | 实时处理，快速导出 |
| Zstd | 平衡 | 3-5x | 中等 | 通用场景，推荐默认 |
| Zstd高级别 | 较慢 | 5-8x | 高 | 存储优化，归档 |

### 压缩配置示例

```rust
// 快速压缩
config.compression = CompressionConfig::fast();

// 平衡压缩
config.compression = CompressionConfig::balanced();

// 最大压缩
config.compression = CompressionConfig::max_compression();

// 自定义压缩
config.compression.algorithm = CompressionAlgorithm::Zstd;
config.compression.level = 6;
config.compression.auto_select = true; // 智能选择
```

## 🔄 异步操作

### 异步导出

```rust
// 简单异步导出
let result = BinaryExport::export_async(&tracker, "async_output.bin").await?;

// 自定义异步导出
let config = IntegratedConfig::high_performance();
let mut exporter = IntegratedBinaryExporter::new(config);
let result = exporter.export_async(&tracker, "custom_async.bin").await?;
```

### 进度监控

```rust
// 创建带监控的导出器
let mut exporter = IntegratedBinaryExporter::new(config);

// 检查系统状态
let status = exporter.get_system_status();
println!("内存使用: {} MB", status.memory_usage / 1_000_000);

// 执行导出
let result = exporter.export(&tracker, "monitored_output.bin")?;

// 查看性能指标
println!("导出效率: {:.1}%", 
         result.performance_metrics.efficiency.overall_efficiency * 100.0);
```

## 📖 文件读取和解析

### 基本读取

```rust
// 使用导出器读取
let exporter = IntegratedBinaryExporter::new(IntegratedConfig::default());
let data = exporter.load("output.bin")?;

// 使用专用解析器
let parser = BinaryDataParser::new(ParserConfig::default());
let parse_result = parser.parse_file("output.bin")?;
```

### 流式解析

```rust
let parser = BinaryDataParser::new(ParserConfig::default());
let file = std::fs::File::open("large_file.bin")?;
let parse_result = parser.parse_streaming(file)?;

println!("解析完成: {} 结构", parse_result.parse_stats.structures_parsed);
```

### 格式转换

```rust
let parser = BinaryDataParser::new(ParserConfig::default());

// 读取MessagePack文件
let msgpack_data = std::fs::read("data.msgpack")?;

// 转换为自定义二进制格式
let binary_data = parser.convert_format(
    &msgpack_data,
    OutputFormat::MessagePack,
    OutputFormat::CustomBinary
)?;

// 保存转换结果
std::fs::write("data.bin", binary_data)?;
```

## ⚡ 性能优化

### 自动优化

```rust
use memscope_rs::export::binary::optimization::optimize_system_performance;

// 运行自动性能优化
let optimization_result = optimize_system_performance(&tracker)?;

println!("性能提升: {:.2}x", optimization_result.improvement.overall_improvement);

// 使用优化后的配置
let mut exporter = IntegratedBinaryExporter::new(optimization_result.optimized_config);
```

### 手动优化技巧

1. **选择合适的块大小**
   ```rust
   // 小数据集
   config.processing.chunk_size = 64 * 1024;   // 64KB
   
   // 大数据集
   config.processing.chunk_size = 1024 * 1024; // 1MB
   ```

2. **优化并行处理**
   ```rust
   if let Some(ref mut parallel) = config.parallel {
       parallel.worker_threads = num_cpus::get() * 2; // 超线程
       parallel.load_balancing = LoadBalancingStrategy::WorkStealing;
   }
   ```

3. **内存管理优化**
   ```rust
   config.processing.max_memory_usage = 512 * 1024 * 1024; // 512MB
   config.enable_monitoring = true; // 监控内存使用
   ```

## 🛡️ 错误处理

### 基本错误处理

```rust
match BinaryExport::export_default(&tracker, "output.bin") {
    Ok(result) => {
        println!("导出成功: {} bytes", result.bytes_written);
    }
    Err(BinaryExportError::NoDataToExport) => {
        println!("没有数据可导出");
    }
    Err(BinaryExportError::OutOfMemory { requested, available }) => {
        println!("内存不足: 需要 {} bytes, 可用 {} bytes", requested, available);
        // 尝试内存高效配置
        let config = IntegratedConfig::memory_efficient();
        let mut exporter = IntegratedBinaryExporter::new(config);
        exporter.export(&tracker, "fallback_output.bin")?;
    }
    Err(e) => {
        eprintln!("导出失败: {:?}", e);
    }
}
```

### 智能错误恢复

```rust
let recovery = ErrorRecovery::new();

let result = recovery.execute_with_recovery(
    || BinaryExport::export_default(&tracker, "output.bin"),
    "binary_export"
)?;
```

## 📊 基准测试

### 运行基准测试

```rust
use memscope_rs::export::binary::benchmarks;

// 快速基准测试
let results = benchmarks::run_quick_benchmark()?;
println!("性能提升: {:.2}x", results.comparison.avg_speed_improvement);

// 完整基准测试
let config = benchmarks::BenchmarkConfig::default();
let runner = benchmarks::BenchmarkRunner::new(config)?;
let results = runner.run_all_benchmarks()?;

// 保存结果
runner.save_results(&results, Path::new("benchmark_results.json"))?;
```

## 🔍 调试和诊断

### 启用详细日志

```rust
let mut config = IntegratedConfig::balanced();
config.enable_monitoring = true;

let mut exporter = IntegratedBinaryExporter::new(config);
let result = exporter.export(&tracker, "debug_output.bin")?;

// 查看详细统计
println!("组件耗时:");
println!("  数据收集: {:?}", result.performance_metrics.component_times.collection_time);
println!("  数据处理: {:?}", result.performance_metrics.component_times.processing_time);
println!("  压缩处理: {:?}", result.performance_metrics.component_times.compression_time);
```

### 数据完整性验证

```rust
use memscope_rs::export::binary::validation::validate_binary_file;

// 验证导出的文件
let validation_report = validate_binary_file("output.bin")?;

if validation_report.is_valid {
    println!("✅ 文件验证通过");
} else {
    println!("❌ 文件验证失败:");
    for error in &validation_report.errors {
        println!("  - {}", error.message);
    }
}
```

## 🎯 最佳实践

### 1. 配置选择
- 小数据集使用 `balanced()` 配置
- 大数据集使用 `memory_efficient()` 配置
- 性能关键场景使用 `high_performance()` 配置

### 2. 错误处理
- 始终处理 `NoDataToExport` 错误
- 实现降级策略处理资源限制
- 使用 `ErrorRecovery` 进行自动重试

### 3. 性能优化
- 定期运行基准测试验证性能
- 使用自动优化功能调整配置
- 监控内存使用和CPU利用率

### 4. 数据安全
- 启用数据验证确保完整性
- 使用校验和验证文件完整性
- 定期验证导出文件的可读性

## 🆘 常见问题

### Q: 导出速度比预期慢？
A: 尝试以下优化：
- 使用 `high_performance()` 配置
- 禁用数据验证 (`validate_data = false`)
- 使用更快的压缩算法 (LZ4)
- 启用并行处理

### Q: 内存使用过高？
A: 尝试以下方法：
- 使用 `memory_efficient()` 配置
- 减小块大小 (`chunk_size`)
- 启用流式处理
- 增加压缩级别减少内存占用

### Q: 文件无法读取？
A: 检查以下方面：
- 文件格式是否正确
- 文件是否完整（未损坏）
- 版本兼容性
- 使用 `validate_binary_file()` 验证文件

### Q: 如何获得最小文件大小？
A: 使用以下设置：
- 最高压缩级别 (`level = 19`)
- Zstd压缩算法
- 启用所有数据验证
- 考虑使用MessagePack格式

## 📚 更多资源

- **API文档**: 查看各模块的详细API文档
- **示例代码**: 参考 `examples.rs` 中的完整示例
- **基准测试**: 运行 `benchmarks.rs` 验证性能
- **集成测试**: 查看 `integration_tests.rs` 了解测试方法

---

这个用户指南涵盖了Binary Export System的所有主要功能和使用场景。如果您有特定的使用需求或遇到问题，请参考相应的示例代码或联系开发团队。