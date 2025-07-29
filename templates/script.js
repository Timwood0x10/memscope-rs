// MemScope Dashboard JavaScript - Complete version with theme support and collapsible tables
// This file contains comprehensive functions for memory analysis dashboard

// Global data store - will be populated by HTML template
window.analysisData = window.analysisData || {};

// Initialize all dashboard components
function initializeDashboard() {
    console.log('🚀 Initializing MemScope dashboard...');
    console.log('📊 Available data:', Object.keys(window.analysisData || {}));

    // Initialize theme system first
    initThemeToggle();

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

    if (isDark) {
        html.classList.remove('light');
        html.classList.add('dark');
    } else {
        html.classList.remove('dark');
        html.classList.add('light');
    }

    // Apply theme to all modules that need explicit dark mode support
    applyThemeToAllModules(isDark);

    // Destroy existing charts before reinitializing
    destroyAllCharts();

    // Reinitialize charts to apply theme changes
    setTimeout(() => {
        initCharts();
        initFFIRiskChart();
    }, 100);
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

    // Initialize complex type analysis chart
    initComplexTypeAnalysisChart();
}

// Initialize complexity chart
function initComplexityChart() {
    const ctx = document.getElementById('complexity-chart');
    if (!ctx) return;

    const complexTypes = window.analysisData.complex_types?.summary?.complexity_distribution || {};

    // Destroy existing chart if it exists
    if (window.chartInstances['complexity-chart']) {
        window.chartInstances['complexity-chart'].destroy();
    }

    window.chartInstances['complexity-chart'] = new Chart(ctx, {
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

// Initialize performance chart
function initPerformanceChart() {
    const ctx = document.getElementById('performance-chart');
    if (!ctx) return;

    const performance = window.analysisData.performance?.memory_performance || {};
    const isDark = document.documentElement.classList.contains('dark');

    // Destroy existing chart if it exists
    if (window.chartInstances['performance-chart']) {
        window.chartInstances['performance-chart'].destroy();
    }

    window.chartInstances['performance-chart'] = new Chart(ctx, {
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

// Initialize memory usage analysis with enhanced data processing
function initMemoryUsageAnalysis() {
    const container = document.getElementById('memory-usage-analysis');
    if (!container) return;

    // Process memory data with validation
    const memoryData = processMemoryAnalysisData(window.analysisData);
    const allocations = memoryData.allocations;

    if (allocations.length === 0 || memoryData.isFallback) {
        container.innerHTML = `
            <div class="h-full flex items-center justify-center text-muted">
                <div class="text-center">
                    <i class="fa fa-info-circle text-4xl mb-4"></i>
                    <h4 class="text-lg font-semibold mb-2 text-heading">No Memory Data Available</h4>
                    <p class="text-sm text-secondary">No memory allocation data found for analysis</p>
                    <p class="text-xs mt-2 text-muted">Use memory tracking features to collect data</p>
                </div>
            </div>
        `;
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

    const userPercentage = totalMemory > 0 ? (userMemory / totalMemory * 100) : 0;
    const systemPercentage = totalMemory > 0 ? (systemMemory / totalMemory * 100) : 0;

    container.innerHTML = `
        <div class="h-full flex flex-col">
            <h4 class="text-lg font-semibold mb-4 text-center text-heading">Memory Usage Analysis</h4>
            
            <!-- Statistics Grid -->
            <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
                <div class="bg-blue-50 dark:bg-blue-900/20 p-3 rounded-lg text-center">
                    <div class="text-2xl font-bold text-blue-600 dark:text-blue-400">${allocations.length}</div>
                    <div class="text-xs text-blue-600 dark:text-blue-400">Total Allocations</div>
                </div>
                <div class="bg-green-50 dark:bg-green-900/20 p-3 rounded-lg text-center">
                    <div class="text-2xl font-bold text-green-600 dark:text-green-400">${formatBytes(totalMemory)}</div>
                    <div class="text-xs text-green-600 dark:text-green-400">Total Memory</div>
                </div>
                <div class="bg-purple-50 dark:bg-purple-900/20 p-3 rounded-lg text-center">
                    <div class="text-2xl font-bold text-purple-600 dark:text-purple-400">${formatBytes(stats.averageSize)}</div>
                    <div class="text-xs text-purple-600 dark:text-purple-400">Average Size</div>
                </div>
                <div class="bg-orange-50 dark:bg-orange-900/20 p-3 rounded-lg text-center">
                    <div class="text-2xl font-bold text-orange-600 dark:text-orange-400">${formatBytes(stats.largestAllocation)}</div>
                    <div class="text-xs text-orange-600 dark:text-orange-400">Largest Block</div>
                </div>
            </div>
            
            <!-- Memory Distribution -->
            <div class="flex-grow">
                <h5 class="text-md font-medium mb-3 text-gray-900 dark:text-white">Memory Distribution</h5>
                <div class="space-y-4">
                    <div>
                        <div class="flex justify-between items-center mb-2">
                            <span class="text-sm font-medium text-blue-600 dark:text-blue-400">User Allocations (${stats.userAllocations})</span>
                            <span class="text-sm text-gray-600 dark:text-gray-300">${formatBytes(userMemory)} (${userPercentage.toFixed(1)}%)</span>
                        </div>
                        <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3">
                            <div class="bg-blue-500 h-3 rounded-full transition-all duration-500" style="width: ${userPercentage}%"></div>
                        </div>
                    </div>
                    <div>
                        <div class="flex justify-between items-center mb-2">
                            <span class="text-sm font-medium text-gray-600 dark:text-gray-300">System Allocations (${stats.systemAllocations})</span>
                            <span class="text-sm text-gray-600 dark:text-gray-300">${formatBytes(systemMemory)} (${systemPercentage.toFixed(1)}%)</span>
                        </div>
                        <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3">
                            <div class="bg-gray-500 h-3 rounded-full transition-all duration-500" style="width: ${systemPercentage}%"></div>
                        </div>
                    </div>
                </div>
                
                <!-- Memory Trends -->
                ${memoryData.trends && (memoryData.trends.peak_memory > 0 || memoryData.trends.growth_rate !== 0) ? `
                    <div class="mt-6 pt-4 border-t border-gray-200 dark:border-gray-600">
                        <h5 class="text-md font-medium mb-3 text-gray-900 dark:text-white">Memory Trends</h5>
                        <div class="grid grid-cols-2 gap-4 text-sm">
                            <div>
                                <span class="text-gray-600 dark:text-gray-400">Peak Memory:</span>
                                <span class="font-medium text-gray-900 dark:text-white ml-2">${formatBytes(memoryData.trends.peak_memory)}</span>
                            </div>
                            <div>
                                <span class="text-gray-600 dark:text-gray-400">Growth Rate:</span>
                                <span class="font-medium text-gray-900 dark:text-white ml-2">${memoryData.trends.growth_rate}%</span>
                            </div>
                        </div>
                    </div>
                ` : ''}
            </div>
        </div>
    `;
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
        tbody.innerHTML = '<tr><td colspan="4" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No allocations found</td></tr>';
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
                <td class="px-4 py-2 text-gray-900 dark:text-gray-100">${alloc.var_name || 'System Allocation'}</td>
                <td class="px-4 py-2 text-gray-900 dark:text-gray-100">${formatTypeName(alloc.type_name || 'System Allocation')}</td>
                <td class="px-4 py-2 text-right text-gray-900 dark:text-gray-100">${formatBytes(alloc.size || 0)}</td>
                <td class="px-4 py-2 text-right text-gray-900 dark:text-gray-100">${new Date(alloc.timestamp_alloc / 1000000).toLocaleTimeString()}</td>
            </tr>
        `).join('');

        if (!showAll && allocations.length > maxInitialRows) {
            tbody.innerHTML += `
                <tr class="bg-gray-50 dark:bg-gray-700">
                    <td colspan="4" class="px-4 py-2 text-center text-gray-500 dark:text-gray-400 text-sm">
                        ... and ${allocations.length - maxInitialRows} more allocations
                    </td>
                </tr>
            `;
        }
    }

    // Initial render
    renderTable(false);

    // Toggle functionality
    if (toggleButton && allocations.length > maxInitialRows) {
        console.log('📊 Setting up toggle button for', allocations.length, 'allocations');

        // Remove any existing event listeners
        const newToggleButton = toggleButton.cloneNode(true);
        toggleButton.parentNode.replaceChild(newToggleButton, toggleButton);

        newToggleButton.addEventListener('click', function (e) {
            e.preventDefault();
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
        varDiv.className = 'flex items-center py-4 border-b border-gray-100 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors';

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
                <div class="text-sm font-semibold text-gray-800 dark:text-gray-200">${group.var_name}</div>
                <div class="text-xs text-gray-500 dark:text-gray-400">${displayTypeName}</div>
            </div>
            <div class="flex-grow relative bg-gray-200 dark:bg-gray-600 rounded-full h-6 overflow-hidden">
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
                <div class="text-xs text-gray-600 dark:text-gray-400">
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
        varDiv.className = 'flex items-end py-3 border-b border-gray-100 dark:border-gray-700';

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
            <div class="w-40 flex-shrink-0 text-sm font-medium dark:text-gray-200">
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

// Initialize FFI visualization with enhanced dashboard style
function initFFIVisualization() {
    console.log('🔄 Initializing FFI visualization...');

    const container = document.getElementById('ffiVisualization');
    if (!container) return;

    const ffiData = window.analysisData.unsafe_ffi;
    if (!ffiData || !ffiData.enhanced_ffi_data || ffiData.enhanced_ffi_data.length === 0) {
        container.innerHTML = `
            <div class="bg-gradient-to-br from-green-50 to-green-100 dark:from-green-900 dark:to-green-800 rounded-xl p-6 card-shadow border border-green-200 dark:border-green-700">
                <h2 class="text-xl font-semibold mb-4 flex items-center dark:text-white">
                    <i class="fa fa-shield text-green-500 mr-2"></i>Unsafe/FFI Analysis
                </h2>
                <div class="text-center py-8 text-green-600 dark:text-green-300">
                    <i class="fa fa-shield text-4xl mb-4"></i>
                    <h3 class="text-lg font-semibold mb-2">No Unsafe/FFI Operations Detected</h3>
                    <p class="text-sm">This is generally good for memory safety!</p>
                </div>
            </div>
        `;
        return;
    }

    const enhancedData = ffiData.enhanced_ffi_data || [];
    const boundaryEvents = ffiData.boundary_events || [];

    // Calculate statistics
    const unsafeAllocations = enhancedData.filter(item => !item.ffi_tracked).length;
    const ffiAllocations = enhancedData.filter(item => item.ffi_tracked).length;
    const safetyViolations = enhancedData.reduce((sum, item) => sum + (item.safety_violations || 0), 0);
    const unsafeMemory = enhancedData.reduce((sum, item) => sum + (item.size || 0), 0);

    // Create dashboard inspired by SVG style
    container.innerHTML = `
        <div class="bg-gradient-to-br from-gray-800 to-gray-900 dark:from-gray-900 dark:to-black rounded-xl p-6 text-white shadow-2xl">
            <h2 class="text-2xl font-bold mb-6 text-center text-white">
                Unsafe Rust & FFI Memory Analysis Dashboard
            </h2>
            
            <!-- Key Metrics Row - inspired by SVG -->
            <div class="grid grid-cols-2 md:grid-cols-5 gap-4 mb-8">
                <div class="bg-red-500 bg-opacity-20 border-2 border-red-500 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-red-400">${unsafeAllocations}</div>
                    <div class="text-xs text-gray-300 uppercase tracking-wide">Unsafe Allocations</div>
                </div>
                <div class="bg-blue-500 bg-opacity-20 border-2 border-blue-500 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-blue-400">${ffiAllocations}</div>
                    <div class="text-xs text-gray-300 uppercase tracking-wide">FFI Allocations</div>
                </div>
                <div class="bg-yellow-500 bg-opacity-20 border-2 border-yellow-500 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-yellow-400">${boundaryEvents.length}</div>
                    <div class="text-xs text-gray-300 uppercase tracking-wide">Boundary Crossings</div>
                </div>
                <div class="bg-orange-500 bg-opacity-20 border-2 border-orange-500 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-orange-400">${safetyViolations}</div>
                    <div class="text-xs text-gray-300 uppercase tracking-wide">Safety Violations</div>
                </div>
                <div class="bg-purple-500 bg-opacity-20 border-2 border-purple-500 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-purple-400">${formatBytes(unsafeMemory)}</div>
                    <div class="text-xs text-gray-300 uppercase tracking-wide">Unsafe Memory</div>
                </div>
            </div>
            
            <!-- Memory Safety Status -->
            ${safetyViolations > 0 ? `
            <div class="bg-red-500 bg-opacity-20 border-2 border-red-500 rounded-lg p-6 mb-6">
                <h3 class="text-lg font-bold text-red-400 mb-4 text-center">
                    ${safetyViolations} Safety Violations Detected
                </h3>
                <div class="text-red-300 text-sm space-y-1">
                    ${enhancedData.filter(item => (item.safety_violations || 0) > 0).map(item =>
        `<div>• Pointer ${item.ptr}: ${item.safety_violations} violations</div>`
    ).join('')}
                </div>
            </div>
            ` : `
            <div class="bg-green-500 bg-opacity-20 border-2 border-green-500 rounded-lg p-6 mb-6">
                <h3 class="text-lg font-bold text-green-400 text-center">
                    No Safety Violations Detected
                </h3>
            </div>
            `}
            
            <!-- Cross-Language Memory Flow -->
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
                <div class="bg-gray-700 bg-opacity-50 rounded-lg p-4">
                    <h4 class="text-lg font-semibold mb-3 text-center">Memory Allocation Sources</h4>
                    <div class="flex justify-center items-end space-x-8 h-32">
                        <div class="flex flex-col items-center">
                            <div class="bg-red-500 rounded" style="width: 40px; height: ${Math.max(16, unsafeAllocations * 4)}px; margin-bottom: 8px;"></div>
                            <div class="text-red-400 font-bold text-sm">${unsafeAllocations}</div>
                            <div class="text-gray-300 text-xs">Unsafe Rust</div>
                        </div>
                        <div class="flex flex-col items-center">
                            <div class="bg-blue-500 rounded" style="width: 40px; height: ${Math.max(16, ffiAllocations * 4)}px; margin-bottom: 8px;"></div>
                            <div class="text-blue-400 font-bold text-sm">${ffiAllocations}</div>
                            <div class="text-gray-300 text-xs">FFI</div>
                        </div>
                    </div>
                </div>
                
                <div class="bg-gray-700 bg-opacity-50 rounded-lg p-4">
                    <h4 class="text-lg font-semibold mb-3 text-center">Unsafe Memory Hotspots</h4>
                    <div class="space-y-2 max-h-32 overflow-y-auto">
                        ${enhancedData.slice(0, 6).map(item => `
                            <div class="flex justify-between items-center text-sm">
                                <span class="font-mono text-xs">${item.ptr}</span>
                                <span class="px-2 py-1 rounded text-xs ${item.ffi_tracked ? 'bg-blue-500' : 'bg-red-500'} text-white">
                                    ${formatBytes(item.size || 0)}
                                </span>
                            </div>
                        `).join('')}
                    </div>
                </div>
            </div>
            
            <!-- Detailed FFI Operations Table -->
            <div class="bg-gray-700 bg-opacity-50 rounded-lg p-4">
                <h4 class="text-lg font-semibold mb-3">FFI Operations Details</h4>
                <div class="overflow-x-auto">
                    <table class="w-full text-sm">
                        <thead>
                            <tr class="border-b border-gray-600">
                                <th class="text-left py-2">Pointer</th>
                                <th class="text-left py-2">Size</th>
                                <th class="text-left py-2">Type</th>
                                <th class="text-left py-2">Safety</th>
                            </tr>
                        </thead>
                        <tbody>
                            ${enhancedData.map(item => `
                                <tr class="border-b border-gray-700 hover:bg-gray-600 hover:bg-opacity-30">
                                    <td class="py-2 font-mono text-xs">${item.ptr}</td>
                                    <td class="py-2">${formatBytes(item.size || 0)}</td>
                                    <td class="py-2">
                                        <span class="px-2 py-1 rounded text-xs ${item.ffi_tracked ? 'bg-blue-500' : 'bg-red-500'} text-white">
                                            ${item.ffi_tracked ? 'FFI' : 'Unsafe'}
                                        </span>
                                    </td>
                                    <td class="py-2">
                                        <span class="px-2 py-1 rounded text-xs ${(item.safety_violations || 0) === 0 ? 'bg-green-500' : 'bg-red-500'} text-white">
                                            ${(item.safety_violations || 0) === 0 ? 'Safe' : `${item.safety_violations} violations`}
                                        </span>
                                    </td>
                                </tr>
                            `).join('')}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    `;
}

// Initialize memory fragmentation analysis
function initMemoryFragmentation() {
    const container = document.getElementById('memoryFragmentation');
    if (!container) return;

    const allocations = window.analysisData.memory_analysis?.allocations || [];

    // Analyze memory fragmentation
    const sortedAllocs = allocations
        .filter(alloc => alloc.ptr && alloc.size)
        .map(alloc => ({
            address: parseInt(alloc.ptr.replace('0x', ''), 16),
            size: alloc.size,
            type: alloc.type_name || 'System Allocation'
        }))
        .sort((a, b) => a.address - b.address);

    let gaps = 0;
    let totalGapSize = 0;
    let maxGap = 0;

    for (let i = 1; i < sortedAllocs.length; i++) {
        const prevEnd = sortedAllocs[i - 1].address + sortedAllocs[i - 1].size;
        const currentStart = sortedAllocs[i].address;

        if (currentStart > prevEnd) {
            const gapSize = currentStart - prevEnd;
            gaps++;
            totalGapSize += gapSize;
            maxGap = Math.max(maxGap, gapSize);
        }
    }

    const totalMemory = sortedAllocs.reduce((sum, alloc) => sum + alloc.size, 0);
    const fragmentationRatio = totalMemory > 0 ? (totalGapSize / (totalMemory + totalGapSize) * 100) : 0;

    container.innerHTML = `
        <div class="bg-white dark:bg-gray-800 rounded-xl p-6 card-shadow transition-colors">
            <h2 class="text-xl font-semibold mb-4 flex items-center dark:text-white">
                <i class="fa fa-puzzle-piece text-orange-500 mr-2"></i>Memory Fragmentation Analysis
            </h2>
            
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
                <div class="bg-orange-100 dark:bg-orange-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-orange-600 dark:text-orange-400">${fragmentationRatio.toFixed(1)}%</div>
                    <div class="text-sm text-gray-600 dark:text-gray-300">Fragmentation Ratio</div>
                </div>
                <div class="bg-blue-100 dark:bg-blue-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-blue-600 dark:text-blue-400">${gaps}</div>
                    <div class="text-sm text-gray-600 dark:text-gray-300">Memory Gaps</div>
                </div>
                <div class="bg-green-100 dark:bg-green-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-green-600 dark:text-green-400">${formatBytes(maxGap)}</div>
                    <div class="text-sm text-gray-600 dark:text-gray-300">Largest Gap</div>
                </div>
                <div class="bg-purple-100 dark:bg-purple-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-purple-600 dark:text-purple-400">${sortedAllocs.length}</div>
                    <div class="text-sm text-gray-600 dark:text-gray-300">Memory Blocks</div>
                </div>
            </div>
            
            <div class="mb-4">
                <h4 class="font-semibold mb-2 dark:text-white">Fragmentation Assessment</h4>
                <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-4">
                    <div class="h-4 rounded-full transition-all duration-500 ${fragmentationRatio < 10 ? 'bg-green-500' :
            fragmentationRatio < 25 ? 'bg-yellow-500' :
                fragmentationRatio < 50 ? 'bg-orange-500' : 'bg-red-500'
        }" style="width: ${Math.min(fragmentationRatio, 100)}%"></div>
                </div>
                <div class="text-sm text-gray-600 dark:text-gray-300 mt-2">
                    ${fragmentationRatio < 10 ? 'Excellent memory layout with minimal fragmentation.' :
            fragmentationRatio < 25 ? 'Good memory layout with low fragmentation.' :
                fragmentationRatio < 50 ? 'Moderate fragmentation detected. Consider memory pool allocation.' :
                    'High fragmentation detected. Memory layout optimization recommended.'}
                </div>
            </div>
            
            <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                <h4 class="font-semibold mb-2 dark:text-white">Memory Layout Visualization</h4>
                <div class="h-8 bg-gray-200 dark:bg-gray-600 rounded relative overflow-hidden">
                    ${sortedAllocs.slice(0, 20).map((alloc, index) => {
                        const width = Math.max(2, (alloc.size / totalMemory) * 100);
                        const left = (index / 20) * 100;
                        return `<div class="absolute h-full bg-blue-500 opacity-70" 
                                     style="left: ${left}%; width: ${width}%;" 
                                     title="${alloc.type}: ${formatBytes(alloc.size)}"></div>`;
                    }).join('')}
                </div>
                <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                    Showing first 20 allocations. Each block represents a memory allocation.
                </div>
            </div>
        </div>
    `;
}

// Initialize memory growth trends
function initMemoryGrowthTrends() {
    const container = document.getElementById('memoryGrowthTrends');
    if (!container) return;

    const allocations = window.analysisData.memory_analysis?.allocations || [];

    // Sort allocations by timestamp
    const sortedAllocs = allocations
        .filter(alloc => alloc.timestamp_alloc)
        .sort((a, b) => a.timestamp_alloc - b.timestamp_alloc);

    if (sortedAllocs.length === 0) {
        container.innerHTML = `
            <div class="bg-white dark:bg-gray-800 rounded-xl p-6 card-shadow transition-colors">
                <h2 class="text-xl font-semibold mb-4 flex items-center dark:text-white">
                    <i class="fa fa-line-chart text-green-500 mr-2"></i>Memory Growth Trends
                </h2>
                <div class="text-center py-8 text-gray-500 dark:text-gray-400">
                    <i class="fa fa-info-circle text-2xl mb-2"></i>
                    <p>No timestamp data available for growth analysis</p>
                </div>
            </div>
        `;
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
                index: index
            });
        }
    });

    const startMemory = timePoints[0]?.memory || 0;
    const endMemory = timePoints[timePoints.length - 1]?.memory || 0;
    const growthRate = startMemory > 0 ? ((endMemory - startMemory) / startMemory * 100) : 0;
    const averageMemory = timePoints.reduce((sum, point) => sum + point.memory, 0) / timePoints.length;

    container.innerHTML = `
        <div class="bg-white dark:bg-gray-800 rounded-xl p-6 card-shadow transition-colors">
            <h2 class="text-xl font-semibold mb-4 flex items-center dark:text-white">
                <i class="fa fa-line-chart text-green-500 mr-2"></i>Memory Growth Trends
            </h2>
            
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
                <div class="bg-red-100 dark:bg-red-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-red-600 dark:text-red-400">${formatBytes(peakMemory)}</div>
                    <div class="text-sm text-gray-600 dark:text-gray-300">Peak Memory Usage</div>
                </div>
                <div class="bg-blue-100 dark:bg-blue-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-blue-600 dark:text-blue-400">${formatBytes(averageMemory)}</div>
                    <div class="text-sm text-gray-600 dark:text-gray-300">Average Memory Usage</div>
                </div>
                <div class="bg-green-100 dark:bg-green-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold ${growthRate > 0 ? 'text-red-600 dark:text-red-400' : 'text-green-600 dark:text-green-400'}">${growthRate > 0 ? '+' : ''}${growthRate.toFixed(1)}%</div>
                    <div class="text-sm text-gray-600 dark:text-gray-300">Growth Rate</div>
                </div>
            </div>
            
            <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                <h4 class="font-semibold mb-2 dark:text-white">Memory Usage Over Time</h4>
                <div class="h-32 relative bg-white dark:bg-gray-600 rounded border dark:border-gray-500">
                    <svg class="w-full h-full" viewBox="0 0 400 120">
                        <polyline
                            fill="none"
                            stroke="#3b82f6"
                            stroke-width="2"
                            points="${timePoints.map((point, index) => {
        const x = (index / (timePoints.length - 1)) * 380 + 10;
        const y = 110 - ((point.memory / peakMemory) * 100);
        return `${x},${y}`;
    }).join(' ')}"
                        />
                        ${timePoints.map((point, index) => {
        const x = (index / (timePoints.length - 1)) * 380 + 10;
        const y = 110 - ((point.memory / peakMemory) * 100);
        return `<circle cx="${x}" cy="${y}" r="3" fill="#3b82f6" />`;
    }).join('')}
                    </svg>
                </div>
                <div class="text-xs text-gray-500 dark:text-gray-400 mt-2">
                    ${growthRate > 50 ? 'High memory growth detected - investigate for potential leaks.' :
            growthRate > 10 ? 'Moderate memory growth - monitor for potential issues.' :
                growthRate > -10 ? 'Stable memory usage with minimal growth.' :
                    'Memory usage is decreasing - good memory management.'}
                </div>
            </div>
        </div>
    `;
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
            alloc.type_name === nodeData.type && alloc.var_name !== nodeData.id
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
                    <p class="font-mono">${nodeData.type}</p>
                    <div class="flex items-center mt-1">
                        <div class="w-3 h-3 rounded-full mr-2" style="background-color: ${getEnhancedTypeColor(nodeData.type, nodeData.category || 'primitive')}"></div>
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
                        ${getTypeAnalysis(nodeData.type, nodeData.size)}
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
            .radius(d => Math.max(15, Math.sqrt(d.size) / 8) + 5)
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

    // Add circles to nodes
    node.append('circle')
        .attr('r', d => Math.max(12, Math.sqrt(d.size) / 8))
        .attr('fill', d => getEnhancedTypeColor(d.type, d.category))
        .attr('stroke', '#fff')
        .attr('stroke-width', 2)
        .style('filter', 'drop-shadow(0px 2px 4px rgba(0,0,0,0.2))')
        .on('mouseover', function(event, d) {
            d3.select(this)
                .transition()
                .duration(200)
                .attr('r', Math.max(15, Math.sqrt(d.size) / 6))
                .style('filter', 'drop-shadow(0px 4px 8px rgba(0,0,0,0.3))');
            
            // Highlight connected links
            link.style('stroke-opacity', l => 
                (l.source.id === d.id || l.target.id === d.id) ? 0.8 : 0.1
            );
        })
        .on('mouseout', function(event, d) {
            d3.select(this)
                .transition()
                .duration(200)
                .attr('r', Math.max(12, Math.sqrt(d.size) / 8))
                .style('filter', 'drop-shadow(0px 2px 4px rgba(0,0,0,0.2))');
            
            // Reset link opacity
            link.style('stroke-opacity', l => 0.3 + l.strength * 0.4);
        });

    // Add complexity indicators (small circles with numbers)
    const complexityGroup = node.append('g')
        .attr('class', 'complexity-indicator');
    
    complexityGroup.append('circle')
        .attr('r', 8)
        .attr('cx', d => Math.max(12, Math.sqrt(d.size) / 8) + 8)
        .attr('cy', d => -Math.max(12, Math.sqrt(d.size) / 8) - 8)
        .attr('fill', d => getComplexityColor(d.complexity))
        .attr('stroke', '#fff')
        .attr('stroke-width', 2);
    
    // Add complexity score text
    complexityGroup.append('text')
        .text(d => d.complexity || 2)
        .attr('x', d => Math.max(12, Math.sqrt(d.size) / 8) + 8)
        .attr('y', d => -Math.max(12, Math.sqrt(d.size) / 8) - 8 + 3)
        .attr('text-anchor', 'middle')
        .style('font-size', '10px')
        .style('font-weight', 'bold')
        .style('fill', '#fff')
        .style('pointer-events', 'none');

    // Add variable names
    node.append('text')
        .text(d => d.id)
        .attr('text-anchor', 'middle')
        .attr('dy', d => Math.max(12, Math.sqrt(d.size) / 8) + 15)
        .style('font-size', '11px')
        .style('font-weight', 'bold')
        .style('fill', 'var(--text-primary)')
        .style('pointer-events', 'none');

    // Add type labels
    node.append('text')
        .text(d => formatTypeName(d.type))
        .attr('text-anchor', 'middle')
        .attr('dy', d => Math.max(12, Math.sqrt(d.size) / 8) + 28)
        .style('font-size', '9px')
        .style('fill', 'var(--text-secondary)')
        .style('pointer-events', 'none');

    // Add click interaction
    const detailPanel = new NodeDetailPanel('variable-graph-container');
    
    node.on('click', function(event, d) {
        event.stopPropagation();
        console.log('Node clicked:', d.id, d);
        const position = {
            x: event.pageX,
            y: event.pageY
        };
        detailPanel.show(d, position);
    });

    // Click on empty space to hide panel
    svg.on('click', function(event) {
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
            <td class="px-6 py-4 text-center text-gray-900 dark:text-gray-100">${analysis.allocation_count}</td>
            <td class="px-6 py-4 text-center text-gray-900 dark:text-gray-100">${formatBytes(analysis.total_size)}</td>
            <td class="px-6 py-4 text-gray-900 dark:text-gray-100">${analysis.optimization_suggestions?.join(', ') || 'None'}</td>
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

// Initialize dashboard when DOM is loaded
document.addEventListener("DOMContentLoaded", () => {
    console.log('MemScope dashboard loaded');
    initializeDashboard();
});