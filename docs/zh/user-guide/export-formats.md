# 导出格式说明

memscope-rs 支持多种导出格式，每种格式都有其特定的用途和优势。本指南将帮你选择最适合的导出方式。

## 📊 格式对比

| 格式 | 文件大小 | 生成速度 | 交互性 | 适用场景 |
|------|---------|---------|--------|----------|
| **JSON** | 中等 | 中等 | 无 | 数据分析、自动化处理 |
| **SVG** | 小 | 快 | 基础 | 报告嵌入、静态可视化 |
| **HTML** | 大 | 慢 | 高 | 交互式分析、演示 |
| **Binary** | 最小 | **最快** | 无 | 大数据量、性能关键 |

### 性能对比（实测数据）

基于 `advanced_metrics_demo` 示例的实际测试结果：

- **Binary 导出**: 211ms，480KB 文件
- **JSON 导出**: 17.1秒，728KB 文件（5个分类文件）
- **速度提升**: Binary 比 JSON 快 **80.72倍**
- **空间节省**: Binary 比 JSON 节省 **34.0%** 空间

## 📄 JSON 导出 - 数据分析首选

### 特点
- **分类数据** - 5个专门的JSON文件，便于分析不同方面
- **结构化数据** - 完整的内存分配信息
- **机器可读** - 便于自动化分析和处理
- **标准格式** - 可与其他工具集成

### 5个分类文件

JSON导出会生成5个专门的文件：

1. **`*_memory_analysis.json`** - 基础内存分析数据
2. **`*_lifetime.json`** - 变量生命周期信息
3. **`*_performance.json`** - 性能相关数据
4. **`*_unsafe_ffi.json`** - Unsafe代码和FFI跟踪
5. **`*_complex_types.json`** - 复杂类型分析

### 基础使用
```rust
use memscope_rs::{get_global_tracker, track_var, init};

fn main() {
    init();
    
    let data = vec![1, 2, 3, 4, 5];
    track_var!(data);
    
    let tracker = get_global_tracker();
    
    // 导出到 JSON（生成5个分类文件）
    if let Err(e) = tracker.export_to_json("my_analysis") {
        eprintln!("导出失败: {}", e);
    } else {
        println!("✅ JSON 导出成功");
        // 文件位置: MemoryAnalysis/my_analysis/
        // - my_analysis_memory_analysis.json
        // - my_analysis_lifetime.json  
        // - my_analysis_performance.json
        // - my_analysis_unsafe_ffi.json
        // - my_analysis_complex_types.json
    }
}
```

### JSON 数据结构
```json
{
  "metadata": {
    "export_timestamp": 1691234567890,
    "export_version": "0.1.4",
    "total_allocations": 3,
    "active_allocations": 3,
    "peak_memory": 1024
  },
  "memory_stats": {
    "active_allocations": 3,
    "active_memory": 512,
    "total_allocations": 3,
    "total_deallocations": 0,
    "peak_memory": 512,
    "peak_allocations": 3
  },
  "allocations": [
    {
      "ptr": 140712345678912,
      "size": 40,
      "var_name": "data",
      "type_name": "Vec<i32>",
      "timestamp_alloc": 1691234567123,
      "thread_id": "ThreadId(1)",
      "is_leaked": false
    }
  ],
  "analysis": {
    "fragmentation_analysis": {...},
    "circular_references": [...],
    "unsafe_ffi_stats": {...}
  }
}
```

### 自定义 JSON 导出
```rust
use memscope_rs::{get_global_tracker, ExportOptions};

let tracker = get_global_tracker();
let options = ExportOptions::new()
    .include_system_allocations(true);  // 包含系统分配（慢但详细）

// 注意：包含系统分配会显著降低性能（5-10倍慢）
tracker.export_to_json_with_options("detailed_analysis", options)?;
```

### 性能模式选择

```rust
// 快速模式（推荐）- 只跟踪用户变量
tracker.export_to_json("fast_analysis")?;

// 详细模式 - 包含所有系统分配（慢）
let detailed_options = ExportOptions::new()
    .include_system_allocations(true);
tracker.export_to_json_with_options("detailed_analysis", detailed_options)?;
```

## 🎨 SVG 导出 - 静态可视化

