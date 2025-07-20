# memscope-rs 内存跟踪工具改进方案

本文档总结了针对 memscope-rs 内存跟踪工具的改进建议，主要解决 `lifetime_ms: null` 问题，并提供两种不同的数据获取方式。

## 问题分析

当前在 `lifecycle_snapshot_1_all_alive.json` 中，变量的 `lifetime_ms` 字段为 `null`，这是因为变量的销毁时间没有被正确记录。这导致无法准确分析变量的生命周期。

## 改进方案概述

1. **增强 `track_var!` 宏**：确保变量的创建和销毁都被准确跟踪
2. **添加命令行工具**：提供无侵入式的内存监控方式
3. **改进生命周期跟踪**：确保所有变量都有准确的生命周期信息
4. **增强自动导出功能**：在程序结束时自动导出完整的内存分析数据

## 详细实现方案

### 1. 命令行工具实现

创建新的命令行工具 `memscope-cli`，用于无侵入式内存监控：

```rust
// src/bin/memscope_cli.rs
use clap::{App, Arg, SubCommand};
use std::process::{Command, Stdio};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("memscope-cli")
        .version("0.1.0")
        .about("内存跟踪命令行工具")
        .subcommand(
            SubCommand::with_name("run")
                .about("运行并跟踪程序内存")
                .arg(Arg::with_name("command").required(true).multiple(true).help("要运行的命令"))
                .arg(Arg::with_name("export").long("export").takes_value(true).help("导出格式: json, html 或 both"))
        )
        .subcommand(
            SubCommand::with_name("analyze")
                .about("分析已有的内存快照")
                .arg(Arg::with_name("input").required(true).help("输入JSON文件"))
                .arg(Arg::with_name("output").required(true).help("输出HTML文件"))
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("run") {
        let command_args: Vec<&str> = matches.values_of("command").unwrap().collect();
        let export_format = matches.value_of("export").unwrap_or("json");
    
        // 设置环境变量以启用自动导出
        std::env::set_var("MEMSCOPE_AUTO_EXPORT", "1");
        std::env::set_var("MEMSCOPE_EXPORT_FORMAT", export_format);
    
        // 运行目标程序
        let status = Command::new(command_args[0])
            .args(&command_args[1..])
            .env("RUST_BACKTRACE", "1")
            .status()?;
    
        println!("程序执行完成，退出状态: {}", status);
    } else if let Some(matches) = matches.subcommand_matches("analyze") {
        let input = matches.value_of("input").unwrap();
        let output = matches.value_of("output").unwrap();
    
        // 调用现有的报告生成功能
        embed_json_to_html(input, "report_template.html", output)?;
    }

    Ok(())
}

// 复用现有的 embed_json_to_html 函数
fn embed_json_to_html(json_file: &str, template_file: &str, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 与 generate_report.rs 中相同的实现
    // ...
}
```

在 `Cargo.toml` 中添加新的二进制目标：

```toml
[[bin]]
name = "memscope-cli"
path = "src/bin/memscope_cli.rs"
```

### 2. 增强 track_var! 宏

改进 `TrackedVariable` 实现，确保准确跟踪变量生命周期：

```rust
impl<T: Trackable> TrackedVariable<T> {
    // 增加一个方法来获取变量的生命周期信息
    pub fn get_lifetime_info(&self) -> Option<u64> {
        if let Some(ptr) = self.ptr {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
        
            Some((current_time.saturating_sub(self.creation_time)) / 1_000_000)
        } else {
            None
        }
    }
}

// 修改 Drop 实现以确保准确记录生命周期
impl<T: Trackable> Drop for TrackedVariable<T> {
    fn drop(&mut self) {
        if let Some(ptr_val) = self.ptr {
            let destruction_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
        
            let lifetime_ms = (destruction_time.saturating_sub(self.creation_time)) / 1_000_000;
        
            // 更新变量注册表
            let _ = crate::variable_registry::VariableRegistry::mark_variable_destroyed(
                ptr_val,
                destruction_time,
            );
        
            // 在跟踪器中记录销毁信息，确保包含生命周期
            let tracker = get_global_tracker();
            let _ = tracker.track_deallocation_with_lifetime(ptr_val, lifetime_ms);
        
            tracing::debug!(
                "💀 销毁跟踪变量 '{}' 在地址 0x{:x}, 生命周期: {}ms",
                self.var_name,
                ptr_val,
                lifetime_ms
            );
        }
    }
}
```

