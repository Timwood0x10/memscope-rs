# SVG嵌入HTML交互式仪表板开发任务

## 🎯 项目目标
将现有的高质量SVG图表（lifecycleTimeline.svg、memoryAnalysis.svg、unsafe_ffi_dashboard.svg）整合到一个交互式HTML仪表板中，提供专业的内存分析可视化体验。

## 📊 现状分析
### 已有资源
- ✅ 高质量的SVG图表生成代码
- ✅ 丰富的JSON数据结构（包含timeline、stack_traces等）
- ✅ 专业的视觉设计（渐变、阴影、交互效果）
- ✅ 完整的数据追踪系统

### 需要改进
- ❌ SVG图表分散在多个文件中
- ❌ 缺乏统一的查看界面
- ❌ 交互功能有限
- ❌ 无法进行数据关联分析

## 🚀 实施步骤

### Phase 1: 基础整合 (预计2-3天)

#### 1.1 创建HTML模板
```html
<!-- src/dashboard_template.html -->
<!DOCTYPE html>
<html>
<head>
    <title>Memory Analysis Dashboard</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        /* 现代化样式设计 */
        body { 
            margin: 0; 
            padding: 20px; 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            min-height: 100vh;
        }
        .dashboard-container {
            max-width: 1400px;
            margin: 0 auto;
        }
        .dashboard-header {
            text-align: center;
            color: white;
            margin-bottom: 30px;
        }
        .svg-panel {
            background: white;
            border-radius: 16px;
            box-shadow: 0 8px 32px rgba(0,0,0,0.1);
            margin: 20px 0;
            overflow: hidden;
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }
        .svg-panel:hover {
            transform: translateY(-4px);
            box-shadow: 0 12px 40px rgba(0,0,0,0.15);
        }
        .panel-header {
            background: linear-gradient(90deg, #4CAF50, #45a049);
            color: white;
            padding: 15px 20px;
            font-weight: 600;
        }
        .svg-container {
            padding: 20px;
        }
        .svg-container svg {
            width: 100%;
            height: auto;
            display: block;
        }
        .controls {
            background: rgba(255,255,255,0.9);
            padding: 15px;
            border-radius: 12px;
            margin: 20px 0;
            backdrop-filter: blur(10px);
        }
    </style>
</head>
<body>
    <div class="dashboard-container">
        <div class="dashboard-header">
            <h1>🔍 Memory Analysis Dashboard</h1>
            <p>Generated at: {{TIMESTAMP}}</p>
        </div>
        
        <div class="controls">
            <button onclick="toggleTheme()">🌓 Toggle Theme</button>
            <button onclick="exportReport()">📄 Export Report</button>
            <button onclick="resetZoom()">🔍 Reset Zoom</button>
        </div>
        
        <div class="svg-panel">
            <div class="panel-header">📈 Memory Lifecycle Timeline</div>
            <div class="svg-container">{{LIFECYCLE_SVG}}</div>
        </div>
        
        <div class="svg-panel">
            <div class="panel-header">🧠 Memory Analysis Overview</div>
            <div class="svg-container">{{MEMORY_ANALYSIS_SVG}}</div>
        </div>
        
        <div class="svg-panel">
            <div class="panel-header">⚠️ Unsafe FFI Dashboard</div>
            <div class="svg-container">{{UNSAFE_FFI_SVG}}</div>
        </div>
    </div>
    
    <script>
        // 交互功能脚本
        {{INTERACTIVE_SCRIPT}}
    </script>
</body>
</html>
```

#### 1.2 修改Rust导出函数
```rust
// src/html_export.rs (新文件)
use crate::tracker::MemoryTracker;
use std::fs;

impl MemoryTracker {
    pub fn export_interactive_dashboard<P: AsRef<std::path::Path>>(&self, path: P) -> crate::types::TrackingResult<()> {
        let path = path.as_ref();
        
        // 生成各种SVG内容
        let lifecycle_svg = self.generate_lifecycle_timeline_svg()?;
        let memory_svg = self.generate_memory_analysis_svg()?;
        let unsafe_svg = self.generate_unsafe_ffi_dashboard_svg()?;
        
        // 生成交互脚本
        let interactive_script = self.generate_interactive_script()?;
        
        // 读取HTML模板
        let html_template = include_str!("dashboard_template.html");
        
        // 替换占位符
        let html = html_template
            .replace("{{TIMESTAMP}}", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .replace("{{LIFECYCLE_SVG}}", &lifecycle_svg)
            .replace("{{MEMORY_ANALYSIS_SVG}}", &memory_svg)
            .replace("{{UNSAFE_FFI_SVG}}", &unsafe_svg)
            .replace("{{INTERACTIVE_SCRIPT}}", &interactive_script);
        
        // 写入文件
        fs::write(path, html)?;
        
        println!("✅ Interactive dashboard exported to: {}", path.display());
        Ok(())
    }
}
```

