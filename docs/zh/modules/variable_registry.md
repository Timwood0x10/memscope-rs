# 变量注册表模块 (Variable Registry Module)

## 概述

变量注册表提供轻量级的基于 HashMap 的变量名追踪。它将内存地址映射到变量信息，实现更好的内存调试。

## 核心类型

**文件**: `src/variable_registry.rs`

```rust
pub struct VariableInfo {
    pub var_name: String,
    pub type_name: String,
    pub timestamp: u64,
    pub size: usize,
    pub thread_id: usize,
    pub memory_usage: u64,
}
```

## 全局注册表

```rust
static GLOBAL_VARIABLE_REGISTRY: OnceLock<Arc<Mutex<HashMap<usize, VariableInfo>>>> = OnceLock::new();

fn get_global_registry() -> Arc<Mutex<HashMap<usize, VariableInfo>>>
```

## 使用

```rust
use memscope_rs::variable_registry::VariableRegistry;

VariableRegistry::register_variable(
    0x1000,
    "my_vec".to_string(),
    "Vec<u8>".to_string(),
    1024,
);

// 按地址查找
let registry = get_global_registry();
if let Ok(map) = registry.lock() {
    if let Some(info) = map.get(&0x1000) {
        println!("变量: {}", info.var_name);
    }
}
```

## 设计决策

1. **全局单例**: 整个应用程序使用单一注册表
2. **尝试锁定**: 争用时快速失败
3. **线程 ID 计数器**: 将 ThreadId 映射到数字 ID

## 限制

1. **无自动清理**: 条目持续到应用程序结束
2. **HashMap 可扩展性**: 可能有数百万条目时变慢
3. **无持久化**: 应用程序重启时丢失
