# MemScope 统一后端用户指南

## 概述

MemScope 统一后端提供智能内存跟踪，可跨不同运行时环境自动选择最优策略，支持单线程、多线程、异步和混合应用程序。

## 快速开始

### 安装

```bash
cargo install memscope-rs
```

### 基本用法

```bash
# 使用自动策略选择进行分析
memscope analyze --mode auto your_program

# 使用统一跟踪运行程序
memscope run --track-async cargo run --bin your_async_app
```

## 命令参考

### Analyze 命令

带有统一后端支持的增强分析：

```bash
memscope analyze [选项] <命令>...
```

#### 选项说明

- `--mode <模式>`: 跟踪模式
  - `unified`: 使用统一后端（推荐）
  - `legacy`: 使用传统跟踪系统
  - `auto`: 自动检测（默认）

- `--strategy <策略>`: 后端策略
  - `single-thread`: 单线程应用优化
  - `thread-local`: 多线程本地存储
  - `async`: 异步感知跟踪，支持 Tokio/async-std
  - `hybrid`: 复杂环境自适应策略
  - `auto`: 自动选择（默认）

- `--sample-rate <比率>`: 采样率（0.0-1.0，默认：1.0）
- `-e, --export <格式>`: 导出格式（json, html, svg）
- `-o, --output <文件>`: 输出文件路径

#### 使用示例

```bash
# 使用自动检测分析 Rust 程序
memscope analyze --mode auto cargo run --bin my_app

# 为异步应用使用特定策略
memscope analyze --strategy async --export html tokio_app

# 高频采样用于详细分析
memscope analyze --sample-rate 1.0 --export json intensive_app

# 多线程应用分析
memscope analyze --strategy thread-local parallel_app --jobs 4
```

### Run 命令

使用统一内存跟踪执行程序：

```bash
memscope run [选项] <命令>...
```

#### 选项说明

- `--track-async`: 启用异步任务跟踪
- `--detailed-tracking`: 启用详细分配跟踪
- `--performance-monitoring`: 启用性能监控
- `--max-overhead <MB>`: 最大内存开销（MB，默认：64）
- `-e, --export <格式>`: 导出格式（json, html, svg）
- `-o, --output <文件>`: 输出文件路径

#### 使用示例

```bash
# 使用完整跟踪运行异步应用
memscope run --track-async --detailed-tracking cargo run --bin async_server

# 性能监控模式
memscope run --performance-monitoring --max-overhead 32 cargo test

# 使用完整功能集
memscope run --track-async --detailed-tracking --performance-monitoring my_program
```

## API 参考

### 核心类型

#### UnifiedBackend

内存跟踪的主要协调器：

```rust
use memscope_rs::unified::{UnifiedBackend, BackendConfig};

let config = BackendConfig {
    auto_detect: true,
    force_strategy: None,
    sample_rate: 1.0,
    max_overhead_percent: 5.0,
};

let backend = UnifiedBackend::initialize(config)?;
```

#### BackendConfig

统一后端配置：

```rust
pub struct BackendConfig {
    /// 启用自动策略检测
    pub auto_detect: bool,
    /// 强制特定策略（覆盖 auto_detect）
    pub force_strategy: Option<TrackingStrategy>,
    /// 性能优化采样率
    pub sample_rate: f64,
    /// 最大内存开销百分比
    pub max_overhead_percent: f64,
}
```

#### TrackingSession

活动跟踪会话的句柄：

```rust
let session = backend.start_tracking()?;
println!("会话 ID: {}", session.session_id());

// ... 运行你的应用程序 ...

let analysis_data = backend.collect_data()?;
```

### 环境检测

#### 自动环境检测

```rust
use memscope_rs::unified::{detect_environment, RuntimeEnvironment};

let environment = detect_environment()?;
match environment {
    RuntimeEnvironment::SingleThreaded => println!("单线程环境"),
    RuntimeEnvironment::MultiThreaded => println!("多线程环境"),
    RuntimeEnvironment::AsyncRuntime(_) => println!("检测到异步运行时"),
    RuntimeEnvironment::Hybrid => println!("混合环境"),
}
```

#### 手动策略选择

