// Hybrid Dashboard JavaScript Functions
// All interactive functionality for the memory analysis dashboard

// Theme toggle functionality
window.toggleTheme = function() {
    const html = document.documentElement;
    const themeToggle = document.getElementById('theme-toggle');
    
    if (html.getAttribute('data-theme') === 'light') {
        html.setAttribute('data-theme', 'dark');
        if (themeToggle) {
            themeToggle.innerHTML = '☀️ Light Mode';
        }
    } else {
        html.setAttribute('data-theme', 'light');
        if (themeToggle) {
            themeToggle.innerHTML = '🌙 Dark Mode';
        }
    }
};

// Memory map toggle functionality
window.toggleMemoryMap = function() {
    console.log('toggleMemoryMap called'); // Debug log
    const memoryMapSection = document.querySelector('.memory-map-section');
    const toggle = document.getElementById('memory-map-toggle');
    
    console.log('Memory map section found:', memoryMapSection); // Debug log
    console.log('Toggle button found:', toggle); // Debug log
    
    if (memoryMapSection) {
        const isHidden = memoryMapSection.style.display === 'none' || 
                        window.getComputedStyle(memoryMapSection).display === 'none';
        
        if (isHidden) {
            memoryMapSection.style.display = 'block';
            if (toggle) toggle.innerHTML = '🗺️ Hide Memory Map';
            console.log('Memory map shown');
        } else {
            memoryMapSection.style.display = 'none';
            if (toggle) toggle.innerHTML = '🗺️ Show Memory Map';
            console.log('Memory map hidden');
        }
    } else {
        console.error('Memory map section not found!');
    }
};

// Focus attribution functionality - 实现热点分析入口
window.focusAttribution = function(type) {
    console.log('Focusing on ' + type + ' attribution');
    
    // 显示归因分析面板
    showAttributionPanel(type);
    
    // 滚动到变量列表区域
    const variablesSection = document.querySelector('.variables-grid').parentElement;
    if (variablesSection) {
        variablesSection.scrollIntoView({ behavior: 'smooth' });
    }
    
    showToast('🎯 ' + getTypeDisplayName(type) + ' hotspot analysis activated');
};

// 显示归因分析面板
function showAttributionPanel(type) {
    // 移除现有的归因面板
    const existingPanel = document.querySelector('.attribution-panel');
    if (existingPanel) {
        existingPanel.remove();
    }
    
    // 创建归因面板
    const panel = document.createElement('div');
    panel.className = 'attribution-panel';
    panel.innerHTML = getAttributionPanelHTML(type);
    
    // 插入到变量列表之前
    const variablesSection = document.querySelector('.variables-grid').parentElement;
    variablesSection.parentNode.insertBefore(panel, variablesSection);
    
    // 高亮相关的变量卡片
    highlightRelevantVariables(type);
}

// 获取归因面板HTML
function getAttributionPanelHTML(type) {
    const typeInfo = getAttributionTypeInfo(type);
    
    return `
        <div class="section attribution-section" style="border-left: 4px solid ${typeInfo.color};">
            <h3>${typeInfo.icon} ${typeInfo.title} 热点归因分析</h3>
            <div class="attribution-summary">
                <div class="hotspot-indicator">
                    <span class="hotspot-badge" style="background: ${typeInfo.color};">${typeInfo.badge}</span>
                    <span class="hotspot-desc">${typeInfo.description}</span>
                </div>
                <div class="attribution-actions">
                    <button class="btn-secondary" onclick="showTopContributors('${type}')">
                        📊 View Top Contributors
                    </button>
                    <button class="btn-secondary" onclick="generateOptimizationReport('${type}')">
                        💡 Generate Optimization Report
                    </button>
                    <button class="btn-secondary" onclick="closeAttributionPanel()">
                        ✖️ Close Analysis
                    </button>
                </div>
            </div>
            <div class="top-contributors" id="top-contributors-${type}">
                ${getTopContributorsHTML(type)}
            </div>
        </div>
    `;
}

