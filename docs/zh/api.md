# API 文档

## 概述

memscope-rs 提供简单统一的 API 用于内存跟踪、分析和可视化。

## 核心宏

### `track!` 宏

跟踪变量的内存分配。

```rust
track!(tracker, variable);
```

**参数**：
- `tracker`: Tracker 实例
- `variable`: 要跟踪的变量

**功能**：
- 自动捕获类型信息
- 记录分配时间戳
- 记录源码位置（文件、行号、模块路径）
- 自动检测智能指针类型
- 自动追踪生命周期

**示例**：
```rust
use memscope_rs::{global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    let data = vec![1, 2, 3];
    track!(tracker, data);

    Ok(())
}
```

### `tracker!` 宏

创建新的 Tracker 实例。

```rust
let tracker = tracker!();
```

**返回**：新的 Tracker 实例

## Tracker API

### 初始化

#### `init_global_tracking()`

初始化全局跟踪器。

```rust
fn init_global_tracking() -> MemScopeResult<()>
```

**返回**：`MemScopeResult<()>`

**示例**：
```rust
use memscope_rs::init_global_tracking;

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    Ok(())
}
```

#### `global_tracker()`

获取全局跟踪器实例。

```rust
fn global_tracker() -> MemScopeResult<Tracker>
```

**返回**：全局 Tracker 实例

**示例**：
```rust
use memscope_rs::{global_tracker, init_global_tracking};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;
    Ok(())
}
```

### 配置

#### `with_sampling()`

设置采样配置。

```rust
fn with_sampling(self, config: SamplingConfig) -> Self
```

**参数**：
- `config`: 采样配置

**示例**：
```rust
use memscope_rs::{tracker, tracker::SamplingConfig};

let tracker = tracker!().with_sampling(SamplingConfig::demo());
```

#### `with_system_monitoring()`

启用系统监控。

```rust
fn with_system_monitoring(self) -> Self
```

**示例**：
```rust
use memscope_rs::tracker;

let tracker = tracker!().with_system_monitoring();
```

#### `with_auto_export()`

设置自动导出路径。

```rust
fn with_auto_export(self, path: &str) -> Self
```

**参数**：
- `path`: 导出路径

**示例**：
```rust
use memscope_rs::tracker;

let tracker = tracker!().with_auto_export("output");
```

### 跟踪方法

#### `track_as()`

跟踪变量的内存分配。

```rust
fn track_as<T: Trackable>(
    &self,
    var: &T,
    name: &str,
    file: &str,
    line: u32,
    module_path: &str,
) -> usize
```

**参数**：
- `var`: 要跟踪的变量
- `name`: 变量名
- `file`: 文件名
- `line`: 行号
- `module_path`: 模块路径

**返回**：指针地址

**注意**：通常通过 `track!` 宏调用，不直接使用此方法。

### 分析方法

#### `analyze()`

执行完整的内存分析。

```rust
fn analyze(&self) -> AnalysisReport
```

**返回**：分析报告

**示例**：
```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    let data = vec![1, 2, 3];
    track!(tracker, data);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();
    
    println!("Allocations: {}", report.stats.allocation_count);
    println!("Total bytes: {}", report.stats.total_bytes);
    
    Ok(())
}
```

### 导出方法

#### `export_all_json()`

导出所有分析结果为 JSON。

```rust
fn export_all_json(&self, path: impl AsRef<Path>) -> MemScopeResult<()>
```

**参数**：
- `path`: 导出路径

**示例**：
```rust
use memscope_rs::tracker;

let tracker = tracker!();
tracker.export_all_json("output")?;
```

#### `render_unified_dashboard()`

渲染统一的 HTML dashboard。

```rust
fn render_unified_dashboard(&self, path: impl AsRef<Path>) -> MemScopeResult<()>
```

**参数**：
- `path`: 输出路径

**示例**：
```rust
use memscope_rs::tracker;

let tracker = tracker!();
tracker.render_unified_dashboard("output/dashboard.html")?;
```

### 统计方法

#### `get_stats()`

获取当前统计信息。

```rust
fn get_stats(&self) -> TrackerStats
```

