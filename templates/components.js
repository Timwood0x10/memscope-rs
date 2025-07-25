/**
 * UI Components for MemScope Analysis Dashboard
 * Modular components for different sections of the dashboard
 */

class MemScopeComponents {
    constructor(dataLoader) {
        this.dataLoader = dataLoader;
    }

    /**
     * Generate Overview Tab Content
     */
    generateOverviewContent() {
        const stats = this.dataLoader.getMemoryStats();
        const typeDistribution = this.dataLoader.getTypeDistribution();
        const recentAllocations = this.dataLoader.getRecentAllocations(5);
        const insights = this.dataLoader.getPerformanceInsights();
        const unsafeStats = this.dataLoader.getUnsafeStats();

        return `
            <!-- Statistics Cards -->
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                <div class="bg-white rounded-lg shadow-sm p-6 hover-lift">
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div class="w-8 h-8 bg-primary rounded-md flex items-center justify-center">
                                <i class="fa fa-memory text-white"></i>
                            </div>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">Total Memory</p>
                            <p class="text-2xl font-semibold text-gray-900">${this.dataLoader.formatBytes(stats.totalMemory)}</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-sm p-6 hover-lift">
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div class="w-8 h-8 bg-secondary rounded-md flex items-center justify-center">
                                <i class="fa fa-list text-white"></i>
                            </div>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">Total Allocations</p>
                            <p class="text-2xl font-semibold text-gray-900">${stats.totalAllocations.toLocaleString()}</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-sm p-6 hover-lift">
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div class="w-8 h-8 bg-accent rounded-md flex items-center justify-center">
                                <i class="fa fa-check-circle text-white"></i>
                            </div>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">Active Allocations</p>
                            <p class="text-2xl font-semibold text-gray-900">${stats.activeAllocations.toLocaleString()}</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-sm p-6 hover-lift">
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div class="w-8 h-8 bg-${unsafeStats.riskLevel === 'high' ? 'ffi-red' : unsafeStats.riskLevel === 'medium' ? 'safe-yellow' : 'safe-green'} rounded-md flex items-center justify-center">
                                <i class="fa fa-shield text-white"></i>
                            </div>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">Safety Level</p>
                            <p class="text-2xl font-semibold text-gray-900 capitalize">${unsafeStats.riskLevel}</p>
                        </div>
                    </div>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                <!-- Type Distribution Chart -->
                <div class="bg-white rounded-lg shadow-sm p-6">
                    <h3 class="text-lg font-semibold text-gray-900 mb-4">Type Distribution</h3>
                    <div class="h-64">
                        <canvas id="typeDistributionChart"></canvas>
                    </div>
                </div>

                <!-- Recent Allocations -->
                <div class="bg-white rounded-lg shadow-sm p-6">
                    <h3 class="text-lg font-semibold text-gray-900 mb-4">Recent Allocations</h3>
                    <div class="space-y-3">
                        ${recentAllocations.map(alloc => `
                            <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                                <div class="flex-1">
                                    <p class="text-sm font-medium text-gray-900">${alloc.type_name || 'Unknown Type'}</p>
                                    <p class="text-xs text-gray-500">${alloc.ptr}</p>
                                </div>
                                <div class="text-right">
                                    <p class="text-sm font-medium text-gray-900">${alloc.formattedSize}</p>
                                    <p class="text-xs text-gray-500">${alloc.formattedTime}</p>
                                </div>
                            </div>
                        `).join('')}
                    </div>
                </div>
            </div>

            <!-- Performance Insights -->
            ${insights.length > 0 ? `
                <div class="mt-8 bg-white rounded-lg shadow-sm p-6">
                    <h3 class="text-lg font-semibold text-gray-900 mb-4">Performance Insights</h3>
                    <div class="space-y-3">
                        ${insights.map(insight => `
                            <div class="flex items-start p-4 rounded-lg ${
                                insight.type === 'warning' ? 'bg-yellow-50 border border-yellow-200' : 'bg-blue-50 border border-blue-200'
                            }">
                                <div class="flex-shrink-0">
                                    <i class="fa fa-${insight.type === 'warning' ? 'exclamation-triangle text-yellow-600' : 'info-circle text-blue-600'}"></i>
                                </div>
                                <div class="ml-3">
                                    <h4 class="text-sm font-medium text-gray-900">${insight.title}</h4>
                                    <p class="text-sm text-gray-700 mt-1">${insight.message}</p>
                                </div>
                            </div>
                        `).join('')}
                    </div>
                </div>
            ` : ''}
        `;
    }

