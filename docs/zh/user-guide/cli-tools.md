# 命令行工具使用指南

memscope-rs 提供了基于 Makefile 的命令行工具，让你可以方便地生成和分析内存报告。

## 🚀 快速开始

### 前置要求

确保你已经构建了项目：

```bash
# 构建项目
cargo build --release
```

### 基本使用流程

```bash
# 1. 运行你的程序生成数据
cargo run --example your_program

# 2. 使用 make 命令生成 HTML 报告
make html DIR=MemoryAnalysis/your_data

# 3. 打开生成的报告
open memory_report.html
```

## 📊 make html 命令

生成交互式 HTML 内存分析报告的主要命令。

### 基本语法

```bash
make html [DIR=directory] [OUTPUT=filename] [BASE=basename] [OPTIONS]
```

### 参数说明

| 参数 | 描述 | 默认值 | 示例 |
|------|------|--------|------|
| `DIR` | JSON文件所在目录 | `MemoryAnalysis/basic_usage` | `DIR=MemoryAnalysis/my_app` |
| `OUTPUT` | 输出HTML文件名 | `memory_report.html` | `OUTPUT=my_report.html` |
| `BASE` | JSON文件的基础名称 | `snapshot` | `BASE=my_analysis` |
| `VERBOSE` | 启用详细输出 | 无 | `VERBOSE=1` |
| `DEBUG` | 启用调试模式 | 无 | `DEBUG=1` |
| `PERFORMANCE` | 启用性能分析 | 无 | `PERFORMANCE=1` |

### 使用示例

```bash
# 基础用法 - 使用默认设置
make html

# 指定自定义目录
make html DIR=MemoryAnalysis/advanced_metrics_demo

# 使用正确的基础名称
make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo

# 自定义输出文件名
make html DIR=MemoryAnalysis/my_data OUTPUT=custom_report.html BASE=my_data

# 启用详细输出
make html DIR=MemoryAnalysis/my_data BASE=my_data VERBOSE=1

# 启用调试和性能分析
make html DIR=MemoryAnalysis/my_data BASE=my_data DEBUG=1 PERFORMANCE=1

# 完整示例
make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo OUTPUT=advanced_report.html VERBOSE=1
```

### 文件命名规则

HTML生成器期望找到以下格式的JSON文件：

```
{BASE}_memory_analysis.json
{BASE}_lifetime.json
{BASE}_performance.json
{BASE}_unsafe_ffi.json
{BASE}_complex_types.json
```

例如，如果 `BASE=my_analysis`，则需要：

- `my_analysis_memory_analysis.json`
- `my_analysis_lifetime.json`
- `my_analysis_performance.json`
- `my_analysis_unsafe_ffi.json`
- `my_analysis_complex_types.json`

## 🎯 实际使用示例

### 示例1：基础使用示例

```bash
# 1. 运行基础示例
cargo run --example basic_usage

# 2. 生成HTML报告（注意：basic_usage生成的文件前缀是basic_usage）
make html DIR=MemoryAnalysis/basic_usage BASE=basic_usage

# 3. 查看报告
open memory_report.html
```

### 示例2：高级多线程示例

```bash
# 1. 运行高级示例
cargo run --example advanced_metrics_demo

# 2. 生成HTML报告（使用正确的基础名称）
make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo

# 3. 查看报告
open memory_report.html
```

### 示例3：二进制导出示例

```bash
# 1. 运行二进制导出示例
cargo run --example binary_export_demo

# 2. 生成HTML报告
make html DIR=MemoryAnalysis/binary_demo_example BASE=binary_demo_example

# 3. 查看报告
open memory_report.html
```

## 🔧 其他 Make 命令

### 构建和测试命令

```bash
# 构建项目
make build          # Debug 构建
make release        # Release 构建

# 运行测试
make test           # 所有测试
make test-unit      # 单元测试
make test-integration  # 集成测试
make test-performance  # 性能测试

# 代码质量
make fmt            # 格式化代码
make clippy         # 运行 Clippy 检查
make audit          # 安全审计
```

### 示例运行命令

