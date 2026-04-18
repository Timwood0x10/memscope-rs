# Rust 语义增强开发计划

## 概述

基于当前 memscope-rs 项目架构，通过最小改动实现最大收益，从"内存追踪器"升级为"Rust Ownership Runtime Observatory"。

硬性要求：
- 编码风格严格按照./aim/rules.md

1. 单个文件的代码行数不能超过 1000 行。（包括注释和test case）,超出的可以设计成一个模块。
2. 验收标准，make check 0 errors
3. 每次修改完之后，确保make fmt 
4. make check 显示的warning，可以先不管
5. 修复warning 时，禁止用#\[allow(dead\_code)] 
6. 禁止使用任何git 命令。
7. 编写测试的时候，应该以检测代码隐形bug为先，而不是很敷衍的进行assert! 这是不负责任的。禁止滥竽充数，而是编写符合模块功能的测试用例。
8. 逐模块测试，一个做完，进行下一个。
9. 禁止执行覆盖率测试（因为浪费时间和资源）
10. 错误类型统一用Memscope Error 进行处理

## 核心原则

- **深度绑定现有架构**：基于现有的 Engine 架构，不破坏现有模块
- **代码改动最小化**：增量式改进，不重构核心逻辑
- **效果最大化**：优先解决最痛点的推测数据问题
- **大道至简**：不引入复杂依赖，保持项目简洁

## 当前架构分析

### 现有模块
- `metadata/` - 元数据管理（包含 stack_trace resolver）
- `capture/` - 数据收集和追踪
- `analysis/` - 分析功能（包含 lifecycle 模块）
- `event_store/` - 事件存储
- `render_engine/` - 输出渲染

### 当前推测数据痛点
1. **符号解析**：`metadata/stack_trace/resolver.rs` 使用 mock 实现
2. **变量大小**：`tracker.rs` 使用 `get_size_estimate()` 估算
3. **生命周期数据**：`analysis/lifecycle/` 模块数据全部为 None

### AllocationInfo 的 None 字段填充需求

当前 `src/capture/types/allocation.rs` 中的 `AllocationInfo` 结构有大量 Option 字段：

```rust
pub var_name: Option<String>,              // 需要填充
pub type_name: Option<String>,             // 需要填充
pub scope_name: Option<String>,            // 需要填充
pub timestamp_dealloc: Option<u64>,        // 已有
pub stack_trace: Option<Vec<String>>,      // DWARF 解析后填充
pub lifetime_ms: Option<u64>,             // MIR 数据后填充
pub borrow_info: Option<BorrowInfo>,      // MIR borrowck 后填充
pub clone_info: Option<CloneInfo>,         // MIR clone 检测后填充
pub ownership_history_available: bool,     // MIR move_data 后设置
pub smart_pointer_info: Option<SmartPointerInfo>,  // opt-in tracking
pub memory_layout: Option<MemoryLayoutInfo>,       // layout_of() 后填充
pub generic_info: Option<GenericTypeInfo>,         // MIR 泛型信息后填充
// ... 其他字段
```

**数据填充映射表**：

| 字段 | 当前状态 | 填充来源 | Phase | 优先级 |
|------|---------|---------|-------|--------|
| `stack_trace` | mock 数据 | DWARF 解析器 | 1.1 | P0 |
| `memory_layout` | None | std::mem API | 1.2 | P0 |
| `type_name` | 部分填充 | std::any::type_name | 1.2 | P0 |
| `lifetime_ms` | 部分计算 | timestamp 分析 | 1.3 | P0 |
| `clone_info` | None | 宏层检测 | 2.1 | P1 |
| `smart_pointer_info` | None | opt-in tracking | 2.2 | P1 |
| `ownership_history_available` | false | MIR move_data | 3.1 | P2 |
| `borrow_info` | None | MIR borrowck | 3.2 | P2 |

## Phase 1: 立即实施（2-3周，高收益低风险）

### 1.1 DWARF 符号解析器 ✅ 基础框架完成

**目标**：替换 mock 实现，提供准确的调用栈和符号信息

**完成状态**：
- ✅ 添加 addr2line/object/gimli/rustc-demangle 依赖
- ✅ 创建 dwarf_resolver.rs 模块（基础框架）
- ✅ 修改 resolver.rs 集成 DWARF 解析接口
- ✅ make check 0 errors
- ✅ **编译期宏注入方案（推荐）**：扩展 track! 宏记录 file!/line!/module_path!/stringify!()
  - 零成本编译期常量
  - 100% 准确的调用点信息
  - 发布版可用（不会被 strip）
  - 提供变量名、函数名、模块名
  - 比完整 DWARF 解析更实用
