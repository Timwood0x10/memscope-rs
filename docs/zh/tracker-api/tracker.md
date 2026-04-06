# Tracker API

> 高级简化接口，含系统监控与热点分析

---

## 概述

**文件:** `src/tracker.rs`

Tracker API 是构建在引擎流水线之上的**高级、用户友好的接口**。它提供基于宏的简单 API，用于快速内存追踪，内置系统监控、采样和热点分析。

---

## 快速开始

```rust
use memscope_rs::{tracker, track};

// 使用宏创建追踪器
let t = tracker!();

// 追踪变量
let data = vec![1, 2, 3, 4, 5];
track!(t, data);

// 分析
let report = t.analyze();
println!("热点: {:?}", report.hotspots);
```

---

## `tracker!` 宏

```rust
// tracker.rs
#[macro_export]
macro_rules! tracker {
    () => {
        $crate::Tracker::new()
    };
}
```

创建一个新的 `Tracker` 实例，使用默认配置。

---

## `track!` 宏

```rust
// tracker.rs
#[macro_export]
macro_rules! track {
    ($tracker:expr, $var:ident) => {
        $tracker.track_variable(
            &$var,
            stringify!($var),
            file!(),
            line!(),
        );
    };
}
```

自动捕获变量名、文件和行号。在追踪器上调用 `track_variable()`，传入变量引用及其字符串化名称。

---

## Tracker 结构体

```rust
// tracker.rs
pub struct Tracker {
    config: Arc<Mutex<TrackerConfig>>,
    allocations: Arc<Mutex<HashMap<usize, TrackedAllocation>>>,
    system_monitor: Option<Arc<SystemMonitor>>,
    hotspots: Arc<Mutex<Vec<AllocationHotspot>>>,
}
```

### 配置

```rust
pub struct TrackerConfig {
    pub sampling_rate: f64,           // 0.0-1.0, 追踪的比例
    pub auto_export_on_drop: bool,    // 追踪器被 drop 时导出
    pub export_path: Option<String>,  // drop 时导出到哪里
    pub enable_system_monitoring: bool, // 收集系统指标
    pub max_hotspots: usize,          // 保留的最大热点条目数
}
```

### 构建器模式

```rust
let tracker = Tracker::new()
    .with_sampling(0.5)              // 追踪 50% 的分配
    .with_system_monitoring()        // 启用 CPU/内存/磁盘监控
    .with_auto_export("output.json"); // drop 时自动导出
```

---

## 变量追踪

```rust
pub fn track_variable<T: Trackable>(
    &self,
    value: &T,
    name: &str,
    file: &str,
    line: u32,
) {
    let ptr = value.get_heap_ptr();
    let size = value.get_size_estimate();
    let type_name = value.get_type_name();

    let allocation = TrackedAllocation {
        ptr,
        size,
        var_name: name.to_string(),
        type_name: type_name.to_string(),
        file: file.to_string(),
        line,
        timestamp: now(),
    };

    self.allocations.lock().unwrap().insert(ptr, allocation);
}
```

使用 `Trackable` trait 从任何被追踪的值中提取堆指针、大小估计和类型名。

---

## Trackable Trait

**文件:** `src/lib.rs`

```rust
pub trait Trackable {
    fn get_heap_ptr(&self) -> Option<usize>;
    fn get_type_name(&self) -> &'static str;
    fn get_size_estimate(&self) -> usize;
    fn get_ref_count(&self) -> Option<usize> { None }
    fn get_data_ptr(&self) -> Option<usize>;
    fn get_data_size(&self) -> Option<usize>;
}
```

### 内置实现

已为以下类型实现：`Vec<T>`, `String`, `HashMap<K,V>`, `BTreeMap<K,V>`, `VecDeque<T>`, `Box<T>`, `Rc<T>`, `Arc<T>`, `RefCell<T>`, `RwLock<T>`。

### 派生宏

```rust
// 功能: "derive"
#[derive(Trackable)]
struct MyStruct {
    data: Vec<u8>,
    name: String,
}
```

