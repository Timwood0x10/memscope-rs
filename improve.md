1. 完善生命周期的统计信息
// In lifetime.json, linked by allocation_ptr
{
  "allocation_ptr": 7891234,
  "ownership_history": [
    // ... an array of OwnershipEvent objects
  ]
}

// Definition of an OwnershipEvent
{
  "timestamp": 1755004694594238100,
  "event_type": "Allocated", // Enum: Allocated, Cloned, Dropped, OwnershipTransferred, Borrowed, MutablyBorrowed
  "source_stack_id": 101,     // ID pointing to the call stack that triggered this event
  "details": {
    "clone_source_ptr": 4567890,     // Only for 'Cloned' event
    "transfer_target_var": "new_owner", // Only for 'OwnershipTransferred'
    "borrower_scope": "process_data_slice" // Only for borrow events
  }
}


提供高级生命周期的摘要
{
    // ... existing fields like ptr, size, type_name ...
    
    "lifetime_ms": 1520, // (You already have this) The total duration from alloc to dealloc.

    "borrow_info": {
      "immutable_borrows": 25, // Total number of immutable borrows during its lifetime.
      "mutable_borrows": 2,    // Total number of mutable borrows.
      "max_concurrent_borrows": 5, // The peak number of simultaneous borrows observed.
      "last_borrow_timestamp": 1755004694594239500
    },

    "clone_info": {
      "clone_count": 3, // How many times this object was cloned.
      "is_clone": true, // Whether this AllocationInfo itself is a result of a clone.
      "original_ptr": 1234567 // If is_clone is true, points to the original object.
    },
    
    "ownership_history_available": true // A flag to indicate if detailed history is in lifetime.json
}


2. unsafe_ffi.json - 精确的跨边界安全取证
这个文件专注于程序中所有 unsafe 块和 FFI 调用的风险评估与生命周期追踪。
核心数据结构: UnsafeReport
unsafe_ffi.json 将是一个包含 UnsafeReport 对象的数组，每个对象代表一个被分析的unsafe代码块或FFI调用。

```json

// An object in the top-level array of unsafe_ffi.json
{
  "report_id": "unsafe-report-uuid-001",
  "source": {
    "type": "UnsafeBlock", // Enum: UnsafeBlock, FfiDeclaration, FfiCallSite
    "location": "examples/unsafe_ffi_demo.rs:37:13"
  },
  
  "risk_assessment": { // (Your existing structure is excellent)
    "risk_level": "High",
    "confidence_score": 0.85,
    "risk_factors": [
      {
        "factor_type": "RawPointerDereference", // Expanded Enum
        "severity": 8.0,
        "description": "Dereferencing a raw pointer `*mut c_void`.",
        "source_location": "examples/unsafe_ffi_demo.rs:42:5"
      },
      {
        "factor_type": "FfiCall",
        "severity": 7.0,
        "description": "Call to external function `process_data_unsafe`.",
        "target_function": "process_data_unsafe", // More precise context
        "target_library": "my_c_library.so"
      }
    ],
    "mitigation_suggestions": [
      "Verify the pointer is non-null before dereferencing.",
      "Ensure the data pointed to is properly initialized and valid.",
      "Consider using a safer abstraction layer around this FFI call."
    ]
  },

  "dynamic_violations": [ // Replaces the old 'safety_violations' for clarity
    {
      "violation_type": "FfiMemoryLeak", // Enum: FfiMemoryLeak, DoubleFree, UseAfterFree
      "description": "Memory allocated at 0xABC123 and passed to FFI was not reclaimed.",
      "passport_id": "passport-uuid-123" // Link to the MemoryPassport
    }
  ],

  "related_passports": [ "passport-uuid-123", "passport-uuid-124" ] // All passports originating from this block
}
```

核心数据结构: MemoryPassport
文件的第二部分将包含所有的内存护照。

