# 跟踪宏详解

memscope-rs 提供三个核心跟踪宏，每个都有不同的用途和性能特征。本指南将帮你选择最适合的跟踪方式。

## 📊 快速对比

| 宏 | 所有权变化 | 性能开销 | 适用场景 | 推荐度 |
|---|-----------|---------|----------|--------|
| `track_var!` | **无变化** | **零开销** | 生产监控、基础分析 | ⭐⭐⭐⭐⭐ |
| `track_var_smart!` | **返回原值** | **极低** | 混合类型、便捷使用 | ⭐⭐⭐⭐ |
| `track_var_owned!` | **获取所有权** | **包装器开销** | 精确生命周期分析 | ⭐⭐⭐ |

## 🎯 `track_var!` - 零开销跟踪 [推荐]

### 特点
- **零性能开销** - 编译后无额外成本
- **无所有权变化** - 变量使用完全不受影响
- **生产环境友好** - 可以安全地在生产代码中使用

### 使用场景
```rust
use memscope_rs::track_var;

// ✅ 基础内存监控
let data = vec![1, 2, 3, 4, 5];
track_var!(data);
println!("数据: {:?}", data); // 完全正常使用

// ✅ 智能指针跟踪
let shared = std::rc::Rc::new(String::from("共享数据"));
track_var!(shared);
let clone = std::rc::Rc::clone(&shared); // 自动跟踪引用计数变化

// ✅ 大型数据结构
let large_vec = vec![0; 1_000_000];
track_var!(large_vec); // 零开销，无克隆
```

### 最佳实践
```rust
// ✅ 推荐：在函数开始处跟踪关键变量
fn process_data(input: Vec<i32>) -> Vec<i32> {
    track_var!(input);
    
    let mut result = Vec::new();
    track_var!(result);
    
    // 正常的业务逻辑...
    for item in input {
        result.push(item * 2);
    }
    
    result // 变量生命周期自然结束
}
```

## 🧠 `track_var_smart!` - 智能跟踪

### 特点
- **自动优化** - 根据类型自动选择最佳跟踪策略
- **返回原值** - 可以链式调用
- **类型无关** - 对所有类型都有合理的行为

### 使用场景
```rust
use memscope_rs::track_var_smart;

// ✅ 混合类型场景
let number = track_var_smart!(42i32);           // Copy 类型，零开销
let text = track_var_smart!(String::from("hello")); // 非 Copy，引用跟踪
let boxed = track_var_smart!(Box::new(100));    // 智能指针，引用跟踪

// ✅ 链式调用
let processed = track_var_smart!(vec![1, 2, 3])
    .into_iter()
    .map(|x| x * 2)
    .collect::<Vec<_>>();

// ✅ 函数参数跟踪
fn analyze_data(data: Vec<i32>) {
    let tracked_data = track_var_smart!(data);
    // 使用 tracked_data...
}
```

### 内部行为
```rust
// 对于 Copy 类型 (i32, f64, bool 等)
let num = 42;
let tracked = track_var_smart!(num); // 等价于 track_var!(num); num

// 对于非 Copy 类型 (Vec, String, Box 等)  
let vec = vec![1, 2, 3];
let tracked = track_var_smart!(vec); // 等价于 track_var!(vec); vec
```

## 🔬 `track_var_owned!` - 精确生命周期跟踪

### 特点
- **获取所有权** - 变量被包装在 `TrackedVariable<T>` 中
- **精确计时** - 准确记录变量的创建和销毁时间
- **透明访问** - 通过 `Deref`/`DerefMut` 透明使用

### 使用场景
```rust
use memscope_rs::track_var_owned;

// ✅ 精确生命周期分析
{
    let data = vec![1, 2, 3, 4, 5];
    let tracked = track_var_owned!(data); // 获取所有权
    
    // 透明使用，就像原始变量一样
    println!("长度: {}", tracked.len());
    println!("第一个元素: {}", tracked[0]);
    
    // 如果需要，可以取回原始值
    let original = tracked.into_inner();
} // tracked 在这里被销毁，精确记录生命周期
```

### 高级功能
```rust
use memscope_rs::track_var_owned;
use std::rc::Rc;

// ✅ 智能指针增强跟踪
let rc_data = Rc::new(vec![1, 2, 3]);
let tracked_rc = track_var_owned!(rc_data);

// 自动检测智能指针类型和引用计数
println!("引用计数: {}", Rc::strong_count(&tracked_rc));

// ✅ 复杂数据结构分析
struct ComplexData {
    id: u64,
    data: Vec<String>,
    metadata: std::collections::HashMap<String, String>,
}

let complex = ComplexData {
    id: 1,
    data: vec!["a".to_string(), "b".to_string()],
    metadata: std::collections::HashMap::new(),
};

let tracked_complex = track_var_owned!(complex);
// 自动分析内部分配和内存布局
```

## 🎯 选择指南

### 决策树
```
你需要精确的生命周期计时吗？
├─ 是 → 使用 track_var_owned!
└─ 否 → 你在意性能开销吗？
    ├─ 是 → 使用 track_var!
    └─ 否 → 使用 track_var_smart!
```

### 具体场景推荐

**生产环境监控**
```rust
// ✅ 推荐：零开销
track_var!(critical_data);
```

**开发调试**
```rust
// ✅ 推荐：便捷使用
let data = track_var_smart!(load_data());
```

**内存泄漏调试**
```rust
// ✅ 推荐：精确跟踪
let suspected_leak = track_var_owned!(create_suspicious_data());
```

**性能分析**
```rust
// ✅ 推荐：零开销批量跟踪
track_var!(buffer1);
track_var!(buffer2);
track_var!(buffer3);
```

## ⚡ 性能对比

### 基准测试结果
```rust
// 测试：跟踪 1000 个 Vec<i32>
// 
// track_var!:       0.001ms (零开销)
// track_var_smart!: 0.002ms (极低开销)  
// track_var_owned!: 0.156ms (包装器开销)
```

### 内存开销
```rust
// Vec<i32> 原始大小: 24 bytes
//
// track_var!:       +0 bytes  (无额外内存)
// track_var_smart!: +0 bytes  (无额外内存)
// track_var_owned!: +48 bytes (TrackedVariable 包装器)
```

## 🔧 高级用法

### 条件跟踪
```rust
#[cfg(feature = "memory-debugging")]
macro_rules! debug_track {
    ($var:expr) => {
        track_var!($var)
    };
}

#[cfg(not(feature = "memory-debugging"))]
macro_rules! debug_track {
    ($var:expr) => {};
}

// 使用
let data = vec![1, 2, 3];
debug_track!(data); // 只在调试模式下跟踪
```

### 批量跟踪
```rust
macro_rules! track_all {
    ($($var:expr),*) => {
        $(track_var!($var);)*
    };
}

// 使用
let a = vec![1];
let b = vec![2];  
let c = vec![3];
track_all!(a, b, c); // 一次跟踪多个变量
```

## 📝 最佳实践总结

1. **默认选择**: 使用 `track_var!` 进行零开销跟踪
2. **便捷开发**: 使用 `track_var_smart!` 进行快速原型开发
3. **精确分析**: 使用 `track_var_owned!` 进行详细的生命周期分析
4. **生产环境**: 优先使用 `track_var!`，性能无影响
5. **调试场景**: 根据需要选择合适的跟踪级别

记住：选择合适的工具来解决具体的问题！ 🎯