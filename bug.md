# Bug Record

## Bug #1: real\_world\_demo.rs 在 export\_json 时 hang 住或崩溃（段错误）

**问题描述**
real\_world\_demo.rs 在运行 export\_json 时会 hang 住或者发生段错误（segmentation fault）导致程序被系统 kill。

**根本原因**
Container 类型的 ptr 被设置为虚拟指针（0x100000000 + index），而 RelationGraphBuilder::build 会尝试读取这些虚拟指针的内存，导致读取无效内存地址。

**具体表现**

1. Container 类型的 ptr 被设置为虚拟指针（0x100000000 + index）
2. RelationGraphBuilder::build 会尝试读取这些虚拟指针的内存
3. HeapScanner::scan、detect\_owner、RangeMap::new 等函数会扫描所有分配的内存，包括虚拟指针
4. 导致读取无效内存地址，造成段错误或 hang 住
5. 初始的虚拟指针基准值 0x100000000 太低，在 macOS 上会与真实堆地址冲突（真实地址可达 0x136704580）

**解决方案**

1. 将虚拟指针基准值从 0x100000000 提高到 0x10000000000（1TB）
2. 在以下函数中添加虚拟指针跳过逻辑：
   - `HeapScanner::dedup_heap_regions`：跳过 >= 0x10000000000 的指针
   - `detect_owner_impl`：跳过 >= 0x10000000000 的指针值
   - `RangeMap::new`：跳过 >= 0x10000000000 的指针
   - `build_ownership_graph_from_allocations`：跳过 Container 类型分配
3. 更新 `rebuild_allocations_from_events` 中的虚拟指针分配逻辑

**修改的文件**

- `/memscope-rs/src/analysis/heap_scanner/reader.rs`
- `/memscope-rs/src/analysis/relation_inference/pointer_scan.rs`
- `/memscope-rs/src/analysis/relation_inference/range_map.rs`
- `/memscope-rs/src/render_engine/export.rs`
- `/memscope-rs/src/render_engine/dashboard/renderer.rs`

**验证结果**

- ✅ 所有 920 个测试通过
- ✅ make check 0 errors
- ✅ real\_world\_demo 成功运行并生成所有报告

***

## Bug #2: Container 类型内存跟踪问题

**问题描述**
HashMap、BTreeMap 等 Container 类型有复杂的内部内存结构，之前需要使用虚假指针来参与关系图构建，导致 HeapScanner 扫描困难，容易触发段错误。

**根本原因**
Container 类型（如 HashMap、BTreeMap）有复杂的内部内存布局，无法通过简单的指针扫描来推断其包含的对象。之前的方案尝试使用虚假指针，但这种方法不可靠且容易导致段错误。

**解决方案（三层对象模型）**

### 1. 引入 TrackKind 枚举（`core/types/track_kind.rs`）

- `HeapOwner`：真正拥有堆内存的对象（Vec、Box、String、Arc、Rc）
- `Container`：组织数据但不直接暴露堆（HashMap、BTreeMap、VecDeque）
- `Value`：无堆分配的普通数据（基本类型、简单结构体）

### 2. 实现 Container 检测器（`analysis/relation_inference/container_detector.rs`）

基于以下启发式规则推断 Contains 关系：

- **时间局部性**：Container 和其包含的对象通常在短时间内分配（默认 1ms 窗口）
- **线程亲和性**：对象必须在同一线程分配
- **大小合理性**：包含的对象不应显著大于容器（默认 10 倍比例）

算法复杂度：

- 时间：O(N)，其中 N 是分配数量，得益于时间排序的滑动窗口方法
- 空间：O(1)，除分配列表外的额外空间

### 3. 优化 HeapScanner

- 只扫描 HeapOwner 类型的分配
- 跳过 Container 和 Value 类型
- 提升性能，避免读取无效内存

### 4. 添加 variable evolution tracking

- 跟踪同一变量名的多次分配
- 推断变量演化关系（如增长的 HashMap、重新分配的 Vec）

**相关 Commits**

- `9864012`: 实现三层对象模型
- `bc17bf5`: 添加变量演化跟踪和源代码位置支持

**修改的文件**

- `src/core/types/track_kind.rs` - TrackKind 枚举定义
- `src/analysis/relation_inference/container_detector.rs` - Container 检测器实现
- `src/analysis/relation_inference/graph_builder.rs` - 关系图构建器集成
- `src/analysis/heap_scanner/reader.rs` - HeapScanner 优化
- `src/render_engine/export.rs` - 导出逻辑更新

**验证结果**

- ✅ 所有单元测试通过（464 行测试代码）
- ✅ Container 检测器 16 个测试全部通过
- ✅ 实际运行 real\_world\_demo 成功显示 Contains 关系
- ✅ Dashboard 图正确显示 Container 节点和关系

***

## 总结

这两个 bug 的解决展示了 memscope-rs 项目从最初的"虚假指针"方案发展到更可靠的"三层对象模型"方案的过程：

