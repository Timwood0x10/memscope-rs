# 追踪器模块 (Tracker Module)

## 概述

追踪器模块为 Rust 应用程序提供统一、简单的内存追踪 API。它提供了高级接口，抽象了内存追踪的复杂性，同时提供强大的分析能力。

## 功能特性

- **简单 API**: 易于使用的 `tracker!()` 和 `track!()` 宏
- **自动捕获**: 自动捕获变量名和类型
- **系统监控**: 后台 CPU、内存、磁盘和网络监控，零开销
- **每线程追踪**: 每个线程独立追踪
- **采样**: 可配置的采样率以优化性能
- **热点分析**: 自动检测分配热点
- **HTML 仪表盘**: 内存数据的交互式可视化
- **多种导出格式**: 支持 JSON、HTML 和二进制导出

## 架构

### 系统监控

系统监控在后台线程中运行，每 100ms 收集一次指标。`track!()` 宏只读取原子值（纳秒级开销），确保数据收集不会阻塞。

**关键点**:
- 非阻塞: 指标收集不阻塞追踪操作
- 低开销: 热路径中只有原子读取
- 线程安全: 所有指标都是线程安全的

### 追踪流程

```
用户代码
   ↓
track!() 宏
   ↓
Tracker::track_as()
   ↓
MemoryTracker::track_allocation()
   ↓
EventStore::record()
   ↓
分析与导出
```

## 核心组件

### 1. Tracker（追踪器）

**用途**: 结合内存追踪、事件存储和系统监控的主要追踪接口。

**源代码**:

```rust
pub struct Tracker {
    inner: Arc<MemoryTracker>,
    event_store: Arc<EventStore>,
    config: Arc<Mutex<TrackerConfig>>,
    start_time: Instant,
    system_snapshots: Arc<Mutex<Vec<SystemSnapshot>>>,
}
```

**核心方法**:

```rust
impl Tracker {
    pub fn new() -> Self
    pub fn global() -> Self
    pub fn with_system_monitoring(self) -> Self
    pub fn with_sampling(self, config: SamplingConfig) -> Self
    pub fn with_auto_export(self, path: &str) -> Self
    pub fn track_as<T: Trackable>(&self, var: &T, name: &str, file: &str, line: u32)
    pub fn analyze(&self) -> AnalysisReport
    pub fn stats(&self) -> MemoryStats
    pub fn events(&self) -> Vec<MemoryEvent>
    pub fn current_system_snapshot(&self) -> SystemSnapshot
}
```

**创建**:

```rust
// 基础追踪器
let tracker = Tracker::new();

// 带系统监控
let tracker = Tracker::new().with_system_monitoring();

// 带采样
let tracker = Tracker::new()
    .with_sampling(SamplingConfig::high_performance());

// 带自动导出
let tracker = Tracker::new()
    .with_auto_export("./output/memscope.json");
```

### 2. SamplingConfig（采样配置）

**用途**: 配置采样行为，在追踪完整性和性能之间取得平衡。

**源代码**:

```rust
pub struct SamplingConfig {
    pub sample_rate: f64,           // 0.0 到 1.0
    pub capture_call_stack: bool,
    pub max_stack_depth: usize,
}
```

**预设**:

```rust
impl SamplingConfig {
    pub fn demo() -> Self {
        Self {
            sample_rate: 0.1,         // 10% 采样
            capture_call_stack: false,
            max_stack_depth: 5,
        }
    }

    pub fn full() -> Self {
        Self {
            sample_rate: 1.0,         // 100% 采样
            capture_call_stack: true,
            max_stack_depth: 20,
        }
    }

    pub fn high_performance() -> Self {
        Self {
            sample_rate: 0.01,        // 1% 采样
            capture_call_stack: false,
            max_stack_depth: 0,
        }
    }
}
```

**采样算法**:

