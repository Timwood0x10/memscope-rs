// MemScope Dashboard JavaScript
// Enhanced memory analysis dashboard with lifetime visualization and FFI tracking

// Tailwind configuration
tailwind.config = {
    theme: {
        extend: {
            colors: {
                primary: '#3B82F6',
                secondary: '#10B981',
                accent: '#8B5CF6',
                neutral: '#1F2937',
                'neutral-light': '#F3F4F6',
                'ffi-red': '#EF4444',
                'safe-yellow': '#F59E0B',
                'safe-green': '#10B981',
            },
            fontFamily: {
                sans: ['Inter', 'system-ui', 'sans-serif'],
            },
        }
    }
}

// Global data store - will be populated by HTML template
window.analysisData = window.analysisData || {};

// Data validation and interaction utilities
window.dataUtils = {
    hasUnsafeFFIData: function () {
        const unsafeData = window.analysisData.basic_usage_snapshot_unsafe_ffi;
        if (!unsafeData) return false;

        const summary = unsafeData.summary || {};
        return (summary.unsafe_count > 0 ||
            summary.ffi_count > 0 ||
            summary.enhanced_entries > 0 ||
            summary.boundary_events > 0 ||
            summary.safety_violations > 0);
    },

    hasFFICallFlowData: function () {
        const unsafeData = window.analysisData.basic_usage_snapshot_unsafe_ffi;
        if (!unsafeData) return false;

        const enhancedData = unsafeData.enhanced_ffi_data || [];
        const boundaryEvents = unsafeData.boundary_events || [];
        return enhancedData.length > 0 || boundaryEvents.length > 0;
    },

    getEmptyStateMessage: function (type) {
        const messages = {
            'unsafe_ffi': 'No unsafe/FFI code detected in your codebase',
            'ffi_flow': 'No FFI call flow patterns found in the analysis'
        };
        return messages[type] || 'No data available';
    },

    createEmptyStateElement: function (message, icon = 'fa-info-circle') {
        return `
            <div class="data-empty-state text-center py-12">
                <div class="mb-4">
                    <i class="fa ${icon} text-6xl text-gray-300"></i>
                </div>
                <h3 class="text-lg font-medium text-gray-600 mb-2">${message}</h3>
                <p class="text-sm text-gray-500">This indicates good memory safety practices in your Rust code.</p>
            </div>
        `;
    },

    // Interactive data filtering and highlighting
    filterDataByRisk: function (riskLevel) {
        const unsafeData = window.analysisData.basic_usage_snapshot_unsafe_ffi;
        if (!unsafeData) return [];

        const filtered = [];

        // Filter enhanced FFI data by risk level
        if (unsafeData.enhanced_ffi_data) {
            unsafeData.enhanced_ffi_data.forEach(item => {
                if (item.source && typeof item.source === 'string') {
                    const riskMatch = item.source.match(/risk_level:\\s*(\\w+)/i);
                    if (riskMatch && riskMatch[1].toLowerCase() === riskLevel.toLowerCase()) {
                        filtered.push(item);
                    }
                }
            });
        }

        return filtered;
    },

    // Get interactive statistics
    getInteractiveStats: function () {
        const unsafeData = window.analysisData.basic_usage_snapshot_unsafe_ffi;
        if (!unsafeData) return null;

        const stats = {
            totalRiskItems: 0,
            riskLevels: { low: 0, medium: 0, high: 0 },
            boundaryEvents: (unsafeData.boundary_events || []).length,
            safetyViolations: (unsafeData.safety_violations || []).length,
            ffiPatterns: (unsafeData.ffi_patterns || []).length
        };

        // Analyze risk levels from enhanced data
        if (unsafeData.enhanced_ffi_data) {
            unsafeData.enhanced_ffi_data.forEach(item => {
                if (item.source && typeof item.source === 'string') {
                    const riskMatch = item.source.match(/risk_level:\\s*(\\w+)/i);
                    if (riskMatch) {
                        const level = riskMatch[1].toLowerCase();
                        if (stats.riskLevels[level] !== undefined) {
                            stats.riskLevels[level]++;
                            stats.totalRiskItems++;
                        }
                    }
                }
            });
        }

        return stats;
    }
};

