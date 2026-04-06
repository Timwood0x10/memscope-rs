# 捕获后端详解

> MemScope 如何捕获内存分配事件 — 四种策略，一个 trait

---

## CaptureBackend Trait

**文件:** `src/capture/backends/mod.rs:133-157`

所有捕获后端都实现这个统一的 trait：

```rust
pub trait CaptureBackend: Send + Sync {
    fn capture_alloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent;
    fn capture_dealloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent;
    fn capture_realloc(&self, ptr: usize, old_size: usize, new_size: usize, thread_id: u64) -> MemoryEvent;
    fn capture_move(&self, from_ptr: usize, to_ptr: usize, size: usize, thread_id: u64) -> MemoryEvent;
}
```

每个方法接收原始分配参数，返回一个 `MemoryEvent` 推送到 `EventStore`。该 trait 是 `Send + Sync` 的，所以后端可以在线程间共享。

---

## 后端 1：Core

**文件:** `src/capture/backends/core_tracker.rs`

### 设计

Core 后端是**功能丰富的原始追踪器**。它将每个分配存储在并发的 `DashMap` 中，并维护原子计数器用于实时统计。

### 核心数据结构

```rust
// core_tracker.rs:55-65
pub struct MemoryTracker {
    active_allocations: DashMap<usize, AllocationInfo>,  // ptr → 完整信息
    total_allocations: AtomicU64,
    total_allocated: AtomicU64,
    total_deallocations: AtomicU64,
    total_deallocated: AtomicU64,
    peak_memory: AtomicU64,
    peak_allocations: AtomicUsize,
}
```

- **`DashMap<usize, AllocationInfo>`** — 分片并发 HashMap。每个分片有自己的读写锁，比单个 `Mutex<HashMap>` 提供更好的并发读性能。
- **`AtomicU64` 计数器** — 所有统计通过 `fetch_add` 无锁更新。
- **`AtomicUsize` 的 `peak_allocations`** — 追踪同时活跃分配的最大数量。

### 分配流程（源码级）

```rust
// core_tracker.rs:92-120
pub fn track_allocation(&self, ptr: usize, size: usize, ...) -> TrackingResult<()> {
    // 1. 构建包含 25+ 字段的 AllocationInfo
    let alloc_info = AllocationInfo {
        ptr, size,
        var_name, type_name,
        thread_id: thread::current().id(),
        allocated_at_ns: timestamp_ns(),
        stack_trace, borrow_info, clone_info, ...
    };

    // 2. 插入 DashMap — 每分片无锁
    self.active_allocations.insert(ptr, alloc_info);

    // 3. 更新原子计数器 — 无锁
    self.total_allocations.fetch_add(1, Ordering::Relaxed);
    self.total_allocated.fetch_add(size as u64, Ordering::Relaxed);

    // 4. 更新峰值 — TOCTOU 竞态（已知问题）
    let current = self.active_allocations.len();
    let peak = self.peak_allocations.load(Ordering::Relaxed);
    if current > peak {
        self.peak_allocations.store(current, Ordering::Relaxed);
    }
    Ok(())
}
```

### 释放流程

```rust
// core_tracker.rs:126-136
pub fn track_deallocation(&self, ptr: usize) -> TrackingResult<()> {
    if let Some((_, allocation)) = self.active_allocations.remove(&ptr) {
        self.total_deallocations.fetch_add(1, Ordering::Relaxed);
        self.total_deallocated.fetch_add(allocation.size as u64, Ordering::Relaxed);
    }
    Ok(())  // 静默忽略未知指针
}
```

### 线程模型

两种模式（`core_tracker.rs:14-32`）：

```rust
// 模式 1：全局单例
static GLOBAL_TRACKER: OnceLock<Arc<MemoryTracker>> = OnceLock::new();

// 模式 2：线程本地注册表
static LOCAL_THREAD_REGISTRY: OnceLock<DashMap<ThreadId, Arc<MemoryTracker>>> = OnceLock::new();
```

### 性能

| 指标 | 值 |
|------|-----|
| 单分配内存 | ~80 字节 (AllocationInfo) + ~32 字节 (DashMap 节点) |
| 追踪开销 | 5-10% |
| 查询延迟 | O(1) 获取统计，O(n) 迭代 |
| 线程安全 | DashMap 分片锁 — 适合读多写少场景 |