### 特点
- **矢量图形** - 可缩放，质量不损失
- **轻量级** - 文件小，加载快
- **嵌入友好** - 可直接嵌入网页和文档

### 基础使用
```rust
use memscope_rs::{get_global_tracker, track_var, init};

fn main() {
    init();
    
    // 创建一些有趣的内存模式
    let vec1 = vec![1; 100];
    track_var!(vec1);
    
    let vec2 = vec![2; 200];
    track_var!(vec2);
    
    let boxed = Box::new(vec![3; 50]);
    track_var!(boxed);
    
    let tracker = get_global_tracker();
    
    // 导出内存使用图表
    if let Err(e) = tracker.export_memory_analysis("memory_chart.svg") {
        eprintln!("SVG 导出失败: {}", e);
    } else {
        println!("✅ SVG 导出成功");
        // 文件位置: MemoryAnalysis/memory_chart.svg
    }
}
```

### SVG 图表类型

**内存使用时间线**
```rust
// 生成内存使用随时间变化的图表
tracker.export_memory_timeline("timeline.svg")?;
```

**分配类型分布**
```rust
// 生成按类型分组的内存分布图
tracker.export_type_distribution("distribution.svg")?;
```

**生命周期分析**
```rust
// 生成变量生命周期可视化
use memscope_rs::export_lifecycle_timeline;
export_lifecycle_timeline("lifecycle.svg", &allocations)?;
```

## 🌐 HTML 导出 - 交互式仪表板

### 特点
- **交互式** - 可点击、过滤、缩放
- **实时分析** - 动态计算和展示
- **美观界面** - 专业的数据可视化
- **两种方式** - 直接导出或通过make命令生成

### 方式1：直接导出HTML
```rust
use memscope_rs::{get_global_tracker, track_var, init};
use std::rc::Rc;

fn main() {
    init();
    
    // 创建复杂的内存场景
    let data1 = vec![1; 1000];
    track_var!(data1);
    
    let shared = Rc::new(String::from("共享数据"));
    track_var!(shared);
    
    let clone1 = Rc::clone(&shared);
    track_var!(clone1);
    
    let tracker = get_global_tracker();
    
    // 导出交互式 HTML 仪表板
    if let Err(e) = tracker.export_interactive_dashboard("interactive_report.html") {
        eprintln!("HTML 导出失败: {}", e);
    } else {
        println!("✅ HTML 导出成功");
        println!("用浏览器打开: MemoryAnalysis/interactive_report.html");
    }
}
```

### 方式2：使用make命令（推荐）
```bash
# 1. 先运行程序生成JSON数据
cargo run --example your_program

# 2. 使用make命令生成增强的HTML报告
make html DIR=MemoryAnalysis/your_analysis BASE=your_analysis

# 3. 打开生成的报告
open memory_report.html
```

这种方式生成的HTML报告功能更丰富，包含更多交互式图表。

### HTML 仪表板功能

**内存概览**
- 实时内存统计
- 分配趋势图表
- 类型分布饼图

**详细分析**
- 可过滤的分配列表
- 智能指针关系图
- 内存泄漏检测结果

**交互功能**
- 点击查看详细信息
- 按类型/线程/时间过滤
- 缩放和平移图表

### 自定义 HTML 主题
```rust
use memscope_rs::HtmlExportOptions;

let html_options = HtmlExportOptions::new()
    .with_theme("dark")              // 深色主题
    .with_charts(true)               // 包含图表
    .with_detailed_tables(true)      // 详细表格
    .with_performance_metrics(true); // 性能指标

tracker.export_to_html_with_options("custom_report.html", &html_options)?;
```

## ⚡ Binary 导出 - 高性能选择

### 特点
- **最小文件** - 紧凑的二进制格式（节省34%空间）
- **最快速度** - 比JSON快80倍以上的导出性能
- **完整数据** - 保留所有分析信息
- **可转换** - 可转换为JSON或HTML格式

### 基础使用
```rust
use memscope_rs::{get_global_tracker, track_var, init};

fn main() {
    init();
    
    // 大量数据场景
    for i in 0..1000 {
        let data = vec![i; 100];
        track_var!(data);
    }
    
    let tracker = get_global_tracker();
    
    // 导出二进制格式（.memscope扩展名）
    if let Err(e) = tracker.export_to_binary("large_dataset") {
        eprintln!("Binary 导出失败: {}", e);
    } else {
        println!("✅ Binary 导出成功");
        // 文件位置: MemoryAnalysis/large_dataset/large_dataset.memscope
    }
}
```

