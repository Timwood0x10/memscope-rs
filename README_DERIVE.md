# Memscope-rs Derive Macro and Extended Type Support

This document describes the new `#[derive(Trackable)]` macro and extended built-in type support added to memscope-rs.

## Features

### 1. Derive Macro (`#[derive(Trackable)]`)

The derive macro automatically implements the `Trackable` trait for custom types, eliminating the need for manual implementation.

#### Usage

Enable the derive feature in your `Cargo.toml`:

```toml
[dependencies]
memscope-rs = { version = "0.1.2", features = ["derive"] }
```

Then use the derive macro on your structs and enums:

```rust
use memscope_rs::{init, track_var, Trackable};

#[derive(Trackable)]
struct UserProfile {
    name: String,
    email: String,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
}

#[derive(Trackable)]
enum DataType {
    Text(String),
    Numbers(Vec<i32>),
    Empty,
}

fn main() {
    init();
    
    let user = UserProfile {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        tags: vec!["developer".to_string()],
        metadata: HashMap::new(),
    };
    
    let _tracked_user = track_var!(user);
    // Automatic tracking with lifecycle management
}
```

#### Supported Types

The derive macro works with:

- **Structs with named fields**: `struct User { name: String, age: u32 }`
- **Tuple structs**: `struct Point(f64, f64)`
- **Unit structs**: `struct Marker;`
- **Enums with data**: `enum Data { Text(String), Number(i32) }`

#### Generated Implementation

The macro automatically generates:

- `get_heap_ptr()`: Returns struct address for types with heap allocations
- `get_type_name()`: Returns the type name as a string literal
- `get_size_estimate()`: Calculates total size including all fields
- `get_internal_allocations()`: Lists all internal heap allocations with field names

### 2. Extended Built-in Type Support

We've added `Trackable` implementations for many more standard library types:

#### Collection Types

- `HashMap<K, V>` ✅ (already supported)
- `BTreeMap<K, V>` ✅ **NEW**
- `HashSet<T>` ✅ **NEW**
- `BTreeSet<T>` ✅ **NEW**
- `VecDeque<T>` ✅ **NEW**
- `LinkedList<T>` ✅ **NEW**
- `BinaryHeap<T>` ✅ **NEW**

#### Smart Pointers and Reference Types

- `Box<T>` ✅ (already supported)
- `Rc<T>` ✅ (already supported)
- `Arc<T>` ✅ (already supported)
- `Weak<T>` (both `std::rc::Weak` and `std::sync::Weak`) ✅ **NEW**
- `RefCell<T>` ✅ **NEW**

#### Synchronization Primitives

- `Mutex<T>` ✅ **NEW**
- `RwLock<T>` ✅ **NEW**

#### Generic Wrapper Types

- `Option<T>` where `T: Trackable` ✅ **NEW**
- `Result<T, E>` where `T: Trackable, E: Trackable` ✅ **NEW**

### 3. Usage Examples

#### Basic Derive Usage

```rust
use memscope_rs::{init, track_var, Trackable};

#[derive(Trackable)]
struct ComplexData {
    name: String,
    scores: Vec<i32>,
    metadata: HashMap<String, String>,
    cache: BTreeMap<u64, Vec<u8>>,
}

fn main() {
    init();
    
    let data = ComplexData {
        name: "test".to_string(),
        scores: vec![1, 2, 3, 4, 5],
        metadata: HashMap::new(),
        cache: BTreeMap::new(),
    };
    
    let _tracked = track_var!(data);
    // All internal allocations are automatically tracked
}
```

#### Extended Collections

```rust
use memscope_rs::{init, track_var};
use std::collections::*;

fn main() {
    init();
    
    // All these types now support automatic tracking
    let _map = track_var!(BTreeMap::<String, i32>::new());
    let _set = track_var!(HashSet::<String>::new());
    let _deque = track_var!(VecDeque::<i32>::new());
    let _list = track_var!(LinkedList::<String>::new());
    let _heap = track_var!(BinaryHeap::<i32>::new());
}
```

#### Smart Pointers and Sync Types

```rust
use memscope_rs::{init, track_var};
use std::sync::{Arc, Mutex, RwLock};
use std::rc::{Rc, Weak};
use std::cell::RefCell;

fn main() {
    init();
    
    let rc_data = Rc::new("shared".to_string());
    let _tracked_rc = track_var!(rc_data.clone());
    
    let weak_ref = Rc::downgrade(&rc_data);
    let _tracked_weak = track_var!(weak_ref);
    
    let mutex_data = Mutex::new(vec![1, 2, 3]);
    let _tracked_mutex = track_var!(mutex_data);
    
    let rwlock_data = RwLock::new("concurrent".to_string());
    let _tracked_rwlock = track_var!(rwlock_data);
}
```

#### Option and Result Types

```rust
use memscope_rs::{init, track_var};

fn main() {
    init();
    
    let some_data: Option<Vec<String>> = Some(vec!["data".to_string()]);
    let _tracked_option = track_var!(some_data);
    
    let result_data: Result<String, String> = Ok("success".to_string());
    let _tracked_result = track_var!(result_data);
}
```

### 4. Running Examples

To test the new functionality:

```bash
# Test derive macro
cargo run --example derive_macro_demo --features derive

# Test without derive feature (shows fallback behavior)
cargo run --example derive_macro_demo

# Test basic derive compilation
cargo run --example tmp_rovodev_test_derive --features derive
```

### 5. Benefits

#### Automatic Implementation
- No more manual `Trackable` implementations for custom types
- Consistent behavior across all derived types
- Automatic handling of nested allocations

#### Comprehensive Type Coverage
- Support for virtually all standard library collection types
- Smart pointer tracking with reference counting
- Synchronization primitive tracking
- Generic wrapper type support

#### Enhanced Analysis
- Better memory usage insights for complex data structures
- Automatic detection of internal allocations
- Improved lifecycle tracking for all supported types

### 6. Migration Guide

#### From Manual Implementation

**Before:**
```rust
impl Trackable for MyStruct {
    fn get_heap_ptr(&self) -> Option<usize> {
        Some(self as *const _ as usize)
    }
    
    fn get_type_name(&self) -> &'static str {
        "MyStruct"
    }
    
    fn get_size_estimate(&self) -> usize {
        // Manual calculation...
    }
    
    fn get_internal_allocations(&self, var_name: &str) -> Vec<(usize, String)> {
        // Manual enumeration...
    }
}
```

**After:**
```rust
#[derive(Trackable)]
struct MyStruct {
    // fields...
}
```

#### Adding Derive Feature

Update your `Cargo.toml`:
```toml
[dependencies]
memscope-rs = { version = "0.1.2", features = ["derive", "tracking-allocator"] }
```

### 7. Limitations

- **Unions**: Not supported for safety reasons
- **Generic constraints**: Derived implementations require all generic parameters to implement `Trackable`
- **Custom behavior**: For specialized tracking behavior, manual implementation is still recommended

### 8. Future Enhancements

- Support for custom derive attributes (e.g., `#[trackable(skip)]`)
- Integration with async types (`Future`, `Stream`, etc.)
- Support for external crate types through extension traits
- Performance optimizations for large-scale tracking