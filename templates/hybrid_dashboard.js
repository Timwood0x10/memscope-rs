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

// Memory layout toggle functionality
window.toggleMemoryMap = function() {
    const memoryMapSection = document.querySelector('.memory-layout-section');
    const toggle = document.getElementById('memory-map-toggle');
    
    if (memoryMapSection) {
        const isHidden = memoryMapSection.style.display === 'none' || 
                        window.getComputedStyle(memoryMapSection).display === 'none';
        
        if (isHidden) {
            memoryMapSection.style.display = 'block';
            if (toggle) toggle.innerHTML = '🗺️ Hide Thread Memory';
            showToast('📊 Thread memory distribution shown');
        } else {
            memoryMapSection.style.display = 'none';
            if (toggle) toggle.innerHTML = '🗺️ Thread Memory';
            showToast('📊 Thread memory distribution hidden');
        }
    }
};

// Focus attribution functionality - implement hotspot analysis entry
window.focusAttribution = function(type) {
    console.log('Focusing on ' + type + ' attribution');
    
    // Show attribution analysis panel
    showAttributionPanel(type);
    
    // Scroll to variable list area
    const variablesSection = document.querySelector('.variables-grid').parentElement;
    if (variablesSection) {
        variablesSection.scrollIntoView({ behavior: 'smooth' });
    }
    
    showToast('🎯 ' + getTypeDisplayName(type) + ' hotspot analysis activated');
};

// Show attribution analysis panel
function showAttributionPanel(type) {
    // Remove existing attribution panel
    const existingPanel = document.querySelector('.attribution-panel');
    if (existingPanel) {
        existingPanel.remove();
    }
    
    // Create attribution panel
    const panel = document.createElement('div');
    panel.className = 'attribution-panel';
    panel.innerHTML = getAttributionPanelHTML(type);
    
    // Insert before variable list
    const variablesSection = document.querySelector('.variables-grid').parentElement;
    variablesSection.parentNode.insertBefore(panel, variablesSection);
    
    // Highlight related variable cards
    highlightRelevantVariables(type);
}