### Binary → JSON 转换
```rust
use memscope_rs::MemoryTracker;

// 将binary文件转换为标准的5个JSON文件
MemoryTracker::parse_binary_to_standard_json(
    "data.memscope", 
    "converted_data"
)?;

// 或转换为单个JSON文件
MemoryTracker::parse_binary_to_json(
    "data.memscope", 
    "single_file.json"
)?;
```

### Binary → HTML 转换
```rust
use memscope_rs::MemoryTracker;

// 直接从binary生成HTML报告
MemoryTracker::parse_binary_to_html(
    "data.memscope", 
    "report.html"
)?;
```

### Binary 格式配置
```rust
use memscope_rs::BinaryExportConfig;

let config = BinaryExportConfig::new()
    .with_compression(true)          // 启用压缩
    .with_string_deduplication(true) // 字符串去重
    .with_fast_mode(true);           // 快速模式

tracker.export_to_binary_with_config("optimized.memscope", &config)?;
```

### 读取 Binary 文件
```rust
use memscope_rs::BinaryReader;

// 读取二进制文件
let reader = BinaryReader::from_file("data.memscope")?;
let allocations = reader.read_allocations()?;
let stats = reader.read_stats()?;

// 转换为其他格式
reader.export_to_json("converted.json")?;
reader.export_to_html("converted.html")?;
```

## 🔧 批量导出

### 导出所有格式
```rust
use memscope_rs::{get_global_tracker, ExportOptions};

fn export_all_formats(base_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tracker = get_global_tracker();
    
    // JSON 数据
    tracker.export_to_json(base_name)?;
    
    // SVG 图表
    tracker.export_memory_analysis(&format!("{}.svg", base_name))?;
    
    // HTML 仪表板
    tracker.export_to_html(&format!("{}.html", base_name))?;
    
    // Binary 数据
    tracker.export_to_binary(&format!("{}.memscope", base_name))?;
    
    println!("✅ 所有格式导出完成");
    Ok(())
}

// 使用
export_all_formats("complete_analysis")?;
```

### 性能优化导出
```rust
use memscope_rs::ExportOptions;

// 快速导出（适合大数据量）
let fast_options = ExportOptions::new()
    .with_fast_mode(true)
    .with_minimal_analysis(true)
    .with_compression(true);

tracker.export_to_json_with_options("fast_export", &fast_options)?;

// 详细导出（适合深度分析）
let detailed_options = ExportOptions::new()
    .with_detailed_analysis(true)
    .with_stack_traces(true)
    .with_thread_info(true)
    .with_circular_reference_detection(true);

tracker.export_to_json_with_options("detailed_export", &detailed_options)?;
```

## 📁 文件组织

### 默认目录结构
```
MemoryAnalysis/
├── my_analysis/
│   ├── my_analysis_memory_analysis.json
│   ├── my_analysis.svg
│   ├── my_analysis.html
│   └── my_analysis.memscope
├── performance_test/
│   └── ...
└── debug_session/
    └── ...
```

### 自定义输出目录
```rust
use memscope_rs::ExportOptions;

let options = ExportOptions::new()
    .with_output_directory("custom_reports")
    .with_create_subdirectory(false);

tracker.export_to_json_with_options("analysis", &options)?;
// 输出到: custom_reports/analysis_memory_analysis.json
```

## 🎯 使用建议

### 开发阶段
```rust
// 快速迭代 - 使用 SVG
tracker.export_memory_analysis("debug.svg")?;
```

### 详细分析
```rust
// 深度分析 - 使用 HTML
tracker.export_to_html("detailed_analysis.html")?;
```

### 自动化处理
```rust
// 数据处理 - 使用 JSON
tracker.export_to_json("automated_analysis")?;
```

### 性能关键
```rust
// 大数据量 - 使用 Binary
tracker.export_to_binary("performance_data.memscope")?;
```

选择合适的导出格式，让内存分析更高效！ 🚀