// 获取归因类型信息
function getAttributionTypeInfo(type) {
    const typeMap = {
        'memory': {
            icon: '🧠',
            title: 'Memory',
            color: '#3b82f6',
            badge: 'Memory Hotspot',
            description: 'Identify variables and threads with abnormal memory usage'
        },
        'variables': {
            icon: '📦',
            title: 'Variables',
            color: '#10b981',
            badge: 'Variable Hotspot',
            description: 'Analyze variable allocation and lifecycle patterns'
        },
        'threads': {
            icon: '🧵',
            title: 'Threads',
            color: '#f59e0b',
            badge: 'Thread Hotspot',
            description: 'Identify thread contention and performance bottlenecks'
        },
        'efficiency': {
            icon: '⚡',
            title: 'Efficiency',
            color: '#ef4444',
            badge: 'Efficiency Bottleneck',
            description: 'Locate root causes of system inefficiency'
        }
    };
    return typeMap[type] || typeMap['memory'];
}

// Get type display name
function getTypeDisplayName(type) {
    const nameMap = {
        'memory': 'Memory',
        'variables': 'Variables',
        'threads': 'Threads',
        'efficiency': 'Efficiency'
    };
    return nameMap[type] || type;
}

// Variable drill down functionality - 实现深度检查器
window.drillDown = function(variableId, type) {
    const modal = document.getElementById('variable-modal');
    const modalBody = document.getElementById('modal-body');
    
    if (!modal || !modalBody) return;
    
    // 生成深度检查器内容
    const content = generateInspectorContent(variableId, type);
    modalBody.innerHTML = content;
    modal.style.display = 'block';
    
    // 初始化检查器功能
    initializeInspector(variableId, type);
};

// 生成检查器内容 - 多标签页深度分析
function generateInspectorContent(variableId, type) {
    const isVariable = variableId.includes('var_');
    const isThread = variableId.includes('Thread ') || /thread_\d+/.test(variableId);
    const isTask = variableId.includes('Task ') || /task\d+/.test(variableId);
    
    return `
        <div class="inspector-container">
            <div class="inspector-header">
                <h3>${getInspectorIcon(type)} ${variableId} Deep Inspector</h3>
                <div class="inspector-tabs">
                    ${generateInspectorTabs(isVariable, isThread, isTask)}
                </div>
            </div>
            <div class="inspector-content">
                ${generateInspectorPages(variableId, type, isVariable, isThread, isTask)}
            </div>
        </div>
    `;
}

// 生成检查器标签页
function generateInspectorTabs(isVariable, isThread, isTask) {
    let tabs = '';
    
    if (isVariable) {
        tabs += `
            <button class="inspector-tab active" data-tab="overview">Overview</button>
            <button class="inspector-tab" data-tab="lifecycle">Lifecycle</button>
            <button class="inspector-tab" data-tab="ffi">FFI Passport</button>
            <button class="inspector-tab" data-tab="optimization">Optimization</button>
        `;
    } else if (isThread) {
        tabs += `
            <button class="inspector-tab active" data-tab="performance">Performance</button>
            <button class="inspector-tab" data-tab="tasks">Task List</button>
            <button class="inspector-tab" data-tab="variables">Variables</button>
        `;
    } else if (isTask) {
        tabs += `
            <button class="inspector-tab active" data-tab="overview">Overview</button>
            <button class="inspector-tab" data-tab="variables">Variables</button>
            <button class="inspector-tab" data-tab="optimization">Optimization</button>
        `;
    } else {
        tabs += `<button class="inspector-tab active" data-tab="overview">Overview</button>`;
    }
    
    return tabs;
}

// 生成检查器页面内容
function generateInspectorPages(variableId, type, isVariable, isThread, isTask) {
    let pages = '';
    
    if (isVariable) {
        pages += generateVariableInspectorPages(variableId, type);
    } else if (isThread) {
        pages += generateThreadInspectorPages(variableId);
    } else if (isTask) {
        pages += generateTaskInspectorPages(variableId);
    } else {
        pages += `<div class="inspector-page active" data-page="overview">
            <h4>Basic Information</h4>
            <p>Analyzing ${variableId}...</p>
        </div>`;
    }
    
    return pages;
}

