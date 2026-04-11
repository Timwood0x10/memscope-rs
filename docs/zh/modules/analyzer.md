# Analyzer 模块

## 概述

Analyzer 模块是 memscope-rs v0.2.0 引入的统一分析入口，整合了所有内存分析功能，提供了一个简洁、高效的分析接口。

## 核心组件

### Analyzer

统一分析入口，整合所有分析模块。

```rust
pub struct Analyzer {
    view: MemoryView,
    graph: Option<GraphAnalysis>,
    detect: Option<DetectionAnalysis>,
    metrics: Option<MetricsAnalysis>,
    timeline: Option<TimelineAnalysis>,
    classify: Option<ClassificationAnalysis>,
    safety: Option<SafetyAnalysis>,
}
```

**特性：**
- 延迟初始化：分析模块只在首次访问时初始化
- 统一接口：所有分析功能通过统一接口访问
- 高效复用：共享 MemoryView，避免重复计算

### 分析模块

| 模块 | 功能 | API |
|------|------|-----|
| **GraphAnalysis** | 图分析和循环检测 | `az.graph()` |
| **DetectionAnalysis** | 泄漏检测、UAF 检测、安全分析 | `az.detect()` |
| **MetricsAnalysis** | 指标分析、统计信息 | `az.metrics()` |
| **TimelineAnalysis** | 时间线分析、事件查询 | `az.timeline()` |
| **ClassificationAnalysis** | 类型分类 | `az.classify()` |
| **SafetyAnalysis** | 安全分析 | `az.safety()` |
| **ExportEngine** | 数据导出 | `az.export()` |

## 使用示例

### 创建 Analyzer

```rust
use memscope_rs::{global_tracker, init_global_tracking, analyzer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化追踪器
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // 创建统一分析器
    let mut az = analyzer(&tracker)?;
    
    Ok(())
}
```

### 图分析

```rust
// 获取图分析
let graph = az.graph();

// 检测循环引用
let cycles = graph.cycles();
println!("循环引用数量: {}", cycles.cycle_count);

// 获取所有权统计
let stats = graph.ownership_stats();
```

### 检测分析

```rust
// 获取检测分析
let detect = az.detect();

// 泄漏检测
let leaks = detect.leaks();
println!("泄漏数量: {}", leaks.leak_count);

// UAF 检测
let uaf = detect.uaf();
println!("UAF 数量: {}", uaf.uaf_count);

// 安全分析
let safety = detect.safety();
println!("安全问题数量: {}", safety.issue_count);
```

### 指标分析

```rust
// 获取指标分析
let metrics = az.metrics();

// 获取摘要
let summary = metrics.summary();

// 按大小排序
let top = metrics.top_by_size(10);

// 按类型统计
let by_type = metrics.by_type();

// 按线程统计
let by_thread = metrics.by_thread();
```

### 快速检查

```rust
// 快速泄漏检查
let leaks = az.quick_leak_check();

// 快速循环检查
let cycles = az.quick_cycle_check();

// 快速指标
let metrics = az.quick_metrics();
```

### 导出

```rust
// 获取导出引擎
let exporter = az.export();

// 导出 JSON
exporter.json("output/analysis.json")?;

// 导出 HTML
exporter.html("output/dashboard.html")?;
```

## 报告类型

### AnalysisReport

完整分析报告。

```rust
pub struct AnalysisReport {
    pub stats: MemoryStatsReport,
    pub leaks: LeakReport,
    pub cycles: CycleReport,
    pub metrics: MetricsReport,
}
```

### LeakReport

泄漏检测报告。

```rust
pub struct LeakReport {
    pub leak_count: usize,
    pub total_leaked_bytes: usize,
    pub leaked_allocations: Vec<LeakInfo>,
}
```

### CycleReport

循环引用报告。

```rust
pub struct CycleReport {
    pub cycle_count: usize,
    pub cycles: Vec<CycleInfo>,
}
```

### MetricsReport

指标报告。

```rust
pub struct MetricsReport {
    pub allocation_count: usize,
    pub total_bytes: usize,
    pub peak_bytes: usize,
    pub thread_count: usize,
    pub by_type: HashMap<String, TypeMetric>,
}
```

## 性能优化

### Lazy Initialization

分析模块只在首次访问时初始化，减少不必要的计算。

```rust
// 第一次访问：初始化 GraphAnalysis
let graph = az.graph();  // ← 这里初始化

// 后续访问：直接返回已初始化的实例
let graph2 = az.graph();  // ← 直接返回，无额外开销
```

### Snapshot 复用

MemoryView 复用 MemorySnapshot，避免重复构建 allocations。

```rust
// MemoryView 复用 Snapshot 的 allocations
let view = MemoryView::from_tracker(tracker);

// 所有分析模块共享同一个 MemoryView
let graph = GraphAnalysis::from_view(&view);
let detect = DetectionAnalysis::from_view(&view);
let metrics = MetricsAnalysis::from_view(&view);
```

## 错误处理

### MemScopeResult

统一使用 `MemScopeResult<T>` 作为返回类型。

```rust
pub type MemScopeResult<T> = Result<T, MemScopeError>;
```

### MemScopeError

统一错误类型，支持错误分类和严重程度。

```rust
pub enum MemScopeError {
    Memory { ... },
    Analysis { ... },
    Export { ... },
    Configuration { ... },
    System { ... },
    Internal { ... },
}
```

### 日志记录

在关键路径添加适当的日志记录。

```rust
use tracing::{error, warn, info, debug};

// 记录错误
error!("Analysis failed: {}", error_message);

// 记录警告
warn!("Potential memory leak detected: {} allocations", count);

// 记录信息
info!("Analysis completed in {}ms", duration);

// 记录调试信息
debug!("Analyzing {} allocations", count);
```

## 最佳实践

1. **使用统一入口** - 优先使用 `analyzer(&tracker)` 而不是直接创建各个分析模块
2. **复用 Analyzer** - 避免重复创建 Analyzer 实例
3. **快速检查优先** - 对于简单场景，使用快速检查方法
4. **错误处理** - 始终处理可能的错误
5. **日志记录** - 在关键路径添加适当的日志

## 相关模块

- [view/](./view.md) - MemoryView 统一读模型
- [snapshot/](./snapshot.md) - MemorySnapshot 快照
- [event_store/](./event_store.md) - EventStore 事件存储
- [core/error.md](./error.md) - 统一错误处理