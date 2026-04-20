# 编译期语义增强

## 概述

Phase 0 是 memscope-rs 的核心功能增强阶段，通过编译期宏注入和运行时分析，提供准确的内存布局信息、生命周期追踪和智能报告。

## 完成的功能

### P0.1 编译期宏注入

**目标**：在 `track!` 宏中添加语义锚点（文件名、行号、模块路径）

**实现**：
- 使用 `file!()`, `line!()`, `module_path!()` 宏
- 在 `MemoryEvent` 中记录源码位置信息
- 支持库路径过滤，区分用户代码和库代码

**效果**：
- 精确的源码定位
- 支持过滤 Rust 标准库路径
- 提供模块级别的内存分析

### P0.2 类型布局准确计算

**目标**：使用 `std::mem::Layout::from_val` 获取精确的内存布局信息

**实现**：
- 在 `track!` 宏中调用 `std::mem::size_of_val` 和 `std::mem::align_of_val`
- 记录精确的分配大小和对齐信息
- 支持复杂类型的递归布局计算

**效果**：
- 精确的内存大小计算
- 正确的对齐信息
- 支持泛型类型和自定义类型

### P0.3 基于时间戳的生命周期分析

**目标**：实现基于时间戳的内存生命周期追踪

**实现**：
- 使用 `std::time::Instant` 记录分配时间戳
- 在 Tracker Drop 时自动记录 deallocation 时间戳
- 计算生命周期（毫秒级精度）

**效果**：
- 自动生命周期追踪，无需手动操作
- 支持临时对象检测（短生命周期）
- 支持长生命周期对象分析
- 生命周期分布统计

### P0.4 采样模式

**目标**：实现基于时间或事件的采样机制

**实现**：
- `SamplingConfig` 配置采样率
- 支持按时间间隔采样
- 支持按事件数量采样
- 零开销采样（只读原子值）

**效果**：
- 减少性能开销
- 支持生产环境部署
- 可配置的采样策略

### P0.5 Top N 报告

**目标**：实现基于内存使用量的 Top N 分配报告

**实现**：
- `TopNAnalyzer` 分析器
- Top 分配站点（按内存使用量排序）
- Top 泄漏分配（按泄漏内存排序）
- Top 临时 churn（按分配频率排序）

**效果**：
- 快速定位内存热点
- 优化建议
- 可视化报告

### P0.6 HTML 报告集成

**目标**：利用现有 render_engine/dashboard 能力，集成 Top N 报表展示和 module_path 信息

**实现**：
- 扩展 `DashboardContext` 包含 Top N 报告
- 在 HTML dashboard 中展示模块路径信息
- 集成生命周期分析结果

**效果**：
- 交互式可视化
- 模块级别分析
- 生命周期热力图

## 使用示例

### 基础使用

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // 自动捕获类型信息、时间戳、模块路径
    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);

    // 分析结果
    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    println!("Total allocations: {}", report.stats.allocation_count);
    println!("Total bytes: {}", report.stats.total_bytes);

    Ok(())
}
```

### 带采样的使用

```rust
use memscope_rs::{tracker, tracker::SamplingConfig, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    let tracker = tracker!()
        .with_sampling(SamplingConfig::high_performance());

    // 只有 1% 的分配会被记录
    for i in 0..10000 {
        let data = vec![i; 100];
        track!(tracker, data);
    }

    Ok(())
}
```

### 导出 HTML 报告

```rust
use memscope_rs::{global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);

    // 导出 HTML dashboard
    tracker.render_unified_dashboard("output/dashboard.html")?;

    Ok(())
}
```

### 生命周期分析

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    {
        let temporary_data = vec![1, 2, 3];
        track!(tracker, temporary_data);
        // temporary_data 在此处离开作用域，生命周期自动记录
    }

    let long_lived_data = vec![1; 1000];
    track!(tracker, long_lived_data);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    // 查看生命周期统计
    for alloc in &report.allocations {
        println!("{}: lifetime_ms = {}", alloc.type_name, alloc.lifetime_ms);
    }

    Ok(())
}
```

## 数据流

```
track! 宏
  ↓
MemoryEvent (包含 file, line, module_path, timestamp)
  ↓
event_store (统一数据源)
  ↓
rebuild_allocations_from_events (单一处理流程)
  ↓
AllocationInfo (包含类型布局、生命周期、模块路径)
  ↓
分析器 (TopN、生命周期、循环引用检测)
  ↓
DashboardContext
  ↓
HTML Dashboard
```

## 性能特性

- **零开销采样**：只读原子值，无阻塞
- **编译期计算**：类型布局在编译期计算
- **自动生命周期追踪**：无需手动操作
- **统一数据源**：所有数据来自 event_store