// 生成变量检查器页面
function generateVariableInspectorPages(variableId, type) {
    const rank = Math.floor(Math.random() * 10) + 1;
    
    return `
        <div class="inspector-page active" data-page="overview">
            <h4>📦 Variable Overview</h4>
            ${window.generateMemoryDrillDown(variableId, rank)}
        </div>
        <div class="inspector-page" data-page="lifecycle">
            <h4>🔄 Lifecycle Timeline</h4>
            <div class="lifecycle-timeline">
                <div class="timeline-events">
                    <div class="timeline-event allocated">
                        <span class="event-time">0ms</span>
                        <span class="event-label">🎯 Allocated</span>
                        <span class="event-details">Initial allocation ${Math.floor(Math.random() * 100)}KB</span>
                    </div>
                    <div class="timeline-event active">
                        <span class="event-time">${Math.floor(Math.random() * 500)}ms</span>
                        <span class="event-label">🟢 Activated</span>
                        <span class="event-details">Started active usage</span>
                    </div>
                    <div class="timeline-event shared">
                        <span class="event-time">${Math.floor(Math.random() * 1000)}ms</span>
                        <span class="event-label">🔄 Shared</span>
                        <span class="event-details">Cross-thread access detected</span>
                    </div>
                </div>
                <canvas id="lifecycle-chart-${rank}" width="400" height="120"></canvas>
            </div>
        </div>
        <div class="inspector-page" data-page="ffi">
            <h4>🌉 FFI Border Passport</h4>
            ${generateFFICrossingSection()}
        </div>
        <div class="inspector-page" data-page="optimization">
            <h4>💡 Smart Optimization Suggestions</h4>
            <div class="optimization-recommendations">
                <div class="rec-item priority-high">
                    <span class="rec-priority high">HIGH</span>
                    <span class="rec-text">Consider using memory pools to reduce frequent allocations</span>
                    <span class="rec-impact">Expected to save ${Math.floor(Math.random() * 30 + 20)}% memory</span>
                </div>
                <div class="rec-item priority-medium">
                    <span class="rec-priority medium">MEDIUM</span>
                    <span class="rec-text">Optimize variable lifecycle management</span>
                    <span class="rec-impact">Expected to improve ${Math.floor(Math.random() * 20 + 10)}% performance</span>
                </div>
            </div>
        </div>
    `;
}

// 生成线程检查器页面
function generateThreadInspectorPages(threadId) {
    const threadNum = parseInt(threadId.match(/\d+/)?.[0] || '1');
    
    return `
        <div class="inspector-page active" data-page="performance">
            <h4>📊 Thread Performance Analysis</h4>
            <div class="thread-metrics">
                <div class="metric-grid">
                    <div class="metric-card">
                        <span class="metric-value">${(Math.random() * 50 + 30).toFixed(1)}%</span>
                        <span class="metric-label">CPU Usage</span>
                    </div>
                    <div class="metric-card">
                        <span class="metric-value">${(Math.random() * 200 + 100).toFixed(0)}MB</span>
                        <span class="metric-label">Memory Usage</span>
                    </div>
                    <div class="metric-card">
                        <span class="metric-value">${Math.floor(Math.random() * 1000 + 500)}</span>
                        <span class="metric-label">Context Switches</span>
                    </div>
                </div>
                <canvas id="thread-perf-chart-${threadNum}" width="400" height="150"></canvas>
            </div>
        </div>
        <div class="inspector-page" data-page="tasks">
            <h4>📋 Task List</h4>
            <div class="task-list">
                ${generateTaskListForThread(threadNum)}
            </div>
        </div>
        <div class="inspector-page" data-page="variables">
            <h4>📦 Variable List</h4>
            <div class="variable-search">
                <input type="text" placeholder="Search variables..." onkeyup="filterVariables(this.value)">
            </div>
            <div class="variables-table">
                ${generateVariableTableForThread(threadNum)}
            </div>
        </div>
    `;
}

