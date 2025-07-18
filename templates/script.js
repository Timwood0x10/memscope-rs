// MemScope-RS Dynamic Interactive Visualizations
// Replaces static SVGs with dynamic, interactive JavaScript visualizations

class MemScopeVisualizer {
    constructor(data) {
        this.data = data;
        this.filteredAllocations = [...data.allocations];
        this.init();
    }

    init() {
        this.setupTabNavigation();
        this.populateOverview();
        this.setupInteractiveExplorer();
        this.updateHeaderStats();
    }

    // Tab Navigation System
    setupTabNavigation() {
        const tabButtons = document.querySelectorAll('.tab-btn');
        const tabContents = document.querySelectorAll('.tab-content');

        tabButtons.forEach(button => {
            button.addEventListener('click', () => {
                const targetTab = button.getAttribute('data-tab');
                
                // Update active tab button
                tabButtons.forEach(btn => btn.classList.remove('active'));
                button.classList.add('active');
                
                // Update active tab content
                tabContents.forEach(content => content.classList.remove('active'));
                document.getElementById(targetTab).classList.add('active');
                
                // Trigger tab-specific updates
                this.onTabChange(targetTab);
            });
        });
    }

    onTabChange(tabName) {
        switch(tabName) {
            case 'overview':
                this.populateOverview();
                break;
            case 'memory-analysis':
                this.renderMemoryAnalysisDashboard();
                break;
            case 'lifecycle':
                this.renderLifecycleTimeline();
                break;
            case 'unsafe-ffi':
                this.renderUnsafeFFIDashboard();
                break;
            case 'interactive':
                this.updateInteractiveExplorer();
                break;
        }
    }

    // Header Statistics
    updateHeaderStats() {
        const stats = this.data.stats;
        
        document.getElementById('totalMemory').textContent = 
            `📊 ${this.formatBytes(stats.current_memory)}`;
        document.getElementById('activeAllocs').textContent = 
            `🔢 ${stats.active_allocations.toLocaleString()} allocs`;
        document.getElementById('peakMemory').textContent = 
            `📈 Peak: ${this.formatBytes(stats.peak_memory)}`;
    }

    // Overview Tab Population
    populateOverview() {
        this.populateMemoryStats();
        this.populateTypeDistribution();
        this.populateRecentAllocations();
        this.populatePerformanceInsights();
    }

    populateMemoryStats() {
        const stats = this.data.stats;
        const container = document.getElementById('memoryStats');
        
        // 安全的数值计算
        const currentMemory = stats.current_memory || 0;
        const peakMemory = stats.peak_memory || 0;
        const activeAllocations = stats.active_allocations || 0;
        const totalAllocations = stats.total_allocations || this.data.allocations.length || 0;
        
        const memoryUtilization = peakMemory > 0 ? (currentMemory / peakMemory * 100).toFixed(1) : '0.0';
        
        container.innerHTML = `
            <div class="memory-stat">
                <span class="stat-label">Current Memory</span>
                <span class="stat-value">${this.formatBytes(currentMemory)}</span>
            </div>
            <div class="memory-stat">
                <span class="stat-label">Peak Memory</span>
                <span class="stat-value">${this.formatBytes(peakMemory)}</span>
            </div>
            <div class="memory-stat">
                <span class="stat-label">Memory Utilization</span>
                <span class="stat-value">${memoryUtilization}%</span>
            </div>
            <div class="memory-stat">
                <span class="stat-label">Active Allocations</span>
                <span class="stat-value">${activeAllocations.toLocaleString()}</span>
            </div>
            <div class="memory-stat">
                <span class="stat-label">Total Allocations</span>
                <span class="stat-value">${totalAllocations.toLocaleString()}</span>
            </div>
        `;
    }

    populateTypeDistribution() {
        const container = document.getElementById('typeDistribution');
        const typeMap = new Map();
        
        // Aggregate by type
        this.data.allocations.forEach(alloc => {
            const typeName = alloc.type_name || 'Unknown';
            if (!typeMap.has(typeName)) {
                typeMap.set(typeName, { size: 0, count: 0 });
            }
            const current = typeMap.get(typeName);
            current.size += alloc.size;
            current.count += 1;
        });
        
        // Sort by size and take top 10
        const sortedTypes = Array.from(typeMap.entries())
            .sort((a, b) => b[1].size - a[1].size)
            .slice(0, 10);
        
        container.innerHTML = sortedTypes.map(([typeName, data]) => `
            <div class="type-item">
                <span class="type-name">${this.truncateText(typeName, 25)}</span>
                <div class="type-stats">
                    <span class="type-size">${this.formatBytes(data.size)}</span>
                    <span class="type-count">${data.count} allocs</span>
                </div>
            </div>
        `).join('');
    }

    populateRecentAllocations() {
        const container = document.getElementById('recentAllocations');
        
        // Sort by timestamp and take most recent 8
        const recentAllocs = [...this.data.allocations]
            .filter(alloc => alloc.var_name) // Only show named variables
            .sort((a, b) => b.timestamp - a.timestamp)
            .slice(0, 8);
        
        if (recentAllocs.length === 0) {
            container.innerHTML = '<p style="color: #64748b; font-style: italic;">No named variables found</p>';
            return;
        }
        
        container.innerHTML = recentAllocs.map(alloc => `
            <div class="type-item">
                <span class="type-name">${alloc.var_name}</span>
                <div class="type-stats">
                    <span class="type-size">${this.formatBytes(alloc.size)}</span>
                    <span class="type-count">${alloc.type_name || 'Unknown'}</span>
                </div>
            </div>
        `).join('');
    }

    populatePerformanceInsights() {
        const container = document.getElementById('performanceInsights');
        const insights = this.generateInsights();
        
        container.innerHTML = insights.map(insight => `
            <div class="insight-item">
                <div class="insight-title">${insight.title}</div>
                <div class="insight-description">${insight.description}</div>
            </div>
        `).join('');
    }

    generateInsights() {
        const insights = [];
        const stats = this.data.stats;
        const allocations = this.data.allocations;
        
        // Memory utilization insight
        const utilization = (stats.current_memory / stats.peak_memory * 100);
        if (utilization > 80) {
            insights.push({
                title: "🔴 High Memory Utilization",
                description: `Current memory usage is ${utilization.toFixed(1)}% of peak. Consider optimizing memory usage.`
            });
        } else if (utilization < 30) {
            insights.push({
                title: "🟢 Efficient Memory Usage",
                description: `Memory utilization is low at ${utilization.toFixed(1)}%. Good memory management!`
            });
        }
        
        // Large allocations insight
        const largeAllocs = allocations.filter(a => a.size > 1024 * 1024); // > 1MB
        if (largeAllocs.length > 0) {
            insights.push({
                title: "📊 Large Allocations Detected",
                description: `Found ${largeAllocs.length} allocation(s) larger than 1MB. Review if necessary.`
            });
        }
        
        // Type diversity insight
        const uniqueTypes = new Set(allocations.map(a => a.type_name).filter(Boolean));
        insights.push({
            title: "🏷️ Type Diversity",
            description: `Using ${uniqueTypes.size} different types across ${allocations.length} allocations.`
        });
        
        // Unsafe/FFI insight
        if (this.data.unsafeFFI && this.data.unsafeFFI.violations.length > 0) {
            insights.push({
                title: "⚠️ Safety Violations",
                description: `Detected ${this.data.unsafeFFI.violations.length} safety violation(s). Review unsafe code carefully.`
            });
        } else if (this.data.unsafeFFI) {
            insights.push({
                title: "✅ No Safety Issues",
                description: "No memory safety violations detected in unsafe/FFI code."
            });
        }
        
        return insights;
    }