```rust
use memscope_rs::unified::{UnifiedBackend, BackendConfig, TrackingStrategy};

let config = BackendConfig {
    auto_detect: false,
    force_strategy: Some(TrackingStrategy::AsyncOptimized),
    sample_rate: 0.8,
    max_overhead_percent: 3.0,
};

let backend = UnifiedBackend::initialize(config)?;
```

## 集成示例

### 基本集成

```rust
use memscope_rs::unified::{unified_quick_start, test_unified_system};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 快速启动立即使用
    let mut backend = unified_quick_start()?;
    
    // 开始跟踪
    let session = backend.start_tracking()?;
    
    // 你的应用程序代码
    let data = vec![1, 2, 3, 4, 5];
    let processed = data.iter().map(|x| x * 2).collect::<Vec<_>>();
    
    // 收集结果
    let analysis = backend.collect_data()?;
    println!("跟踪了 {} 字节", analysis.raw_data.len());
    
    Ok(())
}
```

### 异步应用集成

```rust
use memscope_rs::unified::{UnifiedBackend, BackendConfig, TrackingStrategy};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = BackendConfig {
        auto_detect: false,
        force_strategy: Some(TrackingStrategy::AsyncOptimized),
        sample_rate: 1.0,
        max_overhead_percent: 5.0,
    };
    
    let mut backend = UnifiedBackend::initialize(config)?;
    let session = backend.start_tracking()?;
    
    // 异步工作负载
    let tasks = (0..10).map(|i| {
        tokio::spawn(async move {
            let data = vec![0; 1024 * i];
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            data.len()
        })
    });
    
    let results = futures::future::try_join_all(tasks).await?;
    println!("完成了 {} 个任务", results.len());
    
    let analysis = backend.collect_data()?;
    println!("分析：收集了 {} 字节", analysis.raw_data.len());
    
    Ok(())
}
```

### 多线程应用集成

```rust
use memscope_rs::unified::{UnifiedBackend, BackendConfig, TrackingStrategy};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = BackendConfig {
        auto_detect: false,
        force_strategy: Some(TrackingStrategy::ThreadLocal),
        sample_rate: 1.0,
        max_overhead_percent: 5.0,
    };
    
    let mut backend = UnifiedBackend::initialize(config)?;
    let session = backend.start_tracking()?;
    
    // 多线程工作负载
    let handles: Vec<_> = (0..4).map(|i| {
        thread::spawn(move || {
            let data = vec![i; 1024];
            data.into_iter().sum::<usize>()
        })
    }).collect();
    
    let results: Vec<_> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    
    println!("线程结果: {:?}", results);
    
    let analysis = backend.collect_data()?;
    println!("分析完成: {} 字节", analysis.raw_data.len());
    
    Ok(())
}
```

## 输出格式

### JSON 导出

```json
{
  "session_id": "session_12345",
  "duration_ms": 1250,
  "strategy_used": "AsyncOptimized",
  "total_allocations": 142,
  "peak_memory_mb": 8.5,
  "data_size_bytes": 2048
}
```

### HTML 报告

生成的 HTML 包含：
- 交互式内存使用图表
- 分配时间线
- 策略性能指标
- 详细分配分解

### SVG 可视化

显示以下矢量图形：
- 内存使用随时间变化
- 分配模式
- 性能热点

## 性能调优

### 采样率优化

```bash
# 高精度（100% 采样）
memscope analyze --sample-rate 1.0 performance_critical_app

# 平衡性能（80% 采样）
memscope analyze --sample-rate 0.8 regular_app

# 低开销（20% 采样）
memscope analyze --sample-rate 0.2 production_app
```

### 内存开销控制

```bash
# 严格内存限制
memscope run --max-overhead 16 memory_constrained_app

# 宽松限制用于详细分析
memscope run --max-overhead 128 analysis_target_app
```

## 故障排除

### 常见问题

1. **内存开销过高**
   - 降低采样率: `--sample-rate 0.5`
   - 降低开销限制: `--max-overhead 32`

2. **异步跟踪问题**
   - 确保设置 `--track-async` 标志
   - 对异步密集型应用使用 `--strategy async`

3. **多线程问题**
   - 使用 `--strategy thread-local` 获得更好的线程支持
   - 启用详细跟踪: `--detailed-tracking`

### 调试模式

```bash
# 启用调试日志
RUST_LOG=debug memscope analyze your_program

# 跟踪级别获得最大详细信息
RUST_LOG=trace memscope run --detailed-tracking your_program
```

