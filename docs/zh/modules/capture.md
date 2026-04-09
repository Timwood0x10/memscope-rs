# 捕获模块 (Capture Module)

## 概述

捕获模块为 memscope-rs 库提供事件捕获后端。它负责拦截应用程序中的内存事件，并将这些事件转发到事件存储（EventStore）。捕获模块实现了后端抽象模式，允许在不更改核心追踪逻辑的情况下插入不同的捕获策略。

## 组件

### 1. CaptureEngine（捕获引擎）

**文件**: `src/capture/engine.rs`

**用途**: 主事件捕获引擎，协调捕获后端并将事件转发到事件存储。

**核心功能**:
- 后端抽象：支持多种捕获后端
- 非阻塞操作：所有捕获操作都是非阻塞的
- 零存储：事件转发到事件存储，不在本地存储
- 线程安全：多线程并发使用安全

**核心实现**:

```rust
pub struct CaptureEngine {
    /// 正在使用的捕获后端
    backend: Box<dyn CaptureBackend>,
    /// 用于记录事件的事件存储引用
    event_store: SharedEventStore,
}

impl CaptureEngine {
    /// 使用指定后端创建新的 CaptureEngine
    pub fn new(backend_type: CaptureBackendType, event_store: SharedEventStore) -> Self {
        let backend = backend_type.create_backend();
        Self {
            backend,
            event_store,
        }
    }

    /// 捕获分配事件
    pub fn capture_alloc(&self, ptr: usize, size: usize, thread_id: u64) {
        let event = self.backend.capture_alloc(ptr, size, thread_id);
        self.event_store.record(event);
    }

    /// 捕获释放事件
    pub fn capture_dealloc(&self, ptr: usize, size: usize, thread_id: u64) {
        let event = self.backend.capture_dealloc(ptr, size, thread_id);
        self.event_store.record(event);
    }

    /// 捕获重新分配事件
    pub fn capture_realloc(&self, ptr: usize, old_size: usize, new_size: usize, thread_id: u64) {
        let event = self.backend.capture_realloc(ptr, old_size, new_size, thread_id);
        self.event_store.record(event);
    }

    /// 捕获移动事件
    pub fn capture_move(&self, from_ptr: usize, to_ptr: usize, size: usize, thread_id: u64) {
        let event = self.backend.capture_move(from_ptr, to_ptr, size, thread_id);
        self.event_store.record(event);
    }
}
```

**设计理念**:

1. **后端抽象**: 将捕获逻辑与事件存储分离
2. **非阻塞**: 所有操作设计为快速且非阻塞
3. **零存储**: 不在本地存储事件，立即转发
4. **可插拔**: 易于添加新的捕获后端

### 2. 捕获后端

**文件**: `src/capture/backends/`

**用途**: 不同使用场景的不同捕获策略。

**后端类型**:

```rust
pub enum CaptureBackendType {
    /// 使用追踪分配器的核心后端
    Core,
    /// 高性能场景的无锁后端
    Lockfree,
    /// 异步应用的异步后端
    Async,
    /// 结合所有策略的统一后端
    Unified,
}
```

**后端接口**:

```rust
pub trait CaptureBackend: Send + Sync {
    /// 捕获分配事件
    fn capture_alloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent;

    /// 捕获释放事件
    fn capture_dealloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent;

    /// 捕获重新分配事件
    fn capture_realloc(&self, ptr: usize, old_size: usize, new_size: usize, thread_id: u64) -> MemoryEvent;

    /// 捕获移动事件
    fn capture_move(&self, from_ptr: usize, to_ptr: usize, size: usize, thread_id: u64) -> MemoryEvent;
}
```

**后端实现**:

1. **CoreBackend**: 使用追踪分配器进行基本捕获
2. **LockfreeBackend**: 高并发场景的无锁实现
3. **AsyncBackend**: 专用于异步应用
4. **UnifiedBackend**: 结合所有策略

### 3. 系统监控器

**文件**: `src/capture/system_monitor.rs`

**用途**: 监控系统资源并为内存事件提供上下文。

**核心功能**:
- CPU 使用率监控
- 内存使用跟踪
- 线程活动监控
- 资源利用率分析

**监控数据**:

```rust
pub struct SystemStats {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub thread_count: usize,
    pub active_allocations: usize,
    pub peak_memory: usize,
}
```

### 4. 平台抽象

**文件**: `src/capture/platform/`

**用途**: 不同操作系统的平台特定实现。

**支持的平台**:
- Linux
- macOS
- Windows

**平台检测**:

```rust
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;
```

### 5. 类型系统

**文件**: `src/capture/types/`

**用途**: 捕获模块的类型定义和数据结构。

**核心类型**:

```rust
pub struct AllocationInfo {
    pub ptr: usize,
    pub size: usize,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
    pub scope_name: Option<String>,
    pub timestamp_alloc: u64,
    pub timestamp_dealloc: Option<u64>,
    pub thread_id: ThreadId,
    pub thread_id_u64: u64,
    pub borrow_count: u32,
    pub stack_trace: Option<Vec<String>>,
    pub is_leaked: bool,
    pub lifetime_ms: Option<u64>,
    // ... 更多字段
}

pub enum MemoryOperation {
    Allocation,
    Deallocation,
    Reallocation,
    Move,
    Borrow,
    Return,
}

pub type TrackingResult<T> = Result<T, TrackingError>;
```