    // Interactive Explorer Setup
    setupInteractiveExplorer() {
        this.populateTypeFilter();
        this.setupEventListeners();
        this.updateInteractiveExplorer();
    }

    populateTypeFilter() {
        const select = document.getElementById('filterType');
        const types = new Set(this.data.allocations.map(a => a.type_name).filter(Boolean));
        
        select.innerHTML = '<option value="">All Types</option>' +
            Array.from(types).sort().map(type => 
                `<option value="${type}">${this.truncateText(type, 30)}</option>`
            ).join('');
    }

    setupEventListeners() {
        document.getElementById('filterType').addEventListener('change', () => this.updateFilters());
        document.getElementById('sizeRange').addEventListener('input', () => this.updateFilters());
        document.getElementById('sortBy').addEventListener('change', () => this.updateInteractiveExplorer());
    }

    updateFilters() {
        const typeFilter = document.getElementById('filterType').value;
        const sizeRange = document.getElementById('sizeRange').value;
        const maxSize = Math.max(...this.data.allocations.map(a => a.size));
        const sizeThreshold = (maxSize * sizeRange) / 100;
        
        // Update size range display
        document.getElementById('sizeRangeValue').textContent = 
            sizeRange == 100 ? 'All sizes' : `≤ ${this.formatBytes(sizeThreshold)}`;
        
        // Apply filters
        this.filteredAllocations = this.data.allocations.filter(alloc => {
            const typeMatch = !typeFilter || alloc.type_name === typeFilter;
            const sizeMatch = alloc.size <= sizeThreshold;
            return typeMatch && sizeMatch;
        });
        
        this.updateInteractiveExplorer();
    }

    updateInteractiveExplorer() {
        const sortBy = document.getElementById('sortBy').value;
        
        // Sort allocations
        const sorted = [...this.filteredAllocations].sort((a, b) => {
            switch(sortBy) {
                case 'size':
                    return b.size - a.size;
                case 'timestamp':
                    return b.timestamp - a.timestamp;
                case 'type':
                    return (a.type_name || '').localeCompare(b.type_name || '');
                default:
                    return 0;
            }
        });
        
        this.renderAllocationGrid(sorted);
    }

    renderAllocationGrid(allocations) {
        const container = document.getElementById('allocationGrid');
        
        if (allocations.length === 0) {
            container.innerHTML = `
                <div style="grid-column: 1 / -1; text-align: center; padding: 40px; color: #64748b;">
                    <h3>No allocations match the current filters</h3>
                    <p>Try adjusting the filters to see more results.</p>
                </div>
            `;
            return;
        }
        
        container.innerHTML = allocations.slice(0, 100).map(alloc => `
            <div class="allocation-card" onclick="memscope.showAllocationDetails(${alloc.ptr})">
                <div class="allocation-header">
                    <span class="allocation-name">${alloc.var_name || `Ptr ${alloc.ptr.toString(16)}`}</span>
                    <span class="allocation-size">${this.formatBytes(alloc.size)}</span>
                </div>
                <div class="allocation-type">${alloc.type_name || 'Unknown Type'}</div>
                <div class="allocation-details">
                    <div>Address: 0x${alloc.ptr.toString(16)}</div>
                    <div>Timestamp: ${new Date(alloc.timestamp / 1000000).toLocaleString()}</div>
                    ${alloc.call_stack && alloc.call_stack.length > 0 ? 
                        `<div>Stack depth: ${alloc.call_stack.length} frames</div>` : ''}
                </div>
            </div>
        `).join('');
        
        // Show count info
        if (allocations.length > 100) {
            container.innerHTML += `
                <div style="grid-column: 1 / -1; text-align: center; padding: 20px; color: #64748b; font-style: italic;">
                    Showing first 100 of ${allocations.length} allocations
                </div>
            `;
        }
    }

    showAllocationDetails(ptr) {
        const alloc = this.data.allocations.find(a => a.ptr === ptr);
        if (!alloc) return;
        
        const details = `
            Variable: ${alloc.var_name || 'Unnamed'}
            Type: ${alloc.type_name || 'Unknown'}
            Size: ${this.formatBytes(alloc.size)}
            Address: 0x${alloc.ptr.toString(16)}
            Timestamp: ${new Date(alloc.timestamp / 1000000).toLocaleString()}
            
            Call Stack:
            ${alloc.call_stack ? alloc.call_stack.map((frame, i) => 
                `  ${i + 1}. ${frame.function_name || 'unknown'} (${frame.file_name || 'unknown'}:${frame.line_number || '?'})`
            ).join('\n') : 'No call stack available'}
        `;
        
        alert(details); // Simple popup for now, could be enhanced with a modal
    }

    // ===========================================
    // DYNAMIC VISUALIZATION RENDERERS
    // ===========================================

    // Memory Analysis Dashboard (替换静态SVG)
    renderMemoryAnalysisDashboard() {
        const container = document.getElementById('memory-analysis');
        container.innerHTML = ''; // 清空现有内容
        
        // 创建动态仪表板容器
        const dashboard = document.createElement('div');
        dashboard.className = 'memory-dashboard';
        dashboard.innerHTML = `
            <div class="dashboard-header">
                <h2>🧠 Dynamic Memory Analysis Dashboard</h2>
                <p>Interactive visualization of memory usage patterns</p>
            </div>
            <div class="dashboard-grid">
                <div class="metric-cards" id="metricCards"></div>
                <div class="memory-heatmap" id="memoryHeatmap"></div>
                <div class="type-distribution" id="typeDistribution"></div>
                <div class="fragmentation-analysis" id="fragmentationAnalysis"></div>
                <div class="categorized-allocations" id="categorizedAllocations"></div>
                <div class="callstack-analysis" id="callstackAnalysis"></div>
                <div class="memory-growth-trends" id="memoryGrowthTrends"></div>
                <div class="variable-timeline" id="variableTimeline"></div>
                <div class="interactive-legend" id="interactiveLegend"></div>
                <div class="comprehensive-summary" id="comprehensiveSummary"></div>
            </div>
        `;
        container.appendChild(dashboard);
        
        // 渲染各个组件
        // 渲染完整的12个模块 (对应原始SVG的所有部分)
        this.renderPerformanceMetrics();           // 模块2: 性能仪表板 (3个圆形进度条)
        this.renderMemoryHeatmap();               // 模块3: 内存分配热力图
        this.renderDynamicTypeDistribution();     // 模块4: 内存使用类型图表
        this.renderFragmentationAnalysis();       // 模块5: 内存碎片化分析
        this.renderCategorizedAllocations();      // 模块6: 分类分配
        this.renderCallStackAnalysis();           // 模块7: 调用栈分析
        this.renderMemoryGrowthTrends();          // 模块8: 内存增长趋势
        this.renderVariableTimeline();            // 模块9: 变量分配时间轴
        this.renderInteractiveLegend();           // 模块10: 交互式图例
        this.renderComprehensiveSummary();        // 模块11: 综合摘要
    }