## 最佳实践

1. **策略选择**
   - 对一般应用使用 `auto` 模式
   - 对 Tokio/async-std 应用使用 `async`
   - 对 CPU 密集型多线程应用使用 `thread-local`

2. **性能优化**
   - 从默认采样开始（1.0）
   - 降低生产监控的采样
   - 增加开发分析的开销限制

3. **导出格式选择**
   - 使用 `html` 进行交互式分析
   - 使用 `json` 进行程序化处理
   - 使用 `svg` 进行演示和报告

## 高级功能

### 环境分析

```rust
use memscope_rs::unified::{detect_environment_detailed, DetectionConfig};

let config = DetectionConfig {
    deep_analysis: true,
    confidence_threshold: 0.8,
    timeout_ms: 5000,
};

let analysis = detect_environment_detailed(config)?;
println!("置信度: {:.2}", analysis.confidence);
println!("推荐策略: {:?}", analysis.recommended_strategy);
```

### 自定义策略集成

```rust
use memscope_rs::unified::strategies::{StrategyFactory, StrategyPerformance};

let mut factory = StrategyFactory::new();

let performance = StrategyPerformance {
    avg_overhead_percent: 2.1,
    avg_init_time_us: 150.0,
    success_rate: 0.98,
    satisfaction_score: 0.92,
    session_count: 1,
};

factory.record_performance("custom_strategy", performance);
```

## 迁移指南

### 从传统系统到统一后端

1. **更新 CLI 命令**:
   ```bash
   # 旧版本
   memscope analyze my_program
   
   # 新版本
   memscope analyze --mode unified my_program
   ```

2. **更新 API 调用**:
   ```rust
   // 旧版本
   use memscope_rs::core::{MemoryTracker, quick_start};
   
   // 新版本
   use memscope_rs::unified::{UnifiedBackend, unified_quick_start};
   ```

3. **配置迁移**:
   ```rust
   // 旧配置
   let tracker = MemoryTracker::new()?;
   
   // 新配置
   let backend = unified_quick_start()?;
   ```

## 实际应用场景

### Web 服务器监控

```bash
# 监控高并发 Web 服务器
memscope run --track-async --performance-monitoring \
    --max-overhead 32 --export html \
    cargo run --bin web_server

# 分析请求处理内存模式
memscope analyze --strategy async --sample-rate 0.8 \
    wrk -t12 -c400 -d30s http://localhost:8080/
```

### 批处理任务分析

```bash
# 大数据处理任务
memscope analyze --strategy thread-local --sample-rate 0.5 \
    --export json data_processor --input large_dataset.csv

# CPU 密集型计算
memscope run --detailed-tracking --max-overhead 64 \
    scientific_computation --parallel --threads 8
```

### 游戏引擎优化

```bash
# 游戏循环内存分析
memscope analyze --strategy hybrid --sample-rate 1.0 \
    --export svg game_engine --level test_level

# 实时渲染监控
memscope run --performance-monitoring --max-overhead 16 \
    game_client --graphics-quality high
```

### 机器学习工作负载

```bash
# 训练过程监控
memscope run --track-async --detailed-tracking \
    --max-overhead 128 python train_model.py

# 推理服务分析
memscope analyze --strategy async --sample-rate 0.9 \
    model_server --batch-size 32
```

## 配置文件支持

### TOML 配置文件

创建 `memscope.toml`:

```toml
[unified_backend]
auto_detect = true
sample_rate = 0.8
max_overhead_percent = 5.0

[export]
default_format = "html"
output_dir = "./analysis_results"

[logging]
level = "info"
file = "memscope.log"
```

使用配置文件：

```bash
memscope analyze --config memscope.toml your_program
```

## 持续集成支持

### GitHub Actions 示例

```yaml
name: Memory Analysis
on: [push, pull_request]

jobs:
  memory-analysis:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Install MemScope
      run: cargo install memscope-rs
    - name: Run Memory Analysis
      run: |
        memscope analyze --mode unified --export json \
            --output ci_analysis cargo test
    - name: Upload Results
      uses: actions/upload-artifact@v2
      with:
        name: memory-analysis
        path: ci_analysis.json
```

统一后端保持完全向后兼容性，同时提供增强的功能和性能。