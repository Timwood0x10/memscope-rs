# 核心模块 (Core Module)

## 概述

核心模块为 memscope-rs 库提供基础内存追踪功能。它实现了拦截、追踪和分析 Rust 应用程序中内存分配所需的底层基础设施。

## 组件

### 1. TrackingAllocator（追踪分配器）

**文件**: `src/core/allocator.rs`

**用途**: 自定义全局分配器，拦截应用程序中的所有堆分配和释放操作。

**核心功能**:
- 实现 `GlobalAlloc` trait 以覆盖系统分配器
- 追踪每个堆分配和释放操作
- 使用线程本地存储标志防止递归追踪
- 基于分配大小提供类型推断
- 恐慌安全操作

**源代码**:

```rust
pub struct TrackingAllocator;

thread_local! {
    static TRACKING_DISABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);

        if !ptr.is_null() {
            let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());

            if should_track {
                TRACKING_DISABLED.with(|disabled| disabled.set(true));

                if let Ok(tracker) = std::panic::catch_unwind(crate::core::tracker::get_tracker) {
                    let _ = tracker.track_allocation(ptr as usize, layout.size());
                }

                TRACKING_DISABLED.with(|disabled| disabled.set(false));
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());

        if should_track {
            TRACKING_DISABLED.with(|disabled| disabled.set(true));

            if let Ok(tracker) = std::panic::catch_unwind(crate::core::tracker::get_tracker) {
                let _ = tracker.track_deallocation(ptr as usize);
            }

            TRACKING_DISABLED.with(|disabled| disabled.set(false));
        }

        System.dealloc(ptr, layout);
    }
}
```

**设计理念**:

1. **零开销追踪**: 使用线程本地存储标志而非锁，最小化性能影响
2. **递归保护**: 在追踪操作期间禁用追踪，防止无限循环
3. **恐慌恢复**: 追踪失败不会导致应用崩溃
4. **类型推断**: 基于分配大小模式提供基本类型信息

**类型推断实现**:

```rust
fn _infer_type_from_allocation_context(size: usize) -> &'static str {
    match size {
        // 常见 Rust 类型大小
        1 => "u8",
        2 => "u16",
        4 => "u32",
        8 => "u64",
        16 => "u128",

        // String 和 Vec 常见大小
        24 => "String",
        32 => "Vec<T>",
        48 => "HashMap<K,V>",

        // 智能指针大小
        size if size == std::mem::size_of::<std::sync::Arc<String>>() => "Arc<T>",
        size if size == std::mem::size_of::<std::rc::Rc<String>>() => "Rc<T>",
        size if size == std::mem::size_of::<Box<String>>() => "Box<T>",

        // 其他大小的默认值
        _ => "unknown",
    }
}
```

**使用方法**:

```rust
// 在 main.rs 或 lib.rs 中
use memscope::core::TrackingAllocator;

#[global_allocator]
static GLOBAL_ALLOCATOR: TrackingAllocator = TrackingAllocator;

fn main() {
    // 所有堆分配现在都会自动追踪
    let data = vec![1, 2, 3, 4, 5];
    let string = String::from("Hello");
}
```

### 2. 错误处理

**文件**: `src/core/error.rs`

**用途**: 整个 memscope-rs 库的统一错误处理系统。

**核心功能**:
- 使用 `Arc<str>` 简化高效的错误类型，减少字符串克隆
- 错误分类（内存、分析、导出、配置、系统、内部）
- 错误严重级别（低、中、高、严重）
- 错误恢复机制
- 与旧错误类型的向后兼容性

**错误类型**:

```rust
pub enum MemScopeError {
    Memory {
        operation: MemoryOperation,
        message: Arc<str>,
        context: Option<Arc<str>>,
    },

    Analysis {
        analyzer: Arc<str>,
        message: Arc<str>,
        recoverable: bool,
    },

    Export {
        format: Arc<str>,
        message: Arc<str>,
        partial_success: bool,
    },

    Configuration {
        component: Arc<str>,
        message: Arc<str>,
    },

    System {
        error_type: SystemErrorType,
        message: Arc<str>,
        source_message: Option<Arc<str>>,
    },

    Internal {
        message: Arc<str>,
        location: Option<Arc<str>>,
    },
}
```

**错误严重级别**:

