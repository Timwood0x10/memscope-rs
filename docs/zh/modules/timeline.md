# 时间线引擎模块 (Timeline Engine Module)

## 概述

时间线引擎提供基于时间的内存分析和事件回放功能。它按时间戳索引事件，允许查询时间范围内的事件。

## 组件

### 1. TimelineEngine

**文件**: `src/timeline/engine.rs`

**用途**: 内存事件的时间索引和查询。

```rust
pub struct TimelineEngine {
    event_store: SharedEventStore,
    cached_events: RwLock<Vec<MemoryEvent>>,
    cache_version: AtomicU64,
}

impl TimelineEngine {
    pub fn new(event_store: SharedEventStore) -> Self

    pub fn get_events_in_range(&self, start: u64, end: u64) -> Vec<MemoryEvent>

    pub fn get_events_for_pointer(&self, ptr: usize) -> Vec<MemoryEvent>

    pub fn get_events_for_thread(&self, thread_id: u64) -> Vec<MemoryEvent>
}
```

### 2. TimelineIndex

**文件**: `src/timeline/index.rs`

**用途**: 提供事件的时间索引，支持快速范围查询。

### 3. TimelineQuery

**文件**: `src/timeline/query.rs`

**用途**: 结构化的时间查询。

### 4. TimelineReplay

**文件**: `src/timeline/replay.rs`

**用途**: 按时间顺序回放事件以进行分析。

## 关键特性

1. **时间排序索引**: 事件按时间戳排序
2. **范围查询**: 快速检索时间范围内的事件
3. **缓存**: 带版本跟踪的内存缓存
4. **回放**: 按时间顺序回放事件

## 使用示例

```rust
use memscope_rs::timeline::TimelineEngine;

let timeline = TimelineEngine::new(event_store.clone());

// 获取时间范围内的事件
let events = timeline.get_events_in_range(start_time, end_time);

// 获取特定指针的所有事件
let ptr_events = timeline.get_events_for_pointer(0x1000);

// 获取特定线程的所有事件
let thread_events = timeline.get_events_for_thread(thread_id);
```

## 设计决策

1. **惰性缓存**: 在首次查询时构建缓存
2. **版本跟踪**: 检测缓存何时过期
3. **二分查找**: 使用 partition_point 进行快速范围查询

## 限制

1. **单线程索引**: 索引构建不是并发的
2. **仅内存存储**: 无磁盘持久化
3. **需要完整快照**: 需要整个事件历史
