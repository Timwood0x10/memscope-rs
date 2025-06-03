// 文件: src/allocator.rs

use std::alloc::{GlobalAlloc, Layout, System};
// 从 `tracker` 模块导入线程局部标志和全局追踪器访问函数
// **请确认这里的路径 `crate::tracker` 与您实际的模块结构匹配**
// 例如，如果 `allocator.rs` 和 `tracker.rs` 都在 `src` 目录下，则 `crate::tracker` 是正确的。
use crate::tracker::{get_global_tracker, IS_ALLOCATOR_REENTRANT_CALL};

// 您的全局分配器实例。由于 TrackingAllocator 是一个单元结构体，
// 它不持有内部状态，所以可以直接用 `TrackingAllocator`。
#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

/// 自定义的全局内存分配器，用于跟踪内存分配和释放。
/// 它包装了 `System` 分配器作为回退，并使用 `MemoryTracker` 记录事件。
pub struct TrackingAllocator;

impl TrackingAllocator {
    /// 创建一个新的 TrackingAllocator 实例。
    /// 对于 #[global_allocator]，这个 `new` 通常不会被直接调用，但作为 `const fn` 有其用处。
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
        // 这将把 `IS_ALLOCATOR_REENTRANT_CALL` 设置为 `true`。
        // `MemoryTracker` 中的 `track_allocation` 方法将检查此标志，
        // 并在它为 `true` 时跳过 `backtrace::trace` 调用，从而避免 TLS 错误。
        let _guard = AllocatorReentrantGuard::new();

        // 首先，使用系统分配器执行实际的内存分配。
        let ptr = System.alloc(layout);

        // 如果分配成功，则尝试通知 MemoryTracker。
        // 这里的错误处理是静默的（`let _ = ...`），以避免 `eprintln!` 引起的重入。
        if !ptr.is_null() {
            // 获取全局追踪器的 Arc 克隆。
            // `get_global_tracker()` 返回 `Arc<MemoryTracker>`，克隆它开销很小，且不会重新初始化 `MemoryTracker`。
            let tracker = get_global_tracker();
            // 调用 MemoryTracker 的 `track_allocation`。
            // 由于 `_guard` 已经设置了 `IS_ALLOCATOR_REENTRANT_CALL`，
            // `track_allocation` 会跳过 `backtrace::trace`。
            let _ = tracker.track_allocation(
                ptr as usize,
                layout.size(),
                None, // 在分配器层面，我们通常无法得知具体的 Rust 类型名称
            );
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // **最关键的修改：** 立即创建重入守卫。
        let _guard = AllocatorReentrantGuard::new();

        // 如果指针有效，则尝试通知 MemoryTracker 内存即将被释放。
        if !ptr.is_null() {
            let tracker = get_global_tracker();
            // `track_deallocation` 通常不捕获回溯，但设置守卫依然是良好的实践，
            // 以防未来添加此类逻辑或其内部操作触发分配。
            let _ = tracker.track_deallocation(ptr as usize);
        }

        // 最后，执行实际的内存释放。
        System.dealloc(ptr, layout);
    }
}