```rust
pub enum ErrorSeverity {
    Low,      // 警告，部分失败
    Medium,   // 操作失败
    High,     // 关键分析失败
    Critical, // 内部错误，bug
}
```

**错误恢复**:

```rust
pub enum RecoveryAction {
    Retry { max_attempts: u32, delay_ms: u64 },
    UseDefault { value: String },
    Skip,
    Abort,
    Fallback { strategy: String },
}

pub trait ErrorRecovery {
    fn can_recover(&self, error: &MemScopeError) -> bool;
    fn get_recovery_action(&self, error: &MemScopeError) -> Option<RecoveryAction>;
    fn execute_recovery(&self, action: &RecoveryAction) -> Result<()>;
}
```

**设计理念**:

1. **性能**: 使用 `Arc<str>` 而非 `String` 减少克隆开销
2. **分类**: 清晰的错误类型分离以便更好地处理
3. **可恢复性**: 标记错误为可恢复或不可恢复以便适当响应
4. **向后兼容**: 自动将旧错误类型转换为新格式

**使用方法**:

```rust
use memscope::core::error::{MemScopeError, MemoryOperation};

fn allocate_memory(size: usize) -> Result<*mut u8, MemScopeError> {
    if size == 0 {
        return Err(MemScopeError::memory_with_context(
            MemoryOperation::Allocation,
            "零大小分配",
            "在 allocate_memory 中",
        ));
    }

    // ... 分配逻辑
    Ok(ptr)
}
```

### 3. 作用域追踪器

**文件**: `src/core/scope_tracker.rs`

**用途**: 追踪变量生命周期和作用域层次结构以进行内存分析。

**核心功能**:
- 每线程作用域堆栈追踪
- 作用域层次结构和关系追踪
- 变量与作用域关联
- 作用域生命周期指标
- 使用 RAII 自动作用域管理

**核心实现**:

```rust
pub struct ScopeTracker {
    pub active_scopes: RwLock<HashMap<ScopeId, ScopeInfo>>,
    pub completed_scopes: Mutex<Vec<ScopeInfo>>,
    pub scope_hierarchy: Mutex<ScopeHierarchy>,
    next_scope_id: AtomicU64,
    pub scope_stack: RwLock<HashMap<String, Vec<ScopeId>>>,
}

impl ScopeTracker {
    pub fn enter_scope(&self, name: String) -> TrackingResult<ScopeId> {
        let scope_id = self.allocate_scope_id();
        let thread_id = format!("{:?}", std::thread::current().id());
        let timestamp = current_timestamp();

        // 确定父作用域和深度
        let (parent_scope, depth) = {
            let stack = self.scope_stack.read().unwrap();
            if let Some(thread_stack) = stack.get(&thread_id) {
                if let Some(&parent_id) = thread_stack.last() {
                    let active = self.active_scopes.read().unwrap();
                    if let Some(parent) = active.get(&parent_id) {
                        (Some(parent.name.clone()), parent.depth + 1)
                    } else {
                        (None, 0)
                    }
                } else {
                    (None, 0)
                }
            } else {
                (None, 0)
            }
        };

        // 创建并注册作用域信息
        let scope_info = ScopeInfo {
            name: name.clone(),
            parent: parent_scope.clone(),
            children: Vec::new(),
            depth,
            variables: Vec::new(),
            total_memory: 0,
            peak_memory: 0,
            allocation_count: 0,
            lifetime_start: Some(timestamp as u64),
            lifetime_end: None,
            is_active: true,
            start_time: timestamp as u64,
            end_time: None,
            memory_usage: 0,
            child_scopes: Vec::new(),
            parent_scope: parent_scope.clone(),
        };

        self.active_scopes.write().unwrap().insert(scope_id, scope_info);
        self.scope_stack.write().unwrap()
            .entry(thread_id.clone())
            .or_default()
            .push(scope_id);

        Ok(scope_id)
    }
}
```

**RAII 作用域守卫**:

```rust
pub struct ScopeGuard {
    scope_id: ScopeId,
    tracker: Arc<ScopeTracker>,
}

impl ScopeGuard {
    pub fn enter(name: &str) -> TrackingResult<Self> {
        let tracker = get_global_scope_tracker();
        let scope_id = tracker.enter_scope(name.to_string())?;
        Ok(Self { scope_id, tracker })
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        let _ = self.tracker.exit_scope(self.scope_id);
    }
}
```

