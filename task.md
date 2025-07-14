# Unsafe Rust 和 FFI 内存统计与可视化方案

## 📋 概述

本文档总结了针对 `unsafe` Rust 代码和 FFI 代码的内存统计与分析方案，重点关注如何在现有的 SVG 可视化工具中集成这些高级内存监控功能。

## 🎯 Part 1: Unsafe Rust 内存统计与分析

### 1.1 核心挑战

在 `unsafe` 块中，开发者可以：
- 解引用裸指针 (`*const T`, `*mut T`)
- 调用 unsafe 函数，包括直接与内存分配器交互的函数 (`std::alloc::alloc`, `std::alloc::dealloc`)
- 实现 unsafe 的 Trait
- 访问和修改静态可变变量

**关键问题**：当代码使用 `std::alloc::alloc()` 进行手动内存管理时，我们的工具必须能监督这个过程，确保没有内存安全问题。

### 1.2 监控策略：全局分配器监督

**核心思路**：无论 unsafe 代码如何操作，只要它从标准堆获取内存，最终都必须通过 Rust 的全局分配器。

#### 追踪裸指针的"出身"
- 在 `GlobalAlloc` 的 `alloc` 方法中记录完整调用栈
- 将分配返回的裸指针作为 Key 存入"活跃分配表"
- 扩充 `AllocationInfo` 结构体，增加 `source` 字段标记为 `Source::UnsafeRust`

#### 监控裸指针的"归宿"
- 在 `dealloc` 方法中检查指针是否存在于活跃分配表
- **检查一（双重释放/无效释放）**：不存在则标记为严重 Bug
- **检查二（内存泄漏）**：程序结束时未释放的 unsafe 分配

### 1.3 SVG 可视化方案

#### 调用栈"DNA 双螺旋"可视化
```rust
pub fn create_unsafe_callstack_svg(allocation: &AllocationInfo) -> String {
    format!(r#"
    <svg viewBox="0 0 800 400">
        <!-- 调用栈的"DNA 双螺旋"可视化 -->
        <g id="callstack-helix">
            <!-- 安全 Rust 代码：绿色螺旋 -->
            <path d="M100,200 Q200,150 300,200 Q400,250 500,200" 
                  stroke="#2ecc71" stroke-width="4" fill="none"/>
            
            <!-- unsafe 边界：红色警告线 -->
            <line x1="500" y1="50" x2="500" y2="350" 
                  stroke="#e74c3c" stroke-width="6" stroke-dasharray="10,5"/>
            <text x="510" y="200" fill="#e74c3c" font-weight="bold">UNSAFE BOUNDARY</text>
            
            <!-- unsafe 代码：橙色螺旋 -->
            <path d="M500,200 Q600,150 700,200" 
                  stroke="#f39c12" stroke-width="4" fill="none"/>
            
            <!-- 分配点：脉冲动画 -->
            <circle cx="700" cy="200" r="8" fill="#e74c3c">
                <animate attributeName="r" values="8;12;8" dur="1s" repeatCount="indefinite"/>
            </circle>
        </g>
    </svg>
    "#)
}
```

#### 内存"健康检查"仪表板
- **安全等级指示器**：圆形进度条显示内存安全百分比
- **错误计数器**：显示双重释放和内存泄漏数量
- **实时状态更新**：动态反映当前内存安全状况

## 🌉 Part 2: FFI 内存统计与分析

### 2.1 核心挑战

- **分配器不同源**：FFI 调用可能使用 C 库的 `malloc()`，绕过 Rust 的 GlobalAlloc
- **所有权不明确**：内存应该由谁释放？约定存在于逻辑中而非语言层面
- **数据结构未知**：C 库返回不透明指针，无法知道结构体大小

### 2.2 监控策略：拦截 libc

#### 实现 libc 函数钩子
- 使用平台特定技术（如 Linux 的 `LD_PRELOAD`）
- 提供自定义的 `malloc`, `calloc`, `realloc`, `free` 实现
- 记录分配事件后调用真正的 libc 函数

#### 建立统一的"跨语言内存账本"
- 扩展 `AllocationInfo.source` 字段：
  - `Source::RustSafe`
  - `Source::UnsafeRust` 
  - `Source::FFI_C`

#### 追踪跨语言所有权转移
- 检测内存在 Rust ↔ FFI 之间的流动
- 识别"跨界释放"模式
- 检测 FFI 内存泄漏

### 2.3 SVG 可视化方案

