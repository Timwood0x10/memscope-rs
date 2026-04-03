# HTML 模板迁移到 render_engine 指南

## 概述

本文档详细说明如何将旧的 HTML 模板迁移到新的 `render_engine` 模块中，确保与 Rust 数据结构的正确适配和稳健的渲染。

> **⚠️ 重要提示：网络依赖要求**
> 
> 当前实现的仪表板使用 **CDN 资源** 来实现交互式功能，特别是变量关系图部分。
> 
> - **必需网络连接**: 需要访问 `https://d3js.org/d3.v7.min.js`
> - **影响范围**: 交互式变量关系图
> - **其他功能**: 所有其他功能（KPI 卡片、表格、统计图表等）都可以离线工作
> 
> 如需完全离线使用，请参考文档末尾的 Q5 问答部分。

## 现有模板分析

本文档详细说明如何将旧的 HTML 模板迁移到新的 `render_engine` 模块中，确保与 Rust 数据结构的正确适配和稳健的渲染。

## 现有模板分析

### 模板列表 (templates/ 目录)

| 模板文件 | 用途 | 核心功能 |
|---------|------|---------|
| `async_template.html` | 异步任务分析 | Future 分析、轮询统计、异步任务流 |
| `clean_dashboard.html` | 内存分析仪表板 | 内存分配、变量关系、生命周期可视化 |
| `binary_dashboard.html` | 二进制内存分析 | 3D 内存布局、二进制数据可视化 |
| `multithread_template.html` | 多线程分析 | 线程资源比较、锁竞争分析 |
| `performance_dashboard.html` | 性能仪表板 | 性能指标、扩展性分析 |

### 旧模板的数据注入方式

旧模板使用以下方式注入数据：

```html
<script>
  window.analysisData = {{ json_data }};
  // 或者
  window.analysisData = {{ BINARY_DATA }};
</script>
```

这种方式将整个数据结构作为 JSON 字符串注入到 HTML 中。

## 新渲染引擎架构

### 核心组件

```
src/render_engine/
├── mod.rs              # 模块入口
├── export.rs           # 导出功能
├── dashboard/
│   ├── mod.rs         # Dashboard 模块
│   ├── renderer.rs    # Dashboard 渲染器
│   └── templates/     # 模板文件目录
└── RENDERING_PLAN.md  # 渲染计划文档
```

### 数据结构

#### DashboardContext

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardContext {
    // 基础信息
    pub title: String,
    pub export_timestamp: String,
    
    // 内存统计
    pub total_memory: String,
    pub total_allocations: usize,
    pub active_allocations: usize,
    pub peak_memory: String,
    
    // 高级统计
    pub thread_count: usize,
    pub passport_count: usize,
    pub leak_count: usize,
    pub unsafe_count: usize,
    pub ffi_count: usize,
    
    // 数据列表
    pub allocations: Vec<AllocationInfo>,
    pub relationships: Vec<RelationshipInfo>,
    pub unsafe_reports: Vec<UnsafeReport>,
    
    // 系统信息
    pub system_resources: SystemResources,
    pub threads: Vec<ThreadInfo>,
    
    // JSON 数据（用于前端直接使用）
    pub json_data: String,
}
```

#### 辅助数据结构

```rust
// 分配信息
pub struct AllocationInfo {
    pub address: String,
    pub type_name: String,
    pub size: usize,
    pub var_name: String,
    pub timestamp: String,
    pub thread_id: String,
    pub immutable_borrows: usize,
    pub mutable_borrows: usize,
    pub is_clone: bool,
    pub clone_count: usize,
}

// 变量关系
pub struct RelationshipInfo {
    pub source_ptr: String,
    pub target_ptr: String,
    pub relationship_type: String,
    pub strength: f64,
    pub type_name: String,
}

// Unsafe/FFI 报告
pub struct UnsafeReport {
    pub passport_id: String,
    pub allocation_ptr: String,
    pub size_bytes: usize,
    pub boundary_events: usize,
    pub created_at: u64,
}

// 系统资源
pub struct SystemResources {
    pub os_name: String,
    pub os_version: String,
    pub architecture: String,
    pub cpu_cores: u32,
    pub total_physical: String,
    pub available_physical: String,
    pub used_physical: String,
    pub page_size: u64,
}

// 线程信息
pub struct ThreadInfo {
    pub thread_id: String,
    pub allocation_count: usize,
    pub current_memory: String,
    pub peak_memory: String,
    pub total_allocated: String,
}
```

### 模板引擎：Handlebars

我们选择 **Handlebars** 作为模板引擎，原因如下：

1. **Rust 友好**：原生 Rust 实现，性能优秀
2. **类型安全**：编译时检查模板变量
3. **简单易用**：语法类似 Mustache/Jinja2
4. **稳健性**：错误处理完善，不容易出现运行时错误

### Handlebars 基础语法

```handlebars
<!-- 变量替换 -->
{{title}}