    // 性能指标圆形进度条 (原SVG中的71% Active Memory等)
    renderPerformanceMetrics() {
        const container = document.getElementById('metricCards');
        const stats = this.data.stats;
        
        // 安全的数值计算，避免NaN
        const currentMemory = stats.current_memory || 0;
        const peakMemory = stats.peak_memory || 0;
        const activeAllocations = stats.active_allocations || 0;
        
        const utilizationPercent = peakMemory > 0 ? Math.round((currentMemory / peakMemory) * 100) : 0;
        
        const metrics = [
            {
                label: 'Active Memory',
                value: this.formatBytes(currentMemory),
                percent: utilizationPercent,
                color: '#3498db',
                status: utilizationPercent > 80 ? 'HIGH' : utilizationPercent > 50 ? 'MEDIUM' : 'LOW'
            },
            {
                label: 'Peak Memory', 
                value: this.formatBytes(peakMemory),
                percent: 100,
                color: '#e74c3c',
                status: 'HIGH'
            },
            {
                label: 'Active Allocs',
                value: activeAllocations.toLocaleString(),
                percent: Math.min(100, (activeAllocations / Math.max(1, this.data.allocations.length)) * 100),
                color: '#2ecc71',
                status: 'NORMAL'
            }
        ];
        
        container.innerHTML = metrics.map((metric, index) => `
            <div class="metric-card" style="animation-delay: ${index * 0.2}s">
                <div class="circular-progress">
                    <svg width="80" height="80" viewBox="0 0 80 80">
                        <circle cx="40" cy="40" r="30" fill="none" stroke="#ecf0f1" stroke-width="6"/>
                        <circle 
                            cx="40" cy="40" r="30" fill="none" 
                            stroke="${metric.color}" 
                            stroke-width="6" 
                            stroke-linecap="round"
                            stroke-dasharray="188.5" 
                            stroke-dashoffset="${188.5 - (metric.percent / 100) * 188.5}"
                            transform="rotate(-90 40 40)"
                            class="progress-circle"
                            style="transition: stroke-dashoffset 2s cubic-bezier(0.4, 0, 0.2, 1);"
                        />
                    </svg>
                    <div class="progress-text">
                        <span class="percent" style="color: ${metric.color}">${metric.percent}%</span>
                    </div>
                </div>
                <div class="metric-info">
                    <h4>${metric.label}</h4>
                    <div class="metric-value">${metric.value}</div>
                    <div class="metric-status ${metric.status.toLowerCase()}">${metric.status}</div>
                </div>
            </div>
        `).join('');
        
        // 触发动画
        setTimeout(() => {
            document.querySelectorAll('.progress-circle').forEach((circle, index) => {
                circle.style.strokeDashoffset = `${188.5 - (metrics[index].percent / 100) * 188.5}`;
            });
        }, 100);
    }

    // 交互式内存热力图
    renderMemoryHeatmap() {
        const container = document.getElementById('memoryHeatmap');
        container.innerHTML = `
            <div class="heatmap-header">
                <h3>📊 Memory Allocation Heatmap</h3>
                <div class="heatmap-controls">
                    <button class="heatmap-btn active" data-view="size">By Size</button>
                    <button class="heatmap-btn" data-view="type">By Type</button>
                    <button class="heatmap-btn" data-view="time">By Time</button>
                </div>
            </div>
            <div class="heatmap-canvas" id="heatmapCanvas"></div>
            <div class="heatmap-legend" id="heatmapLegend"></div>
        `;
        
        // 创建热力图数据
        const allocations = this.data.allocations;
        const maxSize = Math.max(...allocations.map(a => a.size));
        
        const heatmapData = allocations.map((alloc, index) => ({
            x: (index % 20) * 25 + 10,
            y: Math.floor(index / 20) * 25 + 10,
            size: alloc.size,
            intensity: alloc.size / maxSize,
            color: this.getHeatmapColor(alloc.size / maxSize),
            allocation: alloc
        }));
        
        this.renderHeatmapCanvas(heatmapData);
        this.setupHeatmapControls();
    }

    // 渲染热力图画布
    renderHeatmapCanvas(data) {
        const canvas = document.getElementById('heatmapCanvas');
        canvas.innerHTML = `
            <svg width="500" height="300" viewBox="0 0 500 300" class="heatmap-svg">
                ${data.map((point, index) => `
                    <rect 
                        x="${point.x}" y="${point.y}" 
                        width="20" height="20" 
                        fill="${point.color}" 
                        opacity="${0.3 + point.intensity * 0.7}"
                        class="heatmap-cell"
                        data-index="${index}"
                        style="transition: all 0.3s ease; cursor: pointer;"
                    />
                `).join('')}
            </svg>
        `;
        
        // 添加悬停交互
        document.querySelectorAll('.heatmap-cell').forEach((cell, index) => {
            const allocation = data[index].allocation;
            
            cell.addEventListener('mouseenter', (e) => {
                cell.style.opacity = '1';
                cell.style.transform = 'scale(1.2)';
                this.showTooltip(e, {
                    title: allocation.var_name || `Allocation ${allocation.ptr.toString(16)}`,
                    size: this.formatBytes(allocation.size),
                    type: allocation.type_name || 'Unknown',
                    timestamp: new Date(allocation.timestamp / 1000000).toLocaleString()
                });
            });
            
            cell.addEventListener('mouseleave', () => {
                cell.style.opacity = `${0.3 + data[index].intensity * 0.7}`;
                cell.style.transform = 'scale(1)';
                this.hideTooltip();
            });
        });
    }