```json
// An object in the 'memory_passports' section/file
{
  "passport_id": "passport-uuid-123",
  "allocation_ptr": 11223344, // The original Rust allocation this passport tracks
  "size_bytes": 1024,
  
  "status_at_shutdown": "InForeignCustody", // Final Status: Reclaimed, FreedByForeign, InForeignCustody (Leaked)

  "lifecycle_events": [
    {
      "event_type": "CreatedAndHandedOver",
      "timestamp": 1755004694594238100,
      "how": "Box::into_raw",
      "source_stack_id": 105, // Where the handover happened
      "ffi_call": {
          "report_id": "unsafe-report-uuid-001", // Link to the UnsafeReport
          "target_function": "process_data_unsafe"
      }
    },
    // If the memory is later freed by an external function...
    {
      "event_type": "FreedByForeign",
      "timestamp": 1755004694594239900,
      "source_stack_id": 110, // The stack of the FFI call that *triggered* the free
      "ffi_call": {
        "report_id": "unsafe-report-uuid-002",
        "target_function": "cleanup_data_unsafe"
      }
    },
    // Or if it's reclaimed by Rust...
    {
      "event_type": "ReclaimedByRust",
      "timestamp": 1755004694594239900,
      "how": "Box::from_raw",
      "source_stack_id": 112
    }
  ]
}

```
清晰分离: UnsafeReport 关注代码块的静态风险和已发生的动态违规。MemoryPassport 则专注于跨边界内存的动态生命周期。
强关联性: 通过 passport_id 和 report_id，可以在 UnsafeReport 和 MemoryPassport 之间轻松跳转，形成完整的证据链。
根本原因分析: 当发现一个 FfiMemoryLeak 违规时，可以直接通过其 passport_id 找到对应的护照，查看其完整的 lifecycle_events，从而精确定位是哪个 HandoverToFfi 事件出了问题，以及本应调用哪个 cleanup 函数而没有调用。这是解决 FFI 内存问题的终极方案。


memory_passport (内存护照) 的设计构想:
目标: 追踪一块跨越FFI边界的内存的完整生命周期。
当Rust通过Box::into_raw创建一个指针并准备传给C时，为其创建一个“护照”。

```json
"memory_passport": {
  "passport_id": "passport-uuid-123",
  "source_alloc_event_id": 42, // 指向分配该内存的Alloc事件
  "status": "InForeignCustody", // 状态: InRust, InForeignCustody, FreedByForeign, ReclaimedByRust
  "events": [
    {
      "type": "HandoverToFfi",
      "timestamp": 1755004694594,
      "ffi_call_event_id": 88 // 指向触发这次递交的FFI调用事件
    }
  ]
}
```

当/如果这块内存被C代码释放，或者被Rust代码回收(Box::from_raw)，就在这个“护照”上追加新的事件。如果程序结束时，护照状态依然是InForeignCustody，那么这就是一个确凿的内存泄漏。
ownership_history (所有权历史) 的设计构想:
目标: 可视化一个值在unsafe块附近的所有权和状态变化。
形式: 一个针对特定内存或变量的事件链。

```json
"ownership_history": {
  "variable_name": "my_box",
  "history": [
    { "timestamp": ..., "operation": "Box::new", "state": "Owned" },
    { "timestamp": ..., "operation": "Box::into_raw", "state": "RawPointer" },
    { "timestamp": ..., "operation": "passed_to_ffi", "state": "BorrowedByFfi" },
    { "timestamp": ..., "operation": "Box::from_raw", "state": "ReclaimedOwned" },
    { "timestamp": ..., "operation": "drop", "state": "Dropped" }
  ]
}
```    

**价值:** 这能极大地帮助开发者理解复杂的`unsafe`代码块中的资源流动，防止悬垂指针或双重释放。


2.1 结构优化与信息去重
问题: call_stack 字段在顶层、source.UnsafeRust内部、以及 cross_boundary_events 内部出现了三次，并且内容完全一样。这造成了数据冗余。
建议: 采用我们之前讨论的**归一化（ID引用）**策略。
在报告的顶层（或lookup_tables.bin中）定义一个call_stacks数组。
所有需要引用该调用栈的地方，只存储一个ID，例如 "call_stack_id": 0。
好处: 减小体积，并确保了信息的一致性。
2.2 增强上下文的精确度
问题: cross_boundary_events 中的 to_context: "potential_ffi_target" 有些模糊。
建议: 如果可能的话，通过分析链接信息或调试信息，尝试解析出具体的FFI函数名。

```json
"to_context": {
  "type": "FfiTarget",
  "library_name": "libc.so.6",
  "function_name": "malloc"
}
```

问题: risk_factors 中的 "factor_type": "ManualMemoryManagement" 很好，但 unsafe 的风险不止于此。
建议: 扩展 factor_type 的枚举范围，例如：
RawPointerDereference: 裸指针解引用
UnsafeDataRace: 访问static mut等可能导致数据竞争的操作
InvalidTransmute: 不安全的类型转换 mem::transmute
FfiCall: 调用外部函数本身就是一种风险


