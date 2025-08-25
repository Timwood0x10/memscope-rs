// MemScope Dashboard JavaScript - Clean rendering for clean_dashboard.html
// This file contains comprehensive functions for memory analysis dashboard

// Global data store - will be populated by HTML template
window.analysisData = window.analysisData || {};

// FFI dashboard render style: 'svg' to mimic Rust SVG dashboard, 'cards' for card-based UI
const FFI_STYLE = 'svg';

// Initialize all dashboard components - Clean layout
function initCleanTemplate() {
    console.log('🚀 Initializing MemScope Dashboard...');
    console.log('📊 Available data:', Object.keys(window.analysisData||{}));
    const data = window.analysisData || {};

    // KPI 关键指标
    updateKPICards(data);

    // Memory by type (Chart.js)
    const typeChartEl = document.getElementById('typeChart');
    if (typeChartEl) {
        const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
        const byType = {};
        for (const a of allocs) { const t=a.type_name||'Unknown'; byType[t]=(byType[t]||0)+(a.size||0); }
        const top = Object.entries(byType).sort((a,b)=>b[1]-a[1]).slice(0,10);
        if (top.length>0) {
            const ctx = typeChartEl.getContext('2d');
            if (window.chartInstances['clean-type']) window.chartInstances['clean-type'].destroy();
            window.chartInstances['clean-type'] = new Chart(ctx, {
                type:'bar',
                data:{ labels: top.map(x=>x[0]), datasets:[{ label:'Bytes', data: top.map(x=>x[1]), backgroundColor:'#3b82f6' }] },
                options:{ responsive:true, plugins:{legend:{display:false}}, scales:{y:{beginAtZero:true}} }
            });
            const legend = document.getElementById('typeLegend');
            if (legend) legend.innerHTML = top.map(([n,v])=>`<span class="pill">${n}: ${formatBytes(v)}</span>`).join(' ');
        }
    }

    // Timeline (Chart.js)
    const timelineEl = document.getElementById('timelineChart');
    if (timelineEl) {
        const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
        const rawTimeline = (data.memory_analysis && data.memory_analysis.memory_timeline) || [];
        let points = [];
        if (rawTimeline.length) {
            points = rawTimeline.map((p,i)=>({ x:i, y:(p.memory_usage||0) }));
        } else {
            const sorted = allocs.slice().sort((a,b)=>(a.timestamp_alloc||0)-(b.timestamp_alloc||0));
            let cum=0; const step=Math.max(1, Math.floor(sorted.length/50));
            for(let i=0;i<sorted.length;i+=step){ cum += sorted[i].size||0; points.push({x:i, y:cum}); }
        }
        if (points.length>1) {
            const ctx = timelineEl.getContext('2d');
            if (window.chartInstances['clean-timeline']) window.chartInstances['clean-timeline'].destroy();
            window.chartInstances['clean-timeline'] = new Chart(ctx, {
                type:'line',
                data:{ labels: points.map(p=>p.x), datasets:[{ label:'Cumulative', data: points.map(p=>p.y), borderColor:'#ef4444', backgroundColor:'rgba(239,68,68,0.1)', fill:true, tension:0.25 }] },
                options:{ responsive:true, plugins:{legend:{display:false}}, scales:{y:{beginAtZero:true}} }
            });
        }
    }

    // Treemap
    const treemapEl = document.getElementById('treemap');
    if (treemapEl) {
        const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
        treemapEl.innerHTML = createTreemapVisualization(allocs);
    }

    // Growth
    const growthEl = document.getElementById('growth');
    if (growthEl) {
        const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
        const total = allocs.reduce((s,a)=>s+(a.size||0),0);
        growthEl.innerHTML = createAdvancedGrowthTrendVisualization(allocs, Math.max(1,total));
    }

    // Lifetimes (top 10)
    const lifetimesEl = document.getElementById('lifetimes');
    if (lifetimesEl) {
        const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
        const top = allocs.filter(a=>a.var_name && a.var_name!=='unknown').sort((a,b)=>(b.size||0)-(a.size||0)).slice(0,10);
        lifetimesEl.innerHTML = top.map(a=>`<div class="flex items-center justify-between py-1 border-b">
            <div class="text-xs font-medium">${a.var_name}</div>
            <div class="text-xs text-gray-500">${formatBytes(a.size||0)}</div>
        </div>`).join('');
    }

    // Update memory allocation table
    updateAllocationsTable(data);
    
    // Update unsafe risk table
    updateUnsafeTable(data);

    // Initialize all charts and visualizations
    initCharts(data);
    
    // Initialize lifecycle visualization
    initLifetimeVisualization(data);

    // Complex types
    const complexSummary = document.getElementById('complexSummary');
    if (complexSummary) {
        const ct = data.complex_types || {};
        const s = ct.summary || {};
        const items = [
            {label:'Complex Types', val: s.total_complex_types||0},
            {label:'Smart Pointers', val: s.smart_pointers_count||0},
            {label:'Collections', val: s.collections_count||0},
            {label:'Generic Types', val: s.generic_types_count||s.generic_type_count||0},
        ];
        complexSummary.innerHTML = items.map(x=>`<div class="pill">${x.label}: ${x.val}</div>`).join('');
        document.getElementById('complexSmart')?.replaceChildren();
        document.getElementById('complexCollections')?.replaceChildren();
        document.getElementById('complexGenerics')?.replaceChildren();
    }

    // Variable relationships
    const graphEl = document.getElementById('graph');
    if (graphEl) {
        // reuse our D3 relationship graph init but mount into #graph
        const container = document.createElement('div');
        container.id = 'variable-graph-container';
        container.style.width = '100%';
        container.style.height = '260px';
        graphEl.appendChild(container);
        try { initVariableGraph(); } catch(e) { console.warn('variable graph init failed', e); }
    }

    // Security violations
    const secEl = document.getElementById('security');
    if (secEl) {
        const root = data.unsafe_ffi || {};
        const list = root.security_hotspots || root.unsafe_reports || [];
        secEl.innerHTML = (list||[]).slice(0,12).map(h=>{
            const score = h.risk_score || h.risk_assessment?.confidence_score || 0;
            const level = h.risk_level || h.risk_assessment?.risk_level || 'Unknown';
            const width = Math.min(100, Math.round((score||0)*10));
            return `<div class="card">
              <div class="text-sm font-semibold">${h.location||h.report_id||'Unknown'}</div>
              <div class="text-xs text-gray-500">${h.description||h.source?.type||''}</div>
              <div class="mt-2 bg-red-100 h-2 rounded"><div style="width:${width}%; background:#ef4444; height:100%" class="rounded"></div></div>
              <div class="text-xs text-gray-500 mt-1">Risk: ${level} (${score})</div>
            </div>`;
        }).join('') || '<div class="muted">No security violations</div>';
    }
}
function initializeDashboard() {
    console.log('🚀 Initializing MemScope dashboard...');
    console.log('📊 Available data:', Object.keys(window.analysisData || {}));

    // Initialize theme system first
    initThemeToggle();

    // Initialize enhanced dashboard with comprehensive data
    initEnhancedSummaryStats();
    
    // Initialize all components
    initSummaryStats();
    initCharts();
    initMemoryUsageAnalysis();
    initLifetimeVisualization();
    initFFIVisualization();
    initMemoryFragmentation();
    initMemoryGrowthTrends();
    initAllocationsTable();
    initVariableGraph();
}

// Initialize theme toggle functionality
function initThemeToggle() {
    const themeToggle = document.getElementById('theme-toggle');
    const html = document.documentElement;

    // Check for saved theme preference or default to light mode
    const savedTheme = localStorage.getItem('memscope-theme') || 'light';

    console.log('🎨 Initializing theme system, saved theme:', savedTheme);

    // Apply initial theme
    applyTheme(savedTheme === 'dark');

    if (themeToggle) {
        themeToggle.addEventListener('click', () => {
            const isDark = html.classList.contains('dark');

            if (isDark) {
                applyTheme(false);
                localStorage.setItem('memscope-theme', 'light');
                console.log('🎨 Theme switched to: light mode');
            } else {
                applyTheme(true);
                localStorage.setItem('memscope-theme', 'dark');
                console.log('🎨 Theme switched to: dark mode');
            }
        });

        console.log('✅ Theme toggle initialized successfully');
    } else {
        console.warn('⚠️ Theme toggle button not found');
    }
}

// Apply theme to all modules
function applyTheme(isDark) {
    const html = document.documentElement;
    const body = document.body;

    if (isDark) {
        html.classList.remove('light');
        html.classList.add('dark');
        body.classList.add('dark');
    } else {
        html.classList.remove('dark');
        html.classList.add('light');
        body.classList.remove('dark');
    }

    // Force immediate repaint
    html.style.display = 'none';
    html.offsetHeight; // Trigger reflow
    html.style.display = '';

    // Apply theme to all modules that need explicit dark mode support
    applyThemeToAllModules(isDark);

    // Update theme toggle button icon
    updateThemeToggleIcon(isDark);

    // Destroy existing charts before reinitializing
    destroyAllCharts();

    // Reinitialize charts to apply theme changes
    setTimeout(() => {
        initCharts();
        initFFIRiskChart();
    }, 100);
}

// Update theme toggle button icon
function updateThemeToggleIcon(isDark) {
    const themeToggle = document.getElementById('theme-toggle');
    if (themeToggle) {
        const icon = themeToggle.querySelector('i');
        if (icon) {
            if (isDark) {
                icon.className = 'fa fa-sun';
            } else {
                icon.className = 'fa fa-moon';
            }
        }
    }
}

// Global chart instances storage
window.chartInstances = {};

// Destroy all existing charts
function destroyAllCharts() {
    Object.keys(window.chartInstances).forEach(chartId => {
        if (window.chartInstances[chartId]) {
            window.chartInstances[chartId].destroy();
            delete window.chartInstances[chartId];
        }
    });
}

// Apply theme to specific modules
function applyThemeToAllModules(isDark) {
    const modules = [
        'memory-usage-analysis',
        'generic-types-details',
        'variable-relationship-graph',
        'complex-type-analysis',
        'memory-optimization-recommendations',
        'unsafe-ffi-data'
    ];

    modules.forEach(moduleId => {
        const module = document.getElementById(moduleId);
        if (module) {
            module.classList.toggle('dark', isDark);
        }
    });

    // Also apply to any table elements that might need it
    const tables = document.querySelectorAll('table');
    tables.forEach(table => {
        table.classList.toggle('dark', isDark);
    });

    // Apply to any chart containers
    const chartContainers = document.querySelectorAll('canvas');
    chartContainers.forEach(container => {
        if (container.parentElement) {
            container.parentElement.classList.toggle('dark', isDark);
        }
    });
}

// Initialize summary statistics
function initSummaryStats() {
    console.log('📊 Initializing summary stats...');

    const data = window.analysisData;

    // Update complex types count
    const complexTypesCount = data.complex_types?.summary?.total_complex_types || 0;
    updateElement('total-complex-types', complexTypesCount);

    // Update total allocations
    const totalAllocations = data.memory_analysis?.allocations?.length || 0;
    updateElement('total-allocations', totalAllocations);

    // Update generic types count
    const genericTypeCount = data.complex_types?.summary?.generic_type_count || 0;
    updateElement('generic-type-count', genericTypeCount);

    // Update unsafe FFI count
    const unsafeFFICount = data.unsafe_ffi?.enhanced_ffi_data?.length || 0;
    updateElement('unsafe-ffi-count', unsafeFFICount);

    // Update category counts
    const smartPointersCount = data.complex_types?.categorized_types?.smart_pointers?.length || 0;
    const collectionsCount = data.complex_types?.categorized_types?.collections?.length || 0;
    const primitivesCount = 0; // Calculate from data if available

    updateElement('smart-pointers-count', smartPointersCount);
    updateElement('collections-count', collectionsCount);
    updateElement('primitives-count', primitivesCount);
}

// Initialize charts - simplified
function initCharts() {
    console.log('📊 Initializing charts...');

    // Initialize memory distribution chart
    initMemoryDistributionChart();

    // Initialize allocation size chart
    initAllocationSizeChart();
}



// Initialize memory distribution chart
function initMemoryDistributionChart() {
    const ctx = document.getElementById('memory-distribution-chart');
    if (!ctx) return;

    const allocations = window.analysisData.memory_analysis?.allocations || [];
    const typeDistribution = {};

    allocations.forEach(alloc => {
        const type = alloc.type_name || 'System Allocation';
        typeDistribution[type] = (typeDistribution[type] || 0) + alloc.size;
    });

    const sortedTypes = Object.entries(typeDistribution)
        .sort(([, a], [, b]) => b - a)
        .slice(0, 10);

    const isDark = document.documentElement.classList.contains('dark');

    // Destroy existing chart if it exists
    if (window.chartInstances['memory-distribution-chart']) {
        window.chartInstances['memory-distribution-chart'].destroy();
    }

    window.chartInstances['memory-distribution-chart'] = new Chart(ctx, {
        type: 'bar',
        data: {
            labels: sortedTypes.map(([type]) => formatTypeName(type)),
            datasets: [{
                label: 'Memory Usage (bytes)',
                data: sortedTypes.map(([, size]) => size),
                backgroundColor: '#3b82f6'
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    labels: {
                        color: isDark ? '#ffffff' : '#374151'
                    }
                }
            },
            scales: {
                x: {
                    ticks: {
                        color: isDark ? '#d1d5db' : '#6b7280'
                    },
                    grid: {
                        color: isDark ? '#374151' : '#e5e7eb'
                    }
                },
                y: {
                    beginAtZero: true,
                    ticks: {
                        color: isDark ? '#d1d5db' : '#6b7280',
                        callback: function (value) {
                            return formatBytes(value);
                        }
                    },
                    grid: {
                        color: isDark ? '#374151' : '#e5e7eb'
                    }
                }
            }
        }
    });
}

// Initialize allocation size chart
function initAllocationSizeChart() {
    const ctx = document.getElementById('allocation-size-chart');
    if (!ctx) return;

    const allocations = window.analysisData.memory_analysis?.allocations || [];
    const sizeDistribution = {
        'Tiny (< 64B)': 0,
        'Small (64B - 1KB)': 0,
        'Medium (1KB - 64KB)': 0,
        'Large (64KB - 1MB)': 0,
        'Huge (> 1MB)': 0
    };

    allocations.forEach(alloc => {
        const size = alloc.size || 0;
        if (size < 64) sizeDistribution['Tiny (< 64B)']++;
        else if (size < 1024) sizeDistribution['Small (64B - 1KB)']++;
        else if (size < 65536) sizeDistribution['Medium (1KB - 64KB)']++;
        else if (size < 1048576) sizeDistribution['Large (64KB - 1MB)']++;
        else sizeDistribution['Huge (> 1MB)']++;
    });

    // Destroy existing chart if it exists
    if (window.chartInstances['allocation-size-chart']) {
        window.chartInstances['allocation-size-chart'].destroy();
    }

    window.chartInstances['allocation-size-chart'] = new Chart(ctx, {
        type: 'pie',
        data: {
            labels: Object.keys(sizeDistribution),
            datasets: [{
                data: Object.values(sizeDistribution),
                backgroundColor: ['#10b981', '#3b82f6', '#f59e0b', '#ef4444', '#7c2d12']
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    labels: {
                        color: document.documentElement.classList.contains('dark') ? '#ffffff' : '#374151'
                    }
                }
            }
        }
    });
}



// Process memory analysis data with validation and fallback
function processMemoryAnalysisData(rawData) {
    if (!rawData || !rawData.memory_analysis) {
        console.warn('⚠️ No memory analysis data found, generating fallback data');
        return generateFallbackMemoryData();
    }

    const memoryData = rawData.memory_analysis;
    const processedData = {
        stats: {
            total_allocations: memoryData.stats?.total_allocations || 0,
            active_allocations: memoryData.stats?.active_allocations || 0,
            total_memory: memoryData.stats?.total_memory || 0,
            active_memory: memoryData.stats?.active_memory || 0
        },
        allocations: memoryData.allocations || [],
        trends: {
            peak_memory: memoryData.peak_memory || 0,
            growth_rate: memoryData.growth_rate || 0,
            fragmentation_score: memoryData.fragmentation_score || 0
        }
    };

    // Calculate additional metrics if not present
    if (processedData.allocations.length > 0) {
        const totalSize = processedData.allocations.reduce((sum, alloc) => sum + (alloc.size || 0), 0);
        if (!processedData.stats.total_memory) {
            processedData.stats.total_memory = totalSize;
        }
        if (!processedData.stats.total_allocations) {
            processedData.stats.total_allocations = processedData.allocations.length;
        }
    }

    console.log('✅ Processed memory analysis data:', processedData);
    return processedData;
}

// Generate fallback memory data when real data is unavailable
function generateFallbackMemoryData() {
    console.log('🔄 Generating fallback memory data');

    return {
        stats: {
            total_allocations: 0,
            active_allocations: 0,
            total_memory: 0,
            active_memory: 0
        },
        allocations: [],
        trends: {
            peak_memory: 0,
            growth_rate: 0,
            fragmentation_score: 0
        },
        isFallback: true
    };
}

// Validate memory data structure
function validateMemoryData(data) {
    if (!data) return false;

    const hasStats = data.stats && typeof data.stats === 'object';
    const hasAllocations = Array.isArray(data.allocations);

    return hasStats && hasAllocations;
}

// Calculate memory statistics from allocations
function calculateMemoryStatistics(allocations) {
    if (!Array.isArray(allocations) || allocations.length === 0) {
        return {
            totalSize: 0,
            averageSize: 0,
            largestAllocation: 0,
            userAllocations: 0,
            systemAllocations: 0
        };
    }

    const totalSize = allocations.reduce((sum, alloc) => sum + (alloc.size || 0), 0);
    const averageSize = totalSize / allocations.length;
    const largestAllocation = Math.max(...allocations.map(alloc => alloc.size || 0));

    const userAllocations = allocations.filter(alloc =>
        alloc.var_name && alloc.var_name !== 'unknown' &&
        alloc.type_name && alloc.type_name !== 'unknown'
    ).length;

    const systemAllocations = allocations.length - userAllocations;

    return {
        totalSize,
        averageSize,
        largestAllocation,
        userAllocations,
        systemAllocations
    };
}

// Initialize memory usage analysis with enhanced SVG-style visualization
function initMemoryUsageAnalysis() {
    const container = document.getElementById('memory-usage-analysis');
    if (!container) return;

    // Process memory data with validation
    const memoryData = processMemoryAnalysisData(window.analysisData);
    const allocations = memoryData.allocations;

    if (allocations.length === 0 || memoryData.isFallback) {
        container.innerHTML = createEnhancedEmptyState();
        return;
    }

    // Calculate comprehensive statistics
    const stats = calculateMemoryStatistics(allocations);
    const totalMemory = stats.totalSize;

    const userAllocations = allocations.filter(alloc =>
        alloc.var_name && alloc.var_name !== 'unknown' &&
        alloc.type_name && alloc.type_name !== 'unknown'
    );
    const systemAllocations = allocations.filter(alloc =>
        !alloc.var_name || alloc.var_name === 'unknown' ||
        !alloc.type_name || alloc.type_name === 'unknown'
    );

    const userMemory = userAllocations.reduce((sum, alloc) => sum + (alloc.size || 0), 0);
    const systemMemory = systemAllocations.reduce((sum, alloc) => sum + (alloc.size || 0), 0);

    // Create enhanced SVG-style visualization
    container.innerHTML = createMemoryAnalysisSVG(stats, allocations, userMemory, systemMemory, totalMemory);
}

// Create enhanced empty state with better styling
function createEnhancedEmptyState() {
    return `
        <div class="h-full flex items-center justify-center">
            <div class="text-center p-8 bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-800 dark:to-gray-700 rounded-xl border-2 border-dashed border-blue-200 dark:border-gray-600">
                <div class="mb-4">
                    <svg class="w-16 h-16 mx-auto text-blue-400 dark:text-blue-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path>
                    </svg>
                </div>
                <h4 class="text-lg font-semibold mb-2 text-gray-800 dark:text-gray-200">Memory Analysis Ready</h4>
                <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">No memory allocation data found for analysis</p>
                <p class="text-xs text-gray-500 dark:text-gray-500">Run your application with memory tracking enabled to see detailed analysis</p>
            </div>
        </div>
    `;
}

// Create comprehensive SVG-style memory analysis visualization inspired by the memoryAnalysis.svg
function createMemoryAnalysisSVG(stats, allocations, userMemory, systemMemory, totalMemory) {
    const userPercentage = totalMemory > 0 ? (userMemory / totalMemory * 100) : 0;
    const systemPercentage = totalMemory > 0 ? (systemMemory / totalMemory * 100) : 0;

    // Calculate comprehensive efficiency metrics
    const efficiency = totalMemory > 0 ? Math.min(100, (userMemory / totalMemory * 100)) : 0;
    const reclamationRate = allocations.length > 0 ? Math.min(100, ((allocations.filter(a => a.timestamp_dealloc).length / allocations.length) * 100)) : 0;
    const fragmentation = Math.min(100, (allocations.length / Math.max(1, totalMemory / 1024)) * 10);

    // Advanced size distribution analysis
    const sizeDistribution = {
        tiny: allocations.filter(a => a.size < 64).length,
        small: allocations.filter(a => a.size >= 64 && a.size < 1024).length,
        medium: allocations.filter(a => a.size >= 1024 && a.size < 65536).length,
        large: allocations.filter(a => a.size >= 65536).length
    };

    // Calculate median and P95 sizes
    const sizes = allocations.map(a => a.size || 0).sort((a, b) => a - b);
    const medianSize = sizes.length > 0 ? sizes[Math.floor(sizes.length / 2)] : 0;
    const p95Size = sizes.length > 0 ? sizes[Math.floor(sizes.length * 0.95)] : 0;

    return `
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg overflow-hidden">
            <!-- Header with gradient background -->
            <div class="bg-gradient-to-r from-blue-600 to-purple-600 text-white p-6">
                <div class="text-center">
                    <h2 class="text-3xl font-bold mb-2">Rust Memory Usage Analysis</h2>
                    <p class="text-blue-100 uppercase tracking-wider text-sm">Key Performance Metrics</p>
                </div>
            </div>

            <div class="p-6">
                <!-- Key Performance Metrics Grid -->
                <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-8 gap-4 mb-8">
                    ${createAdvancedMetricCard('Active Memory', formatBytes(userMemory), Math.round(userPercentage), '#3498db', 'MEDIUM')}
                    ${createAdvancedMetricCard('Peak Memory', formatBytes(totalMemory), 100, '#e74c3c', 'HIGH')}
                    ${createAdvancedMetricCard('Active Allocs', allocations.length, 100, '#2ecc71', 'HIGH')}
                    ${createAdvancedMetricCard('Reclamation', reclamationRate.toFixed(1) + '%', Math.round(reclamationRate), '#f39c12', reclamationRate > 70 ? 'OPTIMAL' : 'MEDIUM')}
                    ${createAdvancedMetricCard('Efficiency', efficiency.toFixed(1) + '%', Math.round(efficiency), '#9b59b6', efficiency > 70 ? 'OPTIMAL' : 'MEDIUM')}
                    ${createAdvancedMetricCard('Median Size', formatBytes(medianSize), Math.min(100, medianSize / 1024), '#1abc9c', medianSize < 100 ? 'OPTIMAL' : 'MEDIUM')}
                    ${createAdvancedMetricCard('P95 Size', formatBytes(p95Size), Math.min(100, p95Size / 1024), '#e67e22', p95Size < 1024 ? 'OPTIMAL' : 'MEDIUM')}
                    ${createAdvancedMetricCard('Fragmentation', fragmentation.toFixed(1) + '%', Math.round(fragmentation), '#95a5a6', fragmentation < 30 ? 'OPTIMAL' : 'MEDIUM')}
                </div>


                <!-- Memory Usage by Type - Enhanced Treemap -->
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6 mb-8 border border-gray-200 dark:border-gray-600">
                    <h3 class="text-xl font-semibold mb-4 text-gray-800 dark:text-white text-center">Memory Usage by Type - Treemap Visualization</h3>
                    <div class="bg-gray-100 dark:bg-gray-600 rounded-lg p-4 h-64 relative overflow-hidden">
                        ${createAdvancedTreemapVisualization(allocations, totalMemory)}
                    </div>
                    <div class="mt-4 grid grid-cols-3 gap-4 text-xs">
                        <div class="flex items-center">
                            <div class="w-3 h-3 bg-blue-500 rounded mr-2"></div>
                            <span class="text-gray-600 dark:text-gray-300">Collections</span>
                        </div>
                        <div class="flex items-center">
                            <div class="w-3 h-3 bg-green-500 rounded mr-2"></div>
                            <span class="text-gray-600 dark:text-gray-300">Basic Types</span>
                        </div>
                        <div class="flex items-center">
                            <div class="w-3 h-3 bg-gray-500 rounded mr-2"></div>
                            <span class="text-gray-600 dark:text-gray-300">System</span>
                        </div>
                    </div>
                </div>

                <!-- Advanced Analysis Grid -->
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                    <!-- Memory Fragmentation Analysis -->
                    <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6 border border-gray-200 dark:border-gray-600">
                        <h3 class="text-xl font-semibold mb-4 text-gray-800 dark:text-white text-center">Memory Fragmentation Analysis</h3>
                        <div class="space-y-4">
                            ${createAdvancedFragmentationBar('Tiny (0-64B)', sizeDistribution.tiny, allocations.length, '#27ae60')}
                            ${createAdvancedFragmentationBar('Small (65B-1KB)', sizeDistribution.small, allocations.length, '#f39c12')}
                            ${createAdvancedFragmentationBar('Medium (1KB-64KB)', sizeDistribution.medium, allocations.length, '#e74c3c')}
                            ${createAdvancedFragmentationBar('Large (>64KB)', sizeDistribution.large, allocations.length, '#8e44ad')}
                        </div>
                    </div>

                    <!-- Call Stack Analysis -->
                    <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6 border border-gray-200 dark:border-gray-600">
                        <h3 class="text-xl font-semibold mb-4 text-gray-800 dark:text-white text-center">Call Stack Analysis</h3>
                        <div class="space-y-3 max-h-64 overflow-y-auto">
                            ${createCallStackAnalysis(allocations)}
                        </div>
                    </div>
                </div>

                <!-- Memory Statistics Summary -->
                <div class="mt-8 bg-gray-50 dark:bg-gray-700 rounded-lg p-6 border border-gray-200 dark:border-gray-600">
                    <h3 class="text-xl font-semibold mb-4 text-gray-800 dark:text-white text-center">Memory Statistics</h3>
                    <div class="grid grid-cols-3 gap-4 text-sm text-center">
                        <div>
                            <span class="text-gray-600 dark:text-gray-400">Peak Memory:</span>
                            <span class="font-semibold text-red-600 dark:text-red-400 ml-2">${formatBytes(totalMemory)}</span>
                        </div>
                        <div>
                            <span class="text-gray-600 dark:text-gray-400">Fragmentation:</span>
                            <span class="font-semibold text-orange-600 dark:text-orange-400 ml-2">${fragmentation.toFixed(1)}%</span>
                        </div>
                        <div>
                            <span class="text-gray-600 dark:text-gray-400">Efficiency:</span>
                            <span class="font-semibold text-purple-600 dark:text-purple-400 ml-2">${efficiency.toFixed(1)}%</span>
                        </div>
                    </div>
                </div>

                <!-- Variable Allocation Timeline -->
                <div class="mt-8 bg-gray-50 dark:bg-gray-700 rounded-lg p-6 border border-gray-200 dark:border-gray-600">
                    <h3 class="text-xl font-semibold mb-4 text-gray-800 dark:text-white text-center">Variable Allocation Timeline</h3>
                    <div class="space-y-3 max-h-64 overflow-y-auto">
                        ${createVariableAllocationTimeline(allocations)}
                    </div>
                </div>
            </div>
        </div>
    `;
}

// Create metric card with circular progress indicator
function createMetricCard(title, value, percentage, color, status) {
    const circumference = 2 * Math.PI * 25;
    const strokeDasharray = circumference;
    const strokeDashoffset = circumference - (percentage / 100) * circumference;

    const statusColors = {
        'OPTIMAL': '#27ae60',
        'MEDIUM': '#f39c12',
        'HIGH': '#e74c3c'
    };

    return `
        <div class="bg-white dark:bg-gray-700 rounded-lg p-4 shadow-sm hover:shadow-md transition-shadow">
            <div class="flex items-center justify-between">
                <div class="flex-1">
                    <p class="text-xs font-medium text-gray-600 dark:text-gray-400 uppercase">${title}</p>
                    <p class="text-lg font-bold text-gray-900 dark:text-white">${value}</p>
                    <div class="flex items-center mt-1">
                        <div class="w-2 h-2 rounded-full mr-2" style="background-color: ${statusColors[status]}"></div>
                        <span class="text-xs font-semibold" style="color: ${statusColors[status]}">${status}</span>
                    </div>
                </div>
                <div class="relative w-12 h-12">
                    <svg class="w-12 h-12 transform -rotate-90" viewBox="0 0 60 60">
                        <circle cx="30" cy="30" r="25" stroke="#e5e7eb" stroke-width="6" fill="none" class="dark:stroke-gray-600"/>
                        <circle cx="30" cy="30" r="25" stroke="${color}" stroke-width="6" fill="none" 
                                stroke-dasharray="${strokeDasharray}" stroke-dashoffset="${strokeDashoffset}"
                                stroke-linecap="round" class="transition-all duration-500"/>
                    </svg>
                    <div class="absolute inset-0 flex items-center justify-center">
                        <span class="text-xs font-bold" style="color: ${color}">${Math.round(percentage)}%</span>
                    </div>
                </div>
            </div>
        </div>
    `;
}

