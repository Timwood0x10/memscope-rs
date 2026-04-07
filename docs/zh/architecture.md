# 架构文档

## 概述

memscope-rs 是一个用于 Rust 应用程序的综合性内存追踪和分析库。它提供了模块化的架构，将关注点分离到多个层级，实现高效的内存监控、分析和可视化。

## 系统架构

```
用户代码
   ↓
外观层 (facade/)
   ↓
捕获引擎 (capture/)
   ↓
事件存储 (event_store/)
   ↓
元数据引擎 (metadata/)
   ↓
快照引擎 (snapshot/)
   ↓
查询引擎 (query/)
   ↓
分析引擎 (analysis/)
   ↓
渲染引擎 (render_engine/)
   ↓
输出 (JSON/HTML/Binary)
```

## 模块职责

### 核心层 (core/)

**用途**: 提供基础内存追踪功能和类型定义。

**组件**:
- `allocator.rs`: 拦截所有堆分配的自定义全局分配器
- `error.rs`: 错误类型定义
- `scope_tracker.rs`: 变量生命周期的作用域追踪
- `safe_operations.rs`: 安全操作工具
- `call_stack_normalizer.rs`: 调用栈规范化
- `unwrap_safe.rs`: 安全解包工具

**核心实现**:

```rust
// src/core/allocator.rs
unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());
            if should_track {
                TRACKING_DISABLED.with(|disabled| disabled.set(true));
                if let Ok(tracker) = std::panic::catch_unwind(crate::core::tracker::get_tracker) {
                    let _ = tracker.track_allocation(ptr as usize, layout.size());
                }
                TRACKING_DISABLED.with(|disabled| disabled.set(false));
            }
        }
        ptr
    }
}
```

**设计原则**:
- **零开销**: 使用线程本地存储标志而非锁
- **递归保护**: 防止无限追踪循环
- **恐慌恢复**: 追踪失败不会导致应用崩溃
- **类型推断**: 基于分配大小追踪类型

### 追踪器层 (tracker.rs)

**用途**: 为用户提供统一的追踪 API 和宏。

**核心功能**:
- 自动变量名和类型捕获
- 系统监控（CPU、内存、磁盘、网络）
- 每线程独立追踪
- 可配置采样率
- 自动热点检测
- JSON/HTML 导出支持

**宏使用**:

```rust
let tracker = tracker!();

let data = vec![1, 2, 3, 4, 5];
track!(tracker, data);
```

**采样支持**:

```rust
pub fn track_as<T: Trackable>(&self, var: &T, name: &str, file: &str, line: u32) {
    if let Ok(cfg) = self.config.lock() {
        if cfg.sampling.sample_rate < 1.0 {
            let hash = compute_hash(timestamp, thread_id, name, file, line);
            let threshold = (cfg.sampling.sample_rate * 1000.0) as u64;
            if (hash % 1000) > threshold {
                return;
            }
        }
    }
    self.track_inner(var, name, file, line);
}
```

### 捕获引擎 (capture/)

**用途**: 使用多种后端策略捕获应用程序的内存事件。

**后端类型**:
- `CoreTracker`: 单线程、简单、低开销
- `LockfreeTracker`: 多线程、线程本地存储
- `AsyncTracker`: 异步任务追踪，带任务 ID
- `UnifiedTracker`: 基于 CPU 核心数自动检测

**核心类型**:

```rust
pub enum CaptureBackendType {
    Core,
    Lockfree,
    Async,
    Unified,
}

pub struct AllocationInfo {
    pub ptr: usize,
    pub size: usize,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
    pub timestamp_alloc: u64,
    pub timestamp_dealloc: Option<u64>,
    pub thread_id: ThreadId,
    pub thread_id_u64: u64,
}
```

**分析功能**:
- 热点分析
- 瓶颈分析
- 任务级内存分析
- 效率评分
- 资源排名

### 事件存储 (event_store/)

**用途**: 使用无锁队列集中存储所有内存事件。

**核心实现**:

```rust
pub struct EventStore {
    // 无锁队列存储事件
}

impl EventStore {
    pub fn record(&self, event: MemoryEvent) {
        // 记录事件到无锁队列
    }

    pub fn snapshot(&self) -> Vec<MemoryEvent> {
        // 获取事件快照
    }
}

pub enum MemoryEventType {
    Allocate,
    Deallocate,
    Reallocate,
    Move,
    Borrow,
    Clone,
}
```

**设计原则**:
- **无锁**: 高性能并发访问
- **仅追加**: 高效事件记录
- **快照支持**: 时间点状态捕获

### 元数据引擎 (metadata/)

**用途**: 集中管理所有元数据，包括变量、作用域和线程。

**组件**:
- `VariableRegistry`: 变量元数据管理
- `ScopeTracker`: 变量生命周期的作用域追踪
- `ThreadRegistry`: 线程元数据管理
- `SmartPointers`: 智能指针信息
- `StackTrace`: 调用栈信息

**核心实现**:

```rust
pub struct MetadataEngine {
    pub variable_registry: Arc<VariableRegistry>,
    pub scope_tracker: Arc<ScopeTracker>,
    pub thread_registry: Arc<ThreadRegistry>,
}

impl MetadataEngine {
    pub fn new() -> Self {
        Self {
            variable_registry: Arc::new(VariableRegistry::new()),
            scope_tracker: Arc::new(ScopeTracker::new()),
            thread_registry: Arc::new(ThreadRegistry::new()),
        }
    }
}
```

### 快照引擎 (snapshot/)

**用途**: 从事件数据构建内存快照。

**核心实现**:

```rust
pub struct SnapshotEngine {
    event_store: SharedEventStore,
}

impl SnapshotEngine {
    pub fn build_snapshot(&self) -> MemorySnapshot {
        // 从事件构建快照
    }
}

pub struct MemorySnapshot {
    pub allocations: Vec<ActiveAllocation>,
    pub stats: MemoryStats,
    pub timestamp: u64,
}

pub struct ActiveAllocation {
    pub ptr: usize,
    pub size: usize,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
    pub allocated_at: u64,
    pub thread_id: ThreadId,
    pub thread_id_u64: u64,
}
```

### 查询引擎 (query/)

**用途**: 提供统一的查询接口以访问内存数据。

**查询类型**:

```rust
pub enum Query {
    Allocation(AllocationQuery),
    Thread(ThreadQuery),
    Summary(SummaryQuery),
}

pub enum QueryResult {
    Allocation(AllocationQueryResult),
    Thread(ThreadQueryResult),
    Summary(SummaryQueryResult),
}
```

### 分析引擎 (analysis/)

**用途**: 分析内存数据以检测问题和提供洞察。

**检测器**:
- `LeakDetector`: 内存泄漏检测
- `UafDetector`: Use-After-Free 检测
- `OverflowDetector`: 缓冲区溢出检测
- `SafetyDetector`: 安全违规检测
- `LifecycleDetector`: 生命周期分析

**检测器接口**:

```rust
pub trait Detector {
    fn name(&self) -> &str;
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult;
    fn get_config(&self) -> &DetectorConfig;
}

pub struct DetectionResult {
    pub issues: Vec<Issue>,
    pub statistics: DetectionStatistics,
}
```

**附加分析**:
- 循环引用检测
- Unsafe FFI 追踪
- 异步模式分析
- 借用模式分析
- 泛型类型分析
- 闭包分析

### 时间线引擎 (timeline/)

**用途**: 基于时间的内存分析和重放。

**组件**:
- `TimelineIndex`: 基于时间的索引
- `TimelineQuery`: 基于时间的查询
- `TimelineReplay`: 事件重放功能

### 渲染引擎 (render_engine/)

**用途**: 以多种格式渲染输出数据。

**输出格式**:
- JSON
- HTML（交互式仪表盘）
- Binary
- SVG

**导出函数**:

```rust
pub fn export_all_json(output_path: &str, tracker: &Tracker, ...) -> Result<()>
pub fn export_dashboard_html(output_path: &str, tracker: &Tracker, ...) -> Result<()>
pub fn export_snapshot_to_json(snapshot: &MemorySnapshot, path: &Path, options: &ExportJsonOptions) -> Result<()>
pub fn export_leak_detection_json(detection_result: &DetectionResult, path: &Path) -> Result<()>
pub fn export_memory_passports_json(passports: &[MemoryPassport], path: &Path) -> Result<()>
pub fn export_unsafe_ffi_json(stats: &UnsafeFFIStats, path: &Path) -> Result<()>
```

### 外观层 (facade/)

**用途**: 提供集成所有引擎的统一外观接口。

**核心实现**:

```rust
pub struct MemScope {
    pub event_store: Arc<EventStore>,
    pub capture: Arc<CaptureEngine>,
    pub metadata: Arc<MetadataEngine>,
    pub snapshot: Arc<SnapshotEngine>,
    pub query: Arc<QueryEngine>,
    pub analysis: Arc<Mutex<AnalysisEngine>>,
    pub timeline: Arc<TimelineEngine>,
    pub render: Arc<RenderEngine>,
}

impl MemScope {
    pub fn new() -> Self {
        // 创建所有引擎并连接它们
        let event_store = Arc::new(EventStore::new());
        let capture = Arc::new(CaptureEngine::new(CaptureBackendType::Unified, event_store.clone()));
        // ... 创建其他引擎

        Self {
            event_store,
            capture,
            metadata,
            snapshot,
            query,
            analysis,
            timeline,
            render,
        }
    }

    pub fn run_leak_detector(&self) -> DetectionResult {
        self.analysis.lock().unwrap().run_detector(LeakDetector::new(...))
    }

    pub fn export_html(&self, path: &str) -> Result<()> {
        self.render.export_html(path, ...)
    }
}
```

### 错误处理 (error/)

**用途**: 统一错误处理，包含上下文和恢复策略。

**错误类型**:

