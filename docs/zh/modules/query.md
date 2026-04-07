# 查询模块 (Query Module)

## 概述

查询模块为内存快照数据提供统一的查询接口。它支持各种查询类型和过滤选项，实现灵活的内存使用模式分析。

## 组件

### 1. QueryEngine（查询引擎）

**文件**: `src/query/engine.rs`

**用途**: 内存数据的统一查询接口。

**核心功能**:
- 统一：所有查询类型的单一接口
- 高效：针对快速查询执行进行优化
- 灵活：支持过滤和排序
- 线程安全：并发访问安全

**核心实现**:

```rust
pub struct QueryEngine {
    /// 快照引擎的引用
    snapshot_engine: SharedSnapshotEngine,
}

impl QueryEngine {
    /// 创建新的 QueryEngine
    pub fn new(snapshot_engine: SharedSnapshotEngine) -> Self {
        Self { snapshot_engine }
    }

    /// 获取当前快照
    fn get_snapshot(&self) -> MemorySnapshot {
        self.snapshot_engine.build_snapshot()
    }

    /// 按大小查询顶级分配
    pub fn top_allocations(&self, limit: usize) -> QueryResult {
        let snapshot = self.get_snapshot();
        let mut allocations: Vec<_> = snapshot.active_allocations.values().cloned().collect();

        // 按大小降序排序
        allocations.sort_by(|a, b| b.size.cmp(&a.size));

        // 限制结果
        allocations.truncate(limit);

        let total_bytes = allocations.iter().map(|a| a.size).sum();

        QueryResult::Allocations(AllocationQueryResult {
            count: allocations.len(),
            total_bytes,
            allocations,
        })
    }

    /// 查询特定线程的分配
    pub fn allocations_by_thread(&self, thread_id: u64) -> QueryResult {
        let snapshot = self.get_snapshot();
        let allocations: Vec<_> = snapshot.active_allocations.values()
            .filter(|a| a.thread_id == thread_id)
            .cloned()
            .collect();

        let total_bytes = allocations.iter().map(|a| a.size).sum();

        QueryResult::Allocations(AllocationQueryResult {
            count: allocations.len(),
            total_bytes,
            allocations,
        })
    }

    /// 查询线程统计
    pub fn thread_stats(&self) -> QueryResult {
        let snapshot = self.get_snapshot();
        let threads: Vec<_> = snapshot.thread_stats.values().cloned().collect();
        let total_bytes = threads.iter().map(|t| t.total_allocated).sum();

        QueryResult::Threads(ThreadQueryResult {
            count: threads.len(),
            total_bytes,
            threads,
        })
    }

    /// 查询内存使用摘要
    pub fn summary(&self) -> QueryResult {
        let snapshot = self.get_snapshot();

        QueryResult::Summary(SummaryQueryResult {
            total_allocations: snapshot.stats.total_allocations,
            total_deallocations: snapshot.stats.total_deallocations,
            active_allocations: snapshot.stats.active_allocations,
            total_allocated: snapshot.stats.total_allocated,
            total_deallocated: snapshot.stats.total_deallocated,
            current_memory: snapshot.stats.current_memory,
            peak_memory: snapshot.stats.peak_memory,
            thread_count: snapshot.thread_stats.len(),
        })
    }

    /// 查询具有特定变量名的分配
    pub fn allocations_by_variable(&self, var_name: &str) -> QueryResult {
        let snapshot = self.get_snapshot();
        let allocations: Vec<_> = snapshot.active_allocations.values()
            .filter(|a| a.var_name.as_ref().map(|n| n == var_name).unwrap_or(false))
            .cloned()
            .collect();

        let total_bytes = allocations.iter().map(|a| a.size).sum();

        QueryResult::Allocations(AllocationQueryResult {
            count: allocations.len(),
            total_bytes,
            allocations,
        })
    }

    /// 查询大于特定大小的分配
    pub fn allocations_larger_than(&self, min_size: usize) -> QueryResult {
        let snapshot = self.get_snapshot();
        let allocations: Vec<_> = snapshot.active_allocations.values()
            .filter(|a| a.size > min_size)
            .cloned()
            .collect();

        let total_bytes = allocations.iter().map(|a| a.size).sum();

        QueryResult::Allocations(AllocationQueryResult {
            count: allocations.len(),
            total_bytes,
            allocations,
        })
    }
}
```

