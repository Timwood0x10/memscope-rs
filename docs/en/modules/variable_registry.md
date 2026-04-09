# Variable Registry Module

## Overview

The variable registry provides lightweight HashMap-based variable name tracking. It maps memory addresses to variable information, enabling better memory debugging.

## Core Types

**File**: `src/variable_registry.rs`

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

## Global Registry

```rust
static GLOBAL_VARIABLE_REGISTRY: OnceLock<Arc<Mutex<HashMap<usize, VariableInfo>>>> = OnceLock::new();

fn get_global_registry() -> Arc<Mutex<HashMap<usize, VariableInfo>>>
```

## Usage

```rust
use memscope_rs::variable_registry::VariableRegistry;

VariableRegistry::register_variable(
    0x1000,
    "my_vec".to_string(),
    "Vec<u8>".to_string(),
    1024,
);

// Lookup by address
let registry = get_global_registry();
if let Ok(map) = registry.lock() {
    if let Some(info) = map.get(&0x1000) {
        println!("Variable: {}", info.var_name);
    }
}
```

## Design Decisions

1. **Global singleton**: Single registry for entire application
2. **Try-lock**: Fails fast under contention
3. **Thread ID counter**: Maps ThreadId to numeric ID

## Limitations

1. **No automatic cleanup**: Entries persist until application ends
2. **HashMap scalability**: May be slow with millions of entries
3. **No persistence**: Lost on application restart