#### 1.3 添加交互脚本生成
```rust
impl MemoryTracker {
    fn generate_interactive_script(&self) -> crate::types::TrackingResult<String> {
        let data = self.get_comprehensive_data()?;
        let script = format!(r#"
            // 嵌入数据
            const memoryData = {};
            
            // 主题切换
            function toggleTheme() {{
                document.body.classList.toggle('dark-theme');
            }}
            
            // 导出报告
            function exportReport() {{
                const content = document.documentElement.outerHTML;
                const blob = new Blob([content], {{type: 'text/html'}});
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = 'memory_report.html';
                a.click();
            }}
            
            // 重置缩放
            function resetZoom() {{
                document.querySelectorAll('svg').forEach(svg => {{
                    svg.style.transform = 'scale(1)';
                }});
            }}
            
            // SVG交互增强
            function enhanceSVGInteractivity() {{
                // 添加缩放功能
                document.querySelectorAll('svg').forEach(svg => {{
                    let scale = 1;
                    svg.addEventListener('wheel', function(e) {{
                        e.preventDefault();
                        const delta = e.deltaY > 0 ? 0.9 : 1.1;
                        scale = Math.max(0.5, Math.min(3, scale * delta));
                        this.style.transform = `scale(${{scale}})`;
                        this.style.transformOrigin = 'center';
                    }});
                }});
                
                // 增强悬停效果
                document.querySelectorAll('.timeline-bar, .memory-segment, .risk-indicator').forEach(element => {{
                    element.addEventListener('mouseenter', function() {{
                        this.style.filter = 'brightness(1.1) drop-shadow(0 4px 8px rgba(0,0,0,0.2))';
                    }});
                    
                    element.addEventListener('mouseleave', function() {{
                        this.style.filter = '';
                    }});
                }});
                
                // 点击显示详细信息
                document.querySelectorAll('[data-info]').forEach(element => {{
                    element.addEventListener('click', function() {{
                        const info = this.getAttribute('data-info');
                        showDetailModal(JSON.parse(info));
                    }});
                }});
            }}
            
            // 详细信息模态框
            function showDetailModal(data) {{
                const modal = document.createElement('div');
                modal.style.cssText = `
                    position: fixed; top: 0; left: 0; width: 100%; height: 100%;
                    background: rgba(0,0,0,0.8); z-index: 1000;
                    display: flex; align-items: center; justify-content: center;
                `;
                modal.innerHTML = `
                    <div style="background: white; padding: 30px; border-radius: 12px; max-width: 500px; max-height: 80vh; overflow-y: auto;">
                        <h3>${{data.title || 'Details'}}</h3>
                        <pre>${{JSON.stringify(data, null, 2)}}</pre>
                        <button onclick="this.closest('div').parentElement.remove()" style="margin-top: 20px; padding: 10px 20px; background: #4CAF50; color: white; border: none; border-radius: 6px; cursor: pointer;">Close</button>
                    </div>
                `;
                document.body.appendChild(modal);
            }}
            
            // 初始化
            document.addEventListener('DOMContentLoaded', function() {{
                enhanceSVGInteractivity();
                console.log('Memory Dashboard initialized');
                console.log('Data:', memoryData);
            }});
        "#, serde_json::to_string_pretty(&data)?);
        
        Ok(script)
    }
}
```

### Phase 2: 交互增强 (预计2-3天)

#### 2.1 添加数据关联功能
- 点击时间线上的点，高亮相关的内存分析区域
- 悬停显示详细的调用栈信息
- 支持按变量名、作用域、类型过滤

#### 2.2 增强视觉效果
- 添加平滑的动画过渡
- 支持暗色主题
- 响应式设计适配移动设备

#### 2.3 数据导航功能
- 添加搜索框，快速定位特定数据
- 时间范围选择器
- 内存使用阈值过滤

### Phase 3: 高级功能 (预计1-2天)

#### 3.1 性能优化
- 大数据集的虚拟滚动
- SVG元素的懒加载
- 交互响应优化

#### 3.2 导出功能
- 导出单个SVG图表
- 导出完整HTML报告
- 生成PDF报告（可选）

## 📋 验收标准

### 功能要求
- [ ] 能够生成包含所有SVG图表的单一HTML文件
- [ ] 支持基础交互（缩放、悬停、点击）
- [ ] 具有现代化的UI设计
- [ ] 保持现有SVG图表的视觉质量
- [ ] 零外部依赖，可离线使用

### 性能要求
- [ ] HTML文件大小 < 5MB
- [ ] 页面加载时间 < 2秒
- [ ] 交互响应时间 < 100ms
- [ ] 支持Chrome、Firefox、Safari

### 用户体验要求
- [ ] 直观的操作界面
- [ ] 清晰的数据展示
- [ ] 响应式设计
- [ ] 错误处理和用户反馈

## 🛠️ 技术栈

### 后端 (Rust)
- 现有的memscope-rs代码库
- SVG生成逻辑复用
- HTML模板系统

### 前端 (原生Web技术)
- HTML5 + CSS3
- 原生JavaScript (ES6+)
- SVG交互API
- 无外部依赖

## 📅 时间估算

- **Phase 1**: 2-3天 (基础整合)
- **Phase 2**: 2-3天 (交互增强)  
- **Phase 3**: 1-2天 (高级功能)
- **总计**: 5-8天

## 🎯 成功指标

1. **易用性**: 一键生成，双击打开即可使用
2. **专业性**: 保持现有SVG的高质量视觉效果
3. **交互性**: 提供丰富的数据探索功能
4. **可维护性**: 代码结构清晰，易于扩展
5. **性能**: 流畅的用户体验，无明显延迟

## 💡 后续扩展可能性

- 支持多个JSON文件对比分析
- 添加数据导入/导出功能
- 集成更多图表类型
- 支持实时数据更新
- 添加数据分析建议功能