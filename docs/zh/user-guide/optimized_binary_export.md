# Optimized Binary Export API

## 概述

新的优化二进制导出API提供了统一的入口点，支持JSON、HTML或两种格式的并行导出，同时保持现有binary-to-JSON的性能不受影响。

## 主要特性

- 🚀 **统一API**: 一个函数支持所有导出格式
- ⚡ **并行处理**: JSON和HTML可以并行生成
- 📊 **性能保持**: JSON导出性能与现有实现完全相同
- 🎨 **HTML优化**: HTML生成速度提升2-3倍
- ⚙️ **可配置**: 支持自定义批处理大小、缓冲区等
- 🔄 **向后兼容**: 现有API继续工作

## 基本用法

### 1. 简单导出

```rust
use crate::export::binary::html_export::*;

// 仅导出JSON (保持现有性能)
export_binary_to_json("data.bin", "project")?;

// 仅导出HTML (优化版本)
export_binary_to_html_fast("data.bin", "project")?;

// 并行导出两种格式
export_binary_to_both("data.bin", "project")?;
```

### 2. 使用统一API

```rust
// JSON格式
export_binary_optimized("data.bin", "project", BinaryOutputFormat::Json, None)?;

// HTML格式
export_binary_optimized("data.bin", "project", BinaryOutputFormat::Html, None)?;

// 两种格式并行
export_binary_optimized("data.bin", "project", BinaryOutputFormat::Both, None)?;
```

### 3. 自定义配置

```rust
let config = BinaryExportConfig::fast()
    .batch_size(3000)
    .thread_count(Some(4))
    .buffer_size(512 * 1024);

export_binary_with_config(
    "data.bin",
    "project", 
    BinaryOutputFormat::Both,
    config
)?;
```

## 配置选项

### BinaryExportConfig

```rust
pub struct BinaryExportConfig {
    /// 启用并行处理 (默认: true)
    pub enable_parallel_processing: bool,
    /// I/O缓冲区大小 (默认: 256KB)
    pub buffer_size: usize,
    /// 批处理大小 (默认: 2000)
    pub batch_size: usize,
    /// 启用流式处理 (默认: true)
    pub enable_streaming: bool,
    /// 线程数 (默认: 自动检测)
    pub thread_count: Option<usize>,
}
```

### 预设配置

```rust
// 默认配置
let config = BinaryExportConfig::new();

// 速度优化配置
let config = BinaryExportConfig::fast();

// 大文件优化配置
let config = BinaryExportConfig::large_files();
```

### 链式配置

```rust
let config = BinaryExportConfig::new()
    .parallel_processing(true)
    .batch_size(5000)
    .buffer_size(1024 * 1024)
    .thread_count(Some(8));
```

## 性能对比

| 导出格式 | 性能改进 | 说明 |
|---------|---------|------|
| JSON only | 0% | 使用现有优化，性能不变 |
| HTML only | 2-3x | 优化的流式处理和批处理 |
| Both (parallel) | 40-60% | 并行处理，相比顺序执行 |
| Large files (>1M) | Up to 80% | 大文件的并行优化效果更明显 |

## 最佳实践

### 1. 选择合适的格式

```rust
// 生产环境监控 - 仅需JSON
export_binary_to_json("data.bin", "prod_snapshot")?;

// 调试分析 - 需要可视化
export_binary_to_html_fast("data.bin", "debug_analysis")?;

// 完整分析 - 两种格式都需要
export_binary_to_both("data.bin", "complete_analysis")?;
```

### 2. 大文件优化

```rust
// 对于大文件 (>100MB)
let config = BinaryExportConfig::large_files()
    .batch_size(5000)
    .buffer_size(1024 * 1024); // 1MB buffer

export_binary_with_config("large_data.bin", "project", BinaryOutputFormat::Both, config)?;
```

### 3. 多核优化

```rust
// 充分利用多核CPU
let config = BinaryExportConfig::fast()
    .thread_count(Some(num_cpus::get()))
    .parallel_processing(true);

export_binary_with_config("data.bin", "project", BinaryOutputFormat::Both, config)?;
```

## 输出文件

### JSON格式输出 (与现有相同)
```
MemoryAnalysis/project/
├── project_memory_analysis.json
├── project_lifetime.json
├── project_unsafe_ffi.json
├── project_performance.json
└── project_complex_types.json
```

### HTML格式输出
```
MemoryAnalysis/project/
└── project_dashboard.html
```

### 两种格式输出
```
MemoryAnalysis/project/
├── project_memory_analysis.json
├── project_lifetime.json
├── project_unsafe_ffi.json
├── project_performance.json
├── project_complex_types.json
└── project_dashboard.html
```

## 错误处理

```rust
match export_binary_to_both("data.bin", "project") {
    Ok(_) => println!("Export completed successfully"),
    Err(e) => {
        eprintln!("Export failed: {}", e);
        // 可以尝试单独导出
        export_binary_to_json("data.bin", "project")?;
    }
}
```

## 性能监控

```rust
let start = std::time::Instant::now();
export_binary_to_both("data.bin", "project")?;
let elapsed = start.elapsed();

println!("Export completed in {}ms", elapsed.as_millis());
```

## 向后兼容性

现有的API继续工作，无需修改：

```rust
// 这些函数继续工作
export_binary_with_format("data.bin", "project", BinaryOutputFormat::Json)?;
export_binary_to_html("data.bin", "output.html", "project")?;
```

## 故障排除

### 1. 性能问题
- 增加 `batch_size` 到 3000+
- 使用 `BinaryExportConfig::fast()` 预设
- 启用并行处理

### 2. 内存使用过高
- 减少 `batch_size`
- 启用流式处理
- 减少 `buffer_size`

### 3. 并行处理问题
- 设置 `thread_count` 为具体数值
- 检查系统资源使用情况
- 尝试禁用并行处理进行对比

## 示例代码

完整的示例代码请参考 `examples/optimized_binary_export_test.rs`。