// Initialize lifetime visualization from JSON data
function initLifetimeVisualization() {
    console.log('🔄 Initializing lifetime visualization...');
    
    // Get lifetime data from the global data store
    const lifetimeData = window.analysisData.lifetime;
    if (!lifetimeData || !lifetimeData.lifecycle_events) {
        console.warn('⚠️ No lifetime data found');
        showEmptyLifetimeState();
        return;
    }
    
    // Filter for user-defined variables (non-unknown var_name and type_name)
    const userVariables = lifetimeData.lifecycle_events.filter(event => 
        event.var_name && event.var_name !== 'unknown' && 
        event.type_name && event.type_name !== 'unknown'
    );
    
    console.log(`📊 Found ${userVariables.length} user-defined variables in lifetime data`);
    
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

// Render the lifetime visualization
function renderLifetimeVisualization(variableGroups) {
    const container = document.getElementById('lifetimeVisualization');
    if (!container) return;
    
    // Clear loading state
    container.innerHTML = '';
    
    // Get colorful scheme for different types - more vibrant colors
    const typeColors = [
        { bg: 'bg-red-500', border: 'border-red-500', color: '#ef4444' },
        { bg: 'bg-blue-500', border: 'border-blue-500', color: '#3b82f6' },
        { bg: 'bg-green-500', border: 'border-green-500', color: '#10b981' },
        { bg: 'bg-yellow-500', border: 'border-yellow-500', color: '#f59e0b' },
        { bg: 'bg-purple-500', border: 'border-purple-500', color: '#8b5cf6' },
        { bg: 'bg-pink-500', border: 'border-pink-500', color: '#ec4899' },
        { bg: 'bg-indigo-500', border: 'border-indigo-500', color: '#6366f1' },
        { bg: 'bg-teal-500', border: 'border-teal-500', color: '#14b8a6' },
        { bg: 'bg-orange-500', border: 'border-orange-500', color: '#f97316' },
        { bg: 'bg-cyan-500', border: 'border-cyan-500', color: '#06b6d4' },
        { bg: 'bg-lime-500', border: 'border-lime-500', color: '#84cc16' },
        { bg: 'bg-rose-500', border: 'border-rose-500', color: '#f43f5e' }
    ];
    
    // Calculate timeline bounds
    const allTimestamps = variableGroups.flatMap(group => 
        group.events.map(e => e.timestamp)
    );
    const minTime = Math.min(...allTimestamps);
    const maxTime = Math.max(...allTimestamps);
    const timeRange = maxTime - minTime;
    
    console.log(`📊 Timeline: ${minTime} to ${maxTime} (range: ${timeRange})`);
    
    // Render each variable
    variableGroups.forEach((group, index) => {
        const varDiv = document.createElement('div');
        varDiv.className = 'flex items-end py-3 border-b border-gray-100';
        
        // Assign colors cyclically to make it colorful
        const colors = typeColors[index % typeColors.length];
        
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
// Data processing functions
function getComplexTypesData() {
    if (window.analysisData && window.analysisData.complex_types) {
        console.log('Using embedded complex types data');
        return window.analysisData.complex_types;
    }
    if (window.embeddedJsonData && window.embeddedJsonData.complex_types) {
        console.log('Using embedded complex types data');
        return window.embeddedJsonData.complex_types;
    }
    console.log('No complex types data available - showing empty state');
    return null;
}

function getMemoryAnalysisData() {
    if (window.analysisData && window.analysisData.memory_analysis) {
        console.log('Using embedded memory analysis data');
        return window.analysisData.memory_analysis;
    }
    if (window.embeddedJsonData && window.embeddedJsonData.memory_analysis) {
        console.log('Using embedded memory analysis data');
        return window.embeddedJsonData.memory_analysis;
    }
    console.log('No memory analysis data available - showing empty state');
    return null;
}

function getPerformanceData() {
    if (window.analysisData && window.analysisData.performance) {
        console.log('Using embedded performance data');
        return window.analysisData.performance;
    }
    if (window.embeddedJsonData && window.embeddedJsonData.performance) {
        console.log('Using embedded performance data');
        return window.embeddedJsonData.performance;
    }
    console.log('No performance data available - showing empty state');
    return null;
}

function getLifetimeData() {
    if (window.analysisData && window.analysisData.lifetime) {
        console.log('Using embedded lifetime data');
        return window.analysisData.lifetime;
    }
    if (window.embeddedJsonData && window.embeddedJsonData.lifetime) {
        console.log('Using embedded lifetime data');
        return window.embeddedJsonData.lifetime;
    }
    console.log('No lifetime data available - showing empty state');
    return null;
}

function getFfiSnapshotData() {
    if (window.analysisData && window.analysisData.unsafe_ffi) {
        console.log('Using embedded FFI data');
        return window.analysisData.unsafe_ffi.enhanced_ffi_data || [];
    }
    if (window.embeddedJsonData && window.embeddedJsonData.unsafe_ffi) {
        console.log('Using embedded FFI data');
        return window.embeddedJsonData.unsafe_ffi.enhanced_ffi_data || [];
    }
    console.log('No FFI data available - showing empty state');
    return [];
}

// Fallback data definitions
// Memory allocations table population using real data - filter out system allocations
function populateMemoryAllocationsTable(memoryAnalysisData) {
    const tableBody = document.getElementById('allocations-table');
    if (!tableBody) return;
    
    // Handle null or missing data
    if (!memoryAnalysisData || !memoryAnalysisData.allocations || memoryAnalysisData.allocations.length === 0) {
        tableBody.innerHTML = '<tr><td colspan="4" class="px-4 py-8 text-center text-gray-500">No memory allocation data available</td></tr>';
        return;
    }
    
    // Clear existing content
    tableBody.innerHTML = '';
    
    // Filter out system allocations - only show user-defined variables
    const userAllocations = memoryAnalysisData.allocations.filter(allocation => 
        allocation.var_name && 
        allocation.var_name !== null && 
        allocation.var_name !== 'unknown' &&
        allocation.type_name && 
        allocation.type_name !== null && 
        allocation.type_name !== 'unknown'
    );
    
    // Show first 50 user allocations to avoid overwhelming the UI
    const allocationsToShow = userAllocations.slice(0, 50);
    
    allocationsToShow.forEach(allocation => {
        const row = document.createElement('tr');
        row.className = 'hover:bg-gray-50 transition-colors';
        
        // Format timestamp to readable date
        const allocTime = new Date(allocation.timestamp_alloc / 1000000); // Convert nanoseconds to milliseconds
        const timeStr = allocTime.toLocaleTimeString();
        
        // Determine allocation status
        const isActive = !allocation.timestamp_dealloc;
        const statusClass = isActive ? 'text-green-600' : 'text-gray-500';
        const statusText = isActive ? 'Active' : 'Deallocated';
        
        row.innerHTML = `
            <td class="px-4 py-2">
                <div class="font-mono text-sm">${allocation.var_name || 'Unknown'}</div>
                <div class="text-xs text-gray-500">${allocation.ptr}</div>
            </td>
            <td class="px-4 py-2">
                <div class="text-sm">${allocation.type_name || 'Unknown'}</div>
                <div class="text-xs ${statusClass}">${statusText}</div>
            </td>
            <td class="px-4 py-2 text-right">
                <span class="font-mono text-sm">${allocation.size} bytes</span>
            </td>
            <td class="px-4 py-2 text-right">
                <span class="text-xs text-gray-500">${timeStr}</span>
            </td>
        `;
        
        tableBody.appendChild(row);
    });
    
    // Add summary row if there are more allocations
    if (userAllocations.length > 50) {
        const summaryRow = document.createElement('tr');
        summaryRow.className = 'bg-gray-50 font-medium';
        summaryRow.innerHTML = `
            <td colspan="4" class="px-4 py-2 text-center text-gray-600">
                Showing 50 of ${userAllocations.length} user-defined allocations (${memoryAnalysisData.allocations.length} total)
            </td>
        `;
        tableBody.appendChild(summaryRow);
    } else if (userAllocations.length === 0) {
        tableBody.innerHTML = '<tr><td colspan="4" class="px-4 py-8 text-center text-gray-500">No user-defined variables found. Use track_var! macro to track variables.</td></tr>';
    }
}

// Allocation size distribution chart using real data
function createAllocationSizeChart(memoryAnalysisData) {
    const ctx = document.getElementById('allocation-size-chart');
    if (!ctx) return;
    
    // Handle null or missing data
    if (!memoryAnalysisData || !memoryAnalysisData.allocations || memoryAnalysisData.allocations.length === 0) {
        const context = ctx.getContext('2d');
        context.fillStyle = '#9CA3AF';
        context.font = '14px Arial';
        context.textAlign = 'center';
        context.fillText('No allocation data available', ctx.width / 2, ctx.height / 2);
        return;
    }
    
    // Categorize allocations by size
    const sizeCategories = {
        'Tiny (< 16B)': 0,
        'Small (16-64B)': 0,
        'Medium (64-256B)': 0,
        'Large (256-1KB)': 0,
        'Huge (> 1KB)': 0
    };
    
    memoryAnalysisData.allocations.forEach(allocation => {
        const size = allocation.size;
        if (size < 16) {
            sizeCategories['Tiny (< 16B)']++;
        } else if (size < 64) {
            sizeCategories['Small (16-64B)']++;
        } else if (size < 256) {
            sizeCategories['Medium (64-256B)']++;
        } else if (size < 1024) {
            sizeCategories['Large (256-1KB)']++;
        } else {
            sizeCategories['Huge (> 1KB)']++;
        }
    });
    
    new Chart(ctx.getContext('2d'), {
        type: 'doughnut',
        data: {
            labels: Object.keys(sizeCategories),
            datasets: [{
                data: Object.values(sizeCategories),
                backgroundColor: [
                    'rgba(34, 197, 94, 0.7)',   // Green for tiny
                    'rgba(59, 130, 246, 0.7)',  // Blue for small
                    'rgba(245, 158, 11, 0.7)',  // Yellow for medium
                    'rgba(239, 68, 68, 0.7)',   // Red for large
                    'rgba(147, 51, 234, 0.7)'   // Purple for huge
                ],
                borderColor: [
                    'rgb(34, 197, 94)',
                    'rgb(59, 130, 246)',
                    'rgb(245, 158, 11)',
                    'rgb(239, 68, 68)',
                    'rgb(147, 51, 234)'
                ],
                borderWidth: 2
            }]
        },
        options: {
            responsive: true,
            plugins: {
                legend: {
                    position: 'bottom'
                },
                title: {
                    display: true,
                    text: 'Memory Allocation Size Distribution'
                }
            }
        }
    });
}

// Utility functions
function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function updateStats(stats) {
    // Update statistics display
    const statsContainer = document.querySelector('.stats-grid');
    if (statsContainer && stats) {
        statsContainer.innerHTML = `
            <div class="stat-card">
                <span class="stat-value">${stats.active_allocations || 0}</span>
                <span class="stat-label">Active Allocations</span>
            </div>
            <div class="stat-card">
                <span class="stat-value">${formatBytes(stats.active_memory || 0)}</span>
                <span class="stat-label">Active Memory</span>
            </div>
            <div class="stat-card">
                <span class="stat-value">${formatBytes(stats.peak_memory || 0)}</span>
                <span class="stat-label">Peak Memory</span>
            </div>
            <div class="stat-card">
                <span class="stat-value">${stats.total_allocations || 0}</span>
                <span class="stat-label">Total Allocations</span>
            </div>
        `;
    }
}

// Create memory usage analysis visualization similar to memoryAnalysis.svg
function createMemoryUsageAnalysis(memoryAnalysisData, performanceData) {
    const container = document.getElementById('memory-usage-analysis');
    if (!container) return;
    
    // Calculate metrics from data
    const metrics = calculateMemoryMetrics(memoryAnalysisData, performanceData);
    
    // Create SVG-style visualization
    container.innerHTML = `
        <div class="text-center mb-6">
            <h3 class="text-xl font-light text-gray-700 mb-2" style="letter-spacing: 1px;">
                Rust Memory Usage Analysis
            </h3>
            <p class="text-xs font-semibold text-gray-500 uppercase tracking-wider">
                Key Performance Metrics
            </p>
        </div>
        
        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
            ${metrics.map(metric => `
                <div class="bg-white rounded-xl p-4 shadow-lg hover:shadow-xl transition-shadow">
                    <div class="flex flex-col items-center">
                        <div class="relative w-16 h-16 mb-3">
                            <svg class="w-16 h-16 transform -rotate-90" viewBox="0 0 36 36">
                                <path class="text-gray-200" stroke="currentColor" stroke-width="3" fill="none"
                                      d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"/>
                                <path class="text-${metric.color}" stroke="currentColor" stroke-width="3" fill="none"
                                      stroke-dasharray="${metric.percentage}, 100"
                                      d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"/>
                            </svg>
                            <div class="absolute inset-0 flex items-center justify-center">
                                <span class="text-sm font-bold text-${metric.color}">${metric.percentage}%</span>
                            </div>
                        </div>
                        <div class="text-center">
                            <p class="text-xs font-semibold text-gray-600 mb-1">${metric.label}</p>
                            <p class="text-sm font-bold text-gray-800">${metric.value}</p>
                            <div class="flex items-center justify-center mt-1">
                                <div class="w-2 h-2 rounded-full bg-${metric.color} mr-1"></div>
                                <span class="text-xs font-semibold text-${metric.color} uppercase">${metric.status}</span>
                            </div>
                        </div>
                    </div>
                </div>
            `).join('')}
        </div>
    `;
}

// Calculate memory metrics from data
function calculateMemoryMetrics(memoryAnalysisData, performanceData) {
    const defaultMetrics = [
        { label: 'Active Memory', value: '0KB', percentage: 0, color: 'blue-500', status: 'LOW' },
        { label: 'Peak Memory', value: '0KB', percentage: 0, color: 'red-500', status: 'LOW' },
        { label: 'Active Allocs', value: '0', percentage: 0, color: 'green-500', status: 'LOW' },
        { label: 'Reclamation', value: '0%', percentage: 0, color: 'yellow-500', status: 'LOW' },
        { label: 'Efficiency', value: '0%', percentage: 0, color: 'purple-500', status: 'LOW' },
        { label: 'Median Size', value: '0B', percentage: 0, color: 'teal-500', status: 'LOW' }
    ];
    
    if (!memoryAnalysisData || !memoryAnalysisData.allocations) {
        return defaultMetrics;
    }
    
    const allocations = memoryAnalysisData.allocations;
    const activeAllocations = allocations.filter(a => !a.timestamp_dealloc);
    const totalAllocated = allocations.reduce((sum, a) => sum + a.size, 0);
    const activeMemory = activeAllocations.reduce((sum, a) => sum + a.size, 0);
    const peakMemory = Math.max(totalAllocated, activeMemory);
    
    // Calculate sizes array for median
    const sizes = allocations.map(a => a.size).sort((a, b) => a - b);
    const medianSize = sizes.length > 0 ? sizes[Math.floor(sizes.length / 2)] : 0;
    
    // Calculate reclamation rate
    const deallocatedCount = allocations.filter(a => a.timestamp_dealloc).length;
    const reclamationRate = allocations.length > 0 ? (deallocatedCount / allocations.length) * 100 : 0;
    
    // Calculate efficiency (active memory / peak memory)
    const efficiency = peakMemory > 0 ? (activeMemory / peakMemory) * 100 : 0;
    
    return [
        {
            label: 'Active Memory',
            value: formatBytes(activeMemory),
            percentage: Math.min(100, Math.round((activeMemory / (1024 * 1024)) * 100)), // Normalize to MB
            color: 'blue-500',
            status: activeMemory > 500000 ? 'HIGH' : activeMemory > 100000 ? 'MEDIUM' : 'LOW'
        },
        {
            label: 'Peak Memory',
            value: formatBytes(peakMemory),
            percentage: 100, // Peak is always 100%
            color: 'red-500',
            status: peakMemory > 1000000 ? 'HIGH' : peakMemory > 200000 ? 'MEDIUM' : 'LOW'
        },
        {
            label: 'Active Allocs',
            value: activeAllocations.length.toString(),
            percentage: Math.min(100, Math.round((activeAllocations.length / 1000) * 100)), // Normalize to thousands
            color: 'green-500',
            status: activeAllocations.length > 1000 ? 'HIGH' : activeAllocations.length > 100 ? 'MEDIUM' : 'LOW'
        },
        {
            label: 'Reclamation',
            value: `${reclamationRate.toFixed(1)}%`,
            percentage: Math.round(reclamationRate),
            color: 'yellow-500',
            status: reclamationRate > 80 ? 'HIGH' : reclamationRate > 50 ? 'MEDIUM' : 'LOW'
        },
        {
            label: 'Efficiency',
            value: `${efficiency.toFixed(1)}%`,
            percentage: Math.round(efficiency),
            color: 'purple-500',
            status: efficiency > 80 ? 'HIGH' : efficiency > 50 ? 'MEDIUM' : 'LOW'
        },
        {
            label: 'Median Size',
            value: formatBytes(medianSize),
            percentage: Math.min(100, Math.round((medianSize / 1024) * 100)), // Normalize to KB
            color: 'teal-500',
            status: medianSize > 1024 ? 'HIGH' : medianSize > 100 ? 'MEDIUM' : 'LOW'
        }
    ];
}

// Dashboard initialization and data population
function initializeDashboard() {
    console.log('Initializing MemScope dashboard...');
    
    // Use embedded data if available, otherwise show empty state
    const complexTypesData = getComplexTypesData();
    const memoryAnalysisData = getMemoryAnalysisData();
    const performanceData = getPerformanceData();
    const lifetimeData = getLifetimeData();
    const ffiSnapshotData = getFfiSnapshotData();

    // Populate summary data
    const totalComplexTypesEl = document.getElementById('total-complex-types');
    const totalAllocationsEl = document.getElementById('total-allocations');
    const genericTypeCountEl = document.getElementById('generic-type-count');
    const unsafeFfiCountEl = document.getElementById('unsafe-ffi-count');

    if (totalComplexTypesEl) totalComplexTypesEl.textContent = complexTypesData?.summary?.total_complex_types || 0;
    if (totalAllocationsEl) totalAllocationsEl.textContent = memoryAnalysisData?.allocations?.length || 0;
    if (genericTypeCountEl) genericTypeCountEl.textContent = complexTypesData?.categorized_types?.generic_types?.length || 0;
    if (unsafeFfiCountEl) unsafeFfiCountEl.textContent = ffiSnapshotData?.length || 0;

    // Populate generic types table with lifetime information
    populateGenericTypesTable(complexTypesData);
    
    // Populate memory allocations table with real data
    populateMemoryAllocationsTable(memoryAnalysisData);
    
    // Populate complex type analysis table
    populateComplexTypeAnalysisTable(complexTypesData);
    
    // Populate optimization recommendations
    populateOptimizationRecommendations(complexTypesData);
    
    // Create charts with real data
    createComplexityChart(complexTypesData);
    createMemoryDistributionChart(complexTypesData);
    createAllocationSizeChart(memoryAnalysisData);
    createPerformanceChart(performanceData);
    createComplexTypeAnalysisChart(complexTypesData);
    createFfiRiskChart(ffiSnapshotData);
    
    // Populate detailed performance metrics
    populatePerformanceMetrics(performanceData);
    
    // Initialize lifetime visualization
    initializeLifetimeVisualization(lifetimeData);
    
    // Render FFI data
    renderFfiData(ffiSnapshotData, document.getElementById('ffi-data-render'));
    
    // Initialize modern variable graph
    initModernVariableGraph(complexTypesData);
    
    // Create memory usage analysis visualization
    createMemoryUsageAnalysis(memoryAnalysisData, performanceData);
}

function populateGenericTypesTable(complexTypesData) {
    const tableBody = document.getElementById('generic-types-table-body');
    if (!tableBody) return;
    
    // Handle null or missing data
    if (!complexTypesData || !complexTypesData.categorized_types || !complexTypesData.categorized_types.generic_types) {
        tableBody.innerHTML = '<tr><td colspan="6" class="px-6 py-8 text-center text-gray-500">No complex types data available</td></tr>';
        return;
    }
    
    complexTypesData.categorized_types.generic_types.forEach(type => {
        let typeClass = '';
        if (type.type_name.includes('Vec')) typeClass = 'bg-blue-50';
        else if (type.type_name.includes('Arc')) typeClass = 'bg-green-50';
        else if (type.type_name.includes('Box')) typeClass = 'bg-purple-50';
        else if (type.type_name.includes('Rc')) typeClass = 'bg-yellow-50';

        const row = document.createElement('tr');
        row.className = `hover:bg-gray-50 transition-colors ${typeClass}`;
        row.innerHTML = `
            <td class="px-6 py-4 whitespace-nowrap">
                <div class="font-medium text-neutral">${type.var_name}</div>
            </td>
            <td class="px-6 py-4">
                <div class="text-sm text-gray-900 break-all max-w-xs">${type.type_name}</div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">${type.ptr}</td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">${type.size}</td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">${type.lifetime_ms}ms
                <div class="lifespan-indicator bg-gray-300 mt-1">
                    <div class="h-full" style="width: ${Math.min(100, type.lifetime_ms)}%; background-color: ${type.type_name.includes('Vec') ? '#3B82F6' :
                type.type_name.includes('Arc') ? '#10B981' :
                    type.type_name.includes('Box') ? '#8B5CF6' :
                        '#F59E0B'
            }"></div>
                </div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
                <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full 
                    ${type.complexity_score > 10 ? 'bg-red-100 text-red-800' :
                type.complexity_score > 5 ? 'bg-yellow-100 text-yellow-800' :
                    'bg-green-100 text-green-800'}">
                    ${type.complexity_score}
                </span>
            </td>
        `;
        tableBody.appendChild(row);
    });
}

function populateOptimizationRecommendations(complexTypesData) {
    const memoryRecList = document.getElementById('memory-optimization-recommendations');
    if (!memoryRecList) return;
    
    // Clear existing content
    memoryRecList.innerHTML = '';
    
    // Handle null or missing data
    if (!complexTypesData) {
        const li = document.createElement('li');
        li.className = 'text-gray-500 italic';
        li.textContent = 'No complex types data available';
        memoryRecList.appendChild(li);
        return;
    }
    
    // Collect optimization suggestions from complex_type_analysis
    const suggestions = [];
    
    // From complex_type_analysis if available
    if (complexTypesData.complex_type_analysis) {
        complexTypesData.complex_type_analysis.forEach(analysis => {
            if (analysis.optimization_suggestions && analysis.optimization_suggestions.length > 0) {
                analysis.optimization_suggestions.forEach(suggestion => {
                    suggestions.push(`${analysis.type_name}: ${suggestion}`);
                });
            }
        });
    }
    
    // From optimization_recommendations if available (fallback)
    if (complexTypesData.optimization_recommendations) {
        complexTypesData.optimization_recommendations.forEach(rec => {
            suggestions.push(rec);
        });
    }
    
    // Display suggestions or empty state
    if (suggestions.length === 0) {
        const li = document.createElement('li');
        li.className = 'text-gray-500 italic';
        li.textContent = 'No optimization recommendations available';
        memoryRecList.appendChild(li);
    } else {
        suggestions.forEach(suggestion => {
            const li = document.createElement('li');
            li.className = 'text-gray-700 flex items-start';
            li.innerHTML = `
                <i class="fa fa-lightbulb-o text-yellow-500 mr-2 mt-1 flex-shrink-0"></i>
                <span>${suggestion}</span>
            `;
            memoryRecList.appendChild(li);
        });
    }
}

function createComplexityChart(complexTypesData) {
    const complexityCtx = document.getElementById('complexity-chart');
    if (!complexityCtx) return;
    
    // Handle null or missing data
    if (!complexTypesData || !complexTypesData.summary || !complexTypesData.summary.complexity_distribution) {
        const ctx = complexityCtx.getContext('2d');
        ctx.fillStyle = '#9CA3AF';
        ctx.font = '14px Arial';
        ctx.textAlign = 'center';
        ctx.fillText('No complexity data available', complexityCtx.width / 2, complexityCtx.height / 2);
        return;
    }
    
    new Chart(complexityCtx.getContext('2d'), {
        type: 'bar',
        data: {
            labels: ['Low', 'Medium', 'High', 'Very High'],
            datasets: [{
                label: 'Number of Types',
                data: [
                    complexTypesData.summary.complexity_distribution.low_complexity || 0,
                    complexTypesData.summary.complexity_distribution.medium_complexity || 0,
                    complexTypesData.summary.complexity_distribution.high_complexity || 0,
                    complexTypesData.summary.complexity_distribution.very_high_complexity || 0
                ],
                backgroundColor: [
                    'rgba(16, 185, 129, 0.7)',
                    'rgba(59, 130, 246, 0.7)',
                    'rgba(245, 158, 11, 0.7)',
                    'rgba(239, 68, 68, 0.7)'
                ],
                borderColor: [
                    'rgb(16, 185, 129)',
                    'rgb(59, 130, 246)',
                    'rgb(245, 158, 11)',
                    'rgb(239, 68, 68)'
                ],
                borderWidth: 1
            }]
        },
        options: {
            responsive: true,
            scales: {
                y: {
                    beginAtZero: true,
                    ticks: {
                        precision: 0
                    }
                }
            }
        }
    });
}

function createMemoryDistributionChart(complexTypesData) {
    const memoryCtx = document.getElementById('memory-distribution-chart');
    if (!memoryCtx) return;
    
    // Handle null or missing data
    if (!complexTypesData || !complexTypesData.complex_type_analysis || complexTypesData.complex_type_analysis.length === 0) {
        const ctx = memoryCtx.getContext('2d');
        ctx.fillStyle = '#9CA3AF';
        ctx.font = '14px Arial';
        ctx.textAlign = 'center';
        ctx.fillText('No memory distribution data available', memoryCtx.width / 2, memoryCtx.height / 2);
        return;
    }
    
    new Chart(memoryCtx.getContext('2d'), {
        type: 'pie',
        data: {
            labels: complexTypesData.complex_type_analysis.map(a => a.type_name),
            datasets: [{
                data: complexTypesData.complex_type_analysis.map(a => a.total_size),
                backgroundColor: [
                    'rgba(245, 158, 11, 0.7)',  // Rc - yellow
                    'rgba(16, 185, 129, 0.7)',  // Arc - green
                    'rgba(139, 92, 246, 0.7)',  // Box - purple
                    'rgba(59, 130, 246, 0.7)',  // Vec - blue
                    'rgba(236, 72, 153, 0.7)'   // String - pink
                ],
                borderColor: [
                    'rgb(245, 158, 11)',
                    'rgb(16, 185, 129)',
                    'rgb(139, 92, 246)',
                    'rgb(59, 130, 246)',
                    'rgb(236, 72, 153)'
                ],
                borderWidth: 1
            }]
        },
        options: {
            responsive: true,
            plugins: {
                tooltip: {
                    callbacks: {
                        label: function (context) {
                            const value = context.raw;
                            const total = context.dataset.data.reduce((a, b) => a + b, 0);
                            const percentage = Math.round((value / total) * 100);
                            return `${context.label}: ${value} bytes (${percentage}%)`;
                        }
                    }
                }
            }
        }
    });
}

// FFI Data Rendering Functions
function renderFfiData(ffiData, container) {
    if (!container) return;
    
    // Handle empty or null data
    if (!ffiData) {
        container.innerHTML = '<div class="bg-gray-50 rounded-lg p-8 text-center text-gray-500 border-2 border-dashed border-gray-300"><i class="fa fa-info-circle text-2xl mb-2"></i><p>No unsafe/FFI data available</p></div>';
        return;
    }
    
    // Clear container
    container.innerHTML = '';
    
    // Create FFI summary section
    createFfiSummarySection(ffiData, container);
    
    // Create enhanced FFI data section if available
    if (ffiData.enhanced_ffi_data && ffiData.enhanced_ffi_data.length > 0) {
        createEnhancedFfiSection(ffiData.enhanced_ffi_data, container);
    }
    
    // Create boundary events section if available
    if (ffiData.boundary_events && ffiData.boundary_events.length > 0) {
        createBoundaryEventsSection(ffiData.boundary_events, container);
    }
    
    // Create safety violations section if available
    if (ffiData.safety_violations && ffiData.safety_violations.length > 0) {
        createSafetyViolationsSection(ffiData.safety_violations, container);
    }
    
    // If no detailed data, show summary only
    if ((!ffiData.enhanced_ffi_data || ffiData.enhanced_ffi_data.length === 0) &&
        (!ffiData.boundary_events || ffiData.boundary_events.length === 0) &&
        (!ffiData.safety_violations || ffiData.safety_violations.length === 0)) {
        
        const emptyDiv = document.createElement('div');
        emptyDiv.className = 'mt-6 bg-blue-50 rounded-lg p-6 border border-blue-200';
        emptyDiv.innerHTML = `
            <div class="flex items-center">
                <i class="fa fa-shield text-blue-500 text-xl mr-3"></i>
                <div>
                    <h4 class="font-semibold text-blue-800">Safe Code Detected</h4>
                    <p class="text-blue-600 text-sm mt-1">No unsafe operations or FFI calls detected in this analysis.</p>
                </div>
            </div>
        `;
        container.appendChild(emptyDiv);
    }
}

function createFfiSummarySection(ffiData, container) {
    const summary = ffiData.summary || {};
    
    const summaryDiv = document.createElement('div');
    summaryDiv.className = 'grid grid-cols-2 md:grid-cols-4 gap-4 mb-6';
    
    // Risk level color
    let riskColor = 'text-green-600 bg-green-100';
    if (summary.risk_assessment === 'high') riskColor = 'text-red-600 bg-red-100';
    else if (summary.risk_assessment === 'medium') riskColor = 'text-yellow-600 bg-yellow-100';
    
    summaryDiv.innerHTML = `
        <div class="bg-white rounded-lg p-4 border border-gray-200">
            <div class="text-2xl font-bold text-gray-900">${summary.unsafe_count || 0}</div>
            <div class="text-sm text-gray-600">Unsafe Operations</div>
        </div>
        <div class="bg-white rounded-lg p-4 border border-gray-200">
            <div class="text-2xl font-bold text-gray-900">${summary.ffi_count || 0}</div>
            <div class="text-sm text-gray-600">FFI Calls</div>
        </div>
        <div class="bg-white rounded-lg p-4 border border-gray-200">
            <div class="text-2xl font-bold text-gray-900">${summary.safety_violations || 0}</div>
            <div class="text-sm text-gray-600">Safety Violations</div>
        </div>
        <div class="bg-white rounded-lg p-4 border border-gray-200">
            <div class="text-sm text-gray-600 mb-1">Risk Level</div>
            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${riskColor}">
                ${(summary.risk_assessment || 'low').toUpperCase()}
            </span>
        </div>
    `;
    
    container.appendChild(summaryDiv);
}

function createEnhancedFfiSection(enhancedFfiData, container) {
    const sectionDiv = document.createElement('div');
    sectionDiv.className = 'mb-6';
    
    const headerDiv = document.createElement('div');
    headerDiv.className = 'mb-4';
    headerDiv.innerHTML = `
        <h4 class="text-lg font-semibold text-gray-800 flex items-center">
            <i class="fa fa-exclamation-triangle text-orange-500 mr-2"></i>
            Enhanced FFI Analysis (${enhancedFfiData.length} entries)
        </h4>
    `;
    sectionDiv.appendChild(headerDiv);
    
    enhancedFfiData.forEach((item, index) => {
        const entryDiv = document.createElement('div');
        entryDiv.className = 'bg-white rounded-lg card-shadow hover-lift border-l-4 ' +
            (item.source.FfiC ? 'border-blue-500' : 'border-yellow-500');

        const sourceType = item.source.FfiC ? 'FFI (C)' : 'Unsafe Rust';
        const sourceColor = item.source.FfiC ? 'text-blue-500' : 'text-yellow-500';

        // Create header with toggle button
        const headerDiv = document.createElement('div');
        headerDiv.className = 'p-5 cursor-pointer flex justify-between items-center';
        headerDiv.innerHTML = `
            <div class="font-semibold text-lg ${sourceColor}">Entry ${index + 1}: ${sourceType}</div>
            <i class="fa fa-chevron-down rotate-icon"></i>
        `;
        entryDiv.appendChild(headerDiv);

        // Create collapsible content
        const contentDiv = document.createElement('div');
        contentDiv.className = 'collapsible-content px-5 pb-5 border-t border-gray-100';
        entryDiv.appendChild(contentDiv);

        // Render the data inside collapsible content
        renderFfiObject(item, contentDiv);
        container.appendChild(entryDiv);

        // Add click handler for toggle
        headerDiv.addEventListener('click', () => {
            contentDiv.classList.toggle('active');
            headerDiv.querySelector('.rotate-icon').classList.toggle('active');
        });
    });
}

function renderFfiObject(obj, container, level = 0) {
    // Limit nesting depth to prevent excessive expansion
    if (level > 3) {
        const moreDiv = document.createElement('div');
        moreDiv.className = 'text-gray-500 text-sm italic mt-1';
        moreDiv.textContent = '(Content truncated for readability)';
        container.appendChild(moreDiv);
        return;
    }

    for (const [key, value] of Object.entries(obj)) {
        // Skip null values and empty objects
        if (value === null || (typeof value === 'object' && Object.keys(value).length === 0)) {
            continue;
        }

        const sectionDiv = document.createElement('div');
        sectionDiv.className = 'mb-4';

        const keyDiv = document.createElement('div');
        keyDiv.className = 'font-medium text-neutral mb-2';
        keyDiv.textContent = key.charAt(0).toUpperCase() + key.slice(1);
        sectionDiv.appendChild(keyDiv);

        const valueDiv = document.createElement('div');
        valueDiv.className = 'pl-4 border-l-2 border-gray-200';

        if (typeof value === 'object') {
            if (Array.isArray(value)) {
                // For arrays, show count and allow expansion
                const arrayHeader = document.createElement('div');
                arrayHeader.className = 'font-medium text-sm text-gray-600 cursor-pointer flex items-center';
                arrayHeader.innerHTML = `
                    Array (${value.length} items)
                    <i class="fa fa-chevron-down rotate-icon ml-2 text-xs"></i>
                `;
                valueDiv.appendChild(arrayHeader);

                const arrayContent = document.createElement('div');
                arrayContent.className = 'collapsible-content mt-2';
                valueDiv.appendChild(arrayContent);

                value.forEach((item, idx) => {
                    const arrayItemDiv = document.createElement('div');
                    arrayItemDiv.className = 'mb-3';
                    arrayItemDiv.innerHTML = `<div class="font-medium text-sm text-gray-600">Item ${idx + 1}:</div>`;
                    renderFfiObject(item, arrayItemDiv, level + 1);
                    arrayContent.appendChild(arrayItemDiv);
                });

                // Add click handler for array toggle
                arrayHeader.addEventListener('click', () => {
                    arrayContent.classList.toggle('active');
                    arrayHeader.querySelector('.rotate-icon').classList.toggle('active');
                });
            } else {
                // For objects, create a toggleable section
                const objectHeader = document.createElement('div');
                objectHeader.className = 'font-medium text-sm text-gray-600 cursor-pointer flex items-center';
                objectHeader.innerHTML = `
                    Object
                    <i class="fa fa-chevron-down rotate-icon ml-2 text-xs"></i>
                `;
                valueDiv.appendChild(objectHeader);

                const objectContent = document.createElement('div');
                objectContent.className = 'collapsible-content mt-2';
                valueDiv.appendChild(objectContent);

                renderFfiObject(value, objectContent, level + 1);

                // Add click handler for object toggle
                objectHeader.addEventListener('click', () => {
                    objectContent.classList.toggle('active');
                    objectHeader.querySelector('.rotate-icon').classList.toggle('active');
                });
            }
        } else {
            // For primitive values, just display them
            valueDiv.textContent = value;
        }

        sectionDiv.appendChild(valueDiv);
        container.appendChild(sectionDiv);
    }
}

// Modern Variable Relationship Graph using D3.js
function initModernVariableGraph(complexTypesData) {
    const container = d3.select("#variable-graph-container");
    if (container.empty()) return;
    
    const width = container.node().getBoundingClientRect().width;
    const height = container.node().getBoundingClientRect().height;

    // Clear any existing content
    container.selectAll("*").remove();

    // Create SVG
    const svg = container.append("svg")
        .attr("width", width)
        .attr("height", height)
        .style("background", "linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%)");

    // Create zoom behavior
    const zoom = d3.zoom()
        .scaleExtent([0.5, 3])
        .on("zoom", (event) => {
            g.attr("transform", event.transform);
        });

    svg.call(zoom);

    const g = svg.append("g");

    // Get data from complexTypesData
    const nodes = [];
    const links = [];

    if (complexTypesData && complexTypesData.categorized_types && complexTypesData.categorized_types.generic_types) {
        complexTypesData.categorized_types.generic_types.forEach((type) => {
            const category = getNodeCategory(type.type_name);
            nodes.push({
                id: type.var_name,
                name: type.var_name,
                type: type.type_name,
                size: type.size || 20,
                complexity: type.complexity_score || 1,
                lifetime: type.lifetime_ms || 0,
                ptr: type.ptr,
                category: category
            });
        });

        // Create links based on type relationships
        for (let i = 0; i < nodes.length; i++) {
            for (let j = i + 1; j < nodes.length; j++) {
                const node1 = nodes[i];
                const node2 = nodes[j];
                
                // Create links for similar types or clones
                if (node1.type === node2.type) {
                    links.push({
                        source: node1.id,
                        target: node2.id,
                        type: 'clone',
                        strength: 0.8
                    });
                } else if (getNodeCategory(node1.type) === getNodeCategory(node2.type)) {
                    links.push({
                        source: node1.id,
                        target: node2.id,
                        type: 'similar',
                        strength: 0.3
                    });
                }
            }
        }
    }

    // Create force simulation
    const simulation = d3.forceSimulation(nodes)
        .force("link", d3.forceLink(links).id(d => d.id).strength(d => d.strength))
        .force("charge", d3.forceManyBody().strength(-300))
        .force("center", d3.forceCenter(width / 2, height / 2))
        .force("collision", d3.forceCollide().radius(d => Math.sqrt(d.complexity) * 8 + 10));

    // Create links
    const link = g.append("g")
        .selectAll("line")
        .data(links)
        .enter().append("line")
        .attr("stroke", d => d.type === 'clone' ? '#666' : '#999')
        .attr("stroke-opacity", 0.6)
        .attr("stroke-width", d => d.type === 'clone' ? 2 : 1)
        .attr("stroke-dasharray", d => d.type === 'similar' ? "5,5" : null);

    // Create nodes
    const node = g.append("g")
        .selectAll("g")
        .data(nodes)
        .enter().append("g")
        .attr("class", "node")
        .call(d3.drag()
            .on("start", dragstarted)
            .on("drag", dragged)
            .on("end", dragended));

    // Add circles to nodes
    node.append("circle")
        .attr("r", d => Math.sqrt(d.complexity) * 4 + 8)
        .attr("fill", d => getNodeColor(d.category))
        .attr("stroke", "#fff")
        .attr("stroke-width", 2);

    // Add labels to nodes
    node.append("text")
        .attr("dy", ".35em")
        .attr("text-anchor", "middle")
        .style("font-size", "12px")
        .style("font-weight", "bold")
        .style("fill", "#333")
        .text(d => d.name);

    // Add tooltips
    node.append("title")
        .text(d => `${d.name}\nType: ${d.type}\nSize: ${d.size} bytes\nComplexity: ${d.complexity}\nLifetime: ${d.lifetime}ms`);

    // Add click event to show node details
    node.on("click", function(event, d) {
        event.stopPropagation();
        showNodeDetails(d);
    });

    // Update positions on simulation tick
    simulation.on("tick", () => {
        link
            .attr("x1", d => d.source.x)
            .attr("y1", d => d.source.y)
            .attr("x2", d => d.target.x)
            .attr("y2", d => d.target.y);

        node
            .attr("transform", d => `translate(${d.x},${d.y})`);
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

    // Add reset zoom button functionality
    const resetZoomBtn = document.getElementById('reset-zoom');
    if (resetZoomBtn) {
        resetZoomBtn.addEventListener('click', () => {
            svg.transition().duration(750).call(
                zoom.transform,
                d3.zoomIdentity
            );
        });
    }

    // Add auto layout button functionality
    const autoLayoutBtn = document.getElementById('auto-layout');
    if (autoLayoutBtn) {
        autoLayoutBtn.addEventListener('click', () => {
            simulation.alpha(1).restart();
        });
    }
}

function getNodeCategory(typeName) {
    if (typeName.includes('Vec') || typeName.includes('HashMap') || typeName.includes('BTreeMap')) {
        return 'collections';
    } else if (typeName.includes('Box') || typeName.includes('Rc') || typeName.includes('Arc')) {
        return 'smart_pointers';
    } else {
        return 'primitives';
    }
}

function getNodeColor(category) {
    const colors = {
        'smart_pointers': '#3B82F6',  // Blue
        'collections': '#10B981',     // Green
        'primitives': '#F59E0B',      // Orange
        'default': '#6B7280'          // Gray
    };
    return colors[category] || colors.default;
}

// Show node details in the side panel
function showNodeDetails(nodeData) {
    const detailsPanel = document.getElementById('node-details');
    const detailsContent = document.getElementById('node-details-content');
    
    if (!detailsPanel || !detailsContent) return;
    
    // Format the details
    const complexityLevel = nodeData.complexity > 10 ? 'High' : 
                           nodeData.complexity > 5 ? 'Medium' : 'Low';
    const complexityColor = nodeData.complexity > 10 ? 'text-red-600' : 
                           nodeData.complexity > 5 ? 'text-yellow-600' : 'text-green-600';
    
    detailsContent.innerHTML = `
        <div class="space-y-3">
            <div>
                <label class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Variable Name</label>
                <p class="text-sm font-medium text-gray-900">${nodeData.name}</p>
            </div>
            
            <div>
                <label class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Type</label>
                <p class="text-sm text-gray-700 break-all">${nodeData.type}</p>
            </div>
            
            <div class="grid grid-cols-2 gap-3">
                <div>
                    <label class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Size</label>
                    <p class="text-sm font-medium text-gray-900">${nodeData.size} bytes</p>
                </div>
                <div>
                    <label class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Lifetime</label>
                    <p class="text-sm font-medium text-gray-900">${nodeData.lifetime}ms</p>
                </div>
            </div>
            
            <div>
                <label class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Complexity</label>
                <div class="flex items-center space-x-2">
                    <span class="text-sm font-bold ${complexityColor}">${nodeData.complexity}</span>
                    <span class="px-2 py-1 text-xs font-semibold rounded-full ${
                        nodeData.complexity > 10 ? 'bg-red-100 text-red-800' :
                        nodeData.complexity > 5 ? 'bg-yellow-100 text-yellow-800' : 'bg-green-100 text-green-800'
                    }">${complexityLevel}</span>
                </div>
            </div>
            
            <div>
                <label class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Category</label>
                <div class="flex items-center space-x-2">
                    <div class="w-3 h-3 rounded-full" style="background-color: ${getNodeColor(nodeData.category)}"></div>
                    <span class="text-sm capitalize text-gray-700">${nodeData.category.replace('_', ' ')}</span>
                </div>
            </div>
            
            <div>
                <label class="text-xs font-semibold text-gray-500 uppercase tracking-wide">Memory Address</label>
                <p class="text-xs font-mono text-gray-600">${nodeData.ptr || 'N/A'}</p>
            </div>
        </div>
    `;
    
    // Show the panel
    detailsPanel.classList.remove('hidden');
}

// Hide node details panel
function hideNodeDetails() {
    const detailsPanel = document.getElementById('node-details');
    if (detailsPanel) {
        detailsPanel.classList.add('hidden');
    }
}

// Add event listener for close button
document.addEventListener('DOMContentLoaded', () => {
    const closeBtn = document.getElementById('close-details');
    if (closeBtn) {
        closeBtn.addEventListener('click', hideNodeDetails);
    }
    
    // Hide details when clicking outside the graph
    document.addEventListener('click', (event) => {
        const graphContainer = document.getElementById('variable-graph-container');
        const detailsPanel = document.getElementById('node-details');
        
        if (graphContainer && detailsPanel && 
            !graphContainer.contains(event.target) && 
            !detailsPanel.contains(event.target)) {
            hideNodeDetails();
        }
    });
});

// Initialize dashboard when DOM is loaded
document.addEventListener("DOMContentLoaded", () => {
    console.log('MemScope dashboard loaded');
    initializeDashboard();
});
// Performance metrics chart using real data
function createPerformanceChart(performanceData) {
    const ctx = document.getElementById('performance-chart');
    if (!ctx) return;
    
    // Handle null or missing data
    if (!performanceData || !performanceData.memory_performance) {
        const context = ctx.getContext('2d');
        context.fillStyle = '#9CA3AF';
        context.font = '14px Arial';
        context.textAlign = 'center';
        context.fillText('No performance data available', ctx.width / 2, ctx.height / 2);
        return;
    }
    
    const memPerf = performanceData.memory_performance;
    
    new Chart(ctx.getContext('2d'), {
        type: 'bar',
        data: {
            labels: ['Active Memory', 'Peak Memory', 'Total Allocated'],
            datasets: [{
                label: 'Memory Usage (bytes)',
                data: [
                    memPerf.active_memory || 0,
                    memPerf.peak_memory || 0,
                    memPerf.total_allocated || 0
                ],
                backgroundColor: [
                    'rgba(34, 197, 94, 0.7)',   // Green for active
                    'rgba(239, 68, 68, 0.7)',   // Red for peak
                    'rgba(59, 130, 246, 0.7)'   // Blue for total
                ],
                borderColor: [
                    'rgb(34, 197, 94)',
                    'rgb(239, 68, 68)',
                    'rgb(59, 130, 246)'
                ],
                borderWidth: 1
            }]
        },
        options: {
            responsive: true,
            scales: {
                y: {
                    beginAtZero: true,
                    ticks: {
                        callback: function(value) {
                            // Format bytes to human readable
                            if (value >= 1024 * 1024) {
                                return (value / (1024 * 1024)).toFixed(1) + 'MB';
                            } else if (value >= 1024) {
                                return (value / 1024).toFixed(1) + 'KB';
                            }
                            return value + 'B';
                        }
                    }
                }
            },
            plugins: {
                title: {
                    display: true,
                    text: 'Memory Performance Metrics'
                }
            }
        }
    });
}// Lifetime visualization using real lifecycle_events data
function initializeLifetimeVisualization(lifetimeData) {
    const container = document.getElementById('lifetimeVisualization');
    if (!container) return;
    
    // Handle null or missing data
    if (!lifetimeData || !lifetimeData.lifecycle_events || lifetimeData.lifecycle_events.length === 0) {
        container.innerHTML = '<div class="text-center py-8 text-gray-500"><i class="fa fa-info-circle text-2xl mb-2"></i><p>No lifetime data available</p></div>';
        return;
    }
    
    // Filter out "unknown" entries as specified in the task
    const validEvents = lifetimeData.lifecycle_events.filter(event => 
        event.type_name !== "unknown" && event.var_name !== "unknown"
    );
    
    if (validEvents.length === 0) {
        container.innerHTML = '<div class="text-center py-8 text-gray-500"><i class="fa fa-info-circle text-2xl mb-2"></i><p>No valid lifetime data available (filtered out unknown entries)</p></div>';
        return;
    }
    
    // Group events by variable
    const variableLifetimes = {};
    validEvents.forEach(event => {
        const key = `${event.var_name}_${event.ptr}`;
        if (!variableLifetimes[key]) {
            variableLifetimes[key] = {
                var_name: event.var_name,
                type_name: event.type_name,
                ptr: event.ptr,
                size: event.size,
                allocation_time: null,
                deallocation_time: null,
                scope: event.scope
            };
        }
        
        if (event.event === 'allocation') {
            variableLifetimes[key].allocation_time = event.timestamp;
        } else if (event.event === 'deallocation') {
            variableLifetimes[key].deallocation_time = event.timestamp;
        }
    });
    
    // Convert to array and calculate lifetimes
    const lifetimes = Object.values(variableLifetimes).map(item => {
        const allocTime = item.allocation_time;
        const deallocTime = item.deallocation_time;
        const lifetime_ms = deallocTime ? (deallocTime - allocTime) / 1000000 : null; // Convert nanoseconds to milliseconds
        
        return {
            ...item,
            lifetime_ms,
            is_active: !deallocTime
        };
    });
    
    // Sort by allocation time
    lifetimes.sort((a, b) => (a.allocation_time || 0) - (b.allocation_time || 0));
    
    // Find time range
    const minTime = Math.min(...lifetimes.map(l => l.allocation_time || 0));
    const maxTime = Math.max(...lifetimes.map(l => l.deallocation_time || l.allocation_time || 0));
    const timeRange = maxTime - minTime;
    
    // Clear container and create timeline
    container.innerHTML = '';
    
    // Create timeline header
    const headerDiv = document.createElement('div');
    headerDiv.className = 'mb-4';
    headerDiv.innerHTML = `
        <h4 class="font-semibold text-gray-800 mb-2">Variable Lifetimes (${lifetimes.length} variables)</h4>
        <div class="text-sm text-gray-600">Timeline shows allocation and deallocation events over time</div>
    `;
    container.appendChild(headerDiv);
    
    // Create timeline container
    const timelineDiv = document.createElement('div');
    timelineDiv.className = 'space-y-2';
    
    lifetimes.slice(0, 20).forEach((lifetime, index) => { // Show first 20 to avoid overwhelming UI
        const row = document.createElement('div');
        row.className = 'flex items-center space-x-4 p-2 hover:bg-gray-50 rounded';
        
        // Variable info
        const infoDiv = document.createElement('div');
        infoDiv.className = 'w-40 flex-shrink-0';
        infoDiv.innerHTML = `
            <div class="font-medium text-sm">${lifetime.var_name}</div>
            <div class="text-xs text-gray-500">${lifetime.type_name}</div>
        `;
        
        // Timeline bar
        const timelineBarDiv = document.createElement('div');
        timelineBarDiv.className = 'flex-grow relative h-6 bg-gray-100 rounded';
        
        if (lifetime.allocation_time) {
            const startPercent = ((lifetime.allocation_time - minTime) / timeRange) * 100;
            const endPercent = lifetime.deallocation_time ? 
                ((lifetime.deallocation_time - minTime) / timeRange) * 100 : 100;
            
            const bar = document.createElement('div');
            bar.className = `absolute h-full rounded ${lifetime.is_active ? 'bg-green-500' : 'bg-blue-500'}`;
            bar.style.left = `${startPercent}%`;
            bar.style.width = `${endPercent - startPercent}%`;
            bar.title = `${lifetime.var_name}: ${lifetime.lifetime_ms ? lifetime.lifetime_ms.toFixed(2) + 'ms' : 'Active'}`;
            
            timelineBarDiv.appendChild(bar);
        }
        
        // Lifetime info
        const lifetimeInfoDiv = document.createElement('div');
        lifetimeInfoDiv.className = 'w-24 flex-shrink-0 text-right text-sm';
        lifetimeInfoDiv.innerHTML = lifetime.is_active ? 
            '<span class="text-green-600">Active</span>' : 
            `<span class="text-gray-600">${lifetime.lifetime_ms?.toFixed(2) || 0}ms</span>`;
        
        row.appendChild(infoDiv);
        row.appendChild(timelineBarDiv);
        row.appendChild(lifetimeInfoDiv);
        
        timelineDiv.appendChild(row);
    });
    
    container.appendChild(timelineDiv);
    
    // Add summary if there are more items
    if (lifetimes.length > 20) {
        const summaryDiv = document.createElement('div');
        summaryDiv.className = 'mt-4 text-center text-gray-500 text-sm';
        summaryDiv.textContent = `Showing 20 of ${lifetimes.length} total variables`;
        container.appendChild(summaryDiv);
    }
}

// Populate detailed performance metrics using real data
function populatePerformanceMetrics(performanceData) {
    // Update smart pointers count
    const smartPointersEl = document.getElementById('smart-pointers-count');
    const collectionsEl = document.getElementById('collections-count');
    const primitivesEl = document.getElementById('primitives-count');
    
    if (performanceData && performanceData.allocation_distribution) {
        const dist = performanceData.allocation_distribution;
        const totalAllocations = Object.values(dist).reduce((sum, count) => sum + count, 0);
        
        // Use allocation distribution as proxy for different categories
        if (smartPointersEl) smartPointersEl.textContent = dist.large || 0;
        if (collectionsEl) collectionsEl.textContent = dist.medium || 0;
        if (primitivesEl) primitivesEl.textContent = dist.small || 0;
    }
    
    // Create allocation distribution chart if we have the data
    if (performanceData && performanceData.allocation_distribution) {
        createAllocationDistributionChart(performanceData.allocation_distribution);
    }
    
    // Display export performance metrics
    if (performanceData && performanceData.export_performance) {
        displayExportPerformanceMetrics(performanceData.export_performance);
    }
}

// Create allocation distribution chart
function createAllocationDistributionChart(allocationDistribution) {
    const ctx = document.getElementById('allocation-distribution-chart');
    if (!ctx) return;
    
    const labels = Object.keys(allocationDistribution).map(key => 
        key.charAt(0).toUpperCase() + key.slice(1)
    );
    const data = Object.values(allocationDistribution);
    
    new Chart(ctx.getContext('2d'), {
        type: 'bar',
        data: {
            labels: labels,
            datasets: [{
                label: 'Number of Allocations',
                data: data,
                backgroundColor: [
                    'rgba(34, 197, 94, 0.7)',   // Green
                    'rgba(59, 130, 246, 0.7)',  // Blue
                    'rgba(245, 158, 11, 0.7)',  // Yellow
                    'rgba(239, 68, 68, 0.7)',   // Red
                    'rgba(147, 51, 234, 0.7)'   // Purple
                ],
                borderColor: [
                    'rgb(34, 197, 94)',
                    'rgb(59, 130, 246)',
                    'rgb(245, 158, 11)',
                    'rgb(239, 68, 68)',
                    'rgb(147, 51, 234)'
                ],
                borderWidth: 1
            }]
        },
        options: {
            responsive: true,
            scales: {
                y: {
                    beginAtZero: true,
                    ticks: {
                        precision: 0
                    }
                }
            },
            plugins: {
                title: {
                    display: true,
                    text: 'Allocation Size Distribution'
                }
            }
        }
    });
}

// Display export performance metrics
function displayExportPerformanceMetrics(exportPerformance) {
    const container = document.getElementById('export-performance-metrics');
    if (!container) return;
    
    const processingRate = exportPerformance.processing_rate;
    const performanceClass = processingRate.performance_class;
    
    // Determine color based on performance class
    let statusColor = 'text-gray-600';
    let statusBg = 'bg-gray-100';
    if (performanceClass === 'excellent') {
        statusColor = 'text-green-600';
        statusBg = 'bg-green-100';
    } else if (performanceClass === 'good') {
        statusColor = 'text-blue-600';
        statusBg = 'bg-blue-100';
    } else if (performanceClass === 'needs_optimization') {
        statusColor = 'text-yellow-600';
        statusBg = 'bg-yellow-100';
    } else if (performanceClass === 'poor') {
        statusColor = 'text-red-600';
        statusBg = 'bg-red-100';
    }
    
    container.innerHTML = `
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div class="bg-white rounded-lg p-4 border border-gray-200">
                <div class="text-sm text-gray-500">Allocations Processed</div>
                <div class="text-2xl font-bold text-gray-900">${exportPerformance.allocations_processed.toLocaleString()}</div>
            </div>
            <div class="bg-white rounded-lg p-4 border border-gray-200">
                <div class="text-sm text-gray-500">Processing Rate</div>
                <div class="text-2xl font-bold text-gray-900">${processingRate.allocations_per_second.toFixed(1)}/s</div>
            </div>
            <div class="bg-white rounded-lg p-4 border border-gray-200">
                <div class="text-sm text-gray-500">Performance Status</div>
                <div class="mt-1">
                    <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${statusBg} ${statusColor}">
                        ${performanceClass.replace('_', ' ').toUpperCase()}
                    </span>
                </div>
            </div>
        </div>
        <div class="mt-4 text-sm text-gray-600">
            Total processing time: ${exportPerformance.total_processing_time_ms.toLocaleString()}ms
        </div>
    `;
}//Complex type analysis chart using real complex_type_analysis data
function createComplexTypeAnalysisChart(complexTypesData) {
    const ctx = document.getElementById('complex-type-analysis-chart');
    if (!ctx) return;
    
    // Handle null or missing data
    if (!complexTypesData || !complexTypesData.complex_type_analysis || complexTypesData.complex_type_analysis.length === 0) {
        const context = ctx.getContext('2d');
        context.fillStyle = '#9CA3AF';
        context.font = '14px Arial';
        context.textAlign = 'center';
        context.fillText('No complex type analysis data available', ctx.width / 2, ctx.height / 2);
        return;
    }
    
    const analysisData = complexTypesData.complex_type_analysis;
    
    // Create a scatter plot showing complexity vs memory efficiency
    new Chart(ctx.getContext('2d'), {
        type: 'scatter',
        data: {
            datasets: [{
                label: 'Type Complexity vs Memory Efficiency',
                data: analysisData.map(item => ({
                    x: item.complexity_score,
                    y: item.memory_efficiency,
                    label: item.type_name,
                    allocation_count: item.allocation_count,
                    total_size: item.total_size
                })),
                backgroundColor: analysisData.map(item => {
                    // Color based on complexity score
                    if (item.complexity_score >= 12) return 'rgba(239, 68, 68, 0.7)';   // Red for high complexity
                    if (item.complexity_score >= 8) return 'rgba(245, 158, 11, 0.7)';   // Yellow for medium
                    return 'rgba(34, 197, 94, 0.7)';  // Green for low complexity
                }),
                borderColor: analysisData.map(item => {
                    if (item.complexity_score >= 12) return 'rgb(239, 68, 68)';
                    if (item.complexity_score >= 8) return 'rgb(245, 158, 11)';
                    return 'rgb(34, 197, 94)';
                }),
                borderWidth: 2,
                pointRadius: analysisData.map(item => Math.max(5, Math.min(15, item.allocation_count * 3)))
            }]
        },
        options: {
            responsive: true,
            plugins: {
                title: {
                    display: true,
                    text: 'Type Complexity vs Memory Efficiency'
                },
                tooltip: {
                    callbacks: {
                        label: function(context) {
                            const point = context.raw;
                            return [
                                `Type: ${point.label}`,
                                `Complexity: ${point.x}`,
                                `Memory Efficiency: ${point.y}%`,
                                `Allocations: ${point.allocation_count}`,
                                `Total Size: ${point.total_size} bytes`
                            ];
                        }
                    }
                }
            },
            scales: {
                x: {
                    title: {
                        display: true,
                        text: 'Complexity Score'
                    },
                    beginAtZero: true
                },
                y: {
                    title: {
                        display: true,
                        text: 'Memory Efficiency (%)'
                    },
                    beginAtZero: true,
                    max: 100
                }
            }
        }
    });
}

// Enhanced complex types table with analysis data
function populateComplexTypeAnalysisTable(complexTypesData) {
    const tableBody = document.getElementById('complex-type-analysis-table');
    if (!tableBody) return;
    
    // Handle null or missing data
    if (!complexTypesData || !complexTypesData.complex_type_analysis || complexTypesData.complex_type_analysis.length === 0) {
        tableBody.innerHTML = '<tr><td colspan="6" class="px-6 py-8 text-center text-gray-500">No complex type analysis data available</td></tr>';
        return;
    }
    
    // Clear existing content
    tableBody.innerHTML = '';
    
    complexTypesData.complex_type_analysis.forEach(analysis => {
        const row = document.createElement('tr');
        row.className = 'hover:bg-gray-50 transition-colors';
        
        // Determine complexity level and color
        let complexityClass = 'text-green-600';
        let complexityLabel = 'Low';
        if (analysis.complexity_score >= 12) {
            complexityClass = 'text-red-600';
            complexityLabel = 'High';
        } else if (analysis.complexity_score >= 8) {
            complexityClass = 'text-yellow-600';
            complexityLabel = 'Medium';
        }
        
        // Determine efficiency level and color
        let efficiencyClass = 'text-red-600';
        if (analysis.memory_efficiency >= 80) {
            efficiencyClass = 'text-green-600';
        } else if (analysis.memory_efficiency >= 60) {
            efficiencyClass = 'text-yellow-600';
        }
        
        row.innerHTML = `
            <td class="px-6 py-4 whitespace-nowrap">
                <div class="font-medium text-gray-900">${analysis.type_name}</div>
                <div class="text-sm text-gray-500">${analysis.category}</div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-center">
                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${complexityClass.replace('text-', 'bg-').replace('-600', '-100')} ${complexityClass}">
                    ${analysis.complexity_score} (${complexityLabel})
                </span>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-center">
                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${efficiencyClass.replace('text-', 'bg-').replace('-600', '-100')} ${efficiencyClass}">
                    ${analysis.memory_efficiency}%
                </span>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-center text-sm text-gray-900">
                ${analysis.allocation_count}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-center text-sm text-gray-900">
                ${analysis.total_size} bytes
            </td>
            <td class="px-6 py-4 text-sm text-gray-500">
                ${analysis.optimization_suggestions && analysis.optimization_suggestions.length > 0 
                    ? analysis.optimization_suggestions.join('; ') 
                    : 'No suggestions'}
            </td>
        `;
        
        tableBody.appendChild(row);
    });
}
function createBoundaryEventsSection(boundaryEvents, container) {
    const sectionDiv = document.createElement('div');
    sectionDiv.className = 'mb-6';
    
    const headerDiv = document.createElement('div');
    headerDiv.className = 'mb-4';
    headerDiv.innerHTML = `
        <h4 class="text-lg font-semibold text-gray-800 flex items-center">
            <i class="fa fa-exchange text-blue-500 mr-2"></i>
            Boundary Events (${boundaryEvents.length} events)
        </h4>
    `;
    sectionDiv.appendChild(headerDiv);
    
    const eventsDiv = document.createElement('div');
    eventsDiv.className = 'space-y-3';
    
    boundaryEvents.forEach((event, index) => {
        const eventDiv = document.createElement('div');
        eventDiv.className = 'bg-blue-50 rounded-lg p-4 border border-blue-200';
        
        eventDiv.innerHTML = `
            <div class="flex justify-between items-start">
                <div>
                    <div class="font-medium text-blue-800">Event ${index + 1}: ${event.event_type || 'Unknown'}</div>
                    <div class="text-sm text-blue-600 mt-1">
                        From: ${event.from_context || 'Unknown'} → To: ${event.to_context || 'Unknown'}
                    </div>
                    ${event.timestamp ? `<div class="text-xs text-blue-500 mt-1">Timestamp: ${new Date(event.timestamp / 1000000).toLocaleString()}</div>` : ''}
                </div>
                <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                    Boundary
                </span>
            </div>
        `;
        
        eventsDiv.appendChild(eventDiv);
    });
    
    sectionDiv.appendChild(eventsDiv);
    container.appendChild(sectionDiv);
}

function createSafetyViolationsSection(safetyViolations, container) {
    const sectionDiv = document.createElement('div');
    sectionDiv.className = 'mb-6';
    
    const headerDiv = document.createElement('div');
    headerDiv.className = 'mb-4';
    headerDiv.innerHTML = `
        <h4 class="text-lg font-semibold text-gray-800 flex items-center">
            <i class="fa fa-warning text-red-500 mr-2"></i>
            Safety Violations (${safetyViolations.length} violations)
        </h4>
    `;
    sectionDiv.appendChild(headerDiv);
    
    const violationsDiv = document.createElement('div');
    violationsDiv.className = 'space-y-3';
    
    safetyViolations.forEach((violation, index) => {
        const violationDiv = document.createElement('div');
        violationDiv.className = 'bg-red-50 rounded-lg p-4 border border-red-200';
        
        // Determine severity color
        let severityColor = 'bg-yellow-100 text-yellow-800';
        if (violation.severity === 'high') severityColor = 'bg-red-100 text-red-800';
        else if (violation.severity === 'low') severityColor = 'bg-green-100 text-green-800';
        
        violationDiv.innerHTML = `
            <div class="flex justify-between items-start">
                <div class="flex-grow">
                    <div class="font-medium text-red-800">Violation ${index + 1}</div>
                    <div class="text-sm text-red-600 mt-1">${violation.description || 'No description available'}</div>
                    ${violation.location ? `<div class="text-xs text-red-500 mt-1">Location: ${violation.location}</div>` : ''}
                </div>
                <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${severityColor}">
                    ${(violation.severity || 'medium').toUpperCase()}
                </span>
            </div>
        `;
        
        violationsDiv.appendChild(violationDiv);
    });
    
    sectionDiv.appendChild(violationsDiv);
    container.appendChild(sectionDiv);
}

// Create FFI risk assessment chart
function createFfiRiskChart(ffiData) {
    const ctx = document.getElementById('ffi-risk-chart');
    if (!ctx) return;
    
    if (!ffiData || !ffiData.summary) {
        const context = ctx.getContext('2d');
        context.fillStyle = '#9CA3AF';
        context.font = '14px Arial';
        context.textAlign = 'center';
        context.fillText('No FFI risk data available', ctx.width / 2, ctx.height / 2);
        return;
    }
    
    const summary = ffiData.summary;
    
    // Create a risk breakdown chart
    const riskData = {
        'Safe Operations': (summary.total_risk_items || 0) === 0 ? 1 : 0,
        'Unsafe Operations': summary.unsafe_count || 0,
        'FFI Calls': summary.ffi_count || 0,
        'Safety Violations': summary.safety_violations || 0
    };
    
    // Filter out zero values for better visualization
    const filteredData = Object.entries(riskData).filter(([key, value]) => value > 0);
    
    if (filteredData.length === 0) {
        filteredData.push(['Safe Operations', 1]);
    }
    
    new Chart(ctx.getContext('2d'), {
        type: 'doughnut',
        data: {
            labels: filteredData.map(([key]) => key),
            datasets: [{
                data: filteredData.map(([, value]) => value),
                backgroundColor: [
                    'rgba(34, 197, 94, 0.7)',   // Green for safe
                    'rgba(245, 158, 11, 0.7)',  // Yellow for unsafe
                    'rgba(59, 130, 246, 0.7)',  // Blue for FFI
                    'rgba(239, 68, 68, 0.7)'    // Red for violations
                ],
                borderColor: [
                    'rgb(34, 197, 94)',
                    'rgb(245, 158, 11)',
                    'rgb(59, 130, 246)',
                    'rgb(239, 68, 68)'
                ],
                borderWidth: 2
            }]
        },
        options: {
            responsive: true,
            plugins: {
                title: {
                    display: true,
                    text: 'FFI Risk Assessment'
                },
                legend: {
                    position: 'bottom'
                }
            }
        }
    });
}