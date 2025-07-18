// MemScope-RS Interactive JavaScript
// Handles all client-side interactions and data visualization

class MemScopeVisualizer {
    constructor(data) {
        this.data = data;
        this.filteredAllocations = [...data.allocations];
        this.init();
    }

    init() {
        this.setupTabNavigation();
        this.populateOverview();
        this.setupInteractiveExplorer();
        this.updateHeaderStats();
    }

    // Tab Navigation System
    setupTabNavigation() {
        const tabButtons = document.querySelectorAll('.tab-btn');
        const tabContents = document.querySelectorAll('.tab-content');

        tabButtons.forEach(button => {
            button.addEventListener('click', () => {
                const targetTab = button.getAttribute('data-tab');
                
                // Update active tab button
                tabButtons.forEach(btn => btn.classList.remove('active'));
                button.classList.add('active');
                
                // Update active tab content
                tabContents.forEach(content => content.classList.remove('active'));
                document.getElementById(targetTab).classList.add('active');
                
                // Trigger tab-specific updates
                this.onTabChange(targetTab);
            });
        });
    }

    onTabChange(tabName) {
        switch(tabName) {
            case 'overview':
                this.populateOverview();
                break;
            case 'interactive':
                this.updateInteractiveExplorer();
                break;
        }
    }

    // Header Statistics
    updateHeaderStats() {
        const stats = this.data.stats;
        
        document.getElementById('totalMemory').textContent = 
            `📊 ${this.formatBytes(stats.current_memory)}`;
        document.getElementById('activeAllocs').textContent = 
            `🔢 ${stats.active_allocations.toLocaleString()} allocs`;
        document.getElementById('peakMemory').textContent = 
            `📈 Peak: ${this.formatBytes(stats.peak_memory)}`;
    }

    // Overview Tab Population
    populateOverview() {
        this.populateMemoryStats();
        this.populateTypeDistribution();
        this.populateRecentAllocations();
        this.populatePerformanceInsights();
    }

    populateMemoryStats() {
        const stats = this.data.stats;
        const container = document.getElementById('memoryStats');
        
        const memoryUtilization = (stats.current_memory / stats.peak_memory * 100).toFixed(1);
        
        container.innerHTML = `
            <div class="memory-stat">
                <span class="stat-label">Current Memory</span>
                <span class="stat-value">${this.formatBytes(stats.current_memory)}</span>
            </div>
            <div class="memory-stat">
                <span class="stat-label">Peak Memory</span>
                <span class="stat-value">${this.formatBytes(stats.peak_memory)}</span>
            </div>
            <div class="memory-stat">
                <span class="stat-label">Memory Utilization</span>
                <span class="stat-value">${memoryUtilization}%</span>
            </div>
            <div class="memory-stat">
                <span class="stat-label">Active Allocations</span>
                <span class="stat-value">${stats.active_allocations.toLocaleString()}</span>
            </div>
            <div class="memory-stat">
                <span class="stat-label">Total Allocations</span>
                <span class="stat-value">${stats.total_allocations.toLocaleString()}</span>
            </div>
        `;
    }

    populateTypeDistribution() {
        const container = document.getElementById('typeDistribution');
        const typeMap = new Map();
        
        // Aggregate by type
        this.data.allocations.forEach(alloc => {
            const typeName = alloc.type_name || 'Unknown';
            if (!typeMap.has(typeName)) {
                typeMap.set(typeName, { size: 0, count: 0 });
            }
            const current = typeMap.get(typeName);
            current.size += alloc.size;
            current.count += 1;
        });
        
        // Sort by size and take top 10
        const sortedTypes = Array.from(typeMap.entries())
            .sort((a, b) => b[1].size - a[1].size)
            .slice(0, 10);
        
        container.innerHTML = sortedTypes.map(([typeName, data]) => `
            <div class="type-item">
                <span class="type-name">${this.truncateText(typeName, 25)}</span>
                <div class="type-stats">
                    <span class="type-size">${this.formatBytes(data.size)}</span>
                    <span class="type-count">${data.count} allocs</span>
                </div>
            </div>
        `).join('');
    }