### 何时使用

单线程或低并发程序 (< 8 线程)，需要每个分配的**丰富类型元数据**。

---

## 后端 2：Lockfree

**文件:** `src/capture/backends/lockfree_tracker.rs`

### 设计

Lockfree 后端专为**高并发生产环境**设计。它使用线程本地存储，在热路径中完全避免跨线程同步，并通过概率采样限制内存使用。

### 核心数据结构

```rust
// lockfree_tracker.rs:28-45
pub struct ThreadLocalTracker {
    events: Arc<Mutex<Vec<Event>>>,                    // 共享事件缓冲区
    active_allocations: Arc<Mutex<HashMap<usize, usize>>>,  // ptr → size
    stats: Arc<Mutex<MemoryStats>>,                    // 聚合统计
    sample_rate: f64,                                  // 0.0-1.0
    total_seen: AtomicUsize,                           // 看到的所有分配
    total_tracked: AtomicUsize,                        // 实际追踪的
}
```

### 分配流程（源码级）

```rust
// lockfree_tracker.rs:71-106
pub fn track_allocation(&mut self, ptr: usize, size: usize, ...) {
    // 1. 采样检查 — 跳过大多数小分配
    if size < 1024 && !self.should_sample() {
        self.total_seen.fetch_add(1, Ordering::Relaxed);
        return;  // 丢弃 — 零开销
    }

    // 2. 非阻塞插入 — 如果锁不可用则丢弃事件
    if let Ok(mut events) = self.events.try_lock() {
        events.push(Event {
            timestamp: now(),
            event_type: EventType::Allocation,
            ptr, size,
            call_stack_hash: self.hash_call_stack(),
            thread_id: self.thread_id,
        });
        self.total_tracked.fetch_add(1, Ordering::Relaxed);
    }
    // 如果 try_lock() 失败 → 事件静默丢弃（优雅降级）
}
```

### 采样策略

```rust
// lockfree_tracker.rs:76-79
fn should_sample(&self) -> bool {
    rand::random::<f64>() < self.sample_rate
}
```

在 `sample_rate = 0.1` 时，大约 10% 的小分配被追踪。大分配（≥1KB）绕过采样，始终被追踪。

### 线程本地存储

```rust
// lockfree_tracker.rs:358-360
thread_local! {
    static THREAD_TRACKER: RefCell<Option<ThreadLocalTracker>> = const { RefCell::new(None) };
}
```

每个线程在首次使用时初始化自己的追踪器。**追踪期间零跨线程同步**。

### 性能

| 指标 | 值 |
|------|-----|
| 单事件内存 | ~64 字节 (简化的 Event 结构体) |
| 追踪开销 | 2-5% |
| 内存有界 | 是 — 事件可以刷到磁盘 |
| 线程安全 | 线程本地 — 零竞争 |

### 何时使用

> 8 线程的多线程程序、生产环境中开销必须最小化、长运行进程。

---

## 后端 3：Async

**文件:** `src/capture/backends/async_tracker.rs`

### 设计

Async 后端**按异步任务**追踪内存，而不是按 OS 线程。这很关键，因为异步任务会在 OS 线程之间迁移。

### 任务识别

```rust
// async_types.rs:470-484
pub fn generate_task_id(cx: &Context<'_>) -> AsyncResult<TaskId> {
    let waker_addr = cx.waker() as *const _ as u64;
    let epoch = TASK_EPOCH.fetch_add(1, Ordering::Relaxed);
    // 128 位 ID: (epoch << 64) | waker_addr
    let task_id = ((epoch as u128) << 64) | (waker_addr as u128);
    Ok(task_id)
}
```

任务 ID 是一个 **128 位组合**：单调递增的 epoch 计数器（高 64 位）+ waker 指针地址（低 64 位）。这保证了即使运行时重用 waker 地址也能保证唯一性。

### TrackedFuture 包装器

```rust
// async_types.rs:375-407
impl<F> Future for TrackedFuture<F> where F: Future {
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 首次 poll 时生成任务 ID
        if self.task_id.is_none() {
            self.task_id = Some(generate_task_id(cx)?);
        }

        // 在线程本地存储中设置任务上下文
        let task_info = ExtendedTaskInfo::new(self.task_id, None);
        set_current_task(task_info);

        let result = self.inner.as_mut().poll(cx);

        // future 完成时清除上下文
        if result.is_ready() {
            clear_current_task();
        }
        result
    }
}
```