// Create timeline visualization
function createTimelineVisualization(allocations) {
    if (allocations.length === 0) return '<div class="flex items-center justify-center h-full text-gray-400">No timeline data</div>';

    const sortedAllocs = allocations.sort((a, b) => (a.timestamp_alloc || 0) - (b.timestamp_alloc || 0));
    const minTime = sortedAllocs[0]?.timestamp_alloc || 0;
    const maxTime = sortedAllocs[sortedAllocs.length - 1]?.timestamp_alloc || minTime + 1;
    const timeRange = maxTime - minTime || 1;

    return sortedAllocs.slice(0, 20).map((alloc, index) => {
        const position = ((alloc.timestamp_alloc - minTime) / timeRange) * 100;
        const height = Math.min(80, Math.max(4, (alloc.size / 1024) * 20));
        const color = alloc.var_name && alloc.var_name !== 'unknown' ? '#3498db' : '#95a5a6';

        return `
            <div class="absolute bottom-0 bg-opacity-80 rounded-t transition-all hover:bg-opacity-100" 
                 style="left: ${position}%; width: 4px; height: ${height}%; background-color: ${color};"
                 title="${alloc.var_name || 'System'}: ${formatBytes(alloc.size)}">
            </div>
        `;
    }).join('');
}

// Create treemap-style visualization
function createTreemapVisualization(allocations) {
    const typeGroups = {};
    allocations.forEach(alloc => {
        const type = alloc.type_name || 'System';
        if (!typeGroups[type]) {
            typeGroups[type] = { count: 0, size: 0 };
        }
        typeGroups[type].count++;
        typeGroups[type].size += alloc.size || 0;
    });

    const sortedTypes = Object.entries(typeGroups)
        .sort(([, a], [, b]) => b.size - a.size)
        .slice(0, 8);

    const totalSize = sortedTypes.reduce((sum, [, data]) => sum + data.size, 0);

    let currentX = 0;
    return sortedTypes.map(([type, data], index) => {
        const width = totalSize > 0 ? (data.size / totalSize) * 100 : 12.5;
        const color = getTypeColor(type, index);
        const result = `
            <div class="absolute h-full transition-all hover:brightness-110 cursor-pointer rounded" 
                 style="left: ${currentX}%; width: ${width}%; background-color: ${color};"
                 title="${type}: ${formatBytes(data.size)} (${data.count} allocs)">
                <div class="p-2 h-full flex flex-col justify-center text-white text-xs font-semibold text-center">
                    <div class="truncate">${type.length > 10 ? type.substring(0, 8) + '...' : type}</div>
                    <div class="text-xs opacity-90">${formatBytes(data.size)}</div>
                </div>
            </div>
        `;
        currentX += width;
        return result;
    }).join('');
}

// Create fragmentation bar
function createFragmentationBar(label, count, total, color) {
    const percentage = total > 0 ? (count / total) * 100 : 0;
    return `
        <div class="flex items-center justify-between">
            <span class="text-sm font-medium text-gray-700 dark:text-gray-300 w-24">${label}</span>
            <div class="flex-1 mx-3">
                <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-4">
                    <div class="h-4 rounded-full transition-all duration-500" 
                         style="width: ${percentage}%; background-color: ${color}"></div>
                </div>
            </div>
            <span class="text-sm font-bold text-gray-900 dark:text-white w-12 text-right">${count}</span>
        </div>
    `;
}

// Create growth trend visualization
function createGrowthTrendVisualization(allocations) {
    if (allocations.length < 2) return '<div class="flex items-center justify-center h-full text-gray-400">Insufficient data</div>';

    const sortedAllocs = allocations.sort((a, b) => (a.timestamp_alloc || 0) - (b.timestamp_alloc || 0));
    const points = [];
    let cumulativeSize = 0;

    sortedAllocs.forEach((alloc, index) => {
        cumulativeSize += alloc.size || 0;
        if (index % Math.max(1, Math.floor(sortedAllocs.length / 10)) === 0) {
            points.push(cumulativeSize);
        }
    });

    const maxSize = Math.max(...points);

    return points.map((size, index) => {
        const x = (index / (points.length - 1)) * 100;
        const y = 100 - (size / maxSize) * 80;

        return `
            <div class="absolute w-2 h-2 bg-green-500 rounded-full transform -translate-x-1 -translate-y-1" 
                 style="left: ${x}%; top: ${y}%"
                 title="Memory: ${formatBytes(size)}">
            </div>
            ${index > 0 ? `
                <div class="absolute h-0.5 bg-green-500" 
                     style="left: ${((index - 1) / (points.length - 1)) * 100}%; 
                            top: ${100 - (points[index - 1] / maxSize) * 80}%; 
                            width: ${(100 / (points.length - 1))}%;
                            transform: rotate(${Math.atan2(y - (100 - (points[index - 1] / maxSize) * 80), 100 / (points.length - 1)) * 180 / Math.PI}deg);
                            transform-origin: left center;">
                </div>
            ` : ''}
        `;
    }).join('');
}

// Get color for type visualization
function getTypeColor(type, index) {
    const colors = [
        '#3498db', '#e74c3c', '#2ecc71', '#f39c12',
        '#9b59b6', '#1abc9c', '#e67e22', '#95a5a6'
    ];

    if (type.toLowerCase().includes('vec')) return '#3498db';
    if (type.toLowerCase().includes('string')) return '#f39c12';
    if (type.toLowerCase().includes('hash')) return '#e74c3c';
    if (type.toLowerCase().includes('btree')) return '#2ecc71';

    return colors[index % colors.length];
}

// Create advanced metric card with enhanced styling
function createAdvancedMetricCard(title, value, percentage, color, status) {
    const circumference = 2 * Math.PI * 20;
    const strokeDasharray = circumference;
    const strokeDashoffset = circumference - (percentage / 100) * circumference;

    const statusColors = {
        'OPTIMAL': '#27ae60',
        'MEDIUM': '#f39c12',
        'HIGH': '#e74c3c'
    };

    return `
        <div class="bg-white dark:bg-gray-700 rounded-lg p-3 shadow-sm hover:shadow-md transition-all border border-gray-200 dark:border-gray-600">
            <div class="flex flex-col items-center">
                <div class="relative w-10 h-10 mb-2">
                    <svg class="w-10 h-10 transform -rotate-90" viewBox="0 0 50 50">
                        <circle cx="25" cy="25" r="20" stroke="#e5e7eb" stroke-width="4" fill="none" class="dark:stroke-gray-600"/>
                        <circle cx="25" cy="25" r="20" stroke="${color}" stroke-width="4" fill="none" 
                                stroke-dasharray="${strokeDasharray}" stroke-dashoffset="${strokeDashoffset}"
                                stroke-linecap="round" class="transition-all duration-500"/>
                    </svg>
                    <div class="absolute inset-0 flex items-center justify-center">
                        <span class="text-xs font-bold" style="color: ${color}">${Math.round(percentage)}%</span>
                    </div>
                </div>
                <p class="text-xs font-medium text-gray-600 dark:text-gray-400 uppercase text-center">${title}</p>
                <p class="text-sm font-bold text-gray-900 dark:text-white text-center">${value}</p>
                <div class="flex items-center mt-1">
                    <div class="w-1.5 h-1.5 rounded-full mr-1" style="background-color: ${statusColors[status]}"></div>
                    <span class="text-xs font-semibold" style="color: ${statusColors[status]}">${status}</span>
                </div>
            </div>
        </div>
    `;
}

// Create advanced timeline visualization
function createAdvancedTimelineVisualization(allocations, totalMemory) {
    if (allocations.length === 0) return '<div class="flex items-center justify-center h-full text-gray-400">No timeline data</div>';

    const sortedAllocs = allocations.sort((a, b) => (a.timestamp_alloc || 0) - (b.timestamp_alloc || 0));
    const minTime = sortedAllocs[0]?.timestamp_alloc || 0;
    const maxTime = sortedAllocs[sortedAllocs.length - 1]?.timestamp_alloc || minTime + 1;
    const timeRange = maxTime - minTime || 1;

    // Group allocations by scope/type for better visualization
    const scopeGroups = {};
    sortedAllocs.forEach(alloc => {
        const scope = alloc.scope_name || (alloc.var_name ? 'User Variables' : 'System');
        if (!scopeGroups[scope]) scopeGroups[scope] = [];
        scopeGroups[scope].push(alloc);
    });

    const scopeColors = ['#3498db', '#e74c3c', '#2ecc71', '#f39c12', '#9b59b6', '#1abc9c'];
    let scopeIndex = 0;

    return Object.entries(scopeGroups).map(([scope, allocs]) => {
        const color = scopeColors[scopeIndex % scopeColors.length];
        scopeIndex++;
        const yOffset = scopeIndex * 25;

        return `
            <div class="absolute" style="top: ${yOffset}px; left: 0; right: 0; height: 20px;">
                <div class="text-xs font-medium text-gray-700 dark:text-gray-300 mb-1" style="color: ${color}">
                    ${scope} (${allocs.length} allocs)
                </div>
                ${allocs.slice(0, 20).map(alloc => {
            const position = ((alloc.timestamp_alloc - minTime) / timeRange) * 100;
            const width = Math.max(2, (alloc.size / totalMemory) * 100);

            return `
                        <div class="absolute h-4 rounded opacity-80 hover:opacity-100 transition-opacity cursor-pointer" 
                             style="left: ${position}%; width: ${Math.max(4, width)}px; background-color: ${color};"
                             title="${alloc.var_name || 'System'}: ${formatBytes(alloc.size)}">
                        </div>
                    `;
        }).join('')}
            </div>
        `;
    }).join('');
}

// Create advanced treemap visualization inspired by SVG design
function createAdvancedTreemapVisualization(allocations, totalMemory) {
    if (allocations.length === 0) return '<div class="flex items-center justify-center h-full text-gray-400">No allocation data</div>';

    // Group allocations by type and category
    const typeGroups = {};
    const categoryGroups = {
        'Collections': { types: {}, totalSize: 0, color: '#3498db' },
        'Basic Types': { types: {}, totalSize: 0, color: '#27ae60' },
        'Smart Pointers': { types: {}, totalSize: 0, color: '#9b59b6' },
        'System': { types: {}, totalSize: 0, color: '#95a5a6' }
    };

    allocations.forEach(alloc => {
        const type = alloc.type_name || 'System';
        const category = getTypeCategory(type);
        const categoryName = getCategoryName(category);
        
        if (!typeGroups[type]) {
            typeGroups[type] = { count: 0, size: 0, category: categoryName };
        }
        typeGroups[type].count++;
        typeGroups[type].size += alloc.size || 0;
        
        // Add to category groups
        if (!categoryGroups[categoryName].types[type]) {
            categoryGroups[categoryName].types[type] = { count: 0, size: 0 };
        }
        categoryGroups[categoryName].types[type].count++;
        categoryGroups[categoryName].types[type].size += alloc.size || 0;
        categoryGroups[categoryName].totalSize += alloc.size || 0;
    });

    // Sort categories by size
    const sortedCategories = Object.entries(categoryGroups)
        .filter(([, data]) => data.totalSize > 0)
        .sort(([, a], [, b]) => b.totalSize - a.totalSize);

    let html = '';
    let currentY = 0;
    const containerHeight = 240;
    const padding = 8;

    sortedCategories.forEach(([categoryName, categoryData], categoryIndex) => {
        const categoryPercentage = (categoryData.totalSize / totalMemory) * 100;
        const categoryHeight = Math.max(40, (categoryPercentage / 100) * containerHeight * 0.8);
        
        // Category container with background
        html += `
            <div class="absolute w-full rounded-lg border-2 border-white shadow-sm transition-all hover:shadow-md" 
                 style="top: ${currentY}px; height: ${categoryHeight}px; background-color: ${categoryData.color}; opacity: 0.15;">
            </div>
        `;

        // Category label
        html += `
            <div class="absolute left-2 font-bold text-sm z-10" 
                 style="top: ${currentY + 8}px; color: ${categoryData.color};">
                ${categoryName} (${categoryPercentage.toFixed(1)}%)
            </div>
        `;

        // Sort types within category
        const sortedTypes = Object.entries(categoryData.types)
            .sort(([, a], [, b]) => b.size - a.size)
            .slice(0, 6); // Limit to top 6 types per category

        let currentX = 20;
        const typeY = currentY + 25;
        const availableWidth = 95; // Leave some margin

        sortedTypes.forEach(([type, typeData], typeIndex) => {
            const typePercentage = (typeData.size / categoryData.totalSize) * 100;
            const typeWidth = Math.max(60, (typePercentage / 100) * availableWidth);
            const typeHeight = Math.max(20, categoryHeight - 35);

            // Type rectangle with enhanced styling
            html += `
                <div class="absolute rounded-md border border-white shadow-sm cursor-pointer transition-all hover:brightness-110 hover:scale-105 hover:z-20" 
                     style="left: ${currentX}px; top: ${typeY}px; width: ${typeWidth}px; height: ${typeHeight}px; 
                            background-color: ${categoryData.color}; opacity: 0.9;"
                     title="${type}: ${formatBytes(typeData.size)} (${typeData.count} allocs, ${typePercentage.toFixed(1)}% of ${categoryName})">
                    <div class="p-1 h-full flex flex-col justify-center text-white text-xs font-bold text-center">
                        <div class="truncate text-shadow" style="text-shadow: 1px 1px 2px rgba(0,0,0,0.8);">
                            ${type.length > 12 ? type.substring(0, 10) + '..' : type}
                        </div>
                        <div class="text-xs opacity-90 font-semibold" style="text-shadow: 1px 1px 2px rgba(0,0,0,0.6);">
                            ${formatBytes(typeData.size)}
                        </div>
                        <div class="text-xs opacity-75" style="text-shadow: 1px 1px 2px rgba(0,0,0,0.6);">
                            (${typePercentage.toFixed(1)}%)
                        </div>
                    </div>
                </div>
            `;

            currentX += typeWidth + 4;
        });

        currentY += categoryHeight + padding;
    });

    return html;
}

// Helper function to get category name
function getCategoryName(category) {
    const categoryMap = {
        'collections': 'Collections',
        'basic': 'Basic Types',
        'smart_pointers': 'Smart Pointers',
        'system': 'System'
    };
    return categoryMap[category] || 'System';
}

// Create advanced fragmentation bar
function createAdvancedFragmentationBar(label, count, total, color) {
    const percentage = total > 0 ? (count / total) * 100 : 0;
    const barHeight = Math.max(8, (count / total) * 60);

    return `
        <div class="flex items-center justify-between">
            <div class="flex items-center w-32">
                <div class="w-4 rounded mr-3 border border-gray-300 dark:border-gray-500" 
                     style="height: ${barHeight}px; background-color: ${color}"></div>
                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">${label}</span>
            </div>
            <div class="flex-1 mx-3">
                <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-3">
                    <div class="h-3 rounded-full transition-all duration-500" 
                         style="width: ${percentage}%; background-color: ${color}"></div>
                </div>
            </div>
            <span class="text-sm font-bold text-gray-900 dark:text-white w-12 text-right">${count}</span>
        </div>
    `;
}

// Create call stack analysis
function createCallStackAnalysis(allocations) {
    const userAllocs = allocations.filter(a => a.var_name && a.var_name !== 'unknown');
    const systemAllocs = allocations.filter(a => !a.var_name || a.var_name === 'unknown');

    const topAllocations = [...userAllocs, ...systemAllocs.slice(0, 3)]
        .sort((a, b) => (b.size || 0) - (a.size || 0))
        .slice(0, 10);

    return topAllocations.map(alloc => {
        const isSystem = !alloc.var_name || alloc.var_name === 'unknown';
        const color = isSystem ? '#e74c3c' : getTypeColor(alloc.type_name || '', 0);
        const radius = Math.min(8, Math.max(3, Math.sqrt((alloc.size || 0) / 100)));

        return `
            <div class="flex items-center space-x-3 p-2 bg-white dark:bg-gray-600 rounded border">
                <div class="w-4 h-4 rounded-full border-2 border-gray-300 dark:border-gray-500" 
                     style="background-color: ${color}"></div>
                <div class="flex-1 min-w-0">
                    <div class="text-sm font-medium text-gray-900 dark:text-white truncate">
                        ${alloc.var_name || 'System/Runtime allocations'}
                    </div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">
                        ${alloc.type_name || 'no type info'} • ${formatBytes(alloc.size || 0)}
                    </div>
                </div>
            </div>
        `;
    }).join('');
}

// Create advanced growth trend visualization
function createAdvancedGrowthTrendVisualization(allocations, totalMemory) {
    if (allocations.length < 2) return '<div class="flex items-center justify-center h-full text-gray-400">Insufficient data</div>';

    const sortedAllocs = allocations.sort((a, b) => (a.timestamp_alloc || 0) - (b.timestamp_alloc || 0));
    const points = [];
    let cumulativeSize = 0;

    sortedAllocs.forEach((alloc, index) => {
        cumulativeSize += alloc.size || 0;
        if (index % Math.max(1, Math.floor(sortedAllocs.length / 15)) === 0) {
            points.push({
                x: (index / sortedAllocs.length) * 100,
                y: 100 - (cumulativeSize / totalMemory) * 80,
                size: cumulativeSize
            });
        }
    });

    return `
        <!-- Background Grid -->
        <div class="absolute inset-0">
            ${[20, 40, 60, 80].map(y => `
                <div class="absolute w-full border-t border-gray-200 dark:border-gray-500 opacity-30" 
                     style="top: ${y}%"></div>
            `).join('')}
        </div>
        
        <!-- Growth Line -->
        <svg class="absolute inset-0 w-full h-full">
            <polyline
                fill="none"
                stroke="#27ae60"
                stroke-width="3"
                stroke-linecap="round"
                stroke-linejoin="round"
                points="${points.map(p => `${p.x},${p.y}`).join(' ')}"
                class="drop-shadow-sm"
            />
        </svg>
        
        <!-- Data Points -->
        ${points.map(point => `
            <div class="absolute w-2 h-2 bg-green-500 rounded-full border border-white dark:border-gray-600 transform -translate-x-1/2 -translate-y-1/2 hover:scale-150 transition-transform cursor-pointer" 
                 style="left: ${point.x}%; top: ${point.y}%"
                 title="Memory: ${formatBytes(point.size)}">
            </div>
        `).join('')}
        
        <!-- Peak Memory Line -->
        <div class="absolute w-full border-t-2 border-red-500 border-dashed opacity-60" style="top: 20%">
            <div class="absolute -top-1 right-0 text-xs text-red-500 bg-white dark:bg-gray-600 px-1 rounded">
                Peak: ${formatBytes(totalMemory)}
            </div>
        </div>
    `;
}

// Create variable allocation timeline
function createVariableAllocationTimeline(allocations) {
    const userAllocs = allocations.filter(a => a.var_name && a.var_name !== 'unknown')
        .sort((a, b) => (a.timestamp_alloc || 0) - (b.timestamp_alloc || 0))
        .slice(0, 10);

    return userAllocs.map((alloc, index) => {
        const color = getTypeColor(alloc.type_name || '', index);

        return `
            <div class="flex items-center space-x-3 p-2 bg-white dark:bg-gray-600 rounded border">
                <div class="w-3 h-3 rounded-full" style="background-color: ${color}"></div>
                <div class="flex-1 min-w-0">
                    <div class="text-sm font-medium text-gray-900 dark:text-white">
                        ${alloc.var_name}
                    </div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">
                        ${alloc.type_name || 'unknown'} • ${formatBytes(alloc.size || 0)}
                    </div>
                </div>
                <div class="text-xs text-gray-500 dark:text-gray-400">
                    ${new Date(alloc.timestamp_alloc / 1000000).toLocaleTimeString()}
                </div>
            </div>
        `;
    }).join('');
}

// Helper functions for type categorization
function getTypeCategory(type) {
    if (!type || type === 'System' || type === 'unknown') return 'system';
    
    const typeLower = type.toLowerCase();
    
    // Collections
    if (typeLower.includes('vec') || typeLower.includes('hash') || typeLower.includes('btree') || 
        typeLower.includes('deque') || typeLower.includes('set') || typeLower.includes('map')) {
        return 'collections';
    }
    
    // Smart Pointers
    if (typeLower.includes('box') || typeLower.includes('rc') || typeLower.includes('arc') || 
        typeLower.includes('refcell') || typeLower.includes('cell') || typeLower.includes('weak')) {
        return 'smart_pointers';
    }
    
    // Basic types (String, primitives, etc.)
    return 'basic';
}

function getCategoryColor(category) {
    const colors = {
        'collections': '#3498db',      // Bright blue
        'basic': '#27ae60',           // Bright green  
        'smart_pointers': '#9b59b6',  // Purple
        'system': '#95a5a6'           // Gray
    };
    return colors[category] || '#95a5a6';
}

// Initialize allocations table with improved collapsible functionality
function initAllocationsTable() {
    console.log('📊 Initializing allocations table...');

    const tbody = document.getElementById('allocations-table');
    const toggleButton = document.getElementById('toggle-allocations');

    if (!tbody) {
        console.warn('⚠️ Allocations table body not found');
        return;
    }

    const allocations = window.analysisData.memory_analysis?.allocations || [];

    if (allocations.length === 0) {
        tbody.innerHTML = '<tr><td colspan="5" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No allocations found</td></tr>';
        if (toggleButton) {
            toggleButton.style.display = 'none';
        }
        return;
    }

    let isExpanded = false;
    const maxInitialRows = 5;

    function renderTable(showAll = false) {
        console.log(`📊 Rendering table, showAll: ${showAll}, total allocations: ${allocations.length}`);

        const displayAllocations = showAll ? allocations : allocations.slice(0, maxInitialRows);

        tbody.innerHTML = displayAllocations.map(alloc => `
            <tr class="hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
                <td class="px-4 py-2 text-gray-900 dark:text-gray-100 font-mono">0x${(alloc.ptr ? parseInt(alloc.ptr.toString().replace('0x', ''), 16) : 0).toString(16).padStart(8, '0')}</td>
                <td class="px-4 py-2 text-gray-900 dark:text-gray-100">${alloc.var_name || 'System Allocation'}</td>
                <td class="px-4 py-2 text-gray-900 dark:text-gray-100">${formatTypeName(alloc.type_name || 'System Allocation')}</td>
                <td class="px-4 py-2 text-right text-gray-900 dark:text-gray-100">${formatBytes(alloc.size || 0)}</td>
                <td class="px-4 py-2 text-right text-gray-900 dark:text-gray-100">
                    <span class="px-2 py-1 text-xs rounded-full ${alloc.is_active ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200' : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'}">
                        ${alloc.is_active ? 'Active' : 'Deallocated'}
                    </span>
                </td>
            </tr>
        `).join('');

        if (!showAll && allocations.length > maxInitialRows) {
            tbody.innerHTML += `
                <tr class="bg-gray-50 dark:bg-gray-700">
                    <td colspan="5" class="px-4 py-2 text-center text-gray-500 dark:text-gray-400 text-sm">
                        ... and ${allocations.length - maxInitialRows} more allocations
                    </td>
                </tr>
            `;
        }
    }

    // Initial render
    renderTable(false);

    // Toggle functionality - Fixed event binding
    if (toggleButton && allocations.length > maxInitialRows) {
        console.log('📊 Setting up toggle button for', allocations.length, 'allocations');

        // Clear any existing event listeners and add new one
        toggleButton.replaceWith(toggleButton.cloneNode(true));
        const newToggleButton = document.getElementById('toggle-allocations');

        newToggleButton.addEventListener('click', function (e) {
            e.preventDefault();
            e.stopPropagation();
            console.log('📊 Toggle button clicked, current state:', isExpanded);

            isExpanded = !isExpanded;
            renderTable(isExpanded);

            const icon = newToggleButton.querySelector('i');
            const text = newToggleButton.querySelector('span');

            if (isExpanded) {
                icon.className = 'fa fa-chevron-up mr-1';
                text.textContent = 'Show Less';
                console.log('📊 Expanded table to show all allocations');
            } else {
                icon.className = 'fa fa-chevron-down mr-1';
                text.textContent = 'Show All';
                console.log('📊 Collapsed table to show first', maxInitialRows, 'allocations');
            }
        });

        console.log('✅ Toggle button initialized successfully');
    } else if (toggleButton) {
        // Hide button if not needed
        toggleButton.style.display = 'none';
        console.log('📊 Toggle button hidden (not enough data)');
    }
}

// Initialize lifetime visualization from JSON data with collapsible functionality
function initLifetimeVisualization() {
    console.log('🔄 Initializing lifetime visualization...');

    // Get lifetime data from various sources (support extended data structure)
    let lifetimeData = null;
    let lifecycleEvents = [];
    
    // Smart data source selection: merge memory_analysis and complex_types data
    let memoryAllocations = window.analysisData.memory_analysis?.allocations || [];
    let complexAllocations = window.analysisData.complex_types?.allocations || [];
    
    console.log('📊 Memory analysis allocations:', memoryAllocations.length);
    console.log('📊 Complex types allocations:', complexAllocations.length);
    
    // 合并数据：使用memory_analysis的lifetime_ms + complex_types的扩展字段
    if (memoryAllocations.length > 0 && complexAllocations.length > 0) {
        // Create mapping from pointer to memory analysis data
        const memoryMap = new Map();
        memoryAllocations.forEach(alloc => {
            if (alloc.ptr) {
                memoryMap.set(alloc.ptr, alloc);
            }
        });
        
        // Merge data: complex_types + lifetime_ms from memory_analysis
        lifecycleEvents = complexAllocations.map(complexAlloc => {
            const memoryAlloc = memoryMap.get(complexAlloc.ptr);
            return {
                ...complexAlloc,
                lifetime_ms: memoryAlloc?.lifetime_ms || null,
                timestamp_dealloc: memoryAlloc?.timestamp_dealloc || null
            };
        });
        console.log('📊 Merged allocation data:', lifecycleEvents.length);
    } else if (memoryAllocations.length > 0) {
        lifecycleEvents = memoryAllocations;
        console.log('📊 Using memory analysis data:', lifecycleEvents.length);
    } else if (complexAllocations.length > 0) {
        lifecycleEvents = complexAllocations;
        console.log('📊 Using complex types data:', lifecycleEvents.length);
    } else if (window.analysisData.lifetime?.lifecycle_events) {
        lifecycleEvents = window.analysisData.lifetime.lifecycle_events;
        console.log('📊 Using lifetime events data:', lifecycleEvents.length);
    }
    
    if (!lifecycleEvents || lifecycleEvents.length === 0) {
        console.warn('⚠️ No lifetime data found');
        console.log('Available data keys:', Object.keys(window.analysisData || {}));
        showEmptyLifetimeState();
        return;
    }

    console.log(`📊 Total lifecycle events: ${lifecycleEvents.length}`);

    // Check if we have Rust-preprocessed data
    if (lifetimeData?.visualization_ready && lifetimeData?.variable_groups) {
        console.log(`📊 Using Rust-preprocessed data with ${lifetimeData.variable_groups.length} variable groups`);
        renderLifetimeVisualizationFromRustWithCollapse(lifetimeData.variable_groups);
        return;
    }

    // Filter for user-defined variables (non-unknown var_name and type_name)
    const userVariables = lifecycleEvents.filter(event =>
        event.var_name && event.var_name !== 'unknown' &&
        event.type_name && event.type_name !== 'unknown'
    );

    console.log(`📊 Found ${userVariables.length} user-defined variables in lifetime data`);

    // Debug: Show some examples of what we found
    if (userVariables.length > 0) {
        console.log('📊 Sample user variables:', userVariables.slice(0, 3));
    } else {
        // Show some examples of unknown variables for debugging
        const unknownSamples = lifecycleEvents.slice(0, 3);
        console.log('📊 Sample unknown variables:', unknownSamples);
    }

    if (userVariables.length === 0) {
        showEmptyLifetimeState();
        return;
    }

    // Group by variable name to get allocation/deallocation pairs
    const variableGroups = groupVariablesByName(userVariables);

    // Render the lifetime visualization with collapse functionality
    renderLifetimeVisualizationWithCollapse(variableGroups);
}

// Group variables by name to track their lifecycle (enhanced for multiple instances)
function groupVariablesByName(events) {
    const groups = {};

    events.forEach(event => {
        const varName = event.var_name;
        const instanceKey = `${varName}_${event.ptr || event.timestamp_alloc}`; // 为每个实例创建唯一key
        
        if (!groups[instanceKey]) {
            groups[instanceKey] = {
                var_name: `${varName}#${Object.keys(groups).filter(k => k.startsWith(varName)).length + 1}`, // 添加实例编号
                original_var_name: varName,
                type_name: event.type_name,
                events: [],
                instance_info: {
                    ptr: event.ptr,
                    timestamp: event.timestamp_alloc,
                    thread_id: event.thread_id
                }
            };
        }
        groups[instanceKey].events.push(event);
    });

    
    const groupValues = Object.values(groups);
    const varCounts = {};
    groupValues.forEach(group => {
        const originalName = group.original_var_name;
        varCounts[originalName] = (varCounts[originalName] || 0) + 1;
    });
    
    groupValues.forEach(group => {
        if (varCounts[group.original_var_name] === 1) {
            group.var_name = group.original_var_name; 
        }
    });

    return groupValues;
}

