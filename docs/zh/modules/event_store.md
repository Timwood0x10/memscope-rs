# 事件存储模块 (Event Store Module)

## 概述

事件存储模块为 memscope-rs 系统中的所有内存事件提供集中式存储。它是内存跟踪数据的单一事实来源，使用无锁设计实现高并发记录和高效的快照生成以便分析。

## 组件

### 1. MemoryEvent（内存事件）

**文件**: `src/event_store/event.rs`

**用途**: 所有跟踪后端使用的统一内存事件类型。

**核心功能**:
- 所有内存操作的统一事件类型
- 全面的元数据（时间戳、线程信息、调用堆栈）
- 可序列化以进行持久化和分析
- 类型安全的事件分类

**事件类型**:

```rust
pub enum MemoryEventType {
    /// 内存分配事件
    Allocate,
    /// 内存释放事件
    Deallocate,
    /// 内存重新分配事件
    Reallocate,
    /// 内存移动事件
    Move,
    /// 内存借用事件
    Borrow,
    /// 内存返回事件
    Return,
}
```

**事件结构**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    /// 事件时间戳（自纪元以来的纳秒数）
    pub timestamp: u64,
    /// 事件类型
    pub event_type: MemoryEventType,
    /// 内存指针地址
    pub ptr: usize,
    /// 分配大小（字节）
    pub size: usize,
    /// 线程标识符
    pub thread_id: u64,
    /// 可选的变量名
    pub var_name: Option<String>,
    /// 可选的类型名
    pub type_name: Option<String>,
    /// 可选的调用堆栈哈希
    pub call_stack_hash: Option<u64>,
    /// 可选的线程名
    pub thread_name: Option<String>,
}
```

**事件创建方法**:

```rust
impl MemoryEvent {
    /// 创建新的分配事件
    pub fn allocate(ptr: usize, size: usize, thread_id: u64) -> Self {
        Self {
            timestamp: Self::now(),
            event_type: MemoryEventType::Allocate,
            ptr,
            size,
            thread_id,
            var_name: None,
            type_name: None,
            call_stack_hash: None,
            thread_name: None,
        }
    }

    /// 创建新的释放事件
    pub fn deallocate(ptr: usize, size: usize, thread_id: u64) -> Self {
        Self {
            timestamp: Self::now(),
            event_type: MemoryEventType::Deallocate,
            ptr,
            size,
            thread_id,
            var_name: None,
            type_name: None,
            call_stack_hash: None,
            thread_name: None,
        }
    }

    /// 创建新的重新分配事件
    pub fn reallocate(ptr: usize, _old_size: usize, new_size: usize, thread_id: u64) -> Self {
        Self {
            timestamp: Self::now(),
            event_type: MemoryEventType::Reallocate,
            ptr,
            size: new_size,
            thread_id,
            var_name: None,
            type_name: None,
            call_stack_hash: None,
            thread_name: None,
        }
    }

    /// 设置变量名
    pub fn with_var_name(mut self, name: String) -> Self {
        self.var_name = Some(name);
        self
    }

    /// 设置类型名
    pub fn with_type_name(mut self, name: String) -> Self {
        self.type_name = Some(name);
        self
    }

    /// 设置调用堆栈哈希
    pub fn with_call_stack_hash(mut self, hash: u64) -> Self {
        self.call_stack_hash = Some(hash);
        self
    }

    /// 设置线程名
    pub fn with_thread_name(mut self, name: String) -> Self {
        self.thread_name = Some(name);
        self
    }

    /// 检查这是否是分配事件
    pub fn is_allocation(&self) -> bool {
        matches!(
            self.event_type,
            MemoryEventType::Allocate | MemoryEventType::Reallocate
        )
    }

    /// 检查这是否是释放事件
    pub fn is_deallocation(&self) -> bool {
        matches!(self.event_type, MemoryEventType::Deallocate)
    }

    /// 检查这是否是移动事件
    pub fn is_move(&self) -> bool {
        matches!(self.event_type, MemoryEventType::Move)
    }

    /// 检查这是否是借用事件
    pub fn is_borrow(&self) -> bool {
        matches!(self.event_type, MemoryEventType::Borrow)
    }

