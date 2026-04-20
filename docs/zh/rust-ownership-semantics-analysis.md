# Rust Move/Borrow 语义深度分析：从源码到LLVM的完整流程

## 概述

本文档深入分析 Rust 的 move/borrow 语义在编译器内部的完整处理流程，从源码定义到 LLVM IR 生成的全过程，以及为什么这些语义难以从外部工具捕捉。

## 目录

1. [编译器中间表示流程](#编译器中间表示流程)
2. [HIR 与类型信息存储](#hir-与类型信息存储)
3. [THIR 与 Type Checking 的关系](#thir-与-type-checking-的关系)
4. [Borrow 语义分析的依赖组件](#borrow-语义分析的依赖组件)
5. [Move/Copy/Borrow 语义的显式化过程](#movecopyborrow-语义的显式化过程)
6. [为什么 Move/Borrow 难以捕捉](#为什么-moveborrow-难以捕捉)
7. [解决方案对比](#解决方案对比)

---

## 编译器中间表示流程

```
源码 (Source Code)
    ↓
AST (Abstract Syntax Tree)
    ↓
HIR (High-level IR)
    ↓
THIR (Typed HIR) ← 类型检查在这里完成
    ↓
MIR (Mid-level IR) ← move/copy/borrow 语义在这里显式化
    ↓
LLVM IR ← 语义消失，变成普通内存操作
    ↓
机器码
```

### 关键阶段特征

| 阶段 | 类型信息 | Move/Copy/Borrow 语义 | 访问方式 |
|------|---------|-------------------|---------|
| 源码 | ❌ 无 | ❌ 无语义标记 | 公开 |
| AST | ❌ 无 | ❌ 无语义标记 | 公开 |
| HIR | ❌ 无 | ❌ 无语义标记 | 公开 |
| THIR | ✅ 有（来自 typeck） | ❌ 未显式化 | 公开 |
| MIR | ✅ 有 | ✅ **显式化** | **私有** |
| LLVM IR | ✅ 有 | ❌ **语义消失** | 公开 |

---

## HIR 与类型信息存储

### 核心观点

> HIR 节点本身主要保留语法结构，类型结果存放在 typeck tables / tcx 查询系统中。

### 源码验证

**文件**: `rustc_middle/src/ty/typeck_results.rs:31`

```rust
#[derive(TyEncodable, TyDecodable, Debug, HashStable)]
pub struct TypeckResults<'tcx> {
    /// The `HirId::owner` all `ItemLocalId`s in this table are relative to.
    pub hir_owner: OwnerId,
    
    /// Resolved definitions for `<T>::X` associated paths and
    /// method calls, including those of overloaded operators.
    type_dependent_defs: ItemLocalMap<Result<(DefKind, DefId), ErrorGuaranteed>>,
    
    /// Stores the types for various nodes in the AST.
    node_types: ItemLocalMap<Ty<'tcx>>,  // ← 类型信息存储在这里
    
    /// Stores the type parameters which were instantiated
    node_args: ItemLocalMap<GenericArgsRef<'tcx>>,
    
    /// Adjustments applied to expressions
    adjustments: ItemLocalMap<Vec<ty::adjustment::Adjustment<'tcx>>>,
    
    /// Stores the actual binding mode for all instances of [`BindingMode`].
    pat_binding_modes: ItemLocalMap<BindingMode>,
    
    // ... 其他字段
}
```

### 关键发现

- ✅ **HIR 节点本身只保留语法结构** - 不包含类型信息
- ✅ **类型信息存储在 `TypeckResults` 中** - 通过 `ItemLocalMap` 索引
- ✅ **通过 tcx 查询系统访问** - `tcx.typeck(def)` 获取类型检查结果
- ✅ **类型信息与 HIR 节点分离** - 允许类型检查独立于语法分析

### 架构意义

这种分离设计使得：
1. 类型检查可以独立进行
2. 类型信息可以被多个阶段共享
3. 类型系统可以独立演进

---

## THIR 与 Type Checking 的关系

### 核心观点

> Type checking occurs before/during THIR construction pipeline. THIR 是 typed lowered representation，消费类型结果。

### 源码验证

**文件**: `rustc_mir_build/src/thir/cx/mod.rs:16-50`

```rust
/// Query implementation for [`TyCtxt::thir_body`].
pub(crate) fn thir_body<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner_def: LocalDefId,
) -> Result<(&'tcx Steal<Thir<'tcx>>, ExprId), ErrorGuaranteed> {
    debug_assert!(!tcx.is_type_const(owner_def.to_def_id()), "thir_body queried for type_const");

    let body = tcx.hir_body_owned_by(owner_def);
    let mut cx: ThirBuildCx<'tcx> = ThirBuildCx::new(tcx, owner_def);
    
    // 关键：先获取 typeck_results
    if let Some(reported) = cx.typeck_results.tainted_by_errors {
        return Err(reported);
    }

    // Lower the params before the body's expression so errors from params are shown first.
    let owner_id = tcx.local_def_id_to_hir_id(owner_def);
    if let Some(fn_decl) = tcx.hir_fn_decl_by_hir_id(owner_id) {
        let closure_env_param = cx.closure_env_param(owner_def, owner_id);
        let explicit_params = cx.explicit_params(owner_id, fn_decl, &body);
        cx.thir.params = closure_env_param.into_iter().chain(explicit_params).collect();
        // ...
    }

    // 然后构建 THIR
    let expr = cx.mirror_expr(body.value);
    Ok((tcx.alloc_steal_thir(cx.thir), expr))
}
```

**文件**: `rustc_mir_build/src/thir/cx/mod.rs:70-71`

```rust
fn new(tcx: TyCtxt<'tcx>, def: LocalDefId) -> Self {
    let typeck_results = tcx.typeck(def);  // ← 先获取类型检查结果
    let hir_id = tcx.local_def_id_to_hir_id(def);
    // ...
}
```

**文件**: `rustc_mir_build/src/thir/cx/mod.rs:60`

```rust
pub(crate) struct ThirBuildCx<'tcx> {
    tcx: TyCtxt<'tcx>,
    /// The THIR data that this context is building.
    thir: Thir<'tcx>,
    
    typing_env: ty::TypingEnv<'tcx>,
    
    typeck_results: &'tcx ty::TypeckResults<'tcx>,  // ← 消费类型结果
    
    /// False to indicate that adjustments should not be applied.
    apply_adjustments: bool,
    
    /// The `DefId` of the owner of this body.
    body_owner: DefId,
}
```

### 关键发现

- ✅ **Type checking 发生在 THIR construction 之前** - `tcx.typeck(def)` 先执行
- ✅ **THIR 是 typed lowered representation** - 消费类型检查结果
- ✅ **THIR 构建时消费 typeck_results** - 通过引用持有
- ✅ **类型信息通过 `typing_env` 传递** - 用于类型查询

### 流程顺序

```
1. HIR 解析
    ↓
2. Type checking (生成 TypeckResults)
    ↓
3. THIR construction (消费 TypeckResults)
    ↓
4. MIR building (消费 THIR)
```

---

## Borrow 语义分析的依赖组件

### 核心观点

> Borrow 语义真实分析还依赖：
> - BorrowSet
> - Region inference
> - NLL facts
> - Two-phase borrow state

### 源码验证

#### 1. BorrowSet

**文件**: `rustc_borrowck/src/lib.rs:338`

```rust
let move_data = MoveData::gather_moves(body, tcx, |_| true);
let locals_are_invalidated_at_exit = tcx.hir_body_owner_kind(def).is_fn_or_closure();
let borrow_set = BorrowSet::build(tcx, body, locals_are_invalidated_at_exit, &move_data);
```

**文件**: `rustc_borrowck/src/borrow_set.rs:66-70`

```rust
/// Location where a two-phase borrow is activated, if a borrow
/// is in fact a two-phase borrow.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TwoPhaseActivation {
    NotTwoPhase,
    NotActivated,
    ActivatedAt(Location),
}
```

**作用**: 追踪所有的 borrows 及其位置

#### 2. Region Inference

**文件**: `rustc_borrowck/src/nll.rs:29`

```rust
use crate::region_infer::RegionInferenceContext;
```

**文件**: `rustc_borrowck/src/nll.rs:38-46`

```rust
/// The output of `nll::compute_regions`. This includes the computed `RegionInferenceContext`, any
/// closure requirements to propagate, and any generated errors.
pub(crate) struct NllOutput<'tcx> {
    pub regioncx: RegionInferenceContext<'tcx>,  // ← Region inference context
    pub polonius_input: Option<Box<PoloniusFacts>>,
    pub polonius_output: Option<Box<PoloniusOutput>>,
    pub opt_closure_req: Option<ClosureRegionRequirements<'tcx>>,
    pub nll_errors: RegionErrors<'tcx>,
}
```

**作用**: 推断生命周期和区域约束

#### 3. NLL Facts

**文件**: `rustc_borrowck/src/nll.rs:162`

```rust
// If requested: dump NLL facts, and run legacy polonius analysis.
let polonius_output = polonius_facts.as_ref().and_then(|polonius_facts| {
    if infcx.tcx.sess.opts.unstable_opts.nll_facts {
        let def_id = body.source.def_id();
        let def_path = infcx.tcx.def_path(def_id);
        let dir_path = PathBuf::from(&infcx.tcx.sess.opts.unstable_opts.nll_facts_dir)
            .join(def_path.to_filename_friendly_no_crate());
        polonius_facts.write_to_dir(dir_path, location_table).unwrap();
    }
    // ...
});
```

**文件**: `rustc_borrowck/src/polonius/legacy/facts.rs:54-61`

```rust
#[extension(pub(crate) trait PoloniusFactsExt)]
impl PoloniusFacts {
    /// Returns `true` if there is a need to gather `PoloniusFacts` given the
    /// current `-Z` flags.
    fn enabled(tcx: TyCtxt<'_>) -> bool {
        tcx.sess.opts.unstable_opts.nll_facts  // ← NLL facts 选项
            || tcx.sess.opts.unstable_opts.polonius.is_legacy_enabled()
    }
    // ...
}
```

**作用**: Polonius 分析的事实数据，用于更精确的借用检查

#### 4. Two-phase Borrow State

**文件**: `rustc_borrowck/src/borrow_set.rs:66-70`

```rust
pub enum TwoPhaseActivation {
    NotTwoPhase,
    NotActivated,
    ActivatedAt(Location),
}
```

**文件**: `rustc_borrowck/src/places_conflict.rs:87-97`

```rust
pub(crate) fn is_mutable_borrow(
    tcx: TyCtxt<'tcx>,
    body: &Body<'tcx>,
    borrow_place: Place<'tcx>,
    access_place: Place<'tcx>,
    bias: AccessDirection,
) -> bool {
    borrow_conflicts_with_place(
        tcx,
        body,
        borrow_place,
        BorrowKind::Mut { kind: MutBorrowKind::TwoPhaseBorrow },  // ← Two-phase borrow
        access_place.as_ref(),
        AccessDepth::Deep,
        bias,
    )
}
```

**作用**: 处理两阶段借用（先预留后激活）

### 完整的 Borrow 分析流程

**文件**: `rustc_borrowck/src/lib.rs:335-366`

```rust
// 1. 收集 move 信息
let move_data = MoveData::gather_moves(body, tcx, |_| true);

// 2. 构建 BorrowSet
let locals_are_invalidated_at_exit = tcx.hir_body_owner_kind(def).is_fn_or_closure();
let borrow_set = BorrowSet::build(tcx, body, locals_are_invalidated_at_exit, &move_data);

// 3. 准备 Polonius facts（如果启用）
let location_table = PoloniusLocationTable::new(body);
let location_map = Rc::new(DenseLocationMap::new(body));

let polonius_input = root_cx.consumer.as_ref().map_or(false, |c| c.polonius_input())
    || infcx.tcx.sess.opts.unstable_opts.polonius.is_legacy_enabled();
let mut polonius_facts =
    (polonius_input || PoloniusFacts::enabled(infcx.tcx)).then_some(PoloniusFacts::default());

// 4. 运行 MIR type-checker
let MirTypeckResults {
    constraints,
    universal_region_relations,
    region_bound_pairs,
    known_type_outlives_obligations,
    deferred_closure_requirements,
    polonius_context,
} = type_check::type_check(
    root_cx,
    &infcx,
    body,
    &promoted,
    universal_regions,
    &location_table,
    &borrow_set,  // ← 依赖 BorrowSet
    &mut polonius_facts,  // ← 依赖 NLL facts
    &move_data,
    Rc::clone(&location_map),
);
```

---

## Move/Copy/Borrow 语义的显式化过程

### 关键决策点

**文件**: `rustc_mir_build/src/builder/misc.rs:61-68`

```rust
pub(crate) fn consume_by_copy_or_move(&self, place: Place<'tcx>) -> Operand<'tcx> {
    let tcx = self.tcx;
    let ty = place.ty(&self.local_decls, tcx).ty;
    if self.infcx.type_is_copy_modulo_regions(self.param_env, ty) {
        Operand::Copy(place)  // ← 类型是 Copy → Copy 语义
    } else {
        Operand::Move(place)  // ← 类型不是 Copy → Move 语义
    }
}
```

### 决策逻辑

```
检查类型是否实现 Copy trait
    ↓
  是 Copy
    ↓
Operand::Copy(place)
    ↓
  不是 Copy
    ↓
Operand::Move(place)
```

### MIR 中的显式表示

**文件**: `rustc_middle/src/mir/syntax.rs:1308-1332`

```rust
#[derive(Clone, PartialEq, TyEncodable, TyDecodable, Hash, HashStable, TypeFoldable, TypeVisitable)]
pub enum Operand<'tcx> {
    /// Creates a value by loading the given place.
    ///
    /// Before drop elaboration, the type of the place must be `Copy`. After drop elaboration there
    /// is no such requirement.
    Copy(Place<'tcx>),  // ← Copy 语义

    /// Creates a value by performing loading the place, just like the `Copy` operand.
    ///
    /// This *may* additionally overwrite the place with `uninit` bytes, depending on how we decide
    /// in [UCG#188]. You should not emit MIR that may attempt a subsequent second load of this
    /// place without first re-initializing it.
    ///
    /// **Needs clarification:** The operational impact of `Move` is unclear. Currently (both in
    /// Miri and codegen) it has no effect at all unless it appears in an argument to `Call`; for
    /// `Call` it allows the argument to be passed to the callee "in-place", i.e. the callee might
    /// just get a reference to this place instead of a full copy. Miri implements this with a
    /// combination of aliasing model "protectors" and putting `uninit` into the place. Ralf
    /// proposes that we don't want these semantics for `Move` in regular assignments, because
    /// loading a place should not have side-effects, and the aliasing model "protectors" are
    /// inherently tied to a function call. Are these the semantics we want for MIR? Is this
    /// something we can even decide without knowing more about Rust's memory model?
    ///
    /// [UCG#188]: https://github.com/rust-lang/unsafe-code-guidelines/issues/188
    Move(Place<'tcx>),  // ← Move 语义

    /// Constants are already semantically values, and remain unchanged.
    Constant(Box<ConstOperand<'tcx>>),

    /// Query the compilation session of the current crate for a particular flag. This is not quite
    /// a const since its value can differ across crates within a single crate graph.
    RuntimeChecks(RuntimeChecks),
}
```

**文件**: `rustc_middle/src/mir/syntax.rs:1384`

```rust
/// Creates a reference of the indicated kind to the place.
///
/// There is not much to document here, because besides the obvious parts the semantics of this
/// are essentially entirely a part of the aliasing model. There are many UCG issues discussing
/// exactly what the behavior of this operation should be.
///
/// `Shallow` borrows are disallowed after drop lowering.
Ref(Region<'tcx>, BorrowKind, Place<'tcx>),  // ← Borrow 语义
```

### 各阶段的语义表示

| 阶段 | Move 表示 | Copy 表示 | Borrow 表示 |
|------|----------|----------|------------|
| 源码 | `let y = x` | `let y = x` | `let y = &x` |
| HIR | `Assign` | `Assign` | `AddrOf` |
| THIR | `Assign` | `Assign` | `ExprKind::AddrOf` |
| MIR | `Operand::Move` | `Operand::Copy` | `Rvalue::Ref` |
| LLVM IR | `store` | `store` | `getelementptr` |

---

## 为什么 Move/Borrow 难以捕捉

### 根本原因

**Move/Copy/Borrow 语义在 MIR 阶段才显式化，但 MIR API 是私有的**

```
源码 → HIR → THIR → MIR → LLVM IR
     ↑      ↑     ↑     ↑      ↑
    公开   公开  公开  私有   公开
     ↓      ↓     ↓     ↓      ↓
   无语义  无语义  无语义  有语义  语义消失
```

### 具体难点

#### 1. 源码层面：无语义标记

```rust
let y = x;  // ← 你不知道这是 move 还是 copy
```

需要类型信息才能判断，但源码解析阶段没有类型信息。

#### 2. AST/HIR 层面：无类型信息

```rust
// AST 只能看到语法树
Assign {
    target: Var("y"),
    source: Var("x"),
}
// 无法知道 x 的类型，无法判断 move/copy
```

#### 3. THIR 层面：有类型但无显式语义

```rust
// THIR 有类型信息
Expr {
    kind: Assign { target: ..., source: ... },
    ty: String,  // ← 有类型了
}
// 但仍然没有显式的 move/copy 标记
```

#### 4. MIR 层面：语义显式化但需要 rustc API

```rust
// MIR 有显式语义
Operand::Move(place)  // ← 明确的 move
// 但需要 rustc 内部API 才能访问
```

#### 5. LLVM 层面：语义消失

```llvm
; move 语义变成普通内存操作
%2 = alloca %String*
store %String* %1, %String** %2
; 无法区分 move 和 copy
```

### 类型推断的复杂性

```rust
fn example<T>(x: T) {
    let y = x;  // ← T 是否 Copy？需要类型推断
}
```

类型推断发生在编译期，外部工具无法访问。

### 生命周期的影响

```rust
fn example<'a>(x: &'a String) -> &'a String {
    x  // ← 借用的生命周期如何影响 move？
}
```

生命周期分析也是编译期的，外部工具难以捕捉。

### 优化的影响

```rust
let x = String::from("hello");
let y = x;  // move
use(y);     // 编译器可能优化掉这个 move
```

优化后，move 可能被消除，运行时无法检测。

---

## 解决方案对比

| 方案 | 可捕捉的阶段 | 优点 | 缺点 | 稳定性 |
|------|------------|------|------|--------|
| **源码分析 (syn)** | 源码/AST | 不依赖 rustc，稳定 | 无类型信息，不精确 | ⭐⭐⭐⭐⭐ |
| **THIR 解析** | THIR | 有类型信息 | 语义未显式化 | ⭐⭐⭐⭐ |
| **MIR 解析 (rustc driver)** | MIR | 信息完整，精确 | 需要 rustc API，不稳定 | ⭐⭐ |
| **MIR 文本解析** | MIR 文本 | 不依赖 rustc API | 格式不稳定，类型信息缺失 | ⭐⭐ |
| **二进制分析 (gimli)** | LLVM IR | 完全独立，零侵入 | 语义已消失 | ⭐⭐⭐⭐⭐ |
| **运行时追踪** | 运行时 | 实际行为 | 优化后可能丢失信息 | ⭐⭐⭐ |
| **混合方案 (AST + 运行时)** | 源码 + 运行时 | 互补性强 | 实现复杂 | ⭐⭐⭐⭐ |

### 推荐方案

#### 优先级 1：源码静态分析（syn 库）

```rust
use syn::{parse_file, Item, Expr};

fn analyze_ownership(source: &str) -> Vec<<OwnershipEvent> {
    let ast = parse_file(source).unwrap();
    let mut events = Vec::new();
    
    for item in &ast.items {
        if let Item::Fn(func) = item {
            analyze_function(func, &mut events);
        }
    }
    
    events
}
```

**优点**：
- ✅ 完全不依赖 rustc
- ✅ 使用稳定的 syn 库
- ✅ 可以在 stable Rust 上运行
- ✅ 零运行时开销

**缺点**：
- ⚠️ 需要类型推断才能准确判断 move
- ⚠️ 宏、泛型处理复杂
- ⚠️ 不如 MIR 精确

#### 优先级 2：二进制分析（gimli + object）

```rust
use object::{Object, ObjectSection};
use gimli::*;

fn analyze_binary(binary_path: &Path) -> Result<<OwnershipGraph>> {
    let file = std::fs::File::open(binary_path)?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let object = Object::parse(&mmap)?;
    
    // 解析 DWARF 获取函数和变量信息
    let dwarf = Dwarf::load(|section| {
        object.section_by_name(section)
            .and_then(|s| s.uncompressed_data().ok())
    })?;
    
    // 解析 .text 段获取代码
    let text_section = object.section_by_name(".text")?;
    
    // 分析函数调用图和所有权转移模式
    let call_graph = build_call_graph(&text_section, &dwarf)?;
    let ownership_events = identify_ownership_patterns(&text_section, &dwarf)?;
    
    Ok(OwnershipGraph {
        call_graph,
        ownership_events,
    })
}
```

**优点**：
- ✅ 完全零侵入
- ✅ 不需要 nightly
- ✅ 可以分析任意二进制
- ✅ 零运行时开销

**缺点**：
- ❌ 信息不如 MIR 精确
- ❌ 需要理解机器码
- ❌ 优化后可能丢失信息

#### 优先级 3：混合方案（AST + 运行时）

```rust
pub struct HybridOwnershipAnalyzer {
    ast_analyzer: ASTOwnershipAnalyzer,
    runtime_tracker: RuntimeOwnershipTracker,
    fused_events: Vec<<OwnershipEvent>>,
}

impl HybridOwnershipAnalyzer {
    pub fn analyze(&mut self) -> Result<Vec<<OwnershipEvent>> {
        // 1. AST 分析（编译期）
        let ast_events = self.ast_analyzer.analyze()?;
        
        // 2. 运行时追踪（需要实际运行程序）
        let runtime_events = self.runtime_tracker.track()?;
        
        // 3. 融合两种分析
        self.fuse_events(ast_events, runtime_events)
    }
}
```

**优点**：
- ✅ 互补性强
- ✅ 有 fallback 机制
- ✅ 不依赖 rustc

**缺点**：
- ⚠️ 实现复杂度较高

---

## 结论

### 核心发现

1. **HIR 节点本身主要保留语法结构，类型结果存放在 typeck tables / tcx 查询系统中**
   - ✅ 验证正确：类型信息存储在 `TypeckResults` 中，通过 tcx 查询

2. **Type checking occurs before/during THIR construction pipeline. THIR 是 typed lowered representation，消费类型结果**
   - ✅ 验证正确：THIR 构建时消费 `typeck_results`

3. **Borrow 语义真实分析还依赖：BorrowSet、Region inference、NLL facts、Two-phase borrow state**
   - ✅ 验证正确：所有组件都在 borrowck 中使用

### Move/Borrow 难以捕捉的根本原因

1. **语义在 MIR 阶段才显式化** - 之前阶段没有明确标记
2. **MIR API 是私有的** - 外部工具无法稳定访问
3. **类型推断和生命周期** - 需要编译期信息
4. **优化的影响** - 运行时信息可能不完整

### 最佳实践建议

1. **优先使用源码静态分析** - 稳定且实现简单
2. **二进制分析作为 fallback** - 适用于第三方二进制
3. **混合方案提高精确度** - 结合编译期和运行时信息
4. **避免依赖 rustc 内部 API** - 保证长期稳定性

---

## 参考文献

### Rust 编译器源码

- `rustc_middle/src/ty/typeck_results.rs` - 类型检查结果存储
- `rustc_mir_build/src/thir/cx/mod.rs` - THIR 构建上下文
- `rustc_mir_build/src/builder/misc.rs` - Move/Copy 决策
- `rustc_middle/src/mir/syntax.rs` - MIR 语法定义
- `rustc_borrowck/src/lib.rs` - Borrow check 主流程
- `rustc_borrowck/src/borrow_set.rs` - BorrowSet 定义
- `rustc_borrowck/src/nll.rs` - Non-lexical lifetimes
- `rustc_mir_dataflow/src/move_paths/mod.rs` - Move path 追踪

### 相关文档

- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) - Rust 内存模型
- [Rust Reference](https://doc.rust-lang.org/reference/) - Rust 语言参考
- [Rustc Development Guide](https://rustc-dev-guide.rust-lang.org/) - 编译器开发指南

---

**文档版本**: 1.0  
**最后更新**: 2026-04-19  
**基于 Rust 源码版本**: 最新稳定版