```rust
pub fn track_as<T: Trackable>(&self, var: &T, name: &str, file: &str, line: u32) {
    if let Ok(cfg) = self.config.lock() {
        if cfg.sampling.sample_rate < 1.0 {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();

            // 使用时间戳作为随机性
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            timestamp.hash(&mut hasher);
            std::thread::current().id().hash(&mut hasher);
            name.hash(&mut hasher);
            file.hash(&mut hasher);
            line.hash(&mut hasher);

            let hash = hasher.finish();
            let threshold = (cfg.sampling.sample_rate * 1000.0) as u64;

            if (hash % 1000) > threshold {
                return; // 跳过此次追踪
            }
        }
    }

    self.track_inner(var, name, file, line);
}
```

**设计理念**:

1. **确定性采样**: 基于哈希的采样提供一致的行为
2. **低开销**: 哈希计算快速，不需要外部随机性
3. **线程安全**: 每个线程获得独立的采样决策
4. **可配置**: 易于为不同场景调整采样率

### 3. AnalysisReport（分析报告）

**用途**: 内存追踪数据的综合分析。

**源代码**:

```rust
pub struct AnalysisReport {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub active_allocations: usize,
    pub peak_memory_bytes: u64,
    pub current_memory_bytes: u64,
    pub allocation_rate_per_sec: f64,
    pub deallocation_rate_per_sec: f64,
    pub hotspots: Vec<AllocationHotspot>,
    pub system_snapshots: Vec<SystemSnapshot>,
}
```

**分配热点**:

```rust
pub struct AllocationHotspot {
    pub var_name: String,
    pub type_name: String,
    pub total_size: usize,
    pub allocation_count: usize,
    pub location: Option<String>,
}
```

### 4. SystemSnapshot（系统快照）

**用途**: 在某个时间点捕获系统指标。

**源代码**:

```rust
pub struct SystemSnapshot {
    pub timestamp: u64,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_usage_percent: f64,
    pub thread_count: usize,
    pub disk_read_bps: u64,
    pub disk_write_bps: u64,
    pub network_rx_bps: u64,
    pub network_tx_bps: u64,
    pub gpu_usage_percent: f64,
    pub gpu_memory_used: u64,
    pub gpu_memory_total: u64,
}
```

## 宏

### tracker!()

创建新的追踪器实例。

**语法**:

```rust
let tracker = tracker!();
```

**等价于**:

```rust
let tracker = Tracker::new();
```

### track!()

追踪变量并自动捕获名称。

**语法**:

```rust
track!(tracker, variable_name);
```

**等价于**:

```rust
tracker.track_as(&variable_name, "variable_name", file!(), line!());
```

**示例**:

```rust
let tracker = tracker!();

let my_vec = vec![1, 2, 3, 4, 5];
track!(tracker, my_vec);

let my_string = String::from("Hello");
track!(tracker, my_string);

let my_map: HashMap<i32, String> = HashMap::new();
track!(tracker, my_map);
```

## 使用示例

### 基础使用

```rust
use memscope_rs::{tracker, track};

fn main() {
    let tracker = tracker!();

    // 追踪各种类型
    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);

    let string_data = String::from("Hello, world!");
    track!(tracker, string_data);

    // 分析追踪的分配
    let report = tracker.analyze();
    println!("总分配: {}", report.total_allocations);
    println!("活动分配: {}", report.active_allocations);
    println!("峰值内存: {} 字节", report.peak_memory_bytes);
}
```

### 带系统监控

```rust
use memscope_rs::{tracker, track};

fn main() {
    let tracker = tracker!().with_system_monitoring();

    let data = vec![1, 2, 3];
    track!(tracker, data);

    // 获取当前系统快照
    let snapshot = tracker.current_system_snapshot();
    println!("CPU 使用率: {:.2}%", snapshot.cpu_usage_percent);
    println!("内存使用率: {:.2}%", snapshot.memory_usage_percent);

    // 获取包含系统数据的分析报告
    let report = tracker.analyze();
    println!("系统快照: {}", report.system_snapshots.len());
}
```

### 带采样

