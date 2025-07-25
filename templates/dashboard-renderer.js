/**
 * MemScope Dashboard Renderer
 * Renders dashboard components with real JSON data
 */

class MemScopeDashboardRenderer {
    constructor() {
        this.loader = window.memScopeDashboardLoader;
        this.charts = {};
        this.data = {};
    }

    /**
     * Initialize dashboard with data
     */
    async init() {
        try {
            // Show loading state
            this.showLoading();
            
            // Load all data
            this.data = await this.loader.loadAllData();
            
            // Process and render components
            await this.renderAll();
            
            // Hide loading state
            this.hideLoading();
            
            console.log('Dashboard initialized successfully', this.data);
        } catch (error) {
            console.error('Failed to initialize dashboard:', error);
            this.showError('Failed to load dashboard data');
        }
    }

    /**
     * Render all dashboard components
     */
    async renderAll() {
        await Promise.all([
            this.renderSummaryStats(),
            this.renderCharts(),
            this.renderComplexTypes(),
            this.renderUnsafeFFI(),
            this.renderLifecycleTimeline(),
            this.renderVariableRelationships()
        ]);
    }

    /**
     * Render summary statistics
     */
    async renderSummaryStats() {
        const memoryStats = this.loader.processMemoryAnalysis(this.data.memory_analysis);
        const complexTypes = this.loader.processComplexTypes(this.data.complex_types);
        const unsafeFFI = this.loader.processUnsafeFFI(this.data.unsafe_ffi);

        // Update summary cards
        this.updateElement('total-complex-types', complexTypes.totalTypes);
        this.updateElement('total-allocations', this.loader.formatNumber(memoryStats.totalAllocations));
        this.updateElement('generic-type-count', complexTypes.genericTypes);
        this.updateElement('unsafe-ffi-count', unsafeFFI.totalOperations);

        // Update additional stats if elements exist
        this.updateElement('active-memory', this.loader.formatBytes(memoryStats.activeMemory));
        this.updateElement('peak-memory', this.loader.formatBytes(memoryStats.peakMemory));
        this.updateElement('memory-efficiency', `${(memoryStats.memoryEfficiency * 100).toFixed(1)}%`);
    }

    /**
     * Render charts
     */
    async renderCharts() {
        await Promise.all([
            this.renderComplexityChart(),
            this.renderMemoryDistributionChart(),
            this.renderPerformanceChart()
        ]);
    }

