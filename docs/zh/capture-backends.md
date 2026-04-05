# 捕获后端详解

> 深入理解 MemScope 的四种数据收集策略

---

## 概述

MemScope 的 `CaptureEngine` 支持四种可插拔后端，每种都针对不同的工作负载模式进行了优化。后端决定了**分配事件如何被捕获、存储和聚合**。

```
                    ┌───────────────┐
                    │ CaptureEngine │
                    └───────┬───────┘
                            │ 路由到
           ┌────────────────┼────────────────┐
           ▼                ▼                ▼
    ┌──────────┐    ┌──────────────┐  ┌───────────┐
    │   Core   │    │  Lockfree    │  │   Async   │
    │ 后端     │    │  后端        │  │  后端     │
    └──────────┘    └──────────────┘  └───────────┘
                          │
                   ┌──────┴──────┐
                   │  Unified    │  ← 自动选择最佳后端
                   │  后端       │
                   └─────────────┘
```

---

## 1. Core 后端

**文件:** `src/capture/backends/core_tracker.rs`

### 工作原理

```
alloc(ptr, size)
  → MemoryTracker::track_allocation()
  → DashMap::insert(ptr, AllocationInfo)    ← 分片并发 HashMap
  → AtomicU64::fetch_add() 更新计数器       ← 无锁统计
```

### 数据结构

- **`DashMap<usize, AllocationInfo>`** — 存储所有活跃分配。DashMap 内部使用分片读写锁，比单个 `Mutex<HashMap>` 提供更好的并发读性能。
- **`AtomicU64` 计数器** — `total_allocations`、`total_allocated`、`total_deallocations`、`total_deallocated`、`peak_memory` — 全部无锁更新。
- **`AtomicUsize` 的 `peak_allocations`** — 追踪同时活跃分配的最大数量。

### 线程模型

由 `STRATEGY_THREAD_LOCAL` 标志控制两种模式：
- **全局单例** — 一个 `MemoryTracker` 通过 `Arc` 在所有线程间共享
- **线程本地** — 每个线程有自己的 `MemoryTracker`，注册在 `DashMap<ThreadId, Arc<MemoryTracker>>` 中

### 优点
- 完整的 `AllocationInfo`，含 25+ 字段（类型名、变量名、调用栈、borrow 信息等）
- 通过 `get_stats()`、`get_active_allocations()` 实时查询
- 丰富的类型信息用于分析

### 缺点
- DashMap 分片锁在极高写入率下引入竞争
- 内存无限增长 — 所有活跃分配保留在内存中
- 峰值分配追踪存在 TOCTOU 竞态条件

### 何时使用
- 单线程程序
- 低到中等并发 (< 8 线程)
- 需要每个分配的丰富类型元数据时

---

## 2. Lockfree 后端

**文件:** `src/capture/backends/lockfree_tracker.rs`

### 工作原理

```
alloc(ptr, size)
  → THREAD_TRACKER.with(|tl| ...)              ← 线程本地追踪器
  → ThreadLocalTracker::track_allocation()
  → 采样检查 (rand::random() < rate)           ← 概率采样
  → Arc<Mutex<Vec<Event>>>::try_lock()         ← 非阻塞
  → 如果获取锁: vec.push(Event)                ← 事件存储
  → 如果获取失败: 事件丢弃                      ← 优雅降级
```

### 数据结构

- **`thread_local! { RefCell<Option<ThreadLocalTracker>> }`** — 每个线程有自己的追踪器，热路径零跨线程同步。
- **`Arc<Mutex<Vec<Event>>>`** — 共享事件缓冲区，通过 `try_lock()` 访问以避免阻塞。
- **`Arc<Mutex<HashMap<usize, usize>>>`** — 活跃分配大小追踪。
- **`AtomicUsize` 的 `total_seen` / `total_tracked`** — 无锁采样统计。

### 采样策略

简单基于速率的采样：
```rust
if rand::random::<f64>() < self.sample_rate {
    // 追踪此分配
}
```

这意味着在 `sample_rate = 0.1` 时，大约 10% 的分配被追踪。大分配始终被追踪（基于大小的覆盖）。

### 优点
- 近乎零竞争 — 追踪期间线程本地
- 内存有界 — 事件可以刷到磁盘
- 竞争下优雅降级（`try_lock` 丢弃事件而非阻塞）

### 缺点
- 更简单的 `Event` 结构体（比 `AllocationInfo` 字段少）
- 采样意味着会遗漏一些分配
- 高竞争下 `try_lock()` 丢弃事件
- 不支持实时查询 — 必须先 finalize