    // 动态类型分布图
    renderDynamicTypeDistribution() {
        const container = document.getElementById('typeDistribution');
        const typeMap = new Map();
        
        // 聚合类型数据
        this.data.allocations.forEach(alloc => {
            const typeName = alloc.type_name || 'Unknown';
            if (!typeMap.has(typeName)) {
                typeMap.set(typeName, { size: 0, count: 0, color: this.getTypeColor(typeName) });
            }
            const current = typeMap.get(typeName);
            current.size += alloc.size;
            current.count += 1;
        });
        
        const sortedTypes = Array.from(typeMap.entries())
            .sort((a, b) => b[1].size - a[1].size)
            .slice(0, 8);
        
        const maxSize = sortedTypes[0]?.[1].size || 1;
        
        container.innerHTML = `
            <div class="type-dist-header">
                <h3>🏷️ Dynamic Type Distribution</h3>
                <div class="view-toggle">
                    <button class="toggle-btn active" data-view="bar">Bar Chart</button>
                    <button class="toggle-btn" data-view="pie">Pie Chart</button>
                </div>
            </div>
            <div class="type-chart" id="typeChart">
                <svg width="400" height="250" viewBox="0 0 400 250" class="type-svg">
                    ${sortedTypes.map((type, index) => {
                        const [typeName, data] = type;
                        const barHeight = (data.size / maxSize) * 180;
                        const x = 40 + index * 45;
                        const y = 200 - barHeight;
                        
                        return `
                            <g class="type-bar-group" data-type="${typeName}">
                                <rect 
                                    x="${x}" y="${y}" 
                                    width="35" height="${barHeight}"
                                    fill="${data.color}" 
                                    class="type-bar"
                                    style="transition: all 0.5s cubic-bezier(0.4, 0, 0.2, 1); cursor: pointer;"
                                />
                                <text x="${x + 17.5}" y="220" text-anchor="middle" font-size="10" fill="#2c3e50">
                                    ${this.truncateText(typeName, 8)}
                                </text>
                                <text x="${x + 17.5}" y="${y - 5}" text-anchor="middle" font-size="9" fill="${data.color}" font-weight="bold">
                                    ${data.count}
                                </text>
                            </g>
                        `;
                    }).join('')}
                </svg>
            </div>
        `;
        
        // 添加交互效果
        this.setupTypeDistributionInteractions(sortedTypes);
    }

    // Lifecycle Timeline (替换静态SVG)
    renderLifecycleTimeline() {
        const container = document.getElementById('lifecycle');
        container.innerHTML = '';
        
        const timeline = document.createElement('div');
        timeline.className = 'lifecycle-timeline';
        timeline.innerHTML = `
            <div class="timeline-header">
                <h2>⏱️ Dynamic Scope Matrix & Lifecycle</h2>
                <div class="timeline-controls">
                    <button class="timeline-btn" id="playBtn">▶️ Play</button>
                    <button class="timeline-btn" id="pauseBtn">⏸️ Pause</button>
                    <button class="timeline-btn" id="resetBtn">🔄 Reset</button>
                    <input type="range" id="timelineSlider" min="0" max="100" value="0" class="timeline-slider">
                </div>
            </div>
            <div class="scope-matrix" id="scopeMatrix"></div>
            <div class="variable-relationships" id="variableRelationships"></div>
        `;
        container.appendChild(timeline);
        
        this.renderScopeMatrix();
        this.renderVariableRelationships();
        this.setupTimelineControls();
    }

    // Unsafe FFI Dashboard (替换静态SVG)
    renderUnsafeFFIDashboard() {
        const container = document.getElementById('unsafe-ffi');
        
        if (!this.data.unsafeFFI || !this.data.unsafeFFI.allocations || this.data.unsafeFFI.allocations.length === 0) {
            container.innerHTML = `
                <div class="empty-state enhanced">
                    <div class="empty-icon">🛡️</div>
                    <h3>No Unsafe/FFI Data Available</h3>
                    <p>This analysis did not detect any unsafe Rust code or FFI operations.</p>
                    <p>This is generally a good sign for memory safety! 🎉</p>
                    <div class="safety-score">
                        <div class="score-circle">
                            <span class="score">100</span>
                            <span class="score-label">Safety Score</span>
                        </div>
                    </div>
                </div>
            `;
            return;
        }
        
        container.innerHTML = '';
        const dashboard = document.createElement('div');
        dashboard.className = 'unsafe-ffi-dashboard';
        dashboard.innerHTML = `
            <div class="ffi-header">
                <h2>⚠️ Dynamic Unsafe/FFI Analysis</h2>
                <div class="safety-alert ${this.data.unsafeFFI.violations.length > 0 ? 'danger' : 'safe'}">
                    ${this.data.unsafeFFI.violations.length > 0 ? '🚨 Safety Issues Detected' : '✅ No Safety Issues'}
                </div>
            </div>
            <div class="ffi-metrics" id="ffiMetrics"></div>
            <div class="ffi-flow" id="ffiFlow"></div>
            <div class="ffi-hotspots" id="ffiHotspots"></div>
        `;
        container.appendChild(dashboard);
        
        this.renderFFIMetrics();
        this.renderFFIFlow();
        this.renderFFIHotspots();
    }