<!-- 条件渲染 -->
{{#if leak_count}}
  <div class="alert">检测到 {{leak_count}} 个内存泄漏</div>
{{/if}}

<!-- 循环渲染 -->
{{#each allocations}}
  <div class="allocation">
    <span>{{var_name}}</span>
    <span>{{size}}</span>
  </div>
{{/each}}

<!-- 访问嵌套属性 -->
{{system_resources.os_name}}

<!-- 格式化输出（需要注册自定义 helper） -->
{{format_bytes total_memory}}
```

## 迁移策略

### 方案 1：渐进式迁移（推荐）

将旧模板逐步迁移到新的模板引擎，保持向后兼容。

#### 步骤 1：创建基础模板结构

```bash
src/render_engine/dashboard/templates/
├── base.html           # 基础模板（CSS、JS）
├── dashboard.html      # 主仪表板
├── async.html          # 异步分析模板
├── multithread.html    # 多线程分析模板
├── performance.html    # 性能分析模板
└── binary.html         # 二进制分析模板
```

#### 步骤 2：提取共享资源

将旧模板中的共享 CSS 和 JavaScript 提取到独立文件：

```bash
src/render_engine/dashboard/assets/
├── styles.css          # 共享样式
├── script.js           # 共享脚本
└── chart-config.js     # Chart.js 配置
```

#### 步骤 3：适配数据结构

将旧模板的数据注入方式改为 Handlebars 语法：

**旧方式：**
```html
<script>
  window.analysisData = {{ json_data }};
</script>
```

**新方式：**
```html
<script>
  window.dashboardData = {
    allocations: {{{json allocations}}},
    relationships: {{{json relationships}}},
    systemResources: {{{json system_resources}}}
  };
</script>
```

#### 步骤 4：注册自定义 Helpers

在 `DashboardRenderer` 中注册自定义 helper：

```rust
impl DashboardRenderer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut handlebars = Handlebars::new();
        
        // 注册模板
        handlebars.register_template_file("dashboard", "templates/dashboard.html")?;
        
        // 注册自定义 helpers
        handlebars.register_helper("format_bytes", Box::new(format_bytes_helper));
        handlebars.register_helper("format_timestamp", Box::new(format_timestamp_helper));
        handlebars.register_helper("risk_level", Box::new(risk_level_helper));
        
        Ok(Self { handlebars })
    }
}

fn format_bytes_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).unwrap().value();
    if let Some(bytes) = param.as_u64() {
        let formatted = format_bytes(bytes as usize);
        out.write(&formatted)?;
    }
    Ok(())
}
```

### 方案 2：统一仪表板 + 条件渲染

创建一个统一的仪表板模板，根据可用数据动态渲染组件。

#### 模板结构

```handlebars
<!-- dashboard.html -->
<!DOCTYPE html>
<html>
<head>
    <title>{{title}}</title>
    <link rel="stylesheet" href="assets/styles.css">
