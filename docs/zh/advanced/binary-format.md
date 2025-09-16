# 二进制导出格式详解

memscope-rs 的二进制格式（`.memscope`）是一个高性能的内存数据存储格式，专为大规模内存分析设计。

## 🚀 性能优势

### 实测性能对比

基于 `advanced_metrics_demo` 示例的实际测试结果：

| 指标 | Binary格式 | JSON格式 | 性能提升 |
|------|-----------|----------|----------|
| **导出时间** | 211ms | 17.1s | **80.72倍** |
| **文件大小** | 480KB | 728KB | **节省34.0%** |
| **内存使用** | 低 | 高 | 显著降低 |

### 为什么这么快？

1. **二进制序列化** - 直接写入内存布局，无需文本转换
2. **紧凑格式** - 优化的数据结构，减少冗余
3. **批量写入** - 减少系统调用次数
4. **无格式化开销** - 不需要JSON美化或SVG渲染

## 📁 基础使用

### 导出到二进制格式

```rust
use memscope_rs::{get_global_tracker, track_var, init};

fn main() {
    init();
    
    // 创建一些数据
    let data = vec![1, 2, 3, 4, 5];
    track_var!(data);
    
    let tracker = get_global_tracker();
    
    // 导出到二进制格式（推荐用于大数据量）
    if let Err(e) = tracker.export_to_binary("my_analysis") {
        eprintln!("Binary导出失败: {}", e);
    } else {
        println!("✅ Binary导出成功");
        // 文件位置: MemoryAnalysis/my_analysis/my_analysis.memscope
    }
}
```

### 文件命名规则

```
输入: "my_analysis"
输出: MemoryAnalysis/my_analysis/my_analysis.memscope
```

## 🔄 格式转换

### Binary → JSON 转换

```rust
use memscope_rs::MemoryTracker;

// 转换为5个分类的JSON文件
MemoryTracker::parse_binary_to_standard_json(
    "data.memscope", 
    "converted_data"
)?;

// 生成的文件:
// - converted_data_memory_analysis.json
// - converted_data_lifetime.json
// - converted_data_performance.json
// - converted_data_unsafe_ffi.json
// - converted_data_complex_types.json
```

### Binary → 单个JSON文件

```rust
use memscope_rs::MemoryTracker;

// 转换为单个JSON文件
MemoryTracker::parse_binary_to_json(
    "data.memscope", 
    "single_output.json"
)?;
```

### Binary → HTML 报告

```rust
use memscope_rs::MemoryTracker;

// 直接从binary生成HTML报告
MemoryTracker::parse_binary_to_html(
    "data.memscope", 
    "report.html"
)?;
```

## 🎯 实际使用示例

### 示例1：高性能数据导出

```bash
# 运行高级示例（生成大量数据）
cargo run --example advanced_metrics_demo

# 查看生成的二进制文件
ls -la MemoryAnalysis/advanced_metrics_demo/
# -rw-r--r-- 1 user staff 480737 Aug  5 10:30 advanced_metrics_demo.memscope

# 转换为JSON进行分析
make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo
```

### 示例2：二进制导出专门示例

```bash
# 运行二进制导出示例
cargo run --example binary_export_demo

# 查看性能对比结果
# Binary vs Standard JSON Export Performance:
#   📊 Binary export time:     14ms
#   📊 Standard JSON time:     4.2s  
#   🚀 Speed improvement:      291.80x faster
#   📁 Binary file size:       480KB
#   📁 JSON files size:        728KB
#   💾 Size reduction:         34.0%
```

## 🔧 高级用法

### 批量转换

```rust
use memscope_rs::MemoryTracker;
use std::fs;

fn batch_convert_binary_to_json(input_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension() == Some(std::ffi::OsStr::new("memscope")) {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let output_base = format!("{}_converted", stem);
            
            println!("转换: {} → {}", path.display(), output_base);
            
            MemoryTracker::parse_binary_to_standard_json(&path, &output_base)?;
        }
    }
    
    Ok(())
}

// 使用
batch_convert_binary_to_json("MemoryAnalysis/")?;
```

### 性能基准测试

```rust
use memscope_rs::{get_global_tracker, track_var, init};
use std::time::Instant;

fn performance_comparison() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    // 创建大量数据
    for i in 0..1000 {
        let data = vec![i; 100];
        track_var!(data);
    }
    
    let tracker = get_global_tracker();
    
    // 测试Binary导出性能
    let start = Instant::now();
    tracker.export_to_binary("perf_test_binary")?;
    let binary_time = start.elapsed();
    
    // 测试JSON导出性能
    let start = Instant::now();
    tracker.export_to_json("perf_test_json")?;
    let json_time = start.elapsed();
    
    println!("性能对比:");
    println!("  Binary导出: {:?}", binary_time);
    println!("  JSON导出:   {:?}", json_time);
    println!("  速度提升:   {:.2}x", json_time.as_nanos() as f64 / binary_time.as_nanos() as f64);
    
    Ok(())
}
```