// 生成任务检查器页面
function generateTaskInspectorPages(taskId) {
    const taskNum = parseInt(taskId.match(/\d+/)?.[0] || '1');
    
    return `
        <div class="inspector-page active" data-page="overview">
            <h4>📋 Task Overview</h4>
            <div class="task-overview">
                <div class="task-basic-info">
                    <p><strong>Task ID:</strong> ${taskNum}</p>
                    <p><strong>Execution Status:</strong> <span class="status-active">Running</span></p>
                    <p><strong>Priority:</strong> ${Math.floor(Math.random() * 10 + 1)}</p>
                    <p><strong>Execution Time:</strong> ${Math.floor(Math.random() * 1000 + 100)}ms</p>
                </div>
                <canvas id="task-io-chart-${taskNum}" width="300" height="100"></canvas>
            </div>
        </div>
        <div class="inspector-page" data-page="variables">
            <h4>📦 Associated Variables</h4>
            <div class="task-variables">
                ${generateVariableTableForTask(taskNum)}
            </div>
        </div>
        <div class="inspector-page" data-page="optimization">
            <h4>🚀 Task Optimization Suggestions</h4>
            <div class="task-optimization">
                <div class="rec-item">
                    <span class="rec-priority medium">MEDIUM</span>
                    <span class="rec-text">Consider async I/O operations to reduce blocking</span>
                </div>
                <div class="rec-item">
                    <span class="rec-priority low">LOW</span>
                    <span class="rec-text">Optimize task scheduling strategy</span>
                </div>
            </div>
        </div>
    `;
}

// Memory drill down generator
window.generateMemoryDrillDown = function(variableId, rank) {
    const memoryUsage = 50 + rank * 10;
    const allocations = 100 + rank * 20;
    const deallocations = 80 + rank * 15;
    
    return '<div class="drill-down-content">' +
               '<h4>🧠 Memory Analysis: ' + variableId + '</h4>' +
               '<div class="perf-metrics">' +
                   '<div class="metric-row">' +
                       '<span>Memory Usage:</span>' +
                       '<span>' + memoryUsage + 'MB</span>' +
                   '</div>' +
                   '<div class="metric-row">' +
                       '<span>Allocations:</span>' +
                       '<span>' + allocations + '</span>' +
                   '</div>' +
                   '<div class="metric-row">' +
                       '<span>Deallocations:</span>' +
                       '<span>' + deallocations + '</span>' +
                   '</div>' +
               '</div>' +
               '<h4>💡 Memory Recommendations</h4>' +
               '<div class="recommendations">' +
                   '<div class="rec-item">' +
                       '<span class="rec-priority high">HIGH</span>' +
                       '<span class="rec-text">Consider implementing memory pooling</span>' +
                   '</div>' +
                   '<div class="rec-item">' +
                       '<span class="rec-priority medium">MEDIUM</span>' +
                       '<span class="rec-text">Monitor with memory passport tracking</span>' +
                   '</div>' +
               '</div>' +
               '<canvas id="allocation-timeline-' + rank + '" width="300" height="80"></canvas>' +
           '</div>';
};

// CPU drill down generator
window.generateCpuDrillDown = function(variableId, rank) {
    const cpuUsage = 30 + rank * 12;
    const taskQueue = 3 + rank;
    const contextSwitches = 150 + rank * 50;
    
    return '<div class="drill-down-content">' +
               '<h4>🔄 Thread Performance Analysis</h4>' +
               '<div class="perf-metrics">' +
                   '<div class="metric-row">' +
                       '<span>CPU Usage:</span>' +
                       '<span>' + cpuUsage + '%</span>' +
                   '</div>' +
                   '<div class="metric-row">' +
                       '<span>Task Queue:</span>' +
                       '<span>' + taskQueue + ' tasks</span>' +
                   '</div>' +
                   '<div class="metric-row">' +
                       '<span>Context Switches:</span>' +
                       '<span>' + contextSwitches + '/sec</span>' +
                   '</div>' +
               '</div>' +
               '<h4>💡 Optimization Suggestions</h4>' +
               '<div class="recommendations">' +
                   '<div class="rec-item">' +
                       '<span class="rec-priority medium">MEDIUM</span>' +
                       '<span class="rec-text">Implement work-stealing queue</span>' +
                   '</div>' +
                   '<div class="rec-item">' +
                       '<span class="rec-priority low">LOW</span>' +
                       '<span class="rec-text">Consider thread affinity optimization</span>' +
                   '</div>' +
               '</div>' +
           '</div>';
};