    // Utility Functions
    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    }

    truncateText(text, maxLength) {
        if (!text) return 'Unknown';
        return text.length > maxLength ? text.substring(0, maxLength) + '...' : text;
    }

    // ===========================================
    // UTILITY FUNCTIONS FOR DYNAMIC VISUALIZATIONS
    // ===========================================

    getHeatmapColor(intensity) {
        // 从蓝色到红色的渐变
        const colors = [
            '#3498db', '#2ecc71', '#f1c40f', '#e67e22', '#e74c3c'
        ];
        const index = Math.floor(intensity * (colors.length - 1));
        return colors[Math.min(index, colors.length - 1)];
    }

    getTypeColor(typeName) {
        if (!typeName || typeName === 'Unknown') {
            return '#95a5a6';
        }
        
        const colors = {
            'Vec': '#3498db',
            'String': '#2ecc71', 
            'Box': '#e74c3c',
            'HashMap': '#9b59b6',
            'BTreeMap': '#f39c12',
            'Small Object': '#1abc9c',
            'Medium Structure': '#3498db',
            'Large Buffer': '#e74c3c',
            'Huge Object': '#8e44ad'
        };
        
        // 精确匹配
        if (colors[typeName]) {
            return colors[typeName];
        }
        
        // 部分匹配
        for (const [key, color] of Object.entries(colors)) {
            if (typeName.includes(key)) return color;
        }
        
        // 为其他类型生成一致的颜色
        let hash = 0;
        for (let i = 0; i < typeName.length; i++) {
            hash = typeName.charCodeAt(i) + ((hash << 5) - hash);
        }
        const hue = Math.abs(hash) % 360;
        return `hsl(${hue}, 70%, 50%)`;
    }

    showTooltip(event, data) {
        let tooltip = document.getElementById('dynamicTooltip');
        if (!tooltip) {
            tooltip = document.createElement('div');
            tooltip.id = 'dynamicTooltip';
            tooltip.className = 'dynamic-tooltip';
            document.body.appendChild(tooltip);
        }
        
        tooltip.innerHTML = `
            <div class="tooltip-header">${data.title}</div>
            <div class="tooltip-content">
                <div><strong>Size:</strong> ${data.size}</div>
                <div><strong>Type:</strong> ${data.type}</div>
                <div><strong>Time:</strong> ${data.timestamp}</div>
            </div>
        `;
        
        tooltip.style.display = 'block';
        tooltip.style.left = event.pageX + 10 + 'px';
        tooltip.style.top = event.pageY + 10 + 'px';
    }

    hideTooltip() {
        const tooltip = document.getElementById('dynamicTooltip');
        if (tooltip) {
            tooltip.style.display = 'none';
        }
    }

    setupHeatmapControls() {
        document.querySelectorAll('.heatmap-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.heatmap-btn').forEach(b => b.classList.remove('active'));
                e.target.classList.add('active');
                
                const view = e.target.dataset.view;
                this.updateHeatmapView(view);
            });
        });
    }

    updateHeatmapView(view) {
        // 根据视图类型重新渲染热力图
        console.log(`Switching heatmap to ${view} view`);
        // 这里可以添加不同视图的逻辑
    }

    setupTypeDistributionInteractions(types) {
        document.querySelectorAll('.type-bar').forEach((bar, index) => {
            const typeData = types[index][1];
            
            bar.addEventListener('mouseenter', () => {
                bar.style.transform = 'scaleY(1.1)';
                bar.style.filter = 'brightness(1.2)';
            });
            
            bar.addEventListener('mouseleave', () => {
                bar.style.transform = 'scaleY(1)';
                bar.style.filter = 'brightness(1)';
            });
            
            bar.addEventListener('click', () => {
                this.showTypeDetails(types[index]);
            });
        });
    }

    showTypeDetails(typeData) {
        const [typeName, data] = typeData;
        alert(`Type: ${typeName}\nAllocations: ${data.count}\nTotal Size: ${this.formatBytes(data.size)}`);
    }

    // 作用域矩阵渲染
    renderScopeMatrix() {
        const container = document.getElementById('scopeMatrix');
        const trackedVars = this.data.allocations.filter(a => a.var_name);
        
        if (trackedVars.length === 0) {
            container.innerHTML = '<div class="no-data">No tracked variables found</div>';
            return;
        }
        
        // 按作用域分组变量
        const scopes = this.groupVariablesByScope(trackedVars);
        
        container.innerHTML = Object.entries(scopes).map(([scopeName, vars]) => `
            <div class="scope-container" data-scope="${scopeName}">
                <div class="scope-header">
                    <h4>📦 ${scopeName}</h4>
                    <span class="scope-stats">${vars.length} variables</span>
                </div>
                <div class="scope-variables">
                    ${vars.map(v => `
                        <div class="variable-item">
                            <div class="var-name">${v.var_name}</div>
                            <div class="var-progress">
                                <div class="progress-bar" style="width: ${Math.random() * 100}%; background: ${this.getTypeColor(v.type_name || 'Unknown')}"></div>
                            </div>
                            <div class="var-size">${this.formatBytes(v.size)}</div>
                        </div>
                    `).join('')}
                </div>
            </div>
        `).join('');
    }

    groupVariablesByScope(variables) {
        const scopes = {};
        variables.forEach(v => {
            const scope = this.extractScope(v);
            if (!scopes[scope]) scopes[scope] = [];
            scopes[scope].push(v);
        });
        return scopes;
    }

    extractScope(variable) {
        // 简单的作用域提取逻辑
        if (variable.var_name) {
            if (variable.var_name.includes('global')) return 'Global';
            if (variable.var_name.includes('main')) return 'Main Function';
            if (variable.var_name.includes('test')) return 'Test Scope';
        }
        return 'Local Scope';
    }

    renderVariableRelationships() {
        const container = document.getElementById('variableRelationships');
        container.innerHTML = `
            <div class="relationships-header">
                <h4>🔗 Variable Relationships</h4>
            </div>
            <div class="relationship-graph">
                <svg width="100%" height="200" viewBox="0 0 500 200">
                    <!-- 这里可以添加变量关系的连线图 -->
                    <text x="250" y="100" text-anchor="middle" fill="#64748b">
                        Relationship analysis coming soon...
                    </text>
                </svg>
            </div>
        `;
    }

    setupTimelineControls() {
        // 时间轴控制逻辑
        document.getElementById('playBtn')?.addEventListener('click', () => {
            console.log('Timeline play');
        });
        
        document.getElementById('pauseBtn')?.addEventListener('click', () => {
            console.log('Timeline pause');
        });
        
        document.getElementById('resetBtn')?.addEventListener('click', () => {
            console.log('Timeline reset');
        });
    }

    renderFFIMetrics() {
        const container = document.getElementById('ffiMetrics');
        const ffiData = this.data.unsafeFFI;
        
        container.innerHTML = `
            <div class="ffi-metric-cards">
                <div class="ffi-card danger">
                    <div class="card-value">${ffiData.violations.length}</div>
                    <div class="card-label">Safety Violations</div>
                </div>
                <div class="ffi-card warning">
                    <div class="card-value">${ffiData.allocations.length}</div>
                    <div class="card-label">Unsafe Allocations</div>
                </div>
                <div class="ffi-card info">
                    <div class="card-value">${ffiData.boundaryEvents.length}</div>
                    <div class="card-label">Boundary Events</div>
                </div>
            </div>
        `;
    }

    renderFFIFlow() {
        const container = document.getElementById('ffiFlow');
        container.innerHTML = `
            <div class="flow-diagram">
                <h4>🔄 Memory Flow Analysis</h4>
                <div class="flow-visualization">
                    <!-- 动态流程图将在这里渲染 -->
                    <div class="flow-placeholder">Interactive flow diagram coming soon...</div>
                </div>
            </div>
        `;
    }

    renderFFIHotspots() {
        const container = document.getElementById('ffiHotspots');
        container.innerHTML = `
            <div class="hotspots-map">
                <h4>🔥 Memory Hotspots</h4>
                <div class="hotspot-visualization">
                    <!-- 热点气泡图将在这里渲染 -->
                    <div class="hotspot-placeholder">Hotspot visualization coming soon...</div>
                </div>
            </div>
        `;
    }

    // ===========================================
    // 完整12个模块实现 (对应原始SVG)
    // ===========================================

    // 模块5: 内存碎片化分析
    renderFragmentationAnalysis() {
        const container = document.getElementById('fragmentationAnalysis');
        const allocations = this.data.allocations;
        
        // 计算碎片化指标
        const totalMemory = allocations.reduce((sum, a) => sum + a.size, 0);
        const avgSize = totalMemory / allocations.length || 0;
        const sizeVariance = allocations.reduce((sum, a) => sum + Math.pow(a.size - avgSize, 2), 0) / allocations.length;
        const fragmentationScore = Math.min(100, (sizeVariance / (avgSize * avgSize)) * 100);
        
        container.innerHTML = `
            <div class="analysis-header">
                <h3>🧩 Memory Fragmentation Analysis</h3>
                <div class="fragmentation-score ${fragmentationScore > 70 ? 'high' : fragmentationScore > 40 ? 'medium' : 'low'}">
                    ${fragmentationScore.toFixed(1)}% Fragmented
                </div>
            </div>
            <div class="fragmentation-visual">
                <div class="memory-blocks" id="memoryBlocks"></div>
                <div class="fragmentation-metrics">
                    <div class="metric-item">
                        <span class="metric-label">Average Size:</span>
                        <span class="metric-value">${this.formatBytes(avgSize)}</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">Size Variance:</span>
                        <span class="metric-value">${this.formatBytes(Math.sqrt(sizeVariance))}</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">Total Blocks:</span>
                        <span class="metric-value">${allocations.length}</span>
                    </div>
                </div>
            </div>
        `;
        
        this.renderMemoryBlocks(allocations);
    }

    renderMemoryBlocks(allocations) {
        const container = document.getElementById('memoryBlocks');
        const maxSize = Math.max(...allocations.map(a => a.size));
        
        // 创建内存块可视化
        const blocks = allocations.slice(0, 20).map((alloc, index) => {
            const width = Math.max(10, (alloc.size / maxSize) * 100);
            const height = 15;
            const color = this.getTypeColor(alloc.type_name || 'Unknown');
            
            return `
                <div class="memory-block" 
                     style="width: ${width}px; height: ${height}px; background: ${color}; margin: 2px;"
                     title="${alloc.var_name || 'Unknown'}: ${this.formatBytes(alloc.size)}">
                </div>
            `;
        }).join('');
        
        container.innerHTML = `<div class="blocks-container">${blocks}</div>`;
    }

    // 模块6: 分类分配
    renderCategorizedAllocations() {
        const container = document.getElementById('categorizedAllocations');
        const allocations = this.data.allocations;
        
        // 按大小分类
        const categories = {
            'Small (< 1KB)': allocations.filter(a => a.size < 1024),
            'Medium (1KB - 100KB)': allocations.filter(a => a.size >= 1024 && a.size < 102400),
            'Large (100KB - 1MB)': allocations.filter(a => a.size >= 102400 && a.size < 1048576),
            'Huge (> 1MB)': allocations.filter(a => a.size >= 1048576)
        };
        
        container.innerHTML = `
            <div class="categories-header">
                <h3>📂 Categorized Allocations</h3>
                <div class="category-toggle">
                    <button class="cat-btn active" data-cat="size">By Size</button>
                    <button class="cat-btn" data-cat="type">By Type</button>
                </div>
            </div>
            <div class="categories-list" id="categoriesList"></div>
        `;
        
        this.renderCategoryList(categories);
        this.setupCategoryToggle();
    }

    renderCategoryList(categories) {
        const container = document.getElementById('categoriesList');
        
        container.innerHTML = Object.entries(categories).map(([name, allocs]) => {
            const totalSize = allocs.reduce((sum, a) => sum + a.size, 0);
            const percentage = (allocs.length / this.data.allocations.length * 100).toFixed(1);
            
            return `
                <div class="category-item">
                    <div class="category-header">
                        <span class="category-name">${name}</span>
                        <span class="category-count">${allocs.length} (${percentage}%)</span>
                    </div>
                    <div class="category-bar">
                        <div class="bar-fill" style="width: ${percentage}%; background: ${this.getCategoryColor(name)}"></div>
                    </div>
                    <div class="category-size">${this.formatBytes(totalSize)}</div>
                </div>
            `;
        }).join('');
    }

    // 模块7: 调用栈分析
    renderCallStackAnalysis() {
        const container = document.getElementById('callstackAnalysis');
        const allocations = this.data.allocations.filter(a => a.call_stack && a.call_stack.length > 0);
        
        if (allocations.length === 0) {
            container.innerHTML = `
                <div class="analysis-header">
                    <h3>📞 Call Stack Analysis</h3>
                </div>
                <div class="no-callstack">No call stack information available</div>
            `;
            return;
        }
        
        // 分析调用栈深度
        const stackDepths = allocations.map(a => a.call_stack.length);
        const avgDepth = stackDepths.reduce((sum, d) => sum + d, 0) / stackDepths.length;
        const maxDepth = Math.max(...stackDepths);
        
        // 统计常见函数
        const functionCounts = new Map();
        allocations.forEach(a => {
            a.call_stack.forEach(frame => {
                const funcName = frame.function_name || 'unknown';
                functionCounts.set(funcName, (functionCounts.get(funcName) || 0) + 1);
            });
        });
        
        const topFunctions = Array.from(functionCounts.entries())
            .sort((a, b) => b[1] - a[1])
            .slice(0, 8);
        
        container.innerHTML = `
            <div class="analysis-header">
                <h3>📞 Call Stack Analysis</h3>
                <div class="stack-stats">
                    <span>Avg Depth: ${avgDepth.toFixed(1)}</span>
                    <span>Max Depth: ${maxDepth}</span>
                </div>
            </div>
            <div class="callstack-visual">
                <div class="depth-distribution" id="depthDistribution"></div>
                <div class="top-functions">
                    <h4>Top Functions</h4>
                    ${topFunctions.map(([func, count]) => `
                        <div class="function-item">
                            <span class="func-name">${this.truncateText(func, 20)}</span>
                            <span class="func-count">${count}</span>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
        
        this.renderDepthDistribution(stackDepths);
    }

    renderDepthDistribution(depths) {
        const container = document.getElementById('depthDistribution');
        const maxDepth = Math.max(...depths);
        const depthCounts = new Array(maxDepth + 1).fill(0);
        
        depths.forEach(depth => depthCounts[depth]++);
        const maxCount = Math.max(...depthCounts);
        
        container.innerHTML = `
            <h4>Stack Depth Distribution</h4>
            <div class="depth-bars">
                ${depthCounts.map((count, depth) => {
                    const height = count > 0 ? (count / maxCount * 60) : 0;
                    return `
                        <div class="depth-bar" style="height: ${height}px" title="Depth ${depth}: ${count} allocations">
                            <span class="depth-label">${depth}</span>
                        </div>
                    `;
                }).join('')}
            </div>
        `;
    }

    // 模块8: 内存增长趋势
    renderMemoryGrowthTrends() {
        const container = document.getElementById('memoryGrowthTrends');
        const allocations = this.data.allocations.sort((a, b) => a.timestamp - b.timestamp);
        
        container.innerHTML = `
            <div class="trends-header">
                <h3>📈 Memory Growth Trends</h3>
                <div class="trend-controls">
                    <button class="trend-btn active" data-trend="cumulative">Cumulative</button>
                    <button class="trend-btn" data-trend="rate">Growth Rate</button>
                </div>
            </div>
            <div class="trends-chart" id="trendsChart"></div>
        `;
        
        this.renderTrendsChart(allocations);
        this.setupTrendControls();
    }

    renderTrendsChart(allocations) {
        const container = document.getElementById('trendsChart');
        
        if (allocations.length === 0) {
            container.innerHTML = '<div class="no-data">No allocation data for trends</div>';
            return;
        }
        
        // 计算累积内存使用
        let cumulativeMemory = 0;
        const dataPoints = allocations.map((alloc, index) => {
            cumulativeMemory += alloc.size;
            return {
                x: index,
                y: cumulativeMemory,
                timestamp: alloc.timestamp
            };
        });
        
        const maxMemory = Math.max(...dataPoints.map(p => p.y));
        
        // 大幅增加图表尺寸，让它更加突出
        const chartWidth = 600;
        const chartHeight = 280;
        const margin = { top: 20, right: 40, bottom: 60, left: 60 };
        const innerWidth = chartWidth - margin.left - margin.right;
        const innerHeight = chartHeight - margin.top - margin.bottom;
        
        // 修复路径计算，确保不越界，添加边距
        const pathData = dataPoints.map((point, index) => {
            const x = dataPoints.length > 1 ? 
                margin.left + (point.x / (dataPoints.length - 1)) * innerWidth : 
                margin.left + innerWidth / 2;
            const y = maxMemory > 0 ? 
                margin.top + innerHeight - (point.y / maxMemory) * innerHeight : 
                margin.top + innerHeight / 2;
            
            // 确保坐标在有效范围内
            const safeX = Math.max(margin.left, Math.min(margin.left + innerWidth, x));
            const safeY = Math.max(margin.top, Math.min(margin.top + innerHeight, y));
            
            return index === 0 ? `M ${safeX} ${safeY}` : `L ${safeX} ${safeY}`;
        }).join(' ');
        
        // 生成网格线
        const gridLines = [];
        for (let i = 0; i <= 5; i++) {
            const y = margin.top + (i / 5) * innerHeight;
            const value = maxMemory * (1 - i / 5);
            gridLines.push(`
                <line x1="${margin.left}" y1="${y}" x2="${margin.left + innerWidth}" y2="${y}" 
                      stroke="#ecf0f1" stroke-width="1"/>
                <text x="${margin.left - 10}" y="${y + 4}" text-anchor="end" font-size="10" fill="#7f8c8d">
                    ${this.formatBytes(value)}
                </text>
            `);
        }
        
        container.innerHTML = `
            <div class="chart-container">
                <svg width="100%" height="${chartHeight + 40}" viewBox="0 0 ${chartWidth} ${chartHeight + 40}" class="trends-svg">
                    <defs>
                        <linearGradient id="trendGradient" x1="0%" y1="0%" x2="0%" y2="100%">
                            <stop offset="0%" style="stop-color:#3498db;stop-opacity:0.6" />
                            <stop offset="100%" style="stop-color:#3498db;stop-opacity:0.1" />
                        </linearGradient>
                        <filter id="dropShadow">
                            <feDropShadow dx="0" dy="2" stdDeviation="3" flood-color="rgba(52, 152, 219, 0.3)"/>
                        </filter>
                    </defs>
                    
                    <!-- 背景 -->
                    <rect x="${margin.left}" y="${margin.top}" width="${innerWidth}" height="${innerHeight}" 
                          fill="#f8fafc" stroke="#ecf0f1" stroke-width="1" rx="4"/>
                    
                    <!-- 网格线 -->
                    ${gridLines.join('')}
                    
                    <!-- 数据可视化 -->
                    ${dataPoints.length > 1 ? `
                        <!-- 填充区域 -->
                        <path d="${pathData} L ${margin.left + innerWidth} ${margin.top + innerHeight} L ${margin.left} ${margin.top + innerHeight} Z" 
                              fill="url(#trendGradient)" stroke="none"/>
                        
                        <!-- 趋势线 -->
                        <path d="${pathData}" fill="none" stroke="#3498db" stroke-width="3" 
                              filter="url(#dropShadow)" stroke-linecap="round"/>
                        
                        <!-- 数据点 -->
                        ${dataPoints.map((point, index) => {
                            const x = margin.left + (point.x / (dataPoints.length - 1)) * innerWidth;
                            const y = margin.top + innerHeight - (point.y / maxMemory) * innerHeight;
                            return `
                                <circle cx="${x}" cy="${y}" r="4" fill="#3498db" stroke="white" stroke-width="2"
                                        class="data-point" data-index="${index}"/>
                            `;
                        }).join('')}
                    ` : `
                        <circle cx="${margin.left + innerWidth/2}" cy="${margin.top + innerHeight/2}" r="8" 
                                fill="#3498db" stroke="white" stroke-width="3"/>
                        <text x="${margin.left + innerWidth/2}" y="${margin.top + innerHeight/2 + 30}" 
                              text-anchor="middle" font-size="12" fill="#7f8c8d">
                            Single allocation
                        </text>
                    `}
                    
                    <!-- 坐标轴标签 -->
                    <text x="${margin.left}" y="${chartHeight + 20}" font-size="12" fill="#7f8c8d">Start</text>
                    <text x="${margin.left + innerWidth}" y="${chartHeight + 20}" font-size="12" fill="#7f8c8d" text-anchor="end">Now</text>
                    
                    <!-- 标题 -->
                    <text x="${chartWidth/2}" y="15" text-anchor="middle" font-size="14" font-weight="600" fill="#2c3e50">
                        Memory Growth Over Time (Peak: ${this.formatBytes(maxMemory)})
                    </text>
                </svg>
            </div>
        `;
        
        // 添加数据点交互
        this.setupTrendsInteraction(dataPoints);
    }

    // 模块9: 变量分配时间轴
    renderVariableTimeline() {
        const container = document.getElementById('variableTimeline');
        const trackedVars = this.data.allocations.filter(a => a.var_name);
        
        container.innerHTML = `
            <div class="timeline-header">
                <h3>⏰ Variable Allocation Timeline</h3>
                <div class="timeline-info">
                    ${trackedVars.length} tracked variables
                </div>
            </div>
            <div class="timeline-visual" id="timelineVisual"></div>
        `;
        
        this.renderTimelineVisual(trackedVars);
    }

    renderTimelineVisual(variables) {
        const container = document.getElementById('timelineVisual');
        
        if (variables.length === 0) {
            container.innerHTML = '<div class="no-timeline">No tracked variables for timeline</div>';
            return;
        }
        
        const sortedVars = variables.sort((a, b) => a.timestamp - b.timestamp);
        const timelineWidth = 500;
        const itemHeight = 25;
        
        container.innerHTML = `
            <div class="timeline-container">
                ${sortedVars.slice(0, 15).map((variable, index) => {
                    const relativeTime = index / (sortedVars.length - 1);
                    const x = relativeTime * timelineWidth;
                    const color = this.getTypeColor(variable.type_name || 'Unknown');
                    
                    return `
                        <div class="timeline-item" style="top: ${index * itemHeight}px;">
                            <div class="timeline-dot" style="left: ${x}px; background: ${color}"></div>
                            <div class="timeline-label" style="left: ${x + 15}px;">
                                <span class="var-name">${variable.var_name}</span>
                                <span class="var-size">${this.formatBytes(variable.size)}</span>
                            </div>
                        </div>
                    `;
                }).join('')}
            </div>
        `;
    }

    // 模块10: 交互式图例
    renderInteractiveLegend() {
        const container = document.getElementById('interactiveLegend');
        
        const legendItems = [
            { color: '#3498db', label: 'Active Memory', description: 'Currently allocated memory' },
            { color: '#e74c3c', label: 'Peak Memory', description: 'Maximum memory usage' },
            { color: '#2ecc71', label: 'Safe Allocations', description: 'Memory-safe allocations' },
            { color: '#f39c12', label: 'Medium Priority', description: 'Moderate memory usage' },
            { color: '#9b59b6', label: 'Large Objects', description: 'Objects > 100KB' },
            { color: '#1abc9c', label: 'Small Objects', description: 'Objects < 1KB' }
        ];
        
        container.innerHTML = `
            <div class="legend-header">
                <h3>🎨 Interactive Legend & Guide</h3>
            </div>
            <div class="legend-grid">
                ${legendItems.map(item => `
                    <div class="legend-item" data-color="${item.color}">
                        <div class="legend-color" style="background: ${item.color}"></div>
                        <div class="legend-text">
                            <div class="legend-label">${item.label}</div>
                            <div class="legend-desc">${item.description}</div>
                        </div>
                    </div>
                `).join('')}
            </div>
        `;
        
        this.setupLegendInteractions();
    }

    // 模块11: 综合摘要
    renderComprehensiveSummary() {
        const container = document.getElementById('comprehensiveSummary');
        const stats = this.data.stats;
        const allocations = this.data.allocations;
        
        // 计算关键指标 - 安全计算避免NaN
        const currentMemory = stats.current_memory || 0;
        const peakMemory = stats.peak_memory || 0;
        const efficiency = peakMemory > 0 ? ((currentMemory / peakMemory) * 100).toFixed(1) : '0.0';
        
        const totalMemoryUsed = allocations.reduce((sum, a) => sum + (a.size || 0), 0);
        const avgSize = allocations.length > 0 ? (totalMemoryUsed / allocations.length) : 0;
        
        const trackedVars = allocations.filter(a => a.var_name && a.var_name !== 'Unknown').length;
        const trackedPercentage = allocations.length > 0 ? ((trackedVars / allocations.length) * 100).toFixed(1) : '0.0';
        
        container.innerHTML = `
            <div class="summary-header">
                <h3>📋 Comprehensive Memory Analysis Summary</h3>
            </div>
            <div class="summary-grid">
                <div class="summary-section">
                    <h4>Memory Efficiency</h4>
                    <div class="efficiency-meter">
                        <div class="meter-bar">
                            <div class="meter-fill" style="width: ${efficiency}%; background: ${efficiency > 80 ? '#e74c3c' : efficiency > 60 ? '#f39c12' : '#2ecc71'}"></div>
                        </div>
                        <span class="meter-value">${efficiency}%</span>
                    </div>
                </div>
                
                <div class="summary-section">
                    <h4>Key Metrics</h4>
                    <div class="metrics-list">
                        <div class="metric-row">
                            <span>Average Allocation Size:</span>
                            <span>${this.formatBytes(avgSize)}</span>
                        </div>
                        <div class="metric-row">
                            <span>Tracked Variables:</span>
                            <span>${trackedVars} (${trackedPercentage}%)</span>
                        </div>
                        <div class="metric-row">
                            <span>Memory Utilization:</span>
                            <span>${efficiency}%</span>
                        </div>
                    </div>
                </div>
                
                <div class="summary-section">
                    <h4>Recommendations</h4>
                    <div class="recommendations">
                        ${this.generateRecommendations(stats, allocations)}
                    </div>
                </div>
            </div>
        `;
    }

    generateRecommendations(stats, allocations) {
        const recommendations = [];
        const efficiency = (stats.current_memory / stats.peak_memory) * 100;
        
        if (efficiency > 80) {
            recommendations.push('⚠️ High memory utilization - consider optimization');
        }
        
        if (allocations.length > 1000) {
            recommendations.push('📊 Large number of allocations - consider pooling');
        }
        
        const largeAllocs = allocations.filter(a => a.size > 1048576).length;
        if (largeAllocs > 0) {
            recommendations.push(`🔍 ${largeAllocs} large allocations detected`);
        }
        
        if (recommendations.length === 0) {
            recommendations.push('✅ Memory usage appears optimal');
        }
        
        return recommendations.map(rec => `<div class="recommendation">${rec}</div>`).join('');
    }

    // 辅助函数
    getCategoryColor(categoryName) {
        const colors = {
            'Small': '#2ecc71',
            'Medium': '#3498db', 
            'Large': '#f39c12',
            'Huge': '#e74c3c'
        };
        
        for (const [key, color] of Object.entries(colors)) {
            if (categoryName.includes(key)) return color;
        }
        return '#95a5a6';
    }

    setupCategoryToggle() {
        document.querySelectorAll('.cat-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.cat-btn').forEach(b => b.classList.remove('active'));
                e.target.classList.add('active');
                
                const category = e.target.dataset.cat;
                if (category === 'type') {
                    this.renderCategoriesByType();
                } else {
                    this.renderCategorizedAllocations();
                }
            });
        });
    }

    setupTrendControls() {
        document.querySelectorAll('.trend-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.trend-btn').forEach(b => b.classList.remove('active'));
                e.target.classList.add('active');
                
                const trend = e.target.dataset.trend;
                console.log(`Switching to ${trend} trend view`);
            });
        });
    }

    setupLegendInteractions() {
        document.querySelectorAll('.legend-item').forEach(item => {
            item.addEventListener('click', () => {
                const color = item.dataset.color;
                this.highlightElementsByColor(color);
            });
        });
    }

    highlightElementsByColor(color) {
        // 高亮显示对应颜色的元素
        console.log(`Highlighting elements with color: ${color}`);
    }

    setupTrendsInteraction(dataPoints) {
        document.querySelectorAll('.data-point').forEach((point, index) => {
            const data = dataPoints[index];
            
            point.addEventListener('mouseenter', (e) => {
                point.setAttribute('r', '6');
                point.style.filter = 'brightness(1.2)';
                
                this.showTooltip(e, {
                    title: `Data Point ${index + 1}`,
                    size: this.formatBytes(data.y),
                    type: 'Cumulative Memory',
                    timestamp: new Date(data.timestamp / 1000000).toLocaleString()
                });
            });
            
            point.addEventListener('mouseleave', () => {
                point.setAttribute('r', '4');
                point.style.filter = 'none';
                this.hideTooltip();
            });
        });
    }
}

// Initialize when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    // Global instance for easy access
    window.memscope = new MemScopeVisualizer(MEMORY_DATA);
    
    // Add some debug info to console
    console.log('🔍 MemScope-RS Interactive Visualizer Loaded');
    console.log('📊 Data Summary:', {
        allocations: MEMORY_DATA.allocations.length,
        totalMemory: window.memscope.formatBytes(MEMORY_DATA.stats.current_memory),
        hasUnsafeFFI: !!MEMORY_DATA.unsafeFFI,
        timestamp: MEMORY_DATA.timestamp
    });
});