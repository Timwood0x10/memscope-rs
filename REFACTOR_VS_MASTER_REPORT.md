# memscope-rs 重构对比报告

> 对比 refactor 分支和 master 分支的系统架构差异、功能覆盖率分析
>
> 生成时间: 2026-04-03
>
> 对比分支: master vs refactor

---

## 执行摘要

### 关键指标

| 指标 | Master 分支 | Refactor 分支 | 变化 |
|------|------------|---------------|------|
| **文件总数** | 377 | 506 | +129 |
| **新增文件** | - | 143 | - |
| **删除文件** | - | 14（6 个 src + 8 个 examples） | - |
| **代码行数变化** | - | +51,179 / -18,977 | - |
| **新增引擎模块** | - | 9 个 | - |
| **新增检测器** | - | 5 个 | - |

### 功能覆盖结论

**旧系统所有 28 个功能模块在新系统中均有对应实现，无功能丢失。**

迁移策略为：旧代码保留并标记 `#[deprecated]`，同时在新架构下重建。新旧代码并行存在。

---

## 新系统架构图

```
用户应用
    │
    ▼
┌──────────────┐
│  Facade API   │  MemScope 结构体，持有所有引擎的 Arc 引用
│  (facade/)    │  提供兼容层 (compat.rs) 和宏 (macros.rs)
└──────┬───────┘
       │
       ▼
┌──────────────────────────────────────────────────────┐
│                   捕获层                               │
│                                                      │
│  ┌─────────────┐    ┌─────────────┐                  │
│  │CaptureEngine│───▶│ Event Store │                  │
│  │ (4 种后端)   │    │ (无锁队列)   │                  │
│  └─────────────┘    └──────┬──────┘                  │
└────────────────────────────┼─────────────────────────┘
                             │
       ┌─────────────────────┼─────────────────────┐
       ▼                     ▼                     ▼
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│  Snapshot   │      │  Metadata   │      │  Timeline   │
│   Engine    │      │   Engine    │      │   Engine    │
│ (快照构建)   │      │ (元数据聚合) │      │ (索引+回放)  │
└──────┬──────┘      └─────────────┘      └──────┬──────┘
       │                                         │
       ▼                                         ▼
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│  Analysis   │      │   Query     │      │   Render    │
│   Engine    │      │   Engine    │      │   Engine    │
│ +检测器适配  │      │ (6种查询)    │      │ (JSON完整,   │
└─────────────┘      └─────────────┘      │  HTML独立函数)│
                                          └──────┬──────┘
                                                 │
                                                 ▼
                                          导出 (JSON/HTML)
```

---

## 代码树状图