// Render lifetime visualization from Rust-preprocessed data with collapsible functionality
function renderLifetimeVisualizationFromRustWithCollapse(variableGroups) {
    console.log(`📊 Rendering ${variableGroups.length} Rust-preprocessed variable groups with collapse functionality`);

    const container = document.getElementById('lifetimeVisualization');
    const toggleButton = document.getElementById('toggle-lifecycle');
    
    if (!container) return;

    // Clear loading state
    container.innerHTML = '';

    if (!variableGroups || variableGroups.length === 0) {
        showEmptyLifetimeState();
        if (toggleButton) {
            toggleButton.style.display = 'none';
        }
        return;
    }

    let isExpanded = false;
    const maxInitialRows = 5;

    // Calculate timeline bounds from preprocessed data
    const allTimestamps = variableGroups.flatMap(group =>
        group.events ? group.events.map(e => e.timestamp) : [group.start_time, group.end_time].filter(t => t !== undefined)
    );

    const minTime = Math.min(...allTimestamps);
    const maxTime = Math.max(...allTimestamps);
    const timeRange = maxTime - minTime || 1;

    console.log(`📊 Rust data timeline: ${minTime} to ${maxTime} (range: ${timeRange})`);

    // Color palette for different data types and visualizations
    const COLOR_PALETTE = {
        progress: [
            '#ff6b6b', '#4ecdc4', '#45b7d1', '#96ceb4', '#feca57',
            '#ff9ff3', '#54a0ff', '#5f27cd', '#00d2d3', '#ff9f43'
        ]
    };

    function renderLifetimeRows(showAll = false) {
        console.log(`📊 Rendering lifecycle rows, showAll: ${showAll}, total groups: ${variableGroups.length}`);
        
        container.innerHTML = '';
        
        const displayGroups = showAll ? variableGroups : variableGroups.slice(0, maxInitialRows);

        // Render each variable with colorful progress bars
        displayGroups.forEach((group, index) => {
            const varDiv = document.createElement('div');
            varDiv.className = 'flex items-center py-4 border-b border-gray-100 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors';

            // Get color from palette (cycle through colors)
            const colorIndex = index % COLOR_PALETTE.progress.length;
            const progressColor = COLOR_PALETTE.progress[colorIndex];

            // Use preprocessed timing data or fallback to events
            const startTime = group.start_time || (group.events && group.events[0] ? group.events[0].timestamp : minTime);
            const firstEvent = group.events && group.events[0];
            
            const startPercent = timeRange > 0 ? ((startTime - minTime) / timeRange) * 100 : 0;
            
            
            let widthPercent;
            if (firstEvent && firstEvent.lifetime_ms && firstEvent.lifetime_ms > 0) {
                
                const lifetimeNs = firstEvent.lifetime_ms * 1000000; 
                widthPercent = timeRange > 0 ? Math.max(1, (lifetimeNs / timeRange) * 100) : 6.8;
            } else {
                //
                widthPercent = 6.8;
            }
            
            // 安全的变量定义，防止NaN
            const finalStartPercent = isNaN(startPercent) ? 0 : Math.max(0, Math.min(95, startPercent));
            const finalWidthPercent = isNaN(widthPercent) ? 40 : Math.max(2, Math.min(100 - finalStartPercent, widthPercent));

            // Format type name for display
            const displayTypeName = formatTypeName(group.type_name);

            // Create gradient background for more visual appeal
            const gradientStyle = `background: linear-gradient(90deg, ${progressColor}, ${progressColor}dd);`;

            varDiv.innerHTML = `
                <div class="w-48 flex-shrink-0 pr-4">
                    <div class="text-sm font-semibold text-gray-800 dark:text-gray-200">${group.var_name}</div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">${displayTypeName}</div>
                </div>
                <div class="flex-grow relative bg-gray-200 dark:bg-gray-600 rounded-full h-6 overflow-hidden">
                    <div class="absolute inset-0 rounded-full" 
                         style="${gradientStyle} width: ${finalWidthPercent}%; margin-left: ${finalStartPercent}%; 
                                box-shadow: 0 2px 4px rgba(0,0,0,0.1); 
                                transition: all 0.3s ease;"
                         title="Variable: ${group.var_name}, Type: ${displayTypeName}">
                        <div class="absolute inset-0 flex items-center justify-center">
                            <span class="text-xs font-bold text-white drop-shadow-sm">
                                ${Math.round(finalWidthPercent)}%
                            </span>
                        </div>
                    </div>
                    <div class="absolute -top-8 left-0 text-xs bg-gray-700 text-white px-2 py-1 rounded opacity-0 hover:opacity-100 transition-opacity whitespace-nowrap">
                        Duration: ${firstEvent && firstEvent.lifetime_ms ? firstEvent.lifetime_ms + 'ms' : 'Active'}
                    </div>
                </div>
                <div class="w-20 flex-shrink-0 pl-4 text-right">
                    <div class="text-xs text-gray-600 dark:text-gray-400">
                        ${formatBytes(group.size || (group.events && group.events[0] ? group.events[0].size : 0) || 0)}
                    </div>
                </div>
            `;

            container.appendChild(varDiv);
        });

        // Add "show more" indicator if collapsed
        if (!showAll && variableGroups.length > maxInitialRows) {
            const moreDiv = document.createElement('div');
            moreDiv.className = 'flex items-center py-4 bg-gray-50 dark:bg-gray-700 border-b border-gray-100 dark:border-gray-600';
            moreDiv.innerHTML = `
                <div class="w-full text-center text-gray-500 dark:text-gray-400 text-sm">
                    ... and ${variableGroups.length - maxInitialRows} more variables
                </div>
            `;
            container.appendChild(moreDiv);
        }
    }

    // Initial render
    renderLifetimeRows(false);

    // Toggle functionality
    if (toggleButton && variableGroups.length > maxInitialRows) {
        console.log('📊 Setting up lifecycle toggle button for', variableGroups.length, 'variables');

        // Remove any existing event listeners
        const newToggleButton = toggleButton.cloneNode(true);
        toggleButton.parentNode.replaceChild(newToggleButton, toggleButton);

        newToggleButton.addEventListener('click', function (e) {
            e.preventDefault();
            console.log('📊 Lifecycle toggle button clicked, current state:', isExpanded);

            isExpanded = !isExpanded;
            renderLifetimeRows(isExpanded);

            const icon = newToggleButton.querySelector('i');
            const text = newToggleButton.querySelector('span');

            if (isExpanded) {
                icon.className = 'fa fa-chevron-up mr-1';
                text.textContent = 'Show Less';
                console.log('📊 Expanded lifecycle to show all variables');
            } else {
                icon.className = 'fa fa-chevron-down mr-1';
                text.textContent = 'Show All';
                console.log('📊 Collapsed lifecycle to show first', maxInitialRows, 'variables');
            }
        });

        console.log('✅ Lifecycle toggle button initialized successfully');
    } else if (toggleButton) {
        // Hide button if not needed
        toggleButton.style.display = 'none';
        console.log('📊 Lifecycle toggle button hidden (not enough data)');
    }

    console.log(`✅ Rendered ${variableGroups.length} Rust-preprocessed variables in lifetime visualization with collapse functionality`);
}

// Render the lifetime visualization with collapsible functionality
function renderLifetimeVisualizationWithCollapse(variableGroups) {
    const container = document.getElementById('lifetimeVisualization');
    const toggleButton = document.getElementById('toggle-lifecycle');
    
    if (!container) return;

    // Clear loading state
    container.innerHTML = '';

    if (!variableGroups || variableGroups.length === 0) {
        showEmptyLifetimeState();
        if (toggleButton) {
            toggleButton.style.display = 'none';
        }
        return;
    }

    let isExpanded = false;
    const maxInitialRows = 5;

    // Get color scheme for different types
    const typeColors = {
        'Vec': { bg: 'bg-blue-500', border: 'border-blue-500' },
        'Box': { bg: 'bg-purple-500', border: 'border-purple-500' },
        'Rc': { bg: 'bg-yellow-500', border: 'border-yellow-500' },
        'Arc': { bg: 'bg-green-500', border: 'border-green-500' },
        'String': { bg: 'bg-pink-500', border: 'border-pink-500' },
        'default': { bg: 'bg-gray-500', border: 'border-gray-500' }
    };

    // Calculate timeline bounds
    const allTimestamps = variableGroups.flatMap(group =>
        group.events.map(e => e.timestamp)
    );
    const minTime = Math.min(...allTimestamps);
    const maxTime = Math.max(...allTimestamps);
    const timeRange = maxTime - minTime;

    console.log(`📊 Timeline: ${minTime} to ${maxTime} (range: ${timeRange})`);

    function renderLifetimeRows(showAll = false) {
        console.log(`📊 Rendering lifecycle rows, showAll: ${showAll}, total groups: ${variableGroups.length}`);
        
        container.innerHTML = '';
        
        const displayGroups = showAll ? variableGroups : variableGroups.slice(0, maxInitialRows);

        // Render each variable
        displayGroups.forEach((group) => {
            const varDiv = document.createElement('div');
            varDiv.className = 'flex items-end py-3 border-b border-gray-100 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors';

            // Determine color based on type
            const typeKey = Object.keys(typeColors).find(key =>
                group.type_name.includes(key)
            ) || 'default';
            const colors = typeColors[typeKey];

            // Calculate position and width based on timestamps
            const firstEvent = group.events[0];
            const startTime = firstEvent.timestamp;
            const startPositionPercent = timeRange > 0 ? ((startTime - minTime) / timeRange) * 100 : 0;

            // real time correct time axis calculation: based on actual allocation and survival time
            const allocTime = firstEvent.timestamp;
            const deallocTime = firstEvent.timestamp_dealloc;
            const lifetimeMs = firstEvent.lifetime_ms || 1; // default 1ms lifetime
            
            // calculate survival time length (percentage)
            let durationPercent;
            if (deallocTime && deallocTime > allocTime) {
                // if there is a clear release time, use actual time span
                const actualDuration = deallocTime - allocTime;
                durationPercent = (actualDuration / timeRange) * 100;
            } else {
                // if there is no release time, use lifetime_ms calculation
                const lifetimeNs = lifetimeMs * 1000000; // convert to nanoseconds
                durationPercent = (lifetimeNs / timeRange) * 100;
            }
            
            // ensure value is within reasonable range
            const widthPercent = Math.max(0.5, Math.min(100 - startPositionPercent, durationPercent));
            
            // 安全的变量定义，防止NaN
            const finalStartPercent = isNaN(startPositionPercent) ? 0 : Math.max(0, Math.min(95, startPositionPercent));
            const finalWidthPercent = isNaN(widthPercent) ? 30 : Math.max(2, Math.min(100 - finalStartPercent, widthPercent));

            // Format type name for display
            const displayTypeName = formatTypeName(group.type_name);

            varDiv.innerHTML = `
                <div class="w-40 flex-shrink-0 text-sm font-medium dark:text-gray-200">
                    ${group.var_name} (${displayTypeName})
                </div>
                <div class="flex-grow relative">
                    <div class="lifespan-indicator ${colors.bg}" 
                         style="width: ${finalWidthPercent}%; margin-left: ${finalStartPercent}%;" 
                         title="Variable: ${group.var_name}, Type: ${displayTypeName}">
                        <div class="absolute -top-6 left-0 text-xs ${colors.bg} text-white px-2 py-1 rounded whitespace-nowrap">
                            Allocated: ${formatTimestamp(startTime, minTime)}
                        </div>
                    </div>
                </div>
            `;

            container.appendChild(varDiv);
        });

        // Add "show more" indicator if collapsed
        if (!showAll && variableGroups.length > maxInitialRows) {
            const moreDiv = document.createElement('div');
            moreDiv.className = 'flex items-center py-3 bg-gray-50 dark:bg-gray-700 border-b border-gray-100 dark:border-gray-600';
            moreDiv.innerHTML = `
                <div class="w-full text-center text-gray-500 dark:text-gray-400 text-sm">
                    ... and ${variableGroups.length - maxInitialRows} more variables
                </div>
            `;
            container.appendChild(moreDiv);
        }
    }

    // Initial render
    renderLifetimeRows(false);

    // Toggle functionality
    if (toggleButton && variableGroups.length > maxInitialRows) {
        console.log('📊 Setting up lifecycle toggle button for', variableGroups.length, 'variables');

        // Remove any existing event listeners
        const newToggleButton = toggleButton.cloneNode(true);
        toggleButton.parentNode.replaceChild(newToggleButton, toggleButton);

        newToggleButton.addEventListener('click', function (e) {
            e.preventDefault();
            console.log('📊 Lifecycle toggle button clicked, current state:', isExpanded);

            isExpanded = !isExpanded;
            renderLifetimeRows(isExpanded);

            const icon = newToggleButton.querySelector('i');
            const text = newToggleButton.querySelector('span');

            if (isExpanded) {
                icon.className = 'fa fa-chevron-up mr-1';
                text.textContent = 'Show Less';
                console.log('📊 Expanded lifecycle to show all variables');
            } else {
                icon.className = 'fa fa-chevron-down mr-1';
                text.textContent = 'Show All';
                console.log('📊 Collapsed lifecycle to show first', maxInitialRows, 'variables');
            }
        });

        console.log('✅ Lifecycle toggle button initialized successfully');
    } else if (toggleButton) {
        // Hide button if not needed
        toggleButton.style.display = 'none';
        console.log('📊 Lifecycle toggle button hidden (not enough data)');
    }

    console.log(`✅ Rendered ${variableGroups.length} variables in lifetime visualization with collapse functionality`);
}

// Initialize FFI visualization with enhanced support for improve.md fields
function initFFIVisualization() {
    console.log('🔄 Initializing FFI visualization...');

    const container = document.getElementById('ffiVisualization');
    if (!container) return;

    // Get FFI data from multiple sources with comprehensive field support
    let allocations = [];
    let unsafeReports = [];
    let memoryPassports = [];
    let ffiStatistics = {};
    
    console.log('🔍 Checking analysisData structure:', Object.keys(window.analysisData || {}));
    
    // Enhanced data extraction supporting improve.md structure
    if (window.analysisData) {
        // Debug: Show what data structure we actually have FIRST
        console.log('🔍 Available data keys:', Object.keys(window.analysisData));
        if (window.analysisData.unsafe_ffi) {
            console.log('🔍 unsafe_ffi keys:', Object.keys(window.analysisData.unsafe_ffi));
            console.log('🔍 unsafe_ffi.allocations exists:', !!window.analysisData.unsafe_ffi.allocations);
            
            // Data will be handled by initializeAnalysis function
            console.log('🔍 unsafe_ffi.allocations length:', window.analysisData.unsafe_ffi.allocations ? window.analysisData.unsafe_ffi.allocations.length : 'undefined');
        }
        
        // Try unsafe_ffi data first (improve.md structure)
        if (window.analysisData.unsafe_ffi) {
            allocations = window.analysisData.unsafe_ffi.allocations || [];
            unsafeReports = window.analysisData.unsafe_ffi.unsafe_reports || [];
            memoryPassports = window.analysisData.unsafe_ffi.memory_passports || [];
            ffiStatistics = window.analysisData.unsafe_ffi.ffi_statistics || {};
            console.log('📊 Found unsafe_ffi data - allocations:', allocations.length, 'reports:', unsafeReports.length, 'passports:', memoryPassports.length);
        }
        // Try complex_types structure (for large_scale_user files)
        else if (window.analysisData.complex_types && window.analysisData.complex_types.allocations) {
            allocations = window.analysisData.complex_types.allocations;
            console.log('📊 Found complex_types allocations:', allocations.length);
        }
        // Try direct allocations array (for files like large_scale_user_unsafe_ffi.json)
        else if (window.analysisData.allocations) {
            allocations = window.analysisData.allocations;
            console.log('📊 Found direct allocations:', allocations.length);
        }
        // Fallback to memory_analysis
        else if (window.analysisData.memory_analysis && window.analysisData.memory_analysis.allocations) {
            allocations = window.analysisData.memory_analysis.allocations;
            console.log('📊 Using memory_analysis allocations:', allocations.length);
        }
        
        // Debug: Show what data structure we actually have
        console.log('🔍 Available data keys:', Object.keys(window.analysisData));
        if (window.analysisData.unsafe_ffi) {
            console.log('🔍 unsafe_ffi keys:', Object.keys(window.analysisData.unsafe_ffi));
        }
        
        // Extract metadata if available
        const metadata = window.analysisData.metadata || {};
        console.log('📊 Metadata:', metadata);
    }

    // Filter for FFI-tracked allocations with enhanced field support
    const ffiAllocations = allocations.filter(alloc => 
        alloc.ffi_tracked === true || 
        (alloc.safety_violations && alloc.safety_violations.length > 0) ||
        alloc.ownership_history_available === true ||
        (alloc.borrow_info && (alloc.borrow_info.immutable_borrows > 0 || alloc.borrow_info.mutable_borrows > 0)) ||
        (alloc.clone_info && alloc.clone_info.clone_count > 0)
    );
    console.log('📊 Found FFI-tracked allocations:', ffiAllocations.length);
    
    // Debug: show first few allocations with improve.md fields
    if (allocations.length > 0) {
        console.log('🔍 Sample allocation with improve.md fields:', allocations[0]);
        console.log('🔍 FFI tracked allocations sample:', ffiAllocations.slice(0, 3));
        
        // Check for improve.md specific fields
        const sampleAlloc = allocations[0];
        console.log('🔍 Improve.md fields check:');
        console.log('  - borrow_info:', sampleAlloc.borrow_info);
        console.log('  - clone_info:', sampleAlloc.clone_info);
        console.log('  - ownership_history_available:', sampleAlloc.ownership_history_available);
        console.log('  - ffi_tracked:', sampleAlloc.ffi_tracked);
        console.log('  - safety_violations:', sampleAlloc.safety_violations);
    }

    // Debug: Show what we found before filtering
    console.log('🔍 Before filtering - Total allocations:', allocations.length);
    console.log('🔍 Sample allocation fields:', allocations[0] ? Object.keys(allocations[0]) : 'No allocations');
    console.log('🔍 FFI tracked count:', allocations.filter(a => a.ffi_tracked === true).length);
    console.log('🔍 Borrow info count:', allocations.filter(a => a.borrow_info).length);
    console.log('🔍 Clone info count:', allocations.filter(a => a.clone_info).length);
    
    // Enhanced rendering with improve.md support - ALWAYS show if we have any allocations
    if (allocations.length === 0) {
        container.innerHTML = createFFIEmptyState();
        return;
    }
    
    // If we have allocations but no FFI-specific ones, still show the dashboard with all data
    const displayAllocations = ffiAllocations.length > 0 ? ffiAllocations : allocations.slice(0, 20);
    console.log('🎯 Rendering FFI dashboard with:', displayAllocations.length, 'allocations,', unsafeReports.length, 'reports,', memoryPassports.length, 'passports');

    // Generate enhanced FFI analysis with improve.md fields
    try {
        if (FFI_STYLE === 'svg') {
            const boundaryEvents = window.analysisData.unsafe_ffi?.boundary_events || [];
            const unsafeAllocs = displayAllocations.filter(a => (a.safety_violations || []).length > 0).length;
            const ffiAllocs = displayAllocations.filter(a => a.ffi_tracked).length;
            const safetyViolations = displayAllocations.reduce((sum, a) => sum + ((a.safety_violations || []).length || 0), 0);
            const unsafeMemory = displayAllocations
                .filter(a => (a.safety_violations || []).length > 0)
                .reduce((sum, a) => sum + (a.size || 0), 0);

            container.innerHTML = createFFIDashboardSVG(
                unsafeAllocs,
                ffiAllocs,
                boundaryEvents.length,
                safetyViolations,
                unsafeMemory,
                displayAllocations,
                boundaryEvents,
                unsafeReports
            );
            console.log('✅ FFI SVG-style dashboard rendered');
            return;
        }
        console.log('🔄 Generating FFI analysis...');
        const ffiAnalysis = generateEnhancedFFIAnalysisWithImproveFields(displayAllocations, unsafeReports, memoryPassports, ffiStatistics);
        console.log('✅ FFI analysis generated:', ffiAnalysis);
        
        console.log('🔄 Creating FFI dashboard...');
        const dashboardHTML = createEnhancedFFIDashboardWithImproveFields(ffiAnalysis, displayAllocations, unsafeReports, memoryPassports);
        console.log('✅ Dashboard HTML created, length:', dashboardHTML.length);
        
        container.innerHTML = dashboardHTML;
        console.log('✅ Dashboard rendered successfully!');
    } catch (error) {
        console.error('❌ Error in FFI rendering:', error);
        container.innerHTML = `<div class="bg-red-100 p-4 rounded text-red-800">Error rendering FFI data: ${error.message}</div>`;
    }
}

// Generate enhanced FFI analysis with improve.md fields support
function generateEnhancedFFIAnalysisWithImproveFields(ffiAllocations, unsafeReports, memoryPassports, ffiStatistics) {
    let totalFFI = ffiAllocations.length;
    let totalViolations = 0;
    let totalMemory = 0;
    let highRiskCount = 0;
    let mediumRiskCount = 0;
    let lowRiskCount = 0;
    let totalBorrows = 0;
    let totalClones = 0;
    let leakedAllocations = 0;

    const analysisData = ffiAllocations.map(alloc => {
        const violations = alloc.safety_violations?.length || 0;
        const size = alloc.size || 0;
        
        // Enhanced borrow analysis from improve.md fields
        const borrowConflicts = alloc.borrow_info ? 
            (alloc.borrow_info.mutable_borrows > 0 && alloc.borrow_info.immutable_borrows > 0) : false;
        const totalBorrowsForAlloc = alloc.borrow_info ? 
            (alloc.borrow_info.immutable_borrows || 0) + (alloc.borrow_info.mutable_borrows || 0) : 0;
        totalBorrows += totalBorrowsForAlloc;
        
        // Enhanced clone analysis from improve.md fields
        const cloneCount = alloc.clone_info?.clone_count || 0;
        const isClone = alloc.clone_info?.is_clone || false;
        totalClones += cloneCount;
        
        // Enhanced ownership and lifecycle analysis
        const ownershipHistoryAvailable = alloc.ownership_history_available || false;
        const isLeaked = alloc.is_leaked || false;
        if (isLeaked) leakedAllocations++;
        
        // Enhanced risk calculation with improve.md fields
        let riskScore = 0;
        if (violations > 0) riskScore += 50;
        if (borrowConflicts) riskScore += 30;
        if (size > 1024) riskScore += 20;
        if (isLeaked) riskScore += 40;
        if (cloneCount > 3) riskScore += 15;
        if (totalBorrowsForAlloc > 5) riskScore += 10;
        
        let riskLevel = 'Low';
        if (riskScore >= 70) {
            riskLevel = 'High';
            highRiskCount++;
        } else if (riskScore >= 35) {
            riskLevel = 'Medium';
            mediumRiskCount++;
        } else {
            lowRiskCount++;
        }

        totalViolations += violations;
        totalMemory += size;

        return {
            ...alloc,
            riskScore,
            riskLevel,
            violations,
            borrowConflicts,
            totalBorrowsForAlloc,
            cloneCount,
            isClone,
            ownershipHistoryAvailable,
            isLeaked
        };
    });

    // Enhanced statistics from improve.md structure
    const enhancedStats = {
        boundary_crossings: ffiStatistics.boundary_crossings || 0,
        memory_violations: ffiStatistics.memory_violations || 0,
        total_ffi_calls: ffiStatistics.total_ffi_calls || 0,
        unsafe_operations: ffiStatistics.unsafe_operations || 0
    };

    return {
        totalFFI,
        totalViolations,
        totalMemory,
        highRiskCount,
        mediumRiskCount,
        lowRiskCount,
        totalBorrows,
        totalClones,
        leakedAllocations,
        analysisData,
        unsafeReports,
        memoryPassports,
        ffiStatistics: enhancedStats
    };
}

// Legacy function for backward compatibility
function generateEnhancedFFIAnalysis(ffiAllocations) {
    return generateEnhancedFFIAnalysisWithImproveFields(ffiAllocations, [], [], {});
}

// Create enhanced FFI dashboard with improve.md fields support
function createEnhancedFFIDashboardWithImproveFields(analysis, ffiAllocations, unsafeReports, memoryPassports) {
    return `
        <div class="space-y-6">
            <!-- Enhanced FFI Overview Cards with improve.md metrics -->
            <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
                <div class="bg-blue-100 dark:bg-blue-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-blue-600 dark:text-blue-300">${analysis.totalFFI}</div>
                    <div class="text-sm text-blue-700 dark:text-blue-400">FFI Allocations</div>
                </div>
                <div class="bg-red-100 dark:bg-red-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-red-600 dark:text-red-300">${analysis.highRiskCount}</div>
                    <div class="text-sm text-red-700 dark:text-red-400">High Risk</div>
                </div>
                <div class="bg-orange-100 dark:bg-orange-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-orange-600 dark:text-orange-300">${analysis.mediumRiskCount}</div>
                    <div class="text-sm text-orange-700 dark:text-orange-400">Medium Risk</div>
                </div>
                <div class="bg-green-100 dark:bg-green-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-green-600 dark:text-green-300">${analysis.lowRiskCount}</div>
                    <div class="text-sm text-green-700 dark:text-green-400">Low Risk</div>
                </div>
                <div class="bg-purple-100 dark:bg-purple-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-purple-600 dark:text-purple-300">${analysis.totalBorrows}</div>
                    <div class="text-sm text-purple-700 dark:text-purple-400">Total Borrows</div>
                </div>
                <div class="bg-indigo-100 dark:bg-indigo-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-indigo-600 dark:text-indigo-300">${analysis.totalClones}</div>
                    <div class="text-sm text-indigo-700 dark:text-indigo-400">Total Clones</div>
                </div>
            </div>

            <!-- FFI Statistics from improve.md -->
            ${analysis.ffiStatistics && Object.keys(analysis.ffiStatistics).length > 0 ? `
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6">
                    <h3 class="text-lg font-semibold mb-4 text-gray-800 dark:text-white">FFI Statistics</h3>
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                        <div class="text-center">
                            <div class="text-xl font-bold text-gray-900 dark:text-white">${analysis.ffiStatistics.boundary_crossings}</div>
                            <div class="text-sm text-gray-600 dark:text-gray-400">Boundary Crossings</div>
                        </div>
                        <div class="text-center">
                            <div class="text-xl font-bold text-gray-900 dark:text-white">${analysis.ffiStatistics.memory_violations}</div>
                            <div class="text-sm text-gray-600 dark:text-gray-400">Memory Violations</div>
                        </div>
                        <div class="text-center">
                            <div class="text-xl font-bold text-gray-900 dark:text-white">${analysis.ffiStatistics.total_ffi_calls}</div>
                            <div class="text-sm text-gray-600 dark:text-gray-400">Total FFI Calls</div>
                        </div>
                        <div class="text-center">
                            <div class="text-xl font-bold text-gray-900 dark:text-white">${analysis.ffiStatistics.unsafe_operations}</div>
                            <div class="text-sm text-gray-600 dark:text-gray-400">Unsafe Operations</div>
                        </div>
                    </div>
                </div>
            ` : ''}

            <!-- Unsafe Reports from improve.md structure -->
            ${analysis.unsafeReports && analysis.unsafeReports.length > 0 ? `
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6">
                    <h3 class="text-lg font-semibold mb-4 text-gray-800 dark:text-white">Unsafe Reports</h3>
                    <div class="space-y-4">
                        ${analysis.unsafeReports.map(report => createUnsafeReportCard(report)).join('')}
                    </div>
                </div>
            ` : ''}

            <!-- Memory Passports from improve.md structure -->
            ${analysis.memoryPassports && analysis.memoryPassports.length > 0 ? `
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6">
                    <h3 class="text-lg font-semibold mb-4 text-gray-800 dark:text-white">Memory Passports</h3>
                    <div class="space-y-3">
                        ${analysis.memoryPassports.map(passport => createMemoryPassportCard(passport)).join('')}
                    </div>
                </div>
            ` : ''}

            <!-- Enhanced FFI Risk Analysis with improve.md fields -->
            <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6">
                <h3 class="text-lg font-semibold mb-4 text-gray-800 dark:text-white">Enhanced FFI Risk Analysis</h3>
                <div class="space-y-4">
                    ${analysis.analysisData.map(alloc => createEnhancedFFIAllocationCard(alloc)).join('')}
                </div>
            </div>

            <!-- Enhanced Borrow Checker Analysis with improve.md fields -->
            <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6">
                <h3 class="text-lg font-semibold mb-4 text-gray-800 dark:text-white">Enhanced Borrow Checker Analysis</h3>
                <div class="space-y-3">
                    ${ffiAllocations.filter(alloc => alloc.borrow_info).map(alloc => createEnhancedBorrowAnalysisCard(alloc)).join('')}
                </div>
            </div>

            <!-- Clone Analysis from improve.md fields -->
            ${analysis.totalClones > 0 ? `
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6">
                    <h3 class="text-lg font-semibold mb-4 text-gray-800 dark:text-white">Clone Analysis</h3>
                    <div class="space-y-3">
                        ${ffiAllocations.filter(alloc => alloc.clone_info && alloc.clone_info.clone_count > 0).map(alloc => createCloneAnalysisCard(alloc)).join('')}
                    </div>
                </div>
            ` : ''}

            <!-- Ownership History Analysis -->
            ${ffiAllocations.some(alloc => alloc.ownership_history_available) ? `
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6">
                    <h3 class="text-lg font-semibold mb-4 text-gray-800 dark:text-white">Ownership History Analysis</h3>
                    <div class="space-y-3">
                        ${ffiAllocations.filter(alloc => alloc.ownership_history_available).map(alloc => createOwnershipHistoryCard(alloc)).join('')}
                    </div>
                </div>
            ` : ''}
        </div>
    `;
}

