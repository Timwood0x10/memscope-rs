# 5分钟快速上手

这个指南将帮你在5分钟内开始使用 memscope-rs 进行内存跟踪和分析。

## 1. 添加依赖 (30秒)

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
memscope-rs = "0.1.10"
```

## 2. 基础使用 (2分钟)

创建一个简单的示例：

```rust
use memscope_rs::{track_var};
use std::rc::Rc;

fn main() {
    // 创建 MemScope 实例
    let memscope = memscope_rs::MemScope::new();

    // 创建一些变量并跟踪它们
    let my_vec = vec![1, 2, 3, 4, 5];
    track_var!(my_vec);  // 零开销跟踪

    let my_string = String::from("Hello, memscope!");
    track_var!(my_string);

    let boxed_data = Box::new(42);
    track_var!(boxed_data);

    // 智能指针跟踪
    let rc_data = Rc::new(vec![10, 20, 30]);
    track_var!(rc_data);

    // 变量依然可以正常使用
    println!("Vector: {:?}", my_vec);
    println!("String: {}", my_string);
    println!("Boxed: {}", *boxed_data);
    println!("RC data: {:?}", *rc_data);

    // 获取内存统计
    if let Ok(stats) = memscope.summary() {
        println!("活跃分配: {}", stats.active_allocations);
        println!("活跃内存: {} bytes", stats.active_memory);
        println!("总分配数: {}", stats.total_allocations);
        println!("峰值内存: {} bytes", stats.peak_memory);
    }
}
```

## 3. 生成分析报告 (2分钟)

添加导出功能：

```rust
use memscope_rs::{track_var};
use std::rc::Rc;

fn main() {
    let memscope = memscope_rs::MemScope::new();

    // 跟踪更多类型的数据
    let numbers = vec![1, 2, 3, 4, 5];
    track_var!(numbers);

    let shared_data = Rc::new(vec!["a", "b", "c"]);
    track_var!(shared_data);

    let shared_clone = Rc::clone(&shared_data);
    track_var!(shared_clone);

    // 1. 导出 JSON 数据
    if let Err(e) = memscope.export_json("my_analysis") {
        eprintln!("JSON 导出失败: {}", e);
    } else {
        println!("✅ JSON 导出成功: MemoryAnalysis/my_analysis/");
    }

    // 2. 导出 HTML 交互式仪表板
    if let Err(e) = memscope.export_html("my_analysis.html") {
        eprintln!("HTML 导出失败: {}", e);
    } else {
        println!("✅ HTML 导出成功: MemoryAnalysis/my_analysis/");
    }
}
```

## 4. 查看结果 (30秒)

运行程序后，检查生成的文件：

```bash
# 运行程序
cargo run

# 查看生成的文件
ls MemoryAnalysis/my_analysis/
# 你会看到:
# - my_analysis_memory_analysis.json  (内存分析数据)
# - my_analysis_lifetime.json         (生命周期数据)
# - my_analysis_performance.json      (性能数据)
# - my_analysis_unsafe_ffi.json       (Unsafe/FFI数据)
# - my_analysis_complex_types.json    (复杂类型数据)
# - my_analysis.svg                   (内存使用图表)
# - my_analysis.html                  (交互式仪表板)
# - my_analysis.memscope              (二进制格式)
```

### 使用 make html 生成增强报告

```bash
# 使用 make 命令生成更丰富的 HTML 报告
make html DIR=MemoryAnalysis/my_analysis BASE=my_analysis

# 打开生成的报告
open memory_report.html  # macOS
# 或者在浏览器中打开 memory_report.html
```

## 🎯 你刚刚学会了什么

✅ **零开销跟踪**: `track_var!` 宏不影响程序性能  
✅ **多种数据类型**: Vec, String, Box, Rc, Arc 等都可以跟踪  
✅ **实时统计**: 获取当前内存使用情况和峰值  
✅ **多种导出格式**: JSON 数据、SVG 图表、HTML 仪表板、二进制格式  
✅ **分类数据**: 5个专门的JSON文件，便于分析不同方面  
✅ **变量依然可用**: 跟踪后变量完全正常使用  
✅ **高性能二进制**: 比JSON快80倍以上的导出格式  

## 🚀 下一步

现在你已经掌握了基础用法，可以继续学习：

- **[基础跟踪使用](basic-tracking.md)** - 深入了解三种跟踪宏
- **[第一次内存分析](first-analysis.md)** - 学会解读分析报告
- **[跟踪宏详解](../user-guide/tracking-macros.md)** - 选择最适合的跟踪方式

## 💡 快速提示

- **性能**: `track_var!` 是零开销的，可以在生产环境使用
- **智能指针**: Rc/Arc 会自动跟踪引用计数变化
- **文件位置**: 所有导出文件都在 `MemoryAnalysis/` 目录下
- **HTML 报告**: 包含可点击的图表和过滤功能
- **二进制格式**: 使用 `.memscope` 扩展名，可转换为JSON或HTML
- **Make 命令**: 使用 `make html` 生成增强的交互式报告
- **多线程**: 支持多线程程序的内存跟踪和分析

## 🔥 高级示例

想看更复杂的用法？运行这些示例：

```bash
# 基础用法示例
cargo run --example basic_usage

# 二进制导出示例
cargo run --example binary_export_demo

# 高级多线程示例
cargo run --example advanced_metrics_demo

# 然后生成HTML报告
make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo
```

开始享受高效的内存分析吧！ 🎉