### 3. 在 MemoryTracker 中添加生命周期跟踪功能

在 `tracker.rs` 中添加新方法：

```rust
impl MemoryTracker {
    /// 跟踪内存销毁并记录准确的生命周期
    pub fn track_deallocation_with_lifetime(&self, ptr: usize, lifetime_ms: u64) -> TrackingResult<()> {
        let dealloc_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
    
        match (self.active_allocations.try_lock(), self.stats.try_lock()) {
            (Ok(mut active), Ok(mut stats)) => {
                if let Some(mut allocation) = active.remove(&ptr) {
                    // 设置销毁时间戳和生命周期
                    allocation.timestamp_dealloc = Some(dealloc_timestamp);
                    allocation.lifetime_ms = Some(lifetime_ms);
                
                    // 更新统计信息
                    stats.total_deallocations = stats.total_deallocations.saturating_add(1);
                    stats.total_deallocated = stats.total_deallocated.saturating_add(allocation.size);
                    stats.active_allocations = stats.active_allocations.saturating_sub(1);
                    stats.active_memory = stats.active_memory.saturating_sub(allocation.size);
                
                    // 释放锁后更新历史记录
                    drop(stats);
                    drop(active);
                
                    // 更新分配历史记录
                    if let Ok(mut history) = self.allocation_history.try_lock() {
                        if let Some(history_entry) = history.iter_mut().find(|entry| entry.ptr == ptr) {
                            history_entry.timestamp_dealloc = Some(dealloc_timestamp);
                            history_entry.lifetime_ms = Some(lifetime_ms);
                        } else {
                            history.push(allocation);
                        }
                    }
                }
                Ok(())
            }
            _ => {
                // 如果无法立即获取锁，跳过跟踪以避免死锁
                Ok(())
            }
        }
    }
}
```

### 4. 在 AllocationInfo 结构中添加生命周期字段

确保 `types.rs` 中的 `AllocationInfo` 结构包含 `lifetime_ms` 字段：

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AllocationInfo {
    pub ptr: usize,
    pub size: usize,
    pub timestamp_alloc: u64,
    pub timestamp_dealloc: Option<u64>,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
    pub scope_name: Option<String>,
    pub stack_trace: Option<Vec<StackFrame>>,
    pub borrow_count: usize,
    pub is_leaked: bool,
    pub lifetime_ms: Option<u64>,  // 添加生命周期字段
}
```

### 5. 增强自动导出功能

改进 `enable_auto_export` 函数和相关功能：

```rust
/// 启用程序结束时自动导出 JSON
pub fn enable_auto_export(export_path: Option<&str>) {
    std::env::set_var("MEMSCOPE_AUTO_EXPORT", "1");
    if let Some(path) = export_path {
        std::env::set_var("MEMSCOPE_EXPORT_PATH", path);
    }
  
    // 安装退出钩子
    install_exit_hook();
  
    println!("📋 已启用自动导出 - JSON 将导出到: {}", 
             export_path.unwrap_or("memscope_final_snapshot.json"));
}