// Legacy function for backward compatibility
function createEnhancedFFIDashboard(analysis, ffiAllocations) {
    return createEnhancedFFIDashboardWithImproveFields(analysis, ffiAllocations, [], []);
}

// Create enhanced FFI allocation card with improve.md fields
function createEnhancedFFIAllocationCard(alloc) {
    const riskColor = alloc.riskLevel === 'High' ? 'red' : alloc.riskLevel === 'Medium' ? 'orange' : 'green';
    const hasViolations = alloc.violations > 0;
    const hasBorrowConflicts = alloc.borrowConflicts;
    const hasClones = alloc.cloneCount > 0;
    const isLeaked = alloc.isLeaked;
    const hasOwnershipHistory = alloc.ownershipHistoryAvailable;
    
    return `
        <div class="bg-white dark:bg-gray-600 rounded-lg p-4 border-l-4 border-${riskColor}-500">
            <div class="flex justify-between items-start mb-3">
                <div>
                    <h4 class="font-semibold text-gray-900 dark:text-white">${alloc.var_name || 'Unknown Variable'}</h4>
                    <p class="text-sm text-gray-600 dark:text-gray-300">${formatTypeName(alloc.type_name || 'Unknown Type')}</p>
                    ${alloc.isClone ? '<span class="inline-block px-2 py-1 text-xs bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200 rounded-full mt-1">Clone</span>' : ''}
                </div>
                <div class="text-right">
                    <span class="px-2 py-1 text-xs font-bold rounded-full bg-${riskColor}-100 text-${riskColor}-800 dark:bg-${riskColor}-900 dark:text-${riskColor}-200">
                        ${alloc.riskLevel} Risk
                    </span>
                    ${isLeaked ? '<div class="mt-1"><span class="px-2 py-1 text-xs bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200 rounded-full">LEAKED</span></div>' : ''}
                </div>
            </div>
            
            <div class="grid grid-cols-2 gap-4 text-sm mb-3">
                <div>
                    <span class="text-gray-500 dark:text-gray-400">Size:</span>
                    <span class="ml-2 font-mono">${formatBytes(alloc.size || 0)}</span>
                </div>
                <div>
                    <span class="text-gray-500 dark:text-gray-400">Risk Score:</span>
                    <span class="ml-2 font-bold text-${riskColor}-600">${alloc.riskScore}/100</span>
                </div>
                <div>
                    <span class="text-gray-500 dark:text-gray-400">Pointer:</span>
                    <span class="ml-2 font-mono text-xs">${alloc.ptr}</span>
                </div>
                <div>
                    <span class="text-gray-500 dark:text-gray-400">Thread:</span>
                    <span class="ml-2">${alloc.thread_id || 'Unknown'}</span>
                </div>
            </div>

            <!-- Enhanced improve.md fields -->
            <div class="grid grid-cols-3 gap-4 text-sm mb-3">
                <div>
                    <span class="text-gray-500 dark:text-gray-400">Total Borrows:</span>
                    <span class="ml-2 font-bold">${alloc.totalBorrowsForAlloc || 0}</span>
                </div>
                <div>
                    <span class="text-gray-500 dark:text-gray-400">Clone Count:</span>
                    <span class="ml-2 font-bold">${alloc.cloneCount || 0}</span>
                </div>
                <div>
                    <span class="text-gray-500 dark:text-gray-400">FFI Tracked:</span>
                    <span class="ml-2">${alloc.ffi_tracked ? '✅' : '❌'}</span>
                </div>
            </div>
            
            ${hasViolations || hasBorrowConflicts || hasClones || hasOwnershipHistory ? `
                <div class="mt-3 pt-3 border-t border-gray-200 dark:border-gray-500">
                    <div class="text-sm space-y-1">
                        ${hasViolations ? `<div class="text-red-600 dark:text-red-400">⚠️ ${alloc.violations} safety violations</div>` : ''}
                        ${hasBorrowConflicts ? `<div class="text-orange-600 dark:text-orange-400">⚠️ Borrow conflicts detected</div>` : ''}
                        ${hasClones ? `<div class="text-blue-600 dark:text-blue-400">🔄 ${alloc.cloneCount} clones created</div>` : ''}
                        ${hasOwnershipHistory ? `<div class="text-green-600 dark:text-green-400">📋 Ownership history available</div>` : ''}
                    </div>
                </div>
            ` : ''}
        </div>
    `;
}

// Legacy function for backward compatibility
function createFFIAllocationCard(alloc) {
    return createEnhancedFFIAllocationCard(alloc);
}

// Create enhanced borrow analysis card with improve.md fields
function createEnhancedBorrowAnalysisCard(alloc) {
    const borrowInfo = alloc.borrow_info;
    const hasConflict = borrowInfo.mutable_borrows > 0 && borrowInfo.immutable_borrows > 0;
    const lastBorrowTime = borrowInfo.last_borrow_timestamp ? new Date(borrowInfo.last_borrow_timestamp / 1000000).toLocaleTimeString() : 'Unknown';
    
    return `
        <div class="bg-white dark:bg-gray-600 rounded-lg p-3 ${hasConflict ? 'border-l-4 border-red-500' : 'border border-gray-200 dark:border-gray-500'}">
            <div class="flex justify-between items-start">
                <div>
                    <h5 class="font-medium text-gray-900 dark:text-white">${alloc.var_name}</h5>
                    <p class="text-xs text-gray-500 dark:text-gray-400">${formatTypeName(alloc.type_name)}</p>
                    <p class="text-xs text-gray-500 dark:text-gray-400">Last borrow: ${lastBorrowTime}</p>
                </div>
                <div class="text-right text-sm">
                    <div class="text-blue-600 dark:text-blue-400">Immutable: ${borrowInfo.immutable_borrows}</div>
                    <div class="text-red-600 dark:text-red-400">Mutable: ${borrowInfo.mutable_borrows}</div>
                    <div class="text-purple-600 dark:text-purple-400">Max Concurrent: ${borrowInfo.max_concurrent_borrows}</div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">Total: ${(borrowInfo.immutable_borrows || 0) + (borrowInfo.mutable_borrows || 0)}</div>
                </div>
            </div>
            ${hasConflict ? `
                <div class="mt-2 text-xs text-red-600 dark:text-red-400 font-bold">
                    ⚠️ BORROW CONFLICT: Simultaneous mutable and immutable borrows detected
                </div>
            ` : ''}
        </div>
    `;
}

// Legacy function for backward compatibility
function createBorrowAnalysisCard(alloc) {
    return createEnhancedBorrowAnalysisCard(alloc);
}

// Create clone analysis card for improve.md clone_info fields
function createCloneAnalysisCard(alloc) {
    const cloneInfo = alloc.clone_info;
    const isClone = cloneInfo.is_clone;
    const cloneCount = cloneInfo.clone_count;
    const originalPtr = cloneInfo.original_ptr;
    
    return `
        <div class="bg-white dark:bg-gray-600 rounded-lg p-3 border-l-4 border-blue-500">
            <div class="flex justify-between items-start">
                <div>
                    <h5 class="font-medium text-gray-900 dark:text-white">${alloc.var_name}</h5>
                    <p class="text-xs text-gray-500 dark:text-gray-400">${formatTypeName(alloc.type_name)}</p>
                    ${isClone ? `<p class="text-xs text-blue-600 dark:text-blue-400">Clone of: ${originalPtr}</p>` : ''}
                </div>
                <div class="text-right">
                    <div class="text-blue-600 dark:text-blue-400 font-bold text-lg">${cloneCount}</div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">Clones Created</div>
                    ${isClone ? '<div class="text-xs bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200 px-2 py-1 rounded mt-1">IS CLONE</div>' : ''}
                </div>
            </div>
            <div class="mt-2 text-sm text-gray-600 dark:text-gray-300">
                ${cloneCount > 0 ? `🔄 This allocation has been cloned ${cloneCount} times` : ''}
                ${isClone ? `<br>📋 This is a clone of allocation at ${originalPtr}` : ''}
            </div>
        </div>
    `;
}

// Create ownership history card for improve.md ownership_history_available field
function createOwnershipHistoryCard(alloc) {
    return `
        <div class="bg-white dark:bg-gray-600 rounded-lg p-3 border-l-4 border-green-500">
            <div class="flex justify-between items-center">
                <div>
                    <h5 class="font-medium text-gray-900 dark:text-white">${alloc.var_name}</h5>
                    <p class="text-xs text-gray-500 dark:text-gray-400">${formatTypeName(alloc.type_name)}</p>
                </div>
                <div class="text-right">
                    <div class="text-green-600 dark:text-green-400">📋 History Available</div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">Detailed tracking enabled</div>
                </div>
            </div>
            <div class="mt-2 text-sm text-gray-600 dark:text-gray-300">
                ✅ Ownership history is available for this allocation in lifetime.json
            </div>
        </div>
    `;
}

// Create unsafe report card for improve.md UnsafeReport structure
function createUnsafeReportCard(report) {
    const riskLevel = report.risk_assessment?.risk_level || 'Unknown';
    const riskColor = riskLevel === 'High' ? 'red' : riskLevel === 'Medium' ? 'orange' : 'green';
    const confidenceScore = report.risk_assessment?.confidence_score || 0;
    const riskFactors = report.risk_assessment?.risk_factors || [];
    const dynamicViolations = report.dynamic_violations || [];
    
    return `
        <div class="bg-white dark:bg-gray-600 rounded-lg p-4 border-l-4 border-${riskColor}-500">
            <div class="flex justify-between items-start mb-3">
                <div>
                    <h4 class="font-semibold text-gray-900 dark:text-white">Unsafe Report: ${report.report_id || 'Unknown'}</h4>
                    <p class="text-sm text-gray-600 dark:text-gray-300">${report.source?.type || 'Unknown'} at ${report.source?.location || 'Unknown location'}</p>
                </div>
                <div class="text-right">
                    <span class="px-2 py-1 text-xs font-bold rounded-full bg-${riskColor}-100 text-${riskColor}-800 dark:bg-${riskColor}-900 dark:text-${riskColor}-200">
                        ${riskLevel} Risk
                    </span>
                    <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">Confidence: ${(confidenceScore * 100).toFixed(1)}%</div>
                </div>
            </div>
            
            ${riskFactors.length > 0 ? `
                <div class="mb-3">
                    <h5 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Risk Factors:</h5>
                    <div class="space-y-1">
                        ${riskFactors.map(factor => `
                            <div class="text-sm">
                                <span class="font-medium text-${riskColor}-600 dark:text-${riskColor}-400">${factor.factor_type}</span>
                                <span class="text-gray-600 dark:text-gray-400"> (Severity: ${factor.severity}/10)</span>
                                <div class="text-xs text-gray-500 dark:text-gray-400">${factor.description}</div>
                            </div>
                        `).join('')}
                    </div>
                </div>
            ` : ''}
            
            ${dynamicViolations.length > 0 ? `
                <div class="mt-3 pt-3 border-t border-gray-200 dark:border-gray-500">
                    <h5 class="text-sm font-medium text-red-700 dark:text-red-300 mb-2">Dynamic Violations:</h5>
                    <div class="space-y-1">
                        ${dynamicViolations.map(violation => `
                            <div class="text-sm text-red-600 dark:text-red-400">
                                ⚠️ ${violation.violation_type}: ${violation.description}
                            </div>
                        `).join('')}
                    </div>
                </div>
            ` : ''}
        </div>
    `;
}

// Create memory passport card for improve.md MemoryPassport structure
function createMemoryPassportCard(passport) {
    const status = passport.status_at_shutdown || 'Unknown';
    const statusColor = status === 'Reclaimed' ? 'green' : status === 'InForeignCustody' ? 'red' : 'orange';
    const lifecycleEvents = passport.lifecycle_events || [];
    
    return `
        <div class="bg-white dark:bg-gray-600 rounded-lg p-3 border-l-4 border-${statusColor}-500">
            <div class="flex justify-between items-start">
                <div>
                    <h5 class="font-medium text-gray-900 dark:text-white">Passport: ${passport.passport_id || 'Unknown'}</h5>
                    <p class="text-xs text-gray-500 dark:text-gray-400">Allocation: ${passport.allocation_ptr} (${formatBytes(passport.size_bytes || 0)})</p>
                </div>
                <div class="text-right">
                    <span class="px-2 py-1 text-xs font-bold rounded-full bg-${statusColor}-100 text-${statusColor}-800 dark:bg-${statusColor}-900 dark:text-${statusColor}-200">
                        ${status}
                    </span>
                </div>
            </div>
            
            ${lifecycleEvents.length > 0 ? `
                <div class="mt-2">
                    <h6 class="text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Lifecycle Events:</h6>
                    <div class="space-y-1">
                        ${lifecycleEvents.slice(0, 3).map(event => `
                            <div class="text-xs text-gray-600 dark:text-gray-400">
                                📅 ${event.event_type} ${event.how ? `(${event.how})` : ''}
                            </div>
                        `).join('')}
                        ${lifecycleEvents.length > 3 ? `<div class="text-xs text-gray-500 dark:text-gray-400">... and ${lifecycleEvents.length - 3} more events</div>` : ''}
                    </div>
                </div>
            ` : ''}
        </div>
    `;
}

// Create FFI empty state
function createFFIEmptyState() {
    return `
        <div class="text-center py-8">
            <div class="mb-4">
                <svg class="w-16 h-16 mx-auto text-green-400 dark:text-green-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"></path>
                </svg>
            </div>
            <h4 class="text-lg font-semibold mb-2 text-gray-800 dark:text-gray-200">Memory Safety Verified</h4>
            <p class="text-sm text-gray-600 dark:text-gray-400">No unsafe FFI operations detected in this analysis</p>
            <p class="text-xs mt-2 text-gray-500 dark:text-gray-500">Your code appears to be using safe Rust patterns</p>
        </div>
    `;
}

// Create comprehensive FFI dashboard with SVG-style visualization
function createFFIDashboardSVG(unsafeAllocs, ffiAllocs, boundaryCrossings, safetyViolations, unsafeMemory, enhancedData, boundaryEvents, violations) {
    return `
        <div class="bg-gradient-to-br from-gray-800 to-gray-900 rounded-xl p-6 text-white shadow-2xl">
            <!-- Header -->
            <div class="text-center mb-6">
                <h2 class="text-2xl font-bold mb-2 flex items-center justify-center">
                    <i class="fa fa-shield mr-3 text-red-400"></i>
                    Unsafe Rust & FFI Memory Analysis Dashboard
                </h2>
            </div>

            <!-- Key Metrics Row -->
            <div class="grid grid-cols-2 md:grid-cols-5 gap-4 mb-8">
                ${createFFIMetricCard('Unsafe Allocations', unsafeAllocs, '#e74c3c', 'fa-exclamation-triangle')}
                ${createFFIMetricCard('FFI Allocations', ffiAllocs, '#3498db', 'fa-exchange')}
                ${createFFIMetricCard('Boundary Crossings', boundaryCrossings, '#f39c12', 'fa-arrows-h')}
                ${createFFIMetricCard('Safety Violations', safetyViolations, '#e67e22', 'fa-warning')}
                ${createFFIMetricCard('Unsafe Memory', formatBytes(unsafeMemory), '#9b59b6', 'fa-memory')}
            </div>

            <!-- Main Dashboard Content -->
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
                <!-- Memory Allocation Sources -->
                <div class="bg-gray-700/50 rounded-lg p-4 backdrop-blur-sm">
                    <h3 class="text-lg font-semibold mb-4 text-white">Memory Allocation Sources</h3>
                    <div class="space-y-4">
                        ${createAllocationSourceBar('Unsafe Rust', unsafeAllocs, Math.max(unsafeAllocs, ffiAllocs), '#e74c3c')}
                        ${createAllocationSourceBar('FFI', ffiAllocs, Math.max(unsafeAllocs, ffiAllocs), '#3498db')}
                    </div>
                </div>

                <!-- Memory Safety Status -->
                <div class="bg-gray-700/50 rounded-lg p-4 backdrop-blur-sm">
                    <h3 class="text-lg font-semibold mb-4 text-white">Memory Safety Status</h3>
                    ${safetyViolations > 0 ? `
                        <div class="bg-red-900/30 border border-red-500/50 rounded-lg p-4">
                            <h4 class="text-red-300 font-semibold mb-2 flex items-center">
                                <i class="fa fa-exclamation-triangle mr-2"></i>
                                ${safetyViolations} Safety Violations Detected
                            </h4>
                            ${enhancedData.filter(item => (item.safety_violations || 0) > 0).slice(0, 2).map(item => `
                                <div class="text-red-400 text-sm flex items-center mb-1">
                                    <i class="fa fa-dot-circle-o mr-2 text-xs"></i>
                                    Pointer ${item.ptr}: ${item.safety_violations} violations
                                </div>
                            `).join('')}
                        </div>
                    ` : `
                        <div class="bg-green-900/30 border border-green-500/50 rounded-lg p-4">
                            <h4 class="text-green-300 font-semibold flex items-center mb-2">
                                <i class="fa fa-check-circle mr-2"></i>
                                No Safety Violations Detected
                            </h4>
                            <p class="text-green-400 text-sm">All unsafe operations appear to be handled correctly</p>
                        </div>
                    `}
                </div>
            </div>

            <!-- Cross-Language Memory Flow -->
            <div class="bg-gray-700/50 rounded-lg p-6 mb-6 backdrop-blur-sm">
                <h3 class="text-lg font-semibold mb-6 text-white text-center">Cross-Language Memory Flow</h3>
                <div class="flex items-center justify-center space-x-8">
                    <!-- Rust Side -->
                    <div class="bg-green-800/30 border-2 border-green-400/50 rounded-lg p-6 text-center backdrop-blur-sm">
                        <div class="text-green-300 font-bold text-xl mb-2">RUST</div>
                        <div class="text-green-400 text-sm">${unsafeAllocs} allocations</div>
                        <div class="w-16 h-16 mx-auto mt-3 bg-green-500/20 rounded-full flex items-center justify-center">
                            <i class="fa fa-rust text-green-400 text-2xl"></i>
                        </div>
                    </div>
                    
                    <!-- Flow Arrows -->
                    <div class="flex flex-col items-center space-y-4">
                        <div class="flex items-center space-x-2">
                            <div class="flex items-center space-x-1">
                                <div class="w-8 h-0.5 bg-red-400"></div>
                                <div class="w-0 h-0 border-l-4 border-l-red-400 border-t-2 border-t-transparent border-b-2 border-b-transparent"></div>
                            </div>
                            <span class="text-red-400 text-sm font-bold bg-red-900/30 px-2 py-1 rounded">
                                ${boundaryEvents.filter(e => e.event_type === 'RustToFfi').length}
                            </span>
                        </div>
                        <div class="flex items-center space-x-2">
                            <span class="text-orange-400 text-sm font-bold bg-orange-900/30 px-2 py-1 rounded">
                                ${boundaryEvents.filter(e => e.event_type === 'FfiToRust').length}
                            </span>
                            <div class="flex items-center space-x-1">
                                <div class="w-0 h-0 border-r-4 border-r-orange-400 border-t-2 border-t-transparent border-b-2 border-b-transparent"></div>
                                <div class="w-8 h-0.5 bg-orange-400"></div>
                            </div>
                        </div>
                    </div>
                    
                    <!-- FFI/C Side -->
                    <div class="bg-blue-800/30 border-2 border-blue-400/50 rounded-lg p-6 text-center backdrop-blur-sm">
                        <div class="text-blue-300 font-bold text-xl mb-2">FFI / C</div>
                        <div class="text-blue-400 text-sm">${ffiAllocs} allocations</div>
                        <div class="w-16 h-16 mx-auto mt-3 bg-blue-500/20 rounded-full flex items-center justify-center">
                            <i class="fa fa-code text-blue-400 text-2xl"></i>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Unsafe Memory Hotspots -->
            <div class="bg-gray-700/50 rounded-lg p-4 backdrop-blur-sm">
                <h3 class="text-lg font-semibold mb-4 text-white">Unsafe Memory Hotspots</h3>
                <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
                    ${enhancedData.slice(0, 12).map(item => createMemoryHotspot(item)).join('')}
                </div>
                ${enhancedData.length === 0 ? `
                    <div class="text-center py-8 text-gray-400">
                        <i class="fa fa-shield-alt text-4xl mb-2"></i>
                        <p>No unsafe memory hotspots detected</p>
                    </div>
                ` : ''}
            </div>
        </div>
    `;
}

// Create FFI metric card
function createFFIMetricCard(title, value, color, icon) {
    return `
        <div class="bg-gray-700/30 border border-gray-600/50 rounded-lg p-4 text-center backdrop-blur-sm hover:bg-gray-600/30 transition-all">
            <div class="flex items-center justify-center mb-2">
                <i class="fa ${icon} text-2xl" style="color: ${color}"></i>
            </div>
            <div class="text-2xl font-bold mb-1" style="color: ${color}">${value}</div>
            <div class="text-xs text-gray-300 uppercase tracking-wide">${title}</div>
        </div>
    `;
}

// Create allocation source bar
function createAllocationSourceBar(label, count, maxCount, color) {
    const percentage = maxCount > 0 ? (count / maxCount) * 100 : 0;
    const barHeight = Math.max(20, (count / maxCount) * 80);

    return `
        <div class="flex items-end space-x-4">
            <div class="flex-1">
                <div class="flex justify-between items-center mb-2">
                    <span class="text-sm font-medium text-gray-300">${label}</span>
                    <span class="text-lg font-bold text-white">${count}</span>
                </div>
                <div class="w-full bg-gray-600 rounded-full h-6 overflow-hidden">
                    <div class="h-full rounded-full transition-all duration-500 flex items-center justify-center text-white text-xs font-bold" 
                         style="width: ${percentage}%; background-color: ${color};">
                        ${count > 0 ? count : ''}
                    </div>
                </div>
            </div>
        </div>
    `;
}

// Create memory hotspot visualization
function createMemoryHotspot(item) {
    const size = item.size || 0;
    const isUnsafe = !item.ffi_tracked;
    const radius = Math.min(30, Math.max(12, Math.sqrt(size / 50)));
    const color = isUnsafe ? '#e74c3c' : '#3498db';
    const bgColor = isUnsafe ? 'bg-red-900/20' : 'bg-blue-900/20';
    const borderColor = isUnsafe ? 'border-red-500/50' : 'border-blue-500/50';

    return `
        <div class="flex flex-col items-center p-3 ${bgColor} border ${borderColor} rounded-lg backdrop-blur-sm hover:scale-105 transition-transform">
            <div class="relative mb-2">
                <div class="rounded-full border-2 flex items-center justify-center text-white text-xs font-bold shadow-lg"
                     style="width: ${radius * 2}px; height: ${radius * 2}px; background-color: ${color}; border-color: ${color};">
                    ${size > 1024 ? Math.round(size / 1024) + 'K' : size + 'B'}
                </div>
                ${(item.safety_violations || 0) > 0 ? `
                    <div class="absolute -top-1 -right-1 w-4 h-4 bg-red-500 rounded-full flex items-center justify-center">
                        <i class="fa fa-exclamation text-white text-xs"></i>
                    </div>
                ` : ''}
            </div>
            <div class="text-xs text-center">
                <div class="font-semibold" style="color: ${color}">
                    ${isUnsafe ? 'UNSAFE' : 'FFI'}
                </div>
                <div class="text-gray-400 text-xs">
                    ${formatBytes(size)}
                </div>
            </div>
        </div>
    `;
}

// Initialize memory fragmentation analysis with enhanced SVG-style visualization
function initMemoryFragmentation() {
    const container = document.getElementById('memoryFragmentation');
    if (!container) return;

    const allocations = window.analysisData.memory_analysis?.allocations || [];

    if (allocations.length === 0) {
        container.innerHTML = createFragmentationEmptyState();
        return;
    }

    // Fixed memory fragmentation analysis: based on allocation size distribution rather than address gaps
    const sortedAllocs = allocations
        .filter(alloc => alloc.size && alloc.size > 0)
        .map(alloc => ({
            size: alloc.size,
            type: alloc.type_name || 'System Allocation',
            var_name: alloc.var_name || 'unknown'
        }))
        .sort((a, b) => a.size - b.size);

    const totalMemory = sortedAllocs.reduce((sum, alloc) => sum + alloc.size, 0);
    
    // Calculate fragmentation based on allocation size distribution
    const sizeVariance = calculateSizeVariance(sortedAllocs);
    const smallAllocRatio = sortedAllocs.filter(a => a.size < 1024).length / sortedAllocs.length;
    
    // Fragmentation score: based on size distribution unevenness
    const fragmentationRatio = Math.min(100, (sizeVariance / 1000 + smallAllocRatio * 50));
    
    // Simplified gap analysis: only count quantity, not fake address gaps
    const gaps = Math.max(0, sortedAllocs.length - 1);
    const maxGap = 0; // No longer calculate address gaps
    let totalGapSize = 0; // Reset to 0 to avoid huge fake values

    // Size distribution analysis (inspired by SVG)
    const sizeDistribution = {
        tiny: sortedAllocs.filter(a => a.size < 64).length,
        small: sortedAllocs.filter(a => a.size >= 64 && a.size < 1024).length,
        medium: sortedAllocs.filter(a => a.size >= 1024 && a.size < 65536).length,
        large: sortedAllocs.filter(a => a.size >= 65536).length
    };

    container.innerHTML = createFragmentationAnalysisSVG(
        fragmentationRatio, gaps, maxGap, sortedAllocs.length,
        totalMemory, sizeDistribution, sortedAllocs
    );
}

// Create fragmentation empty state
function createFragmentationEmptyState() {
    return `
        <div class="bg-white dark:bg-gray-800 rounded-xl p-6 card-shadow transition-colors">
            <h2 class="text-xl font-semibold mb-4 flex items-center text-heading">
                <i class="fa fa-puzzle-piece text-orange-500 mr-2"></i>Memory Fragmentation Analysis
            </h2>
            <div class="text-center py-8">
                <div class="mb-4">
                    <svg class="w-16 h-16 mx-auto text-gray-400 dark:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path>
                    </svg>
                </div>
                <h4 class="text-lg font-semibold mb-2 text-gray-800 dark:text-gray-200">No Memory Data for Analysis</h4>
                <p class="text-sm text-gray-600 dark:text-gray-400">Memory fragmentation analysis requires allocation data</p>
            </div>
        </div>
    `;
}

// Create comprehensive fragmentation analysis with SVG-style visualization
function createFragmentationAnalysisSVG(fragmentationRatio, gaps, maxGap, blockCount, totalMemory, sizeDistribution, sortedAllocs) {
    return `
        <div class="bg-white dark:bg-gray-800 rounded-xl p-6 card-shadow transition-colors">
            <h2 class="text-xl font-semibold mb-6 flex items-center text-heading">
                <i class="fa fa-puzzle-piece text-orange-500 mr-2"></i>Memory Fragmentation Analysis
            </h2>
            
            <!-- Key Metrics Grid -->
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
                ${createFragmentationMetricCard('Fragmentation', fragmentationRatio.toFixed(1) + '%', fragmentationRatio, '#f39c12')}
                ${createFragmentationMetricCard('Memory Gaps', gaps, 100, '#3498db')}
                ${createFragmentationMetricCard('Largest Gap', formatBytes(maxGap), 100, '#27ae60')}
                ${createFragmentationMetricCard('Memory Blocks', blockCount, 100, '#9b59b6')}
            </div>

            <!-- Main Analysis Content -->
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
                <!-- Fragmentation Assessment -->
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                    <h4 class="font-semibold mb-4 text-gray-800 dark:text-white">Fragmentation Assessment</h4>
                    <div class="space-y-4">
                        <div>
                            <div class="flex justify-between items-center mb-2">
                                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">Overall Health</span>
                                <span class="text-sm font-bold ${getFragmentationColor(fragmentationRatio)}">${fragmentationRatio.toFixed(1)}%</span>
                            </div>
                            <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-4">
                                <div class="h-4 rounded-full transition-all duration-500 ${getFragmentationBgColor(fragmentationRatio)}" 
                                     style="width: ${Math.min(fragmentationRatio, 100)}%"></div>
                            </div>
                        </div>
                        <div class="text-sm text-gray-600 dark:text-gray-300">
                            ${getFragmentationAssessment(fragmentationRatio)}
                        </div>
                    </div>
                </div>

                <!-- Size Distribution (inspired by SVG bar chart) -->
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                    <h4 class="font-semibold mb-4 text-gray-800 dark:text-white">Size Distribution</h4>
                    <div class="space-y-3">
                        ${createSizeDistributionBar('Tiny (0-64B)', sizeDistribution.tiny, blockCount, '#27ae60')}
                        ${createSizeDistributionBar('Small (64B-1KB)', sizeDistribution.small, blockCount, '#f39c12')}
                        ${createSizeDistributionBar('Medium (1KB-64KB)', sizeDistribution.medium, blockCount, '#e74c3c')}
                        ${createSizeDistributionBar('Large (>64KB)', sizeDistribution.large, blockCount, '#8e44ad')}
                    </div>
                </div>
            </div>

            <!-- Memory Layout Visualization -->
            <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                <h4 class="font-semibold mb-4 text-gray-800 dark:text-white">Memory Layout Visualization</h4>
                <div class="relative">
                    <!-- Memory blocks visualization -->
                    <div class="h-16 bg-gray-200 dark:bg-gray-600 rounded relative overflow-hidden mb-4">
                        ${createMemoryLayoutVisualization(sortedAllocs, totalMemory)}
                    </div>
                    
                    <!-- Memory address timeline -->
                    <div class="flex justify-between text-xs text-gray-500 dark:text-gray-400 mb-2">
                        <span>Low Address</span>
                        <span>Memory Layout</span>
                        <span>High Address</span>
                    </div>
                    
                    <!-- Legend -->
                    <div class="flex flex-wrap gap-4 text-xs">
                        <div class="flex items-center">
                            <div class="w-3 h-3 bg-blue-500 rounded mr-2"></div>
                            <span class="text-gray-600 dark:text-gray-300">User Allocations</span>
                        </div>
                        <div class="flex items-center">
                            <div class="w-3 h-3 bg-gray-400 rounded mr-2"></div>
                            <span class="text-gray-600 dark:text-gray-300">System Allocations</span>
                        </div>
                        <div class="flex items-center">
                            <div class="w-3 h-3 bg-red-300 rounded mr-2"></div>
                            <span class="text-gray-600 dark:text-gray-300">Memory Gaps</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    `;
}