</head>
<body>
    <!-- 通用头部 -->
    <header>
        <h1>{{title}}</h1>
        <p>{{export_timestamp}}</p>
    </header>
    
    <!-- KPI 卡片 -->
    <section class="kpi-cards">
        <div class="kpi-card">
            <span class="value">{{total_memory}}</span>
            <span class="label">总内存</span>
        </div>
        <div class="kpi-card">
            <span class="value">{{total_allocations}}</span>
            <span class="label">分配次数</span>
        </div>
        <!-- 更多 KPI -->
    </section>
    
    <!-- 条件渲染：线程分析 -->
    {{#if (gt thread_count 1)}}
    <section class="thread-analysis">
        <h2>线程分析</h2>
        {{#each threads}}
        <div class="thread-info">
            <span>{{thread_id}}</span>
            <span>{{allocation_count}}</span>
            <span>{{current_memory}}</span>
        </div>
        {{/each}}
    </section>
    {{/if}}
    
    <!-- 条件渲染：内存泄漏检测 -->
    {{#if (gt leak_count 0)}}
    <section class="leak-detection">
        <h2>⚠️ 内存泄漏 ({{leak_count}})</h2>
        <!-- 泄漏详情 -->
    </section>
    {{/if}}
    
    <!-- 条件渲染：Unsafe/FFI -->
    {{#if (gt unsafe_count 0)}}
    <section class="unsafe-ffi">
        <h2>🔒 Unsafe/FFI 操作 ({{unsafe_count}})</h2>
        {{#each unsafe_reports}}
        <div class="unsafe-report">
            <span>{{passport_id}}</span>
            <span>{{size_bytes}}</span>
        </div>
        {{/each}}
    </section>
    {{/if}}
    
    <script src="assets/script.js"></script>
</body>
</html>
```

## 最终采用方案：方案 1.5 - 基础模板 + 特化模板

经过对现有5个模板的分析，我们采用混合方案，既保持模板的独特性，又减少代码重复。

### 方案架构

```
src/render_engine/dashboard/
├── mod.rs                      # 模块入口
├── renderer.rs                 # 渲染器实现
├── templates/
│   ├── base.html              # 基础模板（共享框架）
│   ├── clean_dashboard.html   # 主仪表板（内存+线程+性能）
│   ├── async_dashboard.html   # 异步专用模板
│   └── binary_dashboard.html  # 二进制专用模板
└── assets/
    ├── styles.css             # 共享样式
    ├── script.js              # 共享脚本
    ├── chart-config.js        # Chart.js 配置
    └── d3-config.js           # D3.js 配置
```

### 模板整合策略

| 原模板 | 整合到 | 说明 |
|--------|--------|------|
| clean_dashboard.html | clean_dashboard.html | 主仪表板，保留核心功能 |
| multithread_template.html | clean_dashboard.html | 整合为条件渲染的线程分析模块 |
| performance_dashboard.html | clean_dashboard.html | 整合为条件渲染的性能分析模块 |
| async_template.html | async_dashboard.html | 独立模板，保留独特异步特性 |
| binary_dashboard.html | binary_dashboard.html | 独立模板，保留3D可视化特性 |

## 详细开发计划

### 第一阶段：基础架构搭建（预计1-2小时）

#### 步骤 1.1：创建目录结构

```bash
mkdir -p src/render_engine/dashboard/assets
mkdir -p src/render_engine/dashboard/templates
```

#### 步骤 1.2：创建 base.html

**文件路径**：`src/render_engine/dashboard/templates/base.html`

**完整内容**：

```html
<!DOCTYPE html>
<html lang="en" data-theme="light">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{title}} - MemScope Dashboard</title>
    
    <!-- External Libraries -->
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.8/dist/chart.umd.min.js"></script>
    <script src="https://d3js.org/d3.v7.min.js"></script>
    <script src="https://unpkg.com/three@0.128.0/build/three.min.js"></script>
    <script src="https://unpkg.com/three@0.128.0/examples/js/controls/OrbitControls.js"></script>
    <link href="https://cdn.jsdelivr.net/npm/font-awesome@4.7.0/css/font-awesome.min.css" rel="stylesheet"/>
    
    <!-- Shared Styles -->
    <style>
        /* CSS Variables */
        :root {
            --primary-blue: #2563eb;
            --primary-green: #059669;
            --primary-red: #dc2626;
            --primary-orange: #ea580c;
            --text-primary: #1f2937;
            --text-secondary: #6b7280;
            --bg-primary: #ffffff;
            --bg-secondary: #f8fafc;
            --border-light: #e5e7eb;
            --shadow-light: 0 1px 3px 0 rgb(0 0 0 / 0.1);
        }
        
        .dark {
            --text-primary: #f9fafb;
            --text-secondary: #d1d5db;
            --bg-primary: #111827;
            --bg-secondary: #1f2937;
            --border-light: #374151;
            --shadow-light: 0 4px 6px -1px rgb(0 0 0 / 0.3);
        }
        
        body {
            font-family: 'SF Pro Display', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: var(--bg-secondary);
            color: var(--text-primary);
            transition: all 0.3s ease;
            line-height: 1.6;
            margin: 0;
            padding: 0;
        }
        
        .dashboard-container {
            max-width: 1400px;
            margin: 0 auto;
            padding: 24px;
            min-height: 100vh;
        }
        
        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 32px;
            padding: 20px 0;
            border-bottom: 1px solid var(--border-light);
        }
        
        .header h1 {
            font-size: 2rem;
            font-weight: 700;
            color: var(--text-primary);
            margin: 0;
        }
        
        .header .subtitle {
            color: var(--text-secondary);
            font-size: 0.9rem;
            margin-top: 4px;
        }
        
        .theme-toggle {
            background: var(--primary-blue);
            color: white;
            border: none;
            padding: 10px 16px;
            border-radius: 8px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 500;
            transition: all 0.2s ease;
            display: flex;
            align-items: center;
            gap: 8px;
        }
        
        .theme-toggle:hover {
            background: #1d4ed8;
            transform: translateY(-1px);
        }
        
        .card {
            background: var(--bg-primary);
            border: 1px solid var(--border-light);
            border-radius: 12px;
            padding: 24px;
            box-shadow: var(--shadow-light);
            transition: all 0.3s ease;
        }
        
        .card:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 25px -5px rgb(0 0 0 / 0.1);
        }
        
        .card h2 {
            font-size: 1.25rem;
            font-weight: 600;
            color: var(--text-primary);
            margin: 0 0 16px 0;
            border-bottom: 2px solid var(--primary-blue);
            padding-bottom: 8px;
        }
        
        .grid {
            display: grid;
            gap: 24px;
            margin-bottom: 32px;
        }
        
        .grid-2 { grid-template-columns: 1fr 1fr; }
        .grid-3 { grid-template-columns: repeat(3, 1fr); }
        .grid-4 { grid-template-columns: repeat(4, 1fr); }
        
        table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 16px;
        }
        
        th, td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid var(--border-light);
        }
        
        th {
            background: var(--bg-secondary);
            font-weight: 600;
            color: var(--text-primary);
        }
        
        tr:hover {
            background: var(--bg-secondary);
        }
        
        .kpi-card {
            text-align: center;
            padding: 20px;
            background: linear-gradient(135deg, var(--primary-blue) 0%, #3b82f6 100%);
            color: white;
            border-radius: 12px;
            border: none;
            box-shadow: var(--shadow-light);
        }
        
        .kpi-value {
            font-size: 2rem;
            font-weight: 700;
            margin-bottom: 4px;
        }
        
        .kpi-label {
            font-size: 0.875rem;
            opacity: 0.9;
            font-weight: 500;
        }
        
        .status-badge {
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 0.75rem;
            font-weight: 500;
        }
        
        .status-active {
            background: #dcfce7;
            color: #166534;
        }
        
        .status-leaked {
            background: #fee2e2;
            color: #dc2626;
        }
        
        .dark .status-active {
            background: #064e3b;
            color: #34d399;
        }
        
        .dark .status-leaked {
            background: #7f1d1d;
            color: #fca5a5;
        }
    </style>
    
    {{#> page-styles}}{{/page-styles}}
</head>
<body>
    <div class="dashboard-container">
        <!-- Header -->
        <header class="header">
            <div>
                <h1>{{title}}</h1>
                <p class="subtitle">{{export_timestamp}}</p>
            </div>
            <button class="theme-toggle" onclick="toggleTheme()">
                <i class="fa fa-moon-o"></i>
                <span>Dark Mode</span>
            </button>
        </header>
        
        <!-- Main Content -->
        {{> content}}
        
        <!-- Footer -->
        <footer style="text-align: center; color: var(--text-secondary); margin-top: 40px; padding: 20px; border-top: 1px solid var(--border-light);">
            <p>Generated by MemScope-rs | {{os_name}} {{architecture}} | {{cpu_cores}} Cores</p>
            <p style="font-size: 0.8rem; margin-top: 8px;">Export Time: {{export_timestamp}}</p>
        </footer>
    </div>
    
    <!-- Shared Scripts -->
    <script>
        function toggleTheme() {
            const html = document.documentElement;
            const current = html.getAttribute('data-theme');
            const next = current === 'light' ? 'dark' : 'light';
            html.setAttribute('data-theme', next);
            localStorage.setItem('theme', next);
        }
        
        // Load saved theme
        const savedTheme = localStorage.getItem('theme') || 'light';
        document.documentElement.setAttribute('data-theme', savedTheme);
        
        function formatBytes(bytes) {
            if (bytes === 0) return '0 B';
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }
        
        // Data injection
        window.dashboardData = {
            allocations: {{{json allocations}}},
            relationships: {{{json relationships}}},
            unsafeReports: {{{json unsafe_reports}}},
            systemResources: {{{json system_resources}}},
            threads: {{{json threads}}},
            metadata: {
                totalMemory: "{{total_memory}}",
                totalAllocations: {{total_allocations}},
                activeAllocations: {{active_allocations}},
                peakMemory: "{{peak_memory}}",
                threadCount: {{thread_count}},
                leakCount: {{leak_count}},
                unsafeCount: {{unsafe_count}},
                passportCount: {{passport_count}}
            }
        };
    </script>
    
    {{#> page-scripts}}{{/page-scripts}}
</body>
</html>
```

#### 步骤 1.3：创建共享样式文件

**文件路径**：`src/render_engine/dashboard/assets/styles.css`

**完整内容**：

```css
/* Additional shared styles that can be loaded separately */
.scroll {
    max-height: 400px;
    overflow: auto;
}

.scroll::-webkit-scrollbar {
    width: 6px;
}

.scroll::-webkit-scrollbar-track {
    background: var(--border-light);
}

.scroll::-webkit-scrollbar-thumb {
    background: var(--primary-blue);
    border-radius: 3px;
}

.chart-container {
    height: 300px;
    background: var(--bg-primary);
    border-radius: 8px;
    position: relative;
    padding: 16px;
}

/* Alert styles */
.alert {
    padding: 16px;
    border-radius: 8px;
    margin-bottom: 16px;
}

.alert-warning {
    background: #fef3c7;
    border: 1px solid #f59e0b;
    color: #92400e;
}

.alert-error {
    background: #fee2e2;
    border: 1px solid #dc2626;
    color: #dc2626;
}

/* Responsive */
@media (max-width: 768px) {
    .grid-2, .grid-3, .grid-4 {
        grid-template-columns: 1fr;
    }
    
    .dashboard-container {
        padding: 16px;
    }
    
    .header {
        flex-direction: column;
        gap: 16px;
        text-align: center;
    }
}
```

#### 步骤 1.4：创建共享脚本文件

**文件路径**：`src/render_engine/dashboard/assets/script.js`

**完整内容**：

```javascript
// Shared utility functions
window.MemScopeUtils = {
    formatBytes: function(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    },
    
    formatTimestamp: function(timestamp) {
        const date = new Date(timestamp);
        return date.toLocaleString();
    },
    
    createTooltip: function(element, content) {
        const tooltip = document.createElement('div');
        tooltip.className = 'tooltip';
        tooltip.textContent = content;
        tooltip.style.cssText = `
            position: absolute;
            background: rgba(0, 0, 0, 0.8);
            color: white;
            padding: 8px 12px;
            border-radius: 4px;
            font-size: 12px;
            pointer-events: none;
            z-index: 1000;
        `;
        document.body.appendChild(tooltip);
        
        element.addEventListener('mouseenter', (e) => {
            tooltip.style.display = 'block';
            tooltip.style.left = e.pageX + 10 + 'px';
            tooltip.style.top = e.pageY + 10 + 'px';
        });
        
        element.addEventListener('mousemove', (e) => {
            tooltip.style.left = e.pageX + 10 + 'px';
            tooltip.style.top = e.pageY + 10 + 'px';
        });
        
        element.addEventListener('mouseleave', () => {
            tooltip.style.display = 'none';
        });
    }
};

// Chart.js default configuration
Chart.defaults.font.family = "'SF Pro Display', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";
Chart.defaults.color = '#6b7280';
Chart.defaults.borderColor = '#e5e7eb';

console.log('MemScope Dashboard utilities loaded');
```

### 第二阶段：创建 clean_dashboard.html（预计2-3小时）

#### 步骤 2.1：创建主仪表板模板

**文件路径**：`src/render_engine/dashboard/templates/clean_dashboard.html`

**完整内容**：

```html
{{#*inline "page-styles"}}
<style>
    /* Clean Dashboard Specific Styles */
    .kpi-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 16px;
        margin-bottom: 32px;
    }
    
    .kpi-card-custom {
        background: linear-gradient(135deg, var(--primary-blue), #3b82f6);
        color: white;
        padding: 20px;
        border-radius: 12px;
        text-align: center;
    }
    
    .kpi-card-warning {
        background: linear-gradient(135deg, #f59e0b, #d97706);
    }
    
    .kpi-card-danger {
        background: linear-gradient(135deg, #dc2626, #b91c1c);
    }
    
    .allocation-type {
        padding: 3px 6px;
        border-radius: 3px;
        font-size: 0.7rem;
        font-weight: 600;
        text-transform: uppercase;
        display: inline-block;
    }
    
    .type-heap {
        background: #fef3c7;
        color: #92400e;
        border: 1px solid #f59e0b;
    }
    
    .type-smart {
        background: #dbeafe;
        color: #1e40af;
        border: 1px solid #3b82f6;
    }
</style>
{{/inline}}

{{#*inline "page-scripts"}}
<script>
// Clean Dashboard Specific Scripts
document.addEventListener('DOMContentLoaded', function() {
    console.log('Dashboard data:', window.dashboardData);
    
    // Initialize charts
    initMemoryChart();
    initAllocationChart();
    initThreadChart();
    
    // Initialize D3.js visualizations
    if (window.dashboardData.relationships.length > 0) {
        initRelationshipGraph();
    }
});

function initMemoryChart() {
    const ctx = document.getElementById('memoryChart');
    if (!ctx) return;
    
    const data = window.dashboardData.allocations;
    const typeDistribution = {};
    
    data.forEach(alloc => {
        const type = alloc.type_name || 'unknown';
        typeDistribution[type] = (typeDistribution[type] || 0) + alloc.size;
    });
    
    new Chart(ctx, {
        type: 'doughnut',
        data: {
            labels: Object.keys(typeDistribution),
            datasets: [{
                data: Object.values(typeDistribution),
                backgroundColor: [
                    '#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6', '#ec4899'
                ]
            }]
        },
        options: {
            responsive: true,
            plugins: {
                legend: { position: 'bottom' }
            }
        }
    });
}

function initAllocationChart() {
    const ctx = document.getElementById('allocationChart');
    if (!ctx) return;
    
    const data = window.dashboardData.allocations;
    const sizeDistribution = {};
    
    data.forEach(alloc => {
        const sizeRange = getSizeRange(alloc.size);
        sizeDistribution[sizeRange] = (sizeDistribution[sizeRange] || 0) + 1;
    });
    
    new Chart(ctx, {
        type: 'bar',
        data: {
            labels: Object.keys(sizeDistribution),
            datasets: [{
                label: 'Allocation Count',
                data: Object.values(sizeDistribution),
                backgroundColor: '#3b82f6'
            }]
        },
        options: {
            responsive: true,
            plugins: {
                legend: { display: false }
            }
        }
    });
}

function initThreadChart() {
    const ctx = document.getElementById('threadChart');
    if (!ctx) return;
    
    const threads = window.dashboardData.threads;
    
    new Chart(ctx, {
        type: 'bar',
        data: {
            labels: threads.map(t => t.thread_id),
            datasets: [{
                label: 'Memory (MB)',
                data: threads.map(t => parseFloat(t.current_memory)),
                backgroundColor: '#10b981'
            }]
        },
        options: {
            responsive: true,
            plugins: {
                legend: { display: false }
            }
        }
    });
}

function initRelationshipGraph() {
    const container = document.getElementById('relationshipGraph');
    if (!container) return;
    
    const width = container.clientWidth;
    const height = 400;
    
    const svg = d3.select('#relationshipGraph')
        .append('svg')
        .attr('width', width)
        .attr('height', height);
    
    const simulation = d3.forceSimulation()
        .force('link', d3.forceLink().id(d => d.id).distance(100))
        .force('charge', d3.forceManyBody().strength(-300))
        .force('center', d3.forceCenter(width / 2, height / 2));
    
    const relationships = window.dashboardData.relationships;
    const nodes = new Set();
    const links = [];
    
    relationships.forEach(rel => {
        nodes.add({ id: rel.source_ptr, type: rel.type_name });
        nodes.add({ id: rel.target_ptr, type: rel.type_name });
        links.push({ source: rel.source_ptr, target: rel.target_ptr, strength: rel.strength });
    });
    
    const nodeArray = Array.from(nodes);
    
    const link = svg.append('g')
        .selectAll('line')
        .data(links)
        .join('line')
        .attr('stroke', '#999')
        .attr('stroke-opacity', 0.6)
        .attr('stroke-width', d => d.strength * 3);
    
    const node = svg.append('g')
        .selectAll('circle')
        .data(nodeArray)
        .join('circle')
        .attr('r', 8)
        .attr('fill', '#3b82f6')
        .call(drag(simulation));
    
    node.append('title')
        .text(d => d.id);
    
    simulation.on('tick', () => {
        link
            .attr('x1', d => d.source.x)
            .attr('y1', d => d.source.y)
            .attr('x2', d => d.target.x)
            .attr('y2', d => d.target.y);
        
        node
            .attr('cx', d => d.x)
            .attr('cy', d => d.y);
    });
    
    function drag(simulation) {
        function dragstarted(event) {
            if (!event.active) simulation.alphaTarget(0.3).restart();
            event.subject.fx = event.subject.x;
            event.subject.fy = event.subject.y;
        }
        
        function dragged(event) {
            event.subject.fx = event.x;
            event.subject.fy = event.y;
        }
        
        function dragended(event) {
            if (!event.active) simulation.alphaTarget(0);
            event.subject.fx = null;
            event.subject.fy = null;
        }
        
        return d3.drag()
            .on('start', dragstarted)
            .on('drag', dragged)
            .on('end', dragended);
    }
}

function getSizeRange(bytes) {
    if (bytes < 1024) return '< 1KB';
    if (bytes < 10240) return '1KB-10KB';
    if (bytes < 102400) return '10KB-100KB';
    if (bytes < 1048576) return '100KB-1MB';
    return '> 1MB';
}
</script>
{{/inline}}

{{#*inline "content"}}
<!-- KPI Cards -->
<div class="kpi-grid">
    <div class="kpi-card kpi-card-custom">
        <div class="kpi-value">{{total_memory}}</div>
        <div class="kpi-label">总内存</div>
    </div>
    <div class="kpi-card kpi-card-custom">
        <div class="kpi-value">{{total_allocations}}</div>
        <div class="kpi-label">分配次数</div>
    </div>
    <div class="kpi-card kpi-card-custom">
        <div class="kpi-value">{{active_allocations}}</div>
        <div class="kpi-label">活跃分配</div>
    </div>
    <div class="kpi-card kpi-card-custom">
        <div class="kpi-value">{{peak_memory}}</div>
        <div class="kpi-label">峰值内存</div>
    </div>
    {{#if (gt leak_count 0)}}
    <div class="kpi-card kpi-card-danger">
        <div class="kpi-value">{{leak_count}}</div>
        <div class="kpi-label">内存泄漏</div>
    </div>
    {{/if}}
    {{#if (gt unsafe_count 0)}}
    <div class="kpi-card kpi-card-warning">
        <div class="kpi-value">{{unsafe_count}}</div>
        <div class="kpi-label">Unsafe操作</div>
    </div>
    {{/if}}
</div>

<!-- System Information -->
<div class="card" style="margin-bottom: 24px;">
    <h2>💻 系统信息</h2>
    <div class="grid grid-4">
        <div>
            <strong>操作系统:</strong> {{system_resources.os_name}}
        </div>
        <div>
            <strong>架构:</strong> {{system_resources.architecture}}
        </div>
        <div>
            <strong>CPU核心:</strong> {{system_resources.cpu_cores}}
        </div>
        <div>
            <strong>总内存:</strong> {{system_resources.total_physical}}
        </div>
    </div>
</div>

<!-- Thread Analysis (Conditional) -->
{{#if (gt thread_count 1)}}
<div class="card" style="margin-bottom: 24px;">
    <h2>🧵 线程分析 ({{thread_count}} 线程)</h2>
    <div style="height: 300px;">
        <canvas id="threadChart"></canvas>
    </div>
    <table class="scroll">
        <thead>
            <tr>
                <th>线程ID</th>
                <th>分配次数</th>
                <th>当前内存</th>
                <th>峰值内存</th>
                <th>总分配</th>
            </tr>
        </thead>
        <tbody>
            {{#each threads}}
            <tr>
                <td>{{thread_id}}</td>
                <td>{{allocation_count}}</td>
                <td>{{current_memory}}</td>
                <td>{{peak_memory}}</td>
                <td>{{total_allocated}}</td>
            </tr>
            {{/each}}
        </tbody>
    </table>
</div>
{{/if}}

<!-- Memory Analysis -->
<div class="grid grid-2" style="margin-bottom: 24px;">
    <div class="card">
        <h2>📊 内存类型分布</h2>
        <div style="height: 300px;">
            <canvas id="memoryChart"></canvas>
        </div>
    </div>
    <div class="card">
        <h2>📈 分配大小分布</h2>
        <div style="height: 300px;">
            <canvas id="allocationChart"></canvas>
        </div>
    </div>
</div>

<!-- Relationship Graph (Conditional) -->
{{#if (gt relationships_count 0)}}
<div class="card" style="margin-bottom: 24px;">
    <h2>🔗 变量关系图 ({{relationships_count}} 关系)</h2>
    <div id="relationshipGraph" style="height: 400px;"></div>
</div>
{{/if}}

<!-- Allocation Table -->
<div class="card" style="margin-bottom: 24px;">
    <h2>📋 内存分配详情 ({{allocations_count}} 项)</h2>
    <table class="scroll">
        <thead>
            <tr>
                <th>地址</th>
                <th>变量名</th>
                <th>类型</th>
                <th>大小</th>
                <th>线程</th>
                <th>时间戳</th>
            </tr>
        </thead>
        <tbody>
            {{#each allocations}}
            <tr>
                <td><code>{{address}}</code></td>
                <td>{{var_name}}</td>
                <td>
                    {{#if (contains type_name "Arc")}}
                    <span class="allocation-type type-smart">Arc</span>
                    {{else if (contains type_name "Rc")}}
                    <span class="allocation-type type-smart">Rc</span>
                    {{else if (contains type_name "Box")}}
                    <span class="allocation-type type-smart">Box</span>
                    {{else}}
                    <span class="allocation-type type-heap">Heap</span>
                    {{/if}}
                    {{type_name}}
                </td>
                <td>{{size}} bytes</td>
                <td>{{thread_id}}</td>
                <td>{{timestamp}}</td>
            </tr>
            {{/each}}
        </tbody>
    </table>
</div>

<!-- Unsafe/FFI Reports (Conditional) -->
{{#if (gt unsafe_reports_count 0)}}
<div class="card" style="margin-bottom: 24px;">
    <h2>🔒 Unsafe/FFI 操作 ({{unsafe_reports_count}} 项)</h2>
    <table class="scroll">
        <thead>
            <tr>
                <th>Passport ID</th>
                <th>分配指针</th>
                <th>大小</th>
                <th>边界事件</th>
                <th>创建时间</th>
            </tr>
        </thead>
        <tbody>
            {{#each unsafe_reports}}
            <tr>
                <td><code>{{passport_id}}</code></td>
                <td><code>{{allocation_ptr}}</code></td>
                <td>{{size_bytes}} bytes</td>
                <td>{{boundary_events}}</td>
                <td>{{created_at}}</td>
            </tr>
            {{/each}}
        </tbody>
    </table>
</div>
{{/if}}
{{/inline}}
```

#### 步骤 2.2：更新 renderer.rs 注册模板

**修改文件**：`src/render_engine/dashboard/renderer.rs`

**修改位置**：在 `DashboardRenderer::new()` 方法中

**修改内容**：

```rust
impl DashboardRenderer {
    /// Create a new dashboard renderer
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut handlebars = Handlebars::new();
        
        // Register base template
        handlebars.register_template_file("base", "src/render_engine/dashboard/templates/base.html")?;
        
        // Register dashboard template
        handlebars.register_template_file("clean_dashboard", "src/render_engine/dashboard/templates/clean_dashboard.html")?;
        
        // Register custom helpers
        handlebars.register_helper("format_bytes", Box::new(format_bytes_helper));
        handlebars.register_helper("gt", Box::new(greater_than_helper));
        handlebars.register_helper("contains", Box::new(contains_helper));
        
        Ok(Self { handlebars })
    }
}
```

**添加 Helper 函数**：

```rust
// Add at the end of renderer.rs

fn format_bytes_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).unwrap().value();
    if let Some(bytes) = param.as_u64() {
        let formatted = format_bytes(bytes as usize);
        out.write(&formatted)?;
    }
    Ok(())
}

fn greater_than_helper(
    h: &Helper,
    _: &Handlebars,
    ctx: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param1 = h.param(0).unwrap().value();
    let param2 = h.param(1).unwrap().value();
    
    if let (Some(v1), Some(v2)) = (param1.as_u64(), param2.as_u64()) {
        if v1 > v2 {
            out.write("true")?;
        }
    }
    Ok(())
}

fn contains_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let haystack = h.param(0).unwrap().value();
    let needle = h.param(1).unwrap().value();
    
    if let (Some(h_str), Some(n_str)) = (haystack.as_str(), needle.as_str()) {
        if h_str.contains(n_str) {
            out.write("true")?;
        }
    }
    Ok(())
}
```

### 第三阶段：扩展 DashboardContext 数据结构

#### 步骤 3.1：添加性能指标字段

**修改文件**：`src/render_engine/dashboard/renderer.rs`

**在 `DashboardContext` 结构体中添加**：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardContext {
    // ... 现有字段 ...
    
    // 性能指标（新增）
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub peak_performance: f64,
    pub efficiency_rating: f64,
    pub scalability_score: f64,
    pub memory_efficiency: f64,
    pub processing_time_ms: f64,
}
```

#### 步骤 3.2：在 render_from_tracker 中计算性能指标

**修改文件**：`src/render_engine/dashboard/renderer.rs`

**在 `render_from_tracker` 方法中添加**：

```rust
// 在构建 context 之前添加
let performance_metrics = PerformanceMetrics {
    peak_performance: calculate_peak_performance(&alloc_info),
    efficiency_rating: calculate_efficiency_rating(&alloc_info),
    scalability_score: calculate_scalability_score(&alloc_info),
    memory_efficiency: calculate_memory_efficiency(&alloc_info),
    processing_time_ms: 0.0, // 可以从传入参数获取
};

// 在 context 初始化中添加
let context = DashboardContext {
    // ... 现有字段 ...
    performance_metrics,
    // ...
};
```

**添加计算函数**：

```rust
fn calculate_peak_performance(allocations: &[AllocationInfo]) -> f64 {
    if allocations.is_empty() {
        return 0.0;
    }
    
    // 简单计算：总分配次数 / 假设的执行时间
    // 实际应用中需要更精确的计算
    allocations.len() as f64 * 1000.0 // 假设每秒1000次分配
}

fn calculate_efficiency_rating(allocations: &[AllocationInfo]) -> f64 {
    if allocations.is_empty() {
        return 0.0;
    }
    
    // 计算平均分配效率
    let total_size: usize = allocations.iter().map(|a| a.size).sum();
    let avg_size = total_size / allocations.len();
    
    // 假设理想平均大小为 1024 字节
    let ideal_size = 1024.0;
    let actual_size = avg_size as f64;
    
    // 效率评分：接近理想大小的得分更高
    let ratio = actual_size / ideal_size;
    if ratio > 1.0 {
        100.0 / ratio
    } else {
        100.0 * ratio
    }
}

fn calculate_scalability_score(allocations: &[AllocationInfo]) -> f64 {
    if allocations.is_empty() {
        return 0.0;
    }
    
    // 基于线程数量计算扩展性
    let thread_count = allocations.iter()
        .map(|a| a.thread_id.clone())
        .collect::<std::collections::HashSet<_>>()
        .len();
    
    if thread_count <= 1 {
        return 100.0;
    }
    
    // 简单的线性扩展性计算
    // 实际应用中需要更复杂的分析
    let ideal_scaling = thread_count as f64;
    let actual_scaling = allocations.len() as f64 / thread_count as f64;
    
    (actual_scaling / ideal_scaling) * 100.0
}

fn calculate_memory_efficiency(allocations: &[AllocationInfo]) -> f64 {
    if allocations.is_empty() {
        return 0.0;
    }
    
    let total_size: usize = allocations.iter().map(|a| a.size).sum();
    let actual_size = total_size as f64;
    
    // 假设有20%的内存开销是正常的
    let overhead = actual_size * 0.2;
    let efficient_size = actual_size - overhead;
    
    (efficient_size / actual_size) * 100.0
}
```

### 第四阶段：测试和验证（预计1小时）

#### 步骤 4.1：创建测试示例

**修改文件**：`examples/dashboard_export.rs`

**确保示例代码正确**：

```rust
use memscope_rs::analysis::memory_passport_tracker::{PassportTrackerConfig, MemoryPassportTracker};
use memscope_rs::render_engine::export::export_dashboard_html;
use memscope_rs::{track, tracker};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Dashboard HTML Export Example");
    
    // Create tracker
    let tracker = tracker!();
    
    // Track allocations
    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);
    
    let string_data = String::from("Hello, MemScope!");
    track!(tracker, string_data);
    
    let arc_data = std::sync::Arc::new(vec![1.0, 2.0, 3.0]);
    track!(tracker, arc_data.clone());
    
    // Create passport tracker
    let passport_tracker = Arc::new(
        MemoryPassportTracker::new(PassportTrackerConfig::default())
    );
    
    // Export dashboard
    println!("📊 Exporting dashboard...");
    export_dashboard_html("./MemoryAnalysis/dashboard_export", &tracker, &passport_tracker)?;
    
    println!("✅ Export successful!");
    println!("📁 Open ./MemoryAnalysis/dashboard_export/dashboard.html in your browser");
    
    Ok(())
}
```

#### 步骤 4.2：运行测试

```bash
cargo run --example dashboard_export
```

#### 步骤 4.3：验证输出

检查生成的 HTML 文件：
- [ ] 文件存在
- [ ] 包含所有 KPI 卡片
- [ ] 图表正常显示
- [ ] 表格数据正确
- [ ] 样式正常加载

### 第五阶段：后续模板迁移（预留）

#### 步骤 5.1：async_dashboard.html
- 保留异步任务流可视化
- 保留 Future 分析图表
- 保留轮询统计

#### 步骤 5.2：binary_dashboard.html
- 保留 3D 内存布局
- 保留二进制数据可视化
- 保留 Three.js 交互

## 总结

本开发计划提供了：

1. ✅ **完整的文件结构**：精确到每个文件的路径
2. ✅ **详细的代码内容**：每个文件的完整代码
3. ✅ **数据字段映射**：新旧数据结构的对应关系
4. ✅ **分阶段实施**：从基础到高级的渐进式开发
5. ✅ **可测试的步骤**：每个阶段都可以独立测试

现在开始执行第一阶段！

### 1. 迁移 clean_dashboard.html

**目标**：保留原有的内存分析、变量关系、生命周期可视化功能

**关键点**：
- 将内存分配表格改为 `{{#each allocations}}` 循环
- 将 D3.js 关系图数据源改为 `window.dashboardData.relationships`
- 保留 Chart.js 图表，更新数据源

**模板片段示例**：

```handlebars
<!-- 内存分配表格 -->
<table>
    <thead>
        <tr>
            <th>地址</th>
            <th>变量名</th>
            <th>类型</th>
            <th>大小</th>
            <th>线程</th>
        </tr>
    </thead>
    <tbody>
        {{#each allocations}}
        <tr>
            <td>{{address}}</td>
            <td>{{var_name}}</td>
            <td>{{type_name}}</td>
            <td>{{size}} bytes</td>
            <td>{{thread_id}}</td>
        </tr>
        {{/each}}
    </tbody>
</table>
```

### 2. 迁移 async_template.html

**目标**：保留 Future 分析、轮询统计功能

**挑战**：
- 新数据结构中没有直接对应 Future 分析的字段
- 需要从 `allocations` 中推断异步任务信息

**解决方案**：

```rust
// 在 DashboardRenderer 中添加异步相关数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncTaskInfo {
    pub task_id: String,
    pub future_count: usize,
    pub poll_count: usize,
    pub avg_poll_time: f64,
    pub tasks: Vec<TaskDetail>,
}

// 在 render_from_tracker 中构建异步任务信息
async_tasks = build_async_task_info(&allocations);
```

### 3. 迁移 multithread_template.html

**目标**：保留线程资源比较功能

**映射关系**：

| 旧模板字段 | 新数据结构 |
|-----------|-----------|
| 线程统计 | `threads: Vec<ThreadInfo>` |
| 锁竞争 | 需要新增字段 |
| 死锁检测 | 需要新增字段 |

### 4. 迁移 performance_dashboard.html

**目标**：保留性能指标、扩展性分析

**映射关系**：

| 旧模板字段 | 新数据结构 |
|-----------|-----------|
| PEAK_PERFORMANCE | 需要计算 |
| EFFICIENCY_RATING | 需要计算 |
| SCALABILITY_SCORE | 需要计算 |
| PERFORMANCE_DATA | 需要聚合 |

## 数据适配层

创建一个数据适配层，将 `Tracker` 数据转换为模板所需格式：

```rust
impl DashboardRenderer {
    fn build_adapted_data(
        &self,
        tracker: &Tracker,
        passport_tracker: &Arc<MemoryPassportTracker>,
    ) -> AdaptedDashboardData {
        // 基础数据
        let allocations = self.build_allocation_info(tracker);
        
        // 扩展数据 - 异步任务
        let async_tasks = self.infer_async_tasks(&allocations);
        
        // 扩展数据 - 性能指标
        let performance = self.calculate_performance_metrics(&allocations);
        
        // 扩展数据 - 线程比较
        let thread_comparison = self.build_thread_comparison(tracker);
        
        AdaptedDashboardData {
            basic: DashboardContext { /* ... */ },
            async_tasks,
            performance,
            thread_comparison,
        }
    }
}
```

## 渲染流程

```
1. 收集数据
   ├─ Tracker::get_active_allocations()
   ├─ MemoryPassportTracker::get_all_passports()
   └─ PlatformMemoryInfo::collect_stats()

2. 构建上下文
   ├─ build_allocation_info()
   ├─ build_relationship_info()
   ├─ build_unsafe_reports()
   └─ build_system_resources()

3. 注册模板
   ├─ handlebars.register_template_file()
   └─ handlebars.register_helper()

4. 渲染 HTML
   └─ handlebars.render("dashboard", &context)

5. 写入文件
   └─ std::fs::write(output_path, html_content)
```

## 错误处理

### 模板渲染错误

```rust
pub fn render_dashboard(&self, context: &DashboardContext) -> Result<String, RenderError> {
    self.handlebars
        .render("dashboard", context)
        .map_err(|e| {
            RenderError::TemplateError {
                template: "dashboard".to_string(),
                message: e.to_string(),
            }
        })
}
```

### 数据验证

在渲染前验证数据完整性：

```rust
fn validate_context(context: &DashboardContext) -> Result<(), RenderError> {
    if context.allocations.is_empty() {
        return Err(RenderError::NoData("没有可用的分配数据".to_string()));
    }
    Ok(())
}
```

## 性能优化

### 1. 模板预编译

```rust
// 在应用启动时预编译模板
lazy_static! {
    static ref HANDLEBARS: Handlebars<'static> = {
        let mut hb = Handlebars::new();
        hb.register_template_file("dashboard", "templates/dashboard.html").unwrap();
        hb
    };
}
```

### 2. 数据预序列化

```rust
// 预序列化 JSON 数据，避免重复计算
let json_data = serde_json::to_string(&data)?;
let context = DashboardContext {
    json_data,
    // ...
};
```

### 3. 条件渲染

只在需要时渲染复杂组件：

```handlebars
{{#if (gt allocations_count 100)}}
<div class="pagination">
    <!-- 分页控件 -->
</div>
{{/if}}
```

## 测试策略

### 1. 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic_dashboard() {
        let renderer = DashboardRenderer::new().unwrap();
        let context = create_test_context();
        let html = renderer.render_dashboard(&context).unwrap();
        assert!(html.contains("MemScope Dashboard"));
    }
}
```

### 2. 集成测试

```rust
#[test]
fn test_export_dashboard_html() {
    let tracker = create_test_tracker();
    let passport_tracker = create_test_passport_tracker();
    
    export_dashboard_html("./test_output", &tracker, &passport_tracker).unwrap();
    
    assert!(Path::new("./test_output/dashboard.html").exists());
}
```

### 3. 视觉回归测试

使用工具对比新旧模板的渲染结果。

## 迁移检查清单

### 准备阶段
- [ ] 备份现有模板文件
- [ ] 确认所有依赖项（Handlebars、serde_json 等）
- [ ] 创建新的模板目录结构

### 迁移阶段
- [ ] 提取共享 CSS 和 JS
- [ ] 创建基础模板 (base.html)
- [ ] 迁移 clean_dashboard.html
- [ ] 迁移 async_template.html
- [ ] 迁移 multithread_template.html
- [ ] 迁移 performance_dashboard.html
- [ ] 迁移 binary_dashboard.html

### 测试阶段
- [ ] 单元测试每个模板
- [ ] 集成测试完整流程
- [ ] 视觉回归测试
- [ ] 性能基准测试

### 部署阶段
- [ ] 更新文档
- [ ] 迁移示例代码
- [ ] 更新 CHANGELOG
- [ ] 发布新版本

## 常见问题

### Q1: 如何处理模板中的复杂 JavaScript 逻辑？

**A**: 将复杂逻辑移到独立的 JS 文件中，通过数据注入进行交互：

```html
<script src="assets/visualizer.js"></script>
<script>
  window.initDashboard({{{json_data}}});
</script>
```

### Q2: 如何支持自定义模板？

**A**: 提供模板注册 API：

```rust
let mut renderer = DashboardRenderer::new()?;
renderer.register_custom_template("custom", "path/to/custom.html")?;
```

### Q3: 如何处理大型数据集？

**A**: 使用分页和懒加载：

```rust
let paginated_allocations = &context.allocations[page * page_size..(page + 1) * page_size];
```

### Q4: 如何支持多语言？

**A**: 使用 Handlebars 的国际化支持或构建自己的 i18n 系统。

## 总结

通过本迁移指南，你应该能够：

1. ✅ 理解新旧模板系统的差异
2. ✅ 选择合适的迁移策略
3. ✅ 逐步迁移所有现有模板
4. ✅ 保持数据一致性和渲染稳定性
5. ✅ 优化性能和用户体验

记住：**渐进式迁移**是最稳妥的方式，先迁移核心功能，再逐步添加高级特性。

## 参考资料

- [Handlebars Rust 文档](https://docs.rs/handlebars/)
- [Serde 序列化文档](https://serde.rs/)
- [RENDERING_PLAN.md](./src/render_engine/dashboard/RENDERING_PLAN.md)
- [现有模板示例](./templates/)
### Q5: 仪表板是否需要网络连接？

**A**: 是的，当前实现的交互式关系图需要网络连接。仪表板使用了以下外部 CDN 资源：

```html
<!-- D3.js - 用于交互式关系图 -->
<script src="https://d3js.org/d3.v7.min.js"></script>
```

**网络依赖说明：**
- **必需资源**: D3.js v7（用于交互式变量关系图）
- **CDN 地址**: https://d3js.org/d3.v7.min.js
- **用途**: 实现力导向图、节点拖拽、缩放等交互功能
- **大小**: 约 200KB

**离线环境处理：**
如果需要在完全离线的环境中使用仪表板，可以考虑以下方案：

1. **下载 D3.js 到本地**:
   ```bash
   curl -o d3.v7.min.js https://d3js.org/d3.v7.min.js
   ```
   然后修改模板中的 CDN 引用：
   ```html
   <script src="./d3.v7.min.js"></script>
   ```

2. **使用表格展示**: 移除关系图部分，使用表格展示变量关系数据

3. **实现纯 SVG 版本**: 使用原生 JavaScript 和 SVG 实现简化版的关系图（不依赖外部库）

**最佳实践：**
- 在生产环境中，建议将 CDN 资源下载到本地或使用公司内部 CDN
- 在文档中明确说明网络依赖要求
- 提供离线版本作为备选方案