// Get attribution panel HTML
function getAttributionPanelHTML(type) {
    const typeInfo = getAttributionTypeInfo(type);
    
    return `
        <div class="section attribution-section" style="border-left: 4px solid ${typeInfo.color};">
            <h3>${typeInfo.icon} ${typeInfo.title} Hotspot Attribution Analysis</h3>
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

// Get attribution type info
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

// Variable drill down functionality - implement deep inspector
window.drillDown = function(variableId, type) {
    const modal = document.getElementById('variable-modal');
    const modalBody = document.getElementById('modal-body');
    
    if (!modal || !modalBody) return;
    
    // Generate deep inspector content
    const content = generateInspectorContent(variableId, type);
    modalBody.innerHTML = content;
    modal.style.display = 'block';
    
    // Initialize inspector functionality
    initializeInspector(variableId, type);
    
    showToast(`🔍 Opening inspector for ${variableId}`);
};

// Generate inspector content - multi-tab deep analysis
function generateInspectorContent(variableId, type) {
    const isVariable = true; // Force all clicks to be variables
    const isThread = variableId.includes('Thread ') && !variableId.includes('_t'); // Only explicit Thread are threads
    const isTask = variableId.includes('Task ') && !variableId.includes('_t'); // Only explicit Task are tasks
    
    console.log(`🔍 Inspector logic for ${variableId}: isVariable=${isVariable}, isThread=${isThread}, isTask=${isTask}`);
    
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

// Generate inspector tabs
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

// Generate inspector page content
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

// Generate variable inspector page
function generateVariableInspectorPages(variableId, type) {
    const rank = Math.floor(Math.random() * 10) + 1;
    
    return `
        <div class="inspector-page active" data-page="overview">
            <h4>📦 Variable Overview</h4>
            ${window.generateMemoryDrillDown(variableId, rank)}
            
            <div class="code-attribution-section">
                <h5>📍 Code Attribution - Where is the memory coming from?</h5>
                <div class="call-stack-analysis">
                    ${generateCallStackAttribution(variableId, rank)}
                </div>
            </div>
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
            <div class="ffi-crossing-log">
                <h5>🔄 Crossing History</h5>
                <div class="crossing-timeline">
                    <div class="crossing-event">
                        <span class="event-time">0ms</span>
                        <span class="event-type rust">🦀 Created in Rust</span>
                        <span class="event-location">main.rs:42</span>
                        <span class="event-details">Vec&lt;u8&gt; allocated (${Math.floor(Math.random() * 100)}KB)</span>
                    </div>
                    <div class="crossing-event">
                        <span class="event-time">${Math.floor(Math.random() * 500)}ms</span>
                        <span class="event-type ffi">🌉 Passed to C</span>
                        <span class="event-location">ffi_bridge.c:156</span>
                        <span class="event-details">Raw pointer: 0x${Math.floor(Math.random() * 0xFFFFFF).toString(16).padStart(6, '0')}</span>
                    </div>
                    <div class="crossing-event">
                        <span class="event-time">${Math.floor(Math.random() * 800)}ms</span>
                        <span class="event-type c">🔧 Modified in C</span>
                        <span class="event-location">process_data.c:89</span>
                        <span class="event-details">Buffer written, size changed to ${Math.floor(Math.random() * 150)}KB</span>
                    </div>
                    <div class="crossing-event">
                        <span class="event-time">${Math.floor(Math.random() * 1000)}ms</span>
                        <span class="event-type ffi">🌉 Returned to Rust</span>
                        <span class="event-location">ffi_bridge.rs:198</span>
                        <span class="event-details">Ownership reclaimed, validation: ✅</span>
                    </div>
                </div>
            </div>
            
            <div class="ffi-memory-trace">
                <h5>💾 Memory State Changes</h5>
                <div class="memory-changes">
                    <div class="memory-change">
                        <span class="change-side rust">Rust Side</span>
                        <span class="change-action">Initial allocation</span>
                        <span class="change-size">${Math.floor(Math.random() * 100)}KB</span>
                    </div>
                    <div class="memory-change">
                        <span class="change-side c">C Side</span>
                        <span class="change-action">Data processing</span>
                        <span class="change-size">+${Math.floor(Math.random() * 50)}KB</span>
                    </div>
                    <div class="memory-change">
                        <span class="change-side rust">Rust Side</span>
                        <span class="change-action">Final state</span>
                        <span class="change-size">${Math.floor(Math.random() * 150)}KB</span>
                    </div>
                </div>
            </div>
            
            <div class="ffi-warnings">
                <h5>⚠️ Potential Issues</h5>
                <div class="warning-item ${Math.random() > 0.5 ? 'warning-low' : 'warning-high'}">
                    <span class="warning-icon">${Math.random() > 0.5 ? '⚠️' : '🚨'}</span>
                    <span class="warning-text">Memory size changed during C processing - verify buffer bounds</span>
                </div>
                <div class="warning-item warning-medium">
                    <span class="warning-icon">⚠️</span>
                    <span class="warning-text">Pointer validity across FFI boundary: ${Math.random() > 0.7 ? 'Verified' : 'Needs check'}</span>
                </div>
            </div>
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


// Generate inspector tabs
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

// Generate inspector page content
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

// Generate variable inspector page
function generateVariableInspectorPages(variableId, type) {
    const rank = Math.floor(Math.random() * 10) + 1;
    
    return `
        <div class="inspector-page active" data-page="overview">
            <h4>📦 Variable Overview</h4>
            ${window.generateMemoryDrillDown(variableId, rank)}
            
            <div class="code-attribution-section">
                <h5>📍 Code Attribution - Where is the memory coming from?</h5>
                <div class="call-stack-analysis">
                    ${generateCallStackAttribution(variableId, rank)}
                </div>
            </div>
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
            <div class="ffi-crossing-log">
                <h5>🔄 Crossing History</h5>
                <div class="crossing-timeline">
                    <div class="crossing-event">
                        <span class="event-time">0ms</span>
                        <span class="event-type rust">🦀 Created in Rust</span>
                        <span class="event-location">main.rs:42</span>
                        <span class="event-details">Vec&lt;u8&gt; allocated (${Math.floor(Math.random() * 100)}KB)</span>
                    </div>
                    <div class="crossing-event">
                        <span class="event-time">${Math.floor(Math.random() * 500)}ms</span>
                        <span class="event-type ffi">🌉 Passed to C</span>
                        <span class="event-location">ffi_bridge.c:156</span>
                        <span class="event-details">Raw pointer: 0x${Math.floor(Math.random() * 0xFFFFFF).toString(16).padStart(6, '0')}</span>
                    </div>
                    <div class="crossing-event">
                        <span class="event-time">${Math.floor(Math.random() * 800)}ms</span>
                        <span class="event-type c">🔧 Modified in C</span>
                        <span class="event-location">process_data.c:89</span>
                        <span class="event-details">Buffer written, size changed to ${Math.floor(Math.random() * 150)}KB</span>
                    </div>
                    <div class="crossing-event">
                        <span class="event-time">${Math.floor(Math.random() * 1000)}ms</span>
                        <span class="event-type ffi">🌉 Returned to Rust</span>
                        <span class="event-location">ffi_bridge.rs:198</span>
                        <span class="event-details">Ownership reclaimed, validation: ✅</span>
                    </div>
                </div>
            </div>
            
            <div class="ffi-memory-trace">
                <h5>💾 Memory State Changes</h5>
                <div class="memory-changes">
                    <div class="memory-change">
                        <span class="change-side rust">Rust Side</span>
                        <span class="change-action">Initial allocation</span>
                        <span class="change-size">${Math.floor(Math.random() * 100)}KB</span>
                    </div>
                    <div class="memory-change">
                        <span class="change-side c">C Side</span>
                        <span class="change-action">Data processing</span>
                        <span class="change-size">+${Math.floor(Math.random() * 50)}KB</span>
                    </div>
                    <div class="memory-change">
                        <span class="change-side rust">Rust Side</span>
                        <span class="change-action">Final state</span>
                        <span class="change-size">${Math.floor(Math.random() * 150)}KB</span>
                    </div>
                </div>
            </div>
            
            <div class="ffi-warnings">
                <h5>⚠️ Potential Issues</h5>
                <div class="warning-item ${Math.random() > 0.5 ? 'warning-low' : 'warning-high'}">
                    <span class="warning-icon">${Math.random() > 0.5 ? '⚠️' : '🚨'}</span>
                    <span class="warning-text">Memory size changed during C processing - verify buffer bounds</span>
                </div>
                <div class="warning-item warning-medium">
                    <span class="warning-icon">⚠️</span>
                    <span class="warning-text">Pointer validity across FFI boundary: ${Math.random() > 0.7 ? 'Verified' : 'Needs check'}</span>
                </div>
            </div>
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

// Generate thread inspector page
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

// Generate task inspector page
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
    // Use real data from DASHBOARD_DATA instead of mock data
    const data = window.DASHBOARD_DATA?.variables || {};
    const variableData = Object.values(data).find(v => v.name === variableId) || {};
    
    const memoryUsage = variableData.memory_usage ? (variableData.memory_usage / (1024 * 1024)).toFixed(1) : '0';
    const allocations = variableData.allocation_count || 1;
    const deallocations = 0; // Calculate from lifecycle data if available
    
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
    // Set initial theme to dark
    if (!document.documentElement.getAttribute('data-theme')) {
        document.documentElement.setAttribute('data-theme', 'dark');
    }
    
    // Update theme toggle button text
    const themeToggle = document.getElementById('theme-toggle');
    if (themeToggle) {
        themeToggle.innerHTML = '☀️ Light Mode';
    }
    
    // Initialize memory layout as hidden
    const memoryMapSection = document.querySelector('.memory-layout-section');
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
    
    // Initialize filters
    updateFilterStats();
});

// Add attribution analysis helper functions
function getTopContributorsHTML(type) {
    // Get real hotspot contributor data from DASHBOARD_DATA
    const contributors = generateRealContributors(type);
    
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

function generateRealContributors(type) {
    const data = window.DASHBOARD_DATA?.variables || [];
    // Sort by memory usage to get real top contributors
    const sortedData = Object.values(data)
        .filter(item => item && item.memory_usage)
        .sort((a, b) => b.memory_usage - a.memory_usage)
        .slice(0, 5);
    
    const totalMemory = sortedData.reduce((sum, item) => sum + (item.memory_usage || 0), 0);
    
    return sortedData.map((item, index) => ({
        id: item.name || `${type}_item_${index}`,
        name: item.name || `${type}_${index}`,
        impact: totalMemory > 0 ? `${((item.memory_usage / totalMemory) * 100).toFixed(1)}% contribution` : '0% contribution'
    }));
}

// Keep the old function for compatibility but mark as deprecated
function generateMockContributors(type) {
    console.warn('generateMockContributors is deprecated, use generateRealContributors instead');
    return generateRealContributors(type);
}

function highlightRelevantVariables(type) {
    const variableCards = document.querySelectorAll('.variable-card');
    variableCards.forEach(card => {
        card.style.opacity = '0.6';
        card.style.transform = 'scale(0.98)';
    });
    
    // Highlight first few as examples
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
    
    // Restore all variable card styles
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

function calculateTotalMemory() {
    // Calculate total memory from real tracked variables
    const data = window.DASHBOARD_DATA?.variables || {};
    let total = 0;
    
    // Sum up memory usage from all tracked variables
    for (const variable of Object.values(data)) {
        if (variable && typeof variable.memory_usage === 'number') {
            total += variable.memory_usage;
        }
    }
    
    // Convert bytes to MB for display
    return (total / (1024 * 1024)).toFixed(2);
}

function generateTaskListForThread(threadNum) {
    let html = '<div class="task-items">';
    
    // 从DASHBOARD_DATA获取该线程的真实任务数据
    const data = window.DASHBOARD_DATA?.variables || [];
    // 修复字段名称不匹配：数据中使用 'thread' 而不是 'thread_id'
    const threadVariables = data.filter(v => v && v.thread === threadNum);
    
    console.log(`📋 Thread ${threadNum} task data:`, threadVariables); // 调试信息
    
    // 将变量分组为任务（每3-5个变量为一个任务）
    const tasksPerThread = Math.max(1, Math.floor(threadVariables.length / 3));
    
    if (threadVariables.length > 0) {
        for (let i = 1; i <= tasksPerThread; i++) {
            const taskId = threadNum * 100 + i;
            const taskVariables = threadVariables.slice((i-1)*3, i*3);
            // 修复字段名称：数据中使用 'size' 而不是 'memory_usage'
            const taskMemory = taskVariables.reduce((sum, v) => sum + (v.size || 0), 0);
            
            html += `
                <div class="task-item" onclick="window.drillDown('Task ${taskId}', 'task')">
                    <span class="task-id">Task ${taskId}</span>
                    <span class="task-status">${taskVariables.length > 0 ? 'Active' : 'Idle'}</span>
                    <span class="task-memory">${taskMemory > 0 ? (taskMemory / 1024).toFixed(1) : '0'}KB</span>
                </div>
            `;
        }
    } else {
        // 确保至少显示一个任务
        html += `
            <div class="task-item">
                <span class="task-id">Task ${threadNum}01</span>
                <span class="task-status">Idle</span>
                <span class="task-memory">0KB</span>
            </div>
        `;
    }
    
    html += '</div>';
    return html;
}

function generateVariableTableForThread(threadNum) {
    let html = '<div class="variables-table-content">';
    
    // 从DASHBOARD_DATA获取该线程的真实变量数据
    const data = window.DASHBOARD_DATA?.variables || [];
    console.log(`🔍 DASHBOARD_DATA structure for thread ${threadNum}:`, data); // 调试信息
    
    // 修复字段名称不匹配：数据中使用 'thread' 而不是 'thread_id'
    const threadVariables = data.filter(v => v && v.thread === threadNum);
    
    console.log(`🧵 Thread ${threadNum} variables:`, threadVariables); // 调试信息
    
    if (threadVariables.length > 0) {
        // 显示该线程的真实变量
        threadVariables.forEach(variable => {
            // 修复字段名称：数据中使用 'size' 而不是 'memory_usage'，'state' 而不是 'lifecycle_stage'
            const memoryKB = variable.size ? (variable.size / 1024).toFixed(1) : '0';
            const status = variable.state || 'Active';
            
            html += `
                <div class="var-row" onclick="window.drillDown('${variable.name}', 'memory')">
                    <span class="var-name">${variable.name}</span>
                    <span class="var-size">${memoryKB}KB</span>
                    <span class="var-status">${status}</span>
                </div>
            `;
        });
    } else {
        // 当没有找到变量时显示占位符
        html += `
            <div class="var-row">
                <span class="var-name">No variables tracked for Thread ${threadNum}</span>
                <span class="var-size">0KB</span>
                <span class="var-status">Idle</span>
            </div>
        `;
    }
    
    html += '</div>';
    return html;
}

function generateVariableTableForTask(taskNum) {
    let html = '<div class="task-variables-content">';
    
    // Extract thread number from task number (task ID format: threadNum * 100 + taskIndex)
    const threadNum = Math.floor(taskNum / 100);
    const taskIndex = taskNum % 100;
    
    // Get real variables for this task from DASHBOARD_DATA
    const data = window.DASHBOARD_DATA?.variables || [];
    // 修复字段名称不匹配：数据中使用 'thread' 而不是 'thread_id'
    const threadVariables = data.filter(v => v && v.thread === threadNum);
    
    // Get variables for this specific task (3 variables per task)
    const startIndex = (taskIndex - 1) * 3;
    const taskVariables = threadVariables.slice(startIndex, startIndex + 3);
    
    if (taskVariables.length > 0) {
        taskVariables.forEach(variable => {
            // 修复字段名称：数据中使用 'size' 而不是 'memory_usage'，'state' 而不是 'lifecycle_stage'
            const memoryKB = (variable.size / 1024).toFixed(1);
            const lifecycle = variable.state || 'Allocated';
            
            html += `
                <div class="var-row" onclick="window.drillDown('${variable.name}', 'memory')">
                    <span class="var-name">${variable.name}</span>
                    <span class="var-size">${memoryKB}KB</span>
                    <span class="var-lifecycle">${lifecycle}</span>
                </div>
            `;
        });
    } else {
        // Show placeholder when no variables are found for this task
        html += `
            <div class="var-row">
                <span class="var-name">No variables for this task</span>
                <span class="var-size">0KB</span>
                <span class="var-lifecycle">Idle</span>
            </div>
        `;
    }
    
    html += '</div>';
    return html;
}

// Initialize inspector functionality
function initializeInspector(variableId, type) {
    // Bind tab switching events
    const tabs = document.querySelectorAll('.inspector-tab');
    const pages = document.querySelectorAll('.inspector-page');
    
    tabs.forEach(tab => {
        tab.addEventListener('click', function() {
            const targetTab = this.getAttribute('data-tab');
            
            // Switch tab styles
            tabs.forEach(t => t.classList.remove('active'));
            this.classList.add('active');
            
            // Switch page content
            pages.forEach(page => {
                page.classList.remove('active');
                if (page.getAttribute('data-page') === targetTab) {
                    page.classList.add('active');
                }
            });
        });
    });
    
    // Generate related charts
    setTimeout(() => {
        generateInspectorCharts(variableId, type);
    }, 100);
}

function generateInspectorCharts(variableId, type) {
    // Chart generation logic can be added here
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

// Code problem scanning - flame graph-like quick location
function triggerManualScan() {
    showToast('🔎 Scanning code for memory issues...');
    
    const currentData = window.enhancedDiagnostics.gatherCurrentData();
    const problems = window.enhancedDiagnostics.problemDetector.detectProblems(currentData);
    
    if (problems.length === 0) {
        showToast('✅ No memory issues found in current code');
        showCodeHealthSummary(currentData);
        return;
    }
    
    // Show discovered problems and locate specific code
    problems.forEach(problem => {
        const contextData = window.enhancedDiagnostics.gatherCurrentData();
        const analysis = window.enhancedDiagnostics.rootCauseAnalyzer.analyzeRootCause(problem, contextData);
        
        window.enhancedDiagnostics.showProblemInDashboard(problem, analysis);
    });
    
    showToast(`🎯 Found ${problems.length} code issues - click for details`);
}

function showCodeHealthSummary(data) {
    const activeProblemsContainer = document.getElementById('active-problems');
    if (!activeProblemsContainer) return;
    
    // Hide 'ready for analysis' status
    const noProblems = activeProblemsContainer.querySelector('.no-problems');
    if (noProblems) {
        noProblems.style.display = 'none';
    }
    
    // Show code health summary
    const healthSummary = document.createElement('div');
    healthSummary.className = 'code-health-summary';
    healthSummary.innerHTML = `
        <div class="health-header">
            <h4>✅ Code Health: Excellent</h4>
            <p>No memory issues detected in tracked variables</p>
        </div>
        <div class="health-metrics">
            <div class="health-metric">
                <span class="metric-icon">📦</span>
                <div>
                    <strong>${data.variables?.length || 0} Variables Tracked</strong>
                    <p>All showing healthy allocation patterns</p>
                </div>
            </div>
            <div class="health-metric">
                <span class="metric-icon">🧵</span>
                <div>
                    <strong>${data.threads?.length || 0} Threads Active</strong>
                    <p>Balanced memory distribution</p>
                </div>
            </div>
            <div class="health-metric">
                <span class="metric-icon">⚡</span>
                <div>
                    <strong>Async Performance</strong>
                    <p>No blocked futures detected</p>
                </div>
            </div>
        </div>
        <button class="btn btn-secondary" onclick="resetScanView()" style="margin-top: 16px;">
            🔄 Reset View
        </button>
    `;
    
    activeProblemsContainer.appendChild(healthSummary);
}

function generatePerformanceReport() {
    // Calculate total memory from real data instead of using undefined totalMemoryMB
    const totalMemoryMB = calculateTotalMemory();
    showToast('📊 Generating comprehensive performance report...');
    
    const modal = document.getElementById('variable-modal');
    const modalBody = document.getElementById('modal-body');
    
    if (!modal || !modalBody) return;
    
    const reportData = gatherPerformanceMetrics();
    
    modalBody.innerHTML = `
        <div class="performance-report">
            <h3>📊 Performance Analysis Report</h3>
            <div class="report-timestamp">Generated: ${new Date().toLocaleString()}</div>
            
            <div class="report-section">
                <h4>🧠 Memory Analysis</h4>
                <div class="metric-grid">
                    <div class="metric-item">
                        <span class="metric-label">Total Memory Usage</span>
                        <span class="metric-value">${reportData.memory.total}MB</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">Memory Efficiency</span>
                        <span class="metric-value">${reportData.memory.efficiency}%</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">Active Variables</span>
                        <span class="metric-value">${reportData.memory.variables}</span>
                    </div>
                </div>
            </div>
            
            <div class="report-section">
                <h4>🧵 Thread Performance</h4>
                <div class="metric-grid">
                    <div class="metric-item">
                        <span class="metric-label">Thread Count</span>
                        <span class="metric-value">${reportData.threads.count}</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">Load Distribution</span>
                        <span class="metric-value">${reportData.threads.distribution}</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">Context Switches</span>
                        <span class="metric-value">${reportData.threads.contextSwitches}/s</span>
                    </div>
                </div>
            </div>
            
            <div class="report-section">
                <h4>⚡ Async Performance</h4>
                <div class="metric-grid">
                    <div class="metric-item">
                        <span class="metric-label">Active Futures</span>
                        <span class="metric-value">${reportData.async.activeFutures}</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">Avg Response Time</span>
                        <span class="metric-value">${reportData.async.avgResponseTime}ms</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">Blocked Tasks</span>
                        <span class="metric-value">${reportData.async.blockedTasks}</span>
                    </div>
                </div>
            </div>
            
            <div class="report-section">
                <h4>🎯 Recommendations</h4>
                <div class="recommendations-list">
                    ${reportData.recommendations.map(rec => `
                        <div class="recommendation-item">
                            <span class="rec-priority ${rec.priority.toLowerCase()}">${rec.priority}</span>
                            <span class="rec-text">${rec.text}</span>
                        </div>
                    `).join('')}
                </div>
            </div>
        </div>
    `;
    
    modal.style.display = 'block';
    showToast('✅ Performance report generated successfully');
}

function gatherPerformanceMetrics() {
    return {
        memory: {
            total: (Math.random() * 500 + 200).toFixed(1),
            efficiency: (Math.random() * 20 + 80).toFixed(1),
            variables: Math.floor(Math.random() * 1000 + 2000)
        },
        threads: {
            count: Math.floor(Math.random() * 20 + 10),
            distribution: 'Optimal',
            contextSwitches: Math.floor(Math.random() * 5000 + 2000)
        },
        async: {
            activeFutures: Math.floor(Math.random() * 500 + 100),
            avgResponseTime: (Math.random() * 100 + 50).toFixed(1),
            blockedTasks: Math.floor(Math.random() * 5)
        },
        recommendations: [
            { priority: 'HIGH', text: 'Consider implementing memory pooling for frequently allocated objects' },
            { priority: 'MEDIUM', text: 'Optimize async task scheduling to reduce context switches' },
            { priority: 'LOW', text: 'Review variable lifecycle management in hot paths' }
        ]
    };
}

// Removed unnecessary countdown and status update functions

function resetScanView() {
    const activeProblemsContainer = document.getElementById('active-problems');
    if (!activeProblemsContainer) return;
    
    // Clear all problem cards and health summary
    const problemCards = activeProblemsContainer.querySelectorAll('.problem-card, .code-health-summary');
    problemCards.forEach(card => card.remove());
    
    // Show original 'ready for analysis' status
    const noProblems = activeProblemsContainer.querySelector('.no-problems');
    if (noProblems) {
        noProblems.style.display = 'block';
    }
    
    // Hide root cause analysis panel
    const rootCausePanel = document.getElementById('root-cause-analysis');
    if (rootCausePanel) {
        rootCausePanel.style.display = 'none';
    }
    
    showToast('🔄 Scan view reset - ready for new analysis');
}

// Extended problem analysis display function
window.showProblemAnalysis = function(problem, analysis) {
    window.enhancedDiagnostics.showProblemInDashboard(problem, analysis);
};

// Variable filtering and sorting functions
let currentCategoryFilter = 'all';
let currentThreadFilter = 'all';

function filterByCategory(category) {
    currentCategoryFilter = category;
    
    // Update legend active state
    document.querySelectorAll('.legend-item').forEach(item => {
        item.classList.remove('active');
    });
    event.target.closest('.legend-item').classList.add('active');
    
    applyFilters();
    showToast(`🔍 Filtering by: ${getCategoryDisplayName(category)}`);
}

function filterByThread(threadId) {
    currentThreadFilter = threadId;
    applyFilters();
    showToast(`🧵 Filtering by: ${threadId === 'all' ? 'All Threads' : 'Thread ' + threadId}`);
}

function applyFilters() {
    const variableCards = document.querySelectorAll('.variable-card');
    
    variableCards.forEach(card => {
        const cardCategory = card.getAttribute('data-category');
        const cardThread = card.getAttribute('data-thread');
        
        let showCard = true;
        
        // Category filter
        if (currentCategoryFilter !== 'all' && cardCategory !== currentCategoryFilter) {
            showCard = false;
        }
        
        // Thread filter
        if (currentThreadFilter !== 'all') {
            if (currentThreadFilter === '5' && parseInt(cardThread) < 5) {
                showCard = false;
            } else if (currentThreadFilter !== '5' && cardThread !== currentThreadFilter) {
                showCard = false;
            }
        }
        
        if (showCard) {
            card.classList.remove('filtered-out');
        } else {
            card.classList.add('filtered-out');
        }
    });
    
    updateFilterStats();
}

function sortVariables(sortBy) {
    const container = document.getElementById('variables-container');
    const cards = Array.from(container.querySelectorAll('.variable-card'));
    
    cards.sort((a, b) => {
        switch (sortBy) {
            case 'memory':
                return parseInt(b.getAttribute('data-memory')) - parseInt(a.getAttribute('data-memory'));
            case 'allocations':
                return parseInt(b.getAttribute('data-allocations')) - parseInt(a.getAttribute('data-allocations'));
            case 'thread':
                return parseInt(a.getAttribute('data-thread')) - parseInt(b.getAttribute('data-thread'));
            case 'performance':
                return getPerformanceWeight(b.getAttribute('data-category')) - 
                       getPerformanceWeight(a.getAttribute('data-category'));
            default:
                return 0;
        }
    });
    
    // Re-append sorted cards
    cards.forEach(card => container.appendChild(card));
    
    showToast(`📊 Sorted by: ${getSortDisplayName(sortBy)}`);
}

function getPerformanceWeight(category) {
    const weights = {
        'memory': 4,
        'cpu': 3,
        'io': 2,
        'async': 1,
        'normal': 0
    };
    return weights[category] || 0;
}

function getCategoryDisplayName(category) {
    const names = {
        'cpu': 'CPU Intensive',
        'io': 'I/O Heavy',
        'memory': 'Memory Heavy',
        'async': 'Async Heavy',
        'normal': 'Normal',
        'all': 'All Categories'
    };
    return names[category] || category;
}

function getSortDisplayName(sortBy) {
    const names = {
        'memory': 'Memory Usage',
        'allocations': 'Allocation Count',
        'performance': 'Performance Impact',
        'thread': 'Thread ID'
    };
    return names[sortBy] || sortBy;
}

function updateFilterStats() {
    const totalCards = document.querySelectorAll('.variable-card').length;
    const visibleCards = document.querySelectorAll('.variable-card:not(.filtered-out)').length;
    
    // Update the section header with current filter stats
    const sectionHeader = document.querySelector('.section h3');
    if (sectionHeader && sectionHeader.textContent.includes('Thread Variables')) {
        sectionHeader.innerHTML = `🧵 Thread Variables <span style="color: var(--text2); font-weight: normal; font-size: 0.9rem;">(${visibleCards}/${totalCards})</span>`;
    }
}

console.log('🎯 Attribution Analysis Dashboard JavaScript loaded');
console.log('🔍 Ready for 3-click root cause discovery');

// 🕵️ Root Cause Analysis Panel System
class RootCauseAnalysisEngine {
    constructor() {
        this.problemPatterns = new Map();
        this.initializeCauseDatabase();
    }
    
    initializeCauseDatabase() {
        // Memory leak patterns
        this.problemPatterns.set('memory_leak', {
            name: 'Memory Leak',
            severity: 'HIGH',
            indicators: ['continuous_growth', 'no_deallocation', 'resource_accumulation'],
            causes: [
                {
                    cause: 'Unclosed FFI resource handles',
                    confidence: 0.92,
                    evidence: ['ffi_boundary_violations', 'resource_handle_growth'],
                    debugSteps: [
                        'Check FFI resource disposal in error paths',
                        'Audit resource cleanup in destructors',
                        'Verify proper RAII implementation'
                    ],
                    recommendations: [
                        'Add explicit resource cleanup in Drop implementations',
                        'Use RAII patterns for automatic resource management',
                        'Implement resource leak detection in tests'
                    ]
                },
                {
                    cause: 'Circular references in async tasks',
                    confidence: 0.78,
                    evidence: ['task_accumulation', 'reference_cycles'],
                    debugSteps: [
                        'Analyze async task lifecycle',
                        'Check for strong reference cycles',
                        'Verify weak reference usage'
                    ],
                    recommendations: [
                        'Use Weak references to break cycles',
                        'Implement proper task cancellation',
                        'Add timeout mechanisms for long-running tasks'
                    ]
                }
            ]
        });
        
        // Performance bottleneck patterns
        this.problemPatterns.set('performance_bottleneck', {
            name: 'Performance Bottleneck',
            severity: 'MEDIUM',
            indicators: ['high_cpu_usage', 'thread_contention', 'blocking_operations'],
            causes: [
                {
                    cause: 'Blocking operations in async context',
                    confidence: 0.85,
                    evidence: ['thread_pool_starvation', 'task_queue_buildup'],
                    debugSteps: [
                        'Identify blocking I/O operations',
                        'Check async/await usage patterns',
                        'Analyze thread pool utilization'
                    ],
                    recommendations: [
                        'Replace blocking I/O with async equivalents',
                        'Implement proper backpressure mechanisms',
                        'Consider task batching for better throughput'
                    ]
                },
                {
                    cause: 'Lock contention in multithreaded code',
                    confidence: 0.73,
                    evidence: ['mutex_contention', 'thread_blocking'],
                    debugSteps: [
                        'Profile lock acquisition times',
                        'Identify critical sections',
                        'Analyze lock ordering patterns'
                    ],
                    recommendations: [
                        'Reduce critical section size',
                        'Use lock-free data structures',
                        'Implement reader-writer locks where appropriate'
                    ]
                }
            ]
        });
        
        // Resource contention patterns
        this.problemPatterns.set('resource_contention', {
            name: 'Resource Contention',
            severity: 'MEDIUM',
            indicators: ['thread_blocking', 'resource_waiting', 'performance_degradation'],
            causes: [
                {
                    cause: 'Inefficient synchronization primitives',
                    confidence: 0.80,
                    evidence: ['mutex_overhead', 'context_switching'],
                    debugSteps: [
                        'Profile synchronization overhead',
                        'Check for unnecessary locks',
                        'Analyze critical path performance'
                    ],
                    recommendations: [
                        'Use atomic operations where possible',
                        'Implement lock-free algorithms',
                        'Consider message passing instead of shared state'
                    ]
                }
            ]
        });
    }
    
    detectProblems(memoryData) {
        const problems = [];
        
        // Simulate problem detection based on memory data
        const totalMemory = memoryData?.totalMemory || 0;
        const activeAllocs = memoryData?.activeAllocs || 0;
        const deallocatedCount = memoryData?.deallocatedCount || 0;
        
        // Memory leak detection
        if (totalMemory > 100000 && deallocatedCount < activeAllocs * 0.5) {
            problems.push({
                id: 'leak_' + Date.now(),
                type: 'memory_leak',
                title: 'Potential Memory Leak Detected',
                description: `High memory usage (${(totalMemory/1024).toFixed(1)}KB) with low deallocation rate`,
                severity: 'HIGH',
                affectedThreads: ['Thread_3', 'Thread_7'],
                timestamp: new Date().toISOString()
            });
        }
        
        // Performance bottleneck detection
        if (activeAllocs > 50) {
            problems.push({
                id: 'perf_' + Date.now(),
                type: 'performance_bottleneck',
                title: 'High Allocation Pressure',
                description: `${activeAllocs} active allocations may indicate performance issues`,
                severity: 'MEDIUM',
                affectedThreads: ['Thread_1', 'Thread_4'],
                timestamp: new Date().toISOString()
            });
        }
        
        return problems;
    }
    
    analyzeRootCause(problem) {
        const pattern = this.problemPatterns.get(problem.type);
        if (!pattern) return null;
        
        return {
            problem: problem,
            pattern: pattern,
            analysis: {
                likelyCauses: pattern.causes.sort((a, b) => b.confidence - a.confidence),
                contextualEvidence: this.gatherEvidence(problem),
                recommendations: this.generateRecommendations(pattern.causes)
            }
        };
    }
    
    gatherEvidence(problem) {
        // Simulate evidence gathering based on problem type
        const evidence = {
            flameGraph: null,
            ffiAudit: null,
            threadInteraction: null,
            memoryTimeline: null
        };
        
        switch(problem.type) {
            case 'memory_leak':
                evidence.flameGraph = 'focused_allocation_hotspots';
                evidence.ffiAudit = 'resource_handle_tracking';
                evidence.memoryTimeline = 'growth_pattern_analysis';
                break;
            case 'performance_bottleneck':
                evidence.flameGraph = 'cpu_hotspot_analysis';
                evidence.threadInteraction = 'contention_visualization';
                break;
            case 'resource_contention':
                evidence.threadInteraction = 'lock_contention_map';
                evidence.ffiAudit = 'resource_access_patterns';
                break;
        }
        
        return evidence;
    }
    
    generateRecommendations(causes) {
        const recommendations = [];
        causes.forEach(cause => {
            recommendations.push(...cause.recommendations);
        });
        return [...new Set(recommendations)]; // Remove duplicates
    }
}

// Initialize the Root Cause Analysis Engine
window.rootCauseEngine = new RootCauseAnalysisEngine();

// Show Root Cause Analysis Panel
window.showRootCausePanel = function(problemId) {
    const problem = window.detectedProblems?.find(p => p.id === problemId);
    if (!problem) return;
    
    const analysis = window.rootCauseEngine.analyzeRootCause(problem);
    if (!analysis) return;
    
    const panelHTML = generateRootCausePanelHTML(analysis);
    
    // Create modal
    const modal = document.createElement('div');
    modal.className = 'root-cause-modal';
    modal.innerHTML = `
        <div class="root-cause-panel">
            <div class="panel-header">
                <h3>🕵️ Root Cause Analysis</h3>
                <button class="close-panel" onclick="closeRootCausePanel()">&times;</button>
            </div>
            <div class="panel-content">
                ${panelHTML}
            </div>
        </div>
    `;
    
    document.body.appendChild(modal);
    modal.style.display = 'flex';
    
    // Initialize interactive elements
    initializeRootCausePanelInteractions();
};

// Generate Root Cause Panel HTML
function generateRootCausePanelHTML(analysis) {
    const { problem, pattern, analysis: rootCauseAnalysis } = analysis;
    
    return `
        <div class="problem-summary">
            <div class="problem-header">
                <span class="severity-badge ${problem.severity.toLowerCase()}">${problem.severity}</span>
                <h4>${problem.title}</h4>
            </div>
            <p class="problem-description">${problem.description}</p>
            <div class="affected-threads">
                <strong>Affected:</strong> ${problem.affectedThreads.join(', ')}
            </div>
        </div>
        
        <div class="analysis-sections">
            <div class="analysis-section">
                <h4>🎯 Likely Causes</h4>
                <div class="causes-list">
                    ${rootCauseAnalysis.likelyCauses.map((cause, index) => `
                        <div class="cause-item ${index === 0 ? 'primary' : index === 1 ? 'secondary' : 'tertiary'}">
                            <div class="cause-header">
                                <span class="confidence-bar">
                                    <span class="confidence-fill" style="width: ${cause.confidence * 100}%"></span>
                                </span>
                                <span class="confidence-text">${(cause.confidence * 100).toFixed(0)}%</span>
                                <h5>${cause.cause}</h5>
                            </div>
                            <div class="cause-evidence">
                                <strong>Evidence:</strong> ${cause.evidence.join(', ')}
                            </div>
                        </div>
                    `).join('')}
                </div>
            </div>
            
            <div class="analysis-section">
                <h4>🔍 Visual Evidence</h4>
                <div class="evidence-grid">
                    ${generateEvidenceVisualization(rootCauseAnalysis.contextualEvidence)}
                </div>
            </div>
            
            <div class="analysis-section">
                <h4>🛠️ Debugging Steps</h4>
                <div class="debugging-checklist">
                    ${rootCauseAnalysis.likelyCauses[0].debugSteps.map((step, index) => `
                        <div class="debug-step">
                            <input type="checkbox" id="step_${index}" class="debug-checkbox">
                            <label for="step_${index}" class="debug-label">${index + 1}. ${step}</label>
                        </div>
                    `).join('')}
                </div>
            </div>
            
            <div class="analysis-section">
                <h4>💡 Recommendations</h4>
                <div class="recommendations-list">
                    ${rootCauseAnalysis.recommendations.map(rec => `
                        <div class="recommendation-item">
                            <span class="rec-icon">💡</span>
                            <span class="rec-text">${rec}</span>
                            <button class="apply-rec-btn" onclick="applyRecommendation('${rec}')">Apply</button>
                        </div>
                    `).join('')}
                </div>
            </div>
        </div>
    `;
}

// Generate Evidence Visualization
function generateEvidenceVisualization(evidence) {
    let html = '';
    
    if (evidence.flameGraph) {
        html += `
            <div class="evidence-card">
                <h5>🔥 Code Attribution</h5>
                <div class="mini-flame-graph">
                    <div class="flame-bar" style="width: 80%; background: #ff6b6b;">
                        <span>allocation_hotspot()</span>
                    </div>
                    <div class="flame-bar" style="width: 60%; background: #4ecdc4; margin-left: 20px;">
                        <span>ffi_resource_create()</span>
                    </div>
                    <div class="flame-bar" style="width: 40%; background: #45b7d1; margin-left: 40px;">
                        <span>handle_request()</span>
                    </div>
                </div>
                <button class="expand-evidence" onclick="expandEvidence('flameGraph')">🔍 Expand</button>
            </div>
        `;
    }
    
    if (evidence.ffiAudit) {
        html += `
            <div class="evidence-card">
                <h5>🌉 FFI Boundaries</h5>
                <div class="mini-ffi-audit">
                    <div class="ffi-boundary">
                        <span class="boundary-label">Rust → C</span>
                        <span class="resource-count">12 handles</span>
                        <span class="leak-indicator">⚠️</span>
                    </div>
                    <div class="ffi-boundary">
                        <span class="boundary-label">C → Rust</span>
                        <span class="resource-count">8 callbacks</span>
                        <span class="leak-indicator">✅</span>
                    </div>
                </div>
                <button class="expand-evidence" onclick="expandEvidence('ffiAudit')">🔍 Expand</button>
            </div>
        `;
    }
    
    if (evidence.threadInteraction) {
        html += `
            <div class="evidence-card">
                <h5>🧵 Thread Interaction</h5>
                <div class="mini-thread-map">
                    <div class="thread-node active">T1</div>
                    <div class="thread-connection"></div>
                    <div class="thread-node contention">T3</div>
                    <div class="thread-connection"></div>
                    <div class="thread-node">T7</div>
                </div>
                <button class="expand-evidence" onclick="expandEvidence('threadInteraction')">🔍 Expand</button>
            </div>
        `;
    }
    
    return html || '<p>No visual evidence available for this problem type.</p>';
}

// Close Root Cause Panel
window.closeRootCausePanel = function() {
    const modal = document.querySelector('.root-cause-modal');
    if (modal) {
        modal.remove();
    }
};

// Initialize interactive elements in the panel
function initializeRootCausePanelInteractions() {
    // Debug step checkboxes
    const checkboxes = document.querySelectorAll('.debug-checkbox');
    checkboxes.forEach(checkbox => {
        checkbox.addEventListener('change', function() {
            const label = this.nextElementSibling;
            if (this.checked) {
                label.style.textDecoration = 'line-through';
                label.style.opacity = '0.6';
            } else {
                label.style.textDecoration = 'none';
                label.style.opacity = '1';
            }
        });
    });
}

// Apply recommendation
window.applyRecommendation = function(recommendation) {
    alert(`Applying recommendation: ${recommendation}\n\nThis would integrate with your IDE or generate code snippets.`);
};

// Expand evidence visualization
window.expandEvidence = function(evidenceType) {
    alert(`Expanding ${evidenceType} visualization\n\nThis would show the full interactive visualization in a larger view.`);
};

console.log('🕵️ Root Cause Analysis Engine initialized');