// Create fragmentation metric card with circular progress
function createFragmentationMetricCard(title, value, percentage, color) {
    const normalizedPercentage = Math.min(100, Math.max(0, percentage));
    const circumference = 2 * Math.PI * 20;
    const strokeDashoffset = circumference - (normalizedPercentage / 100) * circumference;

    return `
        <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 text-center hover:shadow-md transition-shadow">
            <div class="flex items-center justify-between">
                <div class="flex-1">
                    <p class="text-xs font-medium text-gray-600 dark:text-gray-400 uppercase">${title}</p>
                    <p class="text-lg font-bold text-gray-900 dark:text-white">${value}</p>
                </div>
                <div class="relative w-10 h-10">
                    <svg class="w-10 h-10 transform -rotate-90" viewBox="0 0 50 50">
                        <circle cx="25" cy="25" r="20" stroke="#e5e7eb" stroke-width="4" fill="none" class="dark:stroke-gray-600"/>
                        <circle cx="25" cy="25" r="20" stroke="${color}" stroke-width="4" fill="none" 
                                stroke-dasharray="${circumference}" stroke-dashoffset="${strokeDashoffset}"
                                stroke-linecap="round" class="transition-all duration-500"/>
                    </svg>
                </div>
            </div>
        </div>
    `;
}

// Create size distribution bar
function createSizeDistributionBar(label, count, total, color) {
    const percentage = total > 0 ? (count / total) * 100 : 0;
    return `
        <div class="flex items-center justify-between">
            <span class="text-sm font-medium text-gray-700 dark:text-gray-300 w-28">${label}</span>
            <div class="flex-1 mx-3">
                <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-4">
                    <div class="h-4 rounded-full transition-all duration-500" 
                         style="width: ${percentage}%; background-color: ${color}"></div>
                </div>
            </div>
            <span class="text-sm font-bold text-gray-900 dark:text-white w-8 text-right">${count}</span>
        </div>
    `;
}

// Create memory layout visualization
function createMemoryLayoutVisualization(sortedAllocs, totalMemory) {
    if (sortedAllocs.length === 0) return '<div class="flex items-center justify-center h-full text-gray-400">No memory layout data</div>';

    return sortedAllocs.slice(0, 30).map((alloc, index) => {
        const width = Math.max(1, (alloc.size / totalMemory) * 100);
        const left = (index / 30) * 100;
        const isUserAlloc = alloc.type !== 'System Allocation';
        const color = isUserAlloc ? '#3498db' : '#95a5a6';

        return `
            <div class="absolute h-full transition-all hover:brightness-110 cursor-pointer" 
                 style="left: ${left}%; width: ${width}%; background-color: ${color}; opacity: 0.8;"
                 title="${alloc.type}: ${formatBytes(alloc.size)} at ${(alloc.address || 0).toString(16)}">
            </div>
        `;
    }).join('');
}

// Calculate variance of allocation sizes to assess fragmentation level
function calculateSizeVariance(allocations) {
    if (allocations.length === 0) return 0;
    
    const sizes = allocations.map(a => a.size);
    const mean = sizes.reduce((sum, size) => sum + size, 0) / sizes.length;
    const variance = sizes.reduce((sum, size) => sum + Math.pow(size - mean, 2), 0) / sizes.length;
    
    return Math.sqrt(variance); // 返回标准差
}

// Helper functions for fragmentation analysis
function getFragmentationColor(ratio) {
    if (ratio < 10) return 'text-green-600 dark:text-green-400';
    if (ratio < 25) return 'text-yellow-600 dark:text-yellow-400';
    if (ratio < 50) return 'text-orange-600 dark:text-orange-400';
    return 'text-red-600 dark:text-red-400';
}

function getFragmentationBgColor(ratio) {
    if (ratio < 10) return 'bg-green-500';
    if (ratio < 25) return 'bg-yellow-500';
    if (ratio < 50) return 'bg-orange-500';
    return 'bg-red-500';
}

function getFragmentationAssessment(ratio) {
    if (ratio < 10) return 'Excellent memory layout with minimal fragmentation. Memory is well-organized.';
    if (ratio < 25) return 'Good memory layout with low fragmentation. No immediate concerns.';
    if (ratio < 50) return 'Moderate fragmentation detected. Consider memory pool allocation strategies.';
    return 'High fragmentation detected. Memory layout optimization strongly recommended.';
}

// Initialize memory growth trends with enhanced SVG-style visualization
function initMemoryGrowthTrends() {
    const container = document.getElementById('memoryGrowthTrends');
    if (!container) return;

    const allocations = window.analysisData.memory_analysis?.allocations || [];

    // Sort allocations by timestamp
    const sortedAllocs = allocations
        .filter(alloc => alloc.timestamp_alloc)
        .sort((a, b) => a.timestamp_alloc - b.timestamp_alloc);

    if (sortedAllocs.length === 0) {
        container.innerHTML = createGrowthTrendsEmptyState();
        return;
    }

    // Calculate cumulative memory usage over time
    let cumulativeMemory = 0;
    let peakMemory = 0;
    const timePoints = [];

    sortedAllocs.forEach((alloc, index) => {
        cumulativeMemory += alloc.size || 0;
        peakMemory = Math.max(peakMemory, cumulativeMemory);

        if (index % Math.max(1, Math.floor(sortedAllocs.length / 20)) === 0) {
            timePoints.push({
                timestamp: alloc.timestamp_alloc,
                memory: cumulativeMemory,
                index: index,
                allocCount: index + 1
            });
        }
    });

    const startMemory = timePoints[0]?.memory || 0;
    const endMemory = timePoints[timePoints.length - 1]?.memory || 0;
    const growthRate = startMemory > 0 ? ((endMemory - startMemory) / startMemory * 100) : 0;
    const averageMemory = timePoints.reduce((sum, point) => sum + point.memory, 0) / timePoints.length;

    container.innerHTML = createMemoryGrowthTrendsSVG(
        peakMemory, averageMemory, growthRate, timePoints, sortedAllocs.length
    );
}

// Create growth trends empty state
function createGrowthTrendsEmptyState() {
    return `
        <div class="bg-white dark:bg-gray-800 rounded-xl p-6 card-shadow transition-colors">
            <h2 class="text-xl font-semibold mb-4 flex items-center text-heading">
                <i class="fa fa-line-chart text-green-500 mr-2"></i>Memory Growth Trends
            </h2>
            <div class="text-center py-8">
                <div class="mb-4">
                    <svg class="w-16 h-16 mx-auto text-gray-400 dark:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path>
                    </svg>
                </div>
                <h4 class="text-lg font-semibold mb-2 text-gray-800 dark:text-gray-200">No Timeline Data Available</h4>
                <p class="text-sm text-gray-600 dark:text-gray-400">Memory growth analysis requires timestamp data</p>
            </div>
        </div>
    `;
}

// Create comprehensive memory growth trends visualization
function createMemoryGrowthTrendsSVG(peakMemory, averageMemory, growthRate, timePoints, totalAllocs) {
    return `
        <div class="bg-white dark:bg-gray-800 rounded-xl p-6 card-shadow transition-colors">
            <h2 class="text-xl font-semibold mb-6 flex items-center text-heading">
                <i class="fa fa-line-chart text-green-500 mr-2"></i>Memory Growth Trends
            </h2>
            
            <!-- Key Metrics Grid -->
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
                ${createGrowthMetricCard('Peak Memory', formatBytes(peakMemory), 100, '#e74c3c')}
                ${createGrowthMetricCard('Average Memory', formatBytes(averageMemory), Math.round((averageMemory / peakMemory) * 100), '#3498db')}
                ${createGrowthMetricCard('Growth Rate', (growthRate > 0 ? '+' : '') + growthRate.toFixed(1) + '%', Math.abs(growthRate), getGrowthRateColor(growthRate))}
                ${createGrowthMetricCard('Total Allocations', totalAllocs, 100, '#9b59b6')}
            </div>

            <!-- Main Growth Chart -->
            <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6 mb-6">
                <h4 class="font-semibold mb-4 text-gray-800 dark:text-white">Memory Usage Over Time</h4>
                <div class="relative">
                    <!-- Chart Container -->
                    <div class="h-48 relative bg-white dark:bg-gray-600 rounded border dark:border-gray-500 overflow-hidden">
                        ${createMemoryGrowthChart(timePoints, peakMemory)}
                    </div>
                    
                    <!-- Chart Labels -->
                    <div class="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-2">
                        <span>Start</span>
                        <span>Memory Usage Timeline</span>
                        <span>End</span>
                    </div>
                    
                    <!-- Peak Memory Line -->
                    <div class="absolute top-2 right-2 text-xs text-red-500 dark:text-red-400 bg-white dark:bg-gray-800 px-2 py-1 rounded shadow">
                        Peak: ${formatBytes(peakMemory)}
                    </div>
                </div>
            </div>

            <!-- Growth Analysis -->
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <!-- Growth Assessment -->
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                    <h4 class="font-semibold mb-4 text-gray-800 dark:text-white">Growth Assessment</h4>
                    <div class="space-y-3">
                        <div class="flex items-center justify-between">
                            <span class="text-sm text-gray-600 dark:text-gray-300">Memory Efficiency</span>
                            <span class="text-sm font-bold ${getEfficiencyColor(averageMemory, peakMemory)}">${((averageMemory / peakMemory) * 100).toFixed(1)}%</span>
                        </div>
                        <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-2">
                            <div class="h-2 rounded-full transition-all duration-500 ${getEfficiencyBgColor(averageMemory, peakMemory)}" 
                                 style="width: ${(averageMemory / peakMemory) * 100}%"></div>
                        </div>
                        <div class="text-sm text-gray-600 dark:text-gray-300">
                            ${getGrowthAssessment(growthRate)}
                        </div>
                    </div>
                </div>

                <!-- Memory Allocation Timeline -->
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                    <h4 class="font-semibold mb-4 text-gray-800 dark:text-white">Recent Allocations</h4>
                    <div class="space-y-2 max-h-32 overflow-y-auto">
                        ${timePoints.slice(-6).map((point, index) => `
                            <div class="flex justify-between items-center text-sm">
                                <span class="text-gray-600 dark:text-gray-300">Alloc #${point.allocCount}</span>
                                <span class="font-mono text-xs font-bold text-gray-900 dark:text-white">${formatBytes(point.memory)}</span>
                            </div>
                        `).join('')}
                    </div>
                    <div class="text-xs text-gray-500 dark:text-gray-400 mt-2">
                        Showing latest allocation points
                    </div>
                </div>
            </div>
        </div>
    `;
}

// Create growth metric card
function createGrowthMetricCard(title, value, percentage, color) {
    const normalizedPercentage = Math.min(100, Math.max(0, percentage));

    return `
        <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 text-center hover:shadow-md transition-shadow">
            <div class="mb-2">
                <div class="text-2xl font-bold" style="color: ${color}">${value}</div>
                <div class="text-xs text-gray-600 dark:text-gray-400 uppercase tracking-wide">${title}</div>
            </div>
            <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-2">
                <div class="h-2 rounded-full transition-all duration-500" 
                     style="width: ${normalizedPercentage}%; background-color: ${color}"></div>
            </div>
        </div>
    `;
}

// Create memory growth chart
function createMemoryGrowthChart(timePoints, peakMemory) {
    if (timePoints.length < 2) return '<div class="flex items-center justify-center h-full text-gray-400">Insufficient data points</div>';

    const chartHeight = 180;
    const chartWidth = 100; // percentage

    // Create SVG path for the growth line
    const pathPoints = timePoints.map((point, index) => {
        const x = (index / (timePoints.length - 1)) * chartWidth;
        const y = chartHeight - ((point.memory / peakMemory) * (chartHeight - 20));
        return `${x},${y}`;
    });

    return `
        <!-- Background Grid -->
        <div class="absolute inset-0">
            ${[0, 25, 50, 75, 100].map(y => `
                <div class="absolute w-full border-t border-gray-200 dark:border-gray-500 opacity-30" 
                     style="top: ${y}%"></div>
            `).join('')}
        </div>
        
        <!-- Growth Line -->
        <svg class="absolute inset-0 w-full h-full" preserveAspectRatio="none">
            <polyline
                fill="none"
                stroke="#27ae60"
                stroke-width="3"
                stroke-linecap="round"
                stroke-linejoin="round"
                points="${timePoints.map((point, index) => {
        const x = (index / (timePoints.length - 1)) * 100;
        const y = 100 - ((point.memory / peakMemory) * 90);
        return `${x},${y}`;
    }).join(' ')}"
                class="drop-shadow-sm"
            />
        </svg>
        
        <!-- Data Points -->
        ${timePoints.map((point, index) => {
        const x = (index / (timePoints.length - 1)) * 100;
        const y = 100 - ((point.memory / peakMemory) * 90);
        return `
                <div class="absolute w-3 h-3 bg-green-500 rounded-full border-2 border-white dark:border-gray-600 shadow-sm transform -translate-x-1/2 -translate-y-1/2 hover:scale-125 transition-transform cursor-pointer" 
                     style="left: ${x}%; top: ${y}%"
                     title="Memory: ${formatBytes(point.memory)} at allocation #${point.allocCount}">
                </div>
            `;
    }).join('')}
        
        <!-- Peak Memory Indicator -->
        <div class="absolute w-full border-t-2 border-red-500 border-dashed opacity-60" style="top: 10%">
            <div class="absolute -top-1 right-0 text-xs text-red-500 bg-white dark:bg-gray-600 px-1 rounded">
                Peak
            </div>
        </div>
    `;
}

// Helper functions for growth analysis
function getGrowthRateColor(rate) {
    if (rate < -10) return '#27ae60'; // Green for decreasing
    if (rate < 10) return '#3498db';  // Blue for stable
    if (rate < 50) return '#f39c12'; // Orange for moderate growth
    return '#e74c3c'; // Red for high growth
}

function getEfficiencyColor(avg, peak) {
    const efficiency = (avg / peak) * 100;
    if (efficiency > 80) return 'text-red-600 dark:text-red-400';
    if (efficiency > 60) return 'text-orange-600 dark:text-orange-400';
    if (efficiency > 40) return 'text-yellow-600 dark:text-yellow-400';
    return 'text-green-600 dark:text-green-400';
}

function getEfficiencyBgColor(avg, peak) {
    const efficiency = (avg / peak) * 100;
    if (efficiency > 80) return 'bg-red-500';
    if (efficiency > 60) return 'bg-orange-500';
    if (efficiency > 40) return 'bg-yellow-500';
    return 'bg-green-500';
}

function getGrowthAssessment(rate) {
    if (rate < -10) return 'Excellent: Memory usage is decreasing, indicating good cleanup.';
    if (rate < 10) return 'Good: Stable memory usage with minimal growth.';
    if (rate < 50) return 'Moderate: Some memory growth detected, monitor for trends.';
    return 'Concerning: High memory growth detected, investigate for potential leaks.';
}

// Node Detail Panel for Variable Relationship Graph
class NodeDetailPanel {
    constructor(containerId) {
        this.container = document.getElementById(containerId);
        this.panel = null;
        this.currentNode = null;
    }

    show(nodeData, position) {
        console.log('Showing panel for:', nodeData.id);
        this.hide(); // Close existing panel
        this.panel = this.createPanel(nodeData);
        console.log('Panel created:', this.panel);
        this.positionPanel(position);
        this.container.appendChild(this.panel);
        console.log('Panel added to container');
        this.currentNode = nodeData;
    }

    hide() {
        if (this.panel) {
            this.panel.remove();
            this.panel = null;
            this.currentNode = null;
        }
    }

    createPanel(nodeData) {
        const panel = document.createElement('div');
        panel.className = 'node-detail-panel';

        // Find related allocation data
        const allocations = window.analysisData.memory_analysis?.allocations || [];
        const allocation = allocations.find(alloc => alloc.var_name === nodeData.id);

        // Calculate relationships
        const sameTypeCount = allocations.filter(alloc =>
            alloc.type_name === nodeData.type_name && alloc.var_name !== nodeData.id
        ).length;

        const sameCategoryCount = allocations.filter(alloc =>
            getTypeCategory(alloc.type_name || '') === (nodeData.category || 'primitive') && alloc.var_name !== nodeData.id
        ).length;

        panel.innerHTML = `
            <div class="flex justify-between items-center mb-3">
                <h3>Variable Details</h3>
                <button class="close-button text-xl leading-none">&times;</button>
            </div>
            
            <div class="space-y-3">
                <div>
                    <label>Variable Name</label>
                    <p class="font-mono">${nodeData.id}</p>
                </div>
                
                <div>
                    <label>Type</label>
                    <p class="font-mono">${nodeData.type_name || 'Unknown'}</p>
                    <div class="flex items-center mt-1">
                        <div class="w-3 h-3 rounded-full mr-2" style="background-color: ${getEnhancedTypeColor(nodeData.type_name || 'unknown', nodeData.category || 'primitive')}"></div>
                        <span class="text-xs capitalize">${(nodeData.category || 'primitive').replace('_', ' ')}</span>
                    </div>
                </div>
                
                <div>
                    <label>Memory Size</label>
                    <p>${formatBytes(nodeData.size)}</p>
                </div>
                
                <div>
                    <label>Complexity Score</label>
                    <div class="flex items-center mb-2">
                        <div class="w-5 h-5 rounded-full mr-2 flex items-center justify-center text-white font-bold text-xs" style="background-color: ${getComplexityColor(nodeData.complexity || 2)}">${nodeData.complexity || 2}</div>
                        <span class="font-semibold">${nodeData.complexity || 2}/10 - ${getComplexityLevel(nodeData.complexity || 2)}</span>
                    </div>
                    <div class="text-xs text-gray-600 dark:text-gray-400">
                        ${getComplexityExplanation(nodeData.complexity || 2)}
                    </div>
                </div>
                
                ${allocation ? `
                    <div>
                        <label>Memory Address</label>
                        <p class="font-mono text-xs">${allocation.ptr}</p>
                    </div>
                    
                    <div>
                        <label>Allocated At</label>
                        <p class="text-sm">${new Date(allocation.timestamp_alloc / 1000000).toLocaleString()}</p>
                    </div>
                ` : ''}
                
                <div>
                    <label>Relationships</label>
                    <div class="text-sm space-y-1">
                        <div class="flex justify-between">
                            <span>Same type:</span>
                            <span class="font-semibold">${sameTypeCount}</span>
                        </div>
                        <div class="flex justify-between">
                            <span>Same category:</span>
                            <span class="font-semibold">${sameCategoryCount}</span>
                        </div>
                    </div>
                </div>
                
                <div>
                    <label>Type Analysis</label>
                    <div class="text-xs space-y-1">
                        ${getTypeAnalysis(nodeData.type_name || 'unknown', nodeData.size)}
                    </div>
                </div>
            </div>
        `;

        // Add close button functionality
        const closeButton = panel.querySelector('.close-button');
        closeButton.addEventListener('click', () => {
            this.hide();
        });

        return panel;
    }

    positionPanel(position) {
        if (!this.panel) return;

        // Simple positioning - place panel at a fixed position relative to container
        this.panel.style.position = 'absolute';
        this.panel.style.left = '20px';
        this.panel.style.top = '20px';
        this.panel.style.zIndex = '1000';

        console.log('Panel positioned at:', this.panel.style.left, this.panel.style.top);
    }
}

// Initialize variable relationship graph with enhanced D3.js force simulation
function initVariableGraph() {
    const container = document.getElementById('variable-graph-container');
    if (!container) return;

    const allocations = window.analysisData.memory_analysis?.allocations || [];
    const userAllocations = allocations.filter(alloc =>
        alloc.var_name && alloc.var_name !== 'unknown' &&
        alloc.type_name && alloc.type_name !== 'unknown'
    );

    if (userAllocations.length === 0) {
        container.innerHTML = `
            <div class="flex items-center justify-center h-full text-gray-500 dark:text-gray-400">
                <div class="text-center">
                    <i class="fa fa-sitemap text-4xl mb-4"></i>
                    <p class="text-lg font-semibold mb-2">No User Variables Found</p>
                    <p class="text-sm">Use track_var! macro to track variable relationships</p>
                </div>
            </div>
        `;
        return;
    }

    // Clear container
    container.innerHTML = '';

    // Set up dimensions
    const width = container.clientWidth;
    const height = container.clientHeight;

    // Create SVG
    const svg = d3.select(container)
        .append('svg')
        .attr('width', width)
        .attr('height', height)
        .style('background', 'transparent');

    // Create zoom behavior
    const zoom = d3.zoom()
        .scaleExtent([0.1, 4])
        .on('zoom', (event) => {
            g.attr('transform', event.transform);
        });

    svg.call(zoom);

    // Create main group for zooming/panning
    const g = svg.append('g');

    // Prepare nodes data
    const nodes = userAllocations.map((alloc, index) => ({
        id: alloc.var_name,
        type: alloc.type_name,
        size: alloc.size || 0,
        complexity: getComplexityFromType(alloc.type_name),
        category: getTypeCategory(alloc.type_name),
        allocation: alloc
    }));

    // Create more sophisticated relationships
    const links = [];

    // Type similarity relationships
    for (let i = 0; i < nodes.length; i++) {
        for (let j = i + 1; j < nodes.length; j++) {
            const node1 = nodes[i];
            const node2 = nodes[j];

            // Same type relationship
            if (node1.type === node2.type) {
                links.push({
                    source: node1.id,
                    target: node2.id,
                    type: 'same_type',
                    strength: 1.0
                });
            }
            // Similar category relationship
            else if (node1.category === node2.category && node1.category !== 'primitive') {
                links.push({
                    source: node1.id,
                    target: node2.id,
                    type: 'similar_category',
                    strength: 0.6
                });
            }
            // Generic type relationship (Vec<T>, Box<T>, etc.)
            else if (getGenericBase(node1.type) === getGenericBase(node2.type)) {
                links.push({
                    source: node1.id,
                    target: node2.id,
                    type: 'generic_family',
                    strength: 0.8
                });
            }
        }
    }

    // Create force simulation
    const simulation = d3.forceSimulation(nodes)
        .force('link', d3.forceLink(links)
            .id(d => d.id)
            .distance(d => 80 + (1 - d.strength) * 40)
            .strength(d => d.strength * 0.7)
        )
        .force('charge', d3.forceManyBody()
            .strength(d => -200 - (d.size / 100))
        )
        .force('center', d3.forceCenter(width / 2, height / 2))
        .force('collision', d3.forceCollide()
            .radius(d => {
                const minRadius = 15;
                const maxRadius = 50;
                const maxSize = Math.max(...nodes.map(n => n.size));
                const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
                const nodeRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
                return nodeRadius + 5;
            })
        );

    // Create link elements
    const link = g.append('g')
        .attr('class', 'links')
        .selectAll('line')
        .data(links)
        .enter().append('line')
        .attr('stroke', d => getLinkColor(d.type))
        .attr('stroke-opacity', d => 0.3 + d.strength * 0.4)
        .attr('stroke-width', d => 1 + d.strength * 2)
        .attr('stroke-dasharray', d => d.type === 'similar_category' ? '5,5' : null);

    // Create node groups
    const node = g.append('g')
        .attr('class', 'nodes')
        .selectAll('g')
        .data(nodes)
        .enter().append('g')
        .attr('class', 'graph-node')
        .style('cursor', 'pointer')
        .call(d3.drag()
            .on('start', dragstarted)
            .on('drag', dragged)
            .on('end', dragended)
        );

    // Add circles to nodes - size based on memory usage
    node.append('circle')
        .attr('r', d => {
            // Scale node size based on memory usage (larger memory = larger node)
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            return minRadius + (sizeRatio * (maxRadius - minRadius));
        })
        .attr('fill', d => getEnhancedTypeColor(d.type, d.category))
        .attr('stroke', '#fff')
        .attr('stroke-width', 2)
        .style('filter', 'drop-shadow(0px 2px 4px rgba(0,0,0,0.2))')
        .on('mouseover', function (event, d) {
            const currentRadius = d3.select(this).attr('r');
            d3.select(this)
                .transition()
                .duration(200)
                .attr('r', parseFloat(currentRadius) * 1.2)
                .style('filter', 'drop-shadow(0px 4px 8px rgba(0,0,0,0.3))');

            // Highlight connected links
            link.style('stroke-opacity', l =>
                (l.source.id === d.id || l.target.id === d.id) ? 0.8 : 0.1
            );
        })
        .on('mouseout', function (event, d) {
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            const originalRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
            
            d3.select(this)
                .transition()
                .duration(200)
                .attr('r', originalRadius)
                .style('filter', 'drop-shadow(0px 2px 4px rgba(0,0,0,0.2))');

            // Reset link opacity
            link.style('stroke-opacity', l => 0.3 + l.strength * 0.4);
        });

    // Add complexity indicators (small circles with numbers)
    const complexityGroup = node.append('g')
        .attr('class', 'complexity-indicator');

    complexityGroup.append('circle')
        .attr('r', 8)
        .attr('cx', d => {
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            const nodeRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
            return nodeRadius + 8;
        })
        .attr('cy', d => {
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            const nodeRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
            return -nodeRadius - 8;
        })
        .attr('fill', d => getComplexityColor(d.complexity))
        .attr('stroke', '#fff')
        .attr('stroke-width', 2);

    // Add complexity score text
    complexityGroup.append('text')
        .text(d => d.complexity || 2)
        .attr('x', d => {
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            const nodeRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
            return nodeRadius + 8;
        })
        .attr('y', d => {
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            const nodeRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
            return -nodeRadius - 8 + 3;
        })
        .attr('text-anchor', 'middle')
        .style('font-size', '10px')
        .style('font-weight', 'bold')
        .style('fill', '#fff')
        .style('pointer-events', 'none');

    // Add variable names
    node.append('text')
        .text(d => d.id)
        .attr('text-anchor', 'middle')
        .attr('dy', d => {
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            const nodeRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
            return nodeRadius + 15;
        })
        .style('font-size', '11px')
        .style('font-weight', 'bold')
        .style('fill', 'var(--text-primary)')
        .style('pointer-events', 'none');

    // Add type labels
    node.append('text')
        .text(d => formatTypeName(d.type))
        .attr('text-anchor', 'middle')
        .attr('dy', d => {
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            const nodeRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
            return nodeRadius + 28;
        })
        .style('font-size', '9px')
        .style('fill', 'var(--text-secondary)')
        .style('pointer-events', 'none');

    // Add click interaction
    const detailPanel = new NodeDetailPanel('variable-graph-container');

    node.on('click', function (event, d) {
        event.stopPropagation();
        console.log('Node clicked:', d.id, d);
        const position = {
            x: event.pageX,
            y: event.pageY
        };
        detailPanel.show(d, position);
    });

    // Click on empty space to hide panel
    svg.on('click', function (event) {
        if (event.target === this) {
            detailPanel.hide();
        }
    });

    // Update positions on simulation tick
    simulation.on('tick', () => {
        link
            .attr('x1', d => d.source.x)
            .attr('y1', d => d.source.y)
            .attr('x2', d => d.target.x)
            .attr('y2', d => d.target.y);

        node
            .attr('transform', d => `translate(${d.x},${d.y})`);
    });

    // Add control buttons
    const controls = d3.select(container)
        .append('div')
        .attr('class', 'absolute top-2 right-2 flex space-x-2');

    controls.append('button')
        .attr('class', 'px-3 py-1 bg-blue-500 hover:bg-blue-600 text-white text-xs rounded transition-colors')
        .text('Reset View')
        .on('click', () => {
            svg.transition().duration(750).call(
                zoom.transform,
                d3.zoomIdentity
            );
        });

    controls.append('button')
        .attr('class', 'px-3 py-1 bg-green-500 hover:bg-green-600 text-white text-xs rounded transition-colors')
        .text('Reheat')
        .on('click', () => {
            simulation.alpha(0.3).restart();
        });

    // Drag functions
    function dragstarted(event, d) {
        if (!event.active) simulation.alphaTarget(0.3).restart();
        d.fx = d.x;
        d.fy = d.y;
    }

    function dragged(event, d) {
        d.fx = event.x;
        d.fy = event.y;
    }

    function dragended(event, d) {
        if (!event.active) simulation.alphaTarget(0);
        d.fx = null;
        d.fy = null;
    }
}