```
src/
├── analysis/                          # 分析模块（重构）
│   ├── detectors/                     # 检测器系统（新增）
│   │   ├── mod.rs                     # Detector trait
│   │   ├── types.rs                   # 共享类型
│   │   ├── leak_detector.rs           # 464 行
│   │   ├── uaf_detector.rs            # 508 行
│   │   ├── overflow_detector.rs       # 837 行
│   │   ├── safety_detector.rs         # 813 行
│   │   └── lifecycle_detector.rs      # 822 行
│   ├── classification/                # 类型分类（重构为新目录）
│   ├── closure/                       # 闭包分析（重构为新目录）
│   ├── enhanced/                      # 增强分析（重构为新目录）
│   ├── estimation/                    # 大小估算（重构为新目录）
│   ├── generic/                       # 泛型分析（重构为新目录）
│   ├── lifecycle/                     # 生命周期分析（重构为新目录）
│   ├── metrics/                       # 性能指标（重构为新目录）
│   ├── quality/                       # 质量保证（重构为新目录）
│   ├── safety/                        # 安全分析（重构为新目录）
│   ├── security/                      # 安全违规分析（重构为新目录）
│   ├── unknown/                       # 未知类型分析（重构为新目录）
│   └── [旧文件仍保留: circular_reference, async_analysis,
│        borrow_analysis, variable_relationships,
│        memory_passport_tracker, unsafe_ffi_tracker,
│        ffi_function_resolver, lifecycle_analysis.rs]
├── analysis_engine/                   # 分析引擎（新增）
│   ├── analyzer.rs                    # Analyzer trait
│   ├── engine.rs                      # AnalysisEngine
│   └── detector_adapter.rs            # Detector→Analyzer 适配器
├── capture/                           # 捕获引擎（新增）
│   ├── engine.rs                      # CaptureEngine
│   ├── backends/
│   │   ├── mod.rs                     # CaptureBackend trait + 4 种后端
│   │   ├── core_tracker.rs            # CoreBackend
│   │   ├── async_tracker.rs           # AsyncBackend
│   │   ├── lockfree_tracker.rs        # LockfreeBackend
│   │   ├── unified_tracker.rs         # UnifiedCaptureBackend（占位，默认用 CoreBackend）
│   │   ├── unsafe_tracking.rs         # 不安全代码追踪
│   │   ├── bottleneck_analysis.rs     # 瓶颈分析
│   │   ├── hotspot_analysis.rs        # 热点分析
│   │   ├── efficiency_scoring.rs      # 效率评分
│   │   └── resource_ranking.rs        # 资源排名
│   ├── platform/                      # 平台支持
│   └── types/                         # 捕获类型
├── event_store/                       # 事件存储（新增）
│   ├── event.rs
│   └── store.rs                       # crossbeam SegQueue 无锁队列
├── facade/                            # 门面模式（新增）
│   ├── facade.rs                      # MemScope（436 行）
│   ├── compat.rs                      # 兼容层
│   └── macros.rs                      # 便捷宏
├── metadata/                          # 元数据引擎（新增）
│   ├── engine.rs                      # 薄聚合层（80 行）
│   ├── registry.rs                    # 变量注册表
│   ├── scope.rs                       # 作用域追踪
│   ├── thread.rs                      # 线程信息
│   ├── smart_pointers/
│   └── stack_trace/
├── query/                             # 查询引擎（新增）
│   ├── engine.rs                      # 6 种查询方法
│   ├── presets.rs
│   └── types.rs
├── render_engine/                     # 渲染引擎（新增）
│   ├── engine.rs
│   ├── renderer.rs                    # Renderer trait（目前仅注册 JsonRenderer）
│   ├── export.rs                      # HTML/JSON 导出（独立函数，未整合到 Renderer trait）
│   └── dashboard/
│       ├── renderer.rs
│       └── assets/
├── snapshot/                          # 快照引擎（新增）
│   ├── engine.rs
│   ├── types.rs
│   └── memory/
├── timeline/                          # 时间线引擎（新增）
│   ├── engine.rs
│   ├── index.rs
│   ├── query.rs
│   └── replay.rs
├── core/                              # 旧核心模块（保留，标记 deprecated）
├── lockfree/                          # 旧 lockfree 模块（保留，标记 deprecated）
├── unified/                           # 旧统一后端（保留，标记 deprecated）
├── async_memory/                      # 旧异步内存（保留，标记 deprecated）
├── export/                            # 旧导出模块（保留，标记 deprecated）
├── classification/                    # 旧类型分类（保留，标记 deprecated）
├── estimation/                        # 旧大小估算（保留，标记 deprecated）
├── metrics/                           # 旧性能监控（保留，标记 deprecated）
├── quality/                           # 旧质量保证（保留，标记 deprecated）
├── smart_pointers/                    # 旧智能指针（保留，标记 deprecated）
├── stack_trace/                       # 旧堆栈跟踪（保留，标记 deprecated）
├── error/                             # 错误处理（保留）
├── cli/                               # 命令行界面（保留）
└── [其他保留模块...]
```

---

## 详细功能对比

### 1. 核心引擎模块（新增 9 个）

| 引擎模块 | 状态 | 备注 |
|---------|------|------|
| **Capture Engine** | 完整实现 | 4 种后端，CaptureBackend trait 抽象清晰 |
| **Event Store** | 完整实现 | crossbeam SegQueue，有并发测试 |
| **Analysis Engine** | 完整实现 | 可插拔分析器 + DetectorToAnalyzer 适配器 |
| **Query Engine** | 完整实现 | 6 种查询方法，有测试 |
| **Timeline Engine** | 完整实现 | 索引 + 时间范围查询 + 事件回放，有测试 |
| **Render Engine** | 部分实现 | JSON 渲染通过 Renderer trait 完整；HTML 导出通过独立函数，未整合到插件体系 |
| **Snapshot Engine** | 完整实现 | 从事件构建快照，有测试 |
| **Metadata Engine** | 完整实现 | 薄聚合层（80 行），聚合子模块 |
| **Facade API** | 完整实现 | MemScope 结构体 + 兼容层 + 宏 |

### 2. 检测器系统（新增 5 个）