    /// 检查这是否是返回事件
    pub fn is_return(&self) -> bool {
        matches!(self.event_type, MemoryEventType::Return)
    }
}
```

### 2. EventStore（事件存储）

**文件**: `src/event_store/store.rs`

**用途**: 内存事件的集中式存储，具有无锁记录和高效快照功能。

**核心功能**:
- 无锁记录：使用 SegQueue 实现 O(1) 追加，无阻塞
- 线程安全：所有操作对并发使用安全
- 高效快照：使用 RwLock 实现快速读取访问
- 原子计数：近似事件计数，无需锁定

**核心实现**:

```rust
#[derive(Debug)]
pub struct EventStore {
    /// 用于高并发记录的无锁队列
    queue: SegQueue<MemoryEvent>,
    /// 用于快速快照访问的缓存事件
    cache: RwLock<Vec<MemoryEvent>>,
    /// 近似事件计数（可能略有延迟）
    count: AtomicUsize,
}

impl EventStore {
    /// 创建新的 EventStore
    pub fn new() -> Self {
        Self {
            queue: SegQueue::new(),
            cache: RwLock::new(Vec::new()),
            count: AtomicUsize::new(0),
        }
    }

    /// 记录内存事件
    ///
    /// 此方法是无锁的，可以从任何线程调用，
    /// 而不会阻塞其他记录操作。
    pub fn record(&self, event: MemoryEvent) {
        self.queue.push(event);
        // 使用 Release 顺序确保在计数递增之前推送可见
        self.count.fetch_add(1, Ordering::Release);
    }

    /// 将待处理事件从队列刷新到缓存
    fn flush_to_cache(&self) {
        let mut cache = self.cache.write();
        while let Some(event) = self.queue.pop() {
            cache.push(event);
        }
    }

    /// 获取所有事件作为快照
    ///
    /// 返回存储中当前所有事件的快照。
    /// 此方法在返回之前将无锁队列中的任何
    /// 待处理事件刷新到缓存。
    pub fn snapshot(&self) -> Vec<MemoryEvent> {
        self.flush_to_cache();
        self.cache.read().clone()
    }

    /// 获取存储中的事件数量
    pub fn len(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    /// 检查存储是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 清除存储中的所有事件
    pub fn clear(&self) {
        // 首先获取写锁以防止并发修改
        let mut cache = self.cache.write();

        // 清除队列
        while self.queue.pop().is_some() {}

        // 清除缓存
        cache.clear();

        // 最后重置计数，同时仍持有写锁
        self.count.store(0, Ordering::Release);
    }
}

/// EventStore 的共享引用
pub type SharedEventStore = Arc<EventStore>;
```

**设计理念**:

1. **无锁记录**: 使用 SegQueue 实现 O(1) 追加操作
2. **高效快照**: 刷新到缓存以实现快速读取访问
3. **原子计数**: 近似计数，无锁定开销
4. **线程安全**: 所有操作对并发使用安全

## 设计原则

### 1. 无锁记录
EventStore 使用无锁队列进行记录：
- **优势**: 无阻塞，高并发支持
- **权衡**: 队列中的事件可能不会立即在快照中可见

### 2. 双重存储
同时使用队列和缓存：
- **队列**: 无锁，针对写入优化
- **缓存**: RwLock 保护，针对读取优化
- **优势**: 针对读写操作都优化

### 3. 原子计数
使用原子操作进行计数：
- **优势**: 快速，无锁定开销
- **权衡**: 计数可能略有延迟

### 4. 快照隔离
快照与正在进行的记录隔离：
- **优势**: 事件的一致视图
- **权衡**: 对于大型数据集，快照创建可能需要时间

## 使用示例

### 基本使用

```rust
use memscope::event_store::{EventStore, MemoryEvent};
use std::sync::Arc;

// 创建事件存储
let event_store = Arc::new(EventStore::new());

// 记录事件
event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));
event_store.record(MemoryEvent::deallocate(0x1000, 1024, 1));

// 获取快照
let events = event_store.snapshot();
println!("总事件数: {}", events.len());

