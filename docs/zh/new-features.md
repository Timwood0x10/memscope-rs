# 新功能与亮点

> MemScope v0.1.10 — 新增功能与核心亮点

---

## 1. 九引擎流水线架构

MemScope 已从单体追踪器重构为**模块化九引擎流水线**：

| 引擎 | 用途 |
|------|------|
| **CaptureEngine** | 可插拔后端 (Core, Lockfree, Async, Unified) |
| **EventStore** | 无锁 `SegQueue` 事件存储 |
| **MetadataEngine** | 作用域、调用栈、智能指针分析 |
| **SnapshotEngine** | 时间点内存快照 |
| **QueryEngine** | 统一查询接口 (汇总, Top-N, 过滤) |
| **TimelineEngine** | 时间序列内存分析与回放 |
| **AnalysisEngine** | 可插拔检测器 (泄漏, UAF, 溢出, 安全, 生命周期) |
| **RenderEngine** | JSON, HTML 仪表盘, SVG, 二进制导出 |
| **Tracker API** | 高级简化接口，含系统监控 |

**为什么重要：** 每个引擎独立可测试、可替换、可组合。你可以只用捕获引擎做轻量追踪，也可以用完整流水线做深度分析。

---

## 2. Unified 自动检测后端

**Unified 后端** 根据运行时环境自动选择最佳捕获策略：

- 单线程 → **Core 后端** (DashMap + 原子计数器)
- 多线程 → **Lockfree 后端** (线程本地 + 采样)
- 检测到异步运行时 → **Async 后端** (任务级分析)

```rust
// 零配置 — 开箱即用
let memscope = MemScope::new();
```

---

## 3. MemScope Facade — 统一入口

新的 `MemScope` 结构体提供**统一 API**，将所有 9 个引擎整合在一起：

```rust
use memscope_rs::facade::MemScope;

let memscope = MemScope::new();

// ... 你的代码在追踪下运行 ...

// 查询
let summary = memscope.summary()?;
let top = memscope.top_allocations(10)?;

// 运行检测器
memscope.run_leak_detector()?;
memscope.run_uaf_detector()?;

// 导出
memscope.export_html("report.html")?;
memscope.export_json("report.json")?;
```

**一个结构体，完全控制。** 不再需要在 `get_global_tracker()`、`get_capture_tracker()` 和手动引擎协调之间来回切换。

---

## 4. 可插拔检测器系统

五个内置检测器， plus 注册自定义检测器的能力：

| 检测器 | 发现什么 |
|--------|----------|
| **LeakDetector** | 没有匹配释放的分配 |
| **UafDetector** | Use-after-free 模式 (释放后访问) |
| **OverflowDetector** | 缓冲区溢出风险 (超出分配边界的写入) |
| **SafetyDetector** | 不安全代码安全违规 |
| **LifecycleDetector** | RAII/Drop 模式分析 |

```rust
// 注册自定义检测器
memscope.register_detector(MyDetector::new())?;

// 运行所有已注册检测器
memscope.run_detectors()?;
```

---

## 5. 系统监控

Tracker API 包含**实时系统监控**：

- **CPU 使用率** — 每核和平均利用率
- **内存** — RSS、虚拟内存、swap 使用
- **磁盘 I/O** — 读/写字节和操作数
- **网络** — 发送/接收字节
- **GPU** — 内存和利用率 (可用时)

```rust
let tracker = tracker!().with_system_monitoring();
let report = tracker.analyze();
println!("CPU: {:.1}%, 内存: {}MB",
    report.system_snapshot.cpu_usage,
    report.system_snapshot.memory_rss / 1024 / 1024);
```

---

## 6. Unsafe 类型推断引擎

一个**基于启发式的类型推断**系统，用于类型信息不可用的 FFI/unsafe 分配：

- **大小启发** — 检测常见类型大小 (8 = 指针, 24 = Vec/String)
- **布局检测** — 识别 Vec/String 的 `(ptr, len, cap)` 三元组
- **内容分析** — Shannon 熵分析二进制数据、零填充检测
- **指针计数** — 区分缓冲区 (0 指针) 和 C 结构体 (2+ 指针)
- **2 的幂信号** — Rust Vec 容量增长模式检测

```rust
use memscope_rs::analysis::unsafe_inference::{
    UnsafeInferenceEngine, TypeKind,
};

let memory = /* 来自 FFI 分配的原始字节 */;
let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, size);
println!("可能类型: {} ({}% 置信度)", guess.kind, guess.confidence);
```

---

## 7. Memory Passport 系统

追踪**跨 FFI 边界的内存所有权**：

- 记录内存何时从 Rust → C 或 C → Rust 跨越
- 追踪所有权转移、借用和归还
- 检测 FFI 边界的泄漏 (Rust 中分配，C 中从未释放，或反之)

---

## 8. 时间线回放

**按时间顺序回放内存事件**，了解内存如何随时间演变：

```rust
use memscope_rs::timeline::TimelineEngine;

let timeline = TimelineEngine::new(&events);
let replay = timeline.replay_until(timestamp);
println!("此时有 {} 个活跃分配", replay.len());
```

---

## 9. HTML 仪表盘

丰富的**交互式 HTML 报告**，包含：

- 随时间变化的内存使用图表
- 按大小和数量排列的 top 分配
- 每线程细分
- 检测结果 (泄漏、UAF 等)
- 类型分布饼图
- 分配时间线可视化

```rust
memscope.export_html("dashboard.html")?;
// 在浏览器中打开 dashboard.html
```

---

## 10. Trackable Trait + 派生宏

为自定义类型实现 `Trackable` 以获得**自动类型感知追踪**：

```rust
// Vec, String, HashMap, Box, Rc, Arc 等的内置实现

// 自定义类型使用派生宏 (feature = "derive")
#[derive(Trackable)]
struct MyStruct {
    data: Vec<u8>,
    name: String,
}
```

该 trait 提供：
- `get_heap_ptr()` — 堆分配地址
- `get_type_name()` — 静态类型名
- `get_size_estimate()` — 估计内存占用
- `get_ref_count()` — 引用计数 (用于 Rc/Arc)
- `get_data_ptr()` / `get_data_size()` — 内部数据位置

---

## 性能基准

| 场景 | 开销 | 内存成本 |
|------|------|----------|
| 单线程 (Core) | 5-10% | ~80 字节/分配 |
| 多线程 (Lockfree) | 2-5% | ~64 字节/事件 |
| 异步 (Async) | 10-20% | ~80 字节/任务 |
| 自动选择 (Unified) | 自适应 | 自适应 |

---

## 从 v0.1.x 迁移

### 旧 API (已弃用)
```rust
memscope_rs::init();
let tracker = get_global_tracker();
tracker.export_to_json("output.json")?;
```

### 新 API (推荐)
```rust
let memscope = MemScope::new();
// ... 代码 ...
memscope.export_json("output.json")?;
```

或用于快速脚本：
```rust
let t = tracker!();
track!(t, my_data);
let report = t.analyze();
```