    /**
     * Generate Memory Analysis Tab Content
     */
    generateMemoryContent() {
        return `
            <div class="bg-white rounded-lg shadow-sm p-6">
                <h3 class="text-lg font-semibold text-gray-900 mb-4">Memory Analysis Visualization</h3>
                <div class="graph-container bg-gray-50 rounded-lg p-4">
                    <div id="memory-visualization" class="w-full h-96 flex items-center justify-center">
                        <p class="text-gray-500">Memory visualization will be rendered here</p>
                    </div>
                </div>
            </div>

            <div class="mt-8 grid grid-cols-1 md:grid-cols-2 gap-6">
                <div class="bg-white rounded-lg shadow-sm p-6">
                    <h4 class="text-md font-semibold text-gray-900 mb-3">Memory Distribution</h4>
                    <canvas id="memoryDistributionChart" class="w-full h-48"></canvas>
                </div>
                
                <div class="bg-white rounded-lg shadow-sm p-6">
                    <h4 class="text-md font-semibold text-gray-900 mb-3">Allocation Timeline</h4>
                    <canvas id="allocationTimelineChart" class="w-full h-48"></canvas>
                </div>
            </div>
        `;
    }

    /**
     * Generate Lifecycle Timeline Tab Content
     */
    generateLifecycleContent() {
        return `
            <div class="bg-white rounded-lg shadow-sm p-6">
                <h3 class="text-lg font-semibold text-gray-900 mb-4">Lifecycle Timeline</h3>
                <div class="graph-container bg-gray-50 rounded-lg p-4">
                    <div id="lifecycle-visualization" class="w-full h-96 flex items-center justify-center">
                        <p class="text-gray-500">Lifecycle timeline will be rendered here</p>
                    </div>
                </div>
            </div>

            <div class="mt-8 bg-white rounded-lg shadow-sm p-6">
                <h4 class="text-md font-semibold text-gray-900 mb-3">Scope Analysis</h4>
                <div id="scope-analysis" class="space-y-4">
                    <!-- Scope analysis content will be populated here -->
                </div>
            </div>
        `;
    }

    /**
     * Generate Unsafe/FFI Tab Content
     */
    generateFFIContent() {
        const unsafeStats = this.dataLoader.getUnsafeStats();

        if (!unsafeStats.hasUnsafeOperations) {
            return `
                <div class="bg-white rounded-lg shadow-sm p-8 text-center">
                    <div class="w-16 h-16 bg-safe-green rounded-full flex items-center justify-center mx-auto mb-4">
                        <i class="fa fa-shield text-white text-2xl"></i>
                    </div>
                    <h3 class="text-lg font-semibold text-gray-900 mb-2">No Unsafe Operations Detected</h3>
                    <p class="text-gray-600">Your code appears to be using only safe Rust constructs. Great job!</p>
                </div>
            `;
        }

        return `
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
                <div class="bg-white rounded-lg shadow-sm p-6">
                    <div class="flex items-center">
                        <div class="w-8 h-8 bg-ffi-red rounded-md flex items-center justify-center">
                            <i class="fa fa-exclamation-triangle text-white"></i>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">Unsafe Blocks</p>
                            <p class="text-2xl font-semibold text-gray-900">${unsafeStats.unsafeBlocks}</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-sm p-6">
                    <div class="flex items-center">
                        <div class="w-8 h-8 bg-safe-yellow rounded-md flex items-center justify-center">
                            <i class="fa fa-link text-white"></i>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">FFI Calls</p>
                            <p class="text-2xl font-semibold text-gray-900">${unsafeStats.ffiCalls}</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-sm p-6">
                    <div class="flex items-center">
                        <div class="w-8 h-8 bg-${unsafeStats.riskLevel === 'high' ? 'ffi-red' : unsafeStats.riskLevel === 'medium' ? 'safe-yellow' : 'safe-green'} rounded-md flex items-center justify-center">
                            <i class="fa fa-shield text-white"></i>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">Risk Level</p>
                            <p class="text-2xl font-semibold text-gray-900 capitalize">${unsafeStats.riskLevel}</p>
                        </div>
                    </div>
                </div>
            </div>

            <div class="bg-white rounded-lg shadow-sm p-6">
                <h3 class="text-lg font-semibold text-gray-900 mb-4">Unsafe/FFI Analysis</h3>
                <div class="graph-container bg-gray-50 rounded-lg p-4">
                    <div id="ffi-visualization" class="w-full h-96 flex items-center justify-center">
                        <p class="text-gray-500">Unsafe/FFI visualization will be rendered here</p>
                    </div>
                </div>
            </div>
        `;
    }

