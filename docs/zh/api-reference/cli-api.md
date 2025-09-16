# CLI API 参考

memscope-rs 提供了强大的命令行工具，用于内存分析、报告生成和数据处理。

## 🚀 概览

memscope CLI 工具提供以下主要功能：

- **analyze** - 分析程序内存使用情况
- **report** - 从现有数据生成分析报告
- **html-from-json** - 从 JSON 文件生成交互式 HTML 报告
- **test** - 运行增强内存测试

## 📋 基本用法

```bash
# 基本语法
memscope <SUBCOMMAND> [OPTIONS]

# 查看帮助
memscope --help
memscope <SUBCOMMAND> --help
```

## 🔍 analyze 命令

分析程序的内存使用情况。

### 语法

```bash
memscope analyze <COMMAND> [OPTIONS]
```

### 参数

- `<COMMAND>` - 要执行和分析的命令（必需）

### 选项

- `--export <FORMAT>` - 导出格式：json, html, binary, all
  - 默认值：`html`
  - 可选值：`json`, `html`, `binary`, `all`

- `--output <PATH>` - 输出文件路径（不含扩展名）
  - 默认值：`memory_analysis`

### 示例

```bash
# 分析 Rust 程序
memscope analyze cargo run --release

# 分析并导出为 JSON
memscope analyze --export json --output my_analysis cargo run

# 分析并导出所有格式
memscope analyze --export all ./my_program

# 分析带参数的程序
memscope analyze ./my_program arg1 arg2 --flag

# 分析 Python 程序
memscope analyze python my_script.py

# 分析 Node.js 程序
memscope analyze node app.js
```

### 输出文件

根据导出格式，会生成以下文件：

```
MemoryAnalysis/
├── my_analysis_memory_analysis.json    # 基础内存分析
├── my_analysis_lifetime.json           # 生命周期数据
├── my_analysis_performance.json        # 性能指标
├── my_analysis_unsafe_ffi.json         # 不安全/FFI 跟踪
├── my_analysis_complex_types.json      # 复杂类型分析
├── my_analysis.html                     # 交互式 HTML 报告
├── my_analysis.svg                      # SVG 可视化
└── my_analysis.memscope                 # 二进制格式
```

## 📊 report 命令

从现有数据生成内存分析报告。

### 语法

```bash
memscope report --input <INPUT_FILE> --output <OUTPUT_FILE> [OPTIONS]
```

### 参数

- `--input <INPUT_FILE>` - 输入 JSON 文件路径（必需）
- `--output <OUTPUT_FILE>` - 输出报告文件路径（必需）

### 选项

- `--format <FORMAT>` - 输出格式
  - 默认值：`html`
  - 可选值：`html`, `svg`, `pdf`

### 示例

```bash
# 从 JSON 生成 HTML 报告
memscope report --input analysis.json --output report.html

# 生成 SVG 可视化
memscope report --input analysis.json --output chart.svg --format svg

# 使用自定义模板
memscope report --input analysis.json --output custom_report.html --template my_template.html
```

## 🌐 html-from-json 命令

从导出的 JSON 文件生成交互式 HTML 报告，比直接从跟踪器导出快得多。

### 语法

```bash
memscope html-from-json --input-dir <DIR> --output <HTML_FILE> [OPTIONS]
```

### 参数

- `--input-dir <DIR>` - 包含 JSON 文件的输入目录（必需）
- `--output <HTML_FILE>` - 输出 HTML 文件路径（必需）

### 选项

- `--base-name <NAME>` - JSON 文件的基础名称
  - 默认值：`snapshot`

- `--verbose` - 启用详细输出和进度信息

- `--debug` - 启用调试模式，包含详细日志和时间信息

- `--performance` - 启用性能分析模式，包含全面的时间和内存跟踪

- `--validate-only` - 仅验证 JSON 文件，不生成 HTML

### 示例

```bash
# 基本用法
memscope html-from-json --input-dir MemoryAnalysis/my_analysis --output report.html

# 使用自定义基础名称
memscope html-from-json --input-dir ./data --output analysis.html --base-name my_snapshot

# 详细模式
memscope html-from-json --input-dir ./data --output report.html --verbose

# 调试模式
memscope html-from-json --input-dir ./data --output report.html --debug --performance

# 仅验证 JSON 文件
memscope html-from-json --input-dir ./data --validate-only

# 处理大型数据集
memscope html-from-json --input-dir ./large_dataset --output big_report.html --performance
```

### 性能优势

html-from-json 命令相比直接导出 HTML 有显著性能优势：

| 操作 | 直接导出 | html-from-json | 性能提升 |
|------|----------|----------------|----------|
| 小型数据集 (< 1MB) | 2-5 秒 | 0.5-1 秒 | 2-5x |
| 中型数据集 (1-10MB) | 10-30 秒 | 2-5 秒 | 5-6x |
| 大型数据集 (> 10MB) | 60+ 秒 | 5-15 秒 | 4-12x |

## 🧪 test 命令