    populateRecentAllocations() {
        const container = document.getElementById('recentAllocations');
        
        // Sort by timestamp and take most recent 8
        const recentAllocs = [...this.data.allocations]
            .filter(alloc => alloc.var_name) // Only show named variables
            .sort((a, b) => b.timestamp - a.timestamp)
            .slice(0, 8);
        
        if (recentAllocs.length === 0) {
            container.innerHTML = '<p style="color: #64748b; font-style: italic;">No named variables found</p>';
            return;
        }
        
        container.innerHTML = recentAllocs.map(alloc => `
            <div class="type-item">
                <span class="type-name">${alloc.var_name}</span>
                <div class="type-stats">
                    <span class="type-size">${this.formatBytes(alloc.size)}</span>
                    <span class="type-count">${alloc.type_name || 'Unknown'}</span>
                </div>
            </div>
        `).join('');
    }

    populatePerformanceInsights() {
        const container = document.getElementById('performanceInsights');
        const insights = this.generateInsights();
        
        container.innerHTML = insights.map(insight => `
            <div class="insight-item">
                <div class="insight-title">${insight.title}</div>
                <div class="insight-description">${insight.description}</div>
            </div>
        `).join('');
    }

    generateInsights() {
        const insights = [];
        const stats = this.data.stats;
        const allocations = this.data.allocations;
        
        // Memory utilization insight
        const utilization = (stats.current_memory / stats.peak_memory * 100);
        if (utilization > 80) {
            insights.push({
                title: "🔴 High Memory Utilization",
                description: `Current memory usage is ${utilization.toFixed(1)}% of peak. Consider optimizing memory usage.`
            });
        } else if (utilization < 30) {
            insights.push({
                title: "🟢 Efficient Memory Usage",
                description: `Memory utilization is low at ${utilization.toFixed(1)}%. Good memory management!`
            });
        }
        
        // Large allocations insight
        const largeAllocs = allocations.filter(a => a.size > 1024 * 1024); // > 1MB
        if (largeAllocs.length > 0) {
            insights.push({
                title: "📊 Large Allocations Detected",
                description: `Found ${largeAllocs.length} allocation(s) larger than 1MB. Review if necessary.`
            });
        }
        
        // Type diversity insight
        const uniqueTypes = new Set(allocations.map(a => a.type_name).filter(Boolean));
        insights.push({
            title: "🏷️ Type Diversity",
            description: `Using ${uniqueTypes.size} different types across ${allocations.length} allocations.`
        });
        
        // Unsafe/FFI insight
        if (this.data.unsafeFFI && this.data.unsafeFFI.violations.length > 0) {
            insights.push({
                title: "⚠️ Safety Violations",
                description: `Detected ${this.data.unsafeFFI.violations.length} safety violation(s). Review unsafe code carefully.`
            });
        } else if (this.data.unsafeFFI) {
            insights.push({
                title: "✅ No Safety Issues",
                description: "No memory safety violations detected in unsafe/FFI code."
            });
        }
        
        return insights;
    }

    // Interactive Explorer Setup
    setupInteractiveExplorer() {
        this.populateTypeFilter();
        this.setupEventListeners();
        this.updateInteractiveExplorer();
    }

    populateTypeFilter() {
        const select = document.getElementById('filterType');
        const types = new Set(this.data.allocations.map(a => a.type_name).filter(Boolean));
        
        select.innerHTML = '<option value="">All Types</option>' +
            Array.from(types).sort().map(type => 
                `<option value="${type}">${this.truncateText(type, 30)}</option>`
            ).join('');
    }

    setupEventListeners() {
        document.getElementById('filterType').addEventListener('change', () => this.updateFilters());
        document.getElementById('sizeRange').addEventListener('input', () => this.updateFilters());
        document.getElementById('sortBy').addEventListener('change', () => this.updateInteractiveExplorer());
    }

    updateFilters() {
        const typeFilter = document.getElementById('filterType').value;
        const sizeRange = document.getElementById('sizeRange').value;
        const maxSize = Math.max(...this.data.allocations.map(a => a.size));
        const sizeThreshold = (maxSize * sizeRange) / 100;
        
        // Update size range display
        document.getElementById('sizeRangeValue').textContent = 
            sizeRange == 100 ? 'All sizes' : `≤ ${this.formatBytes(sizeThreshold)}`;
        
        // Apply filters
        this.filteredAllocations = this.data.allocations.filter(alloc => {
            const typeMatch = !typeFilter || alloc.type_name === typeFilter;
            const sizeMatch = alloc.size <= sizeThreshold;
            return typeMatch && sizeMatch;
        });
        
        this.updateInteractiveExplorer();
    }