每次 `poll()` 调用都设置任务上下文，所以 poll 期间的任何分配都归因于正确的任务。

### 分配流程（源码级）

```rust
// async_tracker.rs:227-267
pub fn track_allocation_with_location(
    &self, ptr: usize, size: usize, task_id: u64,
    var_name: Option<String>, type_name: Option<String>,
    source_location: Option<SourceLocation>,
) {
    // 1. 记录分配，关联任务
    let allocation = AsyncAllocation { ptr, size, timestamp, task_id, var_name, type_name, source_location };
    self.allocations.lock().unwrap().insert(ptr, allocation);

    // 2. 更新每任务分析
    if let Some(profile) = self.profiles.lock().unwrap().get_mut(&task_id) {
        profile.record_allocation(size as u64);
    }

    // 3. 更新全局统计
    let mut stats = self.stats.lock().unwrap();
    stats.total_allocations += 1;
    stats.total_memory += size;
    stats.active_memory += size;
    if stats.active_memory > stats.peak_memory {
        stats.peak_memory = stats.active_memory;
    }
}
```

### 任务内存分析

```rust
// task_profile.rs
pub struct TaskMemoryProfile {
    pub task_id: u64,
    pub task_name: String,
    pub task_type: TaskType,  // CpuIntensive, MemoryIntensive, IoIntensive, ...
    pub total_allocations: u64,
    pub total_bytes: u64,
    pub peak_memory: u64,
    pub current_memory: u64,
    pub allocation_rate: f64,
    pub created_at_ms: u64,
    pub completed_at_ms: Option<u64>,
}
```

### 任务效率分析

```rust
// async_tracker.rs:353-436
pub fn analyze_task(&self, task_id: u64, task_type: TaskType) -> Option<TaskReport> {
    // 计算:
    // - cpu_efficiency: 每毫秒分配数 (或 MemoryIntensive 的内存复用率)
    // - memory_efficiency: 分配密度
    // - io_efficiency: 吞吐量 MB/s
    // - efficiency_score: 三者的平均值
    // - bottleneck: "Execution Time" | "Memory" | "Allocations" | "None"
    // - recommendations: 可操作的建议
}
```

### 性能

| 指标 | 值 |
|------|-----|
| 单分配内存 | ~80 字节 (AsyncAllocation) + HashMap 节点开销 |
| 追踪开销 | 10-20% |
| 每任务开销 | ~200 字节 (TaskMemoryProfile) |
| 线程安全 | Mutex (阻塞) |

### 何时使用

异步优先的应用 (tokio, async-std)、每任务内存分析、调试异步内存泄漏。

---

## 后端 4：Unified

**文件:** `src/capture/backends/mod.rs:348-424`

### 设计

Unified 后端是一个**策略选择器** — 它根据运行时环境检测委托给最佳可用后端。

```rust
// mod.rs:362-372
fn detect_best_backend() -> (Box<dyn CaptureBackend>, CaptureBackendType) {
    let thread_count = std::thread::available_parallelism()
        .map(|p| p.get()).unwrap_or(1);

    if thread_count <= 1 {
        (Box::new(CoreBackend), CaptureBackendType::Core)
    } else {
        (Box::new(LockfreeBackend), CaptureBackendType::Lockfree)
    }
    // 注意: AsyncBackend 检测尚未实现
}
```

### 性能

零开销 — 它是一个薄的委托层。实际开销取决于所选后端。

### 何时使用

默认选择。"开箱即用"，无需配置。

---

## 对比矩阵

| 维度 | Core | Lockfree | Async | Unified |
|------|------|----------|-------|---------|
| **同步方式** | DashMap (分片锁) | 线程本地 + try_lock | Mutex (阻塞) | 委托 |
| **单分配内存** | ~80 + 32 字节 | ~64 字节 | ~80 字节 | 可变 |
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

### 通过 CaptureBackendType 枚举

```rust
let backend = CaptureBackendType::Core.create_backend();
let event = backend.capture_alloc(0x1000, 1024, 1);
```