// Get color for variable type
function getTypeColor(typeName) {
    if (typeName.includes('Vec')) return '#3b82f6';
    if (typeName.includes('Box')) return '#8b5cf6';
    if (typeName.includes('Rc') || typeName.includes('Arc')) return '#10b981';
    if (typeName.includes('String')) return '#f59e0b';
    return '#6b7280';
}

// Enhanced type color with comprehensive type mapping
function getEnhancedTypeColor(typeName, category) {
    // Comprehensive color mapping for specific types
    const typeColorMap = {
        // Smart Pointers - Purple/Violet family
        'Box': '#8b5cf6',           // Purple
        'Rc': '#a855f7',            // Purple-500
        'Arc': '#9333ea',           // Violet-600
        'RefCell': '#7c3aed',       // Violet-700
        'Cell': '#6d28d9',          // Violet-800
        'Weak': '#5b21b6',          // Violet-900

        // Collections - Blue family
        'Vec': '#3b82f6',           // Blue-500
        'VecDeque': '#2563eb',      // Blue-600
        'LinkedList': '#1d4ed8',    // Blue-700
        'HashMap': '#1e40af',       // Blue-800
        'BTreeMap': '#1e3a8a',      // Blue-900
        'HashSet': '#60a5fa',       // Blue-400
        'BTreeSet': '#93c5fd',      // Blue-300

        // String types - Orange/Amber family
        'String': '#f59e0b',        // Amber-500
        'str': '#d97706',           // Amber-600
        'OsString': '#b45309',      // Amber-700
        'OsStr': '#92400e',         // Amber-800
        'CString': '#78350f',       // Amber-900
        'CStr': '#fbbf24',          // Amber-400

        // Numeric types - Green family
        'i8': '#10b981',            // Emerald-500
        'i16': '#059669',           // Emerald-600
        'i32': '#047857',           // Emerald-700
        'i64': '#065f46',           // Emerald-800
        'i128': '#064e3b',          // Emerald-900
        'u8': '#34d399',            // Emerald-400
        'u16': '#6ee7b7',           // Emerald-300
        'u32': '#a7f3d0',           // Emerald-200
        'u64': '#d1fae5',           // Emerald-100
        'u128': '#ecfdf5',          // Emerald-50
        'f32': '#14b8a6',           // Teal-500
        'f64': '#0d9488',           // Teal-600
        'usize': '#0f766e',         // Teal-700
        'isize': '#115e59',         // Teal-800

        // Boolean and char - Pink family
        'bool': '#ec4899',          // Pink-500
        'char': '#db2777',          // Pink-600

        // Option and Result - Indigo family
        'Option': '#6366f1',        // Indigo-500
        'Result': '#4f46e5',        // Indigo-600
        'Some': '#4338ca',          // Indigo-700
        'None': '#3730a3',          // Indigo-800
        'Ok': '#312e81',            // Indigo-900
        'Err': '#6366f1',           // Indigo-500

        // Synchronization types - Red family
        'Mutex': '#ef4444',         // Red-500
        'RwLock': '#dc2626',        // Red-600
        'Condvar': '#b91c1c',       // Red-700
        'Barrier': '#991b1b',       // Red-800
        'Once': '#7f1d1d',          // Red-900

        // Channel types - Cyan family
        'Sender': '#06b6d4',        // Cyan-500
        'Receiver': '#0891b2',      // Cyan-600
        'mpsc': '#0e7490',          // Cyan-700

        // Path types - Lime family
        'Path': '#84cc16',          // Lime-500
        'PathBuf': '#65a30d',       // Lime-600

        // Time types - Yellow family
        'Duration': '#eab308',      // Yellow-500
        'Instant': '#ca8a04',       // Yellow-600
        'SystemTime': '#a16207',    // Yellow-700

        // IO types - Stone family
        'File': '#78716c',          // Stone-500
        'BufReader': '#57534e',     // Stone-600
        'BufWriter': '#44403c',     // Stone-700

        // Thread types - Rose family
        'Thread': '#f43f5e',        // Rose-500
        'JoinHandle': '#e11d48',    // Rose-600

        // Custom/Unknown types - Gray family
        'unknown': '#6b7280',       // Gray-500
        'custom': '#4b5563',        // Gray-600
    };

    // First, try exact type name match
    if (typeColorMap[typeName]) {
        return typeColorMap[typeName];
    }

    // Then try to match by type name contains
    for (const [type, color] of Object.entries(typeColorMap)) {
        if (typeName.includes(type)) {
            return color;
        }
    }

    // Extract generic base type and try to match
    const genericBase = getGenericBase(typeName);
    if (typeColorMap[genericBase]) {
        return typeColorMap[genericBase];
    }

    // Fall back to category-based colors
    switch (category) {
        case 'smart_pointer': return '#8b5cf6';  // Purple
        case 'collection': return '#3b82f6';     // Blue
        case 'string': return '#f59e0b';         // Amber
        case 'numeric': return '#10b981';        // Emerald
        case 'sync': return '#ef4444';           // Red
        case 'channel': return '#06b6d4';        // Cyan
        case 'path': return '#84cc16';           // Lime
        case 'time': return '#eab308';           // Yellow
        case 'io': return '#78716c';             // Stone
        case 'thread': return '#f43f5e';         // Rose
        default: return '#6b7280';               // Gray
    }
}

// Get type category for grouping with comprehensive type recognition
function getTypeCategory(typeName) {
    // Smart pointers
    if (typeName.includes('Box') || typeName.includes('Rc') || typeName.includes('Arc') ||
        typeName.includes('RefCell') || typeName.includes('Cell') || typeName.includes('Weak')) {
        return 'smart_pointer';
    }

    // Collections
    if (typeName.includes('Vec') || typeName.includes('HashMap') || typeName.includes('BTreeMap') ||
        typeName.includes('HashSet') || typeName.includes('BTreeSet') || typeName.includes('VecDeque') ||
        typeName.includes('LinkedList')) {
        return 'collection';
    }

    // String types
    if (typeName.includes('String') || typeName.includes('str') || typeName.includes('OsString') ||
        typeName.includes('OsStr') || typeName.includes('CString') || typeName.includes('CStr')) {
        return 'string';
    }

    // Numeric types
    if (typeName.match(/^[iuf]\d+$/) || typeName === 'usize' || typeName === 'isize' ||
        typeName === 'bool' || typeName === 'char') {
        return 'numeric';
    }

    // Synchronization types
    if (typeName.includes('Mutex') || typeName.includes('RwLock') || typeName.includes('Condvar') ||
        typeName.includes('Barrier') || typeName.includes('Once')) {
        return 'sync';
    }

    // Channel types
    if (typeName.includes('Sender') || typeName.includes('Receiver') || typeName.includes('mpsc')) {
        return 'channel';
    }

    // Path types
    if (typeName.includes('Path') || typeName.includes('PathBuf')) {
        return 'path';
    }

    // Time types
    if (typeName.includes('Duration') || typeName.includes('Instant') || typeName.includes('SystemTime')) {
        return 'time';
    }

    // IO types
    if (typeName.includes('File') || typeName.includes('BufReader') || typeName.includes('BufWriter')) {
        return 'io';
    }

    // Thread types
    if (typeName.includes('Thread') || typeName.includes('JoinHandle')) {
        return 'thread';
    }

    // Option and Result
    if (typeName.includes('Option') || typeName.includes('Result')) {
        return 'option_result';
    }

    return 'primitive';
}

// Get generic base type (Vec<T> -> Vec, Box<T> -> Box)
function getGenericBase(typeName) {
    const match = typeName.match(/^([^<]+)/);
    return match ? match[1] : typeName;
}

// Get complexity score from type with comprehensive scoring
function getComplexityFromType(typeName) {
    // Very high complexity (9-10)
    if (typeName.includes('HashMap') || typeName.includes('BTreeMap') ||
        typeName.includes('BTreeSet') || typeName.includes('LinkedList')) return 9;

    // High complexity (7-8)
    if (typeName.includes('Arc') || typeName.includes('Mutex') || typeName.includes('RwLock') ||
        typeName.includes('Condvar') || typeName.includes('Barrier')) return 8;
    if (typeName.includes('Rc') || typeName.includes('RefCell') || typeName.includes('HashSet') ||
        typeName.includes('VecDeque')) return 7;

    // Medium complexity (5-6)
    if (typeName.includes('Vec') || typeName.includes('Box') || typeName.includes('Option') ||
        typeName.includes('Result')) return 6;
    if (typeName.includes('String') || typeName.includes('PathBuf') || typeName.includes('OsString') ||
        typeName.includes('CString')) return 5;

    // Low complexity (3-4)
    if (typeName.includes('str') || typeName.includes('Path') || typeName.includes('OsStr') ||
        typeName.includes('CStr') || typeName.includes('Duration') || typeName.includes('Instant')) return 4;
    if (typeName.includes('Sender') || typeName.includes('Receiver') || typeName.includes('File') ||
        typeName.includes('Thread') || typeName.includes('JoinHandle')) return 3;

    // Very low complexity (1-2)
    if (typeName.match(/^[iuf]\d+$/) || typeName === 'usize' || typeName === 'isize' ||
        typeName === 'bool' || typeName === 'char') return 1;

    // Default for unknown types
    return 2;
}

// Get link color based on relationship type
function getLinkColor(linkType) {
    switch (linkType) {
        case 'same_type': return '#ef4444';
        case 'similar_category': return '#3b82f6';
        case 'generic_family': return '#10b981';
        default: return '#6b7280';
    }
}

// Get complexity level description
function getComplexityLevel(score) {
    if (score <= 2) return 'Simple';
    if (score <= 5) return 'Medium';
    if (score <= 8) return 'Complex';
    return 'Very Complex';
}

// Get complexity explanation
function getComplexityExplanation(score) {
    if (score <= 2) return 'Basic types with minimal performance overhead and simple memory usage';
    if (score <= 5) return 'Medium complexity with some memory management overhead';
    if (score <= 8) return 'Complex types involving heap allocation and smart pointers, performance considerations needed';
    return 'Very complex types with significant performance overhead, optimization recommended';
}

// Get type analysis information
function getTypeAnalysis(typeName, size) {
    const analysis = [];

    if (typeName.includes('Vec')) {
        analysis.push('• Dynamic array with heap allocation');
        analysis.push('• Grows automatically as needed');
        if (size > 1000) analysis.push('• Large allocation - consider capacity optimization');
    } else if (typeName.includes('Box')) {
        analysis.push('• Single heap allocation');
        analysis.push('• Unique ownership semantics');
    } else if (typeName.includes('Rc')) {
        analysis.push('• Reference counted smart pointer');
        analysis.push('• Shared ownership with runtime checks');
    } else if (typeName.includes('Arc')) {
        analysis.push('• Atomic reference counted pointer');
        analysis.push('• Thread-safe shared ownership');
    } else if (typeName.includes('String')) {
        analysis.push('• Growable UTF-8 string');
        analysis.push('• Heap allocated with capacity buffer');
    } else {
        analysis.push('• Basic type allocation');
    }

    if (size === 0) {
        analysis.push('• Zero-sized type (ZST)');
    } else if (size < 64) {
        analysis.push('• Small allocation - good for performance');
    } else if (size > 1024) {
        analysis.push('• Large allocation - monitor memory usage');
    }

    return analysis.join('<br>');
}

// Initialize generic types table
function initGenericTypesTable() {
    const tbody = document.getElementById('generic-types-table-body');
    if (!tbody) return;

    const genericTypes = window.analysisData.complex_types?.categorized_types?.generic_types || [];

    if (genericTypes.length === 0) {
        tbody.innerHTML = '<tr><td colspan="6" class="px-6 py-8 text-center text-gray-500 dark:text-gray-400">No generic types found</td></tr>';
        return;
    }

    tbody.innerHTML = genericTypes.map(type => `
        <tr class="hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
            <td class="px-6 py-4 text-gray-900 dark:text-gray-100">${type.var_name || 'System Allocation'}</td>
            <td class="px-6 py-4 text-gray-900 dark:text-gray-100">${formatTypeName(type.type_name || 'System Allocation')}</td>
            <td class="px-6 py-4 font-mono text-xs text-gray-900 dark:text-gray-100">${type.ptr}</td>
            <td class="px-6 py-4 text-gray-900 dark:text-gray-100">${formatBytes(type.size || 0)}</td>
            <td class="px-6 py-4 text-gray-900 dark:text-gray-100">N/A</td>
            <td class="px-6 py-4">
                <span class="px-2 py-1 rounded text-xs ${getComplexityColor(type.complexity_score)} text-white">
                    ${type.complexity_score || 0}
                </span>
            </td>
        </tr>
    `).join('');
}

// Initialize complex type analysis
function initComplexTypeAnalysis() {
    const tbody = document.getElementById('complex-type-analysis-table');
    if (!tbody) return;

    const complexTypeAnalysis = window.analysisData.complex_types?.complex_type_analysis || [];

    if (complexTypeAnalysis.length === 0) {
        tbody.innerHTML = '<tr><td colspan="6" class="px-6 py-8 text-center text-gray-500 dark:text-gray-400">No complex type analysis available</td></tr>';
        return;
    }

    tbody.innerHTML = complexTypeAnalysis.map(analysis => `
        <tr class="hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
            <td class="px-6 py-4 text-gray-900 dark:text-gray-100">${formatTypeName(analysis.type_name)}</td>
            <td class="px-6 py-4 text-center">
                <span class="px-2 py-1 rounded text-xs ${getComplexityColor(analysis.complexity_score)} text-white">
                    ${analysis.complexity_score}
                </span>
            </td>
            <td class="px-6 py-4 text-center">
                <span class="px-2 py-1 rounded text-xs ${getEfficiencyColor(analysis.memory_efficiency)} text-white">
                    ${analysis.memory_efficiency}%
                </span>
            </td>
            <td class="px-6 py-4 text-center text-gray-900 dark:text-gray-100">${analysis.allocation_count || 0}</td>
            <td class="px-6 py-4 text-center text-gray-900 dark:text-gray-100">${formatBytes(analysis.total_size || 0)}</td>
            <td class="px-6 py-4 text-gray-700 dark:text-gray-300">
                ${Array.isArray(analysis.optimization_suggestions) && analysis.optimization_suggestions.length > 0 
                    ? analysis.optimization_suggestions.join(', ') 
                    : '<span class="text-gray-400 italic">No optimization suggestions available</span>'}
            </td>
        </tr>
    `).join('');
}

// Initialize memory optimization recommendations
function initMemoryOptimizationRecommendations() {
    const container = document.getElementById('memory-optimization-recommendations');
    if (!container) return;

    const recommendations = window.analysisData.complex_types?.optimization_recommendations || [];

    if (recommendations.length === 0) {
        container.innerHTML = '<li class="text-gray-500 dark:text-gray-400">No specific recommendations available</li>';
        return;
    }

    container.innerHTML = recommendations.map(rec => `
        <li class="flex items-start">
            <i class="fa fa-lightbulb-o text-yellow-500 mr-2 mt-1"></i>
            <span class="dark:text-gray-200">${rec}</span>
        </li>
    `).join('');
}

// Initialize FFI risk chart
function initFFIRiskChart() {
    const ctx = document.getElementById('ffi-risk-chart');
    if (!ctx) return;

    const ffiData = window.analysisData.unsafe_ffi?.enhanced_ffi_data || [];

    const riskLevels = {
        'Low Risk': ffiData.filter(item => (item.safety_violations || 0) === 0).length,
        'Medium Risk': ffiData.filter(item => (item.safety_violations || 0) > 0 && (item.safety_violations || 0) <= 2).length,
        'High Risk': ffiData.filter(item => (item.safety_violations || 0) > 2).length
    };

    const isDark = document.documentElement.classList.contains('dark');

    // Destroy existing chart if it exists
    if (window.chartInstances['ffi-risk-chart']) {
        window.chartInstances['ffi-risk-chart'].destroy();
    }

    window.chartInstances['ffi-risk-chart'] = new Chart(ctx, {
        type: 'doughnut',
        data: {
            labels: Object.keys(riskLevels),
            datasets: [{
                data: Object.values(riskLevels),
                backgroundColor: ['#10b981', '#f59e0b', '#ef4444'],
                borderColor: isDark ? '#374151' : '#ffffff',
                borderWidth: 2
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    labels: {
                        color: isDark ? '#f9fafb' : '#374151',
                        font: {
                            size: 12
                        }
                    }
                },
                tooltip: {
                    backgroundColor: isDark ? '#1f2937' : '#ffffff',
                    titleColor: isDark ? '#f9fafb' : '#374151',
                    bodyColor: isDark ? '#f9fafb' : '#374151',
                    borderColor: isDark ? '#374151' : '#e5e7eb',
                    borderWidth: 1
                }
            }
        }
    });
}