## 设计原则

### 1. 后端抽象
捕获模块使用后端抽象模式：
- **优势**: 易于添加新的捕获策略
- **权衡**: 特征分发带来的轻微开销

### 2. 非阻塞操作
所有捕获操作都设计为非阻塞：
- **优势**: 不影响应用程序线程的性能
- **权衡**: 事件处理可能延迟

### 3. 零存储
捕获引擎不在本地存储事件：
- **优势**: 低内存占用
- **权衡**: 依赖于事件存储的容量

### 4. 线程安全
所有操作都是线程安全的：
- **优势**: 并发使用安全
- **权衡**: 同步开销

## 使用示例

### 基本使用

```rust
use memscope::capture::{CaptureEngine, CaptureBackendType};
use memscope::event_store::EventStore;
use std::sync::Arc;

// 创建事件存储和捕获引擎
let event_store = Arc::new(EventStore::new());
let capture = Arc::new(CaptureEngine::new(
    CaptureBackendType::Core,
    event_store.clone(),
));

// 捕获分配事件
capture.capture_alloc(0x1000, 1024, 1);

// 捕获释放事件
capture.capture_dealloc(0x1000, 1024, 1);
```

### 使用不同后端

```rust
// 高性能场景的无锁后端
let capture = CaptureEngine::new(
    CaptureBackendType::Lockfree,
    event_store.clone(),
);

// 异步应用的异步后端
let capture = CaptureEngine::new(
    CaptureBackendType::Async,
    event_store.clone(),
);

// 所有场景的统一后端
let capture = CaptureEngine::new(
    CaptureBackendType::Unified,
    event_store.clone(),
);
```

### 自定义后端

```rust
struct MyCustomBackend;

impl CaptureBackend for MyCustomBackend {
    fn capture_alloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent {
        MemoryEvent::allocate(ptr, size, thread_id)
    }

    fn capture_dealloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent {
        MemoryEvent::deallocate(ptr, size, thread_id)
    }

    // ... 实现其他方法
}

// 使用自定义后端
let capture = CaptureEngine::new(
    CaptureBackendType::Custom(Box::new(MyCustomBackend)),
    event_store.clone(),
);
```

## 与其他模块的集成

```
捕获引擎
    ↓
事件存储（记录事件）
    ↓
快照引擎（从事件构建快照）
    ↓
查询引擎（查询快照数据）
    ↓
分析引擎（分析内存模式）
    ↓
渲染引擎（可视化结果）
```

## 性能考虑

### 后端选择
根据使用场景选择合适的后端：
- **Core**: 适用于一般用例
- **Lockfree**: 最适合高并发场景
- **Async**: 最适合异步应用
- **Unified**: 结合所有策略

### 事件转发
事件立即转发到事件存储：
- **优势**: 低延迟
- **权衡**: 事件存储上可能存在竞争

### 内存开销
捕获引擎的内存开销最小：
- **优势**: 低内存占用
- **权衡**: 依赖于事件存储的容量

## 测试

捕获模块包含全面的测试：

```rust
#[test]
fn test_capture_engine_creation() {
    let event_store = Arc::new(EventStore::new());
    let engine = CaptureEngine::new(CaptureBackendType::Core, event_store);
    assert!(engine.event_store().is_empty());
}

#[test]
fn test_capture_alloc() {
    let event_store = Arc::new(EventStore::new());
    let engine = CaptureEngine::new(CaptureBackendType::Core, event_store.clone());
    engine.capture_alloc(0x1000, 1024, 1);
    assert_eq!(event_store.len(), 1);
}

#[test]
fn test_capture_multiple_events() {
    let event_store = Arc::new(EventStore::new());
    let engine = CaptureEngine::new(CaptureBackendType::Core, event_store.clone());
    engine.capture_alloc(0x1000, 1024, 1);
    engine.capture_dealloc(0x1000, 1024, 1);
    engine.capture_alloc(0x2000, 2048, 1);
    assert_eq!(event_store.len(), 3);
}
```

## 最佳实践

1. **后端选择**: 根据使用场景选择合适的后端
2. **错误处理**: 捕获事件时始终检查错误
3. **性能**: 在高并发场景中使用无锁后端
4. **测试**: 在生产使用前彻底测试捕获逻辑

## 限制

1. **栈分配**: 仅追踪堆分配
2. **外部内存**: 外部库分配的内存可能无法追踪
3. **性能影响**: 事件捕获带来一些开销
4. **平台支持**: 某些功能可能是平台特定的

## 未来改进

1. **更好的堆栈跟踪**: 捕获更详细的堆栈跟踪
2. **变量名**: 捕获实际的变量名
3. **类型信息**: 提供更准确的类型信息
4. **性能**: 进一步优化捕获开销
5. **自定义后端**: 更容易的自定义后端注册