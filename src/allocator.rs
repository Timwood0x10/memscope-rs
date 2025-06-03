// 文件: src/allocator.rs

use std::alloc::{GlobalAlloc, Layout, System};
// 从 `tracker` 模块导入线程局部标志
use crate::tracker::IS_ALLOCATOR_REENTRANT_CALL;
// **关键：从您的 `lib.rs` 或主入口文件导入新的安全访问函数**
use crate::get_global_tracker_for_allocator;

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

/// 自定义的全局内存分配器，用于跟踪内存分配和释放。
/// 它包装了 `System` 分配器作为回退，并使用 `MemoryTracker` 记录事件。
pub struct TrackingAllocator;

impl TrackingAllocator {
    pub const fn new() -> Self {
        Self
    }
}

/// 一个 RAII 守卫，用于管理 `IS_ALLOCATOR_REENTRANT_CALL` 线程局部标志。
/// 这确保了即使发生 panic，标志也能被正确设置和重置。
struct AllocatorReentrantGuard {
    old_value: bool,
}

impl AllocatorReentrantGuard {
    /// 设置线程局部 `IS_ALLOCATOR_REENTRANT_CALL` 标志为 `true`，
    /// 并返回一个将在 `Drop` 时恢复旧值的守卫。
    /// 这必须在 `GlobalAlloc` 方法的入口处立即调用。
    fn new() -> AllocatorReentrantGuard {
        IS_ALLOCATOR_REENTRANT_CALL.with(|cell| {
            let old_value = cell.replace(true); // 设置为 true，并获取旧值
            AllocatorReentrantGuard { old_value } // 存储原始状态以便后续恢复
        })
    }
}

impl Drop for AllocatorReentrantGuard {
    /// 当守卫被丢弃时，将线程局部标志恢复到其原始值。
    /// 这保证了无论 `alloc` 或 `dealloc` 方法如何退出（正常返回或 panic），
    /// 标志都会被正确清理。
    fn drop(&mut self) {
        IS_ALLOCATOR_REENTRANT_CALL.with(|cell| {
            cell.set(self.old_value); // 恢复到此守卫创建之前的值
        });
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // **最关键的修改：** 立即创建重入守卫。
        let _guard = AllocatorReentrantGuard::new();

        // 首先，使用系统分配器执行实际的内存分配。
        let ptr = System.alloc(layout);

        // 如果分配成功，则尝试通知 MemoryTracker。
        // 现在我们使用 `get_global_tracker_for_allocator()`，它返回 `Option`。
        // 如果 `MemoryTracker` 尚未初始化，就安全地跳过跟踪。
        if !ptr.is_null() {
            if let Some(tracker) = get_global_tracker_for_allocator() {
                // 只有当 tracker 确实存在（已初始化）时才进行跟踪。
                // 此时，`IS_ALLOCATOR_REENTRANT_CALL` 标志为 `true`，
                // `tracker.track_allocation` 会跳过 `backtrace` 和 `tracing` 宏。
                let _ = tracker.track_allocation(
                    ptr as usize,
                    layout.size(),
                    None, // 在分配器层面，我们通常无法得知具体的 Rust 类型名称
                );
            }
            // else {
            //     // 追踪器尚未准备好，在非常早期的启动阶段跳过此次追踪。
            //     // 避免在这里添加任何可能分配内存或访问 TLS 的调试日志。
            // }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // **最关键的修改：** 立即创建重入守卫。
        let _guard = AllocatorReentrantGuard::new();

        // 如果指针有效，则尝试通知 MemoryTracker 内存即将被释放。
        if !ptr.is_null() {
            if let Some(tracker) = get_global_tracker_for_allocator() {
                let _ = tracker.track_deallocation(ptr as usize);
            }
            // else {
            //     // 追踪器尚未准备好，在非常早期的启动阶段跳过此次追踪。
            // }
        }

        // 最后，执行实际的内存释放。
        System.dealloc(ptr, layout);
    }
}
