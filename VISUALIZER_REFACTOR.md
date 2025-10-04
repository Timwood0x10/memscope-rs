# Visualizer.rs 结构分析和重构计划

## 当前文件结构分析

### 文件大小: 3152行

### 主要组成部分:

#### 1. **数据结构定义** (第1-112行)
```rust
#[derive(Serialize, Debug)]
struct DashboardData { ... }

#[derive(Serialize, Debug, Clone)]
struct ThreadData { ... }

#[derive(Serialize, Debug)]
struct ThreadPerformanceData { ... }

#[derive(Serialize, Debug)]
struct ThreadAllocationPattern { ... }

#[derive(Serialize, Debug)]
struct ResourceSample { ... }

#[derive(Serialize, Debug)]
struct CpuCoreData { ... }

#[derive(Serialize, Debug)]
struct ThreadDetailData { ... }
```

#### 2. **公共API函数** (第114-144行)
- `generate_comprehensive_html_report()` - 生成完整的HTML报告到文件
- `build_comprehensive_html_report_with_template()` - 使用模板生成HTML

#### 3. **核心数据处理函数** (第146-315行)
- `build_template_data()` - 从分析结果构建模板数据

#### 4. **辅助函数** (第317-353行)
- `classify_thread_role()` - 线程角色分类
- `get_role_display()` - 角色显示信息
- `determine_alert_level()` - 警报级别确定

#### 5. **硬编码的HTML和CSS** (第355-约1000行)
- 大量的内联CSS样式
- 硬编码的HTML结构

#### 6. **JavaScript代码** (约1000-约2500行)
- 大量的交互逻辑JavaScript代码
- 图表生成代码
- 线程分析功能

#### 7. **具体构建函数** (约2500-3152行)
- `build_comprehensive_html_report()` - 主要的HTML构建函数
- `build_multi_thread_overview_tab()` - 多线程概览标签
- `build_thread_details_tab()` - 线程详情标签
- `build_resource_timeline_tab()` - 资源时间线标签
- `build_system_summary_tab()` - 系统摘要标签

## 重构计划

### 目标:
1. 删除所有硬编码的HTML和CSS
2. 删除内联的JavaScript代码
3. 使用外部模板文件
4. 用Rust处理真实数据
5. 保持原有的模板占位符兼容性

### 重构步骤:

#### 第一阶段: 数据结构优化
- 添加新的数据结构支持动态内容
- 优化现有的数据结构以匹配模板占位符

#### 第二阶段: 模板引擎集成
- 使用handlebars模板引擎
- 加载外部模板文件
- 渲染模板数据

#### 第三阶段: 数据处理逻辑重写
- 重写`build_template_data()`函数
- 添加真实的数据处理逻辑
- 移除模拟数据生成

#### 第四阶段: 函数清理
- 删除硬编码的HTML构建函数
- 保留核心的公共API函数
- 重写主要的HTML生成函数

### 模板占位符需求:
根据模板文件，需要以下数据结构:

```rust
struct MemoryPatternsData {
    thread_id: u32,
    allocations: usize,
    avg_size: f32,
    efficiency: f32,
}

struct CpuCoresData {
    name: String,  // "Core 0", "Core 1", etc.
    usage: f32,
    level: String, // "low", "medium", "high"
}
```

### 重构后的文件结构:
```rust
// 数据结构定义
// 公共API函数
// 模板数据构建函数
// 辅助函数
// 结束
```

### 预期结果:
- 文件大小从3152行减少到约500行
- 完全使用模板渲染
- 支持真实数据处理
- 保持向后兼容性
```

## 具体实现步骤

### 1. 删除硬编码内容
- 删除第355-约2500行的所有硬编码HTML、CSS和JavaScript
- 只保留数据结构和核心函数

### 2. 重写模板数据构建
- 修改`build_template_data()`函数以处理真实数据
- 添加对新数据结构的处理

### 3. 更新公共API
- 确保`build_comprehensive_html_report_with_template()`正常工作
- 保持与现有模板的兼容性

### 4. 添加新的辅助函数
- 添加处理动态数据的辅助函数
- 优化数据转换逻辑

这个重构将使代码更清洁、可维护，并完全支持真实数据的处理和模板渲染。
