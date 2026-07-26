# Dashboard 扩展方案

## 原则

1. **禁止过度设计** — 每个新模式=一个 Handlebars `mode-section` + 一份 JS 渲染函数，不引入新框架
2. **数据驱动** — 只展示已捕获的数据，不新增追踪端采集逻辑
3. **渐进式** — 先做价值最高、改动最小的，验证后再做下一个

---

## 优先级 & 实施顺序

### P0 — 现有数据补全（半天）

| 数据源 | 改什么 |
|--------|--------|
| `smart_pointer.rs` | 在现有 Variables 模式里加一栏：Rc/Arc 引用计数快照、智能指针类型分布 |
| 改动范围 | 只改 `render_methods.rs` + `dashboard_unified.html` 的 Variables `mode-section` |

### P1 — 新增 Drop Chain 模式（1 天）

| 数据源 | 展示方式 |
|--------|----------|
| `drop_chain.rs` | 新 `mode-section="drop"`，左侧为 call chain 树（嵌套 `<ul>`），右侧为性能瓶颈列表 |
| 特殊点 | 使用现有 `DATA` 注入，JS 渲染为可展开的树形列表 |
| 数据字段 | `type_name`, `chain_length`, `estimated_latency_us`, `bottlenecks[]` |

### P2 — 新增 Runtime State 模式（1 天）

| 数据源 | 展示方式 |
|--------|----------|
| `runtime_state.rs` | 新 `mode-section="runtime"`，4 张 KPI 卡片 + 2 个图表 |
| 卡片 | CPU 使用率、内存压力等级、Cache L1/L2 命中率、分配器状态 |
| 图表 | 缓存性能趋势（折线图 SVG）、分配器活跃度（柱状图） |

### P3 — 新增 Memory Layout 模式（1 天）

| 数据源 | 展示方式 |
|--------|----------|
| `memory_layout.rs` | 新 `mode-section="layout"`，结构体布局可视化 + 容器利用率 |
| 展示 | padding 分析表、容量利用率条形图、重分配模式频率 |

### 不做（过度设计）

- ❌ Access Tracking — 需接入 perf 或 eBPF 数据，当前追踪层不支持
- ❌ Dynamic Type — VTable 信息在 Rust 中运行时获取成本高，数据不够稳定
- ❌ Drop Chain 火焰图 — SVG 火焰图实现复杂，树形列表足够

---

## 实现约束

- 所有模式共用 `dashboard_unified.html` 一个文件
- 新增字段统一在 `render_methods.rs` 的 `template_data` + `mod.rs` 的 `render_with_template` 注入
- JS 渲染函数用 IIFE 包裹，挂在 `DATA` 对象上读取
- 不新增 npm 包、不新增 CDN 脚本、不新增 CSS 框架