运行增强内存测试。

### 语法

```bash
memscope test [OPTIONS]
```

### 选项

- `--output <PATH>` - 输出路径
  - 默认值：`enhanced_memory_test`

### 示例

```bash
# 运行基本测试
memscope test

# 指定输出路径
memscope test --output my_test_results

# 运行测试并查看详细输出
memscope test --output test_2024 --verbose
```

## 🔧 全局选项

所有命令都支持以下全局选项：

- `--help` - 显示帮助信息
- `--version` - 显示版本信息

## 📁 输出目录结构

memscope 默认在 `MemoryAnalysis/` 目录下创建输出文件：

```
MemoryAnalysis/
├── <base_name>/
│   ├── <base_name>_memory_analysis.json
│   ├── <base_name>_lifetime.json
│   ├── <base_name>_performance.json
│   ├── <base_name>_unsafe_ffi.json
│   ├── <base_name>_complex_types.json
│   └── <base_name>.memscope
├── <base_name>.html
├── <base_name>.svg
└── logs/
    └── memscope.log
```

## 🌍 环境变量

可以通过环境变量配置 memscope 行为：

```bash
# 启用内存跟踪
export MEMSCOPE_ENABLED=1

# 自动导出
export MEMSCOPE_AUTO_EXPORT=1

# 导出格式
export MEMSCOPE_EXPORT_FORMAT=json

# 导出路径
export MEMSCOPE_EXPORT_PATH=my_analysis

# 自动跟踪
export MEMSCOPE_AUTO_TRACK=1

# 等待完成
export MEMSCOPE_WAIT_COMPLETION=1

# 日志级别
export RUST_LOG=memscope_rs=debug
```

## 📊 性能对比

不同命令的性能特征：

### analyze 命令
- **开销**：5-15% 程序执行时间
- **内存**：额外 10-50MB 内存使用
- **适用**：开发和测试阶段

### html-from-json 命令
- **速度**：比直接 HTML 导出快 4-12 倍
- **内存**：低内存占用，支持大文件
- **适用**：生产环境报告生成

### report 命令
- **速度**：快速报告生成
- **灵活性**：支持多种输出格式
- **适用**：自动化报告流程

## 🔍 高级用法

### 1. 批量分析

```bash
#!/bin/bash
# 批量分析多个程序

programs=("./app1" "./app2" "./app3")

for program in "${programs[@]}"; do
    echo "Analyzing $program..."
    memscope analyze --export all --output "analysis_$(basename $program)" "$program"
done

# 生成汇总报告
memscope html-from-json --input-dir MemoryAnalysis --output summary.html
```

### 2. 持续集成

```yaml
# .github/workflows/memory-analysis.yml
name: Memory Analysis

on: [push, pull_request]

jobs:
  memory-analysis:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install memscope-rs
        run: cargo install memscope-rs
      
      - name: Run memory analysis
        run: |
          memscope analyze --export json cargo test
          memscope html-from-json --input-dir MemoryAnalysis --output memory-report.html
      
      - name: Upload report
        uses: actions/upload-artifact@v2
        with:
          name: memory-analysis-report
          path: memory-report.html
```

### 3. 性能监控

```bash
#!/bin/bash
# 性能监控脚本

# 运行分析
echo "Starting memory analysis..."
time memscope analyze --export binary --output perf_test ./my_app

# 生成快速报告
echo "Generating HTML report..."
time memscope html-from-json --input-dir MemoryAnalysis/perf_test --output perf_report.html --performance

# 显示文件大小
echo "Output files:"
ls -lh MemoryAnalysis/perf_test/
ls -lh perf_report.html
```

## ❌ 错误处理

### 常见错误和解决方案

#### 1. "Command not found"
```bash
# 确保 memscope 在 PATH 中
which memscope

# 如果没有，添加到 PATH 或使用完整路径
export PATH="$HOME/.cargo/bin:$PATH"
```

#### 2. "Permission denied"
```bash
# 检查输出目录权限
ls -la MemoryAnalysis/

# 创建目录并设置权限
mkdir -p MemoryAnalysis
chmod 755 MemoryAnalysis
```

#### 3. "JSON files not found"
```bash
# 检查文件是否存在
ls -la MemoryAnalysis/my_analysis/

# 验证文件名模式
memscope html-from-json --input-dir MemoryAnalysis/my_analysis --validate-only
```

#### 4. "Out of memory"
```bash
# 对于大文件，使用性能模式
memscope html-from-json --input-dir ./large_data --output report.html --performance

# 或者增加系统内存限制
ulimit -v 8388608  # 8GB
```

## 🔗 相关文档

- [跟踪 API 参考](tracking-api.md) - 程序内跟踪接口
- [导出 API 参考](export-api.md) - 数据导出功能
- [CLI 工具指南](../user-guide/cli-tools.md) - CLI 使用指南
- [导出格式指南](../user-guide/export-formats.md) - 输出格式详解

---

CLI 工具让内存分析变得简单高效！ 🎯