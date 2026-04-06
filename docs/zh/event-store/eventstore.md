# EventStore

> 中央事件总线 — 无锁事件存储与快照机制

---

## 概述

**文件:** `src/event_store/store.rs`

EventStore 是**中央事件总线**，所有捕获后端都将 `MemoryEvent` 记录推送到这里。它使用 crossbeam 的无锁 `SegQueue` 实现高吞吐事件摄入，并提供快照机制用于时间点分析。

---

## 核心数据结构

### EventStore

```rust
// store.rs:23-28
pub struct EventStore {
    events: SegQueue<MemoryEvent>,   // 无锁 MPMC 队列
    snapshot_lock: Mutex<()>,        // 保护快照期间的清空-恢复操作
}
```

- **`SegQueue<MemoryEvent>`** — 来自 `crossbeam` crate 的无锁多生产者多消费者队列。多个线程可以并发推送事件，无需任何锁。
- **`snapshot_lock`** — 序列化并发快照操作的互斥锁。同一时间只能有一个快照运行。

### MemoryEvent

```rust
// event_store/event.rs
pub struct MemoryEvent {
    pub timestamp: u64,
    pub event_type: MemoryEventType,
    pub ptr: usize,
    pub size: usize,
    pub thread_id: u64,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
    pub call_stack_hash: Option<u64>,
    pub thread_name: Option<String>,
}

pub enum MemoryEventType {
    Allocate,      // 分配
    Deallocate,    // 释放
    Reallocate { old_size: usize },  // 重新分配
    Move,          // 内存移动
}
```

每个事件捕获一次内存操作，包含时间戳、类型、指针、大小和可选元数据（变量名、类型名、调用栈哈希、线程名）。

---

## 事件记录

```rust
// store.rs:46-48
pub fn record(&self, event: MemoryEvent) {
    self.events.push(event);  // 无锁, O(1)
}
```

**工作原理:** `SegQueue::push()` 是无锁操作。多个线程可以同时调用 `record()` 而不会互相阻塞。事件被追加到队列的内部段列表中。

**性能:** O(1) 均摊，无锁，热路径上无原子操作（除了队列内部的 CAS）。

---

## 快照机制

```rust
// store.rs:62-74
pub fn snapshot(&self) -> Vec<MemoryEvent> {
    let _guard = self.snapshot_lock.lock().unwrap();
    let mut events = Vec::new();

    // 从队列中清空所有事件
    while let Some(event) = self.events.pop() {
        events.push(event);
    }

    // 恢复它们，以便未来快照仍能看到
    for event in &events {
        self.events.push(event.clone());
    }

    events
}
```

**工作原理:**

1. 获取 `snapshot_lock` — 确保同一时间只有一个快照
2. 从 `SegQueue` 中清空所有事件到 `Vec`
3. 将所有事件推回队列，以便未来快照可用
4. 返回 `Vec` 给调用者

**为什么需要清空-恢复？** `SegQueue` 不支持不消费的迭代。要获取时间点视图，必须清空队列并恢复事件。

**已知问题:** 在清空-恢复窗口期间，并发的 `record()` 调用可能推送事件，这些事件会与恢复的事件以不可预测的顺序交错。`snapshot_lock` 只保护并发快照，不保护并发 `record()` 调用。

---

## 长度查询

```rust
// store.rs:80-82
pub fn len(&self) -> usize {
    self.events.len()
}
```

返回队列中当前事件数。这是一个并发读取 — 调用者使用时可能已经过时。

---

## 事件创建辅助方法

```rust
// event_store/event.rs
impl MemoryEvent {
    pub fn allocate(ptr: usize, size: usize, thread_id: u64) -> Self { ... }
    pub fn deallocate(ptr: usize, size: usize, thread_id: u64) -> Self { ... }
    pub fn reallocate(ptr: usize, old_size: usize, new_size: usize, thread_id: u64) -> Self { ... }
    pub fn move_event(from_ptr: usize, to_ptr: usize, size: usize, thread_id: u64) -> Self { ... }

    pub fn with_var_name(mut self, name: String) -> Self { ... }
    pub fn with_type_name(mut self, name: String) -> Self { ... }
    pub fn with_call_stack_hash(mut self, hash: u64) -> Self { ... }
    pub fn with_thread_name(mut self, name: String) -> Self { ... }

    pub fn is_allocation(&self) -> bool { ... }
    pub fn is_deallocation(&self) -> bool { ... }
}
```

用于构建带可选元数据的事件的构建器模式辅助方法。

---

## 时间戳生成

```rust
/// Get current timestamp in nanoseconds
/// Returns 0 if system time is before Unix epoch (should not happen in practice)
pub fn now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or_default() 
}
```

---

## 性能特征

| 操作 | 复杂度 | 锁 | 说明 |
|------|--------|-----|------|
| `record(event)` | O(1) | 无 (无锁) | Crossbeam SegQueue 推送 |
| `snapshot()` | O(n) | `snapshot_lock` | 清空并恢复所有事件 |
| `len()` | O(1) | 无 | 可能过时 |

**单事件内存成本:** ~80-100 字节 (`MemoryEvent`) + ~32 字节 (SegQueue 节点开销)。

---

## 使用模式

```rust
// 1. 捕获后端推送事件
let event = MemoryEvent::allocate(ptr, size, thread_id);
event_store.record(event);

// 2. 分析引擎获取快照
let events = event_store.snapshot();

// 3. 处理事件
for event in &events {
    if event.is_allocation() {
        // 处理分配
    }
}
```

---

## 与其他模块的集成

| 模块 | 如何使用 EventStore |
|------|---------------------|
| **CaptureEngine** | 对每次分配/释放调用 `record()` |
| **SnapshotEngine** | 调用 `snapshot()` 构建时间点视图 |
| **TimelineEngine** | 调用 `snapshot()` 获取时间序列分析的事件 |
| **QueryEngine** | 使用快照回答查询 |