    updateInteractiveExplorer() {
        const sortBy = document.getElementById('sortBy').value;
        
        // Sort allocations
        const sorted = [...this.filteredAllocations].sort((a, b) => {
            switch(sortBy) {
                case 'size':
                    return b.size - a.size;
                case 'timestamp':
                    return b.timestamp - a.timestamp;
                case 'type':
                    return (a.type_name || '').localeCompare(b.type_name || '');
                default:
                    return 0;
            }
        });
        
        this.renderAllocationGrid(sorted);
    }

    renderAllocationGrid(allocations) {
        const container = document.getElementById('allocationGrid');
        
        if (allocations.length === 0) {
            container.innerHTML = `
                <div style="grid-column: 1 / -1; text-align: center; padding: 40px; color: #64748b;">
                    <h3>No allocations match the current filters</h3>
                    <p>Try adjusting the filters to see more results.</p>
                </div>
            `;
            return;
        }
        
        container.innerHTML = allocations.slice(0, 100).map(alloc => `
            <div class="allocation-card" onclick="memscope.showAllocationDetails(${alloc.ptr})">
                <div class="allocation-header">
                    <span class="allocation-name">${alloc.var_name || `Ptr ${alloc.ptr.toString(16)}`}</span>
                    <span class="allocation-size">${this.formatBytes(alloc.size)}</span>
                </div>
                <div class="allocation-type">${alloc.type_name || 'Unknown Type'}</div>
                <div class="allocation-details">
                    <div>Address: 0x${alloc.ptr.toString(16)}</div>
                    <div>Timestamp: ${new Date(alloc.timestamp / 1000000).toLocaleString()}</div>
                    ${alloc.call_stack && alloc.call_stack.length > 0 ? 
                        `<div>Stack depth: ${alloc.call_stack.length} frames</div>` : ''}
                </div>
            </div>
        `).join('');
        
        // Show count info
        if (allocations.length > 100) {
            container.innerHTML += `
                <div style="grid-column: 1 / -1; text-align: center; padding: 20px; color: #64748b; font-style: italic;">
                    Showing first 100 of ${allocations.length} allocations
                </div>
            `;
        }
    }

    showAllocationDetails(ptr) {
        const alloc = this.data.allocations.find(a => a.ptr === ptr);
        if (!alloc) return;
        
        const details = `
            Variable: ${alloc.var_name || 'Unnamed'}
            Type: ${alloc.type_name || 'Unknown'}
            Size: ${this.formatBytes(alloc.size)}
            Address: 0x${alloc.ptr.toString(16)}
            Timestamp: ${new Date(alloc.timestamp / 1000000).toLocaleString()}
            
            Call Stack:
            ${alloc.call_stack ? alloc.call_stack.map((frame, i) => 
                `  ${i + 1}. ${frame.function_name || 'unknown'} (${frame.file_name || 'unknown'}:${frame.line_number || '?'})`
            ).join('\n') : 'No call stack available'}
        `;
        
        alert(details); // Simple popup for now, could be enhanced with a modal
    }

    // Utility Functions
    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    }

    truncateText(text, maxLength) {
        if (!text) return 'Unknown';
        return text.length > maxLength ? text.substring(0, maxLength) + '...' : text;
    }
}

// Initialize when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    // Global instance for easy access
    window.memscope = new MemScopeVisualizer(MEMORY_DATA);
    
    // Add some debug info to console
    console.log('🔍 MemScope-RS Interactive Visualizer Loaded');
    console.log('📊 Data Summary:', {
        allocations: MEMORY_DATA.allocations.length,
        totalMemory: window.memscope.formatBytes(MEMORY_DATA.stats.current_memory),
        hasUnsafeFFI: !!MEMORY_DATA.unsafeFFI,
        timestamp: MEMORY_DATA.timestamp
    });
});