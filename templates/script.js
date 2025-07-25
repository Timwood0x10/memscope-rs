// MemScope-RS Interactive Visualizations - Dynamic JSON Loading Version
// This version properly handles embedded data

/**
 * 初始化应用程序
 */
function initializeMemScopeApp() {
    console.log('🚀 Initializing MemScope-RS Interactive App...');
    
    try {
        // 首先尝试使用嵌入的数据
        if (typeof EMBEDDED_DATA !== 'undefined' && EMBEDDED_DATA) {
            console.log('✅ Using embedded data');
            processEmbeddedData(EMBEDDED_DATA);
        } else {
            console.log('🔍 No embedded data found, trying to load from MemoryAnalysis directory...');
            loadMemoryAnalysisData();
        }
    } catch (error) {
        console.error('❌ Initialization failed:', error);
        showErrorState(error);
    }
}

/**
 * 从 MemoryAnalysis 目录加载 JSON 数据
 */
async function loadMemoryAnalysisData() {
    const dataFiles = [
        'MemoryAnalysis/snapshot_memory_analysis_memory_analysis.json',
        'MemoryAnalysis/snapshot_memory_analysis_lifetime.json',
        'MemoryAnalysis/snapshot_memory_analysis_security_violations.json',
        'MemoryAnalysis/snapshot_unsafe_ffi.json',
        'MemoryAnalysis/snapshot_memory_analysis_complex_types.json'
    ];
    
    let loadedData = {};
    let hasData = false;
    let availableFiles = [];
    
    for (const file of dataFiles) {
        try {
            const response = await fetch(file);
            if (response.ok) {
                const data = await response.json();
                const dataType = extractDataType(file);
                loadedData[dataType] = data;
                availableFiles.push(file);
                hasData = true;
                console.log(`✅ Loaded ${file}`);
            } else {
                console.log(`⚠️ Could not load ${file}: ${response.status}`);
            }
        } catch (error) {
            console.log(`⚠️ Error loading ${file}:`, error.message);
        }
    }
    
    if (hasData) {
        const consolidatedData = consolidateLoadedData(loadedData);
        updateDataSourceInfo(availableFiles);
        processEmbeddedData(consolidatedData);
    } else {
        console.warn('⚠️ No data files found, showing error state');
        showErrorState(new Error('No memory analysis data available'));
    }
}

/**
 * 从文件名提取数据类型
 */
function extractDataType(filename) {
    if (filename.includes('memory_analysis.json')) return 'memory_analysis';
    if (filename.includes('lifetime')) return 'lifetime';
    if (filename.includes('security_violations')) return 'security_violations';
    if (filename.includes('unsafe_ffi')) return 'unsafe_ffi';
    if (filename.includes('complex_types')) return 'complex_types';
    return 'unknown';
}

/**
 * 整合加载的数据为期望格式
 */
function consolidateLoadedData(loadedData) {
    const consolidated = {
        allocations: [],
        stats: {
            total_allocated: 0,
            total_deallocated: 0,
            peak_memory: 0,
            active_allocations: 0,
            allocation_count: 0,
            deallocation_count: 0
        },
        memory_by_type: {},
        lifecycle_data: null,
        security_violations: [],
        unsafe_ffi_data: null,
        complex_types: []
    };
    
    // 处理内存分析数据
    if (loadedData.memory_analysis) {
        const data = loadedData.memory_analysis;
        if (data.allocations) {
            consolidated.allocations = data.allocations;
            consolidated.stats.allocation_count = data.allocations.length;
            consolidated.stats.active_allocations = data.allocations.filter(a => !a.timestamp_dealloc).length;
            consolidated.stats.total_allocated = data.allocations.reduce((sum, a) => sum + (a.size || 0), 0);
            consolidated.stats.peak_memory = Math.max(...data.allocations.map(a => a.size || 0));
        }
        if (data.stats) {
            Object.assign(consolidated.stats, data.stats);
        }
        if (data.memory_by_type) {
            consolidated.memory_by_type = data.memory_by_type;
        }
    }
    
    // 处理其他数据类型
    if (loadedData.lifetime) {
        consolidated.lifecycle_data = loadedData.lifetime;
    }
    
    if (loadedData.security_violations) {
        consolidated.security_violations = Array.isArray(loadedData.security_violations) 
            ? loadedData.security_violations 
            : [loadedData.security_violations];
    }
    
    if (loadedData.unsafe_ffi) {
        consolidated.unsafe_ffi_data = loadedData.unsafe_ffi;
    }
    
    if (loadedData.complex_types) {
        consolidated.complex_types = Array.isArray(loadedData.complex_types) 
            ? loadedData.complex_types 
            : [loadedData.complex_types];
    }
    
    return consolidated;
}

/**
 * 更新数据源信息显示
 */
function updateDataSourceInfo(availableFiles) {
    const header = document.querySelector('.header');
    if (header && availableFiles.length > 0) {
        const dataInfo = document.createElement('div');
        dataInfo.className = 'data-source-info';
        dataInfo.innerHTML = `
            <div style="margin-top: 10px; font-size: 0.9rem; color: #666;">
                📁 Data loaded from: ${availableFiles.length} files
                <details style="margin-top: 5px;">
                    <summary style="cursor: pointer;">View loaded files</summary>
                    <ul style="margin: 5px 0; padding-left: 20px;">
                        ${availableFiles.map(file => `<li>${file}</li>`).join('')}
                    </ul>
                </details>
            </div>
        `;
        header.appendChild(dataInfo);
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