// I/O drill down generator
window.generateIoDrillDown = function(variableId, rank) {
    const peakOps = 200 + rank * 50;
    const avgLatency = 15 + rank * 5;
    const blockingTime = 30 + rank * 10;
    
    return '<div class="drill-down-content">' +
               '<h4>📊 I/O Pattern Analysis</h4>' +
               '<div class="io-pattern">' +
                   '<div class="pattern-row">' +
                       '<span>Peak Operations:</span>' +
                       '<span>' + peakOps + ' ops/sec</span>' +
                   '</div>' +
                   '<div class="pattern-row">' +
                       '<span>Average Latency:</span>' +
                       '<span>' + avgLatency + 'ms</span>' +
                   '</div>' +
                   '<div class="pattern-row">' +
                       '<span>Blocking Time:</span>' +
                       '<span>' + blockingTime + '% of period</span>' +
                   '</div>' +
               '</div>' +
               '<h4>💡 Performance Improvements</h4>' +
               '<div class="recommendations">' +
                   '<div class="rec-item">' +
                       '<span class="rec-priority high">HIGH</span>' +
                       '<span class="rec-text">Implement connection pooling</span>' +
                   '</div>' +
                   '<div class="rec-item">' +
                       '<span class="rec-priority medium">MEDIUM</span>' +
                       '<span class="rec-text">Add async buffering</span>' +
                   '</div>' +
               '</div>' +
           '</div>';
};

// FFI Crossing section generator
function generateFFICrossingSection() {
    return '<div class="ffi-crossing-section">' +
               '<h4>🌉 FFI Boundary Crossing Analysis</h4>' +
               '<div class="ffi-swimlane">' +
                   '<div class="ffi-lane rust-lane">' +
                       '<div class="lane-label">🦀 Rust Side</div>' +
                       '<div class="ffi-event">Variable created</div>' +
                   '</div>' +
                   '<div class="ffi-boundary">' +
                       '<div class="boundary-arrow">→</div>' +
                       '<div class="boundary-label">FFI Call</div>' +
                   '</div>' +
                   '<div class="ffi-lane c-lane">' +
                       '<div class="lane-label">🔧 C Side</div>' +
                       '<div class="ffi-event">Pointer passed</div>' +
                   '</div>' +
                   '<div class="ffi-boundary">' +
                       '<div class="boundary-arrow">←</div>' +
                       '<div class="boundary-label">Return</div>' +
                   '</div>' +
                   '<div class="ffi-lane rust-lane">' +
                       '<div class="lane-label">🦀 Rust Side</div>' +
                       '<div class="ffi-event">Memory managed</div>' +
                   '</div>' +
               '</div>' +
               '<div class="ffi-warning">' +
                   '<span class="warning-icon">⚠️</span>' +
                   '<span class="warning-text">Memory may have been modified on C side - verify ownership</span>' +
               '</div>' +
           '</div>';
}

// Timeline chart generator
function generateTimelineChart(variableId, rank) {
    const canvas = document.getElementById('allocation-timeline-' + rank);
    if (!canvas) return;
    
    const ctx = canvas.getContext('2d');
    canvas.width = 300;
    canvas.height = 80;
    
    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    // Generate sample data
    const dataPoints = 20;
    const values = [];
    for (let i = 0; i < dataPoints; i++) {
        values.push(Math.random() * 50 + 10);
    }
    
    // Draw chart
    ctx.strokeStyle = '#3b82f6';
    ctx.lineWidth = 2;
    ctx.beginPath();
    
    for (let i = 0; i < values.length; i++) {
        const x = (i / (values.length - 1)) * canvas.width;
        const y = canvas.height - (values[i] / 60) * canvas.height;
        
        if (i === 0) {
            ctx.moveTo(x, y);
        } else {
            ctx.lineTo(x, y);
        }
        
        // Draw data points
        ctx.fillStyle = '#3b82f6';
        ctx.beginPath();
        ctx.arc(x, y, 3, 0, 2 * Math.PI);
        ctx.fill();
        
        // Value labels
        ctx.fillStyle = '#374151';
        ctx.font = '10px sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText(values[i].toFixed(0) + 'MB', x, y - 8);
    }
    
    ctx.stroke();
}

