# Codebase Strengthening Plan

Scan performed 2026-07-26.

---

## 1. 硬编码布局数据（P1）

`context.rs` 中 `build_task_topology_nodes` 使用硬编码坐标：
```
y_pct: 8.0   (root → 固定)
y_pct: 35.0 / 62.0  (子节点 → 固定两行)
x_pct: 按 fixed offset 公式计算 → 不会随真实 task 数量自适应
```
**修复**：根据 `async_tasks.len()` 动态计算行列数，多行自适应。

---

## 2. 数据字段定义但从未填充（P1）

`DashboardContext` / `AllocationInfo` 中：

| 字段 | 位置 | 状态 |
|------|------|------|
| `dynamic_type_info: Option<DynamicTypeInfo>` | `AllocationInfo` | 始终为 `None`，capture 层不填充 |
| `root_cause: Option<RootCauseInfo>` | `DashboardContext` | 仅 ownership_graph 链路可能填充 |

**建议**：标记 `#[serde(skip_serializing_if = "Option::is_none")]` 减少 JSON 体积，不做数据层改动。

---

## 3. unwrap() 非测试代码（P2）

| 位置 | 风险 |
|------|------|
| `render_methods.rs:1147` — `data.get("system_resources").unwrap()` | 若 JSON key 缺失直接 panic |
| `render_methods.rs:1177` — `.as_array().unwrap()` | 同上 |
| `async_tracker.rs:927` — `.unwrap()` | Mutex 锁失败 panic |
| `lockfree_tracker.rs:523` — `create_dir_all` unwrap | 磁盘满时 panic |

**建议**：关键路径（render_methods）改为 `?` 或 `context.get(key).and_then(...)` 传播错误，不要 unwrap。

---

## 4. `build_task_graph_json()` 空实现（P2）

`context.rs:595`：
```rust
fn build_task_graph_json() -> Result<String, Box<dyn std::error::Error>> {
    // Placeholder — this function is reserved for D3-compatible JSON export
    Ok("{}".to_string())
}
```
返回空 `{}`，`task_graph_json` 字段始终为空。

**建议**：若短期内无 D3 JSON 需求，标记 `#[deprecated]` 或删除。

---

## 5. take(N) 截断数据（P3）

多处使用 `.take(4)` / `.take(6)` 截断数据：
- `build_task_topology_nodes` → `.take(4)` 只取前 4 个 async task
- `build_task_topology_edges` → `.take(4)` 同样截断
- `build_waker_efficiency_grid` → `vec![0.0; 8]` 硬编码 8 个格子

**建议**：取消固定 take，改为按 `async_tasks.len()` 动态渲染，前端 SVG 自适应。

---

## 6. Option 字段序列化冗余（P3）

`selected_node_detail: Option<NodeDetailPanel>` 等 Option 字段即使为 `None` 也序列化到 JSON，每个 dashboard 输出增加 ~50KB。

**建议**：`#[serde(skip_serializing_if = "Option::is_none")]`。

---

## 7. 跨模块重复类型定义（P1）

`core/types/mod.rs` 和 `capture/types/*.rs` 中定义了**完全相同的结构体**，靠 `impl From<core::X> for capture::X` 做桥接：

| 类型 | core 位置 | capture 位置 |
|------|-----------|-------------|
| `DropChainAnalysis` | `core/types/mod.rs:3294` | `capture/types/drop_chain.rs:14` |
| `DropChainNode` | `core/types/mod.rs:3311` | `capture/types/drop_chain.rs:31` |
| `SmartPointerInfo` | `core/types/mod.rs:224` | `capture/types/smart_pointer.rs:13` |
| `MemoryLayoutInfo` | `core/types/mod.rs:1276` | `capture/types/memory_layout.rs:10` |
| `DynamicTypeInfo` | `core/types/mod.rs:1654` | `capture/types/dynamic_type.rs:10` |
| `RuntimeStateInfo` | `core/types/mod.rs:1734` | `capture/types/runtime_state.rs:10` |

每次新增字段要改两个文件、写 `From` impl，容易不同步。且 `DropChainAnalysis` 在 core 层始终为 `None`，capture 层有完整类型定义但从不采集。

**建议**：将 capture/types 作为唯一来源（single source of truth），core 层 `use crate::capture::types::*` 引用，删除重复定义。

---

## 8. analysis 层 unimplemented!() 桩函数（P2）

| 文件 | 行 | 问题 |
|------|----|------|
| `data_race_detector.rs:152` | `unimplemented!("Use DataRaceConfig directly")` | 两个方法不可用 |
| `double_free_detector.rs:102` | `unimplemented!("Use DoubleFreeConfig directly")` | 同上 |

**建议**：要么实现，要么标记 `#[deprecated]` 并公开说明。

---

## 9. 追踪后端碎片化（P2）

4 个 tracker backend 各有自己的 `track_allocation` / `track_deallocation`，签名不一致：

| Backend | 方法 | 返回值 |
|---------|------|--------|
| `core_tracker.rs` | `track_allocation(ptr, size)` | `TrackingResult<()>` |
| `lockfree_tracker.rs` | `track_allocation(ptr, size, call_stack_hash)` | `()`（无返回值） |
| `async_tracker.rs` | `track_allocation(ptr, size, task_id)` | `()` |
| `unsafe_tracking.rs` | `track_deallocation(ptr)` | `Result<()>` |

无统一 Trait，新增 backend 容易遗漏或签名不匹配。

**建议**：定义 `trait AllocationTracker { fn track_alloc(...) -> Result<()>; fn track_dealloc(...) -> Result<()>; }`，让各 backend 实现。

---

## 优先级更新

| 优先级 | 内容 | 工作量 |
|--------|------|--------|
| P1 | 硬编码布局数据 → 动态自适应 | ~0.5 天 |
| P1 | 跨模块重复类型定义去重 | ~0.5 天 |
| P1 | 标记 skip_serializing_if | ~0.2 天 |
| P2 | unwrap 改错误传播 | ~0.5 天 |
| P2 | 删除/废弃空实现函数 | ~0.1 天 |
| P2 | 追踪后端碎片化 → 统一 Trait | ~1 天 |
| P3 | 去掉 take(N) 截断 | ~0.3 天 |
