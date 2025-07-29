// MemScope Dashboard JavaScript - Simplified version
// This file contains only the essential functions for the existing dashboard

// Global data store - will be populated by HTML template
window.analysisData = window.analysisData || {};

// Initialize lifetime visualization from JSON data
function initLifetimeVisualization() {
    console.log('🔄 Initializing lifetime visualization...');

    // Get lifetime data from the global data store
    const lifetimeData = window.analysisData.lifetime;
    if (!lifetimeData || !lifetimeData.lifecycle_events) {
        console.warn('⚠️ No lifetime data found');
        console.log('Available data keys:', Object.keys(window.analysisData || {}));
        showEmptyLifetimeState();
        return;
    }

    console.log(`📊 Total lifecycle events: ${lifetimeData.lifecycle_events.length}`);

    // Check if we have Rust-preprocessed data
    if (lifetimeData.visualization_ready && lifetimeData.variable_groups) {
        console.log(`📊 Using Rust-preprocessed data with ${lifetimeData.variable_groups.length} variable groups`);
        renderLifetimeVisualizationFromRust(lifetimeData.variable_groups);
        return;
    }

    // Filter for user-defined variables (non-unknown var_name and type_name)
    const userVariables = lifetimeData.lifecycle_events.filter(event =>
        event.var_name && event.var_name !== 'unknown' &&
        event.type_name && event.type_name !== 'unknown'
    );

    console.log(`📊 Found ${userVariables.length} user-defined variables in lifetime data`);

    // Debug: Show some examples of what we found
    if (userVariables.length > 0) {
        console.log('📊 Sample user variables:', userVariables.slice(0, 3));
    } else {
        // Show some examples of unknown variables for debugging
        const unknownSamples = lifetimeData.lifecycle_events.slice(0, 3);
        console.log('📊 Sample unknown variables:', unknownSamples);
    }

    if (userVariables.length === 0) {
        showEmptyLifetimeState();
        return;
    }

    // Group by variable name to get allocation/deallocation pairs
    const variableGroups = groupVariablesByName(userVariables);

    // Render the lifetime visualization
    renderLifetimeVisualization(variableGroups);
}

// Group variables by name to track their lifecycle
function groupVariablesByName(events) {
    const groups = {};

    events.forEach(event => {
        const varName = event.var_name;
        if (!groups[varName]) {
            groups[varName] = {
                var_name: varName,
                type_name: event.type_name,
                events: []
            };
        }
        groups[varName].events.push(event);
    });

    return Object.values(groups);
}

// Render lifetime visualization from Rust-preprocessed data
function renderLifetimeVisualizationFromRust(variableGroups) {
    console.log(`📊 Rendering ${variableGroups.length} Rust-preprocessed variable groups`);

    const container = document.getElementById('lifetimeVisualization');
    if (!container) return;

    // Clear loading state
    container.innerHTML = '';

    if (!variableGroups || variableGroups.length === 0) {
        showEmptyLifetimeState();
        return;
    }

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

    // Render each variable with colorful progress bars
    variableGroups.forEach((group, index) => {
        const varDiv = document.createElement('div');
        varDiv.className = 'flex items-center py-4 border-b border-gray-100 hover:bg-gray-50 transition-colors';

        // Get color from palette (cycle through colors)
        const colorIndex = index % COLOR_PALETTE.progress.length;
        const progressColor = COLOR_PALETTE.progress[colorIndex];

        // Use preprocessed timing data or fallback to events
        const startTime = group.start_time || (group.events && group.events[0] ? group.events[0].timestamp : minTime);
        const endTime = group.end_time || (group.events && group.events[group.events.length - 1] ? group.events[group.events.length - 1].timestamp : maxTime);

        const startPercent = timeRange > 0 ? ((startTime - minTime) / timeRange) * 100 : 0;
        const duration = endTime - startTime;
        const widthPercent = timeRange > 0 ? Math.max(5, (duration / timeRange) * 100) : 40;

        // Format type name for display
        const displayTypeName = formatTypeName(group.type_name);

        // Create gradient background for more visual appeal
        const gradientStyle = `background: linear-gradient(90deg, ${progressColor}, ${progressColor}dd);`;

        varDiv.innerHTML = `
            <div class="w-48 flex-shrink-0 pr-4">
                <div class="text-sm font-semibold text-gray-800">${group.var_name}</div>
                <div class="text-xs text-gray-500">${displayTypeName}</div>
            </div>
            <div class="flex-grow relative bg-gray-200 rounded-full h-6 overflow-hidden">
                <div class="absolute inset-0 rounded-full" 
                     style="${gradientStyle} width: ${widthPercent}%; margin-left: ${startPercent}%; 
                            box-shadow: 0 2px 4px rgba(0,0,0,0.1); 
                            transition: all 0.3s ease;"
                     title="Variable: ${group.var_name}, Type: ${displayTypeName}">
                    <div class="absolute inset-0 flex items-center justify-center">
                        <span class="text-xs font-bold text-white drop-shadow-sm">
                            ${Math.round(widthPercent)}%
                        </span>
                    </div>
                </div>
                <div class="absolute -top-8 left-0 text-xs bg-gray-700 text-white px-2 py-1 rounded opacity-0 hover:opacity-100 transition-opacity whitespace-nowrap">
                    Duration: ${formatTimestamp(duration, 0)}
                </div>
            </div>
            <div class="w-20 flex-shrink-0 pl-4 text-right">
                <div class="text-xs text-gray-600">
                    ${formatBytes(group.size || (group.events && group.events[0] ? group.events[0].size : 0) || 0)}
                </div>
            </div>
        `;

        container.appendChild(varDiv);
    });

    console.log(`✅ Rendered ${variableGroups.length} Rust-preprocessed variables in lifetime visualization`);
}