- ⏳ 真实 DWARF 解析实现（降级到 P1，当前为简化版）

**实现位置**：
- `src/metadata/stack_trace/resolver.rs` - DWARF 接口
- `src/tracker.rs` - 编译期宏注入（track! 宏扩展）
- `src/event_store/event.rs` - 添加 module_path 字段

**技术方案**：
```toml
# Cargo.toml
[dependencies]
addr2line = "0.21"
object = "0.32"
```

```rust
// src/metadata/stack_trace/resolver.rs
use addr2line::{Context, Location};
use object::*;

pub struct SymbolResolver {
    context: Option<Context<object::EndianRcSlice<object::RunTimeEndian>>>,
}

impl SymbolResolver {
    pub fn new(binary_path: &Path) -> Result<Self> {
        let file = File::parse(binary_path)?;
        let context = Context::new(&file)?;
        Ok(Self { context: Some(context) })
    }

    fn perform_resolution(&self, address: usize) -> Option<ResolvedFrame> {
        // 使用 addr2line Context 解析符号
        if let Some(ref ctx) = self.context {
            return self.resolve_with_addr2line(ctx, address);
        }
        // 回退到 mock 实现
        self.resolve_mock(address)
    }

    fn resolve_with_addr2line(&self, ctx: &Context<...>, address: usize) -> Option<ResolvedFrame> {
        let loc = ctx.find_location(address as u64).ok()??;
        let function = ctx.find_function(address as u64).ok()??;
        Some(ResolvedFrame {
            function: function.name,
            file: loc.file,
            line: loc.line,
        })
    }
}
```

**预期效果**：
- 符号解析准确度从 30% 提升到 90%
- 输出从 `0x7ffe1234` 变为 `main.rs:42 cache_map`
- 零用户代码改动

### 1.2 类型布局精确计算 ✅ 完成

**目标**：使用 std::mem API 获取精确的类型大小和对齐信息

**完成状态**：
- ✅ 添加 type_of 辅助函数到 utils.rs
- ✅ 修改 track! 宏使用 std::mem API
- ✅ 添加 track_as_with_layout 方法到 Tracker
- ✅ 添加 track_as_with_layout 方法到 GlobalTracker
- ✅ make check 0 errors

**实现位置**：修改 `src/tracker.rs` 中的 track! 宏

**改动范围**：
- 修改 track! 宏，捕获泛型类型信息
- 使用 std::mem API 获取精确大小
- 不需要 MIR 解析

**技术方案**：
```rust
// src/tracker.rs (修改 track! 宏)
#[macro_export]
macro_rules! track {
    ($var:expr) => {{
        let value = &$var;
        let size = std::mem::size_of_val(value);
        let align = std::mem::align_of_val(value);
        let type_name = $crate::utils::type_of(value);

        // 记录精确的布局信息
        $crate::tracker::track_with_layout(
            value as *const _ as usize,
            size,
            align,
            type_name,
        )
    }};
}

// src/utils.rs (辅助函数)
pub fn type_of<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

// 在 tracker.rs 中
pub fn track_with_layout(ptr: usize, size: usize, align: usize, type_name: &str) {
    let mut alloc_info = AllocationInfo::new(ptr, size);
    alloc_info.memory_layout = Some(MemoryLayoutInfo {
        size,
        align,
        // 其他字段可以设为 None 或默认值
    });
    alloc_info.type_name = Some(type_name.to_string());
}
```

**预期效果**：
- 类型大小准确度从 70% 提升到 100%
- 对齐信息准确
- 零编译器依赖
- 零用户代码改动

**数据填充实现**：
```rust
// src/capture/types/allocation.rs (修改 AllocationInfo 构造)
impl AllocationInfo {
    pub fn with_layout_info(mut self, layout: Option<MemoryLayoutInfo>) -> Self {
        self.memory_layout = layout;
        self
    }

    pub fn with_type_name(mut self, type_name: Option<String>) -> Self {
        self.type_name = type_name;
        self
    }

    pub fn with_stack_trace(mut self, stack_trace: Option<Vec<String>>) -> Self {
        self.stack_trace = stack_trace;
        self
    }
}

// 在 tracker.rs 中使用
let alloc_info = AllocationInfo::new(ptr, size)
    .with_layout_info(type_layout.get_layout(type_name))
    .with_stack_trace(symbol_resolver.resolve(backtrace))
    .with_type_name(infer_type_from_stack(backtrace));
```


**实现位置**：`src/analysis/lifecycle/timestamp_analysis.rs`