    /**
     * Render complexity distribution chart
     */
    async renderComplexityChart() {
        const complexTypes = this.loader.processComplexTypes(this.data.complex_types);
        
        const chartData = {
            labels: ['Struct Types', 'Enum Types', 'Generic Types', 'Other Types'],
            datasets: [{
                data: [
                    complexTypes.structTypes,
                    complexTypes.enumTypes, 
                    complexTypes.genericTypes,
                    Math.max(0, complexTypes.totalTypes - complexTypes.structTypes - complexTypes.enumTypes - complexTypes.genericTypes)
                ],
                backgroundColor: this.loader.generateColors(4),
                borderWidth: 2,
                borderColor: '#ffffff'
            }]
        };

        this.renderChart('complexity-chart', 'doughnut', chartData, {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    position: 'bottom'
                }
            }
        });
    }

    /**
     * Render memory distribution chart
     */
    async renderMemoryDistributionChart() {
        const memoryStats = this.loader.processMemoryAnalysis(this.data.memory_analysis);
        
        const typeEntries = Object.entries(memoryStats.memoryByType)
            .sort((a, b) => b[1].size - a[1].size)
            .slice(0, 8); // Top 8 types

        const chartData = {
            labels: typeEntries.map(([type]) => type || 'Unknown'),
            datasets: [{
                data: typeEntries.map(([, data]) => data.size),
                backgroundColor: this.loader.generateColors(typeEntries.length),
                borderWidth: 2,
                borderColor: '#ffffff'
            }]
        };

        this.renderChart('memory-distribution-chart', 'pie', chartData, {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    position: 'bottom'
                },
                tooltip: {
                    callbacks: {
                        label: (context) => {
                            const label = context.label || '';
                            const value = this.loader.formatBytes(context.parsed);
                            return `${label}: ${value}`;
                        }
                    }
                }
            }
        });
    }

    /**
     * Render performance chart
     */
    async renderPerformanceChart() {
        const performance = this.loader.processPerformance(this.data.performance);
        
        const chartData = {
            labels: ['Allocation Rate', 'Deallocation Rate', 'Memory Throughput'],
            datasets: [{
                label: 'Performance Metrics',
                data: [
                    performance.allocationRate,
                    performance.deallocationRate,
                    performance.memoryThroughput / 1000 // Convert to KB for better scale
                ],
                backgroundColor: ['#3B82F6', '#10B981', '#8B5CF6'],
                borderWidth: 2,
                borderColor: '#ffffff'
            }]
        };

        this.renderChart('performance-chart', 'bar', chartData, {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    display: false
                }
            },
            scales: {
                y: {
                    beginAtZero: true
                }
            }
        });
    }

    /**
     * Render complex types section
     */
    async renderComplexTypes() {
        const complexTypes = this.loader.processComplexTypes(this.data.complex_types);
        const container = document.getElementById('complex-types-container');
        
        if (!container || !complexTypes.types.length) return;

        const html = complexTypes.types.map(type => `
            <div class="bg-white rounded-lg p-4 border border-gray-200 hover-lift">
                <div class="flex justify-between items-start mb-2">
                    <h4 class="font-semibold text-lg">${type.type_name || 'Unknown Type'}</h4>
                    <span class="px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded-full">
                        ${type.type_category || 'Unknown'}
                    </span>
                </div>
                <p class="text-gray-600 text-sm mb-2">${type.description || 'No description available'}</p>
                <div class="flex flex-wrap gap-2">
                    ${type.is_generic ? '<span class="px-2 py-1 bg-purple-100 text-purple-800 text-xs rounded-full">Generic</span>' : ''}
                    <span class="px-2 py-1 bg-gray-100 text-gray-800 text-xs rounded-full">
                        Size: ${type.size_bytes || 0} bytes
                    </span>
                </div>
            </div>
        `).join('');

        container.innerHTML = html;
    }

    /**
     * Render unsafe FFI section
     */
    async renderUnsafeFFI() {
        const unsafeFFI = this.loader.processUnsafeFFI(this.data.unsafe_ffi);
        const container = document.getElementById('ffi-data-render');
        
        if (!container || !unsafeFFI.operations.length) {
            container.innerHTML = '<p class="text-gray-500">No unsafe FFI operations detected.</p>';
            return;
        }

        const html = unsafeFFI.operations.map((op, index) => `
            <div class="border border-gray-200 rounded-lg mb-4">
                <div class="p-4 bg-gray-50 cursor-pointer flex justify-between items-center" 
                     onclick="toggleCollapsible(this)">
                    <div>
                        <h4 class="font-semibold">${op.operation_type || 'Unknown Operation'}</h4>
                        <p class="text-sm text-gray-600">${op.location || 'Unknown location'}</p>
                    </div>
                    <i class="fa fa-chevron-down rotate-icon"></i>
                </div>
                <div class="collapsible-content p-4">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <h5 class="font-medium mb-2">Operation Details</h5>
                            <p class="text-sm"><strong>Type:</strong> ${op.operation_type || 'N/A'}</p>
                            <p class="text-sm"><strong>Safety Level:</strong> ${op.safety_level || 'N/A'}</p>
                            <p class="text-sm"><strong>Risk Score:</strong> ${op.risk_score || 'N/A'}</p>
                        </div>
                        <div>
                            <h5 class="font-medium mb-2">Context</h5>
                            <p class="text-sm"><strong>Function:</strong> ${op.function_name || 'N/A'}</p>
                            <p class="text-sm"><strong>Module:</strong> ${op.module_path || 'N/A'}</p>
                            <p class="text-sm"><strong>Line:</strong> ${op.line_number || 'N/A'}</p>
                        </div>
                    </div>
                    ${op.description ? `<div class="mt-4"><h5 class="font-medium mb-2">Description</h5><p class="text-sm text-gray-700">${op.description}</p></div>` : ''}
                </div>
            </div>
        `).join('');

        container.innerHTML = html;
    }

    /**
     * Render lifecycle timeline
     */
    async renderLifecycleTimeline() {
        const memoryStats = this.loader.processMemoryAnalysis(this.data.memory_analysis);
        const container = document.getElementById('lifecycle-timeline');
        
        if (!container || !this.data.memory_analysis?.allocations) return;

        // Sample recent allocations for timeline
        const recentAllocations = this.data.memory_analysis.allocations
            .filter(alloc => alloc.timestamp_alloc)
            .sort((a, b) => b.timestamp_alloc - a.timestamp_alloc)
            .slice(0, 10);

        const html = recentAllocations.map(alloc => {
            const allocTime = new Date(alloc.timestamp_alloc / 1000000); // Convert from nanoseconds
            const isActive = !alloc.timestamp_dealloc;
            
            return `
                <div class="timeline-item pl-8 pb-6 relative">
                    <div class="timeline-item::before bg-${isActive ? 'green' : 'gray'}-500"></div>
                    <div class="timeline-line bg-gray-300"></div>
                    <div class="bg-white rounded-lg p-4 shadow-sm border">
                        <div class="flex justify-between items-start mb-2">
                            <h4 class="font-semibold">${alloc.var_name || alloc.type_name || 'Unknown Variable'}</h4>
                            <span class="px-2 py-1 bg-${isActive ? 'green' : 'gray'}-100 text-${isActive ? 'green' : 'gray'}-800 text-xs rounded-full">
                                ${isActive ? 'Active' : 'Deallocated'}
                            </span>
                        </div>
                        <p class="text-sm text-gray-600 mb-2">Size: ${this.loader.formatBytes(alloc.size)}</p>
                        <p class="text-xs text-gray-500">Allocated: ${allocTime.toLocaleString()}</p>
                        ${alloc.scope_name ? `<p class="text-xs text-gray-500">Scope: ${alloc.scope_name}</p>` : ''}
                    </div>
                </div>
            `;
        }).join('');

        container.innerHTML = html;
    }

    /**
     * Render variable relationships (placeholder)
     */
    async renderVariableRelationships() {
        const container = document.getElementById('variable-relationships-svg');
        if (!container) return;

        // Simple placeholder SVG
        container.innerHTML = `
            <svg width="100%" height="400" viewBox="0 0 800 400">
                <text x="400" y="200" text-anchor="middle" class="text-gray-500">
                    Variable relationship visualization will be rendered here
                </text>
            </svg>
        `;
    }

    /**
     * Helper method to render Chart.js charts
     */
    renderChart(canvasId, type, data, options = {}) {
        const canvas = document.getElementById(canvasId);
        if (!canvas) return;

        // Destroy existing chart if it exists
        if (this.charts[canvasId]) {
            this.charts[canvasId].destroy();
        }

        const ctx = canvas.getContext('2d');
        this.charts[canvasId] = new Chart(ctx, {
            type,
            data,
            options: {
                responsive: true,
                maintainAspectRatio: false,
                ...options
            }
        });
    }

    /**
     * Helper method to update element content
     */
    updateElement(id, content) {
        const element = document.getElementById(id);
        if (element) {
            element.textContent = content;
        }
    }

    /**
     * Show loading state
     */
    showLoading() {
        const loadingHtml = `
            <div id="dashboard-loading" class="fixed inset-0 bg-white bg-opacity-90 flex items-center justify-center z-50">
                <div class="text-center">
                    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4"></div>
                    <p class="text-gray-600">Loading dashboard data...</p>
                </div>
            </div>
        `;
        document.body.insertAdjacentHTML('beforeend', loadingHtml);
    }

    /**
     * Hide loading state
     */
    hideLoading() {
        const loading = document.getElementById('dashboard-loading');
        if (loading) {
            loading.remove();
        }
    }

    /**
     * Show error message
     */
    showError(message) {
        this.hideLoading();
        const errorHtml = `
            <div id="dashboard-error" class="fixed inset-0 bg-white bg-opacity-90 flex items-center justify-center z-50">
                <div class="text-center max-w-md">
                    <i class="fa fa-exclamation-triangle text-red-500 text-4xl mb-4"></i>
                    <h3 class="text-xl font-semibold mb-2">Dashboard Error</h3>
                    <p class="text-gray-600 mb-4">${message}</p>
                    <button onclick="location.reload()" class="px-4 py-2 bg-primary text-white rounded hover:bg-blue-600">
                        Retry
                    </button>
                </div>
            </div>
        `;
        document.body.insertAdjacentHTML('beforeend', errorHtml);
    }
}

// Global toggle function for collapsible content
function toggleCollapsible(element) {
    const content = element.nextElementSibling;
    const icon = element.querySelector('.rotate-icon');
    
    content.classList.toggle('active');
    icon.classList.toggle('active');
}

// Global instance
window.memScopeDashboardRenderer = new MemScopeDashboardRenderer();