### 何时使用
- 高并发多线程程序 (> 8 线程)
- 生产环境中开销必须最小化
- 长运行进程，内存必须有界

---

## 3. Async 后端

**文件:** `src/capture/backends/async_tracker.rs`

### 工作原理

```
异步任务生成
  → TrackedFuture::poll() 拦截
  → AsyncTracker::track_allocation()
  → Mutex<HashMap<usize, AsyncAllocation>>  ← 按 ptr 键
  → 更新 TaskMemoryProfile (每任务统计)
  → 更新 AsyncStats (全局统计)
```

### 数据结构

- **`Mutex<HashMap<usize, AsyncAllocation>>`** — 活跃分配，按指针键。
- **`Mutex<AsyncStats>`** — 全局统计：总分配数、活跃内存、任务数。
- **`Mutex<HashMap<u64, TaskMemoryProfile>>`** — 每任务内存分析，按任务 ID 键。

### 任务追踪

每个异步任务有自己的 `TaskMemoryProfile`：
- `allocation_count` — 此任务中的总分配数
- `active_memory` — 当前已分配字节
- `peak_memory` — 峰值内存使用
- `efficiency_score` — 资源利用效率

### 优点
- 任务级内存隔离 — 查看哪个异步任务使用了多少内存
- 追踪任务生命周期 (spawn → run → complete)
- 跨任务资源排名

### 缺点
- 全程阻塞 `Mutex` — 高任务并发下竞争
- 无采样 — 追踪所有分配
- 更高开销 (~10-20%) 由于任务上下文管理

### 何时使用
- 异步优先的应用 (tokio, async-std)
- 需要每任务内存分析时
- 调试异步内存泄漏

---

## 4. Unified 后端

**文件:** `src/capture/backends/unified_tracker.rs`

### 工作原理

Unified 后端是一个**策略选择器** — 它本身不收集数据，而是根据运行时环境检测委托给最佳可用后端。

### 自动检测逻辑

```rust
fn detect_best_backend() -> (Box<dyn CaptureBackend>, CaptureBackendType) {
    let thread_count = std::thread::available_parallelism()
        .map(|p| p.get()).unwrap_or(1);

    if thread_count <= 1 {
        (Box::new(CoreBackend), CaptureBackendType::Core)
    } else {
        (Box::new(LockfreeBackend), CaptureBackendType::Lockfree)
    }
    // 注意: AsyncBackend 检测已规划但尚未实现
}
```

### 优点
- 零配置 — "开箱即用"
- 针对当前环境自动优化
- 易于扩展新后端

### 缺点
- 异步检测尚未实现（依赖环境变量）
- 对于混合工作负载可能不是最优选择

### 何时使用
- 大多数用户的默认选择
- 不想考虑后端选择时
- 需要在不同宿主环境中工作的库

---

## 对比矩阵

| 维度 | Core | Lockfree | Async | Unified |
|------|------|----------|-------|---------|
| **同步方式** | DashMap (分片锁) | 线程本地 + try_lock | Mutex (阻塞) | 委托 |
| **单分配开销** | ~80 字节 | ~64 字节 | ~80 字节 | 可变 |
| **追踪开销** | 5-10% | 2-5% | 10-20% | 自动 |
| **丰富类型信息** | ✅ 25+ 字段 | ❌ 极简 | ✅ 每任务 | 可变 |
| **实时查询** | ✅ | ❌ | ✅ | 可变 |
| **采样** | ❌ 全部追踪 | ✅ 基于速率 | ❌ 全部追踪 | 可变 |
| **崩溃恢复** | ❌ | ✅ (如果已刷盘) | ❌ | 可变 |
| **异步任务支持** | ❌ | ❌ | ✅ | 部分 |

---

## 选择正确的后端

```
你的程序是异步的吗？
├── 是 → Async 后端
└── 否
    ├── 单线程或 < 8 线程？
    │   └── 是 → Core 后端
    └── 多线程且 > 8 线程？
        └── 是 → Lockfree 后端

不确定？ → Unified 后端 (自动选择)
```

---

## 切换后端

### 通过 MemScope Facade

```rust
use memscope_rs::facade::MemScope;
use memscope_rs::capture::backends::CaptureBackendType;

// 显式指定后端
let memscope = MemScope::new()
    .with_backend(CaptureBackendType::Lockfree)?;

// 自动选择 (默认)
let memscope = MemScope::new();  // 使用 Unified
```

### 通过环境变量

Unified 后端尊重：
- `TOKIO_WORKER_THREADS` — 检测 tokio 运行时
- `ASYNC_STD_THREAD_COUNT` — 检测 async-std 运行时
