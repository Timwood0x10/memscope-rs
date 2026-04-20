# 智能指针追踪与循环引用检测

## 概述

Phase 1 专注于智能指针的追踪和循环引用检测，通过自动检测和简化 API，提供强大的内存泄漏检测能力。

## 完成的功能

### P1.1 Clone 操作检测（宏层）

**目标**：通过宏检测 clone 操作，记录克隆源和目标

**实现**：
- `track_clone!` 宏记录克隆操作
- 在 `MemoryEvent` 中添加 `clone_source_ptr` 和 `clone_target_ptr` 字段
- 在 `rebuild_allocations_from_events` 中维护 `clone_info_map`

**优化**：
- 已集成到 `track!` 宏中，无需单独使用 `track_clone!`
- 在 `rebuild_allocations_from_events` 中自动检测智能指针类型
- 通过类型名称（Arc、Rc、Box）自动填充 `smart_pointer_info`

**效果**：
- 自动检测智能指针克隆操作
- 简化 API，只需使用 `track!` 宏
- 支持循环引用检测的数据基础

### P1.2 智能指针 Opt-in 追踪

**目标**：集成现有循环引用检测，利用 clone 信息构建引用图

**实现**：
- 在 `rebuild_allocations_from_events` 中自动检测智能指针类型
- 填充 `smart_pointer_info` 字段
- 集成 `detect_circular_references` 函数
- 在 `DashboardContext` 中添加 `circular_references` 字段

**数据结构**：
```rust
pub struct SmartPointerInfo {
    pub data_ptr: usize,
    pub pointer_type: SmartPointerType,
    pub is_data_owner: bool,
    pub ref_count_history: Vec<u64>,
    pub weak_count: Option<u64>,
    pub cloned_from: Option<usize>,
    pub clones: Vec<usize>,
    pub is_implicitly_deallocated: bool,
    pub is_weak_reference: false,
}
```

**循环引用检测**：
- 构建 `ReferenceGraph` 从分配信息
- 使用 DFS 检测循环
- 计算泄漏内存和循环统计

**效果**：
- 自动检测 Arc/Rc 循环引用
- 提供循环路径和泄漏估计
- 可视化循环引用报告

## 集成改进

### 统一数据源

**目标**：确保所有功能通过单一数据源（event_store）和单一处理流程（rebuild_allocations_from_events）

**实现**：
- 所有数据来自 `event_store.snapshot()`
- 统一通过 `rebuild_allocations_from_events` 处理
- 返回 `capture::types::AllocationInfo`（包含 `smart_pointer_info`）

**数据流**：
```
event_store (MemoryEvent)
  ↓
rebuild_allocations_from_events
  ↓
capture::types::AllocationInfo (含 smart_pointer_info)
  ↓
各报告构建函数
  ↓
DashboardContext
  ↓
HTML Dashboard
```

### 自动生命周期追踪

**目标**：无需用户手动操作，自动记录 deallocation

**实现**：
- 在 `Tracker::drop()` 中自动记录所有活跃分配的 deallocation 事件
- 自动计算生命周期（lifetime_ms）
- 无需用户手动调用 `drop()`

**效果**：
- 完全自动的生命周期追踪
- 用户无需额外操作
- 准确的生命周期数据

### API 简化

**目标**：减少宏的数量，简化使用

**实现**：
- 移除 `track_clone!` 宏依赖
- 在 `rebuild_allocations_from_events` 中自动检测智能指针
- 只需使用 `track!` 宏即可

**使用方式**：
```rust
// 之前：需要两个宏
track!(tracker, data);
track_clone!(tracker, source, target);

// 现在：只需要一个宏
track!(tracker, data); // 自动检测智能指针
```

## 使用示例

### 基础智能指针追踪

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};
use std::rc::Rc;
use std::sync::Arc;

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // 智能指针自动检测
    let rc_data = Rc::new(vec![1.0, 2.0, 3.0]);
    track!(tracker, rc_data);

    let arc_data = Arc::new(vec![1.0, 2.0, 3.0]);
    track!(tracker, arc_data);

    // 自动生命周期追踪
    // 无需手动 drop，Tracker drop 时自动记录

    // 分析结果
    let mut az = analyzer(&tracker)?;
    let report = az.analyze();
    
    println!("Smart pointers detected: {}", 
        report.circular_references.total_smart_pointers);
    println!("Circular references: {}", 
        report.circular_references.count);
    
    Ok(())
}
```

### 循环引用检测

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};
use std::cell::RefCell;
use std::rc::Rc;

struct Node {
    data: i32,
    next: Option<Rc<RefCell<Node>>>,
}

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // 创建循环引用
    let node1 = Rc::new(RefCell::new(Node { data: 1, next: None }));
    let node2 = Rc::new(RefCell::new(Node { data: 2, next: None }));

    node1.borrow_mut().next = Some(node2.clone());
    node2.borrow_mut().next = Some(node1.clone());

    track!(tracker, node1);
    track!(tracker, node2);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    if report.circular_references.has_cycles {
        println!("Detected {} circular references!", report.circular_references.count);
        println!("Total leaked memory: {} bytes", report.circular_references.total_leaked_memory);
    }

    Ok(())
}
```

### 混合类型追踪

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};
use std::rc::Rc;
use std::sync::Arc;
use std::boxed::Box;

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // 混合使用不同智能指针类型
    let rc_data = Rc::new(vec![1, 2, 3]);
    track!(tracker, rc_data);

    let arc_data = Arc::new(vec![4, 5, 6]);
    track!(tracker, arc_data);

    let boxed_data = Box::new(42);
    track!(tracker, boxed_data);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    println!("Total smart pointers: {}", report.circular_references.total_smart_pointers);

    Ok(())
}
```

## 循环引用检测

### 检测机制

1. **构建引用图**：
   - 从 `smart_pointer_info` 提取引用关系
   - 构建邻接表和反向引用映射
   - 跳过 weak 引用（不创建强循环）

2. **检测循环**：
   - 使用 DFS 遍历引用图
   - 识别长度 ≥ 2 的循环
   - 分析循环路径和泄漏估计

3. **生成报告**：
   - 循环引用数量
   - 总泄漏内存
   - 循环中的指针数量
   - 统计信息

### 报告字段

```rust
pub struct CircularReferenceReport {
    pub count: usize,                    // 循环引用数量
    pub total_leaked_memory: usize,       // 总泄漏内存
    pub pointers_in_cycles: usize,        // 循环中的指针数量
    pub total_smart_pointers: usize,      // 总智能指针数量
    pub has_cycles: bool,                // 是否存在循环
}
```

## 性能特性

- **自动检测**：无需手动标记智能指针
- **零开销**：类型检测在数据处理阶段进行
- **统一处理**：单一数据源和单一处理流程
- **简化 API**：只需一个 `track!` 宏

## 限制

- 只能检测智能指针（Arc、Rc、Box）的循环引用
- 无法检测普通引用的循环引用
- 循环检测基于克隆关系，不是真实的所有权转移
- 真正的所有权转移追踪需要 Phase 3（MIR 提取）