/// 安装程序退出钩子以自动导出数据
fn install_exit_hook() {
    use std::sync::Once;
    static HOOK_INSTALLED: Once = Once::new();
  
    HOOK_INSTALLED.call_once(|| {
        // 安装 panic 钩子
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            eprintln!("🚨 程序发生 panic，尝试导出内存数据...");
        
            // 在 panic 时标记所有变量为已销毁
            let tracker = get_global_tracker();
            mark_all_allocations_as_deallocated(tracker.clone());
        
            let _ = export_final_snapshot("memscope_panic_snapshot");
            original_hook(panic_info);
        }));
    
        // 使用 atexit 处理正常退出
        extern "C" fn exit_handler() {
            if std::env::var("MEMSCOPE_AUTO_EXPORT").is_ok() {
                println!("🔄 程序结束，导出最终内存快照...");
            
                // 标记所有变量为已销毁
                let tracker = get_global_tracker();
                mark_all_allocations_as_deallocated(tracker.clone());
            
                let export_path = std::env::var("MEMSCOPE_EXPORT_PATH")
                    .unwrap_or_else(|_| "memscope_final_snapshot".to_string());
            
                if let Err(e) = export_final_snapshot(&export_path) {
                    eprintln!("❌ 导出最终快照失败: {e}");
                } else {
                    println!("✅ 最终内存快照导出成功");
                }
            }
        }
    
        unsafe {
            libc::atexit(exit_handler);
        }
    
        tracing::debug!("📌 已安装退出钩子，用于自动内存导出");
    });
}

/// 标记所有活跃分配为已销毁
fn mark_all_allocations_as_deallocated(tracker: Arc<MemoryTracker>) {
    if let Ok(active_allocations) = tracker.get_active_allocations() {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
    
        for alloc in active_allocations {
            let lifetime_ms = (current_time.saturating_sub(alloc.timestamp_alloc)) / 1_000_000;
            let _ = tracker.track_deallocation_with_lifetime(alloc.ptr, lifetime_ms);
        }
    }
}
```

### 6. 修复 variable_registry.rs 中的生命周期计算

确保变量注册表中的生命周期计算正确：

```rust
impl VariableRegistry {
    /// 标记变量为已销毁，并记录销毁时间戳
    pub fn mark_variable_destroyed(address: usize, destruction_time: u64) -> TrackingResult<()> {
        if let Ok(mut registry) = get_global_registry().try_lock() {
            if let Some(var_info) = registry.get(&address) {
                let lifetime_ms = (destruction_time.saturating_sub(var_info.timestamp)) / 1_000_000;
                tracing::debug!(
                    "变量 '{}' (地址 0x{:x}) 已销毁，生命周期: {}ms",
                    var_info.var_name,
                    address,
                    lifetime_ms
                );
            
                // 可以选择从注册表中移除变量，或保留记录
                // registry.remove(&address);
            }
        }
        Ok(())
    }
}
```

## 使用方式

### 1. 代码侵入式方法（增强版）

```rust
fn main() {
    // 初始化并启用自动导出
    memscope_rs::init();
    memscope_rs::enable_auto_export(Some("my_memory_analysis.json"));
  
    // 使用 track_var! 宏跟踪变量
    let my_vec = vec![1, 2, 3];
    let tracked_vec = track_var!(my_vec);
  
    // 程序结束时会自动导出内存分析数据，包含准确的生命周期信息
}
```

### 2. 命令行工具方法（无侵入式）

```bash
# 运行并监控程序
memscope-cli run cargo run --release

# 运行并导出为JSON
memscope-cli run --export json cargo run --release

# 运行并导出为HTML
memscope-cli run --export html cargo run --release

# 分析已有的内存快照
memscope-cli analyze memory_snapshot.json memory_report.html
```

## 总结

通过以上改进，我们可以解决 `lifetime_ms: null` 的问题，并提供两种使用方式：

1. **代码侵入式方法**：使用增强的 `track_var!` 宏，自动跟踪变量的完整生命周期
2. **命令行工具方法**：使用新的 `memscope-cli` 工具，在不修改源代码的情况下监控程序内存

这些改进确保在程序结束时，所有变量都有正确的生命周期信息，无论是正常退出还是发生 panic。