**设计理念**:

1. **线程本地追踪**: 每个线程维护自己的作用域堆栈
2. **层次感知**: 追踪作用域之间的父子关系
3. **自动清理**: 使用 RAII 模式自动退出作用域
4. **性能**: 使用原子操作进行作用域 ID 分配

**使用方法**:

```rust
use memscope::core::scope_tracker::ScopeGuard;

fn process_data() {
    // 进入作用域并自动清理
    let _guard = ScopeGuard::enter("process_data").unwrap();

    // 变量自动与此作用域关联
    let data = vec![1, 2, 3, 4, 5];

    // 嵌套作用域
    {
        let _nested_guard = ScopeGuard::enter("inner_scope").unwrap();
        let temp = String::from("temporary");
    } // 内层作用域自动退出

} // 外层作用域自动退出
```

**宏使用**:

```rust
// 简单作用域追踪
track_scope!("function_name");

// 带代码块的作用域
track_scope!("block_name", {
    let data = vec![1, 2, 3];
    // ... 处理
});
```

## 设计原则

### 1. 零开销
核心模块设计为具有最小的性能影响：
- 使用线程本地存储而非锁
- 避免在追踪操作期间进行分配
- 使用静态字符串进行类型推断

### 2. 恐慌安全
所有操作都是恐慌安全的：
- 追踪失败不会导致应用崩溃
- 使用 `catch_unwind` 处理追踪代码中的恐慌
- 追踪失败时优雅降级

### 3. 线程安全
所有共享状态都正确同步：
- 使用 `RwLock` 处理读密集型数据结构
- 使用 `Mutex` 处理写密集型数据结构
- 使用 `AtomicU64` 处理计数器

### 4. 类型安全
强类型系统确保正确性：
- 自定义错误类型防止无效操作
- Result 类型强制错误处理
- 类型推断提供额外上下文

## 性能考虑

### 线程本地存储
使用线程本地存储标志而非锁：
- **优势**: 线程间无锁竞争
- **权衡**: 每个线程有自己的追踪状态

### 静态字符串
使用静态字符串进行类型推断：
- **优势**: 追踪期间无分配
- **权衡**: 类型信息有限

### 原子操作
使用原子操作处理计数器：
- **优势**: 无锁性能
- **权衡**: 仅限于简单操作

## 测试

核心模块包含全面的测试：

```rust
#[test]
fn test_allocation_tracking() {
    let allocator = TrackingAllocator::new();
    let layout = Layout::from_size_align(1024, 8).unwrap();

    unsafe {
        let ptr = allocator.alloc(layout);
        assert!(!ptr.is_null());
        allocator.dealloc(ptr, layout);
    }
}

#[test]
fn test_recursive_allocation_handling() {
    // 验证递归分配不会导致无限循环
    let allocator = TrackingAllocator::new();
    let layout = Layout::from_size_align(64, 8).unwrap();

    unsafe {
        let ptr = allocator.alloc(layout);
        if !ptr.is_null() {
            allocator.dealloc(ptr, layout);
        }
    }
}
```

## 集成

核心模块与 memscope-rs 库的其余部分集成：

```
core/
  ↓
tracker/      (使用 TrackingAllocator)
  ↓
capture/      (追踪分配事件)
  ↓
analysis/     (分析作用域数据)
  ↓
render/       (可视化作用域层次结构)
```

## 最佳实践

1. **全局分配器**: 在程序启动时设置一次追踪分配器为全局分配器
2. **错误处理**: 始终适当处理 `MemScopeError`
3. **作用域管理**: 使用 `ScopeGuard` 进行自动清理
4. **性能**: 在生产环境中使用采样减少开销

## 限制

1. **类型推断**: 仅限于常见类型大小
2. **栈变量**: 仅追踪堆分配
3. **静态变量**: 不追踪静态变量
4. **外部内存**: 除非外部库使用全局分配器，否则不追踪外部库分配的内存

## 未来改进

1. **更好的类型推断**: 与编译器集成以获取准确的类型信息
2. **栈追踪**: 追踪栈分配
3. **变量名**: 捕获实际变量名（目前使用推断的名称）
4. **性能**: 进一步优化追踪开销