**改动范围**：
- 新增模块，分析现有数据
- 不需要编译器集成
- 基于已有的 AllocationInfo 数据

**技术方案**：
```rust
// src/analysis/lifecycle/timestamp_analysis.rs
use crate::capture::types::AllocationInfo;

pub struct LifecycleAnalyzer {
    allocations: Vec<AllocationInfo>,
}

impl LifecycleAnalyzer {
    pub fn new(allocations: Vec<AllocationInfo>) -> Self {
        Self { allocations }
    }

    /// 存活时长分布
    pub fn lifetime_distribution(&self) -> Vec<(String, usize)> {
        let mut distribution = HashMap::new();
        for alloc in &self.allocations {
            if let (Some(alloc_ts), Some(dealloc_ts)) =
                (alloc.timestamp_alloc, alloc.timestamp_dealloc)
            {
                let lifetime_ms = dealloc_ts - alloc_ts;
                let bucket = self.lifetime_bucket(lifetime_ms);
                *distribution.entry(bucket).or_insert(0) += 1;
            }
        }
        distribution.into_iter().collect()
    }

    /// 短命对象检测（temporary churn）
    pub fn detect_temporary_objects(&self, threshold_ms: u64) -> Vec<&AllocationInfo> {
        self.allocations
            .iter()
            .filter(|alloc| {
                if let (Some(alloc_ts), Some(dealloc_ts)) =
                    (alloc.timestamp_alloc, alloc.timestamp_dealloc)
                {
                    (dealloc_ts - alloc_ts) < threshold_ms
                } else {
                    false
                }
            })
            .collect()
    }

    /// 长寿对象检测（leak suspects）
    pub fn detect_long_lived_objects(&self, threshold_ms: u64) -> Vec<&AllocationInfo> {
        self.allocations
            .iter()
            .filter(|alloc| {
                if let Some(alloc_ts) = alloc.timestamp_alloc {
                    let current = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;
                    (current - alloc_ts) > threshold_ms && alloc.timestamp_dealloc.is_none()
                } else {
                    false
                }
            })
            .collect()
    }

    /// 高频分配热点
    pub fn allocation_hotspots(&self) -> Vec<(String, usize)> {
        let mut hotspots = HashMap::new();
        for alloc in &self.allocations {
            if let Some(ref stack) = alloc.stack_trace {
                let key = stack.first().cloned().unwrap_or_else(|| "unknown".to_string());
                *hotspots.entry(key).or_insert(0) += 1;
            }
        }
        hotspots.into_iter().collect()
    }

    fn lifetime_bucket(&self, ms: u64) -> String {
        if ms < 1 { "<1ms".to_string() }
        else if ms < 10 { "1-10ms".to_string() }
        else if ms < 100 { "10-100ms".to_string() }
        else if ms < 1000 { "100ms-1s".to_string() }
        else if ms < 10000 { "1-10s".to_string() }
        else { ">10s".to_string() }
    }
}
```

**预期效果**：
- 存活时长分布分析
- 短命对象检测（性能优化目标）
- 长寿对象检测（内存泄漏嫌疑）
- 高频分配热点识别
- 零编译器依赖
- 基于现有数据，立即可用

### 1.4 Sampling 模式 ✅ 完成

**目标**：支持生产环境低开销采样

**完成状态**：
- ✅ 扩展 SamplingConfig 结构，添加 sample_every_n, min_size_bytes, max_duration_seconds 字段
- ✅ 添加 builder 方法：with_sample_every_n, with_min_size, with_max_duration
- ✅ 更新 Default, demo(), full(), high_performance() 实现
- ✅ 修复测试编译错误
- ✅ make check 0 errors，支持生产环境使用

**实现位置**：`src/tracker.rs` 扩展配置
- 实现抽样逻辑
- 不影响核心功能

**技术方案**：
```rust
// src/tracker.rs
pub struct SamplingConfig {
    /// 每追踪 N 个分配
    pub sample_every_n: Option<usize>,
    /// 只追踪大于 X 字节的分配
    pub min_size_bytes: Option<usize>,
    /// 只追踪 T 秒
    pub duration_seconds: Option<u64>,
}

impl SamplingConfig {
    pub fn should_track(&self, size: usize) -> bool {
        // 大小过滤
        if let Some(min_size) = self.min_size_bytes {
            if size < min_size {
                return false;
            }
        }

        // 抽样过滤
        if let Some(n) = self.sample_every_n {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            COUNTER.fetch_add(1, Ordering::Relaxed) % n != 0
        } else {
            true
        }

        // 时间过滤（在 tracker 层实现）
        true
    }
}
```