// 清除存储
event_store.clear();
```

### 并发记录

```rust
use std::thread;
use std::sync::Arc;

let event_store = Arc::new(EventStore::new());
let mut handles = vec![];

// 多个线程记录事件
for i in 0..10 {
    let store = Arc::clone(&event_store);
    let handle = thread::spawn(move || {
        for j in 0..100 {
            let event = MemoryEvent::allocate(i * 1000 + j, 1024, i as u64);
            store.record(event);
        }
    });
    handles.push(handle);
}

// 等待所有线程
for handle in handles {
    handle.join().unwrap();
}

// 获取所有事件
let events = event_store.snapshot();
println!("总事件数: {}", events.len());
```

### 带元数据的事件

```rust
let event = MemoryEvent::allocate(0x1000, 1024, 1)
    .with_var_name("my_variable".to_string())
    .with_type_name("Vec<u8>".to_string())
    .with_thread_name("main".to_string())
    .with_call_stack_hash(0x12345678);

event_store.record(event);
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
时间线引擎（基于时间的分析）
```

## 性能考虑

### 无锁记录
记录是无锁的且高度并发：
- **优势**: 高吞吐量，无阻塞
- **权衡**: 事件可能不会立即可见

### 快照创建
快照创建将队列刷新到缓存：
- **优势**: 所有事件的一致视图
- **权衡**: 对于大型数据集可能较慢

### 内存使用
存储所有事件直到清除：
- **优势**: 完整的事件历史
- **权衡**: 内存随事件计数增长

### 原子计数
计数是近似的，可能略有延迟：
- **优势**: 快速，无锁定
- **权衡**: 可能不反映确切计数

## 测试

事件存储模块包含全面的测试：

```rust
#[test]
fn test_event_store_creation() {
    let store = EventStore::new();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
}

#[test]
fn test_record_event() {
    let store = EventStore::new();
    let event = MemoryEvent::allocate(0x1000, 1024, 1);
    store.record(event);
    assert_eq!(store.len(), 1);
}

#[test]
fn test_snapshot() {
    let store = EventStore::new();
    let event1 = MemoryEvent::allocate(0x1000, 1024, 1);
    let event2 = MemoryEvent::deallocate(0x1000, 1024, 1);
    store.record(event1.clone());
    store.record(event2.clone());

    let snapshot = store.snapshot();
    assert_eq!(snapshot.len(), 2);
    // 验证快照后事件仍在存储中
    assert_eq!(store.len(), 2);
}

#[test]
fn test_clear() {
    let store = EventStore::new();
    let event = MemoryEvent::allocate(0x1000, 1024, 1);
    store.record(event);
    assert_eq!(store.len(), 1);

    store.clear();
    assert!(store.is_empty());
}

#[test]
fn test_concurrent_access() {
    use std::thread;
    let store = Arc::new(EventStore::new());
    let mut handles = vec![];

    for i in 0..10 {
        let store_clone = Arc::clone(&store);
        let handle = thread::spawn(move || {
            for j in 0..100 {
                let event = MemoryEvent::allocate(i * 1000 + j, 1024, i as u64);
                store_clone.record(event);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(store.len(), 1000);
    let snapshot = store.snapshot();
    assert_eq!(snapshot.len(), 1000);
}
```

## 最佳实践

1. **事件记录**: 对高并发使用无锁记录
2. **快照管理**: 定期清除存储以释放内存
3. **错误处理**: 始终处理潜在的序列化错误
4. **线程安全**: 使用 Arc<EventStore> 进行共享访问

## 限制

1. **内存增长**: 存储增长直到清除
2. **近似计数**: 计数可能略有延迟
3. **快照延迟**: 队列中的事件可能不会立即在快照中
4. **竞争条件**: 清除和记录可能存在竞争条件

## 未来改进

1. **事件采样**: 添加事件采样以减少内存使用
2. **基于时间的过滤**: 按时间范围过滤事件
3. **压缩**: 压缩存储的事件
4. **持久化**: 添加磁盘持久化以进行长期存储
5. **事件过滤**: 按类型或元数据过滤事件