| 检测器 | 代码量 | 实现内容 |
|--------|--------|---------|
| **LeakDetector** | 464 行 | 泄漏检测 + 智能指针循环检测 + 引用计数异常 |
| **UafDetector** | 508 行 | 裸指针 UAF + 生命周期违规 + 多重可变借用 |
| **OverflowDetector** | 837 行 | 堆溢出 + 栈溢出 + 整数溢出 |
| **SafetyDetector** | 813 行 | 不安全代码 + FFI + static mut + 裸指针 + 数据竞争 |
| **LifecycleDetector** | 822 行 | 生命周期 + 所有权模式 + Drop trait + 借用违规 |

所有检测器均实现 `Detector` trait，有配置结构体和单元测试。

> 注意：报告中提到的"准确率 85%-95%"和"分析时间 < 1-2ms"无实际 benchmark 支撑，为预期目标而非实测数据。

### 3. 分析模块迁移状态

| 功能模块 | Master | Refactor | 状态 |
|---------|--------|----------|------|
| 循环引用检测 | `circular_reference.rs` | `circular_reference.rs` | 保留 |
| 闭包分析 | `closure_analysis.rs` | `analysis/closure/` | 重构为目录 |
| 泛型分析 | `generic_analysis.rs` | `analysis/generic/` | 重构为目录 |
| 安全分析 | `safety_analyzer.rs` | `analysis/safety/` | 重构为目录 |
| 安全违规分析 | `security_violation_analyzer.rs` | `analysis/security/` | 重构为目录 |
| 未知类型分析 | `unknown_memory_regions.rs` | `analysis/unknown/` | 重构为目录 |
| 增强分析 | `enhanced_memory_analysis.rs` | `analysis/enhanced/` | 重构为目录 |
| 类型分类 | `classification/` | `analysis/classification/` + 旧 `classification/` | 新旧并存 |
| 大小估算 | `estimation/` | `analysis/estimation/` + 旧 `estimation/` | 新旧并存 |
| 性能监控 | `metrics/` | `analysis/metrics/` + 旧 `metrics/` | 新旧并存 |
| 质量保证 | `quality/` | `analysis/quality/` + 旧 `quality/` | 新旧并存 |
| 生命周期分析 | `lifecycle_analysis.rs` | `analysis/lifecycle/` + **旧文件仍存在** | 新旧并存 |
| 不安全 FFI 追踪 | `unsafe_ffi_tracker.rs` | 保留 + 新增 `capture/backends/unsafe_tracking.rs` | 保留并增强 |
| 变量关系分析 | `variable_relationships.rs` | 保留 | 保留 |
| 异步分析 | `async_analysis.rs` | 保留 | 保留 |
| 借用分析 | `borrow_analysis.rs` | 保留 | 保留 |
| FFI 函数解析器 | `ffi_function_resolver.rs` | 保留 | 保留 |
| 内存护照追踪器 | `memory_passport_tracker.rs` | 保留 | 保留 |

### 4. 辅助模块

| 功能模块 | 状态 |
|---------|------|
| 导出功能 | 旧 `export/` 保留（deprecated），新 `render_engine/export.rs` 提供独立导出函数 |
| 错误处理 | 保留 |
| CLI | 保留 |
| 高级类型分析 | 保留 |
| 增强类型 | 保留 |
| 可跟踪宏 | 保留 + 新增 `facade/macros.rs` |
| 变量注册表 | 保留 + 新增 `metadata/registry.rs` |
| 追踪统计 | 保留 |
| 工具函数 | 保留 |
| 系统监控 | 保留（独立为 611 行模块） |

---

## 已知问题和局限性

### 占位/未完成实现

1. **UnifiedCaptureBackend 是占位** — `Default` 实现直接返回 `CoreBackend`，没有真正实现自动选择最优后端的逻辑。

2. **LockfreeBackend 的 hash_call_stack 是占位** — 使用时间戳哈希而非真实调用栈捕获。

3. **AnalysisManager 部分方法是空壳** — `analyze_fragmentation()`、`analyze_system_libraries()`、`analyze_concurrency_safety()` 返回默认值，无实际分析逻辑。

4. **Render Engine 未完全统一** — 只有 JSON 渲染通过 `Renderer` trait 实现。HTML 和二进制导出仍通过独立的 `export_*` 函数，未整合到 Renderer 插件体系。

### 代码膨胀

旧模块全部保留并标记 `#[deprecated]`，与新架构并行存在，导致文件数从 377 增至 506。建议在确认新架构稳定后逐步移除旧代码。

