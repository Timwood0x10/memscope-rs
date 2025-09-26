# MemScope 用户指南

## 概览

MemScope 是一个高性能的 Rust 内存追踪分析工具，提供三种互补的追踪模式和交互式 HTML 仪表板生成，用于全面的内存分析。

## 三种追踪模式

### 1. 实时内存追踪（实时模式）
**用途**: 程序执行期间的实时内存分配追踪  
**适用场景**: 开发、调试、即时反馈

```rust
use memscope::core::tracker::MemoryTracker;
use std::sync::Arc;

// 初始化追踪器
let tracker = Arc::new(MemoryTracker::new());

// 实时追踪内存分配
tracker.track_allocation(ptr, size, Some("my_variable".to_string()))?;
```

### 2. 采样追踪（采样模式）
**用途**: 高性能采样，最小运行时开销  
**适用场景**: 生产环境、性能关键应用

```rust
use memscope::core::sampling_tracker::SamplingTracker;

// 初始化采样追踪器
let tracker = SamplingTracker::new("./output");

// 以较低开销追踪变量
tracker.track_variable(ptr, size, "var_name".to_string(), "String".to_string())?;
```

### 3. 二进制分析（事后分析模式）
**用途**: 分析之前追踪会话生成的二进制文件  
**适用场景**: 离线分析、CI/CD 流水线、详细调查

```rust
use memscope::export::api::Exporter;

// 将二进制文件转换为 HTML 仪表板
Exporter::binary_to_html("memory_data.memscope", "analysis.html")?;
```

## HTML 仪表板生成

### 主要导出 API

#### 1. 直接 HTML 导出（实时数据）
```rust
use memscope::export::clean_unified_api::export_html;

// 将当前追踪器状态导出为 HTML
export_html(tracker, "dashboard.html")?;
```

#### 2. 二进制转 HTML 转换
```rust
use memscope::core::tracker::memory_tracker::MemoryTracker;

// 将现有二进制文件转换为交互式 HTML
MemoryTracker::export_binary_to_html("data.memscope", "dashboard.html")?;
```

#### 3. 统一导出 API
```rust
use memscope::export::api::Exporter;

// 使用自定义配置创建导出器
let exporter = Exporter::new(allocations, stats, config);
let stats = exporter.export_html("output.html")?;
```

## 仪表板功能

生成的 HTML 仪表板包括：

### 1. 概览面板
- 总内存使用统计
- 活跃/已释放分配计数
- 性能指标和趋势
- 内存效率指标

### 2. 线程变量 (50/50) 卡片
交互式卡片显示：
- 变量名和生命周期阶段（🟢 活跃、🟡 已分配、🔄 共享、⚫ 已释放）
- 内存大小和分配次数
- 线程信息
- 性能类别（CPU/IO/内存/异步密集型）

### 3. 详细变量检查器
点击任何变量卡片可访问：
- **概览标签**: 基本信息和生命周期
- **生命周期标签**: 详细分配时间线
- **FFI Passport 标签**: 外部函数接口边界追踪
- **优化标签**: 性能建议

### 4. 内存映射可视化
- 基于线程的内存布局
- 变量大小的可视化表示
- 内存热点识别

### 5. 增强诊断
- 实时问题检测
- 内存泄漏模式识别
- 性能瓶颈识别
- 根因分析

## 可追踪的数据类型

### 核心分配信息
```rust
pub struct AllocationInfo {
    pub ptr: usize,                    // 内存地址
    pub size: usize,                   // 分配大小（字节）
    pub var_name: Option<String>,      // 变量名
    pub type_name: Option<String>,     // Rust 类型名
    pub thread_id: String,             // 线程标识符
    pub timestamp_alloc: u64,          // 分配时间戳
    pub timestamp_dealloc: Option<u64>, // 释放时间戳
    pub borrow_count: usize,           // 活跃借用计数
    pub stack_trace: Option<Vec<String>>, // 调用栈
    pub is_leaked: bool,               // 泄漏检测标志
    pub lifetime_ms: Option<u64>,      // 变量生命周期
    // ... 广泛的元数据字段
}
```

### 内存统计
```rust
pub struct MemoryStats {
    pub total_allocations: usize,      // 总分配计数
    pub active_allocations: usize,     // 当前活跃
    pub peak_memory: usize,            // 峰值内存使用
    pub leaked_allocations: usize,     // 检测到的泄漏
    pub fragmentation_analysis: FragmentationAnalysis,
    pub lifecycle_stats: ScopeLifecycleMetrics,
    // ... 全面的统计信息
}
```

## 模式协调工作流

### 开发阶段
1. **实时模式**: 使用实时追踪获取即时反馈
2. **导出为二进制**: 保存会话数据供后续分析
3. **生成 HTML**: 创建交互式仪表板进行详细调查

### 生产阶段
1. **采样模式**: 以最小开销部署
2. **收集二进制数据**: 随时间收集性能数据
3. **离线分析**: 转换为 HTML 进行事后分析

### 完整工作流示例
```rust
// 1. 开发期间的实时追踪
let tracker = Arc::new(MemoryTracker::new());
// ... 追踪分配 ...

// 2. 导出为二进制以供存档
tracker.export_user_binary("session.memscope")?;

// 3. 生成交互式 HTML 仪表板
MemoryTracker::export_binary_to_html("session.memscope", "analysis.html")?;

// 4. 稍后：生产的采样模式
let sampler = SamplingTracker::new("./prod_data");
// ... 生产追踪 ...

// 5. 分析生产数据
Exporter::binary_to_html("prod_data.memscope", "production_analysis.html")?;
```

## 使用建议

### 性能考虑
- **开发环境**: 使用实时模式获取最大可见性
- **测试环境**: 使用采样模式平衡性能和数据质量
- **生产环境**: 使用采样模式确保最小性能影响

### 数据管理
- 定期导出二进制文件以防数据丢失
- 使用版本控制追踪内存分析报告
- 建立内存性能基准线和告警阈值

### 故障排除
- 使用详细变量检查器深入分析特定内存问题
- 利用增强诊断进行自动问题检测
- 结合多个追踪会话进行趋势分析

## 快速开始示例

```rust
use memscope::core::tracker::MemoryTracker;
use memscope::export::clean_unified_api::export_html;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化追踪器
    let tracker = Arc::new(MemoryTracker::new());
    
    // 2. 您的应用程序代码与内存追踪
    let data = vec![1, 2, 3, 4, 5];
    tracker.track_allocation(
        data.as_ptr() as usize,
        data.len() * std::mem::size_of::<i32>(),
        Some("my_vector".to_string())
    )?;
    
    // 3. 导出为交互式 HTML 仪表板
    export_html(tracker, "memory_analysis.html")?;
    
    println!("内存分析已保存到 memory_analysis.html");
    Ok(())
}
```

## 最佳实践

### 开发阶段最佳实践
1. **启用详细追踪**: 在开发环境中使用实时模式获取最大的可见性
2. **定期导出数据**: 每日导出追踪数据以建立性能基准线
3. **变量命名规范**: 使用有意义的变量名以便在仪表板中识别

### 生产环境最佳实践
1. **最小化开销**: 使用采样模式确保性能不受影响
2. **监控内存趋势**: 设置自动化脚本定期生成 HTML 报告
3. **告警设置**: 基于内存泄漏检测设置告警系统

### 调试技巧
1. **利用生命周期分析**: 查看变量的完整生命周期以识别异常模式
2. **FFI 边界追踪**: 特别关注 FFI 调用边界的内存管理
3. **线程间分析**: 使用线程视图识别线程间的内存竞争问题