// Modal close functionality
function closeModal() {
    const modal = document.getElementById('variable-modal');
    if (modal) {
        modal.style.display = 'none';
    }
}

// Toast notification system
function showToast(message) {
    // Remove existing toast
    const existingToast = document.querySelector('.toast');
    if (existingToast) {
        existingToast.remove();
    }
    
    // Create new toast
    const toast = document.createElement('div');
    toast.className = 'toast';
    toast.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        background: var(--primary);
        color: white;
        padding: 12px 24px;
        border-radius: 8px;
        z-index: 10000;
        animation: slideIn 0.3s ease;
    `;
    toast.textContent = message;
    
    document.body.appendChild(toast);
    
    // Remove after 3 seconds
    setTimeout(function() {
        toast.remove();
    }, 3000);
}

// Add CSS for toast animation
const style = document.createElement('style');
style.textContent = `
    @keyframes slideIn {
        from { transform: translateX(100%); opacity: 0; }
        to { transform: translateX(0); opacity: 1; }
    }
`;
document.head.appendChild(style);

// Initialize theme on page load
document.addEventListener('DOMContentLoaded', function() {
    // Set initial theme
    if (!document.documentElement.getAttribute('data-theme')) {
        document.documentElement.setAttribute('data-theme', 'light');
    }
    
    // Initialize memory map as hidden
    const memoryMapSection = document.querySelector('.memory-map-section');
    if (memoryMapSection) {
        memoryMapSection.style.display = 'none';
    }
    
    // Close modal when clicking outside
    const modal = document.getElementById('variable-modal');
    if (modal) {
        modal.addEventListener('click', function(e) {
            if (e.target === modal) {
                closeModal();
            }
        });
    }
});

// 添加归因分析相关的辅助函数
function getTopContributorsHTML(type) {
    // 模拟热点贡献者数据
    const contributors = generateMockContributors(type);
    
    let html = '<div class="contributors-list">';
    contributors.forEach((item, index) => {
        html += `
            <div class="contributor-item" onclick="window.drillDown('${item.id}', '${type}')">
                <span class="contributor-rank">#${index + 1}</span>
                <span class="contributor-name">${item.name}</span>
                <span class="contributor-impact">${item.impact}</span>
                <span class="contributor-action">🔍 Deep Analysis</span>
            </div>
        `;
    });
    html += '</div>';
    
    return html;
}

function generateMockContributors(type) {
    const data = window.DASHBOARD_DATA?.variables || [];
    return data.slice(0, 5).map((item, index) => ({
        id: item.name || `${type}_item_${index}`,
        name: item.name || `${type}_${index}`,
        impact: `${Math.floor(Math.random() * 50 + 30)}% contribution`
    }));
}

function highlightRelevantVariables(type) {
    const variableCards = document.querySelectorAll('.variable-card');
    variableCards.forEach(card => {
        card.style.opacity = '0.6';
        card.style.transform = 'scale(0.98)';
    });
    
    // 高亮前几个作为示例
    for (let i = 0; i < Math.min(3, variableCards.length); i++) {
        const card = variableCards[i];
        card.style.opacity = '1';
        card.style.transform = 'scale(1.02)';
        card.style.boxShadow = '0 4px 12px rgba(59, 130, 246, 0.3)';
        card.style.border = '2px solid #3b82f6';
    }
}

function showTopContributors(type) {
    const container = document.getElementById(`top-contributors-${type}`);
    if (container) {
        container.innerHTML = getTopContributorsHTML(type);
        container.style.display = 'block';
    }
}

function generateOptimizationReport(type) {
    showToast(`🚀 Generating ${getTypeDisplayName(type)} optimization report...`);
    
    setTimeout(() => {
        const report = `
            <div class="optimization-report">
                <h4>📊 ${getTypeDisplayName(type)} Optimization Report</h4>
                <div class="report-summary">
                    <p>✅ Found ${Math.floor(Math.random() * 5 + 3)} optimization opportunities</p>
                    <p>🎯 Expected performance improvement ${Math.floor(Math.random() * 30 + 20)}%</p>
                    <p>💾 Expected memory savings ${Math.floor(Math.random() * 20 + 10)}%</p>
                </div>
            </div>
        `;
        
        const container = document.getElementById(`top-contributors-${type}`);
        if (container) {
            container.innerHTML = report;
        }
        
        showToast(`✅ ${getTypeDisplayName(type)} optimization report generated`);
    }, 1500);
}

function closeAttributionPanel() {
    const panel = document.querySelector('.attribution-panel');
    if (panel) {
        panel.remove();
    }
    
    // 恢复所有变量卡片样式
    const variableCards = document.querySelectorAll('.variable-card');
    variableCards.forEach(card => {
        card.style.opacity = '1';
        card.style.transform = 'scale(1)';
        card.style.boxShadow = '';
        card.style.border = '';
    });
}

function getInspectorIcon(type) {
    const iconMap = {
        'memory': '🧠',
        'cpu': '⚡',
        'io': '💾',
        'thread': '🧵',
        'task': '📋'
    };
    return iconMap[type] || '🔍';
}

function generateTaskListForThread(threadNum) {
    let html = '<div class="task-items">';
    for (let i = 1; i <= 3; i++) {
        const taskId = threadNum * 3 + i;
        html += `
            <div class="task-item" onclick="window.drillDown('Task ${taskId}', 'task')">
                <span class="task-id">Task ${taskId}</span>
                <span class="task-status">Running</span>
                <span class="task-memory">${Math.floor(Math.random() * 100 + 50)}KB</span>
            </div>
        `;
    }
    html += '</div>';
    return html;
}

function generateVariableTableForThread(threadNum) {
    let html = '<div class="variables-table-content">';
    for (let i = 0; i < 5; i++) {
        const varName = `thread_${threadNum}_var_${i}`;
        html += `
            <div class="var-row" onclick="window.drillDown('${varName}', 'memory')">
                <span class="var-name">${varName}</span>
                <span class="var-size">${Math.floor(Math.random() * 200 + 50)}KB</span>
                <span class="var-status">Active</span>
            </div>
        `;
    }
    html += '</div>';
    return html;
}

function generateVariableTableForTask(taskNum) {
    let html = '<div class="task-variables-content">';
    for (let i = 0; i < 3; i++) {
        const varName = `task_${taskNum}_var_${i}`;
        html += `
            <div class="var-row" onclick="window.drillDown('${varName}', 'memory')">
                <span class="var-name">${varName}</span>
                <span class="var-size">${Math.floor(Math.random() * 150 + 30)}KB</span>
                <span class="var-lifecycle">Allocated</span>
            </div>
        `;
    }
    html += '</div>';
    return html;
}

// 初始化检查器功能
function initializeInspector(variableId, type) {
    // 绑定标签页切换事件
    const tabs = document.querySelectorAll('.inspector-tab');
    const pages = document.querySelectorAll('.inspector-page');
    
    tabs.forEach(tab => {
        tab.addEventListener('click', function() {
            const targetTab = this.getAttribute('data-tab');
            
            // 切换标签页样式
            tabs.forEach(t => t.classList.remove('active'));
            this.classList.add('active');
            
            // 切换页面内容
            pages.forEach(page => {
                page.classList.remove('active');
                if (page.getAttribute('data-page') === targetTab) {
                    page.classList.add('active');
                }
            });
        });
    });
    
    // 生成相关图表
    setTimeout(() => {
        generateInspectorCharts(variableId, type);
    }, 100);
}

function generateInspectorCharts(variableId, type) {
    // 这里可以添加图表生成逻辑
    console.log('Generating charts for inspector:', variableId, type);
}

function filterVariables(searchTerm) {
    const rows = document.querySelectorAll('.var-row');
    rows.forEach(row => {
        const varName = row.querySelector('.var-name').textContent.toLowerCase();
        if (varName.includes(searchTerm.toLowerCase())) {
            row.style.display = 'flex';
        } else {
            row.style.display = 'none';
        }
    });
}

console.log('🎯 Attribution Analysis Dashboard JavaScript loaded');
console.log('🔍 Ready for 3-click root cause discovery');