---

## 系统监控

**文件:** `src/tracker.rs` + `src/capture/system_monitor.rs`

Tracker 可以 alongside 内存数据收集实时系统指标：

```rust
pub struct SystemSnapshot {
    pub cpu_usage: f64,           // 0.0-100.0%
    pub memory_rss: usize,        // 常驻集大小 (字节)
    pub memory_virtual: usize,    // 虚拟内存大小 (字节)
    pub disk_read_bytes: u64,     // 总读取字节
    pub disk_write_bytes: u64,    // 总写入字节
    pub network_rx_bytes: u64,    // 接收字节
    pub network_tx_bytes: u64,    // 发送字节
    pub gpu_memory: Option<u64>,  // GPU 内存 (如果可用)
}
```

### 工作原理

```rust
// tracker.rs
pub fn with_system_monitoring(mut self) -> Self {
    self.system_monitor = Some(SystemMonitor::global());
    self
}
```

`SystemMonitor::global()` 返回一个 `'static` 单例，运行后台线程以固定间隔收集系统指标。

**实现:** 使用平台特定 API：
- **macOS:** `host_statistics64` 用于 CPU/内存，`sysctl` 用于磁盘/网络
- **Linux:** `/proc/stat`, `/proc/meminfo`, `/proc/diskstats`
- **Windows:** `GetSystemTimes`, `GlobalMemoryStatusEx`

---

## 分析

```rust
pub fn analyze(&self) -> AnalysisReport {
    let allocations = self.allocations.lock().unwrap();
    let alloc_vec: Vec<_> = allocations.values().cloned().collect();

    // 构建热点分析
    let hotspots = self.build_hotspots(&alloc_vec);

    // 收集系统快照
    let system_snapshots = if let Some(ref monitor) = self.system_monitor {
        vec![monitor.get_current_snapshot()]
    } else {
        vec![]
    };

    // 计算峰值内存 (对 broken stats.peak_memory 的 workaround)
    let current_memory: usize = alloc_vec.iter().map(|a| a.size).sum();
    let peak_memory = current_memory;  // 已知限制

    AnalysisReport {
        hotspots,
        system_snapshots,
        total_allocations: alloc_vec.len(),
        total_memory: current_memory,
        peak_memory,
    }
}
```

### 分析报告

```rust
pub struct AnalysisReport {
    pub hotspots: Vec<AllocationHotspot>,
    pub system_snapshots: Vec<SystemSnapshot>,
    pub total_allocations: usize,
    pub total_memory: usize,
    pub peak_memory: usize,
}

pub struct AllocationHotspot {
    pub file: String,
    pub line: u32,
    pub function: String,
    pub allocation_count: usize,
    pub total_bytes: usize,
    pub average_size: f64,
}
```

---

## Drop 时自动导出

```rust
impl Drop for Tracker {
    fn drop(&mut self) {
        if let Ok(cfg) = self.config.lock() {
            if cfg.auto_export_on_drop {
                if let Some(ref path) = cfg.export_path {
                    // 导出到 JSON
                    export_snapshot_to_json(&snapshot, Path::new(path), &options);
                }
            }
        }
    }
}
```

当追踪器被 drop 时，如果配置了则自动导出结果。**注意:** 这在 `Drop` 中执行阻塞 I/O，可能很慢，并且在 panic 展开期间可能出现问题。

---

## 性能

| 操作 | 复杂度 | 说明 |
|------|--------|------|
| `track!()` | O(1) | HashMap 插入 |
| `analyze()` | O(n) | 迭代所有分配 |
| 系统监控 | 后台线程 | 每个采样间隔约 1ms |
| 自动导出 | O(n) | 完整快照序列化 |

---

## 何时使用

- **快速脚本** — 不需要完整 9 引擎流水线时
- **简单程序** — 单线程、低复杂度应用
- **系统监控** — 需要 CPU/内存/磁盘指标 alongside 内存追踪时
- **热点分析** — 想识别哪些源码行分配最多时