// Render the lifetime visualization
function renderLifetimeVisualization(variableGroups) {
    const container = document.getElementById('lifetimeVisualization');
    if (!container) return;

    // Clear loading state
    container.innerHTML = '';

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

    // Render each variable
    variableGroups.forEach((group) => {
        const varDiv = document.createElement('div');
        varDiv.className = 'flex items-end py-3 border-b border-gray-100';

        // Determine color based on type
        const typeKey = Object.keys(typeColors).find(key =>
            group.type_name.includes(key)
        ) || 'default';
        const colors = typeColors[typeKey];

        // Calculate position and width based on timestamps
        const firstEvent = group.events[0];
        const startTime = firstEvent.timestamp;
        const startPercent = timeRange > 0 ? ((startTime - minTime) / timeRange) * 100 : 0;

        // For now, assume a fixed width since we don't have deallocation events
        // In a real implementation, you'd track deallocation events too
        const widthPercent = 60; // Default width

        // Format type name for display
        const displayTypeName = formatTypeName(group.type_name);

        varDiv.innerHTML = `
            <div class="w-40 flex-shrink-0 text-sm font-medium">
                ${group.var_name} (${displayTypeName})
            </div>
            <div class="flex-grow relative">
                <div class="lifespan-indicator ${colors.bg}" 
                     style="width: ${widthPercent}%; margin-left: ${startPercent}%;" 
                     title="Variable: ${group.var_name}, Type: ${displayTypeName}">
                    <div class="absolute -top-6 left-0 text-xs ${colors.bg} text-white px-2 py-1 rounded whitespace-nowrap">
                        Allocated: ${formatTimestamp(startTime, minTime)}
                    </div>
                </div>
            </div>
        `;

        container.appendChild(varDiv);
    });

    console.log(`✅ Rendered ${variableGroups.length} variables in lifetime visualization`);
}