```rust
pub struct MemScopeError {
    pub kind: ErrorKind,
    pub severity: ErrorSeverity,
    pub context: ErrorContext,
    pub source: Option<Box<dyn std::error::Error>>,
}

pub enum ErrorKind {
    AllocationError,
    TrackingError,
    AnalysisError,
    RenderError,
    ExportError,
}

pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
}

pub enum RecoveryAction {
    Retry,
    Skip,
    Abort,
    Recover,
}
```

### 派生宏 (memscope-derive/)

**用途**: 提供过程宏，自动实现 `Trackable` trait。

**核心实现**:

```rust
#[proc_macro_derive(Trackable)]
pub fn derive_trackable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        Data::Struct(data_struct) => {
            let heap_ptr_impl = generate_heap_ptr_impl(&data_struct.fields);
            let size_estimate_impl = generate_size_estimate_impl(&data_struct.fields);
            let internal_allocations_impl = generate_internal_allocations_impl(&data_struct.fields);

            quote! {
                impl Trackable for #name {
                    fn get_heap_ptr(&self) -> Option<usize> { #heap_ptr_impl }
                    fn get_type_name(&self) -> &'static str { stringify!(#name) }
                    fn get_size_estimate(&self) -> usize { #size_estimate_impl }
                    fn get_internal_allocations(&self, var_name: &str) -> Vec<(usize, String)> {
                        #internal_allocations_impl
                    }
                }
            }
        }
        Data::Enum(data_enum) => { /* ... */ }
        Data::Union(_) => { /* 不支持 */ }
    }
}
```

### Trackable Trait

**用途**: 标记可被内存追踪器追踪的类型。

**Trait 定义**:

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

**内置实现**:
- `Vec<T>`
- `String`
- `HashMap<K, V>`
- `Box<T>`
- `Arc<T>`
- `Rc<T>`

## 数据流

### 追踪流程

```
1. 用户代码使用 track!() 宏
   ↓
2. Tracker 捕获变量信息
   ↓
3. CaptureEngine 记录分配事件
   ↓
4. EventStore 在无锁队列中存储事件
   ↓
5. MetadataEngine 管理元数据
   ↓
6. SnapshotEngine 构建快照
```

### 分析流程

```
1. SnapshotEngine 提供内存快照
   ↓
2. QueryEngine 查询特定数据
   ↓
3. AnalysisEngine 运行检测器
   ↓
4. 生成检测结果
   ↓
5. RenderEngine 渲染输出
   ↓
6. 导出到 JSON/HTML/Binary
```

### 导出流程

```
1. 用户调用导出函数
   ↓
2. RenderEngine 从 SnapshotEngine 检索数据
   ↓
3. 根据输出格式格式化数据
   ↓
4. 写入文件系统
```

## 设计模式

### 外观模式
`MemScope` 提供所有引擎的统一接口，简化用户交互。

### 策略模式
`CaptureBackend` 支持多种追踪策略（Core、Lockfree、Async、Unified）。

### 观察者模式
事件存储和事件记录使用观察者模式处理事件。

### 工厂模式
后端创建和配置使用工厂模式。

### 适配器模式
检测器适配到分析引擎工作。

### 建造者模式
配置对象使用建造者模式进行灵活构建。

### 单例模式
全局追踪器使用单例模式管理共享状态。

## 架构原则

### 1. 分层架构
清晰的关注点分离，每层都有特定的职责。

### 2. 模块化设计
每个模块都有单一、明确定义的职责，使代码库易于理解和维护。

### 3. 类型安全
强类型系统确保内存安全并防止许多常见错误。

### 4. 线程安全
使用 `Arc` 和 `Mutex` 实现共享状态，确保线程安全的并发访问。

### 5. 零开销追踪
使用 TLS 标志而非锁来最小化性能影响。

### 6. 异步支持
专为异步任务设计，支持任务 ID 追踪。

### 7. 可扩展性
易于添加新的检测器、后端和导出格式。

## 性能考虑

### 无锁队列
事件存储使用无锁队列实现高性能并发访问。

### 采样
支持可配置的采样率，以减少生产环境中的开销。

### TLS 标志
使用线程本地存储标志而非锁来防止递归追踪。

### 恐慌恢复
追踪失败被优雅处理，不会导致应用崩溃。

### 有界历史
事件存储支持有界历史记录以限制内存使用。

## 安全考虑

### 恐慌安全
所有操作都是恐慌安全的，确保追踪失败不会导致应用崩溃。

### 类型安全
强类型系统防止内存安全违规。

### 线程安全
所有共享状态都使用 `Arc` 和 `Mutex` 正确同步。

### 递归保护
在追踪操作期间禁用追踪以防止无限循环。

## 依赖

### 外部依赖
- `serde`: 序列化/反序列化
- `serde_json`: JSON 支持
- `std`: 标准库

### 内部依赖
- `memscope-derive`: `Trackable` trait 的过程宏

## 结论

memscope-rs 为 Rust 应用程序的内存追踪和分析提供了全面、模块化的架构。分层设计确保了清晰的关注点分离，而模块化方法使得扩展和维护变得容易。该架构优先考虑性能、安全性和可用性，使其适用于开发和生产环境。