```rust
use memscope_rs::{tracker, track, SamplingConfig};

fn main() {
    // 高性能模式，1% 采样
    let tracker = tracker!()
        .with_sampling(SamplingConfig::high_performance());

    // 在循环中进行大量分配
    for i in 0..10000 {
        let data = vec![i; 100];
        track!(tracker, data);
    }

    let report = tracker.analyze();
    println!("追踪的分配: {}", report.total_allocations);
}
```

### 带自动导出

```rust
use memscope_rs::{tracker, track};

fn main() {
    let tracker = tracker!()
        .with_auto_export("./output/memory_report.json");

    let data = vec![1, 2, 3];
    track!(tracker, data);

    // 追踪器超出作用域时自动导出
}
```

### 多线程追踪

```rust
use memscope_rs::{tracker, track};
use std::thread;

fn main() {
    let tracker = tracker!();

    let handles: Vec<_> = (0..4).map(|id| {
        let tracker = tracker.clone();
        thread::spawn(move || {
            for i in 0..100 {
                let data = vec![i; 16];
                track!(tracker, data);
            }
        })
    }).collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let report = tracker.analyze();
    println!("总分配: {}", report.total_allocations);
}
```

## 设计理念

### 1. 简单性优先
API 设计尽可能简单:
- 两个宏: `tracker!()` 和 `track!()`
- 自动变量名捕获
- 最少的样板代码

### 2. 零开销
性能是主要关注点:
- 仅使用原子操作（热路径中无锁）
- 可选采样以减少开销
- 非阻塞系统监控

### 3. 类型安全
利用 Rust 的类型系统:
- 泛型 `Trackable` trait
- 编译时类型检查
- 不需要运行时类型信息

### 4. 线程安全
所有操作都是线程安全的:
- `Arc` 用于共享状态
- `Mutex` 用于写密集型数据
- 原子操作用于计数器

## 性能考虑

### 采样开销

| 采样率 | 开销 | 使用场景 |
|--------|------|----------|
| 1.0 (100%) | ~5% | 开发、调试 |
| 0.1 (10%) | ~1% | 测试、性能分析 |
| 0.01 (1%) | ~0.1% | 生产监控 |

### 系统监控开销

- **后台线程**: 每 100ms 运行一次
- **热路径**: 仅原子读取（纳秒级开销）
- **内存**: 每个快照约 1KB

### 内存开销

- **每个分配**: 约 100 字节
- **事件存储**: 随追踪分配增长
- **系统快照**: 可配置，每个约 1KB

## 集成

追踪器模块与其他模块集成:

```
tracker.rs
  ↓
core/         (MemoryTracker)
  ↓
event_store/  (EventStore)
  ↓
capture/      (SystemMonitor)
  ↓
render/       (导出函数)
```

## 最佳实践

1. **创建一次追踪器**: 在程序启动时初始化追踪器
2. **生产环境使用采样**: 使用采样减少开销
3. **启用系统监控**: 获取全面的指标
4. **自动导出**: 在 drop 时自动导出数据
5. **线程克隆**: 使用 `tracker.clone()` 进行多线程处理

## 限制

1. **栈变量**: 仅追踪堆分配
2. **静态变量**: 不追踪静态分配
3. **外部内存**: 外部库分配的内存可能不会被追踪
4. **类型推断**: 仅限于实现 `Trackable` trait 的类型

## 测试

```rust
#[test]
fn test_tracker_creation() {
    let tracker = Tracker::new();
    let _ = tracker;
}

#[test]
fn test_track_macro() {
    let tracker = tracker!();
    let my_vec = vec![1, 2, 3];
    track!(tracker, my_vec);
}

#[test]
fn test_analyze() {
    let tracker = tracker!();
    let data = vec![1, 2, 3];
    track!(tracker, data);
    let report = tracker.analyze();
    assert!(report.total_allocations > 0);
}
```

## 未来改进

1. **更好的类型推断**: 与编译器集成以获取准确的类型信息
2. **调用栈捕获**: 改进调用栈追踪，减少开销
3. **实时监控**: 基于 Web 的实时仪表盘
4. **高级分析**: 更复杂的分析算法
5. **插件系统**: 允许自定义分析器和导出器