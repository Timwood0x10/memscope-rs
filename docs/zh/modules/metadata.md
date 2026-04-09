# 元数据模块 (Metadata Module)

## 概述

元数据模块为 memscope-rs 系统提供集中式元数据管理。它管理系统中的所有元数据，包括变量、作用域、线程、类型和指针，提供访问和更新元数据的统一接口。

## 组件

### 1. MetadataEngine（元数据引擎）

**文件**: `src/metadata/engine.rs`

**用途**: 集中式元数据管理引擎，协调所有元数据组件。

**核心功能**:
- 集中化：所有元数据的单一事实来源
- 线程安全：所有操作通过 Arc 实现线程安全
- 高效：针对快速查找和更新进行优化
- 统一：所有元数据类型的单一接口

**核心实现**:

```rust
pub struct MetadataEngine {
    /// 变量注册表
    pub variable_registry: Arc<VariableRegistry>,
    /// 作用域追踪器
    pub scope_tracker: Arc<ScopeTracker>,
    /// 线程注册表
    pub thread_registry: Arc<ThreadRegistry>,
}

impl MetadataEngine {
    /// 创建新的 MetadataEngine
    pub fn new() -> Self {
        Self {
            variable_registry: Arc::new(VariableRegistry::new()),
            scope_tracker: Arc::new(ScopeTracker::new()),
            thread_registry: Arc::new(ThreadRegistry::new()),
        }
    }

    /// 获取变量注册表
    pub fn variables(&self) -> &Arc<VariableRegistry> {
        &self.variable_registry
    }

    /// 获取作用域追踪器
    pub fn scopes(&self) -> &Arc<ScopeTracker> {
        &self.scope_tracker
    }

    /// 获取线程注册表
    pub fn threads(&self) -> &Arc<ThreadRegistry> {
        &self.thread_registry
    }
}
```

### 2. VariableRegistry（变量注册表）

**文件**: `src/metadata/registry.rs`

**用途**: 管理系统中的变量元数据。

**核心功能**:
- 变量跟踪和生命周期管理
- 类型信息存储
- 关系跟踪
- 性能指标

**变量元数据**:

```rust
pub struct VariableInfo {
    pub variable_id: VariableId,
    pub name: String,
    pub type_name: String,
    pub ptr: usize,
    pub size: usize,
    pub scope_id: ScopeId,
    pub thread_id: ThreadId,
    pub created_at: u64,
    pub last_accessed: u64,
    pub access_count: u64,
    pub is_active: bool,
}
```

### 3. ScopeTracker（作用域追踪器）

**文件**: `src/metadata/scope.rs`

**用途**: 跟踪作用域层次结构和生命周期以进行内存分析。

**核心功能**:
- 作用域层次结构跟踪
- 父子关系
- 生命周期指标
- 自动作用域管理

**作用域元数据**:

```rust
pub struct ScopeInfo {
    pub scope_id: ScopeId,
    pub name: String,
    pub parent_scope: Option<ScopeId>,
    pub depth: u32,
    pub variables: Vec<VariableId>,
    pub total_memory: usize,
    pub peak_memory: usize,
    pub allocation_count: usize,
    pub lifetime_start: Option<u64>,
    pub lifetime_end: Option<u64>,
    pub is_active: bool,
}
```

### 4. ThreadRegistry（线程注册表）

**文件**: `src/metadata/thread.rs`

**用途**: 管理线程元数据和活动跟踪。

**核心功能**:
- 线程注册和跟踪
- 每线程内存使用
- 活动监控
- 线程关系

**线程元数据**:

```rust
pub struct ThreadInfo {
    pub thread_id: ThreadId,
    pub thread_id_u64: u64,
    pub name: Option<String>,
    pub created_at: u64,
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub current_memory: usize,
    pub peak_memory: usize,
    pub is_active: bool,
}
```

## 设计原则

### 1. 集中化管理
所有元数据集中管理：
- **优势**: 单一事实来源，一致的状态
- **权衡**: 可能存在集中化瓶颈

### 2. 线程安全
所有操作都是线程安全的：
- **优势**: 并发访问安全
- **权衡**: 同步开销

### 3. 高效查找
针对快速查找进行优化：
- **优势**: 查询性能高
- **权衡**: 索引的内存使用较高

### 4. 统一接口
所有元数据类型的单一接口：
- **优势**: 一致的 API，更易使用
- **权衡**: 特定用例的灵活性较低

## 使用示例

### 基本使用

```rust
use memscope::metadata::MetadataEngine;

// 创建元数据引擎
let engine = MetadataEngine::new();

// 访问注册表
let variable_registry = engine.variables();
let scope_tracker = engine.scopes();
let thread_registry = engine.threads();
```

### 变量跟踪

```rust
use memscope::metadata::registry::VariableRegistry;

let registry = VariableRegistry::new();

// 注册变量
let var_id = registry.register_variable(
    "my_variable",
    "Vec<u8>",
    0x1000,
    1024,
    scope_id,
    thread_id,
);

// 获取变量信息
if let Some(var_info) = registry.get_variable(var_id) {
    println!("变量: {}, 大小: {}", var_info.name, var_info.size);
}

// 更新变量访问
registry.record_access(var_id);
```

### 作用域跟踪

```rust
use memscope::metadata::scope::ScopeTracker;

let tracker = ScopeTracker::new();

// 进入作用域
let scope_id = tracker.enter_scope("function_name".to_string()).unwrap();

// 退出作用域
tracker.exit_scope(scope_id).unwrap();

// 获取作用域层次结构
let hierarchy = tracker.get_scope_hierarchy();
```

### 线程跟踪

```rust
use memscope::metadata::thread::ThreadRegistry;

let registry = ThreadRegistry::new();

// 注册线程
let thread_id = registry.register_thread(
    std::thread::current().id(),
    "main".to_string(),
);

// 更新线程统计
registry.record_allocation(thread_id, 1024);
registry.record_deallocation(thread_id, 512);

// 获取线程信息
if let Some(thread_info) = registry.get_thread(thread_id) {
    println!("线程: {}, 内存: {}", thread_info.name.unwrap_or_default(), thread_info.current_memory);
}
```

## 与其他模块的集成

```
捕获引擎
    ↓
事件存储（记录事件）
    ↓
元数据引擎（管理元数据）
    ↓
快照引擎（使用元数据构建快照）
    ↓
查询引擎（使用元数据上下文查询）
    ↓
分析引擎（使用元数据进行分析）
```

## 性能考虑

### 集中化访问
所有元数据访问都通过引擎：
- **优势**: 一致的状态管理
- **权衡**: 高并发可能存在瓶颈

### 内存使用
元数据存储在内存中：
- **优势**: 快速访问
- **权衡**: 内存随跟踪项目增长

### 同步
线程安全操作需要同步：
- **优势**: 并发访问安全
- **权衡**: 性能开销

## 最佳实践

1. **元数据注册**: 尽早注册变量和作用域
2. **清理**: 定期清理不活动的元数据
3. **线程安全**: 使用 Arc<MetadataEngine> 进行共享访问
4. **错误处理**: 始终处理注册错误

## 限制

1. **内存增长**: 元数据增长直到清理
2. **集中化瓶颈**: 所有访问都通过引擎
3. **同步开销**: 线程安全有性能成本
4. **复杂性**: 管理所有元数据类型可能很复杂

## 未来改进

1. **元数据压缩**: 压缩存储的元数据
2. **延迟加载**: 按需加载元数据
3. **持久化**: 添加元数据的磁盘持久化
4. **分布式支持**: 支持分布式元数据管理
5. **更好的索引**: 改进索引以实现更快的查找