1. **Bug #1** 解决了虚拟指针与真实内存地址冲突的技术问题
2. **Bug #2** 从根本上解决了 Container 类型内存跟踪的架构问题

这些改进使得 memscope-rs 能够：

- 正确跟踪复杂的容器类型（HashMap、BTreeMap 等）
- 避免读取无效内存导致的段错误
- 提供更准确的内存关系分析
- 支持更丰富的内存可视化

两个 bug 的解决都遵循了严格编码规范，通过了所有测试和代码检查。

***

## Bug #3: 虚拟指针设计缺陷和正确的 Allocation 模型

**问题描述**
最初的虚拟指针方案（fake pointer）是一个设计层问题，不是 HeapScanner 的 bug。使用虚拟指针会破坏地址语义，导致 HeapScanner 尝试读取不确定地址，引发 SIGSEGV。

**根本原因**

1. **Trackable trait 的设计缺陷**
   - 原设计：`Trackable::get_heap_ptr() -> Option<usize>`
   - 隐含假设：每个 tracked object 都有 heap ptr
   - 实际情况：Rust 类型并非都有 heap ptr（HashMap、struct、stack variable）

2. **虚拟指针的问题**
   - 破坏地址语义：fake pointer 污染 RangeMap、PointerScan、OwnerDetect
   - 安全风险：HeapScanner 尝试读取无效地址
   - 不可维护：复杂、危险、不安全

**正确的模型（三层对象模型）**

### 1. 明确的 Allocation 分类

```rust
enum AllocationKind {
    Heap {
        ptr: usize,
        size: usize,
    },
    NonHeap,
}
```

### 2. Trackable trait 正确设计

```rust
trait Trackable {
    fn allocation(&self) -> AllocationKind;
}
```

示例：
- `Vec<T>` → `AllocationKind::Heap { ptr, size }`
- `String` → `AllocationKind::Heap { ptr, size }`
- `HashMap<K,V>` → `AllocationKind::NonHeap`

### 3. HeapScanner 的正确行为

```rust
allocations
    .iter()
    .filter(|a| matches!(a.kind, AllocationKind::Heap{..}))
```

只扫描 Heap allocation，不扫描 NonHeap object。

**设计原则**

1. **不要使用虚拟指针**
2. **不要伪造内存**
3. **不要让 HeapScanner 读取不确定地址**

**HashMap 的真实情况**

HashMap 实际上是：
```
HashMap (stack object)
   │
   ▼
RawTable
   │
   ▼
heap buckets
```

由于 Rust 没有稳定 API 获取 bucket pointer，所以不能可靠实现 `get_heap_ptr()`。

正确做法：HashMap = NonHeap container，只跟踪 HashMap object，而不是内部 bucket。

**性能优化：Heap Region Dedup**

避免对同一块 heap 内存重复扫描。

### 优化前

```
allocations
   │
   ▼
HeapScanner.scan()
   │
   ├─ scan alloc1 memory
   ├─ scan alloc2 memory
   ├─ scan alloc3 memory
   └─ ...

问题：很多 allocation 指向同一块 heap region
例如：Arc clone、slice、iterator、struct field reference
```

### 优化后

```rust
use std::collections::HashSet;

fn dedup_heap_regions(allocs: &[Allocation]) -> Vec<(usize, usize)> {
    let mut seen = HashSet::new();
    let mut regions = Vec::new();

    for a in allocs {
        let key = (a.ptr, a.size);
        if seen.insert(key) {
            regions.push(key);
        }
    }
    regions
}
```

**性能提升**

- 假设 10000 allocations，真实 heap block 2000
- 扫描次数：10000 → 2000（减少 80%）
- 如果有大量 Arc/Slice：甚至能减少 90%

**符合项目哲学**

| 要求 | 结果 |
|------|------|
| zero fake pointer | ✅ |
| safe heap scan | ✅ |
| portable | ✅ |
| simple | ✅ |

**实现成本**

- 代码修改：~80 行
- 架构改动：0
- 复杂度：极低
- 性能提升：巨大

**这是 memory profiler 的标准做法**

很多工具也是 heap allocations 和 stack objects 分开处理，例如：

- heaptrack
- jemalloc profiler
- pprof

他们不会扫描 stack objects。

**总结**

这个问题的本质是：pointer model 没定义清楚。

正确模型是：

```
Heap allocation vs NonHeap object
```

最终架构：

```
Tracking
   │
   ▼
AllocationEvent
   │
   ├── Heap
   │     └─ ptr,size
   │
   └── NonHeap
         └─ metadata only

然后：
HeapScanner
   ↓
only Heap allocations
```

**参考文档**

- 设计建议详见：`./aim/bug/bug.md`
- 实现详见：`src/core/types/track_kind.rs`
- 优化详见：`src/analysis/heap_scanner/reader.rs::dedup_heap_regions`