### 测试状态

存在 12 个 doctest 失败（`DetectionStatistics` 类型未找到等编译错误），需修复。

---

## 迁移指南

### API 变化

旧 API（仍可用，标记 deprecated）：
```rust
use memscope_rs::core::tracker::MemoryTracker;
let tracker = MemoryTracker::new();
let allocations = tracker.get_active_allocations()?;
```

新 API：
```rust
use memscope_rs::facade::MemScope;
let memscope = MemScope::new();
let snapshot = memscope.snapshot()?;
let allocations = &snapshot.active_allocations;
```

### 检测器使用

```rust
let memscope = MemScope::new();
let leak_result = memscope.run_leak_detector()?;
let uaf_result = memscope.run_uaf_detector()?;
let overflow_result = memscope.run_overflow_detector()?;
let safety_result = memscope.run_safety_detector()?;
let lifecycle_result = memscope.run_lifecycle_detector()?;
```

### 导出功能

```rust
memscope.export_html("./output")?;
memscope.export_json("./output/memory.json")?;
```

---

## 结论

### 新系统能否替代旧系统？

**是的，功能上可以。** 旧系统所有 28 个功能模块在新系统中均有对应实现。

### 但需要注意

1. **旧代码未清理** — 新旧代码并行存在，代码库膨胀。建议迁移完成后删除旧模块。
2. **部分实现为占位** — UnifiedCaptureBackend、LockfreeBackend hash_call_stack、AnalysisManager 部分方法需要后续完善。
3. **Render Engine 不统一** — HTML 导出未整合到 Renderer trait 体系。
4. **测试有失败** — 12 个 doctest 编译错误需修复。
5. **无性能 benchmark** — 报告中所有性能提升数据（+50% 速度等）无实际 benchmark 支撑，不应作为决策依据。

### 推荐行动

1. 修复 12 个失败的 doctest
2. 完善 UnifiedCaptureBackend 的自动选择逻辑
3. 将 HTML 导出整合到 Render Engine 的 Renderer trait 体系
4. 实现 AnalysisManager 的空壳方法
5. 确认稳定后逐步移除标记 deprecated 的旧代码

---

## 附录：完整模块映射表

| 旧系统模块 | 新系统模块 | 迁移状态 |
|-----------|-----------|---------|
| `core/tracker/` | `capture/backends/core_tracker.rs` | 已迁移（旧代码保留） |
| `async_memory/` | `capture/backends/async_tracker.rs` | 已迁移（旧代码保留） |
| `lockfree/` | `capture/backends/lockfree_tracker.rs` | 已迁移（旧代码保留） |
| `unified/` | `capture/backends/unified_tracker.rs` | 已迁移（占位实现） |
| `core/types/` | `capture/types/` | 已迁移 |
| `platform/` | `capture/platform/` | 已迁移 |
| `analysis/closure_analysis.rs` | `analysis/closure/` | 已迁移（旧文件已删除） |
| `analysis/enhanced_memory_analysis.rs` | `analysis/enhanced/` | 已迁移（旧文件已删除） |
| `analysis/generic_analysis.rs` | `analysis/generic/` | 已迁移（旧文件已删除） |
| `analysis/safety_analyzer.rs` | `analysis/safety/` | 已迁移（旧文件已删除） |
| `analysis/security_violation_analyzer.rs` | `analysis/security/` | 已迁移（旧文件已删除） |
| `analysis/unknown_memory_regions.rs` | `analysis/unknown/` | 已迁移（旧文件已删除） |
| `classification/` | `analysis/classification/` | 已迁移（旧代码保留） |
| `estimation/` | `analysis/estimation/` | 已迁移（旧代码保留） |
| `metrics/` | `analysis/metrics/` | 已迁移（旧代码保留） |
| `quality/` | `analysis/quality/` | 已迁移（旧代码保留） |
| `smart_pointers/` | `metadata/smart_pointers/` | 已迁移（旧代码保留） |
| `stack_trace/` | `metadata/stack_trace/` | 已迁移（旧代码保留） |
| `memory/` | `snapshot/memory/` | 已迁移 |
| `export/` | `render_engine/export.rs` | 已迁移（旧代码保留） |
| `system_monitor.rs` | `system_monitor.rs`（独立模块） | 已迁移 |

---

*报告基于实际代码对比生成*
*日期: 2026-04-03*
