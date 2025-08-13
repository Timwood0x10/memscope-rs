# Unsafe 和 FFI 跟踪

跟踪和分析 Rust 程序中的 unsafe 代码和 FFI 调用的内存使用。

## 🎯 目标

- 跟踪 unsafe 代码块
- 监控 FFI 调用
- 检测内存安全问题
- 分析 C/C++ 互操作

## ⚠️ Unsafe 代码跟踪

### 基础 Unsafe 跟踪

```rust
use memscope_rs::{init, track_var, track_unsafe_operation};

fn main() {
    init();
    
    // 跟踪 unsafe 操作
    unsafe {
        track_unsafe_operation("raw_pointer_deref", "main.rs:10");
        
        let ptr = Box::into_raw(Box::new(42));
        let value = *ptr;
        track_var!(value);
        
        // 记得释放内存
        let _ = Box::from_raw(ptr);
    }
}
```

### Unsafe 操作分类

```rust
use memscope_rs::UnsafeOperationType;

fn track_various_unsafe_operations() {
    // 1. 原始指针解引用
    unsafe {
        let ptr = 0x1000 as *const i32;
        track_unsafe_operation(
            UnsafeOperationType::RawPointerDereference,
            "Dereferencing raw pointer",
            Some("main.rs:25".to_string())
        );
        // let value = *ptr; // 危险操作
    }
    
    // 2. 可变静态变量访问
    static mut COUNTER: i32 = 0;
    unsafe {
        track_unsafe_operation(
            UnsafeOperationType::StaticMutAccess,
            "Accessing mutable static",
            Some("main.rs:35".to_string())
        );
        COUNTER += 1;
    }
    
    // 3. 联合体字段访问
    union MyUnion {
        i: i32,
        f: f32,
    }
    
    let u = MyUnion { i: 42 };
    unsafe {
        track_unsafe_operation(
            UnsafeOperationType::UnionFieldAccess,
            "Accessing union field",
            Some("main.rs:50".to_string())
        );
        let value = u.f;
        track_var!(value);
    }
}
```

## 🔗 FFI 调用跟踪

### C 库集成

```rust
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int, c_void};

// 外部 C 函数声明
extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn strlen(s: *const c_char) -> usize;
}

fn track_c_memory_operations() {
    memscope_rs::init();
    
    unsafe {
        // 跟踪 C 内存分配
        track_unsafe_operation(
            UnsafeOperationType::FFICall,
            "malloc call",
            Some("ffi.rs:15".to_string())
        );
        
        let ptr = malloc(1024);
        if !ptr.is_null() {
            // 跟踪分配的内存
            let allocated_memory = std::slice::from_raw_parts_mut(
                ptr as *mut u8, 1024
            );
            track_var!(allocated_memory);
            
            // 使用内存...
            
            // 释放内存
            track_unsafe_operation(
                UnsafeOperationType::FFICall,
                "free call",
                Some("ffi.rs:30".to_string())
            );
            free(ptr);
        }
    }
}
```

### 复杂 FFI 场景

```rust
// 模拟复杂的 C 结构体
#[repr(C)]
struct CStruct {
    data: *mut u8,
    size: usize,
    capacity: usize,
}

extern "C" {
    fn create_c_struct(size: usize) -> *mut CStruct;
    fn destroy_c_struct(s: *mut CStruct);
    fn c_struct_add_data(s: *mut CStruct, data: *const u8, len: usize) -> c_int;
}

fn complex_ffi_tracking() {
    memscope_rs::init();
    
    unsafe {
        // 创建 C 结构体
        track_unsafe_operation(
            UnsafeOperationType::FFICall,
            "create_c_struct",
            Some("complex_ffi.rs:10".to_string())
        );
        
        let c_struct = create_c_struct(1024);
        if !c_struct.is_null() {
            track_var!(c_struct);
            
            // 添加数据到 C 结构体
            let rust_data = b"Hello from Rust!";
            track_unsafe_operation(
                UnsafeOperationType::FFICall,
                "c_struct_add_data",
                Some("complex_ffi.rs:20".to_string())
            );
            
            let result = c_struct_add_data(
                c_struct,
                rust_data.as_ptr(),
                rust_data.len()
            );
            
            if result == 0 {
                println!("数据添加成功");
            }
            
            // 清理
            track_unsafe_operation(
                UnsafeOperationType::FFICall,
                "destroy_c_struct",
                Some("complex_ffi.rs:35".to_string())
            );
            destroy_c_struct(c_struct);
        }
    }
}
```

