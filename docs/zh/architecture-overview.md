# Architecture Overview

> MemScope v0.1.10 — 9 引擎流水线架构

***

## 系统架构

MemScope 通过 **9 引擎流水线** 捕获、分析和可视化内存分配：

```
┌─────────────────────────────────────────────────────────────────┐
│                        应用程序代码                              │
│                    (你的 Rust 程序)                              │
└──────────────────────────┬──────────────────────────────────────┘
                           │
              ┌────────────▼────────────┐
              │   #[global_allocator]   │
              │   TrackingAllocator     │
              └────────────┬────────────┘
                           │ alloc / dealloc
              ┌────────────▼────────────┐
              │     CaptureEngine       │  ← 可插拔后端
              │  ┌────┬────┬────┬────┐  │
              │  │Core│Lock│Async│Unified│ │
              │  └────┴────┴────┴────┘  │
              └────────────┬────────────┘
                           │ MemoryEvent
              ┌────────────▼────────────┐
              │       EventStore        │  ← SegQueue (无锁)
              └────────────┬────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────┐
│ MetadataEngine│  │SnapshotEngine │  │ TimelineEngine│
│  作用域,       │  │  时间点        │  │  时间序列      │
│  调用栈        │  │  快照视图      │  │  回放          │
└───────┬───────┘  └───────┬───────┘  └───────┬───────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           ▼
              ┌────────────────────┐
              │    QueryEngine     │  ← 汇总, Top-N, 过滤
              └────────┬───────────┘
                       │
              ┌────────▼───────────┐
              │  AnalysisEngine    │  ← 检测器, 分析器
              │  ┌──────────────┐  │
              │  │ 泄漏检测      │  │
              │  │ UAF 检测     │  │
              │  │ 溢出检测      │  │
              │  │ 安全检测      │  │
              │  │ 生命周期      │  │
              │  └──────────────┘  │
              └────────┬───────────┘
                       │
              ┌────────▼───────────┐
              │    RenderEngine    │  ← JSON, HTML 仪表盘, SVG
              └────────────────────┘
```

***

## 九大引擎

### 1. CaptureEngine — 数据捕获

**模块:** `src/capture/engine.rs`

所有内存事件的入口点。将分配/释放调用路由到可插拔后端，并将事件转发到 EventStore。

**四个后端：**

| 后端           | 文件                    | 策略                                                 | 适用场景                     |
| ------------ | --------------------- | -------------------------------------------------- | ------------------------ |
| **Core**     | `core_tracker.rs`     | `DashMap<usize, AllocationInfo>` + `AtomicU64` 计数器 | 单线程或低并发应用                |
| **Lockfree** | `lockfree_tracker.rs` | `Arc<Mutex<Vec<Event>>>` + `try_lock()` + 线程本地追踪   | 高并发多线程应用                 |
| **Async**    | `async_tracker.rs`    | `Mutex<HashMap<task_id, AsyncAllocation>>`         | 异步运行时 (tokio, async-std) |
| **Unified**  | `unified_tracker.rs`  | 自动检测: CPU 数 → Core/Lockfree; 异步运行时 → Async         | "开箱即用" — 推荐默认            |

**Unified 自动检测逻辑：**

```
线程数 = 1          → Core 后端
线程数 > 1           → Lockfree 后端
检测到 tokio/async-std → Async 后端 (规划中)
```

### 2. EventStore — 集中式事件存储

**模块:** `src/event_store/store.rs`

将所有 `MemoryEvent` 记录存储在无锁 `SegQueue<MemoryEvent>` (来自 crossbeam) 中。每个分配、释放、重新分配和移动事件都经过这里。

**关键操作：**

- `record(event)` — 推送事件到队列 (无锁, O(1))
- `snapshot()` — 清空并恢复以获取时间点视图 (O(n))
- `len()` — 当前事件数 (并发读取)

### 3. MetadataEngine — 上下文信息

**模块:** `src/metadata/`

为原始分配事件添加上下文元数据：

- **作用域追踪** — 变量作用域和生命周期边界
- **线程元数据** — 线程名称、ID、分组
- **调用栈** — 调用栈捕获和标准化
- **智能指针分析** — Rc/Arc 引用计数追踪、clone/borrow 检测

### 4. SnapshotEngine — 时间点视图

**模块:** `src/snapshot/engine.rs`

从 EventStore 构建 `MemorySnapshot`：

- 按指针地址映射的活跃分配
- 每线程统计
- 聚合内存统计 (总计、活跃、峰值)

### 5. QueryEngine — 数据查询

**模块:** `src/query/engine.rs`

统一的快照查询接口：

- `summary()` — 总体内存统计
- `top_allocations(n)` — 按大小排列的前 N 个分配
- `by_thread(thread_id)` — 按线程过滤
- `by_type(type_name)` — 按类型过滤