    /**
     * Generate Explorer Tab Content
     */
    generateExplorerContent() {
        return `
            <!-- Search and Filter Controls -->
            <div class="bg-white rounded-lg shadow-sm p-6 mb-6">
                <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">Search</label>
                        <input type="text" id="search-input" placeholder="Search allocations..." 
                               class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary">
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">Type Filter</label>
                        <select id="type-filter" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary">
                            <option value="">All Types</option>
                        </select>
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">Sort By</label>
                        <select id="sort-select" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary">
                            <option value="timestamp">Timestamp</option>
                            <option value="size">Size</option>
                            <option value="type">Type</option>
                        </select>
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">Size Range</label>
                        <input type="range" id="size-range" min="0" max="100" value="100" 
                               class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer">
                        <div class="flex justify-between text-xs text-gray-500 mt-1">
                            <span>0</span>
                            <span id="size-range-value">All</span>
                        </div>
                    </div>
                </div>
                <div class="mt-4 flex items-center">
                    <label class="flex items-center">
                        <input type="checkbox" id="active-only" class="rounded border-gray-300 text-primary focus:ring-primary">
                        <span class="ml-2 text-sm text-gray-700">Show only active allocations</span>
                    </label>
                </div>
            </div>

            <!-- Results -->
            <div class="bg-white rounded-lg shadow-sm p-6">
                <div class="flex justify-between items-center mb-4">
                    <h3 class="text-lg font-semibold text-gray-900">Allocation Explorer</h3>
                    <span id="results-count" class="text-sm text-gray-500">Loading...</span>
                </div>
                <div id="allocation-grid" class="space-y-3">
                    <!-- Results will be populated here -->
                </div>
                <div id="load-more" class="mt-6 text-center hidden">
                    <button class="bg-primary text-white px-6 py-2 rounded-lg hover:bg-blue-600 transition-colors">
                        Load More
                    </button>
                </div>
            </div>
        `;
    }

    /**
     * Initialize charts for overview tab
     */
    initializeOverviewCharts() {
        this.initializeTypeDistributionChart();
    }

    /**
     * Initialize type distribution chart
     */
    initializeTypeDistributionChart() {
        const canvas = document.getElementById('typeDistributionChart');
        if (!canvas) return;

        const ctx = canvas.getContext('2d');
        const typeDistribution = this.dataLoader.getTypeDistribution();

        new Chart(ctx, {
            type: 'doughnut',
            data: {
                labels: typeDistribution.slice(0, 8).map(item => item.type),
                datasets: [{
                    data: typeDistribution.slice(0, 8).map(item => item.count),
                    backgroundColor: [
                        '#3B82F6', '#10B981', '#8B5CF6', '#F59E0B',
                        '#EF4444', '#6B7280', '#EC4899', '#14B8A6'
                    ],
                    borderWidth: 2,
                    borderColor: '#ffffff'
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

// Export for global use
window.MemScopeComponents = MemScopeComponents;