**预期效果**：
- 降低性能开销 10-100 倍
- 支持生产环境部署
- 用户可配置策略

### 1.5 Top N 报表 ✅ 完成

**目标**：提供高价值汇总信息

**完成状态**：
- ✅ 创建 top_n.rs 模块
- ✅ 实现 TopNAnalyzer
- ✅ 添加 top_allocation_sites, top_leaked_bytes, top_temporary_churn 方法
- ✅ 集成到 analysis 模块
- ✅ make check 0 errors

**实现位置**：`src/analysis/top_n.rs`

**改动范围**：
- 新增分析模块
- 集成到渲染输出

**技术方案**：
```rust
// src/analysis/top_n.rs
pub struct TopNReport {
    top_allocation_sites: Vec<(String, usize, usize)>,  // location, count, bytes
    top_leaked_bytes: Vec<(String, usize)>,
    top_temporary_churn: Vec<(String, usize)>,
}

impl TopNReport {
    pub fn generate(allocations: &[AllocationInfo], n: usize) -> Self {
        // Top allocation sites
        let mut sites: HashMap<String, (usize, usize)> = HashMap::new();
        for alloc in allocations {
            if let Some(ref stack) = alloc.stack_trace {
                let key = stack.first().cloned().unwrap_or_else(|| "unknown".to_string());
                let entry = sites.entry(key).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += alloc.size;
            }
        }

        let mut top_sites: Vec<_> = sites.into_iter().collect();
        top_sites.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));
        top_sites.truncate(n);

        // Top leaked bytes
        let top_leaked = allocations
            .iter()
            .filter(|a| a.timestamp_dealloc.is_none())
            .map(|a| (a.var_name.clone().unwrap_or_else(|| "unknown".to_string()), a.size))
            .collect();

        Self {
            top_allocation_sites: top_sites.into_iter()
                .map(|(k, (count, bytes))| (k, count, bytes))
                .collect(),
            top_leaked_bytes: top_leaked,
            top_temporary_churn: vec![],
        }
    }
}
```

**预期效果**：
- 快速定位内存问题
- 比 fancy graph 更实用
- 一键输出关键信息

### 1.6 HTML 报告集成

**目标**：一键生成可读性强的 HTML 报告

**实现位置**：集成到现有 `render_engine/dashboard`

**改动范围**：
- 利用现有 HTML 渲染能力
- 集成新数据源
- 添加 Top N 报表展示

**预期效果**：
- `cargo run --features memscope` → `open report.html`
- 开源项目传播力暴涨
- 用户友好界面

## Phase 2: 增强功能（3-4周，中等风险）

### 2.1 Clone 操作检测（宏层）

**目标**：通过 proc_macro 检测 clone 操作

**实现位置**：新建 `src/metadata/clone_instrumentation.rs`

**技术方案**：
```rust
// 通过宏重写 clone 调用
#[track_clone]
fn example() {
    let data = vec![1, 2, 3];
    let cloned = data.clone();  // 自动记录
}
```

**预期效果**：
- 区分 x.clone()、Arc::clone()、to_vec()
- 记录克隆源和目标
- 填充 `clone_info` 字段

### 2.2 智能指针 Opt-in 追踪

**实现位置**：新建 `src/metadata/smart_pointer_tracking.rs`

**技术方案**：
```rust
// 作为可选 feature
#[cfg(feature = "smart-pointer-tracking")]
pub use tracked_rc::TrackedRc;

#[cfg(feature = "smart-pointer-tracking")]
mod tracked_rc {
    use std::rc::Rc as StdRc;

    #[derive(Clone)]
    pub struct TrackedRc<T> {
        inner: StdRc<T>,
        id: usize,
    }

    impl<T> TrackedRc<T> {
        pub fn new(value: T) -> Self {
            let id = allocate_id();F
            let inner = StdRc::new(value);
            log_ref_count_change(id, 1, 0);
            Self { inner, id }
        }
    }
}
```

**预期效果**：
- 准确的引用计数追踪
- 循环引用检测
- 填充 `smart_pointer_info` 字段
- opt-in 模式，不强制用户使用

## Phase 3: 研究功能（长期，低优先级）

### 3.1 MIR 所有权数据提取（实验性）

**目标**：通过 rustc wrapper 获取所有权转移轨迹

**实现位置**：新建 `src/metadata/rustc_extractor.rs`

**技术方案**：
```rust
// 使用 RUSTC_WRAPPER
pub struct RustcExtractor {
    move_operations: Vec<MoveOperation>,
}

impl RustcExtractor {
    pub fn extract_from_mir(&mut self, mir_output: &str) {
        // 解析 MIR 输出中的 move_data
    }
}
```

