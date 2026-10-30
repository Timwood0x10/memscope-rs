# 数据准确性增强开发计划

> 状态：**PR-01 已实施**（见 §17），PR-02–09 仍为设计方案，不代表已经实现。
>
> 基线：`dev` 分支，`c8db9de`，版本 `0.2.5`。编写日期：2026-10-30。
>
> 目标：让采集、生命周期重建、统计、关系分析、JSON 和 Dashboard 对同一批数据给出一致、可解释、可回归验证的结果。

阅读导航：

- [现状与修改入口](#2-当前实现问题与修改入口)
- [数据结构设计](#4-数据结构设计)
- [重建与统计算法](#5-唯一重建器事件如何改变状态)
- [采集层接入](#6-采集层如何接入)
- [关联模块修改](#7-关联模块如何修改)
- [JSON 与 Dashboard](#9-dashboard-与-json-字段改造)
- [测试与验收](#12-测试设计与验收矩阵)
- [PR 拆分](#13-pr-拆分依赖与预计工作量)
- [完整文件清单](#14-代码变更总表)
- [评审意见与源码核查](#16-评审意见核查记录)

## 1. 开发顺序与交付目标

按以下顺序实施：

1. **修正生命周期事实**：停止生成虚假释放，区分存活、结束观测和泄漏。
2. **确定身份与事件契约**：区分堆分配、变量观测、智能指针句柄；统一 ID、时钟和来源。
3. **统一事件重建**：Snapshot、MemoryView、Analyzer、Dashboard、JSON 共用一个状态归约器。
4. **接通采集与归属**：allocator、显式插桩、FFI、线程和异步任务统一入账，消除重复计数。
5. **修正展示口径**：字段级证据、采集完整性、指标公式、前端索引与未知值。
6. **建立准确性基准并改进推断**：真实标签、误报/漏报、拒判机制、UTI 校准。

建议按第 13 节的 9 个逻辑工作包实施，其中 PR-09 拆为 a/b/c 三次滚动交付。PR-01 可独立交付；后续涉及公共数据模型的改动按下一个 minor 版本管理。

### 1.1 必须达到的结果

- 同一个观测窗口内，Snapshot、Tracker 报告、Analyzer、HTML 内嵌 JSON 和磁盘 JSON 的公共统计一致。
- 真实堆分配只入账一次；重复 `track!`、Arc/Rc clone、变量元数据不增加真实堆分配数量。
- 地址复用不继承前一个对象的 clone、Passport、泄漏或所有权信息。
- 完整事件流可以正确重放 allocate、deallocate、原地 realloc、搬迁 realloc 和失败 realloc。
- 观测缺失有明确状态；没有观测到释放不能直接推导“已泄漏”。
- “已采集的事实”“由事实计算的指标”“启发式推断”能在字段级追溯来源。
- 基准能够报告实际测量结果，文档中的准确率不再以预估值代替实测。

### 1.2 准确性的适用范围

所有“精确”承诺都必须附带 `capture_mode`、观测时间范围和数据完整性。

| 采集方式 | 可以知道什么 | 输出约束 |
| --- | --- | --- |
| 普通 `track!(&value)` | 该调用时的类型、部分容器属性和地址 | 是一次对象观测；不能自动知道此前分配时间、之后的 drop 或中途 realloc |
| allocator hook | 经过该 allocator 的请求大小、地址和 alloc/free/realloc | 只覆盖对应 allocator 与启用窗口；不自动覆盖 C 的 malloc/free |
| 显式生命周期插桩 | 插桩声明的分配、释放、clone、交接 | API 必须规定在实际操作成功后记录，以及重复记录规则 |
| 运行时 hook | task poll、wake、线程执行信息 | 只能展示实际接通的指标 |
| 字节/布局推断 | 类型或关系候选 | 必须附来源、分数和拒判理由 |

默认 features 不含 `tracking-allocator`。因此，仅调用 `start()` 和普通 `track!` 的程序不能被标为“完整 allocator 事件流”。

## 2. 当前实现问题与修改入口

以下为静态源码检查发现的行为，尚未逐项运行回归复现；实施时先为每项建立回归场景，再按后文设计修改。

| 编号 | 当前行为 | 主要入口 | 影响 |
| --- | --- | --- | --- |
| A01 | `Tracker::drop()` 为所有存活记录生成 Deallocate；随后还有独立的可选自动导出分支 | `src/tracker.rs:578–608` | 跟踪器销毁被误当成对象释放；clone 共享事件库时也会发生。只删除释放循环，按 §8.1 保留旧入口导出 |
| A02 | Dashboard 根据 `timestamp_dealloc.is_none()` 设置 `is_leaked` | `src/render_engine/dashboard/renderer/report_builder.rs` | 正常存活对象被显示为泄漏 |
| A03 | Dashboard 重建忽略 Reallocate，Clone map 以裸地址为键 | `src/render_engine/dashboard/renderer/event_reconstructor.rs` | 大小过期；地址复用串联旧关系 |
| A04 | Snapshot 与 Dashboard 使用不同重建器 | `src/snapshot/builder.rs`、`event_reconstructor.rs` | 重复地址、Metadata、Clone、realloc 的结果不一致 |
| A05 | StackOwner 用堆地址发 Allocate，内层 tracker 却用句柄地址记账 | `src/tracker.rs::track_inner` | Arc/Rc 多句柄被混同为多次堆分配，释放关联口径不一致 |
| A06 | Snapshot 对重复地址覆盖 map，但仍累加 bytes/count | `src/snapshot/builder.rs` | map 中的存活集合与累计统计不守恒 |
| A07 | 跨线程释放按释放线程扣减 current memory | `src/snapshot/builder.rs` | 原分配线程的存活归属无法清零 |
| A08 | `MemoryEvent::allocate()` 修改 TaskIdRegistry，GlobalTracker 路径还可能再次记录 | `src/event_store/event.rs`、`src/capture/backends/global_tracking.rs` | 构造事件有副作用，同一业务事件可能重复累计 |
| A09 | `Tracker::analyze()` 混用 inner stats 与另一时刻的 events | `src/tracker.rs` | 单份报告内的数值可能来自不同窗口 |
| A10 | `export_all_json()` 把当前存活集合转换成快照 | `src/render_engine/export.rs`、`src/snapshot/types.rs` | 历史累计量和峰值不能由存活集合恢复 |
| A11 | EvidenceLevel 根据 generation 是否大于零决定 | `report_builder.rs` | 分配身份不能证明类型、布局或风险结论真实 |
| A12 | 分配间隔被解释为 scheduler lag，线程分布被解释为 migration | `src/render_engine/dashboard/renderer/context.rs`、模板 | 数值有来源，但指标含义错误 |
| A13 | 地址复用率条件为 `generation_id > 0`，首次分配 generation 为 1 | `context.rs`、`event_reconstructor.rs` | 首次分配也被算作复用 |
| A14 | EventStore clear 期间直接跳过 record，没有报告对应缺口 | `src/event_store/store.rs` | 缺失事件可能被当作完整轨迹 |
| A15 | Passport 以裸地址为键；shutdown 检测会修改 Passport 状态，且被报告构建调用 | `src/analysis/memory_passport_tracker.rs`、`context.rs` | 地址复用污染与查询副作用；运行中导出可能套用退出判定 |
| A16 | 准确性测试把独立同内容数组作为 Clone 正例，owner 未检测出来也可能通过 | `tests/relation_inference_integration.rs` | 测试通过不能证明语义判断正确 |
| A17 | UTI 选择最高分，没有低分/近似平票拒判；分数以百分比显示 | `src/analysis/unsafe_inference/engine.rs` | 低证据结果过度确定；分数被误读为概率 |
| A18 | `Trackable` 返回 `Vec<T>` 等占位类型名；derive 生成的接口与当前 trait 存在差异 | `src/lib.rs`、`memscope-derive/src/lib.rs` | 已知类型信息丢失，编译期增强需要实际编译用例验证 |
| A19 | `GlobalTracker::track_as()` 按类型名是否含 Mutex/Cell 自动产生 borrow 记录 | `src/capture/backends/global_tracking.rs` | 跟踪变量不等于发生了一次 Rust 借用 |

注意：`src/core/tracker/memory_tracker.rs` 主要是重导出，实际核心记账在 `src/capture/backends/core_tracker.rs`。`memscope-report/` 是生成结果目录，不是独立 Rust crate。

## 3. 目标架构与职责

```text
AllocatorHook / 显式操作插桩 / 普通变量观测 / FFI / RuntimeHook
                              │
                              ▼
                 CaptureSession + RecordingSink
          来源标记、入队顺序、身份规范化、丢失计数
                              │
                              ▼
                   EventStore<RecordedEvent>
            已提交的事件前缀 + CaptureQuality + 边界
                              │ snapshot_at(cut)
                              ▼
              snapshot::reducer::AllocationReducer
          分配账本 / 地址绑定 / 句柄与观测 / 线程和任务归属
                              │
                              ▼
                        AnalysisSnapshot
                  同一个不可变分析输入与 cutoff
                 ┌────────────┼────────────┐
                 ▼            ▼            ▼
             MemoryView    Analyzer     Passport/关系投影
                 └────────────┼────────────┘
                              ▼
                       ReportSnapshot
                  ┌───────────┼───────────┐
                  ▼           ▼           ▼
                HTML      磁盘 JSON    snapshot_json()
```

### 3.1 核心规则

1. **采集层记录操作或观测，重建层改变账本，分析层产出结论，展示层只映射字段。**
2. `render_engine` 不拥有生命周期真相；`Tracker` 不再依赖 Dashboard 的重建函数。
3. 一个逻辑操作只有一个记账来源。其他后端提供补充信息，不能再发一份等价 Allocate。
4. 系统 RSS/CPU 是独立测量源，不强行与跟踪到的请求字节数相等。
5. 最终复用 `snapshot/` 作为唯一重建模块，不新增另一套并行“准确性引擎”。

## 4. 数据结构设计

本节代码是字段与语义草图，省略 derive、错误类型及部分辅助结构。实现时优先增加 V2 内部模型和适配器，避免在第一步给所有旧结构体同时增加字段。

### 4.1 身份：Session、Allocation、Handle、Observation

新增 `src/core/types/identity.rs`，由 `src/core/types/mod.rs` 重导出：

```rust
pub struct SessionId(pub u128);
pub struct AllocationId(pub u64);
pub struct HandleId(pub u64);
pub struct ObservationId(pub u64);
pub struct EventSeq(pub u64);

pub struct AddressBinding {
    pub address: usize,
    pub generation: u64,
}

pub enum EntityRef {
    Allocation(AllocationId),
    Handle(HandleId),
    Observation(ObservationId),
}
```

约定：

- `SessionId` 在一次采集会话创建时生成；`Tracker::clone()` 共享同一个 SessionId 和起始时钟。
- AllocationId、HandleId、ObservationId 在会话内单调分配，不回收。它们不等于地址。
- 外部关联主键是 `(session_id, entity_kind, entity_id)`。JSON 可以表达为 `session:allocation:42`。
- AllocationId 表示一次可追踪的逻辑分配，成功 realloc 后继续使用该 ID；地址变化写入绑定历史。
- generation 表示同一地址建立新分配绑定的次序，从 1 开始。重复观测、clone、原地 realloc 不增加 generation。
- 搬迁 realloc 关闭旧地址绑定，并在新地址建立绑定；旧地址后续被另一分配使用时，使用新的 AllocationId 与对应 generation。
- 成功 realloc 即使地址相同，旧指针的有效性也不能由“ID 没变”推导。ID 只用于分析身份，布局版本需要更新。
- 计数器溢出返回诊断并停止创建新身份，不能回绕后复用 ID。
- `stack_ptr` 改为“句柄地址采样值”的语义。Arc 可以存放在堆中，移动后地址也会改变，不能将其作为持久句柄 ID。
- 普通 `track!` 无法证明两次调用是同一变量实例。没有显式注册 token 时，两次调用是两个 Observation，通过证据关联，不能硬编码合并为同一 Handle。

#### 地址与指向数据地址必须分开

`Arc::as_ptr()` / `Rc::as_ptr()` 给出 T 的数据地址，不保证等于 allocator 返回的整块分配基址。不能通过减去“两个 usize”推导头部。

因此：

- allocator 事件保存 `allocation_base` 与 `requested_size`；
- 类型观测保存 `data_address`、`payload_size`；
- 两者通过当前有效地址区间关联，保留 `offset` 与关联证据；
- 无可靠关联时，保持 Observation，不生成伪造的 allocator 分配。

### 4.2 证据与数据来源

将 Dashboard 私有的证据概念下沉到 `src/core/types/evidence.rs`。现有路径通过 `pub use` 过渡。

```rust
pub enum EvidenceLevel {
    Observed,
    Inferred,
    Heuristic,
    Unknown,
}

pub enum DataSource {
    AllocatorHook,
    TypedObservation,
    ExplicitInstrumentation,
    RuntimeHook,
    ForeignInstrumentation,
    DerivedFromEvents,
    ByteHeuristic,
    LegacyImport,
    Unavailable,
}

pub struct Evidence {
    pub level: EvidenceLevel,
    pub source: DataSource,
    pub event_seq: Option<EventSeq>,
    pub rule_id: Option<String>,
    pub reason: Option<String>,
}

pub struct Fact<T> {
    pub value: Option<T>,
    pub evidence: Evidence,
}
```

使用规则：

- `requested_size` 可由 allocator 观测获得，但这不能把同一行的 `type_name` 自动提升为 Observed。
- 真实 `type_name::<T>()` 与安全容器 API 的返回值可标记 TypedObservation。
- 扫描得到的类型、所有权、clone 候选分别附带自己的规则与证据。
- Unknown 使用 `value: None`，不能以 `0`、空字符串或 `false` 代替未采集。
- 由完整账本求和是有公式的派生值，来源为 DerivedFromEvents；它不是类型推断。
- `RiskConfidence` 用于某个诊断结论，不作为整条 allocation 的统一“真实性”。
- 热路径事件只保存来源枚举与必要标量；`Fact<T>`、说明字符串和诊断说明在冷路径构建。

### 4.3 事件模型

新增 `src/event_store/recorded.rs`：

```rust
pub struct EventHeader {
    pub session_id: SessionId,
    pub seq: EventSeq,
    pub monotonic_ns: u64,
    pub thread_id: u64,
    pub task_id: Option<u64>,
    pub source: DataSource,
}

pub struct RecordedEvent {
    pub header: EventHeader,
    pub payload: EventPayload,
}

pub enum EventPayload {
    AllocationCreated {
        id: AllocationId,
        binding: AddressBinding,
        requested_size: usize,
        align: Option<usize>,
        domain: AllocationDomain,
    },
    AllocationFreed {
        id: AllocationId,
        binding: AddressBinding,
        reported_size: Option<usize>,
        cause: FreeCause,
    },
    AllocationReallocated {
        id: AllocationId,
        old_binding: AddressBinding,
        old_size: usize,
        outcome: ReallocOutcome,
    },
    ObjectObserved {
        id: ObservationId,
        handle_id: Option<HandleId>,
        metadata: ObjectMetadata,
        pointee: Option<PointeeObservation>,
    },
    HandleRegistered {
        id: HandleId,
        kind: HandleKind,
    },
    HandleUnregistered {
        id: HandleId,
    },
    CloneObserved {
        source: EntityRef,
        target: EntityRef,
        kind: CloneKind,
    },
    FfiTransition {
        allocation_id: AllocationId,
        transition: FfiTransitionKind,
    },
    TaskLifecycle {
        task_id: u64,
        parent_id: Option<u64>,
        transition: TaskTransition,
    },
    SessionEnded {
        reason: CaptureEndReason,
    },
}

pub enum ReallocOutcome {
    Succeeded {
        new_binding: AddressBinding,
        new_size: usize,
    },
    Failed,
}
```

辅助类型放置与含义：

| 类型 | 字段或变体 | 建议位置 |
| --- | --- | --- |
| `AllocationDomain` | RustAllocator / ForeignAllocator / ExplicitManaged | `core/types/identity.rs` |
| `FreeCause` | RustAllocator / ForeignFree / ExplicitRelease | `event_store/recorded.rs` |
| `ObjectMetadata` | 真实类型名、变量名、源码结构体、对象角色、类型布局观测 | `core/types/observation.rs`（新增） |
| `PointeeObservation` | data_address、payload_size、可选已解析 allocation_id、offset、关联证据 | `core/types/observation.rs` |
| `HandleKind` | Unique / Arc / Rc / Weak / Container | `core/types/observation.rs` |
| `CloneKind` | DeepCopy / SharedArc / SharedRc | `event_store/recorded.rs` |
| `FfiTransitionKind` | Handover / Reclaimed / ForeignFreed / BoundaryAccess | `event_store/recorded.rs` |
| `TaskTransition` | Started / Completed / Cancelled；任务名称等附加元数据可另附 | `event_store/recorded.rs` |
| `CaptureEndReason` | ExplicitStop / GuardDrop / Panic / Signal / ProcessExit | `event_store/recorded.rs` |

约束：

1. `MemoryEvent::allocate()` 等旧构造器改为纯构造，不再调用 TaskIdRegistry。
2. `ObjectObserved` 不改变真实 allocator 的 bytes/count。
3. `HandleUnregistered` 表示不再跟踪该句柄，不等于对象 drop，更不等于堆块被释放。
4. CloneObserved 必须由真实 clone 操作的插桩或明确契约产生；内容相似仅生成分析候选。
5. `track!` 不隐式产生 Borrow 事件。借用开始/结束以后通过显式 BorrowId 契约扩展，现阶段未采集就标 Unknown。
6. 未匹配释放保存为诊断原始记录，不能为了满足 enum 而创造 AllocationId 或删除当前同地址的新对象。
7. `ExplicitInstrumentation` 必须规定“成功操作后报告”的契约；普通用户描述性标记不能自动作为确认释放。

### 4.4 时钟、序号与快照边界

- 每个会话保存 `wall_clock_start`，只用于显示现实时间。
- 时长与排序使用会话单调时钟 `monotonic_ns`，消除系统时钟回拨影响。
- 对并发事件，规范化后的 `EventSeq` 定义重放顺序；不能仅按 timestamp 排序。
- 序号在事件提交到 journal 时确认。单独使用 `fetch_add()` 后再异步 enqueue，不能证明此前序号都已经可读。
- `SnapshotCut` 必须代表连续、已经提交的事件前缀。

新增快照元数据：

```rust
pub struct SnapshotCut {
    pub session_id: SessionId,
    pub through_seq: EventSeq,
    pub captured_at_ns: u64,
    pub reason: SnapshotReason,
}

pub enum SnapshotReason {
    OnDemand,
    Periodic,
    CaptureEnded(CaptureEndReason),
    Emergency,
}
```

内存事件、Passport 交接和任务事件应共享序列或有明确投影水位。系统资源采样保留自身采样时间与延迟，不伪装为同时完成的原子系统快照。

### 4.5 分配状态与泄漏评估分离

新增 `src/snapshot/state.rs`：

```rust
pub enum AllocationState {
    Live,
    Freed { at_seq: EventSeq, at_ns: u64, cause: FreeCause },
    Unknown { since_seq: EventSeq, reason: String },
}

pub enum LeakStatus {
    NotAssessed,
    NoEvidence,
    Suspected,
    Confirmed,
}

pub struct LeakAssessment {
    pub status: LeakStatus,
    pub evidence: Evidence,
    pub rule_id: Option<String>,
}

pub struct AllocationRecord {
    pub id: AllocationId,
    pub domain: AllocationDomain,
    pub current_binding: AddressBinding,
    pub requested_size: usize,
    pub align: Option<usize>,
    pub layout_revision: u64,
    pub allocated_at_ns: Option<u64>,
    pub first_seen_seq: EventSeq,
    pub allocating_thread: u64,
    pub allocating_task: Option<u64>,
    pub state: AllocationState,
    pub type_name: Fact<String>,
}
```

说明：

- AllocationState 记录物理分配生命周期；LeakAssessment 是在指定 cut 上计算出的分析结论。
- 结束会话时没有释放的记录仍然保留，结束原因不替代释放事件。
- 普通变量观测只证明“那个时刻可观察到对象”，保存在 Observation 中，不赋予持续 Live 的分配保证。
- 当前窗口正常存活：NoEvidence；事件存在缺口且无法评估：NotAssessed/Unknown。
- 最终快照仍存活、长期未释放、已交给 FFI 等只能在明确规则下产生 Suspected。
- Confirmed 必须有可以独立说明的充分证据与完整性条件。基础版本可以没有任何规则生成 Confirmed。
- `mem::forget`、最后一个被跟踪句柄离开、ForeignCustody 都不是默认 Confirmed 依据。
- 活对象展示“观测年龄”，已释放对象展示完整 lifetime；缺少真实起始时间时不伪造完整 lifetime。

### 4.6 重建状态与索引

```rust
pub struct ReconstructionState {
    pub allocations: HashMap<AllocationId, AllocationRecord>,
    pub current_by_address: HashMap<usize, AllocationId>,
    pub address_history: HashMap<usize, Vec<BindingInterval>>,
    pub handles: HashMap<HandleId, HandleRecord>,
    pub observations: HashMap<ObservationId, ObjectObservation>,
    pub stats: HashMap<AllocationDomain, LedgerStats>,
    pub thread_stats: HashMap<(AllocationDomain, u64), AttributionStats>,
    pub task_stats: HashMap<(AllocationDomain, Option<u64>), AttributionStats>,
    pub diagnostics: Vec<ReconstructionDiagnostic>,
}

pub struct LedgerStats {
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub reallocation_count: u64,
    pub failed_reallocation_count: u64,
    pub allocated_bytes: u64,
    pub deallocated_bytes: u64,
    pub current_bytes: u64,
    pub peak_bytes: u64,
}

pub struct AttributionStats {
    pub origin_live_bytes: u64,
    pub origin_peak_bytes: u64,
    pub alloc_operations: u64,
    pub free_operations: u64,
    pub realloc_operations: u64,
    pub allocated_by_actor_bytes: u64,
    pub freed_by_actor_bytes: u64,
}

pub struct AnalysisSnapshot {
    pub cut: SnapshotCut,
    pub quality: CaptureQuality,
    pub state: ReconstructionState,
    pub events: Arc<[RecordedEvent]>,
}
```

- `BindingInterval`：allocation_id、binding、start_seq、end_seq、size；支持按某一事件时刻查询地址归属。
- `HandleRecord`：handle_id、kind、注册区间、目标关联历史、最近一次地址观测；句柄迁移不改变 ID。
- `ObjectObservation`：observation_id、seq、metadata、pointee；与真实分配账本分开保存。
- `ReconstructionDiagnostic`：code、seq、关联 entity、expected/actual、对完整性的影响。
- allocations 保留已释放历史；current_by_address 只维护当前有效绑定。
- 活跃集合是对状态的投影，不能只保留 active 而丢失历史后再计算累计量与峰值。
- 元数据节点用 EntityRef 表示，不再制造 `VIRTUAL_PTR_BASE + index` 冒充内存地址。
- LedgerStats 是对已接受事件的内部账本值；导出阶段依据 CaptureQuality 包装为 `Fact<u64>`。缺口期间的内部差值不能直接冒充真实 current/peak。
- Rust/Foreign/ExplicitManaged 分域记账；跨域总和必须确认没有同一物理分配重复登记，不能直接把各来源 counter 相加。
- AnalysisSnapshot 创建后不可变。在线状态可继续变化，但不能修改已经发布的 Arc 快照；checkpoint 模式的 events 只保留可重放后缀并附 checkpoint 元数据。

### 4.7 数据完整性

新增 `src/event_store/quality.rs`：

```rust
pub enum CaptureMode {
    ObservationsOnly,
    ExplicitLifecycle,
    AllocatorEvents,
    Hybrid,
}

pub struct CaptureQuality {
    pub mode: CaptureMode,
    pub started_before_workload: bool,
    pub sampling_probability: Option<f64>,
    pub accepted_events: u64,
    pub sampled_out_events: u64,
    pub dropped_buffer_full: u64,
    pub dropped_contention: u64,
    pub dropped_during_reset: u64,
    pub unmatched_deallocations: u64,
    pub conflicting_events: u64,
    pub history_truncated: bool,
    pub legacy_semantics_ambiguous: bool,
}
```

实际实现按来源保存相同计数，并区分“主动排除 profiler 自身事件”和“业务事件丢失”。不能用 excluded_internal 数量代表丢失率。

**采集采样与页面截断是两回事：**

- CaptureQuality 描述上游是否看到了事件。
- 原有 SamplingMetadata 改为 `DisplaySelection`，描述全量分析结果向页面展示多少条。
- 页面展示 100/100 条不代表真实程序的采集完整率是 100%。
- 日志缺口不能静默补出释放；受影响的生命周期、泄漏与峰值标记不完整。
- 精确计数模式保持全部生命周期事件，只采样调用栈/字节内容等高开销附属信息。
- 若提供分配采样模式，按分配身份决定是否保留，并保持该分配的 free/realloc 成对；输出明确的样本统计，估计值另列。

## 5. 唯一重建器：事件如何改变状态

新增 `src/snapshot/reducer.rs`，提供：

```rust
impl AllocationReducer {
    pub fn apply(&mut self, event: &RecordedEvent) -> Result<(), ReconstructionError>;
    pub fn finish(self, cut: SnapshotCut, quality: CaptureQuality) -> AnalysisSnapshot;
}
```

`apply()` 是可重放的纯状态转换：不访问全局 tracker、不读取当前时间、不扫描进程内存、不更新外部 TaskIdRegistry。

### 5.1 转换规则

| 输入 | 状态变化 | 统计变化 | 异常处理 |
| --- | --- | --- | --- |
| AllocationCreated | 创建 ID 和有效地址绑定 | allocation_count +1，current +size，allocated_bytes +size | 同地址已有 Live：冲突诊断，不无声覆盖 |
| ObjectObserved | 增加一次观测或丰富已验证的目标元数据 | 仅 observation_count 增加 | 无法确定目标时保留未关联观测 |
| CloneObserved(DeepCopy) | 记录 source → target 的操作关系 | 不再增加 bytes；目标分配应由 allocator/显式 Allocate 提供 | 未观测目标分配时只建立实体关系 |
| CloneObserved(SharedArc/Rc) | 两个句柄关联同一分配 | bytes/count 不变 | 仅地址相同但无 clone 事件时只推导 Shares 候选 |
| HandleUnregistered | 结束跟踪注册 | 分配 bytes/count 不变 | 重复注销记录诊断或幂等 no-op，规则固定 |
| AllocationFreed | 按 ID+binding 关闭分配 | current -已记录 size，deallocation_count +1 | ID/generation 不匹配不能释放新一代 |
| Realloc 成功且原地 | 同 ID，更新 size、layout_revision | realloc_count +1，current += new-old | 核对 old_size，保留类型与来源元数据 |
| Realloc 成功且搬迁 | 同 ID，关闭旧绑定，建立新绑定 | 同上 | 新地址冲突则诊断，不能静默覆盖 |
| Realloc 失败 | 原 ID、地址和 size 不变 | failed_realloc_count +1 | 不生成 Free，不清空旧块 |
| FFI Handover/Reclaimed | 改变 custody | bytes/count 不变 | 交接不等于释放 |
| FFI ForeignFreed | 经规范化后产生唯一释放转换 | 只扣一次 bytes | 与显式 free 重复时根据操作身份去重 |
| SessionEnded | 记录观测截止 | 不改变存活 bytes | 不生成批量 Deallocate |

重复事件与重复操作要区分：相同 `(session, seq)` 的重放导入可以去重；不同 seq 的两次业务操作不能仅因地址和类型相同就丢弃。

### 5.2 统一统计公式

对同一 domain、同一完整已提交前缀：

```text
allocated_bytes = Σ successful_alloc.size + Σ successful_realloc.new_size
deallocated_bytes = Σ matched_free.recorded_size + Σ successful_realloc.old_size
current_bytes = allocated_bytes - deallocated_bytes
current_bytes = Σ live_allocation.requested_size
peak_bytes = max(current_bytes after each committed state transition)
```

- realloc 是一个原子账本转换，不因代码先加后减而制造虚假峰值。
- 上述 peak 是“观测到的请求字节数峰值”，不是 RSS 峰值，也不包含 allocator 内部搬迁暂存或碎片。
- allocation_count 只计算初始分配；reallocation_count 单列；成功 realloc 不伪增初始分配次数。
- Metadata、Handle、Container、Value、clone 操作本身不进入堆字节总和。
- 原始日志请求的释放大小不一致时，以身份对应的账本大小扣减，并记录 SizeMismatch。
- 未匹配释放不扣减 current；发现不守恒必须产出诊断，不能只靠 saturating arithmetic 把问题隐藏。
- 内部累加建议使用 checked `u64`。溢出使相应指标不可用，不能 wrap 或静默饱和后继续标精确。
- 旧 `MemorySnapshot::from_allocation_infos()` 只能构造“当前集合视图”；缺少历史的累计/peak 为 Unknown，不能赋零表示已测得零。

### 5.3 跨线程、跨任务归属

区分两个维度：

1. **来源归属**：哪一个 thread/task 最初分配了当前存活字节。
2. **操作活动**：哪一个 thread/task 执行了 alloc/free/realloc。

规则：

- AllocationRecord 固定保存 allocating_thread / allocating_task。
- B 释放 A 创建的块：减少 A 的 live_bytes，增加 B 的 freed_by_this_actor_bytes。
- B 对 A 的块 realloc：新大小仍归属原创建者，B 的 realloc 活动计数增加；所有权迁移另有显式事件才改变相应所有权视图。
- 缺少 task_id 计入 `None` 桶，不能丢掉；所有任务来源桶求和等于同 domain 的全局 live_bytes。
- task 完成不释放其分配。内存可以由返回值、Arc 或其他任务继续持有。
- 来源归属不是“唯一拥有者”，页面命名必须体现区别。

### 5.4 最小重放示例

```text
seq 1: Alloc(A, p, G1, 64, thread=1, task=10)
seq 2: Realloc(A, p/G1 -> q/G1, 128, thread=2)
seq 3: Alloc(B, p, G2, 32, thread=1)
seq 4: Free(A, q/G1, thread=3)
seq 5: EndCapture
```

期望：

- A 为 Freed，B 仍 Live；B 不继承 A 的任何关系。
- allocations=2，reallocations=1，deallocations=1。
- allocated_bytes=224，deallocated_bytes=192，current_bytes=32，peak_bytes=160。
- thread 1 来源 live_bytes=32；thread 3 的释放活动为 128 bytes。
- EndCapture 不把 B 清零，默认也不将 B 标 Confirmed leak。

## 6. 采集层如何接入

### 6.1 RecordingSink 与会话共享

新增 `src/capture/recording.rs`，作为采集侧公共入口：

- `CaptureSession` 拥有会话时钟、source 配置、入队入口、journal 和状态投影。
- `Tracker` 持有 `Arc<CaptureSession>`；clone 不重置起始时间。
- `GlobalTracker`、AsyncTracker 与 PassportTracker 注入同一个 session/sink。
- 核心 tracker 的实时计数可以作为缓存保留，但必须由唯一事件入口驱动，不再各自独立加减。
- 稳定报告从指定 cut 的归约结果获取；实时缓存不能混入该报告。
- 手工事件构造、克隆或反序列化都没有记账副作用。

### 6.2 普通 track! 与显式注册

修改 `src/tracker.rs::track_as/track_inner`：

1. 获取 `ObjectMetadata`：类型、源码、TrackKind、容器可验证属性。
2. 生成 ObjectObserved，而不是一律生成 Allocate。
3. 在事件发生位置尝试关联当前有效分配区间；关联不足时保留未关联对象。
4. HeapOwner、StackOwner/SharedHandle、Container、Value 都是对象角色，不直接决定是否增加 allocator bytes。
5. 删除按类型名推导 Borrow 的自动副作用。

新增 `src/tracking/registration.rs`，提供需要稳定句柄身份的显式接口。接口草图：

```rust
let registration = tracker.register_handle(HandleKind::Arc, "shared");
tracker.observe_registered(&registration, &shared, source_location);
// registration 内的 HandleId 在变量移动、再次观测时仍稳定。
```

注册 token 的 drop 只产生 HandleUnregistered。若未来增加 `TrackedArc<T>` 等 owning wrapper，其 clone/drop 记录的是句柄操作，不能凭 `strong_count()==1` 推断底层块此刻已释放：Weak 和并发操作都会影响这一结论。物理 free 仍以 allocator 或明确的释放插桩为准。

没有 allocator hook 时，默认报告展示“已观测对象、类型、数据区大小、最后观测时间”。真实 allocation_total、真实 peak 和完整 lifetime 标记未采集。该行为变化需要在迁移说明中给出示例。

### 6.3 类型与布局采集

修改 `src/lib.rs::Trackable` 及具体实现：

- `get_type_name()` 使用 `std::any::type_name::<Self>()`，保留具体泛型信息。
- 以有默认实现的附加方法返回 `TypeObservation`，减少自定义 Trackable 的迁移成本。
- Vec/String 记录 len、capacity、element_size、object_size；有效内容长度与容量分开。
- Box 记录 `size_of_val` / `align_of_val`，嵌套对象的缓冲区另关联，不能重复加总。
- Arc/Rc 记录 data_address、payload_size、引用计数采样值；控制块大小与布局未知时保留 Unknown。
- HashMap/BTreeMap 的内部总堆大小仍可能是估算，不能以 `size_of` 表示整个递归堆占用。
- 空 Vec/String、ZST、`Vec<()>`、Box<ZST> 的 dangling/sentinel 地址不创建真实分配。Arc<ZST> 仍可能有真实控制块，需依赖 allocator 事件关联。
- TypeLayoutSnapshot 消费实际类型观测；`infer_layout_snapshot()` 只作为显式标记的旧数据回退。
- 派生宏以当前 trait 为准修正生成代码，增加泛型 struct、enum、嵌套容器的外部编译用例。

### 6.4 allocator hook：完整捕获 alloc/free/realloc

主要修改 `src/core/allocator.rs`，新增 `src/capture/allocator_events.rs`：

1. 显式实现 alloc、alloc_zeroed、dealloc、realloc；调用 `System` 后按结果记录。
2. 分配返回 null 不创建 Allocation；realloc 失败保留原块。
3. 记录 old_ptr、new_ptr、old_size、new_size、align；不能只用旧 `MemoryEvent::reallocate(ptr, ...)` 表达搬迁。
4. 该 hot path 只写固定大小原始事件，不创建 String/Vec、不符号化、不做 JSON、不初始化完整 tracker。
5. 启动时在 tracking suppression 下预分配队列与 session；接收线程的内部内存不再次进入业务统计。
6. 将手动切换 TLS bool 改为可恢复的作用域 guard，避免提前返回后持续禁用跟踪。TLS 已销毁时使用可失败访问并记录可用的降级信息。
7. profiler 的重入排除不应吞掉正常业务路径的事件；需要明确内部对象的生命周期与排除策略。

#### 第一版并发方案：有序参考实现

先实现容易验证的参考路径，再做无锁优化：

- 会话级 ingress gate 为原始 allocator 操作建立顺序；参考 hook 在同一 gate 下完成 System 操作与固定大小事件发布。
- realloc 的 System 调用也在该区间内，避免“旧地址已被别的线程复用，但 realloc 事件尚未发布”的关联竞态。
- 使用预分配有界队列，队列满时记录丢失计数并让实际内存操作继续；不能改变用户分配操作的结果。
- suppression 必须在取得 gate 之前生效；消费者及其内部内存分配不经过该 gate，不持有它执行分析/导出。
- 显式观测经同一顺序入口提交，消费者按入队顺序分配 AllocationId、generation 和最终 EventSeq。
- normal API 可以在冷路径预先构建元数据，再进行短临界区提交；禁止在持有 journal/分析锁时等待 ingress gate。
- 锁顺序与重入测试是该 PR 的验收项。不能只把 EventStore::record 直接塞进 GlobalAlloc 回调。

该 gate 会影响并发吞吐，先在 benchmarks 中量化。后续若改为分线程队列，必须引入可验证的操作 ticket/发布水位，并用同一真值轨迹证明地址复用、realloc 顺序和快照 cut 等价，不能仅按时间戳合并。

### 6.5 EventStore 提交、快照与 clear

修改 `src/event_store/store.rs`：

- 消费者规范化原始事件后，在 journal 提交区更新连续 seq 与相关质量计数。
- 原始入口另带单调 `IngressTicket`，每次尝试记录都占用 ticket。queue overflow 等失败留下计数与区间标记；消费者在处理下一条事件前看到缺口，避免把缺失分配后的裸地址 free 错配到旧身份。IngressTicket 用于发现采集缺口，EventSeq 用于规范事件的已提交顺序，两者不混用。
- 规范化器只维护分配身份/地址解析所需的最小表；正式 bytes、状态与分析规则由同一 AllocationReducer 驱动。身份变更与规范事件提交必须处于同一提交协议，禁止“先更新身份表但没有对应 journal 事件”的半提交状态。
- `snapshot_at()` 先提交/等待一个 ingress barrier，再取得该 barrier 对应的已提交前缀及质量副本。
- 快照不能返回“已看到更高 seq，但更低 seq 尚未入库”的集合。
- 默认 API 可以同步 drain 到 barrier；高吞吐模式使用消费者线程，二者共享规范化与归约规则。
- 快照导出失败不改写原事件或 allocation state。
- 正常采集不使用当前 `clear()` 丢弃活跃历史。reset 明确结束旧 session、处理写入者并创建新 session。
- 若为长服务压缩历史，保存 checkpoint（存活账本、累计值、generation counters、through_seq），再截断 checkpoint 之前事件。
- 历史详情被裁剪要标注 retention，但有正确 checkpoint 时当前账本不必因此变成未知；不能将“有采集缺口”与“有意历史压缩”等同。
- panic/signal 路径不能无界等待正常路径持有的 ingress/journal 锁。超时后导出最后可用 cut，并标 Emergency/不完整。

## 7. 关联模块如何修改

### 7.1 Tracker、Snapshot、MemoryView、Timeline

| 文件 | 具体修改 |
| --- | --- |
| `src/tracker.rs` | 删除 drop 批量释放；共享 session/start_time；track 改为观测入口；analyze 只读取一个 AnalysisSnapshot |
| `src/capture/backends/core_tracker.rs` | 相同地址重复 track 不无声覆盖且累加；逐步改为事件驱动缓存；区分 allocator 操作与附加观测 |
| `src/snapshot/builder.rs` | 成为 AllocationReducer 的兼容入口，移除独立 switch 记账逻辑 |
| `src/snapshot/engine.rs` | 返回带 SnapshotCut、CaptureQuality 的快照；通过 snapshot_at 获取边界 |
| `src/snapshot/types.rs` | 接入 AllocationId、状态、历史与来源统计；旧裸地址 active map 变成兼容投影 |
| `src/view/memory_view.rs` | 持有同一不可变 AnalysisSnapshot；增加按 allocation_id 查找；按 ptr 查找必须注明当前 cut |
| `src/view/stats.rs` | 从统一 LedgerStats 映射，不另行估算 peak/total |
| `src/timeline/query.rs` | 通过 reducer 前缀或 checkpoint 回放，保留身份与 realloc 绑定历史 |
| `src/render_engine/dashboard/renderer/event_reconstructor.rs` | 迁移为 snapshot → DTO 适配；调用者切换后移除其中的生命周期 match |

状态转换与统计规则以 §5 为准；本表只列调用入口迁移。PR-03 完成后，生产代码只保留一个负责分配状态转换的 reducer。

### 7.2 AsyncTracker 与 TaskIdRegistry

修改：

- `src/task_registry.rs`
- `src/capture/backends/global_tracking.rs`
- `src/capture/backends/async_tracker.rs`
- `src/capture/backends/async_types.rs`
- `src/capture/backends/task_profile.rs`

执行步骤：

1. 删除 MemoryEvent 构造与 GlobalTracker 双重累加任务内存的路径。
2. TaskIdRegistry 管理任务身份、父子关系、运行状态；memory counters 由已提交事件投影更新，或在报告时直接使用 task_stats。
3. 把 `memory_usage` 拆清为累计 allocated_bytes 与来源 live_bytes；兼容字段指向哪个量要固定写入 schema。
4. 统一事件使用的 task ID；当前存在不同宽度/来源的任务标识时，建立一对一映射，不能强制截断 u128 到 u64。
5. `TrackedFuture::poll` 在每次 poll 入口设置任务上下文，并通过 guard 在 Pending、Ready、panic 路径恢复之前的上下文。
6. 同步 task scope 也恢复父任务，不能退出内层 scope 后简单清空 TLS。
7. task cancellation/abort 记录任务状态，但不伪造分配释放。
8. task_id 必须沿 RecordedEvent → AllocationRecord → DTO → TaskGraph 传递，不能在中间结构转换时丢失。

### 7.3 Passport 与 FFI

主要修改 `src/analysis/memory_passport_tracker.rs`；同步检查 `src/analysis/unsafe_ffi_tracker.rs`、`src/capture/backends/unsafe_tracking.rs` 的相同导出路径。

数据结构变更：

- Passport 增加 allocation_id 与 session_id，地址作为显示字段/绑定历史。
- 存储主键从 `HashMap<usize, MemoryPassport>` 改为 `HashMap<AllocationId, MemoryPassport>`，独立创建的 tracker 也必须关联到明确 session。
- PassportEvent 增加共享 event_seq，时间统一到会话单调时钟。
- custody 与 physical state 分离：Rust / Foreign / Unknown 不表示释放与否。
- 原 `status_at_shutdown` 在 V2 中改为某个 cut 的状态投影；旧字段仅兼容输出。

接口修改：

1. create_passport 优先接受 AllocationId；旧 ptr API 在调用时解析当前绑定，无法唯一解析时返回结构化错误。
2. 只有 Foreign allocation 的真实创建插桩才能增加 Foreign domain 字节；为 Rust 已存在的块创建 Passport 不再增加一次分配。
3. handover/reclaim/free 写入统一 sink，Passport 成为相关事件的投影。
4. `detect_leaks_at_shutdown()` 拆出只读 `assess_leaks(snapshot, policy)`；运行中导出不修改 tracker 的最终状态。
5. boundary free 与 allocator free 的重复通知需要关联 operation/entity 并记录来源，不能按地址盲目扣两次。
6. `max_events_per_passport` 裁剪时保留状态/checkpoint 及 history_truncated 标志，不能删除了最初事件后仍宣称完整轨迹。

### 7.4 关系图、所有权与检测器

修改模块：

- `src/analysis/relation_inference/{mod,graph_builder,range_map,pointer_scan,clone_detector,shared_detector,slice_detector,container_detector}.rs`
- `src/analysis/ownership_graph.rs`
- `src/analysis/circular_reference.rs`
- `src/analyzer/{core,graph,report}.rs`
- `src/analysis/detectors/{leak_detector,double_free_detector,uaf_detector,data_race_detector}.rs`

关系边 V2 草图：

```rust
pub struct RelationEdgeV2 {
    pub from: EntityRef,
    pub to: EntityRef,
    pub relation: Relation,
    pub evidence: Evidence,
    pub valid_from_seq: EventSeq,
    pub valid_until_seq: Option<EventSeq>,
}
```

规则：

- 向量下标只能是某次图算法的内部索引，不能成为持久/跨报告实体 ID。
- RangeMap 从该 cut 的有效 allocation ranges 构建；检测历史事件时使用对应时间区间的绑定。
- 指针命中另一对象只证明 PointsTo 候选，不能自动证明 Owns；新增/明确 PointsTo 关系语义。
- 两对象字节相等产生 ContentSimilar 候选，不自动证明发生 Clone；真实 CloneObserved 单独保留操作证据。
- 普通 Arc 共享同目标可产生 Shares；有真实 clone 事件才能生成有方向的 ArcClone/RcClone。
- 循环引用分析只将满足所有权语义的强引用边纳入泄漏规则；Weak、普通指针、借用和内容相似边不能同等处理。
- LeakDetector 消费 LeakAssessment，不再从“无 dealloc 时间”重复判定。
- DoubleFree 要区分“同一 ID 已释放后再次释放”“旧一代的迟到事件”“采集前分配/中途丢事件导致的 unmatched”。
- 没有内存访问事件时，分配活动不能证明 UAF 或传统 data race；相关诊断保留启发式名称和证据条件。
- Borrow 统计未采集时是 None，而不是固定 `immutable_borrows=0`、`mutable_borrows=0`。

### 7.5 HeapScanner 与内容快照

修改 `src/analysis/heap_scanner/reader.rs`、`src/analysis/unsafe_inference/memory_view.rs`，并更新两种 InferenceRecord 的适配。

- 内容数据增加 allocation_id/observation_id、captured_at_seq、captured_len、来源与完整性。
- UTI 与关系推断消费已经持有的字节副本，不在历史报告阶段重新读取同一个裸地址。
- 只有映射可读不代表对象仍活着，也不代表没有并发写入。检查 ValidRegions 或持有 allocator 锁不足以保证一致的对象内容。
- 优先提供从有效 `&[u8]` / `&str` 等受控借用复制内容的入口；同步由调用方/类型安全 API 保证。
- Vec 只复制已初始化的 len 对应内容，不能把 capacity 范围全部当作已初始化字节。
- 普通 `&T` 不能默认转成任意字节读取，需考虑 padding、内部可变性与类型约束。
- 无法取得一致内容时返回 None 和原因；类型/关系引擎走元数据路径或拒判。
- 实时扫描与纯离线字节推断分开测试，平台不能读取内容时，不应整组跳过语义测试。

## 8. 生命周期与导出如何修改

### 8.1 Tracker::drop 与会话结束

**当前有两条独立路径，迁移必须同时覆盖：**

| 入口 | 当前 owner/触发者 | PR-01 处理 |
| --- | --- | --- |
| `Tracker::new().with_auto_export(path)` | 当前 Tracker 的 Drop；配置在 clone 间共享 | 删除 587–590 行的释放循环及其专用准备代码，保留 592–606 行独立导出分支；可抽成私有 `export_legacy_on_drop()` |
| `start()/start_with()` | `MemScopeGuard` 持有 GlobalTracker 和 flusher，Drop 调用 `lifecycle::export_for_reason(Drop)` | 保留并验证现有 guard 路径，继续遵守 `on_exit` 配置 |

自动导出代码位于释放循环**之后**，不在循环内部。单独删除循环不会取消导出；删除整个 Drop 或只改调全局 lifecycle 才可能破坏旧入口。独立 Tracker 未必安装了全局 lifecycle，不能让其导出依赖 `start()`。

**明确的目标 owner 与落地阶段：**

1. PR-01 保留上述两条导出路径，自动导出只读事件，不伪造释放。旧独立 Tracker 的 drop 导出是兼容性快照，不表示全会话结束；该阶段不擅自改变其 clone/drop 导出次数契约。
2. PR-02/03 接入会话时，在 `CaptureSession` 实现幂等 `stop(reason)`，唯一负责提交 SessionEnded 并冻结最终 cut。在 `MemScopeGuard` 增加 `stop(&mut self)`，协调停止 flusher、结束会话和最终导出；Drop 调用相同内部流程。公共显式 stop 返回结果，Drop 记录失败。方法名为拟新增 API。
3. 全局模式的正常结束 owner 明确为 **MemScopeGuard**；panic/signal/exit hook 请求同一会话的 stop 协议。独立 Tracker 提供显式 `Tracker::stop()`，调用者主动结束其共享会话；任意普通 clone 的 Drop 都没有结束权限。
4. PR-07 将两条路径的导出实现统一到 ReportSnapshot。迁移文档提供旧 `with_auto_export` 到 guard/显式 stop 的示例；兼容窗口内继续支持旧入口，不能要求未修改代码的用户突然持有 guard。显式 stop 已完成的最终导出不能再由随后的 Drop 重复执行。
5. 不能通过 `Arc::strong_count()==1` 猜测最后一个业务 owner，因为后台线程和全局注册也持有 Arc。普通 export_now、周期 snapshot 都不结束会话。
6. guard 可能早于业务对象销毁，结束只表示观测截止。若周期 flusher 已写过报告，最终 SessionEnded cut 仍必须发布；将结束幂等与周期导出去重分开。

“恰好一次”验收针对**成功完成的最终导出批次**：PR-01 先验证 `on_exit=true`、无周期/手动导出的 guard drop 场景；PR-03/07 再验证 stop+drop、周期+最终导出与重试。一次导出会写多个文件，不能用文件个数判断次数。`on_exit=false` 的零次退出导出是预期行为。

### 8.2 一次导出使用一个 ReportSnapshot

修改 `src/lifecycle.rs`、`src/guard.rs`、`src/periodic_flusher.rs`、`src/render_engine/export.rs`、Dashboard context builder。

新增冷路径输入：

```rust
pub struct ReportSnapshot {
    pub analysis: Arc<AnalysisSnapshot>,
    pub passports: PassportProjection,
    pub tasks: TaskProjection,
    pub system_sample: Option<SystemSample>,
}
```

步骤：

1. lifecycle 将 ExportReason 传到 cut/snapshot 构建处，不只用它判断导出开关。
2. flush 采集到一个 barrier，得到同一 AnalysisSnapshot 与关联投影水位。
3. HTML、JSON、ownership graph、task graph、leak report 都从该 ReportSnapshot 派生。
4. `snapshot_json()` 使用同一 DTO 转换，避免走另一套 Analyzer 统计口径。
5. context builder 不再调用 `tracker.analyze()`、`get_all_passports()` 等重新取得另一个时刻的数据。
6. 旧按路径导出接口内部只负责构建一次 snapshot，然后调用新的 `export_*_from_snapshot`。
7. 周期 flusher 的最后一次快照不能让最终 CaptureEnded 标记因 latch 已置位而丢失。快照去重按 cut/reason 管理，会话结束与“文件已导出”分开。
8. 导出期间用户继续分配不会改变已经捕获的 snapshot；系统指标有独立 sampled_at。

## 9. Dashboard 与 JSON 字段改造

### 9.1 指标口径表

| 当前字段/展示 | 处理方案 | 正确数据来源 |
| --- | --- | --- |
| `scheduler_lag_ms` | 当前分配间隔改名 `allocation_interval_mean_ms`；真正 scheduler lag 无 hook 时为空 | 同线程或同明确分组的已记录分配事件间隔；分组口径写入说明 |
| `scheduler_lag_bars` | 若保留，改为分配间隔直方图并输出 bin 边界/count | 全部相关事件，不仅快照时仍存活的对象 |
| `migration_rate_pct` | 当前值改名 `non_dominant_thread_allocation_share_pct` | 分配线程分布 |
| 真实 task migration | 可后续接通，不能复用上面的比例 | 同一 task 相邻 poll 的 thread_id 变化次数/可比较相邻 poll 次数 |
| poll latency | 分离 task_duration_ms、poll_duration_ms、wake_to_poll_ms | 前者是任务起止；后两者需要 poll/wake hook |
| `CORE_AFFINITY_PIN` | 没有 OS 查询或显式配置记录时为 Unknown | 实际 affinity 信息；不能以线程数 <= 核心数判断 |
| `ADDR_REUSE_RATE` | 短期修复阈值；最终从绑定历史统计 | 重用地址的绑定次数/有效绑定建立次数，分母为 0 时 N/A |
| `is_leaked` | V2 用 allocation_state + leak_assessment | 分配账本 + 诊断规则 |
| `evidence/confidence` | 改为字段/结论级 | Evidence 与诊断来源 |
| `immutable_borrows/mutable_borrows` | 未采集为 null | 显式借用事件 |
| `peak_memory` | 显示 tracked_requested_peak_bytes 与覆盖范围 | 完整 reducer 历史/checkpoint |
| 容器大小 | 分开 payload、capacity、object_layout、allocator_requested | 类型观测与 allocator 事件 |

地址复用率最终定义：只统计数据完整区间中真实分配绑定建立；首次使用某地址不是复用；原地 realloc 不建立新绑定；搬迁到曾用地址会建立复用绑定。不得从页面当前存活行推算整个会话复用率。

真正 wake→poll 指标接入时，必须明确多次 wake 合并、首次 poll 无 wake、跨线程 wake 的处理，保留原始样本。未接通前不展示为已测指标。

### 9.2 DTO 和前端索引

修改 `src/render_engine/dashboard/renderer/{types,event_dto,report_builder,context,render_methods}.rs` 和 `src/render_engine/dashboard/templates/dashboard_unified.html`：

- Allocation DTO 增加 allocation_id、session_id、allocation_state、field_evidence、leak_assessment、binding_history。
- Observation/Handle 使用独立实体 DTO，不能混入 heap allocation count。
- Event DTO 保留 event_seq、实体 ID、旧/新绑定、realloc 结果、task_id、source。
- DataIndex 增加 `allocation_id_to_row`、`entity_id_to_events`、`allocation_id_to_passports`。
- ptr 索引保留为多值搜索索引；Time Travel 根据 seq 查询当时绑定，不能“取该地址第一项”。
- 所有图跳转、详情联动和事件筛选使用 EntityRef 序列化后的稳定 ID。
- 旧 `selectNode(ptr)` 遇到多代对象时显示选择或按 cut 解析，不静默选错。
- Rust u64/u128 ID、seq、绝对纳秒时间用 JSON 字符串保真，前端图表使用相对时间数值。
- 超出 JavaScript 安全整数范围的字节累计值也采用字符串/BigInt；格式化阶段再换算单位，不把 round 后的显示值用于比较与关联。
- 未知值显示“未采集/证据不足”，真实 0 显示 0；UI 的排序、筛选和汇总要支持 null。
- 先计算完整统计再选择页面展示行；显示截断不改变全局计数。

### 9.3 JSON V2 外层

```json
{
  "schema_version": 2,
  "session_id": "0123456789abcdef0123456789abcdef",
  "snapshot": {
    "through_seq": "1200",
    "captured_at_ns": "850000000",
    "reason": "on_demand"
  },
  "capture_quality": {
    "mode": "allocator_events",
    "accepted_events": "1200",
    "dropped_buffer_full": "0",
    "legacy_semantics_ambiguous": false
  },
  "stats": {},
  "allocations": [],
  "observations": [],
  "handles": [],
  "relations": [],
  "diagnostics": [],
  "display_selection": {
    "total_rows": 800,
    "displayed_rows": 100
  }
}
```

此处是 envelope 示例，实际字段按第 4 节完整序列化。所有 V2 导出文件包含相同 session_id/cut，便于验证关联；V1 的兼容投影及双发关联信息按 §10.4 处理。

## 10. 旧接口与旧数据如何迁移

### 10.1 Rust 源码兼容

- 初期保留现有 `MemoryEvent` 与旧方法签名，新增 RecordedEvent 和 V2 recording 方法。
- `src/event_store/legacy.rs` 负责旧事件规范化；内部主路径逐步转为 V2。
- 给公共结构体增加字段会破坏外部 struct literal；`#[serde(default)]` 只解决反序列化，不能解决 Rust 源码兼容。
- 需要变更 `AllocationInfo`、`ActiveAllocation`、`AnalysisReport` 的地方，在 minor 版本迁移说明中列出；过渡期优先新增 V2 类型/转换函数。
- 旧 `is_leaked` 若保留，只兼容映射 Suspected/Confirmed，不能继续由无释放时间生成。V2 同时提供状态字段；严格 V1 的降级含义放在迁移说明及双发 manifest 中，不强加破坏旧 schema 的字段。
- 旧裸 ptr 查询定位当前 cut，不能用于跨代历史关系；新增 ID 查询作为规范接口。
- `Trackable` 新增观测方法提供默认实现，但默认值只能为未知/保守值。

### 10.2 旧事件语义不足时的处理

旧 MemoryEvent 没有 source、generation、realloc 新地址，也混用了 HeapOwner 与变量观测语义，不能可靠恢复所有信息。

导入器必须：

1. 设置 LegacyImport，标记 `legacy_semantics_ambiguous=true`。
2. 使用稳定的导入 session 上下文和输入序号生成局部身份，保证同一次导入重放确定性。
3. 无法区分重复观测与新分配时记录诊断，不凭空声明为完整 allocator trace。
4. 单地址 realloc 仅作为旧格式的原地重分配声明处理，不能推断搬迁目标。
5. 无来源的 clone 关系按旧声明保留，证据降级；不能自动当作真实 clone 操作证明。
6. 缺少时钟信息的旧时间戳只用于旧格式显示/输入顺序，不能强行转换成准确的单调时长。

### 10.3 双路径对照与切换

- 迁移阶段对支持的真值 fixture 同时跑旧输出和新输出，列出预期差异。
- 对照不是要求新代码保留旧 bug；差异需要对应 A01–A19 或明确 schema 变更。
- 切换顺序：新的 replay API → MemoryView/Snapshot → Analyzer/Tracker → JSON/HTML → 删除旧生产重建器。
- `docs/LIMITATIONS.md`、中英文 README/API 文档同步解释观测范围与字段变化。
- 更新旧计划中的“已完成”和百分比表述时，引用基准报告，不沿用未经测量的数字。

### 10.4 JSON 消费兼容、双发与回滚

当前并无统一 V1 envelope：`export_snapshot_to_json()` 写 memory_analysis/lifetime/thread_analysis 三种文件，`export_all_json()` 还写 Passport 等专题文件；`snapshot_json()` 序列化 Analyzer 的 AnalysisReport。`ExportJsonOptions` 没有 schema/version 开关。因此不能只给某一个输出加 `schema_version: 2` 就视为完成迁移。

**发布与配置契约（PR-07 实现，M1 建立样例与文档骨架）：**

- 新增 `JsonExportMode::{LegacyV1, Dual, V2}`，由新的 `ReportExportOptions`/显式版本化接口承载；旧公共 options 的结构体兼容按 §10.1 处理。版本选择必须能在运行配置中切回，不能只靠重新编译 feature。
- 预览阶段旧 API 默认 LegacyV1，主动开启 Dual 验证。首次发布 V2 的 minor 版本，新的报告接口默认 Dual，旧接口继续 LegacyV1；至少经过一个完整已发布 minor 的兼容窗口、消费者验证和发布说明后，才在后续明确公告的版本考虑 V2 默认。
- Dual 下 `<output>/memory_analysis.json` 等旧文件保持各自原有字段名、类型与路径；V2 写入 `<output>/v2/<export_id>/*.json`。同一批次只生成一次 ReportSnapshot，然后执行两套**序列化投影**。
- `export_id` 标识一次发布批次，包含配置/schema 版本；`session_id/through_seq` 标识分析窗口。不能因为切换格式复用一个不含新格式文件的旧导出批次。
- V2 使用 `<output>/v2/current.json` manifest：记录 export_id、session_id、cut、文件清单、投影差异及各格式结果；数据文件写好、校验完成后最后原子替换 manifest。V1 不强制添加 envelope；manifest 关联旧路径的内容哈希，允许对照者发现混合批次。旧路径按文件原子替换不等于整个多文件集合原子发布。
- `snapshot_json()` 继续返回原形状；新增 `snapshot_json_v2()`。字符串 API 不把两种 JSON 拼接返回。HTML 内嵌数据必须与对应 renderer 版本绑定，不能让旧模板直接消费 V2。
- V1 兼容保证结构与已声明可表示字段，不恢复虚假释放等 bug。无法表达 Unknown/新身份的必填旧数值必须在逐字段映射表明确为“已记录事件量/最后观测量”等降级口径，并在 manifest 标记；没有可解释映射则返回格式不支持错误，不静默用 0/false 伪装真实值。

**失败与回滚：**

1. Dual 的 V1/V2 writer 分别记录成功/失败；V2 失败仍尝试完成 V1，显式 API 返回包含分格式结果的 ExportOutcome，Drop/退出路径记录错误。失败批次不更新 V2 current manifest，消费者按 cut 识别上次成功报告。
2. 若问题仅在 V2 schema/序列化/前端，切回 LegacyV1 和匹配的 V1 renderer，不修改事件、身份或 reducer；保留历史 V2 文件以供定位，不把旧 manifest 冒充新导出。
3. 若错误在 reducer/采集，切 schema 无法恢复正确数据。回滚到上一已验证发布版本并新建采集会话；已有数据保持原 schema 与来源，不能假定旧二进制能读取 V2。
4. 删除 V1 writer 前必须完成兼容窗口、下游迁移验证与回滚演练。双发不能通过保留第二套旧 reducer 实现。

**验收：**为每个现有 JSON 家族保留 golden/消费者解析 fixture；验证 Dual 同 cut、旧脚本解析、必填字段类型、V2 精度、部分写入失败、manifest 不提前发布，以及 Dual→LegacyV1→Dual 切换。对目录递归扫描的消费者明确版本子目录过滤规则。仓库中的 `memscope-report/*.json` 可用于盘点，真值 fixture 应由受控场景重新生成。

## 11. UTI 与关系推断如何提升准确性

该部分建立在身份、来源与真值集稳定之后。

### 11.1 类型证据优先级

1. 有明确类型的同一对象观测：保留真实类型，不被字节猜测覆盖。
2. allocator 只知道某块大小：不能直接认为整块布局就是 Vec/String 的对象头。
3. 没有类型观测时，根据内容、调用栈、大小等产生候选。
4. 候选证据不足或冲突时返回 Unknown。

UTI 应区分“对象头布局”“容器数据缓冲区”和“未知字节块”三种输入角色。Vec 的 heap buffer 不是 Vec 对象自身，不能把 buffer 前三个 word 普遍解释成 ptr/len/cap。

### 11.2 拒判与分数设计

在 `src/analysis/unsafe_inference/engine.rs` 增加：

```rust
pub struct InferencePolicy {
    pub min_score: u16,
    pub min_margin: u16,
}

pub struct InferenceDecision {
    pub predicted: Option<TypeKind>,
    pub best_candidate: TypeKind,
    pub raw_score: u16,
    pub runner_up_score: u16,
    pub abstain_reason: Option<String>,
}
```

- `finalize()` 排序候选，检查最高分、第一第二名差值和必要特征是否存在。
- 平票/近似平票不能依赖 enum/table 顺序选择 Vec。
- 内部分数从容易饱和的 u8 改为 checked/受控范围的更宽类型，保留每个信号贡献。
- 阈值从验证集选择，不在计划里写一个未经测量的“最佳阈值”。
- `confidence` 在未校准前展示为 heuristic score，不带概率百分号。
- 如后续做概率校准，输出校准版本、校准数据集版本和 reliability bins；raw_score 与 calibrated_probability 分开。
- Unicode/UTF-8 信号不能把中文、日文、emoji 的低 ASCII 比例当作不是字符串的充分证据。
- CString 与 Vec<u8> / String 可能字节完全相同；接受不可辨识样本返回 Unknown。

### 11.3 推断验收指标

每种 TypeKind 和关系类型分别报告：

- TP / FP / FN、precision、recall、F1。
- Unknown/拒判比例与 accepted coverage。
- accepted predictions 的准确率与总体召回率，防止“全部拒判”虚增准确率。
- 内容不可用、符号缺失、采集缺口、不同平台下的分组结果。
- 去掉 content/stack/lifetime 等信号的消融结果，确定收益来源。

训练/调参用的 fixture 与最终验收集按“程序场景家族”划分，不能只换随机种子就称为独立测试集。

## 12. 测试设计与验收矩阵

### 12.1 新增文件

```text
tests/
  accuracy_lifecycle.rs
  accuracy_identity.rs
  accuracy_reallocation.rs
  accuracy_attribution.rs
  accuracy_export_consistency.rs
  accuracy_capture_quality.rs
  accuracy_inference.rs
  accuracy_derive.rs
  accuracy_allocator_e2e.rs
  fixtures/accuracy/
    README.md
    lifecycle/
    identity/
    inference/
    expected/
```

原则：

- 可注入时钟、会话 ID、分配序列与采集容量，避免依赖 sleep 和实际地址复用概率。
- 纯 replay fixture 使用符号地址，绝不解引用这些地址。
- 真实 allocator 的地址复用不能假定必然发生；用受控 allocator/原始事件注入验证边界。
- 真值期望由测试场景的实际操作声明，不能调用生产 reducer 生成 expected。
- 状态机 proptest 用独立的简化参考账本比较，生成合法与非法事件序列。
- 涉及全局 allocator、全局 TLS/registry 和退出 hook 的场景放独立子进程，避免测试框架自身分配影响断言。

### 12.2 必须覆盖的场景

| 场景 | 必须断言 |
| --- | --- |
| Tracker clone 被销毁，原对象仍有效 | 不新增 Free，不改变存活账本 |
| guard 在业务对象之前销毁 | 记录观测截止，不批量释放/确认泄漏 |
| 独立 Tracker.with_auto_export，未 start 全局会话 | Drop 仍写出旧 JSON 文件，内容保留观测记录，不新增 Free |
| on_exit=true，无周期/手动导出，guard 被销毁 | 完成一个最终导出批次；计数 writer 调用，不能只断言文件存在 |
| 显式 stop 后 drop / 周期快照后结束 | 最终批次不重复，最终 cut 包含结束标记；周期批次可有多个 |
| alloc 后 live snapshot，再实际 free | 第一次 Live/NoEvidence，第二次 Freed，计数只减一次 |
| 相同值重复 track | observation 增加，真实 allocation bytes/count 不增加 |
| Arc/Rc 三个句柄共享一个数据对象 | 多个句柄/观测，一个已验证分配绑定，无重复 bytes |
| Weak 存活、强引用归零 | 不用强引用计数推断整块 allocator 内存已释放 |
| 地址 p 的 G1 free 后 G2 alloc | G2 不继承 G1 的 clone、Passport 和诊断 |
| G1 的迟到 free 到达时 G2 存活 | 不能删除 G2；生成 stale/conflict 诊断 |
| realloc 原地增长/缩小 | 同 AllocationId，size 与累计量更新，元数据保留 |
| realloc 搬迁 | 旧地址关闭、新地址绑定，历史可追溯 |
| realloc 失败 | 原地址、大小、身份不变，无虚假 free |
| 空容器、ZST、未分配 dangling 地址 | 无伪造 heap allocation |
| A 分配 B 释放 | 全局与 A 来源 live_bytes 归零，B 活动计数正确 |
| 任务 Pending 后另一任务执行 | 第二个任务不继承前一个任务 TLS |
| 嵌套 scope / panic / task abort | 恢复父上下文，无跨任务污染 |
| TaskIdRegistry 与 GlobalTracker 同时开启 | 同一提交事件只计一次 |
| FFI handover 后运行中导出 | custody=Foreign，无默认 Confirmed leak |
| FFI free 与重复通知 | 对正确 ID 仅发生一次扣减 |
| 并发写入与 snapshot barrier | 返回连续前缀；所有投影使用同 cut |
| queue overflow / reset / 缺失 free | 质量计数正确；不能标完整/Confirmed leak |
| 显示截断 100/1000 行 | 全局统计仍来自 1000 行，质量不等同采集完整率 |
| Snapshot/MemoryView/Analyzer/JSON/HTML | 公共 bytes/count/状态/ID 完全相同 |
| 小尺寸分配完整历史 | peak 等于每次转换后的最大 current，不受最终 live 集合影响 |
| 大于 2^53 的 ID/纳秒/累计值 | JSON→前端→关联查询不丢精度 |
| V1/V2 双发、V2 writer 失败、切回 V1 | 同 cut 投影；旧消费者可解析；失败显式可见；manifest 不指向半成品 |
| 独立构造的同内容数组 | 不作为真实 Clone 正例 |
| 实际 Vec clone，已记录操作 | Clone 操作被检测/关联，源目标 ID 正确 |
| ASCII/中文/emoji/String/Vec<u8>/CString | 分别报告混淆，证据不足可 Unknown |
| UTI 所有得分低/最高分近似平票 | 必须拒判，输出理由 |
| derive 泛型 struct、enum、嵌套容器 | 实际编译并调用当前 Trackable 接口 |

### 12.3 并发测试

- 对 ingress 发布、barrier、commit watermark、会话结束、reset 的精简同步协议使用 loom。
- loom 验证实际采用的同步协议；不能只对与生产实现无关的小模型给出“并发正确”结论。
- 用压力测试补充真实队列行为与高频 alloc/free/realloc。
- 原子计数更新顺序、完整快照水位与对外 `len()` 的近似语义分开验证。
- 重入测试覆盖初始化、导出内部内存、队列满、panic 期间尝试快照。

### 12.4 工程检查与运行方式

每个 PR 先跑相关测试，合并前完成对应 feature/平台检查。以下新测试命令在文件落地后使用：

```sh
cargo test --locked --test accuracy_lifecycle
cargo test --locked --test accuracy_identity
cargo test --locked --test accuracy_reallocation
cargo test --locked --test accuracy_attribution
cargo test --locked --test accuracy_export_consistency
cargo test --locked --test accuracy_capture_quality
cargo test --locked --test accuracy_inference
cargo test --locked --test accuracy_derive
cargo test --locked --features tracking-allocator --test accuracy_allocator_e2e -- --test-threads=1
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo nextest run --locked --workspace --test-threads 1
cargo test --locked --doc
```

- 另跑 `--no-default-features`、默认 features、backtrace 与 allocator 相关配置；构建检查不替代 feature 实际运行测试。
- derive 是路径依赖且根 manifest 当前没有显式 workspace members。必要时使用 `--manifest-path memscope-derive/Cargo.toml` 单独验证，不假定根 `--workspace` 已覆盖其所有测试。
- CI 的纯 replay/推断 fixture 在 Linux/macOS/Windows 都运行；实际 runtime/allocator 能力按平台标注支持范围。
- 通用 API 性能使用现有 `comprehensive_benchmarks` target：`cargo bench --locked --bench comprehensive_benchmarks`。GlobalAlloc 热路径须补 §13.3 的基准，不能用通用 target 的结果替代。保持同工具链、同 Cargo.lock、同 workload 比较。
- `.github/workflows/ci.yml` 增加准确性分组与结果 artifact；准确性基线任务固定工具链和依赖，避免运行中重新生成锁文件造成对照漂移。
- 已有 CI 包含 Linux/macOS/Windows 测试矩阵（61–107 行），新增的是准确性分组和 feature 覆盖。benchmark job（140 行）仍调用不存在的 `minimal_performance`，应在 M1 修复 target 与命令；现有 benchmark job 只在 master push 上运行，不能作为 PR-05 的自动性能门禁。

### 12.5 发布验收门槛

1. 全采集、受控真值场景中的计数、字节、状态和 ID 关联全部精确相等。
2. 正常存活与正常释放负例中，Confirmed leak 数量为 0。
3. 地址复用、跨线程释放、realloc 各类场景没有串代或重复扣减。
4. 对同一 ReportSnapshot，所有导出投影公共指标零差异。
5. 每个注入的采集缺口都有质量标记，不再输出“完整采集”。
6. UTI 同时报告 precision、recall 和拒判比例；不预先承诺未经基准支持的百分比提升。
7. 提交采集吞吐、单事件开销、snapshot 时间和峰值 profiler 内存的基线/改造对照。性能预算在基线测得后写入 PR，不能沿用未经测量的“<5%”。

## 13. PR 拆分、依赖与预计工作量

工作量为单人有效开发日估计，包含相关回归测试，不包含等待 CI/评审。allocator 与跨平台部分波动较大，以验收结果推进。

| PR | 内容 | 主要文件 | 依赖 | 估计 |
| --- | --- | --- | --- | --- |
| PR-01 | 停止虚假释放，初步分离 live/leak，保留两条自动导出路径并回归 | tracker.rs、report_builder.rs、guard/lifecycle 测试、accuracy_lifecycle.rs | 无 | 2–3 日 |
| PR-02 | 身份、来源、V2 事件、质量与旧格式适配，构造器纯化 | core/types 新文件、event_store 新文件、event.rs、store.rs | PR-01 | 3–5 日 |
| PR-03 | 唯一 reducer、realloc、历史、统计守恒、同 cut 快照与 stop 契约 | snapshot/、view/、timeline/、tracker.rs、guard/lifecycle、event_reconstructor.rs | PR-02 | 4–6 日 |
| PR-04 | 变量观测/句柄分离、真实类型布局、derive 对齐 | lib.rs、tracker.rs、tracking/registration.rs、memscope-derive/ | PR-02/03 | 3–5 日 |
| PR-05 | allocator 完整事件与有序提交、barrier、丢失可见、源去重、专用热路径基准 | core/allocator.rs、capture/recording.rs、capture/allocator_events.rs、core_tracker.rs、benches | PR-03/04 | 5–8 日 |
| PR-06 | 任务归属/TLS、Passport ID、FFI 事件与关系图身份迁移 | task_registry.rs、async_*、memory_passport_tracker.rs、relation_inference/、ownership_graph.rs | PR-03/05 | 5–8 日 |
| PR-07 | ReportSnapshot、导出一致性、Dashboard 指标与前端关联、JSON 双发/回滚 | lifecycle/guard/periodic_flusher、export.rs、renderer/、模板 | PR-03/04/06 | 4–6 日 |
| PR-08 | 一致内容快照、准确性语料、UTI 拒判/校准、关系误报治理 | heap_scanner/、unsafe_inference/、relation_inference/、fixtures | PR-04/06/07 | 4–7 日 |
| PR-09a | M1 滚动完成：修正 CI benchmark target、接入已落地准确性测试、迁移文档/消费者 fixture 骨架 | CI、Makefile、docs、tests | 框架可独立；具体测试随 PR-01–03 | 计入 PR-09 总预算 |
| PR-09b | M2 滚动完成：hook 性能结果、版本映射、feature 门禁与回滚演练随代码合入 | CI、benches、docs、tests | 对应 PR-04–07，按项依赖 | 计入 PR-09 总预算 |
| PR-09c | M3 最终收口：汇总平台/性能证据、核查迁移说明和旧路径清理 | docs、README、适配入口 | PR-08 及 09a/09b 验收 | PR-09 三部分共 2–4 日 |

原总量约 32–52 个有效开发日，作为初始估算保留。提前交付改变依赖等待，不会凭空减少单人总工作量；新增双发/专用基准若超出原 PR-05/07 预算，应在各自短 spec 中重新估时。PR-09 的 2–4 日分布到 09a/b/c，不能全部留到最后：

- **M1：PR-01–03 + PR-09a**，生命周期与统一账本可回归验证，CI 与迁移骨架同步可用。
- **M2：PR-04–07 + PR-09b**，采集与报告闭环，热路径性能、消费者兼容与回滚验证同步完成。
- **M3：PR-08 + PR-09c**，推断验收与发布证据汇总，不在此阶段首次搭建 CI 或运行基准。

### 13.1 PR-01 的具体实施清单

- [x] 在 `tests/accuracy_lifecycle.rs` 添加 Tracker clone drop 不产生 Free 的回归。
- [x] 添加活对象导出不自动标泄漏的回归。
- [x] 添加实际释放后 bytes/count 正确扣减的回归；使用明确显式事件契约或受控 allocator，不能靠 tracker drop 模拟真实释放。
- [x] 仅删除 `Tracker::drop()` 的批量 Deallocate 及专用准备代码；保留循环之后的独立 `with_auto_export` 分支，必要时抽私有 helper。
- [x] 独立 Tracker 不调用 start()，启用 with_auto_export 后 drop 仍写出 memory_analysis/lifetime/thread_analysis JSON；读取内容并检查共享 EventStore 无新增 Free。使用临时目录，不能只运行当前无断言的 `test_tracker_with_auto_export`。
- [x] 明确正常结束 owner 为 MemScopeGuard，按 §8.1 将正式结束逻辑迁入 guard/显式 stop；PR-01 保持旧 drop 快照兼容，不等待 PR-02/03 的 V2 会话接口。
- [x] 验证 guard 被销毁后 auto-export 仍完成一次（on_exit=true、无周期/手动导出），通过测试 writer/批次计数断言；增加 on_exit=false 不产生退出导出的对照。
- [x] `report_builder` 停止以缺少 dealloc 时间赋值 is_leaked，增加可过渡的状态字段/映射。
- [x] 确认现有测试中哪些预期依赖虚假释放，逐个改为真实操作或明确的观察状态。
- [x] 运行相关现有生命周期与 auto-export 测试，验证导出文件仍生成且包含存活记录。

### 13.2 每个后续 PR 的提交要求

每个 PR 描述固定包含：

1. 解决的 Axx 问题与新数据契约。
2. 修改的数据类型、主要调用路径、源兼容和 JSON 兼容影响。
3. 至少一个可重放的输入与预期输出。
4. 相关测试结果与该阶段仍不能判定的场景。
5. 若触及热路径，提交同环境性能对照。

### 13.3 PR-05 的热路径基准交付

源码核查结论：现有 `allocator_comparison`（benches/comprehensive_benchmarks.rs:81）比较 Vec 分配与显式 track!；`backend_alloc/realloc`（439/515 行）使用 0x1000 等常量调用 capture 方法。CoreBackend 直接构造 MemoryEvent，这些场景不覆盖实际 GlobalAlloc→队列→提交。默认 features 未启用 tracking-allocator；启用后同一二进制中的 `system_allocator_baseline` Vec 也走全局 TrackingAllocator，不能再称为干净的 System 对照。

- [ ] PR-05 改热路径前确认覆盖并记录基线；新增 `benches/allocator_hook.rs` 和 Cargo.toml target（拟新增），通用 benchmark 继续保留。
- [ ] 使用同 workload 的独立进程/构建对比 System、hook 注册但关闭、hook 开启；控制 feature 差异，验证每种模式实际使用的 allocator 和事件计数。不能只根据 benchmark 名称认定测到了 hook。
- [ ] 覆盖 alloc/alloc_zeroed/free/realloc、多种大小/对齐、单线程/多线程、跨线程释放；固定迭代数量、队列容量、消费者配置。增长/缩小的真实 realloc 与受控搬迁测试分别记录，不能宣称系统 allocator 必定原地或搬迁。
- [ ] 分别测调用线程成本（ns/op、吞吐、门锁等待）和包含 drain/barrier 的端到端吞吐；需要 p50/p95/p99 时采集实际调用延迟样本，不能把 Criterion 批次均值当尾延迟。
- [ ] 测正常负载与队列溢出负载；报告 accepted/dropped、最大 backlog、profiler 内存。较快但丢事件的运行不能与完整采集基线直接比较。
- [ ] 初始化/预热不计入稳定热路径；预分配测试辅助内存，确认框架/消费者内部内存的排除规则。单列端到端结果中的真实消费者和归约开销。
- [ ] 合入前附可复现命令、硬件/OS、工具链、Cargo.lock、feature、基线/改造 commit 和结果；M1 修正 CI target，PR-05 开通该基准的 PR/按需运行入口，PR-09b 收录报告。

### 13.4 每个 PR 的短实施 spec

主文档保留跨模块契约；每个 PR 开工前在 `docs/accuracy/pr-XX.md` 写 1–2 页 spec（拟新增，随 PR 创建，不先堆空模板），固定包含：

1. 本 PR 目标、依赖及对应 Axx/本文章节链接。
2. 修改的类型/API、文件及调用链；只有发生变更的契约才摘录。
3. 输入→状态转换→预期输出的最小真值场景。
4. 可直接执行的检查命令、通过标准、兼容/回滚办法。
5. 交付清单与本 PR 重新核实的工期、性能预算。

spec 引用 §4/5 的数据结构和算法，不维护第二份完整设计。PR-01 的具体任务以 §13.1 为起点。

## 14. 代码变更总表

### 14.1 建议新增

| 文件 | 职责 |
| --- | --- |
| `src/core/types/identity.rs` | 会话、分配、句柄、观测身份和地址绑定 |
| `src/core/types/evidence.rs` | 字段级证据与来源 |
| `src/core/types/observation.rs` | 类型、布局、句柄和数据区观测 |
| `src/event_store/recorded.rs` | V2 规范事件 |
| `src/event_store/legacy.rs` | 旧事件适配及模糊语义诊断 |
| `src/event_store/quality.rs` | 来源覆盖与采集缺口 |
| `src/capture/recording.rs` | 会话、公共提交入口与 barrier |
| `src/capture/allocator_events.rs` | 固定大小 hook 原始事件和队列 |
| `src/snapshot/state.rs` | 分配账本、历史、统计、诊断类型 |
| `src/snapshot/reducer.rs` | 唯一状态归约器 |
| `src/tracking/registration.rs` | 显式句柄注册 token |
| `src/render_engine/report_snapshot.rs` | 所有导出共享的不可变输入 |
| `benches/allocator_hook.rs`、Cargo.toml bench target | 实际 allocator hook 的独立性能基准 |
| `docs/accuracy/pr-XX.md` | 随 PR 创建的短实施 spec |
| 第 12 节测试文件与 fixtures | 真值回归与准确性评估 |

各目录的 `mod.rs` / `src/lib.rs` 补模块声明与必要重导出。优先复用已有等价结构，只有语义不兼容时才新增文件/类型。

### 14.2 必须联动修改

- **入口与类型**：`src/lib.rs`、`src/core/types/{mod,track_kind}.rs`、`src/capture/types/{allocation,type_layout}.rs`、`memscope-derive/src/lib.rs`。
- **采集与状态**：`src/tracker.rs`、`src/core/allocator.rs`、`src/capture/engine.rs`、`src/capture/backends/{mod,core_tracker,global_tracking,async_tracker,async_types,task_profile,lockfree_tracker}.rs`、`src/event_store/{event,store,mod}.rs`。
- **快照与视图**：`src/snapshot/{builder,engine,types,mod}.rs`、`src/view/{memory_view,stats}.rs`、`src/timeline/query.rs`。
- **分析**：`src/task_registry.rs`、`src/analysis/memory_passport_tracker.rs`、`src/analysis/ownership_graph.rs`、`src/analysis/relation_inference/`、`src/analysis/unsafe_inference/`、`src/analysis/heap_scanner/`、`src/analysis/detectors/`、`src/analyzer/`。
- **输出**：`src/lifecycle.rs`、`src/guard.rs`、`src/periodic_flusher.rs`、`src/render_engine/export.rs`、`src/render_engine/dashboard/renderer/`、`src/render_engine/dashboard/templates/dashboard_unified.html`。
- **验证与说明**：`tests/relation_inference_integration.rs`、`tests/test_reallocation_tracking.rs`、`tests/auto_export_e2e.rs`、相关 examples、`benches/comprehensive_benchmarks.rs`、`.github/workflows/ci.yml`、`Makefile`、README/API/LIMITATIONS 文档。

`core/types`、`capture/types`、backend 私有类型中存在多个同名 AllocationInfo/MemorySnapshot，迁移时以第 3 节的数据流为边界建立明确转换，不做仅为统一名字的全库重命名。

## 15. 最终完成定义

专项完成时，应能够给出一份带 fixture 版本与 commit 的结果报告，回答：

1. **捕获到了什么？** 哪些 allocator/线程/task/FFI 操作在范围内，采样或丢失了多少。
2. **每个对象是谁？** 地址发生复用或 realloc 后，身份与关系仍然可追溯。
3. **统计怎么算出来？** 所有出口对同一 cut 使用同一个账本与公式。
4. **结论凭什么成立？** 类型、布局、关系、泄漏各自有来源、规则和未知状态。
5. **准确性改善了多少？** 确定性场景零偏差；推断场景有 precision/recall/拒判与性能实测。

下一步从 PR-01 开始，把三条生命周期回归及两条自动导出入口的兼容回归跑通；然后按身份契约推进唯一 reducer，逐步接通采集与所有报告出口。

## 16. 评审意见核查记录

本次核查基于 `c8db9de` 源码、当前计划与 `cargo metadata --locked --no-deps`；没有执行 allocator 性能测量或复现所有生命周期问题。

| 意见 | 核查结论 | 证据与处理 |
| --- | --- | --- |
| P0-1 自动导出 owner | 迁移职责不够明确，需补充；“导出在释放循环内部、仅删循环就丢导出”的定位不准确 | tracker.rs:587–606 是先循环、后独立导出；guard.rs:99–114 已有 guard 路径。§8.1/13.1 明确双入口、stop owner、分阶段迁移和次数回归 |
| P0-2 JSON 双发/回滚 | 成立 | export.rs:77–94 无格式版本选择，170–198 输出三类文件；guard.rs:87/lifecycle::snapshot_json 是另一种结构。新增 §10.4 |
| P1-1 热路径基准 | 成立，已确认覆盖不足 | benches/comprehensive_benchmarks.rs:81/439/515；Cargo.toml:60–64；src/core/allocator.rs:76。新增 §13.3；另核实 CI:140 引用不存在的 minimal_performance target |
| P1-2 PR-09 收口风险 | 成立，属于实施依赖风险 | 原 PR-09 等待 01–08。现有 CI 已有三平台矩阵；§13 改为 09a/b/c 随 M1/M2/M3 滚动交付 |
| P2-1 导航锚点 | 不成立 | 实际标题为 `## 9. Dashboard 与 JSON 字段改造`、`## 12. 测试设计与验收矩阵`。GitHub 去掉句点但保留数字，因此现有 `#9-dashboard-与-json-字段改造`、`#12-测试设计与验收矩阵` 匹配，无需删数字 |
| P2-2 体量与短 spec | 建议合理 | 主文档承担整体设计；§7.1 改为引用算法章节，§13.4 规定每个 PR 随实施创建短 spec |
## 17. PR-01 实施记录（2026-10-30）

本段记录实际落地的内容，供 PR-02 引用；未在本文出现的条目仍未实施。

### 17.1 行为变更

| 位置 | 变更 |
| --- | --- |
| `src/tracker.rs` | `Drop` 不再为存活记录批量生成 `Deallocate`；`with_auto_export` 分支抽为只读私有方法 `export_legacy_on_drop()` / `auto_export_path()` |
| `src/snapshot/state.rs`（新增） | 引入 `AllocationState`（`Live` / `Freed{at_ns}` / `Unknown{reason}`）与 `LeakStatus` / `LeakAssessment`，把“物理状态事实”和“泄漏结论”分开；默认 `NotAssessed` 不产生泄漏 |
| `renderer/types.rs` | Allocation DTO 增加 `allocation_state`、`leak_status`；`is_leaked` 降级为 `leak_status` 的兼容投影 |
| `renderer/report_builder.rs` | 停止以 `timestamp_dealloc.is_none()` 赋值 `is_leaked`；`evidence` 改为按“是否存在类型观测”判定，`confidence` 不再由 leak 布尔值推导（A11） |
| `analysis/top_n.rs` | `top_leaked_bytes` 与 `summary()` 的泄漏口径改为显式 `is_leaked` 标志，不再把“无释放时间”当泄漏 |

### 17.2 结构整理（不改变行为）

- `src/tracker.rs` 测试迁至 `src/tracker/tests.rs`（699 / 484 行）。
- `renderer/report_builder.rs` 拆出 `layout_inference.rs` 与 `report_builder/tests.rs`（882 行）。
- `renderer/types.rs` 拆出证据类枚举 `evidence.rs`（986 行）。
- 清理 6 处既有 clippy 警告，`make check` 达到 0 error 0 warning。

### 17.3 回归覆盖

`tests/accuracy_lifecycle.rs`：clone/末次句柄 drop 不产生 Free、存活对象不被标泄漏、显式释放只扣减一次、`with_auto_export` drop 只读且写出三份 JSON。
`src/guard.rs`：guard drop 恰好一次退出导出、`on_exit=false` 不导出且不占用 latch。
`src/analysis/top_n.rs`：存活分配不计入泄漏统计。

通过变异验证：恢复旧的 drop 批量释放与 `is_leaked` 旧映射后，上述 4 个生命周期用例全部失败；恢复实现后全部通过。

### 17.4 仍待后续 PR 处理

A03–A19 全部保留：Dashboard/Snapshot 双重建器、realloc 与地址复用、跨线程归属、EventStore 缺口、Passport 副作用、UTI 拒判、derive 与 `Trackable` 占位类型名等。`context.rs`（1418 行）与 `render_methods.rs`（1279 行）仍超出单文件行数预算，需在对应 PR 内拆分。

### 17.5 第二轮核查与修复（P0/P1）

按源码逐条核实评审意见后落地的内容。表中“结论”为核查结果，“处理”为本次改动。

| 意见 | 结论 | 处理 |
| --- | --- | --- |
| P0-1 Dashboard 重建忽略 `Reallocate` | 成立 | 新增 `snapshot/bindings.rs::BindingLedger` 作为唯一绑定账本，`Reallocate` 按“地址 + 记录大小”匹配绑定并原地刷新 size；`event_reconstructor` 与 `snapshot/builder` 都改为消费该账本 |
| P0-2 释放按裸地址 `retain` 跨代污染 | 成立 | 释放只关闭**一个**绑定（优先命中相同大小，否则最旧一代）；迟到释放不再删除新一代 |
| P0-3 首次分配被算作复用（A13） | 成立 | 绑定代号从 1 开始；`context.rs` 复用率改为 `generation_id > 1`，并抽成 `compute_address_reuse_pct` 加回归测试 |
| P1-4 重建器把当前线程写入 `thread_id` | 成立 | 重建器不再写 `std::thread::current().id()`；`analysis/mod.rs`、`safety_detector` 聚合改用 `thread_id_u64`；`AllocationInfo` 序列化输出数值线程 ID 并补 `thread_id_u64` 字段 |
| P1-5 用 generation 判证据、用 is_leaked 判置信度（A11） | 上一轮已修，本轮复核 | `evidence` 依据“是否存在类型观测”，`confidence` 为 `Unknown`，`is_leaked` 由 `LeakAssessment` 投影；`accuracy_lifecycle` 经由 `build_context_from_tracker` 覆盖的正是这条 DTO 路径 |
| P1-6 UTI 取最高分、不拒判、u8 饱和、百分号（A17） | 成立 | 新增 `unsafe_inference/policy.rs`（`InferencePolicy` / `InferenceDecision`），`finalize` 要求 `min_score` 且领先次优 `min_margin`，否则拒判并给出理由；评分统一 `u16`；展示改为 `score N`；`utf8_validation` 改为 Unicode 感知并修正 NUL 文本误判 |
| P1-7 按 size 猜类型 | 成立 | 删除 `allocator.rs` 三个死代码推断函数及其测试；`report_builder` 的 `infer_type_from_size`（含伪造百分比）删除，无类型观测时输出 `unknown` |
| P2-8 双重建器（A04） | 部分收敛 | 语义层已合并为 `BindingLedger`，两个投影只做 DTO 映射；新增 `tests/accuracy_reconstruction.rs` 断言二者存活集合、字节总量、峰值、线程归属一致。完整 reducer（含 `AnalysisSnapshot`）仍属 PR-03 |
| P2-9 `MemoryEvent::allocate()` 构造副作用（A08） | 成立且范围更大 | `MemoryEvent::allocate` 与 `AllocationInfo::new` 的 `record_allocation` 副作用删除；任务内存改为在唯一采集点 `Tracker::track_inner` 记录，`global_tracking` 的重复累计删除 |
| P2-10 重名类型 / 生成产物 / 过期文档 | 成立 | `memscope-report/`、`aim/coverage/` 加入 `.gitignore`（文件未删除，取消跟踪需仓库 owner 执行 `git rm -r --cached`）；`docs/ARCHITECTURE.md` 与 `docs/README.md` 标注为历史设计说明，不再声称当前指标。6 组同名 `AllocationInfo` / 5 组 `MemorySnapshot` 的合并留待 PR-02/03 的类型边界重构 |

结构整理：`unsafe_inference` 拆出 `policy.rs` 与 `scoring.rs`（engine 1000 → 963 行）；`event_reconstructor` 由 345 行降至 414 行（去掉重复的生命周期 match 后净值下降，逻辑移至账本）。

变异验证：把 `Reallocate` 恢复为忽略、释放恢复为按裸地址 `retain` 后，`tests/accuracy_reconstruction.rs` 5 个用例中 4 个立即失败；恢复实现后全部通过。

补充（实现过程中新发现）：

- `report_builder::infer_provenance` 同样用 `generation_id > 0` 判定 `reallocation`，首次分配会被标成“重分配”，且 `is_leaked` 参与了溯源判断；已改为 `> 1` → `reused_address`，并把泄漏从 provenance 中剥离。
- `capture/backends/global_tracking` 的 4 个全局单例测试在并行执行下会互相 reset，导致偶发失败；已加 `#[serial]`。
- `test_real_box_i32` 读取的是活动堆地址，字节本身随运行变化，现按“接受任意 8 字节候选或拒判（必须带理由）”断言，避免把不确定数据当作确定结论。

### 17.6 第三轮核查与修复（A06 / A08 / A19 活体、任务归属）

| 意见 | 核查结论 | 处理 |
| --- | --- | --- |
| P0-1 `core_tracker` 复用地址覆盖 map 但仍累加（A06） | **成立**，两处：`active_allocations.insert` 覆盖 + `total_allocations/total_allocated` 照加 | `track_allocation`/`fast_track_allocation` 统一走 `record_binding`：同地址同大小 = 重复观测（不记账，计入 `duplicate_observations` 诊断）；同地址不同大小 = 原地重分配（`allocated += new`、`deallocated += old`，条目唯一）；新地址才新增计数。`allocated - deallocated == Σ live` 恢复守恒 |
| P0-2 async 双重累计（A08） | **部分成立**：`TaskIdRegistry` 的双重累计上一轮已删；`global_tracking → AsyncTracker` 是**另一套独立存储**（async KPI/任务档案），不是同一账本记两次，删除会让 async 面板恒为 0。真正的缺陷在 AsyncTracker 内部同样存在 A06 | 保留投影，修 `track_allocation_with_location`：重复观测不重复记账；原地大小变化走 `record_reallocation`，`total_allocations` 不增加；`global_tracking` 注释改为说明"两个独立投影各自只记一次" |
| P0-3 每次 HeapOwner 观测都记一次 borrow（A19） | **成立** | 删除 `global_tracking::track_as` 中按类型名推导 borrow 的整段；未采集的借用统计保持为空，而不是伪造事件 |
| P1-4 async 快照把当前线程当任务线程 | **成立**，且 `track_task_start` 的 `_thread_id` 参数被丢弃 | `TaskMemoryProfile` 新增 `thread_id`（创建/启动时写入，`ThreadId` 不参与 serde），`snapshot()` 使用它 |
| P1-5 task id 被截断 32 位 | **成立**，`TaskId = u128` 被 `& 0xFFFFFFFF` | 改为 `u64::try_from`，超出 u64 时告警并跳过归属，不做静默截断 |
| P1-6 `TaskGuard` 跨线程 drop 清错 TLS | **成立**，且同步侧 `task_registry::TaskGuard` 同样问题（嵌套作用域结束后父任务上下文丢失） | 两个 guard 都改为**保存并恢复**前一个上下文；仅当槽位仍属于自己时才写回。`track_task_end` / `unregister_task` 只在拥有槽位时清除 |

同时修复：全局单例测试（`task_registry` 7 个、`async_tracker` 6 个）并行互相 reset 导致偶发失败，已加 `#[serial]`。
结构整理：`core_tracker.rs`（1210 → 537）、`async_tracker.rs`（1560 → 999）的测试模块拆到同名目录 `tests.rs`。

变异验证：恢复"同地址照样计数 + 无条件插入"、guard 无条件清空 TLS、以及"观测即借用"三处旧行为后，新增用例立即失败（core_tracker 3 个、async_tracker 2 个、global_tracking 1 个）；恢复实现后全部通过。

### 17.7 第四轮：剩余缺口收口（async 代号、线程、诊断、搬迁 realloc、UTI 校准）

| 项目 | 结论 | 处理 |
| --- | --- | --- |
| async 释放只按地址关闭 | 成立（每地址单条，迟到释放会删掉新一代） | `allocations` 改为 `地址 -> Vec<记录>`，记录带 1 起始 `generation`（每地址代号持久化）；新增 `track_deallocation_matching(ptr, size, generation)`，带过滤条件时**只**关闭匹配的那一代，不匹配则仅记录诊断 |
| `TaskInfo.thread_id: ThreadId` 不可序列化 | 成立 | 改为 `Option<u64>`（数值线程 ID），`TaskInfo`/`AsyncAllocation`/`AsyncStats`/`AsyncSnapshot` 支持 serde；`TaskMemoryProfile` 记录 `thread_id_u64`，任务启动时仅当调用线程可映射时写入，分配时刷新 |
| `duplicate_observations` 只是内部计数 | 成立 | `Tracker::duplicate_observations()` 对外暴露，`EventSummary` 增加 `duplicate_observations` 字段，进入 HTML/磁盘 JSON 的 `event_summary` |
| allocator 无法表达搬迁 realloc | 成立 | `MemoryEvent` 新增 `new_ptr`（默认 `None`）与 `reallocate_moved()`；账本关闭旧绑定、在新地址开新代；`core_tracker::track_reallocation` 保持单条目；`TrackingAllocator::realloc` 覆写为“一次系统 realloc + 一次搬迁事件”，失败时保留原块。同时把手动 TLS 开关改为 `TrackingGuard` 作用域守卫（§6.4 第 6 条），提前返回/panic 都不会再把跟踪永久关闭 |
| `variable_registry` 的 `leaked_count` = active | 成立 | `leaked_count` 改为显式 `is_leaked` 计数，条目补 `is_leaked` 字段；`active_count` 语义不变 |
| 6 组 `AllocationInfo` / 5 组 `MemorySnapshot` | 成立，但按 §14.2 不做全库改名 | 为 11 处定义补充边界说明（属于哪一层、需在模块边界转换） |
| `memscope-report/` 取消跟踪 | 成立 | `.gitignore` 已加入；`git rm -r --cached` 属 git 操作，按项目规则未执行，留给仓库 owner |
| UTI 阈值未经测量 | 成立 | 新增 `policy::evaluate` / `policy::calibrate`（网格搜索 + precision/F1/拒判率）；确定性 4 样本 fixture 实测：`PRELIMINARY` 接受 2 个、precision 1.00、recall 0.50，其余 2 个因候选分差 5 分拒判。实测值写入 `InferencePolicy::PRELIMINARY` 文档并固化为断言；同时修正“非 UTF-8 即 Vec(+50)”的过强信号（降为 +20） |

结构整理：`async_tracker.rs` 1075 → 996（释放逻辑下沉 `async_tracker/dealloc.rs`），`variable_registry.rs` 1730 → 922（测试与 `inference.rs` 下沉）。

变异验证：把 async 释放过滤改回“无匹配就删最旧”、账本忽略 `new_ptr`、`leaked_count` 改回 active 计数后，对应用例立即失败；恢复实现后全绿。

### 17.8 第五轮：A06 复查（同址复用不得走原地 realloc）

**评审结论：成立且必须前置到 PR-02 验收。** `record_binding` 把"同址异 size"当成原地
realloc 时，无法区分两种情况：

1. 真的原地 realloc（allocator 返回了同一地址）；
2. 地址复用 + 中间 free 丢失（hook 未接、事件缺口、双路观测）。

第 2 种情况下 G2 会继承 G1 的账本行，`track_deallocation(ptr)` 随后按裸地址删代，
删错就是必然结果。**本轮先把行为改成保守语义**，把"身份连续"的判定权收回到唯一能观察它的
地方（allocator / 显式契约）。

| 位置 | 本轮改动 |
| --- | --- |
| `core_tracker::record_binding` | 同址同 size → 重复观测（不记账）；同址异 size → **新代**（旧绑定字节记为被取代 `replaced_bindings`，新绑定独立计数），不再原地覆盖 |
| `Tracker::track_inner` | 类型观测不再合成 `Reallocate` 事件；一律发 `Allocate`，让账本给新代独立身份 |
| `BindingLedger::on_allocate` | 同地址已有 live 绑定时保留两条并计 `conflicting_allocations`（冲突诊断，不静默覆盖），窗口标记为有缺口 |
| `AsyncTracker` | 同址异 size 仍是"更替旧记录"，但记录带 `generation`，释放可按代号匹配 |

#### PR-02 验收门槛（新增，必须满足后才可宣称"身份正确"）

1. **同址复用必须换 generation，不得走原地 realloc 分支。** `record_binding`、`BindingLedger`、
   `AsyncTracker` 三处必须一致；"原地 realloc"只允许由 allocator 观察（`realloc` 返回地址）
   或显式契约产生。
2. 释放必须按 `(ptr, generation)` 匹配；裸地址版本只能作为"无法解析时产出诊断"的降级路径。
3. 同地址仍有 live 绑定时到达的 Allocate 必须产出冲突诊断，并把该窗口标记为不完整
   （不得输出"完整采集"）。
4. 必须覆盖三个场景的独立用例：①G1 的 free 丢失 ②G1 的 dealloc 迟到 ③hook 与 `track!`
   双路观测同一 buffer。三者都要求：G2 拥有独立 ID、G1 的释放不删除 G2、字节守恒。
5. `total_allocations` 不得因为重复观测增长，也不得因为地址复用而漏计。

当前进度：①②③的账本层与采集层语义已按上表落地（含 `reused_address_without_release_keeps_both_generations`、
`late_release_does_not_close_reused_address`、`late_release_does_not_close_reused_async_address`
等用例）；**核心 tracker 的缓存仍是 `Map<ptr, AllocationInfo>`**，真正的 `(ptr, generation)`
落库与 `AllocationId` 分配属于 PR-02，届时 `track_deallocation` 的裸地址重载将降级为诊断路径。

### 17.9 示例实测暴露的遗漏：Analyzer 仍把"存活"当"泄漏"

用 `examples/basic_usage.rs` 验证导出数据时发现：`az.detect().leaks()` 报告
"5 leaks / 85 bytes"，而程序里 5 个对象全部存活、没有任何释放。原因在
`src/analyzer/detect.rs`：

```rust
let leaked = allocations.into_iter().filter(|a| a.ptr.is_some()).collect();
```

即"**所有堆分配都是泄漏**"——比 Dashboard 的 `is_leaked` 更激进，是 A02 的另一处活体。

处理：

- `DetectionAnalysis::leaks()` 不再下结论，返回空报告（附 debug 说明）；泄漏结论必须来自
  泄漏规则 + 采集完整性（§4.5）。
- 新增 `DetectionAnalysis::unreleased()` 保留"事实"：窗口结束时仍未释放的分配列表。
- `analyzer/core.rs` 两个断言"存活即泄漏"的测试改为新契约（存活 2 条 / 0 泄漏）。

实测（`examples/basic_usage.rs`，独立目录运行）：

| 检查点 | 结果 |
| --- | --- |
| Analyzer | Allocations 5 / Total 85 / Peak 85 / Leak 0 / Leaked 0 |
| `memory_analysis.json` | 5 条，Σsize=85，`metadata.total_size_bytes=85` |
| 尺寸明细 | `[24,20,24,4,13]`（Rc 24、Vec 20、Arc 24、Box 4、String 13），与程序内计数一致 |
| `lifetime.json` | 5 条，Σsize=85，`timestamp_dealloc` 全为 null（无伪造释放） |
| `thread_analysis.json` | 1 线程，allocation_count=5，current=85，peak=85 |
| HTML 内嵌 JSON | `event_summary.total_allocations=5`、`duplicate_observations=0`、`sampling_strategy=complete`；allocation 行 `allocation_state=live`、`leak_status=not_assessed`、`is_leaked=false`、`evidence=Observed`、`confidence=Unknown` |

### 17.10 第六轮：P1/P2 复核（预登记、搬迁 realloc、异步代号、缓存扣减、账本复杂度）

| 意见 | 核查结论 | 处理 |
| --- | --- | --- |
| P1-1 allocator 预登记导致 `track!` 记录消失 | **成立**：`Tracker::global().inner` 就是 allocator hook 写的那个核心 tracker，`track_inner` 仅凭"缓存已有同地址同大小"就 return，事件流永远缺这条 | `Tracker` 增加 `recorded_bindings: DashMap<addr, size>`，只有"本 tracker 确实为该地址发过事件"才算真正重复；hook 预登记但无事件时补发一次 `Allocate` 并记账任务内存。释放/重分配同步维护该索引 |
| P1-2 首次可见的搬迁 realloc 落在旧地址 | **成立**：`on_reallocate` 的 unmatched 分支用 `event.ptr` 开绑定，忽略 `new_ptr`，且完全没更新线程统计 | 未匹配时在新地址（`new_ptr.unwrap_or(ptr)`）开绑定，补 `attribute_allocation`（线程计数/字节/峰值），旧块不扣字节；后续释放新地址可正常匹配 |
| P1-3 异步更替记录沿用旧 generation | **成立**：删旧记录却继承 `existing.generation`，按代释放形同虚设 | 与核心 tracker 统一：同址异大小 = 新代（`generations` 计数递增），旧记录按"被取代"记入释放字节；新增 `different_size_at_same_address_starts_new_generation` 用例 |
| P2-4 异步 resize 漏记旧尺寸释放字节 | **成立**：只加 `total_memory`、调 `active_memory`，没有 `total_deallocated`，`total - deallocated != active` | 新代路径完整记账：`total_allocations += 1`、`total_memory += new`、`total_deallocated += old`、`active -= old + new`；profile 侧同样 `record_deallocation(old)` + `record_allocation(new)` |
| P2-5 核心 tracker 为未观测旧块扣减字节 | **成立**：旧地址不存在时构造 `AllocationInfo(old_size)` 后仍加 `total_deallocated`，空 tracker 收到 realloc 会得到 `active=64 / 存活 128 / 计数 0` | 未匹配分支单独处理：只记录新块（`total_allocations += 1`、`total_allocated += new`），不扣从未入账的旧字节 |
| P2-6 重建账本 O(N²) | **成立**：每次分配扫描整个 `live`，释放/realloc 线性查找加搬移 | `live` 改为墓碑式 `Vec<Option<..>>`（释放 O(1)），新增 `by_addr: HashMap<addr, Vec<index>>` 地址索引，分配/释放/重分配只访问单地址桶；`live()` 返回过滤后的引用列表 |

实测（`cargo test --release`，仅重放不同地址的分配）：

| 事件数 | 修复前（评审提供） | 修复后 |
| --- | --- | --- |
| 10,000 | 26.6 ms | 2.06 ms |
| 40,000 | 418.7 ms | 10.33 ms |

规模 4 倍时耗时约 5 倍（修复前约 15.7 倍），并新增 `replay_scales_linearly_with_distinct_addresses` 以防回退。

变异验证：把三处 P1 修复分别改回旧行为（忽略"是否已发事件"、unmatched realloc 用旧地址、异步继承旧代号）后，新增用例立即失败；恢复实现后全绿。

### 17.11 第七轮：导出层结论、护照主键与报告口径

先逐条核对，再按核对结果修复。表中"结论"为源码核查事实。

| 意见 | 结论 | 处理 |
| --- | --- | --- |
| A15-1 护照以裸地址为键，复用静默覆盖 | **成立**：`HashMap<usize, _>` + `insert`，无代号、无诊断 | `MemoryPassport` 增加 `generation`/`superseded`/`history_truncated`；被取代的护照进入 `superseded` 归档（`get_archived_passports()`），不再丢失证据；新护照带该地址的下一代号。主键完整迁移到 `AllocationId` 仍属 PR-07 |
| A15-2 `detect_leaks_at_shutdown` 查询即修改 | **成立**：写回 `status_at_shutdown` 并改 `stats`，被每次导出调用 | 改为**只读投影**：状态经 `assess_status()` 计算，不写护照、不动统计；新增"重复检测不漂移"用例 |
| A15-3 一次导出的 9 个文件不同切面 | **成立**：`memory_passports` → `leak_detection`(突变) → `unsafe_ffi` | 三个文件都改用同一 `assess_status` 只读投影，同批次口径一致 |
| A15-4 未移交 FFI 被判 `FreedByRust` | **成立**：`determine_final_status` 的 `else` 分支 | 新增 `PassportStatus::StillLive` 作为"已分配、无释放证据"的终态；`FreedByRust` 不再被凭空产出 |
| A15-5 活对象 `lifetime_ms` 当完整寿命 | **成立** | 导出改为 `lifetime_ms: null` + `observed_age_ms` + `lifetime_complete: false`；`memory_analysis.json` 同口径 |
| A15-6 截断无标记 | **成立**：`drain(0..1)` 无任何标志 | 置 `history_truncated` 并写入 `metadata`，两个护照 JSON 均输出该字段 |
| A15-7 ownership_graph 自环边 | **已消除**：A19 的自动借用已删除，残留只有显式 `track_borrow`（语义正确） | 本轮不改代码 |
| A15-8 UAF 文案当实锤 | **成立** | 文案改为 `Heuristic use-after-free: ... (heuristic, no memory-access evidence)`，算法未动 |
| B1 hook 释放后同址同大小新分配被吞 | **成立**：`recorded_bindings` 只由 `Tracker` 门面清理 | 核心缓存新增 `binding_serial`（每次新绑定递增）；去重条件改为 `(serial, size, typed)`，地址复用得到新 serial 后照常上报 |
| B2 原地 realloc 改变绑定顺序 | **成立**：关闭后追加到桶尾 | 原地分支保留原 slot 与桶内序号（只 `take` 不 `close`），迟到释放仍关闭最旧绑定 |
| B3 最终报告丢失累计统计与峰值 | **成立**：导出先取存活列表再建快照；仪表盘线程聚合按存活量重算 | 新增 `override_snapshot_stats`（导出用账本 `allocated/deallocated/current/peak/thread_stats`），仪表盘改用 `aggregate_thread_data_from_ledger` |
| B4 Arc/Rc 任务分配统计遗漏 | **成立**：只在 HeapOwner 分支补了 `record_allocation` | StackOwner 分支同样记一次 |
| B5 无类型 realloc 后首次类型观测被吞 | **成立**：重分配事件无类型信息，`track!` 被当作重复 | 记录项带 `typed` 标记；未类型化时发 `Metadata` 注释事件（`ptr != 0`），账本 `on_metadata` 把变量/类型挂到既有绑定，不新增分配 |
| B6 跨线程 realloc 的活动记到原分配线程 | **成立** | 存活字节留在原线程，`total_allocated/total_deallocated` 记到执行重分配的线程（与普通释放的 actor 口径一致） |
| B7 非逆序销毁 TaskGuard 恢复已退出作用域 | **成立**：inner 保存的 previous 未随 outer 退出失效 | 改为线程内**作用域栈**：drop 时移除自身并恢复到栈顶，任意销毁顺序都正确 |

新增/更新用例：`live_passport_is_not_reported_as_freed_by_rust`、`detect_is_read_only_and_repeatable`、`address_reuse_archives_previous_passport`、`truncated_history_is_flagged`、`reuse_after_hook_release_is_not_deduplicated`、`in_place_reallocation_keeps_binding_order`、`cross_thread_reallocation_books_activity_to_the_actor`、`smart_pointer_observation_charges_the_current_task`、`type_after_untyped_reallocation_is_attached`、`export_reports_ledger_totals_after_release`、`out_of_order_guard_drop_does_not_restore_exited_scope`。

变异验证：把去重条件退回"只看 size"后 `reuse_after_hook_release_is_not_deduplicated` 立即失败；恢复后全绿。

### 17.12 第八轮：并发首记、drop 导出统计、护照归档与上下文栈

| 意见 | 结论 | 处理 |
| --- | --- | --- |
| P1-1 并发观测重复记账 | **成立**：去重检查、事件写入、索引更新非原子，24 线程可产出多条 Allocate | `recorded_bindings` 改为"每地址一个 `Arc<Mutex<Option<RecordedBinding>>>`"，读索引→更新缓存→写事件→回写索引在同一把锁内完成。新增 24 线程用例（重复 5 次稳定通过） |
| P1-2 `with_auto_export` 仍丢累计与峰值 | **成立**：drop 路径未应用账本覆盖 | `override_snapshot_stats` 提升为 `pub(crate)` 并用于 drop 路径；新增"64+128 分配、释放 128"后 drop 导出断言 192/192/64 |
| P2-3 类型注释写到错误代际 | **成立**：`on_metadata` 取同址最旧绑定 | 先按事件 size 匹配绑定；无匹配时取该地址**最新**绑定。新增 G1/64 + G2/128 注释测试 |
| P2-4 只读评估未同步到消费者 | **成立**：`build_unsafe_reports`/`build_passport_details`/`get_stats` 仍读缓存字段 | 两者改为接收 `&MemoryPassportTracker` 并用 `assess_status`；`get_stats()` 在读取时重算 `leaks_detected` 与 `passports_by_status` |
| P2-5 任务上下文栈两处恢复错误 | **成立**：(a) 无 guard 的 `set_current_task` 父上下文丢失；(b) 按 id 移除误删最内层同 id 帧 | 帧带 guard token（按 token 移除）；`enter_task` 把外部安装的上下文压为无主基帧。新增两条用例 |
| P2-6 归档护照未进入最终报告 | **成立**：检测与导出只遍历当前表 | 检测把当前 + 归档统一成记录列表一起评估；两个护照 JSON 用 `passports_for_export()`（当前 + 归档） |
| P2-7 归档绕过护照预算 | **成立**：限额只查当前表 | 预算改为"当前 + 归档"；满额时先丢弃最旧归档，仍无空间才返回 `ResourceExhausted` |
| P2-8 截断后仍给确定状态 | **成立**：`history_truncated` 未参与评估 | `assess_status` 在链被截断时返回 `Unknown`；检测同样经该投影，因此不会再把残缺链判成泄漏 |

新增用例：`concurrent_observations_emit_one_allocation_event`、`auto_export_on_drop_reports_ledger_totals`、`metadata_annotation_targets_matching_generation`、`stats_reflect_projected_leaks`、`guard_over_external_context_restores_parent`、`reentrant_task_ids_are_removed_by_guard_identity`、`archived_passport_takes_part_in_detection`、`archive_respects_passport_budget`、`truncated_history_yields_unknown_status`。

遗留（PR-07）：护照主键完整迁移到 `AllocationId`；`export.rs`（1826 行）仍需拆出统一 `ReportSnapshot`；`memory_analysis.json` 的 `statistics.total_size_bytes` 保持"当前存活和"语义，累计值由账本口径的 `thread_analysis.json` 承载。