```bash
# 运行各种示例
make run-basic                    # 基础使用示例
make run-ownership               # 所有权模式演示
make run-unsafe-ffi              # Unsafe/FFI 演示
make run-improved-tracking       # 改进的跟踪展示
make run-speed-test              # 速度测试
make run-memory-stress           # 内存压力测试
make run-lifecycle               # 生命周期示例

# 运行二进制工具
make run-benchmark               # 综合性能基准测试
make run-simple-benchmark        # 简单基准测试
make run-core-performance        # 核心性能评估
```

### HTML 相关命令

```bash
# HTML 生成的不同模式
make html-verbose               # 详细输出模式
make html-debug                 # 调试模式
make html-performance           # 性能分析模式
make html-validate              # 仅验证JSON文件

# 清理HTML文件
make html-clean                 # 清理生成的HTML文件

# 获取帮助
make html-help                  # 显示HTML命令的详细帮助
```

## 📈 演示工作流

### 快速演示

```bash
# 完整的演示流程
make demo
# 这会执行：构建 → 运行基础示例 → 生成HTML报告
```

### 综合演示

```bash
# 全功能演示
make demo-all
# 这会运行多个示例并生成报告
```

### 性能演示

```bash
# 性能评估演示
make perf-demo
# 运行性能基准测试并生成分析报告
```

## 🚨 常见问题和解决方案

### 问题1：找不到JSON文件

```bash
# 错误信息：No JSON files found in directory
# 解决方案：检查目录和基础名称是否正确

# 查看实际生成的文件
ls MemoryAnalysis/your_directory/

# 使用正确的基础名称
make html DIR=MemoryAnalysis/your_directory BASE=actual_base_name
```

### 问题2：HTML报告显示错误

```bash
# 如果HTML报告中的图表显示错误，可能是基础名称不匹配
# 确保BASE参数与实际的JSON文件前缀匹配

# 例如，如果文件是 advanced_metrics_demo_*.json
make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo
```

### 问题3：权限问题

```bash
# 确保有执行权限
chmod +x target/release/memscope-rs

# 确保输出目录可写
mkdir -p reports && chmod +w reports
```

## 💡 最佳实践

### 1. 文件组织

```bash
# 建议的目录结构
MemoryAnalysis/
├── basic_usage/
│   ├── basic_usage_*.json
│   └── memory_report.html
├── advanced_demo/
│   ├── advanced_demo_*.json
│   └── memory_report.html
└── performance_test/
    ├── performance_test_*.json
    └── memory_report.html
```

### 2. 命名约定

```bash
# 保持一致的命名
cargo run --example my_feature
make html DIR=MemoryAnalysis/my_feature BASE=my_feature
```

### 3. 自动化脚本

```bash
#!/bin/bash
# 自动化分析脚本

EXAMPLE_NAME="advanced_metrics_demo"

echo "运行示例: $EXAMPLE_NAME"
cargo run --example $EXAMPLE_NAME

echo "生成HTML报告"
make html DIR=MemoryAnalysis/$EXAMPLE_NAME BASE=$EXAMPLE_NAME VERBOSE=1

echo "报告生成完成: memory_report.html"
open memory_report.html
```

### 4. 批量处理

```bash
#!/bin/bash
# 批量生成报告

for dir in MemoryAnalysis/*/; do
    if [ -d "$dir" ]; then
        dirname=$(basename "$dir")
        echo "处理目录: $dirname"
        make html DIR="$dir" BASE="$dirname" OUTPUT="${dirname}_report.html"
    fi
done
```

## 🔗 相关文档

- [导出格式说明](export-formats.md) - 了解各种导出格式
- [快速开始](../getting-started/quick-start.md) - 基础使用指南
- [并发分析示例](../examples/concurrent-analysis.md) - 多线程分析示例

## 📋 命令速查表

| 任务 | 命令 |
|------|------|
| 运行基础示例 | `cargo run --example basic_usage` |
| 生成HTML报告 | `make html DIR=path BASE=name` |
| 运行高级示例 | `cargo run --example advanced_metrics_demo` |
| 清理HTML文件 | `make html-clean` |
| 获取帮助 | `make html-help` |
| 快速演示 | `make demo` |
| 构建项目 | `make build` |
| 运行测试 | `make test` |

---

使用这些命令行工具，让内存分析变得更加高效和自动化！ 🎯