## 🛡️ 安全性分析

### 内存安全检查

```rust
use memscope_rs::analysis::UnsafeFFITracker;

fn analyze_memory_safety() {
    let mut tracker = UnsafeFFITracker::new();
    
    // 模拟一些 unsafe 操作
    unsafe {
        let ptr = Box::into_raw(Box::new(42));
        
        // 记录操作
        tracker.track_unsafe_operation(
            UnsafeOperationType::RawPointerDereference,
            "main.rs:100",
            Some("Potential use-after-free".to_string())
        ).unwrap();
        
        // 检查安全违规
        let violations = tracker.get_safety_violations();
        for violation in violations {
            println!("安全违规: {:?}", violation);
        }
        
        // 清理
        let _ = Box::from_raw(ptr);
    }
}
```

### FFI 边界分析

```rust
fn analyze_ffi_boundaries() {
    // 分析 Rust 和 C 之间的数据传递
    let rust_string = String::from("Hello, FFI!");
    let c_string = CString::new(rust_string.clone()).unwrap();
    
    track_var!(rust_string);
    track_var!(c_string);
    
    unsafe {
        // 传递给 C 函数
        let c_ptr = c_string.as_ptr();
        track_unsafe_operation(
            UnsafeOperationType::FFICall,
            "strlen",
            Some("Passing Rust string to C".to_string())
        );
        
        let len = strlen(c_ptr);
        println!("C 函数返回长度: {}", len);
    }
}
```

## 📊 分析报告

### 生成安全报告

```rust
fn generate_safety_report() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = memscope_rs::get_global_tracker();
    
    // 导出包含 unsafe/FFI 信息的报告
    tracker.export_to_json("unsafe_ffi_analysis")?;
    
    // 生成专门的安全报告
    let unsafe_tracker = UnsafeFFITracker::new();
    let safety_report = unsafe_tracker.generate_safety_report();
    
    println!("安全报告:");
    println!("  Unsafe 操作: {}", safety_report.unsafe_operations_count);
    println!("  FFI 调用: {}", safety_report.ffi_calls_count);
    println!("  安全违规: {}", safety_report.safety_violations_count);
    
    Ok(())
}
```

## 🔧 最佳实践

### 1. 最小化 Unsafe 使用

```rust
// ❌ 避免不必要的 unsafe
unsafe fn unnecessary_unsafe(data: &[i32]) -> i32 {
    data[0] // 这不需要 unsafe
}

// ✅ 只在必要时使用 unsafe
fn safe_access(data: &[i32]) -> Option<i32> {
    data.get(0).copied()
}
```

### 2. 封装 Unsafe 操作

```rust
pub struct SafeWrapper {
    inner: *mut CStruct,
}

impl SafeWrapper {
    pub fn new(size: usize) -> Option<Self> {
        unsafe {
            let ptr = create_c_struct(size);
            if ptr.is_null() {
                None
            } else {
                Some(Self { inner: ptr })
            }
        }
    }
    
    pub fn add_data(&mut self, data: &[u8]) -> Result<(), &'static str> {
        unsafe {
            let result = c_struct_add_data(
                self.inner,
                data.as_ptr(),
                data.len()
            );
            if result == 0 {
                Ok(())
            } else {
                Err("Failed to add data")
            }
        }
    }
}

impl Drop for SafeWrapper {
    fn drop(&mut self) {
        unsafe {
            destroy_c_struct(self.inner);
        }
    }
}
```

## 🎉 总结

Unsafe 和 FFI 跟踪帮助你：
- 识别潜在的安全问题
- 监控跨语言边界
- 分析内存安全违规
- 优化 FFI 性能