### 2. 查询类型

**文件**: `src/query/types.rs`

**用途**: 查询结果的类型定义。

**查询结果**:

```rust
pub enum QueryResult {
    /// 分配查询结果
    Allocations(AllocationQueryResult),
    /// 线程查询结果
    Threads(ThreadQueryResult),
    /// 摘要查询结果
    Summary(SummaryQueryResult),
}

pub struct AllocationQueryResult {
    pub count: usize,
    pub total_bytes: usize,
    pub allocations: Vec<ActiveAllocation>,
}

pub struct ThreadQueryResult {
    pub count: usize,
    pub total_bytes: usize,
    pub threads: Vec<ThreadMemoryStats>,
}

pub struct SummaryQueryResult {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub active_allocations: usize,
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub current_memory: usize,
    pub peak_memory: usize,
    pub thread_count: usize,
}
```

## 使用示例

### 基本使用

```rust
use memscope::query::QueryEngine;
use memscope::snapshot::SnapshotEngine;
use std::sync::Arc;

// 创建查询引擎
let event_store = Arc::new(EventStore::new());
let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
let query_engine = QueryEngine::new(snapshot_engine);

// 查询摘要
let result = query_engine.summary();
match result {
    QueryResult::Summary(summary) => {
        println!("总分配: {}", summary.total_allocations);
        println!("当前内存: {} 字节", summary.current_memory);
    }
    _ => {}
}
```

### 查询顶级分配

```rust
// 获取按大小排序的前 10 个分配
let result = query_engine.top_allocations(10);
match result {
    QueryResult::Allocations(allocations) => {
        println!("顶级分配:");
        for alloc in &allocations.allocations {
            println!("  0x{:x}: {} 字节", alloc.ptr, alloc.size);
        }
    }
    _ => {}
}
```

### 按线程查询

```rust
// 获取线程 1 的分配
let result = query_engine.allocations_by_thread(1);
match result {
    QueryResult::Allocations(allocations) => {
        println!("线程 1 分配: {}", allocations.count);
    }
    _ => {}
}
```

### 查询大分配

```rust
// 查找大于 1MB 的分配
let result = query_engine.allocations_larger_than(1024 * 1024);
match result {
    QueryResult::Allocations(allocations) => {
        println!("大分配: {}", allocations.count);
    }
    _ => {}
}
```

## 设计原则

### 1. 统一接口
所有查询类型的单一接口：
- **优势**: 一致的 API，更易使用
- **权衡**: 特定查询的灵活性较低

### 2. 类型安全
查询结果的强类型：
- **优势**: 编译时安全，更好的文档
- **权衡**: 代码更冗长

### 3. 高效性
针对快速查询执行进行优化：
- **优势**: 快速分析
- **权衡**: 缓存可能使用更多内存

## 最佳实践

1. **查询优化**: 尽可能使用特定查询
2. **结果处理**: 始终处理所有查询结果类型
3. **线程安全**: 使用 Arc<QueryEngine> 进行共享访问
4. **错误处理**: 始终处理查询错误

## 限制

1. **基于快照**: 查询操作于快照，而非实时数据
2. **内存使用**: 查询结果可能很大
3. **性能**: 复杂查询可能较慢
4. **过滤**: 过滤功能有限

## 未来改进

1. **高级过滤**: 更强大的过滤选项
2. **聚合查询**: 分组、求和、平均值等
3. **基于时间的查询**: 按时间范围查询
4. **自定义查询**: 允许自定义查询逻辑
5. **查询缓存**: 缓存查询结果