## 📊 格式规范

### 文件结构

```
.memscope 文件结构:
┌─────────────────┐
│ 文件头 (Header)  │ - 魔数、版本、元数据
├─────────────────┤
│ 字符串表         │ - 去重的字符串数据
├─────────────────┤
│ 分配记录         │ - 内存分配信息
├─────────────────┤
│ 统计数据         │ - 汇总统计信息
├─────────────────┤
│ 扩展数据         │ - 高级分析数据
└─────────────────┘
```

### 数据完整性

二进制格式保留所有信息：
- ✅ 变量名和类型信息
- ✅ 时间戳和线程ID
- ✅ 内存地址和大小
- ✅ 生命周期数据
- ✅ 性能指标
- ✅ 复杂类型分析
- ✅ Unsafe/FFI跟踪

## 🛠️ 故障排除

### 常见问题

1. **文件损坏**
   ```rust
   // 验证文件完整性
   match MemoryTracker::parse_binary_to_json("data.memscope", "test.json") {
       Ok(_) => println!("文件完整"),
       Err(e) => println!("文件可能损坏: {}", e),
   }
   ```

2. **版本兼容性**
   ```rust
   // 二进制格式向后兼容
   // 新版本可以读取旧版本的文件
   // 但旧版本无法读取新版本的文件
   ```

3. **大文件处理**
   ```bash
   # 对于超大文件，分批转换
   # 使用流式处理避免内存不足
   ```

### 性能调优

```rust
// 对于大数据量，优先使用二进制格式
if data_size > 1_000_000 {
    tracker.export_to_binary("large_dataset")?;
} else {
    tracker.export_to_json("small_dataset")?;
}
```

## 🔗 与其他格式对比

### 使用场景建议

| 场景 | 推荐格式 | 原因 |
|------|---------|------|
| **大数据量分析** | Binary | 80倍速度提升 |
| **自动化处理** | Binary → JSON | 先快速导出，再按需转换 |
| **交互式分析** | Binary → HTML | 直接生成可视化报告 |
| **数据存档** | Binary | 文件小，完整性好 |
| **快速调试** | SVG | 立即可视化 |

### 工作流建议

```bash
# 推荐的工作流程
# 1. 开发阶段 - 使用Binary快速导出
cargo run --example your_program
# → 生成 .memscope 文件

# 2. 分析阶段 - 按需转换
make html DIR=MemoryAnalysis/your_data BASE=your_data
# → 生成交互式HTML报告

# 3. 数据处理 - 转换为JSON
MemoryTracker::parse_binary_to_standard_json("data.memscope", "analysis")
# → 生成5个分类JSON文件
```

## 💡 最佳实践

### 1. 命名约定

```rust
// ✅ 使用描述性名称
tracker.export_to_binary("user_session_analysis")?;
tracker.export_to_binary("performance_benchmark_2024")?;

// ❌ 避免通用名称
tracker.export_to_binary("data")?;
tracker.export_to_binary("test")?;
```

### 2. 文件管理

```bash
# 建议的目录结构
MemoryAnalysis/
├── daily_reports/
│   ├── 2024-08-05.memscope
│   └── 2024-08-06.memscope
├── benchmarks/
│   ├── baseline.memscope
│   └── optimized.memscope
└── debugging/
    ├── issue_123.memscope
    └── crash_analysis.memscope
```

### 3. 自动化脚本

```bash
#!/bin/bash
# 自动化二进制分析脚本

BINARY_FILE="$1"
OUTPUT_NAME="$2"

if [ -z "$BINARY_FILE" ] || [ -z "$OUTPUT_NAME" ]; then
    echo "用法: $0 <binary_file> <output_name>"
    exit 1
fi

echo "🔄 转换二进制文件: $BINARY_FILE"

# 转换为JSON
echo "生成JSON文件..."
./target/release/memscope-rs parse-binary-to-json "$BINARY_FILE" "${OUTPUT_NAME}.json"

# 生成HTML报告
echo "生成HTML报告..."
./target/release/memscope-rs parse-binary-to-html "$BINARY_FILE" "${OUTPUT_NAME}.html"

echo "✅ 转换完成!"
echo "📄 JSON: ${OUTPUT_NAME}.json"
echo "🌐 HTML: ${OUTPUT_NAME}.html"
```

## 🎉 总结

二进制格式是 memscope-rs 的核心优势之一：

✅ **极致性能** - 比JSON快80倍以上  
✅ **空间效率** - 节省34%存储空间  
✅ **完整数据** - 保留所有分析信息  
✅ **灵活转换** - 可转换为任何其他格式  
✅ **向后兼容** - 版本升级无忧  

对于任何需要高性能内存分析的场景，二进制格式都是最佳选择！🚀