### 6. TimelineEngine — 时间序列分析

**模块:** `src/timeline/engine.rs`

时间序列内存分析：

- `get_events_in_range(start, end)` — 时间窗口内的事件
- `get_memory_usage_over_time(start, end, interval)` — 内存趋势
- `get_peak_memory_in_range(start, end)` — 峰值内存
- `TimelineReplay` — 按时间顺序回放事件

### 7. AnalysisEngine — 检测器与分析器

**模块:** `src/analysis_engine/engine.rs` + `src/analysis/`

通过 `Detector` trait 实现可插拔分析：

| 检测器                   | 用途                   |
| --------------------- | -------------------- |
| **LeakDetector**      | 查找没有匹配释放的分配          |
| **UafDetector**       | 检测 use-after-free 模式 |
| **OverflowDetector**  | 识别缓冲区溢出风险            |
| **SafetyDetector**    | 通用不安全代码安全违规          |
| **LifecycleDetector** | RAII/Drop 模式分析       |

额外分析模块：

- **Async 分析** — 任务内存分析、效率评分
- **Borrow 分析** — 可变/不可变借用模式检测
- **泛型分析** — 泛型类型实例化统计
- **Closure 分析** — 闭包捕获和生命周期分析
- **Memory Passport** — FFI 边界所有权追踪
- **Unsafe 类型推断** — 原始分配的启发式类型检测

### 8. RenderEngine — 输出生成

**模块:** `src/render_engine/`

多种导出格式：

- **JSON** — 机器可读的分析结果
- **HTML Dashboard** — 交互式 Web 可视化，含图表

### 9. Tracker API — 简化接口

**模块:** `src/tracker.rs`

基于引擎构建的高级、用户友好的 API：

```rust
let tracker = tracker!();
track!(tracker, my_vec);
let report = tracker.analyze();
```

特性：

- **系统监控** — CPU、内存、磁盘 I/O、网络、GPU 指标
- **采样** — 可配置采样率以降低开销
- **自动导出** — Drop 时自动导出
- **热点分析** — 按调用位置识别分配热点

***

## 数据流：端到端

### 场景：在多线程程序中追踪一个 `Vec`

```
1. 用户代码:
   let data = vec![1, 2, 3];
   track_var!(data);

2. TrackingAllocator::alloc() 拦截堆分配
   → ptr: 0x7f..., size: 24, thread_id: 3

3. CaptureEngine 路由到 LockfreeBackend (检测到多线程)
   → ThreadLocalTracker.push(Event { ptr, size, timestamp, ... })

4. 事件转发到 EventStore
   → SegQueue.push(MemoryEvent::Allocate { ... })

5. MetadataEngine 丰富事件信息
   → var_name: "data", type_name: "Vec<i32>", scope: "main"

6. 用户调用: memscope.export_html("report.html")

7. SnapshotEngine 从 EventStore 构建 MemorySnapshot
   → active_allocations: { 0x7f...: ActiveAllocation { ... } }

8. QueryEngine 运行 summary() 和 top_allocations(10)

9. AnalysisEngine 运行所有已注册检测器
   → LeakDetector: 发现 0 个泄漏
   → UafDetector: 发现 0 个问题

10. RenderEngine 生成 HTML 仪表盘
    → 图表、表格、分配时间线、检测结果
```

***

## 两套 API：用哪个？

### MemScope Facade（推荐）

```rust
use memscope_rs::facade::MemScope;

let memscope = MemScope::new();
// ... 你的代码 ...
memscope.export_html("report.html")?;
memscope.run_leak_detector()?;
```

**适合：** 大多数用户。统一访问所有 9 个引擎。

### Tracker 宏（快速简单）

```rust
use memscope_rs::{tracker, track};

let t = tracker!();
track!(t, my_data);
let report = t.analyze();
```

**适合：** 快速脚本、简单程序、不需要完整引擎流水线时。

***

## 功能标志

| 功能                   | 默认 | 描述                          |
| -------------------- | -- | --------------------------- |
| `tracking-allocator` | ✅  | 启用 `#[global_allocator]` 拦截 |
| `backtrace`          | ❌  | 调用栈捕获                       |
| `derive`             | ❌  | `#[derive(Trackable)]` 宏    |
| `test`               | ❌  | 测试工具                        |

***

## 性能特征

| 后端       | 追踪开销     | 内存开销       | 线程安全             |
| -------- | -------- | ---------- | ---------------- |
| Core     | \~5-10%  | \~80 字节/分配 | DashMap (分片锁)    |
| Lockfree | \~2-5%   | \~64 字节/事件 | 线程本地 + try\_lock |
| Async    | \~10-20% | \~80 字节/任务 | Mutex (每任务)      |
| Unified  | 自动选择     | 自动选择       | 自动选择             |