#### 内存"护照"系统
```rust
pub fn create_memory_passport_svg(allocation: &CrossBoundaryAllocation) -> String {
    format!(r#"
    <svg viewBox="0 0 1000 300">
        <!-- 内存的"旅行路线" -->
        <g id="memory-journey">
            <!-- Rust 领土 -->
            <rect x="50" y="50" width="300" height="200" fill="#2ecc71" opacity="0.2" rx="10"/>
            <text x="200" y="40" text-anchor="middle" font-weight="bold">RUST TERRITORY</text>
            
            <!-- 边界检查站 -->
            <rect x="350" y="100" width="100" height="100" fill="#f39c12" opacity="0.3" rx="5"/>
            <text x="400" y="145" text-anchor="middle" font-size="12">FFI</text>
            <text x="400" y="160" text-anchor="middle" font-size="12">BORDER</text>
            
            <!-- C 领土 -->
            <rect x="450" y="50" width="300" height="200" fill="#3498db" opacity="0.2" rx="10"/>
            <text x="600" y="40" text-anchor="middle" font-weight="bold">C LIBRARY TERRITORY</text>
            
            <!-- 内存块的旅行路径 -->
            <path d="M600,150 L400,150" stroke="#e74c3c" stroke-width="3" 
                  marker-end="url(#arrowhead)" stroke-dasharray="5,5">
                <animate attributeName="stroke-dashoffset" values="0;-10" dur="1s" repeatCount="indefinite"/>
            </path>
        </g>
    </svg>
    "#)
}
```

#### 实时内存"雷达"监控
- **雷达扫描动画**：显示不同类型内存的"信号"
- **目标分类**：
  - 绿色点：Rust 分配
  - 红色点：FFI 分配  
  - 黄色点：跨界内存
- **实时信息显示**：追踪数量和警告信息

## 🔧 技术实现要点

### 3.1 扩展数据结构

```rust
#[derive(Debug, Clone)]
pub struct EnhancedAllocationInfo {
    // 原有字段
    pub var_name: Option<String>,
    pub size: usize,
    pub timestamp_alloc: u128,
    pub timestamp_dealloc: Option<u128>,
    
    // 新增字段
    pub source: AllocationSource,
    pub call_stack: Vec<StackFrame>,
    pub cross_boundary_events: Vec<BoundaryEvent>,
    pub safety_violations: Vec<SafetyViolation>,
}

#[derive(Debug, Clone)]
pub enum AllocationSource {
    RustSafe,
    UnsafeRust { unsafe_block_location: String },
    FFI_C { library_name: String, function_name: String },
    CrossBoundary { from: Box<AllocationSource>, to: Box<AllocationSource> },
}

#[derive(Debug, Clone)]
pub enum SafetyViolation {
    DoubleFree { first_free_stack: Vec<StackFrame> },
    InvalidFree { attempted_pointer: usize },
    PotentialLeak { allocation_stack: Vec<StackFrame> },
    CrossBoundaryRisk { risk_level: RiskLevel },
}
```

### 3.2 交互式错误报告

```rust
pub fn create_interactive_error_panel(violations: &[SafetyViolation]) -> String {
    // 为每个安全违规创建可点击的 SVG 元素
    // 支持显示详细的调用栈信息
    // 用不同颜色和图标区分错误类型
}
```

## 📋 实施计划

### 阶段 1：Unsafe 内存监控
1. 增强现有的 GlobalAlloc 钩子
2. 添加调用栈追踪
3. 实现安全违规检测
4. 创建 SVG 可视化组件

### 阶段 2：SVG 可视化增强  
1. 实现"DNA 双螺旋"调用栈可视化
2. 添加内存健康仪表板
3. 创建交互式错误面板

### 阶段 3：FFI 内存监控
1. 实现 libc 函数钩子（平台特定）
2. 建立跨语言内存账本
3. 检测跨界内存操作

### 阶段 4：跨界分析可视化
1. 实现内存"护照"系统
2. 添加实时雷达监控
3. 创建跨界事件时间线

## 🎯 预期收益

1. **全面的内存安全监控**：覆盖 safe Rust、unsafe Rust 和 FFI 代码
2. **直观的可视化界面**：通过 SVG 动画和交互元素提升用户体验
3. **实时错误检测**：及时发现双重释放、内存泄漏等问题
4. **跨语言内存分析**：理解复杂的内存所有权转移模式

---

*本方案结合了深度的技术分析和创新的可视化设计，为 Rust 生态系统提供了一个全面的内存分析解决方案。*