// Initialize complex type analysis chart
function initComplexTypeAnalysisChart() {
    const ctx = document.getElementById('complex-type-analysis-chart');
    if (!ctx) return;

    const complexTypeAnalysis = window.analysisData.complex_types?.complex_type_analysis || [];

    if (complexTypeAnalysis.length === 0) {
        // Show empty state
        const container = ctx.parentElement;
        container.innerHTML = `
            <div class="h-64 flex items-center justify-center text-gray-500 dark:text-gray-400">
                <div class="text-center">
                    <i class="fa fa-chart-bar text-4xl mb-4"></i>
                    <p class="text-lg font-semibold mb-2">No Complex Type Data</p>
                    <p class="text-sm">No complex type analysis data available</p>
                </div>
            </div>
        `;
        return;
    }

    const isDark = document.documentElement.classList.contains('dark');

    // Destroy existing chart if it exists
    if (window.chartInstances['complex-type-analysis-chart']) {
        window.chartInstances['complex-type-analysis-chart'].destroy();
    }

    window.chartInstances['complex-type-analysis-chart'] = new Chart(ctx, {
        type: 'scatter',
        data: {
            datasets: [{
                label: 'Type Complexity vs Memory Efficiency',
                data: complexTypeAnalysis.map(analysis => ({
                    x: analysis.complexity_score || 0,
                    y: analysis.memory_efficiency || 0,
                    typeName: analysis.type_name
                })),
                backgroundColor: 'rgba(59, 130, 246, 0.6)',
                borderColor: 'rgba(59, 130, 246, 1)',
                borderWidth: 2,
                pointRadius: 6,
                pointHoverRadius: 8
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: {
                    title: {
                        display: true,
                        text: 'Complexity Score',
                        color: isDark ? '#f9fafb' : '#374151'
                    },
                    ticks: {
                        color: isDark ? '#d1d5db' : '#6b7280'
                    },
                    grid: {
                        color: isDark ? '#374151' : '#e5e7eb'
                    }
                },
                y: {
                    title: {
                        display: true,
                        text: 'Memory Efficiency (%)',
                        color: isDark ? '#f9fafb' : '#374151'
                    },
                    ticks: {
                        color: isDark ? '#d1d5db' : '#6b7280'
                    },
                    grid: {
                        color: isDark ? '#374151' : '#e5e7eb'
                    }
                }
            },
            plugins: {
                legend: {
                    labels: {
                        color: isDark ? '#f9fafb' : '#374151'
                    }
                },
                tooltip: {
                    backgroundColor: isDark ? '#1f2937' : '#ffffff',
                    titleColor: isDark ? '#f9fafb' : '#374151',
                    bodyColor: isDark ? '#f9fafb' : '#374151',
                    borderColor: isDark ? '#374151' : '#e5e7eb',
                    borderWidth: 1,
                    callbacks: {
                        title: function (context) {
                            return context[0].raw.typeName || 'Unknown Type';
                        },
                        label: function (context) {
                            return [
                                `Complexity: ${context.parsed.x}`,
                                `Efficiency: ${context.parsed.y}%`
                            ];
                        }
                    }
                }
            }
        }
    });
}

// Format type name for better display
function formatTypeName(typeName) {
    if (!typeName || typeName === 'unknown') return 'System Allocation';
    // Simplify complex type names
    return typeName
        .replace(/alloc::/g, '')
        .replace(/std::/g, '')
        .replace(/::Vec/g, 'Vec')
        .replace(/::Box/g, 'Box')
        .replace(/::Rc/g, 'Rc')
        .replace(/::Arc/g, 'Arc')
        .replace(/::String/g, 'String');
}

// Format timestamp relative to start time
function formatTimestamp(timestamp, minTime) {
    const relativeMs = Math.round((timestamp - minTime) / 1000000); // Convert nanoseconds to milliseconds
    return `${relativeMs}ms`;
}

// Enhanced summary statistics with comprehensive data analysis
function initEnhancedSummaryStats() {
    console.log('📊 Initializing enhanced summary statistics...');
    
    try {
        // Get merged data from all sources
        const memoryAllocations = window.analysisData.memory_analysis?.allocations || [];
        const complexAllocations = window.analysisData.complex_types?.allocations || [];
        const unsafeAllocations = window.analysisData.unsafe_ffi?.allocations || [];
        
        // Merge all data sources for comprehensive analysis
        const allData = mergeAllDataSources(memoryAllocations, complexAllocations, unsafeAllocations);
        
        // Calculate comprehensive statistics
        const stats = calculateComprehensiveStats(allData);
        
        // Update enhanced dashboard
        updateElement('total-allocations', stats.totalAllocations);
        updateElement('allocation-rate', `${stats.allocationRate.toFixed(1)}/ms`);
        updateElement('active-variables', stats.activeVariables);
        updateElement('variable-types', `${stats.uniqueTypes} types`);
        updateElement('borrow-operations', stats.totalBorrows);
        updateElement('max-concurrent', `Max: ${stats.maxConcurrent}`);
        updateElement('safety-score', `${stats.safetyScore}%`);
        updateElement('ffi-tracked', `${stats.ffiTracked} FFI`);
        
        console.log('✅ Enhanced dashboard updated successfully');
    } catch (error) {
        console.error('❌ Error initializing enhanced stats:', error);
    }
}

// Merge data from all sources with comprehensive field mapping
function mergeAllDataSources(memory, complex, unsafe) {
    const dataMap = new Map();
    
    // Add memory analysis data (has lifetime_ms)
    memory.forEach(alloc => {
        if (alloc.ptr) {
            dataMap.set(alloc.ptr, { ...alloc, source: 'memory' });
        }
    });
    
    // Merge complex types data (has extended fields)
    complex.forEach(alloc => {
        if (alloc.ptr) {
            const existing = dataMap.get(alloc.ptr) || {};
            dataMap.set(alloc.ptr, { 
                ...existing, 
                ...alloc, 
                source: existing.source ? `${existing.source}+complex` : 'complex'
            });
        }
    });
    
    // Merge unsafe FFI data (has safety info)
    unsafe.forEach(alloc => {
        if (alloc.ptr) {
            const existing = dataMap.get(alloc.ptr) || {};
            dataMap.set(alloc.ptr, { 
                ...existing, 
                ...alloc, 
                source: existing.source ? `${existing.source}+unsafe` : 'unsafe'
            });
        }
    });
    
    return Array.from(dataMap.values());
}

// Calculate comprehensive statistics from merged data
function calculateComprehensiveStats(allData) {
    const validData = allData.filter(d => d.var_name && d.var_name !== 'unknown');
    
    // Basic counts
    const totalAllocations = validData.length;
    const uniqueVars = new Set(validData.map(d => d.var_name)).size;
    const uniqueTypes = new Set(validData.map(d => d.type_name)).size;
    
    // Time-based calculations
    const timestamps = validData.map(d => d.timestamp_alloc).filter(t => t);
    const timeRange = timestamps.length > 0 ? (Math.max(...timestamps) - Math.min(...timestamps)) / 1000000 : 1;
    const allocationRate = totalAllocations / Math.max(timeRange, 1);
    
    // Borrow analysis
    let totalBorrows = 0;
    let maxConcurrent = 0;
    validData.forEach(d => {
        if (d.borrow_info) {
            totalBorrows += (d.borrow_info.immutable_borrows || 0) + (d.borrow_info.mutable_borrows || 0);
            maxConcurrent = Math.max(maxConcurrent, d.borrow_info.max_concurrent_borrows || 0);
        }
    });
    
    // Safety analysis
    const ffiTracked = validData.filter(d => d.ffi_tracked).length;
    const leaked = validData.filter(d => d.is_leaked).length;
    const withSafetyViolations = validData.filter(d => d.safety_violations && d.safety_violations.length > 0).length;
    const safetyScore = Math.max(0, 100 - (leaked * 20) - (withSafetyViolations * 10));
    
    return {
        totalAllocations,
        activeVariables: uniqueVars,
        uniqueTypes,
        allocationRate,
        totalBorrows,
        maxConcurrent,
        ffiTracked,
        safetyScore: Math.round(safetyScore)
    };
}

// Helper function to safely update DOM elements
function updateElement(id, value) {
    const element = document.getElementById(id);
    if (element) {
        element.textContent = value;
    }
}

// Utility function to format bytes
function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

// Show empty state when no user variables found
function showEmptyLifetimeState() {
    const container = document.getElementById('lifetimeVisualization');
    if (!container) return;

    container.innerHTML = `
        <div class="text-center py-8 text-gray-500 dark:text-gray-400">
            <i class="fa fa-info-circle text-2xl mb-2"></i>
            <p>No user-defined variables found in lifetime data</p>
            <p class="text-sm mt-1">Use track_var! macro to track variable lifetimes</p>
        </div>
    `;
}

// Utility functions
function updateElement(id, value) {
    const element = document.getElementById(id);
    if (element) {
        element.textContent = value;
    }
}

function getComplexityColor(score) {
    if (score <= 2) return '#10b981';  // Green - Low complexity
    if (score <= 5) return '#eab308';  // Yellow - Medium complexity  
    if (score <= 8) return '#f97316';  // Orange - High complexity
    return '#ef4444';                  // Red - Very high complexity
}

function getEfficiencyColor(efficiency) {
    if (efficiency >= 80) return 'bg-green-500';
    if (efficiency >= 60) return 'bg-yellow-500';
    if (efficiency >= 40) return 'bg-orange-500';
    return 'bg-red-500';
}

// Update KPI Cards
function updateKPICards(data) {
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const total = allocs.reduce((s,a)=>s+(a.size||0),0);
    const active = allocs.filter(a=>!a.timestamp_dealloc).length;
    const safetyScore = calculateSafetyScore(allocs);
    
    updateElement('total-allocations', allocs.length.toLocaleString());
    updateElement('active-variables', active.toLocaleString());
    updateElement('total-memory', formatBytes(total));
    updateElement('safety-score', safetyScore + '%');
}

// Calculate Safety Score
function calculateSafetyScore(allocs) {
    if (!allocs.length) return 100;
    const leaked = allocs.filter(a => a.is_leaked).length;
    const violations = allocs.filter(a => a.safety_violations && a.safety_violations.length > 0).length;
    return Math.max(0, 100 - (leaked * 20) - (violations * 10));
}

// Theme Toggle Functionality
function initThemeToggle() {
    const toggleBtn = document.getElementById('theme-toggle');
    if (!toggleBtn) return;
    
    // Check local storage for theme
    const savedTheme = localStorage.getItem('memscope-theme') || 'light';
    applyTheme(savedTheme === 'dark');
    
    toggleBtn.addEventListener('click', () => {
        const isDark = document.documentElement.classList.contains('dark');
        const newTheme = isDark ? 'light' : 'dark';
        
        applyTheme(newTheme === 'dark');
        localStorage.setItem('memscope-theme', newTheme);
        
        // Update button text
        const icon = toggleBtn.querySelector('i');
        const text = toggleBtn.querySelector('span');
        if (newTheme === 'dark') {
            icon.className = 'fa fa-sun';
            text.textContent = 'Light Mode';
        } else {
            icon.className = 'fa fa-moon';
            text.textContent = 'Dark Mode';
        }
        
        console.log('🎨 Theme switched to:', newTheme);
    });
}

// 应用主题
function applyTheme(isDark) {
    const html = document.documentElement;
    if (isDark) {
        html.classList.add('dark');
    } else {
        html.classList.remove('dark');
    }
}

// Update Memory Allocation Table
function updateAllocationsTable(data) {
    const allocTable = document.getElementById('allocTable');
    if (!allocTable) return;
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const top = allocs.slice().sort((a,b)=>(b.size||0)-(a.size||0)).slice(0,50);
    
    allocTable.innerHTML = top.map(a => {
        const status = a.is_leaked ? 'Leaked' : (a.timestamp_dealloc ? 'Freed' : 'Active');
        const statusClass = a.is_leaked ? 'status-leaked' : (a.timestamp_dealloc ? 'status-freed' : 'status-active');
        
        return `<tr>
            <td>${a.var_name || 'Unknown'}</td>
            <td>${formatTypeName(a.type_name || 'Unknown')}</td>
            <td>${formatBytes(a.size || 0)}</td>
            <td><span class="status-badge ${statusClass}">${status}</span></td>
        </tr>`;
    }).join('');
}

// Update Unsafe Risk Table
function updateUnsafeTable(data) {
    const unsafeTable = document.getElementById('unsafeTable');
    if (!unsafeTable) return;
    
    const root = data.unsafe_ffi || {};
    const ops = root.enhanced_ffi_data || root.unsafe_operations || root.allocations || [];
    
    unsafeTable.innerHTML = (ops || []).slice(0, 50).map(op => {
        const riskLevel = op.risk_level || ((op.safety_violations||[]).length > 2 ? 'High' : 
                         ((op.safety_violations||[]).length > 0 ? 'Medium' : 'Low'));
        
        const riskText = riskLevel === 'High' ? 'High Risk' : (riskLevel === 'Medium' ? 'Medium Risk' : 'Low Risk');
        const riskClass = riskLevel === 'High' ? 'risk-high' : (riskLevel === 'Medium' ? 'risk-medium' : 'risk-low');
        
        return `<tr>
            <td>${op.location || op.var_name || 'Unknown'}</td>
            <td>${op.operation_type || op.type_name || 'Unknown'}</td>
            <td><span class="status-badge ${riskClass}">${riskText}</span></td>
        </tr>`;
    }).join('');
}

// Initialize Charts
function initCharts(data) {
    console.log('📊 Initializing charts...');
    
    // Memory type distribution chart
    initTypeChart(data);
    
    // Memory timeline chart
    initTimelineChart(data);
    
    // Type treemap chart
    initTreemapChart(data);
    
    // FFI risk chart
    initFFIRiskChart(data);
    
    // Memory growth trends
    initGrowthTrends(data);
    
    // Memory fragmentation
    initMemoryFragmentation(data);
    
    // Variable relationship graph
    initVariableGraph(data);
}

// Memory Type Distribution Chart
function initTypeChart(data) {
    const ctx = document.getElementById('typeChart');
    if (!ctx) return;
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const byType = {};
    
    allocs.forEach(a => {
        const type = a.type_name || 'Unknown';
        byType[type] = (byType[type] || 0) + (a.size || 0);
    });
    
    const top = Object.entries(byType).sort((a,b) => b[1] - a[1]).slice(0, 8);
    
    if (top.length > 0 && window.Chart) {
        const chart = new Chart(ctx, {
            type: 'bar',
            data: {
                labels: top.map(x => {
                    const formatted = formatTypeName(x[0]);
                    return formatted.length > 15 ? formatted.substring(0, 12) + '...' : formatted;
                }),
                datasets: [{
                    label: 'Memory Usage',
                    data: top.map(x => x[1]),
                    backgroundColor: '#2563eb',
                    borderRadius: 6
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { display: false }
                },
                scales: {
                    x: {
                        ticks: {
                            maxRotation: 45,
                            minRotation: 0,
                            font: {
                                size: 10
                            }
                        }
                    },
                    y: { 
                        beginAtZero: true,
                        ticks: {
                            callback: function(value) {
                                return formatBytes(value);
                            }
                        }
                    }
                }
            }
        });
    }
}

// Memory Timeline Chart
function initTimelineChart(data) {
    const ctx = document.getElementById('timelineChart');
    if (!ctx) return;
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const sorted = allocs.slice().sort((a,b) => (a.timestamp_alloc||0) - (b.timestamp_alloc||0));
    
    let cumulative = 0;
    const points = [];
    const step = Math.max(1, Math.floor(sorted.length / 30));
    
    for (let i = 0; i < sorted.length; i += step) {
        cumulative += sorted[i].size || 0;
        points.push({ x: i, y: cumulative });
    }
    
    if (points.length > 1 && window.Chart) {
        const chart = new Chart(ctx, {
            type: 'line',
            data: {
                labels: points.map(p => p.x),
                datasets: [{
                    label: 'Cumulative Memory',
                    data: points.map(p => p.y),
                    borderColor: '#059669',
                    backgroundColor: 'rgba(5, 150, 105, 0.1)',
                    fill: true,
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { display: false }
                },
                scales: {
                    y: { 
                        beginAtZero: true,
                        ticks: {
                            callback: function(value) {
                                return formatBytes(value);
                            }
                        }
                    }
                }
            }
        });
    }
}

// Type Treemap Chart
function initTreemapChart(data) {
    const container = document.getElementById('treemap');
    if (!container) return;
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const byType = {};
    
    allocs.forEach(a => {
        const type = a.type_name || 'Unknown';
        byType[type] = (byType[type] || 0) + (a.size || 0);
    });
    
    const top = Object.entries(byType).sort((a,b) => b[1] - a[1]).slice(0, 12);
    const totalSize = top.reduce((sum, [, size]) => sum + size, 0);
    
    if (totalSize > 0) {
        let html = '<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 8px; height: 100%; padding: 16px;">';
        
        top.forEach(([type, size], index) => {
            const percentage = (size / totalSize) * 100;
            const color = `hsl(${index * 30}, 70%, 55%)`;
            
            html += `
                <div style="
                    background: ${color};
                    color: white;
                    padding: 12px;
                    border-radius: 8px;
                    font-size: 11px;
                    font-weight: 600;
                    display: flex;
                    flex-direction: column;
                    justify-content: center;
                    text-align: center;
                    min-height: 80px;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                    transition: transform 0.2s ease;
                " title="${type}: ${formatBytes(size)}" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'">
                    <div style="margin-bottom: 4px;">${formatTypeName(type)}</div>
                    <div style="font-size: 10px; opacity: 0.9;">${formatBytes(size)}</div>
                    <div style="font-size: 9px; opacity: 0.7;">${percentage.toFixed(1)}%</div>
                </div>
            `;
        });
        
        html += '</div>';
        container.innerHTML = html;
    } else {
        container.innerHTML = '<div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">No data available</div>';
    }
}

// FFI Risk Chart
function initFFIRiskChart(data) {
    const ctx = document.getElementById('ffi-risk-chart');
    if (!ctx) return;
    
    const ffiData = data.unsafe_ffi?.enhanced_ffi_data || [];
    
    const riskLevels = {
        'Low Risk': ffiData.filter(item => (item.safety_violations || []).length === 0).length,
        'Medium Risk': ffiData.filter(item => (item.safety_violations || []).length > 0 && (item.safety_violations || []).length <= 2).length,
        'High Risk': ffiData.filter(item => (item.safety_violations || []).length > 2).length
    };
    
    if (window.Chart) {
        const chart = new Chart(ctx, {
            type: 'doughnut',
            data: {
                labels: Object.keys(riskLevels),
                datasets: [{
                    data: Object.values(riskLevels),
                    backgroundColor: ['#059669', '#ea580c', '#dc2626'],
                    borderWidth: 2,
                    borderColor: 'var(--bg-primary)'
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        position: 'bottom',
                        labels: {
                            padding: 20,
                            usePointStyle: true
                        }
                    }
                }
            }
        });
    }
}

// Add missing chart and graph functions
function initGrowthTrends(data) {
    const container = document.getElementById('growth');
    if (!container) return;
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    if (allocs.length === 0) {
        container.innerHTML = '<div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">No growth data available</div>';
        return;
    }
    
    // Simple growth visualization
    const sorted = allocs.slice().sort((a,b) => (a.timestamp_alloc||0) - (b.timestamp_alloc||0));
    let cumulative = 0;
    const points = [];
    
    for (let i = 0; i < Math.min(sorted.length, 20); i++) {
        cumulative += sorted[i].size || 0;
        points.push(cumulative);
    }
    
    const maxValue = Math.max(...points);
    let html = '<div style="display: flex; align-items: end; height: 200px; gap: 4px; padding: 20px;">';
    
    points.forEach((value, i) => {
        const height = (value / maxValue) * 160;
        html += `
            <div style="
                width: 12px;
                height: ${height}px;
                background: linear-gradient(to top, #2563eb, #3b82f6);
                border-radius: 2px;
                margin: 0 1px;
            " title="Step ${i + 1}: ${formatBytes(value)}"></div>
        `;
    });
    
    html += '</div>';
    container.innerHTML = html;
}

function initMemoryFragmentation(data) {
    const container = document.getElementById('memoryFragmentation');
    if (!container) return;
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const totalMemory = allocs.reduce((sum, a) => sum + (a.size || 0), 0);
    const activeMemory = allocs.filter(a => !a.timestamp_dealloc).reduce((sum, a) => sum + (a.size || 0), 0);
    const fragmentationRate = totalMemory > 0 ? ((totalMemory - activeMemory) / totalMemory * 100) : 0;
    
    container.innerHTML = `
        <div style="padding: 20px;">
            <div style="display: flex; justify-content: space-between; margin-bottom: 16px;">
                <div>
                    <div style="color: var(--text-secondary); font-size: 0.9rem;">Fragmentation Rate</div>
                    <div style="font-size: 2rem; font-weight: 700; color: ${fragmentationRate > 30 ? '#dc2626' : fragmentationRate > 15 ? '#ea580c' : '#059669'};">
                        ${fragmentationRate.toFixed(1)}%
                    </div>
                </div>
                <div>
                    <div style="color: var(--text-secondary); font-size: 0.9rem;">Active Memory</div>
                    <div style="font-size: 1.2rem; font-weight: 600;">${formatBytes(activeMemory)}</div>
                </div>
            </div>
            <div style="background: var(--bg-secondary); height: 8px; border-radius: 4px; overflow: hidden;">
                <div style="
                    background: linear-gradient(to right, #059669, #ea580c);
                    width: ${Math.min(100, fragmentationRate)}%;
                    height: 100%;
                    border-radius: 4px;
                    transition: width 0.8s ease;
                "></div>
            </div>
            <div style="margin-top: 12px; font-size: 0.8rem; color: var(--text-secondary);">
                ${fragmentationRate > 30 ? 'High fragmentation detected' : fragmentationRate > 15 ? 'Moderate fragmentation' : 'Low fragmentation'}
            </div>
        </div>
    `;
}

function initVariableGraph(data) {
    const container = document.getElementById('graph');
    if (!container) return;
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    if (allocs.length === 0) {
        container.innerHTML = '<div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">No relationship data available</div>';
        return;
    }
    
    // Create a simple node-link visualization
    const nodes = allocs.slice(0, 20).map((a, i) => ({
        id: i,
        name: a.var_name || `var_${i}`,
        type: a.type_name || 'unknown',
        size: a.size || 0,
        x: 50 + (i % 4) * 80,
        y: 50 + Math.floor(i / 4) * 60
    }));
    
    let svg = `
        <svg width="100%" height="100%" style="background: transparent;">
            <defs>
                <filter id="glow">
                    <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
                    <feMerge>
                        <feMergeNode in="coloredBlur"/>
                        <feMergeNode in="SourceGraphic"/>
                    </feMerge>
                </filter>
            </defs>
    `;
    
    // Add links between nearby nodes
    for (let i = 0; i < nodes.length - 1; i++) {
        if (i % 4 !== 3) { // Connect horizontally
            svg += `<line x1="${nodes[i].x}" y1="${nodes[i].y}" x2="${nodes[i+1].x}" y2="${nodes[i+1].y}" stroke="var(--border-light)" stroke-width="1" opacity="0.3"/>`;
        }
        if (i < nodes.length - 4) { // Connect vertically
            svg += `<line x1="${nodes[i].x}" y1="${nodes[i].y}" x2="${nodes[i+4].x}" y2="${nodes[i+4].y}" stroke="var(--border-light)" stroke-width="1" opacity="0.3"/>`;
        }
    }
    
    // Add nodes
    nodes.forEach(node => {
        const radius = Math.max(8, Math.min(20, Math.log(node.size + 1) * 2));
        const color = node.type.includes('String') ? '#fbbf24' : 
                     node.type.includes('Vec') ? '#3b82f6' : 
                     node.type.includes('Box') || node.type.includes('Rc') ? '#8b5cf6' : '#6b7280';
        
        svg += `
            <circle 
                cx="${node.x}" 
                cy="${node.y}" 
                r="${radius}" 
                fill="${color}" 
                stroke="white" 
                stroke-width="2" 
                filter="url(#glow)"
                style="cursor: pointer;"
                onmouseover="this.r.baseVal.value = ${radius + 3}"
                onmouseout="this.r.baseVal.value = ${radius}"
            >
                <title>${node.name} (${node.type})</title>
            </circle>
            <text 
                x="${node.x}" 
                y="${node.y + radius + 12}" 
                text-anchor="middle" 
                font-size="10" 
                fill="var(--text-primary)"
                style="font-weight: 500;"
            >${node.name.length > 8 ? node.name.substring(0, 8) + '...' : node.name}</text>
        `;
    });
    
    svg += '</svg>';
    container.innerHTML = svg;
}

// Initialize lifetime visualization
function initLifetimeVisualization(data) {
    const container = document.getElementById('lifetimes');
    if (!container) return;
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    if (allocs.length === 0) {
        container.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--text-secondary);">No lifetime data available</div>';
        return;
    }
    
    // Show top allocations by lifetime
    const withLifetime = allocs.filter(a => a.lifetime_ms || (a.timestamp_alloc && a.timestamp_dealloc));
    const sorted = withLifetime.sort((a, b) => {
        const aLifetime = a.lifetime_ms || (a.timestamp_dealloc - a.timestamp_alloc);
        const bLifetime = b.lifetime_ms || (b.timestamp_dealloc - b.timestamp_alloc);
        return bLifetime - aLifetime;
    }).slice(0, 10);
    
    if (sorted.length === 0) {
        container.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--text-secondary);">No lifetime data available</div>';
        return;
    }
    
    let html = '<div style="padding: 16px;">';
    
    sorted.forEach((alloc, index) => {
        const lifetime = alloc.lifetime_ms || (alloc.timestamp_dealloc - alloc.timestamp_alloc);
        const isActive = !alloc.timestamp_dealloc;
        const varName = alloc.var_name || `allocation_${index}`;
        const size = formatBytes(alloc.size || 0);
        
        html += `
            <div style="
                margin-bottom: 12px; 
                padding: 12px; 
                background: var(--bg-secondary); 
                border-radius: 8px;
                border-left: 4px solid ${isActive ? '#059669' : '#2563eb'};
            ">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                    <span style="font-weight: 600; color: var(--text-primary);">${varName}</span>
                    <span style="font-size: 0.9rem; color: var(--text-secondary);">${size}</span>
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <span style="font-size: 0.8rem; color: var(--text-secondary);">
                        ${formatTypeName(alloc.type_name || 'Unknown')}
                    </span>
                    <span style="
                        font-size: 0.8rem; 
                        font-weight: 600; 
                        color: ${isActive ? '#059669' : '#2563eb'};
                    ">
                        ${isActive ? 'Active' : `${lifetime}ms`}
                    </span>
                </div>
            </div>
        `;
    });
    
    html += '</div>';
    container.innerHTML = html;
}

// Helper function to update elements
function updateElement(id, value) {
    const element = document.getElementById(id);
    if (element) {
        element.textContent = value;
    }
}

// Original dashboard functions from dashboard.html
function renderKpis() {
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const total = allocs.reduce((s,a)=>s+(a.size||0),0);
    const active = allocs.filter(a=>!a.timestamp_dealloc).length;
    const leaks = allocs.filter(a=>a.is_leaked).length;
    const safety = Math.max(0, 100 - (leaks * 20));
    
    updateElement('total-allocations', allocs.length.toLocaleString());
    updateElement('active-variables', active.toLocaleString());
    updateElement('total-memory', formatBytes(total));
    updateElement('safety-score', safety + '%');
}

function populateAllocationsTable() {
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const allocTable = document.getElementById('allocTable');
    if (!allocTable) return;
    
    const top = allocs.slice().sort((a,b)=>(b.size||0)-(a.size||0)).slice(0,50);
    allocTable.innerHTML = top.map(a => {
        const status = a.is_leaked ? 'Leaked' : (a.timestamp_dealloc ? 'Freed' : 'Active');
        const statusClass = a.is_leaked ? 'status-leaked' : (a.timestamp_dealloc ? 'status-freed' : 'status-active');
        
        return `<tr>
            <td>${a.var_name || 'Unknown'}</td>
            <td>${formatTypeName(a.type_name || 'Unknown')}</td>
            <td>${formatBytes(a.size || 0)}</td>
            <td><span class="status-badge ${statusClass}">${status}</span></td>
        </tr>`;
    }).join('');
}

function renderTypeChart() {
    const ctx = document.getElementById('typeChart');
    if (!ctx || !window.Chart) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const byType = {};
    
    allocs.forEach(a => {
        const type = a.type_name || 'Unknown';
        byType[type] = (byType[type] || 0) + (a.size || 0);
    });
    
    const top = Object.entries(byType).sort((a,b) => b[1] - a[1]).slice(0, 8);
    
    if (top.length > 0) {
        new Chart(ctx, {
            type: 'bar',
            data: {
                labels: top.map(x => formatTypeName(x[0])),
                datasets: [{
                    label: 'Memory Usage',
                    data: top.map(x => x[1]),
                    backgroundColor: '#2563eb',
                    borderRadius: 6
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: { legend: { display: false } },
                scales: {
                    y: { 
                        beginAtZero: true,
                        ticks: { callback: function(value) { return formatBytes(value); } }
                    }
                }
            }
        });
    }
}

function renderTimelineChart() {
    const ctx = document.getElementById('timelineChart');
    if (!ctx || !window.Chart) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const sorted = allocs.slice().sort((a,b) => (a.timestamp_alloc||0) - (b.timestamp_alloc||0));
    
    let cumulative = 0;
    const points = [];
    const step = Math.max(1, Math.floor(sorted.length / 30));
    
    for (let i = 0; i < sorted.length; i += step) {
        cumulative += sorted[i].size || 0;
        points.push({ x: i, y: cumulative });
    }
    
    if (points.length > 1) {
        new Chart(ctx, {
            type: 'line',
            data: {
                labels: points.map(p => p.x),
                datasets: [{
                    label: 'Cumulative Memory',
                    data: points.map(p => p.y),
                    borderColor: '#059669',
                    backgroundColor: 'rgba(5, 150, 105, 0.1)',
                    fill: true,
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: { legend: { display: false } },
                scales: {
                    y: { 
                        beginAtZero: true,
                        ticks: { callback: function(value) { return formatBytes(value); } }
                    }
                }
            }
        });
    }
}

function renderTreemap() {
    const container = document.getElementById('treemap');
    if (!container) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const byType = {};
    
    allocs.forEach(a => {
        const type = a.type_name || 'Unknown';
        byType[type] = (byType[type] || 0) + (a.size || 0);
    });
    
    const top = Object.entries(byType).sort((a,b) => b[1] - a[1]).slice(0, 12);
    const totalSize = top.reduce((sum, [, size]) => sum + size, 0);
    
    if (totalSize > 0) {
        let html = '<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 8px; height: 100%; padding: 16px;">';
        
        top.forEach(([type, size], index) => {
            const percentage = (size / totalSize) * 100;
            const color = `hsl(${index * 30}, 70%, 55%)`;
            
            html += `
                <div style="
                    background: ${color};
                    color: white;
                    padding: 12px;
                    border-radius: 8px;
                    font-size: 11px;
                    font-weight: 600;
                    display: flex;
                    flex-direction: column;
                    justify-content: center;
                    text-align: center;
                    min-height: 80px;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                    transition: transform 0.2s ease;
                " title="${type}: ${formatBytes(size)}" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'">
                    <div style="margin-bottom: 4px;">${formatTypeName(type)}</div>
                    <div style="font-size: 10px; opacity: 0.9;">${formatBytes(size)}</div>
                    <div style="font-size: 9px; opacity: 0.7;">${percentage.toFixed(1)}%</div>
                </div>
            `;
        });
        
        html += '</div>';
        container.innerHTML = html;
    } else {
        container.innerHTML = '<div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">No data available</div>';
    }
}

function renderLifetimes() {
    const container = document.getElementById('lifetimes');
    if (!container) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const withLifetime = allocs.filter(a => a.lifetime_ms || (a.timestamp_alloc && a.timestamp_dealloc));
    const sorted = withLifetime.sort((a, b) => {
        const aLifetime = a.lifetime_ms || (a.timestamp_dealloc - a.timestamp_alloc);
        const bLifetime = b.lifetime_ms || (b.timestamp_dealloc - b.timestamp_alloc);
        return bLifetime - aLifetime;
    }).slice(0, 10);
    
    if (sorted.length === 0) {
        container.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--text-secondary);">No lifetime data available</div>';
        return;
    }
    
    let html = '<div style="padding: 16px;">';
    
    sorted.forEach((alloc, index) => {
        const lifetime = alloc.lifetime_ms || (alloc.timestamp_dealloc - alloc.timestamp_alloc);
        const isActive = !alloc.timestamp_dealloc;
        const varName = alloc.var_name || `allocation_${index}`;
        const size = formatBytes(alloc.size || 0);
        
        html += `
            <div style="
                margin-bottom: 12px; 
                padding: 12px; 
                background: var(--bg-secondary); 
                border-radius: 8px;
                border-left: 4px solid ${isActive ? '#059669' : '#2563eb'};
            ">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                    <span style="font-weight: 600; color: var(--text-primary);">${varName}</span>
                    <span style="font-size: 0.9rem; color: var(--text-secondary);">${size}</span>
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <span style="font-size: 0.8rem; color: var(--text-secondary);">
                        ${formatTypeName(alloc.type_name || 'Unknown')}
                    </span>
                    <span style="
                        font-size: 0.8rem; 
                        font-weight: 600; 
                        color: ${isActive ? '#059669' : '#2563eb'};
                    ">
                        ${isActive ? 'Active' : `${lifetime}ms`}
                    </span>
                </div>
            </div>
        `;
    });
    
    html += '</div>';
    container.innerHTML = html;
}

function renderFFI() {
    // First try the chart container
    const chartContainer = document.getElementById('ffi-risk-chart');
    if (chartContainer && window.Chart) {
        const data = window.analysisData || {};
        const ffiData = data.unsafe_ffi || {};
        const operations = ffiData.enhanced_ffi_data || ffiData.unsafe_operations || [];
        
        if (operations.length > 0) {
            const highRisk = operations.filter(op => (op.safety_violations || []).length > 2).length;
            const mediumRisk = operations.filter(op => (op.safety_violations || []).length > 0 && (op.safety_violations || []).length <= 2).length;
            const lowRisk = operations.filter(op => (op.safety_violations || []).length === 0).length;
            
            new Chart(chartContainer, {
                type: 'doughnut',
                data: {
                    labels: ['Low Risk', 'Medium Risk', 'High Risk'],
                    datasets: [{
                        data: [lowRisk, mediumRisk, highRisk],
                        backgroundColor: ['#059669', '#ea580c', '#dc2626'],
                        borderWidth: 2,
                        borderColor: 'var(--bg-primary)'
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {
                        legend: {
                            position: 'bottom',
                            labels: {
                                padding: 20,
                                usePointStyle: true,
                                generateLabels: function(chart) {
                                    const data = chart.data;
                                    return data.labels.map((label, i) => ({
                                        text: `${label}: ${data.datasets[0].data[i]}`,
                                        fillStyle: data.datasets[0].backgroundColor[i],
                                        strokeStyle: data.datasets[0].backgroundColor[i],
                                        pointStyle: 'circle'
                                    }));
                                }
                            }
                        }
                    }
                }
            });
            return;
        }
    }
    
    // Fallback to ffiVisualization container with SVG
    const container = document.getElementById('ffiVisualization');
    if (!container) return;
    
    const data = window.analysisData || {};
    const ffiData = data.unsafe_ffi || {};
    const operations = ffiData.enhanced_ffi_data || ffiData.unsafe_operations || [];
    
    if (operations.length === 0) {
        container.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--text-secondary);">No FFI data available</div>';
        return;
    }
    
    // Create enhanced FFI dashboard with SVG (from original dashboard.html)
    const width = 320;
    const height = 240;
    
    const highRisk = operations.filter(op => (op.safety_violations || []).length > 2).length;
    const mediumRisk = operations.filter(op => (op.safety_violations || []).length > 0 && (op.safety_violations || []).length <= 2).length;
    const lowRisk = operations.filter(op => (op.safety_violations || []).length === 0).length;
    
    const total = highRisk + mediumRisk + lowRisk;
    if (total === 0) return;
    
    let html = `
        <div style="display: flex; flex-direction: column; align-items: center; padding: 16px;">
            <svg width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" style="background: var(--bg-secondary); border-radius: 8px;">
                <defs>
                    <filter id="ffi-glow">
                        <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
                        <feMerge>
                            <feMergeNode in="coloredBlur"/>
                            <feMergeNode in="SourceGraphic"/>
                        </feMerge>
                    </filter>
                    <radialGradient id="centerGradient" cx="50%" cy="50%" r="50%">
                        <stop offset="0%" style="stop-color:var(--bg-primary);stop-opacity:1" />
                        <stop offset="100%" style="stop-color:var(--bg-secondary);stop-opacity:1" />
                    </radialGradient>
                </defs>
    `;
    
    // Enhanced risk level visualization with better graphics
    const centerX = width / 2;
    const centerY = height / 2;
    const outerRadius = 80;
    const innerRadius = 40;
    
    let startAngle = -Math.PI / 2; // Start from top
    
    // Draw risk segments
    if (lowRisk > 0) {
        const angle = (lowRisk / total) * 2 * Math.PI;
        const endAngle = startAngle + angle;
        
        const x1 = centerX + innerRadius * Math.cos(startAngle);
        const y1 = centerY + innerRadius * Math.sin(startAngle);
        const x2 = centerX + outerRadius * Math.cos(startAngle);
        const y2 = centerY + outerRadius * Math.sin(startAngle);
        const x3 = centerX + outerRadius * Math.cos(endAngle);
        const y3 = centerY + outerRadius * Math.sin(endAngle);
        const x4 = centerX + innerRadius * Math.cos(endAngle);
        const y4 = centerY + innerRadius * Math.sin(endAngle);
        
        const largeArc = angle > Math.PI ? 1 : 0;
        
        html += `
            <path d="M ${x1} ${y1} L ${x2} ${y2} A ${outerRadius} ${outerRadius} 0 ${largeArc} 1 ${x3} ${y3} L ${x4} ${y4} A ${innerRadius} ${innerRadius} 0 ${largeArc} 0 ${x1} ${y1} Z"
                  fill="#059669" opacity="0.9" filter="url(#ffi-glow)" stroke="white" stroke-width="1">
                <title>Low Risk: ${lowRisk} operations (${(lowRisk/total*100).toFixed(1)}%)</title>
            </path>
        `;
        startAngle = endAngle;
    }
    
    if (mediumRisk > 0) {
        const angle = (mediumRisk / total) * 2 * Math.PI;
        const endAngle = startAngle + angle;
        
        const x1 = centerX + innerRadius * Math.cos(startAngle);
        const y1 = centerY + innerRadius * Math.sin(startAngle);
        const x2 = centerX + outerRadius * Math.cos(startAngle);
        const y2 = centerY + outerRadius * Math.sin(startAngle);
        const x3 = centerX + outerRadius * Math.cos(endAngle);
        const y3 = centerY + outerRadius * Math.sin(endAngle);
        const x4 = centerX + innerRadius * Math.cos(endAngle);
        const y4 = centerY + innerRadius * Math.sin(endAngle);
        
        const largeArc = angle > Math.PI ? 1 : 0;
        
        html += `
            <path d="M ${x1} ${y1} L ${x2} ${y2} A ${outerRadius} ${outerRadius} 0 ${largeArc} 1 ${x3} ${y3} L ${x4} ${y4} A ${innerRadius} ${innerRadius} 0 ${largeArc} 0 ${x1} ${y1} Z"
                  fill="#ea580c" opacity="0.9" filter="url(#ffi-glow)" stroke="white" stroke-width="1">
                <title>Medium Risk: ${mediumRisk} operations (${(mediumRisk/total*100).toFixed(1)}%)</title>
            </path>
        `;
        startAngle = endAngle;
    }
    
    if (highRisk > 0) {
        const angle = (highRisk / total) * 2 * Math.PI;
        const endAngle = startAngle + angle;
        
        const x1 = centerX + innerRadius * Math.cos(startAngle);
        const y1 = centerY + innerRadius * Math.sin(startAngle);
        const x2 = centerX + outerRadius * Math.cos(startAngle);
        const y2 = centerY + outerRadius * Math.sin(startAngle);
        const x3 = centerX + outerRadius * Math.cos(endAngle);
        const y3 = centerY + outerRadius * Math.sin(endAngle);
        const x4 = centerX + innerRadius * Math.cos(endAngle);
        const y4 = centerY + innerRadius * Math.sin(endAngle);
        
        const largeArc = angle > Math.PI ? 1 : 0;
        
        html += `
            <path d="M ${x1} ${y1} L ${x2} ${y2} A ${outerRadius} ${outerRadius} 0 ${largeArc} 1 ${x3} ${y3} L ${x4} ${y4} A ${innerRadius} ${innerRadius} 0 ${largeArc} 0 ${x1} ${y1} Z"
                  fill="#dc2626" opacity="0.9" filter="url(#ffi-glow)" stroke="white" stroke-width="1">
                <title>High Risk: ${highRisk} operations (${(highRisk/total*100).toFixed(1)}%)</title>
            </path>
        `;
    }
    
    // Center circle with gradient
    html += `
        <circle cx="${centerX}" cy="${centerY}" r="${innerRadius}" fill="url(#centerGradient)" stroke="var(--border-light)" stroke-width="2"/>
        <text x="${centerX}" y="${centerY - 8}" text-anchor="middle" font-size="18" font-weight="bold" fill="var(--text-primary)">
            ${total}
        </text>
        <text x="${centerX}" y="${centerY + 8}" text-anchor="middle" font-size="12" fill="var(--text-secondary)">
            FFI Operations
        </text>
    `;
    
    html += '</svg>';
    
    // Enhanced legend with statistics
    html += `
        <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; margin-top: 16px; width: 100%;">
            <div style="text-align: center; padding: 8px; background: var(--bg-primary); border-radius: 6px; border: 1px solid var(--border-light);">
                <div style="width: 16px; height: 16px; background: #059669; border-radius: 50%; margin: 0 auto 4px;"></div>
                <div style="font-size: 0.8rem; font-weight: 600;">${lowRisk}</div>
                <div style="font-size: 0.7rem; color: var(--text-secondary);">Low Risk</div>
            </div>
            <div style="text-align: center; padding: 8px; background: var(--bg-primary); border-radius: 6px; border: 1px solid var(--border-light);">
                <div style="width: 16px; height: 16px; background: #ea580c; border-radius: 50%; margin: 0 auto 4px;"></div>
                <div style="font-size: 0.8rem; font-weight: 600;">${mediumRisk}</div>
                <div style="font-size: 0.7rem; color: var(--text-secondary);">Medium Risk</div>
            </div>
            <div style="text-align: center; padding: 8px; background: var(--bg-primary); border-radius: 6px; border: 1px solid var(--border-light);">
                <div style="width: 16px; height: 16px; background: #dc2626; border-radius: 50%; margin: 0 auto 4px;"></div>
                <div style="font-size: 0.8rem; font-weight: 600;">${highRisk}</div>
                <div style="font-size: 0.7rem; color: var(--text-secondary);">High Risk</div>
            </div>
        </div>
    `;
    
    html += '</div>';
    container.innerHTML = html;
}

function renderMemoryUsageAnalysis() {
    // Will be implemented if container exists
}

function renderMemoryFragmentation() {
    const container = document.getElementById('memoryFragmentation');
    if (!container) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const totalMemory = allocs.reduce((sum, a) => sum + (a.size || 0), 0);
    const activeMemory = allocs.filter(a => !a.timestamp_dealloc).reduce((sum, a) => sum + (a.size || 0), 0);
    const fragmentationRate = totalMemory > 0 ? ((totalMemory - activeMemory) / totalMemory * 100) : 0;
    
    // Create chart container
    const chartDiv = document.createElement('div');
    chartDiv.style.height = '200px';
    chartDiv.style.position = 'relative';
    
    const canvas = document.createElement('canvas');
    chartDiv.appendChild(canvas);
    
    container.innerHTML = `
        <div style="padding: 16px;">
            <div style="display: flex; justify-content: space-between; margin-bottom: 16px;">
                <div>
                    <div style="color: var(--text-secondary); font-size: 0.9rem;">Fragmentation Rate</div>
                    <div style="font-size: 1.8rem; font-weight: 700; color: ${fragmentationRate > 30 ? '#dc2626' : fragmentationRate > 15 ? '#ea580c' : '#059669'};">
                        ${fragmentationRate.toFixed(1)}%
                    </div>
                </div>
                <div>
                    <div style="color: var(--text-secondary); font-size: 0.9rem;">Active Memory</div>
                    <div style="font-size: 1.2rem; font-weight: 600;">${formatBytes(activeMemory)}</div>
                </div>
            </div>
        </div>
    `;
    
    container.appendChild(chartDiv);
    
    // Create fragmentation visualization chart
    if (window.Chart && allocs.length > 0) {
        new Chart(canvas, {
            type: 'doughnut',
            data: {
                labels: ['Active Memory', 'Fragmented/Freed'],
                datasets: [{
                    data: [activeMemory, totalMemory - activeMemory],
                    backgroundColor: ['#059669', '#dc2626'],
                    borderWidth: 2,
                    borderColor: 'var(--bg-primary)'
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        position: 'bottom',
                        labels: {
                            padding: 15,
                            usePointStyle: true,
                            generateLabels: function(chart) {
                                const data = chart.data;
                                return data.labels.map((label, i) => ({
                                    text: `${label}: ${formatBytes(data.datasets[0].data[i])}`,
                                    fillStyle: data.datasets[0].backgroundColor[i],
                                    strokeStyle: data.datasets[0].backgroundColor[i],
                                    pointStyle: 'circle'
                                }));
                            }
                        }
                    }
                }
            }
        });
    }
}

function renderMemoryGrowthTrends() {
    const container = document.getElementById('growth');
    if (!container) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    if (allocs.length === 0) {
        container.innerHTML = '<div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">No growth data available</div>';
        return;
    }
    
    // Create a proper growth chart using Chart.js
    const canvas = document.createElement('canvas');
    canvas.style.width = '100%';
    canvas.style.height = '100%';
    container.innerHTML = '';
    container.appendChild(canvas);
    
    const sorted = allocs.slice().sort((a,b) => (a.timestamp_alloc||0) - (b.timestamp_alloc||0));
    let cumulative = 0;
    const points = [];
    
    for (let i = 0; i < Math.min(sorted.length, 30); i++) {
        cumulative += sorted[i].size || 0;
        points.push({ x: i, y: cumulative });
    }
    
    if (points.length > 1 && window.Chart) {
        new Chart(canvas, {
            type: 'line',
            data: {
                labels: points.map((_, i) => `T${i}`),
                datasets: [{
                    label: 'Memory Growth',
                    data: points.map(p => p.y),
                    borderColor: '#059669',
                    backgroundColor: 'rgba(5, 150, 105, 0.1)',
                    fill: true,
                    tension: 0.3,
                    pointRadius: 3,
                    pointHoverRadius: 5
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { display: false }
                },
                scales: {
                    x: {
                        title: {
                            display: true,
                            text: 'Time Steps'
                        }
                    },
                    y: { 
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Cumulative Memory'
                        },
                        ticks: {
                            callback: function(value) {
                                return formatBytes(value);
                            }
                        }
                    }
                }
            }
        });
    }
}

function setupLifecycle() {
    // Lifecycle setup functionality
}

function populateUnsafeTable() {
    const data = window.analysisData || {};
    const root = data.unsafe_ffi || {};
    const ops = root.enhanced_ffi_data || root.unsafe_operations || root.allocations || [];
    const unsafeTable = document.getElementById('unsafeTable');
    if (!unsafeTable) return;
    
    unsafeTable.innerHTML = (ops || []).slice(0, 50).map(op => {
        const riskLevel = op.risk_level || ((op.safety_violations||[]).length > 2 ? 'High' : 
                         ((op.safety_violations||[]).length > 0 ? 'Medium' : 'Low'));
        
        const riskText = riskLevel === 'High' ? 'High Risk' : (riskLevel === 'Medium' ? 'Medium Risk' : 'Low Risk');
        const riskClass = riskLevel === 'High' ? 'risk-high' : (riskLevel === 'Medium' ? 'risk-medium' : 'risk-low');
        
        return `<tr>
            <td>${op.location || op.var_name || 'Unknown'}</td>
            <td>${op.operation_type || op.type_name || 'Unknown'}</td>
            <td><span class="status-badge ${riskClass}">${riskText}</span></td>
        </tr>`;
    }).join('');
}

function renderVariableGraph() {
    const container = document.getElementById('graph');
    if (!container) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    if (allocs.length === 0) {
        container.innerHTML = '<div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">No relationship data available</div>';
        return;
    }
    
    // Enhanced variable relationship graph with drag and click functionality
    const nodes = allocs.slice(0, 30).map((a, i) => ({
        id: i,
        name: a.var_name || `var_${i}`,
        type: a.type_name || 'unknown',
        size: a.size || 0,
        status: a.is_leaked ? 'leaked' : (a.timestamp_dealloc ? 'freed' : 'active'),
        ptr: a.ptr || 'unknown',
        timestamp_alloc: a.timestamp_alloc || 0,
        timestamp_dealloc: a.timestamp_dealloc || null,
        x: 100 + (i % 6) * 80 + Math.random() * 20,
        y: 100 + Math.floor(i / 6) * 80 + Math.random() * 20,
        isDragging: false
    }));
    
    // Create links between related variables
    const links = [];
    for (let i = 0; i < nodes.length; i++) {
        for (let j = i + 1; j < nodes.length; j++) {
            if (nodes[i].type === nodes[j].type || 
                nodes[i].name.startsWith(nodes[j].name.substring(0, 3))) {
                links.push({ source: i, target: j });
            }
        }
    }
    
    const width = container.offsetWidth || 500;
    const height = 400;
    
    // Create SVG with interactive elements
    let html = `
        <div style="position: relative; width: 100%; height: ${height}px;">
            <svg id="graph-svg" width="100%" height="${height}" viewBox="0 0 ${width} ${height}" style="background: transparent; cursor: grab;">
                <defs>
                    <filter id="node-glow">
                        <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
                        <feMerge>
                            <feMergeNode in="coloredBlur"/>
                            <feMergeNode in="SourceGraphic"/>
                        </feMerge>
                    </filter>
                    <marker id="arrow" viewBox="0 0 10 10" refX="8" refY="3"
                            markerWidth="6" markerHeight="6" orient="auto">
                        <path d="M0,0 L0,6 L9,3 z" fill="var(--border-light)" opacity="0.6"/>
                    </marker>
                </defs>
                <g id="links-group">
    `;
    
    // Draw links
    links.forEach((link, linkIndex) => {
        const source = nodes[link.source];
        const target = nodes[link.target];
        html += `
            <line id="link-${linkIndex}" x1="${source.x}" y1="${source.y}" x2="${target.x}" y2="${target.y}" 
                  stroke="var(--border-light)" stroke-width="1" opacity="0.4" 
                  marker-end="url(#arrow)"/>
        `;
    });
    
    html += '</g><g id="nodes-group">';
    
    // Draw nodes
    nodes.forEach((node, nodeIndex) => {
        const radius = Math.max(8, Math.min(25, Math.log(node.size + 1) * 3));
        let color = '#6b7280'; // default
        
        if (node.type.includes('String')) color = '#fbbf24';
        else if (node.type.includes('Vec')) color = '#3b82f6';
        else if (node.type.includes('Box') || node.type.includes('Rc')) color = '#8b5cf6';
        else if (node.type.includes('HashMap')) color = '#10b981';
        else if (node.type.includes('Arc')) color = '#f59e0b';
        
        if (node.status === 'leaked') color = '#dc2626';
        else if (node.status === 'freed') color = '#9ca3af';
        
        html += `
            <circle 
                id="node-${nodeIndex}"
                cx="${node.x}" 
                cy="${node.y}" 
                r="${radius}" 
                fill="${color}" 
                stroke="white" 
                stroke-width="2" 
                filter="url(#node-glow)"
                style="cursor: grab;"
                class="graph-node"
                data-index="${nodeIndex}"
                data-name="${node.name}"
                data-type="${node.type}"
                data-size="${node.size}"
                data-status="${node.status}"
                data-ptr="${node.ptr}"
                data-alloc="${node.timestamp_alloc}"
                data-dealloc="${node.timestamp_dealloc || 'null'}"
            />
            <text 
                id="text-${nodeIndex}"
                x="${node.x}" 
                y="${node.y + radius + 15}" 
                text-anchor="middle" 
                font-size="9" 
                fill="var(--text-primary)"
                style="font-weight: 500; pointer-events: none;"
            >${node.name.length > 10 ? node.name.substring(0, 8) + '...' : node.name}</text>
        `;
    });
    
    html += `
            </g>
        </svg>
        
        <!-- Node detail panel -->
        <div id="node-detail-panel" style="
            position: absolute;
            background: var(--bg-primary);
            border: 1px solid var(--border-light);
            border-radius: 8px;
            padding: 12px;
            width: 280px;
            box-shadow: 0 10px 25px rgba(0,0,0,0.1);
            z-index: 1000;
            font-size: 0.875rem;
            display: none;
            backdrop-filter: blur(10px);
        ">
            <div style="display: flex; justify-content: between; align-items: center; margin-bottom: 8px;">
                <h4 id="detail-title" style="margin: 0; font-size: 1rem; font-weight: 600;"></h4>
                <button onclick="hideNodeDetails()" style="background: none; border: none; font-size: 16px; cursor: pointer; color: var(--text-secondary);">×</button>
            </div>
            <div id="detail-content"></div>
        </div>
        
        <!-- Legend -->
        <div style="display: flex; gap: 12px; margin-top: 12px; font-size: 0.75rem; flex-wrap: wrap;">
            <div style="display: flex; align-items: center; gap: 4px;">
                <div style="width: 10px; height: 10px; background: #fbbf24; border-radius: 50%;"></div>
                <span>String</span>
            </div>
            <div style="display: flex; align-items: center; gap: 4px;">
                <div style="width: 10px; height: 10px; background: #3b82f6; border-radius: 50%;"></div>
                <span>Vec</span>
            </div>
            <div style="display: flex; align-items: center; gap: 4px;">
                <div style="width: 10px; height: 10px; background: #8b5cf6; border-radius: 50%;"></div>
                <span>Smart Ptr</span>
            </div>
            <div style="display: flex; align-items: center; gap: 4px;">
                <div style="width: 10px; height: 10px; background: #dc2626; border-radius: 50%;"></div>
                <span>Leaked</span>
            </div>
            <div style="display: flex; align-items: center; gap: 4px;">
                <div style="width: 10px; height: 10px; background: #9ca3af; border-radius: 50%;"></div>
                <span>Freed</span>
            </div>
        </div>
    </div>
    `;
    
    container.innerHTML = html;
    
    // Store nodes and links data for interaction
    window.graphNodes = nodes;
    window.graphLinks = links;
    
    // Add drag and click functionality
    setTimeout(() => {
        setupGraphInteractions();
    }, 100);
}

// Graph interaction functions
function setupGraphInteractions() {
    const svg = document.getElementById('graph-svg');
    const nodeElements = document.querySelectorAll('.graph-node');
    
    let draggedNode = null;
    let isDragging = false;
    let startX, startY;
    
    nodeElements.forEach(node => {
        // Mouse events for drag
        node.addEventListener('mousedown', function(e) {
            e.preventDefault();
            draggedNode = this;
            isDragging = false;
            startX = e.clientX;
            startY = e.clientY;
            this.style.cursor = 'grabbing';
            svg.style.cursor = 'grabbing';
        });
        
        // Click event for details
        node.addEventListener('click', function(e) {
            if (!isDragging) {
                showNodeDetails(this);
            }
        });
        
        // Hover effects
        node.addEventListener('mouseover', function() {
            if (!draggedNode) {
                this.r.baseVal.value = this.r.baseVal.value * 1.2;
            }
        });
        
        node.addEventListener('mouseout', function() {
            if (!draggedNode) {
                this.r.baseVal.value = this.r.baseVal.value / 1.2;
            }
        });
    });
    
    // Global mouse events for dragging
    document.addEventListener('mousemove', function(e) {
        if (draggedNode) {
            const deltaX = e.clientX - startX;
            const deltaY = e.clientY - startY;
            
            if (Math.abs(deltaX) > 3 || Math.abs(deltaY) > 3) {
                isDragging = true;
            }
            
            if (isDragging) {
                const rect = svg.getBoundingClientRect();
                const svgX = e.clientX - rect.left;
                const svgY = e.clientY - rect.top;
                
                // Update node position
                draggedNode.cx.baseVal.value = svgX;
                draggedNode.cy.baseVal.value = svgY;
                
                // Update text position
                const nodeIndex = draggedNode.getAttribute('data-index');
                const textElement = document.getElementById(`text-${nodeIndex}`);
                if (textElement) {
                    textElement.x.baseVal[0].value = svgX;
                    textElement.y.baseVal[0].value = svgY + parseInt(draggedNode.r.baseVal.value) + 15;
                }
                
                // Update connected links
                updateConnectedLinks(parseInt(nodeIndex), svgX, svgY);
                
                // Update stored node position
                if (window.graphNodes && window.graphNodes[nodeIndex]) {
                    window.graphNodes[nodeIndex].x = svgX;
                    window.graphNodes[nodeIndex].y = svgY;
                }
            }
        }
    });
    
    document.addEventListener('mouseup', function() {
        if (draggedNode) {
            draggedNode.style.cursor = 'grab';
            svg.style.cursor = 'grab';
            draggedNode = null;
            setTimeout(() => { isDragging = false; }, 100);
        }
    });
}

function updateConnectedLinks(nodeIndex, newX, newY) {
    if (!window.graphLinks) return;
    
    window.graphLinks.forEach((link, linkIndex) => {
        const linkElement = document.getElementById(`link-${linkIndex}`);
        if (!linkElement) return;
        
        if (link.source === nodeIndex) {
            linkElement.x1.baseVal.value = newX;
            linkElement.y1.baseVal.value = newY;
        }
        if (link.target === nodeIndex) {
            linkElement.x2.baseVal.value = newX;
            linkElement.y2.baseVal.value = newY;
        }
    });
}

function showNodeDetails(nodeElement) {
    const panel = document.getElementById('node-detail-panel');
    const title = document.getElementById('detail-title');
    const content = document.getElementById('detail-content');
    
    if (!panel || !title || !content) return;
    
    const name = nodeElement.getAttribute('data-name');
    const type = nodeElement.getAttribute('data-type');
    const size = parseInt(nodeElement.getAttribute('data-size'));
    const status = nodeElement.getAttribute('data-status');
    const ptr = nodeElement.getAttribute('data-ptr');
    const alloc = nodeElement.getAttribute('data-alloc');
    const dealloc = nodeElement.getAttribute('data-dealloc');
    
    title.textContent = name;
    
    const lifetime = dealloc !== 'null' ? parseInt(dealloc) - parseInt(alloc) : 'Active';
    
    content.innerHTML = `
        <div style="margin-bottom: 8px;">
            <strong>Type:</strong> ${formatTypeName(type)}
        </div>
        <div style="margin-bottom: 8px;">
            <strong>Size:</strong> ${formatBytes(size)}
        </div>
        <div style="margin-bottom: 8px;">
            <strong>Status:</strong> <span style="color: ${status === 'leaked' ? '#dc2626' : status === 'freed' ? '#6b7280' : '#059669'};">${status.charAt(0).toUpperCase() + status.slice(1)}</span>
        </div>
        <div style="margin-bottom: 8px;">
            <strong>Pointer:</strong> <code style="font-size: 0.8rem; background: var(--bg-secondary); padding: 2px 4px; border-radius: 3px;">${ptr}</code>
        </div>
        <div style="margin-bottom: 8px;">
            <strong>Allocated:</strong> ${alloc}ms
        </div>
        <div style="margin-bottom: 8px;">
            <strong>Lifetime:</strong> ${typeof lifetime === 'number' ? lifetime + 'ms' : lifetime}
        </div>
    `;
    
    // Position panel near the node
    const rect = nodeElement.getBoundingClientRect();
    const containerRect = nodeElement.closest('#graph').getBoundingClientRect();
    
    panel.style.left = Math.min(rect.left - containerRect.left + 30, containerRect.width - 300) + 'px';
    panel.style.top = Math.max(rect.top - containerRect.top - 50, 10) + 'px';
    panel.style.display = 'block';
}

function hideNodeDetails() {
    const panel = document.getElementById('node-detail-panel');
    if (panel) {
        panel.style.display = 'none';
    }
}

// Lifecycle toggle functionality
function setupLifecycleToggle() {
    const toggleBtn = document.getElementById('toggle-lifecycle');
    if (!toggleBtn) return;
    
    let isExpanded = false;
    
    toggleBtn.addEventListener('click', function() {
        const container = document.getElementById('lifetimeVisualization');
        if (!container) return;
        
        const data = window.analysisData || {};
        const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
        
        if (allocs.length === 0) {
            container.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--text-secondary);">No lifecycle data available</div>';
            return;
        }
        
        const icon = toggleBtn.querySelector('i');
        const text = toggleBtn.querySelector('span');
        
        if (!isExpanded) {
            // Show all allocations
            renderFullLifecycleTimeline(allocs);
            icon.className = 'fa fa-chevron-up';
            text.textContent = 'Show Less';
            isExpanded = true;
        } else {
            // Show only top 20
            renderLimitedLifecycleTimeline(allocs);
            icon.className = 'fa fa-chevron-down';
            text.textContent = 'Show All';
            isExpanded = false;
        }
    });
}

function renderLimitedLifecycleTimeline(allocs) {
    const container = document.getElementById('lifetimeVisualization');
    if (!container) return;
    
    // Create timeline visualization (limited to 20)
    const maxTime = Math.max(...allocs.map(a => a.timestamp_dealloc || a.timestamp_alloc || 0));
    const minTime = Math.min(...allocs.map(a => a.timestamp_alloc || 0));
    const timeRange = maxTime - minTime || 1;
    
    let html = '<div style="padding: 16px; max-height: 300px; overflow-y: auto;">';
    
    allocs.slice(0, 20).forEach((alloc, index) => {
        const startTime = alloc.timestamp_alloc || 0;
        const endTime = alloc.timestamp_dealloc || maxTime;
        const startPercent = ((startTime - minTime) / timeRange) * 100;
        const widthPercent = ((endTime - startTime) / timeRange) * 100;
        const isActive = !alloc.timestamp_dealloc;
        
        html += `
            <div style="margin-bottom: 8px;">
                <div style="display: flex; justify-content: space-between; margin-bottom: 4px; font-size: 0.8rem;">
                    <span style="font-weight: 600;">${alloc.var_name || `var_${index}`}</span>
                    <span style="color: var(--text-secondary);">${formatBytes(alloc.size || 0)}</span>
                </div>
                <div style="position: relative; background: var(--bg-secondary); height: 8px; border-radius: 4px;">
                    <div style="
                        position: absolute;
                        left: ${startPercent}%;
                        width: ${widthPercent}%;
                        height: 100%;
                        background: ${isActive ? 'linear-gradient(to right, #059669, #34d399)' : 'linear-gradient(to right, #2563eb, #60a5fa)'};
                        border-radius: 4px;
                        ${isActive ? 'animation: pulse 2s infinite;' : ''}
                    " title="Lifetime: ${endTime - startTime}ms"></div>
                </div>
            </div>
        `;
    });
    
    html += '</div>';
    
    // Add CSS for pulse animation
    html += `
        <style>
            @keyframes pulse {
                0%, 100% { opacity: 1; }
                50% { opacity: 0.7; }
            }
        </style>
    `;
    
    container.innerHTML = html;
}

function renderFullLifecycleTimeline(allocs) {
    const container = document.getElementById('lifetimeVisualization');
    if (!container) return;
    
    // Create full timeline visualization
    const maxTime = Math.max(...allocs.map(a => a.timestamp_dealloc || a.timestamp_alloc || 0));
    const minTime = Math.min(...allocs.map(a => a.timestamp_alloc || 0));
    const timeRange = maxTime - minTime || 1;
    
    let html = '<div style="padding: 16px; max-height: 600px; overflow-y: auto;">';
    
    // Add timeline header
    html += `
        <div style="margin-bottom: 16px; padding: 12px; background: var(--bg-secondary); border-radius: 8px;">
            <div style="font-weight: 600; margin-bottom: 8px;">Full Lifecycle Timeline</div>
            <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; font-size: 0.8rem;">
                <div>
                    <div style="color: var(--text-secondary);">Total Variables</div>
                    <div style="font-weight: 600;">${allocs.length}</div>
                </div>
                <div>
                    <div style="color: var(--text-secondary);">Active</div>
                    <div style="font-weight: 600; color: #059669;">${allocs.filter(a => !a.timestamp_dealloc).length}</div>
                </div>
                <div>
                    <div style="color: var(--text-secondary);">Freed</div>
                    <div style="font-weight: 600; color: #2563eb;">${allocs.filter(a => a.timestamp_dealloc && !a.is_leaked).length}</div>
                </div>
                <div>
                    <div style="color: var(--text-secondary);">Leaked</div>
                    <div style="font-weight: 600; color: #dc2626;">${allocs.filter(a => a.is_leaked).length}</div>
                </div>
            </div>
        </div>
    `;
    
    allocs.forEach((alloc, index) => {
        const startTime = alloc.timestamp_alloc || 0;
        const endTime = alloc.timestamp_dealloc || maxTime;
        const startPercent = ((startTime - minTime) / timeRange) * 100;
        const widthPercent = ((endTime - startTime) / timeRange) * 100;
        const isActive = !alloc.timestamp_dealloc;
        const isLeaked = alloc.is_leaked;
        
        let barColor = 'linear-gradient(to right, #2563eb, #60a5fa)'; // freed
        if (isActive) barColor = 'linear-gradient(to right, #059669, #34d399)'; // active
        if (isLeaked) barColor = 'linear-gradient(to right, #dc2626, #f87171)'; // leaked
        
        html += `
            <div style="margin-bottom: 6px;">
                <div style="display: flex; justify-content: space-between; margin-bottom: 3px; font-size: 0.75rem;">
                    <span style="font-weight: 600;">${alloc.var_name || `var_${index}`}</span>
                    <div style="display: flex; gap: 8px;">
                        <span style="color: var(--text-secondary);">${formatTypeName(alloc.type_name || 'Unknown')}</span>
                        <span style="color: var(--text-secondary);">${formatBytes(alloc.size || 0)}</span>
                    </div>
                </div>
                <div style="position: relative; background: var(--bg-secondary); height: 6px; border-radius: 3px;">
                    <div style="
                        position: absolute;
                        left: ${startPercent}%;
                        width: ${widthPercent}%;
                        height: 100%;
                        background: ${barColor};
                        border-radius: 3px;
                        ${isActive ? 'animation: pulse 2s infinite;' : ''}
                    " title="Lifetime: ${endTime - startTime}ms | Status: ${isLeaked ? 'Leaked' : isActive ? 'Active' : 'Freed'}"></div>
                </div>
            </div>
        `;
    });
    
    html += '</div>';
    
    // Add CSS for pulse animation
    html += `
        <style>
            @keyframes pulse {
                0%, 100% { opacity: 1; }
                50% { opacity: 0.7; }
            }
        </style>
    `;
    
    container.innerHTML = html;
}

function setupLifecycleVisualization() {
    const container = document.getElementById('lifetimeVisualization');
    if (!container) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    
    if (allocs.length === 0) {
        container.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--text-secondary);">No lifecycle data available</div>';
        return;
    }
    
    // Create timeline visualization
    const maxTime = Math.max(...allocs.map(a => a.timestamp_dealloc || a.timestamp_alloc || 0));
    const minTime = Math.min(...allocs.map(a => a.timestamp_alloc || 0));
    const timeRange = maxTime - minTime || 1;
    
    let html = '<div style="padding: 16px; max-height: 300px; overflow-y: auto;">';
    
    allocs.slice(0, 20).forEach((alloc, index) => {
        const startTime = alloc.timestamp_alloc || 0;
        const endTime = alloc.timestamp_dealloc || maxTime;
        const startPercent = ((startTime - minTime) / timeRange) * 100;
        const widthPercent = ((endTime - startTime) / timeRange) * 100;
        const isActive = !alloc.timestamp_dealloc;
        
        html += `
            <div style="margin-bottom: 8px;">
                <div style="display: flex; justify-content: space-between; margin-bottom: 4px; font-size: 0.8rem;">
                    <span style="font-weight: 600;">${alloc.var_name || `var_${index}`}</span>
                    <span style="color: var(--text-secondary);">${formatBytes(alloc.size || 0)}</span>
                </div>
                <div style="position: relative; background: var(--bg-secondary); height: 8px; border-radius: 4px;">
                    <div style="
                        position: absolute;
                        left: ${startPercent}%;
                        width: ${widthPercent}%;
                        height: 100%;
                        background: ${isActive ? 'linear-gradient(to right, #059669, #34d399)' : 'linear-gradient(to right, #2563eb, #60a5fa)'};
                        border-radius: 4px;
                        ${isActive ? 'animation: pulse 2s infinite;' : ''}
                    " title="Lifetime: ${endTime - startTime}ms"></div>
                </div>
            </div>
        `;
    });
    
    html += '</div>';
    
    // Add CSS for pulse animation
    html += `
        <style>
            @keyframes pulse {
                0%, 100% { opacity: 1; }
                50% { opacity: 0.7; }
            }
        </style>
    `;
    
    container.innerHTML = html;
}

function initFFIVisualization() {
    // Additional FFI initialization if needed
    renderFFI();
}

// Initialize dashboard when DOM is loaded
document.addEventListener("DOMContentLoaded", () => {
    console.log('🚀 MemScope Dashboard Loaded');
    
    // Initialize theme toggle
    try { 
        initThemeToggle(); 
        console.log('✅ Theme toggle initialized');
    } catch(e) { 
        console.warn('⚠️ Theme toggle initialization failed:', e?.message); 
    }
    
    // Initialize main dashboard with all original functions
    try { 
        // Use original dashboard functions
        renderKpis();
        renderTypeChart();
        renderTimelineChart();
        renderTreemap();
        renderLifetimes();
        renderFFI();
        renderMemoryFragmentation();
        renderMemoryGrowthTrends();
        populateAllocationsTable();
        populateUnsafeTable();
        renderVariableGraph();
        initFFIVisualization();
        setupLifecycleVisualization();
        setupLifecycleToggle();
        
        console.log('✅ All dashboard components initialized');
    } catch(e) { 
        console.error('❌ Dashboard initialization failed:', e); 
    }
});