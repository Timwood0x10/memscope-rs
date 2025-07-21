# 🎯 Memory Tracking Macros Usage Guide

This guide helps you choose the right tracking macro for your specific use case. We provide three different tracking macros, each optimized for different scenarios.

## 📋 Quick Reference

| Macro | Ownership | Performance | Use Case | When to Use |
|-------|-----------|-------------|----------|-------------|
| `track_var!` | **No change** | **Zero cost** | Basic profiling | ✅ **Most common** |
| `track_var_smart!` | **Automatic** | **Optimized** | Mixed types | ✅ **Convenience** |
| `track_var_owned!` | **Takes ownership** | **Overhead** | Advanced analysis | ⚠️ **Specialized** |

---

## 🥇 `track_var!` - **[RECOMMENDED]**

**Zero-cost tracking without ownership changes**

### ✅ Use this when:
- You want to track memory usage without changing your code
- Performance is critical (zero overhead)
- You need to continue using the variable after tracking
- You're tracking many variables and don't want clone overhead
- You're doing basic memory profiling and analysis

### ❌ Don't use this when:
- You need precise lifecycle tracking with automatic cleanup
- You're tracking temporary variables that will be moved/consumed immediately

### 📝 Example:
```rust
use memscope_rs::track_var;

let my_vec = vec![1, 2, 3, 4, 5];
track_var!(my_vec); // Zero-cost tracking

// my_vec can still be used normally - no ownership changes!
println!("Vector: {:?}", my_vec);
my_vec.push(6); // Still fully usable
drop(my_vec);   // You control the lifecycle
```

### 🎯 Perfect for:
- Memory profiling during development
- Production monitoring with minimal overhead
- Large codebases where you want to add tracking without refactoring

---

## 🧠 `track_var_smart!` - **[SMART]**

**Intelligent tracking that automatically chooses the best strategy**

### ✅ Use this when:
- You want the best of both worlds without thinking about it
- You're tracking mixed types (some Copy, some not)
- You want automatic optimization based on type characteristics
- You're prototyping and want convenience

### ❌ Don't use this when:
- You need explicit control over tracking behavior
- You're in performance-critical code and want predictable behavior
- You need precise lifecycle tracking (use `track_var_owned!` instead)

### 📝 Example:
```rust
use memscope_rs::track_var_smart;
use std::rc::Rc;

let number = 42i32;           // Copy type - will be copied (cheap)
let my_vec = vec![1, 2, 3];   // Non-Copy - will be tracked by reference
let rc_data = Rc::new(vec![]); // Smart pointer - will clone the Rc (cheap)

track_var_smart!(number);   // Copies the i32 (cheap)
track_var_smart!(my_vec);    // Tracks by reference (zero cost)
track_var_smart!(rc_data);   // Clones Rc (cheap reference increment)

// All variables remain fully usable!
println!("{}, {:?}, {:?}", number, my_vec, rc_data);
```

### 🎯 Perfect for:
- Prototyping and experimentation
- Mixed codebases with different types
- When you want convenience without thinking about optimization

---

## ⚙️ `track_var_owned!` - **[ADVANCED]**

**Full lifecycle management with ownership transfer**

### ✅ Use this when:
- You need precise lifecycle tracking with automatic cleanup detection
- You want to measure exact variable lifetimes
- You're doing advanced memory analysis or debugging
- You're tracking variables that will be consumed/moved anyway
- You need the wrapper's additional methods (get(), get_mut(), into_inner())

### ❌ Don't use this when:
- You need to continue using the original variable (use `track_var!` instead)
- Performance is critical and you don't need lifecycle timing
- You're tracking many variables (clone overhead)
- You're doing basic memory profiling

### ⚠️ Performance Note:
This macro takes ownership of the variable. If you need the original variable afterwards, you'll need to clone it first, which has performance implications.

### 📝 Example:
```rust
use memscope_rs::track_var_owned;

let my_vec = vec![1, 2, 3, 4, 5];
let tracked_vec = track_var_owned!(my_vec); // Takes ownership

// tracked_vec behaves like my_vec but with automatic lifecycle tracking
println!("Length: {}", tracked_vec.len()); // Transparent access via Deref
let original = tracked_vec.into_inner(); // Get original back if needed
// Automatic cleanup tracking when tracked_vec is dropped
```

### 🎯 Perfect for:
- Memory leak detection
- Precise lifetime analysis
- Advanced debugging scenarios
- Research and development

---

## 🚀 Performance Comparison

| Scenario | `track_var!` | `track_var_smart!` | `track_var_owned!` |
|----------|--------------|-------------------|-------------------|
| **i32** | Zero cost | Copy (negligible) | Wrapper overhead |
| **Vec<T>** | Zero cost | Zero cost | Move + wrapper |
| **String** | Zero cost | Zero cost | Move + wrapper |
| **Rc<T>** | Zero cost | Zero cost | Move + wrapper |
| **Large struct** | Zero cost | Zero cost | Move + wrapper |

## 🎯 Decision Tree

```
Do you need precise lifecycle tracking?
├─ YES → Use `track_var_owned!`
└─ NO → Do you want automatic optimization?
    ├─ YES → Use `track_var_smart!`
    └─ NO → Use `track_var!` (recommended)
```

## 📊 Common Patterns

### Pattern 1: Basic Memory Profiling
```rust
// ✅ Recommended approach
let data = load_large_dataset();
track_var!(data);
process_data(&data); // data is still usable
```

### Pattern 2: Mixed Type Tracking
```rust
// ✅ Smart approach for convenience
let id = 42u64;
let name = String::from("user");
let items = vec![1, 2, 3];

track_var_smart!(id);    // Copies u64
track_var_smart!(name);  // Tracks by reference
track_var_smart!(items); // Tracks by reference
```

### Pattern 3: Advanced Analysis
```rust
// ✅ Owned approach for detailed tracking
let data = expensive_computation();
let tracked = track_var_owned!(data); // Takes ownership
// Automatic lifecycle tracking
// tracked is dropped here with precise timing
```

### Pattern 4: Avoiding Clone Overhead
```rust
// ❌ Don't do this (unnecessary clones)
let large_vec = vec![0; 1_000_000];
let tracked = track_var_owned!(large_vec.clone()); // Expensive!

// ✅ Do this instead
let large_vec = vec![0; 1_000_000];
track_var!(large_vec); // Zero cost
```

## 🔧 Migration Guide

### From Old `track_var!` (that took ownership)
```rust
// Old way (caused ownership issues)
let data = vec![1, 2, 3];
let tracked = track_var!(data); // Took ownership
// data was no longer usable

// New way (choose based on need)
let data = vec![1, 2, 3];

// Option 1: Zero-cost tracking (recommended)
track_var!(data);
println!("{:?}", data); // Still works!

// Option 2: If you need lifecycle tracking
let tracked = track_var_owned!(data);
println!("{:?}", tracked); // Works through Deref
```

## 🎉 Summary

- **Default choice**: `track_var!` for zero-cost tracking
- **Convenience choice**: `track_var_smart!` for automatic optimization
- **Advanced choice**: `track_var_owned!` for precise lifecycle analysis

Choose the right tool for the job, and enjoy efficient memory tracking without the ownership headaches! 🚀