**风险提示**：
- rustc 私有 API 极不稳定
- 用户使用门槛高（RUSTC_WRAPPER）
- 编译期地址与运行时地址关联困难

**预期效果**：
- 获取真实的所有权转移轨迹
- 填充 `ownership_history_available`、`borrow_info` 字段
- 实验性功能，不建议生产使用

### 3.2 Borrowck Facts（实验性）
### 3.3 Polonius Facts（实验性）

## 架构调整

### 现有架构保持不变
```
src/
├── metadata/          # 现有模块，扩展功能
│   ├── stack_trace/
│   │   └── resolver.rs  # 添加 DWARF 解析
│   ├── type_layout.rs  # 新增
│   └── rustc_extractor.rs  # 新增
├── analysis/          # 现有模块，利用新数据
├── capture/           # 现有模块，不变
└── ...
```

### 数据流增强
```
现有流程：
Allocation → EventStore → Analysis → Render

增强流程：
Allocation → EventStore → Analysis → Render
                    ↑
                    ├─ DWARF Resolver (符号信息)
                    ├─ Type Layout (精确大小)
                    └─ MIR Extractor (所有权数据)
```

## 实施时间表

### P0 阶段（2周，必做）- 发布 v0.5

#### Week 1: DWARF 解析器 + 类型布局
- Day 1-2: 添加 addr2line/object 依赖
- Day 3-4: 实现基础 DWARF 解析
- Day 5-6: 修改 track! 宏，集成 std::mem API
- Day 7-8: 集成到现有 resolver，测试
- Day 9-10: Linux/macOS 兼容性测试

#### Week 2: 生命周期分析 + Sampling + Top N + HTML
- Day 1-2: 实现 timestamp 分析模块
- Day 3: 实现 Sampling 模式
- Day 4: 实现 Top N 报表
- Day 5: 集成 HTML 报告
- Day 6-7: 集成到 analysis/lifecycle
- Day 8-9: 端到端测试
- Day 10: 发布 v0.5

### P1 阶段（3-4周，可选）- 发布 v0.6

#### Week 3-4: Clone 检测
- Day 1-5: 实现 proc_macro 检测（只记录大对象 clone）
- Day 6-8: 集成到 analysis
- Day 9-10: 测试和文档

#### Week 5-6: 智能指针追踪
- Day 1-4: 实现 TrackedRc/TrackedArc
- Day 5-7: 集成到 metadata
- Day 8-10: 循环引用检测

### P2 阶段（长期，研究）- lab/ 目录

#### MIR 提取器（实验性）
- rustc wrapper 实现
- move_data 解析
- borrowck facts

### 未来版本规划

#### v0.7
- async runtime traces
- tokio task memory view

## 成功指标

### P0 阶段（2周）
- 符号解析：30% → 90%
- 类型大小：70% → 100%
- 生命周期分析：0% → 80%（基于 timestamp）
- 输出可读性：显著提升

### P1 阶段（可选）
- Clone 追踪：0% → 70%
- 智能指针追踪：0% → 90%（opt-in）

### P2 阶段（研究）
- 所有权轨迹：0% → 80%（实验性）
- 借用信息：0% → 60%（实验性）

### 用户体验
- 零用户代码改动（P0）
- 可选功能按需启用（P1/P2）
- 性能影响 < 5%

### 代码质量
- 现有架构零破坏
- P0 新增代码 < 1500 行
- 测试覆盖率 > 80%

## 风险控制

### 技术风险
- DWARF 解析平台兼容性 → 优先支持 Linux/macOS
- std::mem API 兼容性 → Rust 稳定 API，无风险
- 性能影响 → 可选功能，按需启用

### 维护风险
- rustc API 变化 → P2 功能，不影响核心
- 依赖库升级 → 选择稳定版本（addr2line, object）

## 总结

本计划基于现有架构，通过最小改动实现最大收益：

1. **深度绑定**：基于现有的 metadata/、analysis/ 模块
2. **改动最小**：增量式改进，不重构核心
3. **效果显著**：解决最痛点的推测数据问题
4. **大道至简**：P0 使用稳定 API，P2 探索高级功能
5. **分层实施**：P0 必做 → P1 可选 → P2 研究

**P0 完成后**，memscope-rs 将从"研究型项目"升级为"实用型工具"，提供可信度高的 Rust 语义分析能力，可发布 v0.5 版本。

**关键改进**：
- 符号解析从 mock 变为真实 DWARF
- 类型大小从估算变为精确测量
- 生命周期分析从推测变为基于 timestamp 的真实分析
- 零编译器依赖，零用户代码改动