**返回**：跟踪器统计信息

**示例**：
```rust
use memscope_rs::tracker;

let tracker = tracker!();
let stats = tracker.get_stats();
println!("Total allocations: {}", stats.total_allocations);
```

## SamplingConfig

采样配置。

```rust
pub struct SamplingConfig {
    pub sample_rate: f64,
    pub capture_call_stack: bool,
    pub max_stack_depth: usize,
}
```

### 预设配置

#### `default()`

默认配置（100% 采样率）。

```rust
SamplingConfig::default()
```

#### `demo()`

演示配置（10% 采样率）。

```rust
SamplingConfig::demo()
```

#### `full()`

完整配置（100% 采样率，捕获调用栈）。

```rust
SamplingConfig::full()
```

#### `high_performance()`

高性能配置（1% 采样率）。

```rust
SamplingConfig::high_performance()
```

## Analyzer API

### `analyzer()`

创建分析器。

```rust
fn analyzer(tracker: &Tracker) -> MemScopeResult<Analyzer>
```

**参数**：
- `tracker`: Tracker 实例

**返回**：分析器实例

**示例**：
```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    let mut az = analyzer(&tracker)?;
    Ok(())
}
```

### 分析方法

#### `analyze()`

执行完整分析。

```rust
fn analyze(&mut self) -> AnalysisReport
```

**返回**：分析报告

#### `top_allocation_sites()`

获取 Top N 分配站点。

```rust
fn top_allocation_sites(&mut self, n: usize) -> Vec<TopAllocationSite>
```

**参数**：
- `n`: 返回数量

**返回**：Top 分配站点列表

#### `top_leaked_allocations()`

获取 Top N 泄漏分配。

```rust
fn top_leaked_allocations(&mut self, n: usize) -> Vec<TopLeakedAllocation>
```

**参数**：
- `n`: 返回数量

**返回**：Top 泄漏分配列表

#### `top_temporary_churn()`

获取 Top N 临时 churn。

```rust
fn top_temporary_churn(&mut self, n: usize) -> Vec<TopTemporaryChurn>
```

**参数**：
- `n`: 返回数量

**返回**：Top 临时 churn 列表

## 数据类型

### AnalysisReport

分析报告。

```rust
pub struct AnalysisReport {
    pub stats: AnalysisStats,
    pub hotspots: Vec<AllocationHotspot>,
    pub leaks: Vec<LeakInfo>,
}
```

### AnalysisStats

分析统计。

```rust
pub struct AnalysisStats {
    pub allocation_count: usize,
    pub total_bytes: usize,
    pub peak_bytes: usize,
    pub thread_count: usize,
}
```

### CircularReferenceReport

循环引用报告。

```rust
pub struct CircularReferenceReport {
    pub count: usize,
    pub total_leaked_memory: usize,
    pub pointers_in_cycles: usize,
    pub total_smart_pointers: usize,
    pub has_cycles: bool,
}
```

## 使用示例

### 基础使用

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();
    
    println!("Allocations: {}", report.stats.allocation_count);
    println!("Total bytes: {}", report.stats.total_bytes);
    
    Ok(())
}
```

### 智能指针追踪

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};
use std::rc::Rc;
use std::sync::Arc;

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    let rc_data = Rc::new(vec![1.0, 2.0, 3.0]);
    track!(tracker, rc_data);

    let arc_data = Arc::new(vec![1.0, 2.0, 3.0]);
    track!(tracker, arc_data);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();
    
    println!("Smart pointers: {}", 
        report.circular_references.total_smart_pointers);
    println!("Circular refs: {}", 
        report.circular_references.count);
    
    Ok(())
}
```

### 导出报告

```rust
use memscope_rs::{global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    let data = vec![1, 2, 3];
    track!(tracker, data);

    tracker.export_all_json("output")?;
    tracker.render_unified_dashboard("output/dashboard.html")?;
    
    Ok(())
}
```

## 性能特性

- **零开销采样**：只读原子值，无阻塞
- **编译期计算**：类型布局在编译期计算
- **自动生命周期追踪**：无需手动操作
- **统一数据源**：所有数据来自 event_store
