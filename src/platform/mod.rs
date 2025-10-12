//! Platform-specific implementations for memory tracking
//!
//! Provides optimized implementations for different operating systems
//! and architectures, ensuring maximum performance and compatibility.

pub mod allocator;
pub mod memory_info;
pub mod stack_walker;
pub mod symbol_resolver;

pub use allocator::{AllocationHook, HookResult, PlatformAllocator};
pub use memory_info::{MemoryStats, PlatformMemoryInfo, SystemInfo};
pub use stack_walker::{PlatformStackWalker, StackWalkConfig, WalkResult};
pub use symbol_resolver::{PlatformSymbolResolver, ResolverConfig, SymbolInfo};