// Format type name for better display
function formatTypeName(typeName) {
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
        <div class="text-center py-8 text-gray-500">
            <i class="fa fa-info-circle text-2xl mb-2"></i>
            <p>No user-defined variables found in lifetime data</p>
            <p class="text-sm mt-1">Use track_var! macro to track variable lifetimes</p>
        </div>
    `;
}

// Initialize all dashboard components
function initializeDashboard() {
    console.log('🚀 Initializing MemScope dashboard...');
    console.log('📊 Available data:', Object.keys(window.analysisData || {}));
    
    // Initialize all components
    initSummaryStats();
    initCharts();
    initMemoryUsageAnalysis();
    initLifetimeVisualization();
    initFFIVisualization();
    initMemoryFragmentation();
    initMemoryGrowthTrends();
    initAllocationsTable();
    initGenericTypesTable();
    initVariableGraph();
    initComplexTypeAnalysis();
    initMemoryOptimizationRecommendations();
    initFFIRiskChart();
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

// Initialize charts
function initCharts() {
    console.log('📊 Initializing charts...');
    
    // Initialize complexity chart
    initComplexityChart();
    
    // Initialize memory distribution chart
    initMemoryDistributionChart();
    
    // Initialize allocation size chart
    initAllocationSizeChart();
    
    // Initialize performance chart
    initPerformanceChart();
}

// Initialize complexity chart
function initComplexityChart() {
    const ctx = document.getElementById('complexity-chart');
    if (!ctx) return;
    
    const complexTypes = window.analysisData.complex_types?.summary?.complexity_distribution || {};
    
    new Chart(ctx, {
        type: 'doughnut',
        data: {
            labels: ['Low', 'Medium', 'High', 'Very High'],
            datasets: [{
                data: [
                    complexTypes.low_complexity || 0,
                    complexTypes.medium_complexity || 0,
                    complexTypes.high_complexity || 0,
                    complexTypes.very_high_complexity || 0
                ],
                backgroundColor: ['#10b981', '#f59e0b', '#ef4444', '#7c2d12']
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false
        }
    });
}

// Initialize memory distribution chart
function initMemoryDistributionChart() {
    const ctx = document.getElementById('memory-distribution-chart');
    if (!ctx) return;
    
    const allocations = window.analysisData.memory_analysis?.allocations || [];
    const typeDistribution = {};
    
    allocations.forEach(alloc => {
        const type = alloc.type_name || 'Unknown';
        typeDistribution[type] = (typeDistribution[type] || 0) + alloc.size;
    });
    
    const sortedTypes = Object.entries(typeDistribution)
        .sort(([,a], [,b]) => b - a)
        .slice(0, 10);
    
    new Chart(ctx, {
        type: 'bar',
        data: {
            labels: sortedTypes.map(([type]) => formatTypeName(type)),
            datasets: [{
                label: 'Memory Usage (bytes)',
                data: sortedTypes.map(([,size]) => size),
                backgroundColor: '#3b82f6'
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
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
    
    new Chart(ctx, {
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
            maintainAspectRatio: false
        }
    });
}

// Initialize performance chart
function initPerformanceChart() {
    const ctx = document.getElementById('performance-chart');
    if (!ctx) return;
    
    const performance = window.analysisData.performance?.memory_performance || {};
    
    new Chart(ctx, {
        type: 'bar',
        data: {
            labels: ['Active Memory', 'Peak Memory', 'Total Allocated'],
            datasets: [{
                label: 'Memory (bytes)',
                data: [
                    performance.active_memory || 0,
                    performance.peak_memory || 0,
                    performance.total_allocated || 0
                ],
                backgroundColor: ['#10b981', '#f59e0b', '#ef4444']
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
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

// Initialize memory usage analysis
function initMemoryUsageAnalysis() {
    const container = document.getElementById('memory-usage-analysis');
    if (!container) return;
    
    container.innerHTML = `
        <div class="text-center py-8">
            <h4 class="text-lg font-semibold mb-4">Memory Usage Overview</h4>
            <p class="text-gray-600">Memory usage analysis visualization will be implemented here</p>
        </div>
    `;
}

// Initialize FFI visualization
function initFFIVisualization() {
    console.log('🔄 Initializing FFI visualization...');
    
    const container = document.getElementById('ffiVisualization');
    if (!container) return;
    
    const ffiData = window.analysisData.unsafe_ffi;
    if (!ffiData || !ffiData.enhanced_ffi_data || ffiData.enhanced_ffi_data.length === 0) {
        container.innerHTML = `
            <div class="bg-white rounded-xl p-6 card-shadow">
                <h2 class="text-xl font-semibold mb-4 flex items-center">
                    <i class="fa fa-shield text-green-500 mr-2"></i>Unsafe/FFI Analysis
                </h2>
                <div class="text-center py-8 text-gray-500">
                    <i class="fa fa-shield text-2xl mb-2"></i>
                    <p>No unsafe/FFI operations detected</p>
                    <p class="text-sm mt-1">This is generally good for memory safety!</p>
                </div>
            </div>
        `;
        return;
    }
    
    const enhancedData = ffiData.enhanced_ffi_data || [];
    const boundaryEvents = ffiData.boundary_events || [];
    
    container.innerHTML = `
        <div class="bg-white rounded-xl p-6 card-shadow">
            <h2 class="text-xl font-semibold mb-4 flex items-center">
                <i class="fa fa-exclamation-triangle text-red-500 mr-2"></i>Unsafe/FFI Analysis
            </h2>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
                <div class="bg-red-100 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-red-600">${enhancedData.length}</div>
                    <div class="text-sm text-gray-600">Unsafe Operations</div>
                </div>
                <div class="bg-blue-100 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-blue-600">${boundaryEvents.length}</div>
                    <div class="text-sm text-gray-600">Boundary Events</div>
                </div>
                <div class="bg-yellow-100 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-yellow-600">${enhancedData.reduce((sum, item) => sum + (item.safety_violations || 0), 0)}</div>
                    <div class="text-sm text-gray-600">Safety Violations</div>
                </div>
            </div>
            <div class="overflow-x-auto">
                <table class="w-full text-sm">
                    <thead>
                        <tr class="border-b">
                            <th class="text-left py-2">Pointer</th>
                            <th class="text-left py-2">Size</th>
                            <th class="text-left py-2">FFI Tracked</th>
                            <th class="text-left py-2">Safety Violations</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${enhancedData.map(item => `
                            <tr class="border-b">
                                <td class="py-2 font-mono text-xs">${item.ptr}</td>
                                <td class="py-2">${formatBytes(item.size || 0)}</td>
                                <td class="py-2">
                                    <span class="px-2 py-1 rounded text-xs ${item.ffi_tracked ? 'bg-blue-500' : 'bg-red-500'} text-white">
                                        ${item.ffi_tracked ? 'Yes' : 'No'}
                                    </span>
                                </td>
                                <td class="py-2">${item.safety_violations || 0}</td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
            </div>
        </div>
    `;
}

// Initialize memory fragmentation
function initMemoryFragmentation() {
    const container = document.getElementById('memoryFragmentation');
    if (!container) return;
    
    container.innerHTML = `
        <div class="bg-white rounded-xl p-6 card-shadow">
            <h2 class="text-xl font-semibold mb-4 flex items-center">
                <i class="fa fa-puzzle-piece text-orange-500 mr-2"></i>Memory Fragmentation Analysis
            </h2>
            <div class="text-center py-8 text-gray-500">
                <i class="fa fa-info-circle text-2xl mb-2"></i>
                <p>Memory fragmentation analysis will be implemented here</p>
            </div>
        </div>
    `;
}

// Initialize memory growth trends
function initMemoryGrowthTrends() {
    const container = document.getElementById('memoryGrowthTrends');
    if (!container) return;
    
    container.innerHTML = `
        <div class="bg-white rounded-xl p-6 card-shadow">
            <h2 class="text-xl font-semibold mb-4 flex items-center">
                <i class="fa fa-line-chart text-green-500 mr-2"></i>Memory Growth Trends
            </h2>
            <div class="text-center py-8 text-gray-500">
                <i class="fa fa-info-circle text-2xl mb-2"></i>
                <p>Memory growth trends analysis will be implemented here</p>
            </div>
        </div>
    `;
}

// Initialize allocations table
function initAllocationsTable() {
    const tbody = document.getElementById('allocations-table');
    if (!tbody) return;
    
    const allocations = window.analysisData.memory_analysis?.allocations || [];
    
    if (allocations.length === 0) {
        tbody.innerHTML = '<tr><td colspan="4" class="px-4 py-8 text-center text-gray-500">No allocations found</td></tr>';
        return;
    }
    
    // Show first 50 allocations
    const displayAllocations = allocations.slice(0, 50);
    
    tbody.innerHTML = displayAllocations.map(alloc => `
        <tr>
            <td class="px-4 py-2">${alloc.var_name || 'Unknown'}</td>
            <td class="px-4 py-2">${formatTypeName(alloc.type_name || 'Unknown')}</td>
            <td class="px-4 py-2 text-right">${formatBytes(alloc.size || 0)}</td>
            <td class="px-4 py-2 text-right">${new Date(alloc.timestamp_alloc / 1000000).toLocaleTimeString()}</td>
        </tr>
    `).join('');
}

// Initialize generic types table
function initGenericTypesTable() {
    const tbody = document.getElementById('generic-types-table-body');
    if (!tbody) return;
    
    const genericTypes = window.analysisData.complex_types?.categorized_types?.generic_types || [];
    
    if (genericTypes.length === 0) {
        tbody.innerHTML = '<tr><td colspan="6" class="px-6 py-8 text-center text-gray-500">No generic types found</td></tr>';
        return;
    }
    
    tbody.innerHTML = genericTypes.map(type => `
        <tr>
            <td class="px-6 py-4">${type.var_name || 'Unknown'}</td>
            <td class="px-6 py-4">${formatTypeName(type.type_name || 'Unknown')}</td>
            <td class="px-6 py-4 font-mono text-xs">${type.ptr}</td>
            <td class="px-6 py-4">${formatBytes(type.size || 0)}</td>
            <td class="px-6 py-4">N/A</td>
            <td class="px-6 py-4">
                <span class="px-2 py-1 rounded text-xs ${getComplexityColor(type.complexity_score)} text-white">
                    ${type.complexity_score || 0}
                </span>
            </td>
        </tr>
    `).join('');
}

// Initialize variable graph
function initVariableGraph() {
    const container = document.getElementById('variable-graph-container');
    if (!container) return;
    
    container.innerHTML = `
        <div class="flex items-center justify-center h-full text-gray-500">
            <div class="text-center">
                <i class="fa fa-sitemap text-4xl mb-4"></i>
                <p>Variable relationship graph will be implemented here</p>
            </div>
        </div>
    `;
}

// Initialize complex type analysis
function initComplexTypeAnalysis() {
    const tbody = document.getElementById('complex-type-analysis-table');
    if (!tbody) return;
    
    const complexTypeAnalysis = window.analysisData.complex_types?.complex_type_analysis || [];
    
    if (complexTypeAnalysis.length === 0) {
        tbody.innerHTML = '<tr><td colspan="6" class="px-6 py-8 text-center text-gray-500">No complex type analysis available</td></tr>';
        return;
    }
    
    tbody.innerHTML = complexTypeAnalysis.map(analysis => `
        <tr>
            <td class="px-6 py-4">${formatTypeName(analysis.type_name)}</td>
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
            <td class="px-6 py-4 text-center">${analysis.allocation_count}</td>
            <td class="px-6 py-4 text-center">${formatBytes(analysis.total_size)}</td>
            <td class="px-6 py-4">${analysis.optimization_suggestions?.join(', ') || 'None'}</td>
        </tr>
    `).join('');
}

// Initialize memory optimization recommendations
function initMemoryOptimizationRecommendations() {
    const container = document.getElementById('memory-optimization-recommendations');
    if (!container) return;
    
    const recommendations = window.analysisData.complex_types?.optimization_recommendations || [];
    
    if (recommendations.length === 0) {
        container.innerHTML = '<li class="text-gray-500">No specific recommendations available</li>';
        return;
    }
    
    container.innerHTML = recommendations.map(rec => `
        <li class="flex items-start">
            <i class="fa fa-lightbulb-o text-yellow-500 mr-2 mt-1"></i>
            <span>${rec}</span>
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
    
    new Chart(ctx, {
        type: 'doughnut',
        data: {
            labels: Object.keys(riskLevels),
            datasets: [{
                data: Object.values(riskLevels),
                backgroundColor: ['#10b981', '#f59e0b', '#ef4444']
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false
        }
    });
}

// Utility functions
function updateElement(id, value) {
    const element = document.getElementById(id);
    if (element) {
        element.textContent = value;
    }
}

function getComplexityColor(score) {
    if (score <= 2) return 'bg-green-500';
    if (score <= 5) return 'bg-yellow-500';
    if (score <= 8) return 'bg-orange-500';
    return 'bg-red-500';
}

function getEfficiencyColor(efficiency) {
    if (efficiency >= 80) return 'bg-green-500';
    if (efficiency >= 60) return 'bg-yellow-500';
    if (efficiency >= 40) return 'bg-orange-500';
    return 'bg-red-500';
}

// Initialize dashboard when DOM is loaded
document.addEventListener("DOMContentLoaded", () => {
    console.log('MemScope dashboard loaded');
    initializeDashboard();
});