// MemScope-RS Interactive Visualizations - Fixed Version
// This version properly handles embedded data

/**
 * 初始化应用程序
 */
function initializeMemScopeApp() {
    console.log('🚀 Initializing MemScope-RS Interactive App...');
    
    try {
        // 使用嵌入的数据而不是尝试从外部加载
        if (typeof EMBEDDED_DATA !== 'undefined' && EMBEDDED_DATA) {
            console.log('✅ Using embedded data');
            processEmbeddedData(EMBEDDED_DATA);
        } else {
            console.warn('⚠️ No embedded data found, showing error state');
            showErrorState(new Error('No data available'));
        }
    } catch (error) {
        console.error('❌ Initialization failed:', error);
        showErrorState(error);
    }
}

/**
 * 处理嵌入的数据
 */
function processEmbeddedData(data) {
    console.log('📊 Processing embedded data...');
    
    try {
        // 初始化可视化器
        globalVisualizer = new MemScopeVisualizer(data);
        
        // 更新统计信息
        updateHeaderStats(data.stats);
        
        // 初始化各个标签页
        initializeTabs();
        
        // 渲染概览页面
        renderOverviewTab(data);
        
        console.log('✅ Data processing completed successfully');
    } catch (error) {
        console.error('❌ Data processing failed:', error);
        showErrorState(error);
    }
}

/**
 * 更新头部统计信息
 */
function updateHeaderStats(stats) {
    const totalMemoryEl = document.getElementById('totalMemory');
    const activeAllocsEl = document.getElementById('activeAllocs');
    const peakMemoryEl = document.getElementById('peakMemory');
    
    if (totalMemoryEl) totalMemoryEl.textContent = formatBytes(stats.active_memory || 0);
    if (activeAllocsEl) activeAllocsEl.textContent = `${stats.active_allocations || 0} Active`;
    if (peakMemoryEl) peakMemoryEl.textContent = formatBytes(stats.peak_memory || 0);
}

/**
 * 渲染概览标签页
 */
function renderOverviewTab(data) {
    renderMemoryStats(data.stats);
    renderTypeDistribution(data.memoryByType || {});
    renderRecentAllocations(data.allocations || []);
    renderPerformanceInsights(data.stats);
}

/**
 * 渲染内存统计
 */
function renderMemoryStats(stats) {
    const element = document.getElementById('memoryStats');
    if (!element) return;
    
    const html = `
        <div class="stats-grid">
            <div class="stat-item">
                <span class="stat-label">Active Memory:</span>
                <span class="stat-value">${formatBytes(stats.active_memory || 0)}</span>
            </div>
            <div class="stat-item">
                <span class="stat-label">Peak Memory:</span>
                <span class="stat-value">${formatBytes(stats.peak_memory || 0)}</span>
            </div>
            <div class="stat-item">
                <span class="stat-label">Total Allocations:</span>
                <span class="stat-value">${stats.total_allocations || 0}</span>
            </div>
            <div class="stat-item">
                <span class="stat-label">Active Allocations:</span>
                <span class="stat-value">${stats.active_allocations || 0}</span>
            </div>
        </div>
    `;
    element.innerHTML = html;
}

/**
 * 渲染类型分布
 */
function renderTypeDistribution(memoryByType) {
    const element = document.getElementById('typeDistribution');
    if (!element) return;
    
    const types = Object.entries(memoryByType).slice(0, 5);
    
    if (types.length === 0) {
        element.innerHTML = '<p>No type information available</p>';
        return;
    }
    
    const html = types.map(([typeName, [size, count]]) => `
        <div class="type-item">
            <span class="type-name">${typeName}</span>
            <span class="type-stats">${formatBytes(size)} (${count} allocs)</span>
        </div>
    `).join('');
    
    element.innerHTML = html;
}

/**
 * 渲染最近分配
 */
function renderRecentAllocations(allocations) {
    const element = document.getElementById('recentAllocations');
    if (!element) return;
    
    const recent = allocations.slice(0, 5);
    
    if (recent.length === 0) {
        element.innerHTML = '<p>No recent allocations</p>';
        return;
    }
    
    const html = recent.map(alloc => `
        <div class="allocation-item">
            <span class="alloc-size">${formatBytes(alloc.size)}</span>
            <span class="alloc-type">${alloc.type_name || 'Unknown'}</span>
        </div>
    `).join('');
    
    element.innerHTML = html;
}

/**
 * 渲染性能洞察
 */
function renderPerformanceInsights(stats) {
    const element = document.getElementById('performanceInsights');
    if (!element) return;
    
    const insights = [];
    
    if (stats.active_memory > 1024 * 1024) {
        insights.push('🔍 High memory usage detected');
    }
    
    if (stats.active_allocations > 1000) {
        insights.push('📊 Many active allocations');
    }
    
    if (insights.length === 0) {
        insights.push('✅ Memory usage looks healthy');
    }
    
    const html = insights.map(insight => `<div class="insight-item">${insight}</div>`).join('');
    element.innerHTML = html;
}

/**
 * 初始化标签页导航
 */
function initializeTabs() {
    const tabButtons = document.querySelectorAll('.tab-btn');
    const tabContents = document.querySelectorAll('.tab-content');
    
    tabButtons.forEach(button => {
        button.addEventListener('click', () => {
            const targetTab = button.getAttribute('data-tab');
            
            // 移除所有活动状态
            tabButtons.forEach(btn => btn.classList.remove('active'));
            tabContents.forEach(content => content.classList.remove('active'));
            
            // 激活当前标签
            button.classList.add('active');
            const targetContent = document.getElementById(targetTab);
            if (targetContent) {
                targetContent.classList.add('active');
            }
        });
    });
}

/**
 * 显示错误状态
 */
function showErrorState(error) {
    const container = document.querySelector('.container');
    if (!container) return;
    
    container.innerHTML = `
        <div class="error-state" style="text-align: center; padding: 60px 20px; color: #e74c3c;">
            <h2>❌ Error Loading Data</h2>
            <p>Failed to load memory analysis data: ${error.message}</p>
            <button onclick="location.reload()" style="
                padding: 10px 20px; background: #3498db; color: white;
                border: none; border-radius: 5px; cursor: pointer; margin-top: 20px;
            ">Reload Page</button>
        </div>
    `;
}

/**
 * 格式化字节数
 */
function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

/**
 * MemScope可视化器类
 */
class MemScopeVisualizer {
    constructor(data) {
        this.data = data;
        console.log('🎨 MemScope Visualizer initialized with data:', data);
    }
}

// 全局变量
let globalVisualizer;

// 导出函数供HTML使用
window.initializeMemScopeApp = initializeMemScopeApp;
window.processEmbeddedData = processEmbeddedData;