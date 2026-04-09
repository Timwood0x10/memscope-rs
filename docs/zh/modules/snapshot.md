# 快照模块 (Snapshot Module)

## 概述

快照模块提供内存快照构建和聚合功能。它从事件存储中存储的事件构建内存使用的时间点视图，实现当前内存状态和模式的分析。

## 组件

### 1. SnapshotEngine（快照引擎）

**文件**: `src/snapshot/engine.rs`

**用途**: 从事件数据构建内存快照。

**核心功能**:
- 只读：不从事件存储消耗事件
- 高效：针对快速快照构建进行优化
- 全面：捕获所有相关的内存状态
- 线程安全：并发访问安全

**核心实现**:

```rust
pub struct SnapshotEngine {
    /// 事件存储的引用
    event_store: SharedEventStore,
}

impl SnapshotEngine {
    /// 从当前事件存储状态构建快照
    pub fn build_snapshot(&self) -> MemorySnapshot {
        let events = self.event_store.snapshot();
        self.build_snapshot_from_events(events)
    }

    /// 从特定事件集构建快照
    pub fn build_snapshot_from_events(&self, events: Vec<MemoryEvent>) -> MemorySnapshot {
        let mut snapshot = MemorySnapshot::new();
        let mut ptr_to_allocation: HashMap<usize, ActiveAllocation> = HashMap::new();
        let mut thread_stats: HashMap<u64, ThreadMemoryStats> = HashMap::new();
        let mut peak_memory: usize = 0;
        let mut current_memory: usize = 0;

        for event in events {
            match event.event_type {
                MemoryEventType::Allocate | MemoryEventType::Reallocate => {
                    // 记录分配
                    let allocation = ActiveAllocation {
                        ptr: event.ptr,
                        size: event.size,
                        allocated_at: event.timestamp,
                        var_name: event.var_name,
                        type_name: event.type_name,
                        thread_id: event.thread_id,
                    };

                    ptr_to_allocation.insert(event.ptr, allocation);

                    // 更新统计
                    snapshot.stats.total_allocations += 1;
                    snapshot.stats.total_allocated += event.size;
                    current_memory += event.size;

                    // 更新线程统计
                    let thread_stat = thread_stats.entry(event.thread_id).or_insert_with(
                        || ThreadMemoryStats {
                            thread_id: event.thread_id,
                            allocation_count: 0,
                            total_allocated: 0,
                            current_memory: 0,
                            peak_memory: 0,
                        }
                    );
                    thread_stat.allocation_count += 1;
                    thread_stat.total_allocated += event.size;
                    thread_stat.current_memory += event.size;
                    if thread_stat.current_memory > thread_stat.peak_memory {
                        thread_stat.peak_memory = thread_stat.current_memory;
                    }
                }
                MemoryEventType::Deallocate => {
                    // 移除分配
                    if let Some(allocation) = ptr_to_allocation.remove(&event.ptr) {
                        snapshot.stats.total_deallocations += 1;
                        snapshot.stats.total_deallocated += allocation.size;
                        current_memory -= allocation.size;

                        // 更新线程统计
                        if let Some(thread_stat) = thread_stats.get_mut(&event.thread_id) {
                            thread_stat.current_memory -= allocation.size;
                        }
                    } else {
                        snapshot.stats.unmatched_deallocations += 1;
                    }
                }
                _ => {}
            }

            // 更新峰值内存
            if current_memory > peak_memory {
                peak_memory = current_memory;
            }
        }

        // 构建最终快照
        snapshot.active_allocations = ptr_to_allocation;
        snapshot.thread_stats = thread_stats;
        snapshot.stats.active_allocations = snapshot.active_allocations.len();
        snapshot.stats.current_memory = current_memory;
        snapshot.stats.peak_memory = peak_memory;

        snapshot
    }
}
```

### 2. MemorySnapshot（内存快照）

**文件**: `src/snapshot/types.rs`

**用途**: 表示内存状态的时间点视图。

**快照结构**:

```rust
pub struct MemorySnapshot {
    /// 活动分配
    pub active_allocations: HashMap<usize, ActiveAllocation>,
    /// 线程特定的内存统计
    pub thread_stats: HashMap<u64, ThreadMemoryStats>,
    /// 整体统计
    pub stats: SnapshotStats,
}

pub struct SnapshotStats {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub active_allocations: usize,
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub current_memory: usize,
    pub peak_memory: usize,
    pub unmatched_deallocations: usize,
}
```

## 使用示例

### 基本使用

```rust
use memscope::snapshot::{SnapshotEngine, MemorySnapshot};
use memscope::event_store::EventStore;
use std::sync::Arc;

// 创建快照引擎
let event_store = Arc::new(EventStore::new());
let engine = SnapshotEngine::new(event_store);

// 构建快照
let snapshot = engine.build_snapshot();

// 访问快照数据
println!("活动分配: {}", snapshot.active_count());
println!("当前内存: {} 字节", snapshot.current_memory());
println!("峰值内存: {} 字节", snapshot.peak_memory());
```

### 带事件的快照

```rust
// 添加一些事件
event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));
event_store.record(MemoryEvent::allocate(0x2000, 2048, 1));
event_store.record(MemoryEvent::deallocate(0x1000, 1024, 1));

// 构建快照
let snapshot = engine.build_snapshot();

// 检查结果
assert_eq!(snapshot.active_count(), 1);
assert_eq!(snapshot.current_memory(), 2048);
```

## 设计原则

### 1. 只读操作
快照不修改事件数据：
- **优势**: 安全、可重现的分析
- **权衡**: 必须为快照复制数据

### 2. 高效构建
针对快速快照构建进行优化：
- **优势**: 快速分析
- **权衡**: 构建期间可能使用更多内存

### 3. 线程安全
并发访问安全：
- **优势**: 多线程分析
- **权衡**: 同步开销

## 最佳实践

1. **快照频率**: 在准确性和性能之间取得平衡
2. **内存管理**: 清除旧快照以释放内存
3. **线程安全**: 使用 Arc<SnapshotEngine> 进行共享访问
4. **错误处理**: 始终处理快照构建错误

## 限制

1. **时间点**: 仅显示构建时的状态
2. **内存使用**: 快照消耗内存
3. **构建时间**: 大型数据集可能需要时间构建
4. **事件顺序**: 依赖于正确的事件顺序

## 未来改进

1. **增量快照**: 增量更新现有快照
2. **差异快照**: 比较快照以查找更改
3. **压缩**: 压缩快照数据
4. **持久化**: 将快照保存到磁盘
5. **